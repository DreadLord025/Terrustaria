//! World chest loot generation — the port of `WorldGen.AddBuriedChest`'s
//! stocking block (`WorldGen.cs:35871-37580`).
//!
//! The seam (S1-A-4): the loot rolls at GENERATION time, exactly the
//! vanilla's inline shape. The placement sites in `tr-core` resolve the
//! anchor's context and band (`tr_core::worldgen::chest_stock`), hand the
//! [`ChestSiteEvent`](tr_core::worldgen::chest_stock::ChestSiteEvent) to the
//! [`ChestStock`] hook the app passes into `generate_with_chest_stock`, and
//! write the rolled rows into `World.chests` — the records that save with
//! the world and become the store at load. Nothing rolls at load: a `.wld`'s
//! store is the file's chests (`LoadChests`, `WorldFile.cs:2723-2805`), a
//! `.trwld`'s store is the file's native records. The draws ride the
//! placing pass's own stream (`WorldGenRand`) — the C#'s `genRand`
//! interleave at the pass granularity (the port's per-pass stream model,
//! ANALYSIS) — and the prefix rolls draw from the same stream
//! (`Item.Prefix`'s `isGeneratingOrLoadingWorld → WorldGen.genRand` arm,
//! `Item.cs:531`).
//!
//! The roll is the whole C# block: the depth band (surface
//! `y < worldSurface+25` / mid `y < rockLayer` / deep
//! `y < maxTilesY-250` / else), the rows with their verbatim probabilities
//! (`Next(6) == 0` etc), the stack ranges, and the `Prefix(-1)` calls
//! routed through the ported `prefix::roll_prefix` engine.
//! Out of scope here (documented in the ROADMAP row): the seed-flavour rows
//! (vampire/errorWorld/rainbow/tenthAnniversary, `SecretSeed`/`Main`
//! statics the port does not generate), the `generatedShadowKey`/
//! `generatedRamRune` fairness flags (the underworld/water arms they gate
//! have no gen-time sites yet), the remix/tenth-anniversary/FTW variants of
//! the ported arms.

use crate::chest::WorldChest;
use crate::item::{EMPTY_SLOT, ItemSlot};
use crate::prefix;
use tr_core::save::{ChestItem, ChestRecord};
use tr_core::tile::FLAG_ACTIVE;
use tr_core::world::World;

/// The chest tile kinds an anchor can carry (`craft::CHEST_TILES`).
const CHEST_KINDS: [u16; 2] = [21, 341];
/// `GenVars.hellChestItem` — the underwater-chest 939 override carries the
/// water chests' item; the unused slot keeps the vanilla name.
const UNDERWATER_ITEM: u16 = 939;

/// `WorldGen.AddBuriedChest`'s `flag3` arm's main table (`:36043`): the
/// underground-desert family's slot 0 — the AncientChisel 4056 / SandBoots
/// 4055 / MysticCoilSnake 4262 / MagicConch 4263 shallow row and the
/// ThunderSpear 4061 / ThunderStaff 4062 / CatBast 4276 deep row, the split
/// by `flag20`'s desert-hive depth test (`:36041`).
const DESERT_CHEST_SHALLOW: [u16; 4] = [4056, 4055, 4262, 4263];
const DESERT_CHEST_DEEP: [u16; 3] = [4061, 4062, 4276];
/// The `flag3` arm's depth test's divisor — `(High*3 + Low*4) / 7` with the
/// no-hive `Low = 0`.
const DESERT_DEPTH_DIVISOR: i32 = 7;

/// `Pyramid`'s own chest (`WorldGen.cs:27853-27873`): the double-roll
/// `Next(3)` with the 0 → `Next(3)` re-roll, so the Pharaoh'sMask 848 only
/// on two zeros (1/9) and the SandstorminaBottle 857 / FlyingCarpet 934
/// otherwise (`Main.tenthAnniversaryWorld`'s `0 → 1` rides M5-SEED-1).
const PYRAMID_CHEST: [u16; 3] = [848, 857, 934];
/// The Pharaoh'sRobe 866 — `flag11`'s companion row, written into slot 1
/// right after the mask's slot 0 (`:36234-36239`).
const PHARAOHS_ROBE: u16 = 866;

/// `flag2`'s main table (`:36075`): the ice family's seven items.
const ICE_CHEST: [u16; 7] = [670, 724, 950, 1319, 987, 1579, 6153];
/// The remix variant of the SnowballCannon pick (1319 → the IceBow 725,
/// `:36076-36079`) — parked with the remix worlds, which the port does not
/// generate.
const ICE_CHEST_REMIX: u16 = 1319;
const ICE_CHEST_REMIX_ITEM: u16 = 725;
/// The ice row's 1/20 Extractinator 997 override (`:36080-36083`).
const ICE_EXTRACTINATOR_ODDS: u64 = 20;
const EXTRACTINATOR: u16 = 997;

/// `flag8`'s main: `GenVars.hellChestItem[GenVars.hellChest]` (`:36118`) —
/// the five hell-chest items shuffled once per world at `WorldGen.Reset`
/// (`:11246-11258`, the remix row swaps the FlowerofFire 112 for the
/// UnholyTrident 683).
const HELL_CHEST: [u16; 5] = [274, 220, 112, 218, 3019];
const HELL_CHEST_REMIX: [u16; 5] = [274, 220, 683, 218, 3019];

/// `IslandHouse`'s main table (`WorldGen.cs:79344-79363`): the
/// `GenVars.skyIslandHouseCount` cycle over the ShinyRedBalloon 159 /
/// Starfury 65 / LuckyHorseshoe 158 / CelestialMagnet 2219, with the
/// `> 3 → Next(4)` re-roll on the fifth and later house.
const SKY_ISLAND_CHEST: [u16; 4] = [159, 65, 158, 2219];

/// The `GenVars` bar item tables (`SavedOreTiers`/`GenVars` statics):
/// copper/iron/silver/gold bars. The port generates ore tiers by id, not
/// the tier statics — these are the vanilla mid-tier ids.
const BAR_COPPER: u16 = 20;
const BAR_IRON: u16 = 22;
const BAR_SILVER: u16 = 21;
const BAR_GOLD: u16 = 19;

/// The vanilla ore tiers the bar tables pick with:
/// copper 908 / iron(silver in vanilla mid) 909 / silver 168 / gold 940.
const ORE_COPPER: u16 = 908;

/// One stocked row: item id, stack, and the prefix id the roll picked
/// (0 when the vanilla row carries no `Prefix(-1)`).
struct Row {
    kind: u16,
    count: u32,
    prefix: u16,
}

impl Row {
    fn plain(kind: u16) -> Self {
        Self {
            kind,
            count: 1,
            prefix: 0,
        }
    }

    fn stacked(kind: u16, count: u32) -> Self {
        Self {
            kind,
            count,
            prefix: 0,
        }
    }
}

/// `UnifiedRandom.Next(max)` — `[0, max)` — over the placing pass's own
/// stream (S1-A-4: the C#'s `genRand` draws, the pass-granularity stand-in).
fn next(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, max: u64) -> u64 {
    if max == 0 {
        return 0;
    }
    tr_core::worldgen::chest_stock::WorldGenRand::next(rng, max as i32) as u64
}

/// `Next(min, max)` — `[min, max)`.
fn ranged(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, min: u64, max: u64) -> u64 {
    min + next(rng, max.saturating_sub(min))
}

/// The band/context types live in `tr_core::worldgen::chest_stock` now
/// (S1-A-4): the placement sites resolve them from the world state and ride
/// the `ChestSiteEvent`.
use tr_core::worldgen::chest_stock::Band;

/// `WorldGen.GetDungeonLootAndChestStyle` (:35800-35848): the 8-row dungeon
/// cycle + the hell override. The counter wrap and the advance are
/// vanilla-exact (`dungeonLootStyle >= 8 → 0` before the switch; the
/// `DungeonUtils.cs:368` advance after a successful place).
struct DungeonCycle {
    style: u8,
}

impl DungeonCycle {
    fn new() -> Self {
        Self { style: 0 }
    }

    /// The `(item, chest_style)` the current row carries; advances after a
    /// successful place (`DungeonUtils.cs:366-369` — style 0 only).
    fn take(&mut self, hell: bool) -> Option<(u16, i32)> {
        if self.style >= 8 {
            self.style = 0;
        }
        if hell {
            return Some((327, 0));
        }
        let row = match self.style {
            0 => (155, 2),
            1 => (156, 2),
            2 => (157, 2),
            3 => (163, 2),
            4 => (113, 2),
            5 => (3317, 2),
            6 => (327, 0),
            _ => (164, 2),
        };
        Some(row)
    }

    fn advance(&mut self, placed: bool) {
        if placed {
            self.style += 1;
        }
    }
}

/// The per-chest context the style dispatcher reads — `ChestCtx`, resolved
/// by the placement site (S1-A-4: `tr_core::worldgen::chest_stock`). The
/// alias keeps the stocking arms' spelling.
use tr_core::worldgen::chest_stock::ChestCtx as Context;

/// The `flag3` arm's `flag20` depth test with the no-hive `GenVars` values
/// (`High = maxTilesY`, `Low = 0`, `GenVars.cs:142-148`): the vanilla's
/// hive bounds ride the worldgen-only `GenVars` and are gone by the
/// load-time sweep, so the port keeps the reset values — the deep table
/// below three sevenths of the world height (ANALYSIS).
fn deep_desert_chest(height: i32, y: i32) -> bool {
    y > height * 3 / DESERT_DEPTH_DIVISOR
}

/// The `flag3` arm's main pick (`:36043`): `Utils.SelectRandom` over the
/// shallow four, or the deep three when `flag20` holds (`:36041`).
fn desert_chest_main(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, deep: bool) -> u16 {
    let table: &[u16] = if deep {
        &DESERT_CHEST_DEEP
    } else {
        &DESERT_CHEST_SHALLOW
    };
    table[next(rng, table.len() as u64) as usize]
}

/// `flag2`'s main pick (`:36075-36083`): the seven-item table then the 1/20
/// Extractinator override, in the C# order. The remix's `1319 → 725` swap
/// (`:36076-36079`) stays parked — the port generates no remix worlds.
fn ice_chest_main(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) -> u16 {
    let mut item = ICE_CHEST[next(rng, ICE_CHEST.len() as u64) as usize];
    if item == ICE_CHEST_REMIX {
        // The remix arm — parked with the remix worlds (`ICE_CHEST_REMIX_ITEM`).
    }
    if next(rng, ICE_EXTRACTINATOR_ODDS) == 0 {
        item = EXTRACTINATOR;
    }
    item
}

/// `WorldGen.Reset`'s hell-chest shuffle (`:11246-11258`): the
/// `while (list.Count > 0)` draw — a uniform pick-and-remove over the five
/// items, once per world before any chest stocks. The vanilla shuffles the
/// worldgen `genRand` at the reset; the port runs the same algorithm over
/// the sweep's stream at the front of `generate_world_chests`, so a world
/// re-loads with the same table (ANALYSIS).
fn shuffle_hell_chest(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, remix: bool) -> [u16; 5] {
    let mut list: Vec<u16> = if remix {
        HELL_CHEST_REMIX.to_vec()
    } else {
        HELL_CHEST.to_vec()
    };
    let mut out = [0u16; 5];
    for slot in out.iter_mut() {
        let index = next(rng, list.len() as u64) as usize;
        *slot = list[index];
        list.remove(index);
    }
    out
}

/// The `flag14` arm's counter (`:36210-36217`): `GenVars.hellChest++` after
/// every hell-chest place, wrapping at the table's length — the port models
/// the vanilla's `hellChest` index over the shuffled table.
fn next_hell_chest(table: &[u16; 5], counter: &mut usize) -> u16 {
    let item = table[*counter];
    *counter += 1;
    if *counter >= table.len() {
        *counter = 0;
    }
    item
}

/// `ItemSlot::Prefix(-1)` on a row: the natural roll through the ported
/// engine (the `roll_prefix_splitmix(kind, -1, rng)` — the 1/4 no-prefix gate, the
/// ReducedNaturalChance arm, the rejection retry). The prefix lands on the
/// row's id; the id kinds outside the ported engine roll 0 (no prefix) with
/// **no stream draws** (the `CanHavePrefixes` early return).
fn roll_prefix_on(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, row: &mut Row) {
    row.prefix = prefix::roll_prefix(row.kind, -1, rng);
}

/// The surface band's weapon row (`:36269-36331`): the `Next(11)` weapon
/// pick — 280 Spear/281 Blowgun/284 Flintlock/285 Musket/953 Pistol/946
/// Shortsword/3068 MusketBall?/3069/3084/4341/6165 — each with the natural
/// prefix.
fn surface_weapon(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) -> Row {
    let num15 = next(rng, 11);
    let mut row = match num15 {
        0 => Row::plain(280),
        1 => Row::plain(281),
        2 => Row::plain(284),
        3 => Row::plain(285),
        4 => Row::plain(953),
        5 => Row::plain(946),
        6 => Row::plain(3068),
        7 => Row::plain(3069),
        8 => Row::plain(3084),
        9 => Row::plain(4341),
        _ => Row::plain(6165),
    };
    roll_prefix_on(rng, &mut row);
    row
}

/// The mid/deep/deepest bands' weapon row (the `Next(6)` pick
/// `:36583-36603`: the Shadow/Suspecting/Sergeant/Mr/General/Legend —
/// 49/50/53/54/5011/975 — or the `Next(4)` pick in the deepest
/// `:37196-37208`: 49/50/53/54).
fn deep_weapon(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, deepest: bool) -> Row {
    let pick = if deepest {
        next(rng, 4)
    } else {
        next(rng, 6)
    };
    let mut row = match pick {
        0 => Row::plain(49),
        1 => Row::plain(50),
        2 => Row::plain(53),
        3 => Row::plain(54),
        4 => Row::plain(5011),
        _ => Row::plain(975),
    };
    roll_prefix_on(rng, &mut row);
    row
}

/// The depth band's stock (`while (itemIndex == 0)` — the loop keeps the
/// vanilla order; the `AddBuriedChest` block closes with the `Next(12)`
/// voice-change row `:37546`).
fn stock_band(
    band: Band,
    ctx: &Context,
    chest_style: i32,
    chest_kind: u16,
    rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::with_capacity(16);
    // Row 0: the special slot/weapon (`flag*` extras follow the main item).
    match band {
        Band::Surface => {
            // `:36222` flag23 arm: the special item or the surface weapon.
            out.push(surface_weapon(rng));
            // `:36334-36471`: the consumable rows.
            if next(rng, 6) == 0 {
                out.push(Row::stacked(282, ranged(rng, 40, 76) as u32));
            }
            if next(rng, 6) == 0 {
                out.push(Row::stacked(279, ranged(rng, 150, 301) as u32));
            }
            if next(rng, 6) == 0 {
                out.push(Row::stacked(3093, 1));
            }
            if next(rng, 6) == 0 {
                out.push(Row::stacked(4345, 1));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(168, ranged(rng, 3, 6) as u32));
            }
            if next(rng, 2) == 0 {
                let mut bars = 0u16;
                if next(rng, 2) == 0 {
                    bars = BAR_COPPER;
                    out.push(Row::stacked(bars, ranged(rng, 3, 11) as u32));
                } else {
                    bars = BAR_IRON;
                    out.push(Row::stacked(bars, ranged(rng, 3, 11) as u32));
                }
                let _ = bars;
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(965, ranged(rng, 50, 101) as u32));
            }
            if next(rng, 3) != 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(40, ranged(rng, 25, 51) as u32));
                } else {
                    out.push(Row::stacked(42, ranged(rng, 25, 51) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(28, ranged(rng, 3, 6) as u32));
            }
            if next(rng, 3) != 0 {
                out.push(Row::stacked(2350, ranged(rng, 3, 6) as u32));
            }
            if next(rng, 3) > 0 {
                let pick = next(rng, 6);
                let herbs = match pick {
                    0 => 292,
                    1 => 298,
                    2 => 299,
                    3 => 290,
                    4 => 2322,
                    _ => 2325,
                };
                out.push(Row::stacked(herbs, ranged(rng, 1, 3) as u32));
            }
            if next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(8, ranged(rng, 11, 21) as u32));
                } else {
                    out.push(Row::stacked(31, ranged(rng, 11, 21) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(72, ranged(rng, 10, 30) as u32));
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(9, ranged(rng, 50, 100) as u32));
            }
        }
        Band::Mid => {
            // `:36503` — the `flag*`-gated extras, the deep weapon, the
            // accessory row.
            if ctx.mahogany && next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4425));
                } else {
                    out.push(Row::plain(4460));
                }
            }
            if ctx.sky && next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4425));
                } else {
                    out.push(Row::plain(4460));
                }
            }
            out.push(deep_weapon(rng, false));
            if ctx.dungeon && next(rng, 2) == 0 {
                out.push(Row::stacked(329, 1));
            }
            if ctx.dungeon && next(rng, 8) == 0 {
                out.push(Row::stacked(5465, 1));
            }
            if ctx.dungeon && next(rng, 4) == 0 {
                out.push(Row::stacked(6156, 1));
            }
            if next(rng, 20) == 0 {
                let mut muramasa = Row::plain(997);
                roll_prefix_on(rng, &mut muramasa);
                out.push(muramasa);
            } else if next(rng, 20) == 0 {
                let mut key = Row::plain(930);
                roll_prefix_on(rng, &mut key);
                out.push(key);
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(931, ranged(rng, 26, 51) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(silver_bar(), ranged(rng, 3, 6) as u32));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(52, 1));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(965, ranged(rng, 50, 101) as u32));
            }
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                let bar = if pick == 0 { iron_bar() } else { silver_bar() };
                out.push(Row::stacked(bar, ranged(rng, 10, 21) as u32));
            }
            if next(rng, 3) != 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(40, ranged(rng, 25, 51) as u32));
                } else {
                    out.push(Row::stacked(42, ranged(rng, 25, 51) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(28, ranged(rng, 3, 6) as u32));
            }
            if next(rng, 3) != 0 {
                let herbs = match next(rng, 9) {
                    0 => 289,
                    1 => 298,
                    2 => 299,
                    3 => 290,
                    4 => 303,
                    5 => 291,
                    6 => 304,
                    7 => 2322,
                    _ => 2329,
                };
                out.push(Row::stacked(herbs, ranged(rng, 2, 5) as u32));
            }
            if next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(974, ranged(rng, 11, 21) as u32));
                } else {
                    out.push(Row::stacked(8, ranged(rng, 11, 21) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(72, ranged(rng, 50, 91) as u32));
            }
        }
        Band::Deep => {
            // `:36803` — mahogany/pyramid extras, the deep weapon.
            if ctx.mahogany && next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4425));
                } else {
                    out.push(Row::plain(4460));
                }
            }
            if ctx.pyramid && next(rng, 7) == 0 {
                out.push(Row::plain(4346));
            }
            if ctx.pyramid && next(rng, 15) == 0 {
                out.push(Row::plain(4066));
            }
            if ctx.mahogany && (next(rng, 5) == 0 || next(rng, 1) == 0) {
                if next(rng, 1) == 0 {
                    out.push(Row::plain(4426));
                } else {
                    out.push(Row::plain(5525));
                }
            }
            out.push(deep_weapon(rng, false));
            if next(rng, 20) == 0 {
                let mut lava = Row::plain(906);
                roll_prefix_on(rng, &mut lava);
                out.push(lava);
            } else if next(rng, 15) == 0 {
                let mut muramasa = Row::plain(997);
                roll_prefix_on(rng, &mut muramasa);
                out.push(muramasa);
            } else {
                let mut deep_pick = deep_weapon(rng, false);
                roll_prefix_on(rng, &mut deep_pick);
                out.push(deep_pick);
                if next(rng, 20) == 0 {
                    let mut key = Row::plain(930);
                    roll_prefix_on(rng, &mut key);
                    out.push(key);
                    if next(rng, 2) == 0 {
                        out.push(Row::stacked(931, ranged(rng, 26, 51) as u32));
                    }
                }
            }
            if next(rng, 5) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(5120, 1));
                } else {
                    out.push(Row::stacked(43, 1));
                }
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(167, 1));
            }
            if next(rng, 4) == 0 {
                out.push(Row::stacked(51, ranged(rng, 26, 51) as u32));
            }
            if next(rng, 2) == 0 {
                let bar = if next(rng, 2) == 0 { gold_bar() } else { silver_bar() };
                out.push(Row::stacked(bar, ranged(rng, 3, 11) as u32));
            }
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                if pick == 0 {
                    out.push(Row::stacked(41, ranged(rng, 26, 51) as u32));
                } else {
                    out.push(Row::stacked(279, ranged(rng, 26, 51) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(188, ranged(rng, 3, 6) as u32));
            }
            if next(rng, 3) > 0 {
                let gems = match next(rng, 6) {
                    0 => 296,
                    1 => 295,
                    2 => 299,
                    3 => 302,
                    4 => 303,
                    _ => 305,
                };
                out.push(Row::stacked(gems, ranged(rng, 1, 3) as u32));
            }
            if next(rng, 3) > 1 {
                let gems = match next(rng, 6) {
                    0 => 301,
                    1 => 297,
                    2 => 304,
                    3 => 2329,
                    4 => 2351,
                    _ => 2326,
                };
                out.push(Row::stacked(gems, ranged(rng, 1, 3) as u32));
            }
            if next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    let bars = gold_bar();
                    out.push(Row::stacked(bars, ranged(rng, 16, 31) as u32));
                } else {
                    let bars = silver_bar();
                    out.push(Row::stacked(bars, ranged(rng, 16, 31) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(ore_pick(), ranged(rng, 16, 31) as u32));
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(73, ranged(rng, 2, 5) as u32));
            }
        }
        Band::Deepest => {
            // `:37162` — the idol extras, the accessories, the ore rows.
            if ctx.idol {
                if next(rng, 5) == 0 {
                    out.push(Row::plain(5010));
                }
                if next(rng, 10) == 0 {
                    out.push(Row::plain(4443));
                }
                if next(rng, 10) == 0 {
                    out.push(Row::plain(4737));
                }
                if next(rng, 10) == 0 {
                    out.push(Row::plain(4551));
                }
            }
            out.push(deep_weapon(rng, true));
            if next(rng, 3) == 0 {
                out.push(Row::stacked(167, 1));
            }
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                if pick == 0 {
                    out.push(Row::stacked(117, ranged(rng, 16, 31) as u32));
                } else {
                    out.push(Row::stacked(gold_bar(), ranged(rng, 16, 31) as u32));
                }
            }
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                let ores = 265;
                out.push(Row::stacked(ores, ranged(rng, 51, 76) as u32));
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(227, ranged(rng, 16, 21) as u32));
            }
            if next(rng, 4) > 0 {
                let gems = match next(rng, 8) {
                    0 => 296,
                    1 => 295,
                    2 => 293,
                    3 => 288,
                    4 => 294,
                    5 => 297,
                    6 => 304,
                    _ => 2323,
                };
                out.push(Row::stacked(gems, ranged(rng, 1, 3) as u32));
            }
            if next(rng, 3) > 0 {
                let gems = match next(rng, 8) {
                    0 => 305,
                    1 => 301,
                    2 => 302,
                    3 => 288,
                    4 => 300,
                    5 => 2351,
                    6 => 2348,
                    _ => 2345,
                };
                out.push(Row::stacked(gems, ranged(rng, 1, 3) as u32));
            }
            if next(rng, 3) == 0 {
                let plants = if next(rng, 2) == 0 { 2350 } else { 4870 };
                out.push(Row::stacked(plants, ranged(rng, 1, 3) as u32));
            }
            if next(rng, 2) == 0 {
                let glowstick = if next(rng, 2) == 0 { 8 } else { 282 };
                out.push(Row::stacked(glowstick, ranged(rng, 16, 31) as u32));
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(73, ranged(rng, 2, 5) as u32));
            }
        }
    }
    // `:37386-37393` — the jungle family's (`flag4`) pair: the 1/4 and
    // 1/50 rows. Both chest-tile blocks carry the same pair (`:37470-47479`),
    // so the rows apply to either anchor kind.
    if ctx.jungle && next(rng, 4) == 0 {
        out.push(Row::plain(2204));
    }
    if ctx.jungle && next(rng, 50) == 0 {
        out.push(Row::plain(753));
    }
    // `:37415-37448` — the skyware family's tail (`flag12`): the Sky Mill at
    // 1/3, one of the six pottery pieces and the Cloud stack 50-101. The
    // pottery row used to ride the `wood_family` approximation at 1/2; the
    // C# gates it on `flag12` (the floating-island houses) unconditionally,
    // and this task re-gates it to `ctx.sky`.
    if ctx.sky && next(rng, 3) == 0 {
        out.push(Row::stacked(2197, 1));
    }
    if ctx.sky {
        let pottery = match next(rng, 6) {
            0 => 5258,
            1 => 5226,
            2 => 5254,
            3 => 5238,
            4 => 5255,
            _ => 5388,
        };
        out.push(Row::stacked(pottery, 1));
        // The Cloud stack is 21-block only (`:37447-37453`) — the 467 block
        // stops at the pottery.
        if chest_kind == 21 {
            out.push(Row::stacked(751, ranged(rng, 50, 101) as u32));
        }
    }
    // `:37400-37407` — the ice family's tail: the Ice Machine at 1/7 and the
    // Fish at 1/50.
    if ctx.ice && next(rng, 7) == 0 {
        out.push(Row::stacked(2198, 1));
    }
    if ctx.ice && next(rng, 50) == 0 {
        out.push(Row::stacked(669, 1));
    }
    // `:37406-47409` — the water family's (`flag13`) row: the unconditional
    // globe (the 21-block only — the 467 block has no such row).
    if ctx.water && chest_kind == 21 {
        out.push(Row::plain(2195));
    }
    // `:47412-47416` — the dungeon family's (`flag9`) row at 1/8 (the
    // `getGoodWorld` 5515 arm at `:47417-47421` parks with M5-SEED-1).
    if ctx.dungeon && chest_kind == 21 && next(rng, 8) == 0 {
        out.push(Row::plain(2192));
    }
    if ctx.wood_family() && next(rng, 2) == 0 {
        out.push(Row::stacked(2198, ranged(rng, 50, 101) as u32));
    }
    // `:37419-37422` — the temple family (`flag13`, chest style 16): the
    // Lihzahrd Furnace unconditionally, then the Solar Tablet at 1/5 or the
    // Lunar Tablet Fragment stack 3-7.
    if chest_style == 16 {
        out.push(Row::stacked(2195, 1));
        if next(rng, 5) == 0 {
            out.push(Row::stacked(2767, 1));
        } else {
            out.push(Row::stacked(2766, ranged(rng, 3, 8) as u32));
        }
    }
    // `:37543-37547` — the voice-change row (1/12): `Item.GetRandomVoiceItem`'s
    // fourteen-item table (`Item.cs:49427-49450`). The vampire-seed monster
    // voice row (`AddMonsterVoiceChangeItemToChest`, `:37549`) has no port
    // seed model and is skipped.
    if next(rng, 12) == 0 {
        let mut voice = Row::plain(voice_change_item(rng));
        roll_prefix_on(rng, &mut voice);
        out.push(voice);
    }
    out
}

/// `Item.GetRandomVoiceItem` (`Item.cs:49427-49450`): the fourteen-item voice
/// table, `Next(14)` picking one (case 0 and the unreachable upper cases fall
/// to 5499).
fn voice_change_item(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) -> u16 {
    match next(rng, 14) {
        1 => 5500,
        2 => 5501,
        3 => 5502,
        4 => 5503,
        5 => 5504,
        6 => 5505,
        7 => 5506,
        8 => 5507,
        9 => 5508,
        10 => 5509,
        11 => 5484,
        12 => 5485,
        13 => 5534,
        _ => 5499,
    }
}

fn silver_bar() -> u16 {
    BAR_SILVER
}

fn iron_bar() -> u16 {
    BAR_IRON
}

fn gold_bar() -> u16 {
    BAR_GOLD
}

fn ore_pick() -> u16 {
    ORE_COPPER
}

/// One anchored chest's stock decided by its surroundings: the fixed
/// special slot (`GetDungeonLootAndChestStyle`'s dungeon cycle, the
/// `GetNextJungleChestItem` cycle, `Pyramid`'s Pharaoh roll, the
/// `flag3`/`flag2`/`flag8`/`IslandHouse` main tables, the underwater
/// override 939), then the depth band rows.
fn stock_chest(
    ev: &tr_core::worldgen::chest_stock::ChestSiteEvent,
    dungeon: &mut DungeonCycle,
    genned_wands: &mut bool,
    hell_chest: &[u16; 5],
    hell_counter: &mut usize,
    rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
) -> Vec<Row> {
    let ctx = ev.ctx;
    let band = ev.band;
    let chest_style = ev.chest_style;
    let chest_kind = ev.chest_tile_type;
    let y = ev.y;
    let mut rows = stock_band(band, &ctx, chest_style, chest_kind, rng);
    if ctx.dungeon {
        // The dungeon-family chests cycle the fixed loot item
        // (`DungeonUtils.cs:342-355`); the shallow caves roll the plain
        // `num10 == 0` skip.
        if let Some(item) = dungeon.take(ev.hell) {
            rows.insert(0, Row::plain(item.0));
        }
    } else if ctx.jungle && ev.main_item != 0 {
        // The jungle family's main item — the `GetNextJungleChestItem`
        // cycle's pick, drawn at the shrine site (`structures.rs`, S1-A-4)
        // and riding `main_item` exactly as the C#'s caller pre-sets
        // `mainItemInChest` (`:17219-17220`). The vanilla water override
        // only fires on `num10 == 0` (`:36201-36203`), so the cycle wins
        // over it wherever both families read true. The vanilla slot-0
        // arm's `chest.item[0].Prefix(-1)` (`:36230`) rolls here.
        let mut row = Row::plain(ev.main_item);
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    } else if ctx.pyramid && ev.main_item != 0 {
        // The `Pyramid` structure's own chest (`:27853-27873`): its main is
        // pre-set by the CALLER (the Pharaoh double-roll, drawn at the site
        // since S1-A-4), so the vanilla's `flag3` arm never fires on it (its
        // `num10 != 0` disjuncts all fail).
        rows.insert(0, Row::plain(ev.main_item));
        if ev.main_item == 848 {
            // `flag11`'s companion (`:36234-36239`) — slot 1, right after the
            // mask's slot 0.
            rows.insert(1, Row::plain(PHARAOHS_ROBE));
        }
    } else if ctx.desert {
        // `flag3`'s underground-desert arm (`:36023-36047`): the shallow
        // four or the deep three by `flag20`'s hive-depth test.
        let deep = deep_desert_chest(ev.height, y);
        rows.insert(0, Row::plain(desert_chest_main(rng, deep)));
    } else if ctx.ice {
        // `flag2`'s main arm (`:36064-36088`): the seven-item table with the
        // 1/20 Extractinator override.
        rows.insert(0, Row::plain(ice_chest_main(rng)));
    } else if ctx.sky && ev.main_item != 0 {
        // `IslandHouse`'s main (`:79344-79363`): the sky-island house count
        // cycle — drawn at the island-house site since S1-A-4 (the counter
        // advances there, `:79394`).
        rows.insert(0, Row::plain(ev.main_item));
    } else if ev.hell {
        // `flag8`'s main arm (`:36115-36118`): the world-shuffled
        // `GenVars.hellChestItem` table, the `GenVars.hellChest` counter
        // advancing after the place (`:36210-36217`).
        rows.insert(0, Row::plain(next_hell_chest(hell_chest, hell_counter)));
    } else if ctx.water {
        // The water family's main (`:36197-36200`): the 939 override, which
        // the vanilla gates on `num10 == 0` — the port keeps this arm last in
        // the chain so the other families' mains win wherever both read
        // true. The non-family natural chests carry no fixed slot here (the
        // vanilla num10 assignment rides the placement-style dispatcher).
        rows.insert(0, Row::plain(UNDERWATER_ITEM));
    }
    // The fixed rows' family extras (`AddBuriedChest`'s two `num10 > 0`
    // arms, the vanilla order: the surface arm's flag6 first `:36236-36257`,
    // then the deep arm's flag2/flag3/flag4 `:36806-36847`); the tenth
    // anniversary's flag11 rows park with M5-SEED-1.
    if band == Band::Surface {
        if ctx.living_tree {
            // `flag6` `:36240-36257`.
            if next(rng, 3) == 0 {
                rows.push(Row::plain(5629));
            }
            if next(rng, 6) == 0 && next(rng, 2) == 0 {
                rows.push(Row::plain(4429));
            } else if next(rng, 6) == 0 {
                rows.push(Row::plain(4427));
            }
            if next(rng, 3) != 0 {
                rows.push(Row::plain(5528));
            }
        }
    } else if band != Band::Deepest {
        // The deep arm (the vanilla `num7 < maxTilesY - 250` branch).
        if ctx.ice {
            // `flag2` `:36809-36813`.
            if next(rng, 5) == 0 {
                rows.push(Row::plain(3199));
            }
        }
        if ctx.mahogany || ctx.desert {
            // `flag3` `:36814-36823`. The vanilla's flag3 family is the
            // underground-desert cave chests and the 467-style-10 mahogany
            // pair; the port's `desert` (hardened-sand shell) and `mahogany`
            // (wood walls) probes carry the two halves. The `Pyramid`
            // structure's chest is NOT a flag3 chest (the caller pre-sets its
            // main), so the pyramid probe stays out of this arm.
            if next(rng, 7) == 0 {
                rows.push(Row::plain(4346));
            }
            if next(rng, 15) == 0 {
                rows.push(Row::plain(4066));
            }
        }
        if ctx.jungle {
            // `flag4` `:36824-36844`: the once-per-world living-wood wands
            // (`GenVars.gennedLivingMahoganyWands` — the sweep's state, the
            // vanilla fairness flag) then the two 1/10 rows.
            if !*genned_wands || next(rng, 5) == 0 {
                *genned_wands = true;
                rows.push(Row::plain(3360));
                rows.push(Row::plain(3361));
            }
            if next(rng, 10) == 0 {
                rows.push(Row::plain(4426));
            }
            if next(rng, 10) == 0 {
                rows.push(Row::plain(5525));
            }
        }
    }
    // The wooden-chest family tails (`:37384-47400`, the
    // `itemIndex > 0 && chestTileType == 21` block — every band): the two
    // style rows and the flag13 water pair. The mahogany-tile 467 block
    // (`:47466-47500`) carries NONE of them — it repeats only the flag4 /
    // flag2 / flag12 rows, which land ungated in `stock_band` — so these
    // three arms gate on the anchor kind. The style rows landed once
    // (2026-11-09, `600dafdb`) and were dropped by a later merge's
    // resolution; restored with the citations intact.
    // `:37453-47455` — the styles 23-27 pottery row at 1/2 (the sweep's
    // `chest_style` off the anchor's `frame_x / 18`).
    if chest_kind == 21 && (23..=27).contains(&chest_style) && next(rng, 2) == 0 {
        rows.push(Row::plain(5234));
    }
    // `:37529-37531` — the skyware style-13 pottery row: a SECOND 1/2 roll
    // on its own (the vanilla keeps the two style rows separate).
    if chest_kind == 21 && chest_style == 13 && next(rng, 2) == 0 {
        rows.push(Row::plain(5234));
    }
    if chest_kind == 21 && ctx.water {
        // `flag13` `:37457-47469` — the 1/5 raincoat, else the 3-7 potions.
        if next(rng, 5) == 0 {
            rows.push(Row::plain(2767));
        } else {
            rows.push(Row::stacked(2766, ranged(rng, 3, 8) as u32));
        }
    }
    rows
}

/// S1-A-4: the gen-time stocking driver — the hook the app hands to
/// `tr_core::worldgen::generate_with_chest_stock`. One per world; owns the
/// once-per-world state the C# keeps in `GenVars`/the stocking statics (the
/// hell table's Reset shuffle, the `flag8` counter, the flag4
/// `gennedLivingMahoganyWands`, the dungeon cycle) and returns the rolled
/// rows per event; the placement site writes them into `World.chests`.
pub(crate) struct ChestStock {
    dungeon: DungeonCycle,
    genned_wands: bool,
    hell_chest: [u16; 5],
    hell_counter: usize,
}

impl ChestStock {
    pub(crate) fn new(seed: u64, remix: bool) -> Self {
        // `WorldGen.Reset`'s hell-chest shuffle (`:11246-11258`) — the
        // Reset-window draws ride `UnifiedRandom(seed)`: the vanilla
        // algorithm and per-world determinism, the sequence position
        // standing in for genRand's Reset state (ANALYSIS).
        let mut gen_rand = tr_core::UnifiedRandom::new(seed as i32);
        Self {
            dungeon: DungeonCycle::new(),
            genned_wands: false,
            hell_chest: shuffle_hell_chest(&mut gen_rand, remix),
            hell_counter: 0,
        }
    }

    /// One event = one stocked chest. The rows become the chest's slots
    /// (in order — the save's implicit-air compression keeps them packed,
    /// `save::ChestRecord`'s shape).
    fn stock(
        &mut self,
        ev: &tr_core::worldgen::chest_stock::ChestSiteEvent,
        rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
    ) -> Vec<ChestItem> {
        let rows = stock_chest(
            ev,
            &mut self.dungeon,
            &mut self.genned_wands,
            &self.hell_chest,
            &mut self.hell_counter,
            rng,
        );
        rows.into_iter()
            .map(|row| ChestItem {
                stack: row.count.max(1) as i32,
                kind: row.kind,
                prefix: row.prefix as u8,
            })
            .collect()
    }
}

/// The hook closure for the generation call: `Some(&mut hook)` into
/// `generate_with_chest_stock` with a [`ChestStock`] captured.
pub(crate) fn chest_stock_hook(
    seed: u64,
    remix: bool,
) -> impl FnMut(
    &tr_core::worldgen::chest_stock::ChestSiteEvent,
    &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
) -> Vec<ChestItem> {
    let mut stock = ChestStock::new(seed, remix);
    move |ev: &tr_core::worldgen::chest_stock::ChestSiteEvent,
          rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand| {
        stock.stock(ev, rng)
    }
}

/// The stocking-arms' test shim — the driver over a synthetic event with the
/// anchor resolved the way the placement sites do (`context_at` + `band_for`
/// + the caller's hell gate). `main_item` rides the site-side rolls (the
/// jungle/sky cycles moved to `GenVars`, S1-A-4).
#[cfg(test)]
fn stock_chest_at(
    stock: &mut ChestStock,
    w: &World,
    x: i32,
    y: i32,
    hell: bool,
    chest_style: i32,
    chest_kind: u16,
    main_item: u16,
    rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
) -> Vec<ChestItem> {
    let ctx = tr_core::worldgen::chest_stock::context_at(w, x, y);
    let ev = tr_core::worldgen::chest_stock::ChestSiteEvent {
        x,
        y,
        chest_style,
        chest_tile_type: chest_kind,
        main_item,
        not_near_other_chests: false,
        hell,
        ctx,
        band: tr_core::worldgen::chest_stock::band_for(y, w.world_surface, w.rock_layer, w.height),
        world_surface: w.world_surface,
        rock_layer: w.rock_layer,
        height: w.height,
    };
    stock.stock(&ev, rng)
}

/// M6-SAVE-1b: the save's v7 chest records (`save::load`, `SaveChests`'s
/// write-side twin `WorldFile.cs:1606-1676`) as a seed store for
/// [`merge_store`] — the load half of `App::save_world`'s record build.
///
/// The anchor cull is the vanilla `LoadChests` validity sweep
/// (`WorldFile.cs:2723-2805`, `FixAgainstExploits`): a record survives only when
/// its `(x, y)` is an active chest tile of kind 21/341 in the loaded world, so
/// a hand-edited or stale save cannot resurrect a chest over plain air.
///
/// Deviation: the vanilla writes a fixed `maxItems` (40) slots per chest; the
/// port keeps the store's implicit-air compression (trailing airs dropped,
/// interior airs explicit) — `save::ChestRecord`'s documented shape, which the
/// open path materializes into the 40-slot window anyway.
pub(crate) fn store_from_records(
    world: &World,
    records: &[ChestRecord],
) -> Vec<crate::chest::WorldChest> {
    let mut out = Vec::with_capacity(records.len());
    for rec in records {
        // `FixAgainstExploits`'s tile read: an out-of-bounds, inactive or
        // non-chest-tile anchor drops the record (the C# nulls the slot).
        let Some(tile) = world.tile(rec.x, rec.y) else {
            continue;
        };
        if !tile.active() || !CHEST_KINDS.contains(&tile.kind) {
            continue;
        }
        let slots = rec
            .slots
            .iter()
            .map(|item| {
                if item.stack <= 0 {
                    EMPTY_SLOT
                } else {
                    ItemSlot {
                        kind: item.kind,
                        // `Item.SetDefaults`'s registry name — the store's
                        // items are registry-named (`chest::set_chest_slot`).
                        name: crate::registry::name(item.kind),
                        count: item.stack as u32,
                        prefix: item.prefix as u16,
                    }
                }
            })
            .collect();
        out.push(WorldChest {
            tile: (rec.x, rec.y),
            slots,
            // M6-SAVE-1 phase 3's fold: the name and the slot count ride the
            // entity (`WorldFile.cs:1641-1642` writes both per chest); the
            // implicit-air store still reads back through the 40-slot window.
            name: rec.name.clone(),
            max_items: rec.slots.len().max(crate::chest::DEFAULT_MAX_ITEMS as usize),
        });
    }
    out
}

/// `TenthAnniversaryWorldInfo`'s good-prefix tables
/// (`WorldGen.cs:3803-3820`) — the static cctor's five arrays.
const GOOD_PREFIX_ACCESSORY: &[u16] = &[65, 68, 72, 76];
const GOOD_PREFIX_MELEE: &[u16] = &[81, 59, 57];
const GOOD_PREFIX_RANGED: &[u16] = &[82, 57, 60];
const GOOD_PREFIX_MAGIC: &[u16] = &[83, 57, 60];
const GOOD_PREFIX_SUMMONER: &[u16] = &[85, 59, 95, 96, 97];

/// `WorldGen.ImproveAllChestContents` (`:24716-24731`) with
/// `GiveItemGoodPrefixes` (`:24733-24754`) and `PrefixItemFromOptions`
/// (`:24760-24778`): the tenth-anniversary world's chest sweep — every stocked
/// item takes one of its class's "good" prefixes when the roll allows it.
///
/// The C# walks `Main.chest[0..8000]` during worldgen
/// (`FinishTenthAnniversaryWorld`, `!getGoodWorldGen`); the port stocks its
/// chests at load time, so the sweep rides the same load-time seam — the
/// documented `chest_gen` deviation.
pub(crate) fn improve_all_chest_contents(chests: &mut [WorldChest], rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) {
    for chest in chests.iter_mut() {
        for slot in chest.slots.iter_mut() {
            if slot.kind == 0 {
                continue;
            }
            give_item_good_prefixes(slot, rng);
        }
    }
}

/// `GiveItemGoodPrefixes` (`:24733-24754`): the accessory / melee / ranged /
/// magic / summon arms in the C# order.
fn give_item_good_prefixes(slot: &mut ItemSlot, rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) {
    let def = match crate::itemdata::ITEMS.get(slot.kind as usize) {
        Some(d) => d,
        None => return,
    };
    if def.accessory {
        prefix_item_from_options(slot, GOOD_PREFIX_ACCESSORY, rng);
    }
    if def.class == crate::registry::DamageClass::Melee {
        prefix_item_from_options(slot, GOOD_PREFIX_MELEE, rng);
    }
    if prefix::in_set(prefix::GUNS_BOWS, slot.kind) {
        prefix_item_from_options(slot, GOOD_PREFIX_RANGED, rng);
    }
    if prefix::in_set(prefix::MAGIC, slot.kind) {
        prefix_item_from_options(slot, GOOD_PREFIX_MAGIC, rng);
    }
    if prefix::in_set(prefix::SUMMON, slot.kind) {
        prefix_item_from_options(slot, GOOD_PREFIX_SUMMONER, rng);
    }
}

/// `PrefixItemFromOptions` (`:24760-24778`): try the option list in the
/// `Next(count)` order — the first prefix the item actually takes wins, and
/// the original prefix is restored when none does.
fn prefix_item_from_options(slot: &mut ItemSlot, options: &[u16], rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) {
    let prefix = slot.prefix;
    // `item.Prefix(-3)` is the `CanHavePrefixes` gate (the port's roll returns
    // 0 for both a rejection and the no-op mode, so the gate reads the
    // itemdata directly).
    if prefix::rollable_prefixes(slot.kind).is_none() {
        return;
    }
    let mut list: Vec<u16> = options.to_vec();
    while !list.is_empty() {
        let index = (next(rng, list.len() as u64)) as usize;
        let num = list[index];
        let rolled = prefix::roll_prefix(slot.kind, num as i32, rng);
        slot.prefix = rolled;
        if rolled == num {
            return;
        }
        list.remove(index);
    }
    slot.prefix = prefix;
}

/// Merges the load-time roll with a pre-seeded store (the `TR_CHEST`
/// fixture, or M6-SAVE-1b's loaded records): the seeded chests keep their
/// slots, the roll's own anchors append after them.
pub(crate) fn merge_store(mut rolled: Vec<crate::chest::WorldChest>, seeded: Vec<crate::chest::WorldChest>) -> Vec<crate::chest::WorldChest> {
    let taken: std::collections::HashSet<(i32, i32)> =
        seeded.iter().map(|c| c.tile).collect();
    rolled.retain(|c| !taken.contains(&c.tile));
    rolled.extend(seeded);
    rolled
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S1-A-4: the gen-time loot is deterministic per seed — two generations
    /// of the same seed through two freshly-built drivers produce identical
    /// `World.chests` records (the placement-order state — the hell counter,
    /// the wands flag, the cycles — advances identically), and the rows are
    /// real content: every stocked record carries the band's rows.
    #[test]
    fn the_gen_time_stock_is_deterministic_per_seed_s1_a_4() {
        let opts = tr_core::worldgen::GenOptions {
            seed: 91,
            ..Default::default()
        };
        let mut a = chest_stock_hook(42, false);
        let mut b = chest_stock_hook(42, false);
        let wa = tr_core::worldgen::generate_with_chest_stock(&opts, Some(&mut a));
        let wb = tr_core::worldgen::generate_with_chest_stock(&opts, Some(&mut b));
        assert!(!wa.chests.is_empty(), "the world stocks its chests");
        assert_eq!(wa.chests.len(), wb.chests.len(), "the same chest count");
        for (ca, cb) in wa.chests.iter().zip(wb.chests.iter()) {
            assert_eq!((ca.x, ca.y), (cb.x, cb.y), "the same anchor");
            assert_eq!(ca.slots, cb.slots, "the same rows");
        }
        assert!(
            wa.chests.iter().all(|c| !c.slots.is_empty()),
            "every record stocks rows"
        );
    }

    /// The test shim: the anchor event the way the placement sites build it
    /// (`context_at` + `band_for` + the hell gate, S1-A-4), stocked through a
    /// [`ChestStock`] on a fresh `UnifiedRandom` stream — the row-content
    /// tests' stand-in for the site wiring.
    fn stock_anchor(
        stock: &mut ChestStock,
        w: &World,
        x: i32,
        y: i32,
        style: i32,
        kind: u16,
        main: u16,
    ) -> Vec<ChestItem> {
        let ctx = tr_core::worldgen::chest_stock::context_at(w, x, y);
        let ev = tr_core::worldgen::chest_stock::ChestSiteEvent {
            x,
            y,
            chest_style: style,
            chest_tile_type: kind,
            main_item: main,
            not_near_other_chests: false,
            hell: y > w.height - 205,
            ctx,
            band: tr_core::worldgen::chest_stock::band_for(y, w.world_surface, w.rock_layer, w.height),
            world_surface: w.world_surface,
            rock_layer: w.rock_layer,
            height: w.height,
        };
        let mut rng = tr_core::UnifiedRandom::new(7);
        stock.stock(&ev, &mut rng)
    }

    /// A world with the floor rows laid at 20 (like the editing test rig).
    fn rig() -> World {
        let mut w = World::new("t".into(), 200, 300);
        w.world_surface = 200.0; // the floor row: y=199 sits inside the surface band
        w.rock_layer = 240.0;
        w.seed = 42;
        for x in 0..200 {
            let i = w.index(x, 200);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w
    }

    fn put_chest(world: &mut World, x: i32, y: i32) {
        for (dx, dy, fx, fy) in [
            (0i32, 0i32, 0i16, 0i16),
            (1, 0, 18, 0),
            (0, 1, 0, 18),
            (1, 1, 18, 18),
        ] {
            let i = world.index(x + dx, y + dy);
            world.tiles[i] = tr_core::tile::Tile {
                kind: 21,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
    }

    /// `ImproveAllChestContents` (`:24716-24731`): the anniversary world's
    /// chest sweep takes the class "good" prefixes — the accessory/melee/
    /// ranged/magic/summon tables — through `PrefixItemFromOptions`'s ordered
    /// try loop. A plain world's prefixes are untouched.
    #[test]
    fn the_anniversary_sweep_takes_the_good_prefixes() {
        // A sword (melee 24 = Wooden Sword) with no prefix: the good table
        // {81, 59, 57} should land one of them (or keep 0 when every try
        // rejects).
        let mut chests = vec![WorldChest {
            tile: (10, 10),
            slots: vec![ItemSlot {
                kind: 24,
                name: "",
                count: 1,
                prefix: 0,
            }],
            name: String::new(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
        }];
        let mut rng = tr_core::UnifiedRandom::new(0x5EED);
        improve_all_chest_contents(&mut chests, &mut rng);
        let good = [81u16, 59, 57];
        assert!(
            chests[0].slots[0].prefix == 0 || good.contains(&chests[0].slots[0].prefix),
            "the melee sweep took a good prefix: {}",
            chests[0].slots[0].prefix
        );
        // Determinism: the same stream replays.
        let mut again = vec![WorldChest {
            tile: (10, 10),
            slots: vec![ItemSlot {
                kind: 24,
                name: "",
                count: 1,
                prefix: 0,
            }],
            name: String::new(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
        }];
        let mut rng2 = tr_core::UnifiedRandom::new(0x5EED);
        improve_all_chest_contents(&mut again, &mut rng2);
        assert_eq!(
            chests[0].slots[0].prefix, again[0].slots[0].prefix,
            "the sweep is deterministic"
        );
    }

    #[test]
    fn test_chest_gen_surface_chest_stocks() {
        // The surface band: the weapon row + the consumable rows.
        let mut w = rig();
        put_chest(&mut w, 40, 199); // the anchor: the floor row is 200
        let mut stock = ChestStock::new(0x5EED, false);
        let rows = &stock_anchor(&mut stock, &w, 40, 199, 0, 21, 0);
        assert!(
            rows.len() >= 2,
            "the surface band carries at least the weapon row"
        );
        // The 1/11 weapon pick: any of the surface-table ids is faithful.
        let weapon = [280, 281, 284, 285, 953, 946, 3068, 3069, 3084, 4341, 6165];
        assert!(
            rows.iter().any(|r| weapon.contains(&r.kind)),
            "a surface weapon row landed: {:?}",
            rows.iter().map(|r| r.kind).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_chest_gen_deep_chest_stocks() {
        // The deep band (below the rock layer): the deep weapon + the gems.
        let mut w = rig();
        put_chest(&mut w, 40, 295); // the anchor: the floor row is 296? use 294
        let mut stock = ChestStock::new(0x5EED, false);
        let rows = stock_anchor(&mut stock, &w, 40, 295, 0, 21, 0);
        assert!(rows.len() >= 2);
    }

    #[test]
    fn test_chest_gen_deterministic() {
        // Two scans of the same world agree row-for-row.
        let mut w = rig();
        put_chest(&mut w, 40, 199);
        put_chest(&mut w, 60, 194);
        put_chest(&mut w, 80, 210);
        // Two identically-built drivers agree row-for-row on the same
        // anchor sequence (the placement order the sites walk).
        let spots = [(40, 199, 0u16), (60, 194, 0), (80, 210, 0)];
        let mut a = ChestStock::new(7, false);
        let mut b = ChestStock::new(7, false);
        for (x, y, main) in spots {
            assert_eq!(stock_anchor(&mut a, &w, x, y, 0, 21, main), stock_anchor(&mut b, &w, x, y, 0, 21, main));
        }
    }

    /// One rolled chest at (x, y) — the generator's own stock.
    fn rolled(x: i32, y: i32, kind: u16) -> crate::chest::WorldChest {
        crate::chest::WorldChest {
            tile: (x, y),
            slots: vec![ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count: 3,
                prefix: 0,
            }],
            name: String::new(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
        }
    }

    /// A file-loaded chest at (x, y) with the player's own contents.
    fn loaded(x: i32, y: i32, name: &str, kind: u16) -> crate::chest::WorldChest {
        crate::chest::WorldChest {
            tile: (x, y),
            slots: vec![ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count: 99,
                prefix: 0,
            }],
            name: name.into(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
        }
    }

    /// DW-3.4 — the `.trwld` path keeps `merge_store`: the file's native
    /// records (the gen-time loot for a saved generated world) + the
    /// `TR_CHEST` fixture. S1-A-4 retired the `.wld` displacement half —
    /// nothing rolls at load, so the file's chests ARE the store
    /// (`LoadChests`, `WorldFile.cs:2723-2805`).
    #[test]
    fn test_DW_3_4_the_trwld_path_keeps_merge_store() {
        let roll = vec![rolled(40, 199, 21), rolled(60, 194, 22)];
        let seeded = vec![loaded(40, 199, "Fixture", 9)];
        let store = merge_store(roll, seeded);
        // The roll's own anchors first (minus the seeded one), the seeded after.
        assert_eq!(store.len(), 2);
        assert_eq!(store[0].tile, (60, 194));
        assert_eq!(store[1].tile, (40, 199));
        assert_eq!(store[1].name, "Fixture");
    }
    #[test]
    fn jungle_chests_stock_the_cycle_item() {
        // A jungle-family anchor (tile 60 near the anchor) stocks the site's
        // cycle pick on slot 0 — the `main_item` the shrine site draws
        // (`GetNextJungleChestItem`, `structures.rs`), with its
        // `Prefix(-1)` roll (`:36230`), not the band weapon.
        let mut w = rig();
        // The jungle grass beside the chest seat (`context_at`'s `near(60)`).
        for (dx, dy) in [(-2i32, 0i32), (-2, -1), (-2, -2)] {
            let i = w.index(100 + dx, 199 + dy);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 60,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        put_chest(&mut w, 100, 198);
        let mut stock = ChestStock::new(42, false);
        let slots = stock_anchor(&mut stock, &w, 100, 198, 0, 21, 212);
        assert_eq!(slots[0].kind, 212, "slot 0 rides the site's cycle pick");
        // The band weapon moved to slot 1 — the insert, not a replacement.
        assert!(slots.len() >= 2, "the band rows follow ({:?})", slots.iter().map(|s| s.kind).collect::<Vec<_>>());
    }

    /// A rig tall enough for a real Deep band (`y < height - 250` past the
    /// rock layer): the 200x300 rig collapses its band to Deepest below the
    /// rock line, where the vanilla deep arm (`num7 < maxTilesY - 250`)
    /// does not run.
    fn deep_rig() -> World {
        let mut w = World::new("t".into(), 200, 600);
        w.world_surface = 200.0;
        w.rock_layer = 300.0;
        w.seed = 42;
        for x in 0..200 {
            for y in 341..344 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        w
    }

    /// The sky probe reads the port's own writer: the floating-island houses
    /// write `WALL_CLOUD` = 73 (`worldgen/islands.rs:26`/`:133`), not the
    /// vanilla 216/187 cloud walls (216 is the underground desert's shell —
    /// `worldgen/desert.rs`).
    #[test]
    fn the_sky_probe_reads_the_cloud_wall() {
        let mut w = rig();
        // A cloud wall near the anchor: the skyware family's tail rows fire.
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(120 + dx, 197 + dy);
                w.tiles[i].wall = 73;
            }
        }
        let ctx = tr_core::worldgen::chest_stock::context_at(&w, 120, 198);
        assert!(ctx.sky, "the cloud wall reads as the skyware family");
        let mut w2 = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w2.index(120 + dx, 197 + dy);
                w2.tiles[i].wall = 216; // the underground desert's shell
            }
        }
        let ctx2 = tr_core::worldgen::chest_stock::context_at(&w2, 120, 198);
        assert!(
            !ctx2.sky,
            "the desert's shell wall is not the skyware family"
        );
        assert!(ctx2.desert, "and it reads as the underground-desert family");
    }

    /// The dungeon probe reads the dungeon walls (7/8/9, the port's own
    /// `biomes.rs:1754`) instead of the old wall-24 stand-in — the port's
    /// dungeon pass never writes 24.
    #[test]
    fn the_dungeon_probe_reads_the_dungeon_walls() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(120 + dx, 197 + dy);
                w.tiles[i].wall = 8;
            }
        }
        assert!(
            tr_core::worldgen::chest_stock::context_at(&w, 120, 198).dungeon,
            "wall 8 reads as the dungeon"
        );
        let mut w2 = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w2.index(120 + dx, 197 + dy);
                w2.tiles[i].wall = 24; // the old stand-in
            }
        }
        assert!(!tr_core::worldgen::chest_stock::context_at(&w2, 120, 198).dungeon, "wall 24 does not");
    }

    /// The anchor gate decodes the chest style off the left cell's
    /// `frame_x / 18` (the port's tile-object writer), so the temple's
    /// style-16 chests are stocked and counted once.
    #[test]
    fn styled_chests_stock_and_decode_their_style() {
        let mut w = rig();
        // A style-16 chest: the left cell at frame_x = 288 (`tileobject.rs`).
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let i = w.index(60 + dx, 197 + dy);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 21,
                frame_x: (288 + dx * 18) as i16,
                frame_y: (dy * 18) as i16,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        let mut stock = ChestStock::new(0x5EED, false);
        let slots = stock_anchor(&mut stock, &w, 60, 197, 16, 21, 0);
        // `flag13` (`:37419-37427`): the Lihzahrd Furnace guaranteed, then the
        // Solar Tablet at 1/5 or the Lunar Tablet stack.
        let kinds: Vec<u16> = slots.iter().map(|s| s.kind).collect();
        assert!(
            kinds.contains(&2195),
            "the temple furnace landed: {kinds:?}"
        );
        assert!(
            kinds.iter().any(|k| *k == 2766 || *k == 2767),
            "a temple tablet landed: {kinds:?}"
        );
    }

    /// The 1/12 voice-change row rolls `Item.GetRandomVoiceItem`'s
    /// fourteen-item table (`Item.cs:49427-49450`) — not a fixed item.
    #[test]
    fn the_voice_change_row_rolls_the_fourteen_item_table() {
        let table = [
            5499u16, 5500, 5501, 5502, 5503, 5504, 5505, 5506, 5507, 5508, 5509, 5484, 5485, 5534,
        ];
        // The row is unconditional per chest up to its 1/12 gate, so many
        // seeds cover the table.
        let mut seen = Vec::new();
        for seed in 1u64..600 {
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            if next(&mut rng, 12) == 0 {
                let item = voice_change_item(&mut rng);
                assert!(table.contains(&item), "item {item} is outside the table");
                if !seen.contains(&item) {
                    seen.push(item);
                }
            }
        }
        assert!(seen.len() >= 10, "the table is covered: {seen:?}");
    }

    /// The ice family's tail rolls the Ice Machine at 1/7 and the Fish at 1/50
    /// (`:37397-37407`) — both land over a seed sweep.
    #[test]
    fn the_ice_tail_rolls_the_machine_and_the_fish() {
        let mut w = deep_rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 340 + dy);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 147,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        put_chest(&mut w, 100, 340);
        let mut machines = 0;
        let mut fish = 0;
        let mut runs = 0;
        for seed in 1u64..900 {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 338, false, 0, 21, 0, &mut rng);
            runs += 1;
            machines += rows.iter().filter(|r| r.kind == 2198).count();
            fish += rows.iter().filter(|r| r.kind == 669).count();
        }
        assert!(
            machines > 0,
            "the Ice Machine row fires ({machines}/{runs})"
        );
        assert!(fish > 0, "the Fish row fires ({fish}/{runs})");
        // The odds: 1/7 and 1/50 — the machine row is the more common one.
        assert!(
            machines > fish,
            "the 1/7 machine row beats the 1/50 fish row ({machines} vs {fish})"
        );
    }

    #[test]
    fn the_family_extras_roll_at_the_vanilla_odds() {
        // The two fixed-row family arms (`AddBuriedChest`'s `num10 > 0`
        // branches): the surface arm's flag6 (living-tree) extras and the
        // deep arm's flag2/flag3/flag4 extras, each at its vanilla odds.
        // The rig is a plain surface natural chest (no family flags), so
        // the arms add nothing.
        let mut w = rig();
        put_chest(&mut w, 100, 198);
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let plain = stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng);
        let family_ids: &[u16] = &[
            5629, 4429, 4427, 5528, 3199, 4346, 4066, 3360, 3361, 4426, 5525,
        ];
        assert!(
            !plain.iter().any(|r| family_ids.contains(&r.kind)),
            "a family-less chest takes no family extras"
        );

        // The ice family (flag2, tiles 147/161/162/197 near the anchor) in
        // the deep band: the 1/5 3199 arm.
        let mut w = deep_rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 340 + dy);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 147,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        put_chest(&mut w, 100, 340);
        // The tail rows added after the deep arm (the ice machine/fish, the
        // voice change) moved the stream, so the 1/5 row's seed is swept.
        let hit = (1u64..500).find(|seed| {
            let mut stock = ChestStock::new(*seed, false);
            let mut rng = tr_core::UnifiedRandom::new(*seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 338, false, 0, 21, 0, &mut rng);
            rows.iter().any(|r| r.kind == 3199)
        });
        assert!(hit.is_some(), "the ice family's 1/5 row fires");
    }

    #[test]
    fn the_living_wood_wands_grant_once_per_world() {
        // The flag4 arm's `GenVars.gennedLivingMahoganyWands` fairness: the
        // first jungle chest grants 3360+3361 unconditionally, later ones
        // only on the 1/5 roll.
        let mut w = World::new("t".into(), 60, 320);
        w.world_surface = 20.0;
        w.rock_layer = 40.0;
        w.seed = 42;
        for x in 0..60 {
            for y in 50..53 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        // The jungle grass beside the anchor (`context_at`'s `near(60)`) in
        // the deep band (y=49: past the rock layer, under `height - 250`).
        for (dx, dy) in [(-2i32, 0i32), (-2, -1)] {
            let i = w.index(30 + dx, 49 + dy);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 60,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        put_chest(&mut w, 30, 49);
        // A seed whose SECOND stock_chest misses the 1/5 wand roll: the
        // fairness gate then keeps the pair granted once for the world.
        let seed = (1u64..400)
            .find(|&s| {
                let wands = |rows: &[ChestItem]| {
                    rows.iter()
                        .filter(|r| r.kind == 3360 || r.kind == 3361)
                        .count()
                };
                let mut stock = ChestStock::new(s, false);
                let mut rng = tr_core::UnifiedRandom::new(s as i32);
                let first = stock_chest_at(&mut stock, &w, 30, 49, false, 0, 21, 0, &mut rng);
                let second = stock_chest_at(&mut stock, &w, 30, 49, false, 0, 21, 0, &mut rng);
                wands(&first) == 2 && wands(&second) == 0
            })
            .expect("a seed whose second call misses the 1/5 roll exists");
        let mut stock = ChestStock::new(seed, false);
        let mut rng = tr_core::UnifiedRandom::new(seed as i32);
        let rows = stock_chest_at(&mut stock, &w, 30, 49, false, 0, 21, 0, &mut rng);
        let wands = rows
            .iter()
            .filter(|r| r.kind == 3360 || r.kind == 3361)
            .count();
        assert_eq!(wands, 2, "both wands row together ({wands})");
        // A second chest with the same driver now needs the 1/5 roll — with
        // this stream it does not fire again, so the pair is granted once.
        let rows2 = stock_chest_at(&mut stock, &w, 30, 49, false, 0, 21, 0, &mut rng);
        let wands2 = rows2
            .iter()
            .filter(|r| r.kind == 3360 || r.kind == 3361)
            .count();
        assert_eq!(
            wands2, 0,
            "the once-per-world grant does not repeat ({wands2})"
        );
    }

    #[test]
    fn the_remaining_tail_rows_roll_at_the_vanilla_odds() {
        // The `itemIndex > 0 && chestTileType == 21` block's remaining rows
        // (`:37384-47500`): the flag4 pair (1/4 + 1/50), the flag13 globe
        // (unconditional), the style-23-27 pottery row (1/2) and the style-13
        // row (a second 1/2), and the flag13 water pair (1/5 raincoat, else
        // 3-7 potions). The flag4 pair and the globe ride the families
        // either tile block carries, so they apply to both anchor kinds.

        let mut stock = ChestStock::new(3, false);

        // A water-family anchor (tile 4869 beside the seat) takes the globe
        // unconditionally and exactly one of the water pair.
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -1i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 4869,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        put_chest(&mut w, 100, 198);
        let mut rng = tr_core::UnifiedRandom::new(3);
        let rows = stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng);
        assert!(
            rows.iter().any(|r| r.kind == 2195),
            "the flag13 globe is unconditional"
        );
        let water: Vec<u16> = rows
            .iter()
            .filter(|r| r.kind == 2766 || r.kind == 2767)
            .map(|r| r.kind)
            .collect();
        assert_eq!(water.len(), 1, "exactly one water row ({water:?})");

        // A jungle-family anchor (tile 60 beside the seat): the flag4 pair
        // fires only on its odds — search a stream where the 1/4 row lands.
        let mut w = rig();
        for (dx, dy) in [(-2i32, 0i32), (-2, -1), (-1, 0)] {
            let i = w.index(100 + dx, 198 + dy);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 60,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        put_chest(&mut w, 100, 198);
        let hits = (1u64..400)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng)
                    .iter()
                    .any(|r| r.kind == 2204)
            })
            .count();
        assert!(hits > 0, "the 1/4 flag4 row fires on some stream ({hits})");
        assert!(hits < 400, "and not on every one — the 1/4 gate is real");

        // A style-13 anchor takes its own 1/2 pottery row.
        let mut w = rig();
        put_chest(&mut w, 100, 198);
        for dy in 0..2 {
            for dx in 0..2 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].frame_x = 18 * 13;
                w.tiles[i].frame_y = 18 * dy as i16;
            }
        }
        let rows13 = (1u64..64).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 198, false, 13, 21, 0, &mut rng)
                .iter()
                .any(|r| r.kind == 5234)
        });
        assert!(rows13, "the style-13 row fires on some stream");
    }

    #[test]
    fn the_21_block_rows_gate_on_the_chest_kind() {
        // The vanilla `itemIndex > 0 && chestTileType == 21` block's rows
        // (`:37384-47400`) apply to the wooden chests only; the mahogany
        // (467/341) block (`:47466-47500`) repeats just the flag4/flag2/
        // flag12 rows. The port gates the 21-only rows on the anchor kind.

        // A dungeon-family anchor: the flag9 row (2192) rolls 1/8.
        let mut w = rig();
        for dx in -2i32..=2 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].wall = 8; // a dungeon wall
            }
        }
        put_chest(&mut w, 100, 198);
        let hits = (1u64..600)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng)
                    .iter()
                .any(|r| r.kind == 2192)
            })
            .count();
        assert!(hits > 0, "the 1/8 flag9 row fires on some stream ({hits})");
        assert!(hits < 600, "and not on every one — the 1/8 gate is real");

        // A kind-341 (mahogany) anchor with a water + sky-25 context takes
        // none of the 21-only rows, while a kind-21 anchor does.
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -1i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].kind = 4869; // the water family's tile
            }
        }
        // The mahogany chest tile 341 at the anchor.
        put_chest(&mut w, 100, 198);
        for dy in 0..2 {
            for dx in 0..2 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].kind = 341;
            }
        }
        let mut stock = ChestStock::new(3, false);
        let mut rng = tr_core::UnifiedRandom::new(3);
        let mahog = stock_chest_at(&mut stock, &w, 100, 198, false, 25, 341, 0, &mut rng);
        assert!(
            !mahog.iter().any(|r| matches!(r.kind, 2766 | 2767 | 5234)),
            "the 21-only rows skip the mahogany anchor: {:?}",
            mahog.iter().map(|r| r.kind).collect::<Vec<_>>()
        );

        // The same surroundings on a kind-21 anchor take them (on some stream).
        let mut w2 = rig();
        for dx in -2i32..=0 {
            for dy in -1i32..=0 {
                let i = w2.index(100 + dx, 198 + dy);
                w2.tiles[i].kind = 4869;
            }
        }
        put_chest(&mut w2, 100, 198);
        let rows21 = (1u64..64).find(|&seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w2, 100, 198, false, 25, 21, 0, &mut rng)
                .iter()
                .any(|r| matches!(r.kind, 2766 | 2767 | 5234))
        });
        assert!(rows21.is_some(), "the 21 anchor takes a 21-only row");
    }

    /// The probe split: the pyramid's own sandstone walls (187) read the
    /// `Pyramid` family, the underground desert's hardened-sand shell (216)
    /// reads the `flag3` desert family — the vanilla's two separate arms
    /// (`:27853` the caller main / `:36023` the `IsUndergroundDesert` arm).
    #[test]
    fn the_pyramid_and_desert_probes_read_their_own_walls() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(120 + dx, 197 + dy);
                w.tiles[i].wall = 187; // the pyramid's sandstone
            }
        }
        let ctx = tr_core::worldgen::chest_stock::context_at(&w, 120, 198);
        assert!(ctx.pyramid, "the sandstone reads the pyramid family");
        assert!(!ctx.desert, "and not the desert one");
    }

    /// `Pyramid`'s chest main (`:27853-27873`): the double-roll, so the
    /// Pharaoh'sMask 848 only on two zeros and the 857/934 otherwise, with
    /// the `flag11` companion 866 in slot 1 right after the mask.
    #[test]
    fn the_pyramid_chest_stocks_the_pharaoh_roll_and_the_robe() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].wall = 187; // the pyramid chamber's sandstone
            }
        }
        put_chest(&mut w, 100, 197);
        // The main roll moved to the site (`structures.rs`, `:27853-27873`);
        // the hook inserts the site's pick and the robe only ever rides a
        // mask chest. All three mains walk the arm.
        let mut mask_with_robe = 0u32;
        let mut pair_without_mask = 0u32;
        for main in [848u16, 857, 934] {
            let mut stock = ChestStock::new(42, false);
            let mut rng = tr_core::UnifiedRandom::new(42);
            let rows = stock_chest_at(&mut stock, &w, 100, 197, false, 0, 21, main, &mut rng);
            assert_eq!(rows[0].kind, main, "slot 0 rides the site's pick ({main})");
            if main == 848 {
                assert_eq!(rows[1].kind, PHARAOHS_ROBE, "the robe rides slot 1");
                mask_with_robe += 1;
            } else {
                assert!(rows.len() < 2 || rows[1].kind != PHARAOHS_ROBE);
                pair_without_mask += 1;
            }
        }
        assert!(mask_with_robe > 0, "the mask+robe pair lands");
        assert!(pair_without_mask > 0, "and the 857/934 arms dominate");
    }

    /// `flag3`'s underground-desert arm (`:36023-36047`): the shallow four
    /// above three sevenths of the height and the deep three below it, the
    /// no-hive `GenVars` split the port keeps at load time.
    #[test]
    fn the_desert_chest_stocks_the_flag3_table_both_depths() {
        let shallow: &[u16] = &DESERT_CHEST_SHALLOW;
        let deep: &[u16] = &DESERT_CHEST_DEEP;
        for (deep_flag, y, table) in [(false, 100i32, shallow), (true, 500i32, deep)] {
            let mut w = World::new("t".into(), 200, 600);
            w.world_surface = 200.0;
            w.rock_layer = 300.0;
            w.seed = 42;
            for x in 0..200 {
                for yy in 341..344 {
                    let i = w.index(x, yy);
                    w.tiles[i] = tr_core::tile::Tile {
                        kind: 1,
                        flags: FLAG_ACTIVE,
                        ..Default::default()
                    };
                }
            }
            for dx in -2i32..=0 {
                for dy in -2i32..=0 {
                    let i = w.index(100 + dx, y as i32 + dy);
                    w.tiles[i].wall = 216; // the hardened-sand shell
                }
            }
            put_chest(&mut w, 100, y as i32);
            let mut stock = ChestStock::new(42, false);
            let mut rng = tr_core::UnifiedRandom::new(42);
            let rows = stock_chest_at(&mut stock, &w, 100, y as i32, false, 0, 21, 0, &mut rng);
            assert_eq!(
                deep_desert_chest(w.height, y as i32),
                deep_flag,
                "the depth split reads the height line"
            );
            assert!(
                table.contains(&rows[0].kind),
                "slot 0 rides the {} table ({})",
                if deep_flag { "deep" } else { "shallow" },
                rows[0].kind
            );
        }
    }

    /// `flag2`'s main arm (`:36064-36088`): the ice family's seven-item
    /// table (or the 1/20 Extractinator) on the ice tiles the port probes.
    #[test]
    fn the_ice_chest_stocks_the_flag2_table() {
        let mut w = rig();
        for (dx, dy) in [(-2i32, 0i32), (-2, -1), (-2, -2)] {
            let i = w.index(100 + dx, 199 + dy);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 147,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        put_chest(&mut w, 100, 198);
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let rows = stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng);
        assert!(
            matches!(
                rows[0].kind,
                670 | 724 | 950 | 1319 | 987 | 1579 | 6153 | 997
            ),
            "slot 0 rides the ice table ({})",
            rows[0].kind
        );
    }

    /// `IslandHouse`'s cycle (`:79338-79363`): the first four sky chests of a
    /// world take the fixed 159/65/158/2219 in order — the vanilla's
    /// `skyIslandHouseCount` table before the `> 3` re-roll.
    #[test]
    fn the_sky_chests_walk_the_island_house_cycle() {
        let mut w = rig();
        for (slot, x) in [40i32, 60, 80, 100].iter().enumerate() {
            for dx in -2i32..=0 {
                for dy in -2i32..=0 {
                    let i = w.index(x + dx, 197 + dy);
                    w.tiles[i].wall = 73; // the island house's cloud walls
                }
            }
            let _ = slot;
            put_chest(&mut w, *x, 199);
        }
        // The cycle itself moved to the site (`islands.rs::build_house`,
        // `GenVars.skyIslandHouseCount`); the hook inserts the site's pick
        // (`main_item`) into slot 0 — the walk order rides the same table.
        let mut stock = ChestStock::new(0x5EED, false);
        let mut stock2 = ChestStock::new(0x5EED, false);
        let mut mains = Vec::new();
        for (x, main) in [(40, 159u16), (60, 65), (80, 158), (100, 2219)] {
            let slots = stock_anchor(&mut stock, &w, x, 199, 13, 21, main);
            mains.push(slots[0].kind);
        }
        assert_eq!(
            mains,
            vec![159, 65, 158, 2219],
            "the four houses walk the vanilla table in order: {mains:?}"
        );
        // The style-13 pottery row (the second 1/2 roll) does not displace
        // the main — the second driver re-runs the walk cleanly.
        for (x, main) in [(40, 159u16), (60, 65), (80, 158), (100, 2219)] {
            let slots = stock_anchor(&mut stock2, &w, x, 199, 13, 21, main);
            assert_eq!(slots[0].kind, main, "the main rides slot 0 ({x})");
        }
    }

    /// `WorldGen.Reset`'s hell-chest shuffle (`:11246-11258`): the five items
    /// come out as a permutation of the table — the same `while (Count > 0)`
    /// pick-and-remove, over the port's stream, once per world.
    #[test]
    fn the_hell_table_shuffles_to_a_permutation() {
        for seed in 1u64..200 {
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let table = shuffle_hell_chest(&mut rng, false);
            let mut sorted = table.to_vec();
            sorted.sort_unstable();
            let mut want = HELL_CHEST.to_vec();
            want.sort_unstable();
            assert_eq!(sorted, want, "the shuffle permutes the table ({seed})");
        }
        let mut rng = tr_core::UnifiedRandom::new(42);
        assert_eq!(
            shuffle_hell_chest(&mut rng, true)
                .iter()
                .filter(|&&k| k == 683)
                .count(),
            1,
            "the remix row swaps the 112 for the 683"
        );
    }

    /// `flag8`'s main arm (`:36115-36118` + `:36210-36217`): the hell-band
    /// chests walk the world's shuffled `hellChestItem` table, the counter
    /// wrapping at its length.
    #[test]
    fn the_hell_chests_walk_the_shuffled_table() {
        let mut w = rig();
        for x in [40i32, 60, 80, 100, 120, 140] {
            put_chest(&mut w, x, 199);
        }
        // The driver's `UnifiedRandom(seed)` shuffle is the walk's source —
        // the same table the hook's counter walks with the wrap.
        let mut stock = ChestStock::new(42, false);
        let table = {
            let mut stream = tr_core::UnifiedRandom::new(42);
            shuffle_hell_chest(&mut stream, false)
        };
        let mut mains = Vec::new();
        for x in [40i32, 60, 80, 100, 120, 140] {
            let slots = stock_anchor(&mut stock, &w, x, 199, 0, 21, 0);
            mains.push(slots[0].kind);
        }
        assert_eq!(
            mains,
            vec![table[0], table[1], table[2], table[3], table[4], table[0]],
            "the six hell chests walk the table with the wrap: {mains:?}"
        );
        for kind in &mains {
            assert!(
                HELL_CHEST.contains(kind),
                "every main is a hell-chest item ({kind})"
            );
        }
    }

    /// The families' mains displace the band's own weapon row on slot 0 —
    /// the vanilla writes `num10` into `chest.item[0]` (`:36229-36232`) and
    /// the band rows follow it.
    #[test]
    fn the_family_mains_ride_slot_zero() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 197 + dy);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 147,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        put_chest(&mut w, 100, 197);
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let rows = stock_chest_at(&mut stock, &w, 100, 197, false, 0, 21, 0, &mut rng);
        assert!(
            !matches!(
                rows[0].kind,
                280 | 281 | 284 | 285 | 953 | 946 | 3068 | 3069 | 3084 | 4341 | 6165
            ),
            "slot 0 is the family main, not the surface weapon ({})",
            rows[0].kind
        );
        assert!(rows.len() > 1, "the band rows still follow it");
    }
}
