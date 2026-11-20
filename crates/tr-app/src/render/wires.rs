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
//! Deviations: `DrawWiresSpecialTiles` (the tile-423 LightSeaGreen rectangle,
//! `:58660-58672`) is content-gated — the port has no tile 423.
//! `Player.CanShowWireStuffHere` keeps the pre-Golem wall-87 guard
//! (`Player.cs:46946-46961`). The per-colour strip side-claims use the port's
//! unified `claimed` walk (the C# red wire claims unconditionally, the later
//! colours only on draw — a micro-asymmetry unreachable in practice, since the
//! UI cycle wraps at 3 so Transparent is never selected, `Main.cs:47551-47555`).

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

/// The per-channel tint switch (`Main.cs:58318-58329` and the colour twins,
/// the actuator `:58646-58657`): 0 → White, 2 → ×0.5, 3 → Transparent (the
/// caller skips the draw), 1/default → the light colour unchanged. Returns
/// `false` for Transparent (the wire is hidden and `num7` must decrement).
fn channel_tint(c: &mut [f32; 3], channel: i32) -> bool {
    match channel {
        0 => *c = [1.0, 1.0, 1.0],
        2 => {
            for v in c.iter_mut() {
                *v *= 0.5;
            }
        }
        3 => return false,
        _ => {}
    }
    true
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

    let wires = &world.wires;
    let colours = [
        (WireColor::Red, 0.0, 0usize),
        (WireColor::Blue, 18.0, 1),
        (WireColor::Green, 36.0, 2),
        (WireColor::Yellow, 54.0, 3),
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
            // The junction-box column offset (`num8`, `Main.cs:58268-58290`).
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
            // `Main.cs:58266`: the wire colours draw only when `flag` rides
            // (`HideWires` off) and the tile passes `CanShowWireStuffHere`.
            if draw_wire_colours {
                for &(colour, row, ch) in &colours {
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
                    // The per-channel tint (`Main.cs:58318-58329`): Transparent
                    // hides the wire and gives its `num7` count back.
                    if !channel_tint(&mut c, view.channels[ch]) {
                        num7 -= 1.0;
                        continue;
                    }
                    if num7 > 1.0 {
                        // `color2 * (1f / num7)` (`Main.cs:58405` and the colour
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
                    // The 6 px inset strips (`Main.cs:58406-58437` and the
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
            }
            // The actuator plate (`Main.cs:58643-58658`): a full-tile quad on
            // every actuator-carrying tile that receives light OR whose channel
            // is Bright (`num6 == 0`). The channel tint applies, then `* num`
            // (the rod's ×0.5). Unlike the wires this draws even under
            // `HideWires` — the C# block sits outside the `flag` guard.
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if t.flags & FLAG_ACTUATOR != 0 {
                let l = light.get(tx, ty);
                let bright = l.iter().any(|v| *v > 0.0);
                if bright || view.channels[4] == 0 {
                    let mut c = style.shade(l, tx, ty, anim_time);
                    if channel_tint(&mut c, view.channels[4]) {
                        for v in &mut c {
                            *v *= actuator_scale;
                        }
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

    /// The per-channel tint switch (`Main.cs:58318-58329`): 0 → White,
    /// 2 → ×0.5, 3 → Transparent (hidden), 1 → unchanged.
    #[test]
    fn the_channel_tint_switch_is_verbatim() {
        let mut c = [0.4, 0.6, 0.8];
        assert!(channel_tint(&mut c, 1));
        assert_eq!(c, [0.4, 0.6, 0.8], "normal keeps the light colour");
        assert!(channel_tint(&mut c, 0));
        assert_eq!(c, [1.0, 1.0, 1.0], "bright → White");
        assert!(channel_tint(&mut c, 2));
        assert_eq!(c, [0.5, 0.5, 0.5], "faded → ×0.5");
        assert!(!channel_tint(&mut c, 3), "Transparent hides the wire");
    }
}
