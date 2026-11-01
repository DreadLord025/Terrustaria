//! `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the
//! transparent marker tiles of a tree.
use crate::assets;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::{TILE_PX, World, tree};
use tr_render::SpriteRenderer;

/// `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the transparent
/// marker tiles of a tree. The pass scans a slightly larger rect than the tile loop
/// because an 80x80 canopy sticks out above and beside its anchor.
pub(crate) fn draw_tree_foliage(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
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
            match part {
                tree::TreePart::Top => {
                    let sx = f.frame as i32 * (f.top_w as i32 + 2);
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, true, sx, 0, f.top_w as u32, f.top_h as u32)
                    {
                        r.push_sprite(
                            [px + 8.0 - f.top_w * 0.5, py + 16.0 - f.top_h],
                            [f.top_w, f.top_h],
                            uv,
                            color,
                        );
                        quads += 1;
                    }
                }
                tree::TreePart::BranchLeft => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 0, f.frame as i32 * 42, 40, 40)
                    {
                        r.push_sprite([px - 24.0, py - 12.0], [40.0, 40.0], uv, color);
                        quads += 1;
                    }
                }
                tree::TreePart::BranchRight => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 42, f.frame as i32 * 42, 40, 40)
                    {
                        r.push_sprite([px, py - 12.0], [40.0, 40.0], uv, color);
                        quads += 1;
                    }
                }
            }
        }
    }
    quads
}
