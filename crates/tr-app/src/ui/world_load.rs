//! The world-loading screen's state + progress math — `UIWorldLoad`
//! (`GameContent/UI/States/UIWorldLoad.cs:16-137`) and its
//! `UIGenProgressBar` (`GameContent/UI/Elements/UIGenProgressBar.cs:8-126`).
//!
//! S1-E-17: wired as `FrontScreen::WorldLoad` — the generation worker
//! (`app/worldgen_job.rs`) feeds the progress through the `GenProgressHook`
//! seam; the sprite draw below paints the bar + header. The
//! `CaptureManager` GPU chunk render is the S1-G-13 render-target seam.
//!
//! SOUND CENSUS (W4-g24 FINAL, research/146 §4): the screen's ONE audible
//! cue is `Update`'s Inventory-abort `PlaySound(11)`
//! (`UIWorldLoad.cs:57`) — landed in `session.rs::world_load_queue_abort`
//! (`Menu_Close`, once per queue via the `!QueuedAbort` gate; the pure
//! `queue_abort_on_inventory` below returns the edge, the caller owns the
//! play). The bar itself (`UIGenProgressBar`) is CUE-LESS — zero
//! `PlaySound` in the element; the F5 arm (`:50-53`) opens `UIWorldGenDebug`
//! which is ALSO cue-less (f23's pass-icons/debug-elements ports carry no
//! sound sites) and stays DEBUG-DEFERRED. The generation-start 10
//! (`WorldGen.cs:6593`) and completion 10 (`:6603`) bracket the screen —
//! both landed (g24 batch 1 / S1-E-17). No gen-progress cue is missing.

use tr_core::scene::get_lerp_value;

/// The worldgen progress snapshot — `GenerationProgress`'s three fields the
/// loading screen reads (`UIWorldLoad.cs:83-85`: `TotalProgress`, `Value`,
/// `Message`). Doubles in C#; the draw side casts to float
/// (`UIWorldLoad.cs:83-84`), which [`GenProgressBar::draw_frame`] mirrors.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GenerationProgress {
    /// `GenerationProgress.TotalProgress` — the whole worldgen's fraction.
    pub total_progress: f64,
    /// `GenerationProgress.Value` — the current pass's fraction.
    pub value: f64,
    /// `GenerationProgress.Message` — the current pass's label.
    pub message: String,
}

/// `UIGenProgressBar` (`UIGenProgressBar.cs:8-126`) — the two target/visual
/// progress pairs and the bar-width constants. The visual fields are kept
/// even though the 1.4.5.8 `DrawSelf` assigns them straight from the targets
/// (`:68-69` — no smoothing lerp in this version); the fields exist in the
/// source and the port mirrors them 1:1.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GenProgressBar {
    visual_overall_progress: f32,
    target_overall_progress: f32,
    visual_current_progress: f32,
    target_current_progress: f32,
}

/// `UIGenProgressBar._smallBarWidth = 508` / `_longBarWidth = 570`
/// (`UIGenProgressBar.cs:26-28`).
pub(crate) const SMALL_BAR_WIDTH: i32 = 508;
pub(crate) const LONG_BAR_WIDTH: i32 = 570;

/// The per-frame geometry `UIGenProgressBar.DrawSelf` computes
/// (`UIGenProgressBar.cs:68-80`) — the inputs the deferred sprite draw will
/// consume at the S1-G-13 seam.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GenBarFrame {
    /// `(int)(_visualOverallProgress * (float)_longBarWidth)` after
    /// `DrawFilling2`'s even round-down (`:71`, `:118-121`).
    pub overall_width: i32,
    /// `(int)(_visualCurrentProgress * (float)_smallBarWidth)` after the
    /// same round-down (`:72`, `:118-121`).
    pub current_width: i32,
    /// The long bar's `Color.PackedValue` (`:74-77`): random-evil
    /// `4292696893`, crimson `4286836223`, corrupt `4283888223`.
    pub overall_color: u32,
    /// The small bar's fixed `Color.PackedValue = 4290947159` (`:79`).
    pub current_color: u32,
}

impl GenProgressBar {
    /// `SetProgress` (`UIGenProgressBar.cs:51-55`): only the targets move;
    /// the visual pair follows on the next draw (`:68-69`). Current first,
    /// overall second — the source's assignment order.
    pub(crate) fn set_progress(&mut self, overall_progress: f32, current_progress: f32) {
        self.target_current_progress = current_progress;
        self.target_overall_progress = overall_progress;
    }

    /// `DrawSelf`'s math head (`UIGenProgressBar.cs:68-80`) minus the
    /// `spriteBatch.Draw` calls (deferred to S1-G-13). The drunk-world evil
    /// flicker (`:63-67`, two/three `Main.rand.Next` rolls) is deliberately
    /// not ported: it draws from `Main.rand` and wiring it before the draw
    /// seam exists would perturb that stream's order (docs/ANALYSIS.md).
    /// `evil_crimson` = `WorldGen.crimson`, `random_evil` =
    /// `WorldGen.generatingRandomEvil` (`:61-62`).
    pub(crate) fn draw_frame(&mut self, evil_crimson: bool, random_evil: bool) -> GenBarFrame {
        // `:68-69` — the 1.4.5.8 version copies the targets verbatim.
        self.visual_overall_progress = self.target_overall_progress;
        self.visual_current_progress = self.target_current_progress;
        // `:71-72` — float multiply, C# `(int)` truncation toward zero.
        let completed_width = (self.visual_overall_progress * LONG_BAR_WIDTH as f32) as i32;
        let completed_width2 = (self.visual_current_progress * SMALL_BAR_WIDTH as f32) as i32;
        GenBarFrame {
            overall_width: filling_even_width(completed_width),
            current_width: filling_even_width(completed_width2),
            // `:74-77` — flag2 ? 4292696893 : (flag ? 4286836223 : 4283888223).
            overall_color: if random_evil {
                4292696893
            } else if evil_crimson {
                4286836223
            } else {
                4283888223
            },
            current_color: 4290947159, // `:79`
        }
    }
}

/// `DrawFilling2`'s head (`UIGenProgressBar.cs:118-121`): an odd completed
/// width loses one pixel so the fill stays on 2 px steps. (`DrawFilling`,
/// `:90-93`, rounds the same way.)
pub(crate) fn filling_even_width(completed_width: i32) -> i32 {
    if completed_width % 2 != 0 {
        completed_width - 1
    } else {
        completed_width
    }
}

/// `Color.Lerp(value1, value2, amount)` (XNA) as the bars use it —
/// `Color.Lerp(color, Color.Black, 0.5f)` (`UIGenProgressBar.cs:78`, `:80`):
/// a per-byte `MathHelper.Lerp` (unclamped float lerp) truncated to byte by
/// the `Color(byte, byte, byte, byte)` ctor. The packed values are XNA's
/// ABGR (`0xAARRGGBB` reversed in memory).
pub(crate) fn lerp_color_packed(value1: u32, value2: u32, amount: f32) -> u32 {
    let mut out = 0u32;
    for shift in [0, 8, 16, 24] {
        let a = ((value1 >> shift) & 0xFF) as f32;
        let b = ((value2 >> shift) & 0xFF) as f32;
        let lerped = (a + (b - a) * amount) as u8;
        out |= (lerped as u32) << shift;
    }
    out
}

/// `UIWorldLoad`'s state (`UIWorldLoad.cs:16-137`): the progress bar, the
/// cached `_progress` reference (taken once per `Draw`, `:69`), the
/// `QueuedAbort` flag the Update's Inventory trigger sets (`:54-58`, on
/// `WorldGenerator.CurrentController`), and the bar's vertical position the
/// Update lerps by screen height (`:60-61`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorldLoadState {
    /// `_progressBar` (`:18`).
    pub bar: GenProgressBar,
    /// `_progress` (`:22`) — `None` while `WorldGenerator.
    /// CurrentGenerationProgress` is null; `Draw` then skips the whole
    /// screen (`:70-73`).
    pub progress: Option<GenerationProgress>,
    /// `WorldGenerator.CurrentController.QueuedAbort` (`:54-58`, read at
    /// `:86`). The controller itself (the abortable worldgen thread) is the
    /// deferred piece; the flag's UI behaviour is ported here.
    pub queued_abort: bool,
    /// `_progressBar.Top.Pixels` — 270 from the ctor (`:26`), re-lerped
    /// every Update (`:60`).
    pub bar_top: f32,
    /// `_progressMessage.Top.Pixels` — always `bar_top - 70` (`:31`, `:61`).
    pub message_top: f32,
    /// The secret-seed message overrides' inputs (`UIWorldLoad.cs:93-116`),
    /// resolved from the live job's seed at start (`WorldGen.Reset`).
    pub flags: WorldLoadFlags,
    /// `WorldGen.crimson` (`UIGenProgressBar.cs:61`) — the bar's colour pick.
    pub evil_crimson: bool,
    /// `WorldGen.generatingRandomEvil` (`UIGenProgressBar.cs:62`).
    pub random_evil: bool,
}

/// The secret-seed message overrides `DrawSelf` reads (`UIWorldLoad.cs:93`,
/// `:104`, `:108`): `WorldGen.drunkWorldGen` / `notTheBees` /
/// `getGoodWorldGen` plus the two gates (`placingTraps`, `noTrapsWorldGen`)
/// and `Main.zenithWorld`. The drunk arm's random-digit message (`:95-102`)
/// rolls `Main.rand` and is deferred with the draw wiring (ANALYSIS).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct WorldLoadFlags {
    pub drunk_world_gen: bool,
    pub not_the_bees: bool,
    pub get_good_world_gen: bool,
    pub no_traps_world_gen: bool,
    pub placing_traps: bool,
    pub zenith_world: bool,
    /// `WorldGen.tenthAnniversaryWorldGen` — `UIHeader.DrawSelf`'s HotPink
    /// arm (`UIHeader.cs:54-57`).
    pub tenth_anniversary_world_gen: bool,
}

impl WorldLoadState {
    /// The `UIWorldLoad` ctor (`UIWorldLoad.cs:24-35`): bar top 270, the
    /// message header copies the style 70 px above (`:30-31`).
    pub(crate) fn new() -> Self {
        Self {
            bar_top: 270.0,
            message_top: 200.0,
            ..Self::default()
        }
    }

    /// `Update`'s screen-height lerp (`UIWorldLoad.cs:60-61`):
    /// `MathHelper.Lerp(270, 370, Utils.GetLerpValue(600, 700,
    /// Main.screenHeight, clamped: true))`, message 70 px above.
    pub(crate) fn update_layout(&mut self, screen_height: f32) {
        let t = get_lerp_value(600.0, 700.0, screen_height, true);
        self.bar_top = 270.0 + (370.0 - 270.0) * t;
        self.message_top = self.bar_top - 70.0;
    }

    /// `Update`'s Inventory-abort arm (`UIWorldLoad.cs:54-58`): the trigger
    /// queues the abort once (the `!QueuedAbort` gate) and plays legacy
    /// sound 11. Returns whether the abort was newly queued so the caller
    /// owns the `SoundEngine.PlaySound(11)` side effect.
    pub(crate) fn queue_abort_on_inventory(&mut self, inventory_trigger: bool) -> bool {
        if inventory_trigger && !self.queued_abort {
            self.queued_abort = true;
            true
        } else {
            false
        }
    }

    /// `Draw`'s progress latch (`UIWorldLoad.cs:69`): `_progress =
    /// WorldGenerator.CurrentGenerationProgress`. `None` hides the screen
    /// (`:70-73` — `base.Draw` only runs when the snapshot exists).
    pub(crate) fn set_progress(&mut self, progress: Option<GenerationProgress>) {
        self.progress = progress;
    }

    /// Whether `Draw` renders anything (`UIWorldLoad.cs:70-73`).
    pub(crate) fn draws(&self) -> bool {
        self.progress.is_some()
    }

    /// `DrawSelf`'s message assembly + `SetProgress` feed
    /// (`UIWorldLoad.cs:78-116`), in source order: the snapshot's fields
    /// (zero/empty when null, `:78-85`), the `UI.Canceling` override when
    /// the abort is queued (`:86-89`), the bar feed (`:91`), then the
    /// secret-seed overrides — bees (`:104-107`) and the get-good reversal
    /// (`:108-116`). The drunk random-digit arm (`:93-103`) is deferred
    /// (see [`WorldLoadFlags`]); `Main.gameTips` (`:117-118`) is research-19
    /// M3, out of this round's scope.
    pub(crate) fn draw_self_message(&mut self, flags: WorldLoadFlags) -> String {
        let (overall, current, mut text) = match &self.progress {
            Some(p) => (p.total_progress as f32, p.value as f32, p.message.clone()),
            None => (0.0, 0.0, String::new()),
        };
        if self.progress.is_some() && self.queued_abort {
            // `:86-89` — Language.GetTextValue("UI.Canceling").
            text = crate::lang::text("UI.Canceling");
        }
        // `:91-92` — the bar feed and the header text land before the
        // secret-seed overrides rewrite the header only.
        self.bar.set_progress(overall, current);
        if flags.not_the_bees && !flags.zenith_world {
            // `:104-107`.
            text = crate::lang::text("UI.WorldGenEasterEgg_GeneratingBees");
        }
        if flags.get_good_world_gen && (!flags.no_traps_world_gen || !flags.placing_traps) {
            // `:108-116` — the message reversed character by character.
            // The C# walks UTF-16 code units (`Text.Substring(num, 1)`);
            // `chars()` walks Unicode scalars — they differ only on
            // surrogate-pair characters (emoji etc.), which the gen
            // messages never contain (ANALYSIS "The loading screen").
            text = text.chars().rev().collect();
        }
        text
    }

    /// `GetStatusText` (`UIWorldLoad.cs:129-136`): `"{0:0.0%} - {Message} -
    /// {1:0.0%}"`, or `"0.0% - ... - 0.0%"` with no snapshot. The percent
    /// formatting rides [`format_percent_tenth`].
    pub(crate) fn status_text(&self) -> String {
        match &self.progress {
            None => "0.0% - ... - 0.0%".to_string(),
            Some(p) => format!(
                "{} - {} - {}",
                format_percent_tenth(p.total_progress),
                p.message,
                format_percent_tenth(p.value)
            ),
        }
    }
}

/// .NET's `"0.0%"` custom format on the progress doubles
/// (`UIWorldLoad.cs:133`, `:135`): ×100, one decimal, `%` appended. .NET
/// rounds the scaled value half-away-from-zero — a tie IS representable
/// (e.g. `0.0625` → `6.25%` → `"6.3%"`), where Rust's `{:.1}` half-to-even
/// would print `"6.2%"`. Rides the shared
/// [`tr_core::generation_progress::format_percent`] so the status text and
/// the `Message` `{0:0.0%}` substitution can never disagree.
pub(crate) fn format_percent_tenth(value: f64) -> String {
    tr_core::generation_progress::format_percent(value)
}

// --- The draw (UIWorldLoad.Draw + UIGenProgressBar.DrawSelf) ------------------

use tr_render::SpriteRenderer;

use crate::assets;

use super::color;
use super::text::{font_width, push_font_text};
use super::uistate::{self, Style};

/// `UIWorldLoad.Draw` + `DrawSelf` + the `UIGenProgressBar`/`UIHeader`
/// children (`UIWorldLoad.cs:67-120`, `UIGenProgressBar.cs:57-86`,
/// `UIHeader.cs:46-62`). The frame has already latched the progress
/// snapshot and run `draw_self_message` (the `SetProgress` feed + the
/// message overrides); this paints the bar and the header.
///
/// The drunk-world evil flicker (`UIGenProgressBar.cs:63-67`) and the drunk
/// random-digit message (`UIWorldLoad.cs:93-103`) roll `Main.rand` — both
/// deferred with the RNG-order rule (ANALYSIS "The loading screen"); the
/// bar reads the resolved `evil_crimson`/`random_evil` directly.
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &mut WorldLoadState,
    screen: glam::Vec2,
) {
    // `UIWorldLoad.Draw`'s null gate (`:70-73`): no snapshot, nothing draws.
    if !state.draws() {
        return;
    }
    // The message `DrawSelf` assembled (the bar feed already ran in
    // `draw_self_message`; re-run it here so the draw owns the header text
    // exactly as the C# `DrawSelf` does, `:91-92`).
    let text = state.draw_self_message(state.flags);
    // The bar's resolved geometry (`UIGenProgressBar.DrawSelf`, `:68-80`).
    let frame = state.bar.draw_frame(state.evil_crimson, state.random_evil);

    // `UIGenProgressBar.Recalculate` (`:42-49`): 612x70, HAlign 0.5 /
    // VAlign 0 (`UIWorldLoad.cs:27-28`), Top = bar_top. The parent is the
    // screen rect (`UIState`'s full-screen root).
    let bar_rect = uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(state.bar_top),
        Style::px(612.0),
        Style::px(70.0),
        0.5,
        0.0,
        0.0,
        0.0,
        screen.x,
        [0.0; 4],
    );
    draw_bar(r, lookups, bar_rect, frame);

    // `UIHeader` (`UIWorldLoad.cs:30-31`): CopyStyle the bar, Top -= 70;
    // the Text setter measures the Death font (`UIHeader.cs:25-27`) so the
    // header's own width is the text width, HAlign 0.5 centres it.
    let header_w = match &lookups.death_font {
        Some(f) => font_width(f, &text, 1.0),
        None => r.text_width(&text, 1.0),
    };
    let header_rect = uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(state.message_top),
        Style::px(header_w),
        Style::px(0.0),
        0.5,
        0.0,
        0.0,
        0.0,
        screen.x,
        [0.0; 4],
    );
    draw_header(
        r,
        lookups,
        header_rect,
        &text,
        state.flags.tenth_anniversary_world_gen && !state.flags.zenith_world,
    );
}

/// `UIGenProgressBar.DrawSelf`'s sprite half (`UIGenProgressBar.cs:70-84`):
/// the two `DrawFilling2` bars, then the outer frame + the lower strip.
fn draw_bar(r: &mut SpriteRenderer, lookups: &assets::Lookups, rect: [f32; 4], frame: GenBarFrame) {
    let (x, y) = (rect[0], rect[1]);
    // `:74-78` — the long bar: filled = the evil colour, separator =
    // Lerp(color, Black, 0.5), empty = (48,48,48). DrawFilling2 at
    // topLeft + (20,40), height 16, total 570.
    let overall = color::tint(packed_rgba(frame.overall_color));
    let overall_sep = color::tint(packed_rgba(lerp_color_packed(
        frame.overall_color,
        0xFF000000,
        0.5,
    )));
    draw_filling2(
        r,
        [x + 20.0, y + 40.0],
        16,
        frame.overall_width,
        LONG_BAR_WIDTH,
        overall,
        overall_sep,
        color::tint([48, 48, 48, 255]),
    );
    // `:79-80` — the small bar: fixed colour, empty = (33,33,33),
    // topLeft + (50,60), height 8, total 508.
    let current = color::tint(packed_rgba(frame.current_color));
    let current_sep = color::tint(packed_rgba(lerp_color_packed(
        frame.current_color,
        0xFF000000,
        0.5,
    )));
    draw_filling2(
        r,
        [x + 50.0, y + 60.0],
        8,
        frame.current_width,
        SMALL_BAR_WIDTH,
        current,
        current_sep,
        color::tint([33, 33, 33, 255]),
    );
    // `:81-84` — the outer frame at (rect.X - 8, rect.Y), the lower strip
    // at + (44,60). The frame pick: random ? Random : crimson ? Crimson :
    // Corrupt (`:83`).
    let outer_idx = if frame.overall_color == 4292696893 {
        2 // Random
    } else if frame.overall_color == 4286836223 {
        1 // Crimson
    } else {
        0 // Corrupt
    };
    if let Some(sheet) = lookups.worldgen_outer[outer_idx] {
        super::push_whole(
            r,
            lookups,
            &sheet,
            [x - 8.0, y, sheet.width as f32, sheet.height as f32],
            [1.0, 1.0, 1.0, 1.0],
        );
    }
    if let Some(sheet) = lookups.worldgen_outer[3] {
        super::push_whole(
            r,
            lookups,
            &sheet,
            [
                x - 8.0 + 44.0,
                y + 60.0,
                sheet.width as f32,
                sheet.height as f32,
            ],
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

/// `DrawFilling2` (`UIGenProgressBar.cs:116-125`): three `MagicPixel`
/// rects — the filled span, the empty remainder, the 2 px separator at the
/// boundary. The even round-down already ran in `draw_frame`.
#[allow(clippy::too_many_arguments)]
fn draw_filling2(
    r: &mut SpriteRenderer,
    top_left: [f32; 2],
    height: i32,
    completed_width: i32,
    total_width: i32,
    filled: [f32; 4],
    separator: [f32; 4],
    empty: [f32; 4],
) {
    let (x, y) = (top_left[0], top_left[1]);
    let h = height as f32;
    // `:122` — the filled span.
    r.push_ui_rect([x, y], [completed_width as f32, h], filled);
    // `:123` — the empty remainder.
    r.push_ui_rect(
        [x + completed_width as f32, y],
        [(total_width - completed_width) as f32, h],
        empty,
    );
    // `:124` — the 2 px separator at the boundary.
    r.push_ui_rect([x + completed_width as f32 - 2.0, y], [2.0, h], separator);
}

/// `UIHeader.DrawSelf` (`UIHeader.cs:46-62`): the four ±1.2 px black corner
/// passes, then the colour on top — White, or HotPink on the
/// tenth-anniversary seed when not zenith (`:54-57`). The Death font, or
/// the bitmap fallback.
fn draw_header(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    text: &str,
    hot_pink: bool,
) {
    if text.is_empty() {
        return;
    }
    let (x, y) = (rect[0], rect[1]);
    let num = 1.2f32;
    let black = [0.0, 0.0, 0.0, 1.0];
    // `Color.HotPink` = (255, 105, 180, 255) (XNA's named colour).
    let top = if hot_pink {
        color::tint([255, 105, 180, 255])
    } else {
        [1.0, 1.0, 1.0, 1.0]
    };
    let push = |r: &mut SpriteRenderer, px: f32, py: f32, c: [f32; 4]| match &lookups.death_font {
        Some(f) => push_font_text(r, f, px, py, 1.0, c, text),
        None => r.push_text(px, py, 1.0, c, text),
    };
    // `:50-53` — the four corners.
    push(r, x - num, y - num, black);
    push(r, x + num, y - num, black);
    push(r, x - num, y + num, black);
    push(r, x + num, y + num, black);
    // `:56`/`:60` — the colour pass.
    push(r, x, y, top);
}

/// XNA `Color.PackedValue` is ABGR (`0xAABBGGRR`); the renderer tint wants
/// RGBA bytes. Unpack the four bytes in order.
fn packed_rgba(packed: u32) -> [u8; 4] {
    [
        (packed & 0xFF) as u8,
        ((packed >> 8) & 0xFF) as u8,
        ((packed >> 16) & 0xFF) as u8,
        ((packed >> 24) & 0xFF) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(total: f64, value: f64, message: &str) -> GenerationProgress {
        GenerationProgress {
            total_progress: total,
            value,
            message: message.to_string(),
        }
    }

    /// `SetProgress` (`:51-55`) moves targets only; the visual pair follows
    /// on the draw copy (`:68-69`) — the 1.4.5.8 "smoothing" is a verbatim
    /// assignment, pinned here so a later lerp cannot sneak in unnoticed.
    #[test]
    fn set_progress_moves_targets_and_draw_copies_verbatim() {
        let mut bar = GenProgressBar::default();
        bar.set_progress(0.25, 0.5);
        assert_eq!(bar.target_overall_progress, 0.25);
        assert_eq!(bar.target_current_progress, 0.5);
        assert_eq!(bar.visual_overall_progress, 0.0);
        assert_eq!(bar.visual_current_progress, 0.0);
        let frame = bar.draw_frame(false, false);
        assert_eq!(bar.visual_overall_progress, 0.25);
        assert_eq!(bar.visual_current_progress, 0.5);
        // `(int)(0.25 * 570) = 142` (even), `(int)(0.5 * 508) = 254`.
        assert_eq!(frame.overall_width, 142);
        assert_eq!(frame.current_width, 254);
    }

    /// `:71-72` truncation then `DrawFilling2`'s odd round-down
    /// (`:118-121`): `(int)(0.333 * 570) = 189 -> 188`,
    /// `(int)(0.333 * 508) = 169 -> 168`.
    #[test]
    fn widths_truncate_then_round_down_to_even() {
        let mut bar = GenProgressBar::default();
        bar.set_progress(0.333, 0.333);
        let frame = bar.draw_frame(false, false);
        assert_eq!(frame.overall_width, 188);
        assert_eq!(frame.current_width, 168);
        assert_eq!(filling_even_width(0), 0);
        assert_eq!(filling_even_width(1), 0);
        assert_eq!(filling_even_width(2), 2);
        assert_eq!(filling_even_width(571), 570);
    }

    /// Full progress fills the exact bar widths (`:71-72` at 1.0).
    #[test]
    fn full_progress_fills_the_bars() {
        let mut bar = GenProgressBar::default();
        bar.set_progress(1.0, 1.0);
        let frame = bar.draw_frame(false, false);
        assert_eq!(frame.overall_width, LONG_BAR_WIDTH);
        assert_eq!(frame.current_width, SMALL_BAR_WIDTH);
    }

    /// The packed-colour selection (`:74-79`): random evil wins over
    /// crimson; the small bar is fixed.
    #[test]
    fn bar_colors_match_the_packed_constants() {
        let mut bar = GenProgressBar::default();
        let corrupt = bar.draw_frame(false, false);
        assert_eq!(corrupt.overall_color, 4283888223);
        let crimson = bar.draw_frame(true, false);
        assert_eq!(crimson.overall_color, 4286836223);
        let random = bar.draw_frame(true, true);
        assert_eq!(random.overall_color, 4292696893);
        assert_eq!(random.current_color, 4290947159);
        // Color.Lerp(color, Color.Black, 0.5f) (`:78`, `:80`): each RGB
        // byte halves; Color.Black is (0,0,0,255) so alpha stays 255.
        let half = lerp_color_packed(corrupt.overall_color, 0xFF000000, 0.5);
        assert_eq!(
            half & 0xFF,
            ((corrupt.overall_color & 0xFF) as f32 * 0.5) as u32
        );
        assert_eq!((half >> 24) & 0xFF, 255);
    }

    /// The ctor + Update layout (`:26`, `:31`, `:60-61`): 270 at <=600 px,
    /// 370 at >=700 px, the clamped midpoint 320 at 650; the message header
    /// always rides 70 px above.
    #[test]
    fn layout_lerps_270_to_370_over_screen_height() {
        let mut state = WorldLoadState::new();
        assert_eq!(state.bar_top, 270.0);
        assert_eq!(state.message_top, 200.0);
        state.update_layout(500.0);
        assert_eq!(state.bar_top, 270.0);
        state.update_layout(650.0);
        assert_eq!(state.bar_top, 320.0);
        assert_eq!(state.message_top, 250.0);
        state.update_layout(800.0);
        assert_eq!(state.bar_top, 370.0);
        assert_eq!(state.message_top, 300.0);
    }

    /// The Inventory-abort arm (`:54-58`): queues once, the sound fires once
    /// (the return value), a held trigger does not re-queue.
    #[test]
    fn inventory_trigger_queues_abort_once() {
        let mut state = WorldLoadState::new();
        assert!(!state.queue_abort_on_inventory(false));
        assert!(!state.queued_abort);
        assert!(state.queue_abort_on_inventory(true));
        assert!(state.queued_abort);
        assert!(!state.queue_abort_on_inventory(true));
    }

    /// `Draw`'s null gate (`:69-73`) and `DrawSelf`'s zero defaults
    /// (`:78-80`): no snapshot -> nothing draws, the bar still receives
    /// (0, 0) when DrawSelf runs.
    #[test]
    fn null_progress_hides_the_screen_and_zeroes_the_bar() {
        let mut state = WorldLoadState::new();
        assert!(!state.draws());
        let text = state.draw_self_message(WorldLoadFlags::default());
        assert_eq!(text, "");
        assert_eq!(state.bar.target_overall_progress, 0.0);
        assert_eq!(state.bar.target_current_progress, 0.0);
        state.set_progress(Some(progress(0.5, 0.25, "Generating")));
        assert!(state.draws());
    }

    /// `DrawSelf`'s feed + message (`:83-92`): the snapshot's floats reach
    /// the bar, the message is the pass label.
    #[test]
    fn draw_self_feeds_the_bar_and_message() {
        let mut state = WorldLoadState::new();
        state.set_progress(Some(progress(0.5, 0.25, "Generating structures")));
        let text = state.draw_self_message(WorldLoadFlags::default());
        assert_eq!(text, "Generating structures");
        assert_eq!(state.bar.target_overall_progress, 0.5);
        assert_eq!(state.bar.target_current_progress, 0.25);
    }

    /// The `UI.Canceling` override (`:86-89`) — needs lang init like the
    /// other localization tests (`world_creation.rs:1430`).
    #[test]
    fn queued_abort_overrides_the_message() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut state = WorldLoadState::new();
        state.set_progress(Some(progress(0.5, 0.25, "Generating")));
        state.queue_abort_on_inventory(true);
        let text = state.draw_self_message(WorldLoadFlags::default());
        assert_eq!(text, crate::lang::text("UI.Canceling"));
        assert!(!text.is_empty());
    }

    /// The bees override (`:104-107`) and its zenith gate; the get-good
    /// reversal (`:108-116`) and its noTraps+placingTraps gate.
    #[test]
    fn secret_seed_message_overrides() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut state = WorldLoadState::new();
        state.set_progress(Some(progress(0.0, 0.0, "abc")));
        let bees = WorldLoadFlags {
            not_the_bees: true,
            ..Default::default()
        };
        assert_eq!(
            state.draw_self_message(bees),
            crate::lang::text("UI.WorldGenEasterEgg_GeneratingBees")
        );
        let bees_zenith = WorldLoadFlags {
            zenith_world: true,
            ..bees
        };
        assert_eq!(state.draw_self_message(bees_zenith), "abc");
        let good = WorldLoadFlags {
            get_good_world_gen: true,
            ..Default::default()
        };
        assert_eq!(state.draw_self_message(good), "cba");
        // noTraps && placingTraps -> no reversal (`:108`).
        let good_traps = WorldLoadFlags {
            no_traps_world_gen: true,
            placing_traps: true,
            ..good
        };
        assert_eq!(state.draw_self_message(good_traps), "abc");
        // noTraps but not placing traps -> reversal returns (`:108`).
        let good_no_traps = WorldLoadFlags {
            placing_traps: false,
            ..good_traps
        };
        assert_eq!(state.draw_self_message(good_no_traps), "cba");
    }

    /// `GetStatusText` (`:129-136`): both arms, tenth-percent formatting.
    #[test]
    fn status_text_formats_tenth_percents() {
        let mut state = WorldLoadState::new();
        assert_eq!(state.status_text(), "0.0% - ... - 0.0%");
        state.set_progress(Some(progress(0.5, 0.1234, "Resetting")));
        assert_eq!(state.status_text(), "50.0% - Resetting - 12.3%");
        assert_eq!(format_percent_tenth(0.0), "0.0%");
        assert_eq!(format_percent_tenth(1.0), "100.0%");
        assert_eq!(format_percent_tenth(0.1236), "12.4%");
        assert_eq!(format_percent_tenth(0.1234), "12.3%");
        // The exactly-representable tie: 0.0625 * 100 = 6.25. .NET's "0.0%"
        // rounds half-away-from-zero ("6.3%"); Rust's {:.1} half-to-even
        // would print "6.2%" — the shared tr_core formatter must win.
        assert_eq!(format_percent_tenth(0.0625), "6.3%");
    }
}
