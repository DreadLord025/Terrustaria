//! The aiStyle-75 family — the mount/rider machine (`NPC.AI()`'s
//! `else if (this.aiStyle == 75)` arm, `NPC.cs:35501-36070`). S1-D-3,
//! aiStyle-75 round, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! ## Census
//!
//! `this.aiStyle = 75;` across `NPC.cs` returns exactly SIX hits — the
//! family is:
//! * ScutlixRider **390** (`SetDefaults` `NPC.cs:13966-13977`: 18×40,
//!   dmg 65, def 30, 350 HP, value 1200, npcSlots 0.5),
//! * MartianSaucer **392**, the shell (`:13993-14010`: 150×80, dmg 50,
//!   def 100, 100 HP, kb 0, noGravity/noTileCollide, dontTakeDamage,
//!   netAlways),
//! * MartianSaucerTurret **393** (`:14011-14027`: 40×16, dmg 60, def 20,
//!   5000 HP, npcSlots 1),
//! * MartianSaucerCannon **394** (`:14028-14044`: 46×36, dmg 60, def 20,
//!   3500 HP),
//! * FlyingDutchmanCannon **492** (`:14648-14665`: 30×30, dmg 30, def 20,
//!   2000 HP, `hide = true`),
//! * DrakomireRider **416** (`:15331-15342`: 18×40, dmg 80, def 28,
//!   800 HP, kb 0.2).
//!
//! The saucer CORE 395 rides aiStyle **76** (`:14045+`, dispatch
//! `:36071`) — out of this arm's scope (the port's `update_saucer_core`).
//! The `Projectile.cs` `aiStyle = 75` rows are the projectile space
//! (S1-D-4's territory), not NPC kinds. `npc/data.rs:1994-1995` already
//! claims style 75 for exactly these six port kinds.
//!
//! ## The machine
//!
//! Every member is a RIDER: it resolves a host slot from `ai[0]`
//! (`(int)this.ai[0]` → `Main.npc[slot]`), and when the host is alive
//! with the right type it snaps onto the host's centre with a per-kind
//! offset (`:35659-35676` — INTEGER half-extents, `Entity.width/height`
//! are `int`), mirrors `gfxOffY`/`direction`/`spriteDirection`, and runs
//! its own weapon block off the host's state. The 390/416 riders spawn
//! their own carrier on the first tick (`localAI[0]` latch, `:35508-
//! 35514`/`:35525-35531`); when the carrier is gone they `Transform`
//! (390 → 382 RayGunner, 416 → 518 SolarSpearman, `:36056-36063`); any
//! other member with a dead host dies (`life = 0` + `HitEffect` +
//! deactivate, `:36064-36069`).
//!
//! Per-kind weapon blocks (all inside the `num1055 != -1` ride):
//! * **390** (`:35677-35753`): the `timeLeft`/`velocity`/`target`
//!   mirrors, the 60/−60 fire cycle (`justHit` → −30), the CanHit +
//!   700-px + direction-match fire gate — the 438 ray (30 dmg, 7 speed)
//!   from `Center − UnitY·4` with the ±50 / ×(0.8..1.2) aim jitters —
//!   else the idle octant (`ai[2]` 1..5) with the ×0.5 X damp.
//! * **492** (`:35754-35822`): the 240-tick reload (`ai[3]`), the
//!   `ai[3] == 2` one-shot `TargetClosest(faceTarget: false)`, the
//!   CanHit-gated 240 shell (×14 aim, −5 Y) and the eight-ray closest-
//!   probe aim (`ai[2]` 1..8, the `spriteDirection == 1` → `9 − x`
//!   flip), no-sight → `ai[2] = 0`.
//! * **394** (`:35823-35850`): the core's `ai[3]` mirror, the 440..580
//!   window every 20 ticks → the 448 missile (8 speed, ±π/8 jitter,
//!   `side·36 + 8Y` muzzle, `ai1 = 20` arming delay) + Item39.
//! * **393** (`:35851-35894`): the 280..420 window every 6 ticks → the
//!   449 ray (16 speed, ±π/24 jitter around `ai[2]`) + Item12; outside
//!   the window a per-tick `TargetClosest(false)` + the Y-clamped aim
//!   rotation.
//! * **392** (`:35895-36054`): the phase-0 20..260 window's entry-tick
//!   447 deathray (`ai0 = whoAmI + 1`) + Item12; the missile battery —
//!   four emitters gated by the crew slots' aliveness (`localAI[0..3]`),
//!   each rolling `Next(maxValue4/5)` with the phase/window table
//!   (phase 2 → 120/120, 280..400 → 60/90, 440..580 → 90/60, else
//!   1000/1000); the four dead-crew dust emitters (`Next(8)` gates,
//!   dust 31/228). The mount half (`:35540-35614`) also runs the PHASE
//!   BRIDGE: crew all dead + the core in `[0, 1)` → the core's `ai[0]`
//!   = 3 (classic) / 1 (expert), and the `ai[3] % 200 == 0` crew
//!   netUpdate marks.
//!
//! ## RNG order (by construction)
//!
//! Every `Main.rand` draw the arm makes is consumed here, in C# textual
//! order, on the caller's stream:
//! 1. 390 fire (`:35705-35708`): `Next(-50,51)` ×2 then
//!    `Next(80,121)` ×2 — four draws, only on the fire tick.
//! 2. 394 shot (`:35842`): ONE `NextDouble` (the ±π/8 jitter).
//! 3. 393 shot (`:35886`): ONE `NextDouble` (the ±π/24 jitter).
//! 4. 392 battery (`:35978/:35989/:36004/:36015`): one
//!    `Next(maxValue4/5)` per LIVE-flag emitter, in the order
//!    turret-left, turret-right, cannon-left, cannon-right; a hit then
//!    draws `NextDouble` (velocity X), `NextDouble` (rotation),
//!    `Next(50,70)`/`Next(30,60)` (muzzle X), `Next(30,45)`/
//!    `Next(-30,-10)` (muzzle Y) — `:35982-35984` etc.
//! 5. 392 dust (`:36027-36054`): one `Next(8)` per dead-crew flag; a
//!    hit draws `Next(50,70)`/`Next(30,60)` + `Next(15,30)`/
//!    `Next(-30,-10)` (position), `Next(4)` (the 31-vs-228 pick), the
//!    `Dust.NewDust` internals (consumed by the REAL
//!    `DustPool::new_dust`, `Dust.cs:75-256` — including the type-228
//!    `Next(-10,6)` tweak), then the velocity `NextDouble` pair
//!    (`:36030` etc). The velocity writes land on the returned slot
//!    EVEN when it is the 6000 sentinel (`Main.dust[6000]` — the C#
//!    writes it and the draws happen regardless).
//!
//! The arm's `SoundEngine.PlaySound(Item12/Item39)` cues ride
//! [`Style75Out::sounds`] — the audio seam resolves each audible play's
//! ONE `Next(-6,7)` pitch draw through `sound::item_use_case2`
//! (`LegacySoundPlayer.cs:632`, the style10 `play_item8_sound`
//! precedent); the wiring caller must drain them on the NPC stage's
//! stream at the C# call position. `TargetClosest` is the deterministic
//! Manhattan scan (`npc_target::target_closest`, S1-D-3p) — no draws.
//! `NewNPC`/`Transform`/`DiscourageDespawn` draw nothing.
//!
//! ## Wiring seams (WIRED — `task/S1-D-3-wire-saucer`, code-only)
//!
//! The live `npc.rs::update_scutlix_rider` / `update_saucer_mount` /
//! `update_solar_drakomire_rider` stand-ins are RETIRED; the aiStyle-75
//! axis claim routes the six kinds to this `style75_step`: the SIM runs
//! it as a pooled arm in `world_sim/npcs.rs`'s tick closure over the
//! slot-aligned `App::npcs` on the `UpdateNPCs` stream (draining
//! `Style75Out` there), and a bare `update_rng` caller rides
//! `Npc::update_rng_style75_solo` over a one-slot pool. The carrier-link
//! pre-pass writes the slot-aligned `ai[0]` host links + the shell's
//! `local_ai[0..3]` crew slots each tick (the port's nearest-living-kind
//! stand-in for the C# index links).
//! Seams:
//! * The pool is slot-aligned (`npcs[me]` is `Main.npc[whoAmI]`, the
//!   `npc_ai_style37.rs` convention); `ai[0]`/`localAI[i]` index it
//!   directly. The live sim's uid links are the wiring caller's
//!   translation. `active` ≈ `hp > 0` (the style37 orphan stand-in).
//! * `Main.netMode` is the single-player 0: every `netMode != 1` arm is
//!   active, the `netMode == 1` client skips never taken.
//! * The 390/416 first-tick carrier spawn rides
//!   [`Style75Out::spawn_carrier`] (the `wants_scutlix` shape): the C#
//!   writes `ai[0] = NewNPC(...)` synchronously and rides the SAME
//!   tick; the port's caller allocates after the tick, so the spawn
//!   tick returns early (no ride, NO Transform fallback) — a documented
//!   one-tick fold. `NewNPC` places by `Bottom = (X, Y)`
//!   (`NPC.cs:80770`).
//! * `Transform(382/518)` rides [`Style75Out::transform`]: the caller
//!   runs `NPC.Transform` (`NPC.cs:81084-81114` — the bottom-anchored
//!   reposition, `SetDefaults`, `TargetClosest`, the
//!   `life·lifeMax/oldLifeMax` rescale floored at 1, the ai clears).
//! * `DiscourageDespawn(60)` (`NPC.cs:7258-7265`) maps to
//!   `despawn_timer < 60 → 60`; the `despawnEncouraged = false` half
//!   has no port field (the port's `despawn_timer` is the only
//!   despawn state). The `timeLeft = host.timeLeft` mirrors copy
//!   `despawn_timer` raw.
//! * The orphan death sets `hp = 0` + [`Style75Out::died`]; the caller
//!   owes the `HitEffect()` + deactivation pair (the style72 pop
//!   precedent).
//!
//! See docs/ANALYSIS.md "The mount machine (S1-D-3 aiStyle-75, code-only)"
//! and "The saucer wiring (S1-D-3-wire-saucer, code-only)".

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{HostileShotAi, Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};

/// The arm's per-tick inputs the port's `Npc` does not carry.
#[derive(Clone, Copy, Debug)]
pub struct Style75Ctx {
    /// `NPC.difficulty` (`NPC.cs:6535`, 1 = Classic, 2 = Expert,
    /// 3 = Master; `GameDifficultyLevel.cs:17-21`).
    pub difficulty: f32,
    /// `Main.expertMode` — the 392 phase bridge's expert arm
    /// (`NPC.cs:35596-35611`).
    pub expert: bool,
    /// `this.justHit` — the 390 fire-cycle interrupt (`:35686-35689`).
    pub just_hit: bool,
    /// `Main.EverLastingTicker` for the `TargetClosest` stamp.
    pub ticker: u64,
}

/// The arm's side effects beyond the NPC pool state.
#[derive(Clone, Debug, Default)]
pub struct Style75Out {
    /// The `Projectile.NewProjectile` requests in C# order (the
    /// `HostileShotAi` shape carries the trailing `ai0`/`ai1`).
    pub shots: Vec<HostileShotAi>,
    /// `SoundEngine.PlaySound(SoundID.Item12/Item39, pos)` —
    /// `(style, x, y)`; the audio seam owns the pitch draw (module
    /// docs). `SoundID.Item12 = (2, 12)` (`ID/SoundID.cs:1395`),
    /// `Item39 = (2, 39)` (`:1422`).
    pub sounds: Vec<(i32, f32, f32)>,
    /// The 390/416 first-tick carrier spawn: `(type, x, y)` — the
    /// truncated centre (`NewNPC`'s `(int)Center.X/Y`), Bottom-placed.
    pub spawn_carrier: Option<(i32, f32, f32)>,
    /// The carrier-gone `Transform(newType)` request (382 / 518).
    pub transform: Option<i32>,
    /// The orphan death (`:36066-36068`): `hp` is already 0; the caller
    /// owes `HitEffect()` + deactivation.
    pub died: bool,
}

/// The `this.type` of a pool NPC for the family's host checks — the
/// six riders plus the four hosts (391 Scutlix, 415 Drakomire, 395
/// saucer core, 491 Dutchman) and the two Transform targets. Explicit
/// rows (the `npc_ai_style37.rs::family_type` precedent) — the port's
/// internal protocol ids (`npc/data.rs::to_u16`) are NOT these.
fn pool_type(n: &Npc) -> Option<i32> {
    Some(match n.kind {
        NpcKind::ScutlixRider => 390,
        NpcKind::Scutlix => 391,
        NpcKind::MartianSaucer => 392,
        NpcKind::MartianSaucerTurret => 393,
        NpcKind::MartianSaucerCannon => 394,
        NpcKind::MartianSaucerCore => 395,
        NpcKind::SolarDrakomire => 415,
        NpcKind::SolarDrakomireRider => 416,
        NpcKind::FlyingDutchman => 491,
        NpcKind::FlyingDutchmanCannon => 492,
        NpcKind::RayGunner => 382,
        NpcKind::SolarSpearman => 518,
        _ => return None,
    })
}

/// `Main.npc[slot].active && type == want` — the `active` ≈ `hp > 0`
/// stand-in (module docs). The C# indexes without a bounds check; the
/// port `.get()`s (an out-of-range slot is "no host", the style72
/// defensive seam).
fn host_is(npcs: &[Npc], slot: i32, want: i32) -> bool {
    slot >= 0
        && npcs
            .get(slot as usize)
            .map_or(false, |n| n.hp > 0 && pool_type(n) == Some(want))
}

/// `Entity.Center` — INTEGER half-extents (`Entity.cs:122-125`:
/// `position + (width / 2, height / 2)`, `width`/`height` are `int`).
/// Every family box is even, so this equals the float-half centre.
fn center(n: &Npc) -> Vec2 {
    let (w, h) = n.size();
    Vec2::new(
        n.x + (w as i32 / 2) as f32,
        n.y + (h as i32 / 2) as f32,
    )
}

/// `Player.Center` — the same integer halves (20×42 → 10/21).
fn player_center(p: &TargetCandidate) -> Vec2 {
    Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32)
}

/// The `Main.player[this.target]` read (the style37 convention: an
/// absent slot falls back to slot 0, `NPC.cs:77731-77734`).
fn target_player<'a>(players: &'a [TargetCandidate], target: i32) -> &'a TargetCandidate {
    players.get(target as usize).unwrap_or(&players[0])
}

/// XNA `Vector2.Length()` — `(float)Math.Sqrt((double)(X·X + Y·Y))`.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
}

/// XNA `Vector2.Distance(a, b)`.
fn distance(a: Vec2, b: Vec2) -> f32 {
    length(a - b)
}

/// XNA `Vector2.Normalize` — the RECIPROCAL-FIRST order
/// (`num = 1f / sqrt(...)` then the two multiplies; a zero vector
/// yields NaN components, which the arm's `IsNaN` guards read).
fn normalize(v: Vec2) -> Vec2 {
    let inv = 1.0f32 / length(v);
    Vec2::new(v.x * inv, v.y * inv)
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`, centre = zero): the cos/sin
/// run in DOUBLE then cast to f32; the result accumulates `x·c − y·s`
/// / `x·s + y·c` in that order.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): `(cos f, sin f)`
/// in double, cast to f32.
fn to_rotation_vector2(f: f32) -> Vec2 {
    Vec2::new((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// `Utils.ToRotation` (`Utils.cs:1655-1658`): `(float)Atan2(Y, X)`.
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `NPC.GetAttackDamage_ForProjectiles` (`NPC.cs:7052-7055`):
/// `(int)Utils.Remap(difficulty, Classic, Expert, normal, expert)` —
/// the clamped lerp (`Utils.cs:345-348` → `GetLerpValue` `:315-343`,
/// `MathHelper.Lerp` = `a + (b − a)·t`). Classic = 1, Expert = 2; a
/// Master difficulty clamps to the expert value.
fn attack_damage_for_projectiles(difficulty: f32, normal: f32, expert: f32) -> i32 {
    let t = if difficulty < 1.0 {
        0.0
    } else if difficulty > 2.0 {
        1.0
    } else {
        (difficulty - 1.0) / (2.0 - 1.0)
    };
    (normal + (expert - normal) * t) as i32
}

/// `NPC.DiscourageDespawn(60)` (`NPC.cs:7258-7265`): raise `timeLeft`
/// to at least 60. The `despawnEncouraged = false` half has no port
/// field (module docs).
fn discourage_despawn(n: &mut Npc) {
    if n.despawn_timer < 60 {
        n.despawn_timer = 60;
    }
}

/// `Math.Sign(float)` — 0 at zero (Rust's `signum` returns 1 for +0.0).
fn math_sign(x: f32) -> i32 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}

/// One fixed step of the aiStyle-75 arm (`NPC.cs:35501-36070`), in C#
/// order. `npcs` is the slot-aligned pool, `me` this NPC's slot
/// (`whoAmI`). The caller owns the movement integration after `AI()`
/// (every member is `noTileCollide` except the 390/416 riders, whose
/// ride re-snaps each tick anyway) and drains [`Style75Out`].
pub fn style75_step(
    npcs: &mut [Npc],
    me: usize,
    world: &World,
    players: &[TargetCandidate],
    ctx: &Style75Ctx,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
) -> Style75Out {
    let mut out = Style75Out::default();
    let Some(ty) = pool_type(&npcs[me]) else {
        return out; // no other kind rides aiStyle 75
    };
    if !matches!(ty, 390 | 392 | 393 | 394 | 416 | 492) {
        return out; // a host kind (391/395/491/...) — not a rider
    }
    // `:35503-35505` — num1055 (the host slot), vector132 (the ride
    // offset), num1056 (the cannon's forced sprite side).
    let mut num1055: i32 = -1;
    let mut vector132 = Vec2::ZERO;
    let mut num1056: i32 = 0;
    // `:35506-35522` — the 390 ScutlixRider (its own `if`, not part of
    // the 416 chain below).
    if ty == 390 {
        if npcs[me].local_ai[0] == 0.0 {
            // `:35508-35514` — the one-time carrier spawn (netMode != 1
            // is the single-player constant). The slot write is the
            // caller's (module docs: the one-tick fold).
            npcs[me].local_ai[0] = 1.0;
            let c = center(&npcs[me]);
            out.spawn_carrier = Some((391, c.x.trunc(), c.y.trunc()));
            npcs[me].net_update = true;
            return out;
        }
        let num1058 = npcs[me].ai[0] as i32;
        if host_is(npcs, num1058, 391) {
            discourage_despawn(&mut npcs[me]);
            num1055 = num1058;
            vector132 = Vec2::new(0.0, -14.0); // UnitY * -14f (:35520)
        }
    }
    // `:35523-35658` — the 416/392/393/394/492 else-if chain.
    if ty == 416 {
        if npcs[me].local_ai[0] == 0.0 {
            // `:35525-35531` — the Drakomire spawn.
            npcs[me].local_ai[0] = 1.0;
            let c = center(&npcs[me]);
            out.spawn_carrier = Some((415, c.x.trunc(), c.y.trunc()));
            npcs[me].net_update = true;
            return out;
        }
        let num1060 = npcs[me].ai[0] as i32;
        if host_is(npcs, num1060, 415) {
            discourage_despawn(&mut npcs[me]);
            num1055 = num1060;
            // `:35537` — (-carrier.spriteDirection * 10, -30).
            let sd = npcs[num1060 as usize].sprite_dir;
            vector132 = Vec2::new(-(sd as f32) * 10.0, -30.0);
        }
    } else if ty == 392 {
        let num1061 = npcs[me].ai[0] as i32;
        if host_is(npcs, num1061, 395) {
            discourage_despawn(&mut npcs[me]);
            num1055 = num1061;
            // `:35547-35551` — UnitY·2 × core.scale, rotated by the
            // core's rotation; the shell tilts with it.
            let core = &npcs[num1061 as usize];
            vector132 = Vec2::new(0.0, 2.0) * core.scale;
            let rot = core.rotation;
            vector132 = rotated_by(vector132, rot as f64);
            npcs[me].rotation = rot;
            // `:35552-35613` — the PHASE BRIDGE (netMode != 1).
            // flag80: the core is in [0, 1) AND no crew member lives.
            let core = &npcs[num1061 as usize];
            let mut flag80 = !(core.ai[0] >= 1.0 || core.ai[0] < 0.0);
            if flag80 {
                for k in 0..2 {
                    let slot = npcs[me].local_ai[k] as i32;
                    if host_is(npcs, slot, 393) {
                        flag80 = false;
                    }
                }
                for k in 2..4 {
                    let slot = npcs[me].local_ai[k] as i32;
                    if host_is(npcs, slot, 394) {
                        flag80 = false;
                    }
                }
            }
            // `:35576-35593` — the %200 crew netUpdate marks.
            let core = &npcs[num1061 as usize];
            if core.ai[3] % 200.0 == 0.0 && core.ai[0] != 1.0 {
                for k in 0..2 {
                    let slot = npcs[me].local_ai[k] as i32;
                    if host_is(npcs, slot, 393) {
                        npcs[slot as usize].net_update = true;
                    }
                }
                for k in 2..4 {
                    let slot = npcs[me].local_ai[k] as i32;
                    if host_is(npcs, slot, 394) {
                        npcs[slot as usize].net_update = true;
                    }
                }
                npcs[me].net_update = true;
            }
            // `:35594-35612` — crew all dead → the core opens.
            if flag80 {
                let core = &mut npcs[num1061 as usize];
                core.ai[0] = if ctx.expert { 1.0 } else { 3.0 };
                core.ai[1] = 0.0;
                core.ai[2] = 0.0;
                core.ai[3] = 0.0;
                core.net_update = true;
            }
        }
    } else if ty == 393 {
        let num1067 = npcs[me].ai[0] as i32;
        if host_is(npcs, num1067, 395) {
            discourage_despawn(&mut npcs[me]);
            num1055 = num1067;
            // `:35623-35627` — UnitY·29 ± UnitX·60 (the ai[1] seat).
            let side = if npcs[me].ai[1] == 1.0 { 1.0 } else { -1.0 };
            let core = &npcs[num1067 as usize];
            vector132 = (Vec2::new(0.0, 29.0) + Vec2::new(side * 60.0, 0.0)) * core.scale;
            let rot = core.rotation;
            vector132 = rotated_by(vector132, rot as f64);
            npcs[me].rotation = rot;
        }
    } else if ty == 394 {
        let num1069 = npcs[me].ai[0] as i32;
        if host_is(npcs, num1069, 395) {
            discourage_despawn(&mut npcs[me]);
            num1055 = num1069;
            // `:35637-35642` — UnitY·−13 ± UnitX·49; num1056 = the seat
            // side (the cannon's sprite faces outward, not the core's).
            let side = if npcs[me].ai[1] == 1.0 { 1.0 } else { -1.0 };
            let core = &npcs[num1069 as usize];
            vector132 = (Vec2::new(0.0, -13.0) + Vec2::new(side * 49.0, 0.0)) * core.scale;
            let rot = core.rotation;
            vector132 = rotated_by(vector132, rot as f64);
            npcs[me].rotation = rot;
            num1056 = if npcs[me].ai[1] == 1.0 { 1 } else { -1 };
        }
    } else if ty == 492 {
        let num1071 = npcs[me].ai[0] as i32;
        if host_is(npcs, num1071, 491) {
            discourage_despawn(&mut npcs[me]);
            num1055 = num1071;
            // `:35652-35656` — ((−122 + 68·ai[1]) × the ship's facing
            // flip, −6) × scale, rotated.
            let ship = &npcs[num1071 as usize];
            let flip = if ship.sprite_dir != 1 { 1.0 } else { -1.0 };
            vector132 =
                Vec2::new((-122.0 + 68.0 * npcs[me].ai[1]) * flip, -6.0) * ship.scale;
            let rot = ship.rotation;
            vector132 = rotated_by(vector132, rot as f64);
            npcs[me].rotation = rot;
        }
    }
    if num1055 == -1 {
        // `:36056-36069` — the carrier-gone fallbacks.
        if ty == 390 {
            out.transform = Some(382); // Transform(382) (:36058)
        } else if ty == 416 {
            out.transform = Some(518); // Transform(518) (:36062)
        } else {
            npcs[me].hp = 0; // life = 0; HitEffect(); active = false.
            out.died = true;
        }
        return out;
    }
    // `:35659-35676` — the ride. nPC5 = the host.
    let host = num1055 as usize;
    let (hc, h_gfx, h_dir, h_sprite) = {
        let h = &npcs[host];
        (center(h), h.gfx_off_y, h.facing, h.sprite_dir)
    };
    npcs[me].vx = 0.0;
    npcs[me].vy = 0.0;
    let (w, h) = npcs[me].size();
    npcs[me].x = hc.x - (w as i32 / 2) as f32;
    npcs[me].y = hc.y - (h as i32 / 2) as f32;
    npcs[me].x += vector132.x;
    npcs[me].y += vector132.y;
    npcs[me].gfx_off_y = h_gfx;
    npcs[me].facing = h_dir;
    npcs[me].sprite_dir = if num1056 == 0 { h_sprite } else { num1056 };
    // `:35677-35753` — the 390 gunner block.
    if ty == 390 {
        // `:35679-35681` — the timeLeft/velocity/target mirrors.
        npcs[me].despawn_timer = npcs[host].despawn_timer;
        npcs[me].vx = npcs[host].vx;
        npcs[me].vy = npcs[host].vy;
        npcs[me].target = npcs[host].target;
        // `:35682-35689` — the fire cycle.
        if npcs[me].ai[1] < 60.0 {
            npcs[me].ai[1] += 1.0;
        }
        if ctx.just_hit {
            npcs[me].ai[1] = -30.0;
        }
        let p = target_player(players, npcs[me].target);
        let (mw, mh) = npcs[me].size();
        if crate::projectile::can_hit(
            world,
            npcs[me].x,
            npcs[me].y,
            mw,
            mh,
            p.x,
            p.y,
            p.width as f32,
            p.height as f32,
        ) {
            let c = center(&npcs[me]);
            let vector133 = player_center(p) - c;
            let vector134 = normalize(vector133);
            if length(vector133) < 700.0 {
                if npcs[me].ai[1] == 60.0 && math_sign(vector133.x) == npcs[me].facing {
                    // `:35701-35716` — the fire.
                    npcs[me].ai[1] = -60.0;
                    let vector135 = c - Vec2::new(0.0, 4.0);
                    let mut v = player_center(p) - vector135;
                    // The four draws in C# order (:35705-35708).
                    v.x += rng.next_range(-50, 51) as f32;
                    v.y += rng.next_range(-50, 51) as f32;
                    v.x *= rng.next_range(80, 121) as f32 * 0.01;
                    v.y *= rng.next_range(80, 121) as f32 * 0.01;
                    let mut v = normalize(v);
                    if v.x.is_nan() || v.y.is_nan() {
                        v = Vec2::new(0.0, -1.0); // -UnitY (:35712)
                    }
                    v *= 7.0;
                    out.shots.push(HostileShotAi {
                        kind: 438,
                        x: vector135.x,
                        y: vector135.y,
                        vx: v.x,
                        vy: v.y,
                        damage: 30,
                        ai0: 0.0,
                        ai1: 0.0,
                    });
                    npcs[me].net_update = true;
                } else {
                    // `:35720-35749` — the idle octant.
                    let old = npcs[me].ai[2];
                    npcs[me].vx *= 0.5;
                    npcs[me].ai[2] = 3.0;
                    if vector134.y.abs() > vector134.x.abs() * 2.0 {
                        npcs[me].ai[2] = if vector134.y > 0.0 { 1.0 } else { 5.0 };
                    } else if vector134.x.abs() > vector134.y.abs() * 2.0 {
                        npcs[me].ai[2] = 3.0;
                    } else if vector134.y > 0.0 {
                        npcs[me].ai[2] = 2.0;
                    } else {
                        npcs[me].ai[2] = 4.0;
                    }
                    if npcs[me].ai[2] != old {
                        npcs[me].net_update = true;
                    }
                }
            }
        }
    }
    // `:35754-35822` — the 492 Dutchman cannon block.
    if ty == 492 {
        npcs[me].despawn_timer = npcs[host].despawn_timer;
        npcs[me].vx = npcs[host].vx;
        npcs[me].vy = npcs[host].vy;
        if npcs[me].ai[3] < 240.0 {
            npcs[me].ai[3] += 1.0;
        }
        if npcs[me].ai[3] == 2.0 {
            // `:35764` — the one-shot retarget (no facing write).
            target_closest(&mut npcs[me], world, players, false, 75, ctx.ticker);
        }
        let p = target_player(players, npcs[me].target);
        let (mw, mh) = npcs[me].size();
        let c = center(&npcs[me]);
        if crate::projectile::can_hit(
            world,
            npcs[me].x,
            npcs[me].y,
            mw,
            mh,
            p.x,
            p.y,
            p.width as f32,
            p.height as f32,
        ) {
            // `:35768` — vector137 is a discarded read in the C#.
            if npcs[me].ai[3] >= 240.0 {
                // `:35771-35785` — the shell shot.
                npcs[me].ai[3] = 0.0;
                let mut v = normalize(player_center(p) - c);
                if v.x.is_nan() || v.y.is_nan() {
                    v = Vec2::new(0.0, 1.0); // +UnitY (:35777)
                }
                v *= 14.0;
                v += Vec2::new(0.0, -5.0);
                out.shots.push(HostileShotAi {
                    kind: 240,
                    x: c.x,
                    y: c.y,
                    vx: v.x,
                    vy: v.y,
                    damage: 30,
                    ai0: 0.0,
                    ai1: 0.0,
                });
                npcs[me].net_update = true;
            } else {
                // `:35789-35811` — the eight-ray aim: the CLOSEST of
                // the eight 50-px probes around the cannon.
                let old = npcs[me].ai[2];
                let pc = player_center(p);
                let mut best = 0usize;
                let mut best_d = f32::INFINITY;
                for k in 0..8 {
                    // UnitY.RotatedBy(k * -(float)PI/4) * 50 (:35793).
                    let ang = k as f32 * -(std::f32::consts::PI / 4.0);
                    let probe = c + rotated_by(Vec2::new(0.0, 1.0), ang as f64) * 50.0;
                    let d = distance(probe, pc);
                    if best_d > d {
                        best_d = d;
                        best = k;
                    }
                }
                npcs[me].ai[2] = (best + 1) as f32;
                if npcs[me].sprite_dir == 1 {
                    npcs[me].ai[2] = 9.0 - npcs[me].ai[2];
                }
                if npcs[me].ai[2] != old {
                    npcs[me].net_update = true;
                }
            }
        } else {
            // `:35816-35820` — no sight → the aim clears.
            if npcs[me].ai[2] != 0.0 {
                npcs[me].net_update = true;
            }
            npcs[me].ai[2] = 0.0;
        }
    }
    // `:35823-35850` — the 394 saucer cannon block.
    if ty == 394 {
        // flag81: the core is not in a despawn/vulnerable phase.
        let core = &npcs[host];
        let flag81 = core.hp > 0
            && pool_type(core) == Some(395)
            && core.ai[0] != 1.0
            && core.ai[0] != -1.0
            && core.ai[0] != -2.0;
        npcs[me].despawn_timer = npcs[host].despawn_timer;
        let dmg = attack_damage_for_projectiles(ctx.difficulty, 50.0, 37.0);
        npcs[me].ai[3] = npcs[host].ai[3];
        let t = npcs[me].ai[3];
        if t >= 440.0 && t < 440.0 + 140.0 {
            let rel = t - 440.0;
            if flag81 && rel % 20.0 == 0.0 {
                // `:35842-35845` — the 448 missile, ONE NextDouble.
                let jitter = (rng.next_double() - 0.5) * 0.7853981852531433;
                let v = rotated_by(Vec2::new(num1056 as f32, 0.0), jitter) * 8.0;
                let c = center(&npcs[me]);
                let muzzle = Vec2::new(num1056 as f32 * 36.0, 8.0) + c;
                out.shots.push(HostileShotAi {
                    kind: 448,
                    x: muzzle.x,
                    y: muzzle.y,
                    vx: v.x,
                    vy: v.y,
                    damage: dmg,
                    ai0: 0.0,
                    ai1: 20.0, // the arming delay (:35845)
                });
                let c = center(&npcs[me]);
                out.sounds.push((39, c.x, c.y)); // Item39 (:35847)
            }
        }
    }
    // `:35851-35894` — the 393 saucer turret block.
    if ty == 393 {
        let core = &npcs[host];
        let flag82 = core.hp > 0
            && pool_type(core) == Some(395)
            && core.ai[0] != 1.0
            && core.ai[0] != -1.0
            && core.ai[0] != -2.0;
        npcs[me].despawn_timer = npcs[host].despawn_timer;
        let dmg = attack_damage_for_projectiles(ctx.difficulty, 35.0, 30.0);
        npcs[me].ai[3] = npcs[host].ai[3];
        let t = npcs[me].ai[3];
        let flag83 = t >= 280.0 && t < 280.0 + 140.0;
        if !flag83 {
            // `:35866-35877` — the per-tick aim outside the window.
            target_closest(&mut npcs[me], world, players, false, 75, ctx.ticker);
            let p = target_player(players, npcs[me].target);
            let c = center(&npcs[me]);
            let mut v3 = player_center(p) - c;
            if v3.y < 0.0 {
                v3.y = 0.0; // the turret never aims upward (:35868-35871)
            }
            let mut v3 = normalize(v3);
            if v3.x.is_nan() || v3.y.is_nan() {
                v3 = Vec2::new(0.0, 1.0);
            }
            npcs[me].ai[2] = to_rotation(v3);
        }
        if flag83 {
            let rel = t - 280.0;
            if flag82 && rel % 6.0 == 0.0 {
                // `:35886-35889` — the 449 ray, ONE NextDouble (/3).
                let jitter = (rng.next_double() - 0.5) * 0.7853981852531433 / 3.0;
                let v = rotated_by(to_rotation_vector2(npcs[me].ai[2]), jitter) * 16.0;
                let c = center(&npcs[me]);
                let muzzle = c + v; // vector141 * 1f (:35888)
                out.shots.push(HostileShotAi {
                    kind: 449,
                    x: muzzle.x,
                    y: muzzle.y,
                    vx: v.x,
                    vy: v.y,
                    damage: dmg,
                    ai0: 0.0,
                    ai1: 0.0,
                });
                out.sounds.push((12, c.x, c.y)); // Item12 (:35891)
            }
        }
    }
    // `:35895-35898` — every non-shell member returns here.
    if ty != 392 {
        return out;
    }
    // `:35899-36054` — the 392 shell's battery.
    let core = &npcs[host];
    let flag84 = core.hp > 0
        && pool_type(core) == Some(395)
        && core.ai[0] != 1.0
        && core.ai[0] != -1.0
        && core.ai[0] != -2.0;
    npcs[me].despawn_timer = npcs[host].despawn_timer;
    let dmg_ray = attack_damage_for_projectiles(ctx.difficulty, 50.0, 50.0);
    npcs[me].ai[3] = npcs[host].ai[3];
    let t = npcs[me].ai[3];
    // `:35907-35921` — the phase-0 deathray window's ENTRY tick.
    if t >= 20.0 && t < 20.0 + 240.0 && npcs[host].ai[0] == 0.0 {
        let rel = t - 20.0;
        if flag84 && rel == 0.0 {
            let c = center(&npcs[me]);
            out.shots.push(HostileShotAi {
                kind: 447,
                x: c.x,
                y: c.y,
                vx: 0.0,
                vy: 0.0,
                damage: dmg_ray,
                ai0: me as f32 + 1.0, // base.whoAmI + 1 (:35917)
                ai1: 0.0,
            });
            out.sounds.push((12, c.x, c.y)); // Item12 (:35919)
        }
    }
    // `:35922-35954` — the battery windows + the roll bounds.
    let mut flag85 = false;
    let mut max_value4: i32 = 1000; // the turret-side rolls
    let mut max_value5: i32 = 1000; // the cannon-side rolls
    let dmg_missile = attack_damage_for_projectiles(ctx.difficulty, 30.0, 25.0);
    if npcs[host].ai[0] == 2.0 {
        flag85 = true;
        max_value5 = 120;
        max_value4 = 120;
    }
    if !flag85 {
        flag85 = t >= 280.0 && t < 280.0 + 120.0;
        if flag85 {
            max_value5 = 90;
            max_value4 = 60;
        }
    }
    if !flag85 {
        flag85 = t >= 440.0 && t < 440.0 + 140.0;
        if flag85 {
            max_value5 = 60;
            max_value4 = 90;
        }
    }
    // `:35955-35974` — the four crew-aliveness flags (true = the slot
    // is DEAD/absent → that emitter fires).
    let flag86 = !host_is(npcs, npcs[me].local_ai[0] as i32, 393);
    let flag87 = !host_is(npcs, npcs[me].local_ai[1] as i32, 393);
    let flag88 = !host_is(npcs, npcs[me].local_ai[2] as i32, 394);
    let flag89 = !host_is(npcs, npcs[me].local_ai[3] as i32, 394);
    // `:35975-36000` — the turret-side (downward) emitters.
    if flag84 && flag85 {
        if flag86 && rng.next_max(max_value4) == 0 {
            // `:35982-35984` — velocity X draw FIRST, then the
            // rotation draw, then the two muzzle draws.
            let vx = -1.0 * rng.next_double() as f32 * 3.0;
            let jitter = (rng.next_double() - 0.5) * 0.7853981852531433;
            let v = rotated_by(Vec2::new(vx, 1.0), jitter) * 3.0;
            let c = center(&npcs[me]);
            let muzzle = c
                + Vec2::new(-(rng.next_range(50, 70) as f32), 0.0)
                + Vec2::new(0.0, rng.next_range(30, 45) as f32);
            out.shots.push(HostileShotAi {
                kind: 450,
                x: muzzle.x,
                y: muzzle.y,
                vx: v.x,
                vy: v.y,
                damage: dmg_missile,
                ai0: 0.0,
                ai1: 0.0,
            });
            out.sounds.push((39, c.x, c.y)); // Item39 (:35987)
        }
        if flag87 && rng.next_max(max_value4) == 0 {
            // `:35993-35995` — the right-side mirror (+X).
            let vx = rng.next_double() as f32 * 3.0;
            let jitter = (rng.next_double() - 0.5) * 0.7853981852531433;
            let v = rotated_by(Vec2::new(vx, 1.0), jitter) * 3.0;
            let c = center(&npcs[me]);
            let muzzle = c
                + Vec2::new(rng.next_range(50, 70) as f32, 0.0)
                + Vec2::new(0.0, rng.next_range(30, 45) as f32);
            out.shots.push(HostileShotAi {
                kind: 450,
                x: muzzle.x,
                y: muzzle.y,
                vx: v.x,
                vy: v.y,
                damage: dmg_missile,
                ai0: 0.0,
                ai1: 0.0,
            });
            out.sounds.push((39, c.x, c.y)); // Item39 (:35998)
        }
    }
    // `:36001-36026` — the cannon-side (upward) emitters.
    if flag84 && flag85 {
        if flag88 && rng.next_max(max_value5) == 0 {
            // `:36008-36010` — the −Y velocity base.
            let vx = -1.0 * rng.next_double() as f32 * 2.0;
            let jitter = (rng.next_double() - 0.5) * 0.7853981852531433;
            let v = rotated_by(Vec2::new(vx, -1.0), jitter) * 3.0;
            let c = center(&npcs[me]);
            let muzzle = c
                + Vec2::new(-(rng.next_range(30, 60) as f32), 0.0)
                + Vec2::new(0.0, rng.next_range(-30, -10) as f32);
            out.shots.push(HostileShotAi {
                kind: 450,
                x: muzzle.x,
                y: muzzle.y,
                vx: v.x,
                vy: v.y,
                damage: dmg_missile,
                ai0: 0.0,
                ai1: 0.0,
            });
            out.sounds.push((39, c.x, c.y)); // Item39 (:36013)
        }
        if flag89 && rng.next_max(max_value5) == 0 {
            // `:36019-36021` — the right-side mirror.
            let vx = rng.next_double() as f32 * 2.0;
            let jitter = (rng.next_double() - 0.5) * 0.7853981852531433;
            let v = rotated_by(Vec2::new(vx, -1.0), jitter) * 3.0;
            let c = center(&npcs[me]);
            let muzzle = c
                + Vec2::new(rng.next_range(30, 60) as f32, 0.0)
                + Vec2::new(0.0, rng.next_range(-30, -10) as f32);
            out.shots.push(HostileShotAi {
                kind: 450,
                x: muzzle.x,
                y: muzzle.y,
                vx: v.x,
                vy: v.y,
                damage: dmg_missile,
                ai0: 0.0,
                ai1: 0.0,
            });
            out.sounds.push((39, c.x, c.y)); // Item39 (:36024)
        }
    }
    // `:36027-36054` — the four dead-crew dust emitters. Each gate is
    // `flag && Next(8) == 0`; the spawn consumes the position draws,
    // the Next(4) type pick, the Dust.NewDust internals (the REAL pool
    // allocator) and the velocity NextDouble pair — even when the pool
    // returns the 6000 sentinel (the C# writes Main.dust[6000]).
    let c = center(&npcs[me]);
    for (flag, left, upper) in [
        (flag86, true, false),  // :36027-36033
        (flag87, false, false), // :36034-36040
        (flag88, true, true),   // :36041-36047
        (flag89, false, true),  // :36048-36054
    ] {
        if flag && rng.next_max(8) == 0 {
            let mx = rng.next_range(if upper { 30 } else { 50 }, if upper { 60 } else { 70 }) as f32;
            let my = if upper {
                rng.next_range(-30, -10) as f32
            } else {
                rng.next_range(15, 30) as f32
            };
            let pos = c + Vec2::new(if left { -mx } else { mx }, my);
            let dust_type = if rng.next_max(4) != 0 { 31 } else { 228 };
            let idx = dust.new_dust(
                rng,
                dust_ctx,
                world,
                pos,
                4,
                16,
                dust_type,
                0.0,
                0.0,
                100,
                Color::default(),
                1.2,
            );
            // The velocity overwrite (:36030-36032 etc): the X draw
            // first, then the rotation draw; ×0.5; Y = −|Y|.
            let sx = if upper { 2.0 } else { 3.0 };
            let vx = (if left { -1.0 } else { 1.0 }) * rng.next_double() as f32 * sx;
            let jitter = (rng.next_double() - 0.5) * 0.7853981852531433;
            // NOTE: the upper pair's C# velocity base Y is +1f
            // (:36008's missile is −1, but the DUST at :36043/:36051
            // uses +1f) — the Y flip below makes it negative anyway.
            let mut v = rotated_by(Vec2::new(vx, 1.0), jitter);
            v *= 0.5;
            v.y = 0.0 - v.y.abs();
            dust.slot_mut(idx).velocity = v;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::particles::DUST_SENTINEL;

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
        // Big enough that the (2030, 1520)-region hosts sit inside the
        // world box — `Collision.CanHit` walks out-of-world as blocked.
        World::new("t".into(), 400, 400)
    }

    fn ctx() -> Style75Ctx {
        Style75Ctx {
            difficulty: 1.0,
            expert: false,
            just_hit: false,
            ticker: 0,
        }
    }

    fn dust_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(8000.0, 8000.0),
            max_dust_to_draw: 6000,
        }
    }

    /// A pool NPC of the given kind at the given position, alive, with
    /// a live target slot 0. `hp`/`max_hp` ride the kind's SetDefaults
    /// life so the `active` stand-in reads true.
    fn member(kind: NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 1, 0.0);
        n.hp = 1000;
        n.max_hp = 1000;
        n.target = 0;
        n.despawn_timer = 750; // NewNPC's activeTime default
        n
    }

    /// A two-slot pool: the rider at `me`, its host at slot 1.
    fn pair(rider: NpcKind, host: NpcKind) -> Vec<Npc> {
        vec![member(rider, 1000.0, 1000.0), member(host, 2000.0, 1500.0)]
    }

    /// The census: exactly six rider kinds map onto the arm, and the
    /// hosts/Transform targets resolve for the slot checks.
    #[test]
    fn the_census_maps_the_six_riders_and_their_hosts() {
        for (kind, ty) in [
            (NpcKind::ScutlixRider, 390),
            (NpcKind::MartianSaucer, 392),
            (NpcKind::MartianSaucerTurret, 393),
            (NpcKind::MartianSaucerCannon, 394),
            (NpcKind::FlyingDutchmanCannon, 492),
            (NpcKind::SolarDrakomireRider, 416),
            (NpcKind::Scutlix, 391),
            (NpcKind::MartianSaucerCore, 395),
            (NpcKind::SolarDrakomire, 415),
            (NpcKind::FlyingDutchman, 491),
            (NpcKind::RayGunner, 382),
            (NpcKind::SolarSpearman, 518),
        ] {
            assert_eq!(pool_type(&member(kind, 0.0, 0.0)), Some(ty));
        }
        assert_eq!(pool_type(&member(NpcKind::Zombie, 0.0, 0.0)), None);
    }

    /// `NPC.cs:35508-35514` — the 390 first tick latches localAI[0],
    /// requests the 391 at the truncated centre, syncs, and does NOT
    /// fall into the Transform branch (the one-tick fold).
    #[test]
    fn the_scutlix_rider_spawns_its_carrier_once() {
        let w = air();
        let mut pool = vec![member(NpcKind::ScutlixRider, 1000.5, 900.9)];
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(1);
        let out = style75_step(
            &mut pool, 0, &w, &[cand(1200.0, 900.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].local_ai[0], 1.0);
        // The 18×40 box: centre (1009.5, 920.9) → trunc (1009, 920).
        assert_eq!(out.spawn_carrier, Some((391, 1009.0, 920.0)));
        assert!(pool[0].net_update);
        assert!(out.transform.is_none(), "no Transform on the spawn tick");
        // The second tick (carrier still absent) DOES transform.
        let out2 = style75_step(
            &mut pool, 0, &w, &[cand(1200.0, 900.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(out2.transform, Some(382), ":36058 Transform(382)");
        assert!(out2.spawn_carrier.is_none(), "the latch fired once");
    }

    /// `NPC.cs:35525-35531` + `:36062` — the 416 twin: the 415 spawn
    /// and the Transform(518) fallback.
    #[test]
    fn the_drakomire_rider_spawns_and_transforms_to_the_spearman() {
        let w = air();
        let mut pool = vec![member(NpcKind::SolarDrakomireRider, 500.0, 500.0)];
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(1);
        let out = style75_step(
            &mut pool, 0, &w, &[cand(700.0, 500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(out.spawn_carrier, Some((415, 509.0, 520.0)));
        let out2 = style75_step(
            &mut pool, 0, &w, &[cand(700.0, 500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(out2.transform, Some(518));
    }

    /// `NPC.cs:35659-35681` — the ride: the INTEGER-half snap onto the
    /// host centre + UnitY·−14, the gfx/direction/sprite mirrors, the
    /// timeLeft/velocity/target mirrors, DiscourageDespawn(60).
    #[test]
    fn the_rider_snaps_onto_the_scutlix_and_mirrors_it() {
        let w = air();
        let mut pool = pair(NpcKind::ScutlixRider, NpcKind::Scutlix);
        pool[0].ai[0] = 1.0;
        pool[0].local_ai[0] = 1.0; // the spawn already happened
        pool[0].despawn_timer = 10; // below 60 → discouraged
        pool[1].vx = 3.0;
        pool[1].vy = 1.0;
        pool[1].facing = -1;
        pool[1].sprite_dir = -1;
        pool[1].gfx_off_y = 2.5;
        pool[1].despawn_timer = 500;
        pool[1].target = 0;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(2);
        let witness = rng.clone();
        // The player far away (>700) → no fire, no draws.
        let out = style75_step(
            &mut pool, 0, &w, &[cand(9000.0, 9000.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.shots.is_empty());
        // Host centre (2030, 1520) (the 60×40 Scutlix); the rider's
        // 18×40 halves (9, 20): x = 2030−9, y = 1520−20−14.
        assert_eq!(pool[0].x, 2021.0);
        assert_eq!(pool[0].y, 1486.0);
        assert_eq!(pool[0].gfx_off_y, 2.5);
        assert_eq!(pool[0].facing, -1);
        assert_eq!(pool[0].sprite_dir, -1);
        assert_eq!(pool[0].vx, 3.0, "the velocity mirror (:35680)");
        assert_eq!(pool[0].despawn_timer, 500, "the timeLeft mirror (:35679)");
        assert_eq!(rng, witness, "the ride alone draws nothing");
        // DiscourageDespawn ran BEFORE the mirror raised 10 → 60 → 500.
        pool[1].despawn_timer = 30;
        let _ = style75_step(
            &mut pool, 0, &w, &[cand(9000.0, 9000.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].despawn_timer, 30, "the raw mirror copies 30");
    }

    /// `NPC.cs:35699-35716` — the fire tick: ai[1] 60 → −60, exactly
    /// FOUR draws in the C# order (±50, ±50, 80..120, 80..120), the
    /// 438 ray at 7 speed from centre−4Y, 30 damage.
    #[test]
    fn the_scutlix_rider_fires_with_the_four_jitter_draws() {
        let w = air();
        let mut pool = pair(NpcKind::ScutlixRider, NpcKind::Scutlix);
        pool[0].ai[0] = 1.0;
        pool[0].local_ai[0] = 1.0;
        pool[0].ai[1] = 59.0; // ticks to 60 this step
        pool[1].facing = 1;
        pool[1].sprite_dir = 1;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        let mut oracle = rng.clone();
        // Player to the RIGHT of the host (2030, 1520) and within 700.
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2300.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].ai[1], -60.0);
        assert_eq!(out.shots.len(), 1);
        let s = out.shots[0];
        assert_eq!((s.kind, s.damage), (438, 30));
        // The muzzle: the rider centre (2030, 1506) − 4Y.
        assert_eq!((s.x, s.y), (2030.0, 1502.0));
        // Replay the four draws against the oracle.
        let mut ax = 2310.0 - 2030.0; // player centre X − muzzle X
        let mut ay = 1521.0 - 1502.0;
        ax += oracle.next_range(-50, 51) as f32;
        ay += oracle.next_range(-50, 51) as f32;
        ax *= oracle.next_range(80, 121) as f32 * 0.01;
        ay *= oracle.next_range(80, 121) as f32 * 0.01;
        let inv = 7.0 / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert!((s.vx - ax * inv).abs() < 1e-4);
        assert!((s.vy - ay * inv).abs() < 1e-4);
        assert_eq!(rng, oracle, "exactly four draws, in order");
        assert!(pool[0].net_update);
    }

    /// `NPC.cs:35686-35689` + `:35720-35749` — justHit resets the cycle
    /// to −30; the idle branch halves vx and picks the octant with NO
    /// draws; the netUpdate fires only on an octant CHANGE.
    #[test]
    fn the_idle_octant_halves_vx_and_draws_nothing() {
        let w = air();
        let mut pool = pair(NpcKind::ScutlixRider, NpcKind::Scutlix);
        pool[0].ai[0] = 1.0;
        pool[0].local_ai[0] = 1.0;
        pool[0].ai[1] = 10.0;
        pool[1].vx = 4.0;
        let mut c = ctx();
        c.just_hit = true;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(4);
        let witness = rng.clone();
        // Player straight BELOW (dy dominant) → octant 1.
        let _ = style75_step(
            &mut pool, 0, &w, &[cand(2030.0, 1900.0)], &c, &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].ai[1], -30.0, "justHit (:35688)");
        assert_eq!(pool[0].ai[2], 1.0, "down-dominant → 1 (:35727)");
        assert_eq!(pool[0].vx, 2.0, "the ×0.5 damp (:35721)");
        assert_eq!(rng, witness);
        // Same octant next tick → no netUpdate re-fire.
        pool[0].net_update = false;
        let _ = style75_step(
            &mut pool, 0, &w, &[cand(2030.0, 1900.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(!pool[0].net_update, "the octant did not change");
        // Player far LEFT, level → octant 3.
        let _ = style75_step(
            &mut pool, 0, &w, &[cand(1500.0, 1505.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].ai[2], 3.0);
        assert!(pool[0].net_update);
    }

    /// `NPC.cs:35594-35612` — the phase bridge: crew all dead + the
    /// core in [0,1) → ai[0] = 3 classic / 1 expert, ai[1..3] cleared;
    /// a live crew member blocks it.
    #[test]
    fn the_shell_bridges_the_core_when_the_crew_dies() {
        let w = air();
        for expert in [false, true] {
            let mut pool = vec![
                member(NpcKind::MartianSaucer, 100.0, 100.0),
                member(NpcKind::MartianSaucerCore, 2000.0, 1500.0),
                member(NpcKind::MartianSaucerTurret, 0.0, 0.0),
            ];
            pool[0].ai[0] = 1.0;
            pool[0].local_ai = [2.0, 9.0, 9.0, 9.0]; // turret at slot 2
            pool[1].ai = [0.5, 7.0, 7.0, 7.0];
            pool[2].hp = 0; // the turret is dead
            let mut c = ctx();
            c.expert = expert;
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(5);
            let _ = style75_step(
                &mut pool, 0, &w, &[cand(2000.0, 1500.0)], &c, &mut dust, &dust_ctx(), &mut rng,
            );
            assert_eq!(pool[1].ai[0], if expert { 1.0 } else { 3.0 });
            assert_eq!((pool[1].ai[1], pool[1].ai[2], pool[1].ai[3]), (0.0, 0.0, 0.0));
            assert!(pool[1].net_update);
            // A LIVE turret blocks the bridge.
            pool[1].ai = [0.5, 0.0, 0.0, 0.0];
            pool[2].hp = 100;
            let _ = style75_step(
                &mut pool, 0, &w, &[cand(2000.0, 1500.0)], &c, &mut dust, &dust_ctx(), &mut rng,
            );
            assert_eq!(pool[1].ai[0], 0.5, "the live crew blocks the bridge");
        }
    }

    /// `NPC.cs:35863-35892` — the turret: outside 280..420 the aim is
    /// the Y-clamped rotation (no draws); inside, every 6th tick fires
    /// the 449 at 16 speed with ONE NextDouble/3 jitter, 35 dmg
    /// classic / 30 expert.
    #[test]
    fn the_turret_aims_clamped_and_fires_every_six_ticks() {
        let w = air();
        let mut pool = pair(NpcKind::MartianSaucerTurret, NpcKind::MartianSaucerCore);
        pool[0].ai[0] = 1.0;
        pool[0].ai[1] = 1.0; // the right seat
        pool[1].ai[3] = 100.0; // outside the window
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(6);
        let witness = rng.clone();
        // Player ABOVE the turret → the aim clamps to horizontal (0).
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1000.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.shots.is_empty());
        assert_eq!(pool[0].ai[2], 0.0, "the Y clamp → rotation 0");
        assert_eq!(rng, witness, "the aim draws nothing");
        // Inside the window at a %6 tick → the shot.
        pool[1].ai[3] = 286.0;
        pool[0].ai[2] = 0.0;
        let mut oracle = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(out.shots.len(), 1);
        let s = out.shots[0];
        assert_eq!((s.kind, s.damage), (449, 35));
        let jitter = (oracle.next_double() - 0.5) * 0.7853981852531433 / 3.0;
        assert!((s.vx - (jitter.cos() as f32) * 16.0).abs() < 1e-3);
        assert_eq!(rng, oracle, "one NextDouble per shot");
        // The turret centre: the 120×90 core's centre (2060, 1545) +
        // the seat offset (60, 29) → (2120, 1574); the sound plays at
        // the centre, the muzzle is one velocity-step ahead.
        assert_eq!(out.sounds, vec![(12, 2120.0, 1574.0)]);
        assert_eq!((s.x, s.y), (2120.0 + s.vx, 1574.0 + s.vy));
        // Expert damage 30.
        let mut c = ctx();
        c.difficulty = 2.0;
        pool[1].ai[3] = 292.0;
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &c, &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(out.shots[0].damage, 30);
        // A non-%6 tick inside the window: no shot, no aim, no draws.
        pool[1].ai[3] = 293.0;
        let witness = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.shots.is_empty());
        assert_eq!(rng, witness);
    }

    /// `NPC.cs:35835-35848` — the saucer cannon: the 440..580 window
    /// every 20 ticks, the 448 with ai1 = 20, the side·36+8 muzzle,
    /// the forced outward sprite side, ONE NextDouble jitter, 50/37.
    #[test]
    fn the_saucer_cannon_fires_the_armed_missile() {
        let w = air();
        let mut pool = pair(NpcKind::MartianSaucerCannon, NpcKind::MartianSaucerCore);
        pool[0].ai[0] = 1.0;
        pool[0].ai[1] = 1.0; // the right seat → num1056 = +1
        pool[1].ai[3] = 460.0; // rel 20 → a fire tick
        pool[1].sprite_dir = -1; // the core faces left…
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut oracle = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].sprite_dir, 1, "…but the cannon faces outward (:35642/:35675)");
        assert_eq!(out.shots.len(), 1);
        let s = out.shots[0];
        assert_eq!((s.kind, s.damage, s.ai1), (448, 50, 20.0));
        // The cannon centre: host (2060, 1545) + (49, −13) → (2109,
        // 1532); the muzzle +36X +8Y.
        assert_eq!((s.x, s.y), (2109.0 + 36.0, 1532.0 + 8.0));
        let jitter = (oracle.next_double() - 0.5) * 0.7853981852531433;
        assert!((s.vx - (jitter.cos() as f32) * 8.0).abs() < 1e-3);
        assert!((s.vy - (jitter.sin() as f32) * 8.0).abs() < 1e-3);
        assert_eq!(rng, oracle);
        // The vulnerable core (ai[0] == 1) silences the battery.
        pool[1].ai[0] = 1.0;
        pool[1].ai[3] = 480.0;
        let witness = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.shots.is_empty(), "flag81 gates the shot");
        assert_eq!(rng, witness, "the gate short-circuits BEFORE the draw");
    }

    /// `NPC.cs:35758-35820` — the Dutchman cannon: the 240 reload, the
    /// ×14 −5Y shell, the eight-ray aim with the sprite flip, the
    /// no-sight clear. (The live arm's S1-D-4b-5e behaviour, pinned
    /// here at the module level.)
    #[test]
    fn the_dutchman_cannon_reloads_and_aims() {
        let w = air();
        let mut pool = pair(NpcKind::FlyingDutchmanCannon, NpcKind::FlyingDutchman);
        pool[0].ai[0] = 1.0;
        pool[0].ai[1] = 0.0; // seat 0 → x offset −122
        pool[0].ai[3] = 239.0; // ticks to 240 → fires
        pool[1].sprite_dir = 1;
        // The ship's 350×120 box: centre (2175, 1560); the seat-0
        // offset −122 × the facing flip (spriteDirection == 1 → −1)
        // → +122, so the cannon centre is (2297, 1554).
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(8);
        let witness = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(pool[0].ai[3], 0.0, "the reload reset");
        assert_eq!(out.shots.len(), 1);
        let s = out.shots[0];
        assert_eq!((s.kind, s.damage), (240, 30));
        let c = Vec2::new(2297.0, 1554.0);
        assert_eq!((s.x, s.y), (c.x, c.y));
        let d = normalize(Vec2::new(2510.0, 1521.0) - c) * 14.0 + Vec2::new(0.0, -5.0);
        assert!((s.vx - d.x).abs() < 1e-3 && (s.vy - d.y).abs() < 1e-3);
        assert_eq!(rng, witness, "the 492 block draws NO rand");
        // A reload tick aims instead: the closest of the eight probes
        // wins; the player nearly level-right → probe k=2 (the +X ray)
        // is closest → ai[2] = 3, then the sprite_dir == 1 flip → 6.
        pool[0].ai[3] = 10.0;
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1554.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.shots.is_empty());
        assert_eq!(pool[0].ai[2], 6.0, "9 − 3 (:35804-35807)");
        assert_eq!(rng, witness);
    }

    /// `NPC.cs:35909-35920` — the deathray: ONLY the window's entry
    /// tick (rel == 0) with the core in phase 0 spawns the 447 with
    /// ai0 = whoAmI + 1; no draws.
    #[test]
    fn the_shell_fires_the_deathray_on_the_entry_tick_only() {
        let w = air();
        // A full live crew (slots 2..5) so the dead-crew dust gates
        // stay shut and the phase bridge never fires — the witness
        // then pins ZERO draws for the deathray tick.
        let mut pool = vec![
            member(NpcKind::MartianSaucer, 100.0, 100.0),
            member(NpcKind::MartianSaucerCore, 2000.0, 1500.0),
            member(NpcKind::MartianSaucerTurret, 0.0, 0.0),
            member(NpcKind::MartianSaucerTurret, 0.0, 0.0),
            member(NpcKind::MartianSaucerCannon, 0.0, 0.0),
            member(NpcKind::MartianSaucerCannon, 0.0, 0.0),
        ];
        pool[0].ai[0] = 1.0;
        pool[0].local_ai = [2.0, 3.0, 4.0, 5.0];
        pool[1].ai[0] = 0.0;
        pool[1].ai[3] = 20.0; // the entry tick
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        let witness = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(out.shots.len(), 1);
        let s = out.shots[0];
        assert_eq!((s.kind, s.damage, s.ai0), (447, 50, 1.0)); // me 0 + 1
        assert_eq!((s.vx, s.vy), (0.0, 0.0));
        assert_eq!(out.sounds.len(), 1);
        assert_eq!(out.sounds[0].0, 12);
        assert_eq!(rng, witness);
        // Tick 21: inside the window but rel != 0 → nothing.
        pool[1].ai[3] = 21.0;
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.shots.is_empty());
    }

    /// Replay ONE battery-emitter hit's draws (`NPC.cs:35982-35984` /
    /// `:35993-35995` / `:36008-36010` / `:36019-36021`): the velocity
    /// NextDouble, the rotation NextDouble, then the muzzle pair.
    fn replay_battery_hit(oracle: &mut UnifiedRandom, cannon_side: bool) {
        let _ = oracle.next_double();
        let _ = oracle.next_double();
        if cannon_side {
            let _ = oracle.next_range(30, 60);
            let _ = oracle.next_range(-30, -10);
        } else {
            let _ = oracle.next_range(50, 70);
            let _ = oracle.next_range(30, 45);
        }
    }

    /// Replay ONE dust-emitter hit's draws (`NPC.cs:36029-36032` etc):
    /// the position pair, the Next(4) type pick, the `Dust.NewDust`
    /// internals for a FREE slot 0 (no pressure band, on-screen, no
    /// ChildSafety — `Dust.cs:170-190`, the type-228 tweak `:216`),
    /// then the velocity NextDouble pair. Returns the dust type.
    fn replay_dust_hit(oracle: &mut UnifiedRandom, upper: bool) -> i32 {
        if upper {
            let _ = oracle.next_range(30, 60);
            let _ = oracle.next_range(-30, -10);
        } else {
            let _ = oracle.next_range(50, 70);
            let _ = oracle.next_range(15, 30);
        }
        let ty = if oracle.next_max(4) != 0 { 31 } else { 228 };
        let _ = oracle.next_max(1); // Next(num2-4), num2 = max(4,5) = 5
        let _ = oracle.next_max(12); // Next(num3-4), num3 = 16
        let _ = oracle.next_range(-20, 21);
        let _ = oracle.next_range(-20, 21);
        let _ = oracle.next_max(3); // the frame row
        let _ = oracle.next_range(-20, 21); // the scale jitter
        if ty == 228 {
            let _ = oracle.next_range(-10, 6); // Dust.cs:216
        }
        let _ = oracle.next_double();
        let _ = oracle.next_double();
        ty
    }

    /// `NPC.cs:35927-35954` + `:35975-36054` — the battery window
    /// table: phase 2 → 120/120 always; outside every window with the
    /// core NOT in phase 2 → zero battery rolls (the dust gates still
    /// run). The witness replays the per-slot gates against a mirrored
    /// stream, in the C# order 4,4,5,5 then the four Next(8)s.
    #[test]
    fn the_battery_rolls_the_window_bounds_in_slot_order() {
        let w = air();
        // All four crew slots absent → all four flags true.
        let mut pool = pair(NpcKind::MartianSaucer, NpcKind::MartianSaucerCore);
        pool[0].ai[0] = 1.0;
        pool[0].local_ai = [9.0, 9.0, 9.0, 9.0]; // no crew
        pool[1].ai[0] = 2.0; // the gun phase → 120/120
        pool[1].ai[3] = 10.0; // outside every window, but phase 2 opens
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(10);
        let mut oracle = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        let mut want_shots = 0;
        for cannon_side in [false, false, true, true] {
            if oracle.next_max(120) == 0 {
                want_shots += 1;
                replay_battery_hit(&mut oracle, cannon_side);
            }
        }
        for upper in [false, false, true, true] {
            if oracle.next_max(8) == 0 {
                let _ = replay_dust_hit(&mut oracle, upper);
            }
        }
        assert_eq!(out.shots.len(), want_shots);
        assert_eq!(rng, oracle, "the full battery + dust stream, in C# order");
        // Outside every window with the core in the death phase (3 —
        // no bridge, no phase-2 opening): zero battery rolls; only the
        // dust gates draw.
        pool[1].ai[0] = 3.0;
        pool[1].ai[3] = 700.0;
        let mut rng2 = UnifiedRandom::new(11);
        let mut oracle2 = rng2.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng2,
        );
        assert!(out.shots.is_empty(), "no window → no battery");
        for upper in [false, false, true, true] {
            if oracle2.next_max(8) == 0 {
                let _ = replay_dust_hit(&mut oracle2, upper);
            }
        }
        assert_eq!(rng2, oracle2);
        // The 280..400 window: maxValue4 = 60, maxValue5 = 90.
        pool[1].ai[3] = 300.0;
        let mut rng3 = UnifiedRandom::new(12);
        let mut oracle3 = rng3.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng3,
        );
        let mut want_shots = 0;
        for (bound, cannon_side) in [(60, false), (60, false), (90, true), (90, true)] {
            if oracle3.next_max(bound) == 0 {
                want_shots += 1;
                replay_battery_hit(&mut oracle3, cannon_side);
            }
        }
        for upper in [false, false, true, true] {
            if oracle3.next_max(8) == 0 {
                let _ = replay_dust_hit(&mut oracle3, upper);
            }
        }
        assert_eq!(out.shots.len(), want_shots);
        assert_eq!(rng3, oracle3);
        // The 440..580 window: maxValue4 = 90, maxValue5 = 60 (swapped).
        pool[1].ai[3] = 500.0;
        let mut rng4 = UnifiedRandom::new(13);
        let mut oracle4 = rng4.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng4,
        );
        let mut want_shots = 0;
        for (bound, cannon_side) in [(90, false), (90, false), (60, true), (60, true)] {
            if oracle4.next_max(bound) == 0 {
                want_shots += 1;
                replay_battery_hit(&mut oracle4, cannon_side);
            }
        }
        for upper in [false, false, true, true] {
            if oracle4.next_max(8) == 0 {
                let _ = replay_dust_hit(&mut oracle4, upper);
            }
        }
        assert_eq!(out.shots.len(), want_shots);
        assert_eq!(rng4, oracle4);
    }

    /// `NPC.cs:36027-36033` — one dust emitter in isolation: the
    /// Next(8) gate, the position draws, the 31/228 pick, the pool
    /// spawn and the velocity overwrite (×0.5, Y = −|Y|). The oracle
    /// replays every draw; the spawned slot's fields are pinned.
    #[test]
    fn the_dead_crew_dust_emitter_consumes_the_full_chain() {
        let w = air();
        let mut pool = pair(NpcKind::MartianSaucer, NpcKind::MartianSaucerCore);
        pool[0].ai[0] = 1.0;
        pool[0].local_ai = [9.0, 9.0, 9.0, 9.0];
        // The death phase (3): the bridge stays shut (ai[0] >= 1) and
        // no battery window opens at t = 700 — only the dust gates draw.
        pool[1].ai[0] = 3.0;
        pool[1].ai[3] = 700.0; // no battery window
        // Find a seed whose FIRST Next(8) is 0 (the left-turret gate
        // fires) — deterministic scan, no behavioural assumption.
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(8) == 0 {
                break;
            }
            seed += 1;
        }
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        let mut oracle = rng.clone();
        let _ = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert_eq!(oracle.next_max(8), 0);
        let mx = oracle.next_range(50, 70) as f32;
        let my = oracle.next_range(15, 30) as f32;
        let ty = if oracle.next_max(4) != 0 { 31 } else { 228 };
        let px = oracle.next_max(1) as f32 + 4.0;
        let py = oracle.next_max(12) as f32 + 4.0;
        let _dvx = oracle.next_range(-20, 21); // overwritten below
        let _dvy = oracle.next_range(-20, 21);
        let frame_row = oracle.next_max(3);
        let mut scale = (1.0 + oracle.next_range(-20, 21) as f32 * 0.01) * 1.2;
        if ty == 228 {
            let _ = oracle.next_range(-10, 6);
            scale *= 0.7;
        }
        // The velocity overwrite draws.
        let vx = -1.0 * oracle.next_double() as f32 * 3.0;
        let jitter = (oracle.next_double() - 0.5) * 0.7853981852531433;
        // The remaining three Next(8) gates (replay; chains if hit).
        for upper in [false, true, true] {
            if oracle.next_max(8) == 0 {
                let _ = replay_dust_hit(&mut oracle, upper);
            }
        }
        assert_eq!(rng, oracle, "the dust chain replays exactly");
        // The FIRST spawned slot (index 0 unless the pool refused).
        let d = dust.slot(0);
        if d.active {
            // The shell centre: the core's centre (2060, 1545) + the
            // rotated UnitY·2 (rotation 0) → (2060, 1547).
            let c = Vec2::new(2060.0, 1547.0);
            assert_eq!(d.dust_type, ty);
            assert!((d.position.x - (c.x - mx + px)).abs() < 1e-3);
            assert!((d.position.y - (c.y + my + py)).abs() < 1e-3);
            assert_eq!(d.frame_y, 10 * frame_row + 30 * (ty / 100));
            assert!((d.scale - scale).abs() < 1e-4);
            assert_eq!(d.alpha, 100);
            // The overwritten velocity: rotated (vx, 1) × 0.5, Y ≤ 0.
            let c2 = jitter.cos() as f32;
            let s2 = jitter.sin() as f32;
            let ovx = (vx * c2 - 1.0 * s2) * 0.5;
            let ovy = (vx * s2 + 1.0 * c2) * 0.5;
            assert!((d.velocity.x - ovx).abs() < 1e-4);
            assert!((d.velocity.y - (0.0 - ovy.abs())).abs() < 1e-4);
            assert!(d.velocity.y <= 0.0);
        } else {
            // The pool refused (off-screen/pressure) — the sentinel
            // still received the velocity writes and the draws.
            assert!(dust.slot(DUST_SENTINEL).velocity.y <= 0.0);
        }
    }

    /// `NPC.cs:36064-36069` — a saucer part whose core slot is dead
    /// dies at once (hp 0 + the died seam), with no draws.
    #[test]
    fn an_orphaned_saucer_part_dies() {
        let w = air();
        let mut pool = pair(NpcKind::MartianSaucerTurret, NpcKind::MartianSaucerCore);
        pool[0].ai[0] = 1.0;
        pool[1].hp = 0;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(12);
        let witness = rng.clone();
        let out = style75_step(
            &mut pool, 0, &w, &[cand(2500.0, 1500.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        assert!(out.died);
        assert_eq!(pool[0].hp, 0);
        assert!(out.transform.is_none(), "only the riders transform");
        assert_eq!(rng, witness);
    }

    /// `NPC.cs:7052-7055` — the difficulty ladder: the clamped
    /// Classic→Expert remap, Master clamps to the expert value, the
    /// `(int)` truncation.
    #[test]
    fn the_projectile_damage_ladder_clamps_at_expert() {
        assert_eq!(attack_damage_for_projectiles(1.0, 50.0, 37.0), 50);
        assert_eq!(attack_damage_for_projectiles(2.0, 50.0, 37.0), 37);
        assert_eq!(attack_damage_for_projectiles(3.0, 50.0, 37.0), 37, "Master clamps");
        assert_eq!(attack_damage_for_projectiles(1.5, 35.0, 30.0), 32, "(int)32.5");
        assert_eq!(attack_damage_for_projectiles(0.5, 30.0, 25.0), 30, "below Classic clamps");
    }

    /// The 416 ride offset reads the CARRIER's spriteDirection
    /// (`NPC.cs:35537`): (−sd·10, −30), unscaled/unrotated.
    #[test]
    fn the_drakomire_rider_offset_follows_the_carrier_facing() {
        let w = air();
        let mut pool = pair(NpcKind::SolarDrakomireRider, NpcKind::SolarDrakomire);
        pool[0].ai[0] = 1.0;
        pool[0].local_ai[0] = 1.0;
        pool[1].sprite_dir = -1;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        let _ = style75_step(
            &mut pool, 0, &w, &[cand(9000.0, 9000.0)], &ctx(), &mut dust, &dust_ctx(), &mut rng,
        );
        // Host centre (2030, 1520); offset (+10, −30); halves (9, 20).
        assert_eq!(pool[0].x, 2030.0 - 9.0 + 10.0);
        assert_eq!(pool[0].y, 1520.0 - 20.0 - 30.0);
        assert_eq!(pool[0].sprite_dir, -1, "the carrier's sprite mirrors");
    }
}
