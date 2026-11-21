//! W4-f11 seam 95-UI — `WiresUI` (`GameContent/UI/WiresUI.cs`): the
//! multi-wrench / Grand Design ToolMode radial's LOGIC half. The Settings
//! gates (`:59-90`), the `WiresRadial` state machine (`FlowerUpdate`
//! `:150-187`, `LineUpdate` `:114-148`) and the six-slot hit/toggle dispatch
//! (the `DrawFlower`/`DrawLine` click arms `:303-331`/`:475-501`) port here;
//! the DRAW passes (`DrawFlower` `:342-509`, `DrawLine` `:195-340`,
//! `DrawCursorArea` `:511-691`) are the contract-doc remainder — they need
//! the `TextureAssets.WireUi[12]` + `BuilderAcc` atlas loads, the gamepad
//! thumbstick paths and the centred-rotated sprite draws (the port's UI
//! sprite seam), ~450 C# lines of pure draw. See research/135.
//!
//! Deviations (documented, research/135): `PlayerInput.UsingGamepad` /
//! `LockGamepadTileUseButton` fold to false (no gamepad in the port — the
//! smart-cursor precedent); `player.mouseInterface`/`lastMouseInterface`
//! have no port fields — the caller passes the folded `mouse_interface`
//! snapshot (the `block_mouse` convention); `Main.SmartInteractShowingGenuine`
//! folds to false (the smart-interact UI is unported); `Main.HoveringOverAnNPC`
//! folds to the caller's `hovering_npc` (the `no_throw` arm's hover census is
//! unported, so it passes false today); `player.cursorItemIconEnabled/ID`
//! fold to the caller's pair (the cursor-item-icon draw is unported — the
//! gate keeps its shape).

use tr_core::smart_cursor::multi_tool_mode as mode;

/// `WiresUI.Settings.DrawToolModeUI` (`WiresUI.cs:59-73`): the radial shows
/// only while the Grand Design (3611) or the Multicolor Wrench (3625) is in
/// hand. `held_kind` is `HeldItem.type`; the C#'s `IsAir` gate folds into
/// the kind match (an air slot's type 0 matches neither).
pub(crate) fn draw_tool_mode_ui(held_kind: u16) -> bool {
    held_kind == 3611 || held_kind == 3625
}

/// `WiresUI.Settings.DrawToolAllowActuators` (`WiresUI.cs:75-90`): the
/// actuator slot shows only when the LAST held tool was the Grand Design.
/// The C# getter MUTATES the static `_lastActuatorEnabled` (2 for 3611,
/// 1 for 3625) and returns `== 2`; the port keeps the latch on
/// [`WiresRadial`] and this fn is the pure resolve.
pub(crate) fn draw_tool_allow_actuators(held_kind: u16, last_actuator_enabled: &mut i32) -> bool {
    if held_kind == 3611 {
        *last_actuator_enabled = 2;
    }
    if held_kind == 3625 {
        *last_actuator_enabled = 1;
    }
    *last_actuator_enabled == 2
}

/// The per-frame inputs the state machine reads off `Main`/the player (the
/// pool/UI cannot borrow the App — the `WireKiteInputs` snapshot pattern).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RadialInputs {
    /// `Main.mouseRight` — the right button held.
    pub mouse_right: bool,
    /// `Main.mouseRightRelease` — the release latch (the press tick is
    /// `mouse_right && mouse_right_release`, the `ui_right_release_latch`
    /// convention).
    pub mouse_right_release: bool,
    /// `Main.mouseLeft && Main.mouseLeftRelease` — the left PRESS tick.
    pub left_press: bool,
    /// `player.dead`.
    pub dead: bool,
    /// `Main.mouseItem.type > 0` — a stack rides the cursor.
    pub mouse_item: bool,
    /// `player.noThrow != 0` (`Player.cs:1456`).
    pub no_throw: bool,
    /// `player.talkNPC != -1` — the shop/talk panel is up.
    pub talking: bool,
    /// `Main.drawingPlayerChat` — the chat text field is open.
    pub chat_open: bool,
    /// `player.mouseInterface || player.lastMouseInterface` folded (no port
    /// fields — the caller passes its UI-hover suppression snapshot).
    pub mouse_interface: bool,
    /// `Main.ingameOptionsWindow || Main.InGameUI.IsVisible`.
    pub options_open: bool,
    /// `Main.HoveringOverAnNPC` (unported hover census — false today).
    pub hovering_npc: bool,
    /// `Main.SmartInteractShowingGenuine` (unported — false).
    pub smart_interact_genuine: bool,
    /// `player.cursorItemIconEnabled` (unported draw — false today).
    pub cursor_icon_enabled: bool,
    /// `player.cursorItemIconID` (0 when disabled).
    pub cursor_icon_id: u16,
}

/// `WiresUI.WiresRadial` (`WiresUI.cs:98-107`): the radial's state — the
/// flower's open flag + anchor, the line's opacity and the menu-hover
/// latch, plus the Settings' `_lastActuatorEnabled` static (`:25`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct WiresRadial {
    /// `position` (`:100`) — the flower's anchor (the mouse screen point
    /// at open; the gamepad centre-screen branch folds away).
    pub position: (f32, f32),
    /// `active` (`:102`) — the flower is open.
    pub active: bool,
    /// `OnWiresMenu` (`:104`) — the cursor hovered a slot last frame (the
    /// `FlowerUpdate` mouseInterface exemption).
    pub on_wires_menu: bool,
    /// `_lineOpacity` (`:106`) — the closed-flower line's fade.
    pub line_opacity: f32,
    /// `Settings._lastActuatorEnabled` (`:25`) — 2 after a Grand Design
    /// hold, 1 after a Multicolor Wrench hold, 0 at boot.
    pub last_actuator_enabled: i32,
}

impl Default for WiresRadial {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            active: false,
            on_wires_menu: false,
            line_opacity: 0.0,
            last_actuator_enabled: 0,
        }
    }
}

impl WiresRadial {
    /// `WiresRadial.Update` (`:108-112`): FlowerUpdate then LineUpdate.
    /// `tool_mode` is `Settings.ToolMode` (the App's `wire_tool_mode`);
    /// returns whether the flower drew a slot hover this frame (the
    /// `mouseInterface = true` write the caller folds into its suppression).
    pub(crate) fn update(
        &mut self,
        held_kind: u16,
        mouse: (f32, f32),
        screen: (f32, f32),
        input: &RadialInputs,
        tool_mode: &mut u8,
    ) -> bool {
        let allow_actuators = draw_tool_allow_actuators(held_kind, &mut self.last_actuator_enabled);
        self.flower_update(held_kind, mouse, input);
        self.line_update(held_kind, input);
        // The click dispatch lives in the C#'s DRAW passes (`DrawFlower`
        // when active, `DrawLine` when the line shows); the port runs it
        // here so the logic half is whole without the sprite seam. The
        // geometry is the draw passes' own (research/135 §2).
        if self.active {
            self.flower_clicks(mouse, input, tool_mode, allow_actuators)
        } else if self.line_opacity > 0.0 {
            self.line_clicks(mouse, screen, input, tool_mode, allow_actuators)
        } else {
            false
        }
    }

    /// `FlowerUpdate` (`:150-187`): the open/close latch. The right PRESS
    /// toggles the flower; a refusal condition closes it.
    fn flower_update(&mut self, held_kind: u16, mouse: (f32, f32), input: &RadialInputs) {
        if !draw_tool_mode_ui(held_kind) {
            self.active = false;
            return;
        }
        if input.mouse_interface && !self.on_wires_menu {
            self.active = false;
            return;
        }
        if input.dead || input.mouse_item {
            self.active = false;
            self.on_wires_menu = false;
            return;
        }
        self.on_wires_menu = false;
        // `:170-173` — the right press gate (`LockGamepadTileUseButton`
        // folds false).
        if !input.mouse_right
            || !input.mouse_right_release
            || input.no_throw
            || input.hovering_npc
            || input.talking
        {
            return;
        }
        if self.active {
            self.active = false;
        } else if !input.smart_interact_genuine {
            self.active = true;
            self.position = mouse;
            // The gamepad centre-screen anchor (`:182-185`) folds away.
        }
    }

    /// `LineUpdate` (`:114-148`): the closed-flower line's opacity ramp —
    /// ±0.05 per frame toward the clamped target, snapped when within one
    /// step (`Math.Sign` of the clamped delta).
    fn line_update(&mut self, held_kind: u16, input: &RadialInputs) {
        let mut value = true;
        let mut min = 0.75f32;
        if !draw_tool_mode_ui(held_kind) || input.chat_open {
            value = false;
            min = 0.0;
        }
        if input.dead || input.mouse_item {
            self.line_opacity = 0.0;
            return;
        }
        if input.cursor_icon_enabled && input.cursor_icon_id != 0 && input.cursor_icon_id != 3625 {
            self.line_opacity = 0.0;
            return;
        }
        // `:136-141` — the `!cursorItemIconEnabled && ((!UsingGamepad &&
        // !DrawToolAllowActuators) || mouseInterface || lastMouseInterface)`
        // disjunct: UsingGamepad folds false, so the first arm is
        // `!allow_actuators`; the options arm follows.
        let allow_actuators = self.last_actuator_enabled == 2;
        if (!input.cursor_icon_enabled && (!allow_actuators || input.mouse_interface))
            || input.options_open
        {
            self.line_opacity = 0.0;
            return;
        }
        let dir = if value { 1.0 } else { -1.0 };
        let num = (self.line_opacity + 0.05 * dir).clamp(min, 1.0);
        self.line_opacity += 0.05 * (num - self.line_opacity).signum();
        if (self.line_opacity - num).abs() < 0.05 {
            self.line_opacity = num;
        }
    }

    /// The `DrawFlower` slot geometry + hit test + toggle (`:363-508`):
    /// six slots on a 45-px ring (the cutter at the centre), the actuator
    /// slot only when allowed; the FIRST hit wins (`flag`). Returns the
    /// hover (the `mouseInterface`/`OnWiresMenu` writes).
    fn flower_clicks(
        &mut self,
        mouse: (f32, f32),
        input: &RadialInputs,
        tool_mode: &mut u8,
        allow_actuators: bool,
    ) -> bool {
        let (cx, cy) = self.position;
        let num3 = 4.0 + if allow_actuators { 1.0 } else { 0.0 };
        let num4 = if allow_actuators { 11.0 } else { -0.5 };
        let mouse_angle = (mouse.1 - cy).atan2(mouse.0 - cx);
        let mouse_dist = ((mouse.0 - cx).powi(2) + (mouse.1 - cy).powi(2)).sqrt();
        let mut hit_any = false;
        for i in 0..6i32 {
            if !allow_actuators && i == 5 {
                continue;
            }
            // `:374-401` — the slot centres: the ring slots rotate by
            // `num5 * 2π/num3 − π/num4` (the 0↔3 index swap), the cutter
            // (i 4) sits at the anchor, the actuator (i 5) at `(i−1)` steps.
            let (sx, sy) = if i == 4 {
                (cx, cy)
            } else {
                let num5 = match i {
                    0 => 3.0,
                    3 => 0.0,
                    5 => (i - 1) as f32,
                    _ => i as f32,
                };
                let a = num5 * std::f32::consts::TAU / num3 - std::f32::consts::PI / num4;
                (cx + a.cos() * 45.0, cy + a.sin() * 45.0)
            };
            // `:402-429` — the hit: the cutter is the <20-px centre disc;
            // the ring slots the 19-px mouse distance (the gamepad
            // angle-window branch folds away — `UsingGamepad` false).
            let _ = (mouse_angle, mouse_dist); // the gamepad window's inputs
            let hit = if i == 4 {
                mouse_dist < 20.0
            } else {
                ((sx - mouse.0).powi(2) + (sy - mouse.1).powi(2)).sqrt() < 19.0
            };
            if hit_any || !hit {
                continue;
            }
            hit_any = true;
            // `:475-501` — the left press toggles the slot's channel.
            if input.left_press {
                *tool_mode ^= slot_flag(i);
            }
            self.on_wires_menu = true;
        }
        // `:505-508` — a left press on NO slot closes the flower.
        if input.left_press && !hit_any {
            self.active = false;
        }
        hit_any
    }

    /// The `DrawLine` slot geometry + hit test + toggle (`:214-339`): the
    /// closed-flower line at the screen's bottom centre, the slots spread
    /// by the opacity; the actuator slot only when allowed. The hover
    /// radius scales with the opacity (`19 * _lineOpacity`).
    fn line_clicks(
        &mut self,
        mouse: (f32, f32),
        screen: (f32, f32),
        input: &RadialInputs,
        tool_mode: &mut u8,
        allow_actuators: bool,
    ) -> bool {
        // `:202` — the line's anchor: `(screenWidth/2, screenHeight−70)`
        // (the gamepad Vector2.Zero branch folds away).
        let (cx, cy) = (screen.0 / 2.0, screen.1 - 70.0);
        let mut hit_any = false;
        for i in 0..6i32 {
            if !allow_actuators && i == 5 {
                continue;
            }
            // `:225-252` — the slot centres (the 0↔3 swap, the ±(45+15)
            // spread, the cutter's vertical offset), scaled by the opacity.
            let num = match i {
                0 => 3,
                3 => 0,
                _ => i,
            };
            let spread = 45.0 + if allow_actuators { 15.0 } else { 0.0 };
            let (sx, sy) = match num {
                0 | 1 => (cx + spread * (2 - num) as f32 * self.line_opacity, cy),
                2 | 3 => (cx - spread * (num - 1) as f32 * self.line_opacity, cy),
                5 => (cx, cy + 22.0 * self.line_opacity),
                4 => (
                    cx,
                    cy - if allow_actuators { 22.0 } else { 0.0 } * self.line_opacity,
                ),
                _ => (cx, cy),
            };
            let hit = ((sx - mouse.0).powi(2) + (sy - mouse.1).powi(2)).sqrt()
                < 19.0 * self.line_opacity;
            if hit_any || !hit {
                continue;
            }
            hit_any = true;
            if input.left_press {
                *tool_mode ^= slot_flag(i);
            }
            self.on_wires_menu = true;
        }
        if input.left_press && !hit_any {
            self.active = false;
        }
        hit_any
    }
}

/// The slot→channel map (the `DrawFlower`/`DrawLine` toggle switches,
/// `:305-325`/`:477-497`): 0 Red, 1 Green, 2 Blue, 3 Yellow, 4 Cutter,
/// 5 Actuator. NOTE the C#'s slot order is Red/Green/Blue/Yellow — the
/// CURSOR-AREA draw (`:556-586`) re-maps 1↔2 for its colour dots, the
/// toggle itself uses this order.
fn slot_flag(i: i32) -> u8 {
    match i {
        0 => mode::RED,
        1 => mode::GREEN,
        2 => mode::BLUE,
        3 => mode::YELLOW,
        4 => mode::CUTTER,
        _ => mode::ACTUATOR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Settings.DrawToolModeUI` (`WiresUI.cs:59-73`): only 3611/3625.
    #[test]
    fn w4_f11_the_tool_mode_ui_gates_on_the_two_tools() {
        assert!(draw_tool_mode_ui(3611));
        assert!(draw_tool_mode_ui(3625));
        assert!(!draw_tool_mode_ui(3620)); // the Actuation Rod
        assert!(!draw_tool_mode_ui(0));
    }

    /// `Settings.DrawToolAllowActuators` (`:75-90`): the latch remembers
    /// the LAST tool — the Grand Design enables the actuator slot, the
    /// Multicolor Wrench disables it, and the state persists after the
    /// tool leaves the hand.
    #[test]
    fn w4_f11_the_actuator_slot_latches_the_last_tool() {
        let mut last = 0;
        assert!(!draw_tool_allow_actuators(3625, &mut last));
        assert!(draw_tool_allow_actuators(3611, &mut last));
        // A non-tool hold keeps the Grand Design's enable.
        assert!(draw_tool_allow_actuators(0, &mut last));
        assert!(!draw_tool_allow_actuators(3625, &mut last));
        assert!(!draw_tool_allow_actuators(0, &mut last));
    }

    /// `FlowerUpdate` (`:150-187`): the right press opens at the mouse,
    /// a second press closes; the dead/cursor-stack/talk gates refuse.
    #[test]
    fn w4_f11_the_right_press_toggles_the_flower() {
        let mut r = WiresRadial::default();
        let mut tm = mode::RED;
        let press = RadialInputs { mouse_right: true, mouse_right_release: true, ..Default::default() };
        r.update(3611, (100.0, 200.0), (800.0, 600.0), &press, &mut tm);
        assert!(r.active, "the press opened the flower");
        assert_eq!(r.position, (100.0, 200.0));
        r.update(3611, (120.0, 220.0), (800.0, 600.0), &press, &mut tm);
        assert!(!r.active, "the second press closed it");
        // The talk gate refuses the open.
        let talk = RadialInputs { talking: true, ..press };
        r.update(3611, (100.0, 200.0), (800.0, 600.0), &talk, &mut tm);
        assert!(!r.active);
        // A non-tool hold closes an open flower.
        r.update(3611, (100.0, 200.0), (800.0, 600.0), &press, &mut tm);
        assert!(r.active);
        r.update(530, (100.0, 200.0), (800.0, 600.0), &press, &mut tm);
        assert!(!r.active, "the wire item is not a tool-mode UI");
    }

    /// The `DrawFlower` toggle (`:475-497`): a left press on the cutter
    /// centre XORs the Cutter bit; a press on no slot closes the flower.
    /// The Grand Design's actuator slot rides the ring.
    #[test]
    fn w4_f11_the_flower_click_toggles_the_channel() {
        let mut r = WiresRadial::default();
        let press = RadialInputs { mouse_right: true, mouse_right_release: true, ..Default::default() };
        let mut tm = mode::RED;
        r.update(3611, (100.0, 200.0), (800.0, 600.0), &press, &mut tm);
        assert!(r.active);
        // The cutter sits AT the anchor — a left press there toggles it.
        let click = RadialInputs { left_press: true, ..Default::default() };
        let hovered = r.update(3611, (100.0, 200.0), (800.0, 600.0), &click, &mut tm);
        assert!(hovered, "the centre disc hit");
        assert_eq!(tm, mode::RED | mode::CUTTER);
        assert!(r.on_wires_menu);
        // A left press far from every slot closes the flower.
        let far = r.update(3611, (500.0, 500.0), (800.0, 600.0), &click, &mut tm);
        assert!(!far);
        assert!(!r.active, "the empty-space press closed it");
    }

    /// `LineUpdate` (`:114-148`): the opacity ramps 0.05/frame toward 1
    /// while a tool is held, snaps at the target, and zeroes on the
    /// cursor-stack gate. The Multicolor Wrench (no actuator latch) keeps
    /// the line only when the cursor-icon gate passes — with the icon
    /// disabled and no actuator allowance the line zeroes (`:136`).
    #[test]
    fn w4_f11_the_line_opacity_ramps_and_gates() {
        let mut r = WiresRadial::default();
        let idle = RadialInputs::default();
        let mut tm = mode::RED;
        // The Grand Design hold: the actuator latch enables the line.
        r.update(3611, (0.0, 0.0), (800.0, 600.0), &idle, &mut tm);
        assert!((r.line_opacity - 0.05).abs() < 1e-6, "one 0.05 step");
        for _ in 0..30 {
            r.update(3611, (0.0, 0.0), (800.0, 600.0), &idle, &mut tm);
        }
        assert_eq!(r.line_opacity, 1.0, "the ramp capped at 1");
        // The cursor stack zeroes it instantly.
        let stack = RadialInputs { mouse_item: true, ..idle };
        r.update(3611, (0.0, 0.0), (800.0, 600.0), &stack, &mut tm);
        assert_eq!(r.line_opacity, 0.0);
        // The Multicolor Wrench with the icon disabled: the !allow_actuators
        // arm zeroes the line (the C# `:136` disjunct).
        let mut w = WiresRadial::default();
        w.update(3625, (0.0, 0.0), (800.0, 600.0), &idle, &mut tm);
        assert_eq!(w.line_opacity, 0.0, "no actuator allowance, no icon");
        // The wrench's OWN cursor icon (3625) exempts the disjunct.
        let icon = RadialInputs { cursor_icon_enabled: true, cursor_icon_id: 3625, ..idle };
        w.update(3625, (0.0, 0.0), (800.0, 600.0), &icon, &mut tm);
        assert!((w.line_opacity - 0.05).abs() < 1e-6, "the 3625 icon ramps");
    }
}
