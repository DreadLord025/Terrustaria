//! World item drops (plan stage A4): the fixed 60 Hz `Item.Update` physics and
//! pickup step, plus the `Item.NewItem`-shaped drop spawner.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::achievements;
use crate::item::{ItemDrop, slot_for_pickup};
use crate::registry;
use crate::app::{NetRole, rand01, App};

/// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
/// `ItemID.Sets.IsLavaImmuneRegardlessOfRarity` (`ItemID.cs`'s
/// `CreateBoolSet(false, 318, 312, 173, 174, 175, 4422, 2701, 205, 206,
/// 207, 1128, 2340, 2739, 2492, 1127, 85, 581, 582)`): the drops that
/// survive lava regardless of their rarity.
const LAVA_IMMUNE: &[u16] = &[318, 312, 173, 174, 175, 4422, 2701, 205, 206, 207, 1128, 2340, 2739, 2492, 1127, 85, 581, 582];

pub(crate) fn update_items(app: &mut App, dt: f32) {
    if app.is_client() {
        // The host owns the drops: ask it to collect whatever lies under our feet
        // and hide those locally until the next authoritative snapshot.
        let Some(pc) = app
            .player
            .as_ref()
            .map(|p| Vec2::new(p.center().0, p.center().1))
        else {
            return;
        };
        let ids: Vec<u32> = app
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| (it.center() - pc).length() < 26.0)
            .map(|(i, _)| i as u32)
            .collect();
        if let NetRole::Client(client) = &app.net {
            for id in ids {
                client.send_pickup(id);
            }
        }
        app.items.retain(|it| (it.center() - pc).length() >= 26.0);
        return;
    }
    let world = &app.world;
    let player = &mut app.player;
    let items = &mut app.items;
    let coins = &mut app.coins;
    let inventory = &mut app.inventory;
    let audio = &mut app.audio;
    let mut voodoo: Option<(Vec2, u32)> = None;
    app.item_step.tick(dt.min(0.25), |step| {
        let player_center = player.as_ref().map(|p| Vec2::new(p.center().0, p.center().1));
        // `AchievementsHelper.NotifyItemPickup`'s registry handle.
        let achievements = &mut app.achievements;
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
                let in_lava = world
                    .tile(tx, ty)
                    .is_some_and(|t| {
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
            let in_lava = world
                .tile(tx, ty)
                .is_some_and(|t| {
                    t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
                });
            if in_lava && (rare == 0 || rare == -1) && !lava_immune {
                // `TurnToAirAndSync` — the drop vanishes.
                items.swap_remove(i);
                continue;
            }
            if drop.update(world, player_center, step) {
                if registry::is_coin(drop.kind) {
                    *coins = (*coins).saturating_add(registry::coin_value(drop.kind) * drop.count as u64);
                    let c = drop.center();
                    audio.play_at("Coin_0", c.x, c.y, 1.0);
                    items.swap_remove(i);
                    continue;
                }
                // `Player.GetItem`'s void-vault arm (`Player.cs:39395`):
                // `CanGoIntoVoidVault && IsVoidVaultEnabled &&
                // CanVoidVaultAccept` routes the pickup into bank4 first
                // (`GetItem_VoidVault`, :39402-39423); the leftover falls
                // through to the inventory.
                let mut remaining = drop.count;
                let mut into_vault = false;
                if let Some(p) = player.as_mut() {
                    if p.void_vault_enabled
                        && crate::bank::can_void_vault_accept(drop.kind)
                        && crate::bank::bank_for_chest(-5).is_some()
                    {
                        let def = registry::get(drop.kind);
                        let max_stack = def.map(|d| d.max_stack).unwrap_or(999);
                        let taken = crate::bank::void_vault_take(
                            &mut p.banks[3],
                            drop.kind,
                            drop.prefix,
                            drop.count,
                            max_stack,
                        );
                        if taken == drop.count {
                            into_vault = true;
                        } else {
                            remaining = drop.count - taken;
                        }
                    }
                }
                if into_vault {
                    let c = drop.center();
                    audio.play_at("Item_1", c.x, c.y, 1.0);
                    items.swap_remove(i);
                    continue;
                }
                let drop_count = remaining;
                if let Some(slot) = slot_for_pickup(inventory, drop.kind, drop.prefix) {
                    inventory[slot].count = inventory[slot].count.saturating_add(drop_count);
                    inventory[slot].count = inventory[slot].count.saturating_add(drop.count);
                    let c = drop.center();
                    audio.play_at("Item_1", c.x, c.y, 1.0);
                    // `AchievementsHelper.NotifyItemPickup` — the pickup
                    // feeds the ItemPickupCondition rows (TIMBER).
                    if let Some(_done) = achievements.notify(achievements::CondKind::ItemPickup(drop.kind)) {
                        log::info!("achievement completed");
                    }
                    items.swap_remove(i);
                    continue;
                }
            }
            items[i] = drop;
            i += 1;
        }
    });
    app.item_step.cap_backlog();
    // The voodoo sacrifice resolves after the items borrow drops (the
    // spawn/kill pass takes `app` wholesale).
    if let Some((pos, stack)) = voodoo {
        crate::world_sim::npcs::voodoo_doll_lava_death(app, pos, stack);
    }
}

/// Spawns the drop left by a broken tile, with a small random pop like
/// `Item.NewItem` — `count` carries the C# stack (`Item.NewItem`'s `stack`
/// argument; the pot/pile/herb loot rolls spawn several items at once).
pub(crate) fn spawn_item_drop(app: &mut App, kind: u16, count: u32, tx: i32, ty: i32) {
    let r1 = rand01(&mut app.item_rng);
    let r2 = rand01(&mut app.item_rng);
    let r3 = rand01(&mut app.item_rng);
    // `Item.NewItem`'s `prefix: -1` on the tile/pot loot calls
    // (`WorldGen.cs:54248+`): the natural roll, drawn after the velocity
    // like the C# spawn order. Non-prefixable kinds draw nothing
    // (`Prefix(-1)`'s `CanHavePrefixes` gate).
    let prefix = crate::prefix::roll_prefix(kind, -1, &mut app.item_rng);
    let drop = ItemDrop::new(
        kind,
        count.max(1),
        prefix,
        tx as f32 * TILE_PX + 1.0,
        ty as f32 * TILE_PX,
        (r1 * 2.0 - 1.0) * 1.4,
        -2.2 - r2 * 1.2,
        r3 * std::f32::consts::TAU,
    );
    app.items.push(drop);
    if app.items.len() > 300 {
        app.items.remove(0);
    }
}

#[cfg(test)]
mod lava_burn_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemDrop;
    use crate::npc::{Npc, NpcKind};
    use tr_core::tile::{LiquidKind, Tile, FLAG_ACTIVE};
    use tr_core::world::World;

    /// A hell world with the lava pool at rows 498-499, columns 190-210
    /// (the `hell_world` shape from the voodoo tests).
    fn hell_world() -> World {
        let mut w = World::new("hell".into(), 400, 700);
        for x in 0..w.width {
            for y in 500..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 190..210 {
            for y in 498..500 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { ..Default::default() };
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
        player.x = 200.0 * TILE_PX;
        player.y = 496.0 * TILE_PX;
        app.player = Some(player);
        // The rare-0 dirt block: burns.
        app.items.push(ItemDrop::new(2, 1, 0, 200.0 * TILE_PX, 498.0 * TILE_PX, 0.0, 0.0, 0.0));
        // The lava-immune obsidian (173): survives.
        app.items.push(ItemDrop::new(173, 1, 0, 201.0 * TILE_PX, 498.0 * TILE_PX, 0.0, 0.0, 0.0));
        // A rare-1+ drop: survives (the rarity gate).
        let rare_item = (200..500)
            .find_map(|id| registry::get(id).filter(|d| d.rare >= 1))
            .expect("a rare row");
        app.items.push(ItemDrop::new(rare_item.id, 1, 0, 202.0 * TILE_PX, 498.0 * TILE_PX, 0.0, 0.0, 0.0));
        let before = app.items.len();
        update_items(&mut app, 1.0 / 60.0);
        assert!(
            !app.items.iter().any(|it| it.kind == 2),
            "the rare-0 dirt burned"
        );
        assert!(app.items.iter().any(|it| it.kind == 173), "the immune obsidian survived");
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
        player.x = 200.0 * TILE_PX;
        player.y = 496.0 * TILE_PX;
        app.player = Some(player);
        let mut guide = Npc::new(NpcKind::Guide, 196.0 * TILE_PX, 496.0 * TILE_PX, 1, 0.0);
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        app.items.push(ItemDrop::new(267, 1, 0, 200.0 * TILE_PX, 498.0 * TILE_PX, 0.0, 0.0, 0.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app
            .npcs
            .iter()
            .all(|n| n.kind != NpcKind::Guide || n.hp <= 0), "the sacrifice ran");
        assert!(app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::WallOfFlesh), "the wall awoke");
    }
}
