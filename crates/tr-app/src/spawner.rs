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

use crate::npc::{Npc, NpcKind, SlimeVariant};

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
/// The world/scene state `GetSpawnRate` reads (`NPC.cs:473-810`). Everything
/// the ported simulation cannot supply yet stays `false`/`0` — the chain
/// branches still run in the vanilla order so future systems only flip flags.
#[derive(Clone, Copy, Default)]
pub struct SpawnFlags {
    pub hard_mode: bool,
    pub blood_moon: bool,
    pub eclipse: bool,
    /// `Main.pumpkinMoon || Main.snowMoon`.
    pub moon_event: bool,
    pub zone_dungeon: bool,
    /// `NPC.downedBoss3` (Skeletron defeated).
    pub downed_boss3: bool,
    pub zone_sandstorm: bool,
    pub zone_underground_desert: bool,
    pub zone_jungle: bool,
    pub zone_evil: bool,
    pub zone_meteor: bool,
    pub zone_temple: bool,
    pub zone_hallow_deep: bool,
    pub zone_water_candle: bool,
    pub zone_peace_candle: bool,
    pub invis: bool,
    pub calmed: bool,
    pub sunflower: bool,
    pub angler_set: bool,
    /// `Player.enemySpawns` (the battle potion).
    pub enemy_spawns: bool,
    /// `Main.wofNPCIndex >= 0` (the Wall of Flesh is alive).
    pub wof_active: bool,
}

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
    pub flags: SpawnFlags,
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
/// `Spawner.GetSpawnRate` (`NPC.cs:473-810`): the sequential multiplier
/// chain, in the vanilla order with the vanilla cast widths (`spawn_rate`
/// steps are `(double)`-then-`(int)`, `max_spawns` steps `(float)`-then-
/// `(int)`). The seed/FTW/remix/journey/skyblock branches are constants-off
/// until those seed systems land (M5-SEED-1, M6-CREATIVE-1).
pub fn get_spawn_rate(ctx: &SpawnContext, rng: &mut FastRandom) -> SpawnRate {
    let _ = rng;
    let f = &ctx.flags;
    let mut rate = SpawnRate {
        spawn_rate: DEFAULT_SPAWN_RATE,
        max_spawns: DEFAULT_MAX_SPAWNS,
    };
    let underworld = UNDERWORLD_LAYER(ctx.world.height) as f64 * 16.0;
    let rock_px = ctx.world.rock_layer * 16.0 + S_HEIGHT as f64;
    let surface_px = ctx.world.world_surface * 16.0 + S_HEIGHT as f64;
    let pos_y = ctx.player_pos.1 as f64;
    let tile_y = pos_y / TILE_PX as f64;

    // `Main.hardMode` (`:478-483`).
    if f.hard_mode {
        rate.spawn_rate = (DEFAULT_SPAWN_RATE as f64 * 0.9) as i32;
        rate.max_spawns = DEFAULT_MAX_SPAWNS + 1;
    }
    // The depth ladder — underworld / deep / below-surface / night / eclipse,
    // each an else-if of the previous (`:484-558`).
    if pos_y > underworld {
        rate.max_spawns = (rate.max_spawns as f32 * 2.0) as i32;
    } else if pos_y > rock_px {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.4) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.9) as i32;
    } else if pos_y > surface_px {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.5) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.7) as i32;
    } else if !ctx.day_time {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.6) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.3) as i32;
        if f.blood_moon {
            rate.spawn_rate = (rate.spawn_rate as f64 * 0.3) as i32;
            rate.max_spawns = (rate.max_spawns as f32 * 1.8) as i32;
        }
        if f.moon_event && (pos_y / 16.0) < ctx.world.world_surface {
            rate.spawn_rate = (rate.spawn_rate as f64 * 0.2) as i32;
            rate.max_spawns *= 2;
        }
    } else if ctx.day_time && f.eclipse {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.2) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.9) as i32;
    }
    // The biome/zone branches — the dungeon group is an if, the desert family
    // an else-if ladder, jungle by the town count, then the rest
    // (`:573-657`).
    if f.zone_dungeon {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.3) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.8) as i32;
    } else if f.zone_sandstorm {
        rate.spawn_rate = (rate.spawn_rate as f32 * if f.hard_mode { 0.4 } else { 0.9 }) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * if f.hard_mode { 1.5 } else { 1.2 }) as i32;
    } else if f.zone_underground_desert {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.2) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 3.0) as i32;
    } else if f.zone_jungle {
        // The town-pressure tiers (`:604-621`).
        match ctx.town_npcs {
            0 => {
                rate.spawn_rate = (rate.spawn_rate as f64 * 0.4) as i32;
                rate.max_spawns = (rate.max_spawns as f32 * 1.5) as i32;
            }
            1 => {
                rate.spawn_rate = (rate.spawn_rate as f64 * 0.55) as i32;
                rate.max_spawns = (rate.max_spawns as f64 * 1.4) as i32;
            }
            2 => {
                rate.spawn_rate = (rate.spawn_rate as f64 * 0.7) as i32;
                rate.max_spawns = (rate.max_spawns as f32 * 1.3) as i32;
            }
            _ => {
                rate.spawn_rate = (rate.spawn_rate as f64 * 0.85) as i32;
                rate.max_spawns = (rate.max_spawns as f32 * 1.2) as i32;
            }
        }
    } else if f.zone_evil {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.65) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.3) as i32;
    } else if f.zone_meteor {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.4) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.1) as i32;
    }
    if f.zone_temple {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.8) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.2) as i32;
    }
    if f.zone_hallow_deep && pos_y > rock_px {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.65) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 1.3) as i32;
    }
    // The Wall of Flesh underworld override (`:667-672`).
    if f.wof_active && pos_y > underworld {
        rate.max_spawns = (rate.max_spawns as f32 * 0.3) as i32;
        rate.spawn_rate *= 3;
    }
    // The nearby-NPC ramps — the always pass (`:673-690`), then the extra
    // deep/evil pass (`:691-701`).
    let nearby = ctx.nearby_active_npcs as f64;
    let max_f = rate.max_spawns as f64;
    if nearby < max_f * 0.2 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.6) as i32;
    } else if nearby < max_f * 0.4 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.7) as i32;
    } else if nearby < max_f * 0.6 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.8) as i32;
    } else if nearby < max_f * 0.8 {
        rate.spawn_rate = (rate.spawn_rate as f32 * 0.9) as i32;
    }
    if tile_y > (ctx.world.world_surface + ctx.world.rock_layer) / 2.0 || f.zone_evil {
        if nearby < max_f * 0.2 {
            rate.spawn_rate = (rate.spawn_rate as f32 * 0.7) as i32;
        } else if nearby < max_f * 0.4 {
            rate.spawn_rate = (rate.spawn_rate as f32 * 0.9) as i32;
        }
    }
    // The player-state and candle modifiers (`:702-740`).
    if f.invis {
        rate.spawn_rate = (rate.spawn_rate as f32 * 1.2) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 0.8) as i32;
    }
    if f.calmed {
        rate.spawn_rate = (rate.spawn_rate as f32 * 1.65) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 0.6) as i32;
    }
    if f.sunflower {
        rate.spawn_rate = (rate.spawn_rate as f32 * 1.2) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 0.8) as i32;
    }
    if f.angler_set {
        rate.spawn_rate = (rate.spawn_rate as f32 * 1.3) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 0.7) as i32;
    }
    if f.enemy_spawns {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.5) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 2.0) as i32;
    }
    if f.zone_water_candle {
        if !f.zone_peace_candle {
            rate.spawn_rate = (rate.spawn_rate as f64 * 0.75) as i32;
            rate.max_spawns = (rate.max_spawns as f32 * 1.5) as i32;
        }
    } else if f.zone_peace_candle {
        rate.spawn_rate = (rate.spawn_rate as f64 * 1.3) as i32;
        rate.max_spawns = (rate.max_spawns as f32 * 0.7) as i32;
    }
    if f.zone_water_candle && tile_y < ctx.world.world_surface * 0.349_999_994 {
        rate.spawn_rate = (rate.spawn_rate as f64 * 0.5) as i32;
    }
    // The caps (`:741-751`): the rate floors at 10 % of the default, the cap
    // at 3x the default.
    if (rate.spawn_rate as f64) < DEFAULT_SPAWN_RATE as f64 * 0.1 {
        rate.spawn_rate = (DEFAULT_SPAWN_RATE as f64 * 0.1) as i32;
    }
    if rate.max_spawns > DEFAULT_MAX_SPAWNS * 3 {
        rate.max_spawns = DEFAULT_MAX_SPAWNS * 3;
    }
    // The invasion overrides (`:792-801`) — the moon events force the raid
    // pacing; the pre-Skeletron dungeon crawls at rate 10 (`:806-809`).
    if f.moon_event {
        rate.max_spawns = (DEFAULT_MAX_SPAWNS as f64 * (2.0 + 0.3 * 1.0)) as i32;
        rate.spawn_rate = 20;
    }
    if f.zone_dungeon && !f.downed_boss3 {
        rate.spawn_rate = 10;
    }
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
fn check_not_spawning_on_screen(ctx: &SpawnContext, safe: &SpawnArea, x: i32, y: i32) -> bool {
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
            // `GetBasicSlimeToSpawn(surface: true, tileType, dist)` — the
            // forest default (jungle tile 60 -> -10 and snow 147/161 stay out
            // until those biomes exist).
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
    if kind == NpcKind::Slime {
        // `Spawner.GetBasicSlimeToSpawn` (`NPC.cs:5625-5654`) picks the
        // variant, then `Spawner.SpawnNPC`'s blue-slime roll can still swap
        // Pinky in (`NPC.cs:5238-5246`: `RollLuck(180) == 0`).
        let dist = (x - ctx.world.spawn_x).abs();
        let variant = if surface {
            if rng.next(3) == 0 || dist < 200 {
                SlimeVariant::Green
            } else if rng.next(10) == 0 && dist > 400 {
                SlimeVariant::Purple
            } else {
                SlimeVariant::Blue
            }
        } else if rng.next(5) == 0 {
            SlimeVariant::Yellow
        } else if rng.next(2) == 0 {
            SlimeVariant::Blue
        } else {
            SlimeVariant::Red
        };
        let variant = if variant == SlimeVariant::Blue && rng.next(180) == 0 {
            SlimeVariant::Pinky
        } else {
            variant
        };
        return Npc::spawn_slime_variant(ctx.world, x, variant);
    }
    match kind {
        NpcKind::DemonEye => Npc::spawn_eye(ctx.world, x),
        // `NPCID.WanderingEye` (133) spawns only on the night hardmode roll
        // (`NPC.cs:4555-4558`); the port is pre-hardmode, so it stays out of
        // the ambient pass (it rides the eye machine when debug-spawned).
        NpcKind::WanderingEye => Npc::spawn_eye(ctx.world, x),
        NpcKind::Zombie => Npc::spawn_zombie(ctx.world, x),
        NpcKind::Slime | NpcKind::KingSlime => None, // handled above / boss only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(world: &World, y_px: f32, day: bool) -> SpawnContext<'_> {
        SpawnContext {
            world,
            player_center: (0.0, y_px),
            player_pos: (0.0, y_px),
            // 100 nearby NPCs clears every nearby-ramp threshold
            // (NPC.cs:673-690), isolating these tests from the ramp.
            nearby_active_npcs: 100.0,
            town_npcs: 0,
            day_time: day,
            flags: SpawnFlags::default(),
        }
    }

    fn ground() -> World {
        // 800 tiles tall so `UnderworldLayer` (= height - 200) sits below the
        // tested depth bands.
        let mut w = World::new("t".into(), 64, 800);
        w.world_surface = 30.0;
        w.rock_layer = 40.0;
        w
    }

    #[test]
    fn surface_day_is_the_plain_default() {
        // No branch applies on the surface at day: 600/5 verbatim.
        let w = ground();
        let r = get_spawn_rate(&ctx(&w, 25.0 * 16.0, true), &mut FastRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (600, 5));
    }

    #[test]
    fn night_surface_multiplies() {
        // NPC.cs:540-544: rate x0.6 -> 360, max x1.3 -> (float)6.5 -> 6.
        let w = ground();
        let r = get_spawn_rate(&ctx(&w, 25.0 * 16.0, false), &mut FastRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (360, 6));
    }

    #[test]
    fn deep_rock_layer_multiplies() {
        // NPC.cs:487-491: rate x0.4 -> 240, max x1.9 -> 9.5 -> 9. The position
        // must clear rock_layer*16 + sHeight (640 + 1200 = 1840 px = tile 115).
        let w = ground();
        let r = get_spawn_rate(&ctx(&w, 120.0 * 16.0, true), &mut FastRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (240, 9));
    }

    #[test]
    fn the_nearby_ramp_speeds_up_empty_areas() {
        // NPC.cs:673-677: nearby < max x 0.2 multiplies the rate by 0.6 —
        // smaller = more frequent spawns. Day surface: 600 x 0.6 = 360.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.nearby_active_npcs = 0.0;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
    }

    #[test]
    fn blood_moon_night_compounds() {
        // NPC.cs:545-548: night x0.6/x1.3 then blood x0.3/x1.8:
        // 600 -> 360 -> 108; 6.5 -> 11.7 -> 11.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.flags.blood_moon = true;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (108, 10));
    }

    #[test]
    fn jungle_town_tiers() {
        // NPC.cs:604-621: town 0 x0.4/x1.5, town 1 x0.55/x1.4, town 2 x0.7/x1.3.
        let w = ground();
        for (town, rate, max) in [(0, 96, 13), (1, 132, 12), (2, 168, 11)] {
            let mut c = ctx(&w, 120.0 * 16.0, true);
            c.flags.zone_jungle = true;
            c.town_npcs = town;
            let r = get_spawn_rate(&c, &mut FastRandom::new(1));
            assert_eq!(r.spawn_rate, rate, "jungle town {town} rate");
            assert_eq!(r.max_spawns, max, "jungle town {town} max");
        }
    }

    #[test]
    fn dungeon_pre_skeletron_is_rate_ten() {
        // NPC.cs:806-809: ZoneDungeon && !downedBoss3 -> spawnRate = 10 (the
        // chain multipliers run first; the override is last).
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.flags.zone_dungeon = true;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 10);
        // Post-Skeletron the override stops and the x0.3 branch stands.
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.flags.zone_dungeon = true;
        c.flags.downed_boss3 = true;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 180);
    }

    #[test]
    fn the_rate_floor_and_cap_hold() {
        // NPC.cs:741-751: rate >= 60 (600x0.1), max <= 15 (5x3). The deepest
        // vanilla-compoundable stack here: underground desert x0.2/x3 capped.
        let w = ground();
        let mut c = ctx(&w, 120.0 * 16.0, true);
        c.flags.zone_underground_desert = true;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        // 600 x0.4 (deep) x0.2 (UD) = 48 -> floored at 60 (NPC.cs:741-744).
        assert_eq!(r.spawn_rate, 60);
        assert_eq!(r.max_spawns, 15, "x3 of 5 capped at 15");
    }

    #[test]
    fn moon_event_forces_the_raid_pacing() {
        // NPC.cs:792-796: maxSpawns = 5 x (2 + 0.3x1) = 11, spawnRate = 20.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.flags.moon_event = true;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (20, 11));
    }

    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground_tall() -> World {
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

    fn ctx_at<'a>(w: &'a World, day: bool) -> SpawnContext<'a> {
        SpawnContext {
            world: w,
            player_center: (200.0 * TILE_PX, 295.0 * TILE_PX),
            player_pos: (200.0 * TILE_PX, 295.0 * TILE_PX),
            nearby_active_npcs: 0.0,
            town_npcs: 0,
            day_time: day,
            flags: SpawnFlags::default(),
        }
    }

    #[test]
    fn spawn_rate_matches_the_c_modifiers() {
        let w = ground_tall();
        // With no NPCs nearby the <20% scaling (NPC.cs:670-673) always applies,
        // so the base 600 rate is ×0.6 = 360.
        // Day surface: 600 × 0.6.
        let r = get_spawn_rate(&ctx_at(&w, true), &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
        assert_eq!(r.max_spawns, 5);

        // Night: ×0.6 / ×1.3, then ×0.6 again.
        let r = get_spawn_rate(&ctx_at(&w, false), &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 216);
        assert_eq!(r.max_spawns, 6);

        // Below the rock layer: ×0.4 / ×1.9, then both nearby ramps (the
        // x0.6 pass and the deep x0.7 extra pass; tile y 420 px 6720 is past
        // `rock_layer*16 + sHeight` (6320) and past the mid-depth line 315):
        // 600 -> 240 -> 144 -> 100.8 -> 100.
        let mut c = ctx_at(&w, true);
        c.player_pos = (200.0 * TILE_PX, 420.0 * TILE_PX);
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 100);
        assert_eq!(r.max_spawns, 9);

        // Underworld doubles the cap (tile y 505 > height - 200 = 500).
        let mut c = ctx_at(&w, true);
        c.player_pos = (200.0 * TILE_PX, 505.0 * TILE_PX);
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.max_spawns, 10);

        // The 1.4.5 GetSpawnRate has no town-NPC multiplier of its own
        // (the pressure rides the nearby ramp; NPC.cs:673-690).
        let mut c = ctx_at(&w, true);
        c.town_npcs = 3;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 360);

        // The nearby scaling stops at ≥80% of the cap (4.0 ≥ 0.8·5): no
        // multiplier applies, the base 600 survives.
        let mut c = ctx_at(&w, true);
        c.nearby_active_npcs = 4.0;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 600);
    }

    #[test]
    fn spawns_land_outside_the_safe_area() {
        let w = ground_tall();
        let mut rng = FastRandom::new(0xDEAD_BEEF);
        let mut spawned_near = 0;
        for _ in 0..4000 {
            let mut c = ctx_at(&w, false);
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
        let w = ground_tall();
        let mut rng = FastRandom::new(42);
        let mut saw_zombie = false;
        let mut saw_eye = false;
        for _ in 0..4000 {
            let c = ctx_at(&w, false);
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
            let c = ctx_at(&w, true);
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
        let w = ground_tall();
        let mut rng = FastRandom::new(7);
        let mut c = ctx_at(&w, false);
        c.nearby_active_npcs = 100.0; // far above every max-spawns cap
        for _ in 0..2000 {
            assert!(try_spawn_an_npc(&c, &mut rng).is_none());
        }
    }

    #[test]
    fn spawn_space_requires_clear_tiles() {
        let mut w = ground_tall();
        // Wall in the spawn rect (2 wide, 3 tall above the ground tile):
        // rows 297..=299.
        for dy in 1..=3 {
            let i = w.index(200, 300 - dy);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        assert!(!has_tile_spawn_space(&w, 200, 300));
        assert!(has_tile_spawn_space(&w, 205, 300));
    }
}
