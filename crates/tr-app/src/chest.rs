//! The world-chest entity, its store operations and its open/close +
//! locking interaction (M3-CHEST-UI phases 1-2): `Main.chest`'s entity, the
//! statics over it and the `Player.cs` chest branch, ported from
//! `decompiled/Terraria` (1.4.5.8).
//!
//! Shape: the C# store operations are statics over `Main.chest`
//! (`Main.cs:66994`, `new Chest[8000]`) — the port keeps that shape with
//! free functions over the app's `App.world_chests: Vec<WorldChest>` (the
//! field's frozen `Vec` + element shape come from the existing
//! `world_sim/editing.rs` break-refusal closure and its stocked-chest test,
//! which pin `craft::WorldChest`'s `{ tile, slots }` core; `craft.rs`
//! re-exports the moved type — the `.wld` phase added the name/maxItems fold
//! on top of it). The slot encoding is indexed with implicit air:
//! `slots[i]` is chest slot `i`, airs beyond `slots.len()` are implicit
//! ([`EMPTY_SLOT`]), and there are no trailing airs — except the browsed
//! chest, whose 40-slot window [`open_chest`] materializes (the C#
//! `Chest.item[]` shape the transfer free-slot walks scan; the craft drain's
//! tail fold re-tidies it on a full drain). Entries are created lazily on
//! the first open ([`try_open_chest_at_cursor`]); placement-time creation is
//! unreachable without `editing.rs` edits (deviation recorded for
//! `docs/ANALYSIS.md`).
//!
//! Out of this phase: the panel (phase 3), slot clicks (phase 4), rename +
//! the `Main.editChest` flush arm of the open path (phase 5), contents-drop
//! on break, banks (M3-STORE-1), worldgen loot (M5-CHEST-gen). The `.wld`
//! persistence (`WorldFile.SaveChests:1606` / `LoadChests:2723`) landed with
//! M6-SAVE-1 phase 3: the entity now owns its `name`/`maxItems`
//! ([`WorldChest`]) and [`from_wld`]/[`to_wld`] are the app-side seam the
//! world-file wiring (phase 7) calls.

use glam::Vec2;

use crate::app::App;
use crate::app::flow::KeyboardTail;
use crate::bank::can_stack;
use crate::item::{EMPTY_SLOT, HOTBAR, INV, ItemSlot};
use crate::player;
use crate::registry;
use crate::ui::keyboard::{self, Context, Request};
use tr_core::{TILE_PX, World};
use tr_core::wld::sections::{WldChest, WldChestSlot};

/// `Main.maxChests` (`Main.cs:1033`), the array size (`Main.cs:66994`) and the
/// scan bound baked into `Chest.FindEmptyChest` (`Chest.cs:524`).
pub(crate) const MAX_CHESTS: usize = 8000;

/// `Chest.DefaultMaxItems` (`Chest.cs:64`). A chest carries the default until
/// a `.wld` record says otherwise (`Chest.Resize`, `Chest.cs:118-131`, runs in
/// `LoadChests:2738-2740`).
pub(crate) const DEFAULT_MAX_ITEMS: u32 = 40;

/// `ItemID.Sets.ItemsThatShouldNotBeInInventory` (`ID/ItemID.cs:345`) —
/// `CreateBoolSet(false, 58, 184, 1734, 1735, 1867, 1868, 3453, 3454, 3455,
/// 4143, 5013, 6135, 6136)`. A slot holding one of these is air on both sides
/// of the file: never written (`SaveChests:1658-1661`) and dropped on load
/// (`Item.FixAgainstExploit`, `Item.cs:49489-49495`). Hand-transcribed like
/// `prefix.rs:114-122`'s denial list — the set is not in the generated tables.
const BANNED_IN_INVENTORY: &[u16] = &[
    58, 184, 1734, 1735, 1867, 1868, 3453, 3454, 3455, 4143, 5013, 6135, 6136,
];

/// `Main.tileContainer` (`Main.cs:10076-10080`): the tile kinds whose 2×2 a
/// chest anchor stands on. `SaveChests:1617-1632`'s pre-pass destroys every
/// chest whose four anchor cells are not active tiles from this set.
fn is_tile_container(kind: u16) -> bool {
    matches!(kind, 21 | 467 | 88 | 470 | 475)
}

/// One world chest: the `Main.chest[]` entity (`Chest.cs:74-82`). The port's
/// chest UI/storage owns filling it (`M3-CHEST-UI`) and the `.wld` section
/// persists it (`WorldFile.SaveChests:1606` / `LoadChests:2723`).
#[derive(Clone, Debug, Default)]
pub(crate) struct WorldChest {
    /// The chest tile's top-left `(x, y)` (`Chest.x`/`Chest.y`).
    pub(crate) tile: (i32, i32),
    /// The C# `Chest.item[]` entries as full [`ItemSlot`]s (the M3-CHEST-INT
    /// upgrade — prefixes survive deposits and take-backs, `Item.cs:504`'s
    /// `Item.prefix` field), at most [`max_items`](Self::max_items) real ones;
    /// indexed with implicit air — see the module doc.
    pub(crate) slots: Vec<ItemSlot>,
    /// `Chest.name` (`Chest.cs:82`): the panel's name line and the rename
    /// submit's write target. Folded in from the `App.chest_names` side map by
    /// M6-SAVE-1 phase 3 — the entity owns its name from the file
    /// (`LoadChests:2735`), and the C#'s null default reads as the empty
    /// string.
    pub(crate) name: String,
    /// `Chest.maxItems` (`Chest.cs:70`'s neighbour, `:78`): the per-chest slot
    /// count from the file's `i32` (`SaveChests:1654`, `LoadChests:2738`), the
    /// default [`DEFAULT_MAX_ITEMS`] everywhere else. `slots.len()` is the max
    /// for a loaded chest; `Resize` semantics stay `Chest.cs:118-130`.
    pub(crate) max_items: usize,
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
        self.slots.iter().all(|s| s.kind == 0 && s.count == 0)
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
        name: String::new(),
        max_items: DEFAULT_MAX_ITEMS as usize,
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
                name: String::new(),
                max_items: DEFAULT_MAX_ITEMS as usize,
            };
        }
    }
}

/// The chest's slots at `idx` — the plan contract's
/// `get(idx) -> Option<&[ItemSlot]>` over `Chest.item[]`
/// (`Chest.cs:74-78`). `None` for an out-of-range index; a removed slot
/// reads as absent.
pub(crate) fn chest_slots(chests: &[WorldChest], idx: usize) -> Option<&[ItemSlot]> {
    chests
        .get(idx)
        .filter(|c| !is_removed(c))
        .map(|c| c.slots.as_slice())
}

/// Mutable [`chest_slots`] — the plan contract's `get_mut`
/// (`Chest.cs:74-78`).
pub(crate) fn chest_slots_mut(chests: &mut [WorldChest], idx: usize) -> Option<&mut [ItemSlot]> {
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

// --- `.wld` persistence (M6-SAVE-1 phase 3) ---------------------------------
//
// `WorldFile.SaveChests:1606` (write) / `LoadChests:2723` (read), plus every
// fix-up the C# applies — the duplicate-position and tail passes
// (`LoadChests:2781-2800`), the pre-115 `FixDresserChests:3269`, the
// `Chest.FixLoadedData:3664` item chain (`Item.FixAgainstExploit`,
// `Item.cs:49489-49504`) and the save-side pre-pass that destroys chests with
// dead anchors (`SaveChests:1609-1642`). The plain wire records are
// `tr_core::wld::sections::{WldChest, WldChestSlot}` — the codec's both halves
// live there (`wld/sections/chests.rs`); this is the app-side seam the
// world-file wiring calls.

/// `LoadChests:2732-2800`: the file's records become the store entries, in
/// file order and index-stable (a dropped chest is a tombstone, not a
/// compaction — the C# array slot is nulled, `Chest.RemoveChest:132-141`, and
/// `Player.chest` holds the index). The tail clear (`:2797-2800`) is a no-op
/// here: the port's store is built fresh from the file, so nothing beyond the
/// loaded count exists. The item-level fix-ups run last, exactly as the C#
/// runs them in `LoadWorld_LastMinuteFixes` → `FixAgainstExploits:1864`
/// *after* every section has read.
pub fn from_wld(records: &[WldChest]) -> Vec<WorldChest> {
    let mut chests: Vec<WorldChest> = records
        .iter()
        .map(|r| WorldChest {
            tile: (r.x, r.y),
            slots: slots_from_wld(&r.slots, r.max_items.max(0) as usize),
            name: r.name.clone(),
            max_items: r.max_items.max(0) as usize,
        })
        .collect();
    drop_duplicate_positions(&mut chests);
    fix_loaded_data(&mut chests);
    chests
}

/// `chest.Resize(num3)` + `FillWithEmptyInstances` (`Chest.cs:118-130`,
/// `:157-164`): the loaded slot list is `maxItems` long, the C# array shape —
/// the implicit-air tail is materialized on load, and the free-slot walks the
/// generic transfers run see the same 40 (or per-chest N) entries.
fn slots_from_wld(records: &[WldChestSlot], max_items: usize) -> Vec<ItemSlot> {
    let mut slots: Vec<ItemSlot> = Vec::with_capacity(max_items);
    for rec in records.iter().take(max_items) {
        slots.push(slot_from_wld(rec));
    }
    slots.resize(max_items, EMPTY_SLOT);
    slots
}

/// `LoadChests:2754-2771`'s item build: a stored record is
/// `item.netDefaults(type); item.stack = num6; item.Prefix(prefix)`. The
/// type-range guard of `SetDefaults` (`Item.cs:49929-49932`: a type at or past
/// `ItemID.Count` turns to air) runs here, before the exploit fix-ups — the
/// C# order. An unregistered kind (none in 1.4.5.8) reads as air too.
fn slot_from_wld(rec: &WldChestSlot) -> ItemSlot {
    let Ok(kind) = u16::try_from(rec.kind) else {
        return EMPTY_SLOT;
    };
    if rec.stack <= 0 || registry::get(kind).is_none() {
        return EMPTY_SLOT;
    }
    ItemSlot {
        kind,
        name: registry::name(kind),
        count: rec.stack as u32,
        prefix: rec.prefix as u16,
    }
}

/// `LoadChests:2781-2796`: the duplicate `(x, y)` removal. The C# walks `l`
/// ascending, keeps the FIRST chest at a position and removes every later one
/// (`Chest.RemoveChest(l)`) — so the survivor is the earliest record, not the
/// last as a naive dedup would pick.
fn drop_duplicate_positions(chests: &mut Vec<WorldChest>) {
    let mut kept: Vec<(i32, i32)> = Vec::new();
    for i in 0..chests.len() {
        if is_removed(&chests[i]) {
            continue;
        }
        let at = chests[i].tile;
        if kept.contains(&at) {
            remove_chest(chests, i); // `LoadChests:2789`
        } else {
            kept.push(at); // `LoadChests:2793`
        }
    }
}

/// `Chest.FixLoadedData:3664-3671` over every chest's slots, in the C#'s
/// order: an out-of-range or banned item becomes air, an over-stack is clamped
/// to `maxStack`, an unrollable prefix is reset — `Item.FixAgainstExploit`
/// (`Item.cs:49489-49504`), one decision per item in that order.
fn fix_loaded_data(chests: &mut [WorldChest]) {
    for chest in chests.iter_mut().filter(|c| !is_removed(c)) {
        for slot in &mut chest.slots {
            fix_against_exploit(slot);
        }
    }
}

/// `Item.FixAgainstExploit` (`Item.cs:49489-49504`) for one slot: the banned
/// check first (it returns — an air item skips the stack/prefix arms), then
/// the stack clamp, then the prefix reset.
fn fix_against_exploit(slot: &mut ItemSlot) {
    if registry::get(slot.kind).is_none() || BANNED_IN_INVENTORY.contains(&slot.kind) {
        *slot = EMPTY_SLOT;
        return;
    }
    let max = registry::max_stack(slot.kind);
    if slot.count > max {
        slot.count = max;
    }
    if slot.prefix != 0 && !can_roll_prefix(slot.kind, slot.prefix) {
        slot.prefix = 0;
    }
}

/// `Item.CanRollPrefix` (`Item.cs:620-635`): the prefix must be in the kind's
/// own rollable table (`Item.GetRollablePrefixes`, `Item.cs:1101-1137` — the
/// port's `prefix::rollable_prefixes`).
fn can_roll_prefix(kind: u16, prefix: u16) -> bool {
    crate::prefix::rollable_prefixes(kind).is_some_and(|set| set.contains(&prefix))
}

/// `FixDresserChests:3269-3282`, run by `LoadChests:2801-2804` when the file's
/// version is below 115: every dresser anchor (tile 88 at a `frameX % 54 == 0`
/// / `frameY % 36 == 0` corner) gets a chest through
/// `Chest.CreateChest(i, j)` (`Chest.cs:590-604`) — which refuses a taken
/// anchor and a full store with the C# `-1`. The caller gates the version.
pub fn fix_dresser_chests(chests: &mut Vec<WorldChest>, world: &World) {
    for x in 0..world.width {
        for y in 0..world.height {
            let Some(t) = world.tile(x, y) else { continue };
            if !t.active() || t.kind != 88 {
                continue;
            }
            if t.frame_x % 54 != 0 || t.frame_y % 36 != 0 {
                continue;
            }
            // `Chest.CreateChest` → `FindEmptyChest` (`Chest.cs:522-541`):
            // a taken anchor or a full store is the C# `-1`, silently skipped.
            let Some(i) = find_empty_chest(chests, x, y) else {
                continue;
            };
            create_chest(chests, x, y);
            // `CreateWorldChest`'s `FillWithEmptyInstances` (`Chest.cs:107-109`,
            // `:157-164`): the created chest carries `maxItems` real entries,
            // not the open path's lazy empty list.
            if let Some(chest) = chests.get_mut(i) {
                let max = chest.max_items;
                chest.slots.resize(max, EMPTY_SLOT);
            }
        }
    }
}

/// `SaveChests:1609-1642`'s pre-pass: a chest whose four anchor cells are not
/// all active `tileContainer` tiles is destroyed *before* the write — the C#
/// mutates `Main.chest` here, and the section then writes only the survivors.
/// The port does the same to its store (the tombstone keeps the C#'s index
/// stability); [`to_wld`] skips the tombstoned, so the chests really are
/// dropped from the file.
pub fn destroy_invalid_anchors(chests: &mut Vec<WorldChest>, world: &World) {
    for i in 0..chests.len() {
        if is_removed(&chests[i]) {
            continue;
        }
        let (x, y) = chests[i].tile;
        let mut dead = false;
        'anchors: for dx in 0..=1 {
            for dy in 0..=1 {
                // `j < 0 || k < 0 || j >= maxTilesX || k >= maxTilesY`
                // (`:1621`) — the port's tile probe covers both bounds.
                let Some(t) = world.tile(x + dx, y + dy) else {
                    dead = true;
                    break 'anchors;
                };
                if !t.active() || !is_tile_container(t.kind) {
                    dead = true;
                    break 'anchors;
                }
            }
        }
        if dead {
            remove_chest(chests, i); // `Chest.RemoveChest(i)` (`:1636`)
        }
    }
}

/// `SaveChests:1644-1673`: the store entries become the section's wire list.
/// Tombstones are skipped (the C# `chest == null` continue), each slot writes
/// the bare `i16 0` for air and `(stack, type, prefix)` otherwise, and every
/// chest writes exactly `maxItems` records — the port's implicit airs are
/// materialized for the file (`SaveChests:1655`'s `l < maxItems` loop).
pub fn to_wld(chests: &[WorldChest]) -> Vec<WldChest> {
    chests
        .iter()
        .filter(|c| !is_removed(c))
        .map(|c| WldChest {
            x: c.tile.0,
            y: c.tile.1,
            name: c.name.clone(),
            max_items: c.max_items as i32,
            slots: (0..c.max_items)
                .map(|i| slot_to_wld(c.slots.get(i).copied().unwrap_or(EMPTY_SLOT)))
                .collect(),
        })
        .collect()
}

/// `SaveChests:1657-1672`: an air slot is the bare `i16 0`; a stored item
/// writes its stack first and — because the stack is positive — the
/// `(type, prefix)` pair. The stack is the C#'s `(short)item.stack` cast, so a
/// count above `i16::MAX` wraps exactly as the C# unchecked cast does (no
/// legit store reaches it: `maxStack` caps at 9999).
fn slot_to_wld(slot: ItemSlot) -> WldChestSlot {
    if slot.kind == 0 || slot.count == 0 {
        return WldChestSlot::default();
    }
    WldChestSlot {
        stack: slot.count as i16,
        kind: slot.kind as i32,
        prefix: slot.prefix as u8,
    }
}

// --- Panel name defaulting (M3-CHEST-UI phase 3) ----------------------------
//
// `ChestUI.DrawName`'s tile-style chain (`ChestUI.cs:124-140`). The vanilla
// strings ride the embedded `LegacyChestType.*` localization JSON, which is
// not on disk in this repo, so the names derive from the port's generated
// item table — the item placing `(tile, place_style)` carries the same en-US
// string (recorded deviation, phase 6 `docs/ANALYSIS.md`). The `.wld` load
// path takes its names from the file (`LoadChests:2735`) instead.

/// The chest's default name from the tile at the panel's origin
/// (`ChestUI.cs:124-140`): tile 21 → `Lang.chestType[frameX / 36]`, tile 467
/// style 4 → item 3988's name, tile 467 → `Lang.chestType2[frameX / 36]`,
/// tile 88 → `Lang.dresserType[frameX / 54]`. An empty string when the style
/// has no name (styles that cannot occur through placement — see
/// [`chest_type_name`]).
pub(crate) fn default_chest_name(kind: u16, frame_x: i16) -> String {
    if kind == 21 {
        return chest_type_name(frame_x as i32 / 36, 21);
    }
    if kind == 467 {
        // `ChestUI.cs:129-131`: the Dead Man's Chest is style 4.
        let style = frame_x as i32 / 36;
        if style == 4 {
            return crate::registry::name(3988).to_string();
        }
        return chest_type_name(style, 467);
    }
    if kind == 88 {
        return chest_type_name(frame_x as i32 / 54, 88);
    }
    String::new()
}

/// The `LegacyChestType.{style}` / `LegacyChestType2.{style}` /
/// `LegacyDresserType.{style}` string for a `(tile, style)`: the vanilla name
/// of the item that places it. Tile 21's non-placeable locked frames render
/// the vanilla `Locked <twin>` pattern — `Chest.Lock` shifts the style by +1
/// (frameX +36) or +5 (frameX +180, the biome chests 18-22), `Chest.cs:449-464`,
/// and the localization table names those frames "Locked <chest>" (styles
/// 2/4 are the pinned "Locked Gold/Shadow Chest").
fn chest_type_name(style: i32, kind: u16) -> String {
    let find = |place_style: i32| {
        crate::itemdata::ITEMS
            .iter()
            .find(|d| d.tile == Some(kind) && d.place_style == place_style)
            .map(|d| d.name)
    };
    // `Lang.cs:503-506`: `chestType[52..]` aliases `chestType[0]` (the table
    // is 54 entries for a style space that stops at 53).
    let style = if kind == 21 && style >= 52 { 0 } else { style };
    if let Some(name) = find(style) {
        return name.to_string();
    }
    let twin = match (kind, style) {
        (21, 2) | (21, 4) | (21, 36) | (21, 38) | (21, 40) => Some(style - 1),
        (21, 23..=27) => Some(style - 5),
        _ => None,
    };
    twin.and_then(find)
        .map(|name| format!("Locked {name}"))
        .unwrap_or_default()
}

// --- Open/close + locking interaction (M3-CHEST-UI phase 2) ----------------
//
// The `Player.cs` chest branch: the right-click open path
// (`TileInteractionsUse`, `Player.cs:33056-33260`), `Player.OpenChest`
// (`:33505-33526`) and the per-tick range close
// (`HandleBeingInChestRange`, `:31015-31097` + the hitbox table
// `:31099-31124`), plus `Chest.Lock`/`Chest.Unlock` (`Chest.cs:346-496`).
// Out: the rigged-chest arm (`:33112`/:33233/:33253-33258 — wiring, no
// owning task), the net messages (31/33/52/59), `Main.stackSplit = 600`
// (`:33117`/:33232 — the net quick-stack lock, M3-STORE-2), the void-bag
// key arm (`:33200-33221`, M3-STORE-1), the bank tiles 29/97/463/491
// (`:33056`'s non-`BasicChest` half, M3-STORE-1) and the achievements
// (`HandleSpecialEvent(16)` `:33243-33246`, `NotifyProgressionEvent(19/20)`
// `Chest.cs:365`/:382/:403/:477 — no `CondKind` arm yet).

/// `TileID.Sets.BasicChest` (`TileID.cs:570`) — the world-chest tiles the
/// open/lock paths serve. The bank families (29/97/463/491) stay out until
/// M3-STORE-1.
fn is_basic_chest(kind: u16) -> bool {
    kind == 21 || kind == 467
}

/// The 2×2 chest's top-left tile from a clicked cell
/// (`Player.cs:33062-33066`): the frame's sheet column reduced mod 2 (the
/// `for (num74 = frameX / 18; num74 > 1; num74 -= 2)` loop) and the row from
/// `frameY / 18`. `None` unless the cell is an active `BasicChest` tile (the
/// C# interaction chain only runs on active tiles, `:32035-32041`).
fn chest_origin(world: &World, x: i32, y: i32) -> Option<(i32, i32)> {
    let t = world.tile(x, y)?;
    if !t.active() || !is_basic_chest(t.kind) {
        return None;
    }
    let mut col = t.frame_x / 18;
    while col > 1 {
        col -= 2;
    }
    Some((x - col as i32, y - (t.frame_y / 18) as i32))
}

/// The locked chest's key item id (`Player.cs:33145-33172`) — frame-derived,
/// not style-table-derived: 327 (golden key) default, 329 (shadow key) for
/// `frameX` 144-178, `1533 + (style - 23)` (the biome keys) for 828-1006 and
/// 4714 (temple key) for tile 467 style 13.
fn locked_key_id(world: &World, x: i32, y: i32) -> u16 {
    let Some(t) = world.tile(x, y) else {
        return 327;
    };
    let fx = t.frame_x;
    let mut key = 327u16;
    if t.kind == 21 {
        if (144..=178).contains(&fx) {
            key = 329; // `:33150-33153`
        }
        if (828..=1006).contains(&fx) {
            // `:33154-33164` — the style column: `frameX / 18` halved down
            // (`num78`/`num79`), the key ids 1533-1537.
            let mut col = fx / 18;
            let mut style = 0i16;
            while col >= 2 {
                col -= 2;
                style += 1;
            }
            key = (1533 + (style - 23)) as u16;
        }
    } else if t.kind == 467 && fx / 36 == 13 {
        key = 4714; // `:33166-33171`
    }
    key
}

/// `Chest.Unlock` (`Chest.cs:346-426`): shift the locked frames off the 2×2
/// (`-36`, or `-180` for the Plantera-gated styles 23-27; `-36` for tile 467
/// style 13, also gated — `Chest.cs:376`/:397). Any missing tile of the 2×2
/// refuses (`:348-351`); an unsupported style refuses (`default`/`else`).
/// `plant_boss` is `NPC.downedPlantBoss`'s stand-in — the flag has no world
/// home yet (the recorded assumption fallback: the call site passes `true`;
/// flip it to the flag when the boss work lands it). The sound (22,
/// `Chest.cs:408`) and the dust/achievements are the caller's/the phase-6
/// deviation's — this returns whether the unlock happened.
pub(crate) fn unlock_chest(world: &mut World, x: i32, y: i32, plant_boss: bool) -> bool {
    for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
        if world.tile(x + dx, y + dy).is_none() {
            return false;
        }
    }
    // `Chest.cs:352-407` — the style table (`num2 = frameX / 36`).
    let (kind, style) = {
        let t = world.tile(x, y).expect("the origin tile was just checked");
        (t.kind, t.frame_x / 36)
    };
    let shift = match kind {
        21 => match style {
            2 | 4 | 36 | 38 | 40 => Some(36),
            23..=27 => {
                if !plant_boss {
                    return false; // `Chest.cs:376-379`
                }
                Some(180)
            }
            _ => None,
        },
        467 if style == 13 => {
            if !plant_boss {
                return false; // `Chest.cs:397-400`
            }
            Some(36)
        }
        _ => None,
    };
    let Some(shift) = shift else {
        return false;
    };
    // `Chest.cs:409-424` — every 2×2 tile of the chest's type shifts.
    shift_chest_frames(world, x, y, kind, -shift);
    true
}

/// `Chest.Lock` (`Chest.cs:428-496`): the mirror shift (`+36`, or `+180` for
/// the Plantera-gated styles 18-22; tile 467 style 12 → 13, gated —
/// `Chest.cs:454`/:472). Its player-side caller (the lock-item use arm,
/// `Player.cs:41651`) stays out — it is an `editing.rs` tile-use path
/// (PHYS-7's file); the function is ported + tested for the DW-2.5 table.
pub(crate) fn lock_chest(world: &mut World, x: i32, y: i32, plant_boss: bool) -> bool {
    for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
        if world.tile(x + dx, y + dy).is_none() {
            return false;
        }
    }
    // `Chest.cs:434-481` — the style table.
    let (kind, style) = {
        let t = world.tile(x, y).expect("the origin tile was just checked");
        (t.kind, t.frame_x / 36)
    };
    let shift = match kind {
        21 => match style {
            1 | 3 | 35 | 37 | 39 => Some(36),
            18..=22 => {
                if !plant_boss {
                    return false; // `Chest.cs:454-457`
                }
                Some(180)
            }
            _ => None,
        },
        467 if style == 12 => {
            if !plant_boss {
                return false; // `Chest.cs:472-475`
            }
            Some(36)
        }
        _ => None,
    };
    let Some(shift) = shift else {
        return false;
    };
    // `Chest.cs:483-494` — every 2×2 tile of the chest's type shifts.
    shift_chest_frames(world, x, y, kind, shift);
    true
}

/// The shared `Unlock`/`Lock` frame pass (`Chest.cs:409-424`/:483-494): the
/// shift applies to every 2×2 tile whose type matches the chest's.
fn shift_chest_frames(world: &mut World, x: i32, y: i32, kind: u16, shift: i16) {
    for dx in 0..=1 {
        for dy in 0..=1 {
            if let Some(t) = world.tile_mut(x + dx, y + dy) {
                if t.kind == kind {
                    t.frame_x += shift;
                }
            }
        }
    }
}

/// `Player.OpenChest` (`Player.cs:33505-33526`): the index, the crafting
/// panel close (`NewCraftingUI.Close(quiet: true)` `:33516`), the inventory
/// open (`Main.playerInventory = true` `:33517`) and the tile origin — set
/// for the banks too (`:33524-33525`), or the per-tick range close would
/// probe a stale tile. The `ItemSlot.SetGlowForChest`/input-navigator arms
/// are UI state the port does not carry (`:33508-33515`/:33518-33523).
///
/// The browsed chest's 40-slot window materializes on open: the C#
/// `Chest.item[]` is 40 real entries for every chest (`Chest.cs:107-109`,
/// `Chest.CreateWorldChest` fills them), and the generic `bank.rs` transfers
/// walk all 40 free slots (`ChestUI.cs:1262-1287`'s `item2[j].stack == 0`,
/// `:1115-1147`'s refill) — a scan the implicit-air `Vec` tail hides when it
/// is not stored. Never-opened store entries stay empty, so the lazy-store
/// memory shape (the plan's "`Vec<ItemSlot>` implicit-air kept" decision)
/// only grows to the C#'s own per-chest shape for chests the player opens.
fn open_chest(app: &mut App, x: i32, y: i32, new_chest: i32) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    p.chest = new_chest;
    p.chest_x = x;
    p.chest_y = y;
    app.crafting_open = false;
    app.inventory_open = true;
    if new_chest >= 0 {
        if let Some(c) = app.world_chests.get_mut(new_chest as usize) {
            // `Chest.item[]` is `maxItems` real entries (`Chest.cs:107-109`) —
            // the generic `bank.rs` transfers walk them all; the chest's own
            // count is the file's per-chest one.
            c.slots.resize(c.max_items, EMPTY_SLOT);
        }
    }
}

/// `Player.cs:33230-33252` — the open toggle shared by every container kind
/// (the bank tiles land here from editing.rs's `BankOpen` arm, the world
/// chests from [`try_open_chest_at_cursor`]): the same index closes (sound
/// 11), a fresh open plays 10 (the `frameX 36..72` Shadow-chest special
/// event 16 arm `:33243-33246` is the recorded achievements deviation), a
/// re-open with another container already open plays 12. Returns the legacy
/// sound id (the `SoundEngine.PlaySound` argument) so the caller plays it
/// and the tests pin it.
pub(crate) fn toggle_open(app: &mut App, x: i32, y: i32, new_chest: i32) -> u32 {
    let Some(current) = app.player.as_ref().map(|p| p.chest) else {
        return 0; // no player — no container state (the front screens park the routes)
    };
    if new_chest == current {
        if let Some(p) = app.player.as_mut() {
            p.chest = -1; // `:33236` — `chestX/Y` stay
        }
        return 11; // `:33237`
    }
    let was_closed = current == -1; // `:33239`
    open_chest(app, x, y, new_chest);
    if was_closed {
        10 // `:33242`
    } else {
        12 // `:33251`
    }
}

/// The container-tile origin decode the C# runs ahead of the shared open
/// block (`Player.cs:33060-33079`): the frame's sheet column reduced mod 2
/// and the row from `frameY / 18` — `chest_origin`'s loop — plus the 463/491
/// adjustment (`frameX != 36 ? +1 : -1`, `y += 2`) that lands on the 3×4
/// bbox anchor the hitbox table expects (`:31103-31105`). editing.rs's
/// `BankOpen` arm feeds the pair to [`toggle_open`] (the C# passes the same
/// `num74/num75` to `OpenChest`, `:33241`/:33249).
pub(crate) fn bank_open_origin(world: &World, x: i32, y: i32) -> (i32, i32) {
    let Some(t) = world.tile(x, y) else {
        return (x, y);
    };
    let mut col = t.frame_x / 18;
    while col > 1 {
        col -= 2;
    }
    let mut ox = x - col as i32;
    let mut oy = y - (t.frame_y / 18) as i32;
    if t.kind == 463 || t.kind == 491 {
        ox += if t.frame_x != 36 { 1 } else { -1 };
        oy += 2;
    }
    (ox, oy)
}

pub(crate) fn play_menu_sound(app: &App, legacy_id: u32) {
    // `SoundID.MenuOpen/MenuClose/MenuTick = 10/11/12` (`SoundID.cs:42/44/46`),
    // played without a position in the C# (`:33242`/:33237/:33251).
    match legacy_id {
        10 => app.audio.play("Menu_Open"),
        11 => app.audio.play("Menu_Close"),
        12 => app.audio.play("Menu_Tick"),
        _ => {}
    }
}

/// The right-click open path (`Player.cs:33056-33260`), the editing.rs
/// interception's callee — mirrors `try_open_shop_at_cursor`'s contract:
/// `true` consumes the click. Every active `BasicChest` tile consumes
/// (`flag2`/`Main.mouseRightRelease = false`, `:33058-33059`), even a locked
/// one the key arm can't open.
pub(crate) fn try_open_chest_at_cursor(app: &mut App, world_pos: Vec2) -> bool {
    let x = (world_pos.x / TILE_PX).floor() as i32;
    let y = (world_pos.y / TILE_PX).floor() as i32;
    let Some((ox, oy)) = chest_origin(&app.world, x, y) else {
        return false;
    };
    // `:33087-33090` — `CloseSign`/`SetTalkNPC(-1)`/the chat-text clears:
    // the port's dialog + shop close (`Main.SetNPCShopIndex(-1)`'s
    // equivalent). The `editedChestName` net flush (`:33097-33101`) and the
    // MP arm (`:33103-33119`) are out.
    app.close_shop();
    // `:33091-33096` — a chest-tile click while the rename editor is live
    // flushes it: sound 12, `editChest = false`, `npcChatText = ""`. The
    // port's modal keyboard parks the world routes, so production never
    // reaches this with an open editor — the arm is the overlay shape's
    // revive point (the phase-5 discovery, Gap #6).
    if app.chest_editing() {
        play_menu_sound(app, 12);
        app.front = None;
    }
    // `:33142-33228` — the single-player default arm (`num73 == 0` for the
    // `BasicChest` tiles): `flag14` marks a locked-chest click, which blocks
    // the `FindChest` at `:33223-33226`, so the click is consumed without
    // opening — the chest opens on the next click.
    let locked = is_locked(&app.world, ox, oy); // `:33143`
    if locked {
        let key = locked_key_id(&app.world, ox, oy);
        // `:33175` — `flag16`: the shadow key (329) unlocks without
        // consuming; every other key decrements.
        let consumes = key != 329;
        // `:33176-33199` — the key scan over `inventory[0..=58]`: the port's
        // inventory grid (0-49), the coins (50-53), the ammo (54-57) and the
        // trash slot (58) — `EquipSlots` mirrors the C# array layout
        // (`inv.rs:26-34`); the equipment rows are outside the scan. The
        // loop keeps scanning after a success, but `Chest.Unlock` answers
        // false once the chest is unlocked — exactly one key is consumed.
        // (`flag15`'s only reader is the void-bag arm `:33200`, which is out.)
        for (slot, is_trash) in app
            .inventory
            .iter_mut()
            .map(|s| (s, false))
            .chain(app.equip.coins.iter_mut().map(|s| (s, false)))
            .chain(app.equip.ammo.iter_mut().map(|s| (s, false)))
            .chain(std::iter::once((&mut app.equip.trash, true)))
        {
            if slot.kind != key || slot.count == 0 {
                continue;
            }
            // The Plantera gate passes until the world flag exists (the
            // recorded fallback — flip to the flag when the boss work lands).
            if !unlock_chest(&mut app.world, ox, oy, true) {
                continue;
            }
            if consumes {
                // `:33183-33189` — the consumed key's stack decrements; an
                // emptied slot turns to air.
                slot.count -= 1;
                if slot.count == 0 {
                    *slot = EMPTY_SLOT;
                }
                if is_trash {
                    // `:33190-33193` — the trash slot mirrors into the
                    // cursor stack (`Main.mouseItem`).
                    app.mouse_item = *slot;
                }
            }
        }
        // `:33200-33221` — the void-bag arm is M3-STORE-1.
    }
    if !locked {
        // `:33225` — `Chest.FindChest` on an unlocked click. The port creates
        // the store entry lazily on the first open (the placement-time
        // creation is unreachable without editing.rs edits — the recorded
        // deviation; an empty entry answers like an absent one).
        let new_chest = match find_chest(&app.world_chests, ox, oy) {
            Some(i) => i,
            None => create_chest(&mut app.world_chests, ox, oy),
        };
        // `:33230-33252` — the open toggle. (`Main.stackSplit = 600`
        // `:33232` and the rigged-chest arm are out.)
        let sound = toggle_open(app, ox, oy, new_chest as i32);
        play_menu_sound(app, sound);
    }
    true
}

/// `Player.HandleBeingInChestRange` (`Player.cs:31015-31097`) — the per-tick
/// close for every open container (the unified `Player.chest` covers the
/// world chests and the banks): out of the container's multi-tile-hitbox
/// interaction range or a dead origin tile force-closes with sound 11
/// (`:31078-31090`). The bank-projectile arms (`:31019-31076`) are the
/// remote opens — out of scope. Runs for alive players only (the C# call
/// site sits after the dead branch's return, `:24752`/:25290) — the
/// `update_player` hook inherits that gating.
pub(crate) fn tick_open(app: &mut App) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    if p.chest == -1 {
        return; // `:31017`
    }
    let (chest_x, chest_y) = (p.chest_x, p.chest_y);
    // Both probes read the world only; the close below mutates the player
    // after the borrow ends.
    let in_range = in_interaction_range_to_multi_tile_hitbox(&app.world, p, chest_x, chest_y);
    let origin_active = app.world.tile(chest_x, chest_y).is_some_and(|t| t.active());
    // `:31078-31085` — the range close plays sound 11.
    if !in_range {
        let id = force_close(app);
        play_menu_sound(app, id);
        return;
    }
    // `:31086-31090` — a dead tile force-closes (the `Main.cs:41820-41823`
    // stale-chest equivalent rides this check).
    if !origin_active {
        let id = force_close(app);
        play_menu_sound(app, id); // `:31088` — `SoundEngine.PlaySound(11)`
    }
}

/// `:31080-31090` — the force close: `chest = -1`. Returns the legacy sound
/// id (11 — `SoundEngine.PlaySound(11)`, played by the caller) so the tests
/// pin it.
fn force_close(app: &mut App) -> u32 {
    if let Some(p) = app.player.as_mut() {
        p.chest = -1;
    }
    11
}

/// The per-tile hitbox rect (`Player.cs:31101-31118`), in world px, at the
/// container origin: the 463/491 row (3×4 at −1/−2 tiles), the
/// `BasicChest`/97 row (2×2), the 88 dresser row (3×2) and the 29 piggy row
/// (2×1). An unmatched kind answers `Rectangle.Empty` — `(0, 0, 0, 0)`
/// (`:31100`).
fn multi_tile_hitbox_px_rect(kind: u16, x: i32, y: i32) -> (i32, i32, i32, i32) {
    if kind == 463 || kind == 491 {
        return (x * 16 - 16, y * 16 - 32, 48, 64); // `:31103-31105`
    }
    if is_basic_chest(kind) || kind == 97 {
        return (x * 16, y * 16, 32, 32); // `:31107-31109`
    }
    if kind == 88 {
        return (x * 16, y * 16, 48, 32); // `:31110-31112`
    }
    if kind == 29 {
        return (x * 16, y * 16, 32, 16); // `:31115-31117`
    }
    (0, 0, 0, 0)
}

/// `Player.IsInInteractionRangeToMultiTileHitbox` (`Player.cs:31099-31124`):
/// the tile kind's rect at the container origin, deflated by
/// `Inflate(-1, -1)` (`:31119`), the player's center clamped in
/// (`Utils.ClosestPointInRect`, `Utils.cs:1581-1595`), the `(int)`-truncated
/// tile point (`ToTileCoordinates`) tested against the
/// `TileReachCheckSettings.Simple` bounds
/// (`DataStructures/TileReachCheckSettings.cs:45-54/:81-85` — ranges 5/3,
/// `Player.cs:57113-57114`, under the 20 cap; `Collision.InTileBounds`,
/// `Collision.cs:3776`, inclusive).
fn in_interaction_range_to_multi_tile_hitbox(
    world: &World,
    p: &player::Player,
    chest_x: i32,
    chest_y: i32,
) -> bool {
    let kind = world.tile(chest_x, chest_y).map_or(0, |t| t.kind);
    let (rx, ry, rw, rh) = multi_tile_hitbox_px_rect(kind, chest_x, chest_y);
    // `r.Inflate(-1, -1)` (`:31119`) — X += 1, Y += 1, W -= 2, H -= 2 (the
    // Empty rect degenerates to an inverted (1, 1, -2, -2)).
    let (rx, ry, rw, rh) = (rx + 1, ry + 1, rw - 2, rh - 2);
    let (left, top) = (rx as f32, ry as f32);
    let (right, bottom) = ((rx + rw) as f32, (ry + rh) as f32);
    let (center_x, center_y) = p.center();
    // `ClosestPointInRect`'s two sequential ifs (`Utils.cs:1583-1592`) —
    // `f32::clamp` would panic on the Empty rect's inverted bounds; the
    // chain answers (-1, -1) there, exactly like the C#.
    let mut px = center_x;
    if px < left {
        px = left;
    }
    if px > right {
        px = right;
    }
    let mut py = center_y;
    if py < top {
        py = top;
    }
    if py > bottom {
        py = bottom;
    }
    // `ToTileCoordinates` — the C# `(int)` truncation.
    let tx = (px / 16.0) as i32;
    let ty = (py / 16.0) as i32;
    // `IsInTileInteractionRange` (`Player.cs:32025-32030`) over the Simple
    // region (`TileReachCheckSettings.cs:45-54`).
    let lx = (p.x / 16.0) as i32 - 5;
    let hx = ((p.x + player::WIDTH) / 16.0).ceil() as i32 - 1 + 5;
    let ly = (p.y / 16.0) as i32 - 3;
    let hy = ((p.y + p.height) / 16.0).ceil() as i32 - 1 + 3;
    (lx..=hx).contains(&tx) && (ly..=hy).contains(&ty)
}

// --- Slot interactions (M3-CHEST-UI phase 4) --------------------------------
//
// The chest-side click helpers: the shift transfers (`ItemSlot.OverrideLeftClick`
// cursor 8/9, `ItemSlot.cs:800-850`), `ChestUI.TryPlacingInChest`
// (`ChestUI.cs:1209-1289`) and the favorite context tables (`ItemSlot.cs:
// 386-394`). The per-slot click dispatch itself lives in `app/clicks.rs`
// (`handle_chest_clicks`); the right-click stack-split's timing state is
// `Main.stackSplit`'s cluster (`Main.cs:1693-1697`), ticked by `app/frame.rs`.
// Out: the bank containers (M3-STORE-1 — `GetCurrentContainer`'s chest < -1
// arms and the bank-only `IsBlockedFromTransferIntoChest`, `ChestUI.cs:
// 1291-1302`), the net messages 32/38 (`:1247`/:1258/:1283, `ItemSlot.cs:808`),
// `DoCoins`' roll-over (`Player.cs:39567-39569` — unported everywhere), the
// cursor-override texture draw (see `App.cursor_override`).

/// Reads chest slot `slot` of the store entry `idx` with the implicit-air
/// encoding (decision #4): positions beyond `slots.len()` answer
/// [`EMPTY_SLOT`], the C#'s air `Chest.item[i]` entries. `None` for a
/// missing/removed entry.
pub(crate) fn chest_slot(chests: &[WorldChest], idx: usize, slot: usize) -> Option<ItemSlot> {
    chest_slots(chests, idx).map(|slots| slots.get(slot).copied().unwrap_or(EMPTY_SLOT))
}

/// Writes chest slot `slot` of the entry `idx`, extending with airs first when
/// the position is past the stored length (decision #4). The written value is
/// normalized to air — the C#'s `SetDefaults(0)` / `new Item()` writes
/// (`ItemSlot.cs:976-979`, `ChestUI.cs:1244/:1278`) — so the store never holds
/// a half-air slot, and a real item's `name` is derived from the registry at
/// write (the store's items are registry-named like the C#'s `SetDefaults`
/// entries). `false` for a missing/removed entry.
pub(crate) fn set_chest_slot(
    chests: &mut [WorldChest],
    idx: usize,
    slot: usize,
    value: ItemSlot,
) -> bool {
    let Some(chest) = chests.get_mut(idx).filter(|c| !is_removed(c)) else {
        return false;
    };
    let value = if value.kind == 0 || value.count == 0 {
        EMPTY_SLOT
    } else {
        ItemSlot {
            name: registry::name(value.kind),
            ..value
        }
    };
    if slot >= chest.slots.len() {
        chest.slots.resize(slot + 1, EMPTY_SLOT);
    }
    chest.slots[slot] = value;
    true
}

/// `ItemSlot.canFavoriteAt` (`ItemSlot.cs:386-394`): only the inventory
/// contexts 0/1/2/32 can favorite — a chest slot (3) cannot, so its items
/// never carry the flag and the cursor-3 favorite arm never fires on them.
pub(crate) fn can_favorite_at(context: i32) -> bool {
    matches!(context, 0 | 1 | 2 | 32)
}

/// `ItemSlot.retainFavoriteOnMoveAt` (`ItemSlot.cs:390-394`): the contexts
/// whose `MouseItemSwap` keeps the favorite flag (`:1255-1263` — every other
/// context clears it on both sides). Context 3 is not in the table, so
/// moving an item out of a chest clears `favorited` — the port's `ItemSlot`
/// carries no favorite field yet (item.rs:52-57), so the clear is a no-op
/// with this citation until M7-UI-2 lands the flag.
pub(crate) fn retain_favorite_on_move_at(context: i32) -> bool {
    matches!(context, 0 | 1 | 2 | 32 | 5)
}

/// `Player.ItemSpace(item).CanTakeItemToPersonalInventory`
/// (`Player.cs:39153-39197`) for the shift-out cursor: any grid slot accepts
/// the item (`CanItemSlotAcceptPickup`, `:39220-39232` — empty, or a
/// same-kind partial stack), or the ungated same-kind merge scan over the
/// ammo slots (`:39185-39191`). The `IsAPickup`/`uniqueStack` sets, the
/// `ammo > 0` FillAmmo arm (`:39175-39184`) and the Cofveve void scan
/// (`:39192-39195`) have no port counterparts yet.
pub(crate) fn inventory_has_room_for(app: &App, kind: u16) -> bool {
    if kind == 0 {
        return false;
    }
    let max = registry::max_stack(kind);
    if app
        .inventory
        .iter()
        .any(|s| s.count == 0 || (s.kind == kind && s.count < max))
    {
        return true;
    }
    app.equip
        .ammo
        .iter()
        .any(|s| s.count > 0 && s.kind == kind && s.count < max)
}

/// `ChestUI.TryPlacingInChest` (`ChestUI.cs:1209-1289`) — the shift-in
/// transfer from inventory `slot` into the open chest: a stack-merge pass
/// over every partial same-kind chest slot, then the whole remainder into the
/// first empty slot. The return mirrors the C#'s `flag2`: only the
/// `just_check` branches set it (`:1234-1237`/:1271-1274), so the real pass
/// always answers false — its only caller ignores the value
/// (`ItemSlot.cs:846`) — and mutates without playing a cue for the probe.
/// The bank containers (`chest < -1`) are M3-STORE-1;
/// `IsBlockedFromTransferIntoChest` (`:1291-1302`) is a bank-only rule —
/// noted for STORE-1.
pub(crate) fn try_placing_in_chest(app: &mut App, slot: usize, just_check: bool) -> bool {
    let chest = match app.player.as_ref() {
        Some(p) if p.chest >= 0 => p.chest as usize,
        _ => return false, // `GetCurrentContainer`'s world-chest arm (`:1213`)
    };
    // The C# `Item item = inv[slot]` is a reference: every mutation below
    // (`item.stack -= num`, the final `SetDefaults(0)`) writes through to
    // `inv[slot]` — the port's copy + write-back replicates that, keeping
    // kind, name AND prefix on the drained remainder (the M3-CHEST-INT
    // deposit-prefix fix).
    let mut item = app.inventory[slot];
    if item.kind == 0 || item.count == 0 {
        return false; // an air item never merges and never fills (`item.maxStack` 0)
    }
    let max = registry::max_stack(item.kind);
    let mut placeable = false;
    // `Chest.item.Length` is the chest's own `maxItems` (`ChestUI.cs:1221`'s
    // `for (int i = 0; i < Main.chest[chest].maxItems; i++)`) — the file's
    // per-chest count, the default 40 for every port-created chest.
    let max_items = app
        .world_chests
        .get(chest)
        .filter(|c| !is_removed(c))
        .map_or(0, |c| c.max_items);
    // The merge pass (`ChestUI.cs:1221-1261`) — stackable kinds only.
    if max > 1 {
        for i in 0..max_items {
            let Some(s) = chest_slot(&app.world_chests, chest, i) else {
                break;
            };
            // `:1224-1225` — a full slot (an air slot's maxStack 0 fails
            // `stack >= maxStack`) or one `Item.CanStack` refuses (alien
            // kind OR different prefix, `Item.cs:49865-49871`) skips.
            if s.count == 0 || s.count >= registry::max_stack(s.kind) || !can_stack(&item, &s) {
                continue;
            }
            let slot_max = registry::max_stack(s.kind);
            // `:1228-1233` — the amount, capped by the slot's room.
            let mut num = item.count;
            if item.count + s.count > slot_max {
                num = slot_max - s.count;
            }
            if just_check {
                placeable = placeable || num > 0; // `:1234-1237`
                break;
            }
            // `:1239-1241` — the merge, sound 7. (The `:1251-1255`
            // `item2[i].type == 0` re-clone arm is dead behind the `:1224`
            // skip: a non-skipped slot already stacks with the item.)
            set_chest_slot(
                &mut app.world_chests,
                chest,
                i,
                ItemSlot {
                    count: s.count + num,
                    ..s
                },
            );
            app.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:1241`)
            item.count -= num;
            if item.count == 0 {
                break; // `:1242-1249` — `SetDefaults(0)` (write-through)
            }
        }
    }
    // The empty-slot pass (`ChestUI.cs:1262-1287`) — one slot takes the whole
    // remainder.
    if item.count > 0 {
        for j in 0..max_items {
            let Some(s) = chest_slot(&app.world_chests, chest, j) else {
                break;
            };
            if s.count != 0 {
                continue; // `:1267`
            }
            if just_check {
                placeable = true; // `:1271-1274`
                break;
            }
            app.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:1276`)
            set_chest_slot(&mut app.world_chests, chest, j, item); // `:1275` — the whole clone
            item = EMPTY_SLOT; // `:1277-1278` — `SetDefaults(0)` (write-through)
            break;
        }
    }
    if !just_check {
        // The drained slot's remainder — air when fully moved
        // (`:1244`/:1278's `SetDefaults(0)` write-through).
        app.inventory[slot] = item;
    }
    placeable
}

/// The shift-out transfer of one chest slot into the player's inventory —
/// the cursor-8 arm of `ItemSlot.OverrideLeftClick` (`ItemSlot.cs:800-812`):
/// `inv[slot] = player.GetItem(inv[slot], QuickTransferFromSlot)`.
/// `GetItemSettings.QuickTransferFromSlot` (`GetItemSettings.cs:58`) is
/// NoText-only. The remainder stays in the chest slot. Out: the
/// `FitsAmmoSlot`/`FillAmmo` pre-pass (`Player.cs:39345-39352` — no
/// ammo-accept registry yet), `DoCoins` (`:39567-39569`), the void vault
/// (`:39395-39398`).
pub(crate) fn quick_transfer_from_slot(app: &mut App, chest_slot_idx: usize) {
    /// `GetItem_FillIntoOccupiedSlot`'s merge step (`Player.cs:39485-39524`):
    /// as much of `remainder` as a partial slot accepts — the accept gate is
    /// `CanItemSlotAcceptPickup` (`Player.cs:39487`), i.e. `Item.CanStack`
    /// (same kind AND same prefix, `Item.cs:49865-49871`).
    fn merge_into(s: &mut ItemSlot, item: &ItemSlot, remainder: &mut u32) -> u32 {
        let max = registry::max_stack(item.kind);
        if s.count > 0 && can_stack(s, item) && s.count < max && *remainder > 0 {
            let moved = (max - s.count).min(*remainder);
            s.count += moved;
            *remainder -= moved;
            moved
        } else {
            0
        }
    }

    /// `GetItem_FillEmptyInventorySlot` (`Player.cs:39551-39578`): one empty
    /// slot takes the whole remainder — `inventory[i] = returnItem` clones
    /// the whole item, prefix included.
    fn fill_empty(s: &mut ItemSlot, item: &ItemSlot, remainder: &mut u32) -> bool {
        if s.count == 0 && *remainder > 0 {
            *s = ItemSlot {
                count: *remainder,
                ..*item
            };
            *remainder = 0;
            true
        } else {
            false
        }
    }

    let chest_idx = match app.player.as_ref() {
        Some(p) if p.chest >= 0 => p.chest as usize,
        _ => return,
    };
    let Some(src) = chest_slot(&app.world_chests, chest_idx, chest_slot_idx) else {
        return;
    };
    if src.kind == 0 || src.count == 0 {
        return;
    }
    let is_coin = registry::is_coin(src.kind); // `Item.IsACoin` (`Item.cs:373-383`)
    let mut remainder = src.count;
    let App {
        inventory,
        equip,
        audio,
        world_chests,
        ..
    } = app;
    // The occupied-slot pass (`Player.cs:39353-39364`): coins start at the C#
    // coin slots 50-53 (the `num2 = -4` arm, `:39340-39343`), then the grid.
    // `:39444` — `IsACoin ? 38 : 7` per accepted slot; the port's coin cue is
    // "Coin_0" (the drop-pickup precedent).
    if is_coin {
        for s in equip.coins.iter_mut() {
            if merge_into(s, &src, &mut remainder) > 0 {
                audio.play("Coin_0");
            }
            if remainder == 0 {
                break;
            }
        }
    }
    for s in inventory[..INV].iter_mut() {
        if merge_into(s, &src, &mut remainder) > 0 {
            audio.play("Grab"); // `:39444` — the non-coin arm
        }
        if remainder == 0 {
            break;
        }
    }
    // The empty-slot passes (`Player.cs:39365-39394`): the hotbar-first arm
    // for usable kinds (`useStyle != 0`, `:39365-39374`), then the
    // non-favorited REVERSE scan (`:39387-39394`; chest items cannot be
    // favorited — [`can_favorite_at`] excludes ctx 3 — so the favorited
    // forward arm `:39375-39384` is unreachable here). Coins scan the coin
    // slots descending first (`num = 54`).
    if remainder > 0 && crate::item::use_style(src.kind) != 0 {
        for s in inventory[..HOTBAR].iter_mut() {
            if fill_empty(s, &src, &mut remainder) {
                audio.play("Grab"); // `:39559`
                break;
            }
        }
    }
    if remainder > 0 && is_coin {
        for s in equip.coins.iter_mut().rev() {
            if fill_empty(s, &src, &mut remainder) {
                audio.play("Coin_0"); // `:39559`
                break;
            }
        }
    }
    if remainder > 0 {
        for s in inventory[..INV].iter_mut().rev() {
            if fill_empty(s, &src, &mut remainder) {
                audio.play("Grab"); // `:39559`
                break;
            }
        }
    }
    // `:800-812`'s `inv[slot] = ...` — the unplaced remainder stays in the
    // chest slot (its prefix rides along; an empty remainder normalises to
    // air).
    set_chest_slot(
        world_chests,
        chest_idx,
        chest_slot_idx,
        ItemSlot {
            count: remainder,
            ..src
        },
    );
}

// --- Rename flow (M3-CHEST-UI phase 5) ---------------------------------------
//
// `ChestUI.RenameChest`/`RenameChestSubmit`/`RenameChestCancel`
// (`ChestUI.cs:827-869`) + `IngameFancyUI.OpenVirtualKeyboard(2)`
// (`IngameFancyUI.cs:86-129`). The port's keyboard is the front-end screen:
// the rename opens `FrontScreen::Keyboard` mid-game with the
// `KeyboardTail::ChestRename` session whose submit/cancel tails land here.
// Shape deviations (phase-5 discovery Gaps #1/#2/#6): the keyboard parks the
// world routes, so `Main.editChest` is exactly "the rename session is open"
// and the panel hides while it is; IME/`blockKey`/`clrInput` have no port
// route. The name store is the `WorldChest` entity itself — the M6-SAVE-1
// phase-3 fold retired the `App.chest_names` side map (the `.wld` round-trip
// needs the name on the chest, `SaveChests:1653`).

/// `Chest.MaxNameLength` (`Chest.cs:70`) — also the `UIVirtualKeyboard` ctor's
/// `maxLength` default the chest context opens with (`:159`).
pub(crate) const CHEST_MAX_NAME_LENGTH: usize = 20;

/// The stored `Chest.name` (`Chest.cs:82`) at the store index — the panel's
/// name line (`ChestUI.cs:115-122`), the rename prefill and the rename
/// submit's write target. A negative index (no container open) or a removed
/// slot reads as the empty name.
pub(crate) fn chest_name(app: &App, idx: i32) -> &str {
    stored_name(&app.world_chests, idx)
}

/// [`chest_name`]'s store-scoped form: the borrow stays field-scoped at the
/// frame's draw site (`app/frame.rs` builds the panel bundle while borrowing
/// disjoint `App` fields). A negative index (no container open) or a removed
/// slot reads as the empty name.
pub(crate) fn stored_name(chests: &[WorldChest], idx: i32) -> &str {
    if idx < 0 {
        return "";
    }
    chests
        .get(idx as usize)
        .filter(|c| !is_removed(c))
        .map_or("", |c| c.name.as_str())
}

/// `Chest.maxItems` (`Chest.cs:78`) of the entry at `idx` — the panel's and
/// the scroll's window size (`ChestUI.cs:376-377`). The default
/// [`DEFAULT_MAX_ITEMS`] for a closed container, an out-of-range index or a
/// removed slot (the C# `Main.chest[idx].maxItems` never runs on a null).
pub(crate) fn chest_max_items(chests: &[WorldChest], idx: i32) -> u32 {
    if idx < 0 {
        return DEFAULT_MAX_ITEMS;
    }
    chests
        .get(idx as usize)
        .filter(|c| !is_removed(c))
        .map_or(DEFAULT_MAX_ITEMS, |c| c.max_items as u32)
}

/// `ChestUI.RenameChest` (`ChestUI.cs:827-838`): the rename button's dispatch
/// — open the keyboard while no editor is live, submit the live text
/// otherwise.
pub(crate) fn rename_button_click(app: &mut App) {
    if !app.chest_editing() {
        open_rename(app);
    } else {
        let text = app.chest_edit_text().to_string();
        rename_submit(app, &text);
    }
}

/// `IngameFancyUI.OpenVirtualKeyboard(2)` (`IngameFancyUI.cs:86-129`): sound
/// 12, the prefill (the stored name, else the tile default at the open
/// chest's origin) and the ctx-2 keyboard (`UI.EnterNewName`, the ctor
/// defaults `allowEmpty: false, maxLength: 20` — 20 is also
/// [`CHEST_MAX_NAME_LENGTH`], `Chest.cs:70`). `chest < 0` no-ops (`false`):
/// the C# would index `Main.chest[-1]`, unreachable in vanilla because the
/// rename button only draws with a chest open — the port answers calmly
/// (DW-5.4). The unified `Player.chest` makes the guard exclude the banks
/// too — correct per `ChestUI.cs:198`'s bank hide rule + `:844-848`
/// (M3-CHEST-INT). `CanDisplay(2)` is unconditionally true
/// (`UIVirtualKeyboard.cs:1022-1029`); `Main.clrInput` (`:128`) has no port
/// route (the front screen parks the world input routes).
pub(crate) fn open_rename(app: &mut App) -> bool {
    let (chest, chest_x, chest_y) = match app.player.as_ref() {
        Some(p) if p.chest >= 0 => (p.chest, p.chest_x, p.chest_y),
        _ => return false, // the DW-5.4 guard (`ChestUI.cs:844-848`'s calm arm)
    };
    app.audio.play("Menu_Tick"); // `:90` — `SoundEngine.PlaySound(12)`
    // `:103-119` — the tile-default decode at the open chest's origin (the
    // phase-3 `default_chest_name` chain).
    let default_name = app
        .world
        .tile(chest_x, chest_y)
        .map(|t| default_chest_name(t.kind, t.frame_x))
        .unwrap_or_default();
    // `:102-123` — `npcChatText = chest.name`; an empty name falls to the
    // default, which becomes the keyboard's starting text.
    let initial = {
        let stored = chest_name(app, chest);
        if stored.is_empty() {
            default_name.clone()
        } else {
            stored.to_string()
        }
    };
    app.chest_default_name = default_name;
    // `:124` — `Main.editChest = true`; `:133-156` — the ctx-2 keyboard onto
    // `Main.InGameUI` with the submit/cancel closures
    // (`ChestUI.RenameChestSubmit`/`RenameChestCancel` + `Close`).
    app.open_keyboard_with_tail(
        Request {
            label: keyboard::ENTER_NEW_NAME.into(),
            initial,
            allow_empty: false,
            max_length: CHEST_MAX_NAME_LENGTH,
            context: Context::Chest,
        },
        None,
        KeyboardTail::ChestRename,
    );
    true
}

/// `ChestUI.RenameChestSubmit` (`ChestUI.cs:840-861`) + the submit closure's
/// `IngameFancyUI.Close(quiet: true)` (`IngameFancyUI.cs:148-156`): sound 12,
/// the editor closes, then the chest<0 no-op (:844-848), the
/// default-name-to-empty rule (:849-852) and the write-if-different
/// (:853-860 — the `editedChestName` net flag is MP, out). `text` is the live
/// keyboard text — `Main.npcChatText`'s per-tick sync
/// (`UIVirtualKeyboard.cs:1061-1067`); the C# submit closure ignores its
/// trimmed argument (`:363`), so the untrimmed text is what lands.
pub(crate) fn rename_submit(app: &mut App, text: &str) -> u32 {
    app.audio.play("Menu_Tick"); // `:842`
    app.front = None; // `Main.editChest = false` (:843) + `Close(quiet: true)`
    let chest = match app.player.as_ref() {
        Some(p) if p.chest >= 0 => p.chest,
        _ => return 12, // `:844-848`
    };
    let mut name = text.to_string();
    if name == app.chest_default_name {
        name = String::new(); // `:849-852`
    }
    // `:853-860` — the write-if-different guard, onto the entity's own field
    // (the retired `App.chest_names` side map's target).
    if let Some(entry) = app
        .world_chests
        .get_mut(chest as usize)
        .filter(|c| !is_removed(c))
    {
        if entry.name != name {
            entry.name = name;
        }
    }
    12
}

/// `ChestUI.RenameChestCancel` (`ChestUI.cs:863-869`): sound 12, the editor
/// closes without writing (`editChest = false`, `npcChatText = ""`). The
/// port's modal shape closes the keyboard screen with the edit — the C#
/// panel-cancel (ID 6) leaves the keyboard open (recorded deviation).
/// `Main.blockKey` (`:868`) has no port route.
pub(crate) fn rename_cancel(app: &mut App) -> u32 {
    app.audio.play("Menu_Tick"); // `:865`
    app.front = None;
    12
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemSlot;
    use crate::player::Player;

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
        chests[idx].slots = vec![slot(9, 6)];
        assert!(!chests[idx].is_empty());
        // A drained slot is air in place (`Item.IsAir`: type 0 and stack 0),
        // not a removed entry.
        chests[idx].slots = vec![EMPTY_SLOT, slot(619, 4)];
        assert!(!chests[idx].is_empty());
        chests[idx].slots = vec![EMPTY_SLOT, EMPTY_SLOT];
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
        chests[idx].slots = vec![slot(9, 6), EMPTY_SLOT, slot(619, 4)];
        assert_eq!(
            chest_slots(&chests, idx),
            Some(&[slot(9, 6), EMPTY_SLOT, slot(619, 4)][..])
        );
        assert_eq!(chest_slots(&chests, 5), None);
        remove_chest(&mut chests, idx);
        assert_eq!(chest_slots(&chests, idx), None);
        let other = create_chest(&mut chests, 40, 40);
        // The reuse claimed the tombstone's slot; a fresh chest is empty
        // until a slot is stocked (phase 4's placement grows the list).
        chests[other].slots = vec![slot(9, 6)];
        if let Some(slots) = chest_slots_mut(&mut chests, other) {
            slots[0] = slot(48, 1);
        }
        assert_eq!(chests[other].slots[0], slot(48, 1));
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

    #[test]
    fn default_chest_name_matches_the_chestui_chain() {
        // `ChestUI.cs:124-140` — the tile-style default the panel shows while
        // the stored name is empty. The strings are the port's generated item
        // names (the vanilla en-US localization's own values).
        assert_eq!(default_chest_name(21, 0), "Chest"); // style 0
        assert_eq!(default_chest_name(21, 36), "Gold Chest"); // style 1
        // The locked twins: `Chest.Lock` shifts +36 (`Chest.cs:449-453`).
        assert_eq!(default_chest_name(21, 72), "Locked Gold Chest");
        assert_eq!(default_chest_name(21, 144), "Locked Shadow Chest");
        // The biome chests shift +180 → styles 23-27 (`Chest.cs:454-458`).
        assert_eq!(default_chest_name(21, 828), "Locked Jungle Chest");
        // `Lang.cs:503-506`: styles 52+ alias style 0.
        assert_eq!(default_chest_name(21, 1908), "Chest");
        // Tile 467: style 4 is the Dead Man's Chest (`ChestUI.cs:129-131`).
        assert_eq!(default_chest_name(467, 144), "Dead Mans Chest");
        assert_eq!(default_chest_name(467, 180), "Solar Chest");
        // Tile 88 dressers read frameX / 54 (`:137-139`).
        assert_eq!(default_chest_name(88, 0), "Dresser");
        assert_eq!(default_chest_name(88, 54), "Ebonwood Dresser");
        // A style with no placing item and no twin rule renders nothing.
        assert_eq!(default_chest_name(467, 0), "");
    }

    // --- Phase 2: the open/close + locking interaction -----------------------

    /// A phase-1 tile probe with the frame's row set — the 2×2 placement
    /// needs both axes (the origin decode reads `frameY / 18`).
    fn set_chest_tile(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].frame_x = frame_x;
        w.tiles[i].frame_y = frame_y;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    /// A full 2×2 chest: the left column carries `origin_frame_x`, the right
    /// column `+18` (the 18 px sheet pitch); the bottom row is `frameY` 18.
    fn place_chest(w: &mut World, x: i32, y: i32, kind: u16, origin_frame_x: i16) {
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            set_chest_tile(
                w,
                x + dx,
                y + dy,
                kind,
                origin_frame_x + if dx == 1 { 18 } else { 0 },
                if dy == 1 { 18 } else { 0 },
            );
        }
    }

    /// A headless `App` over `w` with a spawned player (the `mining_app`
    /// shape, editing.rs:2559-2562).
    fn interact_app(w: World) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        app
    }

    /// The tile center's world position (the editing.rs cursor px → tile
    /// round-trip: `(px / TILE_PX).floor()`).
    fn click(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0)
    }

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    /// A prefixed fixture slot (the M3-PREFIX rolled-prefix shape).
    fn prefixed_slot(kind: u16, count: u32, prefix: u16) -> ItemSlot {
        ItemSlot {
            name: registry::name(kind),
            prefix,
            ..slot(kind, count)
        }
    }

    /// A headless app whose open chest (index 0, tile origin (10, 18)) holds
    /// `slots` — the shift-transfer helpers' fixture (the `chest_app` shape
    /// from clicks.rs's tests, minus the screen geometry they need).
    fn transfer_app(slots: Vec<ItemSlot>) -> App {
        let mut app = interact_app(world());
        app.world_chests.push(WorldChest {
            tile: (10, 18),
            slots,
            name: String::new(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        });
        let p = app.player.as_mut().unwrap();
        p.chest = 0;
        p.chest_x = 10;
        p.chest_y = 18;
        app
    }

    /// DW-2.1 — the open/close cycle (`Player.cs:33056-33260` +
    /// `OpenChest :33505-33526`): the click sets `chest/chest_x/chest_y` and
    /// opens the inventory, the same chest's click closes (`chest = -1` —
    /// the C# keeps `chestX/Y`, `:33236`), and the toggle's sound ids are
    /// the C# 10 (fresh open `:33242`) / 11 (close `:33237`) / 12 (re-open
    /// with another chest open `:33251`). The store entry is created lazily
    /// on the first open (the phase-1 decision).
    #[test]
    fn dw_2_1_open_close_cycle_sets_chest_state_and_sounds() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        place_chest(&mut w, 30, 40, 21, 0);
        let mut app = interact_app(w);
        app.crafting_open = true;

        // The open click through the public path: state + the lazy entry.
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
        assert_eq!(
            (
                app.player.as_ref().unwrap().chest_x,
                app.player.as_ref().unwrap().chest_y
            ),
            (10, 18)
        );
        assert!(app.inventory_open, "`Main.playerInventory = true` (:33517)");
        assert!(!app.crafting_open, "`NewCraftingUI.Close` (:33516)");
        assert_eq!(
            find_chest(&app.world_chests, 10, 18),
            Some(0),
            "the store entry is created on the first open"
        );
        assert!(app.world_chests[0].is_empty(), "a fresh entry is empty");

        // The toggle's sound ids, driven through the same helper the click
        // path uses (`:33230-33252`).
        assert_eq!(toggle_open(&mut app, 10, 18, 0), 11, "same chest closes");
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the close resets `chest`"
        );
        assert_eq!(
            (
                app.player.as_ref().unwrap().chest_x,
                app.player.as_ref().unwrap().chest_y
            ),
            (10, 18),
            "the C# keeps chestX/Y on close (:33236)"
        );
        // A fresh open with nothing open plays 10 (`:33239-33242`).
        let b = create_chest(&mut app.world_chests, 30, 40);
        assert_eq!(toggle_open(&mut app, 30, 40, b as i32), 10);
        assert_eq!(app.player.as_ref().unwrap().chest, b as i32);
        // Opening another chest while one is open plays 12 (`:33248-33251`).
        assert_eq!(toggle_open(&mut app, 10, 18, 0), 12);
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
    }

    /// DW-2.2 — the per-tick close (`HandleBeingInChestRange`,
    /// `Player.cs:31015-31097`): out of the `Simple` reach of the chest's
    /// hitbox, or a dead tile at the origin, force-closes with sound 11 —
    /// the close is the tick's job, not an open-time guard.
    #[test]
    fn dw_2_2_tick_close_forces_close_out_of_range_or_dead_tile() {
        let mut w = world();
        place_chest(&mut w, 10, 10, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);

        // In range (the player's center clamps into the chest's deflated
        // rect inside the ±5/±3 bounds): stays open.
        {
            let p = app.player.as_mut().unwrap();
            p.x = 150.0;
            p.y = 150.0;
        }
        tick_open(&mut app);
        assert_eq!(app.player.as_ref().unwrap().chest, 0, "in range stays open");

        // Out of range: the force close (`:31078-31085`, sound 11).
        {
            let p = app.player.as_mut().unwrap();
            p.x = 480.0;
            p.y = 150.0;
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "out of range force-closes"
        );
        assert_eq!(force_close(&mut app), 11, "the close sound is 11");

        // Re-open, then kill the tile: the dead-tile arm (:31086-31090).
        assert!(try_open_chest_at_cursor(&mut app, click(10, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
        {
            let p = app.player.as_mut().unwrap();
            p.x = 150.0;
            p.y = 150.0;
        }
        let i = app.world.index(10, 10);
        app.world.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "a dead tile force-closes"
        );

        // Nothing open: the tick is a no-op (`:31017`).
        tick_open(&mut app);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
    }

    /// DW-1.2 — the per-tile hitbox table (`Player.cs:31101-31118`): every
    /// container family's rect at the origin, in world px, before the
    /// `Inflate(-1, -1)` deflation (`:31119` — X += 1, Y += 1, W −= 2, H −= 2
    /// inside [`in_interaction_range_to_multi_tile_hitbox`]; the 1 px shrink
    /// is below the tile granularity the tick's clamp probes can observe, so
    /// the rect values are pinned here directly). The bank rows are the
    /// M3-CHEST-INT addition — the banks share the world chest's per-tick
    /// close now.
    #[test]
    fn test_DW_1_2_hitbox_rect_table_matches_the_csharp() {
        // 463/491 (`:31103-31105`) — 3×4 at −1/−2 tiles.
        assert_eq!(multi_tile_hitbox_px_rect(463, 10, 20), (144, 288, 48, 64));
        assert_eq!(multi_tile_hitbox_px_rect(491, 10, 20), (144, 288, 48, 64));
        // The `BasicChest` set {21, 467} (`TileID.cs:570`) + the 97 safe
        // (`:31107-31109`) — 2×2.
        assert_eq!(multi_tile_hitbox_px_rect(21, 10, 10), (160, 160, 32, 32));
        assert_eq!(multi_tile_hitbox_px_rect(467, 10, 10), (160, 160, 32, 32));
        assert_eq!(multi_tile_hitbox_px_rect(97, 10, 10), (160, 160, 32, 32));
        // The 88 dresser (`:31110-31112`) — 3×2.
        assert_eq!(multi_tile_hitbox_px_rect(88, 10, 10), (160, 160, 48, 32));
        // The 29 piggy (`:31115-31117`) — 2×1.
        assert_eq!(multi_tile_hitbox_px_rect(29, 10, 19), (160, 304, 32, 16));
        // An unmatched kind answers `Rectangle.Empty` (`:31100`).
        assert_eq!(multi_tile_hitbox_px_rect(0, 10, 10), (0, 0, 0, 0));
    }

    /// DW-1.2 — the bank tile (29) rides the per-tick close
    /// (`HandleBeingInChestRange`, `Player.cs:31015-31097`): the piggy's 2×1
    /// rect (`:31115-31117`, inflated to 30×14) keeps the open bank in reach
    /// below it and force-closes out of reach above — the bank auto-close
    /// the port never had (M3-CHEST-INT). The close's sound id is 11 (the
    /// `force_close` return, the dw_2_2 pin).
    #[test]
    fn test_DW_1_2_tick_open_range_closes_bank_out_of_range() {
        let mut w = world();
        // A piggy bank 2×1 at (10, 19) — frame columns 0/18.
        set_tile(&mut w, 10, 19, 29, 0);
        set_tile(&mut w, 11, 19, 29, 18);
        let mut app = interact_app(w);
        // The bank opens through the shared toggle (the editing.rs arm's
        // callee) — `chest`/`chestX`/`chestY` land on the player.
        assert_eq!(toggle_open(&mut app, 10, 19, -2), 10);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2);
        assert_eq!((p.chest_x, p.chest_y), (10, 19));

        // In range: just below the piggy, the center clamps into the deflated
        // rect (Bottom 319 px → tile 19) inside the ±5/±3 bounds — stays open.
        {
            let p = app.player.as_mut().unwrap();
            p.x = 166.0; // center x 176 — inside [161, 191]
            p.y = 320.0; // center y 341 — clamps to the rect's bottom
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -2,
            "in range stays open"
        );

        // Out of range: far above, the clamp still lands on tile 19 but the
        // reach's hy is 18 — the close (`:31078-31085`, sound 11).
        {
            let p = app.player.as_mut().unwrap();
            p.x = 166.0;
            p.y = 200.0;
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "out of range closes"
        );
        assert_eq!(force_close(&mut app), 11, "the close sound is 11");
    }

    /// DW-1.2 — the bank's dead-tile close (`:31086-31090`): a deactivated
    /// origin tile force-closes the open bank, same as the world chests.
    #[test]
    fn test_DW_1_2_tick_open_bank_dead_tile_closes() {
        let mut w = world();
        set_tile(&mut w, 10, 19, 29, 0);
        set_tile(&mut w, 11, 19, 29, 18);
        let mut app = interact_app(w);
        assert_eq!(toggle_open(&mut app, 10, 19, -2), 10);
        {
            let p = app.player.as_mut().unwrap();
            p.x = 166.0;
            p.y = 320.0;
        }
        let i = app.world.index(10, 19);
        app.world.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "a dead tile force-closes"
        );
    }

    /// DW-1.2 — the inflate(−1)/closest-point boundary (`:31119-31123`): the
    /// reach test runs on the CLAMPED point (`Utils.ClosestPointInRect`,
    /// `Utils.cs:1581-1595`) and the `Simple` bounds are inclusive
    /// (`Collision.InTileBounds`, `Collision.cs:3776`) over the `ceil`-derived
    /// edges (`TileReachCheckSettings.cs:51-53`) — one px across the edge
    /// flips the answer. The inflate's 1 px itself sits inside the pinned
    /// rect values (the table test above): the tick's tile granularity
    /// cannot split it.
    #[test]
    fn test_DW_1_2_inflate_closest_point_boundary_at_the_reach_edge() {
        let mut w = world();
        place_chest(&mut w, 10, 10, 21, 0);
        let mut app = interact_app(w);
        assert_eq!(toggle_open(&mut app, 10, 10, 0), 10);

        // The X edge, far left: the center clamps to the rect's Left (161 px
        // → tile 10); hx = ceil((x + 20)/16) − 1 + 5 decides. px 61 →
        // ceil(81/16) = 6 → hx 10 ≥ 10: in bounds, stays open.
        {
            let p = app.player.as_mut().unwrap();
            p.x = 61.0;
            p.y = 150.0; // center y 171 — inside the rect's [161, 191]
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            0,
            "the boundary px stays in reach"
        );

        // px 60 → ceil(80/16) = 5 → hx 9 < 10: the same clamp point is out of
        // the reach — the close.
        {
            let p = app.player.as_mut().unwrap();
            p.x = 60.0;
            p.y = 150.0;
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "one px past the reach closes"
        );

        // Re-open for the Y edge, above the chest: the clamp lands on tile 10
        // (Top 161), hy = ceil((y + 42)/16) − 1 + 3 — y 71 keeps it (hy 10),
        // y 70 closes (hy 9). The re-open is fresh (the chest was closed) —
        // sound 10.
        assert_eq!(toggle_open(&mut app, 10, 10, 0), 10);
        {
            let p = app.player.as_mut().unwrap();
            p.x = 166.0; // center x 176 — no X clamp
            p.y = 71.0;
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            0,
            "the boundary py stays in reach"
        );
        {
            let p = app.player.as_mut().unwrap();
            p.y = 70.0;
        }
        tick_open(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "one px past the vertical reach closes"
        );
    }

    /// DW-1.3 — the shared block's sound ids for the bank values
    /// (`Player.cs:33230-33252`): a fresh bank open plays 10 (`:33241-33243`),
    /// the same bank's re-click closes with 11 (`:33235-33237`), opening
    /// another bank while one is open plays 12 (`:33248-33252`). The ids are
    /// `toggle_open`'s returned legacy ids — the callers play them through
    /// `play_menu_sound`'s 10/11/12 → Menu_Open/Menu_Close/Menu_Tick mapping
    /// (the world-chest values ride the same contract, the dw_2_1 pin).
    #[test]
    fn test_DW_1_3_toggle_open_returns_the_shared_block_sound_ids() {
        let mut app = interact_app(world());
        assert_eq!(toggle_open(&mut app, 10, 19, -2), 10, "fresh bank open");
        assert_eq!(app.player.as_ref().unwrap().chest, -2);
        assert_eq!(toggle_open(&mut app, 10, 19, -2), 11, "re-click closes");
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        assert_eq!(toggle_open(&mut app, 20, 19, -3), 10, "fresh again");
        assert_eq!(toggle_open(&mut app, 10, 19, -2), 12, "the switch");
        assert_eq!(app.player.as_ref().unwrap().chest, -2);
    }

    /// DW-1.4 — the one open-container field carries the full C# index space
    /// (`Player.chest`, `Player.cs:2876`): −1 closed, −2..−5 the banks (the
    /// `bank_for_chest` resolver order), ≥ 0 the `App.world_chests` store
    /// index. Every step replaces the previous value — there is no second
    /// field (`App.chest` is gone; the compile is the witness, the suite the
    /// reader sweep).
    #[test]
    fn test_DW_1_4_unified_field_carries_the_full_index_space() {
        let mut app = interact_app(world());
        let chest_of = |app: &App| app.player.as_ref().unwrap().chest;
        assert_eq!(chest_of(&app), -1, "closed");
        // The world chests (>= 0).
        assert_eq!(toggle_open(&mut app, 10, 10, 0), 10);
        assert_eq!(chest_of(&app), 0);
        // The banks in the resolver order (−2..−5 → banks[0..4]).
        for (i, idx) in [-2, -3, -4, -5].iter().enumerate() {
            assert_eq!(toggle_open(&mut app, 10, 10, *idx), 12, "switch to {idx}");
            assert_eq!(chest_of(&app), *idx);
            assert_eq!(crate::bank::bank_for_chest(*idx), Some(i));
        }
        // Back to a world chest, then closed.
        assert_eq!(toggle_open(&mut app, 10, 10, 0), 12);
        assert_eq!(chest_of(&app), 0);
        assert_eq!(toggle_open(&mut app, 10, 10, 0), 11);
        assert_eq!(chest_of(&app), -1);
    }

    /// DW-2.3 — the locked chest's click chain (`Player.cs:33142-33228`):
    /// the locked click consumes without opening (`flag14` blocks the
    /// `FindChest`), the right key unlocks and is consumed, and the chest
    /// opens on the next click.
    #[test]
    fn dw_2_3_locked_chest_click_consumes_key_unlocks_then_opens() {
        let mut w = world();
        place_chest(&mut w, 20, 10, 21, 72); // style 2 — the golden key 327
        let mut app = interact_app(w);

        // Locked, no key: the click is consumed, nothing opens, no store
        // entry is created.
        assert!(try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        assert_eq!(find_chest(&app.world_chests, 20, 10), None);
        assert!(is_locked(&app.world, 20, 10), "still locked");

        // The right key: the unlock fires, the click is still consumed (no
        // open this click), and the key's stack decrements.
        app.inventory[0] = slot(327, 2);
        assert!(try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.world.tile(20, 10).unwrap().frame_x, 36, "72 - 36");
        assert_eq!(app.world.tile(21, 10).unwrap().frame_x, 54, "90 - 36");
        assert_eq!(app.inventory[0].count, 1, "the key is consumed");
        assert!(!is_locked(&app.world, 20, 10));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the unlock consumes the click"
        );
        assert_eq!(find_chest(&app.world_chests, 20, 10), None);

        // The next click opens (`:33223-33226` runs `FindChest` now).
        assert!(try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
        assert_eq!(
            (
                app.player.as_ref().unwrap().chest_x,
                app.player.as_ref().unwrap().chest_y
            ),
            (20, 10)
        );
    }

    /// DW-2.3 — the shadow key (329) unlocks without consuming
    /// (`flag16 = num77 != 329`, `Player.cs:33175`).
    #[test]
    fn dw_2_3_shadow_key_unlock_does_not_consume() {
        let mut w = world();
        place_chest(&mut w, 40, 10, 21, 144); // style 4 — the shadow key 329
        let mut app = interact_app(w);
        app.inventory[0] = slot(329, 5);
        assert!(try_open_chest_at_cursor(&mut app, click(40, 10)));
        assert_eq!(app.world.tile(40, 10).unwrap().frame_x, 108, "144 - 36");
        assert!(!is_locked(&app.world, 40, 10));
        assert_eq!(app.inventory[0].count, 5, "the shadow key is not consumed");
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the click is still consumed"
        );
        // And the unlocked chest opens on the next click.
        assert!(try_open_chest_at_cursor(&mut app, click(40, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
    }

    /// DW-2.3 — the key scan covers the C# `inventory[0..=58]` array only
    /// (`Player.cs:33176`): the port's coins (50-53) are searched, the
    /// equipment rows are not, and exactly one key is consumed per click
    /// (`Chest.Unlock` answers false once the chest is unlocked).
    #[test]
    fn dw_2_3_key_search_covers_the_59_array_slots_only() {
        let mut w = world();
        place_chest(&mut w, 60, 10, 21, 72);
        let mut app = interact_app(w);
        // An equipment row (outside 0..58): not searched — the chest stays
        // locked and the key is untouched.
        app.equip.armor[0] = slot(327, 1);
        assert!(try_open_chest_at_cursor(&mut app, click(60, 10)));
        assert!(
            is_locked(&app.world, 60, 10),
            "the armor row is not scanned"
        );
        assert_eq!(app.equip.armor[0].count, 1);

        // A coin slot (C# array index 50-53): searched.
        app.equip.coins[0] = slot(327, 3);
        assert!(try_open_chest_at_cursor(&mut app, click(60, 10)));
        assert!(!is_locked(&app.world, 60, 10));
        assert_eq!(app.equip.coins[0].count, 2, "one key consumed");
    }

    /// DW-2.3 — exactly one key per click: the loop keeps scanning after a
    /// success, but the chest is unlocked, so the second matching slot
    /// survives (`Player.cs:33178`'s `Chest.Unlock` continue arm).
    #[test]
    fn dw_2_3_exactly_one_key_is_consumed_per_click() {
        let mut w = world();
        place_chest(&mut w, 20, 40, 21, 72);
        let mut app = interact_app(w);
        app.inventory[0] = slot(327, 1);
        app.inventory[1] = slot(327, 1);
        assert!(try_open_chest_at_cursor(&mut app, click(20, 40)));
        assert_eq!(app.inventory[0].count, 0, "the first key is consumed");
        assert_eq!(app.inventory[0].kind, 0, "the emptied slot is air");
        assert_eq!(app.inventory[1].count, 1, "the second is not");
    }

    /// DW-2.3 — the trash slot's mirror (`Player.cs:33190-33193`): a key
    /// consumed from slot 58 clones the post-decrement slot into
    /// `Main.mouseItem`.
    #[test]
    fn dw_2_3_trash_slot_key_mirrors_into_the_cursor_stack() {
        let mut w = world();
        place_chest(&mut w, 60, 40, 21, 72);
        let mut app = interact_app(w);
        app.equip.trash = slot(327, 2);
        assert!(try_open_chest_at_cursor(&mut app, click(60, 40)));
        assert_eq!(app.equip.trash.count, 1);
        assert_eq!(app.mouse_item.kind, 327, "`Main.mouseItem` mirrors");
        assert_eq!(app.mouse_item.count, 1);
    }

    /// DW-2.4 — the key-id derivation at the frame boundaries
    /// (`Player.cs:33145-33172`): 327 default, 329 in 144-178, 1533-1537
    /// in 828-1006 (style = the halved `frameX / 18`), 4714 for tile 467
    /// style 13.
    #[test]
    fn dw_2_4_key_id_derivation_at_frame_boundaries() {
        let mut w = world();
        let cases: [(i32, u16, i16, u16); 13] = [
            (4, 21, 143, 327),
            (6, 21, 144, 329),
            (8, 21, 177, 329),
            (10, 21, 178, 329),
            (12, 21, 179, 327),
            (14, 21, 827, 327),
            (16, 21, 828, 1533),
            (18, 21, 1006, 1537),
            (20, 21, 1007, 327),
            (22, 467, 467, 327),  // style 12
            (24, 467, 468, 4714), // style 13
            (26, 467, 503, 4714), // style 13's last frame
            (28, 467, 504, 327),  // style 14
        ];
        for &(x, kind, frame_x, key) in &cases {
            set_tile(&mut w, x, 90, kind, frame_x);
            assert_eq!(
                locked_key_id(&w, x, 90),
                key,
                "tile {kind} frameX {frame_x} derives key {key}"
            );
        }
    }

    /// DW-2.5 — `Chest.Unlock`/`Chest.Lock`'s frame tables
    /// (`Chest.cs:346-496`): −36/+36 for the plain styles, ∓180 for the
    /// Plantera-gated ones, the 2×2 shifts as a whole, the unsupported
    /// styles refuse, and the gate reads the flag parameter (the world flag
    /// has no home yet — the recorded fallback).
    #[test]
    fn dw_2_5_lock_unlock_shift_frames_per_style_with_the_plant_boss_gate() {
        let mut w = world();
        // Unlock: (style, origin frameX, shift, gated). The plain rows run
        // ungated; the gated rows are probed both ways below.
        let unlock_cases = [
            (10, 2i16, 72, 36),
            (20, 4, 144, 36),
            (30, 23, 828, 180),
            (40, 36, 1296, 36),
            (50, 38, 1368, 36),
            (60, 40, 1440, 36),
        ];
        for (x, style, frame_x, shift) in unlock_cases {
            place_chest(&mut w, x, 10, 21, frame_x);
            if style == 23 {
                // The Plantera gate (`Chest.cs:376-379`): refused, frames
                // unchanged.
                assert!(!unlock_chest(&mut w, x, 10, false), "style 23 gates");
                assert_eq!(w.tile(x, 10).unwrap().frame_x, frame_x);
            }
            assert!(unlock_chest(&mut w, x, 10, true), "style {style} unlocks");
            assert_eq!(w.tile(x, 10).unwrap().frame_x, frame_x - shift);
            assert_eq!(
                w.tile(x + 1, 10).unwrap().frame_x,
                frame_x + 18 - shift,
                "the right column shifts too"
            );
            assert!(!is_locked(&w, x, 10));
        }
        // An unsupported style refuses (the `default` arm, `Chest.cs:390-391`).
        place_chest(&mut w, 70, 10, 21, 0);
        assert!(!unlock_chest(&mut w, 70, 10, true));

        // Lock: the mirror table (`Chest.cs:440-467`).
        let lock_cases = [
            (10, 1i16, 36, 36),
            (20, 3, 108, 36),
            (30, 18, 648, 180),
            (40, 35, 1260, 36),
            (50, 37, 1332, 36),
            (60, 39, 1404, 36),
        ];
        for (x, style, frame_x, shift) in lock_cases {
            place_chest(&mut w, x, 40, 21, frame_x);
            if style == 18 {
                assert!(!lock_chest(&mut w, x, 40, false), "style 18 gates");
                assert_eq!(w.tile(x, 40).unwrap().frame_x, frame_x);
            }
            assert!(lock_chest(&mut w, x, 40, true), "style {style} locks");
            assert_eq!(w.tile(x, 40).unwrap().frame_x, frame_x + shift);
            assert!(is_locked(&w, x, 40), "the locked frame decodes locked");
        }
        place_chest(&mut w, 70, 40, 21, 72); // already locked — unsupported
        assert!(!lock_chest(&mut w, 70, 40, true));

        // Tile 467: unlock style 13 → 12, lock style 12 → 13, both gated
        // (`Chest.cs:394-406`/:469-480).
        place_chest(&mut w, 10, 70, 467, 468);
        assert!(!unlock_chest(&mut w, 10, 70, false), "467-13 gates");
        assert!(unlock_chest(&mut w, 10, 70, true));
        assert_eq!(w.tile(10, 70).unwrap().frame_x, 432, "468 - 36");
        assert!(lock_chest(&mut w, 10, 70, true));
        assert_eq!(w.tile(10, 70).unwrap().frame_x, 468, "432 + 36");

        // The round trip restores the frame.
        place_chest(&mut w, 40, 70, 21, 36);
        assert!(lock_chest(&mut w, 40, 70, true));
        assert_eq!(w.tile(40, 70).unwrap().frame_x, 72);
        assert!(unlock_chest(&mut w, 40, 70, true));
        assert_eq!(w.tile(40, 70).unwrap().frame_x, 36);
    }

    /// The origin decode (`Player.cs:33062-33066`): a click on any of the
    /// four cells resolves the same 2×2 origin — here the bottom-right.
    #[test]
    fn a_chest_click_on_any_of_the_four_cells_opens_the_same_chest() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(11, 19)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
        assert_eq!(
            (
                app.player.as_ref().unwrap().chest_x,
                app.player.as_ref().unwrap().chest_y
            ),
            (10, 18),
            "the bottom-right cell"
        );
        // The same chest's origin-cell click closes it (the same index).
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
    }

    /// `TileID.Sets.BasicChest` (`TileID.cs:570`) gates the open path: tile
    /// 341 (the craft scan's frozen second entry) is not a `BasicChest`, and
    /// an inactive chest tile never reaches the interaction chain
    /// (`:32035-32041`).
    #[test]
    fn non_basic_chest_and_inactive_tiles_are_not_openable() {
        let mut w = world();
        set_tile(&mut w, 10, 10, 341, 0);
        set_tile(&mut w, 20, 10, 21, 0);
        let mut app = interact_app(w);
        assert!(!try_open_chest_at_cursor(&mut app, click(10, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        let i = app.world.index(20, 10);
        app.world.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        assert!(!try_open_chest_at_cursor(&mut app, click(20, 10)));
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
    }

    /// `Chest.Unlock`/`Chest.Lock` refuse a 2×2 with a missing tile
    /// (`Chest.cs:348-351`/:430-433) — the C# null-tile check maps to the
    /// port's bounds check (`world.tile` is `None` out of bounds, the
    /// phase-1 mapping).
    #[test]
    fn unlock_and_lock_refuse_a_missing_2x2_tile() {
        let mut w = world();
        // The right column of this 2×2 hangs off the world edge.
        set_chest_tile(&mut w, 119, 10, 21, 72, 0);
        set_chest_tile(&mut w, 119, 11, 21, 72, 18);
        assert!(!unlock_chest(&mut w, 119, 10, true));
        assert!(!lock_chest(&mut w, 119, 10, true));
        assert_eq!(w.tile(119, 10).unwrap().frame_x, 72, "unchanged");
    }

    // --- Phase 4: the slot-interaction helpers -------------------------------

    /// DW-4.4 — `ItemSlot.canFavoriteAt` (`ItemSlot.cs:386-394`): only the
    /// inventory contexts 0/1/2/32 can favorite — a chest slot (3) can never
    /// carry the flag, so the cursor-3 favorite arm never fires on it.
    #[test]
    fn dw_4_4_can_favorite_at_excludes_chest_context() {
        assert!(can_favorite_at(0));
        assert!(can_favorite_at(1));
        assert!(can_favorite_at(2));
        assert!(can_favorite_at(32));
        assert!(!can_favorite_at(3), "chest slots cannot be favorited");
        assert!(!can_favorite_at(5));
        assert!(!can_favorite_at(13));
    }

    /// DW-4.4 — `ItemSlot.retainFavoriteOnMoveAt` (`ItemSlot.cs:390-394`):
    /// moving out of a chest (3) clears the favorite (`MouseItemSwap`,
    /// `ItemSlot.cs:1255-1263`). The port's `ItemSlot` carries no favorite
    /// field yet (item.rs:52-57), so the clear itself is a no-op with the
    /// citation — the tables are the ported rule, M7-UI-2 consumes them.
    #[test]
    fn dw_4_4_retain_favorite_on_move_excludes_chest_context() {
        for ctx in [0, 1, 2, 32, 5] {
            assert!(retain_favorite_on_move_at(ctx), "ctx {ctx} retains");
        }
        assert!(
            !retain_favorite_on_move_at(3),
            "a chest move clears the favorite"
        );
        assert!(!retain_favorite_on_move_at(4));
    }

    /// DW-2.2 — the implicit-air slot helpers (decision #4, ported from the
    /// tuple versions onto `ItemSlot`): reads past the stored length answer
    /// air, writes extend with airs first and normalize half-air values to
    /// air, real items carry the registry name derived at write, and a
    /// removed/missing entry refuses.
    #[test]
    fn test_DW_2_2_implicit_air_normalisation_holds_on_item_slots() {
        let mut chests: Vec<WorldChest> = Vec::new();
        let idx = create_chest(&mut chests, 10, 18);
        assert_eq!(
            chest_slot(&chests, idx, 5),
            Some(EMPTY_SLOT),
            "beyond len is air"
        );
        assert!(set_chest_slot(&mut chests, idx, 35, slot(9, 3)));
        assert_eq!(chests[idx].slots.len(), 36, "extended with airs first");
        assert_eq!(
            chests[idx].slots[34], EMPTY_SLOT,
            "the extension filled air"
        );
        assert_eq!(chest_slot(&chests, idx, 35), Some(slot(9, 3)));
        // The registry name is derived at write — even a nameless write
        // stores the item's name (the plan's `name` from `registry::name`).
        assert!(set_chest_slot(
            &mut chests,
            idx,
            35,
            ItemSlot {
                kind: 9,
                name: "",
                count: 3,
                prefix: 0
            }
        ));
        assert_eq!(chests[idx].slots[35].name, "Wood");
        // Half-air values normalize to air (the `SetDefaults(0)` writes).
        assert!(set_chest_slot(
            &mut chests,
            idx,
            35,
            ItemSlot {
                count: 0,
                ..slot(9, 3)
            }
        ));
        assert_eq!(chest_slot(&chests, idx, 35), Some(EMPTY_SLOT));
        assert!(set_chest_slot(
            &mut chests,
            idx,
            36,
            ItemSlot {
                kind: 0,
                ..slot(9, 3)
            }
        ));
        assert_eq!(chest_slot(&chests, idx, 36), Some(EMPTY_SLOT));
        // The prefix survives a partial drain and dies with the air write —
        // a normalized half-air slot carries nothing.
        assert!(set_chest_slot(&mut chests, idx, 35, prefixed_slot(9, 3, 2)));
        assert_eq!(chest_slot(&chests, idx, 35), Some(prefixed_slot(9, 3, 2)));
        assert!(set_chest_slot(
            &mut chests,
            idx,
            35,
            ItemSlot {
                count: 0,
                ..prefixed_slot(9, 3, 2)
            }
        ));
        assert_eq!(chest_slot(&chests, idx, 35), Some(EMPTY_SLOT));
        assert!(chests[idx].is_empty(), "the writes left pure air");
        // A removed or out-of-range entry refuses both directions.
        remove_chest(&mut chests, idx);
        assert_eq!(chest_slot(&chests, idx, 0), None);
        assert!(!set_chest_slot(&mut chests, idx, 0, slot(9, 1)));
        assert_eq!(chest_slot(&chests, 9, 0), None);
    }

    /// DW-2.1 — the deposit-prefix-loss bug's regression test: depositing a
    /// prefixed item into a world chest and taking it back preserves the
    /// prefix end-to-end (the tuple store dropped it at every move — the
    /// fabricated views wrote `prefix: 0`; the C# moves whole `Item`s).
    #[test]
    fn test_DW_2_1_prefix_survives_a_world_chest_round_trip() {
        let mut app = transfer_app(vec![]);
        // The shift-in deposit (`TryPlacingInChest`): a prefixed stack parks
        // in the first empty chest slot with its prefix.
        app.inventory[3] = prefixed_slot(9, 40, 2);
        assert!(!try_placing_in_chest(&mut app, 3, false));
        assert_eq!(
            chest_slot(&app.world_chests, 0, 0),
            Some(prefixed_slot(9, 40, 2)),
            "the deposit kept the prefix"
        );
        assert_eq!(
            app.inventory[3].kind, 0,
            "the inventory slot drained to air"
        );
        // The shift-out take (`GetItem`'s `FillEmptyInventorySlot` —
        // `inventory[i] = returnItem`): the prefix rides back into the
        // inventory.
        quick_transfer_from_slot(&mut app, 0);
        assert!(
            app.inventory
                .iter()
                .any(|s| s.kind == 9 && s.prefix == 2 && s.count == 40),
            "the take-back carried the prefix into the inventory"
        );
        assert!(app.world_chests[0].is_empty(), "the chest drained");
    }

    /// DW-2.1 (dirty) — a partial merge keeps the prefix on the inventory
    /// remainder: the C# mutates `inv[slot]` in place (`Item item =
    /// inv[slot]` is a reference, `ChestUI.cs:1214`), so the un-moved tail
    /// returns with its prefix. The chest must be FULL for a remainder to
    /// survive — the empty-slot pass (`:1262-1287`) reads the implicit airs
    /// as deposit targets too (`item2[j].stack == 0`), so a roomy chest
    /// always absorbs it.
    #[test]
    fn test_DW_2_1_partial_merge_keeps_the_prefix_on_the_remainder() {
        // 39 full stacks + one partial: the merge soaks 9 into it and, with
        // no empty slot left, the inventory remainder keeps its prefix.
        let mut slots = vec![prefixed_slot(9, 9999, 2); 39];
        slots.push(prefixed_slot(9, 9990, 2));
        let mut app = transfer_app(slots);
        app.inventory[3] = prefixed_slot(9, 20, 2);
        assert!(!try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots.len(), 40, "nothing new deposited");
        assert_eq!(
            app.world_chests[0].slots[39].count, 9999,
            "the merge filled the partial"
        );
        assert_eq!(app.inventory[3].prefix, 2, "the remainder kept its prefix");
        assert_eq!(app.inventory[3].count, 11);
    }

    /// DW-2.1 (dirty, Test Plan) — prefixed + unprefixed same-kind stacks
    /// stay separate: `Item.CanStack` (`Item.cs:49865-49871`) requires the
    /// type AND the prefix, so neither transfer ever blends them.
    #[test]
    fn test_DW_2_1_prefixed_and_unprefixed_same_kind_stacks_stay_separate() {
        // The unprefixed chest stack does not absorb a prefixed deposit —
        // the deposit parks in the first empty slot instead.
        let mut app = transfer_app(vec![slot(9, 10)]);
        app.inventory[3] = prefixed_slot(9, 20, 2);
        assert!(!try_placing_in_chest(&mut app, 3, false));
        assert_eq!(
            app.world_chests[0].slots[0],
            slot(9, 10),
            "no cross-prefix merge"
        );
        assert_eq!(app.world_chests[0].slots[1], prefixed_slot(9, 20, 2));
        assert_eq!(app.inventory[3].kind, 0, "the deposit moved whole");
        // And the reverse: a prefixed chest stack does not absorb an
        // unprefixed deposit.
        let mut app = transfer_app(vec![prefixed_slot(9, 10, 2)]);
        app.inventory[3] = slot(9, 20);
        assert!(!try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], prefixed_slot(9, 10, 2));
        assert_eq!(app.world_chests[0].slots[1], slot(9, 20));
    }

    /// DW-2.1 (dirty) — `quick_transfer_from_slot`'s merge gate is CanStack
    /// too (`CanItemSlotAcceptPickup`, `Player.cs:39487`): a prefixed chest
    /// stack never soaks into an unprefixed partial inventory stack; a
    /// same-prefix partial does.
    #[test]
    fn test_DW_2_1_quick_transfer_refuses_a_cross_prefix_merge() {
        let mut app = transfer_app(vec![prefixed_slot(9, 100, 2)]);
        app.inventory[5] = slot(9, 10); // unprefixed partial — refuses
        quick_transfer_from_slot(&mut app, 0);
        assert_eq!(
            app.inventory[5].count, 10,
            "the unprefixed partial did not soak the prefixed stack"
        );
        assert!(
            app.inventory
                .iter()
                .any(|s| s.kind == 9 && s.prefix == 2 && s.count == 100),
            "the whole prefixed stack parked on an empty slot"
        );
        assert!(app.world_chests[0].is_empty());
        // Same prefix: the partial soaks the whole stack.
        let mut app = transfer_app(vec![prefixed_slot(9, 100, 2)]);
        app.inventory[5] = prefixed_slot(9, 10, 2);
        quick_transfer_from_slot(&mut app, 0);
        assert_eq!(
            app.inventory[5].count, 110,
            "the same-prefix partial soaked it"
        );
        assert!(app.world_chests[0].is_empty());
    }

    /// DW-2.3 — the plan contract: `bank::loot_all` takes a `WorldChest`
    /// directly (`fn(&mut world_chest.slots, ...)`, no bridge) — the drain,
    /// the coin-row routing and the overflow-stays rule all hold against the
    /// store's `ItemSlot` slots.
    #[test]
    fn test_DW_2_3_loot_all_runs_against_a_world_chest() {
        let mut chest = WorldChest {
            tile: (10, 18),
            slots: vec![
                slot(9, 500),            // the wood stack → the pickup
                slot(71, 50),            // the copper coins → the coin rows
                prefixed_slot(48, 7, 3), // the prefixed item → stays (no room)
            ],
            name: String::new(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        };
        let mut inventory: [ItemSlot; INV] = std::array::from_fn(|_| slot(3, 9999));
        inventory[40] = slot(9, 400); // the wood partial soaks the stack
        let mut coins = [slot(71, 10), EMPTY_SLOT, EMPTY_SLOT, EMPTY_SLOT];
        crate::bank::loot_all(&mut chest.slots, &mut inventory, &mut coins, None);
        // The wood drained into the partial (the 9999 cap takes all 500).
        assert!(inventory.iter().any(|s| s.kind == 9 && s.count == 900));
        assert_eq!(chest.slots[0], EMPTY_SLOT, "the wood drained");
        // The coins consolidated into the coin rows.
        assert_eq!(coins[0].count, 60);
        assert_eq!(chest.slots[1], EMPTY_SLOT, "the coins drained");
        // The prefixed item: no pickup slot accepts it — the overflow STAYS
        // in the chest with its prefix (`GetItem`'s returned remainder).
        assert_eq!(
            chest.slots[2],
            prefixed_slot(48, 7, 3),
            "the overflow stayed with its prefix"
        );
    }

    /// The plan contract's other half: `deposit_all`/`quick_stack`/`restock`
    /// take `&mut world_chest.slots` directly — no bridge, no tuple adapter
    /// (the `**Produces:**` seam Phase 3's dispatch calls through). The
    /// container is the transfer configuration [`open_chest`] guarantees —
    /// the browsed chest's 40-slot window materialized, exactly the C#'s
    /// `Chest.item[]` (`Chest.cs:107-109`) the free-slot walks scan.
    #[test]
    fn the_other_bank_transfers_take_a_world_chest_directly() {
        let mut chest = WorldChest {
            tile: (10, 18),
            slots: Vec::new(),
            name: String::new(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        };
        chest.slots.resize(DEFAULT_MAX_ITEMS as usize, EMPTY_SLOT);
        chest.slots[0] = slot(9, 500);
        let mut inventory: [ItemSlot; INV] = std::array::from_fn(|_| EMPTY_SLOT);
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| EMPTY_SLOT);
        inventory[5] = slot(9, 50); // the hotbar partial — restock's demand
        inventory[20] = slot(9, 100);
        inventory[21] = slot(3, 10);
        let fav = |_: usize| false;
        // `quick_stack` merges only the types the chest already holds
        // (wood — the stone and the hotbar row stay put).
        crate::bank::quick_stack(&mut chest.slots, &mut inventory, &mut [EMPTY_SLOT; 4], &fav);
        assert_eq!(chest.slots[0].count, 600, "quick_stack merged the wood");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 10, "the stone is not a target type");
        // `deposit_all` parks the stone on the first free slot
        // (`ChestUI.cs:579-655`'s walk) and leaves the hotbar alone.
        crate::bank::deposit_all(&mut chest.slots, &mut inventory, &fav);
        assert_eq!(chest.slots[1], slot(3, 10), "deposit_all parked the stone");
        assert_eq!(inventory[21].count, 0);
        assert_eq!(inventory[5].count, 50, "the hotbar is not a deposit source");
        // `restock` fills the hotbar partial from the unprefixed chest stack
        // (the prefix-0 supply gate reads the chest's real prefixes now).
        assert!(
            crate::bank::restock(&mut chest.slots, &mut inventory, &mut ammo),
            "restock filled the partial"
        );
        assert_eq!(inventory[5].count, 650);
        assert_eq!(chest.slots[0].count, 0, "the wood drained");
    }

    /// The transfer window's guarantee: opening a chest materializes its
    /// 40-slot tail (`Chest.cs:107-109`'s shape — the free-slot walks the
    /// `bank.rs` transfers run need it), a fresh open grows the lazy entry
    /// to the C# per-chest shape, and a never-opened entry stays empty.
    #[test]
    fn open_chest_materializes_the_browsed_transfer_window() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert_eq!(app.player.as_ref().unwrap().chest, 0);
        assert_eq!(
            app.world_chests[0].slots.len(),
            DEFAULT_MAX_ITEMS as usize,
            "the browsed chest's 40-slot window is materialized"
        );
        assert!(
            app.world_chests[0]
                .slots
                .iter()
                .all(|s| s.kind == 0 && s.count == 0),
            "the materialized tail is air"
        );
        // The store keeps the window after the close (`Chest.item[]` stays
        // materialized in the C# too).
        assert_eq!(toggle_open(&mut app, 10, 18, 0), 11);
        assert_eq!(app.world_chests[0].slots.len(), DEFAULT_MAX_ITEMS as usize);
        // A stocked slot survives the resize.
        app.world_chests[0].slots[5] = slot(9, 4);
        assert_eq!(toggle_open(&mut app, 10, 18, 0), 10, "re-open");
        assert_eq!(app.world_chests[0].slots[5], slot(9, 4));
        // A never-opened entry stays lazy (the memory shape the plan pins).
        create_chest(&mut app.world_chests, 30, 40);
        assert_eq!(
            app.world_chests[1].slots.len(),
            0,
            "no open, no materialization"
        );
    }

    // --- Phase 5: the rename flow ----------------------------------------

    /// DW-5.1 — the submit writes the store name and the second submit with
    /// the unchanged text re-writes nothing (`ChestUI.cs:853-860`'s
    /// write-if-different guard). The submitted text is the keyboard
    /// session's live text (`Main.npcChatText`'s sync,
    /// `UIVirtualKeyboard.cs:1061-1067`).
    #[test]
    fn dw_5_1_submit_sets_the_store_name() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert!(open_rename(&mut app), "the editor opens");
        // Type through the live session (the on-screen/physical path).
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            keyboard::write_str(&mut s.state, "Custom");
        }
        assert_eq!(rename_submit(&mut app, "Custom"), 12, "`:842` — sound 12");
        assert_eq!(chest_name(&app, 0), "Custom");
        assert!(!app.chest_editing(), "the editor closed (`:843` + `Close`)");
        // The unchanged-name submit skips the write (`:853-860`) — the
        // entity's field stays the same object.
        assert_eq!(rename_submit(&mut app, "Custom"), 12);
        assert_eq!(
            chest_name(&app, 0),
            "Custom",
            "no rewrite for the same name"
        );
    }

    /// DW-5.1 — the empty-if-default rule (`ChestUI.cs:849-852`): a submit
    /// whose text equals `Main.defaultChestName` stores "" — on an unnamed
    /// chest the equal write is skipped, on a renamed chest it clears.
    #[test]
    fn dw_5_1_submit_the_default_name_writes_empty() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        // The unnamed chest prefills the default; submitting it untouched
        // writes nothing ("" == "" after the default-name rule).
        assert!(open_rename(&mut app));
        assert_eq!(app.chest_default_name, "Chest");
        let default = app.chest_default_name.clone();
        assert_eq!(rename_submit(&mut app, &default), 12);
        assert_eq!(
            chest_name(&app, 0),
            "",
            "the default-name submit on an unnamed chest stores nothing"
        );
        // A renamed chest submitting the default string clears to "".
        assert!(open_rename(&mut app));
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            keyboard::write_str(&mut s.state, "Custom");
        }
        rename_submit(&mut app, "Custom");
        assert_eq!(chest_name(&app, 0), "Custom");
        assert!(
            open_rename(&mut app),
            "the editor reopens with the stored name"
        );
        assert_eq!(rename_submit(&mut app, "Chest"), 12);
        assert_eq!(
            chest_name(&app, 0),
            "",
            "the default-name submit cleared the stored name (:849-852)"
        );
    }

    /// DW-5.1 (dirty) — the 20-character cap (`Chest.cs:70`'s
    /// `MaxNameLength` = the ctx-2 ctor's `maxLength`, `UIVirtualKeyboard.cs:
    /// 159`) lives in the keyboard session: the 21st+ characters never land
    /// and the submit stores exactly the capped 20.
    #[test]
    fn dw_5_1_twenty_char_cap_caps_the_session_text() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert!(open_rename(&mut app));
        let long_name = "0123456789ABCDEFGHIJKLM";
        let session_text;
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear(); // the user cleared the prefill before typing
            keyboard::write_str(&mut s.state, long_name);
            assert_eq!(s.state.text.len(), 20, "`SetTextMaxLength(20)` caps");
            session_text = s.state.text.clone();
        } else {
            unreachable!("the rename keyboard is live");
        }
        rename_submit(&mut app, &session_text);
        assert_eq!(chest_name(&app, 0), &long_name[..20]);
    }

    /// DW-5.2 — the cancel closes the editor without writing and plays 12
    /// (`ChestUI.cs:863-869`).
    #[test]
    fn dw_5_2_cancel_closes_editing_without_writing_sound_12() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert!(open_rename(&mut app));
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            keyboard::write_str(&mut s.state, "Old");
        }
        assert_eq!(rename_submit(&mut app, "Old"), 12);
        // Re-open (prefills "Old"), edit, cancel.
        assert!(open_rename(&mut app));
        assert_eq!(app.chest_edit_text(), "Old", "the prefill read the store");
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            keyboard::write_str(&mut s.state, "New");
        }
        assert_eq!(rename_cancel(&mut app), 12, "`:865` — sound 12");
        assert!(!app.chest_editing(), "the editor closed");
        assert_eq!(chest_name(&app, 0), "Old", "the cancel never wrote");
    }

    /// DW-5.3 — the ctx-2 keyboard prefills from the chest: the stored name,
    /// or the tile default for an unnamed one, with the `IngameFancyUI.cs:
    /// 98-125` request shape (`UI.EnterNewName`, `allowEmpty: false`,
    /// `maxLength: 20`, context 2).
    #[test]
    fn dw_5_3_keyboard_context_2_prefills_from_the_chest() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        // The unnamed chest prefills the tile default.
        assert!(open_rename(&mut app));
        {
            let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_ref() else {
                panic!("the rename keyboard is live");
            };
            assert_eq!(s.request.label, keyboard::ENTER_NEW_NAME);
            assert_eq!(s.request.initial, "Chest", "the tile default prefills");
            assert_eq!(s.state.text, "Chest");
            assert_eq!(s.request.allow_empty, false, "the ctor default (:147-156)");
            assert_eq!(
                s.request.max_length, 20,
                "the ctor default == MaxNameLength"
            );
            assert_eq!(s.request.context, Context::Chest);
            assert_eq!(s.tail, KeyboardTail::ChestRename);
            assert!(s.back.is_none(), "the chest tail returns to gameplay");
        }
        assert!(app.menu.is_none(), "the single-mode invariant holds");
        rename_cancel(&mut app);
        // A stored name prefills over the default.
        assert!(open_rename(&mut app));
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            keyboard::write_str(&mut s.state, "Custom");
        }
        rename_submit(&mut app, "Custom");
        assert!(open_rename(&mut app));
        let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_ref() else {
            panic!("the rename keyboard is live");
        };
        assert_eq!(s.state.text, "Custom", "the stored name prefills");
    }

    /// DW-5.3 — Enter routes to the submit tail (the name lands, the
    /// keyboard closes into gameplay) and Escape to the cancel tail (no
    /// write), through the real App routes (`keyboard_enter`/
    /// `keyboard_cancel` — the front-screen key path, flow.rs).
    #[test]
    fn dw_5_3_enter_and_escape_route_to_submit_and_cancel() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert!(open_rename(&mut app));
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            keyboard::write_str(&mut s.state, "Named");
        }
        // Enter (`Main.inputTextEnter`'s chest arm, `UIVirtualKeyboard.cs:
        // 757-762` → `RenameChestSubmit`): the chest text is always valid
        // (`TextIsValidForSubmit`'s `_editingChest` gate).
        app.keyboard_enter();
        assert!(!app.chest_editing(), "Enter closed the editor");
        assert_eq!(app.front, None, "back into gameplay");
        assert_eq!(chest_name(&app, 0), "Named");
        // Escape (`Main.inputTextEscape` → `TryEscapingMenu` → `Cancel()`,
        // `:771-774`/:804-819 → `RenameChestCancel`).
        assert!(open_rename(&mut app));
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            keyboard::write_str(&mut s.state, "Other");
        }
        app.keyboard_cancel();
        assert!(!app.chest_editing(), "Escape closed the editor");
        assert_eq!(app.front, None);
        assert_eq!(chest_name(&app, 0), "Named", "Escape never writes");
    }

    /// DW-5.4 — the rename click no-ops without an open chest: `open_rename`
    /// refuses (`false`, no session — the C# would index `Main.chest[-1]`,
    /// unreachable because the button only draws with a chest), and the
    /// click on the button's anchor consumes nothing.
    #[test]
    fn dw_5_4_rename_click_no_ops_without_an_open_chest() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        assert!(!open_rename(&mut app), "chest < 0 refuses");
        assert!(app.front.is_none(), "no keyboard opened");
        assert_eq!(chest_name(&app, 0), "", "nothing stored");
        // The dirty click: the button's anchor with no chest open.
        app.inventory_open = true;
        app.ui_left_press = true;
        app.mouse_pos = Vec2::new(510.0, 428.0); // the ID-5 row
        assert!(!app.handle_chest_clicks(), "the click falls through");
        assert!(app.front.is_none());
        assert!(
            app.ui_left_press,
            "the press rides on to the crafting/inventory handlers"
        );
    }

    /// `ChestUI.RenameChest` (`ChestUI.cs:827-838`) — the button's dispatch:
    /// not editing → the keyboard opens; editing → the live text submits.
    #[test]
    fn rename_button_click_opens_or_submits() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        // Not editing: the keyboard opens (`:830-833`).
        rename_button_click(&mut app);
        assert!(app.chest_editing(), "the keyboard opened");
        // Editing: the live text submits (`:834-837` → `RenameChestSubmit`).
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            keyboard::write_str(&mut s.state, "Clicked");
        }
        rename_button_click(&mut app);
        assert!(!app.chest_editing());
        assert_eq!(chest_name(&app, 0), "Clicked");
    }

    /// `Player.cs:33091-33096` — a chest-tile click while the editor is live
    /// flushes it (sound 12, editor closed, nothing written) before the open
    /// toggle runs.
    #[test]
    fn the_open_path_flushes_a_live_rename_editor() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0);
        place_chest(&mut w, 30, 40, 21, 0);
        let mut app = interact_app(w);
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert!(open_rename(&mut app));
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            keyboard::write_str(&mut s.state, "Flushed");
        }
        // The same chest's tile click: the flush, then the open toggle
        // closes the chest (sound 12 flush → 11 close).
        assert!(try_open_chest_at_cursor(&mut app, click(10, 18)));
        assert!(!app.chest_editing(), "the flush cleared the editor");
        assert!(app.front.is_none());
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the toggle-close followed the flush"
        );
        assert_eq!(
            chest_name(&app, 0),
            "",
            "the flush never writes (`npcChatText = \"\"`)"
        );
    }

    // --- Phase 3: the `.wld` persistence (M6-SAVE-1) -------------------------

    use tr_core::wld::sections::{WldChest, WldChestSlot};

    fn wld_slot(kind: i32, stack: i16, prefix: u8) -> WldChestSlot {
        WldChestSlot {
            stack,
            kind,
            prefix,
        }
    }

    /// A wire record in the C# array shape: exactly `max` slot records.
    fn wld_chest(x: i32, y: i32, name: &str, max: usize, slots: Vec<WldChestSlot>) -> WldChest {
        let mut padded = slots;
        padded.resize(max, WldChestSlot::default());
        WldChest {
            x,
            y,
            name: name.into(),
            max_items: max as i32,
            slots: padded,
        }
    }

    /// A dresser anchor tile: kind 88 at a `frameX % 54 == 0` / `frameY % 36 == 0`
    /// corner (`FixDresserChests:3276`).
    fn set_dresser(w: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = 88;
        w.tiles[i].frame_x = frame_x;
        w.tiles[i].frame_y = frame_y;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    /// DW-3.1 — the entity/file seam: `from_wld(records)` then
    /// `to_wld(entities)` returns the identical wire list (the round-trip the
    /// byte-level codec tests pin from the other side).
    #[test]
    fn test_DW_3_1_from_wld_to_wld_round_trips_the_entity_store() {
        let records = vec![
            wld_chest(10, 18, "", 4, vec![wld_slot(9, 6, 0)]),
            wld_chest(30, 40, "Vault", 3, vec![wld_slot(24, 4, 3)]),
            wld_chest(50, 60, "", 40, vec![wld_slot(8, 1, 0)]),
        ];
        let chests = from_wld(&records);
        assert_eq!(chests.len(), 3);
        // The entity's own fields: the name and the per-chest max come from
        // the file (`LoadChests:2735`/`:2738`).
        assert_eq!(chests[1].name, "Vault");
        assert_eq!(chests[1].max_items, 3);
        // `slots.len()` is the max — the materialized `Chest.item[]`.
        assert_eq!(chests[0].slots.len(), 4);
        assert_eq!(chests[0].slots[0], slot(9, 6));
        assert_eq!(chests[0].slots[1], EMPTY_SLOT);
        assert_eq!(chests[2].slots.len(), 40);
        assert_eq!(chests[1].slots[0], prefixed_slot(24, 4, 3));
        assert_eq!(to_wld(&chests), records, "entities re-encode byte-equal");
    }

    /// DW-3.2 — the duplicate-`(x, y)` removal (`LoadChests:2781-2796`): the
    /// walk goes ascending and keeps the FIRST record at a position, so the
    /// earliest chest survives and every later one is tombstoned in place
    /// (index-stable — a held `Player.chest` still points at it).
    #[test]
    fn test_DW_3_2_duplicate_positions_drop_ascending() {
        let records = vec![
            wld_chest(10, 18, "First", 2, vec![wld_slot(9, 1, 0)]),
            wld_chest(30, 40, "", 2, vec![]),
            wld_chest(10, 18, "Second", 2, vec![wld_slot(9, 2, 0)]),
            wld_chest(30, 40, "", 2, vec![]),
            wld_chest(50, 60, "", 2, vec![wld_slot(9, 3, 0)]),
        ];
        let chests = from_wld(&records);
        assert_eq!(chests.len(), 5, "the C# array keeps its slots");
        assert_eq!(chests[0].name, "First", "the first record survives");
        assert_eq!(chests[1].name, "", "the first of the (30,40) pair survives");
        assert_eq!(
            chests[2].tile, REMOVED_TILE,
            "the later duplicate is removed"
        );
        assert_eq!(chests[3].tile, REMOVED_TILE);
        assert_eq!(chests[4].tile, (50, 60));
        // A removed slot reads as absent (`Chest.RemoveChest:132-141`).
        assert_eq!(chest_slots(&chests, 2), None);
        assert_eq!(find_chest(&chests, 10, 18), Some(0));
        // The store still round-trips only the survivors.
        assert_eq!(to_wld(&chests).len(), 3);
    }

    /// DW-3.2 — `Chest.FixLoadedData:3664` → `Item.FixAgainstExploit:49489`
    /// in the C# order: banned -> air (the stack/prefix arms are skipped),
    /// stack clamp to `maxStack`, unrollable prefix reset. One item per rule,
    /// plus the all-three hostile item whose outcome proves the order.
    #[test]
    fn test_DW_3_2_fix_loaded_data_runs_banned_then_stack_then_prefix() {
        let records = vec![wld_chest(
            10,
            18,
            "",
            4,
            vec![
                wld_slot(58, 99, 7),    // banned (ItemID.cs:345) + oversized + prefixed
                wld_slot(619, 9999, 0), // a normal item past its maxStack
                wld_slot(9, 5, 1),      // wood carries no rollable prefix
                wld_slot(24, 4, 3),     // a valid prefixed item stays untouched
            ],
        )];
        let chests = from_wld(&records);
        let slots = &chests[0].slots;
        assert_eq!(slots[0], EMPTY_SLOT, "banned wins — the item is air");
        assert_eq!(
            slots[1].count,
            registry::max_stack(619),
            "the stack clamps to maxStack"
        );
        assert_eq!(slots[2].prefix, 0, "an unrollable prefix resets");
        assert_eq!(slots[2].count, 5, "the prefix reset keeps the stack");
        assert_eq!(
            slots[3],
            prefixed_slot(24, 4, 3),
            "a valid item is untouched"
        );
    }

    /// DW-3.2 — the `netDefaults` type-range guard (`Item.cs:49929-49932`): a
    /// type at or past `ItemID.Count` turns to air before the fix-ups run.
    #[test]
    fn test_DW_3_2_an_out_of_range_item_drops_to_air() {
        let records = vec![wld_chest(10, 18, "", 2, vec![wld_slot(999_999, 5, 0)])];
        assert_eq!(from_wld(&records)[0].slots[0], EMPTY_SLOT);
    }

    /// DW-3.2 — the legacy negative-stack record (`LoadChests:2764-2769`):
    /// the type and prefix are read and the stack becomes 1, which the
    /// app-side store keeps as a one-count item. A pre-294 stream, where the
    /// shape is the legacy one.
    #[test]
    fn test_DW_3_2_a_legacy_negative_stack_becomes_a_one_count_item() {
        // The section bytes: one chest, a shared slot count of 1, one record
        // whose stack is negative — the `i16 -5` + `(type, prefix)` shape.
        let mut b = tr_core::binio::Writer::new();
        b.i16(1); // chest count
        b.i16(1); // the shared slot count (`LoadChests:2726-2729`)
        b.i32(10);
        b.i32(18);
        b.cs_string("");
        b.i16(-5);
        b.i32(24); // Wooden Sword — a prefixable kind
        b.u8(3);
        let bytes = b.into_inner();
        let mut r = tr_core::binio::Reader::new(&bytes);
        let records =
            tr_core::wld::sections::chests::read(&mut r, 293).expect("the legacy record reads");
        assert_eq!(records[0].slots[0].stack, 1, "`item.stack = 1`");
        assert_eq!(from_wld(&records)[0].slots[0], prefixed_slot(24, 1, 3));
    }

    /// DW-3.3 — the save pre-pass (`SaveChests:1609-1642`): a chest whose four
    /// anchor cells are not all active `tileContainer` tiles is destroyed, and
    /// `to_wld` then drops it — the write-time mutation is observable in the
    /// file. One probe cell per way an anchor dies.
    #[test]
    fn test_DW_3_3_the_pre_pass_destroys_chests_with_dead_anchors() {
        let mut w = world();
        place_chest(&mut w, 10, 18, 21, 0); // a live 2x2 anchor
        let mut chests = vec![
            chest_at(10, 18, "Kept"),
            chest_at(30, 40, "NotAContainer"),
            chest_at(50, 60, "GoneTile"),
            chest_at(2000, 2000, "OutOfBounds"),
        ];
        destroy_invalid_anchors(&mut chests, &w);
        assert_eq!(chests[0].tile, (10, 18), "a live anchor survives");
        assert_eq!(chests[1].tile, REMOVED_TILE, "a non-container anchor dies");
        assert_eq!(chests[2].tile, REMOVED_TILE, "a missing anchor dies");
        assert_eq!(
            chests[3].tile, REMOVED_TILE,
            "an anchor past the world bounds dies"
        );
        let written = to_wld(&chests);
        assert_eq!(written.len(), 1, "the destroyed chests are dropped on save");
        assert_eq!(written[0].name, "Kept");
    }

    /// The `chest_app` fixture's chest shape with a name.
    fn chest_at(x: i32, y: i32, name: &str) -> WorldChest {
        WorldChest {
            tile: (x, y),
            slots: Vec::new(),
            name: name.into(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        }
    }

    /// The port's implicit-air encoding meets the file's `maxItems` records:
    /// a chest whose stored slots stop early still writes (and reads back)
    /// every air the C# `for l < maxItems` loop emits.
    #[test]
    fn test_DW_3_3_implicit_trailing_airs_are_materialized_on_both_sides() {
        let mut chests = vec![chest_at(10, 18, "")];
        chests[0].slots.push(slot(9, 6));
        let written = to_wld(&chests);
        assert_eq!(written[0].slots.len(), 40, "40 air records pad the file");
        assert_eq!(written[0].slots[1].stack, 0);
        let back = from_wld(&written);
        assert_eq!(back[0].slots.len(), 40);
        assert_eq!(back[0].slots[0], slot(9, 6));
        assert_eq!(back[0].slots[39], EMPTY_SLOT);
    }

    /// DW-3.4 — the `< 115` stream (`LoadChests:2801-2804`): the dresser fix-up
    /// scans the tiles and creates a chest per anchor through
    /// `Chest.CreateChest` — refusing a taken anchor and a full store with the
    /// C# `-1`.
    #[test]
    fn test_DW_3_4_dresser_anchors_create_chests_below_115() {
        let mut w = world();
        set_dresser(&mut w, 10, 18, 0, 0); // an anchor: 0 % 54 == 0, 0 % 36 == 0
        set_dresser(&mut w, 30, 40, 54, 36); // the next anchor frame
        set_dresser(&mut w, 50, 60, 18, 0); // not a corner — no chest
        set_dresser(&mut w, 70, 80, 0, 0); // an anchor that already owns a chest
        let records = vec![wld_chest(70, 80, "Taken", 2, vec![])];
        let mut chests = from_wld(&records);
        fix_dresser_chests(&mut chests, &w);
        let mut anchors: Vec<(i32, i32)> = chests
            .iter()
            .filter(|c| !is_removed(c))
            .map(|c| c.tile)
            .collect();
        anchors.sort_unstable();
        assert_eq!(
            anchors,
            vec![(10, 18), (30, 40), (70, 80)],
            "both free anchors get a chest; the taken one is refused"
        );
        let made = find_chest(&chests, 10, 18).expect("the dresser chest exists");
        assert_eq!(
            chest_name_at(&chests, made),
            "",
            "`CreateWorldChest`'s empty name"
        );
        assert_eq!(chests[made].slots.len(), 40, "the C# default maxItems");
        // The fix-up is the caller's version gate: nothing else creates a chest.
        assert_eq!(from_wld(&records).len(), 1, "no fix-up on the modern path");
    }

    /// [`stored_name`]'s index form for the tests.
    fn chest_name_at(chests: &[WorldChest], idx: usize) -> &str {
        stored_name(chests, idx as i32)
    }

    /// DW-3.4 — the whole `.wld` cycle from the app side: the file's chests
    /// become the store through the loader (`LoadWorld_Version2:1781`'s read),
    /// which is what displaces the `chest_gen` re-roll.
    #[test]
    fn test_DW_3_4_a_wld_file_loads_its_chests_into_the_store() {
        let world = World::new("W".into(), 120, 120);
        let store = tr_core::wld::sections::SectionData {
            chests: vec![
                wld_chest(10, 18, "Stash", 2, vec![wld_slot(9, 6, 0)]),
                wld_chest(30, 40, "", 2, vec![]),
            ],
            ..Default::default()
        };
        let buf = tr_core::wld::to_bytes(&world, &store, 1).expect("writes");
        let table = tr_core::wld::SectionTable::open(&buf).expect("opens");
        let loaded = tr_core::wld::load_sections(&table).expect("walks");
        let chests = from_wld(&loaded.chests);
        assert_eq!(chests.len(), 2);
        assert_eq!(chests[0].name, "Stash");
        assert_eq!(chests[0].slots[0], slot(9, 6));
        assert_eq!(chests[1].tile, (30, 40));
    }

    /// DW-3.5 — the golden rule's citations on the ported blocks.
    #[test]
    fn test_DW_3_5_the_citations_are_present() {
        let here = include_str!("chest.rs");
        for anchor in [
            "WorldFile.SaveChests:1606",
            "WorldFile.LoadChests:2723",
            "SaveChests:1609-1642",
            "LoadChests:2781-2800",
            "FixDresserChests:3269-3282",
            "Item.cs:49489-49504",
            "Chest.cs:3664",
            "ItemID.cs:345",
            "Main.cs:10076-10080",
        ] {
            assert!(here.contains(anchor), "missing anchor {anchor}");
        }
        let codec = include_str!("../../tr-core/src/wld/sections/chests.rs");
        assert!(
            codec.contains("WorldFile.SaveChests:1606"),
            "codec write anchor"
        );
        assert!(
            codec.contains("WorldFile.LoadChests:2723"),
            "codec read anchor"
        );
    }
}
