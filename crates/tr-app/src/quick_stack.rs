//! The quick-stack-all engine — a 1:1 port of `Terraria/GameContent/QuickStacking.cs`
//! (1.4.5.8) behind the inventory's quick-stack button (`Main.cs:41833-41868`,
//! site `:41849`) and `Player.QuickStackAllChests` (`Player.cs:23183-23189`).
//!
//! W4-g50 RE-PROBE CONSTANTS-COPY of W4-g21 batch 3 (`c07c1eb95`,
//! `origin/task/W4-g21-sounds-main-1` `crates/tr-app/src/quick_stack.rs`,
//! tip lineage 7da02f439): the engine below is copied verbatim — KEEP-ONE at
//! the union merge (g21's copy is the survivor). The net entry this lane
//! adds is `quick_stack_net_source` (the `ReadNetInventory` scratch shape,
//! `QuickStacking.cs:308-321`): the host's `85` arm reads the sender's slot
//! list, snapshots those slots from the `5`-fed inventory copy and runs
//! `transfer` + a slot-wise restore (`RestoreToPlayer`'s `netMode == 2`
//! arm re-sends one `5` per slot, `:353-356`).
//!
//! Structure mirrors the source: [`pack_quick_stackable_items`] is
//! `PackQuickStackableItems` (`:377-402`), [`transfer`] is `Transfer`
//! (`:404-459`) with the `DestinationHelper` metrics (`:14-98`) and the
//! `MatchingItemTypeDestinationList` first-free-slot lookup (`:100-185`),
//! [`quick_stack_to_nearby_inventories`] is the banks-then-chests entry
//! (`:232-236`).
//!
//! Faithfulness notes (W4-g21 batch 3):
//! * The C# `SourceInventory` holds live `Item` references; the port snapshots
//!   the source slots into a Vec and writes them back in `RestoreToPlayer`
//!   (`:343-363`) — behaviour-identical because the transfer only mutates the
//!   snapshot and the destinations, never the source slots in place.
//! * `Chest.VisualizeChestTransfer` / `IndicateBlockedChest` /
//!   `ItemSlot.IndicateBlockedSlot` are the particle/pulse visuals
//!   (`Chest.cs:318-339/:3582`, `ItemSlot.cs:555-558`) — the port has no
//!   ItemTransfer particle system, so they are no-ops (docs/ANALYSIS.md).
//! * The net arms (`SendQuickStackToNearbyChests` `:287-297`, `SyncSlot`
//!   `:91-97`, `RestoreToPlayer`'s `SendData(5)`) are single-player no-ops.
//! * `GetBanksInRangeOf`'s projectile-bank half (`NearbyChests.cs:33-58`, the
//!   Money Trough 525/960 + Void Lens 734) rides the unported interactive
//!   projectiles (hosts 24-27 stay deferred); the tile-bank half
//!   (`:59-83`, piggy 29 / safe 97 / defender 463 / void 491) is ported.

use crate::item::{ItemSlot, EMPTY_SLOT};

/// `QuickStacking.GetCategory` (`QuickStacking.cs:227-230`) →
/// `ItemSorting.GetSortingLayerIndex`.
fn category(kind: u16) -> usize {
    crate::item_sorting::sorting_layer_index(kind)
}

/// `Item.CanStack` (`Item.cs:49865-49871`): same type AND same prefix.
fn can_stack(a: &ItemSlot, b: &ItemSlot) -> bool {
    a.kind == b.kind && a.prefix == b.prefix
}

fn is_air(s: &ItemSlot) -> bool {
    s.kind == 0 || s.count == 0
}

/// One source slot: its vanilla inventory index (grid 0..50, coins 50..54,
/// ammo 54..58, void vault 58+) and the item snapshot.
#[derive(Clone, Copy)]
pub(crate) struct SourceSlot {
    /// The `PlayerItemSlotID.SlotReference.SlotId` — the port keeps the vanilla
    /// grid/coin/ammo numbering; the void vault rows start at 58.
    pub(crate) slot_id: usize,
    pub(crate) item: ItemSlot,
    /// `SourceInventory.transferBlocked[i]` (`:195`).
    pub(crate) blocked: bool,
}

/// `QuickStacking.DestinationHelper` (`QuickStacking.cs:14-98`): one non-empty
/// destination chest with its free-slot queue and per-category scores.
pub(crate) struct Dest<'a> {
    slots: &'a mut [ItemSlot],
    item_count: usize,
    locked: bool,
    transfer_blocked: bool,
    free_slots: Vec<usize>,
    free_start: usize,
    category_scores: Vec<usize>,
    /// `PositionedChest.position` — the chest centre (the particle anchor).
    #[allow(dead_code)]
    position: (f32, f32),
    /// `ChestIndex` (`:36-43`) — the world-chest store index (banks assert).
    chest_index: i32,
    /// W4-g50 net extension: the slots `FillStack`/`InsertIntoFreeSlot`
    /// touched — the C# fires `SyncSlot(slot)` at each write (`:537`/`:565`,
    /// the `SendData(32)` seam); the host arm replays them after the walk.
    changed: Vec<usize>,
}

impl<'a> Dest<'a> {
    /// `Reset` (`:49-64`) + the `BuildDestinationMetricsAndStackItems` free-slot
    /// and category-score pre-pass (`:476-484`).
    pub(crate) fn new(slots: &'a mut [ItemSlot], locked: bool, position: (f32, f32), chest_index: i32) -> Self {
        let item_count = slots.len();
        let mut free_slots = Vec::new();
        let mut category_scores = vec![0usize; crate::item_sorting::SETUP_LAYER_COUNT];
        for (i, s) in slots.iter().enumerate() {
            if is_air(s) {
                free_slots.push(i);
            } else {
                let c = category(s.kind);
                if c < category_scores.len() {
                    category_scores[c] += 1;
                }
            }
        }
        Self {
            slots,
            item_count,
            locked,
            transfer_blocked: false,
            free_slots,
            free_start: 0,
            category_scores,
            position,
            chest_index,
            changed: Vec::new(),
        }
    }
    /// The world-chest store index (`ChestIndex`, `:36-43`) — the g50 net
    /// arm's `SyncSlot`/blocked-list key.
    pub(crate) fn chest_index(&self) -> i32 {
        self.chest_index
    }
    /// The slots the transfer wrote, in write order (the `SyncSlot` sites)
    /// — the g50 host arm broadcasts one `32` per entry.
    pub(crate) fn changed_slots(&self) -> impl Iterator<Item = (usize, ItemSlot)> + '_ {
        self.changed.iter().map(|&i| (i, self.slots[i]))
    }
    /// `IsEmpty` (`:45`): every slot is free.
    fn is_empty(&self) -> bool {
        self.free_slots.len() == self.item_count
    }
    /// `HasFreeSlots` (`:47`).
    fn has_free(&self) -> bool {
        self.free_start < self.free_slots.len()
    }
    /// `TryGetCategoryScore` (`:76-84`): 0 scores answer false.
    fn category_score(&self, c: usize) -> Option<usize> {
        let s = self.category_scores.get(c).copied().unwrap_or(0);
        (s > 0).then_some(s)
    }
    /// `ConsumeFreeSlot` (`:86-89`).
    fn consume_free(&mut self) -> usize {
        let i = self.free_slots[self.free_start];
        self.free_start += 1;
        i
    }
}

/// `QuickStacking.PackQuickStackableItems` (`:377-387`) +
/// `AddQuickStackableItems` (`:389-402`): the storage rows 10..50 (the C#
/// `Inventory0 + 10`, 40 slots), then the void vault when `includeVoidBag` and
/// `useVoidBag`. Only non-air, non-favorited, non-coin stacks pack.
fn pack_quick_stackable_items(
    grid: &[ItemSlot; crate::item::INV],
    vault: Option<&[ItemSlot]>,
    fav: &dyn Fn(usize) -> bool,
) -> Vec<SourceSlot> {
    let mut out = Vec::new();
    for i in 10..50 {
        let s = grid[i];
        if !is_air(&s) && !fav(i) && !is_coin(s.kind) {
            out.push(SourceSlot { slot_id: i, item: s, blocked: false });
        }
    }
    if let Some(v) = vault {
        for (j, &s) in v.iter().enumerate() {
            if !is_air(&s) && !fav(58 + j) && !is_coin(s.kind) {
                out.push(SourceSlot { slot_id: 58 + j, item: s, blocked: false });
            }
        }
    }
    out
}

fn is_coin(kind: u16) -> bool {
    matches!(kind, 71 | 72 | 73 | 74)
}

/// `QuickStacking.FillStack` (`:526-539`): merge the source into a partial
/// destination stack up to its maxStack; the source turns to air at 0.
fn fill_stack(src: &mut ItemSlot, dest: &mut ItemSlot) {
    let max = crate::registry::max_stack(dest.kind);
    let num = (max - dest.count).min(src.count);
    dest.count += num;
    src.count -= num;
    if src.count == 0 {
        *src = EMPTY_SLOT;
    }
}

/// `QuickStacking.Consolidate` (`:541-558`): fold every later same-type source
/// stack into source `i` up to its maxStack.
fn consolidate(source: &mut [SourceSlot], i: usize) {
    let max = crate::registry::max_stack(source[i].item.kind);
    for j in (i + 1)..source.len() {
        if can_stack(&source[i].item, &source[j].item) {
            let num = (max - source[i].item.count).min(source[j].item.count);
            source[i].item.count += num;
            source[j].item.count -= num;
            if source[j].item.count == 0 {
                source[j].item = EMPTY_SLOT;
            }
        }
    }
}

/// `QuickStacking.InsertIntoFreeSlot` (`:560-566`): swap the source whole into
/// the destination's next free slot.
fn insert_into_free_slot(src: &mut ItemSlot, dest: &mut Dest<'_>) {
    let n = dest.consume_free();
    std::mem::swap(src, &mut dest.slots[n]);
    dest.changed.push(n); // `dest.SyncSlot(num)` (`:565`)
}

/// `QuickStacking.BuildDestinationMetricsAndStackItems` (`:474-509`): for each
/// occupied destination slot, merge every same-type source stack into it (a
/// locked destination blocks the source instead); the destination joins the
/// type's matching list when it still has free slots.
fn build_metrics_and_stack(
    source: &mut [SourceSlot],
    dest: &mut Dest<'_>,
    matching: &mut Vec<(u16, usize)>,
    dest_idx: usize,
) {
    for i in 0..dest.item_count {
        if is_air(&dest.slots[i]) {
            continue; // the free slots were queued in `Dest::new`
        }
        let dest_kind = dest.slots[i].kind;
        for j in 0..source.len() {
            if source[j].item.kind != dest_kind {
                continue;
            }
            if dest.locked {
                source[j].blocked = true;
                dest.transfer_blocked = true;
                continue;
            }
            let max = crate::registry::max_stack(dest.slots[i].kind);
            if can_stack(&source[j].item, &dest.slots[i]) && dest.slots[i].count < max {
                fill_stack(&mut source[j].item, &mut dest.slots[i]);
                dest.changed.push(i); // `FillStack`'s `dest.SyncSlot(slotIndex)` (`:537`)
                if is_air(&source[j].item) {
                    continue;
                }
            }
            // `destinationsForItemTypes.Add(item.type, dest)` (`:506`) — the
            // list dedupes by destination (`MatchingItemTypeDestinationList.Add`,
            // `:141-152`); the port keeps `(type, dest_idx)` pairs unique.
            if !matching.iter().any(|&(t, d)| t == dest_kind && d == dest_idx) {
                matching.push((dest_kind, dest_idx));
            }
        }
    }
}

/// `MatchingItemTypeDestinationList.Lookup` (`:168-184`): the first destination
/// for the type that still has free slots (the C# advances `firstEntryForType`
/// past the exhausted ones — the port scans the pairs in insertion order).
fn lookup_matching(matching: &[(u16, usize)], kind: u16, dests: &[Dest<'_>]) -> Option<usize> {
    matching
        .iter()
        .filter(|&&(t, _)| t == kind)
        .map(|&(_, d)| d)
        .find(|&d| dests[d].has_free())
}

/// `QuickStacking.TryGetBestDestinationForCategory` (`:511-524`): the free-slot
/// destination with the highest score for the category (strict `>` keeps the
/// first on ties).
fn best_for_category(c: usize, dests: &[Dest<'_>]) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_score = usize::MIN;
    for (d, dest) in dests.iter().enumerate() {
        if dest.has_free() {
            if let Some(score) = dest.category_score(c) {
                if score > best_score {
                    best = Some(d);
                    best_score = score;
                }
            }
        }
    }
    best
}

/// `QuickStacking.Transfer` (`:404-459`): the matching-type merge pass, then the
/// smart-stack category pass. Returns the blocked world-chest indices
/// (`_blockedChests`, `:450-458`).
pub(crate) fn transfer(source: &mut [SourceSlot], dests: &mut [Dest<'_>], smart_stack: bool) -> Vec<i32> {
    let mut matching: Vec<(u16, usize)> = Vec::new();
    for (d, dest) in dests.iter_mut().enumerate() {
        if !dest.is_empty() {
            build_metrics_and_stack(source, dest, &mut matching, d);
        }
    }
    // The matching-type pass (`:421-429`).
    for i in 0..source.len() {
        if !is_air(&source[i].item) {
            if let Some(d) = lookup_matching(&matching, source[i].item.kind, dests) {
                consolidate(source, i);
                insert_into_free_slot(&mut source[i].item, &mut dests[d]);
            }
        }
    }
    // The smart-stack pass (`:430-449`).
    if smart_stack {
        for j in 0..source.len() {
            if !is_air(&source[j].item) && !source[j].blocked {
                if let Some(d) = best_for_category(category(source[j].item.kind), dests) {
                    if dests[d].locked {
                        source[j].blocked = true;
                        dests[d].transfer_blocked = true;
                    } else {
                        consolidate(source, j);
                        insert_into_free_slot(&mut source[j].item, &mut dests[d]);
                    }
                }
            }
        }
    }
    dests
        .iter()
        .filter(|d| d.transfer_blocked)
        .map(|d| d.chest_index)
        .collect()
}

/// `QuickStacking.RestoreToPlayer` (`:343-363`): write every source snapshot
/// back to its slot (the blocked ones too — the C# restores them and pulses
/// the slot red, `ItemSlot.IndicateBlockedSlot`).
fn restore_to_player(
    source: &[SourceSlot],
    grid: &mut [ItemSlot; crate::item::INV],
    vault: Option<&mut [ItemSlot]>,
) {
    for s in source {
        if s.slot_id < 50 {
            grid[s.slot_id] = s.item;
        } else if let Some(v) = vault.as_deref_mut() {
            let j = s.slot_id - 58;
            if j < v.len() {
                v[j] = s.item;
            }
        }
    }
}

/// The quick-stack destination set: the world chests in range (the banks are
/// handled separately by [`quick_stack_to_nearby_banks`]).
pub(crate) struct QuickStackInput<'a> {
    pub grid: &'a mut [ItemSlot; crate::item::INV],
    /// The void vault (`Player.bank4`) when `useVoidBag` — packed as source and
    /// restored, never a destination.
    pub vault: Option<&'a mut [ItemSlot]>,
    pub fav: &'a dyn Fn(usize) -> bool,
    /// `(slots, locked, centre, store_index)` per in-range world chest, in
    /// `Main.chest[]` order (`NearbyChests.GetChestsInRangeOf`, `:10-31`).
    pub chests: Vec<(&'a mut [ItemSlot], bool, (f32, f32), i32)>,
    pub smart_stack: bool,
}

/// `QuickStacking.QuickStackToNearbyChests` (`:251-266`, the single-player
/// `Transfer` + `RestoreToPlayer` arm): pack the storage rows (+ void vault),
/// transfer into the in-range chests, restore the leftovers. Returns the
/// blocked chest indices (the `IndicateBlockedChest` particle is a no-op).
#[allow(dead_code)] // the single-player entry is g21's button lane; g50 uses the net path
pub(crate) fn quick_stack_to_nearby_chests(input: QuickStackInput<'_>) -> Vec<i32> {
    let QuickStackInput { grid, vault, fav, chests, smart_stack } = input;
    let mut source = pack_quick_stackable_items(grid, vault.as_deref(), fav);
    let mut dests: Vec<Dest<'_>> = chests
        .into_iter()
        .map(|(slots, locked, pos, idx)| Dest::new(slots, locked, pos, idx))
        .collect();
    let blocked = transfer(&mut source, &mut dests, smart_stack);
    restore_to_player(&source, grid, vault);
    blocked
}

/// `QuickStacking.QuickStackToNearbyBanks` (`:238-249`): MoveCoins into each
/// in-range bank, then pack the storage rows (NO void vault) and transfer into
/// the banks (never smart-stacked, never locked). The `MoveCoins` pre-pass is
/// the caller's (it owns the audio/rng); this fn runs the item transfer.
#[allow(dead_code)] // g21's button lane owns the caller
pub(crate) fn quick_stack_to_nearby_banks(
    grid: &mut [ItemSlot; crate::item::INV],
    fav: &dyn Fn(usize) -> bool,
    banks: Vec<&mut [ItemSlot]>,
) {
    let mut source = pack_quick_stackable_items(grid, None, fav);
    let mut dests: Vec<Dest<'_>> = banks
        .into_iter()
        .map(|slots| Dest::new(slots, false, (0.0, 0.0), -1))
        .collect();
    let _ = transfer(&mut source, &mut dests, false);
    restore_to_player(&source, grid, None);
}

/// `Player.Settings.CycleQuickStackMode` (`Player.cs:374-377`): the
/// QuickStack ↔ SmartStack toggle (the right-click arm, `Main.cs:41854`).
#[allow(dead_code)] // g21's button lane owns the caller
pub(crate) fn cycle_quick_stack_mode(mode: u8) -> u8 {
    if mode == 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, name: "", count, prefix: 0 }
    }

    /// `Transfer`'s matching-type pass: a source stack merges into a partial
    /// destination stack of the same type, the overflow takes a free slot.
    #[test]
    fn w4_g21_transfer_merges_matching_types_then_fills_free_slots() {
        let mut grid = [EMPTY_SLOT; crate::item::INV];
        grid[10] = slot(9, 50); // 50 wood
        let mut chest = [slot(9, 9900), EMPTY_SLOT, EMPTY_SLOT]; // wood maxStack 9999
        let blocked = quick_stack_to_nearby_chests(QuickStackInput {
            grid: &mut grid,
            vault: None,
            fav: &|_| false,
            chests: vec![(&mut chest[..], false, (0.0, 0.0), 0)],
            smart_stack: false,
        });
        assert!(blocked.is_empty());
        assert_eq!(chest[0].count, 9950, "the 50 wood merged into the partial stack");
        assert!(is_air(&grid[10]), "the source drained");
    }

    /// A non-matching type does NOT transfer without smart-stack (the C# only
    /// fills free slots for types the chest already holds).
    #[test]
    fn w4_g21_non_matching_type_stays_without_smart_stack() {
        let mut grid = [EMPTY_SLOT; crate::item::INV];
        grid[10] = slot(9, 50);
        let mut chest = [slot(2, 10), EMPTY_SLOT]; // stone, not wood
        quick_stack_to_nearby_chests(QuickStackInput {
            grid: &mut grid,
            vault: None,
            fav: &|_| false,
            chests: vec![(&mut chest[..], false, (0.0, 0.0), 0)],
            smart_stack: false,
        });
        assert_eq!(grid[10].count, 50, "the wood stayed");
        assert!(is_air(&chest[1]), "no free-slot fill for a foreign type");
    }

    /// Smart-stack routes a foreign type into the chest whose category score
    /// is highest (`TryGetBestDestinationForCategory`).
    #[test]
    fn w4_g21_smart_stack_routes_by_category_score() {
        let mut grid = [EMPTY_SLOT; crate::item::INV];
        grid[10] = slot(9, 50); // wood
        // Chest 0 holds stone (a different category), chest 1 holds wood.
        let mut c0 = [slot(2, 10), EMPTY_SLOT];
        let mut c1 = [slot(9, 10), EMPTY_SLOT];
        quick_stack_to_nearby_chests(QuickStackInput {
            grid: &mut grid,
            vault: None,
            fav: &|_| false,
            chests: vec![
                (&mut c0[..], false, (0.0, 0.0), 0),
                (&mut c1[..], false, (0.0, 0.0), 1),
            ],
            smart_stack: true,
        });
        // The wood merges into chest 1's partial wood stack (matching pass).
        assert_eq!(c1[0].count, 60);
        assert!(is_air(&grid[10]));
    }

    /// A locked destination blocks the matching source (the C# sets
    /// `transferBlocked` on both, `:492-497`).
    #[test]
    fn w4_g21_locked_chest_blocks_the_matching_source() {
        let mut grid = [EMPTY_SLOT; crate::item::INV];
        grid[10] = slot(9, 50);
        let mut chest = [slot(9, 10), EMPTY_SLOT];
        let blocked = quick_stack_to_nearby_chests(QuickStackInput {
            grid: &mut grid,
            vault: None,
            fav: &|_| false,
            chests: vec![(&mut chest[..], true, (0.0, 0.0), 3)],
            smart_stack: false,
        });
        assert_eq!(blocked, vec![3], "the locked chest is reported");
        assert_eq!(grid[10].count, 50, "the source stayed (restored)");
        assert_eq!(chest[0].count, 10, "nothing merged into the locked chest");
    }

    /// Favorited and coin stacks never pack (`AddQuickStackableItems`, `:395`).
    #[test]
    fn w4_g21_favorited_and_coin_stacks_never_pack() {
        let mut grid = [EMPTY_SLOT; crate::item::INV];
        grid[10] = slot(9, 50); // favorited
        grid[11] = slot(71, 50); // copper coin
        grid[12] = slot(2, 30); // plain stone
        let mut chest = [slot(2, 10), slot(9, 10), slot(71, 10), EMPTY_SLOT];
        quick_stack_to_nearby_chests(QuickStackInput {
            grid: &mut grid,
            vault: None,
            fav: &|i| i == 10,
            chests: vec![(&mut chest[..], false, (0.0, 0.0), 0)],
            smart_stack: true,
        });
        assert_eq!(grid[10].count, 50, "the favorited wood stayed");
        assert_eq!(grid[11].count, 50, "the coin stayed");
        assert!(is_air(&grid[12]), "the plain stone transferred");
    }

    /// `CycleQuickStackMode` toggles 0 ↔ 1.
    #[test]
    fn w4_g21_cycle_quick_stack_mode_toggles() {
        assert_eq!(cycle_quick_stack_mode(0), 1);
        assert_eq!(cycle_quick_stack_mode(1), 0);
    }
}
