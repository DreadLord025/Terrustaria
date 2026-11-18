//! The in-game notification popups — `Terraria.UI.InGameNotificationsTracker`
//! (`UI/InGameNotificationsTracker.cs:11-113`) and the achievement-unlock
//! popup `InGamePopups.AchievementUnlockedPopup` (`UI/InGamePopups.cs:13-148`).
//! S1-E-26 wave A: the achievement-complete toast the framework gap left
//! unreachable.
//!
//! The chain: `AchievementManager.AchievementCompleted`
//! (`AchievementManager.cs:180-187`) fires `OnAchievementCompleted`;
//! `InGameNotificationsTracker.AddCompleted` (`:75-81`, the `netMode != 2`
//! gate) appends an `AchievementUnlockedPopup`; the tracker's `Update`
//! (`Main.cs:17045`) ticks the 300-frame display timer and
//! `DrawInterface_13_AchievementCompletePopups` (`Main.cs:44779-44783`,
//! `:46143-46146`; the menu twin at `:48457`) draws the stack bottom-centre.
//! The port's event seam is `AchievementManager::completed_events`, drained
//! by `App::drain_achievement_completions`.
//!
//! The join-request popup (`PlayerWantsToJoinGamePopup`, `InGamePopups.cs:150`)
//! and the notifications-area draw (`DrawInIngameOptions`, `:57-73`) have no
//! port — the join-request social flow is unported.

use crate::assets;
use crate::ui::{self, color, text, tooltip};
use glam::Vec2;
use tr_render::SpriteRenderer;

/// `_ingameDisplayTimeLeft = 300` (`InGamePopups.cs:71`).
pub const DISPLAY_TIME: i32 = 300;

/// `AchievementUnlockedPopup` (`InGamePopups.cs:13-148`): one unlock toast.
/// The C# keeps the achievement reference + the two texture assets; the port
/// keeps the row name (the goto target), the baked icon index
/// (`Main.Achievements.GetIconIndex`, `:74`) and the title string
/// (`achievement.FriendlyName.Value`, `:73`).
#[derive(Clone, Debug)]
pub struct AchievementPopup {
    /// `CreationObject`'s achievement name — the click's
    /// `OpenAchievementsAndGoto` target (`:133`/`:138`).
    pub name: &'static str,
    /// `_title` — `Achievement.FriendlyName.Value` at construction (`:73`).
    pub title: String,
    /// `_iconIndex` — `GetIconIndex(achievement.Name)` (`:74`); the frame is
    /// `(index % 8 * 66, index / 8 * 66, 64, 64)` (`:75`).
    pub icon: u16,
    /// `_ingameDisplayTimeLeft` (`:33`), armed to 300 (`:71`).
    pub time_left: i32,
    /// `ShouldBeRemoved` (`:35`) — set only by the click (`:141`); an expired
    /// popup stays in the list at opacity 0 until `Clear` (the vanilla
    /// behaviour — `Update` never sets it, `:80-87`).
    pub should_be_removed: bool,
}

impl AchievementPopup {
    /// The constructor (`InGamePopups.cs:68-78`): the 300-tick timer, the
    /// title and the icon index; the texture requests resolve through the
    /// port's atlas lookups at draw.
    pub fn new(name: &'static str, icon: u16, title: String) -> Self {
        Self {
            name,
            title,
            icon,
            time_left: DISPLAY_TIME,
            should_be_removed: false,
        }
    }

    /// `Update` (`InGamePopups.cs:80-87`): one tick off the timer, floored
    /// at 0.
    pub fn update(&mut self) {
        self.time_left -= 1;
        if self.time_left < 0 {
            self.time_left = 0;
        }
    }

    /// `Scale` (`InGamePopups.cs:39-53`): the timer counts down from 300 —
    /// the `> 285` arm is the 15-tick fade-in (`Lerp(1, 0, (t-285)/15)` runs
    /// 0→1 as t falls), 30..285 holds 1, the `< 30` arm fades out.
    pub fn scale(&self) -> f32 {
        scale_at(self.time_left)
    }

    /// `Opacity` (`InGamePopups.cs:55-66`): 0 while the scale is ≤ 0.5,
    /// then `(scale - 0.5) / 0.5`.
    pub fn opacity(&self) -> f32 {
        opacity_at(self.time_left)
    }
}

/// `Scale`'s maths (`InGamePopups.cs:43-51`), timer-parameterised for the
/// tests. `MathHelper.Lerp(a, b, t) = a + (b - a) * t`.
pub fn scale_at(time_left: i32) -> f32 {
    if time_left < 30 {
        return time_left as f32 / 30.0;
    }
    if time_left > 285 {
        return 1.0 - (time_left as f32 - 285.0) / 15.0;
    }
    1.0
}

/// `Opacity`'s maths (`InGamePopups.cs:59-64`).
pub fn opacity_at(time_left: i32) -> f32 {
    let scale = scale_at(time_left);
    if scale <= 0.5 {
        return 0.0;
    }
    (scale - 0.5) / 0.5
}

/// `InGameNotificationsTracker._notifications` (`:13`) — the port carries
/// only the achievement popup kind (the join-request half is unported).
#[derive(Default)]
pub struct NotificationsTracker {
    pub popups: Vec<AchievementPopup>,
}

impl NotificationsTracker {
    /// `AddCompleted` (`InGameNotificationsTracker.cs:75-81`): append the
    /// popup. The `Main.netMode != 2` gate rides the caller
    /// (`App::drain_achievement_completions` — the port's `NetRole::Host`
    /// server role, the `audio.rs` S1-E-24 convention).
    pub fn add_completed(&mut self, name: &'static str, icon: u16, title: String) {
        self.popups.push(AchievementPopup::new(name, icon, title));
    }

    /// `Update` (`InGameNotificationsTracker.cs:96-107`): tick every popup,
    /// remove the flagged ones in place (the C#'s `RemoveAt` + `i--`).
    pub fn update(&mut self) {
        let mut i = 0;
        while i < self.popups.len() {
            self.popups[i].update();
            if self.popups[i].should_be_removed {
                self.popups.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// `Clear` (`InGameNotificationsTracker.cs:91-94`) — `WorldGen.SaveAndQuit`
    /// calls it (`WorldGen.cs:6714`).
    pub fn clear(&mut self) {
        self.popups.clear();
    }
}

/// `Utils.CenteredRectangle(center, size)` (`Utils.cs`): the int-truncated
/// top-left + size.
fn centered_rect(center: [f32; 2], size: [f32; 2]) -> [f32; 4] {
    [
        (center[0] - size[0] / 2.0) as i32 as f32,
        (center[1] - size[1] / 2.0) as i32 as f32,
        size[0] as i32 as f32,
        size[1] as i32 as f32,
    ]
}

/// The popup's plate rect (`DrawInGame`, `InGamePopups.cs:100-102`):
/// `size = (MeasureString(title) + (58, 10)) * (Scale * 1.1)`, centred on
/// `anchor + (0, -size.Y / 2)`.
pub fn popup_rect(anchor: [f32; 2], title_size: [f32; 2], scale: f32) -> [f32; 4] {
    let num = scale * 1.1;
    let size = [(title_size[0] + 58.0) * num, (title_size[1] + 10.0) * num];
    centered_rect([anchor[0], anchor[1] - size[1] * 0.5], size)
}

/// `DrawInGame` + `OnMouseOver` (`InGamePopups.cs:95-142`) over the tracker
/// walk (`InGameNotificationsTracker.DrawInGame`, `:38-55`). Returns the
/// clicked achievement's name — the caller runs
/// `IngameFancyUI.OpenAchievementsAndGoto` (`UI/IngameFancyUI.cs`).
///
/// `left_release` is the port's `Main.mouseLeft && Main.mouseLeftRelease`
/// click edge (`:124`); the `PlayerInput.IgnoreMouseInterface` gate (`:119`)
/// and the `mouseInterface` flag (`:123`) have no port counterpart (the buff
/// bar's hover carries the same documented gap). The gamepad anchor drop
/// (`-25f`, `InGameNotificationsTracker.cs:41-44`) is unported — no gamepad.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    tracker: &mut NotificationsTracker,
    mouse: Vec2,
    mouse_text_color: u8,
    left_release: bool,
) -> Option<&'static str> {
    // `Main.screenHeight - 40`, `Main.screenWidth / 2` (`:40`/`:45`).
    let mut anchor = [screen.x / 2.0, screen.y - 40.0];
    let mut clicked: Option<&'static str> = None;
    for popup in &mut tracker.popups {
        let opacity = popup.opacity();
        if opacity > 0.0 {
            let scale = popup.scale();
            // `FontAssets.ItemStack.Value.MeasureString(_title)` (`:101`).
            let title_w = text::hud_item_text_width(r, lookups, &popup.title, 1.0);
            let title_h = lookups.item_font.as_ref().map_or(8.0, |f| f.line_spacing);
            let rect = popup_rect(anchor, [title_w, title_h], scale);
            let hover = ui::in_rect(mouse, rect);
            // `Utils.DrawInvBG` (`:104`): `new Color(64, 109, 164)` at 0.75
            // hovered / 0.5 otherwise (XNA byte truncation).
            let bg = color::mul([64, 109, 164, 255], if hover { 0.75 } else { 0.5 });
            tooltip::draw_inv_bg(r, lookups, rect, color::tint(bg));
            let num = scale * 1.1;
            let num2 = num * 0.3;
            // `Utils.Right(r) - UnitX * num * (12 + num2 * 64)` (`:106`) —
            // `Right`'s Y is the int-divided half height (`Utils.cs`).
            let right_y = rect[1] + (rect[3] as i32 / 2) as f32;
            let vector = [rect[0] + rect[2] - num * (12.0 + num2 * 64.0), right_y];
            // The icon (`:107`): the 64 px frame at `(icon % 8 * 66,
            // icon / 8 * 66)`, origin `(0, 32)`, scale `num2`, White*opacity.
            if let Some(sheet) = lookups.ach_icons {
                let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
                let x0 = sheet.x as f32 + (popup.icon as f32 % 8.0) * 66.0;
                let y0 = sheet.y as f32 + (popup.icon as f32 / 8.0).floor() * 66.0;
                let uv = [x0 / aw, y0 / ah, (x0 + 64.0) / aw, (y0 + 64.0) / ah];
                r.push_ui_sprite(
                    [vector[0], vector[1] - 32.0 * num2],
                    [64.0 * num2, 64.0 * num2],
                    uv,
                    [1.0, 1.0, 1.0, opacity],
                );
            }
            // The border (`:108`): the whole sheet, origin `(4, height / 2)`
            // (int division), scale `num2`.
            if let Some(sheet) = lookups.ach_borders {
                let w = sheet.width as f32;
                let h = sheet.height as f32;
                ui::push_whole(
                    r,
                    lookups,
                    &sheet,
                    [
                        vector[0] - 4.0 * num2,
                        vector[1] - (sheet.height as i32 / 2) as f32 * num2,
                        w * num2,
                        h * num2,
                    ],
                    [1.0, 1.0, 1.0, opacity],
                );
            }
            // The title (`:109`): `new Color(mtc, mtc, mtc / 5, mtc) *
            // opacity`, `DrawBorderString` at `vector - (10, 0)`, scale
            // `num * 0.9`, anchor `(1, 0.4)` — the origin is the unscaled
            // measure times the anchor (the `draw_anchored_text` convention).
            let mtc = mouse_text_color;
            let title_color = color::mul([mtc, mtc, mtc / 5, mtc], opacity);
            let t_scale = num * 0.9;
            let w = text::hud_text_width(r, lookups, &popup.title, 1.0);
            let h = text::line_height(lookups);
            text::draw_border_string(
                r,
                lookups,
                text::BorderFont::Ui,
                vector[0] - 10.0 - w * t_scale,
                vector[1] - 0.4 * h * t_scale,
                t_scale,
                &[[1.0, 1.0]],
                color::tint(title_color),
                &popup.title,
            );
            // `OnMouseOver` (`:117-142`): the click opens the achievements
            // menu at the row, zeroes the timer and flags the removal. The
            // C# consumes `mouseLeftRelease` so only the first hovered popup
            // fires; the `gameMenu` arm's `menuMode == 0` gate (`:129-134`)
            // is moot in the port — `SaveAndQuit` clears the tracker
            // (`WorldGen.cs:6714`), so no popup survives onto the menu.
            if hover && left_release && clicked.is_none() {
                popup.time_left = 0;
                popup.should_be_removed = true;
                clicked = Some(popup.name);
            }
        }
        // `PushAnchor` (`:89-93`): `anchor.Y -= 50 * Opacity`.
        anchor[1] -= 50.0 * opacity;
        // The walk's off-screen break (`InGameNotificationsTracker.cs:50-53`).
        if anchor[1] < -100.0 {
            break;
        }
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Scale`/`Opacity` (`InGamePopups.cs:39-66`): the timer counts DOWN
    /// from 300, so the `> 285` arm is the 15-tick fade-IN (300→285:
    /// `Lerp(1, 0, (t - 285) / 15)` runs 0→1), the hold band is 30..285 and
    /// the `< 30` arm is the 30-tick fade-out. Opacity floors at scale 0.5.
    #[test]
    fn the_scale_and_opacity_follow_the_300_tick_curve() {
        // Fresh popup (t = 300): Lerp(1, 0, 15/15) = 0 — invisible until the
        // first Update ticks, exactly as the C# constructor leaves it.
        assert_eq!(scale_at(300), 0.0);
        assert_eq!(opacity_at(300), 0.0);
        // The fade-in's first tick: Lerp(1, 0, 14/15) = 1/15.
        assert!((scale_at(299) - 1.0 / 15.0).abs() < 1e-6);
        assert!((scale_at(286) - 14.0 / 15.0).abs() < 1e-6);
        // The hold band.
        assert_eq!(scale_at(285), 1.0);
        assert_eq!(opacity_at(285), 1.0);
        assert_eq!(scale_at(30), 1.0);
        // The fade-out: Lerp(0, 1, t/30); opacity 0 until scale > 0.5.
        assert!((scale_at(15) - 0.5).abs() < 1e-6);
        assert_eq!(opacity_at(15), 0.0);
        assert_eq!(opacity_at(16), (16.0 / 30.0 - 0.5) / 0.5);
        assert_eq!(scale_at(0), 0.0);
        assert_eq!(opacity_at(0), 0.0);
    }

    /// `Update` (`InGamePopups.cs:80-87`) + the tracker's removal walk
    /// (`InGameNotificationsTracker.cs:96-107`): the timer floors at 0 and an
    /// expired popup is NOT removed — `ShouldBeRemoved` only rides the click
    /// (`:141`), the vanilla quirk the port keeps.
    #[test]
    fn the_timer_floors_at_zero_and_only_the_click_removes() {
        let mut t = NotificationsTracker::default();
        t.add_completed("TIMBER", 1, "Timber!".into());
        for _ in 0..350 {
            t.update();
        }
        assert_eq!(t.popups.len(), 1, "expiry alone never removes the popup");
        assert_eq!(t.popups[0].time_left, 0);
        assert_eq!(t.popups[0].opacity(), 0.0);
        // The click's flag removes on the next update.
        t.popups[0].should_be_removed = true;
        t.update();
        assert!(t.popups.is_empty());
    }

    /// `AddCompleted` appends in completion order; `Clear`
    /// (`WorldGen.cs:6714`'s SaveAndQuit call) empties the list.
    #[test]
    fn the_tracker_appends_in_order_and_clears() {
        let mut t = NotificationsTracker::default();
        t.add_completed("TIMBER", 1, "a".into());
        t.add_completed("BENCHED", 2, "b".into());
        assert_eq!(
            t.popups.iter().map(|p| p.name).collect::<Vec<_>>(),
            ["TIMBER", "BENCHED"]
        );
        assert!(t.popups.iter().all(|p| p.time_left == DISPLAY_TIME));
        t.clear();
        assert!(t.popups.is_empty());
    }

    /// The plate rect (`InGamePopups.cs:100-102`): `(measure + (58, 10)) *
    /// (Scale * 1.1)`, int-truncated, centred half a height above the anchor.
    #[test]
    fn the_plate_rect_is_the_measured_title_plus_the_padding() {
        // scale 1 → num 1.1; a 100x20 title → size (158*1.1, 30*1.1) =
        // (173.8, 33) → truncated (173, 33).
        let r = popup_rect([800.0, 860.0], [100.0, 20.0], 1.0);
        assert_eq!(r[2], 173.0);
        assert_eq!(r[3], 33.0);
        // centre = (800, 860 - 16.5) → top-left (800 - 86.9, 843.5 - 16.5)
        // truncated to (713, 827).
        assert_eq!(r[0], 713.0);
        assert_eq!(r[1], 827.0);
    }

    /// S1-E-26 wave A: the App seam — a completing row queues the event, the
    /// drain appends the popup (the `netMode != 2` arm), prints the
    /// `Achievements.Completed` chat line (`AchievementInitializer.cs:730`)
    /// and plays the `NewText` tick (`Main.NewText` → `PlaySound(12)`).
    #[test]
    fn the_completion_drain_feeds_the_popup_chat_and_sound() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        assert_eq!(
            app.achievements
                .notify(crate::achievements::CondKind::NpcKilled(14)),
            Some("WORM_FODDER")
        );
        assert_eq!(app.achievements.completed_events, ["WORM_FODDER"]);
        let played_before = app.audio.played_count();
        app.drain_achievement_completions();
        // The popup: name, icon (`GetIconIndex`) and the 300-tick timer.
        assert_eq!(app.notifications.popups.len(), 1);
        let p = &app.notifications.popups[0];
        assert_eq!(p.name, "WORM_FODDER");
        assert_eq!(
            p.icon,
            crate::achievements::AchievementManager::icon_index("WORM_FODDER").unwrap()
        );
        assert_eq!(p.time_left, DISPLAY_TIME);
        // The chat line (`Main.NewText`'s white default) and the tick cue.
        assert_eq!(app.chat.messages.len(), 1);
        assert_eq!(app.chat.messages[0].color, [255, 255, 255]);
        assert!(app.audio.played_count() > played_before);
        // The queue drained — a second drain is a no-op.
        app.drain_achievement_completions();
        assert_eq!(app.notifications.popups.len(), 1);
        assert_eq!(app.chat.messages.len(), 1);
    }

    /// `AddCompleted`'s `Main.netMode != 2` gate
    /// (`InGameNotificationsTracker.cs:77`): the server role queues no
    /// popup, but the chat/sound subscriber still runs (it has no net gate,
    /// `AchievementInitializer.cs:728-735`).
    #[test]
    fn the_server_role_skips_the_popup_but_keeps_the_chat_line() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // The port's `netMode == 2` stand-in (the `audio.rs` S1-E-24
        // convention): a Host role. The server never binds a listener, so
        // the fixture's `Server::start` is not needed — the drain reads the
        // role only.
        app.net = crate::app::NetRole::Host(crate::net::Server::start(0, String::new()).unwrap());
        app.achievements
            .notify(crate::achievements::CondKind::NpcKilled(14));
        app.drain_achievement_completions();
        assert!(
            app.notifications.popups.is_empty(),
            "netMode == 2 adds no popup"
        );
        assert_eq!(app.chat.messages.len(), 1);
    }

    /// `OnMouseOver` → `IngameFancyUI.OpenAchievementsAndGoto`
    /// (`InGamePopups.cs:133-140`): the click opens the overlay and
    /// `UIList.Goto` scrolls to the row's `Top.Pixels`.
    #[test]
    fn the_goto_opens_the_overlay_and_scrolls_to_the_row() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Complete a late row so the goto has a real scroll target: the
        // completed rows sort first, so BULLDOZER (id ~30) sits below them.
        app.achievements.notify_custom_int("Pick", 10_000);
        app.achievements.take_completed_events();
        app.open_achievements_and_goto("BULLDOZER");
        assert!(app.achievements_open);
        assert_eq!(app.achievements_categories, [true; 4]);
        let (sw, sh) = app.screen_size();
        let panel = ui::achievements::panel_rect(Vec2::new(sw, sh), false);
        let list = ui::achievements::list_rect(&panel);
        let rows = ui::achievements::visible_rows(&app.achievements, &[true; 4]);
        let idx = rows
            .iter()
            .position(|&i| app.achievements.list[i].name == "BULLDOZER")
            .unwrap();
        let expected = (ui::achievements::row_rect(&list, idx)[1] - list[1])
            .clamp(0.0, ui::achievements::max_scroll(rows.len(), &list));
        assert_eq!(app.achievements_scroll, expected);
        // An unknown name falls back to the top (the C#'s Goto leaves the
        // ViewPosition untouched when no row matches).
        app.open_achievements_and_goto("NO_S_ROW");
        assert_eq!(app.achievements_scroll, 0.0);
    }

    /// `WorldGen.SaveAndQuit`'s `InGameNotificationsTracker.Clear()`
    /// (`WorldGen.cs:6714`): the toasts never survive onto the menu.
    #[test]
    fn save_and_quit_clears_the_popups() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.notifications
            .add_completed("TIMBER", 0, "Timber!".into());
        app.save_and_quit();
        assert!(app.notifications.popups.is_empty());
    }
}
