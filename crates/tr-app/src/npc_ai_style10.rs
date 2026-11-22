//! The aiStyle-10 machine (S1-D-3 aiStyle-10 slice, code-only mode) —
//! `NPC.AI()`'s `if (this.aiStyle == 10)` arm (`NPC.cs:21572-21918`),
//! shared by exactly three kinds (`SetDefaults`' `aiStyle = 10` rows):
//!
//! - **Cursed Skull** (`NPCID.CursedSkull` 34, `ID/NPCID.cs:10745`,
//!   `SetDefaults` `NPC.cs:9150-9165`: 26×28, dmg 35, def 6, 40 HP,
//!   NPCHit2/NPCDeath2, noGravity + noTileCollide, value 150, kb 0.2,
//!   npcSlots 0.75);
//! - **Giant Cursed Skull** (`NPCID.GiantCursedSkull` 289,
//!   `ID/NPCID.cs:11255`, `NPC.cs:12753-12768`: 44×44, dmg 60, def 20,
//!   400 HP, the same flags/value/kb/slots) — the per-type branch at
//!   `:21767-21823` fires the Shadowflames bolt (299,
//!   `ID/ProjectileID.cs:1152`);
//! - **Water Bolt Mimic** (`NPCID.WaterBoltMimic` 694,
//!   `ID/NPCID.cs:12065`, `NPC.cs:17737-17751`: 44×44, dmg 20, def 4,
//!   60 HP, DD2_BookStaffCast/NPCDeath52, noGravity + noTileCollide,
//!   value 150, kb 0.2) — the book machine at `:21824-21917` (the
//!   charge, the dash, the Water Sphere 33 summon).
//!
//! The shared core (`:21574-21766`): the 600-tick `ai[1]` cycle with the
//! 650 wrap, the distance-banded speed/accel table, the dead-target flee
//! vector, the 289/694 charge (`ai[3] == 2`) and dash (`ai[3] == 1`)
//! overrides, the aim rotation with the 694 sprite flip, and the two
//! stationary book states (`ai[3] == 3` closed / `ai[3] == 4` just-hit
//! wake-up). The collision side is `noTileCollide` (`SetDefaults`), so
//! the step is a raw integration (`UpdateCollision`'s dry move,
//! `NPC.cs:92538-92540` + `:92605-92607`); `Collision_DecideFallThroughPlatforms`
//! forces `aiStyle == 10` through platforms (`NPC.cs:92924-92927`) — moot
//! under noTileCollide.
//!
//! **RNG by construction**: the ONLY `Main.rand` draw in the whole arm is
//! `Main.rand.Next(3)` at `:21851`, inside the 694 tail's
//! `flag19 && (!flag20 || ...)` gate — short-circuit `||` means the draw
//! runs ONLY when `flag19 && flag20` both hold, exactly once per such
//! tick. The 34/289 paths draw NOTHING (the 289 volley at `:21804-21811`
//! is a deterministic tick-20 trigger). The draw rides the caller's
//! `UpdateNPCs` stream (`Main.rand`, `Main.cs:17767`) via
//! `UnifiedRandom::next_max(3)` (`UnifiedRandom.cs:134-141`).
//!
//! Deliberate stand-ins (house pattern, recorded in ANALYSIS "The
//! aiStyle-10 family"):
//! - **WIRED by W4-c3 (2026-11-20).** `NpcKind::CursedSkull` /
//!   `GiantCursedSkull` / `WaterBoltMimic` now exist; the style axis' 10
//!   arm (`npc.rs::update_rng_dust`) calls `style10_ai` with
//!   `npc_type = kind.cs_id()` (34/289/694) and `just_hit = self.just_hit`.
//!   The machine stays keyed on the vanilla `NPCID` (the module's pure
//!   shape); the `SetDefaults` box now rides the kind's own `size()` row
//!   (the `size_override` stand-in is retired for the wired kinds).
//! - `justHit` folds onto a caller-supplied flag (the port's `Npc` has
//!   a `just_hit` field now; the axis arm passes it — the module keeps
//!   the explicit parameter to stay pure).
//! - `knockBackResist` writes land in [`Style10Out::knock_back_resist`];
//!   the axis arm maps them onto `Npc::kb_resist_override` (the runtime
//!   override slot `knock_back_resist()` prefers over the kind table).
//! - `SoundEngine.PlaySound(SoundID.Item8, ...)` (`:21896`, the legacy
//!   (2, 8) cue, `ID/SoundID.cs:1391`) rides
//!   [`Style10Out::play_item8_sound`]; the axis arm queues it on
//!   `Npc::wants_sound` (the S1-E-21g seam, the sim drains it after the
//!   machine's draws).
//! - The `NPC.NewNPC(..., 33)` Water Sphere summon (`:21899`) rides
//!   [`Style10Out::spawn_water_sphere`] (the `wants_sphere` shape); the
//!   axis arm queues it on `Npc::wants_sphere` (the sim's sphere drain
//!   Bottom-places it, `NPC.cs:80770`). The spawn source arguments
//!   (`GetSpawnSourceForProjectileNPC`) are host-only bookkeeping the
//!   port's spawn path does not model.
//! - The Shadowflames bolt (`:21810`) rides `Npc::wants_hostile_shot`
//!   (kind 299, damage 25, kb 0 — the C#'s `NewProjectile` arguments);
//!   the 299 row + its `AI_001` arm landed with W4-c3
//!   (`projectile/data.rs`, `projectile_ai001_a.rs`).

#![allow(dead_code)] // the module's pure helpers stay test-reachable

use crate::npc::Npc;
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// `NPCID.CursedSkull` (`ID/NPCID.cs:10745`).
pub const CURSED_SKULL: i32 = 34;
/// `NPCID.GiantCursedSkull` (`ID/NPCID.cs:11255`).
pub const GIANT_CURSED_SKULL: i32 = 289;
/// `NPCID.WaterBoltMimic` (`ID/NPCID.cs:12065`).
pub const WATER_BOLT_MIMIC: i32 = 694;
/// `ProjectileID.Shadowflames` (`ID/ProjectileID.cs:1152`) — the 289's
/// bolt (`NPC.cs:21808`).
pub const SHADOWFLAMES: u16 = 299;
/// `NPCID.WaterSphere` (`ID/NPCID.cs:10743`) — the 694's summon
/// (`NPC.cs:21899`).
pub const WATER_SPHERE: i32 = 33;

/// The side effects the arm hands back to the caller (the `wants_*`
/// seam shape). Everything the C# writes on `this` lands on the `Npc`
/// directly; this carries the writes the port's `Npc` has no slot for
/// plus the spawn/sound requests.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style10Out {
    /// The arm's `knockBackResist` writes (`:21606` 0, `:21619` 1,
    /// `:21630` 0.2) — `None` when the tick wrote none (the 34/289
    /// paths never do).
    pub knock_back_resist: Option<f32>,
    /// `SoundEngine.PlaySound(SoundID.Item8, base.position)`
    /// (`:21896`) fired this tick.
    pub play_item8_sound: bool,
    /// `NPC.NewNPC(..., (int)center3.X, (int)center3.Y, 33)`
    /// (`:21899`) — the Water Sphere spawn point, int-truncated like
    /// the C# call-site casts.
    pub spawn_water_sphere: Option<(i32, i32)>,
}

/// `NPC.TargetClosest()` (`NPC.cs:77652-77666`, called at `:21585`)
/// through the shared selection (`npc_target.rs`) — the single session
/// player as slot 0, the stock aggro/no-aggro/tank values (the S1-D-3p
/// convention). `ai_style` 10 takes no per-style arm in
/// `SetTargetTrackingValues` (only style 3 does, `:77761`).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = crate::npc_target::TargetCandidate {
        active: true,
        dead: player.dead,
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    // The `EverLastingTicker` stamp's only reader is the unported
    // target-line fade — 0 until the caller migration (S1-D-3p).
    crate::npc_target::target_closest(npc, world, &[cand], true, 10, 0);
}

/// `Vector2.SafeNormalize(v, Vector2.UnitY)` (`Utils.cs:1837-1844`) —
/// the zero/NaN vector falls back to +Y (the dash's `:21728` default).
fn safe_normalize_unit_y(x: f32, y: f32) -> (f32, f32) {
    if (x == 0.0 && y == 0.0) || x.is_nan() || y.is_nan() {
        (0.0, 1.0)
    } else {
        let d = (x * x + y * y).sqrt();
        (x / d, y / d)
    }
}

/// One fixed step of the aiStyle-10 arm (`NPC.cs:21572-21918`).
/// `npc_type` is the vanilla `NPCID` (34 / 289 / 694 — the three
/// `SetDefaults` rows); `just_hit` is `this.justHit` for this tick;
/// `net_mode` is `Main.netMode` (0 single-player, 1 client, 2 server);
/// `rng` is the caller's `Main.rand` stream — drawn from ONLY at
/// `:21851` (see the module note). The step ends with the
/// `noTileCollide` integration (`NPC.cs:92538-92540`, `:92605-92607`);
/// the caller's own collision pass must skip these kinds, as
/// `Npc::integrate_no_tile_collide` does for the ported flyers.
pub fn style10_ai(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    npc_type: i32,
    just_hit: bool,
    net_mode: i32,
    rng: &mut UnifiedRandom,
) -> Style10Out {
    let mut out = Style10Out::default();
    // The `noTileCollide` half of `UpdateCollision` runs even on the
    // arm's early-return ticks — capture + integrate on every exit.
    let result = style10_core(npc, world, player, npc_type, just_hit, net_mode, rng, &mut out);
    // `:90473-90476` — the dead-band X snap (`(double)velocity.X < 0.005 &&
    // > -0.005 → 0`), the `UpdateNPC` step that runs BEFORE the noTileCollide
    // integration (the axis 56/72 arms' precedent). The double compare is
    // the C#'s own (the f32→f64 cast keeps the 0.005 band exact).
    if (npc.vx as f64) < 0.005 && (npc.vx as f64) > -0.005 {
        npc.vx = 0.0;
    }
    // `:92538-92540` — `oldVelocity` capture, the collide flags reset.
    npc.old_vx = npc.vx;
    npc.old_vy = npc.vy;
    npc.collide_x = false;
    npc.collide_y = false;
    // `:92605-92607` — `position += velocity` (no tile collision).
    npc.x += npc.vx;
    npc.y += npc.vy;
    result
}

/// The arm body proper — every `return` of the C# maps to an early
/// return here; the integration tail lives in [`style10_ai`].
#[allow(clippy::too_many_arguments)]
fn style10_core(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    npc_type: i32,
    just_hit: bool,
    net_mode: i32,
    rng: &mut UnifiedRandom,
    out: &mut Style10Out,
) -> Style10Out {
    // `:21574-21577` — flag10/11/13 are all `type == 694`; flag12 its
    // negation.
    let is_mimic = npc_type == WATER_BOLT_MIMIC;
    // `:21578-21582` — num140 the speed, num141 the accel, num142 the
    // 600 cycle, num143 the 650 wrap, num144 the 80 wake-up.
    let mut speed = 1.0f32;
    let mut accel = 0.011f32;
    let cycle = 600.0f32;
    let cycle_wrap = cycle + 50.0;
    let wake = 80.0f32;
    // `:21583-21586` — the skulls retarget every tick; the mimic only
    // while its target is inactive/dead (flag12 || !active || dead).
    if !is_mimic || player.dead {
        target_closest(npc, world, player);
    }
    // `:21587-21591` — the aim vector and its length (num147/num148).
    let (mut cx, mut cy) = npc.center();
    let (pcx, pcy) = player.center();
    let mut dx = pcx - cx;
    let mut dy = pcy - cy;
    let mut dist = (dx * dx + dy * dy).sqrt();
    let raw_dist = dist;
    // `:21592-21595` — the cycle counts except in the closed state.
    if npc.ai[3] != 3.0 {
        npc.ai[1] += 1.0;
    }
    // `:21596-21600` — flag14 the charge, flag15 the dash, flag16 the
    // enraged cycle, flag17 the retreat window, flag18 the mimic's
    // flee (the C#'s `&` is non-short-circuit but side-effect-free).
    let charging = npc.ai[2] >= 0.0 && npc.ai[3] == 2.0;
    let dashing = npc.ai[2] >= 0.0 && npc.ai[3] == 1.0;
    let enraged = npc.ai[1] > cycle;
    let retreating = npc.ai[1] < -30.0;
    let fleeing = is_mimic && !charging && !dashing && !enraged && retreating;
    // `:21601-21613` — state 3: the closed book, parked and immune to
    // knockback; a hit wakes it into state 4.
    if npc.ai[3] == 3.0 {
        npc.sprite_dir = 1;
        npc.rotation = 0.0;
        npc.vx = 0.0;
        npc.vy = 0.0;
        out.knock_back_resist = Some(0.0);
        if just_hit {
            npc.ai[3] = 4.0;
            npc.net_update = true;
        }
        return *out;
    }
    // `:21614-21627` — state 4: the wake-up rest, 80 ticks, then the
    // cycle resets into state 0.
    if npc.ai[3] == 4.0 {
        npc.sprite_dir = 1;
        npc.rotation = 0.0;
        npc.vx = 0.0;
        npc.vy = 0.0;
        out.knock_back_resist = Some(1.0);
        if npc.ai[1] > wake {
            npc.ai[1] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        return *out;
    }
    // `:21628-21632` — the mimic's kb 0.2; the `dontTakeDamage` write
    // tests `ai[3] == 4` which already returned above — dead C#, always
    // false here, carried verbatim.
    if is_mimic {
        out.knock_back_resist = Some(0.2);
        npc.dont_take_damage = npc.ai[3] == 4.0;
    }
    // `:21633-21678` — the pre-steer perturbations (not while charging).
    if !charging {
        if enraged {
            // `:21635-21644` — the 600-tick enrage: 8× accel, 4× speed,
            // the 650 wrap.
            accel *= 8.0;
            speed = 4.0;
            if npc.ai[1] > cycle_wrap {
                npc.ai[1] = 0.0;
            }
            npc.net_update = true;
        } else if is_mimic && raw_dist < 100.0 && npc.ai[1] >= 0.0 {
            // `:21645-21652` — too close: arm the −60 retreat (the
            // netUpdate fires unless already armed at −59 last tick).
            if npc.ai[1] != -59.0 {
                npc.net_update = true;
            }
            npc.ai[1] = -60.0;
        } else if raw_dist < 250.0 {
            // `:21653-21677` — the orbit wobble: ai[0] creeps 0.9/tick,
            // the ±0.019 nudges flip on its sign / ±100 band, the 200
            // wrap re-arms at −200.
            npc.ai[0] += 0.9;
            if npc.ai[0] > 0.0 {
                npc.vy += 0.019;
            } else {
                npc.vy -= 0.019;
            }
            if npc.ai[0] < -100.0 || npc.ai[0] > 100.0 {
                npc.vx += 0.019;
            } else {
                npc.vx -= 0.019;
            }
            if npc.ai[0] > 200.0 {
                npc.ai[0] = -200.0;
                npc.net_update = true;
            }
        }
    }
    // `:21679-21698` — the distance bands (the flee overrides them).
    if fleeing {
        speed = 8.0;
        accel = 0.25;
    } else if raw_dist > 350.0 {
        speed = 5.0;
        accel = 0.3;
    } else if raw_dist > 300.0 {
        speed = 3.0;
        accel = 0.2;
    } else if raw_dist > 250.0 {
        speed = 1.5;
        accel = 0.1;
    }
    // `:21699-21703` — normalize the aim to the speed (num149/num150
    // keep the un-flipped copy for the rotation).
    dist = speed / dist;
    dx *= dist;
    dy *= dist;
    let aim_x = dx;
    let aim_y = dy;
    // `:21704-21708` — the flee flies away from the player.
    if fleeing {
        dx *= -1.0;
        dy *= -1.0;
    }
    // `:21709-21713` — a dead/inactive target: the flee vector along
    // the facing (`direction * num140 / 2`, `-num140 / 2` up).
    if player.dead {
        dx = npc.facing as f32 * speed / 2.0;
        dy = (0.0 - speed) / 2.0;
    }
    if charging {
        // `:21714-21730` — the charge: the first 10 ticks damp ×0.5
        // (the `>= 18` zero inside `< 10` is dead C#, carried verbatim);
        // then the velocity — or the aim when nearly parked — snaps to
        // the 14-speed line.
        let counter = npc.ai[2];
        if counter < 10.0 {
            npc.vx *= 0.5;
            npc.vy *= 0.5;
            if counter >= 18.0 {
                npc.vx = 0.0;
                npc.vy = 0.0;
            }
        } else {
            accel = 14.0;
            let len = (npc.vx * npc.vx + npc.vy * npc.vy).sqrt();
            let (bx, by) = if len < 0.1 {
                (pcx - cx, pcy - cy)
            } else {
                (npc.vx, npc.vy)
            };
            let (nx, ny) = safe_normalize_unit_y(bx, by);
            npc.vx = nx * accel;
            npc.vy = ny * accel;
        }
    } else {
        // `:21731-21749` — the plain approach, X then Y.
        if npc.vx < dx {
            npc.vx += accel;
        } else if npc.vx > dx {
            npc.vx -= accel;
        }
        if npc.vy < dy {
            npc.vy += accel;
        } else if npc.vy > dy {
            npc.vy -= accel;
        }
    }
    // `:21750-21766` — the aim rotation (not while charging): the
    // sprite faces the aim's opposite, the mimic flips once more.
    if !charging {
        if aim_x > 0.0 {
            npc.sprite_dir = -1;
            npc.rotation = aim_y.atan2(aim_x);
        }
        if aim_x < 0.0 {
            npc.sprite_dir = 1;
            npc.rotation = aim_y.atan2(aim_x) + std::f32::consts::PI;
        }
        if is_mimic {
            npc.sprite_dir *= -1;
        }
    }
    // `:21767-21823` — the Giant Cursed Skull's volley machine.
    if npc_type == GIANT_CURSED_SKULL {
        if just_hit {
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
        }
        let (ccx, ccy) = npc.center();
        cx = ccx;
        cy = ccy;
        dx = pcx - cx;
        dy = pcy - cy;
        dist = (dx * dx + dy * dy).sqrt();
        let range = 500.0f32;
        let arm_ticks = 120.0f32;
        let shot_window = 40.0f32;
        let shot_tick = 20.0f32;
        if dist <= range {
            npc.ai[2] += 1.0;
            if npc.ai[3] == 0.0 {
                // `:21785-21793` — 120 ticks in range arm the dash.
                if npc.ai[2] > arm_ticks {
                    npc.ai[2] = 0.0;
                    npc.ai[3] = 1.0;
                    npc.net_update = true;
                }
            } else {
                // `:21794-21812` — the dash lasts 40 ticks; tick 20
                // fires the Shadowflames bolt at the 6-speed aim.
                if npc.ai[2] > shot_window {
                    if npc.ai[3] != 0.0 {
                        npc.net_update = true;
                    }
                    npc.ai[3] = 0.0;
                }
                if net_mode != 1 && npc.ai[2] == shot_tick {
                    let shot_speed = 6.0f32;
                    let shot_damage = 25i32;
                    dist = shot_speed / dist;
                    // `Projectile.NewProjectile(..., center3.X,
                    // center3.Y, num145*num147, num146*num147, 299,
                    // 25, 0f, Main.myPlayer)` (`:21810`) — the
                    // `wants_hostile_shot` seam (kind, x, y, vx, vy,
                    // damage); the knockback 0 is the row's default.
                    npc.wants_hostile_shot.push((
                        SHADOWFLAMES,
                        cx,
                        cy,
                        dx * dist,
                        dy * dist,
                        shot_damage,
                    ));
                }
            }
        } else {
            // `:21814-21822` — out of range: disarm.
            if npc.ai[2] != 0.0 || npc.ai[3] != 0.0 {
                npc.net_update = true;
            }
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
        }
    }
    // `:21824-21827` — only the mimic continues past here.
    if npc_type != WATER_BOLT_MIMIC {
        return *out;
    }
    // `:21828-21832` — a hit disarms the book's cycle.
    if just_hit {
        npc.ai[2] = 0.0;
        npc.ai[3] = 0.0;
    }
    // `:21833-21836` — the book aims from 10 px below its centre.
    let (bcx, bcy) = npc.center();
    cx = bcx;
    cy = bcy + 10.0;
    dx = pcx - cx;
    dy = pcy - cy;
    dist = (dx * dx + dy * dy).sqrt();
    // num145/num146 are recomputed here in the C# but only num147 is
    // read again in the tail — consume the dead recompute.
    let _ = (dx, dy);
    // `:21837-21844` — num159..num166: the 500 range, the 100..300
    // charge band, the 120 arm, the 30 dash, the 60 charge, the 17
    // summon tick, the −300 cooldown.
    let range = 500.0f32;
    let charge_min = 100.0f32;
    let charge_max = 300.0f32;
    let arm_ticks = 120.0f32;
    let dash_ticks = 30.0f32;
    let charge_ticks = 60.0f32;
    let summon_tick = 17.0f32;
    let cooldown = 300.0f32;
    // `:21845-21846` — flag19 the charge band, flag20 the dash range.
    let can_charge = dist >= charge_min
        && dist <= charge_max
        && npc.ai[2] >= 0.0
        && (npc.ai[3] == 0.0 || npc.ai[3] == 2.0);
    let can_dash =
        dist <= range && npc.ai[2] >= 0.0 && (npc.ai[3] == 0.0 || npc.ai[3] == 1.0);
    // `:21847-21850` — the client never runs the cycle.
    if net_mode == 1 {
        return *out;
    }
    // `:21851` — THE arm's only RNG draw: `Main.rand.Next(3)`, reached
    // only when both bands hold (the `||` short-circuits otherwise).
    if can_charge && (!can_dash || rng.next_max(3) == 0) {
        npc.ai[2] += 1.0;
        if npc.ai[3] == 0.0 {
            // `:21854-21862` — 120 ticks arm the charge (state 2).
            if npc.ai[2] > arm_ticks {
                npc.ai[2] = 0.0;
                npc.ai[3] = 2.0;
                npc.net_update = true;
            }
        } else if npc.ai[3] == 2.0 && npc.ai[2] > charge_ticks {
            // `:21863-21868` — the charge ends into the −300 cooldown.
            npc.ai[2] = -cooldown;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
    } else if can_dash {
        npc.ai[2] += 1.0;
        if npc.ai[3] == 0.0 {
            // `:21873-21881` — 120 ticks arm the dash (state 1).
            if npc.ai[2] > arm_ticks {
                npc.ai[2] = 0.0;
                npc.ai[3] = 1.0;
                npc.net_update = true;
            }
        } else {
            // `:21882-21902` — the dash window.
            if npc.ai[3] != 1.0 {
                return *out;
            }
            if npc.ai[2] > dash_ticks {
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
            if npc.ai[2] == summon_tick {
                // `:21896` — the Item8 cast cue; `:21897-21900` — the
                // Water Sphere (33) at the truncated aim point.
                out.play_item8_sound = true;
                if net_mode != 1 {
                    out.spawn_water_sphere = Some((cx as i32, cy as i32));
                }
            }
        }
    } else {
        // `:21904-21916` — neither band: the cooldown creeps back to 0.
        npc.ai[2] += 1.0;
        if npc.ai[2] > 0.0 {
            npc.ai[2] = 0.0;
        }
        if npc.ai[3] != 0.0 {
            npc.net_update = true;
        }
        npc.ai[3] = 0.0;
    }
    *out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A fresh NPC carrying the kind's `SetDefaults` box via
    /// `size_override`. The Zombie kind is only a vehicle; the machine
    /// keys on `npc_type`, never on `npc.kind` (the module's pure shape —
    /// the wired axis arm passes the real kind's `cs_id()` and the kind's
    /// own `size()` row, but the module witnesses keep the override to
    /// stay independent of the kind tables).
    fn skull(npc_type: i32, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some(if npc_type == CURSED_SKULL {
            (26.0, 28.0) // `NPC.cs:9152-9153`
        } else {
            (44.0, 44.0) // `:12755-12756` / `:17739-17740`
        });
        n
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn player_at(w: &World, x: f32, y: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = x;
        p.y = y;
        p
    }

    /// Place the player so its centre sits `dist` px to the RIGHT of
    /// the NPC's centre, on the same row.
    fn player_right(n: &Npc, w: &World, dist: f32) -> Player {
        let (cx, cy) = n.center();
        player_at(w, cx + dist - crate::player::WIDTH * 0.5, cy - crate::player::HEIGHT * 0.5)
    }

    /// `NPC.cs:21578-21582`, `:21633-21644`, `:21679-21698` — the
    /// 600-tick cycle enrages (4-speed / 0.088-accel) and wraps at 650;
    /// the far bands carry 5/0.3 (>350), 3/0.2 (>300), 1.5/0.1 (>250)
    /// and the 1/0.011 default. One approach step from rest reads the
    /// accel directly (`velocity += num141`, `:21733-21748`).
    #[test]
    fn the_cycle_enrages_at_600_and_wraps_at_650() {
        let w = air();
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        // Under 250 px the band table keeps the enrage's 4 / 0.088
        // (the >250 bands would override it, `:21684-21698`); the
        // enrage branch also skips the wobble (the else-if chain).
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(0);
        n.ai[1] = 600.0; // the next tick is the 601st: enraged
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.ai[1], 601.0);
        assert_eq!(n.vx, 0.011f32 * 8.0, "the 8× accel from rest (:21637)");
        // The wrap: ai[1] 650 → tick 651 > 650 → 0.
        n.ai[1] = 650.0;
        n.vx = 0.0;
        n.ai[0] = 0.0;
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.ai[1], 0.0, "the 650 wrap (:21639-21642)");
        assert!(n.net_update);
    }

    /// `NPC.cs:21684-21698` + `:21733-21740` — the distance bands pick
    /// the accel; from rest on the same row the first step is exactly
    /// `num141` toward the player (no wobble at ≥ 250).
    #[test]
    fn the_distance_bands_match_the_csharp_table() {
        let w = air();
        let mut rng = UnifiedRandom::new(0);
        for (dist, accel) in [
            (400.0f32, 0.3f32),  // > 350 → 5 / 0.3
            (320.0, 0.2),       // > 300 → 3 / 0.2
            (260.0, 0.1),       // > 250 → 1.5 / 0.1
        ] {
            let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
            let p = player_right(&n, &w, dist);
            style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
            assert_eq!(n.vx, accel, "the {dist}-px band accelerates {accel}");
            assert_eq!(n.vy, 0.0, "same row: no Y accel");
        }
        // The default band (< 250) wobbles first — the X nudge is
        // −0.019 while ai[0] ∈ [−100, 100] (`:21664-21671`).
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.vx, 0.011 - 0.019, "the default 0.011 minus the wobble");
    }

    /// `NPC.cs:21653-21677` — the orbit wobble: ai[0] creeps 0.9/tick
    /// from 0, so the Y nudge is +0.019 while ai[0] > 0; the 200 wrap
    /// re-arms at −200 with the netUpdate.
    #[test]
    fn the_orbit_wobble_wraps_ai0_at_200() {
        let w = air();
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(0);
        n.ai[0] = 199.5; // +0.9 → 200.4 > 200 → −200
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.ai[0], -200.0, "the 200 wrap (:21672-21676)");
        assert!(n.net_update);
        assert!(n.vy > 0.0, "ai[0] was > 0 at the nudge: +0.019");
    }

    /// `NPC.cs:21709-21713` — a dead target flips the aim to the flee
    /// vector: `direction * speed / 2` sideways, `-speed / 2` up.
    #[test]
    fn a_dead_target_takes_the_flee_vector() {
        let w = air();
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let mut p = player_right(&n, &w, 400.0); // > 350 → speed 5
        p.dead = true;
        let mut rng = UnifiedRandom::new(0);
        // Park the velocity at the flee target so the approach holds it.
        n.vx = 5.0 / 2.0;
        n.vy = -5.0 / 2.0;
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.vx, 2.5, "direction(1) * 5 / 2 (:21711)");
        assert_eq!(n.vy, -2.5, "(0 - 5) / 2 (:21712)");
    }

    /// `NPC.cs:21750-21766` — the rotation: aim right → sprite −1 and
    /// atan2; aim left → sprite 1 and atan2 + π; the mimic flips the
    /// sprite once more (flag13).
    #[test]
    fn the_rotation_follows_the_aim_and_the_mimic_flips() {
        let w = air();
        let mut rng = UnifiedRandom::new(0);
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0);
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.sprite_dir, -1, "aim right (:21752-21756)");
        assert_eq!(n.rotation, 0.0f32.atan2(1.0));
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let (cx, cy) = n.center();
        let p = player_at(&w, cx - 400.0 - 10.0, cy - 21.0); // aim left
        style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.sprite_dir, 1, "aim left (:21757-21761)");
        assert_eq!(n.rotation, 0.0f32.atan2(-1.0) + std::f32::consts::PI);
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0);
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.sprite_dir, 1, "the mimic's flag13 flip (:21762-21765)");
    }

    /// `NPC.cs:21782-21812` — the Giant Cursed Skull: 120 in-range
    /// ticks arm the dash (ai[3] 1); tick 20 of the dash fires the
    /// Shadowflames (299) at the 6-speed aim, damage 25; tick 41 ends
    /// it. Out of range disarms.
    #[test]
    fn the_giant_skull_arms_dashes_and_fires_at_tick_20() {
        let w = air();
        let mut n = skull(GIANT_CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0); // ≤ 500
        let mut rng = UnifiedRandom::new(0);
        n.ai[2] = 120.0; // the next tick is the 121st
        style10_ai(&mut n, &w, &p, GIANT_CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.ai[3], 1.0, "the 120-tick arm (:21787-21792)");
        assert_eq!(n.ai[2], 0.0);
        assert!(n.net_update);
        // The dash: ai[3] 1, ai[2] 0 → each tick ++. Park ai[2] at 19
        // so ONE tick lands on 20 — the shot fires exactly once. (A
        // 20-tick loop would land the shot on its LAST iteration and
        // then clear it; and a free-running dash chases the player at
        // the 5-speed aim, so the tick-20 distance is not the parked
        // one — the bolt's velocity is checked against the pre-step
        // centre instead.)
        n.ai[2] = 19.0;
        let (bcx, bcy) = n.center(); // pre-integration centre (:21774)
        let (pcx, pcy) = p.center();
        n.wants_hostile_shot.clear();
        style10_ai(&mut n, &w, &p, GIANT_CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1, "one bolt at ai[2] == 20 (:21804)");
        let shot = n.wants_hostile_shot.last().unwrap();
        assert_eq!(shot.0, 299, "ProjectileID.Shadowflames");
        assert_eq!(shot.5, 25, "the 25 damage (:21807)");
        assert_eq!((shot.1, shot.2), (bcx, bcy), "the centre spawn point (:21810)");
        // The 6-speed aim: num145/num146 * (6 / num147) (:21809).
        let (dx, dy) = (pcx - bcx, pcy - bcy);
        let d = (dx * dx + dy * dy).sqrt();
        let k = 6.0 / d;
        assert!((shot.3 - dx * k).abs() < 1e-4 && (shot.4 - dy * k).abs() < 1e-4,
            "the 6-speed aim (:21806/21809)");
        // Past 40: the dash ends.
        n.ai[2] = 40.0;
        n.wants_hostile_shot.clear();
        style10_ai(&mut n, &w, &p, GIANT_CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(n.ai[3], 0.0, "the 40-tick window (:21796-21803)");
        // Out of range: disarm.
        let far = player_right(&n, &w, 600.0);
        n.ai[2] = 5.0;
        n.ai[3] = 1.0;
        style10_ai(&mut n, &w, &far, GIANT_CURSED_SKULL, false, 0, &mut rng);
        assert_eq!((n.ai[2], n.ai[3]), (0.0, 0.0), "(:21814-21822)");
    }

    /// `NPC.cs:21769-21773` / `:21828-21832` — a hit resets both
    /// machines' ai[2]/ai[3].
    #[test]
    fn a_hit_disarms_the_cycle() {
        let w = air();
        let mut rng = UnifiedRandom::new(0);
        let mut n = skull(GIANT_CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0);
        n.ai[2] = 50.0;
        n.ai[3] = 1.0;
        style10_ai(&mut n, &w, &p, GIANT_CURSED_SKULL, true, 0, &mut rng);
        // The reset runs before the counter's ++ — 0 → 1, state stays 0.
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.ai[2], 1.0);
    }

    /// `NPC.cs:21714-21730` — the charge (ai[3] 2, ai[2] ≥ 0): the
    /// first 10 ticks damp ×0.5, then the velocity snaps to the
    /// 14-speed line toward the player (SafeNormalize(UnitY) on zero).
    #[test]
    fn the_charge_damps_then_snaps_to_14() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(0);
        n.ai[3] = 2.0;
        n.ai[2] = 0.0;
        n.vx = 4.0;
        n.vy = 2.0;
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        // The ×0.5 damp, then the noTileCollide integration (+2, +1).
        assert_eq!((n.vx, n.vy), (2.0, 1.0), "the ×0.5 damp (:21719)");
        // ai[2] 10: the snap. Parked velocity → the aim direction. The
        // body moved by the damp's integration, so re-aim from the NEW
        // centre — the player stays on the original row, 2 px above it.
        n.ai[2] = 10.0;
        n.vx = 0.0;
        n.vy = 0.0;
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        let len = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((len - 14.0).abs() < 1e-3, "the 14-speed aim line (:21727-21728)");
        assert!(n.vx > 0.0, "toward the player on the right");
        // Zero aim AND zero velocity → SafeNormalize(UnitY) → (0, 14).
        let mut n2 = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let (cx2, cy2) = n2.center();
        let on_top2 = player_at(&w, cx2 - 10.0, cy2 - 21.0);
        n2.ai[3] = 2.0;
        n2.ai[2] = 10.0;
        style10_ai(&mut n2, &w, &on_top2, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n2.vx, 0.0, "the UnitY fallback (:21728)");
        assert_eq!(n2.vy, 14.0);
    }

    /// `NPC.cs:21601-21627` — the closed book (state 3) parks, zeroes
    /// the rotation and the kb; a hit wakes it into state 4, which
    /// rests 80 ticks then re-arms state 0.
    #[test]
    fn the_closed_book_wakes_on_a_hit_and_rests_80_ticks() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(0);
        n.ai[3] = 3.0;
        n.ai[1] = 42.0;
        n.vx = 5.0;
        let out = style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[1], 42.0, "state 3 does not count (:21592-21595)");
        assert_eq!((n.vx, n.vy), (0.0, 0.0));
        assert_eq!(out.knock_back_resist, Some(0.0));
        assert_eq!(n.sprite_dir, 1);
        // The hit wakes it.
        let out = style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, true, 0, &mut rng);
        assert_eq!(n.ai[3], 4.0);
        assert!(n.net_update);
        assert_eq!(out.knock_back_resist, Some(0.0), "the state-3 write ran first");
        // State 4 counts; at ai[1] > 80 it re-arms.
        n.ai[1] = 80.0;
        let out = style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[3], 0.0, "the 80-tick wake (:21620-21625)");
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(out.knock_back_resist, Some(1.0));
    }

    /// `NPC.cs:21645-21652` + `:21596-21600` + `:21679-21683` — the
    /// mimic under 100 px arms the −60 retreat; while ai[1] < −30 (and
    /// no charge/dash/enrage) it flees at the 8-speed / 0.25-accel away
    /// from the player.
    #[test]
    fn the_mimic_retreats_when_too_close() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 80.0); // < 100
        let mut rng = UnifiedRandom::new(0);
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[1], -60.0, "the retreat arm (:21651)");
        assert!(n.net_update);
        // Next tick: ai[1] −59, fleeing — the aim flips, so the X accel
        // is −0.25 (away from the player on the right). The wobble still
        // runs on a flee tick (the else-if chain at :21653 does not test
        // flag18): ai[0] 0 → 0.9 ∈ [−100, 100] → the −0.019 X nudge first.
        n.vx = 0.0;
        n.vy = 0.0;
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[1], -59.0);
        assert_eq!(n.vx, -0.019 - 0.25, "the wobble then the 0.25 flee accel (:21670/:21682)");
    }

    /// `NPC.cs:21845-21916` — the book's cycle: 120 ticks in the dash
    /// range arm state 1; tick 17 of the dash plays Item8 and spawns
    /// the Water Sphere (33) at the truncated centre+10 point; tick 31
    /// ends the dash. The charge band (100..300, both bands holding)
    /// draws the Next(3) and at 120 arms state 2, which ends at 60 into
    /// the −300 cooldown.
    #[test]
    fn the_book_dash_summons_the_water_sphere_at_tick_17() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0); // ≤ 500, > 300: dash only
        let mut rng = UnifiedRandom::new(0);
        n.ai[2] = 120.0;
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[3], 1.0, "the 120-tick dash arm (:21875-21880)");
        assert_eq!(n.ai[2], 0.0);
        let mut summons = 0;
        let mut sounds = 0;
        let mut spawn_point = None;
        for _ in 0..17 {
            // The centre BEFORE the step — the C# reads center3 before
            // the integration moves the body.
            let (cx, cy) = n.center();
            let out = style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
            if out.play_item8_sound {
                sounds += 1;
            }
            if let Some(pt) = out.spawn_water_sphere {
                summons += 1;
                spawn_point = Some((pt, (cx as i32, (cy + 10.0) as i32)));
            }
        }
        assert_eq!(sounds, 1, "the Item8 cue at tick 17 (:21896)");
        assert_eq!(summons, 1, "one Water Sphere (:21899)");
        // The spawn point: the centre + (0, 10), int-truncated
        // (`(int)center3.X, (int)center3.Y`, `:21899` + `:21833`).
        let (pt, expected) = spawn_point.unwrap();
        assert_eq!(pt, expected, "the truncated centre+10 spawn point");
        // Tick 31 ends the dash.
        n.ai[2] = 30.0;
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[3], 0.0, "the 30-tick dash window (:21888-21893)");
        assert_eq!(n.ai[2], 0.0);
    }

    /// `NPC.cs:21851-21868` — the charge path: with both bands holding
    /// (the player at 200 px) the Next(3) draw gates the charge branch;
    /// an accepted tick at ai[2] 120 arms state 2 (`> 120`), and 60
    /// charge ticks end into the −300 cooldown. The cooldown creeps
    /// +1/tick back to 0 (:21904-21910). The oracle stream finds an
    /// accepting draw deterministically — the machine is stepped only
    /// on that tick, so a rejected draw can never arm the dash first.
    #[test]
    fn the_book_charge_ends_into_the_minus_300_cooldown() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0); // 100 ≤ d ≤ 300: both bands
        // The accepting tick: ai[2] 120 → 121 > 120 → state 2. (The
        // centre+10 aim sits at ~200.25 px — inside both bands.)
        n.ai[2] = 120.0;
        let mut rng = UnifiedRandom::new(0);
        // Burn the seed-0 stream's rejected draws so the accepting one
        // lands at the machine's first (and only) draw position.
        let mut oracle = UnifiedRandom::new(0);
        let mut burns = 0;
        while oracle.next_max(3) != 0 {
            rng.next_max(3);
            burns += 1;
            assert!(burns < 1000, "the seed-0 stream draws a 0 within 1000");
        }
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[3], 2.0, "the 120-tick charge arm (:21856-21861)");
        assert_eq!(n.ai[2], 0.0);
        assert!(n.net_update);
        // 60 charge ticks → the −300 cooldown. In state 2 can_dash is
        // false (it needs ai[3] 0/1), so the charge branch runs WITHOUT
        // a draw — re-park so the band keeps holding.
        n.x = 1000.0;
        n.y = 1000.0;
        n.vx = 0.0;
        n.vy = 0.0;
        n.ai[2] = 60.0;
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[2], -300.0, "(:21863-21868)");
        assert_eq!(n.ai[3], 0.0);
        // The cooldown creeps back: −300 → −299 (the else arm — the
        // negative ai[2] fails both bands' `ai[2] >= 0` gate).
        style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(n.ai[2], -299.0, "(:21906-21910)");
    }

    /// RNG witness (`NPC.cs:21851`): the skulls' whole path draws
    /// NOTHING — 600 ticks of the 34 and the 289 (incl. the volley)
    /// leave the stream byte-identical; the mimic's dash-only band
    /// (can_charge false) draws nothing either; ONLY the both-bands
    /// tick draws, exactly one Next(3) per tick.
    #[test]
    fn rng_witness_only_the_both_bands_mimic_tick_draws() {
        let w = air();
        let seed = 12345;
        // The 34: 600 ticks across the enrage wrap — no draws.
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..600 {
            style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        }
        assert_eq!(rng, UnifiedRandom::new(seed), "the 34 path is draw-free");
        // The 289: 600 ticks incl. the arm + the tick-20 volley.
        let mut n = skull(GIANT_CURSED_SKULL, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0);
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..600 {
            style10_ai(&mut n, &w, &p, GIANT_CURSED_SKULL, false, 0, &mut rng);
        }
        assert_eq!(rng, UnifiedRandom::new(seed), "the 289 path is draw-free");
        // The mimic, dash-only band (400 px): no draws. Re-park each
        // tick — without it the approach would carry the body inside
        // the 300-px charge band and the gate would start drawing.
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0);
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..600 {
            n.x = 1000.0;
            n.y = 1000.0;
            n.vx = 0.0;
            n.vy = 0.0;
            style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        }
        assert_eq!(rng, UnifiedRandom::new(seed), "the dash-only band is draw-free");
        // The mimic, both bands (200 px): the draw runs ONLY when
        // flag19 holds AND flag20 does not short-circuit it — i.e.
        // ai[2] >= 0 and ai[3] == 0 (state 1/2 kill flag20, the −300
        // cooldown kills both). The oracle mirrors that gate on the
        // PRE-tick state, exactly where the C# evaluates it. The body
        // is re-parked so the wobble's drift cannot leave the 100..300
        // charge band — the witness pins the DRAW COUNT, not the
        // trajectory.
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(seed);
        let mut oracle = UnifiedRandom::new(seed);
        for _ in 0..300 {
            n.x = 1000.0;
            n.y = 1000.0;
            n.vx = 0.0;
            n.vy = 0.0;
            if n.ai[2] >= 0.0 && n.ai[3] == 0.0 {
                oracle.next_max(3);
            }
            style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        }
        assert_eq!(rng, oracle, "one Next(3) per gated tick (:21851)");
        // The client (netMode 1) returns before the draw (:21847-21850).
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..100 {
            style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 1, &mut rng);
        }
        assert_eq!(rng, UnifiedRandom::new(seed), "the client never draws");
    }

    /// `NPC.cs:21847-21850` — the client runs the movement but never
    /// the cycle: ai[2]/ai[3] stay put and no spawn/sound fires.
    #[test]
    fn the_client_skips_the_book_cycle() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 200.0);
        let mut rng = UnifiedRandom::new(0);
        n.ai[2] = 17.0;
        n.ai[3] = 1.0;
        let out = style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 1, &mut rng);
        assert_eq!(n.ai[2], 17.0, "the cycle is host-only");
        assert_eq!(n.ai[3], 1.0);
        assert!(!out.play_item8_sound);
        assert_eq!(out.spawn_water_sphere, None);
    }

    /// `NPC.cs:21628-21632` — the mimic's kb 0.2 write and the dead
    /// `dontTakeDamage = ai[3] == 4` (always false past the state-4
    /// early return); the skulls write no kb at all.
    #[test]
    fn the_mimic_kb_write_and_the_dead_dont_take_damage() {
        let w = air();
        let mut n = skull(WATER_BOLT_MIMIC, 1000.0, 1000.0);
        let p = player_right(&n, &w, 400.0);
        let mut rng = UnifiedRandom::new(0);
        n.dont_take_damage = true;
        let out = style10_ai(&mut n, &w, &p, WATER_BOLT_MIMIC, false, 0, &mut rng);
        assert_eq!(out.knock_back_resist, Some(0.2));
        assert!(!n.dont_take_damage, "the dead C# write clears it (:21631)");
        let mut n = skull(CURSED_SKULL, 1000.0, 1000.0);
        let out = style10_ai(&mut n, &w, &p, CURSED_SKULL, false, 0, &mut rng);
        assert_eq!(out.knock_back_resist, None, "the skulls never write kb");
    }
}
