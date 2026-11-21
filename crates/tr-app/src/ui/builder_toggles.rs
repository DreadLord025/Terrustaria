//! The builder-accessory toggle column (S1-W4-g81 residuals) —
//! `Main.DrawBuilderAccToggles` (`Main.cs:47353-47590`), called by
//! `DrawInterface_31_BuilderAccToggles` (`:45477-45480`) at `(10, 77)`.
//! The wire-channel rows (4/5/6/7 red/blue/green/yellow + 9 actuator)
//! cycle `builderAccStatus` 0→1→2→0 (Bright/Normal/Faded — the 3=Hidden
//! arm is unreachable through the click, `:47551-47555`); row 8 toggles
//! the lens' wire-mode force; rows 0/1/2/3 the ruler-line/grid,
//! auto-actuator and auto-paint on/off pairs.
//!
//! The block-replacer (`DrawBlockReplacementIcon`, `:42518-42556`) and
//! torch-swap (`DrawTorchBiomeSwapIcon`, `:42558-42596`) side icons ride
//! the same column tail (`:47575-47587`).
//!
//! Deviations (ANALYSIS "The builder-toggle column (S1-W4-g81)"):
//! * `Main.hideUI` has no port field — the caller's `no_hud` gate stands
//!   in (the frame already skips the whole HUD branch);
//! * `PlayerInput.IgnoreMouseInterface` is false (no gamepad);
//! * `UILinkPointNavigator.SetPosition` / `DoStatefulTickSound` /
//!   `Shortcuts.BUILDERACCCOUNT` are gamepad/audit seams — not ported;
//! * the hover `MouseTextNoOverride` rides the port's `draw_mouse_text`
//!   (the tooltip seam the buff bar uses).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::player::Player;
use crate::ui::{color, tooltip, uistate, UiPulse};

/// The column's per-frame inputs.
pub struct ToggleInputs {
    pub mouse: Vec2,
    pub screen: Vec2,
    /// `Main.mouseLeft && Main.mouseLeftRelease` — the press-after-release
    /// conjunction (the buff-bar gate's form).
    pub left_press: bool,
    /// `Main.playerInventory` (`:47355`).
    pub inventory_open: bool,
    pub pulse: UiPulse,
}

/// `Main.GetBuilderAccsCountToShow` (`Main.cs:47592-47597`):
/// `blockReplaceIcons = 1` always, `torchGodIcons = unlockedBiomeTorches`,
/// `totalDrawnIcons = InfoAccMechShowWires*6 + rulerLine + rulerGrid +
/// autoActuator + autoPaint + blockReplaceIcons + torchGodIcons`.
pub fn counts_to_show(p: &Player) -> (i32, i32, i32) {
    let block_replace = 1;
    let torch_god = p.unlocked_biome_torches as i32;
    let total = p.info_acc_mech_show_wires as i32 * 6
        + p.ruler_line as i32
        + p.ruler_grid as i32
        + p.auto_actuator as i32
        + p.auto_paint as i32
        + block_replace
        + torch_god;
    (block_replace, torch_god, total)
}

/// The row's `builderAccStatus` index (`Main.cs:47374-47383`): i0→8,
/// i1→9, else i-2 (i2→0 ruler-line, i3→1 ruler-grid, i4→2 auto-actuator,
/// i5→3 auto-paint, i6→4 red, i7→5 blue, i8→6 green, i9→7 yellow).
pub fn row_status_index(i: usize) -> usize {
    match i {
        0 => 8,
        1 => 9,
        _ => i - 2,
    }
}

/// Whether the row draws at all (`Main.cs:47397-47505`'s `continue`
/// gates): row 8 + the wire rows need `InfoAccMechShowWires`, rows 0/1/2/3
/// need their accessory grant.
pub fn row_visible(p: &Player, num3: usize) -> bool {
    match num3 {
        8 | 4 | 5 | 6 | 7 | 9 => p.info_acc_mech_show_wires,
        0 => p.ruler_line,
        1 => p.ruler_grid,
        2 => p.auto_actuator,
        3 => p.auto_paint,
        _ => false,
    }
}

/// The row's tint (`Main.cs:47403`/`:47507`): the on/off rows are White
/// vs (127,127,127); the wire rows add the Faded `MultiplyRGBA(0.66)` and
/// the unreachable Hidden `MultiplyRGBA(0.33)` arms. XNA's MultiplyRGBA
/// is `(byte)(a * b / 255)` per channel.
pub fn row_color(num3: usize, status: i32) -> [u8; 4] {
    let white = [255u8, 255, 255, 255];
    let grey = [127u8, 127, 127, 255];
    if matches!(num3, 4 | 5 | 6 | 7 | 9) {
        match status {
            0 => white,
            1 => grey,
            2 => multiply_rgba(grey, [168, 168, 168, 168]), // 0.66*255 = 168.3 → 168
            _ => multiply_rgba(grey, [84, 84, 84, 84]),     // 0.33*255 = 84.15 → 84
        }
    } else if status == 0 {
        white
    } else {
        grey
    }
}

/// `Color.MultiplyRGBA` — `(byte)(a * b / 255)` per channel (XNA's
/// integer form; the C# passes the float colour `new Color(0.66f,...)`
/// whose bytes are `(int)(0.66*255) = 168`).
fn multiply_rgba(a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    [
        (a[0] as u32 * b[0] as u32 / 255) as u8,
        (a[1] as u32 * b[1] as u32 / 255) as u8,
        (a[2] as u32 * b[2] as u32 / 255) as u8,
        (a[3] as u32 * b[3] as u32 / 255) as u8,
    ]
}

/// The hover text's channel name (`Main.cs:47512-47528`).
fn channel_name(num3: usize) -> &'static str {
    match num3 {
        4 => "Game.RedWires",
        5 => "Game.BlueWires",
        6 => "Game.GreenWires",
        7 => "Game.YellowWires",
        _ => "Game.Actuators",
    }
}

/// The hover text's mode name (`Main.cs:47531-47544`).
fn mode_name(status: i32) -> &'static str {
    match status {
        0 => "GameUI.Bright",
        1 => "GameUI.Normal",
        2 => "GameUI.Faded",
        _ => "GameUI.Hidden",
    }
}

/// The on/off rows' hover key pairs (`Main.cs:47407`/`:47427`/`:47447`/
/// `:47467`/`:47487`).
fn on_off_keys(num3: usize) -> (&'static str, &'static str) {
    match num3 {
        8 => ("GameUI.WireModeForced", "GameUI.WireModeNormal"),
        0 => ("GameUI.RulerOn", "GameUI.RulerOff"),
        1 => ("GameUI.MechanicalRulerOn", "GameUI.MechanicalRulerOff"),
        3 => ("GameUI.PaintSprayerOn", "GameUI.PaintSprayerOff"),
        _ => ("GameUI.ActuationDeviceOn", "GameUI.ActuationDeviceOff"),
    }
}

/// `Main.DrawBuilderAccToggles` (`Main.cs:47353-47590`). Returns the
/// hovered row's index (the `state` the C# feeds `DoStatefulTickSound` —
/// the port plays the Menu_Tick cue on the click instead, the sound-12
/// the C# plays at `:47413` etc.). The click writes ride `p.builder_acc_status`
/// in place; the caller plays the cue when the return is `Some(true)`.
pub fn draw_builder_acc_toggles(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &mut Player,
    inp: &ToggleInputs,
) -> Option<bool> {
    if !inp.inventory_open {
        return None;
    }
    // `:47361-47364` — the sign gate (the port's sign field stays -1).
    if p.sign >= 0 {
        return None;
    }
    let Some(sheet) = lookups.builder_acc else {
        return None;
    };
    let (_, torch_god, total) = counts_to_show(p);
    let mut start = Vec2::new(10.0, 77.0);
    start.y += 24.0 * torch_god as f32;
    let push_up = total >= 10;
    let mut num = 0i32;
    let mut clicked = false;
    let mut hover_text: Option<String> = None;
    for i in 0..10usize {
        let num3 = row_status_index(i);
        if !row_visible(p, num3) {
            continue;
        }
        let mut pos = start + Vec2::new(0.0, num as f32 * 24.0);
        if push_up {
            pos.y -= 24.0;
        }
        // `Utils.CenteredRectangle(pos, (14,14)).Contains(mouse)` —
        // the int-truncated rect (`Utils.cs:947-950`).
        let rx = (pos.x - 7.0) as i32;
        let ry = (pos.y - 7.0) as i32;
        let mx = inp.mouse.x as i32;
        let my = inp.mouse.y as i32;
        let hover = mx >= rx && mx < rx + 14 && my >= ry && my < ry + 14;
        let status = p.builder_acc_status[num3];
        let c = row_color(num3, status);
        if hover {
            let text = if matches!(num3, 4 | 5 | 6 | 7 | 9) {
                format!(
                    "{}: {}",
                    crate::lang::text(channel_name(num3)),
                    crate::lang::text(mode_name(status))
                )
            } else {
                let (on, off) = on_off_keys(num3);
                crate::lang::text(if status == 0 { on } else { off })
            };
            hover_text = Some(text);
            if inp.left_press {
                // `:47549-47557` — the wire rows cycle 0→1→2→0; the
                // on/off rows flip (`:47412` etc.).
                if matches!(num3, 4 | 5 | 6 | 7 | 9) {
                    p.builder_acc_status[num3] += 1;
                    if p.builder_acc_status[num3] >= 3 {
                        p.builder_acc_status[num3] = 0;
                    }
                } else {
                    p.builder_acc_status[num3] = if status == 0 { 1 } else { 0 };
                }
                clicked = true;
            }
        }
        // `:47563` — the 14x14 icon at `(num3*16, 16)`, origin Size/2.
        push_builder_frame(r, lookups, sheet, pos, (num3 as i32 * 16, 16, 14, 14), color::tint(c));
        if hover {
            // `:47566` — the InfoIcon[13] hover ring in OurFavoriteColor.
            if let Some(ring) = lookups.info_icon_13 {
                if let Some(uv) = lookups.hud_uv(Some(&ring)) {
                    let w = ring.width as f32;
                    let h = ring.height as f32;
                    r.push_ui_sprite(
                        [pos.x - w * 0.5, pos.y - h * 0.5],
                        [w, h],
                        uv,
                        color::tint(uistate::OUR_FAVORITE_COLOR),
                    );
                }
            }
        }
        num += 1;
    }
    // `:47575-47587` — the block-replacer + torch-swap side icons.
    draw_block_replacement_icon(r, lookups, p, inp, push_up);
    if p.unlocked_biome_torches {
        draw_torch_biome_swap_icon(r, lookups, p, inp, push_up);
    }
    if let Some(t) = hover_text {
        tooltip::draw_mouse_text(r, lookups, inp.mouse, &t, inp.screen, inp.pulse);
    }
    if clicked { Some(true) } else { None }
}

/// `Main.DrawBlockReplacementIcon` (`Main.cs:42518-42556`): the 3-frame
/// `BlockReplace_0` sheet at 0.9 scale, the hover ring is frame 2 in
/// OurFavoriteColor, the click flips `builderAccStatus[10]` (sound 22 —
/// the port's Menu_Tick stand-in is the caller's cue; the C#'s 22 is the
/// distinct `Tick` variant, recorded).
fn draw_block_replacement_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &mut Player,
    inp: &ToggleInputs,
    push_up: bool,
) {
    let Some(sheet) = lookups.block_replace_0 else {
        return;
    };
    let y = if push_up { 23.0 } else { 44.0 };
    let scale = 0.9f32;
    let on = p.builder_acc_status[10] == 0;
    let fw = sheet.width / 3;
    let fh = sheet.height;
    let pos = Vec2::new(0.0, y);
    let hover = inp.mouse.x > 0.0
        && inp.mouse.x < fw as f32 * scale
        && inp.mouse.y > y
        && inp.mouse.y < y + fh as f32 * scale;
    if hover && inp.left_press {
        p.builder_acc_status[10] = if on { 1 } else { 0 };
    }
    let frame = if on { 0 } else { 1 };
    push_sheet_frame(r, lookups, sheet, pos, 3, frame, scale, [1.0; 4]);
    if hover {
        push_sheet_frame(
            r,
            lookups,
            sheet,
            pos,
            3,
            2,
            scale,
            color::tint(uistate::OUR_FAVORITE_COLOR),
        );
        let t = crate::lang::game_ui(if on { "BlockReplacerOn" } else { "BlockReplacerOff" });
        tooltip::draw_mouse_text(r, lookups, inp.mouse, &t, inp.screen, inp.pulse);
    }
}

/// `Main.DrawTorchBiomeSwapIcon` (`Main.cs:42558-42596`): the 4-frame
/// `Extra_211` sheet at 0.9 scale, the click flips `builderAccStatus[11]`.
fn draw_torch_biome_swap_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &mut Player,
    inp: &ToggleInputs,
    push_up: bool,
) {
    let Some(sheet) = lookups.extra_211 else {
        return;
    };
    let y = if push_up { 47.0 } else { 68.0 };
    let scale = 0.9f32;
    let on = p.builder_acc_status[11] == 0;
    let fw = sheet.width / 4;
    let fh = sheet.height;
    let pos = Vec2::new(0.0, y);
    let hover = inp.mouse.x > 0.0
        && inp.mouse.x < fw as f32 * scale
        && inp.mouse.y > y
        && inp.mouse.y < y + fh as f32 * scale;
    if hover && inp.left_press {
        p.builder_acc_status[11] = if on { 1 } else { 0 };
    }
    let frame = if on { 1 } else { 0 };
    push_sheet_frame(r, lookups, sheet, pos, 4, frame, scale, [1.0; 4]);
    if hover {
        push_sheet_frame(
            r,
            lookups,
            sheet,
            pos,
            4,
            if on { 3 } else { 2 },
            scale,
            color::tint(uistate::OUR_FAVORITE_COLOR),
        );
        let t = crate::lang::game_ui(if on { "TorchTypeSwapperOn" } else { "TorchTypeSwapperOff" });
        tooltip::draw_mouse_text(r, lookups, inp.mouse, &t, inp.screen, inp.pulse);
    }
}

/// One BuilderIcons source-rect quad centred on `pos` (the column's
/// `origin = rectangle.Size()/2`, scale 1 — `Main.cs:47563`).
fn push_builder_frame(
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

/// One horizontal-frame quad of a side-icon sheet at `scale`, top-left
/// anchored (the side icons draw `origin = default` — `Main.cs:42549`).
fn push_sheet_frame(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: tr_render::SheetRect,
    pos: Vec2,
    cols: u32,
    frame: u32,
    scale: f32,
    tint: [f32; 4],
) {
    let fw = sheet.width / cols.max(1);
    let fh = sheet.height;
    let aw = lookups.atlas_size.0 as f32;
    let ah = lookups.atlas_size.1 as f32;
    let x0 = sheet.x as f32 + (frame * fw) as f32;
    let y0 = sheet.y as f32;
    let uv = [x0 / aw, y0 / ah, (x0 + fw as f32) / aw, (y0 + fh as f32) / ah];
    r.push_ui_sprite(
        [pos.x, pos.y],
        [fw as f32 * scale, fh as f32 * scale],
        uv,
        tint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> Player {
        let mut p = Player::spawn(&crate::test_world_helpers::ground());
        p.info_acc_mech_show_wires = true;
        p
    }

    #[test]
    fn the_row_index_map_matches_the_csharp() {
        // `Main.cs:47374-47383`.
        assert_eq!(row_status_index(0), 8);
        assert_eq!(row_status_index(1), 9);
        assert_eq!(row_status_index(2), 0);
        assert_eq!(row_status_index(6), 4);
        assert_eq!(row_status_index(9), 7);
    }

    #[test]
    fn the_counts_fold_the_grants() {
        // `Main.cs:47592-47597`: lens 6 + blockReplace 1 = 7 base.
        let mut p = player();
        let (br, tg, total) = counts_to_show(&p);
        assert_eq!((br, tg, total), (1, 0, 7));
        p.ruler_line = true;
        p.unlocked_biome_torches = true;
        let (_, tg, total) = counts_to_show(&p);
        assert_eq!((tg, total), (1, 9));
    }

    #[test]
    fn the_wire_rows_cycle_at_three() {
        // `Main.cs:47551-47555`: 0→1→2→0 — the Hidden arm unreachable.
        let mut status = 0;
        for want in [1, 2, 0] {
            status += 1;
            if status >= 3 {
                status = 0;
            }
            assert_eq!(status, want);
        }
    }

    #[test]
    fn the_faded_tint_is_the_multiply_rgba() {
        // `Main.cs:47507`: grey (127) × 0.66 → (127*168/255) = 83.6 → 83.
        let c = row_color(4, 2);
        assert_eq!(c, [83, 83, 83, 83]);
        // The Normal grey keeps alpha 255 (the C# `new Color(127,127,127)`).
        assert_eq!(row_color(4, 1), [127, 127, 127, 255]);
        assert_eq!(row_color(4, 0), [255, 255, 255, 255]);
        // The on/off rows never fade.
        assert_eq!(row_color(8, 1), [127, 127, 127, 255]);
    }

    #[test]
    fn the_visibility_gates_match_the_grants() {
        let mut p = player();
        assert!(row_visible(&p, 8));
        assert!(row_visible(&p, 4));
        assert!(!row_visible(&p, 0), "no ruler grant");
        p.ruler_line = true;
        assert!(row_visible(&p, 0));
        p.info_acc_mech_show_wires = false;
        assert!(!row_visible(&p, 8));
        assert!(!row_visible(&p, 9));
    }
}
