//! W4-g85 — the shadow orb / crimson heart draw slice (code-only, UNWIRED).
//!
//! The orb family's draw-side behaviour splits across three C# sites, all
//! keyed on the SAME break-stage bit — `frameX >= 36` (the crimson heart
//! half of the sheet; `WorldGen.AddShadowOrb` writes `+36` for
//! `crimsonHeart`, `WorldGen.cs:32111-32115`):
//!
//! 1. **The glow pulse** — `TileLightScanner.cs:2965-2983`: the per-tile
//!    `FastRandom.Next(-5, 6) * 0.0025f` jitter, purple `(0.31+n, 0.1,
//!    0.44+2n)` for the orb stage, orange `(0.5+2n, 0.2+n, 0.1)` for the
//!    heart stage. ALREADY LANDED in `tr_core::tile_light::scanner`
//!    (`misc_lights`, the `26 | 31 | 695 | 696` arm); this slice adds the
//!    stage/draw-census witnesses there, NOT a second copy.
//! 2. **The ambient draw dust** — `TileDrawing.DrawTiles_EmitParticles`
//!    (`TileDrawing.cs:7335-7347`): the `(31 || 696) && _rand.Next(20) == 0`
//!    arm — heart stage spawns dust 5 with `velocity.Y = 0`,
//!    `velocity.X *= 0.3`; orb stage spawns dust 14; both alpha 100.
//!    [`orb_ambient_dust`] ports it verbatim.
//! 3. **The hammer-break stages** — `WorldGen.KillTile`'s dust loop
//!    (`WorldGen.cs:63699-63703`) → `KillTile_GetTileDustAmount`
//!    (`:66601-66606`, the default 3-fail / 10-break counts) →
//!    `KillTile_MakeTileDust`'s stage arm (`:68966-68969`): heart stage
//!    dust 5 (NO draw), orb stage `genRand.Next(2) != 0 ? 1 : 14` (one
//!    `Main.rand` draw — `genRand` IS `Main.rand`, `WorldGen.cs:4445`),
//!    then the plain `Dust.NewDust(pos, 16, 16, num)` tail (`:69166`,
//!    alpha 0, scale 1 — none of the special spawn arms match 31/696).
//!    [`orb_break_dust_type`] / [`orb_break_burst`] port the family slice.
//!
//! Plus the ore-shine sparkle colour rows the family owns in the same
//! `DrawTiles_EmitParticles` tail (`:7413-7529`): the Heart 12 red and the
//! ManaCrystal 639 blue ([`orb_shine_sparkle`]). Census pin: the C# colour
//! line `:7511` also lists 665 (LifeCrystalBoulder), but
//! `Main.tileShine[665]` is never written (only `[12] = 300`
//! `Main.cs:7518` and `[639] = 300` `Main.cs:9652`), so the `:7413` gate
//! `tileShine[type] > 0` never admits 665 — the colour row is dead code.
//!
//! **WIRING STATE (W4-g85 wiring round).**
//! * The ambient/shine bodies are WIRED in `render/tiles.rs::draw_tiles`
//!   at the `DrawSingleTile` entry slot ([`emit_entry_gate`] ports the
//!   `TileDrawing.cs:965-968` quarter-roll: `_isActiveAndNotPaused &&
//!   (!Lighting.UpdateEveryFrame || FastRandom(TileFrameSeed)
//!   .WithModifier(x, y).Next(4) == 0)`). The else-if chain position is
//!   stream-safe for the family kinds: every preceding arm (718/244/565/
//!   215/592/406/452/192/384/666/712/711, the flies/rubble sets, 238/139/
//!   463/…, 22/23/24/32/25/112, the altars 26/695, 71/72, 17/77/133, 405,
//!   37, 58/76/684, 61/703) fails its TYPE/SET test for 31/696/12/639
//!   BEFORE any `_rand` draw — the family reaches its arm with zero prior
//!   draws (census in docs/research/W4-g85-orb-draw.md §6).
//! * The draw-side `_rand` rides the port's per-rebuild `leaf_rng`
//!   (`FastRandom(sparkle_seed ^ golden)`, the documented `_rand` stand-in
//!   — `render/tiles.rs:330-334`); the `NewDust` internals ride the
//!   client's `frame_rng` (`world.frame_rng`, the S1-E-1 per-consumer
//!   `Main.rand` stand-in, cloned through the borrow like the d54 break
//!   seam). Recorded deviation: vanilla's `_rand` is one session-lived
//!   `UnifiedRandom`; the port pins it per world-range rebuild.
//! * The break burst is UNWIRED at the `KillTile` dust slot
//!   (`WorldGen.cs:63699-63703`): W4-d54 owns that seam (its
//!   `world_dust_tile::tile_break_burst` call lands in
//!   `world_sim/editing.rs::break_tile` between the sounds and the drops)
//!   and W4-g14 ports the same `:68966` table arm in
//!   `worldgen_dust_tile.rs`. Keep-one: the family slice here is the
//!   verbatim subset; the merger keeps ONE body. [`orb_break_burst`] is
//!   the drop-in for the family kinds at that slot.
//!
//! **Keep-one boundaries** (docs/research/W4-g85-orb-draw.md): W4-d60
//! ports the WHOLE `DrawTiles_EmitParticles` battery (its
//! `ambient_dust_tail` carries the same `:7335-7347` arm) and W4-d54
//! ports the WHOLE `KillTile_MakeTileDust` table (`world_dust_tile.rs`,
//! the `:68966` arm included). This slice is the orb-family verbatim
//! subset for the g-lane; the merger keeps ONE body per arm — the
//! witnesses here pin the contract either way. The wiring round's
//! contested-seam note: `render/tiles.rs` is touched by ~12 in-flight
//! draw lanes; the call site here is a family-kind-gated block (zero
//! draws for every other kind) so a keep-one merge with d60's battery
//! replaces the block wholesale.
//!
//! **RNG census (S1-A-10, two streams kept separate).** The ambient gate
//! `Next(20)` and the shine `Next(300)` ride `TileDrawing._rand` (a
//! `UnifiedRandom`, `TileDrawing.cs:191`); every `Dust.NewDust` internal
//! and the break arm's `Next(2)` ride `Main.rand`. The glow pulse's
//! `Next(-5, 6)` rides the scanner's per-tile `FastRandom` — a THIRD
//! stream, never mixed. Code-only mode: witnesses WRITTEN NOT RUN.
#![allow(dead_code)] // the break-burst half stays UNWIRED (d54 owns the KillTile dust slot).

use glam::Vec2;
use tr_core::World;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};

/// The `DrawTiles_EmitParticles` entry gate (`TileDrawing.cs:965-968`):
/// `_isActiveAndNotPaused && (!Lighting.UpdateEveryFrame ||
/// new FastRandom(Main.TileFrameSeed).WithModifier(tileX, tileY)
/// .Next(4) == 0)`. `update_every_frame` is `Lighting.UpdateEveryFrame`
/// (`Lighting.cs:77-87`: `!RenderTargetsRequired && !NotRetro` — true only
/// in Retro/Trippy at zoom 1); the quarter-roll is a FRESH per-tile
/// `FastRandom` derivation (no stream advance — the modifier XORs into a
/// copy, `FastRandom.cs:27-32`), so a miss costs nothing on any stream.
/// `tile_frame_seed` is `Main.TileFrameSeed` (the port's
/// `flame::tile_frame_seed_at(anim_time)`).
pub(crate) fn emit_entry_gate(
    active_and_not_paused: bool,
    update_every_frame: bool,
    tile_frame_seed: u64,
    x: i32,
    y: i32,
) -> bool {
    active_and_not_paused
        && (!update_every_frame
            || tr_core::tile_light::FastRandom::new(tile_frame_seed)
                .with_modifier(x, y)
                .next(4)
                == 0)
}

/// The `_rand` gate adapter: the port's per-rebuild `leaf_rng`
/// (`FastRandom`, the documented `TileDrawing._rand` stand-in —
/// `render/tiles.rs:330-334`) fed to the slice's `DustRng` seam. One
/// `Next` conversion per draw, no new stream instance (S1-E-1).
pub(crate) struct GateRand<'a>(pub &'a mut tr_core::tile_light::FastRandom);

impl DustRng for GateRand<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next(max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
}

/// The break-stage bit shared by all three sites: `Some(true)` = the
/// crimson heart half (`frameX >= 36`), `Some(false)` = the shadow orb
/// half, `None` = not the orb family. `TileDrawing.cs:7337`,
/// `TileLightScanner.cs:2969`, `WorldGen.cs:68968`; the sheet layout is
/// `AddShadowOrb`'s `num += 36` (`WorldGen.cs:32111-32115`). The replica
/// 696 shares the 31 sheet (`Main.tileFrame[696] = Main.tileFrame[31]`,
/// `Main.cs:18588`). The altar pair 26/695 uses the SAME shape with a
/// `>= 54` threshold — that is the g86 lane, NOT this one.
pub(crate) fn orb_break_stage(kind: u16, frame_x: i16) -> Option<bool> {
    match kind {
        31 | 696 => Some(frame_x >= 36),
        _ => None,
    }
}

/// `DrawTiles_EmitParticles`' orb arm (`TileDrawing.cs:7335-7347`),
/// verbatim. `rand` is `TileDrawing._rand` (the gate stream),
/// `main_rand` is `Main.rand` (the `NewDust` internals). Returns whether
/// the gate hit (a spawn was attempted — a pool refusal still returns
/// true, the C# writes the sentinel slot either way).
///
/// The caller runs this at the arm's else-if chain position; for orb
/// kinds the preceding 22/23/24/32/25/112 arms draw NOTHING (their type
/// tests fail before the `&&` gate, `:7319-7334`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn orb_ambient_dust(
    rand: &mut impl DustRng,
    main_rand: &mut impl DustRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    kind: u16,
    frame_x: i16,
) -> bool {
    let Some(heart) = orb_break_stage(kind, frame_x) else {
        return false;
    };
    // `:7335` — the gate draws exactly once per orb cell per emit tick.
    if rand.next_max(20) != 0 {
        return false;
    }
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    if heart {
        // `:7339-7341` — dust 5, alpha 100, then the velocity clamp.
        let n = dust.new_dust(
            main_rand, ctx, world, pos, 16, 16, 5, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(n);
        d.velocity.y = 0.0;
        d.velocity.x *= 0.3;
    } else {
        // `:7345` — dust 14, alpha 100, no tweak.
        dust.new_dust(
            main_rand, ctx, world, pos, 16, 16, 14, 0.0, 0.0, 100, Color::default(), 1.0,
        );
    }
    true
}

/// The shine-arm slice for the Heart 12 / ManaCrystal 639
/// (`TileDrawing.cs:7413-7432` gate + `:7511-7518` colours + `:7527-7528`
/// spawn). `light` is the cell's `tileLight` BYTES (the C# compares
/// `<= 20` per channel). Both kinds carry `Main.tileShine = 300`
/// (`Main.cs:7518` / `:9652`); 665's colour row is dead (see the module
/// doc census pin) and every other kind is out of this slice.
///
/// Draw order: the dark-light early return draws NOTHING (`:7415-7418`);
/// `num42 = max(R,G,B) / 30` is integer arithmetic (`:7419-7428`); the
/// `Next(300)` is drawn BEFORE the `>= num42` miss test (`:7429` — a miss
/// still consumes the draw); 12/639 have no frame-window terms.
#[allow(clippy::too_many_arguments)]
pub(crate) fn orb_shine_sparkle(
    rand: &mut impl DustRng,
    main_rand: &mut impl DustRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    kind: u16,
    light: Color,
) -> bool {
    // `:7413` — `Main.tileShine[typeCache] > 0`; the family's live rows.
    let shine = match kind {
        12 | 639 => 300,
        _ => return false,
    };
    // `:7415-7418` — the dark early return, zero draws.
    if light.r <= 20 && light.b <= 20 && light.g <= 20 {
        return false;
    }
    // `:7419-7428` — the byte max, then the INTEGER /30.
    let num42 = light.r.max(light.g).max(light.b) as i32 / 30;
    // `:7429` — the roll FIRST; `>=` misses (strict spawn below).
    if rand.next_max(shine) >= num42 {
        return false;
    }
    // `:7511-7518` — Heart red, ManaCrystal blue (alpha 255).
    let color = if kind == 12 {
        Color::new(255, 0, 0, 255)
    } else {
        Color::new(0, 0, 255, 255)
    };
    // `:7527-7528` — dust 43, alpha 254, scale 0.5, velocity zeroed.
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    let n = dust.new_dust(
        main_rand, ctx, world, pos, 16, 16, 43, 0.0, 0.0, 254, color, 0.5,
    );
    dust.slot_mut(n).velocity *= 0.0;
    true
}

/// `KillTile_MakeTileDust`'s orb stage arm (`WorldGen.cs:68966-68969`,
/// the 31/696 half — the shared type-25 ebonstone clause is d54's):
/// heart stage → dust 5 with NO RNG draw; orb stage → one
/// `genRand.Next(2)` (`Main.rand`, `WorldGen.cs:4445`), non-zero → 1,
/// zero → 14. No earlier/later arm in the `:66687-69169` table rewrites
/// 31/696 (census: the only mentions are `:68966`/`:68968`).
pub(crate) fn orb_break_dust_type(
    kind: u16,
    frame_x: i16,
    main_rand: &mut impl DustRng,
) -> Option<i32> {
    let heart = orb_break_stage(kind, frame_x)?;
    Some(if heart {
        5
    } else if main_rand.next_max(2) != 0 {
        1
    } else {
        14
    })
}

/// The hammer-break dust burst: `KillTile`'s loop
/// (`WorldGen.cs:63699-63703`) over `KillTile_GetTileDustAmount`'s
/// default arm (`:66601-66606` — fail 3, break 10; 31/696 have no special
/// count row), each iteration the stage type above then the PLAIN
/// `Dust.NewDust(pos, 16, 16, num)` tail (`:69166` — alpha 0, scale 1,
/// default colour; none of the `:69085-69165` special arms match the
/// family). Every draw rides `Main.rand` in C# order: per dust the
/// stage `Next(2)` (orb only) THEN the `NewDust` internals. Returns the
/// dust count spawned (0 for non-family kinds).
#[allow(clippy::too_many_arguments)]
pub(crate) fn orb_break_burst(
    main_rand: &mut impl DustRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    kind: u16,
    frame_x: i16,
    fail: bool,
) -> i32 {
    if orb_break_stage(kind, frame_x).is_none() {
        return 0;
    }
    let count = if fail { 3 } else { 10 };
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    for _ in 0..count {
        let ty = orb_break_dust_type(kind, frame_x, main_rand).unwrap();
        dust.new_dust(
            main_rand, ctx, world, pos, 16, 16, ty, 0.0, 0.0, 0, Color::default(), 1.0,
        );
    }
    count
}

// ---------------------------------------------------------------------------
// Witnesses (W4-g85 code-only round — WRITTEN NOT RUN).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::unified_random::UnifiedRandom;

    /// A scripted `_rand`: records every gate draw, replays fixed values.
    struct ScriptRand {
        values: std::collections::VecDeque<i32>,
        draws: Vec<i32>,
    }

    impl ScriptRand {
        fn new(values: &[i32]) -> Self {
            Self { values: values.iter().copied().collect(), draws: Vec::new() }
        }
    }

    impl DustRng for ScriptRand {
        fn next_max(&mut self, max: i32) -> i32 {
            self.draws.push(max);
            self.values.pop_front().expect("script exhausted")
        }
        fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
            panic!("the gate stream never draws ranges in this slice")
        }
        fn next_float(&mut self) -> f32 {
            panic!("the gate stream never draws floats in this slice")
        }
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(4000.0, 4000.0),
            max_dust_to_draw: 6000,
            ..Default::default()
        }
    }

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    #[test]
    fn stage_bit_is_the_36px_sheet_half() {
        // WorldGen.cs:32111-32115 (+36 crimson), TileDrawing.cs:7337.
        assert_eq!(orb_break_stage(31, 0), Some(false));
        assert_eq!(orb_break_stage(31, 18), Some(false));
        assert_eq!(orb_break_stage(31, 36), Some(true));
        assert_eq!(orb_break_stage(31, 54), Some(true));
        assert_eq!(orb_break_stage(696, 0), Some(false));
        assert_eq!(orb_break_stage(696, 36), Some(true));
        // The altar pair (>= 54) is g86's lane; 12/639 are not stages.
        assert_eq!(orb_break_stage(26, 54), None);
        assert_eq!(orb_break_stage(695, 54), None);
        assert_eq!(orb_break_stage(12, 0), None);
        assert_eq!(orb_break_stage(639, 0), None);
    }

    #[test]
    fn ambient_gate_draws_once_and_a_miss_spawns_nothing() {
        let w = world();
        let mut pool = DustPool::new();
        let mut gate = ScriptRand::new(&[7]); // Next(20) != 0
        let mut main = UnifiedRandom::new(5);
        let mirror = main.clone();
        assert!(!orb_ambient_dust(
            &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, 31, 0,
        ));
        assert_eq!(gate.draws, vec![20], "exactly one Next(20)");
        assert_eq!(main, mirror, "a gate miss draws zero Main.rand");
        assert!(!pool.slot(0).active);
        // A non-family kind draws NOTHING (the type test precedes the gate).
        let mut gate2 = ScriptRand::new(&[]);
        assert!(!orb_ambient_dust(
            &mut gate2, &mut main, &mut pool, &ctx(), &w, 4, 4, 26, 0,
        ));
        assert!(gate2.draws.is_empty());
    }

    #[test]
    fn ambient_heart_stage_clamps_the_dust5_velocity() {
        let w = world();
        let mut gate = ScriptRand::new(&[0]);
        let mut main = UnifiedRandom::new(5);
        // The mirror spawns the SAME plain dust 5 with a cloned stream.
        let mut mirror_pool = DustPool::new();
        let mut mirror_rand = main.clone();
        mirror_pool.new_dust(
            &mut mirror_rand, &ctx(), &w, Vec2::new(64.0, 64.0),
            16, 16, 5, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let mut pool = DustPool::new();
        assert!(orb_ambient_dust(
            &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, 31, 36,
        ));
        let d = pool.slot(0);
        let m = mirror_pool.slot(0);
        assert_eq!(d.dust_type, 5);
        assert_eq!(d.alpha, 100);
        assert_eq!(d.velocity.y, 0.0, ":7340 velocity.Y = 0");
        assert_eq!(d.velocity.x, m.velocity.x * 0.3, ":7341 velocity.X *= 0.3");
        assert_eq!(main, mirror_rand, "the clamp adds no draws");
    }

    #[test]
    fn ambient_orb_stage_is_plain_dust14_alpha100() {
        let w = world();
        let mut gate = ScriptRand::new(&[0]);
        let mut main = UnifiedRandom::new(5);
        let mut mirror_pool = DustPool::new();
        let mut mirror_rand = main.clone();
        mirror_pool.new_dust(
            &mut mirror_rand, &ctx(), &w, Vec2::new(64.0, 64.0),
            16, 16, 14, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let mut pool = DustPool::new();
        assert!(orb_ambient_dust(
            &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, 696, 18,
        ));
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 14);
        assert_eq!(d.alpha, 100);
        assert_eq!(d.velocity, mirror_pool.slot(0).velocity, "no tweak (:7345)");
        assert_eq!(main, mirror_rand);
    }

    #[test]
    fn shine_dark_light_returns_before_any_draw() {
        let w = world();
        let mut pool = DustPool::new();
        let mut gate = ScriptRand::new(&[]);
        let mut main = UnifiedRandom::new(9);
        let mirror = main.clone();
        // :7415 — all channels <= 20 (the boundary itself is dark).
        assert!(!orb_shine_sparkle(
            &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, 12,
            Color::new(20, 20, 20, 255),
        ));
        assert!(gate.draws.is_empty());
        assert_eq!(main, mirror);
        // One channel above 20 opens the gate; 200/30 = 6 > roll 0.
        let mut gate2 = ScriptRand::new(&[0]);
        assert!(orb_shine_sparkle(
            &mut gate2, &mut main, &mut pool, &ctx(), &w, 4, 4, 12,
            Color::new(200, 0, 0, 255),
        ));
        assert_eq!(gate2.draws, vec![300]);
    }

    #[test]
    fn shine_roll_is_drawn_before_the_miss_test() {
        let w = world();
        let mut pool = DustPool::new();
        let mut main = UnifiedRandom::new(9);
        // max 255 → num42 = 255/30 = 8; a roll of 8 MISSES (>=), but draws.
        let mut gate = ScriptRand::new(&[8]);
        let mirror = main.clone();
        assert!(!orb_shine_sparkle(
            &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, 639,
            Color::new(255, 255, 255, 255),
        ));
        assert_eq!(gate.draws, vec![300], "the miss still consumes Next(300)");
        assert_eq!(main, mirror, "a miss spawns no dust");
        // A roll of 7 spawns.
        let mut gate2 = ScriptRand::new(&[7]);
        assert!(orb_shine_sparkle(
            &mut gate2, &mut main, &mut pool, &ctx(), &w, 4, 4, 639,
            Color::new(255, 255, 255, 255),
        ));
    }

    #[test]
    fn shine_colours_heart_red_mana_blue_dust43() {
        let w = world();
        for (kind, want) in [(12u16, Color::new(255, 0, 0, 255)), (639, Color::new(0, 0, 255, 255))] {
            let mut pool = DustPool::new();
            let mut gate = ScriptRand::new(&[0]);
            let mut main = UnifiedRandom::new(3);
            assert!(orb_shine_sparkle(
                &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, kind,
                Color::new(200, 200, 200, 255),
            ));
            let d = pool.slot(0);
            assert_eq!(d.dust_type, 43);
            assert_eq!(d.alpha, 254);
            assert_eq!(d.color, want);
            assert_eq!(d.velocity, Vec2::ZERO, ":7528 velocity *= 0");
            // scale 0.5: the base roll is 1 + Next(-20,21)*0.01, times 0.5
            // (f32 rounding can land a hair past the exact endpoints).
            assert!(d.scale <= 0.61 && d.scale >= 0.39, "scale band ×0.5: {}", d.scale);
        }
    }

    #[test]
    fn shine_665_is_dead_code_tile_shine_never_written() {
        // Census pin: Main.tileShine[665] has no write (only [12]=300
        // Main.cs:7518, [639]=300 :9652) — the :7511 colour row for 665
        // is unreachable through the :7413 gate.
        let w = world();
        let mut pool = DustPool::new();
        let mut gate = ScriptRand::new(&[]);
        let mut main = UnifiedRandom::new(3);
        let mirror = main.clone();
        assert!(!orb_shine_sparkle(
            &mut gate, &mut main, &mut pool, &ctx(), &w, 4, 4, 665,
            Color::new(255, 255, 255, 255),
        ));
        assert!(gate.draws.is_empty());
        assert_eq!(main, mirror);
    }

    #[test]
    fn break_type_heart_draws_nothing_orb_draws_one_next2() {
        // WorldGen.cs:68968 — the ternary's heart branch skips the draw.
        let mut main = UnifiedRandom::new(11);
        let mirror = main.clone();
        assert_eq!(orb_break_dust_type(31, 36, &mut main), Some(5));
        assert_eq!(orb_break_dust_type(696, 54, &mut main), Some(5));
        assert_eq!(main, mirror, "the heart stage is draw-free");
        // Orb stage: one Next(2) on Main.rand (genRand alias, :4445).
        let mut gate = ScriptRand::new(&[1, 0]);
        assert_eq!(orb_break_dust_type(31, 0, &mut gate), Some(1));
        assert_eq!(orb_break_dust_type(31, 18, &mut gate), Some(14));
        assert_eq!(gate.draws, vec![2, 2]);
        assert_eq!(orb_break_dust_type(12, 0, &mut gate), None);
    }

    #[test]
    fn break_burst_counts_fail3_break10_and_stream_order() {
        let w = world();
        // Heart: 3 fail dusts, all type 5, ZERO stage draws — the pool
        // stream must equal a mirror replaying 3 plain NewDust calls.
        let mut pool = DustPool::new();
        let mut main = UnifiedRandom::new(7);
        assert_eq!(orb_break_burst(&mut main, &mut pool, &ctx(), &w, 4, 4, 31, 36, true), 3);
        let mut mirror_pool = DustPool::new();
        let mut mirror = UnifiedRandom::new(7);
        for _ in 0..3 {
            mirror_pool.new_dust(
                &mut mirror, &ctx(), &w, Vec2::new(64.0, 64.0),
                16, 16, 5, 0.0, 0.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(main, mirror, "heart burst = 3 plain NewDust, no stage draws");
        for k in 0..3 {
            assert_eq!(pool.slot(k).dust_type, 5);
            assert_eq!(pool.slot(k).alpha, 0, ":69166 plain tail alpha 0");
            assert_eq!(pool.slot(k).position, mirror_pool.slot(k).position);
        }
        // Orb break: 10 dusts, each preceded by one Next(2) — the stream
        // diverges from the plain mirror by exactly 10 draws.
        let mut pool2 = DustPool::new();
        let mut main2 = UnifiedRandom::new(7);
        assert_eq!(orb_break_burst(&mut main2, &mut pool2, &ctx(), &w, 4, 4, 31, 0, false), 10);
        let active = pool2.slots().iter().filter(|d| d.active).count();
        assert_eq!(active, 10);
        for d in pool2.slots().iter().take(10) {
            assert!(d.dust_type == 1 || d.dust_type == 14, "orb stage types only");
        }
        // Non-family: nothing, no draws.
        let mut main3 = UnifiedRandom::new(7);
        let m3 = main3.clone();
        let mut pool3 = DustPool::new();
        assert_eq!(orb_break_burst(&mut main3, &mut pool3, &ctx(), &w, 4, 4, 26, 0, true), 0);
        assert_eq!(main3, m3);
    }

    #[test]
    fn entry_gate_pause_and_mode_arms() {
        // TileDrawing.cs:965 — `_isActiveAndNotPaused` gates the whole
        // thing; a paused/menu frame never emits.
        assert!(!emit_entry_gate(false, false, 0x1234, 10, 20));
        assert!(!emit_entry_gate(false, true, 0x1234, 10, 20));
        // `!Lighting.UpdateEveryFrame` — every frame emits, no roll.
        assert!(emit_entry_gate(true, false, 0x1234, 10, 20));
        assert!(emit_entry_gate(true, false, 0, 0, 0));
    }

    #[test]
    fn entry_gate_quarter_roll_is_the_per_tile_fast_random_recipe() {
        // The gate's roll is `FastRandom(TileFrameSeed).WithModifier(x,y)
        // .Next(4) == 0` — a fresh derivation per cell, no stream advance:
        // repeat calls agree, neighbours differ, ~1/4 of cells admit.
        let seed = tr_core::flame::tile_frame_seed_at(37.0);
        let mut admits = 0;
        for x in 0..64 {
            for y in 0..64 {
                let want = tr_core::tile_light::FastRandom::new(seed)
                    .with_modifier(x, y)
                    .next(4)
                    == 0;
                assert_eq!(emit_entry_gate(true, true, seed, x, y), want);
                // Deterministic: the derivation is pure (no stream state).
                assert_eq!(
                    emit_entry_gate(true, true, seed, x, y),
                    emit_entry_gate(true, true, seed, x, y),
                );
                admits += i32::from(want);
            }
        }
        // 4096 cells at 1/4: the band is wide, but a stuck gate (0 or
        // 4096) fails it.
        assert!(admits > 700 && admits < 1400, "quarter band: {admits}");
        // The seed matters (the frame-advance schedule changes the set).
        let other = tr_core::flame::tile_frame_seed_at(37.0 + 5.0 / 60.0);
        let differ = (0..32)
            .filter(|&x| {
                emit_entry_gate(true, true, seed, x, 7)
                    != emit_entry_gate(true, true, other, x, 7)
            })
            .count();
        assert!(differ > 0, "the TileFrameSeed advance re-rolls the set");
    }
}
