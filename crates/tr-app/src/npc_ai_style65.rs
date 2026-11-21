//! W4-f18 (aiStyle-65) — the butterfly family, `NPC.AI`'s
//! `else if (this.aiStyle == 65)` dispatch into `AI_065_Butterflies`
//! (`NPC.cs:44896-45119`), ported 1:1 as a PURE velocity machine
//! (code-only mode: no compiler/tests were run on this branch; the
//! witness tests below are written, not executed).
//!
//! Family identity, verified from the C# `SetDefaults` census (the four
//! `this.aiStyle = 65;` rows in `NPC.cs`):
//!
//! * `NPCID.Butterfly` (356) — `ID/NPCID.cs:11389`, `NPC.cs:13531-13544`:
//!   10×10, damage 0, defense 0, lifeMax 5, npcSlots 0.25, `noGravity`,
//!   catchItem 1994 (re-derived per tick from the colour style `ai[2]`,
//!   `:45104-45107` — the port's `bug_net::catch_item` already reads the
//!   LIVE `ai[2]`, so no field write is owed here).
//! * `NPCID.GoldButterfly` (444) — `NPC.cs:14288-14302` (unported).
//! * `NPCID.HellButterfly` (653) — `NPC.cs:17251-17264` (unported; the
//!   `:45108-45118` orange-light + dust-6 tail is its own).
//! * `NPCID.FairyButterfly` (661) — `NPC.cs:17359-17375` (unported; the
//!   `:44900-44955` hsl-glow + fade-despawn head is its own).
//!
//! Only 356 has an `NpcKind` (W4-f18 first-20 port), so the 661 head and
//! the 653 tail are NOT ported — the fn is keyed on the vanilla type and
//! debug-asserts the ported member (the `npc_ai_style64` precedent).
//!
//! Seam: the machine writes only the fields the C# arm writes — velocity,
//! `ai[0..3]`, `localAI[0]/[1]/[3]`, `scale`, `direction` (`Npc::facing`)
//! and `netUpdate`. It does NOT integrate position: the arm ends before
//! `UpdateCollision` (`NPC.cs:92523`). The caller runs the no-gravity
//! tile-collision tail (`npc.rs::integrate_flyer_collision`).
//!
//! Documented folds (all cited):
//! * `Main.netMode != 1` → true (the port is single-player) — the colour
//!   roll / scale latch / rearm block (`:44956-45014`) always runs.
//! * The threat scan loops `Main.npc[0..maxNPCs]` (`:45071-45079`) in slot
//!   order; the caller hands the arm slot-ordered [`Ai65NpcView`]s (the
//!   f32 sum order is part of the behaviour). The butterfly itself is in
//!   the C# loop but its `damage == 0` excludes it (`:45074`). The SOLO
//!   `update_rng` path passes an EMPTY slice (the style64 677 wiring
//!   precedent — the sibling pool seam lands with the sim wiring; the
//!   flee kick then never fires, documented deviation).
//! * `Main.tileSolid[type]` is the RUNTIME table: `WorldGen.cs:15714`
//!   clears `[192]` (living-tree leaves) after generation (the
//!   `npc_ai_style64::tile_solid_runtime` precedent). Platforms are
//!   `tileSolidTop`, NOT `tileSolid`; liquid does block the down-scan
//!   (`:45026`'s `|| liquid > 0`), not the up-scan.
//! * The 356 `catchItem` re-derive (`:45104-45107`) is the catch path's
//!   (`bug_net::catch_item(356, ai[2])` reads the live style) — no write.
//!
//! ## Faithfulness traps kept
//!
//! * The colour roll (`:44968-44969`) is ONE `Next(100)` through the
//!   8-step ladder: 0→5, <3→1, <9→2, <19→7, <34→3, <53→6, >=75→0,
//!   else 4 — then `ai[2] = 1 + pick` (so the styles are 1..8, the
//!   Empress/Julia/Gold rows' order is NOT sequential).
//! * The rearm timer draw `Next(90, 240)` (`:44978`) comes BEFORE
//!   `TargetClosest()` (`:44979`) — the style64 arm draws it AFTER; the
//!   two families' orders differ and both are kept.
//! * The near-wander speed is `Next(26, 301)` (`:45005`) — NOT the
//!   firefly's `Next(5, 151)`.
//! * The normalize divides are f64 `Math.Sqrt` over an INT sum, cast to
//!   f32 BEFORE the divide (`:44998`/`:45008`); NO zero guard — two zero
//!   draws give Infinity/NaN and the C# passes them through. Kept.
//! * The down-scan's `velocity.Y *= 0.9f` sits INSIDE the 3-tile loop
//!   (`:45024-45034`) — up to three damps in one tick; the up-scan's
//!   single damp (`:45049-45056`) fires once after the 30-tile scan.
//! * The threat flee (`:45058-45088`) runs on a 15-tick `localAI[1]`
//!   cadence (the 661's 10 is unported): `velocity += sum/count * 2f`,
//!   then the 16-cap via `SafeNormalize(Vector2.Zero)` (`Utils.cs:
//!   1837-1844` — zero/NaN falls back to (0,0), so a NaN velocity
//!   zeroes). `Entity.DirectionFrom` (`Entity.cs:210-213`) is XNA
//!   `Vector2.Normalize(Center - Source)` — the f32 sum, the f64 sqrt
//!   cast to f32, the reciprocal, the multiply; a zero vector yields
//!   NaN, which the threat sum then carries. Kept.
//! * `Hitbox.Distance(Center)` is the rect-to-point distance
//!   (`Utils.cs:1507-1542`) — shared with the firefly arm through
//!   `npc_ai_style64::rect_distance` (the minion-aim seam, S1-P-i27).
//!
//! ## RNG by construction
//!
//! The arm's ONLY `Main.rand` draws for type 356, in C# order:
//! 1. the colour roll `Next(100)` (`:44968`) — once, when `ai[2] == 0`.
//! 2. the scale latch `Next(75, 111)` (`:44973`) — once, when
//!    `ai[3] == 0`, AFTER the colour roll.
//! 3. on rearm (`localAI[0]` counts to `<= 0`, `:44976`): `Next(90, 240)`
//!    (the timer, `:44978`) — BEFORE `TargetClosest()` (`:44979`, which
//!    draws nothing) — then the FAR branch (`> 700` px and
//!    `localAI[3] == 0`): the speed `Next(50, 151)` / `Next(150, 201)`
//!    (`> 1000`) / `Next(100, 151)` (`> 850`) (`:44983-44991`), then
//!    `Next(100, 251)` (the direction-scaled X, `:44992`), then
//!    `Next(-50, 51)` (`:44993`), then the CONDITIONAL `Next(100, 251)`
//!    Y-bias (`:44996`); or the NEAR branch: `Next(26, 301)` (`:45005`),
//!    `Next(-100, 101)` ×2 (`:45006-45007`).
//! Nothing else draws: the blend, the scans, the threat flee, the wall
//! bounce and the facing tail are pure maths. `UnifiedRandom`
//! (`tr_core::unified_random`) is the exact `Main.rand` stream.
#![allow(dead_code)] // the 444/653/661 members are unported (the census above)

use crate::npc::Npc;
use crate::npc_ai_style64::rect_distance;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::TILE_SOLID;
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPCID.Butterfly` (`ID/NPCID.cs:11389`; `NPC.cs:13531-13544`).
pub const BUTTERFLY: i32 = 356;

/// One `Main.npc[slot]` as the threat scan reads it (`:45071-45079`):
/// `active`, `damage > 0`, `!friendly`, the `Hitbox` (int-truncated
/// position, `Entity.cs:171-183`) and the `Center` (float halves,
/// `Entity.cs:50-60`). Slot order — the f32 sum accumulates in it.
/// The `npc_ai_style64::Ai64NpcView` shape, re-declared so the module
/// keeps its own seam vocabulary.
#[derive(Clone, Copy, Debug)]
pub struct Ai65NpcView {
    pub active: bool,
    pub damage: i32,
    pub friendly: bool,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

impl Ai65NpcView {
    /// `Entity.Center` — `position + (float)width / 2f` (FLOAT halves).
    fn center(&self) -> (f32, f32) {
        (
            self.x + self.width as f32 / 2.0,
            self.y + self.height as f32 / 2.0,
        )
    }
}

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai65Ctx<'a> {
    /// The `Main.player` slots — feeds `TargetClosest` (`:44979`, the
    /// `npc_target.rs` seam) and the rearm's `Main.player[target]` reads.
    pub players: &'a [TargetCandidate],
    /// `Main.npc[0..maxNPCs]` in slot order (the threat scan). Empty on
    /// the solo path (the module doc's documented deviation).
    pub npcs: &'a [Ai65NpcView],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (S1-D-3p).
    pub ticker: u64,
}

/// The arm's `Main.tileSolid[type]` read — the RUNTIME table (the
/// `npc_ai_style64::tile_solid_runtime` precedent, `WorldGen.cs:15714`).
fn tile_solid_runtime(kind: u16) -> bool {
    TILE_SOLID.has(kind) && kind != 192
}

/// `Tile.nactive()` (`Tile.cs:240-247`): active AND not actuated-off.
fn nactive(t: &tr_core::tile::Tile) -> bool {
    t.active() && !t.inactive()
}

/// XNA `Vector2.Normalize` — the f32 sum, the f64 sqrt cast to f32, the
/// reciprocal, then the two multiplies. A zero vector yields NaN
/// (0 × ∞); NO guard (the C# has none).
fn xna_normalize(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `AI_065_Butterflies` (`NPC.cs:44896-45119`), the type-356 path. `npc`
/// must carry the kind's `SetDefaults` box (10×10 — `kind.size()` once
/// the kind lands, `size_override` in the witnesses).
pub fn ai_065_step(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai65Ctx,
    rng: &mut UnifiedRandom,
) {
    debug_assert!(ty == BUTTERFLY, "only the Butterfly 356 is ported (W4-f18)");

    // `:44898-44899` — the aim locals ride ai[0]/ai[1].
    let mut num = npc.ai[0];
    let mut num2 = npc.ai[1];

    // `:44956-45014` — the host block (`Main.netMode != 1` folds true).
    // `:44958-44970` — the one-time colour roll: ONE Next(100) through
    // the 8-step ladder, ai[2] = 1 + pick.
    if npc.ai[2] == 0.0 {
        let num16 = rng.next_max(100);
        let pick = if num16 == 0 {
            5
        } else if num16 < 3 {
            1
        } else if num16 < 9 {
            2
        } else if num16 < 19 {
            7
        } else if num16 < 34 {
            3
        } else if num16 < 53 {
            6
        } else if num16 >= 75 {
            0
        } else {
            4
        };
        npc.ai[2] = (1 + pick) as f32;
    }
    // `:44971-44974` — the one-time scale latch: Next(75, 111) * 0.01f.
    if npc.ai[3] == 0.0 {
        npc.ai[3] = rng.next_range(75, 111) as f32 * 0.01;
    }
    // `:44975-44976` — the rearm countdown, EVERY tick.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        // `:44978` — the timer draw BEFORE TargetClosest (the style64
        // arm's order is the reverse; both kept).
        npc.local_ai[0] = rng.next_range(90, 240) as f32;
        // `:44979`.
        target_closest(npc, world, ctx.players, true, 65, ctx.ticker);
        // `:44980` — the horizontal gap to the target's Center.X.
        let p = &ctx.players[npc.target as usize];
        let (cx, _cy) = npc.center();
        let num17 = (cx - (p.x + p.width as f32 / 2.0)).abs();
        // `:44981-45001` — the FAR wander.
        if num17 > 700.0 && npc.local_ai[3] == 0.0 {
            // `:44983-44991` — the speed ladder, one draw per tick.
            let mut num18 = rng.next_range(50, 151) as f32 * 0.01;
            if num17 > 1000.0 {
                num18 = rng.next_range(150, 201) as f32 * 0.01;
            } else if num17 > 850.0 {
                num18 = rng.next_range(100, 151) as f32 * 0.01;
            }
            // `:44992-44993` — the direction-scaled X, the Y spread.
            let num19 = npc.facing * rng.next_range(100, 251);
            let mut num20 = rng.next_range(-50, 51);
            // `:44994-44997` — the CONDITIONAL downward-bias draw.
            if npc.y > p.y - 100.0 {
                num20 -= rng.next_range(100, 251);
            }
            // `:44998-45000` — the int-sum f64 sqrt, cast to f32 BEFORE
            // the divide; NO zero guard (the C# NaN/Inf pass-through).
            let num21 =
                num18 / ((num19 * num19 + num20 * num20) as f64).sqrt() as f32;
            num = num19 as f32 * num21;
            num2 = num20 as f32 * num21;
        } else {
            // `:45002-45011` — the NEAR wander: the latch sticks at 1.
            npc.local_ai[3] = 1.0;
            let num22 = rng.next_range(26, 301) as f32 * 0.01;
            let num23 = rng.next_range(-100, 101);
            let num24 = rng.next_range(-100, 101);
            let num25 =
                num22 / ((num23 * num23 + num24 * num24) as f64).sqrt() as f32;
            num = num23 as f32 * num25;
            num2 = num24 as f32 * num25;
        }
        // `:45012`.
        npc.net_update = true;
    }

    // `:45015` — scale rides the latched ai[3] every tick.
    npc.scale = npc.ai[3];

    // `:45016-45018` — the 1/60 blend toward the aim, f32 in the C#'s
    // order: (v * (float)(60 - 1) + aim) / (float)60.
    let num26 = 60.0f32;
    npc.vx = (npc.vx * (num26 - 1.0) + num) / num26;
    npc.vy = (npc.vy * (num26 - 1.0) + num2) / num26;

    // `:45019-45035` — falling: the 3-tile down-scan. A blocking tile
    // (solid OR liquid) flips the aim AND damps the velocity ×0.9 —
    // INSIDE the loop, once per blocking tile.
    if npc.vy > 0.0 {
        let num27 = 3;
        let (cx, cy) = npc.center();
        let num28 = (cx as i32) / 16;
        let num29 = (cy as i32) / 16;
        for j in num29..num29 + num27 {
            if let Some(t) = world.tile(num28, j) {
                if (nactive(t) && tile_solid_runtime(t.kind)) || t.liquid > 0 {
                    num2 *= -1.0;
                    if npc.vy > 0.0 {
                        npc.vy *= 0.9;
                    }
                }
            }
        }
    }
    // `:45036-45057` — rising: the 30-tile down-scan for ANY solid; if
    // none, flip the aim and damp ONCE (liquid does not count here).
    if npc.vy < 0.0 {
        let num30 = 30;
        let mut flag2 = false;
        let (cx, cy) = npc.center();
        let num31 = (cx as i32) / 16;
        let num32 = (cy as i32) / 16;
        for k in num32..num32 + num30 {
            if let Some(t) = world.tile(num31, k) {
                if nactive(t) && tile_solid_runtime(t.kind) {
                    flag2 = true;
                }
            }
        }
        if !flag2 {
            num2 *= -1.0;
            if npc.vy < 0.0 {
                npc.vy *= 0.9;
            }
        }
    }
    // `:45058-45088` — the 15-tick threat flee (the 661's 10-tick arm is
    // unported). The scan reads the slot-ordered sibling views; the
    // butterfly's own damage 0 excludes it.
    if npc.local_ai[1] > 0.0 {
        npc.local_ai[1] -= 1.0;
    } else {
        npc.local_ai[1] = 15.0;
        let mut num33 = 0.0f32;
        let mut zero = (0.0f32, 0.0f32);
        let (cx, cy) = npc.center();
        for view in ctx.npcs {
            if view.active && view.damage > 0 && !view.friendly {
                let d = rect_distance(
                    view.x as i32,
                    view.y as i32,
                    view.width,
                    view.height,
                    cx,
                    cy,
                );
                if d <= 100.0 {
                    num33 += 1.0;
                    // `base.DirectionFrom(nPC.Center)` — the XNA
                    // normalize of (own Centre - threat Centre).
                    let (tcx, tcy) = view.center();
                    let (dx, dy) = xna_normalize(cx - tcx, cy - tcy);
                    zero.0 += dx;
                    zero.1 += dy;
                }
            }
        }
        if num33 > 0.0 {
            // `:45082` — velocity += zero / num33 * 2f (the C#'s
            // divide-then-multiply order).
            npc.vx += zero.0 / num33 * 2.0;
            npc.vy += zero.1 / num33 * 2.0;
            // `:45083-45086` — the 16-cap via SafeNormalize(Zero)
            // (`Utils.cs:1837-1844` — zero/NaN falls back to (0,0)).
            let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
            if len > 16.0 {
                if (npc.vx == 0.0 && npc.vy == 0.0)
                    || npc.vx.is_nan()
                    || npc.vy.is_nan()
                {
                    npc.vx = 0.0;
                    npc.vy = 0.0;
                } else {
                    let (nx, ny) = xna_normalize(npc.vx, npc.vy);
                    npc.vx = nx * 16.0;
                    npc.vy = ny * 16.0;
                }
            }
        }
    }
    // `:45089-45093` — the wall bounce: the aim turns AWAY from the
    // velocity's sign (>= 0 → -|aim|, the NaN compare folds to the
    // first arm), the velocity kicks back ×-0.2.
    if npc.collide_x {
        num = if !(npc.vx < 0.0) {
            0.0 - num.abs()
        } else {
            num.abs()
        };
        npc.vx *= -0.2;
    }
    // `:45094-45101` — the facing: two separate ifs; at vx == 0 (or
    // NaN) neither fires and the direction keeps its old value.
    if npc.vx < 0.0 {
        npc.facing = -1;
    }
    if npc.vx > 0.0 {
        npc.facing = 1;
    }
    // `:45102-45103` — the aim write-back, LAST.
    npc.ai[0] = num;
    npc.ai[1] = num2;
    // `:45104-45107` — the 356 catchItem re-derive is the catch path's
    // (`bug_net::catch_item(356, ai[2])` reads the live style); the
    // 653/661 tails (`:45108-45118` / `:44900-44955`) are unported.
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the
    //! cited `Next` calls on a cloned `UnifiedRandom` and compare the
    //! stream state so a drifted draw order fails by construction.
    use super::*;

    fn air() -> World {
        World::new("t".into(), 400, 300)
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

    fn butterfly(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Butterfly, x, y, 1, 0.0);
        n.size_override = Some((10.0, 10.0));
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], npcs: &'a [Ai65NpcView]) -> Ai65Ctx<'a> {
        Ai65Ctx { players, npcs, ticker: 0 }
    }

    /// The `:44969` ladder as a pure fn — the witness pins every bucket
    /// boundary against the citation (the arm's own copy is inline).
    fn colour_pick(num16: i32) -> i32 {
        if num16 == 0 {
            5
        } else if num16 < 3 {
            1
        } else if num16 < 9 {
            2
        } else if num16 < 19 {
            7
        } else if num16 < 34 {
            3
        } else if num16 < 53 {
            6
        } else if num16 >= 75 {
            0
        } else {
            4
        }
    }

    #[test]
    fn the_colour_roll_ladder_maps_the_cited_buckets() {
        // `:44968-44969` — Next(100): 0→style 6, 1..2→2, 3..8→3,
        // 9..18→8, 19..33→4, 34..52→7, 53..74→5, 75..99→1.
        for (roll, style) in [
            (0, 6),
            (1, 2),
            (2, 2),
            (3, 3),
            (8, 3),
            (9, 8),
            (18, 8),
            (19, 4),
            (33, 4),
            (34, 7),
            (52, 7),
            (53, 5),
            (74, 5),
            (75, 1),
            (99, 1),
        ] {
            assert_eq!(1 + colour_pick(roll), style, "roll {roll} bucket");
        }
    }

    #[test]
    fn the_first_tick_colour_style_rides_the_streams_first_draw() {
        // The arm's ai[2] must equal 1 + ladder(the stream's first
        // Next(100)) — the colour roll is the FIRST draw of a fresh tick.
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        let mut rng = UnifiedRandom::new(0);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let first = mirror.next_max(100);
        assert_eq!(n.ai[2], (1 + colour_pick(first)) as f32);
        assert!((1.0..=8.0).contains(&n.ai[2]));
    }

    #[test]
    fn the_first_tick_draws_colour_then_scale_then_rearm_in_order() {
        // `:44968` Next(100), `:44973` Next(75,111), `:44978`
        // Next(90,240) — the rearm fires on tick 1 (localAI[0] 0 → -1),
        // then the NEAR wander (the target is close): Next(26,301),
        // Next(-100,101) ×2. Six draws, no more.
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        let mut rng = UnifiedRandom::new(123);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let _colour = mirror.next_max(100);
        let scale = mirror.next_range(75, 111);
        let timer = mirror.next_range(90, 240);
        let _speed = mirror.next_range(26, 301);
        let _dx = mirror.next_range(-100, 101);
        let _dy = mirror.next_range(-100, 101);
        assert_eq!(rng, mirror, "the arm's draw order drifted");
        assert_eq!(n.ai[3], scale as f32 * 0.01);
        assert_eq!(n.scale, n.ai[3]);
        assert_eq!(n.local_ai[0], timer as f32);
        assert_eq!(n.local_ai[3], 1.0, "the near latch sticks");
        assert!(n.net_update);
    }

    #[test]
    fn the_far_wander_draws_the_speed_ladder_and_the_conditional_bias() {
        // `:44981-45000` — a target > 1000 px away with localAI[3] == 0:
        // Next(50,151) THEN Next(150,201) (both draws — the ladder
        // overwrites), Next(100,251), Next(-50,51), and the conditional
        // Next(100,251) when the butterfly sits below player.Y - 100.
        let w = air();
        let players = [cand(3000.0, 1000.0)];
        let mut n = butterfly(1000.0, 1500.0);
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let _colour = mirror.next_max(100);
        let _scale = mirror.next_range(75, 111);
        let _timer = mirror.next_range(90, 240);
        let _base = mirror.next_range(50, 151);
        let speed = mirror.next_range(150, 201);
        let dx = mirror.next_range(100, 251);
        let mut dy = mirror.next_range(-50, 51);
        dy -= mirror.next_range(100, 251); // y 1500 > 1000 - 100
        assert_eq!(rng, mirror, "the far ladder's draw order drifted");
        let k = speed as f32 * 0.01
            / ((dx * dx + dy * dy) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], dx as f32 * k);
        // The blended vy is negative (the bias pulls the aim down-hard),
        // so the 30-tile up-scan finds no solid in the air world and
        // flips the aim's sign ONCE (`:45049-45056`) before the write-back.
        assert_eq!(n.ai[1], -(dy as f32 * k));
        assert_eq!(n.local_ai[3], 0.0, "the far branch keeps the latch clear");
    }

    #[test]
    fn the_blend_is_one_sixtieth_and_the_wall_bounce_kicks_back() {
        // `:45016-45018` — (v * 59 + aim) / 60; `:45089-45093` — the
        // collideX aim flip + the ×-0.2 kick.
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        n.ai[2] = 4.0; // the colour roll already latched
        n.ai[3] = 1.0; // the scale latch already latched
        n.local_ai[0] = 50.0; // no rearm this tick
        n.ai[0] = 6.0;
        n.ai[1] = -3.0;
        n.vx = 1.0;
        n.vy = 0.0;
        n.collide_x = true;
        let mut rng = UnifiedRandom::new(0);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "a latched tick draws NOTHING");
        // The blend ran BEFORE the bounce: vx = (1*59 + 6)/60, then
        // ×-0.2; the aim flips to -|6| (vx >= 0 at the bounce read).
        let blended = (1.0f32 * 59.0 + 6.0) / 60.0;
        assert_eq!(n.vx, blended * -0.2);
        assert_eq!(n.ai[0], -6.0);
        // The blended vy = -0.05 < 0 → the up-scan's single aim flip
        // (`:45049-45056`) + the ×0.9 velocity damp fire in the air world.
        let blended_vy = (0.0f32 * 59.0 + -3.0) / 60.0;
        assert_eq!(n.ai[1], 3.0);
        assert_eq!(n.vy, blended_vy * 0.9);
    }

    #[test]
    fn the_threat_flee_averages_the_push_and_caps_at_16() {
        // `:45058-45088` — localAI[1] == 0 arms the 15-tick scan; one
        // hostile view 50 px to the right pushes LEFT (DirectionFrom is
        // own - threat), ×2, and the 16-cap normalizes.
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        n.ai[2] = 4.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0;
        n.local_ai[1] = 0.0;
        // ai[0]/ai[1] stay 0 → the blended vy is 0, so neither tile scan
        // fires (both gate on vy != 0) and the cap is a pure X normalize.
        n.vx = -20.0; // already over the cap after the push
        let views = [Ai65NpcView {
            active: true,
            damage: 10,
            friendly: false,
            x: 1050.0,
            y: 1500.0,
            width: 20,
            height: 20,
        }];
        let mut rng = UnifiedRandom::new(0);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &views), &mut rng);
        assert_eq!(n.local_ai[1], 15.0);
        // The blended vx = -20*59/60 = -19.6667; the push is (-1,0)*2 =
        // -2 → -21.6667; the blended vy stays 0 (ai[1] 0), so the cap
        // normalizes to exactly (-16, 0).
        assert_eq!(n.vx, -16.0);
        assert_eq!(n.vy, 0.0);
    }

    #[test]
    fn the_facing_follows_the_velocity_sign_only() {
        // `:45094-45101` — two separate ifs; vx == 0 keeps the old
        // direction (the firefly arm's shared tail).
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        n.ai[2] = 4.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0;
        n.local_ai[1] = 5.0;
        n.facing = 1;
        let mut rng = UnifiedRandom::new(0);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.facing, 1, "a zero-velocity tick keeps the direction");
    }
}
