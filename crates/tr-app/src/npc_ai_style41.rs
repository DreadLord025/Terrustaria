//! The aiStyle-41 family — `NPC.AI()`'s `else if (this.aiStyle == 41)`
//! block (`NPC.cs:29899-30159`), S1-D-3 (aiStyle-41 round). The jungle
//! hopper family: the Herpling 174 (`SetDefaults` `NPC.cs:11194-11207`:
//! 52x26, damage 65, defense 26, lifeMax 220, kb 0.3, value 450), the
//! Derpling 177 (`:11238-11251`: 58x44, damage 80, defense 26, lifeMax
//! 300, kb 0.5, value 500, NPCHit22/NPCDeath25) and the Chattering Teeth
//! Bomb 378 (`:13825-13838`: 20x26, damage 120, defense 30, lifeMax 200,
//! kb 0.5, value 450). `ID/NPCID.cs:11025`/`:11031`/`:11433`.
//!
//! **Unwired module (code-only mode, the S1-D-3 aiStyle-28 precedent):**
//! none of the three kinds has an `NpcKind` yet, so there is no kind arm
//! to make unreachable — the dispatch claim (`update_rng`'s `41 =>` arm)
//! lands with the kinds (the S1-D-5 types work). The step is keyed on the
//! C# type id (`npc_type`), exactly like the arm's `this.type == 177` /
//! `== 378` branches; the wiring passes `kind.to_u16() as i32`.
//!
//! ## RNG order (by construction)
//!
//! W4-e2 wires the four dust sites through the S1-E-1 `DustPool` on the
//! caller's `UpdateNPCs` stream (`rng` + `dust` + `dust_ctx` parameters):
//! 1. the 378 fuse trail gate `Main.rand.Next(3)` (`:29916`) — on 0 the
//!    type-6 `NewDust` body rolls (site 29919; the type-6 tweak adds the
//!    `Next(-10, 6)` vy re-roll, `Dust.cs:215-219`).
//! 2. the `ai[2] > 7` explosion burst (`:29939-29958`): 8 type-31 spawns
//!    (site 29941), then 32 type-6 PAIRS (sites 29945/29948), then 2 gore
//!    iterations — the ARGUMENT draws `Next(100)` ×2, `Next(61, 64)`,
//!    `Next(-10, 11)` ×2 stay here in C# order (the `NewGore` bodies are
//!    W4-d55's lane, the twins' split).
//! 3. the sounds (`:29961` Item14, `:30083` style 29/124) — the port's
//!    AI arms carry no audio (the `LegacySoundPlayer` pitch jitter draws
//!    ride the audio pass, `LegacySoundPlayer.cs:613-632`).
//!
//! The witnesses below replay the cited draws on a cloned stream
//! (WRITTEN NOT RUN — code-only mode).
//!
//! ## The AI / integration split
//!
//! The C# arm's three `return`s (`:29971` explosion, `:29981` arming,
//! `:30120` no-target) exit `AI()` only — `UpdateNPC` STILL runs the
//! gravity + collision afterwards (`NPC.cs:90465-90503`). The port folds
//! that integration into the family step (the frost-legion shape,
//! `npc.rs` `update_frost_legion`), so `style41_step` runs `ai_041_body`
//! (which may return early, exactly like `AI()`) and THEN integrates on
//! every path — no tick is parked. Gravity 0.3 / maxFallSpeed 10
//! (`UpdateNPC_UpdateGravity`, `:90880-90881` — NOT the port's generic
//! `GRAVITY`/`MAX_FALL` 0.4/8, a pre-existing deviation outside this
//! task), the wet 0.5 slowdown (`Collision_MoveWhileWet`, `:92785-92804`,
//! `waterMovementSpeed` `:6015`), the ±0.005 f64 X deadband (`:90473`).
//! `oldVelocity` is captured before the resolve (`:92538`) and
//! `collideX`/`collideY` latch on a changed axis (`:92597-92604`) — the
//! next tick's wet bounce reads them. Slopes/platforms stay unported like
//! every family arm (the axis-separated `collides_rect` stand-in).
//!
//! ## Documented stand-ins
//!
//! * `NPC.wet` (`CheckFacetedLiquid`'s faceted cache, `:90474`'s read)
//!   rides the port's `is_wet` — any water tile overlapping the body
//!   (the swimmer's stand-in, `npc.rs:13821`); honey/lava/shimmer wet
//!   are unported, so only the water slowdown applies.
//! * `Main.player[this.target]` (`:29976`/`:30036-30037`/`:30125`) —
//!   the C# reads slot 255 (the inactive zero player) before the first
//!   target pick; the port's one session player stands in on every
//!   tick. The airborne `target >= 255` return (`:30118`) is kept
//!   literally.
//! * `life = -1; HitEffect(); active = false` (`:29966-29968`) rides
//!   the port's hp-0 death convention (the sim's retain drops it; the
//!   `HitEffect` gore is the same documented skip).
//! * The gore bodies (`:29954`) are W4-d55's lane (their ARGUMENT draws
//!   are consumed here); the audio arms stay the documented skips (the
//!   RNG header); the `netOffset` add/subtract pairs are kept verbatim
//!   around the spawns (zero in single-player).
#![allow(dead_code)] // wiring deferred (code-only mode) — the kinds + dispatch claim are the follow-up

use crate::npc::{is_wet, Npc};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::{collides_rect, Player};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World, TILE_PX};

/// `NPC.gravity` / `maxFallSpeed` for these kinds
/// (`UpdateNPC_UpdateGravity`, `NPC.cs:90880-90881` — no per-type
/// override touches 174/177/378).
const GRAVITY: f32 = 0.3;
const MAX_FALL: f32 = 10.0;
/// `NPC.waterMovementSpeed` (`NPC.cs:6015`) — the wet integration's
/// slowdown (`Collision_MoveWhileWet`, `:92791`).
const WATER_SLOWDOWN: f32 = 0.5;

/// The C# unchecked `(int)` float cast (ECMA-335 `conv.i4`): NaN and any
/// out-of-range value (incl. ±inf) become `int.MinValue` on the x86
/// `cvttss2si` the vanilla runs — NOT Rust's saturating `as i32`. The
/// arm's only cast is `ai[0] += (int)num615` (`NPC.cs:30045`); `num615`
/// is capped at 30 (`:30041-30044`) so the cast is in-range for every
/// finite position, but a NaN delta (a NaN position) keeps the C#
/// `int.MinValue` wrap instead of saturating to 0.
fn csharp_int_cast(x: f32) -> i32 {
    // The in-range window: [-2^31, 2^31). NaN fails `contains`.
    if (-2147483648.0..2147483648.0).contains(&x) {
        x as i32
    } else {
        i32::MIN
    }
}

/// The shared `TargetClosest()` (`NPC.cs:77652-77666`) through
/// `npc_target::target_closest` — the arm calls it at `:29909` (the
/// init latch), `:29993` (the wet floor bounce) and `:30051` (the hop
/// re-aim while `ai[2] == 1`). The candidate is the session player with
/// the stock unported fields (the S1-D-3p zombie precedent); the
/// `EverLastingTicker` stamp passes 0 (same stand-in).
fn pick_target(npc: &mut Npc, world: &World, player: &Player) {
    let cand = TargetCandidate {
        active: true,
        dead: player.dead,
        // The port has no hardcore ghosts (`Player.ghost`).
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        // `Player.aggro`/`npcTypeNoAggro`/`tankPet` are unported — the
        // stock 0/false/None (the S1-D-3p zombie precedent).
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    target_closest(npc, world, &[cand], true, 41, 0);
}

/// One fixed step of the aiStyle-41 family: the `AI()` block
/// (`NPC.cs:29899-30159`, `ai_041_body`) followed by the generic
/// post-AI integration on EVERY path (the module header). `npc_type` is
/// the C# `this.type` (174/177/378); the npc's box rides `npc.size()`
/// (the wiring's kinds carry the `SetDefaults` sizes; the 378 explosion
/// writes `size_override`).
pub fn style41_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    npc_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    // `NPC.wet` — the `CheckFacetedLiquid` cache (`NPC.cs:90474`'s write;
    // the swimmer's `is_wet` stand-in, `npc.rs:13821`), read with the
    // pre-resize box on the explosion tick.
    let (w0, h0) = npc.size();
    npc.wet = is_wet(world, npc.x, npc.y, w0, h0);

    ai_041_body(npc, world, player, npc_type, rng, dust, dust_ctx);

    // The generic post-AI integration (`UpdateNPC`, `NPC.cs:90465-90503`)
    // — runs even on the AI's early-return paths, exactly like the
    // vanilla. Gravity + fall cap (`:90465-90472`), the ±0.005 f64 X
    // deadband (`:90473-90476`), the wet 0.5 slowdown (`:92791`) and the
    // axis-separated tile resolve (the frost-legion stand-in for
    // `UpdateCollision`; slopes/platforms unported).
    npc.vy = (npc.vy + GRAVITY).min(MAX_FALL);
    if (npc.vx as f64) < 0.005 && (npc.vx as f64) > -0.005 {
        npc.vx = 0.0;
    }
    // `Entity.oldVelocity` (`:92538`) — captured before the resolve;
    // the wet bounce reads it next tick.
    npc.old_vx = npc.vx;
    npc.old_vy = npc.vy;
    npc.collide_x = false;
    npc.collide_y = false;
    let (w, h) = npc.size();
    let slow = if npc.wet { WATER_SLOWDOWN } else { 1.0 };
    let nx = npc.x + npc.vx * slow;
    if !collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
        npc.collide_x = true;
    }
    let ny = npc.y + npc.vy * slow;
    if !collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
        npc.collide_y = true;
    } else if npc.vy < 0.0 {
        let ty = (ny / TILE_PX).floor() as i32;
        npc.y = (ty + 1) as f32 * TILE_PX;
        npc.vy = 0.0;
        npc.collide_y = true;
    }
}

/// The `AI()` aiStyle-41 block (`NPC.cs:29899-30159`). The `return`s
/// mirror the C#'s exactly — they skip the rest of the AI, NOT the
/// caller's integration.
fn ai_041_body(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    npc_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    // `:29901-29904` — the cooldown decrement (stops at 1, never 0).
    if npc.ai[2] > 1.0 {
        npc.ai[2] -= 1.0;
    }
    // `:29905-29911` — the init latch: only the spawn's `ai[2] == 0`
    // reaches it (the decrement above floors at 1).
    if npc.ai[2] == 0.0 {
        npc.ai[0] = -100.0;
        npc.ai[2] = 1.0;
        pick_target(npc, world, player);
        npc.sprite_dir = npc.facing;
    }
    // `:29912-29973` — the Chattering Teeth Bomb's own block.
    if npc_type == 378 {
        // W4-e2 (site 29919) — `:29914-29925`: the fuse trail. vector72 =
        // (-6·spriteDirection, -10); the `ai[1] != 5 && Next(3) == 0`
        // gate (the C# && short-circuits — no draw on the explosion arm).
        let mut vector72 = Vec2::new(-6.0, -10.0);
        vector72.x *= npc.sprite_dir as f32;
        if npc.ai[1] != 5.0 && rng.next_max(3) == 0 {
            npc.x += npc.net_offset.x;
            npc.y += npc.net_offset.y;
            let (cx, cy) = npc.center();
            // `:29919` — NewDust(Center + vector72 − (5, 5), 4, 4, 6):
            // the 8-arg overload (speeds/alpha/color/scale default
            // 0/0/default/1). The type-6 tweak rolls inside the pool.
            let idx = dust.new_dust(
                rng, dust_ctx, world,
                Vec2::new(cx + vector72.x - 5.0, cy + vector72.y - 5.0),
                4, 4, 6, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.scale = 1.5; // `:29920`
            d.no_gravity = true; // `:29921`
            // `:29922` — velocity ×0.25 + Normalize(vector72).
            let len = (vector72.x * vector72.x + vector72.y * vector72.y).sqrt();
            let (nx, ny) = if len > 0.0 {
                (vector72.x / len, vector72.y / len)
            } else {
                (0.0, 0.0)
            };
            d.velocity = d.velocity * 0.25 + Vec2::new(nx, ny);
            // `:29923` — RotatedBy(−π/2 · direction), the double cos/sin
            // form (Utils.cs:1675-1684).
            let a = (-(std::f32::consts::PI / 2.0) * npc.facing as f32) as f64;
            let (cos, sin) = (a.cos() as f32, a.sin() as f32);
            let (vx, vy) = (d.velocity.x, d.velocity.y);
            d.velocity = Vec2::new(vx * cos - vy * sin, vx * sin + vy * cos);
            npc.x -= npc.net_offset.x;
            npc.y -= npc.net_offset.y;
        }
        if npc.ai[1] == 5.0 {
            // `:29928-29935` — the explosion: velocity zero, the box
            // swells to 160x160 about the centre. The half-extent adds
            // are INTEGER division (`base.width / 2`, `:29929-29930` —
            // 20/2 = 10, 26/2 = 13), the subtracts read the NEW 160 box
            // (`:29933-29934`, 160/2 = 80): the net shift is (-70, -67).
            npc.vx = 0.0;
            npc.vy = 0.0;
            let (w, h) = npc.size();
            npc.x += (w as i32 / 2) as f32;
            npc.y += (h as i32 / 2) as f32;
            npc.size_override = Some((160.0, 160.0));
            npc.x -= 160.0 / 2.0;
            npc.y -= 160.0 / 2.0;
            npc.dont_take_damage = true;
            npc.x += npc.net_offset.x;
            npc.y += npc.net_offset.y;
            // W4-e2 (sites 29941/29945/29948) — `:29937-29963`: the
            // ai[2] > 7 burst on the 160×160 box at position.
            if npc.ai[2] > 7.0 {
                // `:29939-29942` — 8 type-31 dusts, alpha 100, scale 1.5,
                // NO post-writes.
                for _ in 0..8 {
                    dust.new_dust(
                        rng, dust_ctx, world, Vec2::new(npc.x, npc.y),
                        160, 160, 31, 0.0, 0.0, 100, Color::default(), 1.5,
                    );
                }
                // `:29943-29951` — 32 type-6 pairs: scale 2.5 (noGravity,
                // velocity ×3) then scale 1.5 (velocity ×2, noGravity).
                for _ in 0..32 {
                    let idx = dust.new_dust(
                        rng, dust_ctx, world, Vec2::new(npc.x, npc.y),
                        160, 160, 6, 0.0, 0.0, 100, Color::default(), 2.5,
                    );
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true; // `:29946`
                    d.velocity *= 3.0; // `:29947`
                    let idx = dust.new_dust(
                        rng, dust_ctx, world, Vec2::new(npc.x, npc.y),
                        160, 160, 6, 0.0, 0.0, 100, Color::default(), 1.5,
                    );
                    let d = dust.slot_mut(idx);
                    d.velocity *= 2.0; // `:29949`
                    d.no_gravity = true; // `:29950`
                }
                // `:29952-29958` — the 2 gores: the ARGUMENT draws stay
                // here in C# order (the NewGore bodies are d55's lane):
                // per gore Next(100) ×2 (the offsets), Next(61, 64) (the
                // type), Next(-10, 11) ×2 (the velocity nudges).
                for _ in 0..2 {
                    let _ox = (160 * rng.next_max(100)) as f32 / 100.0;
                    let _oy = (160 * rng.next_max(100)) as f32 / 100.0;
                    let _gore_type = rng.next_range(61, 64);
                    let _nvx = rng.next_range(-10, 11) as f32 * 0.05;
                    let _nvy = rng.next_range(-10, 11) as f32 * 0.05;
                }
                // `:29959-29962` — the == 9 Item14 sound: the audio pass
                // owns it (the module header).
            }
            if npc.ai[2] == 1.0 {
                // `:29964-29969` — `life = -1; HitEffect(); active =
                // false`: the port's hp-0 death convention (the sim's
                // retain drops `hp <= 0`; `HitEffect`'s gore is the
                // same documented skip).
                npc.hp = 0;
            }
            npc.x -= npc.net_offset.x;
            npc.y -= npc.net_offset.y;
            // `:29971` — the explosion returns before the arming check
            // and the hop machine (the caller still integrates).
            return;
        }
    }
    // `:29974-29983` — the bomb's arming: wet OR within 64 px of the
    // player's centre (`Vector2.Distance`, float halves —
    // `Entity.cs:50-55`). The early return skips the hop machine.
    if npc_type == 378 && npc.ai[1] != 5.0 {
        let (cx, cy) = npc.center();
        let (pcx, pcy) = player.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        if npc.wet || dist < 64.0 {
            npc.ai[1] = 5.0;
            npc.ai[2] = 10.0;
            npc.net_update = true;
            return;
        }
    } else if npc.wet && npc_type != 177 {
        // `:29984-30013` — the wet bounce (the Herpling and the bomb;
        // the Derpling is excluded). NOTE the C# `else if`: a 378 with
        // `ai[1] != 5` took the branch above and NEVER runs this one.
        if npc.collide_x {
            // `:29988` — `base.direction *= -base.direction`: the C#
            // quirk is a SET TO -1 (dir * -dir = -dir² = -1 for ±1),
            // not a flip. Kept literally.
            npc.facing *= -npc.facing;
            npc.sprite_dir = npc.facing;
        }
        if npc.collide_y {
            // `:29993-30002` — re-aim, then the floor bounce reads the
            // PRE-resolve velocity (`Entity.oldVelocity`, `:92538`).
            pick_target(npc, world, player);
            if npc.old_vy < 0.0 {
                npc.vy = 5.0;
            } else {
                npc.vy -= 2.0;
            }
            npc.sprite_dir = npc.facing;
        }
        // `:30004-30012` — the buoyancy: damp above 4, lift 0.3, cap -4.
        if npc.vy > 4.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.3;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
    if npc.vy == 0.0 {
        // `:30016-30020` — the stuck detector: the last big hop anchored
        // `ai[3]` at its X; landing on the SAME X flips the direction
        // and arms the 300-tick no-retarget cooldown.
        if npc.ai[3] == npc.x {
            npc.facing *= -1;
            npc.ai[2] = 300.0;
        }
        // `:30021-30026` — clear the anchor, damp the walk, the f64
        // ±0.1 deadband (`(double)base.velocity.X`, `:30023`).
        npc.ai[3] = 0.0;
        npc.vx *= 0.8;
        if (npc.vx as f64) > -0.1 && (npc.vx as f64) < 0.1 {
            npc.vx = 0.0;
        }
        // `:30027-30034` — the per-kind windup rate: the Derpling 2,
        // everyone else 5.
        if npc_type == 177 {
            npc.ai[0] += 2.0;
        } else {
            npc.ai[0] += 5.0;
        }
        // `:30035-30045` — the distance-scaled windup bonus: the player
        // centre delta (FLOAT halves, `:30036-30037`), `400 / dist`
        // FIRST then the per-kind ×10/×5 (`:30039-30040` — the multiply
        // order is kept), the 30 cap, the `(int)` truncation.
        let (w, h) = npc.size();
        let cx = npc.x + w * 0.5;
        let cy = npc.y + h * 0.5;
        let num612 = player.x + crate::player::WIDTH * 0.5 - cx;
        let num613 = player.y + crate::player::HEIGHT * 0.5 - cy;
        let num614 = (num612 * num612 + num613 * num613).sqrt();
        let mut num615 = 400.0 / num614;
        num615 = if npc_type != 177 {
            num615 * 10.0
        } else {
            num615 * 5.0
        };
        if num615 > 30.0 {
            num615 = 30.0;
        }
        npc.ai[0] += csharp_int_cast(num615) as f32;
        if npc.ai[0] >= 0.0 {
            // `:30046-30109` — the hop.
            npc.net_update = true;
            if npc.ai[2] == 1.0 {
                // `:30049-30052` — re-aim only outside the stuck
                // cooldown (the 300 countdown keeps the flipped side).
                pick_target(npc, world, player);
            }
            // The 200 < dist < 350 band bonus (`:30059`/`:30071`/
            // `:30089`/`:30101` — STRICT on both ends, one extra
            // direction step).
            let band = num614 < 350.0 && num614 > 200.0;
            if npc_type == 177 {
                // `:30053-30077` — the Derpling's 3-hop cycle: two
                // -7.5/4 hops then the -11.5/2 big hop.
                if npc.ai[1] == 2.0 {
                    npc.vy = -11.5;
                    npc.vx += 2.0 * npc.facing as f32;
                    if band {
                        npc.vx += npc.facing as f32;
                    }
                    npc.ai[0] = -200.0;
                    npc.ai[1] = 0.0;
                    npc.ai[3] = npc.x;
                } else {
                    npc.vy = -7.5;
                    // `:30070` — `4 * base.direction` is an INT multiply
                    // widened to float; ±4 either way, the value is kept.
                    npc.vx += (4 * npc.facing) as f32;
                    if band {
                        npc.vx += npc.facing as f32;
                    }
                    npc.ai[0] = -120.0;
                    npc.ai[1] += 1.0;
                }
            } else {
                // `:30079-30108` — the Herpling/bomb 4-hop cycle: three
                // -5/5 hops then the -9/3 big hop.
                if npc_type == 378 {
                    // `:30081-30084` — `PlaySound(29, x, y, 124)`: the
                    // documented audio skip (draw 3).
                }
                if npc.ai[1] == 3.0 {
                    npc.vy = -9.0;
                    npc.vx += (3 * npc.facing) as f32;
                    if band {
                        npc.vx += npc.facing as f32;
                    }
                    npc.ai[0] = -200.0;
                    npc.ai[1] = 0.0;
                    npc.ai[3] = npc.x;
                } else {
                    npc.vy = -5.0;
                    npc.vx += (5 * npc.facing) as f32;
                    if band {
                        npc.vx += npc.facing as f32;
                    }
                    npc.ai[0] = -120.0;
                    npc.ai[1] += 1.0;
                }
            }
        } else if npc.ai[0] >= -30.0 {
            // `:30110-30113` — the windup frame hint (`FindFrame`'s
            // `aiAction == 1` row, `:66701-66704`). The C# never clears
            // it inside this arm — the quirk is kept.
            npc.ai_action = 1;
        }
        // `:30114` — the grounded sprite lockstep.
        npc.sprite_dir = npc.facing;
    } else {
        // `:30116-30158` — the airborne steer.
        if npc.target >= 255 {
            // `:30118-30121` — no target: the AI returns before the
            // steer (the caller still integrates, like the vanilla).
            return;
        }
        if npc_type == 177 {
            // `:30122-30146` — the Derpling's above-player hover damp.
            let (w, h) = npc.size();
            let mut flag49 = false;
            if npc.y + h < player.y
                && npc.x + w > player.x
                && npc.x < player.x + crate::player::WIDTH
            {
                flag49 = true;
                npc.vx *= 0.92;
                if npc.vy < 0.0 {
                    npc.vy *= 0.9;
                    npc.vy += 0.1;
                }
            }
            if !flag49
                && ((npc.facing == 1 && npc.vx < 4.0) || (npc.facing == -1 && npc.vx > -4.0))
            {
                // `:30137` — the f64 inner gate: accelerate while the
                // velocity is behind the facing's +0.1/-0.1 line, damp
                // otherwise (the backwards-moving case).
                if (npc.facing == -1 && (npc.vx as f64) < 0.1)
                    || (npc.facing == 1 && (npc.vx as f64) > -0.1)
                {
                    npc.vx += 0.2 * npc.facing as f32;
                } else {
                    npc.vx *= 0.93;
                }
            }
        } else if (npc.facing == 1 && npc.vx < 3.0) || (npc.facing == -1 && npc.vx > -3.0) {
            // `:30147-30157` — the Herpling/bomb steer: the same shape
            // with the 3 cap.
            if (npc.facing == -1 && (npc.vx as f64) < 0.1)
                || (npc.facing == 1 && (npc.vx as f64) > -0.1)
            {
                npc.vx += 0.2 * npc.facing as f32;
            } else {
                npc.vx *= 0.93;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::UnifiedRandom;

    /// The jungle arena: air above a dirt floor at tile y = 120 (the
    /// npc.rs `ground_air` shape).
    fn arena() -> World {
        let mut w = World::new("t".into(), 400, 200);
        for x in 0..400 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn player_at(w: &World, x: f32, y: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = x;
        p.y = y;
        p.dead = false;
        p
    }

    /// W4-e2: the permissive dust ctx for the witnesses — the whole world
    /// on-screen, the full 6000 budget (the `npc::permissive_dust_ctx`
    /// shape, local so the module keeps no npc.rs test dependency).
    fn permissive_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    /// A stand-in carrier with the kind's `SetDefaults` box — the kinds
    /// have no `NpcKind` yet (the module header); the wiring's kinds
    /// carry these sizes natively. Grounded at the floor: the box's
    /// bottom sits on tile y 120 (px 1920).
    fn hopper(npc_type: i32, x: f32) -> Npc {
        let (w, h) = match npc_type {
            174 => (52.0, 26.0), // `NPC.cs:11197-11198`
            177 => (58.0, 44.0), // `:11241-11242`
            378 => (20.0, 26.0), // `:13828-13829`
            _ => unreachable!(),
        };
        let mut n = Npc::new(NpcKind::Zombie, x, 1920.0 - h, 1, 0.0);
        n.size_override = Some((w, h));
        n
    }

    /// `NPC.cs:29905-29911` — the spawn latch: ai[2] 0 → ai[0] = -100,
    /// ai[2] = 1, the shared TargetClosest picks slot 0, the sprite
    /// rides the facing. The decrement floors at 1 (`:29901-29904`), so
    /// the latch never re-fires (the windup keeps climbing ai[0]).
    #[test]
    fn s1_d_3_s41_the_init_latch_fires_once() {
        let w = arena();
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(174, 80.0 * 16.0);
        assert_eq!(n.target, 255);
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[2], 1.0, "the latch armed the cooldown");
        assert_eq!(n.target, 0, "TargetClosest committed the slot");
        assert_eq!(n.sprite_dir, n.facing);
        // The -100 reset ran, then the grounded windup added to it.
        let ai0 = n.ai[0];
        assert!(ai0 > -100.0 && ai0 < 0.0, "reset to -100 then windup: {ai0}");
        // The next tick: ai[2] stays 1 and ai[0] keeps climbing (no
        // second -100 reset).
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[2], 1.0);
        assert!(n.ai[0] > ai0, "the latch never re-fires");
    }

    /// `NPC.cs:30016-30026` — the stuck detector flips the direction and
    /// arms the 300 cooldown; the anchor clears; the f64 ±0.1 deadband
    /// zeroes the damped walk.
    #[test]
    fn s1_d_3_s41_the_stuck_flip_and_deadband() {
        let w = arena();
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(174, 80.0 * 16.0);
        n.ai[2] = 1.0;
        n.ai[0] = -100000.0;
        n.ai[3] = n.x; // the big hop anchored here
        n.vx = 0.1; // ×0.8 = 0.08 < 0.1 → zeroed
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.facing, -1, "the stuck flip");
        assert_eq!(n.ai[2], 300.0, "the no-retarget cooldown");
        assert_eq!(n.ai[3], 0.0, "the anchor cleared");
        assert_eq!(n.vx, 0.0, "the deadband zeroed the walk");
        // The cooldown counts down 1/tick and never re-fires the latch.
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[2], 299.0);
        // A wide velocity survives the deadband: 1.0 × 0.8 = 0.8.
        let mut m = hopper(174, 80.0 * 16.0);
        m.ai[2] = 1.0;
        m.ai[0] = -100000.0;
        m.vx = 1.0;
        style41_step(&mut m, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(m.vx, 0.8);
    }

    /// `NPC.cs:30035-30045` — the windup bonus: 400/dist FIRST, then
    /// ×10 (Herpling) / ×5 (Derpling), the 30 cap, the int truncation.
    /// At dist 800: 0.5×10 = 5 / 0.5×5 = 2 (the +5/+2 base on top).
    #[test]
    fn s1_d_3_s41_the_distance_scaled_windup() {
        let w = arena();
        // The player's centre exactly 800 px right of the hopper's.
        let mut n = hopper(174, 80.0 * 16.0);
        let (cx, cy) = n.center();
        let player = player_at(
            &w,
            cx + 800.0 - crate::player::WIDTH * 0.5,
            cy - crate::player::HEIGHT * 0.5,
        );
        n.ai[2] = 1.0;
        n.ai[0] = -1000.0;
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[0], -1000.0 + 5.0 + 5.0, "base 5 + (int)(400/800×10)");
        let mut d = hopper(177, 80.0 * 16.0);
        let (dcx, dcy) = d.center();
        let dplayer = player_at(
            &w,
            dcx + 800.0 - crate::player::WIDTH * 0.5,
            dcy - crate::player::HEIGHT * 0.5,
        );
        d.ai[2] = 1.0;
        d.ai[0] = -1000.0;
        style41_step(&mut d, &w, &dplayer, 177, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(d.ai[0], -1000.0 + 2.0 + 2.0, "base 2 + (int)(400/800×5)");
        // Near player: dist 100 → 4×10 = 40 → capped 30.
        let mut near = hopper(174, 80.0 * 16.0);
        let (ncx, ncy) = near.center();
        let nplayer = player_at(
            &w,
            ncx + 100.0 - crate::player::WIDTH * 0.5,
            ncy - crate::player::HEIGHT * 0.5,
        );
        near.ai[2] = 1.0;
        near.ai[0] = -1000.0;
        style41_step(&mut near, &w, &nplayer, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(near.ai[0], -1000.0 + 5.0 + 30.0, "the 30 cap");
    }

    /// `NPC.cs:30085-30107` — the Herpling's 4-hop cycle: three -5/5
    /// small hops (ai[0] = -120, ai[1]++) then the -9/3 big hop
    /// (ai[0] = -200, ai[1] = 0, ai[3] anchored).
    #[test]
    fn s1_d_3_s41_the_herpling_hop_cycle() {
        let w = arena();
        let player = player_at(&w, 200.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(174, 80.0 * 16.0);
        n.ai[2] = 1.0;
        for hop in 0..4 {
            // Grounded at the floor with the windup ready.
            n.y = 1920.0 - 26.0;
            n.vy = 0.0;
            n.vx = 0.0;
            n.ai[0] = 0.0;
            let x0 = n.x;
            style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
            if hop < 3 {
                assert_eq!(n.vy, -5.0 + GRAVITY, "small hop + gravity");
                assert_eq!(n.ai[0], -120.0);
                assert_eq!(n.ai[1], (hop + 1) as f32);
                assert_eq!(n.ai[3], 0.0, "no anchor on the small hops");
            } else {
                assert_eq!(n.vy, -9.0 + GRAVITY, "big hop + gravity");
                assert_eq!(n.ai[0], -200.0);
                assert_eq!(n.ai[1], 0.0);
                assert_eq!(n.ai[3], x0, "the big hop anchors the X");
            }
            assert!(n.net_update, "the hop arms the net flag (:30048)");
        }
    }

    /// `NPC.cs:30053-30077` — the Derpling's 3-hop cycle: two -7.5/4
    /// hops then the -11.5/2 big hop; the windup base is 2.
    #[test]
    fn s1_d_3_s41_the_derpling_hop_cycle() {
        let w = arena();
        let player = player_at(&w, 200.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(177, 80.0 * 16.0);
        n.ai[2] = 1.0;
        for hop in 0..3 {
            n.y = 1920.0 - 44.0;
            n.vy = 0.0;
            n.vx = 0.0;
            n.ai[0] = 0.0;
            style41_step(&mut n, &w, &player, 177, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
            if hop < 2 {
                assert_eq!(n.vy, -7.5 + GRAVITY);
                assert_eq!(n.ai[0], -120.0);
                assert_eq!(n.ai[1], (hop + 1) as f32);
            } else {
                assert_eq!(n.vy, -11.5 + GRAVITY);
                assert_eq!(n.ai[0], -200.0);
                assert_eq!(n.ai[1], 0.0);
            }
        }
    }

    /// `NPC.cs:30059`/`:30089` — the band bonus is STRICT on both ends:
    /// dist 200 and 350 get nothing, 250 gets the extra direction step.
    #[test]
    fn s1_d_3_s41_the_band_bonus_is_strict() {
        let w = arena();
        for (dist, bonus) in [(200.0f32, 0.0f32), (350.0, 0.0), (250.0, 1.0)] {
            let mut n = hopper(174, 80.0 * 16.0);
            let (cx, cy) = n.center();
            let player = player_at(
                &w,
                cx + dist - crate::player::WIDTH * 0.5,
                cy - crate::player::HEIGHT * 0.5,
            );
            n.ai[2] = 1.0;
            n.ai[0] = 0.0;
            n.ai[1] = 3.0; // the big hop: vx += 3*dir (+band)
            style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
            // The hop wrote 3+bonus; the airborne steer is skipped this
            // tick (vy was 0 at the branch) and the integration keeps vx.
            assert_eq!(n.vx, 3.0 + bonus, "dist {dist}");
        }
    }

    /// `NPC.cs:29974-29983` — the bomb arms on wet OR centre distance
    /// < 64: ai[1] = 5, ai[2] = 10, netUpdate, and the early return
    /// skips the hop machine (the caller still integrates — the grounded
    /// bomb stays put).
    #[test]
    fn s1_d_3_s41_the_bomb_arms_and_returns() {
        let w = arena();
        let mut n = hopper(378, 80.0 * 16.0);
        let (cx, cy) = n.center();
        // 63 px away: inside the 64 window.
        let player = player_at(
            &w,
            cx + 63.0 - crate::player::WIDTH * 0.5,
            cy - crate::player::HEIGHT * 0.5,
        );
        n.ai[2] = 1.0;
        let (x0, y0) = (n.x, n.y);
        style41_step(&mut n, &w, &player, 378, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[1], 5.0);
        assert_eq!(n.ai[2], 10.0);
        assert!(n.net_update);
        assert_eq!((n.x, n.y), (x0, y0), "the grounded arming tick stays put");
        // 65 px away, dry: no arming.
        let mut m = hopper(378, 80.0 * 16.0);
        let (mcx, mcy) = m.center();
        let far = player_at(
            &w,
            mcx + 65.0 - crate::player::WIDTH * 0.5,
            mcy - crate::player::HEIGHT * 0.5,
        );
        m.ai[2] = 1.0;
        m.ai[0] = -100000.0;
        style41_step(&mut m, &w, &far, 378, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(m.ai[1], 0.0, "outside the window the bomb keeps hopping");
    }

    /// `NPC.cs:29926-29971` — the explosion: the integer-half centring
    /// (net shift -70, -67 for the 20x26 box), the 160x160 swell, the
    /// dontTakeDamage latch, and the 9-tick fuse (ai[2] 10 → 1: the
    /// head decrement runs first, the == 1 death fires on tick 9). The
    /// bomb hangs in the air so the post-return gravity is a clean fall.
    #[test]
    fn s1_d_3_s41_the_bomb_fuse_and_integer_centring() {
        let w = arena();
        let player = player_at(&w, 200.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(378, 80.0 * 16.0);
        n.y = 1000.0; // well above the floor — the fuse falls freely
        let (x0, y0) = (n.x, n.y);
        n.ai[1] = 5.0;
        n.ai[2] = 10.0;
        n.vx = 7.0;
        n.vy = 3.0;
        style41_step(&mut n, &w, &player, 378, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[2], 9.0, "the head decrement ran first");
        assert_eq!(n.x, x0 + 10.0 - 80.0, "20/2 = 10 in, 160/2 = 80 out");
        // The centring (-67) then the post-return gravity fall (+0.3).
        assert_eq!(n.y, y0 + 13.0 - 80.0 + GRAVITY);
        assert_eq!(n.size_override, Some((160.0, 160.0)));
        assert!(n.dont_take_damage);
        assert!(n.hp > 0, "tick 1 of the fuse: still alive");
        // Ticks 2..8: the 160-box centring nets zero (+80 -80), so X is
        // stable; each tick's velocity reset + gravity falls 0.3.
        let x1 = n.x;
        for _ in 0..7 {
            style41_step(&mut n, &w, &player, 378, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        }
        assert_eq!(n.ai[2], 2.0);
        assert_eq!(n.x, x1, "the 160-box centring nets zero on X");
        assert!(n.hp > 0);
        // Tick 9: ai[2] 2 → 1, the death fires.
        style41_step(&mut n, &w, &player, 378, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.hp, 0, "life = -1 / active = false → the hp-0 convention");
    }

    /// `NPC.cs:29984-30013` — the wet bounce: the collideX quirk SETS
    /// the direction to -1 (`dir *= -dir`), the collideY bounce reads
    /// oldVelocity (rising → 5, falling → -2), the buoyancy damps above
    /// 4, lifts 0.3 and caps at -4. The Derpling (177) is excluded.
    #[test]
    fn s1_d_3_s41_the_wet_bounce_and_buoyancy() {
        let mut w = arena();
        // Flood the arena's air rows with water.
        for x in 0..400 {
            for y in 0..120 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        // The collideX quirk with NO collideY re-aim: the player is to
        // the RIGHT, yet `dir *= -dir` sets the facing to -1 (a flip
        // would also give -1 here, but a re-aim would give +1 — the
        // absence of collideY proves the quirk, not a re-aim).
        let mut n = hopper(174, 80.0 * 16.0);
        n.ai[2] = 1.0;
        n.ai[0] = -100000.0;
        n.facing = 1;
        n.collide_x = true;
        n.collide_y = false;
        n.vy = 0.0;
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.facing, -1, "the dir *= -dir quirk is a SET TO -1");
        // The collideY bounce reads oldVelocity: rising → 5, then the
        // buoyancy (5 > 4 → ×0.95, -0.3) and the post-return gravity.
        // Airborne so the floor never zeroes the bounce.
        let mut m = hopper(174, 80.0 * 16.0);
        m.y -= 100.0;
        m.ai[2] = 1.0;
        m.ai[0] = -100000.0;
        m.collide_y = true;
        m.old_vy = -3.0; // was rising
        m.vy = 0.0;
        style41_step(&mut m, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(m.vy, 5.0 * 0.95 - 0.3 + GRAVITY);
        // A falling oldVelocity takes the -2 arm instead.
        let mut f = hopper(174, 80.0 * 16.0);
        f.y -= 100.0;
        f.ai[2] = 1.0;
        f.ai[0] = -100000.0;
        f.collide_y = true;
        f.old_vy = 3.0;
        f.vy = 0.0;
        style41_step(&mut f, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(f.vy, -2.0 - 0.3 + GRAVITY, "the falling bounce -2, lift 0.3");
        // The buoyancy cap: a strong rise floors at -4 before gravity.
        let mut c = hopper(174, 80.0 * 16.0);
        c.y -= 100.0;
        c.ai[2] = 1.0;
        c.target = 0;
        c.vy = -10.0;
        style41_step(&mut c, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(c.vy, -4.0 + GRAVITY, "the -4 cap");
        // The Derpling never runs the wet block.
        let mut d = hopper(177, 80.0 * 16.0);
        d.y -= 100.0;
        d.ai[2] = 1.0;
        d.target = 0;
        d.collide_x = true;
        d.facing = 1;
        d.vy = -10.0;
        style41_step(&mut d, &w, &player, 177, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(d.facing, 1, "177 is excluded from the wet bounce");
        assert_eq!(d.vy, -10.0 + GRAVITY, "no buoyancy for the Derpling");
    }

    /// `NPC.cs:30116-30158` — the airborne steer: the 3 cap (Herpling)
    /// vs the 4 cap (Derpling), the 0.2 accel / 0.93 damp split on the
    /// f64 inner gate, and the target >= 255 early return (no steer, but
    /// the caller still integrates).
    #[test]
    fn s1_d_3_s41_the_airborne_steer_caps_and_gates() {
        let w = arena();
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        // Airborne (vy != 0), facing right, slow: accelerate 0.2.
        let mut n = hopper(174, 80.0 * 16.0);
        n.ai[2] = 1.0;
        n.target = 0;
        n.vy = -5.0;
        n.vx = 1.0;
        n.facing = 1;
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.vx, 1.2, "the 0.2 accel");
        // At the 3 cap: no steer at all.
        let mut capped = hopper(174, 80.0 * 16.0);
        capped.ai[2] = 1.0;
        capped.target = 0;
        capped.vy = -5.0;
        capped.vx = 3.0;
        capped.facing = 1;
        style41_step(&mut capped, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(capped.vx, 3.0, "the cap blocks the steer");
        // Moving backwards (dir 1, vx <= -0.1): the 0.93 damp.
        let mut back = hopper(174, 80.0 * 16.0);
        back.ai[2] = 1.0;
        back.target = 0;
        back.vy = -5.0;
        back.vx = -1.0;
        back.facing = 1;
        style41_step(&mut back, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(back.vx, -1.0 * 0.93, "the backwards damp");
        // The Derpling's cap is 4.
        let mut d = hopper(177, 80.0 * 16.0);
        d.ai[2] = 1.0;
        d.target = 0;
        d.vy = -5.0;
        d.vx = 3.5;
        d.facing = 1;
        style41_step(&mut d, &w, &player, 177, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(d.vx, 3.7, "the Derpling steers up to 4");
        // No target: the early return skips the steer (vx untouched)
        // but the caller still integrates the gravity.
        let mut t = hopper(174, 80.0 * 16.0);
        t.ai[2] = 1.0;
        t.target = 255;
        t.vy = -5.0;
        t.vx = 1.0;
        style41_step(&mut t, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(t.vx, 1.0, "no steer without a target");
        assert_eq!(t.vy, -5.0 + GRAVITY, "the integration still runs");
    }

    /// `NPC.cs:30122-30134` — the Derpling's above-player hover damp:
    /// the box fully above the player's top with X overlap damps vx
    /// ×0.92 and a rising vy ×0.9 +0.1, and SKIPS the steer.
    #[test]
    fn s1_d_3_s41_the_derpling_hover_damp() {
        let w = arena();
        // The player directly below the Derpling's box.
        let mut n = hopper(177, 80.0 * 16.0);
        let player = player_at(&w, n.x + 10.0, n.y + 44.0 + 10.0);
        n.ai[2] = 1.0;
        n.target = 0;
        n.vy = -5.0;
        n.vx = 2.0;
        n.facing = 1;
        style41_step(&mut n, &w, &player, 177, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        // vx: ×0.92 = 1.84 (no steer — flag49 blocks it).
        assert_eq!(n.vx, 2.0 * 0.92);
        // vy: ×0.9 +0.1 = -4.4, then gravity +0.3 = -4.1.
        assert_eq!(n.vy, -5.0 * 0.9 + 0.1 + GRAVITY);
    }

    /// `NPC.cs:30110-30113` — the windup frame hint: ai[0] in [-30, 0)
    /// writes aiAction = 1; the arm never clears it (the C# quirk).
    #[test]
    fn s1_d_3_s41_the_windup_frame_hint_latches() {
        let w = arena();
        let player = player_at(&w, 200.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(174, 80.0 * 16.0);
        n.ai[2] = 1.0;
        n.ai[0] = -30.0; // +5 + bonus keeps it < 0 but >= -30
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai_action, 1);
        // A later hop tick never clears it.
        n.ai[0] = 0.0;
        style41_step(&mut n, &w, &player, 174, &mut UnifiedRandom::new(0), &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai_action, 1, "the C# never clears aiAction here");
    }

    /// The `(int)` cast helper: in-range truncates toward zero, NaN and
    /// ±inf wrap to int.MinValue (the C# unchecked conv.i4).
    #[test]
    fn s1_d_3_s41_the_int_cast_matches_csharp() {
        assert_eq!(csharp_int_cast(5.9), 5);
        assert_eq!(csharp_int_cast(-5.9), -5);
        assert_eq!(csharp_int_cast(30.0), 30);
        assert_eq!(csharp_int_cast(f32::NAN), i32::MIN);
        assert_eq!(csharp_int_cast(f32::INFINITY), i32::MIN);
        assert_eq!(csharp_int_cast(f32::NEG_INFINITY), i32::MIN);
        assert_eq!(csharp_int_cast(2147483648.0), i32::MIN);
        assert_eq!(csharp_int_cast(-2147483648.0), i32::MIN);
    }

    /// W4-e2: the determinism witness re-armed — same-seed streams +
    /// fresh pools produce identical state across 200 steps of every
    /// kind (incl. the bomb's full fuse, the dustiest path), and the two
    /// streams stay equal draw-for-draw.
    #[test]
    fn s1_d_3_s41_the_arm_is_deterministic_without_an_rng() {
        let w = arena();
        let dc = permissive_ctx(&w);
        let player = player_at(&w, 100.0 * 16.0, 110.0 * 16.0);
        for npc_type in [174, 177, 378] {
            let mut a = hopper(npc_type, 80.0 * 16.0);
            let mut b = hopper(npc_type, 80.0 * 16.0);
            let mut ra = UnifiedRandom::new(777);
            let mut rb = UnifiedRandom::new(777);
            let mut pa = DustPool::new();
            let mut pb = DustPool::new();
            for _ in 0..200 {
                style41_step(&mut a, &w, &player, npc_type, &mut ra, &mut pa, &dc);
                style41_step(&mut b, &w, &player, npc_type, &mut rb, &mut pb, &dc);
                assert_eq!((a.x, a.y, a.vx, a.vy), (b.x, b.y, b.vx, b.vy));
                assert_eq!(a.ai, b.ai);
                assert_eq!(a.hp, b.hp);
                assert_eq!(ra, rb, "the streams stay equal");
            }
        }
    }

    /// W4-e2 (site 29919) — the fuse trail: the `ai[1] != 5 && Next(3)`
    /// gate draws ONLY on the non-explosion arm (the C# && short-circuit);
    /// a passed gate rolls the type-6 body at Center + vector72 − (5, 5)
    /// and the post-writes pin scale 1.5 / noGravity / the rotated
    /// velocity. The mirror replays gate + body on a cloned stream.
    #[test]
    fn w4_e2_the_bomb_fuse_trail_gates_and_shapes() {
        let w = arena();
        let dc = permissive_ctx(&w);
        let player = player_at(&w, 200.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(378, 80.0 * 16.0);
        n.ai[2] = 1.0;
        n.ai[0] = -100000.0; // no hop this tick
        n.sprite_dir = 1;
        n.facing = 1;
        let mut rng = UnifiedRandom::new(5);
        let mut probe = rng.clone();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let (cx, cy) = n.center();
        style41_step(&mut n, &w, &player, 378, &mut rng, &mut pool, &dc);
        let g = probe.next_max(3); // the fuse gate (:29916)
        if g == 0 {
            // vector72 = (-6·1, -10); anchor Center + vector72 − (5, 5).
            let idx = mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(cx - 11.0, cy - 15.0),
                4, 4, 6, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.scale = 1.5;
            d.no_gravity = true;
            let len = (36.0f32 + 100.0).sqrt();
            d.velocity = d.velocity * 0.25 + Vec2::new(-6.0 / len, -10.0 / len);
            let a = (-(std::f32::consts::PI / 2.0)) as f64;
            let (vx, vy) = (d.velocity.x, d.velocity.y);
            d.velocity = Vec2::new(
                vx * a.cos() as f32 - vy * a.sin() as f32,
                vx * a.sin() as f32 + vy * a.cos() as f32,
            );
            let act: Vec<usize> = (0..6000).filter(|&i| pool.slot(i).active).collect();
            assert_eq!(act.len(), 1, "one fuse dust");
            let pd = pool.slot(act[0]);
            assert_eq!(pd.scale, 1.5);
            assert!(pd.no_gravity);
            assert_eq!(pd.position, mpool.slot(idx).position);
            assert_eq!(pd.velocity, mpool.slot(idx).velocity);
        }
        assert_eq!(rng, probe, "gate + body draw order");
        // The explosion arm (ai[1] == 5): the fuse gate NEVER draws.
        let mut m = hopper(378, 80.0 * 16.0);
        m.ai[1] = 5.0;
        m.ai[2] = 5.0; // <= 7: no burst either
        let mut rng2 = UnifiedRandom::new(6);
        let mut probe2 = rng2.clone();
        style41_step(&mut m, &w, &player, 378, &mut rng2, &mut DustPool::new(), &dc);
        assert_eq!(rng2, probe2, "the explosion arm draws nothing at ai[2] <= 7");
    }

    /// W4-e2 (sites 29941/29945/29948) — the ai[2] > 7 explosion burst:
    /// 8 type-31 spawns, then 32 type-6 pairs, then the 2 gores' argument
    /// draws (5 each). The mirror replays every draw through the same
    /// pool calls on a cloned stream and compares full state.
    #[test]
    fn w4_e2_the_bomb_burst_draw_order() {
        let w = arena();
        let dc = permissive_ctx(&w);
        let player = player_at(&w, 200.0 * 16.0, 110.0 * 16.0);
        let mut n = hopper(378, 80.0 * 16.0);
        n.ai[1] = 5.0;
        n.ai[2] = 10.0;
        let mut rng = UnifiedRandom::new(11);
        let mut probe = rng.clone();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        style41_step(&mut n, &w, &player, 378, &mut rng, &mut pool, &dc);
        // The 160-box centring: (x + 10 − 80, y + 13 − 80).
        let (bx, by) = (80.0 * 16.0 + 10.0 - 80.0, 1920.0 - 26.0 + 13.0 - 80.0);
        for _ in 0..8 {
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(bx, by), 160, 160,
                31, 0.0, 0.0, 100, Color::default(), 1.5,
            );
        }
        for _ in 0..32 {
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(bx, by), 160, 160,
                6, 0.0, 0.0, 100, Color::default(), 2.5,
            );
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(bx, by), 160, 160,
                6, 0.0, 0.0, 100, Color::default(), 1.5,
            );
        }
        for _ in 0..2 {
            probe.next_max(100); // the gore offset X (:29954)
            probe.next_max(100); // the gore offset Y
            probe.next_range(61, 64); // the gore type
            probe.next_range(-10, 11); // the velocity nudge X (:29956)
            probe.next_range(-10, 11); // the velocity nudge Y (:29957)
        }
        assert_eq!(rng, probe, "8 + 64 spawns + 10 gore args in C# order");
        let act: Vec<usize> = (0..6000).filter(|&i| pool.slot(i).active).collect();
        assert_eq!(act.len(), 72, "8 + 32×2 dusts (the permissive pool never culls)");
        assert_eq!(n.ai[2], 9.0, "the head decrement ran first");
    }
}
