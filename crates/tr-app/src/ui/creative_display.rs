//! The Journey "infinite items" display's state machines — W4-d82
//! (research/18b row 22): `UICreativeInfiniteItemsDisplay`
//! (`GameContent/UI/Elements/UICreativeInfiniteItemsDisplay.cs:15-578`).
//!
//! The class is the creative menu's two-page body (the infinites grid + the
//! research/sacrifice window). The retained tree (panels, cogs, pistons,
//! particles) is the screen's draw seam; this module ports the *pure state* the
//! draw reads: the sacrifice animation's frame/cog/sound schedule, the
//! description text's remember/forget latch, the page switch, and the
//! tracker-edit rebuild gate. The grid geometry rides `ui::item_collection`,
//! the filter board `craft_filter::FilterBoard`.
//!
//! RNG deviation (docs/ANALYSIS.md): the `:351-364` particle burst rolls
//! `Main.rand.NextVector2Circular` 15 times at `timeLeft == 56`; the port keeps
//! the *schedule* (when the burst fires) but not the rolls, so wiring it cannot
//! perturb the shared stream before the particle seam exists.

use tr_core::scene::get_lerp_value;

/// `InfiniteItemsDisplayPage` (`:17-21`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    InfiniteItemsPickup,
    InfiniteItemsResearch,
}

/// `SetPageTypeToShow` (`:500-503`): Research shows the sacrifice container.
pub fn show_sacrifices(page: Page) -> bool {
    page == Page::InfiniteItemsResearch
}

/// `BeginSacrificeAnimation` (`:482-485`): the timer arms to 60.
pub const SACRIFICE_ANIMATION_FRAMES: i32 = 60;

/// `GetSacrificeAnimationProgress` (`:495-498`):
/// `GetLerpValue(60, 0, timeLeft, clamped)` — 0 at arm, 1 at expiry.
pub fn sacrifice_progress(time_left: i32) -> f32 {
    get_lerp_value(60.0, 0.0, time_left as f32, true)
}

/// `UpdateSacrificeAnimation` (`:487-493`): a positive timer loses one frame.
pub fn tick_sacrifice_animation(time_left: &mut i32) {
    if *time_left > 0 {
        *time_left -= 1;
    }
}

/// `UpdateVisualFrame`'s piston frame pick (`:341-346`): while the animation
/// runs, the progress thresholds select frame row 1..8; an idle timer keeps 0.
/// The cascade is the source's nested ternary, top-down.
pub fn piston_frame_y(progress: f32, animating: bool) -> i32 {
    if !animating {
        return 0; // `:341` frameY = 0
    }
    let num4 = 0.1f32;
    let num5 = 1.0f32 / 15.0;
    if progress >= 1.0 - num4 {
        8
    } else if progress >= 1.0 - num4 * 2.0 {
        7
    } else if progress >= 1.0 - num4 * 3.0 {
        6
    } else if progress >= num5 * 4.0 {
        5
    } else if progress >= num5 * 3.0 {
        4
    } else if progress >= num5 * 2.0 {
        3
    } else if !(progress >= num5) {
        1
    } else {
        2
    }
}

/// `UpdateVisualFrame`'s cog speed (`:332-340`): the base step 0.05 scales by
/// `1 + lerp² * 2` where `lerp = GetLerpValue(1, 0.7, progress, clamped)`; the
/// three cog pairs offset by `2·step`, `1.1428572·step`, `-1·step` (the big
/// pair counter-rotates). Returns `(small, medium, big)` rotation deltas.
pub fn cog_rotation_offsets(progress: f32) -> [f32; 3] {
    let mut num = 0.05f32;
    let lerp = get_lerp_value(1.0, 0.7, progress, true);
    num *= 1.0 + lerp * lerp * 2.0;
    let num2 = 1.1428572f32;
    let num3 = 1.0f32;
    [2.0 * num, num2 * num, (0.0 - num3) * num]
}

/// `OffsetRotationsForCogs` (`:375-379`): a pair's two cogs spin opposite ways.
pub fn offset_pair(rotations: &mut [f32; 2], delta: f32) {
    rotations[0] += delta;
    rotations[1] -= delta;
}

/// The sound/particle schedule inside `UpdateVisualFrame` (`:347-370`): at
/// `timeLeft == 56` play 63 + spawn the 15-particle burst; at `timeLeft == 40`
/// with `_researchComplete` play 64 and clear the flag. Returns
/// `(play_63, spawn_burst, play_64)` for this frame.
pub fn sacrifice_sound_events(time_left: i32, research_complete: bool) -> (bool, bool, bool) {
    let play_63 = time_left == 56;
    let play_64 = time_left == 40 && research_complete;
    (play_63, play_63, play_64)
}

/// The particle burst's constants (`:350-363`): 15 particles, acceleration
/// `(0, 0.16350001)`, scale offset `-1/60` per frame. The velocity rolls are
/// the RNG deviation above; the count + physics are ported.
pub const BURST_COUNT: usize = 15;
pub const BURST_ACCELERATION: [f32; 2] = [0.0, 0.16350001];
pub const BURST_SCALE_OFFSET: f32 = -1.0 / 60.0;

/// `descriptionText_OnUpdate`'s remember/forget latch (`:404-434`, `:468-480`).
/// The text shows the last sacrificed item's `(have/needed)` while the slot is
/// empty, `???` when nothing was remembered (or it was already complete), `X`
/// when the slot item has no sacrifice numbers, else the live `(have/needed)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SacrificeMemory {
    /// `_lastItemIdSacrificed` (`:67`).
    pub item_id: i32,
    /// `_lastItemAmountWeHad` (`:69`).
    pub had: i32,
    /// `_lastItemAmountWeNeededTotal` (`:71`).
    pub needed: i32,
}

impl SacrificeMemory {
    /// `RememberItemSacrifice` (`:468-473`).
    pub fn remember(&mut self, item_id: i32, had: i32, needed: i32) {
        self.item_id = item_id;
        self.had = had;
        self.needed = needed;
    }

    /// `ForgetItemSacrifice` (`:475-480`).
    pub fn forget(&mut self) {
        *self = Self::default();
    }
}

/// The description text pick (`:404-434`). Inputs: whether the cursor carries
/// an item (`!Main.mouseItem.IsAir`, `:409` — forces a forget), the slot's
/// checked id + numbers (`GetSacrificeNumbers`, `:407`), and the memory.
/// Returns the string the `UIText.SetText` receives; the memory is mutated
/// exactly as the source's forget arms do.
pub fn description_text(
    mouse_item_is_air: bool,
    checked_id: i32,
    numbers_ok: bool,
    have: i32,
    needed: i32,
    mem: &mut SacrificeMemory,
) -> String {
    if !mouse_item_is_air {
        mem.forget(); // `:409-412`
    }
    if checked_id == 0 {
        // `:413-424` — the empty slot: the remembered incomplete pair, else ???.
        if mem.item_id != 0 && mem.needed != mem.had {
            return format!("({}/{})", mem.had, mem.needed);
        }
        return "???".to_string();
    }
    mem.forget(); // `:425`
    if !numbers_ok {
        "X".to_string() // `:426-429`
    } else {
        format!("({have}/{needed})") // `:430-433`
    }
}

/// `CreativeUI.ItemSacrificeResult` (`CreativeUI.cs:20-25`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SacrificeResult {
    CannotSacrifice,
    SacrificedButNotDone,
    SacrificedAndDone,
}

/// `SacrificeWhatYouCan`'s result handling (`:441-458`): both success arms arm
/// the animation and remember `have + sacrificed`; only `AndDone` sets
/// `_researchComplete`. Returns `(begin_animation, research_complete, remembered)`
/// where `remembered` is the `(had, needed)` pair to store (or `None`).
pub fn sacrifice_outcome(
    result: SacrificeResult,
    checked_id: i32,
    have: i32,
    sacrificed: i32,
    needed: i32,
) -> (bool, bool, Option<(i32, i32, i32)>) {
    match result {
        SacrificeResult::SacrificedAndDone => {
            (true, true, Some((checked_id, have + sacrificed, needed)))
        }
        SacrificeResult::SacrificedButNotDone => {
            (true, false, Some((checked_id, have + sacrificed, needed)))
        }
        SacrificeResult::CannotSacrifice => (false, false, None),
    }
}

/// `UICreativeInfiniteItemsDisplay_OnUpdate`'s rebuild gate (`:505-529`): the
/// contents rebuild when the tracker object changes (reset the version to -1)
/// or its `LastEditId` differs from the last checked one. Returns whether
/// `UpdateContents` runs this update.
pub fn needs_rebuild(last_edit_id: i32, last_checked: &mut i32, tracker_changed: bool) -> bool {
    if tracker_changed {
        *last_checked = -1; // `:509-513`
    }
    if *last_checked != last_edit_id {
        *last_checked = last_edit_id; // `:515-518`
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The progress + frame cascade (`:341-346`, `:495-498`): at arm (60) the
    /// progress is 0 → frame 1; at expiry (0) it is 1 → frame 8; idle → 0.
    #[test]
    fn the_piston_frame_follows_the_progress_cascade() {
        assert_eq!(sacrifice_progress(60), 0.0);
        assert_eq!(sacrifice_progress(0), 1.0);
        assert_eq!(piston_frame_y(0.0, true), 1);
        assert_eq!(piston_frame_y(1.0, true), 8);
        assert_eq!(piston_frame_y(0.95, true), 8); // >= 1 - 0.1
        assert_eq!(piston_frame_y(0.85, true), 7); // >= 1 - 0.2
        assert_eq!(piston_frame_y(0.75, true), 6); // >= 1 - 0.3
        assert_eq!(piston_frame_y(0.0, false), 0);
        let mut t = 60;
        tick_sacrifice_animation(&mut t);
        assert_eq!(t, 59);
        t = 0;
        tick_sacrifice_animation(&mut t);
        assert_eq!(t, 0, "the timer never goes negative");
    }

    /// The cog speed + the sound schedule (`:332-340`, `:347-370`).
    #[test]
    fn the_cogs_speed_up_and_the_sounds_fire_on_schedule() {
        // At progress 0 the lerp is 0 → base step 0.05.
        let [s, m, b] = cog_rotation_offsets(0.0);
        assert!((s - 0.1).abs() < 1e-6);
        assert!((m - 0.05714286).abs() < 1e-5);
        assert!((b + 0.05).abs() < 1e-6);
        // At progress 1 the lerp is 1 → step 0.05 * 3 = 0.15.
        let [s1, _, _] = cog_rotation_offsets(1.0);
        assert!((s1 - 0.3).abs() < 1e-6);
        let mut pair = [0.0, 0.0];
        offset_pair(&mut pair, 0.1);
        assert_eq!(pair, [0.1, -0.1]);
        assert_eq!(sacrifice_sound_events(56, false), (true, true, false));
        assert_eq!(sacrifice_sound_events(40, true), (false, false, true));
        assert_eq!(sacrifice_sound_events(40, false), (false, false, false));
        assert_eq!(BURST_COUNT, 15);
    }

    /// The description latch (`:404-434`): empty slot + incomplete memory shows
    /// the pair; a carried cursor item forgets; a live slot shows X or the pair.
    #[test]
    fn the_description_text_remembers_and_forgets() {
        let mut mem = SacrificeMemory::default();
        assert_eq!(description_text(true, 0, false, 0, 0, &mut mem), "???");
        mem.remember(8, 50, 100);
        assert_eq!(description_text(true, 0, false, 0, 0, &mut mem), "(50/100)");
        // A completed memory (had == needed) falls back to ???.
        mem.remember(8, 100, 100);
        assert_eq!(description_text(true, 0, false, 0, 0, &mut mem), "???");
        // A live slot with numbers shows the pair and forgets the memory.
        mem.remember(8, 50, 100);
        assert_eq!(description_text(true, 8, true, 60, 100, &mut mem), "(60/100)");
        assert_eq!(mem.item_id, 0);
        // No numbers → X; a carried item forces a forget first.
        assert_eq!(description_text(false, 0, false, 0, 0, &mut mem), "???");
        assert_eq!(description_text(true, 9, false, 0, 0, &mut mem), "X");
    }

    /// The sacrifice outcome + rebuild gate (`:441-458`, `:505-529`).
    #[test]
    fn the_sacrifice_outcome_and_rebuild_gate() {
        let (anim, done, rem) = sacrifice_outcome(SacrificeResult::SacrificedAndDone, 8, 50, 50, 100);
        assert!(anim && done);
        assert_eq!(rem, Some((8, 100, 100)));
        let (anim2, done2, _) = sacrifice_outcome(SacrificeResult::SacrificedButNotDone, 8, 50, 10, 100);
        assert!(anim2 && !done2);
        assert_eq!(sacrifice_outcome(SacrificeResult::CannotSacrifice, 8, 50, 0, 100), (false, false, None));
        let mut checked = -1;
        assert!(needs_rebuild(0, &mut checked, false), "the first edit rebuilds");
        assert!(!needs_rebuild(0, &mut checked, false), "no edit: no rebuild");
        assert!(needs_rebuild(1, &mut checked, false), "a new edit id rebuilds");
        assert!(needs_rebuild(1, &mut checked, true), "a tracker swap forces it");
        assert_eq!(show_sacrifices(Page::InfiniteItemsResearch), true);
        assert_eq!(show_sacrifices(Page::InfiniteItemsPickup), false);
    }
}
