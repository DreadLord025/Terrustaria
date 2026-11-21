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

/// The menuMode 2 gender toggle's cue + state decision (`Main.cs:
/// 49041-49052`): the male->female arm plays case 20 (`:49045`) and sets
/// `Male = false`; the female->male arm plays case 1 (`:49050`) and sets
/// `Male = true`. Returns `(new_male, cue_id)`. The `Player.Male` setter
/// remaps `skinVariant` to the alt-gender reference (`Player.cs:
/// 3411-3424`) — the caller applies it through `char_creation::set_male`.
/// Both plays are unpositioned previews (`PlayVoicePreview` parks the
/// player at `(-1, -1)`, `UICharacterCreation.cs:1522-1524`); the case-20
/// slot pick rides `female_hit_case20`, the case-1 pick
/// `sound::player_hit_case1`, each ONE `Next(3)` on the caller's stream
/// AFTER the audibility gate.
pub fn gender_toggle_cue(male: bool) -> (bool, u32) {
    if male {
        (false, 20) // `:49045-49046`
    } else {
        (true, 1) // `:49050-49051`
    }
}

/// The menuMode 2 Randomize row's hair pick (`Main.cs:49075-49076`): the
/// row plays 12 (`:49075`) then `Main.rand.Next(51)` (`:49076`) — the
/// legacy 51-hair pool, NOT the 888 screen's 213-id `AvailableHairstyles`
/// (`UICharacterCreation.cs:1549`). Returns the hair id; the cue is the
/// constant 12 the caller plays BEFORE the draw (the C# order: sound, then
/// the roll chain `:49076-49105`). The rest of the chain (the `randColor`
/// draws, the `Next(60, 120)` skin tint, the `Next(PlayerVariantID.Count)`
/// variant) is the caller's — this helper pins only the g22 site's own
/// `Next(51)` slot.
pub fn randomize_hair_legacy(rng: &mut dyn FnMut(i32) -> i32) -> i32 {
    rng(51) // `:49076`
}

/// `Main.randColor` (`Main.cs:47871-47883`): the rejection-sampled colour
/// the Randomize chain draws — three `Next(256)` channels, re-rolled while
/// `r + g + b <= 150` (`:47876`), returned as `(r, g, b, 255)`. Each
/// rejection consumes THREE draws on the caller's stream; the loop is
/// unbounded in the C# (the `<= 150` gate keeps roughly 1 in 8 rolls).
pub fn rand_color_legacy(rng: &mut dyn FnMut(i32) -> i32) -> [u8; 4] {
    loop {
        let r = rng(256); // `:47878`
        let g = rng(256); // `:47879`
        let b = rng(256); // `:47880`
        if r + g + b > 150 {
            return [r as u8, g as u8, b as u8, 255]; // `:47882`
        }
    }
}

/// The menuMode 17 hair-cycle row's step (`Main.cs:49243-49259`): the
/// forward row (`selectedMenu == 0`) plays 12 and increments `hair`,
/// wrapping at `count` (`:49246-49250`); the back row (`selectedMenu2 ==
/// 0`, the gamepad/secondary cursor) plays 12 and decrements, wrapping to
/// `count - 1` (`:49255-49258`). `count` is the legacy `num15 = 51`
/// (`:49238`). Returns the new hair id; the cue is the constant 12.
pub fn hair_cycle_legacy(hair: i32, count: i32, forward: bool) -> i32 {
    if forward {
        let next = hair + 1; // `:49246`
        if next >= count {
            0 // `:49249`
        } else {
            next
        }
    } else {
        let next = hair - 1; // `:49255`
        if next < 0 {
            count - 1 // `:49258`
        } else {
            next
        }
    }
}

/// `Main.CycleClothingStyle` (`Main.cs:52397-52457`): the menuMode 20
/// Style row (`:49205-49206`) and the clothes-window dummy cycle
/// (`:44618`) advance `skinVariant` through the gender's variant order,
/// skipping the two DLC styles 10/11 (`IsValidPlayerStyle`, `:52406-52413`
/// — `(uint)(skinVariant - 10) <= 1` is false). The male order is
/// `{0, 1, 2, 3, 8, 10}` and the female `{4, 5, 6, 7, 9, 11}`
/// (`PlayerVariantID.cs` Sets ctor); the cycle wraps at the order's end
/// (`:52429-52436`/`:52449-52456`) and the `while (!IsValidPlayerStyle)`
/// loop (`:52400-52403`) skips 10/11 on landing. Returns the new variant.
/// NO RNG — the cycle is a pure table walk; the cue (12 at `:49205`, 12 at
/// `:44617`) is the caller's.
pub fn cycle_clothing_style(skin_variant: i32, male: bool) -> i32 {
    const MALE: [i32; 6] = [0, 1, 2, 3, 8, 10];
    const FEMALE: [i32; 6] = [4, 5, 6, 7, 9, 11];
    let order = if male { &MALE[..] } else { &FEMALE[..] };
    // `CycleClothingStyle_Inner` (`:52415-52457`): find the current index
    // (0 when absent — the C# `num` starts 0), step +1 with wrap.
    let inner = |variant: i32| -> i32 {
        let idx = order.iter().position(|&v| v == variant).unwrap_or(0);
        if idx == order.len() - 1 {
            order[0] // `:52431`/`:52451`
        } else {
            order[idx + 1] // `:52435`/`:52455`
        }
    };
    // `CycleClothingStyle` (`:52397-52404`): one inner step, then re-cycle
    // while `!IsValidPlayerStyle` (10/11 invalid, `:52406-52413`).
    let mut variant = inner(skin_variant);
    while matches!(variant, 10 | 11) {
        variant = inner(variant);
    }
    variant
}

/// The menuMode 2 char-creation row cues (`Main.cs:48985-49123`): the
/// legacy preview screen's per-row `PlaySound` ids, superseded by the 888
/// `UICharacterCreation` in the port (the rows are unwired — this pins the
/// census). Row indices are the C# `selectedMenu`:
///  * 0/1/2/3 — the Hair/Eyes/Skin/Clothes colour opens play 10
///    (`:48991`/`:48997`/`:49003`/`:49009`);
///  * 4 — the gender toggle plays 20 (male->female) or 1 (female->male)
///    (`:49045`/`:49050`), see [`gender_toggle_cue`];
///  * 5 — the difficulty open plays 10 (`:49070`, -> mode 222);
///  * 6 — the Create/name row plays 10 (`:49119`, -> mode 3);
///  * 7 — the Randomize row plays 12 (`:49075`);
///  * 8 — the Back row plays 11 (`:49114`, -> mode 1).
/// Returns the cue id for the row, or `None` for the gender row (its cue
/// is gender-dependent — the caller uses [`gender_toggle_cue`]).
pub fn char_creation_row_cue(row: i32) -> Option<u32> {
    match row {
        0 | 1 | 2 | 3 => Some(10), // `:48991`/`:48997`/`:49003`/`:49009`
        4 => None,                 // gender-dependent, `:49045`/`:49050`
        5 => Some(10),             // `:49070`
        6 => Some(10),             // `:49119`
        7 => Some(12),             // `:49075`
        8 => Some(11),             // `:49114`
        _ => None,
    }
}

/// The menuMode 222 difficulty-select row cues (`Main.cs:49125-49169`):
/// the three difficulty rows (Classic/Mediumcore/Hardcore, `selectedMenu`
/// 1/2/3) set `PendingPlayer.difficulty` and return to mode 2 with NO
/// `PlaySound` — the C# block has none. Returns `None` for every row (the
/// census pin: the difficulty select is silent, unlike the mode-2 row 5
/// open that plays 10).
pub fn difficulty_select_cue(_row: i32) -> Option<u32> {
    None // `:49154-49168` — no PlaySound in the block
}

/// The menuMode 20 clothes-colour row cues (`Main.cs:49170-49207`): the
/// four colour opens (Shirt/Undershirt/Pants/Shoes, `selectedMenu` 0/1/2/3)
/// play 10 (`:49176`/`:49182`/`:49188`/`:49195`); the Style cycle row
/// (`selectedMenu == 4`) plays 12 (`:49205`) then `CycleClothingStyle`;
/// the Back row (`selectedMenu == 5` | flag5) plays 11 (`:49200`).
pub fn clothes_colour_row_cue(row: i32) -> Option<u32> {
    match row {
        0 | 1 | 2 | 3 => Some(10), // `:49176`/`:49182`/`:49188`/`:49195`
        4 => Some(12),             // `:49205` — the Style cycle
        5 => Some(11),             // `:49200` — Back
        _ => None,
    }
}

/// The menuMode 17 hair-style screen's row cues (`Main.cs:49221-49266`):
/// the forward cycle row (`selectedMenu == 0`) plays 12 (`:49245`), the
/// back cycle row (`selectedMenu2 == 0`, the secondary/gamepad cursor)
/// plays 12 (`:49254`), the Back row (`selectedMenu == 2` | flag5) plays
/// 11 (`:49265`). The cycle itself is [`hair_cycle_legacy`]. `row` is the
/// primary `selectedMenu`; `back_cycle` is the `selectedMenu2 == 0` read.
pub fn hair_style_screen_cue(row: i32, back_cycle: bool) -> Option<u32> {
    if row == 0 {
        Some(12) // `:49245` — forward cycle
    } else if back_cycle {
        Some(12) // `:49254` — back cycle (selectedMenu2 == 0)
    } else if row == 2 {
        Some(11) // `:49265` — Back
    } else {
        None
    }
}

/// The legacy colour-screen Back rows (`Main.cs:49268-49410`): menuModes
/// 18/19/21/22/23/24 (the Eyes/Skin/Shirt/Undershirt/Pants/Shoes colour
/// pickers) each play 11 on Back (`selectedMenu == 2` | flag5) and return
/// to mode 2 (18/19) or mode 20 (21/22/23/24). The cue is the constant 11
/// for every one — the census pin (the screens are unwired, superseded by
/// the 888 HSL menu).
pub fn colour_screen_back_cue() -> u32 {
    11 // `:49289`/`:49313`/`:49337`/`:49361`/`:49385`/`:49409`
}

/// The menuMode -7 world-difficulty select's row cues (`Main.cs:
/// 49473-49538`): the three difficulty rows (Classic/Expert/Master,
/// `selectedMenu` 2/3/4) set `GameMode` and play 10 (`:49506`/`:49516`/
/// `:49526`) then open mode 7 (or -71 when `SettingsUnlock_WorldEvil`);
/// the Back row (`selectedMenu == 5` | flag5) plays 11 (`:49536`) and
/// returns to mode 16. `row` is the C# `selectedMenu`.
pub fn world_difficulty_row_cue(row: i32) -> Option<u32> {
    match row {
        2 | 3 | 4 => Some(10), // `:49506`/`:49516`/`:49526`
        5 => Some(11),         // `:49536` — Back
        _ => None,
    }
}

/// The menuMode -71 world-evil select's row cues (`Main.cs:49541-49584`):
/// the three evil rows (Corruption/Crimson/Random, `WorldGenParam_Evil`
/// 0/1/-1) play 10 (`:49558`/`:49566`/`:49574`) then open mode 7; the Back
/// row plays 11 (`:49582`) and returns to mode -7. The row indices are the
/// C# `num16` walk (0/1/2 for the evils, 3 for Back).
pub fn world_evil_row_cue(row: i32) -> Option<u32> {
    match row {
        0 | 1 | 2 => Some(10), // `:49558`/`:49566`/`:49574`
        3 => Some(11),         // `:49582` — Back
        _ => None,
    }
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

    // ---- char-creation legacy preview (Main.cs:49041-49259) --------------

    #[test]
    fn g22_the_gender_toggle_splits_case20_and_case1() {
        // male->female: case 20, Male=false (:49045-49046).
        assert_eq!(gender_toggle_cue(true), (false, 20));
        // female->male: case 1, Male=true (:49050-49051).
        assert_eq!(gender_toggle_cue(false), (true, 1));
    }

    #[test]
    fn g22_the_legacy_randomize_hair_draws_next51_not_next213() {
        // :49076 — the legacy pool is 51, NOT the 888 screen's 213.
        let mut draws = Vec::new();
        let hair = randomize_hair_legacy(&mut |max| {
            draws.push(max);
            50
        });
        assert_eq!(hair, 50);
        assert_eq!(draws, vec![51]);
    }

    #[test]
    fn g22_rand_color_rejects_until_the_sum_exceeds_150() {
        // :47876 — three Next(256) per attempt, re-roll while sum <= 150.
        let mut rolls = [10, 20, 30, /* sum 60 <= 150: reject */ 100, 100, 100];
        let mut i = 0;
        let mut draws = 0;
        let c = rand_color_legacy(&mut |_| {
            let v = rolls[i];
            i += 1;
            draws += 1;
            v
        });
        assert_eq!(c, [100, 100, 100, 255]);
        assert_eq!(draws, 6, "two attempts, three draws each");
        // The boundary: sum exactly 150 is rejected (<= 150).
        let mut rolls = [50, 50, 50, /* 150: reject */ 50, 50, 51];
        let mut i = 0;
        let c = rand_color_legacy(&mut |_| {
            let v = rolls[i];
            i += 1;
            v
        });
        assert_eq!(c, [50, 50, 51, 255], "151 passes");
    }

    #[test]
    fn g22_the_hair_cycle_wraps_at_the_legacy_51_pool() {
        // Forward (:49246-49250): increment, wrap at count.
        assert_eq!(hair_cycle_legacy(0, 51, true), 1);
        assert_eq!(hair_cycle_legacy(50, 51, true), 0);
        // Back (:49255-49258): decrement, wrap to count-1.
        assert_eq!(hair_cycle_legacy(1, 51, false), 0);
        assert_eq!(hair_cycle_legacy(0, 51, false), 50);
    }

    #[test]
    fn g22_the_clothing_cycle_walks_the_gender_order_and_skips_dlc() {
        // Male order {0,1,2,3,8,10}: 3 -> 8 (:52435).
        assert_eq!(cycle_clothing_style(3, true), 8);
        // 8 -> 10 is invalid (DLC), re-cycle 10 -> 0 (:52400-52403).
        assert_eq!(cycle_clothing_style(8, true), 0);
        // The wrap: the last valid male is 8 (10 skipped), so 8 wraps to 0.
        // Female order {4,5,6,7,9,11}: 7 -> 9.
        assert_eq!(cycle_clothing_style(7, false), 9);
        // 9 -> 11 invalid, re-cycle 11 -> 4.
        assert_eq!(cycle_clothing_style(9, false), 4);
        // An absent variant starts at index 0 (:52419 num=0), steps to [1].
        assert_eq!(cycle_clothing_style(99, true), 1);
    }

    // ---- char-creation row cues (Main.cs:48985-49207) --------------------

    #[test]
    fn g22_the_mode2_rows_cue_ten_twelve_eleven_and_gender_splits() {
        // The four colour opens play 10 (:48991/:48997/:49003/:49009).
        for row in 0..4 {
            assert_eq!(char_creation_row_cue(row), Some(10));
        }
        // The gender row is gender-dependent (None here; gender_toggle_cue).
        assert_eq!(char_creation_row_cue(4), None);
        // Difficulty open 10 (:49070), Create/name 10 (:49119).
        assert_eq!(char_creation_row_cue(5), Some(10));
        assert_eq!(char_creation_row_cue(6), Some(10));
        // Randomize 12 (:49075), Back 11 (:49114).
        assert_eq!(char_creation_row_cue(7), Some(12));
        assert_eq!(char_creation_row_cue(8), Some(11));
        assert_eq!(char_creation_row_cue(9), None);
    }

    #[test]
    fn g22_the_difficulty_select_is_silent() {
        // :49154-49168 — the mode-222 block has NO PlaySound.
        for row in 1..4 {
            assert_eq!(difficulty_select_cue(row), None);
        }
    }

    #[test]
    fn g22_the_mode20_colour_rows_cue_ten_and_the_style_cycle_twelve() {
        // The four colour opens play 10 (:49176/:49182/:49188/:49195).
        for row in 0..4 {
            assert_eq!(clothes_colour_row_cue(row), Some(10));
        }
        // Style cycle 12 (:49205), Back 11 (:49200).
        assert_eq!(clothes_colour_row_cue(4), Some(12));
        assert_eq!(clothes_colour_row_cue(5), Some(11));
        assert_eq!(clothes_colour_row_cue(6), None);
    }

    // ---- hair-style screen + colour Backs + world creation (151-165) -----

    #[test]
    fn g22_the_hair_style_screen_cycles_tick_and_back_closes() {
        // Forward cycle row 0 plays 12 (:49245).
        assert_eq!(hair_style_screen_cue(0, false), Some(12));
        // Back cycle (selectedMenu2 == 0) plays 12 (:49254).
        assert_eq!(hair_style_screen_cue(1, true), Some(12));
        // Back row 2 plays 11 (:49265).
        assert_eq!(hair_style_screen_cue(2, false), Some(11));
        // The forward row wins over the back-cycle read (the C# else-if).
        assert_eq!(hair_style_screen_cue(0, true), Some(12));
        assert_eq!(hair_style_screen_cue(1, false), None);
    }

    #[test]
    fn g22_the_colour_screen_backs_all_close_with_eleven() {
        // :49289/:49313/:49337/:49361/:49385/:49409 — every colour picker
        // Back plays 11.
        assert_eq!(colour_screen_back_cue(), 11);
    }

    #[test]
    fn g22_the_world_difficulty_rows_cue_ten_and_back_eleven() {
        // Classic/Expert/Master (rows 2/3/4) play 10 (:49506/:49516/:49526).
        for row in 2..5 {
            assert_eq!(world_difficulty_row_cue(row), Some(10));
        }
        // Back (row 5) plays 11 (:49536).
        assert_eq!(world_difficulty_row_cue(5), Some(11));
        assert_eq!(world_difficulty_row_cue(0), None);
    }

    #[test]
    fn g22_the_world_evil_rows_cue_ten_and_back_eleven() {
        // Corruption/Crimson/Random (rows 0/1/2) play 10 (:49558/:49566/:49574).
        for row in 0..3 {
            assert_eq!(world_evil_row_cue(row), Some(10));
        }
        // Back (row 3) plays 11 (:49582).
        assert_eq!(world_evil_row_cue(3), Some(11));
        assert_eq!(world_evil_row_cue(4), None);
    }
}
