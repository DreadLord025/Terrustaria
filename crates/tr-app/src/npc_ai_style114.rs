//! The aiStyle-114 family — the Dragonflies
//! (`NPC.AI_114_Dragonflies`, `NPC.cs:47716-47886`, dispatched at
//! `:41076-41079`). W4-d33, research/25 row 114, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has SEVEN vanilla members: the Black/Blue/Green/Orange/
//! Red/Yellow/Gold Dragonflies 595-601 (`ID/NPCID.cs:11867-11879`; the
//! shared `SetDefaults` row `NPC.cs:16720-16738`). The census of
//! `this.aiStyle = 114;` returns exactly that one row (`:16725`) — no
//! projectile-space user. WIRED by W4-d33 (all seven kinds 595-601 +
//! the axis' 114 arm):
//! `Npc::ai_114_dragonfly` feeds the session player + the sim-stamped
//! `dragonfly_neighbors` pool, the integration tail is the no-gravity
//! `integrate_flyer_collision` (the row is noGravity but tile-collide).
//!
//! ## The machine
//!
//! The darting critter: a two-state hop (0 the rest, 1 the dart) around
//! a perch anchor (`ai[2]`, `ai[3]`), fleeing nearby threats and
//! re-perching on cattails or the ground below.
//!
//! The first tick (`:47718-47728`, non-client): the centre anchor
//! latch; the launch `(NextVector2Circular(5,3) +
//! NextVector2CircularEdge(5,3)) * 0.4` (3 NextFloat: the circular's
//! angle + radius, the edge's angle); `ai[1] = 0`, `ai[0] = 1`.
//!
//! State 0 (`:47731-47752`): the ×0.94 damp; the non-client
//! `++ai[1] >= 60 + Next(60)` re-dart — beyond 96 px from the anchor
//! `DirectionTo ×3`; beyond 16 px `DirectionTo ×1 +
//! NextVector2Circular(1, 0.5)`; else the fresh launch; `ai[1] = 0`,
//! `ai[0] = 1`, netUpdate.
//!
//! State 1 (`:47753-47799`): the dart timer (4 ticks, 200 when beyond
//! 112 px) flips back to 0; the 3-row centre-column probe (solid OR
//! liquid) damps a falling vy ×0.9 then lifts -0.2 PER HIT ROW; a
//! rising vy with no solid in the 30-row column damps ×0.9.
//!
//! The tail (`:47801-47814`): the vx-sign facing; the wet -3 snap; the
//! `localAI[1]` 15-tick threat-scan cooldown (decrement + RETURN).
//!
//! The threat scan (`:47815-47851`): every active hostile NPC
//! (`damage > 0 && !friendly`) within Hitbox.Distance 100 and every
//! active player within 150 adds a `DirectionFrom(their centre)`; the
//! mean ×2 kicks the velocity (the SafeNormalize ×16 cap), re-anchors
//! at `Center + kick*10`, `ai[1] = -10`, `ai[0] = 1`, netUpdate.
//!
//! The re-perch (`:47852-47885`, no threats, non-client, within 16 px
//! of the anchor): the anchor tile's type — 519 (the cattail) rolls
//! `Next(30)`, else `Next(4)`; a passing roll runs FindCattailTop (the
//! ±30/±20 reservoir scan with the growing `Next(num)` reservoir pick,
//! `frameX >= 180`); a found top re-anchors at `tile*16`; else the
//! ground search (the centre column down to the first SolidTile or
//! `worldSurface`, then `m -= Next(3,6)`) re-anchors at `tile*16`.
//!
//! ## RNG order (by construction)
//!
//! First tick: 3 NextFloat. State 0: Next(60) per non-client tick,
//! then the re-dart's branch draws (0 / 2 / 3). The threat scan draws
//! NOTHING. The re-perch: Next(30) or Next(4), then FindCattailTop's
//! per-candidate Next(num) (the reservoir), then the ground search's
//! Next(3,6). The probes and the damps draw nothing.
//!
//! ## FindFrame cases 595-601
//!
//! `NPC.cs:72722-72738`: `spriteDirection = direction`; the counter
//! wraps at `frameCount * 3` (12), the row is `(int)counter / 3` over
//! the 4-row sheet (`Main.npcFrameCount[595..601] = 4`).
//! [`dragonfly_find_frame`] ports the counter pair.
#![allow(dead_code)] // WIRED by W4-d33 (axis 114 → style114_step); the defaults row + witnesses stay as the audit cross-check

use crate::bow_launch::safe_normalize;
use crate::npc::Npc;
use crate::npc_ai_style64::rect_distance;
use glam::Vec2;
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// The shared type-595..601 `SetDefaults` row (`NPC.cs:16720-16738`),
/// verbatim. `timeLeft *= 3` is the spawn-time write (the wiring's).
pub struct DragonflyDefaults {
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
    pub npc_slots: f32,
    pub no_gravity: bool,
    /// `catchItem = type - 595 + 4334` (4334..4340).
    pub catch_item_base: i32,
    /// The Gold 601's rarity 3; the rest keep the head default.
    pub gold_rarity: i32,
}

/// `NPC.cs:16720-16738` — `else if (this.type >= 595 && <= 601)`.
pub const DRAGONFLY: DragonflyDefaults = DragonflyDefaults {
    width: 10,
    height: 10,
    ai_style: 114,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.25,
    no_gravity: true,
    catch_item_base: 4334,
    gold_rarity: 3,
};

/// `Main.npcFrameCount[595..=601]` = 4 (`Main.cs:67143`, the 590-599
/// row + the 600-609 row).
pub const DRAGONFLY_FRAME_COUNT: u32 = 4;

/// One `Main.npc[k]` slot as the threat scan sees it (`:47820`).
#[derive(Clone, Copy, Debug)]
pub struct DragonflyNeighbor {
    pub active: bool,
    /// `Main.npc[k].damage > 0`.
    pub damage: i32,
    pub friendly: bool,
    /// The hitbox (`(int)position.X/Y, width, height`) — the
    /// Hitbox.Distance measure.
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

/// One `Main.player[l]` slot as the threat scan sees it (`:47829`).
#[derive(Clone, Copy, Debug)]
pub struct DragonflyPlayer {
    pub active: bool,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

/// The per-tick environment.
pub struct Style114Env<'a> {
    pub world: &'a World,
    pub rng: &'a mut UnifiedRandom,
    /// The NPC slots in `Main.npc` order (the scan has NO self skip —
    /// the dragonfly's own damage 0 excludes it).
    pub neighbors: &'a [DragonflyNeighbor],
    /// The player slots in `Main.player` order.
    pub players: &'a [DragonflyPlayer],
    /// `Main.netMode`.
    pub net_mode: i32,
    /// `Main.worldSurface` in TILES — the ground search's floor
    /// (`:47878`, the DOUBLE compare).
    pub world_surface: f64,
}

/// XNA `Vector2.Distance` — the f32 sum widened to f64 for the root.
fn distance(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32
}

/// XNA `Vector2.Length()` — the same root.
fn length(v: Vec2) -> f32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt() as f32
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

/// `Utils.NextVector2CircularEdge` (`Utils.cs:1271-1274`) — ONE
/// NextFloat, no radius scale.
fn next_vector2_circular_edge(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    Vec2::new(unit.x * half_w, unit.y * half_h)
}

/// The launch velocity (`:47724` / `:47746`): `(Circular(5,3) +
/// CircularEdge(5,3)) * 0.4` — 3 NextFloat in order.
fn launch_velocity(rng: &mut UnifiedRandom) -> Vec2 {
    (next_vector2_circular(rng, 5.0, 3.0) + next_vector2_circular_edge(rng, 5.0, 3.0)) * 0.4
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`): the
/// strict test; null = solid, out-of-bounds = false (the catch).
fn solid_tile_ij(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && !t.inactive()
        && TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `NPC.FindCattailTop` (`NPC.cs:80180-80207`): the ±30/±20 scan for
/// the active 519 tiles with `frameX >= 180`, the growing `Next(num)`
/// reservoir pick (num starts 1, increments per accept). The
/// `InWorld(x, y, 31)` gate refuses the border. Returns the picked
/// tile only when it differs from the start.
fn find_cattail_top(
    world: &World,
    rng: &mut UnifiedRandom,
    land_x: i32,
    land_y: i32,
) -> Option<(i32, i32)> {
    if land_x < 31 || land_x >= world.width - 31 || land_y < 31 || land_y >= world.height - 31 {
        return None;
    }
    let mut num = 1;
    let mut picked = (land_x, land_y);
    for i in land_x - 30..=land_x + 30 {
        for j in land_y - 20..=land_y + 20 {
            if let Some(t) = world.tile(i, j) {
                if t.active() && t.kind == 519 && t.frame_x >= 180 && rng.next_max(num) == 0 {
                    picked = (i, j);
                    num += 1;
                }
            }
        }
    }
    if picked != (land_x, land_y) {
        Some(picked)
    } else {
        None
    }
}

/// One fixed step of the aiStyle-114 arm (`NPC.cs:47716-47886`), in C#
/// order. The caller owns the movement integration (the row's
/// `noGravity` is true). The arm returns `()` — every side effect is
/// NPC state (no shots, no sounds, no dust).
pub fn style114_step(npc: &mut Npc, env: &mut Style114Env<'_>) {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:47718-47728` — the first tick.
    if npc.local_ai[0] == 0.0 && env.net_mode != 1 {
        npc.local_ai[0] = 1.0;
        npc.ai[2] = center.x;
        npc.ai[3] = center.y;
        let v = launch_velocity(env.rng);
        npc.vx = v.x;
        npc.vy = v.y;
        npc.ai[1] = 0.0;
        npc.ai[0] = 1.0;
        npc.net_update = true;
    }
    let anchor = Vec2::new(npc.ai[2], npc.ai[3]);
    match npc.ai[0] as i32 {
        0 => {
            // `:47731-47752` — the rest.
            npc.vx *= 0.94;
            npc.vy *= 0.94;
            npc.ai[1] += 1.0; // `++this.ai[1]` — the client too
            if env.net_mode != 1 && npc.ai[1] >= (60 + env.rng.next_max(60)) as f32 {
                let d = distance(center, anchor);
                if d > 96.0 {
                    let dir = anchor - center;
                    let n = 1.0 / ((dir.x * dir.x + dir.y * dir.y) as f64).sqrt() as f32;
                    npc.vx = dir.x * n * 3.0;
                    npc.vy = dir.y * n * 3.0;
                } else if d > 16.0 {
                    let dir = anchor - center;
                    let n = 1.0 / ((dir.x * dir.x + dir.y * dir.y) as f64).sqrt() as f32;
                    let jitter = next_vector2_circular(env.rng, 1.0, 0.5);
                    npc.vx = dir.x * n + jitter.x;
                    npc.vy = dir.y * n + jitter.y;
                } else {
                    let v = launch_velocity(env.rng);
                    npc.vx = v.x;
                    npc.vy = v.y;
                }
                npc.ai[1] = 0.0;
                npc.ai[0] = 1.0;
                npc.net_update = true;
            }
        }
        1 => {
            // `:47753-47799` — the dart.
            let mut ticks = 4; // num
            if distance(center, anchor) > 112.0 {
                ticks = 200;
            }
            npc.ai[1] += 1.0;
            if npc.ai[1] >= ticks as f32 {
                npc.ai[1] = 0.0;
                npc.ai[0] = 0.0;
                npc.net_update = true;
            }
            let col = center.x as i32 / 16; // num2 — cast BEFORE /16
            let row = center.y as i32 / 16; // num3
                                            // `:47769-47779` — the 3-row probe (solid OR liquid).
            for i in row..row + 3 {
                let blocked = env.world.tile(col, i).is_some_and(|t| {
                    (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
                });
                if blocked {
                    if npc.vy > 0.0 {
                        npc.vy *= 0.9;
                    }
                    npc.vy -= 0.2;
                }
            }
            // `:47780-47797` — the rising ceiling damp.
            if npc.vy < 0.0 {
                let mut ceiling = false; // flag
                for j in row..row + 30 {
                    let solid = env
                        .world
                        .tile(col, j)
                        .is_some_and(|t| t.active() && !t.inactive() && TILE_SOLID.has(t.kind));
                    if solid {
                        ceiling = true;
                        break;
                    }
                }
                if !ceiling && npc.vy < 0.0 {
                    npc.vy *= 0.9;
                }
            }
        }
        _ => {}
    }
    // `:47801-47804` — the vx-sign facing.
    if npc.vx != 0.0 {
        npc.facing = if npc.vx > 0.0 { 1 } else { -1 };
    }
    // `:47805-47808` — the wet snap.
    if npc.wet {
        npc.vy = -3.0;
    }
    // `:47809-47814` — the threat-scan cooldown.
    if npc.local_ai[1] > 0.0 {
        npc.local_ai[1] -= 1.0;
        return;
    }
    npc.local_ai[1] = 15.0;
    // `:47815-47834` — the threat scan.
    let mut count = 0.0f32; // num6
    let mut flee = Vec2::ZERO;
    for n in env.neighbors {
        if n.active && n.damage > 0 && !n.friendly {
            let d = rect_distance(
                n.x as i32, n.y as i32, n.width, n.height, center.x, center.y,
            );
            if d <= 100.0 {
                count += 1.0;
                let their_center =
                    Vec2::new(n.x + n.width as f32 / 2.0, n.y + n.height as f32 / 2.0);
                let away = center - their_center;
                let len = 1.0 / ((away.x * away.x + away.y * away.y) as f64).sqrt() as f32;
                flee += Vec2::new(away.x * len, away.y * len);
            }
        }
    }
    for p in env.players {
        if p.active {
            let d = rect_distance(
                p.x as i32, p.y as i32, p.width, p.height, center.x, center.y,
            );
            if d <= 150.0 {
                count += 1.0;
                let their_center =
                    Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
                let away = center - their_center;
                let len = 1.0 / ((away.x * away.x + away.y * away.y) as f64).sqrt() as f32;
                flee += Vec2::new(away.x * len, away.y * len);
            }
        }
    }
    if count > 0.0 {
        // `:47835-47851` — the flee kick.
        flee /= count;
        flee *= 2.0; // num7
        npc.vx += flee.x;
        npc.vy += flee.y;
        let v = Vec2::new(npc.vx, npc.vy);
        if length(v) > 16.0 {
            let capped = safe_normalize(v, Vec2::ZERO) * 16.0;
            npc.vx = capped.x;
            npc.vy = capped.y;
        }
        let new_anchor = center + flee * 10.0;
        npc.ai[1] = -10.0;
        npc.ai[0] = 1.0;
        npc.ai[2] = new_anchor.x;
        npc.ai[3] = new_anchor.y;
        npc.net_update = true;
    } else {
        // `:47852-47885` — the re-perch.
        if env.net_mode == 1 || length(anchor - center) >= 16.0 {
            return;
        }
        // `:47858-47862` — the anchor tile's roll (the C# reads the
        // tile UNGUARDED — a null tile would throw; the port's None
        // reads type 0, the documented widening).
        let anchor_tile = env.world.tile(npc.ai[2] as i32 / 16, npc.ai[3] as i32 / 16);
        let max_value = if anchor_tile.is_some_and(|t| t.kind == 519) {
            30
        } else {
            4
        };
        if env.rng.next_max(max_value) != 0 {
            return;
        }
        // `:47867-47875` — the cattail search (the dead cattailX/Y
        // pre-init from ai[2] twice is the C#'s own residue — the out
        // params overwrite both).
        if let Some((cx, cy)) = find_cattail_top(
            env.world,
            env.rng,
            npc.ai[2] as i32 / 16,
            npc.ai[3] as i32 / 16,
        ) {
            npc.ai[2] = (cx * 16) as f32;
            npc.ai[3] = (cy * 16) as f32;
            npc.net_update = true;
            return;
        }
        // `:47876-47884` — the ground search.
        let col = (center.x / 16.0) as i32; // num8 — the f32 divide
        let mut m = (center.y / 16.0) as i32;
        while !solid_tile_ij(env.world, col, m) && (m as f64) < env.world_surface {
            m += 1;
        }
        m -= env.rng.next_range(3, 6);
        npc.ai[2] = (col * 16) as f32;
        npc.ai[3] = (m * 16) as f32;
        npc.net_update = true;
    }
}

/// `NPC.FindFrame` cases 595-601 (`NPC.cs:72722-72738`) — the counter
/// pair (the style73 convention): the wrap at `frameCount * 3`, the
/// row `(int)counter / 3`. The `spriteDirection = direction` write is
/// the caller's.
pub fn dragonfly_find_frame(_frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    if counter >= (DRAGONFLY_FRAME_COUNT * 3) as f32 {
        counter = 0.0;
    }
    let row = (counter as i32 / 3) as u32;
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 595-shaped NPC (the style73 placeholder convention).
    fn fly(x: f32, y: f32) -> Npc {
        let d = &DRAGONFLY;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.local_ai[0] = 1.0; // skip the first tick by default
        n.local_ai[1] = 5.0; // skip the threat scan by default
        n.ai[0] = 1.0;
        n.ai[2] = x + 5.0;
        n.ai[3] = y + 5.0;
        n
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// `NPC.cs:16720-16738` — the shared row, field by field.
    #[test]
    fn w4_d33_s114_the_defaults_row() {
        let d = &DRAGONFLY;
        assert_eq!((d.width, d.height), (10, 10));
        assert_eq!(d.ai_style, 114);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 5));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.npc_slots, 0.25);
        assert!(d.no_gravity);
        assert_eq!(d.catch_item_base, 4334);
        assert_eq!(d.gold_rarity, 3);
        assert_eq!(DRAGONFLY_FRAME_COUNT, 4);
    }

    /// `:47718-47728` — the first tick: the centre latch, the 3-draw
    /// launch (circular angle+radius, edge angle), ai[0]=1.
    #[test]
    fn w4_d33_s114_the_first_tick_launch() {
        let w = air();
        let mut rng = UnifiedRandom::new(114);
        let mut oracle = UnifiedRandom::new(114);
        let mut n = fly(100.0, 100.0);
        n.local_ai[0] = 0.0;
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &[],
            net_mode: 0,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        // The launch: circular (2 floats) + edge (1 float).
        let a1 = std::f32::consts::PI * 2.0 * oracle.next_float();
        let r1 = oracle.next_float();
        let a2 = std::f32::consts::PI * 2.0 * oracle.next_float();
        let c = Vec2::new(
            (a1 as f64).cos() as f32 * 5.0 * r1,
            (a1 as f64).sin() as f32 * 3.0 * r1,
        );
        let e = Vec2::new(
            (a2 as f64).cos() as f32 * 5.0,
            (a2 as f64).sin() as f32 * 3.0,
        );
        let want = (c + e) * 0.4;
        assert_eq!(rng.peek(), oracle.peek(), "the launch draw order");
        assert_eq!(n.ai[2], 105.0);
        assert_eq!(n.ai[3], 105.0);
        assert_eq!(n.ai[0], 1.0);
        assert!(n.net_update);
        // The dart's ceiling damp runs the same tick: a rising vy
        // (want.y < 0) with no solid in the 30-row column takes ×0.9.
        assert_eq!(n.vx, want.x);
        let want_vy = if want.y < 0.0 { want.y * 0.9 } else { want.y };
        assert_eq!(n.vy, want_vy);
    }

    /// `:47731-47751` — the rest: the 0.94 damp, the 60+Next(60)
    /// re-dart, the >96 DirectionTo×3 / >16 ×1+jitter / else launch.
    #[test]
    fn w4_d33_s114_the_rest_redart() {
        let w = air();
        let mut rng = UnifiedRandom::new(114);
        let mut n = fly(100.0, 100.0);
        n.ai[0] = 0.0;
        n.ai[1] = 119.0; // one tick from any 60..119 threshold
        n.vx = 1.0;
        n.vy = 1.0;
        // The anchor 200 px right — the >96 arm, no extra draws.
        n.ai[2] = 305.0;
        n.ai[3] = 105.0;
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &[],
            net_mode: 0,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 0.0);
        let len = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((len - 3.0).abs() < 1e-4, "the ×3 re-dart, len {len}");
        assert!(n.vx > 0.0, "toward the right anchor");
        // The client never re-darts (but the counter still ticks).
        let mut n = fly(100.0, 100.0);
        n.ai[0] = 0.0;
        n.ai[1] = 119.0;
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &[],
            net_mode: 1,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0);
        assert_eq!(n.ai[1], 120.0);
    }

    /// `:47753-47765` — the dart timer: 4 ticks near the anchor, 200
    /// beyond 112.
    #[test]
    fn w4_d33_s114_the_dart_timer() {
        let w = air();
        let mut rng = UnifiedRandom::new(114);
        let mut n = fly(100.0, 100.0);
        n.ai[1] = 3.0;
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &[],
            net_mode: 0,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0, "the 4-tick flip");
        let mut n = fly(100.0, 100.0);
        n.ai[1] = 3.0;
        n.ai[2] = 305.0; // 200 px away
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &[],
            net_mode: 0,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 1.0, "the 200-tick hold");
        assert_eq!(n.ai[1], 4.0);
    }

    /// `:47815-47851` — the threat scan: the near player kicks the
    /// velocity away, re-anchors at Center+kick*10, ai[1]=-10.
    #[test]
    fn w4_d33_s114_the_threat_flee() {
        let w = air();
        let mut rng = UnifiedRandom::new(114);
        let players = [DragonflyPlayer {
            active: true,
            x: 120.0,
            y: 100.0,
            width: 20,
            height: 42,
        }];
        let mut n = fly(100.0, 100.0);
        n.local_ai[1] = 0.0; // the scan runs
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &players,
            net_mode: 0,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        assert_eq!(n.ai[1], -10.0);
        assert_eq!(n.ai[0], 1.0);
        assert!(n.vx < 0.0, "flees the right-side player");
        assert!(n.net_update);
        // The anchor moved left of the centre.
        assert!(n.ai[2] < 105.0);
        assert_eq!(n.local_ai[1], 15.0, "the cooldown re-arms");
    }

    /// `:47852-47885` — the re-perch: the non-519 anchor rolls Next(4);
    /// a passing roll with no cattail runs the ground search (the
    /// SolidTile floor minus Next(3,6)).
    #[test]
    fn w4_d33_s114_the_ground_reperch() {
        let mut w = air();
        // A floor at row 150 under the fly's column (centre x 105 →
        // col 6).
        for x in 0..w.width {
            for y in 150..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut rng = UnifiedRandom::new(114);
        let mut oracle = UnifiedRandom::new(114);
        let mut n = fly(100.0, 100.0);
        n.local_ai[1] = 0.0;
        // The anchor at the centre (within 16) on a non-519 tile.
        n.ai[2] = 105.0;
        n.ai[3] = 105.0;
        let mut env = Style114Env {
            world: &w,
            rng: &mut rng,
            neighbors: &[],
            players: &[],
            net_mode: 0,
            world_surface: 200.0,
        };
        style114_step(&mut n, &mut env);
        let roll = oracle.next_max(4);
        if roll == 0 {
            // FindCattailTop: the InWorld(6, 6, 31) gate REFUSES (6 <
            // 31) — no draws; then the ground search's Next(3,6).
            let back = oracle.next_range(3, 6);
            assert_eq!(rng.peek(), oracle.peek(), "the reperch draw order");
            assert_eq!(n.ai[2], (6 * 16) as f32);
            assert_eq!(n.ai[3], ((150 - back) * 16) as f32);
        } else {
            assert_eq!(rng.peek(), oracle.peek(), "the failed roll draws once");
        }
    }

    /// `NPC.cs:80180-80207` — FindCattailTop's reservoir: two cattail
    /// tops, the growing Next(num) picks; the border refusal.
    #[test]
    fn w4_d33_s114_the_cattail_reservoir() {
        let mut w = World::new("t".into(), 400, 300);
        // Two 519 tops (frameX 180) near (100, 100).
        for (x, y) in [(99, 99), (101, 101)] {
            let i = w.index(x, y);
            w.tiles[i] = Tile {
                kind: 519,
                frame_x: 180,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut rng = UnifiedRandom::new(114);
        let mut oracle = UnifiedRandom::new(114);
        let found = find_cattail_top(&w, &mut rng, 100, 100);
        // The scan order: (99,99) first — Next(1) always 0 → accept,
        // num 2; (101,101) — Next(2).
        oracle.next_max(1);
        let second = oracle.next_max(2);
        assert_eq!(rng.peek(), oracle.peek(), "the reservoir order");
        let want = if second == 0 { (101, 101) } else { (99, 99) };
        assert_eq!(found, Some(want));
        // The border refusal draws nothing.
        assert_eq!(find_cattail_top(&w, &mut rng, 5, 5), None);
        assert_eq!(rng.peek(), oracle.peek());
    }

    /// `NPC.cs:72722-72738` — the 12-wrap counter, row = counter/3.
    #[test]
    fn w4_d33_s114_the_find_frame_rows() {
        assert_eq!(dragonfly_find_frame(0, 0.0), (0, 1.0));
        assert_eq!(dragonfly_find_frame(0, 2.0), (1, 3.0));
        assert_eq!(dragonfly_find_frame(1, 5.0), (2, 6.0));
        assert_eq!(dragonfly_find_frame(3, 10.0), (3, 11.0));
        assert_eq!(dragonfly_find_frame(3, 11.0), (0, 0.0), "the 12 wrap");
    }

    /// The stream parity: 60 ticks through the rest/dart cycle match a
    /// cloned twin bit-for-bit (the style101 twin precedent).
    #[test]
    fn w4_d33_s114_the_run_is_twin_stable() {
        let w = air();
        let mut a = fly(280.0, 280.0);
        a.local_ai[0] = 0.0;
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(595);
        let mut rng_b = UnifiedRandom::new(595);
        for tick in 0..60 {
            let mut env_a = Style114Env {
                world: &w,
                rng: &mut rng_a,
                neighbors: &[],
                players: &[],
                net_mode: 0,
                world_surface: 200.0,
            };
            style114_step(&mut a, &mut env_a);
            let mut env_b = Style114Env {
                world: &w,
                rng: &mut rng_b,
                neighbors: &[],
                players: &[],
                net_mode: 0,
                world_surface: 200.0,
            };
            style114_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "state @{tick}");
        }
    }
}
