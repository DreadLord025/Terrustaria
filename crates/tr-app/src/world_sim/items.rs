//! World item drops (plan stage A4): the fixed 60 Hz `Item.Update` physics and
//! pickup step, plus the `Item.NewItem`-shaped drop spawner.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::achievements;
use crate::app::{App, NetRole, rand01};
use crate::item::{ItemDrop, slot_for_pickup};
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
                client.send(&crate::net::C2s::Pickup { drop: id });
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
    // The drops whose shimmer ramp crossed 0.9 this step (the post-pass apply).
    let mut shimmer_due: Vec<u32> = Vec::new();
    app.item_step.tick(dt.min(0.25), |step| {
        let player_center = player
            .as_ref()
            .map(|p| Vec2::new(p.center().0, p.center().1));
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
            let collected = drop.update(world, player_center, step);
            // `WorldItem.UpdateShimmer`'s 0.9 trigger (`WorldItem.cs:528-535`):
            // the transform runs after the step's items borrow is released (the
            // apply needs the app wholesale for the spawns and the world flags).
            if drop.shimmer_transform_due() {
                shimmer_due.push(i as u32);
            }
            if collected {
                // The revenge pickup arm (`WorldItem.cs:1125-1200`): a
                // nearby hostile NPC (lifeMax > 5, not statue-spawned) sucks
                // 50-75% of a coin stack into `extraValue` while its
                // `extraValue < item.value && extraValue + value <
                // 999000000`. The moneyPing cue is not in the port's set.
                if registry::is_coin(drop.kind) {
                    let coin_copper = registry::coin_value(drop.kind) * drop.count as u64;
                    let mut picked = 0u64;
                    if let Some(npc) = app.npcs.iter_mut().find(|n| {
                        n.hp > 5
                            && !n.spawned_from_statue
                            && (n.center().0 - drop.center().x).abs() < 16.0
                            && (n.center().1 - drop.center().y).abs() < 24.0
                            && n.extra_value < coin_copper
                            && n.extra_value + coin_copper < 999_000_000
                    }) {
                        // `num7 = item.stack * (50..75)/100` (:1155-1160).
                        let share = if drop.count == 1 {
                            1
                        } else {
                            let mut rng_state = app.item_rng;
                            let r = crate::app::rand01(&mut rng_state);
                            app.item_rng = rng_state;
                            ((drop.count as f32 * (50.0 + r * 25.0)) as u32).clamp(1, drop.count)
                        };
                        picked = registry::coin_value(drop.kind) * share as u64;
                        npc.extra_value += picked;
                        drop.count -= share;
                    }
                    if picked > 0 {
                        let c = drop.center();
                        audio.play_at("Coin_0", c.x, c.y, 1.0);
                    }
                    if drop.count == 0 {
                        items.swap_remove(i);
                        continue;
                    }
                    *coins = (*coins)
                        .saturating_add(registry::coin_value(drop.kind) * drop.count as u64);
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
                    if let Some(_done) =
                        achievements.notify(achievements::CondKind::ItemPickup(drop.kind))
                    {
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
    // The shimmer transforms apply in the list order (the C#'s single-item
    // `GetShimmered` per tick; a step fires at most one per drop).
    crate::shimmer::apply_due_transforms(app, &shimmer_due);
}

/// Spawns the drop left by a broken tile, with a small random pop like
/// `Item.NewItem` — `count` carries the C# stack (`Item.NewItem`'s `stack`
/// argument; the pot/pile/herb loot rolls spawn several items at once).
pub(crate) fn spawn_item_drop(app: &mut App, kind: u16, count: u32, tx: i32, ty: i32) {
    spawn_item_drop_px(
        app,
        kind,
        count,
        Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX),
    );
}

/// The pixel-position variant of [`spawn_item_drop`] — the spawn points that
/// name a world position instead of a tile cell (`Item.RequestNewItem`'s
/// `center`, e.g. the extractinator's cursor drop,
/// `Player.DropItemFromExtractinator` `Player.cs:42081-42089`). The draws keep
/// the `Item.NewItem` order: the velocity/phase first, the prefix last.
pub(crate) fn spawn_item_drop_px(app: &mut App, kind: u16, count: u32, pos: Vec2) {
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
        pos.x + 1.0,
        pos.y,
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
        player.x = 200.0 * TILE_PX;
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
        player.x = 200.0 * TILE_PX;
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
