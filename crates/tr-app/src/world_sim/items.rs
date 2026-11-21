//! World item drops (plan stage A4): the fixed 60 Hz `Item.Update` physics and
//! pickup step, plus the `Item.NewItem`-shaped drop spawner.

use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::UnifiedRandom;

use crate::achievements;
use crate::app::{App, NetRole};
use crate::item::{slot_for_pickup, ItemDrop};
use crate::registry;

/// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
/// `ItemID.Sets.IsLavaImmuneRegardlessOfRarity` (`ItemID.cs`'s
/// `CreateBoolSet(false, 318, 312, 173, 174, 175, 4422, 2701, 205, 206,
/// 207, 1128, 2340, 2739, 2492, 1127, 85, 581, 582)`): the drops that
/// survive lava regardless of their rarity.
const LAVA_IMMUNE: &[u16] = &[
    318, 312, 173, 174, 175, 4422, 2701, 205, 206, 207, 1128, 2340, 2739, 2492, 1127, 85, 581, 582,
];

/// `NPCID.Sets.CantTakeLunchMoney` (`NPCID.cs:10133`):
/// `CreateBoolSet(394, 393, 392, 690, 492, 491, 662, 384, 478, 535, 658,
/// 659, 660, 128, 131, 129, 130, 139, 267, 247, 248, 246, 249, 245, 409,
/// 410, 397, 396, 401, 400, 440, 68, 534, 316, 259, 260, 43, 175, 101)` —
/// the NPCs the revenge coin-suck refuses (`WorldItem.cs:1124`).
const CANT_TAKE_LUNCH_MONEY: &[u16] = &[
    394, 393, 392, 690, 492, 491, 662, 384, 478, 535, 658, 659, 660, 128, 131, 129, 130, 139, 267,
    247, 248, 246, 249, 245, 409, 410, 397, 396, 401, 400, 440, 68, 534, 316, 259, 260, 43, 175,
    101,
];

pub(crate) fn update_items(app: &mut App, dt: f32) {
    if app.is_client() {
        // The host owns the drops: ask it to collect whatever lies in our
        // grab rectangle and hide those locally until the next authoritative
        // snapshot. The range is `GetItemGrabRange`'s expanded hitbox
        // (`Player.cs:34926`, S1-P-i47) — the old 26 px circle had no C#
        // counterpart. The client's accessory flags are the local player's;
        // the unported grants read false (the `GrabFlags` seam).
        let Some(p) = app.player.as_ref() else {
            return;
        };
        let (px, py, ph) = (p.x, p.y, p.height);
        // `Player.cs:27426-27428` — a dead player grabs nothing.
        if p.dead {
            return;
        }
        let in_range = |it: &ItemDrop| {
            // The `:34911` gate row, the client's half: shimmering and
            // grab-delayed drops stay until the host's authoritative update.
            if it.shimmer_time != 0.0
                || (it.grab_delay_time > 0
                    && (it.grab_delay_player == app.local_id as u8
                        || it.grab_delay_player == 255))
            {
                return false;
            }
            let range = crate::item::get_item_grab_range(it.kind, crate::item::GrabFlags::default())
                as f32;
            crate::item::grab_rect_intersects(
                px,
                py,
                crate::player::WIDTH,
                ph,
                range,
                it.pos,
            )
        };
        let ids: Vec<u32> = app
            .items
            .iter()
            .enumerate()
            .filter(|(i, it)| {
                in_range(it)
                    // `22 ItemOwner` gate (`GrabItems`, `Player.cs:34905-34923`):
                    // a drop reserved for another peer is not grabbable yet.
                    && !app
                        .net_reserved
                        .get(&(*i as i16))
                        .is_some_and(|(who, _)| *who != app.local_id as u8 && *who != 255)
                    // The grab-delay gate (`Player.cs:34911`, S1-P-i51): the
                    // host's `22 ItemOwner` carries the drop's live delay —
                    // a player drop locked for us (or everyone) is not
                    // grabbable until it expires.
                    && !(it.grab_delay_time > 0
                        && (it.grab_delay_player == app.local_id as u8
                            || it.grab_delay_player == 255))
            })
            .map(|(i, _)| i as u32)
            .collect();
        if let NetRole::Client(client) = &app.net {
            for &id in &ids {
                client.send(&crate::net::C2s::Pickup { drop: id });
            }
        }
        // Only the REQUESTED drops hide locally — a delayed/reserved drop
        // stays visible until its lock expires (the C# keeps it in
        // `Main.item`; the grab gate just refuses it, S1-P-i51). The ids are
        // ascending indices, so remove them high-first to keep them valid.
        let mut requested = ids.clone();
        requested.sort_unstable_by(|a, b| b.cmp(a));
        for id in requested {
            app.items.remove(id as usize);
        }
        return;
    }
    let world = &app.world;
    let player = &mut app.player;
    let items = &mut app.items;
    let coins = &mut app.coins;
    let inventory = &mut app.inventory;
    // The coin rows only — the ammo rows stay reachable through `app.equip`
    // (the deposit walk borrows them; a whole-`equip` alias would collide).
    let coin_rows = &mut app.equip.coins;
    let audio = &mut app.audio;
    let mut voodoo: Option<(Vec2, u32)> = None;
    // The drops whose shimmer ramp crossed 0.9 this step (the post-pass apply).
    let mut shimmer_due: Vec<u32> = Vec::new();
    // The splash drain's `NewDust` inputs, snapshotted before the closure
    // takes the field borrows (the `world_sim/player.rs:426` pattern — a
    // whole-`app` method call inside would collide with `&mut app.items`).
    let spawn_ctx = app.spawn_ctx();
    app.item_step.tick(dt.min(0.25), |step| {
        // --- `Player.GrabItems` (`Player.cs:34905-34976`, S1-P-i47) ---------
        // The C# runs it from `Player.Update` (`:27428`) BEFORE the item
        // physics (`Main.cs:17754`'s UpdatePlayers swap precedes `:18024`'s
        // UpdateItems); the port keeps both in the items pass, grab first.
        // A direct hitbox overlap picks the drop up; the expanded grab
        // rectangle only PULLS (the velocity applies in the physics half
        // below via `being_grabbed`).
        let mut i = 0;
        while i < items.len() {
            let kind = items[i].kind;
            // The `:34911` gate row: shimmering drops wait, the grab delay
            // locks its player (255 = everyone), a foreign reservation is
            // not grabbable, and a shimmered drop must nearly rest first.
            // `CanAcceptItemIntoInventory` (`:34896-34903`) reads
            // `preventAllItemPickups` — the Encumbering Stone has no port
            // flag yet, so the accept is unconditional (ANALYSIS).
            let reserved_elsewhere = app
                .net_reserved
                .get(&(i as i16))
                .is_some_and(|(who, _)| *who != app.local_id as u8 && *who != 255);
            let delay_locked = items[i].grab_delay_time > 0
                && (items[i].grab_delay_player == app.local_id as u8
                    || items[i].grab_delay_player == 255);
            if items[i].shimmer_time != 0.0
                || delay_locked
                || reserved_elsewhere
                || (items[i].shimmered && items[i].vel.length() >= 0.2)
            {
                i += 1;
                continue;
            }
            // The `IsAPickup` consume-on-touch arms (`PickupItem`
            // `:35045-35098`): hearts/stars ride the i52 arms below; the
            // nebula/4143 mana arms have no ported buff route — those kinds
            // are not grabbed until they land (ANALYSIS S1-P-i47/i52).
            if crate::item::is_a_pickup(kind) && !crate::item::is_heart_or_star(kind) {
                i += 1;
                continue;
            }
            let Some(pl) = player.as_ref() else {
                break;
            };
            // `Player.cs:27426-27428` — `if (!this.dead) this.GrabItems(i)`:
            // a dead player grabs nothing (the death drops lie until respawn).
            if pl.dead {
                break;
            }
            // The player scalars the grab reads, snapshotted so the vault
            // write below can take `player` mutably.
            let (px, py, ph, pvx) = (pl.x, pl.y, pl.height, pl.vx);
            let pc = Vec2::new(pl.center().0, pl.center().1);
            let (mana, mana_max) = (pl.mana, pl.mana_max);
            let vault_enabled = pl.void_vault_enabled;
            // `Main.expertMode` — the port's `buffs.expert_mode` mirror;
            // gates the revenge coin-suck (`WorldItem.cs:408-411`).
            let expert_mode = pl.buffs.expert_mode;
            let flags = crate::item::GrabFlags {
                gold_ring: pl.gold_ring,
                mana_magnet: pl.mana_magnet,
                life_magnet: pl.life_magnet,
                journey_far_range: pl.far_placement_range,
                ..Default::default()
            };
            let range = crate::item::get_item_grab_range(kind, flags) as f32;
            let drop_pos = items[i].pos;
            if crate::item::rects_intersect(
                px,
                py,
                crate::player::WIDTH,
                ph,
                drop_pos.x,
                drop_pos.y,
                crate::item::SIZE,
                crate::item::SIZE,
            ) {
                // `:34917-34922` — the direct pickup, gated on the held
                // slot being non-air OR the swing finished (`:34919`).
                if inventory[app.selected].kind == 0 && app.swing > 0.0 {
                    i += 1;
                    continue;
                }
                // S1-P-i52 — `Player.PickupItem`'s heart/star arms
                // (`Player.cs:35059-35079`), BEFORE the coin/inventory walk
                // (the C#'s `else if` chain order). `ItemID.Sets.IsAPickup`
                // (`ItemID.cs:493`) makes `ItemSpace` unconditionally true
                // (`:39155-39158`) — the pickup fires even with a FULL
                // inventory, and a full-life/full-mana player still consumes
                // the drop (the heal/restore caps at the max). The
                // `SyncItem` tail (`:35109`) is the MP send seam.
                if crate::item::HEART_KINDS.contains(&kind) {
                    // `:35059-35064`: `PlaySound(7)` (unpositioned), `Heal(20)`,
                    // `TurnToAir`. The case-7 pitch draw rides `UpdatePlayers`
                    // (the `GrabItems` caller under the `Main.cs:17754` swap).
                    // The HealLife text spawns HERE, synchronously at the
                    // pickup site (`Heal` → `HealEffect`, `:35112-35123` →
                    // `:4522-4529`) — the text carries the RAW 20 even when
                    // the cap eats part of the heal.
                    audio.play_pickup_sound(
                        false,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    if let Some(p) = player.as_mut() {
                        p.heal(20);
                        if let Some(amount) = p.take_heal_effect() {
                            let text = amount.to_string();
                            let size = match app.lookups.combat_text_font.as_ref() {
                                Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                                None => [text.chars().count() as f32 * 8.0, 10.0],
                            };
                            let location = crate::combat_text::Rect {
                                x: p.x,
                                y: p.y,
                                width: crate::player::WIDTH,
                                height: p.height,
                            };
                            crate::combat_text::spawn_amount(
                                &mut app.combat_texts,
                                location,
                                crate::combat_text::HEAL_LIFE,
                                amount,
                                false,
                                false,
                                size,
                                1.0,
                                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                            );
                        }
                    }
                    items.swap_remove(i);
                    continue;
                }
                if crate::item::STAR_KINDS.contains(&kind) {
                    // `:35065-35079`: `PlaySound(7)`, `statMana += 100`, the
                    // local-player `TryApplyManaHeat(0.2f, 2f)` +
                    // `ManaEffect(100)` (the RAW 100 — the cap comes AFTER
                    // the text, `:35074-35077`), then `TurnToAir`.
                    audio.play_pickup_sound(
                        false,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    if let Some(p) = player.as_mut() {
                        p.mana += 100;
                        p.try_apply_mana_heat(0.2, 2.0);
                        let text = "100".to_string();
                        let size = match app.lookups.combat_text_font.as_ref() {
                            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                            None => [text.chars().count() as f32 * 8.0, 10.0],
                        };
                        let location = crate::combat_text::Rect {
                            x: p.x,
                            y: p.y,
                            width: crate::player::WIDTH,
                            height: p.height,
                        };
                        crate::combat_text::spawn_amount(
                            &mut app.combat_texts,
                            location,
                            crate::combat_text::HEAL_MANA,
                            100,
                            false,
                            false,
                            size,
                            1.0,
                            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        );
                        if p.mana > p.mana_max {
                            p.mana = p.mana_max;
                        }
                    }
                    items.swap_remove(i);
                    continue;
                }
                if registry::is_coin(kind) {
                    // S1-P-i50 — the revenge pickup arm
                    // (`WorldItem.GetPickedUpByMonsters_Money`,
                    // `WorldItem.cs:1118-1201`): EXPERT-ONLY (`:408-410` —
                    // `Main.expertMode && this.IsACoin`), a nearby hostile
                    // NPC (`lifeMax > 5`, not friendly/immortal/
                    // dontTakeDamage/statue-spawned, not
                    // `NPCID.Sets.CantTakeLunchMoney`, `:1124`) whose
                    // hitbox intersects the drop's sucks 50-75% of the
                    // stack into `extraValue` while the CARRIER's
                    // `extraValue < value && extraValue + value <
                    // 999000000` (`realLife` redirect, `:1143-1155`).
                    let mut drop = items[i].clone();
                    if expert_mode {
                        let coin_copper = registry::coin_value(kind) * drop.count as u64;
                        // The C# walks Main.npc in index order and processes
                        // EVERY intersecting NPC (no break, `:1121-1200`);
                        // the port's compacted list keeps the same order.
                        let (dx0, dy0) = (drop.pos.x as i32, drop.pos.y as i32);
                        let dsz = crate::item::SIZE as i32;
                        let mut n_idx = 0;
                        while n_idx < app.npcs.len() && drop.count > 0 {
                            let eligible = {
                                let n = &app.npcs[n_idx];
                                let (nw, nh) = n.size();
                                // `:1124`'s gate row + `:1155`'s rectangle
                                // intersection (truncated int boxes).
                                n.max_hp > 5
                                    && !n.friendly
                                    && !n.immortal
                                    && !n.dont_take_damage
                                    && !n.spawned_from_statue
                                    && !CANT_TAKE_LUNCH_MONEY.contains(&n.kind.cs_id())
                                    && dx0 < (n.x as i32) + nw as i32
                                    && dx0 + dsz > n.x as i32
                                    && dy0 < (n.y as i32) + nh as i32
                                    && dy0 + dsz > n.y as i32
                            };
                            if eligible {
                                // `:1143-1154` — the realLife redirect: the
                                // carrier's extraValue is the budget and the
                                // deposit target (`Npc::carrier_index`, the
                                // port's stable worm_id/seg_index link).
                                let target = crate::npc::Npc::carrier_index(&app.npcs, n_idx)
                                    .unwrap_or(n_idx);
                                let extra = app.npcs[target].extra_value;
                                // `:1155` — the f32 compares, widened exactly.
                                if (extra as f32) < coin_copper as f32
                                    && (extra as f32 + coin_copper as f32) < 999_000_000.0
                                {
                                    // `num5 = Main.rand.Next(50, 76) * 0.01f`
                                    // + the copper (71) `Next(51)` and silver
                                    // (72) `Next(26)` extras, clamped at 1f;
                                    // `num6 = (int)(stack * num5)` clamped
                                    // 1..stack (`WorldItem.cs:1157-1178`). NO
                                    // count==1 short-circuit — a 1-stack draws
                                    // the same pair. The draws ride the
                                    // `UpdateItems` stream (`Main.cs:18024`).
                                    let rng = app.rngs.stream(crate::app::RngKey::UpdateItems);
                                    let mut num5 = rng.next_range(50, 76) as f32 * 0.01;
                                    if kind == 71 {
                                        num5 += rng.next_max(51) as f32 * 0.01;
                                    }
                                    if kind == 72 {
                                        num5 += rng.next_max(26) as f32 * 0.01;
                                    }
                                    num5 = num5.min(1.0);
                                    let share =
                                        ((drop.count as f32 * num5) as u32).clamp(1, drop.count);
                                    let picked = registry::coin_value(kind) * share as u64;
                                    app.npcs[target].extra_value += picked;
                                    drop.count -= share;
                                    // S1-E-21f — `NPC.moneyPing` (`NPC.cs:
                                    // 90052-90054`, `WorldItem.cs:1189`):
                                    // `PlaySound(38, base.position)` at the
                                    // DROP's top-left, right after the share
                                    // draws on the same `UpdateItems` stream.
                                    audio.play_coin_ping(
                                        drop.pos.x,
                                        drop.pos.y,
                                        app.rngs.stream(crate::app::RngKey::UpdateItems),
                                    );
                                }
                            }
                            n_idx += 1;
                        }
                        if drop.count == 0 {
                            // `:1195-1198` — `TurnToAir`.
                            items.swap_remove(i);
                            continue;
                        }
                    }
                    // S1-P-i50 — the coin routing: `Player.GetItem`'s isACoin
                    // arm (`Player.cs:39330-39344`) fills the coin slots 50-53
                    // first (occupied merges ascending, then empty fills
                    // DESCENDING — the `num = 54` reverse scan `:39387-39394`),
                    // then the grid 0..49 occupied merges (`:39353-39364`), then
                    // the grid empty fills descending. Coins have `useStyle 1`
                    // but the `!isACoin` gate (`:39365`) skips their
                    // hotbar-first empty pass. The stacks are the 1:1 storage;
                    // the `App.coins` purse stays a mirror for the
                    // shop/death/save lanes (the recorded deviation, ANALYSIS
                    // S1-P-i50).
                    let mut remaining = drop.count;
                    let cap = registry::max_stack(kind);
                    // The purse mirror credits what the INVENTORY stacks take
                    // (the vault remainder is banked, not spendable).
                    let purse_before = remaining;
                    {
                        // The carry view in C# `Player.inventory` index order
                        // (grid 0..50, coin rows 50..54; the ammo rows 54..58
                        // are never touched — coins are `notAmmo`, so
                        // `FitsAmmoSlot` is false and both `GetItem` scans
                        // stop at 54). S1-P-i50 retry: `DoCoins` runs
                        // IMMEDIATELY per fill (`Player.cs:39503`/`:39519`/
                        // `:39569`), NOT as a post-sweep — a conversion can
                        // merge-and-air a coin row the later empty pass then
                        // refills, so the deferred sweep changed slot
                        // placement. The conversion is RNG-free; the chime
                        // fires per accepted slot BEFORE the fill (`:39491`/
                        // `:39559`, S1-P-i43's faithful-cue decision), on the
                        // `UpdatePlayers` stream (the GrabItems caller).
                        let mut slots =
                            crate::item::carry_slots_mut(inventory, coin_rows, None);
                        let n = slots.len(); // INV + COINS = 54
                        // Occupied pass, C# `i = -4..50` (`num2 = -4`,
                        // `:39353-39364`): coin rows 50-53 first, then the
                        // grid 0-49, ascending.
                        for i in (crate::item::INV..n).chain(0..crate::item::INV) {
                            if remaining == 0 {
                                break;
                            }
                            let s = &mut *slots[i];
                            if s.count > 0
                                && s.kind == kind
                                && s.prefix == drop.prefix
                                && s.count < cap
                            {
                                let room = cap - s.count;
                                audio.play_pickup_sound(
                                    true,
                                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                                ); // `:39491` — before the fill
                                if remaining <= room {
                                    s.count += remaining;
                                    remaining = 0;
                                    crate::item::do_coins(&mut slots, i); // `:39503`
                                    break;
                                }
                                s.count = cap;
                                remaining -= room;
                                crate::item::do_coins(&mut slots, i); // `:39519`
                            }
                        }
                        // Empty pass, C# `i = 53..0` DESCENDING (the
                        // non-favorited reverse scan `:39387-39394`; the
                        // `!isACoin` gate `:39365` skips the coin hotbar-first
                        // empty pass; world drops are never favorited).
                        for i in (0..n).rev() {
                            if remaining == 0 {
                                break;
                            }
                            if slots[i].count == 0 {
                                audio.play_pickup_sound(
                                    true,
                                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                                ); // `:39559`
                                *slots[i] = crate::item::ItemSlot {
                                    kind,
                                    name: registry::name(kind),
                                    count: remaining,
                                    prefix: drop.prefix,
                                };
                                remaining = 0;
                                crate::item::do_coins(&mut slots, i); // `:39569`
                            }
                        }
                    }
                    let into_inventory = purse_before - remaining;
                    if into_inventory > 0 {
                        *coins = (*coins)
                            .saturating_add(registry::coin_value(kind) * into_inventory as u64);
                    }
                    // The vault last resort (`Player.cs:39395-39398`,
                    // `PickupItemFromWorld`'s `CanGoIntoVoidVault: true`,
                    // `GetItemSettings.cs:57`) — coins included; the fills
                    // chime per accepted slot INSIDE the helper (`:39444`/
                    // `:39534`, S1-P-i43) on the `UpdatePlayers` stream.
                    if remaining > 0 && vault_enabled && crate::bank::can_void_vault_accept(kind)
                    {
                        let taken = crate::bank::void_vault_take(
                            &mut player.as_mut().unwrap().banks[3],
                            kind,
                            drop.prefix,
                            remaining,
                            cap,
                            true,
                            audio,
                            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        );
                        remaining -= taken;
                    }
                    if remaining == 0 {
                        items.swap_remove(i);
                        continue;
                    }
                    // `PickupItem`'s `ReplaceWith(item)` (`Player.cs:
                    // 35102-35103`) — the surplus stays a world drop.
                    items[i].count = remaining;
                    i += 1;
                    continue;
                }
                // `PickupItem`'s inventory arm (`:35099-35108`): `GetItem`
                // with `PickupItemFromWorld` (`GetItemSettings.cs:57` —
                // `CanGoIntoVoidVault`), the remainder `ReplaceWith`'d back
                // into the world drop. The deposit walk is the C#'s order:
                // ammo pre-pass → occupied forward → usable hotbar → empty
                // REVERSE → the void vault LAST (`Player.cs:39330-39399`).
                // S1-C-9: the inventory receives the `remaining` count
                // exactly once — the walk never re-adds a merged stack.
                let count = items[i].count;
                let prefix = items[i].prefix;
                let favorite = |idx: usize| app.favorite_slots.contains(&idx);
                let mut remaining = crate::item::deposit_pickup(
                    inventory,
                    &mut app.equip.ammo,
                    kind,
                    prefix,
                    count,
                    favorite,
                    &mut |moved| {
                        // S1-E-21f — one chime per accepted slot
                        // (`:39491`/`:39559`, `FillAmmo` `:39274`/`:39321`),
                        // the Grab pitch draw on `UpdatePlayers`; the
                        // achievement notify rides each fill
                        // (`AchievementsHelper.NotifyItemPickup`, `:39505`).
                        audio.play_pickup_sound(
                            false,
                            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        );
                        let _ = moved;
                        if let Some(_done) = app
                            .achievements
                            .notify(achievements::CondKind::ItemPickup(kind))
                        {
                            log::info!("achievement completed");
                        }
                    },
                );
                // `:39395-39398` — the vault is the LAST resort, after every
                // inventory pass failed (the old port routed it FIRST).
                // S1-P-i43 — the fills chime per accepted slot INSIDE the
                // helper (`Player.cs:39444`/`:39534`), `IsACoin ? 38 : 7`,
                // on the `UpdatePlayers` stream (the GrabItems caller).
                if remaining > 0
                    && vault_enabled
                    && crate::bank::can_void_vault_accept(kind)
                {
                    let taken = crate::bank::void_vault_take(
                        &mut player.as_mut().unwrap().banks[3],
                        kind,
                        prefix,
                        remaining,
                        registry::max_stack(kind),
                        registry::is_coin(kind),
                        audio,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    remaining -= taken;
                }
                if remaining == 0 {
                    items.swap_remove(i);
                    continue;
                }
                // The surplus stays where it fell (`ReplaceWith(item)`).
                items[i].count = remaining;
                i += 1;
                continue;
            }
            if !crate::item::grab_rect_intersects(
                px,
                py,
                crate::player::WIDTH,
                ph,
                range,
                drop_pos,
            ) {
                i += 1;
                continue;
            }
            // `ItemSpace` + `CanPullItem` (`:34930-34934`): no room, no pull.
            let vault = player.as_ref().unwrap();
            let status = crate::item::item_space(
                inventory,
                &app.equip.ammo,
                kind,
                items[i].prefix,
                |idx| app.favorite_slots.contains(&idx),
                if vault_enabled { Some(&vault.banks[3]) } else { None },
            );
            if !status.can_take {
                i += 1;
                continue;
            }
            // `:34935-34972` — the pull arms. `flag2` = the star family; at
            // full mana without the Mana Magnet stars are not pulled at all
            // (`:34947`). The C# arm order (`:34943-34971`): manaMagnet-star
            // → lifeMagnet-heart → nebula → void-vault → goldRing-coin →
            // journey far-range → common. The treasure magnet's RANGE row
            // is live via GrabFlags; the nebula pull rides the unported
            // consume arms (S1-P-i52).
            let drop = &mut items[i];
            drop.shimmered = false;
            drop.being_grabbed = true;
            let flag2 = matches!(kind, 184 | 1735 | 1868);
            let held_magic = registry::get(inventory[app.selected].kind)
                .is_some_and(|d| d.magic);
            if flags.mana_magnet && flag2 {
                // `:34943-34946` — `PullItem_Pickup(worldItem, 12f, 5)`.
                crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 12.0, 5);
            } else if !flag2 || held_magic || mana != mana_max {
                if flags.life_magnet && matches!(kind, 58 | 1734 | 1867) {
                    // `:34949-34952` — `PullItem_Pickup(worldItem, 15f, 5)`.
                    crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 15.0, 5);
                } else if status.to_void_vault {
                    // `PullItem_ToVoidVault` (`:34978-34981`).
                    crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 12.0, 5);
                } else if flags.gold_ring && registry::is_coin(kind) {
                    // `:34961-34963` — the Gold Ring's coin pull, the
                    // 4:1 velocity blend `(v*4 + pull)/5`.
                    crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 12.0, 5);
                } else if flags.journey_far_range {
                    // `:34965-34967` — the Journey far-range pull,
                    // `PullItem_Pickup(7f, 1)` (no blend).
                    crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 7.0, 1);
                } else {
                    crate::item::pull_item_common(drop.center(), &mut drop.vel, pc, pvx, 0.75);
                }
            }
            i += 1;
        }
        // --- `WorldItem.UpdateItem`'s physics half -------------------------
        let mut i = 0;
        while i < items.len() {
            let mut drop = items[i].clone();
            // `WorldItem.CheckLavaDeath`'s voodoo arm (`WorldItem.cs:418-421`,
            // `:681-763`): the doll in lava is consumed and the sacrifice
            // runs (the resolve rides the post-tick pass — the closure holds
            // the items borrow).
            if drop.kind == 267 {
                let tx = (drop.center().x / TILE_PX) as i32;
                let ty = (drop.center().y / TILE_PX) as i32;
                let in_lava = world.tile(tx, ty).is_some_and(|t| {
                    t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
                });
                if in_lava {
                    voodoo = Some((drop.center(), drop.count));
                    items.swap_remove(i);
                    continue;
                }
            }
            // `CheckLavaDeath`'s non-doll arm (`WorldItem.cs:688-698`): the
            // `rare 0/-1` drops burn unless the item rides
            // `ItemID.Sets.IsLavaImmuneRegardlessOfRarity` (the 18-row set,
            // `ItemID.cs`'s `CreateBoolSet(false, 318, 312, 173, ...)`).
            let lava_immune = LAVA_IMMUNE.contains(&drop.kind);
            let rare = registry::get(drop.kind).map(|d| d.rare).unwrap_or(0);
            let tx = (drop.center().x / TILE_PX) as i32;
            let ty = (drop.center().y / TILE_PX) as i32;
            let in_lava = world.tile(tx, ty).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
            });
            if in_lava && (rare == 0 || rare == -1) && !lava_immune {
                // `TurnToAirAndSync` — the drop vanishes.
                items.swap_remove(i);
                continue;
            }
            let was_grabbed = drop.being_grabbed;
            // `MoveInWorld`'s repulsion scan (`WorldItem.cs:790-847`) runs at
            // the head of the movement half, BEFORE the gravity add — the
            // slice-level pass nudges the neighbours' live velocities (and
            // this drop's own X) and parks the `flag` for the ×0.8 tail. A
            // grabbed drop skips MoveInWorld entirely (`:425-435`), no scan.
            // The clone re-reads its X afterwards — the scan wrote the slot.
            if !was_grabbed && drop.shimmered {
                drop.repulsed = crate::item::ItemDrop::repel_shimmered(items, i);
                drop.vel.x = items[i].vel.x;
            }
            drop.update(world, step);
            // `MoveInWorld`'s entry/exit splash (`WorldItem.cs:881-1035`):
            // the drop queued the edge; the burst drains HERE, inside the
            // per-drop loop, BEFORE the flicker head below — the C# draws
            // the dust + the case-19 play inside `MoveInWorld` (`:413`),
            // ahead of `UpdateItem_VisualEffects` (`:436`), all on the
            // `UpdateItems` stream (`Main.cs:18024`'s swap). The g67
            // rain-splash contract: `DustPool::new_dust` + the post-writes.
            if let Some(splash) = drop.take_splash() {
                item_splash(
                    &mut app.dust,
                    &spawn_ctx,
                    world,
                    app.camera,
                    &mut app.audio,
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                    splash,
                );
            }
            // S1-E-5 — `WorldItem.UpdateItem_VisualEffects`'s rand/essScale
            // head (`WorldItem.cs:1206-1293`, `item_lights.rs`): the C# runs
            // it at `:436`, after the movement half, per live drop in
            // `Main.item` slot order, under the `UpdateItems` swap
            // (`Main.cs:18024`) — so the flicker draws ride this stream, in
            // this order. A collected drop was `TurnToAir`'d in the grab
            // pass above and never reaches here; the lava-burned drops are
            // the same. The RNG-free arms stay in the frame feed
            // (`entity_lights::item_light`).
            let ess = app.ess_scale;
            if let Some((x, y, rgb)) = crate::item_lights::rand_light(
                drop.kind,
                drop.pos,
                crate::item::SIZE,
                ess,
                app.rngs.stream(crate::app::RngKey::UpdateItems),
            ) {
                app.entity_lights.add(x, y, rgb);
            }
            // `WorldItem.UpdateShimmer`'s 0.9 trigger (`WorldItem.cs:528-535`):
            // the transform runs after the step's items borrow is released (the
            // apply needs the app wholesale for the spawns and the world flags).
            if drop.shimmer_transform_due() {
                shimmer_due.push(i as u32);
            }
            items[i] = drop;
            // `TryCombiningIntoNearbyItems` (`WorldItem.cs:157-186`, called
            // at `:404` inside the non-grabbed branch): the passive world
            // stack merge. The port runs it after the movement half (the
            // C# combines before `MoveInWorld`; RNG-free, the lerp targets
            // differ by one tick of motion — ANALYSIS S1-P-i47). The
            // drained sources are later indices, removed highest-first.
            if !was_grabbed {
                let drained = crate::item::ItemDrop::combine_nearby(items, i);
                for j in drained.into_iter().rev() {
                    items.swap_remove(j);
                }
            }
            i += 1;
        }
    });
    app.item_step.cap_backlog();
    // The voodoo sacrifice resolves after the items borrow drops (the
    // spawn/kill pass takes `app` wholesale).
    if let Some((pos, stack)) = voodoo {
        crate::world_sim::npcs::voodoo_doll_lava_death(app, pos, stack);
    }
    // The shimmer transforms apply in the list order (the C#'s single-item
    // `GetShimmered` per tick; a step fires at most one per drop).
    crate::shimmer::apply_due_transforms(app, &shimmer_due);
}

/// `WorldItem.MoveInWorld`'s entry/exit splash burst (`WorldItem.cs:881-1035`,
/// W4-e19 deferred 4 — the g67 rain-splash contract): the dust family by the
/// edge's kind flags (lava 35 ×5 → shimmer 308 ×10 with the `Next(6)` colour
/// switch → honey 152 ×5 → water `dustWater()` ×10), each `Dust.NewDust` at
/// `(pos.X - 6, pos.Y + height/2 - 8)` with the `width + 12 × 24` box (the
/// water EXIT arm drops the `- 8`, `:1011`), then the per-family post-writes
/// and the case-19 play (`SoundID.Splash`, style 4/5 for the shimmer
/// entry/exit, the default 1 otherwise). All draws ride the `UpdateItems`
/// stream — the C# runs inside `WorldItem.UpdateItem` under the
/// `Main.cs:18024` swap. The dust rolls come FIRST (the C# plays the sound
/// AFTER the loop); the play's own pitch draw follows the attenuation gate
/// (`audio::play_splash_sound`).
fn item_splash(
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::world::World,
    camera: Vec2,
    audio: &mut crate::audio::Audio,
    rng: &mut UnifiedRandom,
    splash: crate::item::ItemSplash,
) {
    use tr_core::particles::Color;
    // The dust anchor (`:892` etc.): `position.X - 6f`,
    // `position.Y + height/2 - 8f`; the water exit arm (`:1011`) has no `- 8`.
    let size = crate::item::SIZE;
    let water_exit = !splash.lava && !splash.shimmer && !splash.honey && splash.exit;
    let y_off = if water_exit { size / 2.0 } else { size / 2.0 - 8.0 };
    let anchor = Vec2::new(splash.pos.x - 6.0, splash.pos.y + y_off);
    let (w, h) = (size as i32 + 12, 24);
    // The family: (dust type, count, style). `Dust.dustWater()` reads the
    // camera's water style (`Main.waterStyle`, the g67 seam).
    let (ty, n, style) = if splash.lava {
        (35, 5, 1) // `:944-953`/`:1023-1032`
    } else if splash.shimmer {
        (308, 10, if splash.exit { 5 } else { 4 }) // `:890-913`/`:969-992`
    } else if splash.honey {
        (152, 5, 1) // `:917-926`/`:996-1005`
    } else {
        let ws = crate::biome::calculate_water_style(world, camera) as i32;
        (tr_core::particles::dust_water(ws), 10, 1) // `:930-939`/`:1009-1018`
    };
    for _ in 0..n {
        let idx = dust.new_dust(
            rng, ctx, world, anchor, w, h, ty, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        if ty == 35 {
            // `:946-951` — vel.Y -= 1.5, vel.X *= 2.5, scale 1.3, alpha 100.
            d.velocity.y -= 1.5;
            d.velocity.x *= 2.5;
            d.scale = 1.3;
            d.alpha = 100;
            d.no_gravity = true;
        } else if ty == 308 {
            // `:892-911` — vel.Y -= 4, vel.X *= 2.5, scale 0.8 (SET, not ×),
            // noGravity, then the `Next(6)` four-colour switch.
            d.velocity.y -= 4.0;
            d.velocity.x *= 2.5;
            d.scale = 0.8;
            d.no_gravity = true;
            d.color = match rng.next_max(6) {
                0 => Color::new(255, 255, 210, 255),
                1 => Color::new(190, 245, 255, 255),
                2 => Color::new(255, 150, 255, 255),
                _ => Color::new(190, 175, 255, 255),
            };
        } else if ty == 152 {
            // `:919-924` — vel.Y-- (1.0), vel.X *= 2.5, scale 1.3, alpha 100.
            d.velocity.y -= 1.0;
            d.velocity.x *= 2.5;
            d.scale = 1.3;
            d.alpha = 100;
            d.no_gravity = true;
        } else {
            // `:932-937` — vel.Y -= 4, vel.X *= 2.5, scale *= 0.8 (MULTIPLY),
            // alpha 100, noGravity.
            d.velocity.y -= 4.0;
            d.velocity.x *= 2.5;
            d.scale *= 0.8;
            d.alpha = 100;
            d.no_gravity = true;
        }
    }
    // `:913`/`:926`/`:939`/`:953` — `SoundEngine.PlaySound(19, (int)x, (int)y
    // [, style])`: the shimmer arms pass 4 (entry) / 5 (exit), the rest the
    // default style 1. The play gates on attenuation BEFORE its pitch draw.
    audio.play_splash_sound(style, splash.pos.x as i32 as f32, splash.pos.y as i32 as f32, rng);
}

/// Spawns the drop left by a broken tile, with a small random pop like
/// `Item.NewItem` — `count` carries the C# stack (`Item.NewItem`'s `stack`
/// argument; the pot/pile/herb loot rolls spawn several items at once).
///
/// `rng` is the CALLER's stage stream (`Item.NewItem` draws `Main.rand` = the
/// active swap's stream, `Item.cs:49654-49727`) — shared by every pass, so
/// each caller feeds its own; no field lookups here (the drop list comes in
/// too, so the caller's stream borrow and the list stay disjoint).
pub(crate) fn spawn_item_drop(
    items: &mut Vec<ItemDrop>,
    rng: &mut UnifiedRandom,
    kind: u16,
    count: u32,
    tx: i32,
    ty: i32,
) {
    spawn_item_drop_px(
        items,
        rng,
        kind,
        count,
        Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX),
    );
}

/// The pixel-position variant of [`spawn_item_drop`] — the spawn points that
/// name a world position instead of a tile cell (`Item.RequestNewItem`'s
/// `center`, e.g. the extractinator's cursor drop,
/// `Player.DropItemFromExtractinator` `Player.cs:42081-42089`). The draws keep
/// the port's pinned `Item.NewItem` order (the velocity/phase triplet, then
/// the prefix; the C# runs event remaps → `Prefix` → velocity, the recorded
/// order deviation) — all on `rng`, the site's stage stream (`Item.Prefix`
/// reads `Main.rand` = the active swap, `Item.cs:527`).
pub(crate) fn spawn_item_drop_px(
    items: &mut Vec<ItemDrop>,
    rng: &mut UnifiedRandom,
    kind: u16,
    count: u32,
    pos: Vec2,
) {
    let r1 = rng.next_float();
    let r2 = rng.next_float();
    let r3 = rng.next_float();
    // `Item.NewItem`'s `prefix: -1` on the tile/pot loot calls
    // (`WorldGen.cs:54248+`): the natural roll. Non-prefixable kinds draw
    // nothing (`Prefix(-1)`'s `CanHavePrefixes` gate).
    let prefix = crate::prefix::roll_prefix(kind, -1, rng);
    let drop = ItemDrop::new(
        kind,
        count.max(1),
        prefix,
        pos.x + 1.0,
        pos.y,
        (r1 * 2.0 - 1.0) * 1.4,
        -2.2 - r2 * 1.2,
        r3 * std::f32::consts::TAU,
    );
    items.push(drop);
    if items.len() > 300 {
        items.remove(0);
    }
}

/// The field-wise pixel variant of [`spawn_item_drop_px`] — the call sites that
/// already hold a borrow of one of the app's fields (the player, inside the
/// fixed-step closure) and therefore cannot take the whole `&mut App`; the
/// caller feeds its stage stream, exactly like the pixel entry point above
/// (S1-L-6b1's field-wise split kept, with one stream instead of the old
/// item/prefix pair).
pub(crate) fn spawn_item_drop_px_fields(
    items: &mut Vec<ItemDrop>,
    rng: &mut UnifiedRandom,
    kind: u16,
    count: u32,
    pos: Vec2,
) {
    spawn_item_drop_px(items, rng, kind, count, pos);
}

/// The field-wise tile variant of [`spawn_item_drop`] — same borrow reason;
/// the tile cell becomes the pixel spawn point here.
pub(crate) fn spawn_item_drop_fields(
    items: &mut Vec<ItemDrop>,
    rng: &mut UnifiedRandom,
    kind: u16,
    count: u32,
    tx: i32,
    ty: i32,
) {
    spawn_item_drop_px(
        items,
        rng,
        kind,
        count,
        Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX),
    );
}

/// `Player.dropItemCheck` (`Player.cs:5107-5156`) — the drop/throw key pass,
/// one call per fixed player tick at the `Player.cs:25214-25217` site
/// (`itemAnimation == 0 && ItemTimeIsZero && reuseDelay == 0`).
///
/// The port's folds (ANALYSIS "S1-P-i51"):
/// - the guide/reforge `GetOrDropItem` returns (`:5118-5127`) ride the shop
///   close seam (`App::close_shop`) already;
/// - `inventory[58] = Main.mouseItem.Clone()` (`:5130`) is the port's single
///   `App.mouse_item` — no second copy exists;
/// - the `tileTargetX/Y` cursor refresh (`:5132-5140`) feeds the smart-cursor
///   pass, not the drop spawn (which rides `base.Center`, `:5199`);
/// - the closed-inventory cursor return (`:5141-5146`) runs inline below
///   (`GetOrDropItem`'s inventory fill + the overflow spawn).
///
/// `right_click_drop` is the `Main.mouseRight && !mouseInterface &&
/// Main.mouseRightRelease` arm of the `:5147` gate — the C# draw latch
/// (`Main.cs:62458-62465`) makes it the right PRESS edge; the port reads
/// `place_down && !place_prev_down` outside every slot hit (the
/// `mouseInterface` stand-in).
pub(crate) fn drop_item_check(app: &mut App, right_click_drop: bool) {
    // The dead-player skip is the port's fold of the C# call sites: the
    // local `Player.Update` reaches `:25214` only on the alive path
    // (`UpdateDead` returns first), and death returns the cursor to the
    // inventory FIRST (`KillMe`, `Player.cs:38937-38944`) — so a dead
    // player neither throws nor drops, and the noThrow timer does not tick.
    let alive = app.player.as_ref().is_some_and(|p| !p.dead);
    if !alive {
        return;
    }
    // `:5109-5112`: a closed inventory zeroes noThrow...
    if !app.inventory_open {
        app.no_throw = 0;
    }
    // `:5113-5116`: ...then the timer counts down one per check.
    if app.no_throw > 0 {
        app.no_throw -= 1;
    }
    // `:5141-5146` — a closed inventory returns the cursor stack first
    // (`GetOrDropItem(Main.mouseItem, ReturnItemFromSlot)`,
    // `Player.cs:7193-7200`): the inventory takes what fits; the overflow
    // spawns at `base.Center` under `ReserveForLocalPlayer` — no velocity
    // argument, so `Item.NewItem`'s scatter draws run (the port's pinned
    // [`spawn_item_drop_px`] triplet+prefix order). SP arms no reservation
    // (`WorldItem.cs:300`'s `netMode != 0` gate), so the overflow keeps the
    // port's `PICKUP_DELAY` stand-in like every other loot spawn. The fill
    // plays the `IsACoin ? 38 : 7` chime (`GetItem_FillIntoOccupiedSlot`
    // `Player.cs:39491` — `ReturnItemFromSlot` is NOT NoSound,
    // `GetItemSettings.cs:59`), once per pickup event (the port's
    // merged-fill convention), on the `UpdatePlayers` stream.
    if app.mouse_item.kind > 0 && !app.inventory_open {
        let it = app.mouse_item;
        let max_stack = crate::registry::max_stack(it.kind);
        let mut pending = it.count;
        while pending > 0 {
            let Some(slot) = slot_for_pickup(&mut app.inventory, it.kind, it.prefix) else {
                break;
            };
            let room = max_stack.saturating_sub(app.inventory[slot].count);
            if room == 0 {
                break;
            }
            let take = pending.min(room);
            app.inventory[slot].count += take;
            pending -= take;
        }
        if it.count > pending {
            app.audio.play_pickup_sound(
                crate::registry::is_coin(it.kind),
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        }
        if pending > 0 {
            let (cx, cy) = app.player.as_ref().unwrap().center();
            spawn_item_drop_px(
                &mut app.items,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                it.kind,
                pending,
                Vec2::new(cx - crate::item::SIZE * 0.5, cy - crate::item::SIZE * 0.5),
            );
        }
        app.mouse_item = crate::item::EMPTY_SLOT;
    }
    // The `controlThrow && releaseThrow` latch read (`:5147`): the C#
    // updates `releaseThrow` at the tick TAIL (`:25248-25255`), so the check
    // reads LAST tick's latch — the edge fires on the press tick only.
    let throw_edge =
        app.control_throw && app.player.as_ref().is_some_and(|p| p.release_throw);
    let held = app.inventory[app.selected];
    let throw_arm = throw_edge
        && !app.is_favorite(app.selected)
        && held.kind > 0
        && held.count > 0
        && !app.chat.open; // `!Main.drawingPlayerChat`
    let cursor_arm = (right_click_drop || !app.inventory_open)
        && app.mouse_item.kind > 0
        && app.mouse_item.count > 0;
    // `:5147`'s `(noThrow <= 0 || (controlThrow && releaseThrow))` tail: the
    // throw key bypasses the NPC-hover noThrow window, the cursor arm waits.
    if (throw_arm || cursor_arm) && (app.no_throw <= 0 || throw_edge) {
        drop_selected_item(app, right_click_drop);
    }
}

/// `Player.DropSelectedItem()` (`Player.cs:5158-5168`): the cursor stack
/// first (slot 58), else the selected hotbar slot. The
/// `inventoryChestStack[selectedItem]` gate (`:5164`) has no port state
/// (the quick-stack-to-chest feature is unported) and folds open.
pub(crate) fn drop_selected_item(app: &mut App, right_click_drop: bool) {
    if app.mouse_item.kind > 0 && app.mouse_item.count > 0 {
        drop_selected_item_slot(app, true, right_click_drop);
    } else {
        drop_selected_item_slot(app, false, right_click_drop);
    }
}

/// `Player.DropSelectedItem(int slot, ref Item)` (`Player.cs:5170-5217`).
/// `cursor` selects the slot-58 (`Main.mouseItem`) arm; the favorited guard
/// (`:5173-5184`) returns the item to the inventory instead of dropping it.
fn drop_selected_item_slot(app: &mut App, cursor: bool, right_click_drop: bool) {
    let slot = if cursor { 58 } else { app.selected };
    let mut item = if cursor {
        app.mouse_item
    } else {
        app.inventory[app.selected]
    };
    // The favorited guard: the hotbar arm's `:5147` gate already refused a
    // favorited selected slot; the cursor stack cannot carry the flag (the
    // port's slot-positional favourites, `App::favorite_slots`), so the
    // `GetItem`-return half (`:5175-5183`) folds to "nothing to do".
    if !cursor && app.is_favorite(slot) {
        return;
    }
    // `:5188-5194` — the right-click/closed-inventory SWAP: the slot item
    // goes to the cursor and the cursor stack is what drops.
    let mut swapped_in = crate::item::EMPTY_SLOT;
    if (right_click_drop || !app.inventory_open)
        && app.mouse_item.kind > 0
        && app.mouse_item.count > 0
    {
        swapped_in = item;
        item = app.mouse_item;
        // `delayUseItem = true; controlUseItem = false` (`:5192-5193`): the
        // port's use gate is the swing/itemTime pair — the drop's own
        // `SetItemAnimation(10)` below covers the window.
    }
    // `:5195-5198` — Lucy the Axe's ThrownAway message: no port chat-bubble
    // system (ANALYSIS fold).
    // `:5199` — the spawn itself.
    let (cx, cy) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((app.camera.x, app.camera.y));
    let (vx_player, facing) = app
        .player
        .as_ref()
        .map(|p| (p.vx, p.facing))
        .unwrap_or((0.0, 1));
    let velocity = Vec2::new(4.0 * facing as f32 + vx_player, -2.0);
    // `ItemID.Sets.DropWithGrabDelayForAllPlayers = CreateBoolSet(267)`
    // (`ItemID.cs:430`): the doll delays EVERY player, everything else the
    // local one only (`Player.cs:5199`).
    let grab_delay_player = if item.kind == 267 {
        crate::item::GRAB_DELAY_ALL
    } else {
        crate::item::GRAB_DELAY_LOCAL
    };
    spawn_player_drop(
        &mut app.items,
        item.kind,
        item.count,
        item.prefix,
        Vec2::new(cx, cy),
        velocity,
        grab_delay_player,
    );
    // `:5200-5204` — the dropped slot airs out.
    if cursor {
        app.mouse_item = crate::item::EMPTY_SLOT;
    } else {
        app.inventory[app.selected] = crate::item::EMPTY_SLOT;
    }
    // `:5205-5209` — the swap-back reads the cursor AFTER the drop:
    // `((mouseRight && !mouseInterface) || !playerInventory) &&
    // Main.mouseItem.type > 0` → `theItemWeDrop = item; Main.mouseItem =
    // new Item()`: the hotbar slot KEEPS its own item and the cursor airs
    // out (the cursor stack was what dropped). The C# restores `item` even
    // when it is air (no kind gate). The slot-58 arm aired the cursor at
    // `:5203`, so its gate is false and the animation runs.
    if !cursor
        && (right_click_drop || !app.inventory_open)
        && app.mouse_item.kind > 0
    {
        app.inventory[app.selected] = swapped_in;
        app.mouse_item = crate::item::EMPTY_SLOT;
    } else {
        // `:5212-5214` — `SetItemAnimation(10)` (`Player.cs:4443-4451`:
        // itemAnimation = itemAnimationMax = 10) + `JustDroppedAnItem`.
        app.swing = 10.0 / 60.0;
        app.swing_max = 10.0 / 60.0;
        app.swing_style = 0;
        if let Some(p) = app.player.as_mut() {
            p.just_dropped_an_item = true;
        }
        drop_interrupt_actions(app);
    }
}

/// `Player.DropSelectedItem_InterruptActionsThatUseAnimations`
/// (`Player.cs:5227-5245`): the held projectile dies, then every owned
/// aiStyle-61/160 projectile (the fishing bobbers `AI_061_FishingBobber`
/// `Projectile.cs:53658` and the kites `AI_160_Kites` `:48421`). The port
/// has no `heldProj` field and neither family is ported yet, so the sweep
/// is the shape-keeper (it fires the day a bobber/kite lands).
fn drop_interrupt_actions(app: &mut App) {
    let owner = app.local_id as u8;
    let doomed: Vec<usize> = app
        .projectiles
        .slots()
        .iter()
        .enumerate()
        .filter(|(_, pr)| {
            pr.active
                && pr.owner == owner
                && crate::projectile::proj_data(pr.kind)
                    .is_some_and(|d| d.ai_style == 61 || d.ai_style == 160)
        })
        .map(|(i, _)| i)
        .collect();
    for i in doomed {
        app.projectiles.kill(i);
    }
}

/// `Item.RequestNewItem` → `Item.NewItem` (`Item.cs:49639-49771`) with the
/// player drop's EXPLICIT velocity (`Player.cs:5199`): the `velocity.HasValue`
/// arm (`Item.cs:49738-49741`) skips the scatter draws entirely, and
/// `item.Prefix(prefix)` with the carried prefix draws nothing either
/// (`Item.cs:511-514`: only `-1/-2` roll) — a player drop is DRAW-FREE.
///
/// `center` is `base.Center` — the WorldItem's CENTER, so the port's
/// top-left box sits at `center - SIZE/2` (`WorldItem.Center` setter).
/// The ownership arms `grabDelayTime = DefaultGrabDelay` for the named
/// player (`WorldItem.ApplySpawnOwnership`, `:306-313`); the
/// `enemyGrabDelayTime` twin (`Item.cs:49757-49760`) gates only the monster
/// grab (`WorldItem.cs:405`), unported.
fn spawn_player_drop(
    items: &mut Vec<ItemDrop>,
    kind: u16,
    count: u32,
    prefix: u16,
    center: Vec2,
    velocity: Vec2,
    grab_delay_player: u8,
) {
    if count == 0 {
        // `Item.NewItem`'s `stack <= 0` early return (`Item.cs:49660-49663`).
        return;
    }
    let mut drop = ItemDrop::new(
        kind,
        count,
        prefix,
        center.x - crate::item::SIZE * 0.5,
        center.y - crate::item::SIZE * 0.5,
        velocity.x,
        velocity.y,
        0.0,
    );
    drop.apply_grab_delay(grab_delay_player);
    items.push(drop);
    if items.len() > 300 {
        items.remove(0);
    }
}

/// `Item.NewItem`'s heart/star seasonal remap (`Item.cs:49668-49714`),
/// S1-P-i52. The C# branch order: `tenthAnniversaryWorld` first (ONE
/// `NextFromList` = `Next(len)` draw per remapped kind, `Utils.cs:1213-1216`),
/// else halloween AND xMas (one `Next(2)`), else halloween alone (58→1734,
/// 184→1735 — no draw) then xMas alone (58→1867, 184→1868 — no draw). Every
/// other kind passes through untouched and draws NOTHING. The flags ride
/// the port's standing seasonal stand-ins (`Main.halloween`/`Main.xMas` =
/// the endless micro seeds, `world_sim/npcs.rs`'s loot-flag precedent; the
/// calendar windows are unported).
pub fn seasonal_pickup_kind(
    kind: u16,
    tenth_anniversary: bool,
    halloween: bool,
    xmas: bool,
    rng: &mut UnifiedRandom,
) -> u16 {
    if tenth_anniversary {
        // `:49668-49678` — the two independent `if`s, list order verbatim.
        if kind == 58 {
            return [1734u16, 1867, 58][rng.next_max(3) as usize];
        }
        if kind == 184 {
            return [1735u16, 1868, 184][rng.next_max(3) as usize];
        }
    } else if halloween && xmas {
        // `:49679-49689`.
        if kind == 58 {
            return [1734u16, 1867][rng.next_max(2) as usize];
        }
        if kind == 184 {
            return [1735u16, 1868][rng.next_max(2) as usize];
        }
    } else {
        // `:49692-49713` — the deterministic remaps, no draws.
        if halloween {
            if kind == 58 {
                return 1734;
            }
            if kind == 184 {
                return 1735;
            }
        }
        if xmas {
            if kind == 58 {
                return 1867;
            }
            if kind == 184 {
                return 1868;
            }
        }
    }
    kind
}

#[cfg(test)]
mod shimmer_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::shimmer;
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::world::World;

    /// A 40x40 world with a stone floor at y = 20 and a shimmer pool on top of
    /// it (y = 19), the player standing beside it.
    fn shimmer_world() -> App {
        let mut w = World::new("sh".into(), 40, 40);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        // The floor row 20 (solid) with the two-row shimmer pool 18-19 on top
        // (a drop resting in the liquid has shimmer both in its box and in the
        // tile above it — `CanShimmerAtPosition`'s `position.Y/16 - 1`,
        // `WorldItem.cs:762`).
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            for y in 18..=19 {
                let j = w.index(x, y);
                w.tiles[j] = Tile::default();
                w.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
                w.tiles[j].liquid = 255;
            }
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 9.0 * TILE_PX;
        player.y = 17.0 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(9.0 * TILE_PX, 17.0 * TILE_PX);
        app
    }

    /// A drop resting on the pool's floor, the 16x16 box overlapping the
    /// shimmer row (pos y = 19*16, flush on the 20-row floor), parked at
    /// x = 30 tiles — outside the 42 px pickup magnet of the player at x = 9
    /// (`ItemUpdate`'s magnet, `item.rs:176-188`) but inside the coin-luck
    /// 1000 px window.
    fn resting_drop(kind: u16, count: u32) -> ItemDrop {
        let mut d = ItemDrop::new(
            kind,
            count,
            0,
            30.0 * TILE_PX,
            19.0 * TILE_PX,
            0.0,
            0.0,
            0.0,
        );
        d.vel = Vec2::ZERO;
        d
    }

    #[test]
    fn a_drop_in_shimmer_transforms_on_the_ninetieth_tick() {
        let mut app = shimmer_world();
        app.items.push(resting_drop(11, 3)); // Iron Ore → Tin Ore (the set)
        // The ramp is the C#'s `+= 0.01f` accumulation crossing >0.9
        // (`WorldItem.cs:518-536`); the f32 sum crosses on the 91st ramping
        // tick. W4-e19: the faithful `WetCollision` surface offset
        // (`Collision.cs:1610-1612`, liquid 255 → surface +0.0625 px) means
        // the resting drop's centre-shrunk probe first overlaps on tick 2
        // (the naive full-box probe overlapped on tick 1), so the ramp starts
        // one tick later — the transform lands ~tick 92. The exact start tick
        // is probe-geometry, not the ramp arithmetic; pin the outcome + the
        // 91-tick ramp duration loosely.
        let mut ticks = 0;
        while !app.items[0].shimmered && ticks < 120 {
            update_items(&mut app, 1.0 / 60.0);
            ticks += 1;
        }
        if std::env::var("SHIMMER_TRACE").is_ok() {
            eprintln!("transformed after {ticks} ticks");
        }
        assert!((90..=93).contains(&ticks), "the >0.9 crossing tick count: {ticks}");
        let d = &app.items[0];
        assert!(d.shimmered, "the transform fired");
        assert_eq!(d.kind, 699); // Tin Ore
        assert_eq!(d.count, 3); // the stack is kept
    }

    #[test]
    fn the_ramp_decays_out_of_shimmer() {
        let mut app = shimmer_world();
        // Park the drop above the pool (y = 12), so it is dry.
        app.items.push(ItemDrop::new(
            11,
            1,
            0,
            30.0 * TILE_PX,
            12.0 * TILE_PX,
            0.0,
            0.0,
            0.0,
        ));
        // In shimmer it ramps; the C#'s `shimmerWet` must be set for the ramp.
        for _ in 0..3 {
            let d = &mut app.items[0];
            d.shimmer_wet = true;
            d.shimmer_time = 0.5;
        }
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items[0].shimmer_time < 0.5, "the dry tick decays");
    }

    #[test]
    fn the_coin_merge_multiplies_and_airs_out() {
        let mut app = shimmer_world();
        app.items.push(resting_drop(72, 4)); // Silver coins → x100
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        // The coin arm airs the drop out (`WorldItem.cs:1711-1712`).
        assert!(app.items.is_empty(), "the coin drop vanished");
        // The luck got the merged stack (4 * 100) within 1000 px of the player.
        let luck = app.player.as_ref().unwrap().luck_state.coin_luck;
        assert_eq!(luck, 400.0);
    }

    #[test]
    fn a_decraftable_item_spawns_its_ingredients() {
        let mut app = shimmer_world();
        app.items.push(resting_drop(3077, 60)); // Silk Rope x30 (recipe 62)
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        // The decraft spawns one item of 2 units (amount 2 x the 1-unit
        // ingredient — `WorldItem.cs:1808-1823`'s maxStack chunking) and airs
        // the source out (60 - 2*30).
        let spawns: Vec<&ItemDrop> = app.items.iter().filter(|d| d.kind == 225).collect();
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].count, 2);
        assert!(!app.items.iter().any(|d| d.kind == 3077), "source gone");
    }

    /// DW-4.1 (the items pass) + DW-4.4: the decraft arm's draws (the
    /// ingredient spawns' velocity/prefix, plus the alchemy discount when the
    /// recipe is one) ride the `UpdateItems` stream (`Main.cs:18024`'s
    /// `UpdateItems` swap) and every other stage stream stays pristine; two
    /// same-seed runs replay byte-equal (the pair).
    #[test]
    fn test_dw_4_1_items_pass_draws_the_update_items_stream() {
        let run = || {
            let mut app = shimmer_world();
            app.items.push(resting_drop(3077, 60)); // Silk Rope x30 (recipe 62)
            for _ in 0..120 {
                update_items(&mut app, 1.0 / 60.0);
            }
            let drops: Vec<(u16, u32)> =
                app.items.iter().map(|d| (d.kind, d.count)).collect();
            let streams: Vec<_> = crate::app::RngKey::ALL
                .iter()
                .map(|k| (k.name(), app.rngs.stream(*k).clone()))
                .collect();
            (drops, streams)
        };
        let a = run();
        let b = run();
        // The pair: byte-equal drops and stream states (DW-4.4).
        assert_eq!(a.0, b.0, "the decraft replayed byte-equal");
        assert_eq!(a.1, b.1, "every stream replayed identically");
        // The decraft ran and drew the items stream.
        assert!(a.0.iter().any(|(k, _)| *k == 225), "the decraft ran");
        assert_ne!(
            a.1.iter().find(|(n, _)| *n == "UpdateItems").unwrap().1,
            UnifiedRandom::new(0),
            "UpdateItems drew"
        );
        // The isolation: no other stage stream moved (flow_app's seed is 0,
        // so pristine == the seed state).
        for (name, state) in &a.1 {
            if *name == "UpdateItems" {
                continue;
            }
            assert_eq!(*state, UnifiedRandom::new(0), "{name} stayed pristine");
        }
    }

    #[test]
    fn a_non_transformable_item_never_transforms() {
        let mut app = shimmer_world();
        app.items.push(resting_drop(2, 5)); // Dirt Block — no transform
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        let d = &app.items[0];
        assert!(!d.shimmered);
        assert_eq!(d.count, 5);
        // S1-E-18e — a drop the C# never hands to `GetShimmered`
        // (`CanShimmer()` false) credits nothing.
        assert!(!progression(&app, 27), "no transmute, no TRANSMUTE_ITEM");
    }

    /// S1-E-18e — `WorldItem.cs:1849`'s shared tail notifies progression 27
    /// (TRANSMUTE_ITEM) after any arm that reaches it, the decraft included.
    #[test]
    fn s1_e_18e_a_shimmer_transmute_credits_transmute_item() {
        let mut app = shimmer_world();
        app.items.push(resting_drop(3077, 60)); // Silk Rope — decraftable
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        assert!(app.items.iter().any(|d| d.kind == 225), "the decraft ran");
        assert!(
            progression(&app, 27),
            "the shimmer tail credits TRANSMUTE_ITEM"
        );
    }

    /// Whether the `PROGRESSION_EVENT_<id>` condition (`AchievementCondition
    /// .Name`, the save key) is completed on any row.
    fn progression(app: &App, id: u32) -> bool {
        let key = format!("PROGRESSION_EVENT_{id}");
        app.achievements
            .list
            .iter()
            .flat_map(|a| a.conditions.iter())
            .any(|c| c.name == key && c.completed)
    }

    #[test]
    fn the_shimmered_item_floats_without_gravity() {
        let mut app = shimmer_world();
        let mut d = resting_drop(11, 3);
        d.shimmered = true;
        d.shimmer_wet = true;
        d.vel = Vec2::ZERO;
        app.items.push(d);
        let before = app.items[0].pos.y;
        update_items(&mut app, 1.0 / 60.0);
        // The rise: `velocity.Y -= 0.05` while over shimmer (`WorldItem.cs:474-483`).
        assert!(app.items[0].pos.y < before, "the item rises");
    }

    // --- W4-e19: the shared tail + the decraft spawn shape ----------------

    #[test]
    fn w4_e19_the_transform_tail_cuts_velocity_and_resets_shimmer_time() {
        // `WorldItem.cs:1835-1845` — after the transform the surviving drop
        // gets shimmerTime = 1, shimmerWet = true, velocity *= 0.1.
        let mut app = shimmer_world();
        let mut d = resting_drop(11, 3);
        d.vel = Vec2::new(2.0, 3.0);
        app.items.push(d);
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        let d = &app.items[0];
        assert!(d.shimmered);
        assert_eq!(d.kind, 699);
        // The tail set shimmerTime = 1 at the transform tick (the stack
        // survived). The decay arm (`:538-545`) is NOT gated on `!shimmered`
        // (a shimmered drop's ramp condition is false, so it falls to the
        // decay), so the 1.0 dries 0.01/tick over the remaining ~28 ticks —
        // it is below 1 but still positive (the GrabItems gate re-opens at 0).
        assert!(d.shimmer_time < 1.0 && d.shimmer_time > 0.0,
            "the tail reset then decayed: {}", d.shimmer_time);
        assert!(d.shimmer_wet, "the tail set shimmerWet");
        // The velocity was cut ×0.1 at the transform tick (2.0→0.2, 3.0→0.3),
        // then the shimmered float damps/rises it; the X friction (×0.95/tick)
        // decays 0.2 toward 0. The cut is verified by the magnitude bound.
        assert!(d.vel.x.abs() <= 0.2, "the ×0.1 cut bound: {}", d.vel.x);
    }

    #[test]
    fn w4_e19_the_transform_spawns_the_shimmer_effect_dust() {
        // `WorldItem.cs:1846` → `ShimmerEffect` (`:1669-1688`): 20 dust 309.
        let mut app = shimmer_world();
        app.items.push(resting_drop(11, 3));
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        let n309 = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 309).count();
        assert!(n309 >= 20, "the ShimmerEffect burst: {n309} dust 309");
    }

    #[test]
    fn w4_e19_the_decraft_spawns_shimmered_ingredients_at_the_center() {
        // `WorldItem.cs:1825` → `SpawnShimmeredItem` (`:628-637`): the
        // ingredient spawns shimmered (shimmerTime 1, shimmerWet) at the
        // source's Centre with the alternating fan velocity, prefix 0.
        let mut app = shimmer_world();
        app.items.push(resting_drop(3077, 60)); // Silk Rope → 2 Cobweb units
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        let spawn = app.items.iter().find(|d| d.kind == 225).expect("the cobweb spawn");
        assert!(spawn.shimmered, "the ingredient is shimmered");
        assert!(spawn.shimmer_wet, "SpawnShimmeredItem sets shimmerWet");
        assert_eq!(spawn.prefix, 0, "SpawnShimmeredItem passes prefix 0");
        // The first spawn (n=0) has fan velocity (0 * (0%2*2-1), 0) = (0, 0);
        // the X friction keeps 0 at 0 (the shimmered float only writes vel.Y).
        assert_eq!(spawn.vel.x, 0.0, "the n=0 fan has no horizontal velocity");
        // The source aired out (60 - 2*30 = 0).
        assert!(!app.items.iter().any(|d| d.kind == 3077));
    }

    #[test]
    fn w4_e19_the_coin_merge_does_not_recalculate_luck_eagerly() {
        // `Player.cs:18090-18101` — AddCoinLuck sets coinLuck + luckNeedsSync
        // only; RecalculateLuck runs on the per-tick UpdateLuck, NOT here.
        // The port's add_coin_luck adds the merged stack without rebuilding
        // `luck` (the old apply called recalculate_luck eagerly).
        let mut app = shimmer_world();
        app.items.push(resting_drop(72, 4)); // Silver → 400 copper-equivalent
        let luck_before = app.player.as_ref().unwrap().luck_state.luck;
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.luck_state.coin_luck, 400.0, "the merged stack fed coinLuck");
        // `luck` is rebuilt by the luck pass, not the coin arm — the items
        // pass alone does not touch it (no UpdateLuck in update_items).
        assert_eq!(p.luck_state.luck, luck_before, "no eager RecalculateLuck");
    }

    /// W4-e19 deferred 4 — the entry splash (`WorldItem.cs:881-958`): a dry
    /// drop falling into the shimmer pool arms `wetCount = 20` and bursts 10
    /// dust 308 (the `Next(6)` colour switch) through the shared pool, all on
    /// the `UpdateItems` stream. The camera parks at the pool so the spawn
    /// cull passes.
    #[test]
    fn w4_e19_the_shimmer_entry_splash_bursts_10_dust_308() {
        let mut app = shimmer_world();
        // Above the pool (rows 18-19), dry, falling — parked at x = 30 tiles
        // (outside the player's grab magnet).
        app.items.push(ItemDrop::new(
            2, 1, 0, 30.0 * TILE_PX, 16.0 * TILE_PX, 0.0, 2.0, 0.0,
        ));
        app.camera = Vec2::new(30.0 * TILE_PX, 18.0 * TILE_PX);
        let mut entered = false;
        for _ in 0..30 {
            update_items(&mut app, 1.0 / 60.0);
            if app.items[0].wet {
                entered = true;
                break;
            }
        }
        assert!(entered, "the drop reached the pool");
        let n308 = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        assert!(n308 >= 10, "the entry burst: {n308} dust 308");
        // The cooldown armed 20 then decremented to 19 on the entry tick.
        assert_eq!(app.items[0].wet_count, 19, "wetCount 20 - 1");
        // The colours ride the four-colour switch only.
        for d in app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308) {
            assert!(
                matches!(
                    (d.color.r, d.color.g, d.color.b),
                    (255, 255, 210) | (190, 245, 255) | (255, 150, 255) | (190, 175, 255)
                ),
                "the Next(6) palette: {:?}",
                d.color
            );
            assert!(d.no_gravity, "the splash dust floats");
        }
    }

    /// The exit splash (`:959-1035`): a wet drop lifted out of the pool
    /// bursts again once the cooldown expired — the shimmer exit arm reads
    /// the OLD shimmerWet (still set at the edge) and plays style 5.
    #[test]
    fn w4_e19_the_shimmer_exit_splash_after_the_cooldown() {
        let mut app = shimmer_world();
        // Resting IN the pool (the entry burst fires on tick 1).
        app.items.push(resting_drop(2, 1));
        app.camera = Vec2::new(30.0 * TILE_PX, 18.0 * TILE_PX);
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items[0].wet, "the entry latched");
        let after_entry = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        // Let the cooldown expire (20 ticks), then lift the drop dry.
        for _ in 0..25 {
            update_items(&mut app, 1.0 / 60.0);
        }
        assert_eq!(app.items[0].wet_count, 0, "the cooldown drained");
        app.items[0].pos.y = 10.0 * TILE_PX; // high above the pool — dry
        update_items(&mut app, 1.0 / 60.0);
        assert!(!app.items[0].wet, "the exit cleared wet");
        let after_exit = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        assert!(after_exit > after_entry, "the exit burst added dust 308");
        assert_eq!(app.items[0].wet_count, 19, "the exit re-armed the cooldown");
    }

    /// The cooldown suppresses re-entry churn (`:883`/`:962`): a drop that
    /// exits and re-enters within 20 ticks bursts ONCE.
    #[test]
    fn w4_e19_the_wet_count_suppresses_the_second_burst() {
        let mut app = shimmer_world();
        app.items.push(resting_drop(2, 1));
        app.camera = Vec2::new(30.0 * TILE_PX, 18.0 * TILE_PX);
        update_items(&mut app, 1.0 / 60.0); // entry burst
        let n0 = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        // Out and back in within the cooldown window.
        app.items[0].pos.y = 10.0 * TILE_PX;
        update_items(&mut app, 1.0 / 60.0); // exit edge, wetCount > 0 → no burst
        app.items[0].pos.y = 19.0 * TILE_PX;
        update_items(&mut app, 1.0 / 60.0); // re-entry, wetCount > 0 → no burst
        let n1 = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        assert_eq!(n1, n0, "the cooldown suppressed both edges");
    }

    /// The water entry arm (`:930-939`): 10 `dustWater()` dust with the
    /// scale MULTIPLY (×0.8) and alpha 100 — the water family, not 308.
    #[test]
    fn w4_e19_the_water_entry_splash_uses_the_water_style_dust() {
        let mut app = shimmer_world();
        // Replace the shimmer pool with water.
        for x in 0..40 {
            for y in 18..=19 {
                let j = app.world.index(x, y);
                app.world.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Water);
            }
        }
        app.items.push(ItemDrop::new(
            2, 1, 0, 30.0 * TILE_PX, 16.0 * TILE_PX, 0.0, 2.0, 0.0,
        ));
        app.camera = Vec2::new(30.0 * TILE_PX, 18.0 * TILE_PX);
        for _ in 0..30 {
            update_items(&mut app, 1.0 / 60.0);
            if app.items[0].wet {
                break;
            }
        }
        assert!(app.items[0].wet, "the drop reached the water");
        let ws = crate::biome::calculate_water_style(&app.world, app.camera) as i32;
        let want = tr_core::particles::dust_water(ws);
        let hits: Vec<_> = app.dust.slots().iter().filter(|d| d.active && d.dust_type == want).collect();
        assert!(hits.len() >= 10, "the water burst: {} dust {want}", hits.len());
        for d in hits {
            assert_eq!(d.alpha, 100, "the water arm sets alpha 100");
            assert!(d.no_gravity);
        }
    }
}

#[cfg(test)]
mod rand_light_witness {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::player::Player;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::world::World;

    /// A dry 40x40 world, stone floor at y = 20, the player parked at x = 10
    /// tiles — the drops sit at x = 28..32 tiles, outside the pickup magnet.
    fn dry_world() -> App {
        let mut w = World::new("rl".into(), 40, 40);
        for x in 0..40 {
            for y in 20..40 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 10.0 * TILE_PX;
        player.y = 18.0 * TILE_PX;
        app.player = Some(player);
        app
    }

    fn resting(kind: u16, tx: f32) -> ItemDrop {
        let mut d = ItemDrop::new(kind, 1, 0, tx * TILE_PX, 19.0 * TILE_PX, 0.0, 0.0, 0.0);
        d.vel = Vec2::ZERO;
        d
    }

    /// S1-E-5 witness: the flicker arms draw the `UpdateItems` stream in
    /// `Main.item` slot order — exactly one `Next(90, 111)` per flicker drop,
    /// interleaved with the draw-free skips (116, 501, a plain block) — and
    /// the lights land in the shared `EntityLights` list with the C#
    /// products. Every other stage stream stays pristine.
    #[test]
    fn the_flicker_arms_draw_the_item_stream_in_slot_order() {
        let mut app = dry_world();
        // Slot order: 520 (draw), 116 (skip), 521 (draw), 501 (skip),
        // 58 (draw), 2 (no arm), 3456 (essScale, draw-free).
        for (i, kind) in [520u16, 116, 521, 501, 58, 2, 3456].iter().enumerate() {
            app.items.push(resting(*kind, 28.0 + i as f32));
        }
        *app.rngs.stream(RngKey::UpdateItems) = UnifiedRandom::new(4242);
        update_items(&mut app, 1.0 / 60.0);
        // The twin: three Next(90,111) draws, in slot order, nothing else.
        let mut twin = UnifiedRandom::new(4242);
        let d0 = twin.next_range(90, 111);
        let d1 = twin.next_range(90, 111);
        let d2 = twin.next_range(90, 111);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            twin,
            "exactly the three flicker draws, in slot order"
        );
        // The lights: swap the list pair, then read the frame's sources.
        app.entity_lights.end_frame();
        let lights = app.entity_lights.visible();
        assert_eq!(lights.len(), 4, "3 flickers + 1 fragment");
        let n0 = d0 as f32 * 0.01 * 1.0; // essScale seeds 1.0
        assert_eq!(lights[0].rgb, [0.5 * n0, 0.1 * n0, 0.25 * n0]);
        let n1 = d1 as f32 * 0.01 * 1.0;
        assert_eq!(lights[1].rgb, [0.25 * n1, 0.1 * n1, 0.5 * n1]);
        let n2 = d2 as f32 * 0.01 * (1.0f32 * 0.5);
        assert_eq!(lights[2].rgb, [0.5 * n2, 0.1 * n2, 0.1 * n2]);
        // The fragment arm: no draw, the fixed Vector3 at essScale 1.0.
        assert_eq!(lights[3].rgb, [0.2, 0.4, 0.5]);
        // The isolation: no other stage stream moved (flow_app seeds 0).
        for k in RngKey::ALL {
            if k == RngKey::UpdateItems {
                continue;
            }
            assert!(
                !app.rngs.is_created(k) || *app.rngs.stream(k) == UnifiedRandom::new(0),
                "{} stayed pristine",
                k.name()
            );
        }
    }

    /// The muted path: a world with only draw-free kinds spends nothing on
    /// the item stream across many ticks.
    #[test]
    fn a_world_without_flicker_kinds_stays_draw_free() {
        let mut app = dry_world();
        for (i, kind) in [116u16, 501, 2, 3456, 3459].iter().enumerate() {
            app.items.push(resting(*kind, 28.0 + i as f32));
        }
        *app.rngs.stream(RngKey::UpdateItems) = UnifiedRandom::new(9);
        for _ in 0..30 {
            update_items(&mut app, 1.0 / 60.0);
            // One frame per tick, like the draw pass's swap.
            app.entity_lights.end_frame();
        }
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            UnifiedRandom::new(9),
            "the skipped arms and the fragment arms draw nothing"
        );
        // The fragments still light every tick (RNG-free).
        assert_eq!(app.entity_lights.visible().len(), 2);
    }
}

#[cfg(test)]
mod spawn_rng_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use tr_core::UnifiedRandom;

    /// DW-4.2 (with DW-4.1): `Item.NewItem`'s draws — the velocity/phase
    /// triplet then the `Prefix(-1)` roll — ride the CALLER's stage stream as
    /// one stream (`Item.cs:49654-49727`'s `Main.rand`, `Item.cs:527`'s
    /// `Prefix`); the port's pinned draw order stays, the stream joins. Four
    /// draws exactly.
    #[test]
    fn the_spawn_draws_ride_the_callers_stream() {
        let mut app = flow_app(None, vec![], vec![]);
        *app.rngs.stream(RngKey::UpdateItems) = UnifiedRandom::new(4242);
        // The twin: the same four draws replayed directly.
        let mut expect = UnifiedRandom::new(4242);
        let _ = (expect.next_float(), expect.next_float(), expect.next_float());
        let expected_prefix = crate::prefix::roll_prefix(1, -1, &mut expect);

        let rng = app.rngs.stream(RngKey::UpdateItems);
        spawn_item_drop_px(&mut app.items, rng, 1, 1, Vec2::new(5.0, 5.0));

        assert_eq!(app.items[0].prefix, expected_prefix);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            expect,
            "four draws exactly: the triplet, then the prefix roll"
        );
    }

    /// The revenge coin-suck's faithful share shape
    /// (`WorldItem.cs:1157-1178`): `Next(50, 76) * 0.01f` plus the copper (71)
    /// `Next(51)` and silver (72) `Next(26)` extras on the `UpdateItems`
    /// stream — a copper stack spends exactly two draws. S1-P-i50: the arm
    /// is EXPERT-ONLY (`WorldItem.cs:408-410`), so the fixture arms
    /// `buffs.expert_mode`; the classic-mode twin draws nothing.
    #[test]
    fn the_revenge_share_draws_next_50_76_plus_the_coin_extras() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = crate::test_world_helpers::ground();
        w.compute_column_surface();
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.buffs.expert_mode = true; // `Main.expertMode` (`:408`)
        app.player = Some(player);
        let (px, py) = app.player.as_ref().unwrap().center();
        // A copper coin stack at the player's feet (the pickup magnet range).
        let mut drop = ItemDrop::new(71, 10, 0, px, py, 0.0, 0.0, 0.0);
        drop.age = 1.0;
        // A hostile NPC straddling the drop's box (lifeMax > 5, not
        // friendly/immortal/statue-spawned, hitbox intersection `:1155`):
        // its center pinned to the drop's.
        let mut zombie = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        zombie.hp = zombie.max_hp;
        let (zw, zh) = zombie.size();
        let dc = drop.center();
        zombie.x = dc.x - zw * 0.5;
        zombie.y = dc.y - zh * 0.5;
        app.npcs.push(zombie);
        app.items.push(drop);
        *app.rngs.stream(RngKey::UpdateItems) = UnifiedRandom::new(4242);
        update_items(&mut app, 1.0 / 60.0);
        // The coin was (partly) sucked into the NPC's extraValue.
        let sucked = app.npcs[0].extra_value;
        assert!(sucked > 0, "the revenge share ran");
        assert!(sucked <= 10 * registry::coin_value(71), "at most the stack");
        // The twin: the share draws (Next(50,76), then copper's Next(51)),
        // then S1-E-21f's moneyPing case-38 pair (Next(5) slot, Next(-40,41)
        // pitch — `NPC.cs:90054`, the same UpdateItems stream) when the
        // revenge arm ran.
        let mut expect = UnifiedRandom::new(4242);
        if sucked > 0 {
            expect.next_range(50, 76);
            expect.next_max(51);
            expect.next_range(0, 5);
            expect.next_range(-40, 41);
        }
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            expect,
            "the share pair + the ping pair when sucked, else draw-free"
        );
        // The classic-mode twin: the arm never runs (`:408`'s expert gate),
        // the whole stack rides the player pickup, zero revenge draws.
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = crate::test_world_helpers::ground();
        w.compute_column_surface();
        app.world = w;
        app.player = Some(Player::spawn(&app.world)); // expert_mode false
        let (px, py) = app.player.as_ref().unwrap().center();
        let mut drop = ItemDrop::new(71, 10, 0, px, py, 0.0, 0.0, 0.0);
        drop.age = 1.0;
        let mut zombie = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        zombie.hp = zombie.max_hp;
        let (zw, zh) = zombie.size();
        let dc = drop.center();
        zombie.x = dc.x - zw * 0.5;
        zombie.y = dc.y - zh * 0.5;
        app.npcs.push(zombie);
        app.items.push(drop);
        *app.rngs.stream(RngKey::UpdateItems) = UnifiedRandom::new(4242);
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.npcs[0].extra_value, 0, "classic mode: no suck");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            UnifiedRandom::new(4242),
            "the classic-mode revenge arm is draw-free"
        );
    }
}

#[cfg(test)]
mod lava_burn_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemDrop;
    use crate::npc::{Npc, NpcKind};
    use tr_core::tile::{FLAG_ACTIVE, LiquidKind, Tile};
    use tr_core::world::World;

    /// A hell world with the lava pool at rows 498-499, columns 190-210
    /// (the `hell_world` shape from the voodoo tests).
    fn hell_world() -> World {
        let mut w = World::new("hell".into(), 400, 700);
        for x in 0..w.width {
            for y in 500..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 190..210 {
            for y in 498..500 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
                w.tiles[i].liquid = 255;
                w.tiles[i].set_liquid_kind(LiquidKind::Lava);
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn rare0_drops_burn_in_lava_and_the_immune_set_survives() {
        // `CheckLavaDeath`'s non-doll arm: the rare-0 drop vanishes, the
        // lava-immune set (318 = the Obsidian Skin-ish row family, 173-175
        // = the ores...) and the rare-1+ drops survive.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = hell_world();
        let mut player = crate::player::Player::spawn(&app.world);
        // Far from the lava pool (S1-P-i47): `GrabItems` runs BEFORE the
        // item physics in the C# too — a player standing over the drops
        // would pick them up before they burn.
        player.x = 150.0 * TILE_PX;
        player.y = 496.0 * TILE_PX;
        app.player = Some(player);
        // The rare-0 dirt block: burns.
        app.items.push(ItemDrop::new(
            2,
            1,
            0,
            200.0 * TILE_PX,
            498.0 * TILE_PX,
            0.0,
            0.0,
            0.0,
        ));
        // The lava-immune obsidian (173): survives.
        app.items.push(ItemDrop::new(
            173,
            1,
            0,
            201.0 * TILE_PX,
            498.0 * TILE_PX,
            0.0,
            0.0,
            0.0,
        ));
        // A rare-1+ drop: survives (the rarity gate).
        let rare_item = (200..500)
            .find_map(|id| registry::get(id).filter(|d| d.rare >= 1))
            .expect("a rare row");
        app.items.push(ItemDrop::new(
            rare_item.id,
            1,
            0,
            202.0 * TILE_PX,
            498.0 * TILE_PX,
            0.0,
            0.0,
            0.0,
        ));
        let before = app.items.len();
        update_items(&mut app, 1.0 / 60.0);
        assert!(
            !app.items.iter().any(|it| it.kind == 2),
            "the rare-0 dirt burned"
        );
        assert!(
            app.items.iter().any(|it| it.kind == 173),
            "the immune obsidian survived"
        );
        assert!(
            app.items.iter().any(|it| it.kind == rare_item.id),
            "the rare drop survived"
        );
        let _ = before;
    }

    #[test]
    fn the_doll_still_triggers_the_sacrifice() {
        // The voodoo arm rides the same lava read (M4-BOSS-6c): the doll
        // does NOT burn silently — the sacrifice runs (the Guide dies, the
        // wall spawns).
        let mut app = flow_app(None, vec![], vec![]);
        app.world = hell_world();
        let mut player = crate::player::Player::spawn(&app.world);
        // Out of grab range (S1-P-i47) — the doll must reach the lava, not
        // the player's inventory.
        player.x = 150.0 * TILE_PX;
        player.y = 496.0 * TILE_PX;
        app.player = Some(player);
        let mut guide = Npc::new(NpcKind::Guide, 196.0 * TILE_PX, 496.0 * TILE_PX, 1, 0.0);
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        app.items.push(ItemDrop::new(
            267,
            1,
            0,
            200.0 * TILE_PX,
            498.0 * TILE_PX,
            0.0,
            0.0,
            0.0,
        ));
        update_items(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs
                .iter()
                .all(|n| n.kind != NpcKind::Guide || n.hp <= 0),
            "the sacrifice ran"
        );
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::WallOfFlesh),
            "the wall awoke"
        );
    }
}

#[cfg(test)]
mod pickup_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::{ItemDrop, EMPTY_SLOT};
    use crate::player::Player;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::world::World;

    fn pickup_world() -> App {
        let mut w = World::new("pickup".into(), 40, 40);
        for x in 0..40 {
            for y in 20..40 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 10.0 * TILE_PX;
        player.y = 18.0 * TILE_PX;
        app.player = Some(player);
        app.inventory = std::array::from_fn(|_| EMPTY_SLOT);
        app
    }

    fn collectable_drop(kind: u16, count: u32, x: f32, y: f32) -> ItemDrop {
        let mut d = ItemDrop::new(kind, count, 0, x, y, 0.0, 0.0, 0.0);
        d.age = 1.0;
        d
    }

    #[test]
    fn pickup_transfers_the_stack_once() {
        // `S1-C-9`: the drop must add `remaining` (S1-C-9) exactly once to
        // the slot (`Player.cs:39330`, the `deposit_pickup` walk).
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(3, 5, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the drop was consumed");
        let slot = app
            .inventory
            .iter()
            .find(|s| s.kind == 3)
            .expect("stone landed in inventory");
        assert_eq!(slot.count, 5, "the stack transferred once");
        assert_eq!(app.inventory.iter().filter(|s| s.kind == 3).count(), 1);
    }

    #[test]
    fn pickup_sends_only_the_overflow_to_the_vault_remainder_stays_in_world() {
        // `Player.cs:39395-39398` / `GetItem_VoidVault` (S1-P-i47): the vault
        // is GetItem's LAST resort — the inventory passes run first. The
        // grid is full here, so the vault takes what fits (3) and the
        // leftover 2 stays in the WORLD drop (`PickupItem`'s `ReplaceWith`,
        // `:35103`), never an over-full slot.
        let mut app = pickup_world();
        let mut player = app.player.take().unwrap();
        player.void_vault_enabled = true;
        let max_stack = crate::registry::max_stack(3);
        for slot in player.banks[3].iter_mut() {
            *slot = crate::item::ItemSlot {
                kind: 2,
                name: crate::registry::name(2),
                count: max_stack,
                prefix: 0,
            };
        }
        player.banks[3][0] = crate::item::ItemSlot {
            kind: 3,
            name: crate::registry::name(3),
            count: max_stack.saturating_sub(3),
            prefix: 0,
        };
        app.player = Some(player);
        // The grid is full of dirt — no inventory pass accepts the stone.
        for s in app.inventory.iter_mut() {
            *s = crate::item::ItemSlot {
                kind: 2,
                name: crate::registry::name(2),
                count: crate::registry::max_stack(2),
                prefix: 0,
            };
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(3, 5, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        let in_vault: u32 = app.player.as_ref().unwrap().banks[3]
            .iter()
            .filter(|s| s.kind == 3)
            .map(|s| s.count)
            .sum();
        let in_inv: u32 = app
            .inventory
            .iter()
            .filter(|s| s.kind == 3)
            .map(|s| s.count)
            .sum();
        assert_eq!(in_vault, max_stack, "the vault took exactly 3");
        assert_eq!(in_inv, 0, "the full grid took nothing");
        assert_eq!(app.items.len(), 1, "the surplus stays in the world");
        assert_eq!(app.items[0].count, 2, "5 - 3 = 2 left on the ground");
    }

    /// S1-P-i47 — `GrabItems`' pull half (`Player.cs:34924-34974`): a drop
    /// inside the grab rectangle but outside the hitbox is PULLED, not
    /// teleported — `PullItem_Common`'s 0.45/tick acceleration, the
    /// `being_grabbed` raw-velocity tick (`WorldItem.cs:425-435`), then the
    /// direct-hitbox pickup a few ticks later.
    #[test]
    fn the_grab_rectangle_pulls_before_the_hitbox_collects() {
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        // 30 px to the right of the player's centre: inside the 42 px
        // expanded rectangle, outside the 20x42 body.
        let mut d = ItemDrop::new(3, 5, 0, pc.0 + 30.0, pc.1, 0.0, 0.0, 0.0);
        d.vel = Vec2::ZERO;
        app.items.push(d);
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "not collected on the pull tick");
        // The pull: X accelerates -0.45 toward the player (left), the
        // grabbed tick applies it raw.
        let d = &app.items[0];
        assert!(d.vel.x < 0.0, "pulled left, vel {:?}", d.vel);
        assert!((d.vel.x + 0.45).abs() < 1e-5, "one 0.45 step, got {}", d.vel.x);
        // A few dozen ticks later the drop reaches the body and lands in
        // the inventory (the fall to the floor keeps it in the rectangle).
        for _ in 0..120 {
            update_items(&mut app, 1.0 / 60.0);
        }
        assert!(app.items.is_empty(), "the pull ended in a pickup");
        assert_eq!(
            app.inventory.iter().filter(|s| s.kind == 3).map(|s| s.count).sum::<u32>(),
            5
        );
    }

    /// S1-P-i47 — the grab delay (`WorldItem.cs:26`/`:110`,
    /// `Player.cs:34911`): a `GrabDelayForLocalPlayer` drop refuses the
    /// grabber for 100 ticks, then collects normally.
    #[test]
    fn the_grab_delay_locks_the_drop_for_100_ticks() {
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        let mut d = ItemDrop::new(3, 5, 0, pc.0 - 4.0, pc.1 - 4.0, 0.0, 0.0, 0.0);
        d.apply_grab_delay(app.local_id as u8);
        app.items.push(d);
        for tick in 0..99 {
            update_items(&mut app, 1.0 / 60.0);
            assert_eq!(app.items.len(), 1, "still locked at tick {}", tick);
        }
        // Tick 100: the delay hit 0 on the 100th update — the NEXT grab
        // pass collects (the decrement runs after the grab gate).
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items[0].grab_delay_time, 0);
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the lock expired");
        assert_eq!(
            app.inventory.iter().filter(|s| s.kind == 3).map(|s| s.count).sum::<u32>(),
            5
        );
    }

    /// S1-P-i47 — `ItemSpace` gates the PULL (`Player.cs:34930-34934`): a
    /// full inventory (and no vault) leaves the drop lying — no pull, no
    /// velocity.
    #[test]
    fn a_full_inventory_refuses_the_pull() {
        let mut app = pickup_world();
        for s in app.inventory.iter_mut() {
            *s = crate::item::ItemSlot {
                kind: 2,
                name: crate::registry::name(2),
                count: crate::registry::max_stack(2),
                prefix: 0,
            };
        }
        let pc = app.player.as_ref().unwrap().center();
        let mut d = ItemDrop::new(3, 5, 0, pc.0 + 30.0, pc.1, 0.0, 0.0, 0.0);
        d.vel = Vec2::ZERO;
        app.items.push(d);
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1);
        assert!(!app.items[0].being_grabbed, "no pull without room");
        assert_eq!(app.items[0].vel.x, 0.0, "the velocity stayed untouched");
    }

    /// S1-P-i47 — `Player.cs:27426-27428`: a dead player grabs nothing.
    #[test]
    fn a_dead_player_grabs_nothing() {
        let mut app = pickup_world();
        app.player.as_mut().unwrap().dead = true;
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(3, 5, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "the death drop lies until respawn");
        assert!(app.inventory.iter().all(|s| s.count == 0));
    }

    /// S1-P-i47 — the world stack merge (`WorldItem.TryCombiningIntoNearbyItems`
    /// `:157-186`) inside the items pass: two wood drops within 30 px
    /// combine into one, away from the player.
    #[test]
    fn nearby_world_drops_merge_in_the_items_pass() {
        let mut app = pickup_world();
        // Park the player far away (x = 10 tiles); the drops at x = 30.
        let mut a = ItemDrop::new(9, 10, 0, 30.0 * TILE_PX, 19.0 * TILE_PX, 0.0, 0.0, 0.0);
        a.vel = Vec2::ZERO;
        let mut b = ItemDrop::new(9, 7, 0, 30.0 * TILE_PX + 10.0, 19.0 * TILE_PX, 0.0, 0.0, 0.0);
        b.vel = Vec2::ZERO;
        app.items.push(a);
        app.items.push(b);
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "the source drained into the survivor");
        assert_eq!(app.items[0].count, 17);
        assert_eq!(app.items[0].kind, 9);
    }

    /// S1-P-i47 — the ammo routing end-to-end: an arrow drop lands in the
    /// ammo rows (`FillAmmo`, `Player.cs:39264-39328`), not the grid.
    #[test]
    fn an_arrow_drop_lands_in_the_ammo_rows() {
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(40, 12, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty());
        assert_eq!(app.equip.ammo[0].kind, 40);
        assert_eq!(app.equip.ammo[0].count, 12);
        assert!(app.inventory.iter().all(|s| s.count == 0), "the grid stayed out");
    }

    // --- S1-P-i50: the coin pickup routing + DoCoins ----------------------

    /// `Player.GetItem`'s isACoin arm (`Player.cs:39340-39344`): a coin
    /// drop fills the coin rows 50-53, NOT the hotbar/grid — the occupied
    /// merge first, then the empty fill DESCENDING (row 53 first).
    #[test]
    fn s1_p_i50_a_coin_drop_fills_the_coin_rows_not_the_grid() {
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(71, 30, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the coin was collected");
        // Empty coin rows fill descending: row 53 (index 3) takes it.
        assert_eq!(app.equip.coins[3].kind, 71);
        assert_eq!(app.equip.coins[3].count, 30);
        assert!(
            app.inventory.iter().all(|s| s.kind != 71),
            "no coin landed in the grid/hotbar"
        );
        // The purse mirror credits the copper value.
        assert_eq!(app.coins, 30, "30 copper into the purse mirror");
    }

    /// The occupied coin-row merge (`num2 = -4`, ascending): a partial row
    /// 50 soaks the drop before any empty row is touched.
    #[test]
    fn s1_p_i50_a_coin_drop_merges_into_a_partial_coin_row_first() {
        let mut app = pickup_world();
        app.equip.coins[0] = crate::item::ItemSlot {
            kind: 71,
            name: crate::registry::name(71),
            count: 40,
            prefix: 0,
        };
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(71, 30, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.equip.coins[0].count, 70, "40 + 30 into row 50");
        assert_eq!(app.equip.coins[3].count, 0, "no empty row touched");
    }

    /// `DoCoins` on pickup (`Player.cs:39503`/`:39569`): a drop that fills a
    /// coin row to exactly 100 converts it to one of the next denomination.
    #[test]
    fn s1_p_i50_pickup_converts_a_full_coin_row_via_do_coins() {
        let mut app = pickup_world();
        app.equip.coins[0] = crate::item::ItemSlot {
            kind: 71,
            name: crate::registry::name(71),
            count: 95,
            prefix: 0,
        };
        // The mirror starts where the pre-seeded stacks' value stands (the
        // retry fix: the pickup credits only what the DROP carries — the
        // pre-existing 95 copper are already the mirror's, the C# has no
        // separate purse at all).
        app.coins = 95;
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(71, 5, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        // 95 + 5 = 100 copper → 1 silver in row 50.
        assert_eq!(app.equip.coins[0].kind, 72, "the row converted to silver");
        assert_eq!(app.equip.coins[0].count, 1);
        // The mirror credits the drop's 5 copper on top of the seeded 95.
        assert_eq!(app.coins, 100, "the mirror is value-based, not stack-based");
    }

    /// The 100:1 chain across denominations: 100 silver → 1 gold, and a
    /// platinum row never converts (`DoCoins`' guard excludes 74).
    #[test]
    fn s1_p_i50_the_conversion_chain_and_the_platinum_floor() {
        let mut app = pickup_world();
        app.equip.coins[1] = crate::item::ItemSlot {
            kind: 72,
            name: crate::registry::name(72),
            count: 99,
            prefix: 0,
        };
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(72, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.equip.coins[1].kind, 73, "100 silver → gold");
        assert_eq!(app.equip.coins[1].count, 1);
        // Platinum: a full 9999 row stays platinum (no type 75).
        let mut app = pickup_world();
        app.equip.coins[2] = crate::item::ItemSlot {
            kind: 74,
            name: crate::registry::name(74),
            count: 9998,
            prefix: 0,
        };
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(74, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.equip.coins[2].kind, 74, "platinum never converts");
        assert_eq!(app.equip.coins[2].count, 9999);
    }

    /// S1-P-i50 (retry) — the immediate per-fill `DoCoins` ordering
    /// (`Player.cs:39503`/`:39519` fire INSIDE the occupied pass, before the
    /// empty pass `:39387-39394`): a 55-copper drop onto row 50 = 95 copper
    /// + row 51 = 99 silver (rows 52/53 occupied by platinum) fills row 50
    /// to 100 → converts to 1 silver → MERGES into row 51 (99 + 1 = 100
    /// silver → 1 gold), airing row 50 — and the 50-copper remainder then
    /// refills the AIRED row 50 in the descending empty pass (53/52/51 all
    /// occupied, 50 the first air). A deferred post-sweep would park the
    /// remainder in the GRID instead (row 50 still full at empty-pass time)
    /// — the placement differs, so the conversion must run per fill,
    /// exactly the C#.
    #[test]
    fn s1_p_i50_the_immediate_do_coins_lets_the_empty_pass_refill_the_aired_row() {
        let mut app = pickup_world();
        app.equip.coins[0] = crate::item::ItemSlot {
            kind: 71,
            name: crate::registry::name(71),
            count: 95,
            prefix: 0,
        };
        app.equip.coins[1] = crate::item::ItemSlot {
            kind: 72,
            name: crate::registry::name(72),
            count: 99,
            prefix: 0,
        };
        // Rows 52/53 occupied (platinum never merges with copper) so the
        // descending empty pass reaches the aired row 50 first.
        for k in [2usize, 3] {
            app.equip.coins[k] = crate::item::ItemSlot {
                kind: 74,
                name: crate::registry::name(74),
                count: 1,
                prefix: 0,
            };
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(71, 55, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the whole 55-stack was placed");
        // Row 51: 99 silver + the converted 1 → 100 silver → 1 gold.
        assert_eq!(app.equip.coins[1].kind, 73, "the chain reached gold");
        assert_eq!(app.equip.coins[1].count, 1);
        // Row 50: aired by the merge, then REFILLED by the empty pass with
        // the 50-copper remainder (the descending scan's first air).
        assert_eq!(app.equip.coins[0].kind, 71, "the remainder refilled row 50");
        assert_eq!(app.equip.coins[0].count, 50);
        assert!(
            app.inventory.iter().all(|s| s.kind != 71),
            "the deferred sweep would have parked it in the grid"
        );
        // The purse mirror: the full 55 copper value.
        assert_eq!(app.coins, 55);
    }

    /// The Gold Ring's grab range on the world pickup (`GetItemGrabRange`,
    /// `Player.cs:35133`): a coin 200 px out is collected with the ring,
    /// ignored without it. The drop sits in free air (above the floor) so
    /// the pull is not fought by ground friction.
    #[test]
    fn s1_p_i50_the_gold_ring_collects_a_far_coin_in_the_world() {
        // With the ring: the coin 200 px away is pulled and (over ticks)
        // collected.
        let mut app = pickup_world();
        app.player.as_mut().unwrap().gold_ring = true;
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(71, 5, pc.0 + 200.0, pc.1 - 60.0));
        for _ in 0..240 {
            update_items(&mut app, 1.0 / 60.0);
            if app.items.is_empty() {
                break;
            }
        }
        assert!(app.items.is_empty(), "the ring pulled the far coin in");
        assert!(
            app.equip.coins.iter().any(|s| s.kind == 71 && s.count == 5),
            "the coin landed in the coin rows"
        );
        // Without the ring: 200 px is beyond the base 42 — never collected.
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(71, 5, pc.0 + 200.0, pc.1 - 60.0));
        for _ in 0..240 {
            update_items(&mut app, 1.0 / 60.0);
        }
        assert_eq!(app.items.len(), 1, "the base magnet never reaches it");
    }
    // ---- S1-P-i52 — the heart/star pickup arms (written, NOT run) ----

    /// `Player.PickupItem`'s heart arm (`Player.cs:35059-35064`) + `Heal`
    /// (`:35112-35123`): the drop heals 20 capped at `statLifeMax2`, the
    /// HealLife text carries the RAW 20 (`HealEffect` runs BEFORE the cap),
    /// the drop airs out and NEVER enters the inventory (`IsAPickup`,
    /// `ItemID.cs:493` — the `:35059` arm precedes the `GetItem` walk).
    /// All three heart kinds (58/1734/1867) ride the same arm.
    #[test]
    fn s1_p_i52_the_heart_heals_twenty_capped_and_never_enters_inventory() {
        for kind in crate::item::HEART_KINDS {
            let mut app = pickup_world();
            {
                let p = app.player.as_mut().unwrap();
                p.max_hp = 100;
                p.hp = 95; // the cap eats 15 of the 20
            }
            let pc = app.player.as_ref().unwrap().center();
            app.items.push(collectable_drop(kind, 1, pc.0 - 4.0, pc.1 - 4.0));
            update_items(&mut app, 1.0 / 60.0);
            assert!(app.items.is_empty(), "kind {kind}: the heart aired out");
            assert_eq!(app.player.as_ref().unwrap().hp, 100, "kind {kind}: capped");
            assert!(
                app.inventory.iter().all(|s| s.count == 0),
                "kind {kind}: nothing entered the inventory"
            );
            // The HealLife text: RAW 20, the (100, 255, 100) colour.
            let t = app
                .combat_texts
                .iter()
                .find(|t| t.active)
                .expect("the heal text spawned");
            assert_eq!(t.text, "20");
            assert_eq!(t.color, crate::combat_text::HEAL_LIFE);
        }
    }

    /// The full-life heart is STILL consumed (`ItemSpace`'s `IsAPickup`
    /// short-circuit, `Player.cs:39155-39158` — no `statLife` gate exists in
    /// the grab chain): the heal caps at the max, the text still prints 20,
    /// the drop vanishes. This pins the census verdict — the vanilla does
    /// NOT leave a heart on the ground at full life.
    #[test]
    fn s1_p_i52_the_full_life_heart_is_consumed_not_refused() {
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.max_hp = 100;
            p.hp = 100;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(58, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the heart is consumed at full life");
        assert_eq!(app.player.as_ref().unwrap().hp, 100);
        assert_eq!(
            app.combat_texts.iter().find(|t| t.active).unwrap().text,
            "20",
            "the RAW amount prints even when the cap eats all of it"
        );
    }

    /// `Player.PickupItem`'s star arm (`Player.cs:35065-35079`): +100 mana
    /// capped at `statManaMax2`, `TryApplyManaHeat(0.2f, 2f)` writes the
    /// latest pair (0.2, 120) and adds buff 396 for 120 ticks
    /// (`:52006-52014`), the HealMana text carries the RAW 100 (the cap
    /// comes after, `:35074-35077`). All three star kinds ride the arm.
    #[test]
    fn s1_p_i52_the_star_restores_hundred_mana_and_arms_mana_heat() {
        for kind in crate::item::STAR_KINDS {
            let mut app = pickup_world();
            {
                let p = app.player.as_mut().unwrap();
                p.mana_max = 200;
                p.mana = 150; // the cap eats 50 of the 100
            }
            let pc = app.player.as_ref().unwrap().center();
            app.items.push(collectable_drop(kind, 1, pc.0 - 4.0, pc.1 - 4.0));
            update_items(&mut app, 1.0 / 60.0);
            assert!(app.items.is_empty(), "kind {kind}: the star aired out");
            let p = app.player.as_ref().unwrap();
            assert_eq!(p.mana, 200, "kind {kind}: capped at statManaMax2");
            assert_eq!(p.latest_mana_potion_damage_bonus, 0.2);
            assert!(
                (p.latest_mana_potion_duration - 120.0).abs() < 1e-3,
                "kind {kind}: 2 s = 120 ticks"
            );
            let i = p.buffs.find_buff_index(396).expect("ManaHeat added");
            assert!(
                (p.buffs.buff_time[i] - 120).abs() <= 1,
                "(int)(2f * 60f) truncates to 120 or 119 (the C# f32 product): {}",
                p.buffs.buff_time[i]
            );
            assert!(
                app.inventory.iter().all(|s| s.count == 0),
                "kind {kind}: nothing entered the inventory"
            );
            let t = app
                .combat_texts
                .iter()
                .find(|t| t.active)
                .expect("the mana text spawned");
            assert_eq!(t.text, "100", "the RAW 100 prints before the cap");
            assert_eq!(t.color, crate::combat_text::HEAL_MANA);
        }
    }

    /// The full-mana star touched directly is consumed (`:34917-34922`'s
    /// intersects arm has no full-mana gate; only the PULL at `:34947`
    /// does): mana stays capped, buff 396 still refreshes.
    #[test]
    fn s1_p_i52_the_full_mana_star_is_consumed_on_touch() {
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 200;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(184, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the star is consumed at full mana");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 200);
        assert!(p.buffs.find_buff_index(396).is_some());
    }

    /// The RNG order inside the heart pickup arm: the case-7 pitch draw
    /// (`LegacySoundPlayer.cs:725`, ONE `Next(-10, 11)`) comes FIRST (the
    /// `PlaySound(7)` at `:35061` precedes `Heal`), then the HealLife
    /// text's TWO placement draws (`CombatText.cs:88-89`). All ride
    /// `UpdatePlayers` (the `GrabItems` caller, the `Main.cs:17754` swap).
    /// The peek-twin witness: a same-seeded stream consumes exactly those
    /// three draws, in that order.
    #[test]
    fn s1_p_i52_the_heart_pickup_draws_ride_update_players_in_order() {
        use tr_core::unified_random::UnifiedRandom;
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.max_hp = 100;
            p.hp = 50;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(58, 1, pc.0 - 4.0, pc.1 - 4.0));
        // Re-seed the named stream so the twin starts from the same state.
        let seed = 0x5152_5354i32;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut witness = UnifiedRandom::new(seed);
        let _pitch = witness.next_range(-10, 11); // case 7 (`:725`)
        // The text placement (`CombatText.cs:88-89`) at the player's box:
        // width 20 → Next(-10, 11); height 42 → Next(-21, 22).
        let _dx = witness.next_range(-10, 11);
        let _dy = witness.next_range(-21, 22);
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek(),
            witness.peek(),
            "exactly the chime + two placement draws, in the C# order"
        );
    }

    /// `Item.NewItem`'s seasonal remap (`Item.cs:49668-49714`): the branch
    /// order and the draw counts. Classic flags: no draw, no remap.
    /// Halloween alone: 58→1734, 184→1735, draw-free. Christmas alone:
    /// 58→1867, 184→1868, draw-free. Both: ONE `Next(2)` per remap.
    /// Anniversary: ONE `Next(3)` per remap, and it wins over the seasons.
    #[test]
    fn s1_p_i52_the_seasonal_remap_branches_and_draw_counts() {
        use tr_core::unified_random::UnifiedRandom;
        // Classic: identity, zero draws.
        let mut rng = UnifiedRandom::new(9);
        let peek = rng.peek();
        assert_eq!(seasonal_pickup_kind(58, false, false, false, &mut rng), 58);
        assert_eq!(seasonal_pickup_kind(184, false, false, false, &mut rng), 184);
        assert_eq!(seasonal_pickup_kind(12, false, true, true, &mut rng), 12);
        assert_eq!(rng.peek(), peek, "the classic + non-pickup arms draw nothing");
        // Halloween alone: deterministic, zero draws.
        let peek = rng.peek();
        assert_eq!(seasonal_pickup_kind(58, false, true, false, &mut rng), 1734);
        assert_eq!(seasonal_pickup_kind(184, false, true, false, &mut rng), 1735);
        assert_eq!(rng.peek(), peek);
        // Christmas alone: deterministic, zero draws.
        assert_eq!(seasonal_pickup_kind(58, false, false, true, &mut rng), 1867);
        assert_eq!(seasonal_pickup_kind(184, false, false, true, &mut rng), 1868);
        assert_eq!(rng.peek(), peek);
        // Both seasons: one Next(2) each, list order {1734, 1867}/{1735, 1868}.
        let mut rng = UnifiedRandom::new(9);
        let mut twin = UnifiedRandom::new(9);
        let h = seasonal_pickup_kind(58, false, true, true, &mut rng);
        assert_eq!(h, [1734u16, 1867][twin.next_max(2) as usize]);
        let s = seasonal_pickup_kind(184, false, true, true, &mut rng);
        assert_eq!(s, [1735u16, 1868][twin.next_max(2) as usize]);
        assert_eq!(rng.peek(), twin.peek(), "exactly two draws");
        // Anniversary wins over both seasons: one Next(3), the C# list
        // order {1734, 1867, 58}/{1735, 1868, 184} (`:49672`/`:49676`).
        let mut rng = UnifiedRandom::new(9);
        let mut twin = UnifiedRandom::new(9);
        let h = seasonal_pickup_kind(58, true, true, true, &mut rng);
        assert_eq!(h, [1734u16, 1867, 58][twin.next_max(3) as usize]);
        let s = seasonal_pickup_kind(184, true, false, false, &mut rng);
        assert_eq!(s, [1735u16, 1868, 184][twin.next_max(3) as usize]);
        assert_eq!(rng.peek(), twin.peek(), "the anniversary arm draws once per kind");
    }
}
#[cfg(test)]
mod drop_witness_tests {
    //! S1-P-i51 — the player drop/throw witnesses (`Player.dropItemCheck`
    //! `Player.cs:5107-5156`, `DropSelectedItem` `:5158-5217`, the
    //! `Item.NewItem` spawn `Item.cs:49654-49771` and the grab-delay model
    //! `WorldItem.cs:306-313`/`:441-443` + `Player.GrabItems` `:34911`).
    //! Written, NOT run (code-only swarm).
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::item::{EMPTY_SLOT, ItemSlot};
    use crate::player::Player;

    /// A grounded app with the player parked at (160, 1900) — the floor is
    /// y = 120 tiles (`test_world_helpers::ground`), so the drop falls free.
    fn drop_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = crate::test_world_helpers::ground();
        w.compute_column_surface();
        app.world = w;
        let mut p = Player::spawn(&app.world);
        p.x = 160.0;
        p.y = 1900.0;
        p.vx = 0.0;
        p.facing = 1;
        app.player = Some(p);
        app.inventory = std::array::from_fn(|_| EMPTY_SLOT);
        app
    }

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, name: crate::registry::name(kind), count, prefix: 0 }
    }

    /// The throw key drops the WHOLE selected stack with the facing
    /// velocity `(4*direction + velocity.X, -2)` (`Player.cs:5199`), at
    /// `base.Center`, and the spawn is DRAW-FREE (the explicit velocity
    /// skips `Item.NewItem`'s scatter, `Item.cs:49738-49741`; the carried
    /// prefix rolls nothing, `:511-514`).
    #[test]
    fn the_throw_key_drops_the_stack_at_facing_velocity_draw_free() {
        let mut app = drop_app();
        app.inventory[0] = slot(2, 37); // Dirt Block x37
        app.selected = 0;
        app.control_throw = true;
        let before = app.rngs.stream(RngKey::UpdatePlayers).clone();
        let (cx, cy) = app.player.as_ref().unwrap().center();
        drop_item_check(&mut app, false);
        // The slot aired out, one drop landed.
        assert_eq!(app.inventory[0].count, 0, "the stack left the slot");
        assert_eq!(app.items.len(), 1);
        let d = &app.items[0];
        assert_eq!(d.kind, 2);
        assert_eq!(d.count, 37, "the WHOLE stack drops, not one");
        // The centre spawn: the box top-left is centre - SIZE/2.
        assert_eq!(d.pos.x, cx - crate::item::SIZE * 0.5);
        assert_eq!(d.pos.y, cy - crate::item::SIZE * 0.5);
        // The facing velocity: 4*1 + 0 = 4, y = -2.
        assert_eq!(d.vel.x, 4.0);
        assert_eq!(d.vel.y, -2.0);
        // DRAW-FREE: the stream did not move.
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            before,
            "the explicit-velocity spawn draws nothing"
        );
        // The drop animation armed (`SetItemAnimation(10)`, `:5212`).
        assert!((app.swing - 10.0 / 60.0).abs() < 1e-6);
        assert!(app.player.as_ref().unwrap().just_dropped_an_item);
    }

    /// The facing flips the throw: left-facing drops at `-4 + velocity.X`.
    #[test]
    fn the_throw_velocity_follows_the_facing_and_the_player_velocity() {
        let mut app = drop_app();
        app.inventory[0] = slot(3, 1);
        app.selected = 0;
        app.control_throw = true;
        {
            let p = app.player.as_mut().unwrap();
            p.facing = -1;
            p.vx = 1.5;
        }
        drop_item_check(&mut app, false);
        assert_eq!(app.items[0].vel.x, 4.0 * -1.0 + 1.5);
        assert_eq!(app.items[0].vel.y, -2.0);
    }

    /// The releaseThrow latch: a HELD key drops once (the press edge), not
    /// every tick — the C# reads last tick's `releaseThrow` (`:5147`) and
    /// refreshes it at the tick tail (`:25248-25255`).
    #[test]
    fn the_throw_key_edges_once_per_press() {
        let mut app = drop_app();
        app.inventory[0] = slot(2, 5);
        app.inventory[1] = slot(3, 5);
        app.selected = 0;
        app.control_throw = true;
        // Tick 1: release_throw starts true → the edge fires.
        drop_item_check(&mut app, false);
        assert_eq!(app.items.len(), 1);
        // The tail refresh (the port's update_player does this): held → false.
        app.player.as_mut().unwrap().release_throw = !app.control_throw;
        // Tick 2: still held, latch false → no second drop.
        app.selected = 1;
        app.swing = 0.0; // the drop animation ran out
        drop_item_check(&mut app, false);
        assert_eq!(app.items.len(), 1, "a held key does not spam drops");
        // Release then re-press: the latch re-arms, the edge fires again.
        app.control_throw = false;
        app.player.as_mut().unwrap().release_throw = !app.control_throw;
        app.control_throw = true;
        drop_item_check(&mut app, false);
        assert_eq!(app.items.len(), 2, "the re-press drops the new slot");
    }

    /// A favorited selected slot refuses the throw (`:5147`'s
    /// `!inventory[selectedItem].favorited`).
    #[test]
    fn a_favorited_slot_refuses_the_throw() {
        let mut app = drop_app();
        app.inventory[0] = slot(2, 5);
        app.selected = 0;
        app.favorite_slots.push(0);
        app.control_throw = true;
        drop_item_check(&mut app, false);
        assert!(app.items.is_empty(), "the favorite stays");
        assert_eq!(app.inventory[0].count, 5);
    }

    /// The no-grab-delay model: the drop arms `grabDelayTime = 100` for the
    /// local player (`WorldItem.cs:306-309`), so the dropping player cannot
    /// re-grab it for 100 ticks even standing on it; the timer decrements
    /// one per tick (`:441-443`) and the grab opens on expiry.
    #[test]
    fn the_drop_arms_a_hundred_tick_local_grab_delay() {
        let mut app = drop_app();
        app.inventory[0] = slot(2, 1);
        app.selected = 0;
        app.control_throw = true;
        drop_item_check(&mut app, false);
        let d = &app.items[0];
        assert_eq!(d.grab_delay_time, crate::item::DEFAULT_GRAB_DELAY);
        assert_eq!(d.grab_delay_player, crate::item::GRAB_DELAY_LOCAL);
        // Rest the drop on the ground (floor y = 120 tiles = 1920 px, the
        // 16 px box sits at 1904) with zero velocity, aged past the 0.35 s
        // stand-in, and park the player's centre ON it — so only the
        // grab-delay gates the pickup, not range or age.
        {
            let d = &mut app.items[0];
            d.vel = Vec2::ZERO;
            d.age = 1.0;
            d.pos = Vec2::new(160.0, 1920.0 - crate::item::SIZE);
        }
        let dc = app.items[0].center();
        {
            let p = app.player.as_mut().unwrap();
            p.x = dc.x - crate::player::WIDTH * 0.5;
            p.y = dc.y - crate::player::HEIGHT * 0.5;
        }
        // `DropSelectedItem`'s `SetItemAnimation(10)` (`Player.cs:5212`)
        // arms the 10-tick drop animation — the direct-pickup gate
        // (`:34919`, held-air + swing-live refuses) blocks the grab while
        // it runs. The fixture clears it (the animation ran out long
        // before the 100-tick delay expires; gate V4: the merged i47 grab
        // pass honours the gate the i51 stand-in magnet never had).
        app.swing = 0.0;
        // The grab pass runs BEFORE the physics decrement (the C# order:
        // GrabItems in UpdatePlayers, the delay tick in UpdateItem), so
        // ticks 1..100 all see delay >= 1 and tick 101 is the first grab.
        for _ in 0..100 {
            update_items(&mut app, 1.0 / 60.0);
            assert!(!app.items.is_empty(), "the delayed drop is not grabbed");
        }
        // Tick 101: tick 100's decrement hit 0, the grab opens.
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the grab opens when the delay expires");
        assert!(app.inventory.iter().any(|s| s.kind == 2 && s.count == 1));
    }

    /// The Guide Voodoo Doll (267) delays EVERY player
    /// (`ItemID.Sets.DropWithGrabDelayForAllPlayers`, `ItemID.cs:430`):
    /// `grabDelayPlayer = 255`.
    #[test]
    fn the_voodoo_doll_delays_all_players() {
        let mut app = drop_app();
        app.inventory[0] = slot(267, 1);
        app.selected = 0;
        app.control_throw = true;
        drop_item_check(&mut app, false);
        assert_eq!(app.items[0].grab_delay_player, crate::item::GRAB_DELAY_ALL);
        assert_eq!(app.items[0].grab_delay_time, crate::item::DEFAULT_GRAB_DELAY);
    }

    /// The cursor stack drops on the throw key too (`DropSelectedItem`
    /// prefers slot 58, `:5160-5163`), and the cursor airs out.
    #[test]
    fn the_cursor_stack_drops_before_the_hotbar() {
        let mut app = drop_app();
        app.inventory_open = true;
        app.inventory[0] = slot(2, 5);
        app.selected = 0;
        app.mouse_item = slot(3, 9); // the cursor carries stone
        app.control_throw = true;
        drop_item_check(&mut app, false);
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 3, "the CURSOR stack dropped");
        assert_eq!(app.items[0].count, 9);
        assert_eq!(app.mouse_item.count, 0, "the cursor aired out");
        assert_eq!(app.inventory[0].count, 5, "the hotbar kept its stack");
    }

    /// A closed inventory returns the cursor stack to the inventory
    /// (`dropItemCheck` `:5141-5146` → `GetOrDropItem`), NOT a world drop,
    /// when it fits. The fill plays the Grab chime — `ReturnItemFromSlot`
    /// is NOT NoSound (`GetItemSettings.cs:59`), ONE `Next(-10, 11)` pitch
    /// draw (`LegacySoundPlayer.cs:725`) on the `UpdatePlayers` stream.
    #[test]
    fn a_closed_inventory_returns_the_cursor_to_the_inventory() {
        let mut app = drop_app();
        app.inventory_open = false;
        app.mouse_item = slot(3, 9);
        let mut expect = app.rngs.stream(RngKey::UpdatePlayers).clone();
        let _ = expect.next_range(-10, 11); // the Grab chime's one pitch draw
        drop_item_check(&mut app, false);
        assert!(app.items.is_empty(), "no world drop when it fits");
        assert_eq!(app.mouse_item.count, 0);
        assert!(app.inventory.iter().any(|s| s.kind == 3 && s.count == 9));
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            expect,
            "the return draws only the Grab chime's pitch"
        );
    }

    /// The right-click world drop (open inventory, cursor stack, press edge
    /// outside every slot): the cursor stack drops and the hotbar slot keeps
    /// its own item (`:5188-5209`'s swap + swap-back).
    #[test]
    fn the_right_click_drops_the_cursor_and_keeps_the_hotbar() {
        let mut app = drop_app();
        app.inventory_open = true;
        app.inventory[0] = slot(2, 5);
        app.selected = 0;
        app.mouse_item = slot(3, 9);
        drop_item_check(&mut app, true); // right_click_drop
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 3, "the cursor stack dropped");
        assert_eq!(app.mouse_item.count, 0, "the cursor aired out");
        assert_eq!(app.inventory[0].count, 5, "the hotbar kept its item");
    }

    /// The dead player neither throws nor drops (`KillMe` returns the cursor
    /// first, `Player.cs:38937-38944`; the alive-only `:25214` call site).
    #[test]
    fn a_dead_player_does_not_throw() {
        let mut app = drop_app();
        app.inventory[0] = slot(2, 5);
        app.selected = 0;
        app.control_throw = true;
        app.player.as_mut().unwrap().dead = true;
        drop_item_check(&mut app, false);
        assert!(app.items.is_empty());
        assert_eq!(app.inventory[0].count, 5);
    }

    /// The chat gate: an open chat refuses the throw (`!Main.drawingPlayerChat`,
    /// `:5147`).
    #[test]
    fn an_open_chat_refuses_the_throw() {
        let mut app = drop_app();
        app.inventory[0] = slot(2, 5);
        app.selected = 0;
        app.control_throw = true;
        app.chat.open = true;
        drop_item_check(&mut app, false);
        assert!(app.items.is_empty());
    }

}
