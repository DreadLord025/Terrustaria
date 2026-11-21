//! W4-e2 — the NPC.cs `Dust.NewDust` sites' SECOND batch (the first ~40
//! are W4-d52's: the Eye of Cthulhu, the aiStyle-8 casters, the golem
//! body/fists and the sphere family). This module carries the shared
//! emitters for the sites whose machines live INLINE in `npc.rs`
//! (`update_skeletron`, `update_brain`, `update_flying_dutchman`,
//! `update_stardust_cell`, `update_moon_leech_blob`, `update_golem_rng`'s
//! free head) plus the twins' gate/burst pair shared by
//! `npc_ai_style30`/`npc_ai_style31`.
//!
//! Contract (the S1-E-1 pool, `tr_core::particles`, d50's DustPool):
//! every draw rides the caller's `UpdateNPCs` stream in C# statement
//! order — the explicit `Main.rand` argument draws FIRST (C# evaluates
//! arguments before the call), then the `NewDust` internals roll inside
//! `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn writes
//! (which read `Main.dust[6000]` when the pool refuses — the sentinel
//! slot exists and takes the writes exactly like the C#). The
//! `position ± netOffset` round trips net to zero in the single-player
//! port (the field is never written) and fold away with this note.
//! Gore siblings stay W4-d55's lane: where a gore call sits between
//! dust draws its ARGUMENT draws are consumed here in order (the
//! `NewGore` body rolls are d55's — the same split d52's spin hand-off
//! used), and the site comments cite it.
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id):
//! - 22285/22292 — Skeletron's outside-spin blood trail (`:22283-22297`).
//! - 26462/26755/26760 — Retinazer gate/burst/tick (`npc_ai_style30`).
//! - 27076/27369/27374 — Spazmatism gate/burst/tick (`npc_ai_style31`).
//! - 31576/31579 — the free Golem head's two 228 emitters
//!   (`:31572-31585`, `npc_ai_golem`).
//! - 32416 — the Brain's retreat-entry 20-dust burst (`:32406-32418`).
//! - 36741/36750 — the Moon Leech Clot's 6+3 ring (`:36738-36755`).
//! - 38958 — the Flying Dutchman's 2-iteration 228 trail
//!   (`:38953-38966`).
//! - 39421 — the Stardust Cell's growth-ring emitter (`:39407-39428`).
//!
//! Batch 5 (the line-order continuation after batch 4's 39362 end, 20
//! sites — d52's first-40 lane ends at 21562, f4's third batch starts
//! at 44037):
//! - 39524/39559/39571 — the Nebula Floater's three 20×242 teleport
//!   bursts (`:39522-39528`/`:39557-39563`/`:39569-39575`, the
//!   `update_nebula_floater` stand-in — the machine's teleport structure
//!   is the documented deviation, the dust bodies are verbatim at the
//!   stand-in's teleport points).
//! - 39963/39974 — the Solar Goop 519's 13-dust launch burst + the
//!   3×(Next(3)<2) trail (`:39958-39989`, STAGED — no `NpcKind` yet).
//! - 40024/40035 — the Ancient Light 522's 13×261 launch + the
//!   2×(Next(10-min(7,|v|))<1) trail (`:40017-40049`, STAGED).
//! - 40100/40110/40120 — the Ancient Doom 523's three Next(6) ring gates
//!   (`:40097-40126`, aiStyle 101, STAGED). W4-e2 batch 6 attribution fix:
//!   batch 5 mislabeled these "Squidhead 521" — aiStyle 101's only user is
//!   type 523 (`SetDefaults` `:15799`), the cultist's Ancient Doom; the
//!   Squidhead 521 is aiStyle 86 (`:15772`) and its own sites are
//!   37337/37464 below.
//! - 40193/40302 — the Sand Elemental 541's bottom sand pair + the
//!   CatmullRom tether (`:40189-40204`/`:40266-40309`, STAGED).
//! - 40834/40851/40947 — the Eternia Crystal's win-ring / win-pulse /
//!   loss-ring `NewDustPerfect` sites (`:40828-40858`/`:40941-40952`,
//!   `update_eternia_crystal` — the win/loss scene arms are faithful).
//! - 41367/41396 — the Moon Lord core's crash-drama fan + the
//!   per-iteration smoke (`:41362-41403`, `update_moon_lord_core`).
//! - 42222 — the Moon Lord head's deathray socket fan (`:42212-42227`,
//!   `update_moon_lord_head`).
//! - 42675/42873 — the True Eye's orbit spoke pair + the deathray
//!   socket fan (`:42673-42678`/`:42863-42878`, `update_true_eye`).
//!
//! Batch 6 (the :42873-44037 window's unclaimed tail — f4's third wave
//! owns 44037+, the window holds ONLY the King Slime's three sites; the
//! :21562-39039 gap census + the floater fade-entry follow in the next
//! commits, see research/285, renumbered from 64 per registry-150):
//! - 43251/43278/43365 — the King Slime's shrink 10×4 ring / grow-back
//!   10×4 ring / every-tick body 4 (`:43249-43254`/`:43276-43281`/
//!   `:43365-43367`, `update_king_slime` — WIRED, the live DustPool).
//! - 39524 — the Nebula Floater's fade-entry 20×242 burst
//!   (`:39522-39528`, the faithful aiStyle-97 machine's `localAI[3]`
//!   one-shot in the `ai[1]==1` fade-out — WIRED, `update_nebula_floater`;
//!   the stand-in's invented `ai[1]--` countdown + scatter teleport pick
//!   are retired for the C#'s `AI_AttemptToFindTeleportSpot` (range 20,
//!   telefrag 12, centred, in-air) + the strike-hook latch `:81347`).
//! - 24520 — the Antlion 69's aiStyle-19 fall-drip (`:24517-24529`,
//!   STAGED — no `NpcKind` yet).
//! - 24850/24866 — the Pixie 75's type-55 + the Ice Elemental 169's
//!   type-92 aiStyle-22 ambient drips (`:24848-24852`/`:24864-24869`,
//!   STAGED — no `NpcKind` yet).
//! - 35157/35162 — the Martian Turret 387's materialize ring pair
//!   (`:35133-35166`, the `npc_ai_style73` plan seam's 8 placements —
//!   STAGED, the style-73 machine is unwired).
//! - 37289/37307 — the Shadowflame Apparition 472's aiStyle-86 spawn-in
//!   20×27 ring + the fade drip (`:37279-37297`/`:37302-37314`, STAGED).
//! - 37337/37464 — the Squidhead 521's aiStyle-86 spawn-in 2×228 ring +
//!   the every-tick 228 (`:37334-37341`/`:37462-37467`, STAGED).
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled
//! by the merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// Sites 22285/22292 — Skeletron's blood trail (`NPC.cs:22283-22297`),
/// AFTER the state match and its velocity writes, BEFORE the facing
/// latch: `ai[1] != 2 && ai[1] != 3 && type != 68 && (num168 != 0 ||
/// !expertMode)` — `num168` is the hands-alive count (`:21965`'s scan;
/// the port's `world.skeletron_hands_alive`), expert rides the caller's
/// flag. The C# carries NO rand draws outside `NewDust` here (the
/// velocities are fixed expressions) — only the pool internals roll.
pub fn skeletron_trail(
    x: f32,
    y: f32,
    width: i32,
    height: f32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:22285` — the foot puff: box (30, 10) at
    // (x + width/2 - 15 - vx*5, y + height - 2), type 5, speeds
    // ((0 - vx)*0.2, 3), scale 2. `width / 2` is INTEGER division.
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(x + (width / 2) as f32 - 15.0 - vx * 5.0, y + height - 2.0),
        30,
        10,
        5,
        (0.0 - vx) * 0.2,
        3.0,
        0,
        Color::default(),
        2.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:22286`
    d.velocity.x *= 1.3; // `:22287`
    d.velocity.x += vx * 0.4; // `:22288`
    d.velocity.y += 2.0 + vy; // `:22289`
    // `:22290-22296` — two box dusts at (x, y + 120), (width, 60),
    // speeds (vx, vy), scale 2; noGravity, velocity -= (vx, vy), then
    // velocity.Y += 5.
    for _ in 0..2 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x, y + 120.0), width, 60, 5, vx, vy, 0,
            Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity -= Vec2::new(vx, vy);
        d.velocity.y += 5.0;
    }
}

/// Sites 26462/27076 — the twins' ambient drip (`NPC.cs:26460-26465` /
/// `:27074-27079`, identical bodies): the `Next(5) == 0` gate EVERY
/// tick (the caller consumes it before the day gate), then one type-5
/// `NewDust` at `(x, y + height*0.25)`, box `(width, (int)(height*0.5))`,
/// speeds `(vx, 2f)`; post-spawn `velocity.X *= 0.5`, `velocity.Y *= 0.1`.
/// `gate` is the already-drawn `Next(5) == 0` result — the gate draw
/// stays at the machine's C# position even on the dust-free entries.
pub fn twin_drip(
    gate: bool,
    x: f32,
    y: f32,
    width: i32,
    height: f32,
    vx: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !gate {
        return;
    }
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(x, y + height * 0.25),
        width,
        (height * 0.5) as i32,
        5,
        vx,
        2.0,
        0,
        Color::default(),
        1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity.x *= 0.5;
    d.velocity.y *= 0.1;
}

/// Sites 26755/27369 — the twins' spin hand-off burst
/// (`NPC.cs:26746-26757` / `:27360-27371`): 2 × (3 gores × 2 draws
/// `Next(-30, 31) * 0.2`) then 20 type-5 dusts × the same pair. The
/// machine keeps the six gore ARGUMENT pairs at its C# position (the
/// `NewGore` bodies are d55's); this fn draws each dust's velocity pair
/// then rolls the spawn — 20 × (2 argument draws + internals), the
/// sounds (3, 15) stay the machine's queue.
pub fn twin_spin_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..20 {
        let dx = rng.next_range(-30, 31) as f32 * 0.2;
        let dy = rng.next_range(-30, 31) as f32 * 0.2;
        dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 5, dx, dy, 0,
            Color::default(), 1.0,
        );
    }
}

/// Sites 26760/27374 — the twins' per-tick spin dust
/// (`NPC.cs:26760` / `:27374`): ONE type-5 box dust every spin tick,
/// also on the hand-off tick AFTER the burst, before the 0.98 damp.
pub fn twin_spin_tick(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dx = rng.next_range(-30, 31) as f32 * 0.2;
    let dy = rng.next_range(-30, 31) as f32 * 0.2;
    dust.new_dust(
        rng, ctx, world, Vec2::new(x, y), width, height, 5, dx, dy, 0,
        Color::default(), 1.0,
    );
}

/// Site 32416 — the Brain of Cthulhu's retreat-entry burst
/// (`NPC.cs:32406-32418`): inside the `localAI[2]` latch, AFTER the
/// `PlaySound(3)` and the four gore calls (392/393/394/395 — their
/// eight ARGUMENT draws are consumed here in order, the `NewGore`
/// bodies are d55's), 20 type-5 dusts × `Next(-30, 31) * 0.2` pairs,
/// then the case-15 roar queues (the caller's order).
pub fn brain_retreat_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:32410-32413` — the four gores' argument pairs (d55 bodies).
    for _ in 0..4 {
        let _gx = rng.next_range(-30, 31) as f32 * 0.2;
        let _gy = rng.next_range(-30, 31) as f32 * 0.2;
    }
    // `:32414-32417` — the 20-dust loop (site 32416).
    for _ in 0..20 {
        let dx = rng.next_range(-30, 31) as f32 * 0.2;
        let dy = rng.next_range(-30, 31) as f32 * 0.2;
        dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 5, dx, dy, 0,
            Color::default(), 1.0,
        );
    }
}

/// Site 38958 — the Flying Dutchman's smoke trail
/// (`NPC.cs:38953-38966`): two iterations, each a `Next(2) != 0` gate;
/// on a pass the offset draws `(NextFloat() - 0.5) * (width - 70)` and
/// `(NextFloat() - 0.5) * 20 + height/2 + 10` (INTEGER `height / 2`)
/// rotate by `rotation`, then ONE type-228 `NewDust(Center, 0, 0)`
/// whose post-writes pin position = Center + offset, velocity zero,
/// noGravity + noLight, fadeIn 1.5, scale 0.5. Runs AFTER the
/// `rotation = velocity.X * 0.025` write (`:38951`), before the arm's
/// end. `RotatedBy` is the double-precision cos/sin form
/// (`Utils.cs:1675-1684`, the pillar port's convention).
pub fn dutchman_trail(
    cx: f32,
    cy: f32,
    width: i32,
    height: i32,
    rotation: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..2 {
        if rng.next_max(2) == 0 {
            continue;
        }
        let ox = (rng.next_float() - 0.5) * (width - 70) as f32;
        let oy = (rng.next_float() - 0.5) * 20.0 + (height / 2) as f32 + 10.0;
        let r = rotation as f64;
        let (cos, sin) = (r.cos() as f32, r.sin() as f32);
        let rx = ox * cos - oy * sin;
        let ry = ox * sin + oy * cos;
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx, cy), 0, 0, 228, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position = Vec2::new(cx + rx, cy + ry);
        d.velocity = Vec2::ZERO;
        d.no_gravity = true;
        d.no_light = true;
        d.fade_in = 1.5;
        d.scale = 0.5;
    }
}

/// Site 39421 — the Stardust Cell (402)'s growth ring
/// (`NPC.cs:39407-39428`): `num1319 = (int)(ai[0] / (300/2f))` (the
/// style's `num1318 = 300`), then `num1319 + 1` iterations, each a
/// `Next(2) != 0` gate; on a pass the type is 226 with scale 0.4
/// (odd iterations 0.65 — the C# writes 226 in both arms), the ring
/// point is `Center + (NextDouble() * 2π).ToRotationVector2() *
/// (12 - num1319*2)`, the spawn box is (24, 24) at point - (12, 12)
/// with speeds (vx/2, vy/2); post-writes: position -= (2, 2),
/// velocity = Normalize(Center - point) * 1.5 * (10 - num1319*2) / 10,
/// noGravity, scale. `customData = this` drops with the field (the
/// S1-E-1 record note). Runs at the arm's END — after the 300-tick
/// Transform early-return (`:39394-39401`), so a transform tick draws
/// nothing. `Vector2.Normalize` of a zero vector stays zero (XNA).
pub fn stardust_cell_ring(
    cx: f32,
    cy: f32,
    vx: f32,
    vy: f32,
    ai0: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num1319 = (ai0 / (300.0 / 2.0)) as i32;
    for i in 0..num1319 + 1 {
        if rng.next_max(2) == 0 {
            continue;
        }
        let scale = if i % 2 == 1 { 0.65 } else { 0.4 };
        let ang = rng.next_double() * (std::f32::consts::PI as f64 * 2.0);
        let radius = 12.0 - (num1319 * 2) as f32;
        let px = cx + ang.cos() as f32 * radius;
        let py = cy + ang.sin() as f32 * radius;
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(px - 12.0, py - 12.0), 24, 24, 226,
            vx / 2.0, vy / 2.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position -= Vec2::new(2.0, 2.0);
        let (mut nx, mut ny) = (cx - px, cy - py);
        let len = (nx * nx + ny * ny).sqrt();
        if len > 0.0 {
            nx /= len;
            ny /= len;
        }
        let k = 1.5 * (10.0 - (num1319 * 2) as f32) / 10.0;
        d.velocity = Vec2::new(nx * k, ny * k);
        d.no_gravity = true;
        d.scale = scale;
    }
}

/// Sites 36741/36750 — the Moon Leech Clot (401)'s attached rings
/// (`NPC.cs:36738-36755`): six type-229 dusts on the
/// `-UnitY * height/2` spinner at `i * 2π/6` (velocity = -UnitY,
/// noGravity, scale 0.7), then three type-229 dusts on the
/// `-UnitY * height/6` spinner at `i * 2π/6` (speeds (0, -2),
/// noGravity, scale 1.5). Both spawn boxes are 0×0 at
/// `Center - (4, 4) + spun`. `customData = this` drops with the field.
/// The C#'s lerp-to-the-leech-anchor (`:36733-36737`) needs the
/// sibling/proj pools — the port's stand-in keeps the blob at its own
/// position (the `update_moon_leech_blob` deviation note); the rings
/// read that Centre. NO rand draws outside the pool internals.
pub fn moon_leech_rings(
    cx: f32,
    cy: f32,
    height: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let step = std::f32::consts::PI * 2.0 / 6.0;
    for i in 0..6 {
        // (0, -h/2).RotatedBy(i*step) — the double cos/sin form.
        let a = (i as f32 * step) as f64;
        let (cos, sin) = (a.cos() as f32, a.sin() as f32);
        let (sx, sy) = ((-height / 2.0) * -sin, (-height / 2.0) * cos);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - 4.0 + sx, cy - 4.0 + sy), 0, 0, 229,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity = Vec2::new(0.0, -1.0);
        d.no_gravity = true;
        d.scale = 0.7;
    }
    for i in 0..3 {
        let a = (i as f32 * step) as f64;
        let (cos, sin) = (a.cos() as f32, a.sin() as f32);
        let (sx, sy) = ((-height / 6.0) * -sin, (-height / 6.0) * cos);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - 4.0 + sx, cy - 4.0 + sy), 0, 0, 229,
            0.0, -2.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.5;
    }
}

// =====================================================================
// W4-e2 batch 5 — the line-order continuation (20 sites after 39362).
// The STAGED emitters below carry the kinds the port has no `NpcKind`
// for yet (SolarGoop 519, AncientLight 522, AncientDoom 523, SandElemental
// 541); their machines arrive with the wiring wave, the dust bodies +
// RNG order are pinned here (the `npc_ai_style9` solar-flare-helper
// precedent). The faithful machines' sites are armed inline in npc.rs.
// =====================================================================

/// `Vector2.UnitY.RotatedByRandom(max)` (`Utils.cs:1697-1700` +
/// `:1675-1684`): TWO `NextDouble` draws (+max then −max), then the
/// double cos/sin rotation of (0, 1) → (−sin, cos). The pillar port's
/// convention, lifted here for the batch-5 emitters.
fn rotated_unit_y_random(rng: &mut UnifiedRandom, max_radians: f64) -> (f32, f32) {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    (-(angle.sin() as f32), angle.cos() as f32)
}

/// `Utils.RandomVector2(random, min, max)` (`Utils.cs:2431-2434`):
/// `(max - min) * (float)NextDouble() + min` per axis, X then Y.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `Vector2.Normalize` (XNA): a zero vector stays zero; otherwise the
/// f32 sqrt of the f32 squared-sum divides both axes (the port's
/// `stardust_cell_ring` convention — NOT the double-sqrt `Distance`).
fn normalize(v: Vec2) -> Vec2 {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len > 0.0 {
        v / len
    } else {
        Vec2::ZERO
    }
}

/// Sites 39963/39974 — the Solar Goop (519)'s launch burst + trail
/// (`NPC.cs:39958-39989`, aiStyle 99). The `localAI[0] == 0` one-shot
/// fires 13 type-6 dusts at (position, width, height) with speeds
/// (vx*0.5, vy*0.5), alpha 90, scale 2.5; post-writes noGravity, fadeIn
/// 1, velocity *= 4, noLight. Then EVERY tick 3 iterations of a
/// `Next(3) < 2` gate; on a pass one type-6 dust (same spawn args),
/// post-writes noGravity, velocity *= 0.2, fadeIn 1, then a `Next(6)`
/// draw: on 0 velocity *= 30 + noGravity=false + noLight, else velocity
/// = DirectionFrom(dust.position) * velocity.Length() (the NPC-centre
/// outward normal × the speed). STAGED — no `NpcKind::SolarGoop` yet.
pub fn solar_goop_launch_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..13 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 6,
            vx * 0.5, vy * 0.5, 90, Color::default(), 2.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.0;
        d.velocity *= 4.0;
        d.no_light = true;
    }
}

/// Site 39974 — the Solar Goop's per-tick 3-iteration trail (the
/// `for (num1366 = 0; num1366 < 3; ...)` loop, `:39970-39989`). The
/// caller runs this EVERY tick after the launch burst (the C# loop is
/// outside the `localAI[0]` latch). `cx/cy` = the NPC centre for the
/// `DirectionFrom` outward normal.
pub fn solar_goop_trail(
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..3 {
        if rng.next_max(3) < 2 {
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(x, y), width, height, 6,
                vx * 0.5, vy * 0.5, 90, Color::default(), 2.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.2;
            d.fade_in = 1.0;
            if rng.next_max(6) == 0 {
                d.velocity *= 30.0;
                d.no_gravity = false;
                d.no_light = true;
            } else {
                // DirectionFrom(dust.position) = Normalize(Center - pos).
                let speed = (d.velocity.x * d.velocity.x + d.velocity.y * d.velocity.y).sqrt();
                d.velocity = normalize(Vec2::new(cx - d.position.x, cy - d.position.y)) * speed;
            }
        }
    }
}

/// Sites 40024/40035 — the Ancient Light (522)'s launch burst + trail
/// (`NPC.cs:40017-40049`, aiStyle 100). The `localAI[0] == 0` one-shot
/// writes velocity = (ai[2], ai[3]) THEN fires 13 type-261 dusts (same
/// shape as the goop's: alpha 90, scale 2.5, noGravity/fadeIn 1/×4/
/// noLight). Then EVERY tick 2 iterations of a
/// `Next(10 - (int)Math.Min(7f, velocity.Length())) < 1` gate; on a pass
/// one type-261 dust, post-writes noGravity, velocity *= 0.2, fadeIn
/// 0.4, then a `Next(6)` draw: on 0 velocity *= 5 + noLight, else the
/// DirectionFrom outward normal × speed. STAGED.
pub fn ancient_light_launch_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..13 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 261,
            vx * 0.5, vy * 0.5, 90, Color::default(), 2.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.0;
        d.velocity *= 4.0;
        d.no_light = true;
    }
}

/// Site 40035 — the Ancient Light's per-tick 2-iteration trail
/// (`:40031-40049`). The gate bound is `10 - (int)min(7, |velocity|)`
/// (the C# `Math.Min(7f, base.velocity.Length())` — the f32 length,
/// truncated to int). `cx/cy` = the NPC centre.
pub fn ancient_light_trail(
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vlen = (vx * vx + vy * vy).sqrt();
    let bound = 10 - (vlen.min(7.0) as i32);
    for _ in 0..2 {
        if rng.next_max(bound) < 1 {
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(x, y), width, height, 261,
                vx * 0.5, vy * 0.5, 90, Color::default(), 2.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.2;
            d.fade_in = 0.4;
            if rng.next_max(6) == 0 {
                d.velocity *= 5.0;
                d.no_light = true;
            } else {
                let speed = (d.velocity.x * d.velocity.x + d.velocity.y * d.velocity.y).sqrt();
                d.velocity = normalize(Vec2::new(cx - d.position.x, cy - d.position.y)) * speed;
            }
        }
    }
}

/// Sites 40100/40110/40120 — the Ancient Doom (523)'s three ring gates
/// (`NPC.cs:40097-40126`, aiStyle 101 — `SetDefaults` `:15799` gives 523
/// the style; the Squidhead 521 is aiStyle 86, `:15772`, and owns
/// 37337/37464 instead). Three SEQUENTIAL `Next(6) == 0`
/// gates (each draws even when the previous passed). On a pass:
/// gate 1 — vector210 = UnitY.RotatedByRandom(2π), NewDust(Center -
/// vector210*20, 0, 0, 27), post: noGravity, position = Center -
/// vector210 * Next(10,21) * scale, velocity = vector210.RotatedBy(π/2)
/// * 4, scale = 0.5 + NextFloat, fadeIn 0.5. Gate 2 — vector211 =
/// UnitY.RotatedByRandom(2π), NewDust(Center - vector211*30, 0, 0, 240),
/// post: noGravity, position = Center - vector211*20*scale, velocity =
/// vector211.RotatedBy(-π/2) * 2, scale = 0.5 + NextFloat, fadeIn 0.5.
/// Gate 3 — vector212 = UnitY.RotatedByRandom(2π), NewDust(Center -
/// vector212*30, 0, 0, 240), post: position = Center - vector212*20*
/// scale, velocity = Zero, scale = 0.5 + NextFloat, fadeIn 0.5, noLight
/// (NO noGravity). STAGED.
pub fn ancient_doom_rings(
    cx: f32,
    cy: f32,
    scale: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // Gate 1 (:40097-40106) — the 27 ring.
    if rng.next_max(6) == 0 {
        let (rx, ry) = rotated_unit_y_random(rng, 6.2831854820251465);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - rx * 20.0, cy - ry * 20.0), 0, 0, 27,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        // `Center - vector210 * Next(10,21) * scale` — left-to-right:
        // (v * Next) * scale, NOT v * (Next * scale).
        let n = rng.next_range(10, 21) as f32;
        d.position = Vec2::new(cx - (rx * n) * scale, cy - (ry * n) * scale);
        // vector210.RotatedBy(1.5707963705062866) — the DOUBLE cos/sin
        // form (Utils.cs:1675-1684); the literal is the f32 π/2 widened,
        // so cos ≈ -4.37e-8, NOT exactly 0 (the (-y, x) shortcut drops
        // that term).
        let r = 1.5707963705062866f64;
        let (c, s) = (r.cos() as f32, r.sin() as f32);
        d.velocity = Vec2::new(rx * c - ry * s, rx * s + ry * c) * 4.0;
        d.scale = 0.5 + rng.next_float();
        d.fade_in = 0.5;
    }
    // Gate 2 (:40107-40116) — the 240 ring, -π/2 rotation.
    if rng.next_max(6) == 0 {
        let (rx, ry) = rotated_unit_y_random(rng, 6.2831854820251465);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - rx * 30.0, cy - ry * 30.0), 0, 0, 240,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.position = Vec2::new(cx - (rx * 20.0) * scale, cy - (ry * 20.0) * scale);
        // RotatedBy(-1.5707963705062866) — the double cos/sin form.
        let r = -1.5707963705062866f64;
        let (c, s) = (r.cos() as f32, r.sin() as f32);
        d.velocity = Vec2::new(rx * c - ry * s, rx * s + ry * c) * 2.0;
        d.scale = 0.5 + rng.next_float();
        d.fade_in = 0.5;
    }
    // Gate 3 (:40117-40126) — the 240 still ring (NO noGravity).
    if rng.next_max(6) == 0 {
        let (rx, ry) = rotated_unit_y_random(rng, 6.2831854820251465);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - rx * 30.0, cy - ry * 30.0), 0, 0, 240,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position = Vec2::new(cx - (rx * 20.0) * scale, cy - (ry * 20.0) * scale);
        d.velocity = Vec2::ZERO;
        d.scale = 0.5 + rng.next_float();
        d.fade_in = 0.5;
        d.no_light = true;
    }
}

/// Site 40193 — the Sand Elemental (541)'s bottom sand pair
/// (`NPC.cs:40189-40204`, aiStyle 102). Two iterations of a
/// `Next(2) != 0` gate; on a pass one type-268 NewDust(Bottom, 0, 0),
/// post: position = Lerp(BottomLeft+(0,-12), BottomRight, NextFloat per
/// axis) + (-spriteDirection*10, -4); iteration 1 overrides position =
/// Bottom + RandomVector2(-6, 6); color = (222,108,48)*0.7, scale 0.8,
/// velocity.Y += -0.3 + max(vy*2, 0), velocity.X += spriteDirection*0.2.
/// The `netOffset` round-trips to zero (single-player). STAGED.
pub fn sand_elemental_bottom_sand(
    bottom_x: f32,
    bottom_y: f32,
    bottom_left_x: f32,
    bottom_right_x: f32,
    sprite_direction: i32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `new Color(222, 108, 48) * 0.7f` — XNA's `(int)(channel*scale)`
    // truncation (the `xna_scale_color` convention).
    let c = crate::render::special_tiles::xna_scale_color([222, 108, 48, 255], 0.7);
    let color = Color::new(c[0], c[1], c[2], c[3]);
    let num1391 = -0.3 + (vy * 2.0).max(0.0);
    for num1392 in 0..2 {
        if rng.next_max(2) != 0 {
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(bottom_x, bottom_y), 0, 0, 268,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            let tx = rng.next_float();
            let ty = rng.next_float();
            let lx = bottom_left_x + (bottom_right_x - bottom_left_x) * tx;
            let ly = (bottom_y - 12.0) + (bottom_y - (bottom_y - 12.0)) * ty;
            d.position = Vec2::new(
                lx + (-sprite_direction as f32 * 10.0),
                ly + (-4.0),
            );
            if num1392 == 1 {
                let rv = random_vector2(rng, -6.0, 6.0);
                d.position = Vec2::new(bottom_x + rv.x, bottom_y + rv.y);
            }
            d.color = color;
            d.scale = 0.8;
            d.velocity.y += num1391;
            d.velocity.x += sprite_direction as f32 * 0.2;
        }
    }
}

/// Site 40302 — the Sand Elemental's CatmullRom tether
/// (`NPC.cs:40266-40309`, aiStyle 102). Runs while `0 < ai[0] < 114`
/// and `num1396 = (ai[0]-54)/30` is in `[0, 0.95)`. For each owned 658
/// projectile centre `item3`: value3/value4 = CatmullRom(anchor+(0,1500),
/// anchor, item3, item3+(0,1500), num1396 / num1396+0.05); the dust count
/// is 2 (3 if |value3-value4|>5, 4 if >10); per dust one type-269
/// NewDust(anchor, 0, 0), post: position = Lerp(value3, value4, i/count)
/// + RandomVector2(-2, 2), noLight, scale = 0.3 + num1396. `anchor` =
/// Center + UnitX*direction*50 - UnitY*6 (vector217). STAGED — the
/// caller supplies the 658 centres (the projectile pool walk).
pub fn sand_elemental_tether(
    anchor: Vec2,
    ai0: f32,
    proj_centers: &[Vec2],
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num1396 = (ai0 - 54.0) / 30.0;
    if !(num1396 < 0.95 && num1396 >= 0.0) {
        return;
    }
    let off = Vec2::new(0.0, 1500.0);
    for &item3 in proj_centers {
        let value3 = catmull_rom(anchor + off, anchor, item3, item3 + off, num1396);
        let value4 = catmull_rom(anchor + off, anchor, item3, item3 + off, num1396 + 0.05);
        let dist = ((value3.x - value4.x) * (value3.x - value4.x)
            + (value3.y - value4.y) * (value3.y - value4.y))
            .sqrt();
        let mut count = 2.0f32;
        if dist > 5.0 {
            count = 3.0;
        }
        if dist > 10.0 {
            count = 4.0;
        }
        let mut i = 0.0f32;
        while i < count {
            let idx = dust.new_dust(
                rng, ctx, world, anchor, 0, 0, 269,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            let t = i / count;
            let lerp = value3 + (value4 - value3) * t;
            let rv = random_vector2(rng, -2.0, 2.0);
            d.position = lerp + rv;
            d.no_light = true;
            d.scale = 0.3 + num1396;
            i += 1.0;
        }
    }
}

/// `Vector2.CatmullRom` (XNA): the standard centripetal-free cubic
/// Hermite through p1..p4 at t — `0.5 * ((2*p2) + (-p1+p3)*t +
/// (2*p1-5*p2+4*p3-p4)*t² + (-p1+3*p2-3*p3+p4)*t³)`, per axis in f32.
fn catmull_rom(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    Vec2::new(
        0.5 * ((2.0 * p2.x) + (-p1.x + p3.x) * t
            + (2.0 * p1.x - 5.0 * p2.x + 4.0 * p3.x - p4.x) * t2
            + (-p1.x + 3.0 * p2.x - 3.0 * p3.x + p4.x) * t3),
        0.5 * ((2.0 * p2.y) + (-p1.y + p3.y) * t
            + (2.0 * p1.y - 5.0 * p2.y + 4.0 * p3.y - p4.y) * t2
            + (-p1.y + 3.0 * p2.y - 3.0 * p3.y + p4.y) * t3),
    )
}

/// Sites 39524/39559/39571 — the Nebula Floater (420)'s three 20×242
/// teleport bursts (`NPC.cs:39522-39528`/`:39557-39563`/`:39569-39575`,
/// aiStyle 97). All three are the IDENTICAL body: 20 type-242
/// NewDust(position, width, height) with default speeds/alpha/colour/
/// scale; post-writes velocity *= 3, noGravity, scale 2.5. The C# fires
/// them at the `localAI[3]` one-shot (the ai[1]==1 fade entry), the
/// `ai[2]!=0 && ai[3]!=0` teleport departure (OLD position) and arrival
/// (NEW position). W4-e2 batch 6: `update_nebula_floater` is now the
/// FAITHFUL aiStyle-97 machine (the stand-in's invented `ai[1]--`
/// countdown + scatter pick retired for `AI_AttemptToFindTeleportSpot`),
/// so all three sites are armed at their C# positions — the fade-entry
/// 39524 in the fade-out's one-shot, the departure/arrival 39559/39571
/// bracketing the Centre placement.
pub fn nebula_floater_teleport_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..20 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 242,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 3.0;
        d.no_gravity = true;
        d.scale = 2.5;
    }
}

// =====================================================================
// W4-e2 batch 6 — the King Slime's teleport rings + body drip (3 sites).
// The :42873-44037 window's only unclaimed sites (f4's third wave owns
// 44037+). WIRED inline in `npc.rs::update_king_slime` on the live
// DustPool / UpdateNPCs stream.
// =====================================================================

/// Sites 43251/43278 — the King Slime's shrink / grow-back teleport rings
/// (`NPC.cs:43249-43254` / `:43276-43281`, AI_015). IDENTICAL bodies: 10
/// type-4 `NewDust(position + UnitX*-20, width+40, height, 4, vx, vy,
/// 150, Color(78,136,255,80), 2)`; post-writes noGravity then
/// `velocity *= vel_mult` (0.5 shrink / 2 grow). The shrink loop is gated
/// `!flag3` (the teleport tick itself spawns none); the grow loop runs
/// every tick of state 6. No explicit `Main.rand` draws outside the pool.
pub fn king_slime_teleport_ring(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    vel_mult: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let color = Color::new(78, 136, 255, 80);
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x - 20.0, y), width + 40, height, 4,
            vx, vy, 150, color, 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= vel_mult;
    }
}

/// Site 43365 — the King Slime's every-tick body drip (`NPC.cs:43365-
/// 43367`, AI_015): ONE type-4 `NewDust(position, width, height, 4, vx,
/// vy, 255, Color(0,80,255,80), scale*1.2)`; post noGravity, velocity *=
/// 0.5. Runs AFTER the movement block, BEFORE the `life <= 0` return and
/// the hp-scale write — so it reads the PRE-resize box and scale.
pub fn king_slime_body_drip(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    scale: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let color = Color::new(0, 80, 255, 80);
    let idx = dust.new_dust(
        rng, ctx, world, Vec2::new(x, y), width, height, 4,
        vx, vy, 255, color, scale * 1.2,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true;
    d.velocity *= 0.5;
}

// =====================================================================
// W4-e2 batch 6 — the :21562-39039 gap's unclaimed sites (9). d52's
// first-40 lane ends at 21562; e2's pillar batch starts at 39007. The
// gap's kinds are either unported (Antlion 69, Pixie 75, Ice Elemental
// 169, Shadowflame Apparition 472, Squidhead 521 — no NpcKind) or their
// machines are unwired (Martian Turret 387, npc_ai_style73's plan seam).
// STAGED: the dust bodies + RNG order are pinned here; the machines
// arrive with the wiring wave (the npc_ai_style9 solar-flare precedent).
// =====================================================================

/// Site 24520 — the Antlion (69)'s aiStyle-19 fall-drip (`NPC.cs:24517-
/// 24529`). The caller's machine runs the `Next(2) == 0` gate EVERY tick
/// the body is airborne (the `flag29` ground check returned false); on a
/// pass ONE type-32 `NewDust((x-4, y+height-8), width+8, 24, 32, 0,
/// vy/2)`; post `velocity.X *= 0.4`, `velocity.Y *= -1`, then a SECOND
/// `Next(2) == 0` gate adds noGravity + scale += 0.2. `gate`/`boost` are
/// the already-drawn results — the gate draws stay at the machine's C#
/// position even on the dust-free entries. STAGED.
pub fn antlion_fall_drip(
    gate: bool,
    boost: bool,
    x: f32,
    y: f32,
    width: i32,
    height: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !gate {
        return;
    }
    let idx = dust.new_dust(
        rng, ctx, world, Vec2::new(x - 4.0, y + height - 8.0), width + 8, 24, 32,
        0.0, vy / 2.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity.x *= 0.4;
    d.velocity.y *= -1.0;
    if boost {
        d.no_gravity = true;
        d.scale += 0.2;
    }
}

/// Site 24850 — the Pixie (75)'s aiStyle-22 ambient drip (`NPC.cs:24848-
/// 24852`): the `Next(6) == 0` gate, then ONE type-55 `NewDust(position,
/// width, height, 55, 0, 0, 200, this.color)`; post `velocity *= 0.3`.
/// `gate` is the already-drawn result; `color` is the NPC's tint (the
/// Pixie's `SetDefaults` color). STAGED.
pub fn pixie_ambient_drip(
    gate: bool,
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    color: Color,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !gate {
        return;
    }
    let idx = dust.new_dust(
        rng, ctx, world, Vec2::new(x, y), width, height, 55,
        0.0, 0.0, 200, color, 1.0,
    );
    dust.slot_mut(idx).velocity *= 0.3;
}

/// Site 24866 — the Ice Elemental (169)'s aiStyle-22 ambient drip
/// (`NPC.cs:24864-24869`): the `Next(3) == 0` gate, then ONE type-92
/// `NewDust(position, width, height, 92, 0, 0, 200)`; post `velocity *=
/// 0.3`, noGravity. The `:24862` centre AddLight + `alpha = 30` are the
/// visual lane's (zero RNG). `gate` is the already-drawn result. STAGED.
pub fn ice_elemental_ambient_drip(
    gate: bool,
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !gate {
        return;
    }
    let idx = dust.new_dust(
        rng, ctx, world, Vec2::new(x, y), width, height, 92,
        0.0, 0.0, 200, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity *= 0.3;
    d.no_gravity = true;
}

/// Sites 35157/35162 — the Martian Turret (387)'s materialize ring
/// (`NPC.cs:35133-35166`, aiStyle 73). The `npc_ai_style73` plan seam
/// (`turret_materialize_dust_plan`) computes the 8 placements: for each
/// of the four offsets ((0,-15)×0.15, (0,-5)×0.3, (0,5)×0.6, (0,20)×0.45)
/// a MIRRORED pair — first at `Center + spin*scale + offset`, then
/// `spin *= -1` and the twin. Every dust is type-226 `NewDust(Center, 0,
/// 0, 226, 0, 0, 100, default, 0.5)`; post noGravity, position = the
/// placement, velocity = Zero. The plan's `placements` array is in C#
/// order (the pair interleaving). STAGED — the style-73 machine is
/// unwired (the kind arm is a stand-in hold).
pub fn turret_materialize_ring(
    cx: f32,
    cy: f32,
    placements: &[(f32, f32); 8],
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for &(px, py) in placements.iter() {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx, cy), 0, 0, 226,
            0.0, 0.0, 100, Color::default(), 0.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.position = Vec2::new(px, py);
        d.velocity = Vec2::ZERO;
    }
}

/// Sites 37289/37307 — the Shadowflame Apparition (472)'s aiStyle-86
/// spawn-in ring + fade drip (`NPC.cs:37279-37314`). The `localAI[0]==0`
/// one-shot fires the 20×27 ring: per dust the position is `Center -
/// (0,18) + normalize(Next(-100,101), Next(-100,101)) * (Next(0,100)*0.1)`,
/// the velocity is `-normalize(that offset) * (Next(50,90)*0.2) * 0.3`;
/// `NewDust(center31, 1, 1, 27)`, post velocity = the computed value,
/// alpha 100, then `Next(2)==0` adds noGravity + scale += 0.3. The fade
/// drip (37307) runs while `localAI[0] < 120`: the count is the FLOAT
/// `(1 - localAI[0]/120) * 20` (the C#'s `(float)num1205 < num1204`
/// loop), each gated `Next(5)==0`, ONE type-27 `NewDust(position, width,
/// height, 27)`; post alpha 100, velocity *= 0.3, velocity += base.velocity
/// * 0.75, noGravity. STAGED — no NpcKind yet.
pub fn shadowflame_apparition_spawn_ring(
    cx: f32,
    cy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..20 {
        // `center31 = Center - (0, 18)`.
        let (mut px, mut py) = (cx, cy - 18.0);
        // `vector167 = (Next(-100,101), Next(-100,101))`, normalize, then
        // `*= Next(0,100) * 0.1` — the position offset.
        let ox = rng.next_range(-100, 101) as f32;
        let oy = rng.next_range(-100, 101) as f32;
        let mut v = normalize(Vec2::new(ox, oy));
        let r = rng.next_range(0, 100) as f32 * 0.1;
        v *= r;
        px += v.x;
        py += v.y;
        // Re-normalize the offset, then `*= Next(50,90) * 0.2` — the
        // velocity magnitude (the C# re-normalizes vector167 AFTER the
        // position add, so the direction is the unit offset again).
        let mut vel = normalize(v);
        let speed = rng.next_range(50, 90) as f32 * 0.2;
        vel *= speed;
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(px, py), 1, 1, 27,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity = vel * -0.3;
        d.alpha = 100;
        if rng.next_max(2) == 0 {
            d.no_gravity = true;
            d.scale += 0.3;
        }
    }
}

/// Site 37307 — the Shadowflame Apparition's fade drip (the second half
/// of the 37289/37307 pair). `count` is the FLOAT bound `(1 -
/// localAI[0]/120) * 20` AFTER the increment; the loop runs while
/// `(float)i < count`. Each iteration gates `Next(5)==0`; on a pass ONE
/// type-27 box dust, post alpha 100, velocity *= 0.3, velocity +=
/// (vx,vy)*0.75, noGravity. `gates` are the already-drawn Next(5) results
/// (one per iteration, in order). STAGED.
pub fn shadowflame_apparition_fade_drip(
    count: f32,
    gates: &[bool],
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let mut i = 0.0f32;
    let mut g = 0;
    while i < count {
        if gates.get(g).copied().unwrap_or(false) {
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(x, y), width, height, 27,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.alpha = 100;
            d.velocity *= 0.3;
            d.velocity += Vec2::new(vx, vy) * 0.75;
            d.no_gravity = true;
        }
        g += 1;
        i += 1.0;
    }
}

/// Sites 37337/37464 — the Squidhead (521)'s aiStyle-86 spawn-in 2×228
/// ring + the every-tick 228 (`NPC.cs:37334-37341`/`:37462-37467`). The
/// spawn-in (localAI[0] < 120) fires 2 iterations of `NewDust(position -
/// (10,10), width+20, height+20, 228, 0, 0, 100, default, 2)`; post
/// noGravity + noLight. The every-tick site (37464) is the SAME body but
/// ONE iteration (the C#'s `for num1224 < 1`). `iterations` is 2 for the
/// spawn-in, 1 for the every-tick. STAGED — no NpcKind yet.
pub fn squidhead_228_ring(
    iterations: i32,
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..iterations {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x - 10.0, y - 10.0), width + 20, height + 20, 228,
            0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("e2".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget —
    /// the pool never culls (the `update_rng` scratch convention).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    /// W4-e2 — sites 22285/22292: three dusts, fixed velocities, no
    /// explicit draws. The foot puff's post-chain lands
    /// vx' = ((-vx*0.2) + jitter)*1.3 + vx*0.4 — re-derived against a
    /// mirrored stream through the same pool call.
    #[test]
    fn w4_e2_skeletron_trail_spawns_three_and_chains_the_foot_puff() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = UnifiedRandom::new(11);
        let mut mpool = DustPool::new();
        skeletron_trail(1000.0, 900.0, 100, 100.0, 2.0, -1.0, &mut rng, &mut pool, &c, &w);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w,
            Vec2::new(1000.0 + 50.0 - 15.0 - 10.0, 900.0 + 100.0 - 2.0),
            30, 10, 5, -0.4, 3.0, 0, Color::default(), 2.0,
        );
        let md = mpool.slot_mut(idx);
        md.no_gravity = true;
        md.velocity.x *= 1.3;
        md.velocity.x += 0.8;
        md.velocity.y += 1.0;
        assert_eq!(active(&pool).len(), 3, "1 foot puff + 2 box dusts");
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(pool.slot(0).scale, 2.0);
        // The box pair: velocity -= (vx, vy) then +5 Y.
        for i in 1..3 {
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).scale, 2.0);
        }
    }

    /// W4-e2 — sites 26462/27076: a failed gate draws NOTHING (the
    /// machine already consumed the Next(5)); a passed gate spawns one
    /// type-5 with the quarter-height anchor and the damp chain.
    #[test]
    fn w4_e2_twin_drip_gates_and_damps() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        twin_drip(false, 500.0, 500.0, 100, 110.0, 4.0, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty(), "the failed gate spawns nothing");
        twin_drip(true, 500.0, 500.0, 100, 110.0, 4.0, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 1);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 5);
        assert!(d.position.y >= 527.5 && d.position.y < 527.5 + 55.0);
        // vx' = (4 + jitter) * 0.5 — the jitter is in [-1.2, 1.2]
        // (width 100 box, Dust.cs:158-163), so vx' in [1.4, 2.6].
        assert!(d.velocity.x >= 1.4 && d.velocity.x <= 2.6);
    }

    /// W4-e2 — sites 26755/27369 + 26760/27374: the burst is 20 dusts,
    /// the tick one; both consume the argument pair BEFORE the pool
    /// internals — a mirrored stream through new_dust with the same
    /// pre-draws lands identical records.
    #[test]
    fn w4_e2_twin_burst_and_tick_mirror_the_argument_order() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(21);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(21);
        twin_spin_burst(300.0, 300.0, 100, 110, &mut rng, &mut pool, &c, &w);
        twin_spin_tick(300.0, 300.0, 100, 110, &mut rng, &mut pool, &c, &w);
        for _ in 0..21 {
            let dx = mirror.next_range(-30, 31) as f32 * 0.2;
            let dy = mirror.next_range(-30, 31) as f32 * 0.2;
            mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(300.0, 300.0), 100, 110, 5,
                dx, dy, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(active(&pool).len(), 21);
        for i in 0..21 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
    }

    /// W4-e2 — site 32416: the four gore argument pairs (8 draws) run
    /// BEFORE the 20 dusts; the stream after the burst equals a mirror
    /// that drew 8 + 20×2 range pairs plus the same pool internals.
    #[test]
    fn w4_e2_brain_burst_consumes_the_gore_argument_pairs_first() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(31);
        brain_retreat_burst(700.0, 700.0, 176, 216, &mut rng, &mut pool, &c, &w);
        for _ in 0..4 {
            let _gx = mirror.next_range(-30, 31);
            let _gy = mirror.next_range(-30, 31);
        }
        for _ in 0..20 {
            let dx = mirror.next_range(-30, 31) as f32 * 0.2;
            let dy = mirror.next_range(-30, 31) as f32 * 0.2;
            mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(700.0, 700.0), 176, 216, 5,
                dx, dy, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(active(&pool).len(), 20);
        for i in 0..20 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 — site 38958: each passed gate draws exactly two floats
    /// then the 228 internals; the post-writes pin the rotated offset,
    /// zero velocity, noLight, fadeIn 1.5, scale 0.5.
    #[test]
    fn w4_e2_dutchman_trail_pins_the_rotated_offset_and_shape() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        let mut mirror = UnifiedRandom::new(41);
        dutchman_trail(1600.0, 800.0, 350, 120, 0.3, &mut rng, &mut pool, &c, &w);
        // Re-derive the gates/offsets on the mirror; every accepted
        // dust must match the pinned shape.
        let mut expect = 0usize;
        for _ in 0..2 {
            if mirror.next_max(2) == 0 {
                continue;
            }
            let ox = (mirror.next_float() - 0.5) * 280.0;
            let oy = (mirror.next_float() - 0.5) * 20.0 + 60.0 + 10.0;
            let r = 0.3f64;
            let rx = ox * r.cos() as f32 - oy * r.sin() as f32;
            let ry = ox * r.sin() as f32 + oy * r.cos() as f32;
            // The pool internals roll after the two offset floats; the
            // pinned post-writes overwrite position/velocity, so the
            // record is deterministic given the gate/offset draws.
            let d = pool.slot(expect);
            assert_eq!(d.dust_type, 228);
            assert!((d.position.x - (1600.0 + rx)).abs() < 1e-3);
            assert!((d.position.y - (800.0 + ry)).abs() < 1e-3);
            assert_eq!(d.velocity, Vec2::ZERO);
            assert!(d.no_gravity && d.no_light);
            assert_eq!(d.fade_in, 1.5);
            assert_eq!(d.scale, 0.5);
            expect += 1;
        }
        assert_eq!(active(&pool).len(), expect);
    }

    /// W4-e2 — site 39421: at ai[0] = 200, num1319 = 1 → two iterations
    /// (scales 0.4 / 0.65 on passes), radius 10, velocity factor
    /// 1.5 * 8 / 10 = 1.2 toward the centre; the odd/even scale rows
    /// and the -2 position nudge pin.
    #[test]
    fn w4_e2_stardust_ring_scales_radius_and_inward_velocity() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(51);
        stardust_cell_ring(1000.0, 1000.0, 1.0, -1.0, 200.0, &mut rng, &mut pool, &c, &w);
        let act = active(&pool);
        assert!(act.len() <= 2, "num1319+1 = 2 iterations");
        for (k, &i) in act.iter().enumerate() {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 226);
            assert_eq!(d.scale, if k % 2 == 1 { 0.65 } else { 0.4 });
            assert!(d.no_gravity);
            // |velocity| = 1.2 (the normalized inward vector).
            let v = d.velocity.length();
            assert!((v - 1.2).abs() < 1e-4, "inward speed 1.2, got {v}");
        }
    }

    /// W4-e2 — sites 36741/36750: nine dusts (6 + 3), all type 229,
    /// noGravity; the six-ring scales 0.7 with velocity (0, -1), the
    /// three-ring scales 1.5. Zero explicit draws — the stream advances
    /// by the pool internals only (mirrored).
    #[test]
    fn w4_e2_moon_leech_rings_spawn_six_plus_three() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(61);
        moon_leech_rings(900.0, 900.0, 34.0, &mut rng, &mut pool, &c, &w);
        let step = std::f32::consts::PI * 2.0 / 6.0;
        for i in 0..6 {
            let a = (i as f32 * step) as f64;
            let (sx, sy) = (-17.0 * -(a.sin() as f32), -17.0 * a.cos() as f32);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(896.0 + sx, 896.0 + sy), 0, 0,
                229, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.velocity = Vec2::new(0.0, -1.0);
            d.no_gravity = true;
            d.scale = 0.7;
        }
        for i in 0..3 {
            let a = (i as f32 * step) as f64;
            let (sx, sy) = (-(34.0 / 6.0) * -(a.sin() as f32), -(34.0 / 6.0) * a.cos() as f32);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(896.0 + sx, 896.0 + sy), 0, 0,
                229, 0.0, -2.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
        }
        assert_eq!(active(&pool).len(), 9);
        for i in 0..9 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    // =================================================================
    // W4-e2 batch 5 witnesses — WRITTEN, NOT RUN (code-only mode).
    // =================================================================

    /// Sites 39963/39974 — the goop launch burst is 13 type-6 dusts with
    /// the ×4 velocity chain; the trail's 3 gates each draw Next(3) then
    /// (on a pass) the pool internals + Next(6). The mirror replays the
    /// burst + trail through the same pool calls.
    #[test]
    fn w4_e2_solar_goop_burst_thirteen_and_trail_gates() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        let mut mirror = UnifiedRandom::new(71);
        solar_goop_launch_burst(500.0, 500.0, 30, 30, 2.0, 3.0, &mut rng, &mut pool, &c, &w);
        for _ in 0..13 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(500.0, 500.0), 30, 30, 6, 1.0, 1.5, 90, Color::default(), 2.5);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.fade_in = 1.0;
            d.velocity *= 4.0;
            d.no_light = true;
        }
        assert_eq!(active(&pool).len(), 13);
        for i in 0..13 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert!(pool.slot(i).no_gravity && pool.slot(i).no_light);
            assert_eq!(pool.slot(i).fade_in, 1.0);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The trail: 3 gates, each Next(3) then (on <2) the internals +
        // Next(6). Replay against a fresh mirror.
        let mut pool2 = DustPool::new();
        let mut mpool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(72);
        let mut mirror2 = UnifiedRandom::new(72);
        solar_goop_trail(500.0, 500.0, 515.0, 515.0, 30, 30, 2.0, 3.0, &mut rng2, &mut pool2, &c, &w);
        for _ in 0..3 {
            if mirror2.next_max(3) < 2 {
                let idx = mpool2.new_dust(&mut mirror2, &c, &w, Vec2::new(500.0, 500.0), 30, 30, 6, 1.0, 1.5, 90, Color::default(), 2.5);
                let d = mpool2.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.2;
                d.fade_in = 1.0;
                if mirror2.next_max(6) == 0 {
                    d.velocity *= 30.0;
                    d.no_gravity = false;
                    d.no_light = true;
                } else {
                    let speed = d.velocity.length();
                    let n = normalize(Vec2::new(515.0 - d.position.x, 515.0 - d.position.y));
                    d.velocity = n * speed;
                }
            }
        }
        assert_eq!(rng2.next_range(0, 1000), mirror2.next_range(0, 1000));
        let a2 = active(&pool2);
        let m2 = active(&mpool2);
        assert_eq!(a2.len(), m2.len());
        for (&i, &j) in a2.iter().zip(m2.iter()) {
            assert_eq!(pool2.slot(i).velocity, mpool2.slot(j).velocity);
        }
    }

    /// Sites 40024/40035 — the Ancient Light burst is 13 type-261 dusts;
    /// the trail's gate bound is 10 - (int)min(7, |v|). At |v| = 0 the
    /// bound is 10 (Next(10) < 1 → 1/10); at |v| >= 7 it is 3.
    #[test]
    fn w4_e2_ancient_light_burst_and_speed_scaled_gate() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(81);
        ancient_light_launch_burst(600.0, 600.0, 24, 24, 0.0, 0.0, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 13);
        for i in 0..13 {
            assert_eq!(pool.slot(i).dust_type, 261);
            assert!(pool.slot(i).no_gravity && pool.slot(i).no_light);
        }
        // The gate bound: |v| = 3 → min(7,3) = 3 → bound 7; |v| = 9 →
        // min(7,9) = 7 → bound 3. Replay the trail at both speeds and
        // confirm the stream matches a mirror drawing Next(bound).
        for (vx, vy, bound) in [(3.0f32, 0.0f32, 7i32), (9.0, 0.0, 3)] {
            let mut p = DustPool::new();
            let mut mp = DustPool::new();
            let mut r = UnifiedRandom::new(82);
            let mut m = UnifiedRandom::new(82);
            ancient_light_trail(600.0, 600.0, 612.0, 612.0, 24, 24, vx, vy, &mut r, &mut p, &c, &w);
            for _ in 0..2 {
                if m.next_max(bound) < 1 {
                    let idx = mp.new_dust(&mut m, &c, &w, Vec2::new(600.0, 600.0), 24, 24, 261, vx * 0.5, vy * 0.5, 90, Color::default(), 2.5);
                    let d = mp.slot_mut(idx);
                    d.no_gravity = true;
                    d.velocity *= 0.2;
                    d.fade_in = 0.4;
                    if m.next_max(6) == 0 {
                        d.velocity *= 5.0;
                        d.no_light = true;
                    } else {
                        let speed = d.velocity.length();
                        d.velocity = normalize(Vec2::new(612.0 - d.position.x, 612.0 - d.position.y)) * speed;
                    }
                }
            }
            assert_eq!(r.next_range(0, 1000), m.next_range(0, 1000), "bound {bound}");
        }
    }

    /// Sites 40100/40110/40120 — the Ancient Doom's three sequential Next(6)
    /// gates; each passed gate draws the RotatedByRandom pair + the pool
    /// internals + the per-gate post draws (Next(10,21) for gate 1, a
    /// NextFloat scale for each). The mirror replays all three.
    #[test]
    fn w4_e2_ancient_doom_three_sequential_ring_gates() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(91);
        let mut mirror = UnifiedRandom::new(91);
        ancient_doom_rings(1000.0, 1000.0, 1.0, &mut rng, &mut pool, &c, &w);
        // Gate 1: Next(6), then on 0 the rotation pair + NewDust(Center -
        // v*20, 0, 0, 27) + Next(10,21) + NextFloat.
        if mirror.next_max(6) == 0 {
            let a = mirror.next_double() * 6.2831854820251465 - mirror.next_double() * 6.2831854820251465;
            let (rx, ry) = (-(a.sin() as f32), a.cos() as f32);
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(1000.0 - rx * 20.0, 1000.0 - ry * 20.0), 0, 0, 27, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            let n = mirror.next_range(10, 21) as f32;
            d.position = Vec2::new(1000.0 - (rx * n) * 1.0, 1000.0 - (ry * n) * 1.0);
            let r = 1.5707963705062866f64;
            let (cc, ss) = (r.cos() as f32, r.sin() as f32);
            d.velocity = Vec2::new(rx * cc - ry * ss, rx * ss + ry * cc) * 4.0;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
        }
        // Gate 2: Next(6), rotation pair + NewDust(..., 240) + NextFloat.
        if mirror.next_max(6) == 0 {
            let a = mirror.next_double() * 6.2831854820251465 - mirror.next_double() * 6.2831854820251465;
            let (rx, ry) = (-(a.sin() as f32), a.cos() as f32);
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(1000.0 - rx * 30.0, 1000.0 - ry * 30.0), 0, 0, 240, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = Vec2::new(1000.0 - (rx * 20.0) * 1.0, 1000.0 - (ry * 20.0) * 1.0);
            let r = -1.5707963705062866f64;
            let (cc, ss) = (r.cos() as f32, r.sin() as f32);
            d.velocity = Vec2::new(rx * cc - ry * ss, rx * ss + ry * cc) * 2.0;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
        }
        // Gate 3: Next(6), rotation pair + NewDust(..., 240) + NextFloat.
        if mirror.next_max(6) == 0 {
            let a = mirror.next_double() * 6.2831854820251465 - mirror.next_double() * 6.2831854820251465;
            let (rx, ry) = (-(a.sin() as f32), a.cos() as f32);
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(1000.0 - rx * 30.0, 1000.0 - ry * 30.0), 0, 0, 240, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.position = Vec2::new(1000.0 - (rx * 20.0) * 1.0, 1000.0 - (ry * 20.0) * 1.0);
            d.velocity = Vec2::ZERO;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
            d.no_light = true;
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        let a = active(&pool);
        let m = active(&mpool);
        assert_eq!(a.len(), m.len());
        for (&i, &j) in a.iter().zip(m.iter()) {
            assert_eq!(pool.slot(i).position, mpool.slot(j).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(j).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(j).scale);
        }
    }

    /// Site 40193 — the sand elemental's bottom pair: two Next(2) gates,
    /// each passed gate draws two NextFloats (the lerp) + the pool
    /// internals; iteration 1 adds a RandomVector2 (two NextDouble).
    #[test]
    fn w4_e2_sand_elemental_bottom_pair_gates_and_lerp() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let mut mirror = UnifiedRandom::new(101);
        // Bottom (1000, 1100), BottomLeft.x = 950, BottomRight.x = 1050,
        // spriteDirection 1, vy 2 → num1391 = -0.3 + 4 = 3.7.
        sand_elemental_bottom_sand(1000.0, 1100.0, 950.0, 1050.0, 1, 2.0, &mut rng, &mut pool, &c, &w);
        let color = {
            let cc = crate::render::special_tiles::xna_scale_color([222, 108, 48, 255], 0.7);
            Color::new(cc[0], cc[1], cc[2], cc[3])
        };
        for n in 0..2 {
            if mirror.next_max(2) != 0 {
                let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(1000.0, 1100.0), 0, 0, 268, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = mpool.slot_mut(idx);
                let tx = mirror.next_float();
                let ty = mirror.next_float();
                let lx = 950.0 + 100.0 * tx;
                let ly = 1088.0 + 12.0 * ty;
                d.position = Vec2::new(lx - 10.0, ly - 4.0);
                if n == 1 {
                    let rx = 12.0 * mirror.next_double() as f32 - 6.0;
                    let ry = 12.0 * mirror.next_double() as f32 - 6.0;
                    d.position = Vec2::new(1000.0 + rx, 1100.0 + ry);
                }
                d.color = color;
                d.scale = 0.8;
                d.velocity.y += 3.7;
                d.velocity.x += 0.2;
            }
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        let a = active(&pool);
        let m = active(&mpool);
        assert_eq!(a.len(), m.len());
        for (&i, &j) in a.iter().zip(m.iter()) {
            assert_eq!(pool.slot(i).position, mpool.slot(j).position);
            assert_eq!(pool.slot(i).color, mpool.slot(j).color);
            assert_eq!(pool.slot(i).scale, 0.8);
        }
    }

    /// Site 40302 — the tether: at ai0 = 54, num1396 = 0 → in range; one
    /// 658 centre at the anchor gives value3 = anchor (t=0 Hermite) and
    /// value4 = anchor + (0,-35.625) (t=0.05: the -off·t + off·t² terms),
    /// dist 35.625 > 10 → count 4 type-269 dusts. At ai0 = 40,
    /// num1396 < 0 → NO draws.
    #[test]
    fn w4_e2_sand_tether_count_and_range_gate() {
        let w = world();
        let c = ctx();
        let anchor = Vec2::new(1000.0, 1000.0);
        // Out of range: ai0 = 40 → num1396 = -0.466 < 0.
        let mut p0 = DustPool::new();
        let mut r0 = UnifiedRandom::new(111);
        let before = r0.clone();
        sand_elemental_tether(anchor, 40.0, &[anchor], &mut r0, &mut p0, &c, &w);
        assert_eq!(active(&p0).len(), 0);
        assert_eq!(r0, before, "the range gate draws nothing");
        // In range: ai0 = 54 → num1396 = 0, one centre at the anchor.
        let mut p1 = DustPool::new();
        let mut mp1 = DustPool::new();
        let mut r1 = UnifiedRandom::new(112);
        let mut m1 = UnifiedRandom::new(112);
        sand_elemental_tether(anchor, 54.0, &[anchor], &mut r1, &mut p1, &c, &w);
        let off = Vec2::new(0.0, 1500.0);
        let value3 = catmull_rom(anchor + off, anchor, anchor, anchor + off, 0.0);
        let value4 = catmull_rom(anchor + off, anchor, anchor, anchor + off, 0.05);
        let dist = (value3 - value4).length();
        let count = if dist > 10.0 { 4.0 } else if dist > 5.0 { 3.0 } else { 2.0 };
        assert_eq!(count, 4.0, "the t=0.05 offset spans 35.6 px");
        let mut i = 0.0f32;
        while i < count {
            let idx = mp1.new_dust(&mut m1, &c, &w, anchor, 0, 0, 269, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mp1.slot_mut(idx);
            let lerp = value3 + (value4 - value3) * (i / count);
            let rx = 4.0 * m1.next_double() as f32 - 2.0;
            let ry = 4.0 * m1.next_double() as f32 - 2.0;
            d.position = lerp + Vec2::new(rx, ry);
            d.no_light = true;
            d.scale = 0.3;
            i += 1.0;
        }
        assert_eq!(active(&p1).len(), 4);
        assert_eq!(r1.next_range(0, 1000), m1.next_range(0, 1000));
        for i in 0..4 {
            assert_eq!(p1.slot(i).position, mp1.slot(i).position);
            assert!(p1.slot(i).no_light);
            assert_eq!(p1.slot(i).scale, 0.3);
        }
    }

    /// Sites 39524/39559/39571 — the floater burst: 20 type-242 dusts,
    /// velocity ×3, noGravity, scale 2.5. Zero explicit draws — the
    /// stream advances by the pool internals only (mirrored).
    #[test]
    fn w4_e2_nebula_floater_burst_twenty_242() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(121);
        let mut mirror = UnifiedRandom::new(121);
        nebula_floater_teleport_burst(800.0, 800.0, 40, 40, &mut rng, &mut pool, &c, &w);
        for _ in 0..20 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(800.0, 800.0), 40, 40, 242, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.velocity *= 3.0;
            d.no_gravity = true;
            d.scale = 2.5;
        }
        assert_eq!(active(&pool).len(), 20);
        for i in 0..20 {
            assert_eq!(pool.slot(i).dust_type, 242);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).scale, 2.5);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 batch 6 — sites 43251/43278: the shrink ring (vel_mult 0.5)
    /// and the grow ring (vel_mult 2) are the SAME 10×4 body at
    /// (x-20, y), box (width+40, height), alpha 150, Color(78,136,255,80),
    /// scale 2; post noGravity then velocity *= mult. Mirrored pool
    /// internals pin the stream; the two mults differ only in the post.
    #[test]
    fn w4_e2_king_slime_teleport_ring_shrink_and_grow() {
        let w = world();
        let c = ctx();
        for (mult, seed) in [(0.5f32, 131u64), (2.0, 137)] {
            let mut pool = DustPool::new();
            let mut mpool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let mut mirror = UnifiedRandom::new(seed);
            king_slime_teleport_ring(
                900.0, 700.0, 122, 115, 1.5, -2.0, mult,
                &mut rng, &mut pool, &c, &w,
            );
            for _ in 0..10 {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, Vec2::new(900.0 - 20.0, 700.0),
                    122 + 40, 115, 4, 1.5, -2.0, 150, Color::new(78, 136, 255, 80), 2.0,
                );
                let d = mpool.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= mult;
            }
            assert_eq!(active(&pool).len(), 10, "mult {mult}: 10 dusts");
            for i in 0..10 {
                assert_eq!(pool.slot(i).dust_type, 4);
                assert!(pool.slot(i).no_gravity);
                assert_eq!(pool.slot(i).color, Color::new(78, 136, 255, 80));
                assert_eq!(pool.slot(i).position, mpool.slot(i).position);
                assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e2 batch 6 — site 43365: ONE type-4 body drip at (x, y), box
    /// (width, height), alpha 255, Color(0,80,255,80), scale = scale*1.2;
    /// post noGravity, velocity *= 0.5. The scale argument is the
    /// PRE-resize `this.scale` (the C# runs the drip before the hp-scale
    /// write), so the witness pins scale*1.2 at the caller's value.
    #[test]
    fn w4_e2_king_slime_body_drip_single_255() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(141);
        let mut mirror = UnifiedRandom::new(141);
        king_slime_body_drip(
            900.0, 700.0, 122, 115, 1.5, -2.0, 1.25,
            &mut rng, &mut pool, &c, &w,
        );
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, Vec2::new(900.0, 700.0), 122, 115, 4,
            1.5, -2.0, 255, Color::new(0, 80, 255, 80), 1.25 * 1.2,
        );
        let d = mpool.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.5;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).dust_type, 4);
        assert_eq!(pool.slot(0).alpha, 255);
        assert_eq!(pool.slot(0).color, Color::new(0, 80, 255, 80));
        assert!(pool.slot(0).no_gravity);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 batch 6 — site 24520: the Antlion fall-drip. A failed gate
    /// spawns nothing (the machine already drew the Next(2)); a passed
    /// gate spawns ONE type-32 at (x-4, y+height-8), box (width+8, 24),
    /// speedY vy/2; post velocity.X *= 0.4, velocity.Y *= -1, then the
    /// boost gate adds noGravity + scale += 0.2. Mirrored pool internals.
    #[test]
    fn w4_e2_antlion_fall_drip_gates_and_chains() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(151);
        antlion_fall_drip(false, false, 500.0, 500.0, 30, 30.0, -2.0, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty(), "the failed gate spawns nothing");
        // The passed gate + boost: mirror the pool internals.
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(151);
        antlion_fall_drip(true, true, 500.0, 500.0, 30, 30.0, -2.0, &mut rng, &mut pool, &c, &w);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, Vec2::new(500.0 - 4.0, 500.0 + 30.0 - 8.0),
            30 + 8, 24, 32, 0.0, -2.0 / 2.0, 0, Color::default(), 1.0,
        );
        let d = mpool.slot_mut(idx);
        d.velocity.x *= 0.4;
        d.velocity.y *= -1.0;
        d.no_gravity = true;
        d.scale += 0.2;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).dust_type, 32);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 batch 6 — sites 24850/24866: the Pixie's type-55 (color tint,
    /// velocity *= 0.3) and the Ice Elemental's type-92 (velocity *= 0.3,
    /// noGravity) ambient drips. Both gate on the caller's already-drawn
    /// result; the mirror pins the pool internals + the post chains.
    #[test]
    fn w4_e2_pixie_and_ice_elemental_ambient_drips() {
        let w = world();
        let c = ctx();
        // The Pixie: type 55, alpha 200, the NPC's color, velocity *= 0.3.
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(161);
        let mut mirror = UnifiedRandom::new(161);
        let tint = Color::new(255, 200, 100, 255);
        pixie_ambient_drip(true, 600.0, 600.0, 20, 20, tint, &mut rng, &mut pool, &c, &w);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, Vec2::new(600.0, 600.0), 20, 20, 55,
            0.0, 0.0, 200, tint, 1.0,
        );
        mpool.slot_mut(idx).velocity *= 0.3;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).dust_type, 55);
        assert_eq!(pool.slot(0).color, tint);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The Ice Elemental: type 92, alpha 200, velocity *= 0.3, noGravity.
        let mut pool2 = DustPool::new();
        let mut mpool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(163);
        let mut mirror2 = UnifiedRandom::new(163);
        ice_elemental_ambient_drip(true, 700.0, 700.0, 30, 40, &mut rng2, &mut pool2, &c, &w);
        let idx = mpool2.new_dust(
            &mut mirror2, &c, &w, Vec2::new(700.0, 700.0), 30, 40, 92,
            0.0, 0.0, 200, Color::default(), 1.0,
        );
        let d = mpool2.slot_mut(idx);
        d.velocity *= 0.3;
        d.no_gravity = true;
        assert_eq!(active(&pool2).len(), 1);
        assert_eq!(pool2.slot(0).dust_type, 92);
        assert!(pool2.slot(0).no_gravity);
        assert_eq!(pool2.slot(0).velocity, mpool2.slot(0).velocity);
        assert_eq!(rng2.next_range(0, 1000), mirror2.next_range(0, 1000));
    }

    /// W4-e2 batch 6 — sites 35157/35162: the turret materialize ring's 8
    /// placements (the npc_ai_style73 plan seam). Every dust is type-226
    /// at Center, 0×0 box, alpha 100, scale 0.5; post noGravity, position
    /// = the placement, velocity = Zero. The mirror replays the 8 pool
    /// internals + the position overwrites in C# pair order.
    #[test]
    fn w4_e2_turret_materialize_ring_eight_placements() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(171);
        let mut mirror = UnifiedRandom::new(171);
        let plan = crate::npc_ai_style73::turret_materialize_dust_plan(60.0, (1000.0, 1000.0));
        turret_materialize_ring(1000.0, 1000.0, &plan.placements, &mut rng, &mut pool, &c, &w);
        for &(px, py) in plan.placements.iter() {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(1000.0, 1000.0), 0, 0, 226,
                0.0, 0.0, 100, Color::default(), 0.5,
            );
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = Vec2::new(px, py);
            d.velocity = Vec2::ZERO;
        }
        assert_eq!(active(&pool).len(), 8);
        for i in 0..8 {
            assert_eq!(pool.slot(i).dust_type, 226);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).velocity, Vec2::ZERO);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 batch 6 — sites 37337/37464: the Squidhead's 228 ring. The
    /// spawn-in fires 2 iterations, the every-tick 1; both are the SAME
    /// body at (x-10, y-10), box (width+20, height+20), alpha 100, scale
    /// 2; post noGravity + noLight. The mirror pins the pool internals.
    #[test]
    fn w4_e2_squidhead_228_ring_two_and_one() {
        let w = world();
        let c = ctx();
        for (iters, seed) in [(2i32, 181u64), (1, 183)] {
            let mut pool = DustPool::new();
            let mut mpool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let mut mirror = UnifiedRandom::new(seed);
            squidhead_228_ring(iters, 800.0, 800.0, 60, 60, &mut rng, &mut pool, &c, &w);
            for _ in 0..iters {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, Vec2::new(800.0 - 10.0, 800.0 - 10.0),
                    60 + 20, 60 + 20, 228, 0.0, 0.0, 100, Color::default(), 2.0,
                );
                let d = mpool.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
            }
            assert_eq!(active(&pool).len(), iters as usize);
            for i in 0..iters as usize {
                assert_eq!(pool.slot(i).dust_type, 228);
                assert!(pool.slot(i).no_gravity);
                assert!(pool.slot(i).no_light);
                assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e2 batch 6 — site 37289: the Shadowflame Apparition's spawn-in
    /// 20×27 ring. Per dust the argument draws ride FIRST (the two
    /// Next(-100,101) position components, the Next(0,100) radius, the
    /// Next(50,90) speed), then the pool internals, then the post
    /// Next(2) boost gate. The mirror replays the full census.
    #[test]
    fn w4_e2_shadowflame_apparition_spawn_ring_census() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(191);
        let mut mirror = UnifiedRandom::new(191);
        shadowflame_apparition_spawn_ring(1000.0, 1000.0, &mut rng, &mut pool, &c, &w);
        for _ in 0..20 {
            let (mut px, mut py) = (1000.0f32, 1000.0 - 18.0);
            let ox = mirror.next_range(-100, 101) as f32;
            let oy = mirror.next_range(-100, 101) as f32;
            let mut v = normalize(Vec2::new(ox, oy));
            let r = mirror.next_range(0, 100) as f32 * 0.1;
            v *= r;
            px += v.x;
            py += v.y;
            let mut vel = normalize(v);
            let speed = mirror.next_range(50, 90) as f32 * 0.2;
            vel *= speed;
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(px, py), 1, 1, 27,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.velocity = vel * -0.3;
            d.alpha = 100;
            if mirror.next_max(2) == 0 {
                d.no_gravity = true;
                d.scale += 0.3;
            }
        }
        assert_eq!(active(&pool).len(), 20);
        for i in 0..20 {
            assert_eq!(pool.slot(i).dust_type, 27);
            assert_eq!(pool.slot(i).alpha, 100);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 batch 6 — site 37307: the Shadowflame Apparition's fade drip.
    /// The FLOAT bound `(1 - localAI[0]/120) * 20` drives the loop; each
    /// iteration gates Next(5)==0 (the caller draws them in order). The
    /// witness passes a mixed gate slice and mirrors the passed spawns'
    /// pool internals + the post chain (alpha 100, velocity *= 0.3,
    /// velocity += (vx,vy)*0.75, noGravity).
    #[test]
    fn w4_e2_shadowflame_apparition_fade_drip_float_bound() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        let mut rng = UnifiedRandom::new(193);
        let mut mirror = UnifiedRandom::new(193);
        // localAI[0] = 60 after the increment -> bound = (1 - 60/120)*20 = 10.
        let count = (1.0 - 60.0f32 / 120.0) * 20.0;
        assert_eq!(count, 10.0);
        // 10 gates: indices 0, 3, 7 pass (the Next(5)==0 hits).
        let gates = [true, false, false, true, false, false, false, true, false, false];
        shadowflame_apparition_fade_drip(
            count, &gates, 900.0, 900.0, 40, 24, 1.5, -0.5,
            &mut rng, &mut pool, &c, &w,
        );
        let mut spawned = 0;
        for &g in gates.iter() {
            if g {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, Vec2::new(900.0, 900.0), 40, 24, 27,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = mpool.slot_mut(idx);
                d.alpha = 100;
                d.velocity *= 0.3;
                d.velocity += Vec2::new(1.5, -0.5) * 0.75;
                d.no_gravity = true;
                spawned += 1;
            }
        }
        assert_eq!(active(&pool).len(), spawned);
        assert_eq!(spawned, 3);
        let live = active(&pool);
        let mlive = active(&mpool);
        for (&i, &j) in live.iter().zip(mlive.iter()) {
            assert_eq!(pool.slot(i).dust_type, 27);
            assert_eq!(pool.slot(i).alpha, 100);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).velocity, mpool.slot(j).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }
}
