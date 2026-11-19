//! World item drops (plan stage A4): the fixed 60 Hz `Item.Update` physics and
//! pickup step, plus the `Item.NewItem`-shaped drop spawner.

use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::UnifiedRandom;

use crate::achievements;
use crate::app::{App, NetRole};
use crate::item::ItemDrop;
use crate::registry;

/// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
/// `ItemID.Sets.IsLavaImmuneRegardlessOfRarity` (`ItemID.cs`'s
/// `CreateBoolSet(false, 318, 312, 173, 174, 175, 4422, 2701, 205, 206,
/// 207, 1128, 2340, 2739, 2492, 1127, 85, 581, 582)`): the drops that
/// survive lava regardless of their rarity.
const LAVA_IMMUNE: &[u16] = &[
    318, 312, 173, 174, 175, 4422, 2701, 205, 206, 207, 1128, 2340, 2739, 2492, 1127, 85, 581, 582,
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
            })
            .map(|(i, _)| i as u32)
            .collect();
        if let NetRole::Client(client) = &app.net {
            for id in ids {
                client.send(&crate::net::C2s::Pickup { drop: id });
            }
        }
        app.items.retain(|it| !in_range(it));
        return;
    }
    let world = &app.world;
    let player = &mut app.player;
    let items = &mut app.items;
    let coins = &mut app.coins;
    let inventory = &mut app.inventory;
    let audio = &mut app.audio;
    let mut voodoo: Option<(Vec2, u32)> = None;
    // The drops whose shimmer ramp crossed 0.9 this step (the post-pass apply).
    let mut shimmer_due: Vec<u32> = Vec::new();
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
            // `:35045-35098`): hearts/stars ride i52, the nebula/4143 mana
            // arms have no ported buff route — the kinds are not grabbed
            // until those land (ANALYSIS S1-P-i47).
            if crate::item::is_a_pickup(kind) {
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
            let flags = crate::item::GrabFlags {
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
                if registry::is_coin(kind) {
                    // The coin wallet path (the port's `App.coins` purse —
                    // the coin-slot routing is i50's). The revenge suck
                    // (`WorldItem.cs:1125-1200`) runs here on the pickup,
                    // the port's recorded placement deviation: a nearby
                    // hostile NPC (lifeMax > 5, not statue-spawned) takes
                    // 50-75% of the stack into `extraValue`.
                    let mut drop = items[i].clone();
                    let coin_copper = registry::coin_value(kind) * drop.count as u64;
                    let mut picked = 0u64;
                    if let Some(npc) = app.npcs.iter_mut().find(|n| {
                        n.hp > 5
                            && !n.spawned_from_statue
                            && (n.center().0 - drop.center().x).abs() < 16.0
                            && (n.center().1 - drop.center().y).abs() < 24.0
                            && n.extra_value < coin_copper
                            && n.extra_value + coin_copper < 999_000_000
                    }) {
                        // `num5 = Main.rand.Next(50, 76) * 0.01f` + the
                        // copper (71) `Next(51)` and silver (72) `Next(26)`
                        // extras, clamped at 1f; `num6 = (int)(stack * num5)`
                        // clamped 1..stack (`WorldItem.cs:1158-1172`). The
                        // draws ride the `UpdateItems` stream (the port's
                        // pickup lives in the items pass).
                        let share = if drop.count == 1 {
                            1
                        } else {
                            let rng = app.rngs.stream(crate::app::RngKey::UpdateItems);
                            let mut num5 = rng.next_range(50, 76) as f32 * 0.01;
                            if kind == 71 {
                                num5 += rng.next_max(51) as f32 * 0.01;
                            }
                            if kind == 72 {
                                num5 += rng.next_max(26) as f32 * 0.01;
                            }
                            num5 = num5.min(1.0);
                            ((drop.count as f32 * num5) as u32).clamp(1, drop.count)
                        };
                        picked = registry::coin_value(kind) * share as u64;
                        npc.extra_value += picked;
                        drop.count -= share;
                    }
                    if picked > 0 {
                        // S1-E-21f — `NPC.moneyPing` (`NPC.cs:90052-90054`,
                        // `WorldItem.cs:1189`): case 38's two draws ride the
                        // `UpdateItems` stream, right after the share draws.
                        audio.play_coin_ping(
                            drop.pos.x,
                            drop.pos.y,
                            app.rngs.stream(crate::app::RngKey::UpdateItems),
                        );
                    }
                    if drop.count == 0 {
                        items.swap_remove(i);
                        continue;
                    }
                    *coins = (*coins)
                        .saturating_add(registry::coin_value(kind) * drop.count as u64);
                    // S1-E-21f — the coin chime (`Player.cs:39444`:
                    // `PlaySound(newItem.IsACoin ? 38 : 7)`, UNPOSITIONED)
                    // on the `UpdatePlayers` stream (the GrabItems caller).
                    audio.play_pickup_sound(
                        true,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    items.swap_remove(i);
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
                    );
                    if taken > 0 {
                        remaining -= taken;
                        // `GetItem_FillIntoOccupiedSlot_VoidBag`'s chime
                        // (`:39444`); the port's merged fill plays one per
                        // pickup event (the standing S1-E-21f deviation).
                        audio.play_pickup_sound(
                            false,
                            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        );
                    }
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
            // `:34935-34972` — the pull arms. `flag2` = the heart/star
            // family; at full mana without the Mana Magnet they are not
            // pulled at all (`:34947`). The accessory grants are the
            // `GrabFlags` seam (unported equipment families).
            let drop = &mut items[i];
            drop.shimmered = false;
            drop.being_grabbed = true;
            let flag2 = matches!(kind, 184 | 1735 | 1868);
            let held_magic = registry::get(inventory[app.selected].kind)
                .is_some_and(|d| d.magic);
            if !flag2 || held_magic || mana != mana_max {
                if status.to_void_vault {
                    // `PullItem_ToVoidVault` (`:34978-34981`).
                    crate::item::pull_item_pickup(drop.center(), &mut drop.vel, pc, 12.0, 5);
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
            drop.update(world, step);
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

    /// A drop resting on the pool's floor, the 14x14 box overlapping the
    /// shimmer row (pos y = 19*16 + 2), parked at x = 30 tiles — outside the
    /// 42 px pickup magnet of the player at x = 9 (`ItemUpdate`'s magnet,
    /// `item.rs:176-188`) but inside the coin-luck 1000 px window.
    fn resting_drop(kind: u16, count: u32) -> ItemDrop {
        let mut d = ItemDrop::new(
            kind,
            count,
            0,
            30.0 * TILE_PX,
            19.0 * TILE_PX + 2.0,
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
        // 90 ticks to cross 0.9 (`WorldItem.cs:528-535`).
        // The ramp is the C#'s `+= 0.01f` accumulation, so the >0.9 crossing
        // lands on the tick the accumulated f32 value passes 0.9 — the C# runs
        // the same arithmetic (`WorldItem.cs:518-536`), so the tick count is
        // identical there; the f32 undershoot at the 90th sum is the port's
        // and the C#'s alike.
        let mut ticks = 0;
        while !app.items[0].shimmered && ticks < 120 {
            update_items(&mut app, 1.0 / 60.0);
            ticks += 1;
        }
        if std::env::var("SHIMMER_TRACE").is_ok() {
            eprintln!("transformed after {ticks} ticks");
        }
        assert_eq!(ticks, 91, "the >0.9 crossing tick count");
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
        let mut d = ItemDrop::new(kind, 1, 0, tx * TILE_PX, 19.0 * TILE_PX + 2.0, 0.0, 0.0, 0.0);
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
    /// (`WorldItem.cs:1158-1172`): `Next(50, 76) * 0.01f` plus the copper (71)
    /// `Next(51)` and silver (72) `Next(26)` extras on the `UpdateItems`
    /// stream — a copper stack spends exactly two draws.
    #[test]
    fn the_revenge_share_draws_next_50_76_plus_the_coin_extras() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = crate::test_world_helpers::ground();
        w.compute_column_surface();
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        let (px, py) = app.player.as_ref().unwrap().center();
        // A copper coin stack at the player's feet (the pickup magnet range).
        let mut drop = ItemDrop::new(71, 10, 0, px, py, 0.0, 0.0, 0.0);
        drop.age = 1.0;
        // A hostile NPC straddling the drop's box (lifeMax > 5, not
        // statue-spawned, within the 16x24 px window): its center pinned to
        // the drop's.
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
        let sucked = app.npcs[0].extra_value;
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
        for _ in 0..99 {
            update_items(&mut app, 1.0 / 60.0);
            assert_eq!(app.items.len(), 1, "still locked at tick {}", _);
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
        let mut a = ItemDrop::new(9, 10, 0, 30.0 * TILE_PX, 19.0 * TILE_PX + 2.0, 0.0, 0.0, 0.0);
        a.vel = Vec2::ZERO;
        let mut b = ItemDrop::new(9, 7, 0, 30.0 * TILE_PX + 10.0, 19.0 * TILE_PX + 2.0, 0.0, 0.0, 0.0);
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
}
