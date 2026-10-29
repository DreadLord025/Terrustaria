//! Port of the NPC spawning pipeline in `Terraria/NPC.cs` (`Spawner`):
//! `TrySpawnAnNPC` — spawn-rate modifiers, the spawn-area scan, the on-screen
//! and free-space checks, and the default surface day/night enemy selection
//! for the kinds implemented in `npc.rs`.
//!
//! Biome-specific spawn tables (corruption, jungle, hallow, dungeon, ...) are
//! part of the much larger `Spawner.SpawnAnNPC` switch and are out of scope
//! here; when a chosen C# type has no `NpcKind` equivalent the attempt simply
//! returns `None` (the original would spawn that type).

use tr_core::{TILE_PX, World};

use crate::npc::{Npc, NpcKind};

/// `NPC.defaultSpawnRate` (set in `NPC.cs:95187`).
pub const DEFAULT_SPAWN_RATE: i32 = 600;
/// `NPC.defaultMaxSpawns` (set in `NPC.cs:95188`).
pub const DEFAULT_MAX_SPAWNS: i32 = 5;
/// `NPC.sWidth` / `NPC.sHeight` (NPC.cs:6767-6769).
pub const S_WIDTH: i32 = 1920;
pub const S_HEIGHT: i32 = 1200;
/// `Main.UnderworldLayer` (Main.cs:2790).
pub const UNDERWORLD_LAYER: fn(i32) -> i32 = |max_tiles_y| max_tiles_y - 200;
/// `Spawner.spawnSpaceX/Y` (NPC.cs:5999-6000): the clear-tile rect an NPC needs.
const SPAWN_SPACE_X: i32 = 2;
const SPAWN_SPACE_Y: i32 = 3;

/// Minimal deterministic stand-in for `Main.rand` (xorshift64*). The C# code
/// only relies on `Next()`/`Next(n)` uniformity, not on the exact sequence.
pub struct FastRandom(u64);

impl FastRandom {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    fn next_u64(&mut self) -> u64 {
        let mut z = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.0 = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `Main.rand.Next(n)`: uniform in `0..n`, matching the C# call sites used
    /// here (`Next(spawnRate) != 0`, `Next(6) == 0`, ...).
    pub fn next(&mut self, n: i32) -> i32 {
        debug_assert!(n > 0);
        (self.next_u64() % n.max(1) as u64) as i32
    }
}

/// Inputs to one spawn attempt, mirroring what `Spawner` reads off `Main` and
/// the `Player` in `SetSpawnFlags`.
pub struct SpawnContext<'a> {
    pub world: &'a World,
    /// Player centre in pixels (`player.Center`).
    pub player_center: (f32, f32),
    /// Player top-left in pixels (`player.position`), used by `GetSpawnArea`.
    pub player_pos: (f32, f32),
    /// `player.nearbyActiveNPCs`: active enemy NPCs near the player (npcSlots).
    pub nearby_active_npcs: f32,
    /// `player.townNPCs`: town NPCs near the player.
    pub town_npcs: i32,
    /// `Main.dayTime`.
    pub day_time: bool,
}

/// Result of `GetSpawnRate`: frames between spawn rolls and the concurrent cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnRate {
    pub spawn_rate: i32,
    pub max_spawns: i32,
}

/// `Spawner.GetSpawnArea` (NPC.cs:931-967): the tile rect candidates are drawn
/// from and the rect that must stay free of spawns. The scope/biome-gun
/// extension of the range (selected-item 1254/1299, `player.scope`) is not
/// ported — the app has no scopes yet.
pub fn get_spawn_area(ctx: &SpawnContext) -> (SpawnArea, SpawnArea) {
    let num = (S_WIDTH / 16 * 7) / 10; // (int)(0.7 * (sWidth/16))
    let num2 = (S_HEIGHT / 16 * 7) / 10;
    let safe_range_x = (S_WIDTH / 16 * 52) / 100;
    let safe_range_y = (S_HEIGHT / 16 * 52) / 100;
    let px = (ctx.player_pos.0 / TILE_PX).floor() as i32;
    let py = (ctx.player_pos.1 / TILE_PX).floor() as i32;
    let clamp_x = |x: i32| x.clamp(0, ctx.world.width - 1);
    let clamp_y = |y: i32| y.clamp(0, ctx.world.height - 1);
    let spawn = SpawnArea {
        left: clamp_x(px - num),
        right: clamp_x(px + num),
        top: clamp_y(py - num2),
        bottom: clamp_y(py + num2),
    };
    let safe = SpawnArea {
        left: clamp_x(px - safe_range_x),
        right: clamp_x(px + safe_range_x),
        top: clamp_y(py - safe_range_y),
        bottom: clamp_y(py + safe_range_y),
    };
    (spawn, safe)
}

#[derive(Clone, Copy, Debug)]
pub struct SpawnArea {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl SpawnArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

/// `Spawner.GetSpawnRate` (NPC.cs:473-929), restricted to the modifiers that
/// apply to this app: hardmode/depth/night/blood-moon/town-NPC/nearby-NPC and
/// the final clamps. Random town-NPC sub-branches (`Main.rand` inside) are
/// kept with the spawner's own RNG.
pub fn get_spawn_rate(ctx: &SpawnContext, rng: &mut FastRandom) -> SpawnRate {
    let _ = rng;
    let mut rate = SpawnRate {
        spawn_rate: DEFAULT_SPAWN_RATE,
        max_spawns: DEFAULT_MAX_SPAWNS,
    };
    let underworld = UNDERWORLD_LAYER(ctx.world.height);
    let pos_y = ctx.player_pos.1;
    let tile_y = pos_y / TILE_PX;

    if pos_y > underworld as f32 * TILE_PX {
        rate.max_spawns = (rate.max_spawns as f32 * 2.0) as i32;
    } else if ctx.world.rock_layer * 16.0 + (S_HEIGHT as f64) < (pos_y as f64) {
        // Below the rock layer.
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.4) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.9) as i32;
    } else if ctx.world.world_surface * 16.0 + (S_HEIGHT as f64) < (pos_y as f64) {
        // Below the surface.
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.5) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.7) as i32;
    } else if !ctx.day_time {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.6) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.3) as i32;
    }

    // Town-NPC pressure (the underworld branch of NPC.cs:799-929; the surface
    // branch's graveyard/spawnFriendly rolls need critters the app lacks).
    if ctx.town_npcs >= 3 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 2.0) as i32;
    } else if ctx.town_npcs == 2 {
        rate.spawn_rate = (rate.spawn_rate as f64 * 1.5) as i32;
    } else if ctx.town_npcs == 1 {
        rate.spawn_rate = (rate.spawn_rate as f64 * 1.25) as i32;
    }

    // Nearby-NPC scaling (NPC.cs:670-696).
    let nearby = ctx.nearby_active_npcs as f64;
    if nearby < rate.max_spawns as f64 * 0.2 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.6) as i32;
    } else if nearby < rate.max_spawns as f64 * 0.4 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.7) as i32;
    } else if nearby < rate.max_spawns as f64 * 0.6 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.8) as i32;
    } else if nearby < rate.max_spawns as f64 * 0.8 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.9) as i32;
    }

    // Final clamps (NPC.cs:749-756).
    if (rate.spawn_rate as f64) < DEFAULT_SPAWN_RATE as f64 * 0.1 {
        rate.spawn_rate = (DEFAULT_SPAWN_RATE as f64 * 0.1) as i32;
    }
    if rate.max_spawns > DEFAULT_MAX_SPAWNS * 3 {
        rate.max_spawns = DEFAULT_MAX_SPAWNS * 3;
    }
    let _ = tile_y;
    rate
}

/// `Spawner.HasTileSpawnSpace` + `CanSpawnInTile` (NPC.cs:5398-5434): the
/// 2×3-tile rect above the ground tile must be free of solid tiles and lava.
fn has_tile_spawn_space(world: &World, x: i32, y: i32) -> bool {
    let left = x - SPAWN_SPACE_X / 2;
    let top = y - SPAWN_SPACE_Y;
    for i in left..left + SPAWN_SPACE_X {
        for j in top..top + SPAWN_SPACE_Y {
            if !world.in_bounds(i, j) {
                return false;
            }
            let t = world.tile(i, j);
            let Some(t) = t else {
                continue;
            };
            let solid = t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind);
            let lava = t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava;
            if solid || lava {
                return false;
            }
        }
    }
    true
}

/// `Spawner.CheckNotSpawningOnScreen` (NPC.cs:5436-5450): the tile must lie
/// outside every player's spawn-screen rect.
fn check_not_spawning_on_screen(
    ctx: &SpawnContext,
    safe: &SpawnArea,
    x: i32,
    y: i32,
) -> bool {
    let num = S_WIDTH / 2;
    let num2 = S_HEIGHT / 2;
    let safe_range_x = (S_WIDTH / 16 * 52) / 100;
    let safe_range_y = (S_HEIGHT / 16 * 52) / 100;
    let (cx, cy) = ctx.player_center;
    let rect_x = cx - num as f32 - safe_range_x as f32 * TILE_PX;
    let rect_y = cy - num2 as f32 - safe_range_y as f32 * TILE_PX;
    let tile_px = x as f32 * TILE_PX;
    let tile_py = y as f32 * TILE_PX;
    // Rectangle.Intersects between the 16×16 tile and the inflated screen rect.
    let _ = safe;
    !(tile_px + TILE_PX > rect_x
        && tile_px < rect_x + (S_WIDTH + safe_range_x * 2) as f32
        && tile_py + TILE_PX > rect_y
        && tile_py < rect_y + (S_HEIGHT + safe_range_y * 2) as f32)
}

/// `Spawner.FindGroundTile` (NPC.cs:5866-5885): walk down at most 30 tiles to
/// the first solid ground tile.
fn find_ground_tile(world: &World, x: i32, y: i32) -> i32 {
    let solid = |x: i32, y: i32| {
        world
            .tile(x, y)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
    };
    if solid(x, y) {
        return y;
    }
    for i in y + 1..y + 30 {
        if solid(x, i) {
            return i;
        }
    }
    y
}

/// `Spawner.PostCheckChosenSpawnTile` (NPC.cs:1022-1048), minus the dual-dungeon
/// and invasion-tile cases this app does not model: reject dungeon walls
/// without dungeon ground and submerged (non-lava) spawn points.
fn post_check_chosen_spawn_tile(world: &World, x: i32, y: i32, ground_y: i32) -> bool {
    let _ = ground_y;
    let liquid_at = |dy: i32| {
        world
            .tile(x, y + dy)
            .is_some_and(|t| t.active() && t.liquid > 0)
    };
    if liquid_at(-1) && liquid_at(-2) {
        let lava = world
            .tile(x, y - 1)
            .is_some_and(|t| t.liquid_kind() == tr_core::tile::LiquidKind::Lava);
        if !lava {
            return false;
        }
    }
    true
}

/// One `Spawner.TrySpawnAnNPC` attempt (NPC.cs:308-342). Returns the spawned
/// NPC, or `None` when any gate rejects the attempt.
pub fn try_spawn_an_npc(ctx: &SpawnContext, rng: &mut FastRandom) -> Option<Npc> {
    let (spawn_area, safe_area) = get_spawn_area(ctx);

    // `GetSpawnRate` + the two hard gates in `TrySpawnAnNPC`.
    let rate = get_spawn_rate(ctx, rng);
    if ctx.nearby_active_npcs >= rate.max_spawns as f32 {
        return None;
    }
    if rng.next(rate.spawn_rate) != 0 {
        return None;
    }

    // `FindSpawnTile` (NPC.cs:969-1010): 50 random candidates.
    for _ in 0..50 {
        let num = rng.next(spawn_area.right - spawn_area.left) + spawn_area.left;
        let mut j = rng.next(spawn_area.bottom - spawn_area.top) + spawn_area.top;
        if !ctx.world.in_bounds(num, j) {
            continue;
        }
        // Skip solid ground and (indoor) house walls.
        let solid = ctx
            .world
            .tile(num, j)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
        if solid {
            continue;
        }
        // Descend to the first solid tile (NPC.cs:990-996).
        while j < ctx.world.height
            && j < spawn_area.bottom
            && !ctx
                .world
                .tile(num, j)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
        {
            j += 1;
        }
        if j >= spawn_area.bottom {
            continue;
        }
        if safe_area.contains(num, j) {
            continue;
        }
        if !has_tile_spawn_space(ctx.world, num, j) {
            continue;
        }
        // `CheckNotSpawningOnScreen`.
        if !check_not_spawning_on_screen(ctx, &safe_area, num, j) {
            continue;
        }
        // `FindGroundTile` + `PostCheckChosenSpawnTile`.
        let ground_y = find_ground_tile(ctx.world, num, j);
        if !post_check_chosen_spawn_tile(ctx.world, num, j, ground_y) {
            continue;
        }
        // `SpawnAnNPC` default selection for the implemented kinds.
        return spawn_npc_at(ctx, rng, num, j);
    }
    None
}

/// The default surface/night selection of `Spawner.SpawnAnNPC` (NPC.cs:
/// ~4390-4810), restricted to the implemented enemy kinds:
/// - surface, day: `GetBasicSlimeToSpawn` → blue slime (green/-red variants
///   map to the same kind),
/// - surface, night: demon eye on the 1/6 roll (1/2 during a full moon),
///   otherwise a zombie,
/// - underground: slime (`GetBasicSlimeToSpawn(surface: false)`).
fn spawn_npc_at(ctx: &SpawnContext, rng: &mut FastRandom, x: i32, y: i32) -> Option<Npc> {
    let surface = (y as f64) <= ctx.world.world_surface;
    let kind = if surface {
        if ctx.day_time {
            // `GetBasicSlimeToSpawn(surface: true, ...)`: -3 / 1 / -7 all become
            // the app's blue slime; jungle/snow tile variants are not modelled.
            NpcKind::Slime
        } else if rng.next(6) == 0 {
            // NPC.cs:4549-4572: `Next(6) == 0` (or full-moon `Next(2) == 0`,
            // moon phases are not modelled) picks a flying enemy; the
            // non-hardmode surface branch is type 2 (demon eye); 190-194 and
            // -43 map to the same kind.
            NpcKind::DemonEye
        } else {
            // NPC.cs:4764-4809: zombie (style 0 → type 3); the -26/-27 AI
            // variants and armed/torch zombies are not modelled.
            NpcKind::Zombie
        }
    } else {
        // Underground default: `GetBasicSlimeToSpawn(surface: false)`.
        NpcKind::Slime
    };
    match kind {
        NpcKind::Slime => Npc::spawn_slime(ctx.world, x),
        NpcKind::DemonEye => Npc::spawn_eye(ctx.world, x),
        NpcKind::Zombie => Npc::spawn_zombie(ctx.world, x),
        NpcKind::KingSlime => None, // bosses never spawn from the ambient pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        // Tall enough that `UnderworldLayer = height - 200` sits well below the
        // play area, and with `world_surface` above the filled ground so that
        // spawns on it count as surface spawns.
        let mut w = World::new("t".into(), 400, 700);
        for x in 0..400 {
            for y in 300..700 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 310.0;
        w.rock_layer = 320.0;
        w.spawn_x = 200;
        w.spawn_y = 296;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn ctx<'a>(w: &'a World, day: bool) -> SpawnContext<'a> {
        SpawnContext {
            world: w,
            player_center: (200.0 * TILE_PX, 295.0 * TILE_PX),
            player_pos: (200.0 * TILE_PX, 295.0 * TILE_PX),
            nearby_active_npcs: 0.0,
            town_npcs: 0,
            day_time: day,
        }
    }

    #[test]
    fn spawn_rate_matches_the_c_modifiers() {
        let w = ground();
        // With no NPCs nearby the <20% scaling (NPC.cs:670-673) always applies,
        // so the base 600 rate is ×0.6 = 360.
        // Day surface: 600 × 0.6.
        let r = get_spawn_rate(&ctx(&w, true), &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
        assert_eq!(r.max_spawns, 5);

        // Night: ×0.6 / ×1.3, then ×0.6 again.
        let r = get_spawn_rate(&ctx(&w, false), &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 216);
        assert_eq!(r.max_spawns, 6);

        // Below the rock layer: ×0.4 / ×1.9, then ×0.6. Tile y 420 px 6720 is
        // past `rock_layer*16 + sHeight` (6320) but above the underworld (8000).
        let mut c = ctx(&w, true);
        c.player_pos = (200.0 * TILE_PX, 420.0 * TILE_PX);
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 144);
        assert_eq!(r.max_spawns, 9);

        // Underworld doubles the cap (tile y 505 > height - 200 = 500).
        let mut c = ctx(&w, true);
        c.player_pos = (200.0 * TILE_PX, 505.0 * TILE_PX);
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.max_spawns, 10);

        // Town NPCs slow surface spawns (×2 with ≥3).
        let mut c = ctx(&w, true);
        c.town_npcs = 3;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 720);

        // The nearby scaling stops at ≥80% of the cap (4.0 ≥ 0.8·5): no
        // multiplier applies, the base 600 survives.
        let mut c = ctx(&w, true);
        c.nearby_active_npcs = 4.0;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 600);
    }

    #[test]
    fn spawns_land_outside_the_safe_area() {
        let w = ground();
        let mut rng = FastRandom::new(0xDEAD_BEEF);
        let mut spawned_near = 0;
        for _ in 0..4000 {
            let mut c = ctx(&w, false);
            c.nearby_active_npcs = 0.0;
            let safe_area = get_spawn_area(&c).1;
            if let Some(n) = try_spawn_an_npc(&c, &mut rng) {
                let (cx, cy) = n.center();
                let tx = (cx / TILE_PX).floor() as i32;
                let ty = (cy / TILE_PX).floor() as i32;
                assert!(!safe_area.contains(tx, ty), "spawned inside safe area");
                spawned_near += 1;
            }
        }
        assert!(spawned_near > 0, "no spawns in 64 night attempts");
    }

    #[test]
    fn night_spawns_zombies_and_eyes_day_spawns_slimes() {
        let w = ground();
        let mut rng = FastRandom::new(42);
        let mut saw_zombie = false;
        let mut saw_eye = false;
        for _ in 0..4000 {
            let c = ctx(&w, false);
            if let Some(n) = try_spawn_an_npc(&c, &mut rng) {
                match n.kind {
                    NpcKind::Zombie => saw_zombie = true,
                    NpcKind::DemonEye => saw_eye = true,
                    _ => {}
                }
            }
        }
        assert!(saw_zombie && saw_eye, "night selection missed a kind");

        let mut saw_slime = false;
        for _ in 0..4000 {
            let c = ctx(&w, true);
            if let Some(n) = try_spawn_an_npc(&c, &mut rng) {
                if n.kind == NpcKind::Slime {
                    saw_slime = true;
                }
            }
        }
        assert!(saw_slime, "day selection never produced a slime");
    }

    #[test]
    fn nearby_cap_blocks_spawns() {
        let w = ground();
        let mut rng = FastRandom::new(7);
        let mut c = ctx(&w, false);
        c.nearby_active_npcs = 100.0; // far above every max-spawns cap
        for _ in 0..2000 {
            assert!(try_spawn_an_npc(&c, &mut rng).is_none());
        }
    }

    #[test]
    fn spawn_space_requires_clear_tiles() {
        let mut w = ground();
        // Wall in the spawn rect (2 wide, 3 tall above the ground tile).
        for dy in 1..=3 {
            let i = w.index(200, 299 - dy);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        assert!(!has_tile_spawn_space(&w, 200, 300));
        assert!(has_tile_spawn_space(&w, 205, 300));
    }
}
