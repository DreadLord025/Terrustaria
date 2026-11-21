//! W4-c40 — the storm-lightning bolt geometry (`GameContent/
//! LightningGenerator.cs:9-326`). The `StormLightning` static preset
//! (`:285-304`) drives `WorldGen.TrySpawnLightningBolt`'s
//! `CanHitTarget` gate (`:243-246`): a bolt is generated from a seeded
//! source point toward the target, and the spawn is refused when the
//! MAIN bolt's path collides with a solid tile or liquid on the way.
//!
//! C# sources read WHOLE (the golden rule):
//! - `Generate` (`:64-76`): the source direction is `Vector2.UnitY`
//!   rotated by `(LCG32Random(seed).NextDouble() * 2 - 1) *
//!   SourceRotationLimit` then scaled by `Length`; the main bolt runs
//!   from `target - direction` to `target`.
//! - `GenerateBolt` (`:78-162`): the layered random-walk — `Layers`
//!   rotation accumulators, the perpendicular-deviation steering term,
//!   the `PickLayerToReroll`/`PickHighLayerToReroll` geometric layer
//!   picks, the fork recursion (only when a `bolts` list is supplied)
//!   and the `SolidTileCollision` break.
//! - `TileCollision` (`:164-181`): `SolidOrSlopedTile` OR a liquid whose
//!     surface the point dips below (`(int)pos.Y % 16 > 16*(255-liquid)/255`).
//! - `PickLayerToReroll` (`:183-194`) / `PickHighLayerToReroll`
//!   (`:196-204`): the `r >= 1-chance` then `r /= chance` walk.
//! - `CanHitTarget` (`:243-246`): `!Generate(null, ..., calcPositions:
//!   false, calcRotations: false).collidedWithTile`.
//!
//! RNG contract: the bolt owns a PRIVATE `LCG32Random` per `GenerateBolt`
//! call, seeded from the spawn seed (`Main.rand.Next(MaxFloatInt + 1)`,
//! `WorldGen.cs:72005`). It is NOT `Main.rand` — the geometry draws on no
//! named port stream. `NextDouble` advances then reads `state / 2^32`
//! (`LCG32Random.cs:34-38`); `NextFloat` casts it (`:40-43`). The fork
//! recursion seeds a child with `lCG32Random.state + 1` (`:141`) — the
//! state AFTER the fork-chance `NextFloat`, before any further draw.
//!
//! The `CanHitTarget` path passes `bolts = null`, so the fork block
//! (`:137-142`) never recurses and never draws its `NextFloat` — but the
//! `layer == Layers-1` arm still runs `num13 = NextFloat()` (`:134`)
//! UNCONDITIONALLY (the C# evaluates it before the `bolts != null` test).
//! That draw is state-bearing for every later step and is ported.
//!
//! `CalcRotations`/`SmoothRotations` (`:206-241`) are draw-side only
//! (`calcRotations` is false on every sim path here) — not ported.
//!
//! UNWIRED per the `npc_ai_sand_elemental` precedent: the only sim callers
//! are `storm_lightning.rs`'s spawn gates; the 1091 AI + the tick drain
//! are the wiring follow-up (c40-deferred-to-f42 contract).
#![allow(dead_code)] // wiring deferred — the storm-bolt spawn lane lands with the 1091 AI

use glam::Vec2;
use tr_core::World;

/// `Terraria.Utilities.LCG32Random` (`Utilities/LCG32Random.cs:5-44`) —
/// the 32-bit LCG the bolt geometry seeds per call. `Advance` is
/// `state = (uint)((int)state * -1856014347 + 1)`; `NextDouble` advances
/// then reads `state / 2^32` as f64. The Aqua Scepter's lava bounce uses
/// the same generator (`projectile.rs`'s private `Lcg32Random`); this is
/// the bolt-geometry copy with the f64 `NextDouble` the steering math
/// needs (the bounce only reads `NextFloat`).
pub(crate) struct Lcg32 {
    pub state: u32,
}

impl Lcg32 {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }
    /// `NextDouble` (`:34-38`): advance, then `state / 4294967296.0`.
    pub fn next_double(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(-1856014347i32 as u32).wrapping_add(1);
        self.state as f64 / 4294967296.0
    }
    /// `NextFloat` (`:40-43`): `(float)NextDouble()`.
    pub fn next_float(&mut self) -> f32 {
        self.next_double() as f32
    }
}

/// `FloatRange` (`Utilities/Terraria/Utilities/FloatRange.cs:6-47`) — the
/// `[Minimum, Maximum]` progress window a bolt occupies.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FloatRange {
    pub min: f32,
    pub max: f32,
}

impl FloatRange {
    /// `Contains` (`:14-21`): inclusive both ends.
    pub fn contains(&self, f: f32) -> bool {
        self.min <= f && f <= self.max
    }
    /// `Lerp` (`:23-26`): `MathHelper.Lerp(Minimum, Maximum, amount)`.
    pub fn lerp(&self, amount: f32) -> f32 {
        self.min + (self.max - self.min) * amount
    }
}

/// `LightningGenerator.Bolt` (`LightningGenerator.cs:11-24`).
/// `positions` is `None` when `calcPositions` is false (the
/// `CanHitTarget` path); the `Kill()` strike reads the positioned main
/// bolt (`GenerateMainBoltPath`, `calcPositions: true`).
#[derive(Clone, Debug)]
pub(crate) struct Bolt {
    pub positions: Option<Vec<Vec2>>,
    pub progress_range: FloatRange,
    pub fork_depth: i32,
    pub collided_with_tile: bool,
}

impl Bolt {
    /// `IsMainBolt` (`:23`): `forkDepth == 0`.
    pub fn is_main_bolt(&self) -> bool {
        self.fork_depth == 0
    }
}

/// `LightningGenerator` (`LightningGenerator.cs:9-62`) — the tuning
/// fields the geometry reads. The `StormLightning` preset fills them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LightningGenerator {
    pub solid_tile_collision: bool,
    pub source_rotation_limit: f32,
    pub length: f32,
    pub rotation_strength: f32,
    pub step_size: f32,
    pub layers: usize,
    pub layer_strength_factor: f32,
    pub perpendicular_deviation_factor: f32,
    pub reduce_randomness_after: f32,
    pub fork_generation_threshold_angle_fraction: f32,
    pub fork_reflect_angle_multiplier: f32,
    pub fork_rotation_strength_multiplier: f32,
    pub fork_step_size_multiplier: f32,
    pub fork_length_multiplier: f32,
    pub max_forks_per_bolt: i32,
    pub max_fork_depth: i32,
    pub fork_progress_range: FloatRange,
}

impl LightningGenerator {
    /// The `StormLightning` static preset (`LightningGenerator.cs:285-304`).
    pub fn storm_lightning() -> Self {
        Self {
            solid_tile_collision: true,
            source_rotation_limit: std::f32::consts::PI / 9.0,
            length: 1000.0,
            rotation_strength: 0.9,
            step_size: 8.0,
            layers: 4,
            layer_strength_factor: 1.5,
            perpendicular_deviation_factor: 5.0,
            reduce_randomness_after: 0.8,
            fork_generation_threshold_angle_fraction: 0.65,
            fork_reflect_angle_multiplier: 0.4,
            fork_rotation_strength_multiplier: 0.9,
            fork_step_size_multiplier: 0.8,
            fork_length_multiplier: 0.8,
            max_forks_per_bolt: 2,
            max_fork_depth: 2,
            fork_progress_range: FloatRange { min: 0.3, max: 0.8 },
        }
    }

    /// `CanHitTarget` (`:243-246`): the main bolt must reach the target
    /// without a solid/liquid collision. `direction` defaults to
    /// `Vector2.UnitY` (`:255`). `bolts = null`, `calcPositions = false`.
    pub fn can_hit_target(&self, world: &World, seed: u32, target: Vec2) -> bool {
        !self
            .generate(world, seed, target, Vec2::Y, false, false)
            .collided_with_tile
    }

    /// `GenerateMainBoltPath` (`:248-251`): the positioned main bolt the
    /// `Kill()` strike reads (`calcPositions: true`, `bolts = null`,
    /// `calcRotations = false`). The last position is the collision point
    /// when the bolt hit a tile (`Projectile.cs:70822`).
    pub fn generate_main_bolt_path(&self, world: &World, seed: u32, target: Vec2) -> Bolt {
        self.generate(world, seed, target, Vec2::Y, true, false)
    }

    /// `Generate` (`:64-76`): rotate the source direction by the seeded
    /// LCG draw, scale by `Length`, run the main bolt from
    /// `target - direction` to `target`. `collect_forks` is the C#'s
    /// `bolts != null` (false on both sim paths here).
    fn generate(
        &self,
        world: &World,
        seed: u32,
        target: Vec2,
        direction: Vec2,
        calc_positions: bool,
        collect_forks: bool,
    ) -> Bolt {
        let mut lcg = Lcg32::new(seed);
        let angle = (lcg.next_double() * 2.0 - 1.0) * self.source_rotation_limit as f64;
        let dir = rotated_by(direction, angle) * self.length;
        self.generate_bolt(
            world,
            seed,
            0,
            target - dir,
            target,
            self.rotation_strength,
            self.step_size,
            FloatRange { min: 0.0, max: 1.0 },
            calc_positions,
            collect_forks,
        )
    }

    /// `GenerateBolt` (`:78-162`). `collect_forks` is the C#'s
    /// `bolts != null` — false on the sim paths, so the fork block never
    /// recurses (but its `NextFloat` at `:134` still draws).
    #[allow(clippy::too_many_arguments)]
    fn generate_bolt(
        &self,
        world: &World,
        seed: u32,
        depth: i32,
        start_pos: Vec2,
        target_pos: Vec2,
        rotation_strength: f32,
        step_size: f32,
        progress_range: FloatRange,
        calc_positions: bool,
        collect_forks: bool,
    ) -> Bolt {
        let mut lcg = Lcg32::new(seed);
        let mut num = 0.0f32; // the accumulated rotation
        let mut array = vec![0.0f32; self.layers];
        let point = to_tile(target_pos);
        let mut vector = start_pos;
        let mut value = target_pos - start_pos;
        let num2 = value.length();
        value /= num2;
        let value2 = Vec2::new(value.y, -value.x);
        let num3 = ((num2 * 2.0 / step_size).max(1.0)) as i32;
        let mut num4 = 0i32; // forks spawned
        let mut positions: Option<Vec<Vec2>> = calc_positions.then(Vec::new);
        let mut bolt = Bolt {
            positions: None,
            progress_range,
            fork_depth: depth,
            collided_with_tile: false,
        };
        let mut i = 0i32;
        while i < num3 {
            // `:101-104`: record the step position (calcPositions only).
            if let Some(ps) = positions.as_mut() {
                ps.push(vector);
            }
            // `:105-110`: the remaining-distance break.
            let vector2 = target_pos - vector;
            let num5 = vector2.dot(value);
            if num5 < step_size {
                break;
            }
            let num6 = (1.0 - num5 / num2).clamp(0.0, 1.0);
            // `:112-117`: the solid/liquid collision break.
            if self.solid_tile_collision
                && to_tile(vector) != point
                && self.tile_collision(world, vector)
            {
                bolt.progress_range =
                    FloatRange { min: progress_range.min, max: progress_range.lerp(num6) };
                bolt.collided_with_tile = true;
                break;
            }
            let vector2 = vector2 / vector2.length();
            // `:119`: the perpendicular-deviation steering term.
            let num7 = ((-vector2.dot(value2))
                / (0.01f32.max(num6.min(1.0 - num6) * self.perpendicular_deviation_factor * 2.0)))
                .clamp(-1.0, 1.0);
            // `:120-144`: the low-layer reroll.
            if let Some(layer) = self.pick_layer_to_reroll(lcg.next_double(), 0.5) {
                let mut num8 = rotation_strength;
                let mut num9 = self.layers as i32 - 1;
                while num9 > layer as i32 {
                    num8 /= self.layer_strength_factor;
                    num9 -= 1;
                }
                let num10 = lcg.next_double() as f32 * 2.0 - 1.0;
                let num11 = (num10 + (num7 - num10 * num7.abs()) / 2.0) * num8;
                let num12 = num11 - array[layer];
                num += num12;
                array[layer] = num11;
                if layer == self.layers - 1 {
                    // `:134`: the fork-chance draw — UNCONDITIONAL (the
                    // C# evaluates it before the `bolts != null` test).
                    let num13 = lcg.next_float();
                    let num14 = remap(num4 as f32, 0.0, self.max_forks_per_bolt as f32, 1.0, 0.0);
                    let num15 = num - num12 * (1.0 + self.fork_reflect_angle_multiplier);
                    if collect_forks
                        && num12.abs() >= rotation_strength * self.fork_generation_threshold_angle_fraction
                        && self.fork_progress_range.contains(num6)
                        && depth < self.max_fork_depth
                        && num13 < num14
                        && num15.abs() < std::f32::consts::PI * 4.0 / 9.0
                    {
                        num4 += 1;
                        let num16 = (1.0 - num6) * self.fork_length_multiplier;
                        let fork_target =
                            vector + rotated_by(vector2, num15 as f64) * num2 * num16;
                        let fork_seed = lcg.state.wrapping_add(1);
                        let _ = self.generate_bolt(
                            world,
                            fork_seed,
                            depth + 1,
                            vector,
                            fork_target,
                            rotation_strength * self.fork_rotation_strength_multiplier,
                            step_size * self.fork_step_size_multiplier,
                            FloatRange {
                                min: progress_range.lerp(num6),
                                max: progress_range.lerp(num6 + num16),
                            },
                            calc_positions,
                            collect_forks,
                        );
                    }
                }
            }
            // `:145-149`: the high-layer reset.
            let chance = remap(num6, self.reduce_randomness_after, 1.0, 0.0, 1.0)
                + remap(num7.abs(), 0.5, 1.0, 0.0, 1.0);
            if let Some(layer) = self.pick_high_layer_to_reroll(lcg.next_double(), chance) {
                num -= array[layer];
                array[layer] = 0.0;
            }
            // `:150`: the step.
            vector += rotated_by(vector2, num as f64) * step_size;
            i += 1;
        }
        // `:152-156`: the C# preallocates num3 slots, writes [0..i] at the
        // loop heads, then `Array.Resize(ref array2, i + 1)` on an early
        // break — the final length is i+1 (the last WRITTEN index + 1),
        // not the push count. Truncate to match exactly.
        if let Some(ps) = positions.as_mut() {
            ps.truncate((i + 1).max(0) as usize);
        }
        bolt.positions = positions;
        bolt
    }

    /// `TileCollision` (`:164-181`): out-of-world / missing tile → false;
    /// `SolidOrSlopedTile` → true; a liquid whose surface the point dips
    /// below → true.
    fn tile_collision(&self, world: &World, pos: Vec2) -> bool {
        let p = to_tile(pos);
        if p.0 < 0 || p.0 >= world.width || p.1 < 0 || p.1 >= world.height {
            return false;
        }
        let Some(t) = world.tile(p.0, p.1) else {
            return false;
        };
        if crate::collision::solid_or_sloped_tile(t, false) {
            return true;
        }
        let liquid = t.liquid as i32;
        if liquid > 0 && (pos.y as i32) % 16 > 16 * (255 - liquid) / 255 {
            return true;
        }
        false
    }

    /// `PickLayerToReroll` (`:183-194`): the geometric layer pick —
    /// `r >= 1-chance` hits, else `r /= chance` and the next layer.
    fn pick_layer_to_reroll(&self, mut r: f64, chance: f32) -> Option<usize> {
        for layer in 0..self.layers {
            if r >= (1.0 - chance) as f64 {
                return Some(layer);
            }
            r /= chance as f64;
        }
        None
    }

    /// `PickHighLayerToReroll` (`:196-204`): the low pick mirrored to the
    /// top layer (`Layers - 1 - layer`).
    fn pick_high_layer_to_reroll(&self, r: f64, chance: f32) -> Option<usize> {
        self.pick_layer_to_reroll(r, chance)
            .map(|layer| self.layers - 1 - layer)
    }
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`) — the float cos/sin of the
/// double angle (the `bow_launch::rotated_by` shape).
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    Vec2::new(v.x * num - v.y * num2, v.x * num2 + v.y * num)
}

/// `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4`.
fn to_tile(v: Vec2) -> (i32, i32) {
    ((v.x as i32) >> 4, (v.y as i32) >> 4)
}

/// `Utils.Remap` (`Utils.cs:345-348`) with the default `clamped = true`:
/// `Lerp(toMin, toMax, GetLerpValue(fromMin, fromMax, value, true))`.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * get_lerp_value(from_min, from_max, value)
}

/// `Utils.GetLerpValue(from, to, t, clamped: true)` (`Utils.cs:315-340`).
fn get_lerp_value(from: f32, to: f32, t: f32) -> f32 {
    if from < to {
        if t < from {
            return 0.0;
        }
        if t > to {
            return 1.0;
        }
    } else {
        if t < to {
            return 1.0;
        }
        if t > from {
            return 0.0;
        }
    }
    (t - from) / (to - from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// `LCG32Random` (`LCG32Random.cs:9-12,34-38`): the advance is
    /// `state * -1856014347 + 1` (wrapping), `NextDouble` reads
    /// `state / 2^32`. Seed 0's first draw is `1 / 2^32`.
    #[test]
    fn w4_c40_the_lcg32_advance_and_double_match_the_csharp() {
        let mut lcg = Lcg32::new(0);
        let d = lcg.next_double();
        assert_eq!(lcg.state, 1);
        assert!((d - 1.0 / 4294967296.0).abs() < 1e-18);
        // Second advance: 1 * -1856014347 + 1 = -1856014346 (wrapping u32).
        let d2 = lcg.next_double();
        assert_eq!(lcg.state, (-1856014346i32) as u32);
        assert!((d2 - lcg.state as f64 / 4294967296.0).abs() < 1e-18);
    }

    /// `PickLayerToReroll` (`:183-194`): chance 0.5, r = 0.5 hits layer 0
    /// (0.5 >= 1-0.5); r = 0.25 → 0.5 hits layer 1; r = 0.2 → 0.4 → 0.8
    /// hits layer 2; r = 0.1 → 0.2 → 0.4 → 0.8 hits layer 3; r = 0.01
    /// walks all four (0.01/0.02/0.04/0.08) and misses.
    #[test]
    fn w4_c40_the_layer_pick_walks_the_geometric_bands() {
        let g = LightningGenerator::storm_lightning();
        assert_eq!(g.pick_layer_to_reroll(0.5, 0.5), Some(0));
        assert_eq!(g.pick_layer_to_reroll(0.25, 0.5), Some(1));
        assert_eq!(g.pick_layer_to_reroll(0.2, 0.5), Some(2));
        assert_eq!(g.pick_layer_to_reroll(0.1, 0.5), Some(3));
        assert_eq!(g.pick_layer_to_reroll(0.01, 0.5), None);
        // The high pick mirrors: r = 0.5 → low 0 → high 3.
        assert_eq!(g.pick_high_layer_to_reroll(0.5, 0.5), Some(3));
        assert_eq!(g.pick_high_layer_to_reroll(0.25, 0.5), Some(2));
    }

    /// `CanHitTarget` over pure air: the bolt reaches the target with no
    /// collision, so the spawn is allowed (every seed).
    #[test]
    fn w4_c40_a_bolt_over_air_can_hit_its_target() {
        let w = air();
        let g = LightningGenerator::storm_lightning();
        let target = Vec2::new(1600.0, 1600.0);
        for seed in [0u32, 1, 42, 12345, 16777216] {
            assert!(g.can_hit_target(&w, seed, target), "seed {seed} over air");
        }
    }

    /// `TileCollision` (`:164-181`): a solid tile blocks, a liquid the
    /// point dips below blocks, air does not. The point (1600, 1608) is
    /// tile (100, 100); `(int)1608 % 16 = 8`.
    #[test]
    fn w4_c40_the_tile_collision_reads_solid_and_liquid_surface() {
        let mut w = air();
        let g = LightningGenerator::storm_lightning();
        let pos = Vec2::new(1600.0, 1608.0);
        // Air: no collision.
        assert!(!g.tile_collision(&w, pos));
        // A solid dirt tile at (100, 100).
        let idx = w.index(100, 100);
        w.tiles[idx].kind = 1; // dirt is solid
        w.tiles[idx].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(g.tile_collision(&w, pos));
        // Reset to air, then a half-full liquid (128): surface at
        // 16*(255-128)/255 = 7 (integer), the point's %16 = 8 IS > 7, so
        // the point dips below the surface → collision.
        w.tiles[idx].flags &= !tr_core::tile::FLAG_ACTIVE;
        w.tiles[idx].liquid = 128;
        assert!(g.tile_collision(&w, pos));
        // A thin liquid (16): surface 16*239/255 = 14 (integer division),
        // 8 > 14 is false → the point stays above the surface, no collision.
        w.tiles[idx].liquid = 16;
        assert!(!g.tile_collision(&w, pos));
        // A fuller liquid (200): surface 16*55/255 = 3, 8 > 3 → collision.
        w.tiles[idx].liquid = 200;
        assert!(g.tile_collision(&w, pos));
    }

    /// `FloatRange` (`FloatRange.cs:14-26`): inclusive contains, the lerp.
    #[test]
    fn w4_c40_the_float_range_contains_and_lerps() {
        let r = FloatRange { min: 0.3, max: 0.8 };
        assert!(r.contains(0.3) && r.contains(0.8) && r.contains(0.5));
        assert!(!r.contains(0.29) && !r.contains(0.81));
        assert!((r.lerp(0.0) - 0.3).abs() < 1e-6);
        assert!((r.lerp(1.0) - 0.8).abs() < 1e-6);
        assert!((r.lerp(0.5) - 0.55).abs() < 1e-6);
    }
}
