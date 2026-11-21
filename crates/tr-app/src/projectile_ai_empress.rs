//! S1-D-4 — the Empress of Light's projectile AIs, the documented remainder
//! of R7's `npc_ai_empress.rs` machine (`task/S1-D-3-empress-audit`,
//! `ebdf83e3`): the five kinds `AI_120_HallowBoss` spawns
//! (`NPC.cs:45953-46942`), ported here as pure per-kind functions behind a
//! kind gate — **unwired spawner, code-only mode** (the Fishron/R7
//! precedent). Ids + styles VERIFIED against `ID/ProjectileID.cs` and each
//! `SetDefaults` arm (the brief's guesses corrected):
//!
//! | id  | name (ProjectileID.cs)              | aiStyle | SetDefaults      | AI arm |
//! |-----|-------------------------------------|---------|------------------|--------|
//! | 872 | HallowBossLastingRainbow (`:2298`)  | 173     | `Projectile.cs:8403-8415` | `AI_173_HallowBossRainbowTrail` `:45429-45440` |
//! | 873 | HallowBossRainbowStreak (`:2300`)   | 171     | `:8354-8366`     | `AI_171_HallowBossRainbowStreak` `:45451-45558` |
//! | 874 | HallowBossDeathAurora (`:2302`)     | **0**   | `:8367-8379`     | NONE — `AI()` (`:23199-36807`) has no style-0 arm |
//! | 919 | FairyQueenLance (`:2392`)           | 179     | `:8714-8726`     | `AI_179_FairyQueenLance` `:45032-45056` |
//! | 923 | FairyQueenSunDance (`:2400`)        | 180     | `:8757-8770`     | `AI_180_FairyQueenSunDance` `:44984-45030` |
//!
//! The brief's "874 PrismaticBolt / 873 EtherealLance / 872 SunDance?" is
//! remapped: the prismatic bolt is **873** (the homing streak), the ethereal
//! lance is **919**, the sun dance is **923**; 874 is the death-aurora flash
//! (a pure visual, aiStyle 0). 871 (HallowBossSplitShotCore, aiStyle 172,
//! `:8380-8392`, `AI_172_HallowBossRainbowPelletStorm` `:45569-45625`) and
//! 924 (FairyQueenHymn) are NOT spawned by `AI_120` — 871's only spawner is
//! the Queen Slime's gore-adjacent path (`NPC.cs:83753` is a different 871 —
//! a dust id; no `NewProjectile(871)` exists in `NPC.cs`/`Player.cs`) and 924
//! has no `SetDefaults` arm at all (an unused id). Both stay unclaimed BY
//! THIS GATE — W4-d18 rowed 871 with its own aiStyle-172 arm
//! (`projectile_ai_misc::ai_pellet_storm`); 924 stays unrowed.
//!
//! Boundaries honoured: R56 owns aiStyle 194 (untouched); R19b owns the
//! ai001 slice-C kinds (none of these five is aiStyle 1). The style-171
//! weapon variants **931/1039** (`FairyQueenMagicItemShot` /
//! `ConstellationStar`, `:8826`/`:9807`) were EXCLUDED from the hostile
//! gate by S1-D-4; **W4-d18's tail rowed them and ported their friendly
//! arm here** ([`fairy_weapon_tick`] + [`fairy_weapon_bounce`], the
//! `handles_fairy_weapon` gate): the `AI_171` parameter overrides
//! (`:45462-45492`, `:45551-45554`), the friendly
//! `FindTargetWithLineOfSight` acquire (`:45507-45526`) and the NPC homing
//! (`:45535-45548`), with the `HandleMovement` bounce (`:18592-18602`,
//! 4×4 collision box `:20017-20021`). For hostile 873 the friendly flag is
//! constant false (`SetDefaults` `:8361`, no reflect/parry seam in the
//! port), so the friendly branch is unreachable BY CONSTRUCTION in the
//! hostile arm.
//!
//! RNG by construction (every `Main.rand` draw cited, dust skips included):
//! * **931/1039: 24 draws on the 238-tick, ZERO otherwise.** The burst
//!   (`:45485-45490`) is 3× `NewDustPerfect(Center, 267, …)`: each
//!   `NextVector2CircularEdge(3,3)` = `NextVector2Unit`'s one `NextFloat`
//!   (`Utils.cs:1261-1264`/`:1271-1274`) + the radius `NextFloat` (1) +
//!   `Dust.NewDust`'s 6 internals (`Dust.cs:170-175`, `:190`) = 8 draws;
//!   the colour helpers draw nothing for non-dev names (`:21665-22041`).
//!   The acquire/homing/ramp roll nothing. The `Kill()` death burst
//!   (`:71149-71191`) is the pool-wide kill-dust skip (the 873 precedent).
//! * **872 / 874 / 923: ZERO draws.** `AI_173` is pure maths; style 0 has no
//!   arm; `AI_180`'s only effects are `SoundID.Item159` (`:44988`, skipped —
//!   no sound layer) and the 13-point `CastLightOpen` walk (`:45020-45024`,
//!   skipped — the lighting seam), neither draws.
//! * **873: ZERO draws in the AI.** The swirl/homing/opacity path
//!   (`:45493-45557`) has no rand for the hostile kind. The 20-dust death
//!   burst lives in `Kill()` (`:71490-71521`: per dust `NewDustPerfect` 6
//!   (`Dust.cs:170-175`, `:190`) + the colour `NextFloat` (`:71502`) + the
//!   velocity `NextFloat` (`:71519`) = 8 × 20 = 160 draws) — the port's kill
//!   seam (`Projectile::kill` / the lifetime block) runs NO `Kill()` visuals
//!   pool-wide (the ai001/ai026 precedent), so those draws are the documented
//!   pool-wide kill-dust skip, not this module's stream.
//! * **919: the ONE rand site.** Every tick with `localAI[0] >= 60`
//!   (`:45038`): `Next(3)` ALWAYS drawn (1); on a 0 the dust fires —
//!   `NewDustPerfect(Center, 267)` → `Dust.NewDust(pos, 0, 0, …)`
//!   (`Dust.cs:56`): w/h clamp to 5 (`:155-162`) → `Next(1)` ×2,
//!   `Next(-20,21)` ×2, `Next(3)`, `Next(-20,21)` = 6 draws; type 267 is in
//!   NONE of the extra-draw lists (`Dust.cs:214-252`); then the colour
//!   `Color.Lerp(AI_171_GetColor(), White, NextFloat() * 0.4f)` (`:45044`)
//!   = 1 more (`AI_171_GetColor` `:45560-45567` itself draws nothing — it
//!   reads `ai[1]`/`Main.time`). So 8 draws on a dust tick, 1 otherwise.
//!   Dust-pool assumption (the R7/ai026 documented deviation): the pool is
//!   below the 50 % skip tier (`Dust.cs:142-148`) and on-screen, so the skip
//!   draws (`:114-148`) and early-outs (`:77-101`) never fire;
//!   `ChildSafety.Disabled` is true (`GameContent/ChildSafety.cs:33`).
//!   The dust VISUALS are skipped; the draws are CONSUMED in C# order.
//!
//! Documented deviations (also `docs/ANALYSIS.md` "S1-D-4 empress proj"):
//! * the movement is the port's `HandleMovement` default-arm stand-in
//!   (`position += velocity`, `Projectile.cs:19851`) — all five kinds are
//!   `tileCollide = false`;
//! * `Vector2.Normalize` of an exactly-zero vector yields NaN in XNA; the
//!   port guards to zero (the Fishron/R7 precedent) — only differs on exact
//!   overlap;
//! * the damage-side slices stay with the wiring: the telegraph invuln
//!   (`Damage_CanDealDamage` `:12448` — 919/923 deal no damage while
//!   `localAI[0] <= 60`), the BossNoCheese immunity group (`Damage_EVP`
//!   `:14788-14796`), 919's line hitbox (`Colliding` `:15625-15641`, ±40 px
//!   along the rotation, width 8), 923's three-segment beam hitbox
//!   (`:15642-15668`, 510/660/800 × scale at widths 100/60/10 × 0.7 scale),
//!   872's oldPos-ring hitbox (`:15675-15694`, every 2nd of 50 entries —
//!   needs the 120-entry `TrailCacheLength` ring, `ProjectileID.cs:522`,
//!   vs the port's fixed 10);
//! * the render-side slices stay with the draw work: 872/873's trail rings
//!   (`TrailingMode` 2, cache 120/60, `ProjectileID.cs:521-522`), the
//!   `GetAlpha`/`AI_173_GetColor` tints (`:45442-45449`, `:78999-79002`),
//!   923's `drawLayer = 2` (`:8769`) and beam draw (`Main.cs:28268`);
//! * `npc_ref` maps the port's liveness (`hp > 0`) to `NPC.active` and
//!   `is_empress` is FALSE until R7's `NpcKind::HallowBoss` (636) row lands
//!   — main has no 636 kind, so the sun-dance centre snap cannot fire
//!   pre-wiring (the core itself is faithful and witness-tested through
//!   [`EmpressNpcRef`]).
//!
//! Witness tests are WRITTEN, NOT RUN (code-only mode — no compiler invoked):
//! the S1-D-4 nine pin the rows, the gate, and each arm's maths + RNG order;
//! W4-d18's tail adds the 931/1039 row/gate/arm/bounce witnesses.

#![allow(dead_code)] // wiring deferred (S1-D-4, code-only mode)

use std::f32::consts::PI;

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    chaseable, find_target_with_line_of_sight, get_lerp_value, npc_center, Projectile,
    CONSTELLATION_STAR, FAIRY_QUEEN_LANCE, FAIRY_QUEEN_MAGIC_ITEM_SHOT, FAIRY_QUEEN_SUN_DANCE,
    HALLOW_BOSS_DEATH_AURORA, HALLOW_BOSS_LASTING_RAINBOW, HALLOW_BOSS_RAINBOW_STREAK,
};

/// The claimed kind set — the five `AI_120_HallowBoss` spawns. 871/924 stay
/// out (871 rides its own style-172 arm, 924 unrowed); the style-171 weapon
/// variants 931/1039 ride their OWN friendly gate ([`handles_fairy_weapon`],
/// W4-d18 tail), NOT this hostile set. Every unclaimed kind keeps its
/// pre-existing dispatch byte-identical.
pub(crate) const EMPRESS_PROJ_KINDS: [u16; 5] = [
    HALLOW_BOSS_LASTING_RAINBOW, // 872
    HALLOW_BOSS_RAINBOW_STREAK,  // 873
    HALLOW_BOSS_DEATH_AURORA,    // 874
    FAIRY_QUEEN_LANCE,           // 919
    FAIRY_QUEEN_SUN_DANCE,       // 923
];

/// The dispatch gate: only these kinds leave the `_ => {}` fall-through.
pub(crate) fn handles(kind: u16) -> bool {
    EMPRESS_PROJ_KINDS.contains(&kind)
}

/// W4-d18 tail — the aiStyle-171 FRIENDLY weapon variants' gate: the
/// Nightglow bolt 931 and the Constellation star 1039 ride their own arm
/// ([`fairy_weapon_tick`]), NOT the hostile five-kind gate above. The C#
/// `AI_171` kind checks (`:45462`/`:45467`/`:45480`/`:45551`) name exactly
/// these two kinds; every other style-171 kind keeps the hostile arm.
pub(crate) fn handles_fairy_weapon(kind: u16) -> bool {
    matches!(kind, FAIRY_QUEEN_MAGIC_ITEM_SHOT | CONSTELLATION_STAR)
}

/// One `Main.npc[i]` slot as the sun dance reads it (`:45008-45014`):
/// `IndexInRange` is the slice bound, `active` the liveness, `type == 636`
/// the snap gate. The core takes refs so the witness tests can construct a
/// live Empress without the (unmerged) `NpcKind::HallowBoss` row.
#[derive(Clone, Copy)]
pub(crate) struct EmpressNpcRef {
    /// `NPC.active` — the port's liveness stand-in is `hp > 0`
    /// (`npc.rs::can_be_chased_by`'s doc).
    pub active: bool,
    /// `nPC.type == 636` (`NPCID.HallowBoss`, `ID/NPCID.cs:11949`).
    pub is_empress: bool,
    /// `NPC.Center` (`Entity.cs`: position + size/2).
    pub center: (f32, f32),
}

/// The wiring-side mapping from the port's `Npc`. `is_empress` is false
/// until R7's kind row lands (main has no 636) — the documented remainder.
pub(crate) fn npc_ref(n: &Npc) -> EmpressNpcRef {
    EmpressNpcRef {
        active: n.hp > 0,
        is_empress: false, // NpcKind::HallowBoss (636) lands with R7's merge
        center: n.center(),
    }
}

/// The per-kind tick. `who_am_i` is the pool slot index (the C# `whoAmI` —
/// 873's swirl phase reads it); `players` are the `Main.player` centres by
/// index (the single-player port passes a 0-or-1-length slice, so
/// `IndexInRange((int)ai[0])` keeps its exact semantics); `npcs` are the
/// `Main.npc` slots by index (923's tether). Returns false for an unclaimed
/// kind WITHOUT touching the slot or the stream.
pub(crate) fn empress_proj_tick(
    p: &mut Projectile,
    who_am_i: usize,
    players: &[(f32, f32)],
    npcs: &[EmpressNpcRef],
    rng: &mut dyn DropRng,
) -> bool {
    match p.kind {
        HALLOW_BOSS_LASTING_RAINBOW => ai_173_lasting_rainbow(p),
        HALLOW_BOSS_RAINBOW_STREAK => ai_171_rainbow_streak(p, who_am_i, players),
        // aiStyle 0 — `AI()` has no arm; only `HandleMovement` runs.
        HALLOW_BOSS_DEATH_AURORA => {}
        FAIRY_QUEEN_LANCE => ai_179_lance(p, rng),
        FAIRY_QUEEN_SUN_DANCE => ai_180_sun_dance(p, npcs),
        _ => return false,
    }
    // `HandleMovement`'s default arm (`Projectile.cs:19851`): all five kinds
    // are `tileCollide = false`, so the position integrates the velocity
    // once per sub-step. A killed slot still integrates in the C# (the
    // `Kill()` inside the AI does not skip `Update`'s movement) — verbatim.
    p.pos += p.vel;
    true
}

/// `MathHelper.Lerp` — XNA's `a + (b - a) * amount`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Projectile.Opacity`'s setter (`Projectile.cs:546-549`):
/// `alpha = (int)Clamp((1 - value) * 255, 0, 255)`.
fn set_opacity(p: &mut Projectile, opacity: f32) {
    p.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as i32;
}

/// `Vector2.RotatedBy` (`Utils.cs:1675-1684`, center = 0): the cos/sin are
/// computed in DOUBLE and cast to float (`(float)Math.Cos(radians)`), then
/// the float multiply-add — the port keeps that precision order.
fn rotated_by(v: Vec2, radians: f32) -> Vec2 {
    let c = (radians as f64).cos() as f32;
    let s = (radians as f64).sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Vector2.ToRotation` (`Utils.cs:1655-1658`): `(float)Math.Atan2(Y, X)` —
/// the double atan2 cast to float.
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `float.ToRotationVector2` (`Utils.cs:1665-1668`):
/// `((float)Math.Cos(f), (float)Math.Sin(f))`.
fn rotation_vector(angle: f32) -> Vec2 {
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Vector2.Normalize` with the port's zero guard (the Fishron/R7
/// documented deviation — XNA yields NaN on an exactly-zero vector).
fn safe_normalize(v: Vec2) -> Vec2 {
    let len = v.length();
    if len == 0.0 { Vec2::ZERO } else { v / len }
}

/// `MathHelper.Hermite(v1, 0, v2, 0, t)` — the zero-tangent form XNA's
/// `Vector2.SmoothStep` runs per component (the ReLogic double twin is
/// `Vector2D.cs:437-456`; the float `MathHelper` uses `float.Epsilon` =
/// 1.401298E-45 in the near-zero guard). The amount here rides
/// `Lerp(0.05, 0.1, …)` ∈ [0.05, 0.1], so both guards are unreachable —
/// kept verbatim anyway.
fn smooth_step(v1: f32, v2: f32, amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);
    let num = amount * amount * amount;
    let num2 = amount * amount;
    if amount.abs() <= f32::from_bits(1) {
        return v1;
    }
    if amount == 1.0 {
        return v2;
    }
    (2.0 * v1 - 2.0 * v2) * num + (3.0 * v2 - 3.0 * v1) * num2 + v1
}

/// `AI_173_HallowBossRainbowTrail` (`Projectile.cs:45429-45440`) — the
/// lasting rainbow (872): the opacity is the product of the fade-in
/// (timeLeft 0→60) and fade-out (660→600) ramps; the velocity rotates by
/// `ai[0]` FIRST, then `ai[0]` climbs π/10800 per tick until it reaches
/// π/360 (30 ticks) — a tightening spiral that settles into a constant
/// curvature. ZERO rand draws.
fn ai_173_lasting_rainbow(p: &mut Projectile) {
    // `:45431` — both ramps clamped.
    let opacity = get_lerp_value(0.0, 60.0, p.time_left as f32)
        * get_lerp_value(660.0, 600.0, p.time_left as f32);
    set_opacity(p, opacity);
    let num = PI / 360.0; // `:45432`
    let num2 = 30.0; // `:45433`
    // `:45434` — rotate by the CURRENT ai[0] before the increment.
    p.vel = rotated_by(p.vel, p.ai[0]);
    // `:45435-45438` — the curvature ramp, capped at π/360.
    if p.ai[0] < num {
        p.ai[0] += num / num2;
    }
    // `:45439` — the sprite rides the flight direction + π/2.
    p.rot = to_rotation(p.vel) + PI / 2.0;
}

/// `AI_171_HallowBossRainbowStreak` (`Projectile.cs:45451-45558`) — the
/// prismatic bolt (873), HOSTILE arm only (the friendly branch
/// `:45507-45526` serves the 931/1039 weapon variants, excluded — module
/// doc). Three phases by timeLeft (spawn 200, `:8363`):
/// * `> 140` — the swirl: `velocity *= 0.98` then a cos-phased rotation
///   (`:45501-45506`); the phase reads `whoAmI % 6 / 6 + x/320 + y/160`;
/// * `140 ≥ timeLeft > 30` — the homing: SmoothStep toward
///   `DirectionTo(player[ai[0]].Center) * 30` at amount
///   `Lerp(0.05, 0.1, GetLerpValue(140, 30, timeLeft))` (`:45527-45555`);
///   an out-of-range player index keeps the current velocity (`:45531`);
/// * `≤ 30` — straight flight to the 200-tick lifetime end.
/// The opacity ramp `GetLerpValue(240, 220, timeLeft)` (`:45556`) is 1 for
/// the whole 200-tick life (alpha 0 from the first tick — the spawn's
/// `alpha = 255` never shows). ZERO rand draws in the AI.
fn ai_171_rainbow_streak(p: &mut Projectile, who_am_i: usize, players: &[(f32, f32)]) {
    let num = 140.0; // `:45455`
    let num2 = 30.0; // `:45456`
    let num3 = 0.98; // `:45457`
    let value = 0.05; // `:45458`
    let value2 = 0.1; // `:45459`
    let num4 = 30.0; // `:45460`
    // `:45462-45492` — the 931/1039 parameter overrides + the 238-tick dust
    // burst are the weapon variants' (excluded kinds, module doc).
    let flag = p.time_left as f32 > num; // `:45493-45496`
    let flag2 = !flag && p.time_left as f32 > num2; // `:45497-45500`
    if flag {
        // `:45503` — `(float)Math.Cos((float)whoAmI % 6f / 6f + X/320f +
        // Y/160f)`: the argument is f32 arithmetic, the cos is DOUBLE then
        // cast to float.
        let arg = (who_am_i as f32 % 6.0) / 6.0 + p.pos.x / 320.0 + p.pos.y / 160.0;
        let num6 = (arg as f64).cos() as f32;
        // `:45504-45505` — damp then rotate; the angle's factor order is
        // the C#'s left-to-right `num6 * (π*2) * 0.125 * 1 / 30`.
        p.vel *= num3;
        p.vel = rotated_by(p.vel, num6 * (PI * 2.0) * 0.125 * 1.0 / 30.0);
    }
    // `:45507-45526` — the friendly acquisition branch: unreachable for
    // hostile 873 (SetDefaults `:8361` friendly = false; no reflect seam).
    if flag2 {
        let num9 = p.ai[0] as i32; // `:45529`
        let mut value3 = p.vel; // `:45530`
        // `:45531-45534` — hostile + IndexInRange: aim at the player.
        // `base.DirectionTo` = Normalize(target − Center) (`Entity.cs:205-208`).
        if num9 >= 0 && (num9 as usize) < players.len() {
            let (px, py) = players[num9 as usize];
            let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
            value3 = safe_normalize(Vec2::new(px, py) - center) * num4;
        }
        // `:45550` — SmoothStep at the timeLeft-lerped amount.
        let amount = lerp(value, value2, get_lerp_value(num, 30.0, p.time_left as f32));
        p.vel = Vec2::new(
            smooth_step(p.vel.x, value3.x, amount),
            smooth_step(p.vel.y, value3.y, amount),
        );
        // `:45551-45554` — the 931/1039 speed ramp: excluded kinds.
    }
    // `:45556-45557` — the opacity + rotation tail.
    let opacity = get_lerp_value(240.0, 220.0, p.time_left as f32);
    set_opacity(p, opacity);
    p.rot = to_rotation(p.vel) + PI / 2.0;
}

/// W4-d18 tail — `AI_171_HallowBossRainbowStreak`'s FRIENDLY weapon arm
/// (`Projectile.cs:45451-45558`), the Nightglow bolt 931 and the
/// Constellation star 1039. The hostile 873 keeps [`ai_171_rainbow_streak`];
/// this arm adds the C# kind-gated branches the hostile gate excluded:
/// * the parameter overrides (`:45462-45474`): the swirl window 180, the
///   homing floor 20, the ×0.97 damp, the 0.075/0.125 SmoothStep amounts;
///   1039's homing speed 60 (`:45464`) is reset to 30 by the shared
///   931||1039 block (`:45471`) — a verbatim dead store — but its acquire
///   range 1600 (`:45465`) SURVIVES (the shared block has no maxRange line);
/// * the 238-tick dust burst (`:45475-45491`): the alpha save/restore
///   around the colour helper (which draws nothing for non-dev names,
///   `:21665-22041`), then 3× `NewDustPerfect(Center, 267, …)` — each
///   `NextVector2CircularEdge(3,3)` (1 float, `Utils.cs:1271-1274`) + the
///   radius `NextFloat` (1) + `NewDust`'s 6 internals = 8 draws, 24 total;
/// * the friendly acquire (`:45507-45526`): the `CanBeChasedBy` validity
///   drop + the `FindTargetWithLineOfSight(maxRange)` re-acquire, both
///   stamping `netUpdate`;
/// * the friendly homing (`:45535-45548`): an in-range NPC pins
///   `timeLeft >= 10` and aims at its centre ×30; an out-of-range index
///   spends one extra `timeLeft--`;
/// * the 931/1039 speed ramp (`:45551-45554`): ×Lerp(0.85, 1,
///   GetLerpValue(0, 90, timeLeft)).
///
/// Movement is the dispatch's bounce step ([`fairy_weapon_bounce`],
/// `HandleMovement` `:18592-18602`'s full-axis −1 reflection on the 4×4
/// collision box, `GetCollisionParams` `:20017-20021`), NOT the hostile
/// arm's free integrate. RNG: 24 draws on the 238-tick, ZERO otherwise
/// (the acquire/homing/ramp roll nothing).
pub(crate) fn fairy_weapon_tick(
    p: &mut Projectile,
    who_am_i: usize,
    world: &tr_core::World,
    npcs: &[Npc],
    immune: &[i32],
    rng: &mut dyn DropRng,
) {
    // `:45455-45461` — the hostile defaults.
    let (mut num, mut num2, mut num3) = (140.0f32, 30.0f32, 0.98f32);
    let (mut value, mut value2) = (0.05f32, 0.1f32);
    let mut num4 = 30.0f32;
    let mut max_range = 800.0f32;
    // `:45462-45466` — 1039's speed/range override. num4 is reset below
    // (a dead store); maxRange 1600 survives (the shared block has no
    // maxRange line).
    if p.kind == CONSTELLATION_STAR {
        num4 = 60.0;
        max_range = 1600.0;
    }
    // `:45467-45474` — the shared weapon parameter block. NOTE: it resets
    // num4 to 30 (1039's 60 is a dead store) but NOT maxRange — 1039 keeps
    // its 1600 acquire range (the C# block has no maxRange line).
    num = 180.0;
    num2 = 20.0;
    num4 = 30.0;
    num3 = 0.97;
    value = 0.075;
    value2 = 0.125;
    // `:45475-45491` — the 238-tick dust burst.
    if p.time_left == 238 {
        // `:45477-45484` — the alpha save/restore around the colour helper
        // (GetFairyQueenWeaponsColor / GetConstellationColor draw nothing
        // for non-dev names, `:21665-22041`); the colour feeds only dust.
        let num5 = p.alpha;
        p.alpha = 0;
        p.alpha = num5;
        // `:45485-45490` — 3× NewDustPerfect(Center, 267, …).
        for _ in 0..3 {
            // `:45487` — NextVector2CircularEdge(3,3): NextVector2Unit's
            // one NextFloat (Utils.cs:1261-1264, :1271-1274).
            let _ = rng.next_float();
            // `:45487` — the radius scale's NextFloat.
            let _ = rng.next_float();
            // Dust.NewDust's 6 internals (Dust.cs:170-175, :190; w/h = 0 →
            // clamped 5 → Next(1) ×2). The dust VISUALS are the pool-wide
            // skip; the draws are CONSUMED in C# order.
            let _ = rng.next_range(0, 1); // position.X
            let _ = rng.next_range(0, 1); // position.Y
            let _ = rng.next_range(-20, 21); // velocity.X
            let _ = rng.next_range(-20, 21); // velocity.Y
            let _ = rng.next_range(0, 3); // frame.Y
            let _ = rng.next_range(-20, 21); // scale
        }
    }
    // `:45493-45500` — the phase flags.
    let flag = p.time_left as f32 > num;
    let flag2 = !flag && p.time_left as f32 > num2;
    if flag {
        // `:45503-45505` — the swirl: damp then the cos-phased rotation.
        let arg = (who_am_i as f32 % 6.0) / 6.0 + p.pos.x / 320.0 + p.pos.y / 160.0;
        let num6 = (arg as f64).cos() as f32;
        p.vel *= num3;
        p.vel = rotated_by(p.vel, num6 * (PI * 2.0) * 0.125 * 1.0 / 30.0);
    }
    // `:45507-45526` — the friendly acquire (both weapon kinds are friendly).
    let mut num7 = p.ai[0] as i32; // `:45509`
    // `:45510-45515` — IndexInRange && !CanBeChasedBy → drop the target.
    if num7 >= 0 && (num7 as usize) < npcs.len() && !chaseable(&npcs[num7 as usize]) {
        num7 = -1;
        p.ai[0] = -1.0;
        p.net_update = true;
    }
    // `:45516-45525` — the no-target re-acquire.
    if num7 == -1 {
        let num8 = find_target_with_line_of_sight(world, npcs, p.pos, p.w, p.h, max_range, immune);
        if num8 != -1 {
            num7 = num8;
            p.ai[0] = num8 as f32;
            p.net_update = true;
        }
    }
    if flag2 {
        let num9 = p.ai[0] as i32; // `:45529`
        let mut value3 = p.vel; // `:45530`
        // `:45531-45534` — the hostile arm: unreachable for the friendly
        // weapon kinds (`hostile = false`), kept out by construction.
        // `:45535-45548` — the friendly homing.
        if num9 >= 0 && (num9 as usize) < npcs.len() {
            // `:45539-45542` — pin the life at 10 while a target holds.
            if p.time_left < 10 {
                p.time_left = 10;
            }
            // `:45543` — aim at the NPC centre ×num4.
            let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
            value3 = safe_normalize(npc_center(&npcs[num9 as usize]) - center) * num4;
        } else {
            // `:45547` — no target: one extra life tick.
            p.time_left -= 1;
        }
        // `:45550` — SmoothStep at the timeLeft-lerped amount.
        let amount = lerp(value, value2, get_lerp_value(num, 30.0, p.time_left as f32));
        p.vel = Vec2::new(
            smooth_step(p.vel.x, value3.x, amount),
            smooth_step(p.vel.y, value3.y, amount),
        );
        // `:45551-45554` — the 931/1039 speed ramp.
        p.vel *= lerp(0.85, 1.0, get_lerp_value(0.0, 90.0, p.time_left as f32));
    }
    // `:45556-45557` — the opacity + rotation tail.
    let opacity = get_lerp_value(240.0, 220.0, p.time_left as f32);
    set_opacity(p, opacity);
    p.rot = to_rotation(p.vel) + PI / 2.0;
}

/// W4-d18 tail — the 931/1039 movement: `HandleMovement`'s blocked-move arm
/// (`Projectile.cs:18592-18602`) reflects EACH blocked axis at full speed
/// (−1×lastVelocity), then `UpdatePosition` integrates (`:20445`). The
/// collision box is the 4×4 centre inset (`GetCollisionParams`
/// `:20017-20021`, resizeAnchor 0.5/0.5 → the box centres on the sprite).
/// The port's probe is the per-axis destination-box stand-in the arrow
/// arms use (`kill_step`'s shape) — the swept `TileCollision` span is the
/// recorded approximation; the reflection + integrate order is verbatim.
pub(crate) fn fairy_weapon_bounce(p: &mut Projectile, world: &tr_core::World) {
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let (bx, by) = (cx - 2.0, cy - 2.0);
    let last = p.vel;
    let hit_x = crate::player::collides_rect(world, bx + p.vel.x, by, 4.0, 4.0);
    let hit_y = crate::player::collides_rect(world, bx, by + p.vel.y, 4.0, 4.0);
    // `:18594-18601` — reflect each blocked axis (the C# tests
    // `velocity.X != lastVelocity.X` after TileCollision zeroed the axis).
    if hit_x {
        p.vel.x = -last.x;
    }
    if hit_y {
        p.vel.y = -last.y;
    }
    // `UpdatePosition` (`:20445`) — the free integrate of the resolved vel.
    p.pos += p.vel;
}

/// `AI_179_FairyQueenLance` (`Projectile.cs:45032-45056`) — the ethereal
/// lance (919): a 60-tick telegraph (velocity stays at the spawn's zero,
/// the alpha fades in over the first 20), then the dash —
/// `velocity = ai[0].ToRotationVector2() * 40` re-written EVERY tick from
/// the 60th (`:45037`), with the 1-in-3 dust (`:45038-45047`, draws
/// consumed — module doc). The 360-tick self-kill (`:45049-45053`) is
/// unreachable at the stock 240 timeLeft (`:8723`) but kept verbatim.
/// `rotation = ai[0]` (`:45055`) — the lance points along its dash.
fn ai_179_lance(p: &mut Projectile, rng: &mut dyn DropRng) {
    p.local_ai[0] += 1.0; // `:45034`
    if p.local_ai[0] >= 60.0 {
        // `:45037` — the dash velocity, re-aimed every tick.
        p.vel = rotation_vector(p.ai[0]) * 40.0;
        // `:45038` — Next(3) ALWAYS drawn; the dust fires on a 0.
        if rng.next_range(0, 3) == 0 {
            // `:45040` — NewDustPerfect(Center, 267): Dust.NewDust's 6
            // draws (w/h = 0 → clamp 5 → Next(1) ×2; Dust.cs:155-175, :190).
            let _ = rng.next_range(0, 1); // position.X jitter
            let _ = rng.next_range(0, 1); // position.Y jitter
            let _ = rng.next_range(-20, 21); // velocity.X
            let _ = rng.next_range(-20, 21); // velocity.Y
            let _ = rng.next_range(0, 3); // frame.Y
            let _ = rng.next_range(-20, 21); // scale
            // `:45044` — the colour lerp's NextFloat (AI_171_GetColor
            // itself draws nothing, `:45560-45567`).
            let _ = rng.next_float();
            // The dust VISUALS (fadeIn 1, noGravity, alpha 100, noLight-
            // Emittance, scale ×1.5) are the documented pool-wide skip.
        }
    }
    if p.local_ai[0] >= 360.0 {
        // `:45049-45053` — Kill() + return: the alpha/rotation tail skips.
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:45054` — alpha = (int)Lerp(255, 0, GetLerpValue(0, 20, localAI[0]))
    // — the (int) cast truncates toward zero.
    p.alpha = lerp(255.0, 0.0, get_lerp_value(0.0, 20.0, p.local_ai[0])) as i32;
    // `:45055` — the sprite rides the dash angle verbatim.
    p.rot = p.ai[0];
}

/// `AI_180_FairyQueenSunDance` (`Projectile.cs:44984-45030`) — the sun
/// dance (923): the boss-tethered rotating beam. The first-tick sound
/// (`SoundID.Item159`, `:44988`) is the no-sound-layer skip (no rand). The
/// 180-tick life (`:44994-44998`), the −15/tick alpha fade (`:44999-45003`),
/// the scale product ramp (`:45004`) and the rotation
/// `ai[0] + GetLerpValue(50, 180, localAI[0]) * π/9` (`:45005-45006`).
/// `ai[1]` is the boss's NPC index: in range → the velocity zeroes and (if
/// the NPC is a live 636) the centre snaps to the boss (`:45008-45015`);
/// out of range → `Kill()` (`:45026-45029`). The 13-point light walk
/// (`:45016-45024`) is the lighting-seam skip — ZERO rand draws.
fn ai_180_sun_dance(p: &mut Projectile, npcs: &[EmpressNpcRef]) {
    // `:44986-44989` — the first-tick sound: SKIPPED (no sound layer, no
    // rand). `:44990` — the age counter.
    p.local_ai[0] += 1.0;
    let num = 180.0; // `:44991`
    let num2 = PI / 9.0; // `:44993`
    if p.local_ai[0] >= num {
        // `:44994-44998` — Kill() + return.
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:44999-45003` — the alpha fade, floored at 0.
    p.alpha -= 15;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:45004` — scale = fade-in(0→20) × fade-out(180→120).
    p.scale = get_lerp_value(0.0, 20.0, p.local_ai[0])
        * get_lerp_value(num, num - 60.0, p.local_ai[0]);
    // `:45005-45006` — the sweep: π/9 over the 50→180 window.
    let lerp_value = get_lerp_value(50.0, num, p.local_ai[0]);
    p.rot = p.ai[0] + lerp_value * num2;
    let num3 = p.ai[1] as i32; // `:45007`
    // `:45008` — IndexInRange (Utils.cs:2441-2448).
    if num3 >= 0 && (num3 as usize) < npcs.len() {
        let n = npcs[num3 as usize];
        // `:45011-45014` — the snap fires only for a LIVE 636.
        if n.active && n.is_empress {
            let (cx, cy) = n.center;
            p.pos = Vec2::new(cx - p.w * 0.5, cy - p.h * 0.5);
        }
        // `:45015` — the velocity zeroes for ANY in-range NPC (verbatim —
        // outside the 636 check).
        p.vel = Vec2::ZERO;
        // `:45016-45024` — the hsl colour + the 13-point CastLightOpen
        // walk: the lighting-seam skip (no rand).
    } else {
        // `:45026-45029` — the boss gone: Kill(). The C# falls through to
        // Update's movement with the pre-kill velocity; the port's tick
        // integrates after the match, same order.
        p.active = false;
        p.time_left = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    /// A recording `DropRng` — the draw log pins the C#'s `Main.rand` order
    /// (the golden rule's RNG witness). `next_range` returns `min`, so the
    /// 919 dust gate (`Next(3) == 0`) FIRES; `ScriptedRng` below pins the
    /// non-firing path.
    #[derive(Default)]
    struct RecordRng {
        draws: Vec<String>,
    }
    impl DropRng for RecordRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push(format!("range({min},{max})"));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float".into());
            0.5
        }
    }

    /// A scripted stream — the 919 gate's non-zero path.
    struct ScriptedRng {
        ranges: Vec<i32>,
        idx: usize,
        draws: Vec<String>,
    }
    impl DropRng for ScriptedRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push(format!("range({min},{max})"));
            let v = self.ranges[self.idx];
            self.idx += 1;
            assert!(v >= min && v < max);
            v
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float".into());
            0.5
        }
    }

    fn shot(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("empress row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 255;
        p.pos = Vec2::new(480.0, 320.0);
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

    /// The census: the five rows carry their `SetDefaults` slices.
    #[test]
    fn the_five_rows_carry_the_csharp_set_defaults() {
        for (kind, style, life, w) in [
            (HALLOW_BOSS_LASTING_RAINBOW, 173u8, 660i32, 30.0f32),
            (HALLOW_BOSS_RAINBOW_STREAK, 171, 200, 30.0),
            (HALLOW_BOSS_DEATH_AURORA, 0, 210, 30.0),
            (FAIRY_QUEEN_LANCE, 179, 240, 8.0),
            (FAIRY_QUEEN_SUN_DANCE, 180, 180, 30.0),
        ] {
            let d = proj_data(kind).expect("row");
            assert_eq!((d.w, d.h), (w, w), "kind {kind} box");
            assert_eq!(d.ai_style, style, "kind {kind} aiStyle");
            assert_eq!(d.penetrate, -1, "kind {kind} penetrate");
            assert_eq!(d.time_left, life, "kind {kind} timeLeft");
            assert_eq!(d.alpha, 255, "kind {kind} alpha");
            assert!(!d.tile_collide, "kind {kind} tileCollide");
            assert!(d.ignore_water, "kind {kind} ignoreWater");
            assert_eq!(d.extra_updates, 0, "kind {kind} extraUpdates");
        }
        // The unclaimed neighbours: 924 stays unrowed; 871 was rowed by
        // W4-d18 with its OWN style-172 arm — this gate still does not
        // claim it (the gate witness below).
        assert!(proj_data(924).is_none());
    }

    /// The gate claims exactly the five; an unclaimed kind returns false
    /// with the slot byte-identical and the stream untouched.
    #[test]
    fn the_gate_leaves_unclaimed_kinds_byte_identical() {
        for kind in EMPRESS_PROJ_KINDS {
            assert!(handles(kind));
        }
        for kind in [14u16, 871, 924, 931, 932, 1039] {
            assert!(!handles(kind));
            let mut p = shot(HALLOW_BOSS_RAINBOW_STREAK);
            p.kind = kind;
            let before = p.clone();
            let mut rng = RecordRng::default();
            assert!(!empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
            assert_eq!(p.pos, before.pos);
            assert_eq!(p.vel, before.vel);
            assert_eq!(p.ai, before.ai);
            assert_eq!(p.alpha, before.alpha);
            assert!(rng.draws.is_empty());
        }
    }

    /// 874 (aiStyle 0): no AI state writes at all — only the movement
    /// integration; zero rand.
    #[test]
    fn the_death_aurora_is_a_still_visual() {
        let mut p = shot(HALLOW_BOSS_DEATH_AURORA);
        let mut rng = RecordRng::default();
        assert!(empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
        assert_eq!(p.pos, Vec2::new(480.0, 320.0)); // vel 0 → still
        assert_eq!(p.alpha, 255); // untouched
        assert_eq!(p.rot, 0.0);
        assert!(rng.draws.is_empty());
        // A moving aurora integrates (the HandleMovement default arm).
        p.vel = Vec2::new(2.0, -3.0);
        assert!(empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
        assert_eq!(p.pos, Vec2::new(482.0, 317.0));
        assert!(rng.draws.is_empty());
    }

    /// 872: the curvature ramps π/10800 per tick for 30 ticks then holds;
    /// the rotation happens BEFORE the increment; the opacity is the
    /// two-ramp product; zero rand.
    #[test]
    fn the_lasting_rainbow_spirals_on_the_ramped_curvature() {
        let mut p = shot(HALLOW_BOSS_LASTING_RAINBOW);
        p.vel = Vec2::new(8.0, 0.0);
        let mut rng = RecordRng::default();
        // Tick 1: rotate by ai[0] = 0 (no turn), THEN ai[0] += (π/360)/30.
        assert!(empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
        assert_eq!(p.vel, Vec2::new(8.0, 0.0));
        let step = (PI / 360.0) / 30.0; // the C#'s num / num2, verbatim
        assert_eq!(p.ai[0], step);
        // Tick 2: rotate by the accumulated ai[0].
        assert!(empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
        let expect = rotated_by(Vec2::new(8.0, 0.0), step);
        assert!((p.vel - expect).length() < 1e-5);
        assert_eq!(p.ai[0], step + step);
        // After 30 ticks the curvature sits at π/360 (the f32 accumulation
        // can land one step short/over — the C# accumulates identically);
        // 10 more ticks prove the ramp STOPPED (at most one overshoot).
        for _ in 0..28 {
            empress_proj_tick(&mut p, 0, &[], &[], &mut rng);
        }
        assert!((p.ai[0] - PI / 360.0).abs() < step);
        for _ in 0..10 {
            empress_proj_tick(&mut p, 0, &[], &[], &mut rng);
        }
        assert!(p.ai[0] < PI / 360.0 + 2.0 * step, "the ramp capped");
        // The opacity product: timeLeft 660 → 1 × 0 = 0 → alpha 255;
        // 600 → 1 × 1 → alpha 0; 30 → 0.5 × 1 → (1−0.5)·255 = 127.
        p.time_left = 660;
        empress_proj_tick(&mut p, 0, &[], &[], &mut rng);
        assert_eq!(p.alpha, 255);
        p.time_left = 600;
        empress_proj_tick(&mut p, 0, &[], &[], &mut rng);
        assert_eq!(p.alpha, 0);
        p.time_left = 30;
        empress_proj_tick(&mut p, 0, &[], &[], &mut rng);
        assert_eq!(p.alpha, 127);
        assert!(rng.draws.is_empty(), "AI_173 draws nothing");
    }

    /// 873 swirl phase (timeLeft > 140): ×0.98 then the cos-phased turn;
    /// the phase reads whoAmI % 6 and the position; zero rand.
    #[test]
    fn the_prismatic_bolt_swirls_before_it_homes() {
        let mut p = shot(HALLOW_BOSS_RAINBOW_STREAK);
        p.vel = Vec2::new(10.0, 0.0);
        p.time_left = 200;
        let mut rng = RecordRng::default();
        assert!(empress_proj_tick(&mut p, 3, &[], &[], &mut rng));
        // The phase reads the PRE-movement position (the AI runs before
        // HandleMovement): (480, 320) → 0.5 + 1.5 + 2.0 = 4.0; the cos is
        // the C#'s double Math.Cos cast to float (`:45503`).
        let phase = (3.0f32 % 6.0) / 6.0 + 480.0 / 320.0 + 320.0 / 160.0;
        let expect = rotated_by(
            Vec2::new(10.0, 0.0) * 0.98,
            (phase as f64).cos() as f32 * (PI * 2.0) * 0.125 / 30.0,
        );
        assert!((p.vel - expect).length() < 1e-4);
        // The opacity ramp is 1 for the whole 200-tick life → alpha 0.
        assert_eq!(p.alpha, 0);
        assert!(rng.draws.is_empty(), "AI_171's hostile path draws nothing");
    }

    /// 873 homing phase (140 ≥ timeLeft > 30): SmoothStep toward the
    /// player aim ×30 at the lerped amount; an out-of-range index keeps
    /// the velocity.
    #[test]
    fn the_prismatic_bolt_homes_on_the_indexed_player() {
        let mut p = shot(HALLOW_BOSS_RAINBOW_STREAK);
        p.vel = Vec2::new(10.0, 0.0);
        p.time_left = 140;
        p.ai[0] = 0.0; // player index 0
        // Player dead ahead: centre (960, 335) vs proj centre (495, 335).
        let players = [(960.0f32, 335.0f32)];
        let mut rng = RecordRng::default();
        assert!(empress_proj_tick(&mut p, 0, &players, &[], &mut rng));
        // amount = Lerp(0.05, 0.1, GetLerpValue(140, 30, 140)) = 0.05;
        // s = 3·0.05² − 2·0.05³ = 0.00725; vx = 10 + (30−10)·0.00725.
        let s = 3.0 * 0.05f32 * 0.05 - 2.0 * 0.05 * 0.05 * 0.05;
        assert!((p.vel.x - (10.0 + 20.0 * s)).abs() < 1e-4);
        assert!(p.vel.y.abs() < 1e-5);
        // Out-of-range index (5 ≥ players.len()): value3 = velocity →
        // SmoothStep(v, v) = v, unchanged.
        p.ai[0] = 5.0;
        let v = p.vel;
        empress_proj_tick(&mut p, 0, &players, &[], &mut rng);
        assert_eq!(p.vel, v);
        // timeLeft ≤ 30: neither phase — straight flight.
        p.time_left = 30;
        let v = p.vel;
        empress_proj_tick(&mut p, 0, &players, &[], &mut rng);
        assert_eq!(p.vel, v);
        assert!(rng.draws.is_empty());
    }

    /// 919 telegraph: ticks 1..59 keep the velocity, fade the alpha over
    /// 20 ticks ((int) truncation), pin rotation = ai[0]; NO rand before 60.
    #[test]
    fn the_lance_telegraphs_for_sixty_ticks() {
        let mut p = shot(FAIRY_QUEEN_LANCE);
        p.ai[0] = 0.7;
        let mut rng = RecordRng::default();
        for tick in 1..60 {
            assert!(empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
            assert_eq!(p.vel, Vec2::ZERO, "tick {tick} still telegraphing");
            assert_eq!(p.rot, 0.7);
            // The (int) cast TRUNCATES toward zero (not banker's round):
            // tick 1 → 255 − 12.75 = 242.25 → 242. Same f32 expression as
            // the port (Lerp(255, 0, GetLerpValue(0, 20, t))).
            let expect = lerp(255.0, 0.0, get_lerp_value(0.0, 20.0, tick as f32)) as i32;
            assert_eq!(p.alpha, expect, "tick {tick} alpha");
            if tick == 1 {
                assert_eq!(p.alpha, 242, "truncation, not rounding");
            }
        }
        assert!(rng.draws.is_empty(), "no rand during the telegraph");
        assert_eq!(p.alpha, 0);
    }

    /// 919 dash + the RNG witness: at tick 60 the velocity snaps to
    /// rotvec(ai[0])·40 and Next(3) fires; a 0 consumes the 6 NewDust draws
    /// + the colour NextFloat (8 total, exact order); a non-zero consumes
    /// only the gate draw.
    #[test]
    fn the_lance_dashes_and_consumes_the_dust_draws_in_order() {
        let mut p = shot(FAIRY_QUEEN_LANCE);
        p.ai[0] = 0.0;
        p.local_ai[0] = 59.0;
        let mut rng = RecordRng::default(); // returns min → the gate fires
        assert!(empress_proj_tick(&mut p, 0, &[], &[], &mut rng));
        assert_eq!(p.vel, Vec2::new(40.0, 0.0));
        assert_eq!(
            rng.draws,
            vec![
                "range(0,3)",     // :45038 the gate
                "range(0,1)",     // Dust.cs:170 position.X (w clamped 5)
                "range(0,1)",     // :171 position.Y
                "range(-20,21)",  // :172 velocity.X
                "range(-20,21)",  // :173 velocity.Y
                "range(0,3)",     // :175 frame.Y
                "range(-20,21)",  // :190 scale
                "float",          // :45044 the colour lerp
            ]
        );
        // The non-firing gate: exactly ONE draw.
        let mut p2 = shot(FAIRY_QUEEN_LANCE);
        p2.local_ai[0] = 59.0;
        let mut rng2 = ScriptedRng { ranges: vec![1], idx: 0, draws: vec![] };
        assert!(empress_proj_tick(&mut p2, 0, &[], &[], &mut rng2));
        assert_eq!(rng2.draws, vec!["range(0,3)"]);
        // The 360-tick self-kill returns BEFORE the alpha/rotation tail.
        let mut p3 = shot(FAIRY_QUEEN_LANCE);
        p3.local_ai[0] = 359.0;
        p3.time_left = 400; // outlive the stock 240 to reach the self-kill
        p3.alpha = 123;
        let mut rng3 = ScriptedRng { ranges: vec![2], idx: 0, draws: vec![] };
        assert!(empress_proj_tick(&mut p3, 0, &[], &[], &mut rng3));
        assert!(!p3.active);
        assert_eq!(p3.time_left, 0);
        assert_eq!(p3.alpha, 123, "the kill's return skips the alpha tail");
    }

    /// 923: the alpha −15/tick fade, the scale product, the π/9 sweep, the
    /// boss snap (live 636 only), the velocity zero for ANY in-range NPC,
    /// the out-of-range kill; zero rand throughout.
    #[test]
    fn the_sun_dance_tethers_to_the_boss() {
        let boss = EmpressNpcRef { active: true, is_empress: true, center: (600.0, 400.0) };
        let mut p = shot(FAIRY_QUEEN_SUN_DANCE);
        p.ai[0] = 1.0; // the start angle
        p.ai[1] = 0.0; // the boss index
        p.vel = Vec2::new(5.0, 5.0);
        let mut rng = RecordRng::default();
        assert!(empress_proj_tick(&mut p, 0, &[], &[boss], &mut rng));
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.alpha, 240); // 255 − 15
        assert_eq!(p.pos, Vec2::new(585.0, 385.0)); // centre snap − 15
        assert_eq!(p.vel, Vec2::ZERO);
        assert_eq!(p.scale, 1.0 / 20.0); // fade-in 1/20 × fade-out 1
        assert_eq!(p.rot, 1.0); // GetLerpValue(50, 180, 1) = 0
        // A live NON-636 in range: velocity zeroes, NO snap.
        let other = EmpressNpcRef { active: true, is_empress: false, center: (900.0, 900.0) };
        let mut p2 = shot(FAIRY_QUEEN_SUN_DANCE);
        p2.ai[1] = 0.0;
        let pos = p2.pos;
        empress_proj_tick(&mut p2, 0, &[], &[other], &mut rng);
        assert_eq!(p2.pos, pos, "no snap for a non-636");
        assert_eq!(p2.vel, Vec2::ZERO);
        // Out of range: Kill().
        let mut p3 = shot(FAIRY_QUEEN_SUN_DANCE);
        p3.ai[1] = 7.0;
        empress_proj_tick(&mut p3, 0, &[], &[boss], &mut rng);
        assert!(!p3.active);
        // The 180-tick self-kill.
        let mut p4 = shot(FAIRY_QUEEN_SUN_DANCE);
        p4.ai[1] = 0.0;
        p4.local_ai[0] = 179.0;
        empress_proj_tick(&mut p4, 0, &[], &[boss], &mut rng);
        assert!(!p4.active);
        // The sweep at localAI[0] = 115: GetLerpValue(50,180,115) = 0.5.
        let mut p5 = shot(FAIRY_QUEEN_SUN_DANCE);
        p5.ai[0] = 0.0;
        p5.ai[1] = 0.0;
        p5.local_ai[0] = 114.0;
        empress_proj_tick(&mut p5, 0, &[], &[boss], &mut rng);
        assert!((p5.rot - 0.5 * PI / 9.0).abs() < 1e-6);
        // The alpha floors at 0 (tick 18: 255 − 270 → 0).
        let mut p6 = shot(FAIRY_QUEEN_SUN_DANCE);
        p6.ai[1] = 0.0;
        for _ in 0..18 {
            empress_proj_tick(&mut p6, 0, &[], &[boss], &mut rng);
        }
        assert_eq!(p6.alpha, 0);
        assert!(rng.draws.is_empty(), "AI_180 draws nothing");
    }

    // ===== W4-d18 tail: the aiStyle-171 friendly weapon variants =====

    fn weapon(kind: u16) -> Projectile {
        let mut p = shot(kind);
        p.friendly = true;
        p
    }

    fn target_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, x, y, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        n
    }

    fn open_world() -> tr_core::World {
        tr_core::World::new("t".into(), 300, 200)
    }

    /// The two rows carry their `SetDefaults` slices (`:8826-8841` /
    /// `:9807-9820`): 30x30, aiStyle 171, alpha 255, timeLeft 240,
    /// tileCollide TRUE, ignoreWater, extraUpdates 1; 931 penetrate 3 +
    /// local(30) + magic, 1039 penetrate 1 + local(-1) + no class flag.
    #[test]
    fn w4_d18_the_weapon_rows_carry_the_csharp_set_defaults() {
        let d = proj_data(FAIRY_QUEEN_MAGIC_ITEM_SHOT).expect("931 row");
        assert_eq!((d.w, d.h, d.ai_style), (30.0, 30.0, 171));
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.time_left, 240);
        assert_eq!(d.alpha, 255);
        assert!(d.tile_collide && d.ignore_water);
        assert_eq!(d.extra_updates, 1);
        assert!(d.uses_local_immunity && d.local_hit_cooldown == 30);
        assert!(d.magic && !d.melee && !d.ranged);
        let d = proj_data(CONSTELLATION_STAR).expect("1039 row");
        assert_eq!((d.w, d.h, d.ai_style), (30.0, 30.0, 171));
        assert_eq!(d.penetrate, 1, "the stock penetrate (`:774`)");
        assert_eq!(d.time_left, 240);
        assert!(d.tile_collide && d.ignore_water);
        assert_eq!(d.extra_updates, 1);
        assert!(d.uses_local_immunity && d.local_hit_cooldown == -1);
        assert!(!d.magic && !d.melee && !d.ranged, "no class flag");
    }

    /// The friendly gate claims exactly 931/1039; the hostile gate does NOT
    /// claim them (and the friendly gate does not claim the hostile five).
    #[test]
    fn w4_d18_the_friendly_gate_is_disjoint_from_the_hostile_one() {
        assert!(handles_fairy_weapon(FAIRY_QUEEN_MAGIC_ITEM_SHOT));
        assert!(handles_fairy_weapon(CONSTELLATION_STAR));
        assert!(!handles(FAIRY_QUEEN_MAGIC_ITEM_SHOT));
        assert!(!handles(CONSTELLATION_STAR));
        for kind in EMPRESS_PROJ_KINDS {
            assert!(!handles_fairy_weapon(kind));
        }
        for kind in [14u16, 871, 924, 932] {
            assert!(!handles_fairy_weapon(kind));
        }
    }

    /// The 238-tick dust burst (`:45475-45491`): the alpha save/restore,
    /// then 3× (CircularEdge float + radius float + NewDust's 6) = 24 draws
    /// in C# order; the alpha ends restored.
    #[test]
    fn w4_d18_the_238_tick_burst_consumes_24_draws_in_order() {
        let mut p = weapon(FAIRY_QUEEN_MAGIC_ITEM_SHOT);
        p.time_left = 238;
        p.alpha = 123;
        let w = open_world();
        let mut rng = RecordRng::default();
        fairy_weapon_tick(&mut p, 0, &w, &[], &[], &mut rng);
        assert_eq!(p.alpha, 123, "the alpha save/restore (`:45477-45484`)");
        let mut expect: Vec<String> = Vec::new();
        for _ in 0..3 {
            expect.push("float".into()); // NextVector2CircularEdge
            expect.push("float".into()); // the radius NextFloat
            expect.push("range(0,1)".into());
            expect.push("range(0,1)".into());
            expect.push("range(-20,21)".into());
            expect.push("range(-20,21)".into());
            expect.push("range(0,3)".into());
            expect.push("range(-20,21)".into());
        }
        assert_eq!(rng.draws, expect, "24 draws, exact order");
        // A non-238 tick draws nothing.
        let mut p2 = weapon(CONSTELLATION_STAR);
        p2.time_left = 200;
        let mut rng2 = RecordRng::default();
        fairy_weapon_tick(&mut p2, 0, &w, &[], &[], &mut rng2);
        assert!(rng2.draws.is_empty(), "no burst off the 238-tick");
    }

    /// The friendly acquire (`:45507-45526`): a dead target drops to -1 and
    /// re-acquires the nearest chaseable NPC; both stamp netUpdate.
    #[test]
    fn w4_d18_the_friendly_acquire_drops_and_reacquires() {
        let w = open_world();
        let mut p = weapon(FAIRY_QUEEN_MAGIC_ITEM_SHOT);
        p.time_left = 200; // > 180: the swirl phase, no homing
        p.ai[0] = 0.0;
        // A dead NPC at index 0, a live one at index 1.
        let mut dead = target_at(600.0, 335.0);
        dead.hp = 0;
        let live = target_at(700.0, 335.0);
        let npcs = [dead, live];
        let mut rng = RecordRng::default();
        fairy_weapon_tick(&mut p, 0, &w, &npcs, &[], &mut rng);
        assert_eq!(p.ai[0], 1.0, "re-acquired the live NPC");
        assert!(p.net_update, "the acquire stamps netUpdate");
        assert!(rng.draws.is_empty(), "the acquire rolls nothing");
    }

    /// The friendly homing (`:45535-45548`) + the speed ramp
    /// (`:45551-45554`): an in-range NPC pins timeLeft >= 10 and aims at
    /// its centre ×30; the SmoothStep amount is Lerp(0.075, 0.125, …) and
    /// the velocity scales by Lerp(0.85, 1, GetLerpValue(0, 90, timeLeft)).
    #[test]
    fn w4_d18_the_friendly_homing_pins_life_and_ramps_speed() {
        let w = open_world();
        let mut p = weapon(FAIRY_QUEEN_MAGIC_ITEM_SHOT);
        p.vel = Vec2::new(10.0, 0.0);
        p.time_left = 100; // 180 ≥ 100 > 20: the homing phase
        p.ai[0] = 0.0;
        let npcs = [target_at(960.0, 315.0)]; // centre (969,335) — level with proj centre (495,335)
        let mut rng = RecordRng::default();
        fairy_weapon_tick(&mut p, 0, &w, &npcs, &[], &mut rng);
        // amount = Lerp(0.075, 0.125, GetLerpValue(180, 30, 100)).
        let t = get_lerp_value(180.0, 30.0, 100.0);
        let amount = lerp(0.075, 0.125, t);
        let s = 3.0 * amount * amount - 2.0 * amount * amount * amount;
        let ramp = lerp(0.85, 1.0, get_lerp_value(0.0, 90.0, 100.0));
        assert!((p.vel.x - (10.0 + 20.0 * s) * ramp).abs() < 1e-3);
        assert!(p.vel.y.abs() < 1e-4);
        assert!(rng.draws.is_empty());
        // The life pin: a target holds the life at >= 10. Seed timeLeft 21
        // (> 20, the homing window) with a target — the pin keeps it >= 10.
        let mut p2 = weapon(CONSTELLATION_STAR);
        p2.time_left = 21;
        p2.ai[0] = 0.0;
        fairy_weapon_tick(&mut p2, 0, &w, &npcs, &[], &mut rng);
        assert!(p2.time_left >= 10, "the life pin");
        // No target: one extra life tick (the homing else, `:45547`).
        let mut p3 = weapon(FAIRY_QUEEN_MAGIC_ITEM_SHOT);
        p3.time_left = 100;
        p3.ai[0] = -1.0;
        let before = p3.time_left;
        fairy_weapon_tick(&mut p3, 0, &w, &[], &[], &mut rng);
        assert_eq!(p3.time_left, before - 1, "the no-target life tick");
    }

    /// The bounce (`:18592-18602`): a blocked axis reflects at −1×last and
    /// the position integrates the resolved velocity; open air integrates
    /// straight. The collision box is the 4×4 centre inset (`:20017`).
    #[test]
    fn w4_d18_the_bounce_reflects_the_blocked_axis() {
        // Open air: straight integrate.
        let w = open_world();
        let mut p = weapon(FAIRY_QUEEN_MAGIC_ITEM_SHOT);
        p.pos = Vec2::new(480.0, 320.0);
        p.vel = Vec2::new(5.0, -3.0);
        fairy_weapon_bounce(&mut p, &w);
        assert_eq!(p.pos, Vec2::new(485.0, 317.0));
        assert_eq!(p.vel, Vec2::new(5.0, -3.0));
        // A solid tile to the right: the X axis reflects, Y keeps.
        let mut w2 = open_world();
        // The 4×4 box centre is (495, 335); place a solid at tile (31, 20)
        // = px (496..512, 320..336) so the +X destination box overlaps it.
        let i = w2.index(31, 20);
        w2.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w2.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p2 = weapon(CONSTELLATION_STAR);
        p2.pos = Vec2::new(480.0, 320.0);
        p2.vel = Vec2::new(5.0, 0.0);
        fairy_weapon_bounce(&mut p2, &w2);
        assert_eq!(p2.vel.x, -5.0, "the X reflection (`:18596`)");
        assert_eq!(p2.pos.x, 475.0, "integrate the reflected velocity");
    }
}
