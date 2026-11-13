//! The crafting recipe table + the availability machine (M3-CRAFT-2):
//! `Recipe` / `RecipeGroup` / `PlayerMeetsEnvironmentConditions` /
//! `CollectItemsToCraftWithFrom` ported from `Recipe.cs` over the items the
//! registry actually has. The full ~15k-line `SetupRecipes` extraction is a
//! separate task (`tools/GenRecipes` candidate).
//!
//! Item ids reference [`crate::registry`]; station ids are `TileID` values the
//! player must stand near (`Player.AdjTiles`, `Player.cs:35661`).
//!
//! Port notes: the C# runtime folds group members into a fake item id
//! (`1000000 + groupIndex`, `Recipe.cs:409-416`) — the port's static tables
//! carry the accepted group on the [`Requirement`] directly, which is the
//! same lookup result `CreateRequiredItemQuickLookups` (`Recipe.cs:15162-15191`)
//! produces. The chest store ([`WorldChest`]) is in-memory only — the
//! chest inventories' save/persistence is M6-SAVE-2's and the chest UI is
//! M3-CHEST-UI's; banks (piggy/safe/void bag) are M3-STORE-1's, so the
//! `CollectItemsFromChests` subset here is the tile chests in range
//! (`NearbyChests.GetChestsInRangeOf`).

use std::collections::HashMap;

// The chest entity moved to `chest.rs` (M3-CHEST-UI phase 1); the re-export
// keeps the existing `craft::WorldChest` paths valid — the `world_sim/
// editing.rs` stocked-chest test constructs the type through this path.
#[cfg(test)]
use crate::chest::DEFAULT_MAX_ITEMS;
pub(crate) use crate::chest::WorldChest;
use crate::item::{INV, ItemSlot};
use crate::player::{self, Player};
use crate::registry;
use tr_core::World;
use tr_core::unified_random::UnifiedRandom;

/// `Recipe.maxRequirements` (`Recipe.cs:16775`).
pub(crate) const MAX_REQUIREMENTS: usize = 15;

/// One crafting recipe: `createItem` + `requiredItem[]` + `requiredTile` +
/// the environment flags (`Recipe.cs:72-100`).
#[derive(Default)]
pub(crate) struct Recipe {
    /// `(item id, stack)` — `createItem`.
    pub(crate) create: (u16, u32),
    /// `requiredItem[]` — see [`Requirement`].
    pub(crate) required: &'static [Requirement],
    /// Station tile ids the player must stand near (`requiredTile`,
    /// `SetCraftingStation`); empty = craftable by hand.
    pub(crate) stations: &'static [u16],
    /// `needWater` (`:82`): `player.adjWaterSource`.
    pub(crate) need_water: bool,
    /// `needHoney` (`:80`): `player.adjHoney`.
    pub(crate) need_honey: bool,
    /// `needLava` (`:84`): `player.adjLava`.
    pub(crate) need_lava: bool,
    /// `needSnowBiome` (`:90`): `player.ZoneSnow`.
    pub(crate) need_snow: bool,
    /// `needGraveyardBiome` (`:92`): `player.ZoneGraveyard`.
    pub(crate) need_graveyard: bool,
    /// `alchemy` (`:88`) — auto-set by `AddRecipe` iff `requiredTile == 13`
    /// (`:16722-16726`); gates the 1/3 ingredient discount at craft time
    /// (`GetIngredientCraftingDiscount`, `:214-228`).
    pub(crate) alchemy: bool,
    /// `notDecraftable` — the derived reverse platform/wall rows carry it
    /// (`Recipe.cs:16665`/:16689); the craft direction ignores it.
    pub(crate) not_decraftable: bool,
}

/// The zero recipe the static entries inherit the unset fields from.
const BASE: Recipe = Recipe {
    create: (0, 0),
    required: &[],
    stations: &[],
    need_water: false,
    need_honey: false,
    need_lava: false,
    need_snow: false,
    need_graveyard: false,
    alchemy: false,
    not_decraftable: false,
};

/// One `requiredItem` entry: the authored item id + stack, with the accepted
/// recipe group (`RequireGroup`, `Recipe.cs:143-153`) or [`NO_GROUP`]. At
/// lookup time an entry whose item belongs to the group's members substitutes
/// the whole member set (`CreateRequiredItemQuickLookups`,
/// `Recipe.cs:15162-15191`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Requirement {
    /// The authored `requiredItem[].type`.
    pub(crate) item: u16,
    /// `requiredItem[].stack`.
    pub(crate) stack: u32,
    /// The accepted group id, or [`NO_GROUP`].
    pub(crate) group: u16,
}

/// No accepted group (a plain item requirement).
pub(crate) const NO_GROUP: u16 = u16::MAX;

/// The `ID/RecipeGroups.cs` field ids (the `RegisteredId` = registration
/// order, `Recipe.cs:497-533`).
pub(crate) struct RecipeGroups;
impl RecipeGroups {
    pub(crate) const BIRDS: u16 = 0;
    pub(crate) const SCORPIONS: u16 = 1;
    pub(crate) const SQUIRRELS: u16 = 2;
    pub(crate) const BUGS: u16 = 3;
    pub(crate) const DUCKS: u16 = 4;
    pub(crate) const BUTTERFLIES: u16 = 5;
    pub(crate) const FIREFLIES: u16 = 6;
    pub(crate) const SNAILS: u16 = 7;
    pub(crate) const DRAGONFLIES: u16 = 8;
    pub(crate) const TURTLES: u16 = 9;
    pub(crate) const MACAWS: u16 = 10;
    pub(crate) const COCKATIELS: u16 = 11;
    pub(crate) const CLOUD_BALLOONS: u16 = 12;
    pub(crate) const BLIZZARD_BALLOONS: u16 = 13;
    pub(crate) const SANDSTORM_BALLOONS: u16 = 14;
    pub(crate) const CRITTER_GUIDES: u16 = 15;
    pub(crate) const NATURE_GUIDES: u16 = 16;
    pub(crate) const SEASHELLS: u16 = 17;
    pub(crate) const FRUIT: u16 = 18;
    pub(crate) const BALLOONS: u16 = 19;
    pub(crate) const COBALT_BAR: u16 = 20;
    pub(crate) const MYTHRIL_BAR: u16 = 21;
    pub(crate) const ADAMANTITE_BAR: u16 = 22;
    pub(crate) const GEM_CRITTER: u16 = 23;
    pub(crate) const MAGIC_MIRROR: u16 = 24;
    pub(crate) const WOOD: u16 = 25;
    pub(crate) const STONE: u16 = 26;
    pub(crate) const SAND: u16 = 27;
    pub(crate) const IRON_BAR: u16 = 28;
    pub(crate) const SILVER_BAR: u16 = 29;
    pub(crate) const GOLD_BAR: u16 = 30;
    pub(crate) const FRAGMENT: u16 = 31;
    pub(crate) const PRESSURE_PLATE: u16 = 32;
    pub(crate) const JELLYFISH: u16 = 33;
}

/// `RecipeGroup` (`RecipeGroup.cs:9-27`) — `SetupRecipeGroups`
/// (`Recipe.cs:497-533`) verbatim; `id` is the registration order (the C#
/// `RegisteredId`), `icon` is `Items[0]` (`GetPlaceholderItemType`),
/// `members` is `ValidItems`/`Items`.
pub(crate) struct RecipeGroupDef {
    pub(crate) id: u16,
    pub(crate) name: &'static str,
    pub(crate) icon: u16,
    pub(crate) members: &'static [u16],
}

/// All 34 registered groups, verbatim (`Recipe.cs:499-533`). The registration
/// order is the id; member lists quote the C# call sites.
pub(crate) static GROUPS: &[RecipeGroupDef] = &[
    RecipeGroupDef {
        id: 0,
        name: "Birds",
        icon: 2015,
        members: &[2015, 2016, 2017],
    },
    RecipeGroupDef {
        id: 1,
        name: "Scorpions",
        icon: 2157,
        members: &[2157, 2156],
    },
    RecipeGroupDef {
        id: 2,
        name: "Squirrels",
        icon: 2018,
        members: &[2018, 3563],
    },
    RecipeGroupDef {
        id: 3,
        name: "Bugs",
        icon: 3194,
        members: &[3194, 3192, 3193],
    },
    RecipeGroupDef {
        id: 4,
        name: "Ducks",
        icon: 2123,
        members: &[2123, 2122],
    },
    RecipeGroupDef {
        id: 5,
        name: "Butterflies",
        icon: 1998,
        members: &[1998, 2001, 1994, 1995, 1996, 1999, 1997, 2000],
    },
    RecipeGroupDef {
        id: 6,
        name: "Fireflies",
        icon: 1992,
        members: &[1992, 2004],
    },
    RecipeGroupDef {
        id: 7,
        name: "Snails",
        icon: 2006,
        members: &[2006, 2007],
    },
    RecipeGroupDef {
        id: 8,
        name: "Dragonflies",
        icon: 4334,
        members: &[4334, 4335, 4336, 4338, 4339, 4337],
    },
    RecipeGroupDef {
        id: 9,
        name: "Turtles",
        icon: 4464,
        members: &[4464, 4465],
    },
    RecipeGroupDef {
        id: 10,
        name: "Macaws",
        icon: 5212,
        members: &[5212, 5300],
    },
    RecipeGroupDef {
        id: 11,
        name: "Cockatiels",
        icon: 5312,
        members: &[5312, 5313],
    },
    RecipeGroupDef {
        id: 12,
        name: "CloudBalloons",
        icon: 399,
        members: &[399, 1250],
    },
    RecipeGroupDef {
        id: 13,
        name: "BlizzardBalloons",
        icon: 1163,
        members: &[1163, 1251],
    },
    RecipeGroupDef {
        id: 14,
        name: "SandstormBalloons",
        icon: 983,
        members: &[983, 1252],
    },
    RecipeGroupDef {
        id: 15,
        name: "CritterGuides",
        icon: 4767,
        members: &[4767, 5453],
    },
    RecipeGroupDef {
        id: 16,
        name: "NatureGuides",
        icon: 5309,
        members: &[5309, 5454],
    },
    RecipeGroupDef {
        id: 17,
        name: "Seashells",
        icon: 2625,
        members: &[2625, 2626],
    },
    RecipeGroupDef {
        id: 18,
        name: "Fruit",
        icon: 4009,
        members: &[
            4009, 4282, 4283, 4284, 4285, 4286, 4287, 4288, 4289, 4290, 4291, 4292, 4293, 4294,
            4295, 4296, 4297, 5277, 5278,
        ],
    },
    RecipeGroupDef {
        id: 19,
        name: "Balloons",
        icon: 3738,
        members: &[3738, 3736, 3737],
    },
    RecipeGroupDef {
        id: 20,
        name: "CobaltBar",
        icon: 381,
        members: &[381, 1184],
    },
    RecipeGroupDef {
        id: 21,
        name: "MythrilBar",
        icon: 382,
        members: &[382, 1191],
    },
    RecipeGroupDef {
        id: 22,
        name: "AdamantiteBar",
        icon: 391,
        members: &[391, 1198],
    },
    RecipeGroupDef {
        id: 23,
        name: "GemCritter",
        icon: 4838,
        members: &[
            4838, 4844, 4843, 4841, 4842, 4840, 4839, 4831, 4837, 4836, 4834, 4835, 4833, 4832,
        ],
    },
    RecipeGroupDef {
        id: 24,
        name: "MagicMirror",
        icon: 50,
        members: &[50, 3199],
    },
    RecipeGroupDef {
        id: RecipeGroups::WOOD,
        name: "Wood",
        icon: 9,
        members: &[9, 619, 620, 621, 911, 1729, 2504, 2503, 5215],
    },
    RecipeGroupDef {
        id: RecipeGroups::STONE,
        name: "Stone",
        icon: 3,
        members: &[3, 61, 836, 409],
    },
    RecipeGroupDef {
        id: RecipeGroups::SAND,
        name: "Sand",
        icon: 169,
        members: &[169, 408, 1246, 370, 3272, 3338, 3274, 3275],
    },
    RecipeGroupDef {
        id: RecipeGroups::IRON_BAR,
        name: "IronBar",
        icon: 22,
        members: &[22, 704],
    },
    RecipeGroupDef {
        id: RecipeGroups::SILVER_BAR,
        name: "SilverBar",
        icon: 21,
        members: &[21, 705],
    },
    RecipeGroupDef {
        id: RecipeGroups::GOLD_BAR,
        name: "GoldBar",
        icon: 19,
        members: &[19, 706],
    },
    RecipeGroupDef {
        id: 31,
        name: "Fragment",
        icon: 3458,
        members: &[3458, 3456, 3457, 3459],
    },
    RecipeGroupDef {
        id: 32,
        name: "PressurePlate",
        icon: 542,
        members: &[542, 852, 543, 541, 1151, 529, 853, 4261],
    },
    RecipeGroupDef {
        id: 33,
        name: "Jellyfish",
        icon: 2436,
        members: &[2436, 2437, 2438],
    },
];

/// `RecipeGroup.recipeGroups[id]` (`RecipeGroup.cs:69-73`).
pub(crate) fn group(id: u16) -> Option<&'static RecipeGroupDef> {
    GROUPS.get(id as usize)
}

impl Requirement {
    /// A plain item entry.
    pub(crate) const fn item(item: u16, stack: u32) -> Self {
        Self {
            item,
            stack,
            group: NO_GROUP,
        }
    }

    /// A group entry: the authored member item + the group it may draw from.
    pub(crate) const fn group(item: u16, group: u16, stack: u32) -> Self {
        Self { item, stack, group }
    }

    /// `RequiredItemEntry.Matches` (`Recipe.cs:36-43`): the authored item, or
    /// any member of the accepted group.
    pub(crate) fn matches(&self, kind: u16) -> bool {
        if self.group != NO_GROUP {
            if let Some(g) = group(self.group) {
                return g.members.contains(&kind);
            }
        }
        self.item == kind
    }
}

/// `Recipe.TileCountsAs` (`Recipe.SetupTileInheritance`, `Recipe.cs:559-571`):
/// `(tile, countsAs)` — standing near `tile` also satisfies a `countsAs`
/// station requirement (`Player.SetAdjTile`'s recursion, `Player.cs:35643-35659`).
pub(crate) static TILE_COUNTS_AS: &[(u16, u16)] = &[
    (96, 215),
    (17, 215),
    (302, 17),
    (77, 17),
    (133, 77),
    (134, 16),
    (355, 13),
    (699, 13),
    (304, 86),
];

/// `TileID.Sets.CountsAsWaterForCrafting` (`TileID.cs:553`): the sink (172)
/// and the water fountain (207) count as water sources.
pub(crate) const COUNTS_AS_WATER: [u16; 2] = [172, 207];

/// Does standing near `kind` satisfy a `required` station? `kind` itself, or
/// its `TileCountsAs` chain reaching `required`.
fn satisfies_station(required: u16, kind: u16) -> bool {
    let mut cur = kind;
    loop {
        if cur == required {
            return true;
        }
        match TILE_COUNTS_AS.iter().find(|(a, _)| *a == cur) {
            Some(&(_, parent)) => cur = parent,
            None => return false,
        }
    }
}

/// The `Player.AdjTiles` result (`Player.cs:35661-35699`): the active tile
/// kinds in the reach region plus the liquid-source flags.
pub(crate) struct Adjacent {
    pub(crate) tiles: Vec<u16>,
    /// `player.adjWaterSource` — `tile.liquid > 200` water (or a
    /// `CountsAsWaterForCrafting` tile).
    pub(crate) water: bool,
    /// `player.adjHoney` — `tile.liquid > 200` honey.
    pub(crate) honey: bool,
    /// `player.adjLava` — `tile.liquid > 200` lava.
    pub(crate) lava: bool,
    /// `player.alchemyTable` — the alchemy-table tiles
    /// (`Player.SetAdjTile`, `Player.cs:35647-35649`).
    pub(crate) alchemy_table: bool,
}

/// `Player.AdjTiles` (`Player.cs:35661`) with `TileReachCheckSettings.Simple`
/// (`TileReachCheckSettings.cs:81-86`: reach X 5 / Y 3, `GetRanges` clamped by
/// `TileReachLimit = 20`; region math `:45-54`): the active tile kinds around
/// the player hitbox and the liquid flags (the same region — the vanilla has
/// no separate liquid radius; threshold `tile.liquid > 200`, `:35688-35693`).
/// The region is inclusive of `HX`/`HY`: `GetTileRegion` returns
/// `new Rectangle(LX, LY, HX-LX, HY-LY)` and the XNA `Rectangle.Right` is
/// `X + Width` (exclusive), so `Player.cs:35672`'s `i <= Right` scans
/// `LX..=HX` — the same inclusive loop below (S1-L-11b).
pub(crate) fn scan_adjacent(world: &World, player: &Player) -> Adjacent {
    const REACH_X: i32 = 5;
    const REACH_Y: i32 = 3;
    let lx = (player.x / 16.0).floor() as i32 - REACH_X;
    let hx = ((player.x + player::WIDTH as f32) / 16.0).ceil() as i32 - 1 + REACH_X;
    let ly = (player.y / 16.0).floor() as i32 - REACH_Y;
    let hy = ((player.y + player::HEIGHT as f32) / 16.0).ceil() as i32 - 1 + REACH_Y;
    let mut adj = Adjacent {
        tiles: Vec::new(),
        water: false,
        honey: false,
        lava: false,
        alchemy_table: false,
    };
    for i in lx.max(0)..=hx.min(world.width as i32 - 1) {
        for j in ly.max(0)..=hy.min(world.height as i32 - 1) {
            if let Some(t) = world.tile(i, j) {
                if t.active() {
                    if !adj.tiles.contains(&t.kind) {
                        adj.tiles.push(t.kind);
                    }
                    // `Player.SetAdjTile` (`Player.cs:35646-35649`).
                    if t.kind == 355 || t.kind == 699 {
                        adj.alchemy_table = true;
                    }
                }
                // `TileID.Sets.CountsAsWaterForCrafting` (`:35685-35687`).
                if t.active() && COUNTS_AS_WATER.contains(&t.kind) {
                    adj.water = true;
                }
                if t.liquid > 200 {
                    match t.liquid_kind() {
                        tr_core::tile::LiquidKind::Water => adj.water = true,
                        tr_core::tile::LiquidKind::Honey => adj.honey = true,
                        tr_core::tile::LiquidKind::Lava => adj.lava = true,
                        tr_core::tile::LiquidKind::Shimmer => {}
                    }
                }
            }
        }
    }
    adj
}

/// The `player.ZoneSnow` / `player.ZoneGraveyard` pair
/// (`Player.cs:16705/16709` copy the `SceneMetrics` zone flags).
#[derive(Clone, Copy, Default)]
pub(crate) struct Zones {
    pub(crate) snow: bool,
    pub(crate) graveyard: bool,
}

/// `Recipe.GetIngredientCraftingDiscount` (`Recipe.cs:214-228`): one free unit
/// per `Main.rand.Next(3) == 0` roll while the recipe brews at an alchemy
/// table — every unit rolls, even past a full discount. The C# draws the
/// shared `Main.rand`; the port's craft stream is the exact `UnifiedRandom`
/// generator seeded per world (`App::craft_rng`) — the port-wide shared-stream
/// interleave stays unmodelled (S1-A-11's residual; the same deterministic
/// reduction as the wiring RNG, S1-L-6c).
fn ingredient_discount(
    alchemy: bool,
    alchemy_table: bool,
    stack: u32,
    rng: &mut UnifiedRandom,
) -> u32 {
    if !alchemy || !alchemy_table {
        return 0;
    }
    (0..stack).filter(|_| rng.next_max(3) == 0).count() as u32
}

/// `NearbyChests.GetChestsInRangeOf`'s default range (`NearbyChests.cs:14`).
pub(crate) const CHEST_RANGE_PX: f32 = 600.0;

/// The world chest tiles the range scan counts (`GetChestsInRangeOf` walks
/// `Main.chest[]` — the port scans the tile kinds instead): the chest (21)
/// and the gold chest (341).
pub(crate) const CHEST_TILES: [u16; 2] = [21, 341];

/// The tile chests whose center is within [`CHEST_RANGE_PX`] of the player's
/// feet (`GetChestsInRangeOf(player.position)`, `Recipe.cs:395`), in raster
/// order (the vanilla iterates `Main.chest[]`'s instance order; the port has
/// no instance registry — the deviation is recorded in `docs/ANALYSIS.md`).
/// `Chest.IsLockedOrInUse` (`Chest.cs:178-188`) passes for every chest — the
/// port has no chest locking (`M3-CHEST-UI`'s) and no other players browsing.
pub(crate) fn chests_in_range(
    world: &World,
    player: &Player,
    chests: &[WorldChest],
    from_nearby: bool,
) -> Vec<usize> {
    let mut out = Vec::new();
    if !from_nearby {
        return out;
    }
    let px = player.x;
    let py = player.y;
    let reach = (CHEST_RANGE_PX / 16.0).floor() as i32 + 1;
    let lx = ((px / 16.0).floor() as i32 - reach).max(0);
    let hx =
        (((px + player::WIDTH as f32) / 16.0).ceil() as i32 + reach).min(world.width as i32 - 1);
    let ly = ((py / 16.0).floor() as i32 - reach).max(0);
    let hy =
        (((py + player::HEIGHT as f32) / 16.0).ceil() as i32 + reach).min(world.height as i32 - 1);
    for i in lx..=hx {
        for j in ly..=hy {
            let Some(t) = world.tile(i, j) else { continue };
            if !t.active() || !CHEST_TILES.contains(&t.kind) {
                continue;
            }
            let cx = i as f32 * 16.0 + 16.0;
            let cy = j as f32 * 16.0 + 16.0;
            let dx = cx - px;
            let dy = cy - py;
            if dx * dx + dy * dy > CHEST_RANGE_PX * CHEST_RANGE_PX {
                continue;
            }
            if chests.iter().any(|c| c.tile == (i, j)) {
                out.push(
                    chests
                        .iter()
                        .position(|c| c.tile == (i, j))
                        .expect("found above"),
                );
            }
        }
    }
    out
}

/// `Recipe.CollectItems` (`Recipe.cs:418-433`): the plain stack sums. The
/// group folding is per requirement here (`Requirement::matches` semantics) —
/// the C# folds every group into a fake id first
/// (`AddFakeCountsForItemGroups`, `Recipe.cs:409-416`); both give the same
/// per-requirement totals.
pub(crate) fn collect_owned(
    inv: &[ItemSlot; INV],
    chests: &[WorldChest],
    in_range: &[usize],
) -> HashMap<u16, u32> {
    let mut owned: HashMap<u16, u32> = HashMap::new();
    for slot in inv.iter() {
        if slot.count > 0 {
            *owned.entry(slot.kind).or_insert(0) += slot.count;
        }
    }
    for &i in in_range {
        for slot in &chests[i].slots {
            if slot.count > 0 {
                *owned.entry(slot.kind).or_insert(0) += slot.count;
            }
        }
    }
    owned
}

/// `Recipe.PlayerMeetsEnvironmentConditions` (`Recipe.cs:322-358`): the
/// stations (with the `TileCountsAs` inheritance), the liquid flags and the
/// biome zones. `needTorchGodsFavor`/`needMechdusa` have no port systems (the
/// torch-god event, the getfixedboi seed) — no subset recipe sets them.
pub(crate) fn meets_environment(recipe: &Recipe, adj: &Adjacent, zones: &Zones) -> bool {
    recipe
        .stations
        .iter()
        .all(|&t| adj.tiles.iter().any(|&k| satisfies_station(t, k)))
        && (!recipe.need_water || adj.water)
        && (!recipe.need_honey || adj.honey)
        && (!recipe.need_lava || adj.lava)
        && (!recipe.need_snow || zones.snow)
        && (!recipe.need_graveyard || zones.graveyard)
}

/// `Recipe.CollectedEnoughItemsToCraft` (`Recipe.cs:275-283`) /
/// `HowManyTimesCanRecipeBeCrafted` (`:290-311`): every requirement's folded
/// count reaches its stack.
pub(crate) fn craftable(
    recipe: &Recipe,
    owned: &HashMap<u16, u32>,
    adj: &Adjacent,
    zones: &Zones,
) -> bool {
    meets_environment(recipe, adj, zones)
        && recipe.required.iter().all(|req| {
            let have = if req.group != NO_GROUP {
                group(req.group)
                    .map(|g| {
                        g.members
                            .iter()
                            .map(|m| owned.get(m).copied().unwrap_or(0))
                            .sum()
                    })
                    .unwrap_or(0)
            } else {
                owned.get(&req.item).copied().unwrap_or(0)
            };
            have >= req.stack
        })
}

/// The panel's recipe list — `Recipe.UpdateRecipeList` (`Recipe.cs:230-259`):
/// environment × enough-ingredients. The port's older "any ingredient
/// present" partial list is gone; the classic pane shows craftable rows only.
pub(crate) fn available(
    inv: &[ItemSlot; INV],
    chests: &[WorldChest],
    in_range: &[usize],
    adj: &Adjacent,
    zones: &Zones,
) -> Vec<usize> {
    let owned = collect_owned(inv, chests, in_range);
    RECIPES
        .iter()
        .enumerate()
        .filter(|(_, r)| craftable(r, &owned, adj, zones))
        .map(|(i, _)| i)
        .collect()
}

/// Total count of one item kind across the whole inventory.
fn count_kind(inv: &[ItemSlot; INV], kind: u16) -> u32 {
    inv.iter().filter(|s| s.kind == kind).map(|s| s.count).sum()
}

/// `CraftingRequests.ConsumeItemsFrom` (`CraftingRequests.cs:244-290`): one
/// pass in slot-index order — partial decrement when `stack > toConsume`,
/// otherwise the slot is cleared entirely.
fn consume_kind(inv: &mut [ItemSlot; INV], kind: u16, to_consume: &mut u32) {
    if *to_consume == 0 {
        return;
    }
    for slot in inv.iter_mut() {
        if *to_consume == 0 {
            break;
        }
        if slot.kind == kind && slot.count > 0 {
            if slot.count > *to_consume {
                slot.count -= *to_consume;
                *to_consume = 0;
            } else {
                *to_consume -= slot.count;
                *slot = crate::item::EMPTY_SLOT;
            }
        }
    }
}

/// One drain pass over one chest's slots (`ConsumeItemsFrom`'s chest half,
/// `CraftingRequests.cs:262-290`): the same partial-stack-then-clear shape,
/// matching `RequiredItemEntry.Matches` (any group member). Cleared slots
/// become air in place — chest slots are indexed with implicit air (see
/// `chest.rs`), so the surviving stacks keep their slot positions the way the
/// C# `chest.item[i] = new Item()` leaves them — and the trailing airs fold
/// back into the implicit-air tail.
fn consume_chest(slots: &mut Vec<ItemSlot>, req: &Requirement, to_consume: &mut u32) {
    if *to_consume == 0 {
        return;
    }
    let mut i = 0;
    while i < slots.len() && *to_consume > 0 {
        let (kind, count) = (slots[i].kind, slots[i].count);
        if count > 0 && req.matches(kind) {
            if count > *to_consume {
                slots[i].count -= *to_consume;
                *to_consume = 0;
            } else {
                *to_consume -= count;
                slots[i] = crate::item::EMPTY_SLOT;
                let mut end = slots.len();
                while end > 0 && slots[end - 1].kind == 0 && slots[end - 1].count == 0 {
                    end -= 1;
                }
                slots.truncate(end);
            }
        }
        i += 1;
    }
}

/// All requirements met: the ingredients (groups fold their members) counted
/// across the inventory and the in-range chests, plus the environment.
pub(crate) fn can_craft(
    inv: &[ItemSlot; INV],
    chests: &[WorldChest],
    in_range: &[usize],
    recipe: &Recipe,
    adj: &Adjacent,
    zones: &Zones,
) -> bool {
    let owned = collect_owned(inv, chests, in_range);
    craftable(recipe, &owned, adj, zones)
}

/// `CraftingRequests.CraftItem`'s drain (`CraftingRequests.cs:97-121`,
/// `Consume :218-254`): the inventory first, then each chest in collection
/// order, one `ConsumeItemsFrom`-shaped pass per container matching
/// `RequiredItemEntry.Matches` (any group member for a group entry).
pub(crate) fn try_craft(
    inv: &mut [ItemSlot; INV],
    chests: &mut [WorldChest],
    in_range: &[usize],
    recipe: &Recipe,
    adj: &Adjacent,
    zones: &Zones,
    rng: &mut UnifiedRandom,
) -> Option<(u16, u32)> {
    if !can_craft(inv, chests, in_range, recipe, adj, zones) {
        return None;
    }
    for req in recipe.required {
        // `GetIngredientsForOneCraft` (`Recipe.cs:196-212`): the discount
        // comes off the consumed stack (availability counts full stacks).
        let discount = ingredient_discount(recipe.alchemy, adj.alchemy_table, req.stack, rng);
        let mut to_consume = req.stack - discount;
        // The inventory pass (`Main.LocalPlayer.inventory`, 58 slots). A
        // group entry drains whichever member the slots hold first.
        if req.group != NO_GROUP {
            if let Some(g) = group(req.group) {
                for m in g.members {
                    if to_consume == 0 {
                        break;
                    }
                    consume_kind(inv, *m, &mut to_consume);
                }
            }
        } else {
            consume_kind(inv, req.item, &mut to_consume);
        }
        // The chest passes (`foreach (Chest chest in Recipe._recipeChests)`).
        for &i in in_range {
            if to_consume == 0 {
                break;
            }
            consume_chest(&mut chests[i].slots, req, &mut to_consume);
        }
        debug_assert_eq!(to_consume, 0, "craftable checked the totals");
    }
    Some(recipe.create)
}

/// The full recipe table - GENERATED from the decompiled `Recipe.cs`
/// `SetupRecipes` (1.4.5.8) by `tools/gen_recipes.py` (2764 rows: the main
/// body, the furniture helpers, the critter statues, the loop ranges and the
/// derived reverse platform/wall recipes, in the C# registration order).
/// Do not edit by hand - rerun the generator.
pub(crate) static RECIPES: &[Recipe] = include!("recipes_gen.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::slot_for_pickup;

    #[test]
    fn the_craft_rolls_come_from_the_unified_random_stream() {
        // S1-L-11e / D6: `GetIngredientCraftingDiscount` rolls
        // `Main.rand.Next(3)` (`Recipe.cs:221`) on the C#'s `UnifiedRandom`
        // — the port's discount path consumes the same generator, so a
        // same-seed `UnifiedRandom` replays the discount draws exactly and
        // the stream advances by exactly `stack` draws (watched RED on the
        // splitmix stand-in).
        let mut ref_rng = UnifiedRandom::new(7);
        let mut want = 0u32;
        for _ in 0..9 {
            if ref_rng.next_max(3) == 0 {
                want += 1;
            }
        }
        assert!(want > 0, "the seed rolls at least one free unit");
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(ingredient_discount(true, true, 9, &mut rng), want);
        assert_eq!(rng.peek(), ref_rng.peek(), "exactly 9 draws consumed");
    }

    fn inv_with(entries: &[(u16, u32)]) -> [ItemSlot; INV] {
        let mut inv = [crate::item::EMPTY_SLOT; INV];
        for (id, count) in entries {
            let mut left = *count;
            while left > 0 {
                let i = slot_for_pickup(&mut inv, *id, 0).expect("fixture overflow");
                let add = left.min(999 - inv[i].count);
                inv[i].count += add;
                left -= add;
            }
        }
        inv
    }

    /// A chest-slot fixture (the store's `ItemSlot` shape; the name is the
    /// registry's, matching `set_chest_slot`'s derive-at-write).
    fn chest_slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    fn recipe_by_create(create: u16) -> &'static Recipe {
        // The first authored (non-reverse) row: the C# lookup order over the
        // base table (`Main.recipe[i]` before the derived inserts).
        RECIPES
            .iter()
            .find(|r| r.create.0 == create && !r.not_decraftable)
            .unwrap()
    }

    fn dry_adj() -> Adjacent {
        Adjacent {
            tiles: Vec::new(),
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        }
    }

    fn world_with_station(kind: u16, x: i32, y: i32) -> World {
        let mut w = World::new("t".into(), 120, 120);
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        w
    }

    #[test]
    fn the_progression_batch_registers_verbatim() {
        // The C# anchors: the wooden tools (the workbench), the bars (the
        // furnace), the metal armor (the anvil). The generated rows author
        // the plain item ids exactly as `SetupRecipes` does.
        let sword = recipe_by_create(24);
        assert_eq!(sword.stations, &[18]);
        assert_eq!(sword.required.len(), 1);
        assert_eq!(sword.required[0].item, 9);
        assert_eq!(sword.required[0].stack, 7);
        let hammer = recipe_by_create(196);
        assert_eq!(hammer.required[0].stack, 8);
        let bow = recipe_by_create(39);
        assert_eq!(bow.required[0].stack, 10);
        let arrow = recipe_by_create(40);
        assert_eq!(arrow.create.1, 25);
        assert_eq!(arrow.required.len(), 2);
        assert_eq!(arrow.required[0].group, RecipeGroups::WOOD);
        assert_eq!(arrow.required[1].item, 3);
        assert_eq!(arrow.stations, &[18]);
        let platform = recipe_by_create(94);
        assert_eq!(platform.create.1, 2);
        assert!(platform.stations.is_empty(), "by hand");
        // The bars at the furnace.
        for (bar, ore, stack) in [(20u16, 12u16, 3u32), (21, 14, 4), (19, 13, 4)] {
            let r = recipe_by_create(bar);
            assert_eq!(r.stations, &[17]);
            assert_eq!(r.required[0].item, ore);
            assert_eq!(r.required[0].stack, stack);
        }
        // The armor at the anvil: the copper, the iron and the silver/gold
        // rows author their plain bars (the iron row is NOT group-based in
        // the C# source).
        let copper_helm = recipe_by_create(89);
        assert_eq!(copper_helm.stations, &[16]);
        assert_eq!(copper_helm.required[0].stack, 12);
        let iron_helm = recipe_by_create(90);
        assert_eq!(iron_helm.required[0].item, 22);
        assert_eq!(iron_helm.required[0].stack, 15);
        let gold_greaves = recipe_by_create(79);
        assert_eq!(gold_greaves.required[0].stack, 25);
        // The full generated table.
        let total = RECIPES.len();
        assert!(total >= 2700, "the full table ({total})");
    }

    #[test]
    fn test_dw_1_1_table_integrity() {
        // Every create/required item id resolves in the registry; counts are
        // positive; group entries' authored items are group members; no
        // duplicate create ids; the maxRequirements cap.
        assert!(!RECIPES.is_empty());
        let mut seen = Vec::new();
        for r in RECIPES {
            assert!(
                registry::get(r.create.0).is_some(),
                "create id {}",
                r.create.0
            );
            assert!(r.create.1 > 0);
            assert!(
                !r.required.is_empty() && r.required.len() <= MAX_REQUIREMENTS,
                "recipe {:?}",
                r.create
            );
            for req in r.required {
                assert!(
                    registry::get(req.item).is_some(),
                    "required id {}",
                    req.item
                );
                assert!(req.stack > 0);
                if req.group != NO_GROUP {
                    let g = group(req.group).expect("registered group");
                    assert!(
                        g.members.contains(&req.item),
                        "{} is not a {} member",
                        req.item,
                        g.name
                    );
                }
            }
            // The C# table legitimately repeats create ids across the variant
            // rows (the five Ice Torch crafts, the wall colours, ...), so the
            // create ids are NOT unique - only the row count is pinned.
            seen.push(r.create.0);
        }
        assert!(seen.len() >= 2700, "the full table ({})", seen.len());
    }

    #[test]
    fn the_group_table_is_the_setuprecipegroups_order() {
        // `Recipe.cs:499-533`: 34 groups, ids 0..33 in registration order;
        // the spot checks quote the call sites.
        assert_eq!(GROUPS.len(), 34);
        let wood = group(RecipeGroups::WOOD).unwrap();
        assert_eq!(wood.name, "Wood");
        assert_eq!(wood.icon, 9);
        assert_eq!(
            wood.members,
            &[9, 619, 620, 621, 911, 1729, 2504, 2503, 5215]
        );
        let iron = group(RecipeGroups::IRON_BAR).unwrap();
        assert_eq!(iron.members, &[22, 704]);
        let sand = group(RecipeGroups::SAND).unwrap();
        assert_eq!(sand.members, &[169, 408, 1246, 370, 3272, 3338, 3274, 3275]);
        // `Registration order = id` (`nextRecipeGroupIndex++`).
        GROUPS
            .iter()
            .enumerate()
            .for_each(|(i, g)| assert_eq!(g.id, i as u16));
    }

    #[test]
    fn requirement_matches_any_group_member() {
        // `RequiredItemEntry.Matches` (`Recipe.cs:36-43`).
        let req = Requirement::group(9, RecipeGroups::WOOD, 10);
        assert!(req.matches(9));
        assert!(req.matches(619), "Rich Mahogany");
        assert!(req.matches(2503), "Palm Wood");
        assert!(!req.matches(3), "Stone is not wood");
        let plain = Requirement::item(23, 1);
        assert!(plain.matches(23));
        assert!(!plain.matches(9));
    }

    #[test]
    fn station_inheritance_walks_the_counts_as_chain() {
        // `Recipe.SetupTileInheritance` (`Recipe.cs:559-571`) +
        // `Player.SetAdjTile`'s recursion: near a work bench (17) satisfies a
        // sawmill (215) requirement; the 302→17→215 walk covers three hops.
        assert!(satisfies_station(17, 17));
        assert!(satisfies_station(215, 17));
        assert!(satisfies_station(215, 302));
        assert!(satisfies_station(215, 77));
        assert!(satisfies_station(215, 133));
        assert!(
            !satisfies_station(17, 215),
            "the parent does not satisfy the child"
        );
        assert!(!satisfies_station(300, 17));
    }

    #[test]
    fn test_dw_1_2_workbench_reference_recipe() {
        // `Recipe.cs:4751-4754`: 10x Wood -> Work Bench, no station - the C#
        // row authors the plain item 9 (no RequireGroup on this row).
        let wb = recipe_by_create(36);
        assert_eq!(wb.create, (36, 1));
        assert_eq!(wb.required.len(), 1);
        assert_eq!(wb.required[0].item, 9);
        assert_eq!(wb.required[0].stack, 10);
        assert_eq!(wb.required[0].group, NO_GROUP);
        assert!(wb.stations.is_empty());
    }

    #[test]
    fn test_dw_2_1_can_craft_cases() {
        // Work Bench (10x wood, by hand - the plain item 9): enough -> true.
        let zones = Zones::default();
        let adj = dry_adj();
        let wb = recipe_by_create(36);
        let inv = inv_with(&[(9, 10)]);
        assert!(can_craft(&inv, &[], &[], wb, &adj, &zones));
        // Not enough wood -> false. Rich Mahogany (619) does NOT substitute -
        // the bench row has no group (`Recipe.cs:4751-4754`).
        let inv = inv_with(&[(9, 9)]);
        assert!(!can_craft(&inv, &[], &[], wb, &adj, &zones));
        let inv = inv_with(&[(619, 10)]);
        assert!(!can_craft(&inv, &[], &[], wb, &adj, &zones));
        // The group substitute lives on the rows that carry RequireGroup -
        // the torch (1 Gel + 1 of any wood, `Recipe.cs:610-615`).
        let torch = recipe_by_create(8);
        let inv = inv_with(&[(23, 1), (9, 1)]);
        assert!(can_craft(&inv, &[], &[], torch, &adj, &zones));
        let inv = inv_with(&[(23, 1), (619, 1)]);
        assert!(can_craft(&inv, &[], &[], torch, &adj, &zones));
        // 1 gel + a mix of two woods: the group fold reaches 1.
        let inv = inv_with(&[(23, 1)]);
        assert!(!can_craft(&inv, &[], &[], torch, &adj, &zones));
    }

    #[test]
    fn the_environment_gates_the_liquid_and_biome_recipes() {
        // `PlayerMeetsEnvironmentConditions` (`Recipe.cs:322-358`).
        let zones = Zones::default();
        let mut adj = dry_adj();
        let inv = inv_with(&[(31, 1)]);
        let water = recipe_by_create(126);
        assert!(!can_craft(&inv, &[], &[], water, &adj, &zones), "needWater");
        adj.water = true;
        assert!(can_craft(&inv, &[], &[], water, &adj, &zones));
        // The honey bottle needs honey, not water.
        let honey = recipe_by_create(1134);
        assert!(!can_craft(&inv, &[], &[], honey, &adj, &zones));
        adj.honey = true;
        assert!(can_craft(&inv, &[], &[], honey, &adj, &zones));
        // The lava bomb: lava only.
        let lava_bomb = recipe_by_create(4825);
        let inv = inv_with(&[(4827, 1)]);
        adj.honey = false;
        assert!(!can_craft(&inv, &[], &[], lava_bomb, &adj, &zones));
        adj.lava = true;
        assert!(can_craft(&inv, &[], &[], lava_bomb, &adj, &zones));
        // The snow cloud: the Sky Mill + the snow zone, both required.
        let snow = recipe_by_create(3756);
        let mut snow_adj = Adjacent {
            tiles: vec![305],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let inv = inv_with(&[(751, 1)]);
        assert!(
            !can_craft(&inv, &[], &[], snow, &snow_adj, &zones),
            "no snow zone"
        );
        let zones = Zones {
            snow: true,
            graveyard: false,
        };
        assert!(can_craft(&inv, &[], &[], snow, &snow_adj, &zones));
        snow_adj.tiles = vec![18];
        assert!(
            !can_craft(&inv, &[], &[], snow, &snow_adj, &zones),
            "no Sky Mill"
        );
        // The graveyard shelf: the Bone Welder (300) + the graveyard zone.
        let shelf = recipe_by_create(1389);
        let gy_adj = Adjacent {
            tiles: vec![300],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let inv = inv_with(&[(9, 1)]);
        let zones = Zones {
            snow: false,
            graveyard: false,
        };
        assert!(!can_craft(&inv, &[], &[], shelf, &gy_adj, &zones));
        let zones = Zones {
            snow: false,
            graveyard: true,
        };
        assert!(can_craft(&inv, &[], &[], shelf, &gy_adj, &zones));
    }

    #[test]
    fn scan_adjacent_reads_liquids_and_the_water_tiles() {
        // `Player.cs:35685-35693`: liquid > 200 by type + the sink/fountain
        // tiles count as water.
        let mut w = World::new("t".into(), 40, 40);
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0;
        p.y = 20.0 * 16.0;
        let adj = scan_adjacent(&w, &p);
        assert!(!adj.water && !adj.honey && !adj.lava);
        // Shallow water (liquid <= 200) does not count.
        let i = w.index(20, 20);
        w.tiles[i].liquid = 100;
        let adj = scan_adjacent(&w, &p);
        assert!(!adj.water, "liquid > 200 only");
        w.tiles[i].liquid = 220;
        let adj = scan_adjacent(&w, &p);
        assert!(adj.water);
        w.tiles[i].liquid = 0;
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        w.tiles[i].liquid = 220;
        let adj = scan_adjacent(&w, &p);
        assert!(adj.lava && !adj.water);
        // The sink tile (172) counts as a water source while dry.
        w.tiles[i].liquid = 0;
        w.tiles[i].clear_liquid_kind();
        w.tiles[i].kind = 172;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let adj = scan_adjacent(&w, &p);
        assert!(adj.water, "CountsAsWaterForCrafting");
    }

    #[test]
    fn the_chest_range_scan_finds_tile_chests_in_600px() {
        // `NearbyChests.GetChestsInRangeOf` (`NearbyChests.cs:10-31`): the
        // chest center within 600 px of the player's feet.
        let mut w = World::new("t".into(), 120, 120);
        let mut p = Player::spawn(&w);
        p.x = 40.0 * 16.0;
        p.y = 40.0 * 16.0;
        for (x, y) in [(45, 45), (42, 40), (80, 80)] {
            let i = w.index(x, y);
            w.tiles[i].kind = 21;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let chests = vec![
            WorldChest {
                tile: (45, 45),
                slots: vec![chest_slot(9, 20)],
                name: String::new(),
                max_items: DEFAULT_MAX_ITEMS as usize,
            },
            WorldChest {
                tile: (42, 40),
                slots: Vec::new(),
                name: String::new(),
                max_items: DEFAULT_MAX_ITEMS as usize,
            },
        ];
        let found = chests_in_range(&w, &p, &chests, true);
        assert_eq!(found.len(), 2, "the far chest (80,80) is out of 600 px");
        assert!(found.iter().any(|&i| chests[i].tile == (45, 45)));
        // The toggle off: an empty list (`Player.Settings.CraftFromNearbyChests`).
        assert!(chests_in_range(&w, &p, &chests, false).is_empty());
    }

    #[test]
    fn the_chest_ingredients_count_and_drain() {
        let mut rng = UnifiedRandom::new(1);
        // `CollectItemsFromChests` + `Consume` (`CraftingRequests.cs:218-254`):
        // the chest stacks count toward the recipe and drain after the
        // inventory.
        let mut w = world_with_station(21, 33, 33);
        let mut p = Player::spawn(&w);
        p.x = 30.0 * 16.0;
        p.y = 30.0 * 16.0;
        let mut chests = vec![WorldChest {
            tile: (33, 33),
            slots: vec![chest_slot(9, 6), chest_slot(619, 4)],
            name: String::new(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        }];
        let in_range = chests_in_range(&w, &p, &chests, true);
        // The work bench needs 10 wood: 4 in the inventory + 10 in the chest.
        let mut inv = inv_with(&[(9, 4)]);
        let adj = dry_adj();
        let zones = Zones::default();
        assert!(can_craft(
            &inv,
            &chests,
            &in_range,
            recipe_by_create(36),
            &adj,
            &zones
        ));
        // The drain takes the inventory first, then the chest.

        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &in_range,
                recipe_by_create(36),
                &adj,
                &zones,
                &mut rng
            ),
            Some((36, 1))
        );
        assert_eq!(count_kind(&inv, 9), 0, "the inventory drained first");
        let chest = &chests[0];
        // Chest slots are indexed with implicit air (`chest.rs`): the wood's
        // slot drains to air in place, the mahogany keeps its slot.
        assert_eq!(chest.slots, [crate::item::EMPTY_SLOT, chest_slot(619, 4)]);
    }

    /// M3-CHEST-UI: draining a chest's last stocked slot folds it into the
    /// implicit-air tail — `ConsumeItemsFrom` clears the slot to air in
    /// place (`CraftingRequests.cs:262-290`), and the trailing air is
    /// implied beyond the list length (see `chest.rs`).
    #[test]
    fn draining_the_last_stocked_chest_slot_trims_the_implicit_air_tail() {
        let mut rng = UnifiedRandom::new(1);
        let mut w = world_with_station(21, 33, 33);
        let mut p = Player::spawn(&w);
        p.x = 30.0 * 16.0;
        p.y = 30.0 * 16.0;
        let mut chests = vec![WorldChest {
            tile: (33, 33),
            slots: vec![chest_slot(9, 6)],
            name: String::new(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        }];
        let in_range = chests_in_range(&w, &p, &chests, true);
        // The work bench needs 10 wood: 4 from the inventory drain the chest
        // fully.
        let mut inv = inv_with(&[(9, 4)]);
        let adj = dry_adj();
        let zones = Zones::default();
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &in_range,
                recipe_by_create(36),
                &adj,
                &zones,
                &mut rng
            ),
            Some((36, 1))
        );
        assert_eq!(chests[0].slots, [], "the drained slot folded into the tail");
    }

    /// M3-CHEST-UI DW-1.4: a store entry whose tile is dead (the chest broke
    /// with no store cleanup — the `editing.rs` seam stays untouched) never
    /// answers the range scan: `GetChestsInRangeOf` walks `Main.chest[]`
    /// (`NearbyChests.cs:18-28`), and the port's tile scan matches entries
    /// only against live chest tiles. Stale entries are always empty (a
    /// stocked chest refuses its break), so the exclusion is the
    /// empty-entry ≡ absent rule in action.
    #[test]
    fn a_stale_entry_with_a_dead_tile_is_filtered_from_the_range_scan() {
        let mut w = World::new("t".into(), 120, 120);
        let mut p = Player::spawn(&w);
        p.x = 40.0 * 16.0;
        p.y = 40.0 * 16.0;
        // One live chest tile at (45, 45); the stale entry sits where its
        // tile was broken — (60, 60) has no active tile but is within the
        // 600 px reach, so the exclusion is the tile liveness, not the range.
        let i = w.index(45, 45);
        w.tiles[i].kind = 21;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let chests = vec![
            WorldChest {
                tile: (45, 45),
                slots: vec![chest_slot(9, 20)],
                name: String::new(),
                max_items: DEFAULT_MAX_ITEMS as usize,
            },
            WorldChest {
                tile: (60, 60),
                slots: Vec::new(),
                name: String::new(),
                max_items: DEFAULT_MAX_ITEMS as usize,
            },
        ];
        let found = chests_in_range(&w, &p, &chests, true);
        assert_eq!(found.len(), 1, "the stale entry is filtered");
        assert_eq!(chests[found[0]].tile, (45, 45));
    }

    #[test]
    fn the_toggle_off_ignores_the_chests() {
        // `CraftFromNearbyChests` off: the chest stacks don't count
        // (`Recipe.cs:386-398`).
        let mut w = world_with_station(21, 33, 33);
        let mut p = Player::spawn(&w);
        p.x = 30.0 * 16.0;
        p.y = 30.0 * 16.0;
        let chests = vec![WorldChest {
            tile: (33, 33),
            slots: vec![chest_slot(9, 20)],
            name: String::new(),
            max_items: DEFAULT_MAX_ITEMS as usize,
        }];
        let inv = inv_with(&[(9, 4)]);
        let adj = dry_adj();
        let zones = Zones::default();
        let wb = recipe_by_create(36);
        let owned_off = collect_owned(&inv, &chests, &chests_in_range(&w, &p, &chests, false));
        assert!(!craftable(wb, &owned_off, &adj, &zones));
        let owned_on = collect_owned(&inv, &chests, &chests_in_range(&w, &p, &chests, true));
        assert!(craftable(wb, &owned_on, &adj, &zones));
    }

    #[test]
    fn available_is_the_update_recipe_list_composition() {
        // `Recipe.UpdateRecipeList` (`Recipe.cs:230-259`): environment x
        // enough-ingredients — no partial rows.
        let mut w = world_with_station(18, 31, 32);
        let mut p = Player::spawn(&w);
        p.x = 30.0 * 16.0;
        p.y = 30.0 * 16.0;
        let adj = scan_adjacent(&w, &p);
        assert!(adj.tiles.contains(&18), "the work bench adjacent");
        let zones = Zones::default();
        // 16 wood: the work bench (10) is craftable, the door (6) too.
        let inv = inv_with(&[(9, 16)]);
        let av = available(&inv, &[], &[], &adj, &zones);
        assert!(av.iter().any(|&i| RECIPES[i].create.0 == 36));
        assert!(av.iter().any(|&i| RECIPES[i].create.0 == 25));
        // The water bottle is not: no water, no bottle.
        assert!(!av.iter().any(|&i| RECIPES[i].create.0 == 126));
        // 9 wood: the bench drops off the list.
        let inv = inv_with(&[(9, 9)]);
        let av = available(&inv, &[], &[], &adj, &zones);
        assert!(!av.iter().any(|&i| RECIPES[i].create.0 == 36));
    }

    #[test]
    fn addrecipe_sets_alchemy_on_the_tile_13_recipes() {
        // `AddRecipe` (`Recipe.cs:16722-16726`): `requiredTile == 13` ⇒
        // `alchemy = true` — the discount gate.
        for r in RECIPES {
            if r.stations.contains(&13) {
                assert!(r.alchemy, "recipe {:?}", r.create);
            }
        }
        // And the two subset potions are there.
        assert!(recipe_by_create(28).alchemy && recipe_by_create(28).stations == &[13]);
        assert!(recipe_by_create(188).alchemy && recipe_by_create(188).stations == &[13]);
    }

    #[test]
    fn the_alchemy_table_discounts_a_third_of_the_units() {
        // `GetIngredientCraftingDiscount` (`Recipe.cs:214-228`): while the
        // recipe brews at an alchemy table, each unit survives on a 1/3 roll.
        let mut w = world_with_station(13, 30, 32); // the placed bottle
        let i = w.index(32, 32);
        w.tiles[i].kind = 355; // the alchemy table next to it
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = Player::spawn(&w);
        p.x = 30.0 * 16.0;
        p.y = 30.0 * 16.0;
        let adj = scan_adjacent(&w, &p);
        assert!(adj.alchemy_table, "the alchemy table is adjacent");
        // Near the table the bottle requirement is satisfied too
        // (`TileCountsAs` 355->13).
        assert!(adj.tiles.iter().any(|&k| satisfies_station(13, k)));
        let zones = Zones::default();
        let lhp = recipe_by_create(28);
        let mut chests = Vec::new();
        // Generous stock: 9 mushroom, 18 gel, 18 bottles.
        let mut inv = inv_with(&[(5, 9), (23, 18), (31, 18)]);
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(
            try_craft(&mut inv, &mut chests, &[], lhp, &adj, &zones, &mut rng),
            Some((28, 2))
        );
        // The consumed amounts are the stacks minus the rolled-off units —
        // recompute the expectation from the same stream state.
        let mut expect_rng = UnifiedRandom::new(7);
        let discount =
            |stack: u32, rng: &mut UnifiedRandom| ingredient_discount(true, true, stack, rng);
        let d1 = discount(1, &mut expect_rng);
        let d2 = discount(2, &mut expect_rng);
        let d3 = discount(2, &mut expect_rng);
        assert!(d1 + d2 + d3 > 0, "the seed rolls at least one free unit");
        assert_eq!(count_kind(&inv, 5), 9 - (1 - d1));
        assert_eq!(count_kind(&inv, 23), 18 - (2 - d2));
        assert_eq!(count_kind(&inv, 31), 18 - (2 - d3));
        // The bottle without the alchemy table: every unit drains.
        let bottle_only = Adjacent {
            tiles: vec![13],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let mut inv = inv_with(&[(5, 1), (23, 2), (31, 2)]);
        let mut rng = UnifiedRandom::new(7);
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                lhp,
                &bottle_only,
                &zones,
                &mut rng
            ),
            Some((28, 2))
        );
        assert_eq!(count_kind(&inv, 5), 0);
        assert_eq!(count_kind(&inv, 23), 0);
        assert_eq!(count_kind(&inv, 31), 0);
    }

    #[test]
    fn test_dw_2_2_try_craft_consumes_and_returns() {
        let mut rng = UnifiedRandom::new(1);
        // Wood spread across three slots (4 + 4 + 2): a 10x-wood recipe
        // consumes all three, exact amounts.
        let mut inv = inv_with(&[(9, 4)]);
        inv[1] = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 4,
            prefix: 0,
        };
        inv[2] = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 2,
            prefix: 0,
        };
        let wb = recipe_by_create(36);
        let adj = dry_adj();
        let zones = Zones::default();
        let mut chests = Vec::new();
        assert_eq!(
            try_craft(&mut inv, &mut chests, &[], wb, &adj, &zones, &mut rng),
            Some((36, 1))
        );
        assert_eq!(count_kind(&inv, 9), 0);
    }

    #[test]
    fn test_dw_2_3_scan_adjacent_semantics() {
        let w = world_with_station(18, 18, 18);
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0; // tile 20: region x 15..=26, y 17..=18+
        p.y = 20.0 * 16.0;
        let adj = scan_adjacent(&w, &p);
        assert!(
            adj.tiles.contains(&18),
            "work bench adjacent: {:?}",
            adj.tiles
        );
        // Far away: the station is out of the region.
        let w2 = world_with_station(18, 100, 100);
        let adj2 = scan_adjacent(&w2, &p);
        assert!(!adj2.tiles.contains(&18), "{:?}", adj2.tiles);
    }

    #[test]
    fn the_station_reach_region_matches_the_csharp_xna_rect() {
        // `Player.AdjTiles` (`Player.cs:35671-35674`) scans
        // `WorldUtils.ClampToWorld(TileReachCheckSettings.Simple.GetTileRegion(this))`.
        // `GetTileRegion` returns `new Rectangle(LX, LY, HX - LX, HY - LY)`
        // (`TileReachCheckSettings.cs:56-60`) and the **XNA** `Rectangle.Right`
        // is `X + Width` (exclusive — the same convention as
        // `tr_core::whip::Rect::right`), so `Right == HX` and the C#'s
        // `for (i = Left; i <= Right; i++)` (`:35672`) covers `LX..=HX`.
        // The port's inclusive `lx..=hx` is therefore 1:1; S1-L-11b's
        // "one tile wider" claim assumed the System.Drawing
        // `Right = X + Width - 1` convention (docs/research/21 D2).
        let seen = |x: i32, y: i32| {
            let w = world_with_station(18, x, y);
            let mut p = Player::spawn(&w);
            p.x = 20.0 * 16.0; // LX = 20-5 = 15, HX = ceil(340/16)-1+5 = 26
            p.y = 20.0 * 16.0; // LY = 20-3 = 17, HY = ceil(362/16)-1+3 = 25
            scan_adjacent(&w, &p).tiles.contains(&18)
        };
        assert!(seen(15, 20), "the left edge is in reach");
        assert!(!seen(14, 20), "one tile left of the reach is out");
        assert!(
            seen(26, 20),
            "HX is inclusive (XNA Rectangle.Right == X + Width)"
        );
        assert!(!seen(27, 20), "one tile past HX is out");
        assert!(seen(20, 17), "the top edge is in reach");
        assert!(!seen(20, 16), "one tile above the reach is out");
        assert!(seen(20, 25), "HY is inclusive");
        assert!(!seen(20, 26), "one tile below HY is out");
    }

    #[test]
    fn test_dw_2_4_station_gate_and_dirty_inputs() {
        // Iron Bar requires a furnace adjacent: without one, no craft even
        // with the ingredient present.
        let zones = Zones::default();
        let adj = dry_adj();
        let inv = inv_with(&[(11, 3)]);
        let bar = recipe_by_create(22);
        assert!(!can_craft(&inv, &[], &[], bar, &adj, &zones));
        let furnace = Adjacent {
            tiles: vec![17],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        assert!(can_craft(&inv, &[], &[], bar, &furnace, &zones));
        // Empty inventory: cannot craft anything.
        let inv = [crate::item::EMPTY_SLOT; INV];
        assert!(!can_craft(
            &inv,
            &[],
            &[],
            recipe_by_create(36),
            &adj,
            &zones
        ));
        // Two different ingredients at once (torch: gel + wood).
        let mut inv = inv_with(&[(23, 1)]);
        let torch = recipe_by_create(8);
        assert!(!can_craft(&inv, &[], &[], torch, &adj, &zones));
        slot_for_pickup(&mut inv, 9, 0);
        inv.iter_mut().find(|s| s.kind == 9).unwrap().count = 1;
        assert!(can_craft(&inv, &[], &[], torch, &adj, &zones));
    }

    #[test]
    fn test_dw_2_5_full_inventory_still_crafts() {
        let mut rng = UnifiedRandom::new(1);
        // Fill every slot with stone, keep 10 wood in the first slot: the
        // work bench still crafts (the result-drop duty is the caller's, like
        // vanilla `Item.giveItem` dropping on the ground).
        let mut inv = inv_with(&[(9, 10)]);
        for slot in inv.iter_mut().skip(1) {
            *slot = ItemSlot {
                kind: 3,
                name: "Stone Block",
                count: 999,
                prefix: 0,
            };
        }
        let wb = recipe_by_create(36);
        let adj = dry_adj();
        let zones = Zones::default();
        let mut chests = Vec::new();
        assert_eq!(
            try_craft(&mut inv, &mut chests, &[], wb, &adj, &zones, &mut rng),
            Some((36, 1))
        );
    }

    #[test]
    fn furniture_recipe_crafts_and_consumes() {
        let mut rng = UnifiedRandom::new(1);
        // 16 wood -> work bench (by hand) leaves 6, exactly a door — which
        // then needs the bench standing next to the player. The chest needs
        // the iron bar too: wood alone is refused.
        let zones = Zones::default();
        let adj = dry_adj();
        let mut chests = Vec::new();
        let mut inv = inv_with(&[(9, 16)]);
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(36),
                &adj,
                &zones,
                &mut rng
            ),
            Some((36, 1))
        );
        assert_eq!(count_kind(&inv, 9), 6);
        let door = recipe_by_create(25);
        let bench = Adjacent {
            tiles: vec![18],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        assert!(
            !can_craft(&inv, &[], &[], door, &adj, &zones),
            "no station adjacent yet"
        );
        assert_eq!(
            try_craft(&mut inv, &mut chests, &[], door, &bench, &zones, &mut rng),
            Some((25, 1))
        );
        assert_eq!(count_kind(&inv, 9), 0);
        // The chest's wood is the plain item 9 (no group on the C# row);
        // the iron-bar half still draws from the IronBar group.
        let mut inv = inv_with(&[(9, 8)]);
        assert!(!can_craft(
            &inv,
            &[],
            &[],
            recipe_by_create(48),
            &bench,
            &zones
        ));
        let mut inv = inv_with(&[(9, 8), (704, 2)]);
        assert!(
            can_craft(&inv, &[], &[], recipe_by_create(48), &bench, &zones),
            "Lead Bar is IronBar"
        );
    }

    #[test]
    fn the_utility_batch_chains_the_stations() {
        // The C# rows: Chain x15 (anvil), Sawmill (work bench + the chain),
        // Loom (sawmill), Sign (work bench), Cooking Pot (anvil), Bowl
        // (furnace, clay), Gold Candle (work bench, gold bar + torch).
        let mut rng = UnifiedRandom::new(1);
        let zones = Zones::default();
        let anvil = Adjacent {
            tiles: vec![16],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let bench = Adjacent {
            tiles: vec![18],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let sawmill = Adjacent {
            tiles: vec![106],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let furnace = Adjacent {
            tiles: vec![17],
            water: false,
            honey: false,
            lava: false,
            alchemy_table: false,
        };
        let mut chests = Vec::new();
        // Chain: one Lead Bar (the IronBar group) makes 15 — the anvil gates it.
        let mut inv = inv_with(&[(704, 1)]);
        assert!(
            !can_craft(&inv, &[], &[], recipe_by_create(85), &bench, &zones),
            "anvil required"
        );
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(85),
                &anvil,
                &zones,
                &mut rng
            ),
            Some((85, 15))
        );
        // The sawmill: wood + iron bars + the fresh chain, at the work bench.
        let mut inv = inv_with(&[(9, 10), (704, 2), (85, 1)]);
        assert!(
            !can_craft(&inv, &[], &[], recipe_by_create(363), &anvil, &zones),
            "work bench required"
        );
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(363),
                &bench,
                &zones,
                &mut rng
            ),
            Some((363, 1))
        );
        // The loom rides the sawmill station; the sign is a plain work bench row.
        let mut inv = inv_with(&[(9, 12)]);
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(332),
                &sawmill,
                &zones,
                &mut rng
            ),
            Some((332, 1))
        );
        let mut inv = inv_with(&[(9, 6)]);
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(171),
                &bench,
                &zones,
                &mut rng
            ),
            Some((171, 1))
        );
        // The cooking pot: iron bars + wood at the anvil. The bowl: clay at
        // the furnace. The gold candle: a gold bar + a torch at the work bench.
        let mut inv = inv_with(&[(22, 10), (9, 2)]);
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(345),
                &anvil,
                &zones,
                &mut rng
            ),
            Some((345, 1))
        );
        let mut inv = inv_with(&[(133, 2)]);
        assert!(
            !can_craft(&inv, &[], &[], recipe_by_create(356), &bench, &zones),
            "furnace required"
        );
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(356),
                &furnace,
                &zones,
                &mut rng
            ),
            Some((356, 1))
        );
        let mut inv = inv_with(&[(19, 1), (8, 1)]);
        assert_eq!(
            try_craft(
                &mut inv,
                &mut chests,
                &[],
                recipe_by_create(105),
                &bench,
                &zones,
                &mut rng
            ),
            Some((105, 1))
        );
    }
}
