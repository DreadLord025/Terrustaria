//! S1-D-3 (aiStyle-16) — the fish/swimmer family, `NPC.AI`'s
//! `if (this.aiStyle == 16)` block (`NPC.cs:23492-24021`), ported 1:1 as a
//! PURE velocity machine (code-only mode: no compiler/tests were run on this
//! branch; the witness tests below are written, not executed).
//!
//! The C# family is NOT the spiders — aiStyle 16 is the flopping fish. The
//! 13 vanilla members (`SetDefaults`' `this.aiStyle = 16;` rows):
//! Goldfish 55 (`NPC.cs:9465`), CorruptGoldfish 57 (`:9494`), Piranha 58
//! (`:9508`), Shark 65 (`:9615`), AnglerFish 102 (`:10208`), Arapaima 157
//! (`:10995`), BloodFeeder 241 (`:12111`), CrimsonGoldfish 465 (`:14488`),
//! GoldGoldfish 592 (`:16684`), Pupfish 607 (`:16800`), Dolphin 615
//! (`:16897`), Pufferfish 688 (`:17658`), Orca 692 (`:17714`) —
//! `NPCID.cs:10787`/`:10791`/`:10793`/`:10807`/`:10881`/`:10991`/`:11159`/
//! `:11603`/`:11861`/`:11891`/`:11907`/`:12053`/`:12061`.
//! The port's roster carries three of them (`NpcKind::Goldfish`/`Piranha`/
//! `Shark`, `npc/data.rs`'s aiStyle-16 row); the per-type branches for the
//! other ten are ported verbatim and reachable through [`ai_016_arm`]'s
//! `ty` parameter (the witness tests drive them with a stand-in body).
//!
//! W4-c4 rosters NINE of the ten (CorruptGoldfish 57 / Arapaima 157 /
//! BloodFeeder 241 / CrimsonGoldfish 465 / GoldGoldfish 592 / Pupfish 607 /
//! Dolphin 615 / Pufferfish 688 / Orca 692) through the SAME
//! `update_swimmer` family arm — [`vanilla_type`] maps each kind to its
//! `cs_id`. The tenth, AnglerFish 102, is the W4-d35 roster lane (its
//! lure-light [`Ai16Effects::light`] stays write-only until that kind
//! lands). The Orca's DRAW sheet is atlas-deferred (the 136x992 strip
//! overflows the 8192 bound — the wyvern precedent); its AI + stats are
//! fully wired.
//!
//! WIRED (the S1-D-3 bat/fish reconcile, `task/S1-D-3-reconcile-batfish`):
//! `npc.rs::update_swimmer` routes the three kinds through [`ai_016_step`]
//! (the `is_wet` pass + the session-player `Ai16Ctx`; `justHit` rides the
//! established `hurt_cd > 0` stand-in — only the unported Pufferfish reads
//! it) and integrates through the no-gravity `integrate_flyer_collision`
//! stand-in (`UpdateCollision`, `NPC.cs:92523-92580`; the wet halving is
//! `waterMovementSpeed` 0.5, `:6015`/`:92561`). The bob phase rides the
//! C#'s own `ai[0]` now — the port's `swim_phase` mirror and the
//! `npc_ai::swim_step` approximation are retired. The distrust-verbatim
//! audit re-read the whole arm against `NPC.cs:23492-24021` at the wiring
//! — verdict FAITHFUL (the Dolphin draw-before-increment order, the flop
//! triple order, the Arapaima snap caps and the 157 damp exemption all
//! match; no f1701eb0-class defect found).
//!
//! Seam: the module only writes the
//! fields the C# AI writes — velocity, `ai`/`localAI`, `direction`
//! (`Npc::facing`), `directionY`, `rotation`, `netUpdate` — and reads
//! `wet`/`collideX`/`collideY` a tick late, exactly like the source (the
//! collision pass sets them in `UpdateNPC`, `NPC.cs:92538+`). It does NOT
//! integrate position: the C# arm ends before `UpdateCollision`. The
//! `Lighting.AddLight` call (`:23764`) leaves as an [`Ai16Effects`] request;
//! the `SoundEngine.PlaySound(45, ...)` (`:23680`) is a documented skip —
//! its gating `Main.rand.Next(2)` draw is KEPT so the stream cannot drift.
//! `Main.netMode != 1` folds to true (the port is single-player);
//! `NetUpdateIgnoreSpamLimit` (`NPC.cs:90568-90576`) folds to
//! `netUpdate = true` (the `netSpam` counter is unported).
//!
//! RNG by construction: the arm's ONLY `Main.rand` draws are
//! * the Dolphin idle timer `Next(300, 1200)` (`:23576`) — drawn EVERY tick
//!   while `ai[2] == 0`, before the `++ai[3]` compare;
//! * the Dolphin state pick `Next(1, 3)` (`:23579`) — only when the timer
//!   fires;
//! * the Dolphin surface-splash sound gate `Next(2)` (`:23678`) — only at
//!   `ai[3] == 60` inside the water-line arm;
//! * the dry flop triple `Next(-50, -20)`, `Next(-20, 20)`, `Next(2)`
//!   (`:23977-23979`) — only when `velocity.Y == 0` and the type is neither
//!   65 nor 692, in that order.
//! All ride the caller's `UnifiedRandom` (the `Main.rand` analogue,
//! `tr_core::unified_random`). The wet Goldfish/Piranha/Shark paths draw
//! NOTHING — the `the_wet_goldfish_idle_draws_no_rng` witness pins the
//! stream state byte-for-byte (`UnifiedRandom: PartialEq`).

// WIRED by the S1-D-3 bat/fish reconcile (`update_swimmer`'s migration).
// `Ai16Effects::light` stays write-only until the AnglerFish (102) lands
// its kind + the entity-lights feed — the allow covers that seam.
#![allow(dead_code)]

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai16Ctx<'a> {
    /// The `Main.player` slots as `TargetClosest` sees them (the port has
    /// one session player; `npc_target.rs`'s seam).
    pub players: &'a [TargetCandidate],
    /// `Main.player[target].wet` (`:23704`) — the candidate struct carries
    /// `dead` but not `wet`; the caller folds the session player's flag.
    pub player_wet: bool,
    /// `NPC.justHit` (`:23502`/`:23518`) — the Pufferfish's puff latch. The
    /// port's `Npc` has no `justHit` field yet (the strike path's `hurt_cd`
    /// stand-in is a cooldown, not the one-tick flag) — the caller supplies
    /// the flag at the wiring step.
    pub just_hit: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (the S1-D-3p
    /// precedent).
    pub ticker: u64,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct Ai16Effects {
    /// The AnglerFish's `Lighting.AddLight` (`:23762-23765`):
    /// `(tileX, tileY, [r, g, b])`, the C#'s int-truncated tile coordinates.
    pub light: Option<(i32, i32, [f32; 3])>,
}

/// The port's aiStyle-16 kinds → their vanilla `NPC.type`
/// (`npc/data.rs`'s cs_id rows; `NPCID.cs:10787`/`:10793`/`:10807` +
/// W4-c4's nine: `:10791`/`:10991`/`:11159`/`:11603`/`:11861`/`:11891`/
/// `:11907`/`:12053`/`:12061`). The AnglerFish (102) is the one unported
/// member left — its kind is the W4-d35 roster lane.
fn vanilla_type(kind: NpcKind) -> i32 {
    match kind {
        NpcKind::Goldfish => 55,
        NpcKind::Piranha => 58,
        NpcKind::Shark => 65,
        NpcKind::CorruptGoldfish => 57,
        NpcKind::Arapaima => 157,
        NpcKind::BloodFeeder => 241,
        NpcKind::CrimsonGoldfish => 465,
        NpcKind::GoldGoldfish => 592,
        NpcKind::Pupfish => 607,
        NpcKind::Dolphin => 615,
        NpcKind::Pufferfish => 688,
        NpcKind::Orca => 692,
        _ => unreachable!("only aiStyle-16 kinds reach the family arm"),
    }
}

/// The family entry — `NPC.AI`'s `if (this.aiStyle == 16)`
/// (`NPC.cs:23492-24021`). `npc.wet` must already carry this tick's
/// `Entity.wet` (the caller's `is_wet` pass, the `update_swimmer` shape).
pub fn ai_016_step(
    npc: &mut Npc,
    world: &World,
    ctx: &Ai16Ctx,
    rng: &mut UnifiedRandom,
) -> Ai16Effects {
    ai_016_arm(vanilla_type(npc.kind), npc, world, ctx, rng)
}

/// The target slot's candidate. The C# indexes `Main.player[this.target]`
/// (255 slots always exist); the port's slice holds the session player, so
/// an unset/stale slot falls back to slot 0 — the same fallback
/// `SetTargetTrackingValues` applies (`NPC.cs:77731-77734`).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Tile.topSlope()` (`Tile.cs:290-298`): slope 1 or 2.
fn top_slope(t: &tr_core::tile::Tile) -> bool {
    matches!(t.slope(), 1 | 2)
}

/// `Tile.leftSlope()` (`Tile.cs:310-318`): slope 2 or 4.
fn left_slope(t: &tr_core::tile::Tile) -> bool {
    matches!(t.slope(), 2 | 4)
}

/// `MathHelper.WrapAngle` (XNA's `IEEERemainder(angle, TwoPi)` + the ±2π
/// fix-up) — the same transcription as `projectile.rs::wrap_angle` (kept
/// local so that file stays byte-identical this round; the shared-helper
/// extraction is the wiring step's).
fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    if angle > -PI && angle <= PI {
        return angle;
    }
    let rem = angle - (angle / TAU).round_ties_even() * TAU;
    if rem <= -PI {
        rem + TAU
    } else if rem > PI {
        rem - TAU
    } else {
        rem
    }
}

/// The whole aiStyle-16 arm, keyed on the vanilla `NPC.type` so the ten
/// unported members' branches stay reachable (and testable) verbatim.
pub fn ai_016_arm(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai16Ctx,
    rng: &mut UnifiedRandom,
) -> Ai16Effects {
    let mut fx = Ai16Effects::default();
    let (w, h) = npc.size();
    let (wi, hi) = (w as i32, h as i32);

    // `:23494-23497` — `if (base.direction == 0) this.TargetClosest();`.
    if npc.facing == 0 {
        target_closest(npc, world, ctx.players, true, 16, ctx.ticker);
    }

    // ---- The Pufferfish (688), `:23498-23571` ------------------------------
    if ty == 688 {
        // `:23500-23525` — the `Main.netMode != 1` gate folds to true (the
        // port is single-player).
        if ctx.just_hit && npc.ai[2] == 0.0 {
            // `:23502-23507` — the puff latch.
            npc.ai[2] = 1.0;
            npc.local_ai[0] = 180.0;
            npc.net_update = true;
        } else {
            // `:23508-23524` — the countdown; at 0 the puff deflates (or
            // re-puffs on a fresh hit).
            npc.local_ai[0] -= 1.0;
            if npc.local_ai[0] <= 0.0 {
                npc.local_ai[0] = 120.0;
                if npc.ai[2] == 1.0 {
                    npc.ai[2] = 0.0;
                }
                if ctx.just_hit {
                    npc.ai[2] = 1.0;
                }
                npc.net_update = true;
            }
        }
        if npc.ai[2] == 1.0 {
            // `:23526-23569` — the puffed float: the 0.98 damp on BOTH axes,
            // then the water-line hover or the ±0.3 buoyancy, then RETURN
            // (the rotation tail never runs on a puffed tick).
            npc.vx *= 0.98;
            npc.vy *= 0.98;
            // `base.Top.ToTileCoordinates()` (`Entity.cs:86-91`,
            // `Utils.cs:1807-1810`): `(int)(x + width/2f) >> 4`, `(int)y >> 4`.
            let top_tx = (npc.x + w / 2.0) as i32 >> 4;
            let top_ty = npc.y as i32 >> 4;
            if let Some(water_line) = crate::collision::get_water_line(world, top_tx, top_ty) {
                // `:23531-23551` — `num252 = base.Top.Y - 5f`.
                let num252 = npc.y - 5.0;
                if npc.y + h / 2.0 > water_line {
                    npc.vy -= 0.4;
                    if npc.vy < -2.0 {
                        npc.vy = -2.0;
                    }
                    if num252 + npc.vy < water_line {
                        npc.vy = water_line - num252;
                    }
                } else {
                    npc.vy = npc.vy.min(water_line - num252);
                    if (num252 - water_line).abs() < 2.0 {
                        npc.vy = 0.0;
                    }
                }
            } else if npc.wet {
                // `:23553-23560` — submerged: rise, cap -10.
                npc.vy -= 0.3;
                if npc.vy < -10.0 {
                    npc.vy = -10.0;
                }
            } else {
                // `:23561-23568` — dry: sink, cap 10.
                npc.vy += 0.3;
                if npc.vy > 10.0 {
                    npc.vy = 10.0;
                }
            }
            return fx;
        }
    }

    // ---- The Dolphin (615), `:23572-23697` ---------------------------------
    if ty == 615 {
        if npc.ai[2] == 0.0 {
            // `:23574-23591` — the idle timer. The `Next(300, 1200)`
            // threshold is drawn EVERY idle tick, BEFORE the `++ai[3]`
            // compare (`:23576-23577`) — the stream order is load-bearing.
            let num253 = rng.next_range(300, 1200);
            npc.ai[3] += 1.0;
            if npc.ai[3] >= num253 as f32 {
                npc.ai[2] = rng.next_range(1, 3) as f32;
                // `:23580-23583` — the leap needs 128 px of clear air above
                // (`Collision.CanHitLine` on the own box vs the box 128 up).
                if npc.ai[2] == 1.0
                    && !crate::projectile::can_hit_line(
                        world, npc.x, npc.y, w, h, npc.x, npc.y - 128.0, w, h,
                    )
                {
                    npc.ai[2] = 2.0;
                }
                if npc.ai[2] == 2.0 {
                    target_closest(npc, world, ctx.players, true, 16, ctx.ticker);
                }
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        if npc.ai[2] == 1.0 {
            // `:23592-23634` — the leap.
            if npc.collide_y || npc.collide_x {
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            } else if npc.wet {
                npc.vy -= 0.4;
                if npc.vy < -6.0 {
                    npc.vy = -6.0;
                }
                // `:23607-23615` — the velocity tilt, clamped ±2π/5.
                npc.rotation = npc.vy * npc.facing as f32 * 0.3;
                let lim = std::f32::consts::PI * -2.0 / 5.0;
                if npc.rotation < lim {
                    npc.rotation = lim;
                }
                if npc.rotation > -lim {
                    npc.rotation = -lim;
                }
                if npc.ai[3] == 1.0 {
                    // `:23616-23621` — the airborne flag ends the leap.
                    npc.ai[2] = 0.0;
                    npc.ai[3] = 0.0;
                    npc.net_update = true;
                }
            } else {
                // `:23623-23632` — airborne: the unclamped spin, the fall.
                npc.rotation += npc.facing as f32 * 0.2;
                npc.ai[3] = 1.0;
                npc.vy += 0.3;
                if npc.vy > 10.0 {
                    npc.vy = 10.0;
                }
            }
            return fx;
        }
        if npc.ai[2] == 2.0 {
            // `:23635-23696` — the surface float.
            if npc.collide_y || npc.collide_x {
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            } else if npc.wet {
                npc.vy -= 0.4;
                if npc.vy < -6.0 {
                    npc.vy = -6.0;
                }
                npc.rotation = npc.vy * npc.facing as f32 * 0.3;
                let lim = std::f32::consts::PI * -2.0 / 5.0;
                if npc.rotation < lim {
                    npc.rotation = lim;
                }
                if npc.rotation > -lim {
                    npc.rotation = -lim;
                }
                let top_tx = (npc.x + w / 2.0) as i32 >> 4;
                let top_ty = npc.y as i32 >> 4;
                if let Some(water_line) = crate::collision::get_water_line(world, top_tx, top_ty) {
                    // `:23659-23681` — ride the line: `y = waterLineHeight
                    // + 0f - position.Y`, the [-2, 0.5] clamp, the fixed
                    // -π/5·direction tilt, the 0.95 X damp.
                    let y = water_line - npc.y;
                    npc.vy = y.clamp(-2.0, 0.5);
                    npc.rotation = -(std::f32::consts::PI) / 5.0 * npc.facing as f32;
                    npc.vx *= 0.95;
                    if npc.ai[3] == 0.0 {
                        npc.net_update = true;
                    }
                    npc.ai[3] += 1.0;
                    if npc.ai[3] >= 300.0 {
                        // `:23671-23677` — the 300-tick dive.
                        npc.ai[2] = 0.0;
                        npc.ai[3] = 0.0;
                        npc.net_update = true;
                        npc.vy = 4.0;
                    }
                    // `:23678-23681` — the splash sound gate. The DRAW is
                    // kept (stream order); `SoundEngine.PlaySound(45, ...)`
                    // is the documented skip (the sound seam is unwired).
                    if npc.ai[3] == 60.0 && rng.next_max(2) == 0 {
                        // SoundEngine.PlaySound(45, (int)x, (int)y) — skip.
                    }
                }
            } else {
                // `:23684-23694` — beached mid-float: reset and fall.
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
                npc.vy += 0.3;
                if npc.vy > 10.0 {
                    npc.vy = 10.0;
                }
            }
            return fx;
        }
    }

    // ---- The shared wet/dry machine, `:23698-23989` -------------------------
    if npc.wet {
        // `:23700-23708` — `flag25`: the critter exclusion (55/592/607/615/
        // 688 never chase), then `TargetClosest(faceTarget: false)` and the
        // player-wet + alive + `Collision.CanHit` box gate.
        let mut flag25 = false;
        if ty != 55 && ty != 592 && ty != 607 && ty != 615 && ty != 688 {
            target_closest(npc, world, ctx.players, false, 16, ctx.ticker);
            let p = target_of(npc, ctx.players);
            if ctx.player_wet
                && !p.dead
                && crate::projectile::can_hit(
                    world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
                )
            {
                flag25 = true;
            }
        }
        // `:23709-23736` — the slope flips at the feet. `num254 =
        // (int)Center.X / 16` and `num255 = (int)(position.Y + height) / 16`
        // are INTEGER divisions of the truncated pixel coordinates.
        let num254 = (npc.x + w / 2.0) as i32 / 16;
        let num255 = (npc.y + h) as i32 / 16;
        let foot = world
            .tile(num254, num255)
            .filter(|t| top_slope(t))
            .or_else(|| world.tile(num254, num255 + 1).filter(|t| top_slope(t)));
        if let Some(t) = foot {
            if left_slope(t) {
                npc.facing = -1;
                npc.vx = npc.vx.abs() * -1.0;
            } else {
                npc.facing = 1;
                npc.vx = npc.vx.abs();
            }
        }
        // `:23737-23761` — the idle wall/floor bounces (the chase never
        // bounces here; the collision pass owns the chase's contact).
        if !flag25 {
            if npc.collide_x {
                npc.vx *= -1.0;
                npc.facing *= -1;
                npc.net_update = true;
            }
            if npc.collide_y {
                npc.net_update = true;
                if npc.vy > 0.0 {
                    npc.vy = npc.vy.abs() * -1.0;
                    npc.direction_y = -1;
                    npc.ai[0] = -1.0;
                } else if npc.vy < 0.0 {
                    npc.vy = npc.vy.abs();
                    npc.direction_y = 1;
                    npc.ai[0] = 1.0;
                }
            }
        }
        // `:23762-23765` — the AnglerFish's lure light. The coordinates are
        // the C#'s: `(int)(x + (float)(width / 2) + (float)(direction *
        // (width + 8))) / 16` (INTEGER width/2) and `(int)(y + 2f) / 16`.
        if ty == 102 {
            let lx = (npc.x + (wi / 2) as f32 + (npc.facing * (wi + 8)) as f32) as i32 / 16;
            let ly = (npc.y + 2.0) as i32 / 16;
            fx.light = Some((lx, ly, [0.07, 0.04, 0.025]));
        }
        if flag25 {
            // `:23766-23845` — the chase.
            if npc.ai[0] != 0.0 {
                npc.ai[0] = 0.0;
                npc.net_update = true;
            }
            target_closest(npc, world, ctx.players, true, 16, ctx.ticker);
            if ty == 157 {
                // `:23774-23802` — the Arapaima: the counter-direction 0.95
                // damp, the 0.25/0.2 accel and the ASYMMETRIC caps (past 8
                // snaps to 7, past 5 snaps to 4).
                if npc.vx > 0.0 && npc.facing < 0 {
                    npc.vx *= 0.95;
                }
                if npc.vx < 0.0 && npc.facing > 0 {
                    npc.vx *= 0.95;
                }
                npc.vx += npc.facing as f32 * 0.25;
                npc.vy += npc.direction_y as f32 * 0.2;
                if npc.vx > 8.0 {
                    npc.vx = 7.0;
                }
                if npc.vx < -8.0 {
                    npc.vx = -7.0;
                }
                if npc.vy > 5.0 {
                    npc.vy = 4.0;
                }
                if npc.vy < -5.0 {
                    npc.vy = -4.0;
                }
            } else if ty == 65 || ty == 102 || ty == 692 {
                // `:23803-23823` — the Shark/AnglerFish/Orca group:
                // 0.15/0.15 accel, the ±5/±3 caps.
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
            } else {
                // `:23824-23844` — the rest: 0.1/0.1 accel, ±3/±2 caps.
                npc.vx += npc.facing as f32 * 0.1;
                npc.vy += npc.direction_y as f32 * 0.1;
                if npc.vx > 3.0 {
                    npc.vx = 3.0;
                }
                if npc.vx < -3.0 {
                    npc.vx = -3.0;
                }
                if npc.vy > 2.0 {
                    npc.vy = 2.0;
                }
                if npc.vy < -2.0 {
                    npc.vy = -2.0;
                }
            }
        } else {
            // `:23846-23961` — the idle bob.
            if npc.ai[0] == 0.0 {
                npc.ai[0] = 1.0;
                npc.net_update = true;
            }
            if ty == 157 {
                // `:23853-23902` — the Arapaima's own bob: the directionY
                // re-read from the player's POSITION.Y (`:23855-23862`),
                // the 0.2 X accel with the ±2 damp, the 0.02 Y drift with
                // the directionY-dependent thresholds.
                let p = target_of(npc, ctx.players);
                if p.y > npc.y {
                    npc.direction_y = 1;
                } else {
                    npc.direction_y = -1;
                }
                npc.vx += npc.facing as f32 * 0.2;
                if npc.vx < -2.0 || npc.vx > 2.0 {
                    npc.vx *= 0.95;
                }
                if npc.ai[0] == -1.0 {
                    let mut num256 = -0.6;
                    if npc.direction_y < 0 {
                        num256 = -1.0;
                    }
                    if npc.direction_y > 0 {
                        num256 = -0.2;
                    }
                    npc.vy -= 0.02;
                    if npc.vy < num256 {
                        npc.ai[0] = 1.0;
                    }
                } else {
                    let mut num257 = 0.6;
                    if npc.direction_y < 0 {
                        num257 = 0.2;
                    }
                    if npc.direction_y > 0 {
                        num257 = 1.0;
                    }
                    npc.vy += 0.02;
                    if npc.vy > num257 {
                        npc.ai[0] = -1.0;
                    }
                }
            } else {
                // `:23903-23931` — the generic bob: the 0.1 X accel, the
                // `num258` cap (1, the Dolphin 3), the ±0.3 Y bob through
                // `ai[0]` at 0.01/tick.
                npc.vx += npc.facing as f32 * 0.1;
                let num258 = if ty == 615 { 3.0 } else { 1.0 };
                if npc.vx < 0.0 - num258 || npc.vx > num258 {
                    npc.vx *= 0.95;
                }
                if npc.ai[0] == -1.0 {
                    npc.vy -= 0.01;
                    if npc.vy < -0.3 {
                        npc.ai[0] = 1.0;
                    }
                } else {
                    npc.vy += 0.01;
                    if npc.vy > 0.3 {
                        npc.ai[0] = -1.0;
                    }
                }
            }
            // `:23932-23956` — the beach-avoid. `num259`/`num260` use the
            // INTEGER half-extents (`(float)(base.width / 2)`); the C#'s
            // null-tile guards (`new Tile()`) read as the default inactive,
            // dry tile here (`World::tile`'s `None`).
            let num259 = (npc.x + (wi / 2) as f32) as i32 / 16;
            let num260 = (npc.y + (hi / 2) as f32) as i32 / 16;
            let liquid_above = world
                .tile(num259, num260 - 1)
                .is_some_and(|t| t.liquid > 128);
            if liquid_above {
                if world.tile(num259, num260 + 1).is_some_and(|t| t.active()) {
                    npc.ai[0] = -1.0;
                } else if world.tile(num259, num260 + 2).is_some_and(|t| t.active()) {
                    npc.ai[0] = -1.0;
                }
            }
            // `:23957-23960` — the Y damp past ±0.4 (the Arapaima exempt).
            if ty != 157 && (npc.vy > 0.4 || npc.vy < -0.4) {
                npc.vy *= 0.95;
            }
        }
    } else {
        // `:23963-23989` — the beached flop.
        if npc.vy == 0.0 {
            if ty == 65 || ty == 692 {
                // `:23967-23974` — the Shark/Orca stand decay.
                npc.vx *= 0.94;
                if npc.vx > -0.2 && npc.vx < 0.2 {
                    npc.vx = 0.0;
                }
            } else {
                // `:23975-23981` — the random flop: `Next(-50, -20) * 0.1`
                // on Y, `Next(-20, 20) * 0.1` on X, the `Next(2)` direction
                // coin — in THAT order. `NetUpdateIgnoreSpamLimit`
                // (`NPC.cs:90568-90576`) folds to the flag (the `netSpam`
                // counter is unported). `Main.netMode != 1` folds to true.
                npc.vy = rng.next_range(-50, -20) as f32 * 0.1;
                npc.vx = rng.next_range(-20, 20) as f32 * 0.1;
                npc.facing = if rng.next_max(2) == 0 { 1 } else { -1 };
                npc.net_update = true;
            }
        }
        // `:23983-23988` — the gravity and the sink phase.
        npc.vy += 0.3;
        if npc.vy > 10.0 {
            npc.vy = 10.0;
        }
        npc.ai[0] = 1.0;
    }

    // ---- The rotation tail, `:23990-24019` ---------------------------------
    if ty == 692 && npc.ai[0] == 0.0 {
        // `:23990-24007` — the Orca aims its body at the target:
        // `(Center - player.Center).SafeNormalize(Zero).ToRotation()`
        // (`Utils.cs:1837-1844`, `:1655-1658`), the spriteDirection -1 π
        // fold, `MathHelper.WrapAngle`, the ±0.3 clamp.
        let p = target_of(npc, ctx.players);
        let dx = (npc.x + w / 2.0) - (p.x + p.width as f32 / 2.0);
        let dy = (npc.y + h / 2.0) - (p.y + p.height as f32 / 2.0);
        let len = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = if len == 0.0 || len.is_nan() {
            (0.0, 0.0) // SafeNormalize(Vector2.Zero)
        } else {
            (dx / len, dy / len)
        };
        npc.rotation = ny.atan2(nx);
        if npc.sprite_dir == -1 {
            npc.rotation += std::f32::consts::PI;
        }
        npc.rotation = wrap_angle(npc.rotation);
        if npc.rotation < -0.3 {
            npc.rotation = -0.3;
        }
        if npc.rotation > 0.3 {
            npc.rotation = 0.3;
        }
    } else {
        // `:24008-24019` — the generic tilt: `vy * direction * 0.1`, ±0.2.
        npc.rotation = npc.vy * npc.facing as f32 * 0.1;
        if npc.rotation < -0.2 {
            npc.rotation = -0.2;
        }
        if npc.rotation > 0.2 {
            npc.rotation = 0.2;
        }
    }
    fx
}

/// `NPC.FindFrame`'s type cases for the family. `frame` is the ROW index
/// (the port's `Npc::frame` convention — the C# stores `frame.Y = row *
/// frameHeight`), `frame_counter` the C# `frameCounter`, `frame_count`
/// `Main.npcFrameCount[type]` (`Main.cs:67082`: 57/157/241/465/592/607/688
/// = 6, 615 = 5, 692 = 16). The five cases:
/// * 55/57/58/102/241/465/592/607 (`NPC.cs:74181-74226`) — the wet flap
///   rows 0..3 at the 6-tick bands, the dry flop rows 4/5;
/// * 157 (`:74134-74180`) — the Arapaima's 8-band 5-tick cycle (the dry
///   tick double-counts);
/// * 615 (`:74227-74243`) — the Dolphin's 7-tick strip over rows
///   0..count-2, the dry leap pin row 4;
/// * 688 (`:74244-74290`) — the Pufferfish's puff rows 4/5 (6-tick), dry
///   flop rows 4/5 (12-tick), wet flap rows 0..3 (8-tick);
/// * 692 (`:74693-74762`) — the Orca's two-half strip (the ai[0]==0 half
///   offset), the wet 5/7-tick flap and the dry velocity-keyed row glide.
///   The `!IsABestiaryIconDummy` sprite flip folds to always-flip (the
///   port has no bestiary dummy — the wave-B documented skip).
pub fn style16_find_frame(
    npc_type: i32,
    wet: bool,
    vy: f32,
    facing: i32,
    ai0: f32,
    ai2: f32,
    frame_count: u32,
    frame: &mut u32,
    frame_counter: &mut f32,
    sprite_dir: &mut i32,
) {
    match npc_type {
        // `case 55/57/58/102/241/465/592/607` (`:74181-74226`).
        55 | 57 | 58 | 102 | 241 | 465 | 592 | 607 => {
            *sprite_dir = facing;
            *frame_counter += 1.0;
            if wet {
                if *frame_counter < 6.0 {
                    *frame = 0;
                } else if *frame_counter < 12.0 {
                    *frame = 1;
                } else if *frame_counter < 18.0 {
                    *frame = 2;
                } else if *frame_counter < 24.0 {
                    *frame = 3;
                } else {
                    *frame_counter = 0.0;
                }
            } else if *frame_counter < 6.0 {
                *frame = 4;
            } else if *frame_counter < 12.0 {
                *frame = 5;
            } else {
                *frame_counter = 0.0;
            }
        }
        // `case 157` (`:74134-74180`) — the Arapaima.
        157 => {
            *sprite_dir = facing;
            *frame_counter += 1.0;
            if !wet {
                *frame_counter += 1.0;
            }
            let num50 = 5.0;
            let c = *frame_counter;
            if c < num50 {
                *frame = 0;
            } else if c < num50 * 2.0 {
                *frame = 1;
            } else if c < num50 * 3.0 {
                *frame = 2;
            } else if c < num50 * 4.0 {
                *frame = 1;
            } else if c < num50 * 5.0 {
                *frame = 3;
            } else if c < num50 * 6.0 {
                *frame = 4;
            } else if c < num50 * 7.0 {
                *frame = 5;
            } else if c < num50 * 8.0 {
                *frame = 4;
            } else {
                *frame_counter = 0.0;
            }
        }
        // `case 615` (`:74227-74243`) — the Dolphin.
        615 => {
            *sprite_dir = facing;
            if !wet && ai2 == 1.0 {
                *frame = 4;
                return;
            }
            if !wet {
                *frame_counter += 1.0;
            }
            *frame_counter += 1.0;
            if *frame_counter >= (7 * (frame_count - 1)) as f32 {
                *frame_counter = 0.0;
            }
            *frame = (*frame_counter / 7.0) as u32;
        }
        // `case 688` (`:74244-74290`) — the Pufferfish.
        688 => {
            *sprite_dir = facing;
            if ai2 == 1.0 {
                if *frame != 5 {
                    *frame_counter += 1.0;
                    if *frame_counter >= 6.0 {
                        *frame_counter = 0.0;
                        *frame += 1;
                    }
                }
                if *frame < 4 || *frame > 5 {
                    *frame = 4;
                }
            } else if !wet {
                *frame_counter += 1.0;
                if *frame_counter >= 12.0 {
                    *frame_counter = 0.0;
                    *frame += 1;
                }
                if *frame < 4 || *frame > 5 {
                    *frame = 4;
                }
            } else {
                *frame_counter += 1.0;
                if *frame_counter >= 8.0 {
                    *frame_counter = 0.0;
                    *frame += 1;
                }
                if *frame > 3 {
                    *frame = 0;
                }
            }
        }
        // `case 692` (`:74693-74762`) — the Orca.
        692 => {
            *sprite_dir = -facing;
            let num34 = frame_count / 2;
            let num35 = *frame;
            let mut num36 = num35 % num34;
            let flag3 = ai0 == 0.0;
            let num37 = if flag3 { 5.0 } else { 7.0 };
            if wet {
                *frame_counter += 1.0;
                if *frame_counter >= num37 {
                    num36 += 1;
                    if num36 >= num34 {
                        num36 = 0;
                    }
                    *frame_counter = 0.0;
                }
            } else {
                let num38 = if vy > 0.0 {
                    7
                } else if vy < 0.0 {
                    3
                } else {
                    1
                };
                if num36 != num38 {
                    *frame_counter += 1.0;
                    if *frame_counter >= 4.0 {
                        if num36 < num38 {
                            num36 += 1;
                        } else if num36 > num38 {
                            num36 -= 1;
                        }
                        if num36 == num34 {
                            num36 = num38;
                        }
                        *frame_counter = 0.0;
                    }
                }
            }
            if flag3 {
                num36 += num34;
            }
            if num36 != num35 {
                *frame = num36;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one C#
    //! branch against its citation; the RNG witnesses mirror the cited
    //! `Next` calls on a cloned `UnifiedRandom` and compare the stream
    //! state (`PartialEq`) so a drifted draw order fails by construction.
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        World::new("t".into(), 200, 120)
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

    fn fish(kind: NpcKind, x: f32, y: f32) -> Npc {
        Npc::new(kind, x, y, 1, 0.0)
    }

    fn ctx<'a>(players: &'a [TargetCandidate], player_wet: bool) -> Ai16Ctx<'a> {
        Ai16Ctx {
            players,
            player_wet,
            just_hit: false,
            ticker: 0,
        }
    }

    /// `NPC.cs:23494-23497` + `:23846-23931` — the head `TargetClosest`
    /// fires on direction 0; the Goldfish (55, in the `:23701` exclusion)
    /// idles: `ai[0]` 0 → 1, the 0.1 X accel — and the whole wet path draws
    /// NO `Main.rand` (the stream state is byte-identical afterwards).
    #[test]
    fn the_wet_goldfish_idle_draws_no_rng() {
        let w = air();
        let players = [cand(900.0, 1000.0)]; // to the LEFT of the fish
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.facing = 0;
        n.wet = true;
        let mut rng = UnifiedRandom::new(12345);
        let untouched = rng.clone();
        let fx = ai_016_step(&mut n, &w, &ctx(&players, true), &mut rng);
        assert_eq!(rng, untouched, "the wet goldfish path consumes no rand");
        assert_eq!(n.target, 0, "the head TargetClosest picked the slot");
        assert_eq!(n.facing, -1, "the player is to the left");
        assert_eq!(n.ai[0], 1.0, "the idle bob arms (:23848-23852)");
        assert!((n.vx - -0.1).abs() < 1e-6, "the 0.1 idle accel (:23905)");
        assert_eq!(fx.light, None, "only the AnglerFish lights");
    }

    /// `NPC.cs:23824-23844` — the Piranha's chase: `flag25` (the player
    /// wet + alive + CanHit), `ai[0] = 0`, the 0.1/0.1 accel to the ±3/±2
    /// caps; the generic tilt `vy * direction * 0.1` clamped ±0.2.
    #[test]
    fn the_piranha_chases_to_the_default_caps() {
        let w = air();
        let players = [cand(1300.0, 1200.0)]; // right and below
        let mut n = fish(NpcKind::Piranha, 1000.0, 1000.0);
        n.wet = true;
        n.vx = 2.95;
        n.vy = 1.95;
        let mut rng = UnifiedRandom::new(1);
        let untouched = rng.clone();
        ai_016_step(&mut n, &w, &ctx(&players, true), &mut rng);
        assert_eq!(rng, untouched, "the chase draws no rand");
        assert_eq!(n.ai[0], 0.0, "the chase zeroes the bob (:23768-23772)");
        assert_eq!(n.facing, 1);
        assert_eq!(n.direction_y, 1);
        assert_eq!(n.vx, 3.0, "2.95 + 0.1 caps at 3 (:23828-23831)");
        assert_eq!(n.vy, 2.0, "1.95 + 0.1 caps at 2 (:23836-23839)");
        assert_eq!(n.rotation, 0.2, "2 * 1 * 0.1 clamps at 0.2 (:24010)");
    }

    /// `NPC.cs:23803-23823` — the Shark group's 0.15 accel and ±5/±3 caps.
    #[test]
    fn the_shark_chases_to_the_group_caps() {
        let w = air();
        let players = [cand(1300.0, 1200.0)];
        let mut n = fish(NpcKind::Shark, 1000.0, 1000.0);
        n.wet = true;
        n.vx = 4.95;
        n.vy = 2.95;
        let mut rng = UnifiedRandom::new(2);
        ai_016_step(&mut n, &w, &ctx(&players, true), &mut rng);
        assert_eq!(n.vx, 5.0, "4.95 + 0.15 caps at 5 (:23807-23810)");
        assert_eq!(n.vy, 3.0, "2.95 + 0.15 caps at 3 (:23815-23818)");
    }

    /// `NPC.cs:23967-23974` + `:23983-23988` — the beached Shark decays
    /// `vx *= 0.94` and snaps to 0 inside ±0.2 (NO flop draw — 65 is
    /// excluded), then the 0.3 gravity and `ai[0] = 1`.
    #[test]
    fn the_beached_shark_decays_without_drawing() {
        let w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Shark, 1000.0, 1000.0);
        n.wet = false;
        n.vy = 0.0;
        n.vx = 0.1;
        let mut rng = UnifiedRandom::new(3);
        let untouched = rng.clone();
        ai_016_step(&mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(rng, untouched, "the Shark never rolls the flop");
        assert_eq!(n.vx, 0.0, "0.1 * 0.94 = 0.094 snaps to 0 (:23970-23973)");
        assert_eq!(n.vy, 0.3, "the gravity (:23983)");
        assert_eq!(n.ai[0], 1.0, "the sink phase (:23988)");
    }

    /// `NPC.cs:23975-23981` — the dry flop's RNG WITNESS: the triple
    /// `Next(-50, -20)`, `Next(-20, 20)`, `Next(2)` in the C# order, the
    /// `* 0.1` scales, the coin facing; the gravity lands AFTER the draw
    /// (`:23983`). The mirrored stream must end byte-identical.
    #[test]
    fn the_dry_flop_mirrors_the_cited_rand_triple() {
        let w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.wet = false;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(777);
        let mut mirror = rng.clone();
        let want_vy = mirror.next_range(-50, -20) as f32 * 0.1;
        let want_vx = mirror.next_range(-20, 20) as f32 * 0.1;
        let want_dir = if mirror.next_max(2) == 0 { 1 } else { -1 };
        ai_016_step(&mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(rng, mirror, "exactly the cited triple, in order");
        assert_eq!(n.vx, want_vx, "Next(-20, 20) * 0.1 (:23978)");
        assert_eq!(n.vy, want_vy + 0.3, "Next(-50, -20) * 0.1 + gravity");
        assert_eq!(n.facing, want_dir, "the Next(2) coin (:23979)");
        assert!(n.net_update, "NetUpdateIgnoreSpamLimit (:23980)");
    }

    /// `NPC.cs:23502-23507` + `:23526-23569` — the Pufferfish puff: the
    /// justHit latch (ai[2] = 1, localAI[0] = 180, netUpdate), the 0.98
    /// damp on BOTH axes, the dry +0.3 sink, and the EARLY RETURN — the
    /// rotation tail never runs on a puffed tick.
    #[test]
    fn the_pufferfish_puffs_on_hit_and_skips_the_rotation_tail() {
        let w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0); // stand-in body
        n.wet = false;
        n.vx = 2.0;
        n.vy = 0.0;
        n.rotation = 0.123;
        let mut c = ctx(&players, false);
        c.just_hit = true;
        let mut rng = UnifiedRandom::new(4);
        let untouched = rng.clone();
        ai_016_arm(688, &mut n, &w, &c, &mut rng);
        assert_eq!(rng, untouched, "the puff draws no rand");
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.local_ai[0], 180.0);
        assert!(n.net_update);
        assert!((n.vx - 1.96).abs() < 1e-6, "2 * 0.98 (:23528)");
        assert!((n.vy - 0.3).abs() < 1e-6, "the dry sink (:23563)");
        assert_eq!(n.rotation, 0.123, "the early return (:23569)");
    }

    /// `NPC.cs:23508-23524` — the countdown: localAI[0] 1 → 0 deflates the
    /// puff (ai[2] 1 → 0) and re-arms at 120.
    #[test]
    fn the_pufferfish_deflates_when_the_countdown_ends() {
        let w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.wet = true; // keep the fall-through on the draw-free wet path
        n.ai[2] = 1.0;
        n.local_ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(5);
        ai_016_arm(688, &mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(n.ai[2], 0.0, "the deflate (:23514-23517)");
        assert_eq!(n.local_ai[0], 120.0, "the re-arm (:23513)");
        assert!(n.net_update);
    }

    /// `NPC.cs:23574-23591` — the Dolphin idle timer's RNG WITNESS: the
    /// per-tick `Next(300, 1200)` threshold, then (on the fire) the
    /// `Next(1, 3)` state pick; ai[3] resets, netUpdate arms. With
    /// ai[3] = 1199 the incremented 1200 beats every possible threshold,
    /// so exactly TWO draws land. In clear air the CanHitLine gate keeps
    /// a drawn leap at 1.
    #[test]
    fn the_dolphin_timer_mirrors_its_two_cited_draws() {
        let w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.wet = true;
        n.ai[2] = 0.0;
        n.ai[3] = 1199.0;
        let mut rng = UnifiedRandom::new(999);
        let mut mirror = rng.clone();
        let threshold = mirror.next_range(300, 1200);
        let pick = mirror.next_range(1, 3);
        assert!(1200 >= threshold, "the fire is guaranteed");
        ai_016_arm(615, &mut n, &w, &ctx(&players, false), &mut rng);
        if pick == 1 {
            // The leap state ran this tick (the wet rise, -0.4).
            assert_eq!(rng, mirror, "exactly the two cited draws");
        } else {
            // The float state ran; its water-line arm found no water in
            // the air world, so no third draw either.
            assert_eq!(rng, mirror, "exactly the two cited draws");
        }
        assert_eq!(n.ai[2], pick as f32, "Next(1, 3) picks the state");
        // Both states reset ai[3] at the fire; the leap's wet arm keeps 0
        // (the airborne flag never set) and the float's water-line arm
        // found no water in the air world.
        assert_eq!(n.ai[3], 0.0);
        assert!(n.net_update);
    }

    /// `NPC.cs:23774-23802` — the Arapaima's chase: the 0.25/0.2 accel and
    /// the ASYMMETRIC caps — past 8 snaps to 7, past 5 snaps to 4.
    #[test]
    fn the_arapaima_chase_caps_snap_down_asymmetrically() {
        let w = air();
        let players = [cand(1300.0, 1200.0)];
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.wet = true;
        n.vx = 9.0;
        n.vy = 6.0;
        let mut rng = UnifiedRandom::new(6);
        ai_016_arm(157, &mut n, &w, &ctx(&players, true), &mut rng);
        assert_eq!(n.ai[0], 0.0, "the chase zeroes the bob");
        assert_eq!(n.vx, 7.0, "9 + 0.25 > 8 snaps to 7 (:23786-23789)");
        assert_eq!(n.vy, 4.0, "6 + 0.2 > 5 snaps to 4 (:23794-23797)");
    }

    /// `NPC.cs:23868-23901` + `:23957-23960` — the Arapaima's idle bob:
    /// the directionY-dependent threshold (-1 when rising away from a
    /// player above) flips ai[0] at -1.0, and the ±0.4 Y damp EXEMPTS 157.
    #[test]
    fn the_arapaima_idle_bob_uses_the_direction_y_thresholds() {
        let w = air();
        let players = [cand(1300.0, 900.0)]; // ABOVE the fish
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.wet = true;
        n.ai[0] = -1.0;
        n.direction_y = -1;
        n.vy = -0.99;
        let mut rng = UnifiedRandom::new(7);
        ai_016_arm(157, &mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(n.direction_y, -1, "the player position is above");
        assert!(
            (n.vy - -1.01).abs() < 1e-5,
            "the 0.02 drift, NO 0.95 damp for 157 (:23957)"
        );
        assert_eq!(n.ai[0], 1.0, "-1.01 < num256 = -1 flips (:23880-23883)");
    }

    /// `NPC.cs:23990-24007` — the Orca's aim rotation: the chase zeroes
    /// ai[0], the (Center - player.Center) angle clamps to ±0.3, and the
    /// spriteDirection -1 π fold mirrors it.
    #[test]
    fn the_orca_aims_its_body_with_the_pi_fold() {
        let w = air();
        let players = [cand(1100.0, 900.0)]; // up-right of the fish
        let mut n = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n.wet = true;
        n.sprite_dir = 1;
        let mut rng = UnifiedRandom::new(8);
        ai_016_arm(692, &mut n, &w, &ctx(&players, true), &mut rng);
        assert_eq!(n.ai[0], 0.0, "the chase arm (:23768-23772)");
        assert_eq!(n.rotation, 0.3, "the down-left delta clamps at +0.3");
        let mut n2 = fish(NpcKind::Goldfish, 1000.0, 1000.0);
        n2.wet = true;
        n2.sprite_dir = -1;
        ai_016_arm(692, &mut n2, &w, &ctx(&players, true), &mut rng);
        assert_eq!(n2.rotation, -0.3, "the π fold mirrors (:23994-23997)");
    }

    /// `NPC.cs:23762-23765` — the AnglerFish's lure light: the C#'s
    /// integer-half-width anchor + the direction * (width + 8) lead, the
    /// (0.07, 0.04, 0.025) tint, as an effects request (the pure seam).
    #[test]
    fn the_anglerfish_lure_light_matches_the_csharp_anchor() {
        let w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Goldfish, 1600.0, 1600.0);
        n.wet = true;
        n.facing = 1;
        let (wf, _) = n.size();
        let wi = wf as i32;
        let want_x = (1600.0 + (wi / 2) as f32 + (wi + 8) as f32) as i32 / 16;
        let want_y = (1600.0 + 2.0) as i32 / 16;
        let mut rng = UnifiedRandom::new(9);
        let fx = ai_016_arm(102, &mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(fx.light, Some((want_x, want_y, [0.07, 0.04, 0.025])));
    }

    /// `NPC.cs:23709-23736` + `Tile.cs:290-318` — the feet slope flip:
    /// topSlope = {1, 2}, leftSlope = {2, 4}; a slope-2 foot tile pushes
    /// the fish left (`vx = -|vx|`), a slope-1 pushes right.
    #[test]
    fn the_feet_slope_flips_the_swim_direction() {
        let mut w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Piranha, 1600.0, 1600.0);
        n.wet = true;
        n.vx = 2.0;
        let (wf, hf) = n.size();
        let fx = (1600.0 + wf / 2.0) as i32 / 16;
        let fy = (1600.0 + hf) as i32 / 16;
        let i = w.index(fx, fy);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].set_slope(2);
        let mut rng = UnifiedRandom::new(10);
        ai_016_step(&mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(n.facing, -1, "leftSlope(2) faces -1 (:23713-23717)");
        assert!(n.vx < 0.0, "vx = -|vx| (the idle accel may add on top)");
        w.tiles[i].set_slope(1);
        let mut n2 = fish(NpcKind::Piranha, 1600.0, 1600.0);
        n2.wet = true;
        n2.vx = -2.0;
        ai_016_step(&mut n2, &w, &ctx(&players, false), &mut rng);
        assert_eq!(n2.facing, 1, "topSlope(1) without leftSlope faces +1");
        assert!(n2.vx > 0.0, "vx = |vx|");
    }

    /// `NPC.cs:23932-23960` — the beach-avoid: liquid > 128 above the
    /// centre and a solid tile one below flips ai[0] to -1 (rise); the
    /// generic Y damp scales a > 0.4 vy by 0.95.
    #[test]
    fn the_beach_avoid_rises_and_the_y_damp_scales() {
        let mut w = air();
        let players = [cand(1300.0, 1000.0)];
        let mut n = fish(NpcKind::Piranha, 1600.0, 1600.0);
        n.wet = true;
        n.vy = 0.5;
        let (wf, hf) = n.size();
        let cx = (1600.0 + (wf as i32 / 2) as f32) as i32 / 16;
        let cy = (1600.0 + (hf as i32 / 2) as f32) as i32 / 16;
        let ia = w.index(cx, cy - 1);
        w.tiles[ia].liquid = 200;
        let ib = w.index(cx, cy + 1);
        w.tiles[ib] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[ib].flags |= FLAG_ACTIVE;
        let mut rng = UnifiedRandom::new(11);
        ai_016_step(&mut n, &w, &ctx(&players, false), &mut rng);
        assert_eq!(n.ai[0], -1.0, "the beach-avoid (:23946-23951)");
        assert!(
            (n.vy - (0.5 + 0.01) * 0.95).abs() < 1e-6,
            "the bob's 0.01 then the 0.95 damp (:23957-23960)"
        );
    }

    /// The routing pin: the port's swimmers carry aiStyle 16
    /// (`npc/data.rs`'s row, `NPC.cs:9465`/`:9508`/`:9615`) and map to
    /// their vanilla types (`NPCID.cs:10787`/`:10793`/`:10807`). W4-c4
    /// adds the nine style-16 swimmers (`NPC.cs:9494`/`:10995`/`:12111`/
    /// `:14488`/`:16684`/`:16800`/`:16897`/`:17658`/`:17714`).
    #[test]
    fn the_swimmer_kinds_carry_style_16_and_their_vanilla_types() {
        assert_eq!(NpcKind::Goldfish.ai_style(), 16);
        assert_eq!(NpcKind::Piranha.ai_style(), 16);
        assert_eq!(NpcKind::Shark.ai_style(), 16);
        assert_eq!(vanilla_type(NpcKind::Goldfish), 55);
        assert_eq!(vanilla_type(NpcKind::Piranha), 58);
        assert_eq!(vanilla_type(NpcKind::Shark), 65);
        for (kind, ty) in [
            (NpcKind::CorruptGoldfish, 57),
            (NpcKind::Arapaima, 157),
            (NpcKind::BloodFeeder, 241),
            (NpcKind::CrimsonGoldfish, 465),
            (NpcKind::GoldGoldfish, 592),
            (NpcKind::Pupfish, 607),
            (NpcKind::Dolphin, 615),
            (NpcKind::Pufferfish, 688),
            (NpcKind::Orca, 692),
        ] {
            assert_eq!(kind.ai_style(), 16, "{kind:?} aiStyle");
            assert_eq!(vanilla_type(kind), ty, "{kind:?} vanilla type");
        }
    }

    /// `NPC.cs:74134-74180` — the Arapaima's 8-band 5-tick cycle: the wet
    /// tick counts 1, the dry tick counts 2; the row ladder is
    /// 0/1/2/1/3/4/5/4 then the counter resets at 40. WRITTEN, NOT RUN.
    #[test]
    fn the_arapaima_frame_walks_the_eight_band_cycle() {
        let mut frame = 0u32;
        let mut counter = 0.0f32;
        let mut dir = 0i32;
        // Wet: 40 ticks walk the full cycle, the row at each 5-tick band.
        let want = [0, 1, 2, 1, 3, 4, 5, 4];
        for t in 1..=40u32 {
            style16_find_frame(157, true, 0.0, 1, 0.0, 0.0, 6, &mut frame, &mut counter, &mut dir);
            let band = ((t - 1) / 5) as usize;
            if t % 5 != 0 {
                assert_eq!(frame, want[band] as u32, "wet tick {t}");
            }
        }
        assert_eq!(counter, 0.0, "the 40-tick lap resets");
        assert_eq!(dir, 1, "the sprite faces the direction");
        // Dry: the double-count reaches the reset at tick 20.
        let mut f2 = 0u32;
        let mut c2 = 0.0f32;
        for _ in 0..20 {
            style16_find_frame(157, false, 0.0, 1, 0.0, 0.0, 6, &mut f2, &mut c2, &mut dir);
        }
        assert_eq!(c2, 0.0, "the dry double-count laps at 20");
    }

    /// `NPC.cs:74227-74243` — the Dolphin's 7-tick strip over rows
    /// 0..count-2 (count 5 → rows 0..3, the 28-tick lap); the dry leap
    /// (ai[2]==1) pins row 4. WRITTEN, NOT RUN.
    #[test]
    fn the_dolphin_frame_walks_the_seven_tick_strip() {
        let mut frame = 0u32;
        let mut counter = 0.0f32;
        let mut dir = 0i32;
        // Wet: the counter advances 1/tick, the row is counter/7.
        for t in 1..=28u32 {
            style16_find_frame(615, true, 0.0, 1, 0.0, 0.0, 5, &mut frame, &mut counter, &mut dir);
            if t < 28 {
                assert_eq!(frame, (t / 7) as u32, "wet tick {t}");
            }
        }
        assert_eq!(counter, 0.0, "the 28-tick lap resets");
        // The dry leap pins row 4 and returns (no counter advance).
        let mut f2 = 0u32;
        let mut c2 = 5.0f32;
        style16_find_frame(615, false, 0.0, 1, 0.0, 1.0, 5, &mut f2, &mut c2, &mut dir);
        assert_eq!(f2, 4, "the dry leap row (:74231)");
        assert_eq!(c2, 5.0, "the leap return leaves the counter");
    }

    /// `NPC.cs:74244-74290` — the Pufferfish: the puff (ai[2]==1) walks
    /// rows 4/5 on the 6-tick counter; the wet flap walks rows 0..3 on the
    /// 8-tick counter. WRITTEN, NOT RUN.
    #[test]
    fn the_pufferfish_frame_puffs_and_flaps() {
        // Puffed: from row 4, the 6-tick counter steps to row 5.
        let mut frame = 4u32;
        let mut counter = 0.0f32;
        let mut dir = 0i32;
        for _ in 0..6 {
            style16_find_frame(688, false, 0.0, 1, 0.0, 1.0, 6, &mut frame, &mut counter, &mut dir);
        }
        assert_eq!(frame, 5, "the puff's second row (:74255)");
        assert_eq!(counter, 0.0);
        // A puffed frame outside 4/5 snaps to 4 (:74258-74261).
        let mut f2 = 0u32;
        let mut c2 = 0.0f32;
        style16_find_frame(688, false, 0.0, 1, 0.0, 1.0, 6, &mut f2, &mut c2, &mut dir);
        assert_eq!(f2, 4, "the puff snap");
        // Wet flap: rows 0..3 on the 8-tick counter, wrap at row 4.
        let mut f3 = 0u32;
        let mut c3 = 0.0f32;
        for t in 1..=32u32 {
            style16_find_frame(688, true, 0.0, 1, 0.0, 0.0, 6, &mut f3, &mut c3, &mut dir);
            assert!(f3 <= 3, "wet tick {t} row {f3} stays in 0..3");
        }
    }

    /// `NPC.cs:74693-74762` — the Orca's two-half strip: the ai[0]==0 half
    /// offsets by count/2 (8), the wet flap advances on the 5-tick counter
    /// (ai[0]==0) or 7-tick (else); the sprite flips AGAINST the direction.
    /// WRITTEN, NOT RUN.
    #[test]
    fn the_orca_frame_walks_the_two_half_strip() {
        let mut frame = 0u32;
        let mut counter = 0.0f32;
        let mut dir = 0i32;
        // ai[0]==0 (flag3): the wet 5-tick flap over the offset half 8..15.
        style16_find_frame(692, true, 0.0, 1, 0.0, 0.0, 16, &mut frame, &mut counter, &mut dir);
        assert_eq!(dir, -1, "the Orca flips against the direction (:74697)");
        assert_eq!(counter, 1.0, "the first tick counts 1");
        for _ in 0..4 {
            style16_find_frame(692, true, 0.0, 1, 0.0, 0.0, 16, &mut frame, &mut counter, &mut dir);
        }
        assert_eq!(frame, 9, "the 5th tick flaps 8 → 9 (the offset half)");
        assert_eq!(counter, 0.0, "the 5-tick lap resets");
        // ai[0]!=0: the unoffset half 0..7 on the 7-tick counter.
        let mut f2 = 0u32;
        let mut c2 = 0.0f32;
        for _ in 0..7 {
            style16_find_frame(692, true, 0.0, 1, 1.0, 0.0, 16, &mut f2, &mut c2, &mut dir);
        }
        assert_eq!(f2, 1, "the unoffset half steps 0 → 1");
    }
}
