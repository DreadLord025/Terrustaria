//! Frost Moon mob AI — S1-F-13c slice D, part 2 (**mob slice B**: the
//! ElfArcher 350 shooter machine). The sibling module
//! `npc_ai_frost_mob.rs` (`task/S1-F-13c-mob-ai`, commit 5ef9b24b) owns
//! Flocko/ElfCopter/PresentMimic/the AI_003 walk arms/the Nutcracker
//! transform and DEFERRED exactly this block; this file is deliberately
//! DISJOINT — same house pattern (a plain state struct + a type-keyed
//! step), no shared items, the merger unions the seams. `npc_ai_frost_mob.rs`
//! is NOT edited here.
//!
//! **The C# arm ported here:** the shared `AI_003_Fighters` shooter block
//! (`NPC.cs:59069-59801`, the type-list gate at `:59069` includes 350) —
//! the same block the goblin/pirate/martian shooters ride — resolved to
//! the **type-350 terms only**; every other kind's branch stays untouched
//! (the martian 381/382 arm already lives in `npc.rs::update_martian_shooter`,
//! the pirates beside it — neither is edited). The 350 path:
//!
//! * constants: `flag16`/`flag17`/`flag20` false, `flag18` true,
//!   `num160`/`num161` = -1 (`:59071-59085`, no 350 term), the 110-tick
//!   cycle `num162 = 110` (`:59111-59114`), the fire tick `num163 = 55`
//!   (`:59184`, `num162 / 2`), the shot speed `num164 = 11f` (the `:59214`
//!   default — no 350 override exists), the damage `num169 = 45`
//!   (`:59416-59419`), the projectile `num170 = 82` FlamingArrow (the
//!   `:59403` default — no 350 override; `ProjectileID.cs:718`).
//! * the timer decrement (`:59086-59089`), the `justHit` interrupt
//!   (`:59091-59097`: `ai[1] = 30`, `ai[2] = 0` — the interrupt drops
//!   the cycle below the fire tick, so no shot can leak from it), the
//!   `confused` clear (`:59197-59201`).
//! * the fire (`ai[2] > 0 && ai[1] == 55`, `:59202-59618`): TargetClosest
//!   (`:59206` — the wiring seam), the aim from Center (`:59279`), the
//!   ballistic lead `num166 = |dx| * 0.1f` subtracted from the aim's Y
//!   (`:59310`/`:59344`), the ±40 integer jitter X-then-Y (`:59394-59395`,
//!   the `type != 292` else arm — 350 falls through EVERY earlier arm),
//!   the 11f normalisation (`:59397-59401`), the origin += the shot
//!   velocity (`:59543-59544`), the single `NewProjectile` at
//!   `(chaserPosition2, num165, num167, 82, 45, 0f)` (`:59592`, the
//!   generic else arm under `netMode != 1`), then the aim-octant
//!   `ai[2]` = 1/5/3/2/4 table (`:59595-59617`).
//! * the stop maintenance (`:59619-59628`): airborne or timer-exhausted
//!   → `ai[2] = ai[1] = 0`; otherwise (flag16 false) `velocity.X *= 0.9`
//!   + `spriteDirection = direction`.
//! * the aggression entry (`:59634-59714`): `ai[2] <= 0 && vy == 0 &&
//!   ai[1] <= 0 && !player.dead`, the full-box `Collision.CanHit`
//!   (`:59636`), the `stealth == 0 && itemAnimation == 0` refusal
//!   (`:59641-59644` — carried on the target seam, NOT folded), the
//!   ±40-jittered aim distance gate `< 700f` (`:59647-59680`), then
//!   `velocity.X *= 0.5`, `ai[2] = octant`, `ai[1] = 110` and the octant
//!   re-pick on the NORMALISED aim (`:59683-59711`).
//! * the walk velocity arm (`:59715-59791`): `ai[2] <= 0` → the defaults
//!   cap 1f / accel 0.07f / brake 0.8f (`:59717-59719` — no 350 override);
//!   flag22 is false for 350 (`:59757-59767` are 381/382/520-only), so the
//!   over-cap arm damps BOTH velocity components ×0.8 when grounded
//!   (`:59768-59774`), else the ±0.07 accel clamped at ±1
//!   (`:59775-59790`). 350 is EXCLUDED from the generic AI_003 walker by
//!   the `:58322` list — this arm IS its walk; the shared stuck/jump/
//!   collision core stays S1-D-3p's scope (below).
//!
//! **RNG order (by construction — every `Main.rand.Next` on the 350 path):**
//!
//! * the fire tick (`ai[2] > 0 && ai[1] == 55`): exactly TWO draws —
//!   `Next(-40, 41)` for the aim X (`:59394`), then `Next(-40, 41)` for
//!   the aim Y (`:59395`). None of the earlier jitter arms fire for 350:
//!   `:59330`/`:59334` are 449-452/468, `:59338` is 481, `:59342` is
//!   498-506, `:59347-59390` are 291/381/382/520/481/498-506/426/214/216;
//!   `:59531`'s `NextVector2FromRectangle` is 426-only; the 292/424
//!   spawn loops (`:59567-59568`, `:59582`) never run for 350. The
//!   `NewProjectile` at `:59592` draws NOTHING at spawn for type 82:
//!   its `SetDefaults` arm is rand-free (`Projectile.cs:1625-1631`), the
//!   slot-scan randomiser is default-off (`:10831`) and the per-type
//!   rand switch covers 206/335/358/406 only (`:10905-10925`); 82's
//!   `Next(3)` lives in `StatusPlayer` (`:12284`) — a HIT-time draw on
//!   the projectile's own future tick, not this block.
//! * the aggression entry that passes ALL gates: exactly TWO draws —
//!   `Next(-40, 41)` X (`:59652`) then `Next(-40, 41)` Y (`:59653`) —
//!   drawn BEFORE the `< 700f` distance gate (`:59680`), so a
//!   gate-passing entry that then fails the range check still consumed
//!   them. An entry blocked by any earlier gate (airborne, timers,
//!   dead player, no LOS, the stealth/swing refusal) draws NOTHING —
//!   the draws sit inside `if (flag21)`.
//! * the two sites are mutually exclusive within a tick: the fire needs
//!   `ai[2] > 0` at `:59202` and the entry needs `ai[2] <= 0` at
//!   `:59634`; the fire never zeroes `ai[2]` before `:59634` (the octant
//!   table writes 1..5), and the entry's `ai[1] = 110` cannot re-enter
//!   the fire on the same tick. Per-tick totals: 2, 2 or 0 draws.
//! * everything else — the timers, the justHit/confused arms, the stop
//!   maintenance, the walk arm — draws NOTHING.
//!
//! The witness tests below replay each order against a same-seeded probe
//! stream (WRITTEN, NOT RUN — code-only mode).
//!
//! **Deferred (docs/ANALYSIS.md, "The ElfArcher shooter"):** the AI_003
//! shared stuck/jump/collision core (`:57120-57402` + the movement tail
//! `:59831+`) — S1-D-3p's full AI_003 migration, cited not ported; the
//! `FindFrame` arm (`:76099+`) — the defaults siblings' render wiring;
//! `netUpdate` stamps (`:59096`/`:59398`/`:59682` — netcode, no rand);
//! `Main.netMode != 1` (`:59558`) folds to the port's always-true
//! server/local path.
//!
//! **Integration seam / dependency:** the `NpcKind::ElfArcher` variant +
//! the `SetDefaults` row (350, aiStyle 3, `NPC.cs:13444-13456`) are the
//! defaults worker B's table (`task/S1-F-13c-defaults-b`). This module
//! runs on [`FrostMobBState`] — a field-compatible superset of the
//! sibling's `FrostMobState` (the union note in ANALYSIS: the two structs
//! carry the same core fields with the same names/types; the merger may
//! collapse them or copy field-by-field). The dispatch
//! ([`elf_archer_ai_step`]) keys on the vanilla type id 350 only. The
//! wiring worker runs `npc_target::target_closest` at the C#'s
//! `TargetClosest()` site (`:59206`, every stopped tick) and passes the
//! picked slot as [`FrostArcherTarget`] (which widens the sibling's
//! `FrostTarget` box with `dead`/`stealth`/`item_animation` — the entry
//! gates read them; `Player.stealth` defaults to 1f, `Player.cs:1562`).
//! `justHit`/`confused` ride [`ArcherTick`] — the wiring maps the port's
//! hurt flag/buff onto them (`Npc::hurt_cd` / the confusion debuff).

// The wiring lands with the defaults workers; until then the dispatch is
// exercised by this module's tests only.
#![allow(dead_code)]

use tr_core::{UnifiedRandom, World};

/// `NPCID.ElfArcher` — `NPCID.cs:11377`.
pub const ELF_ARCHER_TYPE: u16 = 350;
/// The C#'s `this.aiStyle` for 350 — `NPC.cs:13448` (3; the shooter block
/// runs inside `AI_003_Fighters` after the shared core).
pub const ELF_ARCHER_AI_STYLE: u8 = 3;
/// `num162` — the 110-tick shot cycle (`NPC.cs:59111-59114`).
const SHOT_CYCLE: i32 = 110;
/// `num163 = num162 / 2` — the fire tick (`NPC.cs:59184`).
const FIRE_TICK: i32 = SHOT_CYCLE / 2;
/// `num164` — the shot speed, the `:59214` default (no 350 override).
const SHOT_SPEED: f32 = 11.0;
/// `num169` — the shot damage (`NPC.cs:59416-59419`).
const SHOT_DAMAGE: i32 = 45;
/// `num170` — FlamingArrow, the `:59403` default (`ProjectileID.cs:718`).
const SHOT_KIND: u16 = 82;
/// `num179` — the aggression range gate, the `:59655` default 700f (no
/// 350 override in `:59656-59679`).
const AGGRO_RANGE: f32 = 700.0;
/// `num180`/`num181`/`num182` — the walk cap/accel/brake defaults
/// (`NPC.cs:59717-59719`; no 350 override in `:59720-59756`).
const WALK_CAP: f32 = 1.0;
const WALK_ACCEL: f32 = 0.07;
const WALK_BRAKE: f32 = 0.8;

/// The AI-visible subset of `NPC`'s fields the ElfArcher machine
/// reads/writes — the sibling `npc_ai_frost_mob.rs` `FrostMobState` shape
/// verbatim (same field names/types, so the merger can union the two
/// structs or copy field-by-field). The wiring worker copies these
/// in/out of `Npc`.
#[derive(Clone, Copy, Debug)]
pub struct FrostMobBState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `NPC.ai[0..3]` — the shooter owns `ai[1]` (the cycle timer) and
    /// `ai[2]` (the aim octant 1..5); `ai[0]`/`ai[3]` are untouched by
    /// the 350 path (the shared AI_003 core owns them — S1-D-3p).
    pub ai: [f32; 4],
    /// `NPC.localAI[0..3]` — untouched on the 350 path (the 216/520
    /// terms are other kinds'); carried for seam compatibility.
    pub local_ai: [f32; 4],
    /// `Entity.direction` — set by the shared core / `TargetClosest`
    /// (the wiring seam); the walk arm reads it, the stop mirrors it.
    pub direction: i32,
    /// `NPC.spriteDirection` — the stop arm's `:59627` write.
    pub sprite_direction: i32,
    pub rotation: f32,
    /// `NPC.life`/`lifeMax` — carried for seam compatibility (the 350
    /// shooter path reads neither).
    pub life: i32,
    pub life_max: i32,
    /// `Entity.width`/`height` (the `SetDefaults` box, `NPC.cs:13445-13446`).
    pub width: f32,
    pub height: f32,
    /// `NPC.scale` — carried for seam compatibility.
    pub scale: f32,
    /// `NPC.timeLeft` — the sibling's `despawn_timer` convention (-1 =
    /// unset); untouched by the 350 path.
    pub despawn_timer: i32,
}

impl FrostMobBState {
    /// A fresh ElfArcher at (x, y) with the `SetDefaults` AI inputs
    /// (`NPC.cs:13444-13456`: 18×40, scale 1, lifeMax 900 — they must
    /// agree with defaults worker B's row) and the C#'s field defaults
    /// (`ai`/`localAI` zeroed, direction 1). Same shape as the sibling's
    /// `FrostMobState::new(FrostMobKind::ElfArcher, x, y)`.
    pub fn elf_archer(x: f32, y: f32) -> Self {
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
            life: 900,
            life_max: 900,
            width: 18.0,
            height: 40.0,
            scale: 1.0,
            despawn_timer: -1,
        }
    }

    /// `Entity.Center` (`Entity.cs:50-55`): position + size/2.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// The resolved `Main.player[this.target]` — the sibling's `FrostTarget`
/// box widened with the three fields the shooter's entry gates read.
/// `TargetClosest()` runs at `:59206` (every stopped tick); the wiring
/// worker passes the picked slot here.
#[derive(Clone, Copy, Debug)]
pub struct FrostArcherTarget {
    /// `Player.position.X`/`Y`.
    pub x: f32,
    pub y: f32,
    /// `Player.width`/`height` (the 20×42 box).
    pub width: f32,
    pub height: f32,
    /// `Player.dead` — the entry gate (`NPC.cs:59634`).
    pub dead: bool,
    /// `Player.stealth` — the `:59641` refusal reads `== 0f`; the C#
    /// field default is 1f (`Player.cs:1562`). NOT folded to a constant.
    pub stealth: f32,
    /// `Player.itemAnimation` — the `:59641` refusal reads `== 0`.
    pub item_animation: i32,
}

impl FrostArcherTarget {
    /// `Player.Center` (`Entity.cs:50-55`).
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// The per-tick NPC flags the block reads outside `FrostMobState`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ArcherTick {
    /// `NPC.justHit` (`NPC.cs:59091`) — the wiring maps the port's hurt
    /// edge (`Npc::hurt_cd` freshly set) onto it.
    pub just_hit: bool,
    /// `NPC.confused` (`NPC.cs:59197`) — the Confused debuff (24).
    pub confused: bool,
}

/// A hostile-shot request — `Projectile.NewProjectile`'s
/// `(X, Y, SpeedX, SpeedY, Type, Damage, KnockBack)` payload
/// (`NPC.cs:59592`). KnockBack is 0f at the call site, so it is not
/// carried. Field-compatible with the sibling's `FrostShot`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrostArcherShot {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// The aim-octant table — the C#'s `ai[2]` rows, byte-identical at the
/// fire (`NPC.cs:59595-59617`) and the entry (`:59689-59711`): steep-down
/// 1 / steep-up 5 / flat-horizontal 3 / down-diagonal 2 / up-diagonal 4.
fn aim_octant(ax: f32, ay: f32) -> f32 {
    if ay.abs() > ax.abs() * 2.0 {
        if ay > 0.0 {
            1.0
        } else {
            5.0
        }
    } else if ax.abs() > ay.abs() * 2.0 {
        3.0
    } else if ay > 0.0 {
        2.0
    } else {
        4.0
    }
}

/// **ElfArcher (350)** — the shared shooter block (`NPC.cs:59069-59801`)
/// resolved to the type-350 terms; see the module header for the full
/// per-line map and the RNG argument. One call = one `AI()` tick's
/// shooter block, in the C#'s textual order: timers → confused → the
/// stopped/fire arm → the aggression entry → the walk arm. Gravity and
/// collision integration are the caller's (the shared AI_003 core,
/// S1-D-3p); `vy == 0` is the grounded predicate the C# itself uses.
pub fn elf_archer_ai(
    s: &mut FrostMobBState,
    world: &World,
    target: &FrostArcherTarget,
    tick: ArcherTick,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostArcherShot>,
) {
    elf_archer_ai_retarget(s, world, target, tick, rng, shots, &mut |_| {});
}

/// The wired form (S1-D-3 W-6): `retarget` runs at the C# `TargetClosest()`
/// site (`NPC.cs:59206`, every stopped tick — `flag18` is true for 350) and
/// syncs the picked facing back into `s.direction`. The pure
/// [`elf_archer_ai`] passes a no-op so the module tests keep the
/// resolved-target form.
pub fn elf_archer_ai_retarget(
    s: &mut FrostMobBState,
    world: &World,
    target: &FrostArcherTarget,
    tick: ArcherTick,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostArcherShot>,
    retarget: &mut dyn FnMut(&mut FrostMobBState),
) {
    // `:59086-59089` — the cycle timer decrements first.
    if s.ai[1] > 0.0 {
        s.ai[1] -= 1.0;
    }
    // `:59090-59097` — the justHit interrupt (flag19 feeds the 216 arm
    // only — not carried; netUpdate is netcode).
    if tick.just_hit {
        s.ai[1] = 30.0;
        s.ai[2] = 0.0;
    }
    // `:59197-59201` — the Confused clear (BEFORE the stopped arm).
    if tick.confused {
        s.ai[1] = 0.0;
        s.ai[2] = 0.0;
    }
    // `:59202-59629` — the stopped arm.
    if s.ai[2] > 0.0 {
        // `:59204-59207` — flag18 is true for 350: TargetClosest() every
        // stopped tick.
        retarget(s);
        if s.ai[1] == FIRE_TICK as f32 {
            // `:59279` — the muzzle starts at Center (no 350 offset in
            // `:59280-59308`).
            let (mut muzzle_x, mut muzzle_y) = s.center();
            let (pcx, pcy) = target.center();
            // `:59309-59310` — the aim + the ballistic lead term.
            let mut num165 = pcx - muzzle_x;
            let num166 = num165.abs() * 0.1;
            // `:59344` — the lead is subtracted from the aim's Y BEFORE
            // the jitter.
            let mut num167 = pcy - muzzle_y - num166;
            // `:59392-59396` — the `type != 292` else arm: draws 1-2,
            // X then Y, integer ±40 (no 0.2/0.4 scale for 350).
            num165 += rng.next_range(-40, 41) as f32;
            num167 += rng.next_range(-40, 41) as f32;
            // `:59397-59401` — the 11f normalisation (Math.Sqrt on the
            // float product — f32 sqrt is exact here).
            let num168 = SHOT_SPEED / (num165 * num165 + num167 * num167).sqrt();
            num165 *= num168;
            num167 *= num168;
            // `:59527-59542` — the 426 interception arm never runs for
            // 350 (vector36 stays null). `:59543-59544` — the origin
            // rides one velocity-step along the shot.
            muzzle_x += num165;
            muzzle_y += num167;
            // `:59550-59557` — the 290/381-392 damage folds skip 350.
            // `:59558-59593` — netMode != 1 (always-true) and the generic
            // else arm: ONE FlamingArrow, no extra draws (see header).
            shots.push(FrostArcherShot {
                kind: SHOT_KIND,
                x: muzzle_x,
                y: muzzle_y,
                vx: num165,
                vy: num167,
                damage: SHOT_DAMAGE,
            });
            // `:59595-59617` — the aim octant on the fired velocity.
            s.ai[2] = aim_octant(num165, num167);
        }
        // `:59619-59623` — airborne (flag17 false) or exhausted → wake.
        if s.vy != 0.0 || s.ai[1] <= 0.0 {
            s.ai[2] = 0.0;
            s.ai[1] = 0.0;
        } else {
            // `:59624-59628` — !flag16: the stop damp + the sprite mirror.
            s.vx *= 0.9;
            s.sprite_direction = s.direction;
        }
    }
    // `:59630-59714` — the aggression entry (the 468 eclipse arm skips).
    // The C#'s `|` operands are side-effect-free — && is exact.
    if s.ai[2] <= 0.0 && s.vy == 0.0 && s.ai[1] <= 0.0 && !target.dead {
        // `:59636` — the FULL-box LOS walk (not the copter's 1×1 centre).
        let mut flag21 = crate::projectile::can_hit(
            world,
            s.x,
            s.y,
            s.width,
            s.height,
            target.x,
            target.y,
            target.width,
            target.height,
        );
        // `:59641-59644` — a fully idle player (no stealth AND no swing)
        // is not engaged. Carried, not folded (stealth defaults 1f).
        if target.stealth == 0.0 && target.item_animation == 0 {
            flag21 = false;
        }
        if flag21 {
            // `:59647-59651` — the entry aim: Center → player Center,
            // the same |dx|*0.1 lead on Y.
            let (cx, cy) = s.center();
            let (pcx, pcy) = target.center();
            let mut num175 = pcx - cx;
            let num176 = num175.abs() * 0.1;
            let mut num177 = pcy - cy - num176;
            // `:59652-59653` — draws 1-2, X then Y — BEFORE the range gate.
            num175 += rng.next_range(-40, 41) as f32;
            num177 += rng.next_range(-40, 41) as f32;
            // `:59654` + `:59680` — the jittered distance vs the 700f gate.
            let num178 = (num175 * num175 + num177 * num177).sqrt();
            if num178 < AGGRO_RANGE {
                // `:59683-59688` — the stop: half the walk speed, the
                // octant latch, the 110-tick cycle.
                s.vx *= 0.5;
                let num178 = 10.0 / num178;
                num175 *= num178;
                num177 *= num178;
                s.ai[2] = 3.0;
                s.ai[1] = SHOT_CYCLE as f32;
                // `:59689-59711` — the octant re-pick on the normalised
                // aim (the :59687 `ai[2] = 3` only survives the flat arm).
                s.ai[2] = aim_octant(num175, num177);
            }
        }
    }
    // `:59715-59791` — the walk velocity arm (flag16 false, num160 -1 →
    // the gate is just `ai[2] <= 0`). 350 is excluded from the generic
    // AI_003 walker (`:58322`) — this IS its walk. The stuck/jump/
    // collision core around it stays S1-D-3p's scope.
    if s.ai[2] <= 0.0 {
        // flag22 (`:59757-59767`) is 381/382/520-only — false for 350.
        if s.vx < 0.0 - WALK_CAP || s.vx > WALK_CAP {
            // `:59768-59774` — the over-cap brake damps BOTH components,
            // grounded only.
            if s.vy == 0.0 {
                s.vx *= WALK_BRAKE;
                s.vy *= WALK_BRAKE;
            }
        } else if s.vx < WALK_CAP && s.direction == 1 {
            // `:59775-59782`.
            s.vx += WALK_ACCEL;
            if s.vx > WALK_CAP {
                s.vx = WALK_CAP;
            }
        } else if s.vx > 0.0 - WALK_CAP && s.direction == -1 {
            // `:59783-59790`.
            s.vx -= WALK_ACCEL;
            if s.vx < 0.0 - WALK_CAP {
                s.vx = 0.0 - WALK_CAP;
            }
        }
    }
    // `:59792-59800` — the 520 arm never runs for 350.
}

/// The dispatch seam — claims ONLY the vanilla type id 350 (the sibling
/// `frost_mob_ai_step` claims the rest of the non-boss set; the merger
/// routes 350 here instead of its no-op arm). `rng` is consumed ONLY by
/// the two cited draw sites — every other path leaves the stream
/// untouched (the witness tests pin both halves).
pub fn elf_archer_ai_step(
    type_id: u16,
    s: &mut FrostMobBState,
    world: &World,
    target: &FrostArcherTarget,
    tick: ArcherTick,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostArcherShot>,
) -> bool {
    if type_id != ELF_ARCHER_TYPE {
        return false;
    }
    elf_archer_ai(s, world, target, tick, rng, shots);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn target(x: f32, y: f32) -> FrostArcherTarget {
        FrostArcherTarget {
            x,
            y,
            width: 20.0,
            height: 42.0,
            dead: false,
            stealth: 1.0,
            item_animation: 0,
        }
    }

    // `NPC.cs:13444-13456` — the SetDefaults AI inputs the state carries.
    #[test]
    fn elf_archer_defaults_match_the_set_defaults_row() {
        let s = FrostMobBState::elf_archer(1000.0, 600.0);
        assert_eq!((s.width, s.height, s.scale), (18.0, 40.0, 1.0));
        assert_eq!((s.life, s.life_max), (900, 900));
        assert_eq!(ELF_ARCHER_TYPE, 350);
        assert_eq!(ELF_ARCHER_AI_STYLE, 3);
    }

    // ---- the aggression entry (NPC.cs:59634-59714) ----

    /// `:59680-59688` — a grounded idle archer with LOS inside 700px
    /// stops: vx halves, ai[1] = 110, ai[2] = the octant (3 for the flat
    /// right aim). Two draws consumed, in X-then-Y order.
    #[test]
    fn entry_stops_and_latches_the_cycle() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.vx = 1.0;
        let mut rng = UnifiedRandom::new(11);
        let mut shots = Vec::new();
        elf_archer_ai(&mut s, &w, &target(1300.0, 600.0), ArcherTick::default(), &mut rng, &mut shots);
        // centre (1009, 620), player centre (1310, 621): dx 301, the
        // lead 30.1 on Y → flat-right octant 3 whatever the ±40 jitter.
        assert_eq!(s.ai[1], 110.0);
        assert_eq!(s.ai[2], 3.0);
        assert_eq!(s.vx, 0.5);
        assert!(shots.is_empty());
        // The entry drew exactly two Next(-40, 41).
        let mut probe = UnifiedRandom::new(11);
        probe.next_range(-40, 41);
        probe.next_range(-40, 41);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// `:59634` — airborne / timers / dead player block the entry and
    /// draw NOTHING (the walk arm still runs).
    #[test]
    fn entry_gates_draw_nothing_when_blocked() {
        let w = air();
        let t = target(1300.0, 600.0);
        for case in 0..4 {
            let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
            let mut tt = t;
            match case {
                0 => s.vy = 0.5,       // airborne
                1 => s.ai[1] = 5.0,    // cycle still running
                2 => s.ai[2] = 3.0,    // already stopped (handled above)
                _ => tt.dead = true,   // dead player
            }
            if case == 2 {
                s.ai[1] = 60.0; // keep the stopped arm from waking it
            }
            let mut rng = UnifiedRandom::new(22);
            let mut probe = UnifiedRandom::new(22);
            let mut shots = Vec::new();
            elf_archer_ai(&mut s, &w, &tt, ArcherTick::default(), &mut rng, &mut shots);
            assert_ne!(s.ai[1], 110.0, "case {case}");
            for _ in 0..8 {
                assert_eq!(rng.next(), probe.next(), "case {case}");
            }
        }
    }

    /// `:59641-59644` — the stealth/swing refusal: a fully idle player
    /// (stealth == 0 AND itemAnimation == 0) is not engaged — no stop,
    /// no draws. Either field non-zero re-opens the gate.
    #[test]
    fn entry_refuses_the_fully_idle_player() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        let mut t = target(1300.0, 600.0);
        t.stealth = 0.0;
        t.item_animation = 0;
        let mut rng = UnifiedRandom::new(33);
        let mut probe = UnifiedRandom::new(33);
        let mut shots = Vec::new();
        elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s.ai[1], 0.0);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // A swinging player (itemAnimation > 0) IS engaged.
        t.item_animation = 10;
        elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s.ai[1], 110.0);
    }

    /// `:59652-59680` — the two jitter draws fire BEFORE the range gate:
    /// a gate-passing entry beyond 700px still consumes them but never
    /// stops.
    #[test]
    fn entry_beyond_range_draws_two_but_never_stops() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        let mut rng = UnifiedRandom::new(44);
        let mut shots = Vec::new();
        // dx ≈ 891 > 700 + the ±40 jitter reach.
        elf_archer_ai(&mut s, &w, &target(1900.0, 600.0), ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s.ai[1], 0.0);
        assert_eq!(s.ai[2], 0.0);
        let mut probe = UnifiedRandom::new(44);
        probe.next_range(-40, 41);
        probe.next_range(-40, 41);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    // ---- the fire tick (NPC.cs:59202-59618) ----

    /// The RNG WITNESS: at ai[1] == 55 the fire draws exactly two
    /// Next(-40, 41) (X then Y) and the shot is the exact composition —
    /// the |dx|*0.1 lead, the jitter, the 11f normalisation, the
    /// one-step origin shift. The stream afterwards aligns with a
    /// manual replay of the same two draws.
    #[test]
    fn fire_consumes_two_draws_in_csharp_order() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.ai[2] = 3.0;
        s.ai[1] = 56.0; // the :59088 decrement lands on 55
        let t = target(1300.0, 600.0);
        let mut rng = UnifiedRandom::new(12345);
        let mut shots = Vec::new();
        elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(shots.len(), 1);
        assert_eq!(s.ai[1], 55.0);
        // The manual replay — the same seed, the same two draws.
        let mut probe = UnifiedRandom::new(12345);
        let (cx, cy) = (1009.0f32, 620.0f32);
        let (pcx, pcy) = (1310.0f32, 621.0f32);
        let mut ax = pcx - cx;
        let lead = ax.abs() * 0.1;
        let mut ay = pcy - cy - lead;
        ax += probe.next_range(-40, 41) as f32;
        ay += probe.next_range(-40, 41) as f32;
        let k = 11.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        assert_eq!(
            shots[0],
            FrostArcherShot {
                kind: 82,
                x: cx + ax,
                y: cy + ay,
                vx: ax,
                vy: ay,
                damage: 45,
            }
        );
        // `:59595-59617` — the octant re-pick on the fired velocity.
        assert_eq!(s.ai[2], aim_octant(ax, ay));
        // Stream alignment: the next draws match the replay's continuation.
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// The other half of the witness: a stopped tick that is NOT the
    /// fire tick consumes ZERO draws and only runs the stop damp +
    /// sprite mirror (`:59626-59627`).
    #[test]
    fn stopped_non_fire_ticks_leave_the_stream_untouched() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.ai[2] = 3.0;
        s.ai[1] = 100.0;
        s.vx = 1.0;
        s.direction = -1;
        s.sprite_direction = 1;
        let mut rng = UnifiedRandom::new(999);
        let mut probe = UnifiedRandom::new(999);
        let mut shots = Vec::new();
        elf_archer_ai(&mut s, &w, &target(1300.0, 600.0), ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s.ai[1], 99.0);
        assert_eq!(s.vx, 0.9);
        assert_eq!(s.sprite_direction, -1);
        assert!(shots.is_empty());
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// `:59619-59623` — going airborne mid-stop wakes the archer
    /// (ai[2] = ai[1] = 0) with no draws; the walk arm then runs.
    #[test]
    fn airborne_wakes_the_stop() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.ai[2] = 3.0;
        s.ai[1] = 80.0;
        s.vy = 1.5;
        let mut rng = UnifiedRandom::new(55);
        let mut probe = UnifiedRandom::new(55);
        let mut shots = Vec::new();
        elf_archer_ai(&mut s, &w, &target(1300.0, 600.0), ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!((s.ai[1], s.ai[2]), (0.0, 0.0));
        assert!(shots.is_empty());
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// `:59091-59097` — the justHit interrupt drops ai[1] to 30 (below
    /// the fire tick 55) and clears the octant: no shot can leak, and
    /// the grounded archer re-enters on the SAME tick (ai[1] <= 0 fails
    /// — 30 > 0 — so it walks instead; the entry resumes 30 ticks later).
    #[test]
    fn just_hit_interrupts_without_a_shot() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.ai[2] = 3.0;
        s.ai[1] = 56.0; // would fire this tick without the interrupt
        let mut rng = UnifiedRandom::new(66);
        let mut probe = UnifiedRandom::new(66);
        let mut shots = Vec::new();
        let tick = ArcherTick { just_hit: true, confused: false };
        elf_archer_ai(&mut s, &w, &target(1300.0, 600.0), tick, &mut rng, &mut shots);
        assert_eq!((s.ai[1], s.ai[2]), (30.0, 0.0));
        assert!(shots.is_empty());
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    /// `:59197-59201` — Confused zeroes both timers before the stopped
    /// arm; the grounded archer then re-enters the same tick (the entry
    /// gates pass), drawing the entry's two — never the fire's.
    #[test]
    fn confused_clears_then_reentries() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.ai[2] = 3.0;
        s.ai[1] = 56.0;
        let mut rng = UnifiedRandom::new(77);
        let mut shots = Vec::new();
        let tick = ArcherTick { just_hit: false, confused: true };
        elf_archer_ai(&mut s, &w, &target(1300.0, 600.0), tick, &mut rng, &mut shots);
        assert!(shots.is_empty());
        assert_eq!(s.ai[1], 110.0); // the entry re-latched
        let mut probe = UnifiedRandom::new(77);
        probe.next_range(-40, 41);
        probe.next_range(-40, 41);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    // ---- the full cycle ----

    /// The 110-tick cycle end to end: entry (2 draws) → the decrement
    /// walks 110 → 55, the fire on the tick that READS 56 (2 draws) →
    /// 54 quiet stop ticks → the wake at ai[1] <= 0, which RE-ENTERS on
    /// the SAME tick (the entry gates pass — 2 draws). Total exactly
    /// 6 draws over 111 ticks, matching a same-seeded probe replay.
    #[test]
    fn the_full_cycle_consumes_exactly_six_draws() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        let t = target(1300.0, 600.0);
        let mut rng = UnifiedRandom::new(4242);
        let mut shots = Vec::new();
        // Tick 1: the entry.
        elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!((s.ai[1], s.ai[2]), (110.0, 3.0));
        // Ticks 2..56: the decrement walks 110 → 55; the fire lands on
        // the tick that READS 56 (55 stop ticks including the fire).
        for _ in 0..55 {
            elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        }
        assert_eq!(shots.len(), 1);
        assert_eq!(s.ai[1], 55.0);
        // Ticks 57..110: 54 quiet stop ticks (55 → 1).
        for _ in 0..54 {
            elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        }
        assert_eq!(s.ai[1], 1.0);
        assert_eq!(shots.len(), 1);
        // Tick 111: the decrement hits 0 → the wake (`:59619-59623`),
        // then the entry re-latches on the SAME tick (grounded, gates
        // pass) — the C# has no gap between the cycles.
        elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s.ai[1], 110.0);
        assert_eq!(shots.len(), 1);
        // Exactly six Next(-40, 41) draws over the 111 ticks.
        let mut probe = UnifiedRandom::new(4242);
        for _ in 0..6 {
            probe.next_range(-40, 41);
        }
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    // ---- the walk arm (NPC.cs:59715-59791) ----

    /// `:59717-59719` + `:59775-59790` — the cap-1 walk with the 0.07
    /// accel (the defaults, NOT the sibling walker table's rows — 350 is
    /// excluded from that walker by `:58322`).
    #[test]
    fn walk_accelerates_to_the_default_cap() {
        let w = air();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        // Far player, dead — the entry never fires; the walk runs.
        let mut t = target(5000.0, 600.0);
        t.dead = true;
        let mut rng = UnifiedRandom::new(1);
        let mut shots = Vec::new();
        for _ in 0..200 {
            elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        }
        assert_eq!(s.vx, 1.0);
        // One accel step from rest is exactly 0.07.
        let mut s2 = FrostMobBState::elf_archer(1000.0, 600.0);
        elf_archer_ai(&mut s2, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s2.vx, 0.07);
        // Facing left mirrors the arm.
        let mut s3 = FrostMobBState::elf_archer(1000.0, 600.0);
        s3.direction = -1;
        elf_archer_ai(&mut s3, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s3.vx, -0.07);
    }

    /// `:59768-59774` — the over-cap brake damps BOTH components ×0.8
    /// when grounded; airborne it does not fire (and the accel arms are
    /// skipped — the velocity is untouched).
    #[test]
    fn walk_over_cap_brake_is_grounded_only() {
        let w = air();
        let mut t = target(5000.0, 600.0);
        t.dead = true;
        let mut rng = UnifiedRandom::new(2);
        let mut shots = Vec::new();
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        s.vx = 3.0;
        s.vy = 0.0;
        elf_archer_ai(&mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!(s.vx, 3.0 * 0.8);
        let mut s2 = FrostMobBState::elf_archer(1000.0, 600.0);
        s2.vx = 3.0;
        s2.vy = 1.5;
        elf_archer_ai(&mut s2, &w, &t, ArcherTick::default(), &mut rng, &mut shots);
        assert_eq!((s2.vx, s2.vy), (3.0, 1.5));
    }

    // ---- the dispatch seam ----

    /// The seam claims exactly 350; the sibling's set and every other id
    /// fall through untouched (no state writes, no draws).
    #[test]
    fn dispatch_claims_only_350() {
        let w = air();
        let t = target(1300.0, 600.0);
        let mut s = FrostMobBState::elf_archer(1000.0, 600.0);
        let mut rng = UnifiedRandom::new(3);
        let mut shots = Vec::new();
        assert!(elf_archer_ai_step(350, &mut s, &w, &t, ArcherTick::default(), &mut rng, &mut shots));
        for id in [0u16, 3, 110, 214, 216, 338, 341, 347, 349, 351, 352, 381, 382, 520] {
            let mut s2 = FrostMobBState::elf_archer(1000.0, 600.0);
            let before = (s2.vx, s2.ai[1]);
            assert!(!elf_archer_ai_step(id, &mut s2, &w, &t, ArcherTick::default(), &mut rng, &mut shots));
            assert_eq!((s2.vx, s2.ai[1]), before);
        }
    }
}
