//! The shimmer transform chain — M7-SHIMMER-1.
//!
//! The app half of the shimmer system: `WorldItem.GetShimmered`
//! (`WorldItem.cs:1691-1849`), the gates around it (`Item.CanShimmer`
//! `Item.cs:49448-49465`, `ShimmerTransforms.cs`'s lock helpers) and the recipe
//! reverse-index the decraft reads (`Recipe.UpdateWhichItemsAreCrafted`,
//! `Recipe.cs:15239-15256`, plus the post-Skeletron/Golem map
//! `ShimmerTransforms.cs:82-86`).
//!
//! The pure data tables (the transform sets) and the liquid predicate live in
//! [`tr_core::shimmer`]; this module resolves WHICH branch runs and what it
//! does, and [`crate::world_sim::items`] executes the outcome (spawns, the
//! physics, the world flags).

use crate::app::App;
use crate::craft::{self, NO_GROUP};
use crate::item::ItemDrop;
use glam::Vec2;
use crate::registry;
use tr_core::shimmer;

/// The item/world snapshot `Item.CanShimmer` and `GetShimmered` read. The item
/// fields mirror `Item`'s (`type`, `stack`, `makeNPC`, `placeStyle`) and the
/// world flags the C#'s `Main`/`NPC` globals (`Main.slimeRain`,
/// `NPC.unlockedSlimeRainbowSpawn`, `NPC.downedMoonlord`/`downedBoss3`,
/// `NPC.downedGolemBoss`, `WorldGen.crimson`, `Main.GetMoonPhase`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ShimmerContext {
    /// `Main.slimeRain` (`world.slime_rain`).
    pub slime_rain: bool,
    /// `NPC.unlockedSlimeRainbowSpawn` — the Main-global stand-in (a `World`
    /// field in the port).
    pub unlocked_slime_rainbow: bool,
    /// `NPC.downedMoonlord`.
    pub downed_moonlord: bool,
    /// `NPC.downedBoss3` — Skeletron.
    pub downed_skeletron: bool,
    /// `NPC.downedGolemBoss`.
    pub downed_golem: bool,
    /// `Main.GetMoonPhase()` — 0-7.
    pub moon_phase: u8,
    /// `WorldGen.crimson` (`World.crimson`, the persisted world evil type —
    /// W4-g69's field, `.wld` flags + `.trwld` v18 tail). The decraft variant
    /// preference (`ShimmerTransforms.cs:22-29`) reads it; `false` is the
    /// corruption default (`WorldFile.cs:3507`'s fallback).
    pub world_crimson: bool,
}

/// What one extraction does. `Execute` is the caller's job: the C# branch names
/// are on the variants so the apply stays 1:1 (`WorldItem.cs:1691-1849`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShimmerOutcome {
    /// No branch applies (the can-shimmer gate refused): the item keeps its
    /// stack, `shimmerTime` stays where it was (`WorldItem.cs:1696`'s implicit
    /// `else` — nothing runs).
    Nothing,
    /// The 4986/560 arms refuse because the event is already unlocked/active
    /// (`:1721-1722`, `:1737-1738`): the item does NOT air out and does not
    /// transform — vanilla returns before any change.
    Refused,
    /// `ItemID.Sets.CommonCoin` (`:1697-1719`): the stack merges by the tier
    /// (silver ×100, gold ×10000, platinum ×1000000 after flooring the stack to
    /// 1; copper has NO switch case — it stays ×1) and `AddCoinLuck` spends the
    /// merged stack.
    CoinMerge {
        /// The tier multiplier (1 / 100 / 10000 / 1000000).
        multiplier: u32,
    },
    /// `GetTransformToItem` (`:1713-1718`): `SetDefaults(target)`, the stack is
    /// preserved, `shimmered = true`.
    Transform { new_type: u16 },
    /// 4986 (`:1720-1735`): the slime-rainbow unlock — the world flag is set,
    /// NPC 681 spawns at the item, the stack drops by one.
    SlimeRainbowUnlock,
    /// 560 (`:1736-1746`): `Main.StartSlimeRain()`, the stack drops by one
    /// (or `shimmered = true` when the stack survives).
    SlimeRainStart,
    /// `makeNPC > 0` (`:1747-1772`): up to `attempts` releases of
    /// `ShimmerTransformToNPC[makeNPC]` when set, else `makeNPC` itself, with
    /// `shimmerTransparency = 1f` on the spawned NPC.
    ReleaseNpc {
        /// The NPC type to spawn (the transform override or `makeNPC`).
        npc_type: u16,
        /// How many releases the spawn-slot budget allows (≤ 50 attempts).
        attempts: u32,
    },
    /// The decraft (`:1773-1828`): `amount` units of the recipe's result are
    /// consumed and the ingredients spawn (`amount * stack` per entry, with the
    /// alchemy discount and the `maxStack` chunking — the caller resolves the
    /// list through [`de craft_ingredients`]).
    Decraft {
        /// The recipe index into `craft::RECIPES` (the C#'s
        /// `GetDecraftingRecipeIndex`).
        recipe_index: usize,
        /// `FindDecraftAmount()` — `stack / createItem.stack`.
        amount: u32,
    },
}

/// `Item.CanShimmer` (`Item.cs:49448-49465`).
pub fn can_shimmer(ctx: &ShimmerContext, item_type: u16, stack: u32, make_npc: u16) -> bool {
    let equivalent = shimmer::shimmer_equivalent_type(item_type, false);
    if shimmer::is_transform_locked(equivalent, ctx.downed_moonlord) {
        return false;
    }
    if equivalent == 560 && ctx.slime_rain {
        return false;
    }
    let create_tile = registry::place_tile(equivalent).map_or(-1, i32::from) as i16;
    let place_style = registry::place_style(equivalent);
    let transform = shimmer::transform_to_item(equivalent, create_tile, place_style, ctx.moon_phase);
    if (equivalent != 4986 || ctx.unlocked_slime_rainbow)
        && transform <= 0
        && !is_item_decraftable_and_unlocked(ctx, item_type, stack)
        && !shimmer::is_common_coin(equivalent)
    {
        return make_npc > 0;
    }
    true
}

/// `ShimmerTransforms.IsItemDec craftableAndIsDecraftUnlocked`
/// (`ShimmerTransforms.cs:64-80`): the recipe index exists, is not locked, and
/// the stack covers at least one result.
pub fn is_item_decraftable_and_unlocked(ctx: &ShimmerContext, item_type: u16, stack: u32) -> bool {
    let index =
        decrafting_recipe_index(ctx.world_crimson, shimmer::shimmer_equivalent_type(item_type, true));
    if index < 0 {
        return false;
    }
    if is_recipe_index_decraft_locked(index as usize) {
        return false;
    }
    let create_stack = craft::RECIPES[index as usize].create.1;
    stack / create_stack > 0
}

/// `ShimmerTransforms.GetDecraftingRecipeIndex` (`ShimmerTransforms.cs:15-31`):
/// the `IsCrafted` index, then the world-evil variant preference — a crimson
/// world takes `IsCraftedCrimson[type]` when set (`:22-25`), a corruption
/// world `IsCraftedCorruption[type]` (`:26-29`), else the plain index. -1 when
/// the type has no decrafting recipe. The three sets are
/// `Recipe.UpdateWhichItemsAreCrafted` (`Recipe.cs:15241-15257`): the LAST
/// matching row wins per set — the plain set gates on `!notDecraftable`, the
/// evil sets on the row's `crimson`/`corruption` tag ALONE (no
/// `notDecraftable` gate there). `world_crimson` is `WorldGen.crimson` — the
/// caller threads the persisted [`tr_core::world::World::crimson`] flag
/// (W4-g69's field; the pre-v18 corruption default).
pub fn decrafting_recipe_index(world_crimson: bool, item_type: u16) -> i32 {
    let mut found = -1;
    let mut crimson = -1;
    let mut corruption = -1;
    for (i, r) in craft::RECIPES.iter().enumerate() {
        if r.create.0 != item_type {
            continue;
        }
        if !r.not_decraftable {
            found = i as i32; // `:15245-15248`
        }
        if r.crimson {
            crimson = i as i32; // `:15249-15252`
        }
        if r.corruption {
            corruption = i as i32; // `:15253-15256`
        }
    }
    // `:18-21` — no plain recipe, no decraft (the evil preference only
    // applies when `IsCrafted[type] >= 0`).
    if found < 0 {
        return -1;
    }
    if world_crimson && crimson >= 0 {
        return crimson; // `ShimmerTransforms.cs:22-25`
    }
    if !world_crimson && corruption >= 0 {
        return corruption; // `:26-29`
    }
    found
}

/// `ShimmerTransforms.IsRecipeIndexDecraftLocked` (`ShimmerTransforms.cs:47-62`)
/// with the post-Skeletron/Golem flags from `UpdateRecipeSets`
/// (`:82-86` — `ContainsIngredient(154)` / `(1101)` per recipe).
pub fn is_recipe_index_decraft_locked(index: usize) -> bool {
    let Some(recipe) = craft::RECIPES.get(index) else {
        return false;
    };
    if recipe_contains_ingredient(recipe, 154) && !skeletron_down() {
        return true;
    }
    if recipe_contains_ingredient(recipe, 1101) && !golem_down() {
        return true;
    }
    false
}

/// The C# reads `NPC.downedBoss3`/`downedGolemBoss` — Main-global state the port
/// carries per world; the caller passes them through [`ShimmerContext`] and this
/// module's entry points are the pure overrides used by the tests. The default
/// (both downed) keeps the locks open when a caller forgets to thread them.
fn skeletron_down() -> bool {
    decraft_flags().0
}
fn golem_down() -> bool {
    decraft_flags().1
}

/// The (Skeletron, Golem) flags, defaulting to both downed. Overridden by
/// [`set_decraft_flags`] (the app wires the world's flags at the item step).
pub(crate) fn decraft_flags() -> (bool, bool) {
    DECRAFT_FLAGS.with(std::cell::Cell::get)
}

thread_local! {
    static DECRAFT_FLAGS: std::cell::Cell<(bool, bool)> = const { std::cell::Cell::new((true, true)) };
}

/// The app's wiring hook: the world's downed flags into the lock checks (so the
/// pure helpers above stay callable from the tests without a world).
pub(crate) fn set_decraft_flags(skeletron: bool, golem: bool) {
    DECRAFT_FLAGS.with(|c| c.set((skeletron, golem)));
}

/// `Recipe.ContainsIngredient` (`Recipe.cs:16756-16773`) over the port's
/// requirement list — the direct item ids only (the C# reads
/// `requiredItemQuickLookup`'s `itemIdOrRecipeGroup`, which for a group entry is
/// the GROUP id, so a bare 154/1101 ingredient is the only match).
fn recipe_contains_ingredient(recipe: &craft::Recipe, item_type: u16) -> bool {
    recipe.required.iter().any(|r| r.item == item_type)
}

/// `WorldItem.GetShimmered` (`WorldItem.cs:1691-1849`) as a decision function:
/// the chain order IS the branch order, and every arm keeps the C#'s exact
/// guard. `make_npc` is `Item.makeNPC` (the port's generated item data does not
/// carry the field yet — the release arm is roster-blocked until it does).
pub fn resolve_shimmer(
    ctx: &ShimmerContext,
    item_type: u16,
    stack: u32,
    make_npc: u16,
) -> ShimmerOutcome {
    // 1. Coins (`:1696-1712`).
    let equivalent = shimmer::shimmer_equivalent_type(item_type, false);
    if shimmer::is_common_coin(equivalent) {
        let multiplier = match equivalent {
            72 => 100,
            73 => 10_000,
            74 => 1_000_000,
            _ => 1,
        };
        return ShimmerOutcome::CoinMerge { multiplier };
    }
    // 2. Transform to item (`:1720-1726`). The C# resolves the transform on
    // the EQUIVALENT type (`:1695`) — the trophy/lunar arms read
    // `ContentSamples.ItemsByType[shimmerEquivalentType]`
    // (`ShimmerTransforms.cs:95-108`), so the createTile/placeStyle fields
    // come from the equivalent too (the old port read the RAW kind).
    let create_tile = registry::place_tile(equivalent).map_or(-1, i32::from) as i16;
    let place_style = registry::place_style(equivalent);
    let transform = shimmer::transform_to_item(equivalent, create_tile, place_style, ctx.moon_phase);
    if transform > 0 {
        return ShimmerOutcome::Transform {
            new_type: transform as u16,
        };
    }
    // 3. 4986 — the slime-rainbow unlock (`:1720-1735`).
    if item_type == 4986 {
        if ctx.unlocked_slime_rainbow {
            return ShimmerOutcome::Refused;
        }
        return ShimmerOutcome::SlimeRainbowUnlock;
    }
    // 4. 560 — the Slime Rain (`:1736-1746`).
    if item_type == 560 {
        if ctx.slime_rain {
            return ShimmerOutcome::Refused;
        }
        return ShimmerOutcome::SlimeRainStart;
    }
    // 5. The makeNPC release (`:1747-1772`) — ≤ 50 attempts.
    if make_npc > 0 {
        let npc_type = {
            let override_type = shimmer::transform_to_npc(make_npc);
            if override_type >= 0 {
                override_type as u16
            } else {
                make_npc
            }
        };
        let attempts = stack.min(50);
        return ShimmerOutcome::ReleaseNpc { npc_type, attempts };
    }
    // 6. The decraft (`:1788-1834`). The C# only checks the index here —
    // `FindDecraftAmount()` (`Item.cs:49466-49474`) is `stack /
    // createItem.stack`, which can be 0; the arm still runs (the ingredient
    // walk spawns nothing, `stack -= 0` keeps the drop) and falls through to
    // the shared tail. The `CanShimmer` gate normally refuses such a drop,
    // but `GetShimmered` itself has no `amount > 0` guard.
    let index =
        decrafting_recipe_index(ctx.world_crimson, shimmer::shimmer_equivalent_type(item_type, true));
    if index >= 0 {
        let index = index as usize;
        let amount = stack / craft::RECIPES[index].create.1;
        return ShimmerOutcome::Decraft {
            recipe_index: index,
            amount,
        };
    }
    ShimmerOutcome::Nothing
}

/// One decraft ingredient, resolved through the C#'s `requiredItemQuickLookup`
/// / `customShimmerResults` walk (`WorldItem.cs:1775-1828`): the group entries
/// spend the group's `DecraftItemId` (the cheapest member by value,
/// `RecipeGroup.cs:57-60`), the plain entries their own id. `units` is
/// `amount * entry.stack` AFTER the alchemy discount is applied by the caller's
/// RNG (the discount itself is the caller's, so the draw order stays with the
/// item step).
pub struct DecraftIngredient {
    /// The spawn id (the plain id or the group's dec item).
    pub item_type: u16,
    /// The total units to spawn (chunked by `maxStack` by the caller).
    pub units: u32,
}

/// `RecipeGroup.SortDecraftingEntries` (`RecipeGroup.cs:57-60`): the group's
/// dec id is the cheapest member by item value.
pub fn group_decraft_item(group_id: u16) -> Option<u16> {
    let group = craft::group(group_id)?;
    group
        .members
        .iter()
        .copied()
        .min_by_key(|&m| registry::get(m).map_or(0, |d| d.value))
}

/// The decraft's ingredient list for a recipe index (`WorldItem.cs:1793-1801`'s
/// `enumerable`), with `units` pre-discount. The `customShimmerResults`
/// override (`Recipe.cs:78`, `AddCustomShimmerResult` `:174-183`) replaces the
/// plain `requiredItem` walk when present — the five vanilla rows (Silver
/// Bullet 278, Crystal Bullet 4915, the copper tools 3509/3506/3507) spend
/// their authored result list instead (no recipe-group substitution there:
/// the C# `Select` reads `item.type` directly, `:1796-1800`).
pub fn decraft_ingredients(recipe_index: usize) -> Vec<DecraftIngredient> {
    let Some(recipe) = craft::RECIPES.get(recipe_index) else {
        return Vec::new();
    };
    if let Some(custom) = custom_shimmer_results(recipe.create.0) {
        return custom
            .iter()
            .map(|&(item_type, units)| DecraftIngredient { item_type, units })
            .collect();
    }
    recipe
        .required
        .iter()
        .take_while(|r| r.item > 0)
        .map(|r| {
            let item_type = if r.group != NO_GROUP {
                group_decraft_item(r.group).unwrap_or(r.item)
            } else {
                r.item
            };
            DecraftIngredient {
                item_type,
                units: r.stack,
            }
        })
        .collect()
}

/// `Recipe.AddCustomShimmerResult`'s five call sites (`Recipe.cs:2193-2194`,
/// `:2202-2203`, `:9664-9665`, `:9674-9675`, `:9694`) keyed by the created
/// item — the port's generated recipe rows carry no `customShimmerResults`
/// list (`tools/gen_recipes.py` does not extract it yet), so the override
/// lives here, verbatim. The C# `null` check (`WorldItem.cs:1794`) means an
/// EMPTY list never occurs — only these five recipes override.
fn custom_shimmer_results(create_type: u16) -> Option<&'static [(u16, u32)]> {
    match create_type {
        // `:2187-2195` — Silver Bullet x70: 70 Musket Balls + 1 Silver Bar
        // normally; the shimmer refunds the balls + a Silver ORE (14).
        278 => Some(&[(97, 70), (14, 1)]),
        // `:2196-2204` — Crystal Bullet x70: the shimmer refunds the balls
        // + 701 (the authored `AddCustomShimmerResult(701)` row) instead of
        // the 705 the craft spends.
        4915 => Some(&[(97, 70), (701, 1)]),
        // `:9657-9666` — Copper Pickaxe (3509): the shimmer refunds 1 Copper
        // Bar (12) + 1 Wood (9) — NOT the 8 bars / 4 wood the craft spends.
        3509 => Some(&[(12, 1), (9, 1)]),
        // `:9667-9676` — Copper Axe (3506): the same 1+1 refund.
        3506 => Some(&[(12, 1), (9, 1)]),
        // `:9690-9695` — Copper Shortsword (3507): 1 Copper Bar only.
        3507 => Some(&[(12, 1)]),
        _ => None,
    }
}

/// The app-side apply of [`ShimmerOutcome`] — `WorldItem.GetShimmered`'s
/// chain tail (`WorldItem.cs:1691-1849`): the coin merge's luck, the transform
/// replace, the two special events, the release spawn and the decraft's
/// ingredient spawns, then the shared tail (`:1830-1848`).
///
/// `indices` are the drops whose `shimmer_time` crossed 0.9 this step. The
/// spawns ride the same 60 Hz step (the C#'s `SyncItem`/net arm is MP-only, so
/// it is out here); the alchemy discount draws `stream(UpdateItems)` (the C#'s
/// `Main.rand` under the `UpdateItems` swap) exactly like the pot-loot rolls.
pub(crate) fn apply_due_transforms(app: &mut App, indices: &[u32]) {
    // The decraft lock checks read the world's downed flags through the
    // thread-local (the pure helpers stay world-free).
    set_decraft_flags(app.world.downed_skeletron, app.world.downed_golem);
    let ctx = ShimmerContext {
        slime_rain: app.world.slime_rain,
        unlocked_slime_rainbow: app.world.unlocked_slime_rainbow_spawn,
        downed_moonlord: app.world.downed_moon_lord,
        downed_skeletron: app.world.downed_skeletron,
        downed_golem: app.world.downed_golem,
        moon_phase: app.world.moon_phase,
        world_crimson: app.world.crimson,
    };
    for &idx in indices {
        let Some(drop) = app.items.get(idx as usize).cloned() else {
            continue;
        };
        // `Item.makeNPC` — the generated item data carries the column
        // (`registry::make_npc`, S1-P-i36); the release arm fires for the
        // critter/bait kinds whose NPC the roster models (the rest consume
        // the stack unit with no spawn — the C# accounting, ANALYSIS).
        let make_npc = crate::registry::make_npc(drop.kind);
        if !can_shimmer(&ctx, drop.kind, drop.count, make_npc) {
            continue;
        }
        let outcome = resolve_shimmer(&ctx, drop.kind, drop.count, make_npc);
        let Some(drop_mut) = app.items.get_mut(idx as usize) else {
            continue;
        };
        match outcome {
            ShimmerOutcome::Nothing | ShimmerOutcome::Refused => {}
            ShimmerOutcome::CoinMerge { multiplier } => {
                // `WorldItem.cs:1699-1719` — the platinum arm floors the stack
                // to 1 before the multiply; copper (71) has no switch case.
                let stack = if multiplier == 1_000_000 {
                    drop_mut.count.min(1)
                } else {
                    drop_mut.count
                };
                drop_mut.count = stack.saturating_mul(multiplier);
                // `AddCoinLuck` (`Player.cs:18090-18101`): the merged stack
                // within 1000 px, capped at 1,000,000. The C# sets
                // `luckNeedsSync` only — NO `RecalculateLuck` here (the
                // per-tick `UpdateLuck` rebuilds it, `:18840`); the old port
                // recalculated eagerly (a same-tick luck read deviation).
                let c = drop_mut.center();
                if let Some(p) = app.player.as_mut() {
                    p.add_coin_luck((c.x, c.y), drop_mut.count as i32);
                }
                air_out(app, idx);
            }
            ShimmerOutcome::Transform { new_type } => {
                // `:1720-1726` — SetDefaults + the stack is kept + shimmered.
                // The shared tail (`:1835-1846`) sets shimmerTime = 1 (the
                // stack survives), shimmerWet/wet, velocity *= 0.1 — applied
                // by `shimmer_tail` below, not per-arm.
                drop_mut.kind = new_type;
                drop_mut.shimmered = true;
            }
            ShimmerOutcome::SlimeRainbowUnlock => {
                // `:1727-1748` — the flag, the NPC 681 spawn, the stack--.
                // The C# `NewNPC((int)Center.X + 4, (int)Center.Y, 681)`
                // (`:1735`) truncates the pixel coords FIRST and spawns
                // DIRECTLY (no `ReleaseNPC` gate); the spawned NPC keeps the
                // item's velocity (`:1739`) and gets `shimmerTransparency =
                // 1f` (`:1740`). The spawn takes `app` wholesale, so the
                // drop borrow ends first.
                let (c, v) = (drop_mut.center(), drop_mut.vel);
                app.world.unlocked_slime_rainbow_spawn = true;
                let spawned = crate::world_sim::npcs::spawn_shimmer_new_npc(
                    app,
                    681,
                    c.x as i32 + 4,
                    c.y as i32,
                );
                if spawned {
                    if let Some(npc) = app.npcs.last_mut() {
                        npc.vx = v.x;
                        npc.vy = v.y;
                    }
                }
                let Some(drop_mut) = app.items.get_mut(idx as usize) else {
                    continue;
                };
                drop_mut.count = drop_mut.count.saturating_sub(1);
                if drop_mut.count == 0 {
                    air_out(app, idx);
                }
                // NOTE: the C# 4986 arm does NOT set `shimmered` — the drop
                // stays un-shimmered (it can transform again next ramp).
            }
            ShimmerOutcome::SlimeRainStart => {
                // `:1749-1765` — `Main.StartSlimeRain()` (the SP branch's
                // duration), the stack-- or `shimmered = true`. The C# call
                // runs in `WorldItem.UpdateItem` under the `UpdateItems`
                // swap (`Main.cs:18024`) — the duration draw rides
                // `UpdateItems` (S1-A-11 P4 formalizes the items pass).
                // StartSlimeRain's own gates apply (`Main.cs:64744-64764`):
                // the item path is `!Main.raining`-gated by the caller only
                // in vanilla, so pass the live rain state.
                let raining = app.rain.is_raining();
                crate::world_sim::weather::start_slime_rain(
                    &mut app.world,
                    raining,
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                );
                drop_mut.count = drop_mut.count.saturating_sub(1);
                if drop_mut.count == 0 {
                    air_out(app, idx);
                } else {
                    drop_mut.shimmered = true;
                }
            }
            ShimmerOutcome::ReleaseNpc { npc_type, attempts: _ } => {
                // `:1766-1787` — the C# loop: `num3 = 50` attempts, `num4` =
                // the free-slot budget (`GetAvailableAmountOfNPCsToSpawnUpToSlot`
                // `NPC.cs:80635-80654`), and `stack > 0`; EVERY iteration
                // decrements all three and the stack BEFORE the release —
                // a failed release still consumes one unit (the old port
                // only decremented on success and never ran the budget).
                // The release position is the TRUNCATED `(int)Center.X,
                // (int)Bottom.Y` (`:1776`), the override style is 0 when the
                // transform map hits, else the item's `placeStyle`.
                let make_npc = crate::registry::make_npc(app.items[idx as usize].kind);
                let style = if shimmer::transform_to_npc(make_npc) >= 0 {
                    0
                } else {
                    crate::registry::place_style(app.items[idx as usize].kind)
                };
                let mut budget = crate::world_sim::npcs::npc_spawn_budget(
                    app.npcs.len(),
                    app.items[idx as usize].count,
                );
                let mut left = 50u32;
                while left > 0 && budget > 0 && app.items[idx as usize].count > 0 {
                    left -= 1;
                    budget -= 1;
                    app.items[idx as usize].count -= 1;
                    let (cx, by) = {
                        let d = &app.items[idx as usize];
                        (d.center().x as i32, (d.pos.y + SIZE_PX) as i32)
                    };
                    crate::world_sim::npcs::spawn_shimmer_release_styled(
                        app, npc_type, cx, by, style,
                    );
                }
                app.items[idx as usize].shimmered = true;
                if app.items[idx as usize].count == 0 {
                    air_out(app, idx);
                }
            }
            ShimmerOutcome::Decraft {
                recipe_index,
                amount,
            } => {
                apply_decraft(app, idx, recipe_index, amount);
            }
        }
        // `WorldItem.cs:1835-1852` — the shared tail every arm falls through
        // to (the two `Refused` early returns and the `Nothing` gate never
        // reach it): `shimmerTime = stack > 0 ? 1 : 0`, `shimmerWet = true`,
        // `wet = true`, `velocity *= 0.1f`, `ShimmerEffect(Center)` (the
        // Item176 cue + 20 dust 309), `SyncItem` (net-only), progression 27
        // (TRANSMUTE_ITEM), and `TurnToAir` when the stack hit 0.
        if !matches!(outcome, ShimmerOutcome::Nothing | ShimmerOutcome::Refused) {
            shimmer_tail(app, idx);
            let _ = app.notify_progression_net(27);
        }
    }
    // The air-out sweep (`WorldItem.TurnToAir`'s list effect — the C# marks the
    // item inactive and the item array's compaction drops it): the zeroed
    // entries leave once every arm of this step has run, so the collected
    // indices stay valid during the applies.
    app.items.retain(|d| !(d.kind == 0 && d.count == 0));
}

/// The 16x16 item box in pixels (`ItemDrop::SIZE`).
const SIZE_PX: f32 = crate::item::SIZE;

/// `WorldItem.GetShimmered`'s shared tail (`WorldItem.cs:1835-1852`): the
/// `shimmerTime` reset (1 while a stack survives, else 0), the wet flags, the
/// ×0.1 velocity cut, `ShimmerEffect(Center)` and the final `TurnToAir`.
/// `SyncItem` (`:1847`) is net-only — the host's snapshot carries the drop.
fn shimmer_tail(app: &mut App, idx: u32) {
    let (center, alive) = {
        let Some(drop) = app.items.get_mut(idx as usize) else {
            return;
        };
        // `:1835-1842` — `stack > 0 ? shimmerTime = 1f : 0f`.
        drop.shimmer_time = if drop.count > 0 { 1.0 } else { 0.0 };
        // `:1843-1845` — shimmerWet + wet + the velocity cut (W4-e19: the
        // per-drop `wet` flag landed with the honey/water gravity arms).
        drop.shimmer_wet = true;
        drop.wet = true;
        drop.vel *= 0.1;
        (drop.center(), drop.count > 0)
    };
    // `:1846` — `WorldItem.ShimmerEffect(base.Center)` (`:1661-1689`): the
    // Item176 cue at the TRUNCATED pixel coords + 20 dust 309 (scale ×1.2,
    // the four-colour `Next(6)` switch). The draws ride `UpdateItems`
    // (`Main.cs:17779`'s swap — `GetShimmered` runs inside `UpdateItem`).
    shimmer_effect(app, center);
    // `:1849-1852` — `stack == 0` → `TurnToAir` (the arm already zeroed the
    // kind/count via `air_out`; the retain sweep drops it).
    if !alive {
        air_out(app, idx);
    }
}

/// `WorldItem.ShimmerEffect` (`WorldItem.cs:1661-1689`): `PlaySound(
/// SoundID.Item176, (int)x, (int)y)` then 20 × `Dust.NewDust(center, 1, 1,
/// 309)` with `scale *= 1.2f` and the `Next(6)` colour switch (0 →
/// (255,255,210), 1 → (190,245,255), 2 → (255,150,255), default →
/// (190,175,255)). The `netMode == 2` arm (`:1663-1667`) is the host's `146`
/// send — out here (SP port); the cue + dust run on the local client.
fn shimmer_effect(app: &mut App, center: Vec2) {
    // The cue: `SoundID.Item176 = new LegacySoundStyle(2, 176)`
    // (`SoundID.cs:1560`) — case 2's ×0.9 volume + the `Next(-6, 7) * 0.01`
    // pitch (`sound.rs::item_use_case2`), gated by attenuation FIRST.
    let idx = crate::soundid::SOUND_STYLES
        .iter()
        .position(|s| s.name == "Item176")
        .expect("Item176 in the generated SoundID table") as u16;
    app.audio.play_use_sound(
        idx,
        center.x as i32 as f32,
        center.y as i32 as f32,
        0.0,
        app.rngs.stream(crate::app::RngKey::UpdateItems),
    );
    // The 20 dust: `NewDust(center, 1, 1, 309)` — the 1x1 box pads to 5x5
    // inside `NewDust` (`Dust.cs:155-162`); the colour switch draws AFTER
    // the spawn (the C# order: NewDust's own draws, then `Next(6)`).
    let ctx = app.spawn_ctx();
    for _ in 0..20 {
        let slot = app.dust.new_dust(
            app.rngs.stream(crate::app::RngKey::UpdateItems),
            &ctx,
            &app.world,
            center,
            1,
            1,
            309,
            0.0,
            0.0,
            0,
            tr_core::particles::Color::default(),
            1.0,
        );
        let d = app.dust.slot_mut(slot);
        d.scale *= 1.2;
        let roll = app.rngs.stream(crate::app::RngKey::UpdateItems).next_max(6);
        d.color = match roll {
            0 => tr_core::particles::Color::new(255, 255, 210, 255),
            1 => tr_core::particles::Color::new(190, 245, 255, 255),
            2 => tr_core::particles::Color::new(255, 150, 255, 255),
            _ => tr_core::particles::Color::new(190, 175, 255, 255),
        };
    }
}

/// The air-out (`WorldItem.TurnToAir`'s effect on the drop list, `:1845-1848`).
fn air_out(app: &mut App, idx: u32) {
    if let Some(drop) = app.items.get_mut(idx as usize) {
        drop.kind = 0;
        drop.count = 0;
    }
}

/// `WorldItem.GetShimmered`'s decraft arm (`:1788-1834`): per ingredient
/// `amount * entry.stack` units, the alchemy discount (1/3 of the units dropped,
/// one `Next(3)` per unit), `maxStack` chunking and the final
/// `stack -= amount * createItem.stack`.
fn apply_decraft(app: &mut App, idx: u32, recipe_index: usize, amount: u32) {
    let center = {
        let drop = &app.items[idx as usize];
        drop.center()
    };
    let ingredients = decraft_ingredients(recipe_index);
    let alchemy = craft::RECIPES.get(recipe_index).is_some_and(|r| r.alchemy);
    // The C# spawns at `base.Center` (`SpawnShimmeredItem` `:623-626` passes
    // `base.Center` to the static `:628-637`), with the per-spawn index
    // `num7` driving the velocity `(num7 * (num7 % 2 * 2 - 1), 0)` — an
    // alternating left/right fan, NOT the random scatter.
    let mut spawns: Vec<(u16, u32)> = Vec::new();
    for ing in ingredients {
        let mut units = ing.units * amount;
        if alchemy {
            // `:1811-1820` — one `Main.rand.Next(3) == 0` per unit; the
            // decraft runs in `WorldItem.GetShimmered` under the `UpdateItems`
            // swap (`Main.cs:17779`), so the draws ride `UpdateItems`.
            for _ in 0..ing.units * amount {
                if app.rngs.stream(crate::app::RngKey::UpdateItems).next_max(3) == 0 {
                    units = units.saturating_sub(1);
                }
            }
        }
        let max_stack = crate::registry::max_stack(ing.item_type);
        while units > 0 {
            let chunk = units.min(max_stack);
            units -= chunk;
            spawns.push((ing.item_type, chunk));
        }
    }
    for (n, (kind, count)) in spawns.into_iter().enumerate() {
        // `SpawnShimmeredItem` (`:628-637`) → `Item.RequestNewItem(source,
        // center, type, stack, prefix: 0, ownership: None, velocity:
        // (n * (n % 2 * 2 - 1), 0), modifier)`. The modifier sets
        // shimmerTime = 1, shimmered = true, shimmerWet = true, wet = true.
        // `prefix: 0` draws NOTHING (`Item.Prefix` `:511-514` returns early);
        // the explicit velocity skips the two scatter draws (`Item.cs:49738-
        // 49741`) — so a shimmered spawn is RNG-FREE apart from the alchemy
        // discount above. The old port ran the full random scatter + prefix
        // roll (a stream deviation).
        let vx = (n as i32 * (n as i32 % 2 * 2 - 1)) as f32;
        let mut drop = ItemDrop::new(kind, count, 0, 0.0, 0.0, vx, 0.0, 0.0);
        // `worldItem.Center = center` (`Item.cs:49736`) — the box centres on
        // the source drop's centre.
        drop.pos = center - Vec2::splat(SIZE_PX * 0.5);
        drop.shimmer_time = 1.0;
        drop.shimmered = true;
        drop.shimmer_wet = true;
        drop.wet = true; // `WorldItem.cs:635` — the modifier sets `wet` too.
        app.items.push(drop);
        if app.items.len() > 300 {
            app.items.remove(0);
        }
    }
    // `:1828-1833` — `stack -= amount * createItem.stack`; `stack <= 0` →
    // `stack = 0; type = 0`. The C# does NOT set `shimmered` on a surviving
    // decraft source (the old port did — a deviation: the source keeps
    // ramping and can decraft again).
    let create_stack = craft::RECIPES.get(recipe_index).map_or(1, |r| r.create.1);
    if let Some(drop) = app.items.get_mut(idx as usize) {
        drop.count = drop.count.saturating_sub(amount * create_stack);
        if drop.count == 0 {
            air_out(app, idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ShimmerContext {
        ShimmerContext {
            slime_rain: false,
            unlocked_slime_rainbow: false,
            downed_moonlord: false,
            downed_skeletron: true,
            downed_golem: true,
            moon_phase: 0,
            world_crimson: false,
        }
    }

    #[test]
    fn coins_merge_by_tier() {
        // `WorldItem.cs:1696-1712` — the tier table.
        assert_eq!(
            resolve_shimmer(&ctx(), 71, 5, 0),
            ShimmerOutcome::CoinMerge { multiplier: 1 }
        );
        assert_eq!(
            resolve_shimmer(&ctx(), 72, 5, 0),
            ShimmerOutcome::CoinMerge { multiplier: 100 }
        );
        assert_eq!(
            resolve_shimmer(&ctx(), 73, 5, 0),
            ShimmerOutcome::CoinMerge { multiplier: 10_000 }
        );
        // The platinum arm floors the stack to 1 in the C# before the multiply;
        // the caller applies that (`WorldItem.cs:1710-1713`).
        assert_eq!(
            resolve_shimmer(&ctx(), 74, 5, 0),
            ShimmerOutcome::CoinMerge {
                multiplier: 1_000_000
            }
        );
    }

    #[test]
    fn transform_beats_decraft_and_the_release() {
        // The chain order: a set hit resolves as the transform even when a
        // decraft exists.
        let out = resolve_shimmer(&ctx(), 11, 99, 0); // Iron Ore → Tin Ore
        assert_eq!(out, ShimmerOutcome::Transform { new_type: 699 });
    }

    #[test]
    fn the_4986_and_560_arms_refuse_when_already_done() {
        assert_eq!(resolve_shimmer(&ctx(), 4986, 1, 0), ShimmerOutcome::SlimeRainbowUnlock);
        assert_eq!(
            resolve_shimmer(
                &ShimmerContext {
                    unlocked_slime_rainbow: true,
                    ..ctx()
                },
                4986,
                1,
                0
            ),
            ShimmerOutcome::Refused
        );
        assert_eq!(resolve_shimmer(&ctx(), 560, 1, 0), ShimmerOutcome::SlimeRainStart);
        assert_eq!(
            resolve_shimmer(
                &ShimmerContext {
                    slime_rain: true,
                    ..ctx()
                },
                560,
                1,
                0
            ),
            ShimmerOutcome::Refused
        );
    }

    #[test]
    fn the_release_uses_the_transform_override_and_the_attempt_budget() {
        // `NPCID.cs:10470`: the Eye of Cthulhu (3) transforms the release to 21.
        assert_eq!(
            resolve_shimmer(&ctx(), 2002, 60, 3),
            ShimmerOutcome::ReleaseNpc {
                npc_type: 21,
                attempts: 50 // ≤ 50 attempts
            }
        );
        // An unmapped makeNPC spawns itself, capped by the stack.
        assert_eq!(
            resolve_shimmer(&ctx(), 2002, 4, 9999),
            ShimmerOutcome::ReleaseNpc {
                npc_type: 9999,
                attempts: 4
            }
        );
    }

    #[test]
    fn the_decraft_resolves_the_recipe_and_the_amount() {
        // Recipe 62 is Silk Rope x30 from Cobwebs (`recipes_gen.rs`) — the type
        // is in no transform set entry, so the decraft is the arm.
        assert_eq!(decrafting_recipe_index(false, 3077), 62);
        assert_eq!(
            resolve_shimmer(&ctx(), 3077, 60, 0),
            ShimmerOutcome::Decraft {
                recipe_index: 62,
                amount: 2 // 60 / 30
            }
        );
        // Below one result unit the C# `GetShimmered` STILL enters the decraft
        // arm with `FindDecraftAmount() == 0` (`Item.cs:49473` — no guard);
        // the ingredient walk spawns nothing and `stack -= 0` keeps the drop.
        // (`CanShimmer` refuses such a drop earlier, so this only matters for
        // a direct `GetShimmered` call — the resolve mirrors the C# branch.)
        assert_eq!(
            resolve_shimmer(&ctx(), 3077, 29, 0),
            ShimmerOutcome::Decraft {
                recipe_index: 62,
                amount: 0
            }
        );
        // A torch (8) is in the transform set (8 → 5353), so the transform arm
        // runs first — the chain order, not the decraft.
        assert_eq!(
            resolve_shimmer(&ctx(), 8, 30, 0),
            ShimmerOutcome::Transform { new_type: 5353 }
        );
    }

    #[test]
    fn the_decraft_lock_reads_the_skeletron_golem_ingredients() {
        // A Soul-of-Night (154) recipe is post-Skeletron; with Skeletron down
        // (the default) it is available.
        set_decraft_flags(false, true);
        let locked = craft::RECIPES
            .iter()
            .enumerate()
            .find(|(_, r)| r.required.iter().any(|req| req.item == 154));
        let Some((idx, _)) = locked else {
            set_decraft_flags(true, true);
            return; // no 154 recipe in the port's data — nothing to lock
        };
        assert!(is_recipe_index_decraft_locked(idx));
        set_decraft_flags(true, true);
        assert!(!is_recipe_index_decraft_locked(idx));
    }

    fn ings_len(index: usize) -> usize {
        decraft_ingredients(index).len()
    }

    fn first_unit(index: usize) -> u32 {
        decraft_ingredients(index)[0].units
    }

    #[test]
    fn group_ingredients_spend_the_dec_item() {
        // The WOOD group (25): its dec item is the cheapest member.
        let wood = group_decraft_item(craft::RecipeGroups::WOOD).expect("wood group");
        assert!(craft::group(craft::RecipeGroups::WOOD).unwrap().members.contains(&wood));
        // Recipe 62 (Silk Rope x30 from item 225 x1) decrafts to that id with the
    // authored stack — and the WOOD-group substitution: recipe 0 (Torch x3)
    // spends the group's dec id, not the authored `9` directly.
        assert_eq!((ings_len(62), first_unit(62)), (1, 1));
        let torch = decraft_ingredients(0);
        assert!(torch.iter().any(|i| i.item_type == 23), "the gel entry");
        let wood = group_decraft_item(craft::RecipeGroups::WOOD).expect("wood group");
        assert!(
            torch.iter().any(|i| i.item_type == wood),
            "the wood group's dec item {wood}"
        );
    }

    #[test]
    fn can_shimmer_gates_on_the_transform_and_the_locks() {
        // Post-Moonlord locked.
        assert!(!can_shimmer(&ctx(), 1326, 1, 0));
        assert!(can_shimmer(
            &ShimmerContext {
                downed_moonlord: true,
                ..ctx()
            },
            1326,
            1,
            0
        ));
        // A coin always can.
        assert!(can_shimmer(&ctx(), 71, 1, 0));
        // A decraftable stack can; below one unit it cannot.
        assert!(can_shimmer(&ctx(), 3077, 60, 0));
        assert!(!can_shimmer(&ctx(), 3077, 29, 0));
        // A plain unmapped item only via makeNPC.
        assert!(!can_shimmer(&ctx(), 2, 1, 0));
        assert!(can_shimmer(&ctx(), 2, 1, 42));
        // 560 with the rain already active cannot.
        assert!(!can_shimmer(
            &ShimmerContext {
                slime_rain: true,
                ..ctx()
            },
            560,
            1,
            0
        ));
    }

    #[test]
    fn resolve_is_order_deterministic() {
        let a = resolve_shimmer(&ctx(), 8, 30, 0);
        let b = resolve_shimmer(&ctx(), 8, 30, 0);
        assert_eq!(a, b);
    }

    // --- W4-e19: the f47-second-half census fixes -------------------------

    #[test]
    fn w4_e19_custom_shimmer_results_override_the_required_list() {
        // `Recipe.cs:2193-2194` — Silver Bullet (278, recipe 266) refunds
        // 70 Musket Balls + 1 Silver Ore (14), NOT the 70 balls + 1 Silver
        // Bar (21) the craft spends. The override replaces the whole list.
        let idx = decrafting_recipe_index(false, 278) as usize;
        let ings = decraft_ingredients(idx);
        assert_eq!(ings.len(), 2);
        assert_eq!((ings[0].item_type, ings[0].units), (97, 70));
        assert_eq!((ings[1].item_type, ings[1].units), (14, 1));
        // `:9664-9665` — Copper Pickaxe (3509) refunds 1 Copper Bar (12) +
        // 1 Wood (9), NOT the 8 bars / 4 wood the craft spends (the group
        // substitution is bypassed — the C# `Select` reads `item.type`).
        let idx = decrafting_recipe_index(false, 3509) as usize;
        let ings = decraft_ingredients(idx);
        assert_eq!(ings.len(), 2);
        assert_eq!((ings[0].item_type, ings[0].units), (12, 1));
        assert_eq!((ings[1].item_type, ings[1].units), (9, 1));
        // `:9694` — Copper Shortsword (3507) refunds 1 Copper Bar only.
        let idx = decrafting_recipe_index(false, 3507) as usize;
        let ings = decraft_ingredients(idx);
        assert_eq!(ings.len(), 1);
        assert_eq!((ings[0].item_type, ings[0].units), (12, 1));
        // A recipe with NO override keeps the plain required walk (Silk
        // Rope 3077 → 1 Cobweb 225).
        let idx = decrafting_recipe_index(false, 3077) as usize;
        let ings = decraft_ingredients(idx);
        assert_eq!(ings.len(), 1);
        assert_eq!((ings[0].item_type, ings[0].units), (225, 1));
    }

    #[test]
    fn w4_e19_the_transform_resolves_on_the_equivalent_type() {
        // `WorldItem.cs:1695` resolves `GetTransformToItem` on the EQUIVALENT
        // type, and `ShimmerTransforms.cs:95-108` reads
        // `ContentSamples.ItemsByType[type]` (the equivalent) for the
        // trophy/lunar arms. The port now reads `registry::place_tile/
        // place_style(equivalent)` to match. The six `ShimmerCountsAsItem`
        // pairs (`ItemID.cs:354`) all map non-placeable items to non-placeable
        // targets, so no current row exercises the trophy/lunar arm through a
        // remap — this pins the mapping + the gate agreement (the structural
        // fix is verified by the cite; a future placeable remap inherits it).
        for (raw, eq) in [(5358u16, 5437u16), (5361, 5437), (5360, 5437), (5359, 5437), (5455, 5323), (6195, 6190)] {
            assert_eq!(shimmer::shimmer_equivalent_type(raw, false), eq);
            // Neither the raw nor the equivalent is in the transform set, and
            // neither places a tile — both resolve Nothing, gate refuses.
            assert_eq!(resolve_shimmer(&ctx(), raw, 1, 0), ShimmerOutcome::Nothing);
            assert!(!can_shimmer(&ctx(), raw, 1, 0));
        }
        // The lunar-brick arm IS reachable directly (3461, no remap): the
        // moon-phase table drives it (`ShimmerTransforms.cs:106-108`).
        assert_eq!(
            resolve_shimmer(&ShimmerContext { moon_phase: 4, ..ctx() }, 3461, 1, 0),
            ShimmerOutcome::Transform { new_type: 5408 }
        );
    }

    #[test]
    fn w4_e19_the_release_budget_counts_free_slots() {
        // `NPC.GetAvailableAmountOfNPCsToSpawnUpToSlot` (`NPC.cs:80635-80654`):
        // 200 slots, the free count capped at the wanted amount.
        use crate::world_sim::npcs::npc_spawn_budget;
        assert_eq!(npc_spawn_budget(0, 60), 60); // 200 free, want 60
        assert_eq!(npc_spawn_budget(190, 60), 10); // 10 free, want 60
        assert_eq!(npc_spawn_budget(200, 60), 0); // full
        assert_eq!(npc_spawn_budget(250, 60), 0); // over-full saturates
        assert_eq!(npc_spawn_budget(0, 0), 0); // want 0 → 0 (no draw)
    }

    #[test]
    fn w4_e19_the_decraft_arm_runs_at_zero_amount() {
        // `WorldItem.cs:1788` — the branch is `decraftingRecipeIndex >= 0`,
        // NOT `amount > 0`. A 29-stack of Silk Rope (recipe 62, create 30)
        // resolves Decraft{amount: 0}: the walk spawns nothing, the source
        // keeps its 29. (`CanShimmer` refuses it earlier — this pins the
        // resolve's branch shape against a direct GetShimmered call.)
        assert_eq!(
            resolve_shimmer(&ctx(), 3077, 29, 0),
            ShimmerOutcome::Decraft { recipe_index: 62, amount: 0 }
        );
    }

    /// W4-e19 — the crimson/corruption decraft variant
    /// (`ShimmerTransforms.cs:22-29`, the `Recipe.cs:15249-15256` tags):
    /// Monster Lasagna (5092) has a corruption row (90, Rotten Chunk 68 ×8)
    /// and a crimson row (91, Vertebrae 1330 ×8). The PERSISTED world-evil
    /// flag (`World.crimson`, the g69 field + `.trwld` v18 tail) picks the
    /// variant; the ingredients follow. The old thread-local hook is gone —
    /// the preference reads the world through [`ShimmerContext`].
    #[test]
    fn w4_e19_the_decraft_variant_follows_the_world_evil() {
        // The corruption world (the default): row 90, the Rotten Chunk.
        let idx = decrafting_recipe_index(false, 5092);
        assert_eq!(idx, 90, "the corruption variant");
        let ings = decraft_ingredients(idx as usize);
        assert_eq!(ings.len(), 1);
        assert_eq!((ings[0].item_type, ings[0].units), (68, 8));
        // The crimson world: row 91, the Vertebrae.
        let idx = decrafting_recipe_index(true, 5092);
        assert_eq!(idx, 91, "the crimson variant");
        let ings = decraft_ingredients(idx as usize);
        assert_eq!((ings[0].item_type, ings[0].units), (1330, 8));
        // The resolve rides the same preference (the Decraft arm's index).
        assert_eq!(
            resolve_shimmer(&ShimmerContext { world_crimson: true, ..ctx() }, 5092, 1, 0),
            ShimmerOutcome::Decraft { recipe_index: 91, amount: 1 }
        );
        assert_eq!(
            resolve_shimmer(&ctx(), 5092, 1, 0),
            ShimmerOutcome::Decraft { recipe_index: 90, amount: 1 }
        );
        // A type with NO evil variant keeps the plain index in both worlds
        // (Silk Rope 3077, recipe 62).
        assert_eq!(decrafting_recipe_index(false, 3077), 62);
        assert_eq!(decrafting_recipe_index(true, 3077), 62);
    }

    /// W4-e19 CONSUME witness — the flag the decraft reads is the PERSISTED
    /// `World.crimson`: a crimson world survives the `.trwld` v18 round trip
    /// (g69's tail byte) and the reloaded world's decraft flips to row 91;
    /// a pre-v18 file loads the corruption default (`WorldFile.cs:3507`'s
    /// fallback) and decrafts row 90 even when the payload world was crimson.
    #[test]
    fn w4_e19_the_crimson_world_decraft_reads_the_persisted_flag() {
        let mut w = tr_core::world::World::new("Test".into(), 64, 48);
        w.crimson = true;
        let mut buf =
            tr_core::save::to_bytes(&w, &tr_core::save::WorldSide::default()).unwrap();
        let (back, _) = tr_core::save::parse(&buf).unwrap();
        assert!(back.crimson, "the crimson world survives the reload");
        let reloaded = ShimmerContext { world_crimson: back.crimson, ..ctx() };
        assert_eq!(
            resolve_shimmer(&reloaded, 5092, 1, 0),
            ShimmerOutcome::Decraft { recipe_index: 91, amount: 1 }
        );
        // The pre-v18 truncation: the version patched to 17, the crimson
        // tail byte gone — the corruption default decrafts row 90.
        buf.truncate(buf.len() - 1);
        buf[8..12].copy_from_slice(&17u32.to_le_bytes());
        let (old, _) = tr_core::save::parse(&buf).unwrap();
        assert!(!old.crimson, "a pre-v18 file keeps the corruption default");
        let legacy = ShimmerContext { world_crimson: old.crimson, ..ctx() };
        assert_eq!(
            resolve_shimmer(&legacy, 5092, 1, 0),
            ShimmerOutcome::Decraft { recipe_index: 90, amount: 1 }
        );
    }
}
