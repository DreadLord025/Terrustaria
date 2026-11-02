//! The `SceneMetrics` tile-window scan (`SceneMetrics.Scan` / `ScanTiles`).
//!
//! Lives in `tr-app` because the scan is centered on the player (a `tr-app`
//! type); everything it fills is `tr_core::scene::SceneMetrics` data.
//!
//! Source: `decompiled/Terraria/SceneMetrics.cs` (`Scan` `:336-351`,
//! `ScanTiles` `:353-379`, `CalculateZones` `:662-707`), `Main.cs:62541`
//! (the scan center is the local player's `Center`).

use tr_core::scene::{is_desert_biome_sand, ocean_depths, SceneMetrics, ZONE_SCAN_SIZE};
use tr_core::scene::{Sandstorm, SceneState};
use tr_core::World;

use crate::player::{self, Player};

/// `Main.SceneMetrics.Scan(new SceneMetricsScanSettings { BiomeScanCenterPositionInWorld =
/// player.Center … })` (`Main.cs:62533-62546`, `SceneMetrics.cs:336-351`): reset,
/// compute the tile center, run the window pass, derive the counts and the zones.
///
/// `raining` is `Main.raining` (our `Rain::is_raining()`), `sandstorm_happening`
/// is `Sandstorm::happening` — both are game-state inputs the zone derivation
/// reads (`SceneMetrics.cs:701,703`).
pub(crate) fn scan_scene(
    world: &World,
    player: &Player,
    raining: bool,
    sandstorm_happening: bool,
) -> SceneMetrics {
    // `metrics.Center` = the local player's center in world pixels
    // (`Main.cs:62541`; the player position is the top-left corner).
    let center = [
        player.x + player::WIDTH * 0.5,
        player.y + player::HEIGHT * 0.5,
    ];
    // `this.TileCenter = Center.ToTileCoordinates().ClampedInWorld()`
    // (`SceneMetrics.cs:333`); `ToTileCoordinates` is `(int)v >> 4`
    // (`Utils.cs:1807-1810`). `ClampedInWorld` clamps both coordinates to
    // `maxTilesX - 1` — the vanilla Y clamp uses `maxTilesX` too
    // (`Utils.cs:1855-1860`, latent bug: width > height on every vanilla world
    // size, so it never binds there either).
    let tx = (center[0] as i32) >> 4;
    let ty = (center[1] as i32) >> 4;
    let tile_center = (tx.clamp(0, world.width - 1), ty.clamp(0, world.width - 1));

    let mut metrics = SceneMetrics::new();
    metrics.center = center;
    metrics.tile_center = tile_center;
    scan_tiles(world, &mut metrics);
    // `CalculateZones`'s `BehindBackwall` probe (`SceneMetrics.cs:671-672`):
    // `Framing.GetTileSafely(TileCenter).wall > 0`.
    metrics.behind_backwall = world
        .tile(tile_center.0, tile_center.1)
        .is_some_and(|t| t.wall > 0);
    metrics.aggregate_tile_counts();
    metrics.calculate_zones(world.world_surface, raining, sandstorm_happening);
    metrics
}

/// `SceneMetrics.ScanTiles` (`SceneMetrics.cs:353-379`): count every active
/// tile in `ClampToWorld(CenteredRectangle(TileCenter, ZoneScanSize))`.
fn scan_tiles(world: &World, metrics: &mut SceneMetrics) {
    // `Utils.CenteredRectangle(Point, Point)` (`Utils.cs:952-956`).
    let left = metrics.tile_center.0 - ZONE_SCAN_SIZE.0 / 2;
    let top = metrics.tile_center.1 - ZONE_SCAN_SIZE.1 / 2;
    let right = left + ZONE_SCAN_SIZE.0;
    let bottom = top + ZONE_SCAN_SIZE.1;
    // `WorldUtils.ClampToWorld` (`WorldBuilding/WorldUtils.cs`, fluff 0): all
    // four edges clamp independently, so a window hanging off the world edge
    // shrinks instead of sliding back in.
    let left = left.clamp(0, world.width);
    let top = top.clamp(0, world.height);
    let right = right.clamp(0, world.width);
    let bottom = bottom.clamp(0, world.height);
    for x in left..right {
        for y in top..bottom {
            let Some(tile) = world.tile(x, y) else {
                // `SceneMetrics.cs:361-363`: absent tiles are skipped.
                continue;
            };
            if !tile.active() {
                // `SceneMetrics.cs:365-371`: inactive tiles only feed the
                // liquid counts (`_liquidCounts`) — no M1 reader.
                continue;
            }
            // `SceneMetrics.cs:373`: `_tileCounts[tile.type]++`.
            if let Some(count) = metrics.tile_counts.get_mut(tile.kind as usize) {
                *count += 1;
            }
            // `SceneMetrics.cs:377-379`: ocean-edge desert sand.
            if is_desert_biome_sand(tile.kind)
                && ocean_depths(world.world_surface, world.rock_layer, x, world.width, y)
            {
                metrics.ocean_sand_tile_count += 1;
            }
        }
    }
}

/// Reads the TR_ determinism forces from the process environment and applies
/// them: `TR_SANDSTORM=<0..1>`, `TR_GRAVEYARD=<0..1>`, `TR_BLOODMOON=1`
/// (parsing is [`tr_core::scene::parse_env_force`]). Called once per fixed
/// step by the phase-3 driver, so a screenshot run pins the whole state.
///
/// While `TR_SANDSTORM` is set the driver should skip `Sandstorm::update_time`
/// entirely (the pin holds `severity == intended_severity`, but the periodic
/// intention roll would still fight it); `forced_severity` marks that.
pub(crate) fn apply_scene_forces_from_env(state: &mut SceneState, sandstorm: &mut Sandstorm) {
    apply_scene_forces(state, sandstorm, |var| std::env::var(var).ok());
}

/// The injectable body of [`apply_scene_forces_from_env`]: `lookup` stands in
/// for the environment read so the wiring is unit-testable.
fn apply_scene_forces(
    state: &mut SceneState,
    sandstorm: &mut Sandstorm,
    lookup: impl Fn(&str) -> Option<String>,
) {
    for var in ["TR_SANDSTORM", "TR_GRAVEYARD", "TR_BLOODMOON"] {
        let Some(raw_value) = lookup(var) else {
            continue;
        };
        let entry = format!("{var}={raw_value}");
        let Some((name, value)) = tr_core::scene::parse_env_force(&entry) else {
            continue;
        };
        if name == "SANDSTORM" {
            sandstorm.force_severity(value);
        }
        state.force(name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::scene::{thresholds, Sandstorm, SceneState};
    use tr_core::tile::FLAG_ACTIVE;

    /// A small test world with vanilla-shaped surface lines.
    fn fixture_world() -> World {
        let mut world = World::new("scene-scan-test".into(), 1000, 400);
        world.world_surface = 100.0;
        world.rock_layer = 180.0;
        world
    }

    fn set_tile(world: &mut World, x: i32, y: i32, kind: u16) {
        let tile = world.tile_mut(x, y).unwrap();
        tile.kind = kind;
        tile.flags |= FLAG_ACTIVE;
    }

    fn fill(world: &mut World, x0: i32, y0: i32, x1: i32, y1: i32, kind: u16) {
        for y in y0..y1 {
            for x in x0..x1 {
                set_tile(world, x, y, kind);
            }
        }
    }

    /// The player standing with its hitbox center exactly on tile `(tx, ty)`
    /// (the scan center is `x + WIDTH/2, y + HEIGHT/2`).
    fn player_at(world: &World, tx: i32, ty: i32) -> Player {
        let mut player = Player::spawn(world);
        player.x = tx as f32 * 16.0 - player::WIDTH * 0.5;
        player.y = ty as f32 * 16.0 - player::HEIGHT * 0.5;
        player
    }

    #[test]
    fn test_DW_1_1_snow_zone_counts() {
        let mut world = fixture_world();
        // 80x20 = 1600 snow tiles (id 147) fully inside the 169x124 window at
        // tile center (300, 50): window x 216..385, y 0..113.
        fill(&mut world, 260, 30, 340, 50, 147);
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_center.0, 300);
        assert_eq!(metrics.tile_center.1, 50);
        assert_eq!(metrics.tile_counts[147], 1600);
        assert_eq!(metrics.snow_tile_count, 1600);
        assert!(metrics.enough_tiles_for_snow());
        assert!(metrics.zone_snow);
        // Not raining, no sandstorm, no desert.
        assert!(!metrics.zone_rain);
        assert!(!metrics.zone_desert);
        assert!(!metrics.zone_sandstorm);
        // The block is a solid rectangle: every window tile of another kind
        // must be untouched (sanity: total active == placed).
        let total: u32 = metrics.tile_counts.iter().sum();
        assert_eq!(total, 1600);
    }

    #[test]
    fn test_DW_1_1_desert_zone_counts() {
        let mut world = fixture_world();
        // Sand far from both beach bands (380 left / 620 right on a 1000-wide
        // world) around the player at tile (500, 50): window x 416..585.
        fill(&mut world, 460, 30, 540, 50, 53);
        let player = player_at(&world, 500, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_counts[53], 1600);
        assert_eq!(metrics.sand_tile_count, 1600);
        assert_eq!(metrics.ocean_sand_tile_count, 0);
        assert_eq!(metrics.desert_sand_tile_count, 1600);
        assert!(metrics.zone_desert);
        assert!(metrics.surface_atmospherics);
    }

    #[test]
    fn beach_sand_is_subtracted_from_the_desert_count() {
        // Sand in the left beach band (x < 380, above oceanLevel=180) counts
        // as ocean sand: `DesertSandTileCount = Sand - OceanSand`
        // (`SceneMetrics.cs:377-379,658`) — a full beach window yields no
        // desert zone at all.
        let mut world = fixture_world();
        fill(&mut world, 60, 30, 140, 50, 53);
        let player = player_at(&world, 100, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.sand_tile_count, 1600);
        assert_eq!(metrics.ocean_sand_tile_count, 1600);
        assert_eq!(metrics.desert_sand_tile_count, 0);
        assert!(!metrics.zone_desert);
    }

    #[test]
    fn test_DW_1_1_rain_and_snow_both_true() {
        // `SceneState.cs:120` gates the blizzard on `ZoneRain && ZoneSnow` —
        // the flags must coexist.
        let mut world = fixture_world();
        fill(&mut world, 260, 30, 340, 50, 147);
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, true, false);
        assert!(metrics.zone_rain, "raining + on the surface");
        assert!(metrics.zone_snow);
    }

    #[test]
    fn sandstorm_zone_requires_desert_surface_and_happening() {
        let mut world = fixture_world();
        fill(&mut world, 460, 30, 540, 50, 53);
        let player = player_at(&world, 500, 50);
        let metrics = scan_scene(&world, &player, true, true);
        // `SceneMetrics.cs:703-707`: the storm zone wins over the rain zone.
        assert!(metrics.zone_sandstorm);
        assert!(!metrics.zone_rain);
        let metrics = scan_scene(&world, &player, true, false);
        assert!(!metrics.zone_sandstorm);
        assert!(metrics.zone_rain);
    }

    #[test]
    fn test_DW_1_1_tile_center_outside_world_empty_scan() {
        let world = fixture_world();
        // Far above-left of the world: the center clamps to (0, 0) and the
        // window is clipped to the world corner.
        let mut player = Player::spawn(&world);
        player.x = -5000.0;
        player.y = -5000.0;
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_center, (0, 0));
        let total: u32 = metrics.tile_counts.iter().sum();
        assert_eq!(total, 0);
        assert!(!metrics.zone_snow && !metrics.zone_desert && !metrics.zone_rain);
        assert!(metrics.surface_atmospherics); // clamped y=0 <= surface

        // Far below the world: the vanilla Y clamp uses maxTilesX (`Utils.cs:1859`,
        // latent bug ported 1:1), so the center lands at (999, 999) — past the
        // 400-tall world — and the whole window clamps away to nothing.
        let mut player = Player::spawn(&world);
        player.x = 50000.0;
        player.y = 50000.0;
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_center, (999, 999));
        let total: u32 = metrics.tile_counts.iter().sum();
        assert_eq!(total, 0);
        assert!(!metrics.surface_atmospherics);
        assert!(!metrics.behind_backwall);
    }

    #[test]
    fn test_DW_1_1_window_clamped_at_world_edge() {
        let mut world = fixture_world();
        // A wall of snow spanning the world's left edge; the player stands at
        // the first column, so the window's left half is cut off.
        fill(&mut world, 0, 30, 200, 50, 147);
        let player = player_at(&world, 0, 50);
        let metrics = scan_scene(&world, &player, false, false);
        // Window at tile center (0, 50): x clamps to 0..85 (85 columns), y to
        // 0..112 — the block's rows 30..50 are fully inside the y range, its
        // columns beyond 85 fall outside the window.
        assert_eq!(metrics.tile_counts[147], 85 * 20);
        assert_eq!(metrics.snow_tile_count, 85 * 20);
        assert!(metrics.zone_snow);
    }

    #[test]
    fn scan_counts_graveyard_minus_sunflowers() {
        // Tombstones (85) minus half the sunflowers (27)
        // (`SceneMetrics.cs:615-624`).
        let mut world = fixture_world();
        fill(&mut world, 260, 30, 280, 50, 85); // 400 tombstones
        fill(&mut world, 300, 30, 304, 50, 27); // 80 sunflowers
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_counts[85], 400);
        assert_eq!(metrics.tile_counts[27], 80);
        assert_eq!(metrics.graveyard_tile_count, 400 - 40);
        assert!(metrics.enough_tiles_for_graveyard());
    }

    #[test]
    fn behind_backwall_reads_the_center_tile_wall() {
        // `SceneMetrics.cs:671-672`: `GetTileSafely(TileCenter).wall > 0`.
        let mut world = fixture_world();
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert!(!metrics.behind_backwall);
        // Give the center tile a backwall without activating it.
        world.tile_mut(300, 50).unwrap().wall = 1;
        let metrics = scan_scene(&world, &player, false, false);
        assert!(metrics.behind_backwall);
    }

    #[test]
    fn test_DW_1_4_scene_forces_wire_through_the_glue() {
        // The env glue parses and applies each force; the lookup closure
        // stands in for the process environment.
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        let vars: Vec<(String, String)> = vec![
            ("TR_SANDSTORM".into(), "0.75".into()),
            ("TR_GRAVEYARD".into(), "0.4".into()),
            ("TR_BLOODMOON".into(), String::new()),
        ];
        apply_scene_forces(&mut state, &mut storm, |var| {
            vars.iter().find(|(k, _)| k == var).map(|(_, v)| v.clone())
        });
        assert_eq!(state.forced_severity, Some(0.75));
        assert_eq!(state.forced_graveyard, Some(0.4));
        assert!(state.blood_moon);
        assert!(storm.happening);
        assert_eq!(storm.severity, 0.75);

        // Junk values are ignored entirely.
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        apply_scene_forces(&mut state, &mut storm, |var| {
            (var == "TR_SANDSTORM").then(|| "garbage".to_string())
        });
        assert_eq!(state.forced_severity, None);
        assert!(!storm.happening);

        // No vars set: nothing changes.
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        apply_scene_forces(&mut state, &mut storm, |_| None);
        assert_eq!(state.forced_severity, None);
        assert_eq!(state.forced_graveyard, None);
        assert!(!state.blood_moon);
        assert!(!storm.happening);
    }

    #[test]
    fn thresholds_match_scene_metrics_statics() {
        // `SceneMetrics.cs:930,934,939-941`.
        assert_eq!(thresholds::SNOW_TILE_NORMAL, 1500);
        assert_eq!(thresholds::DESERT_TILE_NORMAL, 1500);
        assert_eq!(thresholds::GRAVEYARD_TILE_MIN, 16);
        assert_eq!(thresholds::GRAVEYARD_TILE_MAX, 36);
        assert_eq!(thresholds::GRAVEYARD_TILE, 28);
        // `SceneMetrics.cs:917-919`.
        assert_eq!(ZONE_SCAN_SIZE, (169, 124));
    }
}
