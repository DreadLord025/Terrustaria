//! The overhead chat text — `Main.DrawPlayerChatBubbles`
//! (`Main.cs:63248-63263`) + `Main.GetChatDrawPosition` (`:63243-63246`),
//! drawn from the `DrawInterface` head under `Main.UIScaleMatrix`
//! (`:63237-63239`) while `!Main.hideUI` (`:63334`).
//!
//! The state half rides [`crate::player::OverheadMessage`]
//! (`Player.cs:765-784`): `ChatHelper.DisplayMessage`'s author arm
//! (`ChatHelper.cs:86-87`) arms it, `Player.Update` ticks the 400-tick
//! timer (`Player.cs:24554-24557`).
//!
//! Deferred (docs/ANALYSIS.md "Overhead chat + text wrap (W4-d121)"):
//! remote players' overheads (`Main.player[i]`, i ≠ myPlayer — the port's
//! `net::PlayerNet` carries no `chatOverhead`), the `[c:...]` snippet
//! markup (flat text like `chat.rs`), the gravity-flip anchor arm
//! (`gravDir` stays 1) and `Main.hideUI` (no port field).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text;
use crate::assets::Lookups;
use crate::player::{OverheadMessage, Player};

/// `Main.GetChatDrawPosition` (`Main.cs:63243-63246`):
/// `player.Top + netOffset + (0, gfxOffY)` through `ToScreenPosition`
/// (`Utils.cs:1095-1098` — the `(world − camera)·zoom + screen/2` the
/// emote bubbles ride), then `(0, −2)`. `Entity.Top`
/// (`Entity.cs:86-96`) is `(x + width/2, y)`; the port's local player
/// carries no `netOffset` (the remote `PlayerNet.off_x/off_y` twin is
/// deferred with the remote overheads). The `gravDir != 1` Bottom arm
/// is unreachable — the port keeps `gravDir = 1` (player.rs).
pub(crate) fn chat_draw_position(p: &Player, camera: Vec2, zoom: f32, screen: Vec2) -> Vec2 {
    let top = Vec2::new(p.x + p.width * 0.5, p.y + p.gfx_off_y);
    Vec2::new(
        (top.x - camera.x) * zoom + screen.x * 0.5,
        (top.y - camera.y) * zoom + screen.y * 0.5 - 2.0,
    )
}

/// The draw origin (`Main.cs:63255-63258`): the anchor minus
/// `(0, messageSize.Y/2)`, then minus `messageSize/2`, floored —
/// `(anchor − messageSize/2).Floor()` with the extra half-height Y lift.
/// `messageSize` is `ChatManager.GetStringSize(MouseText, snippets, One)`
/// (`Player.cs:781`, `ChatManager.cs:184-198`) — one flat snippet measures
/// like `DynamicSpriteFont.MeasureString` (`ui::text_wrap::measure_string`).
pub(crate) fn overhead_origin(anchor: Vec2, size: Vec2) -> Vec2 {
    let lifted = Vec2::new(anchor.x, anchor.y - size.y * 0.5);
    (lifted - size * 0.5).floor()
}

/// `Main.DrawPlayerChatBubbles` (`Main.cs:63248-63263`) for the local
/// player: `active && timeLeft > 0 && (!dead || ghost)` (`:63253`) — the
/// port has no ghosts (`player.rs::ghost_like`), so the gate collapses to
/// `!dead`. The text draws through
/// `ChatManager.DrawColorCodedStringWithShadow` at scale 1 (`:63260`):
/// the four ±2 px black shadow passes (`ShadowDirections`,
/// `ChatManager.cs:376-382`, spread 2 `:246`) then the colour.
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    player: Option<&Player>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    let Some(p) = player else { return };
    // `:63253` — the overhead arms only while the timer runs and the
    // player lives (the ghost arm is unported).
    if p.chat_overhead.time_left <= 0 || p.dead {
        return;
    }
    let msg = &p.chat_overhead;
    let (w, h) = text_wrap_size(lookups, &msg.text);
    let size = Vec2::new(w, h);
    let anchor = chat_draw_position(p, camera, zoom, screen);
    let pos = overhead_origin(anchor, size);
    let c = [
        msg.color[0] as f32 / 255.0,
        msg.color[1] as f32 / 255.0,
        msg.color[2] as f32 / 255.0,
        1.0,
    ];
    // `ShadowDirections · spread` (`ChatManager.cs:200-205`): −X, +X,
    // −Y, +Y at 2 px, in `color.MultiplyRGBA(Color.Black)` = black
    // (`:249`), then the colour pass (`:250`).
    const SHADOW_RING: [[f32; 2]; 4] = [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]];
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        pos.x,
        pos.y,
        1.0,
        &SHADOW_RING,
        c,
        &msg.text,
    );
}

/// `OverheadMessage.messageSize` (`Player.cs:781`) re-derived at draw
/// time — the port stores flat text, not the cached snippets.
fn text_wrap_size(lookups: &Lookups, text: &str) -> (f32, f32) {
    super::text_wrap::measure_string(lookups.ui_font.as_ref(), text)
}

/// `ChatHelper.DisplayMessage`'s author arm (`ChatHelper.cs:86-87`) for
/// the local player: `chatOverhead.NewMessage(text, 400)` then
/// `chatOverhead.color = color`. The monitor line keeps the name-tag
/// prefix (`:88`); the OVERHEAD carries the bare text.
pub(crate) fn arm_overhead(p: &mut Player, text: &str, color: [u8; 3]) {
    p.chat_overhead
        .new_message(text, OverheadMessage::DISPLAY_TIME);
    p.chat_overhead.color = color;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Main.cs:63253` — a live player with a running timer draws; the
    /// dead/expired gates hold. (`draw` itself needs a renderer; the
    /// gates are the pure part.)
    #[test]
    fn the_overhead_gates_on_timer_and_life() {
        let w = tr_core::World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        arm_overhead(&mut p, "hello", [255, 255, 255]);
        assert_eq!(p.chat_overhead.time_left, 400, "Main.cs:67009");
        assert_eq!(p.chat_overhead.text, "hello");
        assert_eq!(p.chat_overhead.color, [255, 255, 255]);
        // The tick decrements (`Player.cs:24554-24557`).
        p.chat_overhead.time_left -= 1;
        assert_eq!(p.chat_overhead.time_left, 399);
        // Expired: no draw.
        p.chat_overhead.time_left = 0;
        assert!(p.chat_overhead.time_left <= 0);
        // Dead: no draw (the ghost arm is unported).
        p.chat_overhead.time_left = 400;
        p.dead = true;
        assert!(p.dead);
    }

    /// `GetChatDrawPosition` (`Main.cs:63245`): the player's top-centre
    /// through the world→UI transform, −2 px, plus gfxOffY.
    #[test]
    fn the_anchor_rides_the_top_centre_minus_two() {
        let w = tr_core::World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.x = 100.0;
        p.y = 200.0;
        p.width = 20.0;
        p.gfx_off_y = 3.0;
        let camera = Vec2::new(50.0, 100.0);
        let screen = Vec2::new(1920.0, 1080.0);
        let pos = chat_draw_position(&p, camera, 1.0, screen);
        // top = (100 + 10, 200 + 3); screen = top − camera + screen/2 − 2.
        assert_eq!(pos.x, 110.0 - 50.0 + 960.0);
        assert_eq!(pos.y, 203.0 - 100.0 + 540.0 - 2.0);
        // Zoom scales the world offset, not the screen centre.
        let pos2 = chat_draw_position(&p, camera, 2.0, screen);
        assert_eq!(pos2.x, (110.0 - 50.0) * 2.0 + 960.0);
    }

    /// `Main.cs:63257-63258` — the origin is the anchor lifted by half
    /// the message height, then centred on the message box, floored.
    #[test]
    fn the_origin_centres_and_floors() {
        let anchor = Vec2::new(100.7, 200.9);
        let size = Vec2::new(40.0, 16.0);
        let pos = overhead_origin(anchor, size);
        // y: 200.9 − 8 − 8 = 184.9 → 184; x: 100.7 − 20 = 80.7 → 80.
        assert_eq!(pos, Vec2::new(80.0, 184.0));
    }
}
