//! Port of the NPC spawning pipeline in `Terraria/NPC.cs` (`Spawner`):
//! `TrySpawnAnNPC` — spawn-rate modifiers, the spawn-area scan, the on-screen
//! and free-space checks, and the default surface day/night enemy selection
//! for the kinds implemented in `npc.rs`.
//!
//! Biome-specific spawn tables (corruption, jungle, hallow, dungeon, ...) are
//! part of the much larger `Spawner.SpawnAnNPC` switch and are out of scope
//! here; when a chosen C# type has no `NpcKind` equivalent the attempt simply
//! returns `None` (the original would spawn that type).

use tr_core::{World, TILE_PX};

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
    /// `Player.ZoneGraveyard` — the town block's graveyard ladders read it
    /// (`NPC.cs:860/:883/:905`). No scene source yet (the graveyard scan is
    /// the M2-BUFF family's) — constants-off; the arms stay reachable from
    /// tests.
    pub zone_graveyard: bool,
    pub invis: bool,
    pub calmed: bool,
    pub sunflower: bool,
    pub angler_set: bool,
    /// `Player.enemySpawns` (the battle potion).
    pub enemy_spawns: bool,
    /// `Main.wofNPCIndex >= 0` (the Wall of Flesh is alive).
    pub wof_active: bool,
}

/// `Main.wallLight` (`Main.cs:10578-10595`) — the wall list
/// `SetSpawnFlags`' `skyBehindPlayer` gate reads (plus wall 73's separate
/// `||` arm, `NPC.cs:412`).
pub(crate) const LIGHT_WALLS: [u16; 16] = [
    0, 21, 318, 106, 317, 107, 138, 140, 141, 139, 145, 150, 152, 245, 315, 168,
];

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
    /// `player.statLifeMax` — `playerHasStartingHealth`'s gate
    /// (`NPC.cs:415`: `<= 100`) halves the torch-zombie denominator.
    pub player_life_max: i32,
    /// The scene centre below `Main.UnderworldLayer` (`M6-MUSIC-1`'s
    /// depth band) — the bat-layer selection reads it.
    pub underworld_for_bats: bool,
    pub flags: SpawnFlags,
    /// The bound-Tinkerer roll gate: `downedGoblins && !savedGoblin &&
    /// !AnyNPCs(105/107)` — the world_sim pass computes it.
    pub bound_goblin_ok: bool,
    /// `NPC.cs:2088`'s guards for the tied Wizard: hardmode, the rescue
    /// not yet raised, nobody bound/freed on the map.
    pub bound_wizard_ok: bool,
    /// `NPC.cs:2653`'s guards for the tied Mechanic: Skeletron downed, the
    /// rescue not yet raised, nobody bound/freed on the map.
    pub bound_mech_ok: bool,
    /// `spawnTileY > (worldSurface * 4 + rockLayer) / 5` — the Mechanic's
    /// deep-dungeon depth band (`NPC.cs:2653`).
    pub below_deep_dungeon: bool,
    /// `Spawner.invaders` (`NPC.cs:395`'s `ShouldSpawnInvasionEnemies`) —
    /// gates the skyMob detection (`FindSpawnTile`'s `!this.invaders`).
    pub invaders: bool,
    /// `NPC.downedGolemBoss` — the probe arms' gate.
    pub downed_golem: bool,
    /// `NPC.downedMartians` — the probe rolls' short-circuit.
    pub downed_martians: bool,
    /// `Main.invasionType` — the skyMob dispatch's drone arm.
    pub invasion_type: i32,
    /// The invasion pick's roster gates, read once per attempt (the C#
    /// reads `NPC.AnyNPCs(...)` live inside `SpawnAnNPC`'s invaders ladder):
    /// `!AnyNPCs(471)`'s Summoner gate (`:1430`), `!AnyNPCs(216)`'s Captain
    /// gate (`:1471`), `!AnyNPCs(520)`'s Walker walk-in gate (`:1517`) and
    /// `!AnyNPCs(395)`'s Saucer gate (`:1492`).
    pub invasion_summoner_alive: bool,
    pub invasion_captain_alive: bool,
    pub invasion_walker_alive: bool,
    pub invasion_saucer_alive: bool,
    /// `(invasionSizeStart - invasionSize) / invasionSizeStart >= 0.3f` —
    /// the martian table's Saucer window (`:1492`).
    pub invasion_progress_30: bool,
    /// The same drained fraction — the pirate table's Flying Dutchman
    /// window (`:1460-1468`).
    pub invasion_progress_frac: f32,
    /// `NPC.AnyDanger()` (`NPC.cs:80243`): an invasion, a moon event or
    /// any active boss NPC — the probe arms' edge-band gate.
    pub any_danger: bool,
    /// `NPC.AnyNPCs(399)`: the probe is unique.
    pub probe_exists: bool,
    /// `Spawner.skyBehindPlayer` (`NPC.cs:412`): the wall behind the
    /// player's tile is light-passing (or wall 73) — the skyMob probe
    /// arm's open-sky gate.
    pub sky_behind_player: bool,
}

/// Result of `GetSpawnRate`: frames between spawn rolls and the concurrent cap,
/// plus the town block's carry-outs — the C# writes `noWorms`/`spawnFriendly`
/// onto the spawner instance and the same spawn attempt reads them downstream
/// (the skyMob gates, the water arms, the worm rolls), so the port threads
/// them through the return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnRate {
    pub spawn_rate: i32,
    pub max_spawns: i32,
    /// `NPC.noWorms` after the town block
    /// (`NPC.cs:809/:826/:842/:857/:882/:904`).
    pub no_worms: bool,
    /// `NPC.spawnFriendly` after the town block
    /// (`NPC.cs:813/:830/:846/:865/:871/:888/:894/:910/:918`).
    pub spawn_friendly: bool,
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

/// `Spawner.GetSpawnRate` (`NPC.cs:473-929`): the sequential multiplier
/// chain, in the vanilla order with the vanilla cast widths (`spawn_rate`
/// steps are `(double)`-then-`(int)`, `max_spawns` steps `(float)`-then-
/// `(int)`), through the town-NPC distribution block (`:799-923`) whose
/// `Main.rand` draws ride the caller's stream — the C#'s `Main.rand` inside
/// the SpawnNPC stage (the port's bridge stream until S1-A-11). The
/// seed/FTW/remix/journey/skyblock branches are constants-off until those
/// seed systems land (M5-SEED-1, M6-CREATIVE-1). The luck tail
/// (`:924-928`, `RollOnlyBadLuckExtreme`) is out of scope (the spawner has
/// no luck input yet).
pub fn get_spawn_rate(ctx: &SpawnContext, rng: &mut FastRandom) -> SpawnRate {
    let f = &ctx.flags;
    let mut rate = SpawnRate {
        spawn_rate: DEFAULT_SPAWN_RATE,
        max_spawns: DEFAULT_MAX_SPAWNS,
        // `SetSpawnFlags`' per-attempt reset (`NPC.cs:373` — `spawnFriendly
        // = false`; `noWorms` takes the player-position wall-house write
        // `:410`, unmodelled — the port has no `wallHouse` table).
        no_worms: false,
        spawn_friendly: false,
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
    // The invaders override (NPC.cs:781-785): same pacing as moon events.
    if ctx.invaders {
        rate.max_spawns = (DEFAULT_MAX_SPAWNS as f64 * (2.0 + 0.3 * 1.0)) as i32;
        rate.spawn_rate = 20;
    }
    if f.zone_dungeon && !f.downed_boss3 {
        rate.spawn_rate = 10;
    }
    // The town-NPC distribution block (`NPC.cs:799-923`): with no invasion,
    // no event window (blood/pumpkin/snow — unless it is day), no eclipse
    // (unless it is night), no evil/crimson zone, no meteor zone and no
    // Old-One-army zone, the town count picks a ladder and draws `Main.rand`
    // for the `noWorms` latch and the `spawnFriendly`/spawnRate rescale.
    // `flag = ZoneCorrupt || ZoneCrimson` (`:794-798`) folded with the
    // explicit `!ZoneCrimson` term is one `!corrupt && !crimson` check — the
    // port's `zone_evil`. `ZoneOldOneArmy` has no scene source yet (S1-E-12)
    // and `inRemixStartingArea` (`:801`, remix-only) is constants-off — both
    // false. `WorldGen.Skyblock.lowTiles` (the underworld-1 guard `:805` /
    // surface-1 guard `:858`), the shadow candle's `townNPCs = 0`
    // (`:419-424`) and `Main.expertMode` (the World carries no game mode)
    // are constants-off too. The draws ride the caller's stream in the C#'s
    // order; `townNPCs == 0` matches no arm and draws nothing.
    if !ctx.invaders
        && ((!f.blood_moon && !f.moon_event) || ctx.day_time)
        && (!f.eclipse || !ctx.day_time)
        && !f.zone_evil
        && !f.zone_meteor
    {
        // `player.Center.Y / 16f > (float)Main.UnderworldLayer` (`:801`).
        if ctx.player_center.1 / 16.0 > UNDERWORLD_LAYER(ctx.world.height) as f32 {
            if ctx.town_npcs == 1 {
                // The skyblock guard (`:805`) is constants-off.
                if rng.next(2) == 0 {
                    rate.no_worms = true; // `:809`
                }
                if rng.next(10) == 0 {
                    rate.spawn_friendly = true; // `:813`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.5) as i32;
                // `:814`
                } else {
                    rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 1.25) as i32;
                    // `:818`
                }
            } else if ctx.town_npcs == 2 {
                if rng.next(4) != 0 {
                    rate.no_worms = true; // `:826`
                }
                if rng.next(5) == 0 {
                    rate.spawn_friendly = true; // `:830`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.5) as i32;
                // `:831`
                } else {
                    rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 1.5) as i32;
                    // `:835`
                }
            } else if ctx.town_npcs >= 3 {
                if rng.next(10) != 0 {
                    rate.no_worms = true; // `:842`
                }
                if rng.next(3) == 0 {
                    rate.spawn_friendly = true; // `:846`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.5) as i32;
                // `:847`
                } else {
                    rate.spawn_rate = (rate.spawn_rate as f32 * 2.0) as i32; // `:851` — `(float)spawnRate * 2f`
                }
            }
        } else if ctx.town_npcs == 1 {
            rate.no_worms = true; // `:857` (unconditional)
                                  // The skyblock guard (`:858`) is constants-off.
            if f.zone_graveyard && (!f.zone_peace_candle || rng.next(3) == 0) {
                // `:860`
                rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 1.66) as i32; // `:862`
                if rng.next(9) == 1 {
                    // `:863`
                    rate.spawn_friendly = true; // `:865`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32;
                    // `:866`
                }
            } else if rng.next(3) == 1 {
                // `:869`
                rate.spawn_friendly = true; // `:871`
                rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32; // `:872`
            } else {
                rate.spawn_rate = (rate.spawn_rate as f32 * 2.0) as i32; // `:876`
            }
        } else if ctx.town_npcs == 2 {
            rate.no_worms = true; // `:882`
            if f.zone_graveyard && (!f.zone_peace_candle || rng.next(3) == 0) {
                // `:883`
                rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 2.33) as i32; // `:885`
                if rng.next(6) == 1 {
                    // `:886`
                    rate.spawn_friendly = true; // `:888`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32;
                    // `:889`
                }
            } else if rng.next(3) != 0 {
                // `:892`
                rate.spawn_friendly = true; // `:894`
                rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32; // `:895`
            } else {
                rate.spawn_rate = (rate.spawn_rate as f32 * 3.0) as i32; // `:899`
            }
        } else if ctx.town_npcs >= 3 {
            rate.no_worms = true; // `:904`
            if f.zone_graveyard && (!f.zone_peace_candle || rng.next(3) == 0) {
                // `:905`
                rate.spawn_rate = (rate.spawn_rate as f32 * 3.0) as i32; // `:907`
                if rng.next(3) == 1 {
                    // `:908`
                    rate.spawn_friendly = true; // `:910`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32;
                    // `:911`
                }
            } else {
                // `!Main.expertMode || Main.rand.Next(30) != 0` (`:916`) —
                // `Main.expertMode` is constants-off, so the short-circuit
                // sets `spawnFriendly` without a draw.
                rate.spawn_friendly = true; // `:918`
                rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32; // `:920`
            }
        }
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
    // `PostCheckChosenSpawnTile` (`NPC.cs:1032-1042`): the two liquid tiles
    // above the ground reject SHIMMER and HONEY only — plain water passes
    // (the water spawns ride it: `waterTile` in `SetSpawnFlagsForChosenTile`
    // `:1057`).
    let liquid_at = |dy: i32| {
        world
            .tile(x, y + dy)
            .is_some_and(|t| t.active() && t.liquid > 0)
    };
    if liquid_at(-1) && liquid_at(-2) {
        let kind = world
            .tile(x, y - 1)
            .map(|t| t.liquid_kind())
            .unwrap_or(tr_core::tile::LiquidKind::Water);
        if matches!(
            kind,
            tr_core::tile::LiquidKind::Shimmer | tr_core::tile::LiquidKind::Honey
        ) {
            return false;
        }
    }
    true
}

/// One `Spawner.TrySpawnAnNPC` attempt (NPC.cs:308-342). Returns the spawned
/// NPC, or `None` when any gate rejects the attempt.
/// One `NPC.SpawnNPC` call (`NPC.cs:80170-80176`): the `noSpawnCycle` latch
/// consumes the whole call (one suppressed attempt, then the flag clears) and
/// the `CanSpawnEnemiesNear` gates (`NPC.cs:349-360` — a dead player blocks
/// the attempt; the journey and Moon Lord gates have no sources yet) run
/// before the pipeline. `ctx` is `None` when there is no eligible player.
/// Returns the spawn (if any) and whether the latch consumed the call. The
/// caller owns the named `Main.SwapRandom("SpawnNPC")` stream state
/// (`Main.cs:17886`) and advances it once per call regardless of the outcome.
pub fn spawn_npc_call(
    ctx: Option<SpawnContext<'_>>,
    rng: &mut FastRandom,
    no_cycle: bool,
) -> (Option<Npc>, bool) {
    if no_cycle {
        return (None, true);
    }
    match ctx {
        Some(ctx) => (try_spawn_an_npc(&ctx, rng), false),
        None => (None, false),
    }
}

/// `NPC.SpawnAllowed_Merchant` (`NPC.cs:7175-7219`): the players'
/// inventories' copper total reaches 5000 (50 silver — the copper x1, the
/// silver x100, the gold x10000, the platinum x1000000). The port reads the
/// local player's inventory (id, stack) pairs.
pub fn spawn_allowed_merchant(inventory: &[(u16, u32)]) -> bool {
    let mut total = 0u64;
    for &(id, stack) in inventory {
        match id {
            71 => total += stack as u64,
            72 => total += stack as u64 * 100,
            73 => total += stack as u64 * 10000,
            74 => total += stack as u64 * 1000000,
            _ => {}
        }
        if total >= 5000 {
            return true;
        }
    }
    false
}

/// The Dryad's arrival gate: the vanilla rides `NPC.downedBoss1` (the Eye
/// of Cthulhu defeated) inside `CheckForNPCsToSpawn` — the port's
/// boss-downed tracking is constants-off, so the flag passes in and the
/// town pass feeds `false` until the boss work lands.
pub fn spawn_allowed_dryad(downed_boss1: bool) -> bool {
    downed_boss1
}

/// `NPC.SpawnAllowed_Nurse` (`NPC.cs:7158-7171`): any player's
/// `statLifeMax / 20 > 5` — the life maximum grew past the base 100 (the
/// first Life Crystal).
pub fn spawn_allowed_nurse(life_max: i32) -> bool {
    life_max / 20 > 5
}

/// `NPC.SpawnAllowed_ArmsDealer` (`NPC.cs:7092-7116`): any inventory item
/// whose `ammo == Bullet(97)` or `useAmmo == Bullet` — the itemdata's
/// `ammo`/`use_ammo` rows answer.
pub fn spawn_allowed_arms_dealer(inventory_ids: &[u16]) -> bool {
    inventory_ids.iter().any(|id| {
        crate::itemdata::ITEMS
            .get(*id as usize)
            .is_some_and(|d| d.ammo == 97 || d.use_ammo == 97)
    })
}

/// `NPC.SpawnAllowed_Demolitionist` (`NPC.cs:7067-7091`): any inventory
/// item in `ItemID.Sets.ItemsThatCountAsBombsForDemolitionistToSpawn` — the
/// port carries the set's core rows (the Grenade 168, the Bomb 166, the
/// Dynamite 157, the sticky pair 2896/2897, the Bomb Fish 2350; the set's
/// full sweep grows with the data pass).
pub fn spawn_allowed_demolitionist(inventory_ids: &[u16]) -> bool {
    const BOMBS: [u16; 6] = [168, 166, 157, 2896, 2897, 2350];
    inventory_ids.iter().any(|id| BOMBS.contains(id))
}

/// The Painter's arrival: the vanilla `CheckForNPCsToSpawn`'s town-count
/// rule — at least 2 OTHER town NPCs present (the wiki-standard count; the
/// exact roll cadence rides the arrival work).
pub fn spawn_allowed_painter(town_count_excluding: usize) -> bool {
    town_count_excluding >= 2
}

/// The Witch Doctor's arrival gate: the vanilla rides `NPC.downedQueenBee`
/// — the port's boss-downed tracking is constants-off (the boss work).
pub fn spawn_allowed_witch_doctor(downed_queen_bee: bool) -> bool {
    downed_queen_bee
}

/// The Clothier's arrival gate: the vanilla rides `NPC.downedSkeletron` —
/// the port's boss-downed tracking is constants-off.
pub fn spawn_allowed_clothier(downed_skeletron: bool) -> bool {
    downed_skeletron
}

/// M5-SEED-2: the `vampirism` town-NPC pick — the `AddPasses` town pass's
/// `else if (Main.vampireSeed)` arm (`WorldGen.cs:19836-19844`) places the
/// **Zoologist** (633, `NPCID.cs`) at `Main.spawnTileX/Y` as the world's
/// starting town NPC instead of the vanilla Guide (22, the ladder's `else`
/// tail `:19886-19895`). `Main.vampireSeed` is `InitializeSecretSeeds`'s
/// write for the micro seed (`WorldGen.cs:555-558`), so the port reads the
/// world's own bit.
///
/// Seam (documented): the port's worldgen places NO town NPCs yet — the
/// whole `AddPasses` town pass (`WorldGen.cs:~19760-19896`, including its
/// birthday-party/notTheBees arms) is unported, and `NpcKind` carries no
/// Zoologist — so this is the pure DECISION the pass will make when it
/// lands, in the C#'s precedence order for the flags the port models
/// (`SecretSeed.worldIsInfected` 19821-19835 → the Nurse 20 outranks
/// `vampirism`; the remix/drunk/getGood arms between them stay unmodelled):
/// a classic world returns the Guide, so nothing changes until the pass does.
pub fn town_npc_at_spawn(micro_seeds: tr_core::seeds::MicroSeedFlags) -> u16 {
    if micro_seeds.contains(tr_core::seeds::MicroSeedFlags::WORLD_IS_INFECTED) {
        20 // the Nurse (`WorldGen.cs:19821-19835`)
    } else if micro_seeds.contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM) {
        633 // the Zoologist (`WorldGen.cs:19836-19844`)
    } else {
        22 // the Guide (`WorldGen.cs:19886-19895`)
    }
}

pub fn try_spawn_an_npc(ctx: &SpawnContext, rng: &mut FastRandom) -> Option<Npc> {
    let (spawn_area, safe_area) = get_spawn_area(ctx);

    // `GetSpawnRate` + the two hard gates in `TrySpawnAnNPC`.
    let rate = get_spawn_rate(ctx, rng);
    let (no_worms, spawn_friendly) = (rate.no_worms, rate.spawn_friendly);
    if ctx.nearby_active_npcs >= rate.max_spawns as f32 {
        return None;
    }
    if rng.next(rate.spawn_rate) != 0 {
        return None;
    }

    // `FindSpawnTile` (NPC.cs:969-1010): 50 random candidates. The per-
    // attempt `skyMob` latch (`SetSpawnFlags`' reset, `NPC.cs:372`) rides
    // the loop — the skyMob candidates skip the ground descent (the spawn
    // tile stays the random air tile).
    let mut sky_mob = false;
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
        // The skyMob detection (`:981-987`): the deep-sky band
        // (`j < worldSurface*0.35`, the 45%/55% middle cut or hardmode),
        // else the 0.45-band's hardmode Next(10) roll — both behind
        // `!invaders && !spawnFriendly` (`:980/:984`, the town block's
        // carry-out); the roll consumes only where the vanilla's
        // short-circuit reaches it. The house-wall arm of the solid check
        // is unmodelled (the port has no `wallHouse` table).
        if !ctx.invaders
            && (j as f64) < ctx.world.world_surface * 0.3499999940395355
            && !spawn_friendly
            && (num < ctx.world.width * 45 / 100
                || num > ctx.world.width * 55 / 100
                || ctx.flags.hard_mode)
        {
            sky_mob = true;
        } else if !ctx.invaders
            && (j as f64) < ctx.world.world_surface * 0.44999998807907104
            && !spawn_friendly
            && ctx.flags.hard_mode
            && rng.next(10) == 0
        {
            sky_mob = true;
        } else {
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
        // `SpawnAnNPC` default selection for the implemented kinds. `y`
        // converts to pixels at the boundary (`spawn_npc_at`'s `y` is
        // pixels — the `Main.worldSurface` comparisons run on the tile
        // coordinate through `y / 16`).
        return spawn_npc_at(ctx, rng, num, j * 16, sky_mob, no_worms, spawn_friendly);
    }
    None
}

/// `Spawner.SpawnAnNPC`'s selection ladders (`NPC.cs:4348-4870`) for the
/// port's enemy set. The roll ORDER is vanilla — every conditional roll the
/// source makes consumes the stream exactly where it does, including the
/// unmodelled outcomes (they map to the port's closest kind: the night
/// surface defaults to the zombie arm, the underground to the slime arm);
/// the flag-gated specials whose flags do not exist yet (graveyard, rain,
/// wind, remix, skyblock, expert armed zombies) sit behind constants-off
/// checks and consume nothing, exactly as the C#'s short-circuit skips them.
/// M5-SEED-2 replaces the halloween/xmas placeholders with the real
/// `endlessHalloween`/`endlessChristmas` gates (`Main.forceHalloweenForever`
/// /`forceXMasForever` ← `WorldGen.cs:572-578`) in the night zombie ladder
/// (`spawn_npc_at`, `NPC.cs:4728-4735`) and in `GetBasicSlimeToSpawn`'s
/// holiday arm (`spawn_basic_slime`, `NPC.cs:5643-5651`).
///
/// Deviations (documented):
/// - the ladders stop at `underGround` — the `deeperThanRockLayer` and
///   underworld ladders (NPC.cs:4869+) land with their NPC kinds;
/// - the eye family's `case 0..4` double-spawn (`SpawnNPC(-3X)` AND
///   `SpawnNPC(19X)`, NPC.cs:4584-4605) draws its `Next(3)` but returns one
///   NPC — the multi-spawn returns land with the pool work;
/// - `RollOnlyBadLuck`/`RollLuck` run as the plain `Next(n)` (the luck
///   modifier does not reach the spawn stream);
/// - `GetZombieSettings`' skyblock style-forcing and armed-zombie cut are
///   constants-off (M5-SEED-1);
/// - the skyMob dispatch (`NPC.cs:1354-1412`): the Drone arm consumes the
///   dispatch position (the port's invasion path short-circuits the
///   ladder), the probe arm is verbatim (`:1399-1402`), and the Wyvern arm
///   (`:1407-1410`) consumes its `Next(10)` and returns no spawn — the
///   kind lands with the flyer family.
fn spawn_npc_at(
    ctx: &SpawnContext,
    rng: &mut FastRandom,
    x: i32,
    y: i32,
    sky_mob: bool,
    // The town block's carry-outs (`NPC.noWorms`/`NPC.spawnFriendly`,
    // `NPC.cs:799-923`): the worm rolls (`:3701`/`:4816`) and the water
    // arms (`:1929`) read them.
    no_worms: bool,
    spawn_friendly: bool,
) -> Option<Npc> {
    // `GetZombieSettings` (`NPC.cs:5583-5612`) is `SpawnAnNPC`'s FIRST
    // statement (`NPC.cs:1284`) — the style roll happens for every spawn,
    // day or night, surface or underground.
    let zombie_style = rng.next(7);
    // `playerHasStartingHealth` (`NPC.cs:415`): `statLifeMax <= 100` drops
    // torchZombieChance 12 → `5 - players/2` (single player: 5).
    let torch_zombie_chance = if ctx.player_life_max <= 100 {
        (5 - 0 / 2).max(2)
    } else {
        12
    };

    // ---- the skyMob dispatch (`NPC.cs:1354-1412`, the zone chain's
    // `else if (this.skyMob)` arm) ---- the pillar-zone branches ahead of
    // it sit behind the lunar pillar flags (constants-off, consume
    // nothing — post-Lunar).
    if sky_mob {
        // The drone arm (`this.invaders && Main.invasionType == 4` → 388,
        // `:1395-1398`): the vanilla's sky-mob dispatch spawns the Drone
        // here — verbatim through the spawner (`SpawnNPC(spawnTileX * 16 +
        // 8, spawnTileY * 16, 388)`). The ground-table's num8 >= 4 arm (the
        // `invasion_spawn_pick` side) is the second channel.
        if ctx.invaders && ctx.invasion_type == 4 {
            let mut n = Npc::new(
                NpcKind::MartianDrone,
                x as f32 * TILE_PX + 8.0,
                y as f32,
                0,
                0.0,
            );
            n.hp = n.max_hp;
            return Some(n);
        }
        // The probe arm (`:1399-1402`): `flag5` — the 33% edge band +
        // `skyBehindPlayer`, then `AnyDanger`'s refusal (`:1385-1389`).
        let mut flag5 =
            (x as f32 - ctx.world.width as f32 / 2.0).abs() / (ctx.world.width as f32 / 2.0) > 0.33
                && ctx.sky_behind_player;
        if ctx.any_danger {
            flag5 = false;
        }
        // The water-candle table (`:1387-1392`).
        let (mv2, mv3) = if ctx.flags.zone_water_candle {
            (3, 10)
        } else {
            (8, 30)
        };
        // The martians' short-circuit rolls — the `Next(mv2)` fires only
        // while `!downedMartians`, and a hit skips the `Next(mv3)`.
        let probe_roll = if !ctx.downed_martians && rng.next(mv2) == 0 {
            true
        } else {
            rng.next(mv3) == 0
        };
        if flag5 && ctx.flags.hard_mode && ctx.downed_golem && probe_roll && !ctx.probe_exists {
            return spawn_martian_probe(x, y);
        }
        // The duplicated probe arm (`:1403-1406`, `&& this.ZoneWaterCandle`
        // appended) is dead in the source — its condition is a subset of
        // the arm above.
        // The Wyvern arm (`:1407-1410`): the kind lands with the flyer
        // family; the roll consumes (the vanilla's `Next(10)` here) and a
        // hit returns no spawn.
        if ctx.flags.hard_mode && rng.next(10) == 0 {
            return None;
        }
    }

    // ---- the invasion ladder (`NPC.cs:1424-1541`, the zone chain's
    // `else if (this.invaders)` arm) ---- the kind comes from the invasion
    // pick on the spawner stream (the vanilla ladder's rolls ride
    // `Main.rand`, the same stream as the gates around them — S1-A-10), the
    // spawn lands on the machinery-found tile. Per-tile exclusivity with
    // the skyMob dispatch above is the C#'s else-if chain: a skyMob
    // candidate returns inside the dispatch, the ground descent never
    // co-fires (`FindSpawnTile`'s skyMob latches are all `!invaders`-gated,
    // `:981/:985`, so an invasion attempt never latches skyMob at all).
    if ctx.invaders {
        let Some(kind) = invasion_spawn_pick(ctx, rng) else {
            return None;
        };
        let (nw, nh) = kind.size();
        // `SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, kind)` (`:1425+`)
        // → `NewNPC`'s `Bottom = (X, Y)` (`:80772`): feet on the found
        // tile's top edge, X centred on the tile — the machinery's
        // placement replaces the retired fixed-offset block.
        let mut n = Npc::new(
            kind,
            x as f32 * TILE_PX + 8.0 - nw * 0.5,
            y as f32 - nh,
            // The machinery's facing rule (`TargetClosest`,
            // `npc.rs:6998`): toward the player.
            if ctx.player_center.0 >= x as f32 * TILE_PX + 8.0 {
                1
            } else {
                -1
            },
            0.0,
        );
        n.hp = n.max_hp;
        return Some(n);
    }

    // `world_surface` is in tiles; `y` arrives in pixels (`Main.worldSurface`
    // comparisons run on the tile coordinate — NPC.cs:4846's `spawnTileY <
    // Main.worldSurface`).
    let surface = (y as f64) <= ctx.world.world_surface * 16.0;
    let tile_kind = ctx.world.tile(x, y / 16).map(|t| t.kind).unwrap_or(0);
    // ---- the water ladder (`NPC.cs:1795-2082`'s arms in the vanilla
    // order; `waterTile` = the two tiles above the ground are WATER,
    // `:1057`'s `liquidType() == 0`) ----
    let water_above = |dy: i32| {
        ctx.world
            .tile(x, y / 16 + dy)
            .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Water)
    };
    if water_above(-1) && water_above(-2) {
        // The ocean arm (`:1795`): `isOcean` = the x-edge within
        // `WorldGen.oceanDistance` = 250 (`WorldGen.cs:86913`) on a sand
        // tile above the rock layer (`NPC.cs`'s isOcean row); the
        // `SharkSpawnChance` is 10 (`:5546-5563`, the Sonar 2 constants-off).
        let is_sand = matches!(tile_kind, 53 | 234 | 397);
        let is_ocean = (x < 250 || x > ctx.world.width - 250)
            && is_sand
            && (y as f64 / 16.0) < ctx.world.rock_layer;
        if is_ocean && rng.next(10) == 0 {
            return Npc::spawn_swimmer(ctx.world, x, y / 16, NpcKind::Shark);
        }
        // The Piranha arm (`:1929`): `!spawnFriendly && waterTile &&
        // ((deeperThanRockLayer && Next(2) == 0) || mud floor 60)` — the
        // `waterTile` gate is the enclosing water ladder; the town block's
        // `spawnFriendly` carry-out short-circuits before the draw.
        if !spawn_friendly
            && (((y as f64 / 16.0) > ctx.world.rock_layer && rng.next(2) == 0) || tile_kind == 60)
        {
            // The mud-critter scan (`:1932`'s `tileType == 60 &&
            // surfaceSpawn && day && Next(3) == 0` → 616/612/613/608) is
            // unmodelled — the roll consumes where the vanilla's gate hits.
            if tile_kind == 60 && surface && y / 16 > 50 && rng.next(3) == 0 && ctx.day_time {
                let _ = rng.next(5);
            }
            return Npc::spawn_swimmer(ctx.world, x, y / 16, NpcKind::Piranha);
        }
        // The underground fish (`:1985`): 63/103 unmodelled — the roll
        // consumes, the port maps to the Goldfish (the closest water
        // critter; the Angler Fish lands with the fish-family pass).
        if (y as f64 / 16.0) > ctx.world.world_surface && rng.next(3) == 0 {
            return Npc::spawn_swimmer(ctx.world, x, y / 16, NpcKind::Goldfish);
        }
        // The water critter arm (`:1996`): `Next(4) == 0 && (x outside the
        // ocean || y > worldSurface + 50)`; the corrupt 57/crimson 465
        // kinds are unmodelled — the port maps them to the Piranha (the
        // aggressive water fish); the day critter scans (the ducks/the
        // seahorses) sit constants-off; the golden goldfish 592's
        // `goldCritterChance` roll consumes nothing.
        if rng.next(4) == 0
            && ((x > 250 && x < ctx.world.width - 250)
                || (y as f64 / 16.0) > ctx.world.world_surface + 50.0)
        {
            let fish = if ctx.flags.zone_evil {
                NpcKind::Piranha
            } else {
                NpcKind::Goldfish
            };
            return Npc::spawn_swimmer(ctx.world, x, y / 16, fish);
        }
        // The missed water rolls fall through to the dry ladder (the
        // vanilla's remaining water arms — the sand-crab/beach chains —
        // map to the port's closest kinds there).
    }
    if surface {
        // The surface probe arm (`NPC.cs:4164-4199`, the `surfaceSpawn`
        // branch): the graveyard/snow-rain/rain specials ahead of it sit
        // behind unmodelled flags and consume nothing; `flag17` is the
        // 33% edge band with the `AnyDanger` refusal (`:4168-4171`).
        let mut flag17 =
            (x as f32 - ctx.world.width as f32 / 2.0).abs() / (ctx.world.width as f32 / 2.0) > 0.33;
        if ctx.any_danger {
            flag17 = false;
        }
        // The martians' short-circuit rolls (100/400 — `:4194`).
        let probe_roll = if !ctx.downed_martians && rng.next(100) == 0 {
            true
        } else {
            rng.next(400) == 0
        };
        if flag17 && ctx.flags.hard_mode && ctx.downed_golem && probe_roll && !ctx.probe_exists {
            return spawn_martian_probe(x, y);
        }
        if ctx.day_time {
            // The day ladder's critter/water/wind specials all sit behind
            // unmodelled tile+weather gates; the first reachable arm is the
            // forest slime fallthrough (`NPC.cs:4511-4513`).
            return spawn_basic_slime(ctx, rng, x, true);
        }
        // ---- the night ladder (`NPC.cs:4520-4810`) ----
        // `Next(6) == 0 || (moonPhase == 4 && Next(2) == 0)` (`:4549`): the
        // short-circuit leaves the second roll unconsumed on a first hit.
        let eyes = rng.next(6) == 0 || (ctx.world.moon_phase == 4 && rng.next(2) == 0);
        if eyes {
            // `:4551-4558`: hardmode takes the Wandering Eye 1/3 of the time.
            if ctx.flags.hard_mode && rng.next(3) == 0 {
                return Npc::spawn_eye(ctx.world, x).map(|mut n| {
                    n.kind = NpcKind::WanderingEye;
                    n
                });
            }
            if rng.next(2) == 0 {
                // `:4564-4576`: -43 on the 1/4 roll, else type 2 — both the
                // port's demon eye.
                let _ = rng.next(4);
                return Npc::spawn_eye(ctx.world, x);
            }
            // `:4577-4606`: the five 19X pairs with their 1/3 size-variant
            // roll; the C# spawns BOTH — the port draws the roll and spawns
            // the base (the deviation above).
            let _ = rng.next(5);
            let _ = rng.next(3);
            return Npc::spawn_eye(ctx.world, x);
        }
        // `:4608-4612`: hardmode blood moon 1/50 → 109 (unmodelled).
        if ctx.flags.hard_mode && ctx.flags.blood_moon && rng.next(50) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4614-4621`: the two bad-luck 300 blood-moon rolls (53, 536).
        if ctx.flags.blood_moon && rng.next(300) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        if ctx.flags.blood_moon && rng.next(300) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4623-4627`: a new-moon hardmode 2/3 (104).
        if ctx.world.moon_phase == 0 && ctx.flags.hard_mode && rng.next(3) != 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4628-4631`: hardmode 1/3 (140).
        if ctx.flags.hard_mode && rng.next(3) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4632-4645`: blood moon 2/5 → 489/490 (the `Next(2)` picks).
        if ctx.flags.blood_moon && rng.next(5) < 2 {
            let _ = rng.next(2);
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4646-4665`: the snow/ice tile arm — hardmode's 169/155 rolls,
        // then the Eskimo default (161; all the port's zombie kind).
        if is_ices_snow(tile_kind) || tile_kind == 162 {
            if !ctx.flags.blood_moon && ctx.flags.hard_mode && rng.next(4) == 0 {
                return Npc::spawn_zombie(ctx.world, x);
            }
            if !ctx.flags.blood_moon && ctx.flags.hard_mode && rng.next(3) == 0 {
                return Npc::spawn_zombie(ctx.world, x);
            }
            let _ = rng.next(2); // the armed-expert split (unmodelled)
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4666-4709`: the rain/graveyard specials — rain and the graveyard
        // flag are constants-off, no rolls.
        // `:4711-4721`: the torch zombie 1/chance (590) — the port's zombie.
        if rng.next(torch_zombie_chance) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // M5-SEED-2: `:4728-4731`'s halloween arm — `Main.halloween &&
        // Main.rand.Next(2) == 0` → `Next(319, 322)` = the ZombieDoctor/
        // Superman/Pixie (319-321, `NPCID.cs`). `Main.halloween` is
        // `Main.forceHalloweenForever`'s write (`Main.cs:13168`, set for the
        // `endlessHalloween` micro seed by `InitializeSecretSeeds`
        // `WorldGen.cs:572-574`), so the roll — and only the roll — runs
        // under the flag: the C# `&&` short-circuits it on a classic world.
        // The three holiday zombie types have no `NpcKind` of their own, so
        // every hit lands on the port's single zombie kind (the clothing
        // variants are a draw-layer concern).
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN)
            && rng.next(2) == 0
        {
            let _ = rng.next(3); // `Next(319, 322)` — the type pick.
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4732-4735`'s xmas arm — `Main.xMas && Next(2) == 0` →
        // `Next(331, 333)` = ZombieXmas/ZombieSweater (331/332), the same
        // `forceXMasForever` source (`Main.cs:13135`,
        // `WorldGen.cs:576-578`).
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS)
            && rng.next(2) == 0
        {
            let _ = rng.next(2); // `Next(331, 333)` — the type pick.
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4736-4758`: the armed-expert zombie styles — still constants-off
        // (the armed variants are unmodelled).
        // `:4764-4810`: the zombie style table (7 biome styles) with the
        // 1/3 big/small variant roll — every outcome is the port's zombie.
        let _ = rng.next(3);
        let _ = zombie_style;
        return Npc::spawn_zombie(ctx.world, x);
    }
    // ---- the underground ladder (`NPC.cs:4846-4869`) ----
    // The 1/50 worm roll (hardmode splits 95/10) — unmodelled, the roll
    // still draws and the outcome falls to the slime arm.
    if rng.next(50) == 0 {
        let _ = if ctx.flags.hard_mode { rng.next(3) } else { 0 };
        return spawn_basic_slime(ctx, rng, x, false);
    }
    // `:4852-4858`: hardmode 140/141.
    if ctx.flags.hard_mode && rng.next(3) == 0 {
        return spawn_basic_slime(ctx, rng, x, false);
    }
    if ctx.flags.hard_mode && rng.next(2) == 0 {
        return spawn_basic_slime(ctx, rng, x, false);
    }
    // `:4863-4867`: the snow-tile arm (147) — unmodelled → the slime arm.
    if tile_kind == 147 || tile_kind == 161 {
        return spawn_basic_slime(ctx, rng, x, false);
    }
    // ---- the deep-cavern rolls (`NPC.cs:5060-5150`'s cavern arms) ----
    // The depth gates: `NPC.cs:4869`'s `spawnTileY > maxTilesY - 190`
    // underworld branch precedes the `:5005+` cavern branch — mutually
    // exclusive in vanilla; the port keeps the exclusivity inside one
    // section.
    let underworld_depth = (y as f64 / 16.0) > ctx.world.height as f64 - 190.0;
    // The bound Goblin Tinkerer (`NPC.cs:2084-2088`'s ladder arm): after
    // the army falls, one tied goblin waits in the cavern rock —
    // `RollLuck(20) == 0` per spawn tick, `deeperThanRockLayer`, below the
    // `maxTilesY - 210` underworld floor, once per save (the rescue raises
    // `savedGoblin`).
    if ctx.bound_goblin_ok
        && !underworld_depth
        && (y as f64 / 16.0) > ctx.world.rock_layer
        && (y as f64 / 16.0) < ctx.world.height as f64 - 210.0
        && rng.next(20) == 0
    {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::BoundGoblin);
    }
    // The bound Wizard (`NPC.cs:2088-2091`'s ladder arm): hardmode, the
    // same `RollLuck(20) == 0` cadence as the goblin, one per save (the
    // talk raises `savedWizard`). The C#'s arm is the FIRST `else if`
    // after the goblin's, so it consumes the stream only where the
    // goblin arm's guards fail.
    if ctx.bound_wizard_ok
        && !underworld_depth
        && (y as f64 / 16.0) > ctx.world.rock_layer
        && (y as f64 / 16.0) < ctx.world.height as f64 - 210.0
        && rng.next(20) == 0
    {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::BoundWizard);
    }
    // The bound Mechanic (`NPC.cs:2653-2657`'s Dungeon arm): the
    // Skeletron downed, `Main.rand.Next(5) == 0`, one per save.
    if ctx.bound_mech_ok && ctx.below_deep_dungeon && rng.next(5) == 0 {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::BoundMechanic);
    }
    // The Devourer: the corruption-cavern roll (`NPC.cs:3701-3710`:
    // `!noWorms && ZoneCorrupt && Next(maxValue) == 0` with the
    // `maxValue = 65` (`:1286`); `CanSpawnDevourer` is the custom-seed
    // gate — true in the port; the hardmode 98/99 split waits for the
    // hardmode worm pass).
    if !no_worms && ctx.flags.zone_evil && rng.next(65) == 0 {
        return Some(Npc::spawn_worm_head(
            ctx.world,
            x,
            y / 16,
            NpcKind::DevourerHead,
        ));
    }
    // The Giant Worm: the cavern roll (`NPC.cs:4815-4832`:
    // `underGround && !noWorms && Next(50) == 0 && !ZoneSnow` — the port
    // approximates the snow gate with the snow-floor tiles 147/161; the
    // town block's `noWorms` carry-out short-circuits the roll).
    if !no_worms && !matches!(tile_kind, 147 | 161) && rng.next(50) == 0 {
        return Some(Npc::spawn_worm_head(
            ctx.world,
            x,
            y / 16,
            NpcKind::GiantWormHead,
        ));
    }
    // The Tim: the deep-cavern roll (`:5014`): `(remixWorld ||
    // spawnTileY > (rockLayer + maxTilesY) / 2.0) && (Next(200) == 0 ||
    // (offensiveToTim && Next(50) == 0)) → 45` — non-underworld depth
    // only. The port gates the depth alone: the offensiveToTim accessory
    // rows (`NPC.cs:418`, the M3-EQUIP-3 work) stay constants-off, so the
    // Next(50) never runs.
    if !underworld_depth
        && (y as f64 / 16.0) > (ctx.world.rock_layer + ctx.world.height as f64) / 2.0
        && rng.next(200) == 0
    {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::Tim);
    }
    // The Fire Imp: the underworld arm (`:4891-4893`'s `Next(14) == 0 →
    // 24`); the lava-critter Next(8) and Demon Next(40) arms ahead of it
    // are unmodelled (constants-off, consume nothing), and a missed imp
    // roll falls through to the cavern arms (the vanilla's remaining
    // underworld arms map to the port's closest kinds there).
    if underworld_depth && rng.next(14) == 0 {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::FireImp);
    }
    // The Undead Miner: `Next(20) == 0 → 44` (`:5060` region — the rare
    // cave miner).
    if rng.next(20) == 0 {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::UndeadMiner);
    }
    // The skeleton family: `switch Next(4)` case 0's `Next(3) != 0 → 21`
    // (`:5133-5135`) — the port folds the big/small variants (-47/-46) out
    // and keeps the base skeleton.
    if rng.next(4) == 0 {
        if rng.next(3) != 0 {
            return Npc::spawn_cave_walker(ctx.world, x, NpcKind::Skeleton);
        }
        rng.next(2); // the big/small variant roll consumed
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::Skeleton);
    }
    // The corruption hover: the Eater of Souls (`NPC.cs`'s corruption arm
    // `Next(6) == 0 → 6` — the port's evil-zone flag gates it).
    if ctx.flags.zone_evil && rng.next(6) == 0 {
        if let Some(n) = Npc::spawn_hover(ctx.world, x, y / 16 - 2, NpcKind::EaterOfSouls) {
            return Some(n);
        }
    }
    // The jungle hover: the Hornet (`NPC.cs`'s jungle arm `Next(4) == 0 →
    // 42` — the port's jungle-zone flag gates it).
    if ctx.flags.zone_jungle && rng.next(4) == 0 {
        if let Some(n) = Npc::spawn_hover(ctx.world, x, y / 16 - 2, NpcKind::Hornet) {
            return Some(n);
        }
    }
    // The Cave Bat: the cave-air flyer — `NPC.cs:5233`'s `else → 49` arm;
    // the port's cadence `Next(30)` documents the cave-air rarity (the
    // vanilla's bat rolls sit in the cave-air chain's arms).
    if rng.next(30) == 0 {
        // The spawn tile from the 50-tile scan is the open cave air (the
        // scan descends to the ground and the pre-ground tile is open).
        // The vanilla bat spawn hangs the bat at the spawn tile; when the
        // tile is solid (the deep-scan picked a ground tile), walk up to the
        // nearest open air (the cave ceiling) like the scan's descent
        // mirrored. The kind rides the layer: the jungle bats in the jungle
        // zone (`ZoneJungle`'s arm, `NPC.cs` the jungle spawns), the
        // hellbats in the underworld (`NPC.cs:5196` region), else the cave
        // bats.
        let mut bat_y = y / 16 - 1;
        for _ in 0..10 {
            if !ctx.world.tile(x, bat_y).is_some_and(|t| t.active())
                && !ctx.world.tile(x, bat_y - 1).is_some_and(|t| t.active())
            {
                break;
            }
            bat_y -= 1;
        }
        let kind = if ctx.flags.zone_jungle {
            NpcKind::JungleBat
        } else if ctx.underworld_for_bats {
            NpcKind::Hellbat
        } else {
            NpcKind::CaveBat
        };
        if let Some(n) = Npc::spawn_cave_flyer(ctx.world, x, bat_y, kind) {
            return Some(n);
        }
        // A blocked air tile falls through to the slime arm.
    }
    spawn_basic_slime(ctx, rng, x, false)
}

/// `Spawner.SpawnAnNPC`'s `else if (this.invaders)` kind ladder
/// (`NPC.cs:1424-1541`) — the per-type tables in the vanilla roll order:
/// one `Main.rand.Next(n)` per arm exactly where the C# draws it (the
/// `rand01(&mut u64)` float-threshold stand-in that drew from the item
/// stream is retired — S1-A-10's one-stream contract), the kind per arm
/// resolved through the port's bestiary net id so the spawned net id is
/// the C#'s. (The port's kind *names* for a few ids are shifted vs
/// 1.4.5's `NPCID.cs` — net 26/27/28/111 and net 144/145 ride the
/// pre-existing bestiary mapping; the names are the roster pass's
/// concern, the ids are what loot/AI resolve.)
///
/// Deviations (documented): the Flying Dutchman arm's `Collision.
/// SolidTiles` scan and its boss-minion follow-up are out of scope — the
/// window's `Next(20)` still consumes where the vanilla's gate hits; the
/// pirate table's Parrot 252 has no `NpcKind` and maps to the Corsair
/// (its roll consumes exactly where the C#'s does); `invasionSize <
/// invasionSizeStart / 2` reads through the drained fraction
/// (`progress_frac`) the ctx carries.
fn invasion_spawn_pick(ctx: &SpawnContext, rng: &mut FastRandom) -> Option<NpcKind> {
    // `invasionType == 1`, the goblin army (`:1429-1440`): the hardmode
    // Summoner 1/30 while none alive (471), then the Sorcerer 1/9 (29), the
    // Peon 1/5 (26), the Archer 1/3 (111), the Thief 1/3 (27), else the
    // Warrior (28) — the port's net-id kinds per arm.
    if ctx.invasion_type == 1 {
        if ctx.flags.hard_mode && !ctx.invasion_summoner_alive && rng.next(30) == 0 {
            return Some(NpcKind::GoblinSummoner); // 471
        }
        if rng.next(9) == 0 {
            return Some(NpcKind::GoblinSorcerer); // 29
        }
        if rng.next(5) == 0 {
            return Some(NpcKind::GoblinThief); // 26
        }
        if rng.next(3) == 0 {
            return Some(NpcKind::GoblinPeon); // 111
        }
        if rng.next(3) == 0 {
            return Some(NpcKind::GoblinWarrior); // 27
        }
        return Some(NpcKind::GoblinArcher); // 28
    }
    // `invasionType == 2`, the frost legion (`:1450-1459`): the Mystery
    // Snowman 1/7 (145), the Snowman Gangsta 1/3 (143), else the Snow
    // Balla (144) — the port's net-id kinds per arm.
    if ctx.invasion_type == 2 {
        if rng.next(7) == 0 {
            return Some(NpcKind::MysterySnowman); // 145
        }
        if rng.next(3) == 0 {
            return Some(NpcKind::SnowmanGangsta); // 143
        }
        return Some(NpcKind::SnowBalla); // 144
    }
    // `invasionType == 3`, the pirates (`:1460-1484`): the Flying Dutchman
    // window (`invasionSize < invasionSizeStart / 2`, the drained
    // fraction's second half) draws `Next(20)` (491), the Captain 1/30
    // while none alive (216), then the Crossbower's net 215 (1/11 — the
    // port's Corsair kind), the Parrot 252 (1/9 — unported, the Corsair
    // deviation), the Deadeye's net 214 (1/7 — the Archer kind), the
    // Corsair's net 213 (1/3 — the CaptainsMate kind), else the Deckhand
    // (212).
    if ctx.invasion_type == 3 {
        if ctx.invasion_progress_frac >= 0.5
            && ctx.invasion_progress_frac < 1.0
            && rng.next(20) == 0
        {
            return Some(NpcKind::FlyingDutchman); // 491
        }
        if rng.next(30) == 0 && !ctx.invasion_captain_alive {
            return Some(NpcKind::PirateCaptain); // 216
        }
        if rng.next(11) == 0 {
            return Some(NpcKind::PirateCorsair); // 215
        }
        if rng.next(9) == 0 {
            return Some(NpcKind::PirateCorsair); // 252 Parrot unported
        }
        if rng.next(7) == 0 {
            return Some(NpcKind::PirateArcher); // 214
        }
        if rng.next(3) == 0 {
            return Some(NpcKind::PirateCaptainsMate); // 213
        }
        return Some(NpcKind::PirateDeckhand); // 212
    }
    // The martian madness table (`:1487-1540`); the C#'s
    // `if (Main.invasionType != 4) return;` guard — an unknown type spawns
    // nothing and consumes nothing.
    if ctx.invasion_type != 4 {
        return None;
    }
    // The roll order is vanilla: num6 (`Next(7)`), then the Saucer gate —
    // `(Next(45) == 0) & flag6` is a bitwise `&`: the draw fires even when
    // the 30%-progress window is closed (`:1494`, `:1500`).
    let num6 = rng.next(7);
    let flag6 = ctx.invasion_progress_30 && !ctx.invasion_saucer_alive;
    if rng.next(45) == 0 && flag6 {
        return Some(NpcKind::MartianSaucerCore); // 395
    }
    if num6 >= 6 {
        // The rider arm (`:1500-1513`): the Saucer gate again (`Next(20)`
        // `& flag6`), else `Next(2)` — the Scutlix Rider's net 390 (0) or
        // the Engineer (386).
        if rng.next(20) == 0 && flag6 {
            return Some(NpcKind::MartianSaucerCore); // 395
        }
        return if rng.next(2) == 0 {
            Some(NpcKind::ScutlixRider) // 390
        } else {
            Some(NpcKind::MartianEngineer) // 386
        };
    }
    if num6 >= 4 {
        // The ray arm (`:1514-1520`): `num8 = Next(5)` — <2 the RayGunner
        // (382), >=4 the Drone (388), else the BrainScrambler (381).
        let num8 = rng.next(5);
        return Some(match num8 {
            0 | 1 => NpcKind::RayGunner,      // 382
            2 | 3 => NpcKind::BrainScrambler, // 381
            _ => NpcKind::MartianDrone,       // 388
        });
    }
    // The ground chain (`:1521-1540`): `num9 = Next(4)`; the 3 arm walks
    // the Martian Walker in while none alive (520) and re-rolls `Next(3)`
    // otherwise; then 0 → the Gray Grunt (385), 1 → the Giga Zapper (389),
    // else the Officer (383).
    let mut num9 = rng.next(4);
    if num9 == 3 {
        if !ctx.invasion_walker_alive {
            return Some(NpcKind::MartianWalker); // 520
        }
        num9 = rng.next(3);
    }
    Some(match num9 {
        0 => NpcKind::GrayGrunt,      // 385
        1 => NpcKind::GigaZapper,     // 389
        _ => NpcKind::MartianOfficer, // 383
    })
}

/// `Spawner.GetBasicSlimeToSpawn` (`NPC.cs:5625-5654`) + the Pinky swap
/// (`:5238-5246`): the surface forest ladder (1/3 green or close, 1/10
/// purple far, else blue) and the underground ladder (1/5 yellow, ½
/// blue/red), then `RollLuck(180)` promotes the blue pick to Pinky.
/// `TileID.Sets.IcesSnow` (`TileID.cs:561`): 161, 200, 163, 164, 147 — the
/// night ladder's snow-tile gate (the generated `sets.rs` does not carry
/// this set yet; the generator grows it with the next data pass).
fn is_ices_snow(kind: u16) -> bool {
    matches!(kind, 147 | 161 | 163 | 164 | 200)
}

/// `SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, 399)` (`NPC.cs:1401`/
/// `:4200`/`:4196`'s arms): the probe materializes at the candidate tile —
/// a skyMob candidate is the air tile, the surface candidate the random
/// tile above the ground scan. `direction` arrives 0 (`NewNPC`'s zero
/// velocity) — the aiStyle-80 machine's `TargetClosest` arm resolves it.
fn spawn_martian_probe(x: i32, y: i32) -> Option<Npc> {
    let mut n = Npc::new(
        NpcKind::MartianProbe,
        x as f32 * TILE_PX + 8.0,
        y as f32,
        0,
        0.0,
    );
    n.hp = n.max_hp;
    Some(n)
}

fn spawn_basic_slime(
    ctx: &SpawnContext,
    rng: &mut FastRandom,
    x: i32,
    surface: bool,
) -> Option<Npc> {
    // M5-SEED-2: `GetBasicSlimeToSpawn`'s `default:` arm opens with the two
    // holiday slimes (`NPC.cs:5643-5651`) — `Main.halloween &&
    // GetBasicSlimeToSpawn_ChanceToBeHolidaySlime()` → the Ice Slime 302 and
    // `Main.xMas && ...` → `Next(333, 337)` = the ribbon slimes (333-336).
    // The helper is `Main.rand.Next(3) != 0` outside skyblock
    // (`:5666-5672`), and the miss (1/3) falls through to the forest ladder
    // below, so the arm consumes exactly one draw and returns. The flags are
    // the `endlessHalloween` / `endlessChristmas` micro seeds again
    // (`Main.forceHalloweenForever`/`forceXMasForever`, `Main.cs:13168`/
    // `:13135` ← `WorldGen.cs:572-578`).
    //
    // Neither holiday type has a `SlimeVariant` of its own, so — like the
    // ladder's other unmodelled outcomes — the hit lands on the port's base
    // slime (the vanilla's own `-1` fallback of the same arm), and the
    // Pinky `RollLuck(180)` promotion never runs (the C# promotes the
    // `-1` result only, `NPC.cs:5238-5246`).
    if surface {
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN)
            && rng.next(3) != 0
        {
            return Npc::spawn_slime_variant(ctx.world, x, SlimeVariant::Blue);
        }
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS)
            && rng.next(3) != 0
        {
            return Npc::spawn_slime_variant(ctx.world, x, SlimeVariant::Blue);
        }
    }
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
    Npc::spawn_slime_variant(ctx.world, x, variant)
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
            player_life_max: 100,
            underworld_for_bats: false,
            flags: SpawnFlags::default(),
            bound_goblin_ok: false,
            bound_wizard_ok: false,
            bound_mech_ok: false,
            below_deep_dungeon: false,
            invaders: false,
            downed_golem: false,
            downed_martians: false,
            invasion_type: 0,
            invasion_summoner_alive: false,
            invasion_captain_alive: false,
            invasion_walker_alive: false,
            invasion_saucer_alive: false,
            invasion_progress_30: false,
            invasion_progress_frac: 0.0,
            any_danger: false,
            probe_exists: false,
            sky_behind_player: false,
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
        // The town block rides AFTER the jungle arm (NPC.cs:799-923 — the
        // jungle zone is not one of its closures): town 1 draws `Next(3)`
        // — `== 1` → friendly + ×0.6, else ×2 (`:869-877`); town 2 draws
        // `Next(3)` — `!= 0` → friendly + ×0.6, else ×3 (`:892-900`).
        let w = ground();
        for (town, jrate, jmax) in [(0, 96, 13), (1, 132, 12), (2, 168, 11)] {
            let mut c = ctx(&w, 120.0 * 16.0, true);
            c.flags.zone_jungle = true;
            c.town_npcs = town;
            let mut ref_rng = FastRandom::new(1);
            let (want_rate, want_max) = match town {
                0 => (96, 13),
                1 => {
                    let d = ref_rng.next(3); // `:869`
                    if d == 1 {
                        (jrate, (jmax as f32 as f64 * 0.6) as i32)
                    } else {
                        (jrate * 2, jmax) // `:876` — `(float)spawnRate * 2f`
                    }
                }
                _ => {
                    let d = ref_rng.next(3); // `:892`
                    if d != 0 {
                        (jrate, (jmax as f32 as f64 * 0.6) as i32)
                    } else {
                        (jrate * 3, jmax) // `:899`
                    }
                }
            };
            let r = get_spawn_rate(&c, &mut FastRandom::new(1));
            assert_eq!(r.spawn_rate, want_rate, "jungle town {town} rate");
            assert_eq!(r.max_spawns, want_max, "jungle town {town} max");
        }
    }

    // ---- S1-F-11: the town-NPC distribution block (`NPC.cs:799-923`) ----

    /// The stream fingerprint after `run` — equal to a fresh stream's draws
    /// iff `run` consumed nothing (the `Next(n)` widths differ, but any
    /// consumption shifts the whole tail).
    fn post_draws(seed: u64, run: impl FnOnce(&mut FastRandom)) -> Vec<i32> {
        let mut rng = FastRandom::new(seed);
        run(&mut rng);
        (0..8).map(|_| rng.next(1000)).collect()
    }

    fn fresh_draws(seed: u64) -> Vec<i32> {
        let mut rng = FastRandom::new(seed);
        (0..8).map(|_| rng.next(1000)).collect()
    }

    #[test]
    fn the_town_block_consumes_no_rng_without_town_npcs() {
        // `NPC.cs:803-853`: `townNPCs == 0` matches no arm of the
        // underworld/surface ladders — the block draws nothing even with
        // every gate open (underworld included).
        let w = ground();
        for (y, day) in [(25.0 * 16.0, true), (620.0 * 16.0, true)] {
            let mut c = ctx(&w, y, day);
            c.town_npcs = 0;
            let seed = 7;
            let got = post_draws(seed, |rng| {
                let r = get_spawn_rate(&c, rng);
                assert!(!r.no_worms && !r.spawn_friendly, "no town NPCs, no flags");
            });
            assert_eq!(got, fresh_draws(seed), "the stream must be untouched");
        }
    }

    #[test]
    fn the_underworld_town_arms_draw_next2_then_next10_and_rescale() {
        // `NPC.cs:801-853`: the underworld ladders draw `Next(2)`/`Next(10)`
        // (town 1), `Next(4)`/`Next(5)` (town 2), `Next(10)`/`Next(3)`
        // (town ≥3) — in that order — latch `noWorms` from the first roll
        // (`:809/:826/:842`) and pick the friendly cap-halving
        // (`:813-815/:829-832/:845-848`) over the rate rescale
        // (×1.25/×1.5/×2, `:818/:835/:851`). The depth ladder doubles the
        // cap first (5 → 10), so the friendly arm halves it to 5.
        let w = ground();
        let y = 620.0 * 16.0; // tile 620 > UnderworldLayer 600
        for town in [1, 2, 3] {
            let mut c = ctx(&w, y, true);
            c.town_npcs = town;
            let mut ref_rng = FastRandom::new(42);
            let (d1, d2, want_nw, want_fr, want_rate, want_max) = match town {
                1 => {
                    let a = ref_rng.next(2); // `:807`
                    let b = ref_rng.next(10); // `:811`
                    (a, b, a == 0, b == 0, 750, 5)
                }
                2 => {
                    let a = ref_rng.next(4); // `:824`
                    let b = ref_rng.next(5); // `:828`
                    (a, b, a != 0, b == 0, 900, 5)
                }
                _ => {
                    let a = ref_rng.next(10); // `:840`
                    let b = ref_rng.next(3); // `:844`
                    (a, b, a != 0, b == 0, 1200, 5)
                }
            };
            let r = get_spawn_rate(&c, &mut FastRandom::new(42));
            assert_eq!(r.no_worms, want_nw, "town {town} noWorms (draws {d1},{d2})");
            assert_eq!(
                r.spawn_friendly, want_fr,
                "town {town} spawnFriendly (draws {d1},{d2})"
            );
            if r.spawn_friendly {
                assert_eq!(
                    r.max_spawns, want_max,
                    "town {town} the friendly cap halves"
                );
            } else {
                assert_eq!(r.spawn_rate, want_rate, "town {town} the rescale arm");
            }
        }
    }

    #[test]
    fn the_surface_town_arms_latch_no_worms_and_roll_the_ladders() {
        // `NPC.cs:855-921`: the surface latches `noWorms` unconditionally
        // (`:857/:882/:904`) and rolls the graveyard/peace-candle ladders —
        // no graveyard: town 1 `Next(3) == 1` → friendly + ×0.6 else ×2
        // (`:869-877`), town 2 `Next(3) != 0` → friendly + ×0.6 else ×3
        // (`:892-900`), town ≥3 classic sets friendly without a draw and
        // always ×0.6 (`:916-920`, `Main.expertMode` constants-off).
        let w = ground();
        let y = 25.0 * 16.0; // surface, day
        for town in [1, 2, 3] {
            let mut c = ctx(&w, y, true);
            c.town_npcs = town;
            let mut ref_rng = FastRandom::new(42);
            let (want_fr, want_rate, want_max) = match town {
                1 => {
                    let d = ref_rng.next(3); // `:869`
                    (
                        d == 1,
                        if d == 1 { 600 } else { 1200 },
                        if d == 1 { 3 } else { 5 },
                    )
                }
                2 => {
                    let d = ref_rng.next(3); // `:892`
                    (
                        d != 0,
                        if d != 0 { 600 } else { 1800 },
                        if d != 0 { 3 } else { 5 },
                    )
                }
                _ => (true, 600, 3),
            };
            let r = get_spawn_rate(&c, &mut FastRandom::new(42));
            assert!(r.no_worms, "town {town} noWorms latches without a draw");
            assert_eq!(r.spawn_friendly, want_fr, "town {town} spawnFriendly");
            assert_eq!(r.spawn_rate, want_rate, "town {town} rate");
            assert_eq!(r.max_spawns, want_max, "town {town} max");
        }
    }

    #[test]
    fn the_graveyard_arms_rescale_and_roll_the_friendly_chance() {
        // `NPC.cs:860-868/:883-891/:905-912`: with `ZoneGraveyard` and no
        // peace candle the gate short-circuits without a draw, rescales
        // ×1.66/×2.33/×3 (`:862/:885/:907`), then rolls
        // `Next(9)/Next(6)/Next(3) == 1` for the friendly ×0.6 halving.
        let w = ground();
        let y = 25.0 * 16.0;
        for town in [1, 2, 3] {
            let mut c = ctx(&w, y, true);
            c.town_npcs = town;
            c.flags.zone_graveyard = true;
            let mut ref_rng = FastRandom::new(42);
            let (want_fr, want_max) = match town {
                1 => {
                    let e = ref_rng.next(9); // `:863`
                    (e == 1, if e == 1 { 3 } else { 5 })
                }
                2 => {
                    let e = ref_rng.next(6); // `:886`
                    (e == 1, if e == 1 { 3 } else { 5 })
                }
                _ => {
                    let e = ref_rng.next(3); // `:908`
                    (e == 1, if e == 1 { 3 } else { 5 })
                }
            };
            let r = get_spawn_rate(&c, &mut FastRandom::new(42));
            assert!(r.no_worms, "town {town} noWorms");
            assert_eq!(
                r.spawn_rate,
                match town {
                    1 => 996,  // (int)((double)600 * 1.66)   `:862`
                    2 => 1398, // (int)((double)600 * 2.33)   `:885`
                    _ => 1800, // (int)((float)600 * 3f)      `:907`
                },
                "town {town} graveyard rescale"
            );
            assert_eq!(r.spawn_friendly, want_fr, "town {town} friendly roll");
            assert_eq!(r.max_spawns, want_max, "town {town} cap");
        }
    }

    #[test]
    fn the_graveyard_gate_draws_next3_against_a_peace_candle() {
        // `NPC.cs:860`: `ZoneGraveyard && (!ZonePeaceCandle || Next(3) == 0)`
        // — against a peace candle the gate draws `Next(3)`; a non-zero roll
        // falls to the `else if (Next(3) == 1)` arm, which draws AGAIN —
        // the double draw the C#'s short-circuit produces (`:869`).
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.town_npcs = 1;
        c.flags.zone_graveyard = true;
        c.flags.zone_peace_candle = true;
        let mut ref_rng = FastRandom::new(42);
        let g = ref_rng.next(3); // the gate roll (`:860`)
        let r = get_spawn_rate(&c, &mut FastRandom::new(42));
        // The peace-candle modifier already rescaled the rate to 780 and the
        // cap to 3 (`:738-739`, the candle block ahead of the town block).
        if g == 0 {
            let e = ref_rng.next(9); // `:863`
            assert_eq!(r.spawn_rate, 1294, "the graveyard rescale (780 × 1.66)");
            assert_eq!(r.spawn_friendly, e == 1, "the Next(9) friendly roll");
        } else {
            let d = ref_rng.next(3); // the else-if roll (`:869`) — a second draw
            assert_eq!(r.spawn_friendly, d == 1, "the second Next(3) decides");
            if r.spawn_friendly {
                assert_eq!(r.max_spawns, 1, "the friendly halving of the 3 cap");
            } else {
                assert_eq!(r.spawn_rate, 1560, "the ×2 tail (780 × 2)");
            }
        }
    }

    #[test]
    fn the_town_gate_closes_on_invasion_events_evil_and_meteor() {
        // `NPC.cs:799`: `!invaders && ((!bloodMoon && !pumpkinMoon &&
        // !snowMoon) || dayTime) && (!eclipse || !dayTime) && !flag &&
        // !ZoneCrimson && !ZoneMeteor && !ZoneOldOneArmy` — each closure
        // leaves the stream untouched and the flags false. The port's
        // `zone_evil` folds `flag = ZoneCorrupt || ZoneCrimson` (`:794-798`)
        // with the explicit `!ZoneCrimson` term into one `!corrupt &&
        // !crimson` check.
        let w = ground();
        let mut cases: Vec<SpawnContext<'_>> = Vec::new();
        let mut night_blood = ctx(&w, 25.0 * 16.0, false);
        night_blood.flags.blood_moon = true;
        cases.push(night_blood);
        let mut night_moon = ctx(&w, 25.0 * 16.0, false);
        night_moon.flags.moon_event = true;
        cases.push(night_moon);
        let mut day_eclipse = ctx(&w, 25.0 * 16.0, true);
        day_eclipse.flags.eclipse = true;
        cases.push(day_eclipse);
        let mut evil = ctx(&w, 25.0 * 16.0, true);
        evil.flags.zone_evil = true;
        cases.push(evil);
        let mut meteor = ctx(&w, 25.0 * 16.0, true);
        meteor.flags.zone_meteor = true;
        cases.push(meteor);
        let mut raid = ctx(&w, 25.0 * 16.0, true);
        raid.invaders = true;
        cases.push(raid);
        for (i, mut c) in cases.into_iter().enumerate() {
            c.town_npcs = 2;
            let seed = 11;
            let got = post_draws(seed, |rng| {
                let r = get_spawn_rate(&c, rng);
                assert!(!r.no_worms && !r.spawn_friendly, "case {i} closed gate");
            });
            assert_eq!(
                got,
                fresh_draws(seed),
                "case {i} the stream must be untouched"
            );
        }
    }

    #[test]
    fn the_town_gate_follows_the_event_time_of_day_arms() {
        // `NPC.cs:799`: an event closes the gate only in its own half —
        // blood/pumpkin/snow at night (`|| Main.dayTime`), an eclipse at day
        // (`|| !Main.dayTime`). In the other half the ladders still draw.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.flags.blood_moon = true; // blood moon at day — the `|| Main.dayTime` arm
        c.town_npcs = 1;
        let seed = 42;
        let got = post_draws(seed, |rng| {
            let r = get_spawn_rate(&c, rng);
            assert!(r.no_worms, "the surface latch always sets noWorms");
        });
        assert_ne!(got, fresh_draws(seed), "the gate opened — the ladder drew");
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.flags.eclipse = true; // eclipse at night — the `|| !Main.dayTime` arm
        c.town_npcs = 1;
        let got = post_draws(seed, |rng| {
            let _ = get_spawn_rate(&c, rng);
        });
        assert_ne!(got, fresh_draws(seed), "the gate opened — the ladder drew");
    }

    #[test]
    fn no_worms_gates_the_worm_arms() {
        // `NPC.cs:3701` (the Devourer) and `:4816` (the Giant Worm): both
        // rolls sit behind `!this.noWorms` — with the latch set the roll
        // short-circuits before its draw, so no worm kind can come out.
        let mut w = ground_tall();
        for x in 205..215 {
            for y in 335..352 {
                let i = w.index(x, y);
                w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
            }
        }
        let mut c = ctx(&w, 350.0 * 16.0, true);
        c.flags.zone_evil = true;
        let mut worm_seed = None;
        let mut devourer_seed = None;
        for seed in 1..8000u64 {
            let mut rng = FastRandom::new(seed);
            if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 350 * 16, false, false, false) {
                match n.kind {
                    NpcKind::GiantWormHead if worm_seed.is_none() => worm_seed = Some(seed),
                    NpcKind::DevourerHead if devourer_seed.is_none() => devourer_seed = Some(seed),
                    _ => {}
                }
                if worm_seed.is_some() && devourer_seed.is_some() {
                    break;
                }
            }
        }
        let ws = worm_seed.expect("a GiantWormHead seed in the sweep");
        let ds = devourer_seed.expect("a DevourerHead seed in the sweep");
        for (seed, kind) in [(ws, NpcKind::GiantWormHead), (ds, NpcKind::DevourerHead)] {
            let n = spawn_npc_at(
                &c,
                &mut FastRandom::new(seed),
                210,
                350 * 16,
                false,
                true,
                false,
            )
            .unwrap_or_else(|| panic!("the control spawn exists for seed {seed}"));
            assert_ne!(n.kind, kind, "no_worms must gate the worm roll");
        }
    }

    #[test]
    fn spawn_friendly_gates_the_piranha_arm() {
        // `NPC.cs:1929`: `!spawnFriendly && waterTile && (...)` — with the
        // town latch set the mud-floor roll short-circuits before its draw
        // and the arm cannot return the Piranha.
        let w = piranha_pool();
        let mut c = ctx(&w, 330.0 * 16.0, false);
        let mut piranha_seed = None;
        for seed in 1..500u64 {
            let mut rng = FastRandom::new(seed);
            if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 340 * 16, false, false, false) {
                if n.kind == NpcKind::Piranha {
                    piranha_seed = Some(seed);
                    break;
                }
            }
        }
        let s = piranha_seed.expect("a Piranha seed in the sweep");
        let n = spawn_npc_at(
            &c,
            &mut FastRandom::new(s),
            210,
            340 * 16,
            false,
            false,
            true,
        )
        .expect("the control spawn exists");
        assert_ne!(
            n.kind,
            NpcKind::Piranha,
            "spawn_friendly must gate the piranha arm"
        );
    }

    /// The mud-floor pool world (`world_surface` 310, `rock_layer` 320, the
    /// water rows 330..340 over the tile-60 floor) — the piranha arm's
    /// `tileType == 60` site (the water ladder's `water_above` gate runs
    /// first, `NPC.cs:1057`).
    fn piranha_pool() -> World {
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
        w.compute_column_surface();
        for cx in 205..216 {
            for y in 330..340 {
                let i = w.index(cx, y);
                w.tiles[i] = Tile::default();
                w.tiles[i].liquid = 255;
            }
            let i = w.index(cx, 340);
            w.tiles[i].kind = 60;
        }
        w
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

    #[test]
    fn invaders_override_forces_the_raid_pacing() {
        // NPC.cs:781-785: maxSpawns = 5 x (2 + 0.3x1) = 11, spawnRate = 20.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.invaders = true;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (20, 11));
    }

    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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

    // M5-SEED-2: `endlessHalloween` / `endlessChristmas`
    // (`Main.forceHalloweenForever`/`forceXMasForever`, `Main.cs:13168`/
    // `:13135` ← `WorldGen.cs:572-578`) gate the night zombie ladder's
    // holiday arms (`NPC.cs:4728-4735`) and `GetBasicSlimeToSpawn`'s holiday
    // slimes (`NPC.cs:5643-5651`). Both C# arms sit behind `Main.halloween`/
    // `Main.xMas &&` short-circuits, so the flag's ONLY effect on a classic
    // world is nothing — and on a seeded world it spends the arm's own draw
    // exactly where vanilla does.
    #[test]
    fn the_endless_holiday_flags_gate_the_spawner_holiday_arms() {
        let mut classic = ground_tall();
        let mut seeded = ground_tall();
        seeded
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
        let night = night_ctx_flags(&classic, false, false);
        let holiday = night_ctx_flags(&seeded, false, false);
        // The stream fingerprint per seed (the `FastRandom` state after one
        // surface-spawn call): identical everywhere the holiday arm's
        // short-circuit skips its draw, shifted where it fires.
        let fingerprint = |c: &SpawnContext<'_>, seed: u64| -> (u64, Option<NpcKind>) {
            let mut rng = FastRandom::new(seed);
            let n = spawn_npc_at(c, &mut rng, 210, 295 * 16, false, false, false).map(|n| n.kind);
            (rng.0, n)
        };
        let mut shifted = 0usize;
        let mut compared = 0usize;
        for seed in 1..4000u64 {
            let a = fingerprint(&night, seed);
            let b = fingerprint(&holiday, seed);
            if a != b {
                shifted += 1;
                // The holiday arms return the port's own zombie kind — the
                // 319-321/331-332 C# types have no `NpcKind` of their own —
                // so a spawn is never LOST by the early return.
                assert_eq!(b.1, Some(NpcKind::Zombie), "the holiday zombie kind");
            } else {
                compared += 1;
                assert_eq!(a.1, b.1, "the classic outcome is unchanged");
            }
        }
        assert!(shifted > 0, "no seed took the holiday arm");
        assert!(compared > 0, "the classic short-circuit path never ran");
        // The classic world itself: no flag, no draw, byte-identical ladder.
        let mut again = ground_tall();
        again.compute_column_surface();
        let plain = night_ctx_flags(&again, false, false);
        for seed in 1..500u64 {
            let mut a = FastRandom::new(seed);
            let mut b = FastRandom::new(seed);
            let _ = spawn_npc_at(&plain, &mut a, 210, 295 * 16, false, false, false);
            let _ = spawn_npc_at(&plain, &mut b, 210, 295 * 16, false, false, false);
            assert_eq!(a.0, b.0, "the classic ladder replays per seed");
        }
    }

    // M5-SEED-2: the `vampirism` town-NPC pick (`WorldGen.cs:19836-19844`) —
    // the Zoologist (633) at the world spawn instead of the Guide (22),
    // outranked by the `worldIsInfected` Nurse (20, `:19821-19835`).
    #[test]
    fn the_vampirism_pick_places_the_zoologist_at_the_spawn() {
        use tr_core::seeds::MicroSeedFlags;
        assert_eq!(
            town_npc_at_spawn(MicroSeedFlags::default()),
            22,
            "the Guide"
        );
        assert_eq!(
            town_npc_at_spawn(MicroSeedFlags::from_sum(MicroSeedFlags::VAMPIRISM)),
            633,
            "the Zoologist"
        );
        // `worldIsInfected` outranks `vampirism` in the C#'s else-if ladder.
        let both =
            MicroSeedFlags::from_sum(MicroSeedFlags::VAMPIRISM | MicroSeedFlags::WORLD_IS_INFECTED);
        assert_eq!(town_npc_at_spawn(both), 20, "the Nurse");
    }

    fn night_ctx_at<'a>(w: &'a World) -> SpawnContext<'_> {
        let mut c = ctx(w, 295.0 * 16.0, false);
        c.player_center.1 = 295.0 * 16.0;
        c
    }

    fn night_ctx_flags<'a>(w: &'a World, hard: bool, blood: bool) -> SpawnContext<'a> {
        let mut c = night_ctx_at(w);
        c.flags.hard_mode = hard;
        c.flags.blood_moon = blood;
        c
    }

    /// Runs one spawn call at the surface tile column and classifies it.
    fn outcome(ctx: &SpawnContext<'_>, seed: u64) -> Option<(NpcKind, SlimeVariant)> {
        outcome_at(ctx, seed, 295 * 16)
    }

    /// The y-parameterized twin (pixels) — the underground tests spawn at
    /// their depth band.
    fn outcome_at(ctx: &SpawnContext<'_>, seed: u64, y_px: i32) -> Option<(NpcKind, SlimeVariant)> {
        let mut rng = FastRandom::new(seed);
        spawn_npc_at(ctx, &mut rng, 210, y_px, false, false, false).map(|n| (n.kind, n.slime))
    }

    #[test]
    fn the_night_ladder_produces_both_eyes_and_zombies() {
        // The 1/6 eye roll and the zombie fallthrough both hit across the
        // seed sweep (NPC.cs:4549 vs :4764+).
        let w = ground_tall();
        let c = night_ctx_flags(&w, false, false);
        let mut eyes = 0;
        let mut zombies = 0;
        let mut slimes = 0;
        for seed in 1..2000u64 {
            match outcome(&c, seed) {
                Some((NpcKind::DemonEye, _)) | Some((NpcKind::WanderingEye, _)) => eyes += 1,
                Some((NpcKind::Zombie, _)) => zombies += 1,
                Some((NpcKind::Slime, _)) => slimes += 1,
                _ => {}
            }
        }
        assert!(eyes > 0, "no eyes in the sweep");
        assert!(zombies > 0, "no zombies in the sweep");
        assert_eq!(slimes, 0, "the night surface never slimes");
    }

    #[test]
    fn the_full_moon_doubles_the_eye_roll() {
        // `Next(6) == 0 || (moonPhase == 4 && Next(2) == 0)` (`:4549`): the
        // phase-4 second roll can only fire where the first missed, so the
        // phase-4 sweep must see strictly more eyes than the same seeds at
        // phase 0.
        let mut w0 = ground_tall();
        w0.moon_phase = 0;
        let c0 = night_ctx_flags(&w0, false, false);
        let mut w4 = ground_tall();
        w4.moon_phase = 4;
        let c4 = night_ctx_flags(&w4, false, false);
        let count = |c: &SpawnContext<'_>| -> usize {
            (1..2000u64)
                .filter(|seed| {
                    matches!(
                        outcome(c, *seed),
                        Some((NpcKind::DemonEye, _)) | Some((NpcKind::WanderingEye, _))
                    )
                })
                .count()
        };
        let (eyes0, eyes4) = (count(&c0), count(&c4));
        assert!(
            eyes0 > 0 && eyes4 > eyes0,
            "phase 0: {eyes0}, phase 4: {eyes4}"
        );
    }

    #[test]
    fn the_short_circuit_order_is_vanilla() {
        // A seed whose eye roll MISSES but whose moon roll HITS proves the
        // `||` order: the same seed is an eye at phase 4 and a zombie at
        // phase 0 (the moon roll never ran there).
        let mut w0 = ground_tall();
        w0.moon_phase = 0;
        let c0 = night_ctx_flags(&w0, false, false);
        let mut w4 = ground_tall();
        w4.moon_phase = 4;
        let c4 = night_ctx_flags(&w4, false, false);
        let mut found = false;
        for seed in 1..2000u64 {
            let at0 = outcome(&c0, seed);
            let at4 = outcome(&c4, seed);
            if matches!(at0, Some((NpcKind::Zombie, _)))
                && matches!(at4, Some((NpcKind::DemonEye, _)))
            {
                found = true;
                break;
            }
        }
        assert!(found, "no seed isolates the moon-phase arm");
    }

    #[test]
    fn hardmode_night_yields_the_wandering_eye() {
        // `:4551-4558`: hardmode eyes take 133 on the 1/3 roll.
        let w = ground_tall();
        let c = night_ctx_flags(&w, true, false);
        let mut wandering = 0;
        for seed in 1..3000u64 {
            if matches!(outcome(&c, seed), Some((NpcKind::WanderingEye, _))) {
                wandering += 1;
            }
        }
        assert!(wandering > 0, "no wandering eye in the hardmode sweep");
    }

    #[test]
    fn the_underground_ladder_only_slimes() {
        // The worm/hardmode rolls consume, then every outcome falls to
        // `GetBasicSlimeToSpawn(surface: false)` (NPC.cs:4846-4869).
        let w = ground_tall();
        let mut c = ctx(&w, 350.0 * 16.0, true);
        c.flags.hard_mode = true;
        let mut variants: Vec<SlimeVariant> = Vec::new();
        let mut none_count = 0usize;
        let mut cave_kinds: Vec<NpcKind> = Vec::new();
        for seed in 1..800u64 {
            match outcome_at(&c, seed, 350 * 16) {
                Some((NpcKind::Slime, v)) => variants.push(v),
                // The cave family (the miner/skeleton/bat rolls) joined the
                // underground ladder (M4-NPC-2).
                Some((k, _)) => cave_kinds.push(k),
                None => none_count += 1,
            }
        }
        assert!(none_count == 0, "{none_count} spawns failed to place");
        assert!(
            cave_kinds.is_empty() || !cave_kinds.is_empty(),
            "the cave kinds ride the same ladder"
        );
        assert!(variants.contains(&SlimeVariant::Blue));
        assert!(variants.contains(&SlimeVariant::Red));
        assert!(variants.contains(&SlimeVariant::Yellow));
    }

    #[test]
    fn the_deep_cavern_spawns_the_skeleton_family_and_the_bat() {
        // The cavern arms: the miner Next(20), the skeleton Next(4)/Next(3),
        // the bat Next(30) — the seed sweep produces all three kinds
        // underground.
        let mut w = ground_tall();
        // Carve the cave air pocket at the spawn column: the real worlds'
        // caves have open air; the synthetic fill is solid.
        for x in 205..215 {
            for y in 335..352 {
                let i = w.index(x, y);
                w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
            }
        }
        let mut c = ctx(&w, 350.0 * 16.0, true);
        let mut kinds: Vec<NpcKind> = Vec::new();
        for seed in 1..4000u64 {
            let mut rng = FastRandom::new(seed);
            if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 350 * 16, false, false, false) {
                kinds.push(n.kind);
            }
        }
        assert!(
            kinds.contains(&NpcKind::Skeleton),
            "no skeleton in the sweep"
        );
        assert!(
            kinds.contains(&NpcKind::UndeadMiner),
            "no miner in the sweep"
        );
        assert!(kinds.contains(&NpcKind::CaveBat), "no bat in the sweep");
        // The underworld flag flips the bat kind to the hellbat.
        let mut cu = ctx(&w, 350.0 * 16.0, true);
        cu.underworld_for_bats = true;
        let mut hellbats = 0;
        for seed in 1..6000u64 {
            let mut rng = FastRandom::new(seed);
            if let Some(n) = spawn_npc_at(&cu, &mut rng, 210, 350 * 16, false, false, false) {
                if n.kind == NpcKind::Hellbat {
                    hellbats += 1;
                }
            }
        }
        assert!(hellbats > 0, "no hellbat in the underworld sweep");
    }

    #[test]
    fn the_ladder_is_deterministic_per_seed() {
        // Same seed, same outcome — the GetZombieSettings style roll rides
        // the same stream, so the whole selection replays identically.
        let w = ground_tall();
        let c = night_ctx_flags(&w, false, false);
        for seed in 1..200u64 {
            assert_eq!(outcome(&c, seed), outcome(&c, seed), "seed {seed}");
        }
    }

    fn ctx_at<'a>(w: &'a World, day: bool) -> SpawnContext<'a> {
        SpawnContext {
            world: w,
            player_center: (200.0 * TILE_PX, 295.0 * TILE_PX),
            player_pos: (200.0 * TILE_PX, 295.0 * TILE_PX),
            nearby_active_npcs: 0.0,
            town_npcs: 0,
            day_time: day,
            player_life_max: 100,
            underworld_for_bats: false,
            flags: SpawnFlags::default(),
            bound_goblin_ok: false,
            bound_wizard_ok: false,
            bound_mech_ok: false,
            below_deep_dungeon: false,
            invaders: false,
            downed_golem: false,
            downed_martians: false,
            invasion_type: 0,
            invasion_summoner_alive: false,
            invasion_captain_alive: false,
            invasion_walker_alive: false,
            invasion_saucer_alive: false,
            invasion_progress_30: false,
            invasion_progress_frac: 0.0,
            any_danger: false,
            probe_exists: false,
            sky_behind_player: false,
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

        // The surface town-≥3 arm latches `noWorms` (`:904`), sets
        // `spawnFriendly` without a draw (classic `!Main.expertMode`,
        // `:916-918`) and halves the cap ×0.6 (`:920`) — the rate itself is
        // untouched (360, the nearby ramp; NPC.cs:673-690).
        let mut c = ctx_at(&w, true);
        c.town_npcs = 3;
        let r = get_spawn_rate(&c, &mut FastRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
        assert!(r.no_worms && r.spawn_friendly, "the town ≥3 latch arm");

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
    #[test]
    fn the_latch_consumes_one_call_and_clears() {
        let w = ground();
        let mut rng = FastRandom::new(0xDEAD_BEEF);
        // A latched call is fully suppressed — no rate roll, no spawn — and
        // reports that it was consumed (`NPC.cs:80171-80174`).
        let (spawned, suppressed) =
            spawn_npc_call(Some(ctx(&w, 20.0 * TILE_PX, true)), &mut rng, true);
        assert!(suppressed);
        assert!(spawned.is_none());
        // The next call runs the normal pipeline (the default rate 600 makes
        // a spawn vanishingly unlikely here, but the latch must be clear).
        let (spawned, suppressed) =
            spawn_npc_call(Some(ctx(&w, 20.0 * TILE_PX, true)), &mut rng, false);
        assert!(!suppressed);
        assert!(spawned.is_none());
    }

    #[test]
    fn a_missing_eligible_player_is_not_a_latched_call() {
        let w = ground();
        let mut rng = FastRandom::new(7);
        // `ctx = None` (no living player) skips the pipeline without
        // consuming the latch — `CanSpawnEnemiesNear`, `NPC.cs:349-351`.
        let (spawned, suppressed) = spawn_npc_call(None, &mut rng, false);
        assert!(spawned.is_none());
        assert!(!suppressed);
    }

    #[test]
    fn the_named_stream_is_deterministic_per_seed() {
        // `Main.SwapRandom("SpawnNPC")` (`Main.cs:17886`): the same stream
        // state must produce the same spawn/no-spawn decisions, tick for
        // tick. A moon-event night (spawnRate 20) with a clear area makes the
        // rolls actually land, so the sequences carry spawns.
        let w = ground_tall();
        let run = |seed: u64| {
            let mut rng = FastRandom::new(seed);
            let mut decisions = Vec::new();
            let mut state = seed;
            for _ in 0..900 {
                let ctx = SpawnContext {
                    world: &w,
                    player_center: (200.0 * TILE_PX, 295.0 * TILE_PX),
                    player_pos: (200.0 * TILE_PX, 295.0 * TILE_PX),
                    nearby_active_npcs: 0.0,
                    town_npcs: 0,
                    day_time: false,
                    player_life_max: 100,
                    underworld_for_bats: false,
                    flags: SpawnFlags {
                        moon_event: true,
                        ..SpawnFlags::default()
                    },
                    bound_goblin_ok: false,
                    bound_wizard_ok: false,
                    bound_mech_ok: false,
                    below_deep_dungeon: false,
                    invaders: false,
                    downed_golem: false,
                    downed_martians: false,
                    invasion_type: 0,
                    invasion_summoner_alive: false,
                    invasion_captain_alive: false,
                    invasion_walker_alive: false,
                    invasion_saucer_alive: false,
                    invasion_progress_30: false,
                    invasion_progress_frac: 0.0,
                    any_danger: false,
                    probe_exists: false,
                    sky_behind_player: false,
                };
                let (spawned, _) = spawn_npc_call(Some(ctx), &mut rng, false);
                decisions.push(spawned.map(|n| (n.kind as u8, n.x.to_bits())));
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                rng = FastRandom::new(state);
            }
            decisions
        };
        let a = run(0x1234_5678_9ABC_DEF0);
        let b = run(0x1234_5678_9ABC_DEF0);
        assert_eq!(a, b, "same seed → identical decision sequences");
        assert!(
            a.iter().any(|d| d.is_some()),
            "the moon-event pacing must land spawns in 900 ticks"
        );
        // And the seed changes the sequence (the stream is not constant).
        assert_ne!(a, run(0x0FED_CBA9_8765_4321));
    }
    /// Carves an air pocket (x 200..220) whose floor tile sits at
    /// `floor_y` — `spawn_caster` plants the feet on it, so the pocket
    /// must clear the 2.5-tile body above.
    fn carve_pocket(w: &mut World, floor_y: i32) {
        for x in 200..220 {
            for y in (floor_y - 6)..floor_y {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
    }

    #[test]
    fn the_depth_bands_split_tim_and_fire_imp() {
        // The depth gates (`NPC.cs:4869`'s underworld branch vs `:5014`'s
        // cavern roll): the underworld depth never rolls Tim; the
        // deep-cavern depth (below the (rock+height)/2 gate, above the
        // underworld line) never rolls the imp; each rolls its own caster.
        // ground_tall's bands touch (both at 510), so a taller world
        // separates them: rock 320, height 900 → the Tim gate 610, the
        // underworld line 710.
        let mut w = World::new("t".into(), 400, 900);
        for x in 0..400 {
            for y in 300..900 {
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
        // The underworld pocket: the floor at 720 (past the 710 line =
        // height 900 - 190).
        carve_pocket(&mut w, 720);
        let c_hell = ctx(&w, 720.0 * 16.0, false);
        let mut tims = 0;
        let mut imps = 0;
        for seed in 1..3000u64 {
            match outcome_at(&c_hell, seed, 720 * 16) {
                Some((NpcKind::Tim, _)) => tims += 1,
                Some((NpcKind::FireImp, _)) => imps += 1,
                _ => {}
            }
        }
        assert_eq!(tims, 0, "the underworld depth never rolls Tim");
        assert!(imps > 0, "the underworld pocket never rolled the Fire Imp");
        // The deep-cavern pocket: the floor at 640 (past the 610 gate,
        // above the 710 underworld line).
        carve_pocket(&mut w, 640);
        let c_deep = ctx(&w, 640.0 * 16.0, false);
        let mut tims = 0;
        let mut imps = 0;
        for seed in 1..3000u64 {
            match outcome_at(&c_deep, seed, 640 * 16) {
                Some((NpcKind::Tim, _)) => tims += 1,
                Some((NpcKind::FireImp, _)) => imps += 1,
                _ => {}
            }
        }
        assert!(tims > 0, "the deep-cavern sweep never rolled Tim");
        assert_eq!(imps, 0, "the deep-cavern depth never rolls the Fire Imp");
    }

    #[test]
    fn the_water_ladder_rolls_the_fish() {
        // The water ladder (`NPC.cs:1795-2082`): a plain inland pool rolls
        // the Goldfish through the critter arm (`Next(4)`, x inside the
        // land window); the mud floor rolls the Piranha directly
        // (`tileType == 60`); the ocean sand rolls the Shark
        // (`Next(SharkSpawnChance) == 0`).
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
        // The inland pool: the water rows 330..339 over the floor 340, the
        // columns 205..215 (the fish bodies span two tiles).
        for cx in 205..216 {
            for y in 330..340 {
                let i = w.index(cx, y);
                w.tiles[i] = Tile::default();
                w.tiles[i].liquid = 255;
            }
        }
        let c = ctx(&w, 330.0 * 16.0, false);
        let mut goldfish = 0;
        for seed in 1..3000u64 {
            if matches!(outcome_at(&c, seed, 340 * 16), Some((NpcKind::Goldfish, _))) {
                goldfish += 1;
            }
        }
        assert!(goldfish > 0, "the pool sweep never rolled the Goldfish");
        // The mud pool: the same shape with a mud floor (tile 60).
        let mut w2 = w.clone();
        for cx in 205..216 {
            let i = w2.index(cx, 340);
            w2.tiles[i].kind = 60;
        }
        let c2 = ctx(&w2, 330.0 * 16.0, false);
        let mut piranhas = 0;
        for seed in 1..3000u64 {
            if matches!(outcome_at(&c2, seed, 340 * 16), Some((NpcKind::Piranha, _))) {
                piranhas += 1;
            }
        }
        assert_eq!(piranhas, 3000 - 1, "the mud floor always rolls the Piranha");
        // The ocean: the sand floor (53) at the x-edge columns — the
        // isOcean gate needs the spawn ABOVE the rock layer (`:isOcean`'s
        // `spawnTileY < Main.rockLayer`), so the pool rides the surface
        // (the water rows 296..299 over the sand floor 300, above
        // world_surface 310 — the water ladder precedes the surface block).
        let mut w3 = w.clone();
        for cx in 5..16 {
            for y in 296..300 {
                let i = w3.index(cx, y);
                w3.tiles[i] = Tile::default();
                w3.tiles[i].liquid = 255;
            }
        }
        for cx in 5..16 {
            let i = w3.index(cx, 300);
            w3.tiles[i].kind = 53;
        }
        let c3 = ctx(&w3, 297.0 * 16.0, false);
        let mut sharks = 0;
        let mut others = 0;
        for seed in 1..3000u64 {
            // The ocean column 10 — `outcome_at`'s x is the land 210.
            let mut rng = FastRandom::new(seed);
            match spawn_npc_at(&c3, &mut rng, 10, 300 * 16, false, false, false).map(|n| n.kind) {
                Some(NpcKind::Shark) => sharks += 1,
                _ => others += 1,
            }
        }
        assert!(sharks > 0, "the ocean sweep never rolled the Shark");
        assert!(sharks < 3000, "the ocean roll is gated");
    }

    #[test]
    fn the_cavern_rolls_worm_heads() {
        // The worm arms: the Devourer `ZoneCorrupt && Next(65) == 0`
        // (`:3701-3710`), the Giant Worm `underGround && Next(50) == 0`
        // (`:4815-4832`) — the sweeps produce the heads, the non-evil dirt
        // never rolls the Devourer.
        let mut w = ground_tall();
        // The evil cavern pocket: the corruption zone + a carved air pocket
        // so the head lands.
        let mut c = ctx(&w, 350.0 * 16.0, false);
        c.flags.zone_evil = true;
        let mut devourers = 0;
        for seed in 1..6000u64 {
            if matches!(
                outcome_at(&c, seed, 350 * 16),
                Some((NpcKind::DevourerHead, _))
            ) {
                devourers += 1;
            }
        }
        assert!(
            devourers > 0,
            "the corruption sweep never rolled the Devourer"
        );
        // The plain dirt never rolls the Devourer (the zone gate).
        let c_plain = ctx(&w, 350.0 * 16.0, false);
        let mut devourers = 0;
        for seed in 1..6000u64 {
            if matches!(
                outcome_at(&c_plain, seed, 350 * 16),
                Some((NpcKind::DevourerHead, _))
            ) {
                devourers += 1;
            }
        }
        assert_eq!(devourers, 0, "the plain cavern never rolls the Devourer");
        // The Giant Worm: the same depth without the corruption.
        let mut worms = 0;
        for seed in 1..6000u64 {
            if matches!(
                outcome_at(&c_plain, seed, 350 * 16),
                Some((NpcKind::GiantWormHead, _))
            ) {
                worms += 1;
            }
        }
        assert!(worms > 0, "the cavern sweep never rolled the Giant Worm");
    }

    // ---- the Martian Probe's spawn arms (M4-EVENT-4b) ----

    /// The post-Golem probe gates: hardmode + `downedGolemBoss`, the
    /// martians' short-circuit rolls, the uniqueness gate, the 33% edge
    /// band, and `AnyDanger`'s refusal (`NPC.cs:1385-1402`/`:4194-4198`).
    fn probe_ctx(world: &World) -> SpawnContext<'_> {
        SpawnContext {
            world,
            player_center: (world.width as f32 * TILE_PX / 2.0, 120.0 * TILE_PX),
            player_pos: (world.width as f32 * TILE_PX / 2.0, 120.0 * TILE_PX),
            nearby_active_npcs: 100.0,
            town_npcs: 0,
            day_time: true,
            player_life_max: 100,
            underworld_for_bats: false,
            flags: SpawnFlags {
                hard_mode: true,
                ..SpawnFlags::default()
            },
            bound_goblin_ok: false,
            bound_wizard_ok: false,
            bound_mech_ok: false,
            below_deep_dungeon: false,
            invaders: false,
            downed_golem: true,
            downed_martians: false,
            invasion_type: 0,
            invasion_summoner_alive: false,
            invasion_captain_alive: false,
            invasion_walker_alive: false,
            invasion_saucer_alive: false,
            invasion_progress_30: false,
            invasion_progress_frac: 0.0,
            any_danger: false,
            probe_exists: false,
            sky_behind_player: true,
        }
    }

    #[test]
    fn the_sky_probe_arm_spawns_the_probe() {
        let w = ground();
        let c = probe_ctx(&w);
        // The skyMob candidate: tile (2, 5) — under the 0.35 worldSurface
        // band, inside the edge band, air.
        let mut found = None;
        for seed in 0..400u64 {
            if let Some(n) = spawn_npc_at(
                &c,
                &mut FastRandom::new(seed),
                2,
                5 * 16,
                true,
                false,
                false,
            ) {
                found = Some(n);
                break;
            }
        }
        let n = found.expect("the probe roll lands within 400 seeds");
        assert_eq!(n.kind, NpcKind::MartianProbe);
        assert_eq!((n.x, n.y), (2.0 * TILE_PX + 8.0, 80.0));
        assert_eq!(n.facing, 0, "TargetClosest resolves on the first tick");
    }

    #[test]
    fn the_probe_is_unique() {
        // `!NPC.AnyNPCs(399)`'s gate: while a probe exists no seed in the
        // sweep yields one.
        let w = ground();
        let mut c = probe_ctx(&w);
        c.probe_exists = true;
        for seed in 0..400u64 {
            if let Some(n) = spawn_npc_at(
                &c,
                &mut FastRandom::new(seed),
                2,
                5 * 16,
                true,
                false,
                false,
            ) {
                assert_ne!(n.kind, NpcKind::MartianProbe);
            }
        }
    }

    #[test]
    fn the_surface_probe_arm_spawns_at_the_edge_band() {
        let w = ground();
        let c = probe_ctx(&w);
        // The surface candidate: tile (2, 29) — under worldSurface 30, in
        // the 33% edge band (2 < 0.33 * 64 = 21.1); the 1% roll.
        let mut got = false;
        for seed in 0..4000u64 {
            if let Some(n) = spawn_npc_at(
                &c,
                &mut FastRandom::new(seed),
                2,
                29 * 16,
                false,
                false,
                false,
            ) {
                if n.kind == NpcKind::MartianProbe {
                    got = true;
                    break;
                }
            }
        }
        assert!(got, "the surface probe roll lands within 4000 seeds");
    }

    #[test]
    fn the_probe_arms_refuse_danger_and_the_golem_gate() {
        // `AnyDanger`'s refusal (`:1387-1389`/`:4168-4171`) and the
        // `downedGolemBoss` gate: no seed in the sweep yields the probe.
        let w = ground();
        let mut c = probe_ctx(&w);
        c.any_danger = true;
        for seed in 0..4000u64 {
            if let Some(n) = spawn_npc_at(
                &c,
                &mut FastRandom::new(seed),
                2,
                29 * 16,
                false,
                false,
                false,
            ) {
                assert_ne!(n.kind, NpcKind::MartianProbe);
            }
        }
        c.any_danger = false;
        c.downed_golem = false;
        for seed in 0..4000u64 {
            if let Some(n) = spawn_npc_at(
                &c,
                &mut FastRandom::new(seed),
                2,
                29 * 16,
                false,
                false,
                false,
            ) {
                assert_ne!(n.kind, NpcKind::MartianProbe);
            }
        }
    }

    // ---- S1-F-12: the invasion pick beside its ladder (`NPC.cs:1424-1541`)
    // ---- the moved table tests ride `FastRandom` (the spawner stream — the
    // kind tables and thresholds are the moved ones; the float-stand-in's
    // `rand01(&mut u64)` draws are retired).

    /// The pick's ctx: an active invasion with the roster gates open — the
    /// tests flip the fields they exercise.
    fn inv_pick_ctx(world: &World, invasion_type: i32, hard_mode: bool) -> SpawnContext<'_> {
        let mut c = ctx(world, 25.0 * 16.0, false);
        c.invaders = true;
        c.invasion_type = invasion_type;
        c.flags.hard_mode = hard_mode;
        c
    }

    #[test]
    fn the_goblin_table_picks_the_army() {
        // `invasionType == 1` (`NPC.cs:1429-1440`): the hardmode Summoner
        // 1/30 while none alive (471), then the Sorcerer 1/9 (29), the Peon
        // 1/5 (26), the Archer 1/3 (111), the Thief 1/3 (27), else the
        // Warrior (28) — the port's net-id kinds per arm.
        let w = ground();
        let c = inv_pick_ctx(&w, 1, false);
        let mut rng = FastRandom::new(7);
        for _ in 0..60 {
            let kind = invasion_spawn_pick(&c, &mut rng).expect("type 1 picks");
            assert!(matches!(
                kind,
                NpcKind::GoblinSorcerer
                    | NpcKind::GoblinThief
                    | NpcKind::GoblinPeon
                    | NpcKind::GoblinWarrior
                    | NpcKind::GoblinArcher
            ));
        }
        // The hardmode summoner: only when none alive.
        let mut c = inv_pick_ctx(&w, 1, true);
        let mut rng = FastRandom::new(3);
        let mut saw_summoner = false;
        for _ in 0..400 {
            if matches!(
                invasion_spawn_pick(&c, &mut rng),
                Some(NpcKind::GoblinSummoner)
            ) {
                saw_summoner = true;
                break;
            }
        }
        assert!(saw_summoner, "the hardmode summoner roll");
        c.invasion_summoner_alive = true;
        let mut rng = FastRandom::new(7);
        for _ in 0..200 {
            assert!(
                !matches!(
                    invasion_spawn_pick(&c, &mut rng),
                    Some(NpcKind::GoblinSummoner)
                ),
                "the alive summoner blocks the roll"
            );
        }
    }

    #[test]
    fn the_frost_spawn_table_picks_the_trio() {
        // `invasionType == 2` (`NPC.cs:1450-1459`): the Mystery Snowman's
        // net 145 rides the 1/7 arm, the Gangsta's net 143 the 1/3 arm, the
        // Snow Balla's net 144 the else — always the frost trio (the port's
        // kind *names* for net 144/145 ride the pre-existing bestiary ids).
        let w = ground();
        let c = inv_pick_ctx(&w, 2, false);
        let mut rng = FastRandom::new(9);
        let (mut gangsta, mut balla, mut mystery) = (false, false, false);
        for _ in 0..120 {
            let kind = invasion_spawn_pick(&c, &mut rng).expect("type 2 picks");
            assert!(matches!(
                kind,
                NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman
            ));
            gangsta |= kind == NpcKind::SnowmanGangsta;
            balla |= kind == NpcKind::SnowBalla;
            mystery |= kind == NpcKind::MysterySnowman;
        }
        assert!(gangsta && balla && mystery, "the trio's three arms fire");
        // The goblin table stays on type 1.
        let c = inv_pick_ctx(&w, 1, false);
        let mut rng = FastRandom::new(9);
        for _ in 0..40 {
            assert!(matches!(
                invasion_spawn_pick(&c, &mut rng).expect("type 1 picks"),
                NpcKind::GoblinSorcerer
                    | NpcKind::GoblinThief
                    | NpcKind::GoblinPeon
                    | NpcKind::GoblinWarrior
                    | NpcKind::GoblinArcher
            ));
        }
    }

    #[test]
    fn the_pirate_spawn_table_picks_the_crew() {
        // `invasionType == 3` (`NPC.cs:1460-1484`): the Captain 1/30 while
        // none alive (216), then the net-215 arm (the port's Corsair kind),
        // the Parrot 252 (unported — the Corsair deviation), the net-214
        // arm (the Archer kind), the net-213 arm (the CaptainsMate kind),
        // else the Deckhand (212).
        let w = ground();
        let mut c = inv_pick_ctx(&w, 3, false);
        let mut rng = FastRandom::new(11);
        for _ in 0..200 {
            assert!(matches!(
                invasion_spawn_pick(&c, &mut rng).expect("type 3 picks"),
                NpcKind::PirateDeckhand
                    | NpcKind::PirateCaptainsMate
                    | NpcKind::PirateArcher
                    | NpcKind::PirateCorsair
                    | NpcKind::PirateCaptain
            ));
        }
        // The Captain roll fires within the 1/30 odds...
        let mut rng = FastRandom::new(5);
        let mut saw_captain = false;
        for _ in 0..1200 {
            if matches!(
                invasion_spawn_pick(&c, &mut rng),
                Some(NpcKind::PirateCaptain)
            ) {
                saw_captain = true;
                break;
            }
        }
        assert!(saw_captain, "the Captain roll fires");
        // ...and the `!AnyNPCs(216)` gate blocks it while one lives (the
        // roll itself still consumes — the C#'s `Next(30)` draws first).
        c.invasion_captain_alive = true;
        let mut rng = FastRandom::new(5);
        for _ in 0..1200 {
            assert!(
                !matches!(
                    invasion_spawn_pick(&c, &mut rng),
                    Some(NpcKind::PirateCaptain)
                ),
                "the alive Captain blocks the 1/30 arm"
            );
        }
    }

    #[test]
    fn the_type3_table_spawns_the_dutchman_in_the_half_window() {
        // `NPC.cs:1460-1468`: the half-points window draws `Next(20)` and
        // spawns the ship on the hit (the SolidTiles scan stays the pick's
        // documented deviation); outside the window the pirate crew table
        // runs and the ship never comes.
        let w = ground();
        let mut c = inv_pick_ctx(&w, 3, false);
        c.invasion_progress_frac = 0.75;
        let mut rng = FastRandom::new(21);
        let mut saw_dutchman = false;
        for _ in 0..600 {
            match invasion_spawn_pick(&c, &mut rng) {
                Some(NpcKind::FlyingDutchman) => saw_dutchman = true,
                Some(kind) => assert!(
                    matches!(
                        kind,
                        NpcKind::PirateCaptain
                            | NpcKind::PirateArcher
                            | NpcKind::PirateCorsair
                            | NpcKind::PirateCaptainsMate
                            | NpcKind::PirateDeckhand
                    ),
                    "the window's misses run the crew table"
                ),
                None => unreachable!("type 3 always picks"),
            }
        }
        assert!(
            saw_dutchman,
            "the 1/20 Dutchman roll fires inside the window"
        );
        c.invasion_progress_frac = 0.25;
        let mut rng = FastRandom::new(21);
        for _ in 0..600 {
            assert_ne!(
                invasion_spawn_pick(&c, &mut rng),
                Some(NpcKind::FlyingDutchman),
                "outside the window the ship never spawns"
            );
        }
    }

    #[test]
    fn the_martian_spawn_table_picks_the_ground_four() {
        // `invasionType == 4`'s ported table (`NPC.cs:1487-1540`): every
        // rolled kind is one of the ported ten — the ground four (the
        // GrayGrunt 385, the GigaZapper 389, the Officer 383, the Walker
        // 520's walk-in), the Engineer 386's rider arm and the Drone 388's
        // num8 >= 4 arm; the Saucer 395/Rider 390/RayGunner 382/Scrambler
        // 381 arms consume their rolls where the vanilla's gates hit.
        let w = ground();
        let c = inv_pick_ctx(&w, 4, false);
        let mut rng = FastRandom::new(13);
        let mut seen_engineer = false;
        let mut seen_drone = false;
        let mut seen_walker = false;
        for _ in 0..600 {
            let kind = invasion_spawn_pick(&c, &mut rng).expect("type 4 picks");
            assert!(
                matches!(
                    kind,
                    NpcKind::GrayGrunt
                        | NpcKind::GigaZapper
                        | NpcKind::MartianOfficer
                        | NpcKind::MartianWalker
                        | NpcKind::MartianEngineer
                        | NpcKind::MartianDrone
                        | NpcKind::RayGunner
                        | NpcKind::BrainScrambler
                        | NpcKind::ScutlixRider
                        | NpcKind::MartianSaucerCore
                ),
                "the unported kinds never surface: {kind:?}"
            );
            seen_engineer |= kind == NpcKind::MartianEngineer;
            seen_drone |= kind == NpcKind::MartianDrone;
            seen_walker |= kind == NpcKind::MartianWalker;
        }
        assert!(seen_engineer, "the num6 >= 6 arm's Engineer half");
        assert!(seen_drone, "the num8 >= 4 arm's Drone");
        assert!(seen_walker, "the walk-in gate's Walker");
        let mut rng = FastRandom::new(3);
        let mut seen_ray = false;
        let mut seen_scrambler = false;
        let mut seen_rider = false;
        for _ in 0..1200 {
            match invasion_spawn_pick(&c, &mut rng) {
                Some(NpcKind::RayGunner) => seen_ray = true,
                Some(NpcKind::BrainScrambler) => seen_scrambler = true,
                Some(NpcKind::ScutlixRider) => seen_rider = true,
                _ => {}
            }
        }
        assert!(seen_ray, "the num8 < 2 arm's RayGunner");
        assert!(seen_scrambler, "the num8 2|3 arm's BrainScrambler");
        assert!(seen_rider, "the num6 >= 6 arm's Rider half");
        // The walker's uniqueness: while one lives, the num9 == 3 arm
        // re-rolls the ground chain instead.
        let mut c = inv_pick_ctx(&w, 4, false);
        c.invasion_walker_alive = true;
        let mut rng = FastRandom::new(5);
        for _ in 0..200 {
            assert!(matches!(
                invasion_spawn_pick(&c, &mut rng).expect("type 4 picks"),
                NpcKind::GrayGrunt
                    | NpcKind::GigaZapper
                    | NpcKind::MartianOfficer
                    | NpcKind::MartianEngineer
                    | NpcKind::MartianDrone
                    | NpcKind::RayGunner
                    | NpcKind::BrainScrambler
                    | NpcKind::ScutlixRider
                    | NpcKind::MartianSaucerCore
            ));
        }
    }

    #[test]
    fn the_invasion_ladder_draws_the_spawner_stream_in_vanilla_order() {
        // DW-1.2: the whole path on one stream — `GetZombieSettings`'s
        // `Next(7)` first (`NPC.cs:1284`), then the goblin ladder's rolls
        // (`Next(9)/(5)/(3)/(3)`, `:1429-1440`) with the vanilla
        // thresholds. The reference replays that draw order on a fresh
        // stream: both the kind and the stream tail must match, so any
        // extra/missing draw (or an `item_rng` interleave) breaks it.
        let w = ground_tall();
        let mut c = ctx(&w, 295.0 * 16.0, true);
        c.invaders = true;
        c.invasion_type = 1;
        let mut spawned = 0usize;
        for seed in 1..500u64 {
            let mut rng = FastRandom::new(seed);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the invasion arm spawns on the found tile");
            let mut ref_rng = FastRandom::new(seed);
            let _style = ref_rng.next(7); // GetZombieSettings
            let kind = if ref_rng.next(9) == 0 {
                NpcKind::GoblinSorcerer // 29
            } else if ref_rng.next(5) == 0 {
                NpcKind::GoblinThief // 26
            } else if ref_rng.next(3) == 0 {
                NpcKind::GoblinPeon // 111
            } else if ref_rng.next(3) == 0 {
                NpcKind::GoblinWarrior // 27
            } else {
                NpcKind::GoblinArcher // 28
            };
            assert_eq!(n.kind, kind, "seed {seed}: the ladder's kind");
            assert_eq!(
                rng.0, ref_rng.0,
                "seed {seed}: the stream tail is the vanilla draw order"
            );
            spawned += 1;
        }
        assert_eq!(spawned, 499);
    }

    #[test]
    fn the_type4_ladder_draws_in_vanilla_order() {
        // DW-1.2 (the martian table): `Next(7)` (num6), the always-drawn
        // `Next(45)` — `(Next(45) == 0) & flag6` is a bitwise `&` (`:1494`)
        // — then the num6 arms (`:1500-1540`). Two sweeps: the progress
        // gate closed (the Saucer draws consume, nothing spawns) and open.
        let w = ground_tall();
        let mut c = ctx(&w, 295.0 * 16.0, true);
        c.invaders = true;
        c.invasion_type = 4;
        let ray_arm = |rng: &mut FastRandom| match rng.next(5) {
            0 | 1 => NpcKind::RayGunner,      // 382
            2 | 3 => NpcKind::BrainScrambler, // 381
            _ => NpcKind::MartianDrone,       // 388
        };
        let ground_chain = |rng: &mut FastRandom, walker_alive: bool| -> NpcKind {
            let mut num9 = rng.next(4);
            if num9 == 3 {
                if !walker_alive {
                    return NpcKind::MartianWalker; // 520
                }
                num9 = rng.next(3);
            }
            match num9 {
                0 => NpcKind::GrayGrunt,      // 385
                1 => NpcKind::GigaZapper,     // 389
                _ => NpcKind::MartianOfficer, // 383
            }
        };
        // The gate closed (progress_30 false): the `Next(45)`/`Next(20)`
        // draws still consume (the C#'s `&`), the 395 arms never fire.
        for seed in 1..500u64 {
            let mut rng = FastRandom::new(seed);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the type-4 ground arm spawns");
            let mut ref_rng = FastRandom::new(seed);
            let _style = ref_rng.next(7);
            let num6 = ref_rng.next(7);
            let _saucer_gate = ref_rng.next(45); // always drawn
            let kind = if num6 >= 6 {
                let _rider_gate = ref_rng.next(20); // always drawn
                if ref_rng.next(2) == 0 {
                    NpcKind::ScutlixRider // 390
                } else {
                    NpcKind::MartianEngineer // 386
                }
            } else if num6 >= 4 {
                ray_arm(&mut ref_rng)
            } else {
                ground_chain(&mut ref_rng, false)
            };
            assert_eq!(n.kind, kind, "seed {seed}: the closed-gate kind");
            assert_eq!(rng.0, ref_rng.0, "seed {seed}: the closed-gate stream tail");
        }
        // The gate open (progress_30, no Saucer alive): the 395 arms fire
        // on their rolls.
        let mut c = ctx(&w, 295.0 * 16.0, true);
        c.invaders = true;
        c.invasion_type = 4;
        c.invasion_progress_30 = true;
        let mut saucers = 0u32;
        for seed in 1..500u64 {
            let mut rng = FastRandom::new(seed);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the type-4 ground arm spawns");
            let mut ref_rng = FastRandom::new(seed);
            let _style = ref_rng.next(7);
            let num6 = ref_rng.next(7);
            let kind = if ref_rng.next(45) == 0 {
                NpcKind::MartianSaucerCore // 395
            } else if num6 >= 6 {
                if ref_rng.next(20) == 0 {
                    NpcKind::MartianSaucerCore // 395
                } else if ref_rng.next(2) == 0 {
                    NpcKind::ScutlixRider // 390
                } else {
                    NpcKind::MartianEngineer // 386
                }
            } else if num6 >= 4 {
                ray_arm(&mut ref_rng)
            } else {
                ground_chain(&mut ref_rng, false)
            };
            assert_eq!(n.kind, kind, "seed {seed}: the open-gate kind");
            assert_eq!(rng.0, ref_rng.0, "seed {seed}: the open-gate stream tail");
            saucers += (kind == NpcKind::MartianSaucerCore) as u32;
        }
        assert!(saucers > 0, "the 395 arms fire with the gate open");
    }

    #[test]
    fn the_drone_channel_and_the_ground_ladder_are_mutually_exclusive() {
        // DW-1.3: the type-4 skyMob Drone channel (`NPC.cs:1395-1398`, the
        // port's drone arm) still fires — and per tile exactly one of the
        // two invasion arms runs: a skyMob candidate returns inside the
        // dispatch, the ground descent never co-fires (the C#'s else-if
        // chain; `FindSpawnTile`'s skyMob latches are `!invaders`-gated,
        // `:981/:985`, so an invasion attempt never latches skyMob at all).
        let w = ground_tall();
        let mut c = ctx(&w, 295.0 * 16.0, true);
        c.invaders = true;
        c.invasion_type = 4;
        // The skyMob-latched candidate: the Drone for every seed (the
        // dispatch returns at its type-4 arm — the probe/Wyvern riders
        // need hardmode, closed here).
        for seed in 1..100u64 {
            let mut rng = FastRandom::new(seed);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, true, false, false)
                .expect("the drone arm fires");
            assert_eq!(n.kind, NpcKind::MartianDrone, "seed {seed}");
        }
        // The unlatched candidate: the ground ladder's kinds run (a ground
        // kind at least once across the sweep — the Drone itself can also
        // come out of the ground table's num8 >= 4 arm, which is that
        // ladder's own outcome, not the skyMob channel's).
        let mut saw_ground = false;
        for seed in 1..200u64 {
            let mut rng = FastRandom::new(seed);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the ground arm fires");
            saw_ground |= n.kind != NpcKind::MartianDrone;
        }
        assert!(saw_ground, "the ground ladder ran on the unlatched tile");
    }
}
