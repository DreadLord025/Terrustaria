//! S1-D-3 (aiStyle-64) — the firefly family, `NPC.AI`'s
//! `else if (this.aiStyle == 64)` block (`NPC.cs:34019-34248`), ported 1:1
//! as a PURE velocity machine (code-only mode: no compiler/tests were run
//! on this branch; the witness tests below are written, not executed).
//!
//! Family identity, verified from the C# `SetDefaults` census (the four
//! `this.aiStyle = 64;` rows in `NPC.cs`; NOT the projectile family —
//! `Projectile.cs:4670`/`:4692`'s aiStyle 64 is the Martian Saucer's
//! 384/386 deathray shards, a different dispatch):
//!
//! * `NPCID.Firefly` (355) — `ID/NPCID.cs:11387`, `NPC.cs:13517-13530`:
//!   10×10, damage 0, defense 0, lifeMax 5, npcSlots 0.2, `noGravity`,
//!   catchItem 1992 (the Firefly, bait 20).
//! * `NPCID.LightningBug` (358) — `ID/NPCID.cs:11393`, `NPC.cs:13559-13572`:
//!   12×12, the same zero-stat row, npcSlots 0.2, catchItem 2004.
//! * `NPCID.Lavafly` (654) — `ID/NPCID.cs:11985`, `NPC.cs:17265-17279`:
//!   10×10, `lavaImmune`, catchItem 4847 (bait 25).
//! * `NPCID.Shimmerfly` (677) — `ID/NPCID.cs:12031`, `NPC.cs:17579-17593`:
//!   10×10, `noTileCollide`, catchItem 5350.
//!
//! There is no `AI_064_*` method in `NPC.cs` — the arm is inline in `AI()`.
//! The per-kind branches INSIDE the arm: the 677 world-margin + threat-flee
//! block (`:34070-34153`), the 358/654/else glow colours (`:34160-34171`)
//! and the 677 rotation tail (`:34234-34237`).
//!
//! The port's four kinds (`NpcKind::Firefly`/`LightningBug`/`Lavafly`/
//! `Shimmerfly`, vanilla ids 355/358/654/677) landed with W4-c7: the
//! `SetDefaults` rows ride `npc/data.rs` + `npc.rs`'s tables, the
//! aiStyle-64 axis claim lives in `update_rng_dust`, and the SIM runs
//! this arm as a POOL step in `world_sim/npcs.rs`'s tick closure (the
//! 677 threat scan reads the slot-ordered pool; the style75 precedent).
//! The solo `update_rng` path runs [`ai_064_step`] with an empty sibling
//! view; both feed the glow request to the shared `EntityLights` pair
//! (the sim at the slot turn, the solo wrapper via `wants_ai_light`).
//! The integration tail is `Npc::ai_064_integrate` (the dead-band snap +
//! the noTileCollide/`UpdateCollision` split, `NPC.cs:90473-90503`).
//! The port has no `NpcKind` for the 355/358/654 trio yet (W4-c7's kind
//! wave), so those three stay UNWIRED; the Shimmerfly 677 landed with
//! **W4-c9 (2026-11-21)** — `NpcKind::Shimmerfly` claims the kind-gated
//! aiStyle-64 axis arm (`npc.rs::ai_064_shimmerfly`): the sim pre-pass
//! stamps the slot-ordered threat views (`shimmerfly_threat_views`),
//! the solo path runs the scan over the empty list (the style75 solo
//! fold), and the noGravity/noTileCollide tail is the raw integration.
//! The wiring step for the trio adds their kinds + the glow feed.
//! (W4-g62 adopts the c9 rowing + wiring verbatim for the TRAILCACHE
//! WIDENING's 677→60 row — the merge dedupes.)
//! W4-c9's kind-gated Shimmerfly arm + `shimmerfly_threat_views`
//! pre-pass DEDUPED to this routing at the batch-E merge (keep-one,
//! research/349 §5); c9's shimmer Transform(677) wiring survives on
//! the snail adapter + the sim's `wants_transform` drain.
//!
//! Seam: the module writes only the fields the C# arm writes — velocity,
//! `ai[0]`/`ai[1]`/`ai[3]`, `localAI[0..3]`, `scale`, `rotation` (677),
//! `direction` (`Npc::facing`) and `netUpdate`. It does NOT integrate
//! position: the arm ends before `UpdateCollision`. `TargetClosest()`
//! (`:34032`) rides `npc_target::target_closest` UNMODIFIED (style 64 —
//! the `:77761` fighter arm is style-3-only, so no `directionY` rewrite).
//!
//! Documented folds (all cited):
//! * `Main.netMode != 1` → true (the port is single-player) — the rearm
//!   block (`:34023-34068`) always runs.
//! * `Lighting.AddLight(i2, j2, r, g, b)` (`:34162`/`:34166`/`:34170`) →
//!   an [`Ai64Effects::light`] request; the wiring feeds the shared
//!   `EntityLights` pair (`npc_lights.rs`'s seam, S1-E-5). The tile
//!   coordinates are computed HERE because the C# truncates `(int)Center`
//!   BEFORE the `/16` (`:34156-34157`) — NOT the `>>4` floor of
//!   `ToTileCoordinates` (see below).
//! * `Vector2 vector121 = base.Center + zero * 10f` (`:34148`) is a DEAD
//!   read in the C# (never used afterwards) — skipped, no observable
//!   effect, no RNG.
//! * The threat scan loops `Main.npc[0..maxNPCs]` (`:34120-34128`) in slot
//!   order; the port's NPC list is a compacted `Vec`, so the caller hands
//!   the arm slot-ordered [`Ai64NpcView`]s (the f32 sum order is part of
//!   the behaviour). The firefly itself is in the C# loop but its
//!   `damage == 0` excludes it (`:34123`); the caller may include or drop
//!   its own view — both are byte-identical.
//! * `Main.tileSolid[type]` is the RUNTIME table: `WorldGen.cs:15714`
//!   clears `[192]` (living-tree leaves) after generation, so the scan
//!   excludes 192 (the `collision.rs::solid_kind` precedent). Platforms
//!   are `tileSolidTop`, NOT `tileSolid` — they do not bounce a firefly;
//!   liquid does (`:34197`'s `|| liquid > 0`, the down-scan only).
//!
//! ## Faithfulness traps kept (each witnessed below)
//!
//! * The 677 TOP-margin clamp writes `base.velocity.Y = 3f` (`:34098`),
//!   NOT `num959 = 3f` — a vanilla quirk: the aim keeps its unclamped
//!   value while the velocity is pinned. The other three margins clamp
//!   the aim (`num958`/`num959`).
//! * `Center.ToTileCoordinates()` (`:34073`) is `(int)X >> 4`
//!   (`Utils.cs:1807-1810`) — an ARITHMETIC shift (floor for negatives);
//!   the glow tail's `(int)Center.X / 16` (`:34156-34157`) truncates
//!   toward zero. The two differ below coordinate 0.
//! * The down-scan's `velocity.Y *= 0.9f` sits INSIDE the 4-tile loop
//!   (`:34200-34203`) — up to four damps in one tick, one per blocking
//!   tile; the up-scan's single damp (`:34223-34226`) fires once, after
//!   the 30-tile scan found no solid.
//! * `num963 -= Main.rand.Next(100, 251)` (`:34050`) is a CONDITIONAL
//!   draw — only when the firefly is below `player.Y - 100`. The stream
//!   order shifts with the geometry.
//! * The normalize divides are f64 `Math.Sqrt` over an INT sum, cast to
//!   f32 BEFORE the divide (`:34052`/`:34062`); NO zero guard — two zero
//!   draws give Infinity/NaN and the C# passes them through into
//!   `ai[0]`/`ai[1]` and the velocity blend. Kept.
//! * `Entity.DirectionFrom` (`Entity.cs:210-213`) is XNA
//!   `Vector2.Normalize(Center - Source)`: the f32 sum, the f64 sqrt cast
//!   to f32, the reciprocal, the multiply — a zero vector yields NaN
//!   (0 × ∞), which the threat sum then carries. Kept.
//! * `Hitbox.Distance(Center)` is `Utils.cs:1507-1542`'s rect-to-point
//!   distance: the `FloatIntersect` zero-size-point overlap (touching
//!   edges count), the axis bands, then the corner `Vector2.Distance`
//!   (f32). The Hitbox itself is `(int)position` (`Entity.cs:171-183`).
//!
//! ## RNG by construction
//!
//! The arm's ONLY `Main.rand` draws, in C# order:
//! 1. the scale latch `Next(75, 111)` (`:34028`) — once, when
//!    `ai[3] == 0`, drawn EVERY tick until latched, BEFORE the rearm
//!    check; `scale = ai[3]` then rides it (`:34069`).
//! 2. on rearm (`localAI[0]` counts to `<= 0`, `:34030`): `Next(60, 180)`
//!    (the timer, `:34033`) — AFTER `TargetClosest()` (`:34032`, which
//!    draws nothing) — then the FAR branch (`> 700` px and
//!    `localAI[3] == 0`): the speed `Next(50, 151)` / `Next(150, 201)`
//!    (`> 1000`) / `Next(100, 151)` (`> 850`) (`:34037-34045`), then
//!    `Next(100, 251)` (the direction-scaled X, `:34046`), then
//!    `Next(-50, 51)` (`:34047`), then the CONDITIONAL `Next(100, 251)`
//!    Y-bias (`:34050`); or the NEAR branch: `Next(5, 151)` (`:34059`),
//!    `Next(-100, 101)` ×2 (`:34060-34061`).
//! 3. the non-677 idle else (`:34179-34186`): `Next(30, 180)` (the glow
//!    pause), then the CONDITIONAL `Next(10, 30)` (the glow length) when
//!    `!Main.dayTime || position.Y / 16f > worldSurface + 10.0` (the
//!    compare is f64: `(double)(Y / 16f) > worldSurface + 10.0`).
//! Both 2 and 3 can fire on the SAME tick (the rearm precedes the state
//! machine). Nothing else draws: the threat scan, the margins, the
//! blend, the bounces and the rotation tail are pure maths.
//! `UnifiedRandom` (`tr_core::unified_random`) is the exact `Main.rand`
//! stream; the witnesses mirror the cited `Next` calls on a cloned
//! stream and compare state (`PartialEq`), so a drifted order or a
//! skipped draw fails by construction.

#![allow(dead_code)] // WIRED by W4-c7 (the pool pass + axis 64); the witnesses stay as the audit cross-check

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::TILE_SOLID;
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPCID.Firefly` (`ID/NPCID.cs:11387`; `SetDefaults` `NPC.cs:13517-13530`).
pub const FIREFLY: i32 = 355;
/// `NPCID.LightningBug` (`ID/NPCID.cs:11393`; `NPC.cs:13559-13572`).
pub const LIGHTNING_BUG: i32 = 358;
/// `NPCID.Lavafly` (`ID/NPCID.cs:11985`; `NPC.cs:17265-17279`).
pub const LAVAFLY: i32 = 654;
/// `NPCID.Shimmerfly` (`ID/NPCID.cs:12031`; `NPC.cs:17579-17593`).
pub const SHIMMERFLY: i32 = 677;

/// One `Main.npc[slot]` as the 677 threat scan reads it (`:34120-34128`):
/// `active`, `damage > 0`, `!friendly`, the `Hitbox` (int-truncated
/// position, `Entity.cs:171-183`) and the `Center` (float halves,
/// `Entity.cs:50-60`). Slot order — the f32 sum accumulates in it.
#[derive(Clone, Copy, Debug)]
pub struct Ai64NpcView {
    pub active: bool,
    pub damage: i32,
    pub friendly: bool,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

impl Ai64NpcView {
    /// `Entity.Center` — `position + (float)width / 2f` (FLOAT halves).
    pub fn center(&self) -> (f32, f32) {
        (
            self.x + self.width as f32 / 2.0,
            self.y + self.height as f32 / 2.0,
        )
    }
}

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai64Ctx<'a> {
    /// The `Main.player` slots, indexed by player slot — the same slice
    /// feeds `TargetClosest` (`:34032`, the `npc_target.rs` seam) and the
    /// 677 threat scan (`:34129-34137`, which checks ONLY `active` — a
    /// dead or ghost player still scares a Shimmerfly). The C# arrays
    /// always hold 255 slots, so slot 0 exists even when inactive (the
    /// `SetTargetTrackingValues` no-target fallback reads it).
    pub players: &'a [TargetCandidate],
    /// `Main.npc[0..maxNPCs]` in slot order (the 677 threat scan). Empty
    /// on a non-677 tick — the scan never runs for the other three.
    pub npcs: &'a [Ai64NpcView],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (the S1-D-3p
    /// precedent).
    pub ticker: u64,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct Ai64Effects {
    /// The glow tail's `Lighting.AddLight(i2, j2, r, g, b)`
    /// (`:34162`/`:34166`/`:34170`) — `(tile_x, tile_y, [r, g, b])`, the
    /// colours already scaled by `this.scale`. `None` on every other
    /// tick. The wiring feeds the shared `EntityLights` pair.
    pub light: Option<(i32, i32, [f32; 3])>,
}

/// `WorldGen.InWorld(x, y, 2)` (`WorldGen.cs:9688-9695`, fluff 2).
fn in_world2(world: &World, x: i32, y: i32) -> bool {
    x >= 2 && x < world.width - 2 && y >= 2 && y < world.height - 2
}

/// The arm's `Main.tileSolid[type]` read — the RUNTIME table: the
/// generated `TILE_SOLID` minus the living-tree leaves (192) that
/// `WorldGen.cs:15714` clears after generation (the
/// `collision.rs::solid_kind` precedent). `tileSolidTop` (platforms) is
/// NOT in this set.
fn tile_solid_runtime(kind: u16) -> bool {
    TILE_SOLID.has(kind) && kind != 192
}

/// `Tile.nactive()` (`Tile.cs:240-247`): active AND not actuated-off
/// (`(sTileHeader & 0x60) == 32`). The arm's tile tests carry no extra
/// `!inActive()` conjunct beyond this (unlike `Collision`'s).
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

/// XNA `Vector2.Distance` — `sqrt(DistanceSquared)`, f32 sum, f64 sqrt
/// cast back.
fn vector_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// `Utils.FloatIntersect(r, point-as-zero-rect)` (`Utils.cs:982-989`) —
/// the strict compares make a touching edge an overlap.
fn point_in_rect(rx: f32, ry: f32, rw: f32, rh: f32, px: f32, py: f32) -> bool {
    !(rx > px || ry > py || rx + rw < px || ry + rh < py)
}

/// `Rectangle.Distance(this Rectangle r, Vector2 point)`
/// (`Utils.cs:1507-1542`): 0 inside (touching counts), the axis-band
/// distances, then the four corner `Vector2.Distance`s.
///
/// `pub(crate)`: the minion target-command path shares it — the right-click
/// aim (`Player.MinionNPCTargetAim`, `Player.cs:56920`) and the per-tick
/// invalidation (`Player.UpdateMinionTarget`, `:57009`) both measure
/// `NPC.Hitbox.Distance(point)`, NOT a centre-to-centre distance (S1-P-i27).
pub(crate) fn rect_distance(rx: i32, ry: i32, rw: i32, rh: i32, px: f32, py: f32) -> f32 {
    let (l, t) = (rx as f32, ry as f32);
    let (r, b) = ((rx + rw) as f32, (ry + rh) as f32);
    if point_in_rect(l, t, rw as f32, rh as f32, px, py) {
        return 0.0;
    }
    if px >= l && px <= r {
        if py < t {
            return t - py;
        }
        return py - b;
    }
    if py >= t && py <= b {
        if px < l {
            return l - px;
        }
        return px - r;
    }
    if px < l {
        if py < t {
            return vector_distance(px, py, l, t);
        }
        return vector_distance(px, py, l, b);
    }
    if py < t {
        return vector_distance(px, py, r, t);
    }
    vector_distance(px, py, r, b)
}

/// The family entry — `NPC.AI`'s `else if (this.aiStyle == 64)`
/// (`NPC.cs:34019-34248`). `ty` is the vanilla `NPC.type`; `npc` must
/// carry the kind's `SetDefaults` box (10×10, or the Lightning Bug's
/// 12×12) — until the kinds land, the caller pins it via
/// `size_override` (the tentacle precedent).
pub fn ai_064_step(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai64Ctx,
    rng: &mut UnifiedRandom,
) -> Ai64Effects {
    debug_assert!(
        ty == FIREFLY || ty == LIGHTNING_BUG || ty == LAVAFLY || ty == SHIMMERFLY,
        "aiStyle 64 has four members (355/358/654/677)"
    );
    let mut fx = Ai64Effects::default();

    // `:34021-34022` — the aim locals ride ai[0]/ai[1].
    let mut num958 = npc.ai[0];
    let mut num959 = npc.ai[1];

    // `:34023-34068` — the rearm block (`Main.netMode != 1` folds true).
    // The decrement runs EVERY tick, BEFORE the scale latch draw.
    npc.local_ai[0] -= 1.0;
    // `:34026-34029` — the one-time scale latch: Next(75, 111) * 0.01f.
    if npc.ai[3] == 0.0 {
        npc.ai[3] = rng.next_range(75, 111) as f32 * 0.01;
    }
    if npc.local_ai[0] <= 0.0 {
        // `:34032` — TargetClosest BEFORE the timer draw (it draws
        // nothing; the tracking writes ride npc_target.rs unmodified).
        target_closest(npc, world, ctx.players, true, 64, ctx.ticker);
        // `:34033`.
        npc.local_ai[0] = rng.next_range(60, 180) as f32;
        // `:34034` — the horizontal gap to the (possibly new) target's
        // Center.X (float halves).
        let p = &ctx.players[npc.target as usize];
        let (cx, _cy) = npc.center();
        let num960 = (cx - (p.x + p.width as f32 / 2.0)).abs();
        // `:34035-34055` — the FAR wander: > 700 px and the near-latch
        // (localAI[3]) never set.
        if num960 > 700.0 && npc.local_ai[3] == 0.0 {
            // `:34037-34045` — the speed ladder, one draw per tick.
            let mut num961 = rng.next_range(50, 151) as f32 * 0.01;
            if num960 > 1000.0 {
                num961 = rng.next_range(150, 201) as f32 * 0.01;
            } else if num960 > 850.0 {
                num961 = rng.next_range(100, 151) as f32 * 0.01;
            }
            // `:34046-34047` — the direction-scaled X, the Y spread.
            let num962 = npc.facing * rng.next_range(100, 251);
            let mut num963 = rng.next_range(-50, 51);
            // `:34048-34051` — the CONDITIONAL downward-bias draw: only
            // when the firefly sits below player.Y - 100.
            if npc.y > p.y - 100.0 {
                num963 -= rng.next_range(100, 251);
            }
            // `:34052-34054` — the int-sum f64 sqrt, cast to f32 BEFORE
            // the divide; NO zero guard (the C# NaN/Inf pass-through).
            let num964 =
                num961 / ((num962 * num962 + num963 * num963) as f64).sqrt() as f32;
            num958 = num962 as f32 * num964;
            num959 = num963 as f32 * num964;
        } else {
            // `:34056-34065` — the NEAR wander: the latch sticks at 1
            // for the NPC's whole life (nothing in the arm clears it).
            npc.local_ai[3] = 1.0;
            let num965 = rng.next_range(5, 151) as f32 * 0.01;
            let num966 = rng.next_range(-100, 101);
            let num967 = rng.next_range(-100, 101);
            let num968 =
                num965 / ((num966 * num966 + num967 * num967) as f64).sqrt() as f32;
            num958 = num966 as f32 * num968;
            num959 = num967 as f32 * num968;
        }
        // `:34066`.
        npc.net_update = true;
    }

    // `:34069` — scale rides the latched ai[3] every tick.
    npc.scale = npc.ai[3];

    if ty == SHIMMERFLY {
        // `:34070-34153` — the Shimmerfly's world-margin keep-in and
        // threat flee. `ToTileCoordinates` is `(int)X >> 4`
        // (Utils.cs:1807-1810) — the arithmetic shift floors negatives.
        let mut flag67 = true;
        let (cx, cy) = npc.center();
        let point4 = ((cx as i32) >> 4, (cy as i32) >> 4);
        let num969 = 40;
        // `:34075-34092` — the X margins clamp the aim at ±3.
        if point4.0 < num969 {
            num958 += 0.5;
            if num958 > 3.0 {
                num958 = 3.0;
            }
            flag67 = false;
        } else if point4.0 > world.width - num969 {
            num958 -= 0.5;
            if num958 < -3.0 {
                num958 = -3.0;
            }
            flag67 = false;
        }
        // `:34093-34110` — the Y margins. THE TOP QUIRK: the clamp
        // writes `base.velocity.Y = 3f` (`:34098`), NOT num959 — the
        // aim keeps its unclamped value. The bottom clamps the aim.
        if point4.1 < num969 {
            num959 += 0.5;
            if num959 > 3.0 {
                npc.vy = 3.0;
            }
            flag67 = false;
        } else if point4.1 > world.height - num969 {
            num959 -= 0.5;
            if num959 < -3.0 {
                num959 = -3.0;
            }
            flag67 = false;
        }
        // `:34111-34152` — the flee: on cooldown just decrement; else,
        // only when no margin fired (flag67), scan and push away.
        if npc.local_ai[1] > 0.0 {
            npc.local_ai[1] -= 1.0;
        } else if flag67 {
            npc.local_ai[1] = 15.0;
            let mut num970 = 0.0f32;
            let mut zero = (0.0f32, 0.0f32);
            // `:34120-34128` — the NPC scan, slot order: active,
            // damage > 0, not friendly, Hitbox.Distance(Center) <= 100.
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
                        num970 += 1.0;
                        // `Entity.DirectionFrom` (Entity.cs:210-213):
                        // Normalize(Center - Source) — away from it.
                        let (sx, sy) = view.center();
                        let (dx, dy) = xna_normalize(cx - sx, cy - sy);
                        zero.0 += dx;
                        zero.1 += dy;
                    }
                }
            }
            // `:34129-34137` — the player scan: ONLY `active` (a dead
            // or ghost player still counts), distance <= 150.
            for p in ctx.players {
                if p.active {
                    let d =
                        rect_distance(p.x as i32, p.y as i32, p.width, p.height, cx, cy);
                    if d <= 150.0 {
                        num970 += 1.0;
                        let (sx, sy) =
                            (p.x + p.width as f32 / 2.0, p.y + p.height as f32 / 2.0);
                        let (dx, dy) = xna_normalize(cx - sx, cy - sy);
                        zero.0 += dx;
                        zero.1 += dy;
                    }
                }
            }
            // `:34138-34151` — the push: average, ×2, add to velocity,
            // the 8-cap via SafeNormalize(Zero) (Utils.cs:1837-1844 —
            // zero/NaN falls back to (0,0), so a NaN velocity zeroes).
            if num970 > 0.0 {
                let num973 = 2.0f32;
                zero.0 /= num970;
                zero.1 /= num970;
                zero.0 *= num973;
                zero.1 *= num973;
                npc.vx += zero.0;
                npc.vy += zero.1;
                let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
                if len > 8.0 {
                    let (nx, ny) = if (npc.vx == 0.0 && npc.vy == 0.0)
                        || npc.vx.is_nan()
                        || npc.vy.is_nan()
                    {
                        (0.0, 0.0)
                    } else {
                        xna_normalize(npc.vx, npc.vy)
                    };
                    npc.vx = nx * 8.0;
                    npc.vy = ny * 8.0;
                }
                // `:34148` — `vector121 = Center + zero * 10f` is a dead
                // read in the C# (never used) — skipped, no effect.
                // `:34149-34150` — the flee shortens the rearm timer.
                npc.local_ai[0] = 10.0;
                npc.net_update = true;
            }
        }
    } else if npc.local_ai[2] > 0.0 {
        // `:34154-34174` — the glow tail (non-677). The tile coords
        // TRUNCATE: `(int)Center.X / 16` (`:34156-34157`), NOT the
        // `>>4` floor of ToTileCoordinates.
        let (cx, cy) = npc.center();
        let i2 = (cx as i32) / 16;
        let j2 = (cy as i32) / 16;
        // `:34158-34172` — the light fires only while localAI[2] > 3;
        // the per-type colours ride the CURRENT scale (ai[3]).
        if npc.local_ai[2] > 3.0 {
            let s = npc.scale;
            let rgb = if ty == LIGHTNING_BUG {
                // `:34162` — the teal glow.
                [0.10124999 * s, 0.21374999 * s, 0.225 * s]
            } else if ty == LAVAFLY {
                // `:34166` — the ember glow.
                [0.225 * s, 0.105000004 * s, 0.060000002 * s]
            } else {
                // `:34170` — the Firefly's green (the else arm).
                [0.109500006 * s, 0.15 * s, 0.0615 * s]
            };
            fx.light = Some((i2, j2, rgb));
        }
        // `:34173` — the decrement runs on EVERY glow tick.
        npc.local_ai[2] -= 1.0;
    } else if npc.local_ai[1] > 0.0 {
        // `:34175-34178` — the dark pause.
        npc.local_ai[1] -= 1.0;
    } else {
        // `:34179-34186` — the idle re-roll: the pause length, then the
        // CONDITIONAL glow length (night, or below worldSurface + 10 —
        // the f64 compare over the f32 quotient).
        npc.local_ai[1] = rng.next_range(30, 180) as f32;
        if !world.day_time || (npc.y / 16.0) as f64 > world.world_surface + 10.0 {
            npc.local_ai[2] = rng.next_range(10, 30) as f32;
        }
    }

    // `:34187-34189` — the 1/80 blend toward the aim, f32 in the C#'s
    // order: (v * (float)(80 - 1) + aim) / (float)80.
    let num974 = 80.0f32;
    npc.vx = (npc.vx * (num974 - 1.0) + num958) / num974;
    npc.vy = (npc.vy * (num974 - 1.0) + num959) / num974;

    // `:34190-34206` — falling: the 4-tile down-scan. A blocking tile
    // (solid OR liquid) flips the aim AND damps the velocity ×0.9 —
    // INSIDE the loop, once per blocking tile (up to four damps).
    if npc.vy > 0.0 {
        let num975 = 4;
        let (cx, cy) = npc.center();
        let num976 = (cx as i32) / 16;
        let num977 = (cy as i32) / 16;
        for num978 in num977..num977 + num975 {
            if in_world2(world, num976, num978) {
                if let Some(t) = world.tile(num976, num978) {
                    if (nactive(t) && tile_solid_runtime(t.kind)) || t.liquid > 0 {
                        num959 *= -1.0;
                        if npc.vy > 0.0 {
                            npc.vy *= 0.9;
                        }
                    }
                }
            }
        }
    }
    // `:34207-34228` — rising: the 30-tile down-scan for ANY solid; if
    // none, flip the aim and damp ONCE (liquid does not count here).
    if npc.vy < 0.0 {
        let num979 = 30;
        let mut flag68 = false;
        let (cx, cy) = npc.center();
        let num980 = (cx as i32) / 16;
        let num981 = (cy as i32) / 16;
        for num982 in num981..num981 + num979 {
            if in_world2(world, num980, num982) {
                if let Some(t) = world.tile(num980, num982) {
                    if nactive(t) && tile_solid_runtime(t.kind) {
                        flag68 = true;
                    }
                }
            }
        }
        if !flag68 {
            num959 *= -1.0;
            if npc.vy < 0.0 {
                npc.vy *= 0.9;
            }
        }
    }
    // `:34229-34233` — the wall bounce: the aim turns AWAY from the
    // velocity's sign (>= 0 → -|aim|, the NaN compare folds to the
    // first arm), the velocity kicks back ×-0.2.
    if npc.collide_x {
        num958 = if !(npc.vx < 0.0) {
            0.0 - num958.abs()
        } else {
            num958.abs()
        };
        npc.vx *= -0.2;
    }
    // `:34234-34237` — the Shimmerfly's tilt.
    if ty == SHIMMERFLY {
        npc.rotation = npc.vx * 0.3;
    }
    // `:34238-34245` — the facing: two separate ifs; at vx == 0 (or
    // NaN) neither fires and the direction keeps its old value.
    if npc.vx < 0.0 {
        npc.facing = -1;
    }
    if npc.vx > 0.0 {
        npc.facing = 1;
    }
    // `:34246-34247` — the aim write-back, LAST.
    npc.ai[0] = num958;
    npc.ai[1] = num959;
    fx
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the
    //! cited `Next` calls on a cloned `UnifiedRandom` and compare the
    //! stream state (`PartialEq`) so a drifted draw order fails by
    //! construction.
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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

    /// A stand-in firefly body: no aiStyle-64 kind exists yet, so the
    /// witnesses drive `ai_064_step(ty, ...)` on a spare kind with the
    /// C#'s 10×10 box (`NPC.cs:13519-13520`) pinned via `size_override`
    /// — the kinds themselves land at wiring. `ai[3]` pre-latched so
    /// the scale draw stays out of the way unless a test wants it.
    fn fly(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Slime, x, y, 1, 0.0);
        n.size_override = Some((10.0, 10.0));
        n.hp = 5;
        n.max_hp = 5;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0; // a live rearm timer by default
        n.local_ai[1] = 5.0; // a live dark pause by default
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], npcs: &'a [Ai64NpcView]) -> Ai64Ctx<'a> {
        Ai64Ctx {
            players,
            npcs,
            ticker: 0,
        }
    }

    fn solid(w: &mut World, x: i32, y: i32) {
        let t = w.tile_mut(x, y).unwrap();
        *t = Tile {
            kind: 1, // stone — in TILE_SOLID
            flags: FLAG_ACTIVE,
            ..Tile::default()
        };
    }

    /// The census: the four `this.aiStyle = 64;` rows and their NPCIDs.
    #[test]
    fn s1_d_3_s64_the_family_is_the_four_fireflies() {
        assert_eq!(FIREFLY, 355, "NPCID.cs:11387, NPC.cs:13521");
        assert_eq!(LIGHTNING_BUG, 358, "NPCID.cs:11393, NPC.cs:13563");
        assert_eq!(LAVAFLY, 654, "NPCID.cs:11985, NPC.cs:17269");
        assert_eq!(SHIMMERFLY, 677, "NPCID.cs:12031, NPC.cs:17583");
    }

    /// `NPC.cs:34026-34029` + `:34069` — the scale latch draws
    /// Next(75, 111) ONCE (ai[3] == 0), then scale rides it every tick.
    #[test]
    fn s1_d_3_s64_the_scale_latch_draws_once_and_sticks() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[3] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let want = mirror.next_range(75, 111) as f32 * 0.01;
        let fx = ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "exactly one Next(75,111), nothing else");
        assert_eq!(n.ai[3], want, ":34028");
        assert_eq!(n.scale, want, ":34069");
        assert_eq!(fx.light, None);
        // The next tick: latched — no draw.
        let untouched = rng.clone();
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, untouched, "the latch never re-draws");
        assert_eq!(n.scale, want);
    }

    /// `NPC.cs:34030-34033` + `:34056-34066` — the NEAR rearm: the
    /// timer draw Next(60,180), then Next(5,151), Next(-100,101) ×2,
    /// in order; localAI[3] latches to 1; netUpdate fires.
    #[test]
    fn s1_d_3_s64_the_near_rearm_draws_the_cited_quad_in_order() {
        let w = air();
        let players = [cand(1620.0, 1500.0)]; // 25 px away — near
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0; // decrements to 0 → rearm
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(60, 180);
        let speed = mirror.next_range(5, 151) as f32 * 0.01;
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "the quad, in C# order, nothing else");
        assert_eq!(n.local_ai[0], timer as f32, ":34033");
        assert_eq!(n.local_ai[3], 1.0, ":34058 — the near latch");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm, ":34063");
        // The air world's up-scan (`:34220-34222`) flips a negative aim
        // once — the write-back carries |aim| either way.
        assert_eq!(n.ai[1], (ay as f32 * norm).abs(), ":34064 + the up-scan flip");
        assert!(n.net_update, ":34066");
    }

    /// `NPC.cs:34035-34054` — the FAR rearm (> 700, latch unset): the
    /// base speed Next(50,151), the direction-scaled Next(100,251),
    /// Next(-50,51), and the CONDITIONAL Next(100,251) Y-bias when the
    /// fly is below player.Y - 100. Stream-clone replay pins the order.
    #[test]
    fn s1_d_3_s64_the_far_rearm_draws_the_y_bias_only_below_the_player() {
        let w = air();
        // The player 800 px right and 200 px ABOVE the fly: the gap is
        // > 700 (far) and fly.y > player.y - 100 → the bias draw fires.
        let players = [cand(2400.0, 1300.0)];
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(60, 180);
        let speed = mirror.next_range(50, 151) as f32 * 0.01;
        let ax = n.facing * mirror.next_range(100, 251);
        let mut ay = mirror.next_range(-50, 51);
        ay -= mirror.next_range(100, 251); // the conditional draw
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "the five draws, in C# order");
        assert_eq!(n.local_ai[0], timer as f32);
        assert_eq!(n.local_ai[3], 0.0, "the far branch never latches");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm);
        // The bias makes ay negative → the air up-scan flips it once.
        assert_eq!(n.ai[1], (ay as f32 * norm).abs(), "the up-scan flip");

        // The player 200 px BELOW: fly.y (1500) <= player.y - 100
        // (1600) → NO bias draw; the stream is one draw shorter.
        let players2 = [cand(2400.0, 1700.0)];
        let mut n2 = fly(1600.0, 1500.0);
        n2.local_ai[0] = 1.0;
        let mut rng2 = UnifiedRandom::new(11);
        let mut mirror2 = rng2.clone();
        let _ = mirror2.next_range(60, 180);
        let speed2 = mirror2.next_range(50, 151) as f32 * 0.01;
        let ax2 = n2.facing * mirror2.next_range(100, 251);
        let ay2 = mirror2.next_range(-50, 51);
        ai_064_step(FIREFLY, &mut n2, &w, &ctx(&players2, &[]), &mut rng2);
        assert_eq!(rng2, mirror2, "four draws — the bias is skipped");
        let norm2 = speed2 / ((ax2 * ax2 + ay2 * ay2) as f64).sqrt() as f32;
        assert_eq!(n2.ai[1], (ay2 as f32 * norm2).abs(), "the air up-scan flip");
    }

    /// `NPC.cs:34038-34045` — the far speed ladder: > 1000 draws
    /// Next(150,201), > 850 draws Next(100,151) — each REPLACING the
    /// base Next(50,151) draw (both draws happen, the second wins).
    #[test]
    fn s1_d_3_s64_the_far_speed_ladder_draws_both_and_keeps_the_second() {
        let w = air();
        let players = [cand(2800.0, 1700.0)]; // 1200 px — the > 1000 band
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(5);
        let mut mirror = rng.clone();
        let _ = mirror.next_range(60, 180);
        let _base = mirror.next_range(50, 151); // drawn, then replaced
        let speed = mirror.next_range(150, 201) as f32 * 0.01;
        let ax = n.facing * mirror.next_range(100, 251);
        let ay = mirror.next_range(-50, 51);
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "the base draw is NOT skipped");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm, "the 1.50-2.00 speed wins");
    }

    /// `NPC.cs:34035` + `:34058` — once localAI[3] latches, even a
    /// 1200-px gap takes the NEAR branch (the quad, not the ladder).
    #[test]
    fn s1_d_3_s64_the_near_latch_overrides_the_far_gap() {
        let w = air();
        let players = [cand(2800.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0;
        n.local_ai[3] = 1.0; // already latched
        let mut rng = UnifiedRandom::new(9);
        let mut mirror = rng.clone();
        let _ = mirror.next_range(60, 180);
        let _ = mirror.next_range(5, 151);
        let _ = mirror.next_range(-100, 101);
        let _ = mirror.next_range(-100, 101);
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "the near quad despite the far gap");
    }

    /// `NPC.cs:34025` + `:34030` — a live timer only decrements; the
    /// steering path draws NOTHING (scale latched, pause live).
    #[test]
    fn s1_d_3_s64_the_live_timer_decrements_without_drawing() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[0] = 1.0;
        n.ai[1] = -1.0;
        let mut rng = UnifiedRandom::new(3);
        let untouched = rng.clone();
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, untouched, "no rearm, no idle roll — no draws");
        assert_eq!(n.local_ai[0], 49.0);
        assert_eq!(n.local_ai[1], 4.0, "the dark pause decrements");
    }

    /// `NPC.cs:34093-34101` — THE TOP-MARGIN QUIRK: the clamp writes
    /// velocity.Y = 3, NOT the aim; the aim keeps num959 + 0.5 even
    /// beyond 3, and flag67 = false suppresses the flee scan.
    #[test]
    fn s1_d_3_s64_the_shimmerfly_top_margin_pins_velocity_not_the_aim() {
        let w = air();
        let players = [cand(1620.0, 300.0)];
        // Centre y = 305 → tile 19 < 40: the top margin fires.
        let mut n = fly(1600.0, 300.0);
        n.ai[1] = 5.0; // the aim already beyond the clamp
        n.local_ai[1] = 0.0; // the flee would be armed if flag67 held
        let mut rng = UnifiedRandom::new(1);
        let untouched = rng.clone();
        ai_064_step(SHIMMERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, untouched, "the margin path draws nothing");
        // The aim: 5 + 0.5 = 5.5, NOT clamped (the quirk). The pin at
        // :34098 writes velocity.Y = 3 BEFORE the :34189 blend, so the
        // final vy is the blend of the PINNED 3.0 toward the aim 5.5.
        let blended = (0.0f32 * 79.0 + 5.5) / 80.0;
        let want_vy = (3.0f32 * 79.0 + 5.5) / 80.0;
        assert_eq!(n.vy, want_vy, "the pin feeds the blend, not the aim");
        assert_eq!(n.ai[1], 5.5, "the aim is unclamped (the quirk)");
        assert_ne!(n.vy, blended, "a num959 clamp would give this instead");
        assert_eq!(n.local_ai[1], 0.0, "flag67 = false: no flee re-arm");
    }

    /// `NPC.cs:34075-34092` — the X margins clamp the AIM at ±3 and
    /// add ±0.5 per tick; the left margin fires below tile 40.
    #[test]
    fn s1_d_3_s64_the_shimmerfly_left_margin_clamps_the_aim_at_three() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(300.0, 1500.0); // centre x 305 → tile 19 < 40
        n.ai[0] = 2.8;
        let mut rng = UnifiedRandom::new(1);
        ai_064_step(SHIMMERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.ai[0], 3.0, "2.8 + 0.5 capped at 3 (:34078-34081)");
        // A second tick: 3 + 0.5 stays capped.
        ai_064_step(SHIMMERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.ai[0], 3.0);
    }

    /// `NPC.cs:34111-34151` — the flee: a hostile NPC within 100 and a
    /// player within 150 both count; the averaged ×2 push rides the
    /// velocity, localAI[1] = 15, localAI[0] = 10, netUpdate. A
    /// friendly or damage-0 NPC never counts.
    #[test]
    fn s1_d_3_s64_the_shimmerfly_flees_the_averaged_threat_push() {
        let w = air();
        // The player 60 px right of the fly's centre — inside 150.
        let players = [cand(1660.0, 1480.0)];
        // A hostile zombie-centre 40 px left — inside 100.
        let npcs = [
            Ai64NpcView {
                active: true,
                damage: 20,
                friendly: false,
                x: 1540.0,
                y: 1490.0,
                width: 18,
                height: 40,
            },
            // A friendly town NPC at the same spot — excluded.
            Ai64NpcView {
                active: true,
                damage: 10,
                friendly: true,
                x: 1540.0,
                y: 1490.0,
                width: 18,
                height: 40,
            },
            // A damage-0 critter — excluded.
            Ai64NpcView {
                active: true,
                damage: 0,
                friendly: false,
                x: 1540.0,
                y: 1490.0,
                width: 10,
                height: 10,
            },
        ];
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[1] = 0.0; // the flee is armed
        let mut rng = UnifiedRandom::new(2);
        let untouched = rng.clone();
        ai_064_step(SHIMMERFLY, &mut n, &w, &ctx(&players, &npcs), &mut rng);
        assert_eq!(rng, untouched, "the flee draws nothing");
        assert_eq!(n.local_ai[1], 15.0, ":34117");
        assert_eq!(n.local_ai[0], 10.0, ":34149 — the shortened rearm");
        assert!(n.net_update, ":34150");
        // Two threats: the zombie centre (1549, 1510) and the player
        // centre (1670, 1501); the fly centre (1605, 1505). The push
        // is the average of the two away-directions ×2 — the X parts
        // partly cancel, the sum is nonzero.
        assert!(n.vx != 0.0 || n.vy != 0.0, "the push moved the velocity");
        let len = ((n.vx * n.vx + n.vy * n.vy) as f64).sqrt();
        assert!(len <= 8.0 + 1e-4, "the 8-cap (:34144-34147)");
    }

    /// `NPC.cs:34111-34114` — a live flee cooldown only decrements;
    /// the scan never runs (no push, no timer write).
    #[test]
    fn s1_d_3_s64_the_flee_cooldown_decrements_without_scanning() {
        let w = air();
        let players = [cand(1610.0, 1500.0)]; // right on top of the fly
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[1] = 7.0;
        let before_vx = n.vx;
        let mut rng = UnifiedRandom::new(2);
        ai_064_step(SHIMMERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.local_ai[1], 6.0);
        assert_eq!(n.local_ai[0], 49.0, "the rearm timer, not 10");
        assert_eq!(n.vx, before_vx * 79.0 / 80.0, "blend only, no push");
    }

    /// `NPC.cs:34154-34173` — the glow tail: localAI[2] > 3 lights the
    /// per-type colour at the TRUNCATED tile coords; the decrement
    /// runs every glow tick; at localAI[2] <= 3 the light stops but
    /// the decrement continues.
    #[test]
    fn s1_d_3_s64_the_glow_colours_and_the_truncated_tile_coords() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        // Centre (1605.5, 1505.5) → (int)/16 = (100, 94).
        for (ty, want) in [
            (LIGHTNING_BUG, [0.10124999f32, 0.21374999, 0.225]),
            (LAVAFLY, [0.225, 0.105000004, 0.060000002]),
            (FIREFLY, [0.109500006, 0.15, 0.0615]),
        ] {
            let mut n = fly(1600.5, 1500.5);
            n.scale = 0.9;
            n.ai[3] = 0.9;
            n.local_ai[2] = 10.0;
            let mut rng = UnifiedRandom::new(4);
            let untouched = rng.clone();
            let fx = ai_064_step(ty, &mut n, &w, &ctx(&players, &[]), &mut rng);
            assert_eq!(rng, untouched, "the glow tail draws nothing");
            let (i, j, rgb) = fx.light.expect("localAI[2] > 3 lights");
            assert_eq!((i, j), (100, 94), ":34156-34157 truncates");
            assert_eq!(rgb, [want[0] * 0.9, want[1] * 0.9, want[2] * 0.9]);
            assert_eq!(n.local_ai[2], 9.0, ":34173");
        }
        // localAI[2] == 3: no light, still decrements.
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[2] = 3.0;
        let mut rng = UnifiedRandom::new(4);
        let fx = ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(fx.light, None, ":34158 — the > 3 gate");
        assert_eq!(n.local_ai[2], 2.0);
    }

    /// The glow coords TRUNCATE toward zero while ToTileCoordinates
    /// FLOORS — a negative-centre fly lights tile 0, not -1
    /// (`:34156-34157` vs Utils.cs:1807-1810).
    #[test]
    fn s1_d_3_s64_the_glow_coords_truncate_where_the_margins_floor() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = fly(-8.0, -8.0); // centre (-3, -3)
        n.local_ai[2] = 10.0;
        let mut rng = UnifiedRandom::new(4);
        let fx = ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let (i, j, _) = fx.light.expect("the glow fires");
        assert_eq!((i, j), (0, 0), "(int)(-3)/16 truncates to 0");
        // The 677 margin read of the same centre floors: -3 >> 4 = -1.
        assert_eq!((-3i32) >> 4, -1, "the ToTileCoordinates floor");
    }

    /// `NPC.cs:34179-34186` — the idle roll: Next(30,180) always; the
    /// Next(10,30) glow draw only at night or below worldSurface + 10
    /// (the f64 compare). Day + above surface: ONE draw.
    #[test]
    fn s1_d_3_s64_the_idle_roll_draws_the_glow_only_at_night_or_deep() {
        let mut w = air(); // day_time = true, world_surface = 75
        let players = [cand(2000.0, 1500.0)];
        // Day, y = 1000 → 62.5 <= 85: no glow draw.
        let mut n = fly(1600.0, 1000.0);
        n.local_ai[1] = 0.0;
        n.local_ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(6);
        let mut mirror = rng.clone();
        let pause = mirror.next_range(30, 180);
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "one draw — the day surface skips the glow");
        assert_eq!(n.local_ai[1], pause as f32);
        assert_eq!(n.local_ai[2], 0.0);
        // Night: both draws, in order.
        w.day_time = false;
        let mut n2 = fly(1600.0, 1000.0);
        n2.local_ai[1] = 0.0;
        let mut rng2 = UnifiedRandom::new(6);
        let mut mirror2 = rng2.clone();
        let pause2 = mirror2.next_range(30, 180);
        let glow = mirror2.next_range(10, 30);
        ai_064_step(FIREFLY, &mut n2, &w, &ctx(&players, &[]), &mut rng2);
        assert_eq!(rng2, mirror2, "the pair, in C# order");
        assert_eq!(n2.local_ai[1], pause2 as f32);
        assert_eq!(n2.local_ai[2], glow as f32);
        // Day again, but deep: y = 1400 → 87.5 > 85 → the glow draw.
        w.day_time = true;
        let mut n3 = fly(1600.0, 1400.0);
        n3.local_ai[1] = 0.0;
        let mut rng3 = UnifiedRandom::new(6);
        let mut mirror3 = rng3.clone();
        let _ = mirror3.next_range(30, 180);
        let glow3 = mirror3.next_range(10, 30);
        ai_064_step(FIREFLY, &mut n3, &w, &ctx(&players, &[]), &mut rng3);
        assert_eq!(rng3, mirror3, "the deep day glows too");
        assert_eq!(n3.local_ai[2], glow3 as f32);
    }

    /// `NPC.cs:34030-34066` + `:34179-34186` — the rearm AND the idle
    /// roll can fire on the SAME tick; the rearm's draws precede the
    /// idle pair. Stream-clone replay pins the full sequence.
    #[test]
    fn s1_d_3_s64_the_rearm_and_idle_draws_share_a_tick_in_order() {
        let mut w = air();
        w.day_time = false;
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0; // rearm this tick
        n.local_ai[1] = 0.0; // idle else this tick too
        n.local_ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(21);
        let mut mirror = rng.clone();
        let _ = mirror.next_range(60, 180); // the rearm timer
        let _ = mirror.next_range(5, 151); // the near quad
        let _ = mirror.next_range(-100, 101);
        let _ = mirror.next_range(-100, 101);
        let pause = mirror.next_range(30, 180); // then the idle pair
        let glow = mirror.next_range(10, 30);
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "six draws: the rearm quad, then the pair");
        assert_eq!(n.local_ai[1], pause as f32);
        assert_eq!(n.local_ai[2], glow as f32);
    }

    /// `NPC.cs:34187-34206` — the blend and the down-scan: the aim
    /// flips per blocking tile and the velocity damps ×0.9 PER HIT
    /// (two stacked liquid tiles → two damps), all in f32 order.
    #[test]
    fn s1_d_3_s64_the_down_scan_damps_once_per_blocking_tile() {
        let mut w = air();
        let players = [cand(2000.0, 1500.0)];
        // Centre (1605, 1505) → tile (100, 94); the scan reads rows
        // 94..98. Two liquid tiles at 94 and 95.
        w.tile_mut(100, 94).unwrap().liquid = 255;
        w.tile_mut(100, 95).unwrap().liquid = 100;
        let mut n = fly(1600.0, 1500.0);
        n.ai[1] = 4.0; // a downward aim
        n.vy = 2.0;
        let mut rng = UnifiedRandom::new(8);
        let untouched = rng.clone();
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, untouched, "the bounce path draws nothing");
        // Blend first: vy = (2*79 + 4)/80 = 2.025; then two ×0.9 damps.
        let blended = (2.0f32 * 79.0 + 4.0) / 80.0;
        assert_eq!(n.vy, blended * 0.9 * 0.9, "two hits, two damps");
        // The aim flipped TWICE (back to +4) — one flip per hit.
        assert_eq!(n.ai[1], 4.0, ":34199 flips per blocking tile");
    }

    /// `NPC.cs:34207-34228` — the up-scan: 30 rows, solid only (liquid
    /// does not count); no solid → ONE flip + ONE damp. A solid within
    /// the 30 rows keeps the rise.
    #[test]
    fn s1_d_3_s64_the_up_scan_flips_once_when_thirty_rows_are_clear() {
        let mut w = air();
        let players = [cand(2000.0, 1500.0)];
        // Liquid only — the up-scan ignores it.
        w.tile_mut(100, 100).unwrap().liquid = 255;
        let mut n = fly(1600.0, 1500.0);
        n.ai[1] = -4.0;
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(8);
        ai_064_step(FIREFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let blended = (-2.0f32 * 79.0 + -4.0) / 80.0;
        assert_eq!(n.vy, blended * 0.9, "one damp, not per-tile");
        assert_eq!(n.ai[1], 4.0, "the aim flipped once");
        // A solid 20 rows down (inside the 30): the rise stands.
        solid(&mut w, 100, 114);
        let mut n2 = fly(1600.0, 1500.0);
        n2.ai[1] = -4.0;
        n2.vy = -2.0;
        let mut rng2 = UnifiedRandom::new(8);
        ai_064_step(FIREFLY, &mut n2, &w, &ctx(&players, &[]), &mut rng2);
        assert_eq!(n2.vy, blended, "no damp — the ceiling was found");
        assert_eq!(n2.ai[1], -4.0, "no flip");
    }

    /// `NPC.cs:34229-34247` — the collideX bounce (the aim turns away
    /// from the velocity sign, vx ×-0.2), the 677 rotation tail, the
    /// two-if facing and the LAST ai write-back.
    #[test]
    fn s1_d_3_s64_the_wall_bounce_rotation_and_write_back() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[0] = 2.0;
        n.vx = 1.0; // moving right into the wall
        n.collide_x = true;
        let mut rng = UnifiedRandom::new(8);
        ai_064_step(SHIMMERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        // Blend: vx = (1*79 + 2)/80 = 1.0125; bounce: ×-0.2.
        let blended = (1.0f32 * 79.0 + 2.0) / 80.0;
        assert_eq!(n.vx, blended * -0.2, ":34232");
        assert_eq!(n.ai[0], -2.0, ":34231 — away from the +velocity");
        assert_eq!(n.rotation, n.vx * 0.3, ":34236 — the 677 tilt");
        assert_eq!(n.facing, -1, ":34240 — the negative velocity faces left");
        // The non-677 kinds never write rotation.
        let mut n2 = fly(1600.0, 1500.0);
        n2.rotation = 0.7;
        n2.ai[0] = 2.0;
        let mut rng2 = UnifiedRandom::new(8);
        ai_064_step(FIREFLY, &mut n2, &w, &ctx(&players, &[]), &mut rng2);
        assert_eq!(n2.rotation, 0.7, "the Firefly keeps its rotation");
    }

    /// XNA `Vector2.Normalize` on the zero vector is NaN (0 × ∞) — the
    /// C# has no guard and the threat sum carries it; the flee's
    /// SafeNormalize then falls back to Zero (`Utils.cs:1839`).
    #[test]
    fn s1_d_3_s64_the_zero_normalize_is_nan_and_the_safe_fold_is_zero() {
        let (x, y) = xna_normalize(0.0, 0.0);
        assert!(x.is_nan() && y.is_nan(), "no zero guard — the C# NaN");
        // The rect distance: a point ON the edge is inside (0), a
        // corner point rides the f32 Euclidean distance.
        assert_eq!(rect_distance(100, 100, 20, 40, 100.0, 100.0), 0.0);
        assert_eq!(rect_distance(100, 100, 20, 40, 97.0, 96.0), 5.0);
        assert_eq!(rect_distance(100, 100, 20, 40, 110.0, 90.0), 10.0);
    }
}
