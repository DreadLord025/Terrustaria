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
//! Batch 3 (the remaining spheres + the caster/boss live arms):
//! - 89543/89546 — the Water Sphere 33 (`:89538-89548`): the 20-pair
//!   dust-172 shape, scales 1.2/0.6.
//! - 89695/89698 — the Chaos Ball 30/665 (`:89690-89700`): the 20-pair
//!   dust-27 shape, scales 2/1.
//! - 89481/89482 — Meteor Head 23 (`:89470-89485`): ×100 — the Next(2)
//!   25/6 selector BEFORE the spawn, then the dust-6 velocity pair
//!   (vx·0.2, vy·0.2) alpha 100 scale 2 noGravity.
//! - 89509 — Fire Imp 24 (`:89503-89512`): ×100 dust 6 at (vx,vy),
//!   alpha 100 scale 2.5 noGravity.
//! - 84264 — Duke Fishron 370 (`:84258-84266`): ×100 dust 5.
//! - 84213 — Sharkron 372/373 (`:84207-84215`): ×100 dust 5.
//! - 86527 — Wandering Eye 133 (`:86521-86534`): ×100 dust 5 (the
//!   half-life gore latch `:86529-86533` is draw-free, d55's body).
//! Batch 4 (the boss/worm live arms):
//! - 87279 — Eye of Cthulhu 4 (`:87275-87284`, the 4/126/125 group):
//!   ×100 dust 5.
//! - 87345 — Servant of Cthulhu 5 (`:87341-87350`): ×50 dust 5.
//! - 87594 — the Giant Worm 10/11/12 (`:87590-87599`): ×50 dust 5.
//! - 87560 — the Devourer 7/8/9 (`:87556-87565`): ×100 dust 18,
//!   alpha/color/scale ride the NPC.
//! - 87626 — the Eater of Worlds 13/14/15 (`:87622-87631`): ×100 dust
//!   18, alpha/color/scale.
//! - 88913 — the Wyvern 87-92 (`:88909-88919`): ×50 dust 16, (0,0),
//!   scale 1.5, velocity ×1.5, noGravity.
//! - 87362 — the Wall of Flesh 113/114 (`:87358-87366`): FLAT 20 dust 5
//!   (the every-segment death fan `:81476-81484` is g60's).
//! - 87414 — the Hungry 115/116 (`:87410-87418`): FLAT 5 dust 5.
//! - 87440 — the Leech 117-119 (`:87436-87444`): FLAT 5 dust 5.
//! - 87485 — Queen Bee 222 (`:87481-87489`): ×100 dust 18,
//!   alpha/color/scale.
//! Batch 5 (the Martian/solar live arms — two shapes):
//! - 84308/84328/84368/84388/84456/84476/84508 — the martian grunts
//!   (382/390/383/385/386/389/388/399, `:84302-84533`): the
//!   `dmg/(lifeMax·50)` bound, dust 227 (dir,-1).
//! - 84348/84408/84431 — the ForceBubble 384 (`:84342-84351`, ×50), the
//!   Tesla Turret 387 (`:84402-84411`, ×20) and the Martian Walker 520
//!   (`:84425-84434`, ×20): dust 226 at (-dir,-1), post-spawn position
//!   Lerp(pos, Centre, 0.25) + scale 0.5.
//! Batch 6 (the solar pillar's gated-child live arms — ×20 dust 241 +
//! the Next-gated dust-6 child, noGravity/scale 1.5/fadeIn 1/vel ×3):
//! - 82791/82794 — 518/416 (`:82785-82800`), Next(4)==0.
//! - 82829/82832 — 419 (`:82823-82838`), Next(4)==0.
//! - 82890/82893 — 415 (`:82884-82899`), Next(4)==0.
//! - 82924/82927 — 417 (`:82918-82933`), Next(4)==0.
//! - 82965/82968 — 418 (`:82959-82974`), Next(2)==0 (the tighter gate).
//! Batch 7 (the Vortex pillar's gated-child live arms — dust 241 parent
//! + the Next-gated dust-229 child, the same four post-writes):
//! - 83045/83048 — Vortex Hornet Queen 426 (`:83039-83054`), ×50,
//!   Next(4)==0.
//! - 83092/83095 — Vortex Hornet 427 (`:83086-83101`), ×20, Next(2)==0.
//! - 83131/83134 — Vortex Soldier 429 (`:83125-83140`), ×20, Next(4)==0.
//! - 83213/83216 — Vortex Rifleman 425 (`:83207-83222`), ×20, Next(4)==0.
//! - 83253 — Vortex Larva 428 (`:83247-83254`), ×20, NO child gate live
//!   (the 229/240 selector is death-half only).
//! Batch 8 (the Nebula pillar's gated-child live arms — dust 249 parent
//! + the Next-gated dust-242 child, the same four post-writes):
//! - 83280/83283 — Nebula Soldier 424 (`:83274-83289`), ×20, Next(4)==0.
//! - 83318/83321 — Nebula Beast 423 (`:83312-83327`), ×50, Next(4)==0.
//! - 83351/83354 — Nebula Headcrab 421 (`:83345-83360`), ×20, Next(2)==0.
//! - 83384/83387 — Nebula Brain 420 (`:83378-83393`), ×20, Next(2)==0.
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
    /// `base.Center` (the martian Lerp arms read it, `NPC.cs:84349`).
    pub cx: f32,
    pub cy: f32,
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
        let (cx, cy) = n.center();
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
            cx,
            cy,
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
    // `:89526-89548`/`:89690-89700` — the three spheres: EVERY strike
    // (live or dead — the arms carry NO life gate, so the killing blow's
    // HitEffect still bursts; they are strike arms, not death-event
    // arms). 20 pairs at the reflected velocity; the Item10 cues
    // (`:89528`/`:89540`/`:89692`) are c13's sound lane. Pair shape:
    // first noGravity ×2 at scale A, second ×2 at scale B.
    let sphere = match s.npc_type {
        25 => Some((6, 2.0, 1.0)),    // `:89531`/`:89534`
        33 => Some((172, 1.2, 0.6)),  // `:89543`/`:89546`
        30 | 665 => Some((27, 2.0, 1.0)), // `:89695`/`:89698`
        _ => None,
    };
    if let Some((dust_type, scale_a, scale_b)) = sphere {
        for _ in 0..20 {
            let idx = live_dust(
                s, dust_type, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), scale_a,
                rng, dust, ctx, world,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 2.0;
            let idx = live_dust(
                s, dust_type, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), scale_b,
                rng, dust, ctx, world,
            );
            dust.slot_mut(idx).velocity *= 2.0;
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
        // `:89470-89485` — Meteor Head: ×100, the Next(2) selector
        // BEFORE the spawn (0 → dust 6, else 25), then the dust-6
        // velocity pair (alpha 100, scale 2, noGravity). The death
        // halves' second 50-loop is g60's.
        23 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let mut t = 25;
                if rng.next_max(2) == 0 {
                    t = 6; // `:89477-89480`
                }
                live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world); // `:89481`
                let idx = live_dust(
                    s, 6, s.vx * 0.2, s.vy * 0.2, 100, Color::default(), 2.0,
                    rng, dust, ctx, world,
                ); // `:89482`
                dust.slot_mut(idx).no_gravity = true; // `:89483`
            }
        }
        // `:89503-89512` — Fire Imp: ×100 dust 6 at the NPC's own
        // velocity, alpha 100 scale 2.5 noGravity.
        24 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(
                    s, 6, s.vx, s.vy, 100, Color::default(), 2.5,
                    rng, dust, ctx, world,
                );
                dust.slot_mut(idx).no_gravity = true; // `:89510`
            }
        }
        // `:84258-84266` — Duke Fishron's live blood (the death half's
        // 150-loop + the six centre gores are g60's).
        370 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84207-84215` — the Sharkrons' live blood (the death half's
        // 75-dust centre burst is g60's).
        372 | 373 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86521-86534` — the Wandering Eye: ×100 dust 5; the half-life
        // gore latch (`:86529-86533`) draws NOTHING (the body is d55's).
        133 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87275-87284` — the Eye of Cthulhu group (4/126/125 — the
        // twins share the EoC blood arm): ×100 dust 5. The death half's
        // 150-loop + the gore argument pairs are g60/d55's.
        4 | 126 | 125 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87341-87350` — the Servant of Cthulhu: ×50 dust 5.
        5 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87590-87599` — the Giant Worm segments: ×50 dust 5.
        10 | 11 | 12 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87556-87565` — the Devourer segments: ×100 dust 18, the
        // NPC's alpha/color/scale.
        7 | 8 | 9 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:87622-87631` — the Eater of Worlds segments: ×100 dust 18.
        13 | 14 | 15 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:88909-88919` — the Wyvern segments: ×50 dust 16 at (0,0),
        // scale 1.5, post-spawn velocity ×1.5 + noGravity (the death
        // half's ×2 + the Next(1,4) gore batch are g60/d55's).
        87..=92 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 16, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5; // `:88916`
                d.no_gravity = true; // `:88917`
            }
        }
        // `:87358-87366` — the Wall of Flesh + Eye: FLAT 20 dust 5 (no
        // dmg bound). The killing blow's every-segment fan
        // (`:81476-81484`) is g60's death lane.
        113 | 114 => {
            for _ in 0..20 {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87410-87418` — the Hungry: FLAT 5 dust 5.
        115 | 116 => {
            for _ in 0..5 {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87436-87444` — the Leech segments: FLAT 5 dust 5.
        117..=119 => {
            for _ in 0..5 {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87481-87489` — Queen Bee: ×100 dust 18, alpha/color/scale.
        222 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:84302-84533` — the martian grunts' dust 227: the bound is
        // `dmg / (lifeMax * 50)` (the C# `(double)(this.lifeMax * 50)` —
        // the ×50 sits INSIDE the divisor, unlike the ×mult arms).
        382 | 390 | 383 | 385 | 386 | 389 | 388 | 399 => {
            let bound = s.dmg / (s.life_max * 50) as f64;
            let mut count = 0u32;
            while (count as f64) < bound {
                live_dust(s, 227, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                count += 1;
            }
        }
        // `:84342-84351`/`:84402-84411`/`:84425-84434` — the ForceBubble
        // (×50), the Tesla Turret and the Walker (×20): dust 226 at
        // (-dir, -1), post-spawn position Lerp toward Centre 0.25 +
        // scale 0.5 (the XNA float lerp, per axis).
        384 | 387 | 520 => {
            let mult = if s.npc_type == 384 { 50.0 } else { 20.0 };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                let idx = live_dust(s, 226, -dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.position.x += (s.cx - d.position.x) * 0.25; // `:84349`
                d.position.y += (s.cy - d.position.y) * 0.25;
                d.scale = 0.5; // `:84350`
            }
        }
        // `:82785-82974`/`:83039-83222` — the solar/Vortex gated-child
        // arms: the dust-241 parent, then a Next(N)==0 gate per iteration
        // spawning the child (solar 6, Vortex 229) with noGravity, scale
        // 1.5, fadeIn 1, velocity ×3. Gates: Next(4) default; Next(2) for
        // 418 (`:82966`) and 427 (`:83093`); 426 rides ×50 (`:83043`).
        518 | 416 | 419 | 415 | 417 | 418 | 426 | 427 | 429 | 425 => {
            let (gate, child, mult) = match s.npc_type {
                418 => (2, 6, 20.0),
                426 => (4, 229, 50.0),
                427 => (2, 229, 20.0),
                429 | 425 => (4, 229, 20.0),
                _ => (4, 6, 20.0),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                live_dust(s, 241, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                if rng.next_max(gate) == 0 {
                    let idx = live_dust(s, child, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
        }
        // `:83247-83254` — the Vortex Larva: ×20 dust 241, NO live child
        // gate (the 229/240 selector lives in the death half only).
        428 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 241, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83274-83393` — the Nebula gated-child arms: dust 249 parent +
        // the dust-242 child (the same four post-writes). 423 rides ×50
        // (`:83316`); 421/420 gate Next(2) (`:83352`/`:83385`).
        424 | 423 | 421 | 420 => {
            let (gate, mult) = match s.npc_type {
                423 => (4, 50.0),
                421 | 420 => (2, 20.0),
                _ => (4, 20.0),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                live_dust(s, 249, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                if rng.next_max(gate) == 0 {
                    let idx = live_dust(s, 242, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
        }
        // `:89526-89548`/`:89690-89700` — the three spheres are handled
        // ABOVE the life gate (the every-strike arms); this match never
        // sees 25/33/30/665.
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
            cx: 112.0,
            cy: 109.0,
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

    /// W4-g62 batch 3 — the Water Sphere/Chaos Ball pair shapes
    /// (`NPC.cs:89543-89547`/`:89695-89699`): dust 172 at 1.2/0.6 and
    /// dust 27 at 2/1, every strike, the stream mirrors all 20 pairs.
    #[test]
    fn w4_g62_the_water_and_chaos_sphere_pairs() {
        let world = air();
        let ctx = ctx();
        for (t, dt, sa, sb) in [(33, 172, 1.2f32, 0.6f32), (30, 27, 2.0, 1.0), (665, 27, 2.0, 1.0)] {
            let mut s = snap(t, 1, 30, 30.0);
            s.vx = -3.0;
            s.vy = 1.5;
            let mut rng = UnifiedRandom::new(41);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(41);
            let mut twin = DustPool::new();
            for _ in 0..20 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, dt,
                    0.6, -0.3, 100, Color::default(), sa,
                );
                let d = twin.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, dt,
                    0.6, -0.3, 100, Color::default(), sb,
                );
                twin.slot_mut(idx).velocity *= 2.0;
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 3 — the Meteor Head's selector-then-pair order
    /// (`NPC.cs:89476-89483`): Next(2) BEFORE the first spawn, the
    /// dust-6 pair after; one iteration mirrors exactly.
    #[test]
    fn w4_g62_the_meteor_head_selector_rides_before_the_spawn() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(23, 50, 50, 1.0); // ×100 → 2
        s.vx = 5.0;
        s.vy = 2.0;
        let mut rng = UnifiedRandom::new(43);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(43);
        let mut twin = DustPool::new();
        for _ in 0..2 {
            let mut t = 25;
            if twin_rng.next_max(2) == 0 {
                t = 6;
            }
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                1.0, 0.4, 100, Color::default(), 2.0,
            );
            twin.slot_mut(idx).no_gravity = true;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 3 — the Fire Imp's own-velocity dust 6
    /// (`NPC.cs:89509-89510`): scale 2.5, alpha 100, noGravity.
    #[test]
    fn w4_g62_the_fire_imp_rides_its_own_velocity() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(24, 60, 60, 3.0); // ×100 → 5
        s.vx = -2.0;
        s.vy = 0.5;
        let mut rng = UnifiedRandom::new(47);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(47);
        let mut twin = DustPool::new();
        for _ in 0..5 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                -2.0, 0.5, 100, Color::default(), 2.5,
            );
            twin.slot_mut(idx).no_gravity = true;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 3 — the Fishron/Sharkron/WanderingEye live arms are
    /// the plain ×100 dust-5 shape (`NPC.cs:84264`/`:84213`/`:86527`),
    /// and the Wandering Eye's half-life gore latch draws NOTHING.
    #[test]
    fn w4_g62_the_blood_arms_share_the_dust5_shape() {
        let world = air();
        let ctx = ctx();
        for t in [370, 372, 373, 133] {
            let mut s = snap(t, 20, 100, 10.0); // ×100 → 10
            if t == 133 {
                s.life = 20; // below half — the gore latch arm, no draws
            }
            let mut rng = UnifiedRandom::new(53);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(53);
            let mut twin = DustPool::new();
            for _ in 0..10 {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 4 — the WoF/Hungry/Leech FLAT counts (`NPC.cs:87362`
    /// 20, `:87414`/`:87440` 5): no dmg bound — a 1-damage hit and an
    /// overkill hit draw the same count.
    #[test]
    fn w4_g62_the_wof_family_counts_are_flat() {
        let world = air();
        let ctx = ctx();
        for (t, want) in [(113, 20), (114, 20), (115, 5), (116, 5), (117, 5), (118, 5), (119, 5)] {
            for dmg in [1.0, 9999.0] {
                let s = snap(t, 100, 8000, dmg);
                let mut rng = UnifiedRandom::new(59);
                let mut dust = DustPool::new();
                replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
                assert_eq!(
                    dust.slots().iter().filter(|d| d.active).count(),
                    want,
                    "type {t} dmg {dmg}"
                );
            }
        }
    }

    /// W4-g62 batch 4 — the Wyvern's dust-16 post-writes (`NPC.cs:88915-
    /// 88917`): scale 1.5 at spawn, velocity ×1.5, noGravity — the LIVE
    /// multiplier is 1.5, not the death half's 2.
    #[test]
    fn w4_g62_the_wyvern_live_multiplier_is_1_5() {
        let world = air();
        let ctx = ctx();
        let s = snap(87, 4000, 4000, 80.0); // ×50 → 1
        let mut rng = UnifiedRandom::new(61);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(61);
        let mut twin = DustPool::new();
        let idx = twin.new_dust(
            &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 16,
            0.0, 0.0, 0, Color::default(), 1.5,
        );
        let d = twin.slot_mut(idx);
        d.velocity *= 1.5;
        d.no_gravity = true;
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 4 — the worm/boss dust-18 arms pass the NPC rows and
    /// the ×100/×50 bounds (`NPC.cs:87560`/`:87626`/`:87485`/`:87594`/
    /// `:87345`/`:87279`).
    #[test]
    fn w4_g62_the_worm_and_boss_bounds() {
        let world = air();
        let ctx = ctx();
        // (type, mult, dust type)
        for (t, mult, dt) in [
            (4, 100.0, 5), (5, 50.0, 5), (10, 50.0, 5),
            (7, 100.0, 18), (13, 100.0, 18), (222, 100.0, 18),
        ] {
            let s = snap(t, 100, 100, 4.0);
            let want = hit_count(4.0, 100, mult);
            let mut rng = UnifiedRandom::new(67);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 5 — the martian divisor (`NPC.cs:84306`): the ×50
    /// sits INSIDE the divisor — a full-lifeMax hit draws 0, a 50×
    /// lifeMax hit draws exactly 1 (NOT the ×mult arms' shape).
    #[test]
    fn w4_g62_the_martian_bound_divides_by_lifemax_times_50() {
        let world = air();
        let ctx = ctx();
        for t in [382, 390, 383, 385, 386, 389, 388, 399] {
            let s = snap(t, 300, 300, 300.0); // 300/(300·50) = 0.02 → 0
            let mut rng = UnifiedRandom::new(71);
            let twin = UnifiedRandom::new(71);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            assert_eq!(rng, twin, "type {t}: a full-life hit draws nothing");
            let big = snap(t, 300, 300, 15000.0); // 15000/15000 = 1 → 1
            let mut rng2 = UnifiedRandom::new(71);
            let mut dust2 = DustPool::new();
            replay_hit_dust(&big, &mut rng2, &mut dust2, &ctx, &world);
            assert_eq!(dust2.slots().iter().filter(|d| d.active).count(), 1, "type {t}");
            assert!(dust2.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 227));
        }
    }

    /// W4-g62 batch 5 — the ForceBubble/Turret/Walker Lerp arm
    /// (`NPC.cs:84348-84350`): dust 226 at (-dir,-1), the position lerps
    /// 0.25 toward Centre, scale pinned 0.5 — the mirror replays the
    /// exact post-writes on the twin pool.
    #[test]
    fn w4_g62_the_martian_lerp_arm_mirrors_the_post_writes() {
        let world = air();
        let ctx = ctx();
        for (t, mult) in [(384, 50.0), (387, 20.0), (520, 20.0)] {
            let s = snap(t, 1000, 1000, 40.0 * mult); // → 40 dusts
            let mut rng = UnifiedRandom::new(73);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(73);
            let mut twin = DustPool::new();
            for _ in 0..40 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 226,
                    -1.0, -1.0, 0, Color::default(), 1.0,
                );
                let d = twin.slot_mut(idx);
                d.position.x += (112.0 - d.position.x) * 0.25;
                d.position.y += (109.0 - d.position.y) * 0.25;
                d.scale = 0.5;
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 6 — the solar gated-child order (`NPC.cs:82791-
    /// 82799`): the parent 241's internals draw FIRST, then the Next(4)
    /// gate; on a hit the child 6 spawns with the four post-writes. The
    /// 418 arm's gate is Next(2) (`:82966`) — twice the child rate.
    #[test]
    fn w4_g62_the_solar_child_gate_rides_after_the_parent() {
        let world = air();
        let ctx = ctx();
        for (t, gate) in [(518, 4), (416, 4), (419, 4), (415, 4), (417, 4), (418, 2)] {
            let s = snap(t, 100, 100, 50.0); // ×20 → 10 iterations
            let mut rng = UnifiedRandom::new(79);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(79);
            let mut twin = DustPool::new();
            let mut children = 0;
            for _ in 0..10 {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 241,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                if twin_rng.next_max(gate) == 0 {
                    children += 1;
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = twin.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
            assert_eq!(
                dust.slots().iter().filter(|d| d.active && d.dust_type == 6).count(),
                children,
                "type {t}"
            );
        }
    }

    /// W4-g62 batch 7 — the Vortex rows (`NPC.cs:83043-83053`/`:83090-
    /// 83100`/`:83129-83139`/`:83211-83221`/`:83251-83254`): the 229
    /// child, 426's ×50, 427's Next(2), and the Larva's gate-free live
    /// loop (a full-life hit draws exactly the parent count).
    #[test]
    fn w4_g62_the_vortex_rows_and_the_gate_free_larva() {
        let world = air();
        let ctx = ctx();
        for (t, gate, mult) in [(426, 4, 50.0), (427, 2, 20.0), (429, 4, 20.0), (425, 4, 20.0)] {
            let s = snap(t, 100, 100, 50.0); // bound = 50/100·mult = mult/2 iterations
            let iters = hit_count(s.dmg, s.life_max, mult);
            let mut rng = UnifiedRandom::new(83);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(83);
            let mut twin = DustPool::new();
            for _ in 0..iters {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 241,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                if twin_rng.next_max(gate) == 0 {
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 229,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = twin.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
        // The Larva: NO gate draw — the stream is exactly 10 parent spawns.
        let larva = snap(428, 100, 100, 50.0); // ×20 → 10
        let mut rng = UnifiedRandom::new(83);
        let mut dust = DustPool::new();
        replay_hit_dust(&larva, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(83);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 241,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 8 — the Nebula rows (`NPC.cs:83280-83288`/`:83318-
    /// 83326`/`:83351-83359`/`:83384-83392`): dust 249 parent + 242
    /// child; 423 ×50 Next(4), 421/420 ×20 Next(2).
    #[test]
    fn w4_g62_the_nebula_rows() {
        let world = air();
        let ctx = ctx();
        for (t, gate, mult) in [(424, 4, 20.0), (423, 4, 50.0), (421, 2, 20.0), (420, 2, 20.0)] {
            let s = snap(t, 100, 100, 50.0); // mult/2 iterations
            let iters = hit_count(s.dmg, s.life_max, mult);
            let mut rng = UnifiedRandom::new(89);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(89);
            let mut twin = DustPool::new();
            for _ in 0..iters {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 249,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                if twin_rng.next_max(gate) == 0 {
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 242,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = twin.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }
}
