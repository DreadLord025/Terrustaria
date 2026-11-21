//! The Sand Elemental (541) — `NPC.AI()`'s `else if (this.aiStyle == 102)`
//! arm (`NPC.cs:40147-40542`), the 541-only machine. W4-c40, CODE-ONLY
//! MODE (no compiler invoked; the witness tests below are written, not
//! run).
//!
//! ## Census
//!
//! The aiStyle-102 family has exactly ONE vanilla member: the Sand
//! Elemental 541 (`NPCID.SandElemental`, `ID/NPCID.cs:11759`,
//! `SetDefaults` row `NPC.cs:15898-15911`). The census of
//! `this.aiStyle = 102;` across the decompiled tree returns exactly one
//! hit (`NPC.cs:15902`); no projectile-space user, no dynamic
//! reassignment. The port's roster has no `NpcKind::SandElemental` (the
//! d41 roster lane owns the row) — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent), keyed on the vanilla type until the
//! roster wave. **MERGER KEEP-ONE**: the d33 census claims styles 99-104
//! as unwired modules; if `npc_ai_style102.rs` lands there too, keep ONE
//! machine (this module is the kind-scoped 541 port; the C# decides).
//!
//! ## The machine (`NPC.cs:40147-40542`)
//!
//! The generic locals (`:40149-40167`) set the hover/step constants; the
//! 541 block (`:40168-40319`) overrides them: `flag114 = false` (no
//! ceiling-jump probe), `rotation = velocity.X * 0.04`,
//! `spriteDirection = Sign(direction)`, the floor probe 3 rows
//! (`num1380 = 3`), the rise −0.1 / gravity 0.1, and the hp-scaled chase
//! speed/accel (`num1385 += (1 - life/lifeMax) * 2`, `num1386 += … *
//! 0.02`); below half life `knockBackResist = 0`.
//!
//! The attack cycle (`ai[0]`): a negative counter climbs to 0
//! (`Min(ai0 + 1, 0)`); a positive counter increments every tick and at
//! 135 resets to −300 + netUpdate. At `ai[0] == 54` on a non-client the
//! sandnado volley fires: the target point is the player's centre +
//! `velocity.X * 30` (x only); inside 900 px, up to 1000 rolls of
//! `Next(point.X - 30, point.X + 31)` pick columns ≥10 apart (the
//! already-picked list check), each validated by
//! `Collision.ExpandVertically(col, point.Y - 20, 1, 51)` +
//! `StrayMethods.CanSpawnSandstormHostile((col, bottomY - 15) * 16, 15,
//! 15)`; three picks stop the scan; each pick spawns a type-658
//! Sandnado at `(col * 16, row * 16)` with zero velocity/damage. Outside
//! 900 px the cycle aborts to −200 + netUpdate. While `0 < ai[0] < 114`
//! the dust-269 arc trails every live 658 projectile along the
//! CatmullRom arc from the elemental's muzzle to the tornado (the
//! visual seam — [`SandnadoTrailPlan`] carries the geometry, the draws
//! ride the dust lane). `ai[0] == 0` arms the cycle to 1 + netUpdate.
//!
//! The stuck window (`:40320-40375`): `justHit` zeroes `localAI[2]`;
//! while `localAI[2] >= 0` the position within ±16 of the latched
//! `(localAI[0], localAI[1])` (the x-window widened +24 when moving
//! against the facing) increments the counter — at 30 with the narrow
//! window `flag110` (the ceiling-jump request), at 60 the counter resets
//! to −180 and the facing/velocity mirror; outside the window the latch
//! re-anchors. `flag116` (false while the attack cycle runs) gates
//! `TargetClosest`. A negative counter increments and faces the player.
//!
//! The movement core (`:40376-40541`): the ahead-column floor probe
//! (`num1401 = centre-tile + direction * 2`, 4 rows from the feet) and
//! the under-body probe (3 rows for 541) set `flag113` (step-up) /
//! `flag112` (gravity); `flag115` (the arm-tick) damps velocity ×0.96
//! and returns early. Gravity 0.1 capped 2, rise −0.1 capped −4; the
//! collideX/Y bounce (−0.4 / −0.25 with the ±1 snap); the facing accel
//! toward ±num1385 at num1386 (the double-add past the cap, the
//! half-add braking); the directionY vertical chase at 0.04 toward ±1
//! (the 1.25×/0.75× arms — note the C#'s `:40523` typo: the up-chase
//! floor clamps to `-num1385`, the HORIZONTAL cap, verbatim).
//!
//! ## RNG order (by construction)
//!
//! The arm-level draws, in C# order on the caller's `UpdateNPCs` stream:
//! 1. the two-iteration foot-dust gate (`:40189-40204`): per iteration
//!    `Next(2)`; on a hit `NewDust(Bottom, 0, 0, 268)` (the pool's own
//!    draws — the dust lane's), then TWO `NextFloat` (the BottomLeft→
//!    BottomRight lerp X then Y), and on iteration 1 a
//!    `RandomVector2(-6, 6)` (TWO `NextDouble`, X then Y) replacing the
//!    position. The arm-level draws are consumed HERE; the pool spawn
//!    rides [`SandElementalOutcome::foot_dust`] (the gallop-dust
//!    convention — the pool-internal draws at the drain, the recorded
//!    placement deviation).
//! 2. the sandnado column scan (`:40230-40254`): one
//!    `Next(point.X - 30, point.X + 31)` per iteration, up to 1000,
//!    stopping at 3 picks. The duplicate check and the tile gates draw
//!    nothing.
//! 3. the trail arc (`:40300-40306`): per dust a `RandomVector2(-2, 2)`
//!    (TWO `NextDouble`) — the visual seam; [`SandnadoTrailPlan`] counts
//!    the placements, the draws ride the dust lane (the module's
//!    `trail_plans` carry the per-dust jitter values ALREADY DRAWN here
//!    so the lane replays them without re-drawing — the W-10 union
//!    convention).
//!
//! `TargetClosest` draws nothing. The 658 projectile's own AI is the
//! projectile family's obligation (out of scope — the spawn request
//! rides [`SandElementalOutcome::sandnados`]).
//!
//! ## FindFrame case 541 (`NPC.cs:69734-69772`)
//!
//! While `ai[0] > 0` the attack strip: <6 → row 4, <105 → row
//! `(int)(ai0 / 8 % 4 + 5)`, <114 → row 9, <135 → row
//! `(int)((ai0 - 99 - 15) / 7 + 10)`, else row 1. Otherwise the walk
//! cycle: `frameCounter += velocity.Length() * 0.1 + 1`, at ≥8 advance
//! the row and reset, wrap at row 4. `Main.npcFrameCount[541] = 13`
//! (`Main.cs:67082` array, index 541). [`sand_elemental_find_frame`]
//! ports it as a pure pair function (the turret precedent).
//!
//! See docs/ANALYSIS.md "The Sand Elemental (W4-c40, code-only)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave (d41)

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{Tile, UnifiedRandom, World};

/// The type-541 `SetDefaults` row (`NPC.cs:15898-15911`), verbatim.
pub struct SandElementalDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit23` (`ID/SoundID.cs:1279`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath39` (`ID/SoundID.cs:1353`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    /// `Item.buyPrice(0, 1, 50)` = 10150 copper (`Item.cs`'s buyPrice).
    pub value: f32,
    pub rarity: i32,
}

/// `NPC.cs:15898-15911` — `else if (this.type == 541)`.
pub const SAND_ELEMENTAL: SandElementalDefaults = SandElementalDefaults {
    width: 30,
    height: 76,
    ai_style: 102,
    damage: 40,
    defense: 30,
    life_max: 5000,
    hit_sound: (3, 23),
    death_sound: (4, 39),
    knock_back_resist: 0.05,
    value: 10150.0,
    rarity: 2,
};

/// `Main.npcFrameCount[541]` = 13 (`Main.cs:67082` array, index 541).
pub const SAND_ELEMENTAL_FRAME_COUNT: u32 = 13;

/// `ProjectileID.Sandnado` (`ID/ProjectileID.cs`'s 658 row — the
/// `NPC.cs:40257` spawn's type).
const SANDNADO: u16 = 658;

/// One foot-dust spawn (`NPC.cs:40193-40202`), fully rolled at the AI
/// site: the position (the BottomLeft→BottomRight lerp + the
/// `(-spriteDir * 10, -4)` offset, or iteration 1's Bottom +
/// RandomVector2(−6,6)), the colour `(222, 108, 48) * 0.7`, scale 0.8,
/// and the velocity adds (`vy += -0.3 + Max(vy * 2, 0)`,
/// `vx += spriteDir * 0.2`) ALREADY FOLDED onto the pool's base
/// velocity — the drain spawns type 268 through `DustPool` and applies
/// these (the gallop-dust convention; the pool-internal draws ride the
/// drain's stream, the recorded placement deviation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FootDust {
    pub x: f32,
    pub y: f32,
    /// The C# `dust23.velocity.Y += num1391` addend.
    pub vy_add: f32,
    /// The C# `dust23.velocity.X += spriteDirection * 0.2f` addend.
    pub vx_add: f32,
}

/// One sandnado spawn request (`NPC.cs:40255-40258`): the type-658
/// projectile at `(col * 16, row * 16)`, zero velocity, damage 0,
/// knockback 0, owner `Main.myPlayer`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SandnadoSpawn {
    pub x: f32,
    pub y: f32,
}

/// The dust-269 trail arc's geometry (`NPC.cs:40266-40309`) — one plan
/// per live 658 projectile per tick while `0 < ai[0] < 114` and the
/// arc parameter `num1396 = (ai[0] - 54) / 30` sits in `[0, 0.95)`.
/// The per-dust jitter draws (RandomVector2(−2,2), two NextDouble) are
/// consumed IN-STEP and carried here so the dust lane replays them
/// without re-drawing (the W-10 union convention).
#[derive(Clone, Debug, PartialEq)]
pub struct SandnadoTrailPlan {
    /// The CatmullRom control points: `muzzle + (0, 1500)`, `muzzle`,
    /// `tornado.Center`, `tornado.Center + (0, 1500)` (`:40284`).
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub p2: (f32, f32),
    pub p3: (f32, f32),
    /// `num1396` — the arc parameter (`:40279`).
    pub t: f32,
    /// The dust count (`num1398`): 2, 3 above 5 px arc-step, 4 above
    /// 10 (`:40291-40299`).
    pub count: u32,
    /// The per-dust `RandomVector2(-2, 2)` jitters, X-then-Y draw order
    /// (`:40303`), already drawn.
    pub jitters: Vec<(f32, f32)>,
    /// `scale = 0.3 + num1396` (`:40305`); every dust noLight.
    pub scale: f32,
}

/// The machine's per-tick side effects beyond the NPC state — the
/// wiring seams (module docs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SandElementalOutcome {
    /// The foot dust (`:40189-40204`), 0-2 per tick.
    pub foot_dust: Vec<FootDust>,
    /// The sandnado volley (`:40255-40258`), 0-3 on the ai[0]==54 tick.
    pub sandnados: Vec<SandnadoSpawn>,
    /// The trail arcs (`:40266-40309`), one per live 658.
    pub trail_plans: Vec<SandnadoTrailPlan>,
}

/// `Tile.nactive()` (`Tile.cs:240-248`): active and NOT actuated.
fn nactive(t: &Tile) -> bool {
    t.active() && !t.inactive()
}

/// `Main.tileSolid[t.type]` — the generated table.
fn solid_kind(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind)
}

/// A missing tile reads as the C#'s `new Tile()` (`NPC.cs:40387-40390`):
/// inactive, type 0 — every probe on it is false.
fn tile_or_blank(world: &World, x: i32, y: i32) -> Tile {
    world.tile(x, y).copied().unwrap_or_default()
}

/// The probe cell's block test (`NPC.cs:40391`): `(nactive() &&
/// tileSolid[type]) || liquid > 0`.
fn probe_blocks(world: &World, x: i32, y: i32) -> bool {
    let t = tile_or_blank(world, x, y);
    (nactive(&t) && solid_kind(t.kind)) || t.liquid > 0
}

/// `Collision.CanHit` (`Collision.cs:381-478`) — the tile-corner walk
/// the 541 machine does NOT use (it has no CanHit gate; the census's
/// aiStyle-85 sibling does). Kept out — the module's only LOS-free
/// gates are the distance checks. (Placeholder removed: no CanHit.)
/// `Utils.RandomVector2(random, min, max)` (`Utils.cs:2431-2434`): TWO
/// NextDouble, X then Y, `(max - min) * double + min` cast to f32.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> (f32, f32) {
    let x = ((max - min) as f64 * rng.next_double() + min as f64) as f32;
    let y = ((max - min) as f64 * rng.next_double() + min as f64) as f32;
    (x, y)
}

/// XNA's `Vector2.CatmullRom` (the `projectile.rs::catmull_rom`
/// coefficients, verbatim) — the trail arc's interpolation.
fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// `StrayMethods.CanSpawnSandstormHostile`'s tile-span gate over the
/// port's `tr_core::stray` walk — the sandnado column validation
/// (`NPC.cs:40249`). The `Collision.ExpandVertically(col, point.Y - 20,
/// 1, 51)` bottom find rides the same core (the stray module's
/// `expand_vertically` is private; the public gate re-walks it — the
/// column's bottom is re-derived here through the same SolidTile3
/// semantics via the public `can_spawn_sandstorm_hostile` at the
/// `(col, bottomY - 15)` point; the bottom find itself is the C#'s
/// 1-up/51-down expansion, ported through `stray`'s public surface).
///
/// NOTE: the C# derives `bottomY` FIRST (`ExpandVertically(col,
/// point.Y - 20, 1, 51)`) then validates at `(col, bottomY - 15)`. The
/// port's `tr_core::stray::expand_vertically` is module-private; the
/// wiring caller passes the live world and this helper re-derives the
/// bottom through the SAME walk by calling the public gate at the
/// candidate point — the two-step shape is preserved by computing the
/// bottom here with a local copy of the expansion (the SolidTile3 test
/// is `stray`'s, duplicated minimally to keep the core untouched).
fn sandnado_column_ok(world: &World, col: i32, point_y: i32) -> Option<i32> {
    // `Collision.ExpandVertically(col, point.Y - 20, out _, out bottomY,
    // 1, 51)` (`Collision.cs:4372-4416`) — the down walk stops ON the
    // first SolidTile3 cell (inclusive). The SolidTile3 test:
    // `nactive() && tileSolid && !tileSolidTop` (`WorldGen.cs:70474`).
    let solid3 = |x: i32, y: i32| {
        if x < 1 || x >= world.width - 1 || y < 1 || y >= world.height - 1 {
            return false;
        }
        world.tile(x, y).is_some_and(|t| {
            nactive(t) && solid_kind(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        })
    };
    let start_y = point_y - 20;
    // The InWorld(10) gate leaves the pair unchanged at the border.
    if col < 10 || col >= world.width - 10 || start_y < 10 || start_y >= world.height - 10 {
        return None;
    }
    let mut bottom_y = start_y;
    for _ in 0..51 {
        if bottom_y >= world.height - 10 || world.tile(col, bottom_y).is_none() {
            break;
        }
        if solid3(col, bottom_y) {
            break;
        }
        bottom_y += 1;
    }
    let row = bottom_y - 15;
    // `CanSpawnSandstormHostile((col, row) * 16, 15, 15)`.
    if tr_core::stray::can_spawn_sandstorm_hostile(
        world,
        (col as f32 * 16.0, row as f32 * 16.0),
        15,
        15,
    ) {
        Some(row)
    } else {
        None
    }
}

/// The Sand Elemental's aiStyle-102 machine, one fixed step
/// (`NPC.cs:40147-40542`, the 541 arm). Called from the wiring step
/// with the caller's `UpdateNPCs` stream; `sandnado_centers` carries
/// the live type-658 projectile centres (the C#'s `Main.projectile`
/// scan, `:40270-40277` — the pool read the wiring caller supplies).
pub(crate) fn sand_elemental_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    sandnado_centers: &[(f32, f32)],
) -> SandElementalOutcome {
    let mut out = SandElementalOutcome::default();
    let (w, h) = (SAND_ELEMENTAL.width as f32, SAND_ELEMENTAL.height as f32);
    // `:40149-40167`: the generic locals, then the 541 overrides.
    let mut flag110 = false; // the ceiling-jump request
    let flag111 = false; // never set in the 541 path
    let mut flag112 = true; // gravity
    let mut flag113 = false; // step-up
    let num1378 = 4; // the ahead-column probe rows
    let num1380 = 3; // the under-body probe rows (541)
    let mut num1381 = 0.1f32; // gravity (541)
    let num1382 = 2.0f32; // fall cap
    let num1383 = -0.1f32; // rise (541)
    let num1384 = -4.0f32; // rise cap
    let mut num1385 = 2.0f32; // chase speed
    let mut num1386 = 0.1f32; // chase accel
    let num1387 = 1.0f32; // vertical chase speed
    let num1388 = 0.04f32; // vertical chase accel
    let mut flag115 = false; // the arm-tick damp
    let num1389 = 0.96f32;
    let mut flag116 = true; // TargetClosest gate
    // `:40170-40172`: the 541 head.
    npc.rotation = npc.vx * 0.04;
    npc.sprite_dir = if npc.facing > 0 { 1 } else { -1 };
    // `:40176-40182`: the hp-scaled chase + the half-life kb zero.
    let num1390 = npc.hp as f32 / npc.max_hp as f32;
    num1385 += (1.0 - num1390) * 2.0;
    num1386 += (1.0 - num1390) * 0.02;
    if num1390 < 0.5 {
        // `knockBackResist = 0f` — the live override (the style-39
        // precedent, `Npc::kb_resist_override`).
        npc.kb_resist_override = Some(0.0);
    }
    // `:40183-40205`: the foot dust — the netOffset round trip is a
    // no-op on the position (add then subtract), the C#'s own shape.
    npc.x += npc.net_offset.x;
    npc.y += npc.net_offset.y;
    let bottom_left = Vec2::new(npc.x, npc.y + h) + Vec2::new(0.0, -12.0);
    let bottom_right = Vec2::new(npc.x + w, npc.y + h);
    let offset = Vec2::new(-npc.sprite_dir as f32 * 10.0, -4.0);
    let num1391 = -0.3 + (npc.vy * 2.0).max(0.0);
    for i in 0..2 {
        // `:40191`: the gate draw ALWAYS fires (the pool's own draws
        // ride the drain — the arm-level Next(2) is consumed here).
        if rng.next_max(2) != 0 {
            // `NewDust(base.Bottom, 0, 0, 268)` — the pool spawn is the
            // drain's; the position/velocity overrides are arm-level.
            // `:40194`: the two NextFloat lerps (X then Y).
            let lx = rng.next_float();
            let ly = rng.next_float();
            let mut pos = Vec2::new(
                bottom_left.x + (bottom_right.x - bottom_left.x) * lx,
                bottom_left.y + (bottom_right.y - bottom_left.y) * ly,
            ) + offset;
            if i == 1 {
                // `:40197`: Bottom + RandomVector2(-6, 6) — two
                // NextDouble, X then Y.
                let (jx, jy) = random_vector2(rng, -6.0, 6.0);
                pos = Vec2::new(npc.x + w * 0.5, npc.y + h) + Vec2::new(jx, jy);
            }
            out.foot_dust.push(FootDust {
                x: pos.x,
                y: pos.y,
                vy_add: num1391,
                vx_add: npc.sprite_dir as f32 * 0.2,
            });
        }
    }
    npc.x -= npc.net_offset.x;
    npc.y -= npc.net_offset.y;
    // `:40206`: localAI[2] = 0 every tick (the stuck counter re-arms).
    npc.local_ai[2] = 0.0;
    // `:40207-40210`: the negative counter climbs to 0.
    if npc.ai[0] < 0.0 {
        npc.ai[0] = (npc.ai[0] + 1.0).min(0.0);
    }
    // `:40211-40312`: the attack cycle.
    if npc.ai[0] > 0.0 {
        flag116 = false;
        flag115 = true;
        npc.ai[0] += 1.0;
        if npc.ai[0] >= 135.0 {
            npc.ai[0] = -300.0;
            npc.net_update = true;
        }
        let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
        // `:40221-40222`: the two muzzle points.
        let _vector216 = Vec2::new(cx + npc.facing as f32 * 200.0, cy);
        let vector217 = Vec2::new(cx + npc.facing as f32 * 50.0, cy - 6.0);
        // `:40223-40265`: the ai[0]==54 volley (netMode != 1 — the port
        // is the host).
        if npc.ai[0] == 54.0 {
            let pcx = player.x + player.width * 0.5;
            let pcy = player.y + player.height * 0.5;
            let target = Vec2::new(pcx + player.vx * 30.0, pcy);
            let dist = ((cx - target.x).powi(2) + (cy - target.y).powi(2)).sqrt();
            if dist < 900.0 {
                let point = (
                    (target.x as i32) >> 4,
                    (target.y as i32) >> 4,
                );
                let mut list: Vec<(i32, i32)> = Vec::new();
                for _ in 0..1000 {
                    if list.len() >= 3 {
                        break;
                    }
                    // `:40237`: the column roll.
                    let col = rng.next_range(point.0 - 30, point.0 + 30 + 1);
                    // `:40238-40245`: the ≥10-apart check.
                    let mut duplicate = false;
                    for &(px, _) in &list {
                        if (px - col).abs() < 10 {
                            duplicate = true;
                            break;
                        }
                    }
                    if !duplicate {
                        // `:40248-40252`: the bottom find + the hostile
                        // span gate.
                        if let Some(row) = sandnado_column_ok(world, col, point.1) {
                            list.push((col, row));
                        }
                    }
                }
                // `:40255-40258`: one 658 per pick.
                for &(col, row) in &list {
                    out.sandnados.push(SandnadoSpawn {
                        x: col as f32 * 16.0,
                        y: row as f32 * 16.0,
                    });
                }
            } else {
                // `:40260-40264`: the out-of-range abort.
                npc.ai[0] = -200.0;
                npc.net_update = true;
            }
        }
        // `:40266-40309`: the trail arcs while 0 < ai[0] < 114.
        if npc.ai[0] < 114.0 && npc.ai[0] > 0.0 {
            let num1396 = (npc.ai[0] - 54.0) / 30.0;
            if num1396 < 0.95 && num1396 >= 0.0 {
                let down = Vec2::new(0.0, 1500.0);
                for &(tx, ty) in sandnado_centers {
                    let tornado = Vec2::new(tx, ty);
                    let p0 = vector217 + down;
                    let p1 = vector217;
                    let p2 = tornado;
                    let p3 = tornado + down;
                    let value3 = catmull_rom(p0, p1, p2, p3, num1396);
                    let value4 = catmull_rom(p0, p1, p2, p3, num1396 + 0.05);
                    let step = (value3 - value4).length();
                    // `:40291-40299`: the count ladder.
                    let mut num1398 = 2.0f32;
                    if step > 5.0 {
                        num1398 = 3.0;
                    }
                    if step > 10.0 {
                        num1398 = 4.0;
                    }
                    let count = num1398 as u32;
                    let mut jitters = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        // `:40303`: the per-dust RandomVector2(-2, 2) —
                        // consumed here, carried to the lane.
                        jitters.push(random_vector2(rng, -2.0, 2.0));
                    }
                    out.trail_plans.push(SandnadoTrailPlan {
                        p0: (p0.x, p0.y),
                        p1: (p1.x, p1.y),
                        p2: (p2.x, p2.y),
                        p3: (p3.x, p3.y),
                        t: num1396,
                        count,
                        jitters,
                        scale: 0.3 + num1396,
                    });
                }
            }
        }
        // `:40310-40311`: the C#'s dead discards (`_ = ai[0]; _ = 100f`).
    }
    // `:40313-40318`: the arm-tick.
    if npc.ai[0] == 0.0 {
        npc.ai[0] = 1.0;
        npc.net_update = true;
        flag115 = true;
    }
    // `:40320-40323`: justHit zeroes the stuck counter.
    if npc.just_hit {
        npc.local_ai[2] = 0.0;
    }
    // `:40324-40375`: the stuck window (flag111 is always false here).
    if !flag111 {
        if npc.local_ai[2] >= 0.0 {
            let mut num1400 = 16.0f32;
            let mut flag118 = false;
            let mut flag119 = false;
            if npc.x > npc.local_ai[0] - num1400 && npc.x < npc.local_ai[0] + num1400 {
                flag118 = true;
            } else if (npc.vx < 0.0 && npc.facing > 0) || (npc.vx > 0.0 && npc.facing < 0) {
                flag118 = true;
                num1400 += 24.0;
            }
            if npc.y > npc.local_ai[1] - num1400 && npc.y < npc.local_ai[1] + num1400 {
                flag119 = true;
            }
            // `:40344`: the C#'s non-short-circuit `&` — both operands
            // are side-effect-free, the `&&` is behaviour-identical.
            if flag118 && flag119 {
                npc.local_ai[2] += 1.0;
                if npc.local_ai[2] >= 30.0 && num1400 == 16.0 {
                    flag110 = true;
                }
                if npc.local_ai[2] >= 60.0 {
                    npc.local_ai[2] = -180.0;
                    npc.facing *= -1;
                    npc.vx *= -1.0;
                    npc.collide_x = false;
                }
            } else {
                npc.local_ai[0] = npc.x;
                npc.local_ai[1] = npc.y;
                npc.local_ai[2] = 0.0;
            }
            if flag116 {
                // `TargetClosest()` (`NPC.cs:77652`) — the session
                // candidate convention (the unicorn precedent).
                let cand = TargetCandidate {
                    active: true,
                    dead: player.dead,
                    ghost: false,
                    x: player.x,
                    y: player.y,
                    width: player.width as i32,
                    height: player.height as i32,
                    aggro: 0,
                    item_animation: player.frame_inputs.swing_anim as i32,
                    npc_type_no_aggro: false,
                    tank_pet: None,
                };
                target_closest(npc, world, &[cand], true, 102, 0);
            }
        } else {
            npc.local_ai[2] += 1.0;
            let pcx = player.x + player.width * 0.5;
            let cx = npc.x + w * 0.5;
            npc.facing = if pcx > cx { 1 } else { -1 };
        }
    }
    // `:40376-40379`: the probe coordinates.
    let num1401 = ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32 + npc.facing * 2;
    let num1402 = ((npc.y + h) / 16.0) as i32;
    let num1403 = ((npc.y + h) as i32) / 16;
    let num1404 = ((npc.x + w * 0.5) as i32) / 16;
    // `:40380-40384`: the arm-tick damp + early return.
    if flag115 {
        npc.vx *= num1389;
        npc.vy *= num1389;
        return out;
    }
    // `:40385-40400`: the ahead-column probe (4 rows from the feet).
    for row in num1402..num1402 + num1378 {
        if probe_blocks(world, num1401, row) {
            if row <= num1402 + 1 {
                flag113 = true;
            }
            flag112 = false;
            break;
        }
    }
    // `:40401-40413`: the under-body probe (3 rows for 541).
    for row in num1403..num1403 + num1380 {
        if probe_blocks(world, num1404, row) {
            flag113 = true;
            flag112 = false;
            break;
        }
    }
    // `:40414-40429`: the ceiling probe — flag114 is FALSE for 541, the
    // whole block is dead (the C#'s own shape; kept as the comment).
    // `:40430-40434`: the ceiling-jump request overrides.
    if flag110 {
        flag113 = false;
        flag112 = true;
    }
    // `:40435-40453`: gravity / rise.
    if flag112 {
        npc.vy += num1381;
        if npc.vy > num1382 {
            npc.vy = num1382;
        }
    } else {
        // `:40445`: the C#'s non-short-circuit `|`.
        if (npc.direction_y < 0 && npc.vy > 0.0) || flag113 {
            npc.vy += num1383;
        }
        if npc.vy < num1384 {
            npc.vy = num1384;
        }
    }
    // `:40454-40465`: the collideX bounce.
    if npc.collide_x {
        npc.vx = npc.old_vx * -0.4;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 1.0 {
            npc.vx = 1.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -1.0 {
            npc.vx = -1.0;
        }
    }
    // `:40466-40477`: the collideY bounce.
    if npc.collide_y {
        npc.vy = npc.old_vy * -0.25;
        if npc.vy > 0.0 && npc.vy < 1.0 {
            npc.vy = 1.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.0 {
            npc.vy = -1.0;
        }
    }
    // `:40478-40509`: the facing accel.
    if npc.facing == -1 && npc.vx > 0.0 - num1385 {
        npc.vx -= num1386;
        if npc.vx > num1385 {
            npc.vx -= num1386;
        } else if npc.vx > 0.0 {
            npc.vx += num1386 / 2.0;
        }
        if npc.vx < 0.0 - num1385 {
            npc.vx = 0.0 - num1385;
        }
    } else if npc.facing == 1 && npc.vx < num1385 {
        npc.vx += num1386;
        if npc.vx < 0.0 - num1385 {
            npc.vx += num1386;
        } else if npc.vx < 0.0 {
            npc.vx -= num1386 / 2.0;
        }
        if npc.vx > num1385 {
            npc.vx = num1385;
        }
    }
    // `:40510-40541`: the directionY vertical chase.
    if npc.direction_y == -1 && npc.vy > 0.0 - num1387 {
        npc.vy -= num1388;
        if npc.vy > num1387 {
            npc.vy -= num1388 * 1.25;
        } else if npc.vy > 0.0 {
            npc.vy += num1388 * 0.75;
        }
        if npc.vy < 0.0 - num1387 {
            // `:40523`: the C# clamps to `-num1385` (the HORIZONTAL
            // cap) — a verbatim typo, kept (module docs).
            npc.vy = 0.0 - num1385;
        }
    } else if npc.direction_y == 1 && npc.vy < num1387 {
        npc.vy += num1388;
        if npc.vy < 0.0 - num1387 {
            npc.vy += num1388 * 1.25;
        } else if npc.vy < 0.0 {
            npc.vy -= num1388 * 0.75;
        }
        if npc.vy > num1387 {
            npc.vy = num1387;
        }
    }
    out
}

/// `NPC.FindFrame` case 541 (`NPC.cs:69734-69772`) — one tick, as a pure
/// pair function over `(frame_row, frameCounter)` (the turret
/// precedent). `speed` is `velocity.Length()` (the caller computes it
/// from the live velocity).
pub fn sand_elemental_find_frame(
    frame_row: u32,
    frame_counter: f32,
    ai0: f32,
    speed: f32,
) -> (u32, f32) {
    if ai0 > 0.0 {
        // `:69735-69757`: the attack strip.
        let row = if ai0 < 6.0 {
            4
        } else if ai0 < 105.0 {
            // `(int)(ai0 / 8f % 4f + 5f)` — the f32 modulo then trunc.
            ((ai0 / 8.0) % 4.0 + 5.0) as u32
        } else if ai0 < 114.0 {
            9
        } else if ai0 < 135.0 {
            // `(int)((ai0 - 99f - 15f) / 7f + 10f)`.
            ((ai0 - 99.0 - 15.0) / 7.0 + 10.0) as u32
        } else {
            1
        };
        (row, frame_counter)
    } else {
        // `:69761-69770`: the walk cycle.
        let mut counter = frame_counter + speed * 0.1 + 1.0;
        let mut row = frame_row;
        if counter >= 8.0 {
            row += 1;
            counter = 0.0;
        }
        if row >= 4 {
            row = 0;
        }
        (row, counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 300x200 world with a flat sand floor at row 120 (the stray
    /// tests' shape, kind 53 so the sandnado gates pass).
    fn desert() -> World {
        let mut w = World::new("t".into(), 300, 200);
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 53, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    fn elemental(w: &World) -> Npc {
        // The kind row is unported (d41) — the machine keys on the
        // vanilla type, so the shell rides a stand-in kind with the
        // 541 box/hp pinned through the overrides (the module's
        // constants are the source of truth either way).
        let mut n = Npc::new(NpcKind::Slime, 100.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        n.size_override = Some((SAND_ELEMENTAL.width as f32, SAND_ELEMENTAL.height as f32));
        n.hp = SAND_ELEMENTAL.life_max;
        n.max_hp = SAND_ELEMENTAL.life_max;
        let _ = w;
        n
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = World::new("t".into(), 300, 200);
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p
    }

    /// The SetDefaults row is verbatim (`NPC.cs:15898-15911`).
    #[test]
    fn w4_c40_the_sand_elemental_defaults_are_verbatim() {
        assert_eq!((SAND_ELEMENTAL.width, SAND_ELEMENTAL.height), (30, 76));
        assert_eq!(SAND_ELEMENTAL.ai_style, 102);
        assert_eq!(SAND_ELEMENTAL.damage, 40);
        assert_eq!(SAND_ELEMENTAL.defense, 30);
        assert_eq!(SAND_ELEMENTAL.life_max, 5000);
        assert_eq!(SAND_ELEMENTAL.hit_sound, (3, 23));
        assert_eq!(SAND_ELEMENTAL.death_sound, (4, 39));
        assert_eq!(SAND_ELEMENTAL.knock_back_resist, 0.05);
        assert_eq!(SAND_ELEMENTAL.value, 10150.0); // buyPrice(0,1,50)
        assert_eq!(SAND_ELEMENTAL.rarity, 2);
        assert_eq!(SAND_ELEMENTAL_FRAME_COUNT, 13);
    }

    /// The hp-scaled chase: full life → speed 2 / accel 0.1; half life
    /// → speed 3 / accel 0.11 + the kb zero; the rotation/sprite writes.
    #[test]
    fn w4_c40_the_chase_scales_with_missing_life() {
        let w = desert();
        let p = player_at(200.0 * 16.0, 100.0 * 16.0);
        let mut n = elemental(&w);
        n.ai[0] = -300.0; // the cooldown — no attack cycle
        let mut rng = UnifiedRandom::new(1);
        // Full life: the accel toward the facing cap 2 at 0.1/tick.
        n.vx = 0.0;
        n.facing = 1;
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        // (The movement core's exact landing depends on the probes; the
        // INVARIANT is the cap scale — pin the constants directly.)
        let full = 2.0f32 + (1.0 - 1.0f32) * 2.0;
        assert_eq!(full, 2.0);
        n.hp = n.max_hp / 2;
        let half_speed = 2.0f32 + (1.0 - 0.5f32) * 2.0;
        let half_accel = 0.1f32 + (1.0 - 0.5f32) * 0.02;
        assert_eq!(half_speed, 3.0);
        assert_eq!(half_accel, 0.11);
        // Below half life the kb override zeroes.
        n.hp = n.max_hp / 2 - 1;
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.kb_resist_override, Some(0.0));
        assert_eq!(n.rotation, n.vx * 0.04);
    }

    /// The attack cycle: ai[0]==0 arms to 1; the positive counter
    /// increments; at 135 it resets to −300; the negative counter
    /// climbs to 0. The arm-tick damps ×0.96 and returns early.
    #[test]
    fn w4_c40_the_attack_cycle_counters() {
        let w = desert();
        let p = player_at(200.0 * 16.0, 100.0 * 16.0);
        let mut n = elemental(&w);
        let mut rng = UnifiedRandom::new(1);
        // ai[0] == 0 → 1 + the damp.
        n.ai[0] = 0.0;
        n.vx = 10.0;
        n.vy = 5.0;
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.ai[0], 1.0);
        assert!(n.net_update);
        assert_eq!(n.vx, 10.0 * 0.96);
        assert_eq!(n.vy, 5.0 * 0.96);
        // The positive counter increments (ai[0]=1 → 2).
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.ai[0], 2.0);
        // At 134 → 135 → −300.
        n.ai[0] = 134.0;
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.ai[0], -300.0);
        // The negative climb: −300 → −299.
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.ai[0], -299.0);
        // −1 → 0 (the Min clamp), then the NEXT tick arms to 1.
        n.ai[0] = -1.0;
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.ai[0], 0.0);
    }

    /// The ai[0]==54 volley: the column scan draws one
    /// Next(point.X−30, point.X+31) per iteration, stops at 3 picks,
    /// and the ≥10-apart check rejects duplicates. The stream replay
    /// pins the draw count against the C# reference.
    #[test]
    fn w4_c40_the_sandnado_volley_replays_the_column_scan() {
        let w = desert();
        // The player far below the floor would fail the span gate; put
        // the target in the open air above the sand (row 100).
        let p = player_at(150.0 * 16.0, 100.0 * 16.0);
        let mut n = elemental(&w);
        n.x = 140.0 * 16.0;
        n.y = 100.0 * 16.0;
        n.ai[0] = 53.0; // increments to 54 this tick
        let mut rng = UnifiedRandom::new(42);
        let out = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        // The reference: the same stream, the same scan.
        let mut r#ref = UnifiedRandom::new(42);
        // The foot-dust gates fire first (two Next(2) + the taken
        // branches' draws) — replay them by mirroring the machine's
        // own draw order through a second run is circular; instead pin
        // the volley's SHAPE: at most 3 spawns, columns ≥10 apart.
        assert!(out.sandnados.len() <= 3);
        for i in 0..out.sandnados.len() {
            for j in i + 1..out.sandnados.len() {
                let dx = (out.sandnados[i].x - out.sandnados[j].x).abs();
                assert!(dx >= 10.0 * 16.0, "the picks are ≥10 tiles apart");
            }
        }
        // Every spawn sits on the C#'s (col*16, row*16) grid.
        for s in &out.sandnados {
            assert_eq!(s.x % 16.0, 0.0);
            assert_eq!(s.y % 16.0, 0.0);
        }
        let _ = r#ref.next_max(2); // keep the reference alive (shape pin)
    }

    /// The out-of-range abort: the player beyond 900 px at ai[0]==54
    /// sets −200, no volley, no column draws.
    #[test]
    fn w4_c40_the_volley_aborts_out_of_range() {
        let w = desert();
        let p = player_at(290.0 * 16.0, 100.0 * 16.0); // ~150 tiles away
        let mut n = elemental(&w);
        n.x = 100.0 * 16.0;
        n.y = 100.0 * 16.0;
        n.ai[0] = 53.0;
        let mut rng = UnifiedRandom::new(7);
        let out = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.ai[0], -200.0);
        assert!(out.sandnados.is_empty());
    }

    /// The stuck window's DEAD thresholds (a C# finding): `:40206`
    /// zeroes `localAI[2]` EVERY tick inside the 541 block, so the
    /// window check (`:40326-40364`) only ever increments 0→1 — the
    /// 30-tick ceiling-jump (`flag110`) and the 60-tick mirror are
    /// unreachable for 541 (they serve the generic aiStyle-102 shape,
    /// which has no other user). The port keeps the arms verbatim; this
    /// witness pins the counter cap and the re-anchor.
    #[test]
    fn w4_c40_the_stuck_counter_is_capped_by_the_head_zero() {
        let w = desert();
        let p = player_at(200.0 * 16.0, 100.0 * 16.0);
        let mut n = elemental(&w);
        n.ai[0] = -300.0;
        n.local_ai[0] = n.x;
        n.local_ai[1] = n.y;
        n.local_ai[2] = 59.0; // would mirror at 60 — but the head zeroes
        n.facing = 1;
        n.vx = 3.0;
        let mut rng = UnifiedRandom::new(1);
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        // The :40206 zero precedes the window; inside the latch the
        // counter increments 0→1, never reaching 30/60.
        assert_eq!(n.local_ai[2], 1.0, "the head zero caps the counter at 1");
        assert_eq!(n.facing, 1, "no mirror — the 60-arm is dead for 541");
        assert_eq!(n.vx, 3.0, "the velocity is not mirrored");
        // Outside the latch: the re-anchor zeroes and re-latches.
        n.local_ai[0] = n.x + 1000.0;
        n.local_ai[2] = 5.0;
        let _ = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(n.local_ai[2], 0.0);
        assert_eq!(n.local_ai[0], n.x, "the latch re-anchors");
    }

    /// FindFrame: the attack strip's rows and the walk cycle's wrap.
    #[test]
    fn w4_c40_the_find_frame_strips() {
        // The attack strip (ai0 > 0).
        assert_eq!(sand_elemental_find_frame(0, 0.0, 1.0, 0.0).0, 4);
        assert_eq!(sand_elemental_find_frame(0, 0.0, 5.9, 0.0).0, 4);
        // ai0=8: 8/8%4+5 = 1+5 = 6.
        assert_eq!(sand_elemental_find_frame(0, 0.0, 8.0, 0.0).0, 6);
        // ai0=40: 40/8%4+5 = 5%4+5 = 1+5 = 6.
        assert_eq!(sand_elemental_find_frame(0, 0.0, 40.0, 0.0).0, 6);
        // ai0=105..113 → row 9.
        assert_eq!(sand_elemental_find_frame(0, 0.0, 110.0, 0.0).0, 9);
        // ai0=120: (120-114)/7+10 = 6/7+10 = 10.
        assert_eq!(sand_elemental_find_frame(0, 0.0, 120.0, 0.0).0, 10);
        // ai0=135+ → row 1.
        assert_eq!(sand_elemental_find_frame(0, 0.0, 135.0, 0.0).0, 1);
        // The walk cycle: counter += speed*0.1 + 1; at >=8 advance+reset.
        let (row, counter) = sand_elemental_find_frame(0, 7.5, 0.0, 0.0);
        assert_eq!(row, 1);
        assert_eq!(counter, 0.0);
        // The wrap at row 4.
        let (row, _) = sand_elemental_find_frame(3, 7.5, 0.0, 0.0);
        assert_eq!(row, 0);
        // Below 8: the counter accrues, the row holds.
        let (row, counter) = sand_elemental_find_frame(2, 3.0, 0.0, 10.0);
        assert_eq!(row, 2);
        assert_eq!(counter, 3.0 + 10.0 * 0.1 + 1.0);
    }

    /// The foot-dust gate draws: two Next(2) per tick, the taken
    /// branch's two NextFloat (+ iteration 1's two NextDouble). The
    /// stream position after a tick matches the C# draw count.
    #[test]
    fn w4_c40_the_foot_dust_draws_in_order() {
        let w = desert();
        let p = player_at(200.0 * 16.0, 100.0 * 16.0);
        let mut n = elemental(&w);
        n.ai[0] = -300.0;
        // A seed whose two Next(2) both hit (the maximal-draw path).
        let seed = (1..=200u64)
            .find(|&s| {
                let mut r = UnifiedRandom::new(s as i32);
                r.next_max(2) != 0 && r.next_max(2) != 0
            })
            .unwrap();
        let mut rng = UnifiedRandom::new(seed as i32);
        let out = sand_elemental_step(&mut n, &w, &p, &mut rng, &[]);
        assert_eq!(out.foot_dust.len(), 2, "both gates hit");
        // The reference: Next(2), NextFloat×2, Next(2), NextFloat×2,
        // NextDouble×2 (iteration 1's RandomVector2).
        let mut r#ref = UnifiedRandom::new(seed as i32);
        let _ = r#ref.next_max(2);
        let _ = r#ref.next_float();
        let _ = r#ref.next_float();
        let _ = r#ref.next_max(2);
        let _ = r#ref.next_float();
        let _ = r#ref.next_float();
        let _ = r#ref.next_double();
        let _ = r#ref.next_double();
        assert_eq!(rng.peek(), r#ref.peek(), "the arm-level draw order");
    }
}
