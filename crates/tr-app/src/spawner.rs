//! Port of the NPC spawning pipeline in `Terraria/NPC.cs` (`Spawner`):
//! `TrySpawnAnNPC` — spawn-rate modifiers, the spawn-area scan, the on-screen
//! and free-space checks, and the default surface day/night enemy selection
//! for the kinds implemented in `npc.rs`.
//!
//! Biome-specific spawn tables (corruption, jungle, hallow, dungeon, ...) are
//! part of the much larger `Spawner.SpawnAnNPC` switch and are out of scope
//! here; when a chosen C# type has no `NpcKind` equivalent the attempt simply
//! returns `None` (the original would spawn that type).

use tr_core::{UnifiedRandom, World, TILE_PX};

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

// The spawner's draws are `Main.rand.Next(...)` under the `SpawnNPC` swap —
// the persistent `UnifiedRandom` stream the npcs pass binds per tick
// (`world_sim/npcs.rs`).

/// Inputs to one spawn attempt, mirroring what `Spawner` reads off `Main` and
/// the `Player` in `SetSpawnFlags`.
/// The world/scene state `GetSpawnRate` reads (`NPC.cs:473-810`). Everything
/// the ported simulation cannot supply yet stays `false`/`0` — the chain
/// branches still run in the vanilla order so future systems only flip flags.
#[derive(Clone, Copy)]
pub struct SpawnFlags {
    pub hard_mode: bool,
    pub blood_moon: bool,
    pub eclipse: bool,
    /// `Main.pumpkinMoon || Main.snowMoon`.
    pub moon_event: bool,
    pub zone_dungeon: bool,
    /// `NPC.downedBoss3` (Skeletron defeated).
    pub downed_boss3: bool,
    /// `Spawner.hardDungeon` (`NPC.cs:380`): `NPC.downedPlantBoss &&
    /// Main.hardMode` — the hardmode-dungeon roster's gate (the 289 Giant
    /// Cursed Skull roll `:2683-2687` included). Plantera-gated, NOT just
    /// hardmode.
    pub hard_dungeon: bool,
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
    /// `Main.IsJourneyMode` (`Main.cs:2634`: `GameMode == 3`) — the
    /// spawn-rate power's gates (`NPC.cs:350`/`:762`/`:5906`). The port's
    /// stand-in is the session difficulty (`App::active_player_difficulty`,
    /// the same one `journey_available` reads); a Journey character only
    /// joins a Journey world (`UIWorldSelect.cs:177-180`'s rejection).
    pub journey_mode: bool,
    /// `SpawnRateSliderPerPlayerPower.GetShouldDisableSpawnsFor`
    /// (`CreativePowers.cs:1584-1595`): the local player's exact-zero
    /// cache — `NPC.CanSpawnEnemiesNear` refuses the whole attempt
    /// (`NPC.cs:350-357`).
    pub journey_spawns_disabled: bool,
    /// `SpawnRateSliderPerPlayerPower.GetRemappedSliderValueFor`
    /// (`:222-231`): the local player's x0.1..10 factor — `GetSpawnRate`
    /// divides the rate and multiplies the cap by it (`NPC.cs:762-770`).
    /// The fresh default 0.5 remaps to 1.0 (inert).
    pub journey_spawn_rate: f32,
}

impl Default for SpawnFlags {
    fn default() -> Self {
        Self {
            hard_mode: false,
            blood_moon: false,
            eclipse: false,
            moon_event: false,
            zone_dungeon: false,
            downed_boss3: false,
            zone_sandstorm: false,
            zone_underground_desert: false,
            zone_jungle: false,
            zone_evil: false,
            zone_meteor: false,
            zone_temple: false,
            zone_hallow_deep: false,
            zone_water_candle: false,
            zone_peace_candle: false,
            zone_graveyard: false,
            invis: false,
            calmed: false,
            sunflower: false,
            angler_set: false,
            enemy_spawns: false,
            wof_active: false,
            journey_mode: false,
            journey_spawns_disabled: false,
            // The power's fresh cache 0.5 (`_sliderDefaultValue`,
            // `CreativePowers.cs:1580`) remaps to x1 — inert.
            journey_spawn_rate: 1.0,
        }
    }
}

/// The wave-20 boss kinds the `Spawner` ctor weights (`NPC.cs:273-286`):
/// 315/325/327/328/344/345/346, each with its `SetDefaults` `npcSlots`
/// (the common reset is `npcSlots = 1f`, `NPC.cs:8510`; the per-type writes
/// are `NPC.cs:12978`/`:13115`/`:13142`/`:13158`/`:13355`/`:13369`/`:13386`).
pub const MOON_BOSS_SLOTS: [(u16, f32); 7] = [
    (315, 1.0),
    (325, 3.0),
    (327, 5.0),
    (328, 0.0),
    (344, 3.0),
    (345, 5.0),
    (346, 4.0),
];

/// A boss kind's weight in the ctor's sum — `None` for every other type.
pub fn moon_boss_slots(npc_type: u16) -> Option<f32> {
    MOON_BOSS_SLOTS
        .iter()
        .find(|(id, _)| *id == npc_type)
        .map(|(_, slots)| *slots)
}

/// `Spawner`'s ctor weighted sum over the live boss kinds
/// (`NPC.cs:274-283`) — the ctx's boss-cap input (the `invasion_saucer_alive`
/// pattern). `S1-F-13b` lands 315/325/327 as real kinds whose `to_u16` rides
/// the vanilla ids, so the count goes live for exactly those three (1.0/3.0/
/// 5.0 - their `SetDefaults` `npcSlots` rows); `S1-F-13c` slices A+B add the
/// Everscream (344, 3.0), the Ice Queen (345, 5.0) and Santa-NK1 (346, 4.0).
/// W4-c6 lands the blade kind (328): its row is live but `npcSlots = 0f`
/// (`NPC.cs:13172`), so a living blade still contributes nothing.
pub fn moon_boss_cap_count(npcs: &[Npc]) -> f32 {
    npcs.iter()
        .filter(|n| n.hp > 0)
        .filter_map(|n| moon_boss_slots(n.kind.to_u16()))
        .sum()
}

/// The ctor's bound for the port's single active player
/// (`NPC.cs:285`): `(int)(defaultMaxSpawns × (2 + 0.3 × players))` = 11 — the
/// same `5·2.3` the moon raid pacing reads (`spawner.rs:396`).
pub fn moon_boss_cap_bound() -> i32 {
    (DEFAULT_MAX_SPAWNS as f64 * (2.0 + 0.3 * 1.0)) as i32
}

/// `Spawner.reachedInvasionBossCap` (`NPC.cs:286`): the C#'s `>=` compare.
pub fn moon_boss_cap_reached(count: f32) -> bool {
    count >= moon_boss_cap_bound() as f32
}

/// `Main.wallLight` (`Main.cs:10578-10595`) — the wall list
/// `SetSpawnFlags`' `skyBehindPlayer` gate reads (plus wall 73's separate
/// `||` arm, `NPC.cs:412`).
pub(crate) const LIGHT_WALLS: [u16; 16] = [
    0, 21, 318, 106, 317, 107, 138, 140, 141, 139, 145, 150, 152, 245, 315, 168,
];

/// All fields are `Copy` (world/roster borrows + scalars) — the W4-f32
/// witnesses twin a ctx to compare luck 0 against a luck feed.
#[derive(Clone, Copy)]
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
    /// `Spawner.luck` (`NPC.cs:369`: `this.luck = player.luck`) — the
    /// `RollLuck`/`RollOnlyBadLuck` consumers inside the spawn chain read
    /// it (W4-f32). No player (headless shots) reads the neutral 0.
    pub luck: f32,
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
    /// The `Spawner` ctor's boss cap input (`NPC.cs:262-286`): the weighted
    /// `npcSlots` count over the live wave-20 boss kinds — the moon ladders
    /// compare it against [`moon_boss_cap_reached`]. Structurally 0 until the
    /// kinds land (F-13b/c).
    pub moon_boss_cap_count: f32,
    /// The live vanilla types of the active NPC roster — the C#'s
    /// `NPC.CountNPCS(id)`'s scan source (`NPC.cs:7982-7995`): the moon
    /// ladders' `CountNPCS(id) < cap` gates read it per arm.
    pub moon_npc_counts: &'a [u16],
    /// W4-f42 — `NPC.downedBoss1` (the EoC defeated): the sandstorm spawn
    /// arm's pre-hardmode gate (`NPC.cs:3951`).
    pub downed_boss1: bool,
    /// W4-f42 — `Sandstorm.Happening`: the sandstorm spawn arm's event gate
    /// (`NPC.cs:3949`).
    pub sandstorm_happening: bool,
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
/// (`:924-928`, `RollOnlyBadLuckExtreme(50)`) reads `ctx.luck`
/// (W4-f32): at luck >= 0 the roll returns -1 WITHOUT a draw, so the
/// classic stream is byte-identical; negative luck can shrink the rate
/// ×0.85 and grow the cap ×1.15.
pub fn get_spawn_rate(ctx: &SpawnContext, rng: &mut UnifiedRandom) -> SpawnRate {
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
    // The moon-event raid pacing (`NPC.cs:771-775`): gated on the PLAYER
    // being above the surface line (`player.position.Y < worldSurface * 16`;
    // `Main.remixWorld` has no port counterpart) — an underground player
    // during a moon event keeps the un-paced rate (research 24, errata).
    if f.moon_event && pos_y < ctx.world.world_surface * 16.0 {
    // `Main.getGoodWorld` (`:757-761`) is constants-off (no FTW worlds).
    // `Main.IsJourneyMode` (`:762-770`): the spawn-rate power divides the
    // rate and multiplies the cap by the remapped x0.1..10 factor — the
    // `(int)` casts truncate, the factor is the raw float (no rounding).
    if f.journey_mode {
        let value = f.journey_spawn_rate;
        rate.spawn_rate = (rate.spawn_rate as f32 / value) as i32; // `:767`
        rate.max_spawns = (rate.max_spawns as f32 * value) as i32; // `:768`
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
                if rng.next_max(2) == 0 {
                    rate.no_worms = true; // `:809`
                }
                if rng.next_max(10) == 0 {
                    rate.spawn_friendly = true; // `:813`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.5) as i32;
                // `:814`
                } else {
                    rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 1.25) as i32;
                    // `:818`
                }
            } else if ctx.town_npcs == 2 {
                if rng.next_max(4) != 0 {
                    rate.no_worms = true; // `:826`
                }
                if rng.next_max(5) == 0 {
                    rate.spawn_friendly = true; // `:830`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.5) as i32;
                // `:831`
                } else {
                    rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 1.5) as i32;
                    // `:835`
                }
            } else if ctx.town_npcs >= 3 {
                if rng.next_max(10) != 0 {
                    rate.no_worms = true; // `:842`
                }
                if rng.next_max(3) == 0 {
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
            if f.zone_graveyard && (!f.zone_peace_candle || rng.next_max(3) == 0) {
                // `:860`
                rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 1.66) as i32; // `:862`
                if rng.next_max(9) == 1 {
                    // `:863`
                    rate.spawn_friendly = true; // `:865`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32;
                    // `:866`
                }
            } else if rng.next_max(3) == 1 {
                // `:869`
                rate.spawn_friendly = true; // `:871`
                rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32; // `:872`
            } else {
                rate.spawn_rate = (rate.spawn_rate as f32 * 2.0) as i32; // `:876`
            }
        } else if ctx.town_npcs == 2 {
            rate.no_worms = true; // `:882`
            if f.zone_graveyard && (!f.zone_peace_candle || rng.next_max(3) == 0) {
                // `:883`
                rate.spawn_rate = (rate.spawn_rate as f32 as f64 * 2.33) as i32; // `:885`
                if rng.next_max(6) == 1 {
                    // `:886`
                    rate.spawn_friendly = true; // `:888`
                    rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32;
                    // `:889`
                }
            } else if rng.next_max(3) != 0 {
                // `:892`
                rate.spawn_friendly = true; // `:894`
                rate.max_spawns = (rate.max_spawns as f32 as f64 * 0.6) as i32; // `:895`
            } else {
                rate.spawn_rate = (rate.spawn_rate as f32 * 3.0) as i32; // `:899`
            }
        } else if ctx.town_npcs >= 3 {
            rate.no_worms = true; // `:904`
            if f.zone_graveyard && (!f.zone_peace_candle || rng.next_max(3) == 0) {
                // `:905`
                rate.spawn_rate = (rate.spawn_rate as f32 * 3.0) as i32; // `:907`
                if rng.next_max(3) == 1 {
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
        // The luck tail (`NPC.cs:924-928`): `!spawnFriendly &&
        // RollOnlyBadLuckExtreme(50) == 0` shrinks the rate ×0.85 and
        // grows the cap ×1.15. `Luck.cs:53-60`: only NEGATIVE luck draws
        // (the `NextFloat` gate, then `Next(50 / 10)` = `Next(5)`);
        // luck >= 0 returns -1 without touching the stream, so the
        // classic world consumes exactly what it did before (W4-f32).
        if !rate.spawn_friendly
            && tr_core::loot::roll_only_bad_luck_extreme(rng, ctx.luck, 50) == 0
        {
            rate.spawn_rate = (rate.spawn_rate as f32 * 0.85) as i32; // `:926`
            rate.max_spawns = (rate.max_spawns as f32 * 1.15) as i32; // `:927`
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
    rng: &mut UnifiedRandom,
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

/// `NPC.SpawnOnPlayer`'s generic ground scan (`NPC.cs:80565-80632`) — the
/// default arm the Plantera bulb's `SpawnOnPlayer(plr, 262)` rides (the
/// per-type arms 245/370/398/… are their own ports: the Golem altar's
/// `doors.rs::spawn_golem_on_altar`, the Duke's bobber scan). 500 attempts ×
/// 50 candidates: a random tile in the spawn area, refused when
/// `nactive && tileSolid` or `wallHouse && m < 499` (the house-wall arm is
/// unmodelled — the port has no `wallHouse` table, the `try_spawn_an_npc`
/// precedent); descend to the first `nactive && tileSolid` cell; the LAST
/// attempt (`flag2`) bypasses the safe-area + spawn-space gates, every other
/// attempt needs `!safeArea.Contains` + `HasTileSpawnSpace` +
/// `CheckNotSpawningOnScreen`. Returns the chosen ground tile — the caller
/// runs `SpawnBoss(num10 * 16 + 8, num11 * 16, …)` (`:80631`).
///
/// Deviations (the `try_spawn_an_npc` precedent): the skyblock midair arm
/// (`WorldGen.Skyblock.lowTiles`, `:80581-80584`) and the type-50 EoW
/// clearance special (`:80604-80614`) are constants-off / type-gated out;
/// the `nactive` actuated-bit refinement follows the port's spawner
/// convention (`active()` — the actuated-spawn edge is unmodelled).
pub fn spawn_on_player_tile(ctx: &SpawnContext, rng: &mut UnifiedRandom) -> Option<(i32, i32)> {
    let (spawn_area, safe_area) = get_spawn_area(ctx);
    let num12 = 500; // `:80568` (the 316/82 arm's 200/30 is type-gated out).
    let num13 = 50; // `:80569`.
    // `num14 = num12 / 2` (`:80575`) feeds only the skyblock midair arm
    // (`:80581-80584`) and the type-50 clearance special — both out of scope.
    let solid = |x: i32, y: i32| {
        ctx.world
            .tile(x, y)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
    };
    let mut found: Option<(i32, i32)> = None;
    for m in 0..num12 {
        let flag2 = m == num12 - 1; // `:80579`.
        for _ in 0..num13 {
            let num15 = rng.next_range(spawn_area.left, spawn_area.right); // `:80587`.
            let num16 = rng.next_range(spawn_area.top, spawn_area.bottom); // `:80588`.
            // `:80589` — the solid / house-wall refusal (the wallHouse arm
            // unmodelled; the type-50 `Main.tile[num16, num16]` quirk arm is
            // type-gated out).
            if solid(num15, num16) {
                continue;
            }
            // `:80593-80603` — descend to the first solid cell.
            let mut ground: Option<i32> = None;
            for num17 in num16..spawn_area.bottom {
                if solid(num15, num17) && (flag2 || !safe_area.contains(num15, num16)) {
                    ground = Some(num17);
                    break;
                }
            }
            let Some(num11) = ground else { continue };
            // `:80615-80618` — the non-final attempts need the spawn space.
            if !flag2 && !has_tile_spawn_space(ctx.world, num15, num11) {
                continue;
            }
            found = Some((num15, num11));
            break;
        }
        // `:80624-80628` — `(!flag || flag2 || CheckNotSpawningOnScreen) &&
        // flag`: the final attempt spawns anywhere, the others must lie
        // off-screen.
        if let Some((x, y)) = found {
            if flag2 || check_not_spawning_on_screen(ctx, &safe_area, x, y) {
                return Some((x, y));
            }
            found = None;
        }
    }
    None
}

/// `NPC.SpawnBoss`'s post-spawn stamp (`NPC.cs:80661-80714`) for the port's
/// single-player roster: `NewNPC` seeds `timeLeft = activeTime * 1.25` = 937
/// (`:80772`, `activeTime = 750` `:95186`), `SpawnBoss` multiplies it ×20
/// (`:80686`) → 18740, and the default arm broadcasts `Announcement.
/// HasAwoken` in `ChatColors.BossOrEvent` (`:80702-80704`). The caller
/// places the NPC by `Bottom = (X, Y)` (`NewNPC` `:80770`) and pushes it.
pub fn stamp_boss_spawn(n: &mut Npc, chat: &mut crate::chat::ChatMonitor, name: &str) {
    n.despawn_timer = 937 * 20; // `:80772` × `:80686`.
    crate::chat::broadcast_death_text(
        chat,
        &format!("{} has awoken!", name),
        crate::chat::COLOR_BOSS_OR_EVENT,
    );
}

pub fn try_spawn_an_npc(ctx: &SpawnContext, rng: &mut UnifiedRandom) -> Option<Npc> {
    let (spawn_area, safe_area) = get_spawn_area(ctx);

    // `GetSpawnRate` + the two hard gates in `TrySpawnAnNPC`.
    let rate = get_spawn_rate(ctx, rng);
    let (no_worms, spawn_friendly) = (rate.no_worms, rate.spawn_friendly);
    if ctx.nearby_active_npcs >= rate.max_spawns as f32 {
        return None;
    }
    if rng.next_max(rate.spawn_rate) != 0 {
        return None;
    }

    // `FindSpawnTile` (NPC.cs:969-1010): 50 random candidates. The per-
    // attempt `skyMob` latch (`SetSpawnFlags`' reset, `NPC.cs:372`) rides
    // the loop — the skyMob candidates skip the ground descent (the spawn
    // tile stays the random air tile).
    let mut sky_mob = false;
    for _ in 0..50 {
        let num = rng.next_max(spawn_area.right - spawn_area.left) + spawn_area.left;
        let mut j = rng.next_max(spawn_area.bottom - spawn_area.top) + spawn_area.top;
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
            && rng.next_max(10) == 0
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
/// - `RollOnlyBadLuck`/`RollLuck` now ride `ctx.luck` (W4-f32): the
///   `GetSpawnRate` tail `:924-928`, the Pinky promotion `:5242`, the
///   bound Goblin/Wizard `:2084`/`:2088` and the blood-moon 300 pair
///   `:4618`/`:4623` — at luck 0 every roll degenerates to the same plain
///   `Next(n)` (byte-identical stream); the goldCritterChance arms stay
///   constants-off with their unported kinds;
/// - `GetZombieSettings`' skyblock style-forcing and armed-zombie cut are
///   constants-off (M5-SEED-1);
/// - the skyMob dispatch (`NPC.cs:1354-1412`): the Drone arm consumes the
///   dispatch position (the port's invasion path short-circuits the
///   ladder), the probe arm is verbatim (`:1399-1402`), and the Wyvern arm
///   (`:1407-1410`) consumes its `Next(10)` and returns no spawn — the
///   kind lands with the flyer family.
fn spawn_npc_at(
    ctx: &SpawnContext,
    rng: &mut UnifiedRandom,
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
    let zombie_style = rng.next_max(7);
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
        let probe_roll = if !ctx.downed_martians && rng.next_max(mv2) == 0 {
            true
        } else {
            rng.next_max(mv3) == 0
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
        if ctx.flags.hard_mode && rng.next_max(10) == 0 {
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
        if is_ocean && rng.next_max(10) == 0 {
            return Npc::spawn_swimmer(ctx.world, x, y / 16, NpcKind::Shark);
        }
        // The Piranha arm (`:1929`): `!spawnFriendly && waterTile &&
        // ((deeperThanRockLayer && Next(2) == 0) || mud floor 60)` — the
        // `waterTile` gate is the enclosing water ladder; the town block's
        // `spawnFriendly` carry-out short-circuits before the draw.
        if !spawn_friendly
            && (((y as f64 / 16.0) > ctx.world.rock_layer && rng.next_max(2) == 0) || tile_kind == 60)
        {
            // The mud-critter scan (`:1932`'s `tileType == 60 &&
            // surfaceSpawn && day && Next(3) == 0` → 616/612/613/608) is
            // unmodelled — the roll consumes where the vanilla's gate hits.
            if tile_kind == 60 && surface && y / 16 > 50 && rng.next_max(3) == 0 && ctx.day_time {
                let _ = rng.next_max(5);
            }
            return Npc::spawn_swimmer(ctx.world, x, y / 16, NpcKind::Piranha);
        }
        // The underground fish (`:1985`): 63/103 unmodelled — the roll
        // consumes, the port maps to the Goldfish (the closest water
        // critter; the Angler Fish lands with the fish-family pass).
        if (y as f64 / 16.0) > ctx.world.world_surface && rng.next_max(3) == 0 {
            return Npc::spawn_swimmer(ctx.world, x, y / 16, NpcKind::Goldfish);
        }
        // The water critter arm (`:1996`): `Next(4) == 0 && (x outside the
        // ocean || y > worldSurface + 50)`; the corrupt 57/crimson 465
        // kinds are unmodelled — the port maps them to the Piranha (the
        // aggressive water fish); the day critter scans (the ducks/the
        // seahorses) sit constants-off; the golden goldfish 592's
        // `goldCritterChance` roll consumes nothing.
        if rng.next_max(4) == 0
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
    // ---- the dungeon band (`NPC.cs:2626-2793`, the zone chain's
    // `else if (this.ZoneDungeon)` arm) ---- W4-c3 gaps 2+3: the 694
    // Water Bolt Mimic's book-find spawn + the 289 Giant Cursed Skull's
    // hardmode roll + the 34 Cursed Skull's 1/7 roll. The arm sits AFTER
    // the water ladder (`:2217-2322`'s terminal water block) and the
    // bound-NPC rolls (`:2084-2095` — the port keeps those in the
    // deep-cavern section below, the recorded fold) and BEFORE the
    // surface/moon ladders. The unmodelled pre-dungeon arms (the wall-244
    // spider/gnome/critter chain `:1584-1655`, the bartender/spider/
    // underground-desert/golfer specials `:1656-1695`, the evil-water
    // hardmode arms `:1763-1774`, the beach/angler chains, the granite/
    // marble scan's two `SetSpawnFlagsForChosenTile` draws `:1076-1077`
    // and the surface tile-switch critter arms `:2323-2624`) are the
    // documented consume-skip deviation: on a dungeon spawn tile they
    // consume nothing here, exactly as the port's other unmodelled
    // branches do (ANALYSIS "the dungeon band").
    if ctx.flags.zone_dungeon {
        return spawn_dungeon_band(ctx, rng, x, y / 16);
    }
    if surface {
        // ---- the moon-event roster ladders (`NPC.cs:2804-3548`) ---- the
        // C#'s `!Main.dayTime && (snowMoon || pumpkinMoon)` arms sit in the
        // zone chain's `else` after ZoneMeteor (`:2798`) — after every water
        // arm above (`:1763-2079`) and ahead of the `surfaceSpawn` probe arm
        // and the natural ladders (`:4164`+), and matching an arm skips the
        // chain's remaining branches, so the port's arm is terminal: it draws
        // the vanilla rolls and returns. The events are mutually exclusive
        // (each start clears the other, `Main.cs:10979-11017`), so which flag
        // is alive picks the ladder.
        //
        // `Main.remixWorld`'s surface-band alternative has no port counterpart
        // — the `(double)spawnTileY <= Main.worldSurface` twin is the
        // reduction, and the enclosing `if surface` IS that band.
        if !ctx.day_time && (ctx.world.snow_moon || ctx.world.pumpkin_moon) {
            let pick = if ctx.world.snow_moon {
                frost_moon_ladder(ctx, rng)
            } else {
                pumpkin_moon_ladder(ctx, rng)
            };
            // `S1-F-13b` Phase 2 / `S1-F-13c` slice A: the 16 Pumpkin Moon
            // wave kinds and the first 7 Frost Moon kinds (338-344) are
            // ported, so the ladder's vanilla `NPC.type` maps to the kind
            // and spawns at the found tile — `SpawnNPC(spawnTileX * 16 + 8,
            // spawnTileY * 16, type)` (`NPC.cs:3224-3548`'s moon arms) →
            // `NewNPC`'s `Bottom = (X, Y)` (`:80772`): the feet on the found
            // tile's top edge, X centred 8 px into the tile, the facing
            // toward the player (`TargetClosest`) — the placement the
            // invaders arm rides.
            // Every other pick (the pumpkin wave-20 unported bosses) still
            // consumes its draw and spawns nothing - the Wyvern arm's
            // consume-and-return-None shape (`spawner.rs:989-994`).
            // `S1-F-13c` slices A+B land all 15 frost kinds (338-352), so
            // every frost pick spawns through the same map.
            if let Some(kind) = pick.and_then(crate::npc::NpcKind::moon_wave_kind) {
                let (nw, nh) = kind.size();
                let mut n = Npc::new(
                    kind,
                    x as f32 * TILE_PX + 8.0 - nw * 0.5,
                    y as f32 - nh,
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
            return None;
        }
        // W4-f42 — the sandstorm spawn arm (`NPC.cs:3949-4018`): the
        // zone chain's `else if (Sandstorm.Happening && ZoneSandstorm &&
        // TileID.Sets.Conversion.Sand[tileType] && Spawning_SandstoneCheck)`
        // — after the moon-event arms, before the post-sandstorm mummy
        // arms (`:4020-4035`) and the surface probe. The pre-hardmode
        // ladder (546/61/69), the hardmode ladder (541/510/542-545/78/79/
        // 630/80), and the 546/580/581 fallthrough. The 541 SandElemental
        // / 510 DuneSplicer / 542-545 SandSharks are unported kinds —
        // their rolls consume and spawn nothing (the Wyvern arm's
        // consume-and-return-None shape). d33's unwired npc_ai_style102/
        // 103 modules are keyed on the vanilla npc_type; wiring these
        // arms waits on the roster kinds (research/108-input-f42 §5).
        if ctx.sandstorm_happening
            && ctx.flags.zone_sandstorm
            && matches!(tile_kind, 53 | 112 | 116 | 234)
            && spawning_sandstone_check(ctx.world, x, y / 16)
        {
            let hard = ctx.flags.hard_mode;
            if !ctx.downed_boss1 && !hard {
                // `:3953-3964` — the 1/2 tumbleweed, else the 1/2 vulture,
                // else the antlion.
                if rng.next_max(2) == 0 {
                    return spawn_desert_ground(ctx, x, y, NpcKind::Tumbleweed);
                } else if rng.next_max(2) == 0 {
                    return spawn_desert_flyer(ctx, x, y, NpcKind::Vulture);
                } else {
                    return spawn_desert_ground(ctx, x, y, NpcKind::Antlion);
                }
            } else if hard && rng.next_max(20) == 0 && !ctx.moon_npc_counts.contains(&541) {
                // `:3966-3969` — the SandElemental (541, unported): consume.
                // (The roster scan rides `to_u16`'s protocol ids, but the
                // unported 541/510's vanilla ids are free protocol slots —
                // no ported kind encodes them, so `AnyNPCs(541)` is false
                // and `CountNPCS(510)` is 0: the gates never close. The
                // documented unported-kind fold, the Wyvern arm's shape.)
                return None;
            } else if hard && !no_worms && rng.next_max(3) == 0
                && ctx.moon_npc_counts.iter().filter(|&&t| t == 510).count() < 4
            {
                // `:3970-3973` — the DuneSplicer (510, unported): consume.
                return None;
            } else if hard && !no_worms && rng.next_max(2) == 0 {
                // `:3974-3990` — the SandShark family (542-545, unported):
                // the corrupt/crimson/hallow tile picks the type; consume.
                return None;
            } else if hard && tile_kind == 53 && rng.next_max(3) == 0 {
                // `:3991-3994` — the Mummy (78).
                return spawn_desert_ground(ctx, x, y, NpcKind::Mummy);
            } else if hard && tile_kind == 112 && rng.next_max(3) == 0 {
                // `:3995-3998` — the DarkMummy (79).
                return spawn_desert_ground(ctx, x, y, NpcKind::DarkMummy);
            } else if hard && tile_kind == 234 && rng.next_max(3) == 0 {
                // `:3999-4002` — the BloodMummy (630).
                return spawn_desert_ground(ctx, x, y, NpcKind::BloodMummy);
            } else if hard && tile_kind == 116 && rng.next_max(3) == 0 {
                // `:4003-4006` — the LightMummy (80).
                return spawn_desert_ground(ctx, x, y, NpcKind::LightMummy);
            } else if rng.next_max(2) == 0 {
                // `:4007-4010` — the Tumbleweed (546).
                return spawn_desert_ground(ctx, x, y, NpcKind::Tumbleweed);
            } else if rng.next_max(2) == 0 {
                // `:4011-4014` — the WalkingAntlion (580).
                return spawn_desert_ground(ctx, x, y, NpcKind::WalkingAntlion);
            } else {
                // `:4015-4018` — the FlyingAntlion (581).
                return spawn_desert_flyer(ctx, x, y, NpcKind::FlyingAntlion);
            }
        }
        // W4-f42 — the post-sandstorm mummy arms (`NPC.cs:4020-4035`): the
        // zone chain's next four `else if`s AFTER the sandstorm arm, so
        // they fire on the SAME conversion-sand rows with NO event gate —
        // the everyday hardmode desert spawns. The 53 row keeps its
        // `Next(3)` (1/3); the 112/234/116 rows tighten to `Next(2)` (1/2)
        // outside the event (`:4024`/`:4028`/`:4032`). Each arm is
        // terminal: a hit spawns, a miss falls through to the next arm's
        // own roll — exactly the else-if chain's order. The `:4036`
        // underground-hallow arm that follows needs `underGround`/remix
        // and never fires inside this surface block (unmodelled).
        if ctx.flags.hard_mode && tile_kind == 53 && rng.next_max(3) == 0 {
            // `:4020-4023` — the Mummy (78).
            return spawn_desert_ground(ctx, x, y, NpcKind::Mummy);
        } else if ctx.flags.hard_mode && tile_kind == 112 && rng.next_max(2) == 0 {
            // `:4024-4027` — the DarkMummy (79).
            return spawn_desert_ground(ctx, x, y, NpcKind::DarkMummy);
        } else if ctx.flags.hard_mode && tile_kind == 234 && rng.next_max(2) == 0 {
            // `:4028-4031` — the BloodMummy (630).
            return spawn_desert_ground(ctx, x, y, NpcKind::BloodMummy);
        } else if ctx.flags.hard_mode && tile_kind == 116 && rng.next_max(2) == 0 {
            // `:4032-4035` — the LightMummy (80).
            return spawn_desert_ground(ctx, x, y, NpcKind::LightMummy);
        }
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
        let probe_roll = if !ctx.downed_martians && rng.next_max(100) == 0 {
            true
        } else {
            rng.next_max(400) == 0
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
        let eyes = rng.next_max(6) == 0 || (ctx.world.moon_phase == 4 && rng.next_max(2) == 0);
        if eyes {
            // `:4551-4558`: hardmode takes the Wandering Eye 1/3 of the time.
            if ctx.flags.hard_mode && rng.next_max(3) == 0 {
                return Npc::spawn_eye(ctx.world, x).map(|mut n| {
                    n.kind = NpcKind::WanderingEye;
                    n
                });
            }
            if rng.next_max(2) == 0 {
                // `:4564-4576`: -43 on the 1/4 roll, else type 2 — both the
                // port's demon eye.
                let _ = rng.next_max(4);
                return Npc::spawn_eye(ctx.world, x);
            }
            // `:4577-4606`: the five 19X pairs with their 1/3 size-variant
            // roll; the C# spawns BOTH — the port draws the roll and spawns
            // the base (the deviation above).
            let _ = rng.next_max(5);
            let _ = rng.next_max(3);
            return Npc::spawn_eye(ctx.world, x);
        }
        // `:4608-4612`: hardmode blood moon 1/50 → 109 (unmodelled).
        if ctx.flags.hard_mode && ctx.flags.blood_moon && rng.next_max(50) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4614-4621`: the two bad-luck 300 blood-moon rolls (53, 536) —
        // `RollOnlyBadLuck(300)` (W4-f32): at luck >= 0 the plain
        // `Next(300)` (byte-identical to the pre-luck port); negative luck
        // shrinks the inner range to `Next(Next(150, 300))`.
        if ctx.flags.blood_moon && tr_core::loot::roll_only_bad_luck(rng, ctx.luck, 300) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        if ctx.flags.blood_moon && tr_core::loot::roll_only_bad_luck(rng, ctx.luck, 300) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4623-4627`: a new-moon hardmode 2/3 (104).
        if ctx.world.moon_phase == 0 && ctx.flags.hard_mode && rng.next_max(3) != 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4628-4631`: hardmode 1/3 (140).
        if ctx.flags.hard_mode && rng.next_max(3) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4632-4645`: blood moon 2/5 → 489/490 (the `Next(2)` picks).
        // S1-D-3z: the pick now spawns the matching kind — 0 → the Blood
        // Zombie (489, the aiStyle-3 walker), 1 → the Drippler (490, the
        // aiStyle-22 flyer). The draw ORDER is unchanged (the 2/5 gate
        // then the `Next(2)` pick), so the stream fingerprint is
        // byte-identical; only the spawned kind differs.
        if ctx.flags.blood_moon && rng.next_max(5) < 2 {
            if rng.next_max(2) == 0 {
                return Npc::spawn_zombie_variant(ctx.world, x, NpcKind::BloodZombie);
            }
            // The Drippler's `SpawnNPC` bottom placement (`NPC.cs:4646`):
            // the hover ctor at the surface column's top row (the
            // noGravity flyer's feet-on-surface stand-in — the machine
            // takes over on the first tick).
            let top = ctx.world.surface_of(x);
            return Npc::spawn_hover(ctx.world, x, top - 2, NpcKind::Drippler);
        }
        // `:4646-4665`: the snow/ice tile arm — hardmode's 169/155 rolls,
        // then the Eskimo default (161; all the port's zombie kind).
        // W4-d25 keep-one (f5f486c7c): the 155 roll's Wolf spawn DROPPED
        // — d36 owns the canonical Wolf@772 (a11f61850); the zombie
        // stand-in stands until that lane lands.
        if is_ices_snow(tile_kind) || tile_kind == 162 {
            if !ctx.flags.blood_moon && ctx.flags.hard_mode && rng.next_max(4) == 0 {
                return Npc::spawn_zombie(ctx.world, x);
            }
            if !ctx.flags.blood_moon && ctx.flags.hard_mode && rng.next_max(3) == 0 {
                return Npc::spawn_zombie(ctx.world, x);
            }
            let _ = rng.next_max(2); // the armed-expert split (unmodelled)
            return Npc::spawn_zombie(ctx.world, x);
        }
        // `:4666-4709`: the rain/graveyard specials — rain and the graveyard
        // flag are constants-off, no rolls.
        // `:4711-4721`: the torch zombie 1/chance (590) — the port's zombie.
        if rng.next_max(torch_zombie_chance) == 0 {
            return Npc::spawn_zombie(ctx.world, x);
        }
        // M5-SEED-2: `:4728-4731`'s halloween arm — `Main.halloween &&
        // Main.rand.Next(2) == 0` → `Next(319, 322)` = the ZombieDoctor/
        // Superman/Pixie (319-321, `NPCID.cs`). `Main.halloween` is
        // `Main.forceHalloweenForever`'s write (`Main.cs:13168`, set for the
        // `endlessHalloween` micro seed by `InitializeSecretSeeds`
        // `WorldGen.cs:572-574`), so the roll — and only the roll — runs
        // under the flag: the C# `&&` short-circuits it on a classic world.
        // S1-D-3z: the three halloween zombie types now have `NpcKind`
        // variants — the `Next(319, 322)` pick (0/1/2 → 319/320/321) spawns
        // the matching kind. The draw ORDER is unchanged (the gate's
        // `Next(2)` then the type pick), so the stream fingerprint is
        // byte-identical; only the spawned kind differs.
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN)
            && rng.next_max(2) == 0
        {
            let pick = rng.next_max(3); // `Next(319, 322)` — the type pick.
            let kind = match pick {
                0 => NpcKind::ZombieDoctor,
                1 => NpcKind::ZombieSuperman,
                _ => NpcKind::ZombiePixie,
            };
            return Npc::spawn_zombie_variant(ctx.world, x, kind);
        }
        // `:4732-4735`'s xmas arm — `Main.xMas && Next(2) == 0` →
        // `Next(331, 333)` = ZombieXmas/ZombieSweater (331/332), the same
        // `forceXMasForever` source (`Main.cs:13135`,
        // `WorldGen.cs:576-578`). S1-D-3z: the pick (0/1 → 331/332) spawns
        // the matching kind; the draw order is unchanged.
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS)
            && rng.next_max(2) == 0
        {
            let pick = rng.next_max(2); // `Next(331, 333)` — the type pick.
            let kind = if pick == 0 {
                NpcKind::ZombieXmas
            } else {
                NpcKind::ZombieSweater
            };
            return Npc::spawn_zombie_variant(ctx.world, x, kind);
        }
        // `:4736-4758`: the armed-expert zombie styles — still constants-off
        // (the armed variants are unmodelled).
        // `:4764-4810`: the zombie style table (7 biome styles) with the
        // 1/3 big/small variant roll — every outcome is the port's zombie.
        let _ = rng.next_max(3);
        let _ = zombie_style;
        return Npc::spawn_zombie(ctx.world, x);
    }
    // ---- the underground ladder (`NPC.cs:4846-4869`) ----
    // The 1/50 worm roll (hardmode splits 95/10) — unmodelled, the roll
    // still draws and the outcome falls to the slime arm.
    if rng.next_max(50) == 0 {
        let _ = if ctx.flags.hard_mode { rng.next_max(3) } else { 0 };
        return spawn_basic_slime(ctx, rng, x, false);
    }
    // `:4852-4858`: hardmode 140/141.
    if ctx.flags.hard_mode && rng.next_max(3) == 0 {
        return spawn_basic_slime(ctx, rng, x, false);
    }
    if ctx.flags.hard_mode && rng.next_max(2) == 0 {
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
    // `savedGoblin`). W4-f32: the roll is the real `Luck.RollLuck` over
    // `ctx.luck` — at luck 0 it degenerates to the same single `Next(20)`.
    if ctx.bound_goblin_ok
        && !underworld_depth
        && (y as f64 / 16.0) > ctx.world.rock_layer
        && (y as f64 / 16.0) < ctx.world.height as f64 - 210.0
        && tr_core::loot::roll_luck(rng, ctx.luck, 20) == 0
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
        && tr_core::loot::roll_luck(rng, ctx.luck, 20) == 0
    {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::BoundWizard);
    }
    // The bound Mechanic (`NPC.cs:2653-2657`'s Dungeon arm): the
    // Skeletron downed, `Main.rand.Next(5) == 0`, one per save.
    if ctx.bound_mech_ok && ctx.below_deep_dungeon && rng.next_max(5) == 0 {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::BoundMechanic);
    }
    // The Devourer: the corruption-cavern roll (`NPC.cs:3701-3710`:
    // `!noWorms && ZoneCorrupt && Next(maxValue) == 0` with the
    // `maxValue = 65` (`:1286`); `CanSpawnDevourer` is the custom-seed
    // gate — true in the port; the hardmode 98/99 split waits for the
    // hardmode worm pass).
    if !no_worms && ctx.flags.zone_evil && rng.next_max(65) == 0 {
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
    if !no_worms && !matches!(tile_kind, 147 | 161) && rng.next_max(50) == 0 {
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
        && rng.next_max(200) == 0
    {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::Tim);
    }
    // The Fire Imp: the underworld arm (`:4891-4893`'s `Next(14) == 0 →
    // 24`); the lava-critter Next(8) and Demon Next(40) arms ahead of it
    // are unmodelled (constants-off, consume nothing), and a missed imp
    // roll falls through to the cavern arms (the vanilla's remaining
    // underworld arms map to the port's closest kinds there).
    if underworld_depth && rng.next_max(14) == 0 {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::FireImp);
    }
    // The Undead Miner: `Next(20) == 0 → 44` (`:5060` region — the rare
    // cave miner).
    if rng.next_max(20) == 0 {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::UndeadMiner);
    }
    // The skeleton family: `switch Next(4)` case 0's `Next(3) != 0 → 21`
    // (`:5133-5135`) — the port folds the big/small variants (-47/-46) out
    // and keeps the base skeleton.
    if rng.next_max(4) == 0 {
        if rng.next_max(3) != 0 {
            return Npc::spawn_cave_walker(ctx.world, x, NpcKind::Skeleton);
        }
        rng.next_max(2); // the big/small variant roll consumed
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::Skeleton);
    }
    // The corruption hover: the Eater of Souls (`NPC.cs`'s corruption arm
    // `Next(6) == 0 → 6` — the port's evil-zone flag gates it).
    if ctx.flags.zone_evil && rng.next_max(6) == 0 {
        if let Some(n) = Npc::spawn_hover(ctx.world, x, y / 16 - 2, NpcKind::EaterOfSouls) {
            return Some(n);
        }
    }
    // The jungle hover: the Hornet (`NPC.cs`'s jungle arm `Next(4) == 0 →
    // 42` — the port's jungle-zone flag gates it).
    if ctx.flags.zone_jungle && rng.next_max(4) == 0 {
        if let Some(n) = Npc::spawn_hover(ctx.world, x, y / 16 - 2, NpcKind::Hornet) {
            return Some(n);
        }
    }
    // The Cave Bat: the cave-air flyer — `NPC.cs:5233`'s `else → 49` arm;
    // the port's cadence `Next(30)` documents the cave-air rarity (the
    // vanilla's bat rolls sit in the cave-air chain's arms).
    if rng.next_max(30) == 0 {
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
pub(crate) fn invasion_spawn_pick(
    ctx: &SpawnContext,
    rng: &mut UnifiedRandom,
) -> Option<NpcKind> {
    // `invasionType == 1`, the goblin army (`:1429-1440`): the hardmode
    // Summoner 1/30 while none alive (471), then the Sorcerer 1/9 (29), the
    // Peon 1/5 (26), the Archer 1/3 (111), the Thief 1/3 (27), else the
    // Warrior (28) — the port's net-id kinds per arm.
    if ctx.invasion_type == 1 {
        if ctx.flags.hard_mode && !ctx.invasion_summoner_alive && rng.next_max(30) == 0 {
            return Some(NpcKind::GoblinSummoner); // 471
        }
        if rng.next_max(9) == 0 {
            return Some(NpcKind::GoblinSorcerer); // 29
        }
        if rng.next_max(5) == 0 {
            return Some(NpcKind::GoblinThief); // 26
        }
        if rng.next_max(3) == 0 {
            return Some(NpcKind::GoblinPeon); // 111
        }
        if rng.next_max(3) == 0 {
            return Some(NpcKind::GoblinWarrior); // 27
        }
        return Some(NpcKind::GoblinArcher); // 28
    }
    // `invasionType == 2`, the frost legion (`:1453-1466`): the Balla
    // 1/7 (145, the port's `MysterySnowman`), the Snowman Gangsta 1/3
    // (143), else the Stabby (144, the port's `SnowBalla`) — the port's
    // net-id kinds per arm.
    if ctx.invasion_type == 2 {
        if rng.next_max(7) == 0 {
            return Some(NpcKind::MysterySnowman); // 145
        }
        if rng.next_max(3) == 0 {
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
            && rng.next_max(20) == 0
        {
            return Some(NpcKind::FlyingDutchman); // 491
        }
        if rng.next_max(30) == 0 && !ctx.invasion_captain_alive {
            return Some(NpcKind::PirateCaptain); // 216
        }
        if rng.next_max(11) == 0 {
            return Some(NpcKind::PirateCorsair); // 215
        }
        if rng.next_max(9) == 0 {
            return Some(NpcKind::PirateCorsair); // 252 Parrot unported
        }
        if rng.next_max(7) == 0 {
            return Some(NpcKind::PirateArcher); // 214
        }
        if rng.next_max(3) == 0 {
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
    let num6 = rng.next_max(7);
    let flag6 = ctx.invasion_progress_30 && !ctx.invasion_saucer_alive;
    if rng.next_max(45) == 0 && flag6 {
        return Some(NpcKind::MartianSaucerCore); // 395
    }
    if num6 >= 6 {
        // The rider arm (`:1500-1513`): the Saucer gate again (`Next(20)`
        // `& flag6`), else `Next(2)` — the Scutlix Rider's net 390 (0) or
        // the Engineer (386).
        if rng.next_max(20) == 0 && flag6 {
            return Some(NpcKind::MartianSaucerCore); // 395
        }
        return if rng.next_max(2) == 0 {
            Some(NpcKind::ScutlixRider) // 390
        } else {
            Some(NpcKind::MartianEngineer) // 386
        };
    }
    if num6 >= 4 {
        // The ray arm (`:1514-1520`): `num8 = Next(5)` — <2 the RayGunner
        // (382), >=4 the Drone (388), else the BrainScrambler (381).
        let num8 = rng.next_max(5);
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
    let mut num9 = rng.next_max(4);
    if num9 == 3 {
        if !ctx.invasion_walker_alive {
            return Some(NpcKind::MartianWalker); // 520
        }
        num9 = rng.next_max(3);
    }
    Some(match num9 {
        0 => NpcKind::GrayGrunt,      // 385
        1 => NpcKind::GigaZapper,     // 389
        _ => NpcKind::MartianOfficer, // 383
    })
}

// ---- S1-F-13: the moon-event roster ladders (`NPC.cs:2804-3548`) ----

/// One arm of a moon roster ladder: draws the arm's roll on the spawner stream
/// and, when it hits and the `CountNPCS(id) < cap` gate is open, records the
/// vanilla `NPC.type`. `cap` is `Some(c)` for the C#'s `CountNPCS(id) < c`
/// (`Some(1)` = `!AnyNPCs(id)`) and `None` for an ungated roll. Returns
/// whether the arm's whole condition held — the C#'s `else if` chain draws the
/// next roll only while the previous arm's did not.
fn moon_arm(
    ctx: &SpawnContext,
    picked: &mut Option<u16>,
    rng: &mut UnifiedRandom,
    n: i32,
    cap: Option<i32>,
    kind: u16,
) -> bool {
    let count = ctx.moon_npc_counts.iter().filter(|&&t| t == kind).count() as i32;
    moon_arm_count(picked, rng, n, cap, kind, count)
}

/// [`moon_arm`] with the live count explicit — the production path reads
/// the ctx's roster scan; the tests close a gate to prove the `CountNPCS`
/// cap actually gates the arm (a closed cap suppresses the pick and the C#'s
/// `else if` chain advances to the next arm).
fn moon_arm_count(
    picked: &mut Option<u16>,
    rng: &mut UnifiedRandom,
    n: i32,
    cap: Option<i32>,
    kind: u16,
    count: i32,
) -> bool {
    let hit = rng.next_max(n) == 0 && cap.map_or(true, |c| count < c);
    if hit && picked.is_none() {
        *picked = Some(kind);
    }
    hit
}

/// A C# `else if` chain in the vanilla arm order: each arm draws its roll while
/// no earlier arm fired (the `else if` short-circuit), and the chain falls
/// through to `fallback` — the C#'s final `else` type, or `None` for the snow
/// wave 14, whose chain has no `else` (`NPC.cs:2958`).
fn moon_chain<F>(
    ctx: &SpawnContext,
    rng: &mut UnifiedRandom,
    arms: &[(i32, Option<i32>, u16)],
    fallback: F,
) -> Option<u16>
where
    F: FnOnce(&mut UnifiedRandom) -> Option<u16>,
{
    let mut picked: Option<u16> = None;
    for (n, cap, kind) in arms {
        if picked.is_some() {
            break;
        }
        moon_arm(ctx, &mut picked, rng, *n, *cap, *kind);
    }
    if picked.is_none() {
        fallback(rng)
    } else {
        picked
    }
}

/// The frost-moon roster ladder: `SpawnAnNPC`'s
/// `!Main.dayTime && Main.snowMoon` arm (`NPC.cs:2804-3222`), transcribed with
/// the vanilla branch order — the 341 head (`:2807`) at every wave, the
/// `waveNumber >= 20` boss trio (`:2812`), the `>= 19 … >= 15` ladder
/// (`:2832`), then the `switch (waveNumber)` cases (`:2956`).
///
/// Returns the `NPC.type` the arm would spawn. `None` is the C#'s only
/// no-spawn outcome: the wave-20 trio behind a reached boss cap (`:2815`).
/// `S1-F-13c` slices A+B port all 15 frost kinds (338-352) - every pick
/// decodes through [`crate::npc::NpcKind::moon_wave_kind`] and spawns.
fn frost_moon_ladder(ctx: &SpawnContext, rng: &mut UnifiedRandom) -> Option<u16> {
    let wave = ctx.world.wave_number;
    // The 341 head (`:2807-2811`) runs at every wave, before the `>= 20`
    // check, and is the block's only arm that returns on a hit.
    if rng.next_max(30) == 0
        && (ctx.moon_npc_counts.iter().filter(|&&t| t == 341).count() as i32) < 4
    {
        return Some(341);
    }
    if wave >= 20 {
        // `num44 = Next(3)` (`:2814`) is drawn BEFORE the boss-cap gate
        // (`:2815`): the stream consumes even when the cap is reached.
        let num44 = rng.next_max(3);
        if moon_boss_cap_reached(ctx.moon_boss_cap_count) {
            return None;
        }
        return Some(match num44 {
            0 => 345,
            1 => 346,
            _ => 344,
        });
    }
    // The `waveNumber >= 19 … >= 15` ladder (`NPC.cs:2832-2954`): the C#'s
    // sequential `if (waveNumber >= N) { … return; }` chain — a wave takes the
    // first branch it reaches, then the switch below.
    if wave >= 19 {
        return moon_chain(
            ctx,
            rng,
            &[(10, Some(4), 345), (10, Some(5), 346), (10, Some(7), 344)],
            |_| Some(343),
        );
    }
    if wave >= 18 {
        return moon_chain(
            ctx,
            rng,
            &[
                (10, Some(3), 345),
                (10, Some(4), 346),
                (10, Some(6), 344),
                (3, None, 348),
                (3, None, 351),
            ],
            |_| Some(343),
        );
    }
    if wave >= 17 {
        return moon_chain(
            ctx,
            rng,
            &[
                (10, Some(2), 345),
                (10, Some(3), 346),
                (10, Some(5), 344),
                (4, None, 347),
                (2, None, 351),
            ],
            |_| Some(343),
        );
    }
    if wave >= 16 {
        return moon_chain(
            ctx,
            rng,
            &[(10, Some(2), 345), (10, Some(2), 346), (10, Some(4), 344), (2, None, 352)],
            |_| Some(343),
        );
    }
    if wave >= 15 {
        return moon_chain(
            ctx,
            rng,
            &[(10, Some(1), 345), (10, Some(2), 346), (10, Some(3), 344), (3, None, 347)],
            |_| Some(343),
        );
    }
    // The `switch (waveNumber)` (`NPC.cs:2956-3222`): the C#'s cases 14 … 2
    // plus the default — wave 1 (and any out-of-range value) takes the default.
    match wave {
        // `NPC.cs:2958` — case 14's `else if` chain has NO final `else`: when
        // all four arms miss, the C# spawns nothing.
        14 => moon_chain(
            ctx,
            rng,
            &[(10, Some(1), 345), (10, Some(1), 346), (10, Some(1), 344), (3, None, 343)],
            |_| None,
        ),
        13 => moon_chain(
            ctx,
            rng,
            &[
                (10, Some(1), 345),
                (10, Some(1), 346),
                (3, None, 352),
                (6, None, 343),
                (3, None, 342),
            ],
            |_| Some(347),
        ),
        12 => moon_chain(
            ctx,
            rng,
            &[(10, Some(1), 345), (10, Some(1), 344), (8, None, 343), (3, None, 342)],
            // `Main.rand.Next(338, 341)` (`:3021`) IS the chosen type.
            |r| Some(338 + r.next_max(3) as u16),
        ),
        11 => moon_chain(ctx, rng, &[(10, Some(1), 345), (6, None, 352), (2, None, 342)], |r| {
            Some(338 + r.next_max(3) as u16)
        }),
        10 => moon_chain(
            ctx,
            rng,
            &[
                (10, Some(1), 346),
                (10, Some(2), 344),
                (6, None, 351),
                (3, None, 348),
                (3, None, 347),
            ],
            |r| Some(338 + r.next_max(3) as u16),
        ),
        9 => moon_chain(
            ctx,
            rng,
            &[(10, Some(1), 346), (10, Some(1), 344), (2, None, 348), (3, None, 347)],
            |_| Some(342),
        ),
        8 => moon_chain(
            ctx,
            rng,
            &[(10, Some(1), 346), (8, None, 351), (3, None, 348), (3, None, 347)],
            |_| Some(350),
        ),
        7 => moon_chain(ctx, rng, &[(10, Some(1), 346), (3, None, 342), (4, None, 350)], |r| {
            Some(338 + r.next_max(3) as u16)
        }),
        6 => moon_chain(ctx, rng, &[(10, Some(2), 344), (4, None, 347), (2, None, 348)], |_| Some(350)),
        5 => moon_chain(ctx, rng, &[(10, Some(1), 344), (4, None, 350), (8, None, 348)], |r| {
            Some(338 + r.next_max(3) as u16)
        }),
        4 => moon_chain(ctx, rng, &[(10, Some(1), 344), (4, None, 350), (3, None, 342)], |r| {
            Some(338 + r.next_max(3) as u16)
        }),
        3 => moon_chain(ctx, rng, &[(8, None, 348), (4, None, 350), (3, None, 342)], |r| {
            Some(338 + r.next_max(3) as u16)
        }),
        2 => moon_chain(ctx, rng, &[(3, None, 350)], |r| Some(338 + r.next_max(3) as u16)),
        _ => moon_chain(ctx, rng, &[(3, None, 342)], |r| Some(338 + r.next_max(3) as u16)),
    }
}

/// The 327-arm quirk of the pumpkin cases 18/17/15/14 (`NPC.cs:3263`/`:3281`/
/// `3329`/`:3351`): the leading 327 `if` is a SEPARATE statement from the
/// `else if` chain after it, so its roll always draws — and when both fire the
/// C# spawns BOTH NPCs in one attempt (the picker returns the first; the stream
/// stays 1:1 — the deviation is recorded in ANALYSIS).
fn pumpkin_327_chain<F>(
    ctx: &SpawnContext,
    rng: &mut UnifiedRandom,
    n: i32,
    cap: Option<i32>,
    chain: &[(i32, Option<i32>, u16)],
    fallback: F,
) -> Option<u16>
where
    F: FnOnce(&mut UnifiedRandom) -> Option<u16>,
{
    let mut picked = None;
    moon_arm(ctx, &mut picked, rng, n, cap, 327);
    picked.or(moon_chain(ctx, rng, chain, fallback))
}

/// The pumpkin-moon roster ladder: `SpawnAnNPC`'s `!Main.dayTime &&
/// Main.pumpkinMoon` arm (`NPC.cs:3224-3548`) — the `waveNumber >= 20` boss
/// trio (`:3227`) and the `switch (waveNumber2)` cases (`:3246`). Same return
/// contract as [`frost_moon_ladder`]. Every pick is one of the 16 Pumpkin Moon
/// wave kinds (305-314/315/325/326/327/329/330), ported since `S1-F-13b` Phase
/// 2 — the moon arm maps it through [`crate::npc::NpcKind::moon_wave_kind`]
/// and spawns it at the found tile.
fn pumpkin_moon_ladder(ctx: &SpawnContext, rng: &mut UnifiedRandom) -> Option<u16> {
    let wave = ctx.world.wave_number;
    if wave >= 20 {
        // The trio sits WHOLLY inside `!reachedInvasionBossCap` (`:3229`): a
        // reached cap draws nothing and spawns nothing.
        if moon_boss_cap_reached(ctx.moon_boss_cap_count) {
            return None;
        }
        let mut picked = None;
        if moon_arm(ctx, &mut picked, rng, 2, Some(2), 327) {
            return picked; // 327
        }
        // `Next(3) != 0 && CountNPCS(325) < 2` (`:3235`) — the inverted roll.
        if rng.next_max(3) != 0
            && (ctx.moon_npc_counts.iter().filter(|&&t| t == 325).count() as i32) < 2
        {
            return Some(325);
        }
        // The 315 arm (`:3239`) draws no roll.
        if (ctx.moon_npc_counts.iter().filter(|&&t| t == 315).count() as i32) < 3 {
            return Some(315);
        }
        return None;
    }
    match wave {
        19 => {
            // `:3249-3260`: 327, 325, then the 315 arm whose gate ALSO carries
            // `!reachedInvasionBossCap`; the case has no final `else`.
            let mut picked = None;
            if !moon_arm(ctx, &mut picked, rng, 5, Some(2), 327)
                && !moon_arm(ctx, &mut picked, rng, 5, Some(2), 325)
                && !moon_boss_cap_reached(ctx.moon_boss_cap_count)
                && (ctx.moon_npc_counts.iter().filter(|&&t| t == 315).count() as i32) < 5
            {
                picked = Some(315);
            }
            picked
        }
        18 => pumpkin_327_chain(ctx, rng, 7, Some(2), &[(7, Some(2), 325), (7, Some(3), 315)], |_| Some(330)),
        17 => pumpkin_327_chain(
            ctx,
            rng,
            7,
            Some(2),
            &[(7, Some(2), 325), (7, Some(2), 315), (3, None, 330)],
            |_| Some(329),
        ),
        16 => moon_chain(
            ctx,
            rng,
            &[(10, Some(2), 327), (10, Some(2), 315), (6, None, 330), (3, None, 329)],
            |_| Some(326),
        ),
        15 => pumpkin_327_chain(
            ctx,
            rng,
            10,
            Some(1),
            &[(7, Some(2), 325), (5, None, 330), (3, None, 326)],
            |r| Some(305 + r.next_max(10) as u16),
        ),
        14 => pumpkin_327_chain(
            ctx,
            rng,
            10,
            Some(1),
            &[
                (7, Some(2), 325),
                (10, Some(1), 315),
                (10, None, 330),
                (7, None, 329),
                (3, None, 326),
            ],
            |r| Some(305 + r.next_max(10) as u16),
        ),
        13 => moon_chain(
            ctx,
            rng,
            &[(7, Some(2), 325), (10, Some(2), 315), (6, None, 330), (3, None, 329)],
            |_| Some(326),
        ),
        12 => moon_chain(ctx, rng, &[(5, Some(1), 327)], |_| Some(330)),
        11 => moon_chain(ctx, rng, &[(7, Some(2), 325), (3, None, 330)], |_| Some(326)),
        10 => moon_chain(ctx, rng, &[(10, Some(1), 327), (3, None, 329)], |r| {
            Some(305 + r.next_max(10) as u16)
        }),
        9 => moon_chain(
            ctx,
            rng,
            &[(10, Some(2), 325), (8, None, 330), (5, None, 329), (2, None, 326)],
            |r| Some(305 + r.next_max(10) as u16),
        ),
        8 => moon_chain(ctx, rng, &[(8, Some(2), 315), (4, None, 330)], |_| Some(329)),
        7 => moon_chain(ctx, rng, &[(7, Some(2), 325), (4, None, 330)], |_| Some(329)),
        6 => moon_chain(ctx, rng, &[(7, Some(2), 325), (2, None, 326)], |r| {
            Some(305 + r.next_max(10) as u16)
        }),
        5 => moon_chain(ctx, rng, &[(10, Some(1), 315)], |_| Some(329)),
        4 => {
            // Verbatim quirk (`NPC.cs:3511-3513`): the roll gates on
            // `!AnyNPCs(325)` but the arm spawns 330 — the cap must count the
            // GATE kind 325, not the spawned 330 (`moon_chain`'s generic
            // count-by-kind would count the wrong family).
            let mut picked = None;
            let gate_count = ctx.moon_npc_counts.iter().filter(|&&t| t == 325).count() as i32;
            moon_arm_count(&mut picked, rng, 8, Some(1), 330, gate_count);
            if picked.is_some() {
                return picked;
            }
            moon_chain(ctx, rng, &[(2, None, 326)], |r| Some(305 + r.next_max(10) as u16))
        }
        3 => moon_chain(ctx, rng, &[(3, None, 329)], |_| Some(326)),
        2 => moon_chain(ctx, rng, &[(3, None, 326)], |r| Some(305 + r.next_max(10) as u16)),
        // `:3545` — the C#'s default: no roll gate, the type is `Next(305, 315)`.
        _ => Some(305 + rng.next_max(10) as u16),
    }
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

/// W4-c3 gaps 2+3 — the dungeon band (`Spawner.SpawnAnNPC`'s
/// `else if (this.ZoneDungeon)` arm, `NPC.cs:2626-2793`). The wall-colour
/// pick (`num40`), the pre-Skeletron Guardian, the bound Mechanic, the
/// hardmode roster (the 289 Giant Cursed Skull included), the 71/70/72/34/32
/// rolls and the 694 Water Bolt Mimic's book-find. The unported kinds
/// (287 BoneLee, 293/291/292/290 the hardmode skeletons, 281-286 the
/// ragged/necro/diabolist casters, 269-280 the armored bones, 693 the
/// Librarian Skeleton, 294-296 the big angry bones, the -13/-14/31 angry
/// bones) consume their rolls and spawn nothing — the port's
/// consume-and-return-None shape (the Wyvern arm's precedent). The 70
/// SpikeBall's `NearSpikeBall` uniqueness scan and the 72 BlazingWheel's
/// `AnyNPCs` are unmodelled (the port's spawn path does not carry them);
/// the rolls still draw. `RollLuck` runs as the plain `Next(n)` (the
/// spawner's documented luck fold).
fn spawn_dungeon_band(ctx: &SpawnContext, rng: &mut UnifiedRandom, x: i32, y: i32) -> Option<Npc> {
    // `:2628-2642`: the wall-colour pick. The slab walls (94/96/98) set
    // num40 = 1, the tile walls (95/97/99) set num40 = 2 (the tile check
    // runs second and wins a mixed pair); the 1/7 RollLuck re-rolls it.
    let wall = ctx.world.tile(x, y).map(|t| t.wall).unwrap_or(0);
    let wall2 = ctx.world.tile(x, y - 1).map(|t| t.wall).unwrap_or(0);
    let mut num40 = 0;
    if matches!(wall, 94 | 96 | 98) || matches!(wall2, 94 | 96 | 98) {
        num40 = 1;
    }
    if matches!(wall, 95 | 97 | 99) || matches!(wall2, 95 | 97 | 99) {
        num40 = 2;
    }
    if rng.next_max(7) == 0 {
        num40 = rng.next_max(3);
    }
    // `:2643-2652`: the pre-Skeletron Guardian (68). The port has no
    // DungeonGuardian kind — the arm consumes nothing (the flag gate
    // short-circuits before any roll) and returns None.
    if !ctx.flags.downed_boss3 {
        return None;
    }
    // `:2653-2657`: the bound Mechanic (123). The port's `bound_mech_ok`
    // carries the downedBoss3 + !savedMech + !AnyNPCs(123) guards; the
    // `!waterTile` gate is the band's dry-tile precondition (the water
    // ladder above already returned on a wet tile), and the depth band
    // rides `below_deep_dungeon`.
    if ctx.bound_mech_ok && ctx.below_deep_dungeon && rng.next_max(5) == 0 {
        return Npc::spawn_cave_walker(ctx.world, x, NpcKind::BoundMechanic);
    }
    // `:2658-2719`: the hardmode roster (`hardDungeon = downedPlantBoss &&
    // hardMode`, `:380`). Every kind here is unported except the 289 — the
    // rolls draw in the C#'s order and the unported outcomes spawn nothing.
    if ctx.flags.hard_dungeon {
        if rng.next_max(30) == 0 {
            return None; // 287 BoneLee
        }
        if num40 == 0 && rng.next_max(15) == 0 {
            return None; // 293 SkeletonCommando
        }
        if num40 == 1 && rng.next_max(15) == 0 {
            return None; // 291 SkeletonSniper
        }
        if num40 == 2 && rng.next_max(15) == 0 {
            return None; // 292 TacticalSkeleton
        }
        if num40 == 0 && rng.next_max(35) == 0 {
            return None; // 290 Paladin (the AnyNPCs gate is unmodelled)
        }
        // `:2683-2687`: the 289 Giant Cursed Skull — the slab/tile walls
        // (num40 1/2), the 1/30 roll. Plantera-gated via hard_dungeon.
        if (num40 == 1 || num40 == 2) && rng.next_max(30) == 0 {
            return Npc::spawn_hover(ctx.world, x, y, NpcKind::GiantCursedSkull);
        }
        if rng.next_max(20) == 0 {
            let _ = rng.next_max(2); // the 281-286 caster pick
            return None;
        }
        if rng.next_max(3) != 0 {
            let _ = rng.next_max(4); // the 269-280 armored-bones pick
            return None;
        }
    }
    // `:2720-2724`: the 71 Dungeon Slime (1/35).
    if rng.next_max(35) == 0 {
        return Npc::spawn_caster(ctx.world, x, y, NpcKind::DungeonSlime);
    }
    // `:2725-2729`: the 70 SpikeBall (num40 1, 1/3) — unported, the roll
    // draws (the NearSpikeBall scan is unmodelled).
    if num40 == 1 && rng.next_max(3) == 0 {
        return None;
    }
    // `:2730-2734`: the 72 BlazingWheel (num40 2, 1/5).
    if num40 == 2 && rng.next_max(5) == 0 {
        return Npc::spawn_hover(ctx.world, x, y, NpcKind::BlazingWheel);
    }
    // `:2735-2739`: the 34 Cursed Skull (num40 0, 1/7).
    if num40 == 0 && rng.next_max(7) == 0 {
        return Npc::spawn_hover(ctx.world, x, y, NpcKind::CursedSkull);
    }
    // `:2740-2744`: the 32 Dark Caster (1/7).
    if rng.next_max(7) == 0 {
        return Npc::spawn_caster(ctx.world, x, y, NpcKind::DarkCaster);
    }
    // `:2745-2763`: the book-find. The 1/8 roll spawns the 694 Water Bolt
    // Mimic at the closest book (ai3 = 3, the closed book); the else 1/10
    // roll spawns the 693 Librarian Skeleton (unported). The num43 roll
    // (`:2764`) draws AFTER the book-find regardless.
    let mut flag13 = false;
    if rng.next_max(8) == 0 {
        if let Some((bx, by)) = find_nearby_book(ctx, x, y) {
            let mut n = Npc::new(
                NpcKind::WaterBoltMimic,
                bx as f32 * TILE_PX + 8.0 - 22.0,
                by as f32 * TILE_PX - 44.0,
                1,
                0.0,
            );
            n.hp = n.max_hp;
            n.ai[3] = 3.0; // the closed book (`:2751`'s ai3 arg)
            return Some(n);
        }
    } else if rng.next_max(10) == 0 {
        if find_nearby_book(ctx, x, y).is_some() {
            flag13 = true; // the 693 Librarian Skeleton (unported)
        }
    }
    let num43 = rng.next_max(5);
    if flag13 {
        return None;
    }
    // `:2769-2780`: the 294/295/296 big angry bones (unported).
    if num43 < 3 {
        return None;
    }
    // `:2781-2792`: the -14/-13/31 angry-bones tail (unported).
    if rng.next_max(4) == 0 {
        return None;
    }
    if rng.next_max(5) == 0 {
        return None;
    }
    None
}

/// `NPC.AI_FindNearbyBook` (`NPC.cs:62308-62364`) with `closestBook: true,
/// checkPlayerScreenRanges: true` — the 32x32 scan centred 16 tiles up-left
/// of the spawn tile, the closest active book tile (type 50) that passes
/// the on-screen check. The C#'s `closestBook` path keeps a running minimum
/// (no `_nearbyBooks` array write, no `Main.rand` draw); the distance is
/// `Vector2.Distance` from the scan's top-left corner. Returns the book's
/// tile coordinates, or None when no book qualifies.
fn find_nearby_book(ctx: &SpawnContext, x: i32, y: i32) -> Option<(i32, i32)> {
    let sx = x - 16;
    let sy = y - 16;
    let x0 = sx.max(0);
    let x1 = (sx + 32).min(ctx.world.width);
    let y0 = sy.max(0);
    let y1 = (sy + 32).min(ctx.world.height);
    let mut best: Option<(i32, i32)> = None;
    let mut best_d = f32::MAX;
    for ty in y0..y1 {
        for tx in x0..x1 {
            let Some(t) = ctx.world.tile(tx, ty) else {
                continue;
            };
            if !t.active() || t.kind != 50 {
                continue;
            }
            // `checkPlayerScreenRanges`: the book tile must pass the
            // on-screen check (the same rect the spawn scan uses).
            if !check_not_spawning_on_screen(ctx, &SpawnArea { left: 0, right: 0, top: 0, bottom: 0 }, tx, ty) {
                continue;
            }
            let dx = (tx - sx) as f32;
            let dy = (ty - sy) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            if d < best_d {
                best_d = d;
                best = Some((tx, ty));
            }
        }
    }
    // `:62349-62357`: the closestBook return — `bookPosition = vector`,
    // then `if (vector.X == vector2.X) return vector.Y != vector2.Y;
    // return true`. A book exactly at the scan origin (or no book at
    // all, `vector` never moved) compares equal to the search position
    // and returns false.
    best.filter(|&(bx, by)| (bx, by) != (sx, sy))
}

fn spawn_basic_slime(
    ctx: &SpawnContext,
    rng: &mut UnifiedRandom,
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
            && rng.next_max(3) != 0
        {
            return Npc::spawn_slime_variant(ctx.world, x, SlimeVariant::Blue);
        }
        if ctx
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS)
            && rng.next_max(3) != 0
        {
            return Npc::spawn_slime_variant(ctx.world, x, SlimeVariant::Blue);
        }
    }
    let dist = (x - ctx.world.spawn_x).abs();
    let variant = if surface {
        if rng.next_max(3) == 0 || dist < 200 {
            SlimeVariant::Green
        } else if rng.next_max(10) == 0 && dist > 400 {
            SlimeVariant::Purple
        } else {
            SlimeVariant::Blue
        }
    } else if rng.next_max(5) == 0 {
        SlimeVariant::Yellow
    } else if rng.next_max(2) == 0 {
        SlimeVariant::Blue
    } else {
        SlimeVariant::Red
    };
    // `Spawner.SpawnNPC`'s Blue-Slime promotion (`NPC.cs:5240-5249`):
    // the outer gate `FromNetId(Type) == 1` is evaluated ONCE on the
    // original pick — `RollLuck(180) == 0` → -4 (Pinky), THEN the
    // anniversary `tenthAnniversaryWorld && RollLuck(180) == 0` → 667
    // (the Golden Slime — unported, the hit consumes the roll and keeps
    // the pick, the spawner's unmodelled-outcome pattern). The second
    // roll runs even when the first promoted to Pinky (the gate read the
    // ORIGINAL type). Both rolls are the real `Luck.RollLuck` over
    // `ctx.luck` (W4-f32).
    let was_blue = variant == SlimeVariant::Blue;
    let variant = if was_blue && tr_core::loot::roll_luck(rng, ctx.luck, 180) == 0 {
        SlimeVariant::Pinky
    } else {
        variant
    };
    if was_blue
        && ctx
            .world
            .seeds
            .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY)
        && tr_core::loot::roll_luck(rng, ctx.luck, 180) == 0
    {
        // 667 has no SlimeVariant — the roll is consumed, the pick stands
        // (the C# spawns the Golden Slime; the kind lands with its wave).
    }
    Npc::spawn_slime_variant(ctx.world, x, variant)
}

/// W4-f42 — `Spawner.Spawning_SandstoneCheck` (`NPC.cs:5452-5491`): the
/// 8-row × 9-column sand scan below the spawn tile — each row counts the
/// centre tile plus up to 4 left + 4 right contiguous Conversion.Sand
/// tiles; the total must reach 40 (the remix 10 is unported). The scan
/// breaks a row's run at the first non-sand tile.
fn spawning_sandstone_check(world: &tr_core::World, x: i32, y: i32) -> bool {
    // `WorldGen.InWorld(x, y, 10)` — the 10-tile margin.
    if x < 10 || y < 10 || x >= world.width - 10 || y >= world.height - 10 {
        return false;
    }
    let is_sand = |tx: i32, ty: i32| {
        world.tile(tx, ty).is_some_and(|t| {
            t.active() && matches!(t.kind, 53 | 112 | 116 | 234)
        })
    };
    let mut num = 0;
    for i in 0..8 {
        if !is_sand(x, y + i) {
            break;
        }
        num += 1;
        for j in 1..=4 {
            if !is_sand(x + j, y + i) {
                break;
            }
            num += 1;
        }
        for k in 1..=4 {
            if !is_sand(x - k, y + i) {
                break;
            }
            num += 1;
        }
    }
    num >= 40
}

/// W4-f42 — the sandstorm arm's ground spawn (`SpawnNPC(spawnTileX * 16 + 8,
/// spawnTileY * 16, kind)` → `NewNPC`'s `Bottom = (X, Y)`, `NPC.cs:80772`):
/// feet on the found tile's top edge, X centred 8 px into the tile, facing
/// toward the player (the invaders arm's placement).
fn spawn_desert_ground(ctx: &SpawnContext, x: i32, y: i32, kind: NpcKind) -> Option<Npc> {
    let (nw, nh) = kind.size();
    let mut n = Npc::new(
        kind,
        x as f32 * TILE_PX + 8.0 - nw * 0.5,
        y as f32 - nh,
        if ctx.player_center.0 >= x as f32 * TILE_PX + 8.0 { 1 } else { -1 },
        0.0,
    );
    n.hp = n.max_hp;
    Some(n)
}

/// W4-f42 — the sandstorm arm's flyer spawn: the same Bottom placement (the
/// C#'s `SpawnNPC` for the Vulture/FlyingAntlion uses the same call shape;
/// the flyer's own AI lifts it on the first tick).
fn spawn_desert_flyer(ctx: &SpawnContext, x: i32, y: i32, kind: NpcKind) -> Option<Npc> {
    spawn_desert_ground(ctx, x, y, kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-f42 — `Spawner.Spawning_SandstoneCheck` (`NPC.cs:5452-5491`): the
    /// 8×9 sand scan counts ≥ 40 (5 full rows) — 4 rows (36) fail, and a
    /// non-sand centre breaks the row run early.
    #[test]
    fn sandstone_check_needs_forty_sand() {
        use tr_core::tile::{Tile, FLAG_ACTIVE};
        let mut w = World::new("t".into(), 200, 200);
        let sand = |w: &mut World, x: i32, y: i32| {
            let i = w.index(x, y);
            w.tiles[i] = Tile { kind: 53, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        };
        // The InWorld(x, y, 10) margin refuses the edge.
        assert!(!spawning_sandstone_check(&w, 5, 50));
        // 4 full rows = 36 < 40.
        for y in 50..54 {
            for x in 26..=34 {
                sand(&mut w, x, y);
            }
        }
        assert!(!spawning_sandstone_check(&w, 30, 50));
        // The 5th row reaches 45 ≥ 40.
        for x in 26..=34 {
            sand(&mut w, x, 54);
        }
        assert!(spawning_sandstone_check(&w, 30, 50));
        // A non-sand centre on row 0 breaks the whole scan.
        let i = w.index(30, 50);
        w.tiles[i] = Tile { kind: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!spawning_sandstone_check(&w, 30, 50));
        // The conversion family (112/116/234) counts as sand too.
        let i = w.index(30, 50);
        w.tiles[i] = Tile { kind: 234, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(spawning_sandstone_check(&w, 30, 50));
    }

    /// W4-f42 — the sandstorm arm's pre-hardmode ladder (`NPC.cs:3951-3964`):
    /// the 546/61/69 picks over the found sand tile, the Bottom placement,
    /// and the event gate (no sandstorm → no desert pick).
    #[test]
    fn sandstorm_pre_hardmode_picks_the_desert_trio() {
        use tr_core::tile::{Tile, FLAG_ACTIVE};
        let mut w = World::new("t".into(), 400, 700);
        for x in 0..400 {
            for y in 300..700 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 53, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 310.0;
        w.rock_layer = 320.0;
        w.compute_column_surface();
        let mut c = ctx(&w, 290.0 * 16.0, true);
        c.sandstorm_happening = true;
        c.flags.zone_sandstorm = true;
        let mut seen = [false; 3];
        for seed in 0..300u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let Some(n) = spawn_npc_at(&c, &mut rng, 210, 300 * 16, false, false, false) else {
                continue;
            };
            let idx = match n.kind {
                NpcKind::Tumbleweed => 0,
                NpcKind::Vulture => 1,
                NpcKind::Antlion => 2,
                other => panic!("seed {seed}: the pre-hardmode arm picked {other:?}"),
            };
            seen[idx] = true;
            // `SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, kind)` →
            // `NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80772`).
            let (nw, nh) = n.kind.size();
            assert_eq!(n.x, 210.0 * TILE_PX + 8.0 - nw * 0.5);
            assert_eq!(n.y, 300.0 * TILE_PX - nh);
            assert_eq!(n.hp, n.max_hp);
        }
        assert!(seen.iter().all(|&s| s), "all three pre-hardmode kinds spawn");
        // The event gate: no sandstorm → the arm never fires (the day
        // ladder's slime fallthrough takes over).
        c.sandstorm_happening = false;
        for seed in 0..50u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 300 * 16, false, false, false) {
                assert!(
                    !matches!(n.kind, NpcKind::Tumbleweed | NpcKind::Vulture | NpcKind::Antlion),
                    "seed {seed}: the desert arm fired without the event"
                );
            }
        }
    }

    /// W4-f42 — the sandstorm arm's hardmode ladder (`NPC.cs:3966-4018`):
    /// the mummy family rides the sand-type rows (53 → 78, 112 → 79,
    /// 234 → 630, 116 → 80) and the fallthrough picks 546/580/581; the
    /// unported 541/510/542-545 arms consume their rolls and spawn nothing.
    #[test]
    fn sandstorm_hardmode_picks_the_mummies_and_antlions() {
        use tr_core::tile::{Tile, FLAG_ACTIVE};
        let mut w = World::new("t".into(), 400, 700);
        for x in 0..400 {
            for y in 300..700 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 53, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 310.0;
        w.rock_layer = 320.0;
        w.compute_column_surface();
        let mut c = ctx(&w, 290.0 * 16.0, true);
        c.sandstorm_happening = true;
        c.flags.zone_sandstorm = true;
        c.flags.hard_mode = true;
        let mut kinds = std::collections::HashSet::new();
        for seed in 0..600u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 300 * 16, false, false, false) {
                assert!(
                    matches!(
                        n.kind,
                        NpcKind::Mummy
                            | NpcKind::Tumbleweed
                            | NpcKind::WalkingAntlion
                            | NpcKind::FlyingAntlion
                    ),
                    "seed {seed}: unexpected hardmode pick {:?}",
                    n.kind
                );
                kinds.insert(n.kind.to_u16());
            }
        }
        assert!(kinds.contains(&NpcKind::Mummy.to_u16()), "the 53 row spawns the Mummy");
        assert!(kinds.contains(&NpcKind::Tumbleweed.to_u16()));
        assert!(kinds.contains(&NpcKind::WalkingAntlion.to_u16()));
        assert!(kinds.contains(&NpcKind::FlyingAntlion.to_u16()));
        // The 112 row spawns the DarkMummy.
        for x in 206..=214 {
            for y in 300..308 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 112, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut c = ctx(&w, 290.0 * 16.0, true);
        c.sandstorm_happening = true;
        c.flags.zone_sandstorm = true;
        c.flags.hard_mode = true;
        let mut dark = false;
        for seed in 0..600u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 300 * 16, false, false, false) {
                if n.kind == NpcKind::DarkMummy {
                    dark = true;
                    break;
                }
            }
        }
        assert!(dark, "the 112 row spawns the DarkMummy");
    }

    /// W4-f42 — the post-sandstorm mummy arms (`NPC.cs:4020-4035`): with
    /// NO event (`Sandstorm.Happening` false) the hardmode conversion-sand
    /// rows still spawn their mummy — the 53 row on `Next(3)`, the
    /// 112/234/116 rows on `Next(2)`. The witness replays the exact draw
    /// order (`GetZombieSettings`' `Next(7)` first, `NPC.cs:1284`, then
    /// the arm's own roll — the tile-mismatched arms above draw nothing)
    /// and pins the spawn iff the replayed roll hits.
    #[test]
    fn post_sandstorm_mummy_arms_fire_without_the_event() {
        use tr_core::tile::{Tile, FLAG_ACTIVE};
        for (tile, kind, denom) in [
            (53u16, NpcKind::Mummy, 3),
            (112, NpcKind::DarkMummy, 2),
            (234, NpcKind::BloodMummy, 2),
            (116, NpcKind::LightMummy, 2),
        ] {
            let mut w = World::new("t".into(), 400, 700);
            for x in 0..400 {
                for y in 300..700 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile { kind: tile, ..Default::default() };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            w.world_surface = 310.0;
            w.rock_layer = 320.0;
            w.compute_column_surface();
            let mut c = ctx(&w, 290.0 * 16.0, true);
            c.flags.hard_mode = true;
            // The event gates stay OFF — the arms are the outside-event
            // desert spawns (`:4020`'s chain position after `:3949`).
            assert!(!c.sandstorm_happening && !c.flags.zone_sandstorm);
            let mut hits = 0;
            for seed in 0..400u64 {
                let mut replay = UnifiedRandom::new(seed as i32);
                let _ = replay.next_max(7); // the zombie-style roll
                let expect = replay.next_max(denom) == 0;
                let mut rng = UnifiedRandom::new(seed as i32);
                let got = spawn_npc_at(&c, &mut rng, 210, 300 * 16, false, false, false);
                if expect {
                    let n = got.unwrap_or_else(|| panic!("tile {tile} seed {seed}: the arm's roll hit but nothing spawned"));
                    assert_eq!(n.kind, kind, "tile {tile} seed {seed}");
                    let (nw, nh) = kind.size();
                    assert_eq!(n.x, 210.0 * TILE_PX + 8.0 - nw * 0.5);
                    assert_eq!(n.y, 300.0 * TILE_PX - nh);
                    hits += 1;
                } else if let Some(n) = got {
                    assert_ne!(n.kind, kind, "tile {tile} seed {seed}: spawned on a missed roll");
                }
            }
            assert!(hits > 0, "tile {tile}: no mummy across the sweep");
            // The hardmode gate: pre-hardmode the arms never fire.
            c.flags.hard_mode = false;
            for seed in 0..100u64 {
                let mut rng = UnifiedRandom::new(seed as i32);
                if let Some(n) = spawn_npc_at(&c, &mut rng, 210, 300 * 16, false, false, false) {
                    assert_ne!(n.kind, kind, "tile {tile} seed {seed}: pre-hardmode mummy");
                }
            }
        }
    }

    fn ctx(world: &World, y_px: f32, day: bool) -> SpawnContext<'_> {
        SpawnContext {
            moon_npc_counts: &[],
            world,
            player_center: (0.0, y_px),
            player_pos: (0.0, y_px),
            // 100 nearby NPCs clears every nearby-ramp threshold
            // (NPC.cs:673-690), isolating these tests from the ramp.
            nearby_active_npcs: 100.0,
            town_npcs: 0,
            day_time: day,
            player_life_max: 100,
            luck: 0.0,
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
            moon_boss_cap_count: 0.0,
            downed_boss1: false,
            sandstorm_happening: false,
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
        let r = get_spawn_rate(&ctx(&w, 25.0 * 16.0, true), &mut UnifiedRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (600, 5));
    }

    #[test]
    fn night_surface_multiplies() {
        // NPC.cs:540-544: rate x0.6 -> 360, max x1.3 -> (float)6.5 -> 6.
        let w = ground();
        let r = get_spawn_rate(&ctx(&w, 25.0 * 16.0, false), &mut UnifiedRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (360, 6));
    }

    #[test]
    fn deep_rock_layer_multiplies() {
        // NPC.cs:487-491: rate x0.4 -> 240, max x1.9 -> 9.5 -> 9. The position
        // must clear rock_layer*16 + sHeight (640 + 1200 = 1840 px = tile 115).
        let w = ground();
        let r = get_spawn_rate(&ctx(&w, 120.0 * 16.0, true), &mut UnifiedRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (240, 9));
    }

    #[test]
    fn the_nearby_ramp_speeds_up_empty_areas() {
        // NPC.cs:673-677: nearby < max x 0.2 multiplies the rate by 0.6 —
        // smaller = more frequent spawns. Day surface: 600 x 0.6 = 360.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.nearby_active_npcs = 0.0;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
    }

    #[test]
    fn blood_moon_night_compounds() {
        // NPC.cs:545-548: night x0.6/x1.3 then blood x0.3/x1.8:
        // 600 -> 360 -> 108; 6.5 -> 11.7 -> 11.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.flags.blood_moon = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
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
            let mut ref_rng = UnifiedRandom::new(1);
            let (want_rate, want_max) = match town {
                0 => (96, 13),
                1 => {
                    let d = ref_rng.next_max(3); // `:869`
                    if d == 1 {
                        (jrate, (jmax as f32 as f64 * 0.6) as i32)
                    } else {
                        (jrate * 2, jmax) // `:876` — `(float)spawnRate * 2f`
                    }
                }
                _ => {
                    let d = ref_rng.next_max(3); // `:892`
                    if d != 0 {
                        (jrate, (jmax as f32 as f64 * 0.6) as i32)
                    } else {
                        (jrate * 3, jmax) // `:899`
                    }
                }
            };
            let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
            assert_eq!(r.spawn_rate, want_rate, "jungle town {town} rate");
            assert_eq!(r.max_spawns, want_max, "jungle town {town} max");
        }
    }

    // ---- S1-F-11: the town-NPC distribution block (`NPC.cs:799-923`) ----

    /// The stream fingerprint after `run` — equal to a fresh stream's draws
    /// iff `run` consumed nothing (the `Next(n)` widths differ, but any
    /// consumption shifts the whole tail).
    fn post_draws(seed: u64, run: impl FnOnce(&mut UnifiedRandom)) -> Vec<i32> {
        let mut rng = UnifiedRandom::new(seed as i32);
        run(&mut rng);
        (0..8).map(|_| rng.next_max(1000)).collect()
    }

    fn fresh_draws(seed: u64) -> Vec<i32> {
        let mut rng = UnifiedRandom::new(seed as i32);
        (0..8).map(|_| rng.next_max(1000)).collect()
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
            let mut ref_rng = UnifiedRandom::new(42);
            let (d1, d2, want_nw, want_fr, want_rate, want_max) = match town {
                1 => {
                    let a = ref_rng.next_max(2); // `:807`
                    let b = ref_rng.next_max(10); // `:811`
                    (a, b, a == 0, b == 0, 750, 5)
                }
                2 => {
                    let a = ref_rng.next_max(4); // `:824`
                    let b = ref_rng.next_max(5); // `:828`
                    (a, b, a != 0, b == 0, 900, 5)
                }
                _ => {
                    let a = ref_rng.next_max(10); // `:840`
                    let b = ref_rng.next_max(3); // `:844`
                    (a, b, a != 0, b == 0, 1200, 5)
                }
            };
            let r = get_spawn_rate(&c, &mut UnifiedRandom::new(42));
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
            let mut ref_rng = UnifiedRandom::new(42);
            let (want_fr, want_rate, want_max) = match town {
                1 => {
                    let d = ref_rng.next_max(3); // `:869`
                    (
                        d == 1,
                        if d == 1 { 600 } else { 1200 },
                        if d == 1 { 3 } else { 5 },
                    )
                }
                2 => {
                    let d = ref_rng.next_max(3); // `:892`
                    (
                        d != 0,
                        if d != 0 { 600 } else { 1800 },
                        if d != 0 { 3 } else { 5 },
                    )
                }
                _ => (true, 600, 3),
            };
            let r = get_spawn_rate(&c, &mut UnifiedRandom::new(42));
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
            let mut ref_rng = UnifiedRandom::new(42);
            let (want_fr, want_max) = match town {
                1 => {
                    let e = ref_rng.next_max(9); // `:863`
                    (e == 1, if e == 1 { 3 } else { 5 })
                }
                2 => {
                    let e = ref_rng.next_max(6); // `:886`
                    (e == 1, if e == 1 { 3 } else { 5 })
                }
                _ => {
                    let e = ref_rng.next_max(3); // `:908`
                    (e == 1, if e == 1 { 3 } else { 5 })
                }
            };
            let r = get_spawn_rate(&c, &mut UnifiedRandom::new(42));
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
        let mut ref_rng = UnifiedRandom::new(42);
        let g = ref_rng.next_max(3); // the gate roll (`:860`)
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(42));
        // The peace-candle modifier already rescaled the rate to 780 and the
        // cap to 3 (`:738-739`, the candle block ahead of the town block).
        if g == 0 {
            let e = ref_rng.next_max(9); // `:863`
            assert_eq!(r.spawn_rate, 1294, "the graveyard rescale (780 × 1.66)");
            assert_eq!(r.spawn_friendly, e == 1, "the Next(9) friendly roll");
        } else {
            let d = ref_rng.next_max(3); // the else-if roll (`:869`) — a second draw
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
            let mut rng = UnifiedRandom::new(seed as i32);
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
                &mut UnifiedRandom::new(seed as i32),
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
            let mut rng = UnifiedRandom::new(seed as i32);
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
            &mut UnifiedRandom::new(s as i32),
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
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 10);
        // Post-Skeletron the override stops and the x0.3 branch stands.
        let mut c = ctx(&w, 25.0 * 16.0, true);
        c.flags.zone_dungeon = true;
        c.flags.downed_boss3 = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 180);
    }

    /// W4-c3 gaps 2+3 — the dungeon band's spawn witnesses. The band is a
    /// pure roll ladder over the wall-colour pick; these pin the three
    /// style-10 kinds' arms (the 289 hardmode roll, the 34 1/7 roll, the
    /// 694 book-find) and the Plantera gate. WRITTEN, NOT RUN (code-only).
    /// A dungeon world: a solid blue-brick floor (tile 41) at row 340, the
    /// slab wall 94 behind the spawn column, the player far off-screen.
    fn dungeon_world() -> World {
        let mut w = World::new("t".into(), 400, 700);
        for x in 0..400 {
            for y in 340..700 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 41,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 310.0;
        w.rock_layer = 320.0;
        w.compute_column_surface();
        // The slab wall behind the spawn column (num40 = 1).
        for y in 300..340 {
            let i = w.index(210, y);
            w.tiles[i].wall = 94;
        }
        w
    }

    fn dungeon_ctx(w: &World, hard: bool) -> SpawnContext<'_> {
        let mut c = ctx(w, 330.0 * 16.0, false);
        c.flags.zone_dungeon = true;
        c.flags.downed_boss3 = true;
        c.flags.hard_dungeon = hard;
        c.bound_mech_ok = false;
        c
    }

    #[test]
    fn w4_c3_the_giant_skull_needs_hard_dungeon_and_a_slab_wall() {
        // `NPC.cs:2683-2687`: hardDungeon && (num40 1/2) && Next(30) == 0.
        // Without hard_dungeon the 289 never comes out; with it, a sweep
        // finds it. The wall is the slab 94 (num40 = 1).
        let w = dungeon_world();
        let mut saw = false;
        for seed in 1..4000u64 {
            let c = dungeon_ctx(&w, true);
            if let Some(n) = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false) {
                if n.kind == NpcKind::GiantCursedSkull {
                    saw = true;
                    break;
                }
            }
        }
        assert!(saw, "the hardmode dungeon sweep never rolled the 289");
        // The Plantera gate: hard_dungeon off -> no 289 however many rolls.
        let mut saw_off = false;
        for seed in 1..4000u64 {
            let c = dungeon_ctx(&w, false);
            if let Some(n) = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false) {
                if n.kind == NpcKind::GiantCursedSkull {
                    saw_off = true;
                    break;
                }
            }
        }
        assert!(!saw_off, "no hard_dungeon, no 289 (the Plantera gate)");
    }

    #[test]
    fn w4_c3_the_cursed_skull_needs_the_default_wall() {
        // `NPC.cs:2735-2739`: num40 == 0 && Next(7) == 0. The default wall
        // (no slab/tile) keeps num40 = 0 unless the 1/7 re-roll hits.
        let mut w = dungeon_world();
        for y in 300..340 {
            let i = w.index(210, y);
            w.tiles[i].wall = 0; // the default wall -> num40 = 0
        }
        let mut saw = false;
        for seed in 1..4000u64 {
            let c = dungeon_ctx(&w, false);
            if let Some(n) = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false) {
                if n.kind == NpcKind::CursedSkull {
                    saw = true;
                    break;
                }
            }
        }
        assert!(saw, "the default-wall dungeon sweep never rolled the 34");
    }

    #[test]
    fn w4_c3_the_water_bolt_mimic_spawns_at_the_closest_book_closed() {
        // `NPC.cs:2745-2753`: Next(8) == 0 -> AI_FindNearbyBook (closest,
        // on-screen-gated) -> SpawnNPC(bookX*16+8, bookY*16, 694, ai3=3).
        // A book tile (50) in the scan rect, off-screen; the mimic lands
        // at the book with ai[3] = 3 (the closed book).
        let mut w = dungeon_world();
        // The book at (215, 330): inside the 32x32 scan centred (194, 324),
        // off-screen (the player is at (0, 330*16), the book at x=215 is
        // far right). The scan's closest-book pick takes it.
        let i = w.index(215, 330);
        w.tiles[i] = Tile { kind: 50, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut found = None;
        for seed in 1..6000u64 {
            let mut c = dungeon_ctx(&w, false);
            // The player far left so the book at x=215 passes the on-screen
            // check (the spawn-screen rect is centred on the player).
            c.player_center = (0.0, 330.0 * 16.0);
            c.player_pos = (0.0, 330.0 * 16.0);
            if let Some(n) = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false) {
                if n.kind == NpcKind::WaterBoltMimic {
                    found = Some(n);
                    break;
                }
            }
        }
        let n = found.expect("the book-find sweep never rolled the 694");
        assert_eq!(n.ai[3], 3.0, "the closed-book spawn arg (:2751)");
        // Bottom = (bookX*16+8, bookY*16): x = 215*16+8 - 44/2, y = 330*16 - 44.
        assert_eq!(n.x, 215.0 * 16.0 + 8.0 - 22.0);
        assert_eq!(n.y, 330.0 * 16.0 - 44.0);
    }

    #[test]
    fn w4_c3_the_dungeon_band_is_terminal_and_deterministic() {
        // The band returns inside every arm (the C#'s return-per-arm shape);
        // the same seed replays the same outcome. The pre-Skeletron Guardian
        // arm consumes nothing and returns None.
        let w = dungeon_world();
        for seed in 1..200u64 {
            let c = dungeon_ctx(&w, true);
            let a = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false);
            let b = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false);
            assert_eq!(a.map(|n| (n.kind, n.x.to_bits(), n.ai[3])), b.map(|n| (n.kind, n.x.to_bits(), n.ai[3])), "seed {seed}");
        }
        // Pre-Skeletron: the Guardian arm (68, unported) returns None.
        let mut c = dungeon_ctx(&w, false);
        c.flags.downed_boss3 = false;
        for seed in 1..200u64 {
            assert!(spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 340 * 16, false, false, false).is_none(), "the Guardian arm spawns nothing (seed {seed})");
        }
    }

    #[test]
    fn the_rate_floor_and_cap_hold() {
        // NPC.cs:741-751: rate >= 60 (600x0.1), max <= 15 (5x3). The deepest
        // vanilla-compoundable stack here: underground desert x0.2/x3 capped.
        let w = ground();
        let mut c = ctx(&w, 120.0 * 16.0, true);
        c.flags.zone_underground_desert = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        // 600 x0.4 (deep) x0.2 (UD) = 48 -> floored at 60 (NPC.cs:741-744).
        assert_eq!(r.spawn_rate, 60);
        assert_eq!(r.max_spawns, 15, "x3 of 5 capped at 15");
    }

    #[test]
    fn moon_event_forces_the_raid_pacing() {
        // NPC.cs:771-775: maxSpawns = 5 x (2 + 0.3x1) = 11, spawnRate = 20 —
        // gated on the player above the surface line (25 < worldSurface 30).
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.flags.moon_event = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (20, 11));
    }

    #[test]
    fn the_moon_pacing_keeps_the_unpaced_rate_underground() {
        // `NPC.cs:771`: `(Main.pumpkinMoon || Main.snowMoon) &&
        // (Main.remixWorld || player.position.Y < Main.worldSurface * 16.0)`
        // — an UNDERGROUND player during a moon event keeps the un-paced
        // rate (research 24's errata; the port had applied it everywhere).
        // y = 600 px = tile 37.5 >= worldSurface 30, but below the depth
        // band's `worldSurface*16 + sHeight` = 1680 px, so the plain night
        // arm applies: 600 x0.6 = 360, (float)6.5 -> 6. The inner night
        // moon arm (`:542-546`) carries the same `< worldSurface*16` gate,
        // so neither the x0.2/x2 nor the 20/11 override fires.
        let w = ground();
        let mut c = ctx(&w, 600.0, false);
        c.flags.moon_event = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (360, 6));
        // The boundary is STRICT `<`: exactly on the line stays un-paced.
        let mut on_line = ctx(&w, 30.0 * 16.0, false);
        on_line.flags.moon_event = true;
        let r = get_spawn_rate(&on_line, &mut UnifiedRandom::new(1));
        assert_eq!((r.spawn_rate, r.max_spawns), (360, 6));
    }

    #[test]
    fn invaders_override_forces_the_raid_pacing() {
        // NPC.cs:781-785: maxSpawns = 5 x (2 + 0.3x1) = 11, spawnRate = 20.
        let w = ground();
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.invaders = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
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
        // The stream fingerprint per seed (the `UnifiedRandom` state after
        // one surface-spawn call): identical everywhere the holiday arm's
        // short-circuit skips its draw, shifted where it fires.
        let fingerprint = |c: &SpawnContext<'_>, seed: u64| -> (UnifiedRandom, Option<NpcKind>) {
            let mut rng = UnifiedRandom::new(seed as i32);
            let n = spawn_npc_at(c, &mut rng, 210, 295 * 16, false, false, false).map(|n| n.kind);
            (rng.clone(), n)
        };
        let mut shifted = 0usize;
        let mut compared = 0usize;
        for seed in 1..4000u64 {
            let a = fingerprint(&night, seed);
            let b = fingerprint(&holiday, seed);
            if a != b {
                shifted += 1;
                // S1-D-3z: the holiday arm's GATE roll (`Next(2)`) shifts
                // the stream even when it fails — the ladder then falls
                // through to the plain zombie (the C# `&&` short-circuit
                // spends the gate draw either way). When the gate passes,
                // the `Next(319, 322)` pick lands on Doctor/Superman/
                // Pixie — a spawn is never LOST by the early return.
                assert!(
                    matches!(
                        b.1,
                        Some(NpcKind::Zombie)
                            | Some(NpcKind::ZombieDoctor)
                            | Some(NpcKind::ZombieSuperman)
                            | Some(NpcKind::ZombiePixie)
                    ),
                    "the holiday arm's outcome kind, got {:?}",
                    b.1
                );
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
            let mut a = UnifiedRandom::new(seed as i32);
            let mut b = UnifiedRandom::new(seed as i32);
            let _ = spawn_npc_at(&plain, &mut a, 210, 295 * 16, false, false, false);
            let _ = spawn_npc_at(&plain, &mut b, 210, 295 * 16, false, false, false);
            assert_eq!(a, b, "the classic ladder replays per seed");
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
        let mut rng = UnifiedRandom::new(seed as i32);
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
            let mut rng = UnifiedRandom::new(seed as i32);
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
            let mut rng = UnifiedRandom::new(seed as i32);
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
            moon_npc_counts: &[],
            world: w,
            player_center: (200.0 * TILE_PX, 295.0 * TILE_PX),
            player_pos: (200.0 * TILE_PX, 295.0 * TILE_PX),
            nearby_active_npcs: 0.0,
            town_npcs: 0,
            day_time: day,
            player_life_max: 100,
            luck: 0.0,
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
            moon_boss_cap_count: 0.0,
            downed_boss1: false,
            sandstorm_happening: false,
        }
    }

    #[test]
    fn spawn_rate_matches_the_c_modifiers() {
        let w = ground_tall();
        // With no NPCs nearby the <20% scaling (NPC.cs:670-673) always applies,
        // so the base 600 rate is ×0.6 = 360.
        // Day surface: 600 × 0.6.
        let r = get_spawn_rate(&ctx_at(&w, true), &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
        assert_eq!(r.max_spawns, 5);

        // Night: ×0.6 / ×1.3, then ×0.6 again.
        let r = get_spawn_rate(&ctx_at(&w, false), &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 216);
        assert_eq!(r.max_spawns, 6);

        // Below the rock layer: ×0.4 / ×1.9, then both nearby ramps (the
        // x0.6 pass and the deep x0.7 extra pass; tile y 420 px 6720 is past
        // `rock_layer*16 + sHeight` (6320) and past the mid-depth line 315):
        // 600 -> 240 -> 144 -> 100.8 -> 100.
        let mut c = ctx_at(&w, true);
        c.player_pos = (200.0 * TILE_PX, 420.0 * TILE_PX);
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 100);
        assert_eq!(r.max_spawns, 9);

        // Underworld doubles the cap (tile y 505 > height - 200 = 500).
        let mut c = ctx_at(&w, true);
        c.player_pos = (200.0 * TILE_PX, 505.0 * TILE_PX);
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.max_spawns, 10);

        // The surface town-≥3 arm latches `noWorms` (`:904`), sets
        // `spawnFriendly` without a draw (classic `!Main.expertMode`,
        // `:916-918`) and halves the cap ×0.6 (`:920`) — the rate itself is
        // untouched (360, the nearby ramp; NPC.cs:673-690).
        let mut c = ctx_at(&w, true);
        c.town_npcs = 3;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 360);
        assert!(r.no_worms && r.spawn_friendly, "the town ≥3 latch arm");

        // The nearby scaling stops at ≥80% of the cap (4.0 ≥ 0.8·5): no
        // multiplier applies, the base 600 survives.
        let mut c = ctx_at(&w, true);
        c.nearby_active_npcs = 4.0;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, 600);
    }

    #[test]
    fn s1_e_7a_invis_and_battle_flags_feed_the_spawn_modifiers() {
        // S1-E-7 wave A: the Invisibility (10) / Battle (13) buff arms
        // (`Player.cs:9991`/`:10003`) set `player.invis`/`enemySpawns`, which
        // `NPC.GetSpawnRate` reads (`NPC.cs:702-740`): invis ×1.2 rate / ×0.8
        // cap, battle ×0.5 rate / ×2 cap. Day surface base: 360 / 5.
        let w = ground_tall();
        let mut c = ctx_at(&w, true);
        c.flags.invis = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, (360.0 * 1.2) as i32);
        assert_eq!(r.max_spawns, (5.0 * 0.8) as i32);
        let mut c = ctx_at(&w, true);
        c.flags.enemy_spawns = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, (360.0 * 0.5) as i32);
        assert_eq!(r.max_spawns, (5.0 * 2.0) as i32);
    }

    #[test]
    fn w4_d105_the_sunflower_flag_feeds_the_spawn_modifiers() {
        // W4-d105 doc-sweep: the Sunflower (146) buff's `player.sunflower`
        // flag (`Player.cs:11003`, set by the wave-C slot chain) feeds
        // `NPC.GetSpawnRate` (`NPC.cs:712-716`): rate ×1.2, cap ×0.8 — the
        // same shape as invis, applied AFTER it (`:702` then `:712`).
        // Day surface base: 360 / 5 → 432 / 4.
        let w = ground_tall();
        let mut c = ctx_at(&w, true);
        c.flags.sunflower = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, (360.0 * 1.2) as i32);
        assert_eq!(r.max_spawns, (5.0 * 0.8) as i32);
        // The stacked order: invis first (432/4), then sunflower
        // (432 × 1.2 = 518.4 → 518; 4 × 0.8 = 3.2 → 3).
        let mut c = ctx_at(&w, true);
        c.flags.invis = true;
        c.flags.sunflower = true;
        let r = get_spawn_rate(&c, &mut UnifiedRandom::new(1));
        assert_eq!(r.spawn_rate, (432.0 * 1.2) as i32);
        assert_eq!(r.max_spawns, (4.0 * 0.8) as i32);
        // `anglerSetSpawnReduction` (`NPC.cs:717-721`) is dead code in the
        // C# — `Player.cs:2808` declares it, `:18729` clears it, no site
        // sets it true — so the port's `angler_set` arm stays constants-off
        // (the flag exists for the C# shape only).
    }

    #[test]
    fn spawns_land_outside_the_safe_area() {
        let w = ground_tall();
        let mut rng = UnifiedRandom::new(0xDEAD_BEEFu32 as i32);
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
        let mut rng = UnifiedRandom::new(42);
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
        let mut rng = UnifiedRandom::new(7);
        let mut c = ctx_at(&w, false);
        c.nearby_active_npcs = 100.0; // far above every max-spawns cap
        for _ in 0..2000 {
            assert!(try_spawn_an_npc(&c, &mut rng).is_none());
        }
    }

    /// W4-g84: the `SpawnOnPlayer` generic ground scan
    /// (`NPC.cs:80565-80632`) — the bulb's Plantera summon rides it. On the
    /// open `ground_tall` floor the scan finds a ground tile inside the
    /// spawn area, deterministically per seed; the chosen cell is solid
    /// with the 2×3 clear rect above it (the non-final attempts' gates).
    #[test]
    fn w4_g84_spawn_on_player_tile_finds_open_ground() {
        let w = ground_tall();
        let mut c = ctx_at(&w, true);
        c.nearby_active_npcs = 0.0;
        let (spawn_area, _) = get_spawn_area(&c);
        for seed in 1..40u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let (x, y) = spawn_on_player_tile(&c, &mut rng)
                .unwrap_or_else(|| panic!("seed {seed} found no ground"));
            assert!(
                x >= spawn_area.left && x < spawn_area.right,
                "the column inside the spawn area"
            );
            assert!(
                y >= spawn_area.top && y <= spawn_area.bottom,
                "the ground row inside the area"
            );
            assert!(
                w.tile(x, y)
                    .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)),
                "the chosen cell is solid ground"
            );
            // Determinism: the same seed replays the same tile.
            let mut rng2 = UnifiedRandom::new(seed as i32);
            assert_eq!(spawn_on_player_tile(&c, &mut rng2), Some((x, y)));
        }
    }

    /// W4-g84: `stamp_boss_spawn` — the `NewNPC` 937 window × the
    /// `SpawnBoss` ×20 (`NPC.cs:80772`/`:80686`) and the `HasAwoken`
    /// broadcast in `ChatColors.BossOrEvent` (`:80702-80704`).
    #[test]
    fn w4_g84_stamp_boss_spawn_window_and_broadcast() {
        let mut n = Npc::new(NpcKind::Plantera, 0.0, 0.0, 1, 0.0);
        let mut chat = crate::chat::ChatMonitor::default();
        stamp_boss_spawn(&mut n, &mut chat, "Plantera");
        assert_eq!(n.despawn_timer, 18740);
        assert_eq!(chat.messages.len(), 1);
        assert!(chat.messages[0].text.contains("Plantera has awoken"));
        assert_eq!(chat.messages[0].color, crate::chat::COLOR_BOSS_OR_EVENT);
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
        let mut rng = UnifiedRandom::new(0xDEAD_BEEFu32 as i32);
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
        let mut rng = UnifiedRandom::new(7);
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
            let mut rng = UnifiedRandom::new(seed as i32);
            let mut decisions = Vec::new();
            let mut state = seed;
            for _ in 0..900 {
                let ctx = SpawnContext {
                    moon_npc_counts: &[],
                    world: &w,
                    player_center: (200.0 * TILE_PX, 295.0 * TILE_PX),
                    player_pos: (200.0 * TILE_PX, 295.0 * TILE_PX),
                    nearby_active_npcs: 0.0,
                    town_npcs: 0,
                    day_time: false,
                    player_life_max: 100,
                    luck: 0.0,
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
                    moon_boss_cap_count: 0.0,
                    downed_boss1: false,
                    sandstorm_happening: false,
                };
                let (spawned, _) = spawn_npc_call(Some(ctx), &mut rng, false);
                decisions.push(spawned.map(|n| (n.kind as u8, n.x.to_bits())));
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                rng = UnifiedRandom::new(state as i32);
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
            let mut rng = UnifiedRandom::new(seed as i32);
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
            moon_npc_counts: &[],
            world,
            player_center: (world.width as f32 * TILE_PX / 2.0, 120.0 * TILE_PX),
            player_pos: (world.width as f32 * TILE_PX / 2.0, 120.0 * TILE_PX),
            nearby_active_npcs: 100.0,
            town_npcs: 0,
            day_time: true,
            player_life_max: 100,
            luck: 0.0,
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
            moon_boss_cap_count: 0.0,
            downed_boss1: false,
            sandstorm_happening: false,
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
                &mut UnifiedRandom::new(seed as i32),
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
                &mut UnifiedRandom::new(seed as i32),
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
                &mut UnifiedRandom::new(seed as i32),
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
                &mut UnifiedRandom::new(seed as i32),
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
                &mut UnifiedRandom::new(seed as i32),
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
    // ---- the moved table tests ride the spawner stream
    // (`UnifiedRandom`): the kind tables and thresholds are the moved ones;
    // the float-stand-in's `rand01(&mut u64)` draws are retired (S1-A-11).

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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(3);
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
        let mut rng = UnifiedRandom::new(7);
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
        // `invasionType == 2` (`NPC.cs:1453-1466`): the Balla's net 145
        // rides the 1/7 arm, the Gangsta's net 143 the 1/3 arm, the
        // Stabby's net 144 the else — always the frost trio (the port's
        // kind *names* for net 144/145 ride the pre-existing roster
        // labels: SnowBalla = 144, MysterySnowman = 145).
        let w = ground();
        let c = inv_pick_ctx(&w, 2, false);
        let mut rng = UnifiedRandom::new(9);
        let (mut gangsta, mut stabby, mut balla) = (false, false, false);
        for _ in 0..120 {
            let kind = invasion_spawn_pick(&c, &mut rng).expect("type 2 picks");
            assert!(matches!(
                kind,
                NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman
            ));
            gangsta |= kind == NpcKind::SnowmanGangsta;
            stabby |= kind == NpcKind::SnowBalla;
            balla |= kind == NpcKind::MysterySnowman;
        }
        assert!(gangsta && stabby && balla, "the trio's three arms fire");
        // The goblin table stays on type 1.
        let c = inv_pick_ctx(&w, 1, false);
        let mut rng = UnifiedRandom::new(9);
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
        let mut rng = UnifiedRandom::new(11);
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
        let mut rng = UnifiedRandom::new(5);
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
        let mut rng = UnifiedRandom::new(5);
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
        let mut rng = UnifiedRandom::new(21);
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
        let mut rng = UnifiedRandom::new(21);
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
        let mut rng = UnifiedRandom::new(13);
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
        let mut rng = UnifiedRandom::new(3);
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
        let mut rng = UnifiedRandom::new(5);
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
            let mut rng = UnifiedRandom::new(seed as i32);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the invasion arm spawns on the found tile");
            let mut ref_rng = UnifiedRandom::new(seed as i32);
            let _style = ref_rng.next_max(7); // GetZombieSettings
            let kind = if ref_rng.next_max(9) == 0 {
                NpcKind::GoblinSorcerer // 29
            } else if ref_rng.next_max(5) == 0 {
                NpcKind::GoblinThief // 26
            } else if ref_rng.next_max(3) == 0 {
                NpcKind::GoblinPeon // 111
            } else if ref_rng.next_max(3) == 0 {
                NpcKind::GoblinWarrior // 27
            } else {
                NpcKind::GoblinArcher // 28
            };
            assert_eq!(n.kind, kind, "seed {seed}: the ladder's kind");
            assert_eq!(
                rng, ref_rng,
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
        let ray_arm = |rng: &mut UnifiedRandom| match rng.next_max(5) {
            0 | 1 => NpcKind::RayGunner,      // 382
            2 | 3 => NpcKind::BrainScrambler, // 381
            _ => NpcKind::MartianDrone,       // 388
        };
        let ground_chain = |rng: &mut UnifiedRandom, walker_alive: bool| -> NpcKind {
            let mut num9 = rng.next_max(4);
            if num9 == 3 {
                if !walker_alive {
                    return NpcKind::MartianWalker; // 520
                }
                num9 = rng.next_max(3);
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
            let mut rng = UnifiedRandom::new(seed as i32);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the type-4 ground arm spawns");
            let mut ref_rng = UnifiedRandom::new(seed as i32);
            let _style = ref_rng.next_max(7);
            let num6 = ref_rng.next_max(7);
            let _saucer_gate = ref_rng.next_max(45); // always drawn
            let kind = if num6 >= 6 {
                let _rider_gate = ref_rng.next_max(20); // always drawn
                if ref_rng.next_max(2) == 0 {
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
            assert_eq!(rng, ref_rng, "seed {seed}: the closed-gate stream tail");
        }
        // The gate open (progress_30, no Saucer alive): the 395 arms fire
        // on their rolls.
        let mut c = ctx(&w, 295.0 * 16.0, true);
        c.invaders = true;
        c.invasion_type = 4;
        c.invasion_progress_30 = true;
        let mut saucers = 0u32;
        for seed in 1..500u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the type-4 ground arm spawns");
            let mut ref_rng = UnifiedRandom::new(seed as i32);
            let _style = ref_rng.next_max(7);
            let num6 = ref_rng.next_max(7);
            let kind = if ref_rng.next_max(45) == 0 {
                NpcKind::MartianSaucerCore // 395
            } else if num6 >= 6 {
                if ref_rng.next_max(20) == 0 {
                    NpcKind::MartianSaucerCore // 395
                } else if ref_rng.next_max(2) == 0 {
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
            assert_eq!(rng, ref_rng, "seed {seed}: the open-gate stream tail");
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
            let mut rng = UnifiedRandom::new(seed as i32);
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
            let mut rng = UnifiedRandom::new(seed as i32);
            let n = spawn_npc_at(&c, &mut rng, 210, 295 * 16, false, false, false)
                .expect("the ground arm fires");
            saw_ground |= n.kind != NpcKind::MartianDrone;
        }
        assert!(saw_ground, "the ground ladder ran on the unlatched tile");
    }

    // --- S1-F-13 Phase 1: the wave-20 boss cap (`Spawner`'s ctor) ---

    /// DW-1.4: the slots table, transcribed from each kind's `SetDefaults`
    /// (`NPC.cs:8510`'s common reset is 1; the writes are `:12978` (315),
    /// `:13115` (325), `:13142` (327), `:13158` (328), `:13355` (344),
    /// `:13369` (345), `:13386` (346)).
    #[test]
    fn the_boss_cap_slots_table_matches_the_c_sharp() {
        for (id, slots) in [
            (315u16, 1.0f32),
            (325, 3.0),
            (327, 5.0),
            (328, 0.0),
            (344, 3.0),
            (345, 5.0),
            (346, 4.0),
        ] {
            assert_eq!(moon_boss_slots(id), Some(slots), "kind {id} weighs {slots}");
        }
        // Every other type is outside the ctor's switch.
        for id in [4u16, 50, 305, 338, 352, 1253, 6296] {
            assert_eq!(moon_boss_slots(id), None, "kind {id} is not a wave-20 boss");
        }
        assert_eq!(moon_boss_slots(0), None);
    }

    /// DW-1.4: the weighted count over the live NPCs — the non-boss kinds
    /// never move it. `S1-F-13c` slice B lands 345/346, so their `npcSlots`
    /// rows (5.0/4.0) weigh live too.
    #[test]
    fn the_boss_cap_counts_only_the_live_boss_kinds() {
        let npcs: Vec<Npc> = (0..24)
            .map(|i| Npc::new(NpcKind::Zombie, i as f32 * 32.0, 100.0, 1, 0.0))
            .collect();
        assert_eq!(moon_boss_cap_count(&npcs), 0.0);
        assert!(!moon_boss_cap_reached(0.0));
        assert_eq!(moon_boss_cap_count(&[]), 0.0);
        // The frost bosses weigh their `SetDefaults` npcSlots rows
        // (`NPC.cs:13369`/`:13386` — 5/4); a dead one weighs nothing.
        let mut queen = Npc::new(NpcKind::IceQueen, 0.0, 100.0, 1, 0.0);
        let mut santa = Npc::new(NpcKind::SantaNK1, 32.0, 100.0, 1, 0.0);
        let mut dead_queen = Npc::new(NpcKind::IceQueen, 64.0, 100.0, 1, 0.0);
        queen.hp = queen.max_hp;
        santa.hp = santa.max_hp;
        dead_queen.hp = 0;
        assert_eq!(moon_boss_cap_count(&[queen, santa, dead_queen]), 9.0);
        assert!(!moon_boss_cap_reached(9.0), "9 < 11 stays open");
        // The frost walkers are not bosses — they contribute nothing.
        let mut nut = Npc::new(NpcKind::Nutcracker, 0.0, 100.0, 1, 0.0);
        nut.hp = nut.max_hp;
        assert_eq!(moon_boss_cap_count(&[nut]), 0.0);
    }

    /// DW-1.4: the bound — the port's single active player reads
    /// `(int)(defaultMaxSpawns × (2 + 0.3 × players))` = 11 (`NPC.cs:285`), the
    /// same `5·2.3` the moon raid pacing forces, and the C# `>=` compare.
    #[test]
    fn the_boss_cap_bound_is_the_single_player_5_2_3() {
        assert_eq!(moon_boss_cap_bound(), 11);
        assert!(!moon_boss_cap_reached(10.9), "10.9 < 11 stays open");
        assert!(moon_boss_cap_reached(11.0), "the C# >= reaches at 11");
        assert!(moon_boss_cap_reached(20.0));
    }

    // ---- S1-F-13 Phase 2: the roster ladders in the shared spawn chain ----

    /// A night surface world with the frost moon running at a given wave.
    fn moon_world(wave: i32, snow: bool) -> World {
        let mut w = World::new("t".into(), 400, 700);
        w.world_surface = 30.0;
        w.snow_moon = snow;
        w.pumpkin_moon = !snow;
        w.wave_number = wave;
        w
    }

    /// DIRTY (watched RED pre-fix): DW-2.1 — with a night + snow moon +
    /// wave 20 armed, the ladder draws the vanilla boss-trio `Next(3)`
    /// sequence on the spawner stream. `S1-F-13c` slices A+B port all of
    /// 341/344/345/346, so every pick SPAWNS at the found tile.
    ///
    /// The reference replay (`NPC.cs:2804-2830`): `GetZombieSettings`' style
    /// `Next(7)` (`:1284`), then the arm head EXACTLY as the C# runs it — the
    /// 341 `Next(30) == 0 && CountNPCS(341) < 4` arm (`:2807-2811`) hits only
    /// 1/30 of the time and RETURNS on the hit (no `num44` draw); otherwise
    /// the wave-20 branch draws `num44 = Next(3)` (`:2814`) and picks
    /// 345/346/344 per num44 (`:2820-2826`). The picks and both stream tails
    /// (the ladder's and `spawn_npc_at`'s, which is the style draw + the arm)
    /// must match the replay seed for seed.
    #[test]
    fn the_snow_wave_20_boss_trio_replays_the_vanilla_next3_sequence() {
        let w = moon_world(20, true);
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.moon_boss_cap_count = 0.0; // the cap is open
        let (tile_x, tile_y) = (210i32, 25i32 * 16);
        let mut saw_head = 0;
        let mut saw_trio = 0;
        for seed in 1..=200u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let spawn = spawn_npc_at(&c, &mut rng, tile_x, tile_y, false, false, false);
            // The ladder's pick on its own stream: after the style draw
            // `GetZombieSettings` puts ahead of every arm (`NPC.cs:1284`).
            let mut pick_rng = UnifiedRandom::new(seed as i32);
            let _style = pick_rng.next_max(7);
            let picked = frost_moon_ladder(&c, &mut pick_rng);
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let _style = r#ref.next_max(7); // `GetZombieSettings` (`NPC.cs:1284`)
            let expect_ty = if r#ref.next_max(30) == 0 {
                // The 341 head hit (`:2807-2811`): the C# spawns and returns —
                // the trio's `Next(3)` is never drawn, so the replay stops
                // here. `CountNPCS(341) < 4` is open (the ctx's roster is
                // empty).
                saw_head += 1;
                assert_eq!(
                    picked,
                    Some(341),
                    "seed {seed}: the head arm picks the vanilla Present Mimic type"
                );
                341u16
            } else {
                let num44 = r#ref.next_max(3); // the boss-trio pick (`:2814`)
                let trio = match num44 {
                    0 => 345u16,
                    1 => 346,
                    _ => 344,
                };
                saw_trio += 1;
                assert_eq!(
                    picked,
                    Some(trio),
                    "seed {seed}: the ladder picks the vanilla trio type"
                );
                trio
            };
            // Slices A+B: every trio/head pick (341/344/345/346) decodes and
            // spawns at the found tile (the `NewNPC` Bottom placement).
            match NpcKind::moon_wave_kind(expect_ty) {
                Some(kind) => {
                    let n = spawn.expect("seed: the ported frost boss must spawn");
                    assert_eq!(n.kind, kind, "seed {seed}: the wrong kind spawned");
                    let (nw, nh) = kind.size();
                    assert_eq!(
                        n.x, tile_x as f32 * TILE_PX + 8.0 - nw * 0.5,
                        "seed {seed}: the feet sit centred 8 px into the found tile"
                    );
                    assert_eq!(
                        n.y, tile_y as f32 - nh,
                        "seed {seed}: the feet stand on the found tile's top edge"
                    );
                    assert_eq!(n.facing, -1, "seed {seed}: the spawn faces the player");
                    assert_eq!(n.hp, n.max_hp, "seed {seed}: the spawn is at full hp");
                }
                None => assert!(
                    spawn.is_none(),
                    "seed {seed}: the unported kind {expect_ty} must not spawn"
                ),
            }
            assert_eq!(
                pick_rng.peek(), r#ref.peek(),
                "seed {seed}: the ladder's stream tail is the vanilla draw order"
            );
            assert_eq!(
                rng.peek(), r#ref.peek(),
                "seed {seed}: `spawn_npc_at`'s stream tail is the vanilla draw order"
            );
        }
        assert!(
            saw_head > 0 && saw_trio > 0,
            "the 200 seeds must exercise both head outcomes (head {saw_head}, trio {saw_trio})"
        );
    }

    /// DW-2.1's cap half: the wave-20 trio sits behind `!reachedInvasionBossCap`
    /// (`NPC.cs:2815`) — a reached cap draws `num44 = Next(3)` anyway (the draw
    /// precedes the gate, `:2813-2815`) and spawns nothing. The 341 head arm
    /// is NOT gated: its pick still spawns the ported Present Mimic
    /// (`S1-F-13c` slice A).
    #[test]
    fn the_snow_wave_20_trio_spawns_nothing_with_the_boss_cap_reached() {
        let w = moon_world(20, true);
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.moon_boss_cap_count = 20.0; // >= the bound (11): the cap is reached
        for seed in 1..=200u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let spawn = spawn_npc_at(&c, &mut rng, 210, 25 * 16, false, false, false);
            let mut pick_rng = UnifiedRandom::new(seed as i32);
            let _style = pick_rng.next_max(7);
            let picked = frost_moon_ladder(&c, &mut pick_rng);
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let _style = r#ref.next_max(7); // `GetZombieSettings` (`NPC.cs:1284`)
            let _head = r#ref.next_max(30); // the 341 head arm (`:2807`) — still drawn
            if _head == 0 {
                // The head arm's hit still spawns 341 behind the cap — only
                // the boss trio is gated (`:2815`).
                assert_eq!(picked, Some(341), "seed {seed}: the head arm is not gated");
                assert_eq!(
                    spawn.map(|n| n.kind),
                    Some(crate::npc::NpcKind::PresentMimic),
                    "seed {seed}: the ungated head spawns the ported Present Mimic"
                );
            } else {
                let _num44 = r#ref.next_max(3); // `:2814` — drawn BEFORE the cap gate
                assert_eq!(
                    picked, None,
                    "seed {seed}: the reached cap suppresses the trio (and keeps its draw)"
                );
                assert!(
                    spawn.is_none(),
                    "seed {seed}: nothing spawns with the cap reached"
                );
            }
            assert_eq!(
                pick_rng.peek(), r#ref.peek(),
                "seed {seed}: the cap-reached stream tail is the vanilla draw order"
            );
            assert_eq!(
                rng.peek(), r#ref.peek(),
                "seed {seed}: `spawn_npc_at`'s cap-reached tail is the vanilla order"
            );
        }
    }

    /// DW-2.2: the per-wave ladder fires in the C# branch order — wave 19
    /// (`NPC.cs:2832-2840`) is the chain `Next(10)==0 && CountNPCS(345)<4` →
    /// 345, `<5` → 346, `<7` → 344, else 343 — and each arm's roll is only
    /// drawn while no earlier arm fired (the `else if` short-circuit).
    #[test]
    fn the_wave_19_ladder_fires_the_csharp_branch_order() {
        let w = moon_world(19, true);
        let c = ctx(&w, 25.0 * 16.0, false);
        // The reference replay: an arm that hits stops the chain (one Next(10)
        // consumed); an arm that misses draws its Next(10) and advances.
        for seed in 1..=200u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let _style = rng.next_max(7); // `GetZombieSettings` (`NPC.cs:1284`)
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let _s = r#ref.next_max(7); // the style draw (outside the ladder)
            // The 341 head arm (`:2807-2811`) is INSIDE the ladder and returns
            // on its hit — the replay mirrors it.
            let head = r#ref.next_max(30);
            let expect = if head == 0 {
                Some(341u16)
            } else {
                let n1 = r#ref.next_max(10); // arm 1: 345 (`:2832-2835`)
                if n1 == 0 {
                    Some(345)
                } else {
                    let n2 = r#ref.next_max(10); // arm 2: 346 (`:2836-2839`)
                    if n2 == 0 {
                        Some(346)
                    } else {
                        let n3 = r#ref.next_max(10); // arm 3: 344 (`:2840-2843`)
                        if n3 == 0 {
                            Some(344)
                        } else {
                            Some(343) // the `else` (`:2844-2846`)
                        }
                    }
                }
            };
            assert_eq!(
                frost_moon_ladder(&c, &mut rng),
                expect,
                "seed {seed}: the wave-19 chain must replay the C# branch order"
            );
            assert_eq!(
                rng.peek(), r#ref.peek(),
                "seed {seed}: the else-if short-circuit consumes exactly one Next(10) per fired arm"
            );
        }
    }

    /// DW-2.2's cap half: a closed `CountNPCS` gate suppresses its arm's pick
    /// while its roll is still drawn (the C#'s `&&` — `NPC.cs:2832-2835`), so
    /// the chain advances to the next arm. This test drives `moon_arm_count`
    /// directly with synthetic counts — the closed-gate mode — independent
    /// of which kinds are ported.
    #[test]
    fn a_closed_count_cap_suppresses_its_arm_and_advances() {
        let mut picked = None;
        let mut rng = UnifiedRandom::new(1);
        // A seed whose first Next(10) hits, with the cap open: the arm fires.
        let hit_seed = (1..=100u64).find(|&s| UnifiedRandom::new(s as i32).next_max(10) == 0).unwrap();
        let mut rng = UnifiedRandom::new(hit_seed as i32);
        assert!(moon_arm_count(
            &mut picked, &mut rng, 10, Some(4), 345, 0
        ));
        assert_eq!(picked, Some(345));
        // The same roll, the cap closed (count 4 >= 4): suppressed, no pick,
        // the draw consumed.
        let mut picked2 = None;
        let mut rng2 = UnifiedRandom::new(hit_seed as i32);
        let fired = moon_arm_count(&mut picked2, &mut rng2, 10, Some(4), 345, 4);
        assert!(!fired, "the closed cap suppresses the pick");
        assert_eq!(picked2, None, "nothing picked behind the closed cap");
        assert_eq!(rng2.peek(), rng.peek(), "the roll was still drawn (the C# && order)");
    }

    /// DW-2.3: across the whole snow wave range the ladder's picks (when
    /// Some) are vanilla `NPC.type` wave ids. `S1-F-13c` slices A+B land
    /// all of 338-352: every Some pick SPAWNS through `moon_wave_kind` at
    /// the found tile; only the wave-20 boss-cap refusal returns None.
    #[test]
    fn the_snow_ladder_picks_are_wave_ids_and_spawn_only_the_ported_band() {
        const SNOW_WAVE_IDS: std::ops::RangeInclusive<u16> = 338..=352;
        let mut saw_ported_spawn = false;
        let mut saw_unported_consume = false;
        for wave in 1..=20i32 {
            let w = moon_world(wave, true);
            let c = ctx(&w, 25.0 * 16.0, false);
            for seed in 1..=100u64 {
                let mut rng = UnifiedRandom::new(seed as i32);
                // `GetZombieSettings`' style draw sits ahead of every arm
                // (`NPC.cs:1284`) — `spawn_npc_at` consumes it too, so the
                // ladder replay must draw it first to stay stream-aligned.
                let _style = rng.next_max(7);
                let picked = frost_moon_ladder(&c, &mut rng);
                if let Some(kind) = picked {
                    assert!(
                        SNOW_WAVE_IDS.contains(&kind),
                        "wave {wave} seed {seed}: picked {kind} is not a vanilla snow wave id"
                    );
                }
                let spawns = spawn_npc_at(&c, &mut UnifiedRandom::new(seed as i32), 210, 25 * 16, false, false, false);
                match picked.and_then(NpcKind::moon_wave_kind) {
                    Some(kind) => {
                        let n = spawns.expect("the ported frost kind must spawn");
                        assert_eq!(
                            n.kind, kind,
                            "wave {wave} seed {seed}: the wrong kind spawned"
                        );
                        assert_eq!(n.hp, n.max_hp, "wave {wave} seed {seed}: full hp");
                        saw_ported_spawn = true;
                    }
                    None => {
                        assert!(
                            spawns.is_none(),
                            "wave {wave} seed {seed}: no unported wave kind spawns"
                        );
                        if picked.is_some() {
                            saw_unported_consume = true;
                        }
                    }
                }
            }
        }
        // Every frost wave id is ported now (slices A+B): every Some pick
        // spawns; the only no-spawn outcome left is the wave-20 boss-cap
        // refusal, which the dedicated cap tests pin.
        assert!(
            saw_ported_spawn,
            "the sweep must exercise the spawn outcome"
        );
    }

    /// DW-2.4: the mutual exclusion — the snow event dispatches the frost
    /// ladder only, the pumpkin event the pumpkin ladder only, and by day the
    /// moon arms never draw (the flags leave the day path's stream untouched).
    #[test]
    fn the_moon_arms_are_mutually_exclusive_and_gated_by_day() {
        // (a) snow only: the dispatch's stream equals the frost ladder's
        // replay (style + head + the ladder) — the pumpkin arm never runs.
        let w = moon_world(5, true);
        let c = ctx(&w, 25.0 * 16.0, false);
        let mut rng = UnifiedRandom::new(7);
        let _ = spawn_npc_at(&c, &mut rng, 210, 25 * 16, false, false, false);
        // The ladder draws its own head arm internally (`NPC.cs:2807`) — the
        // replay is the style draw then the ladder on the same stream.
        let mut pick_rng = UnifiedRandom::new(7);
        pick_rng.next_max(7); // `GetZombieSettings` (`NPC.cs:1284`)
        let _ = frost_moon_ladder(&c, &mut pick_rng);
        let mut pumpkin_only = UnifiedRandom::new(7);
        pumpkin_only.next_max(7);
        let _ = pumpkin_moon_ladder(&c, &mut pumpkin_only);
        assert_eq!(
            rng.peek(), pick_rng.peek(),
            "the snow dispatch replays the frost ladder exactly"
        );
        assert_ne!(
            rng.peek(), pumpkin_only.peek(),
            "the snow dispatch must NOT run the pumpkin ladder"
        );
        // (b) pumpkin only: the dispatch replays the pumpkin ladder, not the
        // frost one (their draws differ from wave 5 on).
        let w = moon_world(5, false);
        let c = ctx(&w, 25.0 * 16.0, false);
        let mut rng = UnifiedRandom::new(7);
        let _ = spawn_npc_at(&c, &mut rng, 210, 25 * 16, false, false, false);
        let mut frost_only = UnifiedRandom::new(7);
        frost_only.next_max(7);
        let _ = frost_moon_ladder(&c, &mut frost_only);
        let mut pumpkin_only = UnifiedRandom::new(7);
        pumpkin_only.next_max(7);
        let _ = pumpkin_moon_ladder(&c, &mut pumpkin_only);
        assert_eq!(
            rng.peek(), pumpkin_only.peek(),
            "the pumpkin dispatch replays the pumpkin ladder"
        );
        assert_ne!(
            rng.peek(), frost_only.peek(),
            "the pumpkin dispatch must NOT run the frost ladder"
        );
        // (c) by day the moon arms never draw: the day dispatch's stream is
        // identical with and without the event flags on.
        let mut w_day = moon_world(5, true);
        let c_day_moon = ctx(&w_day, 25.0 * 16.0, true);
        let mut w_plain = World::new("t".into(), 400, 700);
        w_plain.world_surface = 30.0;
        let c_day_plain = ctx(&w_plain, 25.0 * 16.0, true);
        let mut rng_moon = UnifiedRandom::new(7);
        let _ = spawn_npc_at(&c_day_moon, &mut rng_moon, 210, 25 * 16, false, false, false);
        let mut rng_plain = UnifiedRandom::new(7);
        let _ = spawn_npc_at(&c_day_plain, &mut rng_plain, 210, 25 * 16, false, false, false);
        assert_eq!(
            rng_moon.peek(), rng_plain.peek(),
            "by day the moon arms draw nothing — the flags leave the day stream untouched"
        );
    }

    /// DIRTY (watched RED pre-fix): DW-2.1 — with a night pumpkin moon at
    /// wave 2 armed, the ladder's ported picks SPAWN: `spawn_npc_at` returns
    /// `Some(Npc)` of the picked kind at the found tile — the
    /// `SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, type)` → `NewNPC`'s
    /// `Bottom = (X, Y)` placement (the moon arm `NPC.cs:3224-3548`,
    /// `NewNPC` `:80772`): the feet on the found tile's top edge, X centred
    /// 8 px into the tile, facing the player (`TargetClosest`), full hp.
    /// Wave 2 (`:3529-3533`) is the 326 arm (1/3) falling through to the
    /// scarecrow band `Next(305, 315)` — Splinterling and all ten
    /// Scarecrows, the ported kinds the plan names.
    #[test]
    fn the_pumpkin_wave_spawns_the_ported_kinds_at_the_found_tile() {
        let w = moon_world(2, false);
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.moon_boss_cap_count = 0.0; // the cap is open
        let (tile_x, tile_y) = (210i32, 25i32 * 16);
        let mut saw_splinterling = 0;
        let mut kinds_seen = std::collections::BTreeSet::new();
        for seed in 1..=200u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let spawn = spawn_npc_at(&c, &mut rng, tile_x, tile_y, false, false, false);
            // The replay (`:1284` the style, `:3529-3533` wave 2's chain):
            // Next(3) == 0 picks 326, else the scarecrow Next(10).
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let _style = r#ref.next_max(7); // `GetZombieSettings`
            let expect_ty = if r#ref.next_max(3) == 0 {
                326u16
            } else {
                305 + r#ref.next_max(10) as u16
            };
            let kind = NpcKind::moon_wave_kind(expect_ty).expect("a ported wave kind");
            let n = spawn.expect("the ported wave kind must spawn at the found tile");
            assert_eq!(n.kind, kind, "seed {seed}: the wrong kind spawned");
            assert_eq!(
                kind.pumpkin_moon_type(),
                Some(expect_ty),
                "seed {seed}: the u16 → kind map rides the vanilla pick"
            );
            // The `NewNPC` Bottom placement: (spawnTileX * 16 + 8,
            // spawnTileY * 16) is the NPC's bottom-centre.
            let (nw, nh) = kind.size();
            assert_eq!(
                n.x, tile_x as f32 * TILE_PX + 8.0 - nw * 0.5,
                "seed {seed}: the feet sit centred 8 px into the found tile"
            );
            assert_eq!(
                n.y, tile_y as f32 - nh,
                "seed {seed}: the feet stand on the found tile's top edge"
            );
            // The facing toward the player (`TargetClosest`): the ctx player
            // sits left of the tile.
            assert_eq!(n.facing, -1, "seed {seed}: the spawn faces the player");
            assert_eq!(n.hp, n.max_hp, "seed {seed}: the spawn is at full hp");
            // The stream consumed exactly the replay's draws.
            assert_eq!(
                rng.peek(), r#ref.peek(),
                "seed {seed}: the pumpkin spawn's stream tail is the vanilla draw order"
            );
            if expect_ty == 326 {
                saw_splinterling += 1;
            }
            kinds_seen.insert(expect_ty);
        }
        assert!(
            saw_splinterling > 0,
            "the 1/3 326 arm must hit across 200 seeds"
        );
        assert!(
            kinds_seen.len() >= 8,
            "the scarecrow band must spread across its kinds (saw {})",
            kinds_seen.len()
        );
    }

    /// DW-2.4: the boss cap's live counts feed the trio gate — the feed is
    /// `moon_boss_cap_count` over the LIVE 315/325/327 kinds (the dead weigh
    /// nothing, `NPC.cs:274-283`), and at wave 20 the pumpkin trio sits
    /// wholly inside `!reachedInvasionBossCap` (`:3227-3229`): a reached cap
    /// draws nothing and spawns nothing, an open one spawns the ported boss
    /// kinds at the found tile.
    #[test]
    fn the_boss_cap_live_counts_feed_the_pumpkin_trio_gate() {
        // Two live MourningWoods (3.0 each) + one live Headless Horseman
        // (1.0) + one dead Pumpking: the live read is 7 < 11 — the gate is
        // open, and the dead 327 must not weigh its 5.
        let roster = [
            NpcKind::MourningWood,
            NpcKind::MourningWood,
            NpcKind::HeadlessHorseman,
            NpcKind::Pumpking,
        ];
        let npcs: Vec<Npc> = roster
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let mut n = Npc::new(*k, i as f32 * 32.0, 100.0, 1, 0.0);
                if i < 3 {
                    n.hp = n.max_hp;
                } else {
                    n.hp = 0;
                } // the Pumpking stays dead
                n
            })
            .collect();
        let count = moon_boss_cap_count(&npcs);
        assert_eq!(count, 7.0, "the dead boss weighs nothing");
        assert!(!moon_boss_cap_reached(count), "7 < 11 keeps the gate open");
        let w = moon_world(20, false);
        let mut c = ctx(&w, 25.0 * 16.0, false);
        c.moon_boss_cap_count = count; // the live feed (`npcs.rs:4022`)
        let mut saw = std::collections::BTreeSet::new();
        for seed in 1..=150u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let spawn = spawn_npc_at(&c, &mut rng, 210, 25 * 16, false, false, false);
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let _style = r#ref.next_max(7); // `GetZombieSettings` (`:1284`)
            // The wave-20 trio (`:3227-3243`): Next(2) == 0 → 327; else
            // Next(3) != 0 → 325; else the ungated 315 arm (no roll).
            let expect_ty = if r#ref.next_max(2) == 0 {
                327u16
            } else if r#ref.next_max(3) != 0 {
                325
            } else {
                315
            };
            let kind = NpcKind::moon_wave_kind(expect_ty).expect("a ported boss kind");
            let n = spawn.expect("the open gate spawns the ported trio");
            assert_eq!(n.kind, kind, "seed {seed}: the wrong boss kind spawned");
            assert_eq!(
                rng.peek(), r#ref.peek(),
                "seed {seed}: the open-gate stream tail is the vanilla order"
            );
            saw.insert(expect_ty);
        }
        assert_eq!(
            saw,
            [327u16, 325, 315].into_iter().collect(),
            "the trio's three arms must all fire under an open gate"
        );
        // At/over the bound the gate closes: the ladder returns before any
        // draw — the stream holds the style draw and nothing spawns.
        let roster2 = [
            NpcKind::Pumpking,
            NpcKind::Pumpking,
            NpcKind::MourningWood,
            NpcKind::Pumpking,
        ];
        let npcs2: Vec<Npc> = roster2
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let mut n = Npc::new(*k, i as f32 * 32.0, 100.0, 1, 0.0);
                if i < 3 {
                    n.hp = n.max_hp;
                } else {
                    n.hp = 0;
                } // one dead Pumpking
                n
            })
            .collect();
        let count2 = moon_boss_cap_count(&npcs2);
        assert_eq!(count2, 13.0, "10 + 3 live, the dead weighs nothing");
        assert!(moon_boss_cap_reached(count2), "13 >= 11 reaches the cap");
        c.moon_boss_cap_count = count2;
        for seed in 1..=100u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let spawn = spawn_npc_at(&c, &mut rng, 210, 25 * 16, false, false, false);
            assert!(
                spawn.is_none(),
                "seed {seed}: the reached cap spawns nothing"
            );
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let _style = r#ref.next_max(7); // only the style draws
            assert_eq!(
                rng.peek(), r#ref.peek(),
                "seed {seed}: the reached cap draws nothing beyond the style"
            );
        }
        // The plan's edge case: the 325 arm's own `CountNPCS(325) < 2` cap
        // (`:3235`) — two live MourningWoods close the arm while its roll is
        // still drawn (`moon_arm_count`'s explicit count, the C# `&&`
        // order), and the chain advances to the next arm.
        let hit_seed = (1..=100u64)
            .find(|&s| UnifiedRandom::new(s as i32).next_max(2) == 0)
            .unwrap();
        let mut open_rng = UnifiedRandom::new(hit_seed as i32);
        let mut open_pick = None;
        assert!(moon_arm_count(&mut open_pick, &mut open_rng, 3, Some(2), 325, 0));
        assert_eq!(open_pick, Some(325), "the open count lets the arm fire");
        let mut closed_rng = UnifiedRandom::new(hit_seed as i32);
        let mut closed_pick = None;
        assert!(!moon_arm_count(
            &mut closed_pick, &mut closed_rng, 3, Some(2), 325, 2
        ));
        assert_eq!(
            closed_pick, None,
            "two live MourningWoods close the 325 arm"
        );
        assert_eq!(
            closed_rng.peek(), open_rng.peek(),
            "the roll was still drawn (the C# &&)"
        );
    }

    #[test]
    fn day_cycle_zombie_spawn_gate() {
        // `NPC.cs:4199` is the surface day/night fork: while `Main.dayTime`
        // runs the day ladder and the night ladder (`:4520-4810`, the zombie
        // fallthrough) is unreachable, so no zombie may appear by day. At
        // night the same column does produce one, and the gate itself
        // multiplies the rate x0.6 and the cap x1.3 (`:540-544`).
        let w = ground_tall();
        let night = ctx(&w, 295.0 * 16.0, false);
        let day = ctx(&w, 295.0 * 16.0, true);
        let (mut day_zombies, mut night_zombies, mut day_slimes) = (0, 0, 0);
        for seed in 1..2000u64 {
            match outcome(&day, seed) {
                Some((NpcKind::Zombie, _)) => day_zombies += 1,
                Some((NpcKind::Slime, _)) => day_slimes += 1,
                _ => {}
            }
            if matches!(outcome(&night, seed), Some((NpcKind::Zombie, _))) {
                night_zombies += 1;
            }
        }
        assert_eq!(day_zombies, 0, "no zombie may spawn by day");
        assert!(day_slimes > 0, "the day ladder's slime arm still runs");
        assert!(night_zombies > 0, "the night ladder does spawn zombies");
        // The gate's own numbers: 600/5 -> 360 and (float)6.5 -> 6.
        let rate = get_spawn_rate(&night, &mut UnifiedRandom::new(1));
        assert_eq!((rate.spawn_rate, rate.max_spawns), (360, 6));
        let rate = get_spawn_rate(&day, &mut UnifiedRandom::new(1));
        assert_eq!((rate.spawn_rate, rate.max_spawns), (600, 5));
    }

    // ---- W4-f32: the spawner's luck consumers (`ctx.luck`, NPC.cs:369) ----

    #[test]
    fn f32_spawn_rate_luck_tail_is_a_no_draw_at_zero_and_positive_luck() {
        // `NPC.cs:924-928` + `Luck.cs:53-60`: RollOnlyBadLuckExtreme returns
        // -1 WITHOUT a draw when luck >= 0 — the town-block stream and the
        // rate/cap pair are byte-identical to the pre-luck port.
        let w = ground();
        let mut zero = ctx(&w, 20.0 * 16.0, true);
        zero.town_npcs = 1; // enter the town block (a draw-taking arm)
        let mut positive = zero;
        positive.luck = 0.9;
        let mut a = UnifiedRandom::new(7);
        let mut b = UnifiedRandom::new(7);
        let ra = get_spawn_rate(&zero, &mut a);
        let rb = get_spawn_rate(&positive, &mut b);
        assert_eq!((ra.spawn_rate, ra.max_spawns), (rb.spawn_rate, rb.max_spawns));
        assert_eq!(a, b, "luck >= 0 consumes nothing extra in the tail");
    }

    #[test]
    fn f32_spawn_rate_bad_luck_shrinks_rate_and_grows_cap() {
        // Luck -1.0: the gate NextFloat < 1.0 always hits, then
        // Next(50/10) = Next(5); a seed whose Next(5) is 0 takes the tail:
        // rate x0.85 (float cast), cap x1.15 (float cast) — `:926-927`.
        let w = ground();
        let mut c = ctx(&w, 20.0 * 16.0, true);
        c.town_npcs = 1;
        c.luck = -1.0;
        // Find a seed whose town-block draws leave Next(5) == 0 next AND
        // leave spawnFriendly clear (the tail's `!spawnFriendly` gate).
        let mut hit = None;
        for seed in 0..500i32 {
            let mut probe = UnifiedRandom::new(seed);
            let base = get_spawn_rate(&{
                let mut z = c;
                z.luck = 0.0;
                z
            }, &mut probe);
            if base.spawn_friendly {
                continue;
            }
            // After the town block the tail's draws are NextFloat, Next(5).
            let mut tail = probe.clone();
            let _gate = tail.next_float();
            if tail.next_max(5) == 0 {
                hit = Some((seed, base));
                break;
            }
        }
        let (seed, base) = hit.expect("a seed with Next(5) == 0");
        let mut rng = UnifiedRandom::new(seed);
        let r = get_spawn_rate(&c, &mut rng);
        assert_eq!(r.spawn_rate, (base.spawn_rate as f32 * 0.85) as i32);
        assert_eq!(r.max_spawns, (base.max_spawns as f32 * 1.15) as i32);
    }

    #[test]
    fn f32_spawn_rate_luck_tail_skipped_when_spawn_friendly() {
        // `:924`: `!this.spawnFriendly` gates the tail — with the friendly
        // latch set even luck -1.0 draws NOTHING (the && short-circuit).
        let w = ground();
        let mut c = ctx(&w, 20.0 * 16.0, true);
        // town_npcs >= 3 off-underworld: the else arm sets spawnFriendly
        // without a draw (`:916-920`, expert constants-off).
        c.town_npcs = 3;
        c.luck = -1.0;
        let mut zero = c;
        zero.luck = 0.0;
        let mut a = UnifiedRandom::new(11);
        let mut b = UnifiedRandom::new(11);
        let ra = get_spawn_rate(&zero, &mut a);
        let rb = get_spawn_rate(&c, &mut b);
        assert!(rb.spawn_friendly);
        assert_eq!((ra.spawn_rate, ra.max_spawns), (rb.spawn_rate, rb.max_spawns));
        assert_eq!(a, b, "the friendly latch short-circuits the luck tail");
    }

    #[test]
    fn f32_pinky_promotion_is_the_luck_double_roll() {
        // `NPC.cs:5242` through `Luck.RollLuck(180)` (`Luck.cs:5-16`): at
        // luck 1.0 the gate always hits and the range SHRINKS to
        // Next(Next(90, 180)) — three draws (NextFloat, Next(90,180),
        // Next(inner)); at luck 0 it is the single plain Next(180)
        // (byte-identical to the pre-luck port).
        let w = ground_tall(); // width 400, spawn_x 200
        let mut lucky = ctx(&w, 20.0 * 16.0, true);
        lucky.luck = 1.0;
        let plain = ctx(&w, 20.0 * 16.0, true);
        // The Blue pick needs dist >= 200 from spawn_x — x 0 is exactly
        // 200 away, so the 1/3 green roll's miss lands Blue (the 1/10
        // purple arm needs dist > 400, unreachable here).
        let mut probe_seed = None;
        for seed in 0..2000i32 {
            let mut rng = UnifiedRandom::new(seed);
            if let Some(npc) = spawn_basic_slime(&plain, &mut rng, 0, true) {
                if npc.slime == SlimeVariant::Blue {
                    probe_seed = Some(seed);
                    break;
                }
            }
            (trace, rng)
        };
        let (a, rng_a) = run();
        let (b, rng_b) = run();
        assert_eq!(a, b, "the 500-tick run is frozen");
        assert_eq!(rng_a, rng_b, "the two runs draw identically");
        let probe = UnifiedRandom::new(26);
        assert_eq!(rng_a, probe, "the Wolf's 500 ticks draw NOTHING");
    // ---- W4-c38: the moon-roster re-verification witnesses ----

    /// The pumpkin wave-4 quirk (`NPC.cs:3510-3522`): the leading arm rolls
    /// `Next(8) == 0 && !AnyNPCs(325)` but SPAWNS 330 — the cap counts the
    /// GATE kind 325, not the spawned 330 (the pre-fix port counted the
    /// wrong family through `moon_chain`'s generic count-by-kind).
    #[test]
    fn the_pumpkin_wave_4_gate_counts_325_not_the_spawned_330() {
        let w = moon_world(4, false);
        let mut c = ctx(&w, 25.0 * 16.0, false);
        // A seed whose first ladder roll (Next(8)) hits.
        let hit_seed = (1..=100u64)
            .find(|&s| UnifiedRandom::new(s as i32).next_max(8) == 0)
            .unwrap();
        // (a) One live Mourning Wood (325) closes the gate — the roll still
        // draws, the chain advances (the vanilla `&&` order), the second
        // arm's Next(2) decides 326 vs the scarecrow band.
        let roster = [325u16];
        c.moon_npc_counts = &roster;
        let mut rng = UnifiedRandom::new(hit_seed as i32);
        let picked = pumpkin_moon_ladder(&c, &mut rng);
        let mut r#ref = UnifiedRandom::new(hit_seed as i32);
        let _gate = r#ref.next_max(8); // drawn behind the closed gate
        let expect = if r#ref.next_max(2) == 0 {
            Some(326u16)
        } else {
            Some(305 + r#ref.next_max(10) as u16)
        };
        assert_eq!(picked, expect, "the closed 325 gate suppresses the 330 arm");
        assert_ne!(picked, Some(330), "one live 325 refuses the arm");
        assert_eq!(rng.peek(), r#ref.peek(), "the stream replays the C# order");
        // (b) A live Poltergeist (330) does NOT close the gate — the count
        // reads 325, so the arm still fires and spawns 330.
        let roster2 = [330u16];
        c.moon_npc_counts = &roster2;
        let mut rng2 = UnifiedRandom::new(hit_seed as i32);
        assert_eq!(
            pumpkin_moon_ladder(&c, &mut rng2),
            Some(330),
            "a live 330 leaves the !AnyNPCs(325) gate open"
        );
        let mut ref2 = UnifiedRandom::new(hit_seed as i32);
        let _ = ref2.next_max(8);
        assert_eq!(rng2.peek(), ref2.peek(), "the hit consumes only its Next(8)");
        // (c) The open gate fires 330 on the same seed.
        c.moon_npc_counts = &[];
        let mut rng3 = UnifiedRandom::new(hit_seed as i32);
        assert_eq!(pumpkin_moon_ladder(&c, &mut rng3), Some(330));
    }

    /// The separate-327 quirk (`NPC.cs:3263-3278`/`:3281-3300`/`:3325-3344`/
    /// `:3347-3374`, cases 18/17/15/14): the leading 327 `if` is its own
    /// statement — its roll ALWAYS draws even when it fires, and the chain
    /// after it draws too. The picker keeps the first pick (the C#'s second
    /// spawn is the documented deviation, ANALYSIS "the pumpkin-327 quirk");
    /// the stream must stay 1:1 with BOTH rolls consumed.
    #[test]
    fn the_pumpkin_327_lead_draws_even_when_it_fires() {
        // Case 18: Next(7)==0 && CountNPCS(327)<2 → 327, then the chain
        // Next(7) 325 / Next(7) 315 / else 330 — all rolls drawn.
        let hit_seed = (1..=100u64)
            .find(|&s| UnifiedRandom::new(s as i32).next_max(7) == 0)
            .unwrap();
        let w = moon_world(18, false);
        let c = ctx(&w, 25.0 * 16.0, false);
        let mut rng = UnifiedRandom::new(hit_seed as i32);
        assert_eq!(pumpkin_moon_ladder(&c, &mut rng), Some(327));
        let mut r#ref = UnifiedRandom::new(hit_seed as i32);
        let _lead = r#ref.next_max(7); // the 327 arm — fired
        let n325 = r#ref.next_max(7); // the chain STILL draws (`:3267`)
        if n325 != 0 {
            let _n315 = r#ref.next_max(7); // `:3271` (the count gates are open)
        }
        assert_eq!(
            rng.peek(), r#ref.peek(),
            "the fired 327 lead does not skip the chain's rolls"
        );
        // A seed sweep: the full stream (lead + chain) replays per seed.
        for seed in 1..=100u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let _ = pumpkin_moon_ladder(&c, &mut rng);
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let lead = r#ref.next_max(7);
            let n325 = r#ref.next_max(7);
            if n325 != 0 {
                let n315 = r#ref.next_max(7);
                let _ = (lead, n315); // both arms' rolls consumed either way
            }
            assert_eq!(rng.peek(), r#ref.peek(), "seed {seed}: lead + chain");
        }
    }

    /// The frost wave-14 chain (`NPC.cs:2958-2975`) has NO final `else`:
    /// when all four arms miss their rolls the C# spawns nothing — and the
    /// stream holds exactly the four Next(10)/Next(10)/Next(10)/Next(3).
    #[test]
    fn the_frost_wave_14_all_miss_spawns_nothing() {
        let w = moon_world(14, true);
        let c = ctx(&w, 25.0 * 16.0, false);
        let mut saw_none = 0;
        for seed in 1..=200u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let picked = frost_moon_ladder(&c, &mut rng);
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let head = r#ref.next_max(30); // the 341 head (`:2807`)
            let expect = if head == 0 {
                Some(341u16)
            } else {
                let a = r#ref.next_max(10); // 345 (`:2959`)
                if a == 0 {
                    Some(345)
                } else {
                    let b = r#ref.next_max(10); // 346 (`:2963`)
                    if b == 0 {
                        Some(346)
                    } else {
                        let d = r#ref.next_max(10); // 344 (`:2967`)
                        if d == 0 {
                            Some(344)
                        } else if r#ref.next_max(3) == 0 {
                            Some(343) // `:2971`
                        } else {
                            None // the missing `else` (`:2975` break)
                        }
                    }
                }
            };
            assert_eq!(picked, expect, "seed {seed}: the wave-14 chain");
            if expect.is_none() {
                saw_none += 1;
            }
            assert_eq!(rng.peek(), r#ref.peek(), "seed {seed}: the draw order");
        }
        assert!(saw_none > 0, "the all-miss no-spawn outcome must occur");
    }

    /// The frost wave-12 fallback (`NPC.cs:3019-3021`): the final `else`
    /// spawns `Main.rand.Next(338, 341)` — the DRAW ITSELF is the type
    /// (338/339/340 the Gingerbread family), one extra slot consumed.
    #[test]
    fn the_frost_wave_12_fallback_draw_is_the_gingerbread_type() {
        let w = moon_world(12, true);
        let c = ctx(&w, 25.0 * 16.0, false);
        let mut band = std::collections::BTreeSet::new();
        for seed in 1..=300u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            let picked = frost_moon_ladder(&c, &mut rng);
            let mut r#ref = UnifiedRandom::new(seed as i32);
            let head = r#ref.next_max(30);
            let expect = if head == 0 {
                Some(341u16)
            } else if r#ref.next_max(10) == 0 {
                Some(345) // `:3003`
            } else if r#ref.next_max(10) == 0 {
                Some(344) // `:3007`
            } else if r#ref.next_max(8) == 0 {
                Some(343) // `:3011`
            } else if r#ref.next_max(3) == 0 {
                Some(342) // `:3015`
            } else {
                Some(338 + r#ref.next_max(3) as u16) // `:3021`
            };
            assert_eq!(picked, expect, "seed {seed}: the wave-12 chain");
            assert_eq!(rng.peek(), r#ref.peek(), "seed {seed}: the draw order");
            if let Some(ty @ 338..=340) = picked {
                band.insert(ty);
            }
        }
        assert_eq!(
            band,
            [338u16, 339, 340].into_iter().collect(),
            "the Next(338, 341) band spreads across its three kinds"
        );

        }        let seed = probe_seed.expect("a Blue pick in the sweep");
        let mut a = UnifiedRandom::new(seed);
        let mut b = UnifiedRandom::new(seed);
        let na = spawn_basic_slime(&plain, &mut a, 0, true).unwrap();
        let nb = spawn_basic_slime(&lucky, &mut b, 0, true).unwrap();
        assert_eq!(na.slime, SlimeVariant::Blue);
        assert!(matches!(nb.slime, SlimeVariant::Blue | SlimeVariant::Pinky));
        // The promotion draw SHAPES, pinned directly on the engine:
        let mut t0 = UnifiedRandom::new(9);
        let r0 = tr_core::loot::roll_luck(&mut t0, 0.0, 180);
        let mut t0b = UnifiedRandom::new(9);
        assert_eq!(r0, t0b.next_max(180));
        assert_eq!(t0, t0b, "luck 0 == plain Next(180), one draw");
        let mut t1 = UnifiedRandom::new(9);
        let r1 = tr_core::loot::roll_luck(&mut t1, 1.0, 180);
        let mut t1b = UnifiedRandom::new(9);
        let _gate = t1b.next_float();
        let inner = t1b.next_range(90, 180);
        assert_eq!(r1, t1b.next_range(0, inner));
        assert_eq!(t1, t1b, "luck 1 == the three-draw shrunk shape");
        assert!(r1 < 180);
        // One vs three promotion draws: the twin streams diverge.
        assert_ne!(a, b, "the luck feeds change the promotion draw shape");
    fn f32_bound_goblin_roll_rides_luck() {
        // `NPC.cs:2084`: downedGoblins && RollLuck(20) == 0 — at luck 0 the
        // arm's roll is the plain Next(20) (byte-identical to the pre-luck
        // port); at luck 1.0 it is the three-draw shrunk shape. Pin both
        // on the same post-ladder stream state: find a seed where the
        // plain arm MISSES, replay the arm's draws, and compare.
        let mut w = ground();
        w.rock_layer = 40.0;
        let mut lucky = ctx(&w, 100.0 * 16.0, true);
        lucky.bound_goblin_ok = true;
        lucky.luck = 1.0;
        let mut plain = lucky;
        plain.luck = 0.0;
        // The arm fires on some seed at luck 0 (the plain cadence).
        let mut fired = false;
        for seed in 0..3000i32 {
            let mut rng = UnifiedRandom::new(seed);
            if let Some(npc) = spawn_npc_at(&plain, &mut rng, 30, 100 * 16, false, false, false) {
                if npc.kind == NpcKind::BoundGoblin {
                    fired = true;
                    // The twin at luck 1.0 consumes a different shape.
                    let mut lucky_rng = UnifiedRandom::new(seed);
                    let _ = spawn_npc_at(&lucky, &mut lucky_rng, 30, 100 * 16, false, false, false);
                    assert_ne!(rng, lucky_rng, "the luck feed changes the roll shape");
                    break;
        assert!(fired, "the plain cadence still spawns the bound goblin");
        // The roll shapes themselves (the engine pin):
        let mut a = UnifiedRandom::new(9);
        let r0 = tr_core::loot::roll_luck(&mut a, 0.0, 20);
        let mut b = UnifiedRandom::new(9);
        assert_eq!(r0, b.next_max(20));
        assert_eq!(a, b, "luck 0 == plain Next(20)");
        let mut c = UnifiedRandom::new(9);
        let r1 = tr_core::loot::roll_luck(&mut c, 1.0, 20);
        let mut d = UnifiedRandom::new(9);
        let _gate = d.next_float();
        let inner = d.next_range(10, 20);
        assert_eq!(r1, d.next_range(0, inner));
        assert_eq!(c, d, "luck 1 == the shrunk three-draw shape");
    fn f32_blood_moon_bad_luck_roll_shrinks_at_negative_luck() {
        // `NPC.cs:4618/4623`: RollOnlyBadLuck(300) — negative luck replaces
        // Next(300) with Next(Next(150, 300)); at luck 0 the stream is the
        // plain Next(300) (byte-identical to the pre-luck port).
        let mut a = UnifiedRandom::new(5);
        let mut b = UnifiedRandom::new(5);
        let zero = tr_core::loot::roll_only_bad_luck(&mut a, 0.0, 300);
        let plain = b.next_max(300);
        assert_eq!(zero, plain, "luck 0 degenerates to the plain roll");
        assert_eq!(a, b, "and consumes exactly one draw");
        // Luck -1.0: gate hits, inner Next(150, 300), outer Next(inner).
        let mut c = UnifiedRandom::new(5);
        let bad = tr_core::loot::roll_only_bad_luck(&mut c, -1.0, 300);
        let mut d = UnifiedRandom::new(5);
        let inner = d.next_range(150, 300);
        let want = d.next_range(0, inner);
        assert_eq!(bad, want);
        assert_eq!(c, d);
    }

// balance-close (union shared delimiter)
}}}}
}
