//! W4-d24 (aiStyle-19) — the Antlion, `NPC.AI`'s `if (this.aiStyle == 19)`
//! block (`NPC.cs:24407-24536`), ported 1:1 as a PURE machine (code-only
//! mode: no compiler/tests were run on this branch; the witness tests
//! below are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 19` rows, whole-file grep)
//!
//! ONE vanilla member: Antlion 69 (`NPCID.cs` `Antlion = 69`,
//! `SetDefaults` `NPC.cs:9666-9679`: 24×24, dmg 10, def 6, 45 HP, kb 0,
//! NPCHit31/NPCDeath34, value 60, behindTiles). The port has no
//! `NpcKind::Antlion` yet — the module lands UNWIRED (the
//! `npc_ai_style17.rs` precedent). The wiring step adds the kind + the
//! style-axis claim.
//!
//! ## The machine
//!
//! * the head (`:24409-24417`): `TargetClosest()` every tick; the
//!   12-speed aim from the NPC centre to the player's centre X / TOP Y
//!   (the asymmetric `player5.position.Y` — verbatim).
//! * the `directionY < 0` aim block (`:24418-24444`): the
//!   `Atan2 + 1.57` rotation, `flag28` = the double-promoted ±1.2 band
//!   AND the player active+alive, the ±0.8 rotation clamp AFTER the
//!   band test, and the vx ×0.9 decay whose `> -0.1 || < 0.1` guard is
//!   ALWAYS true (the C# OR quirk — vx always zeroes, netUpdate arms).
//! * the shot cooldown (`:24445-24452`): `ai[0] > 0` plays
//!   `SoundID.Item5` at exactly 200 (the tick AFTER the shot armed it),
//!   then decrements.
//! * the shot arm (`:24453-24484`, `ai[0] == 0`): `flag28` → the
//!   firing-distance + CanHit gate fires the aimed SandBallFalling 31
//!   (dmg 10, `ai[0] = 2`, `timeLeft = 300`, `friendly = false`);
//!   else the conveyor probe (`WorldGen.InWorld` + tile type 421/422
//!   at `(Center.X/16, (Bottom.Y+8)/16)`) fires along
//!   `rotation − 1.57` at the same 12 speed. Both re-arm `ai[0] = 200`.
//! * the ground probe (`:24485-24535`): the three-column feet scan
//!   (left / INTEGER-half / right at the bottom row, `nactive() &&
//!   tileSolid`); grounded → `noGravity = noTileCollide = true`,
//!   `vy = -0.2`, return. Airborne → both flags false, then the
//!   1/2-gated sand dust (type 32, `width+8` × 24 at
//!   `(x−4, y+height−8) + netOffset`, speed `(0, vy/2)`, the ×0.4 X /
//!   ×−1 Y overwrite, the second 1/2 `noGravity + scale+0.2` roll).
//!   The C# `try/catch` returns on an out-of-world probe — the port's
//!   `world.tile` None folds to that early return (the flags keep
//!   their pre-probe values; the Antlion's defaults are gravity-on).
//!
//! Seam: writes `ai[0]`, `rotation`, `vx`, `netUpdate`; the
//! `noGravity`/`noTileCollide` pair leaves as [`Ai19Outcome`] (the
//! `npc_ai_style50` convention — the port's `Npc` has no such fields).
//! The shot leaves as [`Ai19Effects::shots`] (the `HostileShotAi`
//! shape; the caller's drain owes `timeLeft = 300` + `friendly =
//! false`, the DemonSickle timed-shot precedent); the Item5 play
//! leaves as [`Ai19Effects::sounds`] (`(style, x, y)`, the
//! `wants_item_sounds` seam — the audio drain owns case 2's pitch
//! draw). The dust runs IN-STEP through the caller's `DustPool` on the
//! same `UpdateNPCs` stream (the `npc_ai_style75` pooled convention).
//! `Main.netMode != 1`/`!= 2` fold true (single-player).
//!
//! ## RNG census (in C# order)
//!
//! 1. the sand-dust gate `Next(2)` (`:24517`) — EVERY airborne tick
//!    (after the flag writes, before the dust);
//! 2. `Dust.NewDust`'s internal draws (`Dust.cs:75-256`, the verbatim
//!    `DustPool::new_dust`) — only when gate 1 passes;
//! 3. the dust-tweak gate `Next(2)` (`:24523`) — only when gate 1
//!    passes, AFTER the pool's draws.
//! The grounded path draws NOTHING. `TargetClosest`,
//! `AI_GlobalFiringDistanceCheck` (`NPC.cs:55835-55841`) and
//! `Collision.CanHit` draw nothing; the Item5 pitch draw belongs to
//! the audio drain (the queue preserves the intra-tick order). The
//! witnesses mirror the gates on a cloned stream.
#![allow(dead_code)] // UNWIRED (W4-d24): the Antlion kind + the axis claim are the wiring step

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{HostileShotAi, Npc};
use crate::npc_target::{target_closest, TargetCandidate};

/// The Antlion's `SetDefaults` row (`NPC.cs:9666-9679`), verbatim.
pub struct AntlionDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub hit_sound: (i32, i32),
    pub death_sound: (i32, i32),
    pub value: f32,
    pub behind_tiles: bool,
}

/// `NPC.cs:9666-9679` — `else if (this.type == 69)`.
pub const ANTLION: AntlionDefaults = AntlionDefaults {
    width: 24,
    height: 24,
    ai_style: 19,
    damage: 10,
    defense: 6,
    life_max: 45,
    knock_back_resist: 0.0,
    hit_sound: (3, 31),
    death_sound: (4, 34),
    value: 60.0,
    behind_tiles: true,
};

/// The per-tick movement flags the arm writes (`:24510-24516`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ai19Outcome {
    pub no_gravity: bool,
    pub no_tile_collide: bool,
}

/// The side effects beyond the NPC state.
#[derive(Clone, Debug, Default)]
pub struct Ai19Effects {
    /// The SandBallFalling 31 requests (`:24462`/`:24477`): dmg 10,
    /// `ai0 = 2`; the drain owes `timeLeft = 300` + `friendly = false`.
    pub shots: Vec<HostileShotAi>,
    /// `PlaySound(SoundID.Item5, position)` (`:24449`) — `(5, x, y)`;
    /// `SoundID.Item5 = (2, 5)` (`ID/SoundID.cs:1388`).
    pub sounds: Vec<(i32, f32, f32)>,
}

/// The target slot's candidate (the `npc_ai_style16.rs::target_of`
/// fallback — an unset/stale slot reads slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// The aiStyle-19 arm (`NPC.cs:24407-24536`). One fixed step; the dust
/// runs in-step on the caller's pool + stream. Returns the movement
/// flags + the shot/sound requests.
pub fn ai_style19(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) -> (Ai19Outcome, Ai19Effects) {
    let mut fx = Ai19Effects::default();
    // The pre-probe flag state: the C# field keeps its defaults until
    // the probe writes (the Antlion's SetDefaults has gravity on).
    let mut outcome = Ai19Outcome { no_gravity: false, no_tile_collide: false };
    let (w, h) = npc.size();
    // `:24409` — the every-tick re-aim.
    target_closest(npc, world, players, true, 19, ticker);
    let p = target_of(npc, players);
    // `:24411-24417` — the 12-speed aim: centre X vs the player's TOP Y.
    let num273 = 12.0f32;
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut num274 = p.x + (p.width / 2) as f32 - cx;
    let mut num275 = p.y - cy;
    let num276 = num273 / (num274 * num274 + num275 * num275).sqrt();
    num274 *= num276;
    num275 *= num276;
    // `:24418-24444` — the directionY < 0 aim block.
    let mut flag28 = false;
    if npc.direction_y < 0 {
        npc.rotation = (num275.atan2(num274) as f64 + 1.57) as f32;
        flag28 = !((npc.rotation as f64) < -1.2) && !((npc.rotation as f64) > 1.2);
        if !p.active || p.dead {
            flag28 = false;
        }
        if npc.rotation < -0.8 {
            npc.rotation = -0.8;
        } else if npc.rotation > 0.8 {
            npc.rotation = 0.8;
        }
        if npc.vx != 0.0 {
            npc.vx *= 0.9;
            // `:24438` — the C# OR quirk: `> -0.1 || < 0.1` is always
            // true, so the decay ALWAYS zeroes vx.
            if npc.vx > -0.1 || npc.vx < 0.1 {
                npc.vx = 0.0;
                npc.net_update = true;
            }
        }
    }
    // `:24445-24452` — the cooldown: the Item5 play at exactly 200.
    if npc.ai[0] > 0.0 {
        if npc.ai[0] == 200.0 {
            fx.sounds.push((5, npc.x, npc.y));
        }
        npc.ai[0] -= 1.0;
    }
    // `:24453-24484` — the shot arm at ai[0] == 0.
    if npc.ai[0] == 0.0 {
        let num277 = (cx / 16.0) as i32;
        let num278 = ((npc.y + h + 8.0) / 16.0) as i32;
        if flag28 {
            // `:24459` — the firing-distance rect + the box CanHit.
            let (pcx, pcy) = (p.x + p.width as f32 * 0.5, p.y + p.height as f32 * 0.5);
            if crate::npc_ai_style14::global_firing_distance_check(cx, cy, pcx, pcy)
                && crate::projectile::can_hit(
                    world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
                )
            {
                npc.ai[0] = 200.0;
                fx.shots.push(HostileShotAi {
                    kind: 31, x: cx, y: cy, vx: num274, vy: num275,
                    damage: 10, ai0: 2.0, ai1: 0.0, scale: None,
                });
                npc.net_update = true;
            }
        } else if world.tile(num277, num278).is_some_and(|t| t.kind == 421 || t.kind == 422) {
            // `:24470-24483` — the conveyor shot along rotation − 1.57.
            // (`WorldGen.InWorld` folds into the `tile` None check.)
            let num280 = npc.rotation - 1.57;
            num274 = num280.cos() * num273;
            num275 = num280.sin() * num273;
            npc.ai[0] = 200.0;
            fx.shots.push(HostileShotAi {
                kind: 31, x: cx, y: cy, vx: num274, vy: num275,
                damage: 10, ai0: 2.0, ai1: 0.0, scale: None,
            });
            npc.net_update = true;
        }
    }
    // `:24485-24507` — the three-column feet probe. The C# null-tile
    // allocations are dead code in-world; an out-of-world read throws
    // into the catch (`:24532-24535`) → return with the flags unwritten.
    let num282 = npc.x as i32 / 16;
    let num283 = (npc.x + (w as i32 / 2) as f32) as i32 / 16;
    let num284 = (npc.x + w) as i32 / 16;
    let num285 = (npc.y + h) as i32 / 16;
    let solid_at = |x: i32| -> Option<bool> {
        world.tile(x, num285).map(|t| {
            t.active() && !t.inactive() && tr_core::tiledata::TILE_SOLID.has(t.kind)
        })
    };
    let (Some(a), Some(b), Some(c)) = (solid_at(num282), solid_at(num283), solid_at(num284)) else {
        // The catch return: no flag writes, no dust.
        return (outcome, fx);
    };
    if a || b || c {
        // `:24508-24514` — grounded: phase through, rise at 0.2.
        outcome.no_gravity = true;
        outcome.no_tile_collide = true;
        npc.vy = -0.2;
        return (outcome, fx);
    }
    // `:24515-24516` — airborne: gravity + collision back on.
    outcome.no_gravity = false;
    outcome.no_tile_collide = false;
    // `:24517-24529` — the 1/2-gated sand dust. The netOffset ride is
    // folded into the spawn position (the C# moves the body for the
    // call only — `position += netOffset` / `-= netOffset`).
    if rng.next_max(2) == 0 {
        let pos = Vec2::new(
            npc.x + npc.net_offset.x - 4.0,
            npc.y + npc.net_offset.y + h - 8.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32 + 8, 24, 32,
            0.0, npc.vy / 2.0, 0, Color::default(), 1.0,
        );
        // `:24521-24522` — the velocity overwrite.
        dust.slot_mut(idx).velocity.x *= 0.4;
        dust.slot_mut(idx).velocity.y *= -1.0;
        // `:24523-24527` — the second 1/2 roll.
        if rng.next_max(2) == 0 {
            dust.slot_mut(idx).no_gravity = true;
            dust.slot_mut(idx).scale += 0.2;
        }
    }
    (outcome, fx)
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation (`NPC.cs:24407-24536`).
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// A world with a solid floor at row 40 (kind 1 = dirt, in
    /// TILE_SOLID).
    fn ground() -> World {
        let mut w = air();
        for x in 0..200 {
            for y in 40..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
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

    /// A stand-in body with the Antlion's 24×24 box (the kind is
    /// unported; the machine is the family's only member).
    fn antlion(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((24.0, 24.0));
        n.hp = 45;
        n.max_hp = 45;
        n
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

    /// The `SetDefaults` row (`NPC.cs:9666-9679`).
    #[test]
    fn the_defaults_row_matches_the_census() {
        assert_eq!((ANTLION.width, ANTLION.height), (24, 24));
        assert_eq!(ANTLION.ai_style, 19);
        assert_eq!((ANTLION.damage, ANTLION.defense, ANTLION.life_max), (10, 6, 45));
        assert_eq!(ANTLION.knock_back_resist, 0.0);
        assert_eq!(ANTLION.hit_sound, (3, 31));
        assert_eq!(ANTLION.death_sound, (4, 34));
        assert_eq!(ANTLION.value, 60.0);
        assert!(ANTLION.behind_tiles);
    }

    /// `:24508-24514` — the grounded probe: noGravity + noTileCollide,
    /// vy = −0.2, and NO dust draw (the stream is untouched).
    #[test]
    fn the_grounded_antlion_phases_and_draws_nothing() {
        let w = ground();
        let mut n = antlion(1600.0, 40.0 * 16.0 - 24.0); // feet on row 40
        let mut rng = UnifiedRandom::new(5);
        let witness = rng.clone();
        let mut dust = DustPool::new();
        let (out, fx) = ai_style19(&mut n, &w, &[cand(1620.0, 600.0)], 0, &mut rng, &mut dust, &dust_ctx());
        assert!(out.no_gravity && out.no_tile_collide);
        assert_eq!(n.vy, -0.2);
        assert_eq!(rng, witness, "the grounded path draws nothing");
        assert!(fx.shots.is_empty());
    }

    /// `:24418-24444` — the directionY<0 block: the rotation clamp, the
    /// always-true OR quirk zeroes vx.
    #[test]
    fn the_aim_block_clamps_and_zeroes_vx() {
        let w = air();
        let mut n = antlion(1600.0, 300.0);
        n.direction_y = -1;
        n.vx = 5.0;
        let mut rng = UnifiedRandom::new(5);
        let mut dust = DustPool::new();
        // The player far below-right: the raw rotation exceeds 0.8.
        ai_style19(&mut n, &w, &[cand(1900.0, 900.0)], 0, &mut rng, &mut dust, &dust_ctx());
        assert_eq!(n.rotation, 0.8, "the +0.8 clamp");
        assert_eq!(n.vx, 0.0, "the OR quirk always zeroes");
        assert!(n.net_update);
        // directionY >= 0: no rotation write, no decay.
        let mut m = antlion(1600.0, 300.0);
        m.direction_y = 1;
        m.vx = 5.0;
        m.rotation = 0.3;
        ai_style19(&mut m, &w, &[cand(1900.0, 900.0)], 0, &mut rng, &mut dust, &dust_ctx());
        assert_eq!(m.rotation, 0.3);
        assert_eq!(m.vx, 5.0);
    }

    /// `:24445-24484` — the cooldown + the aimed shot: ai[0] counts
    /// down; at 0 with flag28 the SandBall 31 fires (dmg 10, ai0 2) and
    /// re-arms 200; the NEXT tick plays Item5 then decrements.
    #[test]
    fn the_shot_cycle_and_the_item5_tick() {
        let w = ground();
        let mut n = antlion(1600.0, 40.0 * 16.0 - 24.0);
        n.direction_y = -1;
        // The player above (the aim rotation inside ±1.2 → flag28).
        let p = cand(1602.0, 400.0);
        let mut rng = UnifiedRandom::new(5);
        let mut dust = DustPool::new();
        let (_, fx) = ai_style19(&mut n, &w, &[p], 0, &mut rng, &mut dust, &dust_ctx());
        assert_eq!(fx.shots.len(), 1, "the aimed shot");
        let s = &fx.shots[0];
        assert_eq!((s.kind, s.damage, s.ai0), (31, 10, 2.0));
        assert_eq!(n.ai[0], 200.0);
        assert!(fx.sounds.is_empty(), "no sound on the shot tick");
        // The next tick: the Item5 play at exactly 200, then 199.
        let (_, fx2) = ai_style19(&mut n, &w, &[p], 0, &mut rng, &mut dust, &dust_ctx());
        assert_eq!(fx2.sounds, vec![(5, n.x, n.y)]);
        assert_eq!(n.ai[0], 199.0);
        assert!(fx2.shots.is_empty(), "the cooldown blocks the shot");
    }

    /// `:24470-24483` — the conveyor arm: a 421/422 tile under the feet
    /// fires along rotation − 1.57 even without flag28.
    #[test]
    fn the_conveyor_shot_fires_along_the_rotation() {
        let mut w = ground();
        // The probe tile: (Center.X/16, (Bottom.Y+8)/16). The antlion
        // at (1600, 616): centre x 1612 → col 100; bottom 640+8 → row 40.
        let i = w.index(100, 40);
        w.tiles[i] = Tile { kind: 421, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut n = antlion(1600.0, 616.0);
        n.direction_y = 1; // no flag28 — the conveyor branch runs
        n.rotation = 0.0;
        let mut rng = UnifiedRandom::new(5);
        let mut dust = DustPool::new();
        let (_, fx) = ai_style19(&mut n, &w, &[cand(1620.0, 600.0)], 0, &mut rng, &mut dust, &dust_ctx());
        assert_eq!(fx.shots.len(), 1);
        let s = &fx.shots[0];
        // rotation 0 − 1.57: cos ≈ 0.0008, sin ≈ −1 → vx ≈ 0.0096, vy ≈ −12.
        assert!((s.vx - (-1.57f32).cos() * 12.0).abs() < 1e-4);
        assert!((s.vy - (-1.57f32).sin() * 12.0).abs() < 1e-4);
        assert_eq!(n.ai[0], 200.0);
    }

    /// `:24517-24527` — the airborne dust gates: the stream mirror pins
    /// the Next(2) gate, the pool draws and the second Next(2) in order.
    #[test]
    fn the_airborne_dust_gates_mirror_the_stream() {
        let w = air();
        let mut n = antlion(1600.0, 300.0);
        n.direction_y = 1;
        n.vy = 2.0;
        let mut rng = UnifiedRandom::new(31);
        let mut mirror = UnifiedRandom::new(31);
        let mut dust = DustPool::new();
        let mut mirror_dust = DustPool::new();
        let ctx = dust_ctx();
        // Find a seed tick where the first gate passes, then mirror the
        // whole sequence: gate, pool draws, tweak gate.
        let gate = mirror.next_max(2);
        let (out, _) = ai_style19(&mut n, &w, &[cand(1620.0, 600.0)], 0, &mut rng, &mut dust, &ctx);
        assert!(!out.no_gravity && !out.no_tile_collide);
        if gate == 0 {
            let pos = Vec2::new(n.x - 4.0, n.y + 24.0 - 8.0);
            let idx = mirror_dust.new_dust(
                &mut mirror, &ctx, &w, pos, 32, 24, 32, 0.0, 1.0, 0, Color::default(), 1.0,
            );
            mirror_dust.slot_mut(idx).velocity.x *= 0.4;
            mirror_dust.slot_mut(idx).velocity.y *= -1.0;
            if mirror.next_max(2) == 0 {
                mirror_dust.slot_mut(idx).no_gravity = true;
                mirror_dust.slot_mut(idx).scale += 0.2;
            }
            assert_eq!(dust.slot(idx).no_gravity, mirror_dust.slot(idx).no_gravity);
            assert_eq!(dust.slot(idx).scale, mirror_dust.slot(idx).scale);
        }
        assert_eq!(rng, mirror, "the gate + pool + tweak order holds");
    }

    /// The RNG census: the grounded path (the common Antlion state)
    /// draws NOTHING across many ticks — the stream state is pinned.
    #[test]
    fn the_grounded_census_draws_no_rng() {
        let w = ground();
        let mut n = antlion(1600.0, 40.0 * 16.0 - 24.0);
        let mut rng = UnifiedRandom::new(77);
        let witness = rng.clone();
        let mut dust = DustPool::new();
        for _ in 0..300 {
            ai_style19(&mut n, &w, &[cand(1620.0, 600.0)], 0, &mut rng, &mut dust, &dust_ctx());
        }
        assert_eq!(rng, witness, "300 grounded ticks draw nothing");
    }
}
