//! W4-d21 folds 3+4 — `MultiPointHitbox` (`DataStructures/
//! MultiPointHitbox.cs:5-42`) + `LightningGenerator` (`GameContent/
//! LightningGenerator.cs:9-326`) + the `LCG32Random` seed stream
//! (`Utilities/LCG32Random.cs:5-44`) + `FloatRange`
//! (`Utilities/Terraria/Utilities/FloatRange.cs:6-47`).
//!
//! CENSUS VERDICT (fold 4): the generator is NOT render-only. The
//! `SolidTileCollision` probe (`:112-117`, `:164-181`) truncates the bolt
//! at the first solid-or-sloped tile / liquid surface, sets
//! `collidedWithTile` (the 203 Kill's `flag6` → the projectile RE-CENTERS
//! on the bolt's last position and `localAI[0]` picks 1 vs 2,
//! `Projectile.cs:70818-70825`), and the generated `positions` array IS
//! the damage hitbox (`customHitbox = new MultiPointHitbox(new Point(16,
//! 16), bolt.positions)`, `:70795`/`:70834`). Only `CalcRotations`/
//! `SmoothRotations` (`:206-241`) are render-only — they feed the bolt
//! DRAW angles, no gameplay reads them; they stay the documented skip
//! (the `calc_rotations` parameter rides for parity, the rotations array
//! stays `None` on the damage path — `GenerateMainBoltPath` passes
//! `calcRotations: false` anyway, `:248-251`).
//!
//! RNG contract: the whole family draws ZERO `Main.rand` — every roll
//! rides its OWN `LCG32Random(seed)` where the seed is the projectile's
//! `ai[2]` (the storm spawn's `Main.rand.Next(MaxFloatInt + 1)` draw,
//! `WorldGen.cs:72005`, happens at SPAWN time, not here). The bolt path
//! is therefore a pure function of `(seed, target, direction, config,
//! world tiles)` — deterministic by construction, no stream interference
//! (the `npc_ai_style28` unwired-maths precedent).
//!
//! Precision: `NextDouble` is f64 (`state / 4294967296.0`); the
//! `PickLayerToReroll` comparisons/divisions run in f64 exactly like the
//! C# (`r >= (double)(1f - chance)`, `r /= (double)chance`); `NextFloat`
//! is the f32 CAST of the double (`:40-43`). `Utils.RotatedBy` takes the
//! double angle and spins with float cos/sin (`Utils.cs:1675-1684`, the
//! shared [`crate::bow_launch::rotated_by`]). `Utils.Remap` is the
//! clamped lerp (the shared [`crate::projectile::remap`]).
//!
//! CODE-ONLY MODE (W4-d21): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); verified by re-reading
//! the cited C# blocks and hand arithmetic.

#![allow(dead_code)] // the worldgen storm-spawn CanHitTarget consumer is unported

use glam::Vec2;
use tr_core::whip::Rect;
use tr_core::World;

use crate::bow_launch::rotated_by;
use crate::projectile::remap;

/// `LCG32Random` (`Utilities/LCG32Random.cs:5-44`) — the 32-bit LCG the
/// bolt paths seed off `ai[2]`. NOT the `Main.rand` stream and NOT the
/// 48-bit `FastRandom`: `state = (uint)((int)state * -1856014347 + 1)`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Lcg32Random {
    /// `public uint state` — the fork seed reads `state + 1` (`:141`).
    pub state: u32,
}

impl Lcg32Random {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// `Advance` (`:9-12`): the int multiply wraps, so the u32 wrapping
    /// multiply is identical (the `projectile.rs` Lcg32Random precedent).
    fn advance(&mut self) {
        self.state = self
            .state
            .wrapping_mul(-1856014347i32 as u32)
            .wrapping_add(1);
    }

    /// `NextDouble` (`:34-38`): advance, then `state / 2^32` in f64.
    pub fn next_double(&mut self) -> f64 {
        self.advance();
        self.state as f64 / 4294967296.0
    }

    /// `NextFloat` (`:40-43`): the f32 CAST of `NextDouble`.
    pub fn next_float(&mut self) -> f32 {
        self.next_double() as f32
    }
}

/// `FloatRange` (`Utilities/Terraria/Utilities/FloatRange.cs:6-47`) —
/// the bolt's progress window. Only `Contains`/`Lerp` have gameplay
/// readers; the scale operators are JSON/render sugar (unported).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FloatRange {
    pub minimum: f32,
    pub maximum: f32,
}

impl FloatRange {
    pub fn new(minimum: f32, maximum: f32) -> Self {
        Self { minimum, maximum }
    }

    /// `Contains` (`:14-21`): inclusive both ends.
    pub fn contains(self, f: f32) -> bool {
        self.minimum <= f && f <= self.maximum
    }

    /// `Lerp` (`:23-26`): `MathHelper.Lerp` — `a + (b - a) * amount`.
    pub fn lerp(self, amount: f32) -> f32 {
        self.minimum + (self.maximum - self.minimum) * amount
    }
}

/// `MultiPointHitbox` (`DataStructures/MultiPointHitbox.cs:5-42`) — the
/// storm-lightning bolt's damage shape: a point cloud with a per-point
/// pad. Pure geometry, no engine reads.
#[derive(Clone, Debug)]
pub(crate) struct MultiPointHitbox {
    /// `readonly Point PointSize` — the 203 Kill passes `(16, 16)`.
    pub point_size: (i32, i32),
    /// `readonly Vector2[] Points` — the bolt path.
    pub points: Vec<Vec2>,
    /// `readonly Rectangle BoundingRect` — the ctor's union walk.
    pub bounding_rect: Rect,
}

impl MultiPointHitbox {
    /// The ctor (`:13-23`): `Utils.CenteredRectangle(points[0],
    /// Vector2.Zero)` = the zero-size rect at the first point's
    /// `(int)`-truncated coords, then `Including` EVERY point (index 0
    /// included — the C# loop starts at 0).
    pub fn new(point_size: (i32, i32), points: Vec<Vec2>) -> Self {
        let p0 = points[0];
        let mut rect = Rect::new(p0.x as i32, p0.y as i32, 0, 0);
        for p in &points {
            rect = including(rect, p.x as i32, p.y as i32);
        }
        Self {
            point_size,
            points,
            bounding_rect: rect,
        }
    }

    /// `Intersects` (`:25-41`): inflate the TARGET by `PointSize / 2`
    /// (integer division), the bounding-rect early-out, then the
    /// per-point `Contains` (XNA: `Left <= X < Right`, `Top <= Y <
    /// Bottom` — the point is `(int)`-truncated first).
    pub fn intersects(&self, target: Rect) -> bool {
        let mut t = target;
        t.inflate(self.point_size.0 / 2, self.point_size.1 / 2);
        if !self.bounding_rect.intersects(t) {
            return false;
        }
        self.points.iter().any(|p| {
            let (x, y) = (p.x as i32, p.y as i32);
            t.left() <= x && x < t.right() && t.top() <= y && y < t.bottom()
        })
    }
}

/// `Utils.Including` (`Utils.cs:957-963`): the rect grown to contain the
/// point — `min(Left, X)`, `max(Right, X)`, `min(Top, Y)`, `max(Bottom,
/// Y)`. NOTE the C# reads `Right`/`Bottom` (exclusive edges), so a point
/// exactly on the right edge does NOT grow the rect.
fn including(rect: Rect, x: i32, y: i32) -> Rect {
    let num = rect.left().min(x);
    let num2 = rect.right().max(x);
    let num3 = rect.top().min(y);
    Rect::new(num, num3, num2 - num, rect.bottom().max(y) - num3)
}

/// `LightningGenerator.Bolt` (`:11-24`). `rotations` stays `None` on the
/// damage path (`CalcRotations` is the render-only skip, module doc).
#[derive(Clone, Debug)]
pub(crate) struct Bolt {
    pub positions: Vec<Vec2>,
    pub rotations: Option<Vec<f32>>,
    pub progress_range: FloatRange,
    pub fork_depth: i32,
    pub collided_with_tile: bool,
}

impl Bolt {
    /// `IsMainBolt` (`:23`).
    pub fn is_main_bolt(&self) -> bool {
        self.fork_depth == 0
    }
}

/// `LightningGenerator` (`:9-326`) — the bolt-path generator. The three
/// configs: [`Self::storm_lightning`] (the 1091 trap + the worldgen
/// storm spawn), [`Self::lightning_strike_weapon`] (the 1117),
/// [`Self::arc_surge_weapon`] (the 1122, distance-parameterized).
#[derive(Clone, Debug)]
pub(crate) struct LightningGenerator {
    pub solid_tile_collision: bool,
    pub source_rotation_limit: f32,
    pub length: f32,
    pub rotation_strength: f32,
    pub step_size: i32,
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
    /// The static ctor's `StormLightning` (`:285-304`).
    pub fn storm_lightning() -> Self {
        Self {
            source_rotation_limit: std::f32::consts::PI / 9.0,
            length: 1000.0,
            rotation_strength: 0.9,
            step_size: 8,
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
            fork_progress_range: FloatRange::new(0.3, 0.8),
            solid_tile_collision: true,
        }
    }

    /// The static ctor's `LightningStrikeWeapon` (`:305-324`): the 1117
    /// weapon bolt — shorter (750), tighter source cone (PI/36), and
    /// `SolidTileCollision = false` (it phases through tiles).
    pub fn lightning_strike_weapon() -> Self {
        Self {
            source_rotation_limit: std::f32::consts::PI / 36.0,
            length: 750.0,
            rotation_strength: 0.9,
            step_size: 8,
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
            fork_progress_range: FloatRange::new(0.3, 0.8),
            solid_tile_collision: false,
        }
    }

    /// `GetArcSurgeWeaponGenerator` (`:258-281`): the 1122's generator,
    /// parameterized on the hand→target distance (`Utils.Remap` over
    /// 0..1000). `SourceRotationLimit = 0`, `Length = 1` — the direction
    /// vector is the hand→centre delta itself, unscaled.
    pub fn arc_surge_weapon(start: Vec2, end: Vec2) -> Self {
        let from_value = start.distance(end);
        Self {
            source_rotation_limit: 0.0,
            length: 1.0,
            rotation_strength: 0.7,
            step_size: 6,
            layers: 5,
            layer_strength_factor: 1.2,
            perpendicular_deviation_factor: remap(from_value, 0.0, 1000.0, 5.0, 1.0),
            reduce_randomness_after: 0.7,
            fork_generation_threshold_angle_fraction: remap(from_value, 0.0, 1000.0, 0.3, 0.5),
            fork_reflect_angle_multiplier: remap(from_value, 0.0, 1000.0, 0.6, 0.2),
            fork_rotation_strength_multiplier: 0.9,
            fork_step_size_multiplier: 0.8,
            fork_length_multiplier: 0.7,
            max_forks_per_bolt: 3,
            max_fork_depth: 3,
            fork_progress_range: FloatRange::new(remap(from_value, 0.0, 1000.0, 0.1, 0.5), 0.8),
            solid_tile_collision: true,
        }
    }

    /// `Generate(List<Bolt>, uint, Vector2, Vector2, bool, bool)`
    /// (`:64-76`): the source-direction jitter (ONE `NextDouble` on a
    /// FRESH `LCG32Random(seed)` — the bolt walk then re-seeds its own
    /// instance), the `Length` scale, the main-bolt walk. The
    /// `calcRotations` tail is the render-only skip (module doc).
    pub fn generate(
        &self,
        bolts: Option<&mut Vec<Bolt>>,
        seed: u32,
        target_position: Vec2,
        direction: Vec2,
        world: &World,
        calc_positions: bool,
    ) -> Bolt {
        let jitter = (Lcg32Random::new(seed).next_double() * 2.0 - 1.0)
            * self.source_rotation_limit as f64;
        let direction = rotated_by(direction, jitter) * self.length;
        self.generate_bolt(
            bolts,
            seed,
            0,
            calc_positions,
            target_position - direction,
            target_position,
            self.rotation_strength,
            self.step_size as f32,
            FloatRange::new(0.0, 1.0),
            world,
        )
    }

    /// `GenerateMainBoltPath` (`:248-251`) — the 203 Kill's call:
    /// `calcPositions: true`, `calcRotations: false`, no fork list.
    pub fn generate_main_bolt_path(
        &self,
        seed: u32,
        target_position: Vec2,
        direction: Option<Vec2>,
        world: &World,
    ) -> Bolt {
        self.generate(
            None,
            seed,
            target_position,
            direction.unwrap_or(Vec2::Y),
            world,
            true,
        )
    }

    /// `CanHitTarget` (`:243-246`) — the worldgen storm-spawn gate
    /// (`WorldGen.cs:72012`): the position-free walk's collision verdict.
    pub fn can_hit_target(
        &self,
        seed: u32,
        target_position: Vec2,
        direction: Option<Vec2>,
        world: &World,
    ) -> bool {
        !self
            .generate(
                None,
                seed,
                target_position,
                direction.unwrap_or(Vec2::Y),
                world,
                false,
            )
            .collided_with_tile
    }

    /// `GenerateBolt` (`:78-162`) — the layer-reroll walk, verbatim. The
    /// fork recursion only fires with a bolt list (`bolts != null`,
    /// `:137`); the damage path passes `None`, but the fork-roll DRAWS
    /// (`NextFloat` at `:134`) still happen — they ride this generator's
    /// own LCG, not `Main.rand`.
    #[allow(clippy::too_many_arguments)]
    fn generate_bolt(
        &self,
        mut bolts: Option<&mut Vec<Bolt>>,
        seed: u32,
        depth: i32,
        calc_positions: bool,
        start_pos: Vec2,
        target_pos: Vec2,
        rotation_strength: f32,
        step_size: f32,
        progress_range: FloatRange,
        world: &World,
    ) -> Bolt {
        let mut rng = Lcg32Random::new(seed);
        let mut num = 0.0f32;
        let mut array = vec![0.0f32; self.layers];
        let point = (
            (target_pos.x as i32) / 16,
            (target_pos.y as i32) / 16,
        );
        let mut vector = start_pos;
        let mut value = target_pos - start_pos;
        let num2 = value.length();
        value /= num2;
        let value2 = Vec2::new(value.y, 0.0 - value.x);
        let num3 = (num2 * 2.0 / step_size).max(1.0) as i32;
        let mut num4 = 0i32;
        let mut positions: Vec<Vec2> = Vec::with_capacity(num3.max(0) as usize);
        let mut bolt = Bolt {
            positions: Vec::new(),
            rotations: None,
            progress_range,
            fork_depth: depth,
            collided_with_tile: false,
        };
        let mut i = 0i32;
        while i < num3 {
            if calc_positions {
                positions.push(vector);
            }
            let mut vector2 = target_pos - vector;
            let num5 = vector2.dot(value);
            if num5 < step_size {
                break;
            }
            let num6 = (1.0 - num5 / num2).clamp(0.0, 1.0);
            // `:112-117` — the tile truncation: a NEW tile (not the
            // target's own) that reads solid-or-sloped or liquid-surface.
            let vt = ((vector.x as i32) / 16, (vector.y as i32) / 16);
            if self.solid_tile_collision && vt != point && self.tile_collision(vector, world) {
                bolt.progress_range =
                    FloatRange::new(progress_range.minimum, progress_range.lerp(num6));
                bolt.collided_with_tile = true;
                break;
            }
            vector2 /= vector2.length();
            let num7 = (0.0 - vector2.dot(value2))
                / (0.01f32.max(num6.min(1.0 - num6) * self.perpendicular_deviation_factor * 2.0));
            let num7 = num7.clamp(-1.0, 1.0);
            let mut layer = 0usize;
            if self.pick_layer_to_reroll(rng.next_double(), 0.5, &mut layer) {
                let mut num8 = rotation_strength;
                let mut num9 = self.layers as i32 - 1;
                while num9 > layer as i32 {
                    num8 /= self.layer_strength_factor;
                    num9 -= 1;
                }
                let num10 = rng.next_float() * 2.0 - 1.0;
                let num11 = (num10 + (num7 - num10 * num7.abs()) / 2.0) * num8;
                let num12 = num11 - array[layer];
                num += num12;
                array[layer] = num11;
                if layer == self.layers - 1 {
                    let num13 = rng.next_float();
                    let num14 = remap(num4 as f32, 0.0, self.max_forks_per_bolt as f32, 1.0, 0.0);
                    let num15 = num - num12 * (1.0 + self.fork_reflect_angle_multiplier);
                    if bolts.is_some()
                        && num12.abs() >= rotation_strength * self.fork_generation_threshold_angle_fraction
                        && self.fork_progress_range.contains(num6)
                        && depth < self.max_fork_depth
                        && num13 < num14
                        && num15.abs() < std::f32::consts::PI * 4.0 / 9.0
                    {
                        num4 += 1;
                        let num16 = (1.0 - num6) * self.fork_length_multiplier;
                        let fork_seed = rng.state.wrapping_add(1);
                        let fork_target =
                            vector + rotated_by(vector2, num15 as f64) * num2 * num16;
                        self.generate_bolt(
                            bolts.as_deref_mut(),
                            fork_seed,
                            depth + 1,
                            calc_positions,
                            vector,
                            fork_target,
                            rotation_strength * self.fork_rotation_strength_multiplier,
                            step_size * self.fork_step_size_multiplier,
                            FloatRange::new(
                                progress_range.lerp(num6),
                                progress_range.lerp(num6 + num16),
                            ),
                            world,
                        );
                    }
                }
            }
            let chance = remap(num6, self.reduce_randomness_after, 1.0, 0.0, 1.0)
                + remap(num7.abs(), 0.5, 1.0, 0.0, 1.0);
            if self.pick_high_layer_to_reroll(rng.next_double(), chance, &mut layer) {
                num -= array[layer];
                array[layer] = 0.0;
            }
            vector += rotated_by(vector2, num as f64) * step_size;
            i += 1;
        }
        // `:152-156` — the early-break resize (`Array.Resize(i + 1)`).
        if calc_positions {
            if i < num3 {
                positions.truncate((i + 1) as usize);
            }
            bolt.positions = positions;
        }
        // `:157-160` — the main bolt always joins the list; a fork only
        // when it walked past 2 steps.
        if let Some(list) = bolts {
            if bolt.is_main_bolt() || i > 2 {
                list.push(bolt.clone());
            }
        }
        bolt
    }

    /// `TileCollision` (`:164-181`): out-of-world/null → false;
    /// `SolidOrSlopedTile` → true; else the liquid-surface probe —
    /// `liquid > 0 && (int)Y % 16 > 16 * (255 - liquid) / 255` (INTEGER
    /// division on the right, the C#'s own).
    fn tile_collision(&self, pos: Vec2, world: &World) -> bool {
        let (x, y) = ((pos.x as i32) / 16, (pos.y as i32) / 16);
        if !world.in_bounds(x, y) {
            return false;
        }
        let Some(t) = world.tile(x, y) else {
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

    /// `PickLayerToReroll` (`:183-194`): the geometric layer pick — the
    /// f64 `r` against `(double)(1f - chance)`, then `r /= (double)
    /// chance` per layer. The out-param lands on `Layers` when no layer
    /// fired (the C# `for` leaves `layer == Layers`).
    fn pick_layer_to_reroll(&self, mut r: f64, chance: f32, layer: &mut usize) -> bool {
        *layer = 0;
        while *layer < self.layers {
            if r >= (1.0f32 - chance) as f64 {
                return true;
            }
            r /= chance as f64;
            *layer += 1;
        }
        false
    }

    /// `PickHighLayerToReroll` (`:196-204`): the mirrored pick.
    fn pick_high_layer_to_reroll(&self, r: f64, chance: f32, layer: &mut usize) -> bool {
        if !self.pick_layer_to_reroll(r, chance, layer) {
            return false;
        }
        *layer = self.layers - 1 - *layer;
        true
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// The LCG advance is the C#'s wrapping int multiply (`:9-12`): an
    /// independent u64-mask derivation pins the first three states off
    /// seed 12345, and `NextDouble` is `state / 2^32` in f64.
    #[test]
    fn w4_d21_the_lcg32_matches_the_csharp_advance() {
        let mut rng = Lcg32Random::new(12345);
        let mut want = 12345u64;
        for _ in 0..3 {
            want = (want * 2438952949 + 1) & 0xFFFF_FFFF;
            let d = rng.next_double();
            assert_eq!(rng.state as u64, want, "the wrapping multiply");
            assert!((d - want as f64 / 4294967296.0).abs() < 1e-15, ":34-38");
        }
        // NextFloat is the f32 cast of the same double (`:40-43`).
        let mut a = Lcg32Random::new(7);
        let mut b = Lcg32Random::new(7);
        assert_eq!(a.next_float(), b.next_double() as f32);
    }

    /// FloatRange: inclusive Contains, the plain lerp (`:14-26`).
    #[test]
    fn w4_d21_the_float_range_contains_and_lerps() {
        let r = FloatRange::new(0.3, 0.8);
        assert!(r.contains(0.3) && r.contains(0.8) && r.contains(0.5));
        assert!(!r.contains(0.29) && !r.contains(0.81));
        assert!((r.lerp(0.5) - 0.55).abs() < 1e-6);
        assert!((r.lerp(0.0) - 0.3).abs() < 1e-6);
    }

    /// MultiPointHitbox ctor: the zero-size seed rect at points[0], the
    /// Including walk over EVERY point (`:13-23`). Points (10.9, 20.9),
    /// (30.1, 5.2), (-4.0, 40.0): seed (10, 20, 0, 0); Including(30, 5)
    /// → (10, 5, 20, 15); Including(-4, 40) → (-4, 5, 34, 35).
    #[test]
    fn w4_d21_the_hitbox_bounding_rect_is_the_including_walk() {
        let h = MultiPointHitbox::new(
            (16, 16),
            vec![
                Vec2::new(10.9, 20.9),
                Vec2::new(30.1, 5.2),
                Vec2::new(-4.0, 40.0),
            ],
        );
        assert_eq!(h.bounding_rect, Rect::new(-4, 5, 34, 35));
    }

    /// Intersects: the target inflates by PointSize/2 = 8 (`:27`), the
    /// bounding early-out, then the per-point Contains with the (int)
    /// truncation. A single point at (100.5, 100.5) truncates to (100,
    /// 100): the target rect (90, 90, 4, 4) inflates to (82, 82, 20, 20)
    /// → contains (100, 100) → hit. The rect (110, 110, 4, 4) inflates
    /// to (102, 102, 20, 20) → 100 < 102 → miss (the bounding rect
    /// (100, 100, 0, 0) also fails the strict intersect).
    #[test]
    fn w4_d21_the_hitbox_inflates_the_target_and_tests_points() {
        let h = MultiPointHitbox::new((16, 16), vec![Vec2::new(100.5, 100.5)]);
        assert!(h.intersects(Rect::new(90, 90, 4, 4)));
        assert!(!h.intersects(Rect::new(110, 110, 4, 4)));
        // The point ON the inflated right edge is excluded (XNA Contains
        // is half-open): target (93, 93, 1, 1) inflates to (85, 85, 17,
        // 17) → right edge 102 > 100 → hit; target (108, 108, 1, 1)
        // inflates to (100, 100, 17, 17) → left edge 100 <= 100 → hit;
        // target (109, 109, 1, 1) → (101, 101, ...) → 100 < 101 → miss.
        assert!(h.intersects(Rect::new(108, 108, 1, 1)));
        assert!(!h.intersects(Rect::new(109, 109, 1, 1)));
    }

    /// The three configs are the verbatim static-ctor rows (`:285-324`)
    /// and the Arc Surge distance remaps (`:258-281`).
    #[test]
    fn w4_d21_the_generator_configs_are_verbatim() {
        let storm = LightningGenerator::storm_lightning();
        assert!(storm.solid_tile_collision);
        assert_eq!((storm.length, storm.step_size, storm.layers), (1000.0, 8, 4));
        assert!((storm.source_rotation_limit - std::f32::consts::PI / 9.0).abs() < 1e-7);
        let strike = LightningGenerator::lightning_strike_weapon();
        assert!(!strike.solid_tile_collision, ":323 — the 1117 phases");
        assert_eq!((strike.length, strike.step_size), (750.0, 8));
        assert!((strike.source_rotation_limit - std::f32::consts::PI / 36.0).abs() < 1e-7);
        // Arc Surge at distance 0: the remaps sit at their from-min ends.
        let surge = LightningGenerator::arc_surge_weapon(Vec2::ZERO, Vec2::ZERO);
        assert_eq!((surge.length, surge.step_size, surge.layers), (1.0, 6, 5));
        assert!((surge.perpendicular_deviation_factor - 5.0).abs() < 1e-5);
        assert!((surge.fork_generation_threshold_angle_fraction - 0.3).abs() < 1e-5);
        assert!((surge.fork_reflect_angle_multiplier - 0.6).abs() < 1e-5);
        assert!((surge.fork_progress_range.minimum - 0.1).abs() < 1e-5);
        // At distance 1000: the to-max ends.
        let far = LightningGenerator::arc_surge_weapon(Vec2::ZERO, Vec2::new(1000.0, 0.0));
        assert!((far.perpendicular_deviation_factor - 1.0).abs() < 1e-5);
        assert!((far.fork_progress_range.minimum - 0.5).abs() < 1e-5);
    }

    /// The bolt walk is a pure function of the seed: two runs off the
    /// same seed produce IDENTICAL positions; a different seed diverges.
    /// The main bolt reaches the target neighbourhood (the last position
    /// is within one step of the target, `:107-110` breaks at
    /// `num5 < stepSize`).
    #[test]
    fn w4_d21_the_bolt_path_is_seed_deterministic() {
        let w = air();
        let g = LightningGenerator::storm_lightning();
        let target = Vec2::new(160.0, 160.0);
        let a = g.generate_main_bolt_path(42, target, None, &w);
        let b = g.generate_main_bolt_path(42, target, None, &w);
        let c = g.generate_main_bolt_path(43, target, None, &w);
        assert_eq!(a.positions, b.positions, "same seed, same path");
        assert_ne!(a.positions, c.positions, "a new seed re-rolls the layers");
        assert!(!a.collided_with_tile, "air world — no truncation");
        assert!(a.is_main_bolt());
        let last = *a.positions.last().unwrap();
        assert!(last.distance(target) < 16.0, "the walk closes on the target");
        // The step count: num3 = (int)max(dist*2/8, 1) with dist ≈ 1000
        // → ~250 candidates; the break trims the tail (`:152-156`).
        assert!(a.positions.len() > 200 && a.positions.len() <= 251);
    }

    /// The tile truncation (`:112-117`): a solid row across the world at
    /// tile y=5 (world y 80..96) stops the storm bolt — collidedWithTile
    /// true, the last position inside the row. The 1117's generator
    /// (SolidTileCollision false) phases through the SAME row.
    #[test]
    fn w4_d21_the_solid_row_truncates_the_storm_bolt_only() {
        let mut w = air();
        for x in 0..w.width {
            let i = w.index(x, 5);
            w.tiles[i].kind = 1; // dirt
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let target = Vec2::new(160.0, 160.0);
        let storm = LightningGenerator::storm_lightning()
            .generate_main_bolt_path(42, target, None, &w);
        assert!(storm.collided_with_tile, ":115");
        let last = *storm.positions.last().unwrap();
        assert!((80.0..112.0).contains(&last.y), "the walk died in the row");
        assert!(storm.positions.len() < 251, "the resize trimmed the tail");
        // The progress window narrowed to the collision fraction.
        assert!(storm.progress_range.maximum < 1.0, ":114");
        let strike = LightningGenerator::lightning_strike_weapon()
            .generate_main_bolt_path(42, target, None, &w);
        assert!(!strike.collided_with_tile, ":323 — no tile collision");
    }

    /// The liquid-surface probe (`:175-179`): a FULL liquid cell (255)
    /// collides at any Y inside it (`16*(255-255)/255 = 0`, every
    /// `(int)Y % 16 >= 0 > 0` is false at Y%16==0 — the top pixel row
    /// passes, the rest collide). A half cell (128) collides only below
    /// `16*127/255 = 7` (integer division).
    #[test]
    fn w4_d21_the_liquid_surface_probe_is_the_csharp_integer_math() {
        let mut w = air();
        let i = w.index(10, 10);
        w.tiles[i].liquid = 255;
        let g = LightningGenerator::storm_lightning();
        // Y = 160 → tile (10, 10), Y%16 = 0 → 0 > 0 false → no collision.
        assert!(!g.tile_collision(Vec2::new(160.0, 160.0), &w));
        // Y = 161 → 1 > 0 → collision.
        assert!(g.tile_collision(Vec2::new(160.0, 161.0), &w));
        w.tiles[i].liquid = 128;
        // 16*(255-128)/255 = 2032/255 = 7 (int). Y%16 = 7 → 7 > 7 false.
        assert!(!g.tile_collision(Vec2::new(160.0, 167.0), &w));
        // Y%16 = 8 → 8 > 7 → collision.
        assert!(g.tile_collision(Vec2::new(160.0, 168.0), &w));
        // Out of world → false (`:167-170`).
        assert!(!g.tile_collision(Vec2::new(-16.0, 160.0), &w));
    }

    /// CanHitTarget is the position-free walk's verdict (`:243-246`):
    /// air → true, the solid row → false.
    #[test]
    fn w4_d21_can_hit_target_is_the_collision_verdict() {
        let mut w = air();
        let g = LightningGenerator::storm_lightning();
        let target = Vec2::new(160.0, 160.0);
        assert!(g.can_hit_target(42, target, None, &w));
        for x in 0..w.width {
            let i = w.index(x, 5);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        assert!(!g.can_hit_target(42, target, None, &w));
    }
}
