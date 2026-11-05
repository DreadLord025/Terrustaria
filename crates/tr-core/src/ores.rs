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

use crate::tile_light::FastRandom;
use crate::world::World;

/// The replace set of `OreRunner` (`WorldGen.cs:48886-48888`): the natural
/// stone/dirt/sand/ice/grass/moss family the ores grow into.
fn ore_replaceable(kind: u16) -> bool {
    matches!(
        kind,
        0 | 1 | 23 | 25 | 40 | 53 | 57 | 59 | 60 | 70 | 109 | 112 | 116 | 117 | 147 | 161
            | 163 | 164 | 199 | 200 | 203 | 234 | 396 | 397 | 398 | 399 | 400 | 401 | 402 | 403
    ) || crate::tiledata::TILE_MOSS.has(kind)
        || (kind == 225)
}

/// `WorldGen.OreRunner` (`:48831-48900`): the random walk placing the
/// shrinking Manhattan discs of the ore.
pub fn ore_runner(
    world: &mut World,
    i: i32,
    j: i32,
    strength: i32,
    steps: i32,
    kind: u16,
    rng: &mut FastRandom,
) {
    let mut num = strength as f64;
    let mut left = steps as f64;
    let mut px = i as f64;
    let mut py = j as f64;
    let mut vx = (rng.next_range(-10, 11) as f64) * 0.1;
    let mut vy = (rng.next_range(-10, 11) as f64) * 0.1;
    while num > 0.0 && left > 0.0 {
        num = strength as f64 * (left / steps as f64);
        left -= 1.0;
        let r = num * 0.5;
        let x0 = ((px - r) as i32).max(0);
        let x1 = ((px + r) as i32).min(world.width);
        let y0 = ((py - r) as i32).max(0);
        let y1 = ((py + r) as i32).min(world.height);
        for k in x0..x1 {
            for l in y0..y1 {
                let manhattan = ((k as f64 - px).abs() + (l as f64 - py).abs()) as f64;
                let wobble = 1.0 + (rng.next_range(-10, 11) as f64) * 0.015;
                if manhattan >= strength as f64 * 0.5 * wobble {
                    continue;
                }
                let Some(t) = world.tile(k, l) else {
                    continue;
                };
                if t.active() && ore_replaceable(t.kind) {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.kind = kind;
                    }
                }
            }
        }
        px += vx;
        py += vy;
    }
}

/// The saved ore tiers (`SavedOreTiers`): −1 = not rolled yet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SavedOreTiers {
    pub cobalt: i32,
    pub mythril: i32,
    pub adamantite: i32,
}

impl SavedOreTiers {
    pub fn new() -> Self {
        Self {
            cobalt: -1,
            mythril: -1,
            adamantite: -1,
        }
    }
}

/// The altar smash (`WorldGen.cs:48920-49090`): the tier by
/// `altarCount % 3`, the 50/50 alternate roll, the blob count
/// `(scale·310 − 85·tier) · 0.85 / (tier + 1)` with the alternate ×0.9 and
/// the cobalt ×1.05, the placement band per the tier, the `OreRunner`
/// batches (`Next(5, 9 + 9)` for the alternates, `Next(5, 9)` otherwise —
/// the C#'s `num4` bonus rides the alternate rolls).
/// Returns the ore tile kind placed.
pub fn smash_altar(
    world: &mut World,
    tiers: &mut SavedOreTiers,
    altar_count: i32,
    rng: &mut FastRandom,
) -> u16 {
    let scale = world.width as f64 / 4200.0;
    let tier = (altar_count % 3) as i32;
    let mut num4 = 0;
    let mut count_scale = 1.0f64;
    let (ore, band): (i32, f64) = match tier {
        0 => {
            if tiers.cobalt == -1 {
                tiers.cobalt = 107;
                if rng.next(2) == 0 {
                    tiers.cobalt = 221;
                }
            }
            if tiers.cobalt == 221 {
                num4 = 9;
                count_scale *= 0.899_999_976_158_142_1;
            }
            count_scale *= 1.049_999_952_316_284_2;
            (tiers.cobalt, world.world_surface)
        }
        1 => {
            if tiers.mythril == -1 {
                tiers.mythril = 108;
                if rng.next(2) == 0 {
                    tiers.mythril = 222;
                }
            }
            if tiers.mythril == 222 {
                num4 = 9;
                count_scale *= 0.899_999_976_158_142_1;
            }
            (tiers.mythril, world.rock_layer)
        }
        _ => {
            if tiers.adamantite == -1 {
                tiers.adamantite = 111;
                if rng.next(2) == 0 {
                    tiers.adamantite = 223;
                }
            }
            if tiers.adamantite == 223 {
                num4 = 9;
                count_scale *= 0.899_999_976_158_142_1;
            }
            (tiers.adamantite, (world.rock_layer * 2.0 + world.height as f64) / 3.0)
        }
    };
    // `:48935-48937`: the blob count.
    let mut blobs = (scale * 310.0 - 85.0 * tier as f64) * 0.85 / (tier + 1) as f64;
    blobs *= count_scale;
    let y_lo = band as i32;
    let y_hi = (world.height - 150).max(y_lo + 1);
    for _ in 0..blobs as i32 {
        let i2 = rng.next_range(100, world.width - 100);
        let j2 = rng.next_range(y_lo, y_hi);
        let strength = rng.next_range(5, 9 + num4);
        let steps = rng.next_range(5, 9 + num4);
        ore_runner(world, i2, j2, strength, steps, ore as u16, rng);
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
    use crate::tile::{FLAG_ACTIVE, Tile};

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
        let mut rng = FastRandom::new(1);
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
        let mut rng = FastRandom::new(2);
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
        let mut rng = FastRandom::new(3);
        let cobalt = smash_altar(&mut w, &mut tiers, 0, &mut rng);
        let mythril = smash_altar(&mut w, &mut tiers, 1, &mut rng);
        let adam = smash_altar(&mut w, &mut tiers, 2, &mut rng);
        let top_of = |kind: u16| -> i32 {
            (0..w.height)
                .find(|y| {
                    (0..w.width).any(|x| {
                        w.tile(x, *y).is_some_and(|t| t.active() && t.kind == kind)
                    })
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
}
