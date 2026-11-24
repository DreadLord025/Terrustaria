//! The wire overlay — `Main.DrawWires` (`Main.cs:58218-58701`): the
//! four-colour `WireNew` quads per wire-carrying tile, the neighbour-mask
//! source rects, the multi-colour overlap dimming (`color * (1 / num7)`) and
//! the 6 px inset strips where two colours share a tile, plus the actuator
//! plate (`TextureAssets.Actuator`, `:58643-58658`).
//!
//! Visibility (S1-P-i34): the overlay is gated by `WiresUI.Settings.DrawWires`
//! (`WiresUI.cs:27-45`) — a held mech tool draws it, else the Mechanical Lens /
//! Wire Kite (`InfoAccMechShowWires`) draws it while `builderAccStatus[8] == 0`;
//! the no-traps seed hides it until Skeletron falls. `WiresUI.Settings.HideWires`
//! (`:47-57`, the Actuation Rod 3620 in hand) suppresses the wire colours and
//! halves the actuator plate (`num = 0.5f`, `Main.cs:58224-58229`). The
//! mechanic-accessory `builderAccStatus` channels (`Main.cs:58235-58243`) pick
//! each colour's tint: 0 = White (bright), 1 = the light colour, 2 = ×0.5
//! (faded), 3 = Transparent (hidden). [`wire_view`] resolves all of this; the
//! caller draws only when it returns `Some`.
//!
//! S1-W4-g81 (the verbatim draw census): the tints ride the XNA BYTE maths —
//! `Lighting.GetColor`'s `(int)(light * GlobalBrightness * 255)` byte clamp
//! (`Lighting.cs:216-242`), `color *= 0.5f`'s per-channel truncation
//! (`Main.cs:58324`) and `color * (1f / num7)`'s float scale on the bytes
//! (`:58405`); XNA's `Color * float` scales the ALPHA too (255 → 127 at ×0.5),
//! so the faded channel and the overlap dim draw semi-transparent. The
//! overlap strips draw at the channel-tinted colour BEFORE the `1/num7` dim
//! (`:58410`), which the earlier float port wrongly dimmed too.
//! `Lighting.Brightness` (`Lighting.cs:118-122`) gates the actuator
//! plate — the RAW light mean × GlobalBrightness, not the per-mode shade.
//! The tile-423 LightSeaGreen range rectangle (`DrawWiresSpecialTiles`,
//! `:58660-58672`) is ported through `Utils.DrawRectangle`/`DrawLine`
//! (`Utils.cs:2619-2625`/`:2689-2702`); the port has no placeable 423 yet
//! but `.wld` worlds carry them (`tile_entities.rs` knows the sensor).
//! `Player.CanShowWireStuffHere` keeps the pre-Golem wall-87 guard
//! (`Player.cs:46946-46961`) — it gates ONLY the wire colours (`:58266`);
//! the actuator plate (`:58643`) and the 423 collection (`:58660`) sit
//! outside it and draw inside the temple too. The per-colour strip
//! side-claims use the port's unified `claimed` walk (the C# red wire claims
//! unconditionally, the later colours only on draw — a micro-asymmetry
//! unreachable in practice, since the UI cycle wraps at 3 so Transparent is
//! never selected, `Main.cs:47551-47555`).
//! The per-mode `LightStyle::shade` transforms (White/Retro/Trippy) are NOT
//! applied to the wire tints: the C# draws the raw `Lighting.GetColor` bytes
//! in every light mode.

use glam::Vec2;

use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tile::FLAG_ACTUATOR;
use tr_core::wiring::WireColor;
use tr_core::{TILE_PX, World};
use tr_render::sprite::SpriteRenderer;

/// The resolved wire-overlay visibility for one frame — the inputs
/// `Main.DrawWires` reads off the local player (`Main.cs:58224-58243`) and
/// `WiresUI.Settings` (`WiresUI.cs:27-57`). `None` from [`wire_view`] means the
/// overlay must not draw at all.
pub(crate) struct WireView {
    /// `WiresUI.Settings.HideWires` — the Actuation Rod (3620) in hand: the
    /// wire colours are suppressed and the actuator plate draws at ×0.5.
    pub hide_wires: bool,
    /// The per-channel `builderAccStatus` tint selectors, in `DrawWires`' order
    /// `num2..num6`: red[4], blue[5], green[6], yellow[7], actuator[9]
    /// (`Main.cs:58238-58242`). 0 = White, 1 = light colour, 2 = ×0.5,
    /// 3 = Transparent. Defaults to all-1 (normal) without the lens.
    pub channels: [i32; 5],
}

/// `WiresUI.Settings.DrawWires` (`WiresUI.cs:27-45`) + `HideWires` (`:47-57`)
/// + the `builderAccStatus` channels (`Main.cs:58235-58243`). Returns `None`
/// when the overlay must not draw. `held_mech` is `Item.mech` of the held item;
/// `held_is_air` is `Item.IsAir` (`Item.cs:385-395`: type ≤ 0 or stack ≤ 0).
pub(crate) fn wire_view(
    held_kind: u16,
    held_count: u32,
    held_mech: bool,
    info_acc_mech_show_wires: bool,
    builder_acc_status: &[i32; 12],
    no_traps: bool,
    downed_boss3: bool,
) -> Option<WireView> {
    // `WiresUI.cs:31-34` — the no-traps world hides the wires until Skeletron
    // (`NPC.downedBoss3`) falls.
    if no_traps && !downed_boss3 {
        return None;
    }
    // `WiresUI.cs:35-43` — a held mech tool draws the overlay; otherwise the
    // lens/kite (`InfoAccMechShowWires`) draws it only while its toggle
    // (`builderAccStatus[8]`) is on (0).
    let held_is_air = held_kind == 0 || held_count == 0;
    let draw = if held_is_air || !held_mech {
        info_acc_mech_show_wires && builder_acc_status[8] == 0
    } else {
        true
    };
    if !draw {
        return None;
    }
    // `WiresUI.cs:47-57` — `HideWires` is the Actuation Rod in hand (non-air).
    let hide_wires = !held_is_air && held_kind == 3620;
    // `Main.cs:58230-58243` — the channels default to 1 (normal); the lens
    // overrides them with the saved toggle states.
    let channels = if info_acc_mech_show_wires {
        [
            builder_acc_status[4],
            builder_acc_status[5],
            builder_acc_status[6],
            builder_acc_status[7],
            builder_acc_status[9],
        ]
    } else {
        [1; 5]
    };
    Some(WireView {
        hide_wires,
        channels,
    })
}

/// `Lighting.GetColor(int x, int y)` (`Lighting.cs:216-242`): the RAW engine
/// light sample as XNA bytes — `(int)(light * GlobalBrightness * 255)` per
/// channel, clamped at 255 (`:228-239`), alpha 255 (`:240`'s `-16777216`).
/// No `LightStyle::shade` transforms: the C# draws these bytes in every light
/// mode. Light samples are non-negative, so the C#'s missing low clamp cannot
/// bite. The `Main.gameMenu` white branch (`:218-221`) never runs in-world.
fn light_color_bytes(l: [f32; 3], global_brightness: f32) -> [u8; 4] {
    let num = global_brightness * 255.0;
    [
        ((l[0] * num) as i32).min(255) as u8,
        ((l[1] * num) as i32).min(255) as u8,
        ((l[2] * num) as i32).min(255) as u8,
        255,
    ]
}

/// `Lighting.Brightness(int x, int y)` (`Lighting.cs:118-122`):
/// `GlobalBrightness * (X + Y + Z) / 3f` on the RAW engine sample — the
/// actuator plate's draw gate (`Main.cs:58643`).
fn lighting_brightness(l: [f32; 3], global_brightness: f32) -> f32 {
    global_brightness * (l[0] + l[1] + l[2]) / 3.0
}

/// The per-channel tint switch (`Main.cs:58318-58329` and the colour twins,
/// the actuator `:58646-58657`) on XNA bytes: 0 → White, 2 → `color *= 0.5f`
/// (XNA's `Color * float` truncates EVERY channel incl. alpha — 255 → 127),
/// 3 → Transparent (the caller skips the draw), 1/default → unchanged.
/// Returns `false` for Transparent (the wire is hidden and `num7` must
/// decrement).
fn channel_tint(c: &mut [u8; 4], channel: i32) -> bool {
    match channel {
        0 => *c = [255; 4],
        2 => {
            for v in c.iter_mut() {
                *v = (*v as f32 * 0.5) as u8;
            }
        }
        3 => return false,
        _ => {}
    }
    true
}

/// XNA's `Color * float` (`color * (1f / num7)`, `Main.cs:58405`; the
/// actuator's `color5 * num`, `:58658`): `(int)(byte * scale)` per channel
/// INCLUDING alpha, normalised back to `[0, 1]` for the instance tint (the
/// WGSL multiplies the sRGB bytes in gamma space, S1-G-4).
fn scale_color(c: [u8; 4], scale: f32) -> [f32; 4] {
    [
        ((c[0] as f32 * scale) as i32) as f32 / 255.0,
        ((c[1] as f32 * scale) as i32) as f32 / 255.0,
        ((c[2] as f32 * scale) as i32) as f32 / 255.0,
        ((c[3] as f32 * scale) as i32) as f32 / 255.0,
    ]
}

/// One wire colour's two draw tints: the main quad at `color * (1f / num7)`
/// (`Main.cs:58405` and the colour twins; the red quad `:58336` draws the
/// unscaled colour — at `num7 == 1` the scale is the identity) and the
/// overlap strips at the channel-tinted colour BEFORE the dim (`:58410`).
/// `None` = Transparent: the caller refunds `num7` and draws nothing
/// (`:58330-58333`).
fn wire_tint(
    l: [f32; 3],
    global_brightness: f32,
    channel: i32,
    num7: f32,
) -> Option<([f32; 4], [f32; 4])> {
    let mut c = light_color_bytes(l, global_brightness);
    if !channel_tint(&mut c, channel) {
        return None;
    }
    Some((scale_color(c, 1.0 / num7), scale_color(c, 1.0)))
}

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

/// The junction-box column offset `num8` (`Main.cs:58268-58290`): the 424
/// junction box's `frameX / 18` picks column 72/144/216, the 445 actuated
/// junction box pins 72, everything else 0.
fn junction_offset(t: &tr_core::tile::Tile) -> f32 {
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
}

/// The four overlap-strip draws in the C#'s order — up (`flag8`, rect
/// `(18, value.Y, 16, 6)`), down (`flag9`, `(72, value.Y + 10, 16, 6)` at
/// `+10y`), right (`flag7`, `(46, value.Y, 6, 16)` at `+10x`), left (`flag6`,
/// `(144, value.Y, 6, 16)`) — `Main.cs:58406-58437` and the colour twins.
/// Each row: the side index into `[up, right, down, left]`, the source rect
/// (x, y-offset, w, h) and the tile-local draw offset.
const STRIPS: [(usize, f32, f32, f32, f32, f32, f32); 4] = [
    (0, 18.0, 0.0, 16.0, 6.0, 0.0, 0.0),
    (2, 72.0, 10.0, 16.0, 6.0, 0.0, 10.0),
    (1, 46.0, 0.0, 6.0, 16.0, 10.0, 0.0),
    (3, 144.0, 0.0, 6.0, 16.0, 0.0, 0.0),
];

/// `DrawWires`'s per-tile walk over the visible range. The four colour arms
/// (`Main.cs:58264-58653`) share the state: `flag2..flag5` are the
/// earlier-colour side claims (up/right/down/left) and `num7` counts the
/// drawn colours, dimming the later ones by `1 / num7`. The actuator plate
/// and the 423 collection sit OUTSIDE the `flag && CanShowWireStuffHere`
/// guard (`:58643`/`:58660` vs `:58266`).
pub(crate) fn draw_wires(
    r: &mut SpriteRenderer,
    world: &World,
    light: &LightMap,
    style: &LightStyle,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    view: &WireView,
) {
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    // `Main.cs:58224-58229`: `flag = !HideWires` gates the wire colours;
    // `num` halves the actuator plate while the rod is held.
    let draw_wire_colours = !view.hide_wires;
    let actuator_scale = if view.hide_wires { 0.5 } else { 1.0 };
    // `Lighting.GlobalBrightness` — the style carries it (the blind drop
    // 1.2 → 1.0 is applied at the frame seam, `Lighting.cs:108-116`).
    let gb = style.brightness;

    let wires = &world.wires;
    let colours = [
        (WireColor::Red, 0.0, 0usize),
        (WireColor::Blue, 18.0, 1),
        (WireColor::Green, 36.0, 2),
        (WireColor::Yellow, 54.0, 3),
    ];
    // `this.DrawWiresSpecialTiles` (`Main.cs:58223`/`:58660-58672`): the
    // 423 sensors collected during the walk, drawn after the whole loop.
    let mut special_tiles: Vec<(i32, i32)> = Vec::new();
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            // `Main.cs:58266`: the wire colours draw only when `flag` rides
            // (`HideWires` off) and the tile passes `CanShowWireStuffHere`.
            if draw_wire_colours && can_show_wire_stuff_here(world, tx, ty) {
                let num8 = junction_offset(t);
                let mut num7 = 0.0f32;
                // The earlier-colour side claims: [up, right, down, left]
                // (`flag4`, `flag3`, `flag5`, `flag2` in the C#'s order).
                let mut claimed = [false; 4];
                for &(colour, row, ch) in &colours {
                    if !wires.get(tx, ty, colour) {
                        continue;
                    }
                    let has = [
                        wires.get(tx, ty - 1, colour),
                        wires.get(tx + 1, ty, colour),
                        wires.get(tx, ty + 1, colour),
                        wires.get(tx - 1, ty, colour),
                    ];
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
                    // The byte tint pair: the main quad dimmed by `1/num7`,
                    // the strips at the pre-dim colour (`Main.cs:58405-58410`).
                    // Transparent hides the wire and gives its `num7` back.
                    let Some((main_c, strip_c)) =
                        wire_tint(light.get(tx, ty), gb, view.channels[ch], num7)
                    else {
                        num7 -= 1.0;
                        continue;
                    };
                    let mask = wire_mask(has) as f32;
                    if let Some(uv) = r.uv("WiresNew", mask, num8 + row, 16.0, 16.0) {
                        r.push_sprite_paint(
                            [tx as f32 * TILE_PX, ty as f32 * TILE_PX],
                            [TILE_PX, TILE_PX],
                            uv,
                            main_c,
                            0.0,
                            0,
                        );
                    }
                    // The 6 px inset strips (`Main.cs:58406-58437` and the
                    // colour twins): every side this colour claims that no
                    // earlier colour claimed draws a narrow strip at the
                    // channel-tinted colour (no `1/num7` dim).
                    for &(k, sx, sy, sw, sh, ox, oy) in &STRIPS {
                        if !sides[k] {
                            continue;
                        }
                        if overlap && !claimed[k] {
                            if let Some(uv) = r.uv("WiresNew", sx, num8 + row + sy, sw, sh) {
                                r.push_sprite_paint(
                                    [tx as f32 * TILE_PX + ox, ty as f32 * TILE_PX + oy],
                                    [sw, sh],
                                    uv,
                                    strip_c,
                                    0.0,
                                    0,
                                );
                            }
                        }
                        claimed[k] = true;
                    }
                }
            }
            // The actuator plate (`Main.cs:58643-58658`): a full-tile quad on
            // every actuator-carrying tile with `Lighting.Brightness > 0` OR
            // a Bright channel (`num6 == 0`). The channel tint applies, then
            // `* num` (the rod's ×0.5, alpha included). Unlike the wires this
            // draws even under `HideWires` and inside the temple — the C#
            // block sits outside both guards. A Transparent channel draws an
            // alpha-0 quad in the C#; the port skips it (pixel-identical).
            if t.flags & FLAG_ACTUATOR != 0 {
                let l = light.get(tx, ty);
                if lighting_brightness(l, gb) > 0.0 || view.channels[4] == 0 {
                    let mut c = light_color_bytes(l, gb);
                    if channel_tint(&mut c, view.channels[4]) {
                        let c = scale_color(c, actuator_scale);
                        if let Some(uv) = r.uv("Actuator", 0.0, 0.0, 16.0, 16.0) {
                            r.push_sprite_paint(
                                [tx as f32 * TILE_PX, ty as f32 * TILE_PX],
                                [TILE_PX, TILE_PX],
                                uv,
                                c,
                                0.0,
                                0,
                            );
                        }
                    }
                }
            }
            // `Main.cs:58660-58663`: the 423 sensor's row-2 frame collects
            // into `DrawWiresSpecialTiles` — outside the visibility guard.
            if t.active() && t.kind == 423 && t.frame_y == 36 {
                special_tiles.push((tx, ty));
            }
        }
    }
    // `Main.cs:58666-58673`: after the walk, every collected 423 draws its
    // LightSeaGreen range rectangle through `Utils.DrawRectangle`.
    for (tx, ty) in special_tiles {
        let (start, end) = special_tile_rect(tx, ty);
        draw_rectangle(r, start, end, LIGHT_SEA_GREEN, LIGHT_SEA_GREEN, 2.0);
    }
}

/// `Player.CanShowWireStuffHere` (`Player.cs:46946-46961`): the temple's
/// wall 87 hides the wires until Golem falls (`world.downed_golem`). The
/// dual-dungeons temple-wall style check (`Main.dualDungeonsSeed`) has no
/// ported seed.
fn can_show_wire_stuff_here(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !world.downed_golem && t.wall == 87 {
        return false;
    }
    true
}

/// XNA `Color.LightSeaGreen` — `(32, 178, 170, 255)`, the 423 range
/// rectangle's colour (`Main.cs:58671`).
const LIGHT_SEA_GREEN: [f32; 4] = [32.0 / 255.0, 178.0 / 255.0, 170.0 / 255.0, 1.0];

/// The 423 sensor's range rectangle corners (`Main.cs:58671`):
/// start `(j*16 - 32 - 1, i*16 - 160 - 1)`, end `(j*16 + 48 + 1, i*16 + 1)`
/// — an 82×162 px box reaching 10 tiles up and 2 tiles sideways.
fn special_tile_rect(tx: i32, ty: i32) -> (Vec2, Vec2) {
    let x = tx as f32 * TILE_PX;
    let y = ty as f32 * TILE_PX;
    (
        Vec2::new(x - 32.0 - 1.0, y - 160.0 - 1.0),
        Vec2::new(x + 48.0 + 1.0, y + 1.0),
    )
}

/// `Utils.DrawLine`'s segment plan (`Utils.cs:2689-2702`): `num2 =
/// Math.Min(5, (int)distance)` segments, collapsed to ONE when
/// `colorStart == colorEnd`; segment `i` draws at `Lerp(start, end, i/num2)`
/// with `Lerp(colorStart, colorEnd, (i+0.5)/num2)`, rotated by
/// `(end - start).ToRotation()` about the origin `(0, 8)` (the left-middle
/// of the 16 px `BlackTile`), scaled `(num/num2/16, width/16)` — a
/// `num/num2 × width` px quad. Returns (position, size, rotation, colour).
fn line_segments(
    start: Vec2,
    end: Vec2,
    color_start: [f32; 4],
    color_end: [f32; 4],
    width: f32,
) -> Vec<([f32; 2], [f32; 2], f32, [f32; 4])> {
    let num = (end - start).length();
    let rotation = (end.y - start.y).atan2(end.x - start.x);
    let mut num2 = (num as i32).min(5);
    if color_start == color_end {
        num2 = 1;
    }
    let mut out = Vec::new();
    for i in 0..num2 {
        let pos = start + (end - start) * (i as f32 / num2 as f32);
        let t = (i as f32 + 0.5) / num2 as f32;
        let color = [
            color_start[0] + (color_end[0] - color_start[0]) * t,
            color_start[1] + (color_end[1] - color_start[1]) * t,
            color_start[2] + (color_end[2] - color_start[2]) * t,
            color_start[3] + (color_end[3] - color_start[3]) * t,
        ];
        out.push((
            [pos.x, pos.y],
            [num / num2 as f32, width],
            rotation,
            color,
        ));
    }
    out
}

/// `Utils.DrawLine` (`Utils.cs:2689-2702`): the segment plan above pushed
/// through the port's solid-white pixel block (no `BlackTile` sheet — the
/// `ui/crafting.rs` precedent). XNA's origin `(0, 8)` is in SOURCE-texture
/// pixels and rides the scale `(num/num2/16, width/16)`, so the destination
/// anchor is `(0, width/2)` — the segment's left-middle.
fn draw_line(
    r: &mut SpriteRenderer,
    start: Vec2,
    end: Vec2,
    color_start: [f32; 4],
    color_end: [f32; 4],
    width: f32,
) {
    let uv = r.white_uv();
    for (pos, size, rot, color) in line_segments(start, end, color_start, color_end, width) {
        r.push_sprite_origin_rot(pos, size, uv, color, rot, [0.0, 8.0 * width / 16.0]);
    }
}

/// `Utils.DrawRectangle` (`Utils.cs:2619-2625`): four `DrawLine` calls —
/// start→bottom-left (left edge), start→top-right (top edge), end→bottom-left
/// (bottom edge), end→top-right (right edge), in that order.
fn draw_rectangle(
    r: &mut SpriteRenderer,
    start: Vec2,
    end: Vec2,
    color_start: [f32; 4],
    color_end: [f32; 4],
    width: f32,
) {
    draw_line(r, start, Vec2::new(start.x, end.y), color_start, color_end, width);
    draw_line(r, start, Vec2::new(end.x, start.y), color_start, color_end, width);
    draw_line(r, end, Vec2::new(start.x, end.y), color_start, color_end, width);
    draw_line(r, end, Vec2::new(end.x, start.y), color_start, color_end, width);
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

    /// `WiresUI.Settings.DrawWires` (`WiresUI.cs:27-45`): a held mech tool
    /// draws the overlay; an empty hand or a non-mech item draws nothing unless
    /// the lens/kite (`InfoAccMechShowWires`) is equipped with its toggle on.
    #[test]
    fn the_wire_overlay_gates_on_the_held_mech_tool_or_the_lens() {
        let status = [0; 12];
        // A held wrench (509, mech) draws even without the lens.
        assert!(wire_view(509, 1, true, false, &status, false, false).is_some());
        // An empty hand with no lens draws nothing.
        assert!(wire_view(0, 0, false, false, &status, false, false).is_none());
        // A non-mech item (dirt 2) with no lens draws nothing.
        assert!(wire_view(2, 1, false, false, &status, false, false).is_none());
        // The lens equipped (toggle[8] == 0) draws without a mech tool.
        assert!(wire_view(0, 0, false, true, &status, false, false).is_some());
        // The lens toggle off ([8] != 0) hides the overlay.
        let mut off = status;
        off[8] = 1;
        assert!(wire_view(0, 0, false, true, &off, false, false).is_none());
        // A held mech tool draws even with the lens toggle off.
        assert!(wire_view(509, 1, true, true, &off, false, false).is_some());
    }

    /// The no-traps seed hides the wires until Skeletron falls
    /// (`WiresUI.cs:31-34`).
    #[test]
    fn the_no_traps_seed_hides_the_wires_until_skeletron() {
        let status = [0; 12];
        assert!(wire_view(509, 1, true, false, &status, true, false).is_none());
        assert!(wire_view(509, 1, true, false, &status, true, true).is_some());
        // Without the seed the boss gate is irrelevant.
        assert!(wire_view(509, 1, true, false, &status, false, false).is_some());
    }

    /// `WiresUI.Settings.HideWires` (`WiresUI.cs:47-57`): the Actuation Rod
    /// (3620) in hand suppresses the wire colours; the channels default to
    /// normal (1) without the lens and read the saved toggles with it.
    #[test]
    fn the_rod_hides_the_wires_and_the_lens_sets_the_channels() {
        let status = [0; 12];
        let rod = wire_view(3620, 1, true, false, &status, false, false).unwrap();
        assert!(rod.hide_wires, "the rod hides the wire colours");
        assert_eq!(rod.channels, [1; 5], "no lens → normal channels");
        // The lens reads builderAccStatus[4/5/6/7/9].
        let mut lens = [0; 12];
        lens[4] = 2; // red faded
        lens[5] = 0; // blue bright
        lens[9] = 2; // actuator faded
        let v = wire_view(0, 0, false, true, &lens, false, false).unwrap();
        assert!(!v.hide_wires);
        assert_eq!(v.channels, [2, 0, 0, 0, 2]);
    }

    /// `Lighting.GetColor` (`Lighting.cs:216-242`): the byte truncation and
    /// the 255 clamp on the RAW sample × GlobalBrightness, alpha 255.
    #[test]
    fn the_light_bytes_truncate_and_clamp_like_get_color() {
        // 0.333 × 1.2 × 255 = 101.898 → 101 (truncation, not rounding).
        let c = light_color_bytes([0.333, 0.5, 1.0], 1.2);
        assert_eq!(c, [101, 153, 255, 255]);
        // Above 1.0 the product clamps at 255 (`:228-239`), never wraps.
        let hot = light_color_bytes([2.0, 1.5, 0.0], 1.2);
        assert_eq!(hot, [255, 255, 0, 255]);
        // The blind drop: GlobalBrightness 1.0 (`Lighting.cs:112-115`).
        let blind = light_color_bytes([0.5, 0.5, 0.5], 1.0);
        assert_eq!(blind, [127, 127, 127, 255], "(int)(0.5*255) = 127");
    }

    /// The per-channel tint switch on bytes (`Main.cs:58318-58329`):
    /// 0 → White, 2 → ×0.5 with per-channel truncation INCLUDING alpha
    /// (XNA `Color * float`), 3 → Transparent (hidden), 1 → unchanged.
    #[test]
    fn the_channel_tint_switch_is_verbatim() {
        let mut c = [101u8, 153, 200, 255];
        assert!(channel_tint(&mut c, 1));
        assert_eq!(c, [101, 153, 200, 255], "normal keeps the light colour");
        assert!(channel_tint(&mut c, 0));
        assert_eq!(c, [255; 4], "bright → White");
        assert!(channel_tint(&mut c, 2));
        assert_eq!(
            c,
            [127, 127, 127, 127],
            "faded → ×0.5 truncates the alpha too (255 → 127)"
        );
        let mut odd = [101u8, 153, 200, 255];
        assert!(channel_tint(&mut odd, 2));
        assert_eq!(odd, [50, 76, 100, 127], "(int)(101*0.5)=50, (int)(153*0.5)=76");
        assert!(!channel_tint(&mut odd, 3), "Transparent hides the wire");
    }

    /// The overlap dim is a BYTE scale (`Main.cs:58405`): `(int)(b * (1/num7))`
    /// per channel incl. alpha — 255/2 = 127, not 127.5.
    #[test]
    fn the_overlap_dim_truncates_the_bytes_including_alpha() {
        let c = scale_color([255, 101, 0, 255], 0.5);
        assert_eq!(c, [127.0 / 255.0, 50.0 / 255.0, 0.0, 127.0 / 255.0]);
        // Three colours: 255/3 = 85 exactly, alpha 85.
        let third = scale_color([255, 255, 255, 255], 1.0 / 3.0);
        assert_eq!(third, [85.0 / 255.0; 4]);
        // The identity scale keeps every byte (the red quad's path).
        assert_eq!(scale_color([101, 153, 200, 255], 1.0), [
            101.0 / 255.0,
            153.0 / 255.0,
            200.0 / 255.0,
            1.0
        ]);
    }

    /// The strips draw at the channel-tinted colour BEFORE the `1/num7` dim
    /// (`Main.cs:58405` vs `:58410`): on a shared tile the main quad is
    /// dimmed but the strip keeps the full tint (alpha included).
    #[test]
    fn the_strip_colour_escapes_the_overlap_dim() {
        let l = [0.5, 0.5, 0.5];
        let (main, strip) = wire_tint(l, 1.2, 1, 2.0).unwrap();
        // GetColor: (int)(0.5*306) = 153; main dimmed ×0.5 → 76, strip 153.
        assert_eq!(main, [76.0 / 255.0, 76.0 / 255.0, 76.0 / 255.0, 127.0 / 255.0]);
        assert_eq!(strip, [153.0 / 255.0, 153.0 / 255.0, 153.0 / 255.0, 1.0]);
        assert_ne!(main, strip, "the strip must NOT ride the dim");
        // A faded channel halves the strip too (the tint precedes the split).
        let (main2, strip2) = wire_tint(l, 1.2, 2, 2.0).unwrap();
        assert_eq!(strip2, [76.0 / 255.0, 76.0 / 255.0, 76.0 / 255.0, 127.0 / 255.0]);
        assert_eq!(main2, [38.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0, 63.0 / 255.0]);
        // Transparent: no draw, the caller refunds num7 (`:58330-58333`).
        assert!(wire_tint(l, 1.2, 3, 1.0).is_none());
        // Bright: White on both, the dim still bites the main quad.
        let (m3, s3) = wire_tint(l, 1.2, 0, 2.0).unwrap();
        assert_eq!(s3, [1.0; 4]);
        assert_eq!(m3, [127.0 / 255.0; 4]);
    }

    /// `Lighting.Brightness` (`Lighting.cs:118-122`): the RAW mean ×
    /// GlobalBrightness — the actuator gate (`Main.cs:58643`).
    #[test]
    fn the_actuator_gate_reads_the_raw_brightness_mean() {
        assert_eq!(lighting_brightness([0.0, 0.0, 0.0], 1.2), 0.0);
        assert!(lighting_brightness([0.0, 0.0, 0.01], 1.2) > 0.0);
        let b = lighting_brightness([0.3, 0.6, 0.9], 1.2);
        assert!((b - 1.2 * 1.8 / 3.0).abs() < 1e-6);
    }

    /// The 424/445 junction-box column offsets (`Main.cs:58268-58290`):
    /// `frameX / 18` picks 72/144/216 on the 424 box, the 445 pins 72.
    #[test]
    fn the_junction_box_offsets_match_the_frame_rows() {
        use tr_core::tile::{FLAG_ACTIVE, Tile};
        let mut t = Tile {
            kind: 424,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        for (fx, want) in [(0i16, 72.0f32), (18, 144.0), (36, 216.0), (54, 0.0)] {
            t.frame_x = fx;
            assert_eq!(junction_offset(&t), want, "424 frameX {fx}");
        }
        t.kind = 445;
        assert_eq!(junction_offset(&t), 72.0, "the 445 pins column 72");
        t.kind = 1;
        assert_eq!(junction_offset(&t), 0.0, "a plain tile keeps column 0");
        t.kind = 424;
        t.frame_x = 18;
        t.flags = 0;
        assert_eq!(junction_offset(&t), 0.0, "inactive tiles keep column 0");
    }

    /// `Player.CanShowWireStuffHere` (`Player.cs:46946-46961`): wall 87
    /// hides the wire colours until Golem falls; the actuator plate and the
    /// 423 rect sit OUTSIDE the guard (`Main.cs:58643`/`:58660` vs `:58266`).
    #[test]
    fn the_temple_wall_gates_only_the_wire_colours() {
        use tr_core::tile::FLAG_ACTIVE;
        let mut world = World::new("g81".into(), 8, 8);
        let t = world.tile_mut(2, 2).unwrap();
        t.wall = 87;
        t.flags = FLAG_ACTIVE;
        assert!(!can_show_wire_stuff_here(&world, 2, 2), "pre-Golem temple");
        world.downed_golem = true;
        assert!(can_show_wire_stuff_here(&world, 2, 2), "post-Golem opens");
        // The 423 collection predicate (`Main.cs:58660`) reads the tile
        // directly — no CanShowWireStuffHere — so a temple sensor collects.
        let t = world.tile_mut(3, 3).unwrap();
        t.wall = 87;
        t.kind = 423;
        t.frame_y = 36;
        t.flags = FLAG_ACTIVE;
        world.downed_golem = false;
        assert!(!can_show_wire_stuff_here(&world, 3, 3));
        let t = world.tile(3, 3).unwrap();
        assert!(t.active() && t.kind == 423 && t.frame_y == 36);
    }

    /// The strip table maps the C#'s draw order (up/down/right/left,
    /// `flag8/9/7/6`) onto the `[up, right, down, left]` side indices —
    /// the down strip must check the DOWN claim, not the right one.
    #[test]
    fn the_strip_rows_check_their_own_side() {
        // STRIPS rows: (side index, src x, src y-off, w, h, draw ox, oy).
        assert_eq!(STRIPS[0], (0, 18.0, 0.0, 16.0, 6.0, 0.0, 0.0), "up");
        assert_eq!(STRIPS[1], (2, 72.0, 10.0, 16.0, 6.0, 0.0, 10.0), "down");
        assert_eq!(STRIPS[2], (1, 46.0, 0.0, 6.0, 16.0, 10.0, 0.0), "right");
        assert_eq!(STRIPS[3], (3, 144.0, 0.0, 6.0, 16.0, 0.0, 0.0), "left");
    }

    /// The 423 range rectangle corners (`Main.cs:58671`): the ±1 px outset,
    /// the 32/48 px side reach and the 160 px upward reach.
    #[test]
    fn the_sensor_range_rect_corners_are_verbatim() {
        let (start, end) = special_tile_rect(100, 50);
        assert_eq!(start, Vec2::new(1600.0 - 33.0, 800.0 - 161.0));
        assert_eq!(end, Vec2::new(1600.0 + 49.0, 800.0 + 1.0));
        assert_eq!(end - start, Vec2::new(82.0, 162.0));
    }

    /// `Utils.DrawLine` (`Utils.cs:2689-2702`): equal colours collapse to
    /// ONE full-length segment at `start`; the gradient path caps at 5
    /// segments with the `(i+0.5)/num2` colour lerp.
    #[test]
    fn the_line_segments_collapse_for_solid_colours() {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(82.0, 0.0);
        let segs = line_segments(start, end, LIGHT_SEA_GREEN, LIGHT_SEA_GREEN, 2.0);
        assert_eq!(segs.len(), 1, "colorStart == colorEnd → num2 = 1");
        let (pos, size, rot, color) = segs[0];
        assert_eq!(pos, [0.0, 0.0], "Lerp(start, end, 0) = start");
        assert_eq!(size, [82.0, 2.0], "num/num2 × width");
        assert_eq!(rot, 0.0, "ToRotation of +X");
        assert_eq!(color, LIGHT_SEA_GREEN);
        // A vertical edge rotates by π/2 (`(end-start).ToRotation()`).
        let down = line_segments(start, Vec2::new(0.0, 162.0), LIGHT_SEA_GREEN, LIGHT_SEA_GREEN, 2.0);
        assert!((down[0].2 - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // The gradient path: min(5, (int)distance) segments, mid-lerp colours.
        let grad = line_segments(start, end, [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], 2.0);
        assert_eq!(grad.len(), 5);
        assert_eq!(grad[0].0, [0.0, 0.0]);
        assert_eq!(grad[1].0, [82.0 / 5.0, 0.0]);
        assert!((grad[0].3[0] - 0.1).abs() < 1e-6, "(0+0.5)/5 = 0.1");
        assert_eq!(grad[0].1, [82.0 / 5.0, 2.0]);
    }
}
