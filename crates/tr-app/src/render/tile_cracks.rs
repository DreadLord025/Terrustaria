//! The tile-crack overlay — `Main.DrawTileCracks` (`Main.cs:61533-61626`):
//! the mining-progress crack drawn on damaged tiles from the local player's
//! `HitTile` books. The solid-layer draw sites are
//! `TileDrawing.cs:732-733` (`hitReplace` first, then `hitTile`, crack type
//! 1); the wall sites are `WallDrawing.cs:203-204` (crack type 2).
//!
//! S1-G-1 wave C. The port's mining accumulator (`App::mine_target` +
//! `mine_timer`, `world_sim/editing.rs`) is projected into the C#'s
//! `HitTile` damage units (0..100 across the break interval) by
//! [`mining_crack_records`] — the draw pass itself is the verbatim
//! `DrawTileCracks` walk. Recorded deviations (ANALYSIS "The tile-crack
//! overlay (S1-G-1 wave C)"):
//! - the port has one mining accumulator, so at most one record draws (the
//!   C# keeps 500-entry `hitTile`/`hitReplace` books with 60-tick TTL decay);
//!   `hitReplace` draws nothing until replacement mining lands;
//! - `HitTileObject.Clear`'s `crackStyle` roll (`HitTile.cs:29-46`, a static
//!   `UnifiedRandom` + the no-repeat-of-`lastCrack` loop) is seeded from the
//!   world seed instead of `DateTime.Now.Ticks` (the port's determinism
//!   rule, same deviation class as `NamedRngs::global`);
//! - `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023-58030`) reads
//!   the echo monolith / `CanSeeInvisibleBlocks` — neither exists in the
//!   port, so it is the constant `false` and the port's `Tile` carries no
//!   invisible-block/wall bits (`Tile.cs:528`), making those conjuncts
//!   vacuously true;
//! - the draw rides the per-frame world range right after the cached tile
//!   pass (`frame/draw.rs`), the port's stand-in for the solid-layer site —
//!   the crack must redraw every frame while the damage timer advances, so
//!   it cannot live inside the G6-cached range (the `WorldFrameKey` is
//!   untouched).

use glam::Vec2;

use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tiledata::{SET_IS_A_TREE_TRUNK, TILE_SOLID, TILE_SOLID_TOP};
use tr_core::{TILE_PX, UnifiedRandom, World};
use tr_render::sprite::SpriteRenderer;

/// `HitTile.TILE` (`HitTile.cs:72`).
pub(crate) const CRACK_TILE: i32 = 1;
/// `HitTile.WALL` (`HitTile.cs:74`).
pub(crate) const CRACK_WALL: i32 = 2;

/// `HitTile.HitTileObject`'s draw inputs (`HitTile.cs:11-27`): the cell, the
/// accumulated damage, the book's hit type and the crack-style column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CrackRecord {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) damage: i32,
    /// `HitTileObject.type` — 1 = tile book, 2 = wall book.
    pub(crate) kind: i32,
    pub(crate) crack_style: i32,
}

/// `HitTileObject.Clear`'s style roll (`HitTile.cs:40-45`): `rand.Next(4)`
/// re-rolled while it equals `lastCrack`. The port seeds the stream from the
/// world seed (see the module deviations) and keeps the last style per book.
pub(crate) fn roll_crack_style(rng: &mut UnifiedRandom, last_crack: &mut i32) -> i32 {
    let mut style = rng.next_max(4);
    while style == *last_crack {
        style = rng.next_max(4);
    }
    *last_crack = style;
    style
}

/// The `flag3`/`flag4` tile-shape classification (`Main.cs:61571-61598`):
/// does the crack draw on this tile, and does it use the tree columns?
/// Returns `(draw, tree_style)`.
pub(crate) fn crack_tile_shape(kind: u16, frame_x: i32, frame_y: i32) -> (bool, bool) {
    let mut flag3 = false;
    let mut flag4 = false;
    if kind == 10 {
        // The door never cracks (`:61573-61576`).
        flag3 = false;
    } else if TILE_SOLID.has(kind) && !TILE_SOLID_TOP.has(kind) {
        flag3 = true;
    } else if SET_IS_A_TREE_TRUNK.has(kind) {
        // `WorldGen.IsTreeType` (`WorldGen.cs`, `TileID.Sets.IsATreeTrunk`):
        // the trunk cells crack, the branch/root cells do not — the 22 px
        // frame-grid exclusion windows (`:61584-61589`).
        flag4 = true;
        let num = frame_x / 22;
        let num2 = frame_y / 22;
        if num2 < 9 {
            flag3 = ((num != 1 && num != 2) || num2 < 6 || num2 > 8)
                && (num != 3 || num2 > 2)
                && (num != 4 || num2 < 3 || num2 > 5)
                && (num != 5 || num2 < 6 || num2 > 8);
        }
    } else if kind == 72 {
        // The giant glowing mushroom's stem (`:61591-61598`). Kept verbatim
        // even though 72 is in `IsATreeTrunk` (bits 71-77), so the tree arm
        // above claims it first and this arm is unreachable in 1.4.5.8.
        flag4 = true;
        if frame_x <= 34 {
            flag3 = true;
        }
    }
    (flag3, flag4)
}

/// The damage → crack row ladder (`Main.cs:61601-61617`): <20 draws nothing
/// (checked by the caller), 20..39 row 0, 40..59 row 1, 60..79 row 2, 80+
/// row 3.
pub(crate) fn crack_row(damage: i32) -> i32 {
    if damage >= 80 {
        3
    } else if damage >= 60 {
        2
    } else if damage >= 40 {
        1
    } else {
        0
    }
}

/// The crack source rect (`Main.cs:61618-61622`): `crackStyle * 18` column
/// on the `row * 18` row of the 16 px `TileCrack` sheet; the tree arm
/// shifts to column `4 + crackStyle / 2`.
pub(crate) fn crack_source_rect(crack_style: i32, row: i32, tree_style: bool) -> [f32; 4] {
    let mut x = crack_style * 18;
    if tree_style {
        x = (4 + crack_style / 2) * 18;
    }
    [x as f32, (row * 18) as f32, 16.0, 16.0]
}

/// One record's draw decision — every `continue` arm of
/// `Main.DrawTileCracks` (`Main.cs:61541-61622`) plus the source rect, so
/// the whole pass is unit-testable without a renderer. `None` = the record
/// draws nothing; `Some((sx, sy, sw, sh))` = the `TileCrack` source rect.
pub(crate) fn crack_draw_plan(
    world: &World,
    hit: &CrackRecord,
    crack_type: i32,
) -> Option<[f32; 4]> {
    // `:61543-61551`: the book-type filter and the <20 damage skip.
    if hit.kind != crack_type {
        return None;
    }
    let damage = hit.damage;
    if damage < 20 {
        return None;
    }
    let (x, y) = (hit.x, hit.y);
    // `WorldGen.InWorld(x, y)` (`WorldGen.cs:9688-9695`, fluff 0).
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return None;
    }
    let tile = world.tile(x, y)?;
    // `:61558-61566`: the tile arm needs an active (visible) block, the
    // wall arm a wall. `Main.ShouldShowInvisibleBlocksAndWalls`
    // (`Main.cs:58023-58030`) is the constant `false` in the port (no
    // echo monolith / see-invisible accessory) and the port's `Tile`
    // has no invisible bits, so the `(flag || !invisible…)` conjuncts
    // are vacuously true (module deviations).
    let visible = match crack_type {
        CRACK_TILE => tile.active(),
        CRACK_WALL => tile.wall != 0,
        _ => false,
    };
    if !visible {
        return None;
    }
    let (flag3, flag4) = crack_tile_shape(tile.kind, tile.frame_x as i32, tile.frame_y as i32);
    // `:61599`: slopes and half-bricks never crack.
    if flag3 && tile.slope() == 0 && !tile.half_brick() {
        let row = crack_row(damage);
        Some(crack_source_rect(hit.crack_style, row, flag4))
    } else {
        None
    }
}

/// `Main.DrawTileCracks` (`Main.cs:61533-61626`) over one `HitTile` book.
/// The `offScreenRange` vector (`:61535-61539`) is zero: the port draws the
/// world range straight to the screen-range coordinate space (the wires
/// pass's convention), so the C#'s `drawToScreen` arm is the one taken.
pub(crate) fn draw_tile_cracks(
    r: &mut SpriteRenderer,
    world: &World,
    light: &LightMap,
    style: &LightStyle,
    camera: Vec2,
    anim_time: f32,
    crack_type: i32,
    hitter: &[CrackRecord],
) {
    for hit in hitter {
        let Some([sx, sy, sw, sh]) = crack_draw_plan(world, hit, crack_type) else {
            continue;
        };
        let Some(uv) = r.uv("TileCracks", sx, sy, sw, sh) else {
            continue;
        };
        // `Lighting.GetColor(x, y)` tint (`:61623`) — the port's light
        // style shade, the same seam every tile pass uses. The position is
        // the C#'s `x * 16 - (int)screenPosition.X` (`:61623`).
        let c = style.shade(light.get(hit.x, hit.y), hit.x, hit.y, anim_time);
        r.push_sprite_paint(
            [
                hit.x as f32 * TILE_PX - camera.x,
                hit.y as f32 * TILE_PX - camera.y,
            ],
            [TILE_PX, TILE_PX],
            uv,
            [c[0], c[1], c[2], 1.0],
            0.0,
            0,
        );
    }
}

/// The port's mining-accumulator → `HitTile` projection: the C# damages a
/// tile in `Item.pick`-sized steps until 100 breaks it
/// (`Player.PickTile`/`PickTile_DetermineDamage`, `Player.cs:53757+`); the
/// port's `mine_timer` runs 0..`hardness` seconds over the same interval
/// (`world_sim/editing.rs`), so the damage is the linear map
/// `mine_timer / hardness * 100` (truncated like the C# int accumulator).
/// `None` while no dig is in progress. The crack style rolls once per dig
/// target (the C# rolls on `HitTileObject.Clear`, i.e. when the slot is
/// reused) from the world-seeded stream.
pub(crate) fn mining_crack_record(
    world: &World,
    target: Option<(i32, i32)>,
    timer: f32,
    hardness: f32,
    rng: &mut UnifiedRandom,
    last_crack: &mut i32,
    last_target: &mut Option<(i32, i32)>,
) -> Option<CrackRecord> {
    let (x, y) = target?;
    if *last_target != Some((x, y)) {
        *last_target = Some((x, y));
        roll_crack_style(rng, last_crack);
    }
    let tile = world.tile(x, y)?;
    // The wall book (`HitTile.WALL`): the port mines a wall only when the
    // cell has no active block (`editing.rs`'s wall arm).
    let kind = if tile.active() {
        CRACK_TILE
    } else {
        CRACK_WALL
    };
    let damage = if hardness > 0.0 {
        ((timer / hardness) * 100.0) as i32
    } else {
        0
    };
    Some(CrackRecord {
        x,
        y,
        damage: damage.min(100),
        kind,
        crack_style: *last_crack,
    })
}

/// The frame hook: project the mining state and run both draw sites in the
/// C# order — `hitReplace` then `hitTile` (`TileDrawing.cs:732-733`). The
/// port has no replacement-mining book yet, so the first site is empty.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_mining_cracks(
    r: &mut SpriteRenderer,
    world: &World,
    light: &LightMap,
    style: &LightStyle,
    camera: Vec2,
    anim_time: f32,
    target: Option<(i32, i32)>,
    timer: f32,
    hardness: f32,
    rng: &mut UnifiedRandom,
    last_crack: &mut i32,
    last_target: &mut Option<(i32, i32)>,
) {
    let Some(record) =
        mining_crack_record(world, target, timer, hardness, rng, last_crack, last_target)
    else {
        return;
    };
    // `DrawTileCracks(1, hitReplace)` — empty in the port (module note).
    draw_tile_cracks(r, world, light, style, camera, anim_time, CRACK_TILE, &[]);
    // `DrawTileCracks(1, hitTile)`.
    draw_tile_cracks(
        r,
        world,
        light,
        style,
        camera,
        anim_time,
        CRACK_TILE,
        &[record],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK};

    fn world_with(kind: u16, x: i32, y: i32) -> World {
        let mut w = World::new("cracks".into(), 40, 40);
        let t = w.tile_mut(x, y).unwrap();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        w
    }

    fn rec(x: i32, y: i32, damage: i32, kind: i32, style: i32) -> CrackRecord {
        CrackRecord {
            x,
            y,
            damage,
            kind,
            crack_style: style,
        }
    }

    /// `Main.cs:61601-61617`: the four damage bands and the <20 skip.
    #[test]
    fn s1_g_1c_the_damage_ladder_picks_the_four_rows() {
        assert_eq!(crack_row(20), 0);
        assert_eq!(crack_row(39), 0);
        assert_eq!(crack_row(40), 1);
        assert_eq!(crack_row(59), 1);
        assert_eq!(crack_row(60), 2);
        assert_eq!(crack_row(79), 2);
        assert_eq!(crack_row(80), 3);
        assert_eq!(crack_row(100), 3);
    }

    /// `Main.cs:61618-61622`: the 18 px grid rects and the tree column shift.
    #[test]
    fn s1_g_1c_the_source_rects_ride_the_18px_grid() {
        assert_eq!(crack_source_rect(0, 0, false), [0.0, 0.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(3, 3, false), [54.0, 54.0, 16.0, 16.0]);
        // The tree arm: column 4 + style/2 (styles 0/1 share column 4,
        // styles 2/3 column 5).
        assert_eq!(crack_source_rect(0, 1, true), [72.0, 18.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(1, 1, true), [72.0, 18.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(2, 1, true), [90.0, 18.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(3, 1, true), [90.0, 18.0, 16.0, 16.0]);
    }

    /// `Main.cs:61573-61598`: the door skip, the solid/solid-top split, the
    /// tree frame windows and the glowing-mushroom stem.
    #[test]
    fn s1_g_1c_the_tile_shape_classification_is_verbatim() {
        // Door (10): never.
        assert_eq!(crack_tile_shape(10, 0, 0), (false, false));
        // Dirt (0) / stone (1): solid, not solid-top.
        assert_eq!(crack_tile_shape(0, 0, 0), (true, false));
        assert_eq!(crack_tile_shape(1, 0, 0), (true, false));
        // Platform (19): solid-top — no crack.
        assert_eq!(crack_tile_shape(19, 0, 0), (false, false));
        // Tree trunk (5): the trunk cells crack with the tree columns.
        assert_eq!(crack_tile_shape(5, 0, 0), (true, true));
        assert_eq!(crack_tile_shape(5, 66, 66), (true, true)); // col 3 row 3
        // The branch/root exclusion windows (`:61588`).
        assert_eq!(crack_tile_shape(5, 66, 0), (false, true)); // col 3 row 0
        assert_eq!(crack_tile_shape(5, 22, 132), (false, true)); // col 1 row 6
        assert_eq!(crack_tile_shape(5, 44, 176), (false, true)); // col 2 row 8
        assert_eq!(crack_tile_shape(5, 66, 44), (false, true)); // col 3 row 2
        assert_eq!(crack_tile_shape(5, 88, 66), (false, true)); // col 4 row 3
        assert_eq!(crack_tile_shape(5, 88, 110), (false, true)); // col 4 row 5
        assert_eq!(crack_tile_shape(5, 110, 132), (false, true)); // col 5 row 6
        assert_eq!(crack_tile_shape(5, 110, 176), (false, true)); // col 5 row 8
        // Outside the windows the same columns crack.
        assert_eq!(crack_tile_shape(5, 22, 110), (true, true)); // col 1 row 5
        assert_eq!(crack_tile_shape(5, 88, 132), (true, true)); // col 4 row 6
        // row >= 9: the C# leaves flag3 false.
        assert_eq!(crack_tile_shape(5, 0, 198), (false, true));
        // Giant glowing mushroom (72): it is in `TileID.Sets.IsATreeTrunk`
        // (bits 71-77), so the tree arm claims it BEFORE the C#'s
        // `type == 72` arm (`Main.cs:61591`, unreachable in 1.4.5.8 — the
        // port keeps the arm verbatim). The tree windows apply.
        assert_eq!(crack_tile_shape(72, 0, 0), (true, true));
        assert_eq!(crack_tile_shape(72, 36, 0), (true, true)); // col 1 row 0
        assert_eq!(crack_tile_shape(72, 22, 132), (false, true)); // col 1 row 6
        // A non-solid furniture tile (torch 4): no crack.
        assert_eq!(crack_tile_shape(4, 0, 0), (false, false));
    }

    /// `HitTile.cs:40-45`: the style is 0..3 and never repeats the last one.
    #[test]
    fn s1_g_1c_the_style_roll_never_repeats_the_last_crack() {
        let mut rng = UnifiedRandom::new(42);
        let mut last = -1;
        for _ in 0..200 {
            let prev = last;
            let style = roll_crack_style(&mut rng, &mut last);
            assert!((0..4).contains(&style));
            assert_ne!(style, prev, "the no-repeat loop must reroll");
            assert_eq!(style, last);
        }
        // Deterministic for the seed (the world-seeded deviation).
        let mut a = UnifiedRandom::new(7);
        let mut b = UnifiedRandom::new(7);
        let (mut la, mut lb) = (-1, -1);
        for _ in 0..20 {
            assert_eq!(
                roll_crack_style(&mut a, &mut la),
                roll_crack_style(&mut b, &mut lb)
            );
        }
    }

    /// The mining projection: the timer maps linearly onto the 0..100
    /// damage, the style rolls once per target, and the wall book is chosen
    /// when the cell has no active block.
    #[test]
    fn s1_g_1c_the_mining_projection_maps_the_timer_to_damage() {
        let w = world_with(1, 5, 6);
        let mut rng = UnifiedRandom::new(3);
        let (mut last, mut last_target) = (-1, None);
        let r0 = mining_crack_record(
            &w,
            Some((5, 6)),
            0.0,
            1.0,
            &mut rng,
            &mut last,
            &mut last_target,
        )
        .unwrap();
        assert_eq!(r0.damage, 0);
        assert_eq!(r0.kind, CRACK_TILE);
        let style = r0.crack_style;
        // Same target: no reroll; the damage climbs with the timer.
        for (t, d) in [(0.2, 20), (0.45, 45), (0.8, 80), (1.0, 100)] {
            let r = mining_crack_record(
                &w,
                Some((5, 6)),
                t,
                1.0,
                &mut rng,
                &mut last,
                &mut last_target,
            )
            .unwrap();
            assert_eq!(r.damage, d);
            assert_eq!(r.crack_style, style, "the style holds for one dig");
        }
        // A new target rerolls the style (the slot-reuse `Clear`).
        let w2 = world_with(1, 7, 6);
        let r = mining_crack_record(
            &w2,
            Some((7, 6)),
            0.5,
            1.0,
            &mut rng,
            &mut last,
            &mut last_target,
        )
        .unwrap();
        assert_eq!(r.damage, 50);
        // No target: nothing.
        assert!(
            mining_crack_record(&w, None, 0.5, 1.0, &mut rng, &mut last, &mut last_target)
                .is_none()
        );
        // The wall book: an inactive cell with a wall.
        let mut ww = World::new("wall".into(), 40, 40);
        ww.tile_mut(2, 2).unwrap().wall = 4;
        let r = mining_crack_record(
            &ww,
            Some((2, 2)),
            0.6,
            1.0,
            &mut rng,
            &mut last,
            &mut last_target,
        )
        .unwrap();
        assert_eq!(r.kind, CRACK_WALL);
        assert_eq!(r.damage, 60);
    }

    /// The per-pass draw decision — every `continue` arm of
    /// `Main.DrawTileCracks` (`Main.cs:61541-61622`) in order, plus the
    /// emitted source rect for the drawing arm.
    #[test]
    fn s1_g_1c_the_draw_pass_gates_match_the_csharp_continues() {
        let mut w = world_with(1, 3, 3); // stone, active
        // The drawing arm: damage 80+, style 2 → row 3, column 36.
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 2), CRACK_TILE),
            Some([36.0, 54.0, 16.0, 16.0])
        );
        // The wall book draws on the same cell only via crack type 2 when
        // a wall exists (`:61563-61566`).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 2), CRACK_WALL),
            None,
            "no wall in the cell"
        );
        w.tile_mut(3, 3).unwrap().wall = 4;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 2), CRACK_WALL),
            Some([36.0, 54.0, 16.0, 16.0])
        );
        // `:61543`: the book-type filter.
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 2), CRACK_TILE),
            None
        );
        // `:61548`: the <20 damage floor (19 skips, 20 draws row 0).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 19, CRACK_TILE, 0), CRACK_TILE),
            None
        );
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 20, CRACK_TILE, 0), CRACK_TILE),
            Some([0.0, 0.0, 16.0, 16.0])
        );
        // `:61554` InWorld (fluff 0): every out-of-bounds cell skips.
        for (x, y) in [(-1, 3), (40, 3), (3, -1), (3, 40)] {
            assert_eq!(
                crack_draw_plan(&w, &rec(x, y, 85, CRACK_TILE, 0), CRACK_TILE),
                None
            );
        }
        // `:61561`: an inactive block skips the tile arm.
        w.tile_mut(3, 3).unwrap().flags &= !FLAG_ACTIVE;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE),
            None
        );
        // …but the wall arm still draws (the wall survives).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 0), CRACK_WALL),
            Some([0.0, 54.0, 16.0, 16.0])
        );
        w.tile_mut(3, 3).unwrap().flags |= FLAG_ACTIVE;
        // `:61599`: slope 1 skips…
        w.tile_mut(3, 3).unwrap().meta = 1;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE),
            None
        );
        // …and so does a half-brick.
        w.tile_mut(3, 3).unwrap().meta = 0;
        w.tile_mut(3, 3).unwrap().flags |= FLAG_HALF_BRICK;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE),
            None
        );
        w.tile_mut(3, 3).unwrap().flags &= !FLAG_HALF_BRICK;
        // `:61573`: the door never cracks; `:61577`: solid-top never cracks.
        w.tile_mut(3, 3).unwrap().kind = 10;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE),
            None
        );
        w.tile_mut(3, 3).unwrap().kind = 19; // platform
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE),
            None
        );
        // The tree arm shifts the column (`:61619-61622`): trunk 5, style 3
        // → column (4 + 3/2) * 18 = 90.
        w.tile_mut(3, 3).unwrap().kind = 5;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 3), CRACK_TILE),
            Some([90.0, 54.0, 16.0, 16.0])
        );
    }

    /// The hook's book order (`TileDrawing.cs:732-733`): `hitReplace` first
    /// (empty in the port), then `hitTile` — the projection feeds exactly
    /// one record and the style holds across frames of one dig.
    #[test]
    fn s1_g_1c_the_hook_projects_one_record_per_dig() {
        let w = world_with(1, 5, 6);
        let mut rng = UnifiedRandom::new(11);
        let (mut style, mut target) = (0, None);
        // Frame 1 of a dig: damage below the floor → the plan skips.
        let r = mining_crack_record(
            &w,
            Some((5, 6)),
            0.1,
            1.0,
            &mut rng,
            &mut style,
            &mut target,
        )
        .unwrap();
        assert_eq!(crack_draw_plan(&w, &r, CRACK_TILE), None);
        // Later frames climb the ladder without rerolling the style.
        let mut prev_style = r.crack_style;
        for (t, row) in [(0.25, 0.0), (0.5, 18.0), (0.7, 36.0), (0.9, 54.0)] {
            let r =
                mining_crack_record(&w, Some((5, 6)), t, 1.0, &mut rng, &mut style, &mut target)
                    .unwrap();
            assert_eq!(r.crack_style, prev_style);
            prev_style = r.crack_style;
            let plan = crack_draw_plan(&w, &r, CRACK_TILE).expect("draws");
            assert_eq!(plan[1], row, "the damage ladder's row at t={t}");
            assert_eq!(plan[0], (r.crack_style * 18) as f32);
        }
    }
}
