//! W4-d19 — the aiStyles 182-186 / 190 census slice: the six `AI()` arms
//! (`Projectile.cs` dispatch `:36691-36726`) that had NO ported dispatch arm
//! and NO `proj_data` rows before this task. The 181/187/188/189 arms of the
//! same census window were already armed (181 in `projectile_ai_misc`,
//! 187/188/189 on the pool) and stay untouched.
//!
//! C# sources read WHOLE (the golden rule):
//! - aiStyle 182: `AI_182_FinalFractal` `Projectile.cs:43111-43176` — the
//!   Zenith's orbiting sword illusions (933 FinalFractal / 1100
//!   TrueCopperShortsword, rows `:8857-8873`/`:10265-10281`), plus
//!   `Graphics/FinalFractalHelper.cs` (the profile table `:128-215` and the
//!   `StripDust` draw body `:39-53`).
//! - aiStyle 183: `AI_183_ZoologistStrike` `:43095-43109` — the Zoologist
//!   whip-slash frames (880 ZoologistStrikeGreen / 929 ZoologistStrikeRed,
//!   rows `:8438-8450`/`:8804-8816`; `Main.projFrames` 8 each,
//!   `Main.cs:8694-8695`).
//! - aiStyle 184: `AI_184_BadTorchLuck` `:42787-42822` — the Torch God's
//!   fireball (949 TorchGod, row `:8948-8955`) + `SelfHurtPlayers`
//!   `:15025-15062`.
//! - aiStyle 185: `AI_185_LifeDrain` `:42631-42665` — the Soul Drain's
//!   one-shot mana-pay latch (476 SoulDrain, row `:5505-5517`).
//! - aiStyle 186: `AI_186_PrincessWeapon` `:42667-42785` — the Resonance
//!   Scepter's blast (950 PrincessWeapon, row `:8956-8968`).
//! - aiStyle 190: `AI_190_NightsEdge` `:41972-42227` — the seven big-sword
//!   slashes (972 NightsEdge / 982 Excalibur / 983 TrueExcalibur /
//!   984 TerraBlade2 / 997 TheHorsemansBlade / 1043 DeadCellsFlintSlash /
//!   1126 ChlorophyteClaymoreBlade, rows `:9298-9314`/`:9354-9404`/
//!   `:9466-9482`/`:9858-9874`/`:10543-10558`), plus the Chlorophyte
//!   Claymore's spore helpers `FindChlorophyteSporeCloudSpawn`
//!   `:41810-41846`, `SpawnChlorophyteSporeCloud` `:41861-41970` and
//!   `IsTileSolidForImpact` `:41848-41859`.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`, the
//! `projectile_ai_misc` precedent): every EXPLICIT `Main.rand` draw at the
//! call sites stays, in the C#'s order, INCLUDING the draws inside gates
//! that feed only skipped visuals; the dust/gore/light/particle BODIES and
//! the `Dust.NewDust*` internal rolls go with the dust layer (ANALYSIS
//! "Dust & gore"). The one exception is the 1126 spore cloud's
//! `KillTile_MakeTileDust` internals — pool-occupancy-dependent dust-index
//! rolls the port cannot replay (the `ai_stardust_guardian` documented
//! skip); the surrounding EXPLICIT draws (the fadeIn/scale floats and the
//! spawn gate) stay. Per style:
//! - 182: while `localAI[0] < 60` the dust loop draws one `NextFloat` per
//!   particle for the position lerp (`:43168`), then the profile's
//!   `StripDust` gate `Next(9)` (`FinalFractalHelper.cs:41`); on a 0 the
//!   count `Next(1, 4)` and THREE floats per dust (colour lerp, fadeIn,
//!   velocity length, `:46`/`:48`/`:50`). ZERO draws past tick 60 or on the
//!   kill tick.
//! - 183: ZERO draws (pure velocity/frame maths).
//! - 184: four dusts per tick, each with one `Next(3)` noGravity gate
//!   (`:42811`); the `SelfHurtPlayers` `DamageVar` draws ride
//!   `tr_core::damage::damage_var` on the same stream.
//! - 185: ZERO draws — the arm is a hitbox scan + one `CheckMana` pay.
//! - 186: while `ai[0]/60 < 0.9` two 3-iteration dust loops, each iteration
//!   one `Next(2)` gate, on a hit one `NextFloat` (the ring angle) + one
//!   `NextFloat` (fadeIn) (`:42694-42705`/`:42724-42735`); while `< 0.95`
//!   one `Next(4)` particle gate (`:42753`); at `ai[0] == 50` the four
//!   deterministic particle pairs draw nothing.
//! - 190: per-kind dust gates — 972: `Next(2)`, on a hit one
//!   `NextFloatDirection` + a `Next(5)` gate + a `Next(2)` gate
//!   (`:42028-42040`); 982/983: one `NextFloatDirection` + the two opacity
//!   `NextFloat` gates, each taken branch drawing its colour/fadeIn floats
//!   (`:42052-42064`/`:42185-42197`); 997: the same shape with ONE opacity
//!   gate (`:42068-42076`); 984: one `NextFloatDirection` + two opacity
//!   gates with their colour floats (`:42201-42220`); 1126: the spore
//!   machine (see [`claymore_spore_tick`]).
//!
//! Movement: the arms own their integration. 182's kinds are in
//! `UpdatePosition`'s skip list (`Projectile.cs:20435` names 933/1100) —
//! the AI sets `Center` and NOTHING integrates after it. 183/185/186/190
//! keep `tileCollide = false`, so `HandleMovement`'s default arm is the
//! free `position += velocity` (the `kill_step` stand-in). 184 starts
//! `tileCollide = false` and the arm re-arms it once `ai[1]` drains — the
//! blocked-move kill rides the same `kill_step` probe. `HandleMovement`
//! runs even on the AI-killed tick (`:17015` before the `!active` return
//! `:17027`) — the dispatch's post-arm `kill_step` keeps that order.
//!
//! Documented deviations (also `docs/ANALYSIS.md` "W4-d19 sword styles"):
//! * the sound cues (`SoundID.Item169` 182 `:43120`, `Item60` 984
//!   `:41978`, `Item8` 949 `:42796`, `Item167` spore `:41968`) ride the
//!   `wants_sound` intent seam where the style table carries them; the
//!   post-play `Volume *= 0.65` scale (984) is the S1-E-21h deferred-scale
//!   deviation;
//! * the `Lighting.AddLight` writes (182 `:43172-43173`, 1126
//!   `:42176-42181`, 984 `:42206`) are the lighting seam's skip;
//! * the `ParticleOrchestrator.RequestParticleSpawn` calls (186
//!   `:42757-42782`) are the particle layer's skip — their gates'
//!   `Next(4)` draws stay;
//! * the 933/1100 trail ring is TrailingMode 4 with cache 60
//!   (`ProjectileID.cs:521-522`) — the port's fixed 10-entry ring shifts
//!   once per update WITHOUT mode 4's owner-delta push (the recorded
//!   trail-density deviation, `shift_trailing_history`'s doc);
//! * the damage-side slices stay with the melee port: `ownerHitCheck` /
//!   `usesOwnerMeleeHitCD` / `stopsDealingDamageAfterPenetrateHits`
//!   (190's rows), the 190 cone hitbox (`Colliding` `:15506-15524`);
//! * the 1126 spore machine's CHILDREN — the 1128 cloud (aiStyle 92) and
//!   the 1129 slam (aiStyle 209): the spawn intents ride out of the arm
//!   ([`ClaymoreSpores`], the `wants_mark_spawn` pattern) and the dispatch
//!   drains them at the arm's tail (`projectile_ai_clouds::
//!   drain_claymore_spores` — the W4-d19 follow-up landed the 1128 row +
//!   the aiStyle-92 machine + the drain; the 1129 row + the 209 arm are
//!   W4-d21's claim on `task/W4-d21-proj-ai-201-210`, verified verbatim
//!   against `AI_209_ChlorophyteClaymoreBladeSlam` `:69622-69631` — the
//!   slam's data-seam spawn no-ops until that row merges, the documented
//!   cross-branch contract). The intent payloads and their RNG order are
//!   ported and witness-tested here;
//! * `SelfHurtPlayers` (184): the `deadMansSweater` halving, the
//!   gravestone player-index override and the `trapDebuffSource` write
//!   have no port fields — the 949 is neither trap nor gravestone, so all
//!   three folds are no-ops for the only rowed kind; the `ownerHitCheck`
//!   term is false (the row sets none);
//! * the 255-player PvP scan (185) folds to the single-player port's
//!   one player — the `whoAmI != player.whoAmI` gate refuses it, so the
//!   scan never fires (the empress module's single-player fold).
//!
//! Code-only round (W4-d19): the witness tests below are WRITTEN NOT RUN.

use std::f32::consts::PI;

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, get_lerp_value,
};

/// `MathHelper.Lerp` — XNA's `a + (b - a) * amount`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Utils.Remap` (`Utils.cs:345-348`) — the clamped lerp-of-lerp.
fn remap(v: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    lerp(to_min, to_max, get_lerp_value(from_min, from_max, v))
}

/// `Vector2.RotatedBy` (`Utils.cs:1675-1684`, centre = 0): cos/sin in
/// DOUBLE of the double angle, cast to f32, then the float component math.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Vector2.ToRotation` (`Utils.cs:1655-1658`): double atan2 cast to f32.
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `float.ToRotationVector2` (`Utils.cs:1665-1668`).
fn rotation_vector(angle: f32) -> Vec2 {
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Vector2.SafeNormalize(Vector2.Zero)` (`Utils.cs:1837-1846`).
fn safe_normalize(v: Vec2) -> Vec2 {
    if v == Vec2::ZERO || v.is_nan() {
        Vec2::ZERO
    } else {
        v.normalize()
    }
}

/// `Projectile.Opacity`'s setter (`Projectile.cs:546-549`).
fn set_opacity(p: &mut Projectile, opacity: f32) {
    p.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as i32;
}

/// `Projectile.Opacity`'s getter (`:543-544`).
fn opacity_of(p: &Projectile) -> f32 {
    1.0 - p.alpha as f32 / 255.0
}

/// `Kill()`'s state core (`Projectile.cs:70644-70650`): capture the
/// lifetime, deactivate, zero. The death-hook visuals are the pool-wide
/// kill-dust skip (the empress module doc).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Entity.Center` (`Entity.cs`).
fn center_of(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `Entity.Center`'s setter.
fn set_center(p: &mut Projectile, c: Vec2) {
    p.pos = c - Vec2::new(p.w * 0.5, p.h * 0.5);
}

/// `Utils.NextFloatDirection` (`Utils.cs:1203-1206`): one `NextDouble`
/// mapped to [-1, 1].
fn next_float_direction(rng: &mut dyn DropRng) -> f32 {
    rng.next_float() * 2.0 - 1.0
}

/// XNA `Rectangle.Intersects` — strict overlap (touching edges do not
/// intersect), the integer hitbox the C# reads.
fn hitbox_intersects(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    let (ax, ay, aw, ah) = (ax as i32, ay as i32, aw as i32, ah as i32);
    let (bx, by, bw, bh) = (bx as i32, by as i32, bw as i32, bh as i32);
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

// ---------------------------------------------------------------------------
// aiStyle 182 — AI_182_FinalFractal (Projectile.cs:43111-43176)
// ---------------------------------------------------------------------------

/// `FinalFractalHelper.GetFinalFractalProfile` (`FinalFractalHelper.cs:104-
/// 111` + the static table `:128-215`): the per-sword trail width
/// (`fullBladeLength / 2`) and colour. Only the width feeds the ported
/// dust-position maths; the colour feeds the skipped light/dust tints.
/// Unlisted ids return the default profile (50f blade → 25f width).
fn final_fractal_trail_width(sword_id: i32) -> f32 {
    let full = match sword_id {
        65 => 60.0,
        1123 | 368 | 674 | 3018 => 75.0,
        46 => 65.0,
        121 => 110.0,
        190 => 105.0,
        273 | 675 | 2880 | 3065 | 757 | 155 => 70.0,
        989 => 48.0,
        1826 | 3063 => 76.0,
        795 => 72.0,
        4144 | 3507 => 45.0,
        4956 => 86.0,
        _ => 50.0, // `_defaultProfile` (`:215`)
    };
    full / 2.0
}

/// The 182 arm's owner view: `Main.player[this.owner]`'s MountedCenter
/// (`:43122-43123`). The C# also reads `player.velocity` (`:43168`) but
/// only to feed the skipped dust body — the orbit itself reads the
/// projectile's own velocity, so the port carries the centre alone. The
/// dispatch fills it from the local player when the owner resolves;
/// `None` keeps the slot's own centre (the no-player fold).
pub(crate) struct FractalOwner {
    pub mounted_center: Vec2,
}

/// `AI_182_FinalFractal` (`Projectile.cs:43111-43176`) — the Zenith's
/// sword illusions (933/1100). The orbit: `localAI[0]` accrues the
/// speed-lerped rate, the centre rides the owner's MountedCenter plus the
/// velocity plus the rotating ellipse offset, the rotation follows the
/// orbit angle, and the opacity opens over 5 ticks / closes over the last
/// 5 of the 120-tick life. The kill at 120 RETURNS before any dust draw.
///
/// The `isAPreviewDisplayDoll` head (`:43113-43116`) has no port field —
/// the preview-doll display is unported, so the gate folds false. The
/// 933/1100 one-shot `SoundID.Item169` (`:43117-43121`) rides the
/// `wants_sound` seam (the `localAI[1]` latch is ported).
pub(crate) fn ai_final_fractal(
    p: &mut Projectile,
    owner: Option<&FractalOwner>,
    rng: &mut dyn DropRng,
) {
    // `:43117-43121` — the one-shot launch cue (both rowed kinds qualify).
    if (p.kind == 933 || p.kind == 1100) && p.local_ai[1] == 0.0 {
        p.local_ai[1] = 1.0;
        let c = center_of(p);
        p.wants_sound = Some((2, 169, c.x, c.y, 0.0, 1.0));
    }
    // `:43122-43123` — the owner reads. The C# reads `player.velocity`
    // only to feed the skipped dust (`:43168`); the orbit itself reads
    // the projectile's own velocity, so the owner velocity is unused here.
    let mounted_center = match owner {
        Some(o) => o.mounted_center,
        None => center_of(p),
    };
    // `:43124-43131` — the rate lerp and the 120-tick kill.
    let lerp_value = get_lerp_value(900.0, 0.0, p.vel.length() * 2.0);
    let num = lerp(0.7, 2.0, lerp_value);
    p.local_ai[0] += num;
    if p.local_ai[0] >= 120.0 {
        kill(p);
        return;
    }
    // `:43132-43144` — the orbit parameters.
    let lerp_value2 = get_lerp_value(0.0, 1.0, p.local_ai[0] / 60.0);
    let num2 = p.local_ai[0] / 60.0;
    let num3 = p.ai[0];
    let num4 = to_rotation(p.vel);
    let num5 = PI;
    let num6 = if p.vel.x > 0.0 { 1.0f32 } else { -1.0 };
    let num7 = num5 + num6 * lerp_value2 * (PI * 2.0);
    let mut num8 =
        p.vel.length() + get_lerp_value(0.5, 1.0, lerp_value2) * 40.0;
    let num9 = 60.0f32;
    if num8 < num9 {
        num8 = num9;
    }
    // `:43145-43148` — the centre: the ellipse offset rotated by the
    // flight angle, plus the closing counter-offset.
    let ellipse = Vec2::new(num8, num3 * lerp(2.0, 1.0, lerp_value));
    let vector = mounted_center
        + p.vel
        + rotated_by(rotated_by(Vec2::new(1.0, 0.0), num7 as f64) * ellipse, num4 as f64);
    let vector2 = (1.0 - get_lerp_value(0.0, 0.5, lerp_value2))
        * Vec2::new(num6 * (0.0 - num8) * 0.1, (0.0 - p.ai[0]) * 0.3);
    p.rot = num7 + num4 + PI / 2.0;
    set_center(p, vector + vector2);
    // `:43149-43155` — the facing, flipped for the negative-amplitude
    // (counter-rotating) half of the illusion pair.
    p.sprite_direction = if p.vel.x > 0.0 { 1 } else { -1 };
    p.direction = p.sprite_direction;
    if num3 < 0.0 {
        p.rot = num5 + num6 * lerp_value2 * (PI * -2.0) + num4;
        p.rot += PI / 2.0;
        p.sprite_direction = if !(p.vel.x > 0.0) { 1 } else { -1 };
        p.direction = p.sprite_direction;
    }
    // `:43156-43174` — the first-second dust trail.
    if num2 < 1.0 {
        fractal_dust_trail(p, lerp_value2, rng);
    }
    // `:43175` — the opacity window (5-tick open, last-5-tick close).
    let opacity = get_lerp_value(0.0, 5.0, p.local_ai[0])
        * get_lerp_value(120.0, 115.0, p.local_ai[0]);
    set_opacity(p, opacity);
    // `UpdatePosition`'s skip list (`:20435`) names 933/1100 — NO
    // integration after the AI (the dispatch's post-arm step is gated).
}

/// `AI_182_FinalFractal`'s dust-trail block (`Projectile.cs:43156-43174`):
/// the per-particle position lerp draw + the profile's `StripDust` gate
/// (`FinalFractalHelper.cs:39-53`). The two `AddLight` writes (`:43170-
/// 43173`) are the lighting skip. Called only while `localAI[0] < 60`.
fn fractal_dust_trail(p: &Projectile, lerp_value2: f32, rng: &mut dyn DropRng) {
    let trail_width = final_fractal_trail_width(p.ai[1] as i32);
    let vector3 = rotation_vector(p.rot - PI / 2.0);
    let center = center_of(p);
    // `:43161-43165` — the particle count, floored at 1.
    let mut num10 = ((1.0 + (p.vel.length() / 100.0) as i32 as f32)
        * get_lerp_value(0.0, 0.5, lerp_value2)
        * get_lerp_value(1.0, 0.5, lerp_value2)) as i32;
    if num10 < 1 {
        num10 = 1;
    }
    for _ in 0..num10 {
        // `:43168` — the position lerp draw (explicit, stays).
        let t = rng.next_float();
        let _pos = center + vector3 * trail_width * lerp(0.5, 1.0, t);
        // `StripDust`: the Next(9) gate ALWAYS draws; on a 0 the count
        // Next(1,4) and three floats per dust (colour lerp :46, fadeIn
        // :48, velocity length :50). The dust bodies are the layer's skip.
        if rng.next_range(0, 9) == 0 {
            let count = rng.next_range(1, 4);
            for _ in 0..count {
                let _ = rng.next_float(); // the colour lerpToWhite
                let _ = rng.next_float(); // fadeIn
                let _ = rng.next_float(); // the velocity length
            }
        }
    }
}

// ---------------------------------------------------------------------------
// aiStyle 183 — AI_183_ZoologistStrike (Projectile.cs:43095-43109)
// ---------------------------------------------------------------------------

/// `AI_183_ZoologistStrike` (`:43095-43109`) — the Zoologist's whip
/// slashes (880/929): the x-velocity decays ×0.2, the y zeroes, the
/// facing follows the x sign and the frame advances one per tick,
/// clamped at the sheet's last row (8 frames, `Main.cs:8694-8695`).
/// ZERO rand draws. `manualDirectionChange = true` (`:8449`/`:8815`)
/// keeps the post-movement direction rewrite off — the arm's own write
/// stands.
pub(crate) fn ai_zoologist_strike(p: &mut Projectile) {
    // `:43097-43098` — the velocity brake.
    p.vel.x *= 0.2;
    p.vel.y = 0.0;
    // `:43099-43103` — the facing pair.
    p.sprite_direction = 1;
    p.direction = 1;
    if p.vel.x < 0.0 {
        p.sprite_direction = -1;
        p.direction = -1;
    }
    // `:43104-43108` — the frame clamp at projFrames − 1.
    p.frame += 1;
    let frames = crate::projectile::proj_frames(p.kind) as i32;
    if p.frame >= frames {
        p.frame = frames - 1;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 184 — AI_184_BadTorchLuck (Projectile.cs:42787-42822)
// ---------------------------------------------------------------------------

/// `SelfHurtPlayers` (`Projectile.cs:15025-15062`) for the 949 — the
/// Torch God's fireball hurts its OWN owner when the biome-torch unlock
/// is missing. The 949 sets no `ownerHitCheck`/`trap`/gravestone row, so
/// the melee-line gate, the sweater halving and the gravestone index
/// override all fold out (module doc). `flag = !hostile && type != 949`
/// is FALSE for the 949 — the hurt rides `pvp: false` and the death
/// reason's player index −1 (the port's `hurt` carries no pvp lane; the
/// single-player fold, ANALYSIS). The C#'s post-hurt `StatusPlayer` gate
/// (`:15050-15053`) is a no-op for the 949 — it applies no debuff.
fn torch_god_self_hurt(
    p: &mut Projectile,
    player: Option<&mut crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    let Some(player) = player else { return };
    // `:15028` — the active/dead/immune/colliding gates. The port's
    // liveness is `hp > 0` + `!dead`; `immune` is the field; the hitbox
    // is the strict XNA overlap.
    if player.dead || player.hp <= 0 || player.immune {
        return;
    }
    let c = center_of(p);
    let (pcx, _) = player.center();
    if !hitbox_intersects(
        p.pos.x, p.pos.y, p.w, p.h,
        player.x, player.y, crate::player::WIDTH, player.height,
    ) {
        return;
    }
    // `:15032` — the knockback direction: away from the fireball.
    p.direction = if !(pcx < c.x) { 1 } else { -1 };
    // `:15033-15039` — the damage: no FixedSelfDamageForPlayers row for
    // 949 (`ProjectileID.cs:342-422`), so the shot's own damage rides
    // `DamageVar(damage, -luck)` (`Main.cs:66118-66145`).
    let num2 = tr_core::damage::damage_var(rng, p.damage as f32, -player.luck_state.luck);
    // `:15050` — `Hurt(dodgeable: IsDamageDodgeable(), ...)`; the 949 is
    // not in the 9999-damage exemption set, so dodgeable = true.
    player.hurt(num2, c.x, rng, true);
}

/// `AI_184_BadTorchLuck` (`Projectile.cs:42787-42822`) — the Torch God's
/// fireball (949). The owner-gated self-hurt (no biome-torch unlock), the
/// one-shot `Item8` cue, the `ai[1]` velocity-length drain that re-arms
/// `tileCollide` at zero, and the four-dust trail (one `Next(3)` gate per
/// dust; the dust bodies skip). The dust TYPE is `(int)ai[0]` — the
/// spawner passes the torch biome's dust id; the 66 disco-colour arm is
/// dust-only (skipped, its `Main.DiscoR/G/B` reads draw nothing).
///
/// BATCH-D MERGE-PENDING: the duplicate 949 machine adjudication is
/// keep-f33 (research/26g-clouds-keep-one.md §3/§3a) - f33's event wave
/// is NOT in this batch, so THIS interim machine stands until f33's
/// merge executes the A-L drop table.
pub(crate) fn ai_bad_torch_luck(
    p: &mut Projectile,
    mut player: Option<&mut crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:42789-42792` — the self-hurt gate: local owner + no unlock. The
    // player borrow is reborrowed for the read, then moved into the hurt.
    let unlocked = player.as_deref().is_some_and(|pl| pl.unlocked_biome_torches);
    if p.owner == local_owner && !unlocked {
        torch_god_self_hurt(p, player.as_deref_mut(), rng);
    }
    // `:42793-42797` — the one-shot magic-missile cue.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        let c = center_of(p);
        p.wants_sound = Some((2, 8, c.x, c.y, 0.0, 1.0));
    }
    // `:42798-42805` — the flight-distance drain; at zero the fireball
    // starts colliding with tiles.
    if p.ai[1] > 0.0 {
        p.ai[1] -= p.vel.length();
    } else {
        p.tile_collide = true;
    }
    // `:42806-42821` — the four-dust trail: the position steps along the
    // velocity, each dust rolls its `Next(3)` noGravity gate (explicit,
    // stays); the `velocity *= 0.3` and the 66 disco arm are dust-body
    // writes (skipped). ZERO further draws.
    let num = p.ai[0] as i32;
    let num2 = 4.0f32;
    let mut num3 = 0.0f32;
    while num3 < num2 {
        let _pos = p.pos + p.vel / num2 * num3;
        let _ = rng.next_range(0, 3); // the noGravity gate (`:42811`)
        num3 += 1.0;
    }
    let _ = num; // the dust type feeds only the skipped body
}

// ---------------------------------------------------------------------------
// aiStyle 185 — AI_185_LifeDrain (Projectile.cs:42631-42665)
// ---------------------------------------------------------------------------

/// `AI_185_LifeDrain` (`Projectile.cs:42631-42665`) — the Soul Drain
/// (476): on the FIRST tick only, and only for the local owner, scan for
/// a damageable NPC hitbox overlap (else a hostile PvP player — the
/// single-player fold refuses it, module doc); on a hit pay the held
/// item's mana (`CheckMana(pay: true)`, the ManaV2 machine never refuses)
/// and latch `localAI[0] = 1`. The drain's heal-over-time is the buff
/// layer's follow-up; the arm itself is the latch + the pay.
pub(crate) fn ai_life_drain(
    p: &mut Projectile,
    player: Option<&mut crate::player::Player>,
    npcs: &[Npc],
    local_owner: u8,
) {
    // `:42633-42636` — the owner + first-tick gate.
    if p.owner != local_owner || p.local_ai[0] != 0.0 {
        return;
    }
    let Some(player) = player else { return };
    // `:42638-42647` — the NPC scan: active, not town, not
    // dontTakeDamage, not friendly, hitbox overlap. The port's liveness
    // is `hp > 0`; the town gate rides `is_town_npc_like` (the vanilla
    // `townNPC` flag set).
    let mut flag = false;
    for n in npcs {
        if n.hp > 0
            && !n.kind.is_town_npc_like()
            && !n.dont_take_damage
            && !n.friendly
        {
            let (nw, nh) = n.size();
            if hitbox_intersects(p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
                flag = true;
                break;
            }
        }
    }
    // `:42648-42659` — the 255-player PvP scan: the port's single player
    // is the owner itself, and `whoAmI != player.whoAmI` refuses it — the
    // loop never fires (the documented single-player fold).
    // `:42660-42664` — the pay + the latch.
    if flag {
        let mana = crate::registry::item(player.held_item_kind)
            .map(|d| d.mana)
            .unwrap_or(0);
        crate::regen::check_mana(
            &mut player.mana,
            &mut player.slow_magic_use,
            mana,
            player.mana_cost,
            true,
        );
        p.local_ai[0] = 1.0;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 186 — AI_186_PrincessWeapon (Projectile.cs:42667-42785)
// ---------------------------------------------------------------------------

/// `AI_186_PrincessWeapon` (`Projectile.cs:42667-42785`) — the Resonance
/// Scepter's blast (950): the 60-tick life, the opacity fade, the
/// double-eased scale bloom and the `localAI[0]` draw-scale twin, then
/// the two 3-iteration dust rings (the `Next(2)` gates + the angle/fadeIn
/// floats stay, the dust bodies and their white clones skip), the
/// `Next(4)` particle gate (the orchestrator is the particle layer's
/// skip) and the tick-50 four-pair particle burst (draw-free).
pub(crate) fn ai_princess_weapon(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:42669-42675` — the 60-tick kill.
    let num = 60.0f32;
    p.ai[0] += 1.0;
    if p.ai[0] >= num {
        kill(p);
        return;
    }
    // `:42676-42685` — the fade, the eased scale and the draw-scale twin.
    set_opacity(p, remap(p.ai[0], 0.0, num, 1.0, 0.0));
    let num2 = p.ai[0] / num;
    let num3 = 1.0 - (1.0 - num2) * (1.0 - num2);
    let num4 = 1.0 - (1.0 - num3) * (1.0 - num3);
    let num5 = remap(p.ai[0], num - 15.0, num, 0.0, 1.0);
    let num6 = 1.0 - num5 * num5;
    p.scale = (0.4 + 0.6 * num4) * num6;
    let num7 = remap(p.ai[0], 20.0, num, 0.0, 1.0);
    let num8 = 1.0 - (1.0 - num7) * (1.0 - num7);
    p.local_ai[0] = (0.4 + 0.6 * (1.0 - (1.0 - num8) * (1.0 - num8))) * num6;
    // `:42686-42689` — the ring radius base (INTEGER width / 2) and the
    // dust constants. `newColor` (hslToRgb × Opacity) feeds only the
    // skipped dust tint — rand-free, no draw.
    let num9 = p.w as i32 / 2;
    let num10 = 6.0f32;
    let num11 = 2.0f32;
    // `:42690-42719` — the first dust ring (the +π/4 tangent, the −2
    // velocity). Each iteration: the Next(2) gate, on a hit the angle
    // NextFloat + the fadeIn NextFloat. The CloneDust white twin is a
    // dust-body copy (no further explicit draws).
    if num2 < 0.9 {
        for _ in 0..3 {
            if rng.next_range(0, 2) == 0 {
                let angle = rng.next_float() * (PI * 2.0);
                let vector = rotated_by(Vec2::new(1.0, 0.0), angle as f64);
                let _vector2 = vector * (num9 as f32 * p.scale);
                let _vector4 = rotated_by(vector, 0.7853981852531433);
                let _ = rng.next_float(); // fadeIn (`:42705`)
                let _ = num10;
                let _ = num11;
            }
        }
    }
    // `:42720-42748` — the second ring (the −6 offset, the +2 velocity;
    // the C# OVERWRITES velocity with `base.velocity * 0.5` at `:42738`
    // — a dust-body write, skipped with the body).
    if num2 < 0.9 {
        for _ in 0..3 {
            if rng.next_range(0, 2) == 0 {
                let angle = rng.next_float() * (PI * 2.0);
                let _vector5 = rotated_by(Vec2::new(1.0, 0.0), angle as f64);
                let _ = rng.next_float(); // fadeIn (`:42735`)
            }
        }
    }
    // `:42749-42764` — the particle sprinkle: ONE iteration (the float
    // loop `num14 = 0; num14 < 0.8; num14++` runs once), the Next(4) gate
    // stays; the RequestParticleSpawn is the particle layer's skip.
    if num2 < 0.95 {
        let mut num14 = 0.0f32;
        while num14 < 0.8 {
            if rng.next_range(0, 4) == 0 {
                let _angle = rng.next_float() * (PI * 2.0) + PI / 2.0;
            }
            num14 += 1.0;
        }
    }
    // `:42765-42784` — the tick-50 burst: four deterministic particle
    // pairs, ZERO draws (the orchestrator skip).
    if p.ai[0] == num - 10.0 {
        let mut num15 = 0.0f32;
        while num15 < 1.0 {
            let _vector10 = rotated_by(Vec2::new(1.0, 0.0), (PI * 2.0 * num15) as f64);
            num15 += 0.25;
        }
    }
}

// ---------------------------------------------------------------------------
// aiStyle 190 — AI_190_NightsEdge (Projectile.cs:41972-42227)
// ---------------------------------------------------------------------------

/// `Projectile.IsTileSolidForImpact` (`Projectile.cs:41848-41859`) — the
/// spore scan's tile gate: active, not actuated, `Main.tileSolid[type]`.
/// The `treatPlatformsAsNonSolid` parameter defaults false at every call
/// site here, so the platform arm is unreachable (kept out).
fn tile_solid_for_impact(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind)
    })
}

/// `FindChlorophyteSporeCloudSpawn` (`Projectile.cs:41810-41846`) — scan
/// the 6×5 tile window ahead of the swing (the x direction follows the
/// velocity sign) for the exposed solid top nearest the probe point; on
/// a hit the spawn position snaps to the column centre / the top edge.
/// Returns the adjusted position and whether a cell was found.
fn find_spore_cloud_spawn(
    p: &Projectile,
    world: &World,
    spawn_position: Vec2,
) -> (Vec2, bool) {
    let mut result = false;
    let mut num = f32::MAX;
    let vector = spawn_position;
    let mut out = spawn_position;
    let num2 = (spawn_position.x / 16.0) as i32;
    let num3 = (spawn_position.y / 16.0) as i32;
    let vector2 = Vec2::new((num2 * 16 + 8) as f32, (num3 * 16 + 8) as f32);
    // `:41820-41827` — the scan window leans AGAINST the flight: a
    // rightward swing scans 5 columns left, a leftward one 5 right.
    let (num4, num5) = if p.vel.x > 0.0 { (-5, 0) } else { (0, 5) };
    for i in num4..=num5 {
        for j in -2..=2 {
            if tile_solid_for_impact(world, num2 + i, num3 + j)
                && !tile_solid_for_impact(world, num2 + i, num3 + j - 1)
            {
                let cell = Vec2::new(
                    ((num2 + i) * 16 + 8) as f32,
                    ((num3 + j) * 16 - 8) as f32,
                );
                let num6 = (vector - cell).length();
                if num6 < num {
                    out.x = vector2.x;
                    out.y = ((num3 + j) * 16 - 8) as f32;
                    num = num6;
                    result = true;
                }
            }
        }
    }
    (out, result)
}

/// One 1128 spore-cloud spawn request the pool resolves after the AI
/// (`wants_dart_spawns`'s shape): `(x, y, vx, vy, damage, ai1)`. The C#
/// `NewProjectile(..., 1128, damage, 0f, owner)` — knockback 0, ai0 0,
/// the owner/damage ride the parent slot. The ground-bloom clouds
/// (`:41944`) pass no ai1 (0); the sweep cloud (`:42172`) passes
/// `num26 = 90 + Next(60)` as ai1 (the cloud's lifetime seed).
pub(crate) type SporeSpawn = (f32, f32, f32, f32, i32, f32);

/// `SpawnChlorophyteSporeCloud` (`Projectile.cs:41861-41970`) — the
/// Chlorophyte Claymore's ground-bloom: walk the tile columns around the
/// impact, count the exposed tops, roll the per-column spawn gate and
/// queue the 1128 clouds; on any spawn the 1129 blade-slam intent rides
/// out with it (the `wants_cloud_segment`-style payload, resolved by the
/// dispatch). The dust rows' EXPLICIT fadeIn/scale floats stay; the
/// `KillTile_MakeTileDust` internal rolls are the pool-occupancy skip
/// (module doc). `owned_1128` is the owner's live spore count
/// (`ownedProjectileCounts[1128]`, `:41874`) — the spawn-chance remap.
/// Returns the queued clouds and the optional slam `(x, y, damage, ai1,
/// ai2)` — the 1129 spawn (`:41967`) passes ai1 = 128 / ai2 = 96.
fn spawn_spore_cloud(
    p: &mut Projectile,
    world: &World,
    spawn_position: Vec2,
    local_owner: u8,
    owned_1128: i32,
    player_direction: i32,
    rng: &mut dyn DropRng,
) -> (Vec<SporeSpawn>, Option<(f32, f32, i32, f32, f32)>) {
    let mut clouds = Vec::new();
    // `:41863-41872` — the column window (the ±2 row band, the 5-column
    // lean against the flight).
    let mut px = (spawn_position.x as i32) / 16;
    let mut px2 = (spawn_position.x as i32) / 16;
    let py = (spawn_position.y as i32) / 16 - 2;
    let py2 = (spawn_position.y as i32) / 16 + 2;
    if p.vel.x > 0.0 {
        px -= 5;
    } else {
        px2 += 5;
    }
    // `:41873-41875` — the spawn chance: 1 at ≤10 owned clouds, 0.2 at
    // ≥100, the clamped remap between.
    let mut flag = false;
    let num = remap(owned_1128 as f32, 10.0, 100.0, 1.0, 0.2);
    for i in px..=px2 {
        // `:41878-41888` — the exposed-top count (local owner only).
        let mut num3 = 0;
        if p.owner == local_owner {
            for j in py..=py2 {
                if tile_solid_for_impact(world, i, j)
                    && !tile_solid_for_impact(world, i, j - 1)
                {
                    num3 += 1;
                }
            }
        }
        for k in py..=py2 {
            // `:41891-41946` — the exposed-top cell bloom (its own helper).
            if spore_cell_bloom(p, world, i, k, local_owner, num, num3, player_direction, rng, &mut clouds) {
                flag = true;
            }
        }
    }
    // `:41949-41969` — the blade slam: the offset point ahead of the
    // swing (the 0.8-progress angle), lifted 48 and shifted by the
    // facing; the 1129 spawn carries ai0 = 128 / ai1 = 96 and the FULL
    // damage. The Item167 cue rides the sound seam.
    let mut slam = None;
    if flag {
        let num9 = (170.0 * 0.5 - 4.0) * p.scale - 16.0;
        let vector5 = rotation_vector(p.rot + remap(0.8, 0.0, 1.0, 0.0, PI / 4.0) * p.ai[0]);
        let mut vector6 = center_of(p) + vector5 * num9;
        vector6.y -= 48.0;
        if vector5.x > 0.0 {
            vector6.x -= 112.0;
        }
        if vector5.x > 0.0 {
            vector6.x += 48.0;
        } else {
            vector6.x -= 48.0;
        }
        slam = Some((vector6.x, vector6.y, p.damage, 128.0, 96.0));
        p.wants_sound = Some((2, 167, spawn_position.x, spawn_position.y, 0.0, 1.0));
    }
    (clouds, slam)
}

/// `SpawnChlorophyteSporeCloud`'s per-cell body (`Projectile.cs:41891-
/// 41946`): only an exposed solid top blooms. The two tile-dust rows'
/// EXPLICIT velocity.Y floats stay (the `KillTile_MakeTileDust` internals
/// are the pool-occupancy skip), then the 25-dust ring's fadeIn floats.
/// Non-local owners bloom no clouds. A lone exposed top (`num3 == 1`)
/// spawns TWO clouds, otherwise one; each rolls the chance gate then the
/// velocity/position draws IN ORDER. Returns whether the cell bloomed
/// (the parent's slam flag).
fn spore_cell_bloom(
    p: &Projectile,
    world: &World,
    i: i32,
    k: i32,
    local_owner: u8,
    chance: f32,
    num3: i32,
    player_direction: i32,
    rng: &mut dyn DropRng,
    clouds: &mut Vec<SporeSpawn>,
) -> bool {
    // `:41891-41895` — only exposed solid tops bloom.
    let Some(t) = world.tile(i, k) else { return false };
    if !tile_solid_for_impact(world, i, k) || tile_solid_for_impact(world, i, k - 1) {
        return false;
    }
    // `:41896-41914` — the two tile-dust rows: the count is
    // `KillTile_GetTileDustAmount(fail: true) * 6`; each dust's EXPLICIT
    // velocity.Y NextFloat stays. `num2 = 4 >= 2` always, so the second
    // row (amount − 1) always runs.
    let num4 = crate::collision::tile_dust_amount(t.kind, true, t.frame_x) * 6;
    for _ in 0..num4 {
        let _ = rng.next_float(); // velocity.Y *= NextFloat (`:41901`)
    }
    for _ in 0..(num4 - 1) {
        let _ = rng.next_float(); // the second row (`:41911`)
    }
    // `:41915-41930` — the 25-dust ring: one fadeIn NextFloat each
    // (`:41928`); the ring positions are deterministic.
    for _ in 0..25 {
        let _ = rng.next_float();
    }
    // `:41931-41934` — non-local owners bloom no clouds.
    if p.owner != local_owner {
        return false;
    }
    // `:41936-41946` — the cloud spawn(s).
    let num6 = if num3 != 1 { 1 } else { 2 };
    for n in 0..num6 {
        if !(rng.next_float() > chance) {
            let num7 = -0.25 * p.vel.x;
            let num8 = 0.25 * p.vel.x - num7;
            // `:41943` — the jitter reads the OWNER's direction
            // (`Main.player[owner].direction`), not the swing's.
            let vx = rng.next_float() * num8 + num7
                + (player_direction * 4) as f32 * rng.next_float();
            let vy = rng.next_range(-45, 0) as f32 * 0.1 * 2.3;
            let x = (i * 16) as f32 + rng.next_float() * 12.0 + 4.0;
            let y = (k * 16 - (4 * n - 4)) as f32;
            clouds.push((x, y, vx, vy, (p.damage as f32 * 0.75) as i32, 0.0));
        }
    }
    true
}

/// The 1126 spore machine's per-tick output: the queued 1128 clouds and
/// the optional 1129 slam `(x, y, damage, ai1, ai2)`. The dispatch drains
/// them into the pool.
pub(crate) struct ClaymoreSpores {
    pub clouds: Vec<SporeSpawn>,
    pub slam: Option<(f32, f32, i32, f32, f32)>,
}

/// The 1126 Chlorophyte Claymore's spore half (`AI_190_NightsEdge`
/// `:42078-42182`), split out so the main arm stays a readable window.
/// `num` is the swing progress (`localAI[0] / ai[1]`), `num9`/`num10` the
/// C#'s 0.8 window / 170 reach constants. The vertical-swing latch
/// (`localAI[2]`) gates the ground bloom; the horizontal sweep rolls the
/// melee-speed spawn gate and queues the 1128 clouds directly.
fn claymore_spore_tick(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    local_owner: u8,
    num: f32,
    rng: &mut dyn DropRng,
) -> ClaymoreSpores {
    let mut out = ClaymoreSpores { clouds: Vec::new(), slam: None };
    let num9 = 0.8f32;
    let num10 = 170;
    // The owner reads the bloom needs (`:41874`/`:41943`): the live
    // spore count and the facing. Without a player both fold to 0 / 1.
    let owned = player.map(|pl| pl.owned_projectile_counts[1128]).unwrap_or(0);
    let player_direction = player.map(|pl| pl.facing).unwrap_or(1);
    // `:42082-42145` — the vertical swing (the spawn-time velocity.Y == 0
    // latch): the dust sprinkle, then the 0.8-progress ground bloom.
    if p.local_ai[2] == 1.0 {
        let num11 = p.rot + next_float_direction(rng) * (PI / 2.0) * 0.7;
        let _vector5 = rotation_vector(num11 + p.ai[0] * (PI / 2.0));
        // `:42087-42092` — the opacity-gated dust (one NextFloat gate;
        // the taken branch's NewDustPerfect 157 body skips, no explicit
        // inner draws).
        if rng.next_float() * 2.0 < opacity_of(p) {
            let _ = rng.next_float(); // the radial position (`:42089`)
        }
        let num12 = (p.local_ai[0] - 1.0) / p.ai[1];
        let num13 = (num10 as f32 * 0.5 - 4.0) * p.scale;
        // `:42095-42132` — the bloom trigger: the progress crosses 0.8
        // this tick. The find-or-fan search rides its own helper.
        if num >= num9 && num12 < num9 {
            let (clouds, slam) = claymore_bloom_search(
                p, world, local_owner, owned, player_direction, num13, rng,
            );
            out.clouds = clouds;
            out.slam = slam;
        }
        // `:42133-42144` — the pre-bloom dust arc: four dusts at the
        // lerped reach point, each with the NextVector2Square jitter
        // (TWO NextFloat draws, X then Y, `Utils.cs:1251-1254`).
        if num <= num9 {
            let num19 = lerp(num13 * 0.5, num13, num / num9);
            let num20 = remap(num / num9, 0.0, 1.0, -PI / 4.0, 0.0);
            let _vector6 = center_of(p)
                + rotation_vector(
                    p.rot + (num20 + remap(num, 0.0, 1.0, 0.0, PI / 4.0)) * p.ai[0],
                ) * num19;
            for _ in 0..4 {
                let _ = rng.next_float(); // NextVector2Square X
                let _ = rng.next_float(); // NextVector2Square Y
            }
        }
    }
    // `:42146-42175` — the sweep spawn (both swing orientations): while
    // the progress is inside the 0.8 window, roll the melee-speed gate
    // and queue a 1128 at the arc point. LOCAL OWNER ONLY.
    if num <= num9 {
        if let Some(cloud) = claymore_sweep_spawn(p, player, local_owner, num, rng) {
            out.clouds.push(cloud);
        }
    }
    // `:42176-42181` — the green AddLight: the lighting skip.
    out
}

/// `AI_190_NightsEdge`'s ground-bloom search (`Projectile.cs:42095-42132`):
/// the probe point at the 0.8-angle on the reach ring, then the
/// `FindChlorophyteSporeCloudSpawn` probe; on a miss the fan search walks
/// ±20/±12 π/64 steps (the −step side first each iteration). On a hit the
/// bloom queues its clouds + slam. `num13` is the reach radius
/// `(170·0.5 − 4)·scale`.
fn claymore_bloom_search(
    p: &mut Projectile,
    world: &World,
    local_owner: u8,
    owned: i32,
    player_direction: i32,
    num13: f32,
    rng: &mut dyn DropRng,
) -> (Vec<SporeSpawn>, Option<(f32, f32, i32, f32, f32)>) {
    let num9 = 0.8f32;
    let num14 = num13 - 4.0;
    let num15 = PI / 64.0;
    let num16 = num15 * 20.0;
    let num17 = num15 * 12.0;
    let base_angle = p.rot + remap(num9, 0.0, 1.0, 0.0, PI / 4.0) * p.ai[0];
    let spawn_position = center_of(p) + rotation_vector(base_angle) * num14;
    let (adjusted, found) = find_spore_cloud_spawn(p, world, spawn_position);
    if found {
        return spawn_spore_cloud(p, world, adjusted, local_owner, owned, player_direction, rng);
    }
    // `:42108-42130` — the fan search.
    let mut num18 = 0.0f32;
    while num18 <= num16 || num18 <= num17 {
        num18 += num15;
        if num18 <= num16 {
            let pos = center_of(p)
                + rotation_vector(p.rot + (remap(num9, 0.0, 1.0, 0.0, PI / 4.0) - num18) * p.ai[0])
                    * num14;
            let (adjusted, found) = find_spore_cloud_spawn(p, world, pos);
            if found {
                return spawn_spore_cloud(p, world, adjusted, local_owner, owned, player_direction, rng);
            }
        }
        if num18 <= num17 {
            let pos = center_of(p)
                + rotation_vector(p.rot + (remap(num9, 0.0, 1.0, 0.0, PI / 4.0) + num18) * p.ai[0])
                    * num14;
            let (adjusted, found) = find_spore_cloud_spawn(p, world, pos);
            if found {
                return spawn_spore_cloud(p, world, adjusted, local_owner, owned, player_direction, rng);
            }
        }
    }
    (Vec::new(), None)
}

/// `AI_190_NightsEdge`'s sweep spawn (`Projectile.cs:42146-42175`): the
/// arc point at the lerped reach, the melee-speed `Next` gate (the two
/// progress thirds force it), and the 1128 queue with ai1 = 90 + Next(60).
/// Returns the cloud payload or `None` (non-local owner / no player / the
/// gate refused).
fn claymore_sweep_spawn(
    p: &Projectile,
    player: Option<&crate::player::Player>,
    local_owner: u8,
    num: f32,
    rng: &mut dyn DropRng,
) -> Option<SporeSpawn> {
    let num9 = 0.8f32;
    let num10 = 170;
    let num21 = p.rot + next_float_direction(rng) * (PI / 2.0) * 0.7;
    let vector7 = rotation_vector(num21 + p.ai[0] * (PI / 2.0));
    let num22 = (num10 as f32 * 0.5 - 4.0) * p.scale;
    let num23 = lerp(num22 * 0.5, num22, num / num9);
    let num24 = remap(num / num9, 0.0, 1.0, -PI / 4.0, 0.0);
    let vector8 = center_of(p)
        + rotation_vector(p.rot + (num24 + remap(num, 0.0, 1.0, 0.0, PI / 4.0)) * p.ai[0])
            * num23;
    if p.owner != local_owner {
        return None;
    }
    let pl = player?;
    let num25 = 0.75f32;
    let vector9 = Vec2::new(pl.vx, pl.vy) * 0.77 + vector7 * 5.0;
    // `:42160` — the melee-speed gate: the Remap of the inverted
    // meleeSpeed (1 → 4, 1/3 → 2), truncated, is the Next bound; a 0 roll
    // spawns. The two progress thirds force the spawn regardless.
    let melee = pl.melee_speed(pl.held_item_kind);
    let bound = remap(melee, 1.0, 1.0 / 3.0, 4.0, 2.0) as i32;
    let mut spawn_flag = rng.next_range(0, bound.max(1)) == 0;
    let num26 = 90 + rng.next_range(0, 60);
    if p.local_ai[0] as i32 == (p.local_ai[1] as i32) / 3 {
        spawn_flag = true;
    }
    if p.local_ai[0] as i32 == (p.local_ai[1] / 3.0) as i32 * 2 {
        spawn_flag = true;
    }
    if !spawn_flag {
        return None;
    }
    // `:42172` — the 1128 spawn: ai1 = num26 (the cloud's lifetime seed),
    // damage × 0.75. The seed rides the spawn payload — the PARENT's
    // localAI[1] is the fixed swing duration the progress-thirds gates
    // read, never overwritten here.
    Some((
        vector8.x,
        vector8.y,
        vector9.x,
        vector9.y,
        (p.damage as f32 * num25) as i32,
        num26 as f32,
    ))
}

/// `AI_190_NightsEdge` (`Projectile.cs:41972-42227`) — the seven
/// big-sword slashes. The shared spine: the first-tick cues (984's
/// Item60, 1126's vertical-swing latch), the `localAI[0]` age, the
/// progress-scaled rotation around the owner's rotated hand, the
/// per-kind scale pair, the centre pin at `RotatedRelativePoint −
/// velocity`, the ×ai[2] scale and the progress-1 kill. The per-kind
/// dust gates keep their explicit draws (module doc); the 972/1043
/// thrust extension and the 1126 spore machine ride their own blocks.
///
/// `player` is the owner's player view (the dispatch resolves it for
/// owner == local_owner); `None` keeps the slot's own centre as the
/// anchor and skips the player-read gates (the no-player fold).
pub(crate) fn ai_nights_edge(
    p: &mut Projectile,
    world: &World,
    player: Option<&mut crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> ClaymoreSpores {
    let mut spores = ClaymoreSpores { clouds: Vec::new(), slam: None };
    // `:41974-41989` — the first-tick block.
    if p.local_ai[0] == 0.0 {
        if p.kind == 984 {
            // The Item60 cue; the post-play ×0.65 volume scale is the
            // S1-E-21h deferred-scale deviation.
            p.wants_sound = Some((2, 60, p.pos.x, p.pos.y, 0.0, 1.0));
        }
        if p.kind == 1126 {
            p.local_ai[2] = if p.vel.y == 0.0 { 1.0 } else { 0.0 };
            p.vel.y = 0.0;
        }
    }
    // `:41990-41994` — the age, the progress, the rotation. The owner
    // reads fold to the slot centre / zero rotation without a player.
    p.local_ai[0] += 1.0;
    let num = p.local_ai[0] / p.ai[1];
    let num2 = p.ai[0];
    let full_rotation = player.as_deref().map(|pl| pl.full_rotation).unwrap_or(0.0);
    p.rot = PI * num2 * num + to_rotation(p.vel) + num2 * PI + full_rotation;
    // `:41995-42016` — the per-kind scale pair.
    let mut num3 = 0.2f32;
    let mut num4 = 1.0f32;
    match p.kind {
        982 | 997 | 984 => num3 = 0.6,
        1126 => {
            num3 = 0.6;
            num4 = 0.85;
        }
        983 => {
            num3 = 1.0;
            num4 = 1.2;
        }
        _ => {}
    }
    // `:42017-42018` — the centre pin and the progress scale.
    let anchor = match player.as_deref() {
        Some(pl) => {
            let (mcx, mcy) = pl.mounted_center();
            pl.rotated_relative_point(Vec2::new(mcx, mcy), 0)
        }
        None => center_of(p),
    };
    set_center(p, anchor - p.vel);
    p.scale = num4 + num * num3;
    // `:42019-42049` — the 972 Night's Edge / 1043 Flint thrust extension
    // + the 972 dust sprinkle.
    if p.kind == 972 {
        nights_edge_thrust(p, num, num2);
        nights_edge_972_dust(p, rng);
    }
    if p.kind == 1043 && num2.abs() < 0.2 {
        nights_edge_thrust(p, num, num2);
    }
    // `:42050-42077` — the 982 Excalibur / 997 Horseman's Blade dusts.
    if p.kind == 982 {
        excalibur_dust(p, rng, 2.0, 1.5);
    }
    if p.kind == 997 {
        horseman_dust(p, rng);
    }
    // `:42078-42182` — the 1126 Chlorophyte Claymore's spore machine.
    if p.kind == 1126 {
        spores = claymore_spore_tick(p, world, player.as_deref(), local_owner, num, rng);
    }
    // `:42183-42198` — the 983 True Excalibur: the HotPink twin of 982
    // (the first gate's multiplier is 1, not 2).
    if p.kind == 983 {
        excalibur_dust(p, rng, 1.0, 1.5);
    }
    // `:42199-42221` — the 984 Terra Blade: the colour-ramped dust pair
    // (the AddLight at `:42206` is the lighting skip).
    if p.kind == 984 {
        terra_blade_dust(p, rng);
    }
    // `:42222-42226` — the spawn-scale multiplier and the progress kill.
    p.scale *= p.ai[2];
    if num >= 1.0 {
        kill(p);
    }
    spores
}

/// `AI_190_NightsEdge`'s small-amplitude thrust extension
/// (`Projectile.cs:42021-42027` / `:42043-42049`): while `|ai[0]| < 0.2`
/// the rotation spins extra, the position pushes along the normalized
/// velocity by the 10→(ai[1]−5) remap × 45, and the scale gains × 0.4.
/// Shared by the 972 and the 1043.
fn nights_edge_thrust(p: &mut Projectile, num: f32, num2: f32) {
    if num2.abs() < 0.2 {
        p.rot += PI * 4.0 * num2 * 10.0 * num;
        let t = remap(p.local_ai[0], 10.0, p.ai[1] - 5.0, 0.0, 1.0);
        p.pos += safe_normalize(p.vel) * (45.0 * t);
        p.scale += t * 0.4;
    }
}

/// The 972 Night's Edge dust sprinkle (`Projectile.cs:42028-42041`): the
/// `Next(2)` gate, on a hit the direction float + the `Next(5)` dust-14
/// gate + the `Next(2)` dust-27 gate. The dust bodies skip.
fn nights_edge_972_dust(p: &Projectile, rng: &mut dyn DropRng) {
    if rng.next_range(0, 2) == 0 {
        let f = p.rot + next_float_direction(rng) * (PI / 2.0) * 0.7;
        let _vector = center_of(p) + rotation_vector(f) * 84.0 * p.scale;
        if rng.next_range(0, 5) == 0 {
            // Dust 14 (noLight/noLightEmittance) — body skip.
        }
        if rng.next_range(0, 2) == 0 {
            // Dust 27 (the owner-velocity kick) — body skip.
        }
    }
}

/// The 982 Excalibur / 983 True Excalibur dust pair
/// (`Projectile.cs:42050-42065` / `:42183-42198`): the direction float,
/// the first opacity gate (×`gate1`) with its three inner floats, then
/// the second opacity gate (×`gate2`, the dust-43 body skip). The 982
/// passes (2, 1.5); the 983 (1, 1.5).
fn excalibur_dust(p: &Projectile, rng: &mut dyn DropRng, gate1: f32, gate2: f32) {
    let angle = p.rot + next_float_direction(rng) * (PI / 2.0) * 0.7;
    let _vector2 = center_of(p) + rotation_vector(angle) * 84.0 * p.scale;
    let _vector3 = rotation_vector(angle + p.ai[0] * (PI / 2.0));
    if rng.next_float() * gate1 < opacity_of(p) {
        let _ = rng.next_float(); // the radial position
        let _ = rng.next_float(); // the colour lerpToWhite
        let _ = rng.next_float(); // fadeIn
    }
    if rng.next_float() * gate2 < opacity_of(p) {
        // Dust 43 — body skip, no explicit inner draws.
    }
}

/// The 997 Horseman's Blade dust (`Projectile.cs:42066-42077`): the
/// direction float + ONE opacity gate (×2) with its radial-position
/// float. The dust-6 body skips.
fn horseman_dust(p: &Projectile, rng: &mut dyn DropRng) {
    let angle = p.rot + next_float_direction(rng) * (PI / 2.0) * 0.7;
    let _vector4 = rotation_vector(angle + p.ai[0] * (PI / 2.0));
    if rng.next_float() * 2.0 < opacity_of(p) {
        let _ = rng.next_float(); // the radial position (`:42073`)
    }
}

/// The 984 Terra Blade dust pair (`Projectile.cs:42199-42221`): the
/// direction float, the ×2 opacity gate with its three inner floats,
/// then the ×1 opacity gate whose dead colour expression still draws one
/// float (`:42215`). The AddLight (`:42206`) and the dust-107 body skip.
fn terra_blade_dust(p: &Projectile, rng: &mut dyn DropRng) {
    let angle = p.rot + next_float_direction(rng) * (PI / 2.0) * 0.7;
    let _vector12 = center_of(p) + rotation_vector(angle) * 85.0 * p.scale;
    let _vector13 = rotation_vector(angle + p.ai[0] * (PI / 2.0));
    if rng.next_float() * 2.0 < opacity_of(p) {
        let _ = rng.next_float(); // the radial position (`:42209`)
        let _ = rng.next_float(); // the colour lerpToWhite
        let _ = rng.next_float(); // fadeIn
    }
    if rng.next_float() < opacity_of(p) {
        // `:42215` — the C# computes a colour and DISCARDS it (a dead
        // expression); the NextFloat inside it still draws.
        let _ = rng.next_float();
        // Dust 107's velocity kicks read the player — body skip.
    }
}

// ---------------------------------------------------------------------------
// Witnesses — WRITTEN NOT RUN (code-only mode, the module doc).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::proj_data;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-recording `Main.rand` stand-in (the `projectile_ai_misc`
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

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    /// A slot carrying its `SetDefaults` row slice (the
    /// `projectile_ai_misc` fixture shape).
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
        p.melee = d.melee;
        p.magic = d.magic;
        p.ignore_water = d.ignore_water;
        p.uses_local_immunity = d.uses_local_immunity;
        p.local_hit_cooldown = d.local_hit_cooldown;
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

    /// The thirteen new rows keep their census styles + the `SetDefaults`
    /// slices the arms read (the data half of the task).
    #[test]
    fn w4_d19_the_rows_carry_their_styles_and_slices() {
        assert_eq!(proj_data(SOUL_DRAIN_ID).unwrap().ai_style, 185);
        assert_eq!(proj_data(SOUL_DRAIN_ID).unwrap().time_left, 3);
        assert_eq!(proj_data(SOUL_DRAIN_ID).unwrap().alpha, 255);
        assert!(proj_data(SOUL_DRAIN_ID).unwrap().magic);
        for k in [880u16, 929] {
            let d = proj_data(k).unwrap();
            assert_eq!(d.ai_style, 183, "kind {k}");
            assert_eq!(d.time_left, 18);
            assert!(!d.tile_collide);
            assert!(d.ignore_water);
        }
        for k in [933u16, 1100] {
            let d = proj_data(k).unwrap();
            assert_eq!(d.ai_style, 182, "kind {k}");
            assert_eq!(d.extra_updates, 1);
            assert_eq!(d.local_hit_cooldown, 7, "15 / MaxUpdates(2)");
            assert!(d.melee);
            assert_eq!(d.alpha, 255);
        }
        let torch = proj_data(949).unwrap();
        assert_eq!(torch.ai_style, 184);
        assert_eq!((torch.w, torch.h), (4.0, 4.0));
        assert!(!torch.ignore_water, "the 949 sets none");
        let princess = proj_data(950).unwrap();
        assert_eq!(princess.ai_style, 186);
        assert_eq!((princess.w, princess.h), (104.0, 104.0));
        assert_eq!(princess.time_left, 180);
        assert!(princess.magic);
        for (k, pen) in [(972u16, 2), (982, 3), (983, 6), (984, 3), (997, 3), (1043, 2), (1126, -1)] {
            let d = proj_data(k).unwrap();
            assert_eq!(d.ai_style, 190, "kind {k}");
            assert_eq!(d.penetrate, pen, "kind {k}");
            assert!(d.melee);
            assert!(d.ignore_water);
            assert!(d.uses_local_immunity);
            assert_eq!(d.local_hit_cooldown, -1);
        }
        // The Zoologist sheets slice 8 frames (`Main.cs:8694-8695`).
        assert_eq!(crate::projectile::proj_frames(880), 8);
        assert_eq!(crate::projectile::proj_frames(929), 8);
    }

    // The kind ids as the witnesses name them (the data constants are
    // re-exported through `crate::projectile`; the local aliases keep the
    // test bodies readable).
    const SOUL_DRAIN_ID: u16 = 476;

    /// Style 183 (`:43097-43108`): the x brake, the y zero, the facing and
    /// the frame clamp at 7. ZERO draws.
    #[test]
    fn w4_d19_the_zoologist_strike_brakes_and_clamps_the_frame() {
        let mut p = proj(880);
        p.vel = Vec2::new(-10.0, 5.0);
        p.frame = 6;
        ai_zoologist_strike(&mut p);
        assert_eq!(p.vel.x, -2.0, "×0.2 (`:43097`)");
        assert_eq!(p.vel.y, 0.0, "(`:43098`)");
        assert_eq!(p.direction, -1);
        assert_eq!(p.sprite_direction, -1);
        assert_eq!(p.frame, 7, "6 + 1 (`:43104`)");
        ai_zoologist_strike(&mut p);
        assert_eq!(p.frame, 7, "clamped at projFrames − 1 (`:43105-43107`)");
        // The rightward flight flips the facing positive.
        let mut q = proj(929);
        q.vel = Vec2::new(10.0, 0.0);
        ai_zoologist_strike(&mut q);
        assert_eq!(q.direction, 1);
        assert_eq!(q.vel.x, 2.0);
    }

    /// Style 185 (`:42633-42664`): the first-tick NPC-overlap latch pays
    /// the held item's mana; a second tick (or a non-owner, or no overlap)
    /// never re-latches.
    #[test]
    fn w4_d19_the_soul_drain_latches_once_and_pays_mana() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 3006; // the Soul Drain item (`Item.cs:27744-27760`, mana 10)
        pl.mana = 100;
        pl.mana_cost = 1.0;
        let mut p = proj(476);
        // An NPC overlapping the 200×200 box.
        let npcs = [npc_at(p.pos.x + 10.0, p.pos.y + 10.0)];
        ai_life_drain(&mut p, Some(&mut pl), &npcs, 0);
        assert_eq!(p.local_ai[0], 1.0, "the latch (`:42663`)");
        let paid = 100 - pl.mana;
        assert!(paid > 0, "CheckMana(pay: true) drained the bar");
        // The second tick refuses (localAI[0] != 0).
        let before = pl.mana;
        ai_life_drain(&mut p, Some(&mut pl), &npcs, 0);
        assert_eq!(pl.mana, before, "no second pay");
        // A fresh slot with NO overlap never latches.
        let mut q = proj(476);
        let far = [npc_at(5000.0, 5000.0)];
        ai_life_drain(&mut q, Some(&mut pl), &far, 0);
        assert_eq!(q.local_ai[0], 0.0);
        // A non-owner refuses outright.
        let mut r = proj(476);
        r.owner = 1;
        ai_life_drain(&mut r, Some(&mut pl), &npcs, 0);
        assert_eq!(r.local_ai[0], 0.0);
    }

    /// Style 186 (`:42669-42685`, `:42765`): the 60-tick kill, the opacity
    /// fade, the eased scale and the tick-50 burst gate. The two dust
    /// rings roll 3 `Next(2)` gates each while `ai[0]/60 < 0.9`.
    #[test]
    fn w4_d19_the_princess_blast_fades_and_dies_at_60() {
        let mut p = proj(950);
        let mut rng = census();
        ai_princess_weapon(&mut p, &mut rng);
        assert_eq!(p.ai[0], 1.0);
        assert!(p.active);
        // Opacity = Remap(1, 0, 60, 1, 0) = 1 − 1/60.
        let expected_alpha = ((1.0 - (1.0 - 1.0 / 60.0)) * 255.0) as i32;
        assert_eq!(p.alpha, expected_alpha);
        // The scale bloom is positive and under 1 on tick 1.
        assert!(p.scale > 0.0 && p.scale < 1.0);
        // Two 3-iteration rings: 6 Next(2) gates (CensusRng returns 0 →
        // every gate taken → 2 floats each = 12 floats), plus the Next(4)
        // particle gate (taken → 1 angle float).
        assert_eq!(rng.ranges.len(), 7, "6 ring gates + 1 particle gate");
        assert!(rng.ranges.iter().all(|&(a, b)| (a, b) == (0, 2) || (a, b) == (0, 4)));
        // Run to the kill.
        for _ in 0..59 {
            ai_princess_weapon(&mut p, &mut rng);
        }
        assert_eq!(p.ai[0], 60.0);
        assert!(!p.active, "the 60-tick kill (`:42671-42674`)");
        assert_eq!(p.time_left, 0);
    }

    /// Style 182 (`:43124-43131`, `:43175`): the rate accrual, the 120-tick
    /// kill and the opacity window. The dust loop draws while
    /// `localAI[0] < 60`.
    #[test]
    fn w4_d19_the_fractal_orbit_accrues_and_dies_at_120() {
        let mut p = proj(933);
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 1.0; // the positive-amplitude half
        p.ai[1] = 190.0; // the Night's Edge profile (105f blade)
        let owner = FractalOwner {
            mounted_center: Vec2::new(1600.0, 800.0),
        };
        let mut rng = census();
        ai_final_fractal(&mut p, Some(&owner), &mut rng);
        // rate = Lerp(0.7, 2, GetLerpValue(900, 0, 20)) ≈ 0.7 + 1.3·(1−20/900).
        let lerp_value = get_lerp_value(900.0, 0.0, 20.0);
        let rate = lerp(0.7, 2.0, lerp_value);
        assert!((p.local_ai[0] - rate).abs() < 1e-4);
        assert!(p.active);
        // The one-shot cue latched (at the slot's OWN centre — the C#
        // reads base.Center at `:43120`, before the orbit repositions it).
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.wants_sound, Some((2, 169, 1616.0, 816.0, 0.0, 1.0)));
        // The dust loop drew (localAI[0] < 60): at least the position
        // float + the Next(9) gate.
        assert!(rng.floats >= 1);
        assert!(rng.ranges.contains(&(0, 9)));
        // Run past the kill.
        for _ in 0..200 {
            ai_final_fractal(&mut p, Some(&owner), &mut rng);
        }
        assert!(!p.active, "the 120-tick kill (`:43127-43130`)");
        assert!(p.local_ai[0] >= 120.0);
    }

    /// Style 184 (`:42793-42805`, `:42808-42821`): the one-shot cue, the
    /// ai[1] drain that re-arms tileCollide, and the four `Next(3)` dust
    /// gates. The self-hurt needs the owner + no unlock + an overlap.
    #[test]
    fn w4_d19_the_torch_fireball_drains_and_rearms_collision() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.unlocked_biome_torches = true; // skip the self-hurt lane
        let mut p = proj(949);
        p.vel = Vec2::new(3.0, 4.0); // length 5
        p.ai[0] = 6.0; // the dust type
        p.ai[1] = 12.0;
        let mut rng = census();
        ai_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(p.local_ai[0], 1.0, "the cue latch (`:42795`)");
        assert_eq!(p.wants_sound, Some((2, 8, p.pos.x + 2.0, p.pos.y + 2.0, 0.0, 1.0)));
        assert_eq!(p.ai[1], 7.0, "12 − |v| = 12 − 5 (`:42800`)");
        assert!(!p.tile_collide, "still draining");
        assert_eq!(rng.ranges.len(), 4, "four Next(3) dust gates");
        assert!(rng.ranges.iter().all(|&r| r == (0, 3)));
        // Drain to zero → tileCollide re-arms.
        p.ai[1] = 3.0;
        ai_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(p.ai[1], -2.0);
        assert!(p.tile_collide, "the else arm (`:42804`)");
    }

    /// W4-d19 follow-up — the 949 self-hurt IS dodgeable
    /// (`SelfHurtPlayers` `:15050` passes `IsDamageDodgeable()`; the 949
    /// is outside the exactly-9999 Moon Lord exemption `:15067`, so the
    /// parameter is TRUE): the hurt rides the shared `Player::hurt` dodge
    /// lane (`Player.cs:38214-38241` — sash → belt → brain → shadow).
    /// A belt-armed owner rolls the 1/10 dodge BEFORE any damage lands;
    /// the CensusRng's `next_range` returns min 0 → the dodge fires.
    /// This is the torch-god EVENT integration contract (f33's event
    /// machine — `Player.TorchAttack` `Player.cs:17813-17910` — spawns
    /// the 949 with ai0 = the torch dust id / ai1 = the flight distance;
    /// the arm + the dodge lane are the projectile side, landed here).
    #[test]
    fn w4_d19_the_torch_fireball_self_hurt_is_dodgeable() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.black_belt = true;
        pl.unlocked_biome_torches = false; // the self-hurt lane is armed
        // The fireball ON the player (the hitbox overlap gate `:15028`).
        let mut p = proj(949);
        p.pos = Vec2::new(pl.x, pl.y);
        p.damage = 40;
        p.owner = 0;
        let hp_before = pl.hp;
        let mut rng = census();
        ai_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        // The DamageVar draw + the belt's Next(10) rode the stream; the
        // dodge swallowed the hit (no HP loss, no hurt cooldown).
        assert_eq!(pl.hp, hp_before, "the belt dodged the fireball");
        assert!(rng.ranges.contains(&(0, 10)), "the belt's 1/10 roll");
        // Without the belt the same overlap DOES hurt (the DamageVar
        // draw is the only range draw before the four dust gates).
        let mut pl2 = crate::player::Player::spawn(&w);
        pl2.unlocked_biome_torches = false;
        let mut q = proj(949);
        q.pos = Vec2::new(pl2.x, pl2.y);
        q.damage = 40;
        q.owner = 0;
        let hp2 = pl2.hp;
        let mut rng2 = census();
        ai_bad_torch_luck(&mut q, Some(&mut pl2), 0, &mut rng2);
        assert!(pl2.hp < hp2, "the un-dodged fireball hurts");
    }

    /// Style 190's shared spine (`:41991-42018`, `:42222-42226`): the age,
    /// the progress rotation, the per-kind scale, the centre pin, the
    /// ×ai[2] scale and the progress-1 kill. The 972 dust gates draw.
    #[test]
    fn w4_d19_the_nights_edge_spine_scales_and_kills_at_progress_1() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut p = proj(972);
        p.vel = Vec2::new(8.0, 0.0);
        p.ai[0] = 1.0; // the swing direction sign
        p.ai[1] = 20.0; // the swing duration
        p.ai[2] = 1.0; // the spawn-scale multiplier
        let mut rng = census();
        ai_nights_edge(&mut p, &w, Some(&mut pl), 0, &mut rng);
        assert_eq!(p.local_ai[0], 1.0);
        assert!(p.active);
        // scale = (1 + num·0.2)·ai[2], num = 1/20.
        let num = 1.0 / 20.0;
        assert!((p.scale - (1.0 + num * 0.2)).abs() < 1e-5);
        // The 972 dust block drew its Next(2) gate (taken → the direction
        // float + the Next(5) + Next(2) inner gates).
        assert!(rng.ranges.contains(&(0, 2)));
        // Run to the kill (localAI[0] reaches ai[1]).
        for _ in 0..25 {
            ai_nights_edge(&mut p, &w, Some(&mut pl), 0, &mut rng);
        }
        assert!(!p.active, "the progress-1 kill (`:42223-42225`)");
    }

    /// The 1126 vertical-swing latch (`:41984-41988`) and the spore
    /// machine's sweep gate (`:42156-42172`): a zero velocity.Y latches
    /// localAI[2] = 1; the sweep rolls the melee-speed gate and queues a
    /// 1128 with ai1 = 90 + Next(60).
    #[test]
    fn w4_d19_the_claymore_latches_the_vertical_swing_and_queues_spores() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.held_item_kind = 1226; // the Chlorophyte Claymore
        let mut p = proj(1126);
        p.vel = Vec2::new(8.0, 0.0); // vertical swing (velocity.Y == 0)
        p.damage = 100;
        p.ai[0] = 1.0;
        p.ai[1] = 30.0;
        p.ai[2] = 1.0;
        let mut rng = census();
        let spores = ai_nights_edge(&mut p, &w, Some(&mut pl), 0, &mut rng);
        assert_eq!(p.local_ai[2], 1.0, "the vertical latch (`:41986`)");
        assert_eq!(p.vel.y, 0.0);
        // The sweep window (num = 1/30 ≤ 0.8) rolls the melee gate; the
        // CensusRng's Next returns 0 → the gate spawns. The progress-third
        // gates may force it too. At least one cloud rides out.
        assert!(!spores.clouds.is_empty(), "the sweep queued a 1128");
        let cloud = spores.clouds[0];
        // ai1 = 90 + Next(60); CensusRng returns min 0 → 90.
        assert_eq!(cloud.5, 90.0, "the lifetime seed (`:42161`)");
        // damage × 0.75.
        assert_eq!(cloud.4, (p.damage as f32 * 0.75) as i32);
    }

    /// The spore ground-bloom (`:41861-41970`): an exposed solid top under
    /// the probe blooms a 1128 (a lone top spawns TWO) and queues the 1129
    /// slam with ai1 = 128 / ai2 = 96.
    #[test]
    fn w4_d19_the_spore_bloom_queues_clouds_and_the_slam() {
        let mut w = world();
        // A lone exposed dirt top at (100, 52): solid, nothing above.
        put(&mut w, 100, 52, 1);
        let mut p = proj(1126);
        p.vel = Vec2::new(8.0, 0.0);
        p.damage = 100;
        p.scale = 1.0;
        p.rot = 0.0;
        p.ai[0] = 1.0;
        // The probe point directly above the exposed top.
        let spawn_position = Vec2::new(100.0 * 16.0 + 8.0, 52.0 * 16.0 - 8.0);
        let mut rng = census();
        let (clouds, slam) = spawn_spore_cloud(&mut p, &w, spawn_position, 0, 0, 1, &mut rng);
        assert!(!clouds.is_empty(), "the exposed top bloomed");
        // A lone exposed top (num3 == 1) spawns TWO clouds (`:41936`).
        assert_eq!(clouds.len(), 2, "the lone-top double spawn");
        for c in &clouds {
            assert_eq!(c.4, 75, "damage × 0.75");
            assert_eq!(c.5, 0.0, "the bloom clouds pass no ai1");
        }
        let slam = slam.expect("the slam rides with the bloom");
        assert_eq!(slam.2, 100, "the full damage (`:41967`)");
        assert_eq!(slam.3, 128.0, "ai1 = 128");
        assert_eq!(slam.4, 96.0, "ai2 = 96");
        assert_eq!(p.wants_sound, Some((2, 167, spawn_position.x, spawn_position.y, 0.0, 1.0)));
    }

    /// The terra-chain compose-contract receipt (research/26g §7, the
    /// d20c-T5 dependency): the use-path payload (`Player.cs:48917` —
    /// ai0 = dir·gravDir, ai1 = itemAnimationMax, ai2 =
    /// GetAdjustedItemScale) feeds the 190 envelope through the spawn
    /// seam. The row is the gate (the seam declines rowless kinds —
    /// T5's paired 972/982/983/984 slashes need THIS branch's rows).
    #[test]
    fn w4_d19_the_terra_chain_payload_feeds_the_190_envelope() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut pool = crate::projectile::ProjectilePool::new();
        // The TrueNightsEdge slash spawn (`Player.cs:48917`): kind 972,
        // MountedCenter, (dir, 0), Damage, ai0 = 1·1, ai1 = 20 (the
        // itemAnimationMax stand-in), ai2 = 1.0 (the adjusted scale).
        let slot = pool
            .spawn_with_data_ai_owned(972, 0, 1600.0, 800.0, 1.0, 0.0, 50, 5.0, 1.0, 20.0)
            .expect("the 972 row gates the spawn (d19's rows feed T5)");
        pool.set_ai2(slot, 1.0);
        assert_eq!(pool.slots()[slot].ai, [1.0, 20.0, 1.0, 0.0], "the payload rides");
        let mut rng = census();
        ai_nights_edge(&mut pool.slots_mut()[slot], &w, Some(&mut pl), 0, &mut rng);
        let p = &pool.slots()[slot];
        assert!(p.active, "the envelope does NOT collapse (ai1 = 20, not 0)");
        assert_eq!(p.local_ai[0], 1.0);
        // num = 1/20 — finite; scale = (1 + num·0.2)·ai2 (`:42018/:42222`).
        assert!((p.scale - (1.0 + (1.0 / 20.0) * 0.2)).abs() < 1e-5);
        // Drive to localAI[0] = ai[1]: the progress-1 kill (`:42223-42225`).
        for _ in 0..25 {
            ai_nights_edge(&mut pool.slots_mut()[slot], &w, Some(&mut pl), 0, &mut rng);
        }
        assert!(!pool.slots()[slot].active, "num >= 1 kills at the envelope end");
    }
}