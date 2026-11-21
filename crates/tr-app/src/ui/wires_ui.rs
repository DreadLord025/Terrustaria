//! The wiring tool-mode UI (S1-W4-g81 residuals) — `WiresUI.WiresRadial`
//! (`GameContent/UI/WiresUI.cs:98-692`), drawn by `HandleWiresUI` (`:698-702`)
//! from `Main.DrawInterface_9_WireSelection` (`Main.cs:46196-46204`), the
//! interface layer between the town-house banners (7) and the capture check
//! (10) — under the inventory/hotbar/cursor layers.
//!
//! Three visuals, one state machine:
//! * the radial FLOWER (`DrawFlower`, `:342-509`) — the right-click petal
//!   menu that XORs the `ToolMode` channel bits;
//! * the cursor-area channel BADGES (`DrawCursorArea`, `:511-691`) — the
//!   four wire-colour dots + the actuator dot beside the cursor, dimmed
//!   when their channel is off, with the cutter's X overlay;
//! * the fade-in LINE state (`LineUpdate`, `:114-148`) — the 0.05/tick
//!   opacity ramp the badges ride.
//!
//! `WiresRadial.DrawLine` (`:195-340`) is DEAD CODE in the C# — `Draw`
//! (`:189-193`) calls only `DrawFlower` + `DrawCursorArea`, and no other
//! site calls it (verified by tree grep). Not ported; do not "fix" this
//! by wiring it in.
//!
//! Deviations (recorded in ANALYSIS "The wiring tool-mode UI (S1-W4-g81)"):
//! * the gamepad branches fold out — `PlayerInput.UsingGamepad` is always
//!   false in the port (the `lang.rs:230` precedent);
//! * `Player.cursorItemIconEnabled`/`cursorItemIconID` (the
//!   `DrawInterface_40` cursor-icon system) are unported; the one read
//!   that matters here — the Multicolor Wrench's in-reach icon latch
//!   (`Player.cs:46756-46760`) — rides the `icon_enabled` stand-in the
//!   caller computes from the held kind + reach;
//! * `Main.HoveringOverAnNPC` / `Main.SmartInteractShowingGenuine` /
//!   `player.lastMouseInterface` have no port state — false stand-ins
//!   (the smart-interact subsystem is a recorded unported leftover,
//!   ANALYSIS "Smart cursor (M7-SMARTCURSOR-1)");
//! * the `mouseInterface = true` hover write folds into the port's
//!   `over_ui_slot` mine-suppression stand-in through [`WiresRadial::hovering`].

use glam::Vec2;
use tr_core::smart_cursor::multi_tool_mode as mode;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::ui::color;

/// `WiresUI.Settings.MultiToolMode`'s default (`WiresUI.cs:92-95`): the
/// static ctor seeds `ToolMode = Red`.
pub const TOOL_MODE_DEFAULT: u8 = mode::RED;

/// `WiresUI.WiresRadial` (`WiresUI.cs:98-106`) + the `Settings`
/// `_lastActuatorEnabled` latch (`:25`). The `ToolMode` bitfield itself
/// lives on the App (`wire_tool_mode`) — the smart-cursor lookup reads it
/// (`Lookup::tool_mode`, the M7-SMARTCURSOR-1 seam).
#[derive(Default)]
pub struct WiresRadial {
    /// `WiresRadial.position` (`:100`): the flower's anchor — the mouse
    /// screen point at the right-click that opened it.
    pub position: Vec2,
    /// `WiresRadial.active` (`:102`): the flower is open.
    pub active: bool,
    /// `WiresRadial.OnWiresMenu` (`:104`): a petal was hovered this frame
    /// (the `mouseInterface` write's source).
    pub on_wires_menu: bool,
    /// `WiresRadial._lineOpacity` (`:106`): the badge fade, 0..=1.
    line_opacity: f32,
    /// The per-frame `mouseInterface` stand-in: true while a petal rides
    /// the pointer (the C# writes `player.mouseInterface = true` at
    /// `:499`; the port folds it into `over_ui_slot`).
    pub hovering: bool,
    /// `Settings._lastActuatorEnabled` (`WiresUI.cs:25`): 2 after the
    /// Grand Design (3611) was held, 1 after the Multicolor Wrench
    /// (3625) — the actuator petal/badge shows only while it is 2.
    pub last_actuator_enabled: i32,
}

/// `Settings.DrawToolModeUI` (`WiresUI.cs:59-73`): the Grand Design
/// (3611) or the Multicolor Wrench (3625) in hand (non-air).
pub fn draw_tool_mode_ui(held_kind: u16, held_count: u32) -> bool {
    held_count > 0 && (held_kind == 3611 || held_kind == 3625)
}

/// `Settings.DrawToolAllowActuators` (`WiresUI.cs:75-90`): the getter's
/// SIDE EFFECT — holding 3611 latches 2, holding 3625 latches 1 — then
/// returns `last == 2`. The C# re-runs the getter at every read site
/// within one frame; the selected item cannot change mid-draw, so one
/// call per frame is byte-identical.
pub fn draw_tool_allow_actuators(held_kind: u16, held_count: u32, last: &mut i32) -> bool {
    if held_count > 0 && held_kind == 3611 {
        *last = 2;
    }
    if held_count > 0 && held_kind == 3625 {
        *last = 1;
    }
    *last == 2
}

/// The per-frame inputs `HandleWiresUI` reads off `Main`/the player —
/// snapshotted by the frame before the draw destructure (the
/// `pvp_icon_state` precedent).
pub struct RadialInputs {
    /// `Main.MouseScreen`.
    pub mouse: Vec2,
    /// `Main.screenWidth/Height` — the line's bottom-centre anchor.
    pub screen: Vec2,
    pub held_kind: u16,
    pub held_count: u32,
    /// `Player.dead` (`:124`/`:163`).
    pub dead: bool,
    /// `Main.mouseItem.type > 0` (`:124`/`:163`).
    pub mouse_item: bool,
    /// The `cursorItemIconEnabled` stand-in: the Multicolor Wrench's
    /// in-reach icon latch (`Player.cs:46756-46760`); the icon ID stays
    /// 0 for the wiring tools (`ItemIconCacheUpdate` writes only the
    /// cache, `Main.cs:46515-46520`).
    pub icon_enabled: bool,
    /// `Main.drawingPlayerChat` (`:119`).
    pub chat_open: bool,
    /// `Player.mouseInterface` (`:136`/`:158`) — the port's slot-surface
    /// stand-in.
    pub mouse_interface: bool,
    /// `Main.ingameOptionsWindow || Main.InGameUI.IsVisible` (`:136`).
    pub options_or_fancy: bool,
    /// `Main.mouseRight && Main.mouseRightRelease` (`:170`) — the
    /// press-after-release frame (the buff-bar gate's conjunction).
    pub right_press: bool,
    /// `Main.mouseLeft && Main.mouseLeftRelease` (`:303`/`:475`/`:505`).
    pub left_press: bool,
    /// `Player.noThrow != 0` (`:170`).
    pub no_throw: bool,
    /// `Player.talkNPC != -1` (`:170`).
    pub talking: bool,
}

impl WiresRadial {
    /// `WiresRadial.Update` (`WiresUI.cs:108-112`): `FlowerUpdate` then
    /// `LineUpdate`, in that order.
    pub fn update(&mut self, tool_mode_ui: bool, allow_actuators: bool, inp: &RadialInputs) {
        self.flower_update(tool_mode_ui, inp);
        self.line_update(tool_mode_ui, allow_actuators, inp);
    }

    /// `WiresRadial.FlowerUpdate` (`WiresUI.cs:150-187`). The gamepad
    /// centre-anchor arm (`:182-185`) folds out (no gamepad).
    fn flower_update(&mut self, tool_mode_ui: bool, inp: &RadialInputs) {
        if !tool_mode_ui {
            self.active = false;
            return;
        }
        if inp.mouse_interface && !self.on_wires_menu {
            self.active = false;
            return;
        }
        if inp.dead || inp.mouse_item {
            self.active = false;
            self.on_wires_menu = false;
            return;
        }
        self.on_wires_menu = false;
        // `:170` — the right-click press edge; `LockGamepadTileUseButton`
        // is false (no gamepad), `HoveringOverAnNPC`/
        // `SmartInteractShowingGenuine` are false stand-ins.
        if !inp.right_press || inp.no_throw || inp.talking {
            return;
        }
        if self.active {
            self.active = false;
        } else {
            self.active = true;
            self.position = inp.mouse;
        }
    }

    /// `WiresRadial.LineUpdate` (`WiresUI.cs:114-148`). The double
    /// `0.05 * Sign` step is verbatim: the clamp targets `num`, the
    /// opacity moves one 0.05 sign-step toward it, and snaps when the
    /// remainder is under one step.
    fn line_update(&mut self, tool_mode_ui: bool, allow_actuators: bool, inp: &RadialInputs) {
        let mut value = true;
        let mut min = 0.75f32;
        if !tool_mode_ui || inp.chat_open {
            value = false;
            min = 0.0;
        }
        if inp.dead || inp.mouse_item {
            self.line_opacity = 0.0;
            return;
        }
        // `:130` — `cursorItemIconEnabled && ID != 0 && ID != 3625`: the
        // wiring tools keep ID 0, so the stand-in never trips this gate.
        // `:136` — with no gamepad the clause reduces to
        // `!icon_enabled && (!allow || mouseInterface || lastMouseInterface)`
        // (lastMouseInterface: false stand-in).
        if (!inp.icon_enabled && (!allow_actuators || inp.mouse_interface))
            || inp.options_or_fancy
        {
            self.line_opacity = 0.0;
            return;
        }
        let dir = if value { 1.0 } else { -1.0 }; // ToDirectionInt (`Utils.cs:2010-2017`)
        let num = (self.line_opacity + 0.05 * dir).clamp(min, 1.0);
        self.line_opacity += 0.05 * (num - self.line_opacity).signum();
        if (self.line_opacity - num).abs() < 0.05 {
            self.line_opacity = num;
        }
    }

    /// `WiresRadial.Draw` (`WiresUI.cs:189-193`): the flower then the
    /// cursor badges. `tool_mode` is XORed in place by the petal clicks
    /// (`:475-498`); returns nothing — the hover write rides
    /// [`Self::hovering`].
    pub fn draw(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        tool_mode: &mut u8,
        allow_actuators: bool,
        inp: &RadialInputs,
    ) {
        self.hovering = false;
        self.draw_flower(r, lookups, tool_mode, allow_actuators, inp);
        self.draw_cursor_area(r, lookups, *tool_mode, allow_actuators, inp);
    }

    /// The petal anchor (`DrawFlower`, `WiresUI.cs:374-401`): the four
    /// colour petals ride `UnitX.RotatedBy(num5 * 2π/num3 - π/num4) * 45`
    /// with the 0↔3 index swap; the actuator petal uses `(i-1) = 4`; the
    /// cutter sits at the centre. `num3 = 4 + allow.ToInt()` (0/1 — NOT
    /// ToDirectionInt), `num4 = allow ? 11 : -0.5`.
    pub fn petal_offset(i: usize, allow_actuators: bool) -> Vec2 {
        let num3 = 4.0 + if allow_actuators { 1.0 } else { 0.0 };
        let num4 = if allow_actuators { 11.0 } else { -0.5 };
        let angle = match i {
            0..=3 => {
                let num5 = match i {
                    0 => 3.0,
                    3 => 0.0,
                    _ => i as f32,
                };
                num5 * std::f32::consts::TAU / num3 - std::f32::consts::PI / num4
            }
            5 => 4.0 * std::f32::consts::TAU / num3 - std::f32::consts::PI / num4,
            _ => return Vec2::ZERO, // i == 4 — the centre (`:399`)
        };
        // `Utils.RotatedBy` (`Utils.cs:1675-1684`) on UnitX * 45.
        Vec2::new(angle.cos() * 45.0, angle.sin() * 45.0)
    }

    /// The petal's `ToolMode` bit (`DrawFlower`'s click switch,
    /// `WiresUI.cs:477-497`): i1 toggles GREEN, i2 BLUE — the draw order
    /// is red/green/blue/yellow, NOT the bit order (verbatim).
    fn petal_bit(i: usize) -> u8 {
        match i {
            0 => mode::RED,
            1 => mode::GREEN,
            2 => mode::BLUE,
            3 => mode::YELLOW,
            4 => mode::CUTTER,
            _ => mode::ACTUATOR,
        }
    }

    /// `WiresRadial.DrawFlower` (`WiresUI.cs:342-509`).
    fn draw_flower(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        tool_mode: &mut u8,
        allow_actuators: bool,
        inp: &RadialInputs,
    ) {
        if !self.active {
            return;
        }
        let vector = inp.mouse;
        let vector2 = self.position;
        let num2 = (vector - vector2).length();
        let cutter = *tool_mode & mode::CUTTER != 0;
        let mut any_hover = false;
        for i in 0..6usize {
            if !allow_actuators && i == 5 {
                continue;
            }
            // `:369-373` — the raw `1 << i` test, with the i==5 override
            // to the ACTUATOR bit (the raw test would read Cutter).
            let mut on = *tool_mode & (1u8 << i) != 0;
            if i == 5 {
                on = *tool_mode & mode::ACTUATOR != 0;
            }
            let vector4 = vector2 + Self::petal_offset(i, allow_actuators);
            // `:402-429` — the angle-cone test computes, then the
            // non-gamepad distance test OVERRIDES it (verbatim: the
            // cone is dead under the mouse). i==4: `num2 < 20`.
            let mut hover = if i == 4 {
                num2 < 20.0
            } else {
                (vector4 - vector).length() < 19.0
            };
            if any_hover {
                hover = false;
            }
            if hover {
                any_hover = true;
            }
            // `:438-453` — the petal base (cutter swaps 0/1 → 8/9) and
            // the glyph: colours 2+i, cutter 7 vs 6, actuator 10.
            let petal = lookups.wire_ui[(if cutter { 8 } else { 0 }) + hover as usize];
            let glyph = lookups.wire_ui[match i {
                0..=3 => 2 + i,
                4 => {
                    if cutter {
                        7
                    } else {
                        6
                    }
                }
                _ => 10,
            }];
            // `:455-471` — the off-channel tints. The C# assigns color2
            // twice ((100..) then (120..) / (150..) then (80..)); only
            // the second assignment survives (verbatim result).
            let (mut c_petal, mut c_glyph) = ([255u8; 4], [255u8; 4]);
            if !on && i != 4 {
                if hover {
                    c_glyph = [120, 120, 120, 255];
                    c_petal = [200, 200, 200, 255];
                } else {
                    c_glyph = [80, 80, 80, 255];
                    c_petal = [100, 100, 100, 255];
                }
            }
            if hover {
                if inp.left_press {
                    *tool_mode ^= Self::petal_bit(i);
                }
                // `:499-500` — the flower writes mouseInterface
                // unconditionally on hover (the dead DrawLine's
                // conditional form is NOT this one).
                self.on_wires_menu = true;
                self.hovering = true;
            }
            // `:502-503` — both quads at scale 1, origin Size/2.
            push_centred(r, lookups, petal, vector4, 1.0, color::tint(c_petal));
            push_centred(r, lookups, glyph, vector4, 1.0, color::tint(c_glyph));
        }
        if inp.left_press && !any_hover {
            self.active = false;
        }
    }

    /// The badge's channel index (`DrawCursorArea`, `WiresUI.cs:556-586`):
    /// the i→num4 map (0→3, 1→2, 2→1, 3→0, 4→5) then the num5 1↔2 swap —
    /// net i0→3 (yellow), i1→1 (green bit), i2→2 (blue bit), i3→0 (red),
    /// i4→5 (actuator). The DISPLAY order left-to-right is
    /// yellow/green/blue/red — verbatim.
    pub fn badge_channel(i: usize) -> usize {
        let num4 = match i {
            0 => 3,
            1 => 2,
            2 => 1,
            3 => 0,
            _ => 5,
        };
        match num4 {
            1 => 2,
            2 => 1,
            _ => num4,
        }
    }

    /// The badge colour by num4 (`WiresUI.cs:592-610`): red (253,58,61),
    /// blue (83,180,253), green (83,253,153), yellow (253,254,83),
    /// actuator WhiteSmoke (245,245,245); the HotPink default is
    /// unreachable (num4 ∈ {0,1,2,3,5}).
    pub fn badge_color(num4: usize) -> [u8; 4] {
        match num4 {
            0 => [253, 58, 61, 255],
            1 => [83, 180, 253, 255],
            2 => [83, 253, 153, 255],
            3 => [253, 254, 83, 255],
            _ => [245, 245, 245, 255],
        }
    }

    /// The badge offset (`WiresUI.cs:643-682`, the `flag == false` arms —
    /// `flag` is the constant false at `:538`, the cursorScale branch is
    /// dead): the actuator layout is the diamond ±12/±6, the plain layout
    /// the 0.7-scaled cross.
    pub fn badge_offset(num4: usize, allow_actuators: bool) -> Vec2 {
        if allow_actuators {
            match num4 {
                0 => Vec2::new(-12.0, 0.0),
                3 => Vec2::new(12.0, 0.0),
                1 => Vec2::new(-6.0, 12.0),
                2 => Vec2::new(6.0, 12.0),
                _ => Vec2::ZERO,
            }
        } else {
            let n = 0.7;
            match num4 {
                0 => Vec2::new(0.0, -12.0 * n),
                3 => Vec2::new(12.0 * n, 0.0),
                1 => Vec2::new(-12.0 * n, 0.0),
                2 => Vec2::new(0.0, 12.0 * n),
                _ => Vec2::ZERO,
            }
        }
    }

    /// `WiresRadial.DrawCursorArea` (`WiresUI.cs:511-691`).
    fn draw_cursor_area(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        tool_mode: u8,
        allow_actuators: bool,
        inp: &RadialInputs,
    ) {
        if self.active || self.line_opacity == 0.0 {
            return;
        }
        let Some(sheet) = lookups.builder_acc else {
            return;
        };
        // `:517-530` — the anchor: mouse + (10,25); without the actuator
        // petal the non-gamepad arm adds (-20,10).
        let mut anchor = inp.mouse + Vec2::new(10.0, 25.0);
        if !allow_actuators {
            anchor += Vec2::new(-20.0, 10.0);
        }
        let grey = [50u8, 50, 50, 255];
        let cutter = tool_mode & mode::CUTTER != 0;
        let num2 = self.line_opacity;
        for i in 0..5usize {
            if !allow_actuators && i == 4 {
                continue;
            }
            let num5 = Self::badge_channel(i);
            let num4 = match i {
                0 => 3,
                1 => 2,
                2 => 1,
                3 => 0,
                _ => 5,
            };
            // `:587-591` — the raw `1 << num5` test with the num5==5
            // actuator override.
            let mut on = tool_mode & (1u8 << num5) != 0;
            if num5 == 5 {
                on = tool_mode & mode::ACTUATOR != 0;
            }
            let mut c = Self::badge_color(num4);
            if !on {
                // `:613` — Lerp(color, Black, 0.65): XNA's byte lerp.
                c = lerp_to_black_65(c);
            }
            let vec = (anchor + Self::badge_offset(num4, allow_actuators)).floor();
            // `:684-689` — the 10x10 back plate (128,0), the 6x6 channel
            // dot (140,2), and the cutter's 6x6 X (148,2) in the grey.
            push_frame(r, lookups, sheet, vec, (128, 0, 10, 10), scale_rgba(grey, num2));
            push_frame(r, lookups, sheet, vec, (140, 2, 6, 6), scale_rgba(c, num2));
            if cutter {
                push_frame(r, lookups, sheet, vec, (148, 2, 6, 6), scale_rgba(grey, num2));
            }
        }
    }
}

/// `Color.Lerp(c, Color.Black, 0.65)` == `c * 0.35` per byte — the exact
/// XNA form is `c + (0 - c) * 0.65` truncated; `color::mul`'s `(byte *
/// 0.35) as u8` can differ by one ULP from the lerp form on rounding
/// boundaries, so the badge dim uses the verbatim lerp arithmetic.
fn lerp_to_black_65(c: [u8; 4]) -> [u8; 4] {
    [
        (c[0] as f32 + (0.0 - c[0] as f32) * 0.65) as u8,
        (c[1] as f32 + (0.0 - c[1] as f32) * 0.65) as u8,
        (c[2] as f32 + (0.0 - c[2] as f32) * 0.65) as u8,
        (c[3] as f32 + (0.0 - c[3] as f32) * 0.65) as u8,
    ]
}

/// `Color * float` — XNA's `(int)(byte * scale)` on EVERY channel incl.
/// alpha (the `render/wires.rs` byte-scale precedent).
fn scale_rgba(c: [u8; 4], scale: f32) -> [f32; 4] {
    [
        (c[0] as f32 * scale) as u8 as f32 / 255.0,
        (c[1] as f32 * scale) as u8 as f32 / 255.0,
        (c[2] as f32 * scale) as u8 as f32 / 255.0,
        (c[3] as f32 * scale) as u8 as f32 / 255.0,
    ]
}

/// One whole-sheet quad centred on `pos` at `scale` (the radial's
/// `origin = Size/2` draws, `WiresUI.cs:502-503`).
fn push_centred(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<tr_render::SheetRect>,
    pos: Vec2,
    scale: f32,
    tint: [f32; 4],
) {
    let Some(rect) = sheet else { return };
    let Some(uv) = lookups.hud_uv(Some(&rect)) else {
        return;
    };
    let w = rect.width as f32 * scale;
    let h = rect.height as f32 * scale;
    r.push_ui_sprite([pos.x - w * 0.5, pos.y - h * 0.5], [w, h], uv, tint);
}

/// One source-rect quad of the BuilderIcons sheet, centred on `pos`
/// (`origin = rectangle.Size()/2`, scale 1 — `WiresUI.cs:684-688`).
fn push_frame(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: tr_render::SheetRect,
    pos: Vec2,
    (sx, sy, sw, sh): (i32, i32, i32, i32),
    tint: [f32; 4],
) {
    let aw = lookups.atlas_size.0 as f32;
    let ah = lookups.atlas_size.1 as f32;
    let x0 = sheet.x as f32 + sx as f32;
    let y0 = sheet.y as f32 + sy as f32;
    let uv = [x0 / aw, y0 / ah, (x0 + sw as f32) / aw, (y0 + sh as f32) / ah];
    r.push_ui_sprite(
        [pos.x - sw as f32 * 0.5, pos.y - sh as f32 * 0.5],
        [sw as f32, sh as f32],
        uv,
        tint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values derived from `WiresUI.cs` — never from this code.

    #[test]
    fn the_tool_mode_default_is_red() {
        // `WiresUI.cs:92-95`: the static ctor seeds Red.
        assert_eq!(TOOL_MODE_DEFAULT, 1);
    }

    #[test]
    fn the_actuator_latch_follows_the_held_tool() {
        // `WiresUI.cs:79-88`: 3611 latches 2, 3625 latches 1, the getter
        // returns last == 2 — the latch survives the tool switch-away.
        let mut last = 0;
        assert!(!draw_tool_allow_actuators(509, 1, &mut last));
        assert!(draw_tool_allow_actuators(3611, 1, &mut last));
        assert_eq!(last, 2);
        assert!(!draw_tool_allow_actuators(3625, 1, &mut last));
        assert_eq!(last, 1);
        assert!(!draw_tool_allow_actuators(509, 1, &mut last), "the latch holds 1");
        assert!(draw_tool_allow_actuators(3611, 1, &mut last));
        // An air slot never writes the latch (`HeldItem` non-air reads).
        assert!(draw_tool_allow_actuators(3625, 0, &mut last), "air keeps 2");
    }

    #[test]
    fn the_tool_mode_ui_gates_on_the_two_multi_tools() {
        // `WiresUI.cs:59-73`.
        assert!(draw_tool_mode_ui(3611, 1));
        assert!(draw_tool_mode_ui(3625, 1));
        assert!(!draw_tool_mode_ui(509, 1));
        assert!(!draw_tool_mode_ui(3611, 0));
    }

    #[test]
    fn the_petal_click_bits_swap_green_and_blue() {
        // `WiresUI.cs:477-497`: i1 → Green (2), i2 → Blue (4) — the draw
        // order is red/green/blue/yellow, not the bit order.
        assert_eq!(WiresRadial::petal_bit(0), mode::RED);
        assert_eq!(WiresRadial::petal_bit(1), mode::GREEN);
        assert_eq!(WiresRadial::petal_bit(2), mode::BLUE);
        assert_eq!(WiresRadial::petal_bit(3), mode::YELLOW);
        assert_eq!(WiresRadial::petal_bit(4), mode::CUTTER);
        assert_eq!(WiresRadial::petal_bit(5), mode::ACTUATOR);
    }

    #[test]
    fn the_petal_angles_match_the_rotated_unit_x() {
        // `WiresUI.cs:361-395`: num3 = 5, num4 = 11 with actuators.
        // i0 (num5=3): angle 3*2π/5 - π/11; the offset is 45 px along it.
        let a = 3.0 * std::f32::consts::TAU / 5.0 - std::f32::consts::PI / 11.0;
        let off = WiresRadial::petal_offset(0, true);
        assert!((off.x - a.cos() * 45.0).abs() < 1e-3);
        assert!((off.y - a.sin() * 45.0).abs() < 1e-3);
        // Without actuators: num3 = 4, num4 = -0.5 → -π/num4 = +2π, a
        // full turn — i0 (num5=3) lands at 3*2π/4 + 2π ≡ 270°.
        let off = WiresRadial::petal_offset(0, false);
        assert!(off.x.abs() < 1e-2, "x {}", off.x);
        assert!((off.y + 45.0).abs() < 1e-2, "y {}", off.y);
        // The cutter petal is the centre.
        assert_eq!(WiresRadial::petal_offset(4, true), Vec2::ZERO);
    }

    #[test]
    fn the_badge_channel_map_is_the_double_swap() {
        // `WiresUI.cs:556-586`: i0→num4 3→num5 3, i1→2→1, i2→1→2,
        // i3→0→0, i4→5→5.
        assert_eq!(WiresRadial::badge_channel(0), 3);
        assert_eq!(WiresRadial::badge_channel(1), 1);
        assert_eq!(WiresRadial::badge_channel(2), 2);
        assert_eq!(WiresRadial::badge_channel(3), 0);
        assert_eq!(WiresRadial::badge_channel(4), 5);
    }

    #[test]
    fn the_badge_colours_are_the_csharp_table() {
        // `WiresUI.cs:592-610`.
        assert_eq!(WiresRadial::badge_color(0), [253, 58, 61, 255]);
        assert_eq!(WiresRadial::badge_color(1), [83, 180, 253, 255]);
        assert_eq!(WiresRadial::badge_color(2), [83, 253, 153, 255]);
        assert_eq!(WiresRadial::badge_color(3), [253, 254, 83, 255]);
        assert_eq!(WiresRadial::badge_color(5), [245, 245, 245, 255]);
    }

    #[test]
    fn the_off_badge_dims_by_the_verbatim_lerp() {
        // `Color.Lerp((253,58,61,255), Black, 0.65)`: 253*0.35 = 88.55 →
        // 88, 58*0.35 = 20.3 → 20, 61*0.35 = 21.35 → 21, 255*0.35 =
        // 89.25 → 89 (XNA truncates the lerp result per byte).
        assert_eq!(lerp_to_black_65([253, 58, 61, 255]), [88, 20, 21, 89]);
    }

    #[test]
    fn the_badge_offsets_are_the_diamond_and_the_scaled_cross() {
        // `WiresUI.cs:643-682`.
        assert_eq!(WiresRadial::badge_offset(0, true), Vec2::new(-12.0, 0.0));
        assert_eq!(WiresRadial::badge_offset(1, true), Vec2::new(-6.0, 12.0));
        assert_eq!(WiresRadial::badge_offset(5, true), Vec2::ZERO);
        assert_eq!(WiresRadial::badge_offset(0, false), Vec2::new(0.0, -8.4));
        assert_eq!(WiresRadial::badge_offset(2, false), Vec2::new(0.0, 8.4));
    }

    #[test]
    fn the_line_opacity_ramps_in_005_steps_and_snaps() {
        // `WiresUI.cs:142-147`: from 0 with value=true, min 0.75 — the
        // clamp target jumps to 0.75, the opacity climbs 0.05/tick and
        // snaps at the remainder < 0.05.
        let mut w = WiresRadial::default();
        let inp = RadialInputs {
            mouse: Vec2::ZERO,
            screen: Vec2::new(1280.0, 720.0),
            held_kind: 3611,
            held_count: 1,
            dead: false,
            mouse_item: false,
            icon_enabled: false,
            chat_open: false,
            mouse_interface: false,
            options_or_fancy: false,
            right_press: false,
            left_press: false,
            no_throw: false,
            talking: false,
        };
        w.line_update(true, true, &inp);
        assert!((w.line_opacity - 0.05).abs() < 1e-6);
        for _ in 0..14 {
            w.line_update(true, true, &inp);
        }
        assert!((w.line_opacity - 0.75).abs() < 1e-6, "got {}", w.line_opacity);
        // The fade-out target is 0 (min 0 when the UI gate closes).
        w.line_update(false, true, &inp);
        assert!((w.line_opacity - 0.70).abs() < 1e-6);
    }

    #[test]
    fn the_flower_opens_on_the_right_press_and_toggles_closed() {
        // `WiresUI.cs:170-186`.
        let mut w = WiresRadial::default();
        let mut inp = RadialInputs {
            mouse: Vec2::new(400.0, 300.0),
            screen: Vec2::new(1280.0, 720.0),
            held_kind: 3611,
            held_count: 1,
            dead: false,
            mouse_item: false,
            icon_enabled: false,
            chat_open: false,
            mouse_interface: false,
            options_or_fancy: false,
            right_press: true,
            left_press: false,
            no_throw: false,
            talking: false,
        };
        w.flower_update(true, &inp);
        assert!(w.active);
        assert_eq!(w.position, Vec2::new(400.0, 300.0));
        w.flower_update(true, &inp);
        assert!(!w.active, "the second press closes");
        // noThrow refuses the open (`:170`).
        inp.no_throw = true;
        w.flower_update(true, &inp);
        assert!(!w.active);
    }

    #[test]
    fn the_dead_player_zeroes_the_line_and_closes_the_flower() {
        // `WiresUI.cs:124-128` / `:163-168`.
        let mut w = WiresRadial {
            line_opacity: 1.0,
            active: true,
            ..Default::default()
        };
        let inp = RadialInputs {
            mouse: Vec2::ZERO,
            screen: Vec2::new(1280.0, 720.0),
            held_kind: 3611,
            held_count: 1,
            dead: true,
            mouse_item: false,
            icon_enabled: false,
            chat_open: false,
            mouse_interface: false,
            options_or_fancy: false,
            right_press: false,
            left_press: false,
            no_throw: false,
            talking: false,
        };
        w.update(true, true, &inp);
        assert!(!w.active);
        assert_eq!(w.line_opacity, 0.0);
    }

    #[test]
    fn the_byte_scale_truncates_every_channel_including_alpha() {
        // XNA `Color * float` — the badge fade at 0.5 on (253,58,61,255):
        // 126, 29, 30, 127 (255*0.5 = 127.5 truncates).
        let c = scale_rgba([253, 58, 61, 255], 0.5);
        assert_eq!(c, [126.0 / 255.0, 29.0 / 255.0, 30.0 / 255.0, 127.0 / 255.0]);
    }
}
