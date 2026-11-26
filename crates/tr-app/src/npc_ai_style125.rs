//! The aiStyle-125 family — the Clumsy Slime Balloon
//! (`NPC.AI_125_ClumsySlimeBalloon`, `NPC.cs:43688-43819`, dispatched
//! at `:41120-41123`). W4-d33, research/25 row 125, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the BoundTownSlimePurple
//! 686 (`NPCID.BoundTownSlimePurple`, `ID/NPCID.cs:12049`; `SetDefaults`
//! row `NPC.cs:17624-17639`) — the balloon carrying the trapped town
//! slime. The census of `this.aiStyle = 125;` returns exactly that one
//! hit (`:17628`) — no projectile-space user. The port's roster has no
//! clumsy-balloon kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The wind-drifting balloon (the aiStyle-113 sibling WITHOUT the
//! slave): it pops on contact and frees the town slime.
//!
//! The first tick (`:43690-43695`, non-client): TargetClosest +
//! `localAI[0] = 1` + netUpdate. `spriteDirection = direction`,
//! `rotation = vx * 0.05` (`:43696-43697`).
//!
//! The target vector (`:43698-43714`): the Bottom-relative aim — the
//! above-target tile count `(int)dy / -16` (zeroed when the facing
//! disagrees with the sign), the distance.
//!
//! The pop gate (`:43715-43728`): `wet || collideX || (collideY &&
//! oldVy < 0)` — the `oldVx + direction*8` X-rewind, the no-face
//! retarget, the facing flip + `vx = direction*2`, `life = -1`,
//! HitEffect (the gore + transform arm below), `active = false`,
//! netUpdate, the DeathSound — RETURN.
//!
//! The collideY snap (`:43729-43733`): `oldVy > 0 ? 1 : -1` + the
//! no-face retarget.
//!
//! The wind cruise (`:43734-43753`): the `3 + |windSpeedTarget|*2`
//! cap, the 0.04 accel, the against-sign 0.15/0.1 double-accel, the
//! clamp.
//!
//! The ground scan (`:43754-43774`): the `8 + num2`-deep
//! direction-shifted column (solid OR liquid), the `< row + 5 + num2`
//! near flag.
//!
//! The hover OR the ladder (`:43775-43818`): within 400 px AND sighted
//! — the INTEGER quarter-height compare, the ±0.035 accel with the
//! sign-crossing double; else the 0.05/-0.1/-0.2 ladder caps 2/-4.
//!
//! ## The HitEffect 686 arm
//!
//! `NPC.cs:81752-81781` (the `life <= 0` gate): THREE gores of type
//! `1143 + Next(3)` at `Bottom + (0,-50).RotatedBy(rotation) +
//! NextVector2Circular(15,15)`, velocity `NextVector2Circular(2,2)`
//! then `vy -= NextFloat()` then `×4`; ONE gore 1146 with the (5,5)
//! circle. The non-client release (`:81768-81780`): `position =
//! Bottom + (0, 48)`, `Transform(680)` (the TownSlimePurple — the
//! outcome seam), the `unlockedSlimePurpleSpawn` latch (the wiring's
//! world-state flag + the netMode-2 SendData 7).
//!
//! ## The collision box
//!
//! `GetTileCollisionParameters` (`NPC.cs:92821-92829`): the balloon's
//! box extends DOWN 64 px and re-centres to width 32.
//! [`balloon_collision_box`] ports it.
//!
//! ## RNG order (by construction)
//!
//! The pop: per gore 5 argument draws (Circular(15,15)/(5,5) = 2
//! NextFloat, Circular(2,2) = 2, the type Next(3) for the three colour
//! gores, the vy NextFloat) then the pool internals. Everything else
//! draws NOTHING; TargetClosest is RNG-free.
//!
//! ## FindFrame
//!
//! No case 686 exists in `NPC.FindFrame`;
//! `Main.npcFrameCount[686] = 1`. Nothing to port.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_ai_style37::rotated_by;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// The type-686 `SetDefaults` row (`NPC.cs:17624-17639`), verbatim.
/// The row writes NO HitSound (null — the balloon only pops).
pub struct ClumsyBalloonDefaults {
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
    pub rarity: i32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub dont_take_damage_from_hostiles: bool,
}

/// `NPC.cs:17624-17639` — `else if (this.type == 686)`.
pub const CLUMSY_BALLOON: ClumsyBalloonDefaults = ClumsyBalloonDefaults {
    width: 20,
    height: 20,
    ai_style: 125,
    damage: 0,
    defense: 0,
    life_max: 1,
    death_sound: (4, 63),
    knock_back_resist: 0.3,
    value: 0.0,
    rarity: 2,
    no_gravity: true,
    no_tile_collide: true,
    dont_take_damage_from_hostiles: true,
};

/// `Main.npcFrameCount[686]` = 1 (`Main.cs:67152`, the 680-689 row).
pub const CLUMSY_BALLOON_FRAME_COUNT: u32 = 1;

/// The per-tick environment.
pub struct Style125Env<'a> {
    pub world: &'a World,
    pub gore: &'a mut GorePool,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.windSpeedTarget` — the cruise cap.
    pub wind_speed_target: f32,
    /// `Main.netMode`.
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style125Outcome {
    /// The pop (`:43722-43726`): `life = -1` + HitEffect (the gore
    /// burst ran INSIDE the step) + `active = false`. The caller owes
    /// the pool removal.
    pub died: bool,
    /// The pop's `SoundEngine.PlaySound(this.DeathSound, position)`
    /// — `(type, style, x, y)` = NPCDeath63 `(4, 63)`.
    pub sounds: Vec<(i32, i32, f32, f32)>,
    /// The HitEffect release (`:81771`): `Transform(680)` — the
    /// TownSlimePurple. The wiring runs the full Transform (the
    /// position re-anchor `Bottom + (0,48)` already applied here).
    pub transform: Option<i32>,
    /// The `unlockedSlimePurpleSpawn` latch (`:81772-81779`) — the
    /// wiring's world-state flag (the SendData 7 on netMode 2).
    pub unlock_purple_slime: bool,
    /// The X-rewind amount (`oldVx + direction*8`) applied on the pop.
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

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE NextFloat.
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

/// The target slot's candidate — the style16 `target_of` fallback.
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// The HitEffect type-686 arm (`NPC.cs:81752-81781`) — the `life <= 0`
/// gore burst + the town-slime release. The caller sets `life = -1`
/// (the port's `hp`) BEFORE calling, exactly like the pop path.
fn balloon_hit_effect(npc: &mut Npc, env: &mut Style125Env<'_>, out: &mut Style125Outcome) {
    if npc.hp > 0 {
        return; // the `life <= 0` gate (`:81752`)
    }
    let (w, h) = npc.size();
    let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
    // `:81754-81755` — the gore base + the rotated anchor.
    let gore_base = 1143;
    let (rx, ry) = rotated_by(0.0, -50.0, npc.rotation as f64, 0.0, 0.0);
    let anchor = bottom + Vec2::new(rx, ry);
    // `:81756-81761` — the three colour gores.
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
    // `:81762-81767` — the single 1146 gore (the (5,5) circle).
    let pos = anchor + next_vector2_circular(env.rng, 5.0, 5.0);
    let vel = next_vector2_circular(env.rng, 2.0, 2.0);
    let idx = env
        .gore
        .new_gore_perfect(env.rng, env.world, pos, vel, 1146, 1.0);
    let g = env.gore.slot_mut(idx);
    g.velocity.y -= env.rng.next_float();
    g.velocity *= 4.0;
    // `:81768-81780` — the non-client release.
    if env.net_mode != 1 {
        // `position = Bottom + (0, 48)`.
        npc.x = bottom.x;
        npc.y = bottom.y + 48.0;
        out.transform = Some(680);
        out.unlock_purple_slime = true;
    }
}

/// `NPC.GetTileCollisionParameters`'s 686 arm
/// (`NPC.cs:92821-92829`): the string-extended collision box (the
/// fixed 64-px down-extension, the width-32 re-centre).
pub fn balloon_collision_box(x: f32, y: f32, w: i32, h: i32) -> (f32, f32, i32, i32) {
    let num = 64;
    let cy = y + num as f32;
    let ch = h + num;
    let cw = 32;
    // `cPosition.X += cWidth / 2` reads the OLD width (the C# order).
    let cx = x + (w / 2) as f32 - (cw / 2) as f32;
    (cx, cy, cw, ch)
}

/// One fixed step of the aiStyle-125 arm (`NPC.cs:43688-43819`), in C#
/// order. The caller owns the movement integration (through
/// [`balloon_collision_box`]) and drains [`Style125Outcome`].
pub fn style125_step(npc: &mut Npc, env: &mut Style125Env<'_>) -> Style125Outcome {
    let mut out = Style125Outcome::default();
    // `:43690-43695` — the first tick.
    if npc.local_ai[0] == 0.0 && env.net_mode != 1 {
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 125, env.ticker);
        }
        npc.local_ai[0] = 1.0;
        npc.net_update = true;
    }
    // `:43696-43697` — the facing + the tilt.
    npc.sprite_dir = npc.facing;
    npc.rotation = npc.vx * 0.05;
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let bottom = Vec2::new(npc.x + w / 2.0, npc.y + h);
    // `:43698-43714` — the target vector.
    let mut dist = f32::INFINITY; // num
    let mut extra_depth = 0i32; // num2
    if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize];
        if p.active && !p.dead && !p.ghost {
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
    // `:43715-43728` — the pop gate.
    if npc.wet || npc.collide_x || (npc.collide_y && npc.old_vy < 0.0) {
        let rewind = npc.old_vx + npc.facing as f32 * 8.0;
        npc.x -= rewind;
        out.pop_rewind = Some(rewind);
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, false, 125, env.ticker);
        }
        npc.facing *= -1;
        npc.vx = npc.facing as f32 * 2.0;
        npc.hp = -1; // `this.life = -1`
        balloon_hit_effect(npc, env, &mut out);
        out.died = true; // `active = false`
        npc.net_update = true;
        out.sounds.push((4, 63, npc.x, npc.y));
        return out;
    }
    // `:43729-43733` — the collideY snap.
    if npc.collide_y {
        npc.vy = if npc.old_vy > 0.0 { 1.0 } else { -1.0 };
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, false, 125, env.ticker);
        }
    }
    // `:43734-43753` — the wind cruise.
    let cap = 3.0 + env.wind_speed_target.abs() * 2.0; // num4
    if math_sign(npc.vx) != npc.facing || npc.vx.abs() < cap {
        npc.vx += npc.facing as f32 * 0.04;
        if npc.vx * (npc.facing as f32) < 0.0 {
            if npc.vx.abs() > cap {
                npc.vx += npc.facing as f32 * 0.15;
            } else {
                npc.vx += npc.facing as f32 * 0.1;
            }
        } else if npc.vx.abs() > cap {
            npc.vx = npc.facing as f32 * cap;
        }
    }
    // `:43754-43774` — the ground scan (8 + num2 deep).
    let col = ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32 + npc.facing;
    let row = ((npc.y + h) / 16.0) as i32;
    let mut no_ground = true; // flag
    let depth = 8 + extra_depth; // num7
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
    // `:43775-43818` — the hover OR the gravity ladder.
    let p = target_of(npc, env.players);
    let sighted = crate::projectile::can_hit(
        env.world,
        npc.x,
        npc.y,
        w,
        h,
        p.x,
        p.y,
        p.width as f32,
        p.height as f32,
    );
    if dist < 400.0 && sighted {
        let cap_y = 2.0f32; // num8
        let accel = 0.035f32; // num9
        let my_quarter = center.y + (h as i32 / 4) as f32;
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
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 686-shaped NPC (the style73 placeholder convention).
    fn balloon(x: f32, y: f32) -> Npc {
        let d = &CLUMSY_BALLOON;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.local_ai[0] = 1.0; // skip the first tick by default
        n.target = 0;
        n
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

    /// `NPC.cs:17624-17639` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s125_the_defaults_row() {
        let d = &CLUMSY_BALLOON;
        assert_eq!((d.width, d.height), (20, 20));
        assert_eq!(d.ai_style, 125);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 1));
        assert_eq!(d.death_sound, (4, 63));
        assert_eq!(d.knock_back_resist, 0.3);
        assert_eq!(d.value, 0.0);
        assert_eq!(d.rarity, 2);
        assert!(d.no_gravity && d.no_tile_collide && d.dont_take_damage_from_hostiles);
        assert_eq!(CLUMSY_BALLOON_FRAME_COUNT, 1);
    }

    /// `:43690-43697` — the first tick + the facing/tilt writes.
    #[test]
    fn w4_d33_s125_the_first_tick() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(125);
        let mut n = balloon(100.0, 100.0);
        n.local_ai[0] = 0.0;
        n.vx = 2.0;
        let mut env = Style125Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        let _ = style125_step(&mut n, &mut env);
        assert_eq!(n.local_ai[0], 1.0);
        assert!(n.net_update);
        assert_eq!(n.sprite_dir, n.facing);
        assert_eq!(n.rotation, 0.1, "vx 2 * 0.05");
    }

    /// `:43715-43728` + `:81752-81781` — the pop: the rewind, the
    /// facing flip, life -1, the 4-gore burst (5 draws each), the
    /// Transform(680) + the unlock latch, the death flag.
    #[test]
    fn w4_d33_s125_the_pop_and_release() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(125);
        let mut oracle = UnifiedRandom::new(125);
        let mut n = balloon(100.0, 100.0);
        n.wet = true;
        n.old_vx = 1.0;
        let mut env = Style125Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        let out = style125_step(&mut n, &mut env);
        assert!(out.died);
        assert_eq!(n.hp, -1);
        assert_eq!(n.facing, -1);
        assert_eq!(n.vx, -2.0);
        assert_eq!(out.pop_rewind, Some(9.0));
        assert_eq!(out.transform, Some(680));
        assert!(out.unlock_purple_slime);
        assert!(out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (4, 63)));
        // The 4 gores: each 2+2 circular + the type (3 only) + the vy.
        for g in 0..4 {
            oracle.next_float();
            oracle.next_float();
            oracle.next_float();
            oracle.next_float();
            if g < 3 {
                oracle.next_max(3);
            }
            oracle.next_float();
        }
        assert_eq!(env.rng.peek(), oracle.peek(), "the gore argument order");
        // The client pops but does NOT release.
        let mut n = balloon(100.0, 100.0);
        n.wet = true;
        env.net_mode = 1;
        let out = style125_step(&mut n, &mut env);
        assert!(out.died);
        assert_eq!(out.transform, None);
        assert!(!out.unlock_purple_slime);
    }

    /// `:43734-43753` — the wind cruise: the 3 + |wind|*2 cap, the
    /// 0.04 accel, the against-sign 0.1 double-accel.
    #[test]
    fn w4_d33_s125_the_wind_cruise() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(125);
        let mut env = Style125Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.5,
            net_mode: 0,
            ticker: 7,
        };
        let mut n = balloon(100.0, 100.0);
        let _ = style125_step(&mut n, &mut env);
        assert!((n.vx - 0.04).abs() < 1e-6, "the 0.04 accel");
        let mut n = balloon(100.0, 100.0);
        n.vx = -1.0; // against the +1 facing, under the 4 cap
        let _ = style125_step(&mut n, &mut env);
        assert!(
            (n.vx - (-0.86)).abs() < 1e-6,
            "the 0.04 + 0.1 double, vx {}",
            n.vx
        );
    }

    /// `NPC.cs:92821-92829` — the string-extended collision box.
    #[test]
    fn w4_d33_s125_the_collision_box() {
        let (x, y, w, h) = balloon_collision_box(100.0, 100.0, 20, 20);
        assert_eq!((y, h), (164.0, 84));
        assert_eq!(w, 32);
        assert_eq!(x, 100.0 + 10.0 - 16.0);
    }

    /// `:43729-43733` — the collideY snap (the oldVy sign).
    #[test]
    fn w4_d33_s125_the_collide_y_snap() {
        let w = air();
        let mut gore = GorePool::new();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(125);
        let mut n = balloon(100.0, 100.0);
        n.collide_y = true;
        n.old_vy = 2.0; // falling → snap +1
        let mut env = Style125Env {
            world: &w,
            gore: &mut gore,
            rng: &mut rng,
            players: &players,
            wind_speed_target: 0.0,
            net_mode: 0,
            ticker: 7,
        };
        let _ = style125_step(&mut n, &mut env);
        // The snap set vy 1; the ladder/hover may adjust it, but the
        // far unsighted target runs the ladder: no_ground → +0.05.
        assert!(n.vy > 0.0, "the falling snap");
    }

    /// The stream parity: 60 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s125_the_run_is_twin_stable() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut a = balloon(280.0, 280.0);
        let mut b = a.clone();
        let mut gore_a = GorePool::new();
        let mut gore_b = GorePool::new();
        let mut rng_a = UnifiedRandom::new(686);
        let mut rng_b = UnifiedRandom::new(686);
        for tick in 0..60 {
            let mut env_a = Style125Env {
                world: &w,
                gore: &mut gore_a,
                rng: &mut rng_a,
                players: &players,
                wind_speed_target: 0.3,
                net_mode: 0,
                ticker: 7,
            };
            let _ = style125_step(&mut a, &mut env_a);
            let mut env_b = Style125Env {
                world: &w,
                gore: &mut gore_b,
                rng: &mut rng_b,
                players: &players,
                wind_speed_target: 0.3,
                net_mode: 0,
                ticker: 7,
            };
            let _ = style125_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.vy.to_bits(), b.vy.to_bits(), "vy @{tick}");
        }
    }
}
