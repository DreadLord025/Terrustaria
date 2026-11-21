//! S1-D-3ff d39 wave 1 (aiStyle-65) — the butterfly, `NPC.AI_065_Butterflies`
//! (`NPC.cs:44896-45119`), ported 1:1 for the type-356 branches as a PURE
//! velocity machine (code-only mode: no compiler/tests were run on this
//! branch; the witness tests below are written, not executed).
//!
//! Family identity (`NPC.SetDefaults`'s `this.aiStyle = 65;` rows):
//! * `NPCID.Butterfly` (356) — `ID/NPCID.cs:11389`, `NPC.cs:13531-13544`:
//!   10x10, damage 0, defense 0, lifeMax 5, npcSlots 0.25, `noGravity`,
//!   catchItem 1994 (+ai[2]-1 per tick, `:45106`).
//! * `NPCID.GoldButterfly` (444), `NPCID.HellButterfly` (653),
//!   `NPCID.EmpressButterfly` (661) — the other three members; NOT ported
//!   in this wave (their kinds land with their own rounds), so their
//!   branches are skipped here and cited as exclusions:
//!   - the 661 head block (`:44900-44955`: the hsl light, the fade-out
//!     despawn counter, the Opacity/dust/dontTakeDamage writes);
//!   - the 661 `localAI[1] = 10f` override (`:45065-45068`);
//!   - the 653 tail (`:45108-45118`: the ember light + the 1/60 dust 6);
//!   - the FindFrame `type != 444 && != 653 && != 661` colour-band gate
//!     (`:72768`) — for 356 the band ALWAYS applies (the frame wiring's
//!     obligation, `npc.rs::find_frame`).
//!
//! Seam: the module writes only the fields the C# arm writes — velocity,
//! `ai[0]`/`ai[1]`/`ai[2]`/`ai[3]`, `localAI[0]`/`localAI[1]`/`localAI[3]`,
//! `scale`, `direction` (`Npc::facing`) and `netUpdate`. It does NOT
//! integrate position: the arm ends before `UpdateCollision`. The wiring
//! rides `integrate_flyer_collision` (the noGravity flyer seam, the bat
//! family's precedent). `TargetClosest()` (`:44979`) rides
//! `npc_target::target_closest` UNMODIFIED.
//!
//! The `catchItem = (short)(1994f + ai[2] - 1f)` per-tick re-derive
//! (`:45104-45107`) has no `Npc` field — the port's catch path re-derives
//! it from `ai[2]` at catch time (`bug_net::catch_item`'s 356 row,
//! `1994 + ai2 as i32 - 1`), so the write is a cited no-op here.
//!
//! Documented folds (all cited):
//! * `Main.netMode != 1` → true (the port is single-player) — the rearm
//!   block (`:44956-45014`) always runs.
//! * The threat scan loops `Main.npc[0..maxNPCs]` (`:45071-45079`) in slot
//!   order; the port's NPC list is a compacted `Vec`, so the caller hands
//!   the arm slot-ordered [`Ai65NpcView`]s (the f32 sum order is part of
//!   the behaviour — the style64 threat-scan precedent). The butterfly
//!   itself is in the C# loop but its `damage == 0` excludes it
//!   (`:45074`); the caller may include it.
//! * `Entity.DirectionFrom` (`Entity.cs:210-213`) is XNA
//!   `Vector2.Normalize(Center - Source)` — the f32 sum, the f64 sqrt cast
//!   to f32, the reciprocal; a zero vector yields NaN (0 × ∞), which the
//!   threat sum then carries. Kept (the style64 module's identical fold).
//! * `Vector2.Length()` (`:45083`) is `(float)Math.Sqrt(X*X + Y*Y)`;
//!   `SafeNormalize(Vector2.Zero)` (`Utils.cs:1837-1844`) returns Zero on
//!   a zero/NaN vector, else `Normalize`. Both kept verbatim.
//! * The down-scan is THREE tiles (`num27 = 3`, `:45021`) — NOT the
//!   firefly's four (`AI_064`'s `num975 = 4`); the aim flip + the 0.9 damp
//!   sit INSIDE the loop (`:45026-45033`), one per blocking tile. The
//!   up-scan's 30-tile loop (`:45036-45057`) flips + damps ONCE after the
//!   scan, liquid does not count there.
//! * `Main.tileSolid[type]` is the RUNTIME table (the style64 module's
//!   `tile_solid_runtime` fold — `WorldGen.cs:15714` clears 192).
//! * The tile coordinates TRUNCATE: `(int)Center.X / 16`
//!   (`:45022-45023`/`:45040-45041`), NOT the `>>4` floor.
//!
//! RNG draws, in C# order (the caller's `Main.rand` / `UpdateNPCs`
//! stream): the variant roll `Next(100)` (`:44968`, only while
//! `ai[2] == 0`), the scale latch `Next(75, 111)` (`:44973`, only while
//! `ai[3] == 0`), then on the rearm expiry (`localAI[0] <= 0`): the timer
//! `Next(90, 240)` (`:44978`), the FAR ladder `Next(50, 151)` +
//! conditional `Next(150, 201)`/`Next(100, 151)` + `Next(100, 251)` +
//! `Next(-50, 51)` + conditional `Next(100, 251)`
//! (`:44983-44996`), or the NEAR `Next(26, 301)` + `Next(-100, 101)` ×2
//! (`:45005-45007`). Nothing else draws.

use crate::npc::Npc;
use crate::npc_ai_style64::rect_distance;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::TILE_SOLID;
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPCID.Butterfly` (`ID/NPCID.cs:11389`; `SetDefaults` `NPC.cs:13531-13544`).
pub const BUTTERFLY: i32 = 356;

/// One `Main.npc[slot]` as the threat scan reads it (`:45071-45079`):
/// `active`, `damage > 0`, `!friendly`, the `Hitbox` (int-truncated
/// position, `Entity.cs:171-183`) and the `Center` (float halves,
/// `Entity.cs:50-60`). Slot order — the f32 sum accumulates in it.
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
    /// The `Main.player` slots, indexed by player slot — feeds
    /// `TargetClosest` (`:44979`) and the rearm's target-Center read
    /// (`:44980`/`:44994`). The C# arrays always hold 255 slots, so slot
    /// 0 exists even when inactive (the no-target fallback reads it).
    pub players: &'a [TargetCandidate],
    /// `Main.npc[0..maxNPCs]` in slot order (the threat scan,
    /// `:45071-45079`). Empty on a solo tick — the scan finds nothing.
    pub npcs: &'a [Ai65NpcView],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp — 0 until
    /// the ticker is plumbed (the S1-D-3p precedent).
    pub ticker: u64,
}

/// `WorldGen.InWorld(x, y, 2)` (`WorldGen.cs:9688-9695`, fluff 2) — the
/// style64 module's identical fold.
fn in_world2(world: &World, x: i32, y: i32) -> bool {
    x >= 2 && x < world.width - 2 && y >= 2 && y < world.height - 2
}

/// The arm's `Main.tileSolid[type]` read — the RUNTIME table (the
/// style64 module's fold: the generated `TILE_SOLID` minus the
/// living-tree leaves 192 that `WorldGen.cs:15714` clears).
fn tile_solid_runtime(kind: u16) -> bool {
    TILE_SOLID.has(kind) && kind != 192
}

/// `Tile.nactive()` (`Tile.cs:240-247`): active AND not actuated-off.
fn nactive(t: &tr_core::tile::Tile) -> bool {
    t.active() && !t.inactive()
}

/// XNA `Vector2.Normalize` — the f32 sum, the f64 sqrt cast to f32, the
/// reciprocal, then the two multiplies. A zero vector yields NaN; NO
/// guard (the C# has none — `Entity.DirectionFrom`).
fn xna_normalize(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// The family entry — `AI_065_Butterflies` (`NPC.cs:44896-45119`), the
/// type-356 branches. `npc` must carry the kind's `SetDefaults` box
/// (10x10 — `NpcKind::Butterfly.size()` once wired).
pub fn ai_065_step(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai65Ctx,
    rng: &mut UnifiedRandom,
) {
    debug_assert!(ty == BUTTERFLY, "this wave ports the type-356 branches only");

    // `:44898-44899` — the aim locals ride ai[0]/ai[1].
    let mut num = npc.ai[0];
    let mut num2 = npc.ai[1];

    // `:44956-45014` — the rearm block (`Main.netMode != 1` folds true).
    // `:44958-44970` — the one-time variant roll: Next(100), the ladder
    // picks the colour band (ai[2] = 1..7; the FindFrame band offset and
    // the catchItem re-derive both read it).
    if npc.ai[2] == 0.0 {
        let num8 = 0;
        let num9 = 4;
        let num10 = 6;
        let num11 = 3;
        let num12 = 7;
        let num13 = 2;
        let num14 = 1;
        let num15 = 5;
        let num16 = rng.next_max(100);
        npc.ai[2] = (1 + if num16 == 0 {
            num15
        } else if num16 < 3 {
            num14
        } else if num16 < 9 {
            num13
        } else if num16 < 19 {
            num12
        } else if num16 < 34 {
            num11
        } else if num16 < 53 {
            num10
        } else if num16 >= 75 {
            num8
        } else {
            num9
        }) as f32;
    }
    // `:44971-44974` — the one-time scale latch: Next(75, 111) * 0.01f.
    if npc.ai[3] == 0.0 {
        npc.ai[3] = rng.next_range(75, 111) as f32 * 0.01;
    }
    // `:44975-45013` — the rearm timer. The decrement runs EVERY tick.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        npc.local_ai[0] = rng.next_range(90, 240) as f32;
        // `:44979` — TargetClosest BEFORE the gap read (it draws nothing).
        target_closest(npc, world, ctx.players, true, 65, ctx.ticker);
        // `:44980` — the horizontal gap to the (possibly new) target's
        // Center.X (float halves).
        let p = &ctx.players[npc.target as usize];
        let (cx, _cy) = npc.center();
        let num17 = (cx - (p.x + p.width as f32 / 2.0)).abs();
        // `:44981-45001` — the FAR wander: > 700 px and the near-latch
        // (localAI[3]) never set.
        if num17 > 700.0 && npc.local_ai[3] == 0.0 {
            // `:44983-44991` — the speed ladder, one draw per tier.
            let mut num18 = rng.next_range(50, 151) as f32 * 0.01;
            if num17 > 1000.0 {
                num18 = rng.next_range(150, 201) as f32 * 0.01;
            } else if num17 > 850.0 {
                num18 = rng.next_range(100, 151) as f32 * 0.01;
            }
            // `:44992-44993` — the direction-scaled X, the Y spread.
            let num19 = npc.facing * rng.next_range(100, 251);
            let mut num20 = rng.next_range(-50, 51);
            // `:44994-44997` — the CONDITIONAL downward-bias draw: only
            // when the butterfly sits below player.Y - 100.
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
            // `:45002-45011` — the NEAR wander: the latch sticks at 1
            // for the NPC's whole life (nothing in the arm clears it).
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

    // `:45019-45035` — falling: the THREE-tile down-scan. A blocking tile
    // (solid OR liquid) flips the Y aim AND damps the velocity ×0.9 —
    // INSIDE the loop, once per blocking tile (up to three damps).
    if npc.vy > 0.0 {
        let num27 = 3;
        let (cx, cy) = npc.center();
        let num28 = (cx as i32) / 16;
        let num29 = (cy as i32) / 16;
        for j in num29..num29 + num27 {
            if in_world2(world, num28, j) {
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
            if in_world2(world, num31, k) {
                if let Some(t) = world.tile(num31, k) {
                    if nactive(t) && tile_solid_runtime(t.kind) {
                        flag2 = true;
                    }
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
    // `:45058-45088` — the threat flee, every 15th tick (the 661's 10 is
    // not this wave's). The scan sums `DirectionFrom(nPC.Center)` over
    // every live hostile within Hitbox.Distance(Center) <= 100.
    if npc.local_ai[1] > 0.0 {
        npc.local_ai[1] -= 1.0;
    } else {
        npc.local_ai[1] = 15.0;
        let mut num33 = 0.0f32;
        let mut zero = (0.0f32, 0.0f32);
        let (cx, cy) = npc.center();
        for view in ctx.npcs {
            if view.active && view.damage > 0 && !view.friendly {
                let (nx, ny) = view.center();
                // `Entity.Hitbox` — the int-truncated position.
                if rect_distance(
                    view.x as i32,
                    view.y as i32,
                    view.width,
                    view.height,
                    cx,
                    cy,
                ) <= 100.0
                {
                    num33 += 1.0;
                    // `base.DirectionFrom(nPC.Center)` —
                    // Normalize(ourCenter - theirCenter), NO guard.
                    let (dx, dy) = xna_normalize(cx - nx, cy - ny);
                    zero.0 += dx;
                    zero.1 += dy;
                }
            }
        }
        if num33 > 0.0 {
            // `:45082` — velocity += zero / num33 * 2f.
            npc.vx += zero.0 / num33 * 2.0;
            npc.vy += zero.1 / num33 * 2.0;
            // `:45083-45086` — the 16 cap: Length() > 16 →
            // SafeNormalize(Zero) * 16.
            let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
            if len > 16.0 {
                let (sx, sy) = if (npc.vx == 0.0 && npc.vy == 0.0)
                    || npc.vx.is_nan()
                    || npc.vy.is_nan()
                {
                    (0.0, 0.0)
                } else {
                    xna_normalize(npc.vx, npc.vy)
                };
                npc.vx = sx * 16.0;
                npc.vy = sy * 16.0;
            }
        }
    }
    // `:45089-45093` — the wall bounce: the aim turns AWAY from the
    // velocity's sign (>= 0 → -|aim|, the NaN compare folds to the first
    // arm), the velocity kicks back ×-0.2.
    if npc.collide_x {
        num = if !(npc.vx < 0.0) {
            0.0 - num.abs()
        } else {
            num.abs()
        };
        npc.vx *= -0.2;
    }
    // `:45094-45101` — the facing: two separate ifs; at vx == 0 (or NaN)
    // neither fires and the direction keeps its old value.
    if npc.vx < 0.0 {
        npc.facing = -1;
    }
    if npc.vx > 0.0 {
        npc.facing = 1;
    }
    // `:45102-45103` — the aim write-back, LAST.
    npc.ai[0] = num;
    npc.ai[1] = num2;
    // `:45104-45107` — the catchItem re-derive: no Npc field; the catch
    // path re-derives from ai[2] (`bug_net::catch_item`'s 356 row).
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the
    //! cited `Next` calls on a cloned `UnifiedRandom` and compare the
    //! stream state so a drifted draw order fails by construction.
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

    /// A stand-in butterfly body: the 10x10 box (`NPC.cs:13533-13534`)
    /// pinned via `size_override` until the kind's own `size()` lands;
    /// `ai[2]`/`ai[3]` pre-latched so the one-time draws stay out of the
    /// way unless a test wants them.
    fn flutter(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Slime, x, y, 1, 0.0);
        n.size_override = Some((10.0, 10.0));
        n.hp = 5;
        n.max_hp = 5;
        n.ai[2] = 4.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0; // a live rearm timer by default
        n.local_ai[1] = 5.0; // a live threat-pause by default
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], npcs: &'a [Ai65NpcView]) -> Ai65Ctx<'a> {
        Ai65Ctx { players, npcs, ticker: 0 }
    }

    /// `NPC.cs:44958-44970` — the variant ladder: Next(100) == 0 picks
    /// band 5 (ai[2] = 6), 1-2 → 1, 3-8 → 2, 9-18 → 7, 19-33 → 3,
    /// 34-52 → 6, >= 75 → 0, else 4. The roll draws exactly once and
    /// only while ai[2] == 0.
    #[test]
    fn the_variant_roll_maps_the_next100_ladder() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        // Find a seed whose first Next(100) is 0 (the rarest band).
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(100) == 0 {
                break;
            }
            seed += 1;
        }
        let mut n = flutter(160.0, 160.0);
        n.ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = rng.clone();
        let band = 1 + 5; // num16 == 0 → num15
        mirror.next_max(100);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.ai[2], band as f32, ":44969");
        assert_eq!(rng, mirror, "one Next(100), no other draw");
        // A second tick with ai[2] set draws nothing for the variant.
        let before = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, before, "the roll is one-time");
    }

    /// `NPC.cs:44971-44974` — the scale latch: Next(75, 111) * 0.01f,
    /// once; `:45015` writes scale = ai[3] every tick.
    #[test]
    fn the_scale_latches_once_and_rides_every_tick() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.ai[3] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let latch = mirror.next_range(75, 111) as f32 * 0.01;
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.ai[3], latch, ":44973");
        assert_eq!(n.scale, latch, ":45015");
        assert_eq!(rng, mirror, "one Next(75,111)");
    }

    /// `NPC.cs:44975-44978` — the rearm timer decrements every tick and
    /// re-rolls Next(90, 240) on expiry; the NEAR wander (the target
    /// within 700 px) latches localAI[3] = 1 and draws Next(26, 301) +
    /// Next(-100, 101) ×2 (`:45005-45007`).
    #[test]
    fn the_rearm_expiry_draws_the_near_wander_in_order() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.local_ai[0] = 1.0; // expires THIS tick (the decrement first)
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(90, 240) as f32;
        let speed = mirror.next_range(26, 301) as f32 * 0.01;
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.local_ai[0], timer, ":44978");
        assert_eq!(n.local_ai[3], 1.0, ":45004");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm, ":45009");
        assert_eq!(n.ai[1], ay as f32 * norm, ":45010");
        assert!(n.net_update, ":45012");
        assert_eq!(rng, mirror, "the near-wander draw order");
    }

    /// `NPC.cs:45016-45018` — the 1/60 blend: (v * 59 + aim) / 60, f32.
    #[test]
    fn the_velocity_blends_one_sixtieth_toward_the_aim() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.vx = 2.0;
        n.vy = -1.0;
        n.ai[0] = 5.0;
        n.ai[1] = 3.0;
        let mut rng = UnifiedRandom::new(3);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.vx, (2.0f32 * 59.0 + 5.0) / 60.0, ":45017");
        assert_eq!(n.vy, (-1.0f32 * 59.0 + 3.0) / 60.0, ":45018");
    }

    /// `NPC.cs:45019-45035` — the THREE-tile down-scan: each blocking
    /// tile flips the Y aim and damps vy ×0.9 INSIDE the loop (two
    /// stacked solid tiles → two flips cancel, two damps compound).
    #[test]
    fn the_down_scan_damps_once_per_blocking_tile() {
        let mut w = air();
        // The butterfly's centre tile column: centre (165, 165) → tile
        // (10, 10). Stack solid tiles at rows 10 and 11.
        for ty in [10, 11] {
            let i = w.index(10, ty);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.vy = 2.0;
        n.ai[1] = 4.0;
        let mut rng = UnifiedRandom::new(5);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        // Two flips: 4 → -4 → 4; two damps: 2 * 0.9 * 0.9 (the blend ran
        // first: vy = (2*59 + 4)/60 before the scan).
        let blended = (2.0f32 * 59.0 + 4.0) / 60.0;
        assert_eq!(n.vy, blended * 0.9 * 0.9, ":45029-45032");
        assert_eq!(n.ai[1], 4.0, "two flips cancel");
    }

    /// `NPC.cs:45058-45088` — the threat flee: every 15th tick a live
    /// hostile within Hitbox.Distance <= 100 pushes the velocity by
    /// DirectionFrom * 2 (one threat → the full away-vector ×2).
    #[test]
    fn the_threat_scan_pushes_away_from_a_nearby_hostile() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.local_ai[1] = 1.0; // expires THIS tick → the scan runs
        n.vx = 0.0;
        n.vy = 0.0;
        n.ai[0] = 0.0;
        n.ai[1] = 0.0;
        // A hostile 50 px to the RIGHT: its centre (215, 165), our
        // centre (165, 165) → DirectionFrom = (-1, 0).
        let threat = Ai65NpcView {
            active: true,
            damage: 10,
            friendly: false,
            x: 205.0,
            y: 155.0,
            width: 20,
            height: 20,
        };
        let mut rng = UnifiedRandom::new(9);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[threat]), &mut rng);
        assert_eq!(n.local_ai[1], 15.0, ":45064");
        assert_eq!(n.vx, -2.0, ":45082 — away ×2");
        assert_eq!(n.vy, 0.0);
        // A friendly neighbour never scares (the !friendly gate).
        let mut n2 = flutter(160.0, 160.0);
        n2.local_ai[1] = 1.0;
        let friendly = Ai65NpcView { friendly: true, ..threat };
        ai_065_step(BUTTERFLY, &mut n2, &w, &ctx(&players, &[friendly]), &mut rng);
        assert_eq!(n2.vx, 0.0, ":45074 — the !friendly gate");
    }

    /// `NPC.cs:45089-45093` — the wall bounce: collideX turns the aim
    /// away from the velocity's sign and kicks vx ×-0.2.
    #[test]
    fn the_wall_bounce_flips_the_aim_and_kicks_back() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.collide_x = true;
        n.vx = 1.0;
        n.ai[0] = 3.0;
        let mut rng = UnifiedRandom::new(13);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        // The blend ran first: vx = (1*59 + 3)/60 > 0 → aim = -|3|.
        let blended = (1.0f32 * 59.0 + 3.0) / 60.0;
        assert_eq!(n.vx, blended * -0.2, ":45092");
        assert_eq!(n.ai[0], -3.0, ":45091");
    }

    /// `NPC.cs:45094-45101` — the facing: two separate ifs; vx == 0
    /// keeps the old direction.
    #[test]
    fn the_facing_follows_the_velocity_sign_only() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.facing = 1;
        n.vx = -1.0;
        n.ai[0] = -60.0; // the blend keeps vx negative
        let mut rng = UnifiedRandom::new(17);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.facing, -1, ":45096");
        // Zero velocity: neither if fires.
        let mut n2 = flutter(160.0, 160.0);
        n2.facing = -1;
        n2.vx = 0.0;
        n2.ai[0] = 0.0;
        ai_065_step(BUTTERFLY, &mut n2, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n2.facing, -1, "vx == 0 keeps the direction");
    }
}
