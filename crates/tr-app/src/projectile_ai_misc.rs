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
    Projectile, can_hit, chaseable, chaseable_held, csign, get_lerp_value, rotate, safe_normalize,
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
// W4-d16 — projectile aiStyles 151-160 MINUS 156/164 (task/W4-a9 owns those
// two). The census found 157/158 already armed (projectile.rs's
// `ai_sharp_tears`/`ai_baby_bird` — verify-only) and 151/152/153/154/155/
// 159/160 MISSING: no rows, no arms. The rows land in projectile/data.rs
// (cited per field); this module owns the machines, one style per commit.
//
// RNG contract (the `projectile_ai_misc` S1-D-4 precedent above): every
// EXPLICIT `Main.rand` call-site draw stays in C# order (gates included,
// with the C#'s `&&` short-circuits); the `Dust.NewDust*` bodies + their
// internal rolls go with the dust layer; the `Gore.NewGore` kicks
// `Next(10, 31)` / `Next(-20, 21)` stay (`Gore.cs:1327-1328`).
//
// CODE-ONLY MODE (W4-d16): the witness tests are WRITTEN NOT RUN.

/// `AI_151_SuperStar` (`Projectile.cs:22138-22168`) — the Super Star
/// Cannon's star (728). The −10 alpha ramp to the 100 floor, the Item9
/// cue's `20 + Next(40)` re-arm, the `(|vx|+|vy|)·0.005·direction` spin,
/// the on-screen 1-in-6 gore gate (the `SelectRandom(16,17,17,17)` pick +
/// the NewGore kicks) and the two 1-in-8 dust gates (bodies skipped).
///
/// `screen` is the pool's per-tick view snapshot `(centre, size)` —
/// `Main.screenPosition + (screenWidth, screenHeight)/2` and the size
/// (`:22152-22153`); `None` folds the hitbox test false (the C#'s
/// off-screen short-circuit spends NO `Next(6)`).
///
/// `direction` is the velocity-sign stand-in (`AutomaticallyChangesDirection`
/// is true for style 151, `:17283` — the `ai_spray` precedent).
pub(crate) fn ai_super_star(
    p: &mut Projectile,
    rng: &mut dyn DropRng,
    screen: Option<(Vec2, Vec2)>,
) {
    // `:22140-22145` — the alpha ramp and its floor.
    p.alpha -= 10;
    if p.alpha < 100 {
        p.alpha = 100;
    }
    // `:22146-22150` — the cue re-arm (the pool decrements soundDelay
    // ahead of the AI, `:16768-16771`; the `-1` latch convention holds).
    if p.sound_delay == 0 {
        p.sound_delay = 20 + rng.next_range(0, 40);
        p.wants_sound = Some((2, 9, p.pos.x, p.pos.y, 0.0));
    }
    // `:22151` — the spin.
    let direction = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
    p.rot += (p.vel.x.abs() + p.vel.y.abs()) * 0.005 * direction;
    // `:22152-22156` — the on-screen gore gate: the hitbox against the
    // screen rect grown 400 px (`Utils.CenteredRectangle`), THEN the
    // `Next(6)` (the `&&` order — off-screen spends nothing).
    let on_screen = screen.is_some_and(|(centre, size)| {
        let half = (size + Vec2::splat(400.0)) * 0.5;
        let (l, t) = (centre.x - half.x, centre.y - half.y);
        let (r, b) = (centre.x + half.x, centre.y + half.y);
        // XNA `Rectangle.Intersects` — the strict edge comparisons.
        p.pos.x < r && l < p.pos.x + p.w && p.pos.y < b && t < p.pos.y + p.h
    });
    if on_screen && rng.next_range(0, 6) == 0 {
        // `Utils.SelectRandom(Main.rand, 16, 17, 17, 17)` — one `Next(4)`
        // (`Utils.cs:2459-2462`); the gore body is skipped, the NewGore
        // kicks stay (`Gore.cs:1327-1328`).
        let _ = rng.next_range(0, 4);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
    }
    // `:22157-22167` — the two dust gates: the `Next(8)` draws stay, the
    // `NewDustDirect` bodies + their internal rolls go with the dust
    // layer (the velocity blend is deterministic — no draws inside).
    for _ in 0..2 {
        if rng.next_range(0, 8) == 0 {
            // The dust spawn + its ×0.25 / rotated-velocity blend — SKIPPED.
        }
    }
}

/// `AI_152_SuperStarSlash` (`Projectile.cs:22230-22351`) — the four
/// aiStyle-152 slashes: the Super Star Cannon's 729, the Zenith's
/// Muramasa 977 + Blade-of-Grass 976, the Moon Lord whip's proc 1045.
///
/// The alpha ladder (the 976 skips it; the 1045 ramps −50 to 0, the rest
/// −10 to 100), the per-kind cue latch (977/1045 Item1 once, 976 silent,
/// the rest the `20 + Next(40)` Item9 re-arm), the `ai[0]` velocity spin
/// (the 976 spins the FULL angle, the rest `ai[0] / (num4·MaxUpdates)`),
/// the 977/1045 ×0.96 drag + 1-in-8 centre dust, the 976's lava kill +
/// 7-frame cycle + the Remap speed/opacity envelope + the one-shot
/// 10-dust burst, and the shared `rotation = velocity.ToRotation() + num`
/// + `tileCollide = false` tail.
///
/// RNG: the cue `Next(40)` (the else arm only), the 977/1045 dust gates'
/// `Next(8)` + the taken branch's `NextVector2Circular(1,1)` pair (TWO
/// floats, `Utils.cs:1266-1269` — the dust body's velocity is REWRITTEN
/// from it, so the draws are call-site state), the 976's `Next(3)` gate
/// and the one-shot burst's per-dust `NextFloat` pair (the rotated
/// velocity's angle + speed, `:22345`). The `NewDustDirect` bodies +
/// their internal rolls go with the dust layer.
pub(crate) fn ai_super_star_slash(p: &mut Projectile, world: &World, rng: &mut dyn DropRng) {
    use crate::projectile::{MOON_LORD_WHIP_PROC, ZENITH_GRASS_SLASH, ZENITH_MURAMASA_SLASH, alpha_of};
    // `:22232-22237` — the rotation offset and the alpha-ladder flag.
    let mut num = std::f32::consts::FRAC_PI_2;
    let flag = p.kind != ZENITH_GRASS_SLASH;
    // `:22238-22252` — the alpha ladder.
    if flag {
        let (mut num2, mut num3) = (10, 100);
        if p.kind == MOON_LORD_WHIP_PROC {
            num3 = 0;
            num2 = 50;
        }
        p.alpha -= num2;
        if p.alpha < num3 {
            p.alpha = num3;
        }
    }
    // `:22253-22257` — the spin divisor base.
    let num4 = if p.kind == MOON_LORD_WHIP_PROC { 24 } else { 10 };
    // `:22258-22279` — the cue ladder.
    if p.sound_delay == 0 {
        if p.kind == ZENITH_MURAMASA_SLASH || p.kind == MOON_LORD_WHIP_PROC {
            p.sound_delay = -1;
            p.wants_sound = Some((2, 1, p.pos.x, p.pos.y, 0.0));
        } else if p.kind == ZENITH_GRASS_SLASH {
            p.sound_delay = -1; // the silent latch (`:22270-22273`)
        } else {
            p.sound_delay = 20 + rng.next_range(0, 40);
            p.wants_sound = Some((2, 9, p.pos.x, p.pos.y, 0.0));
        }
    }
    // `:22280-22291` — the ai[0] spin.
    if p.ai[0] != 0.0 {
        if p.kind == ZENITH_GRASS_SLASH {
            p.vel = crate::bow_launch::rotated_by(p.vel, p.ai[0] as f64);
        } else {
            let num5 = num4 * (p.extra_updates + 1) as i32;
            p.vel = crate::bow_launch::rotated_by(p.vel, (p.ai[0] / num5 as f32) as f64);
        }
    }
    // `:22292-22313` — the 977/1045 drag + centre dust (the two kinds
    // differ only in the dust TYPE — 172 vs 229 — the draws are the same).
    if p.kind == ZENITH_MURAMASA_SLASH || p.kind == MOON_LORD_WHIP_PROC {
        p.vel *= 0.96;
        if rng.next_range(0, 8) == 0 {
            // The dust body is skipped; its REWRITTEN velocity reads
            // `NextVector2Circular(1,1)` — two floats stay on the stream.
            let _ = (rng.next_float(), rng.next_float());
        }
    }
    // `:22314-22348` — the 976's machine.
    if p.kind == ZENITH_GRASS_SLASH {
        // `:22316-22320` — `Collision.LavaCollision` over the box.
        if lava_collision(world, p.pos, p.w, p.h) {
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `:22321-22328` — the 2-tick frame cycle over the 7-frame sheet.
        p.frame_counter += 1;
        if p.frame_counter >= 2 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 7 {
                p.frame = 0;
            }
        }
        // `:22329-22332` — the age envelope: the speed Remap pair and the
        // opacity fade-in/out (the `Opacity` property is the alpha view).
        let from_value = 60.0 - p.time_left as f32;
        let num6 = remap_clamped(from_value, p.ai[1], p.ai[1] + 20.0, 0.0, 1.0)
            * remap_clamped(from_value, p.ai[1], p.ai[1] + 60.0, 1.0, 0.0);
        let unit = crate::bow_launch::safe_normalize(p.vel, Vec2::Y);
        p.vel = unit * (4.0 + 12.0 * (1.0 - num6));
        let opacity = remap_clamped(from_value, 0.0, 10.0, 0.0, 1.0)
            * remap_clamped(from_value, 30.0, 60.0, 1.0, 0.0);
        p.alpha = alpha_of(opacity);
        num = 0.0;
        // `:22334-22339` — the 1-in-3 trail dust (no draws inside the
        // body — the velocity is `base.velocity * 0.5f`).
        if rng.next_range(0, 3) == 0 {
            // The dust spawn — SKIPPED.
        }
        // `:22340-22347` — the one-shot 10-dust burst: per dust the
        // rotated velocity's angle `NextFloat` + speed `NextFloat`
        // (`RotatedBy(NextFloat() * 2π * 0.25) * (NextFloat() * 3)`).
        if p.local_ai[1] == 0.0 {
            p.local_ai[1] = 1.0;
            for _ in 0..10 {
                let _ = rng.next_float(); // the angle
                let _ = rng.next_float(); // the speed
            }
        }
    }
    // `:22349-22350` — the shared tail.
    p.rot = p.vel.y.atan2(p.vel.x) + num;
    p.tile_collide = false;
}

/// `Utils.Remap` (`Utils.cs:345-348`, clamped by default) over the port's
/// `get_lerp_value` — XNA's `a + (b - a) * amount` lerp.
fn remap_clamped(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * get_lerp_value(from_min, from_max, from_value)
}

/// `AI_153_ToiletPop` (`Projectile.cs:52198-52236`) — the toilet's pop
/// effect (733). The first tick (`ai[0] == 0`) spawns 6 dust + 1 gore;
/// ticks 1-10 spawn 2 dust + a 1-in-5 gore; past 10 the effect dies.
/// The counter advances AFTER the branch pick (`:52222`).
///
/// RNG: the else branch's `Next(5)` gate (`:52216`, ONLY when
/// `ai[0] <= 10` — the kill returns first); every spawned gore's
/// `NextVector2Circular(1,1)` pair (two floats, `:52233` — the velocity
/// argument `UnitY * -3f + NextVector2Circular(1f, 1f)` is evaluated
/// BEFORE the call, on the first tick too) + the `Next(580,583)` type
/// roll + the NewGore kicks `Next(10,31)` / `Next(-20,21)`
/// (`Gore.cs:1327-1328`). The dust bodies (`:52223-52230`) roll nothing
/// explicit.
pub(crate) fn ai_toilet_pop(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:52200-52221` — the branch pick (the centre/speedY feed only the
    // skipped dust bodies).
    let (num, mut num2) = if p.ai[0] == 0.0 {
        (6, 1)
    } else {
        if p.ai[0] > 10.0 {
            // `:52211-52215` — the kill (the C# `Kill()`; the port's
            // death convention zeroes time_left).
            p.active = false;
            p.time_left = 0;
            return;
        }
        let mut g = 0;
        if rng.next_range(0, 5) == 0 {
            g = 1;
        }
        (2, g)
    };
    // `:52222` — the counter.
    p.ai[0] += 1.0;
    // `:52223-52230` — the dust loop: `num` bodies, no explicit draws.
    let _ = num;
    // `:52231-52235` — the gore loop: `num2` gores, each the circular
    // velocity pair + the type roll + the NewGore kicks.
    for _ in 0..num2 {
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular(1,1)
        let _ = rng.next_range(580, 583); // the gore type
        let _ = rng.next_range(10, 31); // NewGore's Y kick
        let _ = rng.next_range(-20, 21); // NewGore's X kick
    }
}

/// `AI_154_VoidLens` (`Projectile.cs:52154-52196`) — the Void Lens'
/// placed lens (734). The owner-gated same-type dedupe (the YOUNGER lens
/// kills the older siblings; an older sibling kills THIS lens and
/// returns), the ×0.94 velocity damp, the direction pin, the 5-tick
/// 8-frame cycle, the +0.03 opacity climb and the `VoidLensHelper.Update`
/// dust gate.
///
/// The dedupe scans the LIVE pool (the C# reads `Main.projectile` while
/// killing), so this half is POOL-SIDE ([`crate::projectile::ProjectilePool`]'s
/// dispatch, the `dragon_step` shape); this fn is the post-dedupe body.
/// `true` = the lens survived the dedupe (the caller runs the body).
///
/// RNG (`VoidLensHelper.SpawnVoidLensDust`, `VoidLensHelper.cs:40-75`):
/// the `Next(3)` gate, then the `Next(2)` branch pick; EACH branch's
/// `RotatedByRandom(2π)` pair (two doubles — one float step each), the
/// `SelectRandom(86, 88)` pick `Next(2)`, the position's `Next(10,21)` /
/// `Next(5,10)` and the scale `NextFloat`. The dust bodies are skipped.
/// `Lighting.AddLight` (`:36`) is the light layer's skip (no draws).
pub(crate) fn ai_void_lens(p: &mut Projectile, rng: &mut dyn DropRng) {
    use crate::projectile::{alpha_of, opacity_of, proj_frames};
    // `:52177-52178` — the damp and the direction pin.
    p.vel *= 0.94;
    p.direction = 1;
    // `:52179-52186` — the 5-tick frame cycle over projFrames[734] = 8.
    p.frame_counter += 1;
    if p.frame_counter >= 5 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= proj_frames(p.kind) as i32 {
            p.frame = 0;
        }
    }
    // `:52187-52194` — the opacity climb (the alpha view).
    if opacity_of(p.alpha) < 1.0 {
        let mut opacity = opacity_of(p.alpha) + 0.03;
        if opacity > 1.0 {
            opacity = 1.0;
        }
        p.alpha = alpha_of(opacity);
    }
    // `:52195` → `VoidLensHelper.Update` → `SpawnVoidLensDust`.
    if rng.next_range(0, 3) == 0 {
        if rng.next_range(0, 2) == 0 {
            // `:46-53` — the first branch.
            let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom(2π)
            let _ = rng.next_range(0, 2); // SelectRandom(86, 88)
            let _ = rng.next_range(10, 21); // the position radius
            let _ = rng.next_float(); // the scale
        } else {
            // `:61-68` — the second branch (the 5-10 radius).
            let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom(2π)
            let _ = rng.next_range(0, 2); // SelectRandom(86, 88)
            let _ = rng.next_range(5, 10); // the position radius
            let _ = rng.next_float(); // the scale
        }
    }
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world_fluff(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.TileEmpty` (`WorldGen.cs:69976-69983`): no tile / inactive
/// flag clear / actuated (`inActive`).
fn tile_empty(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        Some(t) if t.active() => t.inactive(),
        _ => true,
    }
}

/// `AI_155_RemoveRope` (`Projectile.cs:52114-52129`): the start→end
/// column walk (descending y), killing every active 504 MysticSnakeRope
/// cell. The net SendData is the MP skip.
fn snake_remove_rope(world: &mut World, start: (i32, i32), end: (i32, i32)) {
    let x = start.0;
    let mut y = start.1;
    while y >= end.1 {
        if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 504) {
            let _ = tr_core::edit::kill_tile(world, x, y, false);
        }
        y -= 1;
    }
}

/// `AI_155_SpawnRopeIn` (`Projectile.cs:52131-52152`) — the per-cell
/// conversion burst: the 3-in-4 gore gate (`Next(4) != 0` → the
/// `Next(61,64)` type + the NewGore kicks + the `NextVector2Circular
/// (1.5,0.8)` velocity pair — the unit angle + the radius,
/// `Utils.cs:1261-1268`), then the 5-dust loop (each the `Next(8)`
/// gate; the bodies roll nothing explicit). The call-site draw order:
/// the type feeds `NewGoreDirect`'s argument (rolled FIRST), the kicks
/// run INSIDE `NewGore` (`Gore.cs:1327-1328`), the circular velocity is
/// the assignment AFTER the call (`:52137`).
fn snake_spawn_rope_in(rng: &mut dyn DropRng) {
    // `:52134-52141` — the gore gate.
    if rng.next_range(0, 4) != 0 {
        let _ = rng.next_range(61, 64); // the gore type (`:52136`'s argument)
        let _ = rng.next_range(10, 31); // NewGore's Y kick
        let _ = rng.next_range(-20, 21); // NewGore's X kick
        // `:52137` — NextVector2Circular(1.5, 0.8): the unit angle's
        // NextFloat, then the radius NextFloat (`Utils.cs:1266-1268`).
        let _ = (rng.next_float(), rng.next_float());
    }
    // `:52142-52151` — the 5-dust loop.
    for _ in 0..5 {
        let _ = rng.next_range(0, 8); // the first dust's gate
        // Both dust bodies — SKIPPED (no explicit draws inside).
    }
}

/// `AI_155_MysticSnakeCoil` (`Projectile.cs:51939-52112`) — the Mystic
/// Snake's rope coil (754): the orphan kill (+ rope removal), the launch
/// latch (spriteDirection from the owner's centre, `velocity.Y = -5`),
/// the rise machine (the localAI[1] counter, the 15→30 Remap ease into
/// the −32 launch at 60, the int truncation), the 60-tick 30-dust burst,
/// the 1-in-6 trail dust gate (the roll is the `||`'s LEFT operand — it
/// runs every rise tick; `flag` suppresses only the body), the
/// −5 alpha ramp, the 12-tick 4-frame cycle, the owner-gated rope
/// placement walk (the 100-cell cap, the InWorld-40 gate, the
/// solid-non-cut-non-504 blocker scan, the PlaceTile-504 column fill,
/// the snap-to-anchor + velocity zero), then the attached state's
/// per-cell rope conversion (the one-shot latch / the 1-in-80 dust walk
/// / the timeLeft-4 re-burst) and the ≤4 death (the InWorld-20 gate, the
/// owner-gated rope removal).
///
/// `owner_center` is `Main.player[owner].Center` (the dispatch's owner
/// snapshot); `owner_active` folds `Main.player[owner].active` (the
/// port's single-player stand-in: the resolved player exists). The
/// `Main.netMode != 1` head gate is always true in the port.
///
/// Movement: the C# `HandleMovement` integrates after the AI; the
/// dispatch's `kill_step` is the free integrate (tileCollide false).
pub(crate) fn ai_mystic_snake_coil(
    p: &mut Projectile,
    world: &mut World,
    owner_center: Option<Vec2>,
    owner_active: bool,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:51941-51949` — the orphan kill: remove the rope column, die.
    if !owner_active {
        let num = p.ai[0];
        let y = p.pos.y;
        let cx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
        // (`point.Y` — the centre's tile row — is computed in the C# but
        // only its X feeds the removal points, `:51945-51946`.)
        snake_remove_rope(world, (cx, num as i32 / 16), (cx, y as i32 / 16));
        p.active = false;
        p.time_left = 0;
        return;
    }
    let owner_center = owner_center.unwrap_or(Vec2::ZERO);
    // `:51950-51957` — the launch latch.
    let num2 = 100;
    let num3 = 1800;
    if p.ai[1] == 0.0 {
        let pcx = owner_center.x;
        let scx = p.pos.x + p.w * 0.5;
        p.sprite_direction = if !(pcx > scx) { 1 } else { -1 };
        p.ai[1] = 1.0;
        p.vel.y = -5.0;
    }
    // `:51958-52047` — the rise state.
    if p.ai[1] == 1.0 {
        p.local_ai[1] += 1.0;
        // `:51961-51964` — the ease + the launch, then the truncation.
        let num4 = remap_clamped(p.local_ai[1], 15.0, 30.0, 0.0, 1.0);
        let num5 = if p.local_ai[1] >= 60.0 { 1 } else { 0 };
        p.vel.y = -3.0 * (1.0 - num4) + (-32 * num5) as f32;
        p.vel.y = p.vel.y as i32 as f32;
        // `:51965-51982` — the 60-tick burst: 30 dust, each the
        // NextFloatDirection X jitter + the rotated-velocity pair
        // (NextFloatDirection × 0 — the ANGLE draw still happens) + the
        // conditional NextFloat Y scale. The dust bodies are skipped;
        // the explicit call-site draws stay.
        if p.local_ai[1] == 60.0 {
            for _ in 0..30 {
                let _ = rng.next_float(); // NextFloatDirection (`:51975`)
                let _ = rng.next_float(); // the RotatedBy angle's NextFloatDirection (`:51976`)
                let _ = rng.next_float(); // the speed's NextFloatDirection (`:51976`)
                // The rotated (0,-1) vector's Y is -cos(0) = -1 < 0 for
                // EVERY draw (the angle is ×0f) — the conditional scale
                // ALWAYS fires (`:51977-51980`).
                let _ = rng.next_float();
            }
        }
        // `:51983-51992` — the trail dust gate: `flag = num4 > 0 &&
        // num5 <= 0`; `(Next(6) == 0 || num5 > 0) && !flag` — the roll is
        // the LEFT operand, so it runs EVERY rise tick (num5 > 0 only
        // short-circuits the right operand when the roll failed; flag
        // suppresses the body, never the roll). The body is skipped.
        let flag = num4 > 0.0 && num5 <= 0;
        let rolled = rng.next_range(0, 6) == 0;
        let gate = rolled || num5 > 0;
        if gate && !flag {
            // The dust spawn — SKIPPED (no explicit draws inside).
        }
        // `:51993` — the alpha ramp.
        p.alpha = (p.alpha - 5).clamp(0, 255);
        // `:51994-52001` — the 12-tick 4-frame cycle.
        p.frame_counter += 1;
        if p.frame_counter >= 12 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 4 {
                p.frame = 0;
            }
        }
        // `:52002-52046` — the owner-gated rope placement.
        if local_owner == p.owner {
            let num6 = p.ai[0];
            let y2 = p.pos.y;
            let cx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
            // (`point2.Y` is computed but only its X feeds the points.)
            let point3 = (cx, y2 as i32 / 16);
            let point4 = (cx, num6 as i32 / 16);
            let mut flag2 = point4.1 - point3.1 >= num2;
            let x = point4.0;
            if !in_world_fluff(world, x, point3.1, 40) || !in_world_fluff(world, x, point4.1, 40) {
                flag2 = true;
            }
            if !flag2 {
                let mut y = point4.1;
                while y >= point3.1 {
                    if world.tile(x, y).is_some_and(|t| {
                        t.active() && !tr_core::tileobject::tile_cut(t.kind) && t.kind != 504
                    }) {
                        flag2 = true;
                        break;
                    }
                    y -= 1;
                }
            }
            if flag2 {
                // `:52028-52039` — the column fill (the count is unused
                // state in the C# — `num8` feeds nothing).
                let mut y = point4.1;
                while y > point3.1 {
                    let empty_or_cut = tile_empty(world, x, y)
                        || world.tile(x, y).is_some_and(|t| tr_core::tileobject::tile_cut(t.kind));
                    if empty_or_cut && tr_core::edit::place_tile(world, x, y, 504) {
                        // The net SendData — the MP skip.
                    }
                    y -= 1;
                }
                // `:52040-52044` — the attach: the life re-arm, the
                // state flip, the snap (Top = the anchor cell's bottom
                // centre) and the velocity zero.
                p.time_left = num3;
                p.ai[1] = 2.0;
                p.net_update = true;
                p.pos = Vec2::new(x as f32 * 16.0 + 8.0 - p.w * 0.5, point3.1 as f32 * 16.0 + 16.0);
                p.vel = Vec2::ZERO;
            }
        }
    }
    // `:52048-52051` — only the attached state continues.
    if p.ai[1] != 2.0 {
        return;
    }
    // `:52052-52059` — the attached head.
    p.alpha = 0;
    let num10 = p.ai[0];
    let y3 = p.pos.y;
    let cx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
    // (`point5.Y` is computed but only its X feeds the points.)
    let end = (cx, y3 as i32 / 16);
    let start = (cx, num10 as i32 / 16);
    let x2 = start.0;
    // `:52060-52086` — the one-shot conversion / the per-tick dust walk.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        let mut y = start.1;
        while y >= end.1 {
            if world.tile(x2, y).is_some_and(|t| t.active() && t.kind == 504) {
                snake_spawn_rope_in(rng);
            }
            y -= 1;
        }
    } else {
        let mut y = start.1;
        while y >= end.1 {
            if world.tile(x2, y).is_some_and(|t| t.active() && t.kind == 504)
                && rng.next_range(0, 80) == 0
            {
                // The dust spawn — SKIPPED (no explicit draws inside).
            }
            y -= 1;
        }
    }
    // `:52087-52097` — the timeLeft-4 re-burst.
    if p.time_left == 4 {
        let mut y = start.1;
        while y >= end.1 {
            if world.tile(x2, y).is_some_and(|t| t.active() && t.kind == 504) {
                snake_spawn_rope_in(rng);
            }
            y -= 1;
        }
    }
    // `:52098-52111` — the death window.
    if p.time_left > 4 {
        return;
    }
    if !in_world_fluff(world, x2, end.1, 20) || !in_world_fluff(world, x2, start.1, 20) {
        p.active = false;
        p.time_left = 0;
        return;
    }
    if local_owner == p.owner {
        snake_remove_rope(world, start, end);
    }
    p.active = false;
    p.time_left = 0;
}

/// `AI_159_PaperAirplanes` (`Projectile.cs:48695-48755`) — the thrown
/// paper gliders (761/762): the one-shot launch latch (the velocity-sign
/// direction, the velocity rotation, `ai[0] = -Next(30,80)`), the wet
/// kill (the owner-gated `base.wet && owner == myPlayer` — the port's
/// wet probe is the tick-head stand-in, the bobber precedent), the glide
/// counter, the wind-borne sine bob (`timeForVisualEffects % 90`), the
/// 20..69 downwind loop window (the `GetLerpValue(0,30)` eased
/// `−direction·2π·0.02` bank), the 70 re-arm (`-Next(120,600)`), the
/// speed-preserving `SafeNormalize(UnitY)` steer, and the non-loop
/// float (the `Lerp(0.15,0.05,|wind|)` bob step on the `timeLeft % 40`
/// half-cycle, the ±2 Y clamp, the `wind·0.006` X drift clamped ±6, the
/// X-sign flip re-arm `-Next(120,300)`).
///
/// RNG: the three `Main.rand.Next` draws are the glide TIMERS (`ai[0]`),
/// state-bearing — the launch `Next(30,80)`, the 70 re-arm
/// `Next(120,600)`, the flip re-arm `Next(120,300)`. No dust/gore bodies.
///
/// `wind` is `Main.WindForVisuals` (= `windSpeedCurrent`, `Main.cs:2823`);
/// `visual_time` is `Main.timeForVisualEffects`; `old_vel_x` is
/// `base.oldVelocity.X` (the sub-step's start velocity, `:16766`).
/// Movement: the C# `HandleMovement` integrates after the AI; the row's
/// tileCollide true + the blocked-move chain's default (`:19849-19853`)
/// is the dispatch's `kill_step` (die on contact). `manualDirectionChange`
/// (`:7853`) suppresses the post-move auto-flip (`:17283`) — the arm owns
/// `direction` alone.
pub(crate) fn ai_paper_airplanes(
    p: &mut Projectile,
    rng: &mut dyn DropRng,
    wind: f32,
    visual_time: f64,
    local_owner: u8,
    old_vel_x: f32,
) {
    // `:48697-48704` — the launch latch.
    if p.ai[1] == 0.0 {
        p.direction = if p.vel.x > 0.0 { 1 } else { -1 };
        p.rot = p.vel.y.atan2(p.vel.x);
        p.ai[1] = 1.0;
        p.ai[0] = -rng.next_range(30, 80) as f32;
        p.net_update = true;
    }
    // `:48705-48708` — the wet kill (the owner gate folds to the
    // simulated-slot set; the port probes wet at the tick head).
    if p.wet && p.owner == local_owner {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:48709` — the glide counter.
    p.ai[0] += 1.0;
    // `:48710-48712` — the wind-borne bob. `vector` is the rotation unit
    // ×8; `v` adds the wind X and the sine Y (`direction · WindForVisuals`).
    // The C# association: the double `% 90 / 90` casts to float, the
    // `(float)π * 2f * phase` is float math, `Math.Sin` widens to double
    // and the result casts back (`:48711`).
    let vector = Vec2::new(p.rot.cos(), p.rot.sin()) * 8.0;
    let phase = ((visual_time % 90.0) / 90.0) as f32;
    let bob = ((std::f32::consts::PI * 2.0 * phase) as f64).sin() as f32;
    let mut v = vector + Vec2::new(wind, bob * p.direction as f32 * wind);
    // `:48713-48714` — the downwind flag and the loop window.
    let flag = p.direction as f32 == csign(wind) && p.vel.length() > 3.0;
    let num = p.ai[0] >= 20.0 && p.ai[0] <= 69.0;
    // `:48715-48718` — the 70 re-arm.
    if p.ai[0] == 70.0 {
        p.ai[0] = -rng.next_range(120, 600) as f32;
    }
    // `:48719-48724` — the loop bank (the C# `&` is non-short-circuit, but
    // both operands are pure — no draws).
    let num2 = num && flag;
    if num2 {
        let lerp_value = get_lerp_value(0.0, 30.0, p.ai[0]);
        v = rotate(vector, -p.direction as f32 * std::f32::consts::TAU * 0.02 * lerp_value);
    }
    // `:48725` — the speed-preserving steer (`SafeNormalize(UnitY)`).
    let speed = p.vel.length();
    p.vel = if v == Vec2::ZERO || v.is_nan() { Vec2::Y } else { v.normalize() } * speed;
    // `:48726-48752` — the non-loop float.
    if !num2 {
        let num3 = 0.15 + (0.05 - 0.15) * wind.abs();
        if p.time_left % 40 < 20 {
            p.vel.y -= num3;
        } else {
            p.vel.y += num3;
        }
        p.vel.y = p.vel.y.clamp(-2.0, 2.0);
        p.vel.x = (p.vel.x + wind * 0.006).clamp(-6.0, 6.0);
        // `:48746-48751` — the X-sign flip re-arm.
        if p.vel.x * old_vel_x < 0.0 {
            p.direction *= -1;
            p.ai[0] = -rng.next_range(120, 300) as f32;
            p.net_update = true;
        }
    }
    // `:48753-48754` — the rotation and the sprite facing.
    p.rot = p.vel.y.atan2(p.vel.x);
    p.sprite_direction = p.direction;
}

/// `Collision.LavaCollision` (`Collision.cs:1653-1684`) — the clamped
/// box scan (`[x/16−1, (x+w)/16+2)` × `[y/16−1, (y+h)/16+2)`, the y-range
/// capped at `maxTilesY−40`): any lava cell whose liquid surface
/// (`(256−liquid)/32·2` px down from the cell top) the box overlaps.
fn lava_collision(world: &World, pos: Vec2, w: f32, h: f32) -> bool {
    let x0 = ((pos.x / 16.0) as i32 - 1).clamp(0, world.width - 1);
    let x1 = (((pos.x + w) / 16.0) as i32 + 2).clamp(0, world.width - 1);
    let y0 = ((pos.y / 16.0) as i32 - 1).clamp(0, (world.height - 40).max(0));
    let y1 = (((pos.y + h) / 16.0) as i32 + 2).clamp(0, (world.height - 40).max(0));
    for tx in x0..x1 {
        for ty in y0..y1 {
            let Some(t) = world.tile(tx, ty) else { continue };
            if t.liquid == 0 || t.liquid_kind() != tr_core::tile::LiquidKind::Lava {
                continue;
            }
            let surface = (256.0 - t.liquid as f32) / 32.0 * 2.0;
            let top = ty as f32 * 16.0 + surface;
            let height = 16.0 - surface as i32 as f32;
            if pos.x + w > tx as f32 * 16.0
                && pos.x < tx as f32 * 16.0 + 16.0
                && pos.y + h > top
                && pos.y < top + height
            {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Witnesses (S1-D-4 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs, proj_data};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-recording `Main.rand` stand-in (the `projectile.rs`
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0 — the
    /// census proves count AND order.
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }

    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
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
}

#[cfg(test)]
mod w4_d16_151_tests {
    //! W4-d16 — aiStyle 151 (`AI_151_SuperStar`, `Projectile.cs:22138-22168`,
    //! the dispatch `:36567-36570`). WRITTEN NOT RUN (code-only mode). The
    //! witnesses mirror the C# on the port's seams: the row, the alpha
    //! ramp + floor, the cue re-arm's `Next(40)`, the spin, the on-screen
    //! gore gate's `&&` short-circuit (off-screen spends NO `Next(6)`) and
    //! the two dust gates' `Next(8)` pair — the RNG census pins count AND
    //! order on a recording stream.
    use super::*;
    use crate::projectile::{ProjectilePool, SUPER_STAR, FlailInputs, YoyoInputs, proj_data};

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min // every gate takes its branch — the census sees the kicks
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn star() -> Projectile {
        let d = proj_data(SUPER_STAR).expect("the 728 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = SUPER_STAR;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The row (`Projectile.cs:7659-7671`): 24x24, aiStyle 151, alpha 255,
    /// tileCollide true, ranged, friendly, penetrate -1, the LOCAL pair
    /// (-1).
    #[test]
    fn w4_d16_151_the_row_matches_set_defaults() {
        let d = proj_data(SUPER_STAR).unwrap();
        assert_eq!((d.w, d.h), (24.0, 24.0), ":7661-7662");
        assert_eq!(d.ai_style, 151, ":7663");
        assert_eq!(d.alpha, 255, ":7664");
        assert!(d.tile_collide, ":7665");
        assert!(d.ranged, ":7666");
        assert_eq!(d.penetrate, -1, ":7668");
        assert!(d.uses_local_immunity, ":7669");
        assert_eq!(d.local_hit_cooldown, -1, ":7670");
        assert_eq!(d.extra_updates, 0, "no override — the stock 0");
    }

    /// The alpha ramp (`:22140-22145`): −10/tick, floored at 100. The
    /// cue re-arm (`:22146-22150`): `20 + Next(40)` — the census min 0
    /// gives 20; the Item9 cue rides `wants_sound`. The spin (`:22151`):
    /// (|3|+|4|)·0.005·1 = 0.035.
    #[test]
    fn w4_d16_151_the_ramp_cue_and_spin() {
        let mut p = star();
        p.vel = Vec2::new(3.0, 4.0);
        let mut rng = census();
        // Off-screen (`None`): the gore gate short-circuits BEFORE its
        // Next(6) — the census sees only the cue + the two dust gates.
        ai_super_star(&mut p, &mut rng, None);
        assert_eq!(p.alpha, 245, "255 − 10");
        assert_eq!(p.sound_delay, 20, "20 + Next(40)=0");
        assert_eq!(p.wants_sound, Some((2, 9, 1600.0, 800.0, 0.0)), "Item9");
        assert!((p.rot - 0.035).abs() < 1e-6, "the spin: {}", p.rot);
        assert_eq!(rng.ranges, vec![(0, 40), (0, 8), (0, 8)], "no Next(6) off-screen");
        assert_eq!(rng.floats, 0);
        // The floor: alpha 105 → 95 → 100.
        p.alpha = 105;
        p.sound_delay = -1; // the latch — no re-arm, no draw
        let mut rng2 = census();
        ai_super_star(&mut p, &mut rng2, None);
        assert_eq!(p.alpha, 100, "the floor");
        assert_eq!(rng2.ranges, vec![(0, 8), (0, 8)], "the latch spends no Next(40)");
        // The negative-velocity spin mirrors (`direction = -1`).
        let mut p3 = star();
        p3.vel = Vec2::new(-3.0, 4.0);
        p3.sound_delay = -1;
        ai_super_star(&mut p3, &mut census(), None);
        assert!((p3.rot + 0.035).abs() < 1e-6, "the mirrored spin");
    }

    /// The on-screen gore gate (`:22152-22156`): the hitbox inside the
    /// grown screen rect → `Next(6)`; on the taken branch the
    /// `SelectRandom` pick `Next(4)` + the NewGore kicks `Next(10,31)` /
    /// `Next(-20,21)` (`Gore.cs:1327-1328`). The dust gates follow.
    #[test]
    fn w4_d16_151_the_on_screen_gore_census() {
        let mut p = star();
        p.sound_delay = -1;
        let mut rng = census();
        // The screen centre on the star, any size — the box intersects.
        ai_super_star(&mut p, &mut rng, Some((Vec2::new(1612.0, 812.0), Vec2::new(800.0, 600.0))));
        assert_eq!(
            rng.ranges,
            vec![(0, 6), (0, 4), (10, 31), (-20, 21), (0, 8), (0, 8)],
            "the gate, the pick, the kicks, the dust pair"
        );
        // Far off-screen: the rect test fails, NO Next(6) (`&&` order).
        let mut p2 = star();
        p2.sound_delay = -1;
        let mut rng2 = census();
        ai_super_star(&mut p2, &mut rng2, Some((Vec2::new(9000.0, 9000.0), Vec2::new(800.0, 600.0))));
        assert_eq!(rng2.ranges, vec![(0, 8), (0, 8)]);
    }

    /// The dispatch wiring: one pool update runs the arm (alpha 255→245,
    /// the cue re-arm, the spin) and the kill_step integrates the flight.
    #[test]
    fn w4_d16_151_the_dispatch_arms_the_star() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(SUPER_STAR, 1600.0, 800.0, 3.0, 4.0, 40, 2.0)
            .unwrap();
        pool.slots_mut()[slot].alpha = 255;
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
        assert!(p.active);
        assert_eq!(p.alpha, 245, "one sub-step (extraUpdates 0)");
        assert_eq!(p.pos, Vec2::new(1603.0, 804.0), "the kill_step integrate");
        assert_eq!(p.time_left, 3599, "the lifetime block");
        // screen_view None in the pool-only test: no Next(6).
        assert_eq!(rng.ranges, vec![(0, 40), (0, 8), (0, 8)]);
    }
}

#[cfg(test)]
mod w4_d16_152_tests {
    //! W4-d16 — aiStyle 152 (`AI_152_SuperStarSlash`,
    //! `Projectile.cs:22230-22351`, the dispatch `:36571-36574`). WRITTEN
    //! NOT RUN (code-only mode). The four rows (729/977/976/1045), the
    //! alpha ladders, the cue latches, the ai[0] spin's per-kind divisor,
    //! the 977/1045 drag + dust census, the 976 lava kill + frame cycle +
    //! Remap envelope + one-shot burst census, the shared rotation tail.
    use super::*;
    use crate::projectile::{
        FlailInputs, MOON_LORD_WHIP_PROC, ProjectilePool, SUPER_STAR_SLASH, YoyoInputs,
        ZENITH_GRASS_SLASH, ZENITH_MURAMASA_SLASH, proj_data,
    };

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn slash(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed slash");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.extra_updates = d.extra_updates;
        p
    }

    /// The four rows (`:7672-7687`/`:9208-9224`/`:9270-9286`/`:9890-9907`).
    #[test]
    fn w4_d16_152_the_four_rows_match_set_defaults() {
        let d = proj_data(SUPER_STAR_SLASH).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (20.0, 20.0, 152));
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (-1, 30, 2), "10*MaxUpdates");
        assert!(d.uses_id_static_immunity && d.id_static_hit_cooldown == 10);
        assert!(d.ranged && d.ignore_water && !d.tile_collide);
        let d = proj_data(ZENITH_MURAMASA_SLASH).unwrap();
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (1, 30, 2));
        assert!(d.uses_local_immunity && d.local_hit_cooldown == -1);
        assert!(d.melee && d.ignore_water);
        let d = proj_data(ZENITH_GRASS_SLASH).unwrap();
        assert_eq!((d.penetrate, d.time_left, d.extra_updates), (2, 60, 0), "60*MaxUpdates");
        assert!(d.melee);
        let d = proj_data(MOON_LORD_WHIP_PROC).unwrap();
        assert_eq!((d.penetrate, d.time_left, d.extra_updates, d.alpha), (1, 48, 1, 255), "24*MaxUpdates");
        assert!(d.melee);
    }

    /// The 729: the −10 alpha ramp to 100, the Item9 `20 + Next(40)`
    /// re-arm, the ai[0]/(10·3) spin, the π/2 rotation offset, the
    /// tileCollide pin. NO other draws (the 729 has no dust branch).
    #[test]
    fn w4_d16_152_the_cannon_slash_ramp_spin_and_cue() {
        let mut p = slash(SUPER_STAR_SLASH);
        p.alpha = 255; // the spawn seam's stock 0 — the C# row leaves 0;
        // the witness drives the ramp from a visible value.
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 30.0; // one full divisor turn: 30/(10*3) = 1 rad
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.alpha, 245);
        assert_eq!(p.sound_delay, 20, "20 + Next(40)=0");
        assert_eq!(p.wants_sound, Some((2, 9, 1600.0, 800.0, 0.0)));
        assert_eq!(rng.ranges, vec![(0, 40)], "the cue draw only");
        // The spin: RotatedBy(1 rad) of (10,0) — cos/sin in double.
        let c = 1.0f64.cos() as f32;
        let s = 1.0f64.sin() as f32;
        assert!((p.vel.x - 10.0 * c).abs() < 1e-4 && (p.vel.y - 10.0 * s).abs() < 1e-4);
        assert!((p.rot - (p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2)).abs() < 1e-6);
        assert!(!p.tile_collide, ":22350");
        // The alpha floor.
        p.alpha = 105;
        p.sound_delay = -1;
        ai_super_star_slash(&mut p, &w, &mut census());
        assert_eq!(p.alpha, 100);
    }

    /// The 977: the Item1 one-cue latch, the ×0.96 drag, the 1-in-8 dust
    /// gate + the taken branch's NextVector2Circular pair (two floats).
    #[test]
    fn w4_d16_152_the_muramasa_drag_and_dust_census() {
        let mut p = slash(ZENITH_MURAMASA_SLASH);
        p.vel = Vec2::new(10.0, 0.0);
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.sound_delay, -1, "the one-cue latch");
        assert_eq!(p.wants_sound, Some((2, 1, 1600.0, 800.0, 0.0)), "Item1");
        assert!((p.vel.x - 9.6).abs() < 1e-5, "the ×0.96 drag");
        assert_eq!(rng.ranges, vec![(0, 8)]);
        assert_eq!(rng.floats, 2, "NextVector2Circular(1,1) — the rewritten velocity");
        // The latch holds: no second cue, the gate still rolls.
        let mut rng2 = census();
        ai_super_star_slash(&mut p, &w, &mut rng2);
        assert_eq!(rng2.ranges, vec![(0, 8)]);
        assert_eq!(rng2.floats, 2);
    }

    /// The 1045: the −50 ramp to 0, the 24-base divisor (24·2 = 48), the
    /// Item1 latch + the same dust census as the 977.
    #[test]
    fn w4_d16_152_the_whip_proc_ramp_and_divisor() {
        let mut p = slash(MOON_LORD_WHIP_PROC);
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 48.0; // 48/48 = 1 rad
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.alpha, 205, "255 − 50");
        assert_eq!(p.sound_delay, -1);
        assert_eq!(rng.ranges, vec![(0, 8)], "no Next(40) — the Item1 arm");
        assert_eq!(rng.floats, 2);
        let c = 1.0f64.cos() as f32;
        // The drag runs AFTER the spin: 10·cos(1)·0.96.
        assert!((p.vel.x - 10.0 * c * 0.96).abs() < 1e-4);
        // The floor at 0.
        p.alpha = 30;
        ai_super_star_slash(&mut p, &w, &mut census());
        assert_eq!(p.alpha, 0);
    }

    /// The 976: the silent latch (NO cue draw), the full-angle spin, the
    /// frame cycle, the Remap speed envelope (ai[1] = 0, timeLeft 60 →
    /// fromValue 0 → num6 = 0 → speed 16), the opacity fade-in start
    /// (alpha 255), the Next(3) gate + the one-shot 10-dust burst's 20
    /// floats, the zero rotation offset.
    #[test]
    fn w4_d16_152_the_grass_slash_envelope_and_burst() {
        let mut p = slash(ZENITH_GRASS_SLASH);
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 0.5; // the FULL angle (no divisor)
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.sound_delay, -1, "the silent latch");
        assert_eq!(p.wants_sound, None, "no cue");
        assert_eq!(p.alpha, 255, "Opacity 0 at fromValue 0");
        assert!((p.vel.length() - 16.0).abs() < 1e-4, "4 + 12·(1−0)");
        assert_eq!(p.frame, 0, "frameCounter 1 < 2");
        assert_eq!(p.frame_counter, 1);
        assert_eq!(p.local_ai[1], 1.0, "the burst latch");
        assert_eq!(rng.ranges, vec![(0, 3)], "the trail gate only — no cue draw");
        assert_eq!(rng.floats, 20, "10 dust × (angle + speed)");
        assert!((p.rot - p.vel.y.atan2(p.vel.x)).abs() < 1e-6, "num = 0");
        // Second tick: the frame advances, the burst does NOT re-fire.
        let mut rng2 = census();
        ai_super_star_slash(&mut p, &w, &mut rng2);
        assert_eq!(p.frame, 1);
        assert_eq!(p.frame_counter, 0);
        assert_eq!(rng2.floats, 0, "the one-shot latch");
        // The speed at fromValue 30 (timeLeft 30): num6 = Remap(30,0,20→
        // clamped 1)·Remap(30,0,60,1→0 = 0.5) = 0.5 → speed 10.
        p.time_left = 30;
        ai_super_star_slash(&mut p, &w, &mut census());
        assert!((p.vel.length() - 10.0).abs() < 1e-4);
    }

    /// The 976 lava kill (`:22316-22320`): a full lava cell under the box
    /// kills BEFORE the frame/envelope work — zero draws.
    #[test]
    fn w4_d16_152_the_grass_slash_dies_in_lava() {
        let mut w = World::new("t".into(), 300, 200);
        let i = w.index(100, 50);
        w.tiles[i].liquid = 255;
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        let mut p = slash(ZENITH_GRASS_SLASH);
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0 - 10.0);
        let mut rng = census();
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert!(!p.active, "the lava kill");
        assert_eq!(p.time_left, 0);
        assert_eq!(rng.ranges, Vec::new(), "the kill precedes every draw");
        assert_eq!(rng.floats, 0);
    }

    /// The dispatch wiring: one pool update runs the 729 arm (3 sub-steps
    /// — extraUpdates 2) and the free integrate moves the slot.
    #[test]
    fn w4_d16_152_the_dispatch_arms_the_slashes() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(SUPER_STAR_SLASH, 1600.0, 800.0, 10.0, 0.0, 40, 2.0)
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
        assert!(p.active);
        assert_eq!(p.sound_delay, 20, "the first sub-step's cue re-arm");
        assert_eq!(p.time_left, 27, "30 − 3 sub-steps");
        assert!((p.pos.x - 1630.0).abs() < 1e-3, "3 × 10 px");
        // The cue draws ONCE (the re-arm holds across the sub-steps).
        assert_eq!(rng.ranges, vec![(0, 40)]);
    }
}

#[cfg(test)]
mod w4_d16_153_tests {
    //! W4-d16 — aiStyle 153 (`AI_153_ToiletPop`, `Projectile.cs:52198-
    //! 52236`, the dispatch `:36575-36578`). WRITTEN NOT RUN (code-only
    //! mode). The row, the first-tick burst census (no Next(5) gate —
    //! the gore is unconditional), the 1-in-5 gate ticks, the past-10
    //! kill (zero draws), the counter's after-the-pick advance.
    use super::*;
    use crate::projectile::{FlailInputs, ProjectilePool, TOILET_EFFECT, YoyoInputs, proj_data};

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn pop() -> Projectile {
        let d = proj_data(TOILET_EFFECT).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = TOILET_EFFECT;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The row (`Projectile.cs:7716-7725`): 2x2, aiStyle 153, tileCollide
    /// false, ignoreWater, penetrate -1, alpha 255.
    #[test]
    fn w4_d16_153_the_row_matches_set_defaults() {
        let d = proj_data(TOILET_EFFECT).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (2.0, 2.0, 153));
        assert!(!d.tile_collide && d.ignore_water);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
    }

    /// The first tick (`:52204-52208`, `:52231-52235`): NO Next(5) gate —
    /// the gore is unconditional: the circular pair + the type roll +
    /// the kicks. The counter advances to 1.
    #[test]
    fn w4_d16_153_the_first_tick_burst_census() {
        let mut p = pop();
        let mut rng = census();
        ai_toilet_pop(&mut p, &mut rng);
        assert_eq!(p.ai[0], 1.0);
        assert_eq!(rng.ranges, vec![(580, 583), (10, 31), (-20, 21)], "no gate on tick 0");
        assert_eq!(rng.floats, 2, "the circular velocity pair");
    }

    /// The gate ticks (`:52216-52220`): the Next(5) draws; on the taken
    /// branch (census min 0) the gore row follows. The counter advances.
    #[test]
    fn w4_d16_153_the_gate_ticks_census() {
        let mut p = pop();
        p.ai[0] = 5.0;
        let mut rng = census();
        ai_toilet_pop(&mut p, &mut rng);
        assert_eq!(p.ai[0], 6.0);
        assert_eq!(rng.ranges, vec![(0, 5), (580, 583), (10, 31), (-20, 21)]);
        assert_eq!(rng.floats, 2);
    }

    /// The past-10 kill (`:52211-52215`): ai[0] = 11 dies BEFORE the gate
    /// — zero draws, the counter does NOT advance.
    #[test]
    fn w4_d16_153_the_past_ten_kill_spends_nothing() {
        let mut p = pop();
        p.ai[0] = 11.0;
        let mut rng = census();
        ai_toilet_pop(&mut p, &mut rng);
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert_eq!(p.ai[0], 11.0, "the kill returns before the ++");
        assert_eq!(rng.ranges, Vec::new());
        assert_eq!(rng.floats, 0);
        // ai[0] = 10 is the LAST live tick (`<= 10f` holds).
        let mut p2 = pop();
        p2.ai[0] = 10.0;
        ai_toilet_pop(&mut p2, &mut census());
        assert!(p2.active);
        assert_eq!(p2.ai[0], 11.0);
    }

    /// The dispatch wiring: one pool update runs the first-tick burst.
    #[test]
    fn w4_d16_153_the_dispatch_arms_the_pop() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(TOILET_EFFECT, 1600.0, 800.0, 0.0, 0.0, 0, 0.0)
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
        assert!(p.active);
        assert_eq!(p.ai[0], 1.0);
        assert_eq!(rng.ranges, vec![(580, 583), (10, 31), (-20, 21)]);
        assert_eq!(rng.floats, 2);
    }
}

#[cfg(test)]
mod w4_d16_154_tests {
    //! W4-d16 — aiStyle 154 (`AI_154_VoidLens`, `Projectile.cs:52154-
    //! 52196`, the dispatch `:36579-36582`). WRITTEN NOT RUN (code-only
    //! mode). The row + sheet, the damp/direction/frame/opacity body, the
    //! VoidLensHelper dust census (both branches), the pool-side dedupe
    //! (the younger lens kills the older; an older sibling kills this).
    use super::*;
    use crate::projectile::{FlailInputs, ProjectilePool, VOID_LENS, YoyoInputs, proj_data};

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn lens() -> Projectile {
        let d = proj_data(VOID_LENS).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = VOID_LENS;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The row (`Projectile.cs:7726-7735`) + the 8-frame sheet
    /// (`Main.cs:8504`).
    #[test]
    fn w4_d16_154_the_row_and_sheet_match_set_defaults() {
        let d = proj_data(VOID_LENS).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (30.0, 24.0, 154));
        assert!(!d.tile_collide);
        assert_eq!(d.time_left, 10800);
        assert_eq!(d.alpha, 255);
        assert_eq!(crate::projectile::proj_frames(VOID_LENS), 8);
    }

    /// The body (`:52177-52194`): the ×0.94 damp, direction 1, the 5-tick
    /// frame cycle, the +0.03 opacity climb (alpha 255 → 247: opacity
    /// 0.03 → alpha (1−0.03)·255 = 247.35 → 247). The dust census: the
    /// Next(3) gate + the Next(2) pick (census min 0 → the FIRST branch)
    /// + the RotatedByRandom pair + the SelectRandom Next(2) + the
    /// Next(10,21) radius + the scale float.
    #[test]
    fn w4_d16_154_the_body_and_dust_census() {
        let mut p = lens();
        p.vel = Vec2::new(10.0, 0.0);
        let mut rng = census();
        ai_void_lens(&mut p, &mut rng);
        assert!((p.vel.x - 9.4).abs() < 1e-5, "the ×0.94 damp");
        assert_eq!(p.direction, 1);
        assert_eq!(p.frame_counter, 1);
        assert_eq!(p.frame, 0);
        assert_eq!(p.alpha, 247, "the opacity climb's truncation");
        assert_eq!(rng.ranges, vec![(0, 3), (0, 2), (0, 2), (10, 21)]);
        assert_eq!(rng.floats, 3, "the rotation pair + the scale");
        // The frame cycle: 5 ticks advance the frame, 40 wrap it.
        for _ in 0..4 {
            ai_void_lens(&mut p, &mut census());
        }
        assert_eq!(p.frame, 1, "the 5th tick");
        assert_eq!(p.frame_counter, 0);
        // The opacity caps at 1 (alpha 0).
        p.alpha = 3; // opacity ≈ 0.988 + 0.03 > 1 → clamp
        ai_void_lens(&mut p, &mut census());
        assert_eq!(p.alpha, 0);
    }

    /// The dedupe (`:52157-52176`): two lenses, one owner — the YOUNGER
    /// (lower timeLeft) dies, the older survives; the scan kills the
    /// older siblings when THIS lens is the oldest.
    #[test]
    fn w4_d16_154_the_dedupe_keeps_the_oldest() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let older = pool.spawn_with_data(VOID_LENS, 1600.0, 800.0, 0.0, 0.0, 0, 0.0).unwrap();
        let younger = pool.spawn_with_data(VOID_LENS, 1700.0, 800.0, 0.0, 0.0, 0, 0.0).unwrap();
        pool.slots_mut()[younger].time_left = 100; // younger than 10800
        let mut rng = census();
        pool.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        assert!(pool.slots()[older].active, "the older lens survives");
        assert!(!pool.slots()[younger].active, "the younger lens dies");
        // The foreign-owner lens is NOT deduped (`:52166`'s owner gate).
        let mut pool2 = ProjectilePool::new();
        let a = pool2.spawn_with_data(VOID_LENS, 1600.0, 800.0, 0.0, 0.0, 0, 0.0).unwrap();
        let b = pool2.spawn_with_data(VOID_LENS, 1700.0, 800.0, 0.0, 0.0, 0, 0.0).unwrap();
        pool2.slots_mut()[b].owner = 1;
        pool2.slots_mut()[b].time_left = 100;
        pool2.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
        );
        assert!(pool2.slots()[a].active && pool2.slots()[b].active, "the owner gate");
        let _ = younger;
    }
}

#[cfg(test)]
mod w4_d16_155_tests {
    //! W4-d16 — aiStyle 155 (`AI_155_MysticSnakeCoil`,
    //! `Projectile.cs:51939-52112` + the rope helpers `:52114-52152`, the
    //! dispatch `:36583-36586`). WRITTEN NOT RUN (code-only mode). The
    //! row, the launch latch + rise truncation, the 60-tick burst census
    //! (4 floats × 30), the trail gate's `||` short-circuit, the rope
    //! placement + attach snap, the attached conversion census, the
    //! timeLeft-4 re-burst + the ≤4 removal death, the orphan kill.
    use super::*;
    use crate::projectile::{FlailInputs, MYSTIC_SNAKE_COIL, ProjectilePool, YoyoInputs, proj_data};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn coil() -> Projectile {
        let d = proj_data(MYSTIC_SNAKE_COIL).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = MYSTIC_SNAKE_COIL;
        p.owner = 0;
        // The whip's spawn shape (`Player.cs:47884`): pos.y = ai[0] =
        // the anchor's cell centre − 16 (row 50 → 792). Centre
        // (1608, 800) — tile (100, 50); the rope-scan rows are
        // `position.Y / 16` = 49 (NOT the centre row).
        p.pos = Vec2::new(1600.0, 792.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.ai[0] = 792.0; // the anchor y — scan row 49
        p
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// The row (`Projectile.cs:7767-7776`).
    #[test]
    fn w4_d16_155_the_row_matches_set_defaults() {
        let d = proj_data(MYSTIC_SNAKE_COIL).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (16.0, 16.0, 155));
        assert!(!d.tile_collide && d.ignore_water);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
    }

    /// The launch latch (`:51952-51957`): spriteDirection from the
    /// owner-centre comparison, ai[1] = 1, velocity.Y = −5 → the rise
    /// machine's first tick truncates it to −3 (localAI[1] = 1 < 15 →
    /// num4 = 0 → −3·1 = −3). The trail gate rolls Next(6) (census min 0
    /// → taken), the alpha ramps −5, the frame counter advances.
    #[test]
    fn w4_d16_155_the_launch_and_first_rise_tick() {
        let mut w = world();
        let mut p = coil();
        let mut rng = census();
        // The owner to the RIGHT of the coil → spriteDirection −1.
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(p.sprite_direction, -1);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.vel.y, -3.0, "the int truncation of −3·(1−0)");
        assert_eq!(p.alpha, 250);
        assert_eq!(p.frame_counter, 1);
        assert_eq!(rng.ranges, vec![(0, 6)], "the trail gate only");
        assert_eq!(rng.floats, 0);
        // The owner to the LEFT → spriteDirection 1 (the `!(>)` form).
        let mut p2 = coil();
        ai_mystic_snake_coil(&mut p2, &mut world(), Some(Vec2::new(1000.0, 808.0)), true, 0, &mut census());
        assert_eq!(p2.sprite_direction, 1);
    }

    /// The 60-tick burst (`:51965-51982`): 30 dust × 4 floats (the X
    /// jitter, the angle pair, the ALWAYS-taken Y scale — the rotated
    /// (0,−1) vector's Y is −1 for every draw since the angle is ×0f).
    /// The trail gate's Next(6) is the `||`'s LEFT operand — it rolls
    /// EVERY rise tick (num5 = 1 only skips the body, never the roll).
    #[test]
    fn w4_d16_155_the_sixty_tick_burst_census() {
        let mut w = world();
        let mut p = coil();
        p.ai[1] = 1.0;
        p.local_ai[1] = 59.0; // this tick → 60
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(p.local_ai[1], 60.0);
        assert_eq!(p.vel.y, -32.0, "−3·0 + −32·1");
        assert_eq!(rng.floats, 120, "30 × 4");
        assert_eq!(rng.ranges, vec![(0, 6)], "the Next(6) is the ||'s left operand — always rolled");
    }

    /// The rope placement (`:52002-52046`): the coil has risen to row 43
    /// (pos.y 688), the anchor row 49 — a solid dirt cell at row 45 trips
    /// the blocker scan (`:52017-52024`), so the column fills rows 49..44
    /// (`> point3.Y` — row 43 stays empty; the dirt at 45 is neither
    /// TileEmpty nor tileCut, so it is skipped), the life re-arms to
    /// 1800, ai[1] = 2, the Top snaps to (100·16+8, 43·16+16) and the
    /// velocity zeroes. The attached half runs the SAME tick: the
    /// one-shot conversion over the 5 rope cells (44, 46..49).
    #[test]
    fn w4_d16_155_the_placement_attach_and_conversion() {
        let mut w = world();
        put(&mut w, 100, 45, 1); // the solid blocker (dirt)
        let mut p = coil();
        p.pos = Vec2::new(1600.0, 688.0); // risen — centre row 43
        p.ai[1] = 1.0;
        p.local_ai[1] = 5.0;
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(p.ai[1], 2.0, "the attach");
        assert_eq!(p.time_left, 1800);
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::ZERO);
        // Top = (1608, 704) → pos = (1600, 704) for the 16x16 box.
        assert_eq!(p.pos, Vec2::new(1600.0, 704.0));
        for y in [44, 46, 47, 48, 49] {
            assert_eq!(w.tile(100, y).unwrap().kind, 504, "the column cell {y}");
        }
        assert_eq!(w.tile(100, 45).unwrap().kind, 1, "the blocker untouched");
        assert!(!w.tile(100, 43).unwrap().active(), "the `> point3.Y` bound");
        assert_eq!(p.alpha, 0, "the attached head");
        assert_eq!(p.local_ai[0], 1.0, "the one-shot latch");
        // The conversion census: the trail gate's Next(6) first, then 5
        // rope cells × (Next(4) + 5×Next(8)) — census min 0 → the gore
        // gate FAILS, no gore draws.
        assert_eq!(rng.ranges.len(), 1 + 5 * 6, "the trail gate + 5 cells × 6");
        assert_eq!(rng.ranges[0], (0, 6));
        assert!(rng.ranges[1..].iter().all(|&(a, b)| (a, b) == (0, 4) || (a, b) == (0, 8)));
        assert_eq!(rng.floats, 0, "the gore gate failed — no circular pair");
    }

    /// The gore-taking conversion census: a stream whose Next(4) returns
    /// 1 (gate taken) — per cell the type roll + the kicks + the circular
    /// pair + the 5 dust gates.
    #[test]
    fn w4_d16_155_the_conversion_gore_census() {
        struct GoreRng {
            ranges: Vec<(i32, i32)>,
            floats: usize,
        }
        impl DropRng for GoreRng {
            fn next_range(&mut self, min: i32, max: i32) -> i32 {
                self.ranges.push((min, max));
                if (min, max) == (0, 4) { 1 } else { min }
            }
            fn next_float(&mut self) -> f32 {
                self.floats += 1;
                0.0
            }
        }
        let mut w = world();
        // One rope cell at the coil's own row — the start == end scan.
        put(&mut w, 100, 49, 504);
        let mut p = coil();
        p.ai[0] = 792.0; // the anchor ON the coil row → a 1-cell scan
        p.ai[1] = 2.0;
        let mut rng = GoreRng { ranges: Vec::new(), floats: 0 };
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(
            rng.ranges,
            vec![(0, 4), (61, 64), (10, 31), (-20, 21), (0, 8), (0, 8), (0, 8), (0, 8), (0, 8)],
            "the gate, the type, the kicks, the 5 dust gates"
        );
        assert_eq!(rng.floats, 2, "NextVector2Circular(1.5, 0.8)");
    }

    /// The timeLeft-4 re-burst + the ≤4 death (`:52087-52111`): the
    /// conversion walks AGAIN, then the owner-gated rope removal kills
    /// every 504 cell and the coil dies.
    #[test]
    fn w4_d16_155_the_death_reburst_and_removal() {
        let mut w = world();
        put(&mut w, 100, 49, 504);
        let mut p = coil();
        p.ai[0] = 792.0; // the anchor ON the coil row → a 1-cell scan
        p.ai[1] = 2.0;
        p.local_ai[0] = 1.0; // the per-tick dust walk (not the one-shot)
        p.time_left = 4;
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert!(!p.active, "the ≤4 death");
        assert_eq!(p.time_left, 0);
        assert!(!w.tile(100, 49).unwrap().active(), "the rope removed");
        // The per-tick walk's Next(80) (census min 0 → the dust body,
        // skipped) + the re-burst's per-cell row.
        assert_eq!(rng.ranges[0], (0, 80), "the per-tick dust gate");
        assert_eq!(rng.ranges[1], (0, 4), "the re-burst's gore gate");
    }

    /// The orphan kill (`:51941-51949`): no active owner → the rope
    /// column dies with the coil, ZERO draws.
    #[test]
    fn w4_d16_155_the_orphan_kill_removes_the_rope() {
        let mut w = world();
        put(&mut w, 100, 49, 504);
        put(&mut w, 100, 50, 504);
        let mut p = coil();
        p.ai[0] = 800.0; // the anchor BELOW the coil — rows 50..49 scan
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, None, false, 0, &mut rng);
        assert!(!p.active);
        assert!(!w.tile(100, 49).unwrap().active());
        assert!(!w.tile(100, 50).unwrap().active());
        assert_eq!(rng.ranges, Vec::new());
        assert_eq!(rng.floats, 0);
    }

    /// The dispatch wiring: one pool update runs the launch tick (no
    /// player → the orphan kill; with the owner snapshot the rise runs).
    #[test]
    fn w4_d16_155_the_dispatch_arms_the_coil() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data_ai(MYSTIC_SNAKE_COIL, 1600.0, 800.0, 0.0, 0.0, 0, 0.0, 700.0, 0.0)
            .unwrap();
        let mut rng = census();
        // No player: the orphan kill (the C# reads an inactive player).
        pool.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        assert!(!pool.slots()[slot].active, "the orphan kill");
        assert_eq!(rng.ranges, Vec::new());
    }

    /// The LIVE wiring drain (the residual-3 census): a pool update with
    /// a resolved owner runs the placement walk against the REAL world —
    /// the blocker at row 45 trips the scan, the column fills rows
    /// 49..44 with 504, the coil attaches (ai[1] = 2, life 1800), and the
    /// tile writes bump `world.revision` (the `WorldFrameKey` input —
    /// the frame cache invalidates, `draw.rs:1041`). The death window
    /// then removes every rope cell through the same live path.
    #[test]
    fn w4_d16_155_the_dispatch_drains_the_rope_seams_live() {
        let mut w = world();
        put(&mut w, 100, 45, 1); // the solid blocker (dirt)
        let mut pl = crate::player::Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        // The risen coil: pos.y 688 (centre row 43), anchor row 49,
        // ai[1] = 1 (the rise state) — the first update's blocker scan
        // attaches it.
        let slot = pool
            .spawn_with_data_ai(MYSTIC_SNAKE_COIL, 1600.0, 688.0, 0.0, 0.0, 0, 0.0, 792.0, 1.0)
            .unwrap();
        let rev0 = w.revision;
        let mut rng = census();
        pool.update(
            &mut w, &mut npcs, Some((2000.0, 808.0)), None, Some(&mut pl),
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        assert!(pool.slots()[slot].active, "the coil attached");
        assert_eq!(pool.slots()[slot].ai[1], 2.0, "the attach state");
        for y in [44, 46, 47, 48, 49] {
            assert_eq!(w.tile(100, y).unwrap().kind, 504, "the LIVE column cell {y}");
        }
        assert_eq!(w.tile(100, 45).unwrap().kind, 1, "the blocker untouched");
        assert!(w.revision > rev0, "the tile writes bump the frame-cache key");
        // The death window: timeLeft 4 → the re-burst + the owner-gated
        // removal kill every 504 cell through the same dispatch.
        pool.slots_mut()[slot].time_left = 4;
        let rev1 = w.revision;
        pool.update(
            &mut w, &mut npcs, Some((2000.0, 808.0)), None, Some(&mut pl),
            YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
        );
        assert!(!pool.slots()[slot].active, "the ≤4 death");
        for y in [44, 46, 47, 48, 49] {
            assert!(!w.tile(100, y).unwrap().active(), "the LIVE removal {y}");
        }
        assert!(w.revision > rev1, "the removal bumps the key too");
    }
}

#[cfg(test)]
mod w4_d16_159_tests {
    //! W4-d16 — aiStyle 159 (`AI_159_PaperAirplanes`,
    //! `Projectile.cs:48695-48755`, the dispatch `:36599-36602`, the row
    //! `:7846-7854`, the Kill arm `:76083-76095`). WRITTEN NOT RUN
    //! (code-only mode). The row, the launch latch + the glide census,
    //! the 70 re-arm, the loop bank, the float bob + the X-flip re-arm,
    //! the wet kill, the dispatch + the recovery drop.
    use super::*;
    use crate::projectile::{
        FlailInputs, PAPER_AIRPLANE_A, PAPER_AIRPLANE_B, ProjectilePool, YoyoInputs, proj_data,
    };

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn plane() -> Projectile {
        let d = proj_data(PAPER_AIRPLANE_A).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = PAPER_AIRPLANE_A;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        // The throw: 5 px/tick rightward (`shootSpeed` 5, the item row).
        p.vel = Vec2::new(5.0, 0.0);
        p
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    /// The row (`Projectile.cs:7846-7854`).
    #[test]
    fn w4_d16_159_the_row_matches_set_defaults() {
        for kind in [PAPER_AIRPLANE_A, PAPER_AIRPLANE_B] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h, d.ai_style), (10.0, 10.0, 159));
            assert!(d.tile_collide && !d.ignore_water);
            assert_eq!(d.penetrate, 1);
            assert_eq!(d.time_left, 3600);
            assert!(d.ranged, "the ranged crit class (`:7852`)");
        }
    }

    /// The launch latch (`:48697-48704`): the velocity-sign direction,
    /// the rotation, ai[1] = 1, ai[0] = -Next(30,80) (census min 30 →
    /// -30, then the counter++ → -29). Calm wind: the float arm runs —
    /// the bob step −0.15 (timeLeft 3600 % 40 = 0 < 20), the Y clamp,
    /// the X drift 0. The steer keeps the speed 5 along +X.
    #[test]
    fn w4_d16_159_the_launch_and_first_glide_tick() {
        let mut p = plane();
        let mut rng = census();
        ai_paper_airplanes(&mut p, &mut rng, 0.0, 0.0, 0, 5.0);
        assert_eq!(p.direction, 1);
        assert_eq!(p.rot, 0.0);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.ai[0], -29.0, "-Next(30,80) + the counter++");
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::new(5.0, -0.15), "the steer keeps 5, the bob −0.15");
        assert_eq!(p.sprite_direction, 1);
        assert_eq!(rng.ranges, vec![(30, 80)], "the launch timer only");
        assert_eq!(rng.floats, 0);
        // The leftward throw → direction −1 (the `> 0` form).
        let mut p2 = plane();
        p2.vel = Vec2::new(-5.0, 0.0);
        ai_paper_airplanes(&mut p2, &mut census(), 0.0, 0.0, 0, -5.0);
        assert_eq!(p2.direction, -1);
    }

    /// The 70 re-arm (`:48715-48718`): ai[0] = 69 → the counter++ hits
    /// 70 → ai[0] = -Next(120,600) (census min 120 → -120). The loop
    /// window `num` was computed BEFORE the re-arm (69 ≤ 69 → true), but
    /// calm wind folds `flag` false — the float arm runs.
    #[test]
    fn w4_d16_159_the_seventy_rearm() {
        let mut p = plane();
        p.ai[1] = 1.0;
        p.ai[0] = 69.0;
        let mut rng = census();
        ai_paper_airplanes(&mut p, &mut rng, 0.0, 0.0, 0, 5.0);
        assert_eq!(p.ai[0], -120.0, "the 70 re-arm");
        assert_eq!(rng.ranges, vec![(120, 600)]);
    }

    /// The loop bank (`:48719-48725`): downwind (direction 1, wind 0.5,
    /// speed 5 > 3) and ai[0] = 30 (the window's top) → the eased bank
    /// `−1·2π·0.02·1` rotates the 8-unit vector; the steer normalizes it
    /// back to speed 5. The float arm is SKIPPED (num2 true) — no bob,
    /// no drift, no flip draw.
    #[test]
    fn w4_d16_159_the_loop_bank_steers() {
        let mut p = plane();
        p.ai[1] = 1.0;
        p.ai[0] = 30.0; // the counter++ → 31, still in 20..69
        p.direction = 1;
        let mut rng = census();
        ai_paper_airplanes(&mut p, &mut rng, 0.5, 0.0, 0, 5.0);
        // ai[0] 31 → GetLerpValue(0,30,31) clamps to 1 → the full bank.
        let banked = rotate(Vec2::new(8.0, 0.0), -std::f32::consts::TAU * 0.02);
        let want = banked.normalize() * 5.0;
        assert!((p.vel - want).length() < 1e-4, "the banked steer: {:?} vs {:?}", p.vel, want);
        assert_eq!(rng.ranges, Vec::new(), "the loop arm draws nothing");
        // The rotation follows the banked velocity.
        assert_eq!(p.rot, p.vel.y.atan2(p.vel.x));
    }

    /// The X-flip re-arm (`:48746-48751`): a leftward drift crosses zero
    /// against the old velocity's sign → direction flips, ai[0] =
    /// -Next(120,300). Wind −0.5, the plane flying left at 5: the steer
    /// keeps −5, the drift −0.003 keeps it negative — no flip. Force the
    /// crossing with a tiny leftward velocity and a rightward wind.
    #[test]
    fn w4_d16_159_the_x_flip_rearm() {
        let mut p = plane();
        p.ai[1] = 1.0;
        p.ai[0] = 0.0;
        p.vel = Vec2::new(-0.001, 0.0); // barely leftward
        p.direction = -1;
        let mut rng = census();
        // Wind +0.5: the bob is downwind-flagged false (direction −1 vs
        // sign +1), the float arm runs; the drift +0.003 crosses zero.
        ai_paper_airplanes(&mut p, &mut rng, 0.5, 0.0, 0, -0.001);
        assert_eq!(p.direction, 1, "the X-sign flip");
        // ai[0] was 0 → the counter++ → 1, then the flip re-arm → -120
        // (the re-arm is the LAST write — no counter after it).
        assert_eq!(p.ai[0], -120.0, "-Next(120,300)");
        assert!(p.net_update);
        assert_eq!(rng.ranges, vec![(120, 300)]);
    }

    /// The wet kill (`:48705-48708`): the owner-gated wet probe kills
    /// the plane (the port's death convention zeroes time_left).
    #[test]
    fn w4_d16_159_the_wet_kill() {
        let mut p = plane();
        p.wet = true;
        let mut rng = census();
        ai_paper_airplanes(&mut p, &mut rng, 0.0, 0.0, 0, 5.0);
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        // The foreign owner's wet plane survives (the owner gate).
        let mut p2 = plane();
        p2.wet = true;
        p2.owner = 1;
        ai_paper_airplanes(&mut p2, &mut census(), 0.0, 0.0, 0, 5.0);
        assert!(p2.active, "the owner gate");
    }

    /// The dispatch wiring + the Kill recovery drop: a plane thrown into
    /// a solid tile dies on contact (the blocked-move default) and queues
    /// the Item10 cue + the 4343 recovery drop at its centre.
    #[test]
    fn w4_d16_159_the_dispatch_and_recovery_drop() {
        let mut w = world();
        // A dirt wall at tile (101, 50) — the plane at (1600, 800) flies
        // right into it (centre row 50).
        let i = w.index(101, 50);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data_ai(PAPER_AIRPLANE_A, 1600.0, 800.0, 5.0, 0.0, 4, 2.0, 0.0, 0.0)
            .unwrap();
        let mut rng = census();
        pool.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        assert!(!pool.slots()[slot].active, "the tile kill");
        let sounds = pool.drain_proj_sounds();
        assert!(sounds.iter().any(|&(id, style, _, _, _)| id == 2 && style == 10), "the Item10 cue");
        let drops = pool.drain_item_drops();
        assert_eq!(drops.len(), 1, "the recovery drop");
        assert_eq!(drops[0].0, 4343, "761 → 4343");
        // The B variant drops 4344.
        let mut pool2 = ProjectilePool::new();
        let s2 = pool2
            .spawn_with_data_ai(PAPER_AIRPLANE_B, 1600.0, 800.0, 5.0, 0.0, 4, 2.0, 0.0, 0.0)
            .unwrap();
        pool2.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
        );
        assert!(!pool2.slots()[s2].active);
        assert_eq!(pool2.drain_item_drops()[0].0, 4344, "762 → 4344");
    }
}
