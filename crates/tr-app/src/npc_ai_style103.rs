//! The aiStyle-103 family — the Sand Sharks (`NPC.AI()`'s
//! `else if (this.aiStyle == 103)` arm, `NPC.cs:40543-40723`). W4-d33,
//! research/25 row 103, CODE-ONLY MODE (no compiler invoked; the
//! witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has FOUR vanilla members: the SandShark 542 and its
//! Corrupt 543 / Crimson 544 / Hallow 545 variants
//! (`ID/NPCID.cs:11761-11767`; `SetDefaults` rows
//! `NPC.cs:15912-15971`). The census of `this.aiStyle = 103;` across the
//! decompiled tree returns exactly those four hits (`:15917`, `:15932`,
//! `:15947`, `:15962`) — no projectile-space user. The four rows are
//! identical except damage/defense/lifeMax/knockBackResist: all 100×24,
//! noGravity, behindTiles, value 400, NPCHit1/NPCDeath1. The port's
//! roster has no sand-shark kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent), keyed on the vanilla `npc_type`
//! until the roster wave. The W4-d33 wiring note (f42's roster-wave
//! routing): when the kinds land, the axis' 103 arm feeds `cs_id()`
//! to `style103_step` — the ai_style AXIS dispatch, NOT a kind match
//! (the 112/113/114 arms' precedent).
//!
//! ## The machine
//!
//! The sand swimmer: inside sand (or wet) it burrows and lunges;
//! outside it falls like a fish. The sand gate (`:40550-40551`):
//! `flag120 = (Center tile nactive && (Conversion.Sand || Sandstone ||
//! HardenedSand)) | wet` — the NON-short-circuit `|` (both sides read).
//! `TargetClosest(faceTarget: false)` (`:40553`) refreshes the target
//! rect WITHOUT facing; `vector221` is that rect's centre (the XNA
//! `Rectangle.Center` — INTEGER half-extents). `flag121` (the chase
//! gate, `:40555-40558`): the target's `velocity.Y > -0.1`, not dead,
//! and `Distance > 150`.
//!
//! The surfacing cooldown (`:40559-40566`): `localAI[0] == -1` (the
//! lunge marker) off-sand re-arms to 20; any positive value counts
//! down.
//!
//! In sand (`:40567-40691`): the swim cadence (`:40569-40582`) —
//! `soundDelay == 0` sets it to `Clamp(Distance/40, 10, 20)` truncated
//! and plays `PlaySound(15, Center, 4)` (the audio seam). The dead
//! `num1409 = ai[1]` (`:40583`) is computed and never read — documented.
//! `flag122` (`:40585-40590`): the tile at `(Center + (0,24))` shifted
//! TWO rows up is sand-family; `ai[1] = flag122` (the render layer's
//! submerged flag). `ai[2] < 30` counts up (`:40592-40595`).
//!
//! The chase arm (`flag121`, `:40596-40638`): `TargetClosest()` (faces),
//! `velocity += (direction, directionY) * 0.15`, caps ±5 X / ±3 Y. The
//! lookahead tile (`:40617`): `Center + SafeNormalize(v) *
//! Size.Length()/2 + v` — sand-family OR (wet AND liquid > 0) blocks
//! the lunge. The lunge gate (`:40628`): not blocked, `Sign(vx) ==
//! direction`, `Distance < 400` (700 on `Main.remixWorld` — the port
//! runs false, the env seam), and `ai[2] >= 30 || ai[2] < 0`. The
//! lunge (`:40630-40636`): at `localAI[0] == 0` play `PlaySound(14,
//! Center, 542)` and mark `localAI[0] = -1`; `ai[2] = -30`;
//! `velocity = DirectionTo(vector221 + (0,-80)) * 12` — the 80-px-high
//! leap aim, speed 12.
//!
//! The cruise arm (`!flag121`, `:40639-40690`): collideX flips vx AND
//! direction (+netUpdate); collideY flips vy, `directionY =
//! Sign(vy)`, `ai[0] = directionY` (+netUpdate) — then the bob OVERWRITES
//! `ai[0]` from `flag122` (`:40660-40667`: -1 in sand above, +1 below;
//! the collideY write survives only into the same tick's bob input —
//! verbatim). `vx += direction * 0.1`, the ±6 band damps ×0.95. The Y
//! bob (`:40668-40689`): `ai[0] == -1` → `vy -= 0.01`, below -0.06
//! flips `ai[0] = 1`; else `vy += 0.01`, above 0.06 flips `ai[0] = -1`;
//! the ±0.4 band damps ×0.95.
//!
//! Off sand (`:40692-40713`): `vy == 0` (on ground) runs the ±1-band
//! X crawl (`flag121` retargets first); then `vy += 0.3` capped 10 and
//! `ai[0] = 1` — the fall.
//!
//! The tilt tail (`:40714-40722`): `rotation = vy * direction * 0.1`
//! clamped ±0.2.
//!
//! ## RNG order (by construction)
//!
//! The arm draws NOTHING: `TargetClosest` is RNG-free
//! (`NPC.cs:77652-77781`), the tile probes and the velocity maths draw
//! nothing, and the two `PlaySound` cues ride the audio seam (the
//! style75 `sounds` convention — the int overloads 15/4 and 14/542 are
//! recorded, the pitch draws belong to the sound family). The witness
//! below pins the untouched stream.
//!
//! ## FindFrame cases 542-545
//!
//! `NPC.cs:74681-74692`: `spriteDirection = direction`; the counter
//! increments, wraps at 16, and the row is `(int)(counter / 4)` over
//! the 4-row sheet (`Main.npcFrameCount[542..545] = 4`, `Main.cs:67138`
//! — the 540-549 row `7, 13, 4, 4, 4, 4, …`). [`shark_find_frame`]
//! ports it as the pure pair function (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::bow_launch::safe_normalize;
use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::tiledata::{
    SET_CONVERSION_HARDENED_SAND, SET_CONVERSION_SAND, SET_CONVERSION_SANDSTONE,
};
use tr_core::World;

/// One of the four type-542..545 `SetDefaults` rows
/// (`NPC.cs:15912-15971`), verbatim. All four share the box, sounds,
/// value and flags; only the four stat fields differ.
pub struct SandSharkDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1` (`SoundID.cs:1257`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1` (`SoundID.cs:1315`).
    pub death_sound: (i32, i32),
    /// `this.value = 400f` — the C# float field.
    pub value: f32,
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub behind_tiles: bool,
}

/// `NPC.cs:15912-15971` — the four rows in type order.
pub const SAND_SHARKS: [SandSharkDefaults; 4] = [
    SandSharkDefaults {
        ty: 542,
        width: 100,
        height: 24,
        ai_style: 103,
        damage: 50,
        defense: 20,
        life_max: 360,
        hit_sound: (3, 1),
        death_sound: (4, 1),
        value: 400.0,
        knock_back_resist: 0.9,
        no_gravity: true,
        behind_tiles: true,
    },
    SandSharkDefaults {
        ty: 543,
        width: 100,
        height: 24,
        ai_style: 103,
        damage: 60,
        defense: 24,
        life_max: 380,
        hit_sound: (3, 1),
        death_sound: (4, 1),
        value: 400.0,
        knock_back_resist: 0.8,
        no_gravity: true,
        behind_tiles: true,
    },
    SandSharkDefaults {
        ty: 544,
        width: 100,
        height: 24,
        ai_style: 103,
        damage: 64,
        defense: 22,
        life_max: 400,
        hit_sound: (3, 1),
        death_sound: (4, 1),
        value: 400.0,
        knock_back_resist: 0.8,
        no_gravity: true,
        behind_tiles: true,
    },
    SandSharkDefaults {
        ty: 545,
        width: 100,
        height: 24,
        ai_style: 103,
        damage: 54,
        defense: 26,
        life_max: 450,
        hit_sound: (3, 1),
        death_sound: (4, 1),
        value: 400.0,
        knock_back_resist: 0.7,
        no_gravity: true,
        behind_tiles: true,
    },
];

/// `Main.npcFrameCount[542..=545]` = 4 (`Main.cs:67138`, the 540-549
/// row: `7, 13, 4, 4, 4, 4, …`).
pub const SAND_SHARK_FRAME_COUNT: u32 = 4;

/// The per-tick environment (the `Style102Env` shape).
pub struct Style103Env<'a> {
    pub world: &'a World,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.player[j].velocity.Y` per slot — the chase gate reads it
    /// (`:40555`); the parallel-slice seam (the style102 precedent).
    pub players_vy: &'a [f32],
    /// `Main.remixWorld` — the lunge range 400 → 700 (`:40624-40627`).
    /// The port has no remix flag; the wiring passes false.
    pub remix_world: bool,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style103Outcome {
    /// The `SoundEngine.PlaySound(type, Center, style)` int-overload
    /// cues in C# order — `(type, style, x, y)` (the style75 `sounds`
    /// seam widened with the type): the swim cadence `(15, 4)`
    /// (`:40581`) and the lunge cry `(14, 542)` (`:40632`). The audio
    /// family owns the pitch draws.
    pub sounds: Vec<(i32, i32, f32, f32)>,
}

/// `Math.Sign(float)` — -1/0/+1, the 0-on-tie (`npc.rs::math_sign`).
fn math_sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// XNA `Vector2.Distance(a, b)` — the f32 squared sum widened to f64
/// for the root (the style100 `sqrt_f32` shape).
fn distance(a: Vec2, b: Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// `Framing.GetTileSafely(x, y)` (`Framing.cs:426-436`) — out-of-world
/// or null reads an empty tile: the port's `None`. The sand-family
/// test: `nactive() && (Conversion.Sand || Sandstone || HardenedSand)`
/// (`:40551`).
fn sand_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && (SET_CONVERSION_SAND.has(t.kind)
                || SET_CONVERSION_SANDSTONE.has(t.kind)
                || SET_CONVERSION_HARDENED_SAND.has(t.kind))
    })
}

/// `ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4`.
fn tile_of(v: Vec2) -> (i32, i32) {
    (v.x as i32 >> 4, v.y as i32 >> 4)
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

/// One fixed step of the aiStyle-103 arm (`NPC.cs:40543-40723`), in C#
/// order. The caller owns the movement integration after `AI()` (the
/// rows' `noGravity` is true — the arm's own gravity is the off-sand
/// fall) and drains [`Style103Outcome`].
pub fn style103_step(npc: &mut Npc, env: &Style103Env<'_>) -> Style103Outcome {
    let mut out = Style103Outcome::default();
    let (w, h) = npc.size();
    // `:40545-40548` — the unset facing picks a target first.
    if npc.facing == 0 && !env.players.is_empty() {
        target_closest(npc, env.world, env.players, true, 103, env.ticker);
    }
    // `:40549-40551` — the sand gate. The C# `|` is NON-short-circuit;
    // both sides are pure reads, so the port's `||` is equivalent.
    let (cx, cy) = (npc.x + w / 2.0, npc.y + h / 2.0);
    let (tx, ty) = tile_of(Vec2::new(cx, cy));
    let in_sand = sand_at(env.world, tx, ty) || npc.wet; // flag120
                                                         // `:40553` — the no-face retarget (always runs).
    if !env.players.is_empty() {
        target_closest(npc, env.world, env.players, false, 103, env.ticker);
    }
    // `:40554` — targetRect.Center: the XNA INTEGER half-extents.
    let r = npc.target_rect;
    let aim = Vec2::new((r.0 + r.2 / 2) as f32, (r.1 + r.3 / 2) as f32);
    // `:40555-40558` — the chase gate.
    let p = target_of(npc, env.players);
    let pvy = if npc.target >= 0 && (npc.target as usize) < env.players_vy.len() {
        env.players_vy[npc.target as usize]
    } else {
        env.players_vy.first().copied().unwrap_or(0.0)
    };
    let dist = distance(Vec2::new(cx, cy), aim);
    let chase = pvy > -0.1 && !p.dead && dist > 150.0; // flag121
                                                       // `:40559-40566` — the surfacing cooldown.
    if npc.local_ai[0] == -1.0 && !in_sand {
        npc.local_ai[0] = 20.0;
    }
    if npc.local_ai[0] > 0.0 {
        npc.local_ai[0] -= 1.0;
    }
    if in_sand {
        // `:40569-40582` — the swim cadence.
        if npc.sound_delay == 0 {
            let mut cadence = dist / 40.0; // num1408
            if cadence < 10.0 {
                cadence = 10.0;
            }
            if cadence > 20.0 {
                cadence = 20.0;
            }
            npc.sound_delay = cadence as i32;
            out.sounds.push((15, 4, cx, cy));
        }
        // `:40583` — num1409 = ai[1] is computed and NEVER read (the
        // documented dead local).
        // `:40585-40591` — the submerged flag: (Center + (0,24)) two
        // rows up.
        let (px, py) = tile_of(Vec2::new(cx, cy + 24.0));
        let sand_above = sand_at(env.world, px, py - 2); // flag122
        npc.ai[1] = if sand_above { 1.0 } else { 0.0 };
        // `:40592-40595` — the lunge charge.
        if npc.ai[2] < 30.0 {
            npc.ai[2] += 1.0;
        }
        if chase {
            // `:40596-40616` — the chase accel + caps.
            if !env.players.is_empty() {
                target_closest(npc, env.world, env.players, true, 103, env.ticker);
            }
            npc.vx += npc.facing as f32 * 0.15;
            npc.vy += npc.direction_y as f32 * 0.15;
            if npc.vx > 5.0 {
                npc.vx = 5.0;
            }
            if npc.vx < -5.0 {
                npc.vx = -5.0;
            }
            if npc.vy > 3.0 {
                npc.vy = 3.0;
            }
            if npc.vy < -3.0 {
                npc.vy = -3.0;
            }
            // `:40617-40622` — the lookahead: Center + unit(v) *
            // Size.Length()/2 + v. Size.Length() is XNA's f64 root of
            // the f32 sum.
            let v = Vec2::new(npc.vx, npc.vy);
            let size_len = ((w * w + h * h) as f64).sqrt() as f32;
            let ahead = Vec2::new(cx, cy) + safe_normalize(v, Vec2::ZERO) * (size_len / 2.0) + v;
            let (ax, ay) = tile_of(ahead);
            let mut blocked = sand_at(env.world, ax, ay); // flag123
            if !blocked && npc.wet {
                blocked = env.world.tile(ax, ay).is_some_and(|t| t.liquid > 0);
            }
            // `:40623-40627` — the lunge range (the remix seam).
            let range = if env.remix_world { 700.0 } else { 400.0 };
            // `:40628-40637` — the lunge.
            if !blocked
                && math_sign(npc.vx) == npc.facing
                && dist < range
                && (npc.ai[2] >= 30.0 || npc.ai[2] < 0.0)
            {
                if npc.local_ai[0] == 0.0 {
                    out.sounds.push((14, 542, cx, cy));
                    npc.local_ai[0] = -1.0;
                }
                npc.ai[2] = -30.0;
                // DirectionTo(aim + (0,-80)) * 12 — the unguarded XNA
                // normalize (the style99 convention); the aim is 150+
                // px away by the chase gate, never coincident.
                let d = aim + Vec2::new(0.0, -80.0) - Vec2::new(cx, cy);
                let n = 1.0 / ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32;
                npc.vx = d.x * n * 12.0;
                npc.vy = d.y * n * 12.0;
            }
        } else {
            // `:40641-40646` — the collideX flip.
            if npc.collide_x {
                npc.vx *= -1.0;
                npc.facing *= -1;
                npc.net_update = true;
            }
            // `:40647-40653` — the collideY flip (the ai[0] write is
            // overwritten by the bob below — verbatim).
            if npc.collide_y {
                npc.net_update = true;
                npc.vy *= -1.0;
                npc.direction_y = math_sign(npc.vy);
                npc.ai[0] = npc.direction_y as f32;
            }
            // `:40654-40659` — the X cruise + the ±6 damp.
            let cruise_cap = 6.0; // num1411
            npc.vx += npc.facing as f32 * 0.1;
            if npc.vx < 0.0 - cruise_cap || npc.vx > cruise_cap {
                npc.vx *= 0.95;
            }
            // `:40660-40667` — the bob direction from the sand above.
            if sand_above {
                npc.ai[0] = -1.0;
            } else {
                npc.ai[0] = 1.0;
            }
            // `:40668-40685` — the Y bob.
            let bob_band = 0.06; // num1412
            let bob_accel = 0.01; // num1413
            if npc.ai[0] == -1.0 {
                npc.vy -= bob_accel;
                if npc.vy < 0.0 - bob_band {
                    npc.ai[0] = 1.0;
                }
            } else {
                npc.vy += bob_accel;
                if npc.vy > bob_band {
                    npc.ai[0] = -1.0;
                }
            }
            // `:40686-40689` — the ±0.4 damp.
            if npc.vy > 0.4 || npc.vy < -0.4 {
                npc.vy *= 0.95;
            }
        }
    } else {
        // `:40694-40706` — the grounded crawl.
        if npc.vy == 0.0 {
            if chase && !env.players.is_empty() {
                target_closest(npc, env.world, env.players, true, 103, env.ticker);
            }
            let crawl_cap = 1.0; // num1414
            npc.vx += npc.facing as f32 * 0.1;
            if npc.vx < 0.0 - crawl_cap || npc.vx > crawl_cap {
                npc.vx *= 0.95;
            }
        }
        // `:40707-40712` — the fall.
        npc.vy += 0.3;
        if npc.vy > 10.0 {
            npc.vy = 10.0;
        }
        npc.ai[0] = 1.0;
    }
    // `:40714-40722` — the tilt tail.
    npc.rotation = npc.vy * npc.facing as f32 * 0.1;
    if npc.rotation < -0.2 {
        npc.rotation = -0.2;
    }
    if npc.rotation > 0.2 {
        npc.rotation = 0.2;
    }
    out
}

/// `NPC.FindFrame` cases 542-545 (`NPC.cs:74681-74692`) — one tick, as
/// a pure pair function over `(frame_row, frameCounter)` (the style73
/// convention). The counter wraps at 16; the row is `(int)(counter/4)`
/// over the 4-row sheet. The `spriteDirection = direction` write is
/// the caller's (the port's `Npc.sprite_dir`).
pub fn shark_find_frame(_frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0; // `this.frameCounter++`
    if counter >= 16.0 {
        counter = 0.0;
    }
    let row = (counter / 4.0) as u32;
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::UnifiedRandom;

    /// A 542-shaped NPC (the style73 placeholder convention).
    fn shark(x: f32, y: f32) -> Npc {
        let d = &SAND_SHARKS[0];
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.behind_tiles = d.behind_tiles;
        n.facing = 1;
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

    /// A world whose every cell is sand (tile 53 — Conversion.Sand).
    fn desert() -> World {
        let mut w = World::new("t".into(), 400, 300);
        for x in 0..w.width {
            for y in 0..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 53,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// `NPC.cs:15912-15971` — the four rows, field by field.
    #[test]
    fn w4_d33_s103_the_defaults_rows() {
        let stats = [
            (542, 50, 20, 360, 0.9f32),
            (543, 60, 24, 380, 0.8),
            (544, 64, 22, 400, 0.8),
            (545, 54, 26, 450, 0.7),
        ];
        for (d, &(ty, dmg, def, life, kb)) in SAND_SHARKS.iter().zip(stats) {
            assert_eq!(d.ty, ty);
            assert_eq!((d.width, d.height), (100, 24));
            assert_eq!(d.ai_style, 103);
            assert_eq!((d.damage, d.defense, d.life_max), (dmg, def, life));
            assert_eq!(d.hit_sound, (3, 1));
            assert_eq!(d.death_sound, (4, 1));
            assert_eq!(d.value, 400.0);
            assert_eq!(d.knock_back_resist, kb);
            assert!(d.no_gravity && d.behind_tiles);
        }
        assert_eq!(SAND_SHARK_FRAME_COUNT, 4);
    }

    /// `:40550-40551` + `:40692-40713` — the sand gate splits the
    /// machine: in sand the swim cadence fires (soundDelay 0 → the
    /// clamped Distance/40 + the (15,4) cue); off sand the 0.3 fall
    /// runs and ai[0] = 1.
    #[test]
    fn w4_d33_s103_the_sand_gate_splits_the_machine() {
        let players = [cand(1000.0, 100.0)];
        let vy = [0.0f32];
        // In sand: the whole world is sand.
        let w = desert();
        let mut n = shark(100.0, 100.0);
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let out = style103_step(&mut n, &env);
        assert!(
            out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (15, 4)),
            "the swim cue"
        );
        assert!(
            n.sound_delay >= 10 && n.sound_delay <= 20,
            "the clamped cadence"
        );
        // Off sand: the fall.
        let w = air();
        let mut n = shark(100.0, 100.0);
        n.vy = 0.0;
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let out = style103_step(&mut n, &env);
        assert!(out.sounds.is_empty(), "no swim cue off sand");
        assert_eq!(n.vy, 0.3);
        assert_eq!(n.ai[0], 1.0);
        // The grounded crawl ran first: vx 0 → 0.1 (inside the ±1 band).
        assert_eq!(n.vx, 0.1);
    }

    /// `:40555-40558` — the chase gate: the target's vy > -0.1, alive,
    /// Distance > 150. A falling target (vy -1) or a near one (≤150)
    /// cruises instead.
    #[test]
    fn w4_d33_s103_the_chase_gate() {
        let w = desert();
        let vy = [0.0f32];
        // Far + steady → the chase accel 0.15 (direction 1).
        let players = [cand(1000.0, 100.0)];
        let mut n = shark(100.0, 100.0);
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.vx, 0.15, "the chase accel");
        // Falling target → the cruise accel 0.1 + the bob.
        let vy_fall = [-1.0f32];
        let mut n = shark(100.0, 100.0);
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy_fall,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.vx, 0.1, "the cruise accel");
        // Near target (the rect centre within 150) → cruise too.
        let players = [cand(180.0, 100.0)];
        let mut n = shark(100.0, 100.0);
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.vx, 0.1, "the near cruise");
    }

    /// `:40628-40637` — the lunge: charged (ai[2] ≥ 30), aimed with the
    /// sign, in range, unblocked (the lookahead tile is air) → the
    /// speed-12 leap at aim+(0,-80), ai[2] = -30, the (14,542) cry and
    /// the -1 marker. The desert world blocks it (the lookahead is
    /// sand); the air pocket ahead opens it.
    #[test]
    fn w4_d33_s103_the_lunge() {
        let mut w = desert();
        // Carve the lookahead pocket: the shark at (100,100), centre
        // (150,112), vx 5 → ahead ≈ centre + unit*51.5 + (5,0) →
        // tile ~(16, 7). Clear a generous pocket around it.
        for x in 12..24 {
            for y in 2..14 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        let players = [cand(400.0, 100.0)]; // ~260 px: chase (>150) AND lunge (<400)
        let vy = [0.0f32];
        let mut n = shark(100.0, 100.0);
        n.ai[2] = 30.0;
        n.vx = 5.0;
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let out = style103_step(&mut n, &env);
        assert_eq!(n.ai[2], -30.0, "the lunge reset");
        assert_eq!(n.local_ai[0], -1.0, "the lunge marker");
        assert!(
            out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (14, 542)),
            "the cry"
        );
        let len = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((len - 12.0).abs() < 1e-3, "speed 12, got {len}");
        assert!(n.vx > 0.0, "aimed at the far target");
        // The full desert blocks the lunge (the lookahead is sand).
        let w = desert();
        let mut n = shark(100.0, 100.0);
        n.ai[2] = 30.0;
        n.vx = 5.0;
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let out = style103_step(&mut n, &env);
        assert_eq!(n.ai[2], 30.0, "no reset when blocked");
        assert!(!out.sounds.iter().any(|&(t, s, _, _)| (t, s) == (14, 542)));
    }

    /// `:40559-40566` — the surfacing cooldown: the -1 marker off sand
    /// re-arms to 20, then counts down each tick; in sand it stays.
    #[test]
    fn w4_d33_s103_the_surfacing_cooldown() {
        let players = [cand(1000.0, 100.0)];
        let vy = [0.0f32];
        let w = air();
        let mut n = shark(100.0, 100.0);
        n.local_ai[0] = -1.0;
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.local_ai[0], 19.0, "20 then the same-tick decrement");
        let w = desert();
        let mut n = shark(100.0, 100.0);
        n.local_ai[0] = -1.0;
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.local_ai[0], -1.0, "the marker survives in sand");
    }

    /// `:40641-40667` — the cruise flips: collideX negates vx AND the
    /// facing (+netUpdate); collideY negates vy and writes directionY —
    /// then the bob overwrites ai[0] from the sand-above flag (the
    /// verbatim overwrite).
    #[test]
    fn w4_d33_s103_the_cruise_flips_and_the_bob_overwrite() {
        let w = desert();
        // The sand-above probe: (Center+(0,24)) two rows up — carve it
        // so sand_above is false → the bob writes ai[0] = 1.
        let mut w = w;
        let n0 = shark(100.0, 100.0);
        let (px, py) = tile_of(Vec2::new(n0.x + 50.0, n0.y + 12.0 + 24.0));
        let i = w.index(px, py - 2);
        w.tiles[i] = Tile::default();
        let players = [cand(180.0, 100.0)]; // near → cruise
        let vy = [0.0f32];
        let mut n = shark(100.0, 100.0);
        n.collide_x = true;
        n.collide_y = true;
        n.vx = 2.0;
        n.vy = 2.0;
        n.net_update = false;
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.facing, -1, "the collideX facing flip");
        assert!(n.net_update);
        assert_eq!(n.direction_y, -1, "Sign(-2)");
        // The bob overwrote the collideY ai[0] = -1 with +1 (no sand
        // above), then vy = -2 + 0.01, inside ±0.4? No: -1.99 → damp.
        assert_eq!(n.ai[0], 1.0, "the bob overwrite");
        assert!((n.vy - (-1.99 * 0.95)).abs() < 1e-5, "the ±0.4 damp");
        // vx: -2 (flip) + (-1)*0.1 = -2.1, inside ±6 → no damp.
        assert!((n.vx - (-2.1)).abs() < 1e-5);
    }

    /// `:40714-40722` — the tilt tail: vy * direction * 0.1 clamped
    /// ±0.2.
    #[test]
    fn w4_d33_s103_the_tilt_clamps() {
        let w = air();
        let players = [cand(1000.0, 100.0)];
        let vy = [0.0f32];
        let mut n = shark(100.0, 100.0);
        n.vy = 9.7; // +0.3 fall → 10 → tilt 1.0 → clamp 0.2
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.rotation, 0.2);
        let mut n = shark(100.0, 100.0);
        n.facing = -1;
        n.vy = 9.7;
        let _ = style103_step(&mut n, &env);
        assert_eq!(n.rotation, -0.2);
    }

    /// `NPC.cs:74681-74692` — the 16-tick counter over the 4-row sheet:
    /// row = (int)(counter/4), the wrap at 16 zeroes both.
    #[test]
    fn w4_d33_s103_the_find_frame_rows() {
        assert_eq!(shark_find_frame(0, 0.0), (0, 1.0));
        assert_eq!(shark_find_frame(0, 3.0), (1, 4.0));
        assert_eq!(shark_find_frame(1, 7.0), (2, 8.0));
        assert_eq!(shark_find_frame(3, 14.0), (3, 15.0));
        assert_eq!(shark_find_frame(3, 15.0), (0, 0.0), "the 16 wrap");
    }

    /// The whole arm is draw-free: 60 ticks through the desert chase,
    /// the lunge pocket and the off-sand fall leave the stream
    /// untouched (the TargetClosest scan is RNG-free, the sounds ride
    /// the audio seam).
    #[test]
    fn w4_d33_s103_the_arm_draws_nothing() {
        let players = [cand(1000.0, 100.0)];
        let vy = [0.0f32];
        let mut rng = UnifiedRandom::new(103);
        let w = desert();
        let mut n = shark(100.0, 100.0);
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        for _ in 0..60 {
            let _ = style103_step(&mut n, &env);
        }
        let w = air();
        let env = Style103Env {
            world: &w,
            players: &players,
            players_vy: &vy,
            remix_world: false,
            ticker: 7,
        };
        for _ in 0..60 {
            let _ = style103_step(&mut n, &env);
        }
        assert_eq!(
            rng.peek(),
            UnifiedRandom::new(103).peek(),
            "the 103 arm is draw-free"
        );
    }
}
