//! The bucket / sponge liquid family (W4-g45 doc-sweep) —
//! `Player.ItemCheck_UseBuckets` (`Player.cs:46318-46504`), called from
//! `ItemCheck_OwnerOnlyCode` (`Player.cs:43575`) every tick the item-use tick
//! runs. The twelve kinds: Empty Bucket 205, Water Bucket 206, Lava Bucket
//! 207, Honey Bucket 1128, Bottomless Bucket 3031, Super Absorbant Sponge
//! 3032, Bottomless Lava Bucket 4820, Lava Absorbant Sponge 4872, Bottomless
//! Honey Bucket 5302, Honey Absorbant Sponge 5303, Ultra Absorbant Sponge
//! 5304, Bottomless Shimmer Bucket 5364 (`ID/ItemID.cs:2093-2097/:3939/
//! :7745-7747/:11323/:11427/:12287-12291/:12411`).
//!
//! The action is NOT a use-start arm: the generic use-start
//! (`Player.cs:42733` → `ItemCheck_StartActualUse`) swings the item, and the
//! bucket action fires mid-swing whenever `ItemTimeIsZero && itemAnimation > 0
//! && controlUseItem` (`:46329`) and the cursor is in the Simple reach rect
//! (`:46320`). A refusal leaves `itemTime` at zero, so the action retries every
//! tick while held; a success re-arms `ApplyItemTime` (`useTime`). All twelve
//! kinds are `autoReuse` (`Item.cs:3958-3985/:13878-13885/:28072-28084`), so
//! the swing restarts the moment it ends and the cadence is continuous.
//!
//! The pickup arm converts the Empty Bucket into the filled bucket it drew
//! (honey → 1128, lava → 207, else water → 206; shimmer is refused for 205),
//! the sponges draw without consuming, and the placement arm pours a full 255
//! cell and hands back an Empty Bucket (the bottomless kinds keep their
//! stack). The item hand-back rides `PutItemInInventoryFromItemUsage`
//! (`Player.cs:42141-42160`). The five `PlaySound(19, position)` splashes
//! (`:46374/:46435/:46455/:46475/:46493`, the S1-E-21f deferred bucket family)
//! land here on the `UpdatePlayers` stream through `Audio::play_splash_sound`.
//!
//! W4-g46 SEAL carry: the g45 doc-sweep module (`fa96cf32a`) taken verbatim
//! onto the RE-FOLD union — the `add_water(&mut World)` / `wake_around`
//! signatures it calls are the union's g45b-guarded forms (research/69
//! ADDENDUM A1/A4; research/252 step 5).

use tr_core::tile::{LiquidKind, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA};
use tr_core::{edit, tiledata as td};

use crate::app::App;
use crate::item::ItemSlot;
use crate::registry;

use super::super::items::{spawn_item_drop, spawn_item_drop_px};

/// `Tile.liquidType()` (`Tile.cs:235-238`) as the port's kind code.
fn lq_code(k: LiquidKind) -> u8 {
    match k {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

/// The kind for a `liquidType(int)` write (`Tile.cs:216-233`).
fn lq_kind(c: u8) -> LiquidKind {
    match c {
        1 => LiquidKind::Lava,
        2 => LiquidKind::Honey,
        3 => LiquidKind::Shimmer,
        _ => LiquidKind::Water,
    }
}

/// The four sponge kinds — the pickup arm's `num2 <= 100` box-sum exemption
/// (`Player.cs:46347/:46352`: `sItem.type != 3032 && != 4872 && != 5303 &&
/// != 5304`). They draw a single cell without the 100-unit neighbourhood min.
fn is_sponge(kind: u16) -> bool {
    matches!(kind, 3032 | 4872 | 5303 | 5304)
}

/// Whether the held kind is one of the twelve bucket/sponge kinds
/// (`Player.cs:46320`'s type gate).
fn is_bucket(kind: u16) -> bool {
    matches!(
        kind,
        205 | 206 | 207 | 1128 | 3031 | 3032 | 4820 | 4872 | 5302 | 5303 | 5304 | 5364
    )
}

/// `Player.ItemCheck_UseBuckets` (`Player.cs:46318-46504`). Returns `true`
/// when the held item is a bucket/sponge (the caller consumes the tick — the
/// C# bucket has `damage = -1`, no pick/axe/hammer and no `createTile`, so it
/// never mines, melees or places; the bug-net precedent). The swing start is
/// the generic autoReuse use-start, NOT reach-gated; the liquid action is.
pub(crate) fn use_buckets(app: &mut App, tx: i32, ty: i32) -> bool {
    let held = app.inventory[app.selected];
    if !is_bucket(held.kind) {
        return false;
    }
    if held.count == 0 {
        // `Player.cs:42824-42828` — `itemAnimation == 0 && item.IsAir &&
        // type != 0` → `TurnToAir`: the emptied bucket clears at the swing
        // end (the port keeps the kind through the swing, like the C#).
        // Mid-swing the C# arm would still fire on a zero stack; the port
        // refuses (a one-action-per-stack floor, the safe stand-in).
        if app.swing <= 0.0 {
            app.inventory[app.selected] = crate::item::EMPTY_SLOT;
            return false;
        }
        return true;
    }
    // The generic use-start (`Player.cs:42733` → `ItemCheck_StartActualUse`
    // `:51657-51698` → `ApplyItemAnimation`): all twelve kinds are autoReuse,
    // so the swing restarts the moment it ends (the frame-head auto-reuse
    // block forces `mine_released`). `start_use_timers` is the port's
    // ApplyItemAnimation + reuse-lock + UseSound seam (buckets carry no
    // UseSound, so the start is silent). `item_time` is left alone — the C#
    // cadence is purely itemTime-driven across swing boundaries.
    if app.mine_down
        && !app.inventory_open
        && app.swing <= 0.0
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0
    {
        super::start_use_timers(app, held);
    }
    // `:46320` — the Simple reach rect with the item's tileBoost
    // (`TileReachCheckSettings.cs:81-85`, multiplier 1 / limit 20). The
    // `noBuilding` half has no port state (the frozen/webbed CC gate above
    // already refuses; `editing.rs`'s standing noBuilding note).
    let boost = registry::get(held.kind).map_or(0, |d| d.tile_boost);
    let in_reach = app.player.as_ref().is_some_and(|p| {
        super::tile_in_reach_rect(
            p,
            tx,
            ty,
            p.tile_range_x() as i32,
            p.tile_range_y() as i32,
            boost,
        )
    });
    if !in_reach {
        return true;
    }
    // `:46329` — `!ItemTimeIsZero || itemAnimation <= 0 || !controlUseItem`.
    // The integer tick read is the port's volley convention (no raw float
    // compare against a sub-tick residue).
    if super::ticks_of(app.item_time) > 0
        || app.swing <= 0.0
        || !app.mine_down
        || app.inventory_open
    {
        return true;
    }
    bucket_action(app, tx, ty, held);
    true
}

/// One fired bucket action — the pickup (`:46333-46424`) or placement
/// (`:46425-46503`) branch. Refusals return without re-arming `itemTime`, so
/// the action retries next tick while held.
fn bucket_action(app: &mut App, tx: i32, ty: i32, held: ItemSlot) {
    let kind = held.kind;
    let Some(target) = app.world.tile(tx, ty).copied() else {
        return;
    };
    let tk = target.liquid_kind();
    // `:46333` — the pickup-branch selector.
    let pickup = (kind == 205 && tk != LiquidKind::Shimmer)
        || (kind == 3032 && (lq_code(tk) == 0 || tk == LiquidKind::Shimmer))
        || (kind == 4872 && tk == LiquidKind::Lava)
        || (kind == 5303 && tk == LiquidKind::Honey)
        || kind == 5304;
    if pickup {
        bucket_pickup(app, tx, ty, held, target, tk);
    } else {
        bucket_place(app, tx, ty, held, target);
    }
}

/// The pickup arm (`Player.cs:46333-46424`): the 3×3 same-type box sum, the
/// 100-unit minimum (sponges exempt), the Empty-Bucket conversion, the target
/// wipe + the partial-cell top-up loop.
fn bucket_pickup(
    app: &mut App,
    tx: i32,
    ty: i32,
    held: ItemSlot,
    target: tr_core::tile::Tile,
    tk: LiquidKind,
) {
    let kind = held.kind;
    let num = lq_code(tk);
    // `:46337-46346` — the 3×3 box sum of the SAME liquid type. The C# reads
    // `Main.tile[i, j]` unguarded; the port probes (an out-of-world cell
    // contributes nothing — the reach rect can poke past the edge).
    let mut box_sum = 0u32;
    for i in (tx - 1)..=(tx + 1) {
        for j in (ty - 1)..=(ty + 1) {
            if let Some(t) = app.world.tile(i, j) {
                if lq_code(t.liquid_kind()) == num {
                    box_sum += u32::from(t.liquid);
                }
            }
        }
    }
    // `:46347` — the target must hold liquid, and a non-sponge needs the box
    // over 100.
    if target.liquid == 0 || (box_sum <= 100 && !is_sponge(kind)) {
        return;
    }
    let liquid_type = num; // `:46351`
    // `:46352-46373` — the Empty Bucket converts to the filled bucket it
    // drew; the sponges consume nothing.
    if !is_sponge(kind) {
        if tk == LiquidKind::Honey {
            consume_one(app);
            put_item_from_usage(app, 1128);
        } else if tk == LiquidKind::Lava {
            consume_one(app);
            put_item_from_usage(app, 207);
        } else {
            if tk == LiquidKind::Shimmer {
                return; // `:46366-46369` — 205 never draws shimmer.
            }
            consume_one(app);
            put_item_from_usage(app, 206);
        }
    }
    // `:46374` — the splash (Style defaults to 1), then `:46375` ApplyItemTime.
    play_splash(app);
    app.item_time = super::held_use_time_ticks(held) as f32 * super::ONE_TICK_SECS;
    let mut num3 = i32::from(target.liquid);
    // `:46377-46379` — `liquid = 0; lava(false); honey(false)`. The SHIMMER
    // bit is NOT cleared (the C# clears only lava/honey): a 3032 draw off a
    // shimmer cell leaves the shimmer type on the dry cell.
    if let Some(t) = app.world.tile_mut(tx, ty) {
        t.liquid = 0;
        t.flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
    }
    app.world.mark_chunk(tx, ty);
    // `:46380` SquareTileFrame(resetFrame: false) + `:46387` AddWater (SP).
    frame_and_wake(app, tx, ty, false);
    app.liquids.add_water(&mut app.world, tx, ty);
    // `:46389-46392` — a full target needs no top-up.
    if num3 >= 255 {
        return;
    }
    // `:46393-46423` — drain the same-type neighbours into the captured
    // amount, capping each transfer at `255 - num3`. The loop runs even once
    // num3 hits 255 (a zero transfer still re-frames + re-wakes the cell).
    for k in (tx - 1)..=(tx + 1) {
        for l in (ty - 1)..=(ty + 1) {
            if k == tx && l == ty {
                continue;
            }
            let Some(nb) = app.world.tile(k, l).copied() else {
                continue;
            };
            if nb.liquid > 0 && lq_code(nb.liquid_kind()) == num {
                let mut num4 = i32::from(nb.liquid);
                if num4 + num3 > 255 {
                    num4 = 255 - num3;
                }
                num3 += num4;
                if let Some(t) = app.world.tile_mut(k, l) {
                    t.liquid -= num4 as u8;
                    t.set_liquid_kind(lq_kind(liquid_type));
                    if t.liquid == 0 {
                        t.flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
                    }
                }
                app.world.mark_chunk(k, l);
                frame_and_wake(app, k, l, false);
                app.liquids.add_water(&mut app.world, k, l);
            }
        }
    }
}

/// The placement arm (`Player.cs:46425-46503`): pour a full 255 cell of the
/// bucket's type, hand back an Empty Bucket (the bottomless kinds keep their
/// stack), re-frame. The SP path wakes the liquid through SquareTileFrame's
/// own `TileFrame` AddWater (`WorldGen.cs:81344-81347`) — the C# placement
/// arms call only `NetMessage.sendWater` (MP), never AddWater directly.
fn bucket_place(app: &mut App, tx: i32, ty: i32, held: ItemSlot, target: tr_core::tile::Tile) {
    let kind = held.kind;
    // `:46427` — a >=200 cell or a non-top solid (the grate 546 exempt)
    // refuses the pour.
    let solid = target.active()
        && !target.inactive()
        && td::TILE_SOLID.has(target.kind)
        && !td::TILE_SOLID_TOP.has(target.kind)
        && target.kind != 546;
    if target.liquid >= 200 || solid {
        return;
    }
    // The per-kind type + bottomless flag + hand-back (`:46431-46502`).
    let (pour, bottomless, give) = match kind {
        207 => (LiquidKind::Lava, false, Some(205u16)),
        4820 => (LiquidKind::Lava, true, None),
        206 => (LiquidKind::Water, false, Some(205)),
        3031 => (LiquidKind::Water, true, None),
        1128 => (LiquidKind::Honey, false, Some(205)),
        5302 => (LiquidKind::Honey, true, None),
        5364 => (LiquidKind::Shimmer, true, None),
        _ => return,
    };
    // `:46433/:46453/:46473/:46491` — pour only into an empty cell or one
    // already holding this type.
    if target.liquid != 0 && lq_code(target.liquid_kind()) != lq_code(pour) {
        return;
    }
    play_splash(app);
    if let Some(t) = app.world.tile_mut(tx, ty) {
        t.set_liquid_kind(pour);
        t.liquid = 255;
    }
    app.world.mark_chunk(tx, ty);
    // `:46438/:46458/:46478/:46496` — SquareTileFrame (default resetFrame).
    frame_and_wake(app, tx, ty, true);
    // `:46439-46443/:46459-46463/:46479-46483` — the non-bottomless kinds
    // burn one and hand back an Empty Bucket; 5364 consumes nothing.
    if !bottomless {
        consume_one(app);
        if let Some(g) = give {
            put_item_from_usage(app, g);
        }
    }
    // `:46444/:46464/:46484/:46497` — ApplyItemTime.
    app.item_time = super::held_use_time_ticks(held) as f32 * super::ONE_TICK_SECS;
}

/// `sItem.stack--` (`Player.cs:46356/:46361/:46370/:46441/:46461/:46481`).
/// The kind is kept (the C# `TurnToAir` runs only at `itemAnimation == 0`,
/// `:42824-42828`); a zero count reads as air to the hand-back's selected-slot
/// arm, exactly like `Item.IsAir` (`Item.cs:385-395`).
fn consume_one(app: &mut App) {
    let slot = &mut app.inventory[app.selected];
    slot.count = slot.count.saturating_sub(1);
}

/// `Player.PutItemInInventoryFromItemUsage` (`Player.cs:42141-42160`): the
/// first partial same-type stack over the 58 slots (inventory 50 + coins 4 +
/// ammo 4) takes one; else the selected slot if it IsAir; else `GetOrDropItem`
/// (`ItemCreatedFromItemUsage`, `GetItemSettings.cs:61` = default — no void
/// vault, sound on) with the overflow spawned at the player's centre.
fn put_item_from_usage(app: &mut App, kind: u16) {
    let max = registry::max_stack(kind);
    // `:42143-42151` — the partial-stack scan (type + stack only, no prefix:
    // the buckets are not prefixable).
    for slot in app
        .inventory
        .iter_mut()
        .chain(app.equip.coins.iter_mut())
        .chain(app.equip.ammo.iter_mut())
    {
        if slot.kind == kind && slot.count > 0 && slot.count < max {
            slot.count += 1;
            return;
        }
    }
    // `:42152-42156` — the selected slot if it IsAir.
    let sel = app.selected;
    if app.inventory[sel].kind == 0 || app.inventory[sel].count == 0 {
        app.inventory[sel] = ItemSlot {
            kind,
            name: registry::name(kind),
            count: 1,
            prefix: 0,
        };
        return;
    }
    // `:42157-42159` — GetOrDropItem: the GetItem deposit walk (the chime per
    // accepted slot, `Player.cs:39491/:39559`), the overflow a world drop at
    // `base.Center` (`GetOrDropItem` `:7193-7200`). No void vault (the
    // ItemCreatedFromItemUsage settings carry CanGoIntoVoidVault = false).
    let center = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((app.camera.x, app.camera.y));
    let remaining = {
        // The disjoint-field split the update_items pickup caller uses: the
        // closure borrows only audio/rngs/achievements/favorite_slots, never
        // the inventory/ammo slices passed by &mut.
        let App {
            inventory,
            equip,
            audio,
            rngs,
            achievements,
            favorite_slots,
            ..
        } = app;
        crate::item::deposit_pickup(
            inventory,
            &mut equip.ammo,
            kind,
            0,
            1,
            |i| favorite_slots.contains(&i),
            &mut |_moved| {
                audio.play_pickup_sound(false, rngs.stream(crate::app::RngKey::UpdatePlayers));
                let _ = achievements.notify(crate::achievements::CondKind::ItemPickup(kind));
            },
        )
    };
    if remaining > 0 {
        spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            kind,
            remaining,
            glam::Vec2::new(center.0, center.1),
        );
    }
}

/// The five bucket splashes (`Player.cs:46374/:46435/:46455/:46475/:46493`):
/// `SoundEngine.PlaySound(19, (int)position.X, (int)position.Y)` — Style
/// defaults to 1, the position is the player's top-left truncated to int.
/// The case-19 playing guard + the one pitch draw ride `play_splash_sound`
/// on the `UpdatePlayers` stream (the S1-E-21f seam).
fn play_splash(app: &mut App) {
    let pos = app
        .player
        .as_ref()
        .map(|p| (p.x as i32 as f32, p.y as i32 as f32));
    if let Some((x, y)) = pos {
        app.audio.play_splash_sound(
            1,
            x,
            y,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        );
    }
}

/// `WorldGen.SquareTileFrame` (`WorldGen.cs:80185-80196`) + the liquid wake
/// its `TileFrame` walk runs (`WorldGen.cs:81344-81347`: every framed cell
/// holding liquid joins the active list in SP). `reset_frame` selects the
/// port's two seams: `actuation_square_frame` (resetFrame: false, the pickup
/// arm) vs `square_tile_frame_default` (resetFrame: true, the pour). The
/// support-check drops spawn on the `UpdatePlayers` stream (the ItemCheck
/// caller), the spawn_force_clear_area precedent.
fn frame_and_wake(app: &mut App, x: i32, y: i32, reset_frame: bool) {
    let drops = if reset_frame {
        edit::square_tile_frame_default(&mut app.world, x, y)
    } else {
        edit::actuation_square_frame(&mut app.world, x, y)
    };
    for drop in drops {
        let id = drop
            .item
            .map(|item| item as u16)
            .or_else(|| registry::tile_drop(drop.tile_kind, 0));
        if let Some(id) = id {
            spawn_item_drop(
                &mut app.items,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                id,
                1,
                drop.x,
                drop.y,
            );
        }
    }
    app.liquids.wake_around(&mut app.world, x, y, 1);
    app.world_key = None;
}
