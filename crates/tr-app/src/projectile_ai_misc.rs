//! S1-D-4 unarmed styles тАФ the four data-rowed projectile styles with NO
//! dispatch arm (census ┬з3c, `docs/research/50-entity-ai-census.md` on
//! `origin/docs/entity-ai-census`): **31** (PureSpray 145), **147**
//! (Celebration Mk2 rockets 715-718), **181** (FairyQueenRangedItemShot 932)
//! and **193** (Flamethrower Flames 85). 194 (R56) and 120 (R62) are other
//! workers' claims and stay out of this module.
//!
//! C# sources read WHOLE (the golden rule):
//! - aiStyle 31: the inline `AI()` arm `Projectile.cs:26786-26886` (the
//!   spray family's per-kind dust/conversion ladder, the owner-gated
//!   `WorldGen.Convert`, the 133 timeLeft cap, the ai[0] dust-scale ladder,
//!   the 0.3┬╖direction spin). The tile conversion is
//!   `tr_core::spread::convert_cleanse` (`WorldGen.cs:55067-55083` +
//!   the type-0 cell `:55527-55664`), `convert_sized_hallow` (type 2,
//!   `:55176-55260`) and `convert_sized_spray` (types 5/6/7,
//!   `:55363-55526`) — the W4-f25 C3/C4 cells wired at the per-kind dispatch.
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
//! kicks DO stay тАФ `Next(10, 31)` on Y first, then `Next(-20, 21)` on X
//! (`Gore.cs:1327-1328`, the LEAD-REVIEW G3 standard, ANALYSIS "Gore: the
//! pool"). Per style:
//! - 31: ZERO draws тАФ the arm's only rand-free state is the conversion,
//!   the cap, the counter and the spin; the dust ladder rolls nothing.
//! - 181: `Next(6)` every tick (`:43183`); on a hit one `NextFloat` for the
//!   dust colour's `lerpToWhite` argument (`:43189`). The colour helper
//!   itself is rand-free for non-dev names (`:21665-22041` тАФ only the
//!   "Random" name draws, unported).
//! - 193: one `NextFloat` gate while `localAI[0] < num3` (`:41372`); the
//!   taken branch's `NextVector2Circular` pair (`:41374`, two floats тАФ
//!   `Utils.cs:1266-1269`) + the `Next(4)` noGravity pick (`:41375`); one
//!   `NextFloat` gate while `localAI[0] >= num2` (`:41391`); the taken
//!   branch's `RotatedByRandom` pair (`:41394`), the `NextVector2Circular`
//!   pair (`:41395`) and the scale/customData floats (`:41398-41399`).
//!   Only the `NewDust*` INTERNAL rolls go with the dust pool. The 1106's
//!   sound gate (`:41366-41370`, S1-E-21h d75 part 4) draws NOTHING — the
//!   style overload's GetRandomPitch is audio-only and case 2's pitch
//!   rides the drain.
//! - 147: the rocket AI itself draws only the state-1 ring's per-particle
//!   colour `NextFloat` (`:52940`); the explosion censuses are in
//!   [`celeb2_explosion`]'s doc.
//!
//! Movement is the dispatch's `kill_step` (the `HandleMovement` default
//! arm's destination-box probe stand-in, `Projectile.cs:19849-19855`), run
//! AFTER the AI exactly as the C# runs `HandleMovement` after `AI()`
//! (`:17015`). The 147 state-4 swirl moves the position INSIDE the AI
//! (`:53059`) and the movement step adds the velocity on top тАФ faithful.
//!
//! `base.direction` (the 31 spin, `:26885`) uses the port's velocity-sign
//! stand-in тАФ the C# updates `direction` from `velocity.X`'s sign after
//! each sub-step's movement (`:17016-17026` + `AutomaticallyChangesDirection`
//! `:17281-17288`, true for aiStyle 31), the `projectile_ai001_a` precedent.
//!
//! Code-only round (S1-D-4): the witness tests below are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, alpha_of, can_hit, can_hit_line, chaseable, chaseable_held, csign,
    get_lerp_value, npc_center, remap, rotate, rotation_vector, safe_normalize, solid_tiles,
    wrap_angle, BUBBLE, DRILL_MOUNT_CROSSHAIR, FLAIRON_BUBBLE, SCUTLIX_LASER_CROSSHAIR,
    SCUTLIX_LASER_FRIENDLY,
};

/// `AI()`'s aiStyle-31 arm (`Projectile.cs:26786-26886`) тАФ the Clentaminator
/// spray. W4-d18 rowed the seven solution kinds (146-149/1015-1017 тАФ the
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
/// converts on this branch тАФ the counter/cap/spin run for every kind.
///
/// The dust-scale ladder (`:26845-26879`: the 7/3 threshold, the
/// 0.2/0.4/0.6/0.8 fade steps, the flag13 ├Ч1.2 and the `(int)(12┬╖scale)`
/// box growth) feeds ONLY the skipped `Dust.NewDust` тАФ it rolls nothing and
/// writes no slot state, so the port keeps just the `ai[0]++` both branches
/// share. ZERO `Main.rand` draws by construction.
/// `gen_rng` is the ACTIVE `genRand` stream of the projectile pass (W4-g83
/// contract §2 — the `UpdateProjectiles` swap): the cleanse's thorn/tree
/// kills' framing dispatch draws its seaweed tip re-rolls on it. The arm's
/// own census stays ZERO `Main.rand` draws (the conversion rolls nothing).
pub(crate) fn ai_spray(
    p: &mut Projectile,
    world: &mut World,
    local_owner: u8,
    gen_rng: &mut tr_core::UnifiedRandom,
) {
    // `:26788` — the solution flag (`ai[1] == 1` = the wider spray).
    let solution = p.ai[1] == 1.0;
    // `:26789-26825` — the per-kind conversion ladder. Dust types feed only
    // the skipped dust bodies; conversion types 1/4 are the d1 powder cells,
    // while type 0 is the cleanse cell. Other types remain unported here.
    let conversion: u8 = match p.kind {
        147 => 1,
        146 => 2,
        148 => 3,
        149 => 4,
        1015 => 5,
        1016 => 6,
        1017 => 7,
        _ => 0,
    };
    // `:26826-26835` — owner-gated conversion at Center.ToTileCoordinates,
    // size 2 (3 for the solution). `gen_rng` is the active UpdateProjectiles
    // genRand stream (W4-g83); cleanse framing may draw from it.
    if p.owner == local_owner {
        let tx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
        let ty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
        let size = if solution { 3 } else { 2 };
        match conversion {
            0 => tr_core::spread::convert_cleanse(world, tx, ty, size, gen_rng),
            1 | 4 => tr_core::spread::convert_sized(world, tx, ty, conversion, size),
            2 => tr_core::spread::convert_sized_hallow(world, tx, ty, size),
            // `:26805-26808` + g42's type-3 cell (`WorldGen.cs:55337-55358`).
            3 => tr_core::spread::convert_sized_mushroom(world, tx, ty, size),
            // `:26813-26824` + f25's type-5/6/7 cells
            // (`WorldGen.cs:55363-55526`).
            5..=7 => tr_core::spread::convert_sized_spray(world, tx, ty, conversion, size),
            _ => {}
        }
    }
    // `:26836-26839` тАФ the 133-tick life cap (per sub-step).
    if p.time_left > 133 {
        p.time_left = 133;
    }
    // `:26840-26884` тАФ the counter advances in BOTH branches; the >threshold
    // branch's scale ladder and single dust are dust-only (module doc).
    p.ai[0] += 1.0;
    // `:26885` тАФ the spin. `direction` is the velocity-sign stand-in
    // (module doc): `velocity.X < 0 тЖТ -1, else 1` (`:17018-17025`).
    let direction = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
    p.rot += 0.3 * direction;
}

/// `AI_193_Flamethrower` (`Projectile.cs:41337-41401`) тАФ the Flamethrower's
/// Flames (85). The 1106 Foxparks variant keeps its constants arm; its
/// sound gate (`:41366-41370`) is ARMED (S1-E-21h d75 part 4): the
/// `soundDelay == 0 && whoAmI % 3 == 0` latch queues `SoundID.FoxparksFlame`
/// at `base.Center` on `wants_sound` as the named-row marker `(2, -34)`
/// (the plain `(2, 34)` tuple is Item34's — d76's Nebula Blaze owns it),
/// `sound_delay = -1` (one cue per life). `who_am_i` is the dispatch's slot
/// index (the C# `base.whoAmI`); the 1106 row rides d18's pushed tip
/// `18916087b` — until the merge the kind cannot spawn and the gate stays
/// inert (the owed-arm convention).
///
/// The `Kill()` at 72 (`:41341-41344`) has NO return in the C# тАФ the arm
/// runs on into the velocity decay and the dust gates; the port keeps that
/// shape. `HandleMovement` then runs before the `!active` return
/// (`:17015`/`:17027`), so the dispatch's `kill_step` integrates the dead
/// slot's last step too (a nothing-reads write, faithful to the C#).
pub(crate) fn ai_flamethrower(
    p: &mut Projectile,
    rng: &mut dyn DropRng,
    who_am_i: usize,
) {
    // `:41339-41348` — the age counter, the 72-tick kill, the 60-tick decay.
    // `:41339-41348` тАФ the age counter, the 72-tick kill, the 60-tick decay.
    p.local_ai[0] += 1.0;
    let num = 60.0f32;
    if p.local_ai[0] >= num + 12.0 {
        p.active = false;
        p.time_left = 0;
    }
    if p.local_ai[0] >= num {
        p.vel *= 0.95;
    }
    // `:41349-41356` тАФ the Foxparks flag flips the smoke window off and the
    // flame window to the full 60.
    let foxparks = p.ai[0] == 1.0;
    let (mut num2, mut num3) = (50.0f32, 50.0f32);
    if foxparks {
        num2 = 0.0;
        num3 = num;
    }
    // `:41357-41371` — the per-kind dust constants. S1-E-21h d75 part 4 —
    // the 1106's sound gate (`:41366-41370`): `soundDelay == 0 &&
    // whoAmI % 3 == 0` latches −1 and queues FoxparksFlame at `base.Center`.
    // The intent's style slot carries −34, the NAMED-row marker: the real
    // (2, 34) tuple is ambiguous (Item34 volume 1.0 vs FoxparksFlame 0.15,
    // `SoundID.cs:1725`) and d76's Nebula Blaze queues the plain +34 — the
    // negative sentinel keeps the two apart (the dash-roar −1 precedent).
    // The drain routes −34 to `play_foxparks_flame` (the by-name façade).
    // The dust-window constants num4/num5 (60/50 → 15/15 for the 1106,
    // `:41363-41364`) feed only the skipped dust bodies — not carried.
    // `:41357-41371` тАФ the per-kind dust constants (the 1106 sound gate is
    // the documented skip).
    let mut chance_flame = 0.25f32;
    if p.kind == 1106 {
        chance_flame *= 0.5;
        if p.sound_delay == 0 && who_am_i % 3 == 0 {
            p.sound_delay = -1;
            p.wants_sound = Some((
                2,
                -34,
                p.pos.x + p.w * 0.5,
                p.pos.y + p.h * 0.5,
                0.0, 1.0));
        }
    }
    let chance_smoke = 0.5f32;
    // `:41372-41390` тАФ the flame dust gate: the `NextFloat` draw stays; on
    // the taken branch the position's `NextVector2Circular(num4, num4)`
    // pair (`Utils.cs:1266-1269`: NextVector2Unit's float + the radius
    // float) and the `Next(4)` noGravity pick stay too тАФ explicit
    // call-site draws (the nebula precedent). The `NewDustDirect` body
    // and its internal rolls go with the dust layer.
    if p.local_ai[0] < num3 && rng.next_float() < chance_flame {
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular
        let _ = rng.next_range(0, 4); // the noGravity pick (`:41375`)
    }
    // `:41391-41400` тАФ the smoke dust gate: the `NextFloat` draw stays; on
    // the taken branch the direction's `RotatedByRandom(0.19634954631328583)`
    // pair (`:41394`, `Utils.cs:1697-1700`), the position's
    // `NextVector2Circular(num5, num5)` pair (`:41395`) and the
    // scale/customData floats (`:41398-41399`) stay тАФ the dust body is the
    // skip. The owner-centre read feeds only the skipped dust velocity.
    if num2 > 0.0 && p.local_ai[0] >= num2 && rng.next_float() < chance_smoke {
        let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular
        let _ = rng.next_float(); // the scale wobble (`:41398`)
        let _ = rng.next_float(); // customData (`:41399`)
    }
}

/// `AI_181_FairyQueenRangedItemShot` (`Projectile.cs:43178-43193`) тАФ the
/// Empress drop-shot 932. The counter, the 10-tick alpha ramp and the
/// velocity rotation are the whole state; the 1-in-6 dust gate's `Next(6)`
/// stays on the stream and the taken branch's colour `NextFloat`
/// (`GetFairyQueenWeaponsColor(1f, NextFloat() * 0.4f)`, `:43189`) with it.
/// The dust body (`NewDustPerfect` 267, fadeIn/noGravity/alpha/scale) and
/// the colour computation itself are the dust layer's skip тАФ the colour
/// helper draws nothing for non-dev names (`:21665-22041`).
pub(crate) fn ai_fairy_queen_shot(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:43180` тАФ the age counter.
    p.ai[0] += 1.0;
    // `:43181` тАФ `alpha = (int)MathHelper.Lerp(255, 0,
    // GetLerpValue(0, 10, ai[0], clamped))`: XNA's `a + (b - a) * t` with
    // the truncating cast.
    let t = get_lerp_value(0.0, 10.0, p.ai[0]);
    p.alpha = (255.0f32 + (0.0f32 - 255.0f32) * t) as i32;
    // `:43182` тАФ `rotation = velocity.ToRotation()`.
    p.rot = p.vel.y.atan2(p.vel.x);
    // `:43183-43192` тАФ the 1-in-6 dust gate.
    if rng.next_range(0, 6) == 0 {
        // `:43189` тАФ the colour's lerpToWhite argument draw.
        let _ = rng.next_float();
        // The dust spawn тАФ SKIPPED (module doc).
    }
}

/// `AI_147_Celeb2Rocket` (`Projectile.cs:52881-53092`) тАФ the Celebration
/// Mk2 rockets 715-718. The seven `ai[0]` states: 0 the launch fall (the
/// 20-tick arm + the every-20-tick firework burst), 1 the pop (the тИТ10
/// kick, the 0.25 fall, the 2.5 scale cap, the 10-tick dust ring), 2 the
/// 60-tick fall, 3 the homing (the speed latch, the 800-Manhattan acquire
/// + the 1000-Manhattan hold, the (v┬╖7+d)/8 blend), 4 the swirl (the
/// ai[1]==1 +45 skip, the ╧А/45 phase, the perpendicular 3-px step), 5/6
/// the 40/30-tick falls. Every state clamps `velocity.Y` at 16.
///
/// RNG: the state-1 ring's per-particle colour `NextFloat` (`:52940`) is
/// the AI's only draw; the state-0 burst rides [`celeb2_explosion`].
/// `Lighting.AddLight` (`:52890`) and `GetCeleb2Color` (`:52550-52563`,
/// the pure ai[0] colour switch) feed light/dust only тАФ skipped.
pub(crate) fn ai_celeb2_rocket(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
) {
    // `:52886-52888` тАФ the age counter, the alpha pin, the rotation
    // (`velocity.ToRotation() + ╧А/2`).
    p.local_ai[0] += 1.0;
    p.alpha = 0;
    p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    let state = p.ai[0] as i32;
    match state {
        0 => {
            // `:52894-52902` тАФ the 20-tick arm: the fall and the infinite
            // penetration (`penetrate = (maxPenetrate = -1)`).
            if p.local_ai[0] >= 20.0 {
                p.vel.y += 0.12;
                p.max_penetrate = -1;
                p.penetrate = -1;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
            // `:52903-52906` тАФ the every-20-tick firework burst (strictly
            // past 20).
            if p.local_ai[0] > 20.0 && p.local_ai[0] % 20.0 == 0.0 {
                celeb2_explosion(p, world, rng);
            }
        }
        1 => {
            // `:52910-52921` тАФ the tick-10 kick, the 0.25 fall, the clamp.
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
                // `:52926-52930` тАФ the scale climb, capped at 2.5.
                p.scale += 0.015;
                if p.scale > 2.5 {
                    p.scale = 2.5;
                }
                // `:52931-52951` тАФ the 10-tick dust ring: the f32
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
            // `:52955-52962` тАФ the 60-tick fall.
            if p.local_ai[0] >= 60.0 {
                p.vel.y += 0.15;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        3 => {
            // `:52966-52972` тАФ the speed latch: `localAI[1]` holds the
            // first-seen `velocity.Length()` (the f32 sqrt seam, the W92
            // G5 class тАФ the C# casts `Math.Sqrt`'s double).
            let mut speed = p.local_ai[1];
            if speed == 0.0 {
                speed = p.vel.length();
                p.local_ai[1] = speed;
            }
            // `:52973-52983` тАФ the no-lock fall (the inner `>= 20` is
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
                // `:52988-53017` тАФ the acquire scan: the INTEGER half-width
                // centres (`(float)(width / 2)` on the C#'s int fields),
                // the Manhattan distance, the 800f radius and the
                // `Collision.CanHit` from a 1├Ч1 box at the centre.
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
                // `:53018-53032` тАФ the held target: the ignoreDontTakeDamage
                // chase gate + the 1000-Manhattan validity (a merely distant
                // hold keeps its index тАФ the C# never re-scans).
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
                // `:53033-53036` тАФ the friendly gate.
                if !p.friendly {
                    steer = false;
                }
                // `:53037-53049` тАФ the blend: the reciprocal-first
                // `num22 = speed / sqrt(dx┬▓ + dy┬▓)` (the C#'s unguarded
                // divide тАФ an exactly-coincident centre NaNs in both),
                // then `(v┬╖7 + d)/8`. The centre uses the FLOAT halves
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
            // `:53054-53057` тАФ the ai[1]==1 one-shot +45 phase skip.
            if p.local_ai[0] == 1.0 && p.ai[1] == 1.0 {
                p.local_ai[0] += 45.0;
            }
            // `:53058-53059` тАФ the swirl: the phase angle
            // `localAI[0] ┬╖ (╧А/45)`, the perpendicular step at the C#'s
            // 1.5707963705062866 literal (the double-cast seam).
            let dir = safe_normalize(p.vel);
            let x = rotate(dir, p.local_ai[0] * (std::f32::consts::PI / 45.0)).x;
            p.pos += rotate(dir, 1.5707963705062866f32) * x * 3.0;
            // `:53060-53068` тАФ the scale pin, the 10-tick fall, the clamp.
            p.scale = 2.0;
            if p.local_ai[0] >= 10.0 {
                p.vel.y += 0.04;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        5 => {
            // `:53072-53079` тАФ the 40-tick fall.
            if p.local_ai[0] >= 40.0 {
                p.vel.y += 0.08;
            }
            if p.vel.y > 16.0 {
                p.vel.y = 16.0;
            }
        }
        6 => {
            // `:53082-53089` тАФ the 30-tick fall.
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
/// snaps to the centre, the box grows to 128├Ч128 (240├Ч240 for the large
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
/// DISCARDED `NextFloat` (`:71114`), then the 10-step trail тАФ each step
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

/// S1-E-21h d77 — `Kill()`'s 710 DD2BetsyArrow death census
/// (`Projectile.cs:71660-71678`): the 2-iteration dust-31 lead
/// (`:71660-71664`) — per step a `RotatedByRandom(π)` pair (2 floats) +
/// a `NextDouble` radius; the 10-iteration dust-6 loop (`:71665-71671`)
/// — the same shape; the 5-iteration dust-31 tail (`:71672-71678`) —
/// per step a `RotatedByRandom(π)` pair only (the `UnitX` spin's
/// `.RotatedBy(velocity.ToRotation())` is draw-free). The `NewDust`
/// internals go with the dust pool (the module's RNG contract).
pub(crate) fn betsy_arrow_death_draws(rng: &mut dyn DropRng) {
    for _ in 0..2 {
        let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom
        let _ = rng.next_double(); // radius
    }
    for _ in 0..10 {
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
    }
    for _ in 0..5 {
        let _ = (rng.next_float(), rng.next_float());
    }
}

/// S1-E-21h d77 — `Kill()`'s 617 NebulaArcanum death census
/// (`Projectile.cs:72287-72326`): the 4-iteration dust-240 lead
/// (`:72287-72291`) — per step a `RotatedByRandom(π)` pair + a
/// `NextDouble` radius; the 30-iteration loop (`:72292-72304`) — per
/// step the dust-62's pair + double, then the dust-90's pair + double;
/// the 10-iteration dust-62 (`:72305-72311`) and dust-240
/// (`:72312-72318`) tails — per step a pair only; then 2 gore rows
/// (`:72319-72326`), each 7 ranges + a pair + a double: the position
/// `Next(100)` pair, the `Next(61, 64)` type, the `NewGore` kicks
/// `Next(10, 31)`/`Next(-20, 21)` (`Gore.cs:1327-1328`), the reposition
/// `RotatedByRandom(π)` pair + `NextDouble` radius, and the two
/// `Next(-10, 11)` velocity kicks. The dust/gore BODIES skip.
pub(crate) fn nebula_arcanum_death_draws(rng: &mut dyn DropRng) {
    for _ in 0..4 {
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
    }
    for _ in 0..30 {
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
    }
    for _ in 0..10 {
        let _ = (rng.next_float(), rng.next_float());
    }
    for _ in 0..10 {
        let _ = (rng.next_float(), rng.next_float());
    }
    for _ in 0..2 {
        let _ = rng.next_range(0, 100); // position X
        let _ = rng.next_range(0, 100); // position Y
        let _ = rng.next_range(61, 64); // gore type
        let _ = rng.next_range(10, 31); // NewGore Y kick
        let _ = rng.next_range(-20, 21); // NewGore X kick
        let _ = (rng.next_float(), rng.next_float()); // reposition spin
        let _ = rng.next_double(); // reposition radius
        let _ = rng.next_range(-10, 11); // velocity X kick
        let _ = rng.next_range(-10, 11); // velocity Y kick
    }
}

/// S1-E-21h d77 — `Kill()`'s 618/620 Nebula subshot/shard death census
/// (`Projectile.cs:72394-72417`, the shared `type == 620 || type == 618`
/// arm): the 10-iteration dust loop — per step a `Next(3)` gate
/// (`:72403`); the taken branch (`== 0`) adds the fadeIn/scale
/// `Next(-10, 11)` pair (`:72406-72407`). The dust TYPE is `(int)ai[0]`
/// (the 618's `Kill()` latches it to 86 at `:72398` — a state write the
/// caller owns); the colour/scale bodies skip. ZERO further draws.
pub(crate) fn nebula_shard_kill_draws(rng: &mut dyn DropRng) {
    for _ in 0..10 {
        if rng.next_range(0, 3) == 0 {
            let _ = rng.next_range(-10, 11); // fadeIn
            let _ = rng.next_range(-10, 11); // scale
        }
    }
}

/// `Kill()`'s aimed-spell resize (`Projectile.cs:71332-71337` / `:71412-71417`
/// / `:71260-71265`): `Resize(side, side)` snaps the position to the old
/// centre, grows the box, then recentres on the new half-box. The caller
/// saves the old box, runs [`ProjectilePool::damage_pve_boxed`] through the
/// resized box, then restores (the `gem_bolt_aoe` shape). RNG-free.
pub(crate) fn aimed_spell_kill_resize(p: &mut Projectile, side: f32) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    p.w = side;
    p.h = side;
    p.pos = center - Vec2::splat(side * 0.5);
}

/// `Kill()`'s Magic Missile trail-death dust census
/// (`Projectile.cs:71340-71407`): the 10-step `oldPos` ring — each step rolls
/// a colour `NextFloat` (`:71347`) + a `Next(1, 4)` count (`:71348`),
/// decrements the count past the ring's 0.3/0.75 marks (`:71350-71357`), then
/// per spawned dust a `Next(3)` gate (`:71362`): the taken branch draws THREE
/// floats (velocity scale `:71365`, scale `:71367`, fadeIn `:71368`), the else
/// branch one DISCARDED `Next(2)` (`:71381`) + one fadeIn float (`:71386`).
/// Then the 20-pair tail (`:71393-71406`): per pair a `NextVector2Circular`
/// (2 floats), a scale float, a `Next(1, 16)` position draw, then a second
/// dust's scale float + `Next(1, 16)`. The `CloneDust`/`NewDust` internals go
/// with the dust pool; a negative count spawns nothing (the C# loop guard).
pub(crate) fn magic_missile_death_draws(p: &Projectile, rng: &mut dyn DropRng) {
    for step in 0..p.old_pos.len() {
        if p.old_pos[step] == Vec2::ZERO {
            break; // `:71343-71346`
        }
        let _ = rng.next_float(); // `:71347` colour
        let mut count = rng.next_range(1, 4); // `:71348`
        let n = p.old_pos.len() as f32;
        if step as f32 >= n * 0.3 {
            count -= 1; // `:71350-71353`
        }
        if step as f32 >= n * 0.75 {
            count -= 2; // `:71354-71357`
        }
        for _ in 0..count.max(0) {
            if rng.next_range(0, 3) == 0 {
                // `:71362-71377` — the 267 dust's three explicit floats.
                let _ = rng.next_float();
                let _ = rng.next_float();
                let _ = rng.next_float();
            } else {
                // `:71378-71389` — the 15 dust: the discarded Next(2) + fadeIn.
                let _ = rng.next_range(0, 2);
                let _ = rng.next_float();
            }
        }
    }
    for _ in 0..20 {
        // `:71393-71406` — the two NewDustDirect(15) per pair.
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular
        let _ = rng.next_float(); // scale
        let _ = rng.next_range(1, 16); // position
        let _ = rng.next_float(); // second dust's scale
        let _ = rng.next_range(1, 16); // second dust's position
    }
}

/// `Kill()`'s Flamelash trail-death dust census
/// (`Projectile.cs:71420-71488`): the 10-step ring — colour `NextFloat`
/// (`:71427`) + `Next(1, 5)` count (`:71428`), the 0.3/0.75 decrements, then
/// per dust a `Next(3)` gate (`:71442`): the taken branch THREE floats
/// (`:71445-71447`), the else branch one `Next(2)` gate (`:71457`, its body
/// rolls nothing) . Then the 20-pair tail (`:71474-71487`): per pair one
/// fadeIn float (the first dust) and no explicit draw on the second.
pub(crate) fn flamelash_death_draws(p: &Projectile, rng: &mut dyn DropRng) {
    for step in 0..p.old_pos.len() {
        if p.old_pos[step] == Vec2::ZERO {
            break; // `:71423-71426`
        }
        let _ = rng.next_float(); // `:71427` colour
        let mut count = rng.next_range(1, 5); // `:71428`
        let n = p.old_pos.len() as f32;
        if step as f32 >= n * 0.3 {
            count -= 1; // `:71430-71433`
        }
        if step as f32 >= n * 0.75 {
            count -= 2; // `:71434-71437`
        }
        for _ in 0..count.max(0) {
            if rng.next_range(0, 3) == 0 {
                // `:71442-71455` — the 267 dust's three explicit floats.
                let _ = rng.next_float();
                let _ = rng.next_float();
                let _ = rng.next_float();
            } else {
                // `:71456-71467` — the 6 dust's Next(2) gate (body rolls nothing).
                let _ = rng.next_range(0, 2);
            }
        }
    }
    for _ in 0..20 {
        // `:71474-71487` — the first dust's fadeIn float; the second draws none.
        let _ = rng.next_float();
    }
}

/// `Kill()`'s Rainbow Rod trail-death dust census
/// (`Projectile.cs:71268-71327`): one lead `NextFloat` (`:71268`), the 10-step
/// ring — `Next(1, 4)` count (`:71275`) with the 0.3/0.65/0.85 decrements
/// (`:71278-71289`), then per spawned dust FOUR floats (velocity scale
/// `:71295`, scale `:71297`, fadeIn `:71298`, and the clone's colour is fixed)
/// — no `Next(3)` gate here (every dust is the 267 branch). Then the 14-dust
/// tail (`:71320-71326`): one colour `NextFloat` each. The `CloneDust`/
/// `NewDust` internals go with the dust pool.
pub(crate) fn rainbow_rod_death_draws(p: &Projectile, rng: &mut dyn DropRng) {
    let _ = rng.next_float(); // `:71268` lead hue
    for step in 0..p.old_pos.len() {
        if p.old_pos[step] == Vec2::ZERO {
            break; // `:71271-71274`
        }
        let mut count = rng.next_range(1, 4); // `:71275`
        let n = p.old_pos.len() as f32;
        if step as f32 >= n * 0.3 {
            count -= 1; // `:71278-71281`
        }
        if step as f32 >= n * 0.65 {
            count -= 2; // `:71282-71285`
        }
        if step as f32 >= n * 0.85 {
            count -= 3; // `:71286-71289`
        }
        for _ in 0..count.max(0) {
            // `:71292-71308` — the 267 dust's three explicit floats.
            let _ = rng.next_float();
            let _ = rng.next_float();
            let _ = rng.next_float();
        }
    }
    for _ in 0..14 {
        let _ = rng.next_float(); // `:71322` colour
    }
}

/// `Kill()`'s Nebula Blaze death census (`Projectile.cs:71995-72050`): one
/// leading `SelectRandom` dust-type pick (`:71997`, `Next(5)` for 634 /
/// `Next(3)` for 635 — `Utils.SelectRandom` draws `Next(choices.Length)`),
/// then the 40-iteration loop (`:72021-72041`): per step a SECOND type pick
/// (`:72023`/`:72026`), the first dust's `RotatedByRandom` pair (2 floats) +
/// `NextDouble` radius + `NextFloat` velocity (`:72029-72032`), the second
/// dust's `RotatedByRandom` pair + `NextDouble` + `NextFloat` (`:72034-72040`).
/// Then the 20-iteration tail (`:72042-72049`): per step a `RotatedByRandom`
/// pair + one `NextFloat` velocity. The `NewDust` internals go with the pool.
pub(crate) fn nebula_blaze_death_draws(kind: u16, rng: &mut dyn DropRng) {
    // `:71997`/`:72008` — the leading SelectRandom (5 choices for 634, 3 for 635).
    let _ = rng.next_range(0, if kind == 635 { 3 } else { 5 });
    for _ in 0..40 {
        // `:72023`/`:72026` — the per-step type pick (same choice count).
        let _ = rng.next_range(0, if kind == 635 { 3 } else { 5 });
        // `:72029-72032` — first dust: RotatedByRandom(2) + NextDouble + NextFloat.
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
        let _ = rng.next_float();
        // `:72034-72040` — second dust: same shape.
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
        let _ = rng.next_float();
    }
    for _ in 0..20 {
        // `:72045-72048` — RotatedByRandom(2) + NextFloat velocity.
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_float();
    }
}

/// `Kill()`'s Spirit Flame death census (`Projectile.cs:72512-72541`): the
/// 30-iteration loop (`:72515-72531`) — per step one scale `NextFloat`
/// (`:72517`); the first 15 steps add a fadeIn float + a velocity float
/// (`:72521-72524`), the last 15 a `RotatedByRandom` pair + a radius float +
/// a fadeIn float (`:72527-72529`). Then the 10-iteration tail
/// (`:72532-72540`): per step a `RotatedByRandom` pair + a fadeIn float.
pub(crate) fn spirit_flame_death_draws(rng: &mut dyn DropRng) {
    for step in 0..30 {
        let _ = rng.next_float(); // `:72517` scale
        if step < 15 {
            let _ = rng.next_float(); // `:72521` fadeIn
            let _ = rng.next_float(); // `:72524` velocity
        } else {
            let _ = (rng.next_float(), rng.next_float()); // `:72527` RotatedByRandom
            let _ = rng.next_float(); // `:72527` radius
            let _ = rng.next_float(); // `:72528` fadeIn
        }
    }
    for _ in 0..10 {
        let _ = (rng.next_float(), rng.next_float()); // `:72535` RotatedByRandom
        let _ = rng.next_float(); // `:72536` fadeIn
    }
}

/// `Kill()`'s Onyx Black Bolt death census (`Projectile.cs:72236-72276`): the
/// 4-iteration lead (`:72238-72241`) — per step a `RotatedByRandom` pair + a
/// `NextDouble` radius. The 20-iteration loop (`:72242-72260`) — per step the
/// first dust's `RotatedByRandom` pair + `NextDouble` + a `NextFloat` velocity
/// boost (`:72248`), the second dust's `RotatedByRandom` pair + `NextDouble`
/// (`:72251-72252`, its velocity boost is the fixed 8f). The 20-iteration
/// (`:72261-72268`) and 70-iteration (`:72269-72276`) tails — per step a
/// `RotatedByRandom` pair + `NextDouble` radius (the velocity boosts are
/// fixed). The `NewDust` internals go with the pool.
pub(crate) fn onyx_bolt_death_draws(rng: &mut dyn DropRng) {
    for _ in 0..4 {
        let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom
        let _ = rng.next_double(); // radius
    }
    for _ in 0..20 {
        // first dust: RotatedByRandom + NextDouble + NextFloat boost.
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
        let _ = rng.next_float(); // `:72248`
        // second dust: RotatedByRandom + NextDouble (fixed 8f boost).
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
    }
    for _ in 0..20 {
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
    }
    for _ in 0..70 {
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_double();
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
    p.wants_sound = Some((2, 54, p.pos.x, p.pos.y, 0.0, 1.0));
    for _ in 0..count {
        // `:72868`/`:72938` — the hslToRgb hue draw (0.4 + NextDouble *
        // 0.2); the colour body is the dust layer's.
        let _ = rng.next_double();

        }
        }

/// W4-f50 deviation 2 — the Solar Flare hurt blast's `Kill()` dust/gore
/// burst BODIES (`Projectile.cs:72183-72224`), ported through the dust seam
/// in the g64 adapter pattern (`BlastRng`: `DustRng`/`GoreRng` over the
/// caller's `&mut dyn DropRng`, `Next(max) = Next(0, max)` — one `Sample()`
/// step, `UnifiedRandom.cs:120-141`; g64's `DustDraw` is the same shape,
/// keep-one at merge). Every explicit `Main.rand` draw stays in the C#'s
/// order AND the `NewDust`/`NewGore` internal draws now ride the pools:
/// - 4× dust 31 (`:72183-72187`): alpha 100, scale 1.5; the position
///   overwrite `Center + UnitY.RotatedByRandom(π) * (float)NextDouble() *
///   width / 2f` (TWO `NextDouble` + one radius double).
/// - 30× dust-6 pairs (`:72188-72201`): the first alpha 200 scale 3.7,
///   `noGravity`, velocity ×3; the second alpha 100 scale 1.5, velocity ×2,
///   `noGravity`, `fadeIn = 2.5`. Each its own spin + radius triple.
/// - 10× dust 6 (`:72202-72209`): alpha 0 scale 2.7, `noGravity`, velocity
///   ×3; the position `Center + UnitX.RotatedByRandom(π).RotatedBy(
///   velocity.ToRotation()) * width / 2f` — NO radius double.
/// - 10× dust 31 (`:72210-72216`): alpha 0 scale 1.5, the same UnitX shape.
/// - 2× gore 61-63 (`:72217-72224`): the spawn position `pos + (width *
///   Next(100) / 100f, height * Next(100) / 100f) - One * 10`, type
///   `Next(61, 64)`; the `NewGore` kicks (`Gore.cs:1327-1328`); the
///   reposition spin + radius; velocity ×0.3 then the two `Next(-10, 11) *
///   0.05f` kicks.
/// The `shader` post-arms are the dye layer's (dropped fields). The C#
/// writes into `Main.dust[6000]`/`Main.gore[600]` on a refused spawn — the
/// port's sentinel slots receive the same writes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn solar_flare_blast_bodies(
    rng: &mut dyn DropRng,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
    gore: &mut tr_core::gore::GorePool,
    pos: Vec2,
    w: f32,
    h: f32,
    center: Vec2,
    vel: Vec2,
) {
    let mut r = BlastRng(rng);
    // `3.1415927410125732` — the C# float PI literal widened to double.
    const PI_F: f64 = 3.1415927410125732;
    let (wi, hi) = (w as i32, h as i32);
    let box_pos = pos;
    // `UnitY.RotatedByRandom(π) * (float)NextDouble() * width / 2f`.
    let ring_y = |r: &mut BlastRng| -> Vec2 {
        let angle = r.0.next_double() * PI_F - r.0.next_double() * PI_F;
        let radius = r.0.next_double() as f32 * w / 2.0;
        center + crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), angle) * radius
    };
    // `UnitX.RotatedByRandom(π).RotatedBy(velocity.ToRotation()) * width/2f`
    // — `ToRotation` is `(float)Math.Atan2(Y, X)` (`Utils.cs:1655-1658`).
    let rot = (vel.y as f64).atan2(vel.x as f64) as f32;
    let ring_x = |r: &mut BlastRng| -> Vec2 {
        let angle = r.0.next_double() * PI_F - r.0.next_double() * PI_F;
        let spun = crate::bow_launch::rotated_by(Vec2::new(1.0, 0.0), angle);
        center + crate::bow_launch::rotated_by(spun, rot as f64) * (w / 2.0)
    };
    let spawn = |r: &mut BlastRng,
                 dust: &mut tr_core::particles::DustPool,
                 dust_type: i32,
                 alpha: i32,
                 scale: f32|
     -> usize {
        dust.new_dust(
            r,
            ctx,
            world,
            box_pos,
            wi,
            hi,
            dust_type,
            0.0,
            0.0,
            alpha,
            tr_core::particles::Color::default(),
            scale,
        )
    };
    // `:72183-72187` — 4× dust 31.
    for _ in 0..4 {
        let idx = spawn(&mut r, dust, 31, 100, 1.5);
        dust.slot_mut(idx).position = ring_y(&mut r);
    }
    // `:72188-72201` — 30× dust-6 pairs.
    for _ in 0..30 {
        let idx = spawn(&mut r, dust, 6, 200, 3.7);
        let d = dust.slot_mut(idx);
        d.position = ring_y(&mut r);
        d.no_gravity = true;
        d.velocity *= 3.0;
        let idx = spawn(&mut r, dust, 6, 100, 1.5);
        let d = dust.slot_mut(idx);
        d.position = ring_y(&mut r);
        d.velocity *= 2.0;
        d.no_gravity = true;
        d.fade_in = 2.5;
    }
    // `:72202-72209` — 10× dust 6, the velocity-rotated UnitX ring.
    for _ in 0..10 {
        let idx = spawn(&mut r, dust, 6, 0, 2.7);
        let d = dust.slot_mut(idx);
        d.position = ring_x(&mut r);
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    // `:72210-72216` — 10× dust 31, the same ring.
    for _ in 0..10 {
        let idx = spawn(&mut r, dust, 31, 0, 1.5);
        let d = dust.slot_mut(idx);
        d.position = ring_x(&mut r);
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    // `:72217-72224` — 2× gore 61-63.
    for _ in 0..2 {
        let gx = (wi * r.0.next_range(0, 100)) as f32 / 100.0;
        let gy = (hi * r.0.next_range(0, 100)) as f32 / 100.0;
        let gore_type = r.0.next_range(61, 64);
        let idx = gore.new_gore(
            &mut r,
            world,
            pos + Vec2::new(gx, gy) - Vec2::ONE * 10.0,
            Vec2::ZERO,
            gore_type,
            1.0,
        );
        let g = gore.slot_mut(idx);
        g.position = ring_y(&mut r);
        g.velocity *= 0.3;
        g.velocity.x += r.0.next_range(-10, 11) as f32 * 0.05;
        g.velocity.y += r.0.next_range(-10, 11) as f32 * 0.05;
    }
}

/// The g64-pattern adapter: the pool traits over the machine's
/// `&mut dyn DropRng` stream. `Next(max) = Next(0, max)` — the SAME single
/// `Sample()` step (`UnifiedRandom.cs:134-141`); `NextFloatDirection` =
/// `(float)NextDouble() * 2f - 1f` (`Utils.cs:1203-1206`).
pub(crate) struct BlastRng<'a>(pub &'a mut dyn DropRng);

impl tr_core::particles::DustRng for BlastRng<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next_range(0, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
}

impl tr_core::gore::GoreRng for BlastRng<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next_range(0, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
    fn next_float_direction(&mut self) -> f32 {
        self.0.next_double() as f32 * 2.0 - 1.0
    }
}
/// The DRAW CENSUS oracle (the pre-body form, kept for the stream-parity
/// witness): every EXPLICIT `Main.rand` draw of the burst in the C#'s
/// order — the bodies fn above consumes exactly these plus the
/// `NewDust`/`NewGore` internal draws.
#[cfg(test)]
pub(crate) fn solar_flare_blast_draws(rng: &mut dyn DropRng) {
    // `RotatedByRandom(π)` = two `NextDouble` (`Utils.cs:1697-1700`).
    fn rotated(rng: &mut dyn DropRng) {
        let _ = rng.next_double();
        let _ = rng.next_double();
    }
    // 4× dust 31: RotatedByRandom + the radius NextDouble each.
    for _ in 0..4 {
        rotated(rng);
        let _ = rng.next_double(); // the `(float)NextDouble() * width / 2f`
    }
    // 30× dust-6 pairs: two (RotatedByRandom + NextDouble) each.
    for _ in 0..30 {
        rotated(rng);
        let _ = rng.next_double();
        rotated(rng);
        let _ = rng.next_double();
    }
    // 10× dust 6: RotatedByRandom each (the velocity-rotation draws nothing).
    for _ in 0..10 {
        rotated(rng);
    }
    // 10× dust 31: RotatedByRandom each.
    for _ in 0..10 {
        rotated(rng);
    }
    // 2× gore: the position pair, the type, the RotatedByRandom +
    // NextDouble reposition and the two velocity kicks (`:72217-72224`).
    // The `NewGore` internal kicks (`Next(10, 31)`/`Next(-20, 21)`,
    // `Gore.cs:1327-1328`) are NOT call-site draws — they ride the pool.
    for _ in 0..2 {
        let _ = rng.next_range(0, 100);
        let _ = rng.next_range(0, 100);
        let _ = rng.next_range(61, 64);
        rotated(rng);
        let _ = rng.next_double();
        let _ = rng.next_range(-10, 11);
        let _ = rng.next_range(-10, 11);
    }
}

/// `AI_147_Explosion` (`Projectile.cs:53130-53399`) — the per-state
/// `AI_147_Explosion` (`Projectile.cs:53130-53399`) тАФ the per-state
/// firework burst, as the DRAW CENSUS: every explicit `Main.rand` draw
/// stays on the stream in the C#'s order, the dust/gore bodies are
/// skipped. Per state (the gore rows are 7 range draws each: the position
/// `Next(100)` pair, the `Next(61, 64)` type, the `NewGore` kicks
/// `Next(10, 31)`/`Next(-20, 21)` тАФ `Gore.cs:1327-1328` тАФ and the two
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
/// тАФ the iteration count is the source's own, drift included.
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
            // `:53136-53155` тАФ the dust rolls nothing; one gore.
            gore(rng);
        }
        1 => {
            // `:53162` тАФ the discarded draw (the C# keeps it on the stream).
            let _ = rng.next_float();
            // `:53163-53174` тАФ the 0.0125 ring: colour + velocity scale.
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float();
                let _ = rng.next_float();
                t += 0.0125;
            }
            // `:53175-53181` тАФ three gores.
            for _ in 0..3 {
                gore(rng);
            }
            // `:53182` тАФ the ground explosion.
            create_ground_explosion(p, world, rng, 40.0, 20, 100, 2, 10, 2);
        }
        2 => {
            // `:53191-53197` тАФ the big-burst pick and the star rotation.
            let big = rng.next_range(0, 4) == 0;
            let _ = rng.next_float(); // NextFloatDirection (`Utils.cs:1203-1206`)
            // `:53198-53224` тАФ the 5-spoke web: no explicit draws.
            // `:53225-53270` тАФ both branches end in one gore.
            let _ = big;
            gore(rng);
        }
        3 => {
            // `:53281` тАФ the ring rotation.
            let _ = rng.next_float();
            // `:53282-53298` тАФ 40 colour draws.
            for _ in 0..40 {
                let _ = rng.next_float();
            }
            // `:53299-53305` тАФ one gore.
            gore(rng);
        }
        4 => {
            // `:53312` тАФ the discarded draw.
            let _ = rng.next_float();
            // `:53313-53324` тАФ the 0.025 ring: one velocity-scale float each.
            let mut t = 0.0f32;
            while t < 1.0 {
                let _ = rng.next_float();
                t += 0.025;
            }
            // `:53325-53331` тАФ three gores.
            for _ in 0..3 {
                gore(rng);
            }
        }
        5 => {
            // `:53338-53347` тАФ 20 colour draws (the 27 dust is fixed-colour).
            for _ in 0..20 {
                let _ = rng.next_float();
            }
        }
        6 => {
            // `:53358-53388` тАФ the three radius-wobble rings.
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
            // `:53389-53395` тАФ one gore.
            gore(rng);
        }
        _ => {}
    }
}

/// `WorldGen.KillTile_GetTileDustAmount(fail: true, tile)`
/// (`WorldGen.cs:66597-66685`) тАФ the per-kind dust count the ground
/// explosion's tile walk reads. The `fail: true` arms only (the sole
/// caller passes true).
pub(crate) fn kill_tile_dust_amount_fail(kind: u16, frame_x: i16) -> i32 {
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
/// layerEnd, layerJump)` (`Projectile.cs:53401-53464`) тАФ the state-1
/// burst's ground dust: the fluffed tile box, the four layers (2/4/6/8),
/// the radius gate, the exposed-solid-tile gate, then per tile the dust
/// amount's `NextFloat` velocity scales (the `KillTile_MakeTileDust`
/// bodies are the dust skip; its one explicit `Next(3)` at `:67402` is
/// the type-30 arm тАФ not a full solid, unreachable here), the `Next(3)`
/// gore gate and the taken branch's gore (`61 + Next(3)` type + the
/// NewGore kicks + one `NextFloat` velocity scale).
///
/// The `InWorld(j, k, 10)` refusal RETURNS from the whole walk (`:53418-
/// 53421`) тАФ the C#'s early exit, kept.
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
    // `:53403-53410` тАФ the fluffed tile box (the `(int)(X/16)` truncation)
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
                // `:53418-53421` тАФ the InWorld(10) refusal aborts the walk.
                if j < 10 || j >= world.width - 10 || k < 10 || k >= world.height - 10 {
                    return;
                }
                // `:53422-53425` тАФ the radius gate.
                if center.distance(Vec2::new(j as f32 * 16.0, k as f32 * 16.0)) > radius {
                    continue;
                }
                // `:53426-53435` тАФ the exposed full-solid gate.
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
                // `:53436-53443` тАФ the first dust row: one NextFloat each.
                let amount = kill_tile_dust_amount_fail(t.kind, t.frame_x);
                for _ in 0..amount {
                    let _ = rng.next_float();
                }
                // `:53444-53452` тАФ the layer >= 2 second row (amount - 1).
                if layer >= 2 {
                    for _ in 0..(amount - 1) {
                        let _ = rng.next_float();
                    }
                }
                // `:53453-53460` тАФ the 2-in-3 gore gate.
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

/// `AI_148_StarSpawner` (`Projectile.cs:53094-53128`) — the invisible
/// starfall spawner (720, W4-g70 lane 8). The day kill (`:53096-53100`),
/// the `ai[0] += dayRate` age, the one-shot `localAI[0]` latch that calls
/// `Star.StarFall(position.X)` while the local player is above the surface
/// (`:53102-53109`), and the 180-tick conversion into the natural falling
/// star 12 (damage 1000, knockback 10, `:53110-53127`).
///
/// Returns `true` on the conversion tick — the caller runs
/// [`star_spawner_convert`] with the player centre, spawns the 12 at this
/// slot's position/velocity and kills the slot. The `StarFall` request
/// rides `starfall_x` — the pool cannot borrow the star field, so the
/// caller drains it after the update (the `FlailSpawnRequest` precedent).
/// ZERO `Main.rand` draws: the Fall's two rolls happen in the star field's
/// own pass, on this same stream.
pub(crate) fn ai_star_spawner(
    p: &mut Projectile,
    day_time: bool,
    remix: bool,
    day_rate: i32,
    player_above_surface: bool,
    local_owner: u8,
    starfall_x: &mut Option<f32>,
) -> bool {
    // `:53096-53100` — the day kill (the remix world keeps it alive).
    if day_time && !remix {
        p.active = false;
        p.time_left = 0;
        return false;
    }
    // `:53101` — the age counter rides the day rate.
    p.ai[0] += day_rate as f32;
    // `:53102-53109` — the one-shot latch: `Star.StarFall(base.position.X)`
    // while the local player is above the surface line.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        if player_above_surface {
            *starfall_x = Some(p.pos.x);
        }
    }
    // `:53110-53113` — the owner + 180-tick gate.
    p.owner == local_owner && p.ai[0] >= 180.0
}

/// The conversion half of `AI_148_StarSpawner` (`Projectile.cs:53114-53125`):
/// the velocity's X shrinks to 0.35 and turns toward the locked player's
/// centre (`ai[1] > -1` — the starfall spawn's default ai1 = 0 passes).
pub(crate) fn star_spawner_convert(p: &mut Projectile, target_center_x: f32) {
    if p.ai[1] > -1.0 {
        p.vel.x *= 0.35;
        if p.pos.x + p.w * 0.5 < target_center_x {
            p.vel.x = p.vel.x.abs();
        } else {
            p.vel.x = -p.vel.x.abs();
        }
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
                let _jet = fountain_jet(jet_speed, 1.5707963705062866, rng);
                let _ = rng.next_double(); // the dust scale (`:30503`)
                let _ = rng.next_range(-2, 3); // the position jitter (`:30509`)
            }
        }
        420 => {
            // `:30516-30541` — the red: two dust-219 (1-in-5 dust-222).
            for _ in 0..2 {
                let _jet = fountain_jet(jet_speed, 1.5707963705062866, rng);
                let _ = rng.next_range(0, 5); // the colour pick (`:30522`)
                let _ = rng.next_double(); // the dust scale (`:30528`)
                let _ = rng.next_range(-2, 3); // the position jitter (`:30534`)
            }
        }
        421 => {
            // `:30542-30582` — the blue: two dust-221 (the ±π/4 rotation),
            // then the timeLeft%10 burst's nine dust-222.
            for _ in 0..2 {
                let _jet = fountain_jet(jet_speed, 0.7853981852531433, rng);
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
                let _jet = fountain_jet(jet_speed, 1.5707963705062866, rng);
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
// W4-d16 тАФ projectile aiStyles 151-160 MINUS 156/164 (task/W4-a9 owns those
// two). The census found 157/158 already armed (projectile.rs's
// `ai_sharp_tears`/`ai_baby_bird` тАФ verify-only) and 151/152/153/154/155/
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

/// `AI_151_SuperStar` (`Projectile.cs:22138-22168`) тАФ the Super Star
/// Cannon's star (728). The тИТ10 alpha ramp to the 100 floor, the Item9
/// cue's `20 + Next(40)` re-arm, the `(|vx|+|vy|)┬╖0.005┬╖direction` spin,
/// the on-screen 1-in-6 gore gate (the `SelectRandom(16,17,17,17)` pick +
/// the NewGore kicks) and the two 1-in-8 dust gates (bodies skipped).
///
/// `screen` is the pool's per-tick view snapshot `(centre, size)` тАФ
/// `Main.screenPosition + (screenWidth, screenHeight)/2` and the size
/// (`:22152-22153`); `None` folds the hitbox test false (the C#'s
/// off-screen short-circuit spends NO `Next(6)`).
///
/// `direction` is the velocity-sign stand-in (`AutomaticallyChangesDirection`
/// is true for style 151, `:17283` тАФ the `ai_spray` precedent).
pub(crate) fn ai_super_star(
    p: &mut Projectile,
    rng: &mut dyn DropRng,
    screen: Option<(Vec2, Vec2)>,
) {
    // `:22140-22145` тАФ the alpha ramp and its floor.
    p.alpha -= 10;
    if p.alpha < 100 {
        p.alpha = 100;
    }
    // `:22146-22150` тАФ the cue re-arm (the pool decrements soundDelay
    // ahead of the AI, `:16768-16771`; the `-1` latch convention holds).
    if p.sound_delay == 0 {
        p.sound_delay = 20 + rng.next_range(0, 40);
        p.wants_sound = Some((2, 9, p.pos.x, p.pos.y, 0.0, 1.0));
    }
    // `:22151` тАФ the spin.
    let direction = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
    p.rot += (p.vel.x.abs() + p.vel.y.abs()) * 0.005 * direction;
    // `:22152-22156` тАФ the on-screen gore gate: the hitbox against the
    // screen rect grown 400 px (`Utils.CenteredRectangle`), THEN the
    // `Next(6)` (the `&&` order тАФ off-screen spends nothing).
    let on_screen = screen.is_some_and(|(centre, size)| {
        let half = (size + Vec2::splat(400.0)) * 0.5;
        let (l, t) = (centre.x - half.x, centre.y - half.y);
        let (r, b) = (centre.x + half.x, centre.y + half.y);
        // XNA `Rectangle.Intersects` тАФ the strict edge comparisons.
        p.pos.x < r && l < p.pos.x + p.w && p.pos.y < b && t < p.pos.y + p.h
    });
    if on_screen && rng.next_range(0, 6) == 0 {
        // `Utils.SelectRandom(Main.rand, 16, 17, 17, 17)` тАФ one `Next(4)`
        // (`Utils.cs:2459-2462`); the gore body is skipped, the NewGore
        // kicks stay (`Gore.cs:1327-1328`).
        let _ = rng.next_range(0, 4);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
    }
    // `:22157-22167` тАФ the two dust gates: the `Next(8)` draws stay, the
    // `NewDustDirect` bodies + their internal rolls go with the dust
    // layer (the velocity blend is deterministic тАФ no draws inside).
    for _ in 0..2 {
        if rng.next_range(0, 8) == 0 {
            // The dust spawn + its ├Ч0.25 / rotated-velocity blend тАФ SKIPPED.
        }
    }
}

/// `AI_152_SuperStarSlash` (`Projectile.cs:22230-22351`) тАФ the four
/// aiStyle-152 slashes: the Super Star Cannon's 729, the Zenith's
/// Muramasa 977 + Blade-of-Grass 976, the Moon Lord whip's proc 1045.
///
/// The alpha ladder (the 976 skips it; the 1045 ramps тИТ50 to 0, the rest
/// тИТ10 to 100), the per-kind cue latch (977/1045 Item1 once, 976 silent,
/// the rest the `20 + Next(40)` Item9 re-arm), the `ai[0]` velocity spin
/// (the 976 spins the FULL angle, the rest `ai[0] / (num4┬╖MaxUpdates)`),
/// the 977/1045 ├Ч0.96 drag + 1-in-8 centre dust, the 976's lava kill +
/// 7-frame cycle + the Remap speed/opacity envelope + the one-shot
/// 10-dust burst, and the shared `rotation = velocity.ToRotation() + num`
/// + `tileCollide = false` tail.
///
/// RNG: the cue `Next(40)` (the else arm only), the 977/1045 dust gates'
/// `Next(8)` + the taken branch's `NextVector2Circular(1,1)` pair (TWO
/// floats, `Utils.cs:1266-1269` тАФ the dust body's velocity is REWRITTEN
/// from it, so the draws are call-site state), the 976's `Next(3)` gate
/// and the one-shot burst's per-dust `NextFloat` pair (the rotated
/// velocity's angle + speed, `:22345`). The `NewDustDirect` bodies +
/// their internal rolls go with the dust layer.
pub(crate) fn ai_super_star_slash(p: &mut Projectile, world: &World, rng: &mut dyn DropRng) {
    use crate::projectile::{MOON_LORD_WHIP_PROC, ZENITH_GRASS_SLASH, ZENITH_MURAMASA_SLASH, alpha_of};
    // `:22232-22237` тАФ the rotation offset and the alpha-ladder flag.
    let mut num = std::f32::consts::FRAC_PI_2;
    let flag = p.kind != ZENITH_GRASS_SLASH;
    // `:22238-22252` тАФ the alpha ladder.
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
    // `:22253-22257` тАФ the spin divisor base.
    let num4 = if p.kind == MOON_LORD_WHIP_PROC { 24 } else { 10 };
    // `:22258-22279` тАФ the cue ladder.
    if p.sound_delay == 0 {
        if p.kind == ZENITH_MURAMASA_SLASH || p.kind == MOON_LORD_WHIP_PROC {
            p.sound_delay = -1;
            p.wants_sound = Some((2, 1, p.pos.x, p.pos.y, 0.0, 1.0));
        } else if p.kind == ZENITH_GRASS_SLASH {
            p.sound_delay = -1; // the silent latch (`:22270-22273`)
        } else {
            p.sound_delay = 20 + rng.next_range(0, 40);
            p.wants_sound = Some((2, 9, p.pos.x, p.pos.y, 0.0, 1.0));
        }
    }
    // `:22280-22291` тАФ the ai[0] spin.
    if p.ai[0] != 0.0 {
        if p.kind == ZENITH_GRASS_SLASH {
            p.vel = crate::bow_launch::rotated_by(p.vel, p.ai[0] as f64);
        } else {
            let num5 = num4 * (p.extra_updates + 1) as i32;
            p.vel = crate::bow_launch::rotated_by(p.vel, (p.ai[0] / num5 as f32) as f64);
        }
    }
    // `:22292-22313` тАФ the 977/1045 drag + centre dust (the two kinds
    // differ only in the dust TYPE тАФ 172 vs 229 тАФ the draws are the same).
    if p.kind == ZENITH_MURAMASA_SLASH || p.kind == MOON_LORD_WHIP_PROC {
        p.vel *= 0.96;
        if rng.next_range(0, 8) == 0 {
            // The dust body is skipped; its REWRITTEN velocity reads
            // `NextVector2Circular(1,1)` тАФ two floats stay on the stream.
            let _ = (rng.next_float(), rng.next_float());
        }
    }
    // `:22314-22348` тАФ the 976's machine.
    if p.kind == ZENITH_GRASS_SLASH {
        // `:22316-22320` тАФ `Collision.LavaCollision` over the box.
        if lava_collision(world, p.pos, p.w, p.h) {
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `:22321-22328` тАФ the 2-tick frame cycle over the 7-frame sheet.
        p.frame_counter += 1;
        if p.frame_counter >= 2 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 7 {
                p.frame = 0;
            }
        }
        // `:22329-22332` тАФ the age envelope: the speed Remap pair and the
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
        // `:22334-22339` тАФ the 1-in-3 trail dust (no draws inside the
        // body тАФ the velocity is `base.velocity * 0.5f`).
        if rng.next_range(0, 3) == 0 {
            // The dust spawn тАФ SKIPPED.
        }
        // `:22340-22347` тАФ the one-shot 10-dust burst: per dust the
        // rotated velocity's angle `NextFloat` + speed `NextFloat`
        // (`RotatedBy(NextFloat() * 2╧А * 0.25) * (NextFloat() * 3)`).
        if p.local_ai[1] == 0.0 {
            p.local_ai[1] = 1.0;
            for _ in 0..10 {
                let _ = rng.next_float(); // the angle
                let _ = rng.next_float(); // the speed
            }
        }
    }
    // `:22349-22350` тАФ the shared tail.
    p.rot = p.vel.y.atan2(p.vel.x) + num;
    p.tile_collide = false;
}

/// `Utils.Remap` (`Utils.cs:345-348`, clamped by default) over the port's
/// `get_lerp_value` тАФ XNA's `a + (b - a) * amount` lerp.
fn remap_clamped(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * get_lerp_value(from_min, from_max, from_value)
}

/// `AI_153_ToiletPop` (`Projectile.cs:52198-52236`) тАФ the toilet's pop
/// effect (733). The first tick (`ai[0] == 0`) spawns 6 dust + 1 gore;
/// ticks 1-10 spawn 2 dust + a 1-in-5 gore; past 10 the effect dies.
/// The counter advances AFTER the branch pick (`:52222`).
///
/// RNG: the else branch's `Next(5)` gate (`:52216`, ONLY when
/// `ai[0] <= 10` тАФ the kill returns first); every spawned gore's
/// `NextVector2Circular(1,1)` pair (two floats, `:52233` тАФ the velocity
/// argument `UnitY * -3f + NextVector2Circular(1f, 1f)` is evaluated
/// BEFORE the call, on the first tick too) + the `Next(580,583)` type
/// roll + the NewGore kicks `Next(10,31)` / `Next(-20,21)`
/// (`Gore.cs:1327-1328`). The dust bodies (`:52223-52230`) roll nothing
/// explicit.
pub(crate) fn ai_toilet_pop(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:52200-52221` тАФ the branch pick (the centre/speedY feed only the
    // skipped dust bodies).
    let (num, mut num2) = if p.ai[0] == 0.0 {
        (6, 1)
    } else {
        if p.ai[0] > 10.0 {
            // `:52211-52215` тАФ the kill (the C# `Kill()`; the port's
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
    // `:52222` тАФ the counter.
    p.ai[0] += 1.0;
    // `:52223-52230` тАФ the dust loop: `num` bodies, no explicit draws.
    let _ = num;
    // `:52231-52235` тАФ the gore loop: `num2` gores, each the circular
    // velocity pair + the type roll + the NewGore kicks.
    for _ in 0..num2 {
        let _ = (rng.next_float(), rng.next_float()); // NextVector2Circular(1,1)
        let _ = rng.next_range(580, 583); // the gore type
        let _ = rng.next_range(10, 31); // NewGore's Y kick
        let _ = rng.next_range(-20, 21); // NewGore's X kick
    }
}

/// `AI_154_VoidLens` (`Projectile.cs:52154-52196`) тАФ the Void Lens'
/// placed lens (734). The owner-gated same-type dedupe (the YOUNGER lens
/// kills the older siblings; an older sibling kills THIS lens and
/// returns), the ├Ч0.94 velocity damp, the direction pin, the 5-tick
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
/// `RotatedByRandom(2╧А)` pair (two doubles тАФ one float step each), the
/// `SelectRandom(86, 88)` pick `Next(2)`, the position's `Next(10,21)` /
/// `Next(5,10)` and the scale `NextFloat`. The dust bodies are skipped.
/// `Lighting.AddLight` (`:36`) is the light layer's skip (no draws).
pub(crate) fn ai_void_lens(p: &mut Projectile, rng: &mut dyn DropRng) {
    use crate::projectile::{alpha_of, opacity_of, proj_frames};
    // `:52177-52178` тАФ the damp and the direction pin.
    p.vel *= 0.94;
    p.direction = 1;
    // `:52179-52186` тАФ the 5-tick frame cycle over projFrames[734] = 8.
    p.frame_counter += 1;
    if p.frame_counter >= 5 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= proj_frames(p.kind) as i32 {
            p.frame = 0;
        }
    }
    // `:52187-52194` тАФ the opacity climb (the alpha view).
    if opacity_of(p.alpha) < 1.0 {
        let mut opacity = opacity_of(p.alpha) + 0.03;
        if opacity > 1.0 {
            opacity = 1.0;
        }
        p.alpha = alpha_of(opacity);
    }
    // `:52195` тЖТ `VoidLensHelper.Update` тЖТ `SpawnVoidLensDust`.
    if rng.next_range(0, 3) == 0 {
        if rng.next_range(0, 2) == 0 {
            // `:46-53` тАФ the first branch.
            let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom(2╧А)
            let _ = rng.next_range(0, 2); // SelectRandom(86, 88)
            let _ = rng.next_range(10, 21); // the position radius
            let _ = rng.next_float(); // the scale
        } else {
            // `:61-68` тАФ the second branch (the 5-10 radius).
            let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom(2╧А)
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

/// `AI_155_RemoveRope` (`Projectile.cs:52114-52129`): the startтЖТend
/// column walk (descending y), killing every active 504 MysticSnakeRope
/// cell. The net SendData is the MP skip.
fn snake_remove_rope(world: &mut World, start: (i32, i32), end: (i32, i32)) {
    let x = start.0;
    let mut y = start.1;
    while y >= end.1 {
        if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 504) {
            // The C# `WorldGen.KillTile` draws nothing (noItem: false); the
            // legacy genRand stand-in keeps the AI stream untouched (the
            // W4-g83 stream threading's documented call-site seam).
            let _ = tr_core::edit::kill_tile_legacy(world, x, y, false);
        }
        y -= 1;
    }
}

/// `AI_155_SpawnRopeIn` (`Projectile.cs:52131-52152`) тАФ the per-cell
/// conversion burst: the 3-in-4 gore gate (`Next(4) != 0` тЖТ the
/// `Next(61,64)` type + the NewGore kicks + the `NextVector2Circular
/// (1.5,0.8)` velocity pair тАФ the unit angle + the radius,
/// `Utils.cs:1261-1268`), then the 5-dust loop (each the `Next(8)`
/// gate; the bodies roll nothing explicit). The call-site draw order:
/// the type feeds `NewGoreDirect`'s argument (rolled FIRST), the kicks
/// run INSIDE `NewGore` (`Gore.cs:1327-1328`), the circular velocity is
/// the assignment AFTER the call (`:52137`).
fn snake_spawn_rope_in(rng: &mut dyn DropRng) {
    // `:52134-52141` тАФ the gore gate.
    if rng.next_range(0, 4) != 0 {
        let _ = rng.next_range(61, 64); // the gore type (`:52136`'s argument)
        let _ = rng.next_range(10, 31); // NewGore's Y kick
        let _ = rng.next_range(-20, 21); // NewGore's X kick
        // `:52137` тАФ NextVector2Circular(1.5, 0.8): the unit angle's
        // NextFloat, then the radius NextFloat (`Utils.cs:1266-1268`).
        let _ = (rng.next_float(), rng.next_float());
    }
    // `:52142-52151` тАФ the 5-dust loop.
    for _ in 0..5 {
        let _ = rng.next_range(0, 8); // the first dust's gate
        // Both dust bodies тАФ SKIPPED (no explicit draws inside).
    }
}

/// `AI_155_MysticSnakeCoil` (`Projectile.cs:51939-52112`) тАФ the Mystic
/// Snake's rope coil (754): the orphan kill (+ rope removal), the launch
/// latch (spriteDirection from the owner's centre, `velocity.Y = -5`),
/// the rise machine (the localAI[1] counter, the 15тЖТ30 Remap ease into
/// the тИТ32 launch at 60, the int truncation), the 60-tick 30-dust burst,
/// the 1-in-6 trail dust gate (the roll is the `||`'s LEFT operand тАФ it
/// runs every rise tick; `flag` suppresses only the body), the
/// тИТ5 alpha ramp, the 12-tick 4-frame cycle, the owner-gated rope
/// placement walk (the 100-cell cap, the InWorld-40 gate, the
/// solid-non-cut-non-504 blocker scan, the PlaceTile-504 column fill,
/// the snap-to-anchor + velocity zero), then the attached state's
/// per-cell rope conversion (the one-shot latch / the 1-in-80 dust walk
/// / the timeLeft-4 re-burst) and the тЙд4 death (the InWorld-20 gate, the
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

/// The fountain's per-dust jet vector — `(new Vector2(0f, 0f - num569) *
/// (0.85f + (float)Main.rand.NextDouble() * 0.2f)).RotatedBy((Main.rand.
/// NextDouble() - 0.5) * half_turn)` (`Projectile.cs:30500`/`:30520`/
/// `:30546`/`:30589`). TWO draws in the C#'s order: the speed scale first,
/// then the rotation. The rotation is `Utils.RotatedBy`'s float-cos/sin
/// spin (`Utils.cs:1675-1684`, the [`crate::bow_launch::rotated_by`]
/// port). `half_turn` is the C# float literal widened to f64: π/2
/// (1.5707963705062866) for the yellow/red/rainbow, π/4
/// (0.7853981852531433) for the blue.
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
/// M2-MOUNT-4 (the writes are App-level seams; the split-contract doc,
/// `docs/research/226`, renumbered from 97 per registry-150). W4-d8
/// RETIRE (1): the `Tile.SmoothSlope` blocker is
/// CLOSED — `tr_core::edit::smooth_slope` (`Tile.cs:792-865`) is ported.
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

/// `AI_159_PaperAirplanes` (`Projectile.cs:48695-48755`) тАФ the thrown
/// paper gliders (761/762): the one-shot launch latch (the velocity-sign
/// direction, the velocity rotation, `ai[0] = -Next(30,80)`), the wet
/// kill (the owner-gated `base.wet && owner == myPlayer` тАФ the port's
/// wet probe is the tick-head stand-in, the bobber precedent), the glide
/// counter, the wind-borne sine bob (`timeForVisualEffects % 90`), the
/// 20..69 downwind loop window (the `GetLerpValue(0,30)` eased
/// `тИТdirection┬╖2╧А┬╖0.02` bank), the 70 re-arm (`-Next(120,600)`), the
/// speed-preserving `SafeNormalize(UnitY)` steer, and the non-loop
/// float (the `Lerp(0.15,0.05,|wind|)` bob step on the `timeLeft % 40`
/// half-cycle, the ┬▒2 Y clamp, the `wind┬╖0.006` X drift clamped ┬▒6, the
/// X-sign flip re-arm `-Next(120,300)`).
///
/// RNG: the three `Main.rand.Next` draws are the glide TIMERS (`ai[0]`),
/// state-bearing тАФ the launch `Next(30,80)`, the 70 re-arm
/// `Next(120,600)`, the flip re-arm `Next(120,300)`. No dust/gore bodies.
///
/// `wind` is `Main.WindForVisuals` (= `windSpeedCurrent`, `Main.cs:2823`);
/// `visual_time` is `Main.timeForVisualEffects`; `old_vel_x` is
/// `base.oldVelocity.X` (the sub-step's start velocity, `:16766`).
/// Movement: the C# `HandleMovement` integrates after the AI; the row's
/// tileCollide true + the blocked-move chain's default (`:19849-19853`)
/// is the dispatch's `kill_step` (die on contact). `manualDirectionChange`
/// (`:7853`) suppresses the post-move auto-flip (`:17283`) тАФ the arm owns
/// `direction` alone.
pub(crate) fn ai_paper_airplanes(
    p: &mut Projectile,
    rng: &mut dyn DropRng,
    wind: f32,
    visual_time: f64,
    local_owner: u8,
    old_vel_x: f32,
) {
    // `:48697-48704` тАФ the launch latch.
    if p.ai[1] == 0.0 {
        p.direction = if p.vel.x > 0.0 { 1 } else { -1 };
        p.rot = p.vel.y.atan2(p.vel.x);
        p.ai[1] = 1.0;
        p.ai[0] = -rng.next_range(30, 80) as f32;
        p.net_update = true;
    }
    // `:48705-48708` тАФ the wet kill (the owner gate folds to the
    // simulated-slot set; the port probes wet at the tick head).
    if p.wet && p.owner == local_owner {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:48709` тАФ the glide counter.
    p.ai[0] += 1.0;
    // `:48710-48712` тАФ the wind-borne bob. `vector` is the rotation unit
    // ├Ч8; `v` adds the wind X and the sine Y (`direction ┬╖ WindForVisuals`).
    // The C# association: the double `% 90 / 90` casts to float, the
    // `(float)╧А * 2f * phase` is float math, `Math.Sin` widens to double
    // and the result casts back (`:48711`).
    let vector = Vec2::new(p.rot.cos(), p.rot.sin()) * 8.0;
    let phase = ((visual_time % 90.0) / 90.0) as f32;
    let bob = ((std::f32::consts::PI * 2.0 * phase) as f64).sin() as f32;
    let mut v = vector + Vec2::new(wind, bob * p.direction as f32 * wind);
    // `:48713-48714` тАФ the downwind flag and the loop window.
    let flag = p.direction as f32 == csign(wind) && p.vel.length() > 3.0;
    let num = p.ai[0] >= 20.0 && p.ai[0] <= 69.0;
    // `:48715-48718` тАФ the 70 re-arm.
    if p.ai[0] == 70.0 {
        p.ai[0] = -rng.next_range(120, 600) as f32;
    }
    // `:48719-48724` тАФ the loop bank (the C# `&` is non-short-circuit, but
    // both operands are pure тАФ no draws).
    let num2 = num && flag;
    if num2 {
        let lerp_value = get_lerp_value(0.0, 30.0, p.ai[0]);
        v = rotate(vector, -p.direction as f32 * std::f32::consts::TAU * 0.02 * lerp_value);
    }
    // `:48725` тАФ the speed-preserving steer (`SafeNormalize(UnitY)`).
    let speed = p.vel.length();
    p.vel = if v == Vec2::ZERO || v.is_nan() { Vec2::Y } else { v.normalize() } * speed;
    // `:48726-48752` тАФ the non-loop float.
    if !num2 {
        let num3 = 0.15 + (0.05 - 0.15) * wind.abs();
        if p.time_left % 40 < 20 {
            p.vel.y -= num3;
        } else {
            p.vel.y += num3;
        }
        p.vel.y = p.vel.y.clamp(-2.0, 2.0);
        p.vel.x = (p.vel.x + wind * 0.006).clamp(-6.0, 6.0);
        // `:48746-48751` тАФ the X-sign flip re-arm.
        if p.vel.x * old_vel_x < 0.0 {
            p.direction *= -1;
            p.ai[0] = -rng.next_range(120, 300) as f32;
            p.net_update = true;
        }
    }
    // `:48753-48754` тАФ the rotation and the sprite facing.
    p.rot = p.vel.y.atan2(p.vel.x);
    p.sprite_direction = p.direction;
}

/// `Collision.LavaCollision` (`Collision.cs:1653-1684`) тАФ the clamped
/// box scan (`[x/16тИТ1, (x+w)/16+2)` ├Ч `[y/16тИТ1, (y+h)/16+2)`, the y-range
/// capped at `maxTilesYтИТ40`): any lava cell whose liquid surface
/// (`(256тИТliquid)/32┬╖2` px down from the cell top) the box overlaps.
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
/// `Projectile.Opacity`'s setter (`Projectile.cs:546-549`):
/// `alpha = (int)Clamp((1 - value) * 255, 0, 255)` ╤В╨Р╨д the empress module's
/// helper, re-stated here (that one is private to its file).
fn set_opacity(p: &mut Projectile, opacity: f32) {
    p.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as i32;
}

/// `AI_172_GetPelletStormsCount` (`Projectile.cs:45606-45609`) ╤В╨Р╨д a constant 6.
const PELLET_STORMS_COUNT: i32 = 6;

/// `HallowBossPelletStormInfo` (`Projectile.cs:197-237`) ╤В╨Р╨д one storm's
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

    /// `IsValid` (`Projectile.cs:218-226`) ╤В╨Р╨д progress in [0, 1].
    fn is_valid(&self, bullet_index: i32) -> bool {
        let progress = self.bullet_progress(bullet_index);
        progress >= 0.0 && progress <= 1.0
    }

    /// `GetBulletPosition` (`Projectile.cs:228-231`): the centre plus the
    /// UnitX fan ray scaled by the range and the progress. The C# computes
    /// the position but the AI DISCARDS it (`:45600` ╤В╨Р╨д no assignment); the
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

/// `AI_172_GetPelletStormInfo` (`Projectile.cs:45611-45625`) ╤В╨Р╨д the storm
/// fan for `storm_index`: the start angle
/// `iтФмтХЦтХз╨Р/3 ╤В╨Ш╨в тХз╨Р/2 + iтФмтХЦтХз╨Р/5`, the 2тХз╨Р/3 per-bullet spread, 3 bullets, the
/// progress ramp over `[10i, 90 + 10i]` of `localAI[0]`, the 500-px range
/// and the 16тФЬ╨з16 bullet box.
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

/// `AI_172_HallowBossRainbowPelletStorm` (`Projectile.cs:45569-45604`) ╤В╨Р╨д
/// the Split-Shot Core (871, the style's only kind). A 91-tick grow-in
/// (`localAI[1]` 0╤В╨Ц╨в91, the scale 5╤В╨Ц╨в1 and the squared opacity ramp), then
/// the storm phase: `localAI[0]` counts to `150 + 10тФмтХЦ6 = 210` and kills,
/// the velocity zeroes and the rotation pins 0 every tick, and the six
/// three-bullet fans' positions are computed (the C# discards them ╤В╨Р╨д
/// the hitbox consumer is the damage-side slice). ZERO `Main.rand` draws
/// by construction (the grow-in and the fan maths roll nothing).
pub(crate) fn ai_pellet_storm(p: &mut Projectile) {
    // `:45571-45580` ╤В╨Р╨д the grow-in: `localAI[1] <= 90` keeps ticking.
    if p.local_ai[1] <= 90.0 {
        p.local_ai[1] += 1.0;
        p.scale = 0.5;
        set_opacity(p, 0.5);
        let lerp_value = get_lerp_value(0.0, 90.0, p.local_ai[1]);
        // `:45577` ╤В╨Р╨д MathHelper.Lerp(5, 1, t) = 5 + (1 ╤В╨Ш╨в 5)тФмтХЦt.
        p.scale = 5.0 + (1.0 - 5.0) * lerp_value;
        // `:45578` ╤В╨Р╨д Opacity = 1 ╤В╨Ш╨в (1 ╤В╨Ш╨в tтФмтЦУ) = tтФмтЦУ.
        set_opacity(p, 1.0 - (1.0 - lerp_value * lerp_value));
        return;
    }
    // `:45581-45582` ╤В╨Р╨д the storm phase pins scale 1, opacity 1.
    p.scale = 1.0;
    set_opacity(p, 1.0);
    // `:45583-45589` ╤В╨Р╨д the life cap: 150 + 10тФмтХЦcount, then Kill + return.
    let num = 150.0 + 10.0 * PELLET_STORMS_COUNT as f32;
    p.local_ai[0] += 1.0;
    if p.local_ai[0] >= num {
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:45590-45591` ╤В╨Р╨д the still core.
    p.vel = Vec2::ZERO;
    p.rot = 0.0;
    // `:45592-45603` ╤В╨Р╨д the fan walk. The C# computes each valid bullet's
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
// ---------------------------------------------------------------------------
// W4-d20 тАФ the aiStyle 191-200 census slice. The seven styles the W4-d19
// wave left UNARMED (193/194/197 already landed): 191 TrueNightsEdge,
// 192 JuminoAnimation, 195 JimsDrone, 196 Petal, 198 Flint, 199 MeteorOre,
// 200 BirdDroppings. C# sources read WHOLE (the golden rule); the RNG
// contract is the module's: every EXPLICIT `Main.rand` call-site draw stays
// in order on the `UpdateProjectiles` stream, the dust/gore BODIES and the
// `Dust.NewDust*` internal rolls are the documented skip (no dust layer).
// `Lighting.AddLight` feeds the light layer only тАФ skipped. Witnesses are
// WRITTEN NOT RUN (code-only mode).

/// `AI_200_BirdDroppings` (`Projectile.cs:39985-40017`) тАФ the bird dropping
/// (1080), a hostile falling mote. The 0.3 gravity with the 16 cap, the
/// two-dust trail (type 329) and the player-contact `Kill()`.
///
/// RNG: per tick TWO iterations, each one `Next(2)` branch pick then one
/// `NextFloat` scale (`:39996-40006`) тАФ both branches draw the scale, so
/// 2 range + 2 float draws. The `NewDust` body and its internal position
/// rolls are the dust skip; the `velocity *= 0.1` / `+= velocity` blends
/// feed only the skipped dust.
///
/// The stock `friendly`/`hostile` are both false (`SetDefaults :10078-10086`
/// sets neither), so the generic PVE/PVP passes refuse it тАФ the contact
/// `Kill()` here is the C#'s own player-rectangle test, not a damage hit
/// (the spawner passes damage 0, `NPC.cs:25322`).
pub(crate) fn ai_bird_droppings(
    p: &mut Projectile,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:39987-39991` тАФ the gravity and the 16 fall cap.
    p.vel.y += 0.3;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `:39992-40008` тАФ the two-dust trail. Each iteration draws the
    // `Next(2)` branch pick then the branch's `NextFloat` scale; the dust
    // body (type 329, the velocity blends, noGravity) is the skip.
    for _ in 0..2 {
        if rng.next_range(0, 2) == 0 {
            let _ = rng.next_float(); // scale = NextFloat * 0.5 + 0.5
        } else {
            let _ = rng.next_float(); // scale = NextFloat * 0.4 + 0.4
        }
    }
    // `:40009-40016` тАФ the player-contact kill: the `(int)` truncating
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

/// `AI_199_MeteorOre` (`Projectile.cs:40019-40041`) тАФ the meteor ore (1078),
/// a hostile+friendly+trap falling chunk. The first-tick frame roll, the
/// dust-31 trail, the `!wet` dust-6 sparkle and the two rotation terms.
///
/// RNG: first tick one `Next(3)` frame roll (`:40024`). Every tick the
/// dust-31 `scale` `Next(5)` (`:40027`) then `fadeIn` `Next(5)` (`:40028`);
/// when `!wet` the dust-6 branch's `scale` `Next(5)` (`:40033`) then the
/// position X/Y `Next(-40,41)` jitters (`:40034-40035`). The `NewDust` /
/// `NewDustDirect` bodies and their internal rolls are the dust skip.
/// `Lighting.AddLight` (`:40038`) feeds the light layer only тАФ skipped.
///
/// `base.direction` is the velocity-sign stand-in (`AutomaticallyChangesDirection`
/// true for aiStyle 199, `:17283`; the `ai_spray` precedent). The row keeps
/// stock `tileCollide = true`; the blocked-move default (`:19849-19853`)
/// kills on contact тАФ `kill_step`'s shape. No gravity in the arm: the spawn
/// velocity (`WorldGen.cs:71777`) carries the flight.
pub(crate) fn ai_meteor_ore(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:40021-40025` тАФ the first-tick latch and the 3-frame roll.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        p.frame = rng.next_range(0, 3);
    }
    // `:40026-40029` тАФ the dust-31 trail: the scale and fadeIn draws stay,
    // the body (noGravity) is the skip.
    let _ = rng.next_range(0, 5); // scale = 0.1 + Next(5) * 0.1
    let _ = rng.next_range(0, 5); // fadeIn = 1.5 + Next(5) * 0.1
    // `:40030-40037` тАФ the `!wet` dust-6 sparkle: the scale draw and the
    // two position jitters stay; the body (noGravity) is the skip.
    if !p.wet {
        let _ = rng.next_range(0, 5); // scale = 1 + Next(5) * 0.1
        let _ = rng.next_range(-40, 41); // position.X += Next(-40,41) * 0.1
        let _ = rng.next_range(-40, 41); // position.Y += Next(-40,41) * 0.1
    }
    // `:40039-40040` тАФ the two rotation terms. `direction` is the
    // velocity-sign stand-in (module doc).
    let direction = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
    p.rot += p.vel.x * 0.06;
    p.rot += direction * 0.06;
}

/// `AI_196_Petal` (`Projectile.cs:40466-40560`) тАФ the Flower Whip's petal
/// (1038) and the Petalstorm shot (1116). The first-20-tick drift (the
/// ├Ч0.95 damp), the frame cycle (8 frames at 4 ticks), the spin, the
/// nearest-chaseable homing blend and the no-target sine sway. The 1116
/// (`flag2`) swaps the light colour, the alpha remap, the acquire range
/// (400, no LOS floor), the homing constants (6/30) and the sway (├Ч0.1).
///
/// RNG: the first tick draws `Next(8)` frame (`:40483`), `NextFloat`
/// rotation (`:40485`) and `NextFloat` scale (`:40486`) тАФ in that order.
/// The `SoundID.Item17` cue (`:40481`, 1038 only, `netMode != 2`) rides
/// `wants_sound`. `Lighting.AddLight` (`:40507`/`:40513`) feeds the light
/// layer only тАФ skipped.
///
/// Both rows' movement differs: the 1038 keeps `tileCollide = false`
/// (`:9803`) тАФ the free integrate; the 1116 keeps stock `tileCollide = true`
/// (`:10413-10426` sets no override) and its blocked-move arm
/// (`:18068-18078`) reverses the blocked axis at ├Ч0.25 тАФ [`petal_storm_move`]
/// ports that bounce (the `ai_music_note` per-axis probe stand-in).
pub(crate) fn ai_petal(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
) {
    // `:40468-40475` тАФ the drift flag (read BEFORE the counter advances)
    // and the 1116 variant's colour/alpha.
    let flag = p.ai[0] < 20.0;
    let flag2 = p.kind == 1116;
    if flag2 {
        // `alpha = (int)Remap(timeLeft, 30, 0, 0, 255)`.
        p.alpha = remap(p.time_left as f32, 30.0, 0.0, 0.0, 255.0) as i32;
    }
    // `:40476-40491` тАФ the first-tick latch: the cue (1038 only), the
    // frame roll, the sprite side, the rotation and scale draws.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        if !flag2 {
            // `SoundID.Item17` at base.Center (netMode != 2 folds open).
            let cx = p.pos.x + p.w * 0.5;
            let cy = p.pos.y + p.h * 0.5;
            p.wants_sound = Some((2, 17, cx, cy, 0.0, 1.0));
        }
        p.frame = rng.next_range(0, 8);
        p.sprite_direction = if !(p.vel.x < 0.0) { 1 } else { -1 };
        p.rot = rng.next_float() * std::f32::consts::TAU;
        p.scale = 0.6 + 0.3 * rng.next_float();
        if flag2 {
            p.scale *= 0.75;
        }
    }
    // `:40492-40499` тАФ the 8-frame cycle at 4 ticks.
    p.frame_counter += 1;
    if p.frame_counter >= 4 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= 8 {
            p.frame = 0;
        }
    }
    // `:40500-40501` тАФ the spin and the age counter.
    p.rot += p.vel.x * 0.1 + p.vel.y * 0.1;
    p.ai[0] += 1.0;
    // `:40502-40510` тАФ the drift window: the ├Ч0.95 damp, then return (the
    // AddLight at ai[0] >= 10 is the light skip).
    if flag {
        p.vel *= 0.95;
        return;
    }
    // `:40515-40537` тАФ the nearest-chaseable acquire. num = 2000 (400 for
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
    // `:40538-40559` тАФ the homing blend or the sine sway.
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
/// `Projectile.cs:18068-18078`): the blocked axis reverses at ├Ч0.25 of the
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

/// `AI_192_GetJuminoFall` (`Projectile.cs:41501-41514`) тАФ the jumino's fall
/// offset: `timeSinceFall = max(ai[0] - 420, 0)`, `fall = -4*t + 0.2*t┬▓/2`
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

/// `AI_192_JuminoAnimation` (`Projectile.cs:41516-41638`) тАФ the jumino
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
/// keeps `tileCollide = false` (`:9454`) тАФ the free integrate.
pub(crate) fn ai_jumino(
    p: &mut Projectile,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    let num = 420.0 + 120.0; // 540
    let num2 = num - 150.0; // 390
    // `:41520-41530` тАФ the partial-opacity dust (one, no gate draw; the
    // NewDustDirect body skips, so the opacity read feeds nothing here).
    // `:41531-41556` тАФ the frame machine. num3 = 20 frames, num4 = 4 (60
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
    // `:41557` тАФ the sprite faces the owner (>= 0 тЖТ 1). The port's single
    // player rides slot 0; no player keeps the own centre comparison.
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let owner_x = player.map(|pl| pl.center().0).unwrap_or(center.x);
    let dir = if owner_x - center.x >= 0.0 { 1 } else { -1 };
    p.direction = dir;
    p.sprite_direction = dir;
    // `:41558-41560` тАФ the opacity ramp and the age counter. num5 = num +
    // num3*num4 + 30 (num4 is the frame==1 ? 60 : 4 read above).
    let num5 = num + (num3 * num4) as f32 + 30.0;
    let op = remap(p.ai[0], num2, num2 + 20.0, 0.0, 1.0)
        * remap(p.ai[0], num5 - 5.0, num5, 1.0, 0.5);
    p.alpha = alpha_of(op);
    p.ai[0] += 1.0;
    // `:41561-41581` тАФ the 60..130 gather window.
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
    // `:41582-41603` тАФ the <=60 spawn window.
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
    // `:41604-41623` тАФ the >=num5-20 landing window: five fadeIn draws then
    // one no-draw dust.
    if p.ai[0] >= num5 - 20.0 {
        for _ in 0..5 {
            let _ = rng.next_float(); // fadeIn = 0.6 + NextFloat * 0.3
        }
    }
    // `:41624-41631` тАФ the 1-in-6 sparkle gate.
    if rng.next_range(0, 6) == 0 && p.ai[0] >= 90.0 && p.ai[0] < num {
        // The dust body тАФ SKIPPED (no draws beyond the gate).
    }
    // `:41632-41637` тАФ the completion gift and kill.
    if p.ai[0] >= num5 {
        p.wants_gift_item = Some((5276, center.x, center.y + 12.0));
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
}

/// `AI_195_JimsDrone` (`Projectile.cs:40923-41067`) тАФ Jim's Drone (1020),
/// the RC drone the player steers with the run/up/down keys. The owner block
/// packs the input bits into `ai[0]` (the grav-flip XOR on left/right), the
/// altitude/held-item kill gates, the tilt machine (`ai[1]` = rotation), the
/// thrust/drag/cap physics, the `timeLeft = 2` keep-alive and the ground-probe
/// dust.
///
/// The port keeps `gravDir = 1` (no gravity-flip system тАФ the `mounted_top`
/// precedent), so `flag2` is false and the input XOR folds to the raw keys.
/// `Main.DroneCameraTracker.Track` (`:40955`) and
/// `CheckSectionsInCaseOwnerIsWatching` (`:41066`) are camera/section
/// bookkeeping тАФ skipped. The tracked-loop sound (`SoundID.JimsDrone`,
/// `:40986-40997`) is a continuous volume/pitch tracker, not a one-shot cue тАФ
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
/// (`:18057-18067`) reverses the blocked axis at ├Ч0.25 when |last| > 1 тАФ
/// [`drone_move`] ports that (the `ai_music_note` per-axis probe stand-in).
pub(crate) fn ai_jims_drone(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:40925-40930` тАФ the dead-owner kill flag.
    let mut flag = player.map(|pl| pl.dead).unwrap_or(false);
    // `:40931-40956` тАФ the owner-only block: the altitude kill, the held-item
    // gate and the input-bit pack. The port's single player rides slot 0.
    if p.owner == local_owner {
        // `position.Y - height <= 640` тЖТ kill (the drone flew too high).
        if p.pos.y - p.h <= 640.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `HeldItem.type != 5451` (the RC transmitter) тЖТ the kill flag.
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
        // Main.DroneCameraTracker.Track тАФ skipped (camera bookkeeping).
    }
    // `:40957-40961` тАФ the kill flag (dead owner / wrong held item).
    if flag {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:40962-40967` тАФ unpack the input bits; rotation = ai[1].
    let bits = p.ai[0] as u8;
    p.rot = p.ai[1];
    let flag3 = bits & 1 != 0; // left
    let flag4 = bits & 2 != 0; // right
    let flag5 = bits & 4 != 0; // up (thrust)
    // `:40968-40972` тАФ the steer-age counter (resets when neither left/right).
    p.local_ai[0] += 1.0;
    if !flag3 && !flag4 {
        p.local_ai[0] = 0.0;
    }
    // `:40973-40983` тАФ the throttle target: 0 grounded, 10 airborne, 100
    // thrusting; localAI[1] lerps toward it at 0.2.
    let flag6 = p.vel.y == 0.0;
    let mut num2 = if flag6 { 0.0f32 } else { 10.0 };
    if flag5 {
        num2 = 100.0;
    }
    p.local_ai[1] += (num2 - p.local_ai[1]) * 0.2;
    // `:40984-40997` тАФ the sound volume/pitch envelope feeds only the skipped
    // tracked-loop sound (num3/num4 are draw-free). localAI[2] stays 0.
    // `:40998-41020` тАФ the tilt/thrust machine.
    let mut num5 = 0.15f32;
    if flag6 {
        // Grounded: damp the tilt ├Ч0.7, drag X ├Ч0.97; thrust adds the
        // rotation-vector kick at 0.5 and lowers gravity to 0.05.
        p.rot = wrap_angle(p.rot) * 0.7;
        p.vel.x *= 0.97;
        if flag5 {
            p.vel += rotation_vector(p.rot - std::f32::consts::FRAC_PI_2) * 0.5;
            num5 = 0.05;
        }
    } else {
        // Airborne: the turn rate ramps 0.1├ЧтЖТ1├Ч over the first 10 steer ticks.
        let mut num6 = std::f32::consts::PI / 15.0;
        num6 = remap(p.local_ai[0], 0.0, 10.0, num6 * 0.1, num6);
        p.rot += (flag4 as i32 - flag3 as i32) as f32 * num6;
        if flag5 {
            let vector = rotation_vector(p.rot - std::f32::consts::FRAC_PI_2);
            p.vel.x *= 0.97;
            p.vel += vector * 0.5;
        }
    }
    // `:41021-41030` тАФ gravity, the ai[1] rotation latch, the fall drag and
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
    // `:41031-41032` тАФ AddLight skipped; the timeLeft = 2 keep-alive.
    p.time_left = 2;
    // `:41033-41065` тАФ the airborne ground-probe dust. The 3-step (16 px)
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
    // CheckSectionsInCaseOwnerIsWatching тАФ skipped (section bookkeeping).
}

/// The 1020 drone's blocked-move arm (`HandleMovement`
/// `Projectile.cs:18057-18067`): the blocked axis reverses at ├Ч0.25 of the
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

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`) тАФ the full solid block
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

/// `WorldGen.ActiveAndWalkableTile` (`WorldGen.cs:70300-70318`) тАФ active,
/// solid, not actuated (solid-top counts).
fn flint_walkable(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind))
}

/// `FindBlockSurface` (`Projectile.cs:52361-52387`) тАФ walk up out of solid
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

/// `FindBlockCeiling` (`Projectile.cs:52389-52415`) тАФ the gravity-flip
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

/// `AI_198_Flint_EmitSpikes` (`Projectile.cs:52317-52359`) тАФ the charged
/// release's ground-spike wave: up to 13 spikes (num3 = 0..12) stepping 2
/// tiles along `direction`, each on the first walkable surface/ceiling tile.
/// The port keeps `gravDir = 1` (flag false тЖТ the surface walk).
///
/// RNG: per spawned spike one `NextFloat` (the ai1 scale, `:52346`) then one
/// `NextFloatDirection` (the velocity rotation, `:52352`). The
/// `ParticleOrchestrator` request (`:52322`) is a particle-system spawn тАФ
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
    // gravDir = 1 тЖТ flag false (the surface walk).
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

/// `AI_198_Flint` (`Projectile.cs:52238-52315`) тАФ the Dead Cells flint
/// (1040), the channel weapon. While channelling (`ai[0] == 0`) it holds the
/// player's facing, winds the dummy item time and rolls the charge dust; on
/// release (`!channel`) it spawns the 1043 slash (damage ├Ч1.78) and тАФ if the
/// charge reached 30 ticks тАФ the ground-spike wave, then enters the
/// follow-through (`ai[0] == 1`) that counts down to `useAnimation` and kills.
///
/// The port keeps `gravDir = 1` (the aim's `ReverseGravitySupport` folds to
/// the raw mouse, the spike walk takes the surface branch). The player-side
/// writes (`ChangeDir`, `heldProj`, `SetDummyItemTime`, `itemAnimationMax`)
/// ride the [`FlintTick`] return тЖТ the pool's `GolfEffect::PlayerWrites` seam.
/// The `itemLocation.Y += Next(2)` jitter (`:52309`) keeps its draw on the
/// stream; the draw-side item location is unmodelled (the thrust placement
/// recomputes it each frame тАФ a documented skip).
///
/// RNG (charge phase, `ai[0] == 0`): one `Next(2)` (the itemLocation jitter,
/// `:52309`) then one `Next(10)` gate (`:52310`); on the taken branch the
/// dust's `NextVector2Circular(12,12)` pair + `NextFloat` velocity + `NextFloat`
/// scale (`:52312`). The release's spike wave rides [`flint_emit_spikes`].
/// The `DeadCellsFlintCharge` cue (`:52305`) is a tracked sound тАФ skipped (no
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
    // `:52240-52245` тАФ the inactive-owner kill.
    if !player.active {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return tick;
    }
    // `:52246-52260` тАФ the release: !channel while still charging.
    if !player.channel && p.ai[0] == 0.0 {
        let charged = p.ai[1] >= 30.0;
        p.ai[0] = 1.0;
        p.ai[1] = 0.0;
        p.net_update = true;
        if p.owner == local_owner {
            // The 1043 slash: damage ├Ч1.78, velocity (direction, 0),
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
    // `:52261-52279` тАФ the follow-through: count to useAnimation, then kill.
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
    // `:52280-52314` тАФ the charge hold.
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
        // The 30-tick charge cue is a tracked sound тАФ skipped (the state
        // still gates the spike wave above).
        tick.item_animation_max = Some(use_animation);
        tick.dummy_item_time = Some(num5);
        // `itemLocation.Y += Next(2)` тАФ the draw stays; the location is
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

/// `AI_191_TrueNightsEdge` (`Projectile.cs:41640-41808`) тАФ the True Night's
/// Edge spin slash (973) and the Terra Blade's second shot (985). Both ride
/// the `localAI[0]` age / `ai[1]` lifetime envelope: the opacity ramp, the
/// `num4`-tick kill, then the per-type choreography.
///
/// 973: the spin (`rotation += ai[0] * TAU * (4 + Opacity*4) / 90`), the
/// scale remap, the owner-anchored centre (`RotatedRelativePoint(MountedCenter)
/// - velocity + velocity * num6┬▓ * 77`), the damage-zeroing LOS probe and the
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
/// `lastVelocity * 0.01` and sets `localAI[1] = 1` тАФ [`terra_blade_move`].
pub(crate) fn ai_true_nights_edge(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:41642-41653` тАФ the envelope constants; the 985 swaps them.
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
    // `:41654-41657` тАФ the 973 first-tick Item8 cue.
    if p.local_ai[0] == 0.0 && p.kind == 973 {
        p.wants_sound = Some((2, 8, p.pos.x, p.pos.y, 0.0, 1.0));
    }
    // `:41658-41666` тАФ the age counter and the two fast-forward arms.
    p.local_ai[0] += 1.0;
    if is_terra && p.local_ai[1] == 1.0 {
        p.local_ai[0] += 2.0;
    }
    if p.kind == 973 && p.damage == 0 && p.local_ai[0] < num3 + (num4 - num3) * 0.5 {
        p.local_ai[0] += 6.0;
    }
    // `:41667` тАФ the opacity envelope.
    let opacity = remap(p.local_ai[0], 0.0, p.ai[1], 0.0, 1.0)
        * remap(p.local_ai[0], num3, num4, 1.0, 0.0);
    p.alpha = alpha_of(opacity);
    // `:41668-41673` тАФ the kill at num4.
    if p.local_ai[0] >= num4 {
        p.local_ai[1] = 1.0;
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:41674-41677` тАФ the owner anchor and the facing.
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let from_value = p.local_ai[0] / p.ai[1];
    let mut num6 = remap(p.local_ai[0], p.ai[1] * 0.4, num4, 0.0, 1.0);
    p.direction = p.ai[0] as i32;
    p.sprite_direction = p.ai[0] as i32;
    // `:41678-41725` тАФ the 973 spin-slash branch.
    if p.kind == 973 {
        let num7 = 3.0f32;
        if p.damage != 0 && p.local_ai[0] >= num5 + num7 {
            p.damage = 0;
        }
        if p.damage != 0 {
            // The 5-ray LOS probe (`:41687-41703`): num8 = 80, the rays at
            // ┬▒0.25┬╖(╧А/4) around the velocity angle; a clear ray keeps damage.
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
        // `:41705-41710` тАФ the localAI[1] counter, the eased centre, the spin
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
        // `:41711-41724` тАФ the dust angle draw, the 1-in-5 gate and the
        // opacity-dust loop (the bodies skip).
        let _f = p.rot + (rng.next_float() * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2 * 0.7;
        if rng.next_range(0, 5) == 0 {
            // The dust-14 body тАФ SKIPPED.
        }
        let mut i = 0.0f32;
        while i < 3.0 * opacity {
            let _ = rng.next_float(); // the 75/27 type pick
            i += 1.0;
        }
    }
    // `:41726-41729` тАФ only the 985 continues past here.
    if !is_terra {
        return;
    }
    // `:41730-41739` тАФ the ownerHitCheck window, the damage-zero, the eased
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
    // `:41740-41769` тАФ the >8 speed drag ladder.
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
    // `:41770-41786` тАФ the dust angle draw and the three-gate dust loop.
    let _num13 = p.rot + (rng.next_float() * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2 * 0.9;
    for _ in 0..3 {
        if rng.next_float() < opacity2 + 0.1 {
            // The dust-107 body (colour lerp + velocity blends) тАФ SKIPPED.
        }
    }
    // `:41787-41791` тАФ the damage==0 fast-forward + drag.
    if p.damage == 0 {
        p.local_ai[0] += 3.0;
        p.vel *= 0.76;
    }
    // `:41792-41807` тАФ the early-out / retract dust window.
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
// Witnesses (S1-D-4 code-only round тАФ WRITTEN NOT RUN).

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
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0 тАФ the
    /// census proves count AND order.
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
    /// `projectile_ai001_a` fixture shape тАФ field-by-field on `Default`,
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

    /// The four rows keep their census ┬з3c styles (the data half of the
    /// task тАФ the rows already exist, the arms are what this module adds).
    #[test]
    fn s1_d_4_unarmed_the_four_rows_carry_their_styles() {
        assert_eq!(proj_data(145).unwrap().ai_style, 31);
        for kind in 715..=718u16 {
            assert_eq!(proj_data(kind).unwrap().ai_style, 147, "kind {kind}");
        }
        assert_eq!(proj_data(932).unwrap().ai_style, 181);
        assert_eq!(proj_data(85).unwrap().ai_style, 193);
    }

    /// W4-g70 lane 8 — the starfall spawner (720) and the natural star (12)
    /// rows: 720 is aiStyle 148 (16×16, alpha 255 invisible, penetrate −1,
    /// tileCollide false, `Projectile.cs:7617-7625`); 12 is aiStyle 5
    /// (18×18, alpha 50, light 1, trap, `:924-938`).
    #[test]
    fn w4_g70_the_starfall_rows() {
        let spawner = proj_data(crate::projectile::FALLING_STAR_SPAWNER).unwrap();
        assert_eq!(spawner.ai_style, 148);
        assert_eq!((spawner.w, spawner.h), (16.0, 16.0));
        assert_eq!(spawner.alpha, 255);
        assert_eq!(spawner.penetrate, -1);
        assert!(!spawner.tile_collide);
        let star = proj_data(crate::projectile::NATURAL_FALLING_STAR).unwrap();
        assert_eq!(star.ai_style, 5);
        assert_eq!((star.w, star.h), (18.0, 18.0));
        assert_eq!(star.alpha, 50);
        assert_eq!(star.light, 1.0);
        assert!(star.trap);
        assert_eq!(star.penetrate, -1);
    }

    /// W4-g70 lane 8 — `AI_148_StarSpawner`'s day kill
    /// (`Projectile.cs:53096-53100`): a non-remix day kills the spawner;
    /// the remix world keeps it alive.
    #[test]
    fn w4_g70_star_spawner_day_kill() {
        let mut p = proj(crate::projectile::FALLING_STAR_SPAWNER);
        let mut fall = None;
        // Day, non-remix: killed, no conversion.
        assert!(!ai_star_spawner(&mut p, true, false, 1, true, 0, &mut fall));
        assert!(!p.active);
        // Night: the latch fires the StarFall request (player above surface).
        let mut p = proj(crate::projectile::FALLING_STAR_SPAWNER);
        assert!(!ai_star_spawner(&mut p, false, false, 1, true, 0, &mut fall));
        assert!(p.active);
        assert_eq!(fall, Some(p.pos.x));
        assert_eq!(p.local_ai[0], 1.0);
        // The latch is one-shot: a second call does not re-request.
        fall = None;
        ai_star_spawner(&mut p, false, false, 1, true, 0, &mut fall);
        assert_eq!(fall, None);
    }

    /// W4-g70 lane 8 — the 180-tick conversion gate
    /// (`Projectile.cs:53110-53113`): `ai[0] += dayRate` per call; the
    /// conversion fires only at `ai[0] >= 180` for the local owner.
    #[test]
    fn w4_g70_star_spawner_conversion_gate() {
        let mut p = proj(crate::projectile::FALLING_STAR_SPAWNER);
        let mut fall = None;
        // 179 ticks at dayRate 1: not yet.
        for _ in 0..179 {
            assert!(!ai_star_spawner(&mut p, false, false, 1, false, 0, &mut fall));
        }
        assert_eq!(p.ai[0], 179.0);
        // The 180th tick converts.
        assert!(ai_star_spawner(&mut p, false, false, 1, false, 0, &mut fall));
        // A non-local owner never converts.
        let mut q = proj(crate::projectile::FALLING_STAR_SPAWNER);
        q.owner = 1;
        q.ai[0] = 200.0;
        assert!(!ai_star_spawner(&mut q, false, false, 1, false, 0, &mut fall));
    }

    /// W4-g70 lane 8 — the homing nudge (`Projectile.cs:53114-53125`):
    /// `vel.x *= 0.35` then the sign flips toward the target centre.
    #[test]
    fn w4_g70_star_spawner_homing() {
        let mut p = proj(crate::projectile::FALLING_STAR_SPAWNER);
        p.vel.x = 10.0;
        p.ai[1] = 0.0; // > -1: the homing arm runs
        // Target to the RIGHT of the spawner's centre → positive X.
        let mut target = p.pos.x + p.w * 0.5 + 100.0;
        star_spawner_convert(&mut p, target);
        assert!((p.vel.x - 3.5).abs() < 1e-6);
        // Target to the LEFT → negative X.
        p.vel.x = 10.0;
        target = p.pos.x + p.w * 0.5 - 100.0;
        star_spawner_convert(&mut p, target);
        assert!((p.vel.x + 3.5).abs() < 1e-6);
        // ai[1] == -1: no homing (the velocity keeps its value).
        p.vel.x = 10.0;
        p.ai[1] = -1.0;
        target = p.pos.x + 1000.0;
        star_spawner_convert(&mut p, target);
        assert_eq!(p.vel.x, 10.0);
    }

    /// Style 31 (`:26826-26839`, `:26883-26885`): the owner-gated cleanse
    /// converts the corrupt grass under the spray, the life caps at 133,
    /// the counter advances and the spin reads the velocity sign. ZERO
    /// rand draws тАФ the arm takes no rng at all (by construction).
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
        ai_spray(&mut p, &mut w, 0, &mut tr_core::UnifiedRandom::new(0));
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
        ai_spray(&mut p2, &mut w2, 0, &mut tr_core::UnifiedRandom::new(0));
        assert_eq!(w2.tile(100, 50).unwrap().kind, 23, "the owner gate");
        // The solution flag widens the disc to size 3 (`:26828-26832`) тАФ
        // a cell at Manhattan 5 converts only under the flag.
        let mut w3 = world();
        put(&mut w3, 103, 52, 23); // |3| + |2| = 5 < 6, outside size 2
        let mut p3 = proj(145);
        p3.ai[1] = 1.0;
        ai_spray(&mut p3, &mut w3, 0, &mut tr_core::UnifiedRandom::new(0));
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
        ai_spray(&mut p, &mut w, 0, &mut tr_core::UnifiedRandom::new(0));
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
        ai_spray(&mut p, &mut w, 0, &mut tr_core::UnifiedRandom::new(0));
        assert_eq!(w.tile(100, 50).unwrap().kind, 199, "the grass crimson");
        assert_eq!(w.tile(101, 50).unwrap().kind, 203, "the stone → crimstone");
        assert_eq!(w.tile(100, 51).unwrap().kind, 234, "the sand → crimsand");
        // The solution flag widens the disc to size 3 for the evil types
        // too (`:26828-26832` — the size is conversion-independent).
        let mut w = world();
        put(&mut w, 103, 52, 2); // Manhattan 5, outside size 2
        let mut p = proj(147);
        p.ai[1] = 1.0;
        ai_spray(&mut p, &mut w, 0, &mut tr_core::UnifiedRandom::new(0));
        assert_eq!(w.tile(103, 52).unwrap().kind, 23, "the size-3 evil disc");
        // The non-owner refuses (`:26826`).
        let mut w = world();
        put(&mut w, 100, 50, 2);
        let mut p = proj(149);
        p.owner = 1;
        ai_spray(&mut p, &mut w, 0, &mut tr_core::UnifiedRandom::new(0));
        assert_eq!(w.tile(100, 50).unwrap().kind, 2, "the owner gate");
        // The per-kind cells now all land (the union merge carries f25's
        // `convert_sized_hallow`/`convert_sized_spray` and g42's
        // `convert_sized_mushroom`): the 146 type 2 hallows the grass
        // 2 → 109 (`WorldGen.cs:55176-55260`), the 148 type 3 mushroom arm
        // leaves a plain grass cell alone (`:55345-55358` — only the jungle
        // 60 → 70 and the thorns), the 1015 type 5 desertifies 2 → 53
        // (`:55383-55399`), the 1016 type 6 snows 2 → 147 (`:55429-55436`)
        // and the 1017 type 7 purify arm leaves it (`:55497-55517` — the
        // grass is already 2). The machine runs for every kind.
        for (kind, expected) in [
            (146u16, 109u16),
            (148, 2),
            // The 1015 type 5 desertify turns a grass cell over AIR into the
            // hardened 397 (`WorldGen.cs:86562-86583` — an inactive cell
            // below makes hardened sand; `:55383-55399`).
            (1015, 397),
            (1016, 147),
            (1017, 2),
        ] {
            let mut w = world();
            put(&mut w, 100, 50, 2);
            let mut p = proj(kind);
            ai_spray(&mut p, &mut w, 0, &mut tr_core::UnifiedRandom::new(0));
            assert_eq!(
                w.tile(100, 50).unwrap().kind,
                expected,
                "kind {kind} conversion (`:26801-26824`)"
            );
            assert_eq!(p.ai[0], 1.0, "kind {kind} counter");
        }
    }

    /// Style 193 (`:41339-41348`, `:41372`, `:41391`): the counter, the
    /// 60-tick 0.95 decay, the 72-tick kill (no return тАФ the gates still
    /// run), and the two gates' stream positions. CensusRng's floats are
    /// 0 (< every chance) so both taken branches roll their explicit
    /// call-site draws: the flame branch 2 floats + Next(4), the smoke
    /// branch 2 + 2 + 2 floats.
    #[test]
    fn s1_d_4_unarmed_the_flames_decay_and_die_at_72() {
        let mut p = proj(85);
        p.vel = Vec2::new(8.0, 0.0);
        let mut rng = census();
        // Ticks 1..49: the flame gate alone тАФ 1 gate + 2 circular + 1
        // Next(4) per tick.
        for _ in 0..49 {
            ai_flamethrower(&mut p, &mut rng, 0);
        }
        assert_eq!(rng.floats, 49 * 3, "the flame gate + its circular pair");
        assert_eq!(rng.ranges, vec![(0, 4); 49], "the noGravity picks");
        assert_eq!(p.vel.x, 8.0, "no decay before 60");
        // Ticks 50..59: BOTH gates (the smoke window opens at 50, the
        // flame window closes at 50 тАФ `localAI[0] < 50` fails at 50).
        // At tick 50 the flame gate's `localAI[0] < num3` is FALSE тАФ only
        // the smoke gate rolls: 1 + 6 floats, no ranges.
        ai_flamethrower(&mut p, &mut rng, 0);
        assert_eq!(rng.floats, 49 * 3 + 7, "tick 50: smoke only");
        assert_eq!(rng.ranges.len(), 49);
        // Ticks 51..60: the smoke gate alone (7 floats each). The decay
        // (`localAI[0] >= 60`) lands on tick 60 тАФ one 0.95 step.
        for _ in 0..10 {
            ai_flamethrower(&mut p, &mut rng, 0);
        }
        assert_eq!(rng.floats, 49 * 3 + 11 * 7);
        assert_eq!(p.vel.x, 8.0f32 * 0.95, "the first decay at tick 60 (`:41345-41348`)");
        // Tick 61: a second decay; the smoke gate keeps rolling. The C#'s
        // f32 accumulation — repeated * 0.95, not a pow.
        ai_flamethrower(&mut p, &mut rng, 0);
        assert_eq!(p.vel.x, 8.0f32 * 0.95 * 0.95, "the second decay at tick 61");
        assert_eq!(rng.floats, 49 * 3 + 12 * 7);
        // Ticks 62..72: the kill lands at 72 but the arm runs on.
        for _ in 0..11 {
            ai_flamethrower(&mut p, &mut rng, 0);
        }
        assert!(!p.active, "the 72-tick kill (`:41341-41344`)");
        assert_eq!(p.time_left, 0);
        assert_eq!(rng.floats, 49 * 3 + 23 * 7, "the smoke gate post-kill");
        assert_eq!(p.local_ai[0], 72.0);
    }

    /// Style 193's Foxparks flag (`:41349-41356`): ai[0]==1 flips the
    /// windows тАФ the flame gate runs to 60, the smoke gate never opens.
    #[test]
    fn s1_d_4_unarmed_the_foxparks_flag_flips_the_windows() {
        let mut p = proj(85);
        p.ai[0] = 1.0;
        let mut rng = census();
        for _ in 0..72 {
            ai_flamethrower(&mut p, &mut rng, 0);
        }
        // Ticks 1..59: gate + 2 floats + Next(4); ticks 60..72: no gate.
        assert_eq!(rng.floats, 59 * 3, "one window, 59 taken ticks");
        assert_eq!(rng.ranges.len(), 59, "no smoke window");
    }

    /// S1-E-21h d75 part 4 — the 1106 Foxparks sound gate
    /// (`Projectile.cs:41366-41370`): at `soundDelay == 0 && whoAmI % 3
    /// == 0` the arm latches `sound_delay = -1` and queues the named-row
    /// marker `(2, -34)` at `base.Center`, ONCE per life; the gate draws
    /// NOTHING extra on the pass stream (the style play's pitch rides the
    /// drain). The row lives on d18's pushed tip — the fixture builds the
    /// slot by hand (the owed-arm convention).
    #[test]
    fn s1_e_21h_d75_p4_the_foxparks_flame_latches_once_at_center() {
        let mut p = proj(85);
        p.kind = 1106;
        p.w = 6.0;
        p.h = 6.0;
        p.pos = Vec2::new(1600.0, 800.0);
        let mut rng = census();
        // whoAmI 1: 1 % 3 != 0 — no cue, soundDelay untouched.
        ai_flamethrower(&mut p, &mut rng, 1);
        assert_eq!(p.wants_sound, None, "the %3 gate refuses slot 1");
        assert_eq!(p.sound_delay, 0);
        // whoAmI 3: the latch fires at Center, the −1 keeps it down.
        ai_flamethrower(&mut p, &mut rng, 3);
        assert_eq!(
            p.wants_sound,
            Some((2, -34, 1603.0, 803.0, 0.0, 1.0)),
            "the named-row marker at Center"
        );
        assert_eq!(p.sound_delay, -1, "the one-cue-per-life latch");
        // Later ticks: the −1 latch never re-fires (the pool's decrement
        // is positive-only, `:16768-16771`).
        p.wants_sound = None;
        for _ in 0..20 {
            ai_flamethrower(&mut p, &mut rng, 3);
        }
        assert_eq!(p.wants_sound, None, "no second cue");
        // The 85 Flames twin: the kind gate keeps it silent.
        let mut q = proj(85);
        let mut rng2 = census();
        ai_flamethrower(&mut q, &mut rng2, 0);
        assert_eq!(q.wants_sound, None, "the 85 plays nothing here");
        assert_eq!(q.sound_delay, 0);
    }

    /// Style 181 (`:43180-43183`): the alpha ramp truncates the XNA lerp
    /// (255 тЖТ 0 over 10 ticks), the rotation reads the velocity, and the
    /// Next(6) gate rolls every tick (CensusRng returns min = 0 тЖТ the
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
        // Tick 40: the first burst тАФ the case-0 gore row.
        assert_eq!(
            rng.ranges,
            vec![
                (0, 100), (0, 100), (61, 64), (10, 31), (-20, 21), (-10, 11), (-10, 11)
            ],
            "the gore row (`:53151-53154` + Gore.cs:1327-1328)"
        );
        // 21 fall steps (ticks 20..40) тАФ the C#'s own f32 accumulation.
        let mut vy = 0.0f32;
        for _ in 0..21 {
            vy += 0.12;
        }
        assert_eq!(p.vel.y, vy, "21 fall steps (ticks 20..40)");
    }

    /// Style 147 state 1 (`:52910-52951`): the tick-10 тИТ10 kick, the 0.25
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
        // Tick 10: тИТ10 then +0.25 тЖТ тИТ4.75; the ring rolls its floats.
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
    /// Manhattan acquire committing ai[1] = index + 1, and the (v┬╖7+d)/8
    /// blend at the latched speed. The NPC sits 100 px right of the
    /// rocket's centre; the integer-half centres coincide with the float
    /// halves for the even boxes here.
    #[test]
    fn s1_d_4_unarmed_the_celeb2_homes_on_the_acquired_target() {
        let w = world();
        let mut p = proj(717);
        p.ai[0] = 3.0;
        p.vel = Vec2::new(6.0, 0.0);
        p.local_ai[0] = 24.0; // the next tick is 25 тАФ the scan window
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
        // run this tick тАФ ai[1] was still 0 at `:52973`, before the
        // acquire тАФ so vel.Y picked up 0.15 then blended; only X is
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
    /// lands on 46: x = cos(46┬╖╧А/45) is NEGATIVE, so the step along
    /// rotate((1,0), 1.5707964) тЙИ (0,1) moves тИТY.
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
    /// tile inside the radius rolls amount + (amount тИТ 1) floats per
    /// layer (4 layers) plus the Next(3) gore gate per layer. CensusRng's
    /// min = 0 makes `Next(3) != 0` FALSE тАФ the census pins the refuse
    /// path: 4 layers ├Ч (3 + 2 floats + 1 range). The tile is dirt
    /// (amount 3) at the rocket's centre.
    #[test]
    fn s1_d_4_unarmed_the_ground_explosion_walks_the_layers() {
        let mut w = world();
        let mut p = proj(715);
        p.ai[0] = 1.0;
        // The resized death box (128├Ч128) centred on tile (100, 50).
        p.pos = Vec2::new(100.0 * 16.0 - 64.0, 50.0 * 16.0 - 64.0);
        p.w = 128.0;
        p.h = 128.0;
        put(&mut w, 100, 50, 0); // dirt, exposed (the neighbours are air)
        let mut rng = census();
        create_ground_explosion(&p, &w, &mut rng, 40.0, 20, 100, 2, 10, 2);
        // Radius = 128/2 + 100 = 164: the centre tile is inside; the
        // fluffed box spans ┬▒(4 + 20) tiles but only (100, 50) is solid.
        // 4 layers ├Ч (3 + 2) floats + 4 Next(3) gates (all refuse at min).
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
    /// ten `Next(1, 3)` steps (CensusRng min = 1 тЖТ counts 1,1,1,0,0,0,0,
    /// тИТ1,тИТ2,тИТ2 after the ring marks), three floats per spawned dust.
    #[test]
    fn s1_d_4_unarmed_the_fairy_death_census() {
        let mut rng = census();
        fairy_queen_death_draws(&mut rng);
        assert_eq!(rng.ranges, vec![(1, 3); 10], "the ten trail steps");
        assert_eq!(rng.floats, 1 + 3 * 3, "the lead + 3 dusts ├Ч 3 floats");
    }

    /// The pool death hook: a rocket killed by the tile contact resizes
    /// to the 128 box, deals its death Damage through the resized box and
    /// rolls the state-0 explosion census (7 ranges).
    #[test]
    fn s1_d_4_unarmed_the_death_hook_resizes_and_bursts() {
        let mut w = world();
        // A stone wall one step ahead of the rocket тАФ the kill_step dies.
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
    /// pool update advances each rowed kind's own counter тАФ 145's ai[0],
    /// 715's localAI[0] (├Ч3 sub-steps), 932's ai[0], 85's localAI[0]
    /// (├Ч3 sub-steps).
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
        // flies straight up тАФ its X never moves).
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
        // (0, -1.7) spun by -0.7853981852531433 rad (the C# π/2 literal
        // widened, × (0 - 0.5)). cos/sin are the f32 casts of the f64.
        let jet = fountain_jet(2.0, 1.5707963705062866, &mut rng);
        let a = -0.7853981852531433f64;
        let (c, s) = (a.cos() as f32, a.sin() as f32);
        // RotatedBy((0, -1.7)): x = 0*c - (-1.7)*s = 1.7*s (negative),
        // y = 0*s + (-1.7)*c = -1.7*c.
        assert!((jet.x - 1.7 * s).abs() < 1e-6, "the spun X (`:30500`)");
        assert!((jet.y - (-1.7 * c)).abs() < 1e-6, "the spun Y (`:30500`)");
        assert_eq!(rng.doubles, 2, "the speed scale + the rotation draw");
        // The blue's π/4 literal: the angle halves again (-0.39269909262657166).
        let mut rng2 = census();
        let jet_b = fountain_jet(2.0, 0.7853981852531433, &mut rng2);
        let ab = -0.39269909262657166f64;
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
        let jet_s = fountain_jet(1.0, 1.5707963705062866, &mut rng4);
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
        assert_eq!(p.wants_sound, Some((2, 54, 300.0, 200.0, 0.0, 1.0)),
            "the Item54 pop (`:72921`, SoundID.cs:1437)");
        assert_eq!(rng.doubles, 10, "the 410's ten hue draws (`:72923/72938`)");
        assert!(rng.ranges.is_empty(), "the Next(45,91) else is dead (`:72930`)");
        assert!(rng.floats == 0, "no float draws in the pop");
        // The 405 (the d7 Flairon lane's twin arm): 20 hue doubles.
        let mut q = proj(FLAIRON_BUBBLE);
        q.pos = Vec2::new(400.0, 100.0);
        let mut rng2 = census();
        bubble_pop_kill(&mut q, &mut rng2);
        assert_eq!(q.wants_sound, Some((2, 54, 400.0, 100.0, 0.0, 1.0)),
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

    // ---- W4-f50 deviation 2: the 608 blast burst BODIES -------------------

    /// A recording `Main.rand` stand-in over a real seeded stream: every
    /// draw is logged as `(tag, a, b)` — tag 0 = `Next(min,max)`, 1 =
    /// `NextFloat`, 2 = `NextDouble`. The bodies fn and the census oracle
    /// run on the SAME seed, so the parity witness compares the logs.
    struct BlastRecord {
        inner: tr_core::UnifiedRandom,
        log: Vec<(u8, i32, i32)>,
    }
    impl BlastRecord {
        fn new(seed: i32) -> Self {
            Self { inner: tr_core::UnifiedRandom::new(seed), log: Vec::new() }
        }
    }
    impl DropRng for BlastRecord {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.log.push((0, min, max));
            self.inner.next_range(min, max)
        }
        fn next_float(&mut self) -> f32 {
            self.log.push((1, 0, 0));
            self.inner.next_float()
        }
        fn next_double(&mut self) -> f64 {
            self.log.push((2, 0, 0));
            self.inner.next_double()
        }
    }

    fn blast_ctx() -> tr_core::particles::SpawnCtx {
        tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(4800.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The bodies spawn the C#'s 84 dust + 2 gore, in type order, with the
    /// post-arms (`Projectile.cs:72183-72224`): 4× dust 31, 60× dust 6 (the
    /// 30 pairs), 10× dust 6, 10× dust 31, 2× gore 61-63. The pair's second
    /// dust carries `fadeIn = 2.5`; every burst dust but the first four is
    /// `noGravity`.
    #[test]
    fn w4_f50_the_608_blast_bodies_spawn_the_census_counts_and_post_arms() {
        let w = world();
        let ctx = blast_ctx();
        let mut dust = tr_core::particles::DustPool::new();
        let mut gore = tr_core::gore::GorePool::new();
        let mut rng = BlastRecord::new(42);
        let center = Vec2::new(1600.0, 800.0);
        solar_flare_blast_bodies(
            &mut rng,
            &mut dust,
            &ctx,
            &w,
            &mut gore,
            center - Vec2::new(80.0, 80.0),
            160.0,
            160.0,
            center,
            Vec2::new(3.0, -4.0),
        );
        let active: Vec<&tr_core::particles::Dust> =
            dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 84, "4 + 60 + 10 + 10 dust (:72183-72216)");
        // The type order: 4× 31, 60× 6 (the 30 pairs), 10× 6, 10× 31.
        let types: Vec<i32> = active.iter().map(|d| d.dust_type).collect();
        assert_eq!(&types[..4], &[31, 31, 31, 31]);
        assert!(types[4..64].iter().all(|&t| t == 6), "the 30 pairs are dust 6");
        assert_eq!(&types[64..74], &[6; 10]);
        assert_eq!(&types[74..84], &[31; 10]);
        // The first four (alpha-100 dust 31) keep gravity; the rest noGravity.
        assert!(active[..4].iter().all(|d| !d.no_gravity));
        assert!(active[4..].iter().all(|d| d.no_gravity));
        // The pair's second dust (odd index within 4..64) carries fadeIn 2.5.
        for i in (5..64).step_by(2) {
            assert_eq!(active[i].fade_in, 2.5, "pair second fadeIn (:72199)");
        }
        // 2 gore, types in 61..=63.
        let gores: Vec<&tr_core::gore::Gore> =
            gore.slots().iter().filter(|g| g.active).collect();
        assert_eq!(gores.len(), 2, "2× gore (:72217)");
        assert!(gores.iter().all(|g| (61..=63).contains(&g.gore_type)));
    }

    /// Stream parity: the bodies fn consumes EXACTLY the census oracle's
    /// explicit draws (in order) PLUS the `NewDust`/`NewGore` internal
    /// draws. The oracle's log is an ordered subsequence of the bodies'
    /// log; the unmatched draws are the 578 internal rolls (14 dust-31 × 6
    /// + 70 dust-6 × 7 defaults/damps + 2 gore × 2 kicks). Total 826 =
    /// 248 + 578.
    #[test]
    fn w4_f50_the_608_blast_bodies_keep_the_census_stream_order() {
        let w = world();
        let ctx = blast_ctx();
        let mut dust = tr_core::particles::DustPool::new();
        let mut gore = tr_core::gore::GorePool::new();
        let center = Vec2::new(1600.0, 800.0);
        let mut bodies_rng = BlastRecord::new(42);
        solar_flare_blast_bodies(
            &mut bodies_rng,
            &mut dust,
            &ctx,
            &w,
            &mut gore,
            center - Vec2::new(80.0, 80.0),
            160.0,
            160.0,
            center,
            Vec2::new(3.0, -4.0),
        );
        let mut census_rng = BlastRecord::new(42);
        solar_flare_blast_draws(&mut census_rng);
        let a = &bodies_rng.log;
        let b = &census_rng.log;
        assert_eq!(b.len(), 248, "the explicit census draws");
        assert_eq!(a.len(), 826, "248 explicit + 578 internal");
        // Greedy ordered-subsequence match: every census draw appears in the
        // bodies log in order. The dust section of the census is pure
        // doubles (the rings), so the internal dust ranges never mis-match;
        // the gore ranges align at the tail.
        let mut bi = 0;
        for draw in a {
            if bi < b.len() && *draw == b[bi] {
                bi += 1;
            }
        }
        assert_eq!(bi, b.len(), "the census is an ordered subsequence of the bodies");
        assert_eq!(a.len() - b.len(), 578, "the unmatched draws are the internal rolls");
    }

    // ---- W4-d20 aiStyle 200 (BirdDroppings) тАФ WRITTEN NOT RUN ----

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

    // ---- W4-d20 aiStyle 199 (MeteorOre) тАФ WRITTEN NOT RUN ----

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

    // ---- W4-d20 aiStyle 196 (Petal) тАФ WRITTEN NOT RUN ----

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
    /// rotation, NextFloat scale тАФ in order. The 1038 queues the Item17 cue;
    /// the 1116 does not and scales ├Ч0.75.
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
        // The 1116: no cue, the ├Ч0.75 scale.
        let mut q = proj(1116);
        q.vel = Vec2::new(-2.0, 0.0);
        let mut rng2 = census();
        ai_petal(&mut q, &w, &[], &mut rng2);
        assert_eq!(q.wants_sound, None, "the 1116 plays no cue (`:40479`)");
        assert_eq!(q.sprite_direction, -1, "vel.x < 0");
        // scale = (0.6 + 0.3 * 0) * 0.75 = 0.45 (census NextFloat = 0).
        assert!((q.scale - 0.45).abs() < 1e-6, "the ├Ч0.75 (`:40489`)");
    }

    /// The drift window (`:40502-40510`): while ai[0] < 20 the velocity
    /// damps ├Ч0.95 and the arm returns before the homing scan.
    #[test]
    fn w4_d20_the_petal_drift_damps_and_returns() {
        let w = world();
        let mut p = proj(1038);
        p.local_ai[0] = 1.0; // past the first tick
        p.ai[0] = 5.0;
        p.vel = Vec2::new(4.0, 2.0);
        ai_petal(&mut p, &w, &[], &mut census());
        assert!((p.vel.x - 4.0 * 0.95).abs() < 1e-6, "the ├Ч0.95 damp");
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
        let mut n = npc_at(0.0, 0.0);
        // The acquire reads the NPC CENTRE (`npc_center` ≈ `Entity.Center`)
        // and the blend's `dir` is the unit vector to it (`:40544`), so put
        // the NPC centre 100 px right of the SHOT's centre (pos + half the
        // 18×18 box, not pos itself) to make `dir` exactly (1, 0).
        let pc = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let (nw, nh) = n.size();
        n.x = pc.x + 100.0 - nw * 0.5;
        n.y = pc.y - nh * 0.5;
        n.hp = 40;
        n.max_hp = 40;
        ai_petal(&mut p, &w, &[n], &mut census());
        // dir = (1, 0) * 24; vel = (0 * 15 + (24, 0)) / 16 = (1.5, 0).
        assert!((p.vel.x - 1.5).abs() < 1e-5, "the homing blend");
        assert!(p.vel.y.abs() < 1e-6);
    }

    /// The 1116 bounce (`:18068-18078`): the blocked axis reverses at ├Ч0.25.
    #[test]
    fn w4_d20_the_petalstorm_bounce_reverses_at_quarter() {
        let mut w = world();
        // A solid wall at tile x=102 (spans 1632..1648). The shot's right
        // edge at 1618 + vel 20 = 1638 overlaps it тАФ the X move is blocked.
        put(&mut w, 102, 50, 1);
        let mut p = proj(1116);
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(20.0, 0.0);
        petal_storm_move(&mut p, &w);
        assert!((p.vel.x - (-20.0 * 0.25)).abs() < 1e-6, "the ├Ч0.25 reversal");
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

    // ---- W4-d20 aiStyle 192 (JuminoAnimation) тАФ WRITTEN NOT RUN ----

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
    /// fall = -4t + 0.1t┬▓ capped at 24.
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
        assert_eq!(p.sprite_direction, 1, "no owner тЖТ own centre, >= 0");
    }

    /// The completion gift (`:41632-41637`): at ai[0] >= num5 the arm queues
    /// the 5276 gift at Center + (0, 12) and kills. num5 = 540 + 20*60 + 30
    /// = 1770 when frame == 1 (num4 = 60).
    #[test]
    fn w4_d20_the_jumino_completion_queues_the_gift() {
        let mut p = proj(995);
        p.pos = Vec2::new(1000.0, 500.0);
        p.frame = 1; // num4 = 60 тЖТ num5 = 1770
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

    // ---- W4-d20 aiStyle 195 (JimsDrone) тАФ WRITTEN NOT RUN ----

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
    /// ├Ч0.7 and drags X ├Ч0.97; gravity is 0.15 (no thrust).
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

    /// The drone bounce (`:18057-18067`): the blocked axis reverses at ├Ч0.25
    /// only when |last| > 1; a slow axis keeps its velocity.
    #[test]
    fn w4_d20_the_drone_bounce_speed_gate() {
        let mut w = world();
        put(&mut w, 102, 50, 1); // wall at x 1632..1648
        let mut p = proj(1020);
        p.pos = Vec2::new(1600.0, 800.0);
        // The 12-wide box at 1600 + 24 spans 1624..1636, overlapping the
        // wall (20 would land flush at 1632 and never intersect).
        p.vel = Vec2::new(24.0, 0.0); // fast тЖТ reverses
        drone_move(&mut p, &w);
        assert!((p.vel.x - (-24.0 * 0.25)).abs() < 1e-6, "the ├Ч0.25 reversal");
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

    // ---- W4-d20 aiStyle 198 (Flint) тАФ WRITTEN NOT RUN ----

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
        // so the gate is taken тЖТ the dust's 2+1+1 floats).
        assert_eq!(rng.ranges, vec![(0, 2), (0, 10)], "the charge census");
        assert_eq!(rng.floats, 4, "Circular pair + velocity + scale");
    }

    /// The release (`:52246-52260`): !channel while charging spawns the 1043
    /// slash (damage ├Ч1.78) and enters the follow-through. A charge < 30
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
        p.ai[1] = 10.0; // < 30 тЖТ no spikes
        let pl = flint_player(false); // released
        let tick = ai_flint(&mut p, &w, pl, Vec2::new(1700.0, 800.0), 0, &mut census());
        assert_eq!(p.ai[0], 1.0, "the follow-through state");
        let slash = tick.wants_slash.expect("the slash spawn");
        assert_eq!(slash.4, 178, "damage ├Ч1.78 (`:52254`)");
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
        // MountedCenter at (1600, 800) тЖТ start + 30 = (1630, 800) тЖТ tile
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
        p.ai[1] = 19.0; // advances to 20 = useAnimation тЖТ kill
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

    /// W4-d20 T5-canary тАФ the 1043 flint slash is the 190/198 cross-lane
    /// merge canary: d20's 198 arm spawns it (`:52254`, ai0=dir┬╖gravDir,
    /// ai1=6, ai2=1) but its aiStyle-190 AI is d19's. This witness pins the
    /// SPAWN payload end-to-end through the dart-drain (the ai2=1 scale
    /// survives тАФ `:42222` `scale *= ai[2]` would collapse to 0 if the
    /// drain dropped it). A lossy merge that drops the 198 spawner leaves
    /// no 1043; one that drops the 1043 row leaves the spawn inert. Both
    /// must land for the slash to fly.
    #[test]
    fn w4_d20_t5_the_flint_slash_canary_spawns_with_the_scale() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut pl = crate::player::Player::spawn(&w);
        pl.channel = false; // released тЖТ the slash spawns
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
        assert_eq!(slash.ai[2], 1.0, "ai2 = 1 тАФ the scale survives the drain");
        // The row is the d19 twin (16x16 aiStyle 190 pen 2) тАФ the merge
        // keep-one pin: both branches carry the identical tuple.
        let row = proj_data(crate::projectile::DEAD_CELLS_FLINT_SLASH).unwrap();
        assert_eq!(row.ai_style, 190);
        assert_eq!((row.w, row.h), (16.0, 16.0));
        assert_eq!(row.penetrate, 2);
    }

    // ---- W4-d20 aiStyle 191 (TrueNightsEdge) тАФ WRITTEN NOT RUN ----

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
        // A live slash carries damage; at damage == 0 the C#'s fast-forward
        // (`:41663-41666`) adds 6 to localAI[0] below the median.
        p.damage = 50;
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
        p.local_ai[0] = 94.0; // advances to 95 = num4 тЖТ kill
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
        // Past the window: localAI[0] = 7 тЖТ clears.
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
    /// 0.65) the damage zeroes (num3=30, num4=55 тЖТ 46.25).
    #[test]
    fn w4_d20_the_terra_blade_damage_zeroes_late() {
        let w = world();
        let mut p = proj(985);
        p.ai[1] = 30.0;
        p.ai[2] = 1.0;
        p.damage = 50;
        p.vel = Vec2::new(3.0, 0.0);
        p.local_ai[0] = 46.0; // advances to 47 >= 46.25 тЖТ damage 0
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
#[cfg(test)]
mod w4_d16_151_tests {
    //! W4-d16 ╤В╨Р╨д aiStyle 151 (`AI_151_SuperStar`, `Projectile.cs:22138-22168`,
    //! the dispatch `:36567-36570`). WRITTEN NOT RUN (code-only mode). The
    //! witnesses mirror the C# on the port's seams: the row, the alpha
    //! ramp + floor, the cue re-arm's `Next(40)`, the spin, the on-screen
    //! gore gate's `&&` short-circuit (off-screen spends NO `Next(6)`) and
    //! the two dust gates' `Next(8)` pair ╤В╨Р╨д the RNG census pins count AND
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
            min // every gate takes its branch ╤В╨Р╨д the census sees the kicks
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
        assert_eq!(d.extra_updates, 0, "no override ╤В╨Р╨д the stock 0");
    }

    /// The alpha ramp (`:22140-22145`): ╤В╨Ш╨в10/tick, floored at 100. The
    /// cue re-arm (`:22146-22150`): `20 + Next(40)` ╤В╨Р╨д the census min 0
    /// gives 20; the Item9 cue rides `wants_sound`. The spin (`:22151`):
    /// (|3|+|4|)тФмтХЦ0.005тФмтХЦ1 = 0.035.
    #[test]
    fn w4_d16_151_the_ramp_cue_and_spin() {
        let mut p = star();
        p.vel = Vec2::new(3.0, 4.0);
        let mut rng = census();
        // Off-screen (`None`): the gore gate short-circuits BEFORE its
        // Next(6) ╤В╨Р╨д the census sees only the cue + the two dust gates.
        ai_super_star(&mut p, &mut rng, None);
        assert_eq!(p.alpha, 245, "255 ╤В╨Ш╨в 10");
        assert_eq!(p.sound_delay, 20, "20 + Next(40)=0");
        assert_eq!(p.wants_sound, Some((2, 9, 1600.0, 800.0, 0.0, 1.0)), "Item9");
        assert!((p.rot - 0.035).abs() < 1e-6, "the spin: {}", p.rot);
        assert_eq!(rng.ranges, vec![(0, 40), (0, 8), (0, 8)], "no Next(6) off-screen");
        assert_eq!(rng.floats, 0);
        // The floor: alpha 105 ╤В╨Ц╨в 95 ╤В╨Ц╨в 100.
        p.alpha = 105;
        p.sound_delay = -1; // the latch ╤В╨Р╨д no re-arm, no draw
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
    /// grown screen rect ╤В╨Ц╨в `Next(6)`; on the taken branch the
    /// `SelectRandom` pick `Next(4)` + the NewGore kicks `Next(10,31)` /
    /// `Next(-20,21)` (`Gore.cs:1327-1328`). The dust gates follow.
    #[test]
    fn w4_d16_151_the_on_screen_gore_census() {
        let mut p = star();
        p.sound_delay = -1;
        let mut rng = census();
        // The screen centre on the star, any size ╤В╨Р╨д the box intersects.
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

    /// The dispatch wiring: one pool update runs the arm (alpha 255╤В╨Ц╨в245,
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
    //! W4-d16 ╤В╨Р╨д aiStyle 152 (`AI_152_SuperStarSlash`,
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

    /// The 729: the ╤В╨Ш╨в10 alpha ramp to 100, the Item9 `20 + Next(40)`
    /// re-arm, the ai[0]/(10тФмтХЦ3) spin, the тХз╨Р/2 rotation offset, the
    /// tileCollide pin. NO other draws (the 729 has no dust branch).
    #[test]
    fn w4_d16_152_the_cannon_slash_ramp_spin_and_cue() {
        let mut p = slash(SUPER_STAR_SLASH);
        p.alpha = 255; // the spawn seam's stock 0 ╤В╨Р╨д the C# row leaves 0;
        // the witness drives the ramp from a visible value.
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 30.0; // one full divisor turn: 30/(10*3) = 1 rad
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.alpha, 245);
        assert_eq!(p.sound_delay, 20, "20 + Next(40)=0");
        assert_eq!(p.wants_sound, Some((2, 9, 1600.0, 800.0, 0.0, 1.0)));
        assert_eq!(rng.ranges, vec![(0, 40)], "the cue draw only");
        // The spin: RotatedBy(1 rad) of (10,0) ╤В╨Р╨д cos/sin in double.
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

    /// The 977: the Item1 one-cue latch, the тФЬ╨з0.96 drag, the 1-in-8 dust
    /// gate + the taken branch's NextVector2Circular pair (two floats).
    #[test]
    fn w4_d16_152_the_muramasa_drag_and_dust_census() {
        let mut p = slash(ZENITH_MURAMASA_SLASH);
        p.vel = Vec2::new(10.0, 0.0);
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.sound_delay, -1, "the one-cue latch");
        assert_eq!(p.wants_sound, Some((2, 1, 1600.0, 800.0, 0.0, 1.0)), "Item1");
        assert!((p.vel.x - 9.6).abs() < 1e-5, "the тФЬ╨з0.96 drag");
        assert_eq!(rng.ranges, vec![(0, 8)]);
        assert_eq!(rng.floats, 2, "NextVector2Circular(1,1) ╤В╨Р╨д the rewritten velocity");
        // The latch holds: no second cue, the gate still rolls.
        let mut rng2 = census();
        ai_super_star_slash(&mut p, &w, &mut rng2);
        assert_eq!(rng2.ranges, vec![(0, 8)]);
        assert_eq!(rng2.floats, 2);
    }

    /// The 1045: the ╤В╨Ш╨в50 ramp to 0, the 24-base divisor (24тФмтХЦ2 = 48), the
    /// Item1 latch + the same dust census as the 977.
    #[test]
    fn w4_d16_152_the_whip_proc_ramp_and_divisor() {
        let mut p = slash(MOON_LORD_WHIP_PROC);
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 48.0; // 48/48 = 1 rad
        let mut rng = census();
        let w = World::new("t".into(), 300, 200);
        ai_super_star_slash(&mut p, &w, &mut rng);
        assert_eq!(p.alpha, 205, "255 ╤В╨Ш╨в 50");
        assert_eq!(p.sound_delay, -1);
        assert_eq!(rng.ranges, vec![(0, 8)], "no Next(40) ╤В╨Р╨д the Item1 arm");
        assert_eq!(rng.floats, 2);
        let c = 1.0f64.cos() as f32;
        // The drag runs AFTER the spin: 10тФмтХЦcos(1)тФмтХЦ0.96.
        assert!((p.vel.x - 10.0 * c * 0.96).abs() < 1e-4);
        // The floor at 0.
        p.alpha = 30;
        ai_super_star_slash(&mut p, &w, &mut census());
        assert_eq!(p.alpha, 0);
    }

    /// The 976: the silent latch (NO cue draw), the full-angle spin, the
    /// frame cycle, the Remap speed envelope (ai[1] = 0, timeLeft 60 ╤В╨Ц╨в
    /// fromValue 0 ╤В╨Ц╨в num6 = 0 ╤В╨Ц╨в speed 16), the opacity fade-in start
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
        assert!((p.vel.length() - 16.0).abs() < 1e-4, "4 + 12тФмтХЦ(1╤В╨Ш╨в0)");
        assert_eq!(p.frame, 0, "frameCounter 1 < 2");
        assert_eq!(p.frame_counter, 1);
        assert_eq!(p.local_ai[1], 1.0, "the burst latch");
        assert_eq!(rng.ranges, vec![(0, 3)], "the trail gate only ╤В╨Р╨д no cue draw");
        assert_eq!(rng.floats, 20, "10 dust тФЬ╨з (angle + speed)");
        assert!((p.rot - p.vel.y.atan2(p.vel.x)).abs() < 1e-6, "num = 0");
        // Second tick: the frame advances, the burst does NOT re-fire.
        let mut rng2 = census();
        ai_super_star_slash(&mut p, &w, &mut rng2);
        assert_eq!(p.frame, 1);
        assert_eq!(p.frame_counter, 0);
        assert_eq!(rng2.floats, 0, "the one-shot latch");
        // The speed at fromValue 30 (timeLeft 30): num6 = Remap(30,0,20╤В╨Ц╨в
        // clamped 1)тФмтХЦRemap(30,0,60,1╤В╨Ц╨в0 = 0.5) = 0.5 ╤В╨Ц╨в speed 10.
        p.time_left = 30;
        ai_super_star_slash(&mut p, &w, &mut census());
        assert!((p.vel.length() - 10.0).abs() < 1e-4);
    }

    /// The 976 lava kill (`:22316-22320`): a full lava cell under the box
    /// kills BEFORE the frame/envelope work ╤В╨Р╨д zero draws.
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
    /// ╤В╨Р╨д extraUpdates 2) and the free integrate moves the slot.
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
        assert_eq!(p.time_left, 27, "30 ╤В╨Ш╨в 3 sub-steps");
        assert!((p.pos.x - 1630.0).abs() < 1e-3, "3 тФЬ╨з 10 px");
        // The cue draws ONCE (the re-arm holds across the sub-steps).
        assert_eq!(rng.ranges, vec![(0, 40)]);
    }
}

#[cfg(test)]
mod w4_d16_153_tests {
    //! W4-d16 ╤В╨Р╨д aiStyle 153 (`AI_153_ToiletPop`, `Projectile.cs:52198-
    //! 52236`, the dispatch `:36575-36578`). WRITTEN NOT RUN (code-only
    //! mode). The row, the first-tick burst census (no Next(5) gate ╤В╨Р╨д
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

    /// The first tick (`:52204-52208`, `:52231-52235`): NO Next(5) gate ╤В╨Р╨д
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
    /// ╤В╨Р╨д zero draws, the counter does NOT advance.
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
    //! W4-d16 ╤В╨Р╨д aiStyle 154 (`AI_154_VoidLens`, `Projectile.cs:52154-
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

    /// The body (`:52177-52194`): the тФЬ╨з0.94 damp, direction 1, the 5-tick
    /// frame cycle, the +0.03 opacity climb (alpha 255 ╤В╨Ц╨в 247: opacity
    /// 0.03 ╤В╨Ц╨в alpha (1╤В╨Ш╨в0.03)тФмтХЦ255 = 247.35 ╤В╨Ц╨в 247). The dust census: the
    /// Next(3) gate + the Next(2) pick (census min 0 ╤В╨Ц╨в the FIRST branch)
    /// + the RotatedByRandom pair + the SelectRandom Next(2) + the
    /// Next(10,21) radius + the scale float.
    #[test]
    fn w4_d16_154_the_body_and_dust_census() {
        let mut p = lens();
        p.vel = Vec2::new(10.0, 0.0);
        let mut rng = census();
        ai_void_lens(&mut p, &mut rng);
        assert!((p.vel.x - 9.4).abs() < 1e-5, "the тФЬ╨з0.94 damp");
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
        p.alpha = 3; // opacity ╤В╨Щ╨Ш 0.988 + 0.03 > 1 ╤В╨Ц╨в clamp
        ai_void_lens(&mut p, &mut census());
        assert_eq!(p.alpha, 0);
    }

    /// The dedupe (`:52157-52176`): two lenses, one owner ╤В╨Р╨д the YOUNGER
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
    //! W4-d16 ╤В╨Р╨д aiStyle 155 (`AI_155_MysticSnakeCoil`,
    //! `Projectile.cs:51939-52112` + the rope helpers `:52114-52152`, the
    //! dispatch `:36583-36586`). WRITTEN NOT RUN (code-only mode). The
    //! row, the launch latch + rise truncation, the 60-tick burst census
    //! (4 floats тФЬ╨з 30), the trail gate's `||` short-circuit, the rope
    //! placement + attach snap, the attached conversion census, the
    //! timeLeft-4 re-burst + the ╤В╨Щ╨┤4 removal death, the orphan kill.
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
        // the anchor's cell centre ╤В╨Ш╨в 16 (row 50 ╤В╨Ц╨в 792). Centre
        // (1608, 800) ╤В╨Р╨д tile (100, 50); the rope-scan rows are
        // `position.Y / 16` = 49 (NOT the centre row).
        p.pos = Vec2::new(1600.0, 792.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.ai[0] = 792.0; // the anchor y ╤В╨Р╨д scan row 49
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
    /// owner-centre comparison, ai[1] = 1, velocity.Y = ╤В╨Ш╨в5 ╤В╨Ц╨в the rise
    /// machine's first tick truncates it to ╤В╨Ш╨в3 (localAI[1] = 1 < 15 ╤В╨Ц╨в
    /// num4 = 0 ╤В╨Ц╨в ╤В╨Ш╨в3тФмтХЦ1 = ╤В╨Ш╨в3). The trail gate rolls Next(6) (census min 0
    /// ╤В╨Ц╨в taken), the alpha ramps ╤В╨Ш╨в5, the frame counter advances.
    #[test]
    fn w4_d16_155_the_launch_and_first_rise_tick() {
        let mut w = world();
        let mut p = coil();
        let mut rng = census();
        // The owner to the RIGHT of the coil ╤В╨Ц╨в spriteDirection ╤В╨Ш╨в1.
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(p.sprite_direction, -1);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.vel.y, -3.0, "the int truncation of ╤В╨Ш╨в3тФмтХЦ(1╤В╨Ш╨в0)");
        assert_eq!(p.alpha, 250);
        assert_eq!(p.frame_counter, 1);
        assert_eq!(rng.ranges, vec![(0, 6)], "the trail gate only");
        assert_eq!(rng.floats, 0);
        // The owner to the LEFT ╤В╨Ц╨в spriteDirection 1 (the `!(>)` form).
        let mut p2 = coil();
        ai_mystic_snake_coil(&mut p2, &mut world(), Some(Vec2::new(1000.0, 808.0)), true, 0, &mut census());
        assert_eq!(p2.sprite_direction, 1);
    }

    /// The 60-tick burst (`:51965-51982`): 30 dust тФЬ╨з 4 floats (the X
    /// jitter, the angle pair, the ALWAYS-taken Y scale ╤В╨Р╨д the rotated
    /// (0,╤В╨Ш╨в1) vector's Y is ╤В╨Ш╨в1 for every draw since the angle is тФЬ╨з0f).
    /// The trail gate's Next(6) is the `||`'s LEFT operand ╤В╨Р╨д it rolls
    /// EVERY rise tick (num5 = 1 only skips the body, never the roll).
    #[test]
    fn w4_d16_155_the_sixty_tick_burst_census() {
        let mut w = world();
        let mut p = coil();
        p.ai[1] = 1.0;
        p.local_ai[1] = 59.0; // this tick ╤В╨Ц╨в 60
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(p.local_ai[1], 60.0);
        assert_eq!(p.vel.y, -32.0, "╤В╨Ш╨в3тФмтХЦ0 + ╤В╨Ш╨в32тФмтХЦ1");
        assert_eq!(rng.floats, 120, "30 тФЬ╨з 4");
        assert_eq!(rng.ranges, vec![(0, 6)], "the Next(6) is the ||'s left operand ╤В╨Р╨д always rolled");
    }

    /// The rope placement (`:52002-52046`): the coil has risen to row 43
    /// (pos.y 688), the anchor row 49 ╤В╨Р╨д a solid dirt cell at row 45 trips
    /// the blocker scan (`:52017-52024`), so the column fills rows 49..44
    /// (`> point3.Y` ╤В╨Р╨д row 43 stays empty; the dirt at 45 is neither
    /// TileEmpty nor tileCut, so it is skipped), the life re-arms to
    /// 1800, ai[1] = 2, the Top snaps to (100тФмтХЦ16+8, 43тФмтХЦ16+16) and the
    /// velocity zeroes. The attached half runs the SAME tick: the
    /// one-shot conversion over the 5 rope cells (44, 46..49).
    #[test]
    fn w4_d16_155_the_placement_attach_and_conversion() {
        let mut w = world();
        put(&mut w, 100, 45, 1); // the solid blocker (dirt)
        let mut p = coil();
        p.pos = Vec2::new(1600.0, 688.0); // risen ╤В╨Р╨д centre row 43
        p.ai[1] = 1.0;
        p.local_ai[1] = 5.0;
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert_eq!(p.ai[1], 2.0, "the attach");
        assert_eq!(p.time_left, 1800);
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::ZERO);
        // Top = (1608, 704) ╤В╨Ц╨в pos = (1600, 704) for the 16x16 box.
        assert_eq!(p.pos, Vec2::new(1600.0, 704.0));
        for y in [44, 46, 47, 48, 49] {
            assert_eq!(w.tile(100, y).unwrap().kind, 504, "the column cell {y}");
        }
        assert_eq!(w.tile(100, 45).unwrap().kind, 1, "the blocker untouched");
        assert!(!w.tile(100, 43).unwrap().active(), "the `> point3.Y` bound");
        assert_eq!(p.alpha, 0, "the attached head");
        assert_eq!(p.local_ai[0], 1.0, "the one-shot latch");
        // The conversion census: the trail gate's Next(6) first, then 5
        // rope cells тФЬ╨з (Next(4) + 5тФЬ╨зNext(8)) ╤В╨Р╨д census min 0 ╤В╨Ц╨в the gore
        // gate FAILS, no gore draws.
        assert_eq!(rng.ranges.len(), 1 + 5 * 6, "the trail gate + 5 cells тФЬ╨з 6");
        assert_eq!(rng.ranges[0], (0, 6));
        assert!(rng.ranges[1..].iter().all(|&(a, b)| (a, b) == (0, 4) || (a, b) == (0, 8)));
        assert_eq!(rng.floats, 0, "the gore gate failed ╤В╨Р╨д no circular pair");
    }

    /// The gore-taking conversion census: a stream whose Next(4) returns
    /// 1 (gate taken) ╤В╨Р╨д per cell the type roll + the kicks + the circular
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
        // One rope cell at the coil's own row ╤В╨Р╨д the start == end scan.
        put(&mut w, 100, 49, 504);
        let mut p = coil();
        p.ai[0] = 792.0; // the anchor ON the coil row ╤В╨Ц╨в a 1-cell scan
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

    /// The timeLeft-4 re-burst + the ╤В╨Щ╨┤4 death (`:52087-52111`): the
    /// conversion walks AGAIN, then the owner-gated rope removal kills
    /// every 504 cell and the coil dies.
    #[test]
    fn w4_d16_155_the_death_reburst_and_removal() {
        let mut w = world();
        put(&mut w, 100, 49, 504);
        let mut p = coil();
        p.ai[0] = 792.0; // the anchor ON the coil row ╤В╨Ц╨в a 1-cell scan
        p.ai[1] = 2.0;
        p.local_ai[0] = 1.0; // the per-tick dust walk (not the one-shot)
        p.time_left = 4;
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, Some(Vec2::new(2000.0, 808.0)), true, 0, &mut rng);
        assert!(!p.active, "the ╤В╨Щ╨┤4 death");
        assert_eq!(p.time_left, 0);
        assert!(!w.tile(100, 49).unwrap().active(), "the rope removed");
        // The per-tick walk's Next(80) (census min 0 ╤В╨Ц╨в the dust body,
        // skipped) + the re-burst's per-cell row.
        assert_eq!(rng.ranges[0], (0, 80), "the per-tick dust gate");
        assert_eq!(rng.ranges[1], (0, 4), "the re-burst's gore gate");
    }

    /// The orphan kill (`:51941-51949`): no active owner ╤В╨Ц╨в the rope
    /// column dies with the coil, ZERO draws.
    #[test]
    fn w4_d16_155_the_orphan_kill_removes_the_rope() {
        let mut w = world();
        put(&mut w, 100, 49, 504);
        put(&mut w, 100, 50, 504);
        let mut p = coil();
        p.ai[0] = 800.0; // the anchor BELOW the coil ╤В╨Р╨д rows 50..49 scan
        let mut rng = census();
        ai_mystic_snake_coil(&mut p, &mut w, None, false, 0, &mut rng);
        assert!(!p.active);
        assert!(!w.tile(100, 49).unwrap().active());
        assert!(!w.tile(100, 50).unwrap().active());
        assert_eq!(rng.ranges, Vec::new());
        assert_eq!(rng.floats, 0);
    }

    /// The dispatch wiring: one pool update runs the launch tick (no
    /// player ╤В╨Ц╨в the orphan kill; with the owner snapshot the rise runs).
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
    /// a resolved owner runs the placement walk against the REAL world ╤В╨Р╨д
    /// the blocker at row 45 trips the scan, the column fills rows
    /// 49..44 with 504, the coil attaches (ai[1] = 2, life 1800), and the
    /// tile writes bump `world.revision` (the `WorldFrameKey` input ╤В╨Р╨д
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
        // ai[1] = 1 (the rise state) ╤В╨Р╨д the first update's blocker scan
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
        // The death window: timeLeft 4 ╤В╨Ц╨в the re-burst + the owner-gated
        // removal kill every 504 cell through the same dispatch.
        pool.slots_mut()[slot].time_left = 4;
        let rev1 = w.revision;
        pool.update(
            &mut w, &mut npcs, Some((2000.0, 808.0)), None, Some(&mut pl),
            YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
        );
        assert!(!pool.slots()[slot].active, "the ╤В╨Щ╨┤4 death");
        for y in [44, 46, 47, 48, 49] {
            assert!(!w.tile(100, y).unwrap().active(), "the LIVE removal {y}");
        }
        assert!(w.revision > rev1, "the removal bumps the key too");
    }
}

#[cfg(test)]
mod w4_d16_159_tests {
    //! W4-d16 ╤В╨Р╨д aiStyle 159 (`AI_159_PaperAirplanes`,
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
    /// the rotation, ai[1] = 1, ai[0] = -Next(30,80) (census min 30 ╤В╨Ц╨в
    /// -30, then the counter++ ╤В╨Ц╨в -29). Calm wind: the float arm runs ╤В╨Р╨д
    /// the bob step ╤В╨Ш╨в0.15 (timeLeft 3600 % 40 = 0 < 20), the Y clamp,
    /// the X drift 0. The steer keeps the speed 5 along +X.
    #[test]
    fn w4_d16_159_the_launch_and_first_glide_tick() {
        let mut p = plane();
        let mut rng = census();
        ai_paper_airplanes(&mut p, &mut rng, 0.0, 0.0, 0, 5.0);
        assert_eq!(p.direction, 1);
        // The launch latch writes `rotation = velocity.ToRotation()` = 0, but
        // the tail overwrites it from the POST-bob velocity each tick
        // (`:48751`): atan2(-0.15, 5).
        assert!((p.rot - (-0.15f32).atan2(5.0)).abs() < 1e-6, "the tail rotation: {}", p.rot);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.ai[0], -29.0, "-Next(30,80) + the counter++");
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::new(5.0, -0.15), "the steer keeps 5, the bob ╤В╨Ш╨в0.15");
        assert_eq!(p.sprite_direction, 1);
        assert_eq!(rng.ranges, vec![(30, 80)], "the launch timer only");
        assert_eq!(rng.floats, 0);
        // The leftward throw ╤В╨Ц╨в direction ╤В╨Ш╨в1 (the `> 0` form).
        let mut p2 = plane();
        p2.vel = Vec2::new(-5.0, 0.0);
        ai_paper_airplanes(&mut p2, &mut census(), 0.0, 0.0, 0, -5.0);
        assert_eq!(p2.direction, -1);
    }

    /// The 70 re-arm (`:48715-48718`): ai[0] = 69 ╤В╨Ц╨в the counter++ hits
    /// 70 ╤В╨Ц╨в ai[0] = -Next(120,600) (census min 120 ╤В╨Ц╨в -120). The loop
    /// window `num` was computed BEFORE the re-arm (69 ╤В╨Щ╨┤ 69 ╤В╨Ц╨в true), but
    /// calm wind folds `flag` false ╤В╨Р╨д the float arm runs.
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
    /// speed 5 > 3) and ai[0] = 30 (the window's top) ╤В╨Ц╨в the eased bank
    /// `╤В╨Ш╨в1тФмтХЦ2тХз╨РтФмтХЦ0.02тФмтХЦ1` rotates the 8-unit vector; the steer normalizes it
    /// back to speed 5. The float arm is SKIPPED (num2 true) ╤В╨Р╨д no bob,
    /// no drift, no flip draw.
    #[test]
    fn w4_d16_159_the_loop_bank_steers() {
        let mut p = plane();
        p.ai[1] = 1.0;
        p.ai[0] = 30.0; // the counter++ ╤В╨Ц╨в 31, still in 20..69
        p.direction = 1;
        let mut rng = census();
        ai_paper_airplanes(&mut p, &mut rng, 0.5, 0.0, 0, 5.0);
        // ai[0] 31 ╤В╨Ц╨в GetLerpValue(0,30,31) clamps to 1 ╤В╨Ц╨в the full bank.
        let banked = rotate(Vec2::new(8.0, 0.0), -std::f32::consts::TAU * 0.02);
        let want = banked.normalize() * 5.0;
        assert!((p.vel - want).length() < 1e-4, "the banked steer: {:?} vs {:?}", p.vel, want);
        assert_eq!(rng.ranges, Vec::new(), "the loop arm draws nothing");
        // The rotation follows the banked velocity.
        assert_eq!(p.rot, p.vel.y.atan2(p.vel.x));
    }

    /// The X-flip re-arm (`:48746-48751`): a leftward drift crosses zero
    /// against the old velocity's sign ╤В╨Ц╨в direction flips, ai[0] =
    /// -Next(120,300). Wind ╤В╨Ш╨в0.5, the plane flying left at 5: the steer
    /// keeps ╤В╨Ш╨в5, the drift ╤В╨Ш╨в0.003 keeps it negative ╤В╨Р╨д no flip. Force the
    /// crossing with a tiny leftward velocity and a rightward wind.
    #[test]
    fn w4_d16_159_the_x_flip_rearm() {
        let mut p = plane();
        p.ai[1] = 1.0;
        p.ai[0] = 0.0;
        p.vel = Vec2::new(-0.001, 0.0); // barely leftward
        p.direction = -1;
        let mut rng = census();
        // Wind +0.5: the bob is downwind-flagged false (direction ╤В╨Ш╨в1 vs
        // sign +1), the float arm runs; the drift +0.003 crosses zero.
        ai_paper_airplanes(&mut p, &mut rng, 0.5, 0.0, 0, -0.001);
        assert_eq!(p.direction, 1, "the X-sign flip");
        // ai[0] was 0 ╤В╨Ц╨в the counter++ ╤В╨Ц╨в 1, then the flip re-arm ╤В╨Ц╨в -120
        // (the re-arm is the LAST write ╤В╨Р╨д no counter after it).
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
        // A dirt wall at tile (101, 50) ╤В╨Р╨д the plane at (1600, 800) flies
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
        // extraUpdates 0 → one 5-px sub-step per update; the box (10 wide)
        // reaches the tile-101 edge at 1616 on the THIRD step. Tick until
        // the contact kill, not just once.
        for _ in 0..6 {
            pool.update(
                &mut w, &mut npcs, None, None, None,
                YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
            );
            if !pool.slots()[slot].active {
                break;
            }
        }
        assert!(!pool.slots()[slot].active, "the tile kill");
        let sounds = pool.drain_proj_sounds();
        assert!(sounds.iter().any(|&(id, style, _, _, _, _)| id == 2 && style == 10), "the Item10 cue");
        let drops = pool.drain_item_drops();
        assert_eq!(drops.len(), 1, "the recovery drop");
        assert_eq!(drops[0].0, 4343, "761 ╤В╨Ц╨в 4343");
        // The B variant drops 4344.
        let mut pool2 = ProjectilePool::new();
        let s2 = pool2
            .spawn_with_data_ai(PAPER_AIRPLANE_B, 1600.0, 800.0, 5.0, 0.0, 4, 2.0, 0.0, 0.0)
            .unwrap();
        for _ in 0..6 {
            pool2.update(
                &mut w, &mut npcs, None, None, None,
                YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
            );
            if !pool2.slots()[s2].active {
                break;
            }
        }
        assert!(!pool2.slots()[s2].active);
        assert_eq!(pool2.drain_item_drops()[0].0, 4344, "762 ╤В╨Ц╨в 4344");
    }

    /// The recovery-drop ITEM rows census (residual 4): 4343/4344
    /// (`Item.cs:37117-37129`) ╤В╨Р╨д 22x16, sellPrice(0,0,10) = 5000,
    /// `DefaultToThrownWeapon(761/762, 17, 5f)` (`:48455-48466`:
    /// useStyle 1, useTime/useAnimation 17, shoot, shootSpeed 5,
    /// noMelee, ranged, consumable, autoReuse false) +
    /// `SetWeaponValues(4, 2f)` (damage 4, knockback 2). The generated
    /// rows must carry the drop's re-throw shape exactly.
    #[test]
    fn w4_d16_159_the_recovery_item_rows_match_set_defaults() {
        for (item, shoot) in [(4343u16, 761u16), (4344, 762)] {
            let d = crate::registry::get(item).unwrap_or_else(|| panic!("item {item}"));
            assert_eq!((d.width, d.height), (22, 16), "item {item}");
            assert_eq!(d.value, 5000, "sellPrice(0,0,10) тФЬ╨з 5");
            assert_eq!(d.projectile, Some(shoot));
            assert_eq!(d.use_style, 1);
            assert_eq!((d.use_time, d.use_animation), (17, 17));
            assert_eq!(d.shoot_speed, 5.0);
            assert_eq!(d.damage, 4);
            assert_eq!(d.knock_back, 2.0);
            assert!(d.ranged && d.consumable);
            assert!(!d.auto_reuse && !d.channel);
            assert_eq!(d.class, crate::registry::DamageClass::Ranged);
        }
    }

    /// The recovery-drop gates (`Projectile.cs:76086`): a FOREIGN
    /// owner's plane queues the cue but NO drop; the local owner's
    /// `noDropItem` flag refuses it too.
    #[test]
    fn w4_d16_159_the_recovery_drop_gates() {
        let mut w = world();
        let i = w.index(101, 50);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut npcs: Vec<Npc> = Vec::new();
        // The foreign owner (1 ╤В╨Щ╨░ local 0).
        let mut pool = ProjectilePool::new();
        let s = pool
            .spawn_with_data_ai_owned(PAPER_AIRPLANE_A, 1, 1600.0, 800.0, 5.0, 0.0, 4, 2.0, 0.0, 0.0)
            .unwrap();
        pool.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
        );
        assert!(!pool.slots()[s].active);
        assert!(pool.drain_item_drops().is_empty(), "the owner gate");
        assert!(
            pool.drain_proj_sounds().iter().any(|&(id, style, _, _, _, _)| id == 2 && style == 10),
            "the cue plays regardless"
        );
        // The local owner's noDropItem flag.
        let mut pool2 = ProjectilePool::new();
        let s2 = pool2
            .spawn_with_data_ai(PAPER_AIRPLANE_A, 1600.0, 800.0, 5.0, 0.0, 4, 2.0, 0.0, 0.0)
            .unwrap();
        pool2.slots_mut()[s2].no_drop_item = true;
        pool2.update(
            &mut w, &mut npcs, None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut census(), 0, false,
        );
        assert!(!pool2.slots()[s2].active);
        assert!(pool2.drain_item_drops().is_empty(), "the noDropItem gate");
    }
}
#[cfg(test)]
mod w4_d18_172_tests {
    //! W4-d18 ╤В╨Р╨д aiStyle 172 (`AI_172_HallowBossRainbowPelletStorm`,
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

    /// The grow-in (`:45571-45580`): tick 1 ╤В╨Ц╨в localAI[1] = 1, scale
    /// Lerp(5,1,1/90), opacity (1/90)тФмтЦУ; the alpha rides the Opacity
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
        // Tick 91 is the LAST grow-in tick (localAI[1] 90 ╤В╨Щ╨┤ 90 holds).
        let mut p2 = core();
        p2.local_ai[1] = 90.0;
        ai_pellet_storm(&mut p2);
        assert_eq!(p2.local_ai[1], 91.0);
        assert_eq!(p2.scale, 1.0);
        assert_eq!(p2.alpha, 0, "opacity 1 ╤В╨Ц╨в alpha 0");
    }

    /// The storm phase (`:45581-45591`): scale/opacity pin 1, the counter
    /// climbs, velocity zeroes, rotation pins 0 ╤В╨Р╨д even with a spawn drift.
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

    /// The kill (`:45585-45589`): localAI[0] reaches 210 ╤В╨Ц╨в dead, the
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
    /// ╤В╨Ш╨втХз╨Р/2, the 2тХз╨Р/3 spread, the progress ramp and the 500-px ray.
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
        // Bullet 0 sits straight up at half range: (0, ╤В╨Ш╨в250).
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

/// `Entity.AngleTo` (`Entity.cs:185-188`): `(float)Math.Atan2(dy, dx)` ╤В╨Р╨д
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

/// `AI_174_MultisegmentPet` (`Projectile.cs:45345-45427`) ╤В╨Р╨д the three
/// worm-segment pets (883 Eater of Worlds / 887 Destroyer / 893 Lunatic
/// Cultist). The inactive-owner kill, the per-type keep-alive (death
/// clears the flag FIRST, a standing flag pins `timeLeft = 2` ╤В╨Р╨д the
/// `projectile_ai026_b::keep_alive` shape), the >2000-px teleport, the
/// zero-velocity kick with the `oldPos` refill, the 5тФмтЦС/tick steering past
/// 120 px, the speed clamp to `Min(12, Max(4, |player.velocity|))`, the
/// ╤В╨Ш╨в0.1 Y drift under 1, the flight rotation + тХз╨Р/2, the direction flip
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
    // `:45347-45352` ╤В╨Р╨д the inactive owner kills outright (no `timeLeft`
    // write ╤В╨Р╨д the C# sets `active = false` only, not `Kill()`).
    let Some(player) = player else {
        p.active = false;
        return;
    };
    // `:45353-45385` ╤В╨Р╨д the three keep-alive blocks: death clears the
    // flag, then the (possibly cleared) flag pins the lifetime.
    let idx = usize::from(p.kind);
    if player.dead {
        player.pet_flags[idx] = false;
    }
    if player.pet_flags[idx] {
        p.time_left = 2;
    }
    // `:45386-45392` ╤В╨Р╨д the teleport: `Distance(center) > 2000` snaps the
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
    // `:45393-45395` ╤В╨Р╨д the distance and the speed band. The discarded
    // `base.velocity.Length()` call (`:45395`) has no side effect.
    let num = (center - pcenter).length();
    let player_speed = Vec2::new(player.vx, player.vy).length();
    let num2 = player_speed.max(4.0).min(12.0);
    // `:45396-45405` ╤В╨Р╨д the zero-velocity kick: `velocity.X = 2тФмтХЦdirection`
    // and the `oldPos` ring refills walking BACK from the position.
    if p.vel == Vec2::ZERO {
        p.vel.x = 2.0 * player.facing as f32;
        let mut vector = p.pos;
        for slot in p.old_pos.iter_mut() {
            vector -= p.vel;
            *slot = vector;
        }
    }
    // `:45406-45409` ╤В╨Р╨д past 120 px the heading turns toward the owner at
    // `MathHelper.ToRadians(5)` per tick (`5тФмтХЦтХз╨Р/180`, the XNA constant).
    if !(num < 120.0) {
        let target = angle_to(pcenter, center);
        let cur = to_rotation(p.vel);
        let turned = crate::projectile::angle_towards(cur, target, std::f32::consts::PI / 180.0 * 5.0);
        p.vel = to_rotation_vector2(turned) * num2;
    }
    // `:45410-45413` ╤В╨Р╨д the speed cap (`SafeNormalize(Zero)`).
    if p.vel.length() > num2 {
        p.vel = safe_normalize(p.vel) * num2;
    }
    // `:45414-45417` ╤В╨Р╨д the near-horizontal drift.
    if p.vel.y.abs() < 1.0 {
        p.vel.y -= 0.1;
    }
    // `:45418` ╤В╨Р╨д the flight rotation + тХз╨Р/2.
    p.rot = to_rotation(p.vel) + std::f32::consts::FRAC_PI_2;
    // `:45419-45424` ╤В╨Р╨д the direction/spriteDirection flip (the netUpdate
    // on a change).
    let num3 = p.direction;
    let dir = if p.vel.x > 0.0 { 1 } else { -1 };
    p.direction = dir;
    p.sprite_direction = dir;
    if num3 != p.direction {
        p.net_update = true;
    }
    // `:45425-45426` ╤В╨Р╨д the world-edge clamp (160 px inset).
    p.pos.x = p.pos.x.clamp(160.0, world.width as f32 * 16.0 - 160.0);
    p.pos.y = p.pos.y.clamp(160.0, world.height as f32 * 16.0 - 160.0);
}

/// `AI_175_TitaniumStormShards` (`Projectile.cs:45319-45343`) ╤В╨Р╨д the
/// Titanium Storm Shard (908). The buff-gated kill, the first-tick frame
/// + rotation roll (`Next(12)` then `NextFloat`), the тХз╨Р/200 spin, the
/// group-index orbit angle `(index/total + miscCounterNormalizedтФмтХЦ6)тФмтХЦ2тХз╨Р`,
/// the `24 + totalтФмтХЦ6` radius, the owner's per-tick move added to the
/// centre, the 0.3 lerp toward the squashed orbit point and the
/// `localAI[0] = sin(angle)` draw-layer key.
///
/// RNG: exactly two draws on the FIRST tick (`frameCounter == 0`), zero
/// after. The `drawLayer` write (`:45342`, 3 below / 4 above the owner)
/// is draw-only ╤В╨Р╨д no port field, recorded here.
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
    // `:45321-45326` ╤В╨Р╨д the owner/buff gate: inactive, dead or buff-less
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
    // `:45327-45332` ╤В╨Р╨д the first-tick roll: frameCounter latches 1, the
    // frame picks one of the 12 rows and the rotation a full-circle angle.
    if p.frame_counter == 0 {
        p.frame_counter = 1;
        p.frame = rng.next_range(0, 12);
        p.rot = rng.next_float() * (std::f32::consts::PI * 2.0);
    }
    // `:45333` ╤В╨Р╨д the тХз╨Р/200 spin.
    p.rot += std::f32::consts::PI / 200.0;
    // `:45335-45336` ╤В╨Р╨д the orbit angle and radius. `miscCounterNormalized`
    // is `miscCounter / 300f` (`Player.cs:3389`).
    let misc_norm = player.misc_counter as f32 / 300.0;
    let f = (group_index as f32 / group_total as f32 + misc_norm * 6.0)
        * (std::f32::consts::PI * 2.0);
    let num = 24.0 + group_total as f32 * 6.0;
    // `:45337-45338` ╤В╨Р╨д the owner's per-tick move rides the shard centre.
    let vector = Vec2::new(player.x - player.old_x, player.y - player.old_y);
    let mut center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    center += vector;
    // `:45339-45341` ╤В╨Р╨д the orbit point: the unit ray squashed to 5% Y,
    // scaled by the radius, offset from the owner's centre; the shard
    // lerps 0.3 toward it (`Lerp(value1: Center, value2: target, 0.3)`).
    let (px, py) = player.center();
    let player_center = Vec2::new(px, py);
    let ray = to_rotation_vector2(f);
    p.local_ai[0] = ray.y;
    let target = player_center + ray * Vec2::new(1.0, 0.05) * num;
    center = center.lerp(target, 0.3);
    p.pos = Vec2::new(center.x - p.w * 0.5, center.y - p.h * 0.5);
    // `:45342` ╤В╨Р╨д `drawLayer = localAI[0] <= 0 ? 3 : 4` ╤В╨Р╨д draw-only, the
    // port carries no drawLayer field (the localAI[0] key survives).
}

/// `Projectile.FindTargetWithinRange` (`Projectile.cs:45097-45115`) ╤В╨Р╨д
/// the nearest chaseable NPC within `max_range` whose local-immunity
/// entry is 0. NOT the line-of-sight variant (`:45632-45660`): the
/// default `checkCanHit = false` skips the `Collision.CanHitLine` test.
/// When `check_can_hit` is true (the 969 twister, `:45178-45181`) the
/// `CanHitLine` box test joins the eligibility gate, so a `world` rides
/// along. The strict `!(num <= num2)` keeps the FIRST nearest on ties
/// (the scan order is the NPC index). Returns ╤В╨Ш╨в1 for no target (the C#
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
        // `:45104` ╤В╨Р╨д the optional `Collision.CanHitLine` eligibility term.
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

/// `AI_176_EdgyLightning` (`Projectile.cs:45058-45095`) ╤В╨Р╨д the Dark
/// Harvest's scythe bolt (916). The `ai[0] ╤В╨Ш╨в 1` target latch with its
/// chaseable/immunity re-validation, the 400-px acquire, the 10-px
/// `DirectionTo` flight with the velocity rotation, the 3-tick frame
/// machine (`Next(projFrames)` ╤В╨Р╨д the ONLY draw) and the no-target kill.
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
    // `:45060-45065` ╤В╨Р╨д the held target re-validates: a gone/un-chaseable
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
    // `:45066-45075` ╤В╨Р╨д the acquire: the nearest within 400 px latches
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
        // `:45078-45084` ╤В╨Р╨д the 10-px chase: `DirectionTo` is
        // `Normalize(target ╤В╨Ш╨в Center)`; a NaN direction (the zero delta)
        // keeps the old velocity and rotation.
        let n = &npcs[num as usize];
        let delta = crate::projectile::npc_center(n) - center;
        if !delta.is_nan() && delta != Vec2::ZERO {
            let vector = delta.normalize();
            p.vel = vector * 10.0;
            p.rot = to_rotation(p.vel);
        }
        // `:45085-45089` ╤В╨Р╨д the 3-tick frame machine: the ONLY rand draw.
        p.frame_counter += 1;
        if p.frame_counter >= 3 {
            p.frame_counter = 0;
            p.frame = rng.next_range(0, crate::projectile::proj_frames(p.kind) as i32);
        }
    } else {
        // `:45091-45094` ╤В╨Р╨д no target: `Kill()`.
        p.active = false;
        p.time_left = 0;
    }
}

/// `AI_177_IceWhipSlicer` (`Projectile.cs:45117-45265`) ╤В╨Р╨д the Cool Whip's
/// snowflake slicer (917) and the Weather Pain's twister (969). The
/// 917 buff-gated kill/keep-alive, the 969 twister lifetime (the 555-tick
/// sibling-sync floor, the >560 alpha fade, the 570 kill), the owner-gated
/// target latch/acquire (the 969 adds the `CanHitLine` test), the
/// per-kind steering constants (917: 18 px / 0.1 lerp; 969: 8 px / 0.075
/// lerp / 25-px dead zone), the no-target arms (969 self-direction lerp,
/// 917 тФЬ╨з0.92 decay) and the per-kind rotation/frame/dust tails.
///
/// Returns `true` when the 969 sibling-sync scan must run (the dispatch
/// pre-scans the pool ╤В╨Р╨д the slot arm cannot see the other slots, the
/// `minion_separation` borrow pattern). The sync pins every older
/// same-owner same-kind twister with `ai[1] < 555` up to 555.
///
/// RNG: the 917 tail rolls `Next(3)` every tick; the taken branch's
/// `NextVector2Circular(1,1)` pair stays (the dust body is the skip).
/// The 969 tail rolls nothing explicit ╤В╨Р╨д its `timeLeft % 3` dust gate's
/// `NextVector2Circular` pairs are `Dust.NewDust*` call-site velocity
/// writes, kept as draws (the flamethrower precedent): one pair + two
/// pairs per gated tick. The sound slot (`localAI[1]`, `:45132-45146`)
/// has no tracked-sound layer ╤В╨Р╨д the slot-id bookkeeping is skipped and
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
    // `:45119-45129` ╤В╨Р╨д the 917 buff gate: an absent/dead owner clears
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
    // `:45130-45165` ╤В╨Р╨д the 969 twister lifetime. The tracked-sound block
    // (`:45132-45146`) has no sound layer ╤В╨Р╨д skipped (module doc).
    if p.kind == crate::projectile::WEATHER_PAIN_SHOT {
        p.ai[1] += 1.0;
        if p.ai[1] > 560.0 {
            // `:45150` ╤В╨Р╨д `(int)MathHelper.Lerp(0, 250, (ai[1]╤В╨Ш╨в560)/10)`.
            p.alpha = (0.0f32 + (250.0f32 - 0.0f32) * ((p.ai[1] - 560.0) / 10.0)) as i32;
        }
        if p.ai[1] >= 570.0 {
            p.active = false;
            p.time_left = 0;
        }
        // `:45156-45164` ╤В╨Р╨д the sibling-sync scan runs in the dispatch.
        sync_siblings = true;
    }
    // `:45166-45190` ╤В╨Р╨д the owner-gated target latch/acquire.
    let mut num2 = p.ai[0] as i32 - 1;
    if p.owner == local_owner {
        if num2 != -1 {
            let gone = npcs.get(num2 as usize).map_or(true, |n| !chaseable(n));
            // `:45169` ╤В╨Р╨д the 177 re-validation uses `> 0` (NOT `!= 0`):
            // the ╤В╨Ш╨в1 sentinel keeps the lock.
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
    // `:45191-45199` ╤В╨Р╨д the per-kind steering constants.
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
        // `:45200-45213` ╤В╨Р╨д the chase: the speed clamps to the distance,
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
        // `:45214-45221` ╤В╨Р╨д the twister's self-direction lerp:
        // `DirectionTo(Center + velocity)` = the velocity's own heading.
        let delta = p.vel;
        if !delta.is_nan() && delta != Vec2::ZERO {
            let vector2 = delta.normalize();
            p.vel = p.vel.lerp(vector2 * num3, amount);
        }
    } else {
        // `:45222-45225` ╤В╨Р╨д the snowflake's no-target decay.
        p.vel *= 0.92;
    }
    if p.kind == crate::projectile::WEATHER_PAIN_SHOT {
        // `:45226-45252` ╤В╨Р╨д the twister tail: the X-scaled rotation, the
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
            // `:45243` ╤В╨Р╨д the first dust's NextVector2Circular(2,2) pair.
            let _ = (rng.next_float(), rng.next_float());
            // `:45244-45251` ╤В╨Р╨д the two direct dusts' pairs.
            for _ in 0..2 {
                let _ = (rng.next_float(), rng.next_float());
            }
        }
    } else {
        // `:45254-45264` ╤В╨Р╨д the snowflake tail: the spin and the 1-in-3
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

/// `AI_178_FireExplosion` (`Projectile.cs:45267-45317`) ╤В╨Р╨д the Firecracker's
/// explosion flash (918). The first tick (`localAI[0] == 0`) plays
/// `SoundID.Item14` and spawns the four dust rings + the two gores; then
/// `localAI[0]` counts every tick and the 10-tick cap kills.
///
/// RNG census (the first tick ONLY ╤В╨Р╨д the dust/gore BODIES are the skip,
/// the explicit call-site draws stay in the C#'s order):
/// - the 4тФЬ╨з dust-31 ring (`:45272-45276`): each `RotatedByRandom(тХз╨Р)` pair
///   (two `NextDouble`s) + the radius `NextDouble` ╤В╨Ц╨в 12 doubles;
/// - the 30тФЬ╨з dust-6 pair (`:45277-45288`): each iteration two dusts, each
///   a `RotatedByRandom(тХз╨Р)` pair + a `NextDouble` radius ╤В╨Ц╨в 180 doubles;
/// - the 10тФЬ╨з dust-6 UnitX ring (`:45289-45295`): each a
///   `RotatedByRandom(тХз╨Р)` pair ╤В╨Ц╨в 20 doubles (NO radius draw ╤В╨Р╨д the reach
///   is the fixed `width/2`);
/// - the 10тФЬ╨з dust-31 UnitX ring (`:45296-45302`): the same ╤В╨Ц╨в 20 doubles;
/// - the 2тФЬ╨з gore (`:45303-45310`): each the position `Next(100)` тФЬ╨з2, the
///   type `Next(61,64)`, the `NewGore` internal kicks `Next(10,31)` +
///   `Next(-20,21)`, the `RotatedByRandom(тХз╨Р)` pair + the `NextDouble`
///   radius, then the velocity `Next(-10,11)` тФЬ╨з2 ╤В╨Ц╨в 14 ranges + 6 doubles.
/// Total: 14 ranges + 238 doubles on tick 1, ZERO after.
pub(crate) fn ai_fire_explosion(p: &mut Projectile, rng: &mut dyn DropRng) {
    if p.local_ai[0] == 0.0 {
        // `:45271` ╤В╨Р╨д the explosion cue (the wants_sound seam).
        p.wants_sound = Some((2, 14, p.pos.x, p.pos.y, 0.0, 1.0));
        // `:45272-45276` ╤В╨Р╨д the 4тФЬ╨з dust-31 ring: RotatedByRandom pair +
        // the NextDouble radius each.
        for _ in 0..4 {
            let _ = (rng.next_double(), rng.next_double());
            let _ = rng.next_double();
        }
        // `:45277-45288` ╤В╨Р╨д the 30тФЬ╨з dust-6 pair: two dusts per iteration,
        // each a RotatedByRandom pair + a NextDouble radius.
        for _ in 0..30 {
            for _ in 0..2 {
                let _ = (rng.next_double(), rng.next_double());
                let _ = rng.next_double();
            }
        }
        // `:45289-45295` ╤В╨Р╨д the 10тФЬ╨з dust-6 UnitX ring: the pair only.
        for _ in 0..10 {
            let _ = (rng.next_double(), rng.next_double());
        }
        // `:45296-45302` ╤В╨Р╨д the 10тФЬ╨з dust-31 UnitX ring: the pair only.
        for _ in 0..10 {
            let _ = (rng.next_double(), rng.next_double());
        }
        // `:45303-45310` ╤В╨Р╨д the 2тФЬ╨з gore: the position pair, the type, the
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
    // `:45312-45316` ╤В╨Р╨д the age counter and the 10-tick kill.
    p.local_ai[0] += 1.0;
    if p.local_ai[0] >= 10.0 {
        p.active = false;
        p.time_left = 0;
    }
}

#[cfg(test)]
mod w4_d18_174_tests {
    //! W4-d18 ╤В╨Р╨д aiStyle 174 (`AI_174_MultisegmentPet`,
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
    /// 20x20, aiStyle 174, penetrate -1, timeLeft 18000 (the тФЬ╨з5),
    /// tileCollide false, ignoreWater + netImportant, NOT minions.
    #[test]
    fn w4_d18_174_the_rows_match_set_defaults() {
        for kind in [EATER_OF_WORLDS_PET, DESTROYER_PET, LUNATIC_CULTIST_PET] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h, d.ai_style), (20.0, 20.0, 174), "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, 18000, "kind {kind}: 3600 тФЬ╨з 5");
            assert!(!d.tile_collide && d.ignore_water && d.net_important, "kind {kind}");
            assert!(!d.minion && !d.sentry, "kind {kind}: pets, not minions");
            assert_eq!(d.extra_updates, 0, "kind {kind}");
        }
        // The frame sheets (`Main.cs:8671`/`:8675`/`:8681`).
        assert_eq!(crate::projectile::proj_frames(EATER_OF_WORLDS_PET), 3);
        assert_eq!(crate::projectile::proj_frames(DESTROYER_PET), 3);
        assert_eq!(crate::projectile::proj_frames(LUNATIC_CULTIST_PET), 4);
    }

    /// `:45347-45352` ╤В╨Р╨д an absent owner kills outright (active false,
    /// the C# does NOT zero timeLeft here).
    #[test]
    fn w4_d18_174_an_absent_owner_kills() {
        let w = world();
        let mut p = pet(EATER_OF_WORLDS_PET, 1600.0, 800.0);
        let tl = p.time_left;
        ai_multisegment_pet(&mut p, None, &w);
        assert!(!p.active);
        assert_eq!(p.time_left, tl, "no Kill() ╤В╨Р╨д the raw deactivate");
    }

    /// `:45353-45385` ╤В╨Р╨д the keep-alive: a standing flag pins timeLeft 2;
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

    /// `:45386-45392` ╤В╨Р╨д the >2000-px teleport snaps the centre onto the
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

    /// `:45396-45405` ╤В╨Р╨д the zero-velocity kick: X = 2тФмтХЦdirection and the
    /// oldPos ring walks BACK from the position by the kick velocity.
    #[test]
    fn w4_d18_174_the_zero_kick_refills_old_pos() {
        let w = world();
        let mut pl = owner(&w);
        let mut p = pet(DESTROYER_PET, 1600.0, 800.0);
        p.vel = Vec2::ZERO;
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        assert_eq!(p.vel.x, 2.0, "2 тФЬ╨з facing 1");
        // entry i = pos ╤В╨Ш╨в (i+1)тФмтХЦvel (the kick's X only, Y drifts ╤В╨Ш╨в0.1).
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

    /// `:45393-45417` ╤В╨Р╨д the speed band: `Min(12, Max(4, |player.vel|))`,
    /// the >120-px steering, the cap and the ╤В╨Ш╨в0.1 Y drift.
    #[test]
    fn w4_d18_174_the_speed_band_and_steering() {
        let w = world();
        let mut pl = owner(&w);
        let (cx, cy) = pl.center();
        // The owner stands still: the band floors at 4. The pet sits on
        // the owner's centre-Y so the aim is pure ╤В╨Ш╨вX (no Y component to
        // fight the drift assertion).
        let mut p = pet(LUNATIC_CULTIST_PET, cx + 300.0 - 10.0, cy - 10.0);
        p.vel = Vec2::new(-4.0, 0.0);
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        assert!((p.vel.length() - 4.0).abs() < 2e-3, "the 4 floor (plus the ╤В╨Ш╨в0.1 drift)");
        assert!(p.vel.x < 0.0, "still flying toward the owner");
        assert!(p.vel.y < 0.0, "the ╤В╨Ш╨в0.1 drift under |Y| < 1");
        // A fast owner raises the band to its 12 cap.
        pl.vx = 30.0;
        let mut p2 = pet(LUNATIC_CULTIST_PET, cx + 300.0 - 10.0, cy - 10.0);
        p2.vel = Vec2::new(-20.0, 0.0);
        ai_multisegment_pet(&mut p2, Some(&mut pl), &w);
        assert!(p2.vel.length() <= 12.0 + 1e-3, "the 12 cap");
        // Inside 120 px NO steering runs ╤В╨Р╨д the velocity keeps its shape
        // (only the cap/drift touch it).
        let mut p3 = pet(LUNATIC_CULTIST_PET, cx + 50.0 - 10.0, cy - 10.0);
        p3.vel = Vec2::new(0.0, 3.0);
        ai_multisegment_pet(&mut p3, Some(&mut pl), &w);
        assert_eq!(p3.vel.x, 0.0, "no turn inside the 120 band");
        assert_eq!(p3.vel.y, 3.0, "|Y| ╤В╨Щ╨╡ 1 keeps the drift off");
    }

    /// `:45418-45426` ╤В╨Р╨д the rotation, the direction flip and the clamp.
    #[test]
    fn w4_d18_174_the_rotation_direction_and_clamp() {
        let w = world();
        let mut pl = owner(&w);
        let mut p = pet(EATER_OF_WORLDS_PET, 1600.0, 800.0);
        // |Y| = 1 keeps the drift off and the length ╤В╨Ш╨к10 < 4 keeps the cap
        // off, so the velocity is untouched and the rotation is exact.
        p.vel = Vec2::new(3.0, 1.0);
        p.direction = -1;
        ai_multisegment_pet(&mut p, Some(&mut pl), &w);
        assert_eq!(p.direction, 1, "the X-sign flip");
        assert_eq!(p.sprite_direction, 1);
        assert!(p.net_update, "the flip flags the net update");
        let want_rot = to_rotation(Vec2::new(3.0, 1.0)) + std::f32::consts::FRAC_PI_2;
        assert!((p.rot - want_rot).abs() < 1e-5, "rot = ToRotation + тХз╨Р/2");
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
    //! W4-d18 ╤В╨Р╨д aiStyle 175 (`AI_175_TitaniumStormShards`,
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

    /// `:45321-45326` ╤В╨Р╨д the absent owner, the dead owner and the missing
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

    /// `:45327-45333` ╤В╨Р╨д the first tick rolls `Next(12)` then `NextFloat`
    /// (the rotation), latches frameCounter 1 and adds the тХз╨Р/200 spin;
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
        // rotation = 0тФмтХЦ2тХз╨Р + тХз╨Р/200.
        assert_eq!(p.rot, std::f32::consts::PI / 200.0);
        ai_titanium_storm_shard(&mut p, Some(&pl), true, 0, 1, &mut rng);
        assert_eq!(rng.ranges.len(), 1, "no second frame roll");
        assert_eq!(rng.floats, 1, "no second rotation roll");
        assert_eq!(p.rot, std::f32::consts::PI / 200.0 * 2.0, "the spin stacks");
    }

    /// `:45335-45341` ╤В╨Р╨д the orbit maths: at miscCounter 0, index 0 of 1
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
        // angle 0 ╤В╨Ц╨в ray (1, 0); target = centre + (30, 0); lerp 0.3 from
        // the centre ╤В╨Ц╨в centre + (9, 0).
        assert_eq!(p.local_ai[0], 0.0, "the ray Y at angle 0");
        let want_center = Vec2::new(cx + 9.0, cy);
        assert!((p.pos.x + 7.0 - want_center.x).abs() < 1e-3);
        assert!((p.pos.y + 7.0 - want_center.y).abs() < 1e-3);
        // Index 1 of 2 at miscCounter 0: angle тХз╨Р ╤В╨Ц╨в ray (╤В╨Ш╨в1, ~0); the
        // radius is 24 + 2тФмтХЦ6 = 36, the lerp target centre + (╤В╨Ш╨в10.8, ~0).
        let mut p2 = shard(cx - 7.0, cy - 7.0);
        ai_titanium_storm_shard(&mut p2, Some(&pl), true, 1, 2, &mut rng);
        assert!(p2.pos.x + 7.0 < cx, "the тХз╨Р ray points left");
        // The owner's per-tick move rides the centre FIRST (`:45337-45338`).
        let mut pl2 = owner(&w);
        pl2.x += 10.0; // old_x stays ╤В╨Ц╨в vector (10, 0)
        let mut p3 = shard(cx - 7.0, cy - 7.0);
        ai_titanium_storm_shard(&mut p3, Some(&pl2), true, 0, 1, &mut rng);
        // centre + (10,0) then lerp 0.3 toward (centre + 10 + 30, cy):
        // the player's centre also moved +10 ╤В╨Ц╨в target cx + 40.
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
    //! W4-d18 ╤В╨Р╨д aiStyle 176 (`AI_176_EdgyLightning`,
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
    /// penetrate 3, tileCollide false, extraUpdates 2, local-immunity ╤В╨Ш╨в1.
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

    /// `:45066-45075` + `:45078-45084` ╤В╨Р╨д the acquire latches ai[0] =
    /// whoAmI+1, the velocity aims at 10 px and the rotation follows.
    #[test]
    fn w4_d18_176_the_acquire_and_chase() {
        let mut p = bolt(1600.0, 800.0);
        // The NPC 100 px to the right (its centre ╤В╨Р╨д the zombie box).
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

    /// `:45085-45089` ╤В╨Р╨д the 3-tick frame machine rolls `Next(6)` exactly
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

    /// `:45060-45065` ╤В╨Р╨д a held target that dies or turns immune drops
    /// the latch and re-acquires; `:45091-45094` ╤В╨Р╨д no target kills.
    #[test]
    fn w4_d18_176_the_latch_revalidation_and_kill() {
        let mut p = bolt(1600.0, 800.0);
        p.ai[0] = 1.0; // holds NPC 0
        let mut n = npc_at(1700.0, 800.0);
        n.hp = 0; // dead ╤В╨Ц╨в un-chaseable
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
        assert!(!p2.active, "no re-acquire in range ╤В╨Ц╨в kill");
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
        // extraUpdates 2 ╤В╨Ц╨в 3 sub-steps ╤В╨Ц╨в the frame machine rolled once.
        assert_eq!(rng.ranges, vec![(0, 6)]);
    }
}

#[cfg(test)]
mod w4_d18_177_tests {
    //! W4-d18 ╤В╨Р╨д aiStyle 177 (`AI_177_IceWhipSlicer`,
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

    /// `:45119-45129` ╤В╨Р╨д the 917 buff gate: absent/dead owner kills and
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

    /// `:45147-45155` ╤В╨Р╨д the 969 lifetime: ai[1] counts, the >560 alpha
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

    /// `:45156-45164` ╤В╨Р╨д the sibling sync pins older-living twisters'
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
        slots[2].ai[1] = 600.0; // already past 555 ╤В╨Р╨д stays
        slots[3].owner = 1; // a foreign owner ╤В╨Р╨д stays
        weather_pain_sync_siblings(&mut slots, 0);
        assert_eq!(slots[1].ai[1], 555.0, "the younger sibling pinned");
        assert!(slots[1].net_update);
        assert_eq!(slots[2].ai[1], 600.0, "the past-555 sibling stays");
        assert_eq!(slots[3].ai[1], 10.0, "the foreign owner stays");
        assert_eq!(slots[0].ai[1], 10.0, "the caller itself stays");
    }

    /// `:45191-45225` ╤В╨Р╨д the steering: the 917 no-target тФЬ╨з0.92 decay and
    /// the 1-in-3 dust gate's draw census; the 969 self-direction lerp.
    #[test]
    fn w4_d18_177_the_no_target_arms_and_draws() {
        let w = world();
        let npcs: Vec<Npc> = Vec::new();
        let immune = [0i32; 256];
        let mut rng = census();
        // The 917's head kills an absent/dead owner (`:45119-45123`), so the
        // steering test needs a LIVE owner snapshot (buff off → no pin).
        let mut pl = owner(&w);
        let mut p = slicer(COOL_WHIP_PROJ, 1600.0, 800.0);
        p.vel = Vec2::new(10.0, 0.0);
        ai_ice_whip_slicer(&mut p, &w, &npcs, &immune, Some(&mut pl), 0, &mut rng);
        assert!((p.vel.x - 9.2).abs() < 1e-4, "the 0.92 decay");
        // CensusRng returns min 0 ╤В╨Ц╨в the 1-in-3 gate TAKEN ╤В╨Ц╨в one pair.
        assert_eq!(rng.ranges, vec![(0, 3)]);
        assert_eq!(rng.floats, 2, "the taken branch's circular pair");
        assert!((p.rot - (0.020943953 + 9.2 * 0.2)).abs() < 1e-5, "the spin");
        // The 969 self-lerp: velocity (8,0) lerps 0.075 toward (8,0) ╤В╨Р╨д
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
    //! W4-d18 ╤В╨Р╨д aiStyle 178 (`AI_178_FireExplosion`,
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
    /// penetrate -1, tileCollide false, alpha 255, local-immunity ╤В╨Ш╨в1.
    #[test]
    fn w4_d18_178_the_row_matches_set_defaults() {
        let d = proj_data(FIRE_WHIP_PROJ).unwrap();
        assert_eq!((d.w, d.h, d.ai_style), (96.0, 96.0, 178));
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
        assert!(!d.tile_collide);
        assert!(d.uses_local_immunity && d.local_hit_cooldown == -1);
    }

    /// `:45269-45311` ╤В╨Р╨д the first-tick census: 16 ranges + 238 doubles,
    /// the Item14 cue, and the counter advances to 1.
    #[test]
    fn w4_d18_178_the_first_tick_census() {
        let mut p = flash(1600.0, 800.0);
        let mut rng = census();
        ai_fire_explosion(&mut p, &mut rng);
        // 4тФЬ╨з3 + 30тФЬ╨з2тФЬ╨з3 + 10тФЬ╨з2 + 10тФЬ╨з2 + 2тФЬ╨з3 = 12+180+20+20+6 = 238 doubles.
        assert_eq!(rng.doubles, 238, "the double census");
        // 2 gores тФЬ╨з (2 position + 1 type + 2 NewGore kicks + 2 velocity) = 14.
        assert_eq!(rng.ranges.len(), 14, "the range census");
        assert_eq!(rng.floats, 0, "no float draws");
        assert_eq!(p.wants_sound, Some((2, 14, 1600.0, 800.0, 0.0, 1.0)), "the Item14 cue");
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

    /// `:45312-45316` ╤В╨Р╨д the counter climbs every tick, ZERO draws after
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
