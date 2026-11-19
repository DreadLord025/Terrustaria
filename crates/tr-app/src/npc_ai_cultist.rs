//! The Lunatic Cultist's machine — `NPC.AI_084_LunaticCultist`
//! (`NPC.cs:64414-65388`), aiStyle 84, `NPCID.CultistBoss` (439) +
//! `NPCID.CultistBossClone` (440) — the census B-3 MAJOR GAP (the live
//! `npc.rs::update_lunatic_cultist` is a 35-line spawn-in stub vs the
//! 976-line C# machine). S1-D-3 cultist, **code-only mode**: pure fns
//! over a local state struct + Ctx/Effects seams, UNWIRED (the
//! `npc_ai_fishron.rs` precedent, ad41a646); the live stub stays
//! byte-identical until the wiring round.
//!
//! Identity census (`ID/NPCID.cs`): `CultistBoss = 439` (:11553),
//! `CultistBossClone = 440` (:11555), `CultistTablet = 437` (:11549) /
//! `CultistDevote = 438` (:11551) are the dungeon entrance NPCs (NOT
//! this machine), `CultistArcherBlue/White = 379/380` (:11435/:11437)
//! ride aiStyle 3, `TaxCollector = 441` (:11691) is style 7 — the
//! task's "clones/archers" guess is corrected: the style-84 family is
//! exactly {439, 440}. The dragon parts `CultistDragonHead..Tail =
//! 454-459` (:11581-11591), `AncientCultistSquidhead = 521` (:11719),
//! `AncientLight = 522` (:11721), `AncientDoom = 523` (:11723) are the
//! machine's summons. Projectiles (`ID/ProjectileID.cs`):
//! `CultistBossIceMist = 464` (:1482), `CultistBossLightningOrb = 465`
//! (:1484), `LightningOrbArc = 466` (:1486), `CultistBossFireBall =
//! 467` (:1488), `FireBallClone = 468` (:1490), `CultistRitual = 490`
//! (:1534).
//!
//! SetDefaults: 439 (`NPC.cs:14212-14229`) 24x50, aiStyle 84, damage
//! 50, defense 42, lifeMax 32000, kb 0, noTileCollide + noGravity,
//! npcSlots 10, value 100000f, boss, netAlways. 440
//! (`NPC.cs:14230-14246`) 24x50, aiStyle 84, damage 0, defense 35,
//! lifeMax 10000, npcSlots 0, chaseable false, netAlways.
//!
//! The machine (one `ai[0]` state, `ai[1]` timer, `ai[3]` attack-cycle
//! counter on the boss / owner-slot on a clone, `ai[2]` the ritual
//! projectile key):
//! * head (`:64416-64552`): the 1/1000 idle-sound gate (RNG!), the
//!   difficulty tunables, the half-hp defense fold, the clone mirror /
//!   orphan despawn / clone `justHit` pop, the boss's ritual-window
//!   `justHit` clone purge;
//! * target gate + despawn (`:64553-64585`) — NO early return: the
//!   despawn arm falls through into the state chain (draw-before-
//!   early-return trap class, f1701eb0);
//! * first-tick latch (`:64587-64598`) + the -1 spawn-in
//!   (`:64599-64637`, 420 ticks: 0-120 hover, 120-300 idle, 300-360
//!   rise at -UnitY, 360-420 the 0.95 damp);
//! * state 0 idle/selector (`:64638-64840`): the 40-tick cadence, the
//!   12-step (classic) / 14-step (expert) attack table, the teleport
//!   ring (num13 == 0), the expert Ancient-Doom roll;
//! * state 1 teleport glide (`:64841-64858`): the odd-tick position
//!   back-step;
//! * state 2 ice mist (`:64859-64927`): one 464 per `num` ticks at
//!   4-speed, the clones' 468 fireballs;
//! * state 3 fireball volley (`:64928-65004`): `num3` 467s per `num2`
//!   ticks, the clones join on shot #2;
//! * state 5 ritual teleport (`:65064-65217`): the fade-out, the
//!   clone-ring summon at tick 30 (the 6-clone cap, the antipodal
//!   slot pick with INTEGER halves), the 490 ritual projectile, the
//!   orientation tracking, the fade-in;
//! * state 6 post-purge pause (`:65218-65230`, 120 ticks);
//! * tail (`:65382-65387`): the clone's `ai[3]` restore,
//!   `dontTakeDamage = flag3`, `chaseable = !flag4`.
//!
//! DEFERRED (documented, exact C# ranges — the wiring round ports
//! them before claiming the dispatch):
//! * state 4 lightning orb (`:65005-65063`): the clones' 468 at
//!   `ai[1] == 20`, the 465 orb at `Center - (0, 100)` with
//!   `GetAttackDamage_ForProjectiles(45, 30)`, the `20 + num4` end
//!   (num4 = 80/40/20); its 3-draws-per-clone fireball RNG is cited
//!   below but NOT consumed (the state is never entered here);
//! * state 7 ancient light (`:65231-65308`): the 5-clone 468 fan
//!   (`RotatedByRandom(1.2566370964050293)`, 3 draws x 5 per clone on
//!   shot #2), the boss's 5-spoke 522 spawn (`NextFloat()` per
//!   spawn, `:65293`, ai1 = `(NextFloat() - 0.5f) * 0.3f * (PI*2f) /
//!   60f`, ai2/ai3 = the 8-speed spoke, Y + 7), the `4 + num5*num6`
//!   end (num5 = 20/30, num6 = 2);
//! * state 8 ancient doom (`:65309-65381`): the `min(count+1, 3)`
//!   523 spawns, each up to 100 attempts of `Next(px-20, px+21)` /
//!   `Next(py-20, py+21)` tile pairs (`:65352-65353`) with the
//!   7-tile player / 3-tile self exclusion boxes, the `!nactive()`
//!   probe and the 2-tile `Collision.SolidTiles` box, the >2000-px
//!   zero-draw gate (`:65345-65348`), the `4 + num7*num8` end
//!   (20 * 3);
//! * the dragon spawn at death is NOT in AI_084: the 490 ritual
//!   projectile's aiStyle-89 arm spawns it (`Projectile.cs:32346-
//!   32356` — at `ai[0] == 300`, 454 if no dragon alive else 521;
//!   self-Kill at 320, `:32357-32361`); the moon-event trigger is
//!   `NPCLoot`'s `DoDeathEvents` case 439 (`NPC.cs:79393-79396`:
//!   `downedAncientCultist` + `WorldGen.TriggerLunarApocalypse()`),
//!   the boss-bag loot is `:78989-78992` (item 499) — all wiring-round
//!   obligations, cited here so the census gap is fully mapped.
//! When `ai[0]` holds a deferred state the step reports it on
//! `CultistOut::deferred_state` and leaves the state untouched (the
//! module is UNWIRED; the selector can still pick those attacks —
//! the wiring must not claim the dispatch until 4/7/8 land).
//!
//! **RNG by construction** — every `Main.rand` draw in the ported
//! scope, in stream order (the port consumes the sound-gate draws
//! even though the sounds themselves are skipped):
//! 1. `:64416` `Next(1000)` — ONLY when `ai[0] != -1` (the `&&`
//!    short-circuit); on a zero roll `:64418` `Next(88, 92)` (the
//!    sound-style pick — a DISCARDED draw here, the sound is skipped
//!    but the stream must advance);
//! 2. `:64761` `Next(maxValue)` — ONLY when `expertMode & flag`
//!    (half hp); drawn BEFORE the `num13 != 0 && num13 != 4 &&
//!    num13 != 5 && CountNPCS(523) < 10` tests, so it is consumed
//!    even when those reject the roll (discarded draw);
//!    maxValue = 6, 4 below `lifeMax / 3`, 3 below `lifeMax / 4`
//!    (INTEGER thirds/quarters, `:64752-64760`);
//! 3. per clone 468 fireball (`:64897`, `:64968`): `NextDouble()`
//!    (the `6f + (float)d * 4f` speed) THEN `RotatedByRandom`'s TWO
//!    `NextDouble()`s (`Utils.cs:1697-1700`, left operand first) —
//!    3 draws per clone, clones in ascending slot order, BEFORE the
//!    boss's own shot;
//! 4. the boss's 467 fireball (`:64990`): the same 3-draw shape;
//!    the boss's 464 ice mist (`:64913-64914`) draws NOTHING (fixed
//!    4-speed, no rotation).
//! The deferred states' draws (4/7/8 above) and the cosmetic
//! HitEffect dust/gore (`:84127-84178`: the death burst's
//! `Next(1, 4)` gore count + `Next(11, 14)` per gore) are NOT
//! consumed — documented skips, the wiring round owns them.
//!
//! Documented deviations (also in `docs/ANALYSIS.md`, S1-D-3 cultist):
//! * sounds (`SoundEngine.PlaySound` 29 at `:64418`/`:64589`/
//!   `:64618`) skipped — but their RNG draws are consumed (above);
//! * `TargetClosest(faceTarget: false)` (`:64558`, `:64642`) rides
//!   the S1-D-3p helper when wired; here the calls are counted on
//!   `CultistOut::target_closest_calls` in C# order and the caller's
//!   target/env ride unchanged (the fishron precedent);
//! * `Main.netMode != 1` is the port's single-player constant true;
//!   the `NetMessage.SendData` calls are skipped (no MP);
//! * `HitEffect()` + `active = false` ride the despawn/kill flags
//!   (the port's death convention `life = 0`); the dust/gore inside
//!   HitEffect is the cosmetic skip;
//! * the dead-store `Vector2` constructions inside the `HasNaNs`
//!   guards (`:64862-64865`, `:64931-64934`) and the throwaway
//!   `new List<int>().Add(whoAmI)` (`:64571`) are annotated, not
//!   ported (pure, no RNG, result unused in the C#);
//! * projectile/NPC spawns are REQUESTS (`CultistEffects`) — the
//!   464/467/468 projectiles and the 440 clones have no port kinds
//!   yet; the 490 ritual key write-back (`this.ai[2] = key`,
//!   `:65133`) is the wiring's job (the request carries the spawn,
//!   the caller stores the returned key); `NPC.NewNPC` places the
//!   440 at `Bottom = (X, Y)` (`NPC.cs:80770`) — the request's Y is
//!   already `(int)center6.Y + height / 2` (INTEGER half, `:65119`).

#![allow(dead_code)] // wiring deferred (S1-D-3 cultist, code-only mode)

use crate::npc_ai_style37::{attack_damage_for_projectiles, rotated_by};
use tr_core::unified_random::UnifiedRandom;

/// `NPC.SetDefaults` type 439 (`NPC.cs:14212-14229`).
pub const CULTIST_WIDTH: i32 = 24;
pub const CULTIST_HEIGHT: i32 = 50;
pub const CULTIST_DEF_DAMAGE: i32 = 50;
pub const CULTIST_DEF_DEFENSE: i32 = 42;
pub const CULTIST_LIFE_MAX: i32 = 32000;

/// `NPC.SetDefaults` type 440 (`NPC.cs:14230-14246`) — same box,
/// damage 0, defense 35, lifeMax 10000, chaseable false.
pub const CLONE_DEF_DEFENSE: i32 = 35;
pub const CLONE_LIFE_MAX: i32 = 10000;

/// The machine's live state — the fields `AI_084` reads/writes on the
/// NPC. One fixed step per call. `npc_type` is 439 (boss) or 440
/// (clone) — the C#'s `flag2` (`:64464`).
#[derive(Clone, Copy, Debug)]
pub struct CultistState {
    /// `this.type` — 439 or 440 (`flag2 = this.type == 439`).
    pub npc_type: i32,
    /// `base.whoAmI` — the clone-owner comparisons (`:64515` etc.).
    pub who_am_i: i32,
    /// `this.ai[0]` — the state (-1 spawn-in, 0 idle, 1 glide,
    /// 2 ice, 3 fire, 4 orb*, 5 ritual, 6 pause, 7 light*, 8 doom*).
    pub ai0: f32,
    /// `this.ai[1]` — the state timer.
    pub ai1: f32,
    /// `this.ai[2]` — the 490 ritual projectile key (boss only).
    pub ai2: f32,
    /// `this.ai[3]` — the attack-cycle counter (boss) / the owner
    /// slot (clone).
    pub ai3: f32,
    /// `this.localAI[0]` — the first-tick latch (`:64587`).
    pub local_ai0: f32,
    /// `this.localAI[1]` — the ritual generation counter (`:65081`).
    pub local_ai1: f32,
    /// `this.localAI[2]` — the FindFrame pose selector
    /// (`NPC.cs:72017-72020`).
    pub local_ai2: f32,
    /// `base.position` (top-left); `Center` derives with FLOAT halves
    /// (`Entity.cs:50-58`).
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub direction: i32,
    pub sprite_direction: i32,
    pub rotation: f32,
    /// `this.alpha` (0 opaque .. 255 invisible) — an INT.
    pub alpha: i32,
    pub life: i32,
    pub life_max: i32,
    pub defense: i32,
    /// `this.defDefense` (the SetDefaults latch).
    pub def_defense: i32,
    /// `this.justHit` — the one-tick strike flag (the wiring maps it).
    pub just_hit: bool,
    /// `this.dontTakeDamage` — the tail write (`:65386`).
    pub dont_take_damage: bool,
    /// `this.chaseable` — the tail write (`:65387`).
    pub chaseable: bool,
}

impl CultistState {
    /// A freshly-spawned boss (the SetDefaults 439 row).
    pub fn boss(who_am_i: i32, x: f32, y: f32) -> Self {
        Self {
            npc_type: 439,
            who_am_i,
            ai0: 0.0,
            ai1: 0.0,
            ai2: 0.0,
            ai3: 0.0,
            local_ai0: 0.0,
            local_ai1: 0.0,
            local_ai2: 0.0,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            sprite_direction: 1,
            rotation: 0.0,
            alpha: 0,
            life: CULTIST_LIFE_MAX,
            life_max: CULTIST_LIFE_MAX,
            defense: CULTIST_DEF_DEFENSE,
            def_defense: CULTIST_DEF_DEFENSE,
            just_hit: false,
            dont_take_damage: false,
            chaseable: true,
        }
    }

    /// `base.Center` (`Entity.cs:54`) — FLOAT halves.
    pub fn center(&self) -> (f32, f32) {
        (
            self.x + CULTIST_WIDTH as f32 / 2.0,
            self.y + CULTIST_HEIGHT as f32 / 2.0,
        )
    }
}

/// One live 440 clone as the boss's slot scans see it
/// (`Main.npc[i].active && type == 440 && ai[3] == whoAmI`). The
/// caller supplies them in ASCENDING slot order (the C# scans
/// `0..Main.maxNPCs`); the step mutates them in place (the teleport
/// ring, the fireball facing) and marks kills `life = 0` +
/// `active = false` (the port's death convention).
#[derive(Clone, Copy, Debug)]
pub struct CultistClone {
    pub who_am_i: i32,
    /// `base.position` (top-left).
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ai0: f32,
    pub ai1: f32,
    /// `localAI[1]` — the ritual generation (`:64528`).
    pub local_ai1: f32,
    pub direction: i32,
    pub sprite_direction: i32,
    pub life: i32,
    pub active: bool,
}

impl CultistClone {
    /// `nPC.Center` — the clone's FLOAT halves (same 24x50 box).
    pub fn center(&self) -> (f32, f32) {
        (
            self.x + CULTIST_WIDTH as f32 / 2.0,
            self.y + CULTIST_HEIGHT as f32 / 2.0,
        )
    }
}

/// The per-tick environment the arm reads from `Main`/the target.
#[derive(Clone, Copy, Debug)]
pub struct CultistEnv {
    /// `Main.expertMode` (`:64420`).
    pub expert: bool,
    /// `Main.getGoodWorld` (`:64428` etc.).
    pub get_good_world: bool,
    /// `this.difficulty` — the `GetAttackDamage_ForProjectiles` remap
    /// base (`NPC.cs:7052-7055`; Classic 1, Expert 2,
    /// `GameDifficultyLevel.cs`).
    pub difficulty: f32,
    /// `this.target` (`:64556`).
    pub target: i32,
    /// `Main.player[this.target]` (`:64554-64562`).
    pub player_active: bool,
    pub player_dead: bool,
    pub player_cx: f32,
    pub player_cy: f32,
    /// `player.velocity` — the 20f aim lead (`:64889` etc.).
    pub player_vx: f32,
    pub player_vy: f32,
    /// The active 490 ritual projectile's Center, resolved by the
    /// caller from the OWNER's `ai[2]` key (`TryGetActive(490)`,
    /// `:65141`/`:65179` — the boss reads its own, a clone reads
    /// `Main.npc[(int)ai[3]].ai[2]`). `None` = no active ritual.
    pub ritual_center: Option<(f32, f32)>,
    /// The boss slot as a CLONE's mirror reads it (`:64473-64481`):
    /// `(active && type == 439, ai[0], ai[1])`. Ignored for the boss.
    pub boss_present: bool,
    pub boss_ai0: f32,
    pub boss_ai1: f32,
    /// `NPC.CountNPCS(523)` (`:64761`, `NPC.cs:89836`) — the Ancient
    /// Doom cap for the expert roll.
    pub ancient_doom_count: i32,
}

/// A projectile spawn request — `Projectile.NewProjectile`
/// (`:64898`/`:64914`/`:64969`/`:64991`). `proj_type` is the vanilla
/// id (464 ice mist / 467 fireball / 468 clone fireball); `ai0`/`ai1`
/// are the trailing spawn args (the 464 carries `0f, 1f`, `:64914`).
#[derive(Clone, Copy, Debug)]
pub struct CultistProjectile {
    pub proj_type: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub ai0: f32,
    pub ai1: f32,
}

/// A 440 clone spawn request (`NPC.NewNPC`, `:65119-65122`) — the
/// position is the C#'s `(int)center6.X, (int)center6.Y + height / 2`
/// (the Bottom convention, `NPC.cs:80770`; `height / 2` is INTEGER
/// division, 50 / 2 = 25).
#[derive(Clone, Copy, Debug)]
pub struct CultistCloneSpawn {
    pub x: i32,
    pub y: i32,
    /// `ai[3] = base.whoAmI` (`:65120`).
    pub owner: f32,
    /// `localAI[1] = this.localAI[1]` (`:65122`).
    pub local_ai1: f32,
}

/// The step's side-effect seam — everything the pure fn cannot do
/// itself. The wiring drains it after the step.
#[derive(Clone, Debug, Default)]
pub struct CultistEffects {
    /// In C# emission order (clones first, then the boss's shot).
    pub projectiles: Vec<CultistProjectile>,
    pub clone_spawns: Vec<CultistCloneSpawn>,
    /// The 490 ritual spawn (`:65132`, `ai1 = whoAmI`); the caller
    /// stores the returned projectile key into the boss's `ai[2]`
    /// (`:65133`).
    pub ritual_spawn: Option<(f32, f32)>,
    /// `proj.ai[1] = -1f` on the ritual-window purge (`:64547-64551`).
    pub ritual_cancel: bool,
    /// A clone's `justHit` pop sets the BOSS's `ai[0] = 6, ai[1] = 0`
    /// (`:64493-64496`).
    pub boss_interrupt: bool,
    /// `whoAmI`s of the clones this step killed (the purge/despawn
    /// arms) — the caller runs their HitEffect/deactivate.
    pub killed_clones: Vec<i32>,
}

/// The step's read-only outputs.
#[derive(Clone, Copy, Debug, Default)]
pub struct CultistOut {
    /// `TargetClosest(faceTarget: false)` calls in C# order
    /// (`:64558` then `:64642`) — 0, 1 or 2; the wiring replays them
    /// through `npc_target::target_closest`.
    pub target_closest_calls: u32,
    /// The despawn arm fired (`:64564-64566` / the clone orphan
    /// `:64475-64477` / the clone pop `:64486-64488`): `life = 0`,
    /// HitEffect, `active = false`. NOTE: only the clone-orphan arm
    /// RETURNS in the C#; the boss despawn falls through.
    pub despawn: bool,
    /// `this.netUpdate = true` was written somewhere this tick.
    pub net_update: bool,
    /// `ai[0]` held a deferred state (4 / 7 / 8) — the state chain
    /// was skipped (see the module doc). The wiring must not claim
    /// the dispatch until those arms land.
    pub deferred_state: Option<f32>,
}

/// `Math.Sign(float)` — -1/0/+1.
fn sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `Vector2.Length()` / `Vector2.Distance` — the f32 sum under the
/// DOUBLE `Math.Sqrt`, cast back (the style55 precedent).
fn length(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA/FNA `Vector2.Normalize()` — `num = 1f / (float)Math.Sqrt(X*X +
/// Y*Y); X *= num; Y *= num`. UNGUARDED (the style55 precedent): a
/// zero vector gives NaN, which the C#'s `HasNaNs` fallbacks then
/// catch (`:64890` etc.). The ritual orientation guards the exact
/// zero BEFORE calling it (`:65145-65148`).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `Vector2.HasNaNs` (`Utils.cs:1709`).
fn has_nans(x: f32, y: f32) -> bool {
    x.is_nan() || y.is_nan()
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`):
/// `RotatedBy(NextDouble() * max - NextDouble() * max)` — TWO draws,
/// left operand first, the double subtraction, then the float-cos/sin
/// spin (`rotated_by`, `Utils.cs:1675-1684`).
fn rotated_by_random(px: f32, py: f32, max_radians: f64, rng: &mut UnifiedRandom) -> (f32, f32) {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    rotated_by(px, py, angle, 0.0, 0.0)
}

/// The shared clone-fireball shot (`:64889-64899`, `:64960-64970`,
/// `:65036-65046`): face the player, aim with the 20f velocity lead,
/// the `(direction * 30, 12f)` offset uses the BOSS's `direction`
/// (`base.direction`, NOT the clone's — kept verbatim), the
/// `6f + (float)NextDouble() * 4f` speed then
/// `RotatedByRandom(0.5235987901687622)` (the float-widened PI/6
/// literal the decompiler shows). 3 RNG draws per call.
fn clone_fireball(
    clone: &mut CultistClone,
    env: &CultistEnv,
    boss_direction: i32,
    fx: &mut CultistEffects,
    rng: &mut UnifiedRandom,
) {
    let (c3x, c3y) = clone.center();
    // `:64882-64886` — Math.Sign facing, zero keeps the old flip.
    let num17 = sign(env.player_cx - c3x);
    if num17 != 0 {
        clone.direction = num17;
        clone.sprite_direction = num17;
    }
    // `:64889` — Normalize(player.Center - center3 + player.velocity * 20f).
    let ax = env.player_cx - c3x + env.player_vx * 20.0;
    let ay = env.player_cy - c3y + env.player_vy * 20.0;
    let (mut nx, mut ny) = normalize_xna(ax, ay);
    if has_nans(nx, ny) {
        // `:64890-64893` — the fallback is the BOSS's direction.
        nx = boss_direction as f32;
        ny = 0.0;
    }
    // `:64894` — base.direction * 30 (the BOSS), +12f.
    let sx = c3x + (boss_direction * 30) as f32;
    let sy = c3y + 12.0;
    // `:64897` — the speed draw FIRST, then the two rotation draws.
    let speed = 6.0f32 + rng.next_double() as f32 * 4.0f32;
    let (vx, vy) = rotated_by_random(nx * speed, ny * speed, 0.5235987901687622, rng);
    // `:64898` — 468, damage 18 (a literal, NOT difficulty-scaled).
    fx.projectiles.push(CultistProjectile {
        proj_type: 468,
        x: sx,
        y: sy,
        vx,
        vy,
        damage: 18,
        ai0: 0.0,
        ai1: 0.0,
    });
}

/// The boss's aimed-shot aim vector (`:64905-64909`, `:64982-64986`):
/// Normalize with the 20f lead, the `(direction, 0)` NaN fallback.
fn boss_aim(s: &CultistState, env: &CultistEnv, cx: f32, cy: f32) -> (f32, f32) {
    let ax = env.player_cx - cx + env.player_vx * 20.0;
    let ay = env.player_cy - cy + env.player_vy * 20.0;
    let (nx, ny) = normalize_xna(ax, ay);
    if has_nans(nx, ny) {
        (s.direction as f32, 0.0)
    } else {
        (nx, ny)
    }
}

/// The ritual-orientation tracking (`:65141-65167` and the identical
/// `:65179-65205`): the vector from the cultist to the ritual
/// projectile, the exact-zero -UnitY guard, Normalize, the
/// `|Y| < 0.77` / `Y < 0` pose ladder, the X-sign facing.
fn ritual_orientation(s: &mut CultistState, ritual_center: Option<(f32, f32)>) {
    let Some((rx, ry)) = ritual_center else {
        return;
    };
    let (cx, cy) = s.center();
    let mut dx = rx - cx;
    let mut dy = ry - cy;
    // `:65145-65148` — the exact-zero guard BEFORE Normalize.
    if dx == 0.0 && dy == 0.0 {
        dx = 0.0;
        dy = -1.0;
    }
    let (nx, ny) = normalize_xna(dx, dy);
    // `:65150-65161` — the pose ladder (0.77f strict).
    if ny.abs() < 0.77 {
        s.local_ai2 = 11.0;
    } else if ny < 0.0 {
        s.local_ai2 = 12.0;
    } else {
        s.local_ai2 = 10.0;
    }
    // `:65162-65166` — the X-sign facing, zero keeps the old flip.
    let num37 = sign(nx);
    if num37 != 0 {
        s.direction = num37;
        s.sprite_direction = num37;
    }
}

/// The whole ported scope of `AI_084_LunaticCultist`
/// (`NPC.cs:64414-65388`, states 4/7/8 deferred — see the module doc).
/// `clones` is the boss's live-440 view in ascending slot order
/// (empty for a clone's own step). One fixed step per call.
pub fn cultist_ai_step(
    s: &mut CultistState,
    env: &CultistEnv,
    clones: &mut [CultistClone],
    fx: &mut CultistEffects,
    rng: &mut UnifiedRandom,
) -> CultistOut {
    let mut out = CultistOut::default();

    // `:64416-64419` — the idle-sound gate. The Next(1000) is drawn
    // ONLY when ai[0] != -1 (&& short-circuit); the sound is skipped
    // but BOTH draws are consumed (the style pick is a discarded
    // draw on a zero roll).
    if s.ai0 != -1.0 {
        if rng.next_max(1000) == 0 {
            // `:64418` — Main.rand.Next(88, 92), the sound-style pick.
            let _style = rng.next_range(88, 92);
        }
    }

    // `:64420-64463` — the tunables.
    let expert_mode = env.expert;
    let flag = s.life <= s.life_max / 2; // INTEGER half (`:64421`).
    let mut num = 120; // the ice-mist period
    let attack_damage_for_projectiles = attack_damage_for_projectiles(env.difficulty, 35.0, 25.0);
    if expert_mode {
        num = 90;
    }
    if env.get_good_world {
        num -= 30;
    }
    let mut num2 = 18; // the fireball period
    let mut num3 = 3; // the fireball count
    let attack_damage_for_projectiles2 = attack_damage_for_projectiles(env.difficulty, 30.0, 20.0);
    if expert_mode {
        num2 = 12;
        num3 = 4;
    }
    if env.get_good_world {
        num2 = 10;
        num3 = 5;
    }
    // num4 (the orb period, `:64445-64454`) and num5/num6 (the light
    // period/count, `:64455-64461`) belong to the deferred states
    // 4/7 — their damage rolls `GetAttackDamage_ForProjectiles(45,
    // 30)` (`:64446`) are cited in the module doc, not computed here
    // (pure, no RNG, no state write).
    let flag2 = s.npc_type == 439;
    let mut flag3 = false; // dontTakeDamage accumulator
    let mut flag4 = false; // !chaseable accumulator

    // `:64467-64470` — the half-hp defense fold (BOTH types).
    if flag {
        s.defense = (s.def_defense as f32 * 0.65f32) as i32;
    }

    // `:64471-64504` — the clone mirror / orphan / pop.
    if !flag2 {
        if s.ai3 < 0.0 || !env.boss_present {
            // `:64473-64479` — the orphan despawn RETURNS (the only
            // early return in the arm; the caller's boss_present
            // folds `active && type == 439`).
            s.life = 0;
            out.despawn = true;
            return out;
        }
        s.ai0 = env.boss_ai0;
        s.ai1 = env.boss_ai1;
        if s.ai0 == 5.0 {
            if s.just_hit {
                // `:64484-64497` — the clone pop: die, interrupt the
                // boss into state 6.
                s.life = 0;
                out.despawn = true;
                fx.boss_interrupt = true;
                out.net_update = true; // the boss's netUpdate (`:64496`)
            }
        } else {
            flag3 = true;
            flag4 = true;
        }
    } else if s.ai0 == 5.0 && s.ai1 >= 120.0 && s.ai1 < 420.0 && s.just_hit {
        // `:64505-64552` — the boss's ritual-window purge.
        s.ai0 = 0.0;
        s.ai1 = 0.0;
        s.ai3 += 1.0;
        s.vx = 0.0;
        s.vy = 0.0;
        out.net_update = true;
        // `:64520-64546` — kill up to num9 clones, the matching
        // generation FIRST in slot order (both arms decrement the
        // SAME budget; the foreach has no break — the tail clones
        // past the budget are simply skipped).
        let mut num9 = 10;
        if env.expert {
            num9 = 3;
        }
        for clone in clones.iter_mut() {
            if clone.local_ai1 == s.local_ai1 && num9 > 0 {
                num9 -= 1;
                clone.life = 0;
                clone.active = false;
                fx.killed_clones.push(clone.who_am_i);
            } else if num9 > 0 {
                num9 -= 1;
                clone.life = 0;
                clone.active = false;
                fx.killed_clones.push(clone.who_am_i);
            }
        }
        // `:64547-64551` — the ritual projectile's ai[1] = -1.
        fx.ritual_cancel = true;
    }

    // `:64553-64561` — the target re-pick gate.
    let (cx, cy) = s.center();
    let num10 = 5600.0f32;
    let dist = length(env.player_cx - cx, env.player_cy - cy);
    if target_invalid(env) || dist > num10 {
        // `:64558` — TargetClosest(faceTarget: false); the player
        // re-read (`:64559`) rides the caller's env unchanged here.
        out.target_closest_calls += 1;
        out.net_update = true;
    }
    // `:64562-64585` — the despawn arm. NO return: the C# falls
    // through into the state chain (the f1701eb0 draw-before-early-
    // return trap class). `new List<int>().Add(whoAmI)` (`:64571`)
    // is a dead store — annotated, not ported.
    if env.player_dead || !env.player_active || dist > num10 {
        s.life = 0;
        out.despawn = true;
        for clone in clones.iter_mut() {
            clone.life = 0;
            clone.active = false;
            fx.killed_clones.push(clone.who_am_i);
        }
    }

    // `:64586` — the clone's ai[3] save (restored at the tail).
    let num11 = s.ai3;

    // `:64587-64598` — the first-tick latch. The sound (29, style 89)
    // is skipped — it draws NO RNG. netMode != 1 is the port's
    // single-player true.
    if s.local_ai0 == 0.0 {
        s.local_ai0 = 1.0;
        s.alpha = 255;
        s.rotation = 0.0;
        s.ai0 = -1.0;
        out.net_update = true;
    }

    // `:64599-64637` — the -1 spawn-in. NOTE: this is a SEPARATE if
    // from the state-0 chain below — the tick the intro completes
    // (ai[1] >= 420 -> ai[0] = 0) the idle block runs the SAME tick.
    if s.ai0 == -1.0 {
        // `:64601-64605` — alpha -= 5 then the < 0 clamp (an INT).
        s.alpha -= 5;
        if s.alpha < 0 {
            s.alpha = 0;
        }
        s.ai1 += 1.0;
        if s.ai1 >= 420.0 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            out.net_update = true;
        } else if s.ai1 > 360.0 {
            // `:64613-64621` — the 0.95 damp + the one-shot sound
            // gate (localAI[2] != 13; the sound draws no RNG).
            s.vx *= 0.95;
            s.vy *= 0.95;
            s.local_ai2 = 13.0;
        } else if s.ai1 > 300.0 {
            // `:64622-64626` — velocity = -Vector2.UnitY.
            s.vx = 0.0;
            s.vy = -1.0;
            s.local_ai2 = 10.0;
        } else if s.ai1 > 120.0 {
            s.local_ai2 = 1.0;
        } else {
            s.local_ai2 = 0.0;
        }
        flag3 = true;
        flag4 = true;
    }

    // `:64638-65381` — the state chain (a NEW if, then else-ifs).
    if s.ai0 == 0.0 {
        // `:64640-64643` — the second TargetClosest at timer zero.
        if s.ai1 == 0.0 {
            out.target_closest_calls += 1;
        }
        s.local_ai2 = 10.0;
        // `:64645-64649` — the X-sign facing.
        let num12 = sign(env.player_cx - cx);
        if num12 != 0 {
            s.direction = num12;
            s.sprite_direction = num12;
        }
        s.ai1 += 1.0;
        // `:64651` — `(ai[1] >= 40f) & flag2` (the non-short-circuit
        // & of two pure bools).
        if s.ai1 >= 40.0 && flag2 {
            // `:64653-64751` — the attack table on (int)ai[3].
            let mut num13;
            if flag {
                // The expert/half-hp table (`:64656-64704`).
                num13 = match s.ai3 as i32 {
                    0 | 2 | 4 | 6 | 8 | 10 | 12 => 0,
                    1 => 1,
                    3 | 7 => 5,
                    5 | 11 => 3,
                    9 => 2,
                    13 => {
                        s.ai3 = -1.0;
                        4
                    }
                    _ => {
                        s.ai3 = -1.0;
                        0
                    }
                };
            } else {
                // The classic table (`:64708-64750`).
                num13 = match s.ai3 as i32 {
                    0 | 2 | 4 | 6 | 8 | 10 => 0,
                    1 | 7 => 1,
                    3 | 9 => 2,
                    5 => 3,
                    11 => {
                        s.ai3 = -1.0;
                        4
                    }
                    _ => {
                        s.ai3 = -1.0;
                        0
                    }
                };
            }
            // `:64752-64764` — the expert Ancient-Doom roll. The
            // Next(maxValue) is drawn BEFORE the num13 tests
            // (a discarded draw when they reject). INTEGER thirds/
            // quarters.
            let mut max_value = 6;
            if s.life < s.life_max / 3 {
                max_value = 4;
            }
            if s.life < s.life_max / 4 {
                max_value = 3;
            }
            if expert_mode && flag && rng.next_max(max_value) == 0 {
                if num13 != 0 && num13 != 4 && num13 != 5 && env.ancient_doom_count < 10 {
                    num13 = 6;
                }
            }
            // `:64765-64808` — the teleport ring (num13 == 0).
            if num13 == 0 {
                // `:64767-64771` — num14 = Ceiling(Length / 50f),
                // the 0 -> 1 floor. The Length is the f32-sum double
                // sqrt; the / 50f is f32; Math.Ceiling is DOUBLE.
                let tx = env.player_cx + 0.0 - cx;
                let ty = env.player_cy + -100.0 - cy;
                let mut num14 = ((length(tx, ty) / 50.0f32) as f64).ceil() as f32;
                if num14 == 0.0 {
                    num14 = 1.0;
                }
                // `:64772-64782` — the ring list: the boss FIRST,
                // then the clones in slot order; flag5 = even count.
                let count = 1 + clones.len();
                let flag5 = count % 2 == 0;
                // `:64783-64807` — one pass over boss + clones.
                for num15 in 0..count {
                    // `:64787` — INTEGER (num15 + flag5 + 1) / 2,
                    // then * (PI*2f) * 0.4f / count, in that order.
                    let half = ((num15 as i32 + flag5 as i32 + 1) / 2) as f32;
                    let mut num16 = half
                        * (std::f64::consts::PI as f32 * 2.0f32)
                        * 0.4f32
                        / count as f32;
                    if num15 % 2 == 1 {
                        num16 *= -1.0;
                    }
                    if count == 1 {
                        num16 = 0.0;
                    }
                    // `:64796` — (0,-1).RotatedBy(num16) * (300,200)
                    // componentwise.
                    let (rx, ry) = rotated_by(0.0, -1.0, num16 as f64, 0.0, 0.0);
                    let ox = rx * 300.0;
                    let oy = ry * 200.0;
                    if num15 == 0 {
                        // The boss's own entry (`list2[0] = whoAmI`).
                        let dx = env.player_cx + ox - cx;
                        let dy = env.player_cy + oy - cy;
                        s.ai0 = 1.0;
                        s.ai1 = num14 * 2.0;
                        s.vx = dx / num14;
                        s.vy = dy / num14;
                        // `:64801-64804` — whoAmI >= whoAmI is TRUE
                        // for the boss: it back-steps too.
                        s.x -= s.vx;
                        s.y -= s.vy;
                        out.net_update = true;
                    } else {
                        let clone = &mut clones[num15 - 1];
                        let (c2x, c2y) = clone.center();
                        let dx = env.player_cx + ox - c2x;
                        let dy = env.player_cy + oy - c2y;
                        clone.ai0 = 1.0;
                        clone.ai1 = num14 * 2.0;
                        clone.vx = dx / num14;
                        clone.vy = dy / num14;
                        if s.who_am_i >= clone.who_am_i {
                            clone.x -= clone.vx;
                            clone.y -= clone.vy;
                        }
                    }
                }
            }
            // `:64809-64837` — the attack hand-off.
            match num13 {
                1 => {
                    s.ai0 = 3.0;
                    s.ai1 = 0.0;
                }
                2 => {
                    s.ai0 = 2.0;
                    s.ai1 = 0.0;
                }
                3 => {
                    s.ai0 = 4.0;
                    s.ai1 = 0.0;
                }
                4 => {
                    s.ai0 = 5.0;
                    s.ai1 = 0.0;
                }
                5 => {
                    s.ai0 = 7.0;
                    s.ai1 = 0.0;
                }
                6 => {
                    s.ai0 = 8.0;
                    s.ai1 = 0.0;
                }
                _ => {}
            }
            out.net_update = true;
        }
    } else if s.ai0 == 1.0 {
        // `:64841-64858` — the teleport glide.
        flag3 = true;
        s.local_ai2 = 10.0;
        // `:64845-64848` — (float)(int)ai[1] % 2f != 0 && ai[1] != 1:
        // the INT cast first, then the f32 remainder.
        if (s.ai1 as i32) as f32 % 2.0 != 0.0 && s.ai1 != 1.0 {
            s.x -= s.vx;
            s.y -= s.vy;
        }
        s.ai1 -= 1.0;
        if s.ai1 <= 0.0 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai3 += 1.0;
            s.vx = 0.0;
            s.vy = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 2.0 {
        // `:64859-64927` — the ice mist.
        s.local_ai2 = 11.0;
        // `:64862-64865` — the HasNaNs dead store (the C# builds a
        // Vector2 and drops it) — pure, no RNG, annotated, skipped.
        // `:64866` — (ai[1] >= 4) & flag2, then the INT (ai[1]-4) % num.
        if s.ai1 >= 4.0 && flag2 && (s.ai1 - 4.0) as i32 % num == 0 {
            // `:64870-64901` — the clones' 468s FIRST (slot order,
            // 3 RNG draws each).
            for clone in clones.iter_mut() {
                clone_fireball(clone, env, s.direction, fx, rng);
            }
            // `:64905-64915` — the boss's 464: NO RNG (fixed 4-speed).
            let (nx, ny) = boss_aim(s, env, cx, cy);
            let sx = cx + (s.direction * 30) as f32;
            let sy = cy + 12.0;
            fx.projectiles.push(CultistProjectile {
                proj_type: 464,
                x: sx,
                y: sy,
                vx: nx * 4.0,
                vy: ny * 4.0,
                damage: attack_damage_for_projectiles,
                ai0: 0.0,
                ai1: 1.0, // the trailing `0f, 1f` spawn args (`:64914`)
            });
        }
        s.ai1 += 1.0;
        // `:64919` — the (float)(4 + num) end.
        if s.ai1 >= (4 + num) as f32 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai3 += 1.0;
            s.vx = 0.0;
            s.vy = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 3.0 {
        // `:64928-65004` — the fireball volley.
        s.local_ai2 = 11.0;
        // `:64931-64934` — the HasNaNs dead store, annotated, skipped.
        if s.ai1 >= 4.0 && flag2 && (s.ai1 - 4.0) as i32 % num2 == 0 {
            // `:64937-64974` — shot #2 (INTEGER division): the clones
            // join with one 468 each.
            if (s.ai1 - 4.0) as i32 / num2 == 2 {
                for clone in clones.iter_mut() {
                    clone_fireball(clone, env, s.direction, fx, rng);
                }
            }
            // `:64975-64979` — the boss faces the player per shot.
            let num21 = sign(env.player_cx - cx);
            if num21 != 0 {
                s.direction = num21;
                s.sprite_direction = num21;
            }
            // `:64982-64992` — the boss's 467 (3 RNG draws).
            let (nx, ny) = boss_aim(s, env, cx, cy);
            let sx = cx + (s.direction * 30) as f32;
            let sy = cy + 12.0;
            let speed = 6.0f32 + rng.next_double() as f32 * 4.0f32;
            let (vx, vy) = rotated_by_random(nx * speed, ny * speed, 0.5235987901687622, rng);
            fx.projectiles.push(CultistProjectile {
                proj_type: 467,
                x: sx,
                y: sy,
                vx,
                vy,
                damage: attack_damage_for_projectiles2,
                ai0: 0.0,
                ai1: 0.0,
            });
        }
        s.ai1 += 1.0;
        // `:64996` — the (float)(4 + num2 * num3) end.
        if s.ai1 >= (4 + num2 * num3) as f32 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai3 += 1.0;
            s.vx = 0.0;
            s.vy = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 5.0 {
        // `:65064-65217` — the ritual teleport.
        s.local_ai2 = 10.0;
        // `:65067-65070` — the HasNaNs dead store (not even assigned
        // in the C#) — annotated, skipped.
        if s.ai1 >= 0.0 && s.ai1 < 30.0 {
            // `:65071-65076` — the fade-out.
            flag3 = true;
            flag4 = true;
            s.alpha = ((s.ai1 - 0.0) / 30.0 * 255.0) as i32;
        } else if s.ai1 >= 30.0 && s.ai1 < 90.0 {
            // `:65079-65137` — the clone-ring summon at tick 30
            // (boss only; netMode != 1 is the port's true).
            if s.ai1 == 30.0 && flag2 {
                s.local_ai1 += 1.0;
                // `:65091-65096` — num27 = 6 - count capped at 2
                // (NO lower clamp — a >6-clone ring keeps num27
                // negative and repositions everything, verbatim).
                let mut num27 = 6 - clones.len() as i32;
                if num27 > 2 {
                    num27 = 2;
                }
                let num28 = clones.len() as i32 + num27 + 1;
                // `:65097-65109` — the distance array + the nearest
                // slot (strict >, first wins ties). The spin angle
                // is ALL f32 (`(float)Math.PI * 2f`, `(float)num28`,
                // `2f` — the explicit casts keep the chain float),
                // widened to double only at the RotatedBy call —
                // same shape as the teleport ring's num16
                // (`:64787`).
                let ring_point = |idx: i32| -> (f32, f32) {
                    // `:65100` — (float)idx * ((float)PI * 2f) /
                    // (float)num28 - (float)PI / 2f.
                    let angle = idx as f32 * (std::f64::consts::PI as f32 * 2.0f32)
                        / num28 as f32
                        - (std::f64::consts::PI as f32) / 2.0f32;
                    let (rx, ry) = rotated_by(180.0, 0.0, angle as f64, 0.0, 0.0);
                    (cx + rx, cy + ry)
                };
                let mut dists = Vec::with_capacity(num28 as usize);
                for idx in 0..num28 {
                    let (px, py) = ring_point(idx);
                    dists.push(length(env.player_cx - px, env.player_cy - py));
                }
                let mut num30 = 0usize;
                for idx in 1..num28 as usize {
                    if dists[num30] > dists[idx] {
                        num30 = idx;
                    }
                }
                // `:65110` — the antipodal flip with INTEGER halves.
                let half = num28 / 2;
                let num30 = if num30 as i32 >= half {
                    num30 as i32 - half
                } else {
                    num30 as i32 + half
                };
                // `:65111-65131` — the placement pass: the first
                // num27 non-antipodal slots SPAWN, the rest
                // REPOSITION the existing clones (`list6[-num32-1]`
                // — the post-decrement index, `num32-- > 0`).
                let mut num32 = num27;
                for idx in 0..num28 {
                    if num30 != idx {
                        let (p6x, p6y) = ring_point(idx);
                        let spawn = num32 > 0;
                        num32 -= 1; // the C#'s post-decrement runs either way
                        if spawn {
                            // `:65119-65122` — NewNPC 440 at
                            // (int)X, (int)Y + height/2 (INTEGER 25).
                            fx.clone_spawns.push(CultistCloneSpawn {
                                x: p6x as i32,
                                y: p6y as i32 + CULTIST_HEIGHT / 2,
                                owner: s.who_am_i as f32,
                                local_ai1: s.local_ai1,
                            });
                        } else {
                            // `:65126-65128` — `list6[-num32 - 1]`
                            // (the POST-decrement value): with a
                            // negative num27 (>6 clones) the first
                            // indices are SKIPPED, verbatim. Center
                            // = ring point (the FLOAT-half setter).
                            let list_idx = (-num32 - 1) as usize;
                            let clone = &mut clones[list_idx];
                            clone.x = p6x - CULTIST_WIDTH as f32 / 2.0;
                            clone.y = p6y - CULTIST_HEIGHT as f32 / 2.0;
                        }
                    }
                }
                // `:65132-65133` — the 490 ritual spawn; the key
                // write-back into ai[2] is the wiring's job.
                fx.ritual_spawn = Some((cx, cy));
                // `:65134` — `base.Center += spinningpoint.RotatedBy(...)`
                // — the Center SETTER subtracts the FLOAT halves, so
                // the net effect is position += the spun offset. The
                // ring_point returns centre + offset; subtract the
                // captured centre to recover the offset.
                let (jx, jy) = ring_point(num30);
                s.x += jx - cx;
                s.y += jy - cy;
                out.net_update = true;
            }
            // `:65138-65167` — the invisible hold + orientation.
            flag3 = true;
            flag4 = true;
            s.alpha = 255;
            ritual_orientation(s, env.ritual_center);
        } else if s.ai1 >= 90.0 && s.ai1 < 120.0 {
            // `:65169-65174` — the fade-in.
            flag3 = true;
            flag4 = true;
            s.alpha = 255 - ((s.ai1 - 90.0) / 30.0 * 255.0) as i32;
        } else if s.ai1 >= 120.0 && s.ai1 < 420.0 {
            // `:65175-65206` — the ritual window (the purge gate at
            // the head reads this band).
            flag4 = true;
            s.alpha = 0;
            ritual_orientation(s, env.ritual_center);
        }
        s.ai1 += 1.0;
        // `:65208-65216` — the 420 end.
        if s.ai1 >= 420.0 {
            flag4 = true;
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai3 += 1.0;
            s.vx = 0.0;
            s.vy = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 6.0 {
        // `:65218-65230` — the post-purge pause.
        s.local_ai2 = 13.0;
        s.ai1 += 1.0;
        if s.ai1 >= 120.0 {
            s.ai0 = 0.0;
            s.ai1 = 0.0;
            s.ai3 += 1.0;
            s.vx = 0.0;
            s.vy = 0.0;
            out.net_update = true;
        }
    } else if s.ai0 == 4.0 || s.ai0 == 7.0 || s.ai0 == 8.0 {
        // The deferred attacks — see the module doc for the exact
        // C# ranges (:65005-65063 / :65231-65308 / :65309-65381)
        // and their RNG ledgers. The state is left untouched.
        out.deferred_state = Some(s.ai0);
    }

    // `:65382-65387` — the tail.
    if !flag2 {
        s.ai3 = num11;
    }
    s.dont_take_damage = flag3;
    s.chaseable = !flag4;
    out
}

/// The target-invalid gate (`:64556`): `target < 0 || target == 255
/// || player.dead || !player.active` (the distance term is checked
/// by the caller of this helper).
fn target_invalid(env: &CultistEnv) -> bool {
    env.target < 0 || env.target == 255 || env.player_dead || !env.player_active
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN NOT RUN (code-only mode: no compiler
    //! invoked on this branch). Each pins one C# behaviour with its
    //! citation; the RNG witnesses replay mirrored `UnifiedRandom`
    //! streams (the style55 precedent).

    use super::*;

    fn env_at(pcx: f32, pcy: f32) -> CultistEnv {
        CultistEnv {
            expert: false,
            get_good_world: false,
            difficulty: 1.0,
            target: 0,
            player_active: true,
            player_dead: false,
            player_cx: pcx,
            player_cy: pcy,
            player_vx: 0.0,
            player_vy: 0.0,
            ritual_center: None,
            boss_present: true,
            boss_ai0: 0.0,
            boss_ai1: 0.0,
            ancient_doom_count: 0,
        }
    }

    fn boss() -> CultistState {
        CultistState::boss(0, 0.0, 0.0)
    }

    /// `:64587-64598` + `:64599-64637` — the first tick latches
    /// localAI[0], alpha 255 -> 250, ai[0] = -1; the intro runs 420
    /// ticks then hands off to idle the SAME tick (the separate-if
    /// trap).
    #[test]
    fn the_spawn_in_intro_latches_and_runs_420_ticks() {
        let mut s = boss();
        let env = env_at(1000.0, 0.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(42);
        let out = cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(s.local_ai0, 1.0);
        assert_eq!(s.ai0, -1.0);
        assert_eq!(s.alpha, 250);
        assert_eq!(s.ai1, 1.0);
        assert!(s.dont_take_damage && !s.chaseable);
        assert!(out.net_update);
        // Tick 1 entered with ai[0] == 0, so the head gate DID draw
        // Next(1000) (`:64416` runs BEFORE the latch sets -1).
        let mut mirror = UnifiedRandom::new(42);
        if mirror.next_max(1000) == 0 {
            let _ = mirror.next_range(88, 92);
        }
        assert_eq!(rng, mirror);
        // Ticks 2..420: ai[0] == -1 at entry — NO draws.
        for _ in 1..420 {
            cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
            assert_eq!(rng, mirror, "the intro draws no RNG");
        }
        // Tick 420: ai[1] hits 420 -> ai[0] = 0, and the idle block
        // runs the same tick (ai[1] becomes 1, not 0).
        assert_eq!(s.ai0, 0.0);
        assert_eq!(s.ai1, 1.0);
        assert_eq!(s.alpha, 0);
    }

    /// `:64416-64419` — outside the intro the Next(1000) is consumed
    /// EVERY tick; a zero roll also consumes Next(88, 92) (the
    /// discarded sound-style draw).
    #[test]
    fn the_idle_sound_gate_consumes_the_cited_draws() {
        let mut s = boss();
        s.local_ai0 = 1.0; // skip the intro latch
        let env = env_at(1000.0, 0.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = UnifiedRandom::new(7);
        for _ in 0..200 {
            s.ai1 = 0.0; // hold the idle timer under 40
            cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
            // The mirror replays the head gate exactly.
            if mirror.next_max(1000) == 0 {
                let _ = mirror.next_range(88, 92);
            }
            assert_eq!(rng, mirror);
        }
    }

    /// `:64421` + `:64467-64470` — the half-hp fold is the INTEGER
    /// half and `(int)(defDefense * 0.65f)`: 42 -> 27.
    #[test]
    fn the_half_hp_defense_fold_truncates() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.life = CULTIST_LIFE_MAX / 2; // exactly half counts (<=)
        let env = env_at(1000.0, 0.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(1);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(s.defense, 27); // (int)(42 * 0.65) = 27
    }

    /// `:64708-64750` — the classic table: ai[3] 1 -> fireball
    /// (ai[0] = 3), 3 -> ice (2), 5 -> orb (4, deferred), 7 -> fire,
    /// 9 -> ice, 11 -> ritual (5) with ai[3] = -1; the teleport
    /// steps (even) keep the ring.
    #[test]
    fn the_classic_attack_table_cycles() {
        let env = env_at(1000.0, 0.0);
        for (step, want_ai0) in [
            (1.0, 3.0),
            (3.0, 2.0),
            (5.0, 4.0),
            (7.0, 3.0),
            (9.0, 2.0),
            (11.0, 5.0),
        ] {
            let mut s = boss();
            s.local_ai0 = 1.0;
            s.ai3 = step;
            s.ai1 = 39.0; // the ++ hits 40
            let mut fx = CultistEffects::default();
            let mut rng = UnifiedRandom::new(3);
            cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
            assert_eq!(s.ai0, want_ai0, "step {step}");
            assert_eq!(s.ai1, 0.0);
            if step == 11.0 {
                assert_eq!(s.ai3, -1.0);
            }
        }
    }

    /// `:64752-64764` — the expert doom roll draws Next(maxValue)
    /// BEFORE the num13 tests: on a teleport step (num13 == 0) the
    /// draw is consumed and DISCARDED. maxValue drops to 4 below
    /// lifeMax/3 and 3 below lifeMax/4 (INTEGER bands).
    #[test]
    fn the_expert_doom_roll_is_a_discarded_draw_on_teleport_steps() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai3 = 0.0; // a teleport step, num13 == 0
        s.ai1 = 39.0;
        s.life = CULTIST_LIFE_MAX / 4 - 1; // maxValue = 3
        let mut env = env_at(1000.0, 0.0);
        env.expert = true;
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = UnifiedRandom::new(11);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        // The head gate, then the doom roll — both consumed.
        if mirror.next_max(1000) == 0 {
            let _ = mirror.next_range(88, 92);
        }
        let _ = mirror.next_max(3);
        assert_eq!(rng, mirror);
        assert_eq!(s.ai0, 1.0, "the teleport still fired (num13 == 0 rejects the doom)");
    }

    /// `:64767-64807` — the solo teleport: num14 =
    /// Ceiling(dist-to-(player-100up) / 50), ai[1] = 2*num14, the
    /// destination is player.Center + (0,-200) (the count==1 ring
    /// vector), the velocity is that delta / num14, and the boss
    /// back-steps one velocity (whoAmI >= whoAmI).
    #[test]
    fn the_solo_teleport_ring_backsteps_the_boss() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai1 = 39.0;
        // Boss centre (12, 25); player (12, -475): the num14 aim
        // point is (12, -575), delta (0, -600), num14 = 12. The RING
        // destination is player + (0,-200) = (12, -675): delta
        // (0, -700), velocity (0, -700/12).
        let env = env_at(12.0, -475.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(5);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(s.ai0, 1.0);
        assert_eq!(s.ai1, 24.0);
        assert_eq!(s.vx, 0.0);
        assert!((s.vy - (-700.0f32 / 12.0)).abs() < 1e-4);
        assert!((s.y - (700.0f32 / 12.0)).abs() < 1e-4, "position -= velocity back-step");
    }

    /// `:64866-64926` — the ice mist: the boss fires ONE 464 at
    /// ai[1] == 4 with the classic damage 35 and NO RNG; the state
    /// ends at 4 + 120 = 124.
    #[test]
    fn the_ice_mist_fires_at_four_and_ends_at_124() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai0 = 2.0;
        s.ai1 = 3.0; // the ++ hits 4 AFTER the fire check — no shot
        let env = env_at(1000.0, 25.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(9);
        let mut mirror = UnifiedRandom::new(9);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        if mirror.next_max(1000) == 0 {
            let _ = mirror.next_range(88, 92);
        }
        assert_eq!(rng, mirror, "the head gate only — the 464 draws nothing");
        assert!(fx.projectiles.is_empty(), "the fire check reads the PRE-increment timer");
        // Next tick ai[1] == 4: the shot fires.
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(fx.projectiles.len(), 1);
        let p = fx.projectiles[0];
        assert_eq!(p.proj_type, 464);
        assert_eq!(p.damage, 35);
        assert_eq!(p.ai1, 1.0);
        assert!((p.vx - 4.0).abs() < 1e-4 && p.vy.abs() < 1e-4);
        // Run to the end: 124 total ticks in the state.
        while s.ai0 == 2.0 {
            cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        }
        assert_eq!(s.ai1, 0.0);
        assert_eq!(s.ai3, 1.0, "the cycle counter advanced");
    }

    /// `:64935-64996` — the fireball volley: 3 shots (num2 = 18,
    /// num3 = 3) at ai[1] = 4, 22, 40; each boss 467 consumes 3
    /// draws; the state ends at 4 + 54 = 58.
    #[test]
    fn the_fireball_volley_consumes_three_draws_per_shot() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai0 = 3.0;
        s.ai1 = 4.0;
        let env = env_at(1000.0, 25.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(13);
        let mut mirror = UnifiedRandom::new(13);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        if mirror.next_max(1000) == 0 {
            let _ = mirror.next_range(88, 92);
        }
        // The shot: NextDouble (speed), then the two rotation draws.
        let _ = mirror.next_double();
        let _ = mirror.next_double();
        let _ = mirror.next_double();
        assert_eq!(rng, mirror);
        assert_eq!(fx.projectiles.len(), 1);
        assert_eq!(fx.projectiles[0].proj_type, 467);
        assert_eq!(fx.projectiles[0].damage, 30);
        let mut shots = 1;
        while s.ai0 == 3.0 {
            let before = fx.projectiles.len();
            cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
            if fx.projectiles.len() > before {
                shots += 1;
            }
        }
        assert_eq!(shots, 3);
    }

    /// `:64897` + `Utils.cs:1697-1700` — a clone fireball is 3 draws
    /// (speed, rot-left, rot-right) and the spawn offset uses the
    /// BOSS's direction, not the clone's.
    #[test]
    fn the_clone_fireball_uses_the_boss_direction_and_three_draws() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai0 = 2.0;
        s.ai1 = 4.0;
        s.direction = -1; // the boss faces LEFT
        let mut clone = CultistClone {
            who_am_i: 1,
            x: 500.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            ai0: 2.0,
            ai1: 4.0,
            local_ai1: 1.0,
            direction: 1,
            sprite_direction: 1,
            life: CLONE_LIFE_MAX,
            active: true,
        };
        let env = env_at(1000.0, 25.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(17);
        let mut mirror = UnifiedRandom::new(17);
        cultist_ai_step(&mut s, &env, std::slice::from_mut(&mut clone), &mut fx, &mut rng);
        if mirror.next_max(1000) == 0 {
            let _ = mirror.next_range(88, 92);
        }
        let _ = mirror.next_double();
        let _ = mirror.next_double();
        let _ = mirror.next_double();
        assert_eq!(rng, mirror, "the clone's 3 draws, then the boss's zero-draw 464");
        assert_eq!(fx.projectiles.len(), 2);
        let clone_shot = fx.projectiles[0];
        assert_eq!(clone_shot.proj_type, 468);
        assert_eq!(clone_shot.damage, 18);
        // The clone centre is (512, 25); the offset is boss.direction
        // * 30 = -30 -> x = 482 (NOT 542).
        assert_eq!(clone_shot.x, 482.0);
        assert_eq!(clone_shot.y, 37.0);
        assert_eq!(fx.projectiles[1].proj_type, 464);
    }

    /// `:65079-65137` — the ritual summon at ai[1] == 30: the
    /// generation increments, 2 clones spawn (num27 cap), the 490
    /// request fires, the boss jumps to the antipodal slot (the
    /// INTEGER-half flip), and the spawn Y carries the INTEGER
    /// height/2 = 25.
    #[test]
    fn the_ritual_summons_two_clones_and_jumps_antipodal() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai0 = 5.0;
        s.ai1 = 30.0;
        let env = env_at(1000.0, 25.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(19);
        let mut mirror = UnifiedRandom::new(19);
        let (old_x, old_y) = (s.x, s.y);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        if mirror.next_max(1000) == 0 {
            let _ = mirror.next_range(88, 92);
        }
        assert_eq!(rng, mirror, "the ritual summon draws NO RNG");
        assert_eq!(s.local_ai1, 1.0);
        assert_eq!(fx.clone_spawns.len(), 2, "num27 = 6 - 0 capped at 2");
        assert!(fx.ritual_spawn.is_some());
        for spawn in &fx.clone_spawns {
            assert_eq!(spawn.owner, 0.0);
            assert_eq!(spawn.local_ai1, 1.0);
        }
        // num28 = 3; the player at +X makes idx 1 the nearest slot
        // (angle PI/6 -> the +X side); 1 >= num28/2 (INTEGER 1) ->
        // the antipodal flip SUBTRACTS: num30 = 0 (the -Y slot).
        // The boss jumped by that slot's spun (180, 0) offset.
        let moved = s.x != old_x || s.y != old_y;
        assert!(moved, "the boss jumped to the antipodal ring slot");
        assert!(s.y < old_y, "slot 0 spins (180,0) by -PI/2 -> (0,-180)");
        assert_eq!(s.alpha, 255);
        assert!(s.dont_take_damage && !s.chaseable);
    }

    /// `:64505-64552` — the ritual-window purge: justHit in
    /// [120, 420) resets the boss, advances ai[3], kills up to 10
    /// clones (classic) and cancels the ritual.
    #[test]
    fn the_ritual_window_purge_kills_clones_and_cancels() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.local_ai1 = 2.0;
        s.ai0 = 5.0;
        s.ai1 = 200.0;
        s.just_hit = true;
        let mut clones: Vec<CultistClone> = (0..3)
            .map(|i| CultistClone {
                who_am_i: i + 1,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                ai0: 5.0,
                ai1: 200.0,
                local_ai1: if i == 2 { 1.0 } else { 2.0 },
                direction: 1,
                sprite_direction: 1,
                life: CLONE_LIFE_MAX,
                active: true,
            })
            .collect();
        let env = env_at(1000.0, 25.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(23);
        cultist_ai_step(&mut s, &env, &mut clones, &mut fx, &mut rng);
        assert_eq!(s.ai0, 0.0);
        assert_eq!(s.ai3, 1.0);
        assert!(fx.ritual_cancel);
        assert_eq!(fx.killed_clones.len(), 3);
        assert!(clones.iter().all(|c| !c.active && c.life == 0));
    }

    /// `:64471-64504` — a clone mirrors the boss's ai[0]/ai[1]; an
    /// orphan clone despawns and RETURNS (no tail writes); a hit
    /// clone in state 5 pops and interrupts the boss.
    #[test]
    fn the_clone_mirror_orphan_and_pop() {
        let mut s = boss();
        s.npc_type = 440;
        s.def_defense = CLONE_DEF_DEFENSE;
        s.life = CLONE_LIFE_MAX;
        s.life_max = CLONE_LIFE_MAX;
        s.local_ai0 = 1.0;
        s.ai3 = 0.0; // the owner slot
        let mut env = env_at(1000.0, 25.0);
        env.boss_ai0 = 3.0;
        env.boss_ai1 = 17.0;
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(29);
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(s.ai0, 3.0);
        assert_eq!(s.ai3, 0.0, "the tail restored the owner slot");
        assert!(s.dont_take_damage && !s.chaseable, "the non-5 mirror sets both flags");
        // The pop.
        env.boss_ai0 = 5.0;
        s.just_hit = true;
        let mut fx = CultistEffects::default();
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(s.life, 0);
        assert!(fx.boss_interrupt);
        // The orphan: no boss -> despawn + early return.
        let mut s2 = s;
        s2.life = CLONE_LIFE_MAX;
        env.boss_present = false;
        let mut fx = CultistEffects::default();
        let out = cultist_ai_step(&mut s2, &env, &mut [], &mut fx, &mut rng);
        assert!(out.despawn);
        assert_eq!(s2.life, 0);
    }

    /// `:64562-64585` — the dead-player despawn kills the clones and
    /// does NOT return: the state chain still runs (the fall-through
    /// trap).
    #[test]
    fn the_despawn_arm_falls_through() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai0 = 6.0;
        s.ai1 = 119.0;
        let mut env = env_at(1000.0, 25.0);
        env.player_dead = true;
        let mut clone = CultistClone {
            who_am_i: 1,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            ai0: 6.0,
            ai1: 119.0,
            local_ai1: 1.0,
            direction: 1,
            sprite_direction: 1,
            life: CLONE_LIFE_MAX,
            active: true,
        };
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(31);
        let out = cultist_ai_step(&mut s, &env, std::slice::from_mut(&mut clone), &mut fx, &mut rng);
        assert!(out.despawn);
        assert_eq!(s.life, 0);
        assert!(!clone.active);
        // The state-6 arm still ran: 119 + 1 >= 120 -> the reset.
        assert_eq!(s.ai0, 0.0);
        assert_eq!(s.ai3, 1.0);
    }

    /// `:65218-65230` — the post-purge pause runs 120 ticks then
    /// advances the cycle.
    #[test]
    fn the_post_purge_pause_runs_120_ticks() {
        let mut s = boss();
        s.local_ai0 = 1.0;
        s.ai0 = 6.0;
        let env = env_at(1000.0, 25.0);
        let mut fx = CultistEffects::default();
        let mut rng = UnifiedRandom::new(37);
        for _ in 0..119 {
            cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
            assert_eq!(s.ai0, 6.0);
        }
        cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
        assert_eq!(s.ai0, 0.0);
        assert_eq!(s.ai3, 1.0);
        assert_eq!(s.local_ai2, 13.0);
    }

    /// The deferred states (4/7/8) are reported, not silently run.
    #[test]
    fn the_deferred_states_are_reported() {
        for state in [4.0f32, 7.0, 8.0] {
            let mut s = boss();
            s.local_ai0 = 1.0;
            s.ai0 = state;
            let env = env_at(1000.0, 25.0);
            let mut fx = CultistEffects::default();
            let mut rng = UnifiedRandom::new(41);
            let out = cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
            assert_eq!(out.deferred_state, Some(state));
            assert_eq!(s.ai1, 0.0, "the deferred state is untouched");
        }
    }

    /// The whole scripted run replays byte-identically on a mirrored
    /// stream (the RNG-by-construction witness).
    #[test]
    fn the_scripted_run_replays_identically() {
        let run = |seed: i32| {
            let mut s = boss();
            let env = env_at(600.0, -200.0);
            let mut fx = CultistEffects::default();
            let mut rng = UnifiedRandom::new(seed);
            let mut log = Vec::new();
            for _ in 0..1200 {
                let out = cultist_ai_step(&mut s, &env, &mut [], &mut fx, &mut rng);
                log.push((s.ai0, s.ai1, s.ai3, s.x, s.y, s.alpha, out.target_closest_calls));
            }
            (log, rng, fx.projectiles.len(), fx.clone_spawns.len())
        };
        let a = run(101);
        let b = run(101);
        assert_eq!(a.0, b.0);
        assert_eq!(a.1, b.1, "the streams end in the same state");
        assert_eq!(a.2, b.2);
        assert_eq!(a.3, b.3);
    }
}
