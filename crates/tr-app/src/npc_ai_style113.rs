//! The aiStyle-113 family — the Windy Balloon
//! (`NPC.AI_113_WindyBalloon`, `NPC.cs:47888-48065`, dispatched at
//! `:41072-41075`). W4-d33, research/25 row 113, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the WindyBalloon 594
//! (`NPCID.WindyBalloon`, `ID/NPCID.cs:11865`; `SetDefaults` row
//! `NPC.cs:16707-16719`). The census of `this.aiStyle = 113;` returns
//! exactly that one hit (`:16711`) — no projectile-space user. The
//! port's roster has no balloon kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The balloon carries a SLAVE slime (type 1, `ai[0] = -999`) on a
//! string; the pair drifts with the wind and pops on contact.
//!
//! The first tick (`:47890-47913`, non-client, `localAI[0] == 0`):
//! TargetClosest; `ai[2] = Next(7) + 1` (the colour 1-7); the slave
//! spawn request (type 1 at the truncated position); the variant
//! chain — `Next(180) == 0` → SetDefaults(-4), ELSE `Next(10) == 0` →
//! -7, ELSE `Next(3) == 0` → -3 (the SHORT-CIRCUIT chain: a passing
//! roll skips the later draws); the slave's `ai[0] = -999` +
//! netUpdate; `ai[3] = slot`; `ai[1] = slave.scale`.
//!
//! The size round trip (`:47914-47917`): `position = Center; Size =
//! (20,20) * ai[1]; scale = ai[1]; Center = position` — the box
//! re-centres on the OLD centre at the slave's scale.
//!
//! The slave gate ([`balloon_slave_check`],
//! `AI_113_WindyBalloon_GetSlaveNPC` `:48067-48086`): the `ai[3]` slot
//! must hold an active type-1 with `ai[0] == -999`; a stale slot
//! resets `ai[3] = -1` + netUpdate.
//!
//! With the slave (`:47920-48046`): `rotation = vx * 0.05`; the
//! target's Bottom-relative vector — the above-target tile count
//! `(int)dy / -16` (zeroed when the facing disagrees with the sign);
//! the pop gate (`wet || slave.wet || collideX || (collideY &&
//! oldVy < 0)`): the `oldVx + direction*8` X-rewind on BOTH bodies,
//! the no-face retarget, the facing flip + `vx = direction*2`,
//! `life = -1`, HitEffect (the 594 gore arm below), `active = false`,
//! netUpdate, the DeathSound — and RETURN. The collideY snap
//! (`oldVy > 0 ? 1 : -1`) + no-face retarget. The wind cruise
//! (`2 + |windSpeedTarget|*2` cap, the 0.01 accel, the against-sign
//! 0.1/0.05 double-accel, the clamp). The `8 + num3`-deep ground scan
//! (the `< row + 5 + num3` near flag), the 0.05/-0.1/-0.2 ladder caps
//! 2/-4 — OR the within-400 sighted hover (the INTEGER quarter-height
//! compare, the ±0.035 accel with the sign-crossing double). The
//! slave rides `Bottom + (0,-8) + (0, 56*ai[1])` at the balloon's
//! velocity.
//!
//! Without the slave (`:48047-48064`): the free drift — `vy - 0.2`
//! clamped ±8, `vx + windSpeedTarget*0.3` clamped ±4; the collide pop
//! (the same rewind/flip/death, no RETURN — the arm ends).
//!
//! ## The HitEffect 594 arm
//!
//! `NPC.cs:81789-81812` (the `life <= 0` gate — both pop paths set
//! `life = -1` first): THREE gores of type `1125 + (ai[2]-1)*3 +
//! Next(3)` at `Bottom + (0,-50).RotatedBy(rotation) +
//! NextVector2Circular(15,15)`, velocity `NextVector2Circular(2,2)`
//! then `vy -= NextFloat()` then `×4`; ONE gore 1146 at the same
//! anchor with the (5,5) circle; the slave release (`ai[0] = 0`,
//! `position.Y -= 10`, netUpdate). Per gore the ARGUMENT draws are
//! arm-level (the style39 convention): Circular(15,15) = NextFloat
//! (angle) + NextFloat (radius), Circular(2,2) = 2, the type Next(3),
//! the vy NextFloat — 5 draws, then the NewGore internals (pool-side).
//!
//! ## The collision box
//!
//! `GetTileCollisionParameters` (`NPC.cs:92812-92820`): the balloon's
//! box extends DOWN over the string — `num = (int)(44 + 20*ai[1])`;
//! `Y += num; height += num`; the width re-centres to
//! `(int)(6 + 26*ai[1])`. [`balloon_collision_box`] ports it.
//!
//! ## RNG order (by construction)
//!
//! First tick: Next(7), then the variant chain's short-circuit
//! Next(180)/Next(10)/Next(3). The pop: per gore 5 argument draws +
//! the pool internals. Everything else (the cruise, the scans, the
//! hover, the drift) draws NOTHING; TargetClosest is RNG-free.
//!
//! ## FindFrame case 594
//!
//! `NPC.cs:67993-67997`: `spriteDirection = 1`; `frame.Y = (int)ai[2]`
//! — the colour pick is a raw byte offset into the 32-wide sheet
//! (`frame.Width = 32`), NOT a row index; no counter. The wiring
//! caller applies it directly (documented, no pair function).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_ai_style37::rotated_by;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// The type-594 `SetDefaults` row (`NPC.cs:16707-16719`), verbatim.
/// The row writes NO HitSound (null — the balloon only pops).
pub struct WindyBalloonDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCDeath63` (`SoundID.cs:1377`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub value: f32,
    pub no_gravity: bool,
}

/// `NPC.cs:16707-16719` — `else if (this.type == 594)`.
pub const WINDY_BALLOON: WindyBalloonDefaults = WindyBalloonDefaults {
    width: 20,
    height: 20,
    ai_style: 113,
    damage: 0,
    defense: 0,
    life_max: 1,
    death_sound: (4, 63),
    knock_back_resist: 0.3,
    value: 0.0,
    no_gravity: true,
};

/// The slave slot's view — `Main.npc[(int)ai[3]]` as
/// `AI_113_WindyBalloon_GetSlaveNPC` reads it (`:48067-48086`). The
/// wiring caller fills it from the pool; the pure module never
/// indexes the pool itself.
#[derive(Clone, Copy, Debug)]
pub struct BalloonSlave {
    pub active: bool,
    /// `Main.npc[slot].type` — only 1 (the Blue Slime) rides.
    pub npc_type: i32,
    /// `Main.npc[slot].ai[0]` — the -999 leash marker.
    pub ai0: f32,
    /// `Main.npc[slot].scale` — the first tick's `ai[1]` seed.
    pub scale: f32,
    /// `Main.npc[slot].wet` — the pop gate reads it (`:47939`).
    pub wet: bool,
}

/// The per-tick environment (the `Style111Env` shape).
pub struct Style113Env<'a> {
    pub world: &'a World,
    pub gore: &'a mut GorePool,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.windSpeedTarget` — the cruise cap + the free drift.
    pub wind_speed_target: f32,
    /// `Main.netMode`.
    pub net_mode: i32,
    /// The `ai[3]` slot's view; `None` when the index is out of range
    /// (the C# `num >= 0 && num < maxNPCs` gate).
    pub slave: Option<BalloonSlave>,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// The first tick's slave spawn request (`:47895-47912`): the wiring
/// caller runs `NPC.NewNPC(source, (int)x, (int)y, 1)`, applies the
/// variant `SetDefaults(-4/-7/-3)` when set, writes `ai[0] = -999` +
/// netUpdate, and feeds the slot index back through `ai[3]` and the
/// scale through `ai[1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BalloonSpawn {
    /// The vanilla type — always 1 (the Blue Slime).
    pub npc_type: i32,
    /// The truncated position (the `NewNPC` Bottom placement).
    pub x: i32,
    pub y: i32,
    /// The variant `SetDefaults` net id: -4 / -7 / -3, else 0 (none).
    pub variant: i32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style113Outcome {
    /// The first tick's slave spawn request.
    pub spawn: Option<BalloonSpawn>,
    /// The pop (`:47947-47951` / `:48058-48062`): `life = -1` +
    /// HitEffect (the gore burst ran INSIDE the step) + `active =
    /// false`. The caller owes the pool removal.
    pub died: bool,
    /// The pop's `SoundEngine.PlaySound(this.DeathSound, position)`
    /// (`:47951` / `:48062`) — `(type, style, x, y)` = NPCDeath63
    /// `(4, 63)` at the post-rewind position; the audio family owns
    /// the pitch draws.
    pub sounds: Vec<(i32, i32, f32, f32)>,
    /// The slave's per-tick ride (`:48044-48045`): the new centre +
    /// the balloon's velocity — the wiring writes the pool slot.
    pub slave_ride: Option<(Vec2, Vec2)>,
    /// The HitEffect slave release (`:48105-48111` → `:81805-81811`):
    /// `ai[0] = 0`, `position.Y -= 10`, netUpdate.
    pub slave_release: bool,
    /// The X-rewind amount (`oldVx + direction*8`) applied to BOTH
    /// bodies on the pop — the balloon's position already moved; the
    /// wiring applies it to the slave.
    pub pop_rewind: Option<f32>,
}

/// XNA `Vector2.Length()` — the f32 sum widened to f64 for the root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
}

/// `Math.Sign(float)` — -1/0/+1.
fn math_sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE NextFloat;
/// the float angle then the double cos/sin (the empress port).
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// THEN a second NextFloat radius scale.
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// `AI_113_WindyBalloon_GetSlaveNPC` (`NPC.cs:48067-48086`): the
/// `ai[3]` slot must hold an active type-1 with `ai[0] == -999`; a
/// stale IN-RANGE slot resets `ai[3] = -1` + netUpdate (an
/// out-of-range index leaves `ai[3]` untouched — verbatim).
fn balloon_slave_check(npc: &mut Npc, env: &Style113Env<'_>) -> Option<BalloonSlave> {
    let slot = npc.ai[3] as i32;
    // The C# `num >= 0 && num < Main.maxNPCs` (200); the env's `None`
    // stands for the out-of-range index.
    match env.slave {
        Some(s) => {
            if !s.active || s.npc_type != 1 || s.ai0 != -999.0 {
                npc.ai[3] = -1.0;
                npc.net_update = true;
                None
            } else {
                Some(s)
            }
        }
        None => {
            let _ = slot;
            None
        }
    }
}

/// The HitEffect type-594 arm (`NPC.cs:81789-81812`) — the `life <= 0`
/// gore burst. The caller sets `life = -1` (the port's `hp`) BEFORE
/// calling, exactly like the two pop paths.
fn balloon_hit_effect(npc: &Npc, env: &mut Style113Env<'_>, out: &mut Style113Outcome) {
    if npc.hp > 0 {
        return; // the `life <= 0` gate (`:81789`)
    }
    let (w, h) = npc.size();
    let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
    // `:81791-81792` — the colour gore base + the rotated anchor.
    let gore_base = 1125 + (npc.ai[2] as i32 - 1) * 3;
    let (rx, ry) = rotated_by(0.0, -50.0, npc.rotation as f64, 0.0, 0.0);
    let anchor = bottom + Vec2::new(rx, ry);
    // `:81793-81798` — the three colour gores.
    for _ in 0..3 {
        let pos = anchor + next_vector2_circular(env.rng, 15.0, 15.0);
        let vel = next_vector2_circular(env.rng, 2.0, 2.0);
        let ty = gore_base + env.rng.next_max(3);
        let idx = env
            .gore
            .new_gore_perfect(env.rng, env.world, pos, vel, ty, 1.0);
        let g = env.gore.slot_mut(idx);
        g.velocity.y -= env.rng.next_float();
        g.velocity *= 4.0;
    }
    // `:81799-81804` — the single 1146 gore (the (5,5) circle).
    let pos = anchor + next_vector2_circular(env.rng, 5.0, 5.0);
    let vel = next_vector2_circular(env.rng, 2.0, 2.0);
    let idx = env
        .gore
        .new_gore_perfect(env.rng, env.world, pos, vel, 1146, 1.0);
    let g = env.gore.slot_mut(idx);
    g.velocity.y -= env.rng.next_float();
    g.velocity *= 4.0;
    // `:81805-81811` — the slave release (the caller's seam).
    if balloon_slave_view(npc, env).is_some() {
        out.slave_release = true;
    }
}

/// The HitEffect's own slave re-read (`:81805` calls
/// `AI_113_WindyBalloon_GetSlaveNPC` again) — the same gate, but the
/// pure module does not re-write `ai[3]` here (the pop path already
/// resolved it; the C# re-check is idempotent on the same tick).
fn balloon_slave_view(npc: &Npc, env: &Style113Env<'_>) -> Option<BalloonSlave> {
    match env.slave {
        Some(s) if s.active && s.npc_type == 1 && s.ai0 == -999.0 => Some(s),
        _ => {
            let _ = npc;
            None
        }
    }
}

/// The shared pop sequence (`:47941-47951` / `:48053-48062`): the
/// X-rewind, the no-face retarget, the facing flip + vx re-arm,
/// `life = -1`, HitEffect, the death flag + the DeathSound cue.
fn pop(npc: &mut Npc, env: &mut Style113Env<'_>, out: &mut Style113Outcome) {
    let rewind = npc.old_vx + npc.facing as f32 * 8.0;
    npc.x -= rewind;
    out.pop_rewind = Some(rewind);
    if !env.players.is_empty() {
        target_closest(npc, env.world, env.players, false, 113, env.ticker);
    }
    npc.facing *= -1;
    npc.vx = npc.facing as f32 * 2.0;
    npc.hp = -1; // `this.life = -1`
    balloon_hit_effect(npc, env, out);
    out.died = true; // `active = false`
    npc.net_update = true;
    // `SoundEngine.PlaySound(this.DeathSound, position)` — NPCDeath63.
    out.sounds.push((4, 63, npc.x, npc.y));
}

/// `NPC.GetTileCollisionParameters`'s 594 arm
/// (`NPC.cs:92812-92820`): the string-extended collision box.
pub fn balloon_collision_box(x: f32, y: f32, w: i32, h: i32, ai1: f32) -> (f32, f32, i32, i32) {
    let num = (44.0 + 20.0 * ai1) as i32;
    let cy = y + num as f32;
    let ch = h + num;
    let cw = (6.0 + 26.0 * ai1) as i32;
    // `cPosition.X += cWidth / 2` reads the OLD width (the C# order).
    let cx = x + (w / 2) as f32 - (cw / 2) as f32;
    (cx, cy, cw, ch)
}

/// One fixed step of the aiStyle-113 arm (`NPC.cs:47888-48065`), in C#
/// order. The caller owns the movement integration (through
/// [`balloon_collision_box`]) and drains [`Style113Outcome`].
pub fn style113_step(npc: &mut Npc, env: &mut Style113Env<'_>) -> Style113Outcome {
    let mut out = Style113Outcome::default();
    // `:47890-47913` — the first tick.
    if npc.local_ai[0] == 0.0 && env.net_mode != 1 {
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 113, env.ticker);
        }
        npc.local_ai[0] = 1.0;
        npc.ai[2] = (env.rng.next_max(7) + 1) as f32;
        // The slave spawn request — the wiring resolves the slot.
        let mut variant = 0;
        if env.rng.next_max(180) == 0 {
            variant = -4;
        } else if env.rng.next_max(10) == 0 {
            variant = -7;
        } else if env.rng.next_max(3) == 0 {
            variant = -3;
        }
        out.spawn = Some(BalloonSpawn {
            npc_type: 1,
            x: npc.x as i32,
            y: npc.y as i32,
            variant,
        });
        npc.net_update = true;
        // `ai[3] = num` and `ai[1] = slave.scale` ride the wiring's
        // spawn resolution (the outcome's spawn request); the env's
        // slave view carries the scale for the NEXT tick.
    }
    // `:47914-47917` — the size round trip.
    let (w0, h0) = npc.size();
    let old_center = Vec2::new(npc.x + w0 / 2.0, npc.y + h0 / 2.0);
    let new_w = 20.0 * npc.ai[1];
    let new_h = 20.0 * npc.ai[1];
    npc.size_override = Some((new_w, new_h));
    npc.scale = npc.ai[1];
    npc.x = old_center.x - new_w / 2.0;
    npc.y = old_center.y - new_h / 2.0;
    let (w, h) = npc.size();
    let slave = balloon_slave_check(npc, env);
    npc.rotation = npc.vx * 0.05;
    if let Some(s) = slave {
        // `:47922-47938` — the target's Bottom-relative vector.
        let mut dist = f32::INFINITY; // num2
        let mut extra_depth = 0i32; // num3
        if npc.target >= 0 && (npc.target as usize) < env.players.len() {
            let p = &env.players[npc.target as usize];
            if p.active && !p.dead && !p.ghost {
                let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
                let to_target =
                    Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32) - bottom;
                if to_target.y < 0.0 {
                    extra_depth = to_target.y as i32 / -16;
                }
                dist = length(to_target);
                if npc.facing != math_sign(to_target.x) {
                    extra_depth = 0;
                }
            }
        }
        // `:47939-47953` — the pop gate.
        if npc.wet || s.wet || npc.collide_x || (npc.collide_y && npc.old_vy < 0.0) {
            pop(npc, env, &mut out);
            return out;
        }
        // `:47954-47958` — the collideY snap.
        if npc.collide_y {
            npc.vy = if npc.old_vy > 0.0 { 1.0 } else { -1.0 };
            if !env.players.is_empty() {
                target_closest(npc, env.world, env.players, false, 113, env.ticker);
            }
        }
        // `:47959-47978` — the wind cruise.
        let cap = 2.0 + env.wind_speed_target.abs() * 2.0; // num5
        if math_sign(npc.vx) != npc.facing || npc.vx.abs() < cap {
            npc.vx += npc.facing as f32 * 0.01;
            if npc.vx * (npc.facing as f32) < 0.0 {
                if npc.vx.abs() > cap {
                    npc.vx += npc.facing as f32 * 0.1;
                } else {
                    npc.vx += npc.facing as f32 * 0.05;
                }
            } else if npc.vx.abs() > cap {
                npc.vx = npc.facing as f32 * cap;
            }
        }
        // `:47979-47999` — the ground scan (8 + num3 deep).
        let col = ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32 + npc.facing;
        let row = ((npc.y + h) / 16.0) as i32;
        let mut no_ground = true; // flag
        let depth = 8 + extra_depth; // num8
        let mut ground_near = false; // flag2
        for i in row..row + depth {
            let blocked = env.world.tile(col, i).is_some_and(|t| {
                (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
            });
            if blocked {
                if i < row + 5 + extra_depth {
                    ground_near = true;
                }
                no_ground = false;
                break;
            }
        }
        // `:48000-48043` — the hover OR the gravity ladder.
        let p = if npc.target >= 0 && (npc.target as usize) < env.players.len() {
            Some(&env.players[npc.target as usize])
        } else {
            env.players.first()
        };
        let sighted = p.is_some_and(|p| {
            crate::projectile::can_hit(
                env.world,
                npc.x,
                npc.y,
                w,
                h,
                p.x,
                p.y,
                p.width as f32,
                p.height as f32,
            )
        });
        if dist < 400.0 && sighted {
            let p = p.unwrap();
            let cap_y = 2.0f32; // num9
            let accel = 0.035f32; // num10
            let my_quarter = npc.y + h / 2.0 + (h as i32 / 4) as f32;
            let their_quarter = p.y + (p.height / 4) as f32;
            if my_quarter > their_quarter && npc.vy > 0.0 - cap_y {
                npc.vy -= accel;
                if npc.vy > 0.0 {
                    npc.vy -= accel;
                }
            } else if my_quarter < their_quarter && npc.vy < cap_y {
                npc.vy += accel;
                if npc.vy < 0.0 {
                    npc.vy += accel;
                }
            }
        } else {
            if no_ground {
                npc.vy += 0.05;
            } else {
                npc.vy -= 0.1;
            }
            if ground_near {
                npc.vy -= 0.2;
            }
            if npc.vy > 2.0 {
                npc.vy = 2.0;
            }
            if npc.vy < -4.0 {
                npc.vy = -4.0;
            }
        }
        // `:48044-48045` — the slave ride.
        let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
        let ride = bottom + Vec2::new(0.0, -8.0) + Vec2::new(0.0, 56.0 * npc.ai[1]);
        out.slave_ride = Some((ride, Vec2::new(npc.vx, npc.vy)));
    } else {
        // `:48047-48064` — the free drift.
        npc.vy = (npc.vy - 0.2).clamp(-8.0, 8.0);
        npc.vx = (npc.vx + env.wind_speed_target * 0.3).clamp(-4.0, 4.0);
        if npc.collide_x || npc.collide_y {
            pop(npc, env, &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 594-shaped NPC (the style73 placeholder convention).
    fn balloon(x: f32, y: f32) -> Npc {
        let d = &WINDY_BALLOON;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.ai[1] = 1.0; // the slave scale (the wiring seeds it)
        n.ai[3] = 5.0; // the slave slot
        n.local_ai[0] = 1.0; // skip the first tick by default
        n
    }

    fn slave() -> BalloonSlave {
        BalloonSlave {
            active: true,
            npc_type: 1,
            ai0: -999.0,
            scale: 1.0,
            wet: false,
        }
    }

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

    /// `NPC.cs:16707-16719` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s113_the_defaults_row() {
        let d = &WINDY_BALLOON;
        assert_eq!((d.width, d.height), (20, 20));
        assert_eq!(d.ai_style, 113);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 1));
        assert_eq!(d.death_sound, (4, 63));
        assert_eq!(d.knock_back_resist, 0.3);
        assert_eq!(d.value, 0.0);
        assert!(d.no_gravity);
    }

    /// `:47890-47913` — the first tick: Next(7)+1 colour, the
    /// short-circuit variant chain (a passing Next(180) skips the
    /// Next(10)/Next(3)), the spawn request at the truncated position.
    #[test]
    fn w4_d33_s113_the_first_tick_spawn() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut oracle = UnifiedRandom::new(113);
        let mut n = balloon(100.5, 100.5);
        n.local_ai[0] = 0.0;
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            slave: Some(slave()),
            ticker: 7,
        };
        let out = style113_step(&mut n, &mut env);
        let colour = oracle.next_max(7) + 1;
        let mut variant = 0;
        if oracle.next_max(180) == 0 {
            variant = -4;
        } else if oracle.next_max(10) == 0 {
            variant = -7;
        } else if oracle.next_max(3) == 0 {
            variant = -3;
        }
        assert_eq!(rng.peek(), oracle.peek(), "the variant chain order");
        assert_eq!(n.ai[2], colour as f32);
        let spawn = out.spawn.expect("the slave request");
        assert_eq!(spawn.npc_type, 1);
        assert_eq!((spawn.x, spawn.y), (100, 100), "the truncated position");
        assert_eq!(spawn.variant, variant);
        assert_eq!(n.local_ai[0], 1.0);
        assert!(n.net_update);
    }

    /// `:47914-47917` — the size round trip: the box re-centres on the
    /// OLD centre at 20×ai[1]; scale = ai[1].
    #[test]
    fn w4_d33_s113_the_size_round_trip() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut n = balloon(100.0, 100.0);
        n.ai[1] = 0.5;
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            slave: Some(slave()),
            ticker: 7,
        };
        let _ = style113_step(&mut n, &mut env);
        assert_eq!(n.size_override, Some((10.0, 10.0)));
        assert_eq!(n.scale, 0.5);
        // The old centre (110, 110) — the new top-left (105, 105).
        assert_eq!((n.x, n.y), (105.0, 105.0));
    }

    /// `:48067-48086` — the stale slave resets ai[3] = -1 + netUpdate;
    /// the free drift then runs (vy - 0.2, the wind vx).
    #[test]
    fn w4_d33_s113_the_stale_slave_and_the_free_drift() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut n = balloon(100.0, 100.0);
        n.net_update = false;
        let mut s = slave();
        s.ai0 = 0.0; // the released slime — no longer leashed
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 1.0,
            net_mode: 0,
            slave: Some(s),
            ticker: 7,
        };
        let out = style113_step(&mut n, &mut env);
        assert_eq!(n.ai[3], -1.0);
        assert!(n.net_update);
        assert!(out.slave_ride.is_none());
        assert_eq!(n.vy, -0.2, "the free lift");
        assert_eq!(n.vx, 0.3, "the wind drift");
    }

    /// `:47939-47953` — the pop: the rewind, the facing flip, life -1,
    /// the gore burst (4 gores × 5 argument draws), the slave release,
    /// the death flag. The wet slave pops it.
    #[test]
    fn w4_d33_s113_the_pop_and_the_gore_burst() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut oracle = UnifiedRandom::new(113);
        let mut n = balloon(100.0, 100.0);
        n.ai[2] = 3.0; // the colour — gore base 1125 + 2*3 = 1131
        n.old_vx = 1.0;
        let mut s = slave();
        s.wet = true;
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            slave: Some(s),
            ticker: 7,
        };
        let out = style113_step(&mut n, &mut env);
        assert!(out.died);
        assert_eq!(n.hp, -1);
        assert_eq!(n.facing, -1);
        assert_eq!(n.vx, -2.0);
        assert_eq!(out.pop_rewind, Some(9.0), "oldVx 1 + direction 1 * 8");
        assert!(out.slave_release);
        // The 4 gores: each 2+2 circular draws + the type (3 only) +
        // the vy NextFloat. The 1146 gore has NO type draw.
        for g in 0..4 {
            oracle.next_float(); // circular angle
            oracle.next_float(); // circular radius
            oracle.next_float();
            oracle.next_float();
            if g < 3 {
                oracle.next_max(3);
            }
            oracle.next_float(); // the vy nudge
        }
        assert_eq!(rng.peek(), oracle.peek(), "the gore argument order");
    }

    /// `:47959-47978` — the wind cruise: the 2 + |wind|*2 cap, the
    /// 0.01 accel, the against-sign 0.05 double-accel.
    #[test]
    fn w4_d33_s113_the_wind_cruise() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.5,
            net_mode: 0,
            slave: Some(slave()),
            ticker: 7,
        };
        let mut n = balloon(100.0, 100.0);
        let _ = style113_step(&mut n, &mut env);
        assert!((n.vx - 0.01).abs() < 1e-6, "the 0.01 accel");
        let mut n = balloon(100.0, 100.0);
        n.vx = -1.0; // against the +1 facing, under the 3 cap
        let _ = style113_step(&mut n, &mut env);
        assert!(
            (n.vx - (-0.94)).abs() < 1e-6,
            "the 0.01 + 0.05 double, vx {}",
            n.vx
        );
    }

    /// `:48044-48045` — the slave ride: Bottom + (0,-8) + (0,56*ai[1]).
    #[test]
    fn w4_d33_s113_the_slave_ride() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut n = balloon(100.0, 100.0);
        n.ai[1] = 1.0;
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            slave: Some(slave()),
            ticker: 7,
        };
        let out = style113_step(&mut n, &mut env);
        let (ride, vel) = out.slave_ride.expect("the ride");
        // Bottom (110, 120) + (0, 48) = (110, 168).
        assert_eq!(ride, Vec2::new(110.0, 168.0));
        assert_eq!(vel, Vec2::new(n.vx, n.vy));
    }

    /// `NPC.cs:92812-92820` — the string-extended collision box.
    #[test]
    fn w4_d33_s113_the_collision_box() {
        // ai1 = 1: num = 64, width = 32.
        let (x, y, w, h) = balloon_collision_box(100.0, 100.0, 20, 20, 1.0);
        assert_eq!((y, h), (164.0, 84));
        assert_eq!(w, 32);
        assert_eq!(x, 100.0 + 10.0 - 16.0);
        // ai1 = 0.5: num = 54, width = 19.
        let (x, y, w, h) = balloon_collision_box(100.0, 100.0, 20, 20, 0.5);
        assert_eq!((y, h), (154.0, 74));
        assert_eq!(w, 19);
        assert_eq!(x, 100.0 + 10.0 - 9.0);
    }

    /// `:48051-48063` — the free-drift collide pop (no slave): the
    /// same death sequence, no return-after (the arm ends).
    #[test]
    fn w4_d33_s113_the_free_drift_pop() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(113);
        let mut n = balloon(100.0, 100.0);
        n.ai[3] = -1.0; // no slave
        n.collide_x = true;
        let mut env = Style113Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            slave: None,
            ticker: 7,
        };
        let out = style113_step(&mut n, &mut env);
        assert!(out.died);
        assert_eq!(n.hp, -1);
        assert!(!out.slave_release, "no slave to release");
    }
}
