//! S1-D-4 unarmed styles — the four data-rowed projectile styles with NO
//! dispatch arm (census §3c, `docs/research/50-entity-ai-census.md` on
//! `origin/docs/entity-ai-census`): **31** (PureSpray 145), **147**
//! (Celebration Mk2 rockets 715-718), **181** (FairyQueenRangedItemShot 932)
//! and **193** (Flamethrower Flames 85). 194 (R56) and 120 (R62) are other
//! workers' claims and stay out of this module.
//!
//! C# sources read WHOLE (the golden rule):
//! - aiStyle 31: the inline `AI()` arm `Projectile.cs:26786-26886` (the
//!   spray family's per-kind dust/conversion ladder, the owner-gated
//!   `WorldGen.Convert`, the 133 timeLeft cap, the ai[0] dust-scale ladder,
//!   the 0.3·direction spin). The tile conversion is
//!   `tr_core::spread::convert_cleanse` (`WorldGen.cs:55067-55083` +
//!   the type-0 cell `:55527-55664`).
//! - aiStyle 147: `AI_147_Celeb2Rocket` `:52881-53092`, `AI_147_Explosion`
//!   `:53130-53399`, `CreateGroundExplosion` `:53401-53464`, the `Kill()`
//!   resize-and-Damage arm `:73632-73645`, `GetCeleb2Color` `:52550-52563`.
//! - aiStyle 181: `AI_181_FairyQueenRangedItemShot` `:43178-43193`.
//! - aiStyle 193: `AI_193_Flamethrower` `:41337-41401`.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the dust/gore BODIES are skipped (no draw layer wired here) and
//! the `Dust.NewDust*` internal rolls go with the dust pool (ANALYSIS
//! "Dust & gore", the `ai_style2` precedent). The `Gore.NewGore` internal
//! kicks DO stay — `Next(10, 31)` on Y first, then `Next(-20, 21)` on X
//! (`Gore.cs:1327-1328`, the LEAD-REVIEW G3 standard, ANALYSIS "Gore: the
//! pool"). Per style:
//! - 31: ZERO draws — the arm's only rand-free state is the conversion,
//!   the cap, the counter and the spin; the dust ladder rolls nothing.
//! - 181: `Next(6)` every tick (`:43183`); on a hit one `NextFloat` for the
//!   dust colour's `lerpToWhite` argument (`:43189`). The colour helper
//!   itself is rand-free for non-dev names (`:21665-22041` — only the
//!   "Random" name draws, unported).
//! - 193: one `NextFloat` gate while `localAI[0] < num3` (`:41372`); the
//!   taken branch's `NextVector2Circular` pair (`:41374`, two floats —
//!   `Utils.cs:1266-1269`) + the `Next(4)` noGravity pick (`:41375`); one
//!   `NextFloat` gate while `localAI[0] >= num2` (`:41391`); the taken
//!   branch's `RotatedByRandom` pair (`:41394`), the `NextVector2Circular`
//!   pair (`:41395`) and the scale/customData floats (`:41398-41399`).
//!   Only the `NewDust*` INTERNAL rolls go with the dust pool.
//! - 147: the rocket AI itself draws only the state-1 ring's per-particle
//!   colour `NextFloat` (`:52940`); the explosion censuses are in
//!   [`celeb2_explosion`]'s doc.
//!
//! Movement is the dispatch's `kill_step` (the `HandleMovement` default
//! arm's destination-box probe stand-in, `Projectile.cs:19849-19855`), run
//! AFTER the AI exactly as the C# runs `HandleMovement` after `AI()`
//! (`:17015`). The 147 state-4 swirl moves the position INSIDE the AI
//! (`:53059`) and the movement step adds the velocity on top — faithful.
//!
//! `base.direction` (the 31 spin, `:26885`) uses the port's velocity-sign
//! stand-in — the C# updates `direction` from `velocity.X`'s sign after
//! each sub-step's movement (`:17016-17026` + `AutomaticallyChangesDirection`
//! `:17281-17288`, true for aiStyle 31), the `projectile_ai001_a` precedent.
//!
//! Code-only round (S1-D-4): the witness tests below are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, can_hit, chaseable, chaseable_held, get_lerp_value, rotate, safe_normalize,
    BUBBLE, DRILL_MOUNT_CROSSHAIR, FLAIRON_BUBBLE, SCUTLIX_LASER_CROSSHAIR,
    SCUTLIX_LASER_FRIENDLY,
};

/// `AI()`'s aiStyle-31 arm (`Projectile.cs:26786-26886`) — the Clentaminator
/// spray. Only the PureSpray 145 has a `proj_data` row (the 146-149 /
/// 1015-1017 solutions are unrowed); the per-kind ladder is ported whole so
/// the later rows land on a faithful arm.
///
/// The dust-scale ladder (`:26845-26879`: the 7/3 threshold, the
/// 0.2/0.4/0.6/0.8 fade steps, the flag13 ×1.2 and the `(int)(12·scale)`
/// box growth) feeds ONLY the skipped `Dust.NewDust` — it rolls nothing and
/// writes no slot state, so the port keeps just the `ai[0]++` both branches
/// share. ZERO `Main.rand` draws by construction.
pub(crate) fn ai_spray(p: &mut Projectile, world: &mut World, local_owner: u8) {
    // `:26788` — the solution flag (`ai[1] == 1` = the wider spray).
    let solution = p.ai[1] == 1.0;
    // `:26789-26825` — the per-kind (dust type, conversion type) ladder.
    // The dust type feeds only the skipped dust; the conversion type is
    // the `WorldGen.Convert` argument.
    let conversion = match p.kind {
        147 => 1,
        146 => 2,
        148 => 3,
        149 => 4,
        1015 => 5,
        1016 => 6,
        1017 => 7,
        _ => 0,
    };
    // `:26826-26835` — the owner-gated conversion: `Center.ToTileCoordinates`
    // (the `(int)(X/16)` truncation), size 2 (3 for the solution). Only the
    // type-0 cleanse cell is ported (`spread::convert_cleanse`); the other
    // conversion types' cells are the unrowed kinds' follow-up.
    if p.owner == local_owner && conversion == 0 {
        let tx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
        let ty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
        let size = if solution { 3 } else { 2 };
        tr_core::spread::convert_cleanse(world, tx, ty, size);
    }
    // `:26836-26839` — the 133-tick life cap (per sub-step).
    if p.time_left > 133 {
        p.time_left = 133;
    }
    // `:26840-26884` — the counter advances in BOTH branches; the >threshold
    // branch's scale ladder and single dust are dust-only (module doc).
    p.ai[0] += 1.0;
    // `:26885` — the spin. `direction` is the velocity-sign stand-in
    // (module doc): `velocity.X < 0 → -1, else 1` (`:17018-17025`).
    let direction = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
    p.rot += 0.3 * direction;
}

/// `AI_193_Flamethrower` (`Projectile.cs:41337-41401`) — the Flamethrower's
/// Flames (85). The 1106 Foxparks variant keeps its constants arm; its
/// sound gate (`:41366-41370`) is skipped (no sound layer, no `soundDelay`
/// slot field) and 1106 has no row yet.
///
/// The `Kill()` at 72 (`:41341-41344`) has NO return in the C# — the arm
/// runs on into the velocity decay and the dust gates; the port keeps that
/// shape. `HandleMovement` then runs before the `!active` return
/// (`:17015`/`:17027`), so the dispatch's `kill_step` integrates the dead
/// slot's last step too (a nothing-reads write, faithful to the C#).
pub(crate) fn ai_flamethrower(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:41339-41348` — the age counter, the 72-tick kill, the 60-tick decay.
    p.local_ai[0] += 1.0;
    let num = 60.0f32;
    if p.local_ai[0] >= num + 12.0 {
        p.active = false;
        p.time_left = 0;
    }
    if p.local_ai[0] >= num {
        p.vel *= 0.95;
    }
    // `:41349-41356` — the Foxparks flag flips the smoke window off and the
    // flame window to the full 60.
    let foxparks = p.ai[0] == 1.0;
    let (mut num2, mut num3) = (50.0f32, 50.0f32);
    if foxparks {
        num2 = 0.0;
        num3 = num;
    }
    // `:41357-41371` — the per-kind dust constants (the 1106 sound gate is
    // the documented skip).
    let mut chance_flame = 0.25f32;
    if p.kind == 1106 {
        chance_flame *= 0.5;
    }
    let chance_smoke = 0.5f32;
    // `:41372-41390` — the flame dust gate: the `NextFloat` draw stays; on
    // the taken branch the position's `NextVector2Circular(num4, num4)`
    // pair (`Utils.cs:1266-1269`: NextVector2Unit's float + the radius
    // float) and the `Next(4)` noGravity pick stay too — explicit
    // call-site draws (the nebula precedent). The `NewDustDirect` body
    // and its internal rolls go with the dust layer.
    if p.local_ai[0] < num3 && rng.next_float() < chance_flame {
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular
        let _ = rng.next_range(0, 4); // the noGravity pick (`:41375`)
    }
    // `:41391-41400` — the smoke dust gate: the `NextFloat` draw stays; on
    // the taken branch the direction's `RotatedByRandom(0.19634954631328583)`
    // pair (`:41394`, `Utils.cs:1697-1700`), the position's
    // `NextVector2Circular(num5, num5)` pair (`:41395`) and the
    // scale/customData floats (`:41398-41399`) stay — the dust body is the
    // skip. The owner-centre read feeds only the skipped dust velocity.
    if num2 > 0.0 && p.local_ai[0] >= num2 && rng.next_float() < chance_smoke {
        let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular
        let _ = rng.next_float(); // the scale wobble (`:41398`)
        let _ = rng.next_float(); // customData (`:41399`)
    }
}

/// `AI_181_FairyQueenRangedItemShot` (`Projectile.cs:43178-43193`) — the
/// Empress drop-shot 932. The counter, the 10-tick alpha ramp and the
/// velocity rotation are the whole state; the 1-in-6 dust gate's `Next(6)`
/// stays on the stream and the taken branch's colour `NextFloat`
/// (`GetFairyQueenWeaponsColor(1f, NextFloat() * 0.4f)`, `:43189`) with it.
/// The dust body (`NewDustPerfect` 267, fadeIn/noGravity/alpha/scale) and
/// the colour computation itself are the dust layer's skip — the colour
/// helper draws nothing for non-dev names (`:21665-22041`).
pub(crate) fn ai_fairy_queen_shot(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:43180` — the age counter.
    p.ai[0] += 1.0;
    // `:43181` — `alpha = (int)MathHelper.Lerp(255, 0,
    // GetLerpValue(0, 10, ai[0], clamped))`: XNA's `a + (b - a) * t` with
    // the truncating cast.
    let t = get_lerp_value(0.0, 10.0, p.ai[0]);
    p.alpha = (255.0f32 + (0.0f32 - 255.0f32) * t) as i32;
    // `:43182` — `rotation = velocity.ToRotation()`.
    p.rot = p.vel.y.atan2(p.vel.x);
    // `:43183-43192` — the 1-in-6 dust gate.
    if rng.next_range(0, 6) == 0 {
        // `:43189` — the colour's lerpToWhite argument draw.
        let _ = rng.next_float();
        // The dust spawn — SKIPPED (module doc).
    }
}

/// `AI_147_Celeb2Rocket` (`Projectile.cs:52881-53092`) — the Celebration
/// Mk2 rockets 715-718. The seven `ai[0]` states: 0 the launch fall (the
/// 20-tick arm + the every-20-tick firework burst), 1 the pop (the −10
/// kick, the 0.25 fall, the 2.5 scale cap, the 10-tick dust ring), 2 the
/// 60-tick fall, 3 the homing (the speed latch, the 800-Manhattan acquire
/// + the 1000-Manhattan hold, the (v·7+d)/8 blend), 4 the swirl (the
/// ai[1]==1 +45 skip, the π/45 phase, the perpendicular 3-px step), 5/6
/// the 40/30-tick falls. Every state clamps `velocity.Y` at 16.
///
/// RNG: the state-1 ring's per-particle colour `NextFloat` (`:52940`) is
/// the AI's only draw; the state-0 burst rides [`celeb2_explosion`].
/// `Lighting.AddLight` (`:52890`) and `GetCeleb2Color` (`:52550-52563`,
/// the pure ai[0] colour switch) feed light/dust only — skipped.
pub(crate) fn ai_celeb2_rocket(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
) {
    // `:52886-52888` — the age counter, the alpha pin, the rotation
    // (`velocity.ToRotation() + π/2`).
    p.local_ai[0] += 1.0;
    p.alpha = 0;
    p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    let state = p.ai[0] as i32;
    match state {
        0 => {
            // `:52894-52902` — the 20-tick arm: the fall and the infinite
            // penetration (`penetrate = (maxPenetrate = -1)`).
            if p.local_ai[0] >= 20.0 {
                p.vel.y += 0.12;
                p.max_penetrate = -1;
                p.penetrate = -1;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
            // `:52903-52906` — the every-20-tick firework burst (strictly
            // past 20).
            if p.local_ai[0] > 20.0 && p.local_ai[0] % 20.0 == 0.0 {
                celeb2_explosion(p, world, rng);
            }
        }
        1 => {
            // `:52910-52921` — the tick-10 kick, the 0.25 fall, the clamp.
            if p.local_ai[0] == 10.0 {
                p.vel.y -= 10.0;
            }
            if p.local_ai[0] >= 10.0 {
                p.vel.y += 0.25;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
            if p.local_ai[0] >= 10.0 {
                // `:52926-52930` — the scale climb, capped at 2.5.
                p.scale += 0.015;
                if p.scale > 2.5 {
                    p.scale = 2.5;
                }
                // `:52931-52951` — the 10-tick dust ring: the f32
                // accumulation loop (`num5 += 0.12f`) is the C#'s own
                // iteration count; each particle rolls one colour
                // `NextFloat` (`:52940`), the dust body is skipped.
                if p.local_ai[0] % 10.0 == 0.0 {
                    let mut p5 = 0.0f32;
                    while p5 < 1.0 {
                        let _ = rng.next_float();
                        p5 += 0.12;
                    }
                }
            }
        }
        2 => {
            // `:52955-52962` — the 60-tick fall.
            if p.local_ai[0] >= 60.0 {
                p.vel.y += 0.15;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        3 => {
            // `:52966-52972` — the speed latch: `localAI[1]` holds the
            // first-seen `velocity.Length()` (the f32 sqrt seam, the W92
            // G5 class — the C# casts `Math.Sqrt`'s double).
            let mut speed = p.local_ai[1];
            if speed == 0.0 {
                speed = p.vel.length();
                p.local_ai[1] = speed;
            }
            // `:52973-52983` — the no-lock fall (the inner `>= 20` is
            // always true under the outer `>= 25`, the C#'s own shape).
            if p.local_ai[0] >= 25.0 && p.ai[1] == 0.0 {
                if p.local_ai[0] >= 20.0 {
                    p.vel.y += 0.15;
                }
                if p.vel.y > 16.0 {
                    p.vel.y = 16.0;
                }
            }
            if p.local_ai[0] >= 20.0 {
                // `:52988-53017` — the acquire scan: the INTEGER half-width
                // centres (`(float)(width / 2)` on the C#'s int fields),
                // the Manhattan distance, the 800f radius and the
                // `Collision.CanHit` from a 1×1 box at the centre.
                let cx = p.pos.x + (p.w as i32 / 2) as f32;
                let cy = p.pos.y + (p.h as i32 / 2) as f32;
                let mut target = Vec2::new(p.pos.x, p.pos.y);
                let mut steer = false;
                if p.ai[1] == 0.0 {
                    let mut scan = 800.0f32;
                    let mut best: i32 = -1;
                    for (i, n) in npcs.iter().enumerate() {
                        if !chaseable(n) {
                            continue;
                        }
                        let (nw, nh) = n.size();
                        let nx = n.x + (nw as i32 / 2) as f32;
                        let ny = n.y + (nh as i32 / 2) as f32;
                        let d = (cx - nx).abs() + (cy - ny).abs();
                        if d < scan && can_hit(world, cx, cy, 1.0, 1.0, n.x, n.y, nw, nh) {
                            scan = d;
                            target = Vec2::new(nx, ny);
                            best = i as i32;
                        }
                    }
                    if best >= 0 {
                        p.ai[1] = best as f32 + 1.0;
                    }
                }
                // `:53018-53032` — the held target: the ignoreDontTakeDamage
                // chase gate + the 1000-Manhattan validity (a merely distant
                // hold keeps its index — the C# never re-scans).
                if p.ai[1] != 0.0 {
                    let held = (p.ai[1] - 1.0) as i32;
                    if let Some(n) = npcs.get(held as usize) {
                        if chaseable_held(n) {
                            let (nw, nh) = n.size();
                            let nx = n.x + (nw as i32 / 2) as f32;
                            let ny = n.y + (nh as i32 / 2) as f32;
                            if (cx - nx).abs() + (cy - ny).abs() < 1000.0 {
                                steer = true;
                                target = Vec2::new(nx, ny);
                            }
                        }
                    }
                }
                // `:53033-53036` — the friendly gate.
                if !p.friendly {
                    steer = false;
                }
                // `:53037-53049` — the blend: the reciprocal-first
                // `num22 = speed / sqrt(dx² + dy²)` (the C#'s unguarded
                // divide — an exactly-coincident centre NaNs in both),
                // then `(v·7 + d)/8`. The centre uses the FLOAT halves
                // here (`width * 0.5f`, `:53040`), unlike the scan's
                // integer halves.
                if steer {
                    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
                    let mut dx = target.x - center.x;
                    let mut dy = target.y - center.y;
                    let k = speed / (dx * dx + dy * dy).sqrt();
                    dx *= k;
                    dy *= k;
                    p.vel.x = (p.vel.x * 7.0 + dx) / 8.0;
                    p.vel.y = (p.vel.y * 7.0 + dy) / 8.0;
                }
            }
        }
        4 => {
            // `:53054-53057` — the ai[1]==1 one-shot +45 phase skip.
            if p.local_ai[0] == 1.0 && p.ai[1] == 1.0 {
                p.local_ai[0] += 45.0;
            }
            // `:53058-53059` — the swirl: the phase angle
            // `localAI[0] · (π/45)`, the perpendicular step at the C#'s
            // 1.5707963705062866 literal (the double-cast seam).
            let dir = safe_normalize(p.vel);
            let x = rotate(dir, p.local_ai[0] * (std::f32::consts::PI / 45.0)).x;
            p.pos += rotate(dir, 1.5707963705062866f32) * x * 3.0;
            // `:53060-53068` — the scale pin, the 10-tick fall, the clamp.
            p.scale = 2.0;
            if p.local_ai[0] >= 10.0 {
                p.vel.y += 0.04;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        5 => {
            // `:53072-53079` — the 40-tick fall.
            if p.local_ai[0] >= 40.0 {
                p.vel.y += 0.08;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        6 => {
            // `:53082-53089` — the 30-tick fall.
            if p.local_ai[0] >= 30.0 {
                p.vel.y += 0.1;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        _ => {}
    }
}

/// The `Kill()` arm's resize (`Projectile.cs:73632-73645`): the position
/// snaps to the centre, the box grows to 128×128 (240×240 for the large
/// 717/718) and recentres on the INTEGER half (`width / 2` on the C#'s int
/// field). The `SoundID.Item14` is the sound skip; the `Damage()` and the
/// explosion ride the pool's death hook (`projectile.rs::celeb2_death`).
pub(crate) fn celeb2_kill_resize(p: &mut Projectile) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let side = if p.kind == 717 || p.kind == 718 {
        240.0
    } else {
        128.0
    };
    p.pos = center;
    p.w = side;
    p.h = side;
    p.pos.x -= (side as i32 / 2) as f32;
    p.pos.y -= (side as i32 / 2) as f32;
}

/// The 932's `Kill()` dust census (`Projectile.cs:71109-71147`): one
/// DISCARDED `NextFloat` (`:71114`), then the 10-step trail — each step
/// rolls `Next(1, 3)` (`:71119`), decrements it past the oldPos ring's
/// 0.3/0.75 marks (`:71121-71128`, `oldPos.Length = 10`), and spawns that
/// many dusts with THREE explicit floats each (the velocity scale
/// `:71134`, the scale `:71136`, the fadeIn `:71137`). The `CloneDust`
/// internals go with the dust pool; a negative count spawns nothing (the
/// C#'s `num47 < num45` loop). The colour helper draws nothing (module
/// doc); the `SoundID.Item10` is the sound skip.
pub(crate) fn fairy_queen_death_draws(rng: &mut dyn DropRng) {
    let _ = rng.next_float(); // `:71114`
    for step in 0..10 {
        let mut count = rng.next_range(1, 3);
        if step as f32 >= 10.0 * 0.3 {
            count -= 1;
        }
        if step as f32 >= 10.0 * 0.75 {
            count -= 2;
        }
        for _ in 0..count.max(0) {
            let _ = rng.next_float();
            let _ = rng.next_float();
            let _ = rng.next_float();
        }
    }
}

/// The Bubble family's `Kill()` pop (`Projectile.cs:72849-72873` the
/// Flairon bubble 405, `:72919-72943` the Bubble Gun's bubble 410): the
/// `SoundID.Item54` pop at `base.position` (the legacy style (2, 54),
/// `SoundID.cs:1437`), then the dust-212 ring census. W4-d8 DOC-SWEEP (2):
/// the 405 arm is the d7 Flairon lane's pop (the same cue + the 20-dust
/// ring); the 410 arm is this lane's — 10 dusts on the `(int)(10f * ai[1])`
/// box (`:72925`), the `num294 < 30` velocity arm ALWAYS taken (the loop
/// caps at 10, so the `Next(45, 91)` else at `:72936` is dead code), one
/// `NextDouble` colour-hue draw per dust (`:72938`). The dust BODIES and
/// the `Dust.NewDust` internals go with the dust pool (the module's RNG
/// note); the cue rides the slot's `wants_sound` intent (the S1-E-21h
/// pattern — the death hook runs after the sub-step drain, so the pool
/// forwards it at the hook itself).
pub(crate) fn bubble_pop_kill(p: &mut Projectile, rng: &mut dyn DropRng) {
    let count = match p.kind {
        FLAIRON_BUBBLE => 20, // `:72853` — the fixed 10-px ring box
        BUBBLE => 10,         // `:72923` — the ai[1]-scaled ring box
        _ => return,
    };
    // `:72851`/`:72921` — `SoundEngine.PlaySound(SoundID.Item54,
    // base.position)`: the legacy (2, 54) at the slot's top-left.
    p.wants_sound = Some((2, 54, p.pos.x, p.pos.y, 0.0));
    for _ in 0..count {
        // `:72868`/`:72938` — the hslToRgb hue draw (0.4 + NextDouble *
        // 0.2); the colour body is the dust layer's.
        let _ = rng.next_double();
    }
}

/// `AI_147_Explosion` (`Projectile.cs:53130-53399`) — the per-state
/// firework burst, as the DRAW CENSUS: every explicit `Main.rand` draw
/// stays on the stream in the C#'s order, the dust/gore bodies are
/// skipped. Per state (the gore rows are 7 range draws each: the position
/// `Next(100)` pair, the `Next(61, 64)` type, the `NewGore` kicks
/// `Next(10, 31)`/`Next(-20, 21)` — `Gore.cs:1327-1328` — and the two
/// `Next(-10, 11)` velocity kicks):
/// - 0 (`:53134-53156`): 4+40 dust (no explicit draws) + 1 gore = 7 ranges.
/// - 1 (`:53158-53183`): one DISCARDED `NextFloat` (`:53162`), the 0.0125
///   ring's 2 floats per particle (colour + velocity scale), 3 gores,
///   then [`create_ground_explosion`]'s tile walk.
/// - 2 (`:53185-53271`): `Next(4)` (the big-burst pick), one
///   `NextFloatDirection` (the star rotation), the 5-spoke dust web rolls
///   nothing, then BOTH branches' 1 gore (the dust counts differ, the
///   draws do not).
/// - 3 (`:53273-53306`): one `NextFloat` (the ring rotation), 40 colour
///   `NextFloat`s, 1 gore.
/// - 4 (`:53308-53332`): one DISCARDED `NextFloat`, the 0.025 ring's 1
///   float per particle, 3 gores.
/// - 5 (`:53334-53348`): 20 colour `NextFloat`s (the second dust's colour
///   is fixed).
/// - 6 (`:53350-53396`): the three rings' `NextFloatDirection` per
///   particle (0.015 / 0.025 / 0.07 steps), 1 gore.
///
/// The float-step loops keep the C#'s f32 accumulation (`num += 0.0125f`)
/// — the iteration count is the source's own, drift included.
pub(crate) fn celeb2_explosion(p: &Projectile, world: &World, rng: &mut dyn DropRng) {
    // One gore's draw row (the `:53151-53154` shape, shared by every
    // state): position pair, type, the NewGore kicks, the velocity kicks.
    fn gore(rng: &mut dyn DropRng) {
        let _ = rng.next_range(0, 100);
        let _ = rng.next_range(0, 100);
        let _ = rng.next_range(61, 64);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-10, 11);
        let _ = rng.next_range(-10, 11);
    }
    match p.ai[0] as i32 {
        0 => {
            // `:53136-53155` — the dust rolls nothing; one gore.
            gore(rng);
        }
        1 => {
            // `:53162` — the discarded draw (the C# keeps it on the stream).
            let _ = rng.next_float();
            // `:53163-53174` — the 0.0125 ring: colour + velocity scale.
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float();
                let _ = rng.next_float();
                t += 0.0125;
            }
            // `:53175-53181` — three gores.
            for _ in 0..3 {
                gore(rng);
            }
            // `:53182` — the ground explosion.
            create_ground_explosion(p, world, rng, 40.0, 20, 100, 2, 10, 2);
        }
        2 => {
            // `:53191-53197` — the big-burst pick and the star rotation.
            let big = rng.next_range(0, 4) == 0;
            let _ = rng.next_float(); // NextFloatDirection (`Utils.cs:1203-1206`)
            // `:53198-53224` — the 5-spoke web: no explicit draws.
            // `:53225-53270` — both branches end in one gore.
            let _ = big;
            gore(rng);
        }
        3 => {
            // `:53281` — the ring rotation.
            let _ = rng.next_float();
            // `:53282-53298` — 40 colour draws.
            for _ in 0..40 {
                let _ = rng.next_float();
            }
            // `:53299-53305` — one gore.
            gore(rng);
        }
        4 => {
            // `:53312` — the discarded draw.
            let _ = rng.next_float();
            // `:53313-53324` — the 0.025 ring: one velocity-scale float each.
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float();
                t += 0.025;
            }
            // `:53325-53331` — three gores.
            for _ in 0..3 {
                gore(rng);
            }
        }
        5 => {
            // `:53338-53347` — 20 colour draws (the 27 dust is fixed-colour).
            for _ in 0..20 {
                let _ = rng.next_float();
            }
        }
        6 => {
            // `:53358-53388` — the three radius-wobble rings.
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float(); // NextFloatDirection
                t += 0.015;
            }
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float();
                t += 0.025;
            }
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float();
                t += 0.07;
            }
            // `:53389-53395` — one gore.
            gore(rng);
        }
        _ => {}
    }
}

/// `WorldGen.KillTile_GetTileDustAmount(fail: true, tile)`
/// (`WorldGen.cs:66597-66685`) — the per-kind dust count the ground
/// explosion's tile walk reads. The `fail: true` arms only (the sole
/// caller passes true).
fn kill_tile_dust_amount_fail(kind: u16, frame_x: i16) -> i32 {
    match kind {
        654 | 630 | 631 => 0,
        231 => 3,
        481 | 482 | 483 => 3,
        541 => 1,
        549 => 2,
        125 | 138 | 172 | 287 | 300..=308 | 354 | 355 | 373..=376 | 461 | 484 | 499
        | 564 | 565 | 593 | 594 | 617 | 664 | 665 | 699 | 709 | 711..=716 => 0,
        534 | 536 | 539 | 625 | 627 => 3,
        184 => {
            if frame_x as i32 / 22 >= 5 {
                5
            } else {
                3
            }
        }
        _ => 3,
    }
}

/// `CreateGroundExplosion(MAX_SPREAD, fluff, distFluff, layerStart,
/// layerEnd, layerJump)` (`Projectile.cs:53401-53464`) — the state-1
/// burst's ground dust: the fluffed tile box, the four layers (2/4/6/8),
/// the radius gate, the exposed-solid-tile gate, then per tile the dust
/// amount's `NextFloat` velocity scales (the `KillTile_MakeTileDust`
/// bodies are the dust skip; its one explicit `Next(3)` at `:67402` is
/// the type-30 arm — not a full solid, unreachable here), the `Next(3)`
/// gore gate and the taken branch's gore (`61 + Next(3)` type + the
/// NewGore kicks + one `NextFloat` velocity scale).
///
/// The `InWorld(j, k, 10)` refusal RETURNS from the whole walk (`:53418-
/// 53421`) — the C#'s early exit, kept.
fn create_ground_explosion(
    p: &Projectile,
    world: &World,
    rng: &mut dyn DropRng,
    _max_spread: f32,
    fluff: i32,
    dist_fluff: i32,
    layer_start: i32,
    layer_end: i32,
    layer_jump: i32,
) {
    // `:53403-53410` — the fluffed tile box (the `(int)(X/16)` truncation)
    // and the INTEGER radius half (`width / 2`). The `num = point.X/2 +
    // point2.X/2` mid-column feeds only the gore scale (draw-only, skipped).
    let x0 = (p.pos.x / 16.0) as i32 - fluff;
    let y0 = (p.pos.y / 16.0) as i32 - fluff;
    let x1 = ((p.pos.x + p.w) / 16.0) as i32 + fluff;
    let y1 = ((p.pos.y + p.h) / 16.0) as i32 + fluff;
    let radius = (p.w as i32 / 2 + dist_fluff) as f32;
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let mut layer = layer_start;
    while layer < layer_end {
        for j in x0..=x1 {
            for k in y0..=y1 {
                // `:53418-53421` — the InWorld(10) refusal aborts the walk.
                if j < 10 || j >= world.width - 10 || k < 10 || k >= world.height - 10 {
                    return;
                }
                // `:53422-53425` — the radius gate.
                if center.distance(Vec2::new(j as f32 * 16.0, k as f32 * 16.0)) > radius {
                    continue;
                }
                // `:53426-53435` — the exposed full-solid gate.
                let Some(t) = world.tile(j, k) else {
                    continue;
                };
                let solid = tr_core::tiledata::TILE_SOLID.has(t.kind);
                let solid_top = tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
                let important = world
                    .frame_important
                    .get(t.kind as usize)
                    .copied()
                    .unwrap_or(false);
                if !t.active() || !solid || solid_top || important {
                    continue;
                }
                let above_full_solid = world.tile(j, k - 1).is_some_and(|a| {
                    a.active()
                        && tr_core::tiledata::TILE_SOLID.has(a.kind)
                        && !tr_core::tiledata::TILE_SOLID_TOP.has(a.kind)
                });
                if above_full_solid {
                    continue;
                }
                // `:53436-53443` — the first dust row: one NextFloat each.
                let amount = kill_tile_dust_amount_fail(t.kind, t.frame_x);
                for _ in 0..amount {
                    let _ = rng.next_float();
                }
                // `:53444-53452` — the layer >= 2 second row (amount - 1).
                if layer >= 2 {
                    for _ in 0..(amount - 1) {
                        let _ = rng.next_float();
                    }
                }
                // `:53453-53460` — the 2-in-3 gore gate.
                if amount > 0 && rng.next_range(0, 3) != 0 {
                    let _ = rng.next_range(0, 3); // the 61 + Next(3) type
                    let _ = rng.next_range(10, 31); // NewGore's Y kick
                    let _ = rng.next_range(-20, 21); // NewGore's X kick
                    let _ = rng.next_float(); // the velocity.Y scale
                }
            }
        }
        layer += layer_jump;
    }
}

// ---------------------------------------------------------------------------
// W4-d8 — the aiStyle-72/73/74/76 census (the Bubble Gun, the firework
// fountains, the Scutlix mount's laser and the two mount crosshairs). The
// data rows + kind constants live in projectile/data.rs; the dispatch arms
// are wired in projectile.rs::update. Code-only round — the witness tests
// below are WRITTEN NOT RUN.
//
// C# sources read WHOLE (the golden rule):
// - aiStyle 72: the inline `AI()` arm `Projectile.cs:30455-30479` (the
//   Bubble 410's fade, the 0.96 drag, the one-shot scale roll, the
//   centre-anchored box resize). `SetDefaults` `:4917-4932`.
// - aiStyle 73: the inline `AI()` arm `:30480-30604` (the firework
//   fountains 419-422's tile-338 anchor kill + the four per-kind dust
//   choreographies). `SetDefaults` `:4951-4959`; the `NewProjectile`
//   timeLeft-600 seed `:11033-11048`; the spawn `Player.cs:32374`.
// - aiStyle 74: the inline `AI()` arm `:30605-30616` (the Scutlix laser
//   434's charge-decay kill), the Update-tail charge latch `:17079-17086`,
//   the on-hit `numUpdates = 0` `:13975-13978`, the blocked-move
//   `numUpdates = 0` `:17991-17995`, the `Damage_CanDealDamage` refusal
//   `:12448` (`type == 434 && localAI[0] != 0`). `SetDefaults` `:5047-5055`.
// - aiStyle 76: the inline `AI()` arm `:30621-30670` (the mount crosshairs
//   441/453's owner pin, the mount-type kill, the mouse aim + the channel
//   release). `SetDefaults` `:5117-5125`/`:5250-5258`.
//
// RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
// every EXPLICIT `Main.rand` draw stays in the C#'s order; the dust BODIES
// are skipped (no dust layer) and the `Dust.NewDust*` internal rolls go
// with the dust pool (the module's convention). Per style:
// - 72: one `Next(60, 121)` scale roll on the first tick (`:30469`).
// - 73: the per-kind dust draws — see [`ai_fountain`]'s doc.
// - 74: ZERO draws (the decay/latch are deterministic).
// - 76: ZERO draws (the aim/pin are deterministic).

/// `AI()`'s aiStyle-72 arm (`Projectile.cs:30455-30479`) — the Bubble Gun's
/// bubble (410). The age counter, the post-3-tick alpha fade to its 50
/// floor, the ×0.96 drag, the one-shot `Next(60, 121) / 100f` scale roll
/// (`netUpdate` dirtied) and the centre-anchored `(int)(14 * ai[1])` box
/// resize. The movement is the dispatch's `kill_step` (the row keeps
/// `tileCollide = true`); the wind tail runs at the dispatch (style 72 is
/// on `ShouldUseWindPhysics`'s list, `:22128`).
pub(crate) fn ai_bubble(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:30457-30465` — the age counter; past the 3rd tick the alpha fades
    // 25/tick to its 50 floor.
    p.local_ai[0] += 1.0;
    if p.local_ai[0] > 3.0 {
        p.alpha -= 25;
        if p.alpha < 50 {
            p.alpha = 50;
        }
    }
    // `:30466` — the ×0.96 drag.
    p.vel *= 0.96;
    // `:30467-30471` — the one-shot scale roll (`netUpdate = true`).
    if p.ai[1] == 0.0 {
        p.ai[1] = rng.next_range(60, 121) as f32 / 100.0;
        p.net_update = true;
    }
    // `:30472` — the scale follows the roll every tick.
    p.scale = p.ai[1];
    // `:30473-30478` — the centre-anchored box resize: `position = Center`,
    // `width = height = (int)(14 * ai[1])`, then `position -= (width/2,
    // height/2)`. The C# `width / 2` is INTEGER division (the box is an
    // int), so the port truncates the half-box.
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let size = (14.0 * p.ai[1]) as i32 as f32;
    p.w = size;
    p.h = size;
    p.pos = Vec2::new(cx - (size as i32 / 2) as f32, cy - (size as i32 / 2) as f32);
}

/// `AI()`'s aiStyle-73 arm (`Projectile.cs:30480-30604`) — the firework
/// fountains (419-422). The tile-338 anchor kill, then the four per-kind
/// dust choreographies. The fountain never damages: the C# keeps
/// `friendly = true` (`:4956`) but the tile-interaction spawn passes
/// Damage 0 (`Player.cs:32374`), so `damage_pve` early-returns.
///
/// RNG (per tick, the C#'s order): the shared `num569 = 2f` jet speed
/// scales by `timeLeft / 60f` below 1. Each kind's two-dust loop draws, per
/// dust, a `NextDouble` jet speed-scale + a `NextDouble` jet rotation, the
/// red 420 / rainbow 422 `Next(5)` colour pick, a `NextDouble` dust scale
/// and a `Next(-2, 3)` position jitter; the blue 421's `timeLeft % 10 == 0`
/// burst draws one shared `NextDouble` speed-scale + nine per-dust
/// `NextDouble` scale + `Next(-2, 3)` jitter pairs. The dust BODIES skip;
/// the `Dust.NewDust` internal rolls go with the dust pool.
///
/// W4-d8 DOC-SWEEP deviation (1): the jet-speed computation is GAMEPLAY
/// math, not a dust body — the port now computes `num569` and every jet
/// vector ([`fountain_jet`]) in the C#'s order; only the dust slots
/// themselves (position/scale/colour writes) stay with the g-lanes.
pub(crate) fn ai_fountain(p: &mut Projectile, world: &World, rng: &mut dyn DropRng) {
    // `:30482-30489` — the tile anchor: `ai[0]`/`ai[1]` name the fountain
    // tile; a missing / inactive / non-338 tile kills the projectile.
    let tx = p.ai[0] as i32;
    let ty = p.ai[1] as i32;
    let anchored = world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == 338);
    if !anchored {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:30490-30495` — the jet speed `num569 = 2f`, scaled by
    // `timeLeft / 60f` below 1. The float division + compare are the C#'s
    // exactly (`num570 < 1f`); the product feeds every jet vector below.
    let mut jet_speed = 2.0f32;
    let life_scale = p.time_left as f32 / 60.0;
    if life_scale < 1.0 {
        jet_speed *= life_scale;
    }
    // The per-kind dust choreographies. Each two-dust loop draws, in the
    // C#'s order: a `NextDouble` speed-scale + a `NextDouble` rotation
    // (the jet vector, `:30500`), the red 420 / rainbow 422 `Next(5)`
    // colour pick (`:30522`/`:30590`), a `NextDouble` dust scale
    // (`:30503`) and a `Next(-2, 3)` position jitter (`:30509`). The dust
    // BODIES skip; the `Dust.NewDust` internal rolls go with the dust pool
    // (the module's RNG note). The jet vectors are COMPUTED (the sweep's
    // gameplay-math port); their consumers are the skipped dust slots.
    match p.kind {
        419 => {
            // `:30496-30515` — the yellow: two dust-222, the ±π/2 rotation.
            for _ in 0..2 {
                let _jet = fountain_jet(jet_speed, 1.5701111111162866, rng);
                let _ = rng.next_double(); // the dust scale (`:30503`)
                let _ = rng.next_range(-2, 3); // the position jitter (`:30509`)
            }
        }
        420 => {
            // `:30516-30541` — the red: two dust-219 (1-in-5 dust-222).
            for _ in 0..2 {
                let _jet = fountain_jet(jet_speed, 1.5701111111162866, rng);
                let _ = rng.next_range(0, 5); // the colour pick (`:30522`)
                let _ = rng.next_double(); // the dust scale (`:30528`)
                let _ = rng.next_range(-2, 3); // the position jitter (`:30534`)
            }
        }
        421 => {
            // `:30542-30582` — the blue: two dust-221 (the ±π/4 rotation),
            // then the timeLeft%10 burst's nine dust-222.
            for _ in 0..2 {
                let _jet = fountain_jet(jet_speed, 0.7851111111131433, rng);
                let _ = rng.next_double(); // the dust scale (`:30549`)
                let _ = rng.next_range(-2, 3); // the position jitter (`:30555`)
            }
            if p.time_left % 10 == 0 {
                // `:30563` — the burst's shared speed scale.
                let burst_scale = 0.85 + rng.next_double() as f32 * 0.2;
                for spoke in 0..9 {
                    // `:30566` — the deterministic spoke fan:
                    // `((spoke - 4) / 5f, -num569 * num578)`.
                    let _jet = Vec2::new(
                        (spoke - 4) as f32 / 5.0,
                        (0.0 - jet_speed) * burst_scale,
                    );
                    // `:30569`/`:30575` — per burst dust: the scale draw +
                    // the position jitter.
                    let _ = rng.next_double();
                    let _ = rng.next_range(-2, 3);
                }
            }
        }
        422 => {
            // `:30583-30603` — the rainbow: two dust-(219 + Next(5)).
            for _ in 0..2 {
                let _jet = fountain_jet(jet_speed, 1.5701111111162866, rng);
                let _ = rng.next_range(0, 5); // the colour offset (`:30590`)
                let _ = rng.next_double(); // the dust scale (`:30592`)
                let _ = rng.next_range(-2, 3); // the position jitter (`:30598`)
            }
        }
        _ => {}
    }
    // The movement: the row keeps `tileCollide = true`, but the fountain
    // sits still (velocity 0) on its anchor tile — the dispatch's
    // `kill_step` integrates the zero velocity (no move, no kill).
}

/// The fountain's per-dust jet vector — `(new Vector2(0f, 0f - num569) *
/// (0.85f + (float)Main.rand.NextDouble() * 0.2f)).RotatedBy((Main.rand.
/// NextDouble() - 0.5) * half_turn)` (`Projectile.cs:30500`/`:30520`/
/// `:30546`/`:30589`). TWO draws in the C#'s order: the speed scale first,
/// then the rotation. The rotation is `Utils.RotatedBy`'s float-cos/sin
/// spin (`Utils.cs:1675-1684`, the [`crate::bow_launch::rotated_by`]
/// port). `half_turn` is the C# float literal widened to f64: π/2
/// (1.5701111111162866) for the yellow/red/rainbow, π/4
/// (0.7851111111131433) for the blue.
pub(crate) fn fountain_jet(jet_speed: f32, half_turn: f64, rng: &mut dyn DropRng) -> Vec2 {
    let scale = 0.85 + rng.next_double() as f32 * 0.2;
    let angle = (rng.next_double() - 0.5) * half_turn;
    crate::bow_launch::rotated_by(Vec2::new(0.0, 0.0 - jet_speed) * scale, angle)
}

/// `AI()`'s aiStyle-74 arm (`Projectile.cs:30605-30616`) — the Scutlix
/// mount's laser (434). Once the charge latch flips `extraUpdates` to 1
/// (the Update-tail `:17079-17086`), the decay arm runs: `localAI[0] *=
/// localAI[1]`, `localAI[1] -= 0.001f`, and the kill below 0.01. The
/// charge latch itself reads the slot's `num_updates` field at the
/// dispatch (the Update tail, not this AI arm).
pub(crate) fn ai_scutlix_laser(p: &mut Projectile) {
    // `:30607-30615` — the decay arm, gated on the latched extraUpdates 1.
    if p.extra_updates == 1 {
        p.local_ai[0] *= p.local_ai[1];
        p.local_ai[1] -= 0.001;
        // `:30611-30614` — the `(double)localAI[0] < 0.01` kill.
        if (p.local_ai[0] as f64) < 0.01 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
}

/// The Update-tail charge latch (`Projectile.cs:17079-17086`) — the Scutlix
/// laser's first sub-step (`localAI[0] == 0 && numUpdates == 0`) flips
/// `extraUpdates` to 1, zeroes the velocity, seeds `localAI[0] = 1` /
/// `localAI[1] = 0.9999f` and dirties `netUpdate`. Runs at the dispatch
/// AFTER the AI + movement + damage (the C# `:17078` `Damage()` precedes
/// it), so the wire's last sub-step is the one that latches.
pub(crate) fn scutlix_laser_charge_latch(p: &mut Projectile) {
    if p.kind == SCUTLIX_LASER_FRIENDLY && p.local_ai[0] == 0.0 && p.num_updates == 0 {
        p.extra_updates = 1;
        p.vel = Vec2::ZERO;
        p.local_ai[0] = 1.0;
        p.local_ai[1] = 0.9999;
        p.net_update = true;
    }
}

/// `AI()`'s aiStyle-76 arm (`Projectile.cs:30621-30670`) — the mount
/// crosshairs (441 Scutlix / 453 Drill). The owner pin (`heldProj`), the
/// mount-type kill, the local-owner mouse aim + the channel release, and
/// the remote-owner position follow. The crosshair is stock `friendly =
/// false` (`:788`); the port's slot default is true, so the arm pins it
/// per tick (the ai_dungeon_debris precedent). The `heldProj` write rides
/// the mutable player.
///
/// W4-d8 DOC-SWEEP (3)/(4) — the mount ability calls are PORTED for the
/// self-contained Scutlix half: `AimAbility` (`:30648`/`:30669`) →
/// [`crate::mount::MountState::aim_ability_scutlix`] (the facing fold +
/// the frame buckets + the flip-draw), the channel-release `UseAbility`
/// (`:30651`) → [`crate::mount::MountState::scutlix_laser_fire`] (the two
/// 606 beams park on `wants_scutlix_beams`, the dispatch spawns them),
/// the remote `StartAbilityCharge` (`:30662`) → the `_abilityCharging`
/// latch (the local-owner 441 spawn itself is the auto-aim's
/// `UpdateEffects` path, M2-MOUNT-4 — the crosshair's own remote gate
/// only sets the flag, `Mount.cs:2916-2919`).
///
/// W4-d8 FINAL-PHASE (1) — the Drill (8) half's SELF-CONTAINED state is
/// ported too: `AimAbility`'s Drill case (`:30648`/`:30669`) →
/// [`crate::mount::MountState::aim_ability_drill`] (the diode target/wrap +
/// the crosshair pin), the channel-release `UseAbility(toggleOn: false)`
/// (`:30651`) → [`crate::mount::MountState::use_ability_drill`] (the
/// `_abilityActive` clear). The `UseDrill` mining WRITE (`Mount.cs:3059-
/// 3180` — `PickTile`/`PickWall`/`Tile.SmoothSlope`/`Hurt`/dust) stays
/// M2-MOUNT-4 (`Tile.SmoothSlope` unported, the writes are App-level seams;
/// the split-contract doc, `docs/research/97`).
pub(crate) fn ai_mount_crosshair(
    p: &mut Projectile,
    i: usize,
    aim: Option<(f32, f32)>,
    player: Option<&mut crate::player::Player>,
    local_owner: u8,
) {
    // The stock friendly = false (`:788`), held for the slot's whole life.
    p.friendly = false;
    let Some(pl) = player else {
        // No owner resolved — the slot cannot track; kill it (the C# reads
        // `Main.player[owner]` unconditionally, the ai_drill precedent).
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    };
    // `:30624` — the owner pin (`player.heldProj = whoAmI`).
    pl.held_proj = i as i32;
    // `:30625-30637` — the mount-type kill: the 441 needs mount 9 (Scutlix),
    // the 453 mount 8 (Drill).
    let mount_ok = match p.kind {
        SCUTLIX_LASER_CROSSHAIR => pl.mount.kind() == 9,
        DRILL_MOUNT_CROSSHAIR => pl.mount.kind() == 8,
        _ => false,
    };
    if !mount_ok {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    if p.owner == local_owner {
        // `:30638-30654` — the local owner: the mouse-world aim, the
        // owner-relative ai[0]/ai[1] pin (netUpdate on change), the
        // channel-release kill. `Main.screenPosition + mouse` is the
        // pass's aim snapshot.
        if let Some((mx, my)) = aim {
            p.pos = Vec2::new(mx, my);
            let dx = p.pos.x - pl.x;
            let dy = p.pos.y - pl.y;
            if p.ai[0] != dx || p.ai[1] != dy {
                p.net_update = true;
            }
            p.ai[0] = dx;
            p.ai[1] = dy;
        }
        // `:30648` — `mount.AimAbility(player6, base.position)`: the
        // Scutlix head's aim (the facing write rides the player) OR the
        // Drill's diode aim (W4-d8 FINAL-PHASE (1): the self-contained
        // Drill half — the diode target/wrap + the crosshair pin). The
        // position is the crosshair's own (the mouse pin above), NOT the
        // clamped cursor — the clamp lives inside the ability.
        let (pcx, pcy) = pl.center();
        if p.kind == DRILL_MOUNT_CROSSHAIR {
            let _ = pl.mount.aim_ability_drill(Vec2::new(pcx, pcy), p.pos);
        } else {
            let mut facing = pl.facing;
            let _ = pl.mount.aim_ability_scutlix(
                Vec2::new(pcx, pcy),
                &mut facing,
                pl.vx,
                p.pos,
            );
            pl.facing = facing;
        }
        // `:30649-30653` — the channel release: `UseAbility(player6,
        // base.position, toggleOn: false)` then `Kill()`. The Scutlix fire
        // spawns the two 606 beams (the intent parks on the slot); the
        // Drill's `toggleOn: false` clears `_abilityActive` (`Mount.cs:
        // 3317-3320`) — the flag the mining layer reads (W4-d8 FINAL-PHASE
        // (1): the clear is ported; the `UseDrill` write stays M2-MOUNT-4).
        if !pl.channel {
            if p.kind == SCUTLIX_LASER_CROSSHAIR {
                if let Some(beams) = pl.mount.scutlix_laser_fire(
                    Vec2::new(pl.x, pl.y),
                    pl.width,
                    pl.height,
                    pl.facing,
                    p.pos,
                ) {
                    p.wants_scutlix_beams.extend_from_slice(&beams);
                }
            } else {
                // The 453 release: the local `UseAbility(toggleOn: false)`
                // (`:3317-3320`) — no spawn, just the flag clear.
                let _ = pl.mount.use_ability_drill(
                    Vec2::new(pl.x, pl.y),
                    p.pos,
                    false,
                    true,
                );
            }
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
        return;
    }
    // `:30656-30669` — the remote owner: the position follows the stored
    // owner-relative offset.
    p.pos = Vec2::new(pl.x + p.ai[0], pl.y + p.ai[1]);
    // `:30658-30664` — the 441: `StartAbilityCharge` when not already
    // charging. The REMOTE branch (`Mount.cs:2916-2919`) only sets the
    // flag — the local-owner 441 spawn is the auto-aim's path.
    if p.kind == SCUTLIX_LASER_CROSSHAIR && !pl.mount.ability_charging() {
        pl.mount.start_ability_charge_remote();
    }
    // `:30665-30668` — the 453: `UseAbility(toggleOn: false)` when the
    // drill is not active — the Drill half clears `_abilityActive`
    // (`Mount.cs:3332-3335`, the remote branch), the mining layer's gate.
    // The flag write is the DrillMountData payload's (M2-MOUNT-4) — the
    // port carries `ability_active` on the state, so the clear lands.
    if p.kind == DRILL_MOUNT_CROSSHAIR && !pl.mount.ability_active() {
        pl.mount.set_ability_active(false);
    }
    // `:30669` — the remote `AimAbility` (the Scutlix head's aim OR the
    // Drill's diode aim — W4-d8 FINAL-PHASE (1): the Drill half is ported,
    // the remote smart-cursor reads the crosshair pin it writes).
    let (pcx, pcy) = pl.center();
    if p.kind == DRILL_MOUNT_CROSSHAIR {
        let _ = pl.mount.aim_ability_drill(Vec2::new(pcx, pcy), p.pos);
    } else {
        let mut facing = pl.facing;
        let _ = pl.mount.aim_ability_scutlix(
            Vec2::new(pcx, pcy),
            &mut facing,
            pl.vx,
            p.pos,
        );
        pl.facing = facing;
    }
}

// ---------------------------------------------------------------------------
// Witnesses (S1-D-4 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs, proj_data, BUBBLE};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-recording `Main.rand` stand-in (the `projectile.rs`
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0 — the
    /// census proves count AND order. `next_double` records its own count
    /// (W4-d8's fountain census) and returns 0.0, the same stream step the
    /// real `UnifiedRandom.NextDouble` takes (`loot.rs:17-25`).
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
        doubles: usize,
    }

    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
        fn next_double(&mut self) -> f64 {
            self.doubles += 1;
            0.0
        }
    }

    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0, doubles: 0 }
    }

    /// A tile-free world (the flight witnesses run in open air).
    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    /// A slot carrying its `SetDefaults` row slice (the
    /// `projectile_ai001_a` fixture shape — field-by-field on `Default`,
    /// the struct has private bookkeeping fields).
    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn npc_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        n
    }

    /// The four rows keep their census §3c styles (the data half of the
    /// task — the rows already exist, the arms are what this module adds).
    #[test]
    fn s1_d_4_unarmed_the_four_rows_carry_their_styles() {
        assert_eq!(proj_data(145).unwrap().ai_style, 31);
        for kind in 715..=718u16 {
            assert_eq!(proj_data(kind).unwrap().ai_style, 147, "kind {kind}");
        }
        assert_eq!(proj_data(932).unwrap().ai_style, 181);
        assert_eq!(proj_data(85).unwrap().ai_style, 193);
    }

    /// Style 31 (`:26826-26839`, `:26883-26885`): the owner-gated cleanse
    /// converts the corrupt grass under the spray, the life caps at 133,
    /// the counter advances and the spin reads the velocity sign. ZERO
    /// rand draws — the arm takes no rng at all (by construction).
    #[test]
    fn s1_d_4_unarmed_the_spray_cleanses_and_caps() {
        let mut w = world();
        // The corrupt grass 23 + the ebonstone 25 under the centre
        // (100, 50); the Manhattan < 6 disc reaches both.
        put(&mut w, 100, 50, 23);
        put(&mut w, 101, 50, 25);
        let mut p = proj(145);
        p.vel = Vec2::new(-4.0, 0.0);
        p.time_left = 3600;
        ai_spray(&mut p, &mut w, 0);
        assert_eq!(w.tile(100, 50).unwrap().kind, 2, "the grass cleansed");
        assert_eq!(w.tile(101, 50).unwrap().kind, 1, "the stone cleansed");
        assert_eq!(p.time_left, 133, "the cap (`:26836-26839`)");
        assert_eq!(p.ai[0], 1.0, "the counter (`:26883`)");
        assert_eq!(p.rot, -0.3, "the spin against the flight (`:26885`)");
        // The non-owner refuses the conversion (`:26826`).
        let mut w2 = world();
        put(&mut w2, 100, 50, 23);
        let mut p2 = proj(145);
        p2.owner = 1;
        ai_spray(&mut p2, &mut w2, 0);
        assert_eq!(w2.tile(100, 50).unwrap().kind, 23, "the owner gate");
        // The solution flag widens the disc to size 3 (`:26828-26832`) —
        // a cell at Manhattan 5 converts only under the flag.
        let mut w3 = world();
        put(&mut w3, 103, 52, 23); // |3| + |2| = 5 < 6, outside size 2
        let mut p3 = proj(145);
        p3.ai[1] = 1.0;
        ai_spray(&mut p3, &mut w3, 0);
        assert_eq!(w3.tile(103, 52).unwrap().kind, 2, "the size-3 disc");
    }

    /// Style 193 (`:41339-41348`, `:41372`, `:41391`): the counter, the
    /// 60-tick 0.95 decay, the 72-tick kill (no return — the gates still
    /// run), and the two gates' stream positions. CensusRng's floats are
    /// 0 (< every chance) so both taken branches roll their explicit
    /// call-site draws: the flame branch 2 floats + Next(4), the smoke
    /// branch 2 + 2 + 2 floats.
    #[test]
    fn s1_d_4_unarmed_the_flames_decay_and_die_at_72() {
        let mut p = proj(85);
        p.vel = Vec2::new(8.0, 0.0);
        let mut rng = census();
        // Ticks 1..49: the flame gate alone — 1 gate + 2 circular + 1
        // Next(4) per tick.
        for _ in 0..49 {
            ai_flamethrower(&mut p, &mut rng);
        }
        assert_eq!(rng.floats, 49 * 3, "the flame gate + its circular pair");
        assert_eq!(rng.ranges, vec![(0, 4); 49], "the noGravity picks");
        assert_eq!(p.vel.x, 8.0, "no decay before 60");
        // Ticks 50..59: BOTH gates (the smoke window opens at 50, the
        // flame window closes at 50 — `localAI[0] < 50` fails at 50).
        // At tick 50 the flame gate's `localAI[0] < num3` is FALSE — only
        // the smoke gate rolls: 1 + 6 floats, no ranges.
        ai_flamethrower(&mut p, &mut rng);
        assert_eq!(rng.floats, 49 * 3 + 7, "tick 50: smoke only");
        assert_eq!(rng.ranges.len(), 49);
        // Ticks 51..60: the smoke gate alone (7 floats each). The decay
        // (`localAI[0] >= 60`) lands on tick 60 — one 0.95 step.
        for _ in 0..10 {
            ai_flamethrower(&mut p, &mut rng);
        }
        assert_eq!(rng.floats, 49 * 3 + 11 * 7);
        assert_eq!(p.vel.x, 8.0f32 * 0.95, "the first decay at tick 60 (`:41345-41348`)");
        // Tick 61: a second decay; the smoke gate keeps rolling. The C#'s
        // f32 accumulation — repeated * 0.95, not a pow.
        ai_flamethrower(&mut p, &mut rng);
        assert_eq!(p.vel.x, 8.0f32 * 0.95 * 0.95, "the second decay at tick 61");
        assert_eq!(rng.floats, 49 * 3 + 12 * 7);
        // Ticks 62..72: the kill lands at 72 but the arm runs on.
        for _ in 0..11 {
            ai_flamethrower(&mut p, &mut rng);
        }
        assert!(!p.active, "the 72-tick kill (`:41341-41344`)");
        assert_eq!(p.time_left, 0);
        assert_eq!(rng.floats, 49 * 3 + 23 * 7, "the smoke gate post-kill");
        assert_eq!(p.local_ai[0], 72.0);
    }

    /// Style 193's Foxparks flag (`:41349-41356`): ai[0]==1 flips the
    /// windows — the flame gate runs to 60, the smoke gate never opens.
    #[test]
    fn s1_d_4_unarmed_the_foxparks_flag_flips_the_windows() {
        let mut p = proj(85);
        p.ai[0] = 1.0;
        let mut rng = census();
        for _ in 0..72 {
            ai_flamethrower(&mut p, &mut rng);
        }
        // Ticks 1..59: gate + 2 floats + Next(4); ticks 60..72: no gate.
        assert_eq!(rng.floats, 59 * 3, "one window, 59 taken ticks");
        assert_eq!(rng.ranges.len(), 59, "no smoke window");
    }

    /// Style 181 (`:43180-43183`): the alpha ramp truncates the XNA lerp
    /// (255 → 0 over 10 ticks), the rotation reads the velocity, and the
    /// Next(6) gate rolls every tick (CensusRng returns min = 0 → the
    /// taken branch's colour float rides along).
    #[test]
    fn s1_d_4_unarmed_the_fairy_shot_fades_in_ten_ticks() {
        let mut p = proj(932);
        p.vel = Vec2::new(0.0, -6.0);
        let mut rng = census();
        ai_fairy_queen_shot(&mut p, &mut rng);
        assert_eq!(p.ai[0], 1.0);
        // (int)(255 + (0 - 255) * 0.1) = (int)229.5 = 229.
        assert_eq!(p.alpha, 229, "the truncated lerp (`:43181`)");
        assert_eq!(p.rot, -std::f32::consts::FRAC_PI_2, "ToRotation");
        assert_eq!(rng.ranges, vec![(0, 6)], "the gate (`:43183`)");
        assert_eq!(rng.floats, 1, "the colour argument (`:43189`)");
        for _ in 0..9 {
            ai_fairy_queen_shot(&mut p, &mut rng);
        }
        assert_eq!(p.alpha, 0, "fully visible at tick 10");
        assert_eq!(rng.ranges.len(), 10);
        assert_eq!(rng.floats, 10);
    }

    /// Style 147 state 0 (`:52894-52906`): the 20-tick arm opens the
    /// penetration and starts the 0.12 fall; the burst fires strictly
    /// past 20 on the %20 ticks (the case-0 gore row = 7 ranges).
    #[test]
    fn s1_d_4_unarmed_the_celeb2_launch_falls_and_bursts() {
        let w = world();
        let mut p = proj(715);
        let mut rng = census();
        for _ in 0..20 {
            ai_celeb2_rocket(&mut p, &w, &[], &mut rng);
        }
        assert_eq!(p.penetrate, -1, "the arm at 20 (`:52897`)");
        assert_eq!(p.max_penetrate, -1);
        assert_eq!(p.vel.y, 0.12, "one fall step at tick 20");
        assert_eq!(rng.ranges.len(), 0, "no burst at exactly 20 (strict >)");
        for _ in 0..20 {
            ai_celeb2_rocket(&mut p, &w, &[], &mut rng);
        }
        // Tick 40: the first burst — the case-0 gore row.
        assert_eq!(
            rng.ranges,
            vec![
                (0, 100), (0, 100), (61, 64), (10, 31), (-20, 21), (-10, 11), (-10, 11)
            ],
            "the gore row (`:53151-53154` + Gore.cs:1327-1328)"
        );
        // 21 fall steps (ticks 20..40) — the C#'s own f32 accumulation.
        let mut vy = 0.0f32;
        for _ in 0..21 {
            vy += 0.12;
        }
        assert_eq!(p.vel.y, vy, "21 fall steps (ticks 20..40)");
    }

    /// Style 147 state 1 (`:52910-52951`): the tick-10 −10 kick, the 0.25
    /// fall, the 2.5 scale cap and the ring's per-particle colour floats
    /// (the f32 0.12 loop's own iteration count).
    #[test]
    fn s1_d_4_unarmed_the_celeb2_pop_kicks_and_rings() {
        let w = world();
        let mut p = proj(716);
        p.ai[0] = 1.0;
        p.vel = Vec2::new(0.0, 5.0);
        let mut rng = census();
        for _ in 0..9 {
            ai_celeb2_rocket(&mut p, &w, &[], &mut rng);
        }
        assert_eq!(p.vel.y, 5.0, "no kick before 10");
        ai_celeb2_rocket(&mut p, &w, &[], &mut rng);
        // Tick 10: −10 then +0.25 → −4.75; the ring rolls its floats.
        assert_eq!(p.vel.y, -4.75, "the kick + fall (`:52910-52917`)");
        assert_eq!(p.scale, 1.0f32 + 0.015, "the scale climb (`:52926`)");
        let mut n = 0;
        let mut t = 0.0f32;
        while t < 1.0 {
            n += 1;
            t += 0.12;
        }
        assert_eq!(rng.floats, n, "one colour float per ring particle");
        // The scale caps at 2.5 (`:52927-52930`).
        for _ in 0..200 {
            ai_celeb2_rocket(&mut p, &w, &[], &mut rng);
        }
        assert_eq!(p.scale, 2.5);
        assert_eq!(p.vel.y, 16.0, "the clamp (`:52918-52921`)");
    }

    /// Style 147 state 3 (`:52966-53049`): the speed latch, the 800-
    /// Manhattan acquire committing ai[1] = index + 1, and the (v·7+d)/8
    /// blend at the latched speed. The NPC sits 100 px right of the
    /// rocket's centre; the integer-half centres coincide with the float
    /// halves for the even boxes here.
    #[test]
    fn s1_d_4_unarmed_the_celeb2_homes_on_the_acquired_target() {
        let w = world();
        let mut p = proj(717);
        p.ai[0] = 3.0;
        p.vel = Vec2::new(6.0, 0.0);
        p.local_ai[0] = 24.0; // the next tick is 25 — the scan window
        // Place the NPC so its INTEGER-half centre sits exactly 100 px
        // right of the rocket's centre (the blend then keeps velocity 6).
        let mut n = npc_at(0.0, 0.0);
        let (nw, nh) = n.size();
        let cx = p.pos.x + (p.w as i32 / 2) as f32;
        let cy = p.pos.y + (p.h as i32 / 2) as f32;
        n.x = cx + 100.0 - (nw as i32 / 2) as f32;
        n.y = cy - (nh as i32 / 2) as f32;
        let mut rng = census();
        ai_celeb2_rocket(&mut p, &w, &[n], &mut rng);
        assert_eq!(p.local_ai[1], 6.0, "the speed latch (`:52966-52972`)");
        assert_eq!(p.ai[1], 1.0, "the acquire commits index + 1 (`:53014`)");
        // The blend: d = (6, 0) (the target is exactly right at the
        // latched speed), so velocity.X stays 6. (The no-lock fall DID
        // run this tick — ai[1] was still 0 at `:52973`, before the
        // acquire — so vel.Y picked up 0.15 then blended; only X is
        // pinned here.)
        assert!((p.vel.x - 6.0).abs() < 1e-4, "the blend holds 6: {}", p.vel.x);
        assert_eq!(rng.floats, 0, "the homing draws nothing");
        // A hostile rocket never steers (`:53033-53036`).
        let mut h = proj(717);
        h.ai[0] = 3.0;
        h.friendly = false;
        h.vel = Vec2::new(0.0, 6.0);
        h.local_ai[0] = 24.0;
        let n2 = npc_at(h.pos.x + 100.0, h.pos.y);
        ai_celeb2_rocket(&mut h, &w, &[n2], &mut rng);
        assert_eq!(h.vel.x, 0.0, "the friendly gate refuses the steer");
    }

    /// Style 147 state 4 (`:53054-53068`): the ai[1]==1 +45 skip, the
    /// scale-2 pin and the perpendicular position step. The counter
    /// increments at the arm's top (`:52886`), so the skip reads 1 and
    /// lands on 46: x = cos(46·π/45) is NEGATIVE, so the step along
    /// rotate((1,0), 1.5707964) ≈ (0,1) moves −Y.
    #[test]
    fn s1_d_4_unarmed_the_celeb2_swirl_skips_and_steps() {
        let w = world();
        let mut p = proj(718);
        p.ai[0] = 4.0;
        p.ai[1] = 1.0;
        p.vel = Vec2::new(8.0, 0.0);
        let before = p.pos;
        let mut rng = census();
        ai_celeb2_rocket(&mut p, &w, &[], &mut rng);
        assert_eq!(p.local_ai[0], 46.0, "1 + 45 (`:52886` + `:53054-53057`)");
        assert_eq!(p.scale, 2.0, "the pin (`:53060`)");
        let phase = 46.0f32 * (std::f32::consts::PI / 45.0);
        let expect_y = rotate(Vec2::new(1.0, 0.0), 1.5707963705062866f32).y
            * phase.cos()
            * 3.0;
        let expect_x = rotate(Vec2::new(1.0, 0.0), 1.5707963705062866f32).x
            * phase.cos()
            * 3.0;
        assert!(
            (p.pos.x - before.x - expect_x).abs() < 1e-3,
            "the along-flight residue: {} vs {expect_x}",
            p.pos.x - before.x
        );
        assert!(
            (p.pos.y - before.y - expect_y).abs() < 1e-3,
            "the perpendicular step: {} vs {expect_y}",
            p.pos.y - before.y
        );
        assert!(expect_y < 0.0, "the post-skip phase steps backwards");
        assert_eq!(p.vel.y, 0.04, "the fall at >= 10 (`:53061-53064`)");
    }

    /// The explosion censuses (`:53130-53399`): states 3/5/6's float
    /// counts and gore rows, with the f32 loop counts computed by the
    /// C#'s own accumulation.
    #[test]
    fn s1_d_4_unarmed_the_explosion_draw_censuses() {
        let w = world();
        // State 3: 1 + 40 floats, one gore row.
        let mut p = proj(715);
        p.ai[0] = 3.0;
        let mut rng = census();
        celeb2_explosion(&p, &w, &mut rng);
        assert_eq!(rng.floats, 41);
        assert_eq!(rng.ranges.len(), 7);
        // State 5: 20 floats, no gore.
        p.ai[0] = 5.0;
        let mut rng = census();
        celeb2_explosion(&p, &w, &mut rng);
        assert_eq!(rng.floats, 20);
        assert_eq!(rng.ranges.len(), 0);
        // State 6: the three wobble rings + one gore.
        p.ai[0] = 6.0;
        let mut rng = census();
        celeb2_explosion(&p, &w, &mut rng);
        let count = |step: f32| {
            let mut n = 0;
            let mut t = 0.0f32;
            while t < 1.0 {
                n += 1;
                t += step;
            }
            n
        };
        assert_eq!(rng.floats, count(0.015) + count(0.025) + count(0.07));
        assert_eq!(rng.ranges.len(), 7);
        // State 2: Next(4) + NextFloatDirection + one gore (both branches).
        p.ai[0] = 2.0;
        let mut rng = census();
        celeb2_explosion(&p, &w, &mut rng);
        assert_eq!(rng.floats, 1);
        assert_eq!(rng.ranges[0], (0, 4));
        assert_eq!(rng.ranges.len(), 8);
        // State 4: the discarded float + the 0.025 ring + three gores.
        p.ai[0] = 4.0;
        let mut rng = census();
        celeb2_explosion(&p, &w, &mut rng);
        assert_eq!(rng.floats, 1 + count(0.025));
        assert_eq!(rng.ranges.len(), 21);
    }

    /// The state-1 ground explosion (`:53401-53464`): one exposed solid
    /// tile inside the radius rolls amount + (amount − 1) floats per
    /// layer (4 layers) plus the Next(3) gore gate per layer. CensusRng's
    /// min = 0 makes `Next(3) != 0` FALSE — the census pins the refuse
    /// path: 4 layers × (3 + 2 floats + 1 range). The tile is dirt
    /// (amount 3) at the rocket's centre.
    #[test]
    fn s1_d_4_unarmed_the_ground_explosion_walks_the_layers() {
        let mut w = world();
        let mut p = proj(715);
        p.ai[0] = 1.0;
        // The resized death box (128×128) centred on tile (100, 50).
        p.pos = Vec2::new(100.0 * 16.0 - 64.0, 50.0 * 16.0 - 64.0);
        p.w = 128.0;
        p.h = 128.0;
        put(&mut w, 100, 50, 0); // dirt, exposed (the neighbours are air)
        let mut rng = census();
        create_ground_explosion(&p, &w, &mut rng, 40.0, 20, 100, 2, 10, 2);
        // Radius = 128/2 + 100 = 164: the centre tile is inside; the
        // fluffed box spans ±(4 + 20) tiles but only (100, 50) is solid.
        // 4 layers × (3 + 2) floats + 4 Next(3) gates (all refuse at min).
        assert_eq!(rng.floats, 4 * 5);
        assert_eq!(rng.ranges, vec![(0, 3); 4]);
    }

    /// The Kill resize (`:73632-73645`): the centre snap, the 128/240 box
    /// and the integer-half recentre.
    #[test]
    fn s1_d_4_unarmed_the_kill_resize_recentres() {
        let mut p = proj(715);
        let center = Vec2::new(p.pos.x + 7.0, p.pos.y + 7.0);
        celeb2_kill_resize(&mut p);
        assert_eq!((p.w, p.h), (128.0, 128.0));
        assert_eq!(p.pos, center - Vec2::splat(64.0));
        let mut big = proj(718);
        let center = Vec2::new(big.pos.x + 7.0, big.pos.y + 7.0);
        celeb2_kill_resize(&mut big);
        assert_eq!((big.w, big.h), (240.0, 240.0));
        assert_eq!(big.pos, center - Vec2::splat(120.0));
    }

    /// The 932 death census (`:71109-71147`): the discarded float, the
    /// ten `Next(1, 3)` steps (CensusRng min = 1 → counts 1,1,1,0,0,0,0,
    /// −1,−2,−2 after the ring marks), three floats per spawned dust.
    #[test]
    fn s1_d_4_unarmed_the_fairy_death_census() {
        let mut rng = census();
        fairy_queen_death_draws(&mut rng);
        assert_eq!(rng.ranges, vec![(1, 3); 10], "the ten trail steps");
        assert_eq!(rng.floats, 1 + 3 * 3, "the lead + 3 dusts × 3 floats");
    }

    /// The pool death hook: a rocket killed by the tile contact resizes
    /// to the 128 box, deals its death Damage through the resized box and
    /// rolls the state-0 explosion census (7 ranges).
    #[test]
    fn s1_d_4_unarmed_the_death_hook_resizes_and_bursts() {
        let mut w = world();
        // A stone wall one step ahead of the rocket — the kill_step dies.
        for y in 0..w.height {
            put(&mut w, 101, y, 1);
        }
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(715, 1600.0, 400.0, 8.0, 0.0, 40, 2.0)
            .unwrap();
        let mut rng = census();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        let p = &pool.slots()[slot];
        assert!(!p.active, "the tile contact killed it");
        assert_eq!((p.w, p.h), (128.0, 128.0), "the Kill resize (`:73636`)");
        // The state-0 gore row rode the stream (the AI's own draws: none).
        assert_eq!(
            rng.ranges,
            vec![
                (0, 100), (0, 100), (61, 64), (10, 31), (-20, 21), (-10, 11), (-10, 11)
            ],
            "the explosion census at death"
        );
    }

    /// The dispatch reaches all four styles (the wiring witness): one
    /// pool update advances each rowed kind's own counter — 145's ai[0],
    /// 715's localAI[0] (×3 sub-steps), 932's ai[0], 85's localAI[0]
    /// (×3 sub-steps).
    #[test]
    fn s1_d_4_unarmed_the_dispatch_arms_the_four_styles() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let s145 = pool.spawn_with_data(145, 1600.0, 400.0, 4.0, 0.0, 0, 0.0).unwrap();
        let s715 = pool.spawn_with_data(715, 1600.0, 300.0, 0.0, -8.0, 40, 2.0).unwrap();
        let s932 = pool.spawn_with_data(932, 1600.0, 350.0, 6.0, 0.0, 40, 2.0).unwrap();
        let s85 = pool.spawn_with_data(85, 1600.0, 450.0, 8.0, 0.0, 20, 2.0).unwrap();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            false,
        );
        assert_eq!(pool.slots()[s145].ai[0], 3.0, "31: 3 sub-steps");
        assert_eq!(pool.slots()[s715].local_ai[0], 3.0, "147: 3 sub-steps");
        assert_eq!(pool.slots()[s932].ai[0], 1.0, "181: 1 sub-step");
        assert_eq!(pool.slots()[s85].local_ai[0], 3.0, "193: 3 sub-steps");
        // The movement ran: every slot stepped off its spawn point (715
        // flies straight up — its X never moves).
        for (s, y) in [(s145, 400.0), (s715, 300.0), (s932, 350.0), (s85, 450.0)] {
            assert_ne!(
                pool.slots()[s].pos,
                Vec2::new(1600.0, y),
                "kind {} moved",
                pool.slots()[s].kind
            );
        }
    }

    // -----------------------------------------------------------------------
    // W4-d8 — the aiStyle-72/73/74/76 witnesses (WRITTEN NOT RUN, code-only).
    // Every expected value is hand-derived from the cited C# lines.
    // -----------------------------------------------------------------------

    /// Style 72 (`:30455-30479`): the age counter, the post-3-tick alpha
    /// fade to its 50 floor, the ×0.96 drag, the one-shot scale roll
    /// (CensusRng `Next(60,121)` returns 60 → 0.6) and the centre-anchored
    /// `(int)(14 * 0.6) = 8` box resize.
    #[test]
    fn w4_d8_the_bubble_fades_drags_and_resizes() {
        let mut p = proj(BUBBLE);
        p.vel = Vec2::new(10.0, 0.0);
        let mut rng = census();
        // The first three ticks: the counter advances, no fade yet.
        for _ in 0..3 {
            ai_bubble(&mut p, &mut rng);
        }
        assert_eq!(p.local_ai[0], 3.0, "the age counter");
        assert_eq!(p.alpha, 255, "no fade before the 4th tick (`:30458`)");
        assert_eq!(p.ai[1], 0.6, "the one-shot scale roll (`:30469`)");
        assert!(p.net_update, "the roll dirties netUpdate (`:30470`)");
        assert_eq!(p.scale, 0.6, "the scale follows the roll (`:30472`)");
        // The box: (int)(14 * 0.6) = 8, centre-anchored.
        assert_eq!((p.w, p.h), (8.0, 8.0), "(int)(14 * 0.6) (`:30476-30477`)");
        // The fourth tick: the fade starts (255 - 25 = 230).
        ai_bubble(&mut p, &mut rng);
        assert_eq!(p.alpha, 230, "the 4th tick fades 25 (`:30460`)");
        // The drag: ×0.96 per tick, four ticks → 10 * 0.96^4.
        let expect_v = 10.0f32 * 0.96 * 0.96 * 0.96 * 0.96;
        assert!((p.vel.x - expect_v).abs() < 1e-4, "the ×0.96 drag (`:30466`)");
        // The fade floor: run to the 50 floor (255 → 50 takes 9 fades).
        for _ in 0..20 {
            ai_bubble(&mut p, &mut rng);
        }
        assert_eq!(p.alpha, 50, "the fade floor (`:30461-30464`)");
        // The scale roll is one-shot: a second call never re-rolls.
        assert_eq!(rng.ranges.len(), 1, "one Next(60,121) for the life");
    }

    /// Style 73 (`:30482-30489`): the tile-338 anchor kill — a missing /
    /// inactive / non-338 tile kills the fountain; the anchored tile keeps
    /// it alive and rolls the per-kind dust census.
    #[test]
    fn w4_d8_the_fountain_kills_off_its_anchor_tile() {
        let mut w = world();
        // No tile at (100, 50): the fountain dies.
        let mut p = proj(419);
        p.ai[0] = 100.0;
        p.ai[1] = 50.0;
        let mut rng = census();
        ai_fountain(&mut p, &w, &mut rng);
        assert!(!p.active, "the missing tile kills (`:30485-30488`)");
        assert_eq!(p.time_left, 0, "Kill zeroes timeLeft");
        assert_eq!(rng.doubles, 0, "the kill precedes the dust draws");
        // A non-338 tile: still dies.
        put(&mut w, 100, 50, 1);
        let mut p2 = proj(419);
        p2.ai[0] = 100.0;
        p2.ai[1] = 50.0;
        ai_fountain(&mut p2, &w, &mut rng);
        assert!(!p2.active, "the non-338 tile kills (`:30485`)");
        // The 338 anchor: alive, the yellow's two-dust census runs.
        put(&mut w, 100, 50, 338);
        let mut p3 = proj(419);
        p3.ai[0] = 100.0;
        p3.ai[1] = 50.0;
        let mut rng3 = census();
        ai_fountain(&mut p3, &w, &mut rng3);
        assert!(p3.active, "the 338 anchor keeps it alive");
        // The yellow 419: two dusts × (3 NextDouble + 1 Next(-2,3)).
        assert_eq!(rng3.doubles, 6, "two dusts × 3 doubles (`:30500/30503`)");
        assert_eq!(rng3.ranges, vec![(-2, 3), (-2, 3)], "the position jitters");
    }

    /// Style 73 (`:30516-30603`): the red 420 / rainbow 422 add the
    /// `Next(5)` colour pick; the blue 421's `timeLeft % 10 == 0` burst
    /// draws the shared scale + nine per-dust pairs.
    #[test]
    fn w4_d8_the_fountain_per_kind_dust_census() {
        let mut w = world();
        put(&mut w, 100, 50, 338);
        // The red 420: two dusts × (3 doubles + Next(5) + Next(-2,3)).
        let mut red = proj(420);
        red.ai[0] = 100.0;
        red.ai[1] = 50.0;
        let mut rng = census();
        ai_fountain(&mut red, &w, &mut rng);
        assert_eq!(rng.doubles, 6, "the red's two dusts × 3 doubles");
        assert_eq!(rng.ranges, vec![(0, 5), (-2, 3), (0, 5), (-2, 3)],
            "the colour pick + jitter per dust (`:30522/30534`)");
        // The rainbow 422: the same shape (the colour offset).
        let mut rain = proj(422);
        rain.ai[0] = 100.0;
        rain.ai[1] = 50.0;
        let mut rng2 = census();
        ai_fountain(&mut rain, &w, &mut rng2);
        assert_eq!(rng2.doubles, 6, "the rainbow's two dusts × 3 doubles");
        assert_eq!(rng2.ranges, vec![(0, 5), (-2, 3), (0, 5), (-2, 3)],
            "the colour offset + jitter (`:30590/30598`)");
        // The blue 421 at timeLeft 600 (% 10 == 0): the two-dust loop +
        // the burst's shared scale + nine per-dust pairs.
        let mut blue = proj(421);
        blue.ai[0] = 100.0;
        blue.ai[1] = 50.0;
        assert_eq!(blue.time_left, 600, "the NewProjectile seed");
        let mut rng3 = census();
        ai_fountain(&mut blue, &w, &mut rng3);
        // 2×3 + 1 (shared) + 9×1 = 16 doubles; 2 + 9 = 11 jitters.
        assert_eq!(rng3.doubles, 16, "the blue's loop + burst (`:30546-30575`)");
        assert_eq!(rng3.ranges.len(), 11, "2 loop + 9 burst jitters");
        // The blue at timeLeft 601 (% 10 != 0): no burst.
        let mut blue2 = proj(421);
        blue2.ai[0] = 100.0;
        blue2.ai[1] = 50.0;
        blue2.time_left = 601;
        let mut rng4 = census();
        ai_fountain(&mut blue2, &w, &mut rng4);
        assert_eq!(rng4.doubles, 6, "no burst off the %10 tick");
        assert_eq!(rng4.ranges.len(), 2, "only the two loop jitters");
    }

    /// W4-d8 DOC-SWEEP (1) — the jet-speed math (`:30490-30495` +
    /// [`fountain_jet`]): `num569 = 2f` scales by `timeLeft / 60f` below 1;
    /// the jet vector is `(0, -num569) × (0.85 + NextDouble × 0.2)` spun by
    /// `(NextDouble − 0.5) × half_turn`. CensusRng's doubles return 0.0, so
    /// the scale pins 0.85 and the angle −half_turn/2 (−π/4 for the
    /// yellow's π/2 literal, −π/8 for the blue's π/4) — the float-cos/sin
    /// spin of `Utils.RotatedBy`.
    #[test]
    fn w4_d8_the_fountain_jet_speed_math() {
        let mut rng = census();
        // The full-speed jet (timeLeft ≥ 60): num569 = 2, scale 0.85 →
        // (0, -1.7) spun by -0.7851111111131433 rad (the C# π/2 literal
        // widened, × (0 - 0.5)). cos/sin are the f32 casts of the f64.
        let jet = fountain_jet(2.0, 1.5701111111162866, &mut rng);
        let a = -0.7851111111131433f64;
        let (c, s) = (a.cos() as f32, a.sin() as f32);
        // RotatedBy((0, -1.7)): x = 0*c - (-1.7)*s = 1.7*s (negative),
        // y = 0*s + (-1.7)*c = -1.7*c.
        assert!((jet.x - 1.7 * s).abs() < 1e-6, "the spun X (`:30500`)");
        assert!((jet.y - (-1.7 * c)).abs() < 1e-6, "the spun Y (`:30500`)");
        assert_eq!(rng.doubles, 2, "the speed scale + the rotation draw");
        // The blue's π/4 literal: the angle halves again (-0.3921111111165716).
        let mut rng2 = census();
        let jet_b = fountain_jet(2.0, 0.7851111111131433, &mut rng2);
        let ab = -0.3921111111165716f64;
        let (cb, sb) = (ab.cos() as f32, ab.sin() as f32);
        assert!((jet_b.x - 1.7 * sb).abs() < 1e-6, "the blue's spun X (`:30546`)");
        assert!((jet_b.y - (-1.7 * cb)).abs() < 1e-6, "the blue's spun Y");
        // The life-scaled speed: timeLeft 30 → num570 = 0.5 < 1 →
        // num569 = 2 * 0.5 = 1.0; the jet magnitude halves (scale 0.85).
        let mut p = proj(419);
        p.ai[0] = 100.0;
        p.ai[1] = 50.0;
        p.time_left = 30;
        let mut w = world();
        put(&mut w, 100, 50, 338);
        let mut rng3 = census();
        ai_fountain(&mut p, &w, &mut rng3);
        assert!(p.active, "the anchor holds");
        // The census count is unchanged by the math port (the draws were
        // already kept): two dusts × 3 doubles + 2 jitters.
        assert_eq!(rng3.doubles, 6, "the math port keeps the draw census");
        assert_eq!(rng3.ranges, vec![(-2, 3), (-2, 3)], "the jitters");
        // The scaled jet itself: (0, -1.0 * 0.85) spun by -π/4.
        let mut rng4 = census();
        let jet_s = fountain_jet(1.0, 1.5701111111162866, &mut rng4);
        assert!((jet_s.x - 0.85 * s).abs() < 1e-6, "the life-scaled X");
        assert!((jet_s.y - (-0.85 * c)).abs() < 1e-6, "the life-scaled Y");
        // The boundary: timeLeft 60 → num570 = 1.0, NOT < 1 → no scale.
        let mut p60 = proj(419);
        p60.ai[0] = 100.0;
        p60.ai[1] = 50.0;
        p60.time_left = 60;
        let mut rng5 = census();
        ai_fountain(&mut p60, &w, &mut rng5);
        assert!(p60.active);
        assert_eq!(rng5.doubles, 6, "the boundary tick still draws");
    }

    /// Style 74 (`:30605-30616` + `:17079-17086`): the charge latch seeds
    /// the decay pair on the wire's last sub-step; the decay arm shrinks
    /// `localAI[0]` by `localAI[1]` and kills below 0.01.
    #[test]
    fn w4_d8_the_scutlix_laser_latches_and_decays() {
        let mut p = proj(SCUTLIX_LASER_FRIENDLY);
        p.vel = Vec2::new(12.0, 0.0);
        // The latch: localAI[0] == 0 && numUpdates == 0.
        p.num_updates = 0;
        scutlix_laser_charge_latch(&mut p);
        assert_eq!(p.extra_updates, 1, "the latch flips extraUpdates (`:17081`)");
        assert_eq!(p.vel, Vec2::ZERO, "the velocity zeroes (`:17082`)");
        assert_eq!(p.local_ai[0], 1.0, "the decay seed (`:17083`)");
        assert_eq!(p.local_ai[1], 0.9999, "the decay factor (`:17084`)");
        assert!(p.net_update, "the latch dirties netUpdate (`:17085`)");
        // The latch is one-shot: a re-run never re-seeds.
        p.local_ai[0] = 0.5;
        scutlix_laser_charge_latch(&mut p);
        assert_eq!(p.local_ai[0], 0.5, "the latch holds (localAI[0] != 0)");
        // The decay arm: localAI[0] *= localAI[1], localAI[1] -= 0.001.
        let mut q = proj(SCUTLIX_LASER_FRIENDLY);
        q.extra_updates = 1;
        q.local_ai[0] = 1.0;
        q.local_ai[1] = 0.9999;
        ai_scutlix_laser(&mut q);
        assert!((q.local_ai[0] - 0.9999).abs() < 1e-6, "1 * 0.9999 (`:30609`)");
        assert!((q.local_ai[1] - 0.9989).abs() < 1e-6, "0.9999 - 0.001 (`:30610`)");
        assert!(q.active, "still above the 0.01 kill floor");
        // The kill: localAI[0] below 0.01.
        let mut r = proj(SCUTLIX_LASER_FRIENDLY);
        r.extra_updates = 1;
        r.local_ai[0] = 0.005;
        r.local_ai[1] = 0.5;
        ai_scutlix_laser(&mut r);
        assert!(!r.active, "the < 0.01 kill (`:30611-30614`)");
        // The decay arm is gated on extraUpdates == 1: the wire's 100
        // sub-steps never decay.
        let mut s = proj(SCUTLIX_LASER_FRIENDLY);
        s.local_ai[0] = 1.0;
        s.local_ai[1] = 0.9999;
        ai_scutlix_laser(&mut s);
        assert_eq!(s.local_ai[0], 1.0, "no decay before the latch (`:30607`)");
    }

    /// Style 76 (`:30621-30670`): the owner pin, the mount-type kill, the
    /// local-owner mouse aim + the channel release, the remote follow.
    #[test]
    fn w4_d8_the_crosshair_pins_aims_and_releases() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.x = 1000.0;
        pl.y = 500.0;
        pl.channel = true;
        let _ = pl.mount.set_mount(9, false); // the Scutlix
        // The local owner: the aim pins pos + the owner-relative ai pair.
        let mut p = proj(SCUTLIX_LASER_CROSSHAIR);
        ai_mount_crosshair(&mut p, 7, Some((1200.0, 600.0)), Some(&mut pl), 0);
        assert!(p.active, "the channel holds");
        assert!(!p.friendly, "the stock friendly false (`:788`)");
        assert_eq!(pl.held_proj, 7, "the owner pin (`:30624`)");
        assert_eq!(p.pos, Vec2::new(1200.0, 600.0), "the mouse aim (`:30640-30641`)");
        assert_eq!((p.ai[0], p.ai[1]), (200.0, 100.0), "the owner-relative pin");
        assert!(p.net_update, "the pin change dirties netUpdate (`:30644`)");
        // The channel release kills AND fires the two 606 beams (the
        // DOC-SWEEP (3) port: `UseAbility` `:30651` → the intent pair).
        pl.channel = false;
        ai_mount_crosshair(&mut p, 7, Some((1200.0, 600.0)), Some(&mut pl), 0);
        assert!(!p.active, "the release kills (`:30649-30653`)");
        assert_eq!(p.wants_scutlix_beams.len(), 2, "the two eye beams (`:3271`)");
        // The beams fly at 14 (the normalized aim × 14, `:3284`).
        for &(_, _, vx, vy) in &p.wants_scutlix_beams {
            let speed = (vx * vx + vy * vy).sqrt();
            assert!((speed - 14.0).abs() < 1e-3, "the ×14 beam speed");
        }
        // The aim wrote the facing (the cursor is right-down of the
        // player centre → facing +1, the frame bucket 9/10).
        assert_eq!(pl.facing, 1, "the AimAbility facing (`:30648`)");
        assert!(pl.mount.frame_extra() >= 8, "the head's aim frame");
        // The mount-type kill: a 441 on a non-Scutlix mount dies.
        let _ = pl.mount.set_mount(8, false); // the Drill, not the Scutlix
        pl.channel = true;
        let mut q = proj(SCUTLIX_LASER_CROSSHAIR);
        ai_mount_crosshair(&mut q, 3, Some((1200.0, 600.0)), Some(&mut pl), 0);
        assert!(!q.active, "the 441 needs mount 9 (`:30627-30631`)");
        assert_eq!(pl.held_proj, 3, "the pin precedes the mount kill");
        // The 453 on the Drill mount lives.
        let mut r = proj(DRILL_MOUNT_CROSSHAIR);
        ai_mount_crosshair(&mut r, 4, Some((1300.0, 700.0)), Some(&mut pl), 0);
        assert!(r.active, "the 453 needs mount 8 (`:30633`)");
        // W4-d8 FINAL-PHASE (1): the local 453 aim writes the diode target +
        // the crosshair pin (`Mount.cs:5466-5494`). The cursor (1300, 700)
        // is right-down of the player centre → a positive wrapped angle.
        assert_eq!(
            pl.mount.drill_data().crosshair_position,
            Vec2::new(1300.0, 700.0),
            "the raw mouse pin (`:5493`)"
        );
        assert!(
            pl.mount.drill_data().diode_rotation_target > 0.0,
            "the right-down cursor's wrapped angle"
        );
        // The 453 channel release clears the mining flag (`:30651` →
        // `Mount.cs:3317-3320`) and kills the crosshair.
        pl.mount.set_ability_active(true);
        pl.channel = false;
        ai_mount_crosshair(&mut r, 4, Some((1300.0, 700.0)), Some(&mut pl), 0);
        assert!(!r.active, "the release kills (`:30652`)");
        assert!(!pl.mount.ability_active(), "the Drill flag clears (`:3319`)");
        pl.channel = true;
        // The remote owner: the position follows the stored offset.
        let mut s = proj(SCUTLIX_LASER_CROSSHAIR);
        let _ = pl.mount.set_mount(9, false);
        s.owner = 1; // not the local owner (0)
        s.ai[0] = 50.0;
        s.ai[1] = 60.0;
        ai_mount_crosshair(&mut s, 5, None, Some(&mut pl), 0);
        assert!(s.active, "the remote follow never releases");
        assert_eq!(s.pos, Vec2::new(1050.0, 560.0), "pl.pos + ai (`:30656-30657`)");
        // The remote 441 sets the charge flag (`:30660-30663` →
        // `Mount.cs:2916-2919`).
        assert!(pl.mount.ability_charging(), "the remote StartAbilityCharge");
    }

    /// W4-d8 DOC-SWEEP (2) — the Bubble family's `Kill()` pop
    /// (`:72849-72873` the 405, `:72919-72943` the 410): the Item54 cue
    /// (2, 54) at the slot position, then the ring's per-dust hue draw —
    /// 20 for the Flairon bubble, 10 for the Bubble Gun's. The
    /// `num < 30` velocity arm is always taken (both loops cap below 30),
    /// so NO `Next(45, 91)` range draw ever fires.
    #[test]
    fn w4_d8_the_bubble_pop_cue_and_hue_census() {
        // The 410: 10 hue doubles, the cue at the top-left position.
        let mut p = proj(BUBBLE);
        p.pos = Vec2::new(300.0, 200.0);
        let mut rng = census();
        bubble_pop_kill(&mut p, &mut rng);
        assert_eq!(p.wants_sound, Some((2, 54, 300.0, 200.0, 0.0)),
            "the Item54 pop (`:72921`, SoundID.cs:1437)");
        assert_eq!(rng.doubles, 10, "the 410's ten hue draws (`:72923/72938`)");
        assert!(rng.ranges.is_empty(), "the Next(45,91) else is dead (`:72930`)");
        assert!(rng.floats == 0, "no float draws in the pop");
        // The 405 (the d7 Flairon lane's twin arm): 20 hue doubles.
        let mut q = proj(FLAIRON_BUBBLE);
        q.pos = Vec2::new(400.0, 100.0);
        let mut rng2 = census();
        bubble_pop_kill(&mut q, &mut rng2);
        assert_eq!(q.wants_sound, Some((2, 54, 400.0, 100.0, 0.0)),
            "the 405's Item54 pop (`:72851`)");
        assert_eq!(rng2.doubles, 20, "the 405's twenty hue draws (`:72853/72868`)");
        assert!(rng2.ranges.is_empty(), "the 405's else is dead too (`:72860`)");
        // A non-bubble kind: no cue, no draws.
        let mut r = proj(SCUTLIX_LASER_FRIENDLY);
        let mut rng3 = census();
        bubble_pop_kill(&mut r, &mut rng3);
        assert_eq!(r.wants_sound, None);
        assert_eq!(rng3.doubles, 0);
    }
}
