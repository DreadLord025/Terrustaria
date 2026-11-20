//! The dual-dungeon producer chain — the `GenVars.CurrentDungeonGenVars`
//! setup the gated `Dual Dungeons Dither Snake` pass (`WorldGen.cs:14755`)
//! reads: the per-dungeon colour/brick roll + drunk wall re-roll
//! (`DungeonCrawler.SetupDungeonGenVarVariables`, `DungeonCrawler.cs:54-108`),
//! the potential-dungeon bounds (`DungeonUtils.CreatePotentialDungeonBounds`,
//! `DungeonUtils.cs:2067-2087`), and the `dungeonGenerationStyles` dual quads
//! (`DungeonCrawler.cs:111-128`).
//!
//! W4-f24 lane B. The style rows here carry ONLY the fields the snake's
//! paint path consumes ([`SnakeStyle`]: brick tile/wall, unbreakable tier,
//! edge dither) — the full furniture/chest/window tables are c32's lane
//! (Snow/Desert/Hallow/Crimson/Crystal landed there) and the wider dungeon
//! content; this slice ports the paint-relevant fields of the four statics
//! c32 deferred (Cavern/Corruption/Jungle/Temple) plus the second quad's
//! Snow/Desert/Hallow and the colour-derived style-0 wrapper
//! (`GetCurrentDungeonStyle`, `DungeonGenerationStyles.cs:95-135`). The
//! merger may dedupe the overlapping rows onto c32's tables.
//!
//! UNWIRED until B6: the setup draws only under
//! `MicroSeedFlags::DUAL_DUNGEONS` (the Dunes-pass dual setup,
//! `WorldGen.cs:11520-11525`, and the gated snake step) — a classic world
//! runs neither, so both classic world-hash pins stay; `a == b` untouched.

use super::dither_snake::DitherSnakeDungeon;
use super::dungeon::DungeonColor;
use super::dungeon_bounds::DungeonBounds;
use super::dungeon_control_line::SnakeStyle;
use crate::seeds::{MicroSeedFlags, SeedFlags};

/// `DualDungeonUnbreakableWallTiers` (`DualDungeonUnbreakableWallTiers.cs:
/// 19-24`) — the 0..5 ladder the paint path's tier band reads.
mod wall_tiers {
    pub const EARLY_GAME: i32 = 0;
    pub const EVIL_BOSS: i32 = 1;
    pub const JUNGLE_BOSS: i32 = 2;
    pub const DUNGEON: i32 = 3;
    pub const HALLOW: i32 = 4;
    #[allow(dead_code)] // the ladder's top rung; no in-scope style sets it
    pub const TEMPLE: i32 = 5;
}

/// `DungeonGenerationStyles.Cavern` (`DungeonGenerationStyles.cs:307-351`)
/// — the paint fields: brick 38, wall 349, EarlyGame tier, EdgeDither true.
pub(crate) fn style_cavern() -> SnakeStyle {
    SnakeStyle { brick_tile: 38, brick_wall: 349, unbreakable_tier: wall_tiers::EARLY_GAME, edge_dither: true }
}

/// `DungeonGenerationStyles.Corruption` (`:434-473`) — brick 25, wall 3,
/// EvilBoss tier, EdgeDither true.
pub(crate) fn style_corruption() -> SnakeStyle {
    SnakeStyle { brick_tile: 25, brick_wall: 3, unbreakable_tier: wall_tiers::EVIL_BOSS, edge_dither: true }
}

/// `DungeonGenerationStyles.Crimson` (`:474-513`) — brick 203, wall 83,
/// EvilBoss tier, EdgeDither true.
pub(crate) fn style_crimson() -> SnakeStyle {
    SnakeStyle { brick_tile: 203, brick_wall: 83, unbreakable_tier: wall_tiers::EVIL_BOSS, edge_dither: true }
}

/// `DungeonGenerationStyles.Jungle` (`:714-759`) — brick 59, wall 64,
/// JungleBoss tier, EdgeDither true.
pub(crate) fn style_jungle() -> SnakeStyle {
    SnakeStyle { brick_tile: 59, brick_wall: 64, unbreakable_tier: wall_tiers::JUNGLE_BOSS, edge_dither: true }
}

/// `DungeonGenerationStyles.Snow` (`:352-392`) — brick 161, wall 71,
/// EarlyGame tier, EdgeDither true.
pub(crate) fn style_snow() -> SnakeStyle {
    SnakeStyle { brick_tile: 161, brick_wall: 71, unbreakable_tier: wall_tiers::EARLY_GAME, edge_dither: true }
}

/// `DungeonGenerationStyles.Desert` (`:393-433`) — brick 396, wall 187,
/// EarlyGame tier, EdgeDither FALSE (`:431`).
pub(crate) fn style_desert() -> SnakeStyle {
    SnakeStyle { brick_tile: 396, brick_wall: 187, unbreakable_tier: wall_tiers::EARLY_GAME, edge_dither: false }
}

/// `DungeonGenerationStyles.Hallow` (`:553-593`) — brick 117, wall 28,
/// Hallow tier, EdgeDither true.
pub(crate) fn style_hallow() -> SnakeStyle {
    SnakeStyle { brick_tile: 117, brick_wall: 28, unbreakable_tier: wall_tiers::HALLOW, edge_dither: true }
}

/// `DungeonGenerationStyles.Temple` (`:760-797`, a `TempleStyleData` whose
/// override only gates features) — brick 226, wall 87, the default −1 tier
/// (no `UnbreakableWallProgressionTier` set), EdgeDither FALSE (`:795`).
pub(crate) fn style_temple() -> SnakeStyle {
    SnakeStyle { brick_tile: 226, brick_wall: 87, unbreakable_tier: -1, edge_dither: false }
}

/// `DungeonGenerationStyles.GetCurrentDungeonStyle`
/// (`DungeonGenerationStyles.cs:95-135`) — the style-0 wrapper over the
/// rolled colour's brick triple; Dungeon tier (`:99`), EdgeDither false
/// (`:132`).
pub(crate) fn style_current_dungeon(color: DungeonColor) -> SnakeStyle {
    SnakeStyle {
        brick_tile: color.brick_tile(),
        brick_wall: color.brick_wall(),
        unbreakable_tier: wall_tiers::DUNGEON,
        edge_dither: false,
    }
}

/// The colour roll + drunk wall re-roll of
/// `SetupDungeonGenVarVariables` (`DungeonCrawler.cs:56-108`). Returns the
/// rolled [`DungeonColor`] and the effective brick wall (the drunk
/// `genRand.Next(3)` overrides it to 7/8/9, `:94-108`). `rng` is the Dunes
/// pass stream (`WorldGen.genRand` at the `:11519/:11523` call sites).
///
/// The remix pin (`:57-60`: `crimson ? 2 : 0`) needs the crimson-side flag;
/// the port passes it explicitly. The window-wall triples (`:68-91`) and
/// the entrance-type rolls (`:184-210`) are the dungeon-content lane's, not
/// the snake's — cited skip (the snake paints brick + wall only).
pub(crate) fn roll_dungeon_color(
    rng: &mut super::Rng,
    flags: SeedFlags,
    crimson: bool,
) -> (DungeonColor, u16) {
    let mut num = rng.range(0, 3);
    if flags.contains(SeedFlags::REMIX) {
        num = if crimson { 2 } else { 0 };
    }
    let color = match num {
        1 => DungeonColor::Green,
        2 => DungeonColor::Pink,
        _ => DungeonColor::Blue,
    };
    let mut brick_wall = color.brick_wall();
    if flags.contains(SeedFlags::DRUNK) {
        brick_wall = match rng.range(0, 3) {
            1 => 8,
            2 => 9,
            _ => 7,
        };
    }
    (color, brick_wall)
}

/// `DungeonUtils.CreatePotentialDungeonBounds`
/// (`DungeonUtils.cs:2067-2087`) — the inner/outer potential-dungeon rects.
/// The snake's caller passes `percentInMiddle = 0.1`, `percentOnEdges =
/// 0.05` (`DungeonCrawler.cs:109`); `percentOnTop`/`Bottom` default to the
/// surface/underworld fractions. `dungeon_entrance_is_underground` is
/// `SpecialSeedFeatures.DungeonEntranceIsUnderground`
/// (`SpecialSeedFeatures.cs:65-75`: drunk → true, else noSurface);
/// `world_surface_high` is `GenVars.worldSurfaceHigh` (`TerrainPass.cs:233`).
pub(crate) fn create_potential_dungeon_bounds(
    max_x: i32,
    max_y: i32,
    world_surface: f64,
    world_surface_high: f64,
    underworld_layer: i32,
    left_dungeon: bool,
    dungeon_entrance_is_underground: bool,
) -> (DungeonBounds, DungeonBounds) {
    let percent_in_middle = 0.10000000149011612;
    let percent_on_edges = 0.05000000074505806;
    // `:2069-2076`: the top/bottom fractions (the -1 sentinels resolve to
    // the surface/underworld bands).
    let percent_on_top = if !dungeon_entrance_is_underground {
        (world_surface + 10.0) / max_y as f64
    } else {
        (world_surface_high + 10.0) / max_y as f64
    };
    let percent_on_bottom = (underworld_layer as f64 - 10.0) / max_y as f64;
    let num = percent_in_middle / 2.0;
    // `:2079-2082`: the left/right band per dungeon side.
    let num2 = if left_dungeon {
        (max_x as f64 * percent_on_edges) as i32
    } else {
        (max_x as f64 * (0.5 + num)) as i32
    };
    let num3 = if left_dungeon {
        (max_x as f64 * (0.5 - num)) as i32
    } else {
        max_x - (max_x as f64 * percent_on_edges) as i32
    };
    let num4 = (max_y as f64 * percent_on_top) as i32;
    let num5 = (max_y as f64 * percent_on_bottom) as i32;
    let inner_buffer = 10;
    let mut outer = DungeonBounds::new(max_x, max_y);
    outer.set_bounds(num2, num4, num3, num5);
    let mut inner = DungeonBounds::new(max_x, max_y);
    inner.set_bounds(num2 + inner_buffer, num4 + inner_buffer, num3 - inner_buffer, num5 - inner_buffer);
    (inner, outer)
}

/// The `dungeonGenerationStyles` dual quads (`DungeonCrawler.cs:111-128`):
/// dungeon 0 = [Cavern, evil, Jungle, current], dungeon 1 = [Snow, Desert,
/// Hallow, Temple]. `evil` picks Crimson/Corruption by the world's crimson
/// flag (`WorldGen.crimson`); `current` is the rolled colour's style-0
/// wrapper. Each quad is the snake's `styles` slice (4 entries).
pub(crate) fn dual_dungeon_styles(
    crimson: bool,
    color: DungeonColor,
) -> (Vec<SnakeStyle>, Vec<SnakeStyle>) {
    let evil = if crimson { style_crimson() } else { style_corruption() };
    let quad0 = vec![style_cavern(), evil, style_jungle(), style_current_dungeon(color)];
    let quad1 = vec![style_snow(), style_desert(), style_hallow(), style_temple()];
    (quad0, quad1)
}

/// One dungeon's setup product — the bounds + side + location + style quad
/// the snake step (B6) consumes. `dungeon_location` is the port's
/// `spans().dungeon_x` (the C# `GenVars.CurrentDungeonGenVars.dungeonLocation`,
/// set by the dungeon placement); the second dungeon mirrors it across the
/// world centre (the dual-dungeon layout — the port's documented adaptation
/// until the second-dungeon content lane lands the faithful placement).
#[derive(Clone, Debug)]
pub(crate) struct DualDungeonSetup {
    pub dungeons: Vec<DitherSnakeDungeon>,
    pub styles: Vec<Vec<SnakeStyle>>,
}

/// `SetupDungeonGenVarVariables` for both dungeons (the Dunes-pass dual
/// setup, `WorldGen.cs:11520-11525`: dungeon 1 then dungeon 0, each a fresh
/// colour roll) + the bounds + the style quads. `rng` is the Dunes pass
/// stream; the two colour rolls run in the C#'s order (dungeon 1 first).
/// `crimson` is `WorldGen.crimson` (the evil flavour, `opts.crimson`).
pub(crate) fn setup_dual_dungeons(
    rng: &mut super::Rng,
    flags: SeedFlags,
    micro: MicroSeedFlags,
    crimson: bool,
    max_x: i32,
    max_y: i32,
    world_surface: f64,
    world_surface_high: f64,
    dungeon_location: i32,
) -> DualDungeonSetup {
    let underworld_layer = max_y - 200; // Main.UnderworldLayer (Main.cs:2790)
    let entrance_underground = if flags.contains(SeedFlags::DRUNK) {
        true
    } else {
        micro.contains(MicroSeedFlags::NO_SURFACE)
    };
    // The C# rolls dungeon 1 first (`:11522-11524`), then dungeon 0
    // (`:11519` ran before the dual block). The port rolls dungeon 0's
    // colour first to match the `:11519`-then-`:11523` call order.
    let (color0, _wall0) = roll_dungeon_color(rng, flags, crimson);
    let (color1, _wall1) = roll_dungeon_color(rng, flags, crimson);
    let left0 = dungeon_location < max_x / 2;
    // The second dungeon sits on the opposite side (the dual layout).
    let location1 = max_x - dungeon_location;
    let left1 = !left0;
    let (_inner0, outer0) = create_potential_dungeon_bounds(
        max_x, max_y, world_surface, world_surface_high, underworld_layer, left0, entrance_underground,
    );
    let (_inner1, outer1) = create_potential_dungeon_bounds(
        max_x, max_y, world_surface, world_surface_high, underworld_layer, left1, entrance_underground,
    );
    let (quad0, quad1) = dual_dungeon_styles(crimson, color0);
    let _ = color1; // the second quad is fixed (Snow/Desert/Hallow/Temple)
    let dungeons = vec![
        DitherSnakeDungeon {
            outer: outer0,
            dungeon_side: if left0 { -1 } else { 1 },
            dungeon_location,
            style_count: quad0.len(),
        },
        DitherSnakeDungeon {
            outer: outer1,
            dungeon_side: if left1 { -1 } else { 1 },
            dungeon_location: location1,
            style_count: quad1.len(),
        },
    ];
    DualDungeonSetup { dungeons, styles: vec![quad0, quad1] }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dual quads carry the C# paint fields verbatim
    /// (`DungeonCrawler.cs:111-128` + the style statics).
    #[test]
    fn w4_f24_dual_quads_are_verbatim() {
        let (q0, q1) = dual_dungeon_styles(false, DungeonColor::Blue);
        assert_eq!(q0.len(), 4);
        assert_eq!(q1.len(), 4);
        // dungeon 0: Cavern, Corruption (crimson=false), Jungle, current.
        assert_eq!((q0[0].brick_tile, q0[0].brick_wall), (38, 349));
        assert_eq!((q0[1].brick_tile, q0[1].brick_wall), (25, 3));
        assert_eq!((q0[2].brick_tile, q0[2].brick_wall), (59, 64));
        assert_eq!((q0[3].brick_tile, q0[3].brick_wall), (41, 7));
        assert_eq!(q0[3].unbreakable_tier, 3); // Dungeon tier
        // dungeon 1: Snow, Desert, Hallow, Temple.
        assert_eq!((q1[0].brick_tile, q1[0].brick_wall), (161, 71));
        assert_eq!((q1[1].brick_tile, q1[1].brick_wall), (396, 187));
        assert!(!q1[1].edge_dither); // Desert EdgeDither false
        assert_eq!((q1[2].brick_tile, q1[2].brick_wall), (117, 28));
        assert_eq!((q1[3].brick_tile, q1[3].brick_wall), (226, 87));
        assert_eq!(q1[3].unbreakable_tier, -1); // Temple default
        assert!(!q1[3].edge_dither);
        // crimson=true swaps the evil pick.
        let (qc, _) = dual_dungeon_styles(true, DungeonColor::Blue);
        assert_eq!((qc[1].brick_tile, qc[1].brick_wall), (203, 83));
    }

    /// The colour roll consumes one Next(3) (plus a drunk Next(3)); the
    /// remix pin skips the draw (`DungeonCrawler.cs:56-108`). a == b replay.
    #[test]
    fn w4_f24_color_roll_is_deterministic() {
        let roll = |flags: SeedFlags| {
            let mut rng = super::super::Rng::from_seed(7);
            roll_dungeon_color(&mut rng, flags, false)
        };
        let a = roll(SeedFlags::default());
        let b = roll(SeedFlags::default());
        assert_eq!(a.0 as u8, b.0 as u8);
        assert_eq!(a.1, b.1);
        // The drunk re-roll draws a second Next(3) and pins the wall 7/8/9.
        let drunk = roll(SeedFlags::from(SeedFlags::DRUNK));
        assert!(matches!(drunk.1, 7 | 8 | 9));
    }

    /// `CreatePotentialDungeonBounds` splits the world by side and clamps
    /// the inner buffer (`DungeonUtils.cs:2067-2087`).
    #[test]
    fn w4_f24_potential_bounds_split_by_side() {
        let (inner_r, outer_r) =
            create_potential_dungeon_bounds(4200, 1200, 300.0, 350.0, 1000, false, false);
        // Right dungeon: x in [0.55·W, W-0.05·W] = [2310, 3990].
        assert_eq!(outer_r.left(), 2310);
        assert_eq!(outer_r.right(), 3990);
        assert_eq!(inner_r.left(), 2320);
        assert_eq!(inner_r.right(), 3980);
        let (inner_l, outer_l) =
            create_potential_dungeon_bounds(4200, 1200, 300.0, 350.0, 1000, true, false);
        // Left dungeon: x in [0.05·W, 0.45·W] = [210, 1890].
        assert_eq!(outer_l.left(), 210);
        assert_eq!(outer_l.right(), 1890);
        assert_eq!(inner_l.left(), 220);
        // Top = (worldSurface+10) = 310; bottom = UnderworldLayer-10 = 990.
        assert_eq!(outer_r.top(), 310);
        assert_eq!(outer_r.bottom(), 990);
    }
}
