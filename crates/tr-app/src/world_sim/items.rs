//! World item drops (plan stage A4): the fixed 60 Hz `Item.Update` physics and
//! pickup step, plus the `Item.NewItem`-shaped drop spawner.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::achievements;
use crate::item::{ItemDrop, slot_for_pickup};
use crate::registry;
use crate::app::{NetRole, rand01, App};

/// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
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
    let player = &app.player;
    let items = &mut app.items;
    let coins = &mut app.coins;
    let inventory = &mut app.inventory;
    let audio = &mut app.audio;
    app.item_step.tick(dt.min(0.25), |step| {
        let player_center = player.as_ref().map(|p| Vec2::new(p.center().0, p.center().1));
        // `AchievementsHelper.NotifyItemPickup`'s registry handle.
        let achievements = &mut app.achievements;
        let mut i = 0;
        while i < items.len() {
            let mut drop = items[i].clone();
            if drop.update(world, player_center, step) {
                if registry::is_coin(drop.kind) {
                    *coins = (*coins).saturating_add(registry::coin_value(drop.kind) * drop.count as u64);
                    let c = drop.center();
                    audio.play_at("Coin_0", c.x, c.y, 1.0);
                    items.swap_remove(i);
                    continue;
                }
                if let Some(slot) = slot_for_pickup(inventory, drop.kind) {
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
}

/// Spawns the drop left by a broken tile, with a small random pop like `Item.NewItem`.
pub(crate) fn spawn_item_drop(app: &mut App, kind: u16, tx: i32, ty: i32) {
    let r1 = rand01(&mut app.item_rng);
    let r2 = rand01(&mut app.item_rng);
    let r3 = rand01(&mut app.item_rng);
    let drop = ItemDrop::new(
        kind,
        1,
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
