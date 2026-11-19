//! Plantera's machine (kind 262) — the `aiStyle == 51` arm of `NPC.AI()`
//! (`NPC.cs:31717-32075`). S1-D-3-plantera: the census found the port's
//! `update_plantera` was a **stand-in** — the EoC hover→charge→decel cycle
//! with invented numbers (the 480-tick charge cycle, the 7.5-px rush, the
//! ±0.1 hover) and NONE of the C#'s hook-anchor movement, seed volleys,
//! phase-2 transition or enrage. This module ports the machine's core as
//! pure functions; the seam is the kind-262 arm in `npc.rs` only (the
//! aiStyle-54 generic arm belongs to the Brain census, sibling W66).
//!
//! Source map (all citations `NPC.cs` unless noted):
//! - `SetDefaults` case 262 (`:12379-12395`): aiStyle **51** (NOT 54 — 54
//!   is the Brain, `:12443`), 86x86, damage 50 / defense 14 / lifeMax
//!   30000, noGravity + noTileCollide, knockBackResist 0, boss.
//! - the hook spawn init `:31736-31742`, the anchor average `:31743-31762`,
//!   the speed/accel bands + enrage `:31763-31789`, the leash `:31790-31831`,
//!   the velocity approach `:31832-31863`, the rotation `:31864-31865`,
//!   phase 1 `:31866-31971` (defense/damage, the localAI[1] cadence, the
//!   seed/poison-seed/thorn-ball volley), phase 2 `:31973-32074` (defense
//!   10 / damage 70, the tentacle batch, the gore trio, the spore cadence).
//!
//! Documented seams (ANALYSIS "The Plantera machine"):
//! - kinds 263/264/265 (hook/tentacle/spore) have no port arms yet — the
//!   spawns ride the request lists (`wants_plantera_hooks` /
//!   `wants_plantera_tentacles` / `wants_plantera_spore`) for the sim; the
//!   hook-average anchor therefore falls back to Plantera's own centre
//!   while no hook exists. (The C# has no first-tick NaN: `NewNPC`
//!   activates the hooks synchronously, `NPC.cs:80767-80771`, before the
//!   centroid scan runs — LEAD-REVIEW G4.)
//! - expert/getGoodWorld branches are constants-off (the port is
//!   single-difficulty classic, `World::MOON_POINT_SCALAR` precedent).
//! - `justHit` (`:31905`, expert-only) has no port field — off with expert.
//! - the gore trio 378/379/380 (`:32028-32030`) consumes its FOUR
//!   `Main.rand.Next` per gore in order (position pair + the NewGore
//!   velocity kicks, `Gore.cs:1327-1328`) but emits nothing (gore draw is
//!   a documented skip, same as the Brain machine S1-D-3e; LEAD-REVIEW G3).
//! - `timeLeft` refresh (`:31727-31734`) and `NPC.plantBoss` (`:31735`)
//!   have no port fields — skipped, recorded in ANALYSIS.

use crate::npc::Npc;
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// `ProjectileID.SeedPlantera` (275, `Projectile.cs:3614-3621`).
const SEED: u16 = 275;
/// `ProjectileID.PoisonSeedPlantera` (276, `Projectile.cs:3622-3629`).
const POISON_SEED: u16 = 276;
/// `ProjectileID.ThornBall` (277, `Projectile.cs:3630-3637`).
const THORN_BALL: u16 = 277;

/// The per-tick numbers the machine derives before the movement
/// (`NPC.cs:31763-31808`): the speed cap `num747`, the acceleration
/// `num748` and the leash radius `num752`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanteraTuning {
    pub speed: f32,
    pub accel: f32,
    pub leash: f32,
}

/// `NPC.cs:31763-31808` — the tuning bands. `life < lifeMax/2` doubles the
/// cap to 5 and the accel to 0.05; `< lifeMax/4` raises the cap to 7 (the
/// accel stays 0.05). The enrage (player out of the jungle band, or dead)
/// adds 8 to the cap and pins the accel at 0.15, and widens the leash by
/// 350. Expert (`(n+1)*1.1`, `+150`) and getGood (`*1.15`) are constants-
/// off — classic only.
pub fn tuning(life: i32, life_max: i32, enraged: bool, target_dead: bool) -> PlanteraTuning {
    let mut speed = 2.5f32; // num747 (`:31763`)
    let mut accel = 0.025f32; // num748 (`:31764`)
    if life < life_max / 2 {
        // `:31765-31769`
        speed = 5.0;
        accel = 0.05;
    }
    if life < life_max / 4 {
        // `:31770-31773`
        speed = 7.0;
    }
    if enraged {
        // `:31774-31779`'s flag55 arm
        speed += 8.0;
        accel = 0.15;
    }
    if target_dead {
        // `:31793-31798`'s flag56 arm (the vector reversal happens in
        // `steer`; the +8 rides here)
        speed += 8.0;
    }
    let mut leash = 500.0f32; // num752 (`:31800`)
    if enraged {
        leash += 350.0; // `:31801-31804`
    }
    PlanteraTuning { speed, accel, leash }
}

/// `NPC.cs:31790-31831` — the two-stage leash. The player offset from the
/// hook anchor is clamped to `leash`; the resulting goal is offset from
/// Plantera's centre and normalized to `speed` — UNLESS the goal is closer
/// than `speed`, in which case the desired velocity is the CURRENT
/// velocity (the hover-in-place quirk, `:31821-31825`). Returns the desired
/// velocity (num749, num750).
pub fn desired_velocity(
    anchor: (f32, f32),
    center: (f32, f32),
    player_center: (f32, f32),
    current_vel: (f32, f32),
    t: PlanteraTuning,
    target_dead: bool,
) -> (f32, f32) {
    let mut dx = player_center.0 - anchor.0; // num749 (`:31791`)
    let mut dy = player_center.1 - anchor.1; // num750 (`:31792`)
    if target_dead {
        // flag56 (`:31793-31796`): flee — reverse both components.
        dy *= -1.0;
        dx *= -1.0;
    }
    // num751 (`:31799`) — `(float)Math.Sqrt` on the f32 product: the
    // double seam (G5).
    let mut len = ((dx * dx + dy * dy) as f64).sqrt() as f32;
    if len >= t.leash {
        // `:31809-31814`
        len = t.leash / len;
        dx *= len;
        dy *= len;
    }
    let goal = (anchor.0 + dx, anchor.1 + dy); // `:31815-31816`
    dx = goal.0 - center.0; // `:31817-31819`
    dy = goal.1 - center.1;
    len = ((dx * dx + dy * dy) as f64).sqrt() as f32; // `:31820`, the f64 seam (G5)
    if len < t.speed {
        // `:31821-31825` — inside the band the desired velocity IS the
        // current one (the approach below then holds still).
        current_vel
    } else {
        // `:31826-31831`
        len = t.speed / len;
        (dx * len, dy * len)
    }
}

/// `NPC.cs:31832-31863` — the per-axis approach: step toward the desired
/// component by `accel`; when the velocity crosses zero against the
/// desired sign, add/subtract `accel * 2` extra (the turn-around boost).
pub fn approach_velocity(vel: (f32, f32), desired: (f32, f32), accel: f32) -> (f32, f32) {
    let (mut vx, mut vy) = vel;
    let (dx, dy) = desired;
    if vx < dx {
        vx += accel;
        if vx < 0.0 && dx > 0.0 {
            vx += accel * 2.0;
        }
    } else if vx > dx {
        vx -= accel;
        if vx > 0.0 && dx < 0.0 {
            vx -= accel * 2.0;
        }
    }
    if vy < dy {
        vy += accel;
        if vy < 0.0 && dy > 0.0 {
            vy += accel * 2.0;
        }
    } else if vy > dy {
        vy -= accel;
        if vy > 0.0 && dy < 0.0 {
            vy -= accel * 2.0;
        }
    }
    (vx, vy)
}

/// `NPC.cs:31864-31865` — the boss-head rotation:
/// `Atan2(x: player.X - center.X, y: player.Y - center.Y) + 1.57f`.
/// `Math.Atan2(y, x)`'s signature means the NAMED arguments bind y = the
/// Y delta and x = the X delta — the result is the STANDARD atan2(dy, dx)
/// (LEAD-REVIEW G1: the original port read the named args positionally and
/// swapped them; settled from the signature). The `(float)` cast rides
/// the f64 atan2 seam (G5).
pub fn rotation(center: (f32, f32), player_center: (f32, f32)) -> f32 {
    let dx = (player_center.0 - center.0) as f64;
    let dy = (player_center.1 - center.1) as f64;
    dy.atan2(dx) as f32 + 1.57
}

/// `NPC.cs:31866-31875` — phase 1 (life > lifeMax/2): defense 36, contact
/// damage 50; the enrage doubles both (72 / 100).
/// `GetAttackDamage_ScaledByDifficulty` (`:7036-7039`) at classic = ×1.
pub fn phase1_stats(enraged: bool) -> (i32, i32) {
    let mut defense = 36;
    let mut damage = 50; // num753
    if enraged {
        defense *= 2;
        damage *= 2;
    }
    (defense, damage)
}

/// `NPC.cs:31973-31980` — phase 2: defense 10, contact damage 70; the
/// enrage quadruples the defense and doubles the damage (40 / 140).
pub fn phase2_stats(enraged: bool) -> (i32, i32) {
    let mut defense = 10;
    let mut damage = 70; // num761
    if enraged {
        defense *= 4;
        damage *= 2;
    }
    (defense, damage)
}

/// `NPC.cs:31880-31917` — phase 1's shot cadence: localAI[1] gains 1 per
/// tick, +1 for each of the 0.9/0.8/0.7/0.6 life thresholds crossed, +3
/// while enraged; when it exceeds 80 it resets to 0 and the volley fires.
/// Returns the new counter and whether the volley fires this tick. Expert
/// (`+1`, `:31901-31904`) and getGood (`+1`, `:31909-31912`) are off.
pub fn phase1_cadence(counter: f32, life: i32, life_max: i32, enraged: bool) -> (f32, bool) {
    let mut c = counter + 1.0;
    // The C# compares in double (`(double)life < (double)lifeMax * 0.9`).
    let life = life as f64;
    let life_max = life_max as f64;
    if life < life_max * 0.9 {
        c += 1.0;
    }
    if life < life_max * 0.8 {
        c += 1.0;
    }
    if life < life_max * 0.7 {
        c += 1.0;
    }
    if life < life_max * 0.6 {
        c += 1.0;
    }
    if enraged {
        c += 3.0;
    }
    if c > 80.0 {
        (0.0, true)
    } else {
        (c, false)
    }
}

/// `NPC.cs:32037-32054` — phase 2's spore cadence: localAI[1] gains 1 per
/// tick, +1 for each of the 0.4/0.3/0.2/0.1 life thresholds crossed; at
/// >= 350 the spore launches and the counter resets. Returns the new
/// counter and whether the spore fires this tick.
pub fn phase2_cadence(counter: f32, life: i32, life_max: i32) -> (f32, bool) {
    let mut c = counter + 1.0;
    let life = life as f64;
    let life_max = life_max as f64;
    if life < life_max * 0.4 {
        c += 1.0;
    }
    if life < life_max * 0.3 {
        c += 1.0;
    }
    if life < life_max * 0.2 {
        c += 1.0;
    }
    if life < life_max * 0.1 {
        c += 1.0;
    }
    if c >= 350.0 {
        (0.0, true)
    } else {
        (c, false)
    }
}

/// `NPC.cs:31774` — the enrage gate: the player is out of the jungle, or
/// above the surface line, or below the underworld layer. The port's
/// `UnderworldLayer` stand-in is the WoF machine's `height - 200`
/// (`npc_ai_style27.rs`'s `style27_step`, the documented convention).
pub fn enraged(world: &World, player: &Player) -> bool {
    let underworld_layer = world.height - 200;
    !player.zones.jungle
        || (player.y as f64) < world.world_surface * 16.0
        || player.y > (underworld_layer * 16) as f32
}

/// One fixed step of Plantera's machine — `NPC.cs:31717-32075`. The caller
/// (`npc.rs`'s kind-262 arm) owns the dispatch; the sim resolves the spawn
/// request lists afterwards. `hook_anchor` is the average centre of the
/// (up to three) live aiStyle-52 hooks (`:31743-31762`); with no ported
/// hook kind the caller passes Plantera's own centre (documented seam).
pub fn step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    hook_anchor: (f32, f32),
) {
    // `:31722-31726` — the dead target sets both flags (flee + enrage).
    let target_dead = player.dead;
    let enrage = enraged(world, player) || target_dead;
    let t = tuning(npc.hp, npc.max_hp, enrage, target_dead);
    let center = npc.center();
    let player_center = player.center();

    // `:31736-31742` — the first tick spawns the three hooks (263).
    if npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = 1.0;
        npc.wants_plantera_hooks = 3;
    }

    // `:31790-31863` — the leash + the approach.
    let desired = desired_velocity(hook_anchor, center, player_center, (npc.vx, npc.vy), t, target_dead);
    let (vx, vy) = approach_velocity((npc.vx, npc.vy), desired, t.accel);
    npc.vx = vx;
    npc.vy = vy;

    // `:31864-31865` - the rotation (named args bind the standard
    // atan2(dy, dx); see `rotation`, LEAD-REVIEW G1).
    npc.rotation = rotation(center, player_center);

    // noTileCollide + noGravity (`SetDefaults :12381`/`:12392`): the raw
    // integration, no collision pass — the stand-in's own tail convention.
    // W1-5: capture the PRE-move position — the C#'s CanHit (`:31918`)
    // runs inside AI(), BEFORE NPC.Update's movement integration; the
    // sight gate below reads this, not the moved box (the aim/muzzle
    // already use the pre-move `center`).
    let (x0, y0) = (npc.x, npc.y);
    npc.x += npc.vx;
    npc.y += npc.vy;

    if npc.hp > npc.max_hp / 2 {
        // ---- phase 1 (`:31866-31971`) ----
        let (defense, damage) = phase1_stats(enrage);
        npc.defense_override = Some(defense);
        npc.damage_override = Some(damage);
        let (c, fire) = phase1_cadence(npc.local_ai[1], npc.hp, npc.max_hp, enrage);
        npc.local_ai[1] = c;
        if !fire {
            return;
        }
        // `:31918-31923` — the sight gate (localAI[3]'s expert justHit
        // override is constants-off). W1-5: the C# checks the PRE-move
        // box (`Collision.CanHit(base.position, …)` inside AI()); the
        // captured (x0, y0) — not the integrated npc.x/y — feeds it.
        let (w, h) = npc.size();
        let visible = crate::projectile::can_hit(
            world,
            x0,
            y0,
            w,
            h,
            player.x,
            player.y,
            crate::player::WIDTH,
            player.height,
        );
        if !visible {
            return;
        }
        // `:31926-31936` — the aim: 15 px/tick (expert 17, off).
        let speed = 15.0f32;
        let mut dx = player_center.0 - center.0;
        let mut dy = player_center.1 - center.1;
        // num757 (`:31934`): `(float)Math.Sqrt` — the f64 seam (G5).
        let k = speed / ((dx * dx + dy * dy) as f64).sqrt() as f32;
        dx *= k;
        dy *= k;
        // `:31937-31957` — the volley pick. RNG order (cited): the
        // `Next(4)` poison-seed roll fires only below 0.8 life; on a miss
        // the `Next(8)` thorn-ball roll fires (expert 2/6, off). A pick
        // pre-charges localAI[1] negative (-30 / -120).
        let mut damage = 22; // num758
        let mut kind = SEED; // num759
        let life = npc.hp as f64;
        let life_max = npc.max_hp as f64;
        if life < life_max * 0.8 && rng.next_max(4) == 0 {
            damage = 27;
            npc.local_ai[1] = -30.0;
            kind = POISON_SEED;
        } else if life < life_max * 0.8 && rng.next_max(8) == 0 {
            damage = 31;
            npc.local_ai[1] = -120.0;
            kind = THORN_BALL;
        }
        if enrage {
            damage *= 2; // `:31958-31961`
        }
        // `GetAttackDamage_ForProjectiles(num758, num758 * 0.9)`
        // (`:31962`, `:7052-7055`): classic remap = the normal value.
        // `:31963-31965` — the muzzle is 3 velocity-lengths out.
        npc.wants_hostile_shot.push((
            kind,
            center.0 + dx * 3.0,
            center.1 + dy * 3.0,
            dx,
            dy,
            damage,
        ));
        // `:31966-31969` — the seeds get timeLeft 300 (the thorn ball
        // keeps its own); the sim's drain applies it by kind.
        return;
    }

    // ---- phase 2 (`:31973-32074`) ----
    let (defense, damage) = phase2_stats(enrage);
    npc.defense_override = Some(defense);
    npc.damage_override = Some(damage);

    // `:31983-32009` — the one-shot tentacle batch: 8 (getGood +6, off).
    // The expert per-hook extra batches and the expert respawn roll
    // (`:31995-32024`) are constants-off.
    if npc.local_ai[0] == 1.0 {
        npc.local_ai[0] = 2.0;
        npc.wants_plantera_tentacles = 8;
    }

    // `:32026-32032` — the gore trio 378/379/380: each gore draws
    // `Next(width)` then `Next(height)` for the position (`Main.rand`
    // order preserved), then `Gore.NewGore` itself draws the velocity
    // kicks `Next(10, 31)` (Y lift) and `Next(-20, 21)` (X kick)
    // (`Gore.cs:1327-1328`) — FOUR draws per gore. The emit is a
    // documented skip but the draws are consumed (stream-neutral rule,
    // LEAD-REVIEW G3: the original port drew only the 6 position values).
    if npc.local_ai[2] == 0.0 {
        let (w, h) = npc.size();
        for _ in 0..3 {
            let _gx = rng.next_max(w as i32);
            let _gy = rng.next_max(h as i32);
            let _kick_y = rng.next_range(10, 31);
            let _kick_x = rng.next_range(-20, 21);
        }
        npc.local_ai[2] = 1.0;
    }

    // `:32037-32074` — the spore cadence.
    let (c, fire) = phase2_cadence(npc.local_ai[1], npc.hp, npc.max_hp);
    npc.local_ai[1] = c;
    if !fire {
        return;
    }
    // `:32056-32071` — the spore (265) launch: 8 px/tick toward the player
    // with a ±10 jitter per axis (`Next(-10, 11)`, X first); the vertical
    // lift `|dx * 0.2|` (num773) is ZEROED when the player is below
    // (`num774 > 0`, `:32061-32064`) and subtracted otherwise — an upward
    // lead only when the target is at or above the boss (LEAD-REVIEW G2:
    // the original port inverted the gate and subtracted when dy > 0).
    let speed = 8.0f32;
    let mut dx = player_center.0 - center.0 + rng.next_range(-10, 11) as f32;
    let mut lead = (dx * 0.2).abs();
    let mut dy = player_center.1 - center.1 + rng.next_range(-10, 11) as f32;
    if dy > 0.0 {
        lead = 0.0;
    }
    dy -= lead;
    // `(float)Math.Sqrt` on the f32 product (`:32066`) — the double seam (G5).
    let k = speed / ((dx * dx + dy * dy) as f64).sqrt() as f32;
    dx *= k;
    dy *= k;
    npc.wants_plantera_spore = Some((dx, dy));
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::World;

    fn world() -> World {
        let mut w = World::new("plantera-test".into(), 400, 400);
        // The surface line at row 100: a player at row 150 is below it
        // (the enrage gate's `position.Y < worldSurface * 16` stays off).
        w.world_surface = 100.0;
        w
    }

    fn jungle_player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 120.0 * 16.0;
        p.y = 150.0 * 16.0;
        p.zones.jungle = true;
        p
    }

    /// `NPC.cs:31763-31773` — the speed/accel bands by life fraction.
    #[test]
    fn s1_d_3_plantera_the_tuning_bands_follow_the_life_fractions() {
        let full = tuning(30000, 30000, false, false);
        assert_eq!((full.speed, full.accel, full.leash), (2.5, 0.025, 500.0));
        let half = tuning(14999, 30000, false, false);
        assert_eq!((half.speed, half.accel), (5.0, 0.05));
        let quarter = tuning(7499, 30000, false, false);
        assert_eq!((quarter.speed, quarter.accel), (7.0, 0.05));
        // The exact-half boundary: `life < lifeMax/2` is strict.
        let at_half = tuning(15000, 30000, false, false);
        assert_eq!(at_half.speed, 2.5);
    }

    /// `NPC.cs:31774-31779`, `:31793-31804` — the enrage and dead-target
    /// bands stack (+8 each), the accel pins at 0.15, the leash +350.
    #[test]
    fn s1_d_3_plantera_the_enrage_stacks_the_bands() {
        let e = tuning(30000, 30000, true, false);
        assert_eq!((e.speed, e.accel, e.leash), (10.5, 0.15, 850.0));
        let d = tuning(30000, 30000, true, true);
        assert_eq!(d.speed, 18.5);
        let e2 = tuning(7499, 30000, true, false);
        assert_eq!((e2.speed, e2.accel), (15.0, 0.15));
    }

    /// `NPC.cs:31866-31875` / `:31973-31980` — the phase stats.
    #[test]
    fn s1_d_3_plantera_the_phase_stats_match_the_csharp() {
        assert_eq!(phase1_stats(false), (36, 50));
        assert_eq!(phase1_stats(true), (72, 100));
        assert_eq!(phase2_stats(false), (10, 70));
        assert_eq!(phase2_stats(true), (40, 140));
    }

    /// `NPC.cs:31821-31825` — inside the speed band the desired velocity
    /// IS the current one (hover-in-place).
    #[test]
    fn s1_d_3_plantera_inside_the_band_the_desired_is_the_current_velocity() {
        let t = tuning(30000, 30000, false, false);
        // anchor == centre, player 1 px away: the goal sits inside 2.5.
        let v = desired_velocity((100.0, 100.0), (100.0, 100.0), (101.0, 100.0), (0.3, -0.2), t, false);
        assert_eq!(v, (0.3, -0.2));
        // Far away: normalized to the cap.
        let v = desired_velocity((100.0, 100.0), (100.0, 100.0), (400.0, 100.0), (0.0, 0.0), t, false);
        assert!((v.0 - 2.5).abs() < 1e-4 && v.1.abs() < 1e-4);
    }

    /// `NPC.cs:31809-31814` — the leash clamps the player offset to 500
    /// (850 enraged) before the goal is formed.
    #[test]
    fn s1_d_3_plantera_the_leash_clamps_the_player_offset() {
        let t = tuning(30000, 30000, false, false);
        // Player 2000 px right of the anchor, Plantera at the anchor:
        // the goal is anchor + 500 → the desired velocity points right at
        // the cap, not at the raw player.
        let v = desired_velocity((0.0, 0.0), (0.0, 0.0), (2000.0, 0.0), (0.0, 0.0), t, false);
        assert!((v.0 - 2.5).abs() < 1e-4);
        // Dead target reverses the offset (`:31793-31796`).
        let v = desired_velocity((0.0, 0.0), (0.0, 0.0), (2000.0, 0.0), (0.0, 0.0), t, true);
        assert!((v.0 + 2.5).abs() < 1e-4, "the flee reverses: {v:?}");
    }

    /// `NPC.cs:31832-31847` — the approach steps by accel and adds the
    /// double accel when crossing zero against the desired sign.
    #[test]
    fn s1_d_3_plantera_the_turnaround_boost_triples_the_step() {
        let (vx, _) = approach_velocity((-0.01, 0.0), (2.5, 0.0), 0.025);
        // -0.01 + 0.025 = 0.015 > 0 → no boost this tick.
        assert!((vx - 0.015).abs() < 1e-6);
        let (vx, _) = approach_velocity((-0.05, 0.0), (2.5, 0.0), 0.025);
        // -0.05 + 0.025 = -0.025 < 0 while desired > 0 → +0.05 extra.
        assert!((vx - 0.025).abs() < 1e-6);
        let (vx, _) = approach_velocity((0.05, 0.0), (-2.5, 0.0), 0.025);
        assert!((vx + 0.025).abs() < 1e-6);
    }

    /// `NPC.cs:31865` — `Atan2(x: dx, y: dy)` binds y=dy, x=dx
    /// (`Math.Atan2(y, x)`), so the rotation is the STANDARD
    /// atan2(dy, dx) + 1.57f (LEAD-REVIEW G1: the witness originally
    /// pinned the swapped reading).
    #[test]
    fn s1_d_3_plantera_the_rotation_is_the_standard_atan2_plus_1_57() {
        // Player directly right: atan2(dy=0, dx=100) = 0 → 1.57.
        let r = rotation((0.0, 0.0), (100.0, 0.0));
        assert!((r - 1.57).abs() < 1e-4);
        // Player directly below: atan2(100, 0) = π/2 → π/2 + 1.57.
        let r = rotation((0.0, 0.0), (0.0, 100.0));
        assert!((r - (std::f32::consts::FRAC_PI_2 + 1.57)).abs() < 1e-4);
    }

    /// `NPC.cs:31880-31917` — the phase-1 cadence: 80 ticks at full life,
    /// the thresholds accelerate it, the enrage adds 3.
    #[test]
    fn s1_d_3_plantera_the_phase1_cadence_counts_the_thresholds() {
        // The gate is `!(localAI[1] > 80)` — 80 holds, 81 fires.
        let (c, fire) = phase1_cadence(80.0, 30000, 30000, false);
        assert_eq!((c, fire), (0.0, true), "81 > 80 fires");
        let (c, fire) = phase1_cadence(79.0, 30000, 30000, false);
        assert_eq!((c, fire), (80.0, false));
        // Below every threshold: +5 per tick (+3 enraged = +8).
        let (c, _) = phase1_cadence(0.0, 10000, 30000, false);
        assert_eq!(c, 5.0);
        let (c, _) = phase1_cadence(0.0, 10000, 30000, true);
        assert_eq!(c, 8.0);
    }

    /// `NPC.cs:32037-32054` — the phase-2 cadence fires at >= 350 with
    /// the 0.4/0.3/0.2/0.1 thresholds.
    #[test]
    fn s1_d_3_plantera_the_phase2_cadence_fires_at_350() {
        let (c, fire) = phase2_cadence(349.0, 15000, 30000);
        assert_eq!((c, fire), (0.0, true));
        let (c, _) = phase2_cadence(0.0, 1000, 30000);
        assert_eq!(c, 5.0, "all four thresholds crossed");
    }

    /// The seam: one step of the machine on a fresh Plantera — the hook
    /// init latch, the phase-1 stats, the raw integration.
    #[test]
    fn s1_d_3_plantera_the_first_tick_latches_the_hooks_and_the_stats() {
        let w = world();
        let mut pl = Npc::new(NpcKind::Plantera, 100.0 * 16.0, 150.0 * 16.0, 1, 0.0);
        let player = jungle_player(&w);
        let mut rng = UnifiedRandom::new(0);
        let center = pl.center();
        step(&mut pl, &w, &player, &mut rng, center);
        assert_eq!(pl.local_ai[0], 1.0, "the hook init latch");
        assert_eq!(pl.wants_plantera_hooks, 3);
        assert_eq!(pl.defense_override, Some(36), "phase 1, no enrage");
        assert_eq!(pl.damage_override, Some(50));
        assert_eq!(pl.local_ai[1], 1.0, "the cadence ticked once");
    }

    /// `NPC.cs:31983-31994` — crossing the half life arms the tentacle
    /// batch exactly once (localAI[0] 1 → 2).
    #[test]
    fn s1_d_3_plantera_the_phase2_batch_is_one_shot() {
        let w = world();
        let mut pl = Npc::new(NpcKind::Plantera, 100.0 * 16.0, 150.0 * 16.0, 1, 0.0);
        pl.local_ai[0] = 1.0; // the hooks already spawned
        pl.hp = pl.max_hp / 2; // the phase gate is `> lifeMax/2`
        let player = jungle_player(&w);
        let mut rng = UnifiedRandom::new(0);
        let center = pl.center();
        step(&mut pl, &w, &player, &mut rng, center);
        assert_eq!(pl.wants_plantera_tentacles, 8);
        assert_eq!(pl.local_ai[0], 2.0);
        assert_eq!(pl.local_ai[2], 1.0, "the gore latch fired");
        pl.wants_plantera_tentacles = 0;
        let center = pl.center();
        step(&mut pl, &w, &player, &mut rng, center);
        assert_eq!(pl.wants_plantera_tentacles, 0, "no second batch");
        assert_eq!(pl.defense_override, Some(10));
        assert_eq!(pl.damage_override, Some(70));
    }

    /// `NPC.cs:32061-32064` — the spore's lead is ZEROED when the player
    /// is below (dy > 0) and subtracted otherwise (LEAD-REVIEW G2). The
    /// jitter draws make the exact vectors rng-dependent, so the witness
    /// mirrors the C# arithmetic on the SAME stream and pins equality
    /// plus the gate's direction.
    #[test]
    fn s1_d_3_plantera_the_spore_lead_zeroes_when_the_player_is_below() {
        for (px, py) in [(120.0f32, 170.0f32), (120.0, 130.0), (100.0, 150.0)] {
            let w = world();
            let mut pl = Npc::new(NpcKind::Plantera, 100.0 * 16.0, 150.0 * 16.0, 1, 0.0);
            pl.local_ai[0] = 2.0;
            pl.local_ai[2] = 1.0; // gore latch already fired: no gore draws
            pl.hp = pl.max_hp / 2;
            pl.local_ai[1] = 349.0; // the cadence fires this tick
            let mut player = jungle_player(&w);
            player.x = px * 16.0;
            player.y = py * 16.0;
            let mut rng = UnifiedRandom::new(0);
            let center = pl.center();
            step(&mut pl, &w, &player, &mut rng, center);
            let (vx, vy) = pl.wants_plantera_spore.expect("the spore fired");
            // The mirrored C# arithmetic on a twin stream.
            let mut twin = UnifiedRandom::new(0);
            let pcx = player.x + crate::player::WIDTH * 0.5;
            let pcy = player.y + player.height * 0.5;
            let mut dx = pcx - center.0 + twin.next_range(-10, 11) as f32;
            let mut lead = (dx * 0.2).abs();
            let mut dy = pcy - center.1 + twin.next_range(-10, 11) as f32;
            let below = dy > 0.0;
            if below {
                lead = 0.0;
            }
            dy -= lead;
            let k = 8.0 / ((dx * dx + dy * dy) as f64).sqrt() as f32;
            dx *= k;
            dy *= k;
            assert_eq!((vx, vy), (dx, dy), "the launch mirrors the C#");
            if below {
                // No lead: the direction is the raw jittered delta.
                let raw_dy = pcy - center.1;
                assert!(raw_dy > 10.0, "the witness player is well below");
                assert!(vy > 0.0, "aimed down at the player, no upward lead");
            }
        }
    }

    /// W1-5 (`NPC.cs:31918`): the sight gate reads the PRE-move box —
    /// `Collision.CanHit(base.position, …)` runs inside AI(), BEFORE
    /// NPC.Update's integration. The port integrates mid-step, so the
    /// gate must use the captured (x0, y0). Witness: a one-tile wall
    /// column at x 105; the PRE-move centre sits in column 104 (the
    /// walk's first step onto the solid column rejects — blocked), the
    /// tick's ~15 px motion carries the POST-move centre INTO column
    /// 105, from which `can_hit`'s walk exits through air (clear). The
    /// old post-move read fires the seed; the pre-move read holds it.
    #[test]
    fn w1_5_the_sight_gate_reads_the_pre_move_position() {
        let mut w = world();
        // The wall column at tile x = 105, rows 140..160 (full solids).
        for y in 140..160 {
            let i = w.index(105, y);
            w.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut pl = Npc::new(NpcKind::Plantera, 0.0, 150.0 * 16.0, 1, 0.0);
        // The pre-move centre at px 1679 = tile column 104, one px left
        // of the 104/105 boundary (1680). Plantera's box is 86 wide.
        pl.x = 1679.0 - 43.0;
        pl.local_ai[0] = 1.0; // the hook latch already fired
        pl.local_ai[1] = 80.0; // the >80 cadence fires THIS tick
        // A carried +15 vx: the approach steps it toward the 2.5 cap by
        // the 0.025 accel — still ~14.975 px of motion this tick, enough
        // to cross into column 105 (post-move centre ~1694).
        pl.vx = 15.0;
        let mut player = jungle_player(&w);
        player.x = 120.0 * 16.0;
        player.y = 150.0 * 16.0;
        let mut rng = UnifiedRandom::new(0);
        let center = pl.center();
        let (x0, y0) = (pl.x, pl.y);
        step(&mut pl, &w, &player, &mut rng, center);
        assert!(pl.x + 43.0 >= 1680.0, "the motion crossed into column 105");
        assert!((pl.y - y0).abs() < 0.1, "the vertical motion is one accel step");
        assert!(
            pl.wants_hostile_shot.is_empty(),
            "the PRE-move box is wall-blocked: no seed (a post-move read fires — the W1-5 bug)"
        );
        assert_eq!(pl.local_ai[1], 0.0, "the cadence reset ran before the gate");
        // The control: the same scene without the wall fires the seed —
        // proving the block above is the sight gate, not the cadence.
        for y in 140..160 {
            let i = w.index(105, y);
            w.tiles[i] = tr_core::Tile::default();
        }
        let mut pl2 = Npc::new(NpcKind::Plantera, x0, y0, 1, 0.0);
        pl2.local_ai[0] = 1.0;
        pl2.local_ai[1] = 80.0;
        pl2.vx = 15.0;
        let mut rng2 = UnifiedRandom::new(0);
        step(&mut pl2, &w, &player, &mut rng2, pl2.center());
        assert_eq!(pl2.wants_hostile_shot.len(), 1, "the clear line fires");
    }

    /// `NPC.cs:32026-32032` + `Gore.cs:1327-1328` — the gore burst
    /// consumes TWELVE Main.rand values (4 per gore: the position pair
    /// then the NewGore velocity kicks), not 6 (LEAD-REVIEW G3).
    #[test]
    fn s1_d_3_plantera_the_gore_burst_consumes_twelve_draws() {
        let w = world();
        let mut pl = Npc::new(NpcKind::Plantera, 100.0 * 16.0, 150.0 * 16.0, 1, 0.0);
        pl.local_ai[0] = 2.0;
        pl.hp = pl.max_hp / 2;
        pl.local_ai[1] = 349.0; // the spore fires after the burst
        let player = jungle_player(&w);
        let mut rng = UnifiedRandom::new(0);
        let center = pl.center();
        step(&mut pl, &w, &player, &mut rng, center);
        // Mirror: 3 x (Next(w), Next(h), Next(10,31), Next(-20,21)) then
        // the spore's X/Y jitter — the spore vector pins the stream state.
        let mut twin = UnifiedRandom::new(0);
        let (gw, gh) = pl.size();
        for _ in 0..3 {
            let _ = twin.next_max(gw as i32);
            let _ = twin.next_max(gh as i32);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        let pcx = player.x + crate::player::WIDTH * 0.5;
        let pcy = player.y + player.height * 0.5;
        let mut dx = pcx - center.0 + twin.next_range(-10, 11) as f32;
        let mut lead = (dx * 0.2).abs();
        let mut dy = pcy - center.1 + twin.next_range(-10, 11) as f32;
        if dy > 0.0 {
            lead = 0.0;
        }
        dy -= lead;
        let k = 8.0 / ((dx * dx + dy * dy) as f64).sqrt() as f32;
        dx *= k;
        dy *= k;
        assert_eq!(pl.wants_plantera_spore, Some((dx, dy)), "stream advanced by 12+2");
    }
}
