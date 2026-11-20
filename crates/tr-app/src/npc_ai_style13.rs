//! W4-d23 (aiStyle-13) — the anchored man-eater family, `NPC.AI`'s
//! `if (this.aiStyle == 13)` block (`NPC.cs:22615-22847`), ported 1:1
//! as a PURE machine (code-only mode: no compiler/tests were run on
//! this branch; the witness tests below are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 13` rows, whole-file grep)
//!
//! SIX vanilla members, NONE with a port `NpcKind` — the module lands
//! UNWIRED (the W4-d24 `npc_ai_style17/20` precedent); the step is keyed
//! on the C# type id (`npc_type`), the wiring passes `kind.net_id()`:
//!
//! | type | name (`ID/NPCID.cs`) | SetDefaults | box | dmg | def | HP | kb | value |
//! | --- | --- | --- | --- | --- | --- | --- | --- | --- |
//! | 43 | ManEater (`:10763`) | `NPC.cs:9293-9307` | 30×30 | 34 | 10 | 110 | 0 | 350 |
//! | 56 | Snatcher (`:10789`) | `:9474-9488` | 30×30 | 25 | 10 | 60 | 0 | 90 |
//! | 101 | Clinger (`:10879`) | `:10186-10201` | 30×30 | 70 | 30 | 320 | 0.2 | 600 |
//! | 175 | AngryTrapper (`:11027`) | `:11208-11222` | 30×30 | 100 | 30 | 300 | 0.25 | 650 |
//! | 259 | FungiBulb (`:11195`) | `:12332-12347` | 20×20 | 24 | 4 | 90 | 0 | 350 |
//! | 260 | GiantFungiBulb (`:11197`) | `:12348-12363` | 36×36 | 70 | 20 | 300 | 0 | 1250 |
//!
//! All six are noGravity + noTileCollide; the Clinger 101 alone is
//! behindTiles (`:10190`); the two fungi bulbs carry npcSlots 0.3.
//! HitSound NPCHit1 / DeathSound NPCDeath1 for all six.
//!
//! ## The machine — the tile-anchored leash
//!
//! * the anchor (`:22617-22633`): `ai[0]`/`ai[1]` are the TILE coords
//!   (int-truncated). `!WorldGen.InWorld(x, y, 5)` (`WorldGen.cs:9688-
//!   9695`, the 5-tile fluff) returns. The null-tile arm (`:22623-
//!   22626`) allocates an inactive tile — the port's `World.tile`
//!   `None` folds to the same inactive read. An INACTIVE anchor tile
//!   kills the NPC (`life = -1; HitEffect(); active = false` — the
//!   port's hp-0 death convention, the style41 precedent).
//! * `FixExploitManEaters.ProtectSpot` (`:22634`,
//!   `GameContent/FixExploitManEaters.cs:14-21`) — a tile-protection
//!   registry against the man-eater wall exploit; the port has no
//!   counterpart (no RNG, no AI-visible state) — documented skip.
//! * `TargetClosest()` (`:22635`), then the per-type leash pair
//!   (`:22636-22659`): accel 0.035 / range 150 default; 43 → 250 (ftw
//!   350); 101 → 175; 259 → 100; 175 → 500 + 0.05; 260 → 350 + 0.15.
//! * the enrage ladder (`:22660-22668`): `ai[2]++`; `> 300` scales the
//!   range `(int)((double)range * 1.3)` (the DOUBLE multiply then int
//!   truncation is load-bearing); `> 450` resets `ai[2] = 0`.
//! * the aim (`:22669-22678`): the anchor centre `(x*16+8, y*16+8)`,
//!   the delta to `player.Center - (float)(width/2)` (INTEGER half —
//!   the W81 defect class), clamped to the range by the reciprocal
//!   `range/dist` multiply (only when `dist > range`).
//! * the steering (`:22679-22710`): per axis, accel toward
//!   `anchor + delta` with the 1.5× double-add on a sign flip; the
//!   equality case steers neither way (the if/else-if).
//! * the clamps (`:22711-22733`): 43 → ±3 (ftw ±3.5), 175 → ±4, else
//!   ±2 — `MathHelper.Clamp` on both axes.
//! * the facing (`:22734-22750`): 259/260 → `rotation = atan2(dy, dx)
//!   + 1.57f` (no spriteDirection write); the others face by the
//!   delta's X sign — `+atan2` right, `+3.14f` left (the two
//!   independent ifs; a zero delta keeps the old rotation).
//! * the bounce (`:22751-22776`): collideX/Y → `netUpdate = true`,
//!   `velocity = oldVelocity * -0.7f` with the strict ±2 floors.
//! * the client return (`:22777-22780`) folds away (single-player).
//! * the Clinger 101's cursed-flame shot (`:22782-22809`): gated on a
//!   LIVE player; `justHit` zeroes `localAI[0]`, then `++`; at `>= 120`
//!   the `(!SolidCollision & flag23) && CanHit` gate fires the 10-speed
//!   aimed CursedFlame 96 (damage `GetAttackDamage_ForProjectiles(22f,
//!   17.6f)` — classic 22, the single-difficulty fold) with
//!   `timeLeft = 300` (the caller's write), else `localAI[0] = 100`.
//! * the GiantFungiBulb 260's spore spawn (`:22810-22846`): a dead
//!   player returns; `justHit` zeroes, `++`, at `>= 150` the same gate
//!   spawns a FungiSpore 261 at the INT centre with the 14-speed
//!   jittered aim — the `num221 = |aimX * 0.1|` lift is ZEROED when the
//!   STEERING `num216 > 0` (the C# reads the movement block's leftover
//!   delta, NOT the aim — verbatim quirk), else `localAI[0] = 130`.
//!
//! Seam: writes velocity, `ai[2]`, `localAI[0]`, rotation,
//! spriteDirection, `netUpdate`, hp (the anchor death). Does NOT
//! integrate: all six are noGravity + noTileCollide, so the caller's
//! `UpdateNPC` raw-integrates (`NPC.cs:90494-90498`, the
//! `integrate_no_tile_collide` seam). The shot/spawn leave as
//! [`Style13Out`] requests (the `wants_hostile_shot`/`wants_sphere`
//! shape); the spore's `NewNPC` Bottom semantics (`NPC.cs:80770`) are
//! the wiring's resolve. `Main.netMode != 1` folds true.
//!
//! ## RNG census
//!
//! The arm's ONLY `Main.rand` draws are the two shot/spawn aim jitters,
//! X BEFORE Y in both arms:
//! 1. the Clinger 101 shot (`:22795-22796`): `Next(-10, 11)` X then
//!    `Next(-10, 11)` Y — only on a qualifying shot tick (`localAI[0]
//!    >= 120` AND the gate passes). A blocked gate draws NOTHING.
//! 2. the 260 spore spawn (`:22827`/`:22833`): `Next(-10, 11)` X, then
//!    the `num221` lift (no draw), then `Next(-10, 11)` Y — only on a
//!    qualifying spawn tick (`localAI[0] >= 150` AND the gate passes).
//! Everything else — the anchor guards, TargetClosest
//! (`NPC.cs:77652-77781`), the steering, the bounce — draws NOTHING.
//! The witnesses mirror the rolls on a cloned stream and pin the
//! draw-free paths.
#![allow(dead_code)] // UNWIRED (W4-d23): the six kinds + the axis claim are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// One member's `SetDefaults` row (the census table above).
pub struct Style13Defaults {
    pub npc_type: i32,
    pub width: i32,
    pub height: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub value: f32,
    pub behind_tiles: bool,
    pub npc_slots: f32,
}

/// The six `aiStyle = 13` rows, verbatim (`NPC.cs:9293-9307` /
/// `:9474-9488` / `:10186-10201` / `:11208-11222` / `:12332-12347` /
/// `:12348-12363`). Every row: aiStyle 13, noGravity + noTileCollide,
/// NPCHit1/NPCDeath1; npcSlots stock 1 except the two bulbs' 0.3.
pub const STYLE13_ROWS: [Style13Defaults; 6] = [
    Style13Defaults { npc_type: 43, width: 30, height: 30, damage: 34, defense: 10, life_max: 110, knock_back_resist: 0.0, value: 350.0, behind_tiles: false, npc_slots: 1.0 },
    Style13Defaults { npc_type: 56, width: 30, height: 30, damage: 25, defense: 10, life_max: 60, knock_back_resist: 0.0, value: 90.0, behind_tiles: false, npc_slots: 1.0 },
    Style13Defaults { npc_type: 101, width: 30, height: 30, damage: 70, defense: 30, life_max: 320, knock_back_resist: 0.2, value: 600.0, behind_tiles: true, npc_slots: 1.0 },
    Style13Defaults { npc_type: 175, width: 30, height: 30, damage: 100, defense: 30, life_max: 300, knock_back_resist: 0.25, value: 650.0, behind_tiles: false, npc_slots: 1.0 },
    Style13Defaults { npc_type: 259, width: 20, height: 20, damage: 24, defense: 4, life_max: 90, knock_back_resist: 0.0, value: 350.0, behind_tiles: false, npc_slots: 0.3 },
    Style13Defaults { npc_type: 260, width: 36, height: 36, damage: 70, defense: 20, life_max: 300, knock_back_resist: 0.0, value: 1250.0, behind_tiles: false, npc_slots: 0.3 },
];

/// The Clinger's cursed-flame shot request — `Projectile.NewProjectile`
/// at `NPC.cs:22800-22801`: type 96 (`ProjectileID.CursedFlame`), damage
/// 22 (the classic `GetAttackDamage_ForProjectiles(22f, 17.6f)` fold),
/// knockback 0, owner `Main.myPlayer`, at the float centre; the caller
/// sets `timeLeft = 300` (`:22801`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursedFlameShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// The GiantFungiBulb's spore request — `NPC.NewNPC(..., (int)Center.X,
/// (int)Center.Y, 261)` at `NPC.cs:22837-22840`: the INT-truncated
/// centre (the `NewNPC` Bottom semantics, `:80770`, are the wiring's
/// resolve), the 14-speed jittered velocity, `netUpdate = true` on the
/// spawned slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FungiSporeSpawn {
    pub x: i32,
    pub y: i32,
    pub vx: f32,
    pub vy: f32,
}

/// The arm's effect requests (the pure module never touches the pools).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style13Out {
    pub cursed_flame: Option<CursedFlameShot>,
    pub fungi_spore: Option<FungiSporeSpawn>,
}

/// The aiStyle-13 arm (`NPC.cs:22615-22847`). One fixed step; the shot/
/// spawn jitters ride the caller's stream on their qualifying ticks.
/// `npc_type` is the VANILLA type (the arm's `this.type` branches);
/// `get_good_world` is `Main.getGoodWorld` (the 43 rows).
pub fn ai_style13(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
    npc_type: i32,
    get_good_world: bool,
    rng: &mut UnifiedRandom,
) -> Style13Out {
    let mut out = Style13Out::default();
    // `:22617-22622` — the anchor tile + the 5-fluff InWorld guard.
    let num211 = npc.ai[0] as i32;
    let num212 = npc.ai[1] as i32;
    if num211 < 5
        || num211 >= world.width as i32 - 5
        || num212 < 5
        || num212 >= world.height as i32 - 5
    {
        return out;
    }
    // `:22623-22633` — the null-tile allocation folds to the inactive
    // read (the port's tiles always exist in range); an inactive
    // anchor kills the NPC (the hp-0 convention, the HitEffect gore is
    // the documented skip).
    let active = world.tile(num211, num212).is_some_and(|t| t.active());
    if !active {
        npc.hp = 0;
        return out;
    }
    // `:22634` — FixExploitManEaters.ProtectSpot: the port has no
    // tile-protection registry (no RNG, no AI-visible state) — skip.
    // `:22635` — TargetClosest (faceTarget default true, aiStyle 13).
    target_closest(npc, world, players, true, 13, ticker);
    // `:22636-22659` — the per-type accel/range pair.
    let mut num213 = 0.035f32;
    let mut num214 = 150f32;
    if npc_type == 43 {
        num214 = if !get_good_world { 250.0 } else { 350.0 };
    }
    if npc_type == 101 {
        num214 = 175.0;
    }
    if npc_type == 259 {
        num214 = 100.0;
    }
    if npc_type == 175 {
        num214 = 500.0;
        num213 = 0.05;
    }
    if npc_type == 260 {
        num214 = 350.0;
        num213 = 0.15;
    }
    // `:22660-22668` — the enrage ladder (the DOUBLE ×1.3 then int).
    npc.ai[2] += 1.0;
    if npc.ai[2] > 300.0 {
        num214 = ((num214 as f64) * 1.3) as i32 as f32;
        if npc.ai[2] > 450.0 {
            npc.ai[2] = 0.0;
        }
    }
    // `:22669-22678` — the anchor centre + the INTEGER-half delta,
    // clamped to the range by the reciprocal-first multiply.
    let (w, h) = npc.size();
    let anchor_x = (num211 * 16 + 8) as f32;
    let anchor_y = (num212 * 16 + 8) as f32;
    let p = &players[npc.target as usize];
    let pcx = p.x + p.width as f32 / 2.0;
    let pcy = p.y + p.height as f32 / 2.0;
    let mut num215 = pcx - (w as i32 / 2) as f32 - anchor_x;
    let mut num216 = pcy - (h as i32 / 2) as f32 - anchor_y;
    let mut num217 = (num215 * num215 + num216 * num216).sqrt();
    if num217 > num214 {
        num217 = num214 / num217;
        num215 *= num217;
        num216 *= num217;
    }
    // `:22679-22694` — the X steering (the 1.5× sign-flip double-add).
    if npc.x < anchor_x + num215 {
        npc.vx += num213;
        if npc.vx < 0.0 && num215 > 0.0 {
            npc.vx += num213 * 1.5;
        }
    } else if npc.x > anchor_x + num215 {
        npc.vx -= num213;
        if npc.vx > 0.0 && num215 < 0.0 {
            npc.vx -= num213 * 1.5;
        }
    }
    // `:22695-22710` — the Y steering.
    if npc.y < anchor_y + num216 {
        npc.vy += num213;
        if npc.vy < 0.0 && num216 > 0.0 {
            npc.vy += num213 * 1.5;
        }
    } else if npc.y > anchor_y + num216 {
        npc.vy -= num213;
        if npc.vy > 0.0 && num216 < 0.0 {
            npc.vy -= num213 * 1.5;
        }
    }
    // `:22711-22733` — the per-type velocity clamps.
    let cap = if npc_type == 43 {
        if get_good_world { 3.5 } else { 3.0 }
    } else if npc_type == 175 {
        4.0
    } else {
        2.0
    };
    npc.vx = npc.vx.clamp(-cap, cap);
    npc.vy = npc.vy.clamp(-cap, cap);
    // `:22734-22750` — the facing/rotation.
    if npc_type == 259 || npc_type == 260 {
        npc.rotation = num216.atan2(num215) + 1.57;
    } else {
        if num215 > 0.0 {
            npc.sprite_dir = 1;
            npc.rotation = num216.atan2(num215);
        }
        if num215 < 0.0 {
            npc.sprite_dir = -1;
            npc.rotation = num216.atan2(num215) + 3.14;
        }
    }
    // `:22751-22763` — the collideX bounce (the strict ±2 floors).
    if npc.collide_x {
        npc.net_update = true;
        npc.vx = npc.old_vx * -0.7;
        if npc.vx > 0.0 && npc.vx < 2.0 {
            npc.vx = 2.0;
        }
        if npc.vx < 0.0 && npc.vx > -2.0 {
            npc.vx = -2.0;
        }
    }
    // `:22764-22776` — the collideY bounce.
    if npc.collide_y {
        npc.net_update = true;
        npc.vy = npc.old_vy * -0.7;
        if npc.vy > 0.0 && npc.vy < 2.0 {
            npc.vy = 2.0;
        }
        if npc.vy < 0.0 && npc.vy > -2.0 {
            npc.vy = -2.0;
        }
    }
    // `:22777-22780` — the client return folds away (single-player).
    // `:22781` — the firing-distance flag (targetPos = the player's
    // centre, shootPos = the NPC's centre — the C# named-arg order).
    let (cx, cy) = npc.center();
    let flag23 = crate::npc_ai_bat::global_firing_distance_check((cx, cy), (pcx, pcy));
    // `:22782-22809` — the Clinger's cursed-flame shot.
    if npc_type == 101 && !p.dead {
        if npc.just_hit {
            npc.local_ai[0] = 0.0;
        }
        npc.local_ai[0] += 1.0;
        if npc.local_ai[0] >= 120.0 {
            // `:22791` — `(!SolidCollision & flag23) && CanHit` (the &
            // is non-short-circuit but flag23 is already computed).
            let solid = crate::player::Player::solid_collision(world, npc.x, npc.y, w, h);
            if (!solid & flag23)
                && crate::projectile::can_hit(
                    world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
                )
            {
                // `:22793-22799` — the 10-speed jittered aim; the X
                // jitter draws BEFORE the Y jitter. The aim reads the
                // player's FLOAT half-width (`position + width * 0.5`).
                let num218 = 10f32;
                let sx = npc.x + w * 0.5;
                let sy = npc.y + h * 0.5;
                let mut ax = p.x + p.width as f32 * 0.5 - sx + rng.next_range(-10, 11) as f32;
                let mut ay = p.y + p.height as f32 * 0.5 - sy + rng.next_range(-10, 11) as f32;
                let k = num218 / (ax * ax + ay * ay).sqrt();
                ax *= k;
                ay *= k;
                // `:22800-22802` — CursedFlame 96, classic damage 22
                // (`GetAttackDamage_ForProjectiles(22f, 17.6f)`, the
                // single-difficulty fold); timeLeft 300 is the
                // caller's write; the counter resets.
                out.cursed_flame = Some(CursedFlameShot { x: sx, y: sy, vx: ax, vy: ay });
                npc.local_ai[0] = 0.0;
            } else {
                // `:22806` — the blocked gate parks at 100.
                npc.local_ai[0] = 100.0;
            }
        }
    }
    // `:22810-22813` — only the 260 with a live player continues.
    if npc_type != 260 || p.dead {
        return out;
    }
    // `:22814-22822` — the spore timer.
    if npc.just_hit {
        npc.local_ai[0] = 0.0;
    }
    npc.local_ai[0] += 1.0;
    if npc.local_ai[0] < 150.0 {
        return out;
    }
    // `:22823-22846` — the spore spawn gate + aim.
    let solid = crate::player::Player::solid_collision(world, npc.x, npc.y, w, h);
    if (!solid & flag23)
        && crate::projectile::can_hit(
            world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
        )
    {
        // `:22825-22836` — the 14-speed aim. The X jitter draws first;
        // the `num221` lift reads the STEERING num216 (the movement
        // block's leftover delta — the verbatim quirk), zeroed when it
        // is positive; the Y jitter draws after.
        let num220 = 14f32;
        let sx = npc.x + w * 0.5;
        let sy = npc.y + h * 0.5;
        let mut ax = p.x + p.width as f32 * 0.5 - sx + rng.next_range(-10, 11) as f32;
        let mut num221 = (ax * 0.1).abs();
        if num216 > 0.0 {
            num221 = 0.0;
        }
        let mut ay = p.y + p.height as f32 * 0.5 - sy + rng.next_range(-10, 11) as f32 - num221;
        let k = num220 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        // `:22837-22841` — NewNPC(261) at the INT centre; the velocity
        // + netUpdate writes ride the request; the counter resets.
        out.fungi_spore = Some(FungiSporeSpawn {
            x: cx as i32,
            y: cy as i32,
            vx: ax,
            vy: ay,
        });
        npc.local_ai[0] = 0.0;
    } else {
        // `:22845` — the blocked gate parks at 130.
        npc.local_ai[0] = 130.0;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::player::Player;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// An open-air world with ONE active anchor tile at (100, 100).
    fn anchor_world() -> World {
        let mut w = World::new("t".into(), 200, 200);
        let i = w.index(100, 100);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    fn player_at(w: &World, x: f32, y: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = x;
        p.y = y;
        p.dead = false;
        p
    }

    fn cand(p: &Player) -> TargetCandidate {
        crate::npc_target::session_candidate(p)
    }

    /// A bare NPC standing in for an unported family member: the box
    /// rides `size_override` (the census row), the anchor rides ai[0/1].
    fn member(row: &Style13Defaults, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::ChaosBall, x, y, 1, 0.0);
        n.size_override = Some((row.width as f32, row.height as f32));
        n.hp = row.life_max;
        n.max_hp = row.life_max;
        n.ai[0] = 100.0;
        n.ai[1] = 100.0;
        n
    }

    /// The census table matches the six SetDefaults rows.
    #[test]
    fn w4_d23_s13_the_family_census_matches_setdefaults() {
        let expect = [
            (43, 30, 30, 34, 10, 110, 0.0f32, 350.0f32, false, 1.0f32),
            (56, 30, 30, 25, 10, 60, 0.0, 90.0, false, 1.0),
            (101, 30, 30, 70, 30, 320, 0.2, 600.0, true, 1.0),
            (175, 30, 30, 100, 30, 300, 0.25, 650.0, false, 1.0),
            (259, 20, 20, 24, 4, 90, 0.0, 350.0, false, 0.3),
            (260, 36, 36, 70, 20, 300, 0.0, 1250.0, false, 0.3),
        ];
        assert_eq!(STYLE13_ROWS.len(), 6);
        for (row, e) in STYLE13_ROWS.iter().zip(expect) {
            assert_eq!(
                (row.npc_type, row.width, row.height, row.damage, row.defense,
                 row.life_max, row.knock_back_resist, row.value, row.behind_tiles, row.npc_slots),
                e,
                "type {} row", row.npc_type
            );
        }
    }

    /// `:22619-22622` — the 5-fluff InWorld guard returns untouched
    /// (no target pick, no timer, no draws).
    #[test]
    fn w4_d23_s13_the_out_of_world_anchor_returns_draw_free() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[0], 1600.0, 1600.0);
        n.ai[0] = 4.0; // x < fluff 5
        let mut rng = UnifiedRandom::new(1);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 43, false, &mut rng);
        assert_eq!(out, Style13Out::default());
        assert_eq!(n.target, 255, "no TargetClosest ran");
        assert_eq!(n.ai[2], 0.0, "no timer tick");
        assert_eq!(rng, UnifiedRandom::new(1));
    }

    /// `:22627-22633` — an inactive anchor tile kills the NPC (the
    /// hp-0 fold of `life = -1; HitEffect(); active = false`).
    #[test]
    fn w4_d23_s13_the_inactive_anchor_kills_draw_free() {
        let mut w = anchor_world();
        let i = w.index(100, 100);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[1], 1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(2);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 56, false, &mut rng);
        assert_eq!(out, Style13Out::default());
        assert_eq!(n.hp, 0, "the anchor death");
        assert_eq!(n.target, 255, "the death returns before TargetClosest");
        assert_eq!(rng, UnifiedRandom::new(2));
    }

    /// `:22636-22750` — the ManEater steering: the 0.035 accel toward
    /// the anchor+delta on both axes, the ±3 clamp, the right-face
    /// atan2 rotation, the ai[2] tick. Draw-free (no shot arm).
    #[test]
    fn w4_d23_s13_the_maneater_steers_draw_free() {
        let w = anchor_world();
        // anchor centre (1608, 1608); player centre (1710, 1621).
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[0], 1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(3);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 43, false, &mut rng);
        assert_eq!(out, Style13Out::default());
        assert_eq!(n.target, 0);
        assert_eq!(n.ai[2], 1.0, "the timer tick (:22660)");
        // num215 = 1710 - 15 - 1608 = 87; num216 = 1621 - 15 - 1608 = -2.
        assert!((n.vx - 0.035).abs() < 1e-6, "the X accel (:22681)");
        assert!((n.vy - 0.035).abs() < 1e-6, "the Y accel (:22697)");
        assert_eq!(n.sprite_dir, 1, "the right face (:22742)");
        assert!((n.rotation - (-2f32).atan2(87f32)).abs() < 1e-6, "the atan2 (:22743)");
        assert_eq!(rng, UnifiedRandom::new(3));
    }

    /// `:22682-22685` — the sign-flip double-add: a leftward velocity
    /// steering right gets 0.035 + 0.035*1.5 in one tick.
    #[test]
    fn w4_d23_s13_the_sign_flip_double_add() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[0], 1600.0, 1600.0);
        n.vx = -1.0;
        let mut rng = UnifiedRandom::new(4);
        ai_style13(&mut n, &w, &[cand(&p)], 0, 43, false, &mut rng);
        let expect = -1.0 + 0.035 + 0.035 * 1.5;
        assert!((n.vx - expect).abs() < 1e-6, "the 1.5x double-add");
        assert_eq!(rng, UnifiedRandom::new(4));
    }

    /// `:22660-22668` — the enrage ladder: 300 -> 301 keeps the counter
    /// (the range scales ×1.3 int-truncated); 450 -> 451 resets to 0.
    #[test]
    fn w4_d23_s13_the_enrage_ladder_ticks_and_resets() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[1], 1600.0, 1600.0);
        n.ai[2] = 300.0;
        let mut rng = UnifiedRandom::new(5);
        ai_style13(&mut n, &w, &[cand(&p)], 0, 56, false, &mut rng);
        assert_eq!(n.ai[2], 301.0, "past 300 the counter keeps climbing");
        n.ai[2] = 450.0;
        ai_style13(&mut n, &w, &[cand(&p)], 0, 56, false, &mut rng);
        assert_eq!(n.ai[2], 0.0, "past 450 the reset (:22666)");
        assert_eq!(rng, UnifiedRandom::new(5));
    }

    /// `:22751-22763` — the collideX bounce: oldVelocity × -0.7 with
    /// the strict ±2 floor, netUpdate. Draw-free.
    #[test]
    fn w4_d23_s13_the_collide_bounce_floors_at_two() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[1], 1600.0, 1600.0);
        n.collide_x = true;
        n.old_vx = 1.0; // ×-0.7 = -0.7 -> the -2 floor
        let mut rng = UnifiedRandom::new(6);
        ai_style13(&mut n, &w, &[cand(&p)], 0, 56, false, &mut rng);
        assert_eq!(n.vx, -2.0, "the strict floor (:22759-22762)");
        assert!(n.net_update, "the bounce nets (:22753)");
        // A strong bounce keeps the ×-0.7 value (no floor).
        n.collide_x = true;
        n.old_vx = 10.0;
        ai_style13(&mut n, &w, &[cand(&p)], 0, 56, false, &mut rng);
        assert!((n.vx - (-7.0)).abs() < 1e-5, "the -0.7 bounce (:22754)");
        assert_eq!(rng, UnifiedRandom::new(6));
    }

    /// `:22734-22737` — the fungi bulbs rotate +1.57f and never write
    /// spriteDirection; `:22745-22749` — the others face left +3.14f.
    #[test]
    fn w4_d23_s13_the_bulb_rotation_and_the_left_face() {
        let w = anchor_world();
        // The player LEFT of the anchor: num215 < 0.
        let p = player_at(&w, 1400.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[4], 1600.0, 1600.0);
        n.sprite_dir = 7; // a sentinel the bulb arm must not touch
        let mut rng = UnifiedRandom::new(7);
        ai_style13(&mut n, &w, &[cand(&p)], 0, 259, false, &mut rng);
        // num215 = 1410 - 10 - 1608 = -208; num216 = 1621 - 10 - 1608 = 3.
        // dist > 100 (the 259 range) -> the clamp scales both.
        let k = 100f32 / ((-208f32) * (-208.0) + 3.0 * 3.0).sqrt();
        let expect = (3.0 * k).atan2(-208.0 * k) + 1.57;
        assert!((n.rotation - expect).abs() < 1e-5, "the +1.57 bulb rotation");
        assert_eq!(n.sprite_dir, 7, "the bulb arm writes no spriteDirection");
        // The ManEater faces left with the +3.14 fold.
        let mut m = member(&STYLE13_ROWS[0], 1600.0, 1600.0);
        ai_style13(&mut m, &w, &[cand(&p)], 0, 43, false, &mut rng);
        assert_eq!(m.sprite_dir, -1, "the left face (:22747)");
        // num215 = 1410 - 15 - 1608 = -213; num216 = 1621 - 15 - 1608 = -2.
        let expect = (-2f32).atan2(-213f32) + 3.14;
        assert!((m.rotation - expect).abs() < 1e-5, "the +3.14 left rotation");
        assert_eq!(rng, UnifiedRandom::new(7));
    }

    /// `:22782-22809` — the Clinger shot RNG census: at localAI[0] 119
    /// the ++ reaches 120, the open-air gate passes, and EXACTLY two
    /// `Next(-10, 11)` draws fire (X then Y). The manual replay on a
    /// cloned stream reproduces the 10-speed aim bit-for-bit; the
    /// counter resets; the shot is CursedFlame at the float centre.
    #[test]
    fn w4_d23_s13_the_clinger_shot_draws_x_then_y() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[2], 1650.0, 1600.0);
        n.local_ai[0] = 119.0;
        let mut rng = UnifiedRandom::new(42);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 101, false, &mut rng);
        let shot = out.cursed_flame.expect("the shot fired");
        assert_eq!(n.local_ai[0], 0.0, "the shot resets (:22802)");
        // The manual replay in the C#'s draw order.
        let mut replay = UnifiedRandom::new(42);
        let sx = 1650.0 + 30.0 * 0.5;
        let sy = 1600.0 + 30.0 * 0.5;
        let mut ax = 1700.0 + 20.0 * 0.5 - sx + replay.next_range(-10, 11) as f32;
        let mut ay = 1600.0 + 42.0 * 0.5 - sy + replay.next_range(-10, 11) as f32;
        let k = 10.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        assert_eq!(shot, CursedFlameShot { x: sx, y: sy, vx: ax, vy: ay });
        assert_eq!(rng, replay, "exactly the two jitters, in C# order");
    }

    /// `:22791`/`:22806` — a solid-body Clinger parks at 100 and draws
    /// NOTHING; `:22784-22787` — justHit zeroes the counter first.
    #[test]
    fn w4_d23_s13_the_blocked_clinger_parks_at_100_draw_free() {
        let mut w = anchor_world();
        // A solid floor under the NPC body (the anchor tile stays live).
        for x in 95..115 {
            for y in 100..110 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[2], 1600.0, 1600.0);
        n.local_ai[0] = 119.0;
        n.just_hit = true;
        let mut rng = UnifiedRandom::new(8);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 101, false, &mut rng);
        assert_eq!(out.cursed_flame, None, "the solid gate blocks");
        // justHit zeroed, ++ -> 1 (< 120): the shot arm never runs.
        assert_eq!(n.local_ai[0], 1.0, "the justHit reset (:22786)");
        assert_eq!(rng, UnifiedRandom::new(8));
        // Without justHit the counter reaches 120 and parks at 100.
        n.local_ai[0] = 119.0;
        n.just_hit = false;
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 101, false, &mut rng);
        assert_eq!(out.cursed_flame, None);
        assert_eq!(n.local_ai[0], 100.0, "the blocked park (:22806)");
        assert_eq!(rng, UnifiedRandom::new(8), "the blocked gate draws nothing");
    }

    /// `:22823-22841` — the 260 spore RNG census: at localAI[0] 149 the
    /// ++ reaches 150, the gate passes, the X jitter draws, the num221
    /// lift reads the STEERING delta (negative here -> the |aimX*0.1|
    /// lift stands), the Y jitter draws, the spore request carries the
    /// INT centre + the 14-speed aim. Exactly two draws.
    #[test]
    fn w4_d23_s13_the_bulb_spore_draws_x_then_y_with_the_lift() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut n = member(&STYLE13_ROWS[5], 1650.0, 1600.0);
        n.local_ai[0] = 149.0;
        let mut rng = UnifiedRandom::new(43);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 260, false, &mut rng);
        let spore = out.fungi_spore.expect("the spore spawned");
        assert_eq!(n.local_ai[0], 0.0, "the spawn resets (:22841)");
        let mut replay = UnifiedRandom::new(43);
        let sx = 1650.0 + 36.0 * 0.5;
        let sy = 1600.0 + 36.0 * 0.5;
        let mut ax = 1700.0 + 20.0 * 0.5 - sx + replay.next_range(-10, 11) as f32;
        // The steering num216 = 1621 - 18 - 1608 = -5 (<= 0): the lift
        // stands at |aimX * 0.1| (:22828-22832).
        let lift = (ax * 0.1).abs();
        let mut ay = 1600.0 + 42.0 * 0.5 - sy + replay.next_range(-10, 11) as f32 - lift;
        let k = 14.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        assert_eq!(spore, FungiSporeSpawn { x: sx as i32, y: sy as i32, vx: ax, vy: ay });
        assert_eq!(rng, replay, "exactly the two jitters, in C# order");
    }

    /// `:22829-22832` — the num221 quirk: a player BELOW the anchor
    /// makes the steering num216 positive, zeroing the lift; and
    /// `:22845` — the blocked gate parks at 130 draw-free.
    #[test]
    fn w4_d23_s13_the_positive_steering_zeroes_the_lift() {
        let w = anchor_world();
        // Player centre Y 1721 -> steering num216 = 1721-18-1608 = 95 > 0.
        let p = player_at(&w, 1700.0, 1700.0);
        let mut n = member(&STYLE13_ROWS[5], 1650.0, 1600.0);
        n.local_ai[0] = 149.0;
        let mut rng = UnifiedRandom::new(44);
        let out = ai_style13(&mut n, &w, &[cand(&p)], 0, 260, false, &mut rng);
        let spore = out.fungi_spore.expect("the spore spawned");
        let mut replay = UnifiedRandom::new(44);
        let sx = 1650.0 + 36.0 * 0.5;
        let sy = 1600.0 + 36.0 * 0.5;
        let mut ax = 1700.0 + 20.0 * 0.5 - sx + replay.next_range(-10, 11) as f32;
        // num216 > 0 -> num221 = 0 (no lift on the Y aim).
        let mut ay = 1700.0 + 42.0 * 0.5 - sy + replay.next_range(-10, 11) as f32;
        let k = 14.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        assert_eq!(spore, FungiSporeSpawn { x: sx as i32, y: sy as i32, vx: ax, vy: ay });
        assert_eq!(rng, replay);
        // A dead player returns before the spore arm (:22810-22813).
        let mut dead = player_at(&w, 1700.0, 1700.0);
        dead.dead = true;
        n.local_ai[0] = 149.0;
        let out = ai_style13(&mut n, &w, &[cand(&dead)], 0, 260, false, &mut rng);
        assert_eq!(out.fungi_spore, None, "the dead-player return");
        assert_eq!(n.local_ai[0], 149.0, "no timer tick past the return");
        assert_eq!(rng, replay, "the dead return draws nothing");
    }

    /// The plain members (43/56/175/259) draw NOTHING on any tick —
    /// the shot/spawn arms are type-gated; 100 mixed ticks keep the
    /// stream byte-identical.
    #[test]
    fn w4_d23_s13_the_plain_members_are_draw_free() {
        let w = anchor_world();
        let p = player_at(&w, 1700.0, 1600.0);
        let mut rng = UnifiedRandom::new(123);
        for (row, ty) in [(0, 43), (1, 56), (3, 175), (4, 259)] {
            let mut n = member(&STYLE13_ROWS[row], 1600.0, 1600.0);
            for _ in 0..100 {
                let out = ai_style13(&mut n, &w, &[cand(&p)], 0, ty, false, &mut rng);
                assert_eq!(out, Style13Out::default());
            }
        }
        assert_eq!(rng, UnifiedRandom::new(123), "400 ticks, zero draws");
    }
}
