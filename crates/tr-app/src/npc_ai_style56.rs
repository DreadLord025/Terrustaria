//! The aiStyle-56 family arm — the Dungeon Spirit (`NPC.AI()`'s
//! `else if (this.aiStyle == 56)` block, `NPC.cs:32742-32760`).
//!
//! S1-D-3 (aiStyle-56 round, code-only mode): this module is the faithful
//! 1:1 port of the arm as a pure function over `Npc` + a small Env seam
//! (the d6bc30ed bat precedent).
//!
//! **WIRED by S1-D-3-wire-smallkinds (2026-11-19).** `NpcKind::DungeonSpirit`
//! (protocol id 288, vanilla type 288) now exists with its `SetDefaults`
//! row (`NPC.cs:12738-12752`) in `npc/data.rs`. The arm needs the shared
//! `Main.dust` pool + the sibling-free `TargetCandidate` slice, so it runs
//! in the world-sim tick closure (`world_sim/npcs.rs`) at the spirit's
//! slot turn — [`style56_step`] fills a [`Style56Env`] from the App's
//! `DustPool`/`SpawnCtx` and the `UpdateNPCs` stream. The style axis' 56
//! arm (`npc.rs`) owns the post-`AI()` noTileCollide/noGravity integration
//! tail (`NPC.cs:90494-90498`). See docs/ANALYSIS.md "The Dungeon Spirit
//! aiStyle-56 arm" + "The small-kinds wiring wave".
//!
//! Family identity (verified from the C#, not from the roadmap): the
//! ONLY `aiStyle = 56` assignment in `NPC.cs` is type 288
//! (`NPC.cs:12738-12752`, `NPCID.DungeonSpirit`, `ID/NPCID.cs:11253`):
//! 20×20, damage 70, defense 30, lifeMax 200, knockBackResist 0.2,
//! `SoundID.NPCHit36`/`NPCDeath39`, value 500, noTileCollide,
//! noGravity. (`Projectile.cs:4220` also assigns aiStyle 56 — that is
//! a PROJECTILE style, a different axis, not this family.) There are
//! no per-type branches inside the arm: the whole 19-line block below
//! IS the family. The type-288 sites outside the arm are cited skips:
//! the death dust burst (`NPC.cs:85000-85009`, 50× type-180 dust at
//! ×2 velocity / 1.4 scale — the checkDead family), the ambient sound
//! roll (`NPC.cs:90147-90150`, `Main.rand.Next(400) == 0` then
//! `Next(53, 55)` — UpdateNPC's sound chain, NOT this arm's stream)
//! and the draw colour (`NPC.cs:93605-93608`, `Color(200,200,200,0)`
//! — GetAlpha/render).
//!
//! RNG by construction: the arm body draws NO direct `Main.rand.Next`
//! — its only stream consumer is the one `Dust.NewDust(position,
//! width, height, 180)` call (`NPC.cs:32755`, `Dust.cs:75-256`, the
//! S1-E-1 `DustPool::new_dust` port). For the 20×20 box and an
//! accepted low-band slot the draw order is exactly: pressure-band
//! roll (only when the first free slot is above a `maxDustToDraw`
//! band — the short-circuit gate draws, `Dust.cs:114-148`), then
//! `Next(width-4)` = `Next(16)` (pos X, `:170`), `Next(height-4)` =
//! `Next(16)` (pos Y, `:171`), `Next(-20,21)` (vel X, `:172`),
//! `Next(-20,21)` (vel Y, `:173`), `Next(3)` (frame row, `:175`),
//! `Next(-20,21)` (scale, `:190`). Type 180 hits NONE of the per-type
//! tweak arms (`Dust.cs:214-252`) and is not `DangerousDust`
//! (`ChildSafety.cs:32`), so no further draws. The arm's `scale =
//! 1.3f` OVERWRITES the rolled scale but the roll is still consumed
//! (`:32757` runs after `NewDust` returned). The off-screen cull
//! (`Dust.cs:97-101`) and the menu/pause guards refuse with ZERO
//! draws; a refused spawn returns the 6000 sentinel and the C# still
//! writes velocity/scale/noGravity into `Main.dust[6000]` — the port
//! keeps that (the pool has the sentinel slot). `TargetClosest`
//! (`NPC.cs:77652-77781`) is RNG-free. The stream-peek witnesses
//! below replay the cited draws by hand (WRITTEN NOT RUN — code-only
//! mode).
//!
//! Faithfulness notes (the f1701eb0 defect catalogue applied):
//! * Check order: `TargetClosest()` is the FIRST statement
//!   (`:32744`), before the aim reads — the pick rides
//!   `npc_target::target_closest` unmodified (faceTarget = true, the
//!   C# default; ai_style 56, so no aiStyle-3 bottom-edge arm).
//! * Both centres use FLOAT halves — `Entity.Center` is
//!   `position + (float)width / 2f` (`Entity.cs:50-55`), NOT the
//!   integer `(float)(width / 2)` the facing code uses. For the 20×20
//!   spirit and the 20×42 player both halves are whole, but the port
//!   keeps the float division verbatim.
//! * The normaliser is `12f / (float)Math.Sqrt(dx*dx + dy*dy)`
//!   (`:32748`): the sum of squares is computed in f32, `Math.Sqrt`
//!   runs in f64 (the float argument promotes), the result casts back
//!   to f32 and the division is f32. NOT an f32 `.sqrt()`.
//! * The blend is `(velocity * 100f + aim) / 101f` (`:32751-32752`)
//!   — a 100:1 weighted average, all f32; not a `+= accel` approach
//!   and not a `/100` divide.
//! * The rotation is `(float)Math.Atan2(num833, num832) - 1.57f`
//!   (`:32753`) — atan2 in f64 over the SCALED components, cast to
//!   f32, minus the literal 1.57 (not π/2). It is written EVERY tick,
//!   unconditionally.
//! * Zero distance is NOT guarded: dx = dy = 0 makes num834 = +inf
//!   and the aim 0 × inf = NaN, so velocity and rotation go NaN —
//!   exactly the C#'s behaviour; the port invents no clamp.
//! * The dust position rides the netOffset round trip verbatim
//!   (`:32754`/`:32759`): `position += netOffset` BEFORE NewDust,
//!   `position -= netOffset` after — the add-then-subtract is kept
//!   (not folded into the spawn argument) so the float rounding of
//!   the restored position matches. Zero in single-player.
//! * The dust post-writes are `velocity *= 0.1f`, `scale = 1.3f`
//!   (assignment), `noGravity = true` (`:32756-32758`) — applied to
//!   the returned slot INCLUDING the 6000 sentinel, exactly like
//!   `Main.dust[num835]`.
//! * The arm writes NO ai[], NO resets, NO expert arms, NO shots, NO
//!   spriteDirection — there is nothing else in the block. It does
//!   NOT integrate `position += velocity`: the spirit is
//!   noTileCollide/noGravity (`:12750-12751`), so `UpdateNPC`
//!   integrates after `AI()`; the caller owns the step.
#![allow(dead_code)] // WIRED by S1-D-3-wire-smallkinds (the world-sim dust pre-pass + axis 56's integration tail); the witnesses stay as the audit cross-check

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The per-tick environment the arm reads through `Main`'s statics.
/// The caller (the future style-axis arm) fills it from the App's
/// session player slice, the shared dust pool (S1-E-1) and the pool's
/// `Main.rand` analogue stream — the same seam shape as the bat
/// round's `Style14Env` (d6bc30ed).
pub struct Style56Env<'a> {
    /// The read-only world the target scan's `CanHit` walk reads.
    pub world: &'a World,
    /// The `Main.player` slots as the target scan sees them (at least
    /// one slot — the C# always has the 255; the port's session player
    /// is slot 0).
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — the same stand-in as `update_zombie`.
    pub ticker: u64,
    /// `Main.dust` — the shared 6001-slot pool (`Dust.cs:75-256`).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// The pool's draw stream (`Main.rand`'s per-consumer analogue,
    /// the S1-E-1 convention).
    pub rng: &'a mut UnifiedRandom,
}

/// `NPC.AI()`'s aiStyle-56 arm (`NPC.cs:32742-32760`), one tick.
pub fn style56_step(npc: &mut Npc, env: &mut Style56Env<'_>) {
    // `:32744` — TargetClosest() FIRST (faceTarget = true default).
    target_closest(npc, env.world, env.players, true, 56, env.ticker);
    // `:32745-32747` — both centres through Entity.Center's FLOAT
    // halves (`Entity.cs:50-55`).
    let (cx, cy) = npc.center();
    let p = &env.players[npc.target as usize];
    let mut num832 = p.x + p.width as f32 / 2.0 - cx;
    let mut num833 = p.y + p.height as f32 / 2.0 - cy;
    // `:32748` — 12f / (float)Math.Sqrt(f32 sum of squares): the sqrt
    // runs in f64, the cast and the division are f32. Zero distance
    // makes this +inf and the aim NaN below — no invented guard.
    let num834 = 12.0f32 / ((num832 * num832 + num833 * num833) as f64).sqrt() as f32;
    num832 *= num834;
    num833 *= num834;
    // `:32751-32752` — the 100:1 weighted blend, all f32.
    npc.vx = (npc.vx * 100.0 + num832) / 101.0;
    npc.vy = (npc.vy * 100.0 + num833) / 101.0;
    // `:32753` — atan2 in f64 over the scaled components, cast to
    // f32, minus the literal 1.57f. Written every tick.
    npc.rotation = (num833 as f64).atan2(num832 as f64) as f32 - 1.57;
    // `:32754` — position += netOffset (the client smoothing offset;
    // zero in single-player). The round trip is kept verbatim.
    npc.x += npc.net_offset.x;
    npc.y += npc.net_offset.y;
    // `:32755` — Dust.NewDust(position, width, height, 180) with the
    // C# defaults SpeedX/SpeedY 0, Alpha 0, newColor default(Color)
    // (all-zero), Scale 1f. The pool rolls the caller's stream in the
    // cited order (module header).
    let (w, h) = npc.size();
    let num835 = env.dust.new_dust(
        env.rng,
        env.dust_ctx,
        env.world,
        Vec2::new(npc.x, npc.y),
        w as i32,
        h as i32,
        180,
        0.0,
        0.0,
        0,
        Color::default(),
        1.0,
    );
    // `:32756-32758` — the post-writes, sentinel slot included
    // (`Main.dust[6000]` exists in the C#; the pool keeps it).
    {
        let d = env.dust.slot_mut(num835);
        d.velocity *= 0.1;
        d.scale = 1.3; // assignment — overwrites the rolled scale
        d.no_gravity = true;
    }
    // `:32759` — position -= netOffset.
    npc.x -= npc.net_offset.x;
    npc.y -= npc.net_offset.y;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::particles::DUST_SENTINEL;

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A 20×20 spirit stand-in: the port has no `NpcKind::DungeonSpirit`
    /// yet (the wiring round adds it with the `NPC.cs:12738-12752`
    /// row); the box rides `size_override` so the arm reads the C#'s
    /// 20×20 exactly. Facing 1, the SetDefaults target 255.
    fn spirit(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Creeper, x, y, 1, 0.0);
        n.size_override = Some((20.0, 20.0));
        n
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// On-screen ctx: the (100,100) box sits inside the 400-px cull
    /// margin; maxDustToDraw 6000 puts slot 0 in the low band (no
    /// gate draw).
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

    /// `NPC.cs:32751-32752` — the blend is `(v*100 + aim)/101`, not a
    /// `+= accel` approach: from rest the first step is aim/101, and
    /// from a standing aim-speed chase the velocity HOLDS (the fixed
    /// point of the blend), where an approach would keep accelerating.
    #[test]
    fn s1_d_3_s56_the_blend_is_the_hundred_to_one_weighted_average() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(12345);
        // Spirit centre (110,110); player centre (410,510): delta
        // (300,400), length 500 → num834 = 12/500 = 0.024 → aim
        // (7.2, 9.6).
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 7,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        let aim_x = 300.0f32 * (12.0f32 / 500.0f32);
        let aim_y = 400.0f32 * (12.0f32 / 500.0f32);
        assert_eq!(n.vx, (0.0 * 100.0 + aim_x) / 101.0, "from rest: aim/101");
        assert_eq!(n.vy, (0.0 * 100.0 + aim_y) / 101.0);
        // The fixed point: a velocity already at the aim holds
        // exactly in real arithmetic ((12*100+12)/101 = 12); the f32
        // round trip can wander a few ulps, so the witness is a
        // tight relative band, not bit equality.
        let mut n2 = spirit(100.0, 100.0);
        n2.vx = aim_x;
        n2.vy = aim_y;
        let mut env2 = Style56Env {
            world: &w,
            players: &players,
            ticker: 7,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n2, &mut env2);
        assert!((n2.vx - aim_x).abs() < 1e-5);
        assert!((n2.vy - aim_y).abs() < 1e-5);
    }

    /// `NPC.cs:32748` — the normaliser's sqrt runs in f64 over the
    /// f32 sum of squares; the witness pins the 3-4-5 geometry
    /// against an f64 reference (an f32-sqrt port can differ by an
    /// ulp on non-exact lengths).
    #[test]
    fn s1_d_3_s56_the_normaliser_sqrt_is_f64_over_the_f32_sum() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(1);
        // Delta (300, 400): the f32 sum 90000+160000 = 250000 is
        // exact, sqrt 500 exact — the aim is exactly 12 * (0.6, 0.8).
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        let dx = 300.0f32;
        let dy = 400.0f32;
        let scale = 12.0f32 / ((dx * dx + dy * dy) as f64).sqrt() as f32;
        assert_eq!(n.vx, (dx * scale) / 101.0);
        assert_eq!(n.vy, (dy * scale) / 101.0);
        // A non-exact length: delta (100, 100), sum 20000, sqrt in
        // f64 then cast — pin against the same ladder, f64 reference.
        let players2 = [cand(200.0, 189.0)];
        let mut n2 = spirit(100.0, 100.0);
        let mut env2 = Style56Env {
            world: &w,
            players: &players2,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n2, &mut env2);
        let s2 = 12.0f64 / (20000.0f64).sqrt();
        let expect = (100.0 * s2 / 101.0) as f32;
        // The f32 cast of the f64 sqrt can move the aim by ~1 ulp
        // (~6e-8 relative), so the band scales with the value.
        assert!((n2.vx - expect).abs() < expect.abs() * 1e-6, "f64-sqrt ladder reference");
    }

    /// `NPC.cs:32753` — rotation = atan2(aimY, aimX) − the literal
    /// 1.57f, every tick: right → −1.57, down → π/2 − 1.57, left →
    /// π − 1.57.
    #[test]
    fn s1_d_3_s56_the_rotation_is_the_aim_angle_minus_the_literal() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(2);
        let run = |px: f32, py: f32, pool: &mut DustPool, rng: &mut UnifiedRandom| {
            let players = [cand(px, py)];
            let mut n = spirit(100.0, 100.0);
            let mut env = Style56Env {
                world: &w,
                players: &players,
                ticker: 0,
                dust: pool,
                dust_ctx: &c,
                rng,
            };
            style56_step(&mut n, &mut env);
            n.rotation
        };
        // Player centre right of the spirit centre (110,110).
        let r = run(400.0, 89.0, &mut pool, &mut rng);
        assert!((r - (-1.57)).abs() < 1e-6, "atan2(0,+) = 0");
        // Directly below.
        let r = run(100.0, 489.0, &mut pool, &mut rng);
        assert!((r - (std::f64::consts::FRAC_PI_2 as f32 - 1.57)).abs() < 1e-6);
        // Directly left: atan2(+0, −x) = +π.
        let r = run(0.0, 89.0, &mut pool, &mut rng);
        assert!((r - (std::f64::consts::PI as f32 - 1.57)).abs() < 1e-5);
    }

    /// `NPC.cs:32744` — TargetClosest runs FIRST and the aim reads the
    /// PICKED slot: the aggro offset hands the pick to the farther
    /// player and the velocity chases THAT one; the ticker stamps
    /// `targetSetFrame` (`:77780`).
    #[test]
    fn s1_d_3_s56_the_target_pick_precedes_the_aim() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(3);
        // Slot 0 near (Manhattan 300), slot 1 far (459) with aggro
        // 200 → 259 wins.
        let near = cand(400.0, 89.0);
        let mut far = cand(430.0, 218.0);
        far.aggro = 200;
        let players = [near, far];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 42,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        assert_eq!(n.target, 1, "the aggro offset pulls the pick");
        assert_eq!(n.target_set_frame, 42);
        // The aim is slot 1's centre (440, 239) − (110, 110) =
        // (330, 129) — both components positive.
        assert!(n.vx > 0.0 && n.vy > 0.0);
        let dx = 330.0f32;
        let dy = 129.0f32;
        let s = 12.0f32 / ((dx * dx + dy * dy) as f64).sqrt() as f32;
        assert_eq!(n.vx, (dx * s) / 101.0, "the aim reads the picked slot");
    }

    /// `NPC.cs:32755-32758` + `Dust.cs:170-190` — the stream-peek
    /// witness: an accepted low-band spawn draws EXACTLY Next(16),
    /// Next(16), Next(-20,21), Next(-20,21), Next(3), Next(-20,21)
    /// in that order (pos X, pos Y, vel X, vel Y, frame row, scale);
    /// the arm's post-writes then scale the velocity ×0.1 and ASSIGN
    /// scale 1.3 (the rolled scale is consumed but overwritten).
    #[test]
    fn s1_d_3_s56_the_dust_draws_replay_in_the_cited_order() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(777);
        let peek = rng.clone();
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        // Replay the cited draws by hand on the pre-step clone.
        let mut r = peek;
        let px = r.next_max(16); // Dust.cs:170 — Next(width-4)
        let py = r.next_max(16); // :171 — Next(height-4)
        let vx = r.next_range(-20, 21); // :172
        let vy = r.next_range(-20, 21); // :173
        let fy = r.next_max(3); // :175
        let _scale_roll = r.next_range(-20, 21); // :190 — consumed, then overwritten
        assert_eq!(rng, r, "the arm consumed exactly the six cited draws");
        let d = pool.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 180);
        assert_eq!(d.position.x, 100.0 + px as f32 + 4.0);
        assert_eq!(d.position.y, 100.0 + py as f32 + 4.0);
        assert_eq!(d.velocity.x, (vx as f32 * 0.1) * 0.1, "roll ×0.1 then arm ×0.1");
        assert_eq!(d.velocity.y, (vy as f32 * 0.1) * 0.1);
        assert_eq!(d.frame_y, 10 * fy + 30, "the per-100 shift: 180 → +30 (:181-186)");
        assert_eq!(d.frame_x, 10 * 180 - 1000);
        assert_eq!(d.scale, 1.3, "assignment, not the rolled scale (:32757)");
        assert!(d.no_gravity);
        assert_eq!(d.alpha, 0);
        assert_eq!(d.color, Color::default(), "default(Color) is all-zero");
    }

    /// `NPC.cs:32754`/`:32759` — the netOffset round trip: the dust
    /// spawns at position + netOffset, and the NPC's position is
    /// restored by the subtract (not left shifted).
    #[test]
    fn s1_d_3_s56_the_dust_position_rides_the_net_offset_round_trip() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(4);
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        n.net_offset = Vec2::new(5.0, -3.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        assert_eq!((n.x, n.y), (100.0, 100.0), "the subtract restores (:32759)");
        let d = pool.slot(0);
        // The spawn box is [x+5+4, x+5+4+16) × [y-3+4, y-3+4+16).
        assert!(d.position.x >= 109.0 && d.position.x < 125.0);
        assert!(d.position.y >= 101.0 && d.position.y < 117.0);
    }

    /// `Dust.cs:77-80` + `NPC.cs:32756-32758` — a refused spawn
    /// (gameMenu) returns the 6000 sentinel with ZERO draws, and the
    /// C# still writes velocity/scale/noGravity into Main.dust[6000]
    /// — the port keeps the sentinel writes.
    #[test]
    fn s1_d_3_s56_a_refused_spawn_writes_the_sentinel_and_draws_nothing() {
        let w = air();
        let mut pool = DustPool::new();
        let mut c = ctx();
        c.game_menu = true;
        let mut rng = UnifiedRandom::new(5);
        let peek = rng.clone();
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        assert_eq!(rng, peek, "the menu guard refuses before any draw");
        let d = pool.slot(DUST_SENTINEL);
        assert_eq!(d.scale, 1.3, "the post-writes land on the sentinel too");
        assert!(d.no_gravity);
        assert_eq!(d.velocity, Vec2::ZERO, "0 × 0.1 = 0");
        // The movement half still ran (the guard is inside NewDust).
        assert!(n.vx > 0.0);
    }

    /// `Dust.cs:97-101` — the off-screen cull refuses with ZERO draws
    /// (before the slot scan), sentinel writes included.
    #[test]
    fn s1_d_3_s56_the_offscreen_cull_draws_nothing() {
        let w = air();
        let mut pool = DustPool::new();
        let mut c = ctx();
        c.screen_position = Vec2::new(100000.0, 100000.0);
        let mut rng = UnifiedRandom::new(6);
        let peek = rng.clone();
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        assert_eq!(rng, peek, "the cull precedes the pressure bands");
        assert_eq!(pool.slot(DUST_SENTINEL).scale, 1.3);
    }

    /// `Dust.cs:114-120` — the short-circuit gate draw: with slot 0
    /// active and maxDustToDraw 1, the first free slot (1) is above
    /// the 0.9 band, so NewDust rolls Next(4) and refuses unless it
    /// is 0. The witness replays the ONE gate draw and pins the
    /// stream advance to it (refusal) or to it + the six spawn draws
    /// (acceptance) — no hardcoded seed values.
    #[test]
    fn s1_d_3_s56_the_pressure_band_gate_draw_short_circuits() {
        let w = air();
        let mut pool = DustPool::new();
        pool.slot_mut(0).active = true;
        let mut c = ctx();
        c.max_dust_to_draw = 1;
        let mut rng = UnifiedRandom::new(99);
        let mut peek = rng.clone();
        let gate = peek.next_max(4); // Dust.cs:116
        let players = [cand(400.0, 489.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        if gate != 0 {
            assert_eq!(rng, peek, "refusal consumed exactly the gate draw");
            assert!(!pool.slot(1).active);
            assert_eq!(pool.slot(DUST_SENTINEL).scale, 1.3);
        } else {
            // Accepted: the six default-block draws follow the gate.
            let _ = peek.next_max(16);
            let _ = peek.next_max(16);
            let _ = peek.next_range(-20, 21);
            let _ = peek.next_range(-20, 21);
            let _ = peek.next_max(3);
            let _ = peek.next_range(-20, 21);
            assert_eq!(rng, peek, "gate + the six spawn draws");
            assert!(pool.slot(1).active);
            assert_eq!(pool.slot(1).scale, 1.3);
        }
    }

    /// `NPC.cs:32748-32753` — zero distance is unguarded: the aim is
    /// 0 × inf = NaN, so velocity and rotation go NaN exactly like
    /// the C# (no invented clamp, no panic).
    #[test]
    fn s1_d_3_s56_the_zero_distance_aim_is_nan_like_the_csharp() {
        let w = air();
        let mut pool = DustPool::new();
        let c = ctx();
        let mut rng = UnifiedRandom::new(7);
        // Player centre == spirit centre (110, 110).
        let players = [cand(100.0, 89.0)];
        let mut n = spirit(100.0, 100.0);
        let mut env = Style56Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
        };
        style56_step(&mut n, &mut env);
        assert!(n.vx.is_nan() && n.vy.is_nan(), "0 × (12/0) = NaN");
        assert!(n.rotation.is_nan(), "atan2(NaN, NaN) = NaN");
    }

    /// The arm writes NO ai[], NO hp, NO facing beyond the target
    /// selection's, and integrates NO position (the caller owns the
    /// noTileCollide step); two identical runs give identical state.
    #[test]
    fn s1_d_3_s56_the_arm_writes_no_counters_and_is_deterministic() {
        let w = air();
        let players = [cand(400.0, 489.0)];
        let run = || {
            let mut pool = DustPool::new();
            let c = ctx();
            let mut rng = UnifiedRandom::new(31337);
            let mut n = spirit(100.0, 100.0);
            n.ai = [1.0, 2.0, 3.0, 4.0];
            let hp = n.hp;
            let mut env = Style56Env {
                world: &w,
                players: &players,
                ticker: 0,
                dust: &mut pool,
                dust_ctx: &c,
                rng: &mut rng,
            };
            for _ in 0..300 {
                style56_step(&mut n, &mut env);
            }
            assert_eq!(n.ai, [1.0, 2.0, 3.0, 4.0], "no ai[] writes in the block");
            assert_eq!(n.hp, hp, "no hp writes");
            assert_eq!((n.x, n.y), (100.0, 100.0), "no position integration");
            (n.vx, n.vy, n.rotation, n.target, rng)
        };
        assert_eq!(run(), run(), "identical inputs → identical state + stream");
    }
}
