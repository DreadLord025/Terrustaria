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

/// `NPCID.Sets.CanConvertIntoCopperSlimeTownNPC` (`NPCID.cs:10120`):
/// `CreateBoolSet(1, 302, 335, 336, 333, 334)` — the Blue Slime (1) and the
/// five zombie/skeleton family rows a dropped Copper Shortsword (89) /
/// Chain Knife (3507) converts into the Copper Town Slime
/// (`WorldItem.GetPickedUpByMonsters_Special`, `WorldItem.cs:1105`).
const CAN_CONVERT_COPPER_SLIME: &[u16] = &[1, 302, 335, 336, 333, 334];

/// `WorldItem.GetPickedUpByMonsters_Special` (`WorldItem.cs:1086-1116`,
/// W4-g15 lane 5): the first convertible hostile NPC whose hitbox intersects
/// the drop's consumes it and raises `unlockedSlimeCopperSpawn`. The C#
/// `break`s on the first match (`:1109`); the `NPC.TransformCopperSlime`
/// (`NPC.cs:19160-19182`) target (684 TownSlimeCopper) has no NpcKind row
/// (the town-slime family is unported), so the port raises the flag and
/// leaves the NPC — the documented roster-gap shape (the
/// `spawn_shimmer_release` precedent: the accounting runs, the spawn waits
/// for the roster). The `TownSlimeTransform` particle broadcast is the
/// orchestrator lane's. Returns true when the drop was consumed
/// (`TurnToAirAndSync`, `:1114`). RNG-free.
/// The disjoint-borrow shape (the `DustSink` convention): the scan reads the
/// NPC list only; the caller raises `world.unlocked_slime_copper_spawn`
/// post-tick (the closure holds `world = &app.world`).
fn copper_slime_pickup(npcs: &[crate::npc::Npc], drop: &ItemDrop) -> bool {
    let (dx0, dy0) = (drop.pos.x as i32, drop.pos.y as i32);
    let dsz = crate::item::SIZE as i32;
    // `:1102-1110` — the first intersecting convertible NPC wins.
    npcs.iter().any(|n| {
        let (nw, nh) = n.size();
        n.hp > 0
            && CAN_CONVERT_COPPER_SLIME.contains(&n.kind.cs_id())
            && dx0 < (n.x as i32) + nw as i32
            && dx0 + dsz > n.x as i32
            && dy0 < (n.y as i32) + nh as i32
            && dy0 + dsz > n.y as i32
    })
}

pub(crate) fn update_items(app: &mut App, dt: f32) {
    if app.is_client() {
        // The host owns the drops: ask it to collect whatever lies in our
        // grab rectangle and hide those locally until the next authoritative
        // snapshot. The range is `GetItemGrabRange`'s expanded hitbox
        // (`Player.cs:34926`, S1-P-i47) — the old 26 px circle had no C#
        // counterpart. The client's accessory flags are the local player's
        // LIVE flags (W4-g15 lane 5 — the `GrabFlags` seam now reads them).
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
            // W4-g15 lane 5 — the client's accessory margins read the LIVE
            // local-player flags (the host half below does the same); the
            // old `GrabFlags::default()` refused the Gold Ring / magnet
            // range extensions on the client's optimistic grab.
            let flags = crate::item::GrabFlags {
                gold_ring: p.gold_ring,
                mana_magnet: p.mana_magnet,
                life_magnet: p.life_magnet,
                treasure_magnet: p.treasure_magnet,
                journey_far_range: p.far_placement_range,
            };
            let range = crate::item::get_item_grab_range(it.kind, flags) as f32;
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
    // The splash drain's cull ctx + water style (doc-sweep lane 36,
    // `WorldItem.cs:886-954`/`:965-1033`) — computed BEFORE the mutable
    // field borrows (whole-`app` reads). `Main.waterStyle` is the live
    // biome style the `Dust.dustWater()` arm reads (`Dust.cs:402-418`);
    // the TR_WATER_STYLE pin caps at 14 like the draw pass
    // (`app/frame/draw.rs:734-737`).
    let splash_ctx = app.spawn_ctx();
    let splash_water_style = crate::env::water_style()
        .map(|n| n.min(14) as i32)
        .unwrap_or_else(|| crate::biome::calculate_water_style(&app.world, app.camera) as i32);
    let dust_ctx = app.spawn_ctx();
    let world = &app.world;
    let player = &mut app.player;
    let items = &mut app.items;
    let coins = &mut app.coins;
    let inventory = &mut app.inventory;
    // The coin rows only — the ammo rows stay reachable through `app.equip`
    // (the deposit walk borrows them; a whole-`equip` alias would collide).
    let coin_rows = &mut app.equip.coins;
    let audio = &mut app.audio;
    // The splash arms draw `Main.rand` under the `UpdateItems` swap, so
    // the RNG borrows stay per-call inside the step closure.
    let mut voodoo: Option<(Vec2, u32)> = None;
    // W4-g15 lane 5 — the copper-slime unlock latch: the flag write rides
    // the post-tick pass (the closure holds the world borrow); the latch
    // makes a second coin this tick see the lock the C#'s in-loop write
    // would show.
    let mut copper_unlocked = false;
    // The drops whose shimmer ramp crossed 0.9 this step (the post-pass apply).
    let mut shimmer_due: Vec<u32> = Vec::new();
    // W4-g15 — the dust spawn ctx captured ONCE per tick, before the items
    // borrow (the `world_sim/npcs.rs` dust convention / the
    // `world_sim/player.rs:426` snapshot pattern). The day/remix/DD2 gates
    // the despawn burns read are captured here too (they cannot change
    // mid-tick).
    // 347 §2.1 keep-one — the e19 splash drain's water style wins (the f47b
    // adopt): `Main.waterStyle` is the live biome style the `Dust.dustWater()`
    // arm reads (`Dust.cs:402-418`); the `TR_WATER_STYLE` pin caps at 14 like
    // the draw pass so the splash dust family matches the pinned liquid
    // draws (`app/frame/draw.rs`). The g15 unpinned camera recompute retired.
    let water_style = crate::env::water_style()
        .map(|n| n.min(14) as i32)
        .unwrap_or_else(|| crate::biome::calculate_water_style(&app.world, app.camera) as i32);
    let day_time = app.world.day_time;
    let remix_world = app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    let dd2_ongoing = app.world.dd2_event_ongoing;
    // W4-g15 lane 5 — the TryFillBucket gates (`WorldItem.cs:550-563`): the
    // rain toggle + intensity, the world-surface presence (`Main.cs:2788`'s
    // `worldSurface > 50`), the remix flag and the day rate. Captured once
    // per tick (they cannot change mid-tick).
    let raining = app.rain.is_raining();
    let max_raining = app.rain.max_raining();
    let has_world_surface = app.world.world_surface > 50.0;
    let day_rate = app.world.day_rate();
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
            // nebula trio + the Mana Cloak star 4143 their consume arms
            // (W4-f49 lane 12 / W4-d79). Every `IsAPickup` kind is consumed
            // on touch — none enters the inventory walk. The S1 receipt
            // keeps f49's gate-REMOVED shape (d79's exclusion-list skip
            // gate is the same admission expressed as a skip — dropped).
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
                // W4-g15 lane 5 — the 5010/5126 grants now arm the flag
                // (`player.rs`, `Player.cs:14552`/`:15292`).
                // W4-f49 — the Treasure Magnet 5010 / Hand Of Creation 5126
                // flag (`Player.cs:14551-14553`/`:15292`): +150 on EVERY kind
                // (`:35149-35152`), the range row was already live.
                treasure_magnet: pl.treasure_magnet,
                journey_far_range: pl.far_placement_range,
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
                // W4-f49 lane 12 — `PickupItem`'s Nebula arm FIRST in the
                // C# chain (`:35045-35058`, `ItemID.Sets.NebulaPickup` =
                // 3453/3454/3455, `ItemID.cs:1324`): the POSITIONED
                // `PlaySound(7, (int)position.X, (int)position.Y)` (the
                // player's top-left — this one attenuates against the
                // camera; the heart/star arms are unpositioned), then the
                // `buffType` read + `TurnToAir`, then `NebulaLevelup(buffType)`
                // (the netMode-1 SendData 102 arm is MP-only). The buffType
                // rides the item row (`Item.cs:31008-31016`: 3453→179 Damage,
                // 3454→173 Life, 3455→176 Mana). S1 keep-one: f49's
                // registry-read arm survives (d79's hardcoded-map twin
                // dropped), REWIRED onto d79's `play_grab_sound` — the
                // faithful `:718-722` Stop-then-recreate on the
                // SoundInstanceGrab slot (f49's `_at` emit-only twin
                // retires). The case-7 pitch draw rides `UpdatePlayers`
                // (the `GrabItems` caller, `Main.cs:17754`); `NebulaLevelup`
                // draws nothing (the buff grant is RNG-free).
                if matches!(kind, 3453 | 3454 | 3455) {
                    let buff_kind = crate::registry::get(kind).map_or(0, |d| d.buff_type);
                    // `:35047` — `(int)position.X/(int)position.Y` truncates
                    // (the positioned façade's casts; `as i32 as f32` ≡ trunc).
                    audio.play_grab_sound(
                        px as i32 as f32,
                        py as i32 as f32,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    if let Some(p) = player.as_mut() {
                        p.nebula_levelup(buff_kind);
                    }
                    items.swap_remove(i);
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
                // W4-f49 lane 12 — the Mana Cloak star 4143 (`:35080-
                // 35098`, the S1 keep-one survivor; d79's line-equivalent
                // twin dropped): the UNPOSITIONED `PlaySound(7)` (one
                // `Next(-10, 11)` pitch on `UpdatePlayers` after the gate),
                // `statMana += 50`, the local-player `TryApplyManaHeat(0.2f,
                // 2f)` + `ManaEffect(50)` (the RAW 50 before the cap, the
                // `num2 != 0` gate always true), then `TurnToAir`. NOT in
                // `STAR_KINDS` (`:34942`'s flag2 excludes it — the full-mana
                // pull gate and the Mana Magnet row never apply; the +12
                // grab margin is unconditional, `:35141-35144`). The star
                // SPAWN (the 723 projectile's Kill, `Projectile.cs:75174-
                // 75177`) is the projectile-Kill lane's — the consume arm
                // is complete for any 4143 drop.
                if kind == 4143 {
                    audio.play_pickup_sound(
                        false,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    if let Some(p) = player.as_mut() {
                        p.mana += 50;
                        p.try_apply_mana_heat(0.2, 2.0);
                        let text = "50".to_string();
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
                            50,
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
                    // W4-g15 lane 5 — the `:405` gate row also applies:
                    // `enemyGrabDelayTime == 0 && playerIndexTheItemIsReservedFor
                    // == Main.myPlayer`. The port runs the revenge arm at the
                    // pickup (the standing coin-wallet deviation, ANALYSIS
                    // S1-P-i50), so the reservation is the grab gate's own
                    // `reserved_elsewhere` (already checked above); the
                    // enemy-delay clock gates the monster suck too.
                    let mut drop = items[i].clone();
                    if expert_mode && drop.enemy_grab_delay_time == 0 {
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
                // W4-g56 void-chain item 3 (census): this auto-route IS the
                // C# `GetItem` void-bag arm — `IsVoidVaultEnabled` (the
                // per-tick `HasItem(4131)` refresh, `Player.cs:24550-24553`)
                // gates `vault_enabled`, `CanVoidVaultAccept` the kinds, and
                // `GetItem_VoidVault` (`:39402-39423`) fills occupied then
                // empty. Already landed (S1-P-i43/i47/i50); the e19
                // quicksell/shimmer co-lane keeps these calls untouched. The
                // `DoCoins(i)` inside the vault fills (`:39454`/`:39544`)
                // reads `this.inventory[i]`, NOT the vault array — a vanilla
                // quirk the port records as a deviation (bank.rs:51), not a
                // re-port. No new code: the arm is complete and claimed.
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
                |i| vault.vault_favorites[i],
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
            // is live via GrabFlags; the nebula 12/5 pull below is the S1
            // keep-one survivor (f49's copy, d79's identical twin dropped).
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
                } else if matches!(kind, 3453 | 3454 | 3455) {
                    // `:34953-34956` — `ItemID.Sets.NebulaPickup` →
                    // `PullItem_Pickup(worldItem, 12f, 5)` (W4-f49 lane 12,
                    // the same 12/5 blend as the Mana Magnet star pull).
                    crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 12.0, 5);
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
        // The shimmered-repulsion scan runs PER-DROP below (the e19 KEEP
        // slice form per 379 - the C# scan (WorldItem.cs:790-847) runs inside
        // each drop's MoveInWorld on the LIVE velocities; the f47 whole-list
        // pre-pass is superseded).
        let mut i = 0;
        while i < items.len() {
            let mut drop = items[i].clone();
            // W4-g15 lane 5 — the reservation gate
            // `playerIndexTheItemIsReservedFor != Main.myPlayer`
            // (`WorldItem.cs:683` CheckLavaDeath / `:159` TryCombining):
            // a drop reserved for ANOTHER peer neither burns nor combines.
            // The port's host owns every drop in SP (the map is empty);
            // the MP reservation lives in `net_reserved` (the same read
            // the grab gate above uses).
            // The port's host owns every drop in SP (the map is empty); an
            // MP stream (a non-empty `net_reserved`) defaults an absent slot
            // to the C# 255 (nobody) — also "not us". Both read as
            // `playerIndex != Main.myPlayer` at `:683`/`:159`.
            let reserved_elsewhere = match app.net_reserved.get(&(i as i16)) {
                Some((who, _)) => *who != app.local_id as u8 && *who != 255,
                None => !app.net_reserved.is_empty(),
            };
            // `WorldItem.CheckLavaDeath`'s voodoo arm (`WorldItem.cs:418-421`,
            // `:681-763`): the doll in lava is consumed and the sacrifice
            // runs (the resolve rides the post-tick pass — the closure holds
            // the items borrow).
            if drop.kind == 267 && !reserved_elsewhere {
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
            if in_lava && !reserved_elsewhere && (rare == 0 || rare == -1) && !lava_immune {
                // `TurnToAirAndSync` — the drop vanishes.
                items.swap_remove(i);
                continue;
            }
            let was_grabbed = drop.being_grabbed;
            // W4-g15 lane 5 — `TryFillBucket` (`WorldItem.cs:399-402`,
            // `:548-571`): an empty bucket (205) resting in the rain on a
            // wind-allowing wall at the surface fills into a water bucket
            // (206), keeping its stack. The C# runs it BEFORE UpdateShimmer
            // and MoveInWorld; the two draws (`Next(num3)`, `NextFloat`) ride
            // the `UpdateItems` stream. The gate reads the drop's pre-move
            // position (the C# checks before MoveInWorld).
            if !was_grabbed && drop.kind == 205 && !reserved_elsewhere {
                // The kind swaps to 206 in place; the stack is kept. The drop
                // continues through the rest of the update as a water bucket
                // (the C# mutates `this.inner` in place).
                try_fill_bucket(
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                    &mut drop,
                    world,
                    raining,
                    max_raining,
                    has_world_surface,
                    remix_world,
                    day_rate,
                );
            }
            // W4-g15 lane 5 — `GetPickedUpByMonsters_Special`
            // (`WorldItem.cs:405-407`, `:1086-1116`): a dropped Copper
            // Shortsword (89) / Chain Knife (3507) touching a convertible
            // hostile NPC while the Copper Town Slime is locked is consumed
            // and raises the unlock flag. The C# gate (`:405`) is
            // `enemyGrabDelayTime == 0 && playerIndexTheItemIsReservedFor ==
            // Main.myPlayer`; the reservation is `reserved_elsewhere`. The
            // NPC-side `TransformCopperSlime` (684) is the roster gap — the
            // flag is the observable half (the `spawn_shimmer_release`
            // precedent). The flag write rides the post-tick pass (the
            // closure holds `world = &app.world`); `copper_unlocked` carries
            // the first-grant latch so a second coin this tick sees the lock
            // the C#'s in-loop flag write would show.
            if !was_grabbed
                && (drop.kind == 89 || drop.kind == 3507)
                && !reserved_elsewhere
                && drop.enemy_grab_delay_time == 0
                && !copper_unlocked
                && !app.world.unlocked_slime_copper_spawn
            {
                if copper_slime_pickup(&app.npcs, &drop) {
                    copper_unlocked = true;
                    items.swap_remove(i);
                    continue;
                }
            }
            // W4-g15 — `WorldItem.UpdateShimmer`'s shimmered sparkle
            // (`WorldItem.cs:459-481`) draws `Main.rand` INSIDE UpdateShimmer
            // (`:403`), BEFORE MoveInWorld's splash. The port's `update` is
            // RNG-free, so the sparkle drain runs here first (the C# position),
            // then the movement, then the splash drain below.
            if !was_grabbed && drop.shimmered {
                let mut s = crate::item_dust::DustSink {
                    rng: app.rngs.stream(crate::app::RngKey::UpdateItems),
                    dust: &mut app.dust,
                    world,
                    ctx: &dust_ctx,
                    audio: &mut *audio,
                };
                crate::item_dust::shimmered_sparkle(&mut s, &drop);
            }
            // `MoveInWorld`'s repulsion scan (`WorldItem.cs:790-847`) runs at
            // the head of the movement half, BEFORE the gravity add — the
            // slice-level pass nudges the neighbours' live velocities (and
            // this drop's own X) and parks the `flag` for the ×0.8 tail. A
            // grabbed drop skips MoveInWorld entirely (`:425-435`), no scan.
            // The clone re-reads its X afterwards — the scan wrote the slot.
            // (347 §1: e19's keeper half; the g15 skip confirmed, 358 §5.)
            // `MoveInWorld`'s repulsion scan (`WorldItem.cs:790-847`, the
            // e19 KEEP form per 379) runs at the head of the movement half,
            // BEFORE the gravity add - the slice-level pass nudges the
            // neighbours' live velocities (and this drop's own X) and parks
            // the `flag` for the ?0.8 tail. A grabbed drop skips MoveInWorld
            // entirely (`:425-435`), no scan. The clone re-reads its X
            // afterwards - the scan wrote the slot.
            if !was_grabbed && drop.shimmered {
                drop.repulsion_brake = crate::item::ItemDrop::repel_shimmered(items, i);
                drop.vel.x = items[i].vel.x;
            }
            drop.update(world, step);
            // Doc-sweep lane 36 — the splash arms (`WorldItem.cs:886-954`
            // Doc-sweep lane 36 - the splash arms (`WorldItem.cs:886-954`
            // entry / `:965-1033` exit) run INSIDE MoveInWorld in the C#;
            // the port latches them in `update` and drains here, same tick,
            // same slot order, BEFORE the visual-effects draws (`:436` runs
            // after `:413`'s MoveInWorld). All draws ride `UpdateItems`.
            if drop.splash_due.is_some() {
                drop.splash(
                    &mut app.dust,
                    &splash_ctx,
                    world,
                    audio,
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                    splash_water_style,
                );
            }
            // `MoveInWorld`'s entry/exit splash (`WorldItem.cs:881-1035`):
            // the drop queued the edge; the burst drains HERE, inside the
            // per-drop loop, BEFORE the despawn/visual halves below — the C#
            // draws the dust + the case-19 play inside `MoveInWorld` (`:413`),
            // ahead of `UpdateItem_VisualEffects` (`:436`), all on the
            // `UpdateItems` stream (`Main.cs:18024`'s swap). The g67
            // rain-splash contract: `DustPool::new_dust` + the post-writes.
            // 347 §2.1 keep-one: e19's `ItemSplash`/`take_splash` form; the
            // g15 `splash_edge` drain retired with its marker field.
            if let Some(splash) = drop.take_splash() {
                item_splash(
                    &mut app.dust,
                    &dust_ctx,
                    world,
                    water_style,
                    audio,
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                    splash,
                );
            }
            // W4-g15 — `DespawnIfMeetingConditions` (`WorldItem.cs:423`,
            // `:639-679`): the fallen-star daytime burn / the 4143 expiry /
            // the 3822 DD2 despawn, AFTER MoveInWorld and BEFORE the visual
            // effects. A fired burn airs the drop out (`TurnToAirAndSync`).
            if !was_grabbed {
                let burned = {
                    let mut s = crate::item_dust::DustSink {
                        rng: app.rngs.stream(crate::app::RngKey::UpdateItems),
                        dust: &mut app.dust,
                        world,
                        ctx: &dust_ctx,
                        audio: &mut *audio,
                    };
                    // The C#'s three independent `if`s (`:645`/`:657`/`:665`);
                    // the kinds are mutually exclusive, so at most one fires.
                    let a = crate::item_dust::fallen_star_burn(&mut s, &drop, day_time, remix_world, was_grabbed);
                    let b = crate::item_dust::mana_cloak_star_expiry(&mut s, &drop);
                    let c = crate::item_dust::dd2_energy_despawn(&mut s, &drop, dd2_ongoing);
                    a || b || c
                };
                if burned {
                    items.swap_remove(i);
                    continue;
                }
            // W4-g15 lane 5 — `CheckInWorld` (`WorldItem.cs:422`, `:573-588`):
            // a drop whose position leaves the world (20-tile fluff) is
            // recovered to the spawn if it is a `RecoverableImportantItem`
            // (the boss bags, `ItemID.cs:1386` — the same 21-row set as
            // `BossBag`), else aired out. The C# runs it AFTER CheckLavaDeath
            // and BEFORE DespawnIfMeetingConditions; the port keeps it after
            // the despawn burns (the kinds are disjoint — a boss bag never
            // burns — so the order is observationally identical). The recover
            // target is the world spawn in SP (`Main.LocalPlayer.SpawnX < 0`
            // when no bed is set; the port's `bed_spawn` is the override).
                let ptx = (drop.pos.x as i32) >> 4;
                let pty = (drop.pos.y as i32) >> 4;
                let fluff = 20;
                let in_world = ptx >= fluff
                    && ptx < world.width - fluff
                    && pty >= fluff
                    && pty < world.height - fluff;
                if !in_world {
                    if crate::entity_lights::BOSS_BAG.contains(&drop.kind) {
                        // `:579` — the recover: `Center = spawn.ToWorldCoordinates()`
                        // (the +8/+8 cell centre), `velocity = Zero`. The SP
                        // target is the world spawn; a validated bed overrides.
                        let (sx, sy) = match world.bed_spawn {
                            Some((bx, by))
                                if crate::world_sim::editing::bed_spawn_valid(world, bx, by)
                                    .is_ok() =>
                            {
                                (bx, by)
                            }
                            _ => (world.spawn_x, world.spawn_y),
                        };
                        let cx = sx as f32 * TILE_PX + 8.0;
                        let cy = sy as f32 * TILE_PX + 8.0;
                        drop.pos = Vec2::new(cx - crate::item::SIZE * 0.5, cy - crate::item::SIZE * 0.5);
                        drop.vel = Vec2::ZERO;
                    } else {
                        // `:584` — `TurnToAir`.
                        items.swap_remove(i);
                        continue;
                    }
                }
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
            //
            // W4-g15 — the 116 meteorite arm is now ported (it was deferred
            // here): its flicker + 1/250 dust gate ride `item_dust`. The 501
            // pixie-dust arm is dust-only. The 75 / boss-bag `% 12` sparkles
            // follow the (RNG-free) light halves.
            let ess = app.ess_scale;
            if drop.kind == 116 {
                let light = {
                    let mut s = crate::item_dust::DustSink {
                        rng: app.rngs.stream(crate::app::RngKey::UpdateItems),
                        dust: &mut app.dust,
                        world,
                        ctx: &dust_ctx,
                        audio: &mut *audio,
                    };
                    crate::item_dust::meteorite_visual(&mut s, &drop)
                };
                if let Some((x, y, rgb)) = light {
                    app.entity_lights.add(x, y, rgb);
                }
            } else if let Some((x, y, rgb)) = crate::item_lights::rand_light(
                drop.kind,
                drop.pos,
                crate::item::SIZE,
                ess,
                app.rngs.stream(crate::app::RngKey::UpdateItems),
            ) {
                app.entity_lights.add(x, y, rgb);
            }
            if drop.kind == 501 {
                let mut s = crate::item_dust::DustSink {
                    rng: app.rngs.stream(crate::app::RngKey::UpdateItems),
                    dust: &mut app.dust,
                    world,
                    ctx: &dust_ctx,
                    audio: &mut *audio,
                };
                crate::item_dust::pixie_dust_visual(&mut s, &drop);
            }
            if drop.kind == 75 || crate::entity_lights::BOSS_BAG.contains(&drop.kind) {
                let mut s = crate::item_dust::DustSink {
                    rng: app.rngs.stream(crate::app::RngKey::UpdateItems),
                    dust: &mut app.dust,
                    world,
                    ctx: &dust_ctx,
                    audio: &mut *audio,
                };
                crate::item_dust::fallen_star_sparkle(&mut s, &drop);
                crate::item_dust::boss_bag_sparkle(&mut s, &drop);
            }
            // W4-g15 — `WorldItem.cs:437-440`: the `timeSinceItemSpawned` tick
            // runs AFTER `UpdateItem_VisualEffects` (`:436`), so the sparkle /
            // expiry gates above read the pre-increment value this tick.
            drop.tick_age();
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
                let drained = crate::item::ItemDrop::combine_nearby(items, i, reserved_elsewhere);
                for j in drained.into_iter().rev() {
                    items.swap_remove(j);
                }
            }
            i += 1;
        }
    });
    app.item_step.cap_backlog();
    // W4-g15 lane 5 — the copper-slime unlock flag (`NPC.cs:19171`): the
    // scan consumed the coin(s) inside the closure; the world write lands
    // here (the `Transform(684)` target is the town-slime roster gap).
    if copper_unlocked {
        app.world.unlocked_slime_copper_spawn = true;
    }
    // The voodoo sacrifice resolves after the items borrow drops (the
    // spawn/kill pass takes `app` wholesale).
    if let Some((pos, stack)) = voodoo {
        crate::world_sim::npcs::voodoo_doll_lava_death(app, pos, stack);
    }
    // The shimmer transforms apply in the list order (the C#'s single-item
    // `GetShimmered` per tick; a step fires at most one per drop). They run
    // BEFORE the wish grant: the C#'s `UpdateShimmer` (`:403`) precedes
    // `TryGrantingMakeAWishSet` (`:414`), and the grant's `swap_remove`
    // would invalidate the `shimmer_due` indices.
    crate::shimmer::apply_due_transforms(app, &shimmer_due);
    // W4-g15 lane 5 — the Heroicis-set grant (`TryGrantingMakeAWishSet`): the
    // gate + the five spawns take `app` wholesale, so they resolve here. The
    // C# runs the grant per-drop inside UpdateItem (`:414-417`); the port
    // scans the live list post-tick (the same observable outcome — a wet
    // desert platinum coin grants once; the grant is draw-free so the scan
    // order does not move any RNG stream). A platinum coin in WATER never
    // rides the shimmer ramp (`shimmerWet` refuses), so the two passes are
    // disjoint on the grant's input.
    try_granting_make_a_wish_set(app);
}

/// `WorldItem.TryGrantingMakeAWishSet` (`WorldItem.cs:590-610`, W4-g15
/// lane 5): a platinum coin (74) that is WET (water), a single stack, NOT
/// shimmer-wet, still grab-delayed for a NAMED player (`grabDelayTime > 0 &&
/// grabDelayPlayer != 255` — a player drop, not a tile drop), and whose
/// closest player stands in the DESERT, is consumed and spawns the five
/// Heroicis pieces (5655/5656/5657/5658/5661) as shimmered items, plus the
/// RainbowBoulder cue.
///
/// The gate `playerIndexTheItemIsReservedFor == Main.myPlayer` (`:592`) is
/// the port's `net_reserved` answer (a foreign reservation refuses). The
/// `Player.FindClosest` (`Player.cs:4942-4967`) is the single-session-player
/// scan — the port's one player is slot 0, so the closest is always it; the
/// C#'s `b != byte.MaxValue` guard is vacuous (an all-inactive roster returns
/// 0, never 255), and the port's `app.player.is_none()` is the no-player
/// case. The `ZoneDesert` read is the port's last-scan `scene_metrics`
/// (the standing convention — `update_scene` runs after the items pass).
///
/// `SpawnShimmeredItem` (`WorldItem.cs:623-637`): `Item.RequestNewItem` with
/// the explicit velocity `(itemNumber * (itemNumber % 2 * 2 - 1), 0)` — the
/// alternating ±itemNumber X spread (0, -1, +2, -3, +4 for the five) — and
/// the modifier sets `shimmerTime = 1; shimmered = true; shimmerWet = true;
/// wet = true`. The explicit-velocity arm draws NO scatter (`Item.cs:49738-
/// 49741`), and `prefix: 0` draws nothing (`Item.cs:511-514`) — the spawn is
/// DRAW-FREE, so the port's pinned triplet is skipped entirely.
///
/// The port scans the live list post-tick (the C# runs the grant per-drop
/// inside `UpdateItem`); a granted coin is consumed and its five pieces are
/// appended, so the scan walks the ORIGINAL length only (the pieces are
/// kind != 74 and never re-grant). The grant is draw-free, so the scan order
/// moves no RNG stream.
fn try_granting_make_a_wish_set(app: &mut App) {
    // The desert zone + a live player are the shared gates (`:594-595`);
    // capture them once (they cannot change during the scan).
    if !app.scene_metrics.zone_desert || app.player.is_none() {
        return;
    }
    let n = app.items.len();
    let mut i = 0;
    while i < n {
        let Some(drop) = app.items.get(i) else {
            break;
        };
        if drop.kind != 74 {
            i += 1;
            continue;
        }
        // `:592` — the gate row. `wet` is the water flag (the C# `base.wet`),
        // `shimmerWet` the shimmer-liquid flag; a coin in shimmer is refused.
        let reserved_elsewhere = match app.net_reserved.get(&(i as i16)) {
            Some((who, _)) => *who != app.local_id as u8 && *who != 255,
            None => !app.net_reserved.is_empty(),
        };
        if reserved_elsewhere
            || !drop.wet
            || drop.count != 1
            || drop.shimmer_wet
            || drop.grab_delay_time <= 0
            || drop.grab_delay_player == 255
        {
            i += 1;
            continue;
        }
        let center = drop.center();
        // `:597` — `TurnToAirAndSync`.
        app.items.swap_remove(i);
        // `:598-603` — the five pieces, `SpawnShimmeredItem(num++, type)`.
        for (num, kind) in [5655u16, 5656, 5657, 5658, 5661].into_iter().enumerate() {
            let vx = num as f32 * ((num % 2 * 2) as f32 - 1.0);
            let mut piece = ItemDrop::new(
                kind,
                1,
                0,
                center.x - crate::item::SIZE * 0.5,
                center.y - crate::item::SIZE * 0.5,
                vx,
                0.0,
                0.0,
            );
            // The `SpawnShimmeredItem` modifier (`:631-635`).
            piece.shimmer_time = 1.0;
            piece.shimmered = true;
            piece.shimmer_wet = true;
            piece.wet = true;
            app.items.push(piece);
        }
        // `:604-607` — the `HeroicisSetSpawnSound` particle broadcast is the
        // RainbowBoulder cue at the coin's centre (`ParticleOrchestrator.cs:
        // 897-900`); the port has no particle-orchestrator pass, the sound is
        // the whole observable effect.
        app.audio.play_rainbow_boulder(center.x, center.y);
        // The swap brought a new drop into slot i — re-check it (do not
        // advance). The appended pieces sit beyond the original length n.
    }
}

/// `WorldItem.TryFillBucket` (`WorldItem.cs:548-571`, W4-g15 lane 5): an
/// empty bucket (205) resting in the rain on a wind-allowing wall at the
/// surface fills into a water bucket (206), keeping its stack. The gate
/// (`:550`): the reservation (the caller passes it), `Main.raining`,
/// `Main.isThereAWorldSurface || Main.remixWorld`, and
/// `WorldGen.IsSurfaceForAtmospherics(position.ToTileCoordinates())`. The
/// cell (`:554-556`) is the drop's CENTRE tile, and its wall must ride
/// `WallID.Sets.AllowsWind`. The fill chance (`:558-563`): `num3 = 600`,
/// divided by `dayRate` when `0 < dayRate < 600`; then `Next(num3) == 0 &&
/// NextFloat() < maxRaining`. Both draws ride the `UpdateItems` stream (the
/// C# runs TryFillBucket inside UpdateItem under the `Main.cs:18024` swap).
/// The fill is `SetDefaults(206)` + the stack kept (`:565-567`) — the port
/// swaps the kind in place (the bucket's other stats are not modelled on a
/// drop, the kind is the whole observable identity).
#[allow(clippy::too_many_arguments)]
fn try_fill_bucket(
    rng: &mut UnifiedRandom,
    drop: &mut ItemDrop,
    world: &tr_core::world::World,
    raining: bool,
    max_raining: f32,
    has_world_surface: bool,
    remix_world: bool,
    day_rate: i32,
) {
    // `:550` — the gate row (the reservation is the caller's).
    if !raining || (!has_world_surface && !remix_world) {
        return;
    }
    // `WorldGen.IsSurfaceForAtmospherics` (`WorldGen.cs:10976-10987`): the
    // remix branch reads the rock layer + the maxTilesY-350 floor, else the
    // world surface. The position is the drop's top-left (`base.position`).
    let ptx = (drop.pos.x as i32) >> 4;
    let pty = (drop.pos.y as i32) >> 4;
    let surface = if remix_world {
        (pty as f64) > world.rock_layer && pty < world.height - 350
    } else {
        (pty as f64) <= world.world_surface
    };
    if !surface {
        return;
    }
    // `:554-556` — the CENTRE tile's wall must allow wind.
    let cx = (drop.center().x as i32) / 16;
    let cy = (drop.center().y as i32) / 16;
    let Some(tile) = world.tile(cx, cy) else {
        return;
    };
    if !tr_core::wind::wall_allows_wind(tile.wall) {
        return;
    }
    // `:558-563` — the two draws, in order, on the UpdateItems stream.
    let mut num3 = 600;
    if day_rate > 0 && day_rate < num3 {
        num3 /= day_rate;
    }
    if rng.next_max(num3) == 0 && rng.next_float() < max_raining {
        // `:565-567` — SetDefaults(206), the stack kept.
        drop.kind = 206;
    }
}

/// Spawns the drop left by a broken tile, with a small random pop like
/// `Item.NewItem` — `count` carries the C# stack (`Item.NewItem`'s `stack`
/// argument; the pot/pile/herb loot rolls spawn several items at once).
///
/// `rng` is the CALLER's stage stream (`Item.NewItem` draws `Main.rand` = the
/// active swap's stream, `Item.cs:49654-49727`) — shared by every pass, so
/// each caller feeds its own; no field lookups here (the drop list comes in
/// too, so the caller's stream borrow and the list stay disjoint).
fn item_splash(
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::world::World,
    water_style: i32,
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
        (tr_core::particles::dust_water(water_style), 10, 1) // `:930-939`/`:1009-1018`
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
    spawn_item_drop_at(items, rng, kind, count, pos, 1.0);
}

/// The shared spawn body. `x_bias` is the +1 px the tile-cell entry applies
/// (the C# rect overload's left edge); the CENTRE entry passes 0 so the box
/// lands exactly on `centre - SIZE/2` (W4-g78).
fn spawn_item_drop_at(
    items: &mut Vec<ItemDrop>,
    rng: &mut UnifiedRandom,
    kind: u16,
    count: u32,
    pos: Vec2,
    x_bias: f32,
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
        pos.x + x_bias,
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

/// The CENTRE-position variant — W4-g78's FIX-ONCE for the object-drop
/// centre gap (research/375 §2). The C# `Check*` routines spawn through the
/// obsolete rect overload (`Item.cs:49649-49652`: `center = (X + Width/2,
/// Y + Height/2)`), so `Item.NewItem(…, x*16, y*16, 32, 32, …)` centres the
/// 16×16 `WorldItem` box (`WorldItem.cs:138`) at `(x*16+16, y*16+16)`; the
/// tile variant above fed the tile TOP-LEFT (+1 px), landing every object
/// drop ~15 px left/up of the C# centre. This entry takes the CENTRE (the
/// `ObjectDrop::spawn_center_px` conversion) and places the port's 14×14
/// box at `centre − SIZE/2` — the same box-centre convention, no +1. The
/// draws are byte-identical to [`spawn_item_drop_px`] (only the position
/// differs). The per-cell / pot / tree / wall drops keep the tile variant
/// (their own gap is the separate 16×16-family residual, research/375 §2).
pub(crate) fn spawn_item_drop_centre(
    items: &mut Vec<ItemDrop>,
    rng: &mut UnifiedRandom,
    kind: u16,
    count: u32,
    centre: Vec2,
) {
    spawn_item_drop_at(
        items,
        rng,
        kind,
        count,
        centre - Vec2::splat(crate::item::SIZE * 0.5),
        0.0,
    );
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
/// `enemyGrabDelayTime` twin (`Item.cs:49757-49760`) gates the monster
/// grab (`WorldItem.cs:405`) — W4-g15 lane 5 ports it (the named-player
/// arm sets it, the all-players 255 arm does not).
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
    // `Item.cs:49757-49760` — the named-player ownership arm also sets the
    // monster-grab clock (the all-players 255 arm does not). W4-g15 lane 5.
    if grab_delay_player != 255 {
        drop.apply_enemy_grab_delay();
    }
    items.push(drop);
    if items.len() > 300 {
        items.remove(0);
    }
}

/// W4-d11 DOC-SWEEP — the Nebula set's magic on-hit pickup drop
/// (`Projectile.cs:13829` → `Item.RequestNewItem` → `Item.NewItem`,
/// `Item.cs:49654-49771`): the proc's explicit velocity (`:49738-49741`'s
/// `velocity.HasValue` arm) skips the scatter draws, `prefix: 0` draws
/// nothing (`Item.Prefix`'s `:511-514` early return), and `ownership: None`
/// arms NO grab delay in single-player (`WorldItem.ApplySpawnOwnership`'s
/// switch falls through, `WorldItem.cs:293-313`) — the drop is immediately
/// grabbable and the spawn is DRAW-FREE. `center` is the victim's centre
/// (`targetNPC.Center`); the port's top-left box sits at `center - SIZE/2`
/// (the `WorldItem.Center` setter convention; the C# 12×12 pickup box vs the
/// port's uniform 14 px `SIZE` is the recorded item-box stand-in).
pub(crate) fn spawn_nebula_pickup(
    items: &mut Vec<ItemDrop>,
    kind: u16,
    center: Vec2,
    velocity: Vec2,
) {
    let drop = ItemDrop::new(
        kind,
        1,
        0,
        center.x - crate::item::SIZE * 0.5,
        center.y - crate::item::SIZE * 0.5,
        velocity.x,
        velocity.y,
        0.0,
    );
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
/// other kind passes through untouched and draws NOTHING. W4-f39: the flags
/// ride the RESOLVED season statics (`Main.halloween`/`Main.xMas` =
/// `World.halloween`/`xmas`, re-derived at dawn + load from the date windows
/// + the force flags; the endless micro seeds feed them through
/// `forceHalloweenForever`/`forceXMasForever`, `world_sim/npcs.rs`'s loot-flag
/// precedent).
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

    /// A 100x100 world with a stone floor at y = 60 and a shimmer pool on top
    /// of it (rows 58-59), the player standing beside it. The world is wide
    /// and deep enough for `CheckInWorld`'s 20-tile fluff
    /// (`WorldGen.InWorld(pos, 20)`, `WorldItem.cs:573-588`): the old 40x40
    /// fixture put the drop at y = 19 / x = 30, inside the top/side fluff
    /// bands, so every drop was aired out on its first tick and the pool was
    /// empty. The geometry relative to the pool is unchanged (floor +1 row,
    /// pool rows −1/−2, drop flush on the floor).
    fn shimmer_world() -> App {
        let mut w = World::new("sh".into(), 100, 100);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        // The floor row 60 (solid) with the two-row shimmer pool 58-59 on top
        // (a drop resting in the liquid has shimmer both in its box and in the
        // tile above it — `CanShimmerAtPosition`'s `position.Y/16 - 1`,
        // `WorldItem.cs:762`).
        for x in 0..100 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            for y in 58..=59 {
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
        player.y = 57.0 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(9.0 * TILE_PX, 57.0 * TILE_PX);
        app
    }

    /// A drop resting on the pool's floor, the box overlapping the shimmer
    /// row (pos y = 59*16, flush on the 60-row floor — keep-one 144), parked
    /// at x = 30 tiles — outside the 42 px pickup magnet of the player at
    /// x = 9 (`ItemUpdate`'s magnet, `item.rs:176-188`) but inside the
    /// coin-luck 1000 px window.
    fn resting_drop(kind: u16, count: u32) -> ItemDrop {
        let mut d = ItemDrop::new(
            kind,
            count,
            0,
            30.0 * TILE_PX,
            59.0 * TILE_PX,
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
        // Doc-sweep lane 36 re-pin (91 → 92): `UpdateShimmer` (`:403`) reads
        // the PREVIOUS tick's `Entity.shimmerWet` — a fresh drop's flag is
        // false, so tick 1 only PROBES (MoveInWorld `:875-877` sets it) and
        // the ramp starts on tick 2. The old pre-update full-box re-probe
        // started the ramp on tick 1 (the stale-by-design 91). The ramp
        // itself is the C#'s `+= 0.01f` accumulation: the f32 undershoot at
        // the 90th sum puts the >0.9 crossing on ramp-tick 91 = update-tick
        // 92 (`WorldItem.cs:518-536`, the same arithmetic both sides).
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
        assert_eq!(ticks, 92, "the >0.9 crossing tick count (one-tick probe lag)");
        assert!((90..=93).contains(&ticks), "the >0.9 crossing tick count: {ticks}");
        let d = &app.items[0];
        assert!(d.shimmered, "the transform fired");
        assert_eq!(d.kind, 699); // Tin Ore
        assert_eq!(d.count, 3); // the stack is kept
    }

    #[test]
    fn the_ramp_decays_out_of_shimmer() {
        let mut app = shimmer_world();
        // Park the drop above the pool (y = 52), so it is dry.
        app.items.push(ItemDrop::new(
            11,
            1,
            0,
            30.0 * TILE_PX,
            52.0 * TILE_PX,
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
        // W4-f47 re-derivation onto the faithful MoveInWorld: `wetVelocity`
        // is captured at UpdateItem's head (`WorldItem.cs:379-384`) from the
        // PRE-UpdateShimmer velocity (0 here), so the rise UpdateShimmer arms
        // (`velocity.Y -= 0.05`, `:486-489`) does not integrate until the
        // NEXT tick — the C#'s ref-wetVelocity lag. Tick 1 holds position
        // but arms the rise velocity.
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items[0].pos.y, before, "tick 1 holds (pre-rise wetVelocity capture)");
        assert!(app.items[0].vel.y < 0.0, "the rise velocity armed");
        // Tick 2: wetVelocity = velocity * 0.375 (the shimmer scale, `:384`)
        // integrates the rise (`ApplyMovement`, `:1076-1078`).
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items[0].pos.y < before, "the item rises");
    }

    // -----------------------------------------------------------------
    // Keep-one 144 — the e19 lane's pass-level splash witnesses folded
    // (7d3cb5e3's four, re-pinned at the 16 geometry; the unit-level
    // draw-census twins live in item.rs's w4_f47ds block).
    // -----------------------------------------------------------------

    /// The entry splash (`WorldItem.cs:881-958`): a dry drop falling into
    /// the shimmer pool arms `wetCount = 20` and bursts 10 dust 308 (the
    /// `Next(6)` colour switch) through the shared pool, all on the
    /// `UpdateItems` stream. The camera parks at the pool so the spawn
    /// cull passes.
    #[test]
    fn w4_f47k1_the_shimmer_entry_splash_bursts_10_dust_308() {
        let mut app = shimmer_world();
        // The full dust budget (the flow_app default is 0 — every spawn
        // would hit the >90% pressure band's Next(4) refusal).
        app.gfx_quality.max_dust_to_draw = 6000;
        // Above the pool (rows 58-59), dry, falling - parked at x = 30
        // tiles (outside the player's grab magnet).
        app.items.push(ItemDrop::new(
            2, 1, 0, 30.0 * TILE_PX, 56.0 * TILE_PX, 0.0, 2.0, 0.0,
        ));
        app.camera = Vec2::new(30.0 * TILE_PX, 58.0 * TILE_PX);
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
    /// the OLD shimmerWet (the latch snapshot keeps it past the dry clear).
    #[test]
    fn w4_f47k1_the_shimmer_exit_splash_after_the_cooldown() {
        let mut app = shimmer_world();
        app.gfx_quality.max_dust_to_draw = 6000;
        // Resting IN the pool (the entry burst fires on tick 1).
        app.items.push(resting_drop(2, 1));
        app.camera = Vec2::new(30.0 * TILE_PX, 58.0 * TILE_PX);
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items[0].wet, "the entry latched");
        let after_entry = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        // Let the cooldown expire (20 ticks), then lift the drop dry.
        for _ in 0..25 {
            update_items(&mut app, 1.0 / 60.0);
        }
        assert_eq!(app.items[0].wet_count, 0, "the cooldown drained");
        app.items[0].pos.y = 50.0 * TILE_PX; // high above the pool — dry
        update_items(&mut app, 1.0 / 60.0);
        assert!(!app.items[0].wet, "the exit cleared wet");
        let after_exit = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        assert!(after_exit > after_entry, "the exit burst added dust 308");
        assert_eq!(app.items[0].wet_count, 19, "the exit re-armed the cooldown");
    }

    /// The cooldown suppresses re-entry churn (`:883`/`:962`): a drop that
    /// exits and re-enters within 20 ticks bursts ONCE.
    #[test]
    fn w4_f47k1_the_wet_count_suppresses_the_second_burst() {
        let mut app = shimmer_world();
        app.gfx_quality.max_dust_to_draw = 6000;
        app.items.push(resting_drop(2, 1));
        app.camera = Vec2::new(30.0 * TILE_PX, 58.0 * TILE_PX);
        update_items(&mut app, 1.0 / 60.0); // entry burst
        let n0 = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        // Out and back in within the cooldown window.
        app.items[0].pos.y = 50.0 * TILE_PX;
        update_items(&mut app, 1.0 / 60.0); // exit edge, wetCount > 0 → no burst
        app.items[0].pos.y = 59.0 * TILE_PX;
        update_items(&mut app, 1.0 / 60.0); // re-entry, wetCount > 0 → no burst
        let n1 = app.dust.slots().iter().filter(|d| d.active && d.dust_type == 308).count();
        assert_eq!(n1, n0, "the cooldown suppressed both edges");
    }

    /// The water entry arm (`:930-939`): 10 `dustWater()` dust with the
    /// scale MULTIPLY (×0.8) and alpha 100 — the water family, not 308.
    #[test]
    fn w4_f47k1_the_water_entry_splash_uses_the_water_style_dust() {
        let mut app = shimmer_world();
        app.gfx_quality.max_dust_to_draw = 6000;
        // Replace the shimmer pool with water.
        for x in 0..100 {
            for y in 58..=59 {
                let j = app.world.index(x, y);
                app.world.tiles[j].set_liquid_kind(tr_core::tile::LiquidKind::Water);
            }
        }
        app.items.push(ItemDrop::new(
            2, 1, 0, 30.0 * TILE_PX, 56.0 * TILE_PX, 0.0, 2.0, 0.0,
        ));
        app.camera = Vec2::new(30.0 * TILE_PX, 58.0 * TILE_PX);
        for _ in 0..30 {
            update_items(&mut app, 1.0 / 60.0);
            if app.items[0].wet {
                break;
            }
        }
        assert!(app.items[0].wet, "the drop reached the water");
        let ws = crate::env::water_style()
            .map(|n| n.min(14) as i32)
            .unwrap_or_else(|| crate::biome::calculate_water_style(&app.world, app.camera) as i32);
        let want = tr_core::particles::dust_water(ws);
        let hits: Vec<_> = app.dust.slots().iter().filter(|d| d.active && d.dust_type == want).collect();
        assert!(hits.len() >= 10, "the water burst: {} dust {want}", hits.len());
        for d in hits {
            assert_eq!(d.alpha, 100, "the water arm sets alpha 100");
            assert!(d.no_gravity);
        }
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
        // The full dust budget (the flow_app default is 0 — every spawn
        // would hit the >90% pressure band's `Next(4)` refusal, leaving only
        // the lucky few; the sibling splash witnesses set the same budget).
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(30.0 * TILE_PX, 58.0 * TILE_PX);
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

}

#[cfg(test)]
mod rand_light_witness {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::player::Player;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::world::World;

    /// A dry 100x100 world, stone floor from row 60 down, the player parked
    /// at x = 10 tiles — the drops sit at x = 28..34 tiles, outside the pickup
    /// magnet. The world clears `CheckInWorld`'s 20-tile fluff
    /// (`WorldGen.InWorld(pos, 20)`, `WorldItem.cs:573-588`); the old 40x40
    /// fixture put the drops at y = 19, inside the top band, so the items pass
    /// aired them out before the flicker arms ran.
    fn dry_world() -> App {
        let mut w = World::new("rl".into(), 100, 100);
        for x in 0..100 {
            for y in 60..100 {
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
        player.y = 58.0 * TILE_PX;
        app.player = Some(player);
        app
    }

    fn resting(kind: u16, tx: f32) -> ItemDrop {
        let mut d = ItemDrop::new(kind, 1, 0, tx * TILE_PX, 59.0 * TILE_PX, 0.0, 0.0, 0.0);
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
        // Slot order: 520 (draw), 2 (draw-free skip), 521 (draw), 2 (skip),
        // 58 (draw), 2 (no arm), 3456 (essScale, draw-free). (116/501 gained
        // their own faithful dust draws in the W4-g15 dust port —
        // `WorldItem.cs:1211-1222`/`:1295-1303` — so they are no longer
        // draw-free skips and the witness uses plain blocks for that role.)
        for (i, kind) in [520u16, 2, 521, 2, 58, 2, 3456].iter().enumerate() {
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
        // 116/501 now draw their dust gates (`WorldItem.cs:1211-1222`/
        // `:1295-1303`), so this muted-path witness uses only the genuinely
        // draw-free kinds: blocks and the two essScale fragments.
        for (i, kind) in [2u16, 3456, 2, 3459, 2].iter().enumerate() {
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

    /// W4-g78 FIX-ONCE (research/375 §2): the centre entry places the 14×14
    /// box at `centre − SIZE/2` — a chest's (21) 32×32 rect at tile (10, 18)
    /// centres the drop's box at (176, 304) exactly. The OLD top-left feed
    /// landed the box centre at (168, 295) — 8 px left / 9 px up (the C#
    /// WorldItem box is 16×16 centred on the rect centre; the port's 14×14
    /// box keeps its own size, only the CENTRE is fixed here). The draws
    /// stay byte-identical to the px entry (only the position differs).
    #[test]
    fn w4_g78_the_centre_spawn_lands_the_box_on_the_rect_centre() {
        let mut items = Vec::new();
        let mut rng = UnifiedRandom::new(777);
        let mut twin = UnifiedRandom::new(777);
        let centre = Vec2::new(176.0, 304.0);
        spawn_item_drop_centre(&mut items, &mut rng, 48, 1, centre);
        assert_eq!(items[0].center(), centre, "the box centres on the rect");
        assert_eq!(items[0].pos, centre - Vec2::splat(crate::item::SIZE * 0.5));
        // The draw-stream twin: the px entry at the derived top-left draws
        // the SAME stream state (velocity/prefix identical).
        let mut twin_items = Vec::new();
        spawn_item_drop_px(
            &mut twin_items,
            &mut twin,
            48,
            1,
            centre - Vec2::splat(crate::item::SIZE * 0.5),
        );
        assert_eq!(rng, twin);
        assert_eq!(items[0].vel, twin_items[0].vel);
        assert_eq!(items[0].prefix, twin_items[0].prefix);
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

    /// W4-g15 lane 5 — the reservation gate (`WorldItem.cs:683` / `:159`):
    /// a drop reserved for ANOTHER peer neither burns in lava nor combines
    /// with its neighbours. The port's `net_reserved` map carries the MP
    /// reservation; SP leaves it empty (the gate is vacuous there).
    #[test]
    fn w4_g15_a_foreign_reserved_drop_neither_burns_nor_combines() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = hell_world();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 496.0 * TILE_PX;
        app.player = Some(player);
        // Two rare-0 dirt drops inside the 30 px merge window, both in the
        // lava pool; slot 0 is reserved for peer 3 (not us, not 255).
        app.items.push(ItemDrop::new(
            2, 5, 0, 200.0 * TILE_PX, 498.0 * TILE_PX, 0.0, 0.0, 0.0,
        ));
        app.items.push(ItemDrop::new(
            2, 3, 0, 200.0 * TILE_PX + 10.0, 498.0 * TILE_PX, 0.0, 0.0, 0.0,
        ));
        app.net_reserved.insert(0, (3, 15));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 2, "the reserved drop did not burn");
        assert_eq!(app.items[0].count, 5, "and did not combine");
        assert_eq!(app.items[1].count, 3);
        // The reservation cleared: the next tick burns BOTH (the player is
        // far outside the grab rectangle, so the grab pass never touches
        // them — the lava check is the only consumer).
        app.net_reserved.remove(&0);
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the unreserved drops burned");
    }
}

#[cfg(test)]
mod make_a_wish_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemDrop;
    use crate::player::Player;

    /// A desert world with a water pool at rows 18-19, columns 18-22; the
    /// player stands in the desert (the scene flag is set directly — the
    /// grant reads `scene_metrics.zone_desert`, the last-scan convention).
    fn desert_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = tr_core::world::World::new("desert".into(), 40, 40);
        for x in 18..22 {
            for y in 18..20 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
                w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Water);
            }
        }
        w.compute_column_surface();
        app.world = w;
        let mut player = Player::spawn(&app.world);
        // Far from the coin (outside the grab rectangle) so the grab pass
        // never touches it — the grant is the only consumer.
        player.x = 2.0 * TILE_PX;
        player.y = 10.0 * TILE_PX;
        app.player = Some(player);
        app.scene_metrics.zone_desert = true;
        // The listener at the coin so the positioned RainbowBoulder cue is
        // audible (the attenuation gate, `sound.rs:105`'s 2500 px).
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (20.0 * TILE_PX, 18.0 * TILE_PX),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    /// A wet, single-stack, grab-delayed platinum coin in the desert grants
    /// the five Heroicis pieces as shimmered items with the alternating
    /// ±itemNumber X spread, and plays the RainbowBoulder cue.
    #[test]
    fn w4_g15_a_wet_desert_platinum_coin_grants_the_heroicis_set() {
        let mut app = desert_app();
        let mut coin = ItemDrop::new(74, 1, 0, 20.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.wet = true;
        coin.apply_grab_delay(0); // a player drop, named-player delay
        app.items.push(coin);
        let before = app.audio.played_count();
        try_granting_make_a_wish_set(&mut app);
        assert!(!app.items.iter().any(|it| it.kind == 74), "the coin consumed");
        let pieces: Vec<&ItemDrop> = app
            .items
            .iter()
            .filter(|it| matches!(it.kind, 5655 | 5656 | 5657 | 5658 | 5661))
            .collect();
        assert_eq!(pieces.len(), 5, "the five pieces spawned");
        for (num, p) in pieces.iter().enumerate() {
            assert!(p.shimmered && p.shimmer_wet && p.wet, "the modifier flags");
            assert_eq!(p.shimmer_time, 1.0);
            let vx = num as f32 * ((num % 2 * 2) as f32 - 1.0);
            assert_eq!(p.vel.x, vx, "the ±itemNumber spread (0,-1,2,-3,4)");
            assert_eq!(p.vel.y, 0.0);
        }
        assert!(app.audio.played_count() > before, "the RainbowBoulder cue");
    }

    /// The gate row (`WorldItem.cs:592`): a dry coin, a multi-stack, a
    /// shimmer-wet coin, an undelayed coin, an all-players-delayed coin, or
    /// a non-desert player all refuse the grant.
    #[test]
    fn w4_g15_the_wish_gate_refuses_the_wrong_states() {
        // Dry coin.
        let mut app = desert_app();
        let mut coin = ItemDrop::new(74, 1, 0, 20.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.apply_grab_delay(0);
        app.items.push(coin);
        try_granting_make_a_wish_set(&mut app);
        assert!(app.items.iter().any(|it| it.kind == 74), "dry refuses");
        // Multi-stack.
        let mut app = desert_app();
        let mut coin = ItemDrop::new(74, 2, 0, 20.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.wet = true;
        coin.apply_grab_delay(0);
        app.items.push(coin);
        try_granting_make_a_wish_set(&mut app);
        assert!(app.items.iter().any(|it| it.kind == 74), "stack != 1 refuses");
        // Shimmer-wet.
        let mut app = desert_app();
        let mut coin = ItemDrop::new(74, 1, 0, 20.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.wet = true;
        coin.shimmer_wet = true;
        coin.apply_grab_delay(0);
        app.items.push(coin);
        try_granting_make_a_wish_set(&mut app);
        assert!(app.items.iter().any(|it| it.kind == 74), "shimmerWet refuses");
        // All-players delay (255).
        let mut app = desert_app();
        let mut coin = ItemDrop::new(74, 1, 0, 20.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.wet = true;
        coin.apply_grab_delay(255);
        app.items.push(coin);
        try_granting_make_a_wish_set(&mut app);
        assert!(app.items.iter().any(|it| it.kind == 74), "255 delay refuses");
        // Non-desert.
        let mut app = desert_app();
        app.scene_metrics.zone_desert = false;
        let mut coin = ItemDrop::new(74, 1, 0, 20.0 * TILE_PX, 18.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.wet = true;
        coin.apply_grab_delay(0);
        app.items.push(coin);
        try_granting_make_a_wish_set(&mut app);
        assert!(app.items.iter().any(|it| it.kind == 74), "non-desert refuses");
    }
}

#[cfg(test)]
mod check_in_world_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemDrop;
    use crate::player::Player;

    /// A 100x100 world with the spawn at (50, 50); the player stands far
    /// from the off-world drops so the grab pass never touches them.
    fn wide_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = tr_core::world::World::new("wide".into(), 100, 100);
        w.spawn_x = 50;
        w.spawn_y = 50;
        w.compute_column_surface();
        app.world = w;
        let mut player = Player::spawn(&app.world);
        // Far from every test drop (the grab rectangle is ~52 px) so the
        // grab pass never touches them — CheckInWorld is the only consumer.
        player.x = 50.0 * TILE_PX;
        player.y = 2.0 * TILE_PX;
        app.player = Some(player);
        app
    }

    /// `CheckInWorld` (`WorldItem.cs:573-588`): a rare-0 drop beyond the
    /// 20-tile fluff airs out; a boss bag (RecoverableImportantItem) is
    /// recovered to the spawn cell centre with zero velocity.
    #[test]
    fn w4_g15_off_world_drops_air_out_and_boss_bags_recover() {
        let mut app = wide_app();
        // A dirt drop at tile (5, 50) — inside the 20-tile fluff (x < 20).
        app.items.push(ItemDrop::new(2, 1, 0, 5.0 * TILE_PX, 50.0 * TILE_PX, 3.0, 0.0, 0.0));
        // A boss bag (3318) at tile (95, 50) — beyond width - 20 = 80.
        app.items.push(ItemDrop::new(3318, 1, 0, 95.0 * TILE_PX, 50.0 * TILE_PX, 3.0, 0.0, 0.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(!app.items.iter().any(|it| it.kind == 2), "the dirt aired out");
        let bag = app.items.iter().find(|it| it.kind == 3318).expect("the bag recovered");
        // The spawn cell (50, 50) centre = (50*16+8, 50*16+8); the drop's
        // top-left = centre - SIZE/2.
        let want = Vec2::new(50.0 * TILE_PX + 8.0 - crate::item::SIZE * 0.5, 50.0 * TILE_PX + 8.0 - crate::item::SIZE * 0.5);
        assert!((bag.pos - want).length() < 0.5, "recovered to the spawn, got {:?}", bag.pos);
        assert_eq!(bag.vel, Vec2::ZERO, "the velocity zeroed");
    }

    /// A drop INSIDE the fluff boundary is untouched (the gate is strict).
    #[test]
    fn w4_g15_a_drop_inside_the_fluff_survives() {
        let mut app = wide_app();
        // Tile (25, 50) — inside [20, 80).
        app.items.push(ItemDrop::new(2, 1, 0, 25.0 * TILE_PX, 50.0 * TILE_PX, 0.0, 0.0, 0.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.iter().any(|it| it.kind == 2), "the in-bounds drop survives");
    }
}

#[cfg(test)]
mod fill_bucket_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemDrop;

    /// A surface world (worldSurface 60 > 50) with a wind-allowing wall (0)
    /// at the bucket's centre cell; the rain is on at full intensity.
    fn rain_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = tr_core::world::World::new("rain".into(), 100, 100);
        w.world_surface = 60.0;
        w.compute_column_surface();
        app.world = w;
        app
    }

    /// `TryFillBucket` (`WorldItem.cs:548-571`): the gate refuses when it is
    /// not raining, when there is no world surface (and no remix), when the
    /// drop is below the surface, or when the centre wall blocks wind. The
    /// draws are NOT consumed on a refused gate (the C# returns before them).
    #[test]
    fn w4_g15_the_fill_bucket_gate_refuses_the_wrong_states() {
        // Not raining.
        let mut app = rain_app();
        let mut b = ItemDrop::new(205, 3, 0, 50.0 * TILE_PX, 30.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, false, 1.0, true, false, 1);
        assert_eq!(b.kind, 205, "no rain refuses");
        // No world surface, no remix.
        let mut app = rain_app();
        let mut b = ItemDrop::new(205, 3, 0, 50.0 * TILE_PX, 30.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, true, 1.0, false, false, 1);
        assert_eq!(b.kind, 205, "no surface refuses");
        // Below the surface (tile y 70 > worldSurface 60).
        let mut app = rain_app();
        let mut b = ItemDrop::new(205, 3, 0, 50.0 * TILE_PX, 70.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, true, 1.0, true, false, 1);
        assert_eq!(b.kind, 205, "below surface refuses");
        // A wind-blocking wall (1) at the centre cell.
        let mut app = rain_app();
        let cx = (50.0 * TILE_PX + crate::item::SIZE * 0.5) as i32 / 16;
        let cy = (30.0 * TILE_PX + crate::item::SIZE * 0.5) as i32 / 16;
        let idx = app.world.index(cx, cy);
        app.world.tiles[idx].wall = 1;
        let mut b = ItemDrop::new(205, 3, 0, 50.0 * TILE_PX, 30.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, true, 1.0, true, false, 1);
        assert_eq!(b.kind, 205, "a blocking wall refuses");
    }

    /// A passing gate consumes the two draws in order (`Next(num3)` then
    /// `NextFloat`) and fills the bucket to 206 keeping the stack when both
    /// hit. The day-rate divisor scales num3 (600 / dayRate). A refused
    /// gate consumes NO draws (the C# returns before them).
    #[test]
    fn w4_g15_a_passing_gate_fills_the_bucket_and_keeps_the_stack() {
        // Find a seed whose first Next(600) == 0 (the dayRate-1 num3) so the
        // gate fires; maxRaining 1.0 makes the NextFloat arm always pass.
        let mut seed = 0;
        while tr_core::UnifiedRandom::new(seed).next_max(600) != 0 {
            seed += 1;
        }
        let mut app = rain_app();
        app.rngs.reset(seed);
        let mut b = ItemDrop::new(205, 7, 0, 50.0 * TILE_PX, 30.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, true, 1.0, true, false, 1);
        assert_eq!(b.kind, 206, "the bucket filled");
        assert_eq!(b.count, 7, "the stack kept (SetDefaults(206) + stack restore)");
        // The two draws ran in order: Next(600) then NextFloat. Replay them
        // on a witness seeded identically and confirm the stream advanced
        // by exactly those two (the next draw matches the witness's third).
        let mut witness = tr_core::UnifiedRandom::new(seed);
        let _ = witness.next_max(600);
        let _ = witness.next_float();
        let want_next = witness.next_max(1000);
        let got_next = app.rngs.stream(crate::app::RngKey::UpdateItems).next_max(1000);
        assert_eq!(got_next, want_next, "exactly two draws consumed, in order");
        // The day-rate divisor: dayRate 2 → num3 = 300. A seed whose first
        // Next(300) == 0 fires; the SAME seed's Next(600) may differ — the
        // divisor is load-bearing.
        let mut seed300 = 0;
        while tr_core::UnifiedRandom::new(seed300).next_max(300) != 0 {
            seed300 += 1;
        }
        let mut app = rain_app();
        app.rngs.reset(seed300);
        let mut b = ItemDrop::new(205, 1, 0, 50.0 * TILE_PX, 30.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, true, 1.0, true, false, 2);
        assert_eq!(b.kind, 206, "dayRate 2 → num3 300 fills on the 300-seed");
        // A refused gate (no rain) consumes NO draws.
        let mut app = rain_app();
        app.rngs.reset(seed);
        let mut b = ItemDrop::new(205, 1, 0, 50.0 * TILE_PX, 30.0 * TILE_PX, 0.0, 0.0, 0.0);
        let w = app.world.clone();
        try_fill_bucket(app.rngs.stream(crate::app::RngKey::UpdateItems), &mut b, &w, false, 1.0, true, false, 1);
        assert_eq!(b.kind, 205);
        let mut witness = tr_core::UnifiedRandom::new(seed);
        let want = witness.next_max(1000);
        let got = app.rngs.stream(crate::app::RngKey::UpdateItems).next_max(1000);
        assert_eq!(got, want, "a refused gate draws nothing");
    }
}

#[cfg(test)]
mod copper_slime_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemDrop;
    use crate::npc::{Npc, NpcKind};

    /// A world with a Blue Slime (the convertible kind 1) at (50, 50); the
    /// player stands far away so the grab pass never touches the drop.
    fn slime_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = tr_core::world::World::new("slime".into(), 100, 100);
        w.compute_column_surface();
        app.world = w;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 2.0 * TILE_PX;
        player.y = 2.0 * TILE_PX;
        app.player = Some(player);
        let mut n = Npc::new(NpcKind::Slime, 50.0 * TILE_PX, 50.0 * TILE_PX, 1, 0.0);
        n.hp = n.max_hp;
        app.npcs.push(n);
        app
    }

    /// `GetPickedUpByMonsters_Special` (`WorldItem.cs:1086-1116`): a Copper
    /// Shortsword (89) overlapping the Blue Slime consumes the drop and
    /// raises the unlock flag; the NPC stays (the 684 transform is the
    /// roster gap). A second drop does NOT re-grant (the flag gates).
    #[test]
    fn w4_g15_a_copper_shortsword_on_a_blue_slime_raises_the_unlock() {
        let mut app = slime_app();
        // The drop's box overlaps the slime's (both at tile 50).
        let d = ItemDrop::new(89, 1, 0, 50.0 * TILE_PX, 50.0 * TILE_PX, 0.0, 0.0, 0.0);
        assert!(!app.world.unlocked_slime_copper_spawn);
        assert!(copper_slime_pickup(&app.npcs, &d), "the convertible slime consumed it");
        // The flag write is the caller's post-tick step (the closure holds
        // the world borrow); the helper is the scan only.
        app.world.unlocked_slime_copper_spawn = true;
        assert!(app.world.unlocked_slime_copper_spawn, "the flag raised");
        assert_eq!(app.npcs.len(), 1, "the NPC stays (the 684 roster gap)");
        // The Chain Knife (3507) is the other kind; a non-convertible NPC
        // (a Zombie, kind 3) refuses.
        let mut app = slime_app();
        app.npcs[0].kind = NpcKind::Zombie;
        let d = ItemDrop::new(3507, 1, 0, 50.0 * TILE_PX, 50.0 * TILE_PX, 0.0, 0.0, 0.0);
        assert!(!copper_slime_pickup(&app.npcs, &d), "a zombie is not convertible");
        assert!(!app.world.unlocked_slime_copper_spawn);
        // A drop far from the slime refuses (no hitbox overlap).
        let app = slime_app();
        let d = ItemDrop::new(89, 1, 0, 80.0 * TILE_PX, 50.0 * TILE_PX, 0.0, 0.0, 0.0);
        assert!(!copper_slime_pickup(&app.npcs, &d), "no overlap refuses");
    }

    /// W4-g15 lane 5 — the `enemyGrabDelayTime` clock (`WorldItem.cs:28`,
    /// `:449-452`): the named-player ownership arm sets it to 100
    /// (`Item.cs:49757-49760`), it decrements per update, and the monster
    /// grab gate (`:405`) refuses while it is positive. The all-players 255
    /// arm does NOT set it.
    #[test]
    fn w4_g15_the_enemy_grab_delay_clock_gates_the_monster_pickup() {
        // The named-player arm sets the clock; the 255 arm does not.
        let mut d = ItemDrop::new(89, 1, 0, 0.0, 0.0, 0.0, 0.0, 0.0);
        d.apply_grab_delay(0);
        d.apply_enemy_grab_delay();
        assert_eq!(d.enemy_grab_delay_time, 100, "the named-player arm sets 100");
        // The decrement: one update ticks it down.
        let w = tr_core::world::World::new("t".into(), 32, 32);
        d.update(&w, 1.0 / 60.0);
        assert_eq!(d.enemy_grab_delay_time, 99, "the clock decrements");
        // The gate: a positive clock refuses the monster pickup even with
        // an overlapping convertible slime — the gate is the update_items
        // `:405` row (the scan helper itself does not read the clock).
        let mut app = slime_app();
        let mut coin = ItemDrop::new(89, 1, 0, 50.0 * TILE_PX, 50.0 * TILE_PX, 0.0, 0.0, 0.0);
        coin.enemy_grab_delay_time = 50;
        app.items.push(coin);
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "the positive clock refused the monster grab");
        assert!(!app.world.unlocked_slime_copper_spawn, "no unlock while delayed");
        // A zero clock allows the pickup (the scan fires, the flag lands
        // post-tick).
        app.items[0].enemy_grab_delay_time = 0;
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the zero clock allowed the monster grab");
        assert!(app.world.unlocked_slime_copper_spawn, "the unlock landed post-tick");
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

    /// A grounded 100x100 world; the stone floor starts at row 60 and the
    /// player stands mid-world just above it. The world is deep/wide enough
    /// for `CheckInWorld`'s 20-tile fluff (`WorldGen.InWorld(pos, 20)`,
    /// `WorldItem.cs:573-588`) on BOTH axes — a 40x40 fixture put the player
    /// (x = 10, y = 18) and every drop at x/y inside the fluff bands, so the
    /// items pass aired them out before the assertions could see them.
    fn pickup_world() -> App {
        let mut w = World::new("pickup".into(), 100, 100);
        for x in 0..100 {
            for y in 60..100 {
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
        player.x = 50.0 * TILE_PX;
        player.y = 58.0 * TILE_PX;
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
        // Park the player far away (x = 50 tiles); the drops at x = 30.
        let mut a = ItemDrop::new(9, 10, 0, 30.0 * TILE_PX, 59.0 * TILE_PX, 0.0, 0.0, 0.0);
        a.vel = Vec2::ZERO;
        let mut b = ItemDrop::new(9, 7, 0, 30.0 * TILE_PX + 10.0, 59.0 * TILE_PX, 0.0, 0.0, 0.0);
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

    /// W4-d79 — the Mana Cloak star 4143 (`Player.cs:35080-35097`): the
    /// star arm's 50-mana twin — +50 capped, the same ManaHeat (0.2, 2 s)
    /// pair + buff 396, the RAW-50 HealMana text, the drop airs out and
    /// never enters the inventory. The full-mana touch still consumes
    /// (the `:34947` pull gate's flag2 list excludes 4143 entirely).
    #[test]
    fn w4_d79_the_mana_cloak_star_restores_fifty_and_arms_mana_heat() {
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 180; // the cap eats 30 of the 50
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(4143, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the Mana Cloak star aired out");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 200, "capped at statManaMax2");
        assert_eq!(p.latest_mana_potion_damage_bonus, 0.2);
        assert!(
            (p.latest_mana_potion_duration - 120.0).abs() < 1e-3,
            "2 s = 120 ticks"
        );
        assert!(p.buffs.find_buff_index(396).is_some(), "ManaHeat added");
        assert!(
            app.inventory.iter().all(|s| s.count == 0),
            "nothing entered the inventory"
        );
        let t = app
            .combat_texts
            .iter()
            .find(|t| t.active)
            .expect("the mana text spawned");
        assert_eq!(t.text, "50", "the RAW 50 prints before the cap");
        assert_eq!(t.color, crate::combat_text::HEAL_MANA);
        // The full-mana touch: consumed, capped, heat refreshed.
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 200;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(4143, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "consumed at full mana");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 200);
        assert!(p.buffs.find_buff_index(396).is_some());
    }

    /// W4-d79 — the 4143 arm's RNG order: the case-7 pitch draw FIRST
    /// (`:35082`), then the ManaEffect text's two placement draws — the
    /// same three-draw contract as the heart/star witnesses above.
    #[test]
    fn w4_d79_the_mana_cloak_star_draws_ride_update_players_in_order() {
        use tr_core::unified_random::UnifiedRandom;
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 100;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(4143, 1, pc.0 - 4.0, pc.1 - 4.0));
        let seed = 0x4143_5152i32;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut witness = UnifiedRandom::new(seed);
        let _pitch = witness.next_range(-10, 11); // case 7 (`:725`)
        let _dx = witness.next_range(-10, 11); // width 20
        let _dy = witness.next_range(-21, 22); // height 42
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek(),
            witness.peek(),
            "exactly the chime + two placement draws, in the C# order"
        );
    }

    // W4-d79's `w4_d79_the_nebula_pickup_levels_the_matching_buff_family`
    // twin FOLDED into f49's set at batch-L (the S1 receipt: ONE set per
    // behaviour — `w4_f49_the_nebula_pickups_grant_their_family_buff_and_vanish`
    // below carries the same tier/sweep/independence assertions plus the
    // Player-field pins). The one-pitch RNG-order twin below is UNIQUE
    // (stream-exact) and stays.

    /// W4-d79 — the Nebula pickup's RNG order: the POSITIONED case-7 Grab
    /// draws ONE `Next(-10, 11)` pitch (`LegacySoundPlayer.cs:725`) on
    /// `UpdatePlayers` after the attenuation gate; `NebulaLevelup` draws
    /// nothing (the buff grant is RNG-free). Unlike the heart/star arms there
    /// is NO combat-text placement draw — the nebula arm spawns no text.
    #[test]
    fn w4_d79_the_nebula_pickup_draws_one_pitch_and_no_text() {
        use tr_core::unified_random::UnifiedRandom;
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(3455, 1, pc.0 - 4.0, pc.1 - 4.0));
        let seed = 0x0EB0_5152i32;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut witness = UnifiedRandom::new(seed);
        let _pitch = witness.next_range(-10, 11); // case 7 (`:725`)
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek(),
            witness.peek(),
            "exactly the one pitch draw — no text, no buff-grant draws"
        );
        assert!(app.combat_texts.iter().all(|t| !t.active), "no combat text");
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

    /// W4-f49 lane 12 — `PickupItem`'s Nebula arm (`Player.cs:35045-35058`):
    /// each of the trio consumes on touch, grants its family's tier-1 buff
    /// for 480 ticks via `NebulaLevelup` (the buffType rows
    /// `Item.cs:31008-31016`: 3453→179, 3454→173, 3455→176) and NEVER enters
    /// the inventory. The tier climbs on a second pickup of the same
    /// family. The d79 twin's Player-field + family-independence assertions
    /// folded in at batch-L (the S1 keep-one set). WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f49_the_nebula_pickups_grant_their_family_buff_and_vanish() {
        for (kind, buff) in [(3453u16, 179i32), (3454, 173), (3455, 176)] {
            let mut app = pickup_world();
            let pc = app.player.as_ref().unwrap().center();
            app.items.push(collectable_drop(kind, 1, pc.0 - 4.0, pc.1 - 4.0));
            update_items(&mut app, 1.0 / 60.0);
            assert!(app.items.is_empty(), "kind {kind}: consumed on touch");
            let p = app.player.as_ref().unwrap();
            let i = p.buffs.find_buff_index(buff).expect("the tier-1 family buff");
            assert_eq!(p.buffs.buff_time[i], 480, "kind {kind}: the 480 window");
            // The Player-field grant (the d79 twin's pin — the same-tick
            // hint the regen bridge re-derives).
            let tier = match buff {
                179 => p.nebula_level_damage,
                173 => p.nebula_level_life,
                _ => p.nebula_level_mana,
            };
            assert_eq!(tier, 1, "kind {kind}: the Player-field tier");
            assert!(
                app.inventory.iter().all(|s| s.count == 0),
                "kind {kind}: nothing entered the inventory"
            );
            // A second pickup of the same family steps to tier 2.
            app.items.push(collectable_drop(kind, 1, pc.0 - 4.0, pc.1 - 4.0));
            update_items(&mut app, 1.0 / 60.0);
            let p = app.player.as_ref().unwrap();
            assert!(p.buffs.find_buff_index(buff).is_none(), "the tier-1 id aired");
            assert!(
                p.buffs.find_buff_index(buff + 1).is_some(),
                "kind {kind}: tier 2 granted"
            );
        }
        // The families are independent (the d79 twin's cross-family pin).
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(3453, 1, pc.0 - 4.0, pc.1 - 4.0));
        app.items.push(collectable_drop(3454, 1, pc.0 - 4.0, pc.1 - 4.0));
        app.items.push(collectable_drop(3455, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.nebula_level_damage, 1);
        assert_eq!(p.nebula_level_life, 1);
        assert_eq!(p.nebula_level_mana, 1);
    }

    /// W4-f49 lane 12 — the Mana Cloak star 4143 (`Player.cs:35080-35098`):
    /// +50 mana capped at the max AFTER the text (the RAW 50 prints),
    /// `TryApplyManaHeat(0.2f, 2f)` arms buff 396, and the drop is consumed
    /// even at full mana (the intersects arm has no full-mana gate; only
    /// the star-family PULL at `:34947` does — 4143 is not in that family).
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_f49_the_mana_cloak_star_restores_fifty_and_arms_mana_heat() {
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 170; // the cap eats 20 of the 50
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(4143, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the star aired out");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 200, "capped at statManaMax2");
        assert_eq!(p.latest_mana_potion_damage_bonus, 0.2);
        assert!(p.buffs.find_buff_index(396).is_some(), "ManaHeat added");
        assert!(app.inventory.iter().all(|s| s.count == 0));
        let t = app
            .combat_texts
            .iter()
            .find(|t| t.active)
            .expect("the mana text spawned");
        assert_eq!(t.text, "50", "the RAW 50 prints before the cap");
        assert_eq!(t.color, crate::combat_text::HEAL_MANA);
        // Full mana: still consumed, the heat still refreshes.
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 200;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(4143, 1, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "consumed at full mana");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 200);
        assert!(p.buffs.find_buff_index(396).is_some());
    }

    /// W4-f49 lane 12 — the nebula PULL arm (`Player.cs:34953-34956`): a
    /// nebula drop inside the grab rectangle (the +100 range row,
    /// `:35157-35160`) but outside the hitbox rides `PullItem_Pickup(12f,
    /// 5)` — the same 12/5 blend as the mana-magnet star, NOT the common
    /// 0.75 pull. A resting drop 100 px right blends `(0*4 + (-12))/5` =
    /// -2.4 on the first grabbed tick. WRITTEN NOT RUN.
    #[test]
    fn w4_f49_the_nebula_pull_rides_the_twelve_five_blend() {
        let mut app = pickup_world();
        let pc = app.player.as_ref().unwrap().center();
        // 100 px right of the player centre — inside the 42+100 grab range,
        // outside the touch box. pos = centre - 7 (the 14x14 drop box).
        let x0 = pc.0 + 93.0;
        app.items.push(collectable_drop(3455, 1, x0, pc.1 - 7.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "pulled, not teleported");
        let d = &app.items[0];
        // The grabbed tick applies the blend raw (`WorldItem.cs:425-435`)
        // and clears the flag — the -2.4 velocity moved the drop 2.4 px.
        assert!(
            (d.vel.x - -2.4).abs() < 1e-3,
            "the 12/5 blend from rest: {}",
            d.vel.x
        );
        assert!(
            (d.pos.x - (x0 - 2.4)).abs() < 1e-3,
            "the raw-velocity tick: {}",
            d.pos.x
        );
    }

    /// W4-f49 residuals — the manaMagnet chain END TO END (the 2221
    /// stale-note census): equipping the Celestial Cuffs 2221 arms
    /// `manaMagnet` through the real tick (`ApplyEquipFunctional`
    /// `Player.cs:14949-14953`, `ResetEffects` clear `:18491`), the grab
    /// pass feeds it into `GrabFlags`, the +300 star range row
    /// (`:35137-35140`) reaches a star 100 px out, and the 12/5 pull
    /// (`:34943-34946`) blends -2.4 from rest — NOT the common 0.75 pull.
    /// At FULL mana without a magic held item the star is not pulled at
    /// all (`:34947`); without the magnet the base 42 never reaches.
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f49_the_cuffs_arm_the_star_magnet_end_to_end() {
        let mut app = pickup_world();
        {
            let mut p = app.player.take().unwrap();
            p.equipped[3] = 2221; // the Celestial Cuffs
            let w = app.world.clone();
            for _ in 0..2 {
                p.update(&w, crate::player::PlayerInput::default());
            }
            assert!(p.mana_magnet, "the equip pass armed the flag");
            assert!(p.magic_cuffs, "the cuffs' other half rides along");
            // The bar AFTER the ticks (Update rebases mana_max to the base
            // each tick); the magnet arm (:34943) wins over the :34947
            // full-mana gate regardless, so the exact bar does not gate
            // this pull — set it short for the census shape.
            p.mana_max = 200;
            p.mana = 150;
            app.player = Some(p);
        }
        let pc = app.player.as_ref().unwrap().center();
        let x0 = pc.0 + 93.0; // centre +100 — inside 42+300, outside touch
        app.items.push(collectable_drop(184, 1, x0, pc.1 - 7.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "pulled, not teleported");
        assert!(
            (app.items[0].vel.x - -2.4).abs() < 1e-3,
            "the 12/5 star blend: {}",
            app.items[0].vel.x
        );
        // Full mana, empty hand: the :34947 gate refuses the star pull.
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 200;
        }
        let pc = app.player.as_ref().unwrap().center();
        let x0 = pc.0 + 93.0;
        app.items.push(collectable_drop(184, 1, x0, pc.1 - 7.0));
        update_items(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1);
        assert_eq!(
            app.items[0].vel.x, 0.0,
            "no magnet: no reach at 100 (gravity owns y alone)"
        );
        // With the magnet the full-mana gate does NOT apply (:34943 wins).
        {
            let p = app.player.as_mut().unwrap();
            p.mana_magnet = true;
        }
        update_items(&mut app, 1.0 / 60.0);
        assert!(
            app.items[0].vel.x < 0.0,
            "the magnet pulls at full mana: {}",
            app.items[0].vel.x
        );
    }

    /// W4-f49 residuals — the star pickup's mana-restock CUE census
    /// (`Player.PickupItem` `Player.cs:35067`): `PlaySound(7)` — the Grab
    /// chime, legacy id 7 (`SoundID.Grab`, `SoundID.cs`), UNPOSITIONED
    /// (no attenuation), ONE `Next(-10, 11) * 0.01` pitch draw
    /// (`LegacySoundPlayer.cs:725`) on the `UpdatePlayers` stream (the
    /// `GrabItems` swap, `Main.cs:17754`). NOT legacy Item style 23/25 —
    /// those are `sound_id 2` use cues; the pickup is the id-7 family.
    /// `ManaEffect` (`:4531-4538`) is TEXT-ONLY (no second cue). The heart
    /// (`:35061`) and the Mana Cloak star 4143 (`:35082`) ride the same
    /// unpositioned play; only the nebula trio is POSITIONED (`:35047`).
    /// The i52 arms call `audio.play_pickup_sound(false, UpdatePlayers)`
    /// verbatim — this witness pins the play fires (count +1) and the
    /// silent listener refuses it with NO draw spent. WRITTEN NOT RUN.
    #[test]
    fn w4_f49_the_star_pickup_plays_the_unpositioned_grab_chime() {
        let mut app = pickup_world();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 0;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(184, 1, pc.0 - 4.0, pc.1 - 4.0));
        let before = app.audio.played_count();
        let stream_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek();
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the star was consumed");
        assert_eq!(
            app.audio.played_count(),
            before + 1,
            "ONE cue: the id-7 Grab play (ManaEffect adds no sound)"
        );
        assert_ne!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek(),
            stream_before,
            "the case-7 pitch draw spent on UpdatePlayers"
        );
        // The silent listener (`soundVolume == 0`, the case-7 refusal at
        // `LegacySoundPlayer.cs:381`): the star still restores, but NO cue
        // and NO draw (the gate precedes the pitch roll).
        let mut app = pickup_world();
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 200;
            p.mana = 0;
        }
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(184, 1, pc.0 - 4.0, pc.1 - 4.0));
        let before = app.audio.played_count();
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the restore is not sound-gated");
        assert_eq!(app.player.as_ref().unwrap().mana, 100);
        assert_eq!(app.audio.played_count(), before, "the silent gate refused");
        // (The ManaEffect TEXT jitter still draws — CombatText.NewText's
        // two Next rolls, `CombatText.cs:88-89`, are not sound-gated; the
        // refused play spends NO pitch draw of its own.)
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

    /// W4-d11 DOC-SWEEP — the Nebula pickup drop spawn
    /// (`Projectile.cs:13829` → `Item.RequestNewItem` → `Item.NewItem`): the
    /// proc's explicit velocity skips the scatter draws (`Item.cs:49738-49741`),
    /// prefix 0 rolls nothing (`:511-514`), and ownership None arms NO grab
    /// delay in SP (`WorldItem.cs:293-313`) — the drop is immediately grabbable
    /// and the spawn is DRAW-FREE. The box top-left is the victim centre -
    /// SIZE/2 (the `WorldItem.Center` setter convention). WRITTEN NOT RUN.
    #[test]
    fn w4_d11_the_nebula_pickup_spawns_draw_free_with_no_grab_delay() {
        let mut items: Vec<ItemDrop> = Vec::new();
        let center = Vec2::new(320.0, 384.0);
        let velocity = Vec2::new(2.0, -4.0);
        spawn_nebula_pickup(&mut items, 3453, center, velocity);
        assert_eq!(items.len(), 1);
        let d = &items[0];
        assert_eq!(d.kind, 3453);
        assert_eq!(d.count, 1);
        assert_eq!(d.prefix, 0);
        assert_eq!(d.pos.x, center.x - crate::item::SIZE * 0.5);
        assert_eq!(d.pos.y, center.y - crate::item::SIZE * 0.5);
        assert_eq!(d.vel, velocity, "the explicit velocity carries verbatim");
        // No grab delay: immediately grabbable (ownership None, SP).
        assert_eq!(d.grab_delay_time, 0, "no grab delay arms");
        // The three pickup kinds all ride the same spawn.
        for kind in [3454u16, 3455] {
            spawn_nebula_pickup(&mut items, kind, center, velocity);
            assert_eq!(items.last().unwrap().kind, kind);
        }
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
            d.pos = Vec2::new(400.0, 1920.0 - crate::item::SIZE);
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
