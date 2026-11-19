//! World item drops (plan stage A4): the fixed 60 Hz `Item.Update` physics and
//! pickup step, plus the `Item.NewItem`-shaped drop spawner.

use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::UnifiedRandom;

use crate::achievements;
use crate::app::{App, NetRole};
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
            .filter(|(i, it)| {
                (it.center() - pc).length() < 26.0
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
        // S1-P-i52 — the `GrabItems` player context (`Player.cs:34943-34952`):
        // the magnet flags, the held weapon's class (`HeldItem.magic`) and
        // the full-mana pull gate, snapshotted before the drop walk (the C#
        // reads the live fields inside the same tick; no ported writer
        // changes them mid-walk).
        let pickup_ctx = crate::item::PickupContext {
            life_magnet: player.as_ref().is_some_and(|p| p.life_magnet),
            mana_magnet: player.as_ref().is_some_and(|p| p.mana_magnet),
            held_item_magic: inventory
                .get(app.selected)
                .filter(|s| s.count > 0)
                .and_then(|s| registry::get(s.kind))
                .is_some_and(|d| d.magic),
            mana_not_full: player.as_ref().is_some_and(|p| p.mana != p.mana_max),
        };
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
            let collected = drop.update_with_pickup(world, player_center, &pickup_ctx, step);
            // S1-E-5 — `WorldItem.UpdateItem_VisualEffects`'s rand/essScale
            // head (`WorldItem.cs:1206-1293`, `item_lights.rs`): the C# runs
            // it at `:436`, after the movement half, per live drop in
            // `Main.item` slot order, under the `UpdateItems` swap
            // (`Main.cs:18024`) — so the flicker draws ride this stream, in
            // this order. A collected drop is `TurnToAir`'d (type 0) before
            // `:436` in the C#, so it contributes nothing; the lava-burned
            // drops above are the same. The RNG-free arms stay in the frame
            // feed (`entity_lights::item_light`).
            if !collected {
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
            }
            // `WorldItem.UpdateShimmer`'s 0.9 trigger (`WorldItem.cs:528-535`):
            // the transform runs after the step's items borrow is released (the
            // apply needs the app wholesale for the spawns and the world flags).
            if drop.shimmer_transform_due() {
                shimmer_due.push(i as u32);
            }
            if collected {
                // S1-P-i52 — `Player.PickupItem`'s heart/star arms
                // (`Player.cs:35059-35079`), BEFORE the coin/inventory walk
                // (the C#'s `else if` chain order). `ItemID.Sets.IsAPickup`
                // (`ItemID.cs:493`) makes `ItemSpace` unconditionally true
                // (`:39155-39158`) — the pickup fires even with a FULL
                // inventory, and a full-life/full-mana player still consumes
                // the drop (the heal/restore caps at the max). The
                // `SyncItem` tail (`:35109`) is the MP send seam.
                if crate::item::HEART_KINDS.contains(&drop.kind) {
                    // `:35059-35064`: `PlaySound(7)` (unpositioned), `Heal(20)`,
                    // `TurnToAir`. The case-7 pitch draw rides `UpdatePlayers`
                    // (the `GrabItems` caller under the `Main.cs:17754` swap —
                    // the E21f convention). The HealLife text spawns HERE,
                    // synchronously at the pickup site (`Heal` → `HealEffect`,
                    // `:35112-35123` → `:4522-4529`), not in the player-tick
                    // drain — the combat-text placement draws precede nothing
                    // else in this arm (the C# order).
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
                if crate::item::STAR_KINDS.contains(&drop.kind) {
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
                        // `num5 = Main.rand.Next(50, 76) * 0.01f` + the
                        // copper (71) `Next(51)` and silver (72) `Next(26)`
                        // extras, clamped at 1f; `num6 = (int)(stack * num5)`
                        // clamped 1..stack (`WorldItem.cs:1158-1172`). The
                        // pickup runs inside `WorldItem.UpdateItem` under the
                        // `UpdateItems` swap (`Main.cs:18024`) — the draws
                        // ride the `UpdateItems` stream.
                        let share = if drop.count == 1 {
                            1
                        } else {
                            let rng = app.rngs.stream(crate::app::RngKey::UpdateItems);
                            let mut num5 = rng.next_range(50, 76) as f32 * 0.01;
                            if drop.kind == 71 {
                                num5 += rng.next_max(51) as f32 * 0.01;
                            }
                            if drop.kind == 72 {
                                num5 += rng.next_max(26) as f32 * 0.01;
                            }
                            num5 = num5.min(1.0);
                            ((drop.count as f32 * num5) as u32).clamp(1, drop.count)
                        };
                        picked = registry::coin_value(drop.kind) * share as u64;
                        npc.extra_value += picked;
                        drop.count -= share;
                    }
                    if picked > 0 {
                        // S1-E-21f — `NPC.moneyPing` (`NPC.cs:90052-90054`,
                        // called from `WorldItem.cs:1189`): `PlaySound(38,
                        // base.position)` at the DROP's top-left. Case 38's
                        // two draws (slot, then pitch) ride the `UpdateItems`
                        // stream — the revenge arm runs inside
                        // `WorldItem.UpdateItem` under the `Main.cs:17779`
                        // swap, right after the share draws above.
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
                        .saturating_add(registry::coin_value(drop.kind) * drop.count as u64);
                    // S1-E-21f — the coin pickup chime (`Player.cs:39444`:
                    // `PlaySound(newItem.IsACoin ? 38 : 7)` — UNPOSITIONED).
                    // The C# site runs in `Player.GrabItems` under the
                    // `UpdatePlayers` swap (`Main.cs:17754`); the port's
                    // pickup lives in the items pass, so the drain point
                    // differs but the named stream is the source's.
                    audio.play_pickup_sound(
                        true,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
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
                    // S1-E-21f — `GetItem_VoidVault`'s fills play the same
                    // `IsACoin ? 38 : 7` chime (`Player.cs:39444`'s void-bag
                    // twins `GetItem_FillIntoOccupiedSlot_VoidBag`), unpositioned,
                    // on the `UpdatePlayers` stream (the GrabItems caller).
                    audio.play_pickup_sound(
                        false,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    items.swap_remove(i);
                    continue;
                }
                // `Player.GetItem`'s slot walk (`Player.cs:39347-39372`) runs
                // `GetItem_FillIntoOccupiedSlot` per slot (`:39493-39518`): an
                // occupied slot takes `maxStack - stack` and the leftover
                // `returnItem.stack -= maxStack - stack` stays in the
                // `returnItem` the caller swaps back into the world
                // (`Player.PickupItem`'s `itemToPickUp.ReplaceWith(item)`,
                // `:35085-35087`) — so the surplus is a drop, never an
                // over-full slot. `slot_for_pickup` already skips full slots,
                // so the loop drains through the roomier ones.
                // The vault took `drop.count - remaining`, so the slot walk
                // below sees only `remaining` (S1-C-9).
                let drop_count = remaining;
                let max_stack = registry::max_stack(drop.kind);
                let mut pending = drop_count;
                let mut moved = 0u32;
                while pending > 0 {
                    let Some(slot) = slot_for_pickup(inventory, drop.kind, drop.prefix) else {
                        break;
                    };
                    let room = max_stack.saturating_sub(inventory[slot].count);
                    if room == 0 {
                        break;
                    }
                    let take = pending.min(room);
                    inventory[slot].count += take;
                    pending -= take;
                    moved += take;
                }
                if moved > 0 {
                    // S1-E-21f — `Player.GetItem`'s occupied/empty fill chime
                    // (`Player.cs:39444`/`:39491`/`:39534`/`:39559`:
                    // `PlaySound(newItem.IsACoin ? 38 : 7)` — UNPOSITIONED,
                    // once per accepted slot; the port's slot walk is one
                    // merged fill, so one chime per pickup event). The Grab
                    // pitch draw rides `UpdatePlayers` (the `GrabItems`
                    // caller, `Player.cs:27428` under the `Main.cs:17754` swap).
                    audio.play_pickup_sound(
                        false,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                    // `AchievementsHelper.NotifyItemPickup` — the pickup
                    // feeds the ItemPickupCondition rows (TIMBER).
                    if let Some(_done) =
                        achievements.notify(achievements::CondKind::ItemPickup(drop.kind))
                    {
                        log::info!("achievement completed");
                    }
                    // The slot walk took everything: the drop is gone.
                    if pending == 0 {
                        items.swap_remove(i);
                        continue;
                    }
                    // The remainder stays where it fell (`ReplaceWith(item)`).
                    drop.count = pending;
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
        // `S1-C-9`: the drop must add `remaining` exactly once to the slot
        // (`Player.cs:39330` / `world_sim/items.rs:191-206`).
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
    fn pickup_sends_only_the_vault_remainder_to_inventory() {
        // Player.cs:39395-39423 / GetItem_VoidVault: the vault takes what
        // fits, the leftover falls through to the inventory once.
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
        let pc = app.player.as_ref().unwrap().center();
        app.items.push(collectable_drop(3, 5, pc.0 - 4.0, pc.1 - 4.0));
        update_items(&mut app, 1.0 / 60.0);
        assert!(app.items.is_empty(), "the drop was consumed");
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
        assert_eq!(in_inv, 2, "only the remainder reached inventory");
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
