//! The snail wall-crawler family — the C#'s `else if (this.aiStyle == 67)`
//! inline arm (`NPC.cs:34348-34622`; the family has no `AI_067_*` method).
//! S1-D-3 (aiStyle-67), **code-only mode**: the machine is ported here as pure
//! functions over a single `Npc`.
//! **WIRED by W4-c9 (2026-11-21).** The three kinds (`NpcKind::Snail` /
//! `GlowingSnail` / `MagmaSnail`) claim the aiStyle-67 axis arm
//! (`npc.rs::ai_067_snail`): the adapter drains the shimmer seam (the
//! `Transform(677)` request rides `wants_transform` → the sim's
//! `transform_to_kind`; the statue despawn rides the timer seam +
//! `wants_spawn_no_cycle`), applies the returned `noGravity` flag
//! (`Npc::snail_no_gravity`) and runs the snail's `UpdateCollision` tail
//! (the `WalkDownSlope` head + `TileCollision` + `Collision_MoveWhileDry`
//! + [`snail_move_on_slopes`]). The lights feed the draw-side
//! `EntityLights` pair (`npc_lights::snail_light`). See docs/ANALYSIS.md
//! "The snail wall-crawler family (aiStyle-67)".
//!
//! The C# family is THREE types — the only `this.aiStyle = 67;` rows in
//! `SetDefaults`:
//!   * 359 `Snail`        (`NPC.cs:13573-13586`, `NPCID.cs:11395`): 12×12,
//!     dmg 0 / def 0 / lifeMax 5, npcSlots 0.5, noGravity, catchItem 2006.
//!   * 360 `GlowingSnail` (`NPC.cs:13587-13600`, `NPCID.cs:11397`): 14×14,
//!     same stats, catchItem 2007; emits a blue light.
//!   * 655 `MagmaSnail`   (`NPC.cs:17280-17294`, `NPCID.cs:11987`): 14×14,
//!     same stats + lavaImmune, catchItem 4849; emits an orange light.
//! The machine is type-parameterised (`SnailCtx::npc_type`): the 359 scale
//! roll/resize, the 360/655 lights and the 0.3/0.6 speed split all key on it.
//!
//! NOTE — this is NPC aiStyle 67, NOT projectile aiStyle 67 (the pirate
//! shots, a different dispatch in `Projectile.cs`, siblings S1-D-4b7/12).
//!
//! Determinism — every `Main.rand` draw of the arm, in source order, all on
//! the caller's [`UnifiedRandom`] (the exact `Main.rand` stream):
//!   1. `Next(80, 111)` (`:34381`) — type 359 ONLY, once, when `ai[3] == 0`
//!      AND `netMode != 1`: the scale roll `ai[3] = Next(80,111) * 0.01f`.
//!      Zero draws once `ai[3]` is set (the resize reads it, never re-rolls).
//!   2. `Next(7200)` (`:34411`) — when `ai[2] == 0` AND `netMode != 1`. The
//!      draw rides the RIGHT operand of `ai[2] == 0f && Main.rand.Next(7200)
//!      == 0`, so it is taken ONLY while `ai[2] == 0`, and a gate-FAIL draw
//!      (`!= 0`) is still consumed. On a 0 the fall timer arms (`ai[2] = 2`).
//!   3. Nothing else: the lights, the speed split, the tile probes, the
//!      wall-crawl state machine, the rotation ease and the velocity tail
//!      draw no RNG; `TargetClosest` (`:34400`) scans without `Main.rand`.
//! The witness tests replay the stream draw-for-draw against a clone.
//!
//! Seams (the side effects a pure AI fn cannot apply itself):
//!   * `noGravity` — `Npc` carries no gravity flag, so the arm's writes
//!     (`false` in the fall block `:34445`, `true` at the crawl head `:34488`)
//!     ride [`SnailEvents::no_gravity`] (last write wins; `None` on the
//!     shimmer early-return, which writes nothing).
//!   * `Lighting.AddLight` (`:34387`/`:34391`) — rides
//!     [`SnailEvents::light`] as `(tile_x, tile_y, [r,g,b])` for the caller's
//!     `EntityLights` feed (`npc_lights.rs`'s vocabulary).
//!   * `GetShimmered` (`:34357`, `NPC.cs:91548-91592`) — the statue-despawn
//!     arm (`:91550-91560`) and the `ShimmerTransformToNPC` arm
//!     (`:91561-91565`) ride [`SnailEvents`]. All three types map to 677
//!     `Shimmerfly` (`NPCID.cs:10470`'s `359,677` / `360,677` / `655,677`
//!     rows, `NPCID.cs:12031`); the W4-c9 wiring resolves the transform
//!     through `wants_transform` → `Npc::transform_to_kind` (the style75
//!     one-tick drain fold). `WorldItem.ShimmerEffect`'s Item176 cue +
//!     20-dust `Next(6)` burst is the documented visual skip (the
//!     item-shimmer path's standing fold).
//!
//! Documented skips (render / hit-path, outside the AI arm — ANALYSIS):
//!   * `Collision_WaterCollision` (`:93055`): aiStyle 67 never wets
//!     (`wetCount = 0`, `lava = false`); `Npc` has no `wetCount`, the wiring
//!     keeps `wet = false` for the family.
//!   * `HitEffect` dust/gore (`:85253-85314`): 359 dust 201 + gore 547/548,
//!     360 dust 202 + gore 549/550, 655 dust 174 + gore 1256, each death a
//!     6-dust burst with a `Next(2)` noGravity/scale roll — the hit-path RNG
//!     and particles ride the caller's strike seam (the style52 dust skips).
//!   * `GetAlpha` light floors (`:93555-93591`): 360 min RGB (75,175,255),
//!     655 min (255,100,50), alpha 255 — render-side tint, unported.

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tile::LiquidKind;
use tr_core::{UnifiedRandom, World};

/// `NPCID.Snail` (`NPCID.cs:11395`) — the scale-rolled member.
pub const SNAIL_TYPE: i32 = 359;
/// `NPCID.GlowingSnail` (`NPCID.cs:11397`) — the blue-light member.
pub const GLOWING_SNAIL_TYPE: i32 = 360;
/// `NPCID.MagmaSnail` (`NPCID.cs:11987`) — the orange-light, lava-immune member.
pub const MAGMA_SNAIL_TYPE: i32 = 655;
/// `NPCID.Shimmerfly` (`NPCID.cs:12031`) — the shimmer transform target of all
/// three types (`NPCID.cs:10470`'s `ShimmerTransformToNPC` rows).
pub const SHIMMERFLY_TYPE: i32 = 677;

/// The per-NPC inputs the machine reads that live on `Main`/`NPC` statics in
/// the C#. The wiring passes the port's stock values (single-player → 0).
#[derive(Clone, Debug)]
pub struct SnailCtx {
    /// `this.type` — 359 / 360 / 655 (the three `aiStyle = 67` rows).
    pub npc_type: i32,
    /// `Main.netMode` — the shimmer probe, the 359 scale roll and the
    /// `Next(7200)` fall roll all run when `!= 1` (`:34350`/`:34379`/
    /// `:34409`); the port is single-player → 0.
    pub net_mode: i32,
}

impl SnailCtx {
    /// The port's stock context for a snail of `npc_type` (single-player).
    pub fn stock(npc_type: i32) -> Self {
        Self { npc_type, net_mode: 0 }
    }
}

/// The side effects the machine cannot apply itself (the port has no gravity
/// flag / light list / shimmer-transform seam inside a pure AI fn). Everything
/// the `Npc` struct carries (position, velocity, `ai`, `local_ai`, `rotation`,
/// `scale`, `sprite_dir`, `direction_y`, `size_override`, `net_update`) is
/// written in place, in C# order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SnailEvents {
    /// The arm's `noGravity` writes, last write wins: `Some(false)` from the
    /// fall block (`NPC.cs:34445`), `Some(true)` from the crawl head
    /// (`:34488`). `None` on the shimmer early-return (writes nothing). The
    /// wiring applies it to the integration step's gravity gate.
    pub no_gravity: Option<bool>,
    /// `Lighting.AddLight((int)Center.X / 16, (int)Center.Y / 16, r, g, b)`
    /// (`:34387` type 360 → (0.1,0.2,0.7); `:34391` type 655 → (0.6,0.3,0.1)).
    /// The cast is BEFORE the int divide (`(int)Center.X / 16`). `None` for
    /// 359 (no light). Feeds the caller's `EntityLights`.
    pub light: Option<(i32, i32, [f32; 3])>,
    /// `GetShimmered`'s `SpawnedFromStatue` arm (`NPC.cs:91550-91560`): the
    /// statue-spawned snail is consumed by the shimmer (ShimmerEffect +
    /// noSpawnCycle + active=false). The wiring despawns it.
    pub shimmer_statue_despawn: bool,
    /// `GetShimmered`'s `ShimmerTransformToNPC` arm (`NPC.cs:91561-91565`):
    /// `Transform(SHIMMERFLY_TYPE)` + `WorldItem.ShimmerEffect(Center)`. The
    /// transform needs the unported 677 kind, so it rides the wiring.
    pub shimmer_transform: bool,
}

/// The aiStyle-67 arm (`NPC.cs:34348-34622`) — one snail's tick. The family
/// reads no sibling NPCs, so this is a single-`Npc` pure fn (the hook's pool
/// scan has no counterpart here). Returns the seam effects; the snail's
/// fields are written in place. The caller must have established the
/// `SetDefaults` box (12×12 for 359, 14×14 for 360/655) — via `kind.size()`
/// once the kinds land, or `size_override` in the witness tests.
pub fn snail_ai_step(
    npc: &mut Npc,
    ctx: &SnailCtx,
    world: &World,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
    ticker: u64,
) -> SnailEvents {
    let mut ev = SnailEvents::default();
    // `:34350-34360` — the shimmer probe (server/single-player). NOTE the
    // clamp UPPER is `maxTilesX`/`maxTilesY` INCLUSIVE (the vanilla quirk —
    // not `width - 1`); an edge index reads a null tile and skips. The cast
    // is `(int)(Center.X / 16f)` (cast AFTER the float divide).
    if ctx.net_mode != 1 {
        let (cx, cy) = npc.center();
        let num987 = ((cx / 16.0) as i32).clamp(0, world.width);
        let num988 = ((cy / 16.0) as i32).clamp(0, world.height);
        if let Some(tile) = world.tile(num987, num988) {
            if tile.liquid_kind() == LiquidKind::Shimmer && tile.liquid > 30 {
                // `GetShimmered` (`NPC.cs:91548`): the statue arm vs the
                // transform-to-677 arm; both return immediately (`:34358`).
                if npc.spawned_from_statue {
                    ev.shimmer_statue_despawn = true;
                } else {
                    ev.shimmer_transform = true;
                }
                return ev;
            }
        }
    }
    // `:34361-34384` — type 359's scale roll + box resize.
    if ctx.npc_type == SNAIL_TYPE {
        if npc.ai[3] != 0.0 {
            npc.scale = npc.ai[3];
            let num989 = (12.0 * npc.scale) as i32;
            let num990 = (12.0 * npc.scale) as i32;
            // The current box is the OLD base.width/height (read once, before
            // either resize). `base.width / 2` is an INTEGER half; the X
            // offset carries the `- 2f` quirk (`:34370`); Y uses the FULL
            // height (`:34375`).
            let (old_w, old_h) = npc.size();
            let old_w = old_w as i32;
            let old_h = old_h as i32;
            if num989 != old_w {
                npc.x = npc.x + (old_w / 2) as f32 - num989 as f32 - 2.0;
            }
            if num990 != old_h {
                npc.y = npc.y + old_h as f32 - num990 as f32;
            }
            npc.size_override = Some((num989 as f32, num990 as f32));
        }
        if npc.ai[3] == 0.0 && ctx.net_mode != 1 {
            // `:34381` — the ONE scale draw, `Next(80, 111) * 0.01f`.
            npc.ai[3] = rng.next_range(80, 111) as f32 * 0.01;
            npc.net_update = true;
        }
    }
    // `:34385-34392` — the per-type lights. Cast BEFORE the int divide:
    // `(int)base.Center.X / 16` (NOT `(int)(Center.X / 16f)`).
    if ctx.npc_type == GLOWING_SNAIL_TYPE {
        let (cx, cy) = npc.center();
        ev.light = Some((cx as i32 / 16, cy as i32 / 16, [0.1, 0.2, 0.7]));
    }
    if ctx.npc_type == MAGMA_SNAIL_TYPE {
        let (cx, cy) = npc.center();
        ev.light = Some((cx as i32 / 16, cy as i32 / 16, [0.6, 0.3, 0.1]));
    }
    // `:34393-34397` — the speed split: 0.3 for 359, 0.6 for 360/655.
    let mut num991 = 0.3f32;
    if ctx.npc_type == GLOWING_SNAIL_TYPE || ctx.npc_type == MAGMA_SNAIL_TYPE {
        num991 = 0.6;
    }
    // `:34398-34407` — the one-shot target init.
    if npc.ai[0] == 0.0 {
        target_closest(npc, world, players, true, 67, ticker);
        npc.direction_y = 1;
        npc.ai[0] = 1.0;
        if npc.facing > 0 {
            npc.sprite_dir = 1;
        }
    }
    // `:34408` — flag70 (the wall-hit latch that skips the rotation ease).
    let mut flag70 = false;
    if ctx.net_mode != 1 {
        // `:34411-34415` — the fall roll. `Next(7200)` rides the RIGHT `&&`
        // operand: drawn ONLY while `ai[2] == 0`, gate-fail draw consumed.
        if npc.ai[2] == 0.0 && rng.next_max(7200) == 0 {
            npc.ai[2] = 2.0;
            npc.net_update = true;
        }
        // `:34416-34428` — the airborne counter: 6 uncollided ticks arm the
        // fall (`localAI[3] > 5`); any collision resets it.
        if !npc.collide_x && !npc.collide_y {
            npc.local_ai[3] += 1.0;
            if npc.local_ai[3] > 5.0 {
                npc.ai[2] = 2.0;
                npc.net_update = true;
            }
        } else {
            npc.local_ai[3] = 0.0;
        }
    }
    // `:34430-34483` — the fall/walk block (`ai[2] > 0`).
    if npc.ai[2] > 0.0 {
        npc.ai[1] = 0.0;
        npc.ai[0] = 1.0;
        npc.direction_y = 1;
        if npc.vy > num991 {
            npc.rotation += npc.facing as f32 * 0.1;
        } else {
            npc.rotation = 0.0;
        }
        npc.sprite_dir = npc.facing;
        npc.vx = num991 * npc.facing as f32;
        ev.no_gravity = Some(false);
        // `:34446-34451` — the ground probe behind the snail: integer
        // `width / 2 * -direction`, cast AFTER the add, int `/ 16`. A
        // non-topSlope tile under the feet + collideY decrements the timer.
        let (w, h) = npc.size();
        let half_w = w as i32 / 2;
        let height = h as i32;
        let (cx, _) = npc.center();
        let num992 = ((cx + (half_w * -npc.facing) as f32) as i32) / 16;
        let mut num993 = ((npc.y + height as f32 + 8.0) as i32) / 16;
        if world.tile(num992, num993).is_some_and(|t| !t.top_slope()) && npc.collide_y {
            npc.ai[2] -= 1.0;
        }
        // `:34452-34457` — the ceiling-slope probe ahead: a bottomSlope tile
        // flips the direction.
        num993 = ((npc.y + height as f32 - 4.0) as i32) / 16;
        let num992 = ((cx + (half_w * npc.facing) as f32) as i32) / 16;
        if world.tile(num992, num993).is_some_and(|t| t.bottom_slope()) {
            npc.facing *= -1;
        }
        // `:34458-34464` — a wall hit while settled latches flag70 and drops
        // into the upward crawl.
        if npc.collide_x && npc.vy == 0.0 {
            flag70 = true;
            npc.ai[2] = 0.0;
            npc.direction_y = -1;
            npc.ai[1] = 1.0;
        }
        // `:34465-34482` — the stuck detector: 11 ticks at the same X force
        // the direction to +1.
        if npc.vy == 0.0 {
            if npc.local_ai[1] == npc.x {
                npc.local_ai[2] += 1.0;
                if npc.local_ai[2] > 10.0 {
                    npc.facing = 1;
                    npc.vx = npc.facing as f32 * num991;
                    npc.local_ai[2] = 0.0;
                }
            } else {
                npc.local_ai[2] = 0.0;
                npc.local_ai[1] = npc.x;
            }
        }
    }
    // `:34484-34487` — still falling: nothing else runs this tick.
    if npc.ai[2] != 0.0 {
        return ev;
    }
    // `:34488` — the crawl is gravity-free.
    ev.no_gravity = Some(true);
    // `:34489-34524` — the wall-crawl state machine. `ai[1] == 0` crawls
    // vertically (collideY arms the turn, collideX flips directionY); else
    // horizontally (collideX arms the turn, collideY flips direction).
    if npc.ai[1] == 0.0 {
        if npc.collide_y {
            npc.ai[0] = 2.0;
        }
        if !npc.collide_y && npc.ai[0] == 2.0 {
            npc.facing = -npc.facing;
            npc.ai[1] = 1.0;
            npc.ai[0] = 1.0;
        }
        if npc.collide_x {
            npc.direction_y = -npc.direction_y;
            npc.ai[1] = 1.0;
        }
    } else {
        if npc.collide_x {
            npc.ai[0] = 2.0;
        }
        if !npc.collide_x && npc.ai[0] == 2.0 {
            npc.direction_y = -npc.direction_y;
            npc.ai[1] = 0.0;
            npc.ai[0] = 1.0;
        }
        if npc.collide_y {
            npc.facing = -npc.facing;
            npc.ai[1] = 0.0;
        }
    }
    // `:34525-34619` — the rotation ease (skipped on the flag70 wall latch).
    if !flag70 {
        let num994 = npc.rotation;
        // The quadrant table writes the TARGET rotation + spriteDirection.
        if npc.direction_y < 0 {
            if npc.facing < 0 {
                if npc.collide_x {
                    npc.rotation = 1.57;
                    npc.sprite_dir = -1;
                } else if npc.collide_y {
                    npc.rotation = 3.14;
                    npc.sprite_dir = 1;
                }
            } else if npc.collide_y {
                npc.rotation = 3.14;
                npc.sprite_dir = -1;
            } else if npc.collide_x {
                npc.rotation = 4.71;
                npc.sprite_dir = 1;
            }
        } else if npc.facing < 0 {
            if npc.collide_y {
                npc.rotation = 0.0;
                npc.sprite_dir = -1;
            } else if npc.collide_x {
                npc.rotation = 1.57;
                npc.sprite_dir = 1;
            }
        } else if npc.collide_x {
            npc.rotation = 4.71;
            npc.sprite_dir = -1;
        } else if npc.collide_y {
            npc.rotation = 0.0;
            npc.sprite_dir = 1;
        }
        // `:34577-34578` — the target is captured, then the CURRENT rotation
        // is restored (the spriteDirection writes above persist; only the
        // rotation eases toward the target).
        let num995 = npc.rotation;
        npc.rotation = num994;
        // `:34579-34586` — the 6.28 wrap (strict `> 6.28`, `< 0`), the
        // `(double)` compares kept as f64 promotions.
        if npc.rotation as f64 > 6.28 {
            npc.rotation -= 6.28;
        }
        if npc.rotation < 0.0 {
            npc.rotation += 6.28;
        }
        let num996 = (npc.rotation - num995).abs();
        let num997 = 0.1f32;
        // `:34589-34618` — the ±0.1 ease with the >3.14 wraparound branch and
        // the clamp-to-target. TWO separate `if`s (not else-if), as the C#.
        if npc.rotation > num995 {
            if num996 as f64 > 3.14 {
                npc.rotation += num997;
            } else {
                npc.rotation -= num997;
                if npc.rotation < num995 {
                    npc.rotation = num995;
                }
            }
        }
        if npc.rotation < num995 {
            if num996 as f64 > 3.14 {
                npc.rotation -= num997;
            } else {
                npc.rotation += num997;
                if npc.rotation > num995 {
                    npc.rotation = num995;
                }
            }
        }
    }
    // `:34620-34621` — the crawl velocity.
    npc.vx = num991 * npc.facing as f32;
    npc.vy = num991 * npc.direction_y as f32;
    ev
}

/// `FindFrame`'s case 359/360/655 (`NPC.cs:72238-72295`). The port stores the
/// frame ROW in `Npc::frame` (the C#'s `frame.Y = num * row`, `num` the frame
/// height); the row thresholds are the C#'s absolute `frameCounter` ticks.
/// `vy > 1` → the fall rows 4/5 (6-tick phase, reset at `6*2-1 = 11`); else a
/// non-zero velocity → the walk rows 0/1/2/3/2/1 (10-tick phases, reset at
/// `10*6-1 = 59`); a zero velocity leaves frame + counter untouched
/// (`:72258-72261`'s `break`). Returns `(frame_row, frame_counter)`; the
/// wiring step drives `Npc::frame` and the private `frame_counter` from
/// `find_frame` inside `npc.rs` — the pure twin here takes both by value
/// (the `npc_ai_style49.rs::nimbus_find_frame` convention).
pub fn snail_find_frame(vx: f32, vy: f32, frame: u32, frame_counter: f32) -> (u32, f32) {
    if vy > 1.0 {
        let counter = frame_counter + 1.0;
        let num48 = 6.0;
        if counter < num48 {
            return (4, counter);
        }
        if counter >= num48 * 2.0 - 1.0 {
            return (5, 0.0);
        }
        (5, counter)
    } else if vx != 0.0 || vy != 0.0 {
        let counter = frame_counter + 1.0;
        let num49 = 10.0;
        if counter < num49 {
            return (0, counter);
        }
        if counter < num49 * 2.0 {
            return (1, counter);
        }
        if counter < num49 * 3.0 {
            return (2, counter);
        }
        if counter < num49 * 4.0 {
            return (3, counter);
        }
        if counter < num49 * 5.0 {
            return (2, counter);
        }
        if counter >= num49 * 6.0 - 1.0 {
            return (1, 0.0);
        }
        (1, counter)
    } else {
        (frame, frame_counter)
    }
}

/// `NPC.Collision_MoveSnailOnSlopes` (`NPC.cs:92704-92721`), the aiStyle-67
/// replacement for the normal slope/stair pass (`:92568-92571`). Rides the
/// ported `Collision.SlopeCollision` (`collision::slope_collision`; the C#'s
/// `gravity` argument is unused in `SlopeCollision`'s body — only
/// `WalkDownSlope` reads it, `Collision.cs:1688` — so the port's gravity-free
/// signature is faithful). When the slide moves the snail, a settled
/// downward crawl on the matching wall quadrant flips the direction, then
/// the fall timer arms and the rotation zeroes; the vector's pos/vel are
/// copied back regardless.
pub fn snail_move_on_slopes(npc: &mut Npc, world: &World) {
    let (w, h) = npc.size();
    let sc = crate::collision::slope_collision(
        world,
        [npc.x, npc.y],
        [npc.vx, npc.vy],
        w,
        h,
        false,
    );
    if npc.x != sc.pos[0] || npc.y != sc.pos[1] {
        if npc.ai[2] == 0.0
            && npc.vy > 0.0
            && ((npc.facing < 0 && npc.rotation == 1.57 && npc.sprite_dir == 1)
                || (npc.facing > 0 && npc.rotation == 4.71 && npc.sprite_dir == -1))
        {
            // `:92711` — the vanilla quirk `direction *= -direction`: for a
            // ±1 direction this is `-(direction²) = -1` ALWAYS (kept 1:1).
            npc.facing *= -npc.facing;
        }
        npc.ai[2] = 2.0;
        npc.direction_y = 1;
        npc.rotation = 0.0;
    }
    npc.x = sc.pos[0];
    npc.y = sc.pos[1];
    npc.vx = sc.vel[0];
    npc.vy = sc.vel[1];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        let mut w = World::new("t".into(), 200, 120);
        w.world_surface = 30.0;
        w.day_time = false;
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

    /// The snail stand-in: the kinds are unported, so the pool slot rides a
    /// benign kind with the C# `SetDefaults` box pinned via `size_override`
    /// (12×12 for 359, 14×14 for 360/655) and lifeMax 5. The machine reads
    /// only the fields it writes + `ctx.npc_type`.
    fn snail(npc_type: i32, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::BurningSphere, x, y, 1, 0.0);
        let b = if npc_type == SNAIL_TYPE { 12.0 } else { 14.0 };
        n.size_override = Some((b, b));
        n.hp = 5;
        n.max_hp = 5;
        n
    }

    /// A shimmer liquid tile at (x, y) with `liquid` level.
    fn shimmer_at(w: &mut World, x: i32, y: i32, liquid: u8) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { liquid, ..Default::default() };
        w.tiles[i].set_liquid_kind(LiquidKind::Shimmer);
    }

    /// A solid dirt tile with a slope at (x, y).
    fn slope_at(w: &mut World, x: i32, y: i32, slope: u8) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].set_slope(slope);
    }

    /// `NPC.cs:34350-34360` — a shimmer tile (liquid > 30) under the centre
    /// fires the transform intent and returns BEFORE any roll/movement; the
    /// stream is untouched. The statue variant fires the despawn intent.
    #[test]
    fn s1_d_3_s67_the_shimmer_gate_returns_early_and_draws_nothing() {
        let mut w = air();
        // Snail at (160,160), 12-box → centre (166,166) → tile (10,10).
        shimmer_at(&mut w, 10, 10, 40);
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 160.0, 160.0);
        n.ai = [1.0, 0.0, 0.0, 1.0]; // init done, scale set, no fall
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut rng = UnifiedRandom::new(11);
        let witness = rng.clone();
        let ev = snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert!(ev.shimmer_transform);
        assert!(!ev.shimmer_statue_despawn);
        assert_eq!(rng, witness, "the shimmer path consumes zero draws");
        assert_eq!((n.vx, n.vy), (0.0, 0.0), "no movement before the return");
        assert_eq!(ev.no_gravity, None, "the shimmer return writes noGravity nothing");
        // The statue-spawned variant despawns instead of transforming.
        let mut s = snail(SNAIL_TYPE, 160.0, 160.0);
        s.ai = [1.0, 0.0, 0.0, 1.0];
        s.spawned_from_statue = true;
        let ev2 = snail_ai_step(&mut s, &ctx, &w, &players, &mut rng, 0);
        assert!(ev2.shimmer_statue_despawn);
        assert!(!ev2.shimmer_transform);
        // liquid == 30 is NOT > 30 (strict): no shimmer, the arm runs on.
        let mut w2 = air();
        shimmer_at(&mut w2, 10, 10, 30);
        let mut n2 = snail(SNAIL_TYPE, 160.0, 160.0);
        n2.ai = [1.0, 0.0, 2.0, 1.0];
        let ev3 = snail_ai_step(&mut n2, &ctx, &w2, &players, &mut rng, 0);
        assert!(!ev3.shimmer_transform);
    }

    /// `NPC.cs:34379-34383` — type 359 with `ai[3] == 0` rolls ONE
    /// `Next(80, 111)`, sets `ai[3] = draw * 0.01f` + netUpdate; the resize is
    /// skipped this tick (it reads `ai[3]` BEFORE the roll). Witness replay.
    #[test]
    fn s1_d_3_s67_the_snail_scale_rolls_once_and_replays() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 2.0, 0.0]; // ai[2]=2 skips the Next(7200) roll
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut rng = UnifiedRandom::new(3);
        let mut witness = rng.clone();
        let d = witness.next_range(80, 111);
        let ev = snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness, "exactly one Next(80,111) draw");
        assert_eq!(n.ai[3], d as f32 * 0.01);
        assert!(n.net_update);
        assert_eq!(n.scale, 1.0, "the resize reads ai[3] before the roll — no resize this tick");
        assert_eq!(n.size(), (12.0, 12.0));
        assert_eq!(ev.no_gravity, Some(false), "the fall block wrote noGravity=false");
    }

    /// `NPC.cs:34363-34377` — a preset `ai[3]` resizes the box with the
    /// INTEGER `width/2` half, the `-2f` X quirk and the FULL-height Y offset;
    /// no RNG (the roll is gated on `ai[3] == 0`).
    #[test]
    fn s1_d_3_s67_the_snail_resize_keeps_the_integer_half_and_the_minus_two() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 2.0, 1.1]; // scale 1.1 → (int)(12*1.1) = 13
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness, "a preset ai[3] draws nothing");
        assert_eq!(n.scale, 1.1);
        assert_eq!(n.size(), (13.0, 13.0));
        // X: 100 + (12/2) - 13 - 2 = 91; Y: 100 + 12 - 13 = 99.
        assert_eq!(n.x, 91.0, "integer width/2 half + the -2f quirk (:34370)");
        assert_eq!(n.y, 99.0, "the full-height Y offset (:34375)");
    }

    /// `NPC.cs:34411` — while `ai[2] == 0` the `Next(7200)` is drawn even when
    /// it FAILS (gate-fail draw consumed); a collision resets the airborne
    /// counter. Witness replay pins the single draw.
    #[test]
    fn s1_d_3_s67_the_fall_roll_consumes_the_gate_fail_draw() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(GLOWING_SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 0.0, 0.0];
        n.collide_x = true;
        n.collide_y = true;
        let ctx = SnailCtx::stock(GLOWING_SNAIL_TYPE);
        // A seed whose first Next(7200) is non-zero (the gate fails).
        let mut seed = 0;
        for s in 0..1000 {
            if UnifiedRandom::new(s).next_max(7200) != 0 {
                seed = s;
                break;
            }
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = rng.clone();
        let d = witness.next_max(7200);
        assert_ne!(d, 0);
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness, "exactly one Next(7200), the gate-fail draw");
        assert_eq!(n.ai[2], 0.0, "the failed roll leaves the fall timer unarmed");
        assert_eq!(n.local_ai[3], 0.0, "a collision resets the airborne counter");
    }

    /// `NPC.cs:34411` — while `ai[2] != 0` the `Next(7200)` is NOT drawn (the
    /// left `&&` operand short-circuits): the stream is untouched.
    #[test]
    fn s1_d_3_s67_a_nonzero_fall_timer_skips_the_roll() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 2.0, 1.0]; // ai[2]=2, ai[3] set
        n.collide_x = true;
        n.collide_y = true;
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut rng = UnifiedRandom::new(5);
        let witness = rng.clone();
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness, "ai[2]!=0 short-circuits the Next(7200)");
    }

    /// `NPC.cs:34416-34423` — six uncollided ticks (`localAI[3] > 5`) arm the
    /// fall timer even when the `Next(7200)` fails.
    #[test]
    fn s1_d_3_s67_the_airborne_counter_arms_the_fall_after_six_ticks() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(GLOWING_SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 0.0, 0.0];
        n.local_ai[3] = 5.0; // one more uncollided tick → 6 > 5
        let ctx = SnailCtx::stock(GLOWING_SNAIL_TYPE);
        let mut seed = 0;
        for s in 0..1000 {
            if UnifiedRandom::new(s).next_max(7200) != 0 {
                seed = s;
                break;
            }
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = rng.clone();
        let _ = witness.next_max(7200);
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness);
        assert_eq!(n.local_ai[3], 6.0);
        assert_eq!(n.ai[2], 2.0, "the airborne counter arms the fall");
        assert!(n.net_update);
    }

    /// `NPC.cs:34435-34445` — the fall block spins the rotation while
    /// `vy > speed`, sets spriteDirection = direction, vx = speed*direction
    /// and noGravity = false; with no tiles the timer stays armed (return).
    #[test]
    fn s1_d_3_s67_the_fall_block_spins_and_sets_the_gravity_flag() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 2.0, 1.0];
        n.vy = 5.0; // > 0.3
        n.facing = 1;
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut rng = UnifiedRandom::new(9);
        let witness = rng.clone();
        let ev = snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness);
        assert_eq!(n.rotation, 0.1, "rotation += direction * 0.1 while falling fast");
        assert_eq!(n.sprite_dir, 1);
        assert_eq!(n.vx, 0.3);
        assert_eq!(ev.no_gravity, Some(false));
        assert_eq!(n.ai[2], 2.0, "no ground tile → the timer stays armed");
    }

    /// `NPC.cs:34398-34407` — the one-shot init runs TargetClosest, sets
    /// directionY = 1 and ai[0] = 1.
    #[test]
    fn s1_d_3_s67_the_init_targets_and_arms_the_crawl() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [0.0, 0.0, 2.0, 1.0]; // ai[0]=0 → init; ai[2]=2 skips the roll
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut rng = UnifiedRandom::new(2);
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.direction_y, 1);
        assert_eq!(n.target, 0, "TargetClosest picked the session player");
    }

    /// `NPC.cs:34488-34621` — the ground crawl: noGravity = true, the
    /// quadrant rotation + spriteDirection, and the 0.6 velocity tail for
    /// 360. The rotation eases to the target (0 here).
    #[test]
    fn s1_d_3_s67_the_ground_crawl_sets_the_light_speed_and_rotation() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(GLOWING_SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 0.0, 0.0];
        n.collide_y = true;
        n.facing = 1;
        n.direction_y = 1;
        let ctx = SnailCtx::stock(GLOWING_SNAIL_TYPE);
        let mut seed = 0;
        for s in 0..1000 {
            if UnifiedRandom::new(s).next_max(7200) != 0 {
                seed = s;
                break;
            }
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = rng.clone();
        let _ = witness.next_max(7200);
        let ev = snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness);
        assert_eq!(ev.no_gravity, Some(true));
        assert_eq!(n.ai[0], 2.0, "collideY arms the turn");
        assert_eq!(n.sprite_dir, 1);
        assert_eq!(n.rotation, 0.0);
        assert_eq!((n.vx, n.vy), (0.6, 0.6), "360 crawls at 0.6");
        // The blue light at the centre tile, cast-before-divide.
        let (cx, cy) = n.center();
        assert_eq!(ev.light, Some((cx as i32 / 16, cy as i32 / 16, [0.1, 0.2, 0.7])));
    }

    /// `NPC.cs:34567-34570` — a right-facing wall hit (collideX) targets
    /// rotation 4.71 with spriteDirection -1; the >3.14 wraparound branch
    /// eases the rotation the long way (0 → -0.1 this tick).
    #[test]
    fn s1_d_3_s67_the_wall_quadrant_flips_the_sprite_and_wraps_the_ease() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 1.0, 0.0, 1.0]; // horizontal crawl
        n.collide_x = true;
        n.facing = 1;
        n.direction_y = 1;
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut seed = 0;
        for s in 0..1000 {
            if UnifiedRandom::new(s).next_max(7200) != 0 {
                seed = s;
                break;
            }
        }
        let mut rng = UnifiedRandom::new(seed);
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(n.ai[0], 2.0, "collideX arms the horizontal turn");
        assert_eq!(n.sprite_dir, -1, "the right-wall quadrant flips the sprite");
        // target 4.71, current 0: |0-4.71| > 3.14 → the wraparound subtracts.
        assert!((n.rotation - (-0.1)).abs() < 1e-6, "the long-way ease: {}", n.rotation);
        assert_eq!(n.vx, 0.3);
    }

    /// `NPC.cs:34589-34602` — a short rotation gap eases by 0.1 and clamps to
    /// the target (no overshoot).
    #[test]
    fn s1_d_3_s67_the_rotation_ease_clamps_to_the_target() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 0.0, 1.0];
        n.collide_y = true;
        n.facing = -1;
        n.direction_y = 1;
        n.rotation = 0.05; // target 0 (the down-left ground quadrant)
        let ctx = SnailCtx::stock(SNAIL_TYPE);
        let mut seed = 0;
        for s in 0..1000 {
            if UnifiedRandom::new(s).next_max(7200) != 0 {
                seed = s;
                break;
            }
        }
        let mut rng = UnifiedRandom::new(seed);
        snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(n.sprite_dir, -1);
        assert_eq!(n.rotation, 0.0, "0.05 - 0.1 undershoots → clamped to the target 0");
    }

    /// `NPC.cs:34350`/`:34379`/`:34409` — the client (netMode 1) skips the
    /// shimmer probe, the scale roll and the fall roll: zero draws.
    #[test]
    fn s1_d_3_s67_the_client_path_draws_nothing() {
        let mut w = air();
        shimmer_at(&mut w, 6, 6, 40); // under the centre, but the client skips it
        let players = [cand(1000.0, 800.0)];
        let mut n = snail(SNAIL_TYPE, 100.0, 100.0);
        n.ai = [1.0, 0.0, 0.0, 0.0];
        let mut ctx = SnailCtx::stock(SNAIL_TYPE);
        ctx.net_mode = 1;
        let mut rng = UnifiedRandom::new(4);
        let witness = rng.clone();
        let ev = snail_ai_step(&mut n, &ctx, &w, &players, &mut rng, 0);
        assert_eq!(rng, witness, "the client path consumes zero draws");
        assert_eq!(n.ai[3], 0.0, "no scale roll on the client");
        assert!(!ev.shimmer_transform, "no shimmer probe on the client");
    }

    /// `NPC.cs:72241-72254` — the fall animation: rows 4 (<6) then 5, the
    /// counter reset at 11 (`6*2-1`).
    #[test]
    fn s1_d_3_s67_the_fall_frames_cycle_four_then_five() {
        let (mut frame, mut counter) = (0u32, 0.0f32);
        for _ in 0..6 {
            (frame, counter) = snail_find_frame(0.0, 2.0, frame, counter);
        }
        assert_eq!(frame, 5);
        assert_eq!(counter, 6.0);
        for _ in 0..5 {
            (frame, counter) = snail_find_frame(0.0, 2.0, frame, counter);
        }
        assert_eq!(frame, 5);
        assert_eq!(counter, 0.0, "reset at 6*2-1 = 11");
    }

    /// `NPC.cs:72262-72293` — the walk animation: the 10-tick phases
    /// 0/1/2/3/2/1, reset at 59 (`10*6-1`).
    #[test]
    fn s1_d_3_s67_the_walk_frames_step_the_six_phases() {
        let (mut frame, mut counter) = (0u32, 0.0f32);
        let rows = [0u32, 1, 2, 3, 2, 1];
        for (phase, &row) in rows.iter().enumerate() {
            // Advance to the middle of each 10-tick phase.
            let target = phase as f32 * 10.0 + 5.0;
            while counter < target {
                (frame, counter) = snail_find_frame(0.6, 0.0, frame, counter);
            }
            assert_eq!(frame, row, "phase {phase}");
        }
        // Run to the reset at 59.
        while counter != 0.0 {
            (frame, counter) = snail_find_frame(0.6, 0.0, frame, counter);
        }
        assert_eq!(frame, 1);
    }

    /// `NPC.cs:72258-72261` — a zero velocity leaves frame + counter alone.
    #[test]
    fn s1_d_3_s67_a_zero_velocity_freezes_the_frame() {
        let (frame, counter) = snail_find_frame(0.0, 0.0, 3, 7.0);
        assert_eq!(frame, 3);
        assert_eq!(counter, 7.0);
    }

    /// `NPC.cs:92707-92716` — when the slope slide does NOT move the snail
    /// (open air), the fall timer / rotation / direction are untouched.
    #[test]
    fn s1_d_3_s67_a_no_op_slope_slide_leaves_the_state() {
        let w = air();
        let mut n = snail(SNAIL_TYPE, 160.0, 160.0);
        n.ai = [1.0, 0.0, 0.0, 1.0];
        n.vy = 1.0;
        n.facing = -1;
        n.rotation = 1.57;
        n.sprite_dir = 1;
        snail_move_on_slopes(&mut n, &w);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.rotation, 1.57);
        assert_eq!(n.facing, -1);
        assert_eq!((n.x, n.y), (160.0, 160.0));
    }

    /// `NPC.cs:92709-92715` — a slope slide that moves the snail on the
    /// matching wall quadrant flips the direction to -1 (the `*= -direction`
    /// quirk), arms the fall timer and zeroes the rotation.
    #[test]
    fn s1_d_3_s67_the_slope_slide_flips_the_direction_to_minus_one() {
        let mut w = air();
        // A slope-1 dirt tile overlapping the snail's box forces a slide.
        slope_at(&mut w, 10, 10, 1);
        let mut n = snail(SNAIL_TYPE, 160.0, 160.0);
        n.ai = [1.0, 0.0, 0.0, 1.0];
        n.vx = 0.3;
        n.vy = 1.0; // > 0
        n.facing = -1;
        n.rotation = 1.57;
        n.sprite_dir = 1;
        snail_move_on_slopes(&mut n, &w);
        assert_ne!((n.x, n.y), (160.0, 160.0), "the slope moved the snail");
        assert_eq!(n.facing, -1, "direction *= -direction → -(direction²) = -1");
        assert_eq!(n.ai[2], 2.0);
        assert_eq!(n.direction_y, 1);
        assert_eq!(n.rotation, 0.0);
    }
}
