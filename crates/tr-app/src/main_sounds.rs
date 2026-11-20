//! W4-g22 — the Main.cs `PlaySound` census, batch 2 of 3 (code-only).
//!
//! The lane split is line-order thirds over the **271** `SoundEngine.PlaySound`
//! call sites of `decompiled/Terraria/Main.cs` (recursive `Select-String
//! "SoundEngine\.PlaySound"`): g21 owns sites 1-90 (`:3244`-`:43936`), **g22
//! owns sites 91-180 (`:44118`-`:49727`)**, g23 owns sites 181-271
//! (`:49734`-`:66248`). At probe time neither co-lane had pushed a claim
//! (`git ls-remote --heads origin "*sounds-main*"` empty; both worktrees clean
//! at the wave-3 base `a725acb0`), so the thirds are computed from the raw
//! line-ordered site list, not from a shared ledger.
//!
//! This module ports the *pure* halves of the g22 window that have no live
//! Rust seam yet — the hair-window scroll machine (`Main.DrawHairWindow`,
//! `Main.cs:43709-44138`), the clothes-window open/cancel/save state machine
//! (`Main.cs:44140-44200`) and the builder-accessory toggle cycles
//! (`Main.DrawBuilderAccToggles`, `Main.cs:47353-47574`) — plus the case-20
//! female-hit slot pick (`LegacySoundPlayer.cs:911-923`). The cue plays ride
//! the existing audio seams (`Menu::pending_cue` drained by `app/loop.rs`,
//! `Audio::play`); the in-game windows themselves stay unported (the dresser
//! UI is the documented `chest.rs` out-of-scope, the stylist hair window has
//! no screen), so their machines land unwired behind witnesses, the
//! `npc_ai_style59` precedent.
#![allow(dead_code)] // wiring deferred - the in-game windows land with their UI waves
//!
//! RNG: every g22 site is a draw-free legacy id (10/11/12 menu, 7 grab,
//! 20 female hit, 1 player hit) EXCEPT the case-7 grab pitch
//! (`LegacySoundPlayer.cs:725`, one `Main.rand.Next(-10, 11) * 0.01`) and the
//! case-20 `Next(3)` slot pick (`:913`). The menu ids 10/11 draw nothing
//! (`:750-769`); id 12 draws nothing and gates on
//! `FocusHelper.AllowInputProcessing` (`:770-782`) — the port always claims
//! focus (the standing D12 deviation), so the gate is constants-on. No
//! gameplay stream is touched by this batch: the two drawing cases keep
//! their rolls on the caller's stream behind the audibility gate, and the
//! unwired machines draw nothing at all.

/// `Main.DrawHairWindow`'s scroll state (`Main.cs:43726-43785`): the wheel
/// steps `hairStart` by 5 per 120-delta notch (one sound 12 per notch,
/// `:43735`/`:43740`), the left/right buttons step by 15 on the press edge
/// (`:43758-43759`/`:43772-43773`, one sound 12 each), and the tail clamps
/// `hairStart` into `[0, count - 15]` (`:43778-43785`). Returns
/// `(new_start, tick_count)` — the C# plays the cue inside the loop/branch,
/// so the count is the number of `PlaySound(12)` sites hit, in order.
///
/// `wheel_delta` is `PlayerInput.ScrollWheelDelta` (`:43728` divides by 120
/// and negates); the button edges are the `mouseLeftRelease && mouseLeft`
/// reads of `:43756`/`:43770`, already gated by their hover rects and the
/// `hairStart >= 1` / `hairStart + 15 < count` visibility arms (`:43751`,
/// `:43764`) — the caller passes `false` when the button is not drawn.
pub fn hair_window_scroll(
    hair_start: i32,
    count: i32,
    wheel_delta: i32,
    wheel_over_panel: bool,
    left_button_edge: bool,
    right_button_edge: bool,
) -> (i32, u32) {
    let mut start = hair_start;
    let mut ticks = 0u32;
    // `:43726-43744`: the wheel only acts while the pointer is inside the
    // HairStyleBack rect; `num7 = -(delta / 120)`, one 5-step + one cue per
    // unit, sign-strided (`num8 = Math.Sign(num7)`).
    if wheel_over_panel {
        let mut num7 = -(wheel_delta / 120);
        let num8 = num7.signum();
        while num7 != 0 {
            if num7 < 0 {
                start -= 5; // `:43734`
            } else {
                start += 5; // `:43739`
            }
            ticks += 1; // `PlaySound(12)` `:43735`/`:43740`
            num7 -= num8; // `:43742`
        }
    }
    // `:43751-43761`: the left button draws only at `hairStart >= 1`; its
    // press edge steps -15 and plays 12. The visibility gate is the caller's
    // (the edge arrives false when the button is absent).
    if left_button_edge {
        start -= 15; // `:43758`
        ticks += 1; // `:43759`
    }
    // `:43764-43775`: the right button draws only at `hairStart + 15 < count`.
    if right_button_edge {
        start += 15; // `:43772`
        ticks += 1; // `:43773`
    }
    // `:43778-43785`: the clamp tail — upper first, then lower, no cues.
    if start + 15 >= count {
        start = count - 15;
    }
    if start < 0 {
        start = 0;
    }
    (start, ticks)
}

/// `Main.DrawHairWindow`'s hair-cell pick (`Main.cs:44110-44119`): the hover
/// rect test plus the `mouseLeft && mouseLeftRelease` edge. Returns `true`
/// when the pick fires — the caller writes `player.hair = style`, consumes
/// the release (`:44116`) and plays sound 12 (`:44118`). The rect is the
/// 52x52 `InventoryBack` cell at `(x, y)`; the C# compares are strict on all
/// four edges (`:44110`).
pub fn hair_cell_pick(
    mouse: [f32; 2],
    cell: [f32; 2],
    cell_w: f32,
    cell_h: f32,
    left_down: bool,
    left_release: bool,
) -> bool {
    mouse[0] > cell[0]
        && mouse[0] < cell[0] + cell_w
        && mouse[1] > cell[1]
        && mouse[1] < cell[1] + cell_h
        && left_down
        && left_release
}

/// The clothes-window state machine's sound decisions
/// (`Main.OpenClothesWindow`/`CancelClothesWindow`/`SaveClothesWindow`,
/// `Main.cs:44140-44200`). The window itself is unported (the dresser UI is
/// `chest.rs`'s documented out-of-scope); these are the pure cue arms the
/// wiring wave will call.
///
/// `open_clothes_window` (`:44140-44167`): an already-open window routes to
/// `CancelClothesWindow()` (`:44144`, the loud cancel — sound 11) and
/// returns; otherwise the open plays sound 10 (`:44152`). Returns the cue
/// id, or `None` never (both arms play).
pub fn open_clothes_window_cue(already_open: bool) -> u32 {
    if already_open {
        11 // `:44144` -> `CancelClothesWindow(quiet: false)` -> `:44176`
    } else {
        10 // `:44152`
    }
}

/// `Main.CancelClothesWindow` (`:44169-44192`): the close plays sound 11
/// (`:44176`) only when the window was open AND `!quiet` (`:44171-44177`);
/// the colour restore runs regardless of `quiet` but only while open.
pub fn cancel_clothes_window_cue(open: bool, quiet: bool) -> Option<u32> {
    if open && !quiet {
        Some(11)
    } else {
        None
    }
}

/// `Main.SaveClothesWindow` (`:44194-44200`): the save plays sound 7 (the
/// Grab cue, `:44197`) unconditionally — no open gate, the C# has none.
pub fn save_clothes_window_cue() -> u32 {
    7
}

/// `Main.DrawClothesWindow`'s two click arms — the colour-slot cell pick
/// (`Main.cs:44537-44545`, sound 12 at `:44541`) and the player-dummy cycle
/// (`:44614-44619`, sound 12 at `:44617`): both fire iff the rect hit holds
/// AND `mouseLeft && mouseLeftRelease`, then consume the release. The rect
/// tests differ — the slot cell is four STRICT compares (`:44537`), the
/// dummy is `Rectangle.Contains` (`:44262`, inclusive left/top) — so the
/// caller computes `over` with the right shape; the cue decision is shared.
pub fn clothes_window_click(over: bool, left_down: bool, left_release: bool) -> bool {
    over && left_down && left_release
}

/// `Main.DrawBuilderAccToggles`' per-slot click cycles (`Main.cs:
/// 47395-47562`). `slot` is the C# `num3` (the `i - 2` remap with 0->8,
/// 1->9, `:47374-47383`); `status` is `builderAccStatus[num3]`. Returns the
/// new status when the click toggles (each toggle plays sound 12 and
/// consumes the release), `None` when the slot has no click arm.
///
/// The binary slots 8 (wire mode, `:47410-47415`), 0 (ruler line,
/// `:47430-47435`), 1 (ruler grid, `:47450-47455`), 3 (paint sprayer,
/// `:47470-47475`) and 2 (actuation device, `:47490-47495`) flip 0<->1.
/// The wire-brightness slots 4/5/6/7/9 (`:47549-47558`) increment and wrap
/// at **3** (`>= 3 -> 0`, the C# quirk — the tooltip's "Hidden" state 3 is
/// unreachable through the cycle). Slot 11 `continue`s (`:47560-47561`).
/// The visibility gates (`InfoAccMechShowWires`, `rulerLine`, `rulerGrid`,
/// `autoPaint`, `autoActuator`) are the caller's — a hidden slot never
/// reaches the click.
pub fn builder_acc_toggle(slot: i32, status: i32) -> Option<i32> {
    match slot {
        8 | 0 | 1 | 3 | 2 => Some(if status == 0 { 1 } else { 0 }),
        4 | 5 | 6 | 7 | 9 => {
            let next = status + 1; // `:47551`
            Some(if next >= 3 { 0 } else { next }) // `:47552-47555`
        }
        _ => None, // `:47560` case 11 + the default fall-through
    }
}

/// `Main.DrawInfoAccs`' hideInfo toggle (`Main.cs:47199-47204`): the press
/// edge flips `hideInfo[num23]` and plays sound 12 (`:47201`), consuming the
/// release (`:47202`). `num23` is the icon index with the C#'s 8->7 remap
/// (`:47183-47186`). Pure flip — the cue is the constant 12.
pub fn info_acc_hide_toggle(hidden: bool) -> bool {
    !hidden // `:47203`
}

/// One `LegacySoundPlayer.PlaySound` case-20 (female hurt) play's slot pick
/// (`LegacySoundPlayer.cs:911-923`): `Main.rand.Next(3)` (`:913`) selects
/// the `SoundFemaleHit[0..2]` slot — NO pitch draw (the case sets
/// Volume/Pan only, `:919-920`). The C# reaches the switch only after the
/// attenuation walk passed (`:390-436`), so the caller must gate on
/// audibility BEFORE drawing. The `Main.cs:49045` site is the menuMode 2
/// gender toggle's male->female arm (`:49043-49047`); the female->male arm
/// plays case 1 (`:49050`, `sound::player_hit_case1`).
pub fn female_hit_case20(rng: &mut dyn FnMut(i32, i32) -> i32) -> i32 {
    rng(0, 3)
}

/// `Main.DrawSettingButton`'s hover-enter cue (`Main.cs:45595-45616`): the
/// pointer entering the label rect plays ONE sound 12 (`:45601`) on the
/// `!mouseOver` edge, then latches `mouseOver = true`; leaving the rect
/// clears the latch with NO cue (`:45613-45616`). The `IgnoreMouseInterface`
/// gate (`:45597`) and the `!LocalPlayer.mouseInterface` rect gate
/// (`:45595`) are the caller's — `allowed` arrives false when either holds.
/// Returns `Some("Menu_Tick")` on the enter edge only.
pub fn settings_button_tick(over: bool, was_over: bool, allowed: bool) -> Option<&'static str> {
    if over && allowed && !was_over {
        Some("Menu_Tick") // `:45601`
    } else {
        None
    }
}

/// `Main.DrawClothesWindow`'s Change/Cancel text-button hover latch
/// (`Main.cs:44279-44297` / `:44337-44355`): entering the hovered rect plays
/// sound 12 once (`:44285` / `:44343`, gated on `!npcChatFocusN`), leaving
/// plays sound 12 once (`:44294` / `:44352`, gated on the latch being set).
/// Both edges cue; the steady states are silent. Returns the number of
/// `PlaySound(12)` sites hit (0 or 1). The Change arm's hover additionally
/// requires the dirty flag (`flag2`, `:44279`) and writes
/// `releaseUseItem = false` (`:44288`) — the caller's.
pub fn clothes_window_text_hover(hovered: bool, latched: bool) -> u32 {
    if hovered {
        if !latched {
            1 // `:44285` / `:44343`
        } else {
            0
        }
    } else if latched {
        1 // `:44294` / `:44352`
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- hair window scroll (Main.cs:43726-43785) -------------------------

    #[test]
    fn g22_the_wheel_steps_five_per_notch_with_one_tick_each() {
        // delta -240 -> num7 = 2 -> two +5 steps, two cues (:43739-43740).
        let (start, ticks) = hair_window_scroll(10, 100, -240, true, false, false);
        assert_eq!((start, ticks), (20, 2));
        // delta +120 -> num7 = -1 -> one -5 step, one cue (:43734-43735).
        let (start, ticks) = hair_window_scroll(10, 100, 120, true, false, false);
        assert_eq!((start, ticks), (5, 1));
    }

    #[test]
    fn g22_the_wheel_is_dead_outside_the_panel_rect() {
        // :43726's rect gate — no steps, no cues while the pointer is out.
        let (start, ticks) = hair_window_scroll(10, 100, -360, false, false, false);
        assert_eq!((start, ticks), (10, 0));
    }

    #[test]
    fn g22_the_buttons_step_fifteen_and_the_tail_clamps() {
        // Left edge at start 20: -15 -> 5 (:43758), one cue.
        assert_eq!(hair_window_scroll(20, 100, 0, false, true, false), (5, 1));
        // Right edge at start 80 with count 100: +15 -> 95, then the
        // 95 + 15 >= 100 clamp pins count - 15 = 85 (:43778-43781).
        assert_eq!(hair_window_scroll(80, 100, 0, false, false, true), (85, 1));
        // The lower clamp: start 5, left edge -> -10 -> 0 (:43782-43785).
        assert_eq!(hair_window_scroll(5, 100, 0, false, true, false), (0, 1));
    }

    #[test]
    fn g22_the_hair_cell_pick_needs_the_strict_rect_and_the_edge() {
        // :44110's four strict compares + :44114's mouseLeft && release.
        assert!(hair_cell_pick([30.0, 30.0], [20.0, 20.0], 52.0, 52.0, true, true));
        // On the edge is OUT (strict >).
        assert!(!hair_cell_pick([20.0, 30.0], [20.0, 20.0], 52.0, 52.0, true, true));
        // No release, no pick.
        assert!(!hair_cell_pick([30.0, 30.0], [20.0, 20.0], 52.0, 52.0, true, false));
    }

    // ---- clothes window (Main.cs:44140-44200) -----------------------------

    #[test]
    fn g22_the_clothes_open_plays_ten_and_the_reopen_cancels_loud() {
        assert_eq!(open_clothes_window_cue(false), 10); // :44152
        // :44144 routes an open window through CancelClothesWindow(false).
        assert_eq!(open_clothes_window_cue(true), 11);
    }

    #[test]
    fn g22_the_clothes_cancel_is_quiet_gated_and_the_save_grabs() {
        assert_eq!(cancel_clothes_window_cue(true, false), Some(11)); // :44176
        assert_eq!(cancel_clothes_window_cue(true, true), None); // :44174 quiet
        assert_eq!(cancel_clothes_window_cue(false, false), None); // :44171
        assert_eq!(save_clothes_window_cue(), 7); // :44197, unconditional
    }

    // ---- builder toggles (Main.cs:47395-47562) ----------------------------

    #[test]
    fn g22_the_binary_builder_slots_flip_zero_one() {
        for slot in [8, 0, 1, 3, 2] {
            assert_eq!(builder_acc_toggle(slot, 0), Some(1));
            assert_eq!(builder_acc_toggle(slot, 1), Some(0));
            // The C# ternary is `== 0 ? 1 : 0` — any nonzero collapses to 0.
            assert_eq!(builder_acc_toggle(slot, 2), Some(0));
        }
    }

    #[test]
    fn g22_the_wire_brightness_slots_wrap_at_three_not_four() {
        for slot in [4, 5, 6, 7, 9] {
            assert_eq!(builder_acc_toggle(slot, 0), Some(1));
            assert_eq!(builder_acc_toggle(slot, 1), Some(2));
            // :47552's `>= 3` — state 2 increments to 3 then wraps to 0,
            // so the tooltip's "Hidden" (3) is unreachable by clicking.
            assert_eq!(builder_acc_toggle(slot, 2), Some(0));
        }
        assert_eq!(builder_acc_toggle(11, 0), None); // :47560 continue
        assert_eq!(builder_acc_toggle(10, 0), None); // no arm
    }

    #[test]
    fn g22_the_info_acc_hide_flip_is_a_plain_not() {
        assert!(info_acc_hide_toggle(false)); // :47203
        assert!(!info_acc_hide_toggle(true));
    }

    // ---- settings button hover (Main.cs:45595-45616) ---------------------

    #[test]
    fn g22_the_settings_button_ticks_once_on_the_enter_edge() {
        // :45599-45603: the enter edge plays 12 and latches.
        assert_eq!(settings_button_tick(true, false, true), Some("Menu_Tick"));
        // Steady hover: no repeat (:45599's `!mouseOver` gate).
        assert_eq!(settings_button_tick(true, true, true), None);
        // Leave: the latch clears with NO cue (:45613-45616).
        assert_eq!(settings_button_tick(false, true, true), None);
        assert_eq!(settings_button_tick(false, false, true), None);
        // IgnoreMouseInterface / mouseInterface: no cue (:45595/:45597).
        assert_eq!(settings_button_tick(true, false, false), None);
    }

    // ---- clothes-window text hover (Main.cs:44279-44355) -----------------

    #[test]
    fn g22_the_clothes_text_buttons_tick_both_edges() {
        // Enter while unlatched: one cue (:44285/:44343).
        assert_eq!(clothes_window_text_hover(true, false), 1);
        // Steady hover: silent.
        assert_eq!(clothes_window_text_hover(true, true), 0);
        // Leave while latched: one cue (:44294/:44352).
        assert_eq!(clothes_window_text_hover(false, true), 1);
        assert_eq!(clothes_window_text_hover(false, false), 0);
    }

    #[test]
    fn g22_the_clothes_clicks_need_the_rect_and_the_press_edge() {
        // :44537/:44614: rect + mouseLeft + mouseLeftRelease.
        assert!(clothes_window_click(true, true, true));
        assert!(!clothes_window_click(false, true, true));
        assert!(!clothes_window_click(true, true, false));
        assert!(!clothes_window_click(true, false, true));
    }

    // ---- case 20 (LegacySoundPlayer.cs:911-923) ---------------------------

    #[test]
    fn g22_the_female_hit_draws_one_next3_and_no_pitch() {
        let mut draws = Vec::new();
        let slot = female_hit_case20(&mut |lo, hi| {
            draws.push((lo, hi));
            2
        });
        assert_eq!(slot, 2);
        // Exactly one draw, the Next(3) half-open range — no pitch roll.
        assert_eq!(draws, vec![(0, 3)]);
    }
}
