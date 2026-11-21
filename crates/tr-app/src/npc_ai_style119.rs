//! The aiStyle-119 family — the Dandelion
//! (`NPC.AI_119_Dandelion`, `NPC.cs:46984-47069`, dispatched at
//! `:41096-41099`). W4-d33, research/25 row 119, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the Dandelion 628
//! (`NPCID.Dandelion`, `ID/NPCID.cs:11933`; `SetDefaults` row
//! `NPC.cs:17072-17084`). The census of `this.aiStyle = 119;` returns
//! exactly that one hit (`:17076`) — no projectile-space user. The
//! port's roster has no dandelion kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent).
//!
//! ## The machine
//!
//! The ROOTED wind-turret: it never moves (no velocity write anywhere
//! in the arm). It only fires seeds downwind at a nearby player.
//!
//! The despawn clamp (`:46986-46989`): a non-happy-windy day caps
//! `timeLeft` at 10 (the port's `despawn_timer`).
//!
//! The downwind pick (`:46993-47002`):
//! `NPCUtils.TargetClosestDownwindFromNPC(this, 600)` — the
//! player-only search with the `DownwindFromNPC` filter (active,
//! !dead, `|dy| < 100`, `|dx| < 600`, `sign(dx) == sign(wind)`); the
//! `flag` re-checks the SAME conditions against the picked target's
//! INTEGER hitbox centre (`num = dx`, `num2 = |dx|`).
//!
//! State 1 (`:47003-47058`, the armed turret): `localAI[0] = 0`, then
//! `num2 < 500` sets it 1 (the close-range lock). The client RETURNS.
//! `!flag` disarms to 0. Else the `localAI[0] != 1` RETURNS (the
//! far-but-downwind hold); `localAI[1]++`; past 80 disarms to 0; at
//! exactly 40 fires `1 + Next(3)` seeds — each
//! `num3 = sign(num)` (the -1 default when num == 0), the offset
//! `(num3 * Next(-2, 10), 10 + Next(-6, 6))`, the velocity
//! `(2*num3, -2) + offset*0.25` with the `vy > -3` floor, the spawn at
//! `Center + offset + (num3*6, 0)` — type 836 DandelionSeed, damage 7,
//! knockback 1, `ai1 = target`. netUpdate.
//!
//! State 0 (`:47059-47068`, the idle): both localAI zero; the
//! non-client `& flag` (the NON-short-circuit `&`) arms to 1.
//!
//! ## RNG order (by construction)
//!
//! The seed burst: Next(3) (the count), then per seed Next(-2, 10)
//! and Next(-6, 6). The downwind search and the state flips draw
//! NOTHING.
//!
//! ## FindFrame case 628
//!
//! `NPC.cs:77103-77201`: the `spriteDirection = !(WindForVisuals > 0)
//! ? 1 : -1` write; the bestiary-dummy arm (the documented skip); the
//! four LIVE arms — ai[0]==0 the 6-frame sway (the row-6 hold, the
//! `1 + 0.5*|wind|` counter over 8), localAI[0]==0 the 4-tick
//! 6↔7..10 cycle, localAI[0]==1 the `Lerp(7, 20, counter/80)` charge
//! strip (the >19 cap, the >16 subtract-9 fold, the 80 wrap).
//! `Main.npcFrameCount[628] = 17`. [`dandelion_find_frame`] ports the
//! live machine (the style73 convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::TargetCandidate;
use glam::Vec2;
use tr_core::{UnifiedRandom, World};

/// The type-628 `SetDefaults` row (`NPC.cs:17072-17084`), verbatim.
pub struct DandelionDefaults {
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
    pub knock_back_resist: f32,
    pub value: f32,
}

/// `NPC.cs:17072-17084` — `else if (this.type == 628)`.
pub const DANDELION: DandelionDefaults = DandelionDefaults {
    width: 22,
    height: 40,
    ai_style: 119,
    damage: 15,
    defense: 0,
    life_max: 50,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    knock_back_resist: 0.0,
    value: 50.0,
};

/// `Main.npcFrameCount[628]` = 17 (`Main.cs:67146`, the 620-629 row:
/// `21, 1, 1, 1, 11, 12, 6, 6, 17, 6` — index 628 is the 9th entry).
pub const DANDELION_FRAME_COUNT: u32 = 17;

/// The per-tick environment.
pub struct Style119Env<'a> {
    pub world: &'a World,
    pub rng: &'a mut UnifiedRandom,
    /// The `Main.player[]` slots as the downwind search sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.windSpeedCurrent` — the downwind sign gate (also
    /// `WindForVisuals`, `Main.cs:2823`).
    pub wind_speed_current: f32,
    /// `Main.IsItAHappyWindyDay` (`Main.cs:2903`,
    /// `_shouldUseWindyDayMusic`) — the despawn clamp.
    pub happy_windy_day: bool,
    /// `Main.netMode`.
    pub net_mode: i32,
}

/// A `Projectile.NewProjectile` request — the DandelionSeed (836).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DandelionSeed {
    /// `ProjectileID.DandelionSeed` = 836.
    pub projectile_type: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knock_back: f32,
    /// The `ai1 = this.target` spawn argument (`:47053`).
    pub ai1: f32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style119Outcome {
    /// The seed volley in C# order (empty on every other tick).
    pub shots: Vec<DandelionSeed>,
}

/// `NPCUtils.TargetClosestDownwindFromNPC` (`NPCUtils.cs:300-310`)
/// over the env slice: the player-only `DownwindFromNPC` filter
/// (`:38-48` — active, !dead, `|dy| < 100`, `|dx| < 600`, the
/// wind-sign match), the Distance − aggro pick (the direction-0
/// no-aggro +1000 penalty), then `ShouldFaceTarget` + `FaceTarget`.
/// The tank-pet steal is inert (the session_candidate convention).
/// RNG-free.
fn target_closest_downwind(npc: &mut Npc, env: &Style119Env<'_>) {
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    let mut best_adj = f32::MAX;
    let mut best_dist = f32::MAX;
    let mut best: i32 = -1;
    for (j, p) in env.players.iter().enumerate() {
        if !p.active || p.dead || p.ghost {
            continue;
        }
        let pcx = p.x + (p.width / 2) as f32;
        let pcy = p.y + (p.height / 2) as f32;
        let dx = pcx - center.x;
        let dy = (pcy - center.y).abs();
        let adx = dx.abs();
        // The DownwindFromNPC filter (NPCUtils.cs:43-47).
        if !(dy < 100.0
            && adx < 600.0
            && ((dx > 0.0 && env.wind_speed_current > 0.0)
                || (dx < 0.0 && env.wind_speed_current < 0.0)))
        {
            continue;
        }
        let d = ((dx * dx + (pcy - center.y) * (pcy - center.y)) as f64).sqrt() as f32;
        let mut adj = d - p.aggro as f32;
        if p.npc_type_no_aggro && npc.facing == 0 {
            adj += 1000.0;
        }
        if adj < best_adj {
            best = j as i32;
            best_adj = adj;
            best_dist = d;
        }
    }
    if best == -1 {
        return; // FoundTarget false — nothing writes.
    }
    let p = &env.players[best as usize];
    npc.target = best;
    npc.target_rect = (p.x as i32, p.y as i32, p.width, p.height);
    // ShouldFaceTarget (NPC.cs:77619-77641) — the player arm.
    let face = if p.dead || (p.npc_type_no_aggro && npc.facing != 0) {
        false
    } else {
        let stealth_band = (p.width + p.height + w as i32 + h as i32) as f32 / 4.0 + 800.0;
        if p.item_animation == 0 && p.aggro < 0 && best_dist > stealth_band {
            if npc.old_target >= 0 {
                npc.old_target >= 255
            } else {
                true
            }
        } else {
            true
        }
    };
    if face {
        // FaceTarget (NPC.cs:77646-77650) — the INTEGER rect centre.
        let r = npc.target_rect;
        let rcx = (r.0 + r.2 / 2) as f32;
        let rcy = (r.1 + r.3 / 2) as f32;
        npc.facing = if rcx < center.x { -1 } else { 1 };
        npc.direction_y = if rcy < center.y { -1 } else { 1 };
    }
}

/// One fixed step of the aiStyle-119 arm (`NPC.cs:46984-47069`), in C#
/// order. The dandelion is ROOTED — the arm never writes velocity; the
/// caller drains [`Style119Outcome`] only.
pub fn style119_step(npc: &mut Npc, env: &mut Style119Env<'_>) -> Style119Outcome {
    let mut out = Style119Outcome::default();
    // `:46986-46989` — the despawn clamp.
    if !env.happy_windy_day && npc.despawn_timer > 10 {
        npc.despawn_timer = 10;
    }
    let (w, h) = npc.size();
    let center = Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0);
    // `:46993-47002` — the downwind pick + the flag re-check.
    target_closest_downwind(npc, env);
    let mut flag = false;
    let mut num = 0.0f32; // dx
    let mut num2 = 0.0f32; // |dx|
    if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize];
        if p.active && !p.dead && !p.ghost {
            // The INTEGER hitbox centre (targetData.Hitbox.Center).
            let rcx = (p.x as i32 + p.width / 2) as f32;
            let rcy = (p.y as i32 + p.height / 2) as f32;
            num = rcx - center.x;
            num2 = num.abs();
            flag = (rcy - center.y).abs() < 100.0
                && num2 < 600.0
                && ((num > 0.0 && env.wind_speed_current > 0.0)
                    || (num < 0.0 && env.wind_speed_current < 0.0));
        }
    }
    if npc.ai[0] == 1.0 {
        // `:47003-47058` — the armed turret.
        npc.local_ai[0] = 0.0;
        if num2 < 500.0 {
            npc.local_ai[0] = 1.0;
        }
        if env.net_mode == 1 {
            return out;
        }
        if !flag {
            npc.ai[0] = 0.0;
            npc.net_update = true;
        } else {
            if npc.local_ai[0] != 1.0 {
                return out;
            }
            npc.local_ai[1] += 1.0;
            if npc.local_ai[1] > 80.0 {
                npc.ai[0] = 0.0;
                npc.net_update = true;
            } else {
                if npc.local_ai[1] != 40.0 {
                    return out;
                }
                // `:47037-47054` — the seed burst.
                let count = 1 + env.rng.next_max(3);
                for _ in 0..count {
                    let mut num3 = -1;
                    if num > 0.0 {
                        num3 = 1;
                    }
                    let offset = Vec2::new(
                        num3 as f32 * env.rng.next_range(-2, 10) as f32,
                        10.0 + env.rng.next_range(-6, 6) as f32,
                    );
                    let mut vel = Vec2::new(2.0 * num3 as f32, -2.0);
                    vel += offset * 0.25;
                    if vel.y > -3.0 {
                        vel.y = -3.0;
                    }
                    let mut spawn = center + offset;
                    spawn.x += num3 as f32 * 6.0;
                    out.shots.push(DandelionSeed {
                        projectile_type: 836,
                        x: spawn.x,
                        y: spawn.y,
                        vx: vel.x,
                        vy: vel.y,
                        damage: 7,
                        knock_back: 1.0,
                        ai1: npc.target as f32,
                    });
                }
                npc.net_update = true;
            }
        }
    } else if npc.ai[0] == 0.0 {
        // `:47059-47068` — the idle.
        npc.local_ai[0] = 0.0;
        npc.local_ai[1] = 0.0;
        if (env.net_mode != 1) & flag {
            npc.ai[0] = 1.0;
            npc.net_update = true;
        }
    }
    out
}

/// `NPC.FindFrame` case 628 (`NPC.cs:77103-77201`) — the LIVE machine
/// (the bestiary-dummy arm is the documented skip). `frame_row` is
/// `frame.Y / num`; `wind` is `Main.WindForVisuals`. The
/// `spriteDirection = !(wind > 0) ? 1 : -1` write is the caller's.
pub fn dandelion_find_frame(
    frame_row: u32,
    frame_counter: f32,
    ai0: f32,
    local_ai0: f32,
    wind: f32,
) -> (u32, f32) {
    let mut row = frame_row;
    let mut counter = frame_counter;
    if ai0 == 0.0 {
        // `:77122-77154` — the 6-frame sway.
        let rate = 8.0f32; // num24
        if row == 6 {
            counter += 1.0 + 0.5 * wind.abs();
            if counter > rate {
                counter -= rate;
                row = 0;
            }
            return (row, counter);
        }
        if row > 5 {
            row = 6;
            counter = 0.0;
            return (row, counter);
        }
        counter += 1.0 + 0.5 * wind.abs();
        if counter > rate {
            counter -= rate;
            row += 1;
            if row > 5 {
                row = 0;
            }
        }
    } else if local_ai0 == 0.0 {
        // `:77155-77181` — the 4-tick 6↔7..10 cycle.
        counter += 1.0;
        if counter > 4.0 {
            counter = 0.0;
            if row == 6 {
                row = 7;
            } else if row < 7 {
                row = 6;
            } else {
                row += 1;
                if row > 10 {
                    row = 7;
                }
            }
        }
    } else if local_ai0 == 1.0 {
        // `:77182-77200` — the charge strip.
        let mut r = (7.0 + (20.0 - 7.0) * (counter / 80.0)) as i32;
        if r > 19 {
            r = 19;
        }
        if r > 16 {
            r -= 9;
        }
        row = r as u32;
        counter += 1.0;
        if counter > 80.0 {
            counter = 0.0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 628-shaped NPC (the style73 placeholder convention).
    fn dandelion(x: f32, y: f32) -> Npc {
        let d = &DANDELION;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
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

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// `NPC.cs:17072-17084` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s119_the_defaults_row() {
        let d = &DANDELION;
        assert_eq!((d.width, d.height), (22, 40));
        assert_eq!(d.ai_style, 119);
        assert_eq!((d.damage, d.defense, d.life_max), (15, 0, 50));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 1));
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.value, 50.0);
        assert_eq!(DANDELION_FRAME_COUNT, 17);
    }

    /// `:46986-46989` — the non-happy-windy despawn clamp.
    #[test]
    fn w4_d33_s119_the_despawn_clamp() {
        let w = air();
        let mut rng = UnifiedRandom::new(119);
        let mut n = dandelion(100.0, 100.0);
        n.despawn_timer = 750;
        let mut env = Style119Env {
            world: &w,
            rng: &mut rng,
            players: &[],
            wind_speed_current: 0.5,
            happy_windy_day: false,
            net_mode: 0,
        };
        let _ = style119_step(&mut n, &mut env);
        assert_eq!(n.despawn_timer, 10);
        let mut n = dandelion(100.0, 100.0);
        n.despawn_timer = 750;
        env.happy_windy_day = true;
        let _ = style119_step(&mut n, &mut env);
        assert_eq!(n.despawn_timer, 750, "the happy day keeps it");
    }

    /// `:47059-47068` — the idle arms only when the downwind flag
    /// holds (the player downwind, near, level).
    #[test]
    fn w4_d33_s119_the_idle_arms_downwind() {
        let w = air();
        let mut rng = UnifiedRandom::new(119);
        // The player 200 px RIGHT, level — the +wind matches.
        let players = [cand(300.0, 100.0)];
        let mut n = dandelion(100.0, 100.0);
        let mut env = Style119Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_current: 0.5,
            happy_windy_day: true,
            net_mode: 0,
        };
        let _ = style119_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 1.0, "arms downwind");
        assert!(n.net_update);
        // The wind against the player → no arm.
        let mut n = dandelion(100.0, 100.0);
        env.wind_speed_current = -0.5;
        let _ = style119_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0, "no arm against the wind");
    }

    /// `:47037-47054` — the seed burst at localAI[1]==40: the
    /// 1+Next(3) count, the per-seed Next(-2,10)/Next(-6,6), the 836
    /// type, damage 7, knockback 1, the vy>-3 floor.
    #[test]
    fn w4_d33_s119_the_seed_burst() {
        let w = air();
        let mut rng = UnifiedRandom::new(119);
        let mut oracle = UnifiedRandom::new(119);
        let players = [cand(300.0, 100.0)]; // downwind, within 500
        let mut n = dandelion(100.0, 100.0);
        n.ai[0] = 1.0;
        n.local_ai[1] = 39.0; // ++ → 40, the fire tick
        let mut env = Style119Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_current: 0.5,
            happy_windy_day: true,
            net_mode: 0,
        };
        let out = style119_step(&mut n, &mut env);
        let count = 1 + oracle.next_max(3);
        for _ in 0..count {
            oracle.next_range(-2, 10);
            oracle.next_range(-6, 6);
        }
        assert_eq!(rng.peek(), oracle.peek(), "the burst draw order");
        assert_eq!(out.shots.len(), count as usize);
        for s in &out.shots {
            assert_eq!(s.projectile_type, 836);
            assert_eq!(s.damage, 7);
            assert_eq!(s.knock_back, 1.0);
            assert!(s.vy <= -3.0, "the vy floor");
            assert!(s.vx > 0.0, "the downwind +X aim");
        }
        assert!(n.net_update);
        // The client never fires.
        let mut n = dandelion(100.0, 100.0);
        n.ai[0] = 1.0;
        n.local_ai[1] = 39.0;
        env.net_mode = 1;
        let out = style119_step(&mut n, &mut env);
        assert!(out.shots.is_empty());
    }

    /// `:47021-47030` — the far-but-downwind hold (localAI[0]!=1)
    /// returns without firing; past 80 disarms.
    #[test]
    fn w4_d33_s119_the_hold_and_disarm() {
        let w = air();
        let mut rng = UnifiedRandom::new(119);
        // The player 550 px right — downwind (<600) but NOT <500.
        let players = [cand(650.0, 100.0)];
        let mut n = dandelion(100.0, 100.0);
        n.ai[0] = 1.0;
        n.local_ai[1] = 39.0;
        let mut env = Style119Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_current: 0.5,
            happy_windy_day: true,
            net_mode: 0,
        };
        let out = style119_step(&mut n, &mut env);
        assert!(out.shots.is_empty(), "the far hold");
        assert_eq!(n.local_ai[0], 0.0);
        assert_eq!(n.ai[0], 1.0, "still armed");
        // Past 80 → disarm.
        let mut n = dandelion(100.0, 100.0);
        n.ai[0] = 1.0;
        n.local_ai[1] = 80.0;
        let players = [cand(300.0, 100.0)];
        let mut env = Style119Env {
            world: &w,
            rng: &mut rng,
            players: &players,
            wind_speed_current: 0.5,
            happy_windy_day: true,
            net_mode: 0,
        };
        let _ = style119_step(&mut n, &mut env);
        assert_eq!(n.ai[0], 0.0, "the 80-tick disarm");
    }

    /// `NPC.cs:77103-77201` — the sway, the cycle, the charge strip.
    #[test]
    fn w4_d33_s119_the_find_frame_arms() {
        // The sway: row 0, wind 0 → counter +1; at >8 advances.
        assert_eq!(dandelion_find_frame(0, 0.0, 0.0, 0.0, 0.0), (0, 1.0));
        assert_eq!(dandelion_find_frame(0, 8.0, 0.0, 0.0, 0.0), (1, 1.0));
        // The row-6 hold resets to 0.
        assert_eq!(dandelion_find_frame(6, 8.0, 0.0, 0.0, 0.0), (0, 1.0));
        // The cycle: row 6 → 7 at the 4-tick.
        assert_eq!(dandelion_find_frame(6, 4.0, 1.0, 0.0, 0.0), (7, 0.0));
        // The charge strip: counter 0 → Lerp(7,20,0)=7.
        assert_eq!(dandelion_find_frame(0, 0.0, 1.0, 1.0, 0.0).0, 7);
        // counter 80 → Lerp=20 → cap 19 → >16 subtract 9 → 10.
        assert_eq!(dandelion_find_frame(0, 80.0, 1.0, 1.0, 0.0).0, 10);
    }

    /// The stream parity: 300 ticks match a cloned twin bit-for-bit.
    #[test]
    fn w4_d33_s119_the_run_is_twin_stable() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut a = dandelion(100.0, 100.0);
        let mut b = a.clone();
        let mut rng_a = UnifiedRandom::new(628);
        let mut rng_b = UnifiedRandom::new(628);
        for tick in 0..300 {
            let mut env_a = Style119Env {
                world: &w,
                rng: &mut rng_a,
                players: &players,
                wind_speed_current: 0.5,
                happy_windy_day: true,
                net_mode: 0,
            };
            let _ = style119_step(&mut a, &mut env_a);
            let mut env_b = Style119Env {
                world: &w,
                rng: &mut rng_b,
                players: &players,
                wind_speed_current: 0.5,
                happy_windy_day: true,
                net_mode: 0,
            };
            let _ = style119_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "state @{tick}");
            assert_eq!(
                a.local_ai[1].to_bits(),
                b.local_ai[1].to_bits(),
                "timer @{tick}"
            );
        }
    }
}
