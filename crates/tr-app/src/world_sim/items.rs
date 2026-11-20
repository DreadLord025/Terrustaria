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
        // 14 px box sits at 1906) with zero velocity, aged past the 0.35 s
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
        for _ in 0..99 {
            update_items(&mut app, 1.0 / 60.0);
            assert!(!app.items.is_empty(), "the delayed drop is not grabbed");
        }
        // Tick 100: the delay hit 0, the grab opens.
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
