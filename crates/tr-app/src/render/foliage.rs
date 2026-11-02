//! `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the
//! transparent marker tiles of a tree, their `GetWindCycle` sway
//! (`TileDrawing.cs:8137-8245`), and the ambient leaf gores of
//! `TileDrawing.EmitTreeLeaves` (`TileDrawing.cs:7779-7891`) with the
//! `Gore_UpdateLeaf` fall behaviour (`Gore.cs:1085-1209`).
use crate::assets;
use glam::Vec2;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tile_light::FastRandom;
use tr_core::tiledata as td;
use tr_core::wind;
use tr_core::{TILE_PX, World, tree};
use tr_render::SpriteRenderer;

/// `TileDrawing.EmitTreeLeaves`' spawn, resolved at draw time by the tree scan.
pub struct LeafSpawn {
    /// World-pixel spawn point (the vector of `TileDrawing.cs:7835-7886`).
    pub pos: Vec2,
    /// The gore id (`WorldGen.GetTreeLeaf`'s `passStyle`).
    pub style: u32,
}

/// One ambient leaf: the port of a leaf gore (`GoreID` 910-925 &c.,
/// `SpecialAI = 3`). `phase` is the gore's `velocity.Y` — a growing angle whose
/// sine drives the horizontal sway — and `vel_x` the rest-phase drift.
pub struct LeafGore {
    pub pos: Vec2,
    /// `velocity.Y`: the sway angle; >= 0 falling, < 0 at rest.
    phase: f32,
    /// `velocity.X`: the rest-phase drift (decays ×0.94 per tick).
    vel_x: f32,
    /// The sheet row: `SpriteFrame(32, 8).CurrentRow = rand(8)`.
    row: u8,
    /// `0.7f + Main.rand.NextFloat() * 0.6f` (`TileDrawing.cs:7889`).
    scale: f32,
    /// `rotation = vector4.ToRotation() + PI/2` (`Gore.cs:1200`).
    rot: f32,
    /// `Gore.goreTime = 600` (`Gore.cs:1350-1353`), then the alpha fade.
    time_left: i32,
    alpha: f32,
    /// The gore id — picks the `Gore_<id>` sheet.
    pub style: u32,
}

impl LeafGore {
    /// Builds one leaf from an `EmitTreeLeaves` spawn.
    pub(crate) fn from_spawn(spawn: LeafSpawn, rng: &mut FastRandom) -> Self {
        Self::new(spawn.pos, spawn.style, rng)
    }

    /// `Gore.NewGore`'s leaf init (`Gore.cs:1390-1401`): the velocity becomes
    /// `((rand - 0.5) * 1, rand * 2pi)` (the EmitTreeLeaves velocity argument
    /// is discarded for leaf gores), the frame row is random, and
    /// `sticky/timeLeft` = `Gore.goreTime` (600).
    pub(crate) fn new(pos: Vec2, style: u32, rng: &mut FastRandom) -> Self {
        Self {
            pos,
            phase: rng.next_float() * std::f32::consts::TAU,
            vel_x: 0.0,
            row: rng.next_range(0, 8) as u8,
            scale: 0.7 + rng.next_float() * 0.6,
            rot: 0.0,
            time_left: 600,
            alpha: 0.0,
            style,
        }
    }
}

/// `Gore_UpdateLeaf` (`Gore.cs:1085-1209`) for every leaf, one 60 Hz tick. The
/// falling phase sways the leaf along a growing angle with the wind added
/// above `worldSurface`; a solid or liquid probe switches it to the rest
/// phase, where it drifts with the wind until the ground below frees up or
/// the fade completes. The liquid-specific rest behaviour (honey freeze, lava
/// burn-up) is not ported — leaves rest on liquid surfaces instead.
pub fn tick_leaves(leaves: &mut Vec<LeafGore>, world: &World, wind_speed: f32) {
    for l in leaves.iter_mut() {
        let probe = [l.pos.x + 4.0, l.pos.y];
        if l.phase >= 0.0 {
            // The falling phase (`Gore.cs:1161-1208`).
            l.phase += core::f32::consts::PI / 180.0;
            let rotated = Vec2::new(-l.phase.sin(), l.phase.cos());
            let mut v4 = Vec2::new(rotated.x, rotated.y.abs());
            if l.pos.y < world.world_surface as f32 * 16.0 {
                v4.x += wind_speed * 4.0;
            }
            let pre = v4;
            // `Collision.TileCollision` + `Collision.SlopeCollision` on the 4x4
            // probe box; the position is the resolved probe position re-anchored
            // by `vector2 = (-4, 0)`, then moved by the residual velocity.
            let tc = crate::collision::tile_collision(world, probe, [v4.x, v4.y], 4.0, 4.0, false, false);
            v4 = Vec2::from(tc.vel);
            let sc = crate::collision::slope_collision(world, probe, [v4.x, v4.y], 4.0, 4.0, false);
            l.pos = Vec2::new(sc.pos[0] - 4.0, sc.pos[1]);
            v4 = Vec2::from(sc.vel);
            if v4 != pre {
                // `if (vector4 != vector5) velocity.Y = -1f` (`Gore.cs:1176`).
                l.phase = -1.0;
            }
            // The centre-in-liquid check (`Gore.cs:1180-1198`).
            let centre = (Vec2::new(6.0, 6.0) + l.pos) / 16.0;
            if world
                .tile(centre.x as i32, centre.y as i32)
                .is_some_and(|t| t.liquid > 0)
            {
                l.phase = -1.0;
            }
            l.pos += v4;
            l.rot = v4.y.atan2(v4.x) + std::f32::consts::FRAC_PI_2;
        } else {
            // The rest phase (`Gore.cs:1090-1159`).
            let point = ((Vec2::new(1.5, 1.5) + Vec2::from(probe)) / 16.0).floor();
            let probe_tile = world.tile(point.x as i32, point.y as i32);
            let in_liquid = probe_tile.is_some_and(|t| t.liquid > 0);
            if in_liquid {
                // `Gore.cs:1124-1131`: water drifts with the wind (frozen below
                // the surface); honey/lava are not modelled.
                l.vel_x = wind_speed;
            }
            if probe_tile
                .is_none_or(|t| !(t.active() && td::TILE_SOLID.has(t.kind)))
                && !in_liquid
            {
                // `Gore.cs:1133-1138`: the ground freed up — resume falling.
                l.phase = 0.1;
                l.time_left = 0;
                l.alpha += 20.0;
            }
            let tc = crate::collision::tile_collision(world, probe, [l.vel_x, -0.2], 3.0, 3.0, false, false);
            if in_liquid {
                l.rot = tc.vel[1].atan2(tc.vel[0]) + std::f32::consts::FRAC_PI_2;
            }
            l.vel_x = tc.vel[0] * 0.94;
            if !in_liquid || l.vel_x.abs() < 0.01 {
                l.vel_x = 0.0;
            }
            l.pos.x += l.vel_x;
        }
        // The fade (`Gore.cs:1055-1058`): 600 ticks, then 255 of alpha decay.
        if l.time_left > 0 {
            l.time_left -= 1;
        } else {
            l.alpha += 1.0;
        }
    }
    leaves.retain(|l| l.alpha < 255.0);
    if leaves.len() > 400 {
        leaves.drain(..leaves.len() - 400);
    }
}

/// The leaf draw: the `Gore_<id>` sheets are 32-column (paint) × 8-row
/// (variant) strips; the leaves draw in the always-rebuilt UI range like the
/// rain, so the G6 world range stays unaffected.
pub(crate) fn draw_leaves(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    leaves: &[LeafGore],
) -> u32 {
    let mut quads = 0;
    for l in leaves {
        let Some(uv) = lookups.leaf_uv(l.style, l.row) else {
            continue;
        };
        let c = light.get(
            (l.pos.x / TILE_PX) as i32,
            (l.pos.y / TILE_PX) as i32,
        );
        let a = (1.0 - l.alpha / 255.0).clamp(0.0, 1.0);
        let to_screen = |p: Vec2| {
            [
                (p.x - camera.x) * zoom + screen.x * 0.5,
                (p.y - camera.y) * zoom + screen.y * 0.5,
            ]
        };
        // The frame size comes from the sheet: 32 columns × 8 rows.
        let (fw, fh) = lookups.leaf_frame(l.style).unwrap_or((8.0, 8.0));
        r.push_ui_sprite_rot(
            to_screen(l.pos),
            [fw * l.scale * zoom, fh * l.scale * zoom],
            uv,
            [c[0], c[1], c[2], a],
            l.rot,
        );
        quads += 1;
    }
    quads
}

/// `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the transparent
/// marker tiles of a tree. The pass scans a slightly larger rect than the tile loop
/// because an 80x80 canopy sticks out above and beside its anchor. The sway is
/// `GetWindCycle(x, y, _treeWindCounter)` (`TileDrawing.cs:8137-8245`), skipped
/// when the marker tile sits in front of a wall (`flag`, line 8073), and every
/// top/branch rolls for an ambient leaf (`EmitTreeLeaves`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_tree_foliage(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    wind: &tr_core::wind::Wind,
    tree_counter: f32,
    rng: &mut FastRandom,
    spawns: &mut Vec<LeafSpawn>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> u32 {
    let fx0 = (x0 - 4).max(0);
    let fx1 = (x1 + 4).min(world.width - 1);
    let fy0 = (y0 - 5).max(0);
    let fy1 = (y1 + 1).min(world.height - 1);
    let mut quads = 0;
    for ty in fy0..=fy1 {
        for tx in fx0..=fx1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active() || !tree::is_tree(t.kind) {
                continue;
            }
            let Some(part) = tree::TreePart::marker(t.frame_x, t.frame_y) else {
                continue;
            };
            let f = tree::foliage_at(world, tx, ty, part, t.frame_y);
            if lookups.foliage.get(f.style).is_none_or(|s| s.is_none()) {
                continue;
            }
            let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
            let color = [c[0], c[1], c[2], 1.0];
            let px = tx as f32 * TILE_PX;
            let py = ty as f32 * TILE_PX;
            // `bool flag = tile.wall > 0` (`TileDrawing.cs:8073`): a tree in
            // front of a wall does not sway.
            let cycle = if t.wall > 0 {
                0.0
            } else {
                wind.cycle(tx, ty, tree_counter)
            };
            match part {
                tree::TreePart::Top => {
                    let sx = f.frame as i32 * (f.top_w as i32 + 2);
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, true, sx, 0, f.top_w as u32, f.top_h as u32)
                    {
                        // `TileDrawing.cs:8136-8149`: the top leans right with
                        // the sway, sinks by its magnitude, and rotates about
                        // the trunk anchor by `cycle * 0.08`.
                        let pos = [
                            px + 8.0 - f.top_w * 0.5 + cycle * 2.0,
                            py + 16.0 - f.top_h + cycle.abs() * 2.0,
                        ];
                        r.push_sprite_origin_rot(
                            pos,
                            [f.top_w, f.top_h],
                            uv,
                            color,
                            cycle * 0.08,
                            [f.top_w * 0.5, f.top_h],
                        );
                        quads += 1;
                        // `EmitTreeLeaves(x, y, x, floorY3)` — the top's grass
                        // column is its own (`TileDrawing.cs:8114-8121`).
                        emit_tree_leaves(world, wind, tx, ty, tx, rng, spawns);
                    }
                }
                tree::TreePart::BranchLeft => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 0, f.frame as i32 * 42, 40, 40)
                    {
                        // `TileDrawing.cs:8181-8197` (frameX == 44): the branch
                        // leans away from the trunk for positive sway and
                        // rotates about its trunk-side anchor by
                        // `cycle * 0.06`.
                        let mut pos = [px - 24.0, py - 12.0];
                        if cycle > 0.0 {
                            pos[0] += cycle;
                        }
                        pos[0] += cycle.abs() * 2.0;
                        r.push_sprite_origin_rot(
                            pos,
                            [40.0, 40.0],
                            uv,
                            color,
                            cycle * 0.06,
                            [40.0, 24.0],
                        );
                        quads += 1;
                        // `EmitTreeLeaves(x, y, x + 1, floorY2)` — the grass
                        // column is the trunk to the right
                        // (`TileDrawing.cs:8159-8166`).
                        emit_tree_leaves(world, wind, tx, ty, tx + 1, rng, spawns);
                    }
                }
                tree::TreePart::BranchRight => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 42, f.frame as i32 * 42, 40, 40)
                    {
                        // `TileDrawing.cs:8229-8245` (frameX == 66): mirrored —
                        // the branch leans left for negative sway, origin on
                        // its left edge.
                        let mut pos = [px, py - 12.0];
                        if cycle < 0.0 {
                            pos[0] += cycle;
                        }
                        pos[0] -= cycle.abs() * 2.0;
                        r.push_sprite_origin_rot(
                            pos,
                            [40.0, 40.0],
                            uv,
                            color,
                            cycle * 0.06,
                            [0.0, 30.0],
                        );
                        quads += 1;
                        // `EmitTreeLeaves(x, y, x - 1, floorY)` — the trunk to
                        // the left (`TileDrawing.cs:8207-8214`).
                        emit_tree_leaves(world, wind, tx, ty, tx - 1, rng, spawns);
                    }
                }
            }
        }
    }
    quads
}

/// `TileDrawing.EmitTreeLeaves` (`TileDrawing.cs:7779-7891`): the per-part roll
/// for one ambient leaf. `(tile_x, tile_y)` is the tree-part marker,
/// `grass_x` the trunk column (`x`, `x+1` for a frameX-44 branch and `x-1` for
/// a frameX-66 branch); the grass row is the ground tile the
/// `GetCommonTreeFoliageData` walk found.
fn emit_tree_leaves(
    world: &World,
    wind: &tr_core::wind::Wind,
    tile_x: i32,
    tile_y: i32,
    grass_x: i32,
    rng: &mut FastRandom,
    spawns: &mut Vec<LeafSpawn>,
) {
    let Some(marker) = world.tile(tile_x, tile_y) else {
        return;
    };
    if marker.liquid > 0 {
        // `TileDrawing.cs:7787-7790`.
        return;
    }
    // `WorldGen.GetTreeLeaf` (`WorldGen.cs:29384-29598`) with the ground tile of
    // the grass column (the 100-tile walk of `GetCommonTreeFoliageData`).
    let Some(ground) = ground_tile(world, grass_x, tile_y) else {
        return;
    };
    let (_, pass_style) = tree_leaf_pass_style(marker.kind, marker.frame_y, ground, grass_x);
    // `TileDrawing.cs:7793-7814`: mushroom/boreal/ash trees and unknown styles
    // never shed ambient leaves; hallowed trees (917-925, 1113-1121) shed half
    // as often.
    let hallowed = matches!(pass_style, 917..=925 | 1113..=1121);
    if pass_style == -1 || matches!(pass_style, 912 | 913 | 1278) {
        return;
    }
    // `TileDrawing.cs:7816-7833`: the frequency chain.
    let branch = tile_x != grass_x;
    let mut frequency = wind::leaf_frequency(wind.speed_current);
    if hallowed {
        frequency /= 2;
    }
    if !wind.blows_at_height(tile_y) {
        frequency = 10000;
    }
    if branch {
        frequency *= 3;
    }
    if rng.next_range(0, frequency) != 0 {
        return;
    }
    // The spawn point (`TileDrawing.cs:7834-7886`).
    let mut pos = Vec2::new(tile_x as f32 * 16.0 + 8.0, tile_y as f32 * 16.0 + 8.0);
    if branch {
        pos.x += (tile_x - grass_x) as f32 * 12.0;
        let variant = match marker.frame_y {
            220 => 1,
            242 => 2,
            _ => 0,
        };
        if marker.frame_x == 66 {
            pos += match variant {
                1 => Vec2::new(0.0, -6.0),
                2 => Vec2::new(0.0, 8.0),
                _ => Vec2::new(0.0, -6.0),
            };
        } else {
            pos += match variant {
                1 => Vec2::new(2.0, -6.0),
                2 => Vec2::new(6.0, -6.0),
                _ => Vec2::new(0.0, 4.0),
            };
        }
    } else {
        pos += Vec2::new(-16.0, -16.0);
        if hallowed {
            pos.y -= rng.next_range(0, 28) as f32 * 4.0;
        }
    }
    // `if (!WorldGen.SolidTile(vector.ToTileCoordinates()))`.
    let tx = (pos.x / 16.0).floor() as i32;
    let ty = (pos.y / 16.0).floor() as i32;
    let solid = world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
    if !solid {
        spawns.push(LeafSpawn {
            pos,
            style: pass_style as u32,
        });
    }
}

/// `WorldGen.GetCommonTreeFoliageData`'s downward walk
/// (`WorldGen.cs:62882-63025`): the first non-tree tile in the column.
fn ground_tile(world: &World, x: i32, y: i32) -> Option<u16> {
    for dy in 0..100 {
        let t = world.tile(x, y + dy)?;
        if t.active() && !tree::is_tree(t.kind) {
            return Some(t.kind);
        }
    }
    None
}

/// `WorldGen.GetTreeLeaf` (`WorldGen.cs:29384-29598`): the leaf gore id for a
/// tree top. Returns `(tree_frame_extra, pass_style)`; `tree_frame_extra` comes
/// from the marker's `frameY` (220 → 1, 242 → 2) plus the hallowed column
/// pattern, and shifts the branch spawn offsets.
fn tree_leaf_pass_style(top_kind: u16, frame_y: i16, ground: u16, x: i32) -> (i32, i32) {
    let mut tree_frame = match frame_y {
        220 => 1,
        242 => 2,
        _ => 0,
    };
    // `WorldGen.cs:29410-29434`: the top-tile type overrides (vanity/gem/ash).
    match top_kind {
        596 => return (tree_frame, 1248),
        616 => return (tree_frame, 1257),
        634 => return (tree_frame, 1278),
        583..=589 => return (tree_frame, 1249 + (top_kind - 583) as i32),
        _ => {}
    }
    let base = match ground {
        2 | 477 => 910,
        60 => 914,
        70 => 912,
        23 | 112 => 915,
        199 | 234 => 916,
        53 => 911,
        116 => 919,
        147 => 913,
        633 => 1278,
        109 | 492 => {
            // `WorldGen.cs:29467-29592`: the hallowed tables. `WorldGen
            // .hallowBG` is a generator property our worlds do not have; the
            // port's worlds never generate hallowed ground, so this arm is
            // data-complete but unreachable today.
            match x.rem_euclid(3) {
                1 => tree_frame += 3,
                2 => tree_frame += 6,
                _ => {}
            }
            let styles = [2, 1, 7, 4, 5, 6, 3, 8, 0];
            return (tree_frame, 917 + styles[tree_frame.clamp(0, 8) as usize]);
        }
        _ => return (tree_frame, -1),
    };
    (tree_frame, base)
}
