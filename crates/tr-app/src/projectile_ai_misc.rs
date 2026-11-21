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
    Projectile, alpha_of, can_hit, chaseable, chaseable_held, get_lerp_value, npc_center, remap,
    rotate, rotation_vector, safe_normalize, solid_tiles, wrap_angle,
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
// W4-d20 — the aiStyle 191-200 census slice. The seven styles the W4-d19
// wave left UNARMED (193/194/197 already landed): 191 TrueNightsEdge,
// 192 JuminoAnimation, 195 JimsDrone, 196 Petal, 198 Flint, 199 MeteorOre,
// 200 BirdDroppings. C# sources read WHOLE (the golden rule); the RNG
// contract is the module's: every EXPLICIT `Main.rand` call-site draw stays
// in order on the `UpdateProjectiles` stream, the dust/gore BODIES and the
// `Dust.NewDust*` internal rolls are the documented skip (no dust layer).
// `Lighting.AddLight` feeds the light layer only — skipped. Witnesses are
// WRITTEN NOT RUN (code-only mode).

/// `AI_200_BirdDroppings` (`Projectile.cs:39985-40017`) — the bird dropping
/// (1080), a hostile falling mote. The 0.3 gravity with the 16 cap, the
/// two-dust trail (type 329) and the player-contact `Kill()`.
///
/// RNG: per tick TWO iterations, each one `Next(2)` branch pick then one
/// `NextFloat` scale (`:39996-40006`) — both branches draw the scale, so
/// 2 range + 2 float draws. The `NewDust` body and its internal position
/// rolls are the dust skip; the `velocity *= 0.1` / `+= velocity` blends
/// feed only the skipped dust.
///
/// The stock `friendly`/`hostile` are both false (`SetDefaults :10078-10086`
/// sets neither), so the generic PVE/PVP passes refuse it — the contact
/// `Kill()` here is the C#'s own player-rectangle test, not a damage hit
/// (the spawner passes damage 0, `NPC.cs:25322`).
pub(crate) fn ai_bird_droppings(
    p: &mut Projectile,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:39987-39991` — the gravity and the 16 fall cap.
    p.vel.y += 0.3;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `:39992-40008` — the two-dust trail. Each iteration draws the
    // `Next(2)` branch pick then the branch's `NextFloat` scale; the dust
    // body (type 329, the velocity blends, noGravity) is the skip.
    for _ in 0..2 {
        if rng.next_range(0, 2) == 0 {
            let _ = rng.next_float(); // scale = NextFloat * 0.5 + 0.5
        } else {
            let _ = rng.next_float(); // scale = NextFloat * 0.4 + 0.4
        }
    }
    // `:40009-40016` — the player-contact kill: the `(int)` truncating
    // projectile rect vs each active, living player's rect, XNA's strict
    // `Intersects`. The port's single player rides slot 0.
    let (px, py, pw, ph) = (p.pos.x as i32, p.pos.y as i32, p.w as i32, p.h as i32);
    if let Some(pl) = player {
        if !pl.dead {
            let (qx, qy, qw, qh) =
                (pl.x as i32, pl.y as i32, pl.width as i32, pl.height as i32);
            let intersects = px < qx + qw && qx < px + pw && py < qy + qh && qy < py + ph;
            if intersects {
                p.kill_time_left = p.time_left;
                p.active = false;
                p.time_left = 0;
            }
        }
    }
}

/// `AI_199_MeteorOre` (`Projectile.cs:40019-40041`) — the meteor ore (1078),
/// a hostile+friendly+trap falling chunk. The first-tick frame roll, the
/// dust-31 trail, the `!wet` dust-6 sparkle and the two rotation terms.
///
/// RNG: first tick one `Next(3)` frame roll (`:40024`). Every tick the
/// dust-31 `scale` `Next(5)` (`:40027`) then `fadeIn` `Next(5)` (`:40028`);
/// when `!wet` the dust-6 branch's `scale` `Next(5)` (`:40033`) then the
/// position X/Y `Next(-40,41)` jitters (`:40034-40035`). The `NewDust` /
/// `NewDustDirect` bodies and their internal rolls are the dust skip.
/// `Lighting.AddLight` (`:40038`) feeds the light layer only — skipped.
///
/// `base.direction` is the velocity-sign stand-in (`AutomaticallyChangesDirection`
/// true for aiStyle 199, `:17283`; the `ai_spray` precedent). The row keeps
/// stock `tileCollide = true`; the blocked-move default (`:19849-19853`)
/// kills on contact — `kill_step`'s shape. No gravity in the arm: the spawn
/// velocity (`WorldGen.cs:71777`) carries the flight.
pub(crate) fn ai_meteor_ore(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:40021-40025` — the first-tick latch and the 3-frame roll.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        p.frame = rng.next_range(0, 3);
    }
    // `:40026-40029` — the dust-31 trail: the scale and fadeIn draws stay,
    // the body (noGravity) is the skip.
    let _ = rng.next_range(0, 5); // scale = 0.1 + Next(5) * 0.1
    let _ = rng.next_range(0, 5); // fadeIn = 1.5 + Next(5) * 0.1
    // `:40030-40037` — the `!wet` dust-6 sparkle: the scale draw and the
    // two position jitters stay; the body (noGravity) is the skip.
    if !p.wet {
        let _ = rng.next_range(0, 5); // scale = 1 + Next(5) * 0.1
        let _ = rng.next_range(-40, 41); // position.X += Next(-40,41) * 0.1
        let _ = rng.next_range(-40, 41); // position.Y += Next(-40,41) * 0.1
    }
    // `:40039-40040` — the two rotation terms. `direction` is the
    // velocity-sign stand-in (module doc).
    let direction = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
    p.rot += p.vel.x * 0.06;
    p.rot += direction * 0.06;
}

/// `AI_196_Petal` (`Projectile.cs:40466-40560`) — the Flower Whip's petal
/// (1038) and the Petalstorm shot (1116). The first-20-tick drift (the
/// ×0.95 damp), the frame cycle (8 frames at 4 ticks), the spin, the
/// nearest-chaseable homing blend and the no-target sine sway. The 1116
/// (`flag2`) swaps the light colour, the alpha remap, the acquire range
/// (400, no LOS floor), the homing constants (6/30) and the sway (×0.1).
///
/// RNG: the first tick draws `Next(8)` frame (`:40483`), `NextFloat`
/// rotation (`:40485`) and `NextFloat` scale (`:40486`) — in that order.
/// The `SoundID.Item17` cue (`:40481`, 1038 only, `netMode != 2`) rides
/// `wants_sound`. `Lighting.AddLight` (`:40507`/`:40513`) feeds the light
/// layer only — skipped.
///
/// Both rows' movement differs: the 1038 keeps `tileCollide = false`
/// (`:9803`) — the free integrate; the 1116 keeps stock `tileCollide = true`
/// (`:10413-10426` sets no override) and its blocked-move arm
/// (`:18068-18078`) reverses the blocked axis at ×0.25 — [`petal_storm_move`]
/// ports that bounce (the `ai_music_note` per-axis probe stand-in).
pub(crate) fn ai_petal(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
) {
    // `:40468-40475` — the drift flag (read BEFORE the counter advances)
    // and the 1116 variant's colour/alpha.
    let flag = p.ai[0] < 20.0;
    let flag2 = p.kind == 1116;
    if flag2 {
        // `alpha = (int)Remap(timeLeft, 30, 0, 0, 255)`.
        p.alpha = remap(p.time_left as f32, 30.0, 0.0, 0.0, 255.0) as i32;
    }
    // `:40476-40491` — the first-tick latch: the cue (1038 only), the
    // frame roll, the sprite side, the rotation and scale draws.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        if !flag2 {
            // `SoundID.Item17` at base.Center (netMode != 2 folds open).
            let cx = p.pos.x + p.w * 0.5;
            let cy = p.pos.y + p.h * 0.5;
            p.wants_sound = Some((2, 17, cx, cy, 0.0));
        }
        p.frame = rng.next_range(0, 8);
        p.sprite_direction = if !(p.vel.x < 0.0) { 1 } else { -1 };
        p.rot = rng.next_float() * std::f32::consts::TAU;
        p.scale = 0.6 + 0.3 * rng.next_float();
        if flag2 {
            p.scale *= 0.75;
        }
    }
    // `:40492-40499` — the 8-frame cycle at 4 ticks.
    p.frame_counter += 1;
    if p.frame_counter >= 4 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= 8 {
            p.frame = 0;
        }
    }
    // `:40500-40501` — the spin and the age counter.
    p.rot += p.vel.x * 0.1 + p.vel.y * 0.1;
    p.ai[0] += 1.0;
    // `:40502-40510` — the drift window: the ×0.95 damp, then return (the
    // AddLight at ai[0] >= 10 is the light skip).
    if flag {
        p.vel *= 0.95;
        return;
    }
    // `:40515-40537` — the nearest-chaseable acquire. num = 2000 (400 for
    // 1116), num2 = 400 (0 for 1116): the LOS floor. `CanBeChasedBy(this)`
    // is the plain chaseable gate.
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let (mut num, num2) = if flag2 { (400.0f32, 0.0f32) } else { (2000.0, 400.0) };
    let mut target = Vec2::ZERO;
    let mut found = false;
    for n in npcs {
        if chaseable(n) {
            let nc = npc_center(n);
            let num3 = center.distance(nc);
            // `!(num3 >= num) && (!(num3 > num2) || CanHit(this, nPC))`.
            if num3 < num && (num3 <= num2 || can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, n.size().0, n.size().1)) {
                num = num3;
                target = nc;
                found = true;
            }
        }
    }
    // `:40538-40559` — the homing blend or the sine sway.
    if found {
        let (num4, num5) = if flag2 { (6.0f32, 30.0f32) } else { (24.0, 15.0) };
        let vector3 = safe_normalize(target - center) * num4;
        p.vel = (p.vel * num5 + vector3) / (num5 + 1.0);
    } else if flag2 {
        // `Lerp(vel.X, Sin(ai[0] * 0.1) * 1.5, 0.1)` / `Lerp(vel.Y, 0.3, 0.1)`.
        let tx = (p.ai[0] * 0.1).sin() * 1.5;
        p.vel.x += (tx - p.vel.x) * 0.1;
        p.vel.y += (0.3 - p.vel.y) * 0.1;
    } else {
        let tx = p.ai[0].sin() * 2.5;
        p.vel.x += (tx - p.vel.x) * 0.1;
        p.vel.y += (0.3 - p.vel.y) * 0.1;
    }
}

/// The 1116 Petalstorm shot's blocked-move arm (`HandleMovement`
/// `Projectile.cs:18068-18078`): the blocked axis reverses at ×0.25 of the
/// pre-collision velocity (no speed gate, unlike the 1020 drone's >1f),
/// then `UpdatePosition` integrates the result. The port's per-axis
/// destination-box probe stands in for the swept `TileCollision` (the
/// `ai_music_note` precedent, the recorded death-pixel deviation).
pub(crate) fn petal_storm_move(p: &mut Projectile, world: &World) {
    let last = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x {
        p.vel.x = -last.x * 0.25;
    }
    if hit_y {
        p.vel.y = -last.y * 0.25;
    }
    p.pos += p.vel;
}

/// `AI_192_GetJuminoFall` (`Projectile.cs:41501-41514`) — the jumino's fall
/// offset: `timeSinceFall = max(ai[0] - 420, 0)`, `fall = -4*t + 0.2*t²/2`
/// capped at 24. The two `out` params map to the returned pair.
pub(crate) fn jumino_fall(ai0: f32) -> (f32, f32) {
    let mut time_since_fall = ai0 - 420.0;
    if time_since_fall < 0.0 {
        time_since_fall = 0.0;
    }
    let mut fall = -4.0 * time_since_fall + 0.2 * (time_since_fall * time_since_fall) / 2.0;
    if fall > 24.0 {
        fall = 24.0;
    }
    (time_since_fall, fall)
}

/// `AI_192_JuminoAnimation` (`Projectile.cs:41516-41638`) — the jumino
/// stardrop animation (995). The 20-frame sheet machine (the fall frames
/// advance at `num4` = 4, or 60 while frame == 1), the opacity ramp, the
/// five dust windows and the completion gift (`Item.NewItem` 5276).
///
/// RNG (the explicit call-site draws stay in order; the dust bodies skip):
/// - the 60..130 window (`:41561-41581`): one `NextFloat` gate; on the taken
///   branch the `NextVector2Circular(2,2)` pair; then TWO iterations each a
///   `NextFloat` gate, on the taken branch a `NextFloatDirection` + two
///   `NextFloat` (the speed `2 + NextFloat + NextFloat*3*num6`).
/// - the <=60 window (`:41582-41603`): TWO iterations each a `NextFloat`
///   gate, on the taken branch a `NextFloatDirection` + one `NextFloat`; at
///   ai[0]==58 a 12-iteration ring each a `NextFloatDirection` + one
///   `NextFloat`.
/// - the >=num5-20 window (`:41604-41623`): FIVE dust each a `NextFloat`
///   fadeIn; then ONE dust (no draw).
/// - the 1-in-6 sparkle (`:41624-41631`): `Next(6)` while 90 <= ai[0] < num.
/// `Lighting.AddLight` is absent here; the dust colour is a constant.
///
/// The completion (`:41632-41637`) queues the gift item and kills. The row
/// keeps `tileCollide = false` (`:9454`) — the free integrate.
pub(crate) fn ai_jumino(
    p: &mut Projectile,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    let num = 420.0 + 120.0; // 540
    let num2 = num - 150.0; // 390
    // `:41520-41530` — the partial-opacity dust (one, no gate draw; the
    // NewDustDirect body skips, so the opacity read feeds nothing here).
    // `:41531-41556` — the frame machine. num3 = 20 frames, num4 = 4 (60
    // while frame == 1).
    let num3 = 20;
    let mut num4 = 4;
    if p.frame == 1 {
        num4 = 60;
    }
    let (time_since_fall, fall) = jumino_fall(p.ai[0]);
    if time_since_fall > 0.0 {
        if fall < 20.0 {
            p.frame = 0;
        } else {
            p.frame_counter += 1;
            if p.frame_counter >= num4 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame >= num3 {
                    p.frame = num3 - 1;
                }
            }
        }
    } else {
        p.frame = 1;
    }
    // `:41557` — the sprite faces the owner (>= 0 → 1). The port's single
    // player rides slot 0; no player keeps the own centre comparison.
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let owner_x = player.map(|pl| pl.center().0).unwrap_or(center.x);
    let dir = if owner_x - center.x >= 0.0 { 1 } else { -1 };
    p.direction = dir;
    p.sprite_direction = dir;
    // `:41558-41560` — the opacity ramp and the age counter. num5 = num +
    // num3*num4 + 30 (num4 is the frame==1 ? 60 : 4 read above).
    let num5 = num + (num3 * num4) as f32 + 30.0;
    let op = remap(p.ai[0], num2, num2 + 20.0, 0.0, 1.0)
        * remap(p.ai[0], num5 - 5.0, num5, 1.0, 0.5);
    p.alpha = alpha_of(op);
    p.ai[0] += 1.0;
    // `:41561-41581` — the 60..130 gather window.
    if p.ai[0] <= 130.0 && p.ai[0] >= 60.0 {
        let num6 = remap(p.ai[0], 60.0, 130.0, 0.0, 1.0);
        if rng.next_float() < num6 * 0.5 {
            let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular(2,2)
        }
        let mut num7 = 0.0f32;
        while num7 < 1.0 {
            if rng.next_float() < 0.5 {
                let _ = rng.next_float(); // NextFloatDirection
                let _ = rng.next_float(); // speed: 2 + NextFloat + ...
                let _ = rng.next_float(); // ... + NextFloat * 3 * num6
            }
            num7 += 0.5;
        }
    }
    // `:41582-41603` — the <=60 spawn window.
    if p.ai[0] <= 60.0 {
        let num8 = remap(p.ai[0], 0.0, 60.0, 0.0, 1.0);
        let mut num9 = 0.0f32;
        while num9 < 1.0 {
            if rng.next_float() < 0.5 {
                let _ = rng.next_float(); // NextFloatDirection
                let _ = rng.next_float(); // speed: 2*num8 + NextFloat*num8
            }
            num9 += 0.5;
        }
        if p.ai[0] == 58.0 {
            let mut num10 = 0.0f32;
            while num10 < 1.0 {
                let _ = rng.next_float(); // NextFloatDirection
                let _ = rng.next_float(); // speed: 1*num8 + NextFloat*num8*2
                num10 += 1.0 / 12.0;
            }
        }
    }
    // `:41604-41623` — the >=num5-20 landing window: five fadeIn draws then
    // one no-draw dust.
    if p.ai[0] >= num5 - 20.0 {
        for _ in 0..5 {
            let _ = rng.next_float(); // fadeIn = 0.6 + NextFloat * 0.3
        }
    }
    // `:41624-41631` — the 1-in-6 sparkle gate.
    if rng.next_range(0, 6) == 0 && p.ai[0] >= 90.0 && p.ai[0] < num {
        // The dust body — SKIPPED (no draws beyond the gate).
    }
    // `:41632-41637` — the completion gift and kill.
    if p.ai[0] >= num5 {
        p.wants_gift_item = Some((5276, center.x, center.y + 12.0));
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
}

/// `AI_195_JimsDrone` (`Projectile.cs:40923-41067`) — Jim's Drone (1020),
/// the RC drone the player steers with the run/up/down keys. The owner block
/// packs the input bits into `ai[0]` (the grav-flip XOR on left/right), the
/// altitude/held-item kill gates, the tilt machine (`ai[1]` = rotation), the
/// thrust/drag/cap physics, the `timeLeft = 2` keep-alive and the ground-probe
/// dust.
///
/// The port keeps `gravDir = 1` (no gravity-flip system — the `mounted_top`
/// precedent), so `flag2` is false and the input XOR folds to the raw keys.
/// `Main.DroneCameraTracker.Track` (`:40955`) and
/// `CheckSectionsInCaseOwnerIsWatching` (`:41066`) are camera/section
/// bookkeeping — skipped. The tracked-loop sound (`SoundID.JimsDrone`,
/// `:40986-40997`) is a continuous volume/pitch tracker, not a one-shot cue —
/// skipped (no tracked-sound layer in the pool); `localAI[2]` (the sound slot)
/// stays 0.
///
/// RNG: the ground-probe dust (`:41050-41063`) runs only when airborne
/// (`!flag6`) and the 3-step probe found a floor within 48 px (`num7 != 48`).
/// TWO iterations, each a `NextFloat` gate; on the taken branch
/// (`!(NextFloat < 0.66)`) a `NextVector2CircularEdge(2.5, 1)` (one float,
/// `Utils.cs:1271-1273`) then a `Next(2)` dust-type pick. The dust body skips.
///
/// The row keeps stock `tileCollide = true`; the blocked-move arm
/// (`:18057-18067`) reverses the blocked axis at ×0.25 when |last| > 1 —
/// [`drone_move`] ports that (the `ai_music_note` per-axis probe stand-in).
pub(crate) fn ai_jims_drone(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:40925-40930` — the dead-owner kill flag.
    let mut flag = player.map(|pl| pl.dead).unwrap_or(false);
    // `:40931-40956` — the owner-only block: the altitude kill, the held-item
    // gate and the input-bit pack. The port's single player rides slot 0.
    if p.owner == local_owner {
        // `position.Y - height <= 640` → kill (the drone flew too high).
        if p.pos.y - p.h <= 640.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `HeldItem.type != 5451` (the RC transmitter) → the kill flag.
        if player.map(|pl| pl.held_item_kind).unwrap_or(0) != 5451 {
            flag = true;
        }
        // `gravDir == -1` folds false (the port keeps gravDir = 1).
        let flag2 = false;
        let (cl, cr, cu, cd) = player
            .map(|pl| (pl.control_left, pl.control_right, pl.control_up, pl.control_down))
            .unwrap_or((false, false, false, false));
        // BitsByte: bit0 = left ^ flag2, bit1 = right ^ flag2, bit2 = up,
        // bit3 = down.
        let mut bits = 0u8;
        if cl ^ flag2 {
            bits |= 1;
        }
        if cr ^ flag2 {
            bits |= 2;
        }
        if cu {
            bits |= 4;
        }
        if cd {
            bits |= 8;
        }
        let num = bits as f32;
        if p.ai[0] != num {
            p.ai[0] = num;
            p.net_update = true;
        }
        // Main.DroneCameraTracker.Track — skipped (camera bookkeeping).
    }
    // `:40957-40961` — the kill flag (dead owner / wrong held item).
    if flag {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:40962-40967` — unpack the input bits; rotation = ai[1].
    let bits = p.ai[0] as u8;
    p.rot = p.ai[1];
    let flag3 = bits & 1 != 0; // left
    let flag4 = bits & 2 != 0; // right
    let flag5 = bits & 4 != 0; // up (thrust)
    // `:40968-40972` — the steer-age counter (resets when neither left/right).
    p.local_ai[0] += 1.0;
    if !flag3 && !flag4 {
        p.local_ai[0] = 0.0;
    }
    // `:40973-40983` — the throttle target: 0 grounded, 10 airborne, 100
    // thrusting; localAI[1] lerps toward it at 0.2.
    let flag6 = p.vel.y == 0.0;
    let mut num2 = if flag6 { 0.0f32 } else { 10.0 };
    if flag5 {
        num2 = 100.0;
    }
    p.local_ai[1] += (num2 - p.local_ai[1]) * 0.2;
    // `:40984-40997` — the sound volume/pitch envelope feeds only the skipped
    // tracked-loop sound (num3/num4 are draw-free). localAI[2] stays 0.
    // `:40998-41020` — the tilt/thrust machine.
    let mut num5 = 0.15f32;
    if flag6 {
        // Grounded: damp the tilt ×0.7, drag X ×0.97; thrust adds the
        // rotation-vector kick at 0.5 and lowers gravity to 0.05.
        p.rot = wrap_angle(p.rot) * 0.7;
        p.vel.x *= 0.97;
        if flag5 {
            p.vel += rotation_vector(p.rot - std::f32::consts::FRAC_PI_2) * 0.5;
            num5 = 0.05;
        }
    } else {
        // Airborne: the turn rate ramps 0.1×→1× over the first 10 steer ticks.
        let mut num6 = std::f32::consts::PI / 15.0;
        num6 = remap(p.local_ai[0], 0.0, 10.0, num6 * 0.1, num6);
        p.rot += (flag4 as i32 - flag3 as i32) as f32 * num6;
        if flag5 {
            let vector = rotation_vector(p.rot - std::f32::consts::FRAC_PI_2);
            p.vel.x *= 0.97;
            p.vel += vector * 0.5;
        }
    }
    // `:41021-41030` — gravity, the ai[1] rotation latch, the fall drag and
    // the 16 speed cap.
    p.vel.y += num5;
    p.ai[1] = p.rot;
    if !flag5 && p.vel.y > 12.0 {
        p.vel.y *= 0.95;
    }
    let len = p.vel.length();
    if len > 16.0 {
        p.vel *= 16.0 / len;
    }
    // `:41031-41032` — AddLight skipped; the timeLeft = 2 keep-alive.
    p.time_left = 2;
    // `:41033-41065` — the airborne ground-probe dust. The 3-step (16 px)
    // TileCollision probe accumulates the free-fall distance; a floor within
    // 48 px opens the two-dust window.
    if !flag6 {
        let vector2 = p.pos;
        let mut num7 = 0.0f32;
        let num8 = 3;
        let num9 = 16 * num8;
        for i in 0..num8 {
            let vector3 = Vec2::new(0.0, 16.0);
            let probe_pos = vector2 + vector3 * i as f32;
            let tc = crate::collision::tile_collision(
                world,
                [probe_pos.x, probe_pos.y],
                [vector3.x, vector3.y],
                p.w,
                p.h,
                false,
                false,
            );
            num7 += tc.vel[1];
            if tc.vel != [vector3.x, vector3.y] {
                break;
            }
        }
        if num7 != num9 as f32 {
            for _ in 0..2 {
                if !(rng.next_float() < 0.66) {
                    let _ = rng.next_float(); // NextVector2CircularEdge(2.5, 1)
                    let _ = rng.next_range(0, 2); // the dust-type pick (31 / 16)
                }
            }
        }
    }
    // CheckSectionsInCaseOwnerIsWatching — skipped (section bookkeeping).
}

/// The 1020 drone's blocked-move arm (`HandleMovement`
/// `Projectile.cs:18057-18067`): the blocked axis reverses at ×0.25 of the
/// pre-collision velocity, but ONLY when |last axis| > 1 (the speed gate the
/// 1116 petal lacks). The port's per-axis destination-box probe stands in for
/// the swept `TileCollision` (the `ai_music_note` precedent).
pub(crate) fn drone_move(p: &mut Projectile, world: &World) {
    let last = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x && last.x.abs() > 1.0 {
        p.vel.x = -last.x * 0.25;
    }
    if hit_y && last.y.abs() > 1.0 {
        p.vel.y = -last.y * 0.25;
    }
    p.pos += p.vel;
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`) — the full solid block
/// (active, solid, not solid-top, no half-brick, no slope, not actuated).
/// The flint's surface/ceiling walk reads it (the `npc.rs::solid_block`
/// shape, kept local to avoid the cross-module private fn).
fn flint_solid_tile(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && tr_core::tiledata::TILE_SOLID.has(t.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `WorldGen.ActiveAndWalkableTile` (`WorldGen.cs:70300-70318`) — active,
/// solid, not actuated (solid-top counts).
fn flint_walkable(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind))
}

/// `FindBlockSurface` (`Projectile.cs:52361-52387`) — walk up out of solid
/// (downMax steps), then down to the first walkable (upmax steps). The
/// `y < 10` / `y > maxTilesY - 10` bounds break the walks.
fn find_block_surface(world: &World, x: i32, y: &mut i32, down_max: i32, up_max: i32) {
    for _ in 0..down_max {
        if *y < 10 || !flint_solid_tile(world, x, *y) {
            break;
        }
        *y -= 1;
    }
    for _ in 0..up_max {
        if *y > world.height - 10 || flint_walkable(world, x, *y) {
            break;
        }
        *y += 1;
    }
}

/// `FindBlockCeiling` (`Projectile.cs:52389-52415`) — the gravity-flip
/// mirror: walk down out of solid (upMax), then up to the first walkable
/// (downMax).
fn find_block_ceiling(world: &World, x: i32, y: &mut i32, up_max: i32, down_max: i32) {
    for _ in 0..up_max {
        if *y > world.height - 10 || !flint_solid_tile(world, x, *y) {
            break;
        }
        *y += 1;
    }
    for _ in 0..down_max {
        if *y < 10 || flint_walkable(world, x, *y) {
            break;
        }
        *y -= 1;
    }
}

/// `AI_198_Flint_EmitSpikes` (`Projectile.cs:52317-52359`) — the charged
/// release's ground-spike wave: up to 13 spikes (num3 = 0..12) stepping 2
/// tiles along `direction`, each on the first walkable surface/ceiling tile.
/// The port keeps `gravDir = 1` (flag false → the surface walk).
///
/// RNG: per spawned spike one `NextFloat` (the ai1 scale, `:52346`) then one
/// `NextFloatDirection` (the velocity rotation, `:52352`). The
/// `ParticleOrchestrator` request (`:52322`) is a particle-system spawn —
/// skipped (no particle layer). The returned vec is the `(x, y, vx, vy,
/// ai0, ai1)` spike payloads the pool resolves into 1041 spawns.
fn flint_emit_spikes(
    p: &Projectile,
    world: &World,
    start: Vec2,
    direction: i32,
    rng: &mut dyn DropRng,
) -> Vec<(f32, f32, f32, f32, f32, f32)> {
    let mut out = Vec::new();
    let start = start + Vec2::new(direction as f32 * 30.0, 0.0);
    let mut tx = (start.x / 16.0) as i32;
    let mut ty = (start.y / 16.0) as i32;
    // gravDir = 1 → flag false (the surface walk).
    let num = 12;
    let num2 = 2;
    let mut num3 = 0;
    while num3 <= num {
        find_block_surface(world, tx, &mut ty, 10, 10);
        if flint_walkable(world, tx, ty) {
            // `p.ToWorldCoordinates(8, 0)` = (tx*16 + 8, ty*16).
            let vector = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0);
            let spinningpoint = Vec2::new(direction as f32, -10.0);
            let ai = 1.0 + num3 as f32 * 0.04 + rng.next_float() * 0.2;
            let angle = rng.next_float() * 2.0 - 1.0; // NextFloatDirection
            let vel = rotate(spinningpoint, angle * std::f32::consts::TAU * 0.025);
            out.push((vector.x, vector.y, vel.x, vel.y, -(num3 * 2) as f32, ai));
            num3 += 1;
            tx += direction * num2;
            continue;
        }
        break;
    }
    out
}

/// `AI_198_Flint` (`Projectile.cs:52238-52315`) — the Dead Cells flint
/// (1040), the channel weapon. While channelling (`ai[0] == 0`) it holds the
/// player's facing, winds the dummy item time and rolls the charge dust; on
/// release (`!channel`) it spawns the 1043 slash (damage ×1.78) and — if the
/// charge reached 30 ticks — the ground-spike wave, then enters the
/// follow-through (`ai[0] == 1`) that counts down to `useAnimation` and kills.
///
/// The port keeps `gravDir = 1` (the aim's `ReverseGravitySupport` folds to
/// the raw mouse, the spike walk takes the surface branch). The player-side
/// writes (`ChangeDir`, `heldProj`, `SetDummyItemTime`, `itemAnimationMax`)
/// ride the [`FlintTick`] return → the pool's `GolfEffect::PlayerWrites` seam.
/// The `itemLocation.Y += Next(2)` jitter (`:52309`) keeps its draw on the
/// stream; the draw-side item location is unmodelled (the thrust placement
/// recomputes it each frame — a documented skip).
///
/// RNG (charge phase, `ai[0] == 0`): one `Next(2)` (the itemLocation jitter,
/// `:52309`) then one `Next(10)` gate (`:52310`); on the taken branch the
/// dust's `NextVector2Circular(12,12)` pair + `NextFloat` velocity + `NextFloat`
/// scale (`:52312`). The release's spike wave rides [`flint_emit_spikes`].
/// The `DeadCellsFlintCharge` cue (`:52305`) is a tracked sound — skipped (no
/// tracked-sound layer); the 30-tick charge state still gates the spikes.
pub(crate) fn ai_flint(
    p: &mut Projectile,
    world: &World,
    player: crate::projectile_ai_golf::GolfClubPlayer,
    mouse_world: Vec2,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> FlintTick {
    let mut tick = FlintTick::default();
    // `:52240-52245` — the inactive-owner kill.
    if !player.active {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return tick;
    }
    // `:52246-52260` — the release: !channel while still charging.
    if !player.channel && p.ai[0] == 0.0 {
        let charged = p.ai[1] >= 30.0;
        p.ai[0] = 1.0;
        p.ai[1] = 0.0;
        p.net_update = true;
        if p.owner == local_owner {
            // The 1043 slash: damage ×1.78, velocity (direction, 0),
            // ai0 = direction * gravDir (= direction), ai1 = 6, ai2 = 1.
            let (mcx, mcy) = (player.mounted_center.x, player.mounted_center.y);
            let slash_damage = (p.damage as f32 * 1.78) as i32;
            tick.wants_slash = Some((
                mcx,
                mcy,
                p.direction as f32,
                0.0,
                slash_damage,
                p.knock_back,
                p.direction as f32, // ai0 = direction * gravDir (gravDir = 1)
                6.0,
                1.0,
            ));
            if charged {
                tick.spikes = flint_emit_spikes(
                    p,
                    world,
                    player.mounted_center,
                    p.direction,
                    rng,
                );
            }
        }
    }
    // `:52261-52279` — the follow-through: count to useAnimation, then kill.
    if p.ai[0] == 1.0 {
        p.ai[1] += 1.0;
        let num = player.use_animation as f32;
        if p.ai[1] >= num {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return tick;
        }
        p.vel = Vec2::ZERO;
        tick.facing = Some(p.direction);
        tick.writes_held_proj = true;
        let mut num2 = remap(p.ai[1], 0.0, 5.0, num, 0.0) as i32;
        if num2 < 2 {
            num2 = 2;
        }
        tick.dummy_item_time = Some(num2);
    }
    // `:52280-52314` — the charge hold.
    if p.ai[0] == 0.0 {
        p.vel = Vec2::ZERO;
        if p.owner == local_owner {
            // `ReverseGravitySupport(MouseScreen) + screenPosition` =
            // MouseWorld (gravDir = 1); face toward it from MountedCenter.
            let dx = mouse_world.x - player.mounted_center.x;
            p.direction = if dx >= 0.0 { 1 } else { -1 };
            tick.facing = Some(p.direction);
        }
        p.direction = player.direction;
        tick.writes_held_proj = true;
        let use_animation = player.use_animation;
        let num3 = 3;
        p.ai[1] += 1.0;
        let num4 = p.ai[1].min(num3 as f32);
        let mut num5 = use_animation + num4 as i32;
        if num5 < 2 {
            num5 = 2;
        }
        // The 30-tick charge cue is a tracked sound — skipped (the state
        // still gates the spike wave above).
        tick.item_animation_max = Some(use_animation);
        tick.dummy_item_time = Some(num5);
        // `itemLocation.Y += Next(2)` — the draw stays; the location is
        // draw-side (unmodelled).
        let _ = rng.next_range(0, 2);
        // The 1-in-10 charge dust gate.
        if rng.next_range(0, 10) == 0 {
            let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular(12,12)
            let _ = rng.next_float(); // velocity.Y = NextFloat * -8 - 2
            let _ = rng.next_float(); // scale = 1.3 + NextFloat
        }
    }
    tick
}

/// What one aiStyle-198 tick hands back to the pool: the player-side writes
/// (the `GolfClubTick` shape), the 1043 slash spawn and the 1041 spike wave.
#[derive(Clone, Debug, Default)]
pub(crate) struct FlintTick {
    /// `player.ChangeDir(direction)` (`:52271`/`:52286`).
    pub facing: Option<i32>,
    /// `player.heldProj = whoAmI` (`:52272`/`:52289`).
    pub writes_held_proj: bool,
    /// `player.SetDummyItemTime(frames)` (`:52278`/`:52308`).
    pub dummy_item_time: Option<i32>,
    /// `player.itemAnimationMax = useAnimation` (`:52307`).
    pub item_animation_max: Option<i32>,
    /// The 1043 release slash: `(x, y, vx, vy, damage, kb, ai0, ai1, ai2)`.
    pub wants_slash: Option<(f32, f32, f32, f32, i32, f32, f32, f32, f32)>,
    /// The 1041 ground spikes: `(x, y, vx, vy, ai0, ai1)` per spike.
    pub spikes: Vec<(f32, f32, f32, f32, f32, f32)>,
}

/// `AI_191_TrueNightsEdge` (`Projectile.cs:41640-41808`) — the True Night's
/// Edge spin slash (973) and the Terra Blade's second shot (985). Both ride
/// the `localAI[0]` age / `ai[1]` lifetime envelope: the opacity ramp, the
/// `num4`-tick kill, then the per-type choreography.
///
/// 973: the spin (`rotation += ai[0] * TAU * (4 + Opacity*4) / 90`), the
/// scale remap, the owner-anchored centre (`RotatedRelativePoint(MountedCenter)
/// - velocity + velocity * num6² * 77`), the damage-zeroing LOS probe and the
/// two dust windows. 985: the velocity-aligned rotation, the eased scale, the
/// >8 speed drag ladder (the LOS probe sets `localAI[1]`), the `ownerHitCheck`
/// window (`localAI[0] <= 6`), the damage-zero at the 0.65 lerp and the three
/// dust windows.
///
/// RNG (973): one `NextFloatDirection` (the dust angle, `:41711`), one
/// `Next(5)` gate (`:41713`), then per opacity-dust iteration a `NextFloat`
/// (the 75/27 type pick, `:41721`). (985): one `NextFloatDirection`
/// (`:41770`), then THREE iterations each a `NextFloat` gate (`:41778`); the
/// damage==0 / early-out window adds four iterations each a
/// `NextFloatDirection` + `NextFloat` + `NextFloat` (`:41798-41805`). The dust
/// bodies / colour lerps skip; `Lighting.AddLight` (`:41775`) skips.
///
/// The 973 row keeps `tileCollide = false` (free integrate); the 985 keeps
/// stock true and its blocked-move arm (`:18104-18110`) steps once, restores
/// `lastVelocity * 0.01` and sets `localAI[1] = 1` — [`terra_blade_move`].
pub(crate) fn ai_true_nights_edge(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:41642-41653` — the envelope constants; the 985 swaps them.
    let mut num = 50.0f32;
    let num2 = 15.0f32;
    let mut num3 = p.ai[1] + num;
    let mut num4 = num3 + num2;
    let mut num5 = 77.0f32;
    let is_terra = p.kind == 985;
    if is_terra {
        num = 0.0;
        num3 = p.ai[1];
        num4 = p.ai[1] + 25.0;
        num5 = num4;
    }
    // `:41654-41657` — the 973 first-tick Item8 cue.
    if p.local_ai[0] == 0.0 && p.kind == 973 {
        p.wants_sound = Some((2, 8, p.pos.x, p.pos.y, 0.0));
    }
    // `:41658-41666` — the age counter and the two fast-forward arms.
    p.local_ai[0] += 1.0;
    if is_terra && p.local_ai[1] == 1.0 {
        p.local_ai[0] += 2.0;
    }
    if p.kind == 973 && p.damage == 0 && p.local_ai[0] < num3 + (num4 - num3) * 0.5 {
        p.local_ai[0] += 6.0;
    }
    // `:41667` — the opacity envelope.
    let opacity = remap(p.local_ai[0], 0.0, p.ai[1], 0.0, 1.0)
        * remap(p.local_ai[0], num3, num4, 1.0, 0.0);
    p.alpha = alpha_of(opacity);
    // `:41668-41673` — the kill at num4.
    if p.local_ai[0] >= num4 {
        p.local_ai[1] = 1.0;
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:41674-41677` — the owner anchor and the facing.
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let from_value = p.local_ai[0] / p.ai[1];
    let mut num6 = remap(p.local_ai[0], p.ai[1] * 0.4, num4, 0.0, 1.0);
    p.direction = p.ai[0] as i32;
    p.sprite_direction = p.ai[0] as i32;
    // `:41678-41725` — the 973 spin-slash branch.
    if p.kind == 973 {
        let num7 = 3.0f32;
        if p.damage != 0 && p.local_ai[0] >= num5 + num7 {
            p.damage = 0;
        }
        if p.damage != 0 {
            // The 5-ray LOS probe (`:41687-41703`): num8 = 80, the rays at
            // ±0.25·(π/4) around the velocity angle; a clear ray keeps damage.
            let num8 = 80.0f32;
            let mut flag = false;
            let num9 = p.vel.y.atan2(p.vel.x);
            let mut num10 = -1.0f32;
            while num10 <= 1.0 {
                let ang = num9 + num10 * (std::f32::consts::FRAC_PI_4) * 0.25;
                let position = center + rotation_vector(ang) * num8 * 0.5 * p.scale;
                let position2 = center + rotation_vector(ang) * num8 * p.scale;
                if !solid_tiles(world, center.x, center.y, 0.0, 0.0)
                    && can_hit(world, position.x, position.y, 0.0, 0.0, position2.x, position2.y, 0.0, 0.0)
                {
                    flag = true;
                    break;
                }
                num10 += 0.5;
            }
            if !flag {
                p.damage = 0;
            }
        }
        // `:41705-41710` — the localAI[1] counter, the eased centre, the spin
        // and the scale.
        p.local_ai[1] += 1.0;
        num6 = remap(p.local_ai[1], p.ai[1] * 0.4, num4, 0.0, 1.0);
        if let Some(pl) = player {
            let mc = pl.mounted_center();
            let anchor = pl.rotated_relative_point(Vec2::new(mc.0, mc.1), 0);
            let new_center = anchor - p.vel + p.vel * num6 * num6 * num5;
            p.pos = new_center - Vec2::new(p.w * 0.5, p.h * 0.5);
        }
        p.rot += p.ai[0] * std::f32::consts::TAU * (4.0 + opacity * 4.0) / 90.0;
        p.scale = remap(p.local_ai[0], p.ai[1] + 2.0, num4, 1.12, 1.0) * p.ai[2];
        // `:41711-41724` — the dust angle draw, the 1-in-5 gate and the
        // opacity-dust loop (the bodies skip).
        let _f = p.rot + (rng.next_float() * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2 * 0.7;
        if rng.next_range(0, 5) == 0 {
            // The dust-14 body — SKIPPED.
        }
        let mut i = 0.0f32;
        while i < 3.0 * opacity {
            let _ = rng.next_float(); // the 75/27 type pick
            i += 1.0;
        }
    }
    // `:41726-41729` — only the 985 continues past here.
    if !is_terra {
        return;
    }
    // `:41730-41739` — the ownerHitCheck window, the damage-zero, the eased
    // scale and the velocity-aligned rotation.
    p.owner_hit_check = p.local_ai[0] <= 6.0;
    if p.local_ai[0] >= num3 + (num4 - num3) * 0.65 {
        p.damage = 0;
    }
    let from_value2 = 1.0 - (1.0 - num6) * (1.0 - num6);
    p.rot = p.vel.y.atan2(p.vel.x);
    p.scale = remap(from_value2, 0.0, 1.0, 1.5, 1.0) * p.ai[2];
    num6 = remap(p.local_ai[0], p.ai[1] / 2.0, num4, 0.0, 1.0);
    let opacity2 = remap(p.local_ai[0], 0.0, p.ai[1] * 0.5, 0.0, 1.0)
        * remap(p.local_ai[0], num4 - 12.0, num4, 1.0, 0.0);
    p.alpha = alpha_of(opacity2);
    // `:41740-41769` — the >8 speed drag ladder.
    if p.vel.length() > 8.0 {
        p.vel *= 0.94;
        let num11 = remap(from_value, 0.7, 1.0, 110.0, 110.0);
        if p.local_ai[1] == 0.0 {
            // The 5-ray LOS probe (`:41747-41759`): a blocked ray sets the
            // localAI[1] retract latch.
            let mut flag2 = false;
            let mut num12 = -1.0f32;
            while num12 <= 1.0 {
                let ang = p.rot + num12 * std::f32::consts::FRAC_PI_4 * 0.25;
                let a = center + rotation_vector(ang) * num11 * 0.5 * p.scale;
                let b = center + rotation_vector(ang) * num11 * p.scale;
                if can_hit(world, a.x, a.y, 0.0, 0.0, b.x, b.y, 0.0, 0.0) {
                    flag2 = true;
                    break;
                }
                num12 += 0.5;
            }
            if !flag2 {
                p.local_ai[1] = 1.0;
            }
        }
        if p.local_ai[1] == 1.0 && p.vel.length() > 8.0 {
            p.vel *= 0.8;
        }
        if p.local_ai[1] == 1.0 {
            p.vel *= 0.88;
        }
    }
    // `:41770-41786` — the dust angle draw and the three-gate dust loop.
    let _num13 = p.rot + (rng.next_float() * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2 * 0.9;
    for _ in 0..3 {
        if rng.next_float() < opacity2 + 0.1 {
            // The dust-107 body (colour lerp + velocity blends) — SKIPPED.
        }
    }
    // `:41787-41791` — the damage==0 fast-forward + drag.
    if p.damage == 0 {
        p.local_ai[0] += 3.0;
        p.vel *= 0.76;
    }
    // `:41792-41807` — the early-out / retract dust window.
    if p.local_ai[0] < 10.0 && (p.local_ai[1] == 1.0 || p.damage == 0) {
        p.local_ai[0] += 1.0;
        p.vel *= 0.85;
        for _ in 0..4 {
            let _ = rng.next_float(); // NextFloatDirection
            let _ = rng.next_float(); // the velocity rotation's NextFloat
            let _ = rng.next_float(); // the position pull's NextFloat
        }
    }
}

/// The 985 TerraBlade2Shot's blocked-move arm (`HandleMovement`
/// `Projectile.cs:18104-18110`): step once into the tile, restore
/// `lastVelocity * 0.01` and set the `localAI[1]` retract latch. The port's
/// per-axis destination-box probe stands in for the swept `TileCollision`.
pub(crate) fn terra_blade_move(p: &mut Projectile, world: &World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x || hit_y {
        p.pos += p.vel;
        p.vel = last * 0.01;
        p.local_ai[1] = 1.0;
    } else {
        p.pos += p.vel;
    }
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

    // ---- W4-d20 aiStyle 200 (BirdDroppings) — WRITTEN NOT RUN ----

    /// The row carries the census slice (`Projectile.cs:10078-10086`):
    /// 10x10, aiStyle 200, alpha 255, penetrate -1, ignoreWater, stock
    /// tileCollide true.
    #[test]
    fn w4_d20_the_bird_dropping_row_carries_style_200() {
        let d = proj_data(1080).unwrap();
        assert_eq!(d.ai_style, 200);
        assert_eq!((d.w, d.h), (10.0, 10.0), "`:10080-10081`");
        assert_eq!(d.alpha, 255, "`:10083`");
        assert_eq!(d.penetrate, -1, "`:10084`");
        assert!(d.ignore_water, "`:10085`");
        assert!(d.tile_collide, "the stock true (`:787`)");
    }

    /// The gravity ramp and the 16 cap (`:39987-39991`): +0.3 per tick,
    /// clamped at 16. The RNG census: 2 range picks + 2 float scales per
    /// tick (both branches draw the scale).
    #[test]
    fn w4_d20_the_bird_dropping_falls_with_the_cap_and_the_census() {
        let mut p = proj(1080);
        p.vel = Vec2::new(1.0, 0.0);
        let mut rng = census();
        ai_bird_droppings(&mut p, None, &mut rng);
        assert_eq!(p.vel.y, 0.3, "`:39987`");
        assert_eq!(rng.ranges, vec![(0, 2), (0, 2)], "the two branch picks");
        assert_eq!(rng.floats, 2, "both branches draw the scale");
        // The cap: a fast fall clamps at 16, never overshoots.
        p.vel.y = 15.9;
        ai_bird_droppings(&mut p, None, &mut census());
        assert_eq!(p.vel.y, 16.0, "`:39988-39990`");
    }

    /// The player-contact kill (`:40009-40016`): the `(int)` truncating
    /// rects, XNA's strict `Intersects`. An overlapping living player kills
    /// the dropping; a dead player or a non-overlap keeps it alive.
    #[test]
    fn w4_d20_the_bird_dropping_kills_on_player_contact() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.x = 100.0;
        pl.y = 100.0;
        // Overlap: the dropping's box sits inside the player's.
        let mut p = proj(1080);
        p.pos = Vec2::new(105.0, 105.0);
        ai_bird_droppings(&mut p, Some(&pl), &mut census());
        assert!(!p.active, "the contact killed it");
        // A dead player never kills.
        let mut dead = crate::player::Player::spawn(&w);
        dead.x = 100.0;
        dead.y = 100.0;
        dead.dead = true;
        let mut p2 = proj(1080);
        p2.pos = Vec2::new(105.0, 105.0);
        ai_bird_droppings(&mut p2, Some(&dead), &mut census());
        assert!(p2.active, "a dead player is skipped (`:40012`)");
        // No overlap keeps it alive.
        let mut p3 = proj(1080);
        p3.pos = Vec2::new(500.0, 500.0);
        ai_bird_droppings(&mut p3, Some(&pl), &mut census());
        assert!(p3.active, "no overlap, no kill");
    }

    /// The dispatch wires style 200: one pool update advances the fall and
    /// integrates the position (the tileCollide-true free move).
    #[test]
    fn w4_d20_the_dispatch_arms_style_200() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let s = pool.spawn_with_data(1080, 1600.0, 400.0, 2.0, 0.0, 0, 0.0).unwrap();
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
        let p = &pool.slots()[s];
        assert_eq!(p.vel.y, 0.3, "the gravity ticked");
        assert_ne!(p.pos, Vec2::new(1600.0, 400.0), "the movement ran");
    }

    // ---- W4-d20 aiStyle 199 (MeteorOre) — WRITTEN NOT RUN ----

    /// The row carries the census slice (`Projectile.cs:10067-10077`):
    /// 14x14, aiStyle 199, penetrate -1, trap, netImportant, stock
    /// tileCollide true; the 3-frame sheet (`Main.cs:8443`).
    #[test]
    fn w4_d20_the_meteor_ore_row_carries_style_199() {
        let d = proj_data(1078).unwrap();
        assert_eq!(d.ai_style, 199);
        assert_eq!((d.w, d.h), (14.0, 14.0), "`:10069-10070`");
        assert_eq!(d.penetrate, -1, "`:10074`");
        assert!(d.trap, "`:10075`");
        assert!(d.net_important, "`:10076`");
        assert!(d.tile_collide, "the stock true (`:787`)");
    }

    /// The first-tick frame roll + the per-tick dust census (`:40021-40035`):
    /// dry = Next(3) + 2x Next(5) (dust 31) + Next(5) + 2x Next(-40,41)
    /// (dust 6). The wet slot skips the dust-6 branch.
    #[test]
    fn w4_d20_the_meteor_ore_census_dry_and_wet() {
        let mut p = proj(1078);
        p.vel = Vec2::new(3.0, 1.0);
        let mut rng = census();
        ai_meteor_ore(&mut p, &mut rng);
        assert_eq!(p.local_ai[0], 1.0, "the first-tick latch");
        assert_eq!(
            rng.ranges,
            vec![(0, 3), (0, 5), (0, 5), (0, 5), (-40, 41), (-40, 41)],
            "the dry census in C# order"
        );
        // The rotation: vel.x * 0.06 + direction * 0.06 (direction = +1).
        assert!((p.rot - (3.0 * 0.06 + 0.06)).abs() < 1e-6, "`:40039-40040`");
        // A second tick: no frame roll, the same dust census.
        let mut rng2 = census();
        ai_meteor_ore(&mut p, &mut rng2);
        assert_eq!(
            rng2.ranges,
            vec![(0, 5), (0, 5), (0, 5), (-40, 41), (-40, 41)],
            "no repeat frame roll"
        );
        // Wet: the dust-6 branch is skipped.
        let mut wet = proj(1078);
        wet.wet = true;
        let mut rng3 = census();
        ai_meteor_ore(&mut wet, &mut rng3);
        assert_eq!(
            rng3.ranges,
            vec![(0, 3), (0, 5), (0, 5)],
            "the wet census drops the dust-6 draws (`:40030`)"
        );
    }

    /// The dispatch wires style 199: one pool update rolls the frame and
    /// integrates the position.
    #[test]
    fn w4_d20_the_dispatch_arms_style_199() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let s = pool.spawn_with_data(1078, 1600.0, 400.0, 3.0, 1.0, 20, 5.0).unwrap();
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
        let p = &pool.slots()[s];
        assert_eq!(p.local_ai[0], 1.0, "the first-tick latch");
        assert_ne!(p.pos, Vec2::new(1600.0, 400.0), "the movement ran");
    }

    // ---- W4-d20 aiStyle 196 (Petal) — WRITTEN NOT RUN ----

    /// The two rows carry the census slice (`Projectile.cs:9793-9806` /
    /// `:10413-10426`): 18x18, aiStyle 196, timeLeft 120, penetrate 1, the
    /// LOCAL pair (-1), ignoreWater; the 1038 tileCollide false, the 1116
    /// stock true + magic.
    #[test]
    fn w4_d20_the_petal_rows_carry_style_196() {
        let a = proj_data(1038).unwrap();
        assert_eq!(a.ai_style, 196);
        assert_eq!((a.w, a.h), (18.0, 18.0));
        assert_eq!(a.time_left, 120);
        assert_eq!(a.penetrate, 1);
        assert!(a.uses_local_immunity && a.local_hit_cooldown == -1);
        assert!(!a.tile_collide, "the 1038 sets false (`:9803`)");
        assert!(a.ignore_water);
        assert!(!a.magic, "the 1038 sets no magic");
        let b = proj_data(1116).unwrap();
        assert_eq!(b.ai_style, 196);
        assert!(b.tile_collide, "the 1116 keeps stock true");
        assert!(b.magic, "`:10424`");
        assert!(b.ignore_water);
    }

    /// The first-tick census (`:40476-40491`): Next(8) frame, NextFloat
    /// rotation, NextFloat scale — in order. The 1038 queues the Item17 cue;
    /// the 1116 does not and scales ×0.75.
    #[test]
    fn w4_d20_the_petal_first_tick_census_and_cue() {
        let w = world();
        let mut p = proj(1038);
        p.vel = Vec2::new(2.0, 0.0);
        let mut rng = census();
        ai_petal(&mut p, &w, &[], &mut rng);
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(rng.ranges, vec![(0, 8)], "the frame roll");
        assert_eq!(rng.floats, 2, "rotation + scale");
        assert_eq!(p.wants_sound.map(|r| (r.0, r.1)), Some((2, 17)), "Item17");
        assert_eq!(p.sprite_direction, 1, "vel.x >= 0");
        // The 1116: no cue, the ×0.75 scale.
        let mut q = proj(1116);
        q.vel = Vec2::new(-2.0, 0.0);
        let mut rng2 = census();
        ai_petal(&mut q, &w, &[], &mut rng2);
        assert_eq!(q.wants_sound, None, "the 1116 plays no cue (`:40479`)");
        assert_eq!(q.sprite_direction, -1, "vel.x < 0");
        // scale = (0.6 + 0.3 * 0) * 0.75 = 0.45 (census NextFloat = 0).
        assert!((q.scale - 0.45).abs() < 1e-6, "the ×0.75 (`:40489`)");
    }

    /// The drift window (`:40502-40510`): while ai[0] < 20 the velocity
    /// damps ×0.95 and the arm returns before the homing scan.
    #[test]
    fn w4_d20_the_petal_drift_damps_and_returns() {
        let w = world();
        let mut p = proj(1038);
        p.local_ai[0] = 1.0; // past the first tick
        p.ai[0] = 5.0;
        p.vel = Vec2::new(4.0, 2.0);
        ai_petal(&mut p, &w, &[], &mut census());
        assert!((p.vel.x - 4.0 * 0.95).abs() < 1e-6, "the ×0.95 damp");
        assert!((p.vel.y - 2.0 * 0.95).abs() < 1e-6);
        assert_eq!(p.ai[0], 6.0, "the counter advanced");
    }

    /// The no-target sine sway (`:40555-40558`): past the drift, with no
    /// chaseable NPC, vel.X lerps toward Sin(ai[0]) * 2.5 and vel.Y toward
    /// 0.3, both at 0.1.
    #[test]
    fn w4_d20_the_petal_no_target_sways() {
        let w = world();
        let mut p = proj(1038);
        p.local_ai[0] = 1.0;
        p.ai[0] = 25.0; // past the drift; the counter advances to 26
        p.vel = Vec2::ZERO;
        ai_petal(&mut p, &w, &[], &mut census());
        let tx = 26.0f32.sin() * 2.5;
        assert!((p.vel.x - tx * 0.1).abs() < 1e-5, "the X sway");
        assert!((p.vel.y - 0.3 * 0.1).abs() < 1e-6, "the Y sway");
    }

    /// The homing blend (`:40538-40549`): a chaseable NPC in range pulls the
    /// velocity via (v * 15 + dir * 24) / 16 for the 1038.
    #[test]
    fn w4_d20_the_petal_homes_the_nearest_chaseable() {
        let w = world();
        let mut p = proj(1038);
        p.local_ai[0] = 1.0;
        p.ai[0] = 25.0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::ZERO;
        let mut n = npc_at(1700.0, 800.0);
        n.hp = 40;
        n.max_hp = 40;
        ai_petal(&mut p, &w, &[n], &mut census());
        // dir = (1, 0) * 24; vel = (0 * 15 + (24, 0)) / 16 = (1.5, 0).
        assert!((p.vel.x - 1.5).abs() < 1e-5, "the homing blend");
        assert!(p.vel.y.abs() < 1e-6);
    }

    /// The 1116 bounce (`:18068-18078`): the blocked axis reverses at ×0.25.
    #[test]
    fn w4_d20_the_petalstorm_bounce_reverses_at_quarter() {
        let mut w = world();
        // A solid wall at tile x=102 (spans 1632..1648). The shot's right
        // edge at 1618 + vel 20 = 1638 overlaps it — the X move is blocked.
        put(&mut w, 102, 50, 1);
        let mut p = proj(1116);
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(20.0, 0.0);
        petal_storm_move(&mut p, &w);
        assert!((p.vel.x - (-20.0 * 0.25)).abs() < 1e-6, "the ×0.25 reversal");
    }

    /// The dispatch wires style 196: one pool update advances the counter
    /// and integrates the position for both kinds.
    #[test]
    fn w4_d20_the_dispatch_arms_style_196() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let a = pool.spawn_with_data(1038, 1600.0, 400.0, 3.0, 0.0, 20, 2.0).unwrap();
        let b = pool.spawn_with_data(1116, 1600.0, 500.0, 3.0, 0.0, 20, 2.0).unwrap();
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
        assert_eq!(pool.slots()[a].ai[0], 1.0, "the 1038 counter");
        assert_eq!(pool.slots()[b].ai[0], 1.0, "the 1116 counter");
        assert_ne!(pool.slots()[a].pos, Vec2::new(1600.0, 400.0));
        assert_ne!(pool.slots()[b].pos, Vec2::new(1600.0, 500.0));
    }

    // ---- W4-d20 aiStyle 192 (JuminoAnimation) — WRITTEN NOT RUN ----

    /// The row carries the census slice (`Projectile.cs:9446-9457`):
    /// 240x104, aiStyle 192, penetrate -1, tileCollide false, ignoreWater,
    /// timeLeft 18000 (3600 * 5); the 20-frame sheet (`Main.cs:8709`).
    #[test]
    fn w4_d20_the_jumino_row_carries_style_192() {
        let d = proj_data(995).unwrap();
        assert_eq!(d.ai_style, 192);
        assert_eq!((d.w, d.h), (240.0, 104.0), "`:9449-9450`");
        assert_eq!(d.penetrate, -1, "`:9453`");
        assert!(!d.tile_collide, "`:9454`");
        assert!(d.ignore_water, "`:9455`");
        assert_eq!(d.time_left, 18000, "`:9456` timeLeft *= 5");
    }

    /// The fall helper (`:41501-41514`): timeSinceFall = max(ai0 - 420, 0),
    /// fall = -4t + 0.1t² capped at 24.
    #[test]
    fn w4_d20_the_jumino_fall_helper() {
        let (t, f) = jumino_fall(400.0);
        assert_eq!(t, 0.0, "before 420 the fall time is 0");
        assert_eq!(f, 0.0);
        let (t, f) = jumino_fall(421.0);
        assert_eq!(t, 1.0);
        assert!((f - (-4.0 + 0.1)).abs() < 1e-6, "-4*1 + 0.2*1/2");
        // The cap: a large t clamps fall at 24.
        let (_, f) = jumino_fall(500.0);
        assert_eq!(f, 24.0, "the 24 cap (`:41510-41513`)");
    }

    /// The pre-fall frame pin (`:41553-41556`): while ai[0] < 420 the frame
    /// is 1; the counter advances and the sprite faces the owner.
    #[test]
    fn w4_d20_the_jumino_pre_fall_pins_frame_one() {
        let mut p = proj(995);
        p.ai[0] = 10.0;
        p.frame = 5;
        ai_jumino(&mut p, None, &mut census());
        assert_eq!(p.frame, 1, "the pre-fall pin (`:41555`)");
        assert_eq!(p.ai[0], 11.0, "the counter advanced");
        assert_eq!(p.sprite_direction, 1, "no owner → own centre, >= 0");
    }

    /// The completion gift (`:41632-41637`): at ai[0] >= num5 the arm queues
    /// the 5276 gift at Center + (0, 12) and kills. num5 = 540 + 20*60 + 30
    /// = 1770 when frame == 1 (num4 = 60).
    #[test]
    fn w4_d20_the_jumino_completion_queues_the_gift() {
        let mut p = proj(995);
        p.pos = Vec2::new(1000.0, 500.0);
        p.frame = 1; // num4 = 60 → num5 = 1770
        p.ai[0] = 1769.0; // advances to 1770 = num5
        ai_jumino(&mut p, None, &mut census());
        assert!(!p.active, "the completion kill");
        let center = Vec2::new(1000.0 + 120.0, 500.0 + 52.0);
        assert_eq!(p.wants_gift_item, Some((5276, center.x, center.y + 12.0)));
    }

    /// The dispatch wires style 192: one pool update advances the counter.
    #[test]
    fn w4_d20_the_dispatch_arms_style_192() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let s = pool.spawn_with_data(995, 1600.0, 400.0, 0.0, 0.0, 0, 0.0).unwrap();
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
        assert_eq!(pool.slots()[s].ai[0], 1.0, "the counter advanced");
    }

    // ---- W4-d20 aiStyle 195 (JimsDrone) — WRITTEN NOT RUN ----

    /// The row carries the census slice (`Projectile.cs:9659-9666`): 12x12,
    /// aiStyle 195, penetrate -1, stock tileCollide true.
    #[test]
    fn w4_d20_the_drone_row_carries_style_195() {
        let d = proj_data(1020).unwrap();
        assert_eq!(d.ai_style, 195);
        assert_eq!((d.w, d.h), (12.0, 12.0), "`:9661-9662`");
        assert_eq!(d.penetrate, -1, "`:9665`");
        assert!(d.tile_collide, "the stock true");
    }

    /// The altitude kill (`:40933-40937`): position.Y - height <= 640 kills
    /// the drone (owner-gated).
    #[test]
    fn w4_d20_the_drone_altitude_kill() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 5451;
        let mut p = proj(1020);
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 600.0); // 600 - 12 = 588 <= 640
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut census());
        assert!(!p.active, "the altitude kill (`:40933`)");
    }

    /// The held-item gate (`:40938-40941`): not holding the 5451 transmitter
    /// sets the kill flag (after the altitude check passes).
    #[test]
    fn w4_d20_the_drone_wrong_held_item_kills() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 0; // not the transmitter
        let mut p = proj(1020);
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 2000.0); // well below 640
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut census());
        assert!(!p.active, "the wrong-held-item kill (`:40938`)");
    }

    /// The input-bit pack (`:40942-40954`): left=bit0, right=bit1, up=bit2,
    /// down=bit3 (gravDir=1 folds the XOR off). The keep-alive sets timeLeft=2.
    #[test]
    fn w4_d20_the_drone_packs_the_input_bits() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 5451;
        pl.control_right = true;
        pl.control_up = true;
        let mut p = proj(1020);
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 2000.0);
        p.vel = Vec2::new(0.0, 1.0); // airborne (flag6 false)
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut census());
        assert!(p.active);
        assert_eq!(p.ai[0], 2.0 + 4.0, "right(bit1) + up(bit2)");
        assert!(p.net_update, "the input change dirties the slot");
        assert_eq!(p.time_left, 2, "the keep-alive (`:41032`)");
    }

    /// The grounded tilt damp (`:40999-41008`): vel.Y == 0 wraps the rotation
    /// ×0.7 and drags X ×0.97; gravity is 0.15 (no thrust).
    #[test]
    fn w4_d20_the_drone_grounded_damps_the_tilt() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 5451;
        let mut p = proj(1020);
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 2000.0);
        p.vel = Vec2::new(4.0, 0.0); // grounded (flag6 true)
        p.ai[1] = 1.0; // rotation
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut census());
        assert!((p.vel.x - 4.0 * 0.97).abs() < 1e-6, "the X drag");
        assert!((p.vel.y - 0.15).abs() < 1e-6, "the 0.15 gravity");
        // rotation = wrap(1.0) * 0.7, then ai[1] latches it.
        assert!((p.ai[1] - wrap_angle(1.0) * 0.7).abs() < 1e-6, "the tilt damp");
    }

    /// The drone bounce (`:18057-18067`): the blocked axis reverses at ×0.25
    /// only when |last| > 1; a slow axis keeps its velocity.
    #[test]
    fn w4_d20_the_drone_bounce_speed_gate() {
        let mut w = world();
        put(&mut w, 102, 50, 1); // wall at x 1632..1648
        let mut p = proj(1020);
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(20.0, 0.0); // fast → reverses
        drone_move(&mut p, &w);
        assert!((p.vel.x - (-20.0 * 0.25)).abs() < 1e-6, "the ×0.25 reversal");
        // A slow blocked axis (|last| <= 1) does NOT reverse.
        let mut q = proj(1020);
        q.pos = Vec2::new(1620.0, 800.0); // right edge 1632, +0.5 overlaps wall
        q.vel = Vec2::new(0.5, 0.0);
        drone_move(&mut q, &w);
        assert!((q.vel.x - 0.5).abs() < 1e-6, "the speed gate keeps the slow axis");
    }

    /// The dispatch wires style 195: one pool update packs the input and
    /// keeps the drone alive (timeLeft = 2).
    #[test]
    fn w4_d20_the_dispatch_arms_style_195() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 5451;
        let s = pool.spawn_with_data(1020, 1600.0, 2000.0, 0.0, 1.0, 0, 0.0).unwrap();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            false,
        );
        assert!(pool.slots()[s].active, "the drone keeps alive");
        assert_eq!(pool.slots()[s].time_left, 1, "timeLeft=2 then the tick decrement");
    }

    // ---- W4-d20 aiStyle 198 (Flint) — WRITTEN NOT RUN ----

    /// The flint trio's rows carry their census slices.
    #[test]
    fn w4_d20_the_flint_rows_carry_their_styles() {
        let f = proj_data(1040).unwrap();
        assert_eq!(f.ai_style, 198);
        assert_eq!((f.w, f.h), (2.0, 2.0), "`:9823-9824`");
        assert!(!f.tile_collide, "`:9826`");
        assert!(f.ignore_water, "`:9827`");
        assert_eq!(f.alpha, 255, "`:9831`");
        let s = proj_data(1041).unwrap();
        assert_eq!(s.ai_style, 157);
        assert_eq!(s.penetrate, 4, "`:9844`");
        assert!(s.melee, "`:9841`");
        assert!(s.uses_local_immunity && s.local_hit_cooldown == 25, "`:9845-9846`");
        let sl = proj_data(1043).unwrap();
        assert_eq!(sl.ai_style, 190);
        assert_eq!(sl.penetrate, 2, "`:9865`");
        assert!(sl.melee, "`:9864`");
    }

    /// A flint player snapshot for the arm tests.
    fn flint_player(channel: bool) -> crate::projectile_ai_golf::GolfClubPlayer {
        crate::projectile_ai_golf::GolfClubPlayer {
            active: true,
            center: Vec2::new(1600.0, 800.0),
            bottom: Vec2::new(1600.0, 842.0),
            direction: 1,
            channel,
            use_animation: 20,
            held_item_type: 5451,
            mounted_center: Vec2::new(1600.0, 800.0),
            item_rotation: 0.0,
        }
    }

    /// The inactive-owner kill (`:52240-52245`).
    #[test]
    fn w4_d20_the_flint_inactive_owner_kills() {
        let w = world();
        let mut p = proj(1040);
        p.owner = 0;
        let mut pl = flint_player(true);
        pl.active = false;
        let tick = ai_flint(&mut p, &w, pl, Vec2::new(1700.0, 800.0), 0, &mut census());
        assert!(!p.active, "the orphan kill");
        assert!(tick.wants_slash.is_none());
    }

    /// The charge hold (`:52280-52314`): zero velocity, the facing tracks the
    /// mouse, the dummy item time = useAnimation + min(ai[1], 3), the
    /// itemAnimationMax write, and the Next(2) + Next(10) census.
    #[test]
    fn w4_d20_the_flint_charge_holds_and_winds() {
        let w = world();
        let mut p = proj(1040);
        p.owner = 0;
        p.ai[0] = 0.0;
        p.ai[1] = 0.0;
        let pl = flint_player(true);
        let mut rng = census();
        let tick = ai_flint(&mut p, &w, pl, Vec2::new(1700.0, 800.0), 0, &mut rng);
        assert_eq!(p.vel, Vec2::ZERO, "the charge zeroes velocity");
        assert_eq!(p.direction, 1, "the mouse is to the right");
        assert_eq!(p.ai[1], 1.0, "the wind-up counter");
        assert_eq!(tick.dummy_item_time, Some(20 + 1), "useAnimation + min(1,3)");
        assert_eq!(tick.item_animation_max, Some(20), "`:52307`");
        assert!(tick.writes_held_proj);
        // The census: Next(2) jitter + Next(10) gate (census returns min=0,
        // so the gate is taken → the dust's 2+1+1 floats).
        assert_eq!(rng.ranges, vec![(0, 2), (0, 10)], "the charge census");
        assert_eq!(rng.floats, 4, "Circular pair + velocity + scale");
    }

    /// The release (`:52246-52260`): !channel while charging spawns the 1043
    /// slash (damage ×1.78) and enters the follow-through. A charge < 30
    /// spawns no spikes.
    #[test]
    fn w4_d20_the_flint_release_spawns_the_slash() {
        let w = world();
        let mut p = proj(1040);
        p.owner = 0;
        p.damage = 100;
        p.knock_back = 5.0;
        p.direction = 1;
        p.ai[0] = 0.0;
        p.ai[1] = 10.0; // < 30 → no spikes
        let pl = flint_player(false); // released
        let tick = ai_flint(&mut p, &w, pl, Vec2::new(1700.0, 800.0), 0, &mut census());
        assert_eq!(p.ai[0], 1.0, "the follow-through state");
        let slash = tick.wants_slash.expect("the slash spawn");
        assert_eq!(slash.4, 178, "damage ×1.78 (`:52254`)");
        assert_eq!(slash.2, 1.0, "velocity.X = direction");
        assert_eq!(slash.6, 1.0, "ai0 = direction * gravDir");
        assert_eq!(slash.7, 6.0, "ai1 = 6");
        assert!(tick.spikes.is_empty(), "a < 30 charge spawns no spikes");
    }

    /// The charged release (ai[1] >= 30) emits the ground-spike wave. A flat
    /// walkable floor under the start point spawns spikes stepping 2 tiles.
    #[test]
    fn w4_d20_the_flint_charged_release_emits_spikes() {
        let mut w = world();
        // A walkable floor row at y=52 across x=100..130 (the spike walk).
        for x in 100..130 {
            put(&mut w, x, 52, 1);
        }
        let mut p = proj(1040);
        p.owner = 0;
        p.damage = 50;
        p.direction = 1;
        p.ai[0] = 0.0;
        p.ai[1] = 30.0; // charged
        // MountedCenter at (1600, 800) → start + 30 = (1630, 800) → tile
        // (101, 50); the surface walk drops to the y=52 floor.
        let mut pl = flint_player(false);
        pl.mounted_center = Vec2::new(1600.0, 800.0);
        let tick = ai_flint(&mut p, &w, pl, Vec2::new(1700.0, 800.0), 0, &mut census());
        assert!(!tick.spikes.is_empty(), "the charged wave spawns spikes");
        // Each spike: ai0 = -(num3*2), ai1 = 1 + num3*0.04 + NextFloat*0.2.
        let first = tick.spikes[0];
        assert_eq!(first.4, 0.0, "the first spike ai0 = 0");
        assert!((first.5 - 1.0).abs() < 1e-6, "ai1 = 1 + 0 + 0 (census float 0)");
    }

    /// The follow-through (`:52261-52279`): counts to useAnimation then kills.
    #[test]
    fn w4_d20_the_flint_follow_through_kills_at_use_animation() {
        let w = world();
        let mut p = proj(1040);
        p.owner = 0;
        p.ai[0] = 1.0;
        p.ai[1] = 19.0; // advances to 20 = useAnimation → kill
        let pl = flint_player(false);
        let tick = ai_flint(&mut p, &w, pl, Vec2::new(1700.0, 800.0), 0, &mut census());
        assert!(!p.active, "the follow-through kill at useAnimation");
        assert!(tick.dummy_item_time.is_none(), "the kill tick writes no dummy time");
    }

    /// The dispatch wires style 198: one pool update advances the charge.
    #[test]
    fn w4_d20_the_dispatch_arms_style_198() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut pl = crate::player::Player::spawn(&w);
        pl.channel = true;
        pl.held_item_kind = 5451;
        // The golf context carries the flint's player snapshot + mouse.
        let ctx = crate::projectile_ai_golf::GolfContext {
            golf_state: tr_core::golf_state::GolfState::default(),
            mouse_world: Vec2::new(1700.0, 800.0),
            mouse_right: false,
            mouse_right_release: false,
            player: flint_player(true),
            player_name: String::new(),
            golfer_score: 0,
        };
        pool.set_golf_context(ctx);
        let s = pool.spawn_with_data(1040, 1600.0, 800.0, 0.0, 0.0, 50, 5.0).unwrap();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            false,
        );
        assert_eq!(pool.slots()[s].ai[1], 1.0, "the charge counter advanced");
    }

    /// W4-d20 T5-canary — the 1043 flint slash is the 190/198 cross-lane
    /// merge canary: d20's 198 arm spawns it (`:52254`, ai0=dir·gravDir,
    /// ai1=6, ai2=1) but its aiStyle-190 AI is d19's. This witness pins the
    /// SPAWN payload end-to-end through the dart-drain (the ai2=1 scale
    /// survives — `:42222` `scale *= ai[2]` would collapse to 0 if the
    /// drain dropped it). A lossy merge that drops the 198 spawner leaves
    /// no 1043; one that drops the 1043 row leaves the spawn inert. Both
    /// must land for the slash to fly.
    #[test]
    fn w4_d20_t5_the_flint_slash_canary_spawns_with_the_scale() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut pl = crate::player::Player::spawn(&w);
        pl.channel = false; // released → the slash spawns
        pl.held_item_kind = 5451;
        let ctx = crate::projectile_ai_golf::GolfContext {
            golf_state: tr_core::golf_state::GolfState::default(),
            mouse_world: Vec2::new(1700.0, 800.0),
            mouse_right: false,
            mouse_right_release: false,
            player: flint_player(false),
            player_name: String::new(),
            golfer_score: 0,
        };
        pool.set_golf_context(ctx);
        let s = pool.spawn_with_data(1040, 1600.0, 800.0, 0.0, 0.0, 50, 5.0).unwrap();
        pool.slots_mut()[s].direction = 1;
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7),
            0,
            false,
        );
        // The 1043 slash spawned (the drain resolved the request).
        let slash = pool
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::DEAD_CELLS_FLINT_SLASH)
            .expect("the 198 arm spawned the 1043 slash");
        assert_eq!(slash.damage, 89, "(int)(50 * 1.78) = 89");
        assert_eq!(slash.vel, Vec2::new(1.0, 0.0), "velocity (direction, 0)");
        assert_eq!(slash.ai[0], 1.0, "ai0 = direction * gravDir");
        assert_eq!(slash.ai[1], 6.0, "ai1 = 6 (the 190 lifetime envelope)");
        assert_eq!(slash.ai[2], 1.0, "ai2 = 1 — the scale survives the drain");
        // The row is the d19 twin (16x16 aiStyle 190 pen 2) — the merge
        // keep-one pin: both branches carry the identical tuple.
        let row = proj_data(crate::projectile::DEAD_CELLS_FLINT_SLASH).unwrap();
        assert_eq!(row.ai_style, 190);
        assert_eq!((row.w, row.h), (16.0, 16.0));
        assert_eq!(row.penetrate, 2);
    }

    // ---- W4-d20 aiStyle 191 (TrueNightsEdge) — WRITTEN NOT RUN ----

    /// The two rows carry the census slice (`Projectile.cs:9315-9329` /
    /// `:9405-9421`): aiStyle 191, melee, alpha 255, timeLeft 90, ignoreWater;
    /// the 973 30x30 pen3 LOCAL10 tileCollide-false, the 985 8x8 pen-1
    /// LOCAL-1 stock tileCollide-true.
    #[test]
    fn w4_d20_the_sword_rows_carry_style_191() {
        let a = proj_data(973).unwrap();
        assert_eq!(a.ai_style, 191);
        assert_eq!((a.w, a.h), (30.0, 30.0));
        assert_eq!(a.penetrate, 3);
        assert!(a.melee);
        assert_eq!(a.alpha, 255);
        assert_eq!(a.time_left, 90);
        assert!(!a.tile_collide);
        assert!(a.uses_local_immunity && a.local_hit_cooldown == 10);
        let b = proj_data(985).unwrap();
        assert_eq!(b.ai_style, 191);
        assert_eq!((b.w, b.h), (8.0, 8.0));
        assert_eq!(b.penetrate, -1);
        assert!(b.tile_collide, "the 985 keeps stock true");
        assert!(b.melee);
    }

    /// The 973 first-tick Item8 cue (`:41654-41657`) and the age counter.
    #[test]
    fn w4_d20_the_true_nights_edge_first_tick_cue() {
        let w = world();
        let mut p = proj(973);
        p.ai[1] = 30.0; // lifetime
        p.ai[2] = 1.0; // scale
        p.vel = Vec2::new(6.0, 0.0);
        let mut rng = census();
        ai_true_nights_edge(&mut p, &w, None, &mut rng);
        assert_eq!(p.wants_sound.map(|r| (r.0, r.1)), Some((2, 8)), "Item8");
        assert_eq!(p.local_ai[0], 1.0, "the age counter");
    }

    /// The num4 kill (`:41668-41673`): at localAI[0] >= ai[1] + 65 the 973
    /// kills and sets localAI[1] = 1.
    #[test]
    fn w4_d20_the_true_nights_edge_kills_at_num4() {
        let w = world();
        let mut p = proj(973);
        p.ai[1] = 30.0; // num3 = 80, num4 = 95
        p.local_ai[0] = 94.0; // advances to 95 = num4 → kill
        ai_true_nights_edge(&mut p, &w, None, &mut census());
        assert!(!p.active, "the num4 kill");
        assert_eq!(p.local_ai[1], 1.0, "the death latch");
    }

    /// The 985 ownerHitCheck window (`:41730`): localAI[0] <= 6 sets the flag,
    /// past it clears. The damage-zero at the 0.65 lerp (`:41731-41734`).
    #[test]
    fn w4_d20_the_terra_blade_owner_hit_check_window() {
        let w = world();
        let mut p = proj(985);
        p.ai[1] = 30.0; // num3 = 30, num4 = 55
        p.ai[2] = 1.0;
        p.damage = 50;
        p.vel = Vec2::new(3.0, 0.0);
        p.local_ai[0] = 0.0; // advances to 1 (<= 6)
        ai_true_nights_edge(&mut p, &w, None, &mut census());
        assert!(p.owner_hit_check, "the early window sets ownerHitCheck");
        // Past the window: localAI[0] = 7 → clears.
        let mut q = proj(985);
        q.ai[1] = 30.0;
        q.ai[2] = 1.0;
        q.damage = 50;
        q.vel = Vec2::new(3.0, 0.0);
        q.local_ai[0] = 6.0; // advances to 7 (> 6)
        ai_true_nights_edge(&mut q, &w, None, &mut census());
        assert!(!q.owner_hit_check, "past the window clears it");
    }

    /// The 985 damage-zero (`:41731-41734`): at localAI[0] >= lerp(num3, num4,
    /// 0.65) the damage zeroes (num3=30, num4=55 → 46.25).
    #[test]
    fn w4_d20_the_terra_blade_damage_zeroes_late() {
        let w = world();
        let mut p = proj(985);
        p.ai[1] = 30.0;
        p.ai[2] = 1.0;
        p.damage = 50;
        p.vel = Vec2::new(3.0, 0.0);
        p.local_ai[0] = 46.0; // advances to 47 >= 46.25 → damage 0
        ai_true_nights_edge(&mut p, &w, None, &mut census());
        assert_eq!(p.damage, 0, "the late damage-zero");
    }

    /// The 985 retract move (`:18104-18110`): a blocked move steps once,
    /// restores lastVelocity * 0.01 and sets localAI[1] = 1.
    #[test]
    fn w4_d20_the_terra_blade_retract_move() {
        let mut w = world();
        put(&mut w, 102, 50, 1); // wall at x 1632..1648
        let mut p = proj(985);
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(40.0, 0.0); // right edge 1608 + 40 = 1648 overlaps
        terra_blade_move(&mut p, &w);
        assert_eq!(p.local_ai[1], 1.0, "the retract latch");
        assert!((p.vel.x - 40.0 * 0.01).abs() < 1e-6, "lastVelocity * 0.01");
    }

    /// The dispatch wires style 191: one pool update advances the age for
    /// both kinds.
    #[test]
    fn w4_d20_the_dispatch_arms_style_191() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let a = pool.spawn_with_data_ai(973, 1600.0, 400.0, 6.0, 0.0, 40, 2.0, 1.0, 30.0).unwrap();
        let b = pool.spawn_with_data_ai(985, 1600.0, 500.0, 6.0, 0.0, 40, 2.0, 1.0, 30.0).unwrap();
        // ai[2] is the scale envelope (the spawn seam carries ai0/ai1 only).
        pool.slots_mut()[a].ai[2] = 1.0;
        pool.slots_mut()[b].ai[2] = 1.0;
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
        assert!(pool.slots()[a].local_ai[0] >= 1.0, "the 973 age advanced");
        assert!(pool.slots()[b].local_ai[0] >= 1.0, "the 985 age advanced");
    }
}
