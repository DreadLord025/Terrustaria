//! The aiStyle-126 family — the Statue Mimic
//! (`NPC.AI_126_StatueMimic`, `NPC.cs:43439-43681`, dispatched at
//! `:41124-41127`). W4-d33, research/25 row 126, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the StatueMimic 690
//! (`NPCID.StatueMimic`, `ID/NPCID.cs:12057`; `SetDefaults` row
//! `NPC.cs:17680-17695`). The census of `this.aiStyle = 126;` returns
//! exactly that one hit (`:17687`) — no projectile-space user. The
//! port's roster has no statue-mimic kind — the module lands UNWIRED
//! (the `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The disguised statue: state 0 it lurks (immortal, untouchable),
//! teleports to unseen floor near the player, and reveals when the
//! player comes within 96 px; state 1 it hops after the player.
//!
//! The `noTileCollide` wet-clear (`:43441-43444`).
//!
//! State 0 (`:43445-43555`): `timeLeft = 60`; TargetClosest;
//! `direction = oldDirection`; the client RETURNS. The
//! `localAI[1] == 0` statue-style latch — `ai[1] =
//! GetRandomStatueStyleForStatueMimics()` (the `genRand.Next(83)`
//! re-roll while 43..=49, the SEPARATE worldgen stream — the env
//! seam). `localAI[0]++`; the alive player within 96 px AND sighted
//! reveals to state 1. Else the `localAI[0] >= 10` teleport cadence:
//! the `CanAnyPlayerSeeThisTile` guard (the centre/Bottom tile), the
//! player-vs-NPC screen-box column/row ranges (`NPC.sWidth/2 = 960`,
//! `sHeight/2 = 600`), up to 10 `Next(num2, num3)` × `Next(num4,
//! num5)` rolls — each drops up to 10 rows to a `SolidTile2` pair,
//! unseen, `IsThisAGoodPlaceForAStatueMimic` (the 3-row headroom), and
//! CLOSER to the player than the current spot; the teleport faces the
//! destination, netUpdate. RETURN.
//!
//! State 1 (`:43556-43680`): `immortal = false`, `dontTakeDamage =
//! false`; the `localAI[2] == 0` scare cue. The hop triggers — falling
//! below the player's Bottom AND in solid (the jump cue), OR grounded
//! with the `ai[2]` 20-tick countdown (the jump cue at 20). The
//! over-player X-overlap damp (×0.75, the rising vy too). The dead
//! player's laugh (the `ai[3]` 300 latch, the >200 window). The hop
//! (`flag2`): `noTileCollide = true`, TargetClosest, `vy = -9.01 -
//! Clamp((Bottom.Y - playerBottom.Y)/40, 0, 10)` (0 when dead), `vx =
//! (4 + min(|dx|/50, 12)) * direction`. The air tail: the zero-vy
//! RETURN; the dead-and-over / over-top `vy = 16` slam; else the
//! falling gravity + the past-player ×0.96 damp; the `noTileCollide`
//! re-set (rising / over-top / not-in-solid).
//!
//! ## RNG order (by construction)
//!
//! The statue-style latch: the genRand stream's Next(83) re-roll
//! (SEPARATE from Main.rand — the env seam). The teleport: per attempt
//! Next(num2, num3) + Next(num4, num5) on Main.rand. The hop, the
//! damps, the cues draw NOTHING (the sounds ride the audio seam).
//!
//! ## FindFrame
//!
//! No case 690 exists in `NPC.FindFrame`;
//! `Main.npcFrameCount[690] = 1`. Nothing to port.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::tiledata::{SET_PLATFORMS, TILE_SOLID};
use tr_core::{UnifiedRandom, World};

/// The type-690 `SetDefaults` row (`NPC.cs:17680-17695`), verbatim.
pub struct StatueMimicDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit58` (`SoundID.cs:1314`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath67` (`SoundID.cs:1381`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub npc_slots: f32,
    pub lava_immune: bool,
    pub immortal: bool,
    pub dont_take_damage: bool,
}

/// `NPC.cs:17680-17695` — `else if (this.type == 690)`.
pub const STATUE_MIMIC: StatueMimicDefaults = StatueMimicDefaults {
    width: 28,
    height: 38,
    ai_style: 126,
    damage: 40,
    defense: 10,
    life_max: 400,
    hit_sound: (3, 58),
    death_sound: (4, 67),
    knock_back_resist: 0.0,
    npc_slots: 0.0,
    lava_immune: true,
    immortal: true,
    dont_take_damage: true,
};

/// `Main.npcFrameCount[690]` = 1 (`Main.cs:67153`, the 690-696 row).
pub const STATUE_MIMIC_FRAME_COUNT: u32 = 1;

/// `NPC.sWidth` / `NPC.sHeight` (`NPC.cs:6767-6769`) — the screen box.
pub const S_WIDTH: i32 = 1920;
pub const S_HEIGHT: i32 = 1200;

/// The per-tick environment.
pub struct Style126Env<'a> {
    pub world: &'a World,
    /// The `Main.rand` stream (the teleport rolls).
    pub rng: &'a mut UnifiedRandom,
    /// The `WorldGen.genRand` stream (the statue-style latch) — the
    /// SEPARATE worldgen stream (the editing.rs FastRandom stand-in
    /// convention; the wiring seeds it).
    pub gen_rand: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.netMode`.
    pub net_mode: i32,
    /// The current `noTileCollide` (the `Npc` slot has no field — the
    /// env carries it in, the outcome carries the last write out).
    pub no_tile_collide: bool,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style126Outcome {
    /// The `SoundEngine.PlaySound` trackable cues in C# order —
    /// `(name, x, y)`: `statuemimic_scare` (`:43562`),
    /// `statuemimic_jump` (`:43569`/`:43580`), `statuemimic_laugh`
    /// (`:43608`). The audio family owns the pitch draws.
    pub sounds: Vec<(&'static str, f32, f32)>,
    /// The last `noTileCollide` write (the per-state flips).
    pub no_tile_collide: bool,
}

/// XNA `Vector2.Length()` — the f32 sum widened to f64 for the root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
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

/// `WorldGen.SolidTile2(int, int)` (`WorldGen.cs:70492-70509`): the
/// platform-aware solid — active && solid && ((platform && (half ||
/// topSlope)) || slope == 0) && !half && !inactive; null = solid,
/// out-of-bounds = false (the catch).
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && ((SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope())) || t.slope() == 0)
        && !t.half_brick()
        && !t.inactive()
}

/// `NPC.CanAnyPlayerSeeThisTile` (`NPC.cs:43416-43428`): any active
/// player's `getRect()` intersects the tile's screen box (the
/// `sWidth+32` × `sHeight+32` rect centred on the tile).
fn can_any_player_see_tile(env: &Style126Env<'_>, x: i32, y: i32) -> bool {
    let num = 32;
    let rx = x * 16 - S_WIDTH / 2 - num / 2;
    let ry = y * 16 - S_HEIGHT / 2 - num / 2;
    let rw = S_WIDTH + num;
    let rh = S_HEIGHT + num;
    for p in env.players {
        if p.active {
            // getRect() = ((int)x, (int)y, w, h) (Player.cs:42118).
            let px = p.x as i32;
            let py = p.y as i32;
            if px < rx + rw && px + p.width > rx && py < ry + rh && py + p.height > ry {
                return true;
            }
        }
    }
    false
}

/// `NPC.IsThisAGoodPlaceForAStatueMimic` (`NPC.cs:43430-43437`): the
/// SolidTile2 pair + the 3-row headroom over BOTH columns.
fn is_good_place_for_statue_mimic(world: &World, x: i32, y: i32) -> bool {
    if !solid_tile2(world, x, y) || !solid_tile2(world, x + 1, y) {
        return false;
    }
    for dy in 1..=3 {
        for dx in 0..=1 {
            if world.tile(x + dx, y - dy).is_some_and(|t| t.active()) {
                return false;
            }
        }
    }
    true
}

/// `WorldGen.GetRandomStatueStyleForStatueMimics`
/// (`WorldGen.cs:37918-37926`): the genRand Next(83) re-roll while
/// 43..=49.
fn get_random_statue_style(gen_rand: &mut UnifiedRandom) -> i32 {
    let mut num = gen_rand.next_max(83);
    while (43..=49).contains(&num) {
        num = gen_rand.next_max(83);
    }
    num
}

/// One fixed step of the aiStyle-126 arm (`NPC.cs:43439-43681`), in C#
/// order. The caller owns the movement integration and drains
/// [`Style126Outcome`].
pub fn style126_step(npc: &mut Npc, env: &mut Style126Env<'_>) -> Style126Outcome {
    let mut out = Style126Outcome::default();
    let mut no_tile_collide = env.no_tile_collide;
    // `:43441-43444` — the wet clear.
    if no_tile_collide {
        npc.wet = false;
    }
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    if npc.ai[0] == 0.0 {
        // `:43445-43555` — the lurk.
        npc.despawn_timer = 60; // timeLeft = 60
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 126, env.ticker);
        }
        npc.facing = npc.old_direction;
        if env.net_mode == 1 {
            out.no_tile_collide = no_tile_collide;
            return out;
        }
        if npc.local_ai[1] == 0.0 {
            npc.local_ai[1] = 1.0;
            npc.ai[1] = get_random_statue_style(env.gen_rand) as f32;
            npc.net_update = true;
        }
        npc.local_ai[0] += 1.0;
        let p = target_of(npc, env.players);
        let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
        let to_npc = center - pc;
        if !p.dead && length(to_npc) < 96.0 {
            // `Collision.CanHit(this, player)` — the box sight.
            if crate::projectile::can_hit(
                env.world,
                npc.x,
                npc.y,
                w,
                h,
                p.x,
                p.y,
                p.width as f32,
                p.height as f32,
            ) {
                npc.ai[0] = 1.0;
                npc.net_update = true;
            }
        } else {
            if npc.local_ai[0] < 10.0 {
                out.no_tile_collide = no_tile_collide;
                return out;
            }
            npc.local_ai[0] = 0.0;
            // `CanAnyPlayerSeeThisTile((int)Center.X/16, (int)Bottom.Y/16)`.
            let see_x = center.x as i32 / 16;
            let see_y = (npc.y + h) as i32 / 16;
            if can_any_player_see_tile(env, see_x, see_y) {
                out.no_tile_collide = no_tile_collide;
                return out;
            }
            // `:43481-43515` — the column/row ranges.
            let num = 0;
            let (mut num2, mut num3) = (-1i32, -1i32);
            if (pc.x - center.x).abs() < (S_WIDTH / 2) as f32 {
                num2 = ((pc.x - (S_WIDTH / 2) as f32 - num as f32) / 16.0) as i32;
                num3 = ((pc.x + (S_WIDTH / 2) as f32 + num as f32) / 16.0) as i32;
            } else if pc.x < center.x {
                num2 = ((pc.x + (S_WIDTH / 2) as f32 + num as f32) / 16.0) as i32;
                num3 = (center.x / 16.0) as i32;
            } else {
                num2 = (center.x / 16.0) as i32;
                num3 = ((pc.x - (S_WIDTH / 2) as f32 - num as f32) / 16.0) as i32;
            }
            let (mut num4, mut num5) = (-1i32, -1i32);
            if (pc.y - center.y).abs() < (S_HEIGHT / 2) as f32 {
                num4 = ((pc.y - (S_HEIGHT / 2) as f32 - num as f32) / 16.0) as i32;
                num5 = ((pc.y + (S_HEIGHT / 2) as f32 + num as f32) / 16.0) as i32;
            } else if pc.y < center.y {
                num4 = ((pc.y + (S_HEIGHT / 2) as f32 + num as f32) / 16.0) as i32;
                num5 = (center.y / 16.0) as i32;
            } else {
                num4 = (center.y / 16.0) as i32;
                num5 = ((pc.y - (S_HEIGHT / 2) as f32 - num as f32) / 16.0) as i32;
            }
            // `:43516-43552` — the 10 teleport attempts.
            for _ in (0..10).rev() {
                // `Next(num2, num3)` / `Next(num4, num5)` verbatim —
                // the C# asserts min <= max (an inverted range throws);
                // the port's next_range carries the same assert.
                let tx = env.rng.next_range(num2, num3);
                let mut ty = env.rng.next_range(num4, num5);
                let mut found = false;
                for _ in 0..10 {
                    if solid_tile2(env.world, tx, ty) && solid_tile2(env.world, tx + 1, ty) {
                        if !can_any_player_see_tile(env, tx + 1, ty) {
                            found = true;
                        }
                        break;
                    }
                    ty += 1;
                }
                if found && is_good_place_for_statue_mimic(env.world, tx, ty) {
                    let dest = Vec2::new(
                        (tx * 16 + 16) as f32 - (w as i32 / 2) as f32,
                        (ty * 16) as f32 - h,
                    );
                    let cur_to_player = Vec2::new(npc.x, npc.y) - Vec2::new(p.x, p.y);
                    if length(dest - Vec2::new(p.x, p.y)) < length(cur_to_player) {
                        if dest.x < npc.x {
                            npc.facing = -1;
                        } else if dest.x > npc.x {
                            npc.facing = 1;
                        }
                        npc.x = dest.x;
                        npc.y = dest.y;
                        npc.net_update = true;
                        break;
                    }
                }
            }
        }
        out.no_tile_collide = no_tile_collide;
        return out;
    }
    // `:43556-43680` — the revealed hop.
    npc.immortal = false;
    npc.dont_take_damage = false;
    let mut hop = false; // flag2
    if npc.local_ai[2] == 0.0 {
        npc.local_ai[2] = 1.0;
        out.sounds.push(("statuemimic_scare", center.x, center.y));
    }
    let p = target_of(npc, env.players);
    let p_bottom = p.y + p.height as f32;
    let p_top = p.y;
    let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
    if npc.vy > 0.0 && npc.y > p_bottom {
        if crate::player::Player::solid_collision(env.world, npc.x, npc.y, w, h) {
            hop = true;
            out.sounds.push(("statuemimic_jump", center.x, center.y));
            // `:43570-43572` — the empty `if (!(... > 480)) {}` block
            // (the decompiler residue, no effect).
        }
    } else if npc.vy == 0.0 {
        let countdown = 20; // num9
        if npc.ai[2] == countdown as f32 {
            out.sounds.push(("statuemimic_jump", center.x, center.y));
        }
        npc.vx = 0.0;
        npc.ai[2] -= 1.0;
        if npc.ai[2] <= 0.0 {
            npc.ai[2] = countdown as f32;
            hop = true;
            npc.net_update = true;
        }
    }
    // `:43591-43603` — the over-player overlap.
    let mut over = false; // flag3
    let right = npc.x + w;
    let left = npc.x;
    let p_left = p.x;
    let p_right = p.x + p.width as f32;
    if right >= p_left && left <= p_right {
        if npc.y + h < p_top {
            npc.vx *= 0.75;
            if npc.vy < 0.0 {
                npc.vy *= 0.75;
            }
        }
        over = true;
    }
    // `:43604-43620` — the dead-player laugh.
    if p.dead {
        if npc.ai[3] == 0.0 {
            out.sounds.push(("statuemimic_laugh", center.x, center.y));
            npc.ai[3] = 300.0;
        }
        if npc.ai[3] > 200.0 {
            over = true;
            npc.ai[3] -= 1.0;
        }
    } else if npc.ai[3] > 0.0 {
        npc.ai[3] -= 1.0;
    }
    if hop {
        // `:43621-43635` — the hop launch.
        no_tile_collide = true;
        if !env.players.is_empty() {
            target_closest(npc, env.world, env.players, true, 126, env.ticker);
        }
        let p = target_of(npc, env.players);
        let mut lift = (npc.y + h - (p.y + p.height as f32)) / 40.0; // num10
        lift = if !p.dead { lift.clamp(0.0, 10.0) } else { 0.0 };
        npc.vy = -9.01 - lift;
        let mut side = (center.x - (p.x + (p.width / 2) as f32)).abs() / 50.0; // num11
        if side > 12.0 {
            side = 12.0;
        }
        npc.vx = (4.0 + side) * npc.facing as f32;
        npc.net_update = true;
    } else {
        // `:43636-43680` — the air tail.
        if npc.vy == 0.0 {
            out.no_tile_collide = no_tile_collide;
            return out;
        }
        if (p.dead & over) && npc.y + h < p_bottom {
            npc.vy = 16.0;
        } else if over && npc.y + h < p_top {
            npc.vy = 16.0;
        } else {
            if npc.vy > 0.0 {
                npc.vy += crate::npc::GRAVITY;
            }
            if !p.dead {
                if npc.facing > 0 && center.x > pc.x {
                    npc.vx *= 0.96;
                    npc.vy *= 0.96;
                }
                if npc.facing < 0 && center.x < pc.x {
                    npc.vx *= 0.96;
                    npc.vy *= 0.96;
                }
            }
        }
        if npc.vy < 0.0 {
            no_tile_collide = true;
        } else if over && npc.y + h < p_top {
            no_tile_collide = true;
        } else if !crate::player::Player::solid_collision(env.world, npc.x, npc.y, w, h) {
            no_tile_collide = false;
        }
    }
    out.no_tile_collide = no_tile_collide;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 690-shaped NPC (the style73 placeholder convention).
    fn mimic(x: f32, y: f32) -> Npc {
        let d = &STATUE_MIMIC;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.old_direction = 1;
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

    /// `NPC.cs:17680-17695` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s126_the_defaults_row() {
        let d = &STATUE_MIMIC;
        assert_eq!((d.width, d.height), (28, 38));
        assert_eq!(d.ai_style, 126);
        assert_eq!((d.damage, d.defense, d.life_max), (40, 10, 400));
        assert_eq!(d.hit_sound, (3, 58));
        assert_eq!(d.death_sound, (4, 67));
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.npc_slots, 0.0);
        assert!(d.lava_immune && d.immortal && d.dont_take_damage);
        assert_eq!(STATUE_MIMIC_FRAME_COUNT, 1);
        assert_eq!((S_WIDTH, S_HEIGHT), (1920, 1200));
    }

    /// `:43454-43459` — the statue-style latch: the genRand Next(83)
    /// re-roll while 43..=49 (the SEPARATE stream).
    #[test]
    fn w4_d33_s126_the_statue_style_latch() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(126);
        let mut gen_rng = UnifiedRandom::new(999);
        let mut gen_oracle = UnifiedRandom::new(999);
        let mut n = mimic(100.0, 100.0);
        let mut env = Style126Env {
            world: &w,
            rng: &mut rng,
            gen_rand: &mut gen_rng,
            players: &players,
            net_mode: 0,
            no_tile_collide: false,
            ticker: 7,
        };
        let _ = style126_step(&mut n, &mut env);
        let mut style = gen_oracle.next_max(83);
        while (43..=49).contains(&style) {
            style = gen_oracle.next_max(83);
        }
        assert_eq!(gen_rng.peek(), gen_oracle.peek(), "the genRand re-roll");
        assert_eq!(n.ai[1], style as f32);
        assert_eq!(n.local_ai[1], 1.0);
        assert!(!(43..=49).contains(&(n.ai[1] as i32)));
        // Main.rand untouched by the latch (the far player, no teleport
        // yet — localAI[0] is 1).
        assert_eq!(rng.peek(), UnifiedRandom::new(126).peek());
    }

    /// `:43461-43469` — the near-player reveal: within 96 px AND
    /// sighted → state 1.
    #[test]
    fn w4_d33_s126_the_near_reveal() {
        let w = air();
        let players = [cand(150.0, 100.0)]; // ~50 px from the mimic centre
        let mut rng = UnifiedRandom::new(126);
        let mut gen_rng = UnifiedRandom::new(999);
        let mut n = mimic(100.0, 100.0);
        n.local_ai[1] = 1.0; // skip the latch
        let mut env = Style126Env {
            world: &w,
            rng: &mut rng,
            gen_rand: &mut gen_rng,
            players: &players,
            net_mode: 0,
            no_tile_collide: false,
            ticker: 7,
        };
        let _ = style126_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 1.0, "the reveal");
        assert!(n.net_update);
    }

    /// `:43556-43563` — the reveal clears immortal/dontTakeDamage and
    /// plays the scare cue once.
    #[test]
    fn w4_d33_s126_the_reveal_scare() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(126);
        let mut gen_rng = UnifiedRandom::new(999);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 1.0;
        n.immortal = true;
        n.dont_take_damage = true;
        let mut env = Style126Env {
            world: &w,
            rng: &mut rng,
            gen_rand: &mut gen_rng,
            players: &players,
            net_mode: 0,
            no_tile_collide: false,
            ticker: 7,
        };
        let out = style126_step(&mut n, &mut env);
        assert!(!n.immortal);
        assert!(!n.dont_take_damage);
        assert!(out.sounds.iter().any(|&(n, _, _)| n == "statuemimic_scare"));
        assert_eq!(n.local_ai[2], 1.0);
    }

    /// `:43575-43590` + `:43621-43635` — the grounded countdown hop:
    /// ai[2] 1 → 0 resets to 20, the hop launches vy -9.01-lift.
    #[test]
    fn w4_d33_s126_the_countdown_hop() {
        let w = air();
        let players = [cand(300.0, 100.0)]; // level, to the right
        let mut rng = UnifiedRandom::new(126);
        let mut gen_rng = UnifiedRandom::new(999);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 1.0;
        n.local_ai[2] = 1.0; // skip the scare
        n.ai[2] = 1.0; // the countdown's last tick
        n.vy = 0.0;
        let mut env = Style126Env {
            world: &w,
            rng: &mut rng,
            gen_rand: &mut gen_rng,
            players: &players,
            net_mode: 0,
            no_tile_collide: false,
            ticker: 7,
        };
        let out = style126_step(&mut n, &mut env);
        assert_eq!(n.ai[2], 20.0, "the countdown reset");
        assert!(n.vy < -9.0, "the hop lift, vy {}", n.vy);
        assert!(n.vx > 0.0, "toward the right player");
        assert!(out.no_tile_collide);
        assert!(n.net_update);
    }

    /// `WorldGen.cs:37918-37926` — the style re-roll skips 43..=49.
    #[test]
    fn w4_d33_s126_the_style_reroll_range() {
        // Seed a stream and verify the result is never in 43..=49.
        for seed in 0..50 {
            let mut gen_rng = UnifiedRandom::new(seed);
            let style = get_random_statue_style(&mut gen_rng);
            assert!(!(43..=49).contains(&style), "seed {seed} gave {style}");
            assert!((0..83).contains(&style));
        }
    }

    /// `NPC.cs:43430-43437` — the good-place check: the solid pair +
    /// the 3-row headroom.
    #[test]
    fn w4_d33_s126_the_good_place_check() {
        let mut w = air();
        // A solid floor pair at (100, 150), (101, 150), air above.
        for x in 100..=101 {
            let i = w.index(x, 150);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        assert!(is_good_place_for_statue_mimic(&w, 100, 150));
        // Block the headroom.
        let i = w.index(100, 149);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(!is_good_place_for_statue_mimic(&w, 100, 150));
    }

    /// The stream parity: 60 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s126_the_run_is_twin_stable() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut a = mimic(100.0, 100.0);
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(690);
        let mut rng_b = UnifiedRandom::new(690);
        let mut gen_a = UnifiedRandom::new(42);
        let mut gen_b = UnifiedRandom::new(42);
        for tick in 0..60 {
            let mut env_a = Style126Env {
                world: &w,
                rng: &mut rng_a,
                gen_rand: &mut gen_a,
                players: &players,
                net_mode: 0,
                no_tile_collide: false,
                ticker: 7,
            };
            let _ = style126_step(&mut a, &mut env_a);
            let mut env_b = Style126Env {
                world: &w,
                rng: &mut rng_b,
                gen_rand: &mut gen_b,
                players: &players,
                net_mode: 0,
                no_tile_collide: false,
                ticker: 7,
            };
            let _ = style126_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "state @{tick}");
        }
    }
}
