//! The town-chatter UI half (S1-E-10b): the hover bubble
//! (`Main.DrawNPCChatBubble`, `Main.cs:43135-43161`) and the dialogue panel
//! (`NPCChatPanel.cs` — `Draw` `:52-86`, `DrawText` `:215-263`,
//! `DrawButtons` `:88-162`, `Close` `:265-274`).
//!
//! The selection ladder (`crate::npc_chat::get_chat`) and the right-click
//! open (`crate::world_sim::npcs::talk_to_npc`, `Main.cs:43069-43083`)
//! landed with S1-E-10a; this module draws their result.
//!
//! Deferred (documented in docs/ANALYSIS.md "Town chatter UI (S1-E-10b)"):
//! the interaction buttons other than Close (`NPCInteractions.All`,
//! `NPCInteractions.cs:486-531` — shop/happiness/housing/reforge/… ride
//! systems the panel task does not own), the corner item
//! (`Main.DrawNPCChatBottomRightItem`, `:39789`), the sign-editing blinker
//! (`NPCChatPanel.cs:236-262`), the Dryad Stardew override (`:195-213`)
//! and the gamepad/virtual-keyboard arms. The NPC portrait's Detailed
//! arm landed with the W4-g58 retire (`ui::chat_portrait`, the `:79`
//! call site); its hop math lives below (`portrait_hop_lift`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::{text, text_wrap, UiPulse};
use crate::assets::Lookups;
use crate::npc::Npc;
use crate::npc_chat::{chat_box_height, chat_box_rect};

/// `TextDisplayCache.PrepareCache` (`TextDisplayCache.cs:19-30`):
/// `Utils.WordwrapString(text, FontAssets.MouseText.Value, 460, 10, …)`
/// (`Utils.cs:860-867`) — the ReLogic `WrappedTextBuilder` wrap at 460 px,
/// capped at 10 lines, ported 1:1 in `ui::text_wrap` (W4-d121, the
/// research/37 gap). The port measures with the retail `Mouse_Text` sheet
/// when loaded; the bitmap fallback estimates 8 px per char
/// (docs/ANALYSIS.md "Town chatter UI (S1-E-10b)"). The selection half's
/// `format_chat_lines` keeps its char-count wrap for the tests.
pub(crate) fn panel_lines(lookups: &Lookups, chat_text: &str) -> Vec<String> {
    const MAX_LINES: usize = 10; // `TextDisplayCache.cs:27`'s maxLines
    let max_w = crate::npc_chat::CHAT_TEXT_WRAP_WIDTH as f32;
    text_wrap::wordwrap_string(
        lookups.ui_font.as_ref(),
        chat_text,
        max_w,
        MAX_LINES,
        crate::lang::active_culture(),
    )
}

/// `_neededInteractionLines = (int)Math.Ceiling(count / 4f)`
/// (`PrepareInteractions`, `NPCChatPanel.cs:174`,
/// `AllowedInteractionsPerLine = 4` `:30`) — the integer ceil the panel
/// height (`:68`) and the button-row origin (`:93`) both read. The
/// W4-d120 dynamic-lines fix (research/143 §1's 2-hunk patch, landed here
/// ahead of its publication — verified equivalent): the shop / happiness /
/// housing rows (g58/f38/d120's buttons) push the count past 4 at merge,
/// so the line count grows with the label list instead of pinning 1.
/// Zero interactions → 0 (`Ceiling(0f)`).
pub(crate) fn interaction_lines(count: usize) -> usize {
    count.div_ceil(4) // `Math.Ceiling(count / 4f)` (`:174`)
}

/// The live interaction count (`PrepareInteractions`' Condition filter,
/// `NPCChatPanel.cs:164-175`): index 0 is always CloseChat
/// (`NPCInteractions.cs:271-287`, `Condition() => true`), index 1 the
/// sign's `OpenSign` (`:15-42`, `Condition() => LocalPlayer.sign > -1`,
/// registered AFTER CloseChat at `:515-516`). The shop / happiness /
/// housing / … buttons are deferred (module header); the merge lane
/// grows this count alongside [`interactions`].
pub(crate) fn interaction_count(sign_open: bool) -> usize {
    1 + usize::from(sign_open)
}

/// The live interaction list (`PrepareInteractions`, `NPCChatPanel.cs:
/// 164-175`): index 0 is always CloseChat, index 1 the sign's
/// Close/Edit-Save toggle while a sign is open. The labels ride
/// `Lang.inter[52]` (Close) and `Lang.inter[47]`/`[48]`
/// (`OpenSign.GetText`, `NPCInteractions.cs:22-29` — 47 while
/// `Main.editSign`, 48 otherwise).
pub(crate) fn interactions(sign_open: bool, edit_sign: bool) -> Vec<String> {
    let mut rows = vec![crate::lang::inter(52)];
    if sign_open {
        rows.push(crate::lang::inter(if edit_sign { 47 } else { 48 }));
    }
    rows
}

/// The interaction rows the port renders (`NPCInteractions.Initialize`
/// registration order, `NPCInteractions.cs:513-530`): `CloseChat` (`:515`)
/// always, then `ReportHappiness` (`:529`) when its Condition holds
/// (`:291-298`: `!NPC.CanShowHomelessText(talkNPC)` AND
/// `currentShoppingSettings.HappinessReport != ""`). The other buttons
/// (shop/housing/reforge/…) stay deferred (module header).
/// `_neededInteractionLines = ceil(count / 4)` (`NPCChatPanel.cs:174`,
/// `AllowedInteractionsPerLine = 4` `:30`) — 1 line for either count.
fn happiness_interaction_lines(happiness: bool) -> usize {
    let count = 1 + usize::from(happiness);
    count.div_ceil(4)
}

/// The Happiness button's label (`ReportHappiness.GetText`,
/// `NPCInteractions.cs:300-303`): `Language.GetTextValue("UI.NPCCheckHappiness")`.
pub(crate) fn happiness_label() -> String {
    crate::lang::text("UI.NPCCheckHappiness")
}

/// The panel's box (`NPCChatPanel.cs:65-68` via the selection half's
/// `chat_box_rect`/`chat_box_height`): `[x, y, w, h]` in UI pixels.
/// `interaction_count` is the live label count ([`interaction_count`]) —
/// the height carries `ceil(count/4)` interaction lines (`:68`, `:174`).
pub(crate) fn panel_rect(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    interaction_count: usize,
) -> [f32; 4] {
    let base = chat_box_rect(screen.x as i32, screen.y as i32);
    let lines = panel_lines(lookups, chat_text).len();
    let h = chat_box_height(lines, interaction_lines(interaction_count));
    [base.x as f32, base.y as f32, base.w as f32, h as f32]
}

/// The Close button's hit rect (`NPCChatPanel.cs:93-117`): the row origin
/// is `panelArea.BottomLeft() + (30, -22·lines + max(0, 2·(lines−1)) − 4)`,
/// the rect 22 tall and as wide as the label measured at scale 0.9
/// (shrunk to fit 260 px, `:113-117`). The Close-only convenience over
/// [`interaction_rects`] (one live interaction — the talk case); the
/// sign-aware sites pass the full row.
pub(crate) fn close_button_rect(lookups: &Lookups, screen: Vec2, chat_text: &str) -> [f32; 4] {
    interaction_rects(lookups, screen, chat_text, &[crate::lang::inter(52)])[0]
}

/// The full interaction row's rects (`DrawButtons`' per-button walk,
/// `NPCChatPanel.cs:93-135`): one rect per live label, the x advancing by
/// each measured width + 30 (`:135`), the `num % 4 == 0` row reset
/// (`:103-107`) which also steps y by `num / 4 * 22` (`:106` — the
/// W4-d120 compose note's y-step, landed with the dynamic line count).
/// The sign row joins while a sign is open ([`interactions`]).
pub(crate) fn interaction_rects(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    labels: &[String],
) -> Vec<[f32; 4]> {
    let panel = panel_rect(lookups, screen, chat_text, labels.len());
    let il = interaction_lines(labels.len()) as f32;
    let base_x = panel[0] + 30.0;
    let base_y = panel[1] + panel[3] - 22.0 * il + (2.0 * (il - 1.0)).max(0.0) - 4.0;
    let mut x = base_x;
    let mut y = base_y;
    let mut out = Vec::with_capacity(labels.len());
    for (i, label) in labels.iter().enumerate() {
        if i % 4 == 0 {
            x = base_x; // `:105`
            y = base_y + (i / 4) as f32 * 22.0; // `:106`
        }
        let w0 = text::ui_font_width(lookups, label, 0.9)
            .unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
        let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
        let w = (w0 * shrink).trunc(); // `:117`
        out.push([x, y, w, 22.0]);
        x += w + 30.0; // `:135`
    }
    out
}

/// The panel box in the JFL/g58 happiness lane.
pub(crate) fn happiness_panel_rect(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    happiness: bool,
) -> [f32; 4] {
    let base = chat_box_rect(screen.x as i32, screen.y as i32);
    let lines = panel_lines(lookups, chat_text).len();
    let h = chat_box_height(lines, happiness_interaction_lines(happiness));
    [base.x as f32, base.y as f32, base.w as f32, h as f32]
}

/// One interaction button's hit rect (`NPCChatPanel.cs:93-117`): the row
/// origin is `panelArea.BottomLeft() + (30, -22·lines + max(0, 2·(lines−1))
/// − 4)`, each rect 22 tall and as wide as its label measured at scale 0.9
/// (shrunk to fit 260 px, `:113-117`); the next button starts at
/// `x + width + 30` (`:135`), wrapping every 4 (`:103-107`).
fn button_rect(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    happiness: bool,
    index: usize,
) -> [f32; 4] {
    let panel = happiness_panel_rect(lookups, screen, chat_text, happiness);
    let il = happiness_interaction_lines(happiness) as f32;
    let row_x = panel[0] + 30.0;
    let row_y = panel[1] + panel[3] - 22.0 * il + (2.0 * (il - 1.0)).max(0.0) - 4.0;
    let labels = [crate::lang::inter(52), happiness_label()];
    let count = 1 + usize::from(happiness);
    let mut x = row_x;
    let mut y = row_y;
    let mut rect = [x, y, 100.0, 22.0];
    for (i, label) in labels.iter().enumerate().take(count) {
        if i % 4 == 0 {
            x = row_x;
            y = (i / 4) as f32 * 22.0 + row_y;
        }
        let w0 =
            text::ui_font_width(lookups, label, 0.9).unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
        let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
        // `:117` — `rectangle.Width = (int)(stringSize.X * num4)`.
        rect = [x, y, (w0 * shrink).trunc(), 22.0];
        if i == index {
            return rect;
        }
        x += rect[2] + 30.0; // `:135`
    }
    rect
}

/// The Close button's hit rect — interaction index 0.
/// Happiness-style Close button rect (JFL/g58).
pub(crate) fn happiness_close_button_rect(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    happiness: bool,
) -> [f32; 4] {
    button_rect(lookups, screen, chat_text, happiness, 0)
}

pub(crate) fn happiness_button_rect(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    happiness: bool,
) -> Option<[f32; 4]> {
    happiness.then(|| button_rect(lookups, screen, chat_text, happiness, 1))
}

/// W4-f30 — the interaction row's button rects (`NPCChatPanel.DrawButtons`,
/// `NPCChatPanel.cs:93-135`): the row origin is the Close rect's origin; the
/// buttons lay left-to-right, each `width + 30` apart (`:135`). When the
/// Dryad's Stardew bit shows (`stardew` true), the "Purify" button is index 0
/// and Close index 1; otherwise Close is index 0. Returns
/// `(stardew_rect, close_rect)` — the stardew rect is zero-width when absent.
#[allow(dead_code)] // the f30 Purify+Close pair; `interaction_rects` lays the live row
pub(crate) fn button_rects(
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    stardew: bool,
) -> ([f32; 4], [f32; 4]) {
    let close = close_button_rect(lookups, screen, chat_text);
    if !stardew {
        return ([0.0, 0.0, 0.0, 0.0], close);
    }
    let label = crate::dryad_stardew::give_cola_button_text();
    let w0 = text::ui_font_width(lookups, &label, 0.9)
        .unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
    let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
    let sw = (w0 * shrink).trunc();
    // The Purify button sits at the row origin; Close shifts right by its
    // width + 30 (`:135` — `rectangle.X += rectangle.Width + 30`).
    let purify = [close[0], close[1], sw, 22.0];
    let close_shifted = [close[0] + sw + 30.0, close[1], close[2], close[3]];
    (purify, close_shifted)
}

/// One crop of a sheet pushed at 1:1 — the panel's `ChatBack` splices draw
/// SOURCE rects (`NPCChatPanel.cs:71-72`), not stretched wholes.
fn push_crop(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sheet: &tr_render::SheetRect,
    src: [f32; 4],
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let uv = [
        (sheet.x as f32 + src[0]) / aw,
        (sheet.y as f32 + src[1]) / ah,
        (sheet.x as f32 + src[0] + src[2]) / aw,
        (sheet.y as f32 + src[1] + src[3]) / ah,
    ];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

/// `NPCChatPanel.Draw` (`NPCChatPanel.cs:52-86`) — the text-only port:
/// the two `ChatBack` splices at `Color(200,200,200,200)` (`:62`, `:71-72`),
/// the wrapped lines at `+(20,20)` stepping 30 px (`DrawText` `:217-235`)
/// in `Color(num,num,num,num)` with `num = (mouseTextColor·2 + 255)/3`
/// (`:63-64`), the sign-edit caret (`DrawText` `:236-262`, W4-f22) and the
/// interaction row (`DrawButtons` `:88-162` — Close plus the sign's
/// Edit/Save toggle). `sign_open`/`edit_sign` are `Player.sign >= 0` /
/// `Main.editSign`; `blinker` is the panel's own
/// `textBlinkerCount/State` (`:254-258`). Returns whether the pointer is
/// inside the panel — the C#'s `LocalPlayer.mouseInterface = true` arm
/// (`:81-84`); the port has no mouseInterface field yet, so the caller
/// records the flag (deferred).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_panel(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    happiness: bool,
    pulse: UiPulse,
    mouse: Vec2,
    sign_open: bool,
    edit_sign: bool,
    blinker: &mut super::chest::TextBlinker,
    stardew: bool,
) -> bool {
    let lines = panel_lines(lookups, chat_text);
    let panel = panel_rect(lookups, screen, chat_text, interaction_count(sign_open));
    let (x, y, _w, h) = (panel[0], panel[1], panel[2], panel[3]);
    // `:71-72` — the top splice (source `(0, 0, ChatBack.Width(), h−30)`)
    // and the bottom 30 px (`(0, Height−30, Width, 30)`), both at scale 1:
    // the drawn width is the TEXTURE's width, not the 500 px box's.
    if let Some(sheet) = lookups.ui_chat_back {
        let tint = [200.0 / 255.0; 4];
        let sw = sheet.width as f32;
        let top_h = (h - 30.0).max(0.0);
        push_crop(r, lookups, &sheet, [0.0, 0.0, sw, top_h], [x, y, sw, top_h], tint);
        push_crop(
            r,
            lookups,
            &sheet,
            [0.0, sheet.height as f32 - 30.0, sw, 30.0],
            [x, y + h - 30.0, sw, 30.0],
            tint,
        );
    }
    // `DrawText` (`:215-235`): `allowRichText` is true while talking
    // (`:38`), so the lines ride `DrawColorCodedStringWithShadow` (`:228`)
    // — the black four-way shadow at spread 2 (`ChatManager.cs:246-249`,
    // `:376-382`), then the colour.
    let m = (pulse.text * 255.0) as i32;
    let num = (m * 2 + 255) / 3; // `:63`
    let text_tint = [num as f32 / 255.0; 4];
    const SHADOW_RING: [[f32; 2]; 4] = [[2.0, 0.0], [-2.0, 0.0], [0.0, 2.0], [0.0, -2.0]];
    for (i, line) in lines.iter().enumerate() {
        text::draw_border_string(
            r,
            lookups,
            text::BorderFont::Ui,
            x + 20.0,
            y + 20.0 + i as f32 * 30.0,
            1.0,
            &SHADOW_RING,
            text_tint,
            line,
        );
    }
    // The sign-edit caret (`DrawText` `:236-262`, W4-f22): only while
    // `Main.editSign` and the last line exists; the x rides the last
    // line's measured width, the blinker is `++count >= 20` toggle
    // (`:254-258` — the same 20-frame half-period as the chest editor's),
    // the "|" draws at state 1 (`:259-261`). The IME composition half
    // (`:242-253`) has no port route.
    if edit_sign && !lines.is_empty() {
        let last = &lines[lines.len() - 1];
        let last_w = text::ui_font_width(lookups, last, 1.0)
            .unwrap_or(last.chars().count() as f32 * 8.0);
        let cx = x + 20.0 + last_w;
        let cy = y + 20.0 + (lines.len() - 1) as f32 * 30.0;
        blinker.advance();
        if blinker.on {
            text::draw_border_string(
                r,
                lookups,
                text::BorderFont::Ui,
                cx,
                cy,
                1.0,
                &SHADOW_RING,
                text_tint,
                "|",
            );
        }
    }
    draw_interactions(
        r,
        lookups,
        screen,
        chat_text,
        pulse,
        mouse,
        sign_open,
        edit_sign,
        stardew,
    );
    super::in_rect(mouse, panel)
}

/// The interaction row (`DrawButtons`, `NPCChatPanel.cs:88-162`): every
/// live label at its [`interaction_rects`] rect — idle scale 0.9 / hover
/// 1.2 (`:96`, `:119`), the label colour `(b, b/1.1, b/2, b)` (`:102`),
/// the shadow Black idle / Brown hovered (`:121`), the origin
/// `(0, stringSize.Y·0.5)` at the rect's left/centre-Y (`:120-122`) with
/// the hover x-shift `−(1.2−0.9)·width·0.5` (`:125`).
#[allow(clippy::too_many_arguments)]
fn draw_interactions(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    pulse: UiPulse,
    mouse: Vec2,
    sign_open: bool,
    edit_sign: bool,
    stardew: bool,
) {
    // W4-f30 + W4-f22 union: ONE interaction row. The Dryad Purify button
    // (index 0) joins while the Stardew bit shows and shifts Close right by
    // its width + 30 (`NPCChatPanel.cs:135`); the sign's Edit/Save toggle
    // follows Close. `interaction_rects` lays the row out left-to-right.
    let mut labels = interactions(sign_open, edit_sign);
    if stardew {
        labels.insert(0, crate::dryad_stardew::give_cola_button_text());
    }
    let rects = interaction_rects(lookups, screen, chat_text, &labels);
    for (label, rect) in labels.iter().zip(rects.iter()) {
        draw_interaction(r, lookups, *rect, label, pulse, mouse);
    }
}

/// One interaction button (`DrawButtons`, `NPCChatPanel.cs:88-162`): idle
/// scale 0.9 / hover 1.2 (`:96`, `:119`), the label colour
/// `(b, b/1.1, b/2, b)` (`:102`), the shadow Black idle / Brown hovered
/// (`:121`), the origin `(0, stringSize.Y·0.5)` at the rect's
/// left/centre-Y (`:120-122`) with the hover x-shift
/// `−(1.2−0.9)·width·0.5` (`:125`).
/// The Happiness button's draw (JFL/g58).
fn happiness_interaction_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    happiness: bool,
    index: usize,
    pulse: UiPulse,
    mouse: Vec2,
) {
    let rect = button_rect(lookups, screen, chat_text, happiness, index);
    let label = if index == 0 {
        crate::lang::inter(52)
    } else {
        happiness_label()
    };
    // The shared border/measure helpers take `&str`; keep the owned label
    // alive for the whole draw.
    let label = label.as_str();
    let hovered = super::in_rect(mouse, rect);
    let b = (pulse.text * 255.0) as i32;
    let chat_color = [
        b as f32 / 255.0,
        (b as f64 / 1.1) as i32 as f32 / 255.0, // `:102` тАФ `(int)(b/1.1)`
        (b / 2) as f32 / 255.0,
        b as f32 / 255.0,
    ];
    // `Color.Brown` = (165, 42, 42) тАФ the hovered shadow (`:121`).
    let shadow = if hovered {
        [165.0 / 255.0, 42.0 / 255.0, 42.0 / 255.0, 1.0]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    };
    let scale = if hovered { 1.2 } else { 0.9 };
    let w0 = text::ui_font_width(lookups, label, 0.9).unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
    let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
    let mut px = rect[0];
    if hovered {
        // `:125` тАФ `(int)((1.2 тИТ 0.9) ┬╖ rectangle.Width ┬╖ 0.5)`.
        px -= ((1.2 - 0.9) * rect[2] * 0.5).floor();
    }
    // `origin = (0, stringSize.Y ┬╖ 0.5)` (`:120`) where stringSize is
    // measured at 0.9 (`:112`); XNA applies origin ├Ч draw scale, so the
    // top-left y is centre-Y тИТ h(0.9)┬╖0.5┬╖(scale┬╖shrink).
    let lh = text::ui_font_height(lookups, 0.9).unwrap_or(8.0 * 0.9);
    let py = rect[1] + rect[3] * 0.5 - lh * 0.5 * scale * shrink;
    // `ChatManager.DrawColorCodedStringShadow` (`:132`): four passes at
    // `ShadowDirections ┬╖ spread` тАФ ┬▒X/┬▒Y ├Ч 2 (`ChatManager.cs:200-205`,
    // `:376-382`) тАФ in Brown hovered / Black idle (`:121`), then the
    // colour pass (`DrawColorCodedString`, `:133`).
    for [dx, dy] in [[2.0, 0.0], [-2.0, 0.0], [0.0, 2.0], [0.0, -2.0]] {
        text::draw_border_string(
            r,
            lookups,
            text::BorderFont::Ui,
            px + dx,
            py + dy,
            scale * shrink,
            &[],
            shadow,
            label,
        );
    }
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        px,
        py,
        scale * shrink,
        &[],
        chat_color,
        label,
    );
}

/// One interaction button's draw — the `DrawButtons` loop body
/// (`NPCChatPanel.cs:108-135`).
/// One interaction button's draw — the `DrawButtons` loop body (f22).
fn draw_interaction(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    label: &str,
    pulse: UiPulse,
    mouse: Vec2,
) {
    let hovered = super::in_rect(mouse, rect);
    let b = (pulse.text * 255.0) as i32;
    let chat_color = [
        b as f32 / 255.0,
        (b as f64 / 1.1) as i32 as f32 / 255.0, // `:102` тАФ `(int)(b/1.1)`
        (b / 2) as f32 / 255.0,
        b as f32 / 255.0,
    ];
    // `Color.Brown` = (165, 42, 42) тАФ the hovered shadow (`:121`).
    let shadow = if hovered {
        [165.0 / 255.0, 42.0 / 255.0, 42.0 / 255.0, 1.0]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    };
    let scale = if hovered { 1.2 } else { 0.9 };
    let w0 = text::ui_font_width(lookups, label, 0.9).unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
    let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
    let mut px = rect[0];
    if hovered {
        // `:125` тАФ `(int)((1.2 тИТ 0.9) ┬╖ rectangle.Width ┬╖ 0.5)`.
        px -= ((1.2 - 0.9) * rect[2] * 0.5).floor();
    }
    // `origin = (0, stringSize.Y ┬╖ 0.5)` (`:120`) where stringSize is
    // measured at 0.9 (`:112`); XNA applies origin ├Ч draw scale, so the
    // top-left y is centre-Y тИТ h(0.9)┬╖0.5┬╖(scale┬╖shrink).
    let lh = text::ui_font_height(lookups, 0.9).unwrap_or(8.0 * 0.9);
    let py = rect[1] + rect[3] * 0.5 - lh * 0.5 * scale * shrink;
    // `ChatManager.DrawColorCodedStringShadow` (`:132`): four passes at
    // `ShadowDirections ┬╖ spread` тАФ ┬▒X/┬▒Y ├Ч 2 (`ChatManager.cs:200-205`,
    // `:376-382`) тАФ in Brown hovered / Black idle (`:121`), then the
    // colour pass (`DrawColorCodedString`, `:133`).
    for [dx, dy] in [[2.0, 0.0], [-2.0, 0.0], [0.0, 2.0], [0.0, -2.0]] {
        text::draw_border_string(
            r,
            lookups,
            text::BorderFont::Ui,
            px + dx,
            py + dy,
            scale * shrink,
            &[],
            shadow,
            label,
        );
    }
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        px,
        py,
        scale * shrink,
        &[],
        chat_color,
        label,
    );
}

/// `Utils.EaseOutBounce` (`Utils.cs:355-358`) → `BounceEaseOut(x, 4, 2.0)`
/// (`:360-363`): `1 − (1−t)² · |sin(t·4π)|` — the portrait hop's landing
/// curve. f64 throughout (the C# is `double`).
pub(crate) fn ease_out_bounce(x: f64) -> f64 {
    1.0 - (1.0 - x).powi(2) * (x * (4.0 * std::f64::consts::PI)).sin().abs()
}

/// `Main.DrawNPCPortrait`'s hop lift (`Main.cs:40335-40359`): the frame
/// counter's `num3 = num % 120`, `num4 = num / 120`; while `num4 == 0`
/// (the first 120 drawn frames after a `DoNPCPortraitHop` reset) the
/// portrait centre lifts by `(-56) * 0.25 * (1 − EaseOutBounce(
/// clamp(num3, 0, 80) / 80))` px (`:40358` — `num2 = 56`, the negative
/// y is UP). Past frame 119 the lift is 0 and stays 0 (the `num4 == 0`
/// gate — the hop does NOT repeat every 120 frames). The C# `if (0 != 0)`
/// wobble (`:40347-40354`) and `flag2 = false` sway (`:40360-40372`) are
/// dead code — dropped. The portrait DRAW itself is deferred (no portrait
/// system, S1-E-10b); this is the animation state the draw will consume.
pub(crate) fn portrait_hop_lift(frame_counter: i32) -> f32 {
    let num3 = (frame_counter % 120) as f64;
    let num4 = frame_counter / 120;
    if num4 != 0 {
        return 0.0;
    }
    let t = num3.clamp(0.0, 80.0) / 80.0;
    (-56.0f64 * 0.25 * (1.0 - ease_out_bounce(t))) as f32
}

/// `Main.DoNPCPortraitHop` (`Main.cs:40675-40678`): the counter resets to
/// 0 — the next drawn frame starts the hop. Called by the talk open
/// (`Player.SetTalkNPC`, `Player.cs:4397-4402` — the `NPCPortraits` lookup
/// gates it; every ported town kind has a portrait row, `NPCID.cs:5894+`)
/// and the chat actions (`ReportHappiness.Interact` `NPCInteractions.cs:
/// 310`, the nurse/tax/angler/guide/dryad/party-girl/tavernkeep texts).
pub(crate) fn do_npc_portrait_hop(app: &mut crate::app::App) {
    app.npc_chat_portrait_frame_counter = 0;
}

/// The hover pick behind `Main.instance.currentNPCShowingChatBubble`
/// (`Main.cs:43069`, set in `HoverOverNPCs` `:42990-43086`): the first
/// live town-like NPC whose box the pointer is over AND whose box
/// intersects the player's `TileReachCheckSettings.Simple` world region
/// (`:43049`), while the player lives (`:43065`). The C# hit box is the
/// NPC's frame rect (`:43003`); the port uses its `Npc::contains` box —
/// a recorded stand-in. The stinky / projectile-651 / mouseInterface
/// gates (`:43053-43064`) are deferred with the systems they read.
pub(crate) fn bubble_npc(
    npcs: &[Npc],
    player: Option<&crate::player::Player>,
    mouse_world: Vec2,
) -> Option<usize> {
    let p = player?;
    if p.hp <= 0 {
        return None;
    }
    // The same ±5/±3-tile region the talk arm reads
    // (`world_sim::npcs::chat_world_region`, `TileReachCheckSettings.cs:76-89`).
    let (lx, ly, hx, hy) = crate::world_sim::npcs::chat_world_region(p);
    npcs.iter().position(|n| {
        n.hp > 0
            && n.kind.is_town_npc_like()
            && n.contains(mouse_world.x, mouse_world.y)
            && {
                let (w, h) = n.size();
                n.x < hx && n.x + w > lx && n.y < hy && n.y + h > ly
            }
    })
}

/// `Main.DrawNPCChatBubble` (`Main.cs:43135-43161`), drawn from
/// `DrawInterface_1_1_DrawEmoteBubblesInWorld` (`:46460-46464`) under the
/// world→UI zoom transform — the same `(world − camera)·zoom + screen/2`
/// the emote bubbles ride (`ui::emotes::draw`).
///
/// `num = −(width/2 + 8)` (`:43137`, integer division), flipping to
/// `+(width/2 + 8)` with `SpriteEffects.FlipHorizontally` when
/// `spriteDirection == −1` (`:43144-43148`); the y is
/// `npc.y − Chat.Height()` (`:43138`), −18 more for the cat's ai[0]==5
/// sit (`:43139-43142`, type 637 — no port kind yet, the check rides the
/// net id). The gravity-flip arm (`:43149-43153`) is unreachable: the
/// port keeps `gravDir = 1` (player.rs). The tint is
/// `Color(mouseTextColor ×4)` (`:43158`).
pub(crate) fn draw_bubble(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    npcs: &[Npc],
    player: Option<&crate::player::Player>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    mouse: Vec2,
    pulse: UiPulse,
) {
    let Some(sheet) = lookups.ui_chat_bubble else {
        return;
    };
    // The UI pointer back into world pixels (the inverse of the emote
    // transform) — `Main.mouseX/Y` under the zoom matrix.
    let mouse_world = Vec2::new(
        (mouse.x - screen.x * 0.5) / zoom + camera.x,
        (mouse.y - screen.y * 0.5) / zoom + camera.y,
    );
    let Some(i) = bubble_npc(npcs, player, mouse_world) else {
        return;
    };
    let n = &npcs[i];
    let (w, _h) = n.size();
    let half_w = (w as i32) / 2; // `:43137`'s integer `width / 2`
    let chat_w = sheet.width as f32;
    let chat_h = sheet.height as f32;
    let mut num = -(half_w + 8);
    let mut y_world = n.y - chat_h;
    if n.kind.net_id() == 637 && n.ai[0] == 5.0 {
        y_world -= 18.0; // `:43139-43142`
    }
    let flip = n.sprite_dir == -1;
    if flip {
        num = half_w + 8; // `:43147`
    }
    // `:43154` — `(float)(TextureAssets.Chat.Width() / 2)`, integer division.
    let x_world = n.x + half_w as f32 - (sheet.width / 2) as f32 - num as f32;
    let pos = [
        (x_world - camera.x) * zoom + screen.x * 0.5,
        (y_world - camera.y) * zoom + screen.y * 0.5,
    ];
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let mut uv = [
        sheet.x as f32 / aw,
        sheet.y as f32 / ah,
        (sheet.x + sheet.width) as f32 / aw,
        (sheet.y + sheet.height) as f32 / ah,
    ];
    if flip {
        uv = crate::render::player_layers::flip_uv(uv, true);
    }
    let m = pulse.text;
    r.push_ui_sprite(pos, [chat_w * zoom, chat_h * zoom], uv, [m; 4]);
}

/// `Main.DrawInterface_19_SignTileBubble` (`Main.cs:45739-45760`, the
/// layer at `:44804-44808` — `InterfaceScaleType.Game`, the world zoom
/// matrix): the placed-sign bubble while `Main.signBubble` holds.
/// `num/num2` are the anchor minus `screenPosition` (`:45743-45744`) —
/// the port's `(world − camera)·zoom + screen/2` transform, the same one
/// the NPC bubble rides. The sign sits RIGHT of the player
/// (`signX > position.X + width`, `:45750`) flips the sheet horizontally
/// and shifts `−8 − Chat2.Width()` (`:45753`); otherwise `+8` (`:45757`).
/// The draw is at `(num, num2 − 22)` in `Color(mouseTextColor ×4)`
/// (`:45759`). The `gravDir == −1` mirror (`:45745-45748`) is
/// unreachable: the port keeps `gravDir = 1` (player.rs — the same fold
/// as [`draw_bubble`]'s `:43149-43153` arm). W4-f22.
pub(crate) fn draw_sign_bubble(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    player: Option<&crate::player::Player>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    anchor: Option<(i32, i32)>,
    pulse: UiPulse,
) {
    let Some((sx, sy)) = anchor else {
        return; // `if (Main.signBubble)` (`:45741`)
    };
    let Some(sheet) = lookups.ui_sign_bubble else {
        return;
    };
    let Some(p) = player else {
        return;
    };
    let mut num = (sx as f32 - camera.x) * zoom + screen.x * 0.5;
    let num2 = (sy as f32 - camera.y) * zoom + screen.y * 0.5;
    let flip = sx as f32 > p.x + p.width; // `:45750`
    if flip {
        num += (-8.0 - sheet.width as f32) * zoom; // `:45753`
    } else {
        num += 8.0 * zoom; // `:45757`
    }
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let mut uv = [
        sheet.x as f32 / aw,
        sheet.y as f32 / ah,
        (sheet.x + sheet.width) as f32 / aw,
        (sheet.y + sheet.height) as f32 / ah,
    ];
    if flip {
        uv = crate::render::player_layers::flip_uv(uv, true);
    }
    let m = pulse.text;
    r.push_ui_sprite(
        [num, num2 - 22.0 * zoom], // `:45759`
        [sheet.width as f32 * zoom, sheet.height as f32 * zoom],
        uv,
        [m; 4],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets;

    /// `NPCChatPanel.cs:65-68` + `:93` — with one interaction line the
    /// panel is `30 + 30·lines + 30` tall and the Close row sits at
    /// `bottom − 22 − 4`, 30 px in from the left edge.
    #[test]
    fn the_close_row_rides_the_panel_bottom() {
        let lookups = assets::Lookups::default();
        let screen = Vec2::new(1920.0, 1080.0);
        let panel = panel_rect(&lookups, screen, "Hello there", 1);
        // One text line (no font sheet: the 8 px/char estimate keeps
        // "Hello there" well under 460), one interaction line.
        assert_eq!(panel, [710.0, 100.0, 500.0, 90.0], "30 + 30·1 + 30·1");
        let rect = close_button_rect(&lookups, screen, "Hello there");
        assert_eq!(rect[0], 740.0, "panel left + 30 (`:93`)");
        assert_eq!(rect[1], 100.0 + 90.0 - 22.0 - 4.0, "bottom − 22·1 − 4 (`:93`)");
        assert_eq!(rect[3], 22.0, "the 22 px row (`:97`)");
    }

    /// `TextDisplayCache.cs:27` — `Utils.WordwrapString(text, …, 460, 10)`
    /// caps the display at 10 lines; explicit newlines break rows.
    #[test]
    fn the_text_cache_caps_at_ten_lines() {
        let lookups = assets::Lookups::default();
        let many = "line\n".repeat(30);
        assert_eq!(panel_lines(&lookups, &many).len(), 10);
        assert_eq!(
            panel_lines(&lookups, "one\ntwo"),
            vec!["one".to_string(), "two".to_string()]
        );
    }

    /// `Main.cs:43049`/`:43065` — the bubble pick needs a live town-like
    /// NPC under BOTH the pointer and the player's reach region.
    #[test]
    fn the_bubble_pick_gates_on_hover_and_reach() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let p = app.player.as_ref().unwrap();
        let mut guide = Npc::new(crate::npc::NpcKind::Guide, p.x + 16.0, p.y, 1, 0.0);
        guide.hp = guide.max_hp;
        let (gw, gh) = guide.size();
        let over = Vec2::new(guide.x + gw * 0.5, guide.y + gh * 0.5);
        let npcs = vec![guide];
        assert_eq!(bubble_npc(&npcs, app.player.as_ref(), over), Some(0));
        // Off the NPC box: no bubble.
        assert_eq!(
            bubble_npc(&npcs, app.player.as_ref(), over + Vec2::new(500.0, 0.0)),
            None
        );
        // Out of reach (the region is ±5 tiles): no bubble even hovered.
        let mut far = npcs.clone();
        far[0].x += 16.0 * 40.0;
        let far_over = Vec2::new(far[0].x + gw * 0.5, far[0].y + gh * 0.5);
        assert_eq!(bubble_npc(&far, app.player.as_ref(), far_over), None);
    }

    /// `CloseChat.Interact` → `Main.CloseNPCChatOrSign`
    /// (`NPCInteractions.cs:283-286`, `Main.cs:40680-40692`): the click
    /// clears `talkNPC`/`npcChatText` and plays sound 11; the panel's
    /// `Close` resets `_lastHovered` (`NPCChatPanel.cs:265-269`).
    #[test]
    fn the_close_button_click_clears_the_talk_state() {
        // The `LegacyInterface.52` label the button measures/prints — the
        // same guard the language readers hold (npc_chat.rs's init_lang).
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.talk_npc = Some(0);
        app.npc_chat_text = "Hello there".to_string();
        let (sw, sh) = app.screen_size();
        let rect = close_button_rect(&app.lookups, Vec2::new(sw, sh), &app.npc_chat_text);
        app.mouse_pos = Vec2::new(rect[0] + 2.0, rect[1] + 2.0);
        app.ui_left_press = true;
        app.handle_npc_chat_clicks();
        assert_eq!(app.talk_npc, None, "SetTalkNPC(-1) (`Main.cs:40683`)");
        assert!(app.npc_chat_text.is_empty(), "npcChatText = \"\" (`:40685`)");
        assert!(!app.ui_left_press, "the press is consumed (`NPCChatPanel.cs:152`)");
        assert_eq!(app.npc_chat_last_hovered, -1, "Close resets _lastHovered (`:267`)");
    }

    /// W4-f22 — the sign interaction row (`NPCInteractions.cs:15-42`,
    /// registered after CloseChat at `:515-516`): with a sign open the row
    /// carries Close + Edit (inter 48) / Save (inter 47 while editing), the
    /// second rect rides the first's width + 30 (`NPCChatPanel.cs:135`),
    /// and the Edit click opens the ctx-1 keyboard while the Save click
    /// submits (`OpenSign.Interact`, `:31-41`).
    #[test]
    fn the_sign_row_toggles_edit_and_save() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        // A sign record + the open latch (the OpenSign state).
        app.world.signs.push(tr_core::wld::sections::WldSign {
            text: "stored".into(),
            x: 2,
            y: 2,
        });
        app.player = Some(crate::player::Player::spawn(&app.world));
        if let Some(p) = app.player.as_mut() {
            p.sign = 0;
        }
        app.npc_chat_text = "stored".into();
        let labels = interactions(true, false);
        assert_eq!(labels.len(), 2, "Close + Edit");
        assert_eq!(labels[0], crate::lang::inter(52));
        assert_eq!(labels[1], crate::lang::inter(48));
        assert_eq!(interactions(true, true)[1], crate::lang::inter(47), "Save while editing");
        assert_eq!(interactions(false, false).len(), 1, "no sign, no row");
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let rects = interaction_rects(&app.lookups, screen, &app.npc_chat_text, &labels);
        assert_eq!(rects[1][0], rects[0][0] + rects[0][2] + 30.0, "`:135`");
        // The Edit click opens the keyboard (`OpenSign.Interact`'s else).
        app.mouse_pos = Vec2::new(rects[1][0] + 2.0, rects[1][1] + 2.0);
        app.ui_left_press = true;
        app.handle_npc_chat_clicks();
        assert!(app.sign_editing(), "OpenVirtualKeyboard(1) ran");
        assert!(!app.ui_left_press, "the press is consumed");
        // The Save click submits the live text and closes the editor.
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text = "written".into();
        }
        let labels = interactions(true, true);
        let rects = interaction_rects(&app.lookups, screen, "written", &labels);
        app.mouse_pos = Vec2::new(rects[1][0] + 2.0, rects[1][1] + 2.0);
        app.ui_left_press = true;
        app.handle_npc_chat_clicks();
        assert!(!app.sign_editing(), "SubmitSignText closed the editor");
        assert_eq!(app.world.signs[0].text, "written");
        assert_eq!(app.player.as_ref().unwrap().sign, 0, "the sign stays open");
    }

    /// W4-f22 retire — the d120 compose note's dynamic-lines fix:
    /// `_neededInteractionLines = ceil(count/4)` (`NPCChatPanel.cs:174`)
    /// grows the panel (`:68`'s `30·lines + max(0, 2·(lines−1))`) and the
    /// `num % 4 == 0` reset steps the row y by `num / 4 * 22` (`:106`).
    /// With 5 buttons the 5th rides a second row 22 px down at the row
    /// origin; the ≤4 rows keep the landed one-line layout.
    #[test]
    fn more_than_four_buttons_wrap_to_a_second_row() {
        let lookups = assets::Lookups::default();
        let screen = Vec2::new(1920.0, 1080.0);
        assert_eq!(interaction_lines(0), 0, "ceil(0/4)");
        assert_eq!(interaction_lines(4), 1, "ceil(4/4)");
        assert_eq!(interaction_lines(5), 2, "ceil(5/4) (`:174`)");
        assert_eq!(interaction_lines(8), 2, "ceil(8/4)");
        assert_eq!(interaction_lines(9), 3, "ceil(9/4)");
        // One text line ("Hi" under 460 at the 8 px/char estimate).
        let five: Vec<String> = (0..5).map(|i| format!("B{i}")).collect();
        let panel5 = panel_rect(&lookups, screen, "Hi", five.len());
        // `:68`: 30 + 30·1 + 30·2 + max(0, 2·(2−1)) = 122.
        assert_eq!(panel5[3], 122.0);
        let rects = interaction_rects(&lookups, screen, "Hi", &five);
        let base_y = panel5[1] + panel5[3] - 22.0 * 2.0 + 2.0 - 4.0; // `:93`
        assert_eq!(rects[0][1], base_y);
        assert_eq!(rects[3][1], base_y, "the first row keeps the origin y");
        assert_eq!(rects[4][0], panel5[0] + 30.0, "`:105` — x resets");
        assert_eq!(rects[4][1], base_y + 22.0, "`:106` — num/4·22");
        // The ≤4 case is unchanged: one line, no y step.
        let four = &five[..4];
        let panel4 = panel_rect(&lookups, screen, "Hi", four.len());
        assert_eq!(panel4[3], 90.0, "30 + 30·1 + 30·1 + 0");
        let r4 = interaction_rects(&lookups, screen, "Hi", four);
        assert!(r4.iter().all(|r| r[1] == r4[0][1]));
    }
    /// W4-g58: `ReportHappiness` (`NPCInteractions.cs:289-312`) — the
    /// button's rect rides the Close button's right edge + 30
    /// (`NPCChatPanel.cs:135`), and its click replaces the chat line with
    /// the latched report while the talk latch stays.
    #[test]
    fn the_happiness_button_replaces_the_chat_text() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        // A housed Guide with a non-empty report — the Condition holds.
        let mut guide = Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.hp = guide.max_hp;
        guide.home = Some((6, 6));
        app.npcs.push(guide);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.set_talk_npc(Some(0));
        assert!(
            !app.shopping_settings.happiness_report.is_empty(),
            "GetShoppingSettings latched a report (`Player.cs:4389`)"
        );
        let report = app.shopping_settings.happiness_report.clone();
        app.npc_chat_text = "Hello there".to_string();
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let close = close_button_rect(&app.lookups, screen, &app.npc_chat_text);
        let happy = happiness_button_rect(&app.lookups, screen, &app.npc_chat_text, true)
            .expect("the Condition holds");
        // `:135` — the next button starts at Close's right edge + 30.
        assert_eq!(happy[0], close[0] + close[2] + 30.0);
        assert_eq!(happy[1], close[1], "the same row (`:103-107`)");
        app.mouse_pos = Vec2::new(happy[0] + 2.0, happy[1] + 2.0);
        app.ui_left_press = true;
        app.handle_npc_chat_clicks();
        assert_eq!(app.talk_npc, Some(0), "the talk latch stays (`:305-311`)");
        assert_eq!(app.npc_chat_text, report, "npcChatText = HappinessReport (`:309`)");
        assert!(!app.ui_left_press, "the press is consumed (`:152`)");
        // The report ends with a trailing space — `string.Concat(…, " ")`
        // (`ShopHelper.cs:195`).
        assert!(app.npc_chat_text.ends_with(' '));
    }

    /// `ReportHappiness.Condition` (`NPCInteractions.cs:291-298`): a
    /// homeless NPC shows the Housing text instead — no Happiness button.
    #[test]
    fn the_homeless_npc_hides_the_happiness_button() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut guide = Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.hp = guide.max_hp;
        guide.home = None; // homeless
        app.npcs.push(guide);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.set_talk_npc(Some(0));
        assert!(crate::happiness::can_show_homeless_text(app.npcs.get(0)));
        // The Condition's first arm fails — the button never draws/hits.
        let (sw, sh) = app.screen_size();
        let cond = !crate::happiness::can_show_homeless_text(app.npcs.get(0))
            && !app.shopping_settings.happiness_report.is_empty();
        assert!(!cond);
        assert!(
            happiness_button_rect(&app.lookups, Vec2::new(sw, sh), &app.npc_chat_text, cond)
                .is_none()
        );
        // The Travelling Merchant is excluded from the homeless text
        // (`NPC.cs:94836-94838`) but his report is empty (`ProcessMood`'s
        // early return) — the second arm hides the button.
        let mut tm = Npc::new(crate::npc::NpcKind::TravellingMerchant, 100.0, 100.0, 1, 0.0);
        tm.hp = tm.max_hp;
        assert!(!crate::happiness::can_show_homeless_text(Some(&tm)));
    }

    /// `SetTalkNPC(-1)` resets `currentShoppingSettings` to `NotInShop`
    /// (`Player.cs:4383-4386`, `ShoppingSettings.cs:9-13`).
    #[test]
    fn the_talk_close_resets_the_shopping_settings() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut guide = Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.hp = guide.max_hp;
        guide.home = Some((6, 6));
        app.npcs.push(guide);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.set_talk_npc(Some(0));
        assert!(!app.shopping_settings.happiness_report.is_empty());
        app.set_talk_npc(None);
        assert_eq!(app.shopping_settings.price_adjustment, 1.0);
        assert!(app.shopping_settings.happiness_report.is_empty());
    }

    /// W4-g58 final — `Utils.EaseOutBounce` (`Utils.cs:355-363`) and the
    /// hop lift (`Main.cs:40355-40359`): `1 − (1−t)²·|sin(4πt)|` sampled
    /// at the quarter points (the sin zeros make them exact), the lift
    /// `-56·0.25·(1−ease)` clamped at frame 80, and the `num4 == 0` gate
    /// (past frame 119 the hop does NOT repeat).
    #[test]
    fn the_portrait_hop_lift_matches_the_bounce() {
        // The ease at its exact quarter points: t=0 → 1, t=0.25 → 1−
        // 0.5625·|sin π| = 1, t=0.5 → 1, t=1 → 1 (the sin zeros); the
        // mid-bounce dips: t=0.125 → 1−0.765625·1 = 0.234375, t=0.375 →
        // 1−0.390625 = 0.609375, t=0.625 → 1−0.140625 = 0.859375.
        assert!((ease_out_bounce(0.0) - 1.0).abs() < 1e-12);
        assert!((ease_out_bounce(0.125) - 0.234375).abs() < 1e-12);
        assert!((ease_out_bounce(0.25) - 1.0).abs() < 1e-12);
        assert!((ease_out_bounce(0.375) - 0.609375).abs() < 1e-12);
        assert!((ease_out_bounce(0.5) - 1.0).abs() < 1e-12);
        assert!((ease_out_bounce(0.625) - 0.859375).abs() < 1e-12);
        assert!((ease_out_bounce(1.0) - 1.0).abs() < 1e-12);
        // The lift: frame 0 rests (ease 1 → lift 0); frame 10 (t=0.125)
        // lifts -14·(1−0.234375) = -10.71875 px (UP); frame 30 (t=0.375)
        // -14·0.390625 = -5.46875; frame 50 (t=0.625) -14·0.140625 =
        // -1.96875; frame 80+ clamps to t=1 → 0.
        assert_eq!(portrait_hop_lift(0), 0.0);
        assert!((portrait_hop_lift(10) - (-10.71875)).abs() < 1e-4);
        assert!((portrait_hop_lift(30) - (-5.46875)).abs() < 1e-4);
        assert!((portrait_hop_lift(50) - (-1.96875)).abs() < 1e-4);
        assert_eq!(portrait_hop_lift(80), 0.0);
        assert_eq!(portrait_hop_lift(119), 0.0);
        // The `num4 == 0` gate: frame 120 wraps num3 to 0 but num4 = 1 —
        // no second hop (the C# `flag = num4 == 0`, `:40345`/`:40355`).
        assert_eq!(portrait_hop_lift(120), 0.0);
        assert_eq!(portrait_hop_lift(130), 0.0);
        assert_eq!(portrait_hop_lift(1000), 0.0);
    }

    /// W4-g58 final — `Main.DoNPCPortraitHop` (`Main.cs:40675-40678`)
    /// resets the counter; the talk open hops (`Player.cs:4401`), the
    /// Happiness button hops (`NPCInteractions.cs:310`), and the counter
    /// ticks per drawn frame (`Main.cs:40335`).
    #[test]
    fn the_portrait_hop_resets_on_the_talk_and_the_report() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut guide = Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.hp = guide.max_hp;
        guide.home = Some((6, 6));
        app.npcs.push(guide);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // A mid-hop counter: the talk open resets it (`Player.cs:4401`).
        app.npc_chat_portrait_frame_counter = 57;
        app.set_talk_npc(Some(0));
        assert_eq!(app.npc_chat_portrait_frame_counter, 0, "the talk open hops");
        // The Happiness button's Interact hops again (`:310`).
        app.npc_chat_portrait_frame_counter = 42;
        app.npc_chat_text = "Hello there".to_string();
        let (sw, sh) = app.screen_size();
        let happy = happiness_button_rect(
            &app.lookups,
            Vec2::new(sw, sh),
            &app.npc_chat_text,
            true,
        )
        .expect("the Condition holds");
        app.mouse_pos = Vec2::new(happy[0] + 2.0, happy[1] + 2.0);
        app.ui_left_press = true;
        app.handle_npc_chat_clicks();
        assert_eq!(app.npc_chat_portrait_frame_counter, 0, "the report hops");
        // The close does NOT hop (`Player.cs:4397` clears the provider;
        // the hop is inside the `nPC != null` arm).
        app.npc_chat_portrait_frame_counter = 9;
        app.set_talk_npc(None);
        assert_eq!(app.npc_chat_portrait_frame_counter, 9, "the close is hop-free");
    }
}
// g49 union compile-fix: `mod tests` closed here — the Dryad-status helpers
// below are production code (called from `ui::mod::draw_hud`), not tests.

/// The rect at an explicit row origin (`:97`/`:117`).
fn button_rect_at(lookups: &Lookups, label: &str, x: f32, y: f32) -> [f32; 4] {
    [x, y, label_width(lookups, label), 22.0]
}


/// The label's drawn width at scale 0.9, shrunk to fit 260 px
/// (`NPCChatPanel.cs:112-117`).
fn label_width(lookups: &Lookups, label: &str) -> f32 {
    let w0 = text::ui_font_width(lookups, label, 0.9).unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
    let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
    (w0 * shrink).trunc()
}


/// The `DryadPurification` button's condition (`NPCInteractions.cs:107-114`):
/// the talk NPC is the Dryad (type 20) and the Stardew bit cannot play. The
/// port has no `CanDryadPlayStardewAnimation` (the cola/Stardew override is
/// deferred, module header), so the gate folds to the type check — the same
/// fold the selection half's `player_has_cola` ladder keeps.
pub(crate) fn dryad_status_active(npcs: &[crate::npc::Npc], talk_npc: Option<usize>) -> bool {
    talk_npc.is_some_and(|i| {
        npcs.get(i).is_some_and(|n| n.kind == crate::npc::NpcKind::Dryad)
    })
}


/// The `DryadPurification` status button's hit rect — the SECOND interaction
/// (`NPCInteractions.cs:518` registers it after `CloseChat` `:515`), so its
/// row index is 1: `rectangle.X += rectangle.Width + 30` (`NPCChatPanel.cs:135`)
/// past the Close button.
pub(crate) fn dryad_status_button_rect(lookups: &Lookups, screen: Vec2, chat_text: &str) -> [f32; 4] {
    let close = close_button_rect(lookups, screen, chat_text);
    button_rect_at(lookups, &crate::lang::inter(49), close[0] + close[2] + 30.0, close[1])
}