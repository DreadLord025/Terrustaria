//! The world-chest entity and its store operations (M3-CHEST-UI phase 1):
//! `Main.chest`'s entity + the statics over it, ported from
//! `decompiled/Terraria/Chest.cs` (1.4.5.8).
//!
//! Shape: the C# store operations are statics over `Main.chest`
//! (`Main.cs:66994`, `new Chest[8000]`) — the port keeps that shape with
//! free functions over the app's `App.world_chests: Vec<WorldChest>` (the
//! field's frozen `Vec` + element shape come from the existing
//! `world_sim/editing.rs` break-refusal closure and its stocked-chest test,
//! which pin `craft::WorldChest { tile, slots }`; `craft.rs` re-exports the
//! moved type). The slot encoding is indexed with implicit air:
//! `slots[i]` is chest slot `i`, airs beyond `slots.len()` are implicit
//! (`(0, 0)`), and there are no trailing airs. Entries are created lazily on
//! first open (phase 2); placement-time creation is unreachable without
//! `editing.rs` edits (deviation recorded for `docs/ANALYSIS.md`).
//!
//! Out of this phase: open/close interaction (phase 2), the panel (phase 3),
//! slot clicks (phase 4), rename (phase 5), contents-drop on break, `.wld`
//! persistence + `Resize`/per-chest `maxItems` (M6-SAVE-2), banks
//! (M3-STORE-1), worldgen loot (M5-CHEST-gen).

use tr_core::World;

/// One chest slot: `(item kind, stack)` — the C# `Chest.item[]` entry minus
/// the prefix (the port's inventory slots carry no prefix yet; the save
/// format's per-slot prefix byte, `WorldFile.cs:1671`, lands with M6-SAVE-2,
/// which owns the chest slot type's widening). The 2-tuple shape is pinned by
/// the existing `editing.rs` stocked-chest test literal (`slots: vec![(9, 6)]`).
pub(crate) type ChestSlot = (u16, u32);

/// `Main.maxChests` (`Main.cs:1033`), the array size (`Main.cs:66994`) and the
/// scan bound baked into `Chest.FindEmptyChest` (`Chest.cs:524`).
pub(crate) const MAX_CHESTS: usize = 8000;

/// `Chest.DefaultMaxItems` (`Chest.cs:64`). Every chest carries the default
/// until M6-SAVE-2 ports `Resize` (`Chest.cs:118-131`) — the per-chest field
/// is unreadable on the frozen `WorldChest` literal, and with `Resize` out no
/// chest ever deviates from 40, so the constant read is observably identical.
pub(crate) const DEFAULT_MAX_ITEMS: u32 = 40;

/// One world chest's item store (`Main.chest[]`'s `Chest.item`). The port's
/// chest UI/storage owns filling it (`M3-CHEST-UI`); the world save skips the
/// chest section (`M6-SAVE-2`), so this is in-memory only. The C# entity's
/// `x`/`y`/`item` fields (`Chest.cs:74-78`); `index` is the `Vec` position,
/// `name`/`maxItems`/`frame` land with their phases (5 / M6-SAVE-2 / render
/// follow-up).
#[derive(Clone, Debug, Default)]
pub(crate) struct WorldChest {
    /// The chest tile's top-left `(x, y)`.
    pub(crate) tile: (i32, i32),
    /// `(item id, stack)` stacks, at most [`DEFAULT_MAX_ITEMS`] real ones;
    /// indexed with implicit air — see the module doc.
    pub(crate) slots: Vec<ChestSlot>,
}

/// The removed-slot marker: `Main.chest[i] == null`'s stand-in inside the
/// frozen flat `Vec<WorldChest>` (`Chest.RemoveChest`, `Chest.cs:132-141`,
/// nulls the array slot). Unreachable as a world coordinate, so no live
/// lookup can match it.
const REMOVED_TILE: (i32, i32) = (i32::MIN, i32::MIN);

/// Whether `chest` is a removed (null) slot.
fn is_removed(chest: &WorldChest) -> bool {
    chest.tile == REMOVED_TILE
}

impl WorldChest {
    /// `Chest.IsEmpty` (`Chest.cs:166-176`): every slot within `maxItems` is
    /// air (`Item.IsAir`: type 0 and stack 0). The port's implicit airs
    /// beyond `slots.len()` are air by construction, so the scan covers the
    /// whole chest.
    pub(crate) fn is_empty(&self) -> bool {
        self.slots
            .iter()
            .all(|&(kind, stack)| kind == 0 && stack == 0)
    }
}

/// `Chest.FindChest` (`Chest.cs:513-520`): the store index at the coords, or
/// `None` for the C# `-1`. The C#'s O(1) `_chestsByCoords` dict becomes a
/// linear scan (code-shape-free; at most [`MAX_CHESTS`] entries on a cold
/// path).
pub(crate) fn find_chest(chests: &[WorldChest], x: i32, y: i32) -> Option<usize> {
    chests
        .iter()
        .position(|c| !is_removed(c) && c.tile == (x, y))
}

/// `Chest.FindEmptyChest` (`Chest.cs:522-541`): the first null slot index, or
/// `None` for the C# `-1` — either because a chest already exists at the
/// coords (`Chest.cs:529-533`, the collision rule) or because the store is
/// full ([`MAX_CHESTS`]). Removed slots are null and are reused first.
pub(crate) fn find_empty_chest(chests: &[WorldChest], x: i32, y: i32) -> Option<usize> {
    if find_chest(chests, x, y).is_some() {
        return None;
    }
    match chests.iter().position(is_removed) {
        Some(i) => Some(i),
        None if chests.len() < MAX_CHESTS => Some(chests.len()),
        None => None,
    }
}

/// `Chest.CreateChest` (`Chest.cs:590-604`) + `CreateWorldChest`/`Assign`
/// (`Chest.cs:104-119`): claim the first empty slot for the coords and store
/// the fresh (empty) chest. The C# `-1` failure (coords taken, store full —
/// `Chest.cs:595-597`) is caller-visible through [`find_empty_chest`]'s
/// `None`; this is the second step of the find-first sequence the open path
/// uses (`find` → `Some` → open, `None` → create), so its failure paths are
/// caller-contract violations and panic with the invariant (the house
/// `expect` pattern, `craft.rs:443`).
pub(crate) fn create_chest(chests: &mut Vec<WorldChest>, x: i32, y: i32) -> usize {
    let idx = find_empty_chest(chests, x, y).expect(
        "create_chest follows a find-first miss; a taken coord or full store \
         is the C# -1 path (Chest.cs:595-597)",
    );
    // `CreateWorldChest` fills with empty instances (`Chest.cs:107-109`) —
    // the port's empty slot list, air implied beyond the length.
    let fresh = WorldChest {
        tile: (x, y),
        slots: Vec::new(),
    };
    if idx == chests.len() {
        chests.push(fresh);
    } else {
        chests[idx] = fresh;
    }
    idx
}

/// `Chest.RemoveChest` (`Chest.cs:132-141`): the slot becomes null. Index
/// stable — the C# array slot is nulled, not compacted, so held indices
/// (`Player.chest`) stay valid.
pub(crate) fn remove_chest(chests: &mut Vec<WorldChest>, idx: usize) {
    if let Some(slot) = chests.get_mut(idx) {
        if !is_removed(slot) {
            *slot = WorldChest {
                tile: REMOVED_TILE,
                slots: Vec::new(),
            };
        }
    }
}

/// The chest's slots at `idx` — the plan contract's
/// `get(idx) -> Option<&[ChestSlot]>` over `Chest.item[]`
/// (`Chest.cs:74-78`). `None` for an out-of-range index; a removed slot
/// reads as absent.
pub(crate) fn chest_slots(chests: &[WorldChest], idx: usize) -> Option<&[ChestSlot]> {
    chests
        .get(idx)
        .filter(|c| !is_removed(c))
        .map(|c| c.slots.as_slice())
}

/// Mutable [`chest_slots`] — the plan contract's `get_mut`
/// (`Chest.cs:74-78`).
pub(crate) fn chest_slots_mut(chests: &mut [WorldChest], idx: usize) -> Option<&mut [ChestSlot]> {
    chests
        .get_mut(idx)
        .filter(|c| !is_removed(c))
        .map(|c| c.slots.as_mut_slice())
}

/// `Chest.IsLocked` (`Chest.cs:301-313`): the locked-frame decode. A missing
/// tile reads locked (the C# null-tile branch, `Chest.cs:303-305`); tile 21
/// (`TileID.Containers`) is locked in the six inclusive `frameX` ranges; tile
/// 467 (`TileID.Containers2`) iff `frameX / 36 == 13` (style 13). The decode
/// reads `type`/`frameX` without an activity check, as in the C#.
pub(crate) fn is_locked(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    let fx = t.frame_x;
    if t.kind == 21 {
        return (72..=106).contains(&fx)
            || (144..=178).contains(&fx)
            || (828..=1006).contains(&fx)
            || (1296..=1330).contains(&fx)
            || (1368..=1402).contains(&fx)
            || (1440..=1474).contains(&fx);
    }
    if t.kind == 467 {
        return fx / 36 == 13;
    }
    false
}

/// `Chest.IsLockedOrInUse` (`Chest.cs:178-188`): true when any player is
/// browsing the chest (`IsPlayerInChest`, `Chest.cs:267-274`) or the tile is
/// locked. The bank branch (`bankChest` → false) is unported — the port has
/// no bank chests until M3-STORE-1, which must restore the early `false`.
/// The 255-player loop collapses to the local player (single player): their
/// `Player.chest` (`-1` = none) against the index. An absent or removed slot
/// is neither locked nor in use (the C# instance method can only run on a
/// found chest).
pub(crate) fn is_locked_or_in_use(
    chests: &[WorldChest],
    world: &World,
    idx: usize,
    open_player_chest: i32,
) -> bool {
    let Some(chest) = chests.get(idx).filter(|c| !is_removed(c)) else {
        return false;
    };
    if open_player_chest == idx as i32 {
        return true;
    }
    is_locked(world, chest.tile.0, chest.tile.1)
}

/// `Chest.UsingChest` (`Chest.cs:498-509`): the first active player browsing
/// chest `i`, or `-1`. Single-player reading: the local player is the only
/// candidate, so the answer is whether they are browsing an existing (live)
/// slot — the C# player index collapses to this bool.
pub(crate) fn using_chest(chests: &[WorldChest], idx: usize, open_player_chest: i32) -> bool {
    chests.get(idx).is_some_and(|c| !is_removed(c)) && open_player_chest == idx as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("t".into(), 120, 120)
    }

    fn set_tile(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].frame_x = frame_x;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    /// DW-1.1 — `CreateChest`/`FindChest`/`FindEmptyChest`/`RemoveChest`
    /// (`Chest.cs:513-541`, `:590-604`, `:132-141`): the store cycle with the
    /// coord-collision `-1` rule and the first-null-slot reuse.
    #[test]
    fn store_create_find_find_empty_is_empty_remove_cycles() {
        let mut chests: Vec<WorldChest> = Vec::new();
        // Create claims the first null slot (`Chest.cs:522-541`).
        let a = create_chest(&mut chests, 10, 18);
        assert_eq!(a, 0);
        assert_eq!(chests[a].tile, (10, 18));
        assert!(
            chests[a].slots.is_empty(),
            "a fresh chest is empty instances (Chest.cs:107-109)"
        );
        // `FindChest` answers the coords, misses elsewhere (`Chest.cs:513-520`).
        assert_eq!(find_chest(&chests, 10, 18), Some(0));
        assert_eq!(find_chest(&chests, 11, 18), None);
        // A second chest takes the next slot.
        assert_eq!(create_chest(&mut chests, 30, 40), 1);
        // The collision rule (`Chest.cs:529-533`): any chest at the coords
        // refuses the empty-slot search with `-1` — the C# `None`.
        assert_eq!(find_empty_chest(&chests, 10, 18), None);
        assert_eq!(find_empty_chest(&chests, 30, 40), None);
        // Free coords hand out the append slot.
        assert_eq!(find_empty_chest(&chests, 50, 50), Some(2));
        // `RemoveChest` nulls the slot; `FindChest` misses it, the slot reads
        // as absent, and the next empty-slot search reuses the index
        // (`Chest.cs:524-536` first-null rule).
        remove_chest(&mut chests, 0);
        assert_eq!(find_chest(&chests, 10, 18), None);
        assert_eq!(chest_slots(&chests, 0), None);
        assert_eq!(find_empty_chest(&chests, 90, 90), Some(0));
        // `CreateChest` after the removal reclaims slot 0; held indices of
        // the other slots never shift (`Chest.cs:132-141` nulls in place).
        assert_eq!(create_chest(&mut chests, 90, 90), 0);
        assert_eq!(chests[0].tile, (90, 90));
        assert_eq!(chests[1].tile, (30, 40));
    }

    /// DW-1.1 — `Chest.IsEmpty` (`Chest.cs:166-176`) and the empty-entry ≡
    /// absent rule: an emptied entry answers like an absent one for the
    /// break refusal (the `editing.rs` closure's `stack > 0` scan) while
    /// `FindChest` still finds it — the chest exists, it is just empty.
    #[test]
    fn is_empty_and_empty_entry_equivalence() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        assert!(chests[idx].is_empty());
        chests[idx].slots = vec![(9, 6)];
        assert!(!chests[idx].is_empty());
        // A drained slot is air in place (`Item.IsAir`: type 0 and stack 0),
        // not a removed entry.
        chests[idx].slots = vec![(0, 0), (619, 4)];
        assert!(!chests[idx].is_empty());
        chests[idx].slots = vec![(0, 0), (0, 0)];
        assert!(chests[idx].is_empty());
        assert_eq!(
            find_chest(&chests, 10, 18),
            Some(idx),
            "an empty entry is not a removed slot"
        );
    }

    /// DW-1.1 — the contract's `get`/`get_mut` (`chest_slots`/
    /// `chest_slots_mut`): the slot slice at an index, `None` for
    /// out-of-range and removed slots, `get_mut` writing through.
    #[test]
    fn chest_slots_on_removed_and_out_of_range() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        chests[idx].slots = vec![(9, 6), (0, 0), (619, 4)];
        assert_eq!(
            chest_slots(&chests, idx),
            Some(&[(9, 6), (0, 0), (619, 4)][..])
        );
        assert_eq!(chest_slots(&chests, 5), None);
        remove_chest(&mut chests, idx);
        assert_eq!(chest_slots(&chests, idx), None);
        let other = create_chest(&mut chests, 40, 40);
        // The reuse claimed the tombstone's slot; a fresh chest is empty
        // until a slot is stocked (phase 4's placement grows the list).
        chests[other].slots = vec![(9, 6)];
        if let Some(slots) = chest_slots_mut(&mut chests, other) {
            slots[0] = (48, 1);
        }
        assert_eq!(chests[other].slots[0], (48, 1));
    }

    /// DW-1.1 — `Main.maxChests` (`Main.cs:1033`) bounds the store: the 8000th
    /// chest takes the last slot and the next create hits the C# `-1` path
    /// (`Chest.cs:524` scans `i < 8000`). `create_chest` panics on the
    /// caller-contract violation (its doc).
    #[test]
    #[should_panic(expected = "create_chest follows a find-first miss")]
    fn create_chest_refuses_the_8000th_chest() {
        let mut chests: Vec<WorldChest> = Vec::new();
        for i in 0..MAX_CHESTS {
            let x = (i % 1000) as i32;
            let y = 1000 + (i / 1000) as i32;
            assert_eq!(create_chest(&mut chests, x, y), i);
        }
        assert_eq!(
            find_empty_chest(&chests, 5000, 5000),
            None,
            "the store is full"
        );
        create_chest(&mut chests, 5000, 5000);
    }

    /// DW-1.2 — `Chest.IsLocked` (`Chest.cs:301-313`): every locked `frameX`
    /// range pinned at both boundaries, tile 467 style 13, the null-tile
    /// branch, and the no-active-check reading.
    #[test]
    fn is_locked_frame_boundaries_match_chest_cs() {
        let mut w = world();
        // Tile 21 (`TileID.Containers`): the six inclusive ranges — one probe
        // cell per range so the boundaries cannot mask each other.
        let ranges = [
            (72, 106),
            (144, 178),
            (828, 1006),
            (1296, 1330),
            (1368, 1402),
            (1440, 1474),
        ];
        for (n, &(lo, hi)) in ranges.iter().enumerate() {
            let x = 4 + n as i32 * 2;
            set_tile(&mut w, x, 10, 21, (lo - 1) as i16);
            assert!(!is_locked(&w, x, 10), "frameX {lo}-1 unlocks");
            set_tile(&mut w, x, 10, 21, lo as i16);
            assert!(is_locked(&w, x, 10), "frameX {lo} locks");
            set_tile(&mut w, x, 10, 21, hi as i16);
            assert!(is_locked(&w, x, 10), "frameX {hi} locks");
            set_tile(&mut w, x, 10, 21, (hi + 1) as i16);
            assert!(!is_locked(&w, x, 10), "frameX {hi}+1 unlocks");
        }
        // Tile 467 (`TileID.Containers2`): locked iff `frameX / 36 == 13`
        // (styles 0..12 and 14+ are plain chests).
        set_tile(&mut w, 4, 20, 467, 467);
        assert!(!is_locked(&w, 4, 20), "467/36 == 12 unlocks");
        set_tile(&mut w, 4, 20, 467, 468);
        assert!(is_locked(&w, 4, 20), "468/36 == 13 locks");
        set_tile(&mut w, 4, 20, 467, 503);
        assert!(is_locked(&w, 4, 20), "503/36 == 13 locks");
        set_tile(&mut w, 4, 20, 467, 504);
        assert!(!is_locked(&w, 4, 20), "504/36 == 14 unlocks");
        // A non-container tile is never locked; an unframed tile 21
        // (`frameX -1`, the "must be computed" sentinel) is unlocked.
        set_tile(&mut w, 4, 30, 1, 90);
        assert!(!is_locked(&w, 4, 30), "stone is not a chest");
        set_tile(&mut w, 6, 30, 21, -1);
        assert!(!is_locked(&w, 6, 30), "frameX -1 unlocks");
        // The C# null-tile branch (`Chest.cs:303-305`): a missing tile reads
        // locked. And the decode ignores activity, as the C# does.
        assert!(is_locked(&w, 119, 120), "out of bounds ≡ null tile");
        set_tile(&mut w, 8, 30, 21, 90);
        let i = w.index(8, 30);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        assert!(is_locked(&w, 8, 30), "an inactive locked frame still locks");
    }

    /// DW-1.3 — `Chest.IsLockedOrInUse` (`Chest.cs:178-188`): the in-use
    /// player wins before the lock decode; the bank arm is unported
    /// (M3-STORE-1 must restore `bankChest` → false); absent/removed slots
    /// are neither.
    #[test]
    fn is_locked_or_in_use_matches_chest_cs() {
        let mut w = world();
        set_tile(&mut w, 10, 18, 21, 0); // unlocked chest tile
        set_tile(&mut w, 30, 40, 21, 90); // locked chest tile
        let mut chests: Vec<WorldChest> = Vec::new();
        let open = create_chest(&mut chests, 10, 18);
        let locked = create_chest(&mut chests, 30, 40);
        // Idle player: the lock decode decides.
        assert!(!is_locked_or_in_use(&chests, &w, open, -1));
        assert!(
            is_locked_or_in_use(&chests, &w, locked, -1),
            "frameX 90 locks"
        );
        // `IsPlayerInChest` (`Chest.cs:267-274`): any browsing player puts it
        // in use — even an unlocked chest.
        assert!(is_locked_or_in_use(&chests, &w, open, open as i32));
        assert!(is_locked_or_in_use(&chests, &w, locked, locked as i32));
        // A removed or out-of-range slot is neither locked nor in use.
        remove_chest(&mut chests, open);
        assert!(!is_locked_or_in_use(&chests, &w, open, -1));
        assert!(!is_locked_or_in_use(&chests, &w, 9, -1));
    }

    /// `Chest.UsingChest` (`Chest.cs:498-509`) — the single-player reading:
    /// true iff the slot exists (not removed) and the player browses it.
    #[test]
    fn using_chest_follows_the_single_player_reading() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        assert!(using_chest(&chests, idx, idx as i32));
        assert!(!using_chest(&chests, idx, -1), "no chest open");
        assert!(
            !using_chest(&chests, idx, idx as i32 + 1),
            "another chest open"
        );
        remove_chest(&mut chests, idx);
        assert!(
            !using_chest(&chests, idx, idx as i32),
            "a null slot is unusable"
        );
    }
}
