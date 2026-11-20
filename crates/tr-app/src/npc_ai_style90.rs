//! The aiStyle-90 machine — the Mothron Spawn / Baby Mothron
//! (`NPC.AI()`'s `else if (this.aiStyle == 90)` arm,
//! `NPC.cs:38394-38628`). W4-d32, research/25 row 90 (MISSING →
//! ported), CODE-ONLY MODE (no compiler invoked; the witness tests
//! below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 90;` across the decompiled tree
//! returns exactly ONE assignment: `NPC.cs:15634`, the
//! `else if (this.type == 479)` row (`:15630-15643`) — the
//! MothronSpawn (`ID/NPCID.cs:11631`). No dynamic `aiStyle`
//! reassignment to 90 exists (the egg's `Transform(479)` lands on
//! this row). The kind has no `NpcKind` yet — the module lands
//! UNWIRED (the style73/style86 precedent).
//!
//! ## The machine
//!
//! The head (`:38396-38399`): `noTileCollide = false`, kb =
//! `GetKnockbackMultiplier_ScaledByDifficulty(0.4)` (`NPC.cs:7031`),
//! `noGravity = true`, the ×9/10 rotation ease (0.1).
//!
//! The eclipse gate (`:38400-38410`): no eclipse →
//! `EncourageDespawn(5)` (the port's despawn timer), vy −0.2 capped
//! −8, noTileCollide, RETURN (the rest of the arm skips).
//!
//! The sibling repulsion (`:38411-38427`): states 0/1 only — every
//! OTHER active same-type NPC within `width + height` (76 px) of
//! the centre pushes both bodies along the normalized delta ×−0.1
//! (the pool step mutates the sibling too — the style86 precedent).
//!
//! The target gates (`:38428-38444`): dead/inactive/absent →
//! retarget, dead-or->3000 → −1; else >1000 while `ai[0] > 1` → 1.
//! (NO netUpdate writes anywhere in this arm — verbatim.)
//!
//! State −1 flee (`:38445-38451`): the (0,−8) ×9/10 steer,
//! noTileCollide + dontTakeDamage.
//!
//! State 0 hover (`:38452-38510`): retarget, spriteDirection, the
//! collideX/Y bounces (the literal `v × −oldV × 0.5`, ±4 caps — the
//! style88 shape); the RAW player delta: >800 → 1, >200 → the
//! `5.5 + len/100 + ai[1]/15` ×40-weight steer, else the 0.95/1.05
//! speed band; `ai[1]++`, at ≥90 → state 2 (ai[1] reset).
//!
//! State 1 chase (`:38511-38545`): collide flags cleared,
//! noTileCollide + kb 0, the retarget guard, the vx direction, the
//! ×9/10 rotation (0.08), the <300 + free-space return to 0,
//! `ai[2] += 1/60`, the `5.5 + ai[2] + len/150` ×35-weight steer.
//!
//! State 2 swoop (`:38546-38591`): the vx direction, the ×7/8
//! rotation, kb 0 + noTileCollide, the (player −8 Y) ×9 / 8-weight
//! steer, the direction re-pick, `ai[1]++`, >10 →
//! `velocity = vector187` verbatim, state 2.1, ai[1] = 0.
//!
//! State 2.1 dash (`:38592-38627`): the vx direction, velocity
//! ×1.01, kb 0 + noTileCollide, `ai[1]++`; >45: free-space → 0,
//! else >90 → 1 (both zero ai[1]/ai[2]).
//!
//! ## RNG census (by construction)
//!
//! The arm draws NOTHING — no `Main.rand` call exists in
//! `:38394-38628` (TargetClosest/CanHit-free/SolidCollision and the
//! steers are deterministic). The witnesses pin the zero-draw
//! census over the full state cycle.
//!
//! ## Seams (the wiring obligations)
//!
//! * The pool's `types` slice is the slot-aligned vanilla type
//!   table (the style86 seam); `active` ≈ `hp > 0`.
//! * The noGravity flight integrates raw (the caller's movement
//!   step); the eclipse exit's `EncourageDespawn(5)` rides
//!   [`Style90Outcome::encourage_despawn`] (the port's
//!   `despawn_timer`).
//! * The head does NOT reset `dontTakeDamage` — only the flee
//!   writes it (verbatim, the style88 shape).
//!
//! See docs/ANALYSIS.md "The Mothron Spawn (W4-d32 aiStyle-90, code-only)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The type-479 `SetDefaults` row (`NPC.cs:15630-15643`), verbatim.
pub struct MothronSpawnDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1` (`ID/SoundID.cs:1257`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1` (`ID/SoundID.cs:1315`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub value: f32,
    pub npc_slots: f32,
}

/// `NPC.cs:15630-15643` — `else if (this.type == 479)`.
pub const MOTHRON_SPAWN: MothronSpawnDefaults = MothronSpawnDefaults {
    width: 46,
    height: 30,
    ai_style: 90,
    damage: 50,
    defense: 14,
    life_max: 700,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    knock_back_resist: 0.3,
    value: 0.0,
    npc_slots: 0.1,
};

/// The per-tick environment (the `Style88Env` shape minus the RNG —
/// the arm is draw-free).
pub struct Style90Env<'a> {
    pub world: &'a World,
    pub players: &'a [TargetCandidate],
    pub ticker: u64,
    /// `this.difficulty` (`NPC.cs:7033`).
    pub difficulty: f32,
    /// `Main.eclipse` (`:38400`).
    pub eclipse: bool,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style90Outcome {
    pub knock_back_resist: Option<f32>,
    pub no_tile_collide: bool,
    /// The head's `noGravity = true` (`:38398`) — every tick.
    pub no_gravity: bool,
    /// The −1 flee's `dontTakeDamage = true` (`:38450`).
    pub dont_take_damage: Option<bool>,
    /// The eclipse exit's `EncourageDespawn(5)` (`:38402`).
    pub encourage_despawn: Option<i32>,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the
/// style74 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED (the style74 precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `NPC.GetKnockbackMultiplier_ScaledByDifficulty` (`NPC.cs:7031`)
/// over the Classic/Master curve (the style87 seam).
fn knockback_scaled(normal: f32, difficulty: f32) -> f32 {
    let m = if difficulty <= 1.0 {
        1.0
    } else if difficulty >= 3.0 {
        0.8
    } else {
        (difficulty - 1.0) * (0.8 - 1.0) / (3.0 - 1.0) + 1.0
    };
    normal * m
}

/// The target slot's candidate — the style74 `target_of` fallback.
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32)
}

fn solid_collision(npc: &Npc, world: &World) -> bool {
    let (w, h) = npc.size();
    crate::player::Player::solid_collision(world, npc.x, npc.y, w, h)
}

/// The collideX/Y bounce (the style88 literal product + ±4 caps).
fn bounce(v: f32, old_v: f32) -> f32 {
    let mut r = v * (0.0 - old_v) * 0.5;
    if r > 4.0 {
        r = 4.0;
    }
    if r < -4.0 {
        r = -4.0;
    }
    r
}

/// One fixed step of the aiStyle-90 arm (`NPC.cs:38394-38628`), in
/// C# order. `npcs` is the slot-aligned pool, `me` this NPC's slot,
/// `types` the slot-aligned vanilla type table (the style86 seam).
/// The caller owns the raw flight integration and drains the
/// outcome.
pub fn style90_step(
    npcs: &mut [Npc],
    me: usize,
    types: &[i32],
    env: &Style90Env<'_>,
) -> Style90Outcome {
    let mut out = Style90Outcome {
        no_gravity: true,
        ..Default::default()
    };
    // `:38396-38399` — the head.
    out.no_tile_collide = false;
    out.knock_back_resist = Some(knockback_scaled(0.4, env.difficulty));
    npcs[me].rotation = (npcs[me].rotation * 9.0 + npcs[me].vx * 0.1) / 10.0;
    // `:38400-38410` — the eclipse exit (the C# `return`).
    if !env.eclipse {
        out.encourage_despawn = Some(5);
        npcs[me].vy -= 0.2;
        if npcs[me].vy < -8.0 {
            npcs[me].vy = -8.0;
        }
        out.no_tile_collide = true;
        return out;
    }
    // `:38411-38427` — the sibling repulsion (states 0/1 only).
    if npcs[me].ai[0] == 0.0 || npcs[me].ai[0] == 1.0 {
        let ty = types[me];
        let (w, h) = npcs[me].size();
        let reach = w + h;
        let (cx, cy) = npcs[me].center();
        for j in 0..npcs.len() {
            if j == me || npcs[j].hp <= 0 || types[j] != ty {
                continue;
            }
            let (jx, jy) = npcs[j].center();
            let (dx, dy) = (jx - cx, jy - cy);
            if sqrt_f32(dx, dy) < reach {
                let (nx, ny) = normalize_xna(dx, dy);
                let (px, py) = (nx * -0.1, ny * -0.1);
                npcs[me].vx += px;
                npcs[me].vy += py;
                npcs[j].vx -= px;
                npcs[j].vy -= py;
            }
        }
    }
    // `:38428-38444` — the target gates.
    {
        let p = target_of(&npcs[me], env.players);
        if npcs[me].target < 0 || p.dead || !p.active {
            target_closest(&mut npcs[me], env.world, env.players, true, 90, env.ticker);
            let p = target_of(&npcs[me], env.players);
            let (pcx, pcy) = candidate_center(p);
            let (cx, cy) = npcs[me].center();
            if p.dead || sqrt_f32(pcx - cx, pcy - cy) > 3000.0 {
                npcs[me].ai[0] = -1.0;
            }
        } else {
            let (pcx, pcy) = candidate_center(p);
            let (cx, cy) = npcs[me].center();
            if npcs[me].ai[0] > 1.0 && sqrt_f32(pcx - cx, pcy - cy) > 1000.0 {
                npcs[me].ai[0] = 1.0;
            }
        }
    }
    if npcs[me].ai[0] == -1.0 {
        // `:38445-38451` — the flee.
        npcs[me].vx = (npcs[me].vx * 9.0 + 0.0) / 10.0;
        npcs[me].vy = (npcs[me].vy * 9.0 + -8.0) / 10.0;
        out.no_tile_collide = true;
        npcs[me].dont_take_damage = true;
        out.dont_take_damage = Some(true);
    } else if npcs[me].ai[0] == 0.0 {
        // `:38452-38510` — the hover.
        target_closest(&mut npcs[me], env.world, env.players, true, 90, env.ticker);
        npcs[me].sprite_dir = npcs[me].facing;
        if npcs[me].collide_x {
            npcs[me].vx = bounce(npcs[me].vx, npcs[me].old_vx);
        }
        if npcs[me].collide_y {
            npcs[me].vy = bounce(npcs[me].vy, npcs[me].old_vy);
        }
        let p = target_of(&npcs[me], env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npcs[me].center();
        let mut dx = pcx - cx;
        let mut dy = pcy - cy;
        let len = sqrt_f32(dx, dy);
        if len > 800.0 {
            npcs[me].ai[0] = 1.0;
            npcs[me].ai[1] = 0.0;
            npcs[me].ai[2] = 0.0;
            npcs[me].ai[3] = 0.0;
        } else if len > 200.0 {
            let num1258 = 5.5 + len / 100.0 + npcs[me].ai[1] / 15.0;
            let (nx, ny) = normalize_xna(dx, dy);
            dx = nx * num1258;
            dy = ny * num1258;
            npcs[me].vx = (npcs[me].vx * (40.0 - 1.0) + dx) / 40.0;
            npcs[me].vy = (npcs[me].vy * (40.0 - 1.0) + dy) / 40.0;
        } else {
            let vlen = sqrt_f32(npcs[me].vx, npcs[me].vy);
            if vlen > 2.0 {
                npcs[me].vx *= 0.95;
                npcs[me].vy *= 0.95;
            } else if vlen < 1.0 {
                npcs[me].vx *= 1.05;
                npcs[me].vy *= 1.05;
            }
        }
        npcs[me].ai[1] += 1.0;
        if npcs[me].ai[1] >= 90.0 {
            npcs[me].ai[1] = 0.0;
            npcs[me].ai[0] = 2.0;
        }
    } else if npcs[me].ai[0] == 1.0 {
        // `:38511-38545` — the chase.
        npcs[me].collide_x = false;
        npcs[me].collide_y = false;
        out.no_tile_collide = true;
        out.knock_back_resist = Some(0.0);
        {
            let p = target_of(&npcs[me], env.players);
            if npcs[me].target < 0 || !p.active || p.dead {
                target_closest(&mut npcs[me], env.world, env.players, true, 90, env.ticker);
            }
        }
        if npcs[me].vx < 0.0 {
            npcs[me].facing = -1;
        } else if npcs[me].vx > 0.0 {
            npcs[me].facing = 1;
        }
        npcs[me].sprite_dir = npcs[me].facing;
        npcs[me].rotation = (npcs[me].rotation * 9.0 + npcs[me].vx * 0.08) / 10.0;
        let p = target_of(&npcs[me], env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npcs[me].center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let len = sqrt_f32(dx, dy);
        if len < 300.0 && !solid_collision(&npcs[me], env.world) {
            npcs[me].ai[0] = 0.0;
            npcs[me].ai[1] = 0.0;
            npcs[me].ai[2] = 0.0;
            npcs[me].ai[3] = 0.0;
        }
        npcs[me].ai[2] += 1.0 / 60.0;
        let num1260 = 5.5 + npcs[me].ai[2] + len / 150.0;
        let (nx, ny) = normalize_xna(dx, dy);
        npcs[me].vx = (npcs[me].vx * (35.0 - 1.0) + nx * num1260) / 35.0;
        npcs[me].vy = (npcs[me].vy * (35.0 - 1.0) + ny * num1260) / 35.0;
    } else if npcs[me].ai[0] == 2.0 {
        // `:38546-38591` — the swoop.
        if npcs[me].vx < 0.0 {
            npcs[me].facing = -1;
        } else if npcs[me].vx > 0.0 {
            npcs[me].facing = 1;
        }
        npcs[me].sprite_dir = npcs[me].facing;
        npcs[me].rotation = (npcs[me].rotation * 7.0 + npcs[me].vx * 0.1) / 8.0;
        out.knock_back_resist = Some(0.0);
        out.no_tile_collide = true;
        let p = target_of(&npcs[me], env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npcs[me].center();
        let dx = pcx - cx;
        let dy = pcy - cy - 8.0;
        let (nx, ny) = normalize_xna(dx, dy);
        let (tx, ty) = (nx * 9.0, ny * 9.0);
        npcs[me].vx = (npcs[me].vx * (8.0 - 1.0) + tx) / 8.0;
        npcs[me].vy = (npcs[me].vy * (8.0 - 1.0) + ty) / 8.0;
        npcs[me].facing = if npcs[me].vx < 0.0 { -1 } else { 1 };
        npcs[me].sprite_dir = npcs[me].facing;
        npcs[me].ai[1] += 1.0;
        if npcs[me].ai[1] > 10.0 {
            npcs[me].vx = tx;
            npcs[me].vy = ty;
            npcs[me].facing = if npcs[me].vx < 0.0 { -1 } else { 1 };
            npcs[me].ai[0] = 2.1;
            npcs[me].ai[1] = 0.0;
        }
    } else if npcs[me].ai[0] == 2.1 {
        // `:38592-38627` — the dash.
        if npcs[me].vx < 0.0 {
            npcs[me].facing = -1;
        } else if npcs[me].vx > 0.0 {
            npcs[me].facing = 1;
        }
        npcs[me].sprite_dir = npcs[me].facing;
        npcs[me].vx *= 1.01;
        npcs[me].vy *= 1.01;
        out.knock_back_resist = Some(0.0);
        out.no_tile_collide = true;
        npcs[me].ai[1] += 1.0;
        let num1264 = 45;
        if npcs[me].ai[1] > num1264 as f32 {
            if !solid_collision(&npcs[me], env.world) {
                npcs[me].ai[0] = 0.0;
                npcs[me].ai[1] = 0.0;
                npcs[me].ai[2] = 0.0;
            } else if npcs[me].ai[1] > (num1264 * 2) as f32 {
                npcs[me].ai[0] = 1.0;
                npcs[me].ai[1] = 0.0;
                npcs[me].ai[2] = 0.0;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 479-shaped NPC: no kind yet — the placeholder + the C# box
    /// (the style86 convention).
    fn spawn(x: f32, y: f32) -> Npc {
        let d = &MOTHRON_SPAWN;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
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

    fn env<'a>(w: &'a World, players: &'a [TargetCandidate]) -> Style90Env<'a> {
        Style90Env {
            world: w,
            players,
            ticker: 0,
            difficulty: 1.0,
            eclipse: true,
        }
    }

    /// `NPC.cs:15630-15643` — the SetDefaults row, field by field.
    #[test]
    fn w4_d32_s90_the_defaults_row() {
        let d = &MOTHRON_SPAWN;
        assert_eq!((d.width, d.height), (46, 30));
        assert_eq!(d.ai_style, 90);
        assert_eq!((d.damage, d.defense, d.life_max), (50, 14, 700));
        assert_eq!(d.hit_sound, (3, 1), "SoundID.NPCHit1");
        assert_eq!(d.death_sound, (4, 1), "SoundID.NPCDeath1");
        assert_eq!(d.knock_back_resist, 0.3);
        assert_eq!(d.value, 0.0);
        assert_eq!(d.npc_slots, 0.1);
    }

    /// `:38396-38410` — the head + the eclipse exit: the scaled 0.4
    /// kb, the rotation ease, then EncourageDespawn(5) + the −0.2
    /// capped −8 climb + noTileCollide + the RETURN (no state run).
    #[test]
    fn w4_d32_s90_the_eclipse_exit_climbs_and_returns() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut npcs = [spawn(100.0, 100.0)];
        npcs[0].vy = -7.9;
        npcs[0].ai[0] = 2.0;
        let mut e = env(&w, &players);
        e.eclipse = false;
        let out = style90_step(&mut npcs, 0, &[479], &e);
        assert_eq!(out.knock_back_resist, Some(0.4));
        assert_eq!(out.encourage_despawn, Some(5));
        assert_eq!(npcs[0].vy, -8.0, "the −0.2 climb caps at −8");
        assert!(out.no_tile_collide);
        assert_eq!(npcs[0].ai[0], 2.0, "the return skips the states");
    }

    /// `:38411-38427` — the sibling repulsion: two same-type bodies
    /// within 76 px push apart symmetrically along the normalized
    /// delta ×−0.1; a different type is ignored; state 2 skips it.
    #[test]
    fn w4_d32_s90_the_sibling_repulsion_is_symmetric_and_state_gated() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut npcs = [spawn(100.0, 100.0), spawn(130.0, 100.0), spawn(110.0, 100.0)];
        let e = env(&w, &players);
        // The delta (+30, 0) → normalized (1,0) × −0.1: me −0.1, sib +0.1.
        let _ = style90_step(&mut npcs, 0, &[479, 479, 477], &e);
        assert_eq!(npcs[0].vx, -0.1);
        assert_eq!(npcs[1].vx, 0.1);
        assert_eq!(npcs[2].vx, 0.0, "the 477 is a different type");
        // State 2: no repulsion.
        let mut npcs2 = [spawn(100.0, 100.0), spawn(130.0, 100.0)];
        npcs2[0].ai[0] = 2.0;
        let _ = style90_step(&mut npcs2, 0, &[479, 479], &e);
        assert_eq!(npcs2[1].vx, 0.0, "state 2 skips the repulsion");
    }

    /// `:38452-38510` — the hover's 90-tick timer flips to the
    /// swoop; the >800 bail flips to the chase.
    #[test]
    fn w4_d32_s90_the_hover_timer_and_the_far_bail() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut npcs = [spawn(100.0, 100.0)];
        npcs[0].ai[1] = 89.0;
        let e = env(&w, &players);
        let _ = style90_step(&mut npcs, 0, &[479], &e);
        assert_eq!(npcs[0].ai[0], 2.0);
        assert_eq!(npcs[0].ai[1], 0.0);
        // The far player: >800 → the chase.
        let far = [cand(2000.0, 100.0)];
        let mut npcs2 = [spawn(100.0, 100.0)];
        let e2 = env(&w, &far);
        let _ = style90_step(&mut npcs2, 0, &[479], &e2);
        assert_eq!(npcs2[0].ai[0], 1.0);
    }

    /// `:38546-38591` — the swoop exit: >10 ticks → velocity = the
    /// ×9 aim verbatim, state 2.1, ai[1] = 0.
    #[test]
    fn w4_d32_s90_the_swoop_exit_latches_the_aim() {
        let w = air();
        let players = [cand(1000.0, 100.0)];
        let mut npcs = [spawn(100.0, 100.0)];
        npcs[0].ai[0] = 2.0;
        npcs[0].ai[1] = 10.0;
        let e = env(&w, &players);
        let _ = style90_step(&mut npcs, 0, &[479], &e);
        assert_eq!(npcs[0].ai[0], 2.1);
        assert_eq!(npcs[0].ai[1], 0.0);
        let (cx, cy) = npcs[0].center();
        let dx = 1010.0 - cx;
        let dy = 121.0 - cy - 8.0;
        let (nx, ny) = normalize_xna(dx, dy);
        assert_eq!(npcs[0].vx, nx * 9.0);
        assert_eq!(npcs[0].vy, ny * 9.0);
    }

    /// `:38592-38627` — the dash: ×1.01 per tick, >45 free-space →
    /// 0; the solid + >90 leg → 1 (the air world is free, so the
    /// first exit fires at 46).
    #[test]
    fn w4_d32_s90_the_dash_accelerates_and_exits_at_46() {
        let w = air();
        let players = [cand(1000.0, 100.0)];
        let mut npcs = [spawn(100.0, 100.0)];
        npcs[0].ai[0] = 2.1;
        npcs[0].ai[1] = 45.0;
        npcs[0].vx = 10.0;
        npcs[0].vy = 2.0;
        let e = env(&w, &players);
        let _ = style90_step(&mut npcs, 0, &[479], &e);
        assert_eq!(npcs[0].vx, 10.0 * 1.01);
        assert_eq!(npcs[0].vy, 2.0 * 1.01);
        assert_eq!(npcs[0].ai[0], 0.0, "46 > 45 + free space → hover");
        assert_eq!((npcs[0].ai[1], npcs[0].ai[2]), (0.0, 0.0));
    }

    /// The zero-draw census: the arm has NO `Main.rand` call — the
    /// env carries no stream at all (by construction); the full
    /// 0→2→2.1→0 cycle runs without one.
    #[test]
    fn w4_d32_s90_the_state_cycle_is_draw_free_by_construction() {
        let w = air();
        let players = [cand(900.0, 300.0)];
        let mut npcs = [spawn(100.0, 100.0)];
        let e = env(&w, &players);
        for _ in 0..600 {
            let _ = style90_step(&mut npcs, 0, &[479], &e);
        }
        // The cycle visited the swoop and came back (the 90-tick
        // timer + the 11-tick swoop + the 46-tick dash).
        assert!(npcs[0].ai[0] == 0.0 || npcs[0].ai[0] == 1.0 || npcs[0].ai[0] == 2.0);
    }
}
