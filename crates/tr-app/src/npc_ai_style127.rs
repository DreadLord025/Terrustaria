//! The aiStyle-127 family — the Pals (the Palworld crossover critters)
//! (`NPC.AI_127_Pal`, `NPC.cs:42929-43024`, dispatched at
//! `:41128-41131`). W4-d33, research/25 row 127, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has TWO vanilla members: the PalworldCattivaDistressed
//! 695 / PalworldFoxsparksDistressed 696 (`ID/NPCID.cs:12067-12069`;
//! `SetDefaults` rows `NPC.cs:17752-17779`). The census of
//! `this.aiStyle = 127;` returns exactly those two hits (`:17757`,
//! `:17771`) — no projectile-space user. The port's roster has no pal
//! kind — the module lands UNWIRED (the `npc_ai_style73.rs`
//! precedent).
//!
//! ## The machine
//!
//! The distressed pal: it spawns two GoblinArcher (111) captors on a
//! ritual floor, waits for them to die, then cheers when the player
//! approaches and drops a reward.
//!
//! The head (`:42931-42932`): TargetClosest; `spriteDirection =
//! -direction`.
//!
//! The non-client spawn (`:42933-42977`): the `localAI[0] == 0` latch
//! — `CultistRitual.CheckFloor2(Center)` finds the two captor spots;
//! failure → `life = 0`, HitEffect, `active = false`, RETURN. Success
//! → two `NewNPC(111)` at `spot.X*16+8, spot.Y*16-48`, the
//! `ai[1+i] = slot+1` back-references, the captors' `ai[3] =
//! -(whoAmI+1)` owner payload + netUpdate. The `ai[0] == 0` wait: both
//! captor slots unpacked ([`pal_try_unpack`]) — both gone → state 1;
//! else `timeLeft = activeTime` (750). The `ai[0] == 1 && Distance <
//! 100` cheer → state 2 (ai[1]/ai[2] zero).
//!
//! The pain cry (`:42978-42989`): state 0 with `soundDelay == 0` —
//! `soundDelay = Next(600, 1800)`, the per-type pain trackable.
//!
//! The cheer (`:42990-43014`): state 2 — the `soundDelay != -2` latch
//! plays the joy trackable; `++ai[1] >= 120` → the non-client
//! GiveRewerd (the item drop), `life = 0`, `active = false`, RETURN.
//!
//! The tail (`:43015-43023`): the 696 light `(0.5, 0.3, 0.1)`;
//! `vx *= 0.93`; the DOUBLE-compare `(-0.1, 0.1)` zero-snap.
//!
//! [`pal_give_reward`] (`AI_127_Pal_GiveRewerd`, `:43026-43034`): the
//! `Item.NewItem(Center, 5663/5664, 1, -1)` — the outcome seam (the
//! wiring's item-drop pass). [`pal_summon_attacker`]
//! (`AI_127_Pal_SummonAttacker`, `:43036-43039`) is the C#'s `return 0`
//! stub — documented, never called by the arm.
//!
//! ## RNG order (by construction)
//!
//! The pain cry: Next(600, 1800). The spawn, the unpack, the cheer and
//! the damps draw NOTHING; TargetClosest is RNG-free; CheckFloor2 is a
//! pure tile scan.
//!
//! ## FindFrame cases 695/696
//!
//! `NPC.cs:66707-66766`: 695 is a pure ai[0] → row map (1 → 2, 2 → 1,
//! else 0; frameCount 3). 696 is a raw-`frame.Y` band machine (the
//! ai[0]==1 0..8 cycle at 4 ticks, the ai[0]==2 45..53 cycle at 4, the
//! else 9..17 cycle at 10; frameCount 1 — the sheet is a single tall
//! strip, the bands are pixel offsets). [`pal_find_frame`] ports both
//! (the style73 convention, the raw-Y form for 696).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::tiledata::{SET_PLATFORMS, TILE_SOLID, TILE_SOLID_TOP};
use tr_core::{UnifiedRandom, World};

/// The shared type-695/696 `SetDefaults` rows
/// (`NPC.cs:17752-17779`), verbatim — only the sound names differ.
pub struct PalDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub rarity: i32,
    pub friendly: bool,
}

/// `NPC.cs:17752-17779` — the two rows (the stats are identical).
pub const PAL: PalDefaults = PalDefaults {
    width: 18,
    height: 18,
    ai_style: 127,
    damage: 0,
    defense: 15,
    life_max: 250,
    knock_back_resist: 0.5,
    rarity: 1,
    friendly: true,
};

/// The per-type trackable sound names (`SoundID.cs:1714-1721`).
pub const PAL_SOUNDS: [(&str, &str); 2] = [
    ("pal_cattiva_pain", "pal_cattiva_joy"),   // 695
    ("pal_foxparks_pain", "pal_foxparks_joy"), // 696
];

/// The per-type reward item (`:43028-43032`): 695 → 5663, 696 → 5664.
pub const PAL_REWARDS: [i32; 2] = [5663, 5664];

/// `Main.npcFrameCount[695]` = 3, `[696]` = 1 (`Main.cs:67153`).
pub const PAL_FRAME_COUNTS: [u32; 2] = [3, 1];

/// `NPC.activeTime` (`NPC.cs:95186`) — the wait-state timeLeft reset.
pub const ACTIVE_TIME: i32 = 750;

/// The per-tick environment.
pub struct Style127Env<'a> {
    pub world: &'a World,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.netMode`.
    pub net_mode: i32,
    /// The two captor slots' active flags, indexed by `(int)ai[1+i]-1`
    /// (the [`pal_try_unpack`] view). The wiring fills them from the
    /// pool; `None` entries stand for out-of-range indices.
    pub captors: &'a [Option<bool>],
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// A `NPC.NewNPC` request — the captor spawn (`:42949`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalCaptorSpawn {
    /// The vanilla type — always 111 (the GoblinArcher).
    pub npc_type: i32,
    /// `spot.X * 16 + 8`.
    pub x: i32,
    /// `spot.Y * 16 - 48`.
    pub y: i32,
    /// The captor's `ai[3] = -(whoAmI + 1)` owner payload.
    pub ai3: f32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style127Outcome {
    /// The two captor spawn requests in C# order (the first tick).
    pub spawns: Vec<PalCaptorSpawn>,
    /// The death flag (`:42941-42944` the floor failure / `:43010-43012`
    /// the cheer expiry): `life = 0` + `active = false`. The caller
    /// owes the pool removal (the HitEffect on the floor failure).
    pub died: bool,
    /// The floor-failure HitEffect (`:42942`) — the caller runs it
    /// (the 695/696 HitEffect arms are the generic dust, out of scope).
    pub hit_effect: bool,
    /// The `SoundEngine.PlaySound` trackable cues — `(name, x, y)`.
    pub sounds: Vec<(&'static str, f32, f32)>,
    /// The GiveRewerd item drop (`:43033`) — `(item_type, x, y)`; the
    /// wiring's `Item.NewItem(Center, type, 1, -1)` pass.
    pub reward: Option<(i32, f32, f32)>,
    /// The 696 `Lighting.AddLight(Center, 0.5, 0.3, 0.1)` (`:43017`).
    pub light: Option<(f32, f32, [f32; 3])>,
}

/// XNA `Vector2.Distance` — the f32 sum widened to f64 for the root.
fn distance(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    ((d.x * d.x + d.y * d.y) as f64).sqrt() as f32
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
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out of bounds = solid; a null tile
/// returns false for the whole call; active && !inactive && solid &&
/// !solidTop counts.
fn solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for i in x0..=x1 {
        for j in y0..=y1 {
            match world.tile(i, j) {
                None => return false,
                Some(t) => {
                    if t.active()
                        && !t.inactive()
                        && TILE_SOLID.has(t.kind)
                        && !TILE_SOLID_TOP.has(t.kind)
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// `CultistRitual.CheckFloor2` (`CultistRitual.cs:133-162`): the two
/// captor spots at `Centre tile ± 6` columns (i = -3, +3 step 2, the
/// -1/+1 skip), each dropping j = -5..12 to the first
/// solid-or-platform cell with the headroom test. Returns the two
/// spots only when BOTH found. Draw-free.
fn check_floor2(world: &World, center: Vec2) -> Option<[(i32, i32); 2]> {
    let px = center.x as i32 >> 4;
    let py = center.y as i32 >> 4;
    let mut spots: [(i32, i32); 2] = [(0, 0); 2];
    let mut found = 0;
    let mut i = -3;
    while i <= 3 {
        if i == -1 || i == 1 {
            i += 2;
            continue;
        }
        for j in -5..12 {
            let x = px + i * 2;
            let y = py + j;
            let platform = world.tile(x, y).is_some_and(|t| SET_PLATFORMS.has(t.kind));
            let floor = solid_tile_ij(world, x, y) || platform;
            if floor {
                // The headroom: !SolidTiles(x-1..x+1, y-3..y-1) OR the
                // three-column carve.
                let full = solid_tiles(world, x - 1, x + 1, y - 3, y - 1);
                let carve = !solid_tiles(world, x, x, y - 3, y - 1)
                    && !solid_tiles(world, x + 1, x + 1, y - 3, y - 2)
                    && !solid_tiles(world, x - 1, x - 1, y - 3, y - 2);
                if !full || carve {
                    spots[found] = (x, y);
                    found += 1;
                    break;
                }
            }
        }
        i += 2;
    }
    if found != 2 {
        None
    } else {
        Some(spots)
    }
}

/// `AI_127_Pal_TryUnpackNPC` (`NPC.cs:43041-43054`): the `(int)ai-1`
/// slot must be in range AND active. The env's `captors` slice carries
/// the per-slot active flag (the wiring resolves the index).
fn pal_try_unpack(ai_value: f32, captors: &[Option<bool>]) -> bool {
    let slot = ai_value as i32 - 1;
    if slot < 0 {
        return false;
    }
    match captors.get(slot as usize) {
        Some(Some(active)) => *active,
        _ => false,
    }
}

/// One fixed step of the aiStyle-127 arm (`NPC.cs:42929-43024`), in C#
/// order. `npc_type` is the vanilla id (695/696) until the roster
/// kinds land. The caller owns the movement integration and drains
/// [`Style127Outcome`].
pub fn style127_step(npc: &mut Npc, npc_type: i32, env: &mut Style127Env<'_>) -> Style127Outcome {
    let mut out = Style127Outcome::default();
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let idx = if npc_type == 696 { 1 } else { 0 };
    // `:42931-42932` — the head.
    if !env.players.is_empty() {
        target_closest(npc, env.world, env.players, true, 127, env.ticker);
    }
    npc.sprite_dir = -npc.facing;
    // `:42933-42977` — the non-client spawn + wait.
    if env.net_mode != 1 {
        if npc.local_ai[0] == 0.0 {
            npc.local_ai[0] = 1.0;
            match check_floor2(env.world, center) {
                None => {
                    npc.hp = 0;
                    out.hit_effect = true;
                    out.died = true;
                    return out;
                }
                Some(spots) => {
                    for (i, &(sx, sy)) in spots.iter().enumerate() {
                        out.spawns.push(PalCaptorSpawn {
                            npc_type: 111,
                            x: sx * 16 + 8,
                            y: sy * 16 - 48,
                            ai3: -((npc.slot as i32) + 1) as f32,
                        });
                        // `ai[1+i] = slot+1` — the wiring writes the
                        // resolved slot index back (the spawn request
                        // carries the position; the slot is the
                        // wiring's resolution).
                        let _ = i;
                    }
                    npc.net_update = true;
                }
            }
        }
        if npc.ai[0] == 0.0 {
            let c1 = pal_try_unpack(npc.ai[1], env.captors);
            let c2 = pal_try_unpack(npc.ai[2], env.captors);
            if !c1 && !c2 {
                npc.ai[0] = 1.0;
                npc.net_update = true;
            } else {
                npc.despawn_timer = ACTIVE_TIME; // timeLeft = activeTime
            }
        }
        if npc.ai[0] == 1.0 {
            let p = target_of(npc, env.players);
            let pc = Vec2::new(p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32);
            if distance(center, pc) < 100.0 {
                npc.ai[0] = 2.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.net_update = true;
            }
        }
    }
    // `:42978-42989` — the pain cry.
    if npc.ai[0] == 0.0 && npc.sound_delay == 0 {
        npc.sound_delay = env.rng.next_range(600, 1800);
        out.sounds.push((PAL_SOUNDS[idx].0, center.x, center.y));
    }
    // `:42990-43014` — the cheer.
    if npc.ai[0] == 2.0 {
        if npc.sound_delay != -2 {
            npc.sound_delay = -2;
            out.sounds.push((PAL_SOUNDS[idx].1, center.x, center.y));
        }
        npc.ai[1] += 1.0; // `++this.ai[1]`
        if npc.ai[1] >= 120.0 {
            if env.net_mode != 1 {
                // GiveRewerd — the item drop.
                out.reward = Some((PAL_REWARDS[idx], center.x, center.y));
            }
            npc.hp = 0;
            out.died = true;
            return out;
        }
    }
    // `:43015-43018` — the 696 light.
    if npc_type == 696 {
        out.light = Some((center.x, center.y, [0.5, 0.3, 0.1]));
    }
    // `:43019-43023` — the X damp + the double-compare zero-snap.
    npc.vx *= 0.93;
    if (npc.vx as f64) > -0.1 && (npc.vx as f64) < 0.1 {
        npc.vx = 0.0;
    }
    out
}

/// `NPC.FindFrame` cases 695/696 (`NPC.cs:66707-66766`). 695 is the
/// pure ai[0] → row map (returns the row, the counter untouched). 696
/// is the raw-`frame.Y` band machine — `frame_y` is the RAW pixel
/// offset (the single-strip sheet), the counter advances per the
/// band's rate. Returns `(frame_y, counter)` for 696 and `(row, 0)`
/// for 695 (the caller multiplies 695's row by the sheet height).
pub fn pal_find_frame(npc_type: i32, frame_y: i32, frame_counter: f32, ai0: f32) -> (i32, f32) {
    if npc_type == 695 {
        // `:66707-66720` — the pure row map.
        let row = if ai0 == 1.0 {
            2
        } else if ai0 == 2.0 {
            1
        } else {
            0
        };
        return (row, frame_counter);
    }
    // 696 — the raw-Y band machine.
    let mut y = frame_y;
    let mut counter = frame_counter;
    if ai0 == 1.0 {
        // `:66722-66737` — the 0..8 cycle at 4 ticks.
        if !(0..8).contains(&y) {
            y = 0;
        }
        counter += 1.0;
        if counter >= 4.0 {
            counter = 0.0;
            y += 1;
            if y >= 8 {
                y = 0;
            }
        }
    } else if ai0 == 2.0 {
        // `:66738-66753` — the 45..53 cycle at 4 ticks.
        if !(45..53).contains(&y) {
            y = 45;
        }
        counter += 1.0;
        if counter >= 4.0 {
            counter = 0.0;
            y += 1;
            if y >= 53 {
                y = 45;
            }
        }
    } else {
        // `:66754-66765` — the 9..17 cycle at 10 ticks.
        if !(9..17).contains(&y) {
            y = 9;
        }
        counter += 1.0;
        if counter >= 10.0 {
            counter = 0.0;
            y += 1;
            if y >= 17 {
                y = 9;
            }
        }
    }
    (y, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 695-shaped NPC (the style73 placeholder convention).
    fn pal(x: f32, y: f32) -> Npc {
        let d = &PAL;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.facing = 1;
        n.target = 0;
        n.slot = 3;
        n.local_ai[0] = 1.0; // skip the spawn by default
        n.ai[0] = 1.0; // the wait-done state by default
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

    /// `NPC.cs:17752-17779` — the shared row, field by field.
    #[test]
    fn w4_d33_s127_the_defaults_row() {
        let d = &PAL;
        assert_eq!((d.width, d.height), (18, 18));
        assert_eq!(d.ai_style, 127);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 15, 250));
        assert_eq!(d.knock_back_resist, 0.5);
        assert_eq!(d.rarity, 1);
        assert!(d.friendly);
        assert_eq!(PAL_REWARDS, [5663, 5664]);
        assert_eq!(PAL_FRAME_COUNTS, [3, 1]);
        assert_eq!(ACTIVE_TIME, 750);
    }

    /// `:42931-42932` — the head: spriteDirection = -direction.
    #[test]
    fn w4_d33_s127_the_head_facing() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(127);
        let mut n = pal(100.0, 100.0);
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let _ = style127_step(&mut n, 695, &mut env);
        assert_eq!(n.sprite_dir, -n.facing);
    }

    /// `:42956-42969` — the wait: both captors gone → state 1; else
    /// timeLeft = 750.
    #[test]
    fn w4_d33_s127_the_captor_wait() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(127);
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 0.0;
        n.ai[1] = 1.0; // slot 0
        n.ai[2] = 2.0; // slot 1
                       // Both captors active → the wait holds.
        let captors = [Some(true), Some(true)];
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &captors,
            ticker: 7,
        };
        let _ = style127_step(&mut n, 695, &mut env);
        assert_eq!(n.ai[0], 0.0);
        assert_eq!(n.despawn_timer, 750);
        // Both gone → state 1.
        let captors = [Some(false), Some(false)];
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 0.0;
        n.ai[1] = 1.0;
        n.ai[2] = 2.0;
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &captors,
            ticker: 7,
        };
        let _ = style127_step(&mut n, 695, &mut env);
        assert_eq!(n.ai[0], 1.0);
        assert!(n.net_update);
    }

    /// `:42970-42976` — the near-player cheer → state 2.
    #[test]
    fn w4_d33_s127_the_near_cheer() {
        let w = air();
        let players = [cand(150.0, 100.0)]; // ~50 px
        let mut rng = UnifiedRandom::new(127);
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 1.0;
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let _ = style127_step(&mut n, 695, &mut env);
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(n.ai[2], 0.0);
    }

    /// `:42978-42989` — the pain cry: Next(600,1800) + the per-type
    /// trackable.
    #[test]
    fn w4_d33_s127_the_pain_cry() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(127);
        let mut oracle = UnifiedRandom::new(127);
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 0.0;
        n.sound_delay = 0;
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let out = style127_step(&mut n, 695, &mut env);
        let delay = oracle.next_range(600, 1800);
        assert_eq!(rng.peek(), oracle.peek(), "the pain draw");
        assert_eq!(n.sound_delay, delay);
        assert!(out.sounds.iter().any(|&(n, _, _)| n == "pal_cattiva_pain"));
        // 696 uses the foxparks name.
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 0.0;
        n.sound_delay = 0;
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let out = style127_step(&mut n, 696, &mut env);
        assert!(out.sounds.iter().any(|&(n, _, _)| n == "pal_foxparks_pain"));
    }

    /// `:42990-43014` — the cheer expiry: the joy cue once, the 120
    /// tick → the reward + death.
    #[test]
    fn w4_d33_s127_the_cheer_reward() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(127);
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[1] = 119.0; // ++ → 120
        n.sound_delay = 5;
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let out = style127_step(&mut n, 695, &mut env);
        assert!(out.died);
        assert_eq!(n.hp, 0);
        assert_eq!(n.sound_delay, -2);
        assert!(out.sounds.iter().any(|&(n, _, _)| n == "pal_cattiva_joy"));
        assert_eq!(out.reward.map(|(t, _, _)| t), Some(5663));
        // 696 → 5664.
        let mut n = pal(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[1] = 119.0;
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let out = style127_step(&mut n, 696, &mut env);
        assert_eq!(out.reward.map(|(t, _, _)| t), Some(5664));
        assert!(out.light.is_some(), "the 696 light");
    }

    /// `:43019-43023` — the X damp + the double-compare zero-snap.
    #[test]
    fn w4_d33_s127_the_x_damp() {
        let w = air();
        let players = [cand(100000.0, 100000.0)];
        let mut rng = UnifiedRandom::new(127);
        let mut n = pal(100.0, 100.0);
        n.vx = 0.1; // 0.1*0.93 = 0.093 < 0.1 → snap 0
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let _ = style127_step(&mut n, 695, &mut env);
        assert_eq!(n.vx, 0.0);
        let mut n = pal(100.0, 100.0);
        n.vx = 2.0; // 2*0.93 = 1.86, no snap
        let mut env = Style127Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            net_mode: 0,
            captors: &[],
            ticker: 7,
        };
        let _ = style127_step(&mut n, 695, &mut env);
        assert!((n.vx - 1.86).abs() < 1e-6);
    }

    /// `NPC.cs:66707-66766` — the 695 row map + the 696 band cycles.
    #[test]
    fn w4_d33_s127_the_find_frame() {
        // 695: the pure row map.
        assert_eq!(pal_find_frame(695, 0, 0.0, 1.0).0, 2);
        assert_eq!(pal_find_frame(695, 0, 0.0, 2.0).0, 1);
        assert_eq!(pal_find_frame(695, 0, 0.0, 0.0).0, 0);
        // 696: the 0..8 cycle at 4 ticks.
        assert_eq!(pal_find_frame(696, 0, 3.0, 1.0), (1, 0.0));
        assert_eq!(pal_find_frame(696, 7, 3.0, 1.0), (0, 0.0), "the 8 wrap");
        // The out-of-band reset (40 → 0, then the counter ticks).
        assert_eq!(pal_find_frame(696, 40, 0.0, 1.0), (0, 1.0));
        // The 45..53 cycle.
        assert_eq!(pal_find_frame(696, 52, 3.0, 2.0), (45, 0.0), "the 53 wrap");
        // The 9..17 cycle at 10 ticks.
        assert_eq!(pal_find_frame(696, 16, 9.0, 0.0), (9, 0.0), "the 17 wrap");
    }

    /// `CultistRitual.cs:133-162` — CheckFloor2 finds two spots on a
    /// flat floor, none in the air.
    #[test]
    fn w4_d33_s127_the_floor_check() {
        let mut w = air();
        // A flat floor at row 150.
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
        // The centre at tile (100, 140) — the spots at ±6 columns.
        // The scan drops from j=-5 (row 135) down to the floor at 150
        // (j=10, within the -5..12 range).
        let center = Vec2::new(100.0 * 16.0 + 8.0, 140.0 * 16.0 + 8.0);
        let spots = check_floor2(&w, center);
        assert!(spots.is_some());
        let spots = spots.unwrap();
        assert_eq!(spots[0].0, 100 - 6);
        assert_eq!(spots[1].0, 100 + 6);
        assert_eq!(spots[0].1, 150, "the floor row");
        // The air world finds nothing.
        let w = air();
        assert!(check_floor2(&w, center).is_none());
    }

    /// The stream parity: 60 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s127_the_run_is_twin_stable() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut a = pal(100.0, 100.0);
        a.ai[0] = 0.0;
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(695);
        let mut rng_b = UnifiedRandom::new(695);
        for tick in 0..60 {
            let mut env_a = Style127Env {
                world: &w,
                rng: &mut rng_a,
                players: &players,
                net_mode: 0,
                captors: &[],
                ticker: 7,
            };
            let _ = style127_step(&mut a, 695, &mut env_a);
            let mut env_b = Style127Env {
                world: &w,
                rng: &mut rng_b,
                players: &players,
                net_mode: 0,
                captors: &[],
                ticker: 7,
            };
            let _ = style127_step(&mut b, 695, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "state @{tick}");
        }
    }
}
