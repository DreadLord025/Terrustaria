//! W4-g41 finish gap 2: the remix world's own tail —
//! `WorldGen.FinishRemixWorld` (`WorldGen.cs:26350-26770`), dispatched from
//! `GenerateWorld`'s tail LAST of the classic tails (`:22444-22447`, after
//! `FinishGetGoodWorld`). Seam: `pipeline.rs::step_seed_finish`'s remix
//! slot; the pass draws from its own `pass_rng("FinishRemixWorld")`
//! stream, so a classic world's pipeline is untouched (the
//! `finish_seeds.rs` contract). d107's pushed ANALYSIS defers this host
//! ("`FinishRemixWorld`'s cloud/wall conversions) is deferred") — the
//! ~211-site inverted-terrain family stays deferred with that note; this
//! port is the Finish tail only (census
//! `docs/research/95-finish-no-traps-remix-census.md`).
//!
//! What the pass ports, arm by arm:
//!
//! - The 25-border cloud/sunplate sweep (`:26352-26432`): the deepest
//!   cloud row `num`, the flesh-195 / lesion-474 conversion under
//!   `!tenth || ftw` (the drunk `crimsonLeft` half-split `Next(-2,3)`),
//!   and the cloud-wall 73/82 → flesh-wall-77 / lesion-wall-233 arm.
//! - The 25-border conversion sweep (`:26433-26585`): the inland
//!   above-surface band (`l > num || errorWorld || surfaceIsInSpace`,
//!   between the `beachDistance` 380 margins) — wall-87 liquid → water,
//!   the FTW lava / NTB honey / drunk+anniversary shimmer drip arms with
//!   their kills; the per-cell `Next(3)` surface band — the anniversary
//!   `Convert(2)` + the 661/662 → 109 replacement with the `PlaceTile(110)`
//!   flower arm + the 7×7 mud kill, the drunk half-split `Convert(1/4)`,
//!   else `Convert(crimson ? 4 : 1)`; the per-cell `Next(19,22)`
//!   marble-178 / granite-180 wall clear; the obsidian-56 kill; the
//!   below-rock-layer active-dirt-0 → 2 fill with the 8-neighbour veto.
//! - The spawn-box liquid clear (`:26587-26632`, the `randomSpawn`
//!   micro-seed veto): the 0.38-0.62 × `maxTilesY-200..-125` box,
//!   hellstone 58 → ash 57, the 230/56/659 deactivates.
//! - `AddTrees(undergroundOnly: true)` (`:26633`, body `:31365-31447`):
//!   the two head `NextDouble` draws run even under `undergroundOnly`;
//!   the remix underground loop grows vanity 616/596 on grass-2 at 1/7
//!   (anniversary: always) else the kind-5 forest tree.
//! - The `!tenth || ftw` dungeon repaint (`:26634-26717`): per-column
//!   colour 22 (drunk half-split / crimson → 13), the dungeon bricks +
//!   cracked, the dungeon walls at wallColour 25 + the platform-19 arm,
//!   the anniversary living-wood / sandstone arms, the 226 / 61-on-226 /
//!   trap-137 colour2 band, the `!noSurface` wall-244 / wall-34 / wall-87
//!   arms.
//! - The ash-plants restyle (`:26718-26733`): `maxTilesX/4200 * Next(2,5)`
//!   clusters, kind 637 → frameX `18 * Next(6,11)` (`Main.rand == genRand`,
//!   `:4445` — the same stream).
//! - The NTB hive-wall strip (`:26734-26758`) and the full-map lily-pad
//!   `CheckLilyPad` sweep (`:26760-26769`, body `:59203-59309`).
//!
//! Deviations (the census pins them all):
//!
//! - **`Liquid.QuickWater(-2)` ×3** (`:26586`/`:26622`/`:26759`) is a
//!   documented no-op UNTIL the g47d settle drivers land — the `-2` is the
//!   C# `verbose` parameter (`Liquid.cs:104`, progress display only), so
//!   the activation call is `sys.quick_water(world, -1, -1)` (minY/maxY
//!   default −1 → rows `3..=height-3`, `:116-123`) on the gen settle
//!   driver's `LiquidSystem` (g47 `d222af69` base, research/70 step 6's
//!   recommended unification). The context is armed ONCE at the driver
//!   head: `set_gen_settle_context(gv.water_line, gv.lava_line, true)`
//!   (the remix band, `Liquid.cs:182`) + `set_gen_shimmer_position`
//!   (g36-A1's `gv.shimmer_position`) + `set_gen_dungeon_context` (the
//!   f24/c30 producer); the noTraps `SetBoulderSolidity` window
//!   (`:109-112`/`:139-142`) is subsumed by g47's pass-scoped
//!   `tilesIgnoreWater(true)`. The spawn-box loop then regains its
//!   `while flag` shape (the re-flood recheck reads the SETTLED box).
//!   Until then the recheck reads the cleared box and the loop runs
//!   ONCE: no settle means the state cannot change between iterations.
//!   Wire contract: `docs/research/95-finish-no-traps-remix-census.md`
//!   §"QuickWater activation addendum".
//! - **The tree bridge**: `AddTrees`' density rolls draw the host stream;
//!   the `TryGrowingTreeByType`/`GrowTree` calls take ONE `UnifiedRandom`
//!   seeded from it (the `not_the_bees.rs` tree-bridge precedent). The
//!   kind-5 arm calls `grow_tree` directly (the NTB `:25788` precedent).
//! - **`TileFrame`/`SquareTileFrame`** ride the post-pipeline framer (the
//!   M5-STRUCT-5 idiom) — the 82/83/84 herb re-frames and the pad-move
//!   re-frames draw nothing in-pass.
//! - **The `Convert` seam** is `spread::convert_cell_flavor` (M5-SPREAD-2c).
//!   The shared hallow map is GERunner's, a superset of `Convert`'s case 2
//!   in one arm that matters here: it maps the corrupt/crimson jungle
//!   grass 661/662 → 60, while `Convert(2)` (`:55176-55260`) has NO
//!   `Conversion.JungleGrass` arm — so the host RESTORES a 661/662 cell
//!   after the shared call, keeping the C# state for the `:26524`
//!   replacement arm. The other map differences ride the shared seam's
//!   own documented deviations.
//! - **`PlaceTile(110)`** ports the flower arm's wall-gated schedule
//!   (`:59507-59580` + `IsFitToPlaceFlowerIn` `:58231-58251`) verbatim —
//!   the 110 floor test is `{109, 492}`, the draws are `Next(50)` /
//!   `Next(35)` / `NextFromList(14)` / `Next(6)`.

use super::noise::Rng;
use super::pipeline::{GenVars, Streams};
use super::GenOptions;
use crate::edit::{kill_tile, kill_tile_legacy};
use crate::growth::{grow_tree, try_growing_tree_by_type};
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::spread::{convert_cell_flavor, ConversionFlavor};
use crate::tile::{LiquidKind, FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::tiledata as td;
use crate::world::World;
use crate::UnifiedRandom;

// --- the tile/wall ids the sweeps read --------------------------------------

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks.
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];
/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the nine dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

const TILE_DIRT: u16 = 0;
const TILE_GRASS: u16 = 2;
const TILE_PLATFORM: u16 = 19;
const TILE_OBSIDIAN: u16 = 56;
const TILE_ASH: u16 = 57;
const TILE_HELLSTONE: u16 = 58;
const TILE_MUD: u16 = 59;
const TILE_JUNGLE_GRASS: u16 = 60;
const TILE_JUNGLE_PLANTS: u16 = 61;
const TILE_HALLOWED_GRASS: u16 = 109;
const TILE_HALLOWED_PLANTS: u16 = 110;
const TILE_TRAPS: u16 = 137;
const TILE_SANDSTONE_BRICK: u16 = 151;
const TILE_LIVING_WOOD: u16 = 191;
const TILE_FLESH_BLOCK: u16 = 195;
const TILE_SUNPLATE: u16 = 202;
const TILE_LIHZAHRD_BRICK: u16 = 226;
const TILE_HONEY_BLOCK: u16 = 229;
const TILE_CRISPY_HONEY: u16 = 230;
const TILE_WATER_DRIP: u16 = 373;
const TILE_LAVA_DRIP: u16 = 374;
const TILE_HONEY_DRIP: u16 = 375;
const TILE_LILY_PAD: u16 = 518;
const TILE_ASH_PLANTS: u16 = 637;
const TILE_SHIMMER_BLOCK: u16 = 659;
const TILE_CORRUPT_JUNGLE: u16 = 661;
const TILE_CRIMSON_JUNGLE: u16 = 662;
const TILE_SHIMMER_DRIP: u16 = 709;
const TILE_LESION_BLOCK: u16 = 474;

const WALL_EBONSTONE_UNSAFE: u16 = 3;
const WALL_SANDSTONE_BRICK: u16 = 34;
const WALL_CLOUD: u16 = 73;
const WALL_FLESH: u16 = 77;
const WALL_DISC: u16 = 82;
const WALL_CRIMSTONE_UNSAFE: u16 = 83;
const WALL_HIVE_UNSAFE: u16 = 86;
const WALL_LIHZAHRD_BRICK: u16 = 87;
const WALL_MARBLE_UNSAFE: u16 = 178;
const WALL_GRANITE_UNSAFE: u16 = 180;
const WALL_LESION_BLOCK: u16 = 233;
const WALL_LIVING_WOOD_UNSAFE: u16 = 244;

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`).
const BEACH_DISTANCE: i32 = 380;

/// `WallID.Sets.AllowsPlantsToGrow` (`WallID.cs:97`) — hand-transcribed
/// (the `trees.rs` precedent; the generated tables carry no such set).
const ALLOWS_PLANTS_TO_GROW: [u16; 27] = [
    0, 150, 138, 145, 107, 152, 140, 139, 141, 106, 245, 315, 317, 63, 64, 65, 66, 67, 68, 69, 81, 70,
    264, 268, 265, 74, 80,
];

/// The `PlaceTile(110)` wall-arm `NextFromList` (`:59560`) — 14 entries.
const PLANT_FRAMES: [i32; 14] = [6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];

// --- small helpers ----------------------------------------------------------

fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// `WorldGen.SolidTile` (`:70111-70132`) — the boot table (no window open
/// during this tail; 379 reads solid).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

fn set_kind(world: &mut World, x: i32, y: i32, kind: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
    }
}

// --- the pass body ----------------------------------------------------------

/// `WorldGen.FinishRemixWorld` (`WorldGen.cs:26350-26770`): the remix
/// world's tail — the cloud/wall evil conversions, the surface conversion
/// band, the spawn-box clear, the underground trees, the dungeon repaint,
/// the ash-plants restyle, the NTB hive-wall strip and the lily-pad sweep.
pub(crate) fn finish_remix_world(world: &mut World, gv: &GenVars, streams: &mut Streams, opts: &GenOptions) {
    let flags = opts.flags;
    if !flags.contains(SeedFlags::REMIX) {
        return;
    }
    let mut rng = streams.pass_rng("FinishRemixWorld");
    // The tree bridge (the module note): one UnifiedRandom seeded from the
    // host stream, shared by every AddTrees growth call.
    let mut tree_rng = UnifiedRandom::new(rng.next_u32() as i32);
    let (width, height) = (world.width, world.height);
    let micro = opts.micro_flags;
    let drunk = flags.contains(SeedFlags::DRUNK);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let ftw = flags.contains(SeedFlags::FOR_THE_WORTHY);
    let ntb = flags.contains(SeedFlags::NOT_THE_BEES);
    let no_surface = micro.contains(MicroSeedFlags::NO_SURFACE);
    let crimson_left = gv.crimson_left;

    // `:26352-26432` — the cloud/sunplate sweep (records the deepest row).
    let mut num = 0i32;
    for i in 25..width - 25 {
        for j in 25..height - 25 {
            let (kind, active, wall) = {
                let t = &world.tiles[world.index(i, j)];
                (t.kind, t.active(), t.wall)
            };
            if active && (td::SET_CLOUDS.has(kind) || kind == TILE_SUNPLATE) {
                if j > num {
                    num = j;
                }
                // `:26363-26395` — the tile conversion (!tenth || ftw).
                if !tenth || ftw {
                    let new_kind = if drunk {
                        let left = i < width / 2 + rng.range(-2, 3);
                        if crimson_left {
                            if left { TILE_FLESH_BLOCK } else { TILE_LESION_BLOCK }
                        } else if left {
                            TILE_LESION_BLOCK
                        } else {
                            TILE_FLESH_BLOCK
                        }
                    } else if opts.crimson {
                        TILE_FLESH_BLOCK
                    } else {
                        TILE_LESION_BLOCK
                    };
                    set_kind(world, i, j, new_kind);
                }
            }
            // `:26397-26430` — the cloud-wall arm (the continue gate).
            if (wall != WALL_CLOUD && wall != WALL_DISC) || (tenth && !ftw) {
                continue;
            }
            let new_wall = if drunk {
                let left = i < width / 2 + rng.range(-2, 3);
                if crimson_left {
                    if left { WALL_FLESH } else { WALL_LESION_BLOCK }
                } else if left {
                    WALL_LESION_BLOCK
                } else {
                    WALL_FLESH
                }
            } else if opts.crimson {
                WALL_FLESH
            } else {
                WALL_LESION_BLOCK
            };
            if let Some(t) = world.tile_mut(i, j) {
                t.wall = new_wall;
            }
        }
    }

    // `:26433-26585` — the conversion sweep.
    let conversion_flavor = if opts.crimson {
        ConversionFlavor::Crimson
    } else {
        ConversionFlavor::Corruption
    };
    let error_world = micro.contains(MicroSeedFlags::ERROR_WORLD);
    let surface_in_space = micro.contains(MicroSeedFlags::SURFACE_IS_IN_SPACE);
    for k in 25..width - 25 {
        for l in 25..height - 25 {
            // `:26442-26516` — the inland above-surface band.
            if (l > num || error_world || surface_in_space)
                && (l as f64) < world.world_surface
                && k > BEACH_DISTANCE
                && k < width - BEACH_DISTANCE
            {
                remix_band_cell(world, k, l, ftw, ntb, drunk, tenth);
            }
            // `:26517-26571` — the surface conversion band (the Next(3)
            // draw runs for EVERY cell — the C# computes the condition
            // first).
            if (l as f64) < world.world_surface + rng.range(0, 3) as f64 {
                if tenth && !ftw {
                    // `:26519-26543` — the anniversary Hallow arm.
                    let wall = wall_at(world, k, l);
                    if wall != WALL_EBONSTONE_UNSAFE && wall != WALL_CRIMSTONE_UNSAFE {
                        let pre_kind = kind_at(world, k, l);
                        convert_cell_flavor(world, k, l, ConversionFlavor::Hallow, true);
                        // The case-2 reconciliation (the module note): the
                        // shared hallow map purifies 661/662 → 60, but
                        // Convert(2) has no JungleGrass arm — restore.
                        if matches!(pre_kind, TILE_CORRUPT_JUNGLE | TILE_CRIMSON_JUNGLE)
                            && kind_at(world, k, l) == TILE_JUNGLE_GRASS
                        {
                            set_kind(world, k, l, pre_kind);
                        }
                        let kind = kind_at(world, k, l);
                        if kind == TILE_CORRUPT_JUNGLE || kind == TILE_CRIMSON_JUNGLE {
                            set_kind(world, k, l, TILE_HALLOWED_GRASS);
                            // SquareTileFrame rides the framer (module note).
                            if solid_tile(world, k, l) && !active_at(world, k, l - 1) {
                                place_hallow_plant(world, &mut rng, k, l);
                            }
                            // `:26532-26541` — the 7×7 mud kill.
                            for m in k - 3..=k + 3 {
                                for n in l - 3..=l + 3 {
                                    if kind_at(world, m, n) == TILE_MUD {
                                        set_kind(world, m, n, TILE_DIRT);
                                    }
                                }
                            }
                        }
                    }
                } else if drunk {
                    // `:26545-26566` — the drunk half-split.
                    let left = k < width / 2 + rng.range(-2, 3);
                    let flavor = if crimson_left {
                        if left { ConversionFlavor::Crimson } else { ConversionFlavor::Corruption }
                    } else if left {
                        ConversionFlavor::Corruption
                    } else {
                        ConversionFlavor::Crimson
                    };
                    convert_cell_flavor(world, k, l, flavor, true);
                } else {
                    // `:26567-26570` — the world-flavour conversion.
                    convert_cell_flavor(world, k, l, conversion_flavor, true);
                }
            }
            // `:26572-26575` — the marble/granite wall clear (the
            // Next(19,22) draw runs for EVERY cell).
            if (l as f64) < world.world_surface - rng.range(19, 22) as f64 {
                let wall = wall_at(world, k, l);
                if wall == WALL_MARBLE_UNSAFE || wall == WALL_GRANITE_UNSAFE {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.wall = 0;
                    }
                }
            }
            // `:26576-26579` — the obsidian kill.
            if active_at(world, k, l) && kind_at(world, k, l) == TILE_OBSIDIAN {
                kill_tile_legacy(world, k, l, true);
            }
            // `:26580-26583` — the below-rock-layer dirt fill.
            if (l as f64) > world.rock_layer
                && l < height - 350
                && kind_at(world, k, l) == TILE_DIRT
                && active_at(world, k, l)
                && (!active_at(world, k - 1, l - 1)
                    || !solid_tile(world, k, l - 1)
                    || !active_at(world, k + 1, l - 1)
                    || !active_at(world, k - 1, l)
                    || !active_at(world, k + 1, l)
                    || !active_at(world, k - 1, l + 1)
                    || !active_at(world, k, l + 1)
                    || !active_at(world, k + 1, l + 1))
            {
                set_kind(world, k, l, TILE_GRASS);
            }
        }
    }

    // `:26586` — Liquid.QuickWater(-2): the documented no-op (module note).
    // g47d ACTIVATION SEAM 1/3: `sys.quick_water(world, -1, -1)` here once
    // the settle drivers land (`-2` is the C# verbose param; the gen
    // context is armed at the driver head — research/95 addendum).

    // `:26587-26632` — the spawn-box liquid clear. The randomSpawn micro
    // seed vetoes the whole loop; without a settle engine the re-flood
    // recheck reads the cleared box, so the loop runs ONCE (module note).
    if !micro.contains(MicroSeedFlags::RANDOM_SPAWN) {
        let x0 = (width as f64 * 0.38) as i32;
        let x1 = (width as f64 * 0.62) as i32 + 15;
        for x in x0..x1 {
            for y in height - 200..height - 125 {
                let Some(t) = world.tile_mut(x, y) else {
                    continue;
                };
                t.liquid = 0;
                if t.kind == TILE_HELLSTONE {
                    t.kind = TILE_ASH;
                }
                if matches!(t.kind, TILE_CRISPY_HONEY | TILE_OBSIDIAN | TILE_SHIMMER_BLOCK) {
                    t.flags &= !FLAG_ACTIVE;
                }
            }
        }
        // QuickWater(-2) + the spawn-column recheck: no settle, no state
        // change — the C# loop would exit here on the cleared box.
        // g47d ACTIVATION SEAM 2/3: `sys.quick_water(world, -1, -1)` then
        // the `:26624-26631` recheck loop (spawn_x column, spawn_y-14..
        // spawn_y+10: `liquid > 0` re-arms `flag`) — restore the C#
        // `while flag` shape when the settle drivers land (research/95
        // addendum; the module note carries the exact contract).
    }

    // `:26633` — AddTrees(undergroundOnly: true) (`:31365-31447`).
    add_trees_underground(world, &mut rng, &mut tree_rng, tenth);

    // `:26634-26717` — the dungeon repaint (!tenth || ftw).
    if !tenth || ftw {
        dungeon_repaint(world, drunk, crimson_left, opts.crimson, tenth, no_surface);
    }

    // `:26718-26733` — the ash-plants restyle.
    let clusters = width as f64 / 4200.0 * rng.range(2, 5) as f64;
    let mut c = 0i32;
    while (c as f64) < clusters {
        let cx = rng.range((width as f64 * 0.39) as i32, (width as f64 * 0.61) as i32);
        let r = rng.range(10, 31);
        for x in cx - r..=cx + r {
            for y in height - 250..height - 25 {
                if kind_at(world, x, y) == TILE_ASH_PLANTS {
                    let fx = (18 * rng.range(6, 11)) as i16;
                    if let Some(t) = world.tile_mut(x, y) {
                        t.frame_x = fx;
                    }
                }
            }
        }
        c += 1;
    }

    // `:26734-26758` — the NTB hive-wall strip.
    if ntb {
        hive_wall_strip(world);
    }

    // `:26759` — QuickWater(-2): the documented no-op.
    // g47d ACTIVATION SEAM 3/3: `sys.quick_water(world, -1, -1)` here
    // once the settle drivers land (research/95 addendum).

    // `:26760-26769` — the full-map lily-pad sweep.
    for x in 0..width {
        for y in 0..height {
            if kind_at(world, x, y) == TILE_LILY_PAD {
                check_lily_pad(world, x, y);
            }
        }
    }
}

/// The inland band cell (`:26444-26515`): the wall-87 water arm, then the
/// FTW / NTB / drunk+anniversary drip arms (mutually exclusive else-if).
fn remix_band_cell(world: &mut World, k: i32, l: i32, ftw: bool, ntb: bool, drunk: bool, tenth: bool) {
    let (wall, liquid, kind) = {
        let Some(t) = world.tile(k, l) else {
            return;
        };
        (t.wall, t.liquid, t.kind)
    };
    if wall == WALL_LIHZAHRD_BRICK {
        if liquid > 0 {
            if let Some(t) = world.tile_mut(k, l) {
                t.set_liquid_kind(LiquidKind::Water);
            }
        }
        return;
    }
    if ftw {
        // `:26451-26473` — the FTW lava arm.
        if liquid > 0 {
            if td::TILE_LAVA_DEATH.has(kind) {
                kill_tile_legacy(world, k, l, true);
            }
            if let Some(t) = world.tile_mut(k, l) {
                t.set_liquid_kind(LiquidKind::Lava);
            }
        }
        if matches!(kind, TILE_HONEY_DRIP | TILE_WATER_DRIP | TILE_SHIMMER_DRIP) {
            set_kind(world, k, l, TILE_LAVA_DRIP);
        }
        if matches!(kind, TILE_CRISPY_HONEY | TILE_HONEY_BLOCK | TILE_SHIMMER_BLOCK | TILE_OBSIDIAN) {
            kill_tile_legacy(world, k, l, true);
        }
        // The 82/83/84 TileFrame rides the framer (module note).
    } else if ntb {
        // `:26474-26492` — the NTB honey arm.
        if liquid > 0 {
            if let Some(t) = world.tile_mut(k, l) {
                t.set_liquid_kind(LiquidKind::Honey);
            }
        }
        if matches!(kind, TILE_LAVA_DRIP | TILE_WATER_DRIP | TILE_SHIMMER_DRIP) {
            set_kind(world, k, l, TILE_HONEY_DRIP);
        }
        if matches!(kind, TILE_CRISPY_HONEY | TILE_HONEY_BLOCK | TILE_SHIMMER_BLOCK | TILE_OBSIDIAN) {
            kill_tile_legacy(world, k, l, true);
        }
    } else if drunk && tenth {
        // `:26493-26515` — the drunk+anniversary shimmer arm.
        if liquid > 0 {
            if td::TILE_LAVA_DEATH.has(kind) {
                kill_tile_legacy(world, k, l, true);
            }
            if let Some(t) = world.tile_mut(k, l) {
                t.set_liquid_kind(LiquidKind::Shimmer);
            }
        }
        if matches!(kind, TILE_LAVA_DRIP | TILE_WATER_DRIP | TILE_HONEY_DRIP) {
            set_kind(world, k, l, TILE_SHIMMER_DRIP);
        }
        if matches!(kind, TILE_CRISPY_HONEY | TILE_HONEY_BLOCK | TILE_SHIMMER_BLOCK | TILE_OBSIDIAN) {
            kill_tile_legacy(world, k, l, true);
        }
    }
}

/// `WorldGen.PlaceTile(k, l, 110)`'s flower arm (`:59507-59580`): the
/// `IsFitToPlaceFlowerIn` floor test (`:58231-58251` — for 110 the floor
/// must be 109/492, active, slope 0, !halfBrick), then the wall-gated
/// schedule over `AllowsPlantsToGrow` on BOTH walls. The cell itself is
/// the just-converted 109 grass (active) — the arm overwrites it with the
/// plant, exactly like the C#.
fn place_hallow_plant(world: &mut World, rng: &mut Rng, k: i32, l: i32) {
    if l < 1 || l > world.height - 1 {
        return;
    }
    let floor_ok = world
        .tile(k, l + 1)
        .is_some_and(|t| t.active() && t.slope() == 0 && !t.half_brick() && matches!(t.kind, 109 | 492));
    if !floor_ok {
        return;
    }
    let wall = wall_at(world, k, l);
    let floor_wall = wall_at(world, k, l + 1);
    if !ALLOWS_PLANTS_TO_GROW.contains(&wall) || !ALLOWS_PLANTS_TO_GROW.contains(&floor_wall) {
        return;
    }
    // `:59544` — Next(50)==0 (the 24/201 Next(40) clause short-circuits).
    let frame_x = if rng.range(0, 50) == 0 {
        144
    } else if rng.range(0, 35) == 0 || (63..=70).contains(&wall) {
        // `:59560` — NextFromList over the 14-entry list.
        18 * PLANT_FRAMES[rng.range(0, PLANT_FRAMES.len() as i32) as usize]
    } else {
        // `:59576` — Next(6) * 18.
        18 * rng.range(0, 6)
    };
    if let Some(t) = world.tile_mut(k, l) {
        t.flags |= FLAG_ACTIVE;
        t.kind = TILE_HALLOWED_PLANTS;
        t.frame_x = frame_x as i16;
    }
}

/// `WorldGen.AddTrees(undergroundOnly: true)` (`:31365-31447`): the two
/// head `NextDouble` draws run even under `undergroundOnly` (the surface
/// loop is skipped); the remix underground loop walks
/// `worldSurface..maxTilesY-300` growing vanity trees on grass-2 at 1/7
/// (anniversary: always) else the kind-5 forest tree.
fn add_trees_underground(world: &mut World, rng: &mut Rng, tree_rng: &mut UnifiedRandom, tenth: bool) {
    // `:31366-31367` — the two beach densities, drawn and discarded.
    let _palm_left = 0.1 + rng.next_f64() * 0.35;
    let _palm_right = 0.1 + rng.next_f64() * 0.35;
    // `:31420-31446` — the remix-only underground loop.
    let y0 = world.world_surface as i32;
    let y1 = world.height - 300;
    for k in 1..world.width - 1 {
        for l in y0..y1 {
            let grass = world.tile(k, l).is_some_and(|t| t.active() && t.kind == TILE_GRASS);
            if grass && (tenth || rng.range(0, 7) == 0) {
                if rng.range(0, 2) == 0 {
                    try_growing_tree_by_type(world, 616, k, l, tree_rng);
                } else {
                    try_growing_tree_by_type(world, 596, k, l, tree_rng);
                }
            } else {
                // The kind-5 arm dispatches to GrowTree (the NTB precedent).
                grow_tree(world, k, l, tree_rng);
            }
        }
    }
}

/// The dungeon repaint (`:26634-26717`): the per-column colour pair, the
/// brick/wall/platform arms and the colour2 band.
fn dungeon_repaint(world: &mut World, drunk: bool, crimson_left: bool, crimson: bool, tenth: bool, no_surface: bool) {
    let (width, height) = (world.width, world.height);
    for x in 10..width - 10 {
        let mut color = 22u8;
        let mut color2 = 22u8;
        if drunk {
            if (crimson_left && x < width / 2) || (!crimson_left && x > width / 2) {
                color = 13;
                color2 = 13;
            }
        } else if crimson {
            color = 13;
            color2 = 13;
        }
        for y in 10..height - 10 {
            let i = world.index(x, y);
            let (kind, active, wall, frame_y) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall, t.frame_y)
            };
            if active && (TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind)) {
                world.tiles[i].set_paint(color);
            }
            if WALL_DUNGEON.contains(&wall) {
                world.set_wall_paint(x, y, 25);
                // `:26662-26665` — the platform arm reads the kind even on
                // an inactive cell (the C# has no active gate here).
                if kind == TILE_PLATFORM && frame_y != 180 {
                    world.tiles[i].set_paint(color);
                }
            }
            if active {
                let mut flag2 = false;
                if tenth {
                    if !no_surface && kind == TILE_LIVING_WOOD {
                        flag2 = true;
                    }
                    if kind == TILE_SANDSTONE_BRICK {
                        world.tiles[i].set_paint(color);
                    }
                }
                if kind == TILE_LIHZAHRD_BRICK {
                    flag2 = true;
                }
                if kind == TILE_JUNGLE_PLANTS && kind_at(world, x, y + 1) == TILE_LIHZAHRD_BRICK {
                    flag2 = true;
                }
                if kind == TILE_TRAPS && (1..=4).contains(&(frame_y as i32 / 18)) {
                    flag2 = true;
                }
                if flag2 {
                    world.tiles[i].set_paint(color2);
                }
            }
            if !no_surface && wall == WALL_LIVING_WOOD_UNSAFE {
                world.set_wall_paint(x, y, 25);
            }
            if wall == WALL_SANDSTONE_BRICK {
                world.set_wall_paint(x, y, 25);
            }
            if wall == WALL_LIHZAHRD_BRICK {
                world.set_wall_paint(x, y, 25);
                world.tiles[i].set_paint(color2);
            }
        }
    }
}

/// The NTB hive-wall strip (`:26734-26758`): per column above the surface,
/// wall 86 clears until the first active tile; the 5-down/2-side
/// wall+active pattern re-arms the clear.
fn hive_wall_strip(world: &mut World) {
    for x in 3..world.width - 3 {
        let mut clearing = true;
        let mut y = 0i32;
        while (y as f64) < world.world_surface {
            if clearing {
                if wall_at(world, x, y) == WALL_HIVE_UNSAFE {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = 0;
                    }
                }
                if active_at(world, x, y) {
                    clearing = false;
                }
            } else {
                let walls_clear = (0..5).all(|d| wall_at(world, x, y + d) == 0)
                    && wall_at(world, x - 1, y) == 0
                    && wall_at(world, x + 1, y) == 0
                    && wall_at(world, x - 2, y) == 0
                    && wall_at(world, x + 2, y) == 0;
                let air_clear = (0..4).all(|d| !active_at(world, x, y + d));
                if walls_clear && air_clear {
                    clearing = true;
                }
            }
            y += 1;
        }
    }
}

/// `WorldGen.CheckLilyPad` (`:59203-59309`): a pad on non-water dies; the
/// down-scan finds the first solid non-top floor (the `maxTilesY-50` cap),
/// the floor kind picks the frame row (grass 2/477 → 0, hallow 109/116 →
/// 18, jungle 60 → 36 — the C#'s duplicated `type == 109` clause is dead),
/// and the pad moves UP into an inactive liquid cell above, else DOWN
/// into an inactive cell below (liquid-0 gate), else a non-platform
/// non-solid floor kills it; an unknown floor kills it too.
fn check_lily_pad(world: &mut World, x: i32, y: i32) {
    // The netMode == 1 client gate is single-player-off in the port.
    if world.tile(x, y).is_some_and(|t| t.liquid_kind() != LiquidKind::Water) {
        kill_tile_legacy(world, x, y, true);
        return;
    }
    let mut num = y;
    loop {
        let t = world.tile(x, num);
        let stop = t.is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind));
        if stop || num >= world.height - 50 {
            break;
        }
        num += 1;
        if world.tile(x, num).is_none() {
            return;
        }
    }
    let floor_kind = kind_at(world, x, num);
    let frame_row = match floor_kind {
        2 | 477 => Some(0i16),
        109 | 116 => Some(18),
        60 => Some(36),
        _ => None,
    };
    let Some(frame_row) = frame_row else {
        kill_tile_legacy(world, x, y, true);
        return;
    };
    if let Some(t) = world.tile_mut(x, y) {
        if t.frame_y != frame_row {
            t.frame_y = frame_row;
        }
    }
    // `:59251-59267` — the move UP into an inactive liquid cell above.
    let above = world.tile(x, y - 1).map(|t| (t.liquid, t.active()));
    if let Some((liquid, active)) = above {
        if liquid > 0 && !active {
            move_pad(world, x, y, x, y - 1);
            return;
        }
    }
    // `:59268-59300` — else the move DOWN (the liquid-0 gate).
    if world.tile(x, y).is_some_and(|t| t.liquid != 0) {
        return;
    }
    let below_active = active_at(world, x, y + 1);
    if !below_active {
        move_pad(world, x, y, x, y + 1);
        return;
    }
    // `:59292-59300` — an active non-platform non-solid floor kills the pad.
    let floor = world.tile(x, y + 1);
    if floor.is_some_and(|t| {
        !td::SET_PLATFORMS.has(t.kind) && (!td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
    }) {
        kill_tile_legacy(world, x, y, true);
    }
}

/// The pad move (`:59253-59266` / `:59275-59288`): the target takes the
/// pad's frames + paint, halfBrick/slope cleared; the source deactivates
/// to kind 0. The `SquareTileFrame` rides the framer (module note).
fn move_pad(world: &mut World, sx: i32, sy: i32, tx: i32, ty: i32) {
    let (fx, fy, paint) = {
        let Some(t) = world.tile(sx, sy) else {
            return;
        };
        (t.frame_x, t.frame_y, t.paint())
    };
    if let Some(t) = world.tile_mut(tx, ty) {
        t.flags |= FLAG_ACTIVE;
        t.kind = TILE_LILY_PAD;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags &= !FLAG_HALF_BRICK;
        t.set_slope(0);
        t.set_paint(paint);
    }
    if let Some(t) = world.tile_mut(sx, sy) {
        t.flags &= !FLAG_ACTIVE;
        t.kind = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air(width: i32, height: i32, surface: f64, rock: f64) -> World {
        let mut world = World::new("rx".into(), width, height);
        world.world_surface = surface;
        world.rock_layer = rock;
        world
    }

    fn put(world: &mut World, x: i32, y: i32, kind: u16) {
        let i = world.index(x, y);
        world.tiles[i].kind = kind;
        world.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn wall(world: &mut World, x: i32, y: i32, wall: u16) {
        let i = world.index(x, y);
        world.tiles[i].wall = wall;
    }

    fn opts(flags: u16, crimson: bool) -> GenOptions {
        GenOptions {
            flags: SeedFlags::from(flags),
            crimson,
            ..Default::default()
        }
    }

    fn gv() -> GenVars {
        GenVars::default()
    }

    /// The seed gate: a classic world never enters the pass.
    #[test]
    fn classic_world_is_untouched() {
        let mut w = air(900, 500, 200.0, 300.0);
        put(&mut w, 450, 80, 189); // a cloud
        wall(&mut w, 460, 90, WALL_CLOUD);
        let before = w.tiles.clone();
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(0, false));
        assert_eq!(w.tiles, before, "a classic world reads no REMIX flag");
    }

    /// The cloud/sunplate conversion (`:26357-26394`): corruption → 474,
    /// crimson → 195; the cloud wall 73 → 233/77.
    #[test]
    fn clouds_and_sunplates_convert_by_the_world_flavour() {
        for (crimson, want_tile, want_wall) in
            [(false, TILE_LESION_BLOCK, WALL_LESION_BLOCK), (true, TILE_FLESH_BLOCK, WALL_FLESH)]
        {
            let mut w = air(900, 500, 200.0, 300.0);
            put(&mut w, 450, 80, 189); // a Clouds-set kind
            put(&mut w, 460, 90, TILE_SUNPLATE);
            wall(&mut w, 470, 100, WALL_CLOUD);
            finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(SeedFlags::REMIX, crimson));
            assert_eq!(kind_at(&w, 450, 80), want_tile);
            assert_eq!(kind_at(&w, 460, 90), want_tile);
            assert_eq!(wall_at(&w, 470, 100), want_wall);
        }
    }

    /// The anniversary gate (`:26363`): under tenth-&&!FTW the clouds
    /// record the row but do NOT convert.
    #[test]
    fn anniversary_without_ftw_keeps_the_clouds() {
        let mut w = air(900, 500, 200.0, 300.0);
        put(&mut w, 450, 80, 189);
        wall(&mut w, 470, 100, WALL_CLOUD);
        let o = opts(SeedFlags::REMIX | SeedFlags::TENTH_ANNIVERSARY, false);
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &o);
        assert_eq!(kind_at(&w, 450, 80), 189, "the cloud stays");
        assert_eq!(wall_at(&w, 470, 100), WALL_CLOUD, "the cloud wall stays");
    }

    /// The inland band (`:26444-26473`): wall-87 liquid → water; the FTW
    /// arm turns the honey drip 375 → lava drip 374 and floods lava.
    #[test]
    fn the_inland_band_water_and_ftw_lava_arms() {
        let mut w = air(900, 500, 200.0, 300.0);
        wall(&mut w, 450, 100, WALL_LIHZAHRD_BRICK);
        {
            let t = w.tile_mut(450, 100).unwrap();
            t.liquid = 50;
            t.set_liquid_kind(LiquidKind::Lava);
        }
        put(&mut w, 460, 100, TILE_HONEY_DRIP);
        {
            let t = w.tile_mut(460, 100).unwrap();
            t.liquid = 50;
        }
        let o = opts(SeedFlags::REMIX | SeedFlags::FOR_THE_WORTHY, false);
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &o);
        assert_eq!(w.tile(450, 100).unwrap().liquid_kind(), LiquidKind::Water);
        assert_eq!(kind_at(&w, 460, 100), TILE_LAVA_DRIP);
        assert_eq!(w.tile(460, 100).unwrap().liquid_kind(), LiquidKind::Lava);
    }

    /// The surface conversion (`:26567-26570`): grass above the surface
    /// converts to the world flavour's evil grass.
    #[test]
    fn surface_grass_converts_to_the_evil_flavour() {
        for (crimson, want) in [(false, 23u16), (true, 199)] {
            let mut w = air(900, 500, 200.0, 300.0);
            put(&mut w, 450, 50, TILE_GRASS);
            finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(SeedFlags::REMIX, crimson));
            assert_eq!(kind_at(&w, 450, 50), want);
        }
    }

    /// The anniversary replacement arm (`:26524-26541`): the corrupt
    /// jungle grass 661 survives Convert(2) (the case-2 reconciliation)
    /// and becomes 109; the 7×7 mud around it dies to dirt.
    #[test]
    fn anniversary_jungle_grass_becomes_hallowed_grass_and_mud_dies() {
        let mut w = air(900, 500, 200.0, 300.0);
        put(&mut w, 450, 50, TILE_CORRUPT_JUNGLE);
        put(&mut w, 451, 51, TILE_MUD);
        let o = opts(SeedFlags::REMIX | SeedFlags::TENTH_ANNIVERSARY, false);
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &o);
        assert_eq!(kind_at(&w, 450, 50), TILE_HALLOWED_GRASS);
        assert_eq!(kind_at(&w, 451, 51), TILE_DIRT, "the 7x7 mud kill fires");
    }

    /// The marble/granite wall clear (`:26572-26575`): above the
    /// `worldSurface - Next(19,22)` band the unsafe walls clear.
    #[test]
    fn marble_and_granite_walls_clear_above_the_band() {
        let mut w = air(900, 500, 200.0, 300.0);
        wall(&mut w, 450, 50, WALL_MARBLE_UNSAFE);
        wall(&mut w, 460, 50, WALL_GRANITE_UNSAFE);
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(SeedFlags::REMIX, false));
        assert_eq!(wall_at(&w, 450, 50), 0);
        assert_eq!(wall_at(&w, 460, 50), 0);
    }

    /// The spawn-box clear (`:26594-26621`): hellstone 58 → ash 57 and the
    /// liquid zeroes inside the 0.38-0.62 × height-200..-125 box.
    #[test]
    fn the_spawn_box_clears_liquid_and_demotes_hellstone() {
        let mut w = air(900, 700, 200.0, 300.0);
        put(&mut w, 450, 520, TILE_HELLSTONE);
        {
            let t = w.tile_mut(450, 520).unwrap();
            t.liquid = 255;
        }
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(SeedFlags::REMIX, false));
        let t = w.tile(450, 520).unwrap();
        assert_eq!(t.kind, TILE_ASH);
        assert_eq!(t.liquid, 0);
    }

    /// The dungeon repaint (`:26655-26666`): the bricks paint 22 and the
    /// dungeon walls paint 25 on a plain remix world.
    #[test]
    fn the_dungeon_repaint_paints_22_and_25() {
        let mut w = air(900, 500, 200.0, 300.0);
        put(&mut w, 450, 300, 41); // blue dungeon brick
        wall(&mut w, 460, 300, 7); // a dungeon wall
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(SeedFlags::REMIX, false));
        assert_eq!(w.tiles[w.index(450, 300)].paint(), 22);
        assert_eq!(w.wall_paint(460, 300), 25);
    }

    /// The lily-pad sweep (`:59203-59309`): a pad over a jungle-grass
    /// floor restyles to frameY 36 and falls through every free cell down
    /// to the first one above the floor (the sweep visits the moved pad
    /// again on the next row, so it cascades).
    #[test]
    fn the_lily_pad_restyles_and_moves_down() {
        let mut w = air(900, 500, 50.0, 300.0);
        put(&mut w, 450, 55, TILE_LILY_PAD);
        put(&mut w, 450, 60, TILE_JUNGLE_GRASS);
        finish_remix_world(&mut w, &gv(), &mut Streams::new(7), &opts(SeedFlags::REMIX, false));
        assert!(!active_at(&w, 450, 55), "the source cell deactivates");
        // Rows 56..59 are re-visited by the ascending y sweep, so the pad
        // settles on 59 — the last air cell above the jungle-grass floor.
        for y in 55..59 {
            assert!(!active_at(&w, 450, y), "the pad left row {y}");
        }
        let t = w.tile(450, 59).unwrap();
        assert!(t.active() && t.kind == TILE_LILY_PAD);
        assert_eq!(t.frame_y, 36, "the jungle floor picks frame row 36");
    }

    /// The full-pipeline a==b witness: a REMIX world generates
    /// byte-identically across two runs and differs from a classic world.
    /// Written NOT run (code-only).
    #[test]
    fn the_remix_world_is_reproducible_and_distinct_from_classic() {
        let rx = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            flags: SeedFlags::from(SeedFlags::REMIX),
            ..Default::default()
        };
        let a = crate::worldgen::generate(&rx);
        let b = crate::worldgen::generate(&rx);
        assert_eq!(a.tiles, b.tiles, "the REMIX world is deterministic");
        let classic = GenOptions { width: 512, height: 512, seed: 42, ..Default::default() };
        let c = crate::worldgen::generate(&classic);
        assert_ne!(a.tiles, c.tiles, "the remix tail changes the world");
    }
}
