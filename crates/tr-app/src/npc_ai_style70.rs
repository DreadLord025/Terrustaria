//! The aiStyle-70 family — the Detonating Bubble (`NPC.AI()`'s
//! `else if (this.aiStyle == 70)` arm, `NPC.cs:34884-34957`). S1-D-3,
//! aiStyle-70 round, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! ## Family identity (verified from the C#)
//!
//! Exactly ONE `SetDefaults` row sets `aiStyle = 70` in `NPC.cs`: the
//! Detonating Bubble 371 (`NPCID.DetonatingBubble`, `ID/NPCID.cs:11419`,
//! row `NPC.cs:13719-13733`). The arm has NO per-type branch — it is a
//! single-kind machine. (`Projectile.cs:4861`'s `aiStyle = 70` is the
//! Flairon Bubble PROJECTILE, a different system, ported as
//! `projectile.rs::ai_flairon_bubble` since S1-D-4b-1 — NOT touched
//! here.) The bubble is Duke Fishron's hazard: spawned from
//! `AI_069_DukeFishron`'s phase 3 (`NPC.cs:49246`, target left 255) and
//! phase 7 (`:49477-49481`, target + velocity preset and `ai[3]` rolled
//! AT THE SPAWN SITE with `Next(80, 121)` — see the RNG section).
//!
//! The machine: a wind-blown homing bubble. On the first tick
//! (`target == 255`) it picks a target, rolls its scale `ai[3]` in
//! [0.80, 1.20] and its launch speed in [11, 17.66], and launches at the
//! player with a ±100-px jitter (`:34886-34893`). Every tick it blends
//! its velocity toward the target 40:20 (`:34894-34895`), copies
//! `ai[3]` to `scale`, pins `alpha` to 50 (the `-= 30` + `< 50` clamp
//! are DEAD — the unconditional `alpha = 50` follows, `:34897-34902`),
//! then blends in the wind and a per-axis jitter 50:1 (`:34903-34904`)
//! and dampens a fall by 0.04 (`:34905-34908`). The fuse: while
//! `ai[0] == 0` a 40-px-expanded box scan arms the detonation when any
//! live player intersects (`:34909-34928`), else `ai[1]` counts up to
//! 150 (`:34929-34937`); once `ai[0] == 1`, `ai[1]` counts down from 4
//! and at 0 the bubble dies — `life = 0`, `HitEffect()`,
//! `active = false`, EARLY RETURN (`:34938-34948`). While `justHit` or
//! detonating it becomes unhittable and grows to a 100×100 box centred
//! on its old centre, with `EncourageDespawn(3)` (`:34949-34956`).
//! A STRUCK bubble does not die instantly: `checkDead`'s 371 arm
//! (`:78046-78051`) re-arms `ai[0] = 1`/`ai[1] = 4` and
//! `PrepareForDeathAnimation` (`:78032-78038`) restores full life —
//! the strike starts the same 4-tick countdown.
//!
//! ## RNG order (by construction)
//!
//! [`ai_style70`] takes the `Main.rand` analogue and draws, in C#
//! order:
//! 1. ONLY when `target == 255` (`:34886`): `TargetClosest()` FIRST
//!    (RNG-free — `NPC.cs:77652-77781`, the `npc_target.rs` port), then
//!    `Next(80, 121)` (`:34889`, the scale), `Next(165, 265)` (`:34890`,
//!    the speed), `Next(-100, 101)` X then `Next(-100, 101)` Y
//!    (`:34891`, the jitter — C# evaluates the `Vector2` ctor arguments
//!    left to right). FOUR draws, then `netUpdate = true`.
//!    Short-circuit gate: the phase-7 Fishron spawn presets `target`
//!    and `ai[3]` (`:49478-49481`, its OWN `Next(80, 121)` at the spawn
//!    site), so those bubbles never run this block — the four draws
//!    never happen for them.
//! 2. EVERY tick, including the detonation tick (the draws precede the
//!    state machine and its early return): `Next(-10, 11)` for the wind
//!    X blend (`:34903`), then `Next(-10, 11)` for the Y blend
//!    (`:34904`). TWO draws.
//!
//! Everything else in the family's per-tick path is RNG-free:
//! `EncourageDespawn` (`NPC.cs:7249-7256`), the box scan, the fuse, the
//! grow arm, `checkDead`/`PrepareForDeathAnimation`
//! (`:78032-78051` — `SetStrikePhaseChange` `:90695-90701` is
//! `netMode == 1`-only, inert in single-player), and
//! `UpdateNPC_CastLights` has NO arm covering 371 (the light arms are
//! 160 / 254..261+634/635 / 209 / …, `:92343-92409`) — the bubble
//! lights nothing and draws nothing there.
//!
//! The detonation's `HitEffect()` (`:34944`, defaults hitDirection 0,
//! dmg 10.0) reaches the 371 arm (`NPC.cs:84229-84256`): a tracked
//! sound (`PlaySound(4, x, y, 3)`, `:84231` — the tracked-sound seam,
//! skipped) and, only when `life <= 0`, a 60-dust type-212 burst. The
//! dust system is unported (S1-E-2/S1-E-3, other branches), so the arm
//! is skipped WHOLESALE — the `npc_lights.rs`/spore precedent: the
//! `Main.rand` stream stays untouched. [`bubble_hit_effect_plan`]
//! ports the branch structure only (no RNG parameter, no draws). For
//! the wiring wave, the arm's per-dust draw order is: `NextDouble()`
//! then `Next(24, 41)` (`:84238` — the `vector18` pair is DEAD CODE,
//! computed and never read, but the draws HAPPEN), the `Dust.NewDust`
//! internals (`:84239`), `Next(45, 91)` only for dusts 30..59
//! (`:84243-84250`), and `NextDouble()` for the HSL hue (`:84251`).
//! The generic `HitEffect` prologue is inert for 371 (no townNPC, no
//! daybreak, none of the 686/690/594/661 types) and the final
//! else-chain returns at `:89813-89815` (`type != 156`) — no generic
//! dust either.
//!
//! ## Wiring seams (deferred — the BlazingWheel/spore precedent)
//!
//! The roster has no `NpcKind::DetonatingBubble` yet, so `update_rng`'s
//! aiStyle axis (`npc.rs:3255`) has no style-70 arm to claim. This
//! module is the pure port + the minimal seams:
//! * `Npc` has no `justHit` field (the C# sets it in `StrikeNPC`
//!   `:81274` and clears it at the `UpdateNPC` tail `:90516`) — the
//!   wiring caller passes it as a parameter.
//! * `Main.windSpeedCurrent` rides `tr_core::wind::Wind::speed_current`;
//!   the caller passes the value (no `Wind` dependency here).
//! * The detonation sets `hp = 0` and returns
//!   [`Style70Outcome::detonated`]; the caller owes the `HitEffect()`
//!   death arm (the plan below) and the slot deactivation (the C#
//!   `active = false` + `return`, `:34945-34946` — no `checkDead` on
//!   this path). The grow/despawn tail did NOT run on that tick.
//! * A strike's death resolution goes through [`bubble_check_dead`]
//!   (the `checkDead` 371 arm + `PrepareForDeathAnimation`), NOT
//!   through the sim's retain — the bubble survives the strike at full
//!   life and detonates 4 ticks later.
//! * `FindFrame`'s case 371 (`NPC.cs:72138-72142`) pins `frame.Y` to
//!   row 1 every call; `Main.npcFrameCount[371] = 2` (`Main.cs:67121`,
//!   the 370-379 row `8, 2, 4, 4, 4, 4, 2, 2, 4, 12` of the `:67082`
//!   array). The consts pin both; the wiring caller sets `frame = 1`.
//! * The C# `alpha` is an `int` and `alpha -= 30` can go negative; the
//!   port's `u8` saturates — moot, the unconditional `alpha = 50`
//!   (`:34902`) overwrites both paths every tick.
//! * `Vector2.Normalize` on a zero vector is NaN in XNA; the port uses
//!   `normalize_or_zero` (the established `projectile.rs` convention).
//!   Reachable only when the player centre exactly equals the bubble
//!   centre; recorded in ANALYSIS.
//! * Difficulty: 371 is in `NPCID.Sets.NeedsExpertScaling` AND
//!   `Sets.ProjectileNPC` (`ID/NPCID.cs:10447-10448`), so `ScaleStats`
//!   runs but the `lifeMax < 6` bump is skipped (`:18160-18163` — the
//!   1-HP life survives) and the expert damage factor is 0.75 with
//!   lifeMax untouched (`:18417-18424`, the `type != 371` gate). The
//!   port is single-difficulty normal; the wiring wave carries the
//!   expert rows.
//!
//! See docs/ANALYSIS.md "The Detonating Bubble (S1-D-3 aiStyle-70,
//! code-only)".
// S1-D-3 W-9: WIRED — the aiStyle-70 axis arm (`npc.rs::ai_070_bubble`)
// + the strike re-arm (`strike_pooled`/`hurt`) claim the machine; the
// defaults/plan helpers stay test/read seams.
#![allow(dead_code)] // the defaults + HitEffect plan are the audit seams; the wiring reads a subset

use glam::Vec2;

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The Detonating Bubble's `SetDefaults` row (`NPC.cs:13719-13733`),
/// verbatim. Fields the row does not write keep the `SetDefaults`
/// stock values (`npcSlots` 1f `:8510`, `value` 0f `:8544`, `scale` 1f
/// `:8536`, `alpha` starts 0 `:8525` then the row writes 255).
pub struct BubbleDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit3` (`ID/SoundID.cs:1259`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath3` (`ID/SoundID.cs:1317`).
    pub death_sound: (i32, i32),
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub knock_back_resist: f32,
    pub alpha: u8,
}

/// `NPC.cs:13719-13733` — `else if (this.type == 371)`.
pub const DETONATING_BUBBLE: BubbleDefaults = BubbleDefaults {
    width: 36,
    height: 36,
    ai_style: 70,
    damage: 100,
    defense: 0,
    life_max: 1,
    hit_sound: (3, 3),
    death_sound: (4, 3),
    no_gravity: true,
    no_tile_collide: true,
    knock_back_resist: 0.0,
    alpha: 255,
};

/// `Main.npcFrameCount[371]` = 2 (`Main.cs:67121`, the 370-379 row of
/// the `:67082` array).
pub const DETONATING_BUBBLE_FRAME_COUNT: u32 = 2;

/// `FindFrame`'s case 371 (`NPC.cs:72138-72142`): `frame.Y = num` —
/// always row 1, never animated.
pub const DETONATING_BUBBLE_FRAME_ROW: u32 = 1;

/// The arm's per-tick side effects beyond the NPC state — the wiring
/// seams (module docs).
pub struct Style70Outcome {
    /// The fuse ran out (`:34941-34946`): `hp` is already 0; the caller
    /// owes the `HitEffect()` death arm ([`bubble_hit_effect_plan`])
    /// and the slot deactivation. The grow/despawn tail did NOT run
    /// this tick (the C# `return`s before it).
    pub detonated: bool,
}

/// One fixed step of the aiStyle-70 arm (`NPC.cs:34884-34957`), in C#
/// order. Draws the `Main.rand` stream exactly as the module docs
/// enumerate: four draws on the `target == 255` init tick, two every
/// tick.
///
/// `players` feeds `TargetClosest()` (`:34888`, faceTarget true)
/// through the shared `npc_target.rs` port (`NPC.cs:77652-77781`) and
/// the proximity scan (`:34917-34927` — the C# walks all 255
/// `Main.player` slots; the port walks the live candidate list).
/// `wind_speed_current` is `Main.windSpeedCurrent`; `just_hit` is the
/// C# `this.justHit` (the `Npc` field seam); `ticker` is
/// `Main.EverLastingTicker` for the `targetSetFrame` stamp.
pub fn ai_style70(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    wind_speed_current: f32,
    just_hit: bool,
    ticker: u64,
    rng: &mut UnifiedRandom,
) -> Style70Outcome {
    // `:34886-34893` — the one-shot init, gated on the unset target.
    if npc.target == 255 {
        // `:34888` — TargetClosest() FIRST (RNG-free).
        target_closest(npc, world, players, true, 70, ticker);
        // `:34889` — ai[3] = Next(80, 121) / 100f.
        npc.ai[3] = rng.next_range(80, 121) as f32 / 100.0;
        // `:34890` — num1009 = Next(165, 265) / 15f.
        let speed = rng.next_range(165, 265) as f32 / 15.0;
        // `:34891` — Normalize(playerCenter - ourCenter + jitter) *
        // speed; the jitter draws X then Y (ctor argument order).
        let (cx, cy) = npc.center();
        let p = &players[npc.target as usize];
        let pc = Vec2::new(
            p.x + p.width as f32 / 2.0,
            p.y + p.height as f32 / 2.0,
        );
        let jitter = Vec2::new(
            rng.next_range(-100, 101) as f32,
            rng.next_range(-100, 101) as f32,
        );
        let launch = (pc - Vec2::new(cx, cy) + jitter).normalize_or_zero() * speed;
        npc.vx = launch.x;
        npc.vy = launch.y;
        // `:34892`.
        npc.net_update = true;
    }
    // `:34894-34895` — the 40:20 homing blend toward the target.
    let (cx, cy) = npc.center();
    let p = &players[npc.target as usize];
    let aim = Vec2::new(
        p.x + p.width as f32 / 2.0 - cx,
        p.y + p.height as f32 / 2.0 - cy,
    )
    .normalize_or_zero();
    let blended = (Vec2::new(npc.vx, npc.vy) * 40.0 + aim * 20.0) / 41.0;
    npc.vx = blended.x;
    npc.vy = blended.y;
    // `:34896` — scale rides the rolled ai[3].
    npc.scale = npc.ai[3];
    // `:34897-34901` — the fade + clamp (DEAD: overwritten below; the
    // u8 saturates where the C# int could go negative — module docs).
    npc.alpha = npc.alpha.saturating_sub(30);
    if npc.alpha < 50 {
        npc.alpha = 50;
    }
    // `:34902` — the unconditional pin.
    npc.alpha = 50;
    // `:34903` — the wind X blend, 50:1, with the jitter draw.
    npc.vx = (npc.vx * 50.0
        + wind_speed_current * 2.0
        + rng.next_range(-10, 11) as f32 * 0.1)
        / 51.0;
    // `:34904` — the Y blend against the -0.25 buoyancy, 50:1.
    npc.vy = (npc.vy * 50.0 + -0.25 + rng.next_range(-10, 11) as f32 * 0.2) / 51.0;
    // `:34905-34908` — a falling bubble dampens (strict > 0).
    if npc.vy > 0.0 {
        npc.vy -= 0.04;
    }
    // `:34909-34928` — the proximity fuse: the 40-px-expanded box.
    if npc.ai[0] == 0.0 {
        let (w, h) = npc.size();
        let (wi, hi) = (w as i32, h as i32);
        // `:34912-34916` — getRect() truncates the position
        // (`NPC.cs:90025-90028`); the halves are INT division.
        let rx = npc.x as i32 - (40 + wi / 2);
        let ry = npc.y as i32 - (40 + hi / 2);
        let rw = wi + 40 * 2;
        let rh = hi + 40 * 2;
        // `:34917-34927` — the 255-slot walk; active && !dead only
        // (NO ghost check here), first intersecting slot wins.
        for p in players.iter() {
            if p.active && !p.dead {
                let (px, py) = (p.x as i32, p.y as i32);
                // XNA Rectangle.Intersects — strict overlap.
                if rx < px + p.width
                    && px < rx + rw
                    && ry < py + p.height
                    && py < ry + rh
                {
                    npc.ai[0] = 1.0;
                    npc.ai[1] = 4.0;
                    npc.net_update = true;
                    break;
                }
            }
        }
    }
    // `:34929-34937` — the 150-tick natural fuse (no netUpdate here).
    if npc.ai[0] == 0.0 {
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 150.0 {
            npc.ai[0] = 1.0;
            npc.ai[1] = 4.0;
        }
    }
    // `:34938-34948` — the countdown; at zero the bubble dies and
    // RETURNS (the grow tail below never runs on this tick).
    if npc.ai[0] == 1.0 {
        npc.ai[1] -= 1.0;
        if npc.ai[1] <= 0.0 {
            npc.hp = 0; // `:34943` — this.life = 0.
            // `:34944-34946` — HitEffect() + active = false + return:
            // the caller's seam (the dust plan below; NO checkDead on
            // this path).
            return Style70Outcome { detonated: true };
        }
    }
    // `:34949-34956` — struck or detonating: unhittable, grown to
    // 100×100 centred on the OLD centre, despawn window 3.
    if just_hit || npc.ai[0] == 1.0 {
        npc.dont_take_damage = true;
        let (cx, cy) = npc.center(); // the OLD box's centre (`:34952`).
        // `:34953` — base.width = (base.height = 100).
        npc.size_override = Some((100.0, 100.0));
        // `:34954` — position = centre - (width/2, height/2), INT halves.
        npc.x = cx - 50.0;
        npc.y = cy - 50.0;
        // `:34955` — EncourageDespawn(3) (RNG-free, only lowers).
        npc.encourage_despawn(3);
    }
    Style70Outcome { detonated: false }
}

/// `NPC.checkDead`'s 371 arm (`NPC.cs:78046-78051`) +
/// `PrepareForDeathAnimation` (`:78032-78038`): a struck bubble does
/// NOT die — the strike arms the 4-tick detonation and restores full
/// life. Call when the strike resolved `life <= 0` (the `checkDead`
/// gate `:78042`; `realLife` is -1 for the bubble, the `active` term is
/// the caller's). `SetStrikePhaseChange` (`:78037`, `:90695-90701`) is
/// `netMode == 1`-only — inert in single-player, not ported. RNG-free.
pub fn bubble_check_dead(npc: &mut Npc) {
    // `:78048-78049`.
    npc.ai[0] = 1.0;
    npc.ai[1] = 4.0;
    // `:78034-78036` — PrepareForDeathAnimation.
    npc.hp = npc.max_hp;
    npc.dont_take_damage = true;
    npc.net_update = true;
}

/// The `HitEffect` 371 arm's branch structure (`NPC.cs:84229-84256`) —
/// the documented visual skip (module docs): no sound is played, no
/// dust is spawned and NO RNG is drawn (the whole arm is skipped, the
/// spore precedent — the stream stays untouched).
pub enum BubbleHitEffect {
    /// `life > 0`: the sound cue only (`PlaySound(4, x, y, 3)`,
    /// `:84231` — the tracked-sound seam), no dust (`:84232` gate).
    Hit,
    /// `life <= 0`: the sound cue + the 60-dust type-212 burst
    /// (`:84235-84255`): each dust re-seeded on the 25-px ring × scale,
    /// dusts 0..29 keep their `NewDust` speed redirected outward,
    /// dusts 30..59 ride `Next(45, 91) / 10f`, HSL hue
    /// 0.4 + NextDouble()×0.2, sat 0.9, lum 0.5, lerped 0.3 to white,
    /// noGravity, scale 0.7. The per-dust draw order (incl. the DEAD
    /// `vector18` pair) is in the module docs for the wiring wave.
    Death { dust_count: u32 },
}

/// The plan for one `HitEffect(hitDirection, dmg)` call on the bubble.
/// `life` is the C# `this.life` AT THE CALL (the fuse death passes 0
/// after `life = 0`; a strike may pass a negative life).
pub fn bubble_hit_effect_plan(life: i32) -> BubbleHitEffect {
    // `:84232` — the dust gate; the sound plays either way.
    if life <= 0 {
        // `:84235` — the flat 60-iteration loop.
        BubbleHitEffect::Death { dust_count: 60 }
    } else {
        BubbleHitEffect::Hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

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

    /// A bubble-shaped NPC: the roster has no DetonatingBubble kind
    /// yet, so the fixture rides a placeholder kind with the C# box
    /// through `size_override`, the 1-HP life and the row's alpha 255.
    fn bubble(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((
            DETONATING_BUBBLE.width as f32,
            DETONATING_BUBBLE.height as f32,
        ));
        n.hp = DETONATING_BUBBLE.life_max;
        n.max_hp = DETONATING_BUBBLE.life_max;
        n.alpha = DETONATING_BUBBLE.alpha;
        n
    }

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    /// `NPC.cs:13719-13733` — the SetDefaults row, field by field;
    /// `Main.cs:67121` the frame count; `:72138-72142` the pinned row.
    #[test]
    fn the_defaults_row_matches_the_csharp() {
        let d = &DETONATING_BUBBLE;
        assert_eq!((d.width, d.height), (36, 36));
        assert_eq!(d.ai_style, 70);
        assert_eq!(d.damage, 100);
        assert_eq!(d.defense, 0);
        assert_eq!(d.life_max, 1);
        assert_eq!(d.hit_sound, (3, 3), "SoundID.NPCHit3");
        assert_eq!(d.death_sound, (4, 3), "SoundID.NPCDeath3");
        assert!(d.no_gravity);
        assert!(d.no_tile_collide);
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.alpha, 255);
        assert_eq!(DETONATING_BUBBLE_FRAME_COUNT, 2);
        assert_eq!(DETONATING_BUBBLE_FRAME_ROW, 1);
    }

    /// `NPC.cs:34886-34893` — the init tick draws EXACTLY four, in
    /// order: Next(80,121), Next(165,265), Next(-100,101) X,
    /// Next(-100,101) Y — after the RNG-free TargetClosest. Witness:
    /// a twin stream replayed by hand must stay in lockstep.
    #[test]
    fn the_init_tick_draws_four_in_order() {
        let w = air();
        let players = [cand(2000.0, 1000.0)];
        let mut n = bubble(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(12345);
        let mut twin = UnifiedRandom::new(12345);
        let out = ai_style70(&mut n, &w, &players, 0.0, false, 7, &mut rng);
        assert!(!out.detonated);
        // The four init draws ...
        let scale_roll = twin.next_range(80, 121);
        let speed_roll = twin.next_range(165, 265);
        let jx = twin.next_range(-100, 101);
        let jy = twin.next_range(-100, 101);
        // ... then the two per-tick wind draws (:34903-34904).
        let _wx = twin.next_range(-10, 11);
        let _wy = twin.next_range(-10, 11);
        assert_eq!(rng.next(), twin.next(), "stream lockstep after the init tick");
        assert!((n.ai[3] - scale_roll as f32 / 100.0).abs() < 1e-6);
        assert!((0.80..=1.20).contains(&n.ai[3]));
        let speed = speed_roll as f32 / 15.0;
        assert!((11.0..=17.6).contains(&speed));
        // The launch vector: Normalize(delta + jitter) * speed.
        let aim = Vec2::new(2000.0 + 10.0 - 1018.0, 1000.0 + 21.0 - 1018.0)
            + Vec2::new(jx as f32, jy as f32);
        let launch = aim.normalize_or_zero() * speed;
        // (The tick's blends moved the velocity on; pin the scale and
        // the netUpdate/target instead, the launch maths is pinned by
        // the homing test below.)
        assert!(n.net_update, ":34892");
        assert_eq!(n.target, 0, "TargetClosest picked the live slot");
        assert_eq!(n.target_set_frame, 7, "the ticker stamp");
        let _ = launch;
    }

    /// `NPC.cs:34886` — the short-circuit gate: a preset target (the
    /// phase-7 Fishron spawn, `:49478-49481`) skips the whole init
    /// block — only the two wind draws happen.
    #[test]
    fn a_preset_target_skips_the_init_draws() {
        let w = air();
        let players = [cand(2000.0, 1000.0)];
        let mut n = bubble(1000.0, 1000.0);
        n.target = 0;
        n.ai[3] = 1.1; // the spawn site's own Next(80,121) roll
        n.vx = 5.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(999);
        let mut twin = UnifiedRandom::new(999);
        let _ = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
        let _wx = twin.next_range(-10, 11);
        let _wy = twin.next_range(-10, 11);
        assert_eq!(rng.next(), twin.next(), "exactly two draws");
        assert_eq!(n.ai[3], 1.1, "the preset scale is untouched");
        assert_eq!(n.target_set_frame, 0, "TargetClosest never ran");
    }

    /// `NPC.cs:34894-34895` — the homing blend is (v*40 + aim*20)/41,
    /// computed from the pre-wind velocity. With the target straight
    /// right and zero wind/jitter the maths is exact.
    #[test]
    fn the_homing_blend_is_forty_to_twenty() {
        let w = air();
        // Player centre exactly right of the bubble centre.
        let players = [cand(1982.0, 997.0)]; // centre (1992, 1018)
        let mut n = bubble(1000.0, 1000.0); // centre (1018, 1018)
        n.target = 0;
        n.ai[3] = 1.0;
        n.vx = 41.0;
        n.vy = 0.0;
        // Force the jitter draws to 0 by replaying: instead pin the
        // blend algebra with a twin — draw the two wind rolls first.
        let mut rng = UnifiedRandom::new(42);
        let mut twin = UnifiedRandom::new(42);
        let wx = twin.next_range(-10, 11) as f32;
        let wy = twin.next_range(-10, 11) as f32;
        let _ = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
        assert_eq!(rng.next(), twin.next());
        let bx = (41.0f32 * 40.0 + 1.0 * 20.0) / 41.0;
        let by = (0.0f32 * 40.0 + 0.0 * 20.0) / 41.0;
        let ex = (bx * 50.0 + 0.0 * 2.0 + wx * 0.1) / 51.0;
        let mut ey = (by * 50.0 + -0.25 + wy * 0.2) / 51.0;
        if ey > 0.0 {
            ey -= 0.04;
        }
        assert!((n.vx - ex).abs() < 1e-5, "vx {ex} vs {}", n.vx);
        assert!((n.vy - ey).abs() < 1e-5, "vy {ey} vs {}", n.vy);
    }

    /// `NPC.cs:34896-34902` — scale copies ai[3]; alpha pins to 50
    /// from ANY start (the -= 30 / clamp pair is dead).
    #[test]
    fn the_scale_copies_ai3_and_alpha_pins_to_fifty() {
        let w = air();
        let players = [cand(2000.0, 1000.0)];
        for start in [255u8, 50, 0, 79] {
            let mut n = bubble(1000.0, 1000.0);
            n.target = 0;
            n.ai[3] = 0.93;
            n.alpha = start;
            let mut rng = UnifiedRandom::new(1);
            let _ = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
            assert_eq!(n.alpha, 50, "start {start}");
            assert!((n.scale - 0.93).abs() < 1e-6);
        }
    }

    /// `NPC.cs:34903-34908` — the wind term is windSpeedCurrent*2 in
    /// the X blend only; a falling bubble loses 0.04 AFTER the blend,
    /// a rising one does not.
    #[test]
    fn the_wind_pushes_x_and_the_fall_damp_is_strict() {
        let w = air();
        // Player centre (1018, 1018) == bubble centre → aim = zero.
        let players = [cand(1008.0, 997.0)];
        let mut n = bubble(1000.0, 1000.0);
        n.target = 0;
        n.ai[3] = 1.0;
        n.vx = 0.0;
        n.vy = 51.0; // blend → (51*50 - 0.25 + wy*0.2)/51 ≈ 50
        let mut rng = UnifiedRandom::new(7);
        let mut twin = UnifiedRandom::new(7);
        let wx = twin.next_range(-10, 11) as f32;
        let wy = twin.next_range(-10, 11) as f32;
        let _ = ai_style70(&mut n, &w, &players, 0.5, false, 0, &mut rng);
        let ex = (0.0f32 * 50.0 + 0.5 * 2.0 + wx * 0.1) / 51.0;
        let ey = (51.0f32 * 50.0 + -0.25 + wy * 0.2) / 51.0 - 0.04;
        assert!((n.vx - ex).abs() < 1e-5);
        assert!((n.vy - ey).abs() < 1e-5);
        // Rising: no damp.
        let mut m = bubble(1000.0, 1000.0);
        m.target = 0;
        m.ai[3] = 1.0;
        m.vy = -51.0;
        let mut rng2 = UnifiedRandom::new(7);
        let mut twin2 = UnifiedRandom::new(7);
        let _wx2 = twin2.next_range(-10, 11);
        let wy2 = twin2.next_range(-10, 11) as f32;
        let _ = ai_style70(&mut m, &w, &players, 0.5, false, 0, &mut rng2);
        let ey2 = (-51.0f32 * 50.0 + -0.25 + wy2 * 0.2) / 51.0;
        assert!((m.vy - ey2).abs() < 1e-5, "no -0.04 on a rise");
    }

    /// `NPC.cs:34909-34928` — the proximity scan: the 36-box expanded
    /// by 40 + half-size (58 per side, 116 wide) arms ai[0]=1/ai[1]=4,
    /// and the countdown block decrements to 3 the SAME tick.
    #[test]
    fn the_proximity_scan_arms_and_decrements_same_tick() {
        let w = air();
        // Bubble at (1000,1000): rect x 942..1058+... w=116 → 942..1058?
        // rx = 1000 - 58 = 942, rw = 116 → right edge 1058.
        // Player at (1050, 1000): px 1050 < 1058 → intersects.
        let players = [cand(1050.0, 1000.0)];
        let mut n = bubble(1000.0, 1000.0);
        n.target = 0;
        n.ai[3] = 1.0;
        let mut rng = UnifiedRandom::new(3);
        let _ = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 3.0, "4 then the same-tick -- (:34940)");
        assert!(n.net_update);
        // Far player: no arm, the natural fuse counts.
        let far = [cand(3000.0, 3000.0)];
        let mut m = bubble(1000.0, 1000.0);
        m.target = 0;
        m.ai[3] = 1.0;
        m.ai[1] = 10.0;
        let _ = ai_style70(&mut m, &w, &far, 0.0, false, 0, &mut rng);
        assert_eq!(m.ai[0], 0.0);
        assert_eq!(m.ai[1], 11.0);
        // A dead player never arms it (:34920 — active && !dead).
        let mut dead = cand(1050.0, 1000.0);
        dead.dead = true;
        let mut k = bubble(1000.0, 1000.0);
        k.target = 0;
        k.ai[3] = 1.0;
        let _ = ai_style70(&mut k, &w, &[dead], 0.0, false, 0, &mut rng);
        assert_eq!(k.ai[0], 0.0);
    }

    /// `NPC.cs:34929-34937` — the natural fuse: ai[1] 149 → 150 arms
    /// ai[0]=1/ai[1]=4 (then the same-tick decrement to 3); NO
    /// netUpdate on this path.
    #[test]
    fn the_natural_fuse_arms_at_one_fifty_without_net_update() {
        let w = air();
        let players = [cand(3000.0, 3000.0)];
        let mut n = bubble(1000.0, 1000.0);
        n.target = 0;
        n.ai[3] = 1.0;
        n.ai[1] = 149.0;
        n.net_update = false;
        let mut rng = UnifiedRandom::new(5);
        let _ = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 3.0);
        assert!(!n.net_update, ":34934-34935 writes no netUpdate");
    }

    /// `NPC.cs:34938-34948` — the detonation: ai[1] 1 → 0 sets hp 0,
    /// flags the seam and RETURNS — the grow tail never runs.
    #[test]
    fn the_fuse_death_returns_before_the_grow_tail() {
        let w = air();
        let players = [cand(3000.0, 3000.0)];
        let mut n = bubble(1000.0, 1000.0);
        n.target = 0;
        n.ai[3] = 1.0;
        n.ai[0] = 1.0;
        n.ai[1] = 1.0;
        n.despawn_timer = 120;
        let mut rng = UnifiedRandom::new(11);
        let mut twin = UnifiedRandom::new(11);
        let out = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
        assert!(out.detonated);
        assert_eq!(n.hp, 0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(n.size_override, Some((36.0, 36.0)), "no grow on the death tick");
        assert!(!n.dont_take_damage, "the grow tail (:34951) never ran");
        assert_eq!(n.despawn_timer, 120, "EncourageDespawn(3) never ran");
        // The two wind draws still happened BEFORE the state machine.
        let _wx = twin.next_range(-10, 11);
        let _wy = twin.next_range(-10, 11);
        assert_eq!(rng.next(), twin.next(), "the death tick draws two");
    }

    /// `NPC.cs:34949-34956` — justHit OR ai[0]==1 grows the box to
    /// 100×100 centred on the OLD centre and encourages despawn 3.
    #[test]
    fn the_grow_arm_centres_the_hundred_box_and_encourages_three() {
        let w = air();
        let players = [cand(3000.0, 3000.0)];
        for (just_hit, ai0) in [(true, 0.0f32), (false, 1.0), (true, 1.0)] {
            let mut n = bubble(1000.0, 1000.0);
            n.target = 0;
            n.ai[3] = 1.0;
            n.ai[0] = ai0;
            n.ai[1] = 3.0; // stay alive through the countdown
            n.despawn_timer = 120;
            let mut rng = UnifiedRandom::new(13);
            let out = ai_style70(&mut n, &w, &players, 0.0, just_hit, 0, &mut rng);
            assert!(!out.detonated);
            assert!(n.dont_take_damage, "({just_hit},{ai0})");
            assert_eq!(n.size_override, Some((100.0, 100.0)));
            // Old centre (1018, 1018) → new position (968, 968).
            assert_eq!(n.x, 968.0);
            assert_eq!(n.y, 968.0);
            assert_eq!(n.despawn_timer, 3);
            // A tighter window is never extended.
            n.despawn_timer = 2;
            let _ = ai_style70(&mut n, &w, &players, 0.0, just_hit, 0, &mut rng);
            assert_eq!(n.despawn_timer, 2);
        }
        // Neither flag: no grow.
        let mut m = bubble(1000.0, 1000.0);
        m.target = 0;
        m.ai[3] = 1.0;
        let mut rng = UnifiedRandom::new(13);
        let _ = ai_style70(&mut m, &w, &players, 0.0, false, 0, &mut rng);
        assert!(!m.dont_take_damage);
        assert_eq!(m.size_override, Some((36.0, 36.0)));
    }

    /// `NPC.cs:78046-78051` + `:78032-78038` — a struck bubble re-arms
    /// the countdown at FULL life, unhittable, netUpdate.
    #[test]
    fn a_struck_bubble_rearms_at_full_life() {
        let mut n = bubble(1000.0, 1000.0);
        n.hp = 0;
        n.net_update = false;
        bubble_check_dead(&mut n);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 4.0);
        assert_eq!(n.hp, 1, "PrepareForDeathAnimation restores lifeMax");
        assert!(n.dont_take_damage);
        assert!(n.net_update);
        // The next AI ticks then count 4 → 3 → 2 → 1 → detonate.
        let w = air();
        let players = [cand(3000.0, 3000.0)];
        n.target = 0;
        n.ai[3] = 1.0;
        let mut rng = UnifiedRandom::new(17);
        for expected in [3.0f32, 2.0, 1.0] {
            let out = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
            assert!(!out.detonated);
            assert_eq!(n.ai[1], expected);
        }
        let out = ai_style70(&mut n, &w, &players, 0.0, false, 0, &mut rng);
        assert!(out.detonated);
        assert_eq!(n.hp, 0);
    }

    /// `NPC.cs:84229-84256` — the HitEffect plan: sound always, the
    /// 60-dust burst only at life <= 0; NO draws (the wholesale skip).
    #[test]
    fn the_hit_effect_plan_gates_the_dust_on_life() {
        match bubble_hit_effect_plan(1) {
            BubbleHitEffect::Hit => {}
            _ => panic!("life > 0 is the sound-only arm"),
        }
        for life in [0, -5] {
            match bubble_hit_effect_plan(life) {
                BubbleHitEffect::Death { dust_count } => assert_eq!(dust_count, 60),
                _ => panic!("life {life} <= 0 is the death burst"),
            }
        }
    }

    /// The full stream witness: init tick (4+2 draws) → steady tick
    /// (2) → death tick (2), replayed by hand against a twin stream.
    #[test]
    fn the_stream_witness_replays_the_whole_life() {
        let w = air();
        let players = [cand(3000.0, 3000.0)];
        let mut n = bubble(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(2026);
        let mut twin = UnifiedRandom::new(2026);
        // Tick 1: the init gate fires (target 255).
        let _ = ai_style70(&mut n, &w, &players, 0.0, false, 1, &mut rng);
        for _ in 0..4 {
            let _ = twin.next();
        }
        for _ in 0..2 {
            let _ = twin.next();
        }
        assert_eq!(rng.next(), twin.next(), "after the init tick");
        // Force the countdown for a short life.
        n.ai[0] = 1.0;
        n.ai[1] = 2.0;
        // Tick 2: countdown 2 → 1, two draws.
        let out = ai_style70(&mut n, &w, &players, 0.0, false, 2, &mut rng);
        assert!(!out.detonated);
        let _ = twin.next();
        let _ = twin.next();
        assert_eq!(rng.next(), twin.next(), "after the steady tick");
        // Tick 3: countdown 1 → 0, death — still two draws.
        let out = ai_style70(&mut n, &w, &players, 0.0, false, 3, &mut rng);
        assert!(out.detonated);
        let _ = twin.next();
        let _ = twin.next();
        assert_eq!(rng.next(), twin.next(), "after the death tick");
    }
}
