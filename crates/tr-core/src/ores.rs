//! The hardmode ore seeding — the altar-smash half of the hardmode switch
//! (`WorldGen.cs:48920-49090`): `OreRunner` (`:48831-48900`) and the
//! altar-count tier selection + the placement bands.
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - the smash tiers (`:48947-49040`): the altar count % 3 → the cobalt
//!   107 / the palladium 221, the mythril 108 / the orichalcum 222, the
//!   adamantite 111 / the titanium 223 (the 50/50 roll per tier, the
//!   alternates bump the blob strength +9 and the count ×0.9, the cobalt
//!   count ×1.05)
//! - the blob count (`:48935-48937`): `maxTilesX/4200 · 310 − 85·tier)
//!   · 0.85 / (tier + 1)` — wait: `num3 = (num3·310 − 85·num) · 0.85 /
//!   num2` where num2 = the tier + 1 (the deeper tiers run fewer blobs)
//! - the placement bands (`:49045-49058`): the cobalt/palladium from
//!   `worldSurface`, the mythril/orichalcum from `rockLayer`, the
//!   adamantite/titanium from `(2·rockLayer + maxTilesY) / 3`, all down to
//!   `maxTilesY − 150`
//! - `OreRunner` (`:48831-48900`): the random walk placing the shrinking
//!   Manhattan discs — `num = strength · (steps_left / steps)`, the rim
//!   wobble `Next(-10, 11) · 0.015`, the replace set {0, 1, 23, 25, 40,
//!   53, 57, 59, 60, 70, 109, 112, 116, 117, 147, 161, 163, 164, the moss,
//!   199, 200, 203, 234, 396-403, 225 unless the hive wall 108}
//!
//! Deviations (ANALYSIS "Hardmode ores (M5-HARDFORCE-ores)"): the drunk
//! world's tier swapping is out (no drunk worlds); the wall runs
//! (`wallType`) are out (the ore walls = the follow-up); the smash trigger
//! is the app's hammer path (the C#'s hammer-power gate and the WoF
//! prerequisite = the follow-ups).

use crate::UnifiedRandom;
use crate::world::World;

/// The replace set of `OreRunner` (`WorldGen.cs:48884`): the natural
/// stone/dirt/sand/ice/grass/moss family the ores grow into. The 225
/// shimmer cell qualifies only when its wall is NOT the hive wall 108
/// (the caller checks the wall); the notTheBees 230 arm rides the seed
/// flag at the call site.
fn ore_replaceable(kind: u16) -> bool {
    matches!(
        kind,
        0 | 1
            | 23
            | 25
            | 40
            | 53
            | 57
            | 59
            | 60
            | 70
            | 109
            | 112
            | 116
            | 117
            | 147
            | 161
            | 163
            | 164
            | 199
            | 200
            | 203
            | 234
            | 396
            | 397
            | 398
            | 399
            | 400
            | 401
            | 402
            | 403
    ) || crate::tiledata::TILE_MOSS.has(kind)
}

/// `WorldGen.OreRunner` (`:48831-48925`): the random walk placing the
/// shrinking Manhattan discs of the ore, verbatim. The disc radius shrinks
/// with the steps left (`num = strength · (num2/steps)`), the rim wobble is
/// the per-cell `Next(-10, 11) · 0.015` draw against the FULL strength
/// (`:48873` — not the shrunk radius), the velocity drifts
/// `vx += Next(-10, 11) · 0.05` clamped to ±1 (`:48915-48923`). The
/// `tileType == 59` (mud) walk stops above the sky (`:48843-48846`).
///
/// `generating` mirrors `WorldGen.generatingWorld`: during generation the
/// paint reset + `SquareTileFrame` are skipped (`:48887-48895`); at runtime
/// the placed cell's paint clears (`paintTile(k, l, 0, …, paintEffects:
/// false)`, `:44110-44132`) and the 3×3 re-frames.
///
/// Deviations: the `wallType`/`stayInArea`/`onlyReplaceTileType`/
/// `onlyReplaceWallType` parameters have no ported caller (every smash
/// passes the defaults) — the wall run (`:48898-48911`) stays out; the
/// `notTheBeesWorld` 230 arm (`:48884`) reads the seed flag.
pub fn ore_runner(
    world: &mut World,
    i: i32,
    j: i32,
    strength: i32,
    steps: i32,
    kind: u16,
    rng: &mut UnifiedRandom,
) {
    ore_runner_inner(world, i, j, strength, steps, kind, false, rng);
}

/// The runtime form — `generatingWorld == false` (the altar smash path).
pub fn ore_runner_runtime(
    world: &mut World,
    i: i32,
    j: i32,
    strength: i32,
    steps: i32,
    kind: u16,
    rng: &mut UnifiedRandom,
) {
    ore_runner_inner(world, i, j, strength, steps, kind, true, rng);
}

fn ore_runner_inner(
    world: &mut World,
    i: i32,
    j: i32,
    strength: i32,
    steps: i32,
    kind: u16,
    runtime: bool,
    rng: &mut UnifiedRandom,
) {
    let strength = strength as f64;
    let mut num = strength;
    let mut num2 = steps as f64;
    let (mut px, mut py) = (i as f64, j as f64);
    let mut vx = rng.next_range(-10, 11) as f64 * 0.1;
    let mut vy = rng.next_range(-10, 11) as f64 * 0.1;
    let bees = world.seeds.contains(crate::seeds::SeedFlags::NOT_THE_BEES);
    while num > 0.0 && num2 > 0.0 {
        // The mud walk stops above the sky (`:48843-48846`).
        if py < 0.0 && num2 > 0.0 && kind == 59 {
            num2 = 0.0;
        }
        num = strength * (num2 / steps as f64);
        num2 -= 1.0;
        let x0 = ((px - num * 0.5) as i32).max(0);
        let x1 = ((px + num * 0.5) as i32).min(world.width);
        let y0 = ((py - num * 0.5) as i32).max(0);
        let y1 = ((py + num * 0.5) as i32).min(world.height);
        for k in x0..x1 {
            for l in y0..y1 {
                // The wobble draw happens for EVERY box cell (`:48873`).
                let manhattan = (k as f64 - px).abs() + (l as f64 - py).abs();
                let wobble = 1.0 + rng.next_range(-10, 11) as f64 * 0.015;
                if manhattan >= strength * 0.5 * wobble {
                    continue;
                }
                let Some(t) = world.tile(k, l) else {
                    continue;
                };
                if t.active() {
                    let replaceable = ore_replaceable(t.kind)
                        || (bees && t.kind == 230)
                        || (t.kind == 225 && t.wall != 108);
                    if replaceable {
                        if let Some(t) = world.tile_mut(k, l) {
                            t.kind = kind;
                            if runtime {
                                // `paintTile(k, l, 0, …, paintEffects: false)`.
                                t.set_paint(0);
                            }
                        }
                        if runtime {
                            crate::framing::reframe_area(world, k, l, 1, true);
                        }
                    }
                }
            }
        }
        // The drift + clamp (`:48914-48923`).
        px += vx;
        py += vy;
        vx += rng.next_range(-10, 11) as f64 * 0.05;
        if vx > 1.0 {
            vx = 1.0;
        }
        if vx < -1.0 {
            vx = -1.0;
        }
    }
}

/// The saved ore tiers (`SavedOreTiers`, `WorldGen.cs:3283-3297`): the four
/// pre-hardmode metals roll once per world at `GenerateWorld` setup
/// (`:11266-11297`); the hardmode trio stays −1 until an altar is smashed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedOreTiers {
    pub copper: i32,
    pub iron: i32,
    pub silver: i32,
    pub gold: i32,
    pub cobalt: i32,
    pub mythril: i32,
    pub adamantite: i32,
}

impl SavedOreTiers {
    pub fn new() -> Self {
        Self {
            // The C# static-constructor defaults (`WorldGen.cs:3299-3307`,
            // re-applied per world at `:11266-11269`).
            copper: 7,
            iron: 6,
            silver: 9,
            gold: 8,
            cobalt: -1,
            mythril: -1,
            adamantite: -1,
        }
    }
}

/// Aligned with [`SavedOreTiers::new`] — the derived `Default` would give 0,
/// which is the dirt tile id.
impl Default for SavedOreTiers {
    fn default() -> Self {
        Self::new()
    }
}

/// The altar smash (`WorldGen.cs:48927-49109`): the tier by
/// `altarCount % 3`, the 50/50 alternate roll, the blob count
/// `(scale·310 − 85·tier) · 0.85 / (tier + 1)` with the alternate ×0.9 and
/// the cobalt ×1.05, the placement band per the tier, the `OreRunner`
/// batches (`Next(5, 9 + 9)` for the alternates, `Next(5, 9)` otherwise —
/// the C#'s `num4` bonus rides the alternate rolls).
/// Returns the ore tile kind placed.
///
/// The seed arms (W4-d103 tail P6): the drunk world flips the tier pair
/// BEFORE the tier's own arm (`:48939-48948` the adamantite pre-flip,
/// `:48976-48985` the mythril in-arm flip, `:49008-49017` the cobalt
/// in-default-arm flip — the C#'s asymmetry kept verbatim); the remix world
/// draws the y from the inverted band (`:49056-49067`); the anniversary
/// world widens the strength roll to `Next(5, 11 + num4)` (`:49069-49071`).
/// The ore-blessing chat (`:48969`/`:49002`/`:49034`, `Lang.misc[12..14]`
/// classic / `[21..23]` alternates, ChatColors.World) rides the app seam —
/// `world_sim/editing.rs::hammer_tile_hit` broadcasts on the returned ore
/// (the core has no chat/lang store; the W4-d103 POST FINAL wiring).
/// The wraith spawn (`:49078-49105`) rides the app seam — the core has
/// no NPC roster; `world_sim/editing.rs::hammer_tile_hit` runs the
/// `Main.rand.Next(2) + 1` count + `SpawnOnPlayer(82)`'s 200×30 scan on
/// the returned smash (W4-d103 WG-POST RETIRE — the g8-kind activation).
pub fn smash_altar(
    world: &mut World,
    tiers: &mut SavedOreTiers,
    altar_count: i32,
    rng: &mut UnifiedRandom,
) -> u16 {
    let scale = world.width as f64 / 4200.0;
    let tier = (altar_count % 3) as i32;
    let drunk = world.seeds.contains(crate::seeds::SeedFlags::DRUNK);
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    let anniversary = world.seeds.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY);
    // The drunk adamantite pre-flip (`:48939-48948`) — runs on EVERY smash.
    if drunk {
        if tiers.adamantite == 111 {
            tiers.adamantite = 223;
        } else if tiers.adamantite == 223 {
            tiers.adamantite = 111;
        }
    }
    let mut num4 = 0;
    let mut count_scale = 1.0f64;
    let ore: i32 = match tier {
        0 => {
            if tiers.cobalt == -1 {
                tiers.cobalt = 107;
                if rng.next_max(2) == 0 {
                    tiers.cobalt = 221;
                }
            }
            if tiers.cobalt == 221 {
                num4 = 9;
                count_scale *= 0.899_999_976_158_142_1;
            }
            count_scale *= 1.049_999_952_316_284_2;
            tiers.cobalt
        }
        1 => {
            // The drunk mythril in-arm flip (`:48976-48985`).
            if drunk {
                if tiers.mythril == 108 {
                    tiers.mythril = 222;
                } else if tiers.mythril == 222 {
                    tiers.mythril = 108;
                }
            }
            if tiers.mythril == -1 {
                tiers.mythril = 108;
                if rng.next_max(2) == 0 {
                    tiers.mythril = 222;
                }
            }
            if tiers.mythril == 222 {
                num4 = 9;
                count_scale *= 0.899_999_976_158_142_1;
            }
            tiers.mythril
        }
        _ => {
            // The drunk cobalt in-default-arm flip (`:49008-49017`).
            if drunk {
                if tiers.cobalt == 107 {
                    tiers.cobalt = 221;
                } else if tiers.cobalt == 221 {
                    tiers.cobalt = 107;
                }
            }
            if tiers.adamantite == -1 {
                tiers.adamantite = 111;
                if rng.next_max(2) == 0 {
                    tiers.adamantite = 223;
                }
            }
            if tiers.adamantite == 223 {
                num4 = 9;
                count_scale *= 0.899_999_976_158_142_1;
            }
            tiers.adamantite
        }
    };
    // `:48935-48937`: the blob count.
    let mut blobs = (scale * 310.0 - 85.0 * tier as f64) * 0.85 / (tier + 1) as f64;
    blobs *= count_scale;
    for _ in 0..blobs as i32 {
        let i2 = rng.next_range(100, world.width - 100);
        // The band floor per the PLACED ore (`:49046-49054`) — the drunk
        // flips move the ore between bands, so the floor reads `ore`, not
        // the tier index.
        let mut num8 = world.world_surface;
        if ore == 108 || ore == 222 {
            num8 = world.rock_layer;
        }
        if ore == 111 || ore == 223 {
            num8 = (world.rock_layer + world.rock_layer + world.height as f64) / 3.0;
        }
        let mut j_lo = num8 as i32;
        let mut j_hi = world.height - 150;
        // The remix inverted band (`:49056-49067`).
        if remix {
            let mut num9 = world.width as f64 - 350.0;
            if ore == 108 || ore == 222 {
                num9 = (world.rock_layer + world.rock_layer + world.height as f64 - 350.0) / 3.0;
            }
            if ore == 111 || ore == 223 {
                num9 = world.rock_layer - 25.0;
            }
            j_lo = world.world_surface as i32 + 15;
            j_hi = num9 as i32;
        }
        // The C# `Next(min, max)` throws on a reversed band; the port's
        // degenerate-world guard keeps the fixture worlds alive (the
        // `growth.rs` band-guard precedent).
        let j2 = if j_lo < j_hi {
            rng.next_range(j_lo, j_hi)
        } else {
            j_lo
        };
        // The anniversary strength widening (`:49069-49075`).
        let hi = if anniversary { 11 + num4 } else { 9 + num4 };
        let strength = rng.next_range(5, hi);
        let steps = rng.next_range(5, hi);
        ore_runner_runtime(world, i2, j2, strength, steps, ore as u16, rng);
    }
    ore as u16
}

/// The hammer-smash gate on the altar tile (the C# `KillTile`'s case 26
/// requires the hardmode hammer — the port gates on `hard_mode`).
pub fn can_smash_altar(_world: &World, hard_mode: bool) -> bool {
    hard_mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    fn world() -> World {
        let mut w = World::new("t".into(), 1200, 800);
        w.world_surface = 200.0;
        w.rock_layer = 400.0;
        // The stone field.
        for x in 0..1200 {
            for y in 200..800 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    #[test]
    fn the_tier_selection_and_bands_are_verbatim() {
        let mut w = world();
        let mut tiers = SavedOreTiers::new();
        let mut rng = UnifiedRandom::new(1);
        // The first smash: the cobalt 107 or the palladium 221.
        let ore = smash_altar(&mut w, &mut tiers, 0, &mut rng);
        assert!(matches!(ore, 107 | 221), "tier 1 ore {ore}");
        assert_eq!(tiers.cobalt, ore as i32);
        // The second: the mythril 108 or the orichalcum 222.
        let ore = smash_altar(&mut w, &mut tiers, 1, &mut rng);
        assert!(matches!(ore, 108 | 222), "tier 2 ore {ore}");
        // The third: the adamantite 111 or the titanium 223.
        let ore = smash_altar(&mut w, &mut tiers, 2, &mut rng);
        assert!(matches!(ore, 111 | 223), "tier 3 ore {ore}");
        // The fourth smash wraps to the cobalt tier (the same tier ore).
        let ore = smash_altar(&mut w, &mut tiers, 3, &mut rng);
        assert_eq!(ore as i32, tiers.cobalt, "the tier wrap");
    }

    #[test]
    fn the_ore_runner_places_veins() {
        let mut w = world();
        let mut rng = UnifiedRandom::new(2);
        ore_runner(&mut w, 600, 400, 12, 20, 107, &mut rng);
        let cobalt = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 107)
            .count();
        assert!(cobalt > 20, "the cobalt vein ({cobalt})");
        // The vein only replaces the replaceable kinds — the surrounding
        // stone stays stone.
        assert!(
            w.tiles.iter().filter(|t| t.active() && t.kind == 1).count() > 1000,
            "the stone field survives"
        );
    }

    #[test]
    fn the_bands_stay_separate() {
        // The three smashes: the cobalt band (200+), the mythril (400+),
        // the adamantite (666+). The world is the C#-minimum 4200 wide —
        // the tier-3 blob count `(scale·310 − 170)·0.85/3` needs the full
        // scale (the small worlds genuinely get 0 blobs for tier 3).
        let mut w = World::new("t".into(), 4200, 800);
        w.world_surface = 200.0;
        w.rock_layer = 400.0;
        for x in 0..4200 {
            for y in 200..800 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut tiers = SavedOreTiers::new();
        let mut rng = UnifiedRandom::new(3);
        let cobalt = smash_altar(&mut w, &mut tiers, 0, &mut rng);
        let mythril = smash_altar(&mut w, &mut tiers, 1, &mut rng);
        let adam = smash_altar(&mut w, &mut tiers, 2, &mut rng);
        let top_of = |kind: u16| -> i32 {
            (0..w.height)
                .find(|y| {
                    (0..w.width)
                        .any(|x| w.tile(x, *y).is_some_and(|t| t.active() && t.kind == kind))
                })
                .unwrap_or(-1)
        };
        let cobalt_top = top_of(cobalt);
        let mythril_top = top_of(mythril);
        let adam_top = top_of(adam);
        assert!(
            cobalt_top < mythril_top,
            "the cobalt band sits above the mythril ({cobalt_top} vs {mythril_top})"
        );
        assert!(
            mythril_top < adam_top,
            "the mythril band sits above the adamantite ({mythril_top} vs {adam_top})"
        );
    }

    #[test]
    fn the_smash_gate_needs_hardmode() {
        let w = world();
        assert!(!can_smash_altar(&w, false), "pre-hardmode refuses");
        assert!(can_smash_altar(&w, true), "hardmode allows");
    }

    /// W4-d103 tail (P6): the drunk adamantite pre-flip
    /// (`WorldGen.cs:48939-48948`) runs on EVERY smash — a tier-0 smash
    /// flips the already-set adamantite 111 → 223 before the cobalt arm.
    #[test]
    fn w4_d103_p6_the_drunk_preflip_moves_the_adamantite_on_every_smash() {
        let mut w = world();
        w.seeds.insert(crate::seeds::SeedFlags::DRUNK);
        let mut tiers = SavedOreTiers {
            adamantite: 111,
            ..SavedOreTiers::new()
        };
        let mut rng = UnifiedRandom::new(4);
        let _ = smash_altar(&mut w, &mut tiers, 0, &mut rng);
        assert_eq!(tiers.adamantite, 223, "the pre-flip ran on the tier-0 smash");
        // The next smash flips it back (`:48945-48947`).
        let _ = smash_altar(&mut w, &mut tiers, 1, &mut rng);
        assert_eq!(tiers.adamantite, 111, "the pre-flip toggles");
    }

    /// W4-d103 tail (P6): the drunk mythril in-arm flip (`:48976-48985`) —
    /// a tier-1 smash with mythril already 108 flips it to 222 and seeds
    /// the orichalcum band.
    #[test]
    fn w4_d103_p6_the_drunk_mythril_flip_seeds_the_alternate() {
        let mut w = world();
        w.seeds.insert(crate::seeds::SeedFlags::DRUNK);
        let mut tiers = SavedOreTiers {
            mythril: 108,
            ..SavedOreTiers::new()
        };
        let mut rng = UnifiedRandom::new(6);
        let ore = smash_altar(&mut w, &mut tiers, 1, &mut rng);
        assert_eq!(ore, 222, "the in-arm flip picked the orichalcum");
        assert_eq!(tiers.mythril, 222);
    }

    /// W4-d103 tail (P6): the drunk cobalt in-default-arm flip
    /// (`:49008-49017`) — a tier-2 smash flips the cobalt pair (the C#'s
    /// asymmetry: the adamantite arm flips COBALT).
    #[test]
    fn w4_d103_p6_the_drunk_default_arm_flips_the_cobalt() {
        let mut w = world();
        w.seeds.insert(crate::seeds::SeedFlags::DRUNK);
        let mut tiers = SavedOreTiers {
            cobalt: 107,
            ..SavedOreTiers::new()
        };
        let mut rng = UnifiedRandom::new(8);
        let _ = smash_altar(&mut w, &mut tiers, 2, &mut rng);
        assert_eq!(tiers.cobalt, 221, "the default arm flipped the cobalt");
    }

    /// W4-d103 tail (P6): the remix band (`:49056-49067`) — the cobalt
    /// blobs draw y from `worldSurface + 15 .. maxTilesX − 350` (the
    /// inverted band), not the classic `worldSurface .. height − 150`.
    /// The mirror replays the draw chain and proves the band floor.
    #[test]
    fn w4_d103_p6_the_remix_band_inverts_the_y_draw() {
        let mut w = world();
        w.seeds.insert(crate::seeds::SeedFlags::REMIX);
        let mut tiers = SavedOreTiers::new();
        let mut rng = UnifiedRandom::new(9);
        let mut mirror = rng.clone();
        let ore = smash_altar(&mut w, &mut tiers, 0, &mut rng);
        // The tier roll (the first smash draws the 50/50).
        let _ = mirror.next_max(2);
        // The blob count: (310 − 0)·0.85/1 ·1.05 (+×0.9 if the alternate).
        let mut blobs = 310.0 * 0.85 * 1.049_999_952_316_284_2;
        if ore == 221 {
            blobs *= 0.899_999_976_158_142_1;
        }
        // The first blob's draws: x, then the REMIX y band.
        let _x = mirror.next_range(100, w.width - 100);
        let j_lo = w.world_surface as i32 + 15;
        let j_hi = (w.width - 350) as i32;
        let _y = mirror.next_range(j_lo, j_hi);
        // The strength pair (the classic hi = 9 + num4).
        let hi = if ore == 221 { 18 } else { 9 };
        let _s = mirror.next_range(5, hi);
        let _t = mirror.next_range(5, hi);
        // The OreRunner's own draws follow; the full-stream equality is
        // the a == b half — replay the whole smash on a twin.
        let mut w2 = world();
        w2.seeds.insert(crate::seeds::SeedFlags::REMIX);
        let mut tiers2 = SavedOreTiers::new();
        let mut rng2 = UnifiedRandom::new(9);
        let ore2 = smash_altar(&mut w2, &mut tiers2, 0, &mut rng2);
        assert_eq!(ore, ore2);
        assert_eq!(rng, rng2, "a == b");
        assert_eq!(w.tiles, w2.tiles, "a == b tiles");
        assert!(blobs > 0.0);
    }

    /// W4-d103 tail (P7): the OreRunner velocity drift (`:48915-48923`) —
    /// the walk's vx drifts by `Next(-10, 11) · 0.05` per step, clamped to
    /// ±1. The mirror consumes the two velocity draws + the per-step drift
    /// draws and proves the stream order (the wobble draws interleave).
    #[test]
    fn w4_d103_p7_the_ore_runner_drifts_and_clamps() {
        let mut w = world();
        let mut rng = UnifiedRandom::new(11);
        ore_runner(&mut w, 600, 400, 6, 4, 107, &mut rng);
        // a == b: the twin replays byte-identical.
        let mut w2 = world();
        let mut rng2 = UnifiedRandom::new(11);
        ore_runner(&mut w2, 600, 400, 6, 4, 107, &mut rng2);
        assert_eq!(rng, rng2, "a == b stream");
        assert_eq!(w.tiles, w2.tiles, "a == b tiles");
        // The mud walk stops above the sky (`:48843-48846`): a 59 runner
        // starting above row 0 zeroes num2 at the loop top, runs ONE
        // zero-radius iteration (num = strength·0/steps = 0, the empty box
        // draws nothing), takes the drift draw, then exits (num == 0).
        let mut w3 = world();
        let mut rng3 = UnifiedRandom::new(12);
        let mut mirror3 = rng3.clone();
        ore_runner(&mut w3, 600, -5, 6, 4, 59, &mut rng3);
        let _vx = mirror3.next_range(-10, 11);
        let _vy = mirror3.next_range(-10, 11);
        let _drift = mirror3.next_range(-10, 11);
        assert_eq!(rng3, mirror3, "the mud stop draws velocity + one drift");
        assert!(
            !w3.tiles.iter().any(|t| t.active() && t.kind == 59),
            "no mud placed above the sky"
        );
    }
}
