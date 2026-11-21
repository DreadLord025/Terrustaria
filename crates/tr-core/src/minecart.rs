//! The minecart track-switch machinery — `Terraria.Minecart`'s connection
//! tables (`Minecart.cs:103-547` `Initialize`), `GetNearbyTilesSetLookupIndex`
//! (`:1170-1198`), `FrameTrack` (`:943-1168`) and `FlipSwitchTrack`
//! (`:1296-1319`).
//!
//! **DEPRECATED-BY-UNION (research/95 §6 + research/71 §1-2, EXECUTED on
//! e20 `cf0b8f9c`)**: this WHOLE module is the f11 duplicate half — the
//! merger drops it for e20's `TRACK_FRAMES` superset + `flip_switch_track`/
//! `frame_track_pound` (assert-on-front-absent; this module's `.max(0)`
//! clamp silently diverges) and the `CheckMech(5)` → `switch_track_flips`
//! request seam. The OnceLock cache shape (`track_switch_options`) already
//! merged into e20 (`6336e3b4`) over the e20 table; the table/cooldown
//! witnesses were rebased there (`5272cd52`). Kept, NOT deleted (the
//! pushed-history consumers); the `frame_track` non-pound branch + `flag4`/
//! `KillTile(fail)` half is the hammer-placement follow-up salvage note
//! (95 §8.8), not wiring code. See `docs/research/133-f11-unique-folds.md`
//! §4b.
//!
//! **SEAL NOTE (W4-f11 seal phase, 2026-11-21)**: g48 LANDED this canonical
//! module on its tip (`9460c85d9`, research/237 upstream hunk taken here
//! verbatim at `b46556b2d` — blob `8a7c149cb`, byte-identical to g48's;
//! this note is the only later delta), so the f11×g48 minecart merge is
//! a comment-only dedupe. The §4b drop-for-e20 ruling
//! still binds at the FULL union: whichever `frame_track` shape survives
//! (e20's `frame_track_pound` superset has NO `!mute` branch at all) must
//! keep the fail-tail NO-OP semantics for the net-17 reader's default
//! `mute: false` (`MessageBuffer.cs:1229`) — never a `kill_tile` call
//! (g48 research/237: "any survivor of the union keeps the no-op
//! semantics").
//!
//! W4-f11 ports the slice the wiring switch-track arm (`Wiring.cs:1744-1749`,
//! tile 314) needs: `FlipSwitchTrack` and the `FrameTrack(pound: true, mute:
//! true)` it calls. The tables are hand-transcribed from `Initialize`'s
//! sequential `num++` writes (the C# builds them imperatively; the port keeps
//! the resolved per-frame rows). The riding/physics half (`TrackCollision`,
//! `GetOnTrack`, the mount delegates) stays unported — this module is the
//! track-FRAMING seam the hammer's pound arm (`edit/hammer.rs`'s `Consumed`
//! note) and the wire arm share.

use crate::world::World;

/// `Minecart._leftSideConnection` (`Minecart.cs:109`, the `Initialize` writes
/// `:125-427`): per track frame, the left neighbour's vertical offset class
/// (−1 = no left connection, 0/1/2 = up/level/down). Index = `tile.frameX`.
/// Hand-transcribed from the sequential `num++` rows (frame 0 at `:125`,
/// frame 35 at `:419`); the pin test below replays the C# row order.
pub const LEFT_SIDE_CONNECTION: [i32; 36] = [
    -1, 1, -1, 1, 2, 1, 1, 0, 0, 2, 2, -1, 0, -1, -1, 1, 2, -1, 0, -1, -1, 1, -1, 1, -1, 1, 2,
    -1, 0, -1, 1, 1, 0, 2, 0, 2,
];

/// `Minecart._rightSideConnection` (`Minecart.cs:110`, `Initialize`
/// `:126-426`): the right neighbour's offset class. Index = `tile.frameX`.
pub const RIGHT_SIDE_CONNECTION: [i32; 36] = [
    -1, 1, 1, -1, 1, 2, 0, 1, 2, 0, -1, 2, -1, 0, 1, -1, -1, 2, -1, 0, -1, 1, 1, -1, 1, -1, -1,
    2, -1, 0, 1, 1, 2, 0, 2, 0,
];

/// `Minecart._trackType` (`Minecart.cs:111`, `Initialize` `:299-425`): 0 =
/// plain rail, 1 = pressure plate, 2 = booster. Index = `tile.frameX`. The
/// C# array zero-initialises and only writes the type-1 rows (frames 20-23,
/// `:299`/`:304`/`:310`/`:316`) and the type-2 rows (frames 30-35,
/// `:379`/`:385`/`:395`/`:405`/`:415`/`:425`).
pub const TRACK_TYPE: [i32; 36] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 2,
    2, 2, 2, 2, 2,
];

/// `Minecart._boostLeft` (`Minecart.cs:112`, `Initialize` `:380-426`): the
/// booster frames' direction (30 false, 31 true, 32 false, 33 true, 34 true,
/// 35 false). Only the type-2 rows carry a meaningful value; the rest keep
/// the bool-array default.
pub const BOOST_LEFT: [bool; 36] = [
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, true, false, true, true, false,
];

/// `Minecart._firstPressureFrame` (`Minecart.cs:519-531`): the first type-1
/// frame — 20.
pub const FIRST_PRESSURE_FRAME: i16 = 20;

/// `Minecart.GetNearbyTilesSetLookupIndex` (`Minecart.cs:1170-1198`): the
/// 6-bit mask of which of the six diagonal/side neighbours are track tiles
/// (314). Bit order: (i−1,j−1)=1, (i−1,j)=2, (i−1,j+1)=4, (i+1,j−1)=8,
/// (i+1,j)=16, (i+1,j+1)=32. The C# tests `.type == 314` WITHOUT an `active()`
/// gate (a deactivated track still counts) — kept verbatim.
pub fn nearby_tiles_set_lookup_index(world: &World, i: i32, j: i32) -> i32 {
    let mut num = 0;
    let is_track = |x: i32, y: i32| world.tile(x, y).is_some_and(|t| t.kind == 314);
    if is_track(i - 1, j - 1) {
        num += 1;
    }
    if is_track(i - 1, j) {
        num += 2;
    }
    if is_track(i - 1, j + 1) {
        num += 4;
    }
    if is_track(i + 1, j - 1) {
        num += 8;
    }
    if is_track(i + 1, j) {
        num += 16;
    }
    if is_track(i + 1, j + 1) {
        num += 32;
    }
    num
}

/// `Minecart._trackSwitchOptions` (`Minecart.cs:449-518`): for each of the 64
/// neighbour masks, the list of track frames whose side-connection mask fits.
/// Built once from the connection tables (the C# `Initialize` loop). A mask
/// with <2 neighbours requires an EXACT connection match; otherwise the
/// frame's connection bits must be a non-zero subset of the mask.
fn track_switch_options() -> &'static [Option<Vec<i32>>] {
    static OPTIONS: std::sync::OnceLock<Vec<Option<Vec<i32>>>> = std::sync::OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut out: Vec<Option<Vec<i32>>> = Vec::with_capacity(64);
        for mask in 0..64i32 {
            // `num24` = the mask's popcount (`:452-459`).
            let mut num24 = 0;
            let mut bit = 1;
            while bit < 256 {
                if (mask & bit) == bit {
                    num24 += 1;
                }
                bit <<= 1;
            }
            let mut frames = Vec::new();
            for frame in 0..36i32 {
                // `num28` = the frame's connection bitmask (`:464-488`).
                let mut num28 = 0;
                match LEFT_SIDE_CONNECTION[frame as usize] {
                    0 => num28 |= 1,
                    1 => num28 |= 2,
                    2 => num28 |= 4,
                    _ => {}
                }
                match RIGHT_SIDE_CONNECTION[frame as usize] {
                    0 => num28 |= 8,
                    1 => num28 |= 0x10,
                    2 => num28 |= 0x20,
                    _ => {}
                }
                // `:489-499` — the fit rule.
                if num24 < 2 {
                    if mask != num28 {
                        continue;
                    }
                } else if num28 == 0 || (mask & num28) != num28 {
                    continue;
                }
                frames.push(frame);
            }
            if frames.is_empty() {
                out.push(None);
            } else {
                out.push(Some(frames));
            }
        }
        out
    })
}

/// `Minecart.FrameTrack` (`Minecart.cs:943-1168`): re-derive the track's
/// front/back frames from its neighbours. `pound` selects the switch-cycle
/// branch (the hammer/wire path); `mute` suppresses the fail-tile kill and
/// gates non-314 tiles. Returns the C# bool (false = no valid switch).
pub fn frame_track(world: &mut World, i: i32, j: i32, pound: bool, mute: bool) -> bool {
    let Some(tile) = world.tile(i, j).copied() else {
        return false;
    };
    // `:955-958` — `mute && type != 314` refuses.
    if mute && tile.kind != 314 {
        return false;
    }
    let lookup = nearby_tiles_set_lookup_index(world, i, j);
    let num = i32::from(tile.frame_x); // FrontTrack
    let num2 = i32::from(tile.frame_y); // BackTrack
    let num3 = if num >= 0 && num < TRACK_TYPE.len() as i32 {
        TRACK_TYPE[num as usize]
    } else {
        0
    };
    let options = track_switch_options();
    let array = match options.get(lookup as usize).and_then(|o| o.as_deref()) {
        Some(a) => a,
        None => {
            // `:966-975` — no options: pound refuses, else reset to (0, −1).
            if pound {
                return false;
            }
            world.set_tile_frame_x(i, j, 0);
            world.set_tile_frame_y(i, j, -1);
            return false;
        }
    };
    let mut num4 = -1i32; // the chosen front index
    let mut num5 = -1i32; // the chosen back index
    if !pound {
        // `:976-1038` — the placement branch.
        let mut num6 = -1i32;
        let mut num7 = -1i32;
        let mut flag = false;
        for (k, &opt) in array.iter().enumerate() {
            if num2 == opt {
                num5 = k as i32;
            }
            if TRACK_TYPE[opt as usize] != num3 {
                continue;
            }
            if LEFT_SIDE_CONNECTION[opt as usize] == -1
                || RIGHT_SIDE_CONNECTION[opt as usize] == -1
            {
                if num == opt {
                    num4 = k as i32;
                    flag = true;
                }
                if num6 == -1 {
                    num6 = k as i32;
                }
            } else {
                if num == opt {
                    num4 = k as i32;
                    flag = false;
                }
                if num7 == -1 {
                    num7 = k as i32;
                }
            }
        }
        if num7 != -1 {
            if (num4 == -1) | flag {
                num4 = num7;
            }
        } else {
            if num4 == -1 {
                match num3 {
                    2 | 1 => return false,
                    _ => {}
                }
                num4 = num6;
            }
            num5 = -1;
        }
    } else {
        // `:1040-1108` — the pound (switch-cycle) branch.
        for (l, &opt) in array.iter().enumerate() {
            if num == opt {
                num4 = l as i32;
            }
            if num2 == opt {
                num5 = l as i32;
            }
        }
        let mut num9 = 0; // two-sided count
        let mut num10 = 0; // one-sided count
        for &opt in array {
            if TRACK_TYPE[opt as usize] == num3 {
                if LEFT_SIDE_CONNECTION[opt as usize] == -1
                    || RIGHT_SIDE_CONNECTION[opt as usize] == -1
                {
                    num10 += 1;
                } else {
                    num9 += 1;
                }
            }
        }
        if num9 < 2 && num10 < 2 {
            return false;
        }
        let flag2 = num9 == 0;
        let mut flag3 = false;
        if !flag2 {
            // `:1077-1089` — advance the back index to a different two-sided
            // connection of the same type.
            while !flag3 {
                num5 += 1;
                if num5 >= array.len() as i32 {
                    num5 = -1;
                    break;
                }
                let a = array[num5 as usize] as usize;
                // `array[num4]` — the C# assumes the current front frame is
                // in the options list here (num4 >= 0); the `.max(0)` is a
                // defensive clamp for the degenerate front-not-in-list state
                // (the C# would throw IndexOutOfRange, `:1085`).
                let b = array[num4.max(0) as usize] as usize;
                if (LEFT_SIDE_CONNECTION[a] != LEFT_SIDE_CONNECTION[b]
                    || RIGHT_SIDE_CONNECTION[a] != RIGHT_SIDE_CONNECTION[b])
                    && TRACK_TYPE[a] == num3
                    && LEFT_SIDE_CONNECTION[a] != -1
                    && RIGHT_SIDE_CONNECTION[a] != -1
                {
                    flag3 = true;
                }
            }
        }
        if !flag3 {
            // `:1093-1107` — advance the front index to a matching connection
            // class (one-sided when flag2, two-sided otherwise).
            loop {
                num4 += 1;
                if num4 >= array.len() as i32 {
                    num4 = -1;
                    loop {
                        num4 += 1;
                        let a = array[num4 as usize] as usize;
                        if TRACK_TYPE[a] == num3
                            && (LEFT_SIDE_CONNECTION[a] == -1
                                || RIGHT_SIDE_CONNECTION[a] == -1)
                                == flag2
                        {
                            break;
                        }
                    }
                    break;
                }
                let a = array[num4 as usize] as usize;
                if TRACK_TYPE[a] == num3
                    && (LEFT_SIDE_CONNECTION[a] == -1 || RIGHT_SIDE_CONNECTION[a] == -1) == flag2
                {
                    break;
                }
            }
        }
    }
    // `:1110-1142` — did the front/back change? (the fail-tile gate).
    let mut flag4 = false;
    let front_now = world.tile_frame_x(i, j);
    let back_now = world.tile_frame_y(i, j);
    match num4 {
        -2 => {
            if front_now != i32::from(FIRST_PRESSURE_FRAME) {
                flag4 = true;
            }
        }
        -1 => {
            if front_now != 0 {
                flag4 = true;
            }
        }
        _ => {
            if front_now != array[num4 as usize] {
                flag4 = true;
            }
        }
    }
    if num5 == -1 {
        if back_now != -1 {
            flag4 = true;
        }
    } else if back_now != array[num5 as usize] {
        flag4 = true;
    }
    // `:1143-1162` — the writes.
    match num4 {
        -2 => world.set_tile_frame_x(i, j, i32::from(FIRST_PRESSURE_FRAME)),
        -1 => world.set_tile_frame_x(i, j, 0),
        _ => world.set_tile_frame_x(i, j, array[num4 as usize]),
    }
    if num5 == -1 {
        world.set_tile_frame_y(i, j, -1);
    } else {
        world.set_tile_frame_y(i, j, array[num5 as usize]);
    }
    // `:1163-1166` — `(pound & flag4) && !mute` → KillTile(fail: true). The
    // wire/hammer callers pass mute: true, so this never fires there. The
    // net-17 reader (`MessageBuffer.cs:1229`) passes the DEFAULT mute: false
    // — the branch fires, but the C# fail-tile KillTile never removes the
    // tile (`WorldGen.cs:63709-63737`: the grass/moss conversions + the
    // SquareTileFrame wake, dust/sound only). W4-g48 fix over the f11 body:
    // the old `kill_tile(world, i, j, true)` mapped `true` to no_item and
    // KILLED the track — the port's kill seam has no fail flavour, so the
    // fail tick skips the break (the kind-0 `net_apply` precedent; the
    // fail-dust/sound is the visual seam, ANALYSIS). The framing wake the
    // C# fail arm runs is the caller's (`reframe_area` after the call).
    if (pound & flag4) && !mute {
        // KillTile(fail: true) — no world-state change (see above).
    }
    true
}

/// `Minecart.FlipSwitchTrack` (`Minecart.cs:1296-1319`): the wiring
/// switch-track arm (`Wiring.cs:1747`). `FrontTrack` = frameX, `BackTrack` =
/// frameY (`:1632-1650`). A plain rail (type 0) with a back connection swaps
/// front/back; a booster (type 2) re-frames via `FrameTrack(pound, mute)`.
pub fn flip_switch_track(world: &mut World, i: i32, j: i32) {
    let Some(tile) = world.tile(i, j).copied() else {
        return;
    };
    let num = i32::from(tile.frame_x); // FrontTrack
    if num == -1 {
        return;
    }
    if num < 0 || num >= TRACK_TYPE.len() as i32 {
        return;
    }
    match TRACK_TYPE[num as usize] {
        0 => {
            let back = i32::from(tile.frame_y); // BackTrack
            if back != -1 {
                world.set_tile_frame_x(i, j, back);
                world.set_tile_frame_y(i, j, num);
            }
        }
        2 => {
            frame_track(world, i, j, true, true);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn put_track(w: &mut World, x: i32, y: i32, front: i16, back: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 314,
            frame_x: front,
            frame_y: back,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// The connection tables' popcount sanity: frame 0 is the isolated rail
    /// (both sides −1), frame 1 the level straight (both 1).
    #[test]
    fn w4_f11_the_track_tables_transcribe_the_isolated_and_straight_frames() {
        assert_eq!(LEFT_SIDE_CONNECTION[0], -1);
        assert_eq!(RIGHT_SIDE_CONNECTION[0], -1);
        assert_eq!(TRACK_TYPE[0], 0);
        assert_eq!(LEFT_SIDE_CONNECTION[1], 1);
        assert_eq!(RIGHT_SIDE_CONNECTION[1], 1);
        // The pressure frames 20..23 are type 1; the boosters 30..35 type 2.
        assert_eq!(TRACK_TYPE[20], 1);
        assert_eq!(FIRST_PRESSURE_FRAME, 20);
        assert_eq!(TRACK_TYPE[30], 2);
        assert!(!BOOST_LEFT[30]);
        assert!(BOOST_LEFT[31]);
    }

    /// `FlipSwitchTrack`'s type-0 arm (`Minecart.cs:1306-1313`): a plain rail
    /// with a back connection swaps front/back.
    #[test]
    fn w4_f11_a_plain_rail_swaps_front_and_back() {
        let mut w = world();
        // frame 2 (left −1, right 1) with back frame 3.
        put_track(&mut w, 10, 20, 2, 3);
        flip_switch_track(&mut w, 10, 20);
        assert_eq!(w.tile_frame_x(10, 20), 3, "front took the back");
        assert_eq!(w.tile_frame_y(10, 20), 2, "back took the front");
    }

    /// `FlipSwitchTrack`'s no-back guard (`:1307`): a plain rail with back −1
    /// does nothing.
    #[test]
    fn w4_f11_a_plain_rail_without_a_back_connection_is_a_no_op() {
        let mut w = world();
        put_track(&mut w, 10, 20, 1, -1);
        flip_switch_track(&mut w, 10, 20);
        assert_eq!(w.tile_frame_x(10, 20), 1);
        assert_eq!(w.tile_frame_y(10, 20), -1);
    }

    /// W4-g48 f11-tail-fix pin: the `(pound & flag4) && !mute` fail tail
    /// (`:1163-1166`) NEVER removes the track — the C# `KillTile(fail: true)`
    /// converts grass/moss + wakes framing and returns (`WorldGen.cs:63709-63737`),
    /// and tile 314 matches none of the conversion rows. The f11 body's
    /// `kill_tile(world, i, j, true)` mapped `true` to no_item and KILLED the
    /// tile; the net-17 reader passes the DEFAULT `mute: false`
    /// (`MessageBuffer.cs:1229`), so the branch fires there. The isolated rail
    /// (front 0, both sides −1) with level neighbours both sides (mask 18)
    /// cycles the front 0 → 1 under pound, flag4 fires — the tile survives.
    #[test]
    fn w4_g48_the_pound_fail_tail_never_removes_the_track() {
        let mut w = world();
        put_track(&mut w, 10, 20, 0, -1);
        put_track(&mut w, 9, 20, 1, -1);
        put_track(&mut w, 11, 20, 1, -1);
        assert!(frame_track(&mut w, 10, 20, true, false), "the pound cycles");
        let t = w.tile(10, 20).copied().expect("the track survived the fail tail");
        assert_eq!(t.kind, 314, "KillTile(fail:true) never removes");
        assert!(t.flags & FLAG_ACTIVE != 0, "still active");
        assert_eq!(t.frame_x, 1, "front cycled to the level straight");
        assert_eq!(t.frame_y, -1, "back unchanged (flag2 one-sided branch)");
    }

    /// `GetNearbyTilesSetLookupIndex` (`:1170-1198`): the six-neighbour mask.
    #[test]
    fn w4_f11_the_lookup_index_masks_the_six_track_neighbours() {
        let mut w = world();
        put_track(&mut w, 10, 20, 1, -1);
        assert_eq!(nearby_tiles_set_lookup_index(&w, 10, 20), 0);
        put_track(&mut w, 9, 20, 1, -1); // (i−1, j) → bit 2
        put_track(&mut w, 11, 20, 1, -1); // (i+1, j) → bit 16
        assert_eq!(nearby_tiles_set_lookup_index(&w, 10, 20), 2 + 16);
    }
}
