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

/// The per-chest context the style dispatcher reads (`flag*` arms of
/// `:36010-36205`): the jungle/mahogany/skyware/idol/dungeon flags
/// approximated by the anchors' surroundings.
struct Context {
    /// `flag2` — the deep-jungle wood family (tile 119/337 walls or the
    /// jungle grass at the anchor).
    jungle: bool,
    /// `flag3` — the pyramid sandstone (485/216 walls).
    pyramid: bool,
    /// `flag4` — the living mahogany family (wall 63 wood walls + the tulip
    /// 1683/the 2422 column in the room).
    mahogany: bool,
    /// `flag6` — the ice family (147/161/162/197 floor, the shrine bricks).
    ice: bool,
    /// `flag7` — the granite/marble family (the 164/401 walls).
    granite: bool,
    /// `flag8` — the dungeon idol (`flag9`-style 464/241 walls).
    idol: bool,
    /// `flag9` — the dungeon-key family (`wallDungeon[]`).
    dungeon: bool,
    /// `flag12` — the skyware family (the floating-island houses: the
    /// snow/ice walls + the 48...  the vanilla probes the sky cloud walls
    /// 216/187).
    sky: bool,
    /// `flag13` — the water/beach family.
    water: bool,
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
    Context {
        jungle: near(60) || wall_near(244),
        pyramid: wall_near(216) || wall_near(187),
        mahogany: wall_near(63),
        ice: near(147) || near(161) || near(162) || near(197),
        granite: wall_near(164) || wall_near(401),
        idol: wall_near(464) || wall_near(241),
        dungeon: wall_near(24),
        sky: wall_near(187) || wall_near(216),
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
fn stock_band(band: Band, ctx: &Context, rng: &mut u64) -> Vec<Row> {
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
    // `:37536-37540` — the wooden-chest extras (itemIndex > 0 && type 21):
    // the jungle/ice/furniture piles and the style-23-27 pottery.
    if ctx.jungle && next(rng, 2) == 0 {
        let pottery = match next(rng, 6) {
            0 => 5258,
            1 => 5226,
            2 => 5254,
            3 => 5238,
            4 => 5255,
            _ => 5388,
        };
        out.push(Row::stacked(pottery, 1));
    }
    if ctx.ice && next(rng, 2) == 0 {
        out.push(Row::stacked(2198, 1));
    }
    if ctx.jungle && next(rng, 2) == 0 {
        out.push(Row::stacked(2198, ranged(rng, 50, 101) as u32));
    }
    // `:37546` — the voice-change row (1/12).
    if next(rng, 12) == 0 {
        out.push(Row::plain(2195));
    }
    out
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
/// underwater override 939), then the depth band rows.
fn stock_chest(
    world: &World,
    x: i32,
    y: i32,
    hell: bool,
    dungeon: &mut DungeonCycle,
    rng: &mut u64,
) -> Vec<Row> {
    let ctx = context_at(world, x, y);
    let band = band_for(y, world.world_surface, world.rock_layer as f64, world.height);
    let mut rows = stock_band(band, &ctx, rng);
    if !ctx.dungeon {
        // The non-dungeon natural chests carry no fixed special slot in
        // this port slice (the vanilla num10 assignment rides the
        // placement-style dispatcher [7]); the underwater/water-style
        // chests carry the 939 override.
        if ctx.water {
            rows.insert(0, Row::plain(UNDERWATER_ITEM));
        }
    } else {
        // The dungeon-family chests cycle the fixed loot item
        // (`DungeonUtils.cs:342-355`); the shallow caves roll the plain
        // `num10 == 0` skip.
        if let Some(item) = dungeon.take(hell) {
            rows.insert(0, Row::plain(item.0));
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
    let seed = world
        .seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0x2545_F491_4F6C_DD1D);
    let mut rng = seed | 1;
    let mut out: Vec<WorldChest> = Vec::new();
    let mut dungeon = DungeonCycle::new();
    for y in 1..world.height - 1 {
        for x in 1..world.width - 1 {
            let Some(t) = world.tile(x, y) else { continue };
            if !t.active() || !CHEST_KINDS.contains(&t.kind) {
                continue;
            }
            if t.frame_x != 0 || t.frame_y != 0 {
                continue;
            }
            if t.flags & FLAG_ACTIVE == 0 {
                continue;
            }
            let hell = doubled_hell_depth(world, y);
            let rows = stock_chest(world, x, y, hell, &mut dungeon, &mut rng);
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
            });
            dungeon.advance(true);
            if out.len() >= 8000 {
                return out;
            }
        }
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
        });
    }
    out
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
}
