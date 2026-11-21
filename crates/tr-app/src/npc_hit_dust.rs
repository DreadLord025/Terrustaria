//! W4-g62 — the NPC HIT dust lane: `NPC.HitEffect`'s LIVE (`life > 0`)
//! per-kind dust arms (`NPC.cs:81654-89834`), reached from
//! `NPC.StrikeNPC_Inner`'s `HitEffect(hitDirection, num)` tail
//! (`NPC.cs:81476-81489`). The DEATH halves of the same arms are
//! W4-g59/g60's lane, the spawn-side dust g61's, the hit sounds c13's
//! (`strike_hit_cue`/`queue_hit_effect_sound`), the gore BODIES d55's —
//! where a gore call sits between dust draws its ARGUMENT draws are
//! consumed here in C# order (the W4-e2 split precedent).
//!
//! Contract (the S1-E-1 pool + d50's DustPool, `tr_core::particles`):
//! the strike captures a [`HitDustSnapshot`] at the C# `HitEffect` call
//! moment — AFTER the damage write and the knockback (`NPC.cs:81387-
//! 81475` precede `:81488`), so `life` is post-damage and `vx/vy` are
//! post-knockback. The sim drains the queue in `update_npcs` and every
//! draw rides the `UpdateNPCs` stream in C# statement order through the
//! verbatim `DustPool::new_dust` (`Dust.cs:75-256`): explicit `Main.rand`
//! argument/gate draws FIRST, then the pool internals, then the
//! post-spawn writes (the sentinel slot 6000 takes the writes exactly
//! like the C# when the pool refuses).
//!
//! RNG deviation (recorded, the W-10 gallop precedent): the C# runs
//! `HitEffect` inside the STRIKE's swap (`UpdatePlayers` for melee,
//! `UpdateProjectiles` for the projectile walk); the port's drain rides
//! `UpdateNPCs` after the tick — the draw SEQUENCE per strike is
//! identical, the stream placement is the documented pool-pass fold
//! (docs/ANALYSIS.md "The NPC hit-dust lane (W4-g62)").
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id), batch 1:
//! - 84747 — the slime family's live gel (`:84721-84749`, types
//!   1/16/71/244/535): `dmg/lifeMax*100` × dust 4, alpha/color ride the
//!   NPC's own rows.
//! - 86455 — the demon-eye family (`:86449-86457`, types 2/190-194/
//!   317/318): ×100 dust 5.
//! - 86943 — the zombie/critter group (`:86937-86967`): ×100 dust 5,
//!   then the 186/632 gore-gate `Next(5)` draws (bodies d55's), the
//!   187 SlimedZombie extra loop and the 590/591 dry triple.
//! - 86957 — the 187 extra ×200 dust 4 (alpha 125, blue, height 24).
//! - 86964 — the 590/591 `!wet` 3 × dust 6 (same tint, height 24).
//! - 85803 — the bat group (`:85797-85805`, 49/51/93/150/152/226): ×30
//!   dust 5.
//! - 88533 — the skeleton group (`:88527-88535`): ×50 dust 26.
//! - 89556 — the goblin group (`:89550-89558`, 26/27/28/29/73/111):
//!   ×100 dust 5.
//! - 87508 — the Eater-of-Souls group (`:87502-87510`, 6/94/166): ×100
//!   dust 18, alpha/color/scale ride the NPC.
//! - 89708 — the hornet group (`:89702-89710`, 42/176/205/231-235):
//!   ×100 dust 18, alpha/color/scale ride the NPC.
//! Batch 2 (the slime-variant/jellyfish/lava/boss/sphere arms):
//! - 85415 — Crimslime 183 (`:85409-85417`): ×100 dust 136, (0,0),
//!   this.alpha/color.
//! - 85433 — CorruptSlime 81/121 (`:85427-85435`): ×100 dust 14, (0,0).
//! - 85484 — IlluminantSlime 120/137/138 (`:85478-85487`): ×50 dust 71,
//!   (0,0), alpha 200, velocity ×1.5.
//! - 85343/85345 — IceSlime 147/184 (`:85337-85347`): ×50 PAIRS — dust
//!   76 noGravity + dust 4 (97,200,255,100) alpha 100.
//! - 85549 — the jellyfish trio 63/64/103 (`:85534-85551`): ×50 dust 4,
//!   the per-type tint.
//! - 85568 — LavaSlime/Hellbat 59/60/151 (`:85562-85575`): ×80 dust 6,
//!   (2·dir,-1), this.alpha, scale 1.5, per-dust Next(8)!=0 → noGravity.
//! - 85676 — King Slime 50 (`:85670-85679`): ×300 dust 4, alpha 175,
//!   (0,80,255,100).
//! - 89531/89534 — the Burning Sphere 25 (`:89526-89537`): 20 PAIRS of
//!   dust 6 at (-vx·0.2,-vy·0.2), alpha 100 — scale 2 noGravity ×2, then
//!   scale 1 ×2 (the Item10 cue is c13's lane).
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled
//! by the merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{Npc, NpcKind};

/// The strike-moment snapshot of one `HitEffect(hitDirection, num)` call
/// (`NPC.cs:81488`): every field the dust arms read, captured AFTER the
/// damage + knockback writes. `life <= 0` snapshots are kept in the
/// queue (the g59/g60 death lane reads the same capture shape) but the
/// LIVE replay below emits nothing for them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitDustSnapshot {
    /// `this.type` — the C# NPCID (via `NpcKind::cs_id`).
    pub npc_type: i32,
    /// `this.life` at the call (post-damage; the carrier mirror already
    /// landed, `NPC.cs:81374-81378`).
    pub life: i32,
    pub life_max: i32,
    /// The `dmg` argument — `StrikeNPC_Inner`'s final `num`.
    pub dmg: f64,
    pub hit_dir: i32,
    /// `base.position` / `base.width` / `base.height` (the C# ints).
    pub x: f32,
    pub y: f32,
    pub w: i32,
    pub h: i32,
    /// `base.velocity` post-knockback (the FireImp/sphere arms read it).
    pub vx: f32,
    pub vy: f32,
    /// `this.alpha` / `this.color` / `this.scale` / `base.wet`.
    pub alpha: i32,
    pub color: Color,
    pub scale: f32,
    pub wet: bool,
}

impl HitDustSnapshot {
    /// Capture at the strike tail (the three `StrikeNPC` entry points'
    /// shared position — after the knockback and the bubble re-arm).
    pub fn capture(n: &Npc, dmg: i32, hit_dir: i32) -> Self {
        let (w, h) = n.size();
        Self {
            npc_type: n.kind.cs_id() as i32,
            life: n.hp,
            life_max: n.max_hp,
            dmg: dmg as f64,
            hit_dir,
            x: n.x,
            y: n.y,
            w: w as i32,
            h: h as i32,
            vx: n.vx,
            vy: n.vy,
            alpha: n.alpha as i32,
            color: hit_color(n),
            scale: n.scale,
            wet: n.wet,
        }
    }
}

/// `this.color` for the ported kinds — the `SetDefaults` census: the
/// only writers are the slime-variant rows (`NPC.cs:7685-7753` +
/// `:8583`), the MotherSlime's `:8975` and the unported 537's `:15869`;
/// every other kind keeps `this.color = default` (`NPC.cs:8526`, XNA
/// `default(Color)` = all-zero).
fn hit_color(n: &Npc) -> Color {
    match n.kind {
        NpcKind::Slime => {
            let c = n.slime.stats().color;
            Color::new(c[0], c[1], c[2], c[3])
        }
        // `NPC.cs:8975` — the MotherSlime's own row.
        NpcKind::MotherSlime => Color::new(0, 0, 0, 50),
        _ => Color::default(),
    }
}

/// The C# live-loop bound: `for (int i = 0; (double)i < dmg /
/// (double)lifeMax * mult; i++)` — f64 division FIRST, then the ×mult,
/// counted exactly as the `for` runs it (the spore-plan precedent,
/// `npc_ai_style50.rs`).
fn hit_count(dmg: f64, life_max: i32, mult: f64) -> u32 {
    let bound = dmg / life_max as f64 * mult;
    let mut count = 0u32;
    while (count as f64) < bound {
        count += 1;
    }
    count
}

/// One `Dust.NewDust(base.position, width, height, Type, hitDirection,
/// -1f)`-shaped spawn (the common live-arm body). Post-spawn writes are
/// the caller's; the sentinel takes them like the C#.
#[allow(clippy::too_many_arguments)]
fn live_dust(
    s: &HitDustSnapshot,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> usize {
    dust.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(s.x, s.y),
        s.w,
        s.h,
        dust_type,
        speed_x,
        speed_y,
        alpha,
        color,
        scale,
    )
}

/// Replay one captured strike's LIVE dust arm on the caller's stream.
/// The arm match mirrors the C# chain's exclusivity: every ported type
/// belongs to exactly one arm (the giant `else if` chain, `NPC.cs:85560-
/// 89832`, plus the standalone slime `if` at `:84721`), so a per-type
/// match is faithful. Types outside the ledger emit nothing — their
/// arms land with the later g62 batches or the g59/g60 death lane.
pub fn replay_hit_dust(
    s: &HitDustSnapshot,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dir = s.hit_dir as f32;
    // `:89526-89537` — the Burning Sphere: EVERY strike (live or dead —
    // the arm carries NO life gate, so the killing blow's HitEffect
    // still bursts; it is a strike arm, not a death-event arm). 20
    // pairs of dust 6 at the reflected velocity; the Item10 cue
    // (`:89528`) is c13's sound lane. The pair shape: scale-2 noGravity
    // ×2, then scale-1 ×2.
    if s.npc_type == 25 {
        for _ in 0..20 {
            let idx = live_dust(
                s, 6, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), 2.0,
                rng, dust, ctx, world,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:89532`
            d.velocity *= 2.0; // `:89533`
            let idx = live_dust(
                s, 6, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), 1.0,
                rng, dust, ctx, world,
            ); // `:89534`
            dust.slot_mut(idx).velocity *= 2.0; // `:89535`
        }
        return;
    }
    // The LIVE lane's gate: every remaining arm sits under `if (this.life
    // > 0)` and RETURNS; the death halves are g59/g60's.
    if s.life <= 0 {
        return;
    }
    match s.npc_type {
        // `:84721-84749` — the slime family (1/16/71/244/535). The
        // confetti-slime `ai[1] == 1345` prologue (`:84723-84742`) is a
        // later batch (its Next(139,143)/jitter draws + gore pairs).
        1 | 16 | 71 | 244 | 535 => {
            // `:84745-84748` — ×100 dust 4, this.alpha, this.color.
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 4, dir, -1.0, s.alpha, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:86449-86457` — the demon-eye family.
        2 | 190 | 191 | 192 | 193 | 194 | 317 | 318 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86937-86967` — the zombie/critter group.
        430..=436 | 590 | 591 | 3 | 52 | 53 | 536 | 104 | 109 | 331 | 332 | 132 | 161
        | 162 | 186 | 187 | 188 | 189 | 200 | 223 | 251 | 319 | 320 | 321 | 632 => {
            // `:86941-86944` — ×100 dust 5.
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
            // `:86945-86948` — the PincushionZombie's gore gate: the
            // Next(5) ALWAYS draws for 186 (the gore body is d55's).
            if s.npc_type == 186 {
                let _ = rng.next_max(5);
            }
            // `:86949-86952` — the same gate for 632.
            if s.npc_type == 632 {
                let _ = rng.next_max(5);
            }
            // `:86953-86959` — the SlimedZombie's extra ×200 blue gel,
            // height 24 (NOT base.height).
            if s.npc_type == 187 {
                for _ in 0..hit_count(s.dmg, s.life_max, 200.0) {
                    dust.new_dust(
                        rng, ctx, world, Vec2::new(s.x, s.y), s.w, 24, 4, dir, -1.0,
                        125, Color::new(0, 80, 255, 100), 1.0,
                    );
                }
            }
            // `:86960-86966` — the zombie goldfish pair's dry triple.
            if (s.npc_type == 590 || s.npc_type == 591) && !s.wet {
                for _ in 0..3 {
                    dust.new_dust(
                        rng, ctx, world, Vec2::new(s.x, s.y), s.w, 24, 6, dir, -1.0,
                        125, Color::new(0, 80, 255, 100), 1.0,
                    );
                }
            }
        }
        // `:85797-85805` — the bat group.
        49 | 51 | 93 | 150 | 152 | 226 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 30.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88527-88535` — the skeleton group (the full C# list).
        21 | 31 | 294 | 295 | 296 | 32 | 44 | 45 | 77 | 110 | 167 | 197 | 201 | 202
        | 203 | 287 | 291 | 292 | 293 | 322..=324 | 481 | 449..=452 | 693 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 26, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:89550-89558` — the goblin group.
        26 | 27 | 28 | 29 | 73 | 111 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87502-87510` — the Eater-of-Souls group.
        6 | 94 | 166 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:89702-89710` — the hornet group.
        42 | 176 | 205 | 231..=235 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:85409-85417` — the Crimslime's live crimson gel (the death
        // half's ×2 velocity is g59/g60's).
        183 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 136, 0.0, 0.0, s.alpha, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:85427-85435` — the CorruptSlime pair (81/121).
        81 | 121 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 14, 0.0, 0.0, s.alpha, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:85478-85487` — the hallow trio (120/137/138): ×50 dust 71,
        // alpha 200, post-spawn velocity ×1.5.
        120 | 137 | 138 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 71, 0.0, 0.0, 200, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).velocity *= 1.5; // `:85485`
            }
        }
        // `:85337-85347` — the IceSlime/UmbrellaSlime pair: ×50 PAIRS,
        // dust 76 noGravity then dust 4 with the frost tint alpha 100.
        147 | 184 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 76, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).no_gravity = true; // `:85344`
                live_dust(
                    s, 4, dir, -1.0, 100, Color::new(97, 200, 255, 100), 1.0,
                    rng, dust, ctx, world,
                ); // `:85345`
            }
        }
        // `:85534-85551` — the jellyfish trio: ×50 dust 4, the per-type
        // tint (`:85536-85544`).
        63 | 64 | 103 => {
            let tint = match s.npc_type {
                64 => Color::new(225, 70, 140, 100),
                103 => Color::new(70, 225, 140, 100),
                _ => Color::new(50, 120, 255, 100),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 4, dir, -1.0, 0, tint, 1.0, rng, dust, ctx, world);
            }
        }
        // `:85562-85575` — LavaSlime/Hellbat/SpikedJungleSlime: ×80 dust
        // 6 at (2·dir, -1), this.alpha, scale 1.5; the per-dust Next(8)
        // rides AFTER the spawn internals (`:85569`), != 0 → noGravity.
        // The death half's lava-tile write (`:85584-85668`) is g59/g60's.
        59 | 60 | 151 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 80.0) {
                let idx = live_dust(
                    s, 6, 2.0 * dir, -1.0, s.alpha, Color::default(), 1.5,
                    rng, dust, ctx, world,
                );
                if rng.next_max(8) != 0 {
                    dust.slot_mut(idx).no_gravity = true; // `:85571`
                }
            }
        }
        // `:85670-85679` — King Slime's live gel: ×300 dust 4, alpha 175,
        // the blue tint (the death half's minion batch is g60's).
        50 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 300.0) {
                live_dust(
                    s, 4, dir, -1.0, 175, Color::new(0, 80, 255, 100), 1.0,
                    rng, dust, ctx, world,
                );
            }
        }
        // `:89526-89537` — the Burning Sphere is handled ABOVE the life
        // gate (the every-strike arm); this match never sees 25.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::SlimeVariant;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// The whole world on-screen, the full budget — the pool never
    /// culls, every spawn lands (the `update_rng` permissive ctx).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(200.0 * 16.0, 200.0 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn snap(npc_type: i32, life: i32, life_max: i32, dmg: f64) -> HitDustSnapshot {
        HitDustSnapshot {
            npc_type,
            life,
            life_max,
            dmg,
            hit_dir: 1,
            x: 100.0,
            y: 100.0,
            w: 24,
            h: 18,
            vx: 0.0,
            vy: 0.0,
            alpha: 0,
            color: Color::default(),
            scale: 1.0,
            wet: false,
        }
    }

    /// W4-g62 — the f64 loop bound (`(double)i < dmg / lifeMax * mult`):
    /// the division happens FIRST, so a half-HP 1-damage hit on a
    /// 100-HP NPC draws exactly 1 (0.01 × 100), and a 0.5-damage hit
    /// draws 0 (0.005 × 100 = 0.5 < 1).
    #[test]
    fn w4_g62_the_live_loop_bound_is_f64_division_first() {
        assert_eq!(hit_count(1.0, 100, 100.0), 1);
        assert_eq!(hit_count(0.5, 100, 100.0), 0);
        assert_eq!(hit_count(10.0, 100, 100.0), 10);
        // The bat ×30 and skeleton ×50 multipliers.
        assert_eq!(hit_count(1.0, 100, 30.0), 0);
        assert_eq!(hit_count(4.0, 100, 30.0), 1);
        assert_eq!(hit_count(2.0, 100, 50.0), 1);
        // An overkill on a live snapshot still counts (the C# does not
        // clamp dmg to life).
        assert_eq!(hit_count(200.0, 100, 100.0), 200);
    }

    /// W4-g62 — the death gate: a `life <= 0` snapshot emits nothing
    /// and draws NOTHING (the death halves are g59/g60's lane).
    #[test]
    fn w4_g62_a_dead_snapshot_draws_nothing() {
        let world = air();
        let ctx = ctx();
        for life in [0, -5] {
            let s = snap(1, life, 25, 10.0);
            let mut rng = UnifiedRandom::new(7);
            let twin = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            assert_eq!(rng, twin, "no draws for life {life}");
            assert!(dust.slots().iter().all(|d| !d.active));
        }
    }

    /// W4-g62 — the slime arm's draw mirror (`NPC.cs:84745-84748`):
    /// ×(dmg/lifeMax·100) dust 4 with the NPC's alpha/color, each spawn
    /// the verbatim pool internals — replay equals a hand-rolled twin
    /// pool call-for-call, stream included.
    #[test]
    fn w4_g62_the_slime_live_arm_mirrors_the_pool_draws() {
        let world = air();
        let ctx = ctx();
        let mut n = Npc::new(NpcKind::Slime, 100.0, 100.0, 1, 0.0);
        n.slime = SlimeVariant::Blue;
        n.hp = 20;
        n.max_hp = 25;
        let s = HitDustSnapshot::capture(&n, 5, 1);
        assert_eq!(s.color, Color::new(0, 80, 255, 100), "the blue row");
        let mut rng = UnifiedRandom::new(11);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        // The twin: 5/25 × 100 = 20 spawns, dust 4, (1, -1), alpha 0,
        // the blue tint, scale 1.
        let mut twin_rng = UnifiedRandom::new(11);
        let mut twin = DustPool::new();
        for _ in 0..20 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                1.0, -1.0, 0, Color::new(0, 80, 255, 100), 1.0,
            );
        }
        assert_eq!(rng, twin_rng, "the stream matches call-for-call");
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 — the zombie group's tail draws (`NPC.cs:86945-86966`):
    /// the 186 gore gate consumes exactly ONE Next(5) after the dust
    /// loop (the body is d55's); a plain zombie draws none.
    #[test]
    fn w4_g62_the_pincushion_gore_gate_draws_once() {
        let world = air();
        let ctx = ctx();
        let s = snap(186, 40, 40, 4.0); // ×100 → 10 dusts
        let mut rng = UnifiedRandom::new(3);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(3);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        let _ = twin_rng.next_max(5); // `:86945`
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The plain zombie (3) has no gate draw.
        let p = snap(3, 40, 40, 4.0);
        let mut rng2 = UnifiedRandom::new(3);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&p, &mut rng2, &mut dust2, &ctx, &world);
        let mut plain_rng = UnifiedRandom::new(3);
        let mut plain = DustPool::new();
        for _ in 0..10 {
            plain.new_dust(
                &mut plain_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng2, plain_rng);
        assert_eq!(dust2.slots(), plain.slots());
    }

    /// W4-g62 — the SlimedZombie's extra loop (`NPC.cs:86953-86959`):
    /// ×200 dust 4 at height 24 (NOT the NPC's 18), alpha 125, the blue
    /// tint — after the ×100 dust-5 main loop.
    #[test]
    fn w4_g62_the_slimed_zombie_extra_loop_rides_height_24() {
        let world = air();
        let ctx = ctx();
        let s = snap(187, 50, 50, 1.0); // main ×2, extra ×4
        let mut rng = UnifiedRandom::new(5);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(5);
        let mut twin = DustPool::new();
        for _ in 0..2 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        for _ in 0..4 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 24, 4,
                1.0, -1.0, 125, Color::new(0, 80, 255, 100), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 — the zombie-goldfish dry triple (`NPC.cs:86960-86966`):
    /// 590/591 spawn 3 dust 6 ONLY when `!base.wet`; the wet snapshot
    /// stops after the main loop.
    #[test]
    fn w4_g62_the_zombie_goldfish_triple_is_wet_gated() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(590, 30, 30, 3.0); // main ×10
        let mut rng = UnifiedRandom::new(9);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let dry_count = dust.slots().iter().filter(|d| d.active).count();
        assert_eq!(dry_count, 13, "10 main + 3 dry dust 6");
        s.wet = true;
        let mut rng2 = UnifiedRandom::new(9);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&s, &mut rng2, &mut dust2, &ctx, &world);
        assert_eq!(dust2.slots().iter().filter(|d| d.active).count(), 10);
        assert_ne!(rng, rng2, "the dry triple's internals drew");
    }

    /// W4-g62 — the EoS/hornet arms pass the NPC's alpha/color/scale
    /// through (`NPC.cs:87508`/`:89708`): dust 18, scale = this.scale.
    #[test]
    fn w4_g62_the_eos_arm_passes_the_npc_rows() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(6, 40, 40, 2.0); // ×5
        s.alpha = 55;
        s.color = Color::new(1, 2, 3, 4);
        s.scale = 1.1;
        let mut rng = UnifiedRandom::new(13);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(13);
        let mut twin = DustPool::new();
        for _ in 0..5 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 18,
                1.0, -1.0, 55, Color::new(1, 2, 3, 4), 1.1,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The hornet group shares the shape (dust 18, ×100).
        let h = snap(42, 40, 40, 2.0);
        let mut rng3 = UnifiedRandom::new(13);
        let mut dust3 = DustPool::new();
        replay_hit_dust(&h, &mut rng3, &mut dust3, &ctx, &world);
        let live: Vec<_> = dust3.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 18));
    }

    /// W4-g62 — the skeleton arm's dust 26 and ×50 bound
    /// (`NPC.cs:88531-88534`), the bat arm's ×30 (`:85801`).
    #[test]
    fn w4_g62_the_skeleton_and_bat_arms_match_their_rows() {
        let world = air();
        let ctx = ctx();
        let sk = snap(21, 40, 40, 4.0); // ×50 → 5
        let mut rng = UnifiedRandom::new(17);
        let mut dust = DustPool::new();
        replay_hit_dust(&sk, &mut rng, &mut dust, &ctx, &world);
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 26));
        let bat = snap(49, 45, 45, 3.0); // ×30 → 2
        let mut rng2 = UnifiedRandom::new(17);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&bat, &mut rng2, &mut dust2, &ctx, &world);
        assert_eq!(dust2.slots().iter().filter(|d| d.active).count(), 2);
    }

    /// W4-g62 — the capture reads the POST-strike state: the snapshot's
    /// life is the hp the strike left, the velocity the knockback wrote
    /// (the FireImp/sphere arms read it), the box the C# int truncation.
    #[test]
    fn w4_g62_the_capture_reads_the_post_strike_state() {
        let mut n = Npc::new(NpcKind::MotherSlime, 100.0, 100.0, 1, 0.0);
        n.hp = 70;
        n.max_hp = 90;
        n.vx = 2.5;
        n.vy = -3.0;
        let s = HitDustSnapshot::capture(&n, 20, -1);
        assert_eq!(s.npc_type, 16);
        assert_eq!(s.life, 70);
        assert_eq!(s.dmg, 20.0);
        assert_eq!(s.hit_dir, -1);
        assert_eq!((s.vx, s.vy), (2.5, -3.0));
        assert_eq!(s.color, Color::new(0, 0, 0, 50), "the :8975 row");
        assert_eq!((s.w, s.h), (36, 24), "the MotherSlime box");
    }

    /// W4-g62 batch 2 — the jellyfish trio's per-type tint
    /// (`NPC.cs:85536-85544`): ×50 dust 4, the blue/pink/green rows.
    #[test]
    fn w4_g62_the_jellyfish_tints_match_the_rows() {
        let world = air();
        let ctx = ctx();
        for (t, tint) in [
            (63, Color::new(50, 120, 255, 100)),
            (64, Color::new(225, 70, 140, 100)),
            (103, Color::new(70, 225, 140, 100)),
        ] {
            let s = snap(t, 60, 60, 3.0); // ×50 → 2 (3/60·50 = 2.5 → 3? no: 2.5 → 3)
            let mut rng = UnifiedRandom::new(21);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(21);
            let mut twin = DustPool::new();
            for _ in 0..hit_count(3.0, 60, 50.0) {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                    1.0, -1.0, 0, tint, 1.0,
                );
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 2 — the lava arm's per-dust Next(8) (`NPC.cs:85568-
    /// 85572`): the roll rides AFTER the spawn internals, != 0 sets
    /// noGravity; the whole ×80 sequence mirrors call-for-call.
    #[test]
    fn w4_g62_the_lava_arm_rolls_next8_after_each_spawn() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(59, 50, 50, 1.0); // ×80 → 1
        s.alpha = 50;
        let mut rng = UnifiedRandom::new(23);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(23);
        let mut twin = DustPool::new();
        for _ in 0..1 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                2.0, -1.0, 50, Color::default(), 1.5,
            );
            if twin_rng.next_max(8) != 0 {
                twin.slot_mut(idx).no_gravity = true;
            }
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 2 — the IceSlime pair (`NPC.cs:85343-85345`): each
    /// iteration spawns dust 76 (noGravity) THEN dust 4 (frost tint,
    /// alpha 100) — the pair order is the stream order.
    #[test]
    fn w4_g62_the_ice_slime_pair_order_is_76_then_4() {
        let world = air();
        let ctx = ctx();
        let s = snap(147, 30, 30, 1.0); // ×50 → 1 pair
        let mut rng = UnifiedRandom::new(29);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 2);
        assert_eq!(live[0].dust_type, 76);
        assert!(live[0].no_gravity);
        assert_eq!(live[1].dust_type, 4);
        assert_eq!(live[1].alpha, 100);
        assert_eq!(live[1].color, Color::new(97, 200, 255, 100));
    }

    /// W4-g62 batch 2 — the Burning Sphere's every-strike burst
    /// (`NPC.cs:89529-89536`): 40 dusts even on the KILLING blow (no
    /// life gate), the reflected-velocity pair shape, the stream
    /// mirrors the 20 pairs exactly.
    #[test]
    fn w4_g62_the_burning_sphere_bursts_even_on_death() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(25, -3, 30, 33.0);
        s.vx = 4.0;
        s.vy = -2.0;
        let mut rng = UnifiedRandom::new(31);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(31);
        let mut twin = DustPool::new();
        for _ in 0..20 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                -0.8, 0.4, 100, Color::default(), 2.0,
            );
            let d = twin.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 2.0;
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                -0.8, 0.4, 100, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 2.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 40);
    }

    /// W4-g62 batch 2 — King Slime's ×300 bound (`NPC.cs:85674-85677`)
    /// and the hallow trio's velocity ×1.5 post-write (`:85484-85485`).
    #[test]
    fn w4_g62_king_slime_and_hallow_rows() {
        let world = air();
        let ctx = ctx();
        let ks = snap(50, 2000, 2000, 20.0); // ×300 → 3
        let mut rng = UnifiedRandom::new(37);
        let mut dust = DustPool::new();
        replay_hit_dust(&ks, &mut rng, &mut dust, &ctx, &world);
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 3);
        assert!(live.iter().all(|d| d.dust_type == 4 && d.alpha == 175));
        let il = snap(138, 180, 180, 36.0); // ×50 → 10
        let mut rng2 = UnifiedRandom::new(37);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&il, &mut rng2, &mut dust2, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(37);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 71,
                0.0, 0.0, 200, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 1.5;
        }
        assert_eq!(rng2, twin_rng);
        assert_eq!(dust2.slots(), twin.slots());
    }
}
