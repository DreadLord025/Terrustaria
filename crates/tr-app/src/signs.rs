//! The `Sign` model — `Sign.ReadSign` (`Sign.cs:27-58`), `Sign.TextSign`
//! (`Sign.cs:74-85`), the tile-origin decode the `46`/`47` messages need
//! (`MessageBuffer.cs:2366-2405`) and the local editor half (W4-f22):
//! `Player.OpenSign`/`CloseSign` (`Player.cs:33542-33551`/`:33528-33540`),
//! the `tileSign` right-click arm (`Player.cs:32693-32724`),
//! `Main.SubmitSignText`/`InputTextSignCancel` (`Main.cs:40694-40704`/
//! `:19817-19826`) and `IngameFancyUI.OpenVirtualKeyboard(1)`
//! (`IngameFancyUI.cs:86-145` — the ctx-1 keyboard, `UI.EnterMessage`,
//! `allowEmpty: false`, **maxLength 1200**).
//!
//! The port stores signs as `World.signs: Vec<WldSign>` holding exactly the
//! non-null records in slot order (`tr-core/wld/sections/signs.rs`), so the
//! C#'s `Main.sign[i] == null` is "index past the store" and the array slot
//! index is the store index. `Main.editSign` is exactly "the sign keyboard
//! session is open" (`App::sign_editing`, the chest twin's shape).

use tr_core::wld::sections::WldSign;
use tr_core::World;

use crate::app::flow::KeyboardTail;
use crate::app::App;
use crate::ui::keyboard::{Context, Request};

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

/// `Sign.KillSign` (`Sign.cs:13-22`): every record at the coords dies (the
/// C# nulls in place; the port's dense store splices, the `text_sign` shape).
pub(crate) fn kill_sign(signs: &mut Vec<WldSign>, x: i32, y: i32) {
    signs.retain(|s| !(s.x == x && s.y == y));
}

/// The lazy kill-on-break record sweep (`WorldGen.CheckSign`'s destroy
/// tail, `WorldGen.cs:43686-43689` — `if (type != 395) Sign.KillSign(num7,
/// num8)`): a broken sign tile loses its record at the object's ORIGIN.
/// The port has no `CheckSign` cascade arm (the remaining cells float —
/// the pre-existing deviation), so the sweep is LAZY: the hit cell's
/// pre-kill frames decode the origin (`WorldGen.cs:43573-43578` —
/// `frameX/18 % 2`, `frameY/18 % 2`, the same decode `Sign.ReadSign`
/// uses for the 2x2 sign) and `Sign.KillSign` runs there. The 395 item
/// frame shares `CheckSign` but is not a `tileSign` kind, so it never
/// reaches this sweep (`type != 395`, `:43686`).
///
/// The C# nulls the slot in place; the port's dense store SPLICES, which
/// shifts every later index — so the open `Player.sign` and `signHover`
/// indices are remapped through the surviving records (the C# indices
/// are stable; the port keeps the same record open). The editor close
/// itself is the lazy `sign_tick` (`Player.cs:27509-27513` — `sign ==
/// null -> CloseSign`), which also kills the keyboard through the
/// `Main.editSign` gate (`Main.cs:17389-17394`); the remap keeps a
/// DIFFERENT open sign's index valid across the splice.
pub(crate) fn kill_broken_sign_record(
    app: &mut App,
    kind: u16,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
) {
    if !is_sign_tile(kind) {
        return;
    }
    let ox = x - i32::from(frame_x) / 18 % 2;
    let oy = y - i32::from(frame_y) / 18 % 2;
    // The pre-kill identities of the open/hovered records (the dense-store
    // index remap — see the fn doc).
    let open = app.player.as_ref().map_or(-1, |p| p.sign);
    let open_key = if open >= 0 {
        app.world.signs.get(open as usize).map(|s| (s.x, s.y))
    } else {
        None
    };
    let hover_key = if app.sign_hover >= 0 {
        app.world
            .signs
            .get(app.sign_hover as usize)
            .map(|s| (s.x, s.y))
    } else {
        None
    };
    kill_sign(&mut app.world.signs, ox, oy); // `:43688`
    // The dense-store index remap. A SURVIVING open/hovered record keeps
    // its identity (the C# slot index is stable; the splice shifts it).
    // A KILLED open record becomes a NULL SLOT — an index past the store,
    // the module's `Main.sign[i] == null` convention — so the lazy
    // `sign_tick` (`Player.cs:27509-27513`) sees the missing record and
    // runs `CloseSign()` (sound 11, the dialog clear, the editor gate),
    // exactly like the C# null-in-place + next-tick close. A killed hover
    // clears to -1 (`MouseOversClear` re-derives it every update anyway).
    let remap = |key: Option<(i32, i32)>, null_slot: i32| -> i32 {
        match key {
            Some((x, y)) => app
                .world
                .signs
                .iter()
                .position(|s| s.x == x && s.y == y)
                .map_or(null_slot, |i| i as i32),
            None => -1,
        }
    };
    let null_slot = app.world.signs.len() as i32;
    let new_open = if open >= 0 {
        remap(open_key, null_slot)
    } else {
        -1
    };
    if let Some(p) = app.player.as_mut() {
        p.sign = new_open;
    }
    app.sign_hover = remap(hover_key, -1);
}

/// `Sign.ReadSign` (`Sign.cs:24-60`): the frame decode picks the top-left,
/// a non-sign origin kills the record and answers -1, the slot scan finds
/// the record, and `CreateIfMissing` claims the first free slot with an
/// empty text. -1 when the store is full.
pub(crate) fn read_sign(world: &mut World, i: i32, j: i32, create_if_missing: bool) -> i32 {
    let Some(t) = world.tile(i, j) else {
        return -1;
    };
    let (fx, fy) = (t.frame_x / 18, t.frame_y / 18);
    let (ox, oy) = (i - (fx % 2) as i32, j - fy as i32);
    let sign_tile = world
        .tile(ox, oy)
        .is_some_and(|t| t.active() && is_sign_tile(t.kind));
    if !sign_tile {
        kill_sign(&mut world.signs, ox, oy); // `:30-34`
        return -1;
    }
    if let Some(idx) = world.signs.iter().position(|s| s.x == ox && s.y == oy) {
        return idx as i32; // `:36-43`
    }
    if create_if_missing {
        if let Some(idx) = find_or_create(&mut world.signs, ox, oy) {
            return idx as i32; // `:44-58`
        }
    }
    -1
}

/// `Player.OpenSign` (`Player.cs:33542-33551`): `IngameUIWindows.CloseAll
/// (quiet: true)` first (the port's live pieces: the fullscreen map, the
/// options window, the emotes wheel and the talk latch — the building mode
/// and capture manager have no port state), then for `s >= 0` sound 10, the
/// index latch and `Main.npcChatText = Main.sign[s].text` (the read
/// display).
pub(crate) fn open_sign(app: &mut App, s: i32) {
    if let Some(m) = app.map.as_mut() {
        m.set_open(false);
    }
    app.ingame_options.close();
    app.emotes.close();
    if app.bestiary_open {
        // `IngameFancyUI.Close(quiet)` (`IngameUIWindows.CloseAll`'s
        // `Main.inFancyUI` arm) — the port's bestiary rides the same flag.
        app.bestiary_open = false;
        app.bestiary_selected = None;
    }
    app.talk_npc = None;
    app.npc_chat_text.clear();
    if s >= 0 {
        crate::chest::play_menu_sound(app, 10); // `:33547`
        if let Some(p) = app.player.as_mut() {
            p.sign = s; // `:33548`
        }
        app.npc_chat_text = app
            .world
            .signs
            .get(s as usize)
            .map(|r| r.text.clone())
            .unwrap_or_default(); // `:33549`
    }
}

/// `Player.CloseSign` (`Player.cs:33528-33540`): with a sign open, the index
/// clears, `Main.editSign = false` (the port's sign keyboard session closes
/// — the `Main.cs:17389-17394` gate's shape), `npcChatText` empties and
/// sound 11 plays unless `quiet`.
pub(crate) fn close_sign(app: &mut App, quiet: bool) {
    let open = app.player.as_ref().is_some_and(|p| p.sign > -1);
    if !open {
        return;
    }
    if let Some(p) = app.player.as_mut() {
        p.sign = -1; // `:33532`
    }
    if app.sign_editing() {
        app.front = None; // `:33533` — `Main.editSign = false`
    }
    app.npc_chat_text.clear(); // `:33534`
    if !quiet {
        crate::chest::play_menu_sound(app, 11); // `:33537`
    }
}

/// The `Main.tileSign` right-click arm (`Player.cs:32693-32724`): every
/// active sign tile consumes the click (`flag2 = true`, `:32695`). A
/// re-click on the OPEN sign closes it (`:32697-32701`); otherwise single
/// player opens `Sign.ReadSign`'s record (`:32704-32707`) and the MP client
/// asks the host with `46` at the frame-decoded origin (`:32710-32721`).
/// The host resolves locally — the C#'s `46 -> 47` buffer-256 loopback's
/// net effect (the port has no self-loopback, the chest `31` precedent).
pub(crate) fn try_open_sign_at_cursor(app: &mut App, world_pos: glam::Vec2) -> bool {
    let x = (world_pos.x / tr_core::TILE_PX).floor() as i32;
    let y = (world_pos.y / tr_core::TILE_PX).floor() as i32;
    let Some(t) = app.world.tile(x, y) else {
        return false;
    };
    if !t.active() || !is_sign_tile(t.kind) {
        return false;
    }
    // The frame decode copies out before the mutable `read_sign` borrow.
    let (fx, fy) = (t.frame_x / 18, t.frame_y / 18);
    let current = app.player.as_ref().map_or(-1, |p| p.sign);
    if current >= 0 && read_sign(&mut app.world, x, y, false) == current {
        close_sign(app, false); // `:32699`
        return true;
    }
    if let crate::app::NetRole::Client(client) = &app.net {
        // `:32710-32720` — the origin decode (the `while num40 > 1` loop is
        // `fx % 2`) and the `46` request.
        let (ox, oy) = (x - (fx % 2) as i32, y - fy as i32);
        if app
            .world
            .tile(ox, oy)
            .is_some_and(|o| o.active() && is_sign_tile(o.kind))
        {
            client.send(&crate::net::C2s::SignRequest(crate::net::SignRequest {
                x: ox as i16,
                y: oy as i16,
            })); // `:32720`
        }
        return true;
    }
    let idx = read_sign(&mut app.world, x, y, true); // `:32706`
    open_sign(app, idx);
    true
}

/// `IngameFancyUI.OpenVirtualKeyboard(1)` (`IngameFancyUI.cs:86-145`):
/// `ClearChat` (`:53-57` — the player chat closes), sound 12 (`:90`),
/// `Main.editSign = true` (`:95`) and the ctx-1 keyboard with the
/// `UI.EnterMessage` label, the sign text prefill, `allowEmpty: false` and
/// **`maxLength: 1200`** (`:145` — NOT 500; the census pin). The port's
/// `editSign` is exactly the session, so the keyboard open IS the flag.
/// Deviation: `CanShowVirtualKeyboard(1)`'s `screenHeight > 700` gate
/// (`UIVirtualKeyboard.cs:1022-1029`) is not ported — the C# falls back to
/// the inline `Main.InputTextSign` editor on short screens, which the port
/// does not carry, so the keyboard opens at every height (ANALYSIS "The
/// sign keyboard (W4-f22)").
pub(crate) fn open_sign_keyboard(app: &mut App) {
    app.chat.open = false; // `ClearChat` -> `Main.ClosePlayerChat`
    app.chat.text.clear();
    crate::chest::play_menu_sound(app, 12); // `:90`
    let initial = app.npc_chat_text.clone(); // `:137` — `Main.npcChatText`
    app.open_keyboard_with_tail(
        Request {
            label: crate::ui::keyboard::enter_message(), // `:96`
            initial,
            allow_empty: false,
            max_length: SIGN_MAX_TEXT_LENGTH,
            context: Context::Sign,
        },
        None,
        KeyboardTail::SignEdit,
    );
}

/// The sign keyboard's `SetTextMaxLength` argument (`IngameFancyUI.cs:145`
/// — the literal `1200`; the ctor default 20 is the chest/text contexts').
pub(crate) const SIGN_MAX_TEXT_LENGTH: usize = 1200;

/// `Main.SubmitSignText` (`Main.cs:40694-40704`) + the submit closure's
/// `IngameFancyUI.Close(quiet: true)` (`IngameFancyUI.cs:137-140`): sound
/// 12, `Sign.TextSign(player.sign, Main.npcChatText)`, `editSign = false`,
/// and the netMode-1 client sends `47` with the sign index (`number2/3`
/// default 0 — the sender byte 0, flags 0). `text` is the live keyboard
/// text — `CopyTextToSign`'s per-tick `npcChatText` sync
/// (`UIVirtualKeyboard.cs:1049-1059`); the closure ignores its trimmed
/// argument, so the untrimmed text lands.
pub(crate) fn submit_sign_text(app: &mut App, text: &str) {
    crate::chest::play_menu_sound(app, 12); // `:40696`
    let idx = app.player.as_ref().map_or(-1, |p| p.sign);
    let coords = if idx >= 0 {
        app.world.signs.get(idx as usize).map(|s| (s.x, s.y))
    } else {
        None
    };
    let Some((x, y)) = coords else {
        app.front = None; // the editor closes even without a record
        return;
    };
    text_sign(&mut app.world, idx as usize, x, y, text.to_string()); // `:40698`
    app.front = None; // `:40699` + `Close(quiet: true)`
    if let crate::app::NetRole::Client(client) = &app.net {
        client.send(&crate::net::C2s::SignData(crate::net::SignData {
            sign: idx as i16,
            x: x as i16,
            y: y as i16,
            text: text.to_string(),
            sender: 0, // `SendData(47, -1, -1, null, num)` — number2 = 0
            flags: 0,
        })); // `:40702`
    }
}

/// `Main.InputTextSignCancel` (`Main.cs:19817-19826`) + the cancel closure's
/// `IngameFancyUI.Close(quiet: true)` (`IngameFancyUI.cs:141-144`):
/// `editSign = false`, `CacheCanceledInput(1)` parks the LIVE text in
/// `_cancelCacheSign` (`UIVirtualKeyboard.cs:1031-1037` — the restore key's
/// source), then `npcChatText` re-reads the stored sign text. `live_text`
/// is the keyboard's text at the cancel (the `npcChatText` sync).
/// `Main.blockKey` (`:19820`) has no port route.
pub(crate) fn input_text_sign_cancel(app: &mut App, live_text: &str) {
    app.sign_cancel_cache = live_text.to_string(); // `:19821`
    app.front = None; // `:19819` + `Close(quiet: true)`
    if let Some(p) = app.player.as_ref() {
        if p.sign >= 0 {
            app.npc_chat_text = app
                .world
                .signs
                .get(p.sign as usize)
                .map(|s| s.text.clone())
                .unwrap_or_default(); // `:19822-19825`
        }
    }
}

/// `UIVirtualKeyboard.RestoreCanceledInput(1)` (`UIVirtualKeyboard.cs:
/// 1039-1047`): the cached text returns to `npcChatText` AND the keyboard
/// box, and the cache empties (the bottom row rebuilds without the restore
/// key, `:481-484`).
pub(crate) fn restore_canceled_sign_input(app: &mut App) {
    let cached = std::mem::take(&mut app.sign_cancel_cache);
    app.npc_chat_text = cached.clone(); // `:1043`
    if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
        s.state.text = cached
            .chars()
            .take(s.state.max_length)
            .collect(); // `:1044` — `this.Text = ...` -> `SetText`'s trim
    }
}

/// `Main.MouseOversClear`'s sign half (`Main.cs:46511-46512`, called at
/// the update head `:17035`): `signHover = -1`, `signBubble = false` every
/// update — the scan below re-latches them. The `cursorItemIcon*` clears
/// (`:46507-46510`) have no port state (the cursor-icon system is unported).
pub(crate) fn mouse_overs_clear(app: &mut App) {
    app.sign_hover = -1;
    app.sign_bubble = false;
}

/// The sign half of `Player.TileInteractionsCheckLongDistance`
/// (`Player.cs:32098-32115`, reached from `LookForTileInteractions`
/// `:31962-31970` — the long-distance arm runs even OUT of reach): a
/// hovered sign tile latches `noThrow = 2` (`:32100`), the bubble anchor
/// `signX/signY` at the frame-decoded origin (`:32101-32107` —
/// `frameX/18 % 2`, `frameY/18`, the 2x2 sign's top-left), and
/// `Sign.ReadSign(CreateIfMissing: false)` (`:32108`) answers the record
/// index. The `signHover` latch (`:32109-32114`) additionally needs the
/// cursor INSIDE the origin's 2x2 box (`tileTargetX/Y` within
/// `[num5, num5+1] x [num6, num6+1]`).
///
/// The C# `cursorItemIconEnabled = false` / `cursorItemIconID = -1`
/// (`:32112-32113`) have no port state (the cursor-icon system is
/// unported — the tooltip's `!mouseText` gate covers the visible effect).
/// The smart-interact target override (`:31930-31934`) is unported: the
/// port's cursor cell IS `tileTargetX/Y`.
pub(crate) fn sign_hover_scan(app: &mut App, world_pos: glam::Vec2) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    if p.dead {
        return; // `UpdateDead` returns before `LookForTileInteractions`
    }
    if app.map.as_ref().is_some_and(|m| m.open) {
        return; // `Main.mapFullscreen` (`:31924`)
    }
    if app.bestiary_open {
        return; // `Main.InGameUI.CurrentState == Main.BestiaryUI` (`:31924`)
    }
    let x = (world_pos.x / tr_core::TILE_PX).floor() as i32;
    let y = (world_pos.y / tr_core::TILE_PX).floor() as i32;
    let Some(t) = app.world.tile(x, y) else {
        return; // `!WorldGen.InWorld` (`:32046`)
    };
    if !t.active() || !is_sign_tile(t.kind) {
        return;
    }
    app.no_throw = 2; // `:32100`
    let (fx, fy) = (t.frame_x / 18, t.frame_y / 18);
    let ox = x - (fx % 2) as i32;
    let oy = y - fy as i32;
    app.sign_bubble = true; // `:32105`
    app.sign_x = ox * 16 + 16; // `:32106`
    app.sign_y = oy * 16; // `:32107`
    let idx = read_sign(&mut app.world, x, y, false); // `:32108`
    if idx != -1 && x >= ox && y >= oy && x <= ox + 1 && y <= oy + 1 {
        app.sign_hover = idx; // `:32111`
    }
}

/// The per-tick sign close (`Player.cs:27504-27518`): a missing record or a
/// sign whose 32x32 px box leaves the `TileReachCheckSettings.Simple` world
/// region closes with sound 11 (`CloseSign()` — not quiet). The
/// `Main.editSign` gate (`Main.cs:17389-17394`): the editor dies when the
/// index clears. Runs beside `chest::tick_open` in the player tick.
pub(crate) fn sign_tick(app: &mut App) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    if p.sign >= 0 {
        let record = app.world.signs.get(p.sign as usize).map(|s| (s.x, s.y));
        let in_range = record.is_some_and(|(sx, sy)| {
            // `TileReachCheckSettings.Simple.GetWorldRegion`
            // (`TileReachCheckSettings.cs:45-54/:62-71`): the tile region
            // under the live reach statics, scaled to px with the +15 tail.
            let (rx, ry) = p.tile_interaction_reach();
            let lx = ((p.x / 16.0) as i32 - rx) * 16;
            let ly = ((p.y / 16.0) as i32 - ry) * 16;
            let hx = (((p.x + crate::player::WIDTH) / 16.0).ceil() as i32 - 1 + rx) * 16 + 15;
            let hy = (((p.y + p.height) / 16.0).ceil() as i32 - 1 + ry) * 16 + 15;
            // `Rectangle.Intersects` over `(sign.x*16, sign.y*16, 32, 32)`.
            let (bx, by) = (sx * 16, sy * 16);
            bx < hx && bx + 32 > lx && by < hy && by + 32 > ly
        });
        if !in_range {
            close_sign(app, false); // `:27512`/:27517
        }
    }
    // `Main.cs:17389-17394` — `editSign && sign == -1` clears the editor.
    if app.sign_editing() && app.player.as_ref().is_some_and(|p| p.sign == -1) {
        app.front = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::flow::{FrontScreen, KeyboardTail};
    use crate::player::Player;
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

    /// A headless App over the sign world with a spawned player standing
    /// next to the sign (the `interact_app` shape, chest.rs).
    fn sign_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = sign_world();
        app.player = Some(Player::spawn(&app.world));
        if let Some(p) = app.player.as_mut() {
            // Stand within the sign's reach (the spawn is far away in the
            // 8x8 flow world replaced above).
            p.x = 19.0 * 16.0;
            p.y = 28.0 * 16.0;
        }
        app
    }

    fn click(x: i32, y: i32) -> glam::Vec2 {
        glam::Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0)
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

    /// W4-f22 — `Sign.ReadSign` (`Sign.cs:24-60`): the create-if-missing
    /// arm claims the slot, the non-create arm answers -1 on a miss, and a
    /// non-sign tile kills the record and answers -1.
    #[test]
    fn read_sign_creates_only_when_asked() {
        let mut w = sign_world();
        assert_eq!(read_sign(&mut w, 20, 30, false), -1, "no record yet");
        assert!(w.signs.is_empty());
        assert_eq!(read_sign(&mut w, 20, 30, true), 0, "the first free slot");
        assert_eq!(read_sign(&mut w, 19, 30, false), 0, "the left half resolves");
        // A non-sign tile kills the record (`:30-34`).
        if let Some(cell) = w.tile_mut(19, 30) {
            *cell = Tile::default();
        }
        assert_eq!(read_sign(&mut w, 20, 30, true), -1);
        assert!(w.signs.is_empty(), "KillSign ran");
    }

    /// W4-f22 — the right-click arm (`Player.cs:32693-32724`): the first
    /// click opens (sound 10, the index latch, the text read), the re-click
    /// on the SAME sign closes (sound 11, the text clear).
    #[test]
    fn the_right_click_opens_then_closes_the_sign() {
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        assert_eq!(app.player.as_ref().unwrap().sign, 0, "OpenSign latched");
        assert_eq!(app.npc_chat_text, "", "the fresh record reads empty");
        // Write a text, close, re-open: the read display returns it.
        text_sign(&mut app.world, 0, 19, 30, "hello".into());
        assert!(try_open_sign_at_cursor(&mut app, click(19, 30)));
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "the re-click closed");
        assert!(app.npc_chat_text.is_empty(), "CloseSign cleared the text");
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        assert_eq!(app.npc_chat_text, "hello", "OpenSign re-read the record");
        // A non-sign tile does not consume the click.
        assert!(!try_open_sign_at_cursor(&mut app, click(5, 5)));
    }

    /// W4-f22 — `IngameFancyUI.OpenVirtualKeyboard(1)` (`IngameFancyUI.cs:
    /// 136-145`): the ctx-1 request shape — `UI.EnterMessage`, the sign
    /// text prefill, `allowEmpty: false`, **maxLength 1200** (the census
    /// pin — NOT 500), the SignEdit tail, no back screen.
    #[test]
    fn the_sign_keyboard_opens_with_the_1200_cap() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        text_sign(&mut app.world, 0, 19, 30, "stored".into());
        app.npc_chat_text = "stored".into();
        open_sign_keyboard(&mut app);
        let Some(FrontScreen::Keyboard(s)) = app.front.as_ref() else {
            panic!("the sign keyboard is live");
        };
        assert_eq!(s.request.label, crate::ui::keyboard::enter_message());
        assert_eq!(s.request.initial, "stored", "the npcChatText prefill");
        assert_eq!(s.state.text, "stored");
        assert!(!s.request.allow_empty, "`:145` — allowEmpty: false");
        assert_eq!(s.request.max_length, 1200, "`:145` — the literal 1200");
        assert_eq!(s.request.context, crate::ui::keyboard::Context::Sign);
        assert_eq!(s.tail, KeyboardTail::SignEdit);
        assert!(s.back.is_none(), "the sign tail returns to gameplay");
        assert!(app.sign_editing(), "Main.editSign == the session");
        assert!(app.menu.is_none(), "the single-mode invariant holds");
    }

    /// W4-f22 — `Main.SubmitSignText` (`Main.cs:40694-40704`): the live
    /// text lands on the record, the editor closes, the sign stays open
    /// (the dialog keeps reading). The 1200 cap trims through the
    /// keyboard's `write_str` (`SetTextMaxLength`, `:428`).
    #[test]
    fn the_submit_writes_the_record_and_closes_the_editor() {
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        open_sign_keyboard(&mut app);
        if let Some(FrontScreen::Keyboard(s)) = app.front.as_mut() {
            crate::ui::keyboard::write_str(&mut s.state, "line one\nline two");
        }
        // The 1200 cap: a 1300-char write trims to 1200.
        if let Some(FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            crate::ui::keyboard::write_str(&mut s.state, &"x".repeat(1300));
            assert_eq!(s.state.text.chars().count(), 1200, "the ctor cap");
            s.state.text = "line one\nline two".into();
        }
        app.keyboard_submit();
        assert!(!app.sign_editing(), "editSign = false (`:40699`)");
        assert_eq!(app.world.signs[0].text, "line one\nline two");
        assert_eq!(app.player.as_ref().unwrap().sign, 0, "the sign stays open");
    }

    /// W4-f22 — `Main.InputTextSignCancel` (`Main.cs:19817-19826`): the
    /// cancel parks the LIVE text in `_cancelCacheSign`, the record keeps
    /// its stored text, `npcChatText` re-reads it, and the restore key
    /// (`RestoreCanceledInput`, `UIVirtualKeyboard.cs:1039-1047`) returns
    /// the cache to the box and empties it.
    #[test]
    fn the_cancel_caches_and_the_restore_spends_it() {
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        text_sign(&mut app.world, 0, 19, 30, "stored".into());
        app.npc_chat_text = "stored".into();
        open_sign_keyboard(&mut app);
        if let Some(FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text = "draft".into();
        }
        app.keyboard_cancel();
        assert!(!app.sign_editing(), "the editor closed");
        assert_eq!(app.sign_cancel_cache, "draft", "CacheCanceledInput(1)");
        assert_eq!(app.world.signs[0].text, "stored", "the cancel never wrote");
        assert_eq!(app.npc_chat_text, "stored", "`:19824` — the re-read");
        // Re-open: the cache is non-empty, so CanRestore holds.
        open_sign_keyboard(&mut app);
        assert!(!app.sign_cancel_cache.is_empty());
        restore_canceled_sign_input(&mut app);
        assert!(app.sign_cancel_cache.is_empty(), "the cache emptied");
        assert_eq!(app.npc_chat_text, "draft", "`:1043`");
        let Some(FrontScreen::Keyboard(s)) = app.front.as_ref() else {
            panic!("the keyboard stayed live");
        };
        assert_eq!(s.state.text, "draft", "`:1044` — this.Text = ...");
    }

    /// W4-f22 — the per-tick range close (`Player.cs:27504-27518`): a
    /// player walking out of the sign's 32x32-box reach closes with the
    /// dialog clear; the `Main.editSign` gate (`Main.cs:17389-17394`)
    /// kills the editor when the index clears.
    #[test]
    fn walking_out_of_range_closes_the_sign_and_the_editor() {
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        open_sign_keyboard(&mut app);
        assert!(app.sign_editing());
        // Teleport far away — outside the Simple region.
        if let Some(p) = app.player.as_mut() {
            p.x = 60.0 * 16.0;
            p.y = 60.0 * 16.0;
        }
        sign_tick(&mut app);
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "CloseSign ran");
        assert!(!app.sign_editing(), "the editSign gate cleared the editor");
        assert!(app.npc_chat_text.is_empty());
    }

    /// W4-f22 — `Player.KillMe`'s dead tick (`Player.cs:17014`): the sign
    /// index clears; the next `sign_tick` kills the editor.
    #[test]
    fn the_dead_tick_drops_the_sign() {
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        open_sign_keyboard(&mut app);
        if let Some(p) = app.player.as_mut() {
            p.update_dead(false, false, false, false);
        }
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "`:17014`");
        sign_tick(&mut app);
        assert!(!app.sign_editing(), "the editSign gate (`Main.cs:17391-17393`)");
    }

    /// W4-f22 — the sign hover/bubble latch (`Player.cs:32098-32115`): a
    /// hovered sign tile sets `noThrow = 2`, the bubble anchor at the
    /// frame-decoded origin (`signX = ox*16+16`, `signY = oy*16`), and
    /// `signHover` to the record index when the cursor is inside the
    /// origin's 2x2 box. `MouseOversClear` (`Main.cs:46511-46512`) clears
    /// both latches each update.
    #[test]
    fn the_hover_scan_latches_the_bubble_and_the_hover() {
        let mut app = sign_app();
        // A record at the origin so ReadSign(CreateIfMissing:false) finds it.
        let idx = find_or_create(&mut app.world.signs, 19, 30).unwrap();
        text_sign(&mut app.world, idx, 19, 30, "hello".into());
        mouse_overs_clear(&mut app);
        assert_eq!(app.sign_hover, -1);
        assert!(!app.sign_bubble);
        // Hover the RIGHT half (20, 30): origin (19, 30), cursor inside the
        // 2x2 box -> signHover latches.
        sign_hover_scan(&mut app, click(20, 30));
        assert_eq!(app.no_throw, 2, "`:32100`");
        assert!(app.sign_bubble, "`:32105`");
        assert_eq!(app.sign_x, 19 * 16 + 16, "`:32106`");
        assert_eq!(app.sign_y, 30 * 16, "`:32107`");
        assert_eq!(app.sign_hover, idx as i32, "`:32111`");
        // The clear re-derives.
        mouse_overs_clear(&mut app);
        assert_eq!(app.sign_hover, -1);
        assert!(!app.sign_bubble);
    }

    /// W4-f22 — the hover latch needs a LIVE record: with no record,
    /// `ReadSign(CreateIfMissing:false)` answers -1, so the bubble still
    /// latches (the tile is hovered) but `signHover` stays -1
    /// (`Player.cs:32108-32109`).
    #[test]
    fn the_hover_without_a_record_latches_only_the_bubble() {
        let mut app = sign_app();
        sign_hover_scan(&mut app, click(20, 30));
        assert!(app.sign_bubble, "the tile is hovered");
        assert_eq!(app.sign_hover, -1, "no record -> ReadSign -1");
        assert!(app.world.signs.is_empty(), "CreateIfMissing:false never creates");
    }

    /// W4-f22 — a non-sign tile never latches (`Player.cs:32098`'s
    /// `Main.tileSign` gate).
    #[test]
    fn the_hover_scan_ignores_non_sign_tiles() {
        let mut app = sign_app();
        sign_hover_scan(&mut app, click(5, 5));
        assert!(!app.sign_bubble);
        assert_eq!(app.sign_hover, -1);
        assert_eq!(app.no_throw, 0);
    }

    /// W4-f22 — the lazy kill-on-break (`WorldGen.cs:43686-43689`): breaking
    /// a sign tile kills its record at the frame-decoded origin; the open
    /// index becomes a NULL SLOT (past the store) so the lazy `sign_tick`
    /// closes the dialog + editor with sound 11, exactly like the C#
    /// null-in-place + next-tick close.
    #[test]
    fn breaking_the_open_sign_kills_the_record_and_closes_the_editor() {
        let mut app = sign_app();
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        text_sign(&mut app.world, 0, 19, 30, "stored".into());
        app.npc_chat_text = "stored".into();
        open_sign_keyboard(&mut app);
        assert!(app.sign_editing());
        assert_eq!(app.world.signs.len(), 1);
        // Break the RIGHT half (20, 30): frame_x 18 -> origin (19, 30).
        kill_broken_sign_record(&mut app, 55, 20, 30, 18, 0);
        assert!(app.world.signs.is_empty(), "Sign.KillSign at the origin");
        // The open index is a null slot (>= len), not -1: sign_tick closes.
        let open = app.player.as_ref().unwrap().sign;
        assert!(open >= app.world.signs.len() as i32, "the null-slot convention");
        sign_tick(&mut app);
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "CloseSign ran");
        assert!(!app.sign_editing(), "the editSign gate cleared the editor");
        assert!(app.npc_chat_text.is_empty());
    }

    /// W4-f22 — breaking a DIFFERENT sign remaps the open index through the
    /// dense-store splice: the open record keeps its identity (the C# slot
    /// index is stable; the port's splice shifts it), so the dialog stays
    /// open on the SAME text. The broken record sits BEFORE the open one,
    /// so the splice shifts the open index 1 -> 0.
    #[test]
    fn breaking_another_sign_remaps_the_open_index() {
        let mut app = sign_app();
        // Two records: (5,5) first (the one to break), (19,30) opened.
        let b = find_or_create(&mut app.world.signs, 5, 5).unwrap();
        text_sign(&mut app.world, b, 5, 5, "drop".into());
        let a = find_or_create(&mut app.world.signs, 19, 30).unwrap();
        text_sign(&mut app.world, a, 19, 30, "keep".into());
        assert_eq!(a, 1, "the open record is second");
        if let Some(p) = app.player.as_mut() {
            p.sign = a as i32; // open the SECOND record (index 1)
        }
        app.npc_chat_text = "keep".into();
        // Break the sign at (5,5) (frame 0 -> origin (5,5)).
        kill_broken_sign_record(&mut app, 55, 5, 5, 0, 0);
        assert_eq!(app.world.signs.len(), 1, "only (5,5) died");
        assert_eq!(app.world.signs[0].text, "keep");
        // The open index remapped 1 -> 0 (the survivor is now first).
        assert_eq!(app.player.as_ref().unwrap().sign, 0, "the splice remap");
        assert_eq!(app.npc_chat_text, "keep", "the dialog keeps reading");
    }

    /// W4-f22 — the 395 item frame shares `CheckSign` but is NOT a
    /// `tileSign` kind, so the sweep never touches it (`type != 395`,
    /// `WorldGen.cs:43686`).
    #[test]
    fn the_item_frame_kind_is_not_swept() {
        let mut app = sign_app();
        let idx = find_or_create(&mut app.world.signs, 19, 30).unwrap();
        text_sign(&mut app.world, idx, 19, 30, "kept".into());
        kill_broken_sign_record(&mut app, 395, 20, 30, 18, 0);
        assert_eq!(app.world.signs.len(), 1, "395 is not a tileSign kind");
    }

    /// W4-f22 — the tombstone epitaph DISPLAY (deviation 4; the d2 sweep
    /// owns the `misc_text` WRITE — `projectile_ai_tombstone.rs`'s
    /// `Sign.ReadSign`/`TextSign` at tile 85). Tile 85 IS a `tileSign` kind
    /// (`Main.cs:10082`), so the C# carries NO read-only difference for
    /// tombstones: the sign-read path shows the epitaph exactly like any
    /// sign text. This witness stands in for the d2 write (a record at a
    /// tile-85 origin holding the death text) and verifies the read/display
    /// half — `OpenSign`'s `npcChatText` read (`Player.cs:33549`) and the
    /// hover tooltip's text — surface the epitaph unchanged.
    #[test]
    fn the_tombstone_epitaph_reads_through_the_sign_path() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("tomb".into(), 64, 64);
        // A 2x2 tombstone (tile 85) at origin (19, 30), the d2 placement
        // shape (`projectile_ai_tombstone.rs`'s TileObject.Place).
        for x in [19, 20] {
            for y in [30, 31] {
                if let Some(cell) = w.tile_mut(x, y) {
                    cell.kind = 85;
                    cell.flags |= FLAG_ACTIVE;
                    cell.frame_x = if x == 19 { 0 } else { 18 };
                    cell.frame_y = if y == 30 { 0 } else { 18 };
                }
            }
        }
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        if let Some(p) = app.player.as_mut() {
            p.x = 19.0 * 16.0;
            p.y = 28.0 * 16.0;
        }
        // The d2 write stand-in: the epitaph record at the tile-85 origin.
        let idx = find_or_create(&mut app.world.signs, 19, 30).unwrap();
        let epitaph = "Tester was slain...";
        text_sign(&mut app.world, idx, 19, 30, epitaph.into());
        // The read display: OpenSign latches npcChatText = the epitaph.
        assert!(try_open_sign_at_cursor(&mut app, click(20, 30)));
        assert_eq!(app.npc_chat_text, epitaph, "OpenSign reads the epitaph");
        // The hover tooltip's text is the same record (no read-only gate).
        sign_hover_scan(&mut app, click(20, 30));
        assert_eq!(app.sign_hover, idx as i32);
        assert_eq!(app.world.signs[app.sign_hover as usize].text, epitaph);
        // The editor opens on the epitaph (the C# has no tombstone lock).
        open_sign_keyboard(&mut app);
        let Some(FrontScreen::Keyboard(s)) = app.front.as_ref() else {
            panic!("the sign keyboard is live");
        };
        assert_eq!(s.request.initial, epitaph, "the prefill is the epitaph");
    }
}
