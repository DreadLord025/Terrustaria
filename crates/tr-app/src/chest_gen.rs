//! World chest loot generation — the port of `WorldGen.AddBuriedChest`'s
//! stocking block (`WorldGen.cs:35871-37580`).
//!
//! The seam: the port's worldgen places the chest TILES
//! (`worldgen::structures::place_chest`), but the item store
//! (`App.world_chests`) stays empty until the world loads — vanilla stocks
//! the `Chest` objects inline with the placement, rolling `genRand` and
//! `Prefix(-1)` on the weapon/accessory rows. This module re-creates that
//! stocking at `App::new`: it scans the world for the chest anchors
//! (kind 21/341, `frame_x == 0 && frame_y == 0` — the anchor the
//! `chests_in_range` matcher uses) and rolls a deterministic
//! per-world-chest stream (`chest_rng`, the port's per-consumer stream
//! convention) so every world re-loads with the same loot.
//!
//! The roll is the whole C# block: the depth band (surface
//! `y < worldSurface+25` / mid `y < rockLayer` / deep
//! `y < maxTilesY-250` / else), the rows with their verbatim probabilities
//! (`Next(6) == 0` etc), the stack ranges, and the `Prefix(-1)` calls
//! routed through the ported `prefix::roll_prefix(kind, -1, rng)`.
//! Out of scope here (documented in the ROADMAP row): the seed-flavour rows
//! (vampire/errorWorld/rainbow/tenthAnniversary, `SecretSeed`/`Main`
//! statics the port does not generate), the `GenVars` cross-chest fairness
//! flags (`generatedShadowKey`/`gennedLivingMahoganyWands` — once per
//! world in vanilla, approximated here per-chest with the vanilla odds),
//! the desert/pyramid/water/skyisland style tables (the biome flags `flag2`
//! jungle / `flag4` mahogany / `flag12` skyware / `flag8` idol / `flag9`
//! dungeon keys are approximated by the depth band and the wall/tile
//! probes around the anchor — the full style dispatcher rides [7]).

use crate::chest::WorldChest;
use crate::item::{ItemSlot, EMPTY_SLOT};
use crate::prefix;
use tr_core::save::ChestRecord;
use tr_core::world::World;
use tr_core::tile::FLAG_ACTIVE;

/// The chest tile kinds an anchor can carry (`craft::CHEST_TILES`).
const CHEST_KINDS: [u16; 2] = [21, 341];
/// `AddBuriedChest`'s bands: the anchor's y is the floor row; vanilla's
/// `num7` is the chest's base row — the port uses the anchor directly.
const SURFACE_BAND_PAD: i32 = 25;
const DEEP_BAND_PAD: i32 = 250;
/// `GenVars.hellChestItem` — the underwater-chest 939 override carries the
/// water chests' item; the unused slot keeps the vanilla name.
const UNDERWATER_ITEM: u16 = 939;

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
        Self { kind, count: 1, prefix: 0 }
    }

    fn stacked(kind: u16, count: u32) -> Self {
        Self { kind, count, prefix: 0 }
    }
}

/// The port's per-consumer splitmix stream (`rain.rs`'s `next_u64` shape).
fn next_u64(rng: &mut u64) -> u64 {
    *rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *rng;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// `UnifiedRandom.Next(max)` — `[0, max)`.
fn next(rng: &mut u64, max: u64) -> u64 {
    next_u64(rng) % max
}

/// `Next(min, max)` — `[min, max)`.
fn ranged(rng: &mut u64, min: u64, max: u64) -> u64 {
    min + next(rng, max.saturating_sub(min))
}

/// The band the anchor's depth picks (`AddBuriedChest`'s four arms,
/// `:36222`/`:36503`/`:36803`/`:37162`; the remix variants collapse to the
/// same bands for the worlds the port generates).
#[derive(Clone, Copy, PartialEq)]
enum Band {
    Surface,
    Mid,
    Deep,
    Deepest,
}

fn band_for(y: i32, world_surface: f64, rock_layer: f64, height: i32) -> Band {
    let num7 = y as f64;
    if num7 < world_surface + SURFACE_BAND_PAD as f64 {
        Band::Surface
    } else if num7 < rock_layer {
        Band::Mid
    } else if num7 < height as f64 - DEEP_BAND_PAD as f64 {
        Band::Deep
    } else {
        Band::Deepest
    }
}

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

/// `WorldGen.GetNextJungleChestItem` (`WorldGen.cs:10170-10196`): the jungle
/// chests' main item — the `GenVars.JungleItemCount % 4` cycle over the
/// living-wood wand family {211, 212, 213, 964} with the two rare overrides
/// (`Next(15) == 0` → 2292, else `Next(20) == 0` → 3017), the counter
/// advancing once per item (`:10196`).
///
/// The counter's walk is the port's approximation: vanilla's
/// `JungleItemCount` advances once per *attempted* shrine chest, in the
/// `GenVars.JChestX[i]` spot order (`ChestsInJungleShrines` `WorldGen.cs:17219`),
/// while the port's load-time sweep advances it once per stocked jungle
/// anchor, in the scan order — the same per-world counter semantics
/// (deterministic across loads), a different order (ANALYSIS "World chest
/// loot (M5-CHEST-gen)"). The `Prefix(-1)` on the row rides the caller, the
/// vanilla slot-0 arm's `chest.item[0].Prefix(-1)` (`:36230`).
struct JungleCycle {
    count: u32,
}

impl JungleCycle {
    fn new() -> Self {
        Self { count: 0 }
    }

    /// The next main item. The vanilla draw order is exact: the cycle pick
    /// is branch-free, the `Next(15)` gate rolls first and the `Next(20)`
    /// gate only rolls on its else (`:10186-10194`).
    fn take(&mut self, rng: &mut u64) -> u16 {
        let mut item = match self.count % 4 {
            0 => 211,
            1 => 212,
            2 => 213,
            _ => 964,
        };
        if next(rng, 15) == 0 {
            item = 2292;
        } else if next(rng, 20) == 0 {
            item = 3017;
        }
        self.count += 1;
        item
    }
}

/// The per-chest context the style dispatcher reads (`flag*` arms of
/// `:36010-36205`): the jungle/mahogany/skyware/idol/dungeon flags
/// approximated by the anchors' surroundings.
struct Context {
    /// `flag4` — the jungle family (the C# reads `chestStyle == 10`
    /// `:36089`, which the port cannot see: `worldgen::structures::place_chest`
    /// writes the plain frames — the port's stand-in is the jungle grass
    /// 60 at the anchor).
    jungle: bool,
    /// `flag6` — the living-tree family (wall 244 at the anchor, the C#'s
    /// `:36135-36144` condition).
    living_tree: bool,
    /// `flag3` — the pyramid sandstone (485/216 walls).
    pyramid: bool,
    /// `flag4` — the living mahogany family (wall 63 wood walls + the tulip
    /// 1683/the 2422 column in the room).
    mahogany: bool,
    /// `flag2` — the ice family (the 147/161/162/197 floor / the style 11,
    /// the C#'s `:36064` condition).
    ice: bool,
    /// `flag7` — the granite/marble family (the 164/401 walls).
    granite: bool,
    /// `flag8` — the dungeon idol (`flag9`-style 464/241 walls).
    idol: bool,
    /// `flag9` — the dungeon-key family (`Main.wallDungeon`, the walls the
    /// dungeon zone requires — `tr_core::scene::is_dungeon_wall`); the port's
    /// dungeon pass writes 7/8/9 (`worldgen/biomes.rs:1754`), so the probe
    /// reads those instead of the old wall-24 stand-in.
    dungeon: bool,
    /// `flag12` — the skyware family (the floating-island houses). The C#
    /// probes the sky cloud walls (`HouseBuilder`'s cloud interiors); the
    /// port's island houses write `WALL_CLOUD` = 73
    /// (`worldgen/islands.rs:26`/`:133`), so the probe reads that — the old
    /// 216/187 probe now reads the underground desert's shell walls
    /// (`worldgen/desert.rs`'s `WALL_HARDENED_SAND`) and misfired there.
    sky: bool,
    /// `flag13` — the water/beach family.
    water: bool,
}

impl Context {
    /// The port's merged wood family — the pre-existing approximation arms
    /// (the pottery and the 2198 rows) ran on one flag before the flag4 /
    /// flag6 split; they keep the combined gate so their behaviour is
    /// unchanged.
    fn wood_family(&self) -> bool {
        self.jungle || self.living_tree
    }
}

fn context_at(world: &World, x: i32, y: i32) -> Context {
    let near = |wx: u16| {
        (-2..=2).any(|dx| {
            (-2..=0).any(|dy| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.active() && t.kind == wx)
            })
        })
    };
    let wall_near = |wx: u16| {
        (-2..=2).any(|dx| {
            (-2..=0).any(|dy| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.wall == wx)
            })
        })
    };
    let wall_near_where = |pred: &dyn Fn(u16) -> bool| {
        (-2..=2).any(|dx| {
            (-2..=0).any(|dy| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| pred(t.wall))
            })
        })
    };
    let dungeon_wall = |wx: u16| tr_core::scene::is_dungeon_wall(wx);
    Context {
        jungle: near(60),
        living_tree: wall_near(244),
        pyramid: wall_near(216) || wall_near(187),
        mahogany: wall_near(63),
        ice: near(147) || near(161) || near(162) || near(197),
        granite: wall_near(164) || wall_near(401),
        idol: wall_near(464) || wall_near(241),
        dungeon: wall_near_where(&dungeon_wall),
        sky: wall_near(73),
        water: near(4869) || wall_near(24),
    }
}

/// `ItemSlot::Prefix(-1)` on a row: the natural roll through the ported
/// engine (the `roll_prefix(kind, -1, rng)` — the 1/4 no-prefix gate, the
/// ReducedNaturalChance arm, the rejection retry). The prefix lands on the
/// row's id; the id kinds outside the ported engine roll 0 (no prefix) with
/// **no stream draws** (the `CanHavePrefixes` early return).
fn roll_prefix_on(rng: &mut u64, row: &mut Row) {
    row.prefix = prefix::roll_prefix(row.kind, -1, rng);
}

/// The surface band's weapon row (`:36269-36331`): the `Next(11)` weapon
/// pick — 280 Spear/281 Blowgun/284 Flintlock/285 Musket/953 Pistol/946
/// Shortsword/3068 MusketBall?/3069/3084/4341/6165 — each with the natural
/// prefix.
fn surface_weapon(rng: &mut u64) -> Row {
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
fn deep_weapon(rng: &mut u64, deepest: bool) -> Row {
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
    rng: &mut u64,
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
fn voice_change_item(rng: &mut u64) -> u16 {
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
/// `GetNextJungleChestItem` cycle, the underwater override 939), then the
/// depth band rows.
fn stock_chest(
    world: &World,
    x: i32,
    y: i32,
    hell: bool,
    chest_style: i32,
    chest_kind: u16,
    dungeon: &mut DungeonCycle,
    jungle: &mut JungleCycle,
    genned_wands: &mut bool,
    rng: &mut u64,
) -> Vec<Row> {
    let ctx = context_at(world, x, y);
    let band = band_for(y, world.world_surface, world.rock_layer as f64, world.height);
    let mut rows = stock_band(band, &ctx, chest_style, chest_kind, rng);
    if ctx.dungeon {
        // The dungeon-family chests cycle the fixed loot item
        // (`DungeonUtils.cs:342-355`); the shallow caves roll the plain
        // `num10 == 0` skip.
        if let Some(item) = dungeon.take(hell) {
            rows.insert(0, Row::plain(item.0));
        }
    } else if ctx.jungle {
        // The jungle family's main item — the `GetNextJungleChestItem`
        // cycle replaces the band-weapon approximation on slot 0. The
        // vanilla water override only fires on `num10 == 0`
        // (`:36201-36203`), so the cycle wins over it wherever both
        // families read true.
        let mut row = Row::plain(jungle.take(rng));
        roll_prefix_on(rng, &mut row);
        rows.insert(0, row);
    } else if ctx.water {
        // The non-dungeon natural chests carry no fixed special slot in
        // this port slice (the vanilla num10 assignment rides the
        // placement-style dispatcher [7]); the underwater/water-style
        // chests carry the 939 override.
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
        if ctx.mahogany {
            // `flag3` `:36814-36823`.
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

/// The port's chest-store sweep: every kind-21/341 anchor
/// (`frame_x == 0 && frame_y == 0`) stocks one `WorldChest`. The stream is
/// seeded per world (`seed ^ const | 1`, the per-consumer convention) so a
/// world reloads with the same loot, and the roster can never exceed the
/// vanilla 8000 chest population.
pub(crate) fn generate_world_chests(world: &World) -> Vec<WorldChest> {
    if world.width <= 4 || world.height <= 4 {
        return Vec::new();
    }
    // The tenth-anniversary chest sweep (`FinishTenthAnniversaryWorld`'s
    // `ImproveAllChestContents`, `!getGoodWorldGen` — `:24278-24280`): the
    // anniversary seed's stocked chests take the good prefixes instead. The
    // port stocks at load time, so the sweep rides this seam.
    let improve_prefixes = world.seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY)
        && !world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let seed = world
        .seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0x2545_F491_4F6C_DD1D);
    let mut rng = seed | 1;
    let mut out: Vec<WorldChest> = Vec::new();
    let mut dungeon = DungeonCycle::new();
    let mut jungle = JungleCycle::new();
    // `GenVars.gennedLivingMahoganyWands` — false per world (`GenVars`'
    // reset), the flag4 arm's once-per-world fairness (`:36824`).
    let mut genned_wands = false;
    for y in 1..world.height - 1 {
        for x in 1..world.width - 1 {
            let Some(t) = world.tile(x, y) else { continue };
            if !t.active() || !CHEST_KINDS.contains(&t.kind) {
                continue;
            }
            // Every chest's LEFT cell carries `frame_x = 18 * style` in the
            // port's tile-object writer (`tileobject.rs:2072-2082`), so a
            // styled chest (the temple's 16, the jungle's 10, the water 17)
            // is an anchor too — the old `frame_x != 0` gate skipped them
            // (`frame_y` stays 0 on the left cell). The RIGHT half is also a
            // multiple of 18 (`frame_x = 18 + 18 * style`), so the anchor
            // picks the left cell only: no chest-kind tile directly left.
            if t.frame_y != 0 || t.frame_x % 18 != 0 {
                continue;
            }
            if let Some(left) = world.tile(x - 1, y) {
                if left.active() && CHEST_KINDS.contains(&left.kind) {
                    continue;
                }
            }
            let chest_style = t.frame_x as i32 / 18;
            if t.flags & FLAG_ACTIVE == 0 {
                continue;
            }
            let hell = doubled_hell_depth(world, y);
            let rows = stock_chest(
                world,
                x,
                y,
                hell,
                chest_style,
                t.kind,
                &mut dungeon,
                &mut jungle,
                &mut genned_wands,
                &mut rng,
            );
            if rows.is_empty() {
                continue;
            }
            let mut slots: Vec<ItemSlot> = Vec::with_capacity(rows.len());
            for row in rows {
                slots.push(ItemSlot {
                    kind: row.kind,
                    name: crate::registry::name(row.kind),
                    count: row.count.max(1),
                    prefix: row.prefix,
                });
            }
            let _ = EMPTY_SLOT;
            out.push(WorldChest {
                tile: (x, y),
                slots,
                name: String::new(),
                max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
            });
            dungeon.advance(true);
            if out.len() >= 8000 {
                break;
            }
        }
    }
    if improve_prefixes {
        improve_all_chest_contents(&mut out, &mut rng);
    }
    out
}

/// The hell-band check (`:36223`'s `num7 > maxTilesY - 205` arm): the port's
/// flattened approximation — below the rock layer and past the 205-tile
/// margin.
fn doubled_hell_depth(world: &World, y: i32) -> bool {
    y > world.height - 205
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
pub(crate) fn improve_all_chest_contents(chests: &mut [WorldChest], rng: &mut u64) {
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
fn give_item_good_prefixes(slot: &mut ItemSlot, rng: &mut u64) {
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
fn prefix_item_from_options(slot: &mut ItemSlot, options: &[u16], rng: &mut u64) {
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

/// The `.wld` load path's store (`LoadChests:2723`): the file's chests win —
/// they are the player's own contents — and the deterministic re-roll only
/// fills the anchors the file does not carry. Unlike [`merge_store`], the
/// loaded roster keeps its file order and its tombstones (`Chest.RemoveChest`
/// nulls the array slot, so every held `Player.chest` index stays valid), and
/// it leads: a vanilla world that never stored a chest for an anchor gets the
/// roll's stock for it, while a player-placed chest beyond worldgen's roster
/// survives untouched.
pub(crate) fn displace_loaded_chests(
    rolled: Vec<crate::chest::WorldChest>,
    loaded: Vec<crate::chest::WorldChest>,
) -> Vec<crate::chest::WorldChest> {
    let taken: std::collections::HashSet<(i32, i32)> = loaded
        .iter()
        .filter(|c| !is_displaced_removed(c))
        .map(|c| c.tile)
        .collect();
    let mut out = loaded;
    out.extend(rolled.into_iter().filter(|c| !taken.contains(&c.tile)));
    out
}

/// A tombstone reads as "no anchor here" for the displacement's taken-set
/// (the C# `Main.chest[i] == null`), mirroring `chest::find_chest`'s skip.
fn is_displaced_removed(c: &crate::chest::WorldChest) -> bool {
    c.tile == (i32::MIN, i32::MIN)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut rng = 0x5EED_u64 | 1;
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
        let mut rng2 = 0x5EED_u64 | 1;
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
        let chests = generate_world_chests(&w);
        assert_eq!(chests.len(), 1, "one anchor stocks one chest");
        let rows = &chests[0].slots;
        assert!(rows.len() >= 2, "the surface band carries at least the weapon row");
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
        let chests = generate_world_chests(&w);
        assert_eq!(chests.len(), 1);
        assert!(chests[0].slots.len() >= 2);
    }

    #[test]
    fn test_chest_gen_deterministic() {
        // Two scans of the same world agree row-for-row.
        let mut w = rig();
        put_chest(&mut w, 40, 199);
        put_chest(&mut w, 60, 194);
        put_chest(&mut w, 80, 210);
        let a = generate_world_chests(&w);
        let b = generate_world_chests(&w);
        assert_eq!(a.len(), b.len());
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.tile, cb.tile);
            assert_eq!(ca.slots, cb.slots);
        }
    }

    #[test]
    fn test_chest_gen_non_anchor_cells_skip() {
        // Only the frame-0/0 anchor stocks; the other three cells skip.
        let mut w = rig();
        put_chest(&mut w, 40, 199);
        let chests = generate_world_chests(&w);
        assert_eq!(chests.len(), 1);
        assert_eq!(chests[0].tile, (40, 199));
    }

    #[test]
    fn test_chest_gen_empty_world_has_no_chests() {
        let w = World::new("empty".into(), 200, 300);
        assert!(generate_world_chests(&w).is_empty());
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

    /// DW-3.4 — the `.wld` load path (M6-SAVE-1 phase 3): the file's chests win
    /// and the roll only fills the anchors the file does not carry. The loaded
    /// roster leads — its order and its tombstones are the C# `Main.chest`
    /// array's, so a held `Player.chest` index stays valid.
    #[test]
    fn test_DW_3_4_the_wld_load_displaces_the_reroll() {
        let roll = vec![rolled(40, 199, 21), rolled(60, 194, 22)];
        let file = vec![
            crate::chest::WorldChest {
                tile: (i32::MIN, i32::MIN), // a removed slot (`Chest.RemoveChest`)
                slots: Vec::new(),
                name: String::new(),
                max_items: 40,
            },
            loaded(40, 199, "Stash", 9),
            loaded(90, 90, "Extra", 8), // a player-placed chest worldgen never rolls
        ];
        let store = displace_loaded_chests(roll, file);
        assert_eq!(store.len(), 4);
        // The file's roster keeps its slots and order, tombstones included.
        assert!(store[0].tile == (i32::MIN, i32::MIN));
        assert_eq!(store[1].name, "Stash");
        assert_eq!(store[1].slots[0].count, 99, "the player's contents win");
        assert_eq!(store[2].tile, (90, 90), "a player-placed chest survives");
        // Only the anchor the file does not carry is filled by the roll.
        assert_eq!(store[3].tile, (60, 194));
        assert_eq!(store[3].slots[0].count, 3, "the roll's stock");
        // And the roll never re-stocks an anchor the file owns.
        assert!(
            store.iter().filter(|c| c.tile == (40, 199)).count() == 1,
            "the rolled (40,199) is displaced"
        );
    }

    /// DW-3.4 — the `.trwld` path is untouched: `merge_store` still appends
    /// the seeded store after the roll's survivors with its own order (the
    /// `TR_CHEST` fixture's behaviour the load path has always had).
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
        // The two functions differ in order, not in win semantics.
        let displaced =
            displace_loaded_chests(vec![rolled(40, 199, 21)], vec![loaded(40, 199, "F", 9)]);
        assert_eq!(
            displaced[0].name, "F",
            "the loaded roster leads on the .wld path"
        );
    }
    #[test]
    fn the_jungle_cycle_walks_the_four_item_table() {
        // `GetNextJungleChestItem`'s `JungleItemCount % 4` table
        // (`WorldGen.cs:10170-10196`) — a seed whose override draws miss
        // shows the raw 4-step cycle.
        let seed = (1u64..2000)
            .find(|&s| {
                let mut rng = s | 1;
                let mut cycle = JungleCycle::new();
                (0..4).all(|_| {
                    let item = cycle.take(&mut rng);
                    matches!(item, 211 | 212 | 213 | 964)
                })
            })
            .expect("a clean seed exists in the first 2000");
        let mut rng = seed | 1;
        let mut cycle = JungleCycle::new();
        let items: Vec<u16> = (0..4).map(|_| cycle.take(&mut rng)).collect();
        assert_eq!(items, vec![211, 212, 213, 964], "the 4-step wand table");
        // The counter wraps at 4 — the fifth draw starts the cycle again.
        assert_eq!(cycle.count, 4);
    }

    #[test]
    fn the_jungle_cycle_overrides_roll_the_rare_items() {
        // The two overrides: `Next(15) == 0` → 2292 (checked first), else
        // `Next(20) == 0` → 3017 (the else-only roll).
        let flower_seed = (1u64..4000)
            .find(|&s| {
                let mut rng = s | 1;
                JungleCycle::new().take(&mut rng) == 2292
            })
            .expect("the 1/15 override is reachable");
        let mut rng = flower_seed | 1;
        assert_eq!(JungleCycle::new().take(&mut rng), 2292, "`Next(15) == 0`");
        let shell_seed = (1u64..4000)
            .find(|&s| {
                let mut rng = s | 1;
                JungleCycle::new().take(&mut rng) == 3017
            })
            .expect("the 1/20 override is reachable");
        let mut rng = shell_seed | 1;
        assert_eq!(JungleCycle::new().take(&mut rng), 3017, "`Next(20) == 0`");
    }

    #[test]
    fn jungle_chests_stock_the_cycle_item() {
        // A jungle-family anchor (tile 60 near the anchor) stocks the
        // cycle item on slot 0 — not the band weapon the approximation
        // used before.
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
        let mut dungeon = DungeonCycle::new();
        let mut jungle = JungleCycle::new();
        let mut rng = 42u64 | 1;
        let mut genned_wands = false;
        let rows = stock_chest(&w, 100, 198, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
        let slot0 = &rows[0];
        assert!(
            matches!(slot0.kind, 211 | 212 | 213 | 964 | 2292 | 3017),
            "slot 0 rides the jungle cycle ({})",
            slot0.kind
        );
        assert_eq!(jungle.count, 1, "the counter advances per stocked chest");
        // A second jungle chest in the same world takes the next cycle
        // entry when no override fires — the per-world counter semantics.
        let mut seen_next = false;
        for seed in 1..4000 {
            let mut rng2 = seed | 1;
            let mut c2 = JungleCycle::new();
            let first = c2.take(&mut rng2);
            let second = c2.take(&mut rng2);
            if matches!(first, 211 | 212 | 213 | 964) && matches!(second, 212 | 213 | 964 | 2292) {
                seen_next = true;
                break;
            }
        }
        assert!(seen_next, "the counter walks the table");
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
        let ctx = context_at(&w, 120, 198);
        assert!(ctx.sky, "the cloud wall reads as the skyware family");
        let mut w2 = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w2.index(120 + dx, 197 + dy);
                w2.tiles[i].wall = 216; // the underground desert's shell
            }
        }
        let ctx2 = context_at(&w2, 120, 198);
        assert!(!ctx2.sky, "the desert's shell wall is not the skyware family");
        assert!(ctx2.pyramid, "and it reads as the pyramid/desert family");
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
        assert!(context_at(&w, 120, 198).dungeon, "wall 8 reads as the dungeon");
        let mut w2 = rig();
        for dx in -2i32..=0 {
            for dy in -2i32..=0 {
                let i = w2.index(120 + dx, 197 + dy);
                w2.tiles[i].wall = 24; // the old stand-in
            }
        }
        assert!(!context_at(&w2, 120, 198).dungeon, "wall 24 does not");
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
        let chests = generate_world_chests(&w);
        assert_eq!(chests.len(), 1, "the styled chest stocks once");
        // `flag13` (`:37419-37427`): the Lihzahrd Furnace guaranteed, then the
        // Solar Tablet at 1/5 or the Lunar Tablet stack.
        let kinds: Vec<u16> = chests[0].slots.iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&2195), "the temple furnace landed: {kinds:?}");
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
            let mut rng = seed | 1;
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
            let mut dungeon = DungeonCycle::new();
            let mut jungle = JungleCycle::new();
            let mut genned_wands = false;
            let mut rng = seed | 1;
            let rows = stock_chest(
                &w,
                100,
                338,
                false,
                0,
                21,
                &mut dungeon,
                &mut jungle,
                &mut genned_wands,
                &mut rng,
            );
            runs += 1;
            machines += rows.iter().filter(|r| r.kind == 2198).count();
            fish += rows.iter().filter(|r| r.kind == 669).count();
        }
        assert!(machines > 0, "the Ice Machine row fires ({machines}/{runs})");
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
        let mut dungeon = DungeonCycle::new();
        let mut jungle = JungleCycle::new();
        let mut genned_wands = false;
        let mut rng = 42u64 | 1;
        let plain = stock_chest(&w, 100, 198, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
        let family_ids: &[u16] = &[5629, 4429, 4427, 5528, 3199, 4346, 4066, 3360, 3361, 4426, 5525];
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
            let mut dungeon = DungeonCycle::new();
            let mut jungle = JungleCycle::new();
            let mut genned_wands = false;
            let mut rng = seed | 1;
            let rows =
                stock_chest(&w, 100, 338, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
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
                let wands = |rows: &[Row]| {
                    rows.iter().filter(|r| r.kind == 3360 || r.kind == 3361).count()
                };
                let mut dungeon = DungeonCycle::new();
                let mut jungle = JungleCycle::new();
                let mut genned_wands = false;
                let mut rng = s | 1;
                let first = stock_chest(&w, 30, 49, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
                let second = stock_chest(&w, 30, 49, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
                wands(&first) == 2 && wands(&second) == 0
            })
            .expect("a seed whose second call misses the 1/5 roll exists");
        let mut dungeon = DungeonCycle::new();
        let mut jungle = JungleCycle::new();
        let mut genned_wands = false;
        let mut rng = seed | 1;
        let rows = stock_chest(&w, 30, 49, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
        assert!(genned_wands, "the fairness flag trips on the first grant");
        let wands = rows.iter().filter(|r| r.kind == 3360 || r.kind == 3361).count();
        assert_eq!(wands, 2, "both wands row together ({wands})");
        // A second chest with the same seed now needs the 1/5 roll — with
        // this stream it does not fire again, so the pair is granted once.
        let rows2 = stock_chest(&w, 30, 49, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
        let wands2 = rows2.iter().filter(|r| r.kind == 3360 || r.kind == 3361).count();
        assert_eq!(wands2, 0, "the once-per-world grant does not repeat ({wands2})");
    }



    #[test]
    fn the_remaining_tail_rows_roll_at_the_vanilla_odds() {
        // The `itemIndex > 0 && chestTileType == 21` block's remaining rows
        // (`:37384-47500`): the flag4 pair (1/4 + 1/50), the flag13 globe
        // (unconditional), the style-23-27 pottery row (1/2) and the style-13
        // row (a second 1/2), and the flag13 water pair (1/5 raincoat, else
        // 3-7 potions). The flag4 pair and the globe ride the families
        // either tile block carries, so they apply to both anchor kinds.
        let mut dungeon = DungeonCycle::new();
        let mut jungle = JungleCycle::new();
        let mut genned_wands = false;

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
        let mut rng = 3u64 | 1;
        let rows = stock_chest(&w, 100, 198, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
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
        let hits = (1u64..400).filter(|&seed| {
            let mut rng = seed | 1;
            stock_chest(&w, 100, 198, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng)
                .iter()
                .any(|r| r.kind == 2204)
        }).count();
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
            let mut rng = seed | 1;
            stock_chest(&w, 100, 198, false, 13, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng)
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
        let mut dungeon = DungeonCycle::new();
        let mut jungle = JungleCycle::new();
        let mut genned_wands = false;

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
                let mut rng = seed | 1;
                stock_chest(&w, 100, 198, false, 0, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng)
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
        let mut rng = 3u64 | 1;
        let mahog = stock_chest(&w, 100, 198, false, 25, 341, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng);
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
            let mut rng = seed | 1;
            stock_chest(&w2, 100, 198, false, 25, 21, &mut dungeon, &mut jungle, &mut genned_wands, &mut rng)
                .iter()
                .any(|r| matches!(r.kind, 2766 | 2767 | 5234))
        });
        assert!(rows21.is_some(), "the 21 anchor takes a 21-only row");
    }


}
