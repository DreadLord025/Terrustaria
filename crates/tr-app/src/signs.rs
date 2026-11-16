//! The `Sign` model's net-facing half — `Sign.ReadSign`
//! (`Sign.cs:27-58`), `Sign.TextSign` (`Sign.cs:74-85`) and the tile-origin
//! decode the `46`/`47` messages need (`MessageBuffer.cs:2366-2405`).
//!
//! The port stores signs as `World.signs: Vec<WldSign>` holding exactly the
//! non-null records in slot order (`tr-core/wld/sections/signs.rs`), so the
//! C#'s `Main.sign[i] == null` is "index past the store" and the array slot
//! index is the store index. The sign editor UI is not ported yet (S1-E-26),
//! so the local-side `OpenSign`/`editSign` hooks are absent — the message
//! layer here is what the future editor and the peer relay both ride.

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
}
