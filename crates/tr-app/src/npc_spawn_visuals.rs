//! NPC SPAWN / teleport-in dust — the g61 lane (W4-g61, code-only mode).
//!
//! Scope (the spawn-time visual family; the death dust is g59/g60's
//! `HitEffect`/checkDead lane and the per-hit dust is g62's — neither is
//! touched here):
//!
//! * [`poof_of_smoke`] — `Utils.PoofOfSmoke` (`Utils.cs:1078-1093`): the
//!   statue-spawn / critter-catch poof (3-6 smoke gores 11-13 + ten
//!   type-16 dust). The standing callers: `Wiring.cs:825`/`:2275`/
//!   `:2329`/`:2397` (the statue arms — the port's `wiring.rs:980`
//!   documents the gap), `NPC.cs:79884` (the caught Mystic Frog's
//!   no-spot poof) and `:79913` (the statue-spawned critter catch poof)
//!   — `bug_net.rs:56-61` records the skip this module now has a port
//!   target for; `NPC.cs:90247` (the party-hat-off transition) is the
//!   `MakeTransitionEffectsForTextureChanges` family, cited below.
//! * [`big_mimic_spawn_smoke`] — `NPC.BigMimicSpawnSmoke`
//!   (`NPC.cs:77494-77515`): the biome-mimic chest summon's 20× type-54
//!   smoke + the 3×2 gore-99 grid, called by `BigMimicSummonCheck`
//!   (`:77602`) right after the `NewNPC` (`:77599`).
//! * [`transform_visuals`] — `NPC.TransformVisuals`
//!   (`NPC.cs:80963-81033`): the `Transform` tail's per-kind burst
//!   (`:81136`) — the 158↔159 gore-only arm, the 478→479 flutterfly
//!   30× type-238 + four gores, the 406→405 ring, the 428→427 and
//!   427→426 shimmer-fairy ladders. The port's live
//!   `transform_to_kind` (`npc.rs:5080`) does not call it yet — the
//!   wiring round drains it at the C# call position.
//!
//! The teleport-in family ([`teleport_effect`], [`npc_teleport`],
//! [`update_npc_teleport_visuals`]) lands in the follow-up commits of
//! this lane.
//!
//! **`NewNPC` itself draws no spawn dust.** `NPC.NewNPC`
//! (`NPC.cs:80750-80795`) has exactly ONE RNG draw — the
//! `Main.getGoodWorld && Main.rand.Next(3) != 0` bunny/squirrel swap
//! (`:80752-80762`, the 46→614 / 62→66 pair) — and `OnSpawn`
//! (`:8402-8424`) only stamps the boss presence statics. The per-kind
//! spawn smoke lives in the CALLERS above; the sim's spawn seams
//! already carry the getGoodWorld draw constants-off (the
//! `world_sim/npcs.rs` ftw notes).
//!
//! RNG contract (the d50 `DustPool` contract, S1-E-1): every draw rides
//! the caller's ONE stream in the C# evaluation order — argument
//! expressions draw BEFORE the `NewDust`/`NewGore` internals they feed
//! (`Utils.SelectRandom`'s `Next(len)` at `Utils.cs:2459-2462`, the
//! `684 + Main.rand.Next(3)` gore type at `NPC.cs:80989`), and the
//! post-writes' draws (`Main.rand.Next(2)` noGravity gates, the scale /
//! fadeIn `NextFloat`s) run AFTER the allocator returned — the 6000
//! sentinel slot included (`Main.dust[6000]` exists, the pool keeps
//! it). The off-screen cull / menu guards refuse with ZERO draws
//! (`Dust.cs:77-101`); the gore allocator's own draws (`Gore.cs:1321-
//! 1333`'s velocity pair, the ChildSafety re-type when armed) ride the
//! same stream through `GorePool::new_gore`.
//!
//! Drops (single-player port, cited at the sites): the `netMode == 2`
//! server arms (`BigMimicSpawnSmoke`'s `SendData(51, .., 4)` relay
//! `:77496-77500`, the `Transform` `SendData(23/54)` pair) and the
//! `SoundEngine` pitch draws — the sounds ride the [`SpawnVisualsEnv::
//! play_sound`] callback AT the C# call position so the wiring round
//! can roll the case-2 `Next(-6,7)` pitch (`LegacySoundPlayer.cs:632`,
//! the style75 convention) on the same stream without re-ordering.
//!
//! Witnesses: the tests below record EVERY stream call through a
//! `RecordingRng` and pin the exact slot sequence per site (WRITTEN
//! NOT RUN — code-only mode, no cargo per the lane directive).
#![allow(dead_code)] // UNWIRED (the g61 code-only round): the wiring follow-up drains the poof at the bug-net/statue seams, the mimic smoke at the key-summon seam and TransformVisuals at transform_to_kind's tail.

use crate::npc::Npc;
use glam::Vec2;
use tr_core::gore::{GorePool, GoreRng};
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};
use tr_core::World;

/// The per-call environment the spawn visuals read through `Main`'s
/// statics — the style56 `Env` seam shape, widened with the gore pool
/// (the poof/mimic/transform sites spawn gores on the SAME stream) and
/// the inline sound callback (the pitch draw must land at the C# call
/// position, between the dust draws — a deferred `out.sounds` vec
/// would re-order the stream).
pub struct SpawnVisualsEnv<'a, R: DustRng + GoreRng> {
    /// The read-only world the gore allocator's out-of-world probe reads.
    pub world: &'a World,
    /// `Main.dust` — the shared 6001-slot pool (S1-E-1).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// `Main.gore` — the shared 601-slot pool (S1-E-4).
    pub gore: &'a mut GorePool,
    /// The caller's stream (`Main.rand`'s per-consumer analogue — the
    /// `UpdateNPCs` swap for the Transform tail, `UpdatePlayers` for
    /// the bug-net poof, `SpawnNPC` for the statue arms).
    pub rng: &'a mut R,
    /// `SoundEngine.PlaySound(LegacySoundStyle, x, y)` at the C# call
    /// position: `(sound_id, style, x, y)`. The wiring round resolves
    /// the cue + its pitch draw here (the style75 `play_use_sound`
    /// seam); the pure module only records the request.
    pub play_sound: &'a mut dyn FnMut(i32, i32, f32, f32),
}

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): `(cos f, sin f)`
/// in double, cast to f32 (the style75 helper, same maths).
fn to_rotation_vector2(f: f32) -> Vec2 {
    Vec2::new((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`, centre = zero): cos/sin in
/// DOUBLE cast to f32, then `x·c − y·s` / `x·s + y·c` in that order.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Utils.PoofOfSmoke` (`Utils.cs:1078-1093`), verbatim: the
/// `Next(3,7)` gore count, then per gore the velocity `NextFloat`
/// BEFORE the type `Next(11,14)` (argument order), the allocator's own
/// two draws, and the `scale = 0.7` / `velocity *= 0.5` post-writes;
/// then ten type-16 dust (alpha 100, scale 1.5) with the `+5` position
/// nudge and the two-`NextFloat` velocity overwrite.
pub fn poof_of_smoke<R: DustRng + GoreRng>(env: &mut SpawnVisualsEnv<'_, R>, position: Vec2) {
    // `:1080` — the gore count draws FIRST, on the caller's stream.
    let num = DustRng::next_range(env.rng, 3, 7);
    for _ in 0..num {
        // `:1083` — the velocity argument draws before the type
        // argument: `NextFloat() * 2πf` → ToRotationVector2 → ×(2,0.7)
        // → ×0.7, then `Next(11,14)`; NewGore's default Scale is 1f.
        let rot = DustRng::next_float(env.rng) * (std::f32::consts::PI * 2.0);
        let vel = to_rotation_vector2(rot) * Vec2::new(2.0, 0.7) * 0.7;
        let gore_type = DustRng::next_range(env.rng, 11, 14);
        let idx = env
            .gore
            .new_gore(env.rng, env.world, position, vel, gore_type, 1.0);
        // `:1084-1085` — the post-writes, sentinel slot included.
        let g = env.gore.slot_mut(idx);
        g.scale = 0.7;
        g.velocity *= 0.5;
    }
    for _ in 0..10 {
        // `:1089` — Dust.NewDust(position, 14, 14, 16, 0, 0, 100,
        // default, 1.5). Type 16 hits no per-type tweak arm
        // (`Dust.cs:214-252`) and is not DangerousDust.
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            position,
            14,
            14,
            16,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        // `:1090` — position += (5,5) BEFORE the velocity overwrite.
        env.dust.slot_mut(idx).position += Vec2::splat(5.0);
        // `:1091` — the rotation NextFloat draws first, then the
        // magnitude NextFloat inside `(0.5f + 0.5f * NextFloat())`.
        let rot = DustRng::next_float(env.rng) * (std::f32::consts::PI * 2.0);
        let vel = to_rotation_vector2(rot) * Vec2::new(2.0, 0.7) * 0.7
            * (0.5 + 0.5 * DustRng::next_float(env.rng));
        env.dust.slot_mut(idx).velocity = vel;
    }
}

/// `NPC.BigMimicSpawnSmoke` (`NPC.cs:77494-77515`). The
/// `Main.netMode == 2` relay (`SendData(51, -1, -1, null, whoAmI, 4f)`
/// + return, `:77496-77500`) is the server arm — dropped (single-player
/// port; the client's `51` handler re-runs this body, `NetMessage.cs`
/// case 51's NPC-smoke number 4).
pub fn big_mimic_spawn_smoke<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    position: Vec2,
    width: i32,
    height: i32,
) {
    // `:77501-77506` — 20× type-54 smoke: alpha 50, scale 1.5, then
    // velocity ×2 + noGravity. Type 54 hits no tweak arm.
    for _ in 0..20 {
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            position,
            width,
            height,
            54,
            0.0,
            0.0,
            50,
            Color::default(),
            1.5,
        );
        let d = env.dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.no_gravity = true;
    }
    // `:77507-77514` — the 3×2 gore-99 grid. `(float)(base.width * j)`
    // is the INTEGER product cast to f32, then ×0.45f; the Y row is
    // `(float)(base.height * k) * 0.5f + 10f`. Velocity (0,0), scale
    // 1.1, post ×0.3. Gore 99 is SafeGore + the sticky-exempt list
    // (`Gore.cs:1341-1344`) — the allocator draws only its velocity pair.
    for j in 0..3 {
        for k in 0..2 {
            let p = Vec2::new(
                position.x + (width * j) as f32 * 0.45 - 20.0,
                position.y + (height * k) as f32 * 0.5 + 10.0,
            );
            let idx = env
                .gore
                .new_gore(env.rng, env.world, p, Vec2::ZERO, 99, 1.1);
            env.gore.slot_mut(idx).velocity *= 0.3;
        }
    }
}

/// `NPC.TransformVisuals` (`NPC.cs:80963-81033`) — the `Transform`
/// tail's visual burst (`:81136`), over the port's `Npc`. The
/// `position += netOffset` / `-= netOffset` round trip (`:80965` /
/// `:81032`) is kept verbatim (the style56 precedent — the float
/// rounding of the restored position must match). The first three
/// arms are an if/else-if chain; the 428→427 and 427→426 arms are
/// SEPARATE `if`s in the C# (`:81008` / `:81020`) — kept separate.
pub fn transform_visuals<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    npc: &mut Npc,
    old_type: i32,
    new_type: i32,
) {
    // `:80965` — the netOffset round trip head.
    npc.x += npc.net_offset.x;
    npc.y += npc.net_offset.y;
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let (wf, hf) = npc.size();
    let (w, h) = (wf as i32, hf as i32);
    if (old_type == 158 && new_type == 159) || (old_type == 159 && new_type == 158) {
        // `:80966-80975` — the sleepy/awake zombie pair: NO dust, the
        // Item8 cue + three gore-99 puffs at the top / integer-half /
        // bottom edges (−10f each), each velocity ×0.3.
        // `(float)(base.height / 2)` is INTEGER division (`:80971`).
        (env.play_sound)(2, 8, pos.x, pos.y); // SoundID.Item8 = (2,8), SoundID.cs:1391
        for y in [
            pos.y - 10.0,
            pos.y + (h / 2) as f32 - 10.0,
            pos.y + h as f32 - 10.0,
        ] {
            let idx = env.gore.new_gore(
                env.rng,
                env.world,
                Vec2::new(pos.x, y),
                vel,
                99,
                npc.scale,
            );
            env.gore.slot_mut(idx).velocity *= 0.3;
        }
    } else if old_type == 478 && new_type == 479 {
        // `:80976-80990` — the flutterfly hatch: 30× type-238 dust
        // (SpeedY −2), each with a `Next(2) == 0` noGravity gate drawn
        // AFTER the allocator; then gores 684/685/686 + one
        // `684 + Next(3)` — the type draw lands BEFORE that gore's
        // allocator draws. 684-686 are NOT SafeGore: with the blood
        // gate armed the allocator re-types (its own Next(11,14) +
        // NextFloat, `Gore.cs:1334-1340`).
        for _ in 0..30 {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                pos,
                w,
                h,
                238,
                0.0,
                -2.0,
                0,
                Color::default(),
                1.0,
            );
            if DustRng::next_max(env.rng, 2) == 0 {
                env.dust.slot_mut(idx).no_gravity = true;
            }
        }
        for t in [684, 685, 686] {
            env.gore
                .new_gore(env.rng, env.world, pos, vel, t, npc.scale);
        }
        let t = 684 + DustRng::next_max(env.rng, 3);
        env.gore
            .new_gore(env.rng, env.world, pos, vel, t, npc.scale);
    } else if old_type == 406 && new_type == 405 {
        // `:80991-81007` — the ring: `Next(10,21)` count (stored as
        // float), one `NextFloat` phase, then per dust the 0×0 type-229
        // spawn at the FLOAT-half Centre, the UnitY.RotatedBy seat,
        // the ×(22,22)/2 position offset, and the scale/fadeIn
        // NextFloat pair (scale BEFORE fadeIn, `:81003`/`:81005`).
        let num3 = DustRng::next_range(env.rng, 10, 21) as f32;
        let num4 = (std::f32::consts::PI * 2.0) * DustRng::next_float(env.rng);
        let vector = Vec2::new(22.0, 22.0);
        let (cx, cy) = npc.center();
        let center = Vec2::new(cx, cy);
        let mut num5 = 0.0f32;
        while num5 < num3 {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                center,
                0,
                0,
                229,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            // `:80999` — the angle is the f32 expression
            // `num5 * (2πf) / num3 + num4`, promoted to double at the
            // RotatedBy call.
            let angle = num5 * (std::f32::consts::PI * 2.0) / num3 + num4;
            let v2 = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
            let scale = 0.6 + DustRng::next_float(env.rng) * 1.8;
            let fade_in = DustRng::next_float(env.rng) * 2.0;
            let d = env.dust.slot_mut(idx);
            d.position = center + v2 * vector / 2.0;
            d.velocity = v2;
            d.no_gravity = true;
            d.scale = scale;
            d.velocity *= d.scale;
            d.fade_in = fade_in;
            num5 += 1.0;
        }
    }
    if old_type == 428 && new_type == 427 {
        // `:81008-81019` — the shimmer-fairy step: 20× SelectRandom
        // (229, 240) — ONE `Next(2)` per dust, drawn BEFORE the
        // allocator (`Utils.cs:2459-2462`); noGravity, scale
        // 0.6+NextFloat×1.8, fadeIn 0.25, velocity ×scale/2.
        for _ in 0..20 {
            let t = [229, 240][DustRng::next_max(env.rng, 2) as usize];
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                pos,
                w,
                h,
                t,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            let scale = 0.6 + DustRng::next_float(env.rng) * 1.8;
            let d = env.dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = scale;
            d.fade_in = 0.25;
            d.velocity *= d.scale / 2.0;
        }
    }
    if old_type == 427 && new_type == 426 {
        // `:81020-81031` — the final step: 30× SelectRandom
        // (229, 240, 240) — ONE `Next(3)`, the doubled 240 weight is
        // the C#'s own; velocity ×scale×0.75 (NOT /2).
        for _ in 0..30 {
            let t = [229, 240, 240][DustRng::next_max(env.rng, 3) as usize];
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                pos,
                w,
                h,
                t,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            let scale = 0.6 + DustRng::next_float(env.rng) * 1.8;
            let d = env.dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = scale;
            d.fade_in = 0.25;
            d.velocity *= d.scale * 0.75;
        }
    }
    // `:81032` — the netOffset round trip tail.
    npc.x -= npc.net_offset.x;
    npc.y -= npc.net_offset.y;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::particles::DUST_SENTINEL;
    use tr_core::UnifiedRandom;

    /// One recorded stream call — the exact RNG-slot witness carrier.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Draw {
        Max(i32),
        Range(i32, i32),
        Float,
    }

    /// The recording seam: forwards every draw to a real
    /// `UnifiedRandom` and logs the call shape, so a witness pins the
    /// C#'s draw ORDER without hand-tracing values.
    struct RecordingRng {
        inner: UnifiedRandom,
        log: Vec<Draw>,
    }

    impl RecordingRng {
        fn new(seed: i32) -> Self {
            Self {
                inner: UnifiedRandom::new(seed),
                log: Vec::new(),
            }
        }
    }

    impl DustRng for RecordingRng {
        fn next_max(&mut self, max: i32) -> i32 {
            self.log.push(Draw::Max(max));
            self.inner.next_max(max)
        }
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.log.push(Draw::Range(min, max));
            self.inner.next_range(min, max)
        }
        fn next_float(&mut self) -> f32 {
            self.log.push(Draw::Float);
            self.inner.next_float()
        }
    }

    impl GoreRng for RecordingRng {
        fn next_max(&mut self, max: i32) -> i32 {
            DustRng::next_max(self, max)
        }
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            DustRng::next_range(self, min, max)
        }
        fn next_float(&mut self) -> f32 {
            DustRng::next_float(self)
        }
        fn next_float_direction(&mut self) -> f32 {
            // No g61 site draws NextFloatDirection (the gore types
            // here hit no SpecialAI arm) — record it as a Float so an
            // unexpected call still shows in the log.
            self.log.push(Draw::Float);
            self.inner.next_float_direction()
        }
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// On-screen ctx: the (100,100) box sits inside the 400-px cull
    /// margin; maxDustToDraw 6000 puts slot 0 in the low band (no gate
    /// draw) — the style56 witness convention.
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The six `NewDust` default-block draws for a w×h box
    /// (`Dust.cs:170-190`): pos X/Y, vel X/Y, frame row, scale.
    fn dust_block(w: i32, h: i32) -> Vec<Draw> {
        let nw = w.max(5) - 4;
        let nh = h.max(5) - 4;
        vec![
            Draw::Max(nw),
            Draw::Max(nh),
            Draw::Range(-20, 21),
            Draw::Range(-20, 21),
            Draw::Max(3),
            Draw::Range(-20, 21),
        ]
    }

    /// The gore allocator's default-block velocity pair
    /// (`Gore.cs:1327-1328`) — the only draws for the safe, non-special
    /// types 11-13/99 with the blood gate off.
    fn gore_block() -> Vec<Draw> {
        vec![Draw::Range(10, 31), Draw::Range(-20, 21)]
    }

    /// `Utils.cs:1080-1092` — the poof's exact slot sequence: the
    /// count, then per gore [vel NextFloat, type Next(11,14), the
    /// allocator pair], then ten × [the 14×14 dust block, two
    /// NextFloats].
    #[test]
    fn w4_g61_poof_rng_slots_are_exact() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(42);
        let mut sounds = Vec::new();
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y| sounds.push((a, b, x, y)),
        };
        poof_of_smoke(&mut env, Vec2::new(100.0, 100.0));
        let num = UnifiedRandom::new(42).next_range(3, 7);
        let mut expected = vec![Draw::Range(3, 7)];
        for _ in 0..num {
            expected.push(Draw::Float);
            expected.push(Draw::Range(11, 14));
            expected.extend(gore_block());
        }
        for _ in 0..10 {
            expected.extend(dust_block(14, 14));
            expected.push(Draw::Float);
            expected.push(Draw::Float);
        }
        assert_eq!(rng.log, expected, "the poof's draw order is the C#'s");
        assert!(sounds.is_empty(), "the poof plays no sound");
        // The ten dust carry alpha 100 / type 16 and the +5 nudge.
        for i in 0..10 {
            let d = dust.slot(i);
            assert!(d.active);
            assert_eq!(d.dust_type, 16);
            assert_eq!(d.alpha, 100);
            // 100 + Next(10) + 4 (the 14-wide box roll) + 5 (the nudge).
            assert!(d.position.x >= 109.0 && d.position.x <= 118.0);
        }
        // The gores end at scale 0.7 (the post-write overwrites the
        // allocator's 1f).
        for i in 0..num as usize {
            assert_eq!(gore.slot(i).scale, 0.7);
            assert!((11..14).contains(&gore.slot(i).gore_type));
        }
    }

    /// `NPC.cs:77501-77514` — 20 dust blocks then six gore pairs; the
    /// velocity ×2 / noGravity post-writes and the integer-product
    /// grid positions.
    #[test]
    fn w4_g61_big_mimic_smoke_slots_and_grid() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(7);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        big_mimic_spawn_smoke(&mut env, Vec2::new(100.0, 100.0), 48, 48);
        let mut expected = Vec::new();
        for _ in 0..20 {
            expected.extend(dust_block(48, 48));
        }
        for _ in 0..6 {
            expected.extend(gore_block());
        }
        assert_eq!(rng.log, expected);
        for i in 0..20 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 54);
            assert_eq!(d.alpha, 50);
            assert!(d.no_gravity);
        }
        // The grid: j=0..2 columns at x + (48*j)*0.45 − 20, k=0..1
        // rows at y + (48*k)*0.5 + 10 — slot order j-major.
        for j in 0..3 {
            for k in 0..2 {
                let g = gore.slot(j * 2 + k);
                assert_eq!(g.gore_type, 99);
                assert_eq!(g.scale, 1.1);
                // The allocator's velocity pair then ×0.3: the X
                // component is exactly Next(-20,21)*0.1*0.3.
                assert_eq!(g.position.x, 100.0 + (48 * j as i32) as f32 * 0.45 - 20.0);
                assert_eq!(g.position.y, 100.0 + (48 * k as i32) as f32 * 0.5 + 10.0);
            }
        }
    }

    /// `Dust.cs:97-101` — the off-screen cull refuses with ZERO dust
    /// draws, but the C# still writes into `Main.dust[6000]` and the
    /// gore loop still rolls: the sentinel writes + the gore-only
    /// stream.
    #[test]
    fn w4_g61_offscreen_smoke_writes_the_sentinel_and_keeps_gore_draws() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let mut c = ctx();
        c.screen_position = Vec2::new(100_000.0, 100_000.0);
        let mut rng = RecordingRng::new(9);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        big_mimic_spawn_smoke(&mut env, Vec2::new(100.0, 100.0), 48, 48);
        let mut expected = Vec::new();
        for _ in 0..6 {
            expected.extend(gore_block());
        }
        assert_eq!(rng.log, expected, "the cull draws nothing");
        let s = dust.slot(DUST_SENTINEL);
        assert!(s.no_gravity, "the ×2/noGravity post-writes hit 6000");
        assert!(!dust.slot(0).active, "no real slot was taken");
    }

    /// `NPC.cs:80976-80990` — the 478→479 arm: per dust the block then
    /// the `Next(2)` gate; the four gores last, the fourth's `Next(3)`
    /// BEFORE its allocator pair.
    #[test]
    fn w4_g61_transform_478_479_gate_and_gore_order() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(11);
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((24.0, 24.0));
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        transform_visuals(&mut env, &mut npc, 478, 479);
        let mut expected = Vec::new();
        for _ in 0..30 {
            expected.extend(dust_block(24, 24));
            expected.push(Draw::Max(2));
        }
        for _ in 0..3 {
            expected.extend(gore_block());
        }
        expected.push(Draw::Max(3));
        expected.extend(gore_block());
        assert_eq!(rng.log, expected);
        for i in 0..30 {
            assert_eq!(dust.slot(i).dust_type, 238);
        }
        let types: Vec<i32> = (0..4).map(|i| gore.slot(i).gore_type).collect();
        assert_eq!(&types[..3], &[684, 685, 686]);
        assert!((684..687).contains(&types[3]));
    }

    /// `NPC.cs:80991-81007` — the 406→405 ring: the count + phase
    /// draws, then per dust the 0×0 block (the 5-clamp → Next(1)
    /// pair) + scale/fadeIn NextFloats; every seat sits 11 px from
    /// the centre and |velocity| == scale.
    #[test]
    fn w4_g61_transform_406_405_ring_geometry_and_slots() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(13);
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((20.0, 20.0));
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        transform_visuals(&mut env, &mut npc, 406, 405);
        let count = UnifiedRandom::new(13).next_range(10, 21);
        let mut expected = vec![Draw::Range(10, 21), Draw::Float];
        for _ in 0..count {
            expected.extend(dust_block(0, 0));
            expected.push(Draw::Float);
            expected.push(Draw::Float);
        }
        assert_eq!(rng.log, expected);
        let center = Vec2::new(110.0, 110.0);
        for i in 0..count as usize {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 229);
            assert!(d.no_gravity);
            let r = (d.position - center).length();
            assert!((r - 11.0).abs() < 1e-3, "the 22/2 ring radius");
            assert!((d.velocity.length() - d.scale).abs() < 1e-4);
            assert!(d.scale >= 0.6 && d.scale < 2.4);
            assert!(d.fade_in >= 0.0 && d.fade_in < 2.0);
        }
    }

    /// `NPC.cs:81008-81031` — the fairy ladders: SelectRandom's
    /// Next(2)/Next(3) BEFORE each dust block, one scale NextFloat
    /// after; the velocity multipliers differ (scale/2 vs scale×0.75)
    /// and the fadeIn is the 0.25 constant.
    #[test]
    fn w4_g61_transform_fairy_ladders_select_random_and_multipliers() {
        for (old, new, n, gate) in [(428, 427, 20, 2), (427, 426, 30, 3)] {
            let w = air();
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(17);
            let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((20.0, 20.0));
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _| {},
            };
            transform_visuals(&mut env, &mut npc, old, new);
            let mut expected = Vec::new();
            for _ in 0..n {
                expected.push(Draw::Max(gate));
                expected.extend(dust_block(20, 20));
                expected.push(Draw::Float);
            }
            assert_eq!(rng.log, expected, "{old}→{new}");
            // Replay the stream to check each dust's type pick and
            // velocity multiplier against the rolled values.
            let mut replay = UnifiedRandom::new(17);
            for i in 0..n as usize {
                let pick = replay.next_max(gate);
                let want_type = if gate == 2 {
                    [229, 240][pick as usize]
                } else {
                    [229, 240, 240][pick as usize]
                };
                let _px = replay.next_max(16);
                let _py = replay.next_max(16);
                let vx = replay.next_range(-20, 21) as f32 * 0.1;
                let vy = replay.next_range(-20, 21) as f32 * 0.1;
                let _row = replay.next_max(3);
                let rolled = 1.0 + replay.next_range(-20, 21) as f32 * 0.01;
                let scale = 0.6 + replay.next_float() * 1.8;
                let d = dust.slot(i);
                assert_eq!(d.dust_type, want_type);
                assert_eq!(d.scale, scale);
                assert_eq!(d.fade_in, 0.25);
                assert!(d.no_gravity);
                let mult = if gate == 2 { scale / 2.0 } else { scale * 0.75 };
                assert_eq!(d.velocity.x, vx * mult);
                assert_eq!(d.velocity.y, vy * mult);
                let _ = rolled; // the rolled scale is overwritten (assignment)
            }
        }
    }

    /// `NPC.cs:80965`/`:81032` — the netOffset round trip restores the
    /// position exactly, and the dust spawns at the OFFSET position.
    #[test]
    fn w4_g61_transform_net_offset_round_trip() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(19);
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((20.0, 20.0));
        npc.net_offset = Vec2::new(3.5, -2.25);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _| {},
        };
        transform_visuals(&mut env, &mut npc, 428, 427);
        assert_eq!((npc.x, npc.y), (100.0, 100.0), "add-then-subtract restores");
        for i in 0..20 {
            let d = dust.slot(i);
            assert!(d.position.x >= 103.5 + 4.0 && d.position.x <= 103.5 + 19.0);
            assert!(d.position.y >= 97.75 + 4.0 && d.position.y <= 97.75 + 19.0);
        }
    }

    /// `NPC.cs:80966-80975` — the 158↔159 arm is gore-only: the Item8
    /// cue at the position, three gore pairs, the INTEGER half-height
    /// seat, and NO dust draws.
    #[test]
    fn w4_g61_transform_158_159_is_gore_only_with_the_item8_cue() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(23);
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((18.0, 41.0)); // odd height: 41/2 = 20
        let mut sounds = Vec::new();
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y| sounds.push((a, b, x, y)),
        };
        transform_visuals(&mut env, &mut npc, 159, 158);
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.extend(gore_block());
        }
        assert_eq!(rng.log, expected);
        assert_eq!(sounds, vec![(2, 8, 100.0, 100.0)]);
        assert_eq!(gore.slot(1).position.y, 100.0 + 20.0 - 10.0);
        assert_eq!(gore.slot(2).position.y, 100.0 + 41.0 - 10.0);
        assert!(!dust.slot(0).active);
    }
}
