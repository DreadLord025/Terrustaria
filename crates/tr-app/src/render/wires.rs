//! The wire overlay — `Main.DrawWires` (`Main.cs:58218-58701`): the
//! four-colour `WireNew` quads per wire-carrying tile, the neighbour-mask
//! source rects, the multi-colour overlap dimming (`color * (1 / num7)`) and
//! the 6 px inset strips where two colours share a tile, plus the actuator
//! plate (`TextureAssets.Actuator`, `:58654-58661`).
//!
//! Deviations: `WiresUI.Settings.HideWires` and the mechanic-accessory
//! `builderAccStatus` channels (the per-colour White/×0.5/Transparent
//! switches, `Main.cs:58229-58243`) are fixed at the defaults — the port has
//! no wires UI or accessory equip yet, so every channel draws at the light
//! colour and `num = 1f`. `DrawWiresSpecialTiles` (the tile-423
//! LightSeaGreen rectangle, `:58664-58678`) is content-gated — the port has
//! no tile 423. `Player.CanShowWireStuffHere` keeps the pre-Golem wall-87
//! guard (`Player.cs:46946-46961`; the boss-progress conjunct is always in
//! the pre-boss state, M5-HARDFORCE).

use glam::Vec2;

use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tile::FLAG_ACTUATOR;
use tr_core::wiring::WireColor;
use tr_core::{TILE_PX, World};
use tr_render::sprite::SpriteRenderer;

/// `Main.DrawWires`'s source-rect mask arithmetic (`Main.cs:58279-58296`):
/// a same-colour neighbour above adds 18, right 36, down 72, left 144 — the
/// `WireNew` sheet's 16 px mask columns, `(num8 + row)` the colour's 18 px
/// row.
pub(crate) fn wire_mask(has: [bool; 4]) -> i32 {
    let mut m = 0;
    if has[0] {
        m += 18;
    }
    if has[1] {
        m += 36;
    }
    if has[2] {
        m += 72;
    }
    if has[3] {
        m += 144;
    }
    m
}

/// `DrawWires`'s per-tile walk over the visible range. The four colour arms
/// (`Main.cs:58264-58653`) share the state: `flag2..flag5` are the
/// earlier-colour side claims (up/right/down/left) and `num7` counts the
/// drawn colours, dimming the later ones by `1 / num7`.
pub(crate) fn draw_wires(
    r: &mut SpriteRenderer,
    world: &World,
    light: &LightMap,
    style: &LightStyle,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    anim_time: f32,
) {
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    let wires = &world.wires;
    let colours = [
        (WireColor::Red, 0.0),
        (WireColor::Blue, 18.0),
        (WireColor::Green, 36.0),
        (WireColor::Yellow, 54.0),
    ];
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            // `Player.CanShowWireStuffHere` (`Player.cs:46946-46961`): the
            // pre-Golem Lihzahrd wall guard hides the wires inside the
            // temple.
            if !can_show_wire_stuff_here(world, tx, ty) {
                continue;
            }
            let mut num7 = 0.0f32;
            // The earlier-colour side claims: [up, right, down, left]
            // (`flag4`, `flag3`, `flag5`, `flag2` in the C#'s order).
            let mut claimed = [false; 4];
            // The junction-box column offset (`num8`, `Main.cs:58256-58266`).
            let num8 = {
                let t = match world.tile(tx, ty) {
                    Some(t) => t,
                    None => continue,
                };
                if t.active() {
                    if t.kind == 424 {
                        match t.frame_x / 18 {
                            0 => 72.0,
                            1 => 144.0,
                            2 => 216.0,
                            _ => 0.0,
                        }
                    } else if t.kind == 445 {
                        72.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            };
            for &(colour, row) in &colours {
                let has = [
                    wires.get(tx, ty - 1, colour),
                    wires.get(tx + 1, ty, colour),
                    wires.get(tx, ty + 1, colour),
                    wires.get(tx - 1, ty, colour),
                ];
                if !wires.get(tx, ty, colour) {
                    continue;
                }
                num7 += 1.0;
                let mut overlap = false;
                let mut sides = [false; 4];
                for (k, h) in has.iter().enumerate() {
                    if *h {
                        sides[k] = true;
                        if claimed[k] {
                            overlap = true;
                        }
                    }
                }
                if num7 > 1.0 {
                    overlap = true;
                }
                let l = light.get(tx, ty);
                let mut c = style.shade(l, tx, ty, anim_time);
                if num7 > 1.0 {
                    // `color2 * (1f / num7)` (`Main.cs:58385` and the colour
                    // twins): the later colours dim on the shared tile.
                    for v in &mut c {
                        *v /= num7;
                    }
                }
                let mask = wire_mask(has) as f32;
                let Some(uv) = r.uv("WiresNew", mask, num8 + row, 16.0, 16.0) else {
                    continue;
                };
                r.push_sprite_paint(
                    [tx as f32 * TILE_PX, ty as f32 * TILE_PX],
                    [TILE_PX, TILE_PX],
                    uv,
                    [c[0], c[1], c[2], 1.0],
                    0.0,
                    0,
                );
                // The 6 px inset strips (`Main.cs:58390-58425` and the
                // colour twins): when the colours share this tile, every
                // side this colour claims that no earlier colour claimed
                // draws a narrow strip at the full colour.
                for (k, (sx, sy, sw, sh)) in [
                    (18.0f32, 0.0f32, 16.0f32, 6.0f32),
                    (72.0, 10.0, 16.0, 6.0),
                    (46.0, 0.0, 6.0, 16.0),
                    (144.0, 0.0, 6.0, 16.0),
                ]
                .into_iter()
                .enumerate()
                {
                    if !sides[k] {
                        continue;
                    }
                    if overlap && !claimed[k] {
                        let (ox, oy) = match k {
                            1 => (0.0, 10.0), // down
                            2 => (10.0, 0.0), // right
                            _ => (0.0, 0.0),  // up / left
                        };
                        if let Some(uv) = r.uv("WiresNew", sx, num8 + row + sy, sw, sh) {
                            r.push_sprite_paint(
                                [tx as f32 * TILE_PX + ox, ty as f32 * TILE_PX + oy],
                                [sw, sh],
                                uv,
                                [c[0], c[1], c[2], 1.0],
                                0.0,
                                0,
                            );
                        }
                    }
                    claimed[k] = true;
                }
            }
            // The actuator plate (`Main.cs:58654-58661`): a full-tile quad
            // on every actuator-carrying tile that receives light.
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if t.flags & FLAG_ACTUATOR != 0 && light.get(tx, ty).iter().any(|v| *v > 0.0) {
                let l = light.get(tx, ty);
                let c = style.shade(l, tx, ty, anim_time);
                if let Some(uv) = r.uv("Actuator", 0.0, 0.0, 16.0, 16.0) {
                    r.push_sprite_paint(
                        [tx as f32 * TILE_PX, ty as f32 * TILE_PX],
                        [TILE_PX, TILE_PX],
                        uv,
                        [c[0], c[1], c[2], 1.0],
                        0.0,
                        0,
                    );
                }
            }
        }
    }
}

/// `Player.CanShowWireStuffHere` (`Player.cs:46946-46961`) minus the port's
/// missing boss-progress flag: the temple's wall 87 hides the wires until
/// Golem falls (always "not fallen" in the port, M5-HARDFORCE).
fn can_show_wire_stuff_here(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.wall != 87
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wire_mask_arithmetic_is_verbatim() {
        // `Main.cs:58279-58296`: up +18, right +36, down +72, left +144.
        assert_eq!(wire_mask([false; 4]), 0, "the isolated cell");
        assert_eq!(wire_mask([true, false, false, false]), 18, "up");
        assert_eq!(wire_mask([false, true, false, false]), 36, "right");
        assert_eq!(wire_mask([false, false, true, false]), 72, "down");
        assert_eq!(wire_mask([false, false, false, true]), 144, "left");
        assert_eq!(wire_mask([true, true, true, true]), 270, "the cross");
    }
}
