//! The `Sign` model's net-facing half — `Sign.ReadSign`
//! (`Sign.cs:27-58`), `Sign.TextSign` (`Sign.cs:74-85`) and the tile-origin
//! decode the `46`/`47` messages need (`MessageBuffer.cs:2366-2405`).
//!
//! The port stores signs as `World.signs: Vec<WldSign>` holding exactly the
//! non-null records in slot order (`tr-core/wld/sections/signs.rs`), so the
//! C#'s `Main.sign[i] == null` is "index past the store" and the array slot
//! index is the store index. The sign editor UI is not ported yet (S1-E-26)
//! — `Main.editSign`/`InputTextSign` stay absent — but the open/close latch
//! and its menu cues ARE ported (W4-e10 gap census: `OpenSign`/`CloseSign`
//! `Player.cs:33528-33551`, the `:33547`/`:33537` sites): the message layer,
//! the Escape chain and the per-tick range close all ride them.

use crate::app::App;
use tr_core::wld::sections::WldSign;
use tr_core::World;

/// `Main.sign[0..31999]` (`Sign.cs:8`, `Main.cs:10081-10084`): the array size
/// the `47` range check uses (`MessageBuffer.cs:2384`).
pub(crate) const MAX_SIGNS: i16 = 32_000;

/// `Main.tileSign` (`Main.cs:10081-10084`): the tile kinds a sign text may sit
/// on — 55 Sign, 85 Tattered Wood Sign, 425 Illuminant Hanging Sign, 573
/// Sunplate Sign (the same set as `wld/sections/signs.rs`'s private copy).
fn is_sign_tile(kind: u16) -> bool {
    matches!(kind, 55 | 85 | 425 | 573)
}

/// `Sign.ReadSign`'s origin decode (`Sign.cs:29-33`): the tile's frame picks
/// the top-left the record is stored at. `None` when the tile is not a sign
/// (the C#'s `!Main.tileSign[...] → KillSign, -1`).
pub(crate) fn origin(world: &World, i: i32, j: i32) -> Option<(i32, i32)> {
    let t = world.tile(i, j)?;
    let fx = t.frame_x / 18;
    let fy = t.frame_y / 18;
    let x = i - (fx % 2) as i32;
    let y = j - fy as i32;
    let origin = world.tile(x, y)?;
    (origin.active() && is_sign_tile(origin.kind)).then_some((x, y))
}

/// `Sign.ReadSign`'s slot lookup plus its create-if-missing arm
/// (`Sign.cs:41-58`): the index of the record at the coords, creating one with
/// an empty text when absent. `None` when the store is full (`l < 32000`
/// failing).
pub(crate) fn find_or_create(signs: &mut Vec<WldSign>, x: i32, y: i32) -> Option<usize> {
    if let Some(i) = signs.iter().position(|s| s.x == x && s.y == y) {
        return Some(i);
    }
    if signs.len() >= MAX_SIGNS as usize {
        return None;
    }
    signs.push(WldSign {
        text: String::new(),
        x,
        y,
    });
    Some(signs.len() - 1)
}

/// `Sign.TextSign` (`Sign.cs:74-85`): a dead or non-sign tile kills the record
/// (`Main.sign[i] = null` — the store splice); otherwise the text lands.
pub(crate) fn text_sign(world: &mut World, idx: usize, x: i32, y: i32, text: String) {
    let live = world
        .tile(x, y)
        .is_some_and(|t| t.active() && is_sign_tile(t.kind));
    if !live {
        if idx < world.signs.len() {
            world.signs.remove(idx);
        }
        return;
    }
    if let Some(s) = world.signs.get_mut(idx) {
        s.text = text;
    } else {
        world.signs.push(WldSign { text, x, y });
    }
}

/// `IngameUIWindows.CloseAll(quiet: true)` (`IngameUIWindows.cs:10-70`) —
/// the port's live windows, ALL quiet (no cue): the fullscreen map
/// (`:12-19`), the fancy-UI bestiary (`:28-31`), the talk latch + the chat
/// text (`:33-43`), the chest (`:50-57`), the inventory (`:58-65`), the
/// shop index + the crafting panel (`:66-68`). The hair/clothes/building/
/// capture/tile-entity windows have no port state.
pub(crate) fn close_all_windows_quiet(app: &mut App) {
    if let Some(m) = app.map.as_mut() {
        m.set_open(false);
    }
    app.bestiary_open = false;
    app.talk_npc = None;
    app.npc_chat_text.clear();
    // The chest arm (`:50-57`): the C# sets `chest = -1` BARE (no
    // stackSplit, no cue under quiet) — the sync snapshot catches up on
    // the next `tick_open`.
    if let Some(p) = app.player.as_mut() {
        p.chest = -1;
    }
    app.inventory_open = false;
    app.close_shop(); // `Main.SetNPCShopIndex(0)`'s port mapping
    app.crafting_open = false;
}

/// `Player.OpenSign` (`Player.cs:33542-33551`): `IngameUIWindows.CloseAll
/// (quiet: true)`, then for `s >= 0` the unpositioned `PlaySound(10)`
/// Menu_Open (`:33547`), the sign latch and `npcChatText =
/// Main.sign[s].text`. A negative index only closes the other windows; an
/// index past the store reads "" (the C# would throw on `Main.sign[s] ==
/// null` — the port's store is dense, so this is the defensive shape).
pub(crate) fn open_sign(app: &mut App, s: i32) {
    close_all_windows_quiet(app);
    if s >= 0 {
        crate::chest::play_menu_sound(app, 10); // `:33547`
        let text = app
            .world
            .signs
            .get(s as usize)
            .map(|r| r.text.clone())
            .unwrap_or_default();
        if let Some(p) = app.player.as_mut() {
            p.sign = s; // `:33548`
        }
        app.npc_chat_text = text; // `:33549`
    }
}

/// `Player.CloseSign` (`Player.cs:33528-33540`): with a live sign latch,
/// `sign = -1`, `Main.editSign = false` (no port editor — S1-E-26), the
/// chat text empties, and unless `quiet` the unpositioned `PlaySound(11)`
/// Menu_Close (`:33537`). A closed latch is a no-op — no cue.
pub(crate) fn close_sign(app: &mut App, quiet: bool) {
    let live = app.player.as_ref().is_some_and(|p| p.sign > -1);
    if !live {
        return; // `:33530`
    }
    if let Some(p) = app.player.as_mut() {
        p.sign = -1; // `:33532`
    }
    app.npc_chat_text.clear(); // `:33534`
    if !quiet {
        crate::chest::play_menu_sound(app, 11); // `:33537`
    }
}

/// The per-tick sign range close (`Player.cs:27504-27519`, right after the
/// talk-NPC close): while `sign >= 0`, a record gone or its 32×32 box no
/// longer intersecting `TileReachCheckSettings.Simple.GetWorldRegion`
/// closes the sign WITH the cue (`CloseSign()` default quiet false). The
/// region is the `chat_world_region` shape (`TileReachCheckSettings.cs:
/// 45-71` — the same ±5/±3 tiles the talk close uses, the Simple settings'
/// `TileRangeMultiplier = 1`/`TileReachLimit = 20`); the C# try/catch's
/// null-record arm is the store-past-end probe.
pub(crate) fn tick_sign_range_close(app: &mut App) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    if p.sign < 0 {
        return;
    }
    let (lx, ly, hx, hy) = crate::world_sim::npcs::chat_world_region(p);
    let inside = app.world.signs.get(p.sign as usize).is_some_and(|s| {
        let (sx, sy) = (s.x as f32 * 16.0, s.y as f32 * 16.0);
        sx < hx && sx + 32.0 > lx && sy < hy && sy + 32.0 > ly
    });
    if !inside {
        close_sign(app, false); // `:27512`/:27517
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn sign_world() -> World {
        let mut w = World::new("t".into(), 64, 64);
        let mut t = Tile::default();
        t.kind = 55;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = 18; // the right half of a 2-wide sign
        // A 2-wide sign: both halves are active sign tiles (`Sign.cs:29-33`
        // resolves the origin from the frame, then checks it).
        for x in [19, 20] {
            if let Some(cell) = w.tile_mut(x, 30) {
                *cell = t;
                cell.frame_x = if x == 19 { 0 } else { 18 };
            }
        }
        w
    }

    #[test]
    fn the_sign_origin_follows_the_frame() {
        let w = sign_world();
        assert_eq!(origin(&w, 20, 30), Some((19, 30)));
        assert_eq!(origin(&w, 21, 30), None, "not a sign tile");
    }

    #[test]
    fn the_sign_record_is_created_then_written() {
        let mut w = sign_world();
        let idx = find_or_create(&mut w.signs, 19, 30).unwrap();
        assert_eq!(w.signs[idx].text, "");
        text_sign(&mut w, idx, 19, 30, "hi".into());
        assert_eq!(w.signs[idx].text, "hi");
        // A dead tile kills the record (`Sign.cs:77-80`) — both halves, since
        // the origin (19, 30) is what `TextSign` checks.
        for x in [19, 20] {
            if let Some(cell) = w.tile_mut(x, 30) {
                *cell = Tile::default();
            }
        }
        text_sign(&mut w, idx, 19, 30, "gone".into());
        assert!(w.signs.is_empty());
    }

    // ---- W4-e10 gap census: the open/close menu-tails (WRITTEN NOT RUN) ----

    /// An App with a live player and one sign record at (19, 30).
    fn sign_app() -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = sign_world();
        app.player = Some(crate::player::Player::spawn(&app.world));
        let idx = find_or_create(&mut app.world.signs, 19, 30).unwrap();
        app.world.signs[idx].text = "hello".into();
        app
    }

    /// W4-e10 — `Player.OpenSign` (`Player.cs:33542-33551`): ONE Menu_Open
    /// cue (`:33547`), the latch, the record's text into `npcChatText`;
    /// the CloseAll(quiet) prologue clears the chest/talk/inventory with
    /// NO extra cue. A negative index closes windows only — no cue, no
    /// latch.
    #[test]
    fn w4_e10_open_sign_latches_and_plays_one_menu_open() {
        let mut app = sign_app();
        // The prologue's victims: an open chest + talk latch + inventory.
        app.player.as_mut().unwrap().chest = 3;
        app.talk_npc = Some(0);
        app.inventory_open = true;
        let before = app.audio.played_count();
        open_sign(&mut app, 0);
        assert_eq!(app.audio.played_count(), before + 1, "`:33547` once");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.sign, 0, "`:33548`");
        assert_eq!(p.chest, -1, "CloseAll's chest arm (quiet)");
        assert_eq!(app.talk_npc, None, "CloseAll's talk arm (quiet)");
        assert!(!app.inventory_open, "CloseAll's inventory arm (quiet)");
        assert_eq!(app.npc_chat_text, "hello", "`:33549`");
        // The negative index: windows close, no cue, no latch.
        app.player.as_mut().unwrap().chest = 2;
        let mid = app.audio.played_count();
        open_sign(&mut app, -1);
        assert_eq!(app.audio.played_count(), mid, "no cue for s < 0");
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
    }

    /// W4-e10 — `Player.CloseSign` (`Player.cs:33528-33540`): a live latch
    /// clears + empties the text + ONE Menu_Close (`:33537`); `quiet`
    /// suppresses the cue only; a closed latch is a draw/cue-free no-op.
    #[test]
    fn w4_e10_close_sign_plays_one_menu_close_unless_quiet() {
        let mut app = sign_app();
        open_sign(&mut app, 0);
        let before = app.audio.played_count();
        close_sign(&mut app, false);
        assert_eq!(app.audio.played_count(), before + 1, "`:33537`");
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "`:33532`");
        assert!(app.npc_chat_text.is_empty(), "`:33534`");
        // The closed latch: no cue.
        let mid = app.audio.played_count();
        close_sign(&mut app, false);
        assert_eq!(app.audio.played_count(), mid, "`:33530` no-op");
        // Quiet: state clears, no cue.
        open_sign(&mut app, 0);
        let mid = app.audio.played_count();
        close_sign(&mut app, true);
        assert_eq!(app.audio.played_count(), mid, "quiet suppresses");
        assert_eq!(app.player.as_ref().unwrap().sign, -1);
    }

    /// W4-e10 — the per-tick range close (`Player.cs:27504-27519`): the
    /// record in reach keeps the sign; a record gone (the try/catch arm)
    /// or out of the Simple region closes WITH the cue.
    #[test]
    fn w4_e10_the_sign_range_close_fires_on_gone_or_far_records() {
        let mut app = sign_app();
        // Stand the player beside the sign (19*16, 30*16).
        {
            let p = app.player.as_mut().unwrap();
            p.x = 19.0 * 16.0;
            p.y = 30.0 * 16.0;
        }
        open_sign(&mut app, 0);
        let before = app.audio.played_count();
        tick_sign_range_close(&mut app);
        assert_eq!(app.player.as_ref().unwrap().sign, 0, "in reach: stays");
        assert_eq!(app.audio.played_count(), before, "no cue in reach");
        // Far record: the region no longer intersects — close + cue.
        app.world.signs[0].x = 500;
        tick_sign_range_close(&mut app);
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "`:27512`");
        assert_eq!(app.audio.played_count(), before + 1, "the close cue");
        // Gone record (the null/catch arm): re-open, splice, close + cue.
        app.world.signs[0].x = 19;
        open_sign(&mut app, 0);
        app.world.signs.clear();
        let mid = app.audio.played_count();
        tick_sign_range_close(&mut app);
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "`:27517`");
        assert_eq!(app.audio.played_count(), mid + 1);
    }
}
