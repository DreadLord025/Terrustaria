//! W4-c40 — the damaging storm-lightning strike (`WorldGen.
//! SpawnStormLightning`, `WorldGen.cs:71864-72018`, called from
//! `UpdateWorld` at `:71535`). The flash+thunder chain (`rain.rs::
//! thunder_step`) already landed; THIS is the damaging bolt it stood in
//! for: the per-player strike roll, the attractor scan, the safe-spot /
//! rain-shelter / nearby-bolt gates, and the `LightningGenerator.
//! CanHitTarget` geometry gate before the 1091 StormLightning spawn
//! (500 dmg, `WorldGen.cs:72016`).
//!
//! C# sources read WHOLE (the golden rule):
//! - `SpawnStormLightning` (`:71864-71922`): the seed gates, the
//!   `AnyDanger` ×1.5 / moreLightning ÷2 interval, the 255-player
//!   safe-spot rects, then the per-player `Next(num3)` strike roll.
//! - `SpawnStormLightningNearPlayer` (`:71924-71962`): the
//!   `MaxWorldViewSize/32` inflated view rect clamped to the world, the
//!   400-attempt attractor scan (350 attractor-only, then 50
//!   solid-or-liquid), the upward attractor walk, the liquid-above gate.
//! - `IsLightningAttractor` (`:71964-71975`): `nactive &&
//!   (GetsCheckedForLeaves || AttractsStormLightning)`.
//! - `TrySpawnLightningBolt(safespots, x, y, ref skip)` (`:71977-72001`):
//!   the `IsSafeFromRain(x, y-1)` shelter gate, the safe-spot contains,
//!   the 1000-projectile 800-px nearby-1091 scan.
//! - `TrySpawnLightningBolt(x, y)` (`:72003-72018`): the
//!   `Next(MaxFloatInt+1)` seed, the liquid surface offset, the
//!   `CanHitTarget` gate, the `NewProjectile(..., 1091, 500, 0f,
//!   Main.myPlayer, -1f, 0f, num)` spawn.
//! - `IsSafeFromRain` (`:72960-72968`): the `PlotTileLine` cast up the
//!   rain fall velocity, 85 tiles, width 4, `CheckStopForSolids`.
//!
//! RNG contract: every draw is `Main.rand` under the `UpdateWorldGen`
//! swap (`Main.cs:17802` wraps `UpdateWorld_WorldGenAndInvasion` →
//! `WorldGen.UpdateWorld` → `:71535`). The port binds the
//! `RngKey::UpdateWorldGen` named stream. Draw order per player:
//! `Next(num3)` (the strike roll), then on a hit the 400-attempt
//! `Next(left, right+1)` / `Next(top, bottom+1)` pairs, then on a
//! candidate the `Next(MaxFloatInt+1)` seed. The geometry's LCG draws
//! are PRIVATE (not this stream — `lightning_generator.rs`).
//!
//! The 1091 projectile AI (aiStyle 203) + the `Kill()` strike are the
//! `projectile_storm_lightning` module; this module only spawns the
//! request (the pool seam). The bolt VISUAL (the particle orchestrator)
//! is the render lane — not drawn here.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{UnifiedRandom, World};

use crate::lightning_generator::LightningGenerator;

/// `ProjectileID.StormLightning` (`ProjectileID.cs:2736`).
pub const STORM_LIGHTNING: u16 = 1091;

/// `Utils.MaxFloatInt` (`Utils.cs:2859`) — the seed draw's exclusive max
/// is `MaxFloatInt + 1`.
const MAX_FLOAT_INT: i32 = 16777216;

/// `Main.MaxWorldViewSize` (`Main.cs:66714`) — 1920×1200.
const MAX_WORLD_VIEW: (i32, i32) = (1920, 1200);

/// `Player.AFKTimeNeededForNoLuckyStars` (`Player.cs:57103`).
const AFK_TIME_NEEDED: i32 = 10800;

/// One spawned bolt request — the `NewProjectile` arguments of
/// `WorldGen.cs:72016` (position is the C# CENTRE; the pool seam
/// re-centres to the port's top-left). `seed` is `ai[2]`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LightningSpawn {
    pub center: Vec2,
    pub seed: i32,
}

/// The per-player inputs the strike roll reads (`Main.player[i]`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct StrikePlayer {
    pub active: bool,
    pub dead: bool,
    /// `player.Center` in world pixels.
    pub center: Vec2,
    /// `player.velocity.X` — the safe-spot lead.
    pub vx: f32,
    pub zone_rain: bool,
    pub zone_snow: bool,
    pub afk_counter: i32,
    pub sleeping: bool,
    /// `player.HeldItem.fishingPole > 0`.
    pub holding_fishing_pole: bool,
}

/// The cross-system inputs `SpawnStormLightning` reads off `Main`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StormInputs {
    pub no_lightning: bool,
    pub more_lightning: bool,
    /// `Main.IsItStorming` (`Main.cs:2905`, `_shouldUseStormMusic`).
    pub storming: bool,
    /// `Main.IsItRaining`.
    pub raining: bool,
    /// `Main.cloudAlpha` — the moreLightning-without-storm divisor.
    pub cloud_alpha: f32,
    /// `NPC.AnyDanger()` (`NPC.cs:80243`).
    pub any_danger: bool,
    /// `Main.windSpeedCurrent` — the rain fall velocity's X.
    pub wind_speed_current: f32,
}

/// `WorldGen.SpawnStormLightning` (`WorldGen.cs:71864-71922`). Returns
/// the bolt spawn requests (the caller feeds the pool). `players` is the
/// 255-slot roster (the port's single player + inactive padding);
/// `existing_bolts` is the active 1091 centres (the nearby-bolt scan).
pub(crate) fn spawn_storm_lightning(
    world: &World,
    inputs: StormInputs,
    players: &[StrikePlayer],
    existing_bolts: &[Vec2],
    rng: &mut UnifiedRandom,
) -> Vec<LightningSpawn> {
    let mut out = Vec::new();
    // `:71866-71869`: noLightning alone suppresses the storm lightning.
    if inputs.no_lightning && !inputs.more_lightning {
        return out;
    }
    // `:71870`: the moreLightning-only flag.
    let flag = inputs.more_lightning && !inputs.no_lightning;
    // `:71871-71874`: storming, or (moreLightning && raining).
    if !inputs.storming && (!flag || !inputs.raining) {
        return out;
    }
    // `:71875-71888`: the base interval.
    let mut num = 150i32;
    if inputs.any_danger {
        num = (1.5 * num as f64) as i32; // 225
    }
    if flag {
        let mut num2 = 2.0f32;
        if !inputs.storming {
            num2 *= inputs.cloud_alpha;
        }
        num = (num as f32 / num2) as i32;
    }
    // `:71889-71901`: the safe-spot rects — one per active living player,
    // the 24×24 tile rect around the player's tile centre, unioned with
    // its velocity-lead offset.
    let safespots = safe_spots(players);
    // `:71902-71921`: the per-player strike roll.
    for p in players {
        if !(p.active && !p.dead && p.zone_rain && !p.zone_snow && p.afk_counter < AFK_TIME_NEEDED) {
            continue;
        }
        let mut num3 = num;
        if p.sleeping {
            num3 *= 2;
        }
        if p.holding_fishing_pole {
            num3 *= 2;
        }
        // `:71916`: `Main.rand.Next(num3) == 0`.
        if rng.next_max(num3) == 0 {
            spawn_near_player(world, &safespots, p, existing_bolts, inputs.wind_speed_current, rng, &mut out);
        }
    }
    out
}

/// The safe-spot rects (`WorldGen.cs:71891-71901`): the 24×24
/// `CenteredRectangle` around each active living player's tile centre,
/// unioned with the same rect offset by the clamped velocity lead
/// `(int)(Clamp(vx, -5.75, 5.75) * 60 / 16)`. Stored as (x, y, w, h) in
/// TILE coordinates.
fn safe_spots(players: &[StrikePlayer]) -> Vec<(i32, i32, i32, i32)> {
    let mut out = Vec::new();
    for p in players {
        if !(p.active && !p.dead) {
            continue;
        }
        let (cx, cy) = to_tile(p.center);
        // CenteredRectangle(Point(24,24), center): integer halving.
        let (rx, ry) = (cx - 24 / 2, cy - 24 / 2);
        let lead = (p.vx.clamp(-5.75, 5.75) * 60.0 / 16.0) as i32;
        // Rectangle.Union of (rx, ry, 24, 24) and (rx+lead, ry, 24, 24).
        let (x0, x1) = (rx.min(rx + lead), (rx + 24).max(rx + lead + 24));
        let (y0, y1) = (ry, ry + 24);
        out.push((x0, y0, x1 - x0, y1 - y0));
    }
    out
}

/// `WorldGen.SpawnStormLightningNearPlayer` (`WorldGen.cs:71924-71962`).
fn spawn_near_player(
    world: &World,
    safespots: &[(i32, i32, i32, i32)],
    player: &StrikePlayer,
    existing_bolts: &[Vec2],
    wind: f32,
    rng: &mut UnifiedRandom,
    out: &mut Vec<LightningSpawn>,
) {
    // `:71926-71928`: the view rect — CenteredRectangle(center, Point.Zero)
    // inflated by MaxWorldViewSize/32, clamped to the world with fluff 10.
    let (cx, cy) = to_tile(player.center);
    let (hw, hh) = (MAX_WORLD_VIEW.0 / 32, MAX_WORLD_VIEW.1 / 32); // 60, 37
    let rect = clamp_to_world((cx - hw, cy - hh, hw * 2, hh * 2), world, 10);
    let (left, top, w, h) = rect;
    if w == 0 || h == 0 {
        return;
    }
    let (right, bottom) = (left + w, top + h);
    // `:71933-71961`: the 400-attempt scan.
    for i in 0..400 {
        // `:71935-71936`: Next(Left, Right+1) / Next(Top, Bottom+1).
        let x = rng.next_range(left, right + 1);
        let mut y = rng.next_range(top, bottom + 1);
        // `:71938-71941`: walk up while the tile ABOVE is an attractor.
        while y > 0 && is_lightning_attractor(world, x, y - 1) {
            y -= 1;
        }
        // `:71942-71952`: the first 350 attempts need an attractor; the
        // last 50 accept a solid-or-liquid tile.
        if i < 350 {
            if !is_lightning_attractor(world, x, y) {
                continue;
            }
        } else {
            let t = world.tile(x, y);
            let liquid = t.map(|t| t.liquid).unwrap_or(0);
            let solid = crate::projectile::solid_tile(world, x, y);
            if liquid == 0 && !solid {
                continue;
            }
        }
        // `:71953`: the tile above must have no liquid.
        let above_liquid = world.tile(x, y - 1).map(|t| t.liquid).unwrap_or(0);
        if y <= 0 || above_liquid == 0 {
            // `:71955-71959`: TrySpawnLightningBolt — a spawn OR the
            // nearby-bolt skip both break the scan.
            let mut skip = false;
            if try_spawn_bolt(world, safespots, x, y, existing_bolts, wind, rng, out, &mut skip) || skip {
                break;
            }
        }
    }
}

/// `WorldGen.IsLightningAttractor` (`WorldGen.cs:71964-71975`):
/// `nactive && (GetsCheckedForLeaves[type] || AttractsStormLightning[type])`.
fn is_lightning_attractor(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !(t.active() && !t.inactive()) {
        return false;
    }
    td::SET_GETS_CHECKED_FOR_LEAVES.has(t.kind) || td::SET_ATTRACTS_STORM_LIGHTNING.has(t.kind)
}

/// `WorldGen.TrySpawnLightningBolt(safespots, x, y, ref skip)`
/// (`WorldGen.cs:71977-72001`). Returns true on a spawn; sets `skip` when
/// a nearby 1091 suppresses this player's strike.
#[allow(clippy::too_many_arguments)]
fn try_spawn_bolt(
    world: &World,
    safespots: &[(i32, i32, i32, i32)],
    x: i32,
    y: i32,
    existing_bolts: &[Vec2],
    wind: f32,
    rng: &mut UnifiedRandom,
    out: &mut Vec<LightningSpawn>,
    skip: &mut bool,
) -> bool {
    // `:71979-71982`: the rain-shelter gate.
    if is_safe_from_rain(world, x, y - 1, wind) {
        return false;
    }
    // `:71983-71989`: the safe-spot contains.
    for &(sx, sy, sw, sh) in safespots {
        if x >= sx && x < sx + sw && y >= sy && y < sy + sh {
            return false;
        }
    }
    // `:71990-71999`: the nearby-1091 scan (800 px).
    let other = Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 2.0);
    for &c in existing_bolts {
        if c.distance(other) < 800.0 {
            *skip = true;
            return false;
        }
    }
    // `:72000`: the geometry gate + spawn.
    try_spawn_bolt_at(world, x, y, rng, out)
}

/// `WorldGen.TrySpawnLightningBolt(x, y)` (`WorldGen.cs:72003-72018`).
fn try_spawn_bolt_at(
    world: &World,
    x: i32,
    y: i32,
    rng: &mut UnifiedRandom,
    out: &mut Vec<LightningSpawn>,
) -> bool {
    // `:72005`: the seed draw — Next(MaxFloatInt + 1).
    let num = rng.next_max(MAX_FLOAT_INT + 1);
    // `:72006-72010`: the liquid surface offset.
    let mut num2 = 2i32;
    let liquid = world.tile(x, y).map(|t| t.liquid as i32).unwrap_or(0);
    if liquid > 0 {
        num2 += 16 * (255 - liquid) / 255;
    }
    let vector = Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + num2 as f32);
    // `:72012-72015`: the geometry gate.
    if !LightningGenerator::storm_lightning().can_hit_target(world, num as u32, vector) {
        return false;
    }
    // `:72016`: NewProjectile(..., 1091, 500, 0f, Main.myPlayer, -1f, 0f, num).
    out.push(LightningSpawn { center: vector, seed: num });
    true
}

/// `WorldGen.IsSafeFromRain` (`WorldGen.cs:72960-72968`): cast a width-4
/// tile line from the tile centre up the rain fall velocity for 85 tiles;
/// a solid tile on the line shelters the spot.
fn is_safe_from_rain(world: &World, start_x: i32, start_y: i32, wind: f32) -> bool {
    // `Rain.GetRainFallVelocity()` (`Rain.cs:91-94`): (wind*18, 14).
    let fall = Vec2::new(wind * 18.0, 14.0);
    // `-fall.SafeNormalize((0,1))` — the up-rain direction.
    let dir = -safe_normalize(fall, Vec2::Y);
    let start = Vec2::new(start_x as f32 * 16.0 + 8.0, start_y as f32 * 16.0 + 8.0);
    let end = start + dir * 16.0 * 85.0;
    // `Utils.PlotTileLine(start, end, 4, CheckStopForSolids)` — the result
    // is `DelegateMethods.CheckResultOut`, set true when a solid is hit.
    let mut hit = false;
    plot_tile_line(start, end, 4.0, &mut |x, y| {
        // `DelegateMethods.CheckStopForSolids` (`DelegateMethods.cs:743-759`):
        // out-of-world/null → false (abort); a solid → set the flag, false;
        // else true (keep plotting).
        if x < 0 || x >= world.width || y < 0 || y >= world.height {
            return false;
        }
        let Some(t) = world.tile(x, y) else {
            return false;
        };
        if t.active()
            && !t.inactive()
            && !td::TILE_SOLID_TOP.has(t.kind)
            && td::TILE_SOLID.has(t.kind)
        {
            hit = true;
            return false;
        }
        true
    });
    hit
}

/// `Utils.PlotTileLine` (`Utils.cs:2337-2350`): the Bresenham centre line
/// with the half-width perpendicular offset band. `plot` returns false to
/// abort (the C# `TileActionAttempt` convention).
fn plot_tile_line(start: Vec2, end: Vec2, width: f32, plot: &mut dyn FnMut(i32, i32) -> bool) -> bool {
    let half = (width / 2.0) as f64;
    let delta = end - start;
    let unit = delta / delta.length();
    // The perpendicular offset (−unit.Y, unit.X) * half.
    let perp = Vec2::new(-unit.y, unit.x) * half as f32;
    let p_min = to_tile(start - perp);
    let p_max = to_tile(start + perp);
    let p_start = to_tile(start);
    let p_end = to_tile(end);
    let min_off = (p_min.0 - p_start.0, p_min.1 - p_start.1);
    let max_off = (p_max.0 - p_start.0, p_max.1 - p_start.1);
    plot_line(p_start.0, p_start.1, p_end.0, p_end.1, true, &mut |x, y| {
        plot_line(
            x + min_off.0,
            y + min_off.1,
            x + max_off.0,
            y + max_off.1,
            false,
            plot,
        )
    })
}

/// `Utils.PlotLine` (`Utils.cs:2238-2292`) — the Bresenham walk with the
/// `jump: false` double-plot on the minor-axis step (the `npc_ai_town`
/// shape). `plot` returns false to abort.
fn plot_line(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    jump: bool,
    plot: &mut dyn FnMut(i32, i32) -> bool,
) -> bool {
    if x0 == x1 && y0 == y1 {
        return plot(x0, y0);
    }
    let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);
    let flag = (y1 - y0).abs() > (x1 - x0).abs();
    if flag {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    let num = (x1 - x0).abs();
    let num2 = (y1 - y0).abs();
    let mut num3 = num / 2;
    let mut num4 = y0;
    let num5 = if x0 < x1 { 1 } else { -1 };
    let num6 = if y0 < y1 { 1 } else { -1 };
    let mut i = x0;
    while i != x1 {
        if flag {
            if !plot(num4, i) {
                return false;
            }
        } else if !plot(i, num4) {
            return false;
        }
        num3 -= num2;
        if num3 >= 0 {
            i += num5;
            continue;
        }
        num4 += num6;
        if !jump {
            if flag {
                if !plot(num4, i) {
                    return false;
                }
            } else if !plot(i, num4) {
                return false;
            }
        }
        num3 += num;
        i += num5;
    }
    true
}

/// `WorldUtils.ClampToWorld` (`WorldBuilding/WorldUtils.cs:9-14`): clamp
/// each edge to `[fluff, maxTiles - fluff]`, rebuild the rect.
fn clamp_to_world(rect: (i32, i32, i32, i32), world: &World, fluff: i32) -> (i32, i32, i32, i32) {
    let (x, y, w, h) = rect;
    let (right, bottom) = (x + w, y + h);
    let nx = fluff.max(x.min(world.width - fluff));
    let ny = fluff.max(y.min(world.height - fluff));
    let nr = fluff.max(right.min(world.width - fluff));
    let nb = fluff.max(bottom.min(world.height - fluff));
    (nx, ny, nr - nx, nb - ny)
}

/// `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4`.
fn to_tile(v: Vec2) -> (i32, i32) {
    ((v.x as i32) >> 4, (v.y as i32) >> 4)
}

/// `Utils.SafeNormalize` (`Utils.cs:1837-1844`): the zero/NaN fallback.
fn safe_normalize(v: Vec2, default: Vec2) -> Vec2 {
    if v == Vec2::ZERO || v.x.is_nan() || v.y.is_nan() {
        default
    } else {
        v.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 400, 400)
    }

    fn player(center: Vec2) -> StrikePlayer {
        StrikePlayer {
            active: true,
            dead: false,
            center,
            vx: 0.0,
            zone_rain: true,
            zone_snow: false,
            afk_counter: 0,
            sleeping: false,
            holding_fishing_pole: false,
        }
    }

    fn inputs(storming: bool) -> StormInputs {
        StormInputs {
            no_lightning: false,
            more_lightning: false,
            storming,
            raining: storming,
            cloud_alpha: 1.0,
            any_danger: false,
            wind_speed_current: 0.0,
        }
    }

    /// The seed gates (`:71866-71874`): noLightning alone suppresses;
    /// no storm + no moreLightning suppresses; moreLightning relaxes to
    /// raining. No draws on a suppressed call (the stream stays put).
    #[test]
    fn w4_c40_the_seed_gates_suppress_before_any_draw() {
        let w = air();
        let p = player(Vec2::new(3200.0, 3200.0));
        let mut rng = UnifiedRandom::new(42);
        let pristine = rng;
        // noLightning alone: suppressed.
        let mut i = inputs(true);
        i.no_lightning = true;
        assert!(spawn_storm_lightning(&w, i, &[p], &[], &mut rng).is_empty());
        assert_eq!(rng, pristine, "no draw on the noLightning gate");
        // No storm, no moreLightning: suppressed.
        assert!(spawn_storm_lightning(&w, inputs(false), &[p], &[], &mut rng).is_empty());
        assert_eq!(rng, pristine, "no draw on the storm gate");
        // moreLightning + raining (no storm): the gate passes (a draw runs).
        let mut i = inputs(false);
        i.more_lightning = true;
        i.raining = true;
        let _ = spawn_storm_lightning(&w, i, &[p], &[], &mut rng);
        assert_ne!(rng, pristine, "the moreLightning rain arm draws");
    }

    /// `IsLightningAttractor` (`:71964-71975`): an active tree (5,
    /// GetsCheckedForLeaves) attracts; an active solid dirt (1) does not
    /// (it is not in AttractsStormLightning); an actuated tree does not.
    #[test]
    fn w4_c40_the_attractor_reads_the_leaf_and_attractor_sets() {
        let mut w = air();
        let idx = w.index(100, 100);
        // A tree trunk (5) is in GetsCheckedForLeaves.
        w.tiles[idx].kind = 5;
        w.tiles[idx].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(is_lightning_attractor(&w, 100, 100));
        // Actuated: nactive false.
        w.tiles[idx].flags |= tr_core::tile::FLAG_INACTIVE;
        assert!(!is_lightning_attractor(&w, 100, 100));
        // Plain dirt (1): solid but not an attractor.
        w.tiles[idx].flags &= !tr_core::tile::FLAG_INACTIVE;
        w.tiles[idx].kind = 1;
        assert!(!is_lightning_attractor(&w, 100, 100));
        // A torch (4) is in AttractsStormLightning.
        w.tiles[idx].kind = 4;
        assert!(is_lightning_attractor(&w, 100, 100));
    }

    /// `IsSafeFromRain` (`:72960-72968`): a solid roof above the spot
    /// shelters it; open air does not. The cast goes UP the rain velocity.
    #[test]
    fn w4_c40_the_rain_shelter_cast_finds_a_solid_roof() {
        let mut w = air();
        // Open air: no shelter.
        assert!(!is_safe_from_rain(&w, 200, 200, 0.0));
        // A solid roof at (200, 190) directly above shelters the spot.
        let idx = w.index(200, 190);
        w.tiles[idx].kind = 1;
        w.tiles[idx].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(is_safe_from_rain(&w, 200, 200, 0.0), "the roof shelters");
        // Removing the roof opens the sky again.
        w.tiles[idx].flags &= !tr_core::tile::FLAG_ACTIVE;
        assert!(!is_safe_from_rain(&w, 200, 200, 0.0));
    }

    /// The nearby-1091 scan (`:71990-71999`): a bolt within 800 px sets
    /// the skip flag; one beyond does not. The scan centre is
    /// (x*16+8, y*16+2).
    #[test]
    fn w4_c40_the_nearby_bolt_scan_skips_within_800px() {
        let w = air();
        let safespots = [];
        let mut rng = UnifiedRandom::new(1);
        let mut out = Vec::new();
        let mut skip = false;
        // A bolt 100 px from the candidate (100, 100) → skip.
        let near = Vec2::new(100.0 * 16.0 + 8.0 + 100.0, 100.0 * 16.0 + 2.0);
        let spawned = try_spawn_bolt(&w, &safespots, 100, 100, &[near], 0.0, &mut rng, &mut out, &mut skip);
        assert!(!spawned && skip, "the nearby bolt skips");
        // A bolt 2000 px away → no skip (the geometry gate then decides).
        skip = false;
        let far = Vec2::new(100.0 * 16.0 + 8.0 + 2000.0, 100.0 * 16.0 + 2.0);
        let _ = try_spawn_bolt(&w, &safespots, 100, 100, &[far], 0.0, &mut rng, &mut out, &mut skip);
        assert!(!skip, "the far bolt does not skip");
    }

    /// `ClampToWorld` (`WorldUtils.cs:9-14`): a rect straddling the world
    /// edge clamps each edge to [fluff, max-fluff].
    #[test]
    fn w4_c40_the_view_rect_clamps_to_the_world_fluff() {
        let w = air(); // 400×400
        // A rect centred near the top-left corner, fluff 10.
        let r = clamp_to_world((-50, -50, 120, 74), &w, 10);
        assert_eq!(r, (10, 10, 110, 64), "left/top clamp to 10");
        // A rect fully inside stays put.
        let r = clamp_to_world((100, 100, 120, 74), &w, 10);
        assert_eq!(r, (100, 100, 120, 74));
    }
}
