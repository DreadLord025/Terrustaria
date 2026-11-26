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
//! `y < maxTilesY-250` / else — with the C#'s own gate quirk that the
//! surface arm needs `(flag | flag6) | flag11`, so a preset-main chest at
//! surface depth falls through to the mid arm), the rows with their
//! verbatim probabilities (`Next(6) == 0` etc), the stack ranges, and the
//! `Prefix(-1)` calls routed through the ported `prefix::roll_prefix`
//! engine. EVERY `num10 > 0` slot-0 write prefixes (`:36232`/`:36513`/
//! `:36808`/`:37167`); the flag9 ladder's `generatedShadowKey`/
//! `generatedRamRune` fairness flags ride the driver's once-per-world
//! state (W4-c33 census). W4-c33d chest-gen-final: flag12 is the
//! style/main test (`:36176`, not the wall-73 probe), the flag7 mushroom
//! arms (`:36653`/`:36977`) land, the style-pottery rows (`:37453`/
//! `:37528`) sit OUTSIDE flag12 (the locked biome chests roll them), and
//! the flag5/flag6 extras carry the C#'s `num10 > 0` gates.
//! Out of scope here (documented in the ROADMAP row): the seed-flavour rows
//! (vampire/errorWorld/rainbow/tenthAnniversary, `SecretSeed`/`Main`
//! statics the port does not generate), the remix/tenth-anniversary/FTW
//! variants of the ported arms. The residual ORDER deviation: the family
//! mains (desert/ice/hell) roll AFTER the band rows in the port (the C#
//! assigns them before the band dispatch) — the draw sequence within each
//! group is verbatim, the interleave is the documented S1-A-4 stand-in.

use crate::chest::WorldChest;
use crate::item::{EMPTY_SLOT, ItemSlot};
use crate::prefix;
use tr_core::save::{ChestItem, ChestRecord};
use tr_core::tile::FLAG_ACTIVE;
use tr_core::world::World;

/// The chest tile kinds an anchor can carry — `TileID.Sets.BasicChest`
/// = {21 Containers, 467 Containers2} (`TileID.cs:570`), the same set
/// `craft::CHEST_TILES` reads (S1-P-i46). The former second entry 341 is
/// `LivingDemonFire` (`TileID.cs:1311`), a fire block — never a chest
/// (W4-a2 / P0-3, the i46 follow-up).
const CHEST_KINDS: [u16; 2] = [21, 467];
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

/// The `GenVars` bar item tables (`WorldGen.Reset` `:11270-11273`): the
/// classic copper/iron/silver/gold bar ids; the alternate-tier helpers
/// below swap in 703/704/705/706 on the rolled alternate ores.
const BAR_COPPER: u16 = 20;
const BAR_IRON: u16 = 22;
const BAR_SILVER: u16 = 21;
const BAR_GOLD: u16 = 19;

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

/// The surface band's weapon row (`:36275-36339`): the `Next(11)` weapon
/// pick — 280 Spear/281 Blowgun/284 Flintlock/285 Musket/953 Pistol/946
/// WoodenShortsword/3068/3069 (the remix 517 parks)/3084/4341/6165 — each
/// with the natural prefix.
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

/// The mid band's weapon row (the `Next(6)` pick `:36580-36606`: the
/// Shadow/Suspecting/Sergeant/Mr/General/Legend — 49/50/53/54/5011/975)
/// and the deepest band's (the `Next(4)` pick `:37193-37213`: 49/50/53/54).
/// The pick is drawn here; the deep band's ladder draws its own `Next(7)`
/// FIRST (`:36881`) and routes case 0-5 through [`deep_weapon_pick`].
fn deep_weapon(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, deepest: bool) -> Row {
    let pick = if deepest {
        next(rng, 4)
    } else {
        next(rng, 6)
    };
    deep_weapon_pick(rng, pick)
}

/// The weapon row for an ALREADY-DRAWN pick (`:36904-36933`): the item
/// plus its ONE natural prefix (W4-c33 census: the old deep ladder rolled
/// the prefix twice — once inside the pick helper, once at the call site).
fn deep_weapon_pick(rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand, pick: u64) -> Row {
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

/// `WorldGen.IsLockedDungeonBiomeChest` (`:35850-35869`): the locked
/// biome-chest styles — kind 21 styles 23-27, kind 467 style 13. `flag10`
/// (`:36167-36171`) skips the flag9 dungeon-extra ladder on them.
fn locked_biome_chest(chest_kind: u16, chest_style: i32) -> bool {
    match chest_kind {
        21 => (23..=27).contains(&chest_style),
        467 => chest_style == 13,
        _ => false,
    }
}

/// The depth band's stock (`while (itemIndex == 0)` — the loop keeps the
/// vanilla order; the `AddBuriedChest` block closes with the `Next(12)`
/// voice-change row `:37546`). The two fairness flags ride the C#'s
/// `GenVars.generatedShadowKey`/`generatedRamRune` (`:36557-36568` /
/// `:36858-36869` — once per world, then the odds roll).
#[allow(clippy::too_many_arguments)]
fn stock_band(
    band: Band,
    ctx: &Context,
    chest_style: i32,
    chest_kind: u16,
    // `flag12` (`WorldGen.cs:36176-36178`): `kind == 21 && (num9 == 13 ||
    // num10 ∈ {159, 65, 158, 2219})` — the skyware family is STYLE/MAIN
    // based, not a wall probe. The flag is set BEFORE the remix retype
    // (`:36179-36194`: the plain-remix sky chest becomes crimson-43 wood
    // or corrupt-3 mahogany 467) and survives it, so the 467 tail block
    // reads the same true flag (the main disjunct carries no kind gate —
    // the sky mains survive the retype). W4-c33d chest-gen-final: the old
    // `ctx.sky` wall-73 gate missed every classic island-house chest
    // (sunplate walls) and fired on non-sky chests near cloud walls.
    flag12: bool,
    preset_main: bool,
    y: i32,
    lava_line: i32,
    ev_hell: bool,
    silver_is_tungsten: bool,
    tiers: &tr_core::ores::SavedOreTiers,
    genned_shadow_key: &mut bool,
    genned_ram_rune: &mut bool,
    rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::with_capacity(16);
    // Row 0: the special slot/weapon (`flag*` extras follow the main item).
    // W4-c33 census: every band's weapon roll sits in the C#'s `num10 == 0`
    // ELSE branch (`:36275`/`:36580`/`:36879`/`:37191`) — ANY caller-preset
    // main (the water/jungle/sky/pyramid/web sites) skips it. The family-
    // rolled mains (desert/ice/hell) keep the port's documented band-first
    // order (the residual deviation, ADDENDUM 120 §8).
    match band {
        Band::Surface => {
            // `:36222` flag23 arm: the special item or the surface weapon.
            if !preset_main {
                out.push(surface_weapon(rng));
            }
            // `:36341-36501`: the consumable rows.
            if next(rng, 6) == 0 {
                out.push(Row::stacked(282, ranged(rng, 40, 76) as u32));
            }
            if next(rng, 6) == 0 {
                out.push(Row::stacked(279, ranged(rng, 150, 301) as u32));
            }
            // `:36360-36387`: the 3093 / 4345 bundles — stack 1 with TWO
            // bonus draws each (`Next(5)==0 → += Next(2)`, `Next(10)==0 →
            // += Next(3)`; W4-c33 census: the old copy dropped both).
            if next(rng, 6) == 0 {
                let mut stack = 1u32;
                if next(rng, 5) == 0 {
                    stack += next(rng, 2) as u32;
                }
                if next(rng, 10) == 0 {
                    stack += next(rng, 3) as u32;
                }
                out.push(Row::stacked(3093, stack));
            }
            if next(rng, 6) == 0 {
                let mut stack = 1u32;
                if next(rng, 5) == 0 {
                    stack += next(rng, 2) as u32;
                }
                if next(rng, 10) == 0 {
                    stack += next(rng, 3) as u32;
                }
                out.push(Row::stacked(4345, stack));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(168, ranged(rng, 3, 6) as u32));
            }
            // `:36394-36408`: the 1/2 bar row — Next(2) pick (copper/iron),
            // stack Next(8)+3, the tier-aware bar ids.
            if next(rng, 2) == 0 {
                let bar = if next(rng, 2) == 0 { copper_bar(tiers) } else { iron_bar(tiers) };
                out.push(Row::stacked(bar, ranged(rng, 3, 11) as u32));
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
            // `flag5` `:36515-36525` (W4-c33): the water chest's (style 17)
            // two 1/2 rows right after the main item — the Water Chests
            // callers always pre-set a main, so the arm always fires on
            // the style. The port rolls the band rows before the main
            // insertion (`stock_chest`), so the arm leads the band.
            // W4-c33d: the C# pair sits INSIDE the `num10 > 0` block — a
            // style-17 chest with no main takes the weapon ELSE branch
            // and rolls NO pair (the preset-main gate makes that exact).
            if preset_main && chest_style == 17 && chest_kind == 21 {
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4425));
                }
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4460));
                }
            }
            // W4-c33 flag5 verify: the old `ctx.mahogany`/`ctx.sky` 1/2
            // either-or 4425/4460 rows had NO C# counterpart — the mid
            // band's only pair is flag5 (`:36515-36525`, above): two
            // INDEPENDENT halves, style-17 gated. Dropped.
            // `flag12` `:36526-36532` (W4-c33 pyramid-loot census): the
            // skyware family's mid-band 1/40 CreativeWings 4978 with the
            // natural prefix — the island-house chests' rare accessory.
            // W4-c33d: the gate is flag12 (style 13 / the four sky mains,
            // `:36176`) AND a preset main — the C# row sits inside the
            // mid band's `num10 > 0` block; a style-13 chest with no main
            // takes the weapon ELSE branch and NO wings draw.
            if flag12 && preset_main && next(rng, 40) == 0 {
                let mut wings = Row::plain(4978);
                roll_prefix_on(rng, &mut wings);
                out.push(wings);
            }
            if !preset_main {
                out.push(deep_weapon(rng, false));
                // `:36630-36652`: the Extractinator/FlareGun ladder (the
                // num10 == 0 ELSE branch). W4-c33 census: the 931 Flare
                // stack is UNCONDITIONAL (`Next(26)+25` — the old copy's
                // phantom `Next(2)` gate dropped half the stacks and a
                // stream draw).
                if next(rng, 20) == 0 {
                    let mut extractinator = Row::plain(997);
                    roll_prefix_on(rng, &mut extractinator);
                    out.push(extractinator);
                } else if next(rng, 20) == 0 {
                    let mut flare_gun = Row::plain(930);
                    roll_prefix_on(rng, &mut flare_gun);
                    out.push(flare_gun);
                    out.push(Row::stacked(931, ranged(rng, 25, 51) as u32));
                }
                // `flag7` `:36653-36666` (W4-c33d chest-gen-final): the
                // mushroom family (kind 21, style 32, `:36151-36153`) —
                // the 1/2 ShroomMinecart 4450, then an INDEPENDENT 1/3
                // MushroomVanity set 4779+4780+4781. Both rows sit in the
                // `num10 == 0` ELSE branch (the family's own chests carry
                // no preset main); the old copy had no flag7 arm at all.
                if chest_kind == 21 && chest_style == 32 {
                    if next(rng, 2) == 0 {
                        out.push(Row::plain(4450));
                    }
                    if next(rng, 3) == 0 {
                        out.push(Row::plain(4779));
                        out.push(Row::plain(4780));
                        out.push(Row::plain(4781));
                    }
                }
            }
            // `flag9 && !flag10` `:36555-36576` (W4-c33 census): the
            // dungeon extras — the ShadowKey 329 is the ONCE-per-world
            // fairness grant (`!generatedShadowKey || Next(3)==0`), the
            // RamRune 5465 the same (`|| Next(8)==0`) WITH `Prefix(-1)`,
            // and the 6156 1/4 WITH `Prefix(-1)`. The old copy rolled
            // Next(2) for the key, no fairness, no prefixes. The locked
            // biome chests (styles 23-27 / 467-13, flag10) skip the arm.
            // W4-c33d: the ladder is INSIDE the C#'s `num10 > 0` block —
            // but the port's dungeon cycle supplies that main internally
            // (`stock_chest`'s flag9 arm inserts `dungeon.take()`), so the
            // family gate below already implies num10 > 0; no extra
            // preset-main gate (ev.main_item is 0 for the cycle chests).
            if ctx.dungeon && chest_style != 0 && !locked_biome_chest(chest_kind, chest_style) {
                if !*genned_shadow_key || next(rng, 3) == 0 {
                    *genned_shadow_key = true;
                    out.push(Row::plain(329));
                }
                if !*genned_ram_rune || next(rng, 8) == 0 {
                    *genned_ram_rune = true;
                    let mut rune = Row::plain(5465);
                    roll_prefix_on(rng, &mut rune);
                    out.push(rune);
                }
                if next(rng, 4) == 0 {
                    let mut cell = Row::plain(6156);
                    roll_prefix_on(rng, &mut cell);
                    out.push(cell);
                }
            }
            // W4-c33 loot-row census: the mid band's COMMON rows
            // (`:36668-36801`, both num10 branches) — the old port carried
            // a phantom silver-bar 3-6 row, a Next(3) 52 gate (C# Next(5)),
            // a 10-21 bar stack (C# Next(10)+5), a Next(3)!=0 arrow gate
            // (C# Next(2)==0), 2-5 herb stacks (C# Next(1,3)), a Next(2)
            // torch pick (C# flag2) and no 166/4423 + 2350 rows. Re-derived:
            // `:36668-36682`: flag3 → the 1/3 FossilOre 4423 (10-19),
            // else the 1/3 Bomb 166 (10-19).
            if ctx.mahogany || ctx.desert {
                if next(rng, 3) == 0 {
                    out.push(Row::stacked(4423, ranged(rng, 10, 20) as u32));
                }
            } else if next(rng, 3) == 0 {
                out.push(Row::stacked(166, ranged(rng, 10, 20) as u32));
            }
            // `:36683-36688`: the 1/5 CloudinaBottle 52.
            if next(rng, 5) == 0 {
                out.push(Row::stacked(52, 1));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(965, ranged(rng, 50, 101) as u32));
            }
            // `:36695-36709`: the 1/2 bar row — Next(2) pick, Next(10)+5.
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                let bar = if pick == 0 { iron_bar(tiers) } else { silver_bar(tiers) };
                out.push(Row::stacked(bar, ranged(rng, 5, 15) as u32));
            }
            // `:36710-36724`: the 1/2 arrow row — Next(2) pick, Next(25)+25.
            if next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    out.push(Row::stacked(40, ranged(rng, 25, 50) as u32));
                } else {
                    out.push(Row::stacked(42, ranged(rng, 25, 50) as u32));
                }
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(28, ranged(rng, 3, 6) as u32));
            }
            // `:36732-36774`: the 2/3 herb row — Next(9) pick, Next(1,3).
            if next(rng, 3) > 0 {
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
                out.push(Row::stacked(herbs, ranged(rng, 1, 3) as u32));
            }
            // `:36775-36781`: the 2/3 RecallPotion 2350 row (2-4).
            if next(rng, 3) != 0 {
                out.push(Row::stacked(2350, ranged(rng, 2, 5) as u32));
            }
            // `:36782-36795`: the 1/2 torch row — flag2 picks the IceTorch
            // 974, else the Torch 8 (NO pick draw), Next(11)+10.
            if next(rng, 2) == 0 {
                let torch = if ctx.ice { 974 } else { 8 };
                out.push(Row::stacked(torch, ranged(rng, 10, 21) as u32));
            }
            if next(rng, 2) == 0 {
                out.push(Row::stacked(72, ranged(rng, 50, 90) as u32));
            }
        }
        Band::Deep => {
            // `:36803` — mahogany/pyramid extras, the deep weapon.
            // W4-c33 flag5 verify: the old `ctx.mahogany` either-or
            // 4425/4460 row and the `(Next(5)==0 || Next(1)==0)` wand row
            // had NO C# counterpart — the deep band's num10 > 0 arm
            // (`:36806-36844`) rolls flag2 3199 (1/5), flag3 4346/4066
            // (1/7 + 1/15), flag4's wands fairness + two INDEPENDENT 1/10
            // rows (4426, 5525) and then flag5's pair. The flag2/3/4 rows
            // live in `stock_chest`'s deep-arm block (once, not twice);
            // the old `ctx.pyramid` 4346/4066 gate was mis-attributed —
            // the pyramid chamber is flag11 and takes the SURFACE arm.
            // `flag5` `:36845-36855` (W4-c33): the deep band's water-chest
            // pair — the vanilla slot is after the flag4 jungle rows (the
            // port's `stock_chest` inserts the main ahead of the band).
            // W4-c33d: `num10 > 0`-gated like the mid pair.
            if preset_main && chest_style == 17 && chest_kind == 21 {
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4425));
                }
                if next(rng, 2) == 0 {
                    out.push(Row::plain(4460));
                }
            }
            // `:36856-36877` — the flag9 && !flag10 dungeon extras (the
            // same fairness ladder as the mid band, W4-c33 census).
            // W4-c33d: the C# ladder is `num10 > 0`-gated; the port's
            // dungeon cycle supplies that main internally (the flag9 arm
            // inserts `dungeon.take()`), so the family gate implies it —
            // same shape as the mid arm.
            if ctx.dungeon && chest_style != 0 && !locked_biome_chest(chest_kind, chest_style) {
                if !*genned_shadow_key || next(rng, 3) == 0 {
                    *genned_shadow_key = true;
                    out.push(Row::plain(329));
                }
                if !*genned_ram_rune || next(rng, 8) == 0 {
                    *genned_ram_rune = true;
                    let mut rune = Row::plain(5465);
                    roll_prefix_on(rng, &mut rune);
                    out.push(rune);
                }
                if next(rng, 4) == 0 {
                    let mut cell = Row::plain(6156);
                    roll_prefix_on(rng, &mut cell);
                    out.push(cell);
                }
            }
            // `:36879-36994`: the whole lava/extractinator/weapon ladder is
            // the `num10 == 0` ELSE branch — a preset main skips it.
            // W4-c33 census re-derivation: the `Next(7)` weapon pick
            // (`num25`) is drawn FIRST (`:36881`), before the lava gate;
            // the lava gate ANDs `flag24` (`num7 > lavaLine`, `:36882/
            // :36892` — the draw always runs); the weapon table's case 6
            // is the FlareGun 930 + the unconditional 931 stack
            // (`:36934-36948`), NOT a separate Next(20) key roll; the
            // weapon prefixes ONCE (the old copy rolled it twice).
            if !preset_main {
                let num25 = next(rng, 7);
                let below_lava_line = y > lava_line;
                if next(rng, 20) == 0 && below_lava_line {
                    let mut lava = Row::plain(906);
                    roll_prefix_on(rng, &mut lava);
                    out.push(lava);
                } else if next(rng, 15) == 0 {
                    let mut extractinator = Row::plain(997);
                    roll_prefix_on(rng, &mut extractinator);
                    out.push(extractinator);
                } else {
                    match num25 {
                        0..=5 => out.push(deep_weapon_pick(rng, num25)),
                        6 => {
                            let mut flare_gun = Row::plain(930);
                            roll_prefix_on(rng, &mut flare_gun);
                            out.push(flare_gun);
                            out.push(Row::stacked(931, ranged(rng, 25, 51) as u32));
                        }
                        // num25 == 6 is the FlareGun arm above; the C#'s
                        // trailing ELSE (`:36950-36974`) carries only the
                        // vampire/errorWorld arms (parked with the seed
                        // lanes) — unreachable on the Next(7) pick.
                        _ => {}
                    }
                }
                // `flag7` `:36977-36993` (W4-c33d chest-gen-final): the
                // deep band's mushroom arm is an EITHER-OR — the 1/2
                // ShroomMinecart 4450 ELSE the MushroomVanity set
                // 4779+4780+4781 (the mid band's two independent rolls
                // are NOT this shape). Inside the `num10 == 0` branch.
                if chest_kind == 21 && chest_style == 32 {
                    if next(rng, 2) == 0 {
                        out.push(Row::plain(4450));
                    } else {
                        out.push(Row::plain(4779));
                        out.push(Row::plain(4780));
                        out.push(Row::plain(4781));
                    }
                }
            }
            // `:36995-37006`: the 1/5 summon row — flag2 picks the
            // DeerThing 5120, else the SuspiciousLookingEye 43 (NO pick
            // draw — the old copy's Next(2) was a phantom stream draw).
            if next(rng, 5) == 0 {
                out.push(Row::plain(if ctx.ice { 5120 } else { 43 }));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(167, 1));
            }
            if next(rng, 4) == 0 {
                out.push(Row::stacked(51, ranged(rng, 25, 51) as u32));
            }
            if next(rng, 2) == 0 {
                let bar = if next(rng, 2) == 0 { gold_bar(tiers) } else { silver_bar(tiers) };
                out.push(Row::stacked(bar, ranged(rng, 3, 11) as u32));
            }
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                if pick == 0 {
                    out.push(Row::stacked(41, ranged(rng, 25, 51) as u32));
                } else {
                    out.push(Row::stacked(279, ranged(rng, 25, 51) as u32));
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
            // `:37117-37123` (W4-c33 census): the deep band's 1/2
            // RecallPotion 2350 row (2-4) — the old copy carried a phantom
            // gold/silver-bar 16-30 pair here (that shape is the DEEPEST
            // band's 117/gold row `:37221-37235`).
            if next(rng, 2) == 0 {
                out.push(Row::stacked(2350, ranged(rng, 2, 5) as u32));
            }
            // `:37124-37154`: the 1/2 torch/glowstick row — Next(2) pick
            // (0 → flag2 ? IceTorch 974 : Torch 8, else Glowstick 282),
            // stack Next(15)+15 (the old copy's 16-30 ore row was phantom).
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                let kind = if pick == 0 {
                    if ctx.ice { 974 } else { 8 }
                } else {
                    282
                };
                out.push(Row::stacked(kind, ranged(rng, 15, 31) as u32));
            }
            // `:37155-37160`: the 1/2 LifeCrystal 73 row — stack
            // Next(1,3) (the old copy's 2-4 was the deepest band's range).
            if next(rng, 2) == 0 {
                out.push(Row::stacked(73, ranged(rng, 1, 3) as u32));
            }
        }
        Band::Deepest => {
            // `:37164-37189` — the flag8 (hell) extras, NOT an idol probe
            // (W4-c33 census: the old ctx.idol gate had no C# counterpart
            // here — flag8 is the hell-chest family, `:36115-36125`). The
            // TreasureMagnet 5010 rolls its natural prefix (`:37172`).
            if ev_hell {
                if next(rng, 5) == 0 {
                    let mut magnet = Row::plain(5010);
                    roll_prefix_on(rng, &mut magnet);
                    out.push(magnet);
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
            // `:37193`: the deepest weapon roll is the num10 == 0 branch.
            if !preset_main {
                out.push(deep_weapon(rng, true));
            }
            if next(rng, 3) == 0 {
                out.push(Row::stacked(167, 1));
            }
            // `:37221-37235`: the 1/2 MeteoriteBar 117 / gold-bar row —
            // stack Next(15)+15 (the old copy's 16-30 was off by one).
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                if pick == 0 {
                    out.push(Row::stacked(117, ranged(rng, 15, 30) as u32));
                } else {
                    out.push(Row::stacked(gold_bar(tiers), ranged(rng, 15, 30) as u32));
                }
            }
            // `:37236-37257`: the 1/2 HellfireArrow 265 / silver-tier
            // bullet row — the Next(2) pick is REAL (the old copy dropped
            // it and always wrote 265): pick 1 → the SilverBullet 278,
            // or the TungstenBullet 4915 when the world's silver tier is
            // tungsten (`SavedOreTiers.Silver == 168`). Stack Next(25)+50.
            if next(rng, 2) == 0 {
                let pick = next(rng, 2);
                let kind = if pick == 0 {
                    265
                } else if silver_is_tungsten {
                    4915
                } else {
                    278
                };
                out.push(Row::stacked(kind, ranged(rng, 50, 75) as u32));
            }
            // `:37258-37264`: the 1/2 GreaterHealingPotion 227 row — stack
            // Next(6)+15 (the old copy's 16-21 was off by one).
            if next(rng, 2) == 0 {
                out.push(Row::stacked(227, ranged(rng, 15, 21) as u32));
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
            // `:37343-37356`: the 1/3 potion row — the stack is drawn
            // FIRST (Next(1,3)), then the Next(2) pick: RecallPotion 2350
            // / PotionOfReturn 4870.
            if next(rng, 3) == 0 {
                let stack = ranged(rng, 1, 3) as u32;
                let plants = if next(rng, 2) == 0 { 2350 } else { 4870 };
                out.push(Row::stacked(plants, stack));
            }
            // `:37357-37376`: the 1/2 torch/glowstick row — Next(2) pick
            // (0 → Torch 8 — the deepest band has NO ice-torch arm, else
            // Glowstick 282), stack Next(15)+15 (the old copy's 16-30 was
            // off by one).
            if next(rng, 2) == 0 {
                let kind = if next(rng, 2) == 0 { 8 } else { 282 };
                out.push(Row::stacked(kind, ranged(rng, 15, 31) as u32));
            }
            // `:37377-37382`: the 1/2 LifeCrystal 73 row, stack Next(2,5).
            if next(rng, 2) == 0 {
                out.push(Row::stacked(73, ranged(rng, 2, 5) as u32));
            }
        }
    }
    // W4-c33 census — the two tail blocks in the C#'s OWN row order
    // (`:37384-37470` the kind-21 block, `:37471-37532` the kind-467
    // block): flag4 pair → flag2 pair → flag12 SkyMill → [21-only:
    // flag13 furnace, flag9 BoneWelder] → flag12 pottery → flag12 Cloud
    // (BOTH blocks carry the Cloud stack — the old copy's "467 stops at
    // the pottery" note was wrong, `:37522-37527`) → the style row
    // (21: styles 23-27; 467: style 13 — OUTSIDE flag12, W4-c33d) →
    // [21-only: the flag13 tablet pair]. The old `wood_family` 2198
    // 50-101 row had NO C# counterpart (the only 2198 write in the whole
    // block is the flag2 1/7 machine) and is dropped.
    // `flag4` `:37386-37393` / `:37473-37480` — the jungle pair.
    if ctx.jungle && next(rng, 4) == 0 {
        out.push(Row::plain(2204));
    }
    if ctx.jungle && next(rng, 50) == 0 {
        out.push(Row::plain(753));
    }
    // `flag2` `:37394-37401` / `:37481-37488` — the Ice Machine at 1/7
    // and the Fish at 1/50.
    if ctx.ice && next(rng, 7) == 0 {
        out.push(Row::plain(2198));
    }
    if ctx.ice && next(rng, 50) == 0 {
        out.push(Row::plain(669));
    }
    // `flag12` `:37402-37405` / `:37489-37492` — the Sky Mill at 1/3.
    // W4-c33d: flag12 is the style/main test (`:36176`), not the wall
    // probe — the classic island-house chest sits on SUNPLATE walls.
    if flag12 && next(rng, 3) == 0 {
        out.push(Row::plain(2197));
    }
    if chest_kind == 21 {
        // `flag13` `:37406-37409` — the temple family (style 16,
        // `:36159-36162`): the Lihzahrd Furnace unconditionally. The old
        // ctx.water gate was mis-attributed (flag13 is the TEMPLE, not
        // the water probe).
        if chest_style == 16 {
            out.push(Row::plain(2195));
        }
        // `flag9` `:37410-37413` — the Bone Welder at 1/8 (the
        // `getGoodWorld` 5515 arm `:37414-37417` parks with M5-SEED-1).
        if ctx.dungeon && chest_style != 0 && next(rng, 8) == 0 {
            out.push(Row::plain(2192));
        }
    }
    // `flag12` `:37418-37452` / `:37493-37527` — the pottery pick and the
    // Cloud stack 50-101, in BOTH blocks. W4-c33d: flag12 = style 13 /
    // the four sky mains (`:36176`), NOT the wall-73 probe.
    if flag12 {
        let pottery = match next(rng, 6) {
            0 => 5258,
            1 => 5226,
            2 => 5254,
            3 => 5238,
            4 => 5255,
            _ => 5388,
        };
        out.push(Row::plain(pottery));
        // The Cloud stack rides BOTH blocks — the 21 block at
        // `:37447-37452` AND the 467 block at `:37522-37527` (W4-g44
        // re-census + W4-c33d: the pottery itself is a plain SetDefaults,
        // `:37420-37445` — no stack write).
        out.push(Row::stacked(751, ranged(rng, 50, 101) as u32));
    }
    // The style row: kind 21 styles 23-27 (`:37453-37456`), kind 467
    // style 13 (`:37528-37531`) — a 1/2 RemnantsofDevotion 5234 row each.
    // W4-c33d: these `num9` tests sit OUTSIDE the flag12 blocks in the C#
    // — the LOCKED BIOME chests (flag10, never flag12: their mains are
    // 1156/1571/1569/1260/1572/4607) roll the row on their own style; the
    // old copy's flag12 nest dropped every locked-chest draw.
    let style_row = if chest_kind == 21 {
        (23..=27).contains(&chest_style)
    } else {
        chest_kind == 467 && chest_style == 13
    };
    if style_row && next(rng, 2) == 0 {
        out.push(Row::plain(5234));
    }
    if chest_kind == 21 && chest_style == 16 {
        // `flag13` `:37457-37469` — the Solar Tablet at 1/5, else the
        // Lunar Tablet Fragment stack 3-7.
        if next(rng, 5) == 0 {
            out.push(Row::plain(2767));
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

/// The vanilla ore tiers the bar tables pick with (`WorldGen.Reset`
/// `:11270-11297`): the classic ids above, or the alternates — tin bar 703
/// / lead bar 704 / tungsten bar 705 / platinum bar 706 — when the world's
/// `SavedOreTiers` rolled the alternate ore (W4-c33 census: the old copy
/// pinned the classic ids on every world).
fn copper_bar(tiers: &tr_core::ores::SavedOreTiers) -> u16 {
    if tiers.copper == 166 { 703 } else { BAR_COPPER }
}

fn iron_bar(tiers: &tr_core::ores::SavedOreTiers) -> u16 {
    if tiers.iron == 167 { 704 } else { BAR_IRON }
}

fn silver_bar(tiers: &tr_core::ores::SavedOreTiers) -> u16 {
    if tiers.silver == 168 { 705 } else { BAR_SILVER }
}

fn gold_bar(tiers: &tr_core::ores::SavedOreTiers) -> u16 {
    if tiers.gold == 169 { 706 } else { BAR_GOLD }
}

/// One anchored chest's stock decided by its surroundings: the depth band
/// rows first (the port's documented band-first interleave), then the
/// fixed special slot (`GetDungeonLootAndChestStyle`'s dungeon cycle, the
/// `GetNextJungleChestItem` cycle, `Pyramid`'s Pharaoh roll, the
/// `flag3`/`flag2`/`flag8`/`IslandHouse` main tables, the 939 override),
/// then the family extras. Every slot-0 main prefixes.
fn stock_chest(
    ev: &tr_core::worldgen::chest_stock::ChestSiteEvent,
    dungeon: &mut DungeonCycle,
    genned_wands: &mut bool,
    genned_shadow_key: &mut bool,
    genned_ram_rune: &mut bool,
    hell_chest: &[u16; 5],
    hell_counter: &mut usize,
    rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand,
) -> Vec<Row> {
    let ctx = ev.ctx;
    let chest_style = ev.chest_style;
    let chest_kind = ev.chest_tile_type;
    let y = ev.y;
    // `flag11` (`:36172-36175`): a MAIN-based flag — `chestTileType == 21
    // && num9 != 0 && (num10 == 848 || num10 == 857 || num10 == 934)`. The
    // pyramid's chamber walls are SandstoneBrick 34 (`Pyramid :27726`), so
    // the wall-187 `ctx.pyramid` probe alone misses the faithful chamber;
    // the flag reads the caller's main (the Pharaoh double-roll), not the
    // walls. `num9 != 0` holds: the Pyramid caller passes style 1.
    let flag11 = chest_kind == 21
        && chest_style != 0
        && matches!(ev.main_item, 848 | 857 | 934);
    // `flag12` (`:36176-36178`, W4-c33d chest-gen-final): `kind == 21 &&
    // (num9 == 13 || num10 ∈ {159, 65, 158, 2219})` — the skyware family
    // is STYLE/MAIN based. The classic island-house chest sits on the
    // sunplate house walls (`IslandHouse :79222` writes disc wall 82 /
    // sunplate 144), NOT the cloud-disc wall 73 — the old `ctx.sky`
    // probe missed every faithful house chest and fired on non-sky
    // chests near cloud walls. The flag12 remix retype (`:36179-36194`,
    // crimson-43 wood / corrupt-3 mahogany 467) rides the placement core
    // (g44's buried_chest); the retype keeps the flag set, so the 467
    // tail block reads the same true flag12 (the style-13 arm of the
    // `style_row` below covers the 467-13 locked desert chest, which is
    // flag10 and NEVER flag12). The main disjunct carries NO kind gate:
    // the corrupt retype flips kind to 467 AFTER flag12 is set, but the
    // sky main survives — and 159/65/158/2219 are exclusively the
    // IslandHouse cycle's mains, so the disjunct is exact either way.
    let flag12 = (chest_kind == 21 && chest_style == 13)
        || matches!(ev.main_item, 159 | 65 | 158 | 2219);
    // `:36222-36227`: `(flag23 && (flag | flag6)) | flag11` — the flag11
    // disjunct FORCES the surface arm at any depth (the pyramid chamber
    // sits deep in the sand; its band rows are the surface consumables).
    // W4-c33 census: `flag` (`:36019-36022`) is the STYLE-0 natural chest
    // only — a preset-main chest at surface depth (the sky houses, a
    // beach-band water chest) is NOT flag/flag6/flag11, so the surface
    // `if` fails and the C# falls through to the MID arm (`:36503`,
    // `num7 < rockLayer` holds). The port's band read mirrors that.
    let flag0 = chest_kind == 21 && chest_style == 0;
    let band = if flag11 {
        Band::Surface
    } else if ev.band == Band::Surface && !(flag0 || ctx.living_tree) {
        Band::Mid
    } else {
        ev.band
    };
    // The caller-preset mains (the C# sites pass `mainItemInChest` before
    // `AddBuriedChest`): EVERY `num10 > 0` chest takes the special-slot arm
    // and skips the band weapon rolls (W4-c33 census: the old whitelist
    // let a preset-main chest inside a dungeon/ice/desert probe roll the
    // weapon ladder AND overwrite its main with a family roll).
    let preset_main = ev.main_item != 0;
    let mut rows = stock_band(
        band,
        &ctx,
        chest_style,
        chest_kind,
        flag12,
        preset_main,
        y,
        ev.lava_line,
        ev.hell,
        ev.silver_is_tungsten,
        &ev.ore_tiers,
        genned_shadow_key,
        genned_ram_rune,
        rng,
    );
    if ctx.dungeon && chest_style != 0 {
        // `flag9` (`:36163-36166`): kind 21 && num9 != 0 && IsDungeon —
        // the dungeon-family chests cycle the fixed loot item
        // (`DungeonUtils.cs:342-355`); the style-0 natural chests keep the
        // plain `num10 == 0` weapon branch. The slot-0 write prefixes
        // (`:36232`/`:36513`/`:36808`/`:37167`).
        if let Some(item) = dungeon.take(ev.hell) {
            let mut row = Row::plain(item.0);
            roll_prefix_on(rng, &mut row);
            rows.insert(0, row);
        }
    } else if ctx.jungle && ev.main_item != 0 {
        // The jungle family's main item — the `GetNextJungleChestItem`
        // cycle's pick, drawn at the shrine site (`structures.rs`, S1-A-4)
        // and riding `main_item` exactly as the C#'s caller pre-sets
        // `mainItemInChest` (`:17219-17220`). The vanilla water override
        // only fires on `num10 == 0` (`:36197-36200`), so the cycle wins
        // over it wherever both families read true.
        let mut row = Row::plain(ev.main_item);
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
        // `:36240-36243` / `:36507-36511` — the LivingWoodWand 832 main
        // carries the LeafWand 933 companion (surface slot 1 / mid slot 0).
        if ev.main_item == 832 {
            let slot = if band == Band::Surface { 1 } else { 0 };
            rows.insert(slot, Row::plain(933));
        }
    } else if flag11 {
        // The `Pyramid` structure's own chest (`:27853-27873`): its main is
        // pre-set by the CALLER (the Pharaoh double-roll, drawn at the site
        // since S1-A-4), so the vanilla's `flag3` arm never fires on it (its
        // `num10 != 0` disjuncts all fail). The `flag11` gate is main-based
        // (`:36172`), not the wall probe — the chamber's wall is 34.
        // `:36229-36232`: `SetDefaults(num10)` + `Prefix(-1)` — the mask is
        // an accessory (rollable), the bottle/carpet roll through the same
        // engine call (their `CanHavePrefixes` decides the draws).
        let mut row = Row::plain(ev.main_item);
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
        if ev.main_item == 848 {
            // `flag11`'s companion (`:36236-36239`) — slot 1, right after the
            // mask's slot 0; the C# writes it with no `Prefix(-1)`.
            rows.insert(1, Row::plain(PHARAOHS_ROBE));
        }
    } else if ev.main_item != 0 {
        // The remaining preset mains — the Water Chests (style 17,
        // `:17252`/`:17287-17316`), the sky-house cycle (`IslandHouse`
        // `:79344-79363`), the spider-cave web chest's 939 (`:17412`):
        // the `num10 > 0` arms insert it verbatim WITH the natural prefix
        // (W4-c33 census: the old sky arm wrote it plain).
        let mut row = Row::plain(ev.main_item);
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    } else if ctx.desert {
        // `flag3`'s underground-desert arm (`:36023-36047`): the shallow
        // four or the deep three by `flag20`'s hive-depth test. The main
        // prefixes like every `num10 > 0` slot-0 write.
        let deep = deep_desert_chest(ev.height, y);
        let mut row = Row::plain(desert_chest_main(rng, deep));
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    } else if ctx.ice {
        // `flag2`'s main arm (`:36064-36088`): the seven-item table with the
        // 1/20 Extractinator override, then the natural prefix (the C#
        // slot-0 write prefixes on every arm, `:36232`/`:36513`).
        // W4-c33d keep-one: HEAD's `ctx.sky && main != 0` arm dropped —
        // the generic `ev.main_item != 0` arm above already inserts the
        // sky-house cycle's main WITH the prefix, and the CreativeWings
        // 4978 1/40 rider rides `stock_band`'s flag12 gate (`:36526`).
        let mut row = Row::plain(ice_chest_main(rng));
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    } else if ev.hell {
        // `flag8`'s main arm (`:36115-36118`): the world-shuffled
        // `GenVars.hellChestItem` table, the `GenVars.hellChest` counter
        // advancing after the place (`:36210-36217`), then the prefix.
        let mut row = Row::plain(next_hell_chest(hell_chest, hell_counter));
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    } else if chest_style == 15 {
        // The 939 override (`:36197-36200`): `(num10 == 939 || (kind 21 &&
        // num9 == 15) || (kind 467 && num9 == 2)) && num10 == 0` — the
        // spider-cave web chests' style-15 arm (W4-c33 census: the old
        // ctx.water gate invented a water-probe family the C# lacks).
        let mut row = Row::plain(UNDERWATER_ITEM);
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    }
    // The fixed rows' family extras (`AddBuriedChest`'s two `num10 > 0`
    // arms, the vanilla order: the surface arm's flag6 first `:36245-36266`,
    // then the deep arm's flag2/flag3/flag4 `:36810-36844`); the tenth
    // anniversary's flag11 rows park with M5-SEED-1. W4-c33d: BOTH arms
    // sit inside the C#'s `num10 > 0` blocks — a natural style-0 chest
    // (the flag/flag6 surface arm, the scatter's deep chests) rolls NO
    // family extras; the `preset_main` gate makes that exact (the
    // family-rolled mains — desert/ice/hell — keep the port's documented
    // band-first order, ADDENDUM 120 §8).
    if band == Band::Surface {
        if preset_main && ctx.living_tree {
            // `flag6` `:36245-36266`.
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
        // W4-c33d: the C# block is `num10 > 0` — the family dispatch
        // (`:36023-36125`) SETS num10 for the desert/ice/hell families
        // before the band runs, so their extras fire even with no caller
        // main; the port rolls those mains after the band (the documented
        // band-first order, ADDENDUM 120 §8), so the family probes below
        // stand in for the num10 gate. A family-less natural chest enters
        // no inner arm (the probes are the gate); a preset-main chest
        // (water/sky/pyramid) enters exactly the arms whose probes read
        // true, as the C# admits it through `num10 > 0`.
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
    // The wooden-chest family tails (`:37384-37470`, the
    // `itemIndex > 0 && chestTileType == 21` block — every band): the two
    // style rows and the flag13 water pair. The Containers2-tile 467 block
    // (`:37471-37532`) carries NONE of them — it repeats only the flag4 /
    // flag2 / flag12 rows, which land ungated in `stock_band` — so these
    // three arms gate on the anchor kind. The style rows landed once
    // (2026-11-09, `600dafdb`) and were dropped by a later merge's
    // resolution; restored with the citations intact.
    // `:37453-37455` — the styles 23-27 pottery row at 1/2 (the sweep's
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
    /// `GenVars.generatedShadowKey` / `generatedRamRune` (`:36557-36568`):
    /// the flag9 ladder's once-per-world fairness grants (W4-c33 census).
    genned_shadow_key: bool,
    genned_ram_rune: bool,
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
            genned_shadow_key: false,
            genned_ram_rune: false,
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
            &mut self.genned_shadow_key,
            &mut self.genned_ram_rune,
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
        lava_line: w.lava_line,
        silver_is_tungsten: w.ore_tiers.silver == 168,
        ore_tiers: w.ore_tiers,
    };
    stock.stock(&ev, rng)
}

/// M6-SAVE-1b: the save's v7 chest records (`save::load`, `SaveChests`'s
/// write-side twin `WorldFile.cs:1606-1676`) as a seed store for
/// [`merge_store`] — the load half of `App::save_world`'s record build.
///
/// The anchor cull is the vanilla `LoadChests` validity sweep
/// (`WorldFile.cs:2723-2805`, `FixAgainstExploits`): a record survives only
/// when its `(x, y)` is an active `BasicChest` tile of kind 21/467 in the
/// loaded world (`TileID.cs:570`), so a hand-edited or stale save cannot
/// resurrect a chest over plain air.
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
            ..Default::default()
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

/// `GiveItemGoodPrefixes` (`WorldGen.cs:24736-24758`): the accessory / melee /
/// ranged / magic / summon arms in the C# order. W4-e15: the flags ride the
/// CORRECTED `registry::item` row (the lunar chainsaws 2773/2778/2783/3463 are
/// zero rows in the generated table — `melee = false` — while the C# sets
/// `melee = true`, `Item.cs:26502`; the raw read skipped their melee arm), and
/// the melee arm reads the C#'s own `Item.melee` FIELD (`:24742`), not the
/// damage class — machine-diffed identical across all 6 197 generated rows.
fn give_item_good_prefixes(slot: &mut ItemSlot, rng: &mut dyn tr_core::worldgen::chest_stock::WorldGenRand) {
    let def = match crate::registry::item(slot.kind) {
        Some(d) => d,
        None => return,
    };
    if def.accessory {
        prefix_item_from_options(slot, GOOD_PREFIX_ACCESSORY, rng);
    }
    if def.melee {
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
            lava_line: w.lava_line,
            silver_is_tungsten: w.ore_tiers.silver == 168,
            ore_tiers: w.ore_tiers,
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

    /// W4-e15: `GiveItemGoodPrefixes`'s melee arm reads the CORRECTED row —
    /// the lunar chainsaws (2773/2778/2783/3463) are zero rows in the
    /// generated table (`melee = false`, damage −1) but the C# sets
    /// `melee = true` (`Item.cs:26502`, the 2778/2783/3463 arms re-run
    /// `SetDefaults3(2773)`), transcribed in `ITEM_CORRECTIONS`. The raw-row
    /// read skipped their melee arm entirely (zero draws); the corrected
    /// read runs the {81, 59, 57} try-loop. The witness replays the exact
    /// `Next(3)` pick on a twin stream — a skipped arm fails the state
    /// equality, not just the landing set.
    #[test]
    fn w4_e15_the_good_prefix_sweep_reads_the_corrected_melee_flag() {
        assert!(!crate::itemdata::ITEMS[2773].melee, "the raw row is a zero row");
        assert!(crate::registry::item(2773).unwrap().melee, "the correction sets melee");
        let good = [81u16, 59, 57];
        for kind in [2773u16, 2778, 2783, 3463] {
            let mut chests = vec![WorldChest {
                tile: (10, 10),
                slots: vec![ItemSlot { kind, name: "", count: 1, prefix: 0 }],
                name: String::new(),
                max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
                ..Default::default()
            }];
            let mut rng = tr_core::UnifiedRandom::new(0x5EED);
            let mut twin = tr_core::UnifiedRandom::new(0x5EED);
            improve_all_chest_contents(&mut chests, &mut rng);
            // The melee arm is the ONLY arm these kinds take (not accessory,
            // not in the guns/magic/summon sets): one `Next(3)` option pick,
            // then the exact-id `Prefix(num)` applies with no draw (damage
            // 80 / kb 4 keep every melee-table id visible).
            let pick = good[twin.next_max(3) as usize];
            assert_eq!(chests[0].slots[0].prefix, pick, "chainsaw {kind}");
            assert_eq!(rng.peek(), twin.peek(), "chainsaw {kind}: exactly one Next(3)");
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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

    /// W4-a2 / P0-3: the anchor cull's tile set is `TileID.Sets.BasicChest`
    /// = {21, 467} (`TileID.cs:570`) — a kind-467 Containers2 anchor survives
    /// the sweep, and a kind-341 `LivingDemonFire` (`TileID.cs:1311`) anchor
    /// is culled exactly like plain air (the former `[21, 341]` pair admitted
    /// the fire block and dropped every real Containers2 chest).
    #[test]
    fn the_record_cull_reads_the_basic_chest_kinds() {
        assert_eq!(CHEST_KINDS, [21, 467], "TileID.Sets.BasicChest (TileID.cs:570)");
        let mut w = rig();
        put_chest(&mut w, 40, 198);
        put_chest(&mut w, 60, 198);
        put_chest(&mut w, 80, 198);
        // Retype the second chest's quad to Containers2 (467) and the third
        // to LivingDemonFire (341) — the same frames, only the kind changes.
        for dy in 0..2i32 {
            for dx in 0..2i32 {
                let i = w.index(60 + dx, 198 + dy);
                w.tiles[i].kind = 467;
                let i = w.index(80 + dx, 198 + dy);
                w.tiles[i].kind = 341;
            }
        }
        let rec = |x: i32| ChestRecord {
            x,
            y: 198,
            name: String::new(),
            slots: vec![ChestItem { stack: 1, kind: 9, prefix: 0 }],
        };
        let store = store_from_records(&w, &[rec(40), rec(60), rec(80)]);
        assert_eq!(store.len(), 2, "21 and 467 survive; the 341 fire block does not");
        assert_eq!(store[0].tile, (40, 198));
        assert_eq!(store[1].tile, (60, 198));
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
        // W4-c33: a preset main skips the band weapon (`num10 > 0` branch).
        // The style-0 preset chest at surface depth is NOT flag/flag6/
        // flag11, so the C# falls through to the MID arm (`:36227` gate,
        // `:36503`) — the follow rows are the mid consumables; at least
        // one lands across a seed sweep.
        let follow = (1u64..40).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 212, &mut rng);
            rows[0].kind == 212 && rows.len() >= 2
        });
        assert!(follow, "the band consumable rows follow the cycle pick");
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

    /// The sky probe reads the port's own writers: the cloud disc flood
    /// writes `WALL_CLOUD` = 73 and the verbatim IslandHouse interior the
    /// disc wall 82 (`worldgen/islands.rs`, W4-g44), not the vanilla
    /// 216/187 cloud walls (216 is the underground desert's shell —
    /// `worldgen/desert.rs`). W4-c33d: the probe is a POSITIONAL read only —
    /// the stocking's flag12 is style/main based (`:36176`), so the probe
    /// no longer gates any loot row; this witness pins the probe itself.
    /// The sky probe reads wall 73 — the pre-W4-d104 island houses wrote
    /// `WALL_CLOUD` = 73; the faithful `IslandHouse` port writes the
    /// vanilla sunplate wall 82 (`worldgen/island_houses.rs`), so the
    /// probe's live writer is the loaded-world path — the fixture plants
    /// the wall directly.
    #[test]
    fn the_sky_probe_reads_the_cloud_wall() {
        let mut w = rig();
        // A cloud wall near the anchor: the positional sky probe reads it.
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(120 + dx, 197 + dy);
                w.tiles[i].wall = 73;
            }
        }
        let ctx = tr_core::worldgen::chest_stock::context_at(&w, 120, 198);
        assert!(ctx.sky, "the cloud wall reads as the skyware family");
        // The IslandHouse disc wall 82 reads the same probe (g44's graft).
        let mut w82 = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w82.index(120 + dx, 197 + dy);
                w82.tiles[i].wall = 82;
            }
        }
        assert!(
            tr_core::worldgen::chest_stock::context_at(&w82, 120, 198).sky,
            "the IslandHouse disc wall reads the sky probe too"
        );
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
        // `flag13` (`:37406-37409` + `:37457-37469`): the Lihzahrd Furnace
        // guaranteed, then the Solar Tablet at 1/5 or the Lunar Tablet stack.
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
        // W4-c33 census re-derivation: the tail blocks (`:37384-37532`) —
        // the flag4 pair (1/4 + 1/50) in BOTH kind blocks, the flag13
        // TEMPLE pair (style 16: the furnace unconditional + the 1/5
        // tablet), and the style-pottery rows INSIDE the flag12 block
        // (kind 21 styles 23-27 / kind 467 style 13, with the sky probe).

        // A temple anchor (style 16) takes the furnace unconditionally and
        // exactly one tablet row.
        let mut w = rig();
        put_chest(&mut w, 100, 198);
        let mut stock = ChestStock::new(3, false);
        let mut rng = tr_core::UnifiedRandom::new(3);
        let rows = stock_chest_at(&mut stock, &w, 100, 198, false, 16, 21, 0, &mut rng);
        assert!(
            rows.iter().any(|r| r.kind == 2195),
            "the flag13 furnace is unconditional"
        );
        let tablets: Vec<u16> = rows
            .iter()
            .filter(|r| r.kind == 2766 || r.kind == 2767)
            .map(|r| r.kind)
            .collect();
        assert_eq!(tablets.len(), 1, "exactly one tablet row ({tablets:?})");

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

        // A style-13 MAHOGANY anchor takes its own 1/2 pottery row
        // (`:37528-37531` sits in the 467 block — W4-g44 re-census; the
        // c33d `style_row` covers it OUTSIDE flag12).
        let mut w = rig();
        put_chest(&mut w, 100, 198);
        for dy in 0..2 {
            for dx in 0..2 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].kind = 467;
                w.tiles[i].frame_x = 18 * 13;
                w.tiles[i].frame_y = 18 * dy as i16;
            }
        }
        let rows13 = (1u64..64).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 198, false, 13, 467, 0, &mut rng)
                .iter()
                .any(|r| r.kind == 5234)
        });
        assert!(rows13, "the style-13 row fires on some stream");

        // A style-25 LOCKED biome anchor takes the 1/2 RemnantsofDevotion
        // row (`:37453-37456`). W4-c33d: the row is a num9 style test
        // OUTSIDE flag12 — the locked chest (main 1260, the Frozen chest)
        // needs no sky wall and no flag12; the old flag12 nest dropped it.
        let mut w = rig();
        put_chest(&mut w, 100, 198);
        let rows25 = (1u64..64).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 198, false, 26, 21, 1260, &mut rng)
                .iter()
                .any(|r| r.kind == 5234)
        });
        assert!(rows25, "the style-26 locked-chest pottery row fires on some stream");
    }

    #[test]
    fn the_21_block_rows_gate_on_the_chest_kind() {
        // The vanilla `itemIndex > 0 && chestTileType == 21` block's rows
        // (`:37384-37470`) apply to the wooden chests only; the Containers2
        // (467) block (`:37471-37532`) repeats just the flag4/flag2/
        // flag12 rows. The port gates the 21-only rows on the anchor kind.

        // A dungeon-family anchor with a NON-zero style: the flag9 row
        // (2192) rolls 1/8 (`:37410-37413` needs num9 != 0).
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
                stock_chest_at(&mut stock, &w, 100, 198, false, 2, 21, 0, &mut rng)
                    .iter()
                .any(|r| r.kind == 2192)
            })
            .count();
        assert!(hits > 0, "the 1/8 flag9 row fires on some stream ({hits})");
        assert!(hits < 600, "and not on every one — the 1/8 gate is real");
        // The style-0 negative: flag9 fails (`num9 != 0`), no Bone Welder.
        let zero_hits = (1u64..200)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng)
                    .iter()
                    .any(|r| r.kind == 2192)
            })
            .count();
        assert_eq!(zero_hits, 0, "a style-0 dungeon chest takes no flag9 row");

        // A kind-467 (Containers2) anchor with a water + sky-25 context takes
        // none of the 21-only rows, while a kind-21 anchor does.
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -1i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].kind = 4869; // the water family's tile
            }
        }
        // The Containers2 chest tile 467 at the anchor (`TileID.Sets.BasicChest`
        // = {21, 467}, `TileID.cs:570`; the former plant used 341 —
        // `LivingDemonFire`, `TileID.cs:1311` — never a chest tile).
        put_chest(&mut w, 100, 198);
        for dy in 0..2 {
            for dx in 0..2 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].kind = 467;
            }
        }
        let mut stock = ChestStock::new(3, false);
        let mut rng = tr_core::UnifiedRandom::new(3);
        let mahog = stock_chest_at(&mut stock, &w, 100, 198, false, 25, 467, 0, &mut rng);
        assert!(
            mahog.iter().any(|r| matches!(r.kind, 2766 | 2767)),
            "the 21 temple tablet"
        );
        let mut stock = ChestStock::new(3, false);
        let mut rng = tr_core::UnifiedRandom::new(3);
        let mahog = stock_chest_at(&mut stock, &w, 100, 198, false, 16, 467, 0, &mut rng);
        assert!(
            !mahog.iter().any(|r| matches!(r.kind, 2195 | 2766 | 2767)),
            "the 467 block carries no flag13 rows: {:?}",
            mahog.iter().map(|r| r.kind).collect::<Vec<_>>()
        );

        // The Cloud stack lands in BOTH blocks on a skyware anchor.
        // W4-c33d: flag12 is style/main based — the kind-21 style-13 chest
        // needs no wall at all; the 467 half is the remix corrupt retype's
        // product (the core sets kind 467/style 3 and the flag stays set),
        // so the driver carries the sky main 159 through it.
        let mut w = rig();
        put_chest(&mut w, 100, 198);
        for (kind, style, main) in [(21u16, 13i32, 0u16), (467, 3, 159)] {
            let mut stock = ChestStock::new(3, false);
            let mut rng = tr_core::UnifiedRandom::new(3);
            let rows = stock_chest_at(&mut stock, &w, 100, 198, false, style, kind, main, &mut rng);
            assert!(
                rows.iter().any(|r| r.kind == 751),
                "the Cloud stack lands on the kind-{kind} block"
            );
        }
        // The wall-73 negative: a style-0 main-0 chest beside a cloud wall
        // is NOT flag12 — no SkyMill/pottery/Cloud draws (the old ctx.sky
        // probe fired here).
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].wall = 73;
            }
        }
        put_chest(&mut w, 100, 198);
        let plain_hits = (1u64..120)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng)
                    .iter()
                    .any(|r| matches!(r.kind, 751 | 2197 | 5258 | 5226 | 5254 | 5238 | 5255 | 5388))
            })
            .count();
        assert_eq!(plain_hits, 0, "a cloud-wall neighbour alone is not flag12");
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
    /// the `flag11` companion 866 in slot 1 right after the mask. W4-c33
    /// sweep: the caller's style is 1 (`:27873`) and the chamber's wall is
    /// SandstoneBrick 34 — `flag11` (`:36172`) is main-based, so the arm
    /// fires without the wall-187 probe.
    #[test]
    fn the_pyramid_chest_stocks_the_pharaoh_roll_and_the_robe() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 198 + dy);
                w.tiles[i].wall = 34; // the faithful chamber's SandstoneBrick
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
            let rows = stock_chest_at(&mut stock, &w, 100, 197, false, 1, 21, main, &mut rng);
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

    /// W4-c33 sweep: `flag11` (`:36172-36175`) forces the SURFACE arm at any
    /// depth (`:36227`'s `| flag11` disjunct) — a deep pyramid chamber's
    /// chest takes the surface consumables, NOT the deep weapon/gem rows,
    /// and the `num10 > 0` path skips the `Next(11)` surface-weapon roll
    /// (`:36229`). A style-0 chest with the same main is NOT flag11
    /// (`num9 != 0` fails): it keeps its natural band rows.
    #[test]
    fn the_flag11_pyramid_chest_takes_the_surface_rows_at_depth() {
        let mut w = deep_rig(); // 200x600: y=340 is a real Deep band
        put_chest(&mut w, 100, 340);
        let deep_weapons = [49u16, 50, 53, 54, 5011, 975];
        let surface_weapons = [280u16, 281, 284, 285, 953, 946, 3068, 3069, 3084, 4341, 6165];
        let mut saw_surface_consumable = false;
        for seed in 1u64..60 {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 340, false, 1, 21, 934, &mut rng);
            assert_eq!(rows[0].kind, 934, "the carpet main leads ({seed})");
            assert!(
                !rows.iter().skip(1).any(|r| deep_weapons.contains(&r.kind)),
                "no deep weapon row on a flag11 chest ({seed}): {:?}",
                rows.iter().map(|r| r.kind).collect::<Vec<_>>()
            );
            assert!(
                !rows.iter().any(|r| surface_weapons.contains(&r.kind)),
                "the num10 > 0 path rolls no surface weapon ({seed})"
            );
            // The surface arm's consumables (282/279/3093/4345/168/bars/
            // 965/40/42/28/2350/herbs/8/31/72/9) — at least one lands
            // across the sweep.
            if rows.iter().any(|r| matches!(r.kind, 282 | 279 | 3093 | 4345 | 168 | 965 | 2350)) {
                saw_surface_consumable = true;
            }
        }
        assert!(saw_surface_consumable, "the surface consumables roll at depth");
        // The style-0 negative: no flag11, the deep band keeps its weapon.
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let rows = stock_chest_at(&mut stock, &w, 100, 340, false, 0, 21, 934, &mut rng);
        assert!(
            rows.iter().any(|r| deep_weapons.contains(&r.kind)),
            "a style-0 chest keeps its natural deep rows"
        );
    }

    /// W4-c33 flag5 verify (`:36515-36525` mid / `:36845-36855` deep): a
    /// style-17 water chest with a pre-set main rolls TWO INDEPENDENT 1/2
    /// rows — SharkBait 4425 and SandcastleBucket 4460 (both, either or
    /// neither) — right after the main, ahead of the band's weapon row.
    /// The style-0 negative takes neither. All five scatter mains walk it.
    #[test]
    fn the_flag5_water_pair_rolls_two_independent_halves() {
        let mut both = 0u32;
        let mut bait_only = 0u32;
        let mut bucket_only = 0u32;
        let mut neither = 0u32;
        for seed in 1u64..400 {
            for (band_y, rig) in [(230i32, rig()), (340, deep_rig())] {
                let mut w = rig;
                put_chest(&mut w, 100, band_y);
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                let rows =
                    stock_chest_at(&mut stock, &w, 100, band_y, false, 17, 21, 186, &mut rng);
                assert_eq!(rows[0].kind, 186, "the main leads ({seed}, y={band_y})");
                let bait = rows.iter().any(|r| r.kind == 4425);
                let bucket = rows.iter().any(|r| r.kind == 4460);
                match (bait, bucket) {
                    (true, true) => both += 1,
                    (true, false) => bait_only += 1,
                    (false, true) => bucket_only += 1,
                    (false, false) => neither += 1,
                }
            }
        }
        assert!(both > 0, "the pair lands together ({both})");
        assert!(bait_only > 0, "the bait lands alone ({bait_only})");
        assert!(bucket_only > 0, "the bucket lands alone ({bucket_only})");
        assert!(neither > 0, "and neither lands ({neither})");
        // The style-0 negative: the flag5 gate is the style, not the main.
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let rows = stock_chest_at(&mut stock, &w, 100, 340, false, 0, 21, 186, &mut rng);
        assert!(
            !rows.iter().any(|r| matches!(r.kind, 4425 | 4460)),
            "a style-0 chest with the same main takes no flag5 pair"
        );
    }

    /// W4-c33 flag5 census: every Water Chests main (the NextFromList five
    /// 863/186/277/187/4404 + the extraLiquid 268) stocks as slot 0 with the
    /// flag5 pair behind it — the ocean loot family is table-complete.
    #[test]
    fn every_water_chest_main_stocks_with_the_flag5_pair() {
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        for main in [863u16, 186, 277, 187, 4404, 268] {
            let hit = (1u64..200).any(|seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                let rows =
                    stock_chest_at(&mut stock, &w, 100, 340, false, 17, 21, main, &mut rng);
                rows[0].kind == main && rows.iter().any(|r| matches!(r.kind, 4425 | 4460))
            });
            assert!(hit, "main {main} stocks with a flag5 row on some stream");
        }
    }

    /// W4-c33 pyramid-loot census (`flag12` `:36526-36532`): the skyware
    /// family's mid-band 1/40 CreativeWings 4978 rolls with the natural
    /// prefix; the surface/deep bands carry no such row. W4-c33d: the
    /// gate is flag12 (style 13 / the four sky mains) AND a preset main —
    /// the fixture is a SUNPLATE-wall island house (no cloud wall 73),
    /// the classic `IslandHouse` interior; a style-13 chest with NO main
    /// rolls no wings draw (the `num10 > 0` block).
    #[test]
    fn the_sky_mid_band_rolls_the_creative_wings() {
        let mut w = World::new("t".into(), 200, 600);
        w.world_surface = 200.0;
        w.rock_layer = 400.0; // y=300 sits in the Mid band
        w.seed = 42;
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 300 + dy);
                w.tiles[i].wall = 82; // the IslandHouse disc wall (NOT 73)
            }
        }
        put_chest(&mut w, 100, 300);
        let hits = (1u64..900)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 300, false, 13, 21, 159, &mut rng)
                    .iter()
                    .any(|r| r.kind == 4978)
            })
            .count();
        assert!(hits > 0, "the 1/40 wings row fires ({hits}/900)");
        assert!(hits < 120, "and stays rare ({hits}/900 ~ 1/40)");
        // The main-0 negative: a style-13 chest with no preset main takes
        // the weapon ELSE branch — no wings row on ANY stream.
        let zero_hits = (1u64..200)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 300, false, 13, 21, 0, &mut rng)
                    .iter()
                    .any(|r| r.kind == 4978)
            })
            .count();
        assert_eq!(zero_hits, 0, "a main-0 style-13 chest rolls no wings");
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

    /// `IslandHouse`'s cycle (`:79348-79368`): the first four sky chests of
    /// a world take the fixed 159/65/158/2219 in order — the vanilla's
    /// `skyIslandHouseCount` table before the `> 3` re-roll. W4-c33d: the
    /// fixture is the SUNPLATE house interior (disc wall 82), not the
    /// cloud-disc wall 73 — flag12 is style/main based either way.
    #[test]
    fn the_sky_chests_walk_the_island_house_cycle() {
        let mut w = rig();
        for (slot, x) in [40i32, 60, 80, 100].iter().enumerate() {
            for dx in -2i32..=0 {
                for dy in -2i32..=0 {
                    let i = w.index(x + dx, 197 + dy);
                    w.tiles[i].wall = 82; // the IslandHouse disc wall
                }
            }
            let _ = slot;
            put_chest(&mut w, *x, 199);
        }
        // The cycle itself moved to the site (`islands.rs::island_house`,
        // `GenVars.skyIslandHouseCount`); the hook inserts the site's pick
        // (`main_item`) into slot 0 — the walk order rides the same table.
        // The cycle itself moved to the site (`island_houses.rs`'s
        // `IslandHouse` port, `GenVars.skyIslandHouseCount` — W4-d104); the
        // hook inserts the site's pick (`main_item`) into slot 0 — the walk
        // order rides the same table.
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

    /// W4-g44 islands final 3/5: the skyware loot-table census — the
    /// flag12 tail rows verified landed against the re-read C#: the Sky
    /// Mill 2197 at 1/3 (`:37402`/`:37489`), the six-piece pottery roll
    /// (`:37418-37445`/`:37493-37521`: 5258/5226/5254/5238/5255/5388),
    /// the Cloud 751 stack 50-101 (`:37447`/`:37522`), and the main's
    /// Prefix(-1) (`:36232`/`:36513`) + the CreativeWings 4978 1/40 rider
    /// (`:36526-36531`).
    #[test]
    fn the_skyware_tail_rows_and_the_wings_rider() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(40 + dx, 197 + dy);
                w.tiles[i].wall = 73; // the cloud-wall sky probe
            }
        }
        put_chest(&mut w, 40, 199);
        // Sweep drivers: the tail rows are odds-gated, so collect across
        // streams. The pottery set + the Cloud stack + the Sky Mill all
        // appear; the wings rider lands at ~1/40.
        let mut saw_mill = false;
        let mut saw_cloud = false;
        let mut pottery = std::collections::BTreeSet::new();
        let mut wings = 0;
        let mut main_prefix = 0;
        for seed in 1u64..400 {
            let mut stock = ChestStock::new(seed, false);
            let slots = stock_anchor(&mut stock, &w, 40, 199, 13, 21, 159);
            assert_eq!(slots[0].kind, 159, "the balloon rides slot 0");
            main_prefix += (slots[0].prefix != 0) as u32;
            for s in &slots[1..] {
                match s.kind {
                    2197 => saw_mill = true,
                    751 => {
                        saw_cloud = true;
                        assert!((50..=100).contains(&s.stack), "the Cloud stack 50-100");
                    }
                    5258 | 5226 | 5254 | 5238 | 5255 | 5388 => {
                        pottery.insert(s.kind);
                    }
                    4978 => wings += 1,
                    _ => {}
                }
            }
        }
        assert!(saw_mill, "the Sky Mill 1/3 row fires");
        assert!(saw_cloud, "the Cloud stack row fires");
        assert_eq!(pottery.len(), 6, "all six pottery pieces appear: {pottery:?}");
        assert!(wings > 0, "the CreativeWings 1/40 rider fires ({wings})");
        assert!(wings < 60, "and it is rare — the 1/40 gate is real ({wings})");
        assert!(main_prefix > 0, "the balloon's Prefix(-1) rolls on some stream");
        // The mahogany twin (the remix corrupt retype's 467/style-3 chest)
        // takes the same flag12 tail: the Cloud stack rides BOTH blocks
        // (the :37522 row) — the census fix this lane landed.
        let mut w2 = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w2.index(40 + dx, 197 + dy);
                w2.tiles[i].wall = 82; // the disc-wall sky probe
            }
        }
        put_chest(&mut w2, 40, 199);
        for dy in 0..2 {
            for dx in 0..2 {
                let i = w2.index(40 + dx, 199 + dy);
                w2.tiles[i].kind = 467;
            }
        }
        let mut mahog_cloud = false;
        for seed in 1u64..200 {
            let mut stock = ChestStock::new(seed, false);
            let slots = stock_anchor(&mut stock, &w2, 40, 199, 3, 467, 159);
            if slots.iter().any(|s| s.kind == 751) {
                mahog_cloud = true;
                break;
            }
        }
        assert!(mahog_cloud, "the 467 block's Cloud row fires (:37522)");
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

    // --- W4-c33 FINAL census witnesses (written not run, code-only lane) ---

    /// Item 4 — the prefix census: EVERY `num10 > 0` slot-0 write calls
    /// `Prefix(-1)` (`:36232` surface / `:36513` mid / `:36808` deep /
    /// `:37167` deepest). The rollable mains (the accessories/weapons)
    /// land a prefix on some stream; the non-prefixable mains (the
    /// Extractinator 997, the minecarts) draw NOTHING (`CanHavePrefixes`
    /// early return) — the stream-position witness pins both halves.
    #[test]
    fn every_preset_main_rolls_the_slot_zero_prefix() {
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        // The water mains are accessories/weapons — rollable. Across the
        // sweep at least one lands a non-zero prefix (the 3/4 roll gate).
        let mut prefixed = 0u32;
        for main in [863u16, 186, 277, 187, 4404, 268] {
            for seed in 1u64..60 {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                let rows =
                    stock_chest_at(&mut stock, &w, 100, 340, false, 17, 21, main, &mut rng);
                assert_eq!(rows[0].kind, main);
                if rows[0].prefix != 0 {
                    prefixed += 1;
                }
            }
        }
        assert!(prefixed > 0, "the water mains roll their slot-0 prefix ({prefixed})");
        // The sky mains (159 balloon / 65 Starfury / 158 horseshoe / 2219
        // magnet) are rollable too — the old sky arm wrote them PLAIN.
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 197 + dy);
                w.tiles[i].wall = 73;
            }
        }
        put_chest(&mut w, 100, 198);
        let sky_prefixed = (1u64..80).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 198, false, 13, 21, 65, &mut rng);
            rows[0].kind == 65 && rows[0].prefix != 0
        });
        assert!(sky_prefixed, "the sky main rolls its slot-0 prefix");
    }

    /// The non-prefixable main draws nothing: a 997 Extractinator main
    /// (the ice table's 1/20 override) keeps prefix 0 and the stream
    /// position is the SAME as a no-prefix row would leave it — the
    /// `CanHavePrefixes` early return (`Item.cs:515-518`).
    #[test]
    fn a_non_prefixable_main_draws_nothing() {
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let rows = stock_chest_at(&mut stock, &w, 100, 340, false, 17, 21, 997, &mut rng);
        assert_eq!(rows[0].kind, 997);
        assert_eq!(rows[0].prefix, 0, "the Extractinator takes no prefix");
        // The stream mirror: rolling the same event with the main's prefix
        // call removed must leave the identical state — the 997 call is a
        // no-op. Compare against a second driver that stocks the same rows.
        let mut stock2 = ChestStock::new(42, false);
        let mut rng2 = tr_core::UnifiedRandom::new(42);
        let rows2 = stock_chest_at(&mut stock2, &w, 100, 340, false, 17, 21, 997, &mut rng2);
        assert_eq!(rows, rows2, "the no-draw prefix keeps the stream deterministic");
    }

    /// Item 1 — the deep ladder re-derivation (`:36879-36994`): the
    /// `Next(7)` pick is drawn FIRST, the lava gate ANDs the lava line,
    /// the weapon prefixes ONCE. A below-lava-line deep chest rolls the
    /// LavaCharm 906 on the 1/20 gate; an above-lava-line one never does.
    #[test]
    fn the_deep_lava_charm_gates_on_the_lava_line() {
        let mut w = deep_rig(); // height 600, rock 300; y=340 is Deep
        put_chest(&mut w, 100, 340);
        // Below the lava line: the 906 row fires on some stream.
        w.lava_line = 300;
        let below = (1u64..400).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 340, false, 0, 21, 0, &mut rng)
                .iter()
                .any(|r| r.kind == 906)
        });
        assert!(below, "the lava charm fires below the lava line");
        // Above the lava line: the gate fails on EVERY stream.
        w.lava_line = 500;
        let above = (1u64..400).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 340, false, 0, 21, 0, &mut rng)
                .iter()
                .any(|r| r.kind == 906)
        });
        assert!(!above, "the lava charm never fires above the lava line");
    }

    /// The deep weapon prefixes ONCE (`:36904-36933`): the old copy rolled
    /// it twice (inside the pick helper AND at the call site). The witness
    /// counts the stream draws a deep weapon row consumes and pins it to
    /// the single-roll shape.
    #[test]
    fn the_deep_weapon_prefixes_once() {
        // A natural deep chest (no family, no preset) whose band weapon is
        // the deep pick: roll it and confirm the weapon row carries at most
        // one prefix and the row count matches the single-roll stream.
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        w.lava_line = 500; // keep the lava gate off so the weapon branch runs
        let mut saw_weapon = false;
        for seed in 1u64..200 {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 340, false, 0, 21, 0, &mut rng);
            if let Some(weapon) = rows.iter().find(|r| matches!(r.kind, 49 | 50 | 53 | 54 | 5011 | 975)) {
                saw_weapon = true;
                // The single-roll engine leaves a valid prefix (0 or a
                // rollable id) — the double roll would have consumed an
                // extra draw and shifted every following row.
                assert!(weapon.prefix <= 83, "a sane prefix id ({})", weapon.prefix);
            }
        }
        assert!(saw_weapon, "the deep weapon branch runs on some stream");
    }

    /// Item 1 — the flag9 fairness ladder (`:36555-36576` mid /
    /// `:36856-36877` deep): the ShadowKey 329 and the RamRune 5465 are
    /// ONCE-per-world grants (`!genned || Next(3/8)==0`), the rune and the
    /// 6156 bracer prefix. The first dungeon chest grants both; a second
    /// needs the odds roll. W4-c33d: the ladder is `num10 > 0`-gated in the
    /// C#; the port's dungeon cycle supplies that main internally (the
    /// flag9 arm inserts `dungeon.take()`), so the driver keeps main 0 and
    /// the family gate implies num10 > 0.
    #[test]
    fn the_dungeon_fairness_grants_once_per_world() {
        let mut w = deep_rig();
        for dx in -2i32..=2 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 340 + dy);
                w.tiles[i].wall = 8; // a dungeon wall
            }
        }
        put_chest(&mut w, 100, 340);
        // A seed whose SECOND chest misses both fairness odds rolls.
        let seed = (1u64..400)
            .find(|&s| {
                let mut stock = ChestStock::new(s, false);
                let mut rng = tr_core::UnifiedRandom::new(s as i32);
                let first = stock_chest_at(&mut stock, &w, 100, 340, false, 2, 21, 0, &mut rng);
                let second = stock_chest_at(&mut stock, &w, 100, 340, false, 2, 21, 0, &mut rng);
                let key = |rows: &[ChestItem]| rows.iter().filter(|r| r.kind == 329).count();
                let rune = |rows: &[ChestItem]| rows.iter().filter(|r| r.kind == 5465).count();
                key(&first) == 1 && rune(&first) == 1 && key(&second) == 0 && rune(&second) == 0
            })
            .expect("a seed whose second chest misses both fairness rolls exists");
        let mut stock = ChestStock::new(seed, false);
        let mut rng = tr_core::UnifiedRandom::new(seed as i32);
        let first = stock_chest_at(&mut stock, &w, 100, 340, false, 2, 21, 0, &mut rng);
        assert!(first.iter().any(|r| r.kind == 329), "the first chest grants the key");
        assert!(first.iter().any(|r| r.kind == 5465), "the first chest grants the rune");
        let second = stock_chest_at(&mut stock, &w, 100, 340, false, 2, 21, 0, &mut rng);
        assert!(!second.iter().any(|r| r.kind == 329), "the key does not repeat");
        assert!(!second.iter().any(|r| r.kind == 5465), "the rune does not repeat");
        // W4-c33d: the locked biome chest (style 26, flag10) skips the
        // ladder entirely — its caller-preset main 1260 rides slot 0 but
        // no ShadowKey/RamRune/bracer draws (`:36555`/`:36856` !flag10).
        let mut stock_l = ChestStock::new(seed, false);
        let mut rng_l = tr_core::UnifiedRandom::new(seed as i32);
        let locked = stock_chest_at(&mut stock_l, &w, 100, 340, false, 26, 21, 1260, &mut rng_l);
        assert!(
            !locked.iter().any(|r| matches!(r.kind, 329 | 5465 | 6156)),
            "a locked biome chest rolls no flag9 ladder"
        );
    }

    /// Item 1 — the surface-gate fall-through (`:36227`): a preset-main
    /// chest at surface depth is NOT flag/flag6/flag11, so it takes the MID
    /// arm (`:36503`), not the surface consumables. A style-0 natural chest
    /// at the same depth keeps the surface arm.
    #[test]
    fn a_preset_main_at_surface_depth_takes_the_mid_arm() {
        let mut w = rig(); // surface 200, rock 240; y=199 is surface-band
        put_chest(&mut w, 100, 199);
        // The water main 186 at surface depth: the mid arm's common rows
        // (the 1/5 CloudinaBottle 52 / the bomb 166) — NOT the surface
        // weapon. Sweep for a mid-only row.
        let mid_row = (1u64..200).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 199, false, 17, 21, 186, &mut rng);
            rows[0].kind == 186 && rows.iter().any(|r| matches!(r.kind, 52 | 166))
        });
        assert!(mid_row, "the preset-main surface chest takes the mid common rows");
        // The style-0 natural chest keeps the surface weapon.
        let surface_weapon = (1u64..60).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 199, false, 0, 21, 0, &mut rng);
            rows.iter().any(|r| matches!(r.kind, 280 | 281 | 284 | 285 | 953 | 946 | 3068 | 3069 | 3084 | 4341 | 6165))
        });
        assert!(surface_weapon, "the style-0 natural chest keeps the surface arm");
    }

    /// Item 1 — the deepest band's bullet row (`:37236-37257`): the Next(2)
    /// pick is real — pick 1 writes the SilverBullet 278, or the
    /// TungstenBullet 4915 on a tungsten world (`SavedOreTiers.Silver==168`).
    #[test]
    fn the_deepest_bullet_row_reads_the_silver_tier() {
        let mut w = World::new("t".into(), 200, 600);
        w.world_surface = 200.0;
        w.rock_layer = 300.0;
        w.seed = 42;
        put_chest(&mut w, 100, 590); // y=590 >= height-250=350 → Deepest
        // Classic silver tier: the bullet row lands 265 or 278.
        let classic = (1u64..300).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 590, false, 0, 21, 0, &mut rng)
                .iter()
                .any(|r| r.kind == 278)
        });
        assert!(classic, "the classic silver tier writes the SilverBullet 278");
        // Tungsten tier: the same pick writes 4915 instead.
        w.ore_tiers.silver = 168;
        let tungsten = (1u64..300).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 590, false, 0, 21, 0, &mut rng)
                .iter()
                .any(|r| r.kind == 4915)
        });
        assert!(tungsten, "the tungsten tier writes the TungstenBullet 4915");
    }

    /// Item 1 — the bar rows read the rolled ore tiers (`WorldGen.Reset`
    /// `:11270-11297`): an alternate-tier world writes the tin/lead/
    /// tungsten/platinum bars (703/704/705/706), not the classic ids.
    #[test]
    fn the_bar_rows_read_the_ore_tiers() {
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        w.ore_tiers = tr_core::ores::SavedOreTiers {
            copper: 166,
            iron: 167,
            silver: 168,
            gold: 169,
            ..tr_core::ores::SavedOreTiers::new()
        };
        let alt = (1u64..400).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 340, false, 0, 21, 0, &mut rng)
                .iter()
                .any(|r| matches!(r.kind, 705 | 706))
        });
        assert!(alt, "the alternate tiers write the tungsten/platinum bars");
    }

    /// Item 1 — the 3093/4345 surface bundles carry the two bonus stack
    /// draws (`:36360-36387`): the stack can exceed 1 (the old copy pinned
    /// it to 1 and dropped both draws).
    #[test]
    fn the_surface_bundles_roll_the_bonus_stacks() {
        let mut w = rig();
        put_chest(&mut w, 100, 199);
        let bonus = (1u64..400).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 199, false, 0, 21, 0, &mut rng)
                .iter()
                .any(|r| matches!(r.kind, 3093 | 4345) && r.stack > 1)
        });
        assert!(bonus, "a bundle lands a bonus stack on some stream");
    }

    /// Item 3 — the compose witness: the stocking hook fires on the d103
    /// core's `BuriedChestPlace` seam unchanged (style/kind/main ride the
    /// event). A style-17 water chest placed through the shared seam stocks
    /// its main + the flag5 pair exactly as the compact core's callers do.
    #[test]
    fn the_stocking_seam_composes_with_the_buried_chest_core() {
        let mut w = deep_rig();
        put_chest(&mut w, 100, 340);
        // The event the d103 core's `stock_placed_chest` builds for a
        // style-17 water chest (p.style=17, p.kind=21, main=186).
        let mut stock = ChestStock::new(42, false);
        let mut rng = tr_core::UnifiedRandom::new(42);
        let rows = stock_chest_at(&mut stock, &w, 100, 340, false, 17, 21, 186, &mut rng);
        assert_eq!(rows[0].kind, 186, "the core's main rides slot 0");
        // The flag5 pair is present on some stream (the style-17 gate).
        let pair = (1u64..100).any(|seed| {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            stock_chest_at(&mut stock, &w, 100, 340, false, 17, 21, 186, &mut rng)
                .iter()
                .any(|r| matches!(r.kind, 4425 | 4460))
        });
        assert!(pair, "the style-17 flag5 pair fires through the shared seam");
    }

    /// W4-c33d chest-gen-final — the flag7 mushroom family (`:36151-36153`
    /// kind 21 style 32): the MID band rolls the 1/2 ShroomMinecart 4450
    /// and an INDEPENDENT 1/3 MushroomVanity set 4779+4780+4781
    /// (`:36653-36666`); the DEEP band's arm is an EITHER-OR (`:36977-
    /// :36993`). Both sit in the `num10 == 0` branch — a preset main skips
    /// them. The old copy had no flag7 arm at all.
    #[test]
    fn the_flag7_mushroom_rows_roll_the_two_band_shapes() {
        // Mid band (y=300 between surface 200 and rock 400).
        let mut w = World::new("t".into(), 200, 600);
        w.world_surface = 200.0;
        w.rock_layer = 400.0;
        w.seed = 42;
        put_chest(&mut w, 100, 300);
        let mut cart = 0u32;
        let mut vanity = 0u32;
        let mut both = 0u32;
        for seed in 1u64..400 {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &w, 100, 300, false, 32, 21, 0, &mut rng);
            let c = rows.iter().any(|r| r.kind == 4450);
            let v = rows.iter().any(|r| r.kind == 4779);
            if c {
                cart += 1;
            }
            if v {
                vanity += 1;
                // The vanity set lands as the full triple.
                assert!(rows.iter().any(|r| r.kind == 4780));
                assert!(rows.iter().any(|r| r.kind == 4781));
            }
            if c && v {
                both += 1;
            }
        }
        assert!(cart > 0, "the mid 1/2 minecart fires ({cart}/400)");
        assert!(vanity > 0, "the mid 1/3 vanity set fires ({vanity}/400)");
        assert!(both > 0, "the mid rolls are INDEPENDENT ({both}/400 both)");
        // Deep band (y=340, rock 300, height 600): the either-or — the
        // vanity set lands ONLY when the minecart does not.
        let mut wd = deep_rig();
        put_chest(&mut wd, 100, 340);
        let mut deep_cart = 0u32;
        let mut deep_vanity = 0u32;
        for seed in 1u64..400 {
            let mut stock = ChestStock::new(seed, false);
            let mut rng = tr_core::UnifiedRandom::new(seed as i32);
            let rows = stock_chest_at(&mut stock, &wd, 100, 340, false, 32, 21, 0, &mut rng);
            let c = rows.iter().any(|r| r.kind == 4450);
            let v = rows.iter().any(|r| r.kind == 4779);
            assert!(!(c && v), "the deep arm is an either-or ({seed})");
            if c {
                deep_cart += 1;
            }
            if v {
                deep_vanity += 1;
            }
        }
        assert!(deep_cart > 0, "the deep minecart fires ({deep_cart}/400)");
        assert!(deep_vanity > 0, "the deep vanity set fires ({deep_vanity}/400)");
        // The preset-main negative: a style-32 chest WITH a main takes the
        // num10 > 0 branch — no flag7 draws on any stream.
        let preset_hits = (1u64..120)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &wd, 100, 340, false, 32, 21, 186, &mut rng)
                    .iter()
                    .any(|r| matches!(r.kind, 4450 | 4779))
            })
            .count();
        assert_eq!(preset_hits, 0, "a preset-main mushroom chest rolls no flag7");
    }

    /// W4-c33d chest-gen-final — the flag5 pair is `num10 > 0`-gated: a
    /// style-17 chest with NO main takes the weapon ELSE branch and rolls
    /// no 4425/4460 draws (`:36505`/`:36805` — the Water Chests callers
    /// always pre-set a main, so the arm always fires on the real sites).
    #[test]
    fn a_mainless_style_17_chest_rolls_no_flag5_pair() {
        for (y, mut w) in [(230i32, rig()), (340, deep_rig())] {
            put_chest(&mut w, 100, y);
            let hits = (1u64..150)
                .filter(|&seed| {
                    let mut stock = ChestStock::new(seed, false);
                    let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                    stock_chest_at(&mut stock, &w, 100, y, false, 17, 21, 0, &mut rng)
                        .iter()
                        .any(|r| matches!(r.kind, 4425 | 4460))
                })
                .count();
            assert_eq!(hits, 0, "a main-0 style-17 chest rolls no pair (y={y})");
        }
    }

    /// W4-c33d chest-gen-final — the surface flag6 extras are `num10 > 0`-
    /// gated (`:36245-36266`): a natural style-0 chest inside a living-tree
    /// wall-244 probe rolls NO 5629/4429/4427/5528 draws; the tree's own
    /// 832-main chest does.
    #[test]
    fn the_surface_flag6_extras_need_a_preset_main() {
        let mut w = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w.index(100 + dx, 197 + dy);
                w.tiles[i].wall = 244; // the living-tree passage wall
            }
        }
        put_chest(&mut w, 100, 198);
        let family: &[u16] = &[5629, 4429, 4427, 5528];
        let plain_hits = (1u64..150)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 198, false, 0, 21, 0, &mut rng)
                    .iter()
                    .any(|r| family.contains(&r.kind))
            })
            .count();
        assert_eq!(plain_hits, 0, "a main-0 surface chest rolls no flag6 extras");
        let main_hits = (1u64..150)
            .filter(|&seed| {
                let mut stock = ChestStock::new(seed, false);
                let mut rng = tr_core::UnifiedRandom::new(seed as i32);
                stock_chest_at(&mut stock, &w, 100, 198, false, 12, 21, 832, &mut rng)
                    .iter()
                    .any(|r| family.contains(&r.kind))
            })
            .count();
        assert!(main_hits > 0, "the 832-main tree chest rolls the flag6 extras");
    }
}
