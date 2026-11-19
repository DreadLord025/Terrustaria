//! The aiStyle-59 family — the Pumpking's scythe blades (`NPC.AI()`'s
//! `else if (this.aiStyle == 59)` arm, `NPC.cs:33198-33373`). S1-D-3,
//! aiStyle-59 round.
//!
//! The family has ONE vanilla member: type 328, `NPCID.PumpkingBlade`
//! (`ID/NPCID.cs:11333`, `SetDefaults` row `NPC.cs:13158-13174` — the sole
//! `aiStyle = 59` assignment in the whole decompile). The blades are
//! spawned in pairs by the Pumpking (327, aiStyle 58) inside its own arm
//! (`NPC.cs:33087-33104`): the left blade `ai[0] = -1`, the right blade
//! `ai[0] = 1` with `ai[3] = 150` (a 30-tick phase lead), both `ai[1]` =
//! the carrier's `whoAmI` and `target` copied from the carrier. The
//! sibling arm `aiStyle == 58` (the Pumpking itself) is NOT touched by
//! this module; only the carrier facts the blade reads are modelled
//! ([`PumpkingCarrier`]).
//!
//! The machine (`ai[2]` is the state axis):
//! * the carrier link is read FIRST every tick — `spriteDirection =
//!   -(int)ai[0]` and, when the linked slot is missing/inactive/not
//!   aiStyle 58, `velocity *= 0.9` then the despawn (`life = -1;
//!   HitEffect(); checkDead(); active = false`, `:33203-33218`).
//!   `HitEffect` has no type-328 arm anywhere in its chain (the only
//!   `type == 328` text in NPC.cs is the SetDefaults row `:13158`; the
//!   type else-if chain's else-tail returns for any type != 156 at
//!   `:89811-89816`), so it is effect-free. `checkDead` (`:78040-78412`
//!   + `NPCLoot` `:78830`) is NOT effect-free in vanilla — it plays the
//!   row's `DeathSound` = NPCDeath1 (`:78186-78194`), calls `NPCLoot`
//!   (`:78223` — the loot rule table has no 328 entries, so no items,
//!   but the bestiary/banner kill credit can register, `:78839-78847`)
//!   and runs the wave-progress pair (`:78231-78232`; 328 has no
//!   `CheckProgressPumpkinMoon` points case — the nearest is 327's 150
//!   at `:78681` — so `num2 = 0`, no points; `CheckProgressFrostMoon`
//!   gates on `Main.snowMoon` `:78447` and returns during the pumpkin
//!   moon), and `GetNPCInvasionGroup`'s −2 group lists 327/329/330 but
//!   NOT 328 (`:78359-78376`). The port models the despawn as `hp = 0`
//!   (the kind wave's death resolution owns the cue/credit — the
//!   `npc_ai_style72.rs` precedent). NOTE the death CUE is not
//!   RNG-free: the NPCDeath1 style-4 path draws one pitch roll
//!   `Next(-10, 11) * 0.01` (`LegacySoundPlayer.cs:693`) — when the
//!   wiring lands, that draw rides the port's death-sound batch on the
//!   `UpdateNpcs` stream (the S1-E-21c convention, `world_sim/npcs.rs`'s
//!   dead pass), NOT this arm;
//! * while the carrier's scythe-face volley state is on (`ai[3] == 2`
//!   — the face mode, NOT the flee; the flee is `ai[1] == 2`), every
//!   91 ticks a Flaming Scythe (projectile 329) fires from the blade's
//!   centre toward the player at speed 0.01 — the AIM point is 30 px
//!   below the centre, the SPAWN point is the centre itself, and
//!   `rotation`/`spriteDirection` ride `NewProjectile`'s ai0/ai1 slots
//!   (`:33219-33232`);
//! * day time: sink (`vy += 0.3`, `vx *= 0.9`, `:33233-33237`);
//! * states 0/3 — hover beside the carrier: the midpoint of player and
//!   carrier ±170 px by the blade's side, +90 px down, the carrier-only
//!   anchor when the Manhattan player-carrier distance exceeds 700,
//!   the 6/9/12/15/18/21 speed ladder, the ±4 side push, the ×14/15
//!   velocity lerp and the atan2 + **1.57f literal** rotation
//!   (`:33238-33298`); the 180-tick `ai[3]` clock advances the state
//!   0 → 1 → 2 → 3 → 4 …;
//! * state 1 — rise to 200 px above the carrier's top (`vy -= 0.3`,
//!   cap −14, `vx *= 0.95`), then `TargetClosest()` + the 18-speed dive
//!   at the player, state 2 (`:33299-33324`);
//! * state 2 — dive until below the player / moving up / 800 Manhattan
//!   from the carrier, state 3 (`:33325-33333`);
//! * state 4 — the lateral wind-up: `vy *= 0.95`, `vx += 0.3 * -side`,
//!   ±14 clamps, until 500 px either side of the carrier, then
//!   `TargetClosest()` + the 17-speed sweep at the player, state 5
//!   (`:33334-33363`);
//! * state 5 — sweep until past the player (velocity-sign test) or 800
//!   Manhattan from the carrier, state 0 (`:33364-33372`).
//!
//! RNG: the arm draws NO `Main.rand` — by construction, not by gating.
//! `NPC.cs:33198-33373` contains no `Next`/`NextFloat`; neither does
//! `TargetClosest`/`TryTrackingTarget`/`SetTargetTrackingValues`
//! (`NPC.cs:77652-77781`, the `npc_target.rs` port), `EncourageDespawn`
//! (`NPC.cs:7249-7256`), the despawn's `HitEffect` type-328 path (no
//! 328 arm; the else-tail return `:89811-89816`) or the `checkDead` /
//! `NPCLoot` chain for 328 (no loot rules — the `ItemDropDatabase` has
//! no 328 registration; `num2 = 0` wave points; the one downstream
//! draw, the NPCDeath1 pitch roll `LegacySoundPlayer.cs:693`, is the
//! wiring's death-sound batch, not this arm — module map above).
//! The nearest AI draws live on the CARRIER's
//! aiStyle-58 arm — the `Main.rand.Next(3)` scythe-face roll
//! (`:33067`) and the five-draw lob volley (`Next(-50,51)`,
//! `Next(50,201)`, two `Next(-30,31)` speed jitters and the
//! `Next(326,329)` type pick, `:33078-33083`) — which this module does
//! not port; the carrier's arm keeps its own stream when it lands. The
//! shot itself is RNG-free: every number is a counter (`localAI[1]`),
//! a constant (0.01/60/0) or the aim maths. So [`ai_style59`] takes no
//! RNG parameter; the witness test `the_style59_arm_draws_no_rng` pins
//! a `UnifiedRandom` state across 600 ticks through every state
//! (written, not run — code-only mode).
//!
//! Wiring deferred (the `npc_ai_style21.rs` / S1-D-3 aiStyle-21
//! precedent): the port's roster has no `NpcKind::PumpkingBlade` yet
//! (the Pumpking itself rides the hover stand-in, `npc.rs`'s
//! `NpcKind::Pumpking` arm), so `npc.rs`'s kind dispatch has no style-59
//! arm to claim. This module is the pure port + the minimal seam: the
//! carrier facts arrive as [`PumpkingCarrier`] (the sim resolves the
//! `ai[1]` slot link — the C# reads `Main.npc[(int)MathHelper.Clamp(
//! ai[1], 0f, 199f)]`, `Main.maxNPCs = 200`, `InitData.cs:9`), the shot
//! is RETURNED as [`BladeShot`] for the sim's projectile spawn drain
//! (the `wants_servant` shape), and the despawn is the port's `hp = 0`
//! convention. See docs/ANALYSIS.md "The Pumpking blade arm (S1-D-3
//! aiStyle-59)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The type-328 `SetDefaults` row (`NPC.cs:13158-13174`), verbatim. The
/// `HitSound`/`DeathSound` are `SoundID.NPCHit1`/`NPCDeath1` =
/// `LegacySoundStyle(3, 1)`/`(4, 1)` (`ID/SoundID.cs:1257`/`:1315`).
/// `value` keeps the field default 0 (`NPC.cs:6521` — the row never
/// writes it); `scale` keeps 1.
pub struct PumpkingBladeDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1`.
    pub death_sound: (i32, i32),
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub knock_back_resist: f32,
    pub dont_take_damage: bool,
    pub npc_slots: f32,
    pub dont_count_me: bool,
    pub value: f32,
    pub scale: f32,
}

/// `NPC.cs:13158-13174` — `else if (this.type == 328)`.
pub const PUMPKING_BLADE: PumpkingBladeDefaults = PumpkingBladeDefaults {
    width: 80,
    height: 80,
    ai_style: 59,
    damage: 65,
    defense: 14,
    life_max: 5000,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    no_gravity: true,
    no_tile_collide: true,
    knock_back_resist: 0.0,
    dont_take_damage: true,
    npc_slots: 0.0,
    dont_count_me: true,
    value: 0.0,
    scale: 1.0,
};

/// `Main.npcFrameCount[328]` = 1 (`Main.cs:67082`'s array — index 328
/// is the 9th entry of the 320-329 row `3, 3, 16, 15, 16, 10, 14, 12,
/// 1, 10` at `Main.cs:67116`; the port's verbatim table
/// `render/entities.rs::NPC_FRAME_COUNT` carries it). NOTE: the
/// blade has NO case-328 arm in `FindFrame`'s `switch (this.type)`
/// (`NPC.cs:66705-77291` — the nearest is the carrier's case 327
/// `:72296`) and no post-switch aiStyle tail (`:77292` covers 39 only),
/// so the vanilla blade never advances `frame.Y` from this path; the
/// count is carried for the sheet slicing only. No frame function is
/// invented here. (The blade's VISUAL spin is the chain draw + rotation
/// at `Main.cs:22669-22700`, not sheet frames.)
pub const PUMPKING_BLADE_FRAME_COUNT: u32 = 1;

/// `Main.maxNPCs` (`Main.cs:66796` = `InitData.MaxNPCs`,
/// `InitData.cs:9`): the carrier-slot clamp's upper bound
/// (`NPC.cs:33201` clamps `ai[1]` to `0..=199`).
pub const MAX_NPCS: i32 = 200;

/// The carrier facts the blade reads — the sim resolves
/// `Main.npc[(int)MathHelper.Clamp(this.ai[1], 0f, Main.maxNPCs - 1)]`
/// (`NPC.cs:33201`) before the call and passes `None` when the slot is
/// inactive or holds a non-aiStyle-58 NPC (the `:33203` gate). The
/// blade never writes the carrier.
#[derive(Clone, Copy, Debug)]
pub struct PumpkingCarrier {
    /// `nPC3.Center.X` (`Entity.cs:50-55` — position + width/2f).
    pub center_x: f32,
    /// `nPC3.Center.Y`.
    pub center_y: f32,
    /// `nPC3.Top.Y` = `position.Y` (`Entity.cs:86-91`).
    pub top_y: f32,
    /// `nPC3.ai[1]` — 2 = the carrier's flee state (`NPC.cs:33105-33116`
    /// arms it when the target dies or strays 2000 px; read at
    /// `:33240`).
    pub ai1: f32,
    /// `nPC3.ai[3]` — 2 = the scythe-face volley state (`:33067`'s
    /// `Main.rand.Next(3)` roll; read at `:33219`).
    pub ai3: f32,
}

/// The Flaming Scythe shot (`NPC.cs:33230`):
/// `Projectile.NewProjectile(source, base.Center.X, base.Center.Y,
/// num881 * num883, num882 * num883, 329, 60, 0f, Main.myPlayer,
/// this.rotation, this.spriteDirection)` — the C# centres the projectile
/// on (X, Y) (`Projectile.cs:10876-10877`), type 329 =
/// `ProjectileID.FlamingScythe` (`ID/ProjectileID.cs:1212`, the
/// `SetDefaults` row `Projectile.cs:4215-4225`: 80×80, aiStyle 56,
/// hostile, penetrate −1, no tile collide, timeLeft 420), damage 60,
/// knockback 0, and `rotation`/`spriteDirection` ride the ai0/ai1 slots
/// (`Projectile.cs:10862`'s signature). Returned for the sim's spawn
/// drain; the port's owner convention resolves `Main.myPlayer` there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BladeShot {
    /// Spawn X — the blade's `Center.X` (NOT the +30 aim point).
    pub x: f32,
    /// Spawn Y — the blade's `Center.Y`.
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `ProjectileID.FlamingScythe`.
    pub kind: i32,
    pub damage: i32,
    pub knockback: f32,
    /// `NewProjectile`'s ai0 — the blade's rotation at fire time.
    pub ai0: f32,
    /// `NewProjectile`'s ai1 — the blade's spriteDirection as float.
    pub ai1: f32,
}

/// The aiStyle-59 arm (`NPC.cs:33198-33373`). One fixed step of the
/// blade machine. `players`/`ticker` feed the two mid-machine
/// `TargetClosest()` calls (`:33314`, `:33353`) through the shared
/// `npc_target.rs` port (`NPC.cs:77652-77781`); `day_time` is
/// `Main.dayTime`. Returns the tick's Flaming Scythe shot, if the
/// 91-tick volley fired.
///
/// The silent despawn (`:33206-33216`, `Main.netMode != 1` — the port
/// is host/single-player) is the port's `hp = 0` convention: `life =
/// -1` + `active = false`, with `HitEffect`/`checkDead` effect-free for
/// type 328 (module docs). The `SendData(28)` net arm is a wiring
/// follow-up (`net_update` is left for the sim's fan-out).
pub fn ai_style59(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    carrier: Option<&PumpkingCarrier>,
    day_time: bool,
    ticker: u64,
) -> Option<BladeShot> {
    // `:33200-33202` — the side and the mirror flip, BEFORE the gate.
    let num879 = npc.ai[0] as i32;
    npc.sprite_dir = -num879;
    // `:33203-33218` — no living aiStyle-58 carrier: damp and despawn.
    let carrier = match carrier {
        Some(c) => *c,
        None => {
            npc.vx *= 0.9;
            npc.vy *= 0.9;
            // `life = -1; HitEffect(); checkDead(); active = false` —
            // both effects are no-ops for 328 (module docs).
            npc.hp = 0;
            return None;
        }
    };
    let (cx, cy) = npc.center();
    let p = &players[npc.target as usize];
    // `Entity.Center` (`Entity.cs:50-55`) — float halves of the int box.
    let pcx = p.x + p.width as f32 / 2.0;
    let pcy = p.y + p.height as f32 / 2.0;
    // `Main.player[this.target].Top.Y` = position.Y (`Entity.cs:86-91`).
    let ptop = p.y;

    let mut shot = None;
    // `:33219-33232` — the face-mode volley (netMode != 1).
    if carrier.ai3 == 2.0 {
        npc.local_ai[1] += 1.0;
        if npc.local_ai[1] > 90.0 {
            npc.local_ai[1] = 0.0;
            // `:33225-33229` — the aim point is 30 px BELOW the centre,
            // the aim target is the player's Top; speed 0.01 over the
            // floored length.
            let num880 = 0.01f32;
            let aim_x = cx;
            let aim_y = cy + 30.0;
            let num881 = pcx - aim_x;
            let num882 = ptop - aim_y;
            let num883 = num880 / (num881 * num881 + num882 * num882).sqrt().max(1.0);
            // `:33230` — the SPAWN point is the centre itself.
            shot = Some(BladeShot {
                x: cx,
                y: cy,
                vx: num881 * num883,
                vy: num882 * num883,
                kind: 329,
                damage: 60,
                knockback: 0.0,
                ai0: npc.rotation,
                ai1: npc.sprite_dir as f32,
            });
        }
    }

    if day_time {
        // `:33233-33237` — the daylight sink.
        npc.vy += 0.3;
        npc.vx *= 0.9;
    } else if npc.ai[2] == 0.0 || npc.ai[2] == 3.0 {
        // `:33238-33298` — the hover states.
        if carrier.ai1 == 2.0 {
            // `:33240-33243` — EncourageDespawn(10) (`NPC.cs:7249-7256`
            // via the port's `despawn_timer`, `npc.rs`).
            npc.encourage_despawn(10);
        }
        npc.ai[3] += 1.0;
        if npc.ai[3] >= 180.0 {
            npc.ai[2] += 1.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        // `:33251-33260` — the hover anchor: the player-carrier
        // midpoint ±170 by side, +90 down; the carrier-only anchor past
        // the 700 Manhattan gate.
        let mut num885 = (pcx + carrier.center_x) / 2.0;
        let mut num886 = (pcy + carrier.center_y) / 2.0;
        num885 += (-170 * num879) as f32 - cx;
        num886 += 90.0 - cy;
        if (pcx - carrier.center_x).abs() + (pcy - carrier.center_y).abs() > 700.0 {
            num885 = carrier.center_x - (170 * num879) as f32 - cx;
            num886 = carrier.center_y + 90.0 - cy;
        }
        // `:33261-33282` — the floored length and the speed ladder.
        let mut num887 = (num885 * num885 + num886 * num886).sqrt().max(1.0);
        let num888 = if num887 > 1000.0 {
            21
        } else if num887 > 800.0 {
            18
        } else if num887 > 600.0 {
            15
        } else if num887 > 400.0 {
            12
        } else if num887 > 200.0 {
            9
        } else {
            6
        };
        // `:33283-33290` — the ±4 side push (BEFORE the scale).
        if num879 < 0 && cx > carrier.center_x {
            num885 -= 4.0;
        }
        if num879 > 0 && cx < carrier.center_x {
            num885 += 4.0;
        }
        // `:33291-33293` — the scale over the PRE-push length, then the
        // ×14/15 lerp on both axes.
        num887 = num888 as f32 / num887;
        npc.vx = (npc.vx * 14.0 + num885 * num887) / 15.0;
        npc.vy = (npc.vy * 14.0 + num886 * num887) / 15.0;
        // `:33294-33297` — the rotation gate re-floors the POST-push
        // length; +1.57f is the C# literal, NOT π/2.
        if (num885 * num885 + num886 * num886).sqrt().max(1.0) > 20.0 {
            npc.rotation = num886.atan2(num885) + 1.57;
        }
    } else if npc.ai[2] == 1.0 {
        // `:33299-33324` — the rise above the carrier.
        let mut num889 = carrier.center_x - (200 * num879) as f32 - cx;
        let mut num890 = carrier.top_y + 230.0 - cy;
        // `:33304-33305` — num891 computed then overwritten at the
        // transition; the rotation has NO gate here (unlike the hover).
        let _num891 = (num889 * num889 + num890 * num890).sqrt().max(1.0);
        npc.rotation = num890.atan2(num889) + 1.57;
        npc.vx *= 0.95;
        npc.vy -= 0.3;
        if npc.vy < -14.0 {
            npc.vy = -14.0;
        }
        // `:33312-33323` — 200 px above the carrier's TOP: re-target and
        // dive at the player at speed 18.
        if npc.y < carrier.top_y - 200.0 {
            target_closest(npc, world, players, true, 59, ticker);
            npc.ai[2] = 2.0;
            let p = &players[npc.target as usize];
            let pcx = p.x + p.width as f32 / 2.0;
            let pcy = p.y + p.height as f32 / 2.0;
            num889 = pcx - cx;
            num890 = pcy - cy;
            let num891 = 18.0 / (num889 * num889 + num890 * num890).sqrt().max(1.0);
            npc.vx = num889 * num891;
            npc.vy = num890 * num891;
            npc.net_update = true;
        }
    } else if npc.ai[2] == 2.0 {
        // `:33325-33333` — the dive's exit: below the player's position
        // (NOT centre), moving up, or 800 Manhattan from the carrier.
        let num892 = (cx - carrier.center_x).abs() + (cy - carrier.center_y).abs();
        if npc.y > p.y || npc.vy < 0.0 || num892 > 800.0 {
            npc.ai[2] = 3.0;
            npc.net_update = true;
        }
    } else if npc.ai[2] == 4.0 {
        // `:33334-33363` — the lateral wind-up.
        let mut num893 = carrier.center_x - (200 * num879) as f32 - cx;
        let mut num894 = carrier.top_y + 230.0 - cy;
        let _num895 = (num893 * num893 + num894 * num894).sqrt().max(1.0);
        npc.rotation = num894.atan2(num893) + 1.57;
        npc.vy *= 0.95;
        npc.vx += 0.3 * (-num879) as f32;
        if npc.vx < -14.0 {
            npc.vx = -14.0;
        }
        if npc.vx > 14.0 {
            npc.vx = 14.0;
        }
        // `:33351-33362` — 500 px either side of the carrier: re-target
        // and sweep at the player at speed 17.
        if cx < carrier.center_x - 500.0 || cx > carrier.center_x + 500.0 {
            target_closest(npc, world, players, true, 59, ticker);
            npc.ai[2] = 5.0;
            let p = &players[npc.target as usize];
            let pcx = p.x + p.width as f32 / 2.0;
            let pcy = p.y + p.height as f32 / 2.0;
            num893 = pcx - cx;
            num894 = pcy - cy;
            let num895 = 17.0 / (num893 * num893 + num894 * num894).sqrt().max(1.0);
            npc.vx = num893 * num895;
            npc.vy = num894 * num895;
            npc.net_update = true;
        }
    } else if npc.ai[2] == 5.0 {
        // `:33364-33372` — the sweep's exit: past the player on the
        // velocity's sign, or 800 Manhattan from the carrier.
        let num896 = (cx - carrier.center_x).abs() + (cy - carrier.center_y).abs();
        if (npc.vx > 0.0 && cx > pcx) || (npc.vx < 0.0 && cx < pcx) || num896 > 800.0 {
            npc.ai[2] = 0.0;
            npc.net_update = true;
        }
    }
    shot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::unified_random::UnifiedRandom;

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

    /// A blade-shaped NPC: the roster has no PumpkingBlade kind yet, so
    /// the fixture rides a placeholder kind with the C#'s 80×80 box
    /// (`NPC.cs:13160-13161`) through `size_override`, the left-blade
    /// spawn payload (`ai[0] = -1`, `NPC.cs:33094`) and target 0.
    fn blade(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((80.0, 80.0));
        n.ai[0] = -1.0;
        n.ai[1] = 7.0; // the carrier slot link (resolved by the caller)
        n.target = 0;
        n
    }

    fn carrier(x: f32, y: f32) -> PumpkingCarrier {
        // The Pumpking's 100×100 box (`NPC.cs:13144-13145`): position →
        // centre (+50, +50), top = position.Y.
        PumpkingCarrier {
            center_x: x + 50.0,
            center_y: y + 50.0,
            top_y: y,
            ai1: 0.0,
            ai3: 0.0,
        }
    }

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    /// `NPC.cs:13158-13174` — the SetDefaults row, field by field.
    #[test]
    fn the_defaults_row_matches_the_csharp() {
        let d = &PUMPKING_BLADE;
        assert_eq!((d.width, d.height), (80, 80));
        assert_eq!(d.ai_style, 59);
        assert_eq!(d.damage, 65);
        assert_eq!(d.defense, 14);
        assert_eq!(d.life_max, 5000);
        assert_eq!(d.hit_sound, (3, 1), "SoundID.NPCHit1 (SoundID.cs:1257)");
        assert_eq!(d.death_sound, (4, 1), "SoundID.NPCDeath1 (SoundID.cs:1315)");
        assert!(d.no_gravity);
        assert!(d.no_tile_collide);
        assert_eq!(d.knock_back_resist, 0.0);
        assert!(d.dont_take_damage);
        assert_eq!(d.npc_slots, 0.0);
        assert!(d.dont_count_me);
        assert_eq!(d.value, 0.0, "the row never writes value (NPC.cs:6521)");
        assert_eq!(d.scale, 1.0);
        assert_eq!(PUMPKING_BLADE_FRAME_COUNT, 1, "Main.cs:67116 col 9");
        assert_eq!(
            crate::render::entities::npc_rows(328),
            1,
            "the port's verbatim table agrees"
        );
        assert_eq!(MAX_NPCS, 200, "InitData.cs:9");
    }

    /// `NPC.cs:33200-33218` — the flip is written BEFORE the gate; a
    /// missing carrier damps ×0.9 and despawns silently (hp 0), even in
    /// day time (the gate returns before the sink arm).
    #[test]
    fn the_missing_carrier_damps_and_despawns() {
        let w = air();
        let mut n = blade(1000.0, 1000.0);
        n.vx = 10.0;
        n.vy = -5.0;
        let s = ai_style59(&mut n, &w, &[cand(1200.0, 1100.0)], None, true, 0);
        assert!(s.is_none());
        assert_eq!(n.sprite_dir, 1, "-(int)ai[0] = -(-1) (:33202)");
        assert_eq!(n.vx, 9.0);
        assert_eq!(n.vy, -4.5);
        assert_eq!(n.hp, 0, "life = -1 / active = false → hp 0");
        // The right blade mirrors the other way.
        let mut m = blade(1000.0, 1000.0);
        m.ai[0] = 1.0;
        let _ = ai_style59(&mut m, &w, &[cand(1200.0, 1100.0)], None, false, 0);
        assert_eq!(m.sprite_dir, -1);
    }

    /// `NPC.cs:33219-33232` — the volley runs only in the carrier's
    /// ai[3] == 2 state; localAI[1] increments per tick and fires on the
    /// 91st (> 90), resetting to 0. The spawn point is the CENTRE, the
    /// aim point is centre+30 → the player's TOP, speed 0.01, and the
    /// rotation/sprite ride ai0/ai1.
    #[test]
    fn the_flee_volley_fires_every_91_ticks_from_the_centre() {
        let w = air();
        let mut n = blade(1000.0, 1000.0); // centre (1040, 1040)
        n.rotation = 0.25;
        let mut c = carrier(1200.0, 1000.0);
        c.ai3 = 2.0;
        let p = cand(1030.0, 640.0); // centre (1040, 661), top 640
        let mut fired = 0;
        for tick in 1..=182 {
            let s = ai_style59(&mut n, &w, &[p], Some(&c), true, 0);
            if let Some(s) = s {
                fired += 1;
                assert!(tick == 91 || tick == 182, "fired at tick {tick}");
                assert_eq!((s.x, s.y), (1040.0, 1040.0), "the centre, not +30");
                assert_eq!((s.kind, s.damage), (329, 60));
                assert_eq!(s.knockback, 0.0);
                assert_eq!(s.ai0, n.rotation);
                assert_eq!(s.ai1, 1.0, "spriteDirection = -(-1)");
                // aim (1040, 1070) → player top (1040, 640): straight
                // up, length 430, speed 0.01.
                assert_eq!(s.vx, 0.0);
                // f32 divide-then-multiply round trip: ≤2 ulps of 0.01.
                assert!((s.vy + 0.01).abs() < 1e-7, "vy = {}", s.vy);
            }
        }
        assert_eq!(fired, 2);
        // Not in the volley state: the counter never runs.
        let mut m = blade(1000.0, 1000.0);
        let c2 = carrier(1200.0, 1000.0); // ai3 = 0
        for _ in 0..200 {
            assert!(ai_style59(&mut m, &w, &[p], Some(&c2), true, 0).is_none());
        }
        assert_eq!(m.local_ai[1], 0.0);
    }

    /// `NPC.cs:33233-33237` — day time overrides every state: the sink
    /// runs, the state machine does not advance.
    #[test]
    fn the_daylight_sink_overrides_the_states() {
        let w = air();
        let mut n = blade(1000.0, 1000.0);
        n.vx = 10.0;
        n.vy = 1.0;
        let c = carrier(1200.0, 1000.0);
        let _ = ai_style59(&mut n, &w, &[cand(1200.0, 1100.0)], Some(&c), true, 0);
        assert_eq!(n.vx, 9.0);
        assert_eq!(n.vy, 1.3);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0, "the hover clock does not run in day time");
    }

    /// `NPC.cs:33244-33250` — the 180-tick clock: ai[3] hits 180 on the
    /// 180th night tick, ai[2] advances 0 → 1 and netUpdate arms. The
    /// right blade's spawn lead (`ai[3] = 150`, `:33101`) shortens the
    /// first cycle to 30 ticks.
    #[test]
    fn the_hover_clock_advances_the_state_at_180() {
        let w = air();
        let mut n = blade(1000.0, 1000.0);
        let c = carrier(1000.0, 1000.0);
        for _ in 0..179 {
            let _ = ai_style59(&mut n, &w, &[cand(1200.0, 1100.0)], Some(&c), false, 0);
        }
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 179.0);
        let _ = ai_style59(&mut n, &w, &[cand(1200.0, 1100.0)], Some(&c), false, 0);
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.ai[3], 0.0);
        assert!(n.net_update);
        // The right blade's 150 lead: 30 ticks to the first advance.
        let mut m = blade(1000.0, 1000.0);
        m.ai[0] = 1.0;
        m.ai[3] = 150.0;
        for _ in 0..30 {
            let _ = ai_style59(&mut m, &w, &[cand(1200.0, 1100.0)], Some(&c), false, 0);
        }
        assert_eq!(m.ai[2], 1.0);
    }

    /// `NPC.cs:33240-33243` — the fleeing carrier (ai[1] == 2) encourages
    /// the 10-tick despawn from BOTH hover states (0 and 3), and only
    /// lowers an already-shorter window (`EncourageDespawn`,
    /// `NPC.cs:7249-7256`).
    #[test]
    fn the_fleeing_carrier_encourages_the_despawn() {
        let w = air();
        let mut c = carrier(1000.0, 1000.0);
        c.ai1 = 2.0;
        for state in [0.0, 3.0] {
            let mut n = blade(1000.0, 1000.0);
            n.ai[2] = state;
            let _ = ai_style59(&mut n, &w, &[cand(1200.0, 1100.0)], Some(&c), false, 0);
            assert_eq!(n.despawn_timer, 10, "state {state}");
        }
        // A shorter window is never extended.
        let mut n = blade(1000.0, 1000.0);
        n.despawn_timer = 5;
        let _ = ai_style59(&mut n, &w, &[cand(1200.0, 1100.0)], Some(&c), false, 0);
        assert_eq!(n.despawn_timer, 5);
    }

    /// `NPC.cs:33251-33260` — the anchor: within the 700 Manhattan gate
    /// the midpoint ±170/+90; past it the carrier-only anchor. The
    /// left blade (side −1) hovers to the carrier's RIGHT (+170).
    #[test]
    fn the_hover_anchor_switches_at_the_700_manhattan_gate() {
        let w = air();
        let c = carrier(1000.0, 1000.0); // centre (1050, 1050)
        // Near player: centre (1100, 1071) — Manhattan 71 ≤ 700.
        let mut n = blade(1000.0, 1000.0); // centre (1040, 1040)
        let _ = ai_style59(&mut n, &w, &[cand(1090.0, 1050.0)], Some(&c), false, 0);
        // midpoint (1075, 1060.5); left blade: +170 → 1245, +90 →
        // 1150.5; deltas (205, 110.5); no side push (cx 1040 < 1050).
        // length ≈ 232.9 > 200 → speed 9.
        let len = (205.0f32 * 205.0 + 110.5 * 110.5).sqrt();
        let scale = 9.0 / len;
        assert!((n.vx - 205.0 * scale / 15.0).abs() < 1e-4, "vx {}", n.vx);
        assert!((n.vy - 110.5 * scale / 15.0).abs() < 1e-4, "vy {}", n.vy);
        // Far player: centre (1900, 1071) — Manhattan 871 > 700: the
        // anchor is carrier-only (1050+170, 1050+90) = (1220, 1140).
        let mut m = blade(1000.0, 1000.0);
        let _ = ai_style59(&mut m, &w, &[cand(1890.0, 1050.0)], Some(&c), false, 0);
        let len2 = (180.0f32 * 180.0 + 100.0 * 100.0).sqrt();
        // len2 ≈ 205.9 > 200 → the ladder's 9 band.
        assert!(len2 > 200.0);
        let scale2 = 9.0 / len2;
        assert!((m.vx - 180.0 * scale2 / 15.0).abs() < 1e-4);
        assert!((m.vy - 100.0 * scale2 / 15.0).abs() < 1e-4);
    }

    /// `NPC.cs:33261-33297` — the speed ladder's bands, the ±4 side push
    /// applied AFTER the length (the scale uses the pre-push length),
    /// and the rotation's post-push >20 gate with the 1.57f literal.
    #[test]
    fn the_ladder_push_and_rotation_gate() {
        let w = air();
        // Carrier centre (1050, 1050); blade centre (1060, 1040) — the
        // left blade is RIGHT of the carrier → the −4 push fires.
        let c = carrier(1000.0, 1000.0);
        let mut n = blade(1020.0, 1000.0);
        // Player centre (1050, 1050) = the carrier centre: midpoint =
        // carrier centre; anchor (1220, 1140) − (1060, 1040) =
        // (160, 100), length ≈ 188.7 ≤ 200 → speed 6. Push: 160 − 4.
        let _ = ai_style59(&mut n, &w, &[cand(1040.0, 1029.0)], Some(&c), false, 0);
        let len = (160.0f32 * 160.0 + 100.0 * 100.0).sqrt();
        let scale = 6.0 / len; // the PRE-push length
        assert!((n.vx - 156.0 * scale / 15.0).abs() < 1e-4, "vx {}", n.vx);
        assert!((n.vy - 100.0 * scale / 15.0).abs() < 1e-4);
        // The rotation: atan2(100, 156) + the 1.57 LITERAL.
        let want = 100.0f32.atan2(156.0) + 1.57;
        assert!((n.rotation - want).abs() < 1e-6);
        assert!(
            (n.rotation - (100.0f32.atan2(156.0) + std::f32::consts::FRAC_PI_2)).abs() > 1e-4,
            "1.57f is not π/2"
        );
        // The >20 gate: an anchor within 20 px keeps the old rotation.
        let mut m = blade(1000.0, 1000.0);
        m.rotation = 0.5;
        // The blade centre is (1040, 1040). The carrier-only anchor is
        // centre +170/+90 for the left blade → put the carrier centre
        // at (870, 950): anchor (1040, 1040). The player must be FAR
        // (Manhattan > 700) so the carrier-only arm runs: centre
        // (3010, 3021), Manhattan 2140 + 2071 > 700.
        let c2 = PumpkingCarrier {
            center_x: 870.0,
            center_y: 950.0,
            top_y: 900.0,
            ai1: 0.0,
            ai3: 0.0,
        };
        let _ = ai_style59(&mut m, &w, &[cand(3000.0, 3000.0)], Some(&c2), false, 0);
        // anchor (1040, 1040) − centre (1040, 1040) = (0, 0); the left
        // blade is right of the carrier (1040 > 870) → push −4 →
        // length 4 ≤ 20: the rotation holds.
        assert_eq!(m.rotation, 0.5);
    }

    /// `NPC.cs:33299-33324` — state 1: the rise damps X ×0.95, pulls Y
    /// −0.3 to the −14 cap, aims at (carrier ±200, top+230) with NO
    /// gate; crossing 200 px above the carrier's top re-targets (the
    /// npc_target stamp) and dives at the player at speed 18.
    #[test]
    fn state1_rises_then_dives_at_18() {
        let w = air();
        let c = carrier(1000.0, 1000.0); // top 1000, centre (1050, 1050)
        let mut n = blade(1000.0, 900.0); // centre (1040, 940)
        n.ai[2] = 1.0;
        n.vx = 10.0;
        n.vy = -13.9;
        let p = cand(1400.0, 1400.0); // centre (1410, 1421)
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, 42);
        assert_eq!(n.vx, 9.5);
        assert_eq!(n.vy, -14.0, "the −14 cap");
        assert_eq!(n.ai[2], 1.0, "y 900 is not < 1000 − 200");
        let want = (1000.0 + 230.0 - 940.0f32).atan2(1050.0 + 200.0 - 1040.0) + 1.57;
        assert!((n.rotation - want).abs() < 1e-6);
        // Above the line: y 799 < 800 → the dive.
        n.y = 799.0; // centre (1040, 839)
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, 42);
        assert_eq!(n.ai[2], 2.0);
        assert!(n.net_update);
        assert_eq!(n.target_set_frame, 42, "TargetClosest ran (:33314)");
        let dx = 1410.0 - 1040.0f32;
        let dy = 1421.0 - 839.0;
        let s = 18.0 / (dx * dx + dy * dy).sqrt();
        assert!((n.vx - dx * s).abs() < 1e-3, "vx {}", n.vx);
        assert!((n.vy - dy * s).abs() < 1e-3);
        assert!((n.vx * n.vx + n.vy * n.vy).sqrt() - 18.0 < 1e-3);
    }

    /// `NPC.cs:33325-33333` — state 2's three exits: below the player's
    /// POSITION (not centre), moving up, or 800 Manhattan from the
    /// carrier.
    #[test]
    fn state2_exits_on_position_upward_or_distance() {
        let w = air();
        let c = carrier(1000.0, 1000.0);
        let p = cand(1040.0, 1000.0);
        // Below the player's position.Y (1000): y 1001 fires even with
        // vy > 0 and a near carrier.
        let mut n = blade(1000.0, 1001.0);
        n.ai[2] = 2.0;
        n.vy = 5.0;
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, 0);
        assert_eq!(n.ai[2], 3.0);
        // Moving up fires regardless of position.
        let mut m = blade(1000.0, 900.0);
        m.ai[2] = 2.0;
        m.vy = -1.0;
        let _ = ai_style59(&mut m, &w, &[p], Some(&c), false, 0);
        assert_eq!(m.ai[2], 3.0);
        // 800 Manhattan from the carrier fires (STRICTLY >): centre
        // (1841, 1040) → 791 + 10 = 801 > 800. (1840 would be exactly
        // 800 and hold.)
        let mut k = blade(1801.0, 1000.0);
        k.ai[2] = 2.0;
        k.vy = 5.0;
        let _ = ai_style59(&mut k, &w, &[p], Some(&c), false, 0);
        assert_eq!(k.ai[2], 3.0);
        // None of the three: hold.
        let mut h = blade(1000.0, 900.0);
        h.ai[2] = 2.0;
        h.vy = 5.0;
        let _ = ai_style59(&mut h, &w, &[p], Some(&c), false, 0);
        assert_eq!(h.ai[2], 2.0);
    }

    /// `NPC.cs:33334-33363` — state 4: the lateral wind-up damps Y
    /// ×0.95, accelerates X by 0.3·(−side) to the ±14 clamps; 500 px
    /// either side of the carrier re-targets and sweeps at 17.
    #[test]
    fn state4_winds_up_then_sweeps_at_17() {
        let w = air();
        let c = carrier(1000.0, 1000.0); // centre (1050, 1050)
        let mut n = blade(1000.0, 900.0); // centre (1040, 940)
        n.ai[2] = 4.0;
        n.vx = 13.9;
        n.vy = 10.0;
        let p = cand(600.0, 1400.0); // centre (610, 1421)
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, 77);
        assert_eq!(n.vx, 14.0, "0.3·(−(−1)) = +0.3, the +14 cap");
        assert_eq!(n.vy, 9.5);
        assert_eq!(n.ai[2], 4.0, "1040 is within ±500 of 1050");
        // Past the left line: centre 549 < 1050 − 500.
        n.x = 509.0; // centre (549, 940)
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, 77);
        assert_eq!(n.ai[2], 5.0);
        assert_eq!(n.target_set_frame, 77, "TargetClosest ran (:33353)");
        let dx = 610.0 - 549.0f32;
        let dy = 1421.0 - 940.0;
        let s = 17.0 / (dx * dx + dy * dy).sqrt();
        assert!((n.vx - dx * s).abs() < 1e-3);
        assert!((n.vy - dy * s).abs() < 1e-3);
        // The right blade winds the other way (−0.3, the −14 clamp).
        let mut m = blade(1000.0, 900.0);
        m.ai[0] = 1.0;
        m.ai[2] = 4.0;
        m.vx = -13.9;
        let _ = ai_style59(&mut m, &w, &[p], Some(&c), false, 0);
        assert_eq!(m.vx, -14.0);
    }

    /// `NPC.cs:33364-33372` — state 5's exits: past the player on the
    /// velocity's sign (both directions), or 800 Manhattan from the
    /// carrier; otherwise hold.
    #[test]
    fn state5_exits_past_the_player_or_far_from_the_carrier() {
        let w = air();
        let c = carrier(1000.0, 1000.0);
        let p = cand(1040.0, 1000.0); // centre (1050, 1021)
        // Moving right, past the player's centre.
        let mut n = blade(1020.0, 900.0); // centre (1060, 940) > 1050
        n.ai[2] = 5.0;
        n.vx = 5.0;
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, 0);
        assert_eq!(n.ai[2], 0.0);
        assert!(n.net_update);
        // Moving left, past the player's centre.
        let mut m = blade(980.0, 900.0); // centre (1020, 940) < 1050
        m.ai[2] = 5.0;
        m.vx = -5.0;
        let _ = ai_style59(&mut m, &w, &[p], Some(&c), false, 0);
        assert_eq!(m.ai[2], 0.0);
        // Moving right but still LEFT of the player: hold.
        let mut h = blade(980.0, 900.0);
        h.ai[2] = 5.0;
        h.vx = 5.0;
        let _ = ai_style59(&mut h, &w, &[p], Some(&c), false, 0);
        assert_eq!(h.ai[2], 5.0);
        // 800 Manhattan from the carrier fires regardless — vx 0 so
        // neither velocity-sign arm can fire: centre (1841, 1040) →
        // 791 + 10 = 801 > 800 (the strict gate).
        let mut k = blade(1801.0, 1000.0);
        k.ai[2] = 5.0;
        k.vx = 0.0;
        let _ = ai_style59(&mut k, &w, &[p], Some(&c), false, 0);
        assert_eq!(k.ai[2], 0.0);
    }

    /// The RNG witness (module docs): the arm — through every state, the
    /// volley, the despawn and both TargetClosest calls — draws NOTHING.
    /// A seeded `UnifiedRandom`'s full state (cursor + 56-int seed array,
    /// `PartialEq`) is bit-identical across 600 ticks — long enough to
    /// walk hover(0, 180 ticks) → rise(1) → dive(2) → hover(3, 180
    /// ticks) → wind-up(4) → sweep(5) and back — so no stream can shift
    /// when the wiring lands. The carrier's own draws
    /// (`NPC.cs:33067`, `:33078-33083`) live on the aiStyle-58 arm and
    /// are not this module's stream.
    #[test]
    fn the_style59_arm_draws_no_rng() {
        let w = air();
        let rng = UnifiedRandom::new(12345);
        let witness = rng.clone();
        let mut c = carrier(1000.0, 1000.0);
        let p = cand(1400.0, 1400.0);
        let mut n = blade(1000.0, 1000.0);
        // 600 night ticks walk 0 → 1 → 2 → 3 → 4 → 5 → 0 with the
        // volley on.
        c.ai3 = 2.0;
        for tick in 0..600 {
            let _ = ai_style59(&mut n, &w, &[p], Some(&c), false, tick as u64);
            n.x += n.vx;
            n.y += n.vy;
            if n.hp == 0 {
                n = blade(1000.0, 1000.0);
            }
        }
        // The day arm and the despawn arm too.
        let _ = ai_style59(&mut n, &w, &[p], Some(&c), true, 600);
        let _ = ai_style59(&mut n, &w, &[p], None, false, 601);
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
