//! Frost Moon mob AI — S1-F-13c slice D (the non-boss Snow Moon machines).
//!
//! The C# arms ported here, keyed by the vanilla `NPC.type` / `aiStyle`:
//!
//! * **Flocko (352, aiStyle 63)** — the spinning snowflake chaser
//!   (`NPC.cs:33971-34018`, the `aiStyle == 63` arm of `NPC.AI`).
//! * **ElfCopter (347, aiStyle 62)** — the hover-and-shoot helicopter
//!   (`NPC.cs:33922-33969`); its six `Main.rand` draws ride the caller's
//!   stream in the C# order (the RNG witness tests below).
//! * **PresentMimic (341, aiStyle 25)** — the disguised-present hopper
//!   (`NPC.cs:25538-25630`, the `aiStyle == 25` arm; the type-341 terms
//!   `flag40`/`ai[3] = 1f` are kept verbatim).
//! * **The aiStyle-3 walkers** — ZombieElf/Beard/Girl (338/339/340),
//!   GingerbreadMan (342), Yeti (343), Nutcracker (348),
//!   NutcrackerSpinning (349), Krampus (351): the per-type velocity arms
//!   of `AI_003_Fighters` (`NPC.cs:57403-57440` the 349 spin arm,
//!   `:57554-57578` the cap-2 group, `:57606-57740` the `num83` group)
//!   plus the Nutcracker's 55%-hp transform gate (`NPC.cs:57031-57034`).
//!
//! **Deferred (documented in docs/ANALYSIS.md, S1-F-13c slice D):**
//! ElfArcher (350)'s stop-and-fire machine (the shared shooter block
//! `NPC.cs:59069-59629` — the same block the martian shooters ride), the
//! AI_003 shared stuck/jump/collision core (`:57120-57402` + the movement
//! tail — it lands with the full AI_003 migration, S1-D-3p's note), the
//! `FindFrame` animation arms (`:75733-75824` Krampus/Yeti, `:76099+`
//! ElfArcher, `:76276-76380` Nutcracker/GingerbreadMan, `:72357+`
//! ElfCopter, `:72413+` Flocko, `:73444+` PresentMimic), and the three
//! bosses Everscream (344) / IceQueen (345) / SantaNK1 (346) — worker C's
//! `task/S1-F-13c-boss-ai` row.
//!
//! **Integration seam / dependency:** the `NpcKind` variants + the full
//! `SetDefaults` rows for 338-352 are owned by the parallel workers A
//! (`task/S1-F-13c-defaults-a`, 338-344) and B
//! (`task/S1-F-13c-defaults-b`, 345-352). Until they land, this module is
//! deliberately disjoint: the machines run on the plain [`FrostMobState`]
//! (the AI-visible subset of `NPC`'s fields) and the dispatch
//! ([`frost_mob_ai_step`]) keys on the vanilla type id + the C# `aiStyle`.
//! The wiring worker copies `Npc`'s fields in/out and calls
//! `npc_target::target_closest` where the C# arms call `TargetClosest()`
//! (each machine documents its call sites); the sizes/scales/lifeMax
//! below are the AI-relevant `SetDefaults` inputs, cited per row — they
//! must agree with A/B's tables when those land.

// The pure dispatch seam (`frost_mob_ai_step`) stays test-only — the live
// wiring below (`frost_mob_step`, S1-D-3 W-6) calls the machines directly
// so each `TargetClosest()` site keeps its C# position.
#![allow(dead_code)]

use crate::npc::{Npc, NpcKind, GRAVITY, MAX_FALL};
use crate::npc_ai::WalkerParams;
use crate::npc_ai_frost_mob_b::{
    elf_archer_ai_retarget, ArcherTick, FrostArcherShot, FrostArcherTarget, FrostMobBState,
};
use crate::npc_target::{session_candidate, target_closest};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The non-boss Frost Moon kinds this module owns, with their vanilla
/// `NPCID` values (`NPCID.cs:11353-11381`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrostMobKind {
    /// `NPCID.ZombieElf` (338) — `NPCID.cs:11353`.
    ZombieElf,
    /// `NPCID.ZombieElfBeard` (339) — `NPCID.cs:11355`.
    ZombieElfBeard,
    /// `NPCID.ZombieElfGirl` (340) — `NPCID.cs:11357`.
    ZombieElfGirl,
    /// `NPCID.PresentMimic` (341) — `NPCID.cs:11359`.
    PresentMimic,
    /// `NPCID.GingerbreadMan` (342) — `NPCID.cs:11361`.
    GingerbreadMan,
    /// `NPCID.Yeti` (343) — `NPCID.cs:11363`.
    Yeti,
    /// `NPCID.ElfCopter` (347) — `NPCID.cs:11371`.
    ElfCopter,
    /// `NPCID.Nutcracker` (348) — `NPCID.cs:11373`.
    Nutcracker,
    /// `NPCID.NutcrackerSpinning` (349) — `NPCID.cs:11375`.
    NutcrackerSpinning,
    /// `NPCID.ElfArcher` (350) — `NPCID.cs:11377`. The shooter machine is
    /// deferred (the shared block `NPC.cs:59069-59629`); only the kind
    /// row + aiStyle exist here so the dispatch can name it.
    ElfArcher,
    /// `NPCID.Krampus` (351) — `NPCID.cs:11379`.
    Krampus,
    /// `NPCID.Flocko` (352) — `NPCID.cs:11381`.
    Flocko,
}

impl FrostMobKind {
    /// The vanilla `NPC.type` (`NPCID.cs:11353-11381`). The bosses
    /// 344/345/346 are NOT in this map — worker C owns them.
    pub fn type_id(self) -> u16 {
        match self {
            Self::ZombieElf => 338,
            Self::ZombieElfBeard => 339,
            Self::ZombieElfGirl => 340,
            Self::PresentMimic => 341,
            Self::GingerbreadMan => 342,
            Self::Yeti => 343,
            Self::ElfCopter => 347,
            Self::Nutcracker => 348,
            Self::NutcrackerSpinning => 349,
            Self::ElfArcher => 350,
            Self::Krampus => 351,
            Self::Flocko => 352,
        }
    }

    /// The reverse map — `None` for every id outside the non-boss set
    /// (the bosses 344-346 included: worker C's row).
    pub fn from_type(t: u16) -> Option<Self> {
        Some(match t {
            338 => Self::ZombieElf,
            339 => Self::ZombieElfBeard,
            340 => Self::ZombieElfGirl,
            341 => Self::PresentMimic,
            342 => Self::GingerbreadMan,
            343 => Self::Yeti,
            347 => Self::ElfCopter,
            348 => Self::Nutcracker,
            349 => Self::NutcrackerSpinning,
            350 => Self::ElfArcher,
            351 => Self::Krampus,
            352 => Self::Flocko,
            _ => return None,
        })
    }

    /// The C#'s `this.aiStyle` — the `SetDefaults` assignment lines:
    /// 338-340 `NPC.cs:13285` (3), 341 `:13318` (25), 342 `:13331` (3),
    /// 343 `:13344` (3), 347 `:13404` (62), 348 `:13420` (3),
    /// 349 `:13434` (3), 350 `:13448` (3), 351 `:13461` (3),
    /// 352 `:13475` (63).
    pub fn ai_style(self) -> u8 {
        match self {
            Self::PresentMimic => 25,
            Self::ElfCopter => 62,
            Self::Flocko => 63,
            _ => 3,
        }
    }

    /// The AI-relevant `SetDefaults` inputs: `(width, height, scale,
    /// lifeMax)` — `NPC.cs:13281-13312` (338-340), `:13314-13326` (341),
    /// `:13327-13339` (342), `:13340-13354` (343), `:13400-13415` (347),
    /// `:13416-13443` (348/349), `:13444-13456` (350), `:13457-13470`
    /// (351), `:13471-13487` (352). The full stat rows (damage/defense/
    /// value/knockBackResist/…) are workers A/B's tables.
    pub fn ai_defaults(self) -> (f32, f32, f32, i32) {
        match self {
            Self::ZombieElf => (18.0, 40.0, 1.0, 600),
            Self::ZombieElfBeard => (18.0, 40.0, 1.05, 700),
            Self::ZombieElfGirl => (18.0, 40.0, 0.9, 500),
            Self::PresentMimic => (24.0, 24.0, 1.0, 900),
            Self::GingerbreadMan => (18.0, 40.0, 1.0, 750),
            Self::Yeti => (38.0, 78.0, 1.0, 3500),
            Self::ElfCopter => (50.0, 50.0, 1.0, 1200),
            Self::Nutcracker | Self::NutcrackerSpinning => (28.0, 76.0, 1.0, 1800),
            Self::ElfArcher => (18.0, 40.0, 1.0, 900),
            Self::Krampus => (18.0, 90.0, 1.0, 2500),
            Self::Flocko => (54.0, 54.0, 1.0, 450),
        }
    }
}

/// The AI-visible subset of `NPC`'s fields the machines read/write. The
/// wiring worker copies these in/out of `Npc` (position/velocity/ai/
/// localAI/direction/spriteDirection/rotation/life/timeLeft); the
/// `noGravity`/`noTileCollide` flyers (347/352) integrate the velocity
/// directly, the walkers/mimic ride the caller's collision pass.
#[derive(Clone, Copy, Debug)]
pub struct FrostMobState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `NPC.ai[0..3]`.
    pub ai: [f32; 4],
    /// `NPC.localAI[0..3]`.
    pub local_ai: [f32; 4],
    /// `Entity.direction`.
    pub direction: i32,
    /// `NPC.spriteDirection`.
    pub sprite_direction: i32,
    pub rotation: f32,
    /// `NPC.life`/`lifeMax`.
    pub life: i32,
    pub life_max: i32,
    /// `Entity.width`/`height` (the `SetDefaults` box).
    pub width: f32,
    pub height: f32,
    /// `NPC.scale` (the GingerbreadMan's `num83` scale term reads it).
    pub scale: f32,
    /// `NPC.timeLeft` — the port's `despawn_timer` convention: -1 = the
    /// unset window, `EncourageDespawn` only ever lowers it
    /// (`NPC.cs:7249-7256`, `npc.rs`'s `encourage_despawn`).
    pub despawn_timer: i32,
}

impl FrostMobState {
    /// A fresh mob at (x, y) with the `SetDefaults` AI inputs and the
    /// C#'s field defaults (`ai`/`localAI` zeroed, direction 1).
    pub fn new(kind: FrostMobKind, x: f32, y: f32) -> Self {
        let (width, height, scale, life_max) = kind.ai_defaults();
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            ai: [0.0; 4],
            local_ai: [0.0; 4],
            direction: 1,
            sprite_direction: 1,
            rotation: 0.0,
            life: life_max,
            life_max,
            width,
            height,
            scale,
            despawn_timer: -1,
        }
    }

    /// `Entity.Center` (`Entity.cs:50-55`): position + size/2.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// `NPC.EncourageDespawn` (`NPC.cs:7249-7256`): only ever LOWERS the
    /// `timeLeft` window (the port's -1 = unset convention).
    fn encourage_despawn(&mut self, ticks: i32) {
        if self.despawn_timer < 0 || self.despawn_timer > ticks {
            self.despawn_timer = ticks;
        }
    }
}

/// The resolved `Main.player[this.target]` box. The C# arms call
/// `TargetClosest()` at their top (Flocko `NPC.cs:33973`, ElfCopter
/// `:33924`, PresentMimic `:25550`/`:25566`/`:25601` under `!flag40`);
/// the wiring worker runs `npc_target::target_closest` at those points
/// and passes the picked slot here.
#[derive(Clone, Copy, Debug)]
pub struct FrostTarget {
    /// `Player.position.X`/`Y`.
    pub x: f32,
    pub y: f32,
    /// `Player.width`/`height` (the 20×42 box).
    pub width: f32,
    pub height: f32,
}

impl FrostTarget {
    /// `Player.Center` (`Entity.cs:50-55`).
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// A hostile-shot request — `Projectile.NewProjectile`'s
/// `(X, Y, SpeedX, SpeedY, Type, Damage, KnockBack)` payload
/// (`NPC.cs:33967` for the copter's 180). KnockBack is 0f at every call
/// site ported here, so it is not carried.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrostShot {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// **Flocko (352)** — the `aiStyle == 63` arm (`NPC.cs:33971-34018`),
/// verbatim: the 11f aim from `Center + (direction*20, 6)`, the day
/// reversal, the `ai[0]` spin latch (20 ticks inside 200px), the
/// 50/51 → 10/11 → 7/8 chase lerps by distance and the `velocity.X*0.15`
/// tilt. `TargetClosest()` (`:33973`) is the caller's job (see
/// [`FrostTarget`]). No RNG draws, no gravity (`noGravity` +
/// `noTileCollide`, `NPC.cs:13483-13484`) — the caller integrates.
pub fn flocko_ai(s: &mut FrostMobState, target: &FrostTarget, day_time: bool) {
    // `:33974-33981` — num953 = 11f; vector120 = Center + (dir*20, +6).
    let num953 = 11.0f32;
    let (cx, cy) = s.center();
    let v_x = cx + (s.direction * 20) as f32;
    let v_y = cy + 6.0;
    let (pcx, pcy) = target.center();
    let mut num954 = pcx - v_x;
    // `:33977` — the aim's Y uses the player's CENTER (unlike the
    // copter's chase, which uses position.Y).
    let mut num955 = pcy - v_y;
    let num956 = (num954 * num954 + num955 * num955).sqrt();
    let num957 = num953 / num956;
    num954 *= num957;
    num955 *= num957;
    // `:33982-33986` — the day reversal.
    if day_time {
        num954 = 0.0 - num954;
        num955 = 0.0 - num955;
    }
    // `:33987-34004` — the spin latch.
    s.ai[0] -= 1.0;
    if num956 < 200.0 || s.ai[0] > 0.0 {
        if num956 < 200.0 {
            s.ai[0] = 20.0;
        }
        s.direction = if s.vx < 0.0 { -1 } else { 1 };
        s.rotation += s.direction as f32 * 0.3;
        return;
    }
    // `:34005-34016` — the distance-tiered chase lerps.
    s.vx = (s.vx * 50.0 + num954) / 51.0;
    s.vy = (s.vy * 50.0 + num955) / 51.0;
    if num956 < 350.0 {
        s.vx = (s.vx * 10.0 + num954) / 11.0;
        s.vy = (s.vy * 10.0 + num955) / 11.0;
    }
    if num956 < 300.0 {
        s.vx = (s.vx * 7.0 + num954) / 8.0;
        s.vy = (s.vy * 7.0 + num955) / 8.0;
    }
    // `:34017`.
    s.rotation = s.vx * 0.15;
}

/// **ElfCopter (347)** — the `aiStyle == 62` arm (`NPC.cs:33922-33969`),
/// verbatim. The six `Main.rand` draws of the shot arm (`:33960-33967`)
/// ride `rng` in the C# order: `Next(-35,36)` ×2 (aim jitter),
/// `Next(-20,21)` ×2 (aim scale, ×0.015), `Next(-20,21)` ×2 (speed
/// scale, ×0.0125). The `Main.netMode != 1` gate (`:33952`) is the
/// port's always-true server/local path. `TargetClosest()` (`:33924`)
/// is the caller's job. `Collision.CanHit(Center,1,1, player.Center,1,1)`
/// (`:33935`) rides the shared `projectile::can_hit` walk.
pub fn elf_copter_ai(
    s: &mut FrostMobState,
    world: &World,
    target: &FrostTarget,
    day_time: bool,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostShot>,
) {
    // `:33925-33926` — the tilt + sprite flip BEFORE the velocity update.
    s.rotation = s.vx.abs() * s.direction as f32 * 0.1;
    s.sprite_direction = s.direction;
    // `:33927-33934` — num945 = 7f; vector119 = Center + (dir*20, +6).
    let num945 = 7.0f32;
    let (cx, cy) = s.center();
    let v_x = cx + (s.direction * 20) as f32;
    let v_y = cy + 6.0;
    let (pcx, pcy) = target.center();
    let mut num946 = pcx - v_x;
    // `:33930` — the CHASE aim's Y uses position.Y (the top), not Center.
    let mut num947 = target.y - v_y;
    let num948 = (num946 * num946 + num947 * num947).sqrt();
    let num949 = num945 / num948;
    num946 *= num949;
    num947 *= num949;
    // `:33935` — the 1×1 centre LOS walk.
    let flag66 = crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
    // `:33936-33943` — the day flee + EncourageDespawn(10).
    if day_time {
        let num950 = 60.0f32;
        s.vx = (s.vx * (num950 - 1.0) - num946) / num950;
        s.vy = (s.vy * (num950 - 1.0) - num947) / num950;
        s.encourage_despawn(10);
        return;
    }
    // `:33944-33950` — the far/no-LOS approach.
    if num948 > 600.0 || !flag66 {
        let num951 = 60.0f32;
        s.vx = (s.vx * (num951 - 1.0) + num946) / num951;
        s.vy = (s.vy * (num951 - 1.0) + num947) / num951;
        return;
    }
    // `:33951-33968` — the hover + the 15-tick shot cadence.
    s.vx *= 0.98;
    s.vy *= 0.98;
    if s.vx.abs() < 1.0 && s.vy.abs() < 1.0 {
        s.local_ai[0] += 1.0;
        if s.local_ai[0] >= 15.0 {
            s.local_ai[0] = 0.0;
            // `:33958-33959` — the shot aim re-derives from the player's
            // CENTER.Y (the chase used position.Y).
            num946 = pcx - v_x;
            num947 = pcy - v_y;
            // `:33960-33963` — draws 1-4, in order.
            num946 += rng.next_range(-35, 36) as f32;
            num947 += rng.next_range(-35, 36) as f32;
            num946 *= 1.0 + rng.next_range(-20, 21) as f32 * 0.015;
            num947 *= 1.0 + rng.next_range(-20, 21) as f32 * 0.015;
            // `:33964-33966` — the 10f normalisation.
            let num949b = 10.0 / (num946 * num946 + num947 * num947).sqrt();
            num946 *= num949b;
            num947 *= num949b;
            // `:33967` — draws 5-6 ride the SpeedX/SpeedY arguments;
            // Type 180 (Ice Spike), Damage 32, KnockBack 0f.
            shots.push(FrostShot {
                kind: 180,
                x: v_x,
                y: v_y,
                vx: num946 * (1.0 + rng.next_range(-20, 21) as f32 * 0.0125),
                vy: num947 * (1.0 + rng.next_range(-20, 21) as f32 * 0.0125),
                damage: 32,
            });
        }
    }
}

/// **PresentMimic (341)** — the `aiStyle == 25` arm (`NPC.cs:25538-25630`)
/// with the type-341 terms kept: `flag40 = type == 341 && !Main.snowMoon`
/// (`:25540` — outside the event the mimic never re-targets) and the
/// forced `ai[3] = 1f` (`:25558-25561`). The one-time `ai[3] == 0` block
/// (`:25541-25557`) still runs its `position.X += 8f` nudge and the
/// layer pick (whose `TargetClosest` at `:25550` the caller mirrors when
/// the wiring lands); `Main.UnderworldLayer` is `maxTilesY - 200`
/// (`Main.cs:2790`). The hop cycle: 12 ticks then 20 (`:25588-25592`),
/// the alternating (3.5, -4) / (2.5, -8) hops (`:25608-25619`), the
/// airborne ±0.1 drift (`:25622-25629`). Gravity/collision integration
/// is the caller's (the mimic is a grounded walker).
pub fn present_mimic_ai(
    s: &mut FrostMobState,
    world: &World,
    target: &FrostTarget,
    snow_moon: bool,
) {
    present_mimic_ai_retarget(s, world, target, snow_moon, &mut |_| {});
}

/// The wired form (S1-D-3 W-6): `retarget` runs at each C# `TargetClosest()`
/// site (`NPC.cs:25550` the layer pick, `:25566` the idle arm under
/// `!flag40`, `:25601` the hop launch under `!flag40`) and syncs the picked
/// facing back into `s.direction`. The pure [`present_mimic_ai`] passes a
/// no-op so the module tests keep the resolved-target form.
pub fn present_mimic_ai_retarget(
    s: &mut FrostMobState,
    world: &World,
    target: &FrostTarget,
    snow_moon: bool,
    retarget: &mut dyn FnMut(&mut FrostMobState),
) {
    // `:25540`.
    let flag40 = !snow_moon;
    // `:25541-25557` — the one-time placement nudge + layer pick.
    if s.ai[3] == 0.0 {
        s.x += 8.0;
        let underworld_layer = (world.height - 200) as f32;
        if s.y / 16.0 > underworld_layer {
            s.ai[3] = 3.0;
        } else if (s.y / 16.0) as f64 > world.world_surface {
            // `:25550` — TargetClosest().
            retarget(s);
            s.ai[3] = 2.0;
        } else {
            s.ai[3] = 1.0;
        }
    }
    // `:25558-25561` — the 341/629 override.
    s.ai[3] = 1.0;
    if s.ai[0] == 0.0 {
        // `:25562-25566` — the idle arm re-targets under !flag40.
        if !flag40 {
            retarget(s);
        }
        // `:25567-25584` — the disguised idle: wake on movement, on the
        // ±100px inflated overlap, or on any damage.
        if s.vx != 0.0 || s.vy < 0.0 || s.vy as f64 > 0.3 {
            s.ai[0] = 1.0;
            return;
        }
        // `:25578-25579` — the int-truncated rectangles.
        let (mx, my, mw, mh) = (
            s.x as i32 - 100,
            s.y as i32 - 100,
            s.width as i32 + 200,
            s.height as i32 + 200,
        );
        let (px, py, pw, ph) = (
            target.x as i32,
            target.y as i32,
            target.width as i32,
            target.height as i32,
        );
        let intersects = mx < px + pw && px < mx + mw && my < py + ph && py < my + mh;
        if intersects || s.life < s.life_max {
            s.ai[0] = 1.0;
        }
    } else if s.vy == 0.0 {
        // `:25585-25621` — the grounded hop cycle.
        s.ai[2] += 1.0;
        let num335 = if s.ai[1] == 0.0 { 12.0 } else { 20.0 };
        if s.ai[2] < num335 {
            s.vx *= 0.9;
            return;
        }
        s.ai[2] = 0.0;
        // `:25599-25602` — TargetClosest() under !flag40.
        if !flag40 {
            retarget(s);
        }
        // `:25603-25607`.
        if s.direction == 0 {
            s.direction = -1;
        }
        s.sprite_direction = s.direction;
        // `:25608-25619` — the alternating hops.
        s.ai[1] += 1.0;
        if s.ai[1] == 2.0 {
            s.vx = s.direction as f32 * 2.5;
            s.vy = -8.0;
            s.ai[1] = 0.0;
        } else {
            s.vx = s.direction as f32 * 3.5;
            s.vy = -4.0;
        }
    } else if s.direction == 1 && s.vx < 1.0 {
        // `:25622-25625`.
        s.vx += 0.1;
    } else if s.direction == -1 && s.vx > -1.0 {
        // `:25626-25629`.
        s.vx -= 0.1;
    }
}

/// The `AI_003_Fighters` per-type velocity arms for the Frost walkers —
/// the exact C# branch order: the 349 spin arm first (`NPC.cs:57403-57440`,
/// the type-159-only 0.95 brake at `:57405-57408` does not fire for 349),
/// then the cap-2 group (`:57554-57578`: 343/348/351), then the `num83`
/// group (`:57606-57740`: 338→1.75 `:57637-57640`, 339→1.25 `:57641-57644`,
/// 340→2f `:57645-57648`, 342→1.5×(1+(1-scale)) `:57693-57696`). Returns
/// `None` for kinds whose walk is not a plain velocity arm (350's walk
/// rides the deferred shooter block, `:58322`'s exclusion list).
///
/// The shared stuck-counter/jump core (`:57120-57402`) and the collision
/// tail are the full AI_003 migration's scope (S1-D-3p) — this fn is the
/// per-type velocity delta only, the part that differs between the mobs.
pub fn fighter_walk_velocity(
    kind: FrostMobKind,
    vx: f32,
    vy: f32,
    direction: i32,
    scale: f32,
) -> Option<(f32, f32)> {
    let (mut vx, mut vy) = (vx, vy);
    match kind {
        FrostMobKind::NutcrackerSpinning => {
            // `:57409-57439` — the cap-6 spin arm (the 0.99 counter-drift
            // brake on the ground is THIS arm's own, not the cap-2 group's).
            if vx < -6.0 || vx > 6.0 {
                if vy == 0.0 {
                    vx *= 0.8;
                    vy *= 0.8;
                }
            } else if vx < 6.0 && direction == 1 {
                if vy == 0.0 && vx < 0.0 {
                    vx *= 0.99;
                }
                vx += 0.07;
                if vx > 6.0 {
                    vx = 6.0;
                }
            } else if vx > -6.0 && direction == -1 {
                if vy == 0.0 && vx > 0.0 {
                    vx *= 0.99;
                }
                vx -= 0.07;
                if vx < -6.0 {
                    vx = -6.0;
                }
            }
        }
        FrostMobKind::Yeti | FrostMobKind::Nutcracker | FrostMobKind::Krampus => {
            // `:57556-57578` — the cap-2 group, 0.07 accel, no counter brake.
            if vx < -2.0 || vx > 2.0 {
                if vy == 0.0 {
                    vx *= 0.8;
                    vy *= 0.8;
                }
            } else if vx < 2.0 && direction == 1 {
                vx += 0.07;
                if vx > 2.0 {
                    vx = 2.0;
                }
            } else if vx > -2.0 && direction == -1 {
                vx -= 0.07;
                if vx < -2.0 {
                    vx = -2.0;
                }
            }
        }
        FrostMobKind::ZombieElf
        | FrostMobKind::ZombieElfBeard
        | FrostMobKind::ZombieElfGirl
        | FrostMobKind::GingerbreadMan => {
            // `:57608-57696` — the num83 caps.
            let mut num83 = match kind {
                FrostMobKind::ZombieElf => 1.75,
                FrostMobKind::ZombieElfBeard => 1.25,
                FrostMobKind::ZombieElfGirl => 2.0,
                _ => 1.5,
            };
            // `:57693-57696` — the scale term (342 is in the list).
            if kind == FrostMobKind::GingerbreadMan {
                num83 *= 1.0 + (1.0 - scale);
            }
            // `:57697-57735` — the shared accel/clamp (the 466/586 brakes
            // are other types').
            if vx < 0.0 - num83 || vx > num83 {
                if vy == 0.0 {
                    vx *= 0.8;
                    vy *= 0.8;
                }
            } else if vx < num83 && direction == 1 {
                vx += 0.07;
                if vx > num83 {
                    vx = num83;
                }
            } else if vx > 0.0 - num83 && direction == -1 {
                vx -= 0.07;
                if vx < 0.0 - num83 {
                    vx = 0.0 - num83;
                }
            }
        }
        // 350's walk is the shooter block's (`:59715+`, deferred); the
        // flyers/mimic never reach AI_003.
        _ => return None,
    }
    Some((vx, vy))
}

/// The Nutcracker's transform gate (`NPC.cs:57031-57034`):
/// `Main.netMode != 1 && type == 348 && life <= lifeMax * 0.55` →
/// `Transform(349)`. The `(double)` cast comparison is kept. The port's
/// netMode is the always-!=1 server/local path. Returns the new type id
/// when the transform fires; the wiring worker swaps the kind (the
/// `Transform` machinery — stat re-roll — is worker B's defaults row).
pub fn nutcracker_transform(kind: FrostMobKind, life: i32, life_max: i32) -> Option<u16> {
    if kind == FrostMobKind::Nutcracker && (life as f64) <= (life_max as f64) * 0.55 {
        Some(349)
    } else {
        None
    }
}

/// The dispatch seam — keyed on the vanilla type id, gated on the C#
/// `aiStyle` (`FrostMobKind::ai_style`). Returns `false` for every id
/// this module does not own (the bosses 344-346 included — worker C).
///
/// The aiStyle-3 arm applies the per-type velocity delta only (the
/// shared AI_003 core is the S1-D-3p migration); 350 is recognised but
/// its shooter machine is deferred, so its step is a no-op that still
/// reports `true` (the kind is claimed). `rng` is consumed ONLY by the
/// copter's shot arm — every other path leaves the stream untouched
/// (the witness tests pin both halves).
pub fn frost_mob_ai_step(
    type_id: u16,
    s: &mut FrostMobState,
    world: &World,
    target: &FrostTarget,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostShot>,
) -> bool {
    let Some(kind) = FrostMobKind::from_type(type_id) else {
        return false;
    };
    match kind.ai_style() {
        63 => flocko_ai(s, target, world.day_time),
        62 => elf_copter_ai(s, world, target, world.day_time, rng, shots),
        25 => present_mimic_ai(s, world, target, world.snow_moon),
        3 => {
            if let Some((vx, vy)) = fighter_walk_velocity(kind, s.vx, s.vy, s.direction, s.scale) {
                s.vx = vx;
                s.vy = vy;
            }
        }
        _ => unreachable!("the Frost mob set only carries aiStyles 3/25/62/63"),
    }
    true
}

// =====================================================================
// The live wiring (S1-D-3 W-6, census slice W-6): `Npc` copy-in/out over
// the pure machines above. The kinds wired here are exactly the census's
// UNWIRED mobs — 341 PresentMimic, 347 ElfCopter, 348/349 Nutcracker
// twins, 350 ElfArcher (the sibling `npc_ai_frost_mob_b.rs` machine),
// 352 Flocko. The five WIRED-DELEGATE walkers (338/339/340/342/343/351)
// keep their `update_skeleton` ladder routing; the bosses 344/345/346
// are R41's row (`task/S1-F-13c-wire-bosses`) and are NOT claimed here.
// =====================================================================

/// The `NpcKind` → module-kind map for the wired mob set (the bosses and
/// the ladder-owned walkers are absent by construction).
fn wired_kind(kind: NpcKind) -> Option<FrostMobKind> {
    Some(match kind {
        NpcKind::PresentMimic => FrostMobKind::PresentMimic,
        NpcKind::ElfCopter => FrostMobKind::ElfCopter,
        NpcKind::Nutcracker => FrostMobKind::Nutcracker,
        NpcKind::NutcrackerSpinning => FrostMobKind::NutcrackerSpinning,
        NpcKind::Flocko => FrostMobKind::Flocko,
        _ => return None,
    })
}

/// Copy the AI-visible `Npc` fields into the module state (the module
/// header's field list). `Npc.hp`/`max_hp` are the C# `life`/`lifeMax`,
/// `facing` the `Entity.direction`, `despawn_timer` the `timeLeft`
/// convention (-1 = unset).
fn state_from_npc(npc: &Npc) -> FrostMobState {
    let (w, h) = npc.size();
    FrostMobState {
        x: npc.x,
        y: npc.y,
        vx: npc.vx,
        vy: npc.vy,
        ai: npc.ai,
        local_ai: npc.local_ai,
        direction: npc.facing,
        sprite_direction: npc.sprite_dir,
        rotation: npc.rotation,
        life: npc.hp,
        life_max: npc.max_hp,
        width: w,
        height: h,
        scale: npc.scale,
        despawn_timer: npc.despawn_timer,
    }
}

/// Copy the machine's writes back onto the `Npc`. The position write is
/// kept (the mimic's one-time +8 nudge, `NPC.cs:25543`); the flyers'
/// integration happens in the machine caller below.
fn state_to_npc(npc: &mut Npc, s: &FrostMobState) {
    npc.x = s.x;
    npc.y = s.y;
    npc.vx = s.vx;
    npc.vy = s.vy;
    npc.ai = s.ai;
    npc.local_ai = s.local_ai;
    npc.facing = s.direction;
    npc.sprite_dir = s.sprite_direction;
    npc.rotation = s.rotation;
    npc.despawn_timer = s.despawn_timer;
}

/// The session player as the module's resolved target box (the 20×42
/// `Player` box, `player.rs`'s WIDTH/HEIGHT).
fn frost_target(player: &Player) -> FrostTarget {
    FrostTarget {
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH,
        height: crate::player::HEIGHT,
    }
}

/// `TargetClosest()` at the machine's C# call site: the shared
/// `npc_target::target_closest` scan over the single session candidate
/// (the zombie wiring's convention, `npc.rs::update_zombie`), then the
/// picked facing rides back into the module state's `direction`. The
/// `EverLastingTicker` stamp is 0 (the unported target-line fade's only
/// reader, the zombie precedent). `ai_style` is the machine's C# style.
fn retarget_state(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    s: &mut FrostMobState,
    ai_style: i32,
) {
    // Sync the machine's current position/velocity so the scan's centre
    // math reads the live box (the mimic's nudge moved it).
    npc.x = s.x;
    npc.y = s.y;
    npc.facing = s.direction;
    let cand = session_candidate(player);
    target_closest(npc, world, &[cand], true, ai_style, 0);
    s.direction = npc.facing;
}

/// One wired mob tick — the census's UNWIRED set, each machine at its
/// C# arm position with the caller's `UpdateNPCs` stream. Returns `true`
/// when the kind is claimed (the caller routes only the wired kinds).
///
/// RNG census (preserved from the module witnesses): Flocko draws
/// NOTHING (`NPC.cs:33971-34018` has no `Main.rand`); the mimic draws
/// NOTHING (`:25538-25630`); the walkers draw NOTHING (`:57403-57740`);
/// the copter draws exactly SIX on the firing tick (`:33960-33967`,
/// `Next(-35,36)` ×2 → `Next(-20,21)` ×2 → `Next(-20,21)` ×2) and zero
/// otherwise; the archer draws exactly TWO per fire/entry site
/// (`npc_ai_frost_mob_b.rs`'s header census). The transform
/// (`:57031-57034` → `Transform(349)`) draws NOTHING: `TransformVisuals`
/// (`NPC.cs:80963-81033`) has no 348→349 arm, `SetDefaults` is rand-free
/// and `TargetClosest` is the deterministic Manhattan scan.
pub(crate) fn frost_mob_step(
    npc: &mut Npc,
    world: &World,
    player: &mut Player,
    rng: &mut UnifiedRandom,
) -> bool {
    // ---- ElfArcher (350): the sibling shooter machine ----
    if npc.kind == NpcKind::ElfArcher {
        archer_step(npc, world, player, rng);
        return true;
    }
    let Some(kind) = wired_kind(npc.kind) else {
        return false;
    };
    let mut kind = kind;
    let mut s = state_from_npc(npc);
    let target = frost_target(player);
    match kind {
        // aiStyle 63 (`NPC.cs:33971-34018`): TargetClosest at `:33973`,
        // then the pure machine; noGravity + noTileCollide
        // (`SetDefaults :13483-13484`) → the raw integration.
        FrostMobKind::Flocko => {
            retarget_state(npc, world, player, &mut s, 63);
            flocko_ai(&mut s, &target, world.day_time);
            state_to_npc(npc, &s);
            npc.x += npc.vx;
            npc.y += npc.vy;
        }
        // aiStyle 62 (`NPC.cs:33922-33969`): TargetClosest at `:33924`;
        // the six shot draws ride `rng` in the C# order; the shot request
        // rides `wants_hostile_shot` (BulletDeadeye 180, 32 dmg,
        // KnockBack 0 — the sim's drain resolves it into the pool).
        FrostMobKind::ElfCopter => {
            retarget_state(npc, world, player, &mut s, 62);
            let mut shots = Vec::new();
            elf_copter_ai(&mut s, world, &target, world.day_time, rng, &mut shots);
            state_to_npc(npc, &s);
            for shot in shots {
                npc.wants_hostile_shot.push((
                    crate::projectile::PIRATE_ARROW,
                    shot.x,
                    shot.y,
                    shot.vx,
                    shot.vy,
                    shot.damage,
                ));
            }
            npc.x += npc.vx;
            npc.y += npc.vy;
        }
        // aiStyle 25 (`NPC.cs:25538-25630`): the retarget sites ride the
        // callback at their C# positions; the mimic is a grounded walker
        // (no noGravity/noTileCollide row, `:13314-13326`) → the shared
        // gravity + tile-collision pass after the machine.
        FrostMobKind::PresentMimic => {
            {
                let npc_ref: &mut Npc = npc;
                present_mimic_ai_retarget(
                    &mut s,
                    world,
                    &target,
                    world.snow_moon,
                    &mut |st| {
                        retarget_state(npc_ref, world, player, st, 25);
                    },
                );
            }
            state_to_npc(npc, &s);
            crate::npc_ai::walker_integrate(
                npc,
                world,
                WalkerParams {
                    speed: 3.5,
                    jump: -6.2,
                    aggro_x: 0.0,
                    aggro_y: 0.0,
                },
                GRAVITY,
                MAX_FALL,
            );
        }
        // aiStyle 3 (the Nutcracker twins): the transform gate FIRST
        // (`NPC.cs:57031-57034`, ahead of the velocity ladder), then
        // TargetClosest under the discouraged gate (`:57336`/`:57369`,
        // `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged`
        // `:60300-60330` — the frost terms: day-above-surface discourages
        // every type except the snowMoon-gated 343/350, neither of which
        // is here), then the per-type velocity arm, then the movement
        // tail. The shared stuck/jump core stays S1-D-3p's scope — the
        // port's collision jump (-6.2) is its documented stand-in.
        FrostMobKind::Nutcracker | FrostMobKind::NutcrackerSpinning => {
            // `:57031-57034` — the 55% transform (netMode != 1 always).
            if let Some(new_type) = nutcracker_transform(kind, npc.hp, npc.max_hp) {
                transform_nutcracker(npc, world, player, new_type);
                // Transform re-rolled the kind; re-read the kind + state.
                kind = wired_kind(npc.kind).expect("the transform stays in the wired set");
                s = state_from_npc(npc);
            }
            let discouraged = ai3_discouraged(npc, world);
            if !discouraged {
                retarget_state(npc, world, player, &mut s, 3);
            } else {
                // `:57375-57379` — the day-above-surface despawn nudge.
                if world.day_time && (npc.y as f64 / 16.0) < world.world_surface {
                    npc.encourage_despawn(10);
                    s.despawn_timer = npc.despawn_timer;
                }
            }
            if let Some((vx, vy)) =
                fighter_walk_velocity(kind, s.vx, s.vy, s.direction, s.scale)
            {
                s.vx = vx;
                s.vy = vy;
            }
            state_to_npc(npc, &s);
            crate::npc_ai::walker_integrate(
                npc,
                world,
                WalkerParams {
                    speed: 2.0,
                    jump: -6.2,
                    aggro_x: 0.0,
                    aggro_y: 0.0,
                },
                GRAVITY,
                MAX_FALL,
            );
        }
        // The ladder-owned walkers never reach this dispatch (their
        // `update_skeleton` arms stay); the mimic/flyers/twins above are
        // the whole wired set.
        _ => return false,
    }
    true
}

/// `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged`
/// (`NPC.cs:60300-60330`) resolved to the frost walker terms: the gate
/// is FALSE (discouraged) only on a non-eclipse day above the surface
/// for a non-statue NPC outside the graveyard; the snowMoon exemption
/// covers 343/350 only (neither rides this arm), the invasion/dontStarve
/// terms cannot fire for 348/349, and neither id is on the always-active
/// exclusion list. The graveyard/invasion fields have no port runtime
/// yet (the zombie wiring's convention — they read false).
fn ai3_discouraged(npc: &Npc, world: &World) -> bool {
    !world.eclipse
        && world.day_time
        && !npc.spawned_from_statue
        && !((npc.y as f64) > world.world_surface * 16.0)
}

/// The Nutcracker's `Transform(349)` (`NPC.cs:57031-57034` →
/// `Transform` `:81084-81141`): the stats re-roll to the spinning twin
/// (the same 28×76 box / 1800 lifeMax, `SetDefaults :13430-13443`), the
/// hp fraction preserved (`life = num * lifeMax / num2`, floored at 1),
/// the ai[] slots zeroed (the call's default ai0..ai3 = 0), the
/// `TargetClosest()` re-pick, and the frame reset. The C#'s
/// `position.Y += height` / `-= height` reposition pair is a no-op for
/// the equal-height twins; `TransformVisuals` has no 348→349 arm
/// (`:80963-81033`) so it draws NOTHING. `value == 0` stays 0 (the
/// `flag` guard) — both twins carry value 1500, so the guard never
/// fires here.
fn transform_nutcracker(npc: &mut Npc, world: &World, player: &Player, new_type: u16) {
    debug_assert_eq!(new_type, 349);
    let old_life = npc.hp;
    let old_max = npc.max_hp;
    npc.kind = NpcKind::NutcrackerSpinning;
    npc.max_hp = NpcKind::NutcrackerSpinning.max_hp();
    // `:81105-81109` — the hp fraction, floored at 1.
    let mut life = old_life * npc.max_hp / old_max.max(1);
    if life <= 0 {
        life = 1;
    }
    npc.hp = life;
    // `:81110-81113` — the ai[] slots reset to the call's defaults (0).
    npc.ai = [0.0; 4];
    // `SetDefaults`' frame reset (`:8529`) + the localAI reset
    // (`:8503-8506`); the knockBackResist row changes 0.4 → 0.1
    // (`:13426`/`:13440`) — the kind table reads it live.
    npc.local_ai = [0.0; 4];
    npc.frame = 0;
    npc.frame_counter = 0.0;
    // `:81101` — TargetClosest() after the re-roll.
    let cand = session_candidate(player);
    target_closest(npc, world, &[cand], true, 3, 0);
}

/// The ElfArcher (350) wiring — the sibling machine
/// (`npc_ai_frost_mob_b.rs::elf_archer_ai`) over the `Npc` fields. The
/// shooter block runs INSIDE `AI_003_Fighters` after the shared core;
/// the port runs it at the kind-arm position (the shared stuck/jump core
/// stays S1-D-3p). `justHit` folds onto `hurt_cd > 0` (the Poltergeist
/// precedent, `npc_ai_poltergeist.rs:71`); `confused` reads the buff
/// engine (buff 31, `NPC.cs:92029-92032`). The two-entry draws ride
/// `rng` in the module's witnessed order; the FlamingArrow 82 request
/// rides `wants_hostile_shot` (45 dmg, KnockBack 0).
fn archer_step(npc: &mut Npc, world: &World, player: &mut Player, rng: &mut UnifiedRandom) {
    let (w, h) = npc.size();
    let mut s = FrostMobBState {
        x: npc.x,
        y: npc.y,
        vx: npc.vx,
        vy: npc.vy,
        ai: npc.ai,
        local_ai: npc.local_ai,
        direction: npc.facing,
        sprite_direction: npc.sprite_dir,
        rotation: npc.rotation,
        life: npc.hp,
        life_max: npc.max_hp,
        width: w,
        height: h,
        scale: npc.scale,
        despawn_timer: npc.despawn_timer,
    };
    // The target seam: the C# `TargetClosest()` at `:59206` runs every
    // stopped tick (the callback below); the entry gates read the
    // resolved slot's dead/stealth/itemAnimation. `Player.stealth`
    // defaults 1f in C# (`Player.cs:1562`) and only the shroomite armour
    // drives it toward 0; the port's field is unwritten (starts 0, the
    // shroomite system unported), so the seam maps the C# default 1f
    // while `shroomite_stealth` is off — otherwise the `:59641` refusal
    // (`stealth == 0 && itemAnimation == 0`) would refuse a stationary
    // port player forever. The gate itself is carried verbatim, not
    // folded (ANALYSIS "The Frost Moon MOB wiring").
    let stealth = if player.shroomite_stealth { player.stealth } else { 1.0 };
    let target = FrostArcherTarget {
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH,
        height: crate::player::HEIGHT,
        dead: player.dead,
        stealth,
        item_animation: player.frame_inputs.swing_anim as i32,
    };
    let tick = ArcherTick {
        just_hit: npc.hurt_cd > 0.0,
        confused: npc.buffs.find_buff_index(31).is_some(),
    };
    let mut shots: Vec<FrostArcherShot> = Vec::new();
    {
        let npc_ref: &mut Npc = npc;
        elf_archer_ai_retarget(&mut s, world, &target, tick, rng, &mut shots, &mut |st| {
            npc_ref.x = st.x;
            npc_ref.y = st.y;
            npc_ref.facing = st.direction;
            let cand = session_candidate(player);
            target_closest(npc_ref, world, &[cand], true, 3, 0);
            st.direction = npc_ref.facing;
        });
    }
    npc.x = s.x;
    npc.y = s.y;
    npc.vx = s.vx;
    npc.vy = s.vy;
    npc.ai = s.ai;
    npc.local_ai = s.local_ai;
    npc.facing = s.direction;
    npc.sprite_dir = s.sprite_direction;
    npc.rotation = s.rotation;
    npc.despawn_timer = s.despawn_timer;
    for shot in shots {
        npc.wants_hostile_shot.push((
            crate::projectile::FLAMING_ARROW_HOSTILE,
            shot.x,
            shot.y,
            shot.vx,
            shot.vy,
            shot.damage,
        ));
    }
    // The movement tail: 350 is a grounded walker (no noGravity row,
    // `:13444-13456`); the shared AI_003 collision core's stand-in.
    crate::npc_ai::walker_integrate(
        npc,
        world,
        WalkerParams {
            speed: 1.0,
            jump: -6.2,
            aggro_x: 0.0,
            aggro_y: 0.0,
        },
        GRAVITY,
        MAX_FALL,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn target(x: f32, y: f32) -> FrostTarget {
        FrostTarget {
            x,
            y,
            width: 20.0,
            height: 42.0,
        }
    }

    // ---- Flocko (NPC.cs:33971-34018) ----

    /// `:33988-34003` — inside 200px the spin latch fires: ai[0] = 20,
    /// the direction follows the velocity sign, rotation += dir*0.3 and
    /// the velocity is untouched (the early return).
    #[test]
    fn flocko_spins_inside_200px() {
        let mut s = FrostMobState::new(FrostMobKind::Flocko, 1000.0, 600.0);
        s.vx = -2.0;
        s.vy = 1.0;
        s.ai[0] = 5.0;
        // centre (1027, 627), v120 (1047, 633); player centre (1110, 621)
        // → dist ≈ 64 < 200.
        flocko_ai(&mut s, &target(1100.0, 600.0), false);
        assert_eq!(s.ai[0], 20.0);
        assert_eq!(s.direction, -1);
        assert_eq!(s.rotation, -0.3);
        assert_eq!((s.vx, s.vy), (-2.0, 1.0));
    }

    /// `:33987` + `:34005-34017` — outside 200px with ai[0] <= 0 the
    /// chase runs all three lerp tiers (dist < 300) and the tilt is
    /// velocity.X * 0.15. The expected values replay the C# float ops.
    #[test]
    fn flocko_chases_with_the_three_lerp_tiers() {
        let mut s = FrostMobState::new(FrostMobKind::Flocko, 1000.0, 600.0);
        let t = target(1300.0, 600.0);
        // v120 = (1047, 633); aim = (1310-1047, 621-633) = (263, -12).
        let (mut ax, mut ay) = (263.0f32, -12.0f32);
        let dist = (ax * ax + ay * ay).sqrt();
        assert!(dist > 200.0 && dist < 300.0);
        let k = 11.0 / dist;
        ax *= k;
        ay *= k;
        let mut ex = (0.0f32 * 50.0 + ax) / 51.0;
        let mut ey = (0.0f32 * 50.0 + ay) / 51.0;
        ex = (ex * 10.0 + ax) / 11.0;
        ey = (ey * 10.0 + ay) / 11.0;
        ex = (ex * 7.0 + ax) / 8.0;
        ey = (ey * 7.0 + ay) / 8.0;
        flocko_ai(&mut s, &t, false);
        assert_eq!(s.ai[0], -1.0);
        assert_eq!(s.vx, ex);
        assert_eq!(s.vy, ey);
        assert_eq!(s.rotation, ex * 0.15);
    }

    /// `:33982-33986` — by day the aim reverses (the flee).
    #[test]
    fn flocko_reverses_the_aim_by_day() {
        let mut night = FrostMobState::new(FrostMobKind::Flocko, 1000.0, 600.0);
        let mut day = night;
        let t = target(1600.0, 600.0); // dist > 350: only the 50/51 tier
        flocko_ai(&mut night, &t, false);
        flocko_ai(&mut day, &t, true);
        assert_eq!(day.vx, -night.vx);
        assert_eq!(day.vy, -night.vy);
    }

    // ---- ElfCopter (NPC.cs:33922-33969) ----

    /// `:33936-33943` — the day flee: the 59/60 lerp AWAY from the aim
    /// and EncourageDespawn(10) (which only lowers the window).
    #[test]
    fn elf_copter_flees_and_despawns_by_day() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::ElfCopter, 1000.0, 600.0);
        s.despawn_timer = 600;
        let mut rng = UnifiedRandom::new(7);
        let mut shots = Vec::new();
        // centre (1025, 625), v119 (1045, 631); chase aim uses the
        // player's TOP: (1310-1045, 600-631) = (265, -31).
        let (mut ax, mut ay) = (265.0f32, -31.0f32);
        let k = 7.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        elf_copter_ai(
            &mut s,
            &w,
            &target(1300.0, 600.0),
            true,
            &mut rng,
            &mut shots,
        );
        assert_eq!(s.vx, (0.0f32 * 59.0 - ax) / 60.0);
        assert_eq!(s.vy, (0.0f32 * 59.0 - ay) / 60.0);
        assert_eq!(s.despawn_timer, 10);
        assert!(shots.is_empty());
        // EncourageDespawn never raises: a 5-tick window stays 5.
        s.despawn_timer = 5;
        elf_copter_ai(
            &mut s,
            &w,
            &target(1300.0, 600.0),
            true,
            &mut rng,
            &mut shots,
        );
        assert_eq!(s.despawn_timer, 5);
    }

    /// `:33944-33950` — beyond 600px the copter approaches (the +aim
    /// 59/60 lerp) and never shoots.
    #[test]
    fn elf_copter_approaches_beyond_600px() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::ElfCopter, 1000.0, 600.0);
        s.local_ai[0] = 14.0; // would fire if the hover arm ran
        let mut rng = UnifiedRandom::new(7);
        let mut shots = Vec::new();
        elf_copter_ai(
            &mut s,
            &w,
            &target(2000.0, 600.0),
            false,
            &mut rng,
            &mut shots,
        );
        assert!(s.vx > 0.0);
        assert!(shots.is_empty());
        assert_eq!(s.local_ai[0], 14.0);
    }

    /// `:33951-33967` — the RNG WITNESS: at the 15th slow hover tick the
    /// six draws fire in the C# order and the shot's velocity is the
    /// exact composition; the stream afterwards aligns with a manual
    /// replay of the same six draws.
    #[test]
    fn elf_copter_shot_consumes_six_draws_in_csharp_order() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::ElfCopter, 1000.0, 600.0);
        s.local_ai[0] = 14.0;
        let t = target(1300.0, 600.0);
        let mut rng = UnifiedRandom::new(12345);
        let mut shots = Vec::new();
        elf_copter_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
        assert_eq!(s.local_ai[0], 0.0);
        assert_eq!(shots.len(), 1);
        // The manual replay — the same seed, the same six draws.
        let mut probe = UnifiedRandom::new(12345);
        let (mut ax, mut ay) = (265.0f32, 621.0f32 - 631.0f32);
        ax += probe.next_range(-35, 36) as f32;
        ay += probe.next_range(-35, 36) as f32;
        ax *= 1.0 + probe.next_range(-20, 21) as f32 * 0.015;
        ay *= 1.0 + probe.next_range(-20, 21) as f32 * 0.015;
        let k = 10.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        let evx = ax * (1.0 + probe.next_range(-20, 21) as f32 * 0.0125);
        let evy = ay * (1.0 + probe.next_range(-20, 21) as f32 * 0.0125);
        assert_eq!(
            shots[0],
            FrostShot {
                kind: 180,
                x: 1045.0,
                y: 631.0,
                vx: evx,
                vy: evy,
                damage: 32
            }
        );
        // Stream alignment: the next draws match the replay's continuation.
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// The other half of the witness: every non-firing copter path
    /// consumes ZERO draws (the hover tick below the cadence).
    #[test]
    fn elf_copter_non_firing_ticks_leave_the_stream_untouched() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::ElfCopter, 1000.0, 600.0);
        let mut rng = UnifiedRandom::new(999);
        let mut probe = UnifiedRandom::new(999);
        let mut shots = Vec::new();
        for _ in 0..14 {
            elf_copter_ai(
                &mut s,
                &w,
                &target(1300.0, 600.0),
                false,
                &mut rng,
                &mut shots,
            );
        }
        assert!(shots.is_empty());
        assert_eq!(s.local_ai[0], 14.0);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    // ---- PresentMimic (NPC.cs:25538-25630) ----

    /// `:25541-25561` — the first tick nudges X by 8 and the 341 override
    /// forces ai[3] = 1 regardless of the layer pick.
    #[test]
    fn present_mimic_first_tick_nudges_and_forces_ai3() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::PresentMimic, 1000.0, 600.0);
        present_mimic_ai(&mut s, &w, &target(5000.0, 600.0), true);
        assert_eq!(s.x, 1008.0);
        assert_eq!(s.ai[3], 1.0);
    }

    /// `:25562-25584` — the idle wakes on movement, on the inflated
    /// overlap, or on damage; a far untouched present stays disguised.
    #[test]
    fn present_mimic_idle_wake_conditions() {
        let w = air();
        let far = target(5000.0, 600.0);
        // Untouched: stays asleep.
        let mut s = FrostMobState::new(FrostMobKind::PresentMimic, 1000.0, 600.0);
        s.ai[3] = 1.0;
        present_mimic_ai(&mut s, &w, &far, true);
        assert_eq!(s.ai[0], 0.0);
        // vy > 0.3 wakes.
        let mut s2 = s;
        s2.vy = 0.4;
        present_mimic_ai(&mut s2, &w, &far, true);
        assert_eq!(s2.ai[0], 1.0);
        // The ±100 overlap wakes (player inside the inflated box).
        let mut s3 = s;
        present_mimic_ai(&mut s3, &w, &target(1050.0, 600.0), true);
        assert_eq!(s3.ai[0], 1.0);
        // Damage wakes.
        let mut s4 = s;
        s4.life -= 1;
        present_mimic_ai(&mut s4, &w, &far, true);
        assert_eq!(s4.ai[0], 1.0);
    }

    /// `:25585-25619` — the hop cycle: 12 grounded ticks then the
    /// (3.5, -4) hop, 20 more then the (2.5, -8) hop with ai[1] reset.
    #[test]
    fn present_mimic_alternates_the_two_hops() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::PresentMimic, 1000.0, 600.0);
        s.ai[0] = 1.0;
        s.ai[3] = 1.0;
        s.ai[2] = 11.0; // the 12th tick fires the first hop
        present_mimic_ai(&mut s, &w, &target(5000.0, 600.0), true);
        assert_eq!((s.vx, s.vy), (3.5, -4.0));
        assert_eq!(s.ai[1], 1.0);
        assert_eq!(s.ai[2], 0.0);
        assert_eq!(s.sprite_direction, 1);
        // Back on the ground: 19 ticks brake (vx *= 0.9), the 20th hops.
        s.vy = 0.0;
        for i in 0..19 {
            let before = s.vx;
            present_mimic_ai(&mut s, &w, &target(5000.0, 600.0), true);
            assert_eq!(s.vx, before * 0.9, "brake tick {i}");
            assert_eq!(s.vy, 0.0);
        }
        present_mimic_ai(&mut s, &w, &target(5000.0, 600.0), true);
        assert_eq!((s.vx, s.vy), (2.5, -8.0));
        assert_eq!(s.ai[1], 0.0);
    }

    /// `:25622-25629` — the airborne ±0.1 drift toward ±1.
    #[test]
    fn present_mimic_airborne_drift() {
        let w = air();
        let mut s = FrostMobState::new(FrostMobKind::PresentMimic, 1000.0, 600.0);
        s.ai[0] = 1.0;
        s.ai[3] = 1.0;
        s.vy = -4.0;
        s.vx = 0.5;
        present_mimic_ai(&mut s, &w, &target(5000.0, 600.0), true);
        assert_eq!(s.vx, 0.6);
        s.direction = -1;
        s.vx = -0.5;
        present_mimic_ai(&mut s, &w, &target(5000.0, 600.0), true);
        assert_eq!(s.vx, -0.6);
    }

    // ---- AI_003 walk arms ----

    /// `:57637-57648` — the elf caps 1.75 / 1.25 / 2f with the 0.07 accel.
    #[test]
    fn zombie_elf_walk_caps_match_the_csharp_rows() {
        for (kind, cap) in [
            (FrostMobKind::ZombieElf, 1.75f32),
            (FrostMobKind::ZombieElfBeard, 1.25),
            (FrostMobKind::ZombieElfGirl, 2.0),
        ] {
            let (mut vx, mut vy) = (0.0f32, 0.0f32);
            for _ in 0..200 {
                (vx, vy) = fighter_walk_velocity(kind, vx, vy, 1, 1.0).unwrap();
            }
            assert_eq!(vx, cap, "{kind:?}");
            assert_eq!(vy, 0.0);
            // One accel step from rest is exactly 0.07.
            let (one, _) = fighter_walk_velocity(kind, 0.0, 0.0, 1, 1.0).unwrap();
            assert_eq!(one, 0.07);
        }
    }

    /// `:57693-57696` — the GingerbreadMan's cap scales with (1-scale).
    #[test]
    fn gingerbread_man_cap_rides_the_scale_term() {
        let scale = 0.9f32;
        let cap = 1.5 * (1.0 + (1.0 - scale));
        let (mut vx, mut vy) = (0.0f32, 0.0f32);
        for _ in 0..200 {
            (vx, vy) =
                fighter_walk_velocity(FrostMobKind::GingerbreadMan, vx, vy, 1, scale).unwrap();
        }
        assert_eq!(vx, cap);
    }

    /// `:57554-57578` — Yeti/Nutcracker/Krampus share the cap-2 arm;
    /// an over-cap grounded velocity damps BOTH components by 0.8.
    #[test]
    fn the_cap2_group_walks_at_2_and_damps_overspeed() {
        for kind in [
            FrostMobKind::Yeti,
            FrostMobKind::Nutcracker,
            FrostMobKind::Krampus,
        ] {
            let (mut vx, mut vy) = (0.0f32, 0.0f32);
            for _ in 0..200 {
                (vx, vy) = fighter_walk_velocity(kind, vx, vy, 1, 1.0).unwrap();
            }
            assert_eq!(vx, 2.0, "{kind:?}");
            // Grounded (vy == 0) overspeed damps BOTH components by 0.8
            // (`:57556-57561`).
            let (ox, oy) = fighter_walk_velocity(kind, 3.0, 0.0, 1, 1.0).unwrap();
            assert_eq!((ox, oy), (3.0 * 0.8, 0.0));
            // Airborne (vy != 0) overspeed does NOT damp — and the cap
            // arms are skipped entirely, so the velocity is untouched.
            let (ax, ay) = fighter_walk_velocity(kind, 3.0, 1.5, 1, 1.0).unwrap();
            assert_eq!((ax, ay), (3.0, 1.5));
        }
    }

    /// `:57409-57439` — the spinning Nutcracker: cap 6, and the grounded
    /// counter-drift brake (×0.99) before the 0.07 accel.
    #[test]
    fn nutcracker_spinning_cap6_with_the_counter_brake() {
        let (mut vx, mut vy) = (0.0f32, 0.0f32);
        for _ in 0..300 {
            (vx, vy) =
                fighter_walk_velocity(FrostMobKind::NutcrackerSpinning, vx, vy, 1, 1.0).unwrap();
        }
        assert_eq!(vx, 6.0);
        // Moving left while facing right, on the ground: 0.99 then +0.07.
        let (bx, _) =
            fighter_walk_velocity(FrostMobKind::NutcrackerSpinning, -1.0, 0.0, 1, 1.0).unwrap();
        assert_eq!(bx, -1.0 * 0.99 + 0.07);
        // Airborne: no brake.
        let (ax, _) =
            fighter_walk_velocity(FrostMobKind::NutcrackerSpinning, -1.0, -2.0, 1, 1.0).unwrap();
        assert_eq!(ax, -1.0 + 0.07);
    }

    /// `:57031-57034` — the 55% transform gate (the `(double)` compare).
    #[test]
    fn nutcracker_transforms_at_55_percent() {
        assert_eq!(
            nutcracker_transform(FrostMobKind::Nutcracker, 990, 1800),
            Some(349)
        );
        assert_eq!(
            nutcracker_transform(FrostMobKind::Nutcracker, 991, 1800),
            None
        );
        assert_eq!(
            nutcracker_transform(FrostMobKind::NutcrackerSpinning, 100, 1800),
            None
        );
    }

    // ---- the dispatch seam ----

    /// The seam claims exactly the non-boss set; the bosses 344-346
    /// (worker C) and every other id fall through.
    #[test]
    fn dispatch_claims_the_non_boss_frost_set_only() {
        let w = air();
        let t = target(1300.0, 600.0);
        for id in [
            338u16, 339, 340, 341, 342, 343, 347, 348, 349, 350, 351, 352,
        ] {
            let kind = FrostMobKind::from_type(id).unwrap();
            let mut s = FrostMobState::new(kind, 1000.0, 600.0);
            let mut rng = UnifiedRandom::new(1);
            let mut shots = Vec::new();
            assert!(
                frost_mob_ai_step(id, &mut s, &w, &t, &mut rng, &mut shots),
                "{id}"
            );
        }
        for id in [0u16, 3, 305, 344, 345, 346, 353, 670] {
            let mut s = FrostMobState::new(FrostMobKind::Flocko, 1000.0, 600.0);
            let mut rng = UnifiedRandom::new(1);
            let mut shots = Vec::new();
            assert!(
                !frost_mob_ai_step(id, &mut s, &w, &t, &mut rng, &mut shots),
                "{id}"
            );
        }
    }

    /// The RNG-order witness across the whole seam: a mixed tick sequence
    /// (walkers + mimic + flocko + a firing copter) consumes exactly the
    /// copter's six draws — the same stream a copter-only replay produces.
    #[test]
    fn the_seam_preserves_the_update_stream_order() {
        let mut w = air();
        // The copter only shoots at NIGHT (`:33936` returns by day);
        // World::new starts at day.
        w.day_time = false;
        let t = target(1300.0, 600.0);
        let mut rng = UnifiedRandom::new(4242);
        let mut shots = Vec::new();
        // Non-copter ticks first — none may draw.
        for id in [338u16, 343, 351, 341, 352] {
            let kind = FrostMobKind::from_type(id).unwrap();
            let mut s = FrostMobState::new(kind, 1000.0, 600.0);
            s.ai[0] = 1.0;
            s.ai[3] = 1.0;
            frost_mob_ai_step(id, &mut s, &w, &t, &mut rng, &mut shots);
        }
        assert!(shots.is_empty());
        let mut probe = UnifiedRandom::new(4242);
        for _ in 0..8 {
            assert_eq!(
                rng.next(),
                probe.next(),
                "non-copter ticks drew from the stream"
            );
        }
        // Now the firing copter on a fresh pair of streams.
        let mut rng2 = UnifiedRandom::new(77);
        let mut probe2 = UnifiedRandom::new(77);
        let mut s = FrostMobState::new(FrostMobKind::ElfCopter, 1000.0, 600.0);
        s.local_ai[0] = 14.0;
        frost_mob_ai_step(347, &mut s, &w, &t, &mut rng2, &mut shots);
        assert_eq!(shots.len(), 1);
        // Replay the six draws manually.
        for (min, max) in [
            (-35, 36),
            (-35, 36),
            (-20, 21),
            (-20, 21),
            (-20, 21),
            (-20, 21),
        ] {
            probe2.next_range(min, max);
        }
        for _ in 0..8 {
            assert_eq!(rng2.next(), probe2.next());
        }
    }

    /// The aiStyle table matches the SetDefaults lines (the dispatch's
    /// gate axis).
    #[test]
    fn the_ai_style_table_matches_set_defaults() {
        for (id, style) in [
            (338u16, 3u8),
            (339, 3),
            (340, 3),
            (341, 25),
            (342, 3),
            (343, 3),
            (347, 62),
            (348, 3),
            (349, 3),
            (350, 3),
            (351, 3),
            (352, 63),
        ] {
            let k = FrostMobKind::from_type(id).unwrap();
            assert_eq!(k.ai_style(), style, "{id}");
            assert_eq!(k.type_id(), id);
        }
    }

    // =================================================================
    // S1-D-3 W-6 wiring witnesses — WRITTEN, NOT RUN (code-only mode).
    // These exercise the LIVE `frost_mob_step` seam over `Npc` (the
    // copy-in/out + the C#-position TargetClosest/shot/transform seams),
    // complementing the pure-machine witnesses above. Every expected
    // value is hand-derived from the cited C# lines; the RNG argument is
    // by construction (the machines' draw censuses are preserved).
    // =================================================================

    /// A night world with a ground strip (the walkers/mimic integrate onto
    /// it; the flyers ignore it). `snow_moon` on so the mimic re-targets.
    fn wired_world() -> World {
        let mut w = World::new("t".into(), 200, 120);
        for x in 0..200 {
            for y in 100..120 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.day_time = false;
        w.snow_moon = true;
        w
    }

    fn wired_player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 120.0 * TILE_PX;
        p.y = 95.0 * TILE_PX;
        p
    }

    fn wired_npc(kind: NpcKind, x_tiles: f32, y_tiles: f32) -> Npc {
        let mut n = Npc::new(kind, x_tiles * TILE_PX, y_tiles * TILE_PX, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// The wiring claims exactly the census's UNWIRED mob set; the bosses
    /// (R41) and the ladder walkers fall through (`false`).
    #[test]
    fn s1_d_3_w6_the_seam_claims_the_unwired_mob_set_only() {
        let w = wired_world();
        let mut p = wired_player(&w);
        let mut rng = UnifiedRandom::new(1);
        for kind in [
            NpcKind::PresentMimic,
            NpcKind::ElfCopter,
            NpcKind::Nutcracker,
            NpcKind::NutcrackerSpinning,
            NpcKind::ElfArcher,
            NpcKind::Flocko,
        ] {
            let mut n = wired_npc(kind, 60.0, 60.0);
            assert!(frost_mob_step(&mut n, &w, &mut p, &mut rng), "{kind:?}");
        }
        for kind in [
            NpcKind::Everscream,
            NpcKind::IceQueen,
            NpcKind::SantaNK1,
            NpcKind::ZombieElf,
            NpcKind::Yeti,
            NpcKind::Krampus,
        ] {
            let mut n = wired_npc(kind, 60.0, 60.0);
            assert!(!frost_mob_step(&mut n, &w, &mut p, &mut rng), "{kind:?}");
        }
    }

    /// Flocko (352) through the seam: the spin latch inside 200px
    /// (`NPC.cs:33988-34003`), the velocity untouched, and ZERO draws
    /// (the aiStyle-63 arm has no `Main.rand`).
    #[test]
    fn s1_d_3_w6_flocko_spins_through_the_seam_draw_free() {
        let w = wired_world();
        let mut p = wired_player(&w);
        // The player within 200px of the Flocko's aim point so the spin
        // latch fires: Flocko at (960, 960), centre (987, 987), aim point
        // (967, 993) (facing -1 after the latch); player centre (1002, 981)
        // → dist ≈ 37 < 200.
        p.x = 62.0 * TILE_PX;
        p.y = 60.0 * TILE_PX;
        let mut n = wired_npc(NpcKind::Flocko, 60.0, 60.0);
        n.vx = -2.0;
        n.vy = 1.0;
        n.ai[0] = 5.0;
        let mut rng = UnifiedRandom::new(9);
        let mut probe = UnifiedRandom::new(9);
        frost_mob_step(&mut n, &w, &mut p, &mut rng);
        assert_eq!(n.ai[0], 20.0, "the <200px latch re-arms (`:33990`)");
        assert_eq!(n.facing, -1, "the direction follows vx's sign (`:33993`)");
        // The spin early-returns before the chase: velocity unchanged,
        // then the noTileCollide integration adds it to the position.
        assert_eq!(n.vx, -2.0);
        assert_eq!(n.vy, 1.0);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next(), "Flocko draws nothing");
        }
    }

    /// ElfCopter (347) through the seam: at the 15th slow hover tick the
    /// six draws fire in the C# order and ONE BulletDeadeye-180 request
    /// lands on `wants_hostile_shot` (32 dmg); the stream afterwards
    /// aligns with a manual replay of the same six draws.
    #[test]
    fn s1_d_3_w6_copter_shot_consumes_six_draws_and_requests_180() {
        let w = wired_world();
        let mut p = wired_player(&w);
        // Copter at (1000, 600), player at (1300, 600): dist ≈ 300 < 600,
        // LOS clear (open air above the ground strip) → the hover arm.
        p.x = 1300.0;
        p.y = 600.0;
        let mut n = wired_npc(NpcKind::ElfCopter, 60.0, 60.0);
        n.x = 1000.0;
        n.y = 600.0;
        n.local_ai[0] = 14.0; // the next slow tick fires
        let mut rng = UnifiedRandom::new(12345);
        frost_mob_step(&mut n, &w, &mut p, &mut rng);
        assert_eq!(n.local_ai[0], 0.0, "the cadence resets (`:33957`)");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, _sx, _sy, _svx, _svy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::PIRATE_ARROW, "Type 180 (`:33967`)");
        assert_eq!(dmg, 32, "Damage 32 (`:33967`)");
        // The manual replay — the same seed, the same six draws.
        let mut probe = UnifiedRandom::new(12345);
        for (min, max) in [(-35, 36), (-35, 36), (-20, 21), (-20, 21), (-20, 21), (-20, 21)] {
            probe.next_range(min, max);
        }
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next(), "the six-draw order holds");
        }
    }

    /// The copter's non-firing ticks consume ZERO draws (the hover tick
    /// below the cadence) — the other half of the witness.
    #[test]
    fn s1_d_3_w6_copter_non_firing_ticks_leave_the_stream_untouched() {
        let w = wired_world();
        let mut p = wired_player(&w);
        p.x = 1300.0;
        p.y = 600.0;
        let mut n = wired_npc(NpcKind::ElfCopter, 60.0, 60.0);
        let mut rng = UnifiedRandom::new(999);
        let mut probe = UnifiedRandom::new(999);
        for _ in 0..14 {
            // Re-anchor each tick: the hover arm damps the velocity and the
            // integration drifts the copter; keep it in the hover window so
            // the cadence counts without firing.
            n.x = 1000.0;
            n.y = 600.0;
            n.vx = 0.0;
            n.vy = 0.0;
            frost_mob_step(&mut n, &w, &mut p, &mut rng);
        }
        assert!(n.wants_hostile_shot.is_empty());
        assert_eq!(n.local_ai[0], 14.0);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// PresentMimic (341) through the seam: the first tick nudges X by 8
    /// and forces ai[3] = 1 (`NPC.cs:25543`/`:25558-25561`); the hop cycle
    /// launches the alternating (3.5, -4) hop after 12 grounded ticks.
    /// ZERO draws (the aiStyle-25 arm has no `Main.rand`).
    #[test]
    fn s1_d_3_w6_mimic_nudges_and_hops_through_the_seam() {
        let w = wired_world();
        let mut p = wired_player(&w);
        // The player FAR away so the ±100px wake overlap never fires — the
        // mimic stays disguised until we wake it by hand.
        p.x = 5000.0;
        p.y = 600.0;
        let mut n = wired_npc(NpcKind::PresentMimic, 60.0, 95.0);
        let x0 = n.x;
        let mut rng = UnifiedRandom::new(5);
        let mut probe = UnifiedRandom::new(5);
        frost_mob_step(&mut n, &w, &mut p, &mut rng);
        assert_eq!(n.ai[3], 1.0, "the 341 override (`:25560`)");
        assert_eq!(n.ai[0], 0.0, "the far untouched present stays disguised");
        // The +8 nudge (`:25543`) then the gravity integration's first fall.
        assert!(n.x >= x0 + 8.0, "the one-time +8 nudge (`:25543`)");
        // Wake it and arm the hop cadence: the 12th grounded tick fires.
        n.ai[0] = 1.0;
        n.ai[2] = 11.0;
        n.vx = 0.0;
        n.vy = 0.0;
        n.on_ground = true;
        frost_mob_step(&mut n, &w, &mut p, &mut rng);
        assert!(n.vy < 0.0, "the 12th tick launches the hop (`:25608-25619`)");
        assert_eq!(n.ai[1], 1.0, "the first hop increments ai[1] (`:25608`)");
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next(), "the mimic draws nothing");
        }
    }

    /// The Nutcracker's 55% transform through the seam (`NPC.cs:57031-57034`
    /// → `Transform(349)` `:81084-81141`): the kind swaps, the hp fraction
    /// is preserved, the ai[] slots zero, and the transform draws NOTHING.
    #[test]
    fn s1_d_3_w6_nutcracker_transforms_at_55_percent_through_the_seam() {
        let w = wired_world();
        let mut p = wired_player(&w);
        let mut n = wired_npc(NpcKind::Nutcracker, 60.0, 90.0);
        n.hp = 990; // 990 <= 1800 * 0.55 = 990 → the gate fires
        n.ai = [7.0, 7.0, 7.0, 7.0];
        let mut rng = UnifiedRandom::new(3);
        let mut probe = UnifiedRandom::new(3);
        frost_mob_step(&mut n, &w, &mut p, &mut rng);
        assert_eq!(n.kind, NpcKind::NutcrackerSpinning, "Transform(349)");
        assert_eq!(n.hp, 990, "the hp fraction preserves (1800/1800)");
        assert_eq!(n.ai, [0.0; 4], "the ai[] slots reset (`:81110-81113`)");
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next(), "the transform draws nothing");
        }
        // Above the gate: no transform.
        let mut n2 = wired_npc(NpcKind::Nutcracker, 60.0, 90.0);
        n2.hp = 991;
        frost_mob_step(&mut n2, &w, &mut p, &mut rng);
        assert_eq!(n2.kind, NpcKind::Nutcracker, "991 > 990 stays");
    }

    /// ElfArcher (350) through the seam: the aggression entry stops the
    /// walk and latches the 110-tick cycle (`NPC.cs:59683-59688`), drawing
    /// exactly two `Next(-40, 41)`; the fire at ai[1] == 55 requests ONE
    /// FlamingArrow-82 (45 dmg).
    #[test]
    fn s1_d_3_w6_archer_entry_and_fire_through_the_seam() {
        let w = wired_world();
        let mut p = wired_player(&w);
        // Archer at (960, 1520) falls onto the ground strip (top y=1600,
        // the 40px box rests at y=1560); the player at (1280, 1520) stands
        // 320px to the right — inside the 700px aggro gate (`:59655`), LOS
        // clear above the ground.
        p.x = 1280.0;
        p.y = 1520.0;
        let mut n = wired_npc(NpcKind::ElfArcher, 60.0, 95.0);
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(4242);
        // The settle ticks land the archer (vy == 0), then the entry
        // latches the cycle.
        for _ in 0..30 {
            frost_mob_step(&mut n, &w, &mut p, &mut rng);
            if n.ai[1] == 110.0 {
                break;
            }
        }
        assert_eq!(n.ai[1], 110.0, "the entry latches the cycle (`:59688`)");
        assert!(n.ai[2] > 0.0, "the octant latches (`:59689-59711`)");
        // Run to the fire tick: the decrement walks 110 → 55, the fire
        // lands on the tick that READS 56.
        let mut fired = false;
        for _ in 0..60 {
            frost_mob_step(&mut n, &w, &mut p, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                fired = true;
                break;
            }
        }
        assert!(fired, "the fire tick requests the shot (`:59592`)");
        let (kind, _sx, _sy, _svx, _svy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::FLAMING_ARROW_HOSTILE, "num170 = 82");
        assert_eq!(dmg, 45, "num169 = 45 (`:59416-59419`)");
    }
}
