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
    Projectile, can_hit, can_hit_line, chaseable, chaseable_held, get_lerp_value, rotate,
    safe_normalize,
};

/// `AI()`'s aiStyle-31 arm (`Projectile.cs:26786-26886`) — the Clentaminator
/// spray. W4-d18 rowed the seven solution kinds (146-149/1015-1017 — the
/// rows were unclaimed on every pushed tip); the per-kind ladder below is
/// the whole arm. The conversion cells for types 2/5/6/7 are W4-f25's
/// wiring (its tip b004f4f1b dispatches `convert_sized_hallow` /
/// `convert_sized_spray` here); until that merges, only the type-0 cleanse
/// converts on this branch — the counter/cap/spin run for every kind.
/// spray. All eight spray kinds carry `proj_data` rows now (W4-d1 retire
/// rowed 146-149/1015-1017 beside the 145 — the d18/f25 DUP keep-one, the
/// d18 shape); the per-kind ladder below is the whole arm. The conversion
/// cells: type 0 the cleanse, types 1/4 (the 147 Corrupt / 149 Red
/// solutions) ride d1's own `spread::convert_sized` (the powder port's
/// cells — ACTIVATED here at the spray's size 2/3); types 2/5/6/7 are
/// W4-f25's C3/C4 wiring (`convert_sized_hallow`/`convert_sized_spray`,
/// its tip d587b128d), type 3 is W4-g42's mushroom cell
/// (`convert_sized_mushroom`, its merger contract) — those arms land at
/// the f25/g42 merges (research/352's compose table — registry-150 §9
/// chain 246→352).
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
    // type-0 cleanse cell is ported here (`spread::convert_cleanse`); the
    // 146-149/1015-1017 rows exist (W4-d18), their conversion cells are
    // f25's wiring (types 2/5/6/7) + d1/g42's (types 1/3/4) — a no-op on
    // this branch until those merge.
    if p.owner == local_owner && conversion == 0 {
    // (the `(int)(X/16)` truncation), size 2 (3 for the solution). The
    // ported cells dispatch here (W4-d1 retire compose): type 0 the
    // cleanse (`spread::convert_cleanse`), types 1/4 the corrupt/crimson
    // ladders (`spread::convert_sized` — d1's powder cells, the SAME
    // `Convert(i, j, type, size)` entry the powders call at size 1).
    // Types 2/5/6/7 (f25's C3/C4) and type 3 (g42's mushroom) stay no-ops
    // on this branch until those merges — the counter/cap/spin run for
    // every kind regardless.
    if p.owner == local_owner {
        let tx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
        let ty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
        let size = if solution { 3 } else { 2 };
        match conversion {
            0 => tr_core::spread::convert_cleanse(world, tx, ty, size),
            1 | 4 => tr_core::spread::convert_sized(world, tx, ty, conversion, size),
            _ => {}
        }
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
/// slot field). The 1106 row landed with the d18 census residual
/// (18916087); the 85/1106 `localAI[0] >= 54` damage gate
/// (`:12460-12467`) rides the pool's damage walks (`flames_faded`).
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

    /// W4-d1 retire (the spray compose) — the 147 Corrupt / 149 Red
    /// solutions ACTIVATED: their conversion types 1/4 (`:26797-26800`/
    /// `:26809-26812`) dispatch through d1's own `spread::convert_sized`
    /// cells (the powder port's corrupt/crimson ladders) at the spray's
    /// size 2/3 — the grass 2 → 23/199, the stone 1 → 25/203, the sand 53
    /// → 112/234. The 146/148/1015-1017 kinds stay no-ops on this branch
    /// (types 2/5/6/7 = f25's C3/C4, type 3 = g42's mushroom — the merge
    /// compose, research/352 — registry-150 §9 chain 246→352) but their
    /// counter/cap/spin run. WRITTEN NOT
    /// RUN (code-only retire round).
    #[test]
    fn w4_d1_retire_the_corrupt_and_red_sprays_convert_their_cells() {
        // 147 — type 1 corruption (`:55299-55334`).
        let mut w = world();
        put(&mut w, 100, 50, 2);
        put(&mut w, 101, 50, 1);
        put(&mut w, 100, 51, 53);
        let mut p = proj(147);
        p.time_left = 3600;
        ai_spray(&mut p, &mut w, 0);
        assert_eq!(w.tile(100, 50).unwrap().kind, 23, "the grass corrupts");
        assert_eq!(w.tile(101, 50).unwrap().kind, 25, "the stone → ebonstone");
        assert_eq!(w.tile(100, 51).unwrap().kind, 112, "the sand → ebonsand");
        assert_eq!(p.time_left, 133, "the cap runs for every kind");
        assert_eq!(p.ai[0], 1.0, "the counter runs for every kind");
        // 149 — type 4 crimson (`:55138-55173`).
        let mut w = world();
        put(&mut w, 100, 50, 2);
        put(&mut w, 101, 50, 1);
        put(&mut w, 100, 51, 53);
        let mut p = proj(149);
        ai_spray(&mut p, &mut w, 0);
        assert_eq!(w.tile(100, 50).unwrap().kind, 199, "the grass crimson");
        assert_eq!(w.tile(101, 50).unwrap().kind, 203, "the stone → crimstone");
        assert_eq!(w.tile(100, 51).unwrap().kind, 234, "the sand → crimsand");
        // The solution flag widens the disc to size 3 for the evil types
        // too (`:26828-26832` — the size is conversion-independent).
        let mut w = world();
        put(&mut w, 103, 52, 2); // Manhattan 5, outside size 2
        let mut p = proj(147);
        p.ai[1] = 1.0;
        ai_spray(&mut p, &mut w, 0);
        assert_eq!(w.tile(103, 52).unwrap().kind, 23, "the size-3 evil disc");
        // The non-owner refuses (`:26826`).
        let mut w = world();
        put(&mut w, 100, 50, 2);
        let mut p = proj(149);
        p.owner = 1;
        ai_spray(&mut p, &mut w, 0);
        assert_eq!(w.tile(100, 50).unwrap().kind, 2, "the owner gate");
        // The f25/g42-owned kinds stay no-ops here (their cells land at
        // those merges) — the grass keeps its kind, the machine runs.
        for kind in [146u16, 148, 1015, 1016, 1017] {
            let mut w = world();
            put(&mut w, 100, 50, 2);
            let mut p = proj(kind);
            ai_spray(&mut p, &mut w, 0);
            assert_eq!(w.tile(100, 50).unwrap().kind, 2, "kind {kind} waits for its cell");
            assert_eq!(p.ai[0], 1.0, "kind {kind} counter");
        }
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

/// `Projectile.Opacity`'s setter (`Projectile.cs:546-549`):
/// `alpha = (int)Clamp((1 - value) * 255, 0, 255)` — the empress module's
/// helper, re-stated here (that one is private to its file).
fn set_opacity(p: &mut Projectile, opacity: f32) {
    p.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as i32;
}

/// `AI_172_GetPelletStormsCount` (`Projectile.cs:45606-45609`) — a constant 6.
const PELLET_STORMS_COUNT: i32 = 6;

/// `HallowBossPelletStormInfo` (`Projectile.cs:197-237`) — one storm's
/// fan parameters. The port keeps the four fields the AI reads; the
/// hitbox helper (`GetBulletHitbox`, `:233-236`) is the damage-side slice
/// that stays with the wiring (module precedent: the empress doc).
pub(crate) struct PelletStormInfo {
    pub start_angle: f32,
    pub angle_per_bullet: f32,
    pub bullets_in_storm: i32,
    pub progress_start: f32,
    pub progress_bonus_by_index: f32,
    pub storm_total_range: f32,
    pub bullet_size: Vec2,
}

impl PelletStormInfo {
    /// `GetBulletProgress` (`Projectile.cs:213-216`).
    fn bullet_progress(&self, bullet_index: i32) -> f32 {
        self.progress_start + self.progress_bonus_by_index * bullet_index as f32
    }

    /// `IsValid` (`Projectile.cs:218-226`) — progress in [0, 1].
    fn is_valid(&self, bullet_index: i32) -> bool {
        let progress = self.bullet_progress(bullet_index);
        progress >= 0.0 && progress <= 1.0
    }

    /// `GetBulletPosition` (`Projectile.cs:228-231`): the centre plus the
    /// UnitX fan ray scaled by the range and the progress. The C# computes
    /// the position but the AI DISCARDS it (`:45600` — no assignment); the
    /// hitbox consumer reads it later, so the port keeps the maths.
    fn bullet_position(&self, bullet_index: i32, center: Vec2) -> Vec2 {
        let angle = self.start_angle + self.angle_per_bullet * bullet_index as f32;
        // `RotatedBy` (`Utils.cs:1675-1684`): the double cos/sin cast to
        // float, then the float multiply-add (the empress precedent).
        let c = (angle as f64).cos() as f32;
        let s = (angle as f64).sin() as f32;
        center + Vec2::new(c, s) * self.storm_total_range * self.bullet_progress(bullet_index)
    }
}

/// `AI_172_GetPelletStormInfo` (`Projectile.cs:45611-45625`) — the storm
/// fan for `storm_index`: the start angle
/// `i·π/3 − π/2 + i·π/5`, the 2π/3 per-bullet spread, 3 bullets, the
/// progress ramp over `[10i, 90 + 10i]` of `localAI[0]`, the 500-px range
/// and the 16×16 bullet box.
fn pellet_storm_info(storm_index: i32, age: f32) -> PelletStormInfo {
    let num = 0.0 + storm_index as f32 * 10.0;
    let to = 90.0 + storm_index as f32 * 10.0;
    PelletStormInfo {
        start_angle: storm_index as f32 * (std::f32::consts::PI / 3.0)
            - std::f32::consts::PI / 2.0
            + storm_index as f32 * (std::f32::consts::PI / 5.0),
        angle_per_bullet: std::f32::consts::PI * 2.0 / 3.0,
        bullets_in_storm: 3,
        progress_start: get_lerp_value(num, to, age),
        progress_bonus_by_index: 0.0,
        storm_total_range: 500.0,
        bullet_size: Vec2::new(16.0, 16.0),
    }
}

/// `AI_172_HallowBossRainbowPelletStorm` (`Projectile.cs:45569-45604`) —
/// the Split-Shot Core (871, the style's only kind). A 91-tick grow-in
/// (`localAI[1]` 0→91, the scale 5→1 and the squared opacity ramp), then
/// the storm phase: `localAI[0]` counts to `150 + 10·6 = 210` and kills,
/// the velocity zeroes and the rotation pins 0 every tick, and the six
/// three-bullet fans' positions are computed (the C# discards them —
/// the hitbox consumer is the damage-side slice). ZERO `Main.rand` draws
/// by construction (the grow-in and the fan maths roll nothing).
pub(crate) fn ai_pellet_storm(p: &mut Projectile) {
    // `:45571-45580` — the grow-in: `localAI[1] <= 90` keeps ticking.
    if p.local_ai[1] <= 90.0 {
        p.local_ai[1] += 1.0;
        p.scale = 0.5;
        set_opacity(p, 0.5);
        let lerp_value = get_lerp_value(0.0, 90.0, p.local_ai[1]);
        // `:45577` — MathHelper.Lerp(5, 1, t) = 5 + (1 − 5)·t.
        p.scale = 5.0 + (1.0 - 5.0) * lerp_value;
        // `:45578` — Opacity = 1 − (1 − t²) = t².
        set_opacity(p, 1.0 - (1.0 - lerp_value * lerp_value));
        return;
    }
    // `:45581-45582` — the storm phase pins scale 1, opacity 1.
    p.scale = 1.0;
    set_opacity(p, 1.0);
    // `:45583-45589` — the life cap: 150 + 10·count, then Kill + return.
    let num = 150.0 + 10.0 * PELLET_STORMS_COUNT as f32;
    p.local_ai[0] += 1.0;
    if p.local_ai[0] >= num {
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:45590-45591` — the still core.
    p.vel = Vec2::ZERO;
    p.rot = 0.0;
    // `:45592-45603` — the fan walk. The C# computes each valid bullet's
    // position and DISCARDS it (`GetBulletPosition`'s return is unused);
    // the port keeps the walk + the maths (the hitbox slice reads it).
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    for i in 0..PELLET_STORMS_COUNT {
        let storm = pellet_storm_info(i, p.local_ai[0]);
        for j in 0..storm.bullets_in_storm {
            if storm.is_valid(j) {
                let _ = storm.bullet_position(j, center);
            }
        }
    }
}

#[cfg(test)]
mod w4_d18_172_tests {
    //! W4-d18 — aiStyle 172 (`AI_172_HallowBossRainbowPelletStorm`,
    //! `Projectile.cs:45569-45604`, the dispatch `:36651-36654`). WRITTEN
    //! NOT RUN (code-only mode). The row, the 91-tick grow-in ladder, the
    //! storm-phase pins, the 210-tick kill and the fan maths.
    use super::*;
    use crate::projectile::{
        FlailInputs, HALLOW_BOSS_SPLIT_SHOT_CORE, ProjectilePool, YoyoInputs, proj_data,
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

    fn core() -> Projectile {
        let d = proj_data(HALLOW_BOSS_SPLIT_SHOT_CORE).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = HALLOW_BOSS_SPLIT_SHOT_CORE;
        p.owner = 255;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.friendly = false;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The row (`Projectile.cs:8380-8392`): 30x30, aiStyle 172, alpha 255,
    /// penetrate -1, timeLeft 240, tileCollide false, ignoreWater.
    #[test]
    fn w4_d18_172_the_row_matches_set_defaults() {
        let d = proj_data(HALLOW_BOSS_SPLIT_SHOT_CORE).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (30.0, 30.0, 172));
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.time_left, 240);
        assert_eq!(d.alpha, 255);
        assert!(!d.tile_collide && d.ignore_water);
        assert_eq!(d.extra_updates, 0);
        // The empress gate still does NOT claim 871 (its own arm serves it).
        assert!(!crate::projectile_ai_empress::handles(HALLOW_BOSS_SPLIT_SHOT_CORE));
    }

    /// The grow-in (`:45571-45580`): tick 1 → localAI[1] = 1, scale
    /// Lerp(5,1,1/90), opacity (1/90)²; the alpha rides the Opacity
    /// setter. Zero draws.
    #[test]
    fn w4_d18_172_the_grow_in_ladder() {
        let mut p = core();
        let mut rng = census();
        ai_pellet_storm(&mut p);
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.local_ai[0], 0.0, "the storm counter stays down");
        let t = 1.0f32 / 90.0;
        assert_eq!(p.scale, 5.0 + (1.0 - 5.0) * t);
        let opacity = t * t;
        assert_eq!(p.alpha, ((1.0 - opacity) * 255.0) as i32);
        assert!(rng.ranges.is_empty() && rng.floats == 0);
        // Tick 91 is the LAST grow-in tick (localAI[1] 90 ≤ 90 holds).
        let mut p2 = core();
        p2.local_ai[1] = 90.0;
        ai_pellet_storm(&mut p2);
        assert_eq!(p2.local_ai[1], 91.0);
        assert_eq!(p2.scale, 1.0);
        assert_eq!(p2.alpha, 0, "opacity 1 → alpha 0");
    }

    /// The storm phase (`:45581-45591`): scale/opacity pin 1, the counter
    /// climbs, velocity zeroes, rotation pins 0 — even with a spawn drift.
    #[test]
    fn w4_d18_172_the_storm_phase_pins() {
        let mut p = core();
        p.local_ai[1] = 91.0;
        p.local_ai[0] = 100.0;
        p.vel = Vec2::new(3.0, -2.0);
        p.rot = 1.0;
        let mut rng = census();
        ai_pellet_storm(&mut p);
        assert_eq!(p.local_ai[0], 101.0);
        assert_eq!(p.scale, 1.0);
        assert_eq!(p.alpha, 0);
        assert_eq!(p.vel, Vec2::ZERO);
        assert_eq!(p.rot, 0.0);
        assert!(rng.ranges.is_empty() && rng.floats == 0);
    }

    /// The kill (`:45585-45589`): localAI[0] reaches 210 → dead, the
    /// velocity/rotation tail skips. 209 is the last live tick.
    #[test]
    fn w4_d18_172_the_210_tick_kill() {
        let mut p = core();
        p.local_ai[1] = 91.0;
        p.local_ai[0] = 209.0;
        p.vel = Vec2::new(3.0, 0.0);
        ai_pellet_storm(&mut p);
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert_eq!(p.local_ai[0], 210.0);
        assert_eq!(p.vel, Vec2::new(3.0, 0.0), "the kill returns before the zero");
        let mut p2 = core();
        p2.local_ai[1] = 91.0;
        p2.local_ai[0] = 208.0;
        ai_pellet_storm(&mut p2);
        assert!(p2.active);
        assert_eq!(p2.local_ai[0], 209.0);
    }

    /// The fan maths (`:45611-45625` + `:213-231`): storm 0's start angle
    /// −π/2, the 2π/3 spread, the progress ramp and the 500-px ray.
    #[test]
    fn w4_d18_172_the_fan_geometry() {
        use std::f32::consts::PI;
        let s0 = pellet_storm_info(0, 45.0);
        assert_eq!(s0.start_angle, -PI / 2.0);
        assert_eq!(s0.angle_per_bullet, PI * 2.0 / 3.0);
        assert_eq!(s0.bullets_in_storm, 3);
        assert_eq!(s0.progress_start, 0.5, "GetLerpValue(0,90,45)");
        assert_eq!(s0.storm_total_range, 500.0);
        assert_eq!(s0.bullet_size, Vec2::new(16.0, 16.0));
        assert!(s0.is_valid(0) && s0.is_valid(2));
        // Bullet 0 sits straight up at half range: (0, −250).
        let pos = s0.bullet_position(0, Vec2::new(100.0, 100.0));
        assert!((pos.x - 100.0).abs() < 1e-3);
        assert!((pos.y - (100.0 - 250.0)).abs() < 1e-3);
        // Storm 5's window is [50, 140]: at age 45 nothing is valid yet.
        let s5 = pellet_storm_info(5, 45.0);
        assert_eq!(s5.progress_start, 0.0);
        assert!(s5.is_valid(0), "progress 0 is the inclusive floor");
        // Past the window every bullet clamps to 1 (still valid).
        let s0_late = pellet_storm_info(0, 200.0);
        assert_eq!(s0_late.progress_start, 1.0);
        assert!(s0_late.is_valid(2));
    }

    /// The dispatch wiring: one pool update runs the grow-in tick.
    #[test]
    fn w4_d18_172_the_dispatch_arms_the_storm() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(HALLOW_BOSS_SPLIT_SHOT_CORE, 1600.0, 800.0, 0.0, 0.0, 0, 0.0)
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
            255,
            true,
        );
        let p = &pool.slots()[slot];
        assert!(p.active);
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.local_ai[0], 0.0);
        assert!(rng.ranges.is_empty() && rng.floats == 0);
    }
}

/// `Entity.AngleTo` (`Entity.cs:185-188`): `(float)Math.Atan2(dy, dx)` —
/// the double atan2 cast to float (the `npc_ai_style75` precision order).
fn angle_to(from: Vec2, to: Vec2) -> f32 {
    ((to.y - from.y) as f64).atan2((to.x - from.x) as f64) as f32
}

/// `Utils.ToRotation` (`Utils.cs:1655-1658`): `(float)Math.Atan2(Y, X)`.
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): `(cos f, sin f)` in
/// double, cast to f32 (the `npc_ai_style75` helper's shape).
fn to_rotation_vector2(f: f32) -> Vec2 {
    Vec2::new((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// `AI_174_MultisegmentPet` (`Projectile.cs:45345-45427`) — the three
/// worm-segment pets (883 Eater of Worlds / 887 Destroyer / 893 Lunatic
/// Cultist). The inactive-owner kill, the per-type keep-alive (death
/// clears the flag FIRST, a standing flag pins `timeLeft = 2` — the
/// `projectile_ai026_b::keep_alive` shape), the >2000-px teleport, the
/// zero-velocity kick with the `oldPos` refill, the 5°/tick steering past
/// 120 px, the speed clamp to `Min(12, Max(4, |player.velocity|))`, the
/// −0.1 Y drift under 1, the flight rotation + π/2, the direction flip
/// and the 160-px world-edge clamp. ZERO `Main.rand` draws by construction.
///
/// `player` is the owner channel (`None` = the C# `!player.active`); the
/// `pet_flags` array is indexed by the pet projectile id (the port's
/// per-pet bool stand-in, `player.rs:2034`).
pub(crate) fn ai_multisegment_pet(
    p: &mut Projectile,
    player: Option<&mut crate::player::Player>,
    world: &World,
) {
    // `:45347-45352` — the inactive owner kills outright (no `timeLeft`
    // write — the C# sets `active = false` only, not `Kill()`).
    let Some(player) = player else {
        p.active = false;
        return;
    };
    // `:45353-45385` — the three keep-alive blocks: death clears the
    // flag, then the (possibly cleared) flag pins the lifetime.
    let idx = usize::from(p.kind);
    if player.dead {
        player.pet_flags[idx] = false;
    }
    if player.pet_flags[idx] {
        p.time_left = 2;
    }
    // `:45386-45392` — the teleport: `Distance(center) > 2000` snaps the
    // centre onto the owner's and zeroes the velocity.
    let (px, py) = player.center();
    let center = Vec2::new(px, py);
    let mut pcenter = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    if pcenter.distance(center) > 2000.0 {
        pcenter = center;
        p.pos = Vec2::new(pcenter.x - p.w * 0.5, pcenter.y - p.h * 0.5);
        p.vel = Vec2::ZERO;
        p.net_update = true;
    }
    // `:45393-45395` — the distance and the speed band. The discarded
    // `base.velocity.Length()` call (`:45395`) has no side effect.
    let num = (center - pcenter).length();
    let player_speed = Vec2::new(player.vx, player.vy).length();
    let num2 = player_speed.max(4.0).min(12.0);
    // `:45396-45405` — the zero-velocity kick: `velocity.X = 2·direction`
    // and the `oldPos` ring refills walking BACK from the position.
    if p.vel == Vec2::ZERO {
        p.vel.x = 2.0 * player.facing as f32;
        let mut vector = p.pos;
        for slot in p.old_pos.iter_mut() {
            vector -= p.vel;
            *slot = vector;
        }
    }
    // `:45406-45409` — past 120 px the heading turns toward the owner at
    // `MathHelper.ToRadians(5)` per tick (`5·π/180`, the XNA constant).
    if !(num < 120.0) {
        let target = angle_to(pcenter, center);
        let cur = to_rotation(p.vel);
        let turned = crate::projectile::angle_towards(cur, target, std::f32::consts::PI / 180.0 * 5.0);
        p.vel = to_rotation_vector2(turned) * num2;
    }
    // `:45410-45413` — the speed cap (`SafeNormalize(Zero)`).
    if p.vel.length() > num2 {
        p.vel = safe_normalize(p.vel) * num2;
    }
    // `:45414-45417` — the near-horizontal drift.
    if p.vel.y.abs() < 1.0 {
        p.vel.y -= 0.1;
    }
    // `:45418` — the flight rotation + π/2.
    p.rot = to_rotation(p.vel) + std::f32::consts::FRAC_PI_2;
    // `:45419-45424` — the direction/spriteDirection flip (the netUpdate
    // on a change).
    let num3 = p.direction;
    let dir = if p.vel.x > 0.0 { 1 } else { -1 };
    p.direction = dir;
    p.sprite_direction = dir;
    if num3 != p.direction {
        p.net_update = true;
    }
    // `:45425-45426` — the world-edge clamp (160 px inset).
    p.pos.x = p.pos.x.clamp(160.0, world.width as f32 * 16.0 - 160.0);
    p.pos.y = p.pos.y.clamp(160.0, world.height as f32 * 16.0 - 160.0);
}

/// `AI_175_TitaniumStormShards` (`Projectile.cs:45319-45343`) — the
/// Titanium Storm Shard (908). The buff-gated kill, the first-tick frame
/// + rotation roll (`Next(12)` then `NextFloat`), the π/200 spin, the
/// group-index orbit angle `(index/total + miscCounterNormalized·6)·2π`,
/// the `24 + total·6` radius, the owner's per-tick move added to the
/// centre, the 0.3 lerp toward the squashed orbit point and the
/// `localAI[0] = sin(angle)` draw-layer key.
///
/// RNG: exactly two draws on the FIRST tick (`frameCounter == 0`), zero
/// after. The `drawLayer` write (`:45342`, 3 below / 4 above the owner)
/// is draw-only — no port field, recorded here.
///
/// `has_buff` is the owner's `hasTitaniumStormBuff` (`Player.cs:1180`,
/// set by buff 306 `:10086-10089`); the dispatch reads the buff slot.
/// `group_index`/`group_total` are the pool-precomputed
/// `AI_GetMyGroupIndexAndFillBlackList` pair (`:45334`).
pub(crate) fn ai_titanium_storm_shard(
    p: &mut Projectile,
    player: Option<&crate::player::Player>,
    has_buff: bool,
    group_index: i32,
    group_total: i32,
    rng: &mut dyn DropRng,
) {
    // `:45321-45326` — the owner/buff gate: inactive, dead or buff-less
    // kills outright (`Kill()`'s active+timeLeft zero).
    let Some(player) = player else {
        p.active = false;
        p.time_left = 0;
        return;
    };
    if player.dead || !has_buff {
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:45327-45332` — the first-tick roll: frameCounter latches 1, the
    // frame picks one of the 12 rows and the rotation a full-circle angle.
    if p.frame_counter == 0 {
        p.frame_counter = 1;
        p.frame = rng.next_range(0, 12);
        p.rot = rng.next_float() * (std::f32::consts::PI * 2.0);
    }
    // `:45333` — the π/200 spin.
    p.rot += std::f32::consts::PI / 200.0;
    // `:45335-45336` — the orbit angle and radius. `miscCounterNormalized`
    // is `miscCounter / 300f` (`Player.cs:3389`).
    let misc_norm = player.misc_counter as f32 / 300.0;
    let f = (group_index as f32 / group_total as f32 + misc_norm * 6.0)
        * (std::f32::consts::PI * 2.0);
    let num = 24.0 + group_total as f32 * 6.0;
    // `:45337-45338` — the owner's per-tick move rides the shard centre.
    let vector = Vec2::new(player.x - player.old_x, player.y - player.old_y);
    let mut center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    center += vector;
    // `:45339-45341` — the orbit point: the unit ray squashed to 5% Y,
    // scaled by the radius, offset from the owner's centre; the shard
    // lerps 0.3 toward it (`Lerp(value1: Center, value2: target, 0.3)`).
    let (px, py) = player.center();
    let player_center = Vec2::new(px, py);
    let ray = to_rotation_vector2(f);
    p.local_ai[0] = ray.y;
    let target = player_center + ray * Vec2::new(1.0, 0.05) * num;
    center = center.lerp(target, 0.3);
    p.pos = Vec2::new(center.x - p.w * 0.5, center.y - p.h * 0.5);
    // `:45342` — `drawLayer = localAI[0] <= 0 ? 3 : 4` — draw-only, the
    // port carries no drawLayer field (the localAI[0] key survives).
}

/// `Projectile.FindTargetWithinRange` (`Projectile.cs:45097-45115`) —
/// the nearest chaseable NPC within `max_range` whose local-immunity
/// entry is 0. NOT the line-of-sight variant (`:45632-45660`): the
/// default `checkCanHit = false` skips the `Collision.CanHitLine` test.
/// When `check_can_hit` is true (the 969 twister, `:45178-45181`) the
/// `CanHitLine` box test joins the eligibility gate, so a `world` rides
/// along. The strict `!(num <= num2)` keeps the FIRST nearest on ties
/// (the scan order is the NPC index). Returns −1 for no target (the C#
/// null). Shared by the 176 bolt and the 177 slicers.
pub(crate) fn find_target_within_range(
    pos: Vec2,
    w: f32,
    h: f32,
    npcs: &[Npc],
    immune: &[i32],
    max_range: f32,
    world: Option<&World>,
    check_can_hit: bool,
) -> i32 {
    let center = Vec2::new(pos.x + w * 0.5, pos.y + h * 0.5);
    let mut num = max_range;
    let mut result: i32 = -1;
    for (i, n) in npcs.iter().enumerate() {
        if !chaseable(n) || immune.get(i).is_some_and(|v| *v != 0) {
            continue;
        }
        // `:45104` — the optional `Collision.CanHitLine` eligibility term.
        if check_can_hit {
            if let Some(wr) = world {
                let (nw, nh) = n.size();
                if !can_hit_line(wr, pos.x, pos.y, w, h, n.x, n.y, nw, nh) {
                    continue;
                }
            }
        }
        let num2 = center.distance(crate::projectile::npc_center(n));
        if !(num <= num2) {
            num = num2;
            result = i as i32;
        }
    }
    result
}

/// `AI_176_EdgyLightning` (`Projectile.cs:45058-45095`) — the Dark
/// Harvest's scythe bolt (916). The `ai[0] − 1` target latch with its
/// chaseable/immunity re-validation, the 400-px acquire, the 10-px
/// `DirectionTo` flight with the velocity rotation, the 3-tick frame
/// machine (`Next(projFrames)` — the ONLY draw) and the no-target kill.
///
/// `immune` is the slot's local-NPC immunity row (the pool passes its
/// per-slot slice); the kill rides `Kill()`'s active+timeLeft zero.
pub(crate) fn ai_edgy_lightning(
    p: &mut Projectile,
    npcs: &[Npc],
    immune: &[i32],
    rng: &mut dyn DropRng,
) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:45060-45065` — the held target re-validates: a gone/un-chaseable
    // or immune NPC drops the latch.
    let mut num = p.ai[0] as i32 - 1;
    if num != -1 {
        let gone = npcs
            .get(num as usize)
            .map_or(true, |n| !chaseable(n));
        if gone || immune.get(num as usize).is_some_and(|v| *v != 0) {
            p.ai[0] = 0.0;
            num = -1;
        }
    }
    // `:45066-45075` — the acquire: the nearest within 400 px latches
    // `ai[0] = whoAmI + 1` and flags the net update.
    if num == -1 {
        let target = find_target_within_range(p.pos, p.w, p.h, npcs, immune, 400.0, None, false);
        if target != -1 {
            num = target;
            p.ai[0] = num as f32 + 1.0;
            p.net_update = true;
        }
    }
    if num != -1 {
        // `:45078-45084` — the 10-px chase: `DirectionTo` is
        // `Normalize(target − Center)`; a NaN direction (the zero delta)
        // keeps the old velocity and rotation.
        let n = &npcs[num as usize];
        let delta = crate::projectile::npc_center(n) - center;
        if !delta.is_nan() && delta != Vec2::ZERO {
            let vector = delta.normalize();
            p.vel = vector * 10.0;
            p.rot = to_rotation(p.vel);
        }
        // `:45085-45089` — the 3-tick frame machine: the ONLY rand draw.
        p.frame_counter += 1;
        if p.frame_counter >= 3 {
            p.frame_counter = 0;
            p.frame = rng.next_range(0, crate::projectile::proj_frames(p.kind) as i32);
        }
    } else {
        // `:45091-45094` — no target: `Kill()`.
        p.active = false;
        p.time_left = 0;
    }
}

/// `AI_177_IceWhipSlicer` (`Projectile.cs:45117-45265`) — the Cool Whip's
/// snowflake slicer (917) and the Weather Pain's twister (969). The
/// 917 buff-gated kill/keep-alive, the 969 twister lifetime (the 555-tick
/// sibling-sync floor, the >560 alpha fade, the 570 kill), the owner-gated
/// target latch/acquire (the 969 adds the `CanHitLine` test), the
/// per-kind steering constants (917: 18 px / 0.1 lerp; 969: 8 px / 0.075
/// lerp / 25-px dead zone), the no-target arms (969 self-direction lerp,
/// 917 ×0.92 decay) and the per-kind rotation/frame/dust tails.
///
/// Returns `true` when the 969 sibling-sync scan must run (the dispatch
/// pre-scans the pool — the slot arm cannot see the other slots, the
/// `minion_separation` borrow pattern). The sync pins every older
/// same-owner same-kind twister with `ai[1] < 555` up to 555.
///
/// RNG: the 917 tail rolls `Next(3)` every tick; the taken branch's
/// `NextVector2Circular(1,1)` pair stays (the dust body is the skip).
/// The 969 tail rolls nothing explicit — its `timeLeft % 3` dust gate's
/// `NextVector2Circular` pairs are `Dust.NewDust*` call-site velocity
/// writes, kept as draws (the flamethrower precedent): one pair + two
/// pairs per gated tick. The sound slot (`localAI[1]`, `:45132-45146`)
/// has no tracked-sound layer — the slot-id bookkeeping is skipped and
/// `localAI[1]` stays 0 (recorded here).
pub(crate) fn ai_ice_whip_slicer(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    immune: &[i32],
    mut player: Option<&mut crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> bool {
    let mut sync_siblings = false;
    // `:45119-45129` — the 917 buff gate: an absent/dead owner clears
    // coolWhipBuff and kills; a standing buff pins timeLeft 2.
    if p.kind == crate::projectile::COOL_WHIP_PROJ {
        match player.as_deref_mut() {
            None => {
                p.active = false;
                p.time_left = 0;
                return false;
            }
            Some(pl) => {
                if pl.dead {
                    pl.cool_whip_buff = false;
                    p.active = false;
                    p.time_left = 0;
                    return false;
                }
                if pl.cool_whip_buff {
                    p.time_left = 2;
                }
            }
        }
    }
    // `:45130-45165` — the 969 twister lifetime. The tracked-sound block
    // (`:45132-45146`) has no sound layer — skipped (module doc).
    if p.kind == crate::projectile::WEATHER_PAIN_SHOT {
        p.ai[1] += 1.0;
        if p.ai[1] > 560.0 {
            // `:45150` — `(int)MathHelper.Lerp(0, 250, (ai[1]−560)/10)`.
            p.alpha = (0.0f32 + (250.0f32 - 0.0f32) * ((p.ai[1] - 560.0) / 10.0)) as i32;
        }
        if p.ai[1] >= 570.0 {
            p.active = false;
            p.time_left = 0;
        }
        // `:45156-45164` — the sibling-sync scan runs in the dispatch.
        sync_siblings = true;
    }
    // `:45166-45190` — the owner-gated target latch/acquire.
    let mut num2 = p.ai[0] as i32 - 1;
    if p.owner == local_owner {
        if num2 != -1 {
            let gone = npcs.get(num2 as usize).map_or(true, |n| !chaseable(n));
            // `:45169` — the 177 re-validation uses `> 0` (NOT `!= 0`):
            // the −1 sentinel keeps the lock.
            let immune_hit = immune.get(num2 as usize).is_some_and(|v| *v > 0);
            if gone || immune_hit {
                p.ai[0] = 0.0;
                num2 = -1;
                p.net_update = true;
            }
        }
        if num2 == -1 {
            let check_can_hit = p.kind == crate::projectile::WEATHER_PAIN_SHOT;
            let target = find_target_within_range(
                p.pos, p.w, p.h, npcs, immune, 400.0, Some(world), check_can_hit,
            );
            if target != -1 {
                num2 = target;
                p.ai[0] = num2 as f32 + 1.0;
                p.net_update = true;
            }
        }
    }
    // `:45191-45199` — the per-kind steering constants.
    let mut num3 = 18.0f32;
    let mut amount = 0.1f32;
    let mut num4 = 0.0f32;
    if p.kind == crate::projectile::WEATHER_PAIN_SHOT {
        num3 = 8.0;
        amount = 0.075;
        num4 = 25.0;
    }
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    if num2 != -1 {
        // `:45200-45213` — the chase: the speed clamps to the distance,
        // the lerp runs only outside the dead zone with a valid direction.
        let n = &npcs[num2 as usize];
        let nc = crate::projectile::npc_center(n);
        let num5 = center.distance(nc);
        if num3 > num5 {
            num3 = num5;
        }
        let delta = nc - center;
        if !delta.is_nan() && delta != Vec2::ZERO && num5 >= num4 {
            let vector = delta.normalize();
            p.vel = p.vel.lerp(vector * num3, amount);
        }
    } else if p.kind == crate::projectile::WEATHER_PAIN_SHOT {
        // `:45214-45221` — the twister's self-direction lerp:
        // `DirectionTo(Center + velocity)` = the velocity's own heading.
        let delta = p.vel;
        if !delta.is_nan() && delta != Vec2::ZERO {
            let vector2 = delta.normalize();
            p.vel = p.vel.lerp(vector2 * num3, amount);
        }
    } else {
        // `:45222-45225` — the snowflake's no-target decay.
        p.vel *= 0.92;
    }
    if p.kind == crate::projectile::WEATHER_PAIN_SHOT {
        // `:45226-45252` — the twister tail: the X-scaled rotation, the
        // 5-tick frame wrap and the %3 dust gate (the explicit velocity
        // draws stay; the dust bodies are the skip).
        p.rot = p.vel.x * 0.0125;
        p.frame_counter += 1;
        if p.frame_counter > 4 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= crate::projectile::proj_frames(p.kind) as i32 {
                p.frame = 0;
            }
        }
        if p.time_left % 3 == 0 {
            // `:45243` — the first dust's NextVector2Circular(2,2) pair.
            let _ = (rng.next_float(), rng.next_float());
            // `:45244-45251` — the two direct dusts' pairs.
            for _ in 0..2 {
                let _ = (rng.next_float(), rng.next_float());
            }
        }
    } else {
        // `:45254-45264` — the snowflake tail: the spin and the 1-in-3
        // dust gate (the taken branch's NextVector2Circular(1,1) pair).
        p.rot += 0.020943953 + p.vel.x.abs() * 0.2;
        if rng.next_range(0, 3) == 0 {
            let _ = (rng.next_float(), rng.next_float());
        }
    }
    sync_siblings
}

/// The 969 sibling-sync scan (`AI_177`'s `:45156-45164`): every OTHER
/// active same-owner same-kind twister with a SHORTER life and
/// `ai[1] < 555` gets its `ai[1]` pinned to 555 (the netUpdate rides).
/// The dispatch runs it after the core (the pool borrow the slot arm
/// cannot take). `me` is the twister's own slot index.
pub(crate) fn weather_pain_sync_siblings(slots: &mut [Projectile], me: usize) {
    let (owner, kind, time_left) = {
        let p = &slots[me];
        (p.owner, p.kind, p.time_left)
    };
    let num = 555.0f32;
    for (i, other) in slots.iter_mut().enumerate() {
        if i != me
            && other.active
            && other.owner == owner
            && other.kind == kind
            && time_left > other.time_left
            && other.ai[1] < num
        {
            other.ai[1] = num;
            other.net_update = true;
        }
    }
}

/// `AI_178_FireExplosion` (`Projectile.cs:45267-45317`) — the Firecracker's
/// explosion flash (918). The first tick (`localAI[0] == 0`) plays
/// `SoundID.Item14` and spawns the four dust rings + the two gores; then
/// `localAI[0]` counts every tick and the 10-tick cap kills.
///
/// RNG census (the first tick ONLY — the dust/gore BODIES are the skip,
/// the explicit call-site draws stay in the C#'s order):
/// - the 4× dust-31 ring (`:45272-45276`): each `RotatedByRandom(π)` pair
///   (two `NextDouble`s) + the radius `NextDouble` → 12 doubles;
/// - the 30× dust-6 pair (`:45277-45288`): each iteration two dusts, each
///   a `RotatedByRandom(π)` pair + a `NextDouble` radius → 180 doubles;
/// - the 10× dust-6 UnitX ring (`:45289-45295`): each a
///   `RotatedByRandom(π)` pair → 20 doubles (NO radius draw — the reach
///   is the fixed `width/2`);
/// - the 10× dust-31 UnitX ring (`:45296-45302`): the same → 20 doubles;
/// - the 2× gore (`:45303-45310`): each the position `Next(100)` ×2, the
///   type `Next(61,64)`, the `NewGore` internal kicks `Next(10,31)` +
///   `Next(-20,21)`, the `RotatedByRandom(π)` pair + the `NextDouble`
///   radius, then the velocity `Next(-10,11)` ×2 → 14 ranges + 6 doubles.
/// Total: 14 ranges + 238 doubles on tick 1, ZERO after.
pub(crate) fn ai_fire_explosion(p: &mut Projectile, rng: &mut dyn DropRng) {
    if p.local_ai[0] == 0.0 {
        // `:45271` — the explosion cue (the wants_sound seam).
        p.wants_sound = Some((2, 14, p.pos.x, p.pos.y, 0.0));
        // `:45272-45276` — the 4× dust-31 ring: RotatedByRandom pair +
        // the NextDouble radius each.
        for _ in 0..4 {
            let _ = (rng.next_double(), rng.next_double());
            let _ = rng.next_double();
        }
        // `:45277-45288` — the 30× dust-6 pair: two dusts per iteration,
        // each a RotatedByRandom pair + a NextDouble radius.
        for _ in 0..30 {
            for _ in 0..2 {
                let _ = (rng.next_double(), rng.next_double());
                let _ = rng.next_double();
            }
        }
        // `:45289-45295` — the 10× dust-6 UnitX ring: the pair only.
        for _ in 0..10 {
            let _ = (rng.next_double(), rng.next_double());
        }
        // `:45296-45302` — the 10× dust-31 UnitX ring: the pair only.
        for _ in 0..10 {
            let _ = (rng.next_double(), rng.next_double());
        }
        // `:45303-45310` — the 2× gore: the position pair, the type, the
        // NewGore kicks, the RotatedByRandom pair + radius, the velocity
        // kicks (the celeb2_explosion `gore` shape + the radius draws).
        for _ in 0..2 {
            let _ = rng.next_range(0, 100);
            let _ = rng.next_range(0, 100);
            let _ = rng.next_range(61, 64);
            let _ = rng.next_range(10, 31); // NewGore's Y kick
            let _ = rng.next_range(-20, 21); // NewGore's X kick
            let _ = (rng.next_double(), rng.next_double()); // RotatedByRandom
            let _ = rng.next_double(); // the radius
            let _ = rng.next_range(-10, 11); // velocity.X
            let _ = rng.next_range(-10, 11); // velocity.Y
        }
    }
    // `:45312-45316` — the age counter and the 10-tick kill.
    p.local_ai[0] += 1.0;
    if p.local_ai[0] >= 10.0 {
        p.active = false;
        p.time_left = 0;
    }
}

#[cfg(test)]
mod w4_d18_174_tests {
    //! W4-d18 — aiStyle 174 (`AI_174_MultisegmentPet`,
    //! `Projectile.cs:45345-45427`, the dispatch `:36659-36662`). WRITTEN
    //! NOT RUN (code-only mode). The rows, the keep-alive, the teleport,
    //! the zero-kick oldPos refill, the steering band and the clamp.
    use super::*;
    use crate::player::Player;
    use crate::projectile::{
        DESTROYER_PET, EATER_OF_WORLDS_PET, FlailInputs, LUNATIC_CULTIST_PET,
        ProjectilePool, YoyoInputs, proj_data,
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

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn owner(w: &World) -> Player {
        let mut pl = Player::spawn(w);
        pl.x = 1600.0;
        pl.y = 800.0;
        pl.vx = 0.0;
        pl.vy = 0.0;
        pl.facing = 1;
        pl
    }

    fn pet(kind: u16, x: f32, y: f32) -> Projectile {
        let d = proj_data(kind).expect("the 174 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.direction = 1;
        p
    }

    /// The rows (`Projectile.cs:8474-8485`/`:8496-8507`/`:8531-8542`):
    /// 20x20, aiStyle 174, penetrate -1, timeLeft 18000 (the ×5),
    /// tileCollide false, ignoreWater + netImportant, NOT minions.
    #[test]
    fn w4_d18_174_the_rows_match_set_defaults() {
        for kind in [EATER_OF_WORLDS_PET, DESTROYER_PET, LUNATIC_CULTIST_PET] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h, d.ai_style), (20.0, 20.0, 174), "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, 18000, "kind {kind}: 3600 × 5");
            assert!(!d.tile_collide && d.ignore_water && d.net_important, "kind {kind}");
            assert!(!d.minion && !d.sentry, "kind {kind}: pets, not minions");
            assert_eq!(d.extra_updates, 0, "kind {kind}");
        }
        // The frame sheets (`Main.cs:8671`/`:8675`/`:8681`).
        assert_eq!(crate::projectile::proj_frames(EATER_OF_WORLDS_PET), 3);
        assert_eq!(crate::projectile::proj_frames(DESTROYER_PET), 3);
        assert_eq!(crate::projectile::proj_frames(LUNATIC_CULTIST_PET), 4);
    }

    /// `:45347-45352` — an absent owner kills outright (active false,
    /// the C# does NOT zero timeLeft here).
    #[test]
    fn w4_d18_174_an_absent_owner_kills() {
        let w = world();
        let mut p = pet(EATER_OF_WORLDS_PET, 1600.0, 800.0);
        let tl = p.time_left;
        ai_multisegment_pet(&mut p, None, &w);
        assert!(!p.active);
        assert_eq!(p.time_left, tl, "no Kill() — the raw deactivate");
    }

    /// `:45353-45385` — the keep-alive: a standing flag pins timeLeft 2;
    /// death clears the flag FIRST so the death tick does not pin.
    #[test]
    fn w4_d18_174_the_keep_alive_pins_and_death_clears_first() {
        let w = world();
        let mut pl = owner(&w);
        for kind in [EATER_OF_WORLDS_PET, DESTROYER_PET, LUNATIC_CULTIST_PET] {
            pl.pet_flags[usize::from(kind)] = true;
            let mut p = pet(kind, 1600.0, 800.0);
            ai_multisegment_pet(&mut p, Some(&mut pl), &w);
            assert_eq!(p.time_left, 2, "kind {kind}: the standing flag pins");
            p.time_left = 18000;
            pl.dead = true;
            ai_multisegment_pet(&mut p, Some(&mut pl), &w);
            assert!(!pl.pet_flags[usize::from(kind)], "kind {kind}: death clears");
            assert_eq!(p.time_left, 18000, "kind {kind}: the cleared flag does not pin");
            pl.dead = false;
            pl.pet_flags[usize::from(kind)] = false;
        }
    }

    /// `:45386-45392` — the >2000-px teleport snaps the centre onto the
    /// owner's, zeroes the velocity and flags netUpdate.
    #[test]
    fn w4_d18_174_the_far_teleport() {
        let w = world();
        let mut pl = owner(&w);
        let mut p = pet(EATER_OF_WORLDS_PET, 1600.0 + 2500.0, 800.0);
        p.vel = Vec2::new(3.0, 0.0);
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        let (cx, cy) = pl.center();
        assert_eq!(p.pos, Vec2::new(cx - 10.0, cy - 10.0), "the centre snap");
        assert!(p.net_update);
        // The kick re-arms the velocity (the zeroed teleport velocity).
        assert_eq!(p.vel.x, 2.0, "the zero-kick after the teleport");
    }

    /// `:45396-45405` — the zero-velocity kick: X = 2·direction and the
    /// oldPos ring walks BACK from the position by the kick velocity.
    #[test]
    fn w4_d18_174_the_zero_kick_refills_old_pos() {
        let w = world();
        let mut pl = owner(&w);
        let mut p = pet(DESTROYER_PET, 1600.0, 800.0);
        p.vel = Vec2::ZERO;
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        assert_eq!(p.vel.x, 2.0, "2 × facing 1");
        // entry i = pos − (i+1)·vel (the kick's X only, Y drifts −0.1).
        for (i, slot) in p.old_pos.iter().enumerate() {
            let want = Vec2::new(1600.0 - 2.0 * (i as f32 + 1.0), 800.0);
            assert_eq!(*slot, want, "ring entry {i}");
        }
        pl.facing = -1;
        let mut p2 = pet(DESTROYER_PET, 1600.0, 800.0);
        p2.vel = Vec2::ZERO;
        ai_multisegment_pet(&mut p2, Some(&mut pl), &w);
        assert_eq!(p2.vel.x, -2.0, "the left-facing kick");
    }

    /// `:45393-45417` — the speed band: `Min(12, Max(4, |player.vel|))`,
    /// the >120-px steering, the cap and the −0.1 Y drift.
    #[test]
    fn w4_d18_174_the_speed_band_and_steering() {
        let w = world();
        let mut pl = owner(&w);
        let (cx, cy) = pl.center();
        // The owner stands still: the band floors at 4. The pet sits on
        // the owner's centre-Y so the aim is pure −X (no Y component to
        // fight the drift assertion).
        let mut p = pet(LUNATIC_CULTIST_PET, cx + 300.0 - 10.0, cy - 10.0);
        p.vel = Vec2::new(-4.0, 0.0);
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        assert!((p.vel.length() - 4.0).abs() < 2e-3, "the 4 floor (plus the −0.1 drift)");
        assert!(p.vel.x < 0.0, "still flying toward the owner");
        assert!(p.vel.y < 0.0, "the −0.1 drift under |Y| < 1");
        // A fast owner raises the band to its 12 cap.
        pl.vx = 30.0;
        let mut p2 = pet(LUNATIC_CULTIST_PET, cx + 300.0 - 10.0, cy - 10.0);
        p2.vel = Vec2::new(-20.0, 0.0);
        ai_multisegment_pet(&mut p2, Some(&mut pl), &w);
        assert!(p2.vel.length() <= 12.0 + 1e-3, "the 12 cap");
        // Inside 120 px NO steering runs — the velocity keeps its shape
        // (only the cap/drift touch it).
        let mut p3 = pet(LUNATIC_CULTIST_PET, cx + 50.0 - 10.0, cy - 10.0);
        p3.vel = Vec2::new(0.0, 3.0);
        ai_multisegment_pet(&mut p3, Some(&mut pl), &w);
        assert_eq!(p3.vel.x, 0.0, "no turn inside the 120 band");
        assert_eq!(p3.vel.y, 3.0, "|Y| ≥ 1 keeps the drift off");
    }

    /// `:45418-45426` — the rotation, the direction flip and the clamp.
    #[test]
    fn w4_d18_174_the_rotation_direction_and_clamp() {
        let w = world();
        let mut pl = owner(&w);
        let mut p = pet(EATER_OF_WORLDS_PET, 1600.0, 800.0);
        // |Y| = 1 keeps the drift off and the length √10 < 4 keeps the cap
        // off, so the velocity is untouched and the rotation is exact.
        p.vel = Vec2::new(3.0, 1.0);
        p.direction = -1;
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        assert_eq!(p.direction, 1, "the X-sign flip");
        assert_eq!(p.sprite_direction, 1);
        assert!(p.net_update, "the flip flags the net update");
        let want_rot = to_rotation(Vec2::new(3.0, 1.0)) + std::f32::consts::FRAC_PI_2;
        assert!((p.rot - want_rot).abs() < 1e-5, "rot = ToRotation + π/2");
        // The world-edge clamp: a position past the 160-px inset snaps.
        let mut p2 = pet(EATER_OF_WORLDS_PET, 10.0, 10.0);
        p2.vel = Vec2::new(4.0, 0.0);
        ai_multisegment_pet(&mut p2, Some(&mut pl), &w);
        assert_eq!(p2.pos.x, 160.0, "the left clamp");
        assert_eq!(p2.pos.y, 160.0, "the top clamp");
    }

    /// The dispatch wiring: one pool update runs the keep-alive pin.
    #[test]
    fn w4_d18_174_the_dispatch_arms_the_pet() {
        let mut w = world();
        let mut pl = owner(&w);
        pl.pet_flags[usize::from(EATER_OF_WORLDS_PET)] = true;
        let (cx, cy) = pl.center();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        // Near the owner (inside 120 px) so no teleport/steering flings
        // the pet out of the world before the lifetime block.
        let slot = pool
            .spawn_with_data(EATER_OF_WORLDS_PET, cx - 10.0, cy - 10.0, 0.0, 0.0, 0, 0.0)
            .unwrap();
        let mut rng = census();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        let p = &pool.slots()[slot];
        assert!(p.active);
        assert_eq!(p.time_left, 1, "the pin 2 minus the lifetime tick");
        assert!(rng.ranges.is_empty() && rng.floats == 0);
    }
}

#[cfg(test)]
mod w4_d18_175_tests {
    //! W4-d18 — aiStyle 175 (`AI_175_TitaniumStormShards`,
    //! `Projectile.cs:45319-45343`, the dispatch `:36663-36666`). WRITTEN
    //! NOT RUN (code-only mode). The row, the buff gate, the first-tick
    //! roll census, the orbit maths and the dispatch wiring.
    use super::*;
    use crate::player::Player;
    use crate::projectile::{
        FlailInputs, ProjectilePool, TITANIUM_STORM_SHARD, YoyoInputs, proj_data,
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

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn owner(w: &World) -> Player {
        let mut pl = Player::spawn(w);
        pl.x = 1600.0;
        pl.y = 800.0;
        pl.old_x = 1600.0;
        pl.old_y = 800.0;
        pl
    }

    fn shard(x: f32, y: f32) -> Projectile {
        let d = proj_data(TITANIUM_STORM_SHARD).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = TITANIUM_STORM_SHARD;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The row (`Projectile.cs:8626-8637`): 14x14, aiStyle 175,
    /// penetrate 1, timeLeft 3600, tileCollide false, ignoreWater.
    #[test]
    fn w4_d18_175_the_row_matches_set_defaults() {
        let d = proj_data(TITANIUM_STORM_SHARD).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (14.0, 14.0, 175));
        assert_eq!(d.penetrate, 1);
        assert_eq!(d.time_left, 3600);
        assert!(!d.tile_collide && d.ignore_water);
        assert_eq!(crate::projectile::proj_frames(TITANIUM_STORM_SHARD), 12);
    }

    /// `:45321-45326` — the absent owner, the dead owner and the missing
    /// buff each kill outright; ZERO draws on the kill paths.
    #[test]
    fn w4_d18_175_the_buff_gate_kills() {
        let w = world();
        let mut pl = owner(&w);
        let mut rng = census();
        let mut p = shard(1600.0, 800.0);
        ai_titanium_storm_shard(&mut p, None, true, 0, 1, &mut rng);
        assert!(!p.active && p.time_left == 0);
        let mut p2 = shard(1600.0, 800.0);
        ai_titanium_storm_shard(&mut p2, Some(&pl), false, 0, 1, &mut rng);
        assert!(!p2.active && p2.time_left == 0, "the missing buff");
        pl.dead = true;
        let mut p3 = shard(1600.0, 800.0);
        ai_titanium_storm_shard(&mut p3, Some(&pl), true, 0, 1, &mut rng);
        assert!(!p3.active && p3.time_left == 0, "the dead owner");
        assert!(rng.ranges.is_empty() && rng.floats == 0, "the kills roll nothing");
    }

    /// `:45327-45333` — the first tick rolls `Next(12)` then `NextFloat`
    /// (the rotation), latches frameCounter 1 and adds the π/200 spin;
    /// the second tick rolls NOTHING.
    #[test]
    fn w4_d18_175_the_first_tick_roll_census() {
        let w = world();
        let pl = owner(&w);
        let mut rng = census();
        let mut p = shard(1600.0, 800.0);
        ai_titanium_storm_shard(&mut p, Some(&pl), true, 0, 1, &mut rng);
        assert_eq!(rng.ranges, vec![(0, 12)], "the frame roll");
        assert_eq!(rng.floats, 1, "the rotation roll");
        assert_eq!(p.frame_counter, 1);
        assert_eq!(p.frame, 0, "CensusRng returns min");
        // rotation = 0·2π + π/200.
        assert_eq!(p.rot, std::f32::consts::PI / 200.0);
        ai_titanium_storm_shard(&mut p, Some(&pl), true, 0, 1, &mut rng);
        assert_eq!(rng.ranges.len(), 1, "no second frame roll");
        assert_eq!(rng.floats, 1, "no second rotation roll");
        assert_eq!(p.rot, std::f32::consts::PI / 200.0 * 2.0, "the spin stacks");
    }

    /// `:45335-45341` — the orbit maths: at miscCounter 0, index 0 of 1
    /// the angle is 0 (the +X ray), the radius 30, the target the owner's
    /// centre + (30, 0); the shard lerps 0.3 toward it and localAI[0]
    /// keeps the ray's Y (the draw-layer key).
    #[test]
    fn w4_d18_175_the_orbit_maths() {
        let w = world();
        let pl = owner(&w);
        let mut rng = census();
        let (cx, cy) = pl.center();
        let mut p = shard(cx - 7.0, cy - 7.0);
        ai_titanium_storm_shard(&mut p, Some(&pl), true, 0, 1, &mut rng);
        // angle 0 → ray (1, 0); target = centre + (30, 0); lerp 0.3 from
        // the centre → centre + (9, 0).
        assert_eq!(p.local_ai[0], 0.0, "the ray Y at angle 0");
        let want_center = Vec2::new(cx + 9.0, cy);
        assert!((p.pos.x + 7.0 - want_center.x).abs() < 1e-3);
        assert!((p.pos.y + 7.0 - want_center.y).abs() < 1e-3);
        // Index 1 of 2 at miscCounter 0: angle π → ray (−1, ~0); the
        // radius is 24 + 2·6 = 36, the lerp target centre + (−10.8, ~0).
        let mut p2 = shard(cx - 7.0, cy - 7.0);
        ai_titanium_storm_shard(&mut p2, Some(&pl), true, 1, 2, &mut rng);
        assert!(p2.pos.x + 7.0 < cx, "the π ray points left");
        // The owner's per-tick move rides the centre FIRST (`:45337-45338`).
        let mut pl2 = owner(&w);
        pl2.x += 10.0; // old_x stays → vector (10, 0)
        let mut p3 = shard(cx - 7.0, cy - 7.0);
        ai_titanium_storm_shard(&mut p3, Some(&pl2), true, 0, 1, &mut rng);
        // centre + (10,0) then lerp 0.3 toward (centre + 10 + 30, cy):
        // the player's centre also moved +10 → target cx + 40.
        let want = cx + 10.0 + (40.0 - 10.0) * 0.3;
        assert!((p3.pos.x + 7.0 - want).abs() < 1e-3, "the move-then-lerp order");
    }

    /// The dispatch wiring: a buffed owner's shard runs the first-tick
    /// roll through the pool update (the group pair pre-computed).
    #[test]
    fn w4_d18_175_the_dispatch_arms_the_shard() {
        let mut w = world();
        let mut pl = owner(&w);
        pl.buffs.add_buff(306, 600);
        let (cx, cy) = pl.center();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(TITANIUM_STORM_SHARD, cx - 7.0, cy - 7.0, 0.0, 0.0, 0, 0.0)
            .unwrap();
        let mut rng = census();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        let p = &pool.slots()[slot];
        assert!(p.active);
        assert_eq!(p.frame_counter, 1, "the first-tick latch ran");
        assert_eq!(rng.ranges, vec![(0, 12)]);
        assert_eq!(rng.floats, 1);
        // The buff-less owner's shard dies on the same seam.
        let mut pl2 = owner(&w);
        let slot2 = pool
            .spawn_with_data(TITANIUM_STORM_SHARD, cx - 7.0, cy - 7.0, 0.0, 0.0, 0, 0.0)
            .unwrap();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl2),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        assert!(!pool.slots()[slot2].active, "the missing buff kills");
    }
}

#[cfg(test)]
mod w4_d18_176_tests {
    //! W4-d18 — aiStyle 176 (`AI_176_EdgyLightning`,
    //! `Projectile.cs:45058-45095`, the dispatch `:36667-36670`). WRITTEN
    //! NOT RUN (code-only mode). The row, the target latch/acquire, the
    //! chase, the frame-roll census and the no-target kill.
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::{
        FlailInputs, ProjectilePool, SCYTHE_WHIP_PROJ, YoyoInputs, proj_data,
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

    fn bolt(x: f32, y: f32) -> Projectile {
        let d = proj_data(SCYTHE_WHIP_PROJ).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = SCYTHE_WHIP_PROJ;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p.tile_collide = d.tile_collide;
        p
    }

    fn npc_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        n
    }

    /// The row (`Projectile.cs:8676-8688`): 18x18, aiStyle 176,
    /// penetrate 3, tileCollide false, extraUpdates 2, local-immunity −1.
    #[test]
    fn w4_d18_176_the_row_matches_set_defaults() {
        let d = proj_data(SCYTHE_WHIP_PROJ).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (18.0, 18.0, 176));
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.extra_updates, 2);
        assert!(!d.tile_collide);
        assert!(d.uses_local_immunity && d.local_hit_cooldown == -1);
        assert_eq!(crate::projectile::proj_frames(SCYTHE_WHIP_PROJ), 6);
    }

    /// `:45066-45075` + `:45078-45084` — the acquire latches ai[0] =
    /// whoAmI+1, the velocity aims at 10 px and the rotation follows.
    #[test]
    fn w4_d18_176_the_acquire_and_chase() {
        let mut p = bolt(1600.0, 800.0);
        // The NPC 100 px to the right (its centre — the zombie box).
        let n = npc_at(1700.0, 800.0);
        let npcs = vec![n];
        let immune = [0i32; 256];
        let mut rng = census();
        ai_edgy_lightning(&mut p, &npcs, &immune, &mut rng);
        assert_eq!(p.ai[0], 1.0, "whoAmI 0 + 1");
        assert!(p.net_update, "the acquire flags the net update");
        assert!((p.vel.length() - 10.0).abs() < 1e-3, "the 10-px chase");
        assert!(p.vel.x > 0.0, "toward the right-hand target");
        assert!((p.rot - p.vel.y.atan2(p.vel.x)).abs() < 1e-4, "the rotation follows");
        // The frame counter climbs but rolls nothing before tick 3.
        assert_eq!(p.frame_counter, 1);
        assert!(rng.ranges.is_empty());
    }

    /// `:45085-45089` — the 3-tick frame machine rolls `Next(6)` exactly
    /// once per three ticks (the ONLY draw).
    #[test]
    fn w4_d18_176_the_frame_roll_census() {
        let mut p = bolt(1600.0, 800.0);
        let npcs = vec![npc_at(1700.0, 800.0)];
        let immune = [0i32; 256];
        let mut rng = census();
        for _ in 0..3 {
            ai_edgy_lightning(&mut p, &npcs, &immune, &mut rng);
        }
        assert_eq!(rng.ranges, vec![(0, 6)], "one Next(projFrames) at tick 3");
        assert_eq!(p.frame_counter, 0, "the reset");
        assert_eq!(p.frame, 0, "CensusRng returns min");
        assert_eq!(rng.floats, 0, "no float draws");
    }

    /// `:45060-45065` — a held target that dies or turns immune drops
    /// the latch and re-acquires; `:45091-45094` — no target kills.
    #[test]
    fn w4_d18_176_the_latch_revalidation_and_kill() {
        let mut p = bolt(1600.0, 800.0);
        p.ai[0] = 1.0; // holds NPC 0
        let mut n = npc_at(1700.0, 800.0);
        n.hp = 0; // dead → un-chaseable
        let npcs = vec![n];
        let immune = [0i32; 256];
        let mut rng = census();
        ai_edgy_lightning(&mut p, &npcs, &immune, &mut rng);
        assert!(!p.active && p.time_left == 0, "the dead target kills");
        // An immune held target drops the latch too (ai[0] = 0).
        let mut p2 = bolt(1600.0, 800.0);
        p2.ai[0] = 1.0;
        let npcs2 = vec![npc_at(5000.0, 800.0)]; // out of the 400 range
        let mut immune2 = [0i32; 256];
        immune2[0] = 5;
        ai_edgy_lightning(&mut p2, &npcs2, &immune2, &mut rng);
        assert_eq!(p2.ai[0], 0.0, "the immune target drops the latch");
        assert!(!p2.active, "no re-acquire in range → kill");
    }

    /// The dispatch wiring: a pool update acquires the nearby NPC.
    #[test]
    fn w4_d18_176_the_dispatch_arms_the_bolt() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs = vec![npc_at(1700.0, 800.0)];
        let slot = pool
            .spawn_with_data(SCYTHE_WHIP_PROJ, 1600.0, 800.0, 0.0, 0.0, 10, 0.0)
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
        assert_eq!(p.ai[0], 1.0, "the acquire ran through the dispatch");
        // extraUpdates 2 → 3 sub-steps → the frame machine rolled once.
        assert_eq!(rng.ranges, vec![(0, 6)]);
    }
}

#[cfg(test)]
mod w4_d18_177_tests {
    //! W4-d18 — aiStyle 177 (`AI_177_IceWhipSlicer`,
    //! `Projectile.cs:45117-45265`, the dispatch `:36671-36674`). WRITTEN
    //! NOT RUN (code-only mode). The rows, the 917 buff gate, the 969
    //! lifetime + sibling sync, the steering constants and the draw census.
    use super::*;
    use crate::npc::NpcKind;
    use crate::player::Player;
    use crate::projectile::{
        COOL_WHIP_PROJ, FlailInputs, ProjectilePool, WEATHER_PAIN_SHOT, YoyoInputs, proj_data,
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

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn owner(w: &World) -> Player {
        let mut pl = Player::spawn(w);
        pl.x = 1600.0;
        pl.y = 800.0;
        pl
    }

    fn slicer(kind: u16, x: f32, y: f32) -> Projectile {
        let d = proj_data(kind).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p
    }

    fn npc_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        n
    }

    /// The rows (`Projectile.cs:8701-8713`/`:9169-9181`).
    #[test]
    fn w4_d18_177_the_rows_match_set_defaults() {
        let d = proj_data(COOL_WHIP_PROJ).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (30.0, 30.0, 177));
        assert_eq!(d.penetrate, -1);
        assert!(!d.tile_collide);
        assert!(d.uses_local_immunity && d.local_hit_cooldown == 20);
        let d2 = proj_data(WEATHER_PAIN_SHOT).unwrap();
        assert_eq!((d2.w, d2.h, d2.ai_style), (56.0, 64.0, 177));
        assert_eq!(d2.penetrate, 15);
        assert!(d2.tile_collide && d2.magic);
        assert!(d2.uses_id_static_immunity && d2.id_static_hit_cooldown == 25);
        assert_eq!(crate::projectile::proj_frames(WEATHER_PAIN_SHOT), 8);
        assert_eq!(crate::projectile::proj_frames(COOL_WHIP_PROJ), 1);
    }

    /// `:45119-45129` — the 917 buff gate: absent/dead owner kills and
    /// clears the flag; a standing buff pins timeLeft 2.
    #[test]
    fn w4_d18_177_the_cool_whip_buff_gate() {
        let w = world();
        let mut pl = owner(&w);
        let npcs: Vec<Npc> = Vec::new();
        let immune = [0i32; 256];
        let mut rng = census();
        let mut p = slicer(COOL_WHIP_PROJ, 1600.0, 800.0);
        let sync = ai_ice_whip_slicer(&mut p, &w, &npcs, &immune, None, 0, &mut rng);
        assert!(!p.active && !sync, "the absent owner kills");
        pl.cool_whip_buff = true;
        let mut p2 = slicer(COOL_WHIP_PROJ, 1600.0, 800.0);
        ai_ice_whip_slicer(&mut p2, &w, &npcs, &immune, Some(&mut pl), 0, &mut rng);
        assert_eq!(p2.time_left, 2, "the standing buff pins");
        pl.dead = true;
        let mut p3 = slicer(COOL_WHIP_PROJ, 1600.0, 800.0);
        ai_ice_whip_slicer(&mut p3, &w, &npcs, &immune, Some(&mut pl), 0, &mut rng);
        assert!(!p3.active && !pl.cool_whip_buff, "death clears + kills");
    }

    /// `:45147-45155` — the 969 lifetime: ai[1] counts, the >560 alpha
    /// fade and the 570 kill.
    #[test]
    fn w4_d18_177_the_twister_lifetime() {
        let w = world();
        let npcs: Vec<Npc> = Vec::new();
        let immune = [0i32; 256];
        let mut rng = census();
        let mut p = slicer(WEATHER_PAIN_SHOT, 1600.0, 800.0);
        p.ai[1] = 560.0;
        let sync = ai_ice_whip_slicer(&mut p, &w, &npcs, &immune, None, 0, &mut rng);
        assert!(sync, "the 969 always requests the sibling scan");
        assert_eq!(p.ai[1], 561.0);
        assert_eq!(p.alpha, 25, "Lerp(0,250,1/10) = 25");
        assert!(p.active);
        let mut p2 = slicer(WEATHER_PAIN_SHOT, 1600.0, 800.0);
        p2.ai[1] = 569.0;
        ai_ice_whip_slicer(&mut p2, &w, &npcs, &immune, None, 0, &mut rng);
        assert_eq!(p2.ai[1], 570.0);
        assert!(!p2.active && p2.time_left == 0, "the 570 kill");
    }

    /// `:45156-45164` — the sibling sync pins older-living twisters'
    /// ai[1] up to 555 (the shorter-life / already-past ones stay).
    #[test]
    fn w4_d18_177_the_sibling_sync() {
        let mut slots = vec![Projectile::default(); 4];
        for (i, s) in slots.iter_mut().enumerate() {
            s.active = true;
            s.kind = WEATHER_PAIN_SHOT;
            s.owner = 0;
            s.time_left = 100;
            s.ai[1] = 10.0;
            if i == 0 {
                s.time_left = 200; // the caller outlives the rest
            }
        }
        slots[2].ai[1] = 600.0; // already past 555 — stays
        slots[3].owner = 1; // a foreign owner — stays
        weather_pain_sync_siblings(&mut slots, 0);
        assert_eq!(slots[1].ai[1], 555.0, "the younger sibling pinned");
        assert!(slots[1].net_update);
        assert_eq!(slots[2].ai[1], 600.0, "the past-555 sibling stays");
        assert_eq!(slots[3].ai[1], 10.0, "the foreign owner stays");
        assert_eq!(slots[0].ai[1], 10.0, "the caller itself stays");
    }

    /// `:45191-45225` — the steering: the 917 no-target ×0.92 decay and
    /// the 1-in-3 dust gate's draw census; the 969 self-direction lerp.
    #[test]
    fn w4_d18_177_the_no_target_arms_and_draws() {
        let w = world();
        let npcs: Vec<Npc> = Vec::new();
        let immune = [0i32; 256];
        let mut rng = census();
        let mut p = slicer(COOL_WHIP_PROJ, 1600.0, 800.0);
        p.vel = Vec2::new(10.0, 0.0);
        ai_ice_whip_slicer(&mut p, &w, &npcs, &immune, None, 0, &mut rng);
        assert!((p.vel.x - 9.2).abs() < 1e-4, "the 0.92 decay");
        // CensusRng returns min 0 → the 1-in-3 gate TAKEN → one pair.
        assert_eq!(rng.ranges, vec![(0, 3)]);
        assert_eq!(rng.floats, 2, "the taken branch's circular pair");
        assert!((p.rot - (0.020943953 + 9.2 * 0.2)).abs() < 1e-5, "the spin");
        // The 969 self-lerp: velocity (8,0) lerps 0.075 toward (8,0) —
        // unchanged; the %3 dust gate rolls 3 pairs at timeLeft 3600.
        let mut p2 = slicer(WEATHER_PAIN_SHOT, 1600.0, 800.0);
        p2.vel = Vec2::new(8.0, 0.0);
        let mut rng2 = census();
        ai_ice_whip_slicer(&mut p2, &w, &npcs, &immune, None, 0, &mut rng2);
        assert!((p2.vel.x - 8.0).abs() < 1e-4);
        assert_eq!(p2.rot, 8.0 * 0.0125, "the X-scaled rotation");
        assert_eq!(rng2.floats, 6, "the %3 gate's three circular pairs");
        assert!(rng2.ranges.is_empty(), "the 969 rolls no ranges");
    }

    /// The dispatch wiring: a 917 with a buffed owner pins through the
    /// pool update; a 969 runs its lifetime counter.
    #[test]
    fn w4_d18_177_the_dispatch_arms_the_slicers() {
        let mut w = world();
        let mut pl = owner(&w);
        pl.cool_whip_buff = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(COOL_WHIP_PROJ, 1600.0, 800.0, 0.0, 0.0, 10, 0.0)
            .unwrap();
        let mut rng = census();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        assert!(pool.slots()[slot].active);
        assert_eq!(pool.slots()[slot].time_left, 1, "the pin 2 minus the tick");
        let slot2 = pool
            .spawn_with_data(WEATHER_PAIN_SHOT, 1600.0, 900.0, 0.0, 0.0, 10, 0.0)
            .unwrap();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        assert_eq!(pool.slots()[slot2].ai[1], 1.0, "the twister counter ran");
    }
}

#[cfg(test)]
mod w4_d18_178_tests {
    //! W4-d18 — aiStyle 178 (`AI_178_FireExplosion`,
    //! `Projectile.cs:45267-45317`, the dispatch `:36675-36678`). WRITTEN
    //! NOT RUN (code-only mode). The row, the first-tick draw census, the
    //! 10-tick kill and the dispatch wiring.
    use super::*;
    use crate::projectile::{
        FIRE_WHIP_PROJ, FlailInputs, ProjectilePool, YoyoInputs, proj_data,
    };

    /// A draw-recording rng that counts the DOUBLE draws separately (the
    /// 178 burst is all `NextDouble` + the gore ranges).
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        doubles: usize,
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
        fn next_double(&mut self) -> f64 {
            self.doubles += 1;
            0.0
        }
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), doubles: 0, floats: 0 }
    }

    fn flash(x: f32, y: f32) -> Projectile {
        let d = proj_data(FIRE_WHIP_PROJ).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = FIRE_WHIP_PROJ;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The row (`Projectile.cs:8689-8700`): 96x96, aiStyle 178,
    /// penetrate -1, tileCollide false, alpha 255, local-immunity −1.
    #[test]
    fn w4_d18_178_the_row_matches_set_defaults() {
        let d = proj_data(FIRE_WHIP_PROJ).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (96.0, 96.0, 178));
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
        assert!(!d.tile_collide);
        assert!(d.uses_local_immunity && d.local_hit_cooldown == -1);
    }

    /// `:45269-45311` — the first-tick census: 16 ranges + 238 doubles,
    /// the Item14 cue, and the counter advances to 1.
    #[test]
    fn w4_d18_178_the_first_tick_census() {
        let mut p = flash(1600.0, 800.0);
        let mut rng = census();
        ai_fire_explosion(&mut p, &mut rng);
        // 4×3 + 30×2×3 + 10×2 + 10×2 + 2×3 = 12+180+20+20+6 = 238 doubles.
        assert_eq!(rng.doubles, 238, "the double census");
        // 2 gores × (2 position + 1 type + 2 NewGore kicks + 2 velocity) = 14.
        assert_eq!(rng.ranges.len(), 14, "the range census");
        assert_eq!(rng.floats, 0, "no float draws");
        assert_eq!(p.wants_sound, Some((2, 14, 1600.0, 800.0, 0.0)), "the Item14 cue");
        assert_eq!(p.local_ai[0], 1.0);
        assert!(p.active);
        // The gore range order: position, position, type, Y, X, velX, velY.
        assert_eq!(
            rng.ranges,
            vec![
                (0, 100), (0, 100), (61, 64), (10, 31), (-20, 21), (-10, 11), (-10, 11),
                (0, 100), (0, 100), (61, 64), (10, 31), (-20, 21), (-10, 11), (-10, 11),
            ]
        );
    }

    /// `:45312-45316` — the counter climbs every tick, ZERO draws after
    /// the first, and the 10-tick cap kills.
    #[test]
    fn w4_d18_178_the_ten_tick_kill() {
        let mut p = flash(1600.0, 800.0);
        let mut rng = census();
        for _ in 0..9 {
            ai_fire_explosion(&mut p, &mut rng);
        }
        assert_eq!(p.local_ai[0], 9.0);
        assert!(p.active, "9 is the last live tick");
        let doubles_after_first = rng.doubles;
        ai_fire_explosion(&mut p, &mut rng);
        assert_eq!(p.local_ai[0], 10.0);
        assert!(!p.active && p.time_left == 0, "the 10 kill");
        assert_eq!(rng.doubles, doubles_after_first, "no draws after tick 1");
        assert_eq!(rng.ranges.len(), 14, "no ranges after tick 1");
    }

    /// The dispatch wiring: a pool update runs the first-tick burst and
    /// forwards the Item14 cue into the sound queue.
    #[test]
    fn w4_d18_178_the_dispatch_arms_the_flash() {
        let mut w = World::new("t".into(), 300, 200);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let slot = pool
            .spawn_with_data(FIRE_WHIP_PROJ, 1600.0, 800.0, 0.0, 0.0, 0, 0.0)
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
        assert_eq!(p.local_ai[0], 1.0, "the first tick ran");
        assert_eq!(rng.doubles, 238);
    }
}
