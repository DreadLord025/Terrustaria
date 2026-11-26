//! The block-swap accessibility feature — `Player.PlaceThing_TryReplacingTiles`
//! (`Player.cs:39841-39913`) + `PlaceThing_ValidTileForReplacement`
//! (`:39915-40044`) + the two validity sub-checks (`:40046-40093`), wired at
//! the `PlaceThing_Tiles` head (`:39728-39731`, the `TileReplacementEnabled`
//! gate = `builderAccStatus[10] == 0`, `:3373`). The world write is
//! `tr_core::edit::replace_tile_write` (`WorldGen.ReplaceTile`,
//! `WorldGen.cs:62558-62684`); the kill-effects (dust/sound/drops/fossil
//! shatter) run here in the C# order (`:62570-62578`).
//!
//! The `hitReplace` damage book (`Player.cs:1660`) is a SECOND `HitTile`
//! (`App::hit_replace`); `ClearMiningCacheAt` clears both (`:53846-53850`).
//!
//! Deliberate deviation (the `break_tile` convention, research D3): the
//! `KillTile_MakeTileDust` dust (`:39875-39879` fail tick, `:62570-62574`
//! success) is SKIPPED — the port has no per-tick tile-dust path (the
//! ~2500-line dust-type table, `WorldGen.cs:66687-69169`, is unported).
//! The sounds + drops + the write + every gate ARE ported.

use tr_core::tiledata as td;
use tr_core::{edit, World};

use crate::app::App;
use crate::registry;

use super::auto_actuator::{auto_accessory_cache, auto_actuate_after_place};
use super::{break_drop_events, cell_drops, drop_ctx, pick_world, HitCell, ONE_TICK_SECS};

/// The result of the replacement attempt — the C# `canUse` return plus the
/// timer re-arms the caller must keep (the placement route's generic
/// cadence would clobber them).
pub(super) enum SwapOutcome {
    /// `canUse` unchanged — the normal placement route proceeds.
    Proceed,
    /// `canUse = false` with no timer change (the no-pickaxe / zero-damage
    /// refusals, `:39853`/:39865): the tick is consumed, nothing re-arms.
    Blocked,
    /// The fail tick (`:39886-39887`) or the success (`:39907-39908`):
    /// `(swing_secs, item_time_secs)` — the caller applies them AFTER the
    /// use-start (the C# start runs before `PlaceThing`).
    Timed(f32, f32),
}

/// `ItemID.Sets.SortingPriorityRopes` (`ItemID.cs:1345`): the five rope
/// items — they never replace (`Player.cs:39928-39931`).
fn is_rope_item(kind: u16) -> bool {
    matches!(kind, 965 | 85 | 2996 | 3077 | 3078)
}

/// `Player.GetBestPickaxe` (`Player.cs:40095-40106`): the highest-pick
/// item in the 50-slot grid (the first wins ties — the C#'s strict `>`).
fn best_pickaxe(app: &App) -> Option<(u16, i32)> {
    let mut best: Option<(u16, i32)> = None;
    for slot in &app.inventory {
        if slot.count == 0 {
            continue;
        }
        let Some(def) = registry::get(slot.kind) else {
            continue;
        };
        let pick = i32::from(def.pick);
        if pick > 0 && best.is_none_or(|(_, b)| pick > b) {
            best = Some((slot.kind, pick));
        }
    }
    best
}

/// `Player.PlaceThing_CheckPlatformSolidityForBlockSwap`
/// (`Player.cs:40046-40057`): a platform/planter-box target refuses a
/// non-platform replacement while the player's hitbox overlaps the cell.
fn platform_solidity_ok(app: &App, target: u16, current: u16, x: i32, y: i32) -> bool {
    if (td::SET_PLATFORMS.has(current) || current == 380)
        && !td::SET_PLATFORMS.has(target)
        && target != 380
    {
        let Some(p) = app.player.as_ref() else {
            return true;
        };
        // `Rectangle.Intersects` — strict `<` on both axes.
        let (rx, ry) = (x as f32 * 16.0, y as f32 * 16.0);
        if p.x < rx + 16.0 && rx < p.x + crate::player::WIDTH && p.y < ry + 16.0 && ry < p.y + p.height
        {
            return false;
        }
    }
    true
}

/// `Player.PlaceThing_CheckSpecificValidtyCaseForBlockSwap`
/// (`Player.cs:40059-40093`): the falling-block support rule (a 110+ pick
/// or a non-falling cell above lifts it) + the wet spike refusal.
fn specific_validity_ok(app: &App, target: u16, current: u16, x: i32, y: i32) -> bool {
    let falling_target = td::SET_FALLING.has(target);
    let mut falling_refuse = td::SET_FALLING.has(current) && !falling_target;
    if falling_refuse {
        if best_pickaxe(app).is_some_and(|(_, pick)| pick >= 110) {
            falling_refuse = false;
        }
    }
    if falling_refuse && y > 0 {
        // `:40071-40083` — the above cell lifts the refusal when it is
        // inactive or non-falling (a null above keeps it).
        if let Some(above) = app.world.tile(x, y - 1) {
            if !above.active() || !td::SET_FALLING.has(above.kind) {
                falling_refuse = false;
            }
        }
    }
    if falling_refuse {
        return false;
    }
    // `:40088-40091` — the wet spikes 546/557 refuse.
    let liquid = app.world.tile(x, y).map_or(0, |t| t.liquid);
    if matches!(current, 546 | 557) && liquid > 0 {
        return false;
    }
    true
}

/// `Player.PlaceThing_ValidTileForReplacement` (`Player.cs:39915-40044`).
/// `target`/`style` are the biome-converted held-item values (the caller
/// resolves them, `:39917-39926`).
fn valid_tile_for_replacement(app: &App, target: u16, style: i32, x: i32, y: i32) -> bool {
    let held_kind = app.inventory[app.selected].kind;
    if is_rope_item(held_kind) {
        return false;
    }
    if td::TILE_MOSS.has(target) {
        return false;
    }
    if td::SET_DOESNT_PLACE_WITH_TILE_REPLACEMENT.has(target) {
        return false;
    }
    let Some(tile) = app.world.tile(x, y).copied() else {
        return false;
    };
    if td::SET_DOESNT_GET_REPLACED_WITH_TILE_REPLACEMENT.has(tile.kind) {
        return false;
    }
    if !specific_validity_ok(app, target, tile.kind, x, y) {
        return false;
    }
    if !platform_solidity_ok(app, target, tile.kind, x, y) {
        return false;
    }
    if tr_core::tileobject::tile_cut(tile.kind) {
        return false;
    }
    // `:39956-39993` — the same-kind style swaps: platforms compare
    // frameY/18, torches frameY/22 (the wet-torch gate first), campfires
    // frameX/54 (the wet gate reads the target's WaterPlacement).
    if td::SET_PLATFORMS.has(tile.kind) && tile.kind == target {
        return i32::from(tile.frame_y) != style * 18;
    }
    if td::SET_TORCHES.has(tile.kind) {
        if tile.liquid > 0 {
            if target == 4 && !matches!(style, 8 | 11 | 17) {
                return false;
            }
            if !water_placement_allowed(target, style) {
                return false;
            }
        }
        if tile.kind == target {
            return i32::from(tile.frame_y) != style * 22;
        }
    }
    if td::SET_CAMPFIRES.has(tile.kind) {
        if tile.liquid > 0 && !water_placement_allowed(target, style) {
            return false;
        }
        if tile.kind == target {
            return i32::from(tile.frame_x) / 54 != style;
        }
    }
    // `:39994-40009` — the chest/dresser pairs: a different style always
    // replaces, the same style only when the kinds differ.
    if td::SET_BASIC_CHEST.has(tile.kind) && td::SET_BASIC_CHEST.has(target) {
        if i32::from(tile.frame_x) / 36 == style {
            return tile.kind != target;
        }
        return true;
    }
    if td::SET_BASIC_DRESSER.has(tile.kind) && td::SET_BASIC_DRESSER.has(target) {
        if i32::from(tile.frame_x) / 54 == style {
            return tile.kind != target;
        }
        return true;
    }
    // `:40010-40022` — the rope exception, then the frame-important gate.
    let mut rope_exception = false;
    if td::TILE_ROPE.has(tile.kind) {
        rope_exception = target == 314 || target == 380 || td::SET_PLATFORMS.has(target);
        if !rope_exception {
            return false;
        }
    }
    if !rope_exception
        && tr_core::tileobject::frame_important(target)
        && !td::SET_PLATFORMS.has(target)
    {
        return false;
    }
    if tile.kind == target {
        return false;
    }
    // `:40027-40030` — the getGoodWorld trap dart 230 refuses.
    if tile.kind == 230
        && app
            .world
            .seeds
            .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY)
    {
        return false;
    }
    // `:40031-40038` — the drop-equals-held refusal (the swap would mint
    // the item it consumes). The port's drop table is the `cell_drops`
    // slice + the large-object arms; the RNG-rolling kinds (herbs/trees)
    // never equal a block item, so the deterministic probe is faithful
    // for every reachable swap pair.
    if !td::SET_IGNORES_TILE_REPLACEMENT_DROP_CHECK_WHEN_BEING_PLACED.has(target) {
        if let Some(drop) = replacement_drop_item(app, x, y, &tile) {
            if drop == held_kind {
                return false;
            }
        }
    }
    edit::would_tile_replacement_work(&app.world, target, x, y)
}

/// `TileObjectData.GetTileData(type, style).WaterPlacement == Allowed`
/// (`Player.cs:39968-39972`/:39983-39987): a null def passes (the C#'s
/// `tileData != null &&` guard).
fn water_placement_allowed(target: u16, style: i32) -> bool {
    if style < 0 {
        return true;
    }
    let Some(def) = tr_core::tileobject::def_for_tile(target) else {
        return true;
    };
    let Some(data) = tr_core::tileobject::get_tile_data(def, style, 0) else {
        return true;
    };
    data.water_placement == tr_core::tileobject::LiquidPlacement::Allowed
}

/// The deterministic `KillTile_GetItemDrops` probe for the drop-equals-held
/// refusal (`Player.cs:40033`): the call passes NO `includeLargeObjectDrops`
/// (the default false, `WorldGen.cs:63964`) — the chest/dresser/campfire
/// kinds (21/467/88/215) sit in the no-drop case list
/// (`:65757`/:65775/:65821/:65944 → `break`, dropItem 0), so they never
/// refuse here. Only the port's `cell_drops` slice feeds the probe. The
/// RNG-rolling arms (herbs 83/84, the tree kinds) are suppressed — their
/// drops are never a placeable block item, so they cannot equal the held
/// block. `Main.rand` is a fresh throwaway stream here: the probe's rolls
/// never feed the real drop (the C# `KillTile_GetItemDrops` call at
/// `:40033` DOES roll on the real stream — a recorded deviation: the
/// probe's herb/tree rolls are skipped, so the stream stays untouched;
/// the reachable swap pairs never roll).
fn replacement_drop_item(app: &App, x: i32, y: i32, tile: &tr_core::tile::Tile) -> Option<u16> {
    let hit = HitCell {
        x,
        y,
        kind: tile.kind,
        frame_x: tile.frame_x,
        frame_y: tile.frame_y,
    };
    let ctx = drop_ctx(app);
    let mut probe_rng = tr_core::UnifiedRandom::new(0);
    cell_drops(&hit, false, &mut probe_rng, &ctx)
        .first()
        .map(|&(id, _, _, _)| id)
}

/// `WorldGen.GetItemDrop_Chests(style, secondType)`
/// (`WorldGen.cs:41554-41839`).
fn chest_drop_item(style: i32, second: bool) -> u16 {
    if second {
        return match style {
            1 => 3885,
            2 => 3939,
            3 => 3965,
            4 => 3988,
            5 => 4153,
            6 => 4174,
            7 => 4195,
            8 => 4216,
            9 => 4265,
            10 => 4267,
            11 => 4574,
            12 | 13 => 4712,
            14 => 5156,
            15 => 5177,
            16 => 5198,
            17 => 5556,
            18 => 5609,
            19 => 5697,
            20 => 5720,
            21 => 5745,
            22 => 5763,
            23 => 5784,
            24 => 5805,
            25 => 5826,
            26 => 5846,
            27 => 5865,
            28 => 5886,
            29 => 5905,
            30 => 5939,
            31 => 5962,
            32 => 5982,
            33 => 6005,
            34 => 6028,
            35 => 6051,
            36 => 6074,
            37 => 6118,
            _ => 3884,
        };
    }
    match style {
        0 => 48,
        1 | 2 => 306,
        3 | 4 => 328,
        5 => 343,
        6 => 348,
        7 => 625,
        8 => 626,
        9 => 627,
        10 => 680,
        11 => 681,
        12 => 831,
        13 => 838,
        14 => 914,
        15 => 952,
        16 => 1142,
        17 => 1298,
        18 | 23 => 1528,
        19 | 24 => 1529,
        20 | 25 => 1530,
        21 | 26 => 1531,
        22 | 27 => 1532,
        28 => 2230,
        29 => 2249,
        30 => 2250,
        31 => 2526,
        32 => 2544,
        33 => 2559,
        34 => 2574,
        35 | 36 => 2612,
        37 | 38 => 2613,
        39 | 40 => 2614,
        41 => 2615,
        42 => 2616,
        43 => 2617,
        44 => 2618,
        45 => 2619,
        46 => 2620,
        47 => 2748,
        48 => 2814,
        49 => 3180,
        50 => 3125,
        51 => 3181,
        _ => 48,
    }
}

/// `WorldGen.GetItemDrop_Dressers(style)` (`WorldGen.cs:49855-50037`).
fn dresser_drop_item(style: i32) -> u16 {
    if (1..=3).contains(&style) {
        return (646 + style) as u16;
    }
    if style == 4 {
        return 918;
    }
    if (5..=15).contains(&style) {
        return (2386 + style - 5) as u16;
    }
    match style {
        16 => 2529,
        17 => 2545,
        18 => 2562,
        19 => 2577,
        20 => 2637,
        21 => 2638,
        22 => 2639,
        23 => 2640,
        24 => 2816,
        25 => 3132,
        26 => 3134,
        27 => 3133,
        28 => 3911,
        29 => 3912,
        30 => 3913,
        31 => 3914,
        32 => 3934,
        33 => 3968,
        34 => 4148,
        35 => 4169,
        36 => 4190,
        37 => 4211,
        38 => 4301,
        39 => 4569,
        40 => 5151,
        41 => 5172,
        42 => 5193,
        43 => 5551,
        44 => 5604,
        45 => 5692,
        46 => 5715,
        47 => 5741,
        48 => 5766,
        49 => 5787,
        50 => 5808,
        51 => 5829,
        52 => 5848,
        53 => 5868,
        54 => 5888,
        55 => 5908,
        56 => 5942,
        57 => 5965,
        58 => 5985,
        59 => 6008,
        60 => 6031,
        61 => 6054,
        62 => 6077,
        63 => 6099,
        64 => 6121,
        _ => 334,
    }
}

/// `WorldGen.GetCampfireItemDrop(style)` (`WorldGen.cs:50039-50074`).
fn campfire_drop_item(style: i32) -> u16 {
    match style {
        0 => 966,
        1..=5 => (3046 + style - 1) as u16,
        6 => 3723,
        7 => 3724,
        8..=13 => (4689 + style - 8) as u16,
        14 => 5299,
        15 => 5357,
        _ => 966,
    }
}

/// `WorldGen.AttemptFossilShattering` (`WorldGen.cs:63484-63515`): the
/// desert fossil 404's 3×3 shatter — each neighbour fossil dies on a
/// `genRand.Next(maxValue) == 0` roll (15, or 4 when unsupported / the
/// above-centre cell on a real break). The kills are `noItem: true`; the
/// `fossilBreak` re-entrancy flag is a pass-local here (the C# static
/// guards the nested KillTile's own shatter — the port's `kill_tile`
/// never re-enters this fn, so the flag is vacuous).
fn attempt_fossil_shattering(app: &mut App, x: i32, y: i32, old: &tr_core::tile::Tile) {
    if old.kind != 404 {
        return;
    }
    for k in x - 1..=x + 1 {
        for l in y - 1..=y + 1 {
            let mut max_value = 15;
            if !solid_tile_probe(&app.world, k, l + 1) {
                max_value = 4;
            } else if k == x && l == y - 1 {
                max_value = 4;
            }
            let is_fossil = app
                .world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 404);
            if (k != x || l != y) && is_fossil && app.world.frame_rng.next_max(max_value) == 0 {
                // `KillTile(k, l, fail: false, effectOnly: false, noItem:
                // true)` — the cascade runs, the drops suppress. The
                // sound/dust the kill plays ride the break_tile seam
                // minus the loot (the noItem flag).
                super::break_tile_no_item(app, k, l);
            }
        }
    }
}

/// `WorldGen.SolidTile(k, l + 1)` (`WorldGen.cs:70111-70131`) — the
/// fossil-shatter support probe. The core's `solid_tile` is pub(crate);
/// the predicate is re-derived here (the `bug_net.rs:981` precedent).
fn solid_tile_probe(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && td::TILE_SOLID.has(t.kind)
            && !td::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `Player.PlaceThing_TryReplacingTiles` (`Player.cs:39841-39913`).
/// `can_use` is the wand-usability chain's result (the port's placement
/// route has no wand items yet — the caller passes true, the
/// `CheckWandUsability`/`CheckFlexibleWand` arms' default). The caller
/// gates `controlUseItem && itemAnimation > 0 && ItemTimeIsZero` (the
/// place arm's `inside_swing`).
pub(super) fn try_replacing_tiles(app: &mut App, tx: i32, ty: i32, can_use: bool) -> SwapOutcome {
    let held = app.inventory[app.selected];
    let Some(target) = registry::place_tile(held.kind) else {
        return SwapOutcome::Proceed;
    };
    let mut style = registry::place_style(held.kind);
    // `:39919-39926` — the biome-torch/campfire style conversion (the
    // place_tile route's `biome_torch_place_style` precedent; the campfire
    // twin is `BiomeCampfirePlaceStyle`, `:40348-40401`).
    if let Some(p) = app.player.as_ref() {
        if p.unlocked_biome_torches {
            if target == 4 {
                style = super::biome_torch_place_style(p, &app.world);
            } else if target == 215 && style == 0 {
                style = biome_campfire_place_style(p, &app.world);
            }
        }
    }
    let valid = valid_tile_for_replacement(app, target, style, tx, ty);
    // `:39848` — the action gate: controlUseItem && canUse && active &&
    // itemAnimation > 0 && ItemTimeIsZero && valid. The caller gates the
    // button/animation/time (the place arm's `inside_swing`); the active
    // check reads here.
    let active = app.world.tile(tx, ty).is_some_and(|t| t.active());
    if !can_use || !active || !valid {
        return SwapOutcome::Proceed;
    }
    // `:39850-39854` — the best pickaxe.
    let Some((pick_kind, pick_power)) = best_pickaxe(app) else {
        return SwapOutcome::Blocked;
    };
    let Some(tile) = app.world.tile(tx, ty).copied() else {
        return SwapOutcome::Blocked;
    };
    // `:39857-39862` — the hitReplace book + the damage (the getGoodWorld
    // double). The mannequin/rack redirects ride the same seam as the
    // mining book (`pick_tile_hit`'s convention).
    let buffer = app.hit_replace.hit_object(tx, ty, tr_core::hit_tile::TILE);
    let w = pick_world(app);
    let mut damage = tr_core::tool::pickaxe_damage(
        tx,
        ty,
        pick_power,
        tile.kind,
        i32::from(tile.frame_x),
        i32::from(tile.frame_y),
        &w,
        |x, y| app.world.tile(x, y).copied(),
    );
    let (redir_x, redir_y) = super::pick_redirect_target(&app.world, tx, ty, tile);
    if (redir_x, redir_y) != (tx, ty) {
        // The C# QUIRK: `GetPickaxeDamage` updates `this.hitTile`
        // (`Player.cs:53990`/:54004/:54010) even when the block-swap passes
        // a `hitReplace` buffer index (`:39857-39858`) — the redirect lands
        // in the MINING book, bug-for-bug.
        app.hit_tile.update_position(buffer, redir_x, redir_y);
    }
    if w.get_good {
        damage *= 2;
    }
    if damage == 0 {
        return SwapOutcome::Blocked;
    }
    // `:39867-39870` — the second IsTileReplaceable gate.
    if !edit::is_tile_replaceable(&app.world, tx, ty) {
        return SwapOutcome::Blocked;
    }
    // `:39873-39889` — the accumulating hit: below 100 the fail tick plays
    // (dust + sound), the consumable refund (`stack++` — the C# refunds
    // the ALREADY-consumed item; the port consumes at the placement tail,
    // so the refund is a no-op here, recorded), the item-time re-arm.
    let total = app.hit_replace.add_damage(buffer, damage, true);
    if total < 100 {
        // `:39875-39879` — the fail dust is the recorded visual skip
        // (the module header); the sound plays.
        let sounds = edit::kill_tile_sound(tile.kind, tile.frame_x, true);
        for cue in [sounds.primary, sounds.extra, sounds.shatter]
            .into_iter()
            .flatten()
        {
            if let Some((name, volume)) = super::kill_cue(cue) {
                app.audio
                    .play_at(name, tx as f32 * 16.0, ty as f32 * 16.0, volume);
            }
        }
        // `:39886-39887` — `ApplyItemTime(bestPickaxe, pickSpeed)` +
        // `SetItemAnimation(bestPickaxe.useTime, pickSpeed)` — the SAME
        // scaled useTime for both (the C# passes `bestPickaxe.useTime` to
        // the animation too, NOT useAnimation).
        let pick_speed = app.player.as_ref().map_or(1.0, |p| p.pick_speed());
        let use_time = i32::from(registry::get(pick_kind).map_or(0, |d| d.use_time));
        let mut val = (use_time as f32 * pick_speed) as i32;
        if use_time > 0 {
            val = val.max(1);
        }
        let secs = val as f32 * ONE_TICK_SECS;
        return SwapOutcome::Timed(secs, secs);
    }
    // `:39890` — `ClearMiningCacheAt(x, y, 1)` clears BOTH books
    // (`:53846-53850`: hitReplace first, then hitTile). The closures read
    // the DISJOINT `world` field (the `flatten_torch_supports` precedent).
    let world = &app.world;
    let active = |x: i32, y: i32| world.tile(x, y).is_some_and(|t| t.active());
    let wall_at = |x: i32, y: i32| world.tile(x, y).map_or(0, |t| t.wall);
    app.hit_replace
        .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, active, wall_at);
    app.hit_tile
        .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, active, wall_at);
    // `:39892` — the pre-replacement snapshot (the SAME cache the
    // autoPaint/autoActuate pass reads, `:39909`).
    let cache = auto_accessory_cache(&app.world, tx, ty);
    // `:39903` — `WorldGen.ReplaceTile`: the gates re-run inside, then the
    // kill-effects (dust → sound → drops → fossil shatter) and the write.
    // The port splits the write out (`replace_tile_write`) so the effects
    // keep the C# order with the app's pools.
    if !edit::would_tile_replacement_work(&app.world, target, tx, ty)
        || !edit::is_tile_replaceable(&app.world, tx, ty)
    {
        return SwapOutcome::Proceed;
    }
    let (ax, ay) = edit::move_replace_anchor(&app.world, tx, ty);
    let Some(old) = app.world.tile(tx, ty).copied() else {
        return SwapOutcome::Proceed;
    };
    // `:62570-62574` — the success dust is the recorded visual skip (the
    // module header).
    // `:62575` — the sounds.
    let sounds = edit::kill_tile_sound(old.kind, old.frame_x, false);
    for cue in [sounds.primary, sounds.extra, sounds.shatter]
        .into_iter()
        .flatten()
    {
        if let Some((name, volume)) = super::kill_cue(cue) {
            app.audio
                .play_at(name, ax as f32 * 16.0, ay as f32 * 16.0, volume);
        }
    }
    // `:62576` — the drops (`includeLargeObjectDrops: true`): the OLD
    // cell's item at the anchor. The port's `cell_drops` + the
    // large-object arms; the RNG arms roll on the real UpdatePlayers
    // stream (the C# order).
    let hit = HitCell {
        x: ax,
        y: ay,
        kind: old.kind,
        frame_x: old.frame_x,
        frame_y: old.frame_y,
    };
    let ctx = drop_ctx(app);
    let mut drops: Vec<(u16, u32, i32, i32)> = Vec::new();
    match old.kind {
        21 | 467 => drops.push((
            chest_drop_item(i32::from(old.frame_x) / 36, old.kind == 467),
            1,
            ax,
            ay,
        )),
        88 => drops.push((dresser_drop_item(i32::from(old.frame_x) / 54), 1, ax, ay)),
        215 => drops.push((campfire_drop_item(i32::from(old.frame_x) / 54), 1, ax, ay)),
        _ => {}
    }
    {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        drops.extend(cell_drops(&hit, false, rng, &ctx));
    }
    for (id, count, dx, dy) in drops {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        super::spawn_item_drop(&mut app.items, rng, id, count, dx, dy);
    }
    // `:62577` — the fossil shatter.
    attempt_fossil_shattering(app, ax, ay, &old);
    // The rigged-chest trip (`WorldGen.cs:62586-62590`) — the
    // DoActualReplacement HEAD, AFTER the kill-effects and BEFORE the
    // write; the C# trips on the OLD tile at the anchor (netMode != 1).
    if td::SET_BASIC_CHEST.has(target) && crate::app::netglue::is_rigged(&app.world, ax, ay) {
        app.wiring.hit_switch(&mut app.world, ax, ay);
        for (px, py) in app.wiring.take_switch_clicks() {
            app.audio.play_switch_click(
                px as f32,
                py as f32,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        }
        app.send_hit_switch(ax, ay);
    }
    // `:62578` — the write itself.
    let chests = app.world_chests.clone();
    let holds = |x: i32, y: i32| {
        chests
            .iter()
            .any(|c| c.tile == (x, y) && c.slots.iter().any(|s| s.count > 0))
    };
    let Some(replaced) = edit::replace_tile_write(&mut app.world, tx, ty, target, style, holds)
    else {
        return SwapOutcome::Proceed;
    };
    // The natural-extras kills (`ReplaceTile_EliminateNaturalExtras`'
    // plain `KillTile(x, y±1)`, `WorldGen.cs:62655`/:62659) are FULL kills:
    // their own sounds + drops + cascade run through the shared break seam
    // (the `break_tile` path — `KillTile_PlaySounds` at `:63558`,
    // `KillTile_DropItems` at `:63863`). The kill already landed in the
    // core write; the seam's own `kill_tile` finds the cell dead and the
    // effects below run off the recorded `TileKill`.
    for (kx, ky, kill) in &replaced.extras_kills {
        let sounds = edit::kill_tile_sound(kill.kind, 0, false);
        for cue in [sounds.primary, sounds.extra, sounds.shatter]
            .into_iter()
            .flatten()
        {
            if let Some((name, volume)) = super::kill_cue(cue) {
                app.audio
                    .play_at(name, *kx as f32 * 16.0, *ky as f32 * 16.0, volume);
            }
        }
        let hit = Some(HitCell {
            x: *kx,
            y: *ky,
            kind: kill.kind,
            frame_x: 0,
            frame_y: 0,
        });
        let ctx = drop_ctx(app);
        let (drops, _portals) = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            break_drop_events(kill, &app.world, hit, false, 0, rng, &ctx)
        };
        for (id, count, dx, dy) in drops {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            crate::world_sim::items::spawn_item_drop_px(
                &mut app.items,
                rng,
                id,
                count,
                glam::Vec2::new(dx, dy),
            );
        }
        app.request_edit(*kx, *ky, 0, 0);
    }
    // `:39905-39908` — canUse = false, the net 17/21 (the edit record
    // carries the post-swap tile), the item-time + animation re-arm:
    // `SetItemTime((int)(bestPickaxe.useTime * pickSpeed) + (int)(HeldItem
    // .useTime * tileSpeed))` + `SetItemAnimation(bestPickaxe.useTime,
    // pickSpeed)`.
    app.request_edit(tx, ty, 1, target);
    let pick_speed = app.player.as_ref().map_or(1.0, |p| p.pick_speed());
    let tile_speed = app.player.as_ref().map_or(1.0, |p| p.tile_speed_multiplier());
    let pick_use_time = i32::from(registry::get(pick_kind).map_or(0, |d| d.use_time));
    let pick_time = (pick_use_time as f32 * pick_speed) as i32;
    let held_time = (i32::from(registry::get(held.kind).map_or(0, |d| d.use_time)) as f32
        * tile_speed) as i32;
    let item_secs = (pick_time + held_time) as f32 * ONE_TICK_SECS;
    let mut anim = (pick_use_time as f32 * pick_speed) as i32;
    if pick_use_time > 0 {
        anim = anim.max(1);
    }
    let swing_secs = anim as f32 * ONE_TICK_SECS;
    // `:39909` — the autoPaint/autoActuate pass on the swapped cell.
    auto_actuate_after_place(app, tx, ty, target, &cache);
    // The replacement consumes the held item (the C# `item.consumable`
    // tick, `Player.cs:43280` — the placement route's convention).
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected] = crate::item::EMPTY_SLOT;
    }
    app.world_key = None;
    SwapOutcome::Timed(swing_secs, item_secs)
}

/// `Player.BiomeCampfirePlaceStyle` (`Player.cs:40348-40401`): the biome
/// campfire style for a plain campfire with the Torch God's favour. The
/// ZoneShimmer/ZoneUndergroundDesert arms read fields the port's `Zones`
/// does not carry — skipped in place (the `biome_torch_place_style`
/// precedent).
fn biome_campfire_place_style(p: &crate::player::Player, world: &World) -> i32 {
    let z = p.zones;
    if z.glowshroom {
        return 14;
    }
    if z.temple {
        return 13;
    }
    if z.dungeon {
        return 7;
    }
    let underworld_px = (world.height - 200) as f32 * 16.0;
    if p.y > underworld_px {
        return 2;
    }
    if z.hallow {
        return 12;
    }
    if z.corrupt {
        return 10;
    }
    if z.crimson {
        return 11;
    }
    if z.snow {
        return 3;
    }
    if z.jungle {
        return 13;
    }
    if z.desert && (p.y as f64) < world.world_surface * 16.0 {
        return 8;
    }
    if z.desert && world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
        return 8;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::super::tests::{object_world, slot};
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::FLAG_ACTIVE;

    /// A player holding a Dirt Block stack (item 2 → tile 0) with the
    /// Iron Pickaxe (item 1, pick 40) in slot 5; a stone block at (10, 19).
    fn swapper_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let p = crate::player::Player::spawn(&app.world);
        app.player = Some(p);
        app.inventory[app.selected] = slot(2, 10); // Dirt Block
        app.inventory[5] = slot(1, 1); // Iron Pickaxe (pick 40)
        let i = app.world.index(10, 19);
        app.world.tiles[i] = tr_core::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        app
    }

    /// The swap lands: the stone becomes dirt, the old stone drops, the
    /// held dirt burns, the tick is consumed (`Player.cs:39903-39909`).
    #[test]
    fn the_swap_replaces_the_kind_drops_the_old_and_burns_the_held() {
        let mut app = swapper_app();
        // The iron pick's 40 damage needs 3 hits to reach 100.
        assert!(matches!(
            try_replacing_tiles(&mut app, 10, 19, true),
            SwapOutcome::Timed(_, _)
        ));
        assert!(matches!(
            try_replacing_tiles(&mut app, 10, 19, true),
            SwapOutcome::Timed(_, _)
        ));
        let out = try_replacing_tiles(&mut app, 10, 19, true);
        assert!(matches!(out, SwapOutcome::Timed(_, _)), "the third hit lands");
        assert_eq!(app.world.tile(10, 19).unwrap().kind, 0, "dirt replaced stone");
        assert_eq!(app.inventory[app.selected].count, 9, "the dirt burned");
        assert!(
            app.items.iter().any(|d| d.kind == 3),
            "the old stone dropped"
        );
    }

    /// The accumulating hit: below 100 the fail tick re-arms the pick's
    /// cadence and the tile stays (`Player.cs:39873-39888`).
    #[test]
    fn the_below_100_hit_keeps_the_tile_and_rearms() {
        let mut app = swapper_app();
        let out = try_replacing_tiles(&mut app, 10, 19, true);
        assert!(matches!(out, SwapOutcome::Timed(_, _)));
        assert_eq!(app.world.tile(10, 19).unwrap().kind, 1, "still stone");
        assert_eq!(app.inventory[app.selected].count, 10, "nothing burned");
        assert!(app.items.is_empty(), "no drop on the fail tick");
    }

    /// No pickaxe in the inventory → the swap refuses outright
    /// (`GetBestPickaxe() == null`, `:39851-39853`).
    #[test]
    fn the_no_pickaxe_refusal() {
        let mut app = swapper_app();
        app.inventory[5] = crate::item::EMPTY_SLOT;
        assert!(matches!(
            try_replacing_tiles(&mut app, 10, 19, true),
            SwapOutcome::Blocked
        ));
        assert_eq!(app.world.tile(10, 19).unwrap().kind, 1);
    }

    /// The same-kind refusal (`:40023-40026`): dirt over dirt never swaps.
    #[test]
    fn the_same_kind_refuses() {
        let mut app = swapper_app();
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 0;
        assert!(matches!(
            try_replacing_tiles(&mut app, 10, 19, true),
            SwapOutcome::Proceed
        ));
    }

    /// The drop-equals-held refusal (`:40031-40038`): a Stone Block held
    /// over a gem sparkle 179 (which drops Stone 3,
    /// `WorldGen.cs:63999-64011`) — the swap would mint the item it
    /// consumes, so it refuses.
    #[test]
    fn the_drop_equals_held_refusal() {
        let mut app = swapper_app();
        app.inventory[app.selected] = slot(3, 10); // Stone Block
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 179;
        assert!(matches!(
            try_replacing_tiles(&mut app, 10, 19, true),
            SwapOutcome::Proceed
        ));
    }

    /// The platform style swap (`:39956-39959`): a wood platform held over
    /// a stone platform swaps the style (frameY 0 → 18).
    #[test]
    fn the_platform_style_swap() {
        let mut app = swapper_app();
        // Item 631 = Rich Mahogany Platform (style 1)? Use the Boreal
        // platform 1387 (style 9). Simpler: the same item 94 (platform,
        // style 0) over a style-1 platform frame.
        app.inventory[app.selected] = slot(94, 10);
        let i = app.world.index(10, 19);
        app.world.tiles[i] = tr_core::tile::Tile {
            kind: 19,
            frame_y: 18,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        // The platform is not tileNoFail — three 40-damage hits land.
        let mut out = SwapOutcome::Proceed;
        for _ in 0..3 {
            out = try_replacing_tiles(&mut app, 10, 19, true);
        }
        assert!(matches!(out, SwapOutcome::Timed(_, _)));
        assert_eq!(app.world.tile(10, 19).unwrap().frame_y, 0, "style 0 wrote");
    }

    /// The rope item never replaces (`:39928-39931`).
    #[test]
    fn the_rope_item_refuses() {
        let mut app = swapper_app();
        app.inventory[app.selected] = slot(965, 10); // Rope
        assert!(matches!(
            try_replacing_tiles(&mut app, 10, 19, true),
            SwapOutcome::Proceed
        ));
    }

    /// The autoActuator pass runs on the swapped cell (`:39909`): the
    /// fresh dirt takes the actuator bit.
    #[test]
    fn the_swap_feeds_the_auto_actuator_pass() {
        let mut app = swapper_app();
        app.player.as_mut().unwrap().auto_actuator = true;
        app.inventory[6] = slot(super::super::wiring::ACTUATOR_ITEM, 3);
        for _ in 0..3 {
            let _ = try_replacing_tiles(&mut app, 10, 19, true);
        }
        assert!(
            app.world.tile(10, 19).unwrap().actuator(),
            "the swapped dirt auto-actuated"
        );
        assert_eq!(app.inventory[6].count, 2);
    }
}
