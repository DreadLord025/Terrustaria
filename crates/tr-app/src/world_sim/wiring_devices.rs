//! The wired hopper + extractinator — the app half of `Wiring.Hopper`
//! (`Wiring.cs:2638-2678`) and `Wiring.Extractinator` (`:2554-2584`), the
//! W4-f11 remainder arms (`HitWireSingle`'s cases 21/467 and 219/642).
//!
//! **DEPRECATED-BY-UNION (research/95 §5 → research/71, EXECUTED on e20
//! `cf0b8f9c`)**: the hopper/extractinator halves of this file (`run_hopper`,
//! `run_extractinator`, the `try_*` chest helpers, `coins_split`,
//! `test_chest` + their witnesses) are the f11 duplicate halves — the
//! merger drops them for e20's `hopper_pull`/`extractinator_roll` bodies.
//! The f11 must-merge wins are ALREADY merged on e20 (PlaySound(7)
//! `f167082e`, the grab/locked/coins_split witnesses `b0d0836f`+`5272cd52`);
//! nothing left to salvage here. **The mass-wire append at the file tail
//! (`can_do_wire_stuff_here_world`/`consume_one`/`mass_wire_operation` +
//! its two witnesses) is f11-UNIQUE and KEEPS** — the 109/110 apply-side,
//! research/133 §2d; if it moves when this file shrinks, re-point the
//! anchors.txt row in the same commit. Kept, NOT deleted (the pushed-history
//! consumers). See `docs/research/133-f11-unique-folds.md` §4a.
//!
//! The engine (`tr-core::wiring`) resolves the device anchor + the
//! `CheckMech(60)` cooldown and queues the request; this module runs the
//! chest-store lookup, the locked/in-use gates and the item walk — the
//! `Main.chest[]`/`Main.item[]` statics live app-side (`App.world_chests`,
//! `App.items`), the `world_sim/wiring.rs` drain precedent.
//!
//! RNG: the extractinator's `RollExtractinatorDrop` draws `Main.rand` at the
//! `HitWireSingle` site — the wiring tick rides the `UpdateWorldGen` swap
//! (`Main.cs:17802-17805`), but the port's wiring block uses its own seeded
//! stream (the S1-L-6c contract, the `actuation_drops` precedent); the roll
//! and the `Item.NewItem` spawn share `app.wiring.rand_mut()`. The hopper's
//! `TryAddingToEmptySlot` `PlaySound(7)` rides the same stream.

use crate::app::App;
use crate::chest::{self, WorldChest};
use crate::item::{ItemSlot, EMPTY_SLOT};
use crate::registry;
use glam::Vec2;
use tr_core::extract;
use tr_core::TILE_PX;

/// `Wiring.HopperGrabHitboxSize` (`Wiring.cs:75`/`:3429`): `new Vector2(192f)`.
const HOPPER_GRAB_SIZE: f32 = 192.0;

/// `ItemID.Sets.ItemsThatShouldNotBeInInventory` (`ID/ItemID.cs:345`) — the
/// hopper's grab gate (`Wiring.cs:2667`). The same hand-transcribed set as
/// `chest.rs`'s `BANNED_IN_INVENTORY` (the save/load exploit filter).
const BANNED_IN_INVENTORY: &[u16] = &[
    58, 184, 1734, 1735, 1867, 1868, 3453, 3454, 3455, 4143, 5013, 6135, 6136,
];

/// `Wiring.Hopper` (`Wiring.cs:2638-2678`): the chest at the 2×2 anchor
/// (the engine already resolved it + passed `CheckMech(60)`), the locked /
/// in-use gates, then the 400-slot item walk — every active, local-reserved,
/// non-banned drop whose hitbox intersects the 192×192 grab rect centred on
/// `(num*16+16, num2*16+16)` tries to enter the chest. A successful transfer
/// sorts the chest (`ItemSorting.SortInventory(withSync: false, withFeedback:
/// false)`).
pub(crate) fn run_hopper(app: &mut App, num: i32, num2: i32) {
    // `:2651-2659` — `Chest.IsLocked` / `FindChest` / `UsingChest`.
    if chest::is_locked(&app.world, num, num2) {
        return;
    }
    let Some(idx) = chest::find_chest(&app.world_chests, num, num2) else {
        return;
    };
    let open_chest_idx = app.player.as_ref().map_or(-1, |p| p.chest);
    if chest::using_chest(&app.world_chests, idx, open_chest_idx) {
        return;
    }
    // `:2660` — `Utils.CenteredRectangle((num*16+16, num2*16+16), 192×192)`.
    let cx = num as f32 * TILE_PX + 16.0;
    let cy = num2 as f32 * TILE_PX + 16.0;
    let half = HOPPER_GRAB_SIZE / 2.0;
    let (gx, gy) = (cx - half, cy - half);
    // The C# walks `Main.item[0..400]`; the port's drop list is the active
    // subset (the `WorldItem.active` gate folds into membership). A consumed
    // drop is marked air (count 0) and swept after the walk — the C#
    // `TurnToAir` deactivates the slot in place.
    let mut consumed: Vec<usize> = Vec::new();
    let mut flag = false;
    let max_items = app.world_chests[idx].max_items;
    for i in 0..app.items.len() {
        let kind = app.items[i].kind;
        let count = app.items[i].count;
        if count == 0 || BANNED_IN_INVENTORY.contains(&kind) {
            continue;
        }
        // `:2667` — `playerIndexTheItemIsReservedFor == Main.myPlayer`: the
        // port's single-player drops are all local-reserved (the i51 model;
        // `WorldItem.cs:139` netMode 0 → myPlayer). The hitbox is the port's
        // 14×14 drop box (`item::SIZE`, the grab-path precedent).
        let pos = app.items[i].pos;
        if !crate::item::rects_intersect(gx, gy, HOPPER_GRAB_SIZE, HOPPER_GRAB_SIZE, pos.x, pos.y, crate::item::SIZE, crate::item::SIZE) {
            continue;
        }
        if try_to_put_item_in_chest(app, i, idx, max_items) {
            flag = true;
            if app.items[i].count == 0 {
                consumed.push(i);
            }
            // `:2670-2671` — `SyncItem` (MP) + `VisualizeChestTransfer` (the
            // ItemTransfer particle) are unported seams (the particles task).
        }
    }
    // Sweep the consumed drops (reverse order keeps the indices valid).
    for &i in consumed.iter().rev() {
        app.items.swap_remove(i);
    }
    if flag {
        // `:2676` — `ItemSorting.SortInventory(chest, withSync: false,
        // withFeedback: false)`. The damage-ranked layer order reads the
        // local player's multipliers (the chest_sort_click precedent).
        let mult = match app.player.as_ref() {
            Some(p) => [p.melee_damage, p.ranged_damage, p.magic_damage, p.minion_damage],
            None => [1.0; 4],
        };
        let slots = &mut app.world_chests[idx].slots;
        crate::item_sorting::sort(slots, &[], false, mult);
    }
}

/// `Wiring.TryToPutItemInChest` (`Wiring.cs:2680-2701`): coins route through
/// `TryMoveCoinsInChest`; everything else tries the stack merge first, then
/// the empty slot. Returns whether the drop entered (fully or partially).
fn try_to_put_item_in_chest(app: &mut App, item_idx: usize, chest_idx: usize, max_items: usize) -> bool {
    let kind = app.items[item_idx].kind;
    if registry::is_coin(kind) {
        return try_move_coins_in_chest(app, item_idx, chest_idx, max_items);
    }
    // `:2686-2692` — the stack pass.
    for s in 0..max_items {
        if try_adding_to_stack(app, item_idx, chest_idx, s) {
            return true;
        }
    }
    // `:2693-2699` — the empty-slot pass.
    for s in 0..max_items {
        if try_adding_to_empty_slot(app, item_idx, chest_idx, s) {
            return true;
        }
    }
    false
}

/// `Wiring.TryAddingToStack` (`Wiring.cs:2772-2798`): merge the drop into a
/// matching, non-full chest slot. The C# `Item.CanStack` is type AND prefix
/// (`Item.cs:49865-49871`); the port's drop carries its prefix.
fn try_adding_to_stack(app: &mut App, item_idx: usize, chest_idx: usize, slot: usize) -> bool {
    let drop_kind = app.items[item_idx].kind;
    let drop_prefix = app.items[item_idx].prefix;
    let drop_count = app.items[item_idx].count;
    let chest = &app.world_chests[chest_idx];
    let Some(cs) = chest.slots.get(slot) else {
        return false;
    };
    let max = registry::max_stack(cs.kind);
    if cs.count >= max || cs.kind != drop_kind || cs.prefix != drop_prefix {
        return false;
    }
    let mut num = drop_count;
    if drop_count + cs.count > max {
        num = max - cs.count;
    }
    // The C# decrements the drop and increments the slot, then `TurnToAir`
    // when the drop empties (`:2783-2789`).
    app.items[item_idx].count -= num;
    let cs = &mut app.world_chests[chest_idx].slots[slot];
    cs.count += num;
    if app.items[item_idx].count == 0 {
        return true;
    }
    // `:2790-2796` — the `chestItem.type == 0` clone arm is unreachable here
    // (the slot matched a non-zero drop kind); kept out.
    num > 0
}

/// `Wiring.TryAddingToEmptySlot` (`Wiring.cs:2759-2770`): an air slot takes
/// the whole drop (a clone, `newAndShiny = false`), `PlaySound(7)`.
fn try_adding_to_empty_slot(app: &mut App, item_idx: usize, chest_idx: usize, slot: usize) -> bool {
    let chest = &app.world_chests[chest_idx];
    let is_air = chest.slots.get(slot).is_none_or(|s| s.count == 0);
    if !is_air {
        return false;
    }
    // `:2765` — `SoundEngine.PlaySound(7)`: the case-7 pitch draw rides the
    // wiring stream (the `HitWireSingle` site's `Main.rand`).
    let kind = app.items[item_idx].kind;
    let count = app.items[item_idx].count;
    let prefix = app.items[item_idx].prefix;
    app.audio.play_pickup_sound(false, app.wiring.rand_mut());
    let chest = &mut app.world_chests[chest_idx];
    if chest.slots.len() <= slot {
        chest.slots.resize(slot + 1, EMPTY_SLOT);
    }
    chest.slots[slot] = ItemSlot {
        kind,
        name: registry::name(kind),
        count,
        prefix,
    };
    app.items[item_idx].count = 0;
    true
}

/// `Wiring.TryMoveCoinsInChest` (`Wiring.cs:2703-2757`): the chest's coins
/// plus the drop's value re-split into the four denominations. The C#
/// `worldItem.value / 5 * worldItem.stack` uses the coin's sell value
/// (`Item.cs:2222`/`:2243`/`:2264`/`:2285`: 5/500/50000/5000000) — `/5`
/// recovers the copper count (`registry::coin_value`). The reservation walk
/// finds an air-or-coin slot per non-zero denomination (high→low), then the
/// chest's coins clear and the split writes back (capped at the coin
/// maxStack 100).
fn try_move_coins_in_chest(app: &mut App, item_idx: usize, chest_idx: usize, max_items: usize) -> bool {
    let kind = app.items[item_idx].kind;
    let drop_count = app.items[item_idx].count as u64;
    let copper = registry::coin_value(kind) * drop_count;
    // `Utils.CoinsCount` over the chest's slots (`Utils.cs:1019-1046`).
    let mut chest_copper = 0u64;
    for s in 0..max_items {
        if let Some(cs) = app.world_chests[chest_idx].slots.get(s) {
            if registry::is_coin(cs.kind) {
                chest_copper += registry::coin_value(cs.kind) * cs.count as u64;
            }
        }
    }
    let split = coins_split(chest_copper + copper);
    // `:2713-2732` — reserve a slot per non-zero denomination, high→low.
    let mut reserved = [usize::MAX; 4];
    let mut num = 0usize;
    for denom in (0..4).rev() {
        if split[denom] != 0 {
            loop {
                if num >= max_items {
                    return false;
                }
                let is_air_or_coin = app.world_chests[chest_idx]
                    .slots
                    .get(num)
                    .is_none_or(|s| s.count == 0 || registry::is_coin(s.kind));
                if is_air_or_coin {
                    break;
                }
                num += 1;
            }
            reserved[denom] = num;
            num += 1;
        }
    }
    // `:2733-2740` — clear every coin in the chest.
    for s in 0..app.world_chests[chest_idx].slots.len() {
        if registry::is_coin(app.world_chests[chest_idx].slots[s].kind) {
            app.world_chests[chest_idx].slots[s] = EMPTY_SLOT;
        }
    }
    // `:2741-2754` — write the split back (high→low), capped at maxStack.
    let mut remaining = split;
    for denom in (0..4).rev() {
        if remaining[denom] != 0 {
            let coin_kind = registry::coin_id(denom);
            let max = registry::max_stack(coin_kind) as i64;
            let num5 = (remaining[denom] as i64).min(max);
            let slot = reserved[denom];
            let chest = &mut app.world_chests[chest_idx];
            if chest.slots.len() <= slot {
                chest.slots.resize(slot + 1, EMPTY_SLOT);
            }
            chest.slots[slot] = ItemSlot {
                kind: coin_kind,
                name: registry::name(coin_kind),
                count: num5 as u32,
                prefix: 0,
            };
            remaining[denom] -= num5 as i64;
        }
    }
    // `:2755` — the drop turns to air.
    app.items[item_idx].count = 0;
    true
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the copper count split into
/// [copper, silver, gold, platinum] (the C# array is index 0 = copper).
fn coins_split(count: u64) -> [i64; 4] {
    let mut out = [0i64; 4];
    let mut num = 0i64;
    let mut num2 = 1_000_000i64;
    let count = count as i64;
    for denom in (0..4).rev() {
        out[denom] = (count - num) / num2;
        num += out[denom] * num2;
        num2 /= 100;
    }
    out
}

/// `Wiring.Extractinator` (`Wiring.cs:2554-2584`): the 3×3 anchor (the engine
/// resolved it + passed `CheckMech(60)`), `TryFindChestForExtractinator`, the
/// in-use gate, then the REVERSE slot walk — the first non-air slot rolls
/// `RollExtractinatorDrop`; a positive output consumes one input and spawns
/// the drop at the anchor (`Item.NewItem(num*16, num2*16, 32, 32)`), then
/// breaks.
pub(crate) fn run_extractinator(app: &mut App, num: i32, num2: i32, kind: u16) {
    // `:2561` — `TryFindChestForExtractinator` + `UsingChest`.
    let Some(idx) = try_find_chest_for_extractinator(app, num, num2) else {
        return;
    };
    let open_chest_idx = app.player.as_ref().map_or(-1, |p| p.chest);
    if chest::using_chest(&app.world_chests, idx, open_chest_idx) {
        return;
    }
    let max_items = app.world_chests[idx].max_items;
    // `:2567-2583` — the reverse walk.
    for slot in (0..max_items).rev() {
        let Some(cs) = app.world_chests[idx].slots.get(slot).copied() else {
            continue;
        };
        if cs.count == 0 || cs.kind == 0 {
            continue; // `IsAir`
        }
        let mode = extract::extractinator_mode(cs.kind);
        let (item_type, stack) = extract::roll_extractinator_drop(
            mode,
            kind,
            app.world.hard_mode,
            app.wiring.rand_mut(),
        );
        if item_type > 0 {
            // `:2575-2578` — `--item2.stack <= 0` → TurnToAir.
            let cs = &mut app.world_chests[idx].slots[slot];
            cs.count = cs.count.saturating_sub(1);
            if cs.count == 0 {
                *cs = EMPTY_SLOT;
            }
            // `:2579` — `Item.NewItem(EntitySource_Wiring, num*16, num2*16,
            // 32, 32, itemType, stack)`. The spawn rides the wiring stream
            // (the roll's stream, the S1-L-6c contract).
            crate::world_sim::items::spawn_item_drop(
                &mut app.items,
                app.wiring.rand_mut(),
                item_type as u16,
                stack.max(1) as u32,
                num,
                num2,
            );
            break;
        }
    }
}

/// `Wiring.TryFindChestForExtractinator` (`Wiring.cs:2586-2608`): the first
/// non-locked chest in the `[lookupX-2, lookupX+5] × [lookupY-2, lookupY+5]`
/// scan (row-major, x outer). `num=3, num2=3, num3=2` → the inclusive bounds
/// `lookupX-2 ..= lookupX+3+2` / `lookupY-2 ..= lookupY+3+2`.
fn try_find_chest_for_extractinator(app: &App, lookup_x: i32, lookup_y: i32) -> Option<usize> {
    for x in (lookup_x - 2)..=(lookup_x + 3 + 2) {
        for y in (lookup_y - 2)..=(lookup_y + 3 + 2) {
            if !chest::is_locked(&app.world, x, y) {
                if let Some(idx) = chest::find_chest(&app.world_chests, x, y) {
                    return Some(idx);
                }
            }
        }
    }
    None
}

/// `Player.CanDoWireStuffHere` (`Player.cs:46921-46943`), the world-resolved
/// form the mass-wire core predicate takes — the same gate as
/// `world_sim/editing/wiring.rs::can_do_wire_stuff_here` (the wall-350
/// refusal, the wall-87 temple gate while the Golem lives; the
/// dual-dungeons seed arm has no ported seed).
fn can_do_wire_stuff_here_world(world: &tr_core::world::World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.wall == 350 {
        return false;
    }
    if !world.downed_golem && t.wall == 87 {
        return false;
    }
    true
}

/// `Player.ConsumeItem(type)` (`Player.cs:9269-9308`, forward order, no void
/// bag): one item off the FIRST matching carry cell; a drained cell turns
/// to air (`SetDefaults(0)`). The C# scans `inventory[0..58]` (the 50 carry
/// slots + the 4 coin + 4 ammo slots ride indices 50-57); the port keeps
/// them split (`App.inventory` / `App.equip.coins` / `App.equip.ammo`), so
/// the scan chains them in the C#'s index order.
fn consume_one(inventory: &mut [ItemSlot], equip: &mut crate::inv::EquipSlots, kind: u16) -> bool {
    let cells = inventory
        .iter_mut()
        .chain(equip.coins.iter_mut())
        .chain(equip.ammo.iter_mut());
    for slot in cells {
        if slot.count > 0 && slot.kind == kind {
            slot.count -= 1;
            if slot.count == 0 {
                *slot = EMPTY_SLOT;
            }
            return true;
        }
    }
    false
}

/// `Wiring.MassWireOperation` (`Wiring.cs:419-453`), the app half of the
/// W4-f11 residual (4): the 58-slot stock census (530 wire / 849 actuator),
/// the core's L-shaped rectangle walk (`mass_wire_operation_inner`), then
/// the bill. `Main.netMode == 2` sends the two msg-110 bills (`:441-442`) —
/// the port is single-player, so the `ConsumeItem` loops (`:445-452`) run
/// directly (the i51 netMode-0 fold). The cut half's cached `Kill*` drops
/// (`Item.DropCache`, `:3325-3326`) spawn at `master.Center` with zero
/// spread on the CALLER's stage stream (`drop_rng` — the 3625 use path
/// rides `UpdatePlayers`, the Grand Design's kite Kill rides
/// `UpdateProjectiles`; the d13 activation contract's stream note,
/// research/136).
pub(crate) fn mass_wire_operation(
    app: &mut App,
    ps: (i32, i32),
    pe: (i32, i32),
    mode: u8,
    drop_rng: crate::app::RngKey,
) {
    // `:423-433` — the stock census over the 58 carry slots.
    let mut wire_count: i32 = 0;
    let mut actuator_count: i32 = 0;
    for slot in app
        .inventory
        .iter()
        .chain(app.equip.coins.iter())
        .chain(app.equip.ammo.iter())
    {
        if slot.kind == 530 {
            wire_count += slot.count as i32;
        }
        if slot.kind == 849 {
            actuator_count += slot.count as i32;
        }
    }
    let (num, num2) = (wire_count, actuator_count);
    // `:436` — `master.Center` is the drop point, `master.direction == 1`
    // the walk direction.
    let drop_point = app
        .player
        .as_ref()
        .map_or((0.0, 0.0), |p| p.center());
    let dir = app.player.as_ref().is_some_and(|p| p.facing == 1);
    let mut cut_cache = (0i32, 0i32);
    tr_core::wiring::mass_wire_operation_inner(
        &mut app.world,
        ps,
        pe,
        dir,
        mode,
        &mut wire_count,
        &mut actuator_count,
        &can_do_wire_stuff_here_world,
        &mut cut_cache,
    );
    // `:437-452` — the bill: `num - wireCount` wires, `num2 - actuatorCount`
    // actuators, each a `ConsumeItem` call (a no-op when the stock ran dry).
    for _ in 0..(num - wire_count) {
        consume_one(&mut app.inventory, &mut app.equip, 530);
    }
    for _ in 0..(num2 - actuator_count) {
        consume_one(&mut app.inventory, &mut app.equip, 849);
    }
    // `Item.DropCache(reason, dropPoint, Vector2.Zero, 530/849)`
    // (`:3325-3326`): the cached kills drop in `maxStack` chunks
    // (`Item.cs:466-475`). The port's `spawn_item_drop_px` carries the
    // `Item.NewItem` velocity rolls (the zero-spread C# call still draws
    // them) on the CALLER's stage stream (`drop_rng`).
    for (kind, total) in [(530u16, cut_cache.0), (849u16, cut_cache.1)] {
        let mut left = total;
        while left > 0 {
            let chunk = left.min(registry::max_stack(kind) as i32);
            left -= chunk;
            crate::world_sim::items::spawn_item_drop_px(
                &mut app.items,
                app.rngs.stream(drop_rng),
                kind,
                chunk as u32,
                Vec2::new(drop_point.0, drop_point.1),
            );
        }
    }
}

/// A chest-store helper for the witnesses: a fresh chest at the coords with
/// the given slots (the `WorldChest` shape the hopper/extractinator read).
#[cfg(test)]
pub(crate) fn test_chest(x: i32, y: i32, slots: Vec<ItemSlot>) -> WorldChest {
    WorldChest {
        tile: (x, y),
        slots,
        name: String::new(),
        max_items: chest::DEFAULT_MAX_ITEMS as usize,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): 1234567 copper →
    /// [67, 45, 23, 1] (platinum 1, gold 23, silver 45, copper 67).
    #[test]
    fn w4_f11_coins_split_matches_the_csharp_denominations() {
        assert_eq!(coins_split(1_234_567), [67, 45, 23, 1]);
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
        assert_eq!(coins_split(1_000_000), [0, 0, 0, 1]);
    }

    /// The hopper grab rect is 192×192 centred on `(num*16+16, num2*16+16)`
    /// (`Wiring.cs:2660`, `HopperGrabHitboxSize = 192`).
    #[test]
    fn w4_f11_the_hopper_grab_rect_is_192_centred() {
        let num = 10;
        let num2 = 20;
        let cx = num as f32 * TILE_PX + 16.0;
        let cy = num2 as f32 * TILE_PX + 16.0;
        let half = HOPPER_GRAB_SIZE / 2.0;
        assert_eq!(cx - half, 10.0 * 16.0 + 16.0 - 96.0);
        assert_eq!(cy - half, 20.0 * 16.0 + 16.0 - 96.0);
        assert_eq!(HOPPER_GRAB_SIZE, 192.0);
    }

    /// W4-f11 (4) — `ConsumeItem`'s forward scan (`Player.cs:9280-9291`):
    /// the FIRST matching carry cell drains one; a zeroed cell turns to
    /// air. The coin/ammo slots ride after the 50 carry slots.
    #[test]
    fn w4_f11_consume_one_takes_the_first_matching_cell() {
        let mut inv = [EMPTY_SLOT; crate::item::INV];
        let mut equip = crate::inv::EquipSlots::empty();
        inv[3] = ItemSlot { kind: 530, name: "", count: 2, prefix: 0 };
        inv[7] = ItemSlot { kind: 530, name: "", count: 5, prefix: 0 };
        assert!(consume_one(&mut inv, &mut equip, 530));
        assert_eq!(inv[3].count, 1, "the first cell drained one");
        assert_eq!(inv[7].count, 5, "the later cell untouched");
        assert!(consume_one(&mut inv, &mut equip, 530));
        assert_eq!(inv[3].kind, 0, "the drained cell turned to air");
        assert!(consume_one(&mut inv, &mut equip, 530));
        assert_eq!(inv[7].count, 4, "the scan moved to the next cell");
        // An empty stock refuses.
        let mut empty = [EMPTY_SLOT; crate::item::INV];
        assert!(!consume_one(&mut empty, &mut equip, 530));
    }

    /// W4-f11 (4) — the mass operation's bill (`Wiring.cs:437-452`): the
    /// placed channels consume the wire stock; the single-player fold runs
    /// the `ConsumeItem` loops directly (no msg-110 round trip).
    #[test]
    fn w4_f11_the_mass_operation_bills_the_placed_channels() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = tr_core::world::World::new("t".into(), 40, 40);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // A stone row at y=19 for the walk.
        for x in 5..=15 {
            let i = app.world.index(x, 19);
            app.world.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        app.inventory[0] = ItemSlot { kind: 530, name: "", count: 10, prefix: 0 };
        let mode = tr_core::smart_cursor::multi_tool_mode::RED
            | tr_core::smart_cursor::multi_tool_mode::BLUE;
        mass_wire_operation(&mut app, (10, 19), (10, 19), mode, crate::app::RngKey::UpdatePlayers);
        assert!(app.world.wires.get(10, 19, tr_core::wiring::WireColor::Red));
        assert!(app.world.wires.get(10, 19, tr_core::wiring::WireColor::Blue));
        assert_eq!(app.inventory[0].count, 8, "two channels billed two wires");
    }

    /// W4-f11 — the d13 activation contract's composition (research/136):
    /// the Grand Design's kite Kill passes the RAW ToolMode (the Actuator
    /// bit NOT stripped — `Wiring.cs:3241`; only the 3625 click arm masks,
    /// `Player.cs:46908`) and the `UpdateProjectiles` stage stream (the
    /// Kill runs inside that swap, `Main.cs:17775`). The apply places the
    /// actuator alongside the wire and bills both stocks. WRITTEN NOT RUN.
    #[test]
    fn w4_f11_the_kite_apply_keeps_the_actuator_bit_and_the_proj_stream() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = tr_core::world::World::new("t".into(), 40, 40);
        app.player = Some(crate::player::Player::spawn(&app.world));
        for x in 5..=15 {
            let i = app.world.index(x, 19);
            app.world.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        app.inventory[0] = ItemSlot { kind: 530, name: "", count: 10, prefix: 0 };
        app.inventory[1] = ItemSlot { kind: 849, name: "", count: 10, prefix: 0 };
        // The RAW mode: Red + Actuator (the wrench arm would strip it).
        let mode = tr_core::smart_cursor::multi_tool_mode::RED
            | tr_core::smart_cursor::multi_tool_mode::ACTUATOR;
        mass_wire_operation(&mut app, (10, 19), (10, 19), mode, crate::app::RngKey::UpdateProjectiles);
        assert!(app.world.wires.get(10, 19, tr_core::wiring::WireColor::Red));
        let t = app.world.tile(10, 19).unwrap();
        assert!(t.flags & tr_core::tile::FLAG_ACTUATOR != 0, "the actuator landed unmasked");
        assert_eq!(app.inventory[0].count, 9, "one wire billed");
        assert_eq!(app.inventory[1].count, 9, "one actuator billed");
    }
}
