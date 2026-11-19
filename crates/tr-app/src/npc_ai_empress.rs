//! Empress of Light's machine — `NPC.AI_120_HallowBoss`
//! (`NPC.cs:45953-46942`), aiStyle 120, `NPCID.HallowBoss` (636,
//! `ID/NPCID.cs:11949`). S1-D-3 empress audit/gap-fill, **code-only
//! mode**: the census found the boss entirely missing (only the loot
//! row `loot_db_gen.rs:1483`, the item rows, the name tables and the
//! `IsDamageDodgeable` note in `npc.rs:3006` exist — no `NpcKind`, no
//! aiStyle-120 arm). The whole machine is ported here as pure functions
//! over a local state struct + witness tests; the `NpcKind` roster row
//! and the live dispatch wiring are the documented remainder (the
//! Fishron precedent, `ad41a646`).
//!
//! The machine (`ai[0]` state, `ai[1]` timer, `ai[2]` pattern counter,
//! `ai[3]` phase/enrage latch: 0 normal, 1 phase 2, 2 day-enraged,
//! 3 enraged phase 2 — `IsInPhase2` = 1|3 `:45814-45821`,
//! `IsGenuinelyEnraged` = 2|3 `:45823-45830`):
//! * **materialize** (0, `:46024-46075`): 180 ticks, the 874 Death
//!   Aurora at centre+(0,-80), the opacity ramp, the 2-dust trail;
//! * **pattern pick** (1, `:46076-46264`): the dash (`DashTo`,
//!   `:46944-46960`) then the classic 10-slot cycle
//!   `[2,8,6,8,5,2,8,4,8,5]` (hp ≤ 50 % forces 10), the phase-2
//!   9-slot `[7,2,8,5,2,6,4,8,12]` / expert 10-slot
//!   `[7,2,8,11,5,2,6,4,8,12]`; 8 → 9 when the target is to the right;
//!   the despawn gates (invalid target / > 6400 px / enraged-at-night
//!   or dusk ≥ 53400) force 13;
//! * **prismatic bolts** (2, `:46265-46320`): 873 Rainbow Streaks every
//!   3 ticks (2 expert), the ±π/4 random fan; phase-2 expert: the
//!   full-circle 10-speed variant (BOTH draws fire, `:46291-46294`);
//! * **death aurora** (3, `:46321-46342`): the `% 180` volley is
//!   unreachable (the state ends at 120) — ported as-is;
//! * **ethereal lances** (4, `:46343-46451`): 919 Fairy Queen Lances on
//!   the 300/450-px ring around the player, every 4 ticks < 100;
//! * **lasting rainbows** (5, `:46452-46492`): the 13-bolt 872 ring
//!   with the random start angle;
//! * **sun dance** (6, `:46493-46531`): 923 Sun Dances, 6 (8 expert)
//!   per volley at ticks 0/60/120;
//! * **lance walls** (7, `:46532-46631`): the six geometric 919 walls;
//!   14 lances classic (the f32 `<= 1f` loop hits exactly 1.0 on the
//!   13th add of 1/13) / 18 expert; the discarded `Main.rand.NextFloat()`
//!   (`:46553`) STILL advances the stream;
//! * **dash** (8/9, `:46632-46675`): the side dash, invulnerable only
//!   6..40, damage ×1.5 in 41..90;
//! * **transform** (10, `:46676-46707`): at tick 90 `ai[3]` 0→1 / 2→3
//!   and the centre snaps over the target;
//! * **spiral bolts** (11, `:46708-46793`): the expert-phase-2 lance
//!   variant behind the player's velocity, every 3 ticks;
//! * **rotating streaks** (12, `:46794-46843`): 873 every 6 (4 expert)
//!   ticks in 10..60, the angle rides `(ai[1]-10)/50`;
//! * **despawn** (13, `:46844-46917`): the alpha ramp ±5, the 5-dust
//!   trail; at alpha 255 `active = false` and the tail is SKIPPED.
//!
//! The day-enrage gate (`ShouldEmpressBeEnraged`, `:45925-45951`):
//! non-remix = `Main.dayTime`; remix = the sticky `NPC.empressRageMode`
//! latch set when the boss is above `worldSurface * 16`. At full life
//! the latch adds 2 to `ai[3]` (`:45974-45977`, again at the end of the
//! materialize `:46066-46069`); enraged: every damage row 9999
//! (`:46002-46010`, contact `:46921-46924`), `flag2` forced true. The
//! one-shot kill itself is the 9999 damage — the tint
//! (`Main.OurFavoriteColor`, `Main.cs:26615-26620`, `:28282-28293`)
//! and the `IsDamageDodgeable` false arm (`NPC.cs:78752-78753`) are
//! render/dodge-side, noted for the wiring.
//!
//! **RNG by construction** — every `Main.rand` draw is consumed in C#
//! order (each draw = one `InternalSample`, `UnifiedRandom.cs:62-88`):
//! * `DoMagicEffect` (`:45832-45923`): per dust, the `NewDustPerfect`
//!   ARGUMENTS evaluate first — `NextVector2CircularEdge` (1,
//!   `Utils.cs:1271-1274` → `NextVector2Unit` `:1261-1264`), the scale
//!   `NextFloat` (1), the colour `NextFloat` (1), the scale `NextFloat`
//!   (1) — then `Dust.NewDust`'s own 6 (`Dust.cs:170-175`, `:190`:
//!   `Next(w-4)`, `Next(h-4)` with w/h clamped ≥ 5, `Next(-20,21)` ×2,
//!   `Next(3)`, `Next(-20,21)`). Effect 5 with 0 < progress draws
//!   `Next(86,92)` ONCE (`:45869`) even when progress ≥ 1 zeroes the
//!   count; progress == 0 draws NOTHING (count 0, `:45862-45865` — the
//!   `customData` dust branch `:45902-45909` is dead code).
//! * case 0 trail (`:46040-46058`): per dust 6 (`NewDust`, w/h = 100)
//!   + 2 (`NextVector2Circular`, `Utils.cs:1266-1269`) + 1 (velocity)
//!   + 1 (fadeIn) + 6 (`CloneDust` → `NewDust`, `Dust.cs:263-269`) = 16;
//! * case 13 trail (`:46880-46898`): 1 (colour) + 6 + 2 + 1 (velocity)
//!   + 1 (scale) + 1 (fadeIn) + 6 = 18 per dust ×5;
//! * gameplay: case 2 `NextFloatDirection` (`:46291`, +`NextFloat`
//!   `:46294` when phase-2 expert), case 5 `NextFloat` (`:46473`),
//!   case 7 the DISCARDED `NextFloat` (`:46553`).
//! Dust-pool assumption (documented deviation): the pool is below the
//! 50 % `maxDustToDraw` threshold (`Dust.cs:139-143` — the FIRST skip
//! tier) and on-screen, so `Dust.NewDust`'s skip draws and early-outs
//! (`Dust.cs:77-101`) never fire; `ChildSafety.Disabled` is true by
//! default (`GameContent/ChildSafety.cs:33`) so its branch
//! (`Dust.cs:194-213`) never draws. Dust type 267 is in NONE of the
//! extra-draw type lists (`Dust.cs:214-253`), so exactly 6 draws per
//! `NewDust`; `CloneDust` (`Dust.cs:263-281`) forwards w/h = 0 → 6 more.
//!
//! Documented deviations (also `docs/ANALYSIS.md`, S1-D-3 empress):
//! * the dust VISUALS, sounds (`SoundID.Item160-165`), the
//!   `Lighting.AddLight` tail (`:46941`) and `GetSpawnSource_ForProjectile`
//!   are skipped — the dust RNG is still consumed (above);
//! * `TargetClosest` is a request (the S1-D-3p `npc_target.rs` helper
//!   when wired); the caller's target rides unchanged within the tick;
//! * single-player assumed (`Main.netMode != 1`): every `NewProjectile`
//!   fires and the `Boss_CanShootExtraAt` scans (`:46962-46982`) run
//!   over the caller's ascending-slot player slice;
//! * the remix scan (`:45933-45941`) takes THIS boss as the first live
//!   636 (the single-Empress invariant — `Main.cs:17922-17924` clears
//!   the latch when no 636 is alive);
//! * `Vector2.Normalize` of an exactly-zero vector yields NaN in XNA;
//!   the port guards to zero (the Fishron precedent) — only differs on
//!   exact overlap;
//! * the 874/873/872/919/923 projectiles have no port AI yet — the
//!   spawns are REQUESTS carrying the C# `ai[0]`/`ai[1]` arguments.

#![allow(dead_code)] // wiring deferred (S1-D-3 empress, code-only mode)

use std::f32::consts::PI;
use tr_core::UnifiedRandom;

/// `NPC.SetDefaults` type 636 (`NPC.cs:17188-17206`): 100x100, aiStyle
/// 120, damage 80, defense 50, lifeMax 70000, knockBackResist 0,
/// noGravity + noTileCollide, boss, Opacity 0 (alpha 255),
/// dontTakeDamage true, npcSlots 16, value 25 gold.
pub const EMPRESS_WIDTH: f32 = 100.0;
pub const EMPRESS_HEIGHT: f32 = 100.0;
pub const EMPRESS_DEF_DAMAGE: i32 = 80;
pub const EMPRESS_DEF_DEFENSE: i32 = 50;
pub const EMPRESS_LIFE_MAX: i32 = 70000;

/// The machine's live state — the fields `AI_120` reads/writes.
#[derive(Clone, Copy, Debug)]
pub struct EmpressState {
    /// `this.ai[0]` — the state (0..13).
    pub ai0: f32,
    /// `this.ai[1]` — the state timer.
    pub ai1: f32,
    /// `this.ai[2]` — the pattern counter (reset by the transform).
    pub ai2: f32,
    /// `this.ai[3]` — the phase/enrage latch (0/1/2/3).
    pub ai3: f32,
    /// `this.localAI[0]` — the 44-tick wing-frame clock (`:46933-46936`).
    pub local_ai0: f32,
    /// `base.position` (top-left); `base.Center` derives from it.
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `this.alpha` (0 opaque .. 255 invisible); `Opacity` derives.
    pub alpha: i32,
    /// `this.damage` — rewritten every tick from `defDamage`.
    pub damage: i32,
    /// `this.defense` — rewritten every tick from `defDefense`.
    pub defense: i32,
    pub life: i32,
    pub life_max: i32,
    /// `this.defDamage`/`this.defDefense` (the `SetDefaults` latches).
    pub def_damage: i32,
    pub def_defense: i32,
    /// `this.target` (the spawn owner for the extra-shot skip).
    pub target: i32,
    /// `base.whoAmI` (the sun dance's `ai[1]`, `:46519`).
    pub who_am_i: i32,
}

impl Default for EmpressState {
    fn default() -> Self {
        Self {
            ai0: 0.0,
            ai1: 0.0,
            ai2: 0.0,
            ai3: 0.0,
            local_ai0: 0.0,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            alpha: 255, // Opacity = 0f in SetDefaults (`:17203`)
            damage: EMPRESS_DEF_DAMAGE,
            defense: EMPRESS_DEF_DEFENSE,
            life: EMPRESS_LIFE_MAX,
            life_max: EMPRESS_LIFE_MAX,
            def_damage: EMPRESS_DEF_DAMAGE,
            def_defense: EMPRESS_DEF_DEFENSE,
            target: 0,
            who_am_i: 0,
        }
    }
}

impl EmpressState {
    /// `base.Center` (`Entity.cs:122-132` — position + size/2).
    pub fn center(&self) -> (f32, f32) {
        (self.x + EMPRESS_WIDTH / 2.0, self.y + EMPRESS_HEIGHT / 2.0)
    }
    /// `this.Opacity` getter (`NPC.cs:6820-6825`).
    pub fn opacity(&self) -> f32 {
        1.0 - self.alpha as f32 / 255.0
    }
    /// `this.Opacity` setter (`NPC.cs:6826-6829`) — the truncating cast.
    pub fn set_opacity(&mut self, value: f32) {
        self.alpha = ((1.0 - value) * 255.0).clamp(0.0, 255.0) as i32;
    }
    /// `AI_120_HallowBoss_IsInPhase2` (`NPC.cs:45814-45821`).
    pub fn is_in_phase2(&self) -> bool {
        self.ai3 == 1.0 || self.ai3 == 3.0
    }
    /// `AI_120_HallowBoss_IsGenuinelyEnraged` (`NPC.cs:45823-45830`).
    pub fn is_genuinely_enraged(&self) -> bool {
        self.ai3 == 2.0 || self.ai3 == 3.0
    }
}

/// One player slot for the `Boss_CanShootExtraAt` scans
/// (`NPC.cs:46962-46982`). The caller passes the slice in ASCENDING
/// slot order (the C# scans `for (i = 0; i < 255; i++)`).
#[derive(Clone, Copy, Debug)]
pub struct EmpressExtraPlayer {
    /// The `Main.player` slot index.
    pub index: i32,
    pub cx: f32,
    pub cy: f32,
    pub vx: f32,
    pub vy: f32,
    pub active: bool,
    pub dead: bool,
    /// `this.playerInteraction[index]`.
    pub interaction: bool,
}

/// The per-tick environment the arm reads from `Main`/the target.
#[derive(Clone, Copy, Debug)]
pub struct EmpressEnv {
    /// `this.difficulty` (latched at `SetDefaults`, `NPC.cs:18150`;
    /// `GameDifficultyLevel` Classic 1 / Expert 2 / Master 3,
    /// `DataStructures/GameDifficultyLevel.cs:16-20`).
    pub difficulty: f32,
    /// `Main.dayTime`.
    pub day_time: bool,
    /// `Main.time` (the dusk gate reads ≥ 53400, `:46242`).
    pub time: f64,
    /// `Main.remixWorld` (`:45927`).
    pub remix_world: bool,
    /// `NPC.empressRageMode` — the sticky remix latch (`:45929`);
    /// the updated value rides `EmpressOut::empress_rage_mode`.
    pub empress_rage_mode: bool,
    /// `Main.worldSurface` in TILES (the gate multiplies by 16,
    /// `:45942` — a DOUBLE compare in the C#).
    pub world_surface: f64,
    /// `Main.getGoodWorld` (`:46079`).
    pub get_good_world: bool,
    /// The target player's centre/velocity (`GetTargetData`,
    /// `NPC.cs:6901-6912`).
    pub target_cx: f32,
    pub target_cy: f32,
    pub target_vx: f32,
    pub target_vy: f32,
    /// `!HasValidTarget` (`NPC.cs:6657-6671`: target out of 0..254,
    /// player inactive/dead/ghost) — `NPCAimedTarget.Invalid`.
    pub target_invalid: bool,
}

impl Default for EmpressEnv {
    fn default() -> Self {
        Self {
            difficulty: 1.0,
            day_time: false,
            time: 0.0,
            remix_world: false,
            empress_rage_mode: false,
            world_surface: 0.0,
            get_good_world: false,
            target_cx: 0.0,
            target_cy: 0.0,
            target_vx: 0.0,
            target_vy: 0.0,
            target_invalid: false,
        }
    }
}

/// A spawn/side-effect the arm requests; the wiring resolves them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmpressRequest {
    /// `Projectile.NewProjectile` — `kind` is the vanilla id (874/873/
    /// 872/919/923), `ai0`/`ai1` the two spawn arguments verbatim.
    Projectile {
        kind: i32,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        damage: i32,
        ai0: f32,
        ai1: f32,
    },
    /// `this.TargetClosest()` — the wiring runs the S1-D-3p helper.
    TargetClosest,
}

/// The per-tick output beside the state mutations.
#[derive(Clone, Debug)]
pub struct EmpressOut {
    pub requests: Vec<EmpressRequest>,
    /// `this.netUpdate = true` was written this tick.
    pub net_update: bool,
    /// `this.dontTakeDamage = !flag6` (`:46919`).
    pub dont_take_damage: bool,
    /// `this.active` — case 13's alpha-255 despawn clears it
    /// (`:46904-46911`) and SKIPS the tail.
    pub active: bool,
    /// The updated `NPC.empressRageMode` static (`:45944-45947`).
    pub empress_rage_mode: bool,
}

/// XNA `Vector2` ops on `(x, y)` pairs, f32 throughout (the C# struct
/// is float; `Math.Cos/Sin/Atan2` run in double and cast back).
type V2 = (f32, f32);

fn v_add(a: V2, b: V2) -> V2 {
    (a.0 + b.0, a.1 + b.1)
}
fn v_sub(a: V2, b: V2) -> V2 {
    (a.0 - b.0, a.1 - b.1)
}
fn v_scale(a: V2, s: f32) -> V2 {
    (a.0 * s, a.1 * s)
}
fn v_len(a: V2) -> f32 {
    (a.0 * a.0 + a.1 * a.1).sqrt()
}
fn v_dot(a: V2, b: V2) -> f32 {
    a.0 * b.0 + a.1 * b.1
}
/// `Vector2.Distance` — sqrt of the squared component deltas.
fn v_dist(a: V2, b: V2) -> f32 {
    v_len(v_sub(a, b))
}
/// `Vector2.Normalize` — XNA divides UNGUARDED (zero → NaN); the port
/// guards the exact-overlap case to zero (the documented deviation).
fn normalize(a: V2) -> V2 {
    let len = v_len(a);
    if len > 0.0 && len.is_finite() {
        (a.0 / len, a.1 / len)
    } else {
        (0.0, 0.0)
    }
}
/// `Utils.SafeNormalize` (`Utils.cs:1837-1844`) — zero/NaN → default.
fn safe_normalize(a: V2, default: V2) -> V2 {
    if (a.0 == 0.0 && a.1 == 0.0) || a.0.is_nan() || a.1.is_nan() {
        default
    } else {
        normalize(a)
    }
}
/// `Utils.RotatedBy` (`Utils.cs:1675-1684`) — cos/sin in DOUBLE of the
/// double angle, cast to f32, then the f32 component math.
fn rotated_by(a: V2, radians: f64) -> V2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    (a.0 * c - a.1 * s, a.0 * s + a.1 * c)
}
/// `Utils.ToRotation` (`Utils.cs:1655-1658`) — `Atan2` in double.
fn to_rotation(a: V2) -> f32 {
    (a.1 as f64).atan2(a.0 as f64) as f32
}
/// `Vector2.Lerp` — `value1 + (value2 - value1) * amount`, f32 order.
fn v_lerp(a: V2, b: V2, t: f32) -> V2 {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}
/// `MathHelper.Lerp` (scalar).
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
/// `Utils.GetLerpValue(..., clamped)` (`Utils.cs:315-343`) — the
/// inverted-range clamp branch included.
fn get_lerp_value(from: f32, to: f32, t: f32, clamped: bool) -> f32 {
    if clamped {
        if from < to {
            if t < from {
                return 0.0;
            }
            if t > to {
                return 1.0;
            }
        } else {
            if t < to {
                return 1.0;
            }
            if t > from {
                return 0.0;
            }
        }
    }
    (t - from) / (to - from)
}
/// `Utils.Remap` (`Utils.cs:345-348`) — clamped by default.
fn remap(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    lerp(to_min, to_max, get_lerp_value(from_min, from_max, from_value, true))
}
/// `Utils.ToDirectionInt` (`Utils.cs:2010-2017`) — bool → ±1 (NOT 0/1).
fn to_direction_int(b: bool) -> i32 {
    if b {
        1
    } else {
        -1
    }
}

/// `NPC.SimpleFlyMovement` (`NPC.cs:77397-77431`) — the per-axis accel
/// with the sign-reversal DOUBLE add (only when the first add leaves
/// the velocity still on the old side of zero).
fn simple_fly_movement(vx: &mut f32, vy: &mut f32, desired_x: f32, desired_y: f32, move_speed: f32) {
    if *vx < desired_x {
        *vx += move_speed;
        if *vx < 0.0 && desired_x > 0.0 {
            *vx += move_speed;
        }
    } else if *vx > desired_x {
        *vx -= move_speed;
        if *vx > 0.0 && desired_x < 0.0 {
            *vx -= move_speed;
        }
    }
    if *vy < desired_y {
        *vy += move_speed;
        if *vy < 0.0 && desired_y > 0.0 {
            *vy += move_speed;
        }
    } else if *vy > desired_y {
        *vy -= move_speed;
        if *vy > 0.0 && desired_y < 0.0 {
            *vy -= move_speed;
        }
    }
}

/// `GetAttackDamage_ForProjectiles` (`NPC.cs:7052-7055`) — the
/// classic→expert Remap of `this.difficulty`, truncated to int.
fn attack_damage_for_projectiles(difficulty: f32, normal: f32, expert: f32) -> i32 {
    remap(difficulty, 1.0, 2.0, normal, expert) as i32
}

/// `NPC.ShouldEmpressBeEnraged` (`NPC.cs:45925-45951`). The remix scan
/// (`:45933-45941`) takes THIS boss as the first live 636 (the
/// single-Empress invariant; documented). Returns the updated sticky
/// latch beside the answer; the non-remix arm leaves it untouched.
fn should_empress_be_enraged(s: &EmpressState, env: &EmpressEnv) -> (bool, bool) {
    if env.remix_world {
        if env.empress_rage_mode {
            return (true, true);
        }
        // `:45942` — the DOUBLE compare `Center.Y < worldSurface * 16.0`.
        if (s.center().1 as f64) < env.world_surface * 16.0 {
            return (true, true);
        }
        return (false, false);
    }
    (env.day_time, env.empress_rage_mode)
}

/// `Dust.NewDust`'s RNG consumption (`Dust.cs:170-175`, `:190`) under
/// the documented pool assumption (no skip draws, no early-out):
/// `Next(max(w,5)-4)`, `Next(max(h,5)-4)`, `Next(-20,21)` ×2,
/// `Next(3)`, `Next(-20,21)` — six draws, in order. The visuals are
/// skipped; only the stream advances.
fn new_dust_draws(rng: &mut UnifiedRandom, width: i32, height: i32) {
    let w = width.max(5); // `Dust.cs:155-162`
    let h = height.max(5);
    rng.next_max(w - 4); // :170
    rng.next_max(h - 4); // :171
    rng.next_range(-20, 21); // :172
    rng.next_range(-20, 21); // :173
    rng.next_max(3); // :175
    rng.next_range(-20, 21); // :190
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — one `NextFloat`;
/// the angle is FLOAT arithmetic (`startRotation + rotationRange *
/// NextFloat()`, default range `(float)Math.PI * 2f`), then
/// `ToRotationVector2` (`Utils.cs:1665-1668`) cos/sin in double of the
/// widened float, cast back to f32.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> V2 {
    let angle = PI * 2.0 * rng.next_float();
    ((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// THEN a second `NextFloat` radius scale.
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> V2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    (unit.0 * half_w * r, unit.1 * half_h * r)
}

/// `NPC.AI_120_HallowBoss_DoMagicEffect` (`NPC.cs:45832-45923`) — the
/// dust RNG consumption ONLY (the visuals are the documented skip).
/// Per-dust argument order: CircularEdge (1) → scale NextFloat (1) →
/// colour NextFloat (1) → scale NextFloat (1) → NewDust's own 6
/// (`NewDustPerfect` forwards w/h = 0 → clamped 5, `Dust.cs:54-63`).
fn do_magic_effect_draws(rng: &mut UnifiedRandom, effect_type: i32, progress: f32) {
    let mut count = 2; // num4, `:45838`
    match effect_type {
        2 | 4 => count = 4, // `:45853`
        5 => {
            // `:45861-45875` — progress == 0: count 0, NO draws at all
            // (the customData branch `:45902-45909` is dead code).
            if progress == 0.0 {
                count = 0;
            } else {
                count = 5;
                rng.next_range(86, 92); // :45869 — drawn even if >= 1 zeroes the count
            }
            if progress >= 1.0 {
                count = 0; // :45871-45874
            }
        }
        _ => {}
    }
    for _ in 0..count {
        // `:45879` — the NewDustPerfect ARGUMENTS evaluate first:
        next_vector2_unit(rng); // NextVector2CircularEdge, Utils.cs:1271-1274
        rng.next_float(); // the velocity's NextFloat() * (1-num3) + num3
        rng.next_float(); // hslToRgb(Main.rand.NextFloat(), ...)
        rng.next_float(); // (NextFloat() * 2 + 2) * num2
        new_dust_draws(rng, 0, 0); // Dust.cs:54-63 → NewDust(Position, 0, 0, ...)
    }
}

/// `NPC.AI_120_HallowBoss_DashTo` (`NPC.cs:46944-46960`). The first
/// `base.DirectionTo` result is discarded (`:46946`); the aim point is
/// target + (0,-300), pulled 100 px toward the boss when > 200 away;
/// the velocity lerps between the capped-18 aim and `vector / 6f` by
/// the 100..600 length ramp.
fn dash_to(s: &mut EmpressState, target_position: V2) {
    let mut tp = v_add(target_position, (0.0, -300.0));
    let c = s.center();
    if v_dist(c, tp) > 200.0 {
        // `base.DirectionTo` = Normalize(tp - Center), Entity.cs:205-208.
        tp = v_sub(tp, v_scale(normalize(v_sub(tp, c)), 100.0));
    }
    let vector = v_sub(tp, c);
    let len = v_len(vector);
    let lerp_value = get_lerp_value(100.0, 600.0, len, true);
    let num = if len > 18.0 { 18.0 } else { len };
    let vel = v_lerp(
        v_scale(safe_normalize(vector, (0.0, 0.0)), num),
        (vector.0 / 6.0, vector.1 / 6.0),
        lerp_value,
    );
    s.vx = vel.0;
    s.vy = vel.1;
}

/// `NPC.Boss_CanShootExtraAt` (`NPC.cs:46962-46982`), with
/// `alwaysSkipMainTarget = true` (every call site uses the default).
fn boss_can_shoot_extra_at(
    s: &EmpressState,
    p: &EmpressExtraPlayer,
    rotation_index_to_attack: i32,
    rotation_size: i32,
    attack_scan_distance: f32,
) -> bool {
    if p.index == s.target {
        return false; // :46964-46967
    }
    if p.index % rotation_size != rotation_index_to_attack {
        return false; // :46968-46971
    }
    if !p.active || p.dead || !p.interaction {
        return false; // :46972-46976
    }
    if v_dist(s.center(), (p.cx, p.cy)) > attack_scan_distance {
        return false; // :46977-46980
    }
    true
}

/// One fixed step of `AI_120_HallowBoss` (`NPC.cs:45953-46942`).
/// `players` is the ascending-slot slice for the extra-shot scans.
/// Returns false only via `out.active = false` (the case-13 despawn
/// early-return, `:46904-46911`, which SKIPS the tail writes).
pub fn empress_ai_step(
    s: &mut EmpressState,
    env: &EmpressEnv,
    rng: &mut UnifiedRandom,
    players: &[EmpressExtraPlayer],
) -> EmpressOut {
    let mut out = EmpressOut {
        requests: Vec::new(),
        net_update: false,
        dont_take_damage: false,
        active: true,
        empress_rage_mode: env.empress_rage_mode,
    };
    // The five hover anchors (`:45955-45959`).
    let anchor_left = (-150.0f32, -250.0f32); // vector
    let anchor_right = (150.0f32, -250.0f32); // vector2
    let anchor_up = (0.0f32, -350.0f32); // vector3 AND vector4
    let anchor_high = (-80.0f32, -500.0f32); // vector5
    let num = 0.5f32; // moveSpeed
    let num2 = 12.0f32; // desired speed
    let num3 = 40.0f32; // the hover dead-band
    let num4 = 6400.0f32; // the despawn distance
    let num5 = 40; // the aurora damage (`:45964`)
    let mut num6 = 50; // lance damage
    let mut num7 = 70; // wall damage
    let mut num8 = 45; // streak damage
    let mut num9 = 45; // rainbow damage
    let mut num10 = 50; // sun-dance damage
    let flag = s.is_in_phase2();
    let mut flag2 = env.difficulty >= 2.0; // Main.expertMode, Main.cs:2700
    let flag3 = flag && flag2;
    let (flag4, rage_mode) = should_empress_be_enraged(s, env);
    out.empress_rage_mode = rage_mode;
    // `:45974-45977` — the full-life enrage latch (bitwise & in C#,
    // both sides always evaluated; no side effects, so && is exact).
    if s.life == s.life_max && flag4 && !s.is_genuinely_enraged() {
        s.ai3 += 2.0;
    }
    let mut flag5 = true; // the tail alpha fade gate
    let mut num11 = 30;
    let mut num12 = 30;
    let mut num13 = 30;
    let mut num14 = 35;
    let mut num15 = 65;
    if flag {
        // `:45984-45996`
        num6 = 60;
        num8 = 50;
        num9 = 50;
        num10 = 60;
        num7 = 65;
        num11 = 35;
        num12 = 35;
        num13 = 35;
        num14 = 40;
        num15 = 30;
    }
    // `:45997-46001` — the difficulty Remaps.
    let d = env.difficulty;
    num6 = attack_damage_for_projectiles(d, num6 as f32, num11 as f32);
    num8 = attack_damage_for_projectiles(d, num8 as f32, num12 as f32);
    num9 = attack_damage_for_projectiles(d, num9 as f32, num13 as f32);
    num10 = attack_damage_for_projectiles(d, num10 as f32, num14 as f32);
    num7 = attack_damage_for_projectiles(d, num7 as f32, num15 as f32);
    if flag4 {
        // `:46002-46010` — the enrage overrides.
        num6 = 9999;
        num8 = 9999;
        num9 = 9999;
        num10 = 9999;
        num7 = 9999;
        flag2 = true;
    }
    let mut num16 = 1.0f32; // the contact-damage multiplier
    let mut flag6 = true; // the dontTakeDamage gate
    let mut num17 = 0; // the phase/expert timer shave
    if flag {
        num17 += 15; // :46016
    }
    if flag2 {
        num17 += 5; // :46020
    }
    let target_center = if env.target_invalid {
        s.center()
    } else {
        (env.target_cx, env.target_cy)
    };
    let target_vel = (env.target_vx, env.target_vy);
    // `GetTargetData().Center` WITHOUT the invalid fallback — the raw
    // NPCAimedTarget default is (0,0) (`NPCAimedTarget.cs:21`,
    // `GetTargetData` `NPC.cs:6901-6912` returns `default` when
    // invalid; 636 is NOT in `UsesNewTargeting`, `NPCID.cs:10434`).
    // Cases 1 (despawn distance + expert side-step), 3 (the volley),
    // 4 (the lance ring centre), 7 (the wall centre), 10 (the snap),
    // 11 (the spiral centre) and 13 (the fade distance) read it raw.
    let raw_target_center = (env.target_cx, env.target_cy);
    match s.ai0 as i32 {
        0 => {
            // `:46024-46075` — the materialize.
            if s.ai1 == 0.0 {
                s.vx = 0.0;
                s.vy = 5.0;
                // :46030 — the 874 Death Aurora, still, no damage.
                let c = s.center();
                out.requests.push(EmpressRequest::Projectile {
                    kind: 874,
                    x: c.0,
                    y: c.1 - 80.0,
                    vx: 0.0,
                    vy: 0.0,
                    damage: 0,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
            // :46033-46036 — SoundID.Item161 at tick 10 (skipped).
            s.vx *= 0.95;
            s.vy *= 0.95;
            if s.ai1 > 10.0 && s.ai1 < 150.0 {
                // `:46040-46058` — the 2-dust trail: per dust, NewDust
                // (6 draws, w/h = 100), Circular (2), velocity (1),
                // fadeIn (1), CloneDust → NewDust (6) = 16.
                for _ in 0..2 {
                    new_dust_draws(rng, EMPRESS_WIDTH as i32, EMPRESS_HEIGHT as i32);
                    next_vector2_circular(rng, EMPRESS_WIDTH * 3.0, EMPRESS_HEIGHT * 3.0);
                    rng.next_float(); // velocity *= NextFloat() * 0.8f
                    rng.next_float(); // fadeIn 0.6 + NextFloat() * 0.7 * num66
                    new_dust_draws(rng, 0, 0); // CloneDust, Dust.cs:258-281
                }
            }
            s.ai1 += 1.0;
            flag5 = false;
            flag6 = false;
            s.set_opacity((s.ai1 / 180.0).clamp(0.0, 1.0)); // :46063
            if s.ai1 >= 180.0 {
                // `:46064-46074` — the second enrage-latch site.
                if flag4 && !s.is_genuinely_enraged() {
                    s.ai3 += 2.0;
                }
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
                out.requests.push(EmpressRequest::TargetClosest);
            }
        }
        1 => {
            // `:46076-46264` — the dash + pattern pick.
            let mut num33 = if flag { 20.0f32 } else { 45.0f32 };
            if env.get_good_world {
                num33 /= 2.0; // :46081
            }
            let mut bail = false;
            if s.ai1 <= 10.0 {
                if s.ai1 == 0.0 {
                    out.requests.push(EmpressRequest::TargetClosest); // :46087
                }
                if env.target_invalid {
                    // `:46090-46098` — the despawn hand-off.
                    s.ai0 = 13.0;
                    s.ai1 = 0.0;
                    s.ai2 += 1.0;
                    s.vx /= 4.0;
                    s.vy /= 4.0;
                    out.net_update = true;
                    bail = true;
                } else {
                    dash_to(s, (env.target_cx, env.target_cy)); // :46099
                    out.net_update = true;
                }
            }
            if !bail {
                let speed = v_len((s.vx, s.vy));
                if speed > 16.0 && s.ai1 > 10.0 {
                    s.vx /= 2.0; // :46102-46105
                    s.vy /= 2.0;
                }
                s.vx *= 0.92; // :46106
                s.vy *= 0.92;
                s.ai1 += 1.0;
                if s.ai1 >= num33 {
                    // `:46112-46262` — the pattern table.
                    let num34 = s.ai2 as i32;
                    let mut num35 = 2;
                    if !flag {
                        // `:46115-46171` — the classic 10-slot cycle.
                        num35 = match num34 % 10 {
                            0 => 2,
                            1 => 8,
                            2 => 6,
                            3 => 8,
                            4 => 5,
                            5 => 2,
                            6 => 8,
                            7 => 4,
                            8 => 8,
                            _ => 5, // 9
                        };
                        if (s.life as f32) / (s.life_max as f32) <= 0.5 {
                            num35 = 10; // :46167-46170
                        }
                    } else if flag2 {
                        // `:46172-46232` — the expert 10-slot cycle
                        // (num50 = 3 inserted between slots 2 and 4).
                        num35 = match num34 % 10 {
                            0 => 7,
                            1 => 2,
                            2 => 8,
                            3 => 11,
                            4 => 5,
                            5 => 2,
                            6 => 6,
                            7 => 4,
                            8 => 8,
                            _ => 12, // 9
                        };
                    } else {
                        // The phase-2 classic 9-slot cycle (num50 = -1
                        // never matches; the duplicate slot-5 test
                        // `:46208-46215` is idempotent).
                        num35 = match num34 % 9 {
                            0 => 7,
                            1 => 2,
                            2 => 8,
                            3 => 5,
                            4 => 2,
                            5 => 6,
                            6 => 4,
                            7 => 8,
                            _ => 12, // 8
                        };
                    }
                    out.requests.push(EmpressRequest::TargetClosest); // :46233
                    // `:46236-46246` — the enraged despawn windows.
                    let mut flag11 = false;
                    if s.is_genuinely_enraged() {
                        if !env.day_time {
                            flag11 = true;
                        }
                        if env.day_time && env.time >= 53400.0 {
                            flag11 = true;
                        }
                    }
                    if env.target_invalid || v_dist(s.center(), raw_target_center) > num4 || flag11
                    {
                        num35 = 13; // :46247-46250
                    }
                    if num35 == 8 && env.target_cx > s.center().0 {
                        num35 = 9; // :46251-46254 — dash from the left
                    }
                    if flag2 && num35 != 5 && num35 != 12 {
                        // `:46255-46258` — the expert side-step: the
                        // away-direction rotated ±π/2 by the target's
                        // side, 20 speed. `DirectionFrom` reads the RAW
                        // target centre (`:46257`).
                        let away =
                            safe_normalize(v_sub(s.center(), raw_target_center), (0.0, 0.0));
                        // `:46257` — the angle is FLOAT arithmetic
                        // (`(float)Math.PI / 2f * (float)ToDirectionInt`)
                        // widened to double for RotatedBy.
                        let side = to_direction_int(env.target_cx > s.center().0) as f32;
                        let vel = v_scale(rotated_by(away, (PI / 2.0 * side) as f64), 20.0);
                        s.vx = vel.0;
                        s.vy = vel.1;
                    }
                    s.ai0 = num35 as f32;
                    s.ai1 = 0.0;
                    s.ai2 += 1.0;
                    out.net_update = true;
                }
            }
        }
        2 => {
            // `:46265-46320` — the prismatic bolts.
            // :46267-46270 — SoundID.Item164 at tick 0 (skipped).
            let num88 = 90.0 - num17 as f32;
            let muzzle = (-55.0f32, -30.0f32); // vector36
            let aim = v_add(target_center, anchor_left);
            if v_dist(s.center(), aim) > num3 {
                let desired = v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num);
            }
            if s.ai1 < 60.0 {
                do_magic_effect_draws(rng, 1, get_lerp_value(0.0, 60.0, s.ai1, true));
            }
            let num89 = if flag2 { 2 } else { 3 };
            if (s.ai1 as i32) % num89 == 0 && s.ai1 < 60.0 {
                let ai3_arg = s.ai1 / 60.0;
                // `:46291` — NextFloatDirection ALWAYS drawn; the
                // phase-2 expert variant draws NextFloat TOO (`:46294`
                // overwrites, both draws consumed). The angle factors
                // are FLOAT arithmetic widened to double (RotatedBy).
                let dir = rng.next_float_direction();
                let mut vel = rotated_by((0.0, -6.0), (PI / 2.0 * dir) as f64);
                if flag3 {
                    let full = rng.next_float();
                    vel = rotated_by((0.0, -10.0), (PI * 2.0 * full) as f64);
                }
                let c = s.center();
                out.requests.push(EmpressRequest::Projectile {
                    kind: 873,
                    x: c.0 + muzzle.0,
                    y: c.1 + muzzle.1,
                    vx: vel.0,
                    vy: vel.1,
                    damage: num8,
                    ai0: s.target as f32,
                    ai1: ai3_arg,
                });
                // `:46300-46310` — the extra-target volley.
                let num90 = (s.ai1 / num89 as f32) as i32;
                for p in players {
                    if boss_can_shoot_extra_at(s, p, num90 % 3, 3, 2400.0) {
                        out.requests.push(EmpressRequest::Projectile {
                            kind: 873,
                            x: c.0 + muzzle.0,
                            y: c.1 + muzzle.1,
                            vx: vel.0,
                            vy: vel.1,
                            damage: num8,
                            ai0: p.index as f32,
                            ai1: ai3_arg,
                        });
                    }
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= 60.0 + num88 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        3 => {
            // `:46321-46342` — the death-aurora state. The `% 180`
            // volley at tick 0 (`:46330-46334`) reads the RAW target
            // centre (no invalid fallback) and has NO netMode gate.
            s.ai1 += 1.0; // the increment comes FIRST here
            let aim = v_add(target_center, anchor_right);
            if v_dist(s.center(), aim) > num3 {
                let desired = v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num);
            }
            if (s.ai1 as i32) % 180 == 0 {
                out.requests.push(EmpressRequest::Projectile {
                    kind: 874,
                    x: env.target_cx,
                    y: env.target_cy - 100.0,
                    vx: 0.0,
                    vy: 0.0,
                    damage: num5,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
            if s.ai1 >= 120.0 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        4 => {
            // `:46343-46451` — the ethereal lances.
            let num79 = 20 - num17;
            // :46347-46350 — SoundID.Item162 at tick 0 (skipped).
            if s.ai1 >= 6.0 && s.ai1 < 54.0 {
                // `:46351-46355` — the two wing effects, 2 then 4.
                let p = get_lerp_value(0.0, 100.0, s.ai1, true);
                do_magic_effect_draws(rng, 2, p);
                do_magic_effect_draws(rng, 4, p);
            }
            let aim = v_add(target_center, anchor_up);
            if v_dist(s.center(), aim) > num3 {
                let desired = v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num);
            }
            let num80: i32 = if flag2 { 5 } else { 4 };
            if (s.ai1 as i32) % 4 == 0 && s.ai1 < 100.0 {
                // `:46367-46442` — one lance per 4 ticks (num81 = 1).
                let num83 = (s.ai1 as i32) / 4;
                // `:46374` — the ring angle in FLOAT arithmetic.
                let angle = PI / (num80 * 2) as f32 + num83 as f32 * (PI / num80 as f32) + 0.0;
                let mut ring_dir = rotated_by((1.0, 0.0), angle as f64);
                if !flag2 {
                    // `:46375-46378` — the classic outward push.
                    ring_dir.0 += if ring_dir.0 > 0.0 { 0.5 } else { -0.5 };
                }
                ring_dir = normalize(ring_dir); // :46379
                let num84 = if flag2 { 450.0f32 } else { 300.0f32 };
                // `:46385` — the RAW target centre (no invalid fallback).
                let mut center3 = raw_target_center;
                if v_dist(s.center(), center3) <= 2400.0 {
                    // `:46390-46393` — flip against the player's motion.
                    if v_dot(safe_normalize(target_vel, (0.0, 1.0)), ring_dir) > 0.0 {
                        ring_dir = v_scale(ring_dir, -1.0);
                    }
                    let num85 = 90.0f32;
                    let lead = v_add(center3, v_scale(target_vel, num85)); // vector31
                    // vector32 = center + dir*num84 - vel*30 (`:46396`).
                    let mut spawn = v_sub(
                        v_add(center3, v_scale(ring_dir, num84)),
                        v_scale(target_vel, 30.0),
                    );
                    if v_dist(spawn, center3) < num84 {
                        // `:46397-46405` — snap back onto the ring.
                        let mut back = v_sub(center3, spawn);
                        if back == (0.0, 0.0) {
                            back = ring_dir;
                        }
                        spawn = v_sub(center3, v_scale(normalize(back), num84));
                    }
                    let v3 = v_sub(lead, spawn);
                    out.requests.push(EmpressRequest::Projectile {
                        kind: 919,
                        x: spawn.0,
                        y: spawn.1,
                        vx: 0.0,
                        vy: 0.0,
                        damage: num6,
                        ai0: to_rotation(v3),
                        ai1: s.ai1 / 100.0,
                    });
                    // `:46415-46440` — the extra-target lances. NOTE:
                    // the flipped `ring_dir` CARRIES between players
                    // (each flip mutates the shared local, `:46424`).
                    let num86 = (s.ai1 / 4.0) as i32;
                    for p in players {
                        if !boss_can_shoot_extra_at(s, p, num86 % 3, 3, 2400.0) {
                            continue;
                        }
                        center3 = (p.cx, p.cy);
                        let pvel = (p.vx, p.vy);
                        if v_dot(safe_normalize(pvel, (0.0, 1.0)), ring_dir) > 0.0 {
                            ring_dir = v_scale(ring_dir, -1.0);
                        }
                        let lead = v_add(center3, v_scale(pvel, num85));
                        let mut spawn = v_sub(
                            v_add(center3, v_scale(ring_dir, num84)),
                            v_scale(pvel, 30.0),
                        );
                        if v_dist(spawn, center3) < num84 {
                            let mut back = v_sub(center3, spawn);
                            if back == (0.0, 0.0) {
                                back = ring_dir;
                            }
                            spawn = v_sub(center3, v_scale(normalize(back), num84));
                        }
                        out.requests.push(EmpressRequest::Projectile {
                            kind: 919,
                            x: spawn.0,
                            y: spawn.1,
                            vx: 0.0,
                            vy: 0.0,
                            damage: num6,
                            ai0: to_rotation(v_sub(lead, spawn)),
                            ai1: s.ai1 / 100.0,
                        });
                    }
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= 100.0 + num79 as f32 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        5 => {
            // `:46452-46492` — the lasting rainbows.
            // :46454-46457 — SoundID.Item163 at tick 0 (skipped).
            let num61 = 30.0 - num17 as f32;
            let wing = (55.0f32, -30.0f32); // vector19
            let muzzle = v_add(s.center(), wing); // vector20, BEFORE the fly
            if s.ai1 < 42.0 {
                do_magic_effect_draws(rng, 3, get_lerp_value(0.0, 42.0, s.ai1, true));
            }
            let aim = v_add(target_center, anchor_up);
            if v_dist(s.center(), aim) > num3 {
                let desired = v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num);
            }
            if (s.ai1 as i32) % 42 == 0 && s.ai1 < 42.0 {
                // `:46471-46483` — tick 0 only: the 13-bolt ring.
                // `:46473` — num62 is FLOAT: `(float)Math.PI * 2f *
                // NextFloat()`.
                let num62 = PI * 2.0 * rng.next_float();
                // The f32 accumulation `num63 += 1f/13f` runs 13 times
                // (the 13th add lands exactly on 1.0 — hand-traced).
                let step = 1.0f32 / 13.0f32;
                let mut num63 = 0.0f32;
                while num63 < 1.0 {
                    let num64 = num63;
                    // `:46477` — FLOAT angle arithmetic.
                    let angle = PI / 2.0 + PI * 2.0 * num64 + num62;
                    let dir = rotated_by((0.0, 1.0), angle as f64); // vector22
                    let pos = v_add(
                        muzzle,
                        v_scale(rotated_by(dir, -1.5707963705062866), 30.0),
                    );
                    let vel = v_scale(dir, 8.0);
                    out.requests.push(EmpressRequest::Projectile {
                        kind: 872,
                        x: pos.0,
                        y: pos.1,
                        vx: vel.0,
                        vy: vel.1,
                        damage: num9,
                        ai0: 0.0,
                        ai1: num64,
                    });
                    num63 += step;
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= 42.0 + num61 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        6 => {
            // `:46493-46531` — the sun dance.
            let num25 = 120 - num17;
            let muzzle = v_add(s.center(), (0.0, -100.0)); // vector9
            let aim = v_add(target_center, anchor_high);
            if v_dist(s.center(), aim) > num3 {
                // `:46502` — the 0.3-speed / 0.7-accel variant.
                let desired =
                    v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2 * 0.3);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num * 0.7);
            }
            if (s.ai1 as i32) % 60 == 0 && s.ai1 < 180.0 {
                // `:46504-46522` — volleys at ticks 0/60/120.
                let num26 = (s.ai1 as i32) / 60;
                // `:46507` — ToInt (0/1), NOT ToDirectionInt.
                let num27 = if env.target_cx > s.center().0 { 1 } else { 0 };
                let num28 = if flag2 { 8.0f32 } else { 6.0f32 };
                let num29 = 1.0 / num28;
                let mut num30 = 0.0f32;
                while num30 < 1.0 {
                    // `:46516` — FLOAT angle arithmetic, the % 1 wrap.
                    let ai = PI
                        * 2.0
                        * ((num30 + num29 * 0.5 + num26 as f32 * num29 * 0.5) % 1.0
                            + num27 as f32);
                    out.requests.push(EmpressRequest::Projectile {
                        kind: 923,
                        x: muzzle.0,
                        y: muzzle.1,
                        vx: 0.0,
                        vy: 0.0,
                        damage: num10,
                        ai0: ai,
                        ai1: s.who_am_i as f32,
                    });
                    num30 += num29;
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= 180.0 + num25 as f32 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        7 => {
            // `:46532-46631` — the lance walls.
            let mut num68 = 20.0f32;
            let mut num69 = 60.0f32;
            let mut num70 = num69 * 4.0;
            if flag2 {
                num68 = 40.0;
                num69 = 40.0;
                num70 = num69 * 6.0;
            }
            num68 -= num17 as f32;
            let aim = v_add(target_center, anchor_up);
            if v_dist(s.center(), aim) > num3 {
                let desired =
                    v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2 * 0.4);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num);
            }
            if (s.ai1 as i32) as f32 % num69 == 0.0 && s.ai1 < num70 {
                // `:46550-46622` — SoundID.Item162 (skipped), then the
                // DISCARDED NextFloat — it STILL advances the stream.
                rng.next_float(); // :46553
                let num71 = (s.ai1 as i32) / (num69 as i32);
                let mut num72 = 13.0f32;
                let mut num73 = 150.0f32;
                let mut num74 = num72 * num73; // 1950, BEFORE the expert halving
                let mut center2 = (env.target_cx, env.target_cy);
                if v_dist(s.center(), center2) <= 3200.0 {
                    let num75 = 0.4f32;
                    let num76 = 1.4f32;
                    let mut num77 = 1.0f32;
                    if flag2 {
                        num72 += 5.0; // :46568
                        num73 += 50.0; // :46569
                        num77 *= 1.0; // :46570 (no-op in the C#)
                        num74 *= 0.5; // :46571 — 975 expert
                    }
                    let span; // vector26
                    let dir; // vector27
                    match num71 {
                        0 => {
                            // `:46575-46579`
                            center2 = v_add(center2, v_scale(((0.0 - num74) / 2.0, 0.0), num77));
                            span = (0.0, num74);
                            dir = (1.0, 0.0);
                        }
                        1 => {
                            // `:46580-46584`
                            center2 =
                                v_add(center2, v_scale((num74 / 2.0, num73 / 2.0), num77));
                            span = (0.0, num74);
                            dir = (-1.0, 0.0);
                        }
                        2 => {
                            // `:46585-46589`
                            center2 = v_add(
                                center2,
                                v_scale((0.0 - num74, 0.0 - num74), num75 * num77),
                            );
                            span = (num74 * num76, 0.0);
                            dir = (1.0, 1.0);
                        }
                        3 => {
                            // `:46590-46594`
                            center2 = v_add(
                                center2,
                                v_scale(
                                    (num74 * num75 + num73 / 2.0, (0.0 - num74) * num75),
                                    num77,
                                ),
                            );
                            span = ((0.0 - num74) * num76, 0.0);
                            dir = (-1.0, 1.0);
                        }
                        4 => {
                            // `:46595-46599`
                            center2 = v_add(
                                center2,
                                v_scale((0.0 - num74, num74), num75 * num77),
                            );
                            span = (num74 * num76, 0.0);
                            dir = normalize(v_sub((env.target_cx, env.target_cy), center2));
                        }
                        _ => {
                            // `:46600-46604` (case 5)
                            center2 = v_add(
                                center2,
                                v_scale(
                                    (num74 * num75 + num73 / 2.0, num74 * num75),
                                    num77,
                                ),
                            );
                            span = ((0.0 - num74) * num76, 0.0);
                            dir = normalize(v_sub((env.target_cx, env.target_cy), center2));
                        }
                    }
                    // `:46606-46620` — INCLUSIVE `<= 1f`: classic runs
                    // 14 lances (the 13th add of 1f/13f hits exactly
                    // 1.0), expert 18 (1f/18f never lands on 1.0).
                    let step = 1.0f32 / num72;
                    let mut num78 = 0.0f32;
                    while num78 <= 1.0 {
                        let origin = v_add(center2, v_scale(span, num78 - 0.5));
                        let mut v2 = dir;
                        if flag2 {
                            // `:46610-46614` — the expert lead-aim. The
                            // lead is LEFT-associated f32: `(Velocity *
                            // 20f) * num78` (`:46612`).
                            let lead = v_scale(v_scale(target_vel, 20.0), num78);
                            let to_target = normalize(v_sub(
                                v_add((env.target_cx, env.target_cy), lead),
                                origin,
                            ));
                            v2 = safe_normalize(v_lerp(dir, to_target, 0.75), (0.0, 1.0));
                        }
                        out.requests.push(EmpressRequest::Projectile {
                            kind: 919,
                            x: origin.0,
                            y: origin.1,
                            vx: 0.0,
                            vy: 0.0,
                            damage: num7,
                            ai0: to_rotation(v2),
                            ai1: num78,
                        });
                        num78 += step;
                    }
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= num70 + num68 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        8 | 9 => {
            // `:46632-46675` — the side dash.
            let num31 = 20 - num17;
            // `:46638` — invulnerable ONLY in 6..=40.
            flag6 = !(s.ai1 >= 6.0) || !(s.ai1 <= 40.0);
            let num32 = if s.ai0 != 8.0 { 1 } else { -1 };
            // `:46640` — effect 5 runs EVERY tick (progress 0 → no dust).
            do_magic_effect_draws(rng, 5, get_lerp_value(40.0, 90.0, s.ai1, true));
            if s.ai1 <= 40.0 {
                // :46643-46646 — SoundID.Item160 at tick 20 (skipped).
                let aim = v_add(target_center, (num32 as f32 * -550.0, 0.0));
                let desired =
                    v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num * 2.0);
                if s.ai1 == 40.0 {
                    s.vx *= 0.3; // :46649-46652
                    s.vy *= 0.3;
                }
            } else if s.ai1 <= 90.0 {
                // `:46654-46662` — the 50-speed dash, the ×1.5 damage.
                let vel = v_lerp((s.vx, s.vy), (num32 as f32 * 50.0, 0.0), 0.05);
                s.vx = vel.0;
                s.vy = vel.1;
                if s.ai1 == 90.0 {
                    s.vx *= 0.7;
                    s.vy *= 0.7;
                }
                num16 *= 1.5;
            } else {
                s.vx *= 0.92; // :46663-46666
                s.vy *= 0.92;
            }
            s.ai1 += 1.0;
            if s.ai1 >= 90.0 + num31 as f32 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        10 => {
            // `:46676-46707` — the phase transform.
            let num92 = 20 - num17;
            // :46679-46682 — SoundID.Item161 at tick 0 (skipped).
            flag6 = !(s.ai1 >= 30.0) || !(s.ai1 <= 170.0);
            s.vx *= 0.95;
            s.vy *= 0.95;
            if s.ai1 == 90.0 {
                // `:46685-46697` — the latch flip + the teleport.
                if s.ai3 == 0.0 {
                    s.ai3 = 1.0;
                }
                if s.ai3 == 2.0 {
                    s.ai3 = 3.0;
                }
                // `base.Center = GetTargetData().Center + (0,-250)` —
                // the RAW target centre (no invalid fallback).
                let c = (env.target_cx, env.target_cy - 250.0);
                s.x = c.0 - EMPRESS_WIDTH / 2.0;
                s.y = c.1 - EMPRESS_HEIGHT / 2.0;
                out.net_update = true;
            }
            s.ai1 += 1.0;
            if s.ai1 >= 180.0 + num92 as f32 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                s.ai2 = 0.0; // :46703 — the pattern counter reset
                out.net_update = true;
            }
        }
        11 => {
            // `:46708-46793` — the expert-phase-2 spiral bolts.
            // :46710-46713 — SoundID.Item162 at tick 0 (skipped).
            let num57 = 20 - num17;
            if s.ai1 >= 6.0 && s.ai1 < 54.0 {
                let p = get_lerp_value(0.0, 100.0, s.ai1, true);
                do_magic_effect_draws(rng, 2, p);
                do_magic_effect_draws(rng, 4, p);
            }
            let aim = v_add(target_center, anchor_up);
            if v_dist(s.center(), aim) > num3 {
                let desired = v_scale(safe_normalize(v_sub(aim, s.center()), (0.0, 0.0)), num2);
                simple_fly_movement(&mut s.vx, &mut s.vy, desired.0, desired.1, num);
            }
            if (s.ai1 as i32) % 3 == 0 && s.ai1 < 100.0 {
                // `:46727-46784` — the lance behind the player.
                let mut back = safe_normalize(v_scale(target_vel, -1.0), (0.0, -1.0));
                let num59 = 100.0f32;
                let mut center = (env.target_cx, env.target_cy);
                if v_dist(s.center(), center) <= 2400.0 {
                    let lead = v_add(center, v_scale(target_vel, 90.0));
                    let mut spawn = v_add(center, v_scale(back, num59));
                    if v_dist(spawn, center) < num59 {
                        let mut fix = v_sub(center, spawn);
                        if fix == (0.0, 0.0) {
                            fix = back;
                        }
                        spawn = v_sub(center, v_scale(normalize(fix), num59));
                    }
                    out.requests.push(EmpressRequest::Projectile {
                        kind: 919,
                        x: spawn.0,
                        y: spawn.1,
                        vx: 0.0,
                        vy: 0.0,
                        damage: num6,
                        ai0: to_rotation(v_sub(lead, spawn)),
                        ai1: s.ai1 / 100.0,
                    });
                    let num60 = (s.ai1 / 3.0) as i32;
                    for p in players {
                        if !boss_can_shoot_extra_at(s, p, num60 % 3, 3, 2400.0) {
                            continue;
                        }
                        let pvel = (p.vx, p.vy);
                        back = safe_normalize(v_scale(pvel, -1.0), (0.0, -1.0));
                        center = (p.cx, p.cy);
                        let lead = v_add(center, v_scale(pvel, 90.0));
                        let mut spawn = v_add(center, v_scale(back, num59));
                        if v_dist(spawn, center) < num59 {
                            let mut fix = v_sub(center, spawn);
                            if fix == (0.0, 0.0) {
                                fix = back;
                            }
                            spawn = v_sub(center, v_scale(normalize(fix), num59));
                        }
                        out.requests.push(EmpressRequest::Projectile {
                            kind: 919,
                            x: spawn.0,
                            y: spawn.1,
                            vx: 0.0,
                            vy: 0.0,
                            damage: num6,
                            ai0: to_rotation(v_sub(lead, spawn)),
                            ai1: s.ai1 / 100.0,
                        });
                    }
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= 100.0 + num57 as f32 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        12 => {
            // `:46794-46843` — the rotating streaks.
            let num21 = 90.0 - num17 as f32;
            let wing = (-55.0f32, -30.0f32); // vector6
            if s.ai1 == 0.0 {
                // :46798-46802 — SoundID.Item165 (skipped), the rise.
                s.vx = 0.0;
                s.vy = -12.0;
            }
            s.vx *= 0.95;
            s.vy *= 0.95;
            let flag10 = s.ai1 < 60.0 && s.ai1 >= 10.0;
            if flag10 {
                do_magic_effect_draws(rng, 1, get_lerp_value(0.0, 60.0, s.ai1, true));
            }
            let num22 = if flag2 { 4 } else { 6 };
            let num23 = (s.ai1 - 10.0) / 50.0;
            // `:46815` — bitwise & in C#: both sides always evaluated,
            // no side effects, so && is exact.
            if (s.ai1 as i32) % num22 == 0 && flag10 {
                // `:46817` — the discarded `_ = this.ai[1] / 60f` (no
                // RNG, no effect — noted for the audit).
                // `:46818` — the angle rides the 50-tick ramp, FLOAT
                // arithmetic widened to double.
                let vel = rotated_by((0.0, -20.0), (PI * 2.0 * num23) as f64);
                let c = s.center();
                out.requests.push(EmpressRequest::Projectile {
                    kind: 873,
                    x: c.0 + wing.0,
                    y: c.1 + wing.1,
                    vx: vel.0,
                    vy: vel.1,
                    damage: num8,
                    ai0: s.target as f32,
                    ai1: num23,
                });
                // `:46825` — the C# uses `ai[1] % num22` here (always 0
                // inside this branch), NOT the tick/num22 quotient.
                let num24 = (s.ai1 % num22 as f32) as i32;
                for p in players {
                    if boss_can_shoot_extra_at(s, p, num24 % 3, 3, 2400.0) {
                        out.requests.push(EmpressRequest::Projectile {
                            kind: 873,
                            x: c.0 + wing.0,
                            y: c.1 + wing.1,
                            vx: vel.0,
                            vy: vel.1,
                            damage: num8,
                            ai0: p.index as f32,
                            ai1: num23,
                        });
                    }
                }
            }
            s.ai1 += 1.0;
            if s.ai1 >= 60.0 + num21 {
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        13 => {
            // `:46844-46917` — the despawn fade.
            if s.ai1 == 0.0 {
                // :46847-46851 — SoundID.Item165 (skipped), the rise.
                s.vx = 0.0;
                s.vy = -7.0;
            }
            s.vx *= 0.95;
            s.vy *= 0.95;
            out.requests.push(EmpressRequest::TargetClosest); // :46853
            flag5 = false;
            // `:46856-46876` — the fade direction.
            let mut flag7 = false;
            if s.is_genuinely_enraged() {
                if !env.day_time {
                    flag7 = true;
                }
                if env.day_time && env.time >= 53400.0 {
                    flag7 = true;
                }
            }
            if !flag7 {
                // `:46875` — the RAW target centre (no invalid fallback).
                flag7 |= env.target_invalid || v_dist(s.center(), raw_target_center) > num4;
            }
            s.alpha = (s.alpha + to_direction_int(flag7) * 5).clamp(0, 255); // :46877
            let flag9 = s.alpha == 0 || s.alpha == 255;
            // `:46880-46898` — the 5-dust trail: colour (1), NewDust
            // (6, w/h = 200), Circular (2), velocity (1), scale (1),
            // fadeIn (1), CloneDust (6) = 18 per dust.
            for _ in 0..5 {
                rng.next_float(); // hslToRgb(Main.rand.NextFloat(), ...)
                new_dust_draws(rng, EMPRESS_WIDTH as i32 * 2, EMPRESS_HEIGHT as i32 * 2);
                next_vector2_circular(rng, EMPRESS_WIDTH, EMPRESS_HEIGHT);
                rng.next_float(); // velocity *= NextFloat() * 0.8f
                rng.next_float(); // scale 0.9 + NextFloat() * 1.2
                rng.next_float(); // fadeIn 0.4 + NextFloat() * 1.2 * num19
                new_dust_draws(rng, 0, 0); // CloneDust
            }
            s.ai1 += 1.0;
            if s.ai1 >= 20.0 && flag9 {
                if s.alpha == 255 {
                    // `:46904-46911` — despawn: the tail is SKIPPED.
                    out.active = false;
                    return out;
                }
                s.ai0 = 1.0;
                s.ai1 = 0.0;
                out.net_update = true;
            }
        }
        _ => {} // no default arm in the C# switch
    }
    // `:46919-46941` — the tail writes.
    out.dont_take_damage = !flag6;
    s.damage = (s.def_damage as f32 * num16) as i32;
    if flag4 {
        s.damage = 9999;
    }
    s.defense = if flag {
        (s.def_defense as f32 * 1.2) as i32
    } else {
        s.def_defense
    };
    s.local_ai0 += 1.0; // `++this.localAI[0]` (`:46933`)
    if s.local_ai0 >= 44.0 {
        s.local_ai0 = 0.0;
    }
    if flag5 {
        s.alpha = (s.alpha - 5).clamp(0, 255);
    }
    // :46941 — Lighting.AddLight(Center, One * Opacity) (skipped).
    out
}

/// The kind gate for the wiring seam — 636 ONLY (`NPCID.cs:11949`).
pub fn empress_seam_applies(kind: i32) -> bool {
    kind == 636
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN NOT RUN (code-only mode, no compiler
    //! invoked). Each pins one cited block of `AI_120_HallowBoss`; the
    //! RNG witnesses replay the stream draw-for-draw against a clone
    //! (the style-52 pattern).
    use super::*;

    /// Night, classic, valid target at (400, 0), boss centred at the
    /// origin (position (-50,-50)). The enrage tests override fields.
    fn env_night() -> EmpressEnv {
        EmpressEnv {
            target_cx: 400.0,
            target_cy: 0.0,
            ..Default::default()
        }
    }

    fn state_at_center(cx: f32, cy: f32) -> EmpressState {
        EmpressState {
            x: cx - EMPRESS_WIDTH / 2.0,
            y: cy - EMPRESS_HEIGHT / 2.0,
            ..Default::default()
        }
    }

    fn proj(out: &EmpressOut, kind: i32) -> Vec<EmpressRequest> {
        out.requests
            .iter()
            .copied()
            .filter(|r| matches!(r, EmpressRequest::Projectile { kind: k, .. } if *k == kind))
            .collect()
    }

    #[test]
    fn the_set_defaults_row_matches_the_csharp() {
        // `NPC.cs:17188-17206` + `NPCID.cs:11949`.
        assert_eq!(EMPRESS_WIDTH, 100.0);
        assert_eq!(EMPRESS_HEIGHT, 100.0);
        assert_eq!(EMPRESS_DEF_DAMAGE, 80);
        assert_eq!(EMPRESS_DEF_DEFENSE, 50);
        assert_eq!(EMPRESS_LIFE_MAX, 70000);
        let s = EmpressState::default();
        assert_eq!(s.alpha, 255); // Opacity = 0f (`:17203`)
        assert_eq!(s.opacity(), 0.0);
        assert_eq!(s.damage, 80);
        assert_eq!(s.defense, 50);
        assert!(empress_seam_applies(636));
        assert!(!empress_seam_applies(370));
        assert!(!empress_seam_applies(637));
    }

    #[test]
    fn the_phase_latch_predicates() {
        // `:45814-45830`.
        for (ai3, p2, enr) in [
            (0.0f32, false, false),
            (1.0, true, false),
            (2.0, false, true),
            (3.0, true, true),
        ] {
            let s = EmpressState { ai3, ..Default::default() };
            assert_eq!(s.is_in_phase2(), p2, "ai3 {ai3}");
            assert_eq!(s.is_genuinely_enraged(), enr, "ai3 {ai3}");
        }
    }

    #[test]
    fn the_enrage_gate_non_remix_rides_daytime_and_leaves_the_latch() {
        // `:45925-45951` — the non-remix arm returns Main.dayTime and
        // never touches empressRageMode.
        let s = state_at_center(0.0, 0.0);
        let mut env = env_night();
        assert_eq!(should_empress_be_enraged(&s, &env), (false, false));
        env.day_time = true;
        assert_eq!(should_empress_be_enraged(&s, &env), (true, false));
        env.empress_rage_mode = true; // sticky, but non-remix ignores it
        assert_eq!(should_empress_be_enraged(&s, &env), (true, true));
    }

    #[test]
    fn the_enrage_gate_remix_is_sticky_and_surface_double_compared() {
        // `:45927-45948` — the latch sticks; the reset writes false;
        // the compare is `(double)Center.Y < worldSurface * 16.0`.
        let s = state_at_center(0.0, 1000.0);
        let mut env = EmpressEnv {
            remix_world: true,
            world_surface: 100.0, // ×16 = 1600
            ..env_night()
        };
        assert_eq!(should_empress_be_enraged(&s, &env), (true, true)); // 1000 < 1600
        env.empress_rage_mode = true;
        let deep = state_at_center(0.0, 5000.0);
        assert_eq!(should_empress_be_enraged(&deep, &env), (true, true)); // sticky
        env.empress_rage_mode = false;
        assert_eq!(should_empress_be_enraged(&deep, &env), (false, false)); // reset
        // The boundary: Center.Y == surface×16 is NOT above.
        let edge = state_at_center(0.0, 1600.0);
        assert_eq!(should_empress_be_enraged(&edge, &env), (false, false));
    }

    #[test]
    fn the_full_life_day_enrage_latch_and_the_9999_rows() {
        // `:45974-45977` + `:46002-46010` + `:46921-46924`.
        let mut s = state_at_center(0.0, 0.0); // life == lifeMax
        let mut env = env_night();
        env.day_time = true;
        let mut rng = UnifiedRandom::new(7);
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai3, 2.0); // 0 → 2 at full life
        assert_eq!(s.damage, 9999); // the one-shot contact row
        assert!(out.empress_rage_mode == env.empress_rage_mode);
        // A second full-life tick does NOT add again (genuinely enraged).
        let out2 = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai3, 2.0);
        assert_eq!(out2.requests.len(), 0); // ai1 == 1 now, no 874 re-spawn
        // The enraged projectile rows: case 2's streak damage 9999.
        s.ai0 = 2.0;
        s.ai1 = 0.0;
        let out3 = empress_ai_step(&mut s, &env, &mut rng, &[]);
        let bolts = proj(&out3, 873);
        assert_eq!(bolts.len(), 1);
        match bolts[0] {
            EmpressRequest::Projectile { damage, .. } => assert_eq!(damage, 9999),
            _ => unreachable!(),
        }
        // flag2 forced true by the enrage → num17 += 5 → the state ends
        // at 60 + (90-5) = 145 ticks (`:46018-46021`, `:46313`).
        s.ai1 = 144.0;
        let out4 = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai0, 1.0);
        assert!(out4.net_update);
    }

    #[test]
    fn the_materialize_spawns_the_still_aurora_and_ramps_opacity() {
        // `:46024-46063` — tick 0: velocity (0,5), the 874 at
        // centre+(0,-80), zero damage/ai; the ×0.95 damp; Opacity =
        // ai1/180 AFTER the increment.
        let mut s = state_at_center(0.0, 0.0);
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.vx, 0.0f32);
        assert_eq!(s.vy, 4.75f32); // 5 × 0.95, exact in f32
        assert_eq!(s.ai1, 1.0);
        assert_eq!(s.alpha, ((1.0 - 1.0 / 180.0) * 255.0) as i32); // 253
        assert!(out.dont_take_damage); // flag6 = false in state 0
        assert_eq!(out.requests.len(), 1);
        assert_eq!(
            out.requests[0],
            EmpressRequest::Projectile {
                kind: 874,
                x: 0.0,
                y: -80.0,
                vx: 0.0,
                vy: 0.0,
                damage: 0,
                ai0: 0.0,
                ai1: 0.0,
            }
        );
        // Ticks 1..10 draw NOTHING (the trail starts at ai1 > 10).
        let witness = rng.clone();
        for _ in 0..10 {
            empress_ai_step(&mut s, &env, &mut rng, &[]);
        }
        assert_eq!(rng, witness, "ticks 1..10 consume zero draws");
        assert_eq!(s.ai1, 11.0);
    }

    #[test]
    fn the_materialize_trail_consumes_32_draws_per_tick() {
        // `:46040-46058` — 2 dust × 16: NewDust (6, w/h = 100) +
        // Circular (2) + velocity (1) + fadeIn (1) + CloneDust (6).
        let mut s = state_at_center(0.0, 0.0);
        s.ai1 = 11.0; // the trail window is 10 < ai1 < 150
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let mut witness = rng.clone();
        for _ in 0..2 {
            witness.next_max(96); // Next(100-4)
            witness.next_max(96);
            witness.next_range(-20, 21);
            witness.next_range(-20, 21);
            witness.next_max(3);
            witness.next_range(-20, 21);
            let _ = witness.next_float(); // Circular's unit angle
            let _ = witness.next_float(); // Circular's radius
            let _ = witness.next_float(); // velocity
            let _ = witness.next_float(); // fadeIn
            witness.next_max(1); // CloneDust: Next(5-4)
            witness.next_max(1);
            witness.next_range(-20, 21);
            witness.next_range(-20, 21);
            witness.next_max(3);
            witness.next_range(-20, 21);
        }
        empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(rng, witness, "exactly 32 draws in C# order");
    }

    #[test]
    fn the_materialize_hands_off_at_tick_180_with_the_second_latch() {
        // `:46064-46074` — the enrage latch fires HERE too (a boss that
        // took damage before materializing never latched at the head).
        let mut s = state_at_center(0.0, 0.0);
        s.ai1 = 179.0;
        s.life = 69999; // damaged → the head gate skipped
        let mut env = env_night();
        env.day_time = true;
        let mut rng = UnifiedRandom::new(7);
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai3, 2.0);
        assert_eq!(s.ai0, 1.0);
        assert_eq!(s.ai1, 0.0);
        assert_eq!(s.alpha, 0); // Opacity clamped to 1
        assert!(out.net_update);
        assert_eq!(out.requests.last(), Some(&EmpressRequest::TargetClosest));
    }

    #[test]
    fn the_dash_to_aim_and_the_18_cap() {
        // `:46944-46960` — hand-traced: centre (0,0), target (400,0):
        // tp = (400,-300), dist 500 > 200 → pull 100 → (320,-240);
        // len 400 → lerp (400-100)/500 = 0.6; num = 18;
        // vel = Lerp((14.4,-10.8), (53.333..,-40), 0.6).
        let mut s = state_at_center(0.0, 0.0);
        dash_to(&mut s, (400.0, 0.0));
        let ux = 320.0f32 / 400.0;
        let uy = -240.0f32 / 400.0;
        let want_x = 18.0 * ux + (320.0 / 6.0 - 18.0 * ux) * 0.6;
        let want_y = 18.0 * uy + (-240.0 / 6.0 - 18.0 * uy) * 0.6;
        assert!((s.vx - want_x).abs() < 1e-4, "vx {} want {}", s.vx, want_x);
        assert!((s.vy - want_y).abs() < 1e-4, "vy {} want {}", s.vy, want_y);
        // Inside 200 px: no pull; len ≤ 100 → lerp 0 → the capped aim.
        let mut s2 = state_at_center(0.0, 0.0);
        dash_to(&mut s2, (0.0, 350.0)); // tp = (0,50), len 50
        assert_eq!(s2.vx, 0.0f32);
        assert_eq!(s2.vy, 18.0f32); // SafeNormalize((0,50)) * 18
    }

    #[test]
    fn the_pattern_pick_invalid_target_bails_to_despawn() {
        // `:46090-46098` — ai0 13, ai2++, velocity ÷4; the C# `break`
        // exits the switch (the port's bail flag skips the REST of
        // case 1) but the tail writes STILL run.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 1.0;
        s.vx = 8.0;
        s.vy = -4.0;
        let mut env = env_night();
        env.target_invalid = true;
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai0, 13.0);
        assert_eq!(s.ai1, 0.0);
        assert_eq!(s.ai2, 1.0);
        assert_eq!(s.vx, 2.0f32);
        assert_eq!(s.vy, -1.0f32);
        assert!(out.net_update);
        assert_eq!(out.requests, vec![EmpressRequest::TargetClosest]);
        assert_eq!(rng, witness, "the bail path draws nothing");
    }

    #[test]
    fn the_classic_cycle_and_the_half_life_transform_force() {
        // `:46115-46170` — the 10-slot cycle; hp ≤ 50 % forces 10.
        // num33 = 45 classic → the pick fires when ai1 hits 45.
        let cycle = [2, 8, 6, 8, 5, 2, 8, 4, 8, 5];
        for (slot, want) in cycle.iter().enumerate() {
            let mut s = state_at_center(0.0, 0.0);
            s.ai0 = 1.0;
            s.ai1 = 44.0;
            s.ai2 = slot as f32;
            let env = env_night(); // target right → 8 flips to 9
            let mut rng = UnifiedRandom::new(7);
            empress_ai_step(&mut s, &env, &mut rng, &[]);
            let expect = if *want == 8 { 9 } else { *want };
            assert_eq!(s.ai0, expect as f32, "slot {slot}");
            assert_eq!(s.ai2, (slot + 1) as f32);
            assert_eq!(s.ai1, 0.0);
        }
        // The transform force at half life.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 1.0;
        s.ai1 = 44.0;
        s.life = 35000; // exactly 50 %
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai0, 10.0);
    }

    #[test]
    fn the_phase2_cycles_classic_nine_expert_ten() {
        // `:46172-46232` — classic 9-slot [7,2,8,5,2,6,4,8,12];
        // expert 10-slot [7,2,8,11,5,2,6,4,8,12]. num33 = 20 phase 2.
        let classic = [7, 2, 8, 5, 2, 6, 4, 8, 12];
        for (slot, want) in classic.iter().enumerate() {
            let mut s = state_at_center(0.0, 0.0);
            s.ai0 = 1.0;
            s.ai1 = 19.0;
            s.ai2 = slot as f32;
            s.ai3 = 1.0;
            let env = env_night();
            let mut rng = UnifiedRandom::new(7);
            empress_ai_step(&mut s, &env, &mut rng, &[]);
            let expect = if *want == 8 { 9 } else { *want };
            assert_eq!(s.ai0, expect as f32, "p2 classic slot {slot}");
        }
        let expert = [7, 2, 8, 11, 5, 2, 6, 4, 8, 12];
        for (slot, want) in expert.iter().enumerate() {
            let mut s = state_at_center(0.0, 0.0);
            s.ai0 = 1.0;
            s.ai1 = 19.0;
            s.ai2 = slot as f32;
            s.ai3 = 1.0;
            let mut env = env_night();
            env.difficulty = 2.0;
            let mut rng = UnifiedRandom::new(7);
            empress_ai_step(&mut s, &env, &mut rng, &[]);
            let expect = if *want == 8 { 9 } else { *want };
            assert_eq!(s.ai0, expect as f32, "p2 expert slot {slot}");
        }
    }

    #[test]
    fn the_enraged_night_and_dusk_windows_force_despawn() {
        // `:46236-46250` — genuinely enraged + night → 13; enraged +
        // day with time ≥ 53400 → 13; enraged mid-day keeps the cycle.
        for (day, time, want13) in [(false, 0.0, true), (true, 53400.0, true), (true, 53399.0, false)] {
            let mut s = state_at_center(0.0, 0.0);
            s.ai0 = 1.0;
            s.ai1 = 44.0;
            s.ai3 = 2.0;
            s.life = 60000; // damaged → no re-latch
            let mut env = env_night();
            env.day_time = day;
            env.time = time;
            let mut rng = UnifiedRandom::new(7);
            empress_ai_step(&mut s, &env, &mut rng, &[]);
            if want13 {
                assert_eq!(s.ai0, 13.0, "day {day} time {time}");
            } else {
                assert_eq!(s.ai0, 2.0, "mid-day keeps the cycle");
            }
        }
    }

    #[test]
    fn the_expert_side_step_rotates_the_away_direction() {
        // `:46255-46258` — target right → ToDirectionInt +1 → the away
        // vector (-1,0) rotated +π/2 → (0,-1) × 20.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 1.0;
        s.ai1 = 19.0; // phase-2 num33 = 20
        s.ai3 = 1.0;
        let mut env = env_night();
        env.difficulty = 2.0;
        let mut rng = UnifiedRandom::new(7);
        empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai0, 7.0); // slot 0 → lance walls
        assert!(s.vx.abs() < 1e-5, "vx {}", s.vx);
        assert!((s.vy + 20.0).abs() < 1e-4, "vy {}", s.vy);
    }

    #[test]
    fn the_prismatic_bolts_cadence_and_the_both_draws_variant() {
        // `:46265-46320` — classic fires every 3 ticks < 60 (20 bolts),
        // ONE NextFloatDirection each; phase-2 expert fires every 2
        // (30 bolts) with BOTH draws (`:46291` + `:46294`).
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 2.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let mut bolts = 0;
        for _ in 0..150 {
            let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
            bolts += proj(&out, 873).len();
        }
        assert_eq!(bolts, 20);
        assert_eq!(s.ai0, 1.0); // ended at 60 + 90 = 150
        // The stream: 60 ticks × effect-1 (2 dust × 10 draws) + 20
        // NextFloatDirection, in tick order.
        let mut rng2 = UnifiedRandom::new(7);
        let mut witness = rng2.clone();
        let mut s2 = state_at_center(0.0, 0.0);
        s2.ai0 = 2.0;
        empress_ai_step(&mut s2, &env, &mut rng2, &[]); // tick 0
        for _ in 0..2 {
            let _ = witness.next_float(); // CircularEdge unit
            let _ = witness.next_float(); // velocity scale
            let _ = witness.next_float(); // colour
            let _ = witness.next_float(); // scale
            witness.next_max(1);
            witness.next_max(1);
            witness.next_range(-20, 21);
            witness.next_range(-20, 21);
            witness.next_max(3);
            witness.next_range(-20, 21);
        }
        let _ = witness.next_float_direction(); // the tick-0 bolt
        assert_eq!(rng2, witness, "tick 0: 20 dust draws + 1 direction");
        // Phase-2 expert: both draws fire, the second overwrites.
        let mut s3 = state_at_center(0.0, 0.0);
        s3.ai0 = 2.0;
        s3.ai3 = 1.0;
        let mut env3 = env_night();
        env3.difficulty = 2.0;
        let mut rng3 = UnifiedRandom::new(7);
        let mut witness3 = rng3.clone();
        let out3 = empress_ai_step(&mut s3, &env3, &mut rng3, &[]);
        for _ in 0..2 {
            let _ = witness3.next_float();
            let _ = witness3.next_float();
            let _ = witness3.next_float();
            let _ = witness3.next_float();
            witness3.next_max(1);
            witness3.next_max(1);
            witness3.next_range(-20, 21);
            witness3.next_range(-20, 21);
            witness3.next_max(3);
            witness3.next_range(-20, 21);
        }
        let dir = witness3.next_float_direction(); // drawn FIRST
        let full = witness3.next_float(); // drawn SECOND, overwrites
        assert_eq!(rng3, witness3, "both draws consumed in order");
        let want = rotated_by((0.0, -10.0), (PI * 2.0 * full) as f64);
        let _ = dir; // the discarded first draw still advanced the stream
        match proj(&out3, 873)[0] {
            EmpressRequest::Projectile { vx, vy, damage, .. } => {
                assert_eq!(vx, want.0);
                assert_eq!(vy, want.1);
                assert_eq!(damage, 35); // phase-2 expert streak remap
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_extra_target_scan_skips_the_owner_and_rotates_by_three() {
        // `:46300-46310` + `Boss_CanShootExtraAt` `:46962-46982`.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 2.0;
        s.target = 0;
        let env = env_night();
        let players = [
            EmpressExtraPlayer { index: 0, cx: 100.0, cy: 0.0, vx: 0.0, vy: 0.0, active: true, dead: false, interaction: true },
            EmpressExtraPlayer { index: 1, cx: 100.0, cy: 0.0, vx: 0.0, vy: 0.0, active: true, dead: false, interaction: true },
            EmpressExtraPlayer { index: 3, cx: 100.0, cy: 0.0, vx: 0.0, vy: 0.0, active: true, dead: false, interaction: true },
            EmpressExtraPlayer { index: 6, cx: 100.0, cy: 0.0, vx: 0.0, vy: 0.0, active: true, dead: true, interaction: true },
        ];
        let mut rng = UnifiedRandom::new(7);
        let out = empress_ai_step(&mut s, &env, &mut rng, &players);
        // tick 0: num90 = 0 → index % 3 == 0 → slot 3 passes (0 is the
        // owner, 6 is dead); the main bolt + 1 extra.
        let bolts = proj(&out, 873);
        assert_eq!(bolts.len(), 2);
        match bolts[1] {
            EmpressRequest::Projectile { ai0, .. } => assert_eq!(ai0, 3.0f32),
            _ => unreachable!(),
        }
        // tick 3: num90 = 1 → index % 3 == 1 → slot 1 passes.
        s.ai1 = 3.0;
        let out2 = empress_ai_step(&mut s, &env, &mut rng, &players);
        let bolts2 = proj(&out2, 873);
        assert_eq!(bolts2.len(), 2);
        match bolts2[1] {
            EmpressRequest::Projectile { ai0, .. } => assert_eq!(ai0, 1.0f32),
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_death_aurora_state_is_unreachable_but_faithful() {
        // `:46321-46342` — the increment comes FIRST; `% 180` never
        // fires inside 1..120; the state is not in any cycle table.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 3.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let mut auroras = 0;
        for _ in 0..120 {
            let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
            auroras += proj(&out, 874).len();
        }
        assert_eq!(auroras, 0);
        assert_eq!(s.ai0, 1.0);
        assert_eq!(s.ai1, 0.0);
    }

    #[test]
    fn the_ethereal_lances_ring_and_the_motion_flip() {
        // `:46343-46451` — hand-traced tick 0, classic: num83 = 0,
        // angle = π/8 → dir ≈ (0.92388, 0.38268); the classic outward
        // push +0.5 → normalize ≈ (0.97815, 0.26287); the target is
        // still → no flip; spawn = targetCentre(400,0) + dir×300
        // (`:46396` vector32 = center3 + vector30*num84 - vel*30);
        // ai0 = atan2 of (lead - spawn) = atan2(-78.86, -293.44).
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 4.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(rng, witness, "the lance state draws NO gameplay RNG");
        let lances = proj(&out, 919);
        assert_eq!(lances.len(), 1);
        // `:46374` — the ring angle is FLOAT arithmetic in the C#
        // ((float)Math.PI / 8f); mirror the f32 rounding, then the
        // RotatedBy cos/sin (the module's rotated_by seam).
        let a32 = (PI / 8.0) as f32;
        let dx = (a32 as f64).cos() as f32 + 0.5;
        let dy = (a32 as f64).sin() as f32;
        let len = (dx * dx + dy * dy).sqrt();
        let (ux, uy) = (dx / len, dy / len);
        match lances[0] {
            EmpressRequest::Projectile { x, y, ai0, ai1, damage, .. } => {
                assert!((x - (400.0 + ux * 300.0)).abs() < 1e-3, "x {x}");
                assert!((y - uy * 300.0).abs() < 1e-3, "y {y}");
                let want_rot = (-(uy as f64) * 300.0).atan2(-(ux as f64) * 300.0) as f32;
                assert!((ai0 - want_rot).abs() < 1e-5, "ai0 {ai0}");
                assert_eq!(ai1, 0.0f32);
                assert_eq!(damage, 50); // classic lance remap
            }
            _ => unreachable!(),
        }
        // The motion flip: target velocity along +dir → dot > 0 → the
        // ring direction flips before the spawn.
        let mut s2 = state_at_center(0.0, 0.0);
        s2.ai0 = 4.0;
        let mut env2 = env_night();
        env2.target_vx = 1.0;
        let mut rng2 = UnifiedRandom::new(7);
        let out2 = empress_ai_step(&mut s2, &env2, &mut rng2, &[]);
        match proj(&out2, 919)[0] {
            EmpressRequest::Projectile { x, .. } => {
                assert!((x - (400.0 - ux * 300.0)).abs() < 1e-3, "flipped x {x}");
            }
            _ => unreachable!(),
        }
        // The wing effects: ticks 6..53 draw 2×(4 dust × 10) = 80/tick.
        let mut s3 = state_at_center(0.0, 0.0);
        s3.ai0 = 4.0;
        s3.ai1 = 6.0;
        let mut rng3 = UnifiedRandom::new(7);
        let mut witness3 = rng3.clone();
        empress_ai_step(&mut s3, &env, &mut rng3, &[]);
        for _ in 0..8 {
            let _ = witness3.next_float();
            let _ = witness3.next_float();
            let _ = witness3.next_float();
            let _ = witness3.next_float();
            witness3.next_max(1);
            witness3.next_max(1);
            witness3.next_range(-20, 21);
            witness3.next_range(-20, 21);
            witness3.next_max(3);
            witness3.next_range(-20, 21);
        }
        assert_eq!(rng3, witness3, "effects 2+4: 8 dust × 10 draws");
    }

    #[test]
    fn the_lasting_rainbows_thirteen_bolt_ring() {
        // `:46452-46492` — tick 0: ONE NextFloat start angle, then the
        // f32 `+= 1f/13f` loop runs exactly 13 times (the 13th add
        // lands on 1.0 — hand-traced with fround).
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 5.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let mut witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        // tick 0: effect 3 (2 dust × 10) THEN the ring's NextFloat.
        for _ in 0..2 {
            let _ = witness.next_float();
            let _ = witness.next_float();
            let _ = witness.next_float();
            let _ = witness.next_float();
            witness.next_max(1);
            witness.next_max(1);
            witness.next_range(-20, 21);
            witness.next_range(-20, 21);
            witness.next_max(3);
            witness.next_range(-20, 21);
        }
        let start = witness.next_float();
        assert_eq!(rng, witness, "20 dust draws + 1 ring angle");
        let ring = proj(&out, 872);
        assert_eq!(ring.len(), 13);
        let num62 = PI * 2.0 * start;
        // bolt 0: num64 = 0 → angle = π/2 + num62; dir = UnitY rotated;
        // pos = muzzle + dir rotated -π/2 × 30; vel = dir × 8.
        let muzzle = (55.0f32, -30.0f32); // centre (0,0) + (55,-30)
        let angle = PI / 2.0 + num62;
        let dir = rotated_by((0.0, 1.0), angle as f64);
        let pos = v_add(muzzle, v_scale(rotated_by(dir, -1.5707963705062866), 30.0));
        match ring[0] {
            EmpressRequest::Projectile { x, y, vx, vy, ai1, damage, .. } => {
                assert_eq!(x, pos.0);
                assert_eq!(y, pos.1);
                assert_eq!(vx, dir.0 * 8.0);
                assert_eq!(vy, dir.1 * 8.0);
                assert_eq!(ai1, 0.0f32);
                assert_eq!(damage, 45); // classic rainbow remap
            }
            _ => unreachable!(),
        }
        // The last bolt's ai1 is the 12th f32 accumulation.
        let mut acc = 0.0f32;
        for _ in 0..12 {
            acc += 1.0f32 / 13.0f32;
        }
        match ring[12] {
            EmpressRequest::Projectile { ai1, .. } => assert_eq!(ai1, acc),
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_sun_dance_volleys_six_then_eight() {
        // `:46493-46531` — volleys at 0/60/120; classic 6 per volley
        // (the f32 1/6 loop lands on 1.0 at the 6th add), expert 8.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 6.0;
        s.who_am_i = 42;
        let env = env_night(); // target right → num27 = 1
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(rng, witness, "the sun dance draws NO RNG");
        let dances = proj(&out, 923);
        assert_eq!(dances.len(), 6);
        // bolt 0: num26 = 0 → ai = 2π × ((0 + 1/12) % 1 + 1) = 2π×13/12.
        let want = PI * 2.0 * ((1.0f32 / 6.0 * 0.5) % 1.0 + 1.0);
        match dances[0] {
            EmpressRequest::Projectile { ai0, ai1, x, y, damage, .. } => {
                assert_eq!(ai0, want);
                assert_eq!(ai1, 42.0f32); // whoAmI rides ai[1]
                assert_eq!(x, 0.0f32); // muzzle = centre + (0,-100)
                assert_eq!(y, -100.0f32);
                assert_eq!(damage, 50); // classic sun-dance remap
            }
            _ => unreachable!(),
        }
        // Expert: 8 per volley, num27 unchanged.
        let mut s2 = state_at_center(0.0, 0.0);
        s2.ai0 = 6.0;
        let mut env2 = env_night();
        env2.difficulty = 2.0;
        let mut rng2 = UnifiedRandom::new(7);
        let out2 = empress_ai_step(&mut s2, &env2, &mut rng2, &[]);
        assert_eq!(proj(&out2, 923).len(), 8);
        // The state ends at 180 + (120 - 5) = 295 expert ticks.
        let mut total = 1;
        while s2.ai0 == 6.0 {
            empress_ai_step(&mut s2, &env2, &mut rng2, &[]);
            total += 1;
        }
        assert_eq!(total, 295);
    }

    #[test]
    fn the_lance_walls_discarded_draw_and_the_inclusive_loop() {
        // `:46532-46631` — the DISCARDED NextFloat (`:46553`) still
        // advances the stream; classic wall 0 runs 14 lances (the
        // inclusive `<= 1f` loop), expert 18.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 7.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let mut witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        let _ = witness.next_float(); // the discarded draw
        assert_eq!(rng, witness, "exactly ONE discarded NextFloat");
        let lances = proj(&out, 919);
        assert_eq!(lances.len(), 14);
        // Wall 0: centre2 = (400,0) + (-975,0) = (-575,0); span (0,1950);
        // origin = (-575, 1950×(num78-0.5)); dir (1,0) → ai0 = 0.
        match lances[0] {
            EmpressRequest::Projectile { x, y, ai0, ai1, damage, .. } => {
                assert_eq!(x, -575.0f32);
                assert_eq!(y, -975.0f32);
                assert_eq!(ai0, 0.0f32); // ToRotation((1,0))
                assert_eq!(ai1, 0.0f32);
                assert_eq!(damage, 70); // classic wall remap
            }
            _ => unreachable!(),
        }
        // The 14th lance: num78 == 1.0 exactly (hand-traced f32).
        match lances[13] {
            EmpressRequest::Projectile { y, ai1, .. } => {
                assert_eq!(ai1, 1.0f32);
                assert_eq!(y, 975.0f32);
            }
            _ => unreachable!(),
        }
        // Expert wall 0: num74 = 975, centre2 = (-87.5, 0), 18 lances.
        let mut s2 = state_at_center(0.0, 0.0);
        s2.ai0 = 7.0;
        let mut env2 = env_night();
        env2.difficulty = 2.0;
        let mut rng2 = UnifiedRandom::new(7);
        let out2 = empress_ai_step(&mut s2, &env2, &mut rng2, &[]);
        let lances2 = proj(&out2, 919);
        assert_eq!(lances2.len(), 18);
        match lances2[0] {
            EmpressRequest::Projectile { x, damage, .. } => {
                assert_eq!(x, -87.5f32);
                assert_eq!(damage, 65); // phase-1 expert wall remap (num7=70→num15=65)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_side_dash_invulnerability_window_and_damage_multiplier() {
        // `:46632-46675` — invulnerable ONLY 6..40; ×1.5 damage 41..90.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 8.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        // tick 0: effect 5 progress 0 → NO draws; vulnerable.
        let witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(rng, witness, "progress 0 draws nothing");
        assert!(!out.dont_take_damage); // flag6 true → vulnerable
        assert_eq!(s.damage, 80);
        // tick 6..40: invulnerable.
        s.ai1 = 6.0;
        let out6 = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert!(out6.dont_take_damage);
        // tick 41: vulnerable again, the dash branch, damage ×1.5.
        s.ai1 = 41.0;
        let out41 = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert!(!out41.dont_take_damage);
        assert_eq!(s.damage, 120); // (80 × 1.5) truncated
        // The effect-5 draws at tick 41: progress (41-40)/50 = 0.02 →
        // ONE Next(86,92) + 5 dust × 10.
        let mut s2 = state_at_center(0.0, 0.0);
        s2.ai0 = 8.0;
        s2.ai1 = 41.0;
        let mut rng2 = UnifiedRandom::new(7);
        let mut witness2 = rng2.clone();
        empress_ai_step(&mut s2, &env, &mut rng2, &[]);
        witness2.next_range(86, 92); // drawn even though progress < 1
        for _ in 0..5 {
            let _ = witness2.next_float();
            let _ = witness2.next_float();
            let _ = witness2.next_float();
            let _ = witness2.next_float();
            witness2.next_max(1);
            witness2.next_max(1);
            witness2.next_range(-20, 21);
            witness2.next_range(-20, 21);
            witness2.next_max(3);
            witness2.next_range(-20, 21);
        }
        assert_eq!(rng2, witness2, "effect 5: 1 + 50 draws");
        // tick 90: progress ≥ 1 → the Next(86,92) STILL drawn, count 0.
        let mut s3 = state_at_center(0.0, 0.0);
        s3.ai0 = 8.0;
        s3.ai1 = 90.0;
        let mut rng3 = UnifiedRandom::new(7);
        let mut witness3 = rng3.clone();
        empress_ai_step(&mut s3, &env, &mut rng3, &[]);
        witness3.next_range(86, 92);
        assert_eq!(rng3, witness3, "progress ≥ 1: the one draw survives");
    }

    #[test]
    fn the_transform_flips_the_latch_and_snaps_over_the_target() {
        // `:46676-46707` — tick 90: ai3 0→1 / 2→3, centre = target +
        // (0,-250); the end resets ai2.
        for (from, to) in [(0.0f32, 1.0f32), (2.0, 3.0)] {
            let mut s = state_at_center(0.0, 0.0);
            s.ai0 = 10.0;
            s.ai1 = 90.0;
            s.ai3 = from;
            s.life = 60000; // damaged → no head re-latch
            let env = env_night();
            let mut rng = UnifiedRandom::new(7);
            let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
            assert_eq!(s.ai3, to);
            assert_eq!(s.center(), (400.0f32, -250.0f32));
            assert!(out.net_update);
        }
        // The state end: 180 + 20 = 200 classic ticks, ai2 reset.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 10.0;
        s.ai1 = 199.0;
        s.ai2 = 7.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.ai0, 1.0);
        assert_eq!(s.ai2, 0.0);
    }

    #[test]
    fn the_spiral_bolts_spawn_behind_the_players_motion() {
        // `:46708-46793` — hand-traced tick 0: target velocity (3,0) →
        // back = (-1,0); spawn = (400,0) + (-100,0) = (300,0); lead =
        // (670,0); ai0 = atan2(0, 370) = 0.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 11.0;
        s.ai3 = 1.0; // phase 2 → the 60/35 lance remap
        let mut env = env_night();
        env.difficulty = 2.0;
        env.target_vx = 3.0;
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(rng, witness, "the spiral state draws NO gameplay RNG");
        let lances = proj(&out, 919);
        assert_eq!(lances.len(), 1);
        match lances[0] {
            EmpressRequest::Projectile { x, y, ai0, damage, .. } => {
                assert_eq!(x, 300.0f32);
                assert_eq!(y, 0.0f32);
                assert_eq!(ai0, 0.0f32);
                assert_eq!(damage, 35); // phase-2 expert lance remap (60→35)
            }
            _ => unreachable!(),
        }
        // A still target: back defaults to (0,-1) → spawn (400,-100).
        let mut s2 = state_at_center(0.0, 0.0);
        s2.ai0 = 11.0;
        s2.ai3 = 1.0;
        let mut env2 = env_night();
        env2.difficulty = 2.0;
        let mut rng2 = UnifiedRandom::new(7);
        let out2 = empress_ai_step(&mut s2, &env2, &mut rng2, &[]);
        match proj(&out2, 919)[0] {
            EmpressRequest::Projectile { x, y, .. } => {
                assert_eq!(x, 400.0f32);
                assert_eq!(y, -100.0f32);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_rotating_streaks_angle_ramp_and_the_modulo_quirk() {
        // `:46794-46843` — the rise (0,-12) ×0.95; streaks every 6
        // ticks in 10..60; the angle = 2π×(ai1-10)/50; the extra-shot
        // rotation index is `ai1 % num22` (always 0 here), NOT the
        // quotient (`:46825`).
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 12.0;
        s.target = 0;
        let env = env_night();
        let players = [EmpressExtraPlayer {
            index: 3, cx: 100.0, cy: 0.0, vx: 0.0, vy: 0.0,
            active: true, dead: false, interaction: true,
        }];
        let mut rng = UnifiedRandom::new(7);
        let out = empress_ai_step(&mut s, &env, &mut rng, &players);
        assert_eq!(s.vy, -11.4f32); // -12 × 0.95, exact in f32
        assert_eq!(proj(&out, 873).len(), 0); // tick 0 < 10
        // tick 12: fires; angle = 2π×0.04; the extra slot 3 passes
        // (num24 = 12 % 6 = 0 → index % 3 == 0).
        s.ai1 = 12.0;
        let out12 = empress_ai_step(&mut s, &env, &mut rng, &players);
        let bolts = proj(&out12, 873);
        assert_eq!(bolts.len(), 2); // main + slot 3
        let num23 = (12.0f32 - 10.0) / 50.0;
        let want = rotated_by((0.0, -20.0), (PI * 2.0 * num23) as f64);
        match bolts[0] {
            EmpressRequest::Projectile { vx, vy, ai1, .. } => {
                assert_eq!(vx, want.0);
                assert_eq!(vy, want.1);
                assert_eq!(ai1, num23);
            }
            _ => unreachable!(),
        }
        match bolts[1] {
            EmpressRequest::Projectile { ai0, .. } => assert_eq!(ai0, 3.0f32),
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_despawn_fade_trail_and_the_alpha_255_early_return() {
        // `:46844-46917` — the rise (0,-7); the 5-dust trail = 90
        // draws; alpha ramps ±5; at 255 with ai1 ≥ 20 the tail is
        // SKIPPED (active = false, no damage/defense/localAI writes).
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 13.0;
        s.alpha = 250;
        let mut env = env_night();
        env.target_invalid = true; // fade OUT (+5)
        let mut rng = UnifiedRandom::new(7);
        let mut witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(s.vy, -6.65f32); // -7 × 0.95, exact in f32
        assert_eq!(s.alpha, 255);
        assert_eq!(s.ai1, 1.0); // < 20 → no despawn yet
        assert!(out.active);
        for _ in 0..5 {
            let _ = witness.next_float(); // colour
            witness.next_max(196); // Next(200-4)
            witness.next_max(196);
            witness.next_range(-20, 21);
            witness.next_range(-20, 21);
            witness.next_max(3);
            witness.next_range(-20, 21);
            let _ = witness.next_float(); // Circular unit
            let _ = witness.next_float(); // Circular radius
            let _ = witness.next_float(); // velocity
            let _ = witness.next_float(); // scale
            let _ = witness.next_float(); // fadeIn
            witness.next_max(1); // CloneDust ×6
            witness.next_max(1);
            witness.next_range(-20, 21);
            witness.next_range(-20, 21);
            witness.next_max(3);
            witness.next_range(-20, 21);
        }
        assert_eq!(rng, witness, "5 dust × 18 draws");
        // Run to the despawn: ai1 hits 20 with alpha pinned at 255.
        s.ai1 = 19.0;
        let local_before = s.local_ai0;
        let out2 = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert!(!out2.active);
        assert_eq!(s.local_ai0, local_before, "the tail was SKIPPED");
        assert_eq!(s.ai0, 13.0); // unchanged
        // The fade-IN arm: alpha 0 with ai1 ≥ 20 → back to the cycle.
        let mut s3 = state_at_center(0.0, 0.0);
        s3.ai0 = 13.0;
        s3.ai1 = 19.0;
        s3.alpha = 5;
        let env3 = env_night(); // valid target, close, night, not enraged
        let mut rng3 = UnifiedRandom::new(7);
        let out3 = empress_ai_step(&mut s3, &env3, &mut rng3, &[]);
        assert_eq!(s3.alpha, 0);
        assert_eq!(s3.ai0, 1.0);
        assert!(out3.active);
        assert!(out3.net_update);
    }

    #[test]
    fn the_tail_writes_defense_local_ai_and_the_alpha_fade() {
        // `:46919-46941` — phase-2 defense ×1.2 truncated; the 44-tick
        // wing clock; flag5's -5 fade in the attack states.
        let mut s = state_at_center(0.0, 0.0);
        s.ai0 = 2.0;
        s.ai1 = 60.0; // past the dust window, no volley (60 % 3 == 0
        // but ai1 < 60 fails) → a quiet tick
        s.ai3 = 1.0;
        s.alpha = 100;
        s.local_ai0 = 43.0;
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        let out = empress_ai_step(&mut s, &env, &mut rng, &[]);
        assert_eq!(rng, witness, "the quiet tick draws nothing");
        assert_eq!(s.defense, 60); // (50 × 1.2) = 60
        assert_eq!(s.local_ai0, 0.0); // 44 wraps
        assert_eq!(s.alpha, 95); // flag5 fade
        // `:46919` — dontTakeDamage = !flag6; flag6 stays TRUE outside
        // the materialize/dash windows → the quiet tick is VULNERABLE
        // (the C# default; the earlier comment had the polarity wrong).
        assert!(!out.dont_take_damage); // flag6 true → !flag6 = false
        assert_eq!(s.damage, 80); // num16 = 1
    }

    #[test]
    fn the_full_materialize_stream_witness() {
        // The whole 180-tick materialize: ticks 11..149 draw 32 each
        // (139 ticks × 32 = 4448 draws); every other tick draws 0.
        let mut s = state_at_center(0.0, 0.0);
        let env = env_night();
        let mut rng = UnifiedRandom::new(7);
        let mut witness = rng.clone();
        for tick in 0..180 {
            if tick > 10 && tick < 150 {
                for _ in 0..2 {
                    witness.next_max(96);
                    witness.next_max(96);
                    witness.next_range(-20, 21);
                    witness.next_range(-20, 21);
                    witness.next_max(3);
                    witness.next_range(-20, 21);
                    let _ = witness.next_float();
                    let _ = witness.next_float();
                    let _ = witness.next_float();
                    let _ = witness.next_float();
                    witness.next_max(1);
                    witness.next_max(1);
                    witness.next_range(-20, 21);
                    witness.next_range(-20, 21);
                    witness.next_max(3);
                    witness.next_range(-20, 21);
                }
            }
            empress_ai_step(&mut s, &env, &mut rng, &[]);
        }
        assert_eq!(rng, witness, "4448 draws over the materialize");
        assert_eq!(s.ai0, 1.0);
        assert_eq!(s.ai1, 0.0);
    }
}