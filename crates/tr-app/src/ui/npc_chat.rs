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
//! systems the panel task does not own), the NPC portrait
//! (`Main.DrawNPCPortrait`, `Main.cs:40318`), the corner item
//! (`Main.DrawNPCChatBottomRightItem`, `:39789`), the sign-editing blinker
//! (`NPCChatPanel.cs:236-262`), the Dryad Stardew override (`:195-213`)
//! and the gamepad/virtual-keyboard arms.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::{text, UiPulse};
use crate::assets::Lookups;
use crate::npc::Npc;
use crate::npc_chat::{chat_box_height, chat_box_rect};

/// `TextDisplayCache.PrepareCache` (`TextDisplayCache.cs:19-30`):
/// `Utils.WordwrapString(text, FontAssets.MouseText.Value, 460, 10, …)`
/// (`Utils.cs:860-867`) — the font-measured wrap at 460 px, capped at 10
/// lines. The port measures with the retail `Mouse_Text` sheet when loaded
/// (`text::ui_font_width`); the bitmap fallback estimates 8 px per char
/// (docs/ANALYSIS.md "Town chatter UI (S1-E-10b)"). The selection half's
/// `format_chat_lines` keeps its char-count wrap for the tests; the drawn
/// panel measures the real font.
pub(crate) fn panel_lines(lookups: &Lookups, chat_text: &str) -> Vec<String> {
    const MAX_LINES: usize = 10; // `TextDisplayCache.cs:27`'s maxLines
    let width = |s: &str| {
        text::ui_font_width(lookups, s, 1.0).unwrap_or(s.chars().count() as f32 * 8.0)
    };
    let max_w = crate::npc_chat::CHAT_TEXT_WRAP_WIDTH as f32;
    let mut rows: Vec<String> = Vec::new();
    for paragraph in chat_text.split('\n') {
        let mut cur = String::new();
        for word in paragraph.split(' ') {
            if cur.is_empty() {
                cur.push_str(word);
            } else if width(&format!("{cur} {word}")) <= max_w {
                cur.push(' ');
                cur.push_str(word);
            } else {
                rows.push(std::mem::take(&mut cur));
                cur.push_str(word);
            }
            if rows.len() >= MAX_LINES {
                return rows;
            }
        }
        rows.push(cur);
        if rows.len() >= MAX_LINES {
            return rows;
        }
    }
    rows.truncate(MAX_LINES);
    rows
}

/// The interaction row count the port renders: `CloseChat`
/// (`NPCInteractions.cs:271-287`, `Condition() => true`) plus the Dryad's
/// `DryadPurification` status button (`NPCInteractions.cs:105-131`, W4-g49 —
/// the `57` census display consumer). Both fit one row
/// (`_neededInteractionLines = ceil(2 / 4) = 1`, `NPCChatPanel.cs:174`,
/// `AllowedInteractionsPerLine = 4` `:30`); the shop / happiness / housing /
/// … buttons stay deferred (module header).
const INTERACTION_LINES: usize = 1;

/// The panel's box (`NPCChatPanel.cs:65-68` via the selection half's
/// `chat_box_rect`/`chat_box_height`): `[x, y, w, h]` in UI pixels.
pub(crate) fn panel_rect(lookups: &Lookups, screen: Vec2, chat_text: &str) -> [f32; 4] {
    let base = chat_box_rect(screen.x as i32, screen.y as i32);
    let lines = panel_lines(lookups, chat_text).len();
    let h = chat_box_height(lines, INTERACTION_LINES);
    [base.x as f32, base.y as f32, base.w as f32, h as f32]
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

/// The Close button's hit rect (`NPCChatPanel.cs:93-117`): the row origin
/// is `panelArea.BottomLeft() + (30, -22·lines + max(0, 2·(lines−1)) − 4)`,
/// the rect 22 tall and as wide as the label measured at scale 0.9
/// (shrunk to fit 260 px, `:113-117`).
pub(crate) fn close_button_rect(lookups: &Lookups, screen: Vec2, chat_text: &str) -> [f32; 4] {
    button_rect(lookups, screen, chat_text, &crate::lang::inter(52), 0)
}

/// The `DryadPurification` status button's hit rect — the SECOND interaction
/// (`NPCInteractions.cs:518` registers it after `CloseChat` `:515`), so its
/// row index is 1: `rectangle.X += rectangle.Width + 30` (`NPCChatPanel.cs:135`)
/// past the Close button.
pub(crate) fn dryad_status_button_rect(lookups: &Lookups, screen: Vec2, chat_text: &str) -> [f32; 4] {
    let close = close_button_rect(lookups, screen, chat_text);
    button_rect_at(lookups, &crate::lang::inter(49), close[0] + close[2] + 30.0, close[1])
}

/// One interaction button's rect at row index `idx` (`DrawButtons`
/// `NPCChatPanel.cs:93-117`): the first button sits at the row origin, each
/// next one `width + 30` to the right (`:135`).
fn button_rect(lookups: &Lookups, screen: Vec2, chat_text: &str, label: &str, idx: usize) -> [f32; 4] {
    let panel = panel_rect(lookups, screen, chat_text);
    let il = INTERACTION_LINES as f32;
    let x0 = panel[0] + 30.0;
    let y = panel[1] + panel[3] - 22.0 * il + (2.0 * (il - 1.0)).max(0.0) - 4.0;
    // Walk the preceding buttons' widths (`:135`'s advance).
    let mut x = x0;
    for prev in 0..idx {
        let prev_label = if prev == 0 { crate::lang::inter(52) } else { crate::lang::inter(49) };
        let w = label_width(lookups, &prev_label);
        x += w + 30.0;
    }
    button_rect_at(lookups, label, x, y)
}

/// The label's drawn width at scale 0.9, shrunk to fit 260 px
/// (`NPCChatPanel.cs:112-117`).
fn label_width(lookups: &Lookups, label: &str) -> f32 {
    let w0 = text::ui_font_width(lookups, label, 0.9).unwrap_or(label.chars().count() as f32 * 8.0 * 0.9);
    let shrink = if w0 > 260.0 { 260.0 / w0 } else { 1.0 };
    (w0 * shrink).trunc()
}

/// The rect at an explicit row origin (`:97`/`:117`).
fn button_rect_at(lookups: &Lookups, label: &str, x: f32, y: f32) -> [f32; 4] {
    [x, y, label_width(lookups, label), 22.0]
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
/// (`:63-64`), and the Close button row (`DrawButtons` `:88-162`).
/// Returns whether the pointer is inside the panel — the C#'s
/// `LocalPlayer.mouseInterface = true` arm (`:81-84`); the port has no
/// mouseInterface field yet, so the caller records the flag (deferred).
pub(crate) fn draw_panel(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    pulse: UiPulse,
    mouse: Vec2,
    dryad_status: bool,
) -> bool {
    let lines = panel_lines(lookups, chat_text);
    let panel = panel_rect(lookups, screen, chat_text);
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
    draw_close_button(r, lookups, screen, chat_text, pulse, mouse);
    // The Dryad's status button (`DryadPurification`, W4-g49) — the second
    // interaction, drawn only while its condition holds.
    if dryad_status {
        let rect = dryad_status_button_rect(lookups, screen, chat_text);
        draw_interaction_button(r, lookups, &crate::lang::inter(49), rect, pulse, mouse);
    }
    super::in_rect(mouse, panel)
}

/// The Close button (`DrawButtons`, `NPCChatPanel.cs:88-162`, one
/// interaction): idle scale 0.9 / hover 1.2 (`:96`, `:119`), the label
/// colour `(b, b/1.1, b/2, b)` (`:102`), the shadow Black idle / Brown
/// hovered (`:121`), the origin `(0, stringSize.Y·0.5)` at the rect's
/// left/centre-Y (`:120-122`) with the hover x-shift
/// `−(1.2−0.9)·width·0.5` (`:125`).
fn draw_close_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    chat_text: &str,
    pulse: UiPulse,
    mouse: Vec2,
) {
    let rect = close_button_rect(lookups, screen, chat_text);
    draw_interaction_button(r, lookups, &crate::lang::inter(52), rect, pulse, mouse);
}

/// One interaction button's draw (`DrawButtons`, `NPCChatPanel.cs:98-133`):
/// the shared label maths the Close button and the Dryad status button
/// (W4-g49) both ride.
fn draw_interaction_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    label: &str,
    rect: [f32; 4],
    pulse: UiPulse,
    mouse: Vec2,
) {
    let hovered = super::in_rect(mouse, rect);
    let b = (pulse.text * 255.0) as i32;
    let chat_color = [
        b as f32 / 255.0,
        (b as f64 / 1.1) as i32 as f32 / 255.0, // `:102` — `(int)(b/1.1)`
        (b / 2) as f32 / 255.0,
        b as f32 / 255.0,
    ];
    // `Color.Brown` = (165, 42, 42) — the hovered shadow (`:121`).
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
        // `:125` — `(int)((1.2 − 0.9) · rectangle.Width · 0.5)`.
        px -= ((1.2 - 0.9) * rect[2] * 0.5).floor();
    }
    // `origin = (0, stringSize.Y · 0.5)` (`:120`) where stringSize is
    // measured at 0.9 (`:112`); XNA applies origin × draw scale, so the
    // top-left y is centre-Y − h(0.9)·0.5·(scale·shrink).
    let lh = text::ui_font_height(lookups, 0.9).unwrap_or(8.0 * 0.9);
    let py = rect[1] + rect[3] * 0.5 - lh * 0.5 * scale * shrink;
    // `ChatManager.DrawColorCodedStringShadow` (`:132`): four passes at
    // `ShadowDirections · spread` — ±X/±Y × 2 (`ChatManager.cs:200-205`,
    // `:376-382`) — in Brown hovered / Black idle (`:121`), then the
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
        let panel = panel_rect(&lookups, screen, "Hello there");
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
}
