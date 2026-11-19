//! `AI_026_Unicorns` (`NPC.cs:62366-62924`) — the aiStyle-26 machine for the
//! Pumpkin Moon's Headless Horseman (315) and Hellhound (329), S1-F-13b2
//! slice A. The C# machine also serves the Unicorn (155), the lunar
//! pillar mobs (410's kamikaze, 423's NebulaBeast charge) and the Sand
//! Elemental (546). 155/410/546 are not in the port's roster and 423
//! rides its own ported machine (`npc.rs::update_nebula_beast`,
//! M4-EVENT-9), so only the 315/329 arms are ported here — the shared
//! core is verbatim, the other types' branches are cited and skipped.
//!
//! RNG (`Main.rand` rides the `UpdateNPCs` stream —
//! `Main.cs:17767-17769`'s `SwapRandom("UpdateNPCs")` around
//! `NPC.UpdateNPC`): the machine's only draws are the Horseman's
//! gallop-dust arm (`NPC.cs:62431-62438`: `Next(3)`, then the two
//! `NextFloat()` position jitters) and its 480-tick pumpkin lob
//! (`:62417`: `NextVector2Circular(40, 40)` = two `NextFloat()` draws —
//! `Utils.cs:1261-1269` — then `NextFloatDirection()`). The port has no
//! per-tick dust pool, so the gallop-dust arm is skipped WITH its draws
//! (the `npc_lights.rs` S1-E-5 precedent — ANALYSIS, S1-F-13b2); the lob
//! keeps its exact three-draw order. The Hellhound draws nothing.
//!
//! Deferred to slice B (documented in ANALYSIS): the gallop dust visuals,
//! the `FindFrame` case-315/329 sprite arms, and the Horseman's
//! `Lighting.AddLight(Center, 0.4, 0.36, 0.2)` (`:62420` — the port's
//! light feed is the render-side aiStyle-8 tail, a different seam).

use crate::npc::{HostileShotAi, Npc, NpcKind, GRAVITY, MAX_FALL};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use tr_core::tiledata as td;
use tr_core::{Tile, UnifiedRandom, World, TILE_PX};

/// The `ProjectileID.HorsemanPumpkin` (1001) lob — the Horseman's
/// `Projectile.NewProjectile(..., Type: 1001, ..., ai0: num3)`
/// (`NPC.cs:62417`). The `ai0` payload carries the target player slot.
const HORSEMAN_PUMPKIN: u16 = 1001;

/// `Tile.nactive()` (`Tile.cs:240-248`): active and NOT actuated — the
/// AI_026 tile probes' own gate (`Main.tile[..].nactive()`), narrower
/// than `collision.rs`'s `support_solid` (which is `Collision.StepUp`'s
/// gate, a different call site).
fn nactive(t: &Tile) -> bool {
    t.active() && t.flags & tr_core::tile::FLAG_INACTIVE == 0
}

/// `Main.tileSolid[t.type]` — the generated table (the 192 leaf-block
/// carve is worldgen's, already applied in `td::TILE_SOLID`).
fn solid_kind(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind)
}

/// `Main.tileSolidTop[t.type]`.
fn solid_top(kind: u16) -> bool {
    td::TILE_SOLID_TOP.has(kind)
}

/// `Tile.topSlope()` (`Tile.cs:290-298`): slope 1 or 2.
fn top_slope(t: &Tile) -> bool {
    let s = t.slope();
    s == 1 || s == 2
}

/// A missing tile reads as the C#'s `new Tile()` (`NPC.cs:62756-62775`):
/// inactive, type 0 — every `nactive()` probe on it is false.
fn tile_or_blank(world: &World, x: i32, y: i32) -> Tile {
    world.tile(x, y).copied().unwrap_or_default()
}

/// The player slot `Main.player[this.target]` reads (`NPC.cs:62463`).
/// `SetDefaults`' `ClearTarget` sets 255 (`NPC.cs:42924-42927`); the port
/// has one session player at slot 0 and `target_closest` picks it on the
/// machine's first tick, so a still-255 target reads the session player
/// (documented in ANALYSIS — the C# would index the inactive slot-255
/// dummy at the world origin for exactly that first tick).
fn target_player<'a>(npc: &Npc, player: &'a Player) -> &'a Player {
    let _ = npc.target;
    player
}

/// The shared chase/turn state machine + movement core of
/// `AI_026_Unicorns`, one fixed step. Called from `Npc::update_kind_ai`'s
/// 315/329 arm with the caller's `UpdateNPCs` stream.
pub(crate) fn unicorn_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
) {
    let horseman = npc.kind == NpcKind::HeadlessHorseman;
    let (w, h) = npc.size();
    // `int num = 30; int num2 = 10;` (`NPC.cs:62368-62369`) — the 30-tick
    // stuck window and its 10x wander bound. `flag` (`:62370`) is only
    // set by the unported 423 charge arm; it stays false.
    let num = 30.0f32;
    let num2 = 10.0f32;
    let flag = false;
    // `:62373-62377` — walking against the facing while grounded arms the
    // stuck counter.
    let mut flag2 = false;
    if npc.vy == 0.0 && ((npc.vx > 0.0 && npc.facing < 0) || (npc.vx < 0.0 && npc.facing > 0)) {
        flag2 = true;
        npc.ai[3] += 1.0;
    }
    // `:62409-62440` — the Horseman's own block: the 480-tick pumpkin lob,
    // the light (deferred, module docs) and the gallop dust (deferred with
    // its draws, module docs).
    if horseman {
        // `if (this.localAI[0]++ >= 480f)` (`:62411`) — post-increment:
        // the fire tick is the 481st call.
        let fire = npc.local_ai[0] >= 480.0;
        npc.local_ai[0] += 1.0;
        if fire {
            npc.local_ai[0] = 0.0;
            // `int num3 = this.target; if (Main.netMode != 1 && num3 != 255)`
            // (`:62414-62415`) — the port is the host (netMode 0); a target
            // that never resolved (255) suppresses the shot, verbatim.
            if npc.target != 255 {
                let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
                // `base.Center + Main.rand.NextVector2Circular(40f, 40f)`
                // (`:62417`): `NextVector2Unit() * (40, 40) * NextFloat()`
                // (`Utils.cs:1266-1269`) — NextFloat, (cos, sin) of
                // `2π * NextFloat` (`Utils.cs:1261-1264` + `:1665-1668`,
                // double Math.Cos/Sin cast to f32), scaled by a third
                // NextFloat. Draw order: angle, radius, then the lob's
                // `NextFloatDirection() * 3f` vertical velocity.
                let angle = (std::f64::consts::PI * 2.0) * rng.next_float() as f64;
                let unit = (angle.cos() as f32, angle.sin() as f32);
                let radial = rng.next_float();
                let px = cx + unit.0 * 40.0 * radial;
                let py = cy + unit.1 * 40.0 * radial;
                let vy = rng.next_float_direction() * 3.0;
                // `Damage: this.GetAttackDamage_ForProjectiles(40f, 30f)`
                // (`NPC.cs:7052-7055`): the classic 40 (the port runs
                // classic difficulty — the same fold as the NebulaBeast's
                // 50, `npc.rs`'s `update_nebula_beast`). KnockBack 0,
                // `ai0 = num3` (the target slot).
                npc.wants_hostile_shot_ai.push(HostileShotAi {
                    kind: HORSEMAN_PUMPKIN,
                    x: px,
                    y: py,
                    vx: npc.vx,
                    vy,
                    damage: 40,
                    ai0: npc.target as f32,
                    ai1: 0.0,
                });
            }
        }
        // `Lighting.AddLight(base.Center, 0.4f, 0.36f, 0.2f)` (`:62420`)
        // and the frame-4..7 gallop dust (`:62421-62439`) are deferred —
        // module docs + ANALYSIS (S1-F-13b2 slice A).
    }
    // `:62441-62449` — the stuck counter: a frozen x (or the 30-window,
    // or the turn flag) increments and arms flag3; otherwise it decays.
    // The C#'s `oldPosition.X` is latched AFTER the AI
    // (`Collision_MoveWhileDry`, `:92605`) — "did not move LAST tick". The
    // port's `update_rng` latches `old_x` before the AI, so the machine
    // carries its own end-of-tick latch (`unicorn_stuck_x`, the
    // `frost_stuck_x` precedent, npc.rs:6623-6627).
    let mut flag3 = false;
    if (npc.x == npc.unicorn_stuck_x || npc.ai[3] >= num) | flag2 {
        npc.ai[3] += 1.0;
        flag3 = true;
    } else if npc.ai[3] > 0.0 {
        npc.ai[3] -= 1.0;
    }
    // `:62450-62461` — the 300 cap, the justHit reset (the port's hurt
    // cooldown stands in for `justHit`, the house convention), the
    // netUpdate at exactly 30.
    if npc.ai[3] > num * num2 {
        npc.ai[3] = 0.0;
    }
    if npc.hurt_cd > 0.0 {
        npc.ai[3] = 0.0;
    }
    if npc.ai[3] == num {
        npc.net_update = true;
    }
    // `:62462-62469` — the target vector: num6 to the player's centre-x,
    // num7 to the player's TOP (`position.Y`, not Centre.Y — verbatim),
    // num8 the Euclidean length; inside 200 px a moving NPC clears the
    // stuck counter.
    let p = target_player(npc, player);
    let (pcx, _) = (
        p.x + crate::player::WIDTH * 0.5,
        p.y + crate::player::HEIGHT * 0.5,
    );
    let cx = npc.x + w * 0.5;
    let cy = npc.y + h * 0.5;
    let num6 = pcx - cx;
    let num7 = p.y - cy;
    let num8 = (num6 * num6 + num7 * num7).sqrt();
    if num8 < 200.0 && !flag3 {
        npc.ai[3] = 0.0;
    }
    // `:62558-62564` — the 155/329 hop-over: grounded, target inside
    // 100 px, |vx| > 3 and rolling toward the player's centre → a -4 hop.
    // The centres use INTEGER half-widths (`width / 2`, `:62560`).
    if !horseman
        && npc.vy == 0.0
        && num8 < 100.0
        && npc.vx.abs() > 3.0
    {
        // The integer-half centres (`width / 2`, `:62560`); the two
        // one-sided tests live in a block so the `<`/`>` comparisons
        // never parse as const generics.
        let ncx = npc.x + (w as i32 / 2) as f32;
        let pcx_int = p.x + (crate::player::WIDTH as i32 / 2) as f32;
        if (ncx < pcx_int && npc.vx > 0.0) || (ncx > pcx_int && npc.vx < 0.0) {
            npc.vy -= 4.0;
        }
    }
    // `:62581-62616` — the chase vs wander split. Outside the pumpkin
    // moon both kinds encourage despawn instead of retargeting
    // (`:62583-62586`); inside it, `TargetClosest()` runs while the
    // stuck window is open. Past the window the NPC wanders: a grounded
    // standstill flips the facing every 2 ticks (`ai[0]`), `directionY`
    // arms the jump probe below.
    if npc.ai[3] < num {
        if !world.pumpkin_moon {
            npc.encourage_despawn(10);
        } else {
            let cand = TargetCandidate {
                active: true,
                dead: player.dead,
                ghost: false,
                x: player.x,
                y: player.y,
                width: crate::player::WIDTH as i32,
                height: crate::player::HEIGHT as i32,
                aggro: 0,
                item_animation: player.frame_inputs.swing_anim as i32,
                npc_type_no_aggro: false,
                tank_pet: None,
            };
            target_closest(npc, world, &[cand], true, 26, 0);
        }
    } else {
        // `:62594-62615` — the wander tail.
        if npc.vx == 0.0 {
            if npc.vy == 0.0 {
                npc.ai[0] += 1.0;
                if npc.ai[0] >= 2.0 {
                    npc.facing *= -1;
                    npc.sprite_dir = npc.facing;
                    npc.ai[0] = 0.0;
                }
            }
        } else {
            npc.ai[0] = 0.0;
        }
        npc.direction_y = -1;
        if npc.facing == 0 {
            npc.facing = 1;
        }
    }
    // `:62617-62740` — the walk core. num10 = 6 cap, num11 = 0.07 accel
    // (the 410/546 rows are unported kinds; 423 rides its own machine).
    // The gate: grounded, wet, or already rolling with the facing.
    let num10 = 6.0f32;
    let num11 = 0.07f32;
    if !flag
        && (npc.vy == 0.0
            || npc.wet
            || (npc.vx <= 0.0 && npc.facing < 0)
            || (npc.vx >= 0.0 && npc.facing > 0))
    {
        if horseman {
            // `:62651-62684` — the 315 arm: the 0.95 turnaround damping,
            // then its OWN copy of the cap/approach block (the C# runs
            // the shared block again below — both run, verbatim).
            if npc.vx > 0.0 && npc.facing < 0 {
                npc.vx *= 0.95;
            }
            if npc.vx < 0.0 && npc.facing > 0 {
                npc.vx *= 0.95;
            }
            if npc.vx < 0.0 - num10 || npc.vx > num10 {
                if npc.vy == 0.0 {
                    npc.vx *= 0.8;
                    npc.vy *= 0.8;
                }
            } else if npc.vx < num10 && npc.facing == 1 {
                npc.vx += 0.07;
                if npc.vx > num10 {
                    npc.vx = num10;
                }
            } else if npc.vx > 0.0 - num10 && npc.facing == -1 {
                npc.vx -= 0.07;
                if npc.vx < 0.0 - num10 {
                    npc.vx = 0.0 - num10;
                }
            }
        } else {
            // `:62632-62650` — the 329 arm: the 0.9 turnaround damping
            // and the ±3 soft band's ±0.1 nudge (the shared block below
            // still accelerates to the 6 cap).
            if npc.vx > 0.0 && npc.facing < 0 {
                npc.vx *= 0.9;
            }
            if npc.vx < 0.0 && npc.facing > 0 {
                npc.vx *= 0.9;
            }
            if npc.facing > 0 && npc.vx < 3.0 {
                npc.vx += 0.1;
            }
            if npc.facing < 0 && npc.vx > -3.0 {
                npc.vx -= 0.1;
            }
        }
        // `:62717-62739` — the shared cap/approach block.
        if npc.vx < 0.0 - num10 || npc.vx > num10 {
            if npc.vy == 0.0 {
                npc.vx *= 0.8;
                npc.vy *= 0.8;
            }
        } else if npc.vx < num10 && npc.facing == 1 {
            npc.vx += num11;
            if npc.vx > num10 {
                npc.vx = num10;
            }
        } else if npc.vx > 0.0 - num10 && npc.facing == -1 {
            npc.vx -= num11;
            if npc.vx < 0.0 - num10 {
                npc.vx = 0.0 - num10;
            }
        }
    }
    // `:62741-62805` — the inline step-up (the AI's own copy of
    // Collision.StepUp's shape, NOT the shared helper: the 315/329 probe
    // rows differ). Only while falling or level.
    if npc.vy >= 0.0 {
        let num13 = if npc.vx < 0.0 {
            -1i32
        } else if npc.vx > 0.0 {
            1
        } else {
            0
        };
        let vx_pos = npc.x + npc.vx;
        let num14 = ((vx_pos + (w as i32 / 2) as f32 + ((w as i32 / 2 + 1) * num13) as f32)
            / 16.0) as i32;
        let num15 = ((npc.y + h - 1.0) / 16.0) as i32;
        let t0 = tile_or_blank(world, num14, num15);
        let t1 = tile_or_blank(world, num14, num15 - 1);
        let t2 = tile_or_blank(world, num14, num15 - 2);
        let t3 = tile_or_blank(world, num14, num15 - 3);
        let t4 = tile_or_blank(world, num14 - num13, num15 - 3);
        let tm4 = tile_or_blank(world, num14, num15 - 4);
        // `:62776` — the ledge gate, verbatim conjunct order.
        let base_ok = (nactive(&t0)
            && !top_slope(&t0)
            && !top_slope(&t1)
            && solid_kind(t0.kind)
            && !solid_top(t0.kind))
            || (t1.half_brick() && nactive(&t1));
        let head_clear = !nactive(&t1)
            || !solid_kind(t1.kind)
            || solid_top(t1.kind)
            || (t1.half_brick() && (!nactive(&tm4) || !solid_kind(tm4.kind) || solid_top(tm4.kind)));
        let body_clear = (!nactive(&t2) || !solid_kind(t2.kind) || solid_top(t2.kind))
            && (!nactive(&t3) || !solid_kind(t3.kind) || solid_top(t3.kind))
            && (!nactive(&t4) || !solid_kind(t4.kind));
        if (num14 as f32 * 16.0) < vx_pos + w
            && (num14 as f32 * 16.0 + 16.0) > vx_pos
            && base_ok
            && head_clear
            && body_clear
        {
            // `:62778-62803` — the lift: half-brick offsets, the 16.1
            // reach, the gfxOffY record and the 1/2 step speed.
            let mut num16 = num15 as f32 * 16.0;
            if t0.half_brick() {
                num16 += 8.0;
            }
            if t1.half_brick() {
                num16 -= 8.0;
            }
            if num16 < npc.y + h {
                let num17 = npc.y + h - num16;
                if (num17 as f64) <= 16.1 {
                    npc.gfx_off_y += npc.y + h - num16;
                    npc.y = num16 - h;
                    npc.step_speed = if num17 < 9.0 { 1.0 } else { 2.0 };
                }
            }
        }
    }
    // `:62806-62903` — the jump probe. Grounded, head room over the body
    // (`position.Y - 7f` row), then the ahead column at
    // `centre + (width/2 + 2) * direction + vx * 5`.
    if npc.vy == 0.0 {
        let mut flag6 = true;
        let num18 = (npc.y - 7.0) as i32 / 16;
        let num19 = (npc.x - 7.0) as i32 / 16;
        let num20 = (npc.x + w + 7.0) as i32 / 16;
        for m in num19..=num20 {
            let t = tile_or_blank(world, m, num18);
            if nactive(&t) && solid_kind(t.kind) {
                flag6 = false;
                break;
            }
        }
        if flag6 {
            let num21 = ((npc.x + (w as i32 / 2) as f32
                + ((w as i32 / 2 + 2) * npc.facing) as f32
                + npc.vx * 5.0)
                / 16.0) as i32;
            let num22 = ((npc.y + h - 15.0) / 16.0) as i32;
            let j0 = tile_or_blank(world, num21, num22);
            let j1 = tile_or_blank(world, num21, num22 - 1);
            let j2 = tile_or_blank(world, num21, num22 - 2);
            let j3 = tile_or_blank(world, num21, num22 - 3);
            // The `num22 + 1` probe (`:62840-62843`) is only read behind
            // `flag7` (`:62896`), false for 315/329 — no read to port.
            let below2 = tile_or_blank(world, num21, num22 + 2);
            let ahead3 = tile_or_blank(world, num21 + npc.facing, num22 + 3);
            // `:62864-62868` — num23 = spriteDirection (the 423/410/546
            // inversion is unported kinds).
            let num23 = npc.sprite_dir;
            if (npc.vx < 0.0 && num23 == -1) || (npc.vx > 0.0 && num23 == 1) {
                // `flag7` (the 410/423 platform exemption, `:62871`) is
                // false for 315/329 — the below-1 conjunct `(!flag7 || …)`
                // (`:62896`) folds to true; num24 = 3 (`:62872`).
                let num24 = 3.0f32;
                if nactive(&j2) && solid_kind(j2.kind) {
                    if nactive(&j3) && solid_kind(j3.kind) {
                        npc.vy = -8.5;
                        npc.net_update = true;
                    } else {
                        npc.vy = -7.5;
                        npc.net_update = true;
                    }
                } else if nactive(&j1) && !top_slope(&j1) && solid_kind(j1.kind) {
                    npc.vy = -7.0;
                    npc.net_update = true;
                } else if npc.y + h - (num22 as f32 * 16.0) > 20.0
                    && nactive(&j0)
                    && !top_slope(&j0)
                    && solid_kind(j0.kind)
                {
                    npc.vy = -6.0;
                    npc.net_update = true;
                } else if (npc.direction_y < 0 || npc.vx.abs() > num24)
                    && (!nactive(&below2) || !solid_kind(below2.kind))
                    && (!nactive(&ahead3) || !solid_kind(ahead3.kind))
                {
                    npc.vy = -8.0;
                    npc.net_update = true;
                }
            }
        }
    }
    // `:62904-62923` — the 423 dust and the 546 rotation tails are
    // unported kinds; nothing runs for 315/329.
    //
    // The shared movement after the AI (`NPC.UpdateNPC`, `:90465-90493`):
    // gravity + the fall cap, then the axis-separated tile collision
    // (`UpdateCollision`). The port's family convention (walker_step /
    // update_frost_legion) integrates here with the port's GRAVITY/
    // MAX_FALL pair — the C#'s depth-scaled 0.3/10 gravity ride the
    // port-wide gravity seam (ANALYSIS).
    npc.vy = (npc.vy + GRAVITY).min(MAX_FALL);
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
    }
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
    } else if npc.vy < 0.0 {
        let ty = (ny / TILE_PX).floor() as i32;
        npc.y = (ty + 1) as f32 * TILE_PX;
        npc.vy = 0.0;
    }
    // `base.oldPosition = base.position` (`NPC.cs:92605`, the movement
    // tail's latch — the stuck block above reads it next tick).
    npc.unicorn_stuck_x = npc.x;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// A flat stone floor at row 100 (the `npc_ai.rs` tests' ground
    /// shape), 200x120 tiles.
    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 120);
        for x in 0..200 {
            for y in 100..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.pumpkin_moon = true;
        w
    }

    fn player_at(w: &World, tx: f32, ty: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = tx * TILE_PX;
        p.y = ty * TILE_PX;
        p
    }

    /// A grounded NPC of `kind` at tile (tx, floor-top − height).
    fn grounded(kind: NpcKind, w: &World, tx: f32, facing: i32) -> Npc {
        let (_, h) = kind.size();
        let mut n = Npc::new(kind, tx * TILE_PX, 100.0 * TILE_PX - h, facing, 0.0);
        n.on_ground = true;
        n.sprite_dir = facing;
        n.target = 0;
        let _ = w;
        n
    }

    /// The stuck-counter core (`NPC.cs:62373-62469`): the frozen-x
    /// increment, the >300 wrap, the justHit reset, the netUpdate at
    /// exactly 30, and the <200-px clear.
    #[test]
    fn s1_f_13b2_the_stuck_counter_walks_the_csharp_windows() {
        let w = ground();
        // The player far away (>200 px) so the proximity clear never fires,
        // and BELOW the hound's centre so `directionY` stays 1 — an above
        // player arms the jump probe's `directionY < 0` arm every tick
        // (`:62896`), polluting the netUpdate observation.
        let p = player_at(&w, 150.0, 98.0);
        let mut n = grounded(NpcKind::Hellhound, &w, 60.0, 1);
        // The targeting change-flag inputs pre-latched so the in-window
        // `TargetClosest` calls do not arm netUpdate (`NPC.cs:77776-77779`
        // compares against the previous tick's values).
        n.old_target = 0;
        n.old_direction = 1;
        n.old_direction_y = 1;
        let mut rng = UnifiedRandom::new(0);
        // Frozen x: the machine's own end-of-tick latch
        // (`unicorn_stuck_x`, the C#'s post-AI `oldPosition`,
        // `NPC.cs:92605`) equals the anchor each tick — the wall-stall
        // shape. Pin vx back to 0 and x to the anchor after every step.
        let anchor = n.x;
        let mut saw_net = false;
        for _ in 0..31 {
            n.vx = 0.0;
            n.x = anchor;
            n.unicorn_stuck_x = anchor;
            n.net_update = false;
            unicorn_step(&mut n, &w, &p, &mut rng);
            if n.net_update {
                saw_net = true;
                assert_eq!(n.ai[3], 30.0, "netUpdate fires at exactly 30");
            }
            n.x = anchor; // the wall holds
        }
        assert!(saw_net, "the 30-tick window armed netUpdate");
        assert!(n.ai[3] >= 30.0, "the counter passed the window");
        // The >300 wrap (`:62450-62453`): from 300 the frozen-x tick
        // increments to 301 → wrapped to 0.
        n.ai[3] = 300.0;
        n.vx = 0.0;
        n.x = anchor;
        n.unicorn_stuck_x = anchor;
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[3], 0.0, "301 wraps to 0");
        // The justHit reset (`:62454-62457`, the hurt_cd stand-in).
        n.ai[3] = 120.0;
        n.hurt_cd = 0.18;
        n.unicorn_stuck_x = n.x - 5.0; // moving: no frozen-x increment
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[3], 0.0, "justHit zeroes the counter");
        // The <200-px clear (`:62466-62469`): a moving NPC near the
        // player clears the counter.
        let near = player_at(&w, 62.0, 99.0);
        n.ai[3] = 10.0;
        n.hurt_cd = 0.0;
        n.unicorn_stuck_x = n.x - 5.0;
        n.vx = 1.0;
        unicorn_step(&mut n, &w, &near, &mut rng);
        assert_eq!(n.ai[3], 0.0, "the 200-px proximity clears the counter");
    }

    /// The turn flag (`:62373-62377`): grounded and rolling against the
    /// facing increments ai[3] before the stuck block runs.
    #[test]
    fn s1_f_13b2_the_turn_flag_arms_the_counter() {
        let w = ground();
        let p = player_at(&w, 150.0, 95.0);
        let mut n = grounded(NpcKind::Hellhound, &w, 60.0, -1);
        n.vx = 2.0; // rolling right while facing left
        n.unicorn_stuck_x = n.x - 2.0; // moved: the frozen-x term is off
        let mut rng = UnifiedRandom::new(0);
        let before = n.ai[3];
        unicorn_step(&mut n, &w, &p, &mut rng);
        // flag2's +1 and flag3's +1 both land (`:62376` + `:62443`).
        assert!(n.ai[3] >= before + 2.0, "the turn arms both increments");
    }

    /// The walk core's per-type rows (`:62632-62684` + `:62717-62739`):
    /// the Horseman accelerates 0.07/tick to the 6 cap with the 0.95
    /// turnaround damping; the Hellhound nudges 0.1 inside ±3 then rides
    /// the shared 0.07 to 6 with the 0.9 damping.
    #[test]
    fn s1_f_13b2_the_walk_rows_match_the_csharp_accel_and_caps() {
        let w = ground();
        let p = player_at(&w, 150.0, 99.0);
        let mut rng = UnifiedRandom::new(0);
        // Horseman: first grounded tick from rest = 0.07 (the 315 arm's
        // own block; the shared block's `vx < num10 && direction == 1`
        // is an else-if chain — the arm ran, the shared block runs again:
        // 0.07 + 0.07 = 0.14).
        let mut hh = grounded(NpcKind::HeadlessHorseman, &w, 60.0, 1);
        hh.facing = 1;
        unicorn_step(&mut hh, &w, &p, &mut rng);
        assert!(
            (hh.vx - 0.14).abs() < 1e-5,
            "the 315 double block: 0.07 + 0.07, got {}",
            hh.vx
        );
        // Hellhound: the ±3 band's 0.1 nudge + the shared 0.07 = 0.17.
        let mut hd = grounded(NpcKind::Hellhound, &w, 60.0, 1);
        hd.facing = 1;
        unicorn_step(&mut hd, &w, &p, &mut rng);
        assert!(
            (hd.vx - 0.17).abs() < 1e-5,
            "the 329 nudge + shared accel: 0.1 + 0.07, got {}",
            hd.vx
        );
        // The cap: both settle at exactly 6 after enough ticks.
        for _ in 0..200 {
            hh.facing = 1;
            hd.facing = 1;
            hh.on_ground = true;
            hd.on_ground = true;
            hh.vy = 0.0;
            hd.vy = 0.0;
            hh.y = 100.0 * TILE_PX - hh.size().1;
            hd.y = 100.0 * TILE_PX - hd.size().1;
            unicorn_step(&mut hh, &w, &p, &mut rng);
            unicorn_step(&mut hd, &w, &p, &mut rng);
        }
        assert_eq!(hh.vx, 6.0, "the Horseman caps at num10");
        assert_eq!(hd.vx, 6.0, "the Hellhound caps at num10");
        // The turnaround damping: rolling +6 against facing -1 while
        // grounded, INSIDE the wander window (`ai[3] >= 30` — the C#'s
        // TargetClosest would flip the facing back to the player's side
        // while the window is open). The 315 arm damps 0.95 then runs its
        // OWN -0.07 approach arm (`:62676-62683`), and the shared block
        // (`:62732-62739`) runs its -0.07 AGAIN — both run, verbatim.
        hh.facing = -1;
        hh.sprite_dir = -1;
        hh.vx = 6.0;
        hh.vy = 0.0;
        hh.on_ground = true;
        hh.ai[3] = 31.0;
        hh.y = 100.0 * TILE_PX - hh.size().1;
        unicorn_step(&mut hh, &w, &p, &mut rng);
        assert!(
            (hh.vx - (6.0 * 0.95 - 0.07 - 0.07)).abs() < 1e-4,
            "the 315 turnaround: 6*0.95 then the double -0.07, got {}",
            hh.vx
        );
        // The 329 arm: the 0.9 damping, then the -0.1 band nudge
        // (`:62646-62649`, 5.4 > -3) AND the shared -0.07.
        hd.facing = -1;
        hd.sprite_dir = -1;
        hd.vx = 6.0;
        hd.vy = 0.0;
        hd.on_ground = true;
        hd.ai[3] = 31.0;
        hd.y = 100.0 * TILE_PX - hd.size().1;
        unicorn_step(&mut hd, &w, &p, &mut rng);
        assert!(
            (hd.vx - (6.0 * 0.9 - 0.1 - 0.07)).abs() < 1e-4,
            "the 329 turnaround: 6*0.9 then -0.1 -0.07, got {}",
            hd.vx
        );
    }

    /// The wander tail (`:62592-62616`): past the 30-window a grounded
    /// standstill flips the facing every 2 ticks and arms directionY -1.
    #[test]
    fn s1_f_13b2_the_wander_tail_flips_every_two_ticks() {
        let w = ground();
        let p = player_at(&w, 150.0, 95.0);
        let mut n = grounded(NpcKind::Hellhound, &w, 60.0, 1);
        n.ai[3] = 31.0; // past the window
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(0);
        // Tick 1: ai[0] 0→1, no flip. The frozen-x term keeps flag3 and
        // ai[3] climbs; pin it past the window each tick.
        n.unicorn_stuck_x = n.x;
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.facing, 1, "no flip on the first standstill tick");
        assert_eq!(n.direction_y, -1, "the wander arms the jump probe");
        n.ai[3] = 31.0;
        n.vx = 0.0;
        n.unicorn_stuck_x = n.x;
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.facing, -1, "the second tick flips");
        assert_eq!(n.sprite_dir, -1, "the sprite rides the flip");
        assert_eq!(n.ai[0], 0.0, "the flip resets the beat");
    }

    /// The despawn encouragement (`:62583-62586`): outside the pumpkin
    /// moon both kinds take EncourageDespawn(10) instead of retargeting.
    #[test]
    fn s1_f_13b2_outside_the_moon_the_kinds_despawn_encourage() {
        let mut w = ground();
        w.pumpkin_moon = false;
        let p = player_at(&w, 150.0, 95.0);
        let mut rng = UnifiedRandom::new(0);
        for kind in [NpcKind::HeadlessHorseman, NpcKind::Hellhound] {
            let mut n = grounded(kind, &w, 60.0, 1);
            n.despawn_timer = -1;
            unicorn_step(&mut n, &w, &p, &mut rng);
            assert_eq!(n.despawn_timer, 10, "{kind:?} encourages despawn(10)");
        }
        // Inside the moon: no encouragement, the target resolves.
        w.pumpkin_moon = true;
        let mut n = grounded(NpcKind::Hellhound, &w, 60.0, -1);
        n.despawn_timer = -1;
        n.target = 255;
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.despawn_timer, -1, "the moon keeps the chase alive");
        assert_eq!(n.target, 0, "TargetClosest picked the session player");
        assert_eq!(n.facing, 1, "the facing follows the player's side");
    }

    /// The Hellhound hop-over (`:62558-62564`): grounded, target inside
    /// 100 px, |vx| > 3, rolling toward the player's centre → vy -= 4.
    #[test]
    fn s1_f_13b2_the_hellhound_hops_the_close_target() {
        let w = ground();
        // The player 4 tiles (64 px) right of the hound's centre.
        let mut n = grounded(NpcKind::Hellhound, &w, 60.0, 1);
        let p = player_at(&w, 64.0, 99.0);
        n.vx = 4.0;
        let mut rng = UnifiedRandom::new(0);
        unicorn_step(&mut n, &w, &p, &mut rng);
        // The hop's -4 then the gravity tail's +0.4 → -3.6 airborne.
        assert!(n.vy < 0.0, "the hound hopped (vy {})", n.vy);
        // The Horseman has NO hop arm (the 315 block is dust/light only).
        let mut hh = grounded(NpcKind::HeadlessHorseman, &w, 60.0, 1);
        hh.vx = 4.0;
        unicorn_step(&mut hh, &w, &p, &mut rng);
        assert!(hh.vy >= 0.0, "the Horseman never hop-arms");
    }

    /// The jump probe (`:62806-62903`): the ahead column's rows are
    /// `num22 = (y + h - 15)/16` — for a floor-grounded hound that is the
    /// floor-top row minus 1 (99), so a 2-tile wall fills `num22 - 1`
    /// (the -7 arm), a 3-tile wall `num22 - 2` (-7.5), a 4-tile wall both
    /// (-8.5). A 1-tile wall is `j0` territory — its `> 20f` gate reads
    /// exactly 16 on the flat floor and fails (the step-up handles it).
    #[test]
    fn s1_f_13b2_the_jump_probe_reads_the_wall_height() {
        let mut rng = UnifiedRandom::new(0);
        for (wall_h, want) in [(2usize, -7.0f32), (3, -7.5), (4, -8.5)] {
            let mut w = ground();
            // A wall at tile x=70, `wall_h` tiles tall on the floor.
            for y in (100 - wall_h)..100 {
                let i = w.index(70, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
            let p = player_at(&w, 150.0, 95.0);
            let mut n = grounded(NpcKind::Hellhound, &w, 66.0, 1);
            n.vx = 6.0;
            n.sprite_dir = 1;
            n.direction_y = -1;
            unicorn_step(&mut n, &w, &p, &mut rng);
            // The probe fires before the gravity tail: vy = want + 0.4.
            assert!(
                (n.vy - (want + GRAVITY)).abs() < 1e-4,
                "a {wall_h}-tall wall jumps {want}, got vy {}",
                n.vy
            );
            assert!(n.net_update, "the jump arms netUpdate");
        }
    }

    /// The step-up (`:62741-62805`): a 1-tile ledge lifts the body,
    /// records gfxOffY and sets the 1/2 step speed by the 9-px split.
    #[test]
    fn s1_f_13b2_the_step_up_lifts_and_records() {
        let mut w = ground();
        // A single 1-tile step at x=70 (top row 99).
        let i = w.index(70, 99);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let p = player_at(&w, 150.0, 95.0);
        // The probe column is `(x + vx + width/2 + (width/2 + 1))/16`
        // (`:62754`): x = 67.25 tiles (1076) + vx 6 + 23 + 24 = 1129 →
        // tile 70. Inside the wander window so the facing holds.
        let mut n = grounded(NpcKind::Hellhound, &w, 67.25, 1);
        n.vx = 6.0;
        n.sprite_dir = 1;
        n.ai[3] = 31.0;
        let mut rng = UnifiedRandom::new(0);
        let y0 = n.y;
        unicorn_step(&mut n, &w, &p, &mut rng);
        // The lift is 16 px (> 9) → stepSpeed 2, gfxOffY 16; the body
        // lands exactly on the step top (row 99 * 16 − height).
        assert_eq!(n.y, y0 - 16.0, "the body lifted onto the step");
        assert_eq!(n.gfx_off_y, 16.0, "the lift recorded in gfxOffY");
        assert_eq!(n.step_speed, 2.0, "a 16-px lift sets stepSpeed 2");
        // The jump probe does not fire on the lifted body (its ahead
        // column reads clear air above the step).
        assert!(n.vy >= 0.0, "no jump on the step-up tick (vy {})", n.vy);
    }

    /// The Horseman's 480-tick lob (`:62411-62418`): the post-increment
    /// fires on the 481st tick, the shot carries the classic 40 damage,
    /// vx = the body's vx, ai0 = the target slot, and the three RNG draws
    /// ride the caller's stream in the C#'s order (angle, radius,
    /// direction).
    #[test]
    fn s1_f_13b2_the_horseman_lob_fires_at_481_with_the_csharp_draws() {
        let w = ground();
        let p = player_at(&w, 150.0, 95.0);
        let mut n = grounded(NpcKind::HeadlessHorseman, &w, 60.0, 1);
        n.target = 0;
        let mut rng = UnifiedRandom::new(42);
        for t in 0..480 {
            // The body moves every tick, so the machine's own stuck latch
            // (`unicorn_stuck_x`) reads "moved" and the counter decays.
            unicorn_step(&mut n, &w, &p, &mut rng);
            assert!(
                n.wants_hostile_shot_ai.is_empty(),
                "no lob before tick 481 (t={t})"
            );
        }
        assert_eq!(n.local_ai[0], 480.0, "the counter reached 480");
        // The fire tick: pin the body so the shot's payload is exact.
        n.vx = 2.5;
        n.vy = 0.0;
        let (cx, cy) = (n.x + n.size().0 * 0.5, n.y + n.size().1 * 0.5);
        // The witness stream: the same seed, the same three draws in the
        // C#'s order — NextVector2Unit's angle float, the radial float,
        // then NextFloatDirection (`Utils.cs:1261-1269`, `NPC.cs:62417`).
        let mut witness = UnifiedRandom::new(42);
        // The 480 prior ticks drew nothing (the dust arm is skipped with
        // its draws — module docs), so the witness is still at its seed.
        let angle = (std::f64::consts::PI * 2.0) * witness.next_float() as f64;
        let radial = witness.next_float();
        let dir = witness.next_float_direction();
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.local_ai[0], 0.0, "the fire resets the counter");
        assert_eq!(n.wants_hostile_shot_ai.len(), 1, "one lob fired");
        let shot = n.wants_hostile_shot_ai[0];
        assert_eq!(shot.kind, 1001, "the HorsemanPumpkin");
        assert_eq!(shot.damage, 40, "the classic GetAttackDamage(40, 30)");
        assert_eq!(shot.vx, 2.5, "the lob rides the body's vx");
        assert_eq!(shot.ai0, 0.0, "ai0 carries the target slot");
        let want_x = cx + (angle.cos() as f32) * 40.0 * radial;
        let want_y = cy + (angle.sin() as f32) * 40.0 * radial;
        assert!((shot.x - want_x).abs() < 1e-3, "the circular x draw");
        assert!((shot.y - want_y).abs() < 1e-3, "the circular y draw");
        assert!((shot.vy - dir * 3.0).abs() < 1e-4, "the direction draw");
        // The stream advanced by exactly the three draws.
        assert_eq!(
            rng.next_float().to_bits(),
            witness.next_float().to_bits(),
            "the lob consumed exactly three draws from the caller's stream"
        );
        // A 255 target suppresses the shot but still resets the counter
        // (`:62414-62415`'s `num3 != 255` gate).
        n.target = 255;
        n.local_ai[0] = 480.0;
        n.wants_hostile_shot_ai.clear();
        unicorn_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.local_ai[0], 0.0, "the counter resets regardless");
        assert!(n.wants_hostile_shot_ai.is_empty(), "no shot at target 255");
    }

    /// The Hellhound never draws RNG and never lobs; the Horseman's
    /// non-fire ticks draw nothing either (the dust arm is skipped with
    /// its draws) — the UpdateNPCs stream stays byte-identical.
    #[test]
    fn s1_f_13b2_the_machine_keeps_the_stream_untouched_between_lobs() {
        let w = ground();
        let p = player_at(&w, 150.0, 95.0);
        let mut rng = UnifiedRandom::new(9);
        let mut witness = UnifiedRandom::new(9);
        for kind in [NpcKind::HeadlessHorseman, NpcKind::Hellhound] {
            let mut n = grounded(kind, &w, 60.0, 1);
            for _ in 0..200 {
                unicorn_step(&mut n, &w, &p, &mut rng);
            }
            assert!(n.wants_hostile_shot_ai.is_empty(), "{kind:?} lobbed early");
        }
        for _ in 0..16 {
            assert_eq!(
                rng.next(),
                witness.next(),
                "the machine drew from the stream outside the lob"
            );
        }
    }

    /// The full chase: spawned left of the player on the flat floor,
    /// both kinds close the gap and never exceed the 6 cap (the S1-F-13b
    /// stand-in test's contract, now on the named machine).
    #[test]
    fn s1_f_13b2_the_kinds_chase_within_the_cap() {
        let w = ground();
        let p = player_at(&w, 120.0, 99.0);
        let mut rng = UnifiedRandom::new(7);
        for kind in [NpcKind::HeadlessHorseman, NpcKind::Hellhound] {
            let (_, h) = kind.size();
            let mut n = Npc::new(kind, 60.0 * TILE_PX, 100.0 * TILE_PX - h, 1, 0.0);
            n.on_ground = true;
            let dx0 = p.x - n.x;
            for _ in 0..120 {
                unicorn_step(&mut n, &w, &p, &mut rng);
                assert!(
                    n.vx.abs() <= 6.0 + 0.001,
                    "{kind:?} walks past the num10 cap ({})",
                    n.vx
                );
            }
            assert!(p.x - n.x < dx0, "{kind:?} never closed the gap");
        }
    }
}
