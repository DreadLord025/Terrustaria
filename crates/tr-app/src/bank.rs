//! The player-bounded banks — `Player.bank/bank2/bank3/bank4`
//! (`Player.cs:1580-1591`, `Chest.CreateBank(-2..-5)`, 40 slots each). The
//! banks are plain `Chest` objects in vanilla (`bankChest = true`,
//! `Chest.cs:191-199`), never in `Main.chest`, always unlocked
//! (`IsLockedOrInUse` → false, `Chest.cs:178-189`). The port keeps them as
//! slot arrays on the `Player`, resolved from `Player.chest`'s
//! -2..-5 index space (`Player.cs:33492-33502`).

use crate::item::{ItemSlot, EMPTY_SLOT};

/// `Chest.CreateBank`'s `maxItems = 40` (`Chest.cs:196`).
pub(crate) const BANK_SLOTS: usize = 40;

/// `Player.chest`'s bank index space (`Player.cs:33492-33502`): the port's
/// `Player.chest` carries the same -1 (closed) / -2..-5 (banks) values.
pub(crate) const CHEST_CLOSED: i32 = -1;

/// One bank's slot array (`Chest.item[]`'s port).
pub(crate) type BankStore = [ItemSlot; BANK_SLOTS];

/// The bank's slots for the open `chest` index; `None` for -1 and the world
/// indices (the world chests are M3-CHEST-UI's).
pub(crate) fn bank_for_chest(chest: i32) -> Option<usize> {
    match chest {
        -5..=-2 => Some((-chest - 2) as usize),
        _ => None,
    }
}

/// `Player.CanVoidVaultAccept` (`Player.cs:39425-39436`): the quest items
/// (`item.questItem`, the generated table carries the flag) and the
/// void-lens kind 3822 refuse.
pub(crate) fn can_void_vault_accept(kind: u16) -> bool {
    if crate::registry::get(kind).is_some_and(|d| d.quest_item) {
        return false;
    }
    kind != 3822
}

/// `Player.GetItem_VoidVault` (`Player.cs:39402-39423`): fill the occupied
/// stackable slots first, then the empty ones; returns whether the whole
/// stack fit. The merge rule is `Item.CanStack` (same kind AND same prefix —
/// the port's `slot_for_pickup` shape, applied to the vault's 40 slots).
/// Every accepted slot chimes `IsACoin ? 38 : 7` BEFORE the fill
/// (`GetItem_FillIntoOccupiedSlot_VoidBag` `:39444`,
/// `GetItem_FillEmptyInventorySlot_VoidBag` `:39534`) — one case-7/38 draw
/// per slot, in slot order (S1-P-i43, the Lead's faithful-cue decision;
/// the callers pass their emission stream — the click handlers the bare
/// `Main.rand` stand-in, the GrabItems pickup the `UpdatePlayers` stream).
/// `is_coin` is the LOOTED item's `IsACoin` (the C# reads `newItem`, not the
/// slot). The `DoCoins` roll-over inside the fills (`:39454`/:39544) is the
/// recorded unported deviation.
pub(crate) fn void_vault_take(
    vault: &mut BankStore,
    kind: u16,
    prefix: u16,
    count: u32,
    max_stack: u32,
    is_coin: bool,
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) -> u32 {
    let mut left = count;
    // The occupied stackable slots first (:39408-39413). The C# gate also
    // reads `stack < maxStack`; the `add == 0` skip below is equivalent
    // (a full slot accepts nothing and plays nothing).
    for slot in vault.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.count > 0 && slot.kind == kind && slot.prefix == prefix {
            let add = left.min(max_stack.saturating_sub(slot.count));
            if add == 0 {
                continue;
            }
            audio.play_pickup_sound(is_coin, rng); // `:39444`
            slot.count += add;
            left -= add;
            if left == 0 {
                // The C# occupied fill RETURNS on the whole fit
                // (`:39448-39457`); the empty pass never runs.
                return count;
            }
        }
    }
    // Then the empty slots (:39414-39420). One empty slot takes the WHOLE
    // remainder (`inv[i] = returnItem`, `:39539`) and returns — never a
    // capped multi-slot split (S1-P-i43 fix: the old port spread the
    // surplus across empty slots with one chime total).
    for slot in vault.iter_mut() {
        if slot.count == 0 {
            audio.play_pickup_sound(is_coin, rng); // `:39534`
            *slot = ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count: left,
                prefix,
            };
            return count;
        }
    }
    count - left
}

// --- The transfer buttons (`UI/ChestUI.cs`, M3-STORE-2) ---------------------

/// The coin types (`ItemID.CopperCoin..PlatinumCoin`).
const COIN_TYPES: [u16; 4] = [71, 72, 73, 74];

/// `Item.CanStack` (`Item.cs:49865-49871`): two slots stack only when the
/// type AND the prefix match — the merge gate shared by every transfer
/// (the world-chest paths call it from `chest.rs`/`clicks.rs` too).
pub(crate) fn can_stack(a: &ItemSlot, b: &ItemSlot) -> bool {
    a.kind == b.kind && a.prefix == b.prefix
}

/// `ChestUI.LootAll` (:393-473): the container drains into the player. The
/// coins route into the coin rows (`Player.GetItem`'s isACoin arm,
/// `Player.cs:39330` — the occupied coin slots first, then the empty);
/// everything else rides the pickup stack (hotbar first, then the rows).
/// The overflow STAYS in the container (`GetItem` returns the un-moved
/// remainder) — unless `vault` is `Some`, the world-chest variant's
/// `CanGoIntoVoidVault` delta (`LootAllFromChest`, `GetItemSettings.cs:55-56`;
/// banks ride the default `LootAllFromBank`, :55, and pass `None`): the
/// overflow then fills the void vault as `GetItem`'s per-item last resort
/// (`Player.cs:39395-39398`) via the landed `void_vault_take`.
pub(crate) fn loot_all(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    ammo: &mut [ItemSlot; 4],
    mut vault: Option<&mut BankStore>,
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) {
    // `Player.GetItem`'s fill helpers, shared by every pass below. The
    // occupied merge (`GetItem_FillIntoOccupiedSlot`, `Player.cs:39485-39524`)
    // returns whether the WHOLE remainder fit (the C# `return true` at
    // `:39507`); a partial fill leaves the surplus in `item` and the walk
    // continues. The chime is `IsACoin ? 38 : 7` (`:39491`), one case-7/38
    // draw per accepted slot (S1-P-i43 — the Lead's faithful-cue decision).
    fn merge_into(
        s: &mut ItemSlot,
        item: &mut ItemSlot,
        is_coin: bool,
        audio: &mut crate::audio::Audio,
        rng: &mut tr_core::UnifiedRandom,
    ) -> bool {
        let max = crate::registry::max_stack(item.kind);
        if s.count > 0 && s.count < max && can_stack(s, item) {
            audio.play_pickup_sound(is_coin, rng); // `:39491`
            if item.count + s.count <= max {
                s.count += item.count;
                item.count = 0;
                return true;
            }
            let num = max - s.count;
            item.count -= num;
            s.count = max;
        }
        false
    }
    // The empty fill (`GetItem_FillEmptyInventorySlot`, `:39551-39578`): one
    // empty slot takes the whole remainder, chime `:39559`.
    fn fill_empty(
        s: &mut ItemSlot,
        item: &mut ItemSlot,
        is_coin: bool,
        audio: &mut crate::audio::Audio,
        rng: &mut tr_core::UnifiedRandom,
    ) -> bool {
        if s.count == 0 && item.count > 0 {
            audio.play_pickup_sound(is_coin, rng); // `:39559`
            *s = *item;
            item.count = 0;
            true
        } else {
            false
        }
    }
    for slot in container.iter_mut() {
        if slot.kind == 0 || slot.count == 0 {
            continue;
        }
        let mut item = std::mem::replace(slot, EMPTY_SLOT);
        let is_coin = COIN_TYPES.contains(&item.kind);
        // `Player.GetItem` (`Player.cs:39330-39400`), the C# pass order:
        // the FillAmmo pre-pass (`:39345-39352`), the occupied walk
        // (`:39353-39364`, coins start at pInv 50 via `num2 = -4`), the
        // hotbar-first usable empty arm (`:39365-39374`), the reverse empty
        // walk (`:39387-39394`, coins scan pInv 53..0), then the void vault
        // (`:39395-39398`). S1-P-i43: the coin arm rides the SAME GetItem
        // shape as `quick_transfer_from_slot`.
        // The FillAmmo pre-pass (`:39264-39327`): ammo kinds merge into the
        // occupied ammo rows first (`FitsAmmoSlot`, `Item.cs:1369-1376` —
        // the canonical `item::fits_ammo_slot` seam, the notAmmo family
        // 27/71-74/169/370/408/1246/353 only fits 353).
        if crate::item::fits_ammo_slot(item.kind) {
            for a in ammo.iter_mut() {
                if item.count == 0 {
                    break;
                }
                if merge_into(a, &mut item, false, audio, rng) {
                    break;
                }
            }
            if item.count == 0 {
                continue;
            }
        }
        // The occupied walk: coins merge into the coin rows (pInv 50-53)
        // first, then the grid 0-49; non-coins walk the grid only.
        // S1-P-i50 — the coin fills carry `DoCoins(i)` IMMEDIATELY after
        // every accepted fill (`Player.cs:39503` full-fit / `:39519`
        // cap-fill): a conversion merges-and-airs a coin row the later
        // empty pass then refills, so the deferred sweep changed slot
        // placement. The conversion is RNG-free — the cue order is
        // unchanged (S1-P-i43's per-slot chime stays at `:39491`).
        if is_coin {
            let coin_cap = crate::registry::max_stack(item.kind);
            let mut slots = crate::item::carry_slots_mut(inventory, coins, None);
            let n = slots.len(); // INV + COINS = 54
            // C# i = -4..50 → port indices 50..54 then 0..50.
            for i in (crate::item::INV..n).chain(0..crate::item::INV) {
                if item.count == 0 {
                    break;
                }
                let s = &mut *slots[i];
                if s.count > 0 && s.count < coin_cap && can_stack(s, &item) {
                    audio.play_pickup_sound(true, rng); // `:39491`
                    if item.count + s.count <= coin_cap {
                        s.count += item.count;
                        item.count = 0;
                        crate::item::do_coins(&mut slots, i); // `:39503`
                        break;
                    }
                    let num = coin_cap - s.count;
                    item.count -= num;
                    s.count = coin_cap;
                    crate::item::do_coins(&mut slots, i); // `:39519`
                }
            }
        }
        if item.count > 0 && !is_coin {
            for s in inventory.iter_mut() {
                if merge_into(s, &mut item, false, audio, rng) {
                    break;
                }
            }
        }
        // The hotbar-first empty arm for usable non-coin kinds
        // (`!isACoin && useStyle != 0`, `:39365-39374`).
        if item.count > 0 && !is_coin && crate::item::use_style(item.kind) != 0 {
            for s in inventory[..crate::item::HOTBAR].iter_mut() {
                if fill_empty(s, &mut item, false, audio, rng) {
                    break;
                }
            }
        }
        // The reverse empty walk (`:39387-39394`): coins scan the coin rows
        // descending first (`num = 54`), then the grid 49..0; non-coins the
        // grid only. (The favorited forward arm `:39375-39384` is unreachable
        // — chest/bank items cannot be favorited, `can_favorite_at`.)
        // S1-P-i50 — every coin empty fill carries `DoCoins(i)` immediately
        // (`Player.cs:39569`), the conversion RNG-free.
        if item.count > 0 && is_coin {
            let mut slots = crate::item::carry_slots_mut(inventory, coins, None);
            let n = slots.len(); // INV + COINS = 54
            // C# i = 53..0 descending — one pass covers coin rows then grid.
            for i in (0..n).rev() {
                if item.count == 0 {
                    break;
                }
                if slots[i].count == 0 {
                    audio.play_pickup_sound(true, rng); // `:39559`
                    *slots[i] = item;
                    item.count = 0;
                    crate::item::do_coins(&mut slots, i); // `:39569`
                }
            }
        }
        if item.count > 0 {
            for s in inventory.iter_mut().rev() {
                if fill_empty(s, &mut item, is_coin, audio, rng) {
                    break;
                }
            }
        }
        // The void vault (`:39395-39398`): `CanGoIntoVoidVault &&
        // IsVoidVaultEnabled && CanVoidVaultAccept` — the caller passes the
        // vault only when enabled; `CanVoidVaultAccept` gates the kinds
        // (`:39425-39433`). The fills chime per accepted slot inside the
        // helper (`:39444`/`:39534`, S1-P-i43).
        if item.count > 0 {
            if let Some(vault) = vault.as_deref_mut() {
                if can_void_vault_accept(item.kind) {
                    let taken = void_vault_take(
                        vault,
                        item.kind,
                        item.prefix,
                        item.count,
                        crate::registry::max_stack(item.kind),
                        is_coin,
                        audio,
                        rng,
                    );
                    item.count -= taken;
                }
            }
            if item.count > 0 {
                // The remainder rides back home (:400-414's returned item).
                *slot = item;
            }
        }
    }
}

/// `ChestUI.MoveCoins` (:997-1185) compacted: the container's coin stacks
/// and the player's (un-favorited) coin stacks consolidate into the
/// container — the denomination carry `>= 100 -> next tier` included. The
/// pull only runs when the CONTAINER held a coin stack (`flag2`, :1044 —
/// `if (!flag2) return 0L;` :1048-1051 makes a coin-less container a total
/// no-op). The port's coins live as stacks inside the 4 coin rows, so the
/// carry walks the tallies directly; the leftovers return to the player's
/// rows (the overflow guard, :1090-1208).
pub(crate) fn move_coins(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    is_favorite: &dyn Fn(usize) -> bool,
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) {
    // The vanilla consolidates INTO the container (`MoveCoins(pInv, cInv)`,
    // :997-1185): pass 1 sucks the container's coin stacks into the
    // tallies (:1011-1046) and sets `flag2` only then (:1044), pass 2 pulls
    // the player's un-favorited coin stacks — the whole 0..58 range, the
    // grid rows included (:1052-1080's `j != 58` skip); the carry rolls
    // 100-per-tier (:1082-1089); the container's free slots refill
    // largest-first (:1115-1147); whatever does not fit returns to the
    // player's coin rows (:1090-1208's tail). The chime (`:1197-1200`) is
    // `if (flag) PlaySound(7)` — `flag` sets when a player coin stack is
    // pulled (:1075) and clears when overflow returns (:1189); one case-7
    // draw (S1-P-i43, the Lead's faithful-cue decision).
    let mut tallies = [0u64; 4];
    let mut container_had_coins = false; // `:1044`'s `flag2`
    let mut pulled_player_coins = false; // `:1075`'s `flag`
    for slot in container.iter_mut() {
        if slot.count > 0 {
            if let Some(d) = COIN_TYPES.iter().position(|t| *t == slot.kind) {
                tallies[d] += slot.count as u64;
                *slot = EMPTY_SLOT;
                container_had_coins = true;
            }
        }
    }
    // `ChestUI.cs:1048-1051` — `if (!flag2) return 0L;`: a coin-less
    // container is a total no-op, the player's coins stay put.
    if !container_had_coins {
        return;
    }
    let coin_cap = |d: usize| crate::registry::max_stack(COIN_TYPES[d]);
    for i in 0..50 {
        if inventory[i].count > 0 && !is_favorite(i) {
            if let Some(d) = COIN_TYPES
                .iter()
                .position(|t| *t == inventory[i].kind)
            {
                tallies[d] += inventory[i].count as u64;
                inventory[i] = EMPTY_SLOT;
                pulled_player_coins = true; // `:1075`
            }
        }
    }
    // The coin rows are pInv 50-53 (`ChestUI.cs:1052-1054` reads each coin
    // item's own flag) — the grid-index space (0-49) must not alias them.
    // The C# scan also covers the AMMO rows 54-57 (the single 0..58 walk,
    // `j != 58`); the port omits them — a coin stack can never legitimately
    // occupy an ammo row (`fits_ammo_slot` refuses the notAmmo coin family,
    // `Item.cs:1369-1376`), so the omission is behaviour-equal (S1-P-i43
    // census note).
    for i in 0..4 {
        if coins[i].count > 0 && !is_favorite(50 + i) {
            if let Some(d) = COIN_TYPES.iter().position(|t| *t == coins[i].kind) {
                tallies[d] += coins[i].count as u64;
                coins[i] = EMPTY_SLOT;
                pulled_player_coins = true; // `:1075`
            }
        }
    }
    // The carry (:1082-1089): 100 of a tier rolls into the next.
    for k in 0..3 {
        tallies[k + 1] += tallies[k] / 100;
        tallies[k] %= 100;
    }
    // The container's free slots refill largest-first (:1115-1147).
    for d in (0..4).rev() {
        while tallies[d] > 0 {
            let Some(free) = container
                .iter()
                .position(|sl| sl.kind == 0 || sl.count == 0)
            else {
                break;
            };
            let put = tallies[d].min(coin_cap(d) as u64) as u32;
            container[free] = ItemSlot {
                kind: COIN_TYPES[d],
                name: crate::registry::name(COIN_TYPES[d]),
                count: put,
                prefix: 0,
            };
            tallies[d] -= put as u64;
        }
    }
    // The overflow returns to the player's coin rows (:1090-1208's tail).
    for d in 0..4 {
        while tallies[d] > 0 {
            let row = coins
                .iter()
                .position(|sl| sl.count == 0 && !is_favorite(50 + d))
                .unwrap_or(d.min(3));
            if coins[row].count > 0 && coins[row].kind != COIN_TYPES[d] {
                break;
            }
            let cap = coin_cap(d).saturating_sub(coins[row].count) as u64;
            let put = tallies[d].min(cap) as u32;
            if put == 0 {
                break;
            }
            coins[row] = ItemSlot {
                kind: COIN_TYPES[d],
                name: crate::registry::name(COIN_TYPES[d]),
                count: coins[row].count + put,
                prefix: 0,
            };
            tallies[d] -= put as u64;
            pulled_player_coins = false; // `:1189` — overflow returned
        }
    }
    if pulled_player_coins {
        audio.play_pickup_sound(false, rng); // `:1197-1200`
    }
}

/// `ChestUI.DepositAll` (:540-659) after MoveCoins: the storage rows scan
/// 49..=10 descending (the hotbar stays), the favorited rows stay; the
/// stack-merge phase (maxStack > 1, type+prefix equality, :516-524) then the
/// empty-slot phase (the first free slot per source, :579-655). The merge
/// chime (`:490`/`:525`) and the empty-placement chime (`:587`/`:605`/`:619`/
/// `:633`/`:647`) are `PlaySound(7)` — one case-7 pitch draw each (S1-P-i43,
/// the Lead's faithful-cue decision).
pub(crate) fn deposit_all(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    is_favorite: &dyn Fn(usize) -> bool,
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) {
    for num in (10..50).rev() {
        if inventory[num].count == 0 || inventory[num].kind == 0 || is_favorite(num) {
            continue;
        }
        let max = crate::registry::max_stack(inventory[num].kind);
        if max > 1 {
            // The merge phase (:567-578).
            for cs in container.iter_mut() {
                if inventory[num].count == 0 {
                    break;
                }
                if cs.count > 0 && cs.count < max && can_stack(cs, &inventory[num]) {
                    let moved = inventory[num].count.min(max - cs.count);
                    cs.count += moved;
                    inventory[num].count -= moved;
                    audio.play_pickup_sound(false, rng); // `:490`/`:525`
                }
            }
        }
        if inventory[num].count > 0 {
            // The empty-slot phase (:579-655): the first free slot.
            if let Some(free) = container.iter().position(|sl| sl.kind == 0 || sl.count == 0) {
                audio.play_pickup_sound(false, rng); // `:587` et al.
                container[free] = std::mem::replace(&mut inventory[num], EMPTY_SLOT);
            }
        }
    }
}

/// `ChestUI.QuickStack` (:661-825) after its MoveCoins gate: the container's
/// non-coin stacks are the merge targets (the coin exclusion 71-74 rides the
/// TARGET side, :703-714), the source scan = the storage rows 10..50 (+ the
/// coin rows when a bank is open, :715-725), the same-type filter (:726-732),
/// the CanStack merge (:733-760) and the empty-slot phase (:772-812 — for
/// each empty container slot the first queued source moves whole into it and
/// every subsequent CanStack source stacks into it until maxStack). The
/// MoveCoins pre-pass is the DISPATCH layer's (M3-CHEST-INT decision): the
/// C# runs it for the bank kinds only (:670-685 — the world chest gets
/// none), so the bank call site does `move_coins` + `quick_stack` and the
/// world call site calls this alone.
pub(crate) fn quick_stack(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    is_favorite: &dyn Fn(usize) -> bool,
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) {
    // The targets: the container's non-coin occupied stacks.
    let mut types: Vec<u16> = Vec::new();
    for cs in container.iter() {
        if cs.count > 0 && cs.kind > 0 && !COIN_TYPES.contains(&cs.kind) {
            types.push(cs.kind);
        }
    }
    // The merge phase (`ChestUI.cs:733-760`): TARGET-outer (the container's
    // non-coin occupied stacks, `list2`) × SOURCE-inner (the player's
    // same-type un-favorited rows, `dictionary` — storage rows 10..50 then
    // the coin rows 50..53 for a bank, `:715-725`). The C# breaks the source
    // walk when the target is FULL (`num5 == 0`, `:742-745`); the chime is
    // `PlaySound(7)` per accepted merge (`:750`), one case-7 draw each
    // (S1-P-i43, the Lead's faithful-cue decision). The source favorite gate
    // (`:728`) covers the coin rows too.
    // The source queue (`dictionary`, `:726-732`): pInv 10..50 (the grid
    // storage rows) then 50..54 (the coin rows) — `num = 50`, `+4` for a bank
    // (`player.chest <= -2`, `:717-720`); the port always passes the coin rows
    // (the world chest's coin sources are inert — coin types never join
    // `types`, the container-side exclusion `:705`). The queue holds only
    // non-empty sources; the C# PRUNES the drained ones after the merge
    // phase (`list4`, `:761-770`) and the empty phase re-checks
    // `stack == 0` per source (`:778`).
    let mut sources: Vec<usize> = (10..50)
        .chain(50..54)
        .filter(|&j| {
            let sl = if j < 50 { &inventory[j] } else { &coins[j - 50] };
            sl.count > 0 && types.contains(&sl.kind) && !is_favorite(j)
        })
        .collect();
    for k in 0..container.len() {
        // `list2`'s members: the non-coin occupied container stacks (`:705`).
        if container[k].count == 0 || container[k].kind == 0 || COIN_TYPES.contains(&container[k].kind)
        {
            continue;
        }
        for &j in &sources {
            let src = if j < 50 { &mut inventory[j] } else { &mut coins[j - 50] };
            if !can_stack(src, &container[k]) {
                continue; // `:738`
            }
            let cap = crate::registry::max_stack(container[k].kind);
            let room = cap - container[k].count;
            if room == 0 {
                break; // `:742-745` — the target is full
            }
            let moved = src.count.min(room);
            audio.play_pickup_sound(false, rng); // `:750`
            container[k].count += moved;
            src.count -= moved;
            if src.count == 0 {
                *src = EMPTY_SLOT; // `:753-755`
            }
        }
    }
    // `list4` (`:761-770`): the drained sources leave the queue BEFORE the
    // empty phase (S1-P-i43: the explicit prune mirrors the C# list surgery;
    // the per-source `stack == 0` re-check below is `:778`'s own — together
    // they keep a drained source out of the `flag` first-placement).
    sources.retain(|&j| {
        let sl = if j < 50 { &inventory[j] } else { &coins[j - 50] };
        sl.count > 0
    });
    // The empty-slot phase (:772-812): for each initially-empty container
    // slot in order (`list3`), the first queued source moves WHOLE into it
    // (:783-788) and every subsequent CanStack source stacks into it until
    // maxStack (:789-808) — all the same-type leftovers consolidate into the
    // chest. A source skipped for its prefix stays queued for the next empty
    // slot (`flag` resets per slot, :775). The chime is `PlaySound(7)` per
    // placement (`:782`), before the first/stacked branch. The sources ride
    // the PRUNED `dictionary` as ONE queue in pInv order (the grid rows then
    // the coin rows — the dictionary's insertion order; S1-P-i43 collapsed
    // the old two separate walks into the C#'s single `foreach`, behaviour
    // preserved: a coin-row source can be an empty slot's first placement
    // when the grid drained).
    for free in 0..container.len() {
        if container[free].kind != 0 && container[free].count != 0 {
            continue; // `list3`'s members — the initially-empty slots
        }
        let mut first = true; // `:775`'s `flag`
        for &j in &sources {
            let src = if j < 50 { &mut inventory[j] } else { &mut coins[j - 50] };
            if src.count == 0 {
                continue; // `:778` — drained inside this phase
            }
            if !first && !can_stack(src, &container[free]) {
                continue; // `:778` — after the first placement, same type+prefix only
            }
            audio.play_pickup_sound(false, rng); // `:782`
            if first {
                // `:783-788` — the first source moves whole.
                std::mem::swap(&mut container[free], src);
                first = false;
            } else {
                // `:789-808` — the capped stack into the slot.
                let cap = crate::registry::max_stack(container[free].kind)
                    .saturating_sub(container[free].count);
                if cap == 0 {
                    break; // `:793-795` — the slot is full
                }
                let moved = src.count.min(cap);
                container[free].count += moved;
                src.count -= moved;
                if src.count == 0 {
                    *src = EMPTY_SLOT;
                }
            }
        }
    }
}

/// `Item.OnlyNeedOneInInventory` (`Item.cs:1176-1192`): the override set
/// (`ItemID.Sets.OnlyNeedOneInInventoryOverride`, `ItemID.cs:1387-1444`) —
/// 22 `true` rows (the mana stars 1522-1527, the boss/event summon items
/// 3643/3103/3104/856/1299/4346/5391/4767/5453/5309/5454/5323/5455/6190/6195,
/// the shadow key 329) and 6 `false` rows (the buckets 205-207/1128, the
/// Guide Voodoo Doll 267, the Actuator 849); else `!consumable &&
/// (useStyle != 0 || CanBeEquipped())`. `CanBeEquipped`
/// (`Item.cs:1158-1174`) reads the armor/accessory/mount slots; the
/// `shoot`-hook and pet-buff arms have no port fields (`ItemDef` carries no
/// `shoot` id / vanity-light pet sets) — a cited inert slice. The Restock
/// demand gate's reader (`ChestUI.cs:908`).
pub(crate) fn only_need_one_in_inventory(kind: u16) -> bool {
    if matches!(
        kind,
        1522 | 1523
            | 1524
            | 1525
            | 1526
            | 1527
            | 3643
            | 3103
            | 3104
            | 856
            | 1299
            | 329
            | 4346
            | 5391
            | 4767
            | 5453
            | 5309
            | 5454
            | 5323
            | 5455
            | 6190
            | 6195
    ) {
        return true;
    }
    if matches!(kind, 205 | 206 | 207 | 1128 | 267 | 849) {
        return false; // the `false` override rows
    }
    let Some(d) = crate::registry::get(kind) else {
        return false;
    };
    if d.consumable {
        return false;
    }
    if d.use_style != 0 {
        return true;
    }
    d.head_slot >= 0 || d.body_slot >= 0 || d.leg_slot >= 0 || d.accessory || d.mount_type >= 0
}

/// `ChestUI.Restock` (:871-995): the container -> the player. The demand =
/// the player's stackable types, walked pInv 57..=0 (:899-917 — the ammo
/// rows 54-57 included, the coin rows 50-53 and coin types 71-74 excluded
/// throughout); the supply = the container's UNPREFIXED stacks of demanded
/// types (:919-924's `prefix != 0` + `hashSet.Contains` gate, which governs
/// the whole loop); the capped fill (:925-965) and the ammo-into-empty
/// bonus (:966-989 — a demanded ammo stack that did NOT fully empty into
/// the partials swaps into the first free empty slot). Returns whether
/// anything moved (the single PlaySound(7), :991-994).
pub(crate) fn restock(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    ammo: &mut [ItemSlot; 4],
    is_favorite: &dyn Fn(usize) -> bool,
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) -> bool {
    let mut moved_any = false;
    let mut demand_types: Vec<u16> = Vec::new();
    // The row references ride the C#'s pInv index space: grid 0-49, ammo
    // 54-57 (`ChestUI.cs:899`'s single `inventory` walk maps onto the
    // port's split arrays).
    let mut partial: Vec<usize> = Vec::new();
    let mut empty_slots: Vec<usize> = Vec::new();
    // `ChestUI.cs:899-917` — the demand walk 57..=0: coin types are
    // excluded throughout (:902's type gate — they register neither demand
    // nor free slots), empty slots join `list2`, stackable types join
    // `hashSet` and partials join `list`. The demand gate (:908) is
    // `maxStack > 1 && (!favorited || !OnlyNeedOneInInventory())` — a
    // favorited one-needed row (a summon staff, an accessory) registers no
    // demand (S1-P-i43).
    let mut scan_row = |sl: &ItemSlot,
                        p_inv: usize,
                        demand_types: &mut Vec<u16>,
                        partial: &mut Vec<usize>,
                        empty_slots: &mut Vec<usize>| {
        if COIN_TYPES.contains(&sl.kind) {
            return; // `:902` — `(type < 71 || type > 74)`
        }
        if sl.count == 0 || sl.kind == 0 {
            empty_slots.push(p_inv); // `:904-907`'s `list2`
        } else if crate::registry::max_stack(sl.kind) > 1
            && (!is_favorite(p_inv) || !only_need_one_in_inventory(sl.kind))
        {
            demand_types.push(sl.kind); // `:910`'s `hashSet`
            if sl.count < crate::registry::max_stack(sl.kind) {
                partial.push(p_inv); // `:911-914`'s `list`
            }
        }
    };
    for i in (0..4).rev() {
        scan_row(
            &ammo[i],
            54 + i,
            &mut demand_types,
            &mut partial,
            &mut empty_slots,
        );
    }
    for i in (0..50).rev() {
        scan_row(
            &inventory[i],
            i,
            &mut demand_types,
            &mut partial,
            &mut empty_slots,
        );
    }
    // The supply scan (:919-924): the `hashSet.Contains` gate governs the
    // whole loop — unprefixed demanded types only.
    for i in 0..container.len() {
        let cap_kind = container[i].kind;
        if container[i].count == 0 || container[i].prefix != 0 || !demand_types.contains(&cap_kind)
        {
            continue;
        }
        // The partial fill (:925-965): the capped fill into the demand-side
        // partials; `emptied` is the C#'s `flag2` (:955-958). The C# removes
        // a partial from `list` once it FILLS to maxStack (`:946-954`'s
        // `list.RemoveAt(j); j--`) — S1-P-i43 mirrors that with an index walk.
        let mut emptied = false;
        let mut j = 0;
        while j < partial.len() {
            let p = partial[j];
            // `ItemSlot.PickItemMovementAction` (:934, `ItemSlot.cs:1484-
            // 1516`): context 0 (grid) always accepts; context 2 (the ammo
            // rows, `num2 >= 50`) accepts only `FitsAmmoSlot` kinds — a
            // non-ammo supply never pours into an ammo-row partial
            // (S1-P-i43).
            if p >= 54 && !crate::item::fits_ammo_slot(container[i].kind) {
                j += 1;
                continue;
            }
            let target = if p >= 54 {
                &mut ammo[p - 54]
            } else {
                &mut inventory[p]
            };
            if !can_stack(target, &container[i]) {
                j += 1;
                continue;
            }
            let cap = crate::registry::max_stack(target.kind) - target.count;
            let moved = container[i].count.min(cap);
            target.count += moved;
            container[i].count -= moved;
            moved_any = moved_any || moved > 0;
            let target_full = target.count == crate::registry::max_stack(target.kind);
            if container[i].count == 0 {
                container[i] = EMPTY_SLOT;
                emptied = true;
                if target_full {
                    partial.remove(j); // `:952-953` — the filled partial leaves
                }
                break;
            }
            if target_full {
                partial.remove(j); // `:952-953` — `RemoveAt(j); j--` (no advance)
            } else {
                j += 1;
            }
        }
        // The ammo-into-empty bonus (:966-989): a demanded ammo stack that
        // did NOT fully empty into the partials, with an empty slot left,
        // swaps into the first ACCEPTING empty slot (:970-988 — the
        // `PickItemMovementAction` gate skips a context-2 ammo row for a
        // non-`FitsAmmoSlot` kind without consuming it; `list2.RemoveAt(k)`
        // consumes the accepted one, and the filled slot joins the
        // demand-side partials, :984-985, so later stacks can pour into it).
        if emptied || empty_slots.is_empty() {
            continue;
        }
        let is_ammo_item = crate::registry::get(container[i].kind).is_some_and(|d| d.ammo != 0);
        if !is_ammo_item {
            continue;
        }
        let Some(k) = empty_slots
            .iter()
            .position(|&e| e < 54 || crate::item::fits_ammo_slot(container[i].kind))
        else {
            continue;
        };
        let e = empty_slots.remove(k);
        if e >= 54 {
            std::mem::swap(&mut ammo[e - 54], &mut container[i]);
        } else {
            std::mem::swap(&mut inventory[e], &mut container[i]);
        }
        moved_any = true;
        partial.push(e);
    }
    // The single chime (`:991-994`): `if (flag) PlaySound(7)` — one case-7
    // draw when anything moved (S1-P-i43, the Lead's faithful-cue decision).
    if moved_any {
        audio.play_pickup_sound(false, rng);
    }
    moved_any
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
            prefix: 0,
        }
    }

    /// The headless audio + a fixed-seed bare stream for the transfer cues
    /// (S1-P-i43 — the Lead's faithful-cue decision: every `PlaySound(7/38)`
    /// consumes a case-7/38 draw). `Audio::new(None)` has no output device but
    /// keeps `sound_volume` 1.0, so the unpositioned gate passes and the draw
    /// is spent exactly as in production.
    fn cue_rig() -> (crate::audio::Audio, tr_core::UnifiedRandom) {
        (crate::audio::Audio::new(None), tr_core::UnifiedRandom::new(0))
    }

    #[test]
    fn loot_all_drains_into_the_player_and_routes_the_coins() {
        // `ChestUI.LootAll` (:393-473): the container drains; the coins ride
        // `GetItem`'s coin rows, the stacks ride the pickup; the overflow
        // stays in the container.
        let mut container = [
            slot(9, 5),      // the wood stack -> the pickup
            slot(71, 50),    // the copper coins -> the coin rows
            slot(1, 1),      // the pickaxe (stack 1) -> the pickup
            ItemSlot { kind: 0, name: "", count: 0, prefix: 0 },
        ];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let mut coins = [slot(71, 10), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut ammo = empty_vault_ammo();
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        assert_eq!(coins[0].count, 60, "the coins merged into row 0");
        assert!(inventory.iter().any(|sl| sl.kind == 9 && sl.count == 5));
        assert!(inventory.iter().any(|sl| sl.kind == 1 && sl.count == 1));
        assert!(container[0].count == 0 && container[1].count == 0);
    }

    #[test]
    fn loot_all_merges_to_the_cap_and_keeps_the_overflow() {
        // `GetItem`'s returned item (:400-414): the stack merges to
        // `maxStack` (9999 for the 1.4.5 common items); a FULL same-kind
        // stack leaves the remainder in the slot.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[40] = slot(9, 9499); // cap - 500: the whole stack fits
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut ammo = empty_vault_ammo();
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        assert_eq!(inventory[40].count, 9999, "the merge filled to the cap");
        assert_eq!(container[0].count, 0);
        // The true overflow: every slot occupied — the full wood stack
        // refuses the merge and no empty slot exists, so the remainder
        // stays in the chest.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| slot(3, 9999));
        inventory[40] = slot(9, 9999);
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        assert_eq!(inventory[40].count, 9999);
        assert_eq!(container[0].count, 500, "no pickup slot accepted it");
    }

    #[test]
    fn loot_all_routes_the_overflow_into_the_void_vault_when_allowed() {
        // `Player.cs:39395-39398` — `LootAllFromChest`'s `CanGoIntoVoidVault`
        // (`GetItemSettings.cs:55-56`) makes the vault `GetItem`'s per-item
        // last resort: the inventory passes fail, the overflow fills the
        // vault (occupied stackable slots then empty ones), the coins still
        // ride the coin rows first. The bank variant (`LootAllFromBank`,
        // :55) passes `None` and keeps the overflow in the chest.
        let mut container = [slot(9, 5), slot(71, 50), slot(3, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        for s in inventory.iter_mut() {
            *s = slot(3, 9999); // stone everywhere: nothing else fits
        }
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut vault = empty_vault();
        let mut ammo = empty_vault_ammo();
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, Some(&mut vault), &mut audio, &mut rng);
        // S1-P-i50 retry: the empty coin rows fill DESCENDING (row 53 first,
        // `GetItem`'s `num = 54` reverse scan, `Player.cs:39387-39394`).
        assert_eq!(coins[3].count, 50, "the empty coin fill is DESCENDING (pInv 53, `:39387`)");
        assert_eq!(coins[0].count, 0, "row 0 stays empty");
        assert_eq!(container[0].count, 0, "the vault took the whole wood stack");
        assert_eq!(container[2].count, 0, "the vault took the whole stone stack");
        assert_eq!(vault[0].kind, 9);
        assert_eq!(vault[0].count, 5);
        assert_eq!(vault[1].kind, 3);
        assert_eq!(vault[1].count, 500);
        // Without the vault (`LootAllFromBank`, `GetItemSettings.cs:55`):
        // the overflow stays in the chest.
        let mut container = [slot(9, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        for s in inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        assert_eq!(container[0].count, 5, "the overflow stays put");
        // A kind the vault refuses (`CanVoidVaultAccept`, `Player.cs:
        // 39425-39433` — the dirtiest-block projectile 3822) stays too.
        let mut container = [slot(3822, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        for s in inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut vault = empty_vault();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, Some(&mut vault), &mut audio, &mut rng);
        assert_eq!(container[0].count, 5, "CanVoidVaultAccept refused");
        assert_eq!(vault[0].count, 0);
    }

    /// S1-P-i50 (retry) — `Player.GetItem`'s isACoin empty-fill order
    /// (`Player.cs:39387-39394`, the `num = 54` reverse scan): with every
    /// coin row EMPTY the stack lands in row 53 (the LAST row), not row 50;
    /// and when all four coin rows are FULL the coins overflow into the grid
    /// (the empty grid 49-0 descending), never back into the container while
    /// a grid slot is free. The pre-retry port filled row 0 first and had no
    /// grid passes — the C# walks `i = num - 1 .. 0` over 0..54.
    #[test]
    fn s1_p_i50_loot_all_fills_the_coin_rows_descending_and_overflows_to_the_grid() {
        // All rows empty: row 53 takes the stack.
        let mut container = [slot(71, 50)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut ammo = empty_vault_ammo();
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        assert_eq!(coins[3].count, 50, "the empty fill is DESCENDING (row 53)");
        assert_eq!(coins[0].count, 0, "row 50 stays empty");
        assert_eq!(container[0].count, 0);
        // All four coin rows full: the overflow rides the grid's LAST slot.
        let mut container = [slot(71, 50)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let mut coins = [slot(71, 100), slot(71, 100), slot(71, 100), slot(71, 100)];
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        assert_eq!(inventory[49].kind, 71, "the grid overflow takes slot 49");
        assert_eq!(inventory[49].count, 50);
        assert_eq!(container[0].count, 0, "nothing stays while a grid slot is free");
        // The occupied grid merge precedes the empty fills (`:39353-39364`):
        // a partial grid copper soaks the drop before any empty row is used.
        let mut container = [slot(71, 50)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[12] = slot(71, 60);
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        // 60 + 50 = 110 → the occupied merge caps at 100, DoCoins converts
        // the row to 1 silver (the merge arm `:39519` + `:39503`), the 10
        // remainder rides the empty coin row 53.
        assert_eq!(inventory[12].kind, 72, "the 100-copper row converted");
        assert_eq!(inventory[12].count, 1);
        assert_eq!(coins[3].count, 10, "the remainder rides row 53");
        assert_eq!(container[0].count, 0);
    }

    #[test]
    fn deposit_all_merges_then_parks_the_rest() {
        // `ChestUI.DepositAll` (:540-659): the rows scan 49..=10 (the hotbar
        // stays), the merge phase then the first free slot; the favorited
        // rows stay.
        let mut container = [slot(9, 500), slot(3, 100), slot(0, 0), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[0] = slot(9, 50); // the hotbar: untouched
        inventory[20] = slot(9, 200); // merges 500 -> 700
        inventory[21] = slot(3, 999); // merges 100 -> 999, 100 left
        inventory[22] = slot(41, 1); // a non-merge kind -> the first free slot
        let fav = |_: usize| false;
        let (mut audio, mut rng) = cue_rig();
        deposit_all(&mut container, &mut inventory, &fav, &mut audio, &mut rng);
        assert_eq!(container[0].count, 700);
        assert_eq!(container[1].count, 1099, "the 9999 cap takes the whole 999");
        assert!(container.iter().any(|sl| sl.kind == 41), "the leftover parked in the free slot");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 0);
        assert_eq!(inventory[0].count, 50, "the hotbar untouched");
    }

    #[test]
    fn quick_stack_matches_types_and_excludes_coins_on_the_container_side() {
        // `ChestUI.QuickStack` (:661-825) after the M3-CHEST-INT hoist: the
        // container's non-coin stacks are the targets; the coins never match
        // (the exclusion is container-side, :703-714); NO MoveCoins runs
        // here — the pre-pass is the dispatch layer's (the C# bank arm
        // :670-685 vs the world chest :687+; the dispatch-level bank
        // regression rides the clicks.rs tests).
        let mut container = [slot(9, 100), slot(71, 30), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 150); // merges into the wood target
        inventory[21] = slot(71, 200); // a grid coin stack: no target, stays
        let mut coins = [slot(71, 5), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        let (mut audio, mut rng) = cue_rig();
        quick_stack(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        assert_eq!(container[0].count, 250, "the wood merged");
        assert_eq!(container[1].count, 30, "the chest's coin stack is not a target");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 200, "no MoveCoins ran (the dispatch owns it)");
        assert_eq!(coins[0].count, 5, "the coin rows untouched");
    }

    #[test]
    fn quick_stack_consolidates_every_same_type_leftover_into_the_free_slots() {
        // `ChestUI.cs:772-812` — the empty-slot phase: for each empty
        // container slot the first queued source moves WHOLE into it
        // (:783-788) and every subsequent CanStack source stacks into it
        // until maxStack (:789-808) — all the same-type leftovers
        // consolidate into the chest. A different-prefix source skips the
        // filled slot and takes the next one whole (`flag` resets per slot,
        // :775; CanStack's prefix gate, :778).
        let mut container = [slot(9, 9000), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 9999); // full: pours 999, keeps 9000
        inventory[21] = slot(9, 500);
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        let (mut audio, mut rng) = cue_rig();
        quick_stack(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        assert_eq!(container[0].count, 9999, "the merge filled the partial");
        assert_eq!(container[1].kind, 9);
        assert_eq!(
            container[1].count, 9500,
            "the whole 9000 moved in and the 500 stacked after it"
        );
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 0, "no same-type leftover stays behind");
        // The prefix gate: a different-prefix source skips the filled slot
        // (:778) and takes the next empty slot whole (:783-788).
        let mut container = [slot(9, 10), slot(0, 0), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 9999); // pours 9989, keeps 10
        inventory[21] = ItemSlot { kind: 9, name: "", count: 50, prefix: 3 };
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        quick_stack(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        assert_eq!(container[0].count, 9999);
        assert_eq!(container[1].count, 10, "the unprefixed remainder moved whole");
        assert_eq!(
            container[2],
            ItemSlot { kind: 9, name: "", count: 50, prefix: 3 },
            "the prefixed stack took the next slot whole — never blended"
        );
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 0);
    }

    #[test]
    fn move_coins_is_a_noop_when_the_container_holds_no_coins() {
        // `ChestUI.cs:1044`/`:1048-1051` — `flag2` is set only when the
        // CONTAINER held a coin stack and `if (!flag2) return 0L;` follows:
        // a coin-less container never pulls the player's coins.
        let mut container = [slot(9, 100), slot(0, 0), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(71, 200); // a grid coin stack
        let mut coins = [slot(71, 60), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        let (mut audio, mut rng) = cue_rig();
        move_coins(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        assert_eq!(container[0].count, 100, "the container untouched");
        assert_eq!(inventory[20].count, 200, "the grid coins stay put");
        assert_eq!(coins[0].count, 60, "the coin rows stay put");
    }

    #[test]
    fn move_coins_pulls_the_player_coins_and_checks_the_pinv_favorite_space() {
        // `ChestUI.cs:1052-1081` — with a coin in the container the pull
        // runs; the favorited check reads each coin item's own flag over the
        // pInv indices — the coin rows are pInv 50-53 (:1054), so a GRID
        // favorite (hotbar slot 0) must not alias onto coin row 0.
        let mut container = [slot(71, 10)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[0] = slot(71, 20); // the hotbar slot 0: favorited
        inventory[20] = slot(71, 30);
        let mut coins = [slot(71, 5), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |i: usize| i == 0; // favorite_slots = [0] — a grid index
        let (mut audio, mut rng) = cue_rig();
        move_coins(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        // The favorited grid stack stays (:1054's own-flag check); the
        // unfavored grid stack and the coin row pull (10 + 30 + 5 = 45 — no
        // carry at 45).
        assert_eq!(inventory[0].count, 20, "the favorited grid stack stays");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(coins[0].count, 0, "coin row 0 pulled — pInv 50, not grid 0");
        assert_eq!(container[0].kind, 71);
        assert_eq!(container[0].count, 45, "the tallies consolidated into the container");
    }

    #[test]
    fn restock_fills_from_unprefixed_supply_only() {
        // `ChestUI.Restock` (:871-995): the container -> the player; the
        // supply must be UNPREFIXED demanded types (:919-924 — the
        // `hashSet.Contains` gate governs the whole loop, the ammo bonus
        // included); the capped fill.
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let (mut audio, mut rng) = cue_rig();
        let mut container = [
            slot(40, 30), // the arrows: demanded (partial in inv)
            slot(40, 10),
            slot(40, 20),
        ];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = ItemSlot { kind: 40, name: "", count: 50, prefix: 0 }; // partial
        inventory[21] = slot(43, 5); // the demand type
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(inventory[20].count, 110, "the 9999 cap takes all 60");
        assert!(container[0].count == 0 && container[1].count == 0 && container[2].count == 0);
        // The prefixed supply is refused everywhere (:919-924's gate — the
        // ammo bonus rides the same gated scan, so a prefixed stack never
        // moves via restock).
        let mut container = [ItemSlot { kind: 40, name: "", count: 5, prefix: 2 }];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(!moved, "the prefixed stack is not a restock source");
        assert_eq!(container[0].count, 5);
        // The UNPREFIXED undemanded ammo stack: :921's gate skips it before
        // the :966-989 bonus can see it — nothing moves.
        let mut container = [slot(40, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(!moved, "an undemanded stack is not a restock source");
        assert_eq!(container[0].count, 5, "the chest stack never moved");
        assert!(inventory.iter().all(|sl| sl.kind != 40), "the inventory untouched");
    }

    #[test]
    fn restock_excludes_coin_types_from_the_demand() {
        // `ChestUI.cs:902` — coin types (71-74) register neither demand nor
        // free slots: chest coins never "restock" into a grid coin partial
        // (vanilla moves nothing here).
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let (mut audio, mut rng) = cue_rig();
        let mut container = [slot(71, 100)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(71, 100); // a grid coin partial — not a demand
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(!moved, "coin types are not restock sources or demand");
        assert_eq!(container[0].count, 100, "the chest coins stay put");
        assert_eq!(inventory[20].count, 100, "the grid coins stay put");
    }

    #[test]
    fn restock_registers_ammo_row_demand_and_fills_it() {
        // `ChestUI.cs:899-917` — the demand walk runs pInv 57..=0: the ammo
        // rows 54-57 register demand and partials like the grid rows (the
        // coin rows 50-53 are the skipped span).
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let (mut audio, mut rng) = cue_rig();
        ammo[0] = slot(40, 50); // pInv 54: a partial ammo row — the demand
        let mut container = [slot(40, 30)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(ammo[0].count, 80, "the ammo-row partial filled");
        assert_eq!(container[0].count, 0, "the chest stack drained");
        assert!(inventory.iter().all(|sl| sl.kind != 40), "the grid untouched");
    }

    #[test]
    fn restock_bonus_swaps_a_demanded_ammo_remainder_into_an_empty_slot() {
        // `ChestUI.cs:966-989` — the bonus: a demanded ammo stack that did
        // NOT fully empty into the partials (`flag2 == false`, :955-964)
        // swaps into the first free empty slot — the demand walk's
        // descending order (:899) puts the empty AMMO rows first, so the
        // remainder lands on pInv 57 (`list2.RemoveAt(k)` consumes it, and
        // the filled slot joins the partials, :984-985). A stack that fully
        // emptied, a non-ammo kind, or no empty slot skips (:966).
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let (mut audio, mut rng) = cue_rig();
        let mut container = [slot(40, 20)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(40, 9990); // partial: room 9 — the fill leaves 11
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(inventory[20].count, 9999, "the partial filled to the cap");
        assert_eq!(ammo[3].kind, 40, "the remainder swapped into pInv 57");
        assert_eq!(ammo[3].count, 11);
        assert_eq!(inventory[49].count, 0, "the grid empties come after the ammo rows");
        assert_eq!(container[0].count, 0, "the swap emptied the container slot");
        // A stack that fully emptied into the partials skips the bonus
        // (:955-964's `flag2` → :966's `continue`).
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let mut container = [slot(40, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(40, 9990); // room 9 takes the whole 5
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(inventory[20].count, 9995);
        assert_eq!(ammo[3].count, 0, "no bonus after a full empty");
        assert_eq!(inventory[49].count, 0);
        assert_eq!(container[0].count, 0);
        // A non-ammo remainder never swaps (:966's `item.ammo == 0`).
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let mut container = [slot(9, 20)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 9990); // room 9 leaves 11
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved, "the partial fill moved");
        assert_eq!(inventory[20].count, 9999);
        assert_eq!(container[0].count, 11, "the wood remainder stays in the chest");
        assert_eq!(inventory[49].count, 0, "no bonus for a non-ammo kind");
        assert_eq!(ammo[3].count, 0);
    }



    fn empty_vault() -> BankStore {
        std::array::from_fn(|_| ItemSlot {
            kind: 0,
            name: "",
            count: 0,
            prefix: 0,
        })
    }

    /// The four empty ammo rows for the `loot_all` FillAmmo pre-pass.
    fn empty_vault_ammo() -> [ItemSlot; 4] {
        std::array::from_fn(|_| EMPTY_SLOT)
    }

    #[test]
    fn bank_index_space_maps_chest_to_store() {
        // `Player.cs:33492-33502`: -2..-5 → bank/bank2/bank3/bank4.
        assert_eq!(bank_for_chest(-1), None);
        assert_eq!(bank_for_chest(0), None);
        assert_eq!(bank_for_chest(-2), Some(0));
        assert_eq!(bank_for_chest(-3), Some(1));
        assert_eq!(bank_for_chest(-4), Some(2));
        assert_eq!(bank_for_chest(-5), Some(3));
    }

    #[test]
    fn void_vault_accept_fills_stacks_then_empty_slots() {
        // `Player.cs:39402-39423`: the occupied stackable slots first, then
        // ONE empty slot takes the WHOLE remainder (`inv[i] = returnItem`,
        // `:39539` — no cap, no multi-slot split; S1-P-i43 fix), each
        // accepted slot chiming before the fill (`:39444`/`:39534`).
        let mut vault = empty_vault();
        vault[0] = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 999,
            prefix: 0,
        };
        vault[1] = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 10,
            prefix: 0,
        };
        let (mut audio, mut rng) = cue_rig();
        let before = rng.clone();
        // 999 (full, silent) + 10: slot 1 takes 989 more (one cue), then the
        // FIRST empty slot takes the whole 1011 remainder (one cue).
        let taken = void_vault_take(&mut vault, 9, 0, 2000, 999, false, &mut audio, &mut rng);
        assert_eq!(taken, 2000);
        assert_eq!(vault[0].count, 999);
        assert_eq!(vault[1].count, 999, "slot 1 took 989 more");
        assert_eq!(vault[2].kind, 9);
        assert_eq!(vault[2].count, 1011, "one empty slot takes the WHOLE remainder");
        assert_eq!(vault[3].count, 0, "no multi-slot split");
        let mut witness = before;
        let _ = witness.next_range(-10, 11);
        let _ = witness.next_range(-10, 11);
        assert_eq!(rng, witness, "one case-7 draw per accepted slot");
        // The full vault refuses — no slot accepts, no cue draws.
        let mut vault = empty_vault();
        for slot in vault.iter_mut() {
            *slot = ItemSlot {
                kind: 3,
                name: "Stone Block",
                count: 999,
                prefix: 0,
            };
        }
        let before = rng.clone();
        let taken = void_vault_take(&mut vault, 9, 0, 10, 999, false, &mut audio, &mut rng);
        assert_eq!(taken, 0);
        assert_eq!(rng, before, "a refusal draws nothing");
        // A whole fit into an occupied slot RETURNS — the empty pass never
        // runs (`:39448-39457`), exactly one cue.
        let mut vault = empty_vault();
        vault[5] = ItemSlot { kind: 9, name: "Wood", count: 10, prefix: 0 };
        let before = rng.clone();
        let taken = void_vault_take(&mut vault, 9, 0, 10, 999, false, &mut audio, &mut rng);
        assert_eq!(taken, 10);
        assert_eq!(vault[5].count, 20);
        assert_eq!(vault[0].count, 0, "the empty pass never ran");
        let mut witness = before;
        let _ = witness.next_range(-10, 11);
        assert_eq!(rng, witness, "one cue for the occupied whole fit");
    }

    #[test]
    fn void_vault_rejects_the_excluded_kinds() {
        // `CanVoidVaultAccept` (`Player.cs:39425-39433`): 3822 refuses.
        assert!(!can_void_vault_accept(3822));
        assert!(can_void_vault_accept(9));
    }

    // --- S1-P-i43: the transfer census fixes --------------------------------

    /// S1-P-i43 — the LootAll coin arm rides the FULL `GetItem` pass order
    /// (`Player.cs:39353-39394`): occupied coin rows → occupied GRID → empty
    /// coin rows DESCENDING → empty grid reverse. The old coin-only shortcut
    /// skipped the grid merge and filled the empty coin rows ascending.
    #[test]
    fn s1_p_i43_loot_all_coins_merge_into_the_grid_then_fill_coin_rows_descending() {
        // A copper stack looted with a partial GRID copper stack present:
        // the occupied walk merges the grid partial FIRST (pInv 0-49 after
        // the coin rows), exactly like GetItem's `num2 = -4` loop.
        // Gate V4 re-pin: the fixture's 60+50 = 110 crosses the DoCoins
        // threshold — the grid row hits 100 (`:39519`), converts IN PLACE
        // to 1 silver (`DoCoins` `:39252`, no other silver row exists),
        // and the 10 remainder rides the descending empty coin row 53.
        // (The i43 witness predates i50's DoCoins and pinned the raw 100.)
        let mut container = [slot(71, 50)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        inventory[30] = slot(71, 60); // a grid copper partial (room 40)
        let mut coins = [EMPTY_SLOT; 4]; // the coin rows are empty
        let mut ammo = empty_vault_ammo();
        let (mut audio, mut rng) = cue_rig();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        // The grid partial soaked 40 (60 -> 100 -> DoCoins -> 1 silver), the
        // remaining 10 landed on the HIGHEST empty coin row (pInv 53).
        assert_eq!(inventory[30].kind, 72, "the 100-copper row converted (DoCoins)");
        assert_eq!(inventory[30].count, 1, "the occupied grid merge ran");
        assert_eq!(coins[3].count, 10, "the empty coin fill is descending (row 3)");
        assert_eq!(coins[0].count, 0, "row 0 stays empty — not ascending");
        assert_eq!(container[0].count, 0, "the stack drained");
    }

    /// S1-P-i43 — the transfer cues consume the case-7 pitch draws on the
    /// caller's stream (the Lead's faithful-cue decision): a LootAll that
    /// moves N stacks spends N `Next(-10, 11)` draws, in emission order.
    #[test]
    fn s1_p_i43_loot_all_cues_draw_one_case7_pitch_per_moved_stack() {
        let mut container = [slot(9, 5), slot(3, 7)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        let mut coins = [EMPTY_SLOT; 4];
        let mut ammo = empty_vault_ammo();
        let (mut audio, mut rng) = cue_rig();
        let before = rng.clone();
        loot_all(&mut container, &mut inventory, &mut coins, &mut ammo, None, &mut audio, &mut rng);
        // Two stacks moved (wood + stone), each into an empty grid slot —
        // two case-7 pitch draws (`Player.cs:39559`).
        let mut witness = before;
        let _ = witness.next_range(-10, 11);
        let _ = witness.next_range(-10, 11);
        assert_eq!(rng, witness, "one case-7 draw per moved stack");
    }

    /// S1-P-i43 — the QuickStack merge phase checks the coin-row favorite
    /// gate (`ChestUI.cs:728`, `array[j].favorited` over pInv 50-53): a
    /// favorited coin-row source never merges. (Coin types are excluded from
    /// the TARGET set, so this witness uses a non-coin source on a coin row —
    /// the gate is the source-side favorite check, not the type filter.)
    #[test]
    fn s1_p_i43_quick_stack_merge_checks_the_coin_row_favorite() {
        // A wood stack on coin row 0 (pInv 50), favorited: the merge phase
        // must skip it even though wood is a target type.
        let mut container = [slot(9, 100), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        let mut coins = [slot(9, 50), EMPTY_SLOT, EMPTY_SLOT, EMPTY_SLOT];
        let fav = |i: usize| i == 50; // pInv 50 favorited
        let (mut audio, mut rng) = cue_rig();
        quick_stack(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        // The favorited coin-row wood never merged into the chest target...
        assert_eq!(container[0].count, 100, "the favorited source skipped the merge");
        // ...but the empty-slot phase also gates on the favorite (:778), so
        // it stays on the coin row entirely.
        assert_eq!(coins[0].count, 50, "the favorited coin-row stack stays");
    }

    /// S1-P-i43 — the Restock partial-list removal (`ChestUI.cs:946-954`):
    /// a demand partial that FILLS to maxStack leaves `list`, so a later
    /// supply stack sees fewer candidates. Two supply stacks, one partial
    /// with room for exactly the first: the first fills it (and removes it),
    /// the second finds no partial and rides the ammo bonus / stays.
    #[test]
    fn s1_p_i43_restock_removes_a_filled_partial_from_the_demand_list() {
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| EMPTY_SLOT);
        // Two arrow supply stacks; the grid partial has room for only the
        // first (9990 + 9 = 9999). The first fills it and is removed from
        // `list`; the second (10) finds no partial left.
        let mut container = [slot(40, 9), slot(40, 10)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        inventory[20] = slot(40, 9990); // the only partial, room 9
        let (mut audio, mut rng) = cue_rig();
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(inventory[20].count, 9999, "the first supply filled the partial");
        // The second supply (10) had no partial left (the filled one was
        // removed) and is ammo, so the bonus swaps it into an empty ammo row.
        assert_eq!(container[0].count, 0, "the first supply drained");
        assert_eq!(ammo[3].kind, 40, "the second supply rode the ammo bonus");
        assert_eq!(ammo[3].count, 10);
        assert_eq!(container[1].count, 0, "the bonus emptied the second slot");
    }

    /// S1-P-i43 — the QuickStack empty phase's `list4` prune
    /// (`ChestUI.cs:761-770`) + the ONE pInv-ordered dictionary queue: a
    /// source drained by the MERGE phase leaves the dictionary before the
    /// empty phase, and a coin-row source that never merged rides the SAME
    /// queue as the grid sources — it stacks into the empty slot after the
    /// grid survivor's first placement (the C#'s single `foreach`, :777).
    #[test]
    fn s1_p_i43_quick_stack_prunes_drained_sources_and_interleaves_the_coin_rows() {
        // The wood target has room 100: grid source 20 (100 wood) merges and
        // DRAINS; grid source 21 (50 wood) merges nothing (the target is
        // full — the `num5 == 0` break) and stays queued. The coin-row wood
        // (pInv 50, 30) also stays. Empty slot 1 takes the FIRST queued
        // survivor — grid 21 (50) whole — then the coin row's 30 stacks in
        // (one interleaved walk, both land on slot 1).
        let mut container = [slot(9, 9899), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        inventory[20] = slot(9, 100); // drains into the target
        inventory[21] = slot(9, 50); // the target fills — stays queued
        let mut coins = [slot(9, 30), EMPTY_SLOT, EMPTY_SLOT, EMPTY_SLOT];
        let fav = |_: usize| false;
        let (mut audio, mut rng) = cue_rig();
        quick_stack(&mut container, &mut inventory, &mut coins, &fav, &mut audio, &mut rng);
        assert_eq!(container[0].count, 9999, "the merge filled the target");
        assert_eq!(inventory[20].count, 0, "the first source drained");
        assert_eq!(container[1].count, 80, "the empty slot took grid 21 first, then the coin row");
        assert_eq!(inventory[21].count, 0);
        assert_eq!(coins[0].count, 0, "the coin-row source rode the same queue");
        // Three cues: the merge (:750), the first placement + the stacked
        // placement (:782) — the drained source plays nothing.
        let mut witness = tr_core::UnifiedRandom::new(0);
        for _ in 0..3 {
            let _ = witness.next_range(-10, 11);
        }
        assert_eq!(rng, witness, "one case-7 draw per accepted placement");
    }

    /// S1-P-i43 — the Restock ammo-row `PickItemMovementAction` gate
    /// (`ChestUI.cs:934`, `ItemSlot.cs:1503-1508`): a non-`FitsAmmoSlot`
    /// supply never pours into a context-2 ammo-row partial, and the
    /// ammo-into-empty bonus skips (without consuming) an ammo row for a
    /// non-fitting kind — it lands on the grid's empty slot instead.
    #[test]
    fn s1_p_i43_restock_ammo_rows_gate_on_fits_ammo_slot() {
        // Ebonsand 370 (ammo = AmmoID.Sand but notAmmo — `Item.cs:5721`):
        // demanded by the ammo-row partial, supplied by the chest.
        // `fits_ammo_slot` refuses it, so the partial fill skips the ammo
        // row... and the bonus skips the ammo empty rows too, landing on
        // the grid.
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| EMPTY_SLOT);
        ammo[0] = slot(370, 50); // pInv 54: the gel partial (the demand)
        let mut container = [slot(370, 20)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        let (mut audio, mut rng) = cue_rig();
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(ammo[0].count, 50, "the notAmmo gel never poured into the ammo partial");
        // The bonus: gel's `ammo != 0` passes :966, but the ammo empty rows
        // (55-57) refuse it — the first GRID empty slot (pInv 49, the
        // descending scan's last) takes the swap.
        assert_eq!(inventory[49].kind, 370, "the gel landed on the grid, not an ammo row");
        assert_eq!(inventory[49].count, 20);
        assert!(ammo[1..].iter().all(|s| s.count == 0), "no ammo row took the gel");
        assert_eq!(container[0].count, 0);
        // A TRUE ammo kind (arrows 40) still rides the ammo rows.
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| EMPTY_SLOT);
        ammo[0] = slot(40, 50);
        let mut container = [slot(40, 20)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|_: usize| false, &mut audio, &mut rng);
        assert!(moved);
        assert_eq!(ammo[0].count, 70, "the arrows poured into the ammo partial");
    }

    /// S1-P-i43 — the Restock demand favorite gate (`ChestUI.cs:908`,
    /// `!favorited || !OnlyNeedOneInInventory()`): a favorited one-needed
    /// row registers no demand; a favorited row whose kind answers FALSE
    /// (the override table's bucket rows) still demands.
    #[test]
    fn s1_p_i43_restock_demand_skips_favorited_one_needed_rows() {
        // The Empty Bucket 205 (a `false` override row): favorited, it
        // STILL demands (OnlyNeedOneInInventory false).
        let mut ammo: [ItemSlot; 4] = std::array::from_fn(|_| EMPTY_SLOT);
        let mut container = [slot(205, 10)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        inventory[20] = slot(205, 5); // favorited partial
        let (mut audio, mut rng) = cue_rig();
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|i: usize| i == 20, &mut audio, &mut rng);
        assert!(moved, "the false-override row demands through the favorite");
        assert_eq!(inventory[20].count, 15);
        // The Magic Mirror 50 (useStyle 4, non-consumable → one-needed):
        // favorited, it registers NO demand — the chest supply stays.
        let mut container = [slot(50, 2)];
        let mut inventory: [ItemSlot; crate::item::INV] =
            std::array::from_fn(|_| EMPTY_SLOT);
        inventory[20] = slot(50, 1); // favorited one-needed partial
        let moved = restock(&mut container, &mut inventory, &mut ammo, &|i: usize| i == 20, &mut audio, &mut rng);
        assert!(!moved, "the favorited one-needed row demands nothing");
        assert_eq!(container[0].count, 2, "the supply stayed");
        assert_eq!(inventory[20].count, 1);
        // The override table's own pins.
        assert!(only_need_one_in_inventory(329), "the shadow key override");
        assert!(only_need_one_in_inventory(1522), "the mana star override");
        assert!(!only_need_one_in_inventory(205), "the bucket false override");
        assert!(!only_need_one_in_inventory(267), "the voodoo doll false override");
        assert!(only_need_one_in_inventory(50), "useStyle != 0 non-consumable");
        assert!(!only_need_one_in_inventory(9), "wood is consumable");
    }
}
