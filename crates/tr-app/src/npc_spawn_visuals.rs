//! NPC SPAWN / teleport-in dust — the g61 lane (W4-g61, code-only mode;
//! the wiring round connects the live seams below).
//!
//! Scope (the spawn-time visual family; the death dust is g59/g60's
//! `HitEffect`/checkDead lane and the per-hit dust is g62's — neither is
//! touched here):
//!
//! * [`poof_of_smoke`] — `Utils.PoofOfSmoke` (`Utils.cs:1078-1093`): the
//!   statue-spawn / critter-catch poof (3-6 smoke gores 11-13 + ten
//!   type-16 dust). WIRED at the statue/lamp arms (`Wiring.cs:825`/
//!   `:2275`/`:2329`/`:2397` — the `world_sim/wiring.rs` drain) and the
//!   bug-net catch (`NPC.cs:79884` the caught Mystic Frog's no-spot
//!   poof, `:79913` the statue-spawned critter catch poof —
//!   `bug_net.rs::catch_npc`); `NPC.cs:90247` (the party-hat-off
//!   transition) is the `MakeTransitionEffectsForTextureChanges`
//!   family, cited below (the town alt-texture seam is unported —
//!   the `npc_ai_town.rs` shimmer-state note).
//! * [`big_mimic_spawn_smoke`] — `NPC.BigMimicSpawnSmoke`
//!   (`NPC.cs:77494-77515`): the biome-mimic chest summon's 20× type-54
//!   smoke + the 3×2 gore-99 grid, called by `BigMimicSummonCheck`
//!   (`:77602`) right after the `NewNPC` (`:77599`). WIRED at the
//!   `chest.rs::big_mimic_summon_check` spawn seam (the key-summon
//!   `ChestChangeEvents` edge, `Player.cs:28760-28775`); the big-mimic
//!   kinds 473-476 are roster-gapped (the d40 kind lane), so the smoke
//!   fires only once a kind row lands.
//! * [`transform_visuals`] — `NPC.TransformVisuals`
//!   (`NPC.cs:80963-81033`): the `Transform` tail's per-kind burst
//!   (`:81136`) — the 158↔159 gore-only arm, the 478→479 flutterfly
//!   30× type-238 + four gores, the 406→405 ring, the 428→427 and
//!   427→426 shimmer-fairy ladders. WIRED at the live in-place
//!   transform tails (the 406→405 cell, the 428→427 / 427→426 vortex
//!   ladder — `npc.rs`'s `wants_transform_visuals` drain) and the
//!   `transform_to_kind` seam (the 382/518 carrier-gone pair, a
//!   zero-draw parity call); the 158/159 vampire + 478/479 mothron
//!   arms are roster-gapped (the d35/d40 kind lanes).
//!
//! The teleport-in family (FOLD-SOURCE, already consumed — see the
//! KEEP-ONE note below): [`teleport_effect`] (`Main.TeleportEffect`,
//! `Main.cs:13233-13464` — all eleven style arms), [`npc_teleport`]
//! (`NPC.Teleport`, `NPC.cs:81035-81058` — the two-sided Entry/Exit
//! driver), [`update_npc_teleport_visuals`]
//! (`NPC.UpdateNPC_TeleportVisuals`, `NPC.cs:92194-92241` — the
//! per-tick fade, styles 0/4/12/13) and
//! [`make_transition_effects_for_texture_changes`]
//! (`NPC.cs:90236-90263` — the town alt-texture puffs, the party-hat /
//! zoologist transitions — this one STAYS g61's: the town alt-texture
//! seam is outside the g11 union).
//!
//! **KEEP-ONE (ADJUDICATED + EXECUTED on the g11 branch — g11 research/116
//! §3/§4/§5, union-exec `56afc0c0` + `8540a8ad` + the final-phase
//! `cc8c767b`/`98283df0`):** the surviving copy of this module's
//! [`teleport_effect`] body is **g11's `teleport_dust.rs`** (adapted
//! per-hunk from this module at `a9628928`; `TeleportDustEnv` =
//! [`SpawnVisualsEnv`] minus the gore pool — no teleport arm spawns
//! gores). The g11 SEAMS are canonical: `main_dust::teleport_effect` is
//! the App-level dispatcher (it builds the env + resolves the Audio
//! façade, then calls the body for ALL styles); the Entry-OLD →
//! Exit-NEW `PlayerTeleportReq`/`NpcTeleportReq` replays own the call
//! sites (`other_position` per gap 2, the style-9 pylon latch per gap 3,
//! the stored `water_style` per gap 4). This module's env-level
//! [`npc_teleport`] driver is RETIRED at the merge (the g11 App-level
//! replay supersedes it), [`update_npc_teleport_visuals`] survives
//! re-signed to the g11 `Npc` fields (`teleport_style_id` naming — the
//! `(f32, i32, i32)` triple is retired), WIRED PRE-AI in the `UpdateNPC`
//! slot preamble (`NPC.cs:90322` after the cast lights, BEFORE `AI()`
//! `:90423`) fed `tfv = anim_time*60 % 216000` (the §5 fold of this
//! branch's seam-4 order). The statue-14 grants ride the SAME
//! `NpcTeleportReq` queue as the pads (§5 Fold B). This module's
//! teleport family + its 9 teleport witnesses are the FOLD SOURCE —
//! already consumed (8 adapted as `w4_g11u_*` in `teleport_dust.rs` +
//! the driver witness in `main_dust.rs`); the MERGER DROPS them here
//! and keeps g11's copies. The earlier "supersedes g11 main_dust styles
//! 0-5/7" note (commit `1eedb9a3`) was the INVERSE direction and is
//! RETIRED — both branches agreed the g61 BODY survives; g11 executed
//! the fold on ITS branch. The poof/mimic/transform/alt-texture halves
//! + their 8 witnesses STAY g61's (research/116 §4 merger-drops note);
//! the g61b seam-2/3/5 wirings are NO-DUP (the §5 census). The g61b
//! seam-4 `npc_teleport_wiring` helper is the one DUP RISK — the merger
//! keeps the g11 QUEUE replay and drops the helper with its env-level
//! driver (the statue + fade semantics already landed on g11).
//! `Audio::play_teleport_cue` may hoist into the g11 dispatcher closure
//! with zero behaviour change (the §5 audio note). The g11 `snowing`
//! site is separately adjudicated (research/83, d130-canonical).
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
//! play_sound`] callback AT the C# call position WITH the caller's
//! stream, so the wiring round rolls the case-2 `Next(-6,7)` pitch
//! (`LegacySoundPlayer.cs:632`, the style75 convention) on the same
//! stream without re-ordering.
//!
//! Witnesses: the tests below record EVERY stream call through a
//! `RecordingRng` and pin the exact slot sequence per site (WRITTEN
//! NOT RUN — code-only mode, no cargo per the lane directive). The 9
//! `w4_g61_teleport_*`/`w4_g61_npc_teleport_*` witnesses are the FOLD
//! SOURCE too — g11 union-adapted 8 of them as `w4_g11u_*` in
//! `teleport_dust.rs` + the driver witness as
//! `w4_g11_npc_teleport_style4_two_sided_burst_and_latch` in
//! `main_dust.rs` (research/116 §4); the merger drops the g61 copies.
//! The 8 poof/mimic/transform/alt-texture witnesses + the wiring-seam
//! witnesses STAY g61's.
#![allow(dead_code)] // The wiring round drains the poof at the bug-net/statue seams, the mimic smoke at the key-summon seam, TransformVisuals at the live transform tails and the teleport family at the statue/pad call sites; arms whose kinds are roster-gapped (the 158/159 vampire pair, the 478/479 mothron hatch, the big mimics) keep their parity code unwired until the kind lanes land.

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
    /// position: `(sound_id, style, x, y, rng)`. The wiring round resolves
    /// the cue + its pitch draw here (the style75 `play_use_sound`
    /// seam); the pure module only records the request. The caller's
    /// stream is handed THROUGH the callback (a disjoint-field reborrow
    /// of [`Self::rng`]) so the case-2 `Next(-6,7)` pitch lands at the
    /// C# position between the dust draws — a deferred `out.sounds` vec
    /// would re-order the stream.
    pub play_sound: &'a mut dyn FnMut(i32, i32, f32, f32, &mut R),
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
        (env.play_sound)(2, 8, pos.x, pos.y, env.rng); // SoundID.Item8 = (2,8), SoundID.cs:1391
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

/// `Main.TeleportEffect` (`Main.cs:13233-13464`) — the teleport-in /
/// teleport-out burst family. `NPC.Teleport` (`NPC.cs:81035-81058`)
/// calls it TWICE (Entry at the old rect, Exit at the new) — the `side`
/// parameter is unread inside the C# body (only style 10 reads
/// `otherPosition`), so the port drops it. All eleven C# arms:
/// 0/1/2/3/4/5/7/9/10/11/13 (6/8 have no arm).
///
/// DEPRECATED (FOLD-SOURCE — g11 research/116 §3/§4, executed at g11
/// `56afc0c0`): the surviving copy is g11's `teleport_dust.rs`
/// (this body adapted per-hunk at `a9628928`); the merger drops THIS
/// fn and keeps the g11 dispatcher → body path. Kept compiling until
/// the merge so this branch's witnesses stay runnable.
///
/// The `effectRect` is `NPC.getRect()` (`NPC.cs:90025-90028`): the
/// INTEGER-cast top-left + the integer width/height. Styles 9/10
/// `Inflate(15,15)` mutate the rect FIRST (X−15, Y−15, W+30, H+30).
///
/// `PortalHelper.GetPortalColor` (`PortalHelper.cs:373-392`) rides the
/// `netMode == 0` arm: portal 0 → hsl(0.12, 1, 0.5), else
/// hsl(0.52, 1, 0.6), alpha 66 — then style 4's `obj.color.A = 255`
/// overwrite (`Main.cs:13330`).
#[allow(clippy::too_many_arguments)]
pub fn teleport_effect<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    mut rect: (i32, i32, i32, i32), // (X, Y, Width, Height)
    style: i32,
    extra_info: i32,
    dust_count_mult: f32,
    other_position: Vec2,
    water_style: i32,
) {
    match style {
        0 | 1 => {
            // `:13237-13274` — the wizard (0: type 159, Item6) and
            // rod-of-discord (1: type 164, Item8) arms share the shape:
            // the INTEGER `Width*Height/5` count ×mult, per dust the
            // allocator then the `Next(20,70)*0.01` scale ASSIGNMENT +
            // the j<10 / j<5 +0.25 bumps.
            let dust_type = if style == 0 { 159 } else { 164 };
            let sound_style = if style == 0 { 6 } else { 8 };
            (env.play_sound)(
                2,
                sound_style,
                (rect.0 + rect.2 / 2) as f32,
                (rect.1 + rect.3 / 2) as f32,
                env.rng,
            );
            let count = ((rect.2 * rect.3 / 5) as f32 * dust_count_mult) as i32;
            for j in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    dust_type,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let mut scale = DustRng::next_range(env.rng, 20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                env.dust.slot_mut(idx).scale = scale;
            }
        }
        2 => {
            // `:13275-13283` — the Chaos-State ghost puff: 50×mult
            // type-58 dust, alpha 150, Color.GhostWhite (248,248,255,
            // 255), scale 1.2, velocity ×0.5. NO sound. Type 58 hits
            // no tweak arm (the 59..=65 band starts above it).
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    58,
                    0.0,
                    0.0,
                    150,
                    Color::new(248, 248, 255, 255),
                    1.2,
                );
                env.dust.slot_mut(idx).velocity *= 0.5;
            }
        }
        3 => {
            // `:13284-13317` — the King-Slime-style teleport: 50×mult
            // type-180 dust, noGravity, then the FIVE `Next(3)==0`
            // ×0.75 rolls (each draws), two `Next(3)==0` ×2/×1.2
            // bumps, one `Next(2)==0` gate whose body draws
            // Next(75,100) fadeIn then Next(25,75) scale, and the
            // unconditional ×0.8 scale tail. Item6 first.
            (env.play_sound)(
                2,
                6,
                (rect.0 + rect.2 / 2) as f32,
                (rect.1 + rect.3 / 2) as f32,
                env.rng,
            );
            let count = (50.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    180,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                env.dust.slot_mut(idx).no_gravity = true;
                for _ in 0..5 {
                    if DustRng::next_max(env.rng, 3) == 0 {
                        env.dust.slot_mut(idx).velocity *= 0.75;
                    }
                }
                for _ in 0..2 {
                    if DustRng::next_max(env.rng, 3) == 0 {
                        let d = env.dust.slot_mut(idx);
                        d.velocity *= 2.0;
                        d.scale *= 1.2;
                    }
                }
                if DustRng::next_max(env.rng, 2) == 0 {
                    let fade = DustRng::next_range(env.rng, 75, 100) as f32 * 0.01;
                    let scale = DustRng::next_range(env.rng, 25, 75) as f32 * 0.01;
                    let d = env.dust.slot_mut(idx);
                    d.fade_in = fade;
                    d.scale = scale;
                }
                env.dust.slot_mut(idx).scale *= 0.8;
            }
        }
        4 => {
            // `:13318-13333` — the portal pair: W*H/5×mult type-263
            // dust coloured by GetPortalColor(extraInfo), noLight +
            // noGravity + scale 1.2 + fadeIn 0.4, then the alpha-255
            // overwrite. Item8 first. Type 263 hits no tweak arm.
            (env.play_sound)(
                2,
                8,
                (rect.0 + rect.2 / 2) as f32,
                (rect.1 + rect.3 / 2) as f32,
                env.rng,
            );
            let count = ((rect.2 * rect.3 / 5) as f32 * dust_count_mult) as i32;
            let mut color = portal_color(extra_info);
            color.a = 255; // `:13330`
            for _ in 0..count {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    263,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.color = color;
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        5 | 7 => {
            // `:13334-13369` — the water (5: dustWater(), sound 19)
            // and lava (7: type 35, Item8) columns: 100×mult / 50×mult
            // NewDustDirect over the height+24 box, velocity.Y ×0
            // then −3.5, velocity.X ×1.5, scale 0.8, alpha 130,
            // noGravity, fadeIn 1.2. The sound fires AFTER the loop
            // (both arms — `:13349`/`:13367`), at the rect Centre
            // (float halves), style 5's volume argument 0.
            let (dust_type, count) = if style == 5 {
                (
                    tr_core::particles::dust_water(water_style),
                    (100.0f32 * dust_count_mult) as i32,
                )
            } else {
                (35, (50.0f32 * dust_count_mult) as i32)
            };
            for _ in 0..count {
                let idx = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3 + 24,
                    dust_type,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.velocity.y *= 0.0;
                d.velocity.y -= 3.5;
                d.velocity.x *= 1.5;
                d.scale = 0.8;
                d.alpha = 130;
                d.no_gravity = true;
                d.fade_in = 1.2;
            }
            // `Rectangle.Center` is INTEGER halves (X + Width/2).
            let cx = (rect.0 + rect.2 / 2) as f32;
            let cy = (rect.1 + rect.3 / 2) as f32;
            if style == 5 {
                (env.play_sound)(19, 0, cx, cy, env.rng); // PlaySound(19, .., 0)
            } else {
                (env.play_sound)(2, 8, cx, cy, env.rng);
            }
        }
        9 => {
            // `:13370-13380` — the pylon hop: Inflate(15,15) FIRST,
            // then 100×mult SpawnInWorldDust(extraInfo, rect) calls;
            // Item6 after, at the INFLATED integer centre.
            rect = (rect.0 - 15, rect.1 - 15, rect.2 + 30, rect.3 + 30);
            let count = (100.0f32 * dust_count_mult) as i32;
            for _ in 0..count {
                pylon_spawn_in_world_dust(env, extra_info, rect);
            }
            (env.play_sound)(2, 6, (rect.0 + rect.2 / 2) as f32, (rect.1 + rect.3 / 2) as f32, env.rng);
        }
        10 => {
            // `:13381-13418` — the Rod-of-Harmony shimmer: Inflate,
            // 60×mult, per dust the fadeIn/scale NextFloat pair FIRST
            // (`:13388-13389`), then the hsl colour NextFloat inside
            // the NewDustDirect argument (`:13390`, drawn before the
            // allocator), the Next(20,70) scale roll + the n<10/n<5
            // bumps (OVERWRITTEN below — the rolls still run), the
            // n < count×0.8 velocity lerp toward otherPosition with
            // its NextFloat, the noGravity/noLight flags, the scale/
            // fadeIn ASSIGNMENTS, and the dustIndex != 6000 clone
            // (CloneDust's own allocator draws + the ×0.65 pair + the
            // white colour). Item8 after.
            rect = (rect.0 - 15, rect.1 - 15, rect.2 + 30, rect.3 + 30);
            let count = (60.0f32 * dust_count_mult) as i32;
            let vector = other_position - Vec2::new(rect.0 as f32, rect.1 as f32);
            for n in 0..count {
                let fade_in = 0.4 + DustRng::next_float(env.rng);
                let scale = 0.4 + DustRng::next_float(env.rng);
                let hue = 0.66 + DustRng::next_float(env.rng) * 0.24;
                let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.5, 255);
                let idx = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(rect.0 as f32, rect.1 as f32),
                    rect.2,
                    rect.3,
                    267,
                    0.0,
                    0.0,
                    127,
                    Color::new(rgb[0], rgb[1], rgb[2], rgb[3]),
                    1.0,
                );
                let mut rolled = DustRng::next_range(env.rng, 20, 70) as f32 * 0.01;
                if n < 10 {
                    rolled += 0.25;
                }
                if n < 5 {
                    rolled += 0.25;
                }
                env.dust.slot_mut(idx).scale = rolled;
                if (n as f32) < (count as f32) * 0.8 {
                    let jitter = DustRng::next_float(env.rng);
                    env.dust.slot_mut(idx).velocity += vector * 0.1 * jitter;
                }
                {
                    let d = env.dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.no_light = true;
                    d.scale = scale;
                    d.fade_in = fade_in;
                }
                // `:13408-13414` — the clone guard reads the SPAWNED
                // slot's dustIndex (the sentinel refuses).
                if env.dust.slot(idx).dust_index != tr_core::particles::DUST_SENTINEL {
                    let clone = env.dust.clone_dust(env.rng, env.dust_ctx, env.world, idx);
                    let d = env.dust.slot_mut(clone);
                    d.scale *= 0.65;
                    d.fade_in *= 0.65;
                    d.color = Color::WHITE;
                }
            }
            (env.play_sound)(2, 8, (rect.0 + rect.2 / 2) as f32, (rect.1 + rect.3 / 2) as f32, env.rng);
        }
        11 => {
            // `:13419-13446` — the mystic-frog / shellphone hop: 50
            // dust (NO mult), per dust the `Next(4)` colour pick
            // FIRST, then the position's TWO NextFloats
            // (`Utils.NextVector2FromRectangle`, `Utils.cs:1256-1259`)
            // inside the NewDustPerfect argument, the allocator, then
            // noGravity / colour / velocity ×2 / scale
            // 0.8+NextFloat×0.6 / fadeIn 0.5. NO sound.
            for _ in 0..50 {
                let num4 = DustRng::next_max(env.rng, 4);
                let color = match num4 {
                    0 | 1 => Color::new(100, 255, 100, 255),
                    2 => Color::new(255, 255, 0, 255), // Color.Yellow
                    _ => Color::WHITE,                 // 3: Color.White
                };
                let px = rect.0 as f32 + DustRng::next_float(env.rng) * rect.2 as f32;
                let py = rect.1 as f32 + DustRng::next_float(env.rng) * rect.3 as f32;
                let idx = env.dust.new_dust_perfect(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(px, py),
                    267,
                    None,
                    0,
                    Color::default(),
                    1.0,
                );
                let scale = 0.8 + DustRng::next_float(env.rng) * 0.6;
                let d = env.dust.slot_mut(idx);
                d.no_gravity = true;
                d.color = color;
                d.velocity *= 2.0;
                d.scale = scale;
                d.fade_in = 0.5;
            }
        }
        13 => {
            // `:13447-13462` — the frog's style-13 sparkle: Item8
            // first (integer Centre), then 21 dust (NO mult): the
            // position's two NextFloats, the `Next(1,3)` SCALE
            // argument (drawn before the allocator), a 2×2 type-27
            // box (alpha 100), velocity ×1.75, the i%3==0 ×1.5 bump,
            // noLightEmittance + noGravity.
            (env.play_sound)(2, 8, (rect.0 + rect.2 / 2) as f32, (rect.1 + rect.3 / 2) as f32, env.rng);
            for i in 0..21 {
                let px = rect.0 as f32 + DustRng::next_float(env.rng) * rect.2 as f32;
                let py = rect.1 as f32 + DustRng::next_float(env.rng) * rect.3 as f32;
                let scale_arg = DustRng::next_range(env.rng, 1, 3) as f32;
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(px, py),
                    2,
                    2,
                    27,
                    0.0,
                    0.0,
                    100,
                    Color::default(),
                    scale_arg,
                );
                let d = env.dust.slot_mut(idx);
                d.velocity *= 1.75;
                if i % 3 == 0 {
                    d.velocity *= 1.5;
                }
                d.no_light_emittance = true;
                d.no_gravity = true;
            }
        }
        _ => {
            // Styles 6/8 have no C# arm — the switch falls through
            // with zero draws.
        }
    }
}

/// `PortalHelper.GetPortalColor(int colorIndex)`
/// (`PortalHelper.cs:373-392`), the `netMode == 0` single-player arm:
/// `colorIndex / 2` is the player (unread here), `colorIndex % 2` the
/// portal — 0 → hsl(0.12, 1, 0.5), else hsl(0.52, 1, 0.6); alpha 66.
///
/// DEPRECATED (FOLD-SOURCE — g11 research/116 §4): g11's
/// `teleport_dust.rs::portal_color` is the surviving copy (same
/// canonical `ui::color::hsl_to_rgb`); the merger drops this one with
/// the teleport family.
pub fn portal_color(color_index: i32) -> Color {
    let rgb = if color_index % 2 != 0 {
        crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 66)
    } else {
        crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 66)
    };
    Color::new(rgb[0], rgb[1], rgb[2], 66)
}

/// `TeleportPylonsSystem.SpawnInWorldDust`
/// (`TeleportPylonsSystem.cs:327-393`): the per-pylon-type colour
/// (the `TeleportPylonType` byte enum, `TeleportPylonType.cs:3-17`),
/// one type-43 dust (alpha 254, scale 0.5), velocity ×0.1 then
/// Y −0.2. The float-ctor colour `new Color(r,g,b,1f)` multiplies by
/// 255 and truncates (`(byte)(v*255)`); the Snow/Mushroom 1.2 blue
/// clamps at 255 (XNA's float ctor clamps to [0,1] — the port clamps
/// the same, the pylons render precedent).
///
/// DEPRECATED (FOLD-SOURCE — g11 research/116 §4): the surviving copy
/// is g11's `teleport_dust.rs::pylon_spawn_in_world_dust` (the style-9
/// body); the merger drops this one with the teleport family.
fn pylon_spawn_in_world_dust<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    tile_style: i32,
    rect: (i32, i32, i32, i32),
) {
    // `:329-389` — the colour table, default (1,1,1).
    let (r, g, b) = match tile_style {
        0 | 9 | 10 => (0.05, 0.8, 0.3),  // SurfacePurity/Underworld/Shimmer
        1 => (0.7, 0.8, 0.05),          // Jungle
        2 => (0.5, 0.3, 0.7),           // Hallow
        3 => (0.4, 0.4, 0.6),           // Underground
        4 => (0.2, 0.2, 0.95),          // Beach
        5 => (0.85, 0.45, 0.1),         // Desert
        6 => (1.0, 1.0, 1.2),           // Snow
        7 => (0.4, 0.7, 1.2),           // GlowingMushroom
        8 => (0.7, 0.7, 0.7),           // Victory
        _ => (1.0, 1.0, 1.0),
    };
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
    // `:390` — NewDust(TopLeft, W, H, 43, 0, 0, 254, color, 0.5).
    let idx = env.dust.new_dust(
        env.rng,
        env.dust_ctx,
        env.world,
        Vec2::new(rect.0 as f32, rect.1 as f32),
        rect.2,
        rect.3,
        43,
        0.0,
        0.0,
        254,
        Color::new(byte(r), byte(g), byte(b), 255),
        0.5,
    );
    // `:391-392`.
    let d = env.dust.slot_mut(idx);
    d.velocity *= 0.1;
    d.velocity.y -= 0.2;
}

/// `NPC.Teleport` (`NPC.cs:81035-81058`) — the two-sided burst driver:
/// the PRE-move `teleportTime > 0` 0.3 dustCountMult, the Entry call
/// at the OLD rect (extraInfo2 = lastPortalColorIndex for style 4,
/// else 0), the position move, the style-4 lastPortalColorIndex latch,
/// the Exit call at the NEW rect (otherPosition = the old position),
/// then teleportTime = 1 / teleportStyle = Style. The `netMode == 2`
/// `SendData(65)` relay (`:81054-81057`) is the server arm — dropped.
/// The teleport fields ride the caller's `(teleport_time,
/// teleport_style, last_portal_color_index)` triple — the wiring
/// copies the `Npc` fields in, calls this, and writes them back (the
/// `&mut Npc` + `&mut`-field disjoint-borrow convention); the pure fn
/// stays field-agnostic so the witnesses drive the triple directly.
///
/// DEPRECATED (RETIRED at the merge — g11 research/116 §3/§4): g11's
/// App-level `main_dust::npc_teleport` replay supersedes this env-level
/// driver (it carries `other_position` on both legs per gap 2 and reads
/// the g11 `Npc` fields — the triple is retired). The g61b
/// `npc_teleport_wiring` helper (the statue-14/pad-0 immediate seam) is
/// the §5 DUP RISK: the merger keeps the g11 QUEUE replay (the pad
/// drain + the §5 Fold B statue queue) and drops the helper with this
/// driver. Kept compiling until the merge (the seam witnesses pin it).
#[allow(clippy::too_many_arguments)]
pub fn npc_teleport<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    npc: &mut Npc,
    new_pos: Vec2,
    style: i32,
    extra_info: i32,
    teleport: &mut (f32, i32, i32), // (teleportTime, teleportStyle, lastPortalColorIndex)
    water_style: i32,
) {
    // `:81037-81041` — extraInfo2 starts 0; style 4 pre-reads the OLD
    // portal colour index.
    let mut extra_info2 = 0;
    if style == 4 {
        extra_info2 = teleport.2;
    }
    // `:81042` — the mult reads the PRE-move teleportTime.
    let dust_count_mult = if teleport.0 > 0.0 { 0.3 } else { 1.0 };
    // `:81043-81044` — the Entry burst at the OLD rect.
    let other_position = Vec2::new(npc.x, npc.y);
    let (w, h) = npc.size();
    let old_rect = (npc.x as i32, npc.y as i32, w as i32, h as i32);
    teleport_effect(env, old_rect, style, extra_info2, dust_count_mult, new_pos, water_style);
    // `:81045` — the move.
    npc.x = new_pos.x;
    npc.y = new_pos.y;
    // `:81046-81050` — the style-4 latch BEFORE the Exit call.
    if style == 4 {
        teleport.2 = extra_info;
        extra_info2 = teleport.2;
    }
    // `:81051` — the Exit burst at the NEW rect, otherPosition = old.
    let new_rect = (npc.x as i32, npc.y as i32, w as i32, h as i32);
    teleport_effect(env, new_rect, style, extra_info2, dust_count_mult, other_position, water_style);
    // `:81052-81053`.
    teleport.0 = 1.0;
    teleport.1 = style;
}

/// `NPC.UpdateNPC_TeleportVisuals` (`NPC.cs:92194-92241`) — the
/// per-tick teleport fade, called from `UpdateNPC` (`:90322`) after
/// `UpdateAltTexture` (`:90313`) + `UpdateNPC_CastLights` (`:90320`)
/// and BEFORE `AI()` (`:90423`), on the `UpdateNPCs` stream. The
/// teleport fields ride the caller's `(teleportTime, teleportStyle,
/// lastPortalColorIndex)` triple (see [`npc_teleport`]);
/// `time_for_visual_effects` is `Main.timeForVisualEffects`
/// (`Main.cs:1181`, a double).
///
/// * Style 0 (`:92200-92209`): the `Next(100)` gate against
///   `100×time×2`, then ONE type-159 dust at the getRect box with
///   scale = time×1.5, noGravity, velocity ×1.1. NO time decrement
///   inside the arm — only the shared −0.005 tail.
/// * Style 4 (`:92210-92222`): time −0.02 FIRST, the `Next(100)` gate
///   against `100×time` (the DECREMENTED value), then a type-263 dust
///   coloured `GetPortalColor(lastPortalColorIndex)` — alpha stays 66
///   (no overwrite here, unlike the TeleportEffect burst) — noLight,
///   noGravity, scale 1.2, fadeIn 0.4.
/// * Style 12 (`:92223-92235`): the same shape, colour
///   `hslToRgb((float)(timeForVisualEffects/60.0) % 1f, 1, 0.75, 0)`
///   — alpha 0.
/// * Style 13 (`:92236-92239`): time = 0, no dust.
/// * The tail (`:92240`): `time = max(0, time − 0.005)` runs for EVERY
///   style that entered (the `time > 0` head gate `:92196-92199`
///   returns before it otherwise).
///
/// DEPRECATED (FOLD-SOURCE — g11 research/116 §4/§5, executed at g11
/// `56afc0c0` + `cc8c767b`): the surviving copy is g11's
/// `teleport_dust.rs::update_npc_teleport_visuals`, re-signed to
/// `&mut Npc` on the g11 fields (`teleport_style_id` naming — this
/// triple shape is retired) and WIRED PRE-AI in the `UpdateNPC` slot
/// preamble (the §5 Fold A of this branch's seam-4 order). The merger
/// drops THIS fn + the `world_sim/npcs.rs` copy-in/out drain and keeps
/// the g11 wiring. Kept compiling until the merge.
pub fn update_npc_teleport_visuals<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    npc: &Npc,
    teleport: &mut (f32, i32, i32),
    time_for_visual_effects: f64,
) {
    // `:92196-92199` — the head gate: no draws, no decrement.
    if !(teleport.0 > 0.0) {
        return;
    }
    let pos = Vec2::new(npc.x, npc.y);
    let (wf, hf) = npc.size();
    let (w, h) = (wf as i32, hf as i32);
    match teleport.1 {
        0 => {
            // `:92202` — the gate compares the int roll as float.
            if (DustRng::next_max(env.rng, 100) as f32) <= 100.0 * teleport.0 * 2.0 {
                // `:92204` — the box origin is getRect().X/Y: the
                // INTEGER-cast position (`NPC.cs:90027`).
                let rect_pos = Vec2::new((npc.x as i32) as f32, (npc.y as i32) as f32);
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    rect_pos,
                    w,
                    h,
                    159,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.scale = teleport.0 * 1.5;
                d.no_gravity = true;
                d.velocity *= 1.1;
            }
        }
        4 => {
            // `:92212-92213` — decrement BEFORE the gate.
            teleport.0 -= 0.02;
            if (DustRng::next_max(env.rng, 100) as f32) <= 100.0 * teleport.0 {
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    pos,
                    w,
                    h,
                    263,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.color = portal_color(teleport.2);
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        12 => {
            // `:92225-92226`.
            teleport.0 -= 0.02;
            if (DustRng::next_max(env.rng, 100) as f32) <= 100.0 * teleport.0 {
                let hue = ((time_for_visual_effects / 60.0) as f32) % 1.0;
                let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.75, 0);
                let idx = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    pos,
                    w,
                    h,
                    263,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(idx);
                d.color = Color::new(rgb[0], rgb[1], rgb[2], rgb[3]);
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        13 => {
            // `:92238` — the frog's instant clear.
            teleport.0 = 0.0;
        }
        _ => {}
    }
    // `:92240` — the shared decay tail (Math.Max(0f, ..)).
    teleport.0 = (teleport.0 - 0.005).max(0.0);
}

/// `NPC.MakeTransitionEffectsForTextureChanges`
/// (`NPC.cs:90236-90263`) — the town-NPC alt-texture transition puffs,
/// called by `UpdateAltTexture` (`:90231`) when the party hat /
/// zoologist form flips (the `!ForcePartyHatOn` gate lives in the
/// caller). Three arms, all separate `if`s:
/// * 0→1 (`:90238-90244`): 20× dust of type `139 + Next(4)` — the type
///   draw BEFORE the allocator — SpeedY −2, the height/4 INTEGER box.
/// * 1→0 (`:90245-90248`): the [`poof_of_smoke`] at the position.
/// * type 633 only, 0↔2 (`:90249-90262`): 20× NewDustDirect type 262,
///   SpeedY −2, noGravity + scale 2 post-writes.
pub fn make_transition_effects_for_texture_changes<R: DustRng + GoreRng>(
    env: &mut SpawnVisualsEnv<'_, R>,
    npc: &Npc,
    old_alt_texture: i32,
    current_alt_texture: i32,
) {
    let pos = Vec2::new(npc.x, npc.y);
    let (wf, hf) = npc.size();
    let (w, h) = (wf as i32, hf as i32);
    if old_alt_texture == 0 && current_alt_texture == 1 {
        // `:90240-90243` — `(float)(base.height / 4)` is INTEGER
        // division; the type argument draws per dust.
        for _ in 0..20 {
            let dust_type = 139 + DustRng::next_max(env.rng, 4);
            env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                pos,
                w,
                h / 4,
                dust_type,
                0.0,
                -2.0,
                0,
                Color::default(),
                1.0,
            );
        }
    }
    if old_alt_texture == 1 && current_alt_texture == 0 {
        // `:90247`.
        poof_of_smoke(env, pos);
    }
    // `:90249-90252` — the zoologist gate.
    if npc.kind.cs_id() != 633 {
        return;
    }
    let num = 2;
    if (old_alt_texture == 0 && current_alt_texture == num)
        || (old_alt_texture == num && current_alt_texture == 0)
    {
        // `:90256-90261` — NewDustDirect (the NaN zeroing runs even
        // though the rolled velocity is never NaN here).
        for _ in 0..20 {
            let idx = env.dust.new_dust_direct(
                env.rng,
                env.dust_ctx,
                env.world,
                pos,
                w,
                h,
                262,
                0.0,
                -2.0,
                0,
                Color::default(),
                1.0,
            );
            let d = env.dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 2.0;
        }
    }
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
            play_sound: &mut |a, b, x, y, _| sounds.push((a, b, x, y)),
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
            play_sound: &mut |_, _, _, _, _| {},
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
            play_sound: &mut |_, _, _, _, _| {},
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
            play_sound: &mut |_, _, _, _, _| {},
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
            play_sound: &mut |_, _, _, _, _| {},
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
                play_sound: &mut |_, _, _, _, _| {},
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
            play_sound: &mut |_, _, _, _, _| {},
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
            play_sound: &mut |a, b, x, y, _| sounds.push((a, b, x, y)),
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

    /// `Main.cs:13240-13252` — styles 0/1: the INTEGER W*H/5 count,
    /// per dust the block then the Next(20,70) scale roll; the j<10 /
    /// j<5 bumps stack (+0.5 for the first five).
    #[test]
    fn w4_g61_teleport_style0_count_scale_bumps_and_slots() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(29);
        let mut sounds = Vec::new();
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y, _| sounds.push((a, b, x, y)),
        };
        // 20×40 → 800/5 = 160 dust; mult 1.
        teleport_effect(&mut env, (100, 100, 20, 40), 0, 0, 1.0, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..160 {
            expected.extend(dust_block(20, 40));
            expected.push(Draw::Range(20, 70));
        }
        assert_eq!(rng.log, expected);
        assert_eq!(sounds, vec![(2, 6, 110.0, 120.0)]);
        let mut replay = UnifiedRandom::new(29);
        for j in 0..160 {
            // consume the block draws (shapes pinned by the log above)
            let _ = replay.next_max(16);
            let _ = replay.next_max(36);
            let _ = replay.next_range(-20, 21);
            let _ = replay.next_range(-20, 21);
            let _ = replay.next_max(3);
            let _ = replay.next_range(-20, 21);
            let mut want = replay.next_range(20, 70) as f32 * 0.01;
            if j < 10 {
                want += 0.25;
            }
            if j < 5 {
                want += 0.25;
            }
            let d = dust.slot(j);
            assert_eq!(d.dust_type, 159);
            assert_eq!(d.scale, want);
        }
        // The 0.3 re-teleport mult truncates: 160×0.3 = 48.
        assert_eq!((160.0f32 * 0.3) as i32, 48);
    }

    /// `Main.cs:13284-13317` — style 3: per dust the block + five
    /// Next(3) + two Next(3) + one Next(2), and the Next(2) body's
    /// Next(75,100)/Next(25,75) pair only when the gate opens; the
    /// unconditional ×0.8 scale tail.
    #[test]
    fn w4_g61_teleport_style3_gate_chain_and_scale_tail() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(31);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _, _| {},
        };
        // mult 0.1 → 5 dust (keeps the witness log readable).
        teleport_effect(&mut env, (100, 100, 20, 40), 3, 0, 0.1, Vec2::ZERO, 0);
        let mut replay = UnifiedRandom::new(31);
        let mut expected = Vec::new();
        let mut gates_opened = 0;
        for i in 0..5 {
            expected.extend(dust_block(20, 40));
            // block replay
            let _ = replay.next_max(16);
            let _ = replay.next_max(36);
            let mut vx = replay.next_range(-20, 21) as f32 * 0.1;
            let mut vy = replay.next_range(-20, 21) as f32 * 0.1;
            let _ = replay.next_max(3);
            let mut scale = 1.0 + replay.next_range(-20, 21) as f32 * 0.01;
            for _ in 0..5 {
                expected.push(Draw::Max(3));
                if replay.next_max(3) == 0 {
                    vx *= 0.75;
                    vy *= 0.75;
                }
            }
            for _ in 0..2 {
                expected.push(Draw::Max(3));
                if replay.next_max(3) == 0 {
                    vx *= 2.0;
                    vy *= 2.0;
                    scale *= 1.2;
                }
            }
            expected.push(Draw::Max(2));
            let mut fade = 0.0f32;
            if replay.next_max(2) == 0 {
                gates_opened += 1;
                expected.push(Draw::Range(75, 100));
                expected.push(Draw::Range(25, 75));
                fade = replay.next_range(75, 100) as f32 * 0.01;
                scale = replay.next_range(25, 75) as f32 * 0.01;
            }
            scale *= 0.8;
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 180);
            assert!(d.no_gravity);
            assert_eq!(d.velocity.x, vx);
            assert_eq!(d.velocity.y, vy);
            assert_eq!(d.scale, scale);
            assert_eq!(d.fade_in, fade);
        }
        assert_eq!(rng.log, expected);
        let _ = gates_opened;
    }

    /// `Main.cs:13318-13333` + `PortalHelper.cs:378-392` — style 4:
    /// the portal colour by index parity, alpha 66 → 255 overwrite,
    /// the fixed scale/fadeIn, no allocator-side draws beyond the
    /// block.
    #[test]
    fn w4_g61_teleport_style4_portal_colour_and_alpha() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(37);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _, _| {},
        };
        // 10×10 → 20 dust; extraInfo 3 → portal 1 (the 0.52 hue).
        teleport_effect(&mut env, (100, 100, 10, 10), 4, 3, 1.0, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..20 {
            expected.extend(dust_block(10, 10));
        }
        assert_eq!(rng.log, expected, "the colour is computed, not rolled");
        let want = crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 66);
        for i in 0..20 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 263);
            assert_eq!((d.color.r, d.color.g, d.color.b), (want[0], want[1], want[2]));
            assert_eq!(d.color.a, 255, ":13330 overwrites the 66");
            assert!(d.no_light && d.no_gravity);
            assert_eq!(d.scale, 1.2);
            assert_eq!(d.fade_in, 0.4);
        }
        // Parity: index 2 → portal 0 (the 0.12 hue).
        let p0 = portal_color(2);
        let w0 = crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 66);
        assert_eq!((p0.r, p0.g, p0.b, p0.a), (w0[0], w0[1], w0[2], 66));
    }

    /// `Main.cs:13370-13380` + `TeleportPylonsSystem.cs:327-393` —
    /// style 9 inflates FIRST (the dust box AND the sound centre read
    /// the inflated rect), 100×mult type-43 dust with the per-type
    /// colour and the ×0.1 / Y−0.2 velocity tail.
    #[test]
    fn w4_g61_teleport_style9_inflate_and_pylon_colour() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(41);
        let mut sounds = Vec::new();
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y, _| sounds.push((a, b, x, y)),
        };
        // mult 0.05 → 5 dust; style 6 (Snow) — the 1.2 blue clamps.
        teleport_effect(&mut env, (100, 100, 20, 40), 9, 6, 0.05, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..5 {
            expected.extend(dust_block(50, 70)); // the inflated box
        }
        assert_eq!(rng.log, expected);
        // inflated centre: (100−15 + 50/2, 100−15 + 70/2) = (110, 120).
        assert_eq!(sounds, vec![(2, 6, 110.0, 120.0)]);
        for i in 0..5 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 43);
            assert_eq!(d.alpha, 254);
            assert_eq!((d.color.r, d.color.g, d.color.b, d.color.a), (255, 255, 255, 255));
            assert!(d.position.x >= 85.0 + 4.0 && d.position.x <= 85.0 + 49.0);
        }
    }

    /// `Main.cs:13381-13418` — style 10: the fadeIn/scale/hue
    /// NextFloat triple BEFORE the allocator, the overwritten scale
    /// roll still drawn, the n<80% lerp gate, and the clone's own
    /// 0×0 block + ×0.65 pair + white colour.
    #[test]
    fn w4_g61_teleport_style10_float_triple_clone_and_overwrites() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(43);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _, _| {},
        };
        // mult 0.05 → 3 dust (all n < 2.4 → the lerp gate opens).
        teleport_effect(&mut env, (100, 100, 20, 40), 10, 0, 0.05, Vec2::new(500.0, 300.0), 0);
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.push(Draw::Float); // fadeIn
            expected.push(Draw::Float); // scale
            expected.push(Draw::Float); // hue
            expected.extend(dust_block(50, 70));
            expected.push(Draw::Range(20, 70)); // the overwritten roll
            expected.push(Draw::Float); // the lerp jitter
            expected.extend(dust_block(0, 0)); // CloneDust's spawn
        }
        assert_eq!(rng.log, expected);
        // slot pairs: (primary, clone) — the clone is white, ×0.65.
        for pair in 0..3 {
            let p = dust.slot(pair * 2);
            let cl = dust.slot(pair * 2 + 1);
            assert_eq!(p.dust_type, 267);
            assert_eq!(p.alpha, 127);
            assert!(p.no_gravity && p.no_light);
            assert!(p.scale >= 0.4 && p.scale < 1.4);
            assert!(p.fade_in >= 0.4 && p.fade_in < 1.4);
            assert_eq!(cl.color, Color::WHITE);
            assert_eq!(cl.scale, p.scale * 0.65);
            assert_eq!(cl.fade_in, p.fade_in * 0.65);
            assert_eq!(cl.position, p.position);
        }
    }

    /// `NPC.cs:81035-81058` — the driver: Entry at the old rect, Exit
    /// at the new, the pre-move mult, the style-4 index latch between
    /// the calls, teleportTime/Style written last.
    #[test]
    fn w4_g61_npc_teleport_two_sided_burst_and_latch() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(47);
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((10.0, 10.0)); // 100/5 = 20 per side
        let mut tp = (0.5f32, 0i32, 2i32); // mid-teleport: mult 0.3, old index 2
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _, _| {},
        };
        npc_teleport(&mut env, &mut npc, Vec2::new(300.0, 200.0), 4, 5, &mut tp, 0);
        // 20×0.3 = 6 per side → 12 dust, no extra draws.
        let mut expected = Vec::new();
        for _ in 0..12 {
            expected.extend(dust_block(10, 10));
        }
        assert_eq!(rng.log, expected);
        assert_eq!((npc.x, npc.y), (300.0, 200.0));
        assert_eq!(tp, (1.0, 4, 5), "time 1, style 4, the NEW index latched");
        // Entry dust: the OLD portal colour (index 2 → portal 0);
        // Exit dust: the NEW (index 5 → portal 1).
        let old_c = portal_color(2);
        let new_c = portal_color(5);
        for i in 0..6 {
            assert_eq!(dust.slot(i).color.r, old_c.r);
            assert!(dust.slot(i).position.x < 200.0);
        }
        for i in 6..12 {
            assert_eq!(dust.slot(i).color.r, new_c.r);
            assert!(dust.slot(i).position.x >= 300.0);
        }
    }

    /// `Main.cs:13419-13446` — style 11: per dust the Next(4) colour
    /// pick, the two position NextFloats, the 0×0 block, the scale
    /// NextFloat; the colour table and the ×2 velocity.
    #[test]
    fn w4_g61_teleport_style11_colour_pick_and_position_floats() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut rng = RecordingRng::new(53);
        let mut sounds = Vec::new();
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |a, b, x, y, _| sounds.push((a, b, x, y)),
        };
        teleport_effect(&mut env, (100, 100, 20, 40), 11, 0, 1.0, Vec2::ZERO, 0);
        let mut expected = Vec::new();
        for _ in 0..50 {
            expected.push(Draw::Max(4));
            expected.push(Draw::Float);
            expected.push(Draw::Float);
            expected.extend(dust_block(0, 0));
            expected.push(Draw::Float);
        }
        assert_eq!(rng.log, expected);
        assert!(sounds.is_empty(), "style 11 plays no sound");
        let mut replay = UnifiedRandom::new(53);
        for i in 0..50 {
            let pick = replay.next_max(4);
            let px = 100.0 + replay.next_float() * 20.0;
            let py = 100.0 + replay.next_float() * 40.0;
            let _ = replay.next_max(1);
            let _ = replay.next_max(1);
            let vx = replay.next_range(-20, 21) as f32 * 0.1;
            let vy = replay.next_range(-20, 21) as f32 * 0.1;
            let _ = replay.next_max(3);
            let _ = replay.next_range(-20, 21);
            let scale = 0.8 + replay.next_float() * 0.6;
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 267);
            assert_eq!(d.position, Vec2::new(px, py), "Perfect overwrites");
            let want = match pick {
                0 | 1 => Color::new(100, 255, 100, 255),
                2 => Color::new(255, 255, 0, 255),
                _ => Color::WHITE,
            };
            assert_eq!(d.color, want);
            assert_eq!(d.velocity, Vec2::new(vx * 2.0, vy * 2.0));
            assert_eq!(d.scale, scale);
            assert_eq!(d.fade_in, 0.5);
            assert!(d.no_gravity);
        }
    }

    /// `Main.cs:92200-92240` — the per-tick fade: style 0's ×2 gate
    /// with NO in-arm decrement, style 4's decrement-BEFORE-gate, the
    /// shared −0.005 tail, style 13's instant clear, and the time<=0
    /// head gate drawing nothing.
    #[test]
    fn w4_g61_teleport_visuals_gates_decrements_and_tail() {
        let w = air();
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((20.0, 20.0));
        // Style 0 at time 0.001: the gate bound is 0.2 — only a 0 roll
        // opens; either way NO in-arm decrement, then the −0.005 tail.
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(59);
            let mut tp = (0.001f32, 0i32, 0i32);
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &npc, &mut tp, 0.0);
            let gate = UnifiedRandom::new(59).next_max(100);
            let mut expected = vec![Draw::Max(100)];
            if (gate as f32) <= 0.2 {
                expected.extend(dust_block(20, 20));
            }
            assert_eq!(rng.log, expected);
            assert_eq!(tp.0, 0.0, "0.001 − 0.005 floors at 0");
        }
        // Style 4 at time 0.5: −0.02 first → gate bound 48; tail →
        // 0.475. The dust colour keeps alpha 66 (no overwrite).
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(61);
            let mut tp = (0.5f32, 4i32, 1i32);
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &npc, &mut tp, 0.0);
            let gate = UnifiedRandom::new(61).next_max(100);
            let mut expected = vec![Draw::Max(100)];
            if (gate as f32) <= 48.0 {
                expected.extend(dust_block(20, 20));
                let d = dust.slot(0);
                assert_eq!(d.dust_type, 263);
                assert_eq!(d.color.a, 66, "the fade keeps the portal alpha");
                assert_eq!(d.scale, 1.2);
                assert_eq!(d.fade_in, 0.4);
            }
            assert_eq!(rng.log, expected);
            assert_eq!(tp.0, 0.5 - 0.02 - 0.005);
        }
        // Style 13: instant clear, zero draws, the tail keeps 0.
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(67);
            let mut tp = (0.9f32, 13i32, 0i32);
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &npc, &mut tp, 0.0);
            assert!(rng.log.is_empty());
            assert_eq!(tp.0, 0.0);
        }
        // time <= 0: the head gate returns before any draw/decrement.
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(71);
            let mut tp = (0.0f32, 0i32, 0i32);
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            update_npc_teleport_visuals(&mut env, &npc, &mut tp, 0.0);
            assert!(rng.log.is_empty());
            assert_eq!(tp.0, 0.0);
        }
    }

    /// `Main.cs:92223-92235` — style 12's colour is the visual-effects
    /// clock hue at luminosity 0.75, alpha 0.
    #[test]
    fn w4_g61_teleport_visuals_style12_clock_hue() {
        let w = air();
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let c = ctx();
        let mut npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        npc.size_override = Some((20.0, 20.0));
        // Synthetic time 1.03 → decrement 1.01 → gate bound 101: the
        // Next(100) roll ALWAYS opens, so the colour check is
        // unconditional (at the in-game 1.0 a 99 roll would close it).
        let mut rng = RecordingRng::new(73);
        let mut tp = (1.03f32, 12i32, 0i32);
        let mut env = SpawnVisualsEnv {
            world: &w,
            dust: &mut dust,
            dust_ctx: &c,
            gore: &mut gore,
            rng: &mut rng,
            play_sound: &mut |_, _, _, _, _| {},
        };
        // clock 90 → 90/60 = 1.5 → %1 = 0.5.
        update_npc_teleport_visuals(&mut env, &npc, &mut tp, 90.0);
        let want = crate::ui::color::hsl_to_rgb(0.5, 1.0, 0.75, 0);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 263);
        assert_eq!((d.color.r, d.color.g, d.color.b, d.color.a), (want[0], want[1], want[2], 0));
        assert_eq!(tp.0, 1.03 - 0.02 - 0.005);
    }

    /// `NPC.cs:90238-90262` — the alt-texture transitions: the 0→1
    /// per-dust type draw + the height/4 INTEGER box; the 633-only
    /// 0↔2 direct arm; the 1→0 poof delegation.
    #[test]
    fn w4_g61_alt_texture_transitions_type_draws_and_zoologist_gate() {
        let w = air();
        // 0→1 on a non-633 town NPC: 20 × [Next(4), block(w, h/4)].
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(79);
            let mut npc = Npc::new(NpcKind::Zoologist, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((18.0, 41.0)); // 41/4 = 10
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            make_transition_effects_for_texture_changes(&mut env, &npc, 0, 1);
            let mut expected = Vec::new();
            let mut replay = UnifiedRandom::new(79);
            for i in 0..20 {
                expected.push(Draw::Max(4));
                expected.extend(dust_block(18, 10));
                let t = 139 + replay.next_max(4);
                for _ in 0..6 {
                    let _ = replay.next_max(1); // shapes pinned by the log
                }
                assert!((139..143).contains(&dust.slot(i).dust_type));
                assert_eq!(dust.slot(i).dust_type, t);
            }
            assert_eq!(rng.log, expected);
        }
        // 0↔2 on the Zoologist (633): 20 direct type-262 blocks, the
        // scale-2/noGravity post-writes; a non-633 kind draws nothing.
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(83);
            let mut npc = Npc::new(NpcKind::Zoologist, 100.0, 100.0, 1, 0.0);
            npc.size_override = Some((18.0, 40.0));
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            make_transition_effects_for_texture_changes(&mut env, &npc, 2, 0);
            let mut expected = Vec::new();
            for _ in 0..20 {
                expected.extend(dust_block(18, 40));
            }
            assert_eq!(rng.log, expected);
            for i in 0..20 {
                let d = dust.slot(i);
                assert_eq!(d.dust_type, 262);
                assert!(d.no_gravity);
                assert_eq!(d.scale, 2.0);
            }
            // The same transition on a non-633: zero draws.
            let mut rng2 = RecordingRng::new(83);
            let mut dust2 = DustPool::new();
            let mut gore2 = GorePool::new();
            let npc2 = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            let mut env2 = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust2,
                dust_ctx: &c,
                gore: &mut gore2,
                rng: &mut rng2,
                play_sound: &mut |_, _, _, _, _| {},
            };
            make_transition_effects_for_texture_changes(&mut env2, &npc2, 2, 0);
            assert!(rng2.log.is_empty());
        }
        // 1→0: the poof delegation (count + gores + ten dust).
        {
            let mut dust = DustPool::new();
            let mut gore = GorePool::new();
            let c = ctx();
            let mut rng = RecordingRng::new(89);
            let npc = Npc::new(NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
            let mut env = SpawnVisualsEnv {
                world: &w,
                dust: &mut dust,
                dust_ctx: &c,
                gore: &mut gore,
                rng: &mut rng,
                play_sound: &mut |_, _, _, _, _| {},
            };
            make_transition_effects_for_texture_changes(&mut env, &npc, 1, 0);
            assert_eq!(rng.log.first(), Some(&Draw::Range(3, 7)), "the poof count");
            assert_eq!(dust.slot(0).dust_type, 16);
        }
    }
}
