//! The aiStyle-74 family — the Martian Drone / Solar Corite hover-charge-
//! dash machine (`NPC.AI()`'s `else if (this.aiStyle == 74)` arm,
//! `NPC.cs:35218-35500`). S1-D-3, aiStyle-74 round, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! Census: `this.aiStyle = 74;` appears exactly TWICE in `SetDefaults` —
//! the MartianDrone 388 (`NPC.cs:13939`, row `:13935-13951`: 40×30, dmg
//! 60, def 16, lifeMax 300, kb 0.4, value 1000, noTileCollide/noGravity,
//! npcSlots 1.5, canGhostHeal false, `NPCID.cs:11453`) and the SolarCorite
//! 418 (`NPC.cs:15372`, row `:15368-15381`: 38×38, dmg 70, def 26, lifeMax
//! 600, kb 0.2, noTileCollide/noGravity, `NPCID.cs:11513`). Both kinds
//! exist in the roster (`npc/data.rs:1992` maps them to style 74), but the
//! live routing runs the kind-keyed stand-ins `update_martian_drone`
//! (`npc.rs:8700`) / `update_solar_corite` (`npc.rs:11597`) — this module
//! is the faithful UNWIRED port (the `npc_ai_style56.rs` precedent); the
//! defect catalogue against the stand-ins is in ANALYSIS "The Martian
//! Drone / Solar Corite machine (S1-D-3 aiStyle-74, code-only)".
//!
//! The machine (one parameter table, four states):
//! * Head (`:35220-35237`): `TargetClosest(faceTarget: false)` FIRST;
//!   `rotation = velocity.ToRotation()` folded back into ±π/2 (the two
//!   `Math.PI` adds/subtracts); `spriteDirection = -Math.Sign(velocity.X)`
//!   when the sign is non-zero — the 418 OVERWRITES it unconditionally
//!   with `+Math.Sign(velocity.X)` (`:35234-35237`, even 0).
//! * The parameter rows (`:35238-35275`): [`style74_params`] — the 388
//!   defaults vs the 418 overrides; `num1038 *= num1037` folds the 418's
//!   `4f/15f` homing base into `×60` (kept as the literal f32 product,
//!   NOT the rounded 16).
//! * State 0 (`:35308-35355`): the hover approach — the toward-player
//!   vector and the `num1026`-above / `num1027`-beside hover vector, both
//!   normalized × speed; the `CanHit` 1×1 sight gate with the `ai[3] >=
//!   120` argue override and the ±π/8 below-cone; the near/far bands; the
//!   `num1030`-tick damp or the state-1 latch storing the aim in
//!   `ai[2]`/`ai[3]`.
//! * State 1 (`:35356-35391`): the charge — the `num1032` decay (the 418
//!   gates it on `velocity.Length() > 2` and lerps a stalled corite
//!   toward the player at 0.25, `:35360-35367`); at `num1031` ticks the
//!   release: the stored aim + the `Next(-num1033, num1033+1) × 0.04f`
//!   jitter pair, normalized × `num1034`.
//! * State 2 (`:35392-35431`): the dash — the `num1037`-lerp toward the
//!   player with the `velocity.Length() + num1038` speed term (the NaN
//!   fallback is `(direction, 0)`); the time+past exit or the
//!   `num1039` speed-floor exit halves the velocity (the 418 exits into
//!   state 4 with `ai[1] = 45` instead, `:35406-35410`); the 388-only
//!   `SolidCollision` fold into state 3 (`flag75`, `:35423-35430`).
//! * State 4 (`:35432-35442`, 418-only): the cooldown — `ai[1] -= 3`,
//!   `velocity *= 0.95`, at `<= 0` back to state 0.
//! * The 388-only 64-px contact gate (`:35443-35450`) and the state-3
//!   implosion tail (`:35455-35499`): the 192×192 box about the OLD
//!   centre (the `Center` get uses FLOAT halves, `Entity.cs:50-55`; the
//!   `-= width/2` subtracts are INTEGER 192/2 = 96), velocity zero,
//!   `damage = GetAttackDamage_ScaledByDifficulty(80)`, `alpha = 255`,
//!   the constant `AddLight(0.2, 0.7, 1.1)`, the 10×dust-31 + 40×dust-226
//!   bursts, and the 3-tick fuse → `PlaySound(Item14)` + `life = 0` +
//!   `HitEffect()` + `active = false`.
//!
//! ## RNG order (every `Main.rand` draw cited)
//!
//! All draws ride the caller's `UnifiedRandom` (the `Main.rand`
//! analogue); the dust spawns roll it INSIDE `DustPool::new_dust`
//! (`Dust.cs:75-256`, the S1-E-1 pool — pressure bands, the default
//! block's 6 draws, the per-type tweaks; type 6 adds the `Next(-10, 6)`
//! vy re-roll, `Dust.cs:215-219`; types 226/31 hit no tweak). In C#
//! order, per tick:
//! 1. `:35276-35290` — 388 && `ai[0] != 3`: ONE `Dust.NewDust` 226 (no
//!    explicit `Next`; the pool's internal draws only).
//! 2. `:35291-35307` — 418 (EVERY state, including 3): TWO gate draws
//!    `Main.rand.Next(3)` (`:35297`, compared `< num1041` = 1, or 2 in
//!    state 2); each success spawns one `Dust.NewDust` 6 (internal
//!    draws). The gates are drawn even when the spawn is refused.
//! 3. `:35378` — the charge release (both kinds): `Next(-num1033,
//!    num1033+1)` TWICE (x then y). The 418's `num1033 = 0` makes these
//!    `Next(0, 1)` — the C# STILL draws `Sample()` for the empty range
//!    (`UnifiedRandom.cs:126-130`, the port's `next_range` keeps it), so
//!    the corite's release consumes two draws that always return 0.
//! 4. `:35382` — 388 state 1: `Next(4) == 0` EVERY state-1 tick (drawn
//!    AFTER the release branch, so the release tick draws it too); on 0
//!    one `Dust.NewDust` 226 (internal draws).
//! 5. `:35463-35491` — the state-3 implosion, EVERY state-3 tick:
//!    10 × (`NewDust` 31 internals + `NextDouble()` angle + `NextDouble()`
//!    radius, `:35467`), then 40 × (`NewDust` 226 internals +
//!    `NextDouble()` ×2 (`:35474`) + the `Next(2)` gate (`:35476`, on 0
//!    an extra `NewDust` 226 + `NextDouble()` ×2 (`:35481`)) + the
//!    `Next(4)` gate (`:35484`, on 0 an extra `NewDust` 226 +
//!    `NextDouble()` ×2 (`:35488`))). The gates are drawn every
//!    iteration regardless of the previous gate's outcome.
//!
//! NOT drawn here (documented skips / other passes):
//! * `PlaySound(SoundID.Item14)` (`:35495`) — the audio pass owns the
//!   `LegacySoundPlayer` draws (type 2 style 14 hits the default pitch
//!   jitter `Next(-6, 7)`, `LegacySoundPlayer.cs:631-633`; the volume
//!   gate `:433-436` can refuse before it). Returned as
//!   [`Style74Outcome::sound_item14`] (the `npc_ai_style41.rs` precedent).
//! * `HitEffect()` (`:35497`) — the strike/death pass, not the AI arm:
//!   the 388 arm (`NPC.cs:84470-84501`) and the 418 arm (`:82959-82993`)
//!   are pinned as count-only plans ([`drone_hit_dust_plan`] /
//!   [`corite_hit_dust_plan`]); the wiring wave must draw their
//!   `NewDust`/`Gore.NewGore` internals in C# order when the death feed
//!   lands (the `npc_ai_style72.rs` precedent).
//! * The 388's ambient cry `Next(700)` (`NPC.cs:90171`) lives in
//!   `UpdateNPC`'s sound block BEFORE `AI()` — the wiring caller's
//!   obligation, not this arm's.
//! * `TargetClosest` draws nothing (`npc_target.rs`, the scan is
//!   deterministic).
//!
//! ## Seams (the wiring obligations)
//!
//! * `knockBackResist` — `Npc` has no mutable kb field (the kind table
//!   is a fn); the state-0/1/2 writes are RETURNED in
//!   [`Style74Outcome::knock_back_resist`] (the `Style39Context`
//!   precedent). The difficulty curves are ported verbatim
//!   ([`linear_curve_sample`], `GameDifficultyData.cs:84`/`:86`,
//!   `LinearCurve.Sample` `:35-55`); `env.difficulty` defaults to the
//!   classic 1.0 (`GameDifficultyLevel.cs:18`).
//! * The implosion's `AddLight` (`:35462`) is returned as
//!   [`Style74Outcome::light`] for the `EntityLights` feed
//!   (`CenterTruncated` anchor — `(int)Center.X / 16`, the centre read
//!   AFTER the resize equals the OLD centre exactly).
//! * `life = 0` + `active = false` (`:35496-35498`) — the port sets
//!   `hp = 0` and returns `died`; the caller owes the `HitEffect()` +
//!   deactivation pair. There is NO blast projectile in the C# arm —
//!   the live corite stand-in's `wants_blast` is an invention (ANALYSIS).
//! * The arm does NOT integrate position: both kinds are
//!   noTileCollide/noGravity (`SetDefaults`), so `UpdateNPC`'s raw
//!   integration (`NPC.cs:90494-90498`) runs after `AI()` — the caller's
//!   movement step (the `npc_ai_style56.rs` split).
//! * `FindFrame` — the 388's `case 388` (`NPC.cs:71731-71742`, the
//!   2-frame 6-tick loop, `Main.npcFrameCount[388] = 2`, `Main.cs:67122`)
//!   and the 418's `case 418` (`:70758-70800`, the state-driven rows,
//!   `npcFrameCount[418] = 8`, `Main.cs:67125`) are pure twins
//!   ([`drone_find_frame`] / [`corite_find_frame`], the
//!   `nimbus_find_frame` precedent — frame INDICES, the sheet height is
//!   the draw side's).
//! * `Math.Sign(velocity.X)` on a NaN velocity throws in the C#
//!   (`Math.Sign(Single)` rejects NaN); the port's explicit `> 0 / < 0`
//!   comparisons fold NaN to 0 — unreachable from vanilla writes (the
//!   only NaN source is the unguarded `Normalize` on a zero delta, which
//!   the state-2 `HasNaNs` guard catches before it can reach the head).
//!
//! See docs/ANALYSIS.md "The Martian Drone / Solar Corite machine
//! (S1-D-3 aiStyle-74, code-only)".
#![allow(dead_code)] // wiring deferred — the stand-in swap is the merge step

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The per-tick environment the arm reads through `Main`'s statics
/// (the `Style56Env` shape).
pub struct Style74Env<'a> {
    /// The read-only world the sight gate / `SolidCollision` read.
    pub world: &'a World,
    /// The `Main.player` slots as `TargetClosest` sees them (the port's
    /// session player is slot 0).
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`).
    pub ticker: u64,
    /// `Main.dust` — the shared 6001-slot pool (`Dust.cs:75-256`).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// The `Main.rand` stream — every draw cited in the module header
    /// rides it.
    pub rng: &'a mut UnifiedRandom,
    /// `NPC.difficulty` (`NPC.cs:7033`/`:7038`) — the classic 1.0 until
    /// the difficulty system lands (`GameDifficultyLevel.cs:18`).
    pub difficulty: f32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style74Outcome {
    /// The `knockBackResist` write: `Some(scaled)` in state 0
    /// (`:35310`), `Some(0)` in states 1/2 (`:35358`/`:35394`), `None`
    /// in states 3/4 (no write).
    pub knock_back_resist: Option<f32>,
    /// The state-3 `Lighting.AddLight` (`:35462`):
    /// `(tileX, tileY, [0.2, 0.7, 1.1])` at the truncated centre tile.
    pub light: Option<(i32, i32, [f32; 3])>,
    /// The fuse expiry (`:35496-35498`): `hp` is already 0; the caller
    /// owes the `HitEffect()` + deactivation pair.
    pub died: bool,
    /// The `PlaySound(SoundID.Item14, position)` request (`:35495`) —
    /// the audio pass owns its draws (module header).
    pub sound_item14: bool,
}

/// The `:35238-35275` parameter table — the 388 defaults with the 418
/// overrides. Field names follow the C# locals' roles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style74Params {
    /// `normalKnockbackMult` (`:35238`/`:35257`).
    pub kb: f32,
    /// `num1025` — the approach speed (`:35239`/`:35258`).
    pub speed: f32,
    /// `num1026` — the hover height above the player (`:35240`/`:35259`).
    pub hover: f32,
    /// `num1027` — the hover side-shift (`:35241`/`:35260`).
    pub side_shift: f32,
    /// `num1028` — the far band (`:35242`/`:35261`).
    pub far: f32,
    /// `num1029` — the near band (`:35243`/`:35262`).
    pub near: f32,
    /// `num1030` — the approach damp ticks (`:35244`/`:35263`).
    pub damp: f32,
    /// `num1031` — the charge ticks (`:35245`/`:35264`).
    pub charge_ticks: f32,
    /// `num1032` — the charge decay (`:35246`/`:35265`).
    pub decay: f32,
    /// `num1033` — the release jitter bound (`:35247`/`:35266`).
    pub jitter: i32,
    /// `num1034` — the dash speed (`:35248`/`:35267`).
    pub dash_speed: f32,
    /// `num1035` — the dash time cap (`:35249`/`:35268`).
    pub dash_ticks: f32,
    /// `num1036` — the dash turn distance (`:35250`/`:35269`).
    pub turn_dist: f32,
    /// `num1037` — the dash homing lerp (`:35251`/`:35270`).
    pub lerp: f32,
    /// `num1038` AFTER the `*= num1037` fold (`:35252`/`:35271`/`:35275`).
    pub home_term: f32,
    /// `num1039` — the dash speed floor (`:35253`/`:35272`).
    pub speed_floor: f32,
    /// `flag75` (`:35254`/`:35273`) — the 388's wall/contact explosion
    /// gates; false for the 418.
    pub explodes_on_contact: bool,
}

/// The two parameter rows, verbatim (`NPC.cs:35238-35275`). The 418's
/// `home_term` keeps the literal `(4f / 15f) * 60f` product — NOT a
/// rounded 16 (the f32 round-off is part of the dash maths).
pub fn style74_params(corite: bool) -> Style74Params {
    if corite {
        Style74Params {
            kb: 0.3,
            speed: 8.0,
            hover: 175.0,
            side_shift: 175.0,
            far: 600.0,
            near: 80.0,
            damp: 60.0,
            charge_ticks: 20.0,
            decay: 0.75,
            jitter: 0,
            dash_speed: 9.0,
            dash_ticks: 30.0,
            turn_dist: 150.0,
            lerp: 60.0,
            home_term: (4.0f32 / 15.0f32) * 60.0f32,
            speed_floor: 7.0,
            explodes_on_contact: false,
        }
    } else {
        Style74Params {
            kb: 0.4,
            speed: 10.0,
            hover: 200.0,
            side_shift: 0.0,
            far: 750.0,
            near: 0.0,
            damp: 30.0,
            charge_ticks: 30.0,
            decay: 0.95,
            jitter: 50,
            dash_speed: 14.0,
            dash_ticks: 30.0,
            turn_dist: 100.0,
            lerp: 20.0,
            home_term: 0.0 * 20.0,
            speed_floor: 7.0,
            explodes_on_contact: true,
        }
    }
}

/// `LinearCurve.Sample` (`GameDifficultyData.cs:35-55`) — the bracket
/// scan, the zero-span guard returns `key.output`, else `(value -
/// key.input) * num2 / num + key.output` (the operation order kept).
fn linear_curve_sample(keys: &[(f32, f32)], value: f32) -> f32 {
    let mut key = keys[0];
    let mut key2 = key;
    for k in keys {
        key2 = *k;
        if value <= key2.0 {
            break;
        }
        key = key2;
    }
    let num = key2.0 - key.0;
    let num2 = key2.1 - key.1;
    if num == 0.0 {
        return key.1;
    }
    (value - key.0) * num2 / num + key.1
}

/// `GameDifficultyData.EnemyDamageMultiplier` (`GameDifficultyData.cs:84`):
/// Journey 0.5→0.5, Master 3→3, Legendary 4→5.3333335.
const ENEMY_DAMAGE_KEYS: [(f32, f32); 3] = [(0.5, 0.5), (3.0, 3.0), (4.0, 5.3333335)];

/// `GameDifficultyData.KnockbackToEnemiesMultiplier`
/// (`GameDifficultyData.cs:86`): Classic 1→1, Master 3→0.8.
const KNOCKBACK_KEYS: [(f32, f32); 2] = [(1.0, 1.0), (3.0, 0.8)];

/// `NPC.GetKnockbackMultiplier_ScaledByDifficulty` (`NPC.cs:7031-7034`).
fn knockback_scaled(normal: f32, difficulty: f32) -> f32 {
    normal * linear_curve_sample(&KNOCKBACK_KEYS, difficulty)
}

/// `NPC.GetAttackDamage_ScaledByDifficulty` (`NPC.cs:7036-7039`) — the
/// `(int)` truncation of the scaled product.
fn attack_damage_scaled(normal: f32, difficulty: f32) -> i32 {
    (normal * linear_curve_sample(&ENEMY_DAMAGE_KEYS, difficulty)) as i32
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()`/`Distance`:
/// the f32 sum widened to f64 for the root, cast back (the
/// `npc_ai_style55.rs::sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — `num = 1f / (float)Math.Sqrt(X*X+Y*Y);
/// X *= num; Y *= num`. UNGUARDED: a zero vector gives NaN, exactly like
/// the C# (`:35317-35318`, `:35379`, `:35388`, `:35416`, `:35475`). The
/// state-2 `HasNaNs` check (`:35417`) is the arm's own guard.
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `Utils.SafeNormalize` (`Utils.cs:1837-1844`): zero OR NaN → the
/// default; else `Normalize`.
fn safe_normalize(x: f32, y: f32, dx: f32, dy: f32) -> (f32, f32) {
    if (x == 0.0 && y == 0.0) || x.is_nan() || y.is_nan() {
        (dx, dy)
    } else {
        normalize_xna(x, y)
    }
}

/// `Utils.RotatedBy` about the origin (`Utils.cs:1675-1684`): the f64
/// cos/sin cast to f32, then `x·cos − y·sin` / `x·sin + y·cos`.
fn rotated_by(x: f32, y: f32, radians: f32) -> (f32, f32) {
    let num = (radians as f64).cos() as f32;
    let num2 = (radians as f64).sin() as f32;
    (x * num - y * num2, x * num2 + y * num)
}

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): `((float)cos(f),
/// (float)sin(f))` — the f64 trig over the f32 angle.
fn to_rotation_vector2(f: f32) -> (f32, f32) {
    ((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// The target slot's candidate — the `npc_ai_style55.rs::target_of`
/// fallback (the C# indexes `Main.player[this.target]`, 255 slots always
/// exist; an unset/stale slot falls back to slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Entity.Center` (`Entity.cs:50-55`) for a player candidate.
fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + p.width as f32 / 2.0, p.y + p.height as f32 / 2.0)
}

/// One fixed step of the aiStyle-74 arm (`NPC.cs:35218-35500`), in C#
/// order. Only the two census kinds ride the style; any other kind is a
/// no-op (the C# arm is unreachable for them).
pub fn style74_step(npc: &mut Npc, env: &mut Style74Env<'_>) -> Style74Outcome {
    let corite = match npc.kind {
        NpcKind::MartianDrone => false,
        NpcKind::SolarCorite => true,
        _ => return Style74Outcome::default(),
    };
    let p = style74_params(corite);
    let mut out = Style74Outcome::default();

    // `:35220` — TargetClosest(faceTarget: false) FIRST.
    target_closest(npc, env.world, env.players, false, 74, env.ticker);
    // `:35221-35229` — rotation = velocity.ToRotation() (atan2 in f64
    // over the f32 components), folded back into ±π/2.
    npc.rotation = (npc.vy as f64).atan2(npc.vx as f64) as f32;
    if npc.rotation < -(std::f32::consts::PI / 2.0) {
        npc.rotation += std::f32::consts::PI;
    }
    if npc.rotation > std::f32::consts::PI / 2.0 {
        npc.rotation -= std::f32::consts::PI;
    }
    // `:35230-35237` — spriteDirection = -Sign(vx) when non-zero; the
    // 418 overwrites unconditionally with +Sign(vx). `Math.Sign(Single)`
    // returns 0 for BOTH ±0 (Rust's signum(-0.0) is -1 — the explicit
    // comparisons keep the C# shape).
    let sign = if npc.vx > 0.0 {
        1
    } else if npc.vx < 0.0 {
        -1
    } else {
        0
    };
    if sign != 0 {
        npc.sprite_dir = -sign;
    }
    if corite {
        npc.sprite_dir = sign;
    }

    // `:35276-35290` — the 388 trail dust (ai[0] != 3, checked BEFORE
    // the state machine). One NewDust 226, no explicit Next.
    if !corite && npc.ai[0] != 3.0 {
        let (w, h) = npc.size();
        // `:35278`/`:35289` — the netOffset round trip, verbatim.
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        let num1040 = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(npc.x, npc.y),
            w as i32,
            h as i32,
            226,
            0.0,
            0.0,
            100,
            Color::default(),
            0.5,
        );
        {
            let d = env.dust.slot_mut(num1040);
            d.no_gravity = true;
            // `:35281` — velocity = base.velocity / 5f.
            d.velocity = Vec2::new(npc.vx, npc.vy) / 5.0;
            // `:35282-35288` — the (-10, 10) spin point, X flipped when
            // spriteDirection == 1, RotatedBy(rotation), then position
            // = Center + spinningpoint (the Centre read AFTER the
            // netOffset add — the C# reads base.Center there too).
            let mut sx = -10.0f32;
            let sy = 10.0f32;
            if npc.sprite_dir == 1 {
                sx *= -1.0;
            }
            let (rx, ry) = rotated_by(sx, sy, npc.rotation);
            let (cx, cy) = npc.center();
            d.position = Vec2::new(cx + rx, cy + ry);
        }
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    // `:35291-35307` — the 418 ember dust: TWO Next(3) gates EVERY tick
    // (all states, including 3); threshold 1 (2 in state 2), box half
    // 20 (30 in state 2).
    if corite {
        let num1041 = if npc.ai[0] != 2.0 { 1 } else { 2 };
        let num1042 = if npc.ai[0] == 2.0 { 30 } else { 20 };
        for _ in 0..2 {
            if env.rng.next_max(3) < num1041 {
                npc.x += npc.net_offset.x;
                npc.y += npc.net_offset.y;
                // `:35300` — base.Center is read AFTER the netOffset add.
                let (cx, cy) = npc.center();
                let num1044 = env.dust.new_dust(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(cx - num1042 as f32, cy - num1042 as f32),
                    num1042 * 2,
                    num1042 * 2,
                    6,
                    npc.vx * 0.5,
                    npc.vy * 0.5,
                    90,
                    Color::default(),
                    1.5,
                );
                {
                    let d = env.dust.slot_mut(num1044);
                    d.no_gravity = true;
                    d.velocity *= 0.2;
                    d.fade_in = 1.0;
                }
                npc.x -= npc.net_offset.x;
                npc.y -= npc.net_offset.y;
            }
        }
    }

    let (pcx, pcy) = candidate_center(target_of(npc, env.players));

    if npc.ai[0] == 0.0 {
        // `:35310` — the difficulty-scaled kb write.
        out.knock_back_resist = Some(knockback_scaled(p.kb, env.difficulty));
        // `:35311-35318` — the two approach vectors.
        let num1045 = p.speed;
        let (cx, cy) = npc.center();
        let mut v128x = pcx - cx;
        let mut v128y = pcy - cy;
        let mut v129x = v128x;
        let mut v129y = v128y - p.hover;
        v129x += if v128x < 0.0 { p.side_shift } else { 0.0 - p.side_shift };
        let num1046 = sqrt_f32(v128x, v128y);
        let (nx, ny) = normalize_xna(v128x, v128y);
        v128x = nx * num1045;
        v128y = ny * num1045;
        let (hx, hy) = normalize_xna(v129x, v129y);
        v129x = hx * num1045;
        v129y = hy * num1045;
        // `:35319-35325` — the sight gate: CanHit 1×1, the ai[3] >= 120
        // argue override, then the ±π/8 below-cone on the SCALED aim.
        let mut flag76 =
            crate::projectile::can_hit(env.world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
        if npc.ai[3] >= 120.0 {
            flag76 = true;
        }
        let aim_rot = (v128y as f64).atan2(v128x as f64) as f32;
        flag76 = flag76
            && aim_rot > std::f32::consts::PI / 8.0
            && aim_rot < std::f32::consts::PI - std::f32::consts::PI / 8.0;
        // `:35326-35327` — the near/far bands.
        let flag77 = num1046 < p.near;
        let flag78 = num1046 > p.far;
        if (flag77 | flag78) || !flag76 {
            // `:35330-35331` — the num1030-tick damp toward the hover
            // vector.
            npc.vx = (npc.vx * (p.damp - 1.0) + v129x) / p.damp;
            npc.vy = (npc.vy * (p.damp - 1.0) + v129y) / p.damp;
            if !flag76 {
                // `:35334-35341` — the argue timer (only while in-band).
                if !flag77 && !flag78 {
                    npc.ai[3] += 1.0;
                }
                if npc.ai[3] == 120.0 {
                    npc.net_update = true;
                }
            } else {
                // `:35345` — sight regained: the timer resets.
                npc.ai[3] = 0.0;
            }
        } else {
            // `:35350-35353` — the charge latch: the aim rides ai[2]/ai[3].
            npc.ai[0] = 1.0;
            npc.ai[2] = v128x;
            npc.ai[3] = v128y;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 1.0 {
        // `:35358` — kb 0.
        out.knock_back_resist = Some(0.0);
        // `:35359-35367` — the 418's stalled-charge lerp.
        let mut flag79 = true;
        if corite {
            flag79 = sqrt_f32(npc.vx, npc.vy) > 2.0;
            let player = target_of(npc, env.players);
            if !flag79 && npc.target >= 0 && !player.dead {
                let (cx, cy) = npc.center();
                let (sx, sy) = safe_normalize(pcx - cx, pcy - cy, 0.0, 0.0);
                // MathHelper.Lerp(v, aim*0.1, 0.25) = v + (aim*0.1 - v)*0.25.
                npc.vx += (sx * 0.1 - npc.vx) * 0.25;
                npc.vy += (sy * 0.1 - npc.vy) * 0.25;
            }
        }
        // `:35368-35371` — the decay.
        if flag79 {
            npc.vx *= p.decay;
            npc.vy *= p.decay;
        }
        // `:35372-35381` — the release.
        npc.ai[1] += 1.0;
        if npc.ai[1] >= p.charge_ticks {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
            // `:35378` — the jitter pair: Next(-num1033, num1033+1) ×2,
            // each × 0.04f. The 418's Next(0, 1) still draws Sample.
            let jx = env.rng.next_range(-p.jitter, p.jitter + 1) as f32 * 0.04;
            let jy = env.rng.next_range(-p.jitter, p.jitter + 1) as f32 * 0.04;
            let (dx, dy) = normalize_xna(npc.ai[2] + jx, npc.ai[3] + jy);
            npc.vx = dx * p.dash_speed;
            npc.vy = dy * p.dash_speed;
        }
        // `:35382-35390` — the 388 charge sparkle: Next(4) EVERY state-1
        // tick (the release tick included).
        if !corite && env.rng.next_max(4) == 0 {
            let (w, h) = npc.size();
            npc.x += npc.net_offset.x;
            npc.y += npc.net_offset.y;
            let num1048 = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                226,
                0.0,
                0.0,
                100,
                Color::default(),
                0.5,
            );
            {
                let d = env.dust.slot_mut(num1048);
                d.no_gravity = true;
                d.velocity *= 2.0;
                let (cx, cy) = npc.center();
                let (nx, ny) = normalize_xna(d.position.x - cx, d.position.y - cy);
                d.velocity = d.velocity / 2.0 + Vec2::new(nx, ny);
            }
            npc.x -= npc.net_offset.x;
            npc.y -= npc.net_offset.y;
        }
    } else if npc.ai[0] == 2.0 {
        // `:35394` — kb 0.
        out.knock_back_resist = Some(0.0);
        // `:35395-35398` — the dash exit gates.
        let num1049 = p.dash_ticks;
        npc.ai[1] += 1.0;
        let (cx, cy) = npc.center();
        let num1050 = sqrt_f32(cx - pcx, cy - pcy) > p.turn_dist && cy > pcy;
        if ((npc.ai[1] >= num1049) & num1050) || sqrt_f32(npc.vx, npc.vy) < p.speed_floor {
            // `:35400-35405` — the exit: all ai zeroed, velocity halved.
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.vx /= 2.0;
            npc.vy /= 2.0;
            npc.net_update = true;
            if corite {
                // `:35406-35410` — the 418 exits into the cooldown.
                npc.ai[1] = 45.0;
                npc.ai[0] = 4.0;
            }
        } else {
            // `:35414-35421` — the homing lerp with the speed term.
            let (mut nx, mut ny) = normalize_xna(pcx - cx, pcy - cy);
            if nx.is_nan() || ny.is_nan() {
                // `:35417-35420` — the HasNaNs fallback: (direction, 0).
                nx = npc.facing as f32;
                ny = 0.0;
            }
            let vlen = sqrt_f32(npc.vx, npc.vy);
            npc.vx = (npc.vx * (p.lerp - 1.0) + nx * (vlen + p.home_term)) / p.lerp;
            npc.vy = (npc.vy * (p.lerp - 1.0) + ny * (vlen + p.home_term)) / p.lerp;
        }
        // `:35423-35430` — the 388-only wall fold into the implosion.
        if p.explodes_on_contact {
            let (w, h) = npc.size();
            if crate::player::solid_collision(env.world, npc.x, npc.y, w, h) {
                npc.ai[0] = 3.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
    } else if npc.ai[0] == 4.0 {
        // `:35432-35442` — the 418 cooldown (no kb write).
        npc.ai[1] -= 3.0;
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
        }
        npc.vx *= 0.95;
        npc.vy *= 0.95;
    }
    // `:35443-35450` — the 388-only 64-px contact gate (AFTER the state
    // machine, so an exit tick can still contact-explode).
    if p.explodes_on_contact && npc.ai[0] != 3.0 {
        let (cx, cy) = npc.center();
        if sqrt_f32(cx - pcx, cy - pcy) < 64.0 {
            npc.ai[0] = 3.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
    }
    // `:35451-35454` — the non-3 return.
    if npc.ai[0] != 3.0 {
        return out;
    }

    // `:35455-35458` — the implosion box: position = Center (the OLD
    // box's FLOAT halves), width = height = 192, then -= width/2 /
    // height/2 (INTEGER 192/2 = 96). The new Centre equals the old one.
    let (cx, cy) = npc.center();
    npc.size_override = Some((192.0, 192.0));
    npc.x = cx - (192i32 / 2) as f32;
    npc.y = cy - (192i32 / 2) as f32;
    // `:35459-35461` — velocity zero, the scaled 80 damage, alpha 255.
    npc.vx = 0.0;
    npc.vy = 0.0;
    npc.damage_override = Some(attack_damage_scaled(80.0, env.difficulty));
    npc.alpha = 255;
    // `:35462` — AddLight((int)Center.X/16, (int)Center.Y/16,
    // 0.2, 0.7, 1.1) — the Centre read AFTER the resize.
    let (w, h) = npc.size();
    out.light = Some((
        tile_x(Vec2::new(npc.x, npc.y), w, XAnchor::CenterTruncated),
        tile_y(Vec2::new(npc.x, npc.y), h, XAnchor::CenterTruncated),
        [0.2, 0.7, 1.1],
    ));
    let (ecx, ecy) = npc.center();
    // `:35463-35468` — the 10 dust-31 ring: NewDust internals, then the
    // NextDouble angle + NextDouble radius position overwrite.
    for _ in 0..10 {
        let num1052 = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(npc.x, npc.y),
            w as i32,
            h as i32,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = env.dust.slot_mut(num1052);
        d.velocity *= 1.4;
        let angle = env.rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
        let radius = env.rng.next_double() as f32 * 96.0;
        let (ux, uy) = to_rotation_vector2(angle);
        d.position = Vec2::new(ux * radius + ecx, uy * radius + ecy);
    }
    // `:35469-35491` — the 40 dust-226 ring with the two extra-spawn
    // gates (both drawn EVERY iteration).
    for _ in 0..40 {
        let mut num1054 = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(npc.x, npc.y),
            w as i32,
            h as i32,
            226,
            0.0,
            0.0,
            100,
            Color::default(),
            0.5,
        );
        {
            let d = env.dust.slot_mut(num1054);
            d.no_gravity = true;
            d.velocity *= 2.0;
            let angle = env.rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
            let radius = env.rng.next_double() as f32 * 96.0;
            let (ux, uy) = to_rotation_vector2(angle);
            d.position = Vec2::new(ux * radius + ecx, uy * radius + ecy);
            let (nx, ny) = normalize_xna(d.position.x - ecx, d.position.y - ecy);
            d.velocity = d.velocity / 2.0 + Vec2::new(nx, ny);
        }
        // `:35476-35483` — the Next(2) extra (scale 0.9, noGravity).
        if env.rng.next_max(2) == 0 {
            num1054 = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                226,
                0.0,
                0.0,
                100,
                Color::default(),
                0.9,
            );
            let d = env.dust.slot_mut(num1054);
            d.no_gravity = true;
            d.velocity *= 1.2;
            let angle = env.rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
            let radius = env.rng.next_double() as f32 * 96.0;
            let (ux, uy) = to_rotation_vector2(angle);
            d.position = Vec2::new(ux * radius + ecx, uy * radius + ecy);
            let (nx, ny) = normalize_xna(d.position.x - ecx, d.position.y - ecy);
            d.velocity = d.velocity / 2.0 + Vec2::new(nx, ny);
        }
        // `:35484-35490` — the Next(4) extra (scale 0.7, gravity ON).
        if env.rng.next_max(4) == 0 {
            num1054 = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                226,
                0.0,
                0.0,
                100,
                Color::default(),
                0.7,
            );
            let d = env.dust.slot_mut(num1054);
            d.velocity *= 1.2;
            let angle = env.rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
            let radius = env.rng.next_double() as f32 * 96.0;
            let (ux, uy) = to_rotation_vector2(angle);
            d.position = Vec2::new(ux * radius + ecx, uy * radius + ecy);
            let (nx, ny) = normalize_xna(d.position.x - ecx, d.position.y - ecy);
            d.velocity = d.velocity / 2.0 + Vec2::new(nx, ny);
        }
    }
    // `:35492-35499` — the fuse: ai[1]++ every state-3 tick; at >= 3 the
    // Item14 sound (the audio pass), life = 0, HitEffect + deactivate.
    npc.ai[1] += 1.0;
    if npc.ai[1] >= 3.0 {
        out.sound_item14 = true;
        npc.hp = 0;
        out.died = true;
    }
    out
}

/// `Main.npcFrameCount[388]` = 2 (`Main.cs:67122`, the 380-389 row
/// `12, 9, 9, 9, 1, 9, 11, 2, 2, 9`, index 388 = the 9th entry).
pub const DRONE_FRAME_COUNT: u32 = 2;

/// `Main.npcFrameCount[418]` = 8 (`Main.cs:67125`, the 410-419 row
/// `5, 11, 1, 1, 1, 10, 10, 15, 8, 17`, index 418 = the 9th entry).
pub const CORITE_FRAME_COUNT: u32 = 8;

/// `FindFrame`'s `case 388` (`NPC.cs:71731-71742`): `frameCounter++`, at
/// `>= 6.0` advance + reset, wrap at `num * 2` → 0. Frame INDICES (the
/// `nimbus_find_frame` convention); the sheet height is the draw side's.
pub fn drone_find_frame(frame: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    let mut row = frame;
    if counter >= 6.0 {
        counter = 0.0;
        row += 1;
        if row >= DRONE_FRAME_COUNT {
            row = 0;
        }
    }
    (row, counter)
}

/// `FindFrame`'s `case 418` (`NPC.cs:70758-70800`): states 0/1/-1 cycle
/// rows 0-4 on the STRICT `> 6.0` counter; state 2 pins rows 5/6/7 by
/// the `ai[1] < 10 / < 20` bands; state -2 cycles rows 5-7 on `> 12.0`.
/// States 3/4 hit NO arm — the frame holds (the C# switch falls through).
pub fn corite_find_frame(frame: u32, frame_counter: f32, ai0: f32, ai1: f32) -> (u32, f32) {
    let mut counter = frame_counter;
    let mut row = frame;
    if ai0 == 0.0 || ai0 == 1.0 || ai0 == -1.0 {
        counter += 1.0;
        if counter > 6.0 {
            counter = 0.0;
            row += 1;
        }
        if row > 4 {
            row = 0;
        }
    } else if ai0 == 2.0 {
        row = if ai1 < 10.0 {
            5
        } else if ai1 < 20.0 {
            6
        } else {
            7
        };
    } else if ai0 == -2.0 {
        counter += 1.0;
        if counter > 12.0 {
            counter = 0.0;
            row += 1;
        }
        if row > 7 {
            row = 5;
        }
    }
    (row, counter)
}

/// The 388 `HitEffect` dust plan (`NPC.cs:84470-84501`) — counts only,
/// the documented visual skip (module header). NOTE the hit bound is
/// `dmg / (double)(lifeMax * 50)` — the INT product `lifeMax * 50`
/// (`:84474`), NOT the spore family's `dmg / lifeMax * 50`.
pub fn drone_hit_dust_plan(life: i32, dmg: f64, life_max: i32) -> u32 {
    if life > 0 {
        let bound = dmg / (life_max * 50) as f64;
        let mut count = 0u32;
        while (count as f64) < bound {
            count += 1;
        }
        count
    } else {
        // `:84483-84499` — the flat 10 dust 226 + gore 605/606 (the
        // ai[0] == 3 death rides the rotation vector ×5, `:84489-84497`).
        10
    }
}

/// The 418 `HitEffect` dust plan (`NPC.cs:82959-82993`) — counts only.
/// The hit bound is `dmg / lifeMax * 20.0` (`:82963`); each hit iteration
/// spawns one dust 241 plus a `Next(2)`-gated dust 6 (`:82966-82973`);
/// the death half is a flat 20 × (241 + 6) pairs + 5 gores (841, 842×2,
/// 843×2, `:82987-82991`). Returns the 241 count (the gated 6s and the
/// gore draws ride the wiring wave's death feed).
pub fn corite_hit_dust_plan(life: i32, dmg: f64, life_max: i32) -> u32 {
    if life > 0 {
        let bound = dmg / life_max as f64 * 20.0;
        let mut count = 0u32;
        while (count as f64) < bound {
            count += 1;
        }
        count
    } else {
        20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// On-screen ctx, low pressure band (the style56 fixture).
    fn spawn_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The menu ctx: `NewDust` refuses BEFORE any draw (`Dust.cs:77`) —
    /// isolates the arm's EXPLICIT Next draws for the stream witnesses.
    fn menu_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: true,
            ..spawn_ctx()
        }
    }

    fn drone(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::MartianDrone, x, y, 1, 0.0)
    }

    fn corite(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::SolarCorite, x, y, 1, 0.0)
    }

    /// The census rows: the two parameter sets verbatim
    /// (`NPC.cs:35238-35275`), including the 418's un-rounded
    /// `(4/15)*60` homing term.
    #[test]
    fn s1_d_3_s74_the_two_parameter_rows() {
        let d = style74_params(false);
        assert_eq!((d.kb, d.speed, d.hover, d.side_shift), (0.4, 10.0, 200.0, 0.0));
        assert_eq!((d.far, d.near, d.damp), (750.0, 0.0, 30.0));
        assert_eq!((d.charge_ticks, d.decay, d.jitter), (30.0, 0.95, 50));
        assert_eq!((d.dash_speed, d.dash_ticks, d.turn_dist), (14.0, 30.0, 100.0));
        assert_eq!((d.lerp, d.home_term, d.speed_floor), (20.0, 0.0, 7.0));
        assert!(d.explodes_on_contact);
        let c = style74_params(true);
        assert_eq!((c.kb, c.speed, c.hover, c.side_shift), (0.3, 8.0, 175.0, 175.0));
        assert_eq!((c.far, c.near, c.damp), (600.0, 80.0, 60.0));
        assert_eq!((c.charge_ticks, c.decay, c.jitter), (20.0, 0.75, 0));
        assert_eq!((c.dash_speed, c.dash_ticks, c.turn_dist), (9.0, 30.0, 150.0));
        assert_eq!(c.lerp, 60.0);
        assert_eq!(c.home_term, (4.0f32 / 15.0f32) * 60.0f32);
        assert!(!c.explodes_on_contact);
    }

    /// `:35221-35237` — the rotation fold and the spriteDirection
    /// inversion: the drone flips AGAINST its motion, the corite stands
    /// with it (and writes 0 on a zero velocity).
    #[test]
    fn s1_d_3_s74_the_head_rotation_fold_and_sprite_inversion() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(7);
        let mut n = drone(100.0, 100.0);
        n.vx = -3.0;
        n.vy = 0.0;
        // State 4 runs for either kind (the C# arm has no type gate);
        // the drone never ENTERS it in vanilla, but the head writes are
        // what this witness pins.
        n.ai[0] = 4.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        // atan2(0, -3) = π → > π/2 → − π → 0 (the f32 round trip).
        assert!((n.rotation - (std::f32::consts::PI - std::f32::consts::PI)).abs() < 1e-6);
        assert_eq!(n.sprite_dir, 1, "-Sign(-3) = +1 for the drone");
        let mut k = corite(100.0, 100.0);
        k.vx = -3.0;
        let mut env2 = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut k, &mut env2);
        assert_eq!(k.sprite_dir, -1, "+Sign(-3) = -1 for the corite");
        k.vx = 0.0;
        k.vy = 0.0;
        let mut env3 = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut k, &mut env3);
        assert_eq!(k.sprite_dir, 0, "the corite writes Sign(0) = 0");
    }

    /// `:35308-35355` — the in-band, in-cone, in-sight approach latches
    /// state 1 and stores the scaled aim in ai[2]/ai[3].
    #[test]
    fn s1_d_3_s74_the_approach_latches_the_charge() {
        let w = air();
        // Drone centre (120, 115); player centre (120, 415): delta
        // (0, 300) — in the 0..750 band, straight below (angle π/2).
        let players = [cand(110.0, 394.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = drone(100.0, 100.0);
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        let out = style74_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 10.0, "normalize(0,300)*10 = (0,10)");
        assert!(n.net_update);
        assert_eq!(out.knock_back_resist, Some(0.4), "classic kb = 0.4×1");
    }

    /// `:35325`/`:35334-35341` — a player ABOVE fails the below-cone:
    /// the damp runs, the argue timer ticks, and the 120th tick raises
    /// netUpdate.
    #[test]
    fn s1_d_3_s74_the_cone_failure_ticks_the_argue_timer() {
        let w = air();
        // Player centre (120, 15) — 100 px ABOVE the drone centre
        // (120, 115): angle −π/2 fails the (π/8, 7π/8) cone.
        let players = [cand(110.0, -6.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = drone(100.0, 100.0);
        n.ai[3] = 119.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0, "no charge latch");
        assert_eq!(n.ai[3], 120.0);
        assert!(n.net_update, "the == 120 netUpdate");
        // The hover vector is 200 above the player → (0, -300)
        // normalized ×10 = (0, -10); the 30-tick damp from rest.
        assert_eq!(n.vx, 0.0);
        assert_eq!(n.vy, -10.0 / 30.0);
    }

    /// `:35372-35381` + `:35382` — the drone release draws the ±50
    /// jitter pair THEN the state-1 Next(4) sparkle gate (the release
    /// tick draws it too). Stream-replay witness.
    #[test]
    fn s1_d_3_s74_the_drone_release_rng_order() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(99);
        let peek = rng.clone();
        let mut n = drone(100.0, 100.0);
        n.ai[0] = 1.0;
        n.ai[1] = 29.0; // the release tick
        n.ai[2] = 0.0;
        n.ai[3] = 10.0;
        n.vx = 4.0;
        n.vy = 0.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.ai[1], 0.0);
        // Replay: Next(-50, 51) ×2 (the jitter), then Next(4) (the
        // sparkle gate — menu ctx refuses the spawn, no internal draws).
        let mut mirror = peek;
        let jx = mirror.next_range(-50, 51) as f32 * 0.04;
        let jy = mirror.next_range(-50, 51) as f32 * 0.04;
        let _gate = mirror.next_max(4);
        assert_eq!(rng, mirror, "the release + sparkle draw order");
        let (dx, dy) = normalize_xna(0.0 + jx, 10.0 + jy);
        assert_eq!(n.vx, dx * 14.0);
        assert_eq!(n.vy, dy * 14.0);
    }

    /// `:35378` — the corite's `Next(0, 1)` pair STILL advances the
    /// stream (the C# draws Sample for the empty range) while the
    /// jitter is always 0: the release velocity is the stored aim
    /// normalized ×9 exactly.
    #[test]
    fn s1_d_3_s74_the_corite_empty_range_release_still_draws() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(5);
        let peek = rng.clone();
        let mut n = corite(100.0, 100.0);
        n.ai[0] = 1.0;
        n.ai[1] = 19.0;
        n.ai[2] = 3.0;
        n.ai[3] = 4.0;
        n.vx = 5.0; // length > 2 → the decay path, no lerp
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        let mut mirror = peek;
        // The two corite ember gates (ai[0] was 1 → threshold 1) draw
        // FIRST, then the empty-range jitter pair.
        let _g1 = mirror.next_max(3);
        let _g2 = mirror.next_max(3);
        let z1 = mirror.next_range(0, 1);
        let z2 = mirror.next_range(0, 1);
        assert_eq!((z1, z2), (0, 0));
        assert_eq!(rng, mirror, "the empty ranges consumed two Samples");
        assert_eq!(n.vx, 3.0f32 / 5.0 * 9.0);
        assert_eq!(n.vy, 4.0f32 / 5.0 * 9.0);
    }

    /// `:35360-35367` — a stalled corite (length ≤ 2) lerps toward the
    /// player at 0.25 and skips the ×0.75 decay.
    #[test]
    fn s1_d_3_s74_the_stalled_corite_lerps_not_decays() {
        let w = air();
        // Corite centre (119, 119); player centre (119, 419): straight
        // below, SafeNormalize = (0, 1), aim ×0.1 = (0, 0.1).
        let players = [cand(109.0, 398.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = corite(100.0, 100.0);
        n.ai[0] = 1.0;
        n.vx = 1.0;
        n.vy = 0.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        assert_eq!(n.vx, 1.0 + (0.0 - 1.0) * 0.25);
        assert_eq!(n.vy, 0.0 + (0.1 - 0.0) * 0.25);
    }

    /// `:35414-35421` — the corite dash homing carries the folded
    /// `(4/15)*60` speed term; the drone's is 0.
    #[test]
    fn s1_d_3_s74_the_dash_homing_term() {
        let w = air();
        // Player far below-right so neither exit gate fires.
        let players = [cand(1000.0, 1000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = corite(100.0, 100.0);
        n.ai[0] = 2.0;
        n.vx = 9.0;
        n.vy = 0.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        let (cx, cy) = (119.0f32, 119.0f32);
        let (pcx, pcy) = (1010.0f32, 1021.0f32);
        let (nx, ny) = normalize_xna(pcx - cx, pcy - cy);
        let home = (4.0f32 / 15.0f32) * 60.0f32;
        let vlen = 9.0f32;
        assert_eq!(n.vx, (9.0 * (60.0 - 1.0) + nx * (vlen + home)) / 60.0);
        assert_eq!(n.vy, (0.0 * (60.0 - 1.0) + ny * (vlen + home)) / 60.0);
        assert_eq!(n.ai[0], 2.0, "no exit");
    }

    /// `:35398-35410` — the speed-floor exit: the drone halves and
    /// zeroes; the corite overrides into state 4 with ai[1] = 45.
    #[test]
    fn s1_d_3_s74_the_speed_floor_exits_differ() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = drone(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[1] = 5.0;
        n.vx = 6.0; // < 7 floor
        n.vy = 0.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        assert_eq!(n.ai, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(n.vx, 3.0);
        assert!(n.net_update);
        let mut k = corite(100.0, 100.0);
        k.ai[0] = 2.0;
        k.ai[1] = 5.0;
        k.vx = 6.0;
        let mut env2 = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut k, &mut env2);
        assert_eq!(k.ai[0], 4.0);
        assert_eq!(k.ai[1], 45.0);
        assert_eq!(k.vx, 3.0, "the halving runs BEFORE the 418 override");
    }

    /// `:35432-35442` — the corite cooldown: −3/tick, ×0.95, 15 ticks
    /// from 45 back to state 0.
    #[test]
    fn s1_d_3_s74_the_corite_cooldown() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = corite(100.0, 100.0);
        n.ai[0] = 4.0;
        n.ai[1] = 45.0;
        n.vx = 8.0;
        for i in 0..15 {
            let mut env = Style74Env {
                world: &w,
                players: &players,
                ticker: 0,
                dust: &mut pool,
                dust_ctx: &c,
                rng: &mut rng,
                difficulty: 1.0,
            };
            style74_step(&mut n, &mut env);
            if i < 14 {
                assert_eq!(n.ai[0], 4.0);
            }
        }
        assert_eq!(n.ai[0], 0.0);
        assert_eq!(n.ai[1], 0.0);
        assert!(n.net_update);
        // The sequential f32 multiplies, not powi (the rounding order).
        let mut expected = 8.0f32;
        for _ in 0..15 {
            expected *= 0.95;
        }
        assert_eq!(n.vx, expected);
    }

    /// `:35443-35450` — the 64-px contact gate is 388-only: the drone
    /// explodes on contact (and the tail runs the SAME tick); the
    /// corite keeps approaching.
    #[test]
    fn s1_d_3_s74_the_contact_gate_is_drone_only() {
        let w = air();
        // Player centre (150, 136) — 36 px from the drone centre
        // (120, 115), inside 64.
        let players = [cand(140.0, 115.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = drone(100.0, 100.0);
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 3.0);
        assert_eq!(n.ai[1], 1.0, "the entry tick's tail already counted");
        assert_eq!(n.size_override, Some((192.0, 192.0)));
        assert_eq!(n.alpha, 255);
        assert_eq!(n.damage_override, Some(80));
        assert_eq!((n.vx, n.vy), (0.0, 0.0));
        // The box centres on the OLD centre (120, 115): 120−96, 115−96.
        assert_eq!((n.x, n.y), (24.0, 19.0));
        let mut k = corite(100.0, 100.0);
        let mut env2 = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut k, &mut env2);
        assert_ne!(k.ai[0], 3.0, "flag75 = false for the corite");
    }

    /// `:35463-35491` — the implosion's EXPLICIT draw order, isolated
    /// with the menu ctx (NewDust refuses before drawing): 10 × 2
    /// NextDouble, then 40 × (2 NextDouble + Next(2) [+2 on 0] +
    /// Next(4) [+2 on 0]). Stream-replay witness.
    #[test]
    fn s1_d_3_s74_the_implosion_explicit_draw_order() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1234);
        let peek = rng.clone();
        let mut n = drone(100.0, 100.0);
        n.ai[0] = 3.0;
        n.ai[1] = 0.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        let out = style74_step(&mut n, &mut env);
        assert!(!out.died, "the fuse is 3 ticks");
        assert_eq!(n.ai[1], 1.0);
        let mut mirror = peek;
        for _ in 0..10 {
            let _a = mirror.next_double();
            let _r = mirror.next_double();
        }
        for _ in 0..40 {
            let _a = mirror.next_double();
            let _r = mirror.next_double();
            if mirror.next_max(2) == 0 {
                let _a2 = mirror.next_double();
                let _r2 = mirror.next_double();
            }
            if mirror.next_max(4) == 0 {
                let _a3 = mirror.next_double();
                let _r3 = mirror.next_double();
            }
        }
        assert_eq!(rng, mirror, "the implosion draw order, verbatim");
    }

    /// `:35492-35499` — the 3-tick fuse: death on the third state-3
    /// tick, hp 0, the Item14 request, the light seam every tick.
    #[test]
    fn s1_d_3_s74_the_three_tick_fuse() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(1);
        let mut n = drone(100.0, 100.0);
        n.ai[0] = 3.0;
        for tick in 1..=3 {
            let mut env = Style74Env {
                world: &w,
                players: &players,
                ticker: 0,
                dust: &mut pool,
                dust_ctx: &c,
                rng: &mut rng,
                difficulty: 1.0,
            };
            let out = style74_step(&mut n, &mut env);
            // Centre (120, 115) → tile (7, 7) — (int)120/16, (int)115/16.
            assert_eq!(out.light, Some((7, 7, [0.2, 0.7, 1.1])));
            if tick < 3 {
                assert!(!out.died);
                assert!(!out.sound_item14);
            } else {
                assert!(out.died);
                assert!(out.sound_item14);
                assert_eq!(n.hp, 0);
            }
        }
    }

    /// `:35291-35307` — the corite ember gates draw Next(3) TWICE every
    /// tick (menu ctx isolates them); state 2 raises the threshold to 2
    /// and the box half to 30.
    #[test]
    fn s1_d_3_s74_the_corite_ember_gates_draw_every_tick() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(42);
        let peek = rng.clone();
        let mut n = corite(100.0, 100.0);
        n.ai[0] = 4.0; // the cooldown — the dust still runs
        n.ai[1] = 30.0;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        let mut mirror = peek;
        let _g1 = mirror.next_max(3);
        let _g2 = mirror.next_max(3);
        assert_eq!(rng, mirror, "exactly two Next(3) draws");
    }

    /// `:35276-35290` — the drone trail dust: one NewDust 226 with
    /// velocity = npc velocity / 5 and the rotated (-10, 10) offset
    /// (X flipped when spriteDirection == 1). Live-pool witness.
    #[test]
    fn s1_d_3_s74_the_drone_trail_dust() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut pool = DustPool::new();
        let c = spawn_ctx();
        let mut rng = UnifiedRandom::new(3);
        let mut n = drone(100.0, 100.0);
        n.ai[0] = 4.0; // no state arm — isolate the trail
        n.vx = 5.0;
        n.vy = -2.5;
        let mut env = Style74Env {
            world: &w,
            players: &players,
            ticker: 0,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            difficulty: 1.0,
        };
        style74_step(&mut n, &mut env);
        // vx > 0 → sprite_dir = -1 → the spin point stays (-10, 10).
        assert_eq!(n.sprite_dir, -1);
        let d = pool.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 226);
        assert!(d.no_gravity);
        assert_eq!(d.velocity, Vec2::new(5.0 / 5.0, -2.5 / 5.0));
        let (rx, ry) = rotated_by(-10.0, 10.0, n.rotation);
        assert_eq!(d.position, Vec2::new(120.0 + rx, 115.0 + ry));
    }

    /// `FindFrame` case 388 (`:71731-71742`) — the 2-frame 6-tick loop;
    /// case 418 (`:70758-70800`) — the state rows, the STRICT > 6 / > 12
    /// counters, and the state-3/4 hold.
    #[test]
    fn s1_d_3_s74_the_find_frame_twins() {
        // Drone: 6 ticks per row, wrap at 2.
        let (mut f, mut c) = (0u32, 0.0f32);
        for _ in 0..5 {
            (f, c) = drone_find_frame(f, c);
        }
        assert_eq!((f, c), (0, 5.0));
        (f, c) = drone_find_frame(f, c);
        assert_eq!((f, c), (1, 0.0));
        for _ in 0..6 {
            (f, c) = drone_find_frame(f, c);
        }
        assert_eq!((f, c), (0, 0.0));
        // Corite state 0: the counter is STRICT > 6 (7 ticks per row),
        // wrap at row > 4.
        let (mut f, mut c) = (4u32, 6.0f32);
        (f, c) = corite_find_frame(f, c, 0.0, 0.0);
        assert_eq!((f, c), (0, 0.0), "row 5 > 4 wraps to 0");
        // State 2: the ai[1] bands pin rows 5/6/7.
        assert_eq!(corite_find_frame(0, 9.0, 2.0, 9.0).0, 5);
        assert_eq!(corite_find_frame(0, 9.0, 2.0, 10.0).0, 6);
        assert_eq!(corite_find_frame(0, 9.0, 2.0, 20.0).0, 7);
        // State 3: no arm — the frame and counter hold.
        assert_eq!(corite_find_frame(6, 3.0, 3.0, 1.0), (6, 3.0));
        // State -2: > 12 counter, wrap at row > 7 → 5.
        assert_eq!(corite_find_frame(7, 12.0, -2.0, 0.0), (5, 0.0));
    }

    /// The HitEffect bounds: the 388's INT product `lifeMax * 50`
    /// (`:84474`) vs the 418's `dmg / lifeMax * 20` (`:82963`) — the
    /// same dmg/lifeMax gives 1 vs 10 dusts.
    #[test]
    fn s1_d_3_s74_the_hit_dust_bounds_differ() {
        assert_eq!(drone_hit_dust_plan(300, 300.0, 300), 1);
        assert_eq!(corite_hit_dust_plan(600, 300.0, 600), 10);
        assert_eq!(drone_hit_dust_plan(0, 0.0, 300), 10);
        assert_eq!(corite_hit_dust_plan(0, 0.0, 600), 20);
    }

    /// `GameDifficultyData.cs:84`/`:86` — the difficulty curves: classic
    /// 1 → ×1, master 3 → kb ×0.8 / damage ×3, journey 0.5 → damage ×0.5.
    #[test]
    fn s1_d_3_s74_the_difficulty_curves() {
        assert_eq!(knockback_scaled(0.4, 1.0), 0.4);
        assert_eq!(knockback_scaled(0.4, 3.0), 0.4 * 0.8);
        assert_eq!(attack_damage_scaled(80.0, 1.0), 80);
        assert_eq!(attack_damage_scaled(80.0, 3.0), 240);
        assert_eq!(attack_damage_scaled(80.0, 0.5), 40);
        // Expert 2: the damage bracket (0.5→0.5, 3→3) has slope 1, so
        // the sample is 2.0; the kb bracket (1→3) samples 0.9.
        assert_eq!(attack_damage_scaled(80.0, 2.0), 160);
        assert_eq!(knockback_scaled(0.4, 2.0), 0.4 * 0.9);
    }
}
