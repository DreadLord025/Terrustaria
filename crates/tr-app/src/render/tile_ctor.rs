//! W4-g31 attach phase — the `TileDrawing` static-ctor DRAW residual
//! (`TileDrawing.cs:789-862`): the `noise[256]` table (`System.Random(0)`,
//! `:796-801`), `LinearNoise` (`:804-817`), the two hash mixers (`:819-837`),
//! `DistToWanderingCircle` (`:839-849`), `SmoothStep` (`:784-787`) and
//! `LavaLightA` (`:851-862`) — the type-717 lava-block glowmask colour
//! (`DrawSingleTile`'s `case 717`, `:1193-1197`:
//! `new Color(num12, num12, num12, num12 / 2f)`).
//!
//! Census (the attach phase, UNION-CONTRACT §8): the static ctor's other
//! draws are covered — the four `Layer_*` constants are `tile_pass.rs`'s
//! (`:791-794` ≡ `LAYER_*`), `_zero` is the `[0.0, 0.0]` origin literal the
//! sprite pushes already pass, and the instance ctor's `_rand`/9000-slot
//! buckets are `special_points.rs`'s registry + the caller-owned streams.
//! This module closes the LAST ctor draw: the noise table and its only
//! draw-side consumer chain.
//!
//! The 717 bind (TILEPASS final, EXECUTED as a shadow PLAN): the `case
//! 717` glowmask arm lives in the `Main.tileGlowMask` TABLE pass
//! (`:1178-1342`) which the f15 co-lane (`8ca19097`) explicitly scopes
//! OUT (its seam is the SECOND-switch record bind, `:1343-1403`) — so
//! `tile_attach.rs::classify_and_draw_cell` plans the bind per drawn 717
//! cell (`lava_glows`): mask 362 (`Main.cs:10056`), `Color(v, v, v, v/2)`
//! per-channel truncation, stream-free. The PIXEL push (the `Glow_362`
//! sheet rect at the cell's `vector`, behind the `:971` gate coupling —
//! d56 `tile_draw_gate(glow_mask_id: 362)` keeps a dark 717 cell out of
//! the black merge) lands with the table-pass lane / family merge;
//! the bind-contract is `docs/research/136-tilepass-d-hunks.md` §3.
//! The maths here is pure and complete so that bind is a
//! one-liner. The table is built
//! through the port's oracle-pinned `UnifiedRandom` (`System.Random`'s
//! Knuth subtractive stream — `UnifiedRandom.cs` IS the .NET Framework
//! algorithm, `unified_random.rs` module doc), seed 0, 256 `NextDouble`
//! draws in index order.

#![allow(dead_code)] // the ctor chain beyond lava_light_a (the 717 shadow-plan consumer, tile_attach.rs) has no other caller yet: research/136 §3.

use std::sync::OnceLock;
use tr_core::{UnifiedRandom, World};

/// `TileDrawing.noise` (`TileDrawing.cs:154`, built `:796-801`): 256
/// `(float)(random.NextDouble() * 2.0 - 1.0)` draws from
/// `new Random(0)` — ONE shared table for the whole process (the C#
/// static ctor runs once; the `OnceLock` is the same semantics).
pub(crate) fn noise_table() -> &'static [f32; 256] {
    static NOISE: OnceLock<[f32; 256]> = OnceLock::new();
    NOISE.get_or_init(|| {
        let mut rng = UnifiedRandom::new(0);
        std::array::from_fn(|_| (rng.next_double() * 2.0 - 1.0) as f32)
    })
}

/// `TileDrawing.LinearNoise` (`TileDrawing.cs:804-817`): the negative-arm
/// `num--` makes `(int)x` a FLOOR (not Rust's truncating `as i32`), the
/// table index wraps `& 0xFF`, and the blend is `n0*(1-t) + n1*t` in f32.
pub(crate) fn linear_noise(x: f32) -> f32 {
    let mut num = x as i32;
    if x < 0.0 {
        num -= 1;
    }
    let num2 = num + 1;
    let num3 = x - num as f32;
    let table = noise_table();
    let num4 = table[(num & 0xFF) as usize];
    let num5 = table[(num2 & 0xFF) as usize];
    num4 * (1.0 - num3) + num5 * num3
}

/// `TileDrawing.Hash` (`TileDrawing.cs:819-827`): the 16/15/16 xor-shift
/// mixer with the two odd multipliers, u32 wrapping throughout.
pub(crate) fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(2146121005);
    x ^= x >> 15;
    x = x.wrapping_mul(2221713035);
    x ^= x >> 16;
    x
}

/// `TileDrawing.Hash2` (`TileDrawing.cs:829-837`): the 15/15/15 twin.
pub(crate) fn hash2(mut x: u32) -> u32 {
    x ^= x >> 15;
    x = x.wrapping_mul(3513297581);
    x ^= x >> 15;
    x = x.wrapping_mul(2943497623);
    x ^= x >> 15;
    x
}

/// `TileDrawing.DistToWanderingCircle` (`TileDrawing.cs:839-849`): the
/// grid-cell centre wanders by two `LinearNoise` draws keyed on the
/// XORed hashes (masked `& 0xFFFFFF` BEFORE the float add), clocked by
/// `(float)Main.timeForVisualEffects` (the port feeds the wrapped tick
/// clock, `liquids.rs:207`'s convention). `Vector2.Distance` is the
/// `sqrt(dx*dx + dy*dy)` in f32.
pub(crate) fn dist_to_wandering_circle(
    pos: (i32, i32),
    grid_size: i32,
    wander_dist: f32,
    cycles_per_tick: f32,
    seed: u32,
    time_for_visual_effects: f32,
) -> f32 {
    // `:840` — integer division truncates toward zero (C# `/` on ints).
    let point = (pos.0 / grid_size, pos.1 / grid_size);
    // `:841` — the cell centre in world pixels.
    let mut value = [
        (point.0 as f32 + 0.5) * grid_size as f32,
        (point.1 as f32 + 0.5) * grid_size as f32,
    ];
    let num = time_for_visual_effects;
    // `:844-845` — the u32 casts of possibly-negative grid coords wrap.
    let (px, py) = (point.0 as u32, point.1 as u32);
    let num2 = (hash(px) ^ hash2(py) ^ seed) & 0xFF_FFFF;
    let num3 = (hash2(px) ^ hash(py) ^ seed) & 0xFF_FFFF;
    // `:846-847` — the noise arguments are f32 sums THEN the multiply.
    value[0] += linear_noise((num2 as f32 + num) * cycles_per_tick) * wander_dist;
    value[1] += linear_noise((num3 as f32 + num) * cycles_per_tick) * wander_dist;
    // `:848` — Vector2.Distance(pos.ToVector2(), value).
    let dx = pos.0 as f32 - value[0];
    let dy = pos.1 as f32 - value[1];
    (dx * dx + dy * dy).sqrt()
}

/// `TileDrawing.SmoothStep` (`TileDrawing.cs:784-787`): the CUBIC Hermite
/// `x*x*(3-2x)` — NOT `Utils.SmoothStep` (`Utils.cs:126-129`, a clamped
/// inverse lerp; `scene.rs:1877`'s copy). No clamp here: the caller's
/// `GetLerpValue(clamped: true)` already bounds x to 0..1.
fn smooth_step(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

/// `Utils.Remap` (`Utils.cs:345-348`) over the shared clamped
/// `GetLerpValue` (`scene.rs:1839`, `Utils.cs:315-343`): `Lerp(toMin,
/// toMax, GetLerpValue(fromMin, fromMax, value, clamped: true))`. The
/// `scene.rs` copy is private; this local twin binds the same shared
/// inverse lerp (keep-one on `get_lerp_value`, not a second maths).
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = tr_core::scene::get_lerp_value(from_min, from_max, value, true);
    to_min + (to_max - to_min) * t
}

/// `TileDrawing.LavaLightA` (`TileDrawing.cs:851-862`): the 717 lava
/// block's glow intensity — four wandering circles (grid 7, wander 2,
/// 0.025 cycles/tick, the four literal seeds), the min-pair SmoothStep,
/// then `Remap(0.7, 1, toMin, 1)` where `toMin = 0.8` when the cell
/// ABOVE is not `WorldGen.SolidTile` (`:857-860`). The draw-side
/// consumer builds `Color(num12, num12, num12, num12 / 2f)` (`:1196`).
pub(crate) fn lava_light_a(
    world: &World,
    tile_x: i32,
    tile_y: i32,
    time_for_visual_effects: f32,
) -> f32 {
    let circle = |x: i32, y: i32, seed: u32| {
        dist_to_wandering_circle((x, y), 7, 2.0, 0.025, seed, time_for_visual_effects)
    };
    let val = circle(tile_x, tile_y, 2221713035);
    let val2 = circle(tile_x + 3, tile_y, 657044585);
    // `:855` — min(min(y+3 pair), min(val, val2)), GetLerpValue 0..3.5.
    let lower = circle(tile_x, tile_y + 3, 741521833).min(circle(tile_x + 3, tile_y + 3, 56936621));
    let from_value = smooth_step(1.0 - tr_core::scene::get_lerp_value(0.0, 3.5, lower.min(val.min(val2)), true));
    // `:856-860` — the SolidTile probe on the cell above (null ⇒ true,
    // the WorldGen.cs:70115-70117 arm).
    let mut to_min = 0.0f32;
    if !solid_tile(world, tile_x, tile_y - 1) {
        to_min = 0.8;
    }
    remap(from_value, 0.7, 1.0, to_min, 1.0)
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`): active, in
/// `Main.tileSolid`, not `tileSolidTop`, not half-brick, slope 0, not
/// actuated; a NULL tile is `true` (`:70115-70117`). Same shape as
/// `drawdata_cases_a.rs:129`'s private copy (the family merge keep-ones).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    use tr_core::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// The ctor draw: the 256-entry table through the oracle-pinned
    /// `UnifiedRandom` seed-0 stream (`TileDrawing.cs:796-801`). The
    /// f32 bit patterns were re-derived from the .NET Framework Knuth
    /// subtractive algorithm (the same oracle `unified_random.rs`'s
    /// golden tests pin) — index 0/4/100/128/200/255 + the range.
    #[test]
    fn w4_g31_ctor_noise_table_matches_the_seed0_stream() {
        let t = noise_table();
        assert_eq!(t[0].to_bits(), 0x3EE7_AC51);
        assert_eq!(t[4].to_bits(), 0xBF16_82D3);
        assert_eq!(t[100].to_bits(), 0x3E7F_3A38);
        assert_eq!(t[128].to_bits(), 0xBE67_24BE);
        assert_eq!(t[200].to_bits(), 0xBEFA_B388);
        assert_eq!(t[255].to_bits(), 0x3E92_1848);
        assert!(t.iter().all(|&v| (-1.0..1.0).contains(&v)));
        // The f64 pre-cast value of draw 0 (the ×2−1 then (float) cast).
        assert_eq!(t[0], 0.4524865399359197f64 as f32);
    }

    /// `LinearNoise` (`:804-817`): the integer knots return the table
    /// entry exactly; the negative arm floors (`(int)(-0.5) = 0` then
    /// `num-- = -1`, so the blend runs table[255]→table[0] with t=0.5);
    /// the index wraps at 256.
    #[test]
    fn w4_g31_linear_noise_floors_and_wraps() {
        let t = noise_table();
        assert_eq!(linear_noise(0.0), t[0]);
        assert_eq!(linear_noise(100.0), t[100]);
        assert_eq!(linear_noise(256.0), t[0]);
        let expect = t[255] * (1.0 - 0.5) + t[0] * 0.5;
        assert_eq!(linear_noise(-0.5), expect);
        // Golden f32 bits (independent re-derivation over the seed-0 table).
        assert_eq!(linear_noise(-0.5).to_bits(), 0x3EBCE24C);
        // The midpoint blend at knot 0: t[0]*(1-0.25) + t[1]*0.25.
        let expect = t[0] * 0.75 + t[1] * 0.25;
        assert_eq!(linear_noise(0.25), expect);
        assert_eq!(linear_noise(0.25).to_bits(), 0x3EFEFD79);
    }

    /// The two hash mixers (`:819-837`) — u32 wrapping, golden vectors
    /// re-derived from the shift/multiply chains (independent evaluation
    /// of the same integer ops, not the Rust body).
    #[test]
    fn w4_g31_hash_mixers_wrap_u32() {
        // Hash(0) = 0 through every step; Hash2(0) = 0 likewise.
        assert_eq!(hash(0), 0);
        assert_eq!(hash2(0), 0);
        assert_eq!(hash(1), 0x6889_90C0);
        assert_eq!(hash2(1), 0xCEEC_30FF);
        assert_eq!(hash(0xDEAD_BEEF), 0xE628_C683);
        assert_eq!(hash2(0xDEAD_BEEF), 0x6273_F4F2);
        // Negative grid coords cast to u32 wrap (the :844 (uint) cast).
        assert_eq!(hash((-1i32) as u32), hash(u32::MAX));
    }

    /// `DistToWanderingCircle` (`:839-849`): at a cell centre with the
    /// noise draws pinned by construction the distance is the wander
    /// offset's magnitude; the grid division truncates toward zero and
    /// the hash mask is 24-bit.
    #[test]
    fn w4_g31_wandering_circle_centres_on_the_grid_cell() {
        // Cell (0,0) of grid 7 centres at (3.5, 3.5). Compute the two
        // noise draws the same way the body does and check the distance
        // equals sqrt of the offset pair from pos (0,0).
        let t = 123.0f32;
        let n2 = (hash(0) ^ hash2(0) ^ 2221713035u32) & 0xFF_FFFF;
        let n3 = (hash2(0) ^ hash(0) ^ 2221713035u32) & 0xFF_FFFF;
        let vx = 3.5 + linear_noise((n2 as f32 + t) * 0.025) * 2.0;
        let vy = 3.5 + linear_noise((n3 as f32 + t) * 0.025) * 2.0;
        let expect = (vx * vx + vy * vy).sqrt();
        let got = dist_to_wandering_circle((0, 0), 7, 2.0, 0.025, 2221713035, t);
        assert_eq!(got, expect);
        // The golden f32 bit pattern of the same call (independent f32
        // re-derivation of the whole chain — table, noise, hashes, sqrt).
        assert_eq!(got.to_bits(), 0x40C7_830F);
        assert_eq!(
            dist_to_wandering_circle((10, 12), 7, 2.0, 0.025, 657044585, 37.0).to_bits(),
            0x4008_D9EC
        );
        // The 24-bit mask: seeds differing above bit 23 collide.
        assert_eq!(
            dist_to_wandering_circle((5, 9), 7, 2.0, 0.025, 0x1FF_FFFF, t),
            dist_to_wandering_circle((5, 9), 7, 2.0, 0.025, 0xFFF_FFFF, t)
        );
    }

    /// `SmoothStep` is the CUBIC Hermite (`:784-787`), not Utils'
    /// inverse lerp: S(0.5) = 0.5, S(0.25) = 0.15625, S(1) = 1.
    #[test]
    fn w4_g31_smooth_step_is_the_cubic_hermite() {
        assert_eq!(smooth_step(0.0), 0.0);
        assert_eq!(smooth_step(0.5), 0.5);
        assert_eq!(smooth_step(0.25), 0.15625);
        assert_eq!(smooth_step(1.0), 1.0);
    }

    /// `LavaLightA` (`:851-862`): the remap band is [0,1] when the cell
    /// above is solid and [0.8,1] when it is not (the `:857-860` arm);
    /// the four-circle min feeds the clamped 0..3.5 lerp so the output
    /// never leaves the band. A null cell above counts as SolidTile
    /// true (`WorldGen.cs:70115-70117`) — the world-edge row keeps the
    /// [0,1] band.
    #[test]
    fn w4_g31_lava_light_bands_follow_the_solid_above_arm() {
        let mut world = World::new("t".into(), 64, 64);
        world.compute_column_surface();
        world.compute_chunks();
        // (10, 11) solid stone above the probe cell (10, 12).
        let i = world.index(10, 11);
        world.tiles[i].kind = 1;
        world.tiles[i].flags |= FLAG_ACTIVE;
        for t in [0.0f32, 37.0, 999.0] {
            let v = lava_light_a(&world, 10, 12, t);
            assert!((0.0..=1.0).contains(&v), "solid-above band [0,1]");
            // Air above (no tile written at (20, 11)): the [0.8, 1] band.
            let v = lava_light_a(&world, 20, 12, t);
            assert!((0.8..=1.0).contains(&v), "open-above band [0.8,1]");
        }
        // The world-edge null-above arm: SolidTile(null) = true ⇒ [0,1].
        let v = lava_light_a(&world, 10, 0, 50.0);
        assert!((0.0..=1.0).contains(&v));
        // Determinism: same inputs, same output (the table is shared).
        assert_eq!(
            lava_light_a(&world, 10, 12, 37.0),
            lava_light_a(&world, 10, 12, 37.0)
        );
    }
}
