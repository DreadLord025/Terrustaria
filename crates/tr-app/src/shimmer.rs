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
    /// `ItemID.Sets.CommonCoin` (`:1696-1712`): the stack merges by the tier
    /// (silver ×100, gold ×10000, platinum ×1000000 after flooring the stack to
    /// 1) and `AddCoinLuck` spends the merged stack.
    CoinMerge {
        /// The tier multiplier (100 / 10000 / 1000000).
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
    let create_tile = registry::place_tile(item_type).map_or(-1, i32::from) as i16;
    let place_style = registry::place_style(item_type);
    let transform = shimmer::transform_to_item(equivalent, create_tile, place_style, ctx.moon_phase);
    if (equivalent != 4986 || ctx.unlocked_slime_rainbow)
        && transform <= 0
        && !is_item_decraftable_and_unlocked(item_type, stack)
        && !shimmer::is_common_coin(equivalent)
    {
        return make_npc > 0;
    }
    true
}

/// `ShimmerTransforms.IsItemDec craftableAndIsDecraftUnlocked`
/// (`ShimmerTransforms.cs:64-80`): the recipe index exists, is not locked, and
/// the stack covers at least one result.
pub fn is_item_decraftable_and_unlocked(item_type: u16, stack: u32) -> bool {
    let index = decrafting_recipe_index(shimmer::shimmer_equivalent_type(item_type, true));
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
/// the plain `IsCrafted` index (the port's recipes carry no crimson/corruption
/// tags, so the variant preference is a documented deviation), -1 when the type
/// has no decrafting recipe.
pub fn decrafting_recipe_index(item_type: u16) -> i32 {
    // `Recipe.UpdateWhichItemsAreCrafted` (`Recipe.cs:15241-15244`): the LAST
    // non-`notDecraftable` recipe creating the type wins.
    let mut found = -1;
    for (i, r) in craft::RECIPES.iter().enumerate() {
        if !r.not_decraftable && r.create.0 == item_type {
            found = i as i32;
        }
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
    // 2. Transform to item (`:1713-1718`).
    let create_tile = registry::place_tile(item_type).map_or(-1, i32::from) as i16;
    let place_style = registry::place_style(item_type);
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
    // 6. The decraft (`:1773-1828`).
    let index = decrafting_recipe_index(shimmer::shimmer_equivalent_type(item_type, true));
    if index >= 0 {
        let index = index as usize;
        let amount = stack / craft::RECIPES[index].create.1;
        if amount > 0 {
            return ShimmerOutcome::Decraft {
                recipe_index: index,
                amount,
            };
        }
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

/// The decraft's ingredient list for a recipe index (`WorldItem.cs:1810-1823`'s
/// `enumerable`), with `units` pre-discount. `customShimmerResults` are absent
/// from the port's recipe data (a documented deviation) — the plain
/// `requiredItem` list is used.
pub fn decraft_ingredients(recipe_index: usize) -> Vec<DecraftIngredient> {
    let Some(recipe) = craft::RECIPES.get(recipe_index) else {
        return Vec::new();
    };
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

/// The app-side apply of [`ShimmerOutcome`] — `WorldItem.GetShimmered`'s
/// chain tail (`WorldItem.cs:1691-1849`): the coin merge's luck, the transform
/// replace, the two special events, the release spawn and the decraft's
/// ingredient spawns, then the shared tail (`:1830-1848`).
///
/// `indices` are the drops whose `shimmer_time` crossed 0.9 this step. The
/// spawns ride the same 60 Hz step (the C#'s `SyncItem`/net arm is MP-only, so
/// it is out here); the alchemy discount draws the port's `item_rng` (the C#'s
/// `Main.rand`) exactly like the pot-loot rolls do.
pub(crate) fn apply_due_transforms(app: &mut App, indices: &[u32]) {
    // The decraft lock checks read the world's downed flags through the
    // thread-local (the pure helpers stay world-free).
    set_decraft_flags(app.world.downed_skeletron, app.world.downed_golem);
    let ctx = ShimmerContext {
        slime_rain: app.world.slime_rain,
        unlocked_slime_rainbow: app.world.unlocked_slime_rainbow_spawn,
        downed_moonlord: false, // no Moon Lord flag in the port's save yet
        downed_skeletron: app.world.downed_skeletron,
        downed_golem: app.world.downed_golem,
        moon_phase: app.world.moon_phase,
    };
    for &idx in indices {
        let Some(drop) = app.items.get(idx as usize).cloned() else {
            continue;
        };
        // The C#'s `Item.makeNPC` — the port's generated item data does not
        // carry the field yet, so the release arm only fires for the kinds the
        // item step maps (none today; the map + accounting are ported and unit
        // tested — the roster gap is an ANALYSIS note).
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
                // `WorldItem.cs:1696-1712` — the platinum arm floors the stack
                // to 1 before the multiply.
                let stack = if multiplier == 1_000_000 {
                    drop_mut.count.min(1)
                } else {
                    drop_mut.count
                };
                drop_mut.count = stack.saturating_mul(multiplier);
                // `AddCoinLuck` (`Player.cs:18090-18101`): add the merged stack
                // within 1000 px of the player, capped at 1,000,000.
                if let Some(p) = app.player.as_mut() {
                    let pc = Vec2::new(p.center().0, p.center().1);
                    // `Player.cs:18092`'s `!this.dead && distance < 1000`.
                    if !p.dead && (drop_mut.center() - pc).length() < 1000.0 {
                        p.luck_state.coin_luck =
                            (p.luck_state.coin_luck + drop_mut.count as f32).min(1_000_000.0);
                        p.luck_state.recalculate_luck();
                    }
                }
                air_out(app, idx);
            }
            ShimmerOutcome::Transform { new_type } => {
                // `:1713-1718` — SetDefaults + the stack is kept + shimmered.
                drop_mut.kind = new_type;
                drop_mut.shimmered = true;
                drop_mut.shimmer_time = 1.0;
                drop_mut.shimmer_wet = true;
                drop_mut.vel *= 0.1;
            }
            ShimmerOutcome::SlimeRainbowUnlock => {
                // `:1720-1735` — the flag, the NPC 681 spawn, the stack--. The
                // spawn takes `app` wholesale, so the drop borrow ends first.
                let c = drop_mut.center();
                app.world.unlocked_slime_rainbow_spawn = true;
                crate::world_sim::npcs::spawn_shimmer_release(app, 681, c.x + 4.0, c.y);
                let Some(drop_mut) = app.items.get_mut(idx as usize) else {
                    continue;
                };
                drop_mut.count = drop_mut.count.saturating_sub(1);
                if drop_mut.count == 0 {
                    air_out(app, idx);
                } else {
                    drop_mut.shimmered = true;
                }
            }
            ShimmerOutcome::SlimeRainStart => {
                // `:1736-1746` — `Main.StartSlimeRain()` (the SP branch's
                // duration), the stack-- or `shimmered = true`.
                let mut rng_state = app.item_rng;
                crate::world_sim::weather::start_slime_rain(&mut app.world, &mut rng_state);
                app.item_rng = rng_state;
                drop_mut.count = drop_mut.count.saturating_sub(1);
                if drop_mut.count == 0 {
                    air_out(app, idx);
                } else {
                    drop_mut.shimmered = true;
                }
            }
            ShimmerOutcome::ReleaseNpc { npc_type, attempts } => {
                // `:1747-1772` — one release per attempt, `shimmerTransparency
                // = 1f` on the spawned NPC.
                for _ in 0..attempts {
                    let (cx, by) = {
                        let d = &app.items[idx as usize];
                        (d.center().x, d.pos.y + SIZE_PX)
                    };
                    let spawned =
                        crate::world_sim::npcs::spawn_shimmer_release(app, npc_type, cx, by);
                    if app.items[idx as usize].count == 0 {
                        break;
                    }
                    if spawned {
                        app.items[idx as usize].count -= 1;
                    }
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
    }
    // The air-out sweep (`WorldItem.TurnToAir`'s list effect — the C# marks the
    // item inactive and the item array's compaction drops it): the zeroed
    // entries leave once every arm of this step has run, so the collected
    // indices stay valid during the applies.
    app.items.retain(|d| !(d.kind == 0 && d.count == 0));
}

/// The 14x14 item box in pixels (`ItemDrop::SIZE`).
const SIZE_PX: f32 = crate::item::SIZE;

/// The air-out (`WorldItem.TurnToAir`'s effect on the drop list, `:1845-1848`).
fn air_out(app: &mut App, idx: u32) {
    if let Some(drop) = app.items.get_mut(idx as usize) {
        drop.kind = 0;
        drop.count = 0;
    }
}

/// `WorldItem.GetShimmered`'s decraft arm (`:1773-1828`): per ingredient
/// `amount * entry.stack` units, the alchemy discount (1/3 of the units dropped,
/// one `Next(3)` per unit), `maxStack` chunking and the final
/// `stack -= amount * createItem.stack`.
fn apply_decraft(app: &mut App, idx: u32, recipe_index: usize, amount: u32) {
    let pos = {
        let drop = &app.items[idx as usize];
        drop.pos
    };
    let ingredients = decraft_ingredients(recipe_index);
    let alchemy = craft::RECIPES.get(recipe_index).is_some_and(|r| r.alchemy);
    // The drop's own position spawns the ingredients (the C#'
    // `SpawnShimmeredItem(index, type, stack)` at the item's box).
    let mut spawns: Vec<(u16, u32)> = Vec::new();
    for ing in ingredients {
        let mut units = ing.units * amount;
        if alchemy {
            // `:1794-1800` — one `Main.rand.Next(3) == 0` per unit.
            for _ in 0..ing.units * amount {
                let mut state = app.item_rng;
                let r = crate::app::rand01(&mut state);
                app.item_rng = state;
                if (r * 3.0) as i32 == 0 {
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
    for (kind, count) in spawns {
        crate::world_sim::items::spawn_item_drop_px(app, kind, count, pos);
    }
    let create_stack = craft::RECIPES.get(recipe_index).map_or(1, |r| r.create.1);
    if let Some(drop) = app.items.get_mut(idx as usize) {
        drop.count = drop.count.saturating_sub(amount * create_stack);
        if drop.count == 0 {
            air_out(app, idx);
        } else {
            drop.shimmered = true;
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
        assert_eq!(decrafting_recipe_index(3077), 62);
        assert_eq!(
            resolve_shimmer(&ctx(), 3077, 60, 0),
            ShimmerOutcome::Decraft {
                recipe_index: 62,
                amount: 2 // 60 / 30
            }
        );
        // Below one result unit → no decraft.
        assert_eq!(resolve_shimmer(&ctx(), 3077, 29, 0), ShimmerOutcome::Nothing);
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
}
