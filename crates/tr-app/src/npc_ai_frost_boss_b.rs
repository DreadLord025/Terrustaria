//! Frost Moon boss AI — S1-F-13c slice C, part 2 (this round:
//! **Everscream 344 + SantaNK1 346**). The IceQueen (345) machine lives
//! in the sibling module `npc_ai_frost_boss.rs` (`task/S1-F-13c-boss-ai`,
//! commit 41bc275f); this file is deliberately DISJOINT — same house
//! pattern (a plain state struct + a type-keyed step), no shared items,
//! the merger unions the seams.
//!
//! The C# arms ported here, keyed by the vanilla `NPC.type` / `aiStyle`
//! (the `SetDefaults` rows):
//!
//! * **Everscream (344, aiStyle 57)** — the `NPC.cs:13355-13368` row
//!   (172×130, damage 110, defense 38, lifeMax 13000). aiStyle 57 is
//!   SHARED with the pumpkin MourningWood (325); the machine below ports
//!   the whole `aiStyle == 57` arm (`NPC.cs:32761-33043`) but runs ONLY
//!   the `type == 344` branches — the type-325 arms (and the untyped
//!   `else` arms, which are the 325 paths at runtime) belong to
//!   S1-F-13b2's `npc_ai_pumpkin_boss.rs` (982d3caf) and stay untouched
//!   there. The Everscream-specific differences: the idle attack pick
//!   is ALWAYS `Next(1, 3)` (the `type != 344` guard at `:32806` kills
//!   the low-hp `Next(3, 5)` arm), phase 1 is the pine-needle spray
//!   (345, every 5 ticks, 180-tick phase, the per-tick 245-dust arm) and
//!   phase 2 the ornament lob (346, every 15 ticks strictly between 60
//!   and 240, 300-tick phase). Phases 3/4 are unreachable for 344 (the
//!   idle pick only ever yields 1/2) and are not ported.
//! * **SantaNK1 (346, aiStyle 61)** — the `NPC.cs:13386-13399` row
//!   (112×140, damage 120, defense 56, lifeMax 18000); the whole
//!   `aiStyle == 61` arm (`NPC.cs:33681-33921`): the 300-tick idle →
//!   240-tick bullet volley (180 BulletDeadeye) cycle gated on the
//!   `flag64` target-live-and-close check, the three independent
//!   attack rolls (600/1200/2700, hp-scaled) — the 352 Spike
//!   (`Next(600) == 0`), the 350 Missile salvo (`localAI[1]`, the
//!   `Next(1200)` trigger, 12-tick cadence, 100-tick window) and the
//!   351 Present rain (`localAI[2]`, the `Next(2700)` trigger, 9-tick
//!   cadence) — and the shared floater movement tail.
//!
//! **Deferred (docs/ANALYSIS.md, "Everscream + SantaNK1 core AI"):**
//! the `FindFrame` arms, `GetBossHeadRotation`, the death/bag/drop arms,
//! the `Lighting.AddLight` lines (`:32781`, `:33709` — no rand) and the
//! `netUpdate` writes (no rand; the port is single-player). The
//! Everscream dust arm IS ported (it rolls `Main.rand` — see below), so
//! unlike S1-F-13b2's MourningWood port nothing rand-bearing is skipped.
//!
//! **Integration seam / dependency:** the `NpcKind` variants + the full
//! `SetDefaults` rows for 344/346 are owned by the parallel defaults
//! workers (`task/S1-F-13c-defaults-a/b`). Until they land this module
//! runs on the plain [`FrostBossBState`] (the AI-visible subset of
//! `NPC`'s fields — the `npc_ai_frost_boss.rs` `FrostBossState` shape
//! widened with `direction`, `local_ai` and `time_left`, which these
//! two arms read/write) and the dispatch ([`frost_boss_b_ai_step`])
//! keys on the vanilla type ids + the C# `aiStyle` constants. The
//! wiring worker copies `Npc`'s fields in/out and runs
//! `npc_target::target_closest` where the C# calls `TargetClosest()`
//! (`:32768`, `:33688`, `:33725`); the resolved slot rides
//! [`FrostBossBTarget`] (Santa's `flag64` reads `active`/`dead` too).
//! `Main.netMode != 1` (`:32803`, `:33723`, `:33769`) is the port's
//! always-true server/local path. Both bosses set
//! `noGravity`/`noTileCollide` INSIDE the arm (`:32764-32765`,
//! `:33684-33685`) — the caller integrates the velocity directly.
//!
//! **RNG order (by construction — every `Main.rand.Next` in the arms):**
//!
//! Everscream (aiStyle 57, type-344 paths):
//! * idle (`ai[0] == 0`): the pick at `ai[1] >= 300` draws `Next(1, 3)`
//!   (`:32812`) — ALWAYS, even below 25% hp (the `:32806`
//!   `this.type != 344` guard short-circuits the `Next(3, 5)` arm).
//! * phase 1 (`ai[0] == 1`): EVERY tick `Next(5)` (`:32821`); when it
//!   returns 0 the dust arm draws `NextVector2Square(0, 1)` = 2 ×
//!   `NextDouble` (`Utils.cs:1253`) then `Dust.NewDust` rolls the pool
//!   (on an empty pool: `Next(6)`, `Next(10)`, `Next(-20,21)` ×2,
//!   `Next(3)`, `Next(-20,21)` — `Dust.cs:170-190`; the off-screen /
//!   pressure-band refusals draw LESS — the pool port is verbatim).
//!   Then, when `ai[1] % 5 == 0`: origin `Next(width-40)` then
//!   `Next(height-40)` (`:32830` — X then Y), aim `Next(-50,51)` then
//!   `Next(-50,51)` then `Next(0,21)` (`:32833-32834` — the Y-jitter
//!   BEFORE the |dx| slope roll), then the `NewProjectile` named args
//!   evaluate in TEXTUAL order: SpeedX jitter `Next(-20,21)`, SpeedY
//!   jitter `Next(-20,21)`, …, `ai0: Next(0,31)` LAST (`:32838`).
//! * phase 2 (`ai[0] == 2`): when `60 < ai[1] < 240 && ai[1] % 15 == 0`:
//!   origin `Next(width-40)` then `Next(height-80)` (`:32876`), aim
//!   `Next(-50,51)` then `Next(50,201)` (`:32880-32881`), SpeedX/SpeedY
//!   jitters `Next(-30,31)` ×2 then `ai1: Next(2)` LAST (`:32885`).
//!   `Type: 346` is a constant — no kind draw (unlike the 325 arms).
//! * the day flee and the movement tail draw NOTHING.
//!
//! SantaNK1 (aiStyle 61), per tick in textual order:
//! * the idle/volley state machine draws NOTHING (`:33720-33767`); the
//!   volley shot, when `flag64` and the hp-tiered cadence fires:
//!   origin-Y `Next(15,36)` (`:33752`), aim `Next(-40,41)` ×2
//!   (`:33755-33756`), SpeedX/SpeedY jitters `Next(-20,21)` ×2
//!   (`:33760`; `Type: 180` constant, no ai0/ai1 args).
//! * then the attack block (`Main.netMode != 1`, `:33769-33847`) —
//!   these draws fire EVERY tick regardless of `ai[0]` (even by day):
//!   `Next(num927)` (`:33792`; 600 hp-scaled) — on 0 the 352 Spike
//!   draws `Next(1,100)` (`:33795`); then `Next(num928)` (`:33800`;
//!   1200) — on 0 `localAI[1] = 1`; when `localAI[1] >= 1` it increments
//!   and, if `flag64` and `% 12 == 0`, the 350 Missile draws aim
//!   `Next(-50,51)` ×2 (`:33813-33814`) then SpeedX/SpeedY jitters
//!   `Next(-20,21)` ×2 (`:33818`); then `Next(num929)` (`:33825`;
//!   2700) — on 0 `localAI[2] = 2`; when `localAI[2] > 0` it increments
//!   and, if `% 9 == 0`, the 351 Present draws `Next(-100,101)`
//!   (`:33835`) then SpeedX/SpeedY jitters `Next(-20,21)` ×2 (`:33840`).
//!   The Missile/Present cadence draws are gated on `flag64` /
//!   `localAI[2] > 0` respectively — the trigger rolls themselves are
//!   unconditional.
//! * the movement tail draws NOTHING.
//!
//! The witness tests pin each order against a same-seeded probe stream.

// The wiring lands with the defaults workers; until then the dispatch is
// exercised by this module's tests only.
#![allow(dead_code)]

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `NPCID.Everscream` — `NPCID.cs:11365`.
pub const EVERSCREAM_TYPE: u16 = 344;
/// The C#'s `this.aiStyle` for 344 — `NPC.cs:13359` (shared with the
/// pumpkin MourningWood 325; the type-344 branches are this module's).
pub const EVERSCREAM_AI_STYLE: u8 = 57;
/// `NPCID.SantaNK1` — `NPCID.cs:11369`.
pub const SANTA_NK1_TYPE: u16 = 346;
/// The C#'s `this.aiStyle` for 346 — `NPC.cs:13390`.
pub const SANTA_NK1_AI_STYLE: u8 = 61;

/// The AI-relevant `SetDefaults` inputs `(width, height, lifeMax)` —
/// `NPC.cs:13357-13362` (172×130, 13000). The full stat row (damage
/// 110/defense 38/value/knockBackResist 0/npcSlots 3) is the defaults
/// worker's table.
pub fn everscream_ai_defaults() -> (f32, f32, i32) {
    (172.0, 130.0, 13000)
}

/// The AI-relevant `SetDefaults` inputs `(width, height, lifeMax)` —
/// `NPC.cs:13388-13393` (112×140, 18000). The full stat row (damage
/// 120/defense 56/value/knockBackResist 0/npcSlots 4) is the defaults
/// worker's table.
pub fn santa_nk1_ai_defaults() -> (f32, f32, i32) {
    (112.0, 140.0, 18000)
}

/// The AI-visible subset of `NPC`'s fields the Everscream/SantaNK1
/// machines read/write — the sibling `npc_ai_frost_boss.rs`
/// `FrostBossState` shape widened with `direction` (`base.direction`,
/// the floater approach), `local_ai` (Santa's attack windows) and
/// `time_left` (`EncourageDespawn`, `NPC.cs:7249-7256`). The wiring
/// worker copies these in/out of `Npc`.
#[derive(Clone, Copy, Debug)]
pub struct FrostBossBState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `NPC.ai[0..3]`.
    pub ai: [f32; 4],
    /// `NPC.localAI[0..3]` — Santa's `localAI[1]` (Missile window) and
    /// `localAI[2]` (Present window); Everscream reads none.
    pub local_ai: [f32; 4],
    /// `Entity.direction` — set by `TargetClosest` (the wiring seam).
    pub direction: i32,
    pub rotation: f32,
    /// `NPC.life`/`lifeMax`.
    pub life: i32,
    pub life_max: i32,
    /// `Entity.width`/`height` (the `SetDefaults` box; the C# fields are
    /// ints — the `(float)base.width` casts are exact).
    pub width: f32,
    pub height: f32,
    /// `NPC.timeLeft` — `EncourageDespawn` only ever LOWERS it
    /// (`NPC.cs:7251-7254`); the field default is `NPC.activeTime = 750`
    /// (`NPC.cs:95186`, written at `:8542`).
    pub time_left: i32,
    /// `NPC.despawnEncouraged` (`NPC.cs:7255`) — written by the day
    /// arms; no rand impact.
    pub despawn_encouraged: bool,
}

impl FrostBossBState {
    /// A fresh Everscream at (x, y) with the `SetDefaults` AI inputs and
    /// the C#'s field defaults (`ai`/`localAI` zeroed, `timeLeft` 750).
    pub fn everscream(x: f32, y: f32) -> Self {
        let (width, height, life_max) = everscream_ai_defaults();
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            ai: [0.0; 4],
            local_ai: [0.0; 4],
            direction: 1,
            rotation: 0.0,
            life: life_max,
            life_max,
            width,
            height,
            time_left: 750,
            despawn_encouraged: false,
        }
    }

    /// A fresh SantaNK1 at (x, y) with the `SetDefaults` AI inputs.
    pub fn santa_nk1(x: f32, y: f32) -> Self {
        let (width, height, life_max) = santa_nk1_ai_defaults();
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            ai: [0.0; 4],
            local_ai: [0.0; 4],
            direction: 1,
            rotation: 0.0,
            life: life_max,
            life_max,
            width,
            height,
            time_left: 750,
            despawn_encouraged: false,
        }
    }

    /// `Entity.Center` (`Entity.cs:50-55`): position + size/2. The
    /// position never moves inside `AI()` (the integrate happens
    /// after), so one read per tick is exact.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// `NPC.EncourageDespawn` (`NPC.cs:7249-7256`): only ever LOWERS
    /// the `timeLeft` window; `despawnEncouraged = true`.
    fn encourage_despawn(&mut self, despawn_time: i32) {
        if self.time_left > despawn_time {
            self.time_left = despawn_time;
        }
        self.despawn_encouraged = true;
    }
}

/// The resolved `Main.player[this.target]` box. `TargetClosest()` call
/// sites: `NPC.cs:32768` (Everscream, night only), `:33688` (Santa,
/// night only), `:33725` (Santa's idle→volley flip). The wiring worker
/// runs `npc_target::target_closest` at those points and passes the
/// picked slot here. Santa's `flag64` (`:33692`) reads `active`/`dead`.
#[derive(Clone, Copy, Debug)]
pub struct FrostBossBTarget {
    /// `Player.position.X`/`Y`.
    pub x: f32,
    pub y: f32,
    /// `Player.width`/`height` (the 20×42 box).
    pub width: f32,
    pub height: f32,
    /// `Player.active` (`flag64`, `NPC.cs:33692`).
    pub active: bool,
    /// `Player.dead` (`flag64`).
    pub dead: bool,
}

impl FrostBossBTarget {
    /// `Player.Center` (`Entity.cs:50-55`).
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// A hostile-shot request — `Projectile.NewProjectile`'s
/// `(X, Y, SpeedX, SpeedY, Type, Damage, KnockBack, ai0, ai1)` payload.
/// KnockBack is 0f at every call site ported here, so it is not carried.
/// The non-zero extras: Everscream's pine-needle `ai0 = Next(0,31)`
/// (`:32838`) and ornament `ai1 = Next(2)` (`:32885`). The C# draws no
/// rand inside `NewProjectile`/`NewProjectileSetup`/`SetDefaults` for
/// the types spawned here (180/345/346/350/351/352 — the slot scan's
/// `DebugOptions.Shared_RandomizeProjectileSlots` arm is default-off,
/// `Projectile.cs:10831`; the per-type rand switch `:10905-10925`
/// covers 206/335/358/406 only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrostBossBShot {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub ai0: f32,
    pub ai1: f32,
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`) — Santa's
/// light gate (`NPC.cs:33707`): active, solid, not solid-top, not
/// half-brick, slopeless and not actuated off. An out-of-world index
/// throws in the C# and the empty `catch` returns false
/// (`:70128-70131`); an in-world null tile returns TRUE (`:70115-70117`)
/// but in-world tiles are never null in the port, so `None` ⇒ false is
/// exact (the sibling module's convention).
fn worldgen_solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Collision.SolidCollision` (`Collision.cs:2700-2734`) through the
/// port's verbatim implementation (`player.rs::solid_collision`, the
/// zombie/pumpkin wiring's shared probe). The floater hover tail reads
/// it; no rand.
fn solid_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    crate::player::Player::solid_collision(world, x, y, w, h)
}

/// The shared floater movement tail — the aiStyle-57 block's
/// `:32970-33042` and the aiStyle-61 block's `:33848-33920` are
/// BYTE-IDENTICAL in the C# (the damp flag, the `(vx*20 ± speed)/21`
/// approach, the 80×20 solid probe, the fall/rise ramps, the 10f cap).
/// `flag` is the arm's accumulated damp flag (`flag61`/`flag63`);
/// `speed` is `num836`/`num921`. No rand draws.
fn floater_movement(
    s: &mut FrostBossBState,
    world: &World,
    target: &FrostBossBTarget,
    mut flag: bool,
    speed: f32,
) {
    let (cx, _cy) = s.center();
    let (pcx, _pcy) = target.center();
    // `:32970-32973` / `:33848-33851` — damp above the player.
    if (cx - pcx).abs() < 50.0 {
        flag = true;
    }
    // `:32974-32992` / `:33852-33870` — the damp or the approach.
    if flag {
        s.vx *= 0.9;
        if (s.vx as f64) > -0.1 && (s.vx as f64) < 0.1 {
            s.vx = 0.0;
        }
    } else {
        if s.direction > 0 {
            s.vx = (s.vx * 20.0 + speed) / 21.0;
        }
        if s.direction < 0 {
            s.vx = (s.vx * 20.0 - speed) / 21.0;
        }
    }
    // `:32993-33000` / `:33871-33878` — the 80×20 probe and the
    // player-overlap drop (the C#'s int div `num865 / 2` = 40).
    let num865 = 80;
    let num866 = 20;
    let probe_x = cx - (num865 / 2) as f32;
    let probe_y = s.y + s.height - num866 as f32;
    let over_player = s.x < target.x
        && s.x + s.width > target.x + target.width
        && s.y + s.height < target.y + target.height - 16.0;
    // `:33001-33038` / `:33879-33916` — the vertical ramps.
    if over_player {
        s.vy += 0.5;
    } else if solid_collision(world, probe_x, probe_y, num865 as f32, num866 as f32) {
        if s.vy > 0.0 {
            s.vy = 0.0;
        }
        if (s.vy as f64) > -0.2 {
            s.vy -= 0.025;
        } else {
            s.vy -= 0.2;
        }
        if s.vy < -4.0 {
            s.vy = -4.0;
        }
    } else {
        if s.vy < 0.0 {
            s.vy = 0.0;
        }
        if (s.vy as f64) < 0.1 {
            s.vy += 0.025;
        } else {
            s.vy += 0.5;
        }
    }
    // `:33039-33042` / `:33917-33920`.
    if s.vy > 10.0 {
        s.vy = 10.0;
    }
}

/// **Everscream (344)** — the `aiStyle == 57` arm's type-344 branches
/// (`NPC.cs:32761-33043`), verbatim. The shared movement tail is
/// [`floater_movement`]; the type-325 arms stay in S1-F-13b2's module.
/// `TargetClosest()` (`:32768`) is the caller's seam. The dust arm
/// rolls the caller's `Main.rand` through the verbatim
/// `DustPool::new_dust` (`Dust.cs:75-256`) — `dust_ctx` carries the
/// `Main` statics it reads (the wiring worker fills them; the tests use
/// an on-screen empty-pool ctx).
#[allow(clippy::too_many_arguments)]
pub fn everscream_ai(
    s: &mut FrostBossBState,
    world: &World,
    target: &FrostBossBTarget,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostBossBShot>,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    // `:32763-32765` — `num836 = 2f`; the gravity/tile flags off (the
    // caller integrates the raw velocity).
    let mut num836 = 2.0f32;
    // `:32766-32769` — retarget only at night (the caller's seam).
    let mut flag61 = false;
    // `:32771-32778` — the hp bands (the C#'s double compares).
    if (s.life as f64) < (s.life_max as f64) * 0.75 {
        num836 = 3.0;
    }
    if (s.life as f64) < (s.life_max as f64) * 0.5 {
        num836 = 4.0;
    }
    // `:32779-32782` — the type-344 `Lighting.AddLight` is skipped
    // (module docs; no rand).
    if world.day_time {
        // `:32787-32791` — dawn: the despawn window and the flee speed.
        s.encourage_despawn(10);
        num836 = 8.0;
    } else if s.ai[0] == 0.0 {
        // `:32792-32816` — the idle: the counter runs faster the lower
        // the hp; at 300 the next attack is picked. `Main.netMode != 1`
        // is the port's single-player truth.
        s.ai[1] += 1.0;
        if (s.life as f64) < (s.life_max as f64) * 0.5 {
            s.ai[1] += 1.0;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.25 {
            s.ai[1] += 1.0;
        }
        if s.ai[1] >= 300.0 {
            s.ai[1] = 0.0;
            // `:32806-32813` — type 344 FAILS the `this.type != 344`
            // guard, so the pick is ALWAYS Next(1, 3) — even below 25%
            // hp (the Next(3, 5) arm is the 325-only path).
            s.ai[0] = rng.next_range(1, 3) as f32;
            // `:32814` — netUpdate (no rand; single-player port).
        }
    } else if s.ai[0] == 1.0 {
        // `:32819-32845` — the type-344 pine-needle spray.
        // `:32821-32825` — the per-tick dust roll: Next(5) EVERY tick;
        // on 0 the 245-dust spawn (NextVector2Square = 2 NextDouble,
        // Utils.cs:1253, then the verbatim NewDust pool draws).
        if rng.next_max(5) == 0 {
            // `base.position + NextVector2Square(0, 1) * base.Size -
            // new Vector2(1, 2)` — the C# evaluates the square's X
            // draw before its Y draw.
            let sq_x = (1.0f32 - 0.0) * rng.next_float() + 0.0;
            let sq_y = (1.0f32 - 0.0) * rng.next_float() + 0.0;
            let pos = Vec2::new(s.x + sq_x * s.width - 1.0, s.y + sq_y * s.height - 2.0);
            let num837 = dust.new_dust(
                rng,
                dust_ctx,
                world,
                pos,
                10,
                14,
                245,
                0.0,
                0.0,
                254,
                Color::TRANSPARENT,
                0.25,
            );
            // `:32824` — the velocity damp; the sentinel slot exists
            // (`Main.dust[6000]`) and the C# writes it too.
            dust.slot_mut(num837).velocity *= 0.2;
        }
        flag61 = true;
        s.ai[1] += 1.0;
        if s.ai[1] % 5.0 == 0.0 {
            // `:32830` — the origin: X + 20 + Next(width - 40), Y + 20
            // + Next(height - 40) — X drawn first.
            let ox = s.x + 20.0 + rng.next_max(s.width as i32 - 40) as f32;
            let oy = s.y + 20.0 + rng.next_max(s.height as i32 - 40) as f32;
            // `:32831-32834` — the aim at the player's centre-X / TOP-Y
            // (`position.Y`, not the centre), the X jitter, then the Y
            // jitter MINUS the |dx| slope roll (the Y-jitter draws
            // BEFORE the Next(0, 21)).
            let (pcx, _pcy) = target.center();
            let mut num838 = pcx - ox;
            let mut num839 = target.y - oy;
            num838 += rng.next_range(-50, 51) as f32;
            num839 = num839 + rng.next_range(-50, 51) as f32
                - num838.abs() * (rng.next_range(0, 21) as f32 * 0.01);
            // `:32835-32837` — the 12.5f normalisation.
            let num840 = 12.5 / (num838 * num838 + num839 * num839).sqrt();
            num838 *= num840;
            num839 *= num840;
            // `:32838` — the named args evaluate in TEXTUAL order:
            // SpeedX jitter, SpeedY jitter, …, ai0 LAST. Type 345
            // (PineNeedleHostile, `ProjectileID.cs:1244`), Damage 43,
            // KnockBack 0f.
            let sx = num838 * (1.0 + rng.next_range(-20, 21) as f32 * 0.02);
            let sy = num839 * (1.0 + rng.next_range(-20, 21) as f32 * 0.02);
            let ai0 = rng.next_range(0, 31) as f32;
            shots.push(FrostBossBShot {
                kind: 345,
                x: ox,
                y: oy,
                vx: sx,
                vy: sy,
                damage: 43,
                ai0,
                ai1: 0.0,
            });
        }
        // `:32840-32844` — the 180-tick phase end.
        if s.ai[1] >= 180.0 {
            s.ai[1] = 0.0;
            s.ai[0] = 0.0;
        }
    } else if s.ai[0] == 2.0 {
        // `:32869-32892` — the type-344 ornament lob.
        flag61 = true;
        s.ai[1] += 1.0;
        if s.ai[1] > 60.0 && s.ai[1] < 240.0 && s.ai[1] % 15.0 == 0.0 {
            // `:32875-32876` — the 4.5f base speed and the origin:
            // X + 20 + Next(width - 40), Y + 60 + Next(height - 80).
            let mut num846 = 4.5f32;
            let ox = s.x + 20.0 + rng.next_max(s.width as i32 - 40) as f32;
            let oy = s.y + 60.0 + rng.next_max(s.height as i32 - 80) as f32;
            // `:32877-32879` — the aim at the player's centre-X / TOP-Y
            // with the |dx|*0.3 lift and the |dx|*0.004 speed gain
            // (NO cap — unlike the 325 arm's 14f).
            let (pcx, _pcy) = target.center();
            let mut num847 = pcx - ox;
            let mut num848 = target.y - oy - num847.abs() * 0.3;
            num846 += num847.abs() * 0.004;
            // `:32880-32881` — the aim jitters: X then the DOWNWARD
            // Next(50, 201) lob.
            num847 += rng.next_range(-50, 51) as f32;
            num848 -= rng.next_range(50, 201) as f32;
            // `:32882-32884`.
            let num849 = num846 / (num847 * num847 + num848 * num848).sqrt();
            num847 *= num849;
            num848 *= num849;
            // `:32885` — the textual-order draws: SpeedX jitter, SpeedY
            // jitter, …, ai1 LAST. Type 346 (OrnamentHostile,
            // `ProjectileID.cs:1246`) is a CONSTANT — no kind draw;
            // Damage 57, KnockBack 0f, ai0 0f.
            let sx = num847 * (1.0 + rng.next_range(-30, 31) as f32 * 0.01);
            let sy = num848 * (1.0 + rng.next_range(-30, 31) as f32 * 0.01);
            let ai1 = rng.next_max(2) as f32;
            shots.push(FrostBossBShot {
                kind: 346,
                x: ox,
                y: oy,
                vx: sx,
                vy: sy,
                damage: 57,
                ai0: 0.0,
                ai1,
            });
        }
        // `:32887-32891` — the 300-tick phase end.
        if s.ai[1] >= 300.0 {
            s.ai[1] = 0.0;
            s.ai[0] = 0.0;
        }
    }
    // Phases 3/4 (`:32922-32969`) are the 325-only paths — the 344 idle
    // pick only ever yields 1/2, so they are unreachable here and stay
    // unported (module docs).
    // `:32970-33042` — the shared floater movement tail.
    floater_movement(s, world, target, flag61, num836);
}

/// **SantaNK1 (346)** — the `aiStyle == 61` arm (`NPC.cs:33681-33921`),
/// verbatim. `TargetClosest()` sites (`:33688` night, `:33725` the
/// idle→volley flip) are the caller's seam. The three attack rolls fire
/// EVERY tick (the `Main.netMode != 1` block is the port's always-true
/// path) — even by day and in the idle state.
pub fn santa_nk1_ai(
    s: &mut FrostBossBState,
    world: &World,
    target: &FrostBossBTarget,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostBossBShot>,
) {
    // `:33683-33685` — `num921 = 2f`; the gravity/tile flags off.
    let mut num921 = 2.0f32;
    // `:33686-33689` — retarget only at night (the caller's seam).
    let mut flag63 = false;
    // `:33691-33692` — flag64: the target live and within 2000px
    // (`Entity.Distance` = `Vector2.Distance(Center, other)`,
    // `Entity.cs:195-198` — the float sqrt).
    let (cx, cy) = s.center();
    let (pcx, pcy) = target.center();
    let dx = cx - pcx;
    let dy = cy - pcy;
    let flag64 = target.active && !target.dead && (dx * dx + dy * dy).sqrt() <= 2000.0;
    // `:33693-33704` — the hp bands (four tiers here, unlike 57's two).
    if (s.life as f64) < (s.life_max as f64) * 0.75 {
        num921 = 3.0;
    }
    if (s.life as f64) < (s.life_max as f64) * 0.5 {
        num921 = 4.0;
    }
    if (s.life as f64) < (s.life_max as f64) * 0.25 {
        num921 = 5.0;
    }
    // `:33705-33710` — the centre light: `InWorld` + `!SolidTile` gate
    // (`WorldGen.cs:9688-9695`, `:70111-70132`). The AddLight itself is
    // skipped (module docs; no rand) but the probe is kept for the 1:1
    // shape. `ToTileCoordinates` = `(int)x >> 4` (`Utils.cs:1807-1810`
    // — the arithmetic shift AFTER the truncation).
    let tx = (cx as i32) >> 4;
    let ty = (cy as i32) >> 4;
    let _light_on = tx >= 0
        && tx < world.width
        && ty >= 0
        && ty < world.height
        && !worldgen_solid_tile(world, tx, ty);
    if world.day_time {
        // `:33711-33719` — dawn: the despawn window, the flee speed and
        // the 0.1f X nudge (no rand).
        s.encourage_despawn(10);
        num921 = 8.0;
        if s.vx == 0.0 {
            s.vx = 0.1;
        }
    } else if s.ai[0] == 0.0 {
        // `:33720-33730` — the idle: 300 ticks, then retarget + the
        // volley state. `Main.netMode != 1` is the port's truth.
        s.ai[1] += 1.0;
        if s.ai[1] >= 300.0 {
            // `:33725` — TargetClosest() (the caller's seam).
            s.ai[1] = 0.0;
            s.ai[0] = 1.0;
            // `:33728` — netUpdate (no rand).
        }
    } else if s.ai[0] == 1.0 {
        // `:33731-33768` — the bullet volley.
        s.ai[1] += 1.0;
        flag63 = true;
        if flag64 {
            // `:33737-33749` — the hp-tiered cadence (the else-if chain
            // picks the LOWEST matching tier first).
            let mut num922 = 16;
            if (s.life as f64) < (s.life_max as f64) * 0.25 {
                num922 = 8;
            } else if (s.life as f64) < (s.life_max as f64) * 0.5 {
                num922 = 11;
            } else if (s.life as f64) < (s.life_max as f64) * 0.75 {
                num922 = 14;
            }
            if s.ai[1] % num922 as f32 == 0.0 {
                // `:33752` — the origin: Centre.X + direction*50,
                // Centre.Y + Next(15, 36) (the Y draw FIRST — it sits
                // inside the Vector2 ctor).
                let ox = cx + (s.direction * 50) as f32;
                let oy = cy + rng.next_range(15, 36) as f32;
                // `:33753-33756` — the aim at the player's centre-X /
                // centre-Y (Santa aims at the CENTRE, unlike the Wood),
                // then the two ±40 jitters.
                let mut num923 = pcx - ox;
                let mut num924 = pcy - oy;
                num923 += rng.next_range(-40, 41) as f32;
                num924 += rng.next_range(-40, 41) as f32;
                // `:33757-33759` — the 15f normalisation.
                let num925 = 15.0 / (num923 * num923 + num924 * num924).sqrt();
                num923 *= num925;
                num924 *= num925;
                // `:33760` — SpeedX/SpeedY jitters (×0.015); Type 180
                // (BulletDeadeye, `ProjectileID.cs:914`) constant,
                // Damage 36, no ai0/ai1 args.
                let sx = num923 * (1.0 + rng.next_range(-20, 21) as f32 * 0.015);
                let sy = num924 * (1.0 + rng.next_range(-20, 21) as f32 * 0.015);
                shots.push(FrostBossBShot {
                    kind: 180,
                    x: ox,
                    y: oy,
                    vx: sx,
                    vy: sy,
                    damage: 36,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
        }
        // `:33763-33767` — the 240-tick volley end (strictly >).
        if s.ai[1] > 240.0 {
            s.ai[0] = 0.0;
            s.ai[1] = 0.0;
        }
    }
    // `:33769-33847` — the three independent attack rolls, EVERY tick
    // (`Main.netMode != 1` — the port's always-true path).
    {
        // `:33771-33791` — the hp-scaled denominators (the else-if
        // chain; the (int)(double * k) truncations are exact for these
        // values: 600/1200/2700 → ×0.5 = 300/600/1350, ×0.75 =
        // 450/900/2025, ×0.9 = 540/1080/2430).
        let mut num927 = 600;
        let mut num928 = 1200;
        let mut num929 = 2700;
        if (s.life as f64) < (s.life_max as f64) * 0.25 {
            num927 = (num927 as f64 * 0.5) as i32;
            num928 = (num928 as f64 * 0.5) as i32;
            num929 = (num929 as f64 * 0.5) as i32;
        } else if (s.life as f64) < (s.life_max as f64) * 0.5 {
            num927 = (num927 as f64 * 0.75) as i32;
            num928 = (num928 as f64 * 0.75) as i32;
            num929 = (num929 as f64 * 0.75) as i32;
        } else if (s.life as f64) < (s.life_max as f64) * 0.75 {
            num927 = (num927 as f64 * 0.9) as i32;
            num928 = (num928 as f64 * 0.9) as i32;
            num929 = (num929 as f64 * 0.9) as i32;
        }
        // `:33792-33799` — the Spike roll (drawn unconditionally).
        if rng.next_max(num927) == 0 {
            // `:33794-33798` — the origin BEHIND the boss (−direction*24,
            // −64), the Next(1, 100)*direction X speed, the 1f Y, the
            // normalisation. Type 352 (Spike, `ProjectileID.cs:1258`),
            // Damage 80, KnockBack 0f, no ai args.
            let ox = cx - (s.direction * 24) as f32;
            let oy = cy - 64.0;
            let num930 = rng.next_range(1, 100) as f32 * s.direction as f32;
            let num931 = 1.0f32;
            let num932 = 1.0 / (num930 * num930 + num931 * num931).sqrt();
            shots.push(FrostBossBShot {
                kind: 352,
                x: ox,
                y: oy,
                vx: num930 * num932,
                vy: num931 * num932,
                damage: 80,
                ai0: 0.0,
                ai1: 0.0,
            });
        }
        // `:33800-33803` — the Missile-window trigger (drawn
        // unconditionally).
        if rng.next_max(num928) == 0 {
            s.local_ai[1] = 1.0;
        }
        // `:33804-33824` — the Missile salvo window.
        if s.local_ai[1] >= 1.0 {
            s.local_ai[1] += 1.0;
            let num934 = 12;
            if flag64 && s.local_ai[1] % num934 as f32 == 0.0 {
                // `:33810-33818` — the same behind-origin, the centre
                // aim + ±50 jitters, the 12.5f normalisation, the ×0.015
                // speed jitters. Type 350 (Missile,
                // `ProjectileID.cs:1254`), Damage 42.
                let ox = cx - (s.direction * 24) as f32;
                let oy = cy - 64.0;
                let mut num935 = pcx - ox;
                let mut num936 = pcy - oy;
                num935 += rng.next_range(-50, 51) as f32;
                num936 += rng.next_range(-50, 51) as f32;
                let num937 = 12.5 / (num935 * num935 + num936 * num936).sqrt();
                num935 *= num937;
                num936 *= num937;
                let sx = num935 * (1.0 + rng.next_range(-20, 21) as f32 * 0.015);
                let sy = num936 * (1.0 + rng.next_range(-20, 21) as f32 * 0.015);
                shots.push(FrostBossBShot {
                    kind: 350,
                    x: ox,
                    y: oy,
                    vx: sx,
                    vy: sy,
                    damage: 42,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
            if s.local_ai[1] >= 100.0 {
                s.local_ai[1] = 0.0;
            }
        }
        // `:33825-33828` — the Present-window trigger (drawn
        // unconditionally).
        if rng.next_max(num929) == 0 {
            s.local_ai[2] = 2.0;
        }
        // `:33829-33846` — the Present rain window (NOT gated on
        // flag64 — the C# rains presents at a dead/far target too).
        if s.local_ai[2] > 0.0 {
            s.local_ai[2] += 1.0;
            if s.local_ai[2] % 9.0 == 0.0 {
                // `:33834-33840` — the behind-origin, the Next(-100,101)
                // X aim against the fixed −300f Y, the 11f
                // normalisation, the ×0.01 speed jitters. Type 351
                // (Present, `ProjectileID.cs:1256`), Damage 50.
                let ox = cx - (s.direction * 24) as f32;
                let oy = cy - 64.0;
                let mut num939 = rng.next_range(-100, 101) as f32;
                let mut num940 = -300.0f32;
                let num941 = 11.0 / (num939 * num939 + num940 * num940).sqrt();
                num939 *= num941;
                num940 *= num941;
                let sx = num939 * (1.0 + rng.next_range(-20, 21) as f32 * 0.01);
                let sy = num940 * (1.0 + rng.next_range(-20, 21) as f32 * 0.01);
                shots.push(FrostBossBShot {
                    kind: 351,
                    x: ox,
                    y: oy,
                    vx: sx,
                    vy: sy,
                    damage: 50,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
            if s.local_ai[2] >= 100.0 {
                s.local_ai[2] = 0.0;
            }
        }
    }
    // `:33848-33920` — the shared floater movement tail.
    floater_movement(s, world, target, flag63, num921);
}

/// The dispatch seam — claims **344 and 346 only** (the IceQueen 345 is
/// the sibling module's `frost_boss_ai_step`; the non-boss Frost set is
/// `npc_ai_frost_mob.rs`'s, worker D). Keyed on the vanilla type ids;
/// the machines are the C# `aiStyle` 57/61 arms ([`EVERSCREAM_AI_STYLE`]
/// / [`SANTA_NK1_AI_STYLE`], `NPC.cs:13359`/`:13390`). Returns `false`
/// for every other id. `rng` rides the caller's `Main.rand` stream in
/// the module-header order; the witness tests pin it.
#[allow(clippy::too_many_arguments)]
pub fn frost_boss_b_ai_step(
    type_id: u16,
    s: &mut FrostBossBState,
    world: &World,
    target: &FrostBossBTarget,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostBossBShot>,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) -> bool {
    match type_id {
        EVERSCREAM_TYPE => {
            everscream_ai(s, world, target, rng, shots, dust, dust_ctx);
            true
        }
        SANTA_NK1_TYPE => {
            santa_nk1_ai(s, world, target, rng, shots);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        let mut w = World::new("t".into(), 200, 120);
        w.day_time = false;
        w
    }

    fn target(x: f32, y: f32) -> FrostBossBTarget {
        FrostBossBTarget {
            x,
            y,
            width: 20.0,
            height: 42.0,
            active: true,
            dead: false,
        }
    }

    /// An on-screen empty-pool ctx: the dust spawn always lands in slot
    /// 0 (the low pressure band draws nothing), so the NewDust draw
    /// count is the fixed 6 (position ×2, velocity ×2, frame row,
    /// scale — `Dust.cs:170-190`; type 245 hits no tweak arm).
    fn dust_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::new(0.0, 0.0),
            screen_size: Vec2::new(4000.0, 4000.0),
            max_dust_to_draw: 1000,
        }
    }

    // ---- Everscream (NPC.cs:32761-33043, the type-344 branches) ----

    /// `:32806-32813` — the idle pick is ALWAYS Next(1, 3) for 344,
    /// even below 25% hp (the `type != 344` guard kills Next(3, 5));
    /// the counter runs 3× at <25% hp (`:32794-32802`).
    #[test]
    fn everscream_idle_pick_is_always_one_or_two() {
        let w = air();
        let t = target(1100.0, 700.0);
        for frac in [1.0f32, 0.6, 0.4, 0.2] {
            let mut s = FrostBossBState::everscream(1000.0, 600.0);
            s.life = (s.life_max as f32 * frac) as i32;
            s.ai[1] = 299.0;
            let mut rng = UnifiedRandom::new(7);
            let mut probe = UnifiedRandom::new(7);
            let mut shots = Vec::new();
            let mut dust = DustPool::new();
            everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
            let pick = probe.next_range(1, 3);
            assert!((1..=2).contains(&pick));
            assert_eq!(s.ai[0], pick as f32, "frac {frac}");
            assert_eq!(s.ai[1], 0.0);
            // Exactly ONE draw this tick; the streams stay aligned.
            for _ in 0..8 {
                assert_eq!(rng.next(), probe.next(), "frac {frac}");
            }
        }
        // The counter speed: full hp +1, <50% +2, <25% +3.
        for (frac, step) in [(1.0f32, 1.0f32), (0.4, 2.0), (0.2, 3.0)] {
            let mut s = FrostBossBState::everscream(1000.0, 600.0);
            s.life = (s.life_max as f32 * frac) as i32;
            let mut rng = UnifiedRandom::new(1);
            let mut shots = Vec::new();
            let mut dust = DustPool::new();
            everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
            assert_eq!(s.ai[1], step, "frac {frac}");
        }
    }

    /// `:32821-32838` — the pine-needle spray: the per-tick Next(5)
    /// dust gate (drawn even when the shot cadence does not fire), the
    /// origin draws, the aim jitters (Y-jitter BEFORE the slope roll),
    /// the 12.5f normalisation, the ×0.02 speed jitters, ai0 LAST.
    #[test]
    fn everscream_needle_spray_draw_order() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossBState::everscream(1000.0, 600.0);
        s.ai[0] = 1.0;
        s.ai[1] = 4.0; // 5 % 5 == 0 fires
        let mut rng = UnifiedRandom::new(31);
        let mut probe = UnifiedRandom::new(31);
        let mut shots = Vec::new();
        let mut dust = DustPool::new();
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        // The dust gate draw fires FIRST, every tick.
        let gate = probe.next_max(5);
        if gate == 0 {
            // The square's two NextDouble draws, then the pool's six.
            let _sq_x = probe.next_float();
            let _sq_y = probe.next_float();
            let _px = probe.next_max(6);
            let _py = probe.next_max(10);
            let _vx = probe.next_range(-20, 21);
            let _vy = probe.next_range(-20, 21);
            let _row = probe.next_max(3);
            let _scale = probe.next_range(-20, 21);
        }
        // The shot: origin X then Y, aim X/Y jitters, slope roll, speed
        // jitters, ai0 LAST.
        let ox = 1000.0f32 + 20.0 + probe.next_max(172 - 40) as f32;
        let oy = 600.0f32 + 20.0 + probe.next_max(130 - 40) as f32;
        let mut ax = 1110.0f32 - ox;
        let mut ay = 700.0f32 - oy;
        ax += probe.next_range(-50, 51) as f32;
        ay = ay + probe.next_range(-50, 51) as f32 - ax.abs() * (probe.next_range(0, 21) as f32 * 0.01);
        let k = 12.5 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        let sx = ax * (1.0 + probe.next_range(-20, 21) as f32 * 0.02);
        let sy = ay * (1.0 + probe.next_range(-20, 21) as f32 * 0.02);
        let ai0 = probe.next_range(0, 31) as f32;
        assert_eq!(shots.len(), 1);
        assert_eq!(
            shots[0],
            FrostBossBShot { kind: 345, x: ox, y: oy, vx: sx, vy: sy, damage: 43, ai0, ai1: 0.0 }
        );
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // A non-cadence tick STILL draws the dust gate (and nothing
        // else when the gate refuses).
        let mut s2 = FrostBossBState::everscream(1000.0, 600.0);
        s2.ai[0] = 1.0;
        s2.ai[1] = 5.0; // 6 % 5 != 0
        let mut rng2 = UnifiedRandom::new(31);
        let mut probe2 = UnifiedRandom::new(31);
        let mut shots2 = Vec::new();
        let mut dust2 = DustPool::new();
        everscream_ai(&mut s2, &w, &t, &mut rng2, &mut shots2, &mut dust2, &dust_ctx());
        let gate2 = probe2.next_max(5);
        if gate2 == 0 {
            let _ = probe2.next_float();
            let _ = probe2.next_float();
            let _ = probe2.next_max(6);
            let _ = probe2.next_max(10);
            let _ = probe2.next_range(-20, 21);
            let _ = probe2.next_range(-20, 21);
            let _ = probe2.next_max(3);
            let _ = probe2.next_range(-20, 21);
        }
        assert!(shots2.is_empty());
        for _ in 0..8 {
            assert_eq!(rng2.next(), probe2.next());
        }
    }

    /// `:32821-32825` — a forced dust spawn (gate draw == 0): the slot
    /// lands, type 245, alpha 254, scale ×0.25 then the ×0.2 velocity
    /// damp; the position rides the square draws + the 10×14 box.
    #[test]
    fn everscream_dust_spawn_writes_the_slot() {
        let w = air();
        let t = target(1100.0, 700.0);
        // Find a seed whose FIRST Next(5) is 0.
        let seed = (0..1000)
            .find(|&sd| UnifiedRandom::new(sd).next_max(5) == 0)
            .expect("a Next(5)==0 seed exists");
        let mut s = FrostBossBState::everscream(1000.0, 600.0);
        s.ai[0] = 1.0;
        s.ai[1] = 1.0; // 2 % 5 != 0 — only the dust arm draws
        let mut rng = UnifiedRandom::new(seed);
        let mut probe = UnifiedRandom::new(seed);
        let mut shots = Vec::new();
        let mut dust = DustPool::new();
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(probe.next_max(5), 0);
        let sq_x = probe.next_float();
        let sq_y = probe.next_float();
        let px = probe.next_max(6);
        let py = probe.next_max(10);
        let dvx = probe.next_range(-20, 21) as f32 * 0.1;
        let dvy = probe.next_range(-20, 21) as f32 * 0.1;
        let row = probe.next_max(3);
        let scale_j = probe.next_range(-20, 21) as f32 * 0.01;
        assert!(shots.is_empty());
        let d = dust.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 245);
        assert_eq!(d.alpha, 254);
        assert_eq!(d.position.x, 1000.0 + sq_x * 172.0 - 1.0 + px as f32 + 4.0);
        assert_eq!(d.position.y, 600.0 + sq_y * 130.0 - 2.0 + py as f32 + 4.0);
        // The ×0.2 damp applies AFTER NewDust's own velocity writes.
        assert_eq!(d.velocity.x, dvx * 0.2);
        assert_eq!(d.velocity.y, dvy * 0.2);
        assert_eq!(d.frame_y, 10 * row + 60); // 245 → two per-100 shifts
        assert_eq!(d.scale, (1.0 + scale_j) * 0.25);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// `:32873-32885` — the ornament lob: the 60<ai[1]<240 window, the
    /// %15 cadence, the origin (+20/+60 offsets), the |dx|*0.3 lift and
    /// |dx|*0.004 gain (NO cap), the −Next(50,201) lob, the ×0.01 speed
    /// jitters, ai1 LAST; Type 346 constant (no kind draw).
    #[test]
    fn everscream_ornament_lob_draw_order() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossBState::everscream(1000.0, 600.0);
        s.ai[0] = 2.0;
        s.ai[1] = 74.0; // 75: >60, <240, %15==0
        let mut rng = UnifiedRandom::new(2024);
        let mut probe = UnifiedRandom::new(2024);
        let mut shots = Vec::new();
        let mut dust = DustPool::new();
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        let ox = 1000.0f32 + 20.0 + probe.next_max(172 - 40) as f32;
        let oy = 600.0f32 + 60.0 + probe.next_max(130 - 80) as f32;
        let mut ax = 1110.0f32 - ox;
        let mut ay = 700.0f32 - oy - ax.abs() * 0.3;
        let speed = 4.5f32 + ax.abs() * 0.004;
        ax += probe.next_range(-50, 51) as f32;
        ay -= probe.next_range(50, 201) as f32;
        let k = speed / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        let sx = ax * (1.0 + probe.next_range(-30, 31) as f32 * 0.01);
        let sy = ay * (1.0 + probe.next_range(-30, 31) as f32 * 0.01);
        let ai1 = probe.next_max(2) as f32;
        assert_eq!(shots.len(), 1);
        assert_eq!(
            shots[0],
            FrostBossBShot { kind: 346, x: ox, y: oy, vx: sx, vy: sy, damage: 57, ai0: 0.0, ai1 }
        );
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // Outside the cadence (ai[1] 60 → 61: >60 but 61 % 15 != 0):
        // no shot, NO draws at all in phase 2.
        let mut s2 = FrostBossBState::everscream(1000.0, 600.0);
        s2.ai[0] = 2.0;
        s2.ai[1] = 60.0; // 61: %15 != 0
        let mut rng2 = UnifiedRandom::new(5);
        let mut probe2 = UnifiedRandom::new(5);
        let mut shots2 = Vec::new();
        let mut dust2 = DustPool::new();
        everscream_ai(&mut s2, &w, &t, &mut rng2, &mut shots2, &mut dust2, &dust_ctx());
        assert!(shots2.is_empty());
        for _ in 0..8 {
            assert_eq!(rng2.next(), probe2.next());
        }
        // The phase ends: ai[1] 299 → 300 >= 300 resets both.
        let mut s3 = FrostBossBState::everscream(1000.0, 600.0);
        s3.ai[0] = 2.0;
        s3.ai[1] = 299.0;
        let mut rng3 = UnifiedRandom::new(5);
        let mut shots3 = Vec::new();
        let mut dust3 = DustPool::new();
        everscream_ai(&mut s3, &w, &t, &mut rng3, &mut shots3, &mut dust3, &dust_ctx());
        assert_eq!((s3.ai[0], s3.ai[1]), (0.0, 0.0));
    }

    /// `:32787-32791` — the day flee: EncourageDespawn(10) lowers
    /// timeLeft, speed 8, ZERO draws; the movement tail still runs.
    #[test]
    fn everscream_day_flee_draws_nothing() {
        let mut w = air();
        w.day_time = true;
        let t = target(5000.0, 700.0); // far — no damp
        let mut s = FrostBossBState::everscream(1000.0, 600.0);
        s.direction = 1;
        let mut rng = UnifiedRandom::new(8);
        let mut probe = UnifiedRandom::new(8);
        let mut shots = Vec::new();
        let mut dust = DustPool::new();
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(s.time_left, 10);
        assert!(s.despawn_encouraged);
        assert_eq!(s.vx, 8.0 / 21.0);
        assert!(shots.is_empty());
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // EncourageDespawn only LOWERS: a second call keeps 10.
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(s.time_left, 10);
    }

    /// `:32970-33042` — the movement tail: the <50px damp (with the
    /// ±0.1 snap-to-zero), the (vx*20 ± num836)/21 approach, the
    /// over-player drop and the 10f fall cap.
    #[test]
    fn everscream_movement_damp_and_approach() {
        let w = air();
        // Damp: the centres within 50px. Everscream centre = x+86;
        // player centre = x+10 → place the player at 1076 (centre 1086).
        let t = target(1076.0, 2000.0);
        let mut s = FrostBossBState::everscream(1000.0, 600.0);
        s.ai[0] = 5.0; // an unmatched phase: only the tail runs
        s.vx = 3.0;
        let mut rng = UnifiedRandom::new(1);
        let mut shots = Vec::new();
        let mut dust = DustPool::new();
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(s.vx, 3.0 * 0.9);
        // The snap: 0.05 * 0.9 = 0.045 < 0.1 → 0.
        s.vx = 0.05;
        everscream_ai(&mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(s.vx, 0.0);
        // Approach: far player, direction −1, full hp speed 2.
        let t2 = target(5000.0, 2000.0);
        let mut s2 = FrostBossBState::everscream(1000.0, 600.0);
        s2.ai[0] = 5.0;
        s2.direction = -1;
        everscream_ai(&mut s2, &w, &t2, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(s2.vx, -2.0 / 21.0);
        // The fall cap: no solid probe, vy already 9.8 → +0.5 → cap 10.
        let mut s3 = FrostBossBState::everscream(1000.0, 600.0);
        s3.ai[0] = 5.0;
        s3.vy = 9.8;
        everscream_ai(&mut s3, &w, &t2, &mut rng, &mut shots, &mut dust, &dust_ctx());
        assert_eq!(s3.vy, 10.0);
    }

    // ---- SantaNK1 (NPC.cs:33681-33921) ----

    /// The test-side transcription of Santa's attack block
    /// (`NPC.cs:33769-33847`) on the probe stream. The three trigger
    /// rolls fire EVERY tick — on any seed the Spike may add a shot or
    /// a window may (re)trigger and increment on the SAME tick (→2/3,
    /// never 1/2 after a trigger) — so the witnesses replay the whole
    /// block instead of assuming the rolls decline. Returns
    /// `(local_ai[1], local_ai[2], the block's shots)` — an independent
    /// re-derivation: the machine must produce exactly this payload
    /// and leave the streams aligned.
    fn santa_attack_block_probe(
        probe: &mut UnifiedRandom,
        s: &FrostBossBState,
        t: &FrostBossBTarget,
        flag64: bool,
    ) -> (f32, f32, Vec<FrostBossBShot>) {
        let mut shots = Vec::new();
        let mut la1 = s.local_ai[1];
        let mut la2 = s.local_ai[2];
        let (cx, cy) = s.center();
        let (pcx, pcy) = t.center();
        // `:33771-33791` — the hp-scaled denominators.
        let (d1, d2, d3) = if (s.life as f64) < (s.life_max as f64) * 0.25 {
            (300, 600, 1350)
        } else if (s.life as f64) < (s.life_max as f64) * 0.5 {
            (450, 900, 2025)
        } else if (s.life as f64) < (s.life_max as f64) * 0.75 {
            (540, 1080, 2430)
        } else {
            (600, 1200, 2700)
        };
        // `:33792-33799` — the Spike.
        if probe.next_max(d1) == 0 {
            let ox = cx - (s.direction * 24) as f32;
            let oy = cy - 64.0;
            let n = probe.next_range(1, 100) as f32 * s.direction as f32;
            let k = 1.0 / (n * n + 1.0).sqrt();
            shots.push(FrostBossBShot {
                kind: 352,
                x: ox,
                y: oy,
                vx: n * k,
                vy: k,
                damage: 80,
                ai0: 0.0,
                ai1: 0.0,
            });
        }
        // `:33800-33824` — the Missile trigger + window.
        if probe.next_max(d2) == 0 {
            la1 = 1.0;
        }
        if la1 >= 1.0 {
            la1 += 1.0;
            if flag64 && la1 % 12.0 == 0.0 {
                let ox = cx - (s.direction * 24) as f32;
                let oy = cy - 64.0;
                let mut ax = pcx - ox;
                let mut ay = pcy - oy;
                ax += probe.next_range(-50, 51) as f32;
                ay += probe.next_range(-50, 51) as f32;
                let k = 12.5 / (ax * ax + ay * ay).sqrt();
                ax *= k;
                ay *= k;
                shots.push(FrostBossBShot {
                    kind: 350,
                    x: ox,
                    y: oy,
                    vx: ax * (1.0 + probe.next_range(-20, 21) as f32 * 0.015),
                    vy: ay * (1.0 + probe.next_range(-20, 21) as f32 * 0.015),
                    damage: 42,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
            if la1 >= 100.0 {
                la1 = 0.0;
            }
        }
        // `:33825-33846` — the Present trigger + window (no flag64).
        if probe.next_max(d3) == 0 {
            la2 = 2.0;
        }
        if la2 > 0.0 {
            la2 += 1.0;
            if la2 % 9.0 == 0.0 {
                let ox = cx - (s.direction * 24) as f32;
                let oy = cy - 64.0;
                let n = probe.next_range(-100, 101) as f32;
                let k = 11.0 / (n * n + 300.0f32 * 300.0).sqrt();
                shots.push(FrostBossBShot {
                    kind: 351,
                    x: ox,
                    y: oy,
                    vx: n * k * (1.0 + probe.next_range(-20, 21) as f32 * 0.01),
                    vy: -300.0f32 * k * (1.0 + probe.next_range(-20, 21) as f32 * 0.01),
                    damage: 50,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
            if la2 >= 100.0 {
                la2 = 0.0;
            }
        }
        (la1, la2, shots)
    }

    /// The shared Santa witness: run one tick against a same-seeded
    /// probe that replays the volley (when applicable) then the whole
    /// attack block; assert the exact shot list, both windows and the
    /// stream alignment for 8 further draws.
    fn santa_tick_witness(
        s: &mut FrostBossBState,
        w: &World,
        t: &FrostBossBTarget,
        seed: i32,
        volley: Option<f32>,
        flag64: bool,
    ) {
        let mut rng = UnifiedRandom::new(seed);
        let mut probe = UnifiedRandom::new(seed);
        let mut shots = Vec::new();
        let before = *s;
        santa_nk1_ai(s, w, t, &mut rng, &mut shots);
        let mut expected = Vec::new();
        if let Some(cadence) = volley {
            // `:33750-33760` — the volley fires on the cadence tick.
            assert!((before.ai[1] + 1.0) % cadence == 0.0);
            let (cx, cy) = before.center();
            let (pcx, pcy) = t.center();
            let ox = cx + (before.direction * 50) as f32;
            let oy = cy + probe.next_range(15, 36) as f32;
            let mut ax = pcx - ox;
            let mut ay = pcy - oy;
            ax += probe.next_range(-40, 41) as f32;
            ay += probe.next_range(-40, 41) as f32;
            let k = 15.0 / (ax * ax + ay * ay).sqrt();
            ax *= k;
            ay *= k;
            expected.push(FrostBossBShot {
                kind: 180,
                x: ox,
                y: oy,
                vx: ax * (1.0 + probe.next_range(-20, 21) as f32 * 0.015),
                vy: ay * (1.0 + probe.next_range(-20, 21) as f32 * 0.015),
                damage: 36,
                ai0: 0.0,
                ai1: 0.0,
            });
        }
        let (la1, la2, block_shots) = santa_attack_block_probe(&mut probe, &before, t, flag64);
        expected.extend(block_shots);
        assert_eq!(shots, expected, "seed {seed}");
        assert_eq!(s.local_ai[1], la1, "seed {seed}");
        assert_eq!(s.local_ai[2], la2, "seed {seed}");
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next(), "seed {seed}");
        }
    }

    /// `:33720-33730` — the idle: 300 ticks then the volley flip; the
    /// three attack rolls STILL fire every idle tick (the full-block
    /// witness covers any seed, triggers included).
    #[test]
    fn santa_idle_flip_and_the_per_tick_attack_rolls() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        s.ai[1] = 299.0;
        santa_tick_witness(&mut s, &w, &t, 42, None, true);
        assert_eq!((s.ai[0], s.ai[1]), (1.0, 0.0));
        // A mid-idle tick keeps the state and still rolls.
        let mut s2 = FrostBossBState::santa_nk1(1000.0, 600.0);
        s2.ai[1] = 100.0;
        santa_tick_witness(&mut s2, &w, &t, 43, None, true);
        assert_eq!((s2.ai[0], s2.ai[1]), (0.0, 101.0));
    }

    /// `:33737-33760` — the volley: the hp-tiered cadence (16/14/11/8),
    /// the origin-Y draw FIRST, the centre aim + ±40 jitters, the 15f
    /// normalisation, the ×0.015 speed jitters; Type 180, Damage 36.
    /// The full-block witness follows the volley draws on every tier.
    #[test]
    fn santa_volley_cadence_and_draw_order() {
        let w = air();
        let t = target(1100.0, 700.0);
        for (frac, cadence) in [(1.0f32, 16.0f32), (0.6, 14.0), (0.4, 11.0), (0.2, 8.0)] {
            let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
            s.life = (s.life_max as f32 * frac) as i32;
            s.ai[0] = 1.0;
            s.ai[1] = cadence - 1.0; // increments onto the cadence
            s.direction = 1;
            santa_tick_witness(&mut s, &w, &t, 99, Some(cadence), true);
            // A non-cadence tick fires no volley (the block still rolls).
            let mut s2 = FrostBossBState::santa_nk1(1000.0, 600.0);
            s2.life = (s2.life_max as f32 * frac) as i32;
            s2.ai[0] = 1.0;
            s2.ai[1] = 1.0;
            santa_tick_witness(&mut s2, &w, &t, 100, None, true);
        }
        // The volley end: ai[1] 240 → 241 > 240 resets both, no volley.
        let mut s3 = FrostBossBState::santa_nk1(1000.0, 600.0);
        s3.ai[0] = 1.0;
        s3.ai[1] = 240.0;
        santa_tick_witness(&mut s3, &w, &t, 101, None, true);
        assert_eq!((s3.ai[0], s3.ai[1]), (0.0, 0.0));
    }

    /// `:33692` + `:33735` — flag64 gates the volley AND the Missile
    /// window: a dead target suppresses those shots AND their draws;
    /// the three trigger rolls still fire and the Present window is
    /// NOT flag64-gated (`:33829-33846`).
    #[test]
    fn santa_dead_target_suppresses_the_volley_not_the_rolls() {
        let w = air();
        let mut t = target(1100.0, 700.0);
        t.dead = true;
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        s.ai[0] = 1.0;
        s.ai[1] = 15.0; // 16 % 16 == 0 would fire if flag64
        s.local_ai[1] = 11.0; // the Missile window would fire too
        s.local_ai[2] = 8.0; // the Present window fires regardless
        santa_tick_witness(&mut s, &w, &t, 11, None, false);
        // The >2000px arm of flag64: same suppression (the witness
        // recomputes flag64 from the same box).
        let far = target(9000.0, 700.0);
        let mut s2 = FrostBossBState::santa_nk1(1000.0, 600.0);
        s2.ai[0] = 1.0;
        s2.ai[1] = 15.0;
        s2.local_ai[2] = 8.0;
        santa_tick_witness(&mut s2, &w, &far, 11, None, false);
    }

    /// `:33792-33798` — the Spike: on Next(600)==0 the behind-origin
    /// (−direction*24, −64), the Next(1,100)*direction X speed against
    /// the 1f Y, normalised; Type 352, Damage 80. A seed whose first
    /// roll hits pins the payload through the full-block witness.
    #[test]
    fn santa_spike_roll_and_shot() {
        let w = air();
        let t = target(1100.0, 700.0);
        // Find a seed whose first Next(600) is 0.
        let seed = (0..20000)
            .find(|&sd| UnifiedRandom::new(sd).next_max(600) == 0)
            .expect("a Next(600)==0 seed exists");
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        s.direction = -1;
        santa_tick_witness(&mut s, &w, &t, seed, None, true);
        // The Spike is the FIRST shot the block can produce; the
        // witness already asserted the exact list — re-derive its head
        // here for the payload pin.
        let mut probe = UnifiedRandom::new(seed);
        assert_eq!(probe.next_max(600), 0);
        let n = probe.next_range(1, 100) as f32 * -1.0;
        let k = 1.0 / (n * n + 1.0).sqrt();
        let spike = FrostBossBShot {
            kind: 352,
            x: 1056.0 + 24.0, // −direction*24 with direction −1
            y: 670.0 - 64.0,
            vx: n * k,
            vy: k,
            damage: 80,
            ai0: 0.0,
            ai1: 0.0,
        };
        let mut rng = UnifiedRandom::new(seed);
        let mut shots = Vec::new();
        let mut s2 = FrostBossBState::santa_nk1(1000.0, 600.0);
        s2.direction = -1;
        santa_nk1_ai(&mut s2, &w, &t, &mut rng, &mut shots);
        assert_eq!(shots.first(), Some(&spike));
    }

    /// `:33800-33824` — the Missile window: the trigger sets
    /// localAI[1]=1 and it increments on the SAME tick (→2), fires at
    /// %12, the ±50 aim jitters + ×0.015 speed jitters, resets at
    /// >=100. A re-trigger mid-window overwrites to 1 then increments
    /// to 2 — the witness replays both shapes on any seed.
    #[test]
    fn santa_missile_window_cadence() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        s.local_ai[1] = 11.0; // increments to 12 → fires
        santa_tick_witness(&mut s, &w, &t, 77, None, true);
        // The window end: localAI[1] 99 → 100 → 0 (unless re-triggered,
        // which the witness covers).
        let mut s2 = FrostBossBState::santa_nk1(1000.0, 600.0);
        s2.local_ai[1] = 99.0;
        santa_tick_witness(&mut s2, &w, &t, 3, None, true);
        // A fresh trigger on a quiet tick: localAI[1] 0 → (trigger?)
        // 1 → 2 on the same tick — the witness asserts the C# shape.
        let mut s3 = FrostBossBState::santa_nk1(1000.0, 600.0);
        santa_tick_witness(&mut s3, &w, &t, 78, None, true);
    }

    /// `:33825-33846` — the Present window: the trigger sets
    /// localAI[2]=2 and increments the same tick (→3), fires at %9,
    /// the Next(-100,101) X aim against −300f, the 11f normalisation,
    /// the ×0.01 jitters; NOT flag64-gated (a dead target still gets
    /// presents — the witness runs with a dead target here).
    #[test]
    fn santa_present_window_draw_order() {
        let w = air();
        let mut t = target(1100.0, 700.0);
        t.dead = true; // the window ignores flag64
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        s.local_ai[2] = 8.0; // increments to 9 → fires
        santa_tick_witness(&mut s, &w, &t, 555, None, false);
        // The window end: localAI[2] 99 → 100 → 0 (unless re-triggered).
        let mut s2 = FrostBossBState::santa_nk1(1000.0, 600.0);
        s2.local_ai[2] = 99.0;
        santa_tick_witness(&mut s2, &w, &t, 556, None, false);
    }

    /// `:33711-33719` — the day flee: EncourageDespawn(10), speed 8,
    /// the 0.1f X nudge when vx==0 — and the three attack rolls STILL
    /// fire by day (the block sits OUTSIDE the day/phase chain).
    #[test]
    fn santa_day_flee_still_rolls_the_attacks() {
        let mut w = air();
        w.day_time = true;
        let t = target(5000.0, 700.0);
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        santa_tick_witness(&mut s, &w, &t, 8, None, true);
        assert_eq!(s.time_left, 10);
        assert!(s.despawn_encouraged);
        // vx nudged to 0.1, then the approach: (0.1*20 + 8)/21.
        assert_eq!(s.vx, (0.1f32 * 20.0 + 8.0) / 21.0);
    }

    /// `:33774-33791` — the hp-scaled denominators: <25% halves
    /// (300/600/1350), <50% ×0.75 (450/900/2025), <75% ×0.9
    /// (540/1080/2430). The witness replays the scaled maxima, so a
    /// wrong tier desynchronises the streams on the first roll.
    #[test]
    fn santa_attack_denominator_hp_tiers() {
        let w = air();
        let t = target(1100.0, 700.0);
        for frac in [1.0f32, 0.6, 0.4, 0.2] {
            let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
            s.life = (s.life_max as f32 * frac) as i32;
            santa_tick_witness(&mut s, &w, &t, 123, None, true);
        }
    }

    // ---- the dispatch seam ----

    /// The seam claims exactly 344/346; the IceQueen 345 (the sibling
    /// module), the non-boss Frost set (worker D) and every other id
    /// fall through — a declined id draws nothing.
    #[test]
    fn dispatch_claims_everscream_and_santa_only() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossBState::everscream(1000.0, 600.0);
        let mut rng = UnifiedRandom::new(1);
        let mut shots = Vec::new();
        let mut dust = DustPool::new();
        assert!(frost_boss_b_ai_step(344, &mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx()));
        let mut s = FrostBossBState::santa_nk1(1000.0, 600.0);
        assert!(frost_boss_b_ai_step(346, &mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx()));
        for id in [0u16, 3, 325, 327, 338, 341, 343, 345, 347, 352, 353, 670] {
            let mut s = FrostBossBState::everscream(1000.0, 600.0);
            let mut rng = UnifiedRandom::new(1);
            let mut probe = UnifiedRandom::new(1);
            let mut shots = Vec::new();
            let mut dust = DustPool::new();
            assert!(!frost_boss_b_ai_step(id, &mut s, &w, &t, &mut rng, &mut shots, &mut dust, &dust_ctx()), "{id}");
            assert_eq!(rng.next(), probe.next(), "{id}");
        }
    }

    /// The AI defaults match the SetDefaults rows (`NPC.cs:13357-13362`
    /// / `:13388-13393`) — the defaults workers' agreement points — and
    /// the aiStyle constants match `:13359`/`:13390`.
    #[test]
    fn the_defaults_match_set_defaults() {
        assert_eq!(EVERSCREAM_TYPE, 344);
        assert_eq!(EVERSCREAM_AI_STYLE, 57);
        assert_eq!(everscream_ai_defaults(), (172.0, 130.0, 13000));
        assert_eq!(SANTA_NK1_TYPE, 346);
        assert_eq!(SANTA_NK1_AI_STYLE, 61);
        assert_eq!(santa_nk1_ai_defaults(), (112.0, 140.0, 18000));
        let e = FrostBossBState::everscream(0.0, 0.0);
        assert_eq!((e.width, e.height, e.life, e.life_max), (172.0, 130.0, 13000, 13000));
        assert_eq!(e.center(), (86.0, 65.0));
        assert_eq!(e.time_left, 750);
        let k = FrostBossBState::santa_nk1(0.0, 0.0);
        assert_eq!((k.width, k.height, k.life, k.life_max), (112.0, 140.0, 18000, 18000));
        assert_eq!(k.center(), (56.0, 70.0));
    }
}
