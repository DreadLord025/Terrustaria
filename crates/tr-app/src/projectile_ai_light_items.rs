//! S1-P-i42 — the torch/glowstick/flare LIGHT-ITEM projectile family:
//! the aiStyle-14 glowsticks (50/53/473/515/870/1089) and the aiStyle-33
//! flares (163/310/1008-1011), plus the fairy glowstick's aiStyle-170 hover.
//!
//! Census verdicts (the C# the port now matches):
//!
//! * **aiStyle 14** (`AI()`'s `else if (this.aiStyle == 14)`,
//!   `Projectile.cs:24830-25307`): the thrown-light flight. The generic
//!   default tail (`:25142-25160`) is the 5-tick gravity delay
//!   (`ai[0]++`, clamp 5, `velocity.Y += 0.2f`, the `×0.97` ground friction
//!   with the ±0.01 snap) + `rotation += velocity.X * 0.1f`; the ±16 Y clamp
//!   closes the branch (`:25303-25306`). The per-type heads:
//!   - 870 flips `aiStyle = 170` once `ai[1] > 0` (`:24832-24835`);
//!   - 473's spelunker spot helper (`:24836-24847`) — the 10-tick
//!     `localAI[0]` counter is kept, the `SpelunkerProjectileHelper` call is
//!     the documented skip (no ore-highlight layer);
//!   - 53's stick scan (`:24911-24962`): the box-vs-solid overlap zeroes X
//!     and pins `velocity.Y = -0.2f` (the hover), the `BreaksFromToyBreaker`
//!     kill on a `BreaksToys` tile (740, the rainbow boulder) included.
//! * **The bounce** lives in `HandleMovement`'s blocked-move chain, NOT the
//!   AI. The outer TYPE chain (`:18398-18618`) is checked before the aiStyle
//!   chain: 515 rides the 515-517 arm (`:18479-18489`, both axes `×-0.9`
//!   gated `|last| > 1`). Inside the aiStyle-14 arm (`:19626-19745`): 870's
//!   pre-flip (`:19663-19669`, the full `-lastVelocity` + `ai[1] = 1`), 50
//!   (`:19671-19681`, X `×-0.2`, Y `×-0.2` only when `lastVelocity.Y > 1.5`),
//!   `type >= 1089` (`:19682-19702`, the `×0.99` reverse with the <1 snap to
//!   ±1 / −1), and the generic tail (`:19725-19744`, X `×-0.5`, Y `×-0.5`
//!   only when `lastVelocity.Y > 1`) which takes the spelunker (473). Once
//!   the fairy flips to aiStyle 170, the later `type == 870` arm
//!   (`:19787-19797`) takes its blocked moves (X `×-0.2`, Y `×-0.2` when
//!   `lastVelocity.Y > 0`). [`light_bounce`] ports those arms; the movement
//!   half ([`light_movement`]) runs the dry `TileCollision` + the blocked-move
//!   `Next(3)` roll (`:17948`) + the bounce + `UpdatePosition`'s integrate.
//! * **aiStyle 33** (`AI()`'s `else if (this.aiStyle == 33)`,
//!   `Projectile.cs:27049-27135`): the flare. The alpha −50 fade
//!   (`:27051-27058`), the aim-normalized-to-4 dust vector
//!   (`:27059-27068`), the `alpha < 70` dust trail (`:27069-27105` — the
//!   per-type dust ids 127/187/169/75/66/310, the 1010 rainbow hsl), the
//!   flight/stick state machine (`:27106-27129` — `localAI[0] == 0` snapshots
//!   the velocity into `ai[0]/ai[1]` and counts `localAI[1]` to 30 then adds
//!   `+0.09f` gravity; stuck, a `SolidCollision` miss un-sticks, the damage
//!   zeroes), the ±16 Y clamp and the `atan2(ai[1], ai[0]) + 1.57f` rotation
//!   (`:27130-27134`). The stick itself is `HandleMovement`'s aiStyle-33
//!   block (`:18619-18634`): wet → `position += lastVelocity / 2`, dry →
//!   `+= lastVelocity`, `velocity *= 0`, `localAI[0] = 1`.
//! * **aiStyle 170** (`AI_170_FairyGlowstick`, `Projectile.cs:45662-45716`):
//!   the fairy's post-bounce hover — the Item28 one-shot + X-zero
//!   (`:45670-45675`), the 4-frame animation (`:45676-45685`), the 10-tick
//!   solid-tile repulsion field (`:45686-45708`), the `×0.95` drag and the
//!   <1 snap-to-zero (`:45709-45715`).
//!
//! RNG by construction (ANALYSIS "The light-item projectile family"):
//! - The AI arms draw ZERO `Main.rand` on the UNWIRED path (the dust spawns
//!   are skipped — the pool has no dust layer, the frostmoon/ai026 precedent;
//!   the flare trail's `Dust.NewDust` internal rolls go with them). With a
//!   [`SpelunkerSpot`] attached (W4-g78, research/77 §1) the 473 head's
//!   counter-wrap and the 1008 stuck arm draw the helper's `Next(4)` per
//!   newly-checked spelunkable tile on the caller's UpdateProjectiles
//!   stream — the `_tilesChecked.Add && Next(4)` short-circuit rolls
//!   NOTHING for a repeat tile.
//! - The movement's blocked-move prologue consumes exactly ONE `Next(3)`
//!   (`:17948`, the FIRST operand of the `&&` — the ai026_c precedent): C#'s
//!   left-to-right evaluation rolls it on EVERY blocked move even when the
//!   trailing gates (`owner == myPlayer && CanCutTiles() && (friendly ||
//!   hostile) && damage > 0`) fail — the glowsticks' damage −1 skips only
//!   the cut BODY (`:17950-17985`), never the roll. The flares (damage 2)
//!   pass the gates too. The cut bodies skip with the unported layer. The
//!   fairy's post-flip movement folds the same draw (the aiStyle-170 arm
//!   keeps `tileCollide`, so its blocked moves roll too).
//! - The flare's `StatusNPC` OnFire roll (`Next(3)`, `:11680`) is a HIT-time
//!   draw wired in `projectile.rs::damage_pve` (the whip-debuff site).
//!
//! CODE-ONLY MODE (S1-P-i42): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); verified by re-reading the cited
//! C# blocks and hand arithmetic.
//!
//! Deliberate skips (ANALYSIS): dust (the flare trail + the fairy's), sound
//! (the fairy's Item28 `:45672`), `netUpdate`,
//! the wet movement half (the slot carries no liquid-wet flags — the dry
//! integrate stands, the `wet` stick branch folds dry), the inner
//! `SlopeCollision` passes (`correctSlopeCollision` is false for these kinds
//! but the port's `tile_collision` already resolves the ground ramps the
//! throw lands on — the slope refinement is the recorded stand-in gap, the
//! `pirate_movement` precedent).

use glam::Vec2;
use tr_core::World;

use crate::projectile::{
    safe_normalize, Projectile, BLUE_FLARE, BOUNCY_GLOWSTICK, CURSED_FLARE, FAIRY_GLOWSTICK,
    FLARE, GLOWSTICK, RAINBOW_FLARE, RAINBOW_GLOWSTICK, SHIMMER_FLARE, SPELUNKER_FLARE,
    SPELUNKER_GLOWSTICK, STICKY_GLOWSTICK,
};

/// The aiStyle-14 glowstick kinds this module owns (the bounce + flight).
pub(crate) const GLOWSTICK_KINDS: [u16; 6] = [
    GLOWSTICK,
    STICKY_GLOWSTICK,
    SPELUNKER_GLOWSTICK,
    BOUNCY_GLOWSTICK,
    FAIRY_GLOWSTICK,
    RAINBOW_GLOWSTICK,
];

/// The aiStyle-33 flare kinds.
pub(crate) const FLARE_KINDS: [u16; 6] = [
    FLARE,
    BLUE_FLARE,
    SPELUNKER_FLARE,
    CURSED_FLARE,
    RAINBOW_FLARE,
    SHIMMER_FLARE,
];

/// The aiStyle-14 flight tail (`Projectile.cs:25142-25160` + the ±16 clamp
/// `:25303-25306`): the 5-tick gravity delay, the ground friction with the
/// ±0.01 snap, the `rotation += velocity.X * 0.1f`. The per-type heads run
/// before it ([`glowstick_head`]); the bounce + integrate are
/// [`light_movement`]'s (the C# runs them in `HandleMovement`, after the AI).
fn glowstick_flight(p: &mut Projectile) {
    // `:25144-25158` — the generic default arm (none of 50/53/473/515/870/
    // 1089 matches the 261/277/378/483/538 per-type tails above it).
    p.ai[0] += 1.0;
    if p.ai[0] > 5.0 {
        p.ai[0] = 5.0;
        if p.vel.y == 0.0 && p.vel.x != 0.0 {
            p.vel.x *= 0.97;
            // `:25151` — `(double)velocity.X > -0.01 && < 0.01`.
            if p.vel.x > -0.01 && p.vel.x < 0.01 {
                p.vel.x = 0.0;
            }
        }
        p.vel.y += 0.2;
    }
    // `:25159` — `rotation += velocity.X * 0.1f`.
    p.rot += p.vel.x * 0.1;
    // `:25303-25306` — the branch's closing Y clamp.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
}

/// The aiStyle-14 per-type heads (`Projectile.cs:24832-24962`): the fairy's
/// style flip, the spelunker counter, the sticky stick scan.
///
/// The fairy's flip (`:24832-24835`) is observed on the tick AFTER the bounce
/// set `ai[1] = 1` (HandleMovement runs after AI()). On the flip tick the C#
/// sets `aiStyle = 170` but stays INSIDE the already-entered aiStyle-14 arm —
/// the 14 tail still runs (`:25142-25160`); the 170 hover arm
/// (`:36643-36645`) is a separate `else if` first reached NEXT tick. The port
/// models the persistent style in `local_ai[2]` (a spare slot — the C# 870
/// never reads localAI[2]; the row keeps aiStyle 14 for the dispatch).
fn glowstick_head(p: &mut Projectile, world: &World) {
    glowstick_head_inner(p, world, None, &mut NoSpelunkerRng);
}

/// The W4-g78 spelunker attach context (research/77 §1): the pool-owned
/// helper + the per-tick player/screen snapshot + the dust-request sink.
/// `net_mode_ok` is the C# `Main.netMode != 2` gate (`Projectile.cs:24836` /
/// `:27124`) — the WHOLE 473 counter block (not just the helper call) is
/// server-skipped in C#; the unwired `None` path keeps the S1-P-i42
/// unconditional counter (the documented SP form).
pub(crate) struct SpelunkerSpot<'a> {
    pub helper: &'a mut crate::spelunker_helper::SpelunkerProjectileHelper,
    pub player_center: Option<Vec2>,
    pub screen_width: f32,
    pub net_mode_ok: bool,
    pub out: &'a mut Vec<crate::spelunker_helper::SpelunkerDust>,
}

/// The `None`-attach head draws NOTHING through the helper (the 473 arm's
/// counter is RNG-free), so the legacy path needs no stream — this zero
/// type stands in without touching the callers' signatures.
struct NoSpelunkerRng;
impl tr_core::loot::DropRng for NoSpelunkerRng {
    fn next_range(&mut self, _: i32, _: i32) -> i32 {
        unreachable!("the unwired head draws no helper rolls")
    }
    fn next_float(&mut self) -> f32 {
        unreachable!("the unwired head draws no helper rolls")
    }
}

fn glowstick_head_inner(
    p: &mut Projectile,
    world: &World,
    mut spot: Option<SpelunkerSpot<'_>>,
    rng: &mut dyn tr_core::loot::DropRng,
) {
    // `:24832-24835` — 870: `ai[1] > 0` flips the style to 170. The bounce
    // that SETS `ai[1] = 1` runs in HandleMovement AFTER the AI, so the flip
    // is observed on the NEXT tick's head — exactly the C#'s order.
    if p.kind == FAIRY_GLOWSTICK && p.ai[1] > 0.0 && p.local_ai[2] == 0.0 {
        p.local_ai[2] = 1.0;
        // NOT an early return: the flip tick still runs the 14 tail below
        // (and the 14 per-type heads — none of 473/53 matches 870 anyway).
    }
    // `:24836-24847` — 473: the 10-tick spelunker counter. W4-g78 wiring:
    // with a `SpelunkerSpot` attached the counter-wrap runs the C# distance
    // gate + `AddSpotToCheck` (the helper's Next(4) draws ride the caller's
    // UpdateProjectiles stream); the C# `Main.netMode != 2` gate wraps the
    // WHOLE block (`:24836`). The unwired `None` path keeps the S1-P-i42
    // unconditional counter (the SP form, the documented skip until the
    // dispatch swap lands — research/77 §1.1).
    if p.kind == SPELUNKER_GLOWSTICK && spot.as_ref().is_none_or(|s| s.net_mode_ok) {
        p.local_ai[0] += 1.0;
        if p.local_ai[0] >= 10.0 {
            p.local_ai[0] = 0.0;
            if let Some(s) = spot.as_mut() {
                let (cx, cy) = p.center();
                let center = Vec2::new(cx, cy);
                if crate::spelunker_helper::SpelunkerProjectileHelper::spot_in_range(
                    center,
                    s.player_center,
                    s.screen_width,
                ) {
                    s.helper.add_spot_to_check(world, center, rng, s.out);
                }
            }
        }
    }
    // `:24911-24962` — 53: the stick scan. The box (`position`, 6x6) against
    // every solid non-solidTop tile in the ±1/+2 cell window; an overlap
    // zeroes X and pins `velocity.Y = -0.2f` (the hover). The
    // `BreaksFromToyBreaker[53]` kill on a `BreaksToys` tile (740) is kept.
    if p.kind == STICKY_GLOWSTICK {
        let x0 = ((p.pos.x / 16.0) as i32 - 1).max(0);
        let x1 = (((p.pos.x + p.w) / 16.0) as i32 + 2).min(world.width);
        let y0 = ((p.pos.y / 16.0) as i32 - 1).max(0);
        let y1 = (((p.pos.y + p.h) / 16.0) as i32 + 2).min(world.height);
        for ty in y0..y1 {
            for tx in x0..x1 {
                let Some(t) = world.tile(tx, ty) else {
                    continue;
                };
                // `:24941` — `!nactive() || !tileSolid || tileSolidTop` skip.
                if !tr_core::tileobject::nactive(t)
                    || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                    || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    continue;
                }
                let rx = tx as f32 * 16.0;
                let ry = ty as f32 * 16.0;
                // `:24947` — the box overlap.
                if p.pos.x + p.w > rx
                    && p.pos.x < rx + 16.0
                    && p.pos.y + p.h > ry
                    && p.pos.y < ry + 16.0
                {
                    p.vel.x = 0.0;
                    p.vel.y = -0.2;
                    // `:24951-24954` — `BreaksFromToyBreaker[53]` (true,
                    // `ProjectileID.cs:189`) && `BreaksToys[tile.type]` (740,
                    // `TileID.cs:479`) → Kill.
                    if t.kind == 740 {
                        p.active = false;
                        p.time_left = 0;
                        return;
                    }
                }
            }
        }
    }
}

/// One glowstick tick: the head, the flight (or the fairy hover once the
/// style flip has been OBSERVED — the flip tick itself still flies, the C#'s
/// aiStyle-14 arm body), then the movement ([`light_movement`]). The C# order
/// is AI() → HandleMovement(); the port folds both here so the dispatch arm
/// is a single call (the frostmoon precedent).
pub(crate) fn glowstick_step(p: &mut Projectile, world: &World, rng: &mut dyn tr_core::loot::DropRng) {
    glowstick_step_with_spot(p, world, rng, None);
}

/// The W4-g78 attach form (research/77 §1.1): the same tick with the
/// pool-owned spelunker helper threaded to the 473 head. The dispatch
/// swap (`projectile.rs`'s aiStyle-14 arm passes `Some(SpelunkerSpot { … })`
/// from the pool fields the pass head fills) is the contested hunk — this
/// variant is the landing target, compiled + witnessed here.
pub(crate) fn glowstick_step_with_spot(
    p: &mut Projectile,
    world: &World,
    rng: &mut dyn tr_core::loot::DropRng,
    spot: Option<SpelunkerSpot<'_>>,
) {
    // The flip is observed at the head; the tick it happens on still runs
    // the 14 tail (the C# stays inside the entered arm).
    let flipped_this_tick =
        p.kind == FAIRY_GLOWSTICK && p.ai[1] > 0.0 && p.local_ai[2] == 0.0;
    glowstick_head_inner(p, world, spot, rng);
    if !p.active {
        return;
    }
    if p.kind == FAIRY_GLOWSTICK && p.local_ai[2] > 0.0 && !flipped_this_tick {
        // aiStyle 170 (`:36643-36645` → `AI_170_FairyGlowstick`) — the hover.
        fairy_glowstick_hover(p, world);
    } else {
        glowstick_flight(p);
    }
    light_movement(p, world, rng);
}

/// `HandleMovement`'s dry half for the glowsticks (`Projectile.cs:17469-
/// 19865`): the `TileCollision` (the >16 px/tick hoik step loop else the
/// single pass), the blocked-move `Next(3)` roll + the per-type bounce, then
/// `UpdatePosition`'s integrate. The sticky (53) has `tileCollide = false`
/// (`:1364`) — it just integrates (the stick scan in the AI owns the stop).
fn light_movement(p: &mut Projectile, world: &World, rng: &mut dyn tr_core::loot::DropRng) {
    if !p.tile_collide {
        // `:17476`'s gate — no tile pass, just the integrate (`:20445`).
        p.pos += p.vel;
        return;
    }
    let last_velocity = p.vel;
    // `:17702-17710` — the step size clamp (min(w,h) clamped 3..16).
    let num2 = (p.w.min(p.h) as i32).clamp(3, 16) as f32;
    if p.vel.length() > num2 {
        // `:17711-17846` — the hoik step loop (the pirate_movement port).
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            true,
            true,
        );
        let mut num3 = p.vel.length();
        let mut vector8 = safe_normalize(p.vel);
        if tc.vel[1] == 0.0 {
            vector8.y = 0.0;
        }
        let mut zero = Vec2::ZERO;
        let mut num5 = 0;
        while num3 > 0.0 {
            num5 += 1;
            if num5 > 300 {
                break;
            }
            let num6 = num3.min(num2);
            num3 -= num6;
            let step_vel = vector8 * num6;
            let tc = crate::collision::tile_collision(
                world,
                [p.pos.x, p.pos.y],
                [step_vel.x, step_vel.y],
                p.w,
                p.h,
                true,
                true,
            );
            let v10 = Vec2::new(tc.vel[0], tc.vel[1]);
            p.pos += v10;
            p.vel = v10;
            zero += p.vel;
        }
        p.vel = zero;
        // `:17795-17802` — the last-velocity snap.
        if (p.vel.x - last_velocity.x).abs() < 0.0001 {
            p.vel.x = last_velocity.x;
        }
        if (p.vel.y - last_velocity.y).abs() < 0.0001 {
            p.vel.y = last_velocity.y;
        }
    } else {
        // `:17849` — the single collision pass.
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            true,
            true,
        );
        p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
    }
    // `:17896-17899` — a changed velocity is a blocked move.
    if p.vel != last_velocity {
        // `:17948` — `Main.rand.Next(3) == 0 && owner == Main.myPlayer &&
        // CanCutTiles() && (friendly || hostile) && damage > 0`: the draw is
        // the FIRST operand of the `&&`, so C#'s left-to-right evaluation
        // CONSUMES it on every blocked move regardless of the trailing
        // gates (the ai026_c precedent — "even when the owner is dead or
        // the damage is 0"). The glowsticks' damage −1 only skips the cut
        // BODY, never the roll. The locally-simulated slots fold the owner
        // gate; the glowsticks pass `CanCutTiles()` (`:15323-15334` — none
        // is on a refusal list). The cut bodies skip with the unported
        // layer (module doc).
        let _ = rng.next_range(0, 3);
        // The per-type bounce arms (the blocked-move chain).
        light_bounce(p, last_velocity);
    }
    // `UpdatePosition` (`:20445`) — the dry integrate.
    p.pos += p.vel;
}

/// `HandleMovement`'s blocked-move bounce arms for the glowsticks. The
/// chain order is load-bearing: the outer type chain
/// (`Projectile.cs:18398-18618`) is checked BEFORE the aiStyle chain, so
/// the 515-517 arm (`:18479-18489`, both axes `×-0.9` gated `|last| > 1`)
/// takes the bouncy glowstick before the aiStyle-14 arm is ever reached.
/// Inside the aiStyle-14 arm (`:19626-19745`): 870 pre-flip
/// (`:19663-19669`, the full `-lastVelocity` + `ai[1] = 1`), 50
/// (`:19671-19681`, X `×-0.2`, Y `×-0.2` only when `lastVelocity.Y > 1.5`),
/// `type >= 1089` (`:19682-19702`, the `×0.99` reverse with the <1 snaps),
/// and the generic tail (`:19725-19744`, X `×-0.5`, Y `×-0.5` only when
/// `lastVelocity.Y > 1`) which takes the SPELUNKER (473) — 53 never
/// collides (`tileCollide = false`). Once the fairy has flipped to
/// aiStyle 170 (`ai[1] > 0`), the aiStyle-14 arm no longer matches and the
/// later `type == 870` arm (`:19787-19797`) takes its blocked moves
/// (X `×-0.2`, Y `×-0.2` when `lastVelocity.Y > 0`).
fn light_bounce(p: &mut Projectile, last: Vec2) {
    match p.kind {
        BOUNCY_GLOWSTICK => {
            // `:18481-18488` — the outer 515-517 arm, both axes `×-0.9`,
            // gated `|last| > 1`.
            if p.vel.x != last.x && last.x.abs() > 1.0 {
                p.vel.x = last.x * -0.9;
            }
            if p.vel.y != last.y && last.y.abs() > 1.0 {
                p.vel.y = last.y * -0.9;
            }
        }
        FAIRY_GLOWSTICK if p.ai[1] > 0.0 => {
            // `:19789-19796` — the post-flip arm (aiStyle 170 skipped the
            // aiStyle-14 arm above it).
            if p.vel.x != last.x {
                p.vel.x = last.x * -0.2;
            }
            if p.vel.y != last.y && last.y > 0.0 {
                p.vel.y = last.y * -0.2;
            }
        }
        FAIRY_GLOWSTICK => {
            // `:19663-19669` — `ai[1] == 0`: the full reverse + the flip flag.
            if p.vel.x != last.x || p.vel.y != last.y {
                p.vel = -last;
                p.ai[1] = 1.0;
            }
        }
        GLOWSTICK => {
            // `:19673-19680` — X `×-0.2` on any X change; Y `×-0.2` only when
            // `lastVelocity.Y > 1.5`.
            if p.vel.x != last.x {
                p.vel.x = last.x * -0.2;
            }
            if p.vel.y != last.y && last.y > 1.5 {
                p.vel.y = last.y * -0.2;
            }
        }
        RAINBOW_GLOWSTICK => {
            // `:19684-19701` — the `type >= 1089` arm: the `×0.99` reverse
            // with the <1 snaps.
            if p.vel.x != last.x {
                let small = (p.vel.x - last.x).abs() < 1.0;
                p.vel.x = (0.0 - last.x) * 0.99;
                if small {
                    p.vel.x = if p.vel.x < 0.0 { -1.0 } else { 1.0 };
                }
            }
            if p.vel.y != last.y {
                let small = last.y > 0.0 && last.y < 1.0;
                p.vel.y = (0.0 - last.y) * 0.99;
                if small {
                    p.vel.y = -1.0;
                }
            }
        }
        SPELUNKER_GLOWSTICK => {
            // `:19736-19743` — the aiStyle-14 generic tail (473 matches none
            // of the per-type arms above it): X `×-0.5` on any X change;
            // Y `×-0.5` only when `lastVelocity.Y > 1`.
            if p.vel.x != last.x {
                p.vel.x = last.x * -0.5;
            }
            if p.vel.y != last.y && last.y > 1.0 {
                p.vel.y = last.y * -0.5;
            }
        }
        _ => {}
    }
}

/// `AI_170_FairyGlowstick` (`Projectile.cs:45662-45716`): the post-bounce
/// hover. The Item28 sound (`:45672`) is the documented skip; the X-zero +
/// `ai[1] = 2` latch, the 4-frame animation, the 10-tick solid-tile repulsion
/// field, the `×0.95` drag and the <1 snap-to-zero are 1:1. The movement
/// half stays [`light_movement`]'s (the C# runs HandleMovement after AI()
/// regardless of style — the post-flip blocked moves take the `type == 870`
/// bounce arm `:19787-19797`).
fn fairy_glowstick_hover(p: &mut Projectile, world: &World) {
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let px = (cx / 16.0) as i32;
    let py = (cy / 16.0) as i32;
    // `:45670-45675` — the one-shot: `ai[1] == 1` zeroes X and latches 2.
    if p.ai[1] == 1.0 {
        p.vel.x = 0.0;
        p.ai[1] = 2.0;
    }
    // `:45676-45685` — the 4-frame animation (`Main.projFrames[870] = 4`,
    // `Main.cs:8667`).
    p.frame_counter += 1;
    if p.frame_counter > 4 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= 4 {
            p.frame = 0;
        }
    }
    // `:45686-45708` — the repulsion field: every 10th tick while moving
    // (`velocity.Length() > 1`), the ±10-cell solid scan sums the
    // away-from-tile vectors weighted `(10 - dist) * 3`, averaged over the
    // count; a sum longer than 1 becomes the new velocity.
    if p.vel.length() > 1.0 && p.time_left % 10 == 0 {
        let mut sum = Vec2::ZERO;
        let mut count = 0;
        for i in px - 10..=px + 10 {
            for j in py - 10..=py + 10 {
                // `WorldGen.SolidTile(i, j)` (`:45692`) — out-of-world reads
                // solid (`:70115-70117`), the port's `solid_tile` matches.
                if crate::projectile::solid_tile(world, i, j) {
                    let v = Vec2::new(px as f32 - i as f32, py as f32 - j as f32);
                    let len = v.length();
                    let dir = if len > 0.0 { v / len } else { Vec2::ZERO };
                    sum += dir * (10.0 - len).max(0.0) * 3.0;
                    count += 1;
                }
            }
        }
        if count > 0 {
            sum /= count as f32;
        }
        if sum.length() > 1.0 {
            p.vel = sum;
        }
    }
    // `:45709-45715` — the drag, the rotation reset, the <1 snap-to-zero.
    p.vel *= 0.95;
    p.rot = 0.0;
    if p.vel != Vec2::ZERO && p.vel.length() < 1.0 {
        p.vel = Vec2::ZERO;
    }
}

/// One flare tick (aiStyle 33, `Projectile.cs:27049-27135`): the alpha fade,
/// the dust-trail vector (the dust itself skipped), the flight/stick state
/// machine, the rotation, then the movement ([`flare_movement`]).
pub(crate) fn flare_step(p: &mut Projectile, world: &World, rng: &mut dyn tr_core::loot::DropRng) {
    flare_step_with_spot(p, world, rng, None);
}

/// The W4-g78 attach form (research/77 §1.2): the 1008 Spelunker Flare's
/// STUCK arm calls `AddSpotToCheck(base.Center)` EVERY stuck tick (no
/// counter — `Projectile.cs:27124-27127`), gated `Main.netMode != 2` + the
/// same `screenWidth + 480` distance gate as the 473 head.
pub(crate) fn flare_step_with_spot(
    p: &mut Projectile,
    world: &World,
    rng: &mut dyn tr_core::loot::DropRng,
    mut spot: Option<SpelunkerSpot<'_>>,
) {
    // `:27051-27058` — the alpha −50 fade (the AI_001 head `:55592-55602`
    // also fades −25, but that is the aiStyle-1 path; the flare's own arm
    // here is −50).
    if p.alpha > 0 {
        p.alpha -= 50;
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }
    // `:27059-27068` — the aim vector normalized to 4 (the dust trail's
    // direction; `ai[0]/ai[1]` snapshot the launch velocity once stuck).
    let mut ax = p.ai[0];
    let mut ay = p.ai[1];
    if ax == 0.0 && ay == 0.0 {
        ax = 1.0;
    }
    let scale = 4.0 / (ax * ax + ay * ay).sqrt();
    let _dust_dir = Vec2::new(ax * scale, ay * scale);
    // `:27069-27105` — the `alpha < 70` dust trail: SKIPPED (no dust layer;
    // the per-type ids 127/187/169/75/66/310 and the 1010 hsl colour go with
    // it, module doc). The `Dust.NewDust` internal rolls are not consumed.
    //
    // `:27106-27129` — the flight/stick state machine.
    if p.local_ai[0] == 0.0 {
        // In flight: snapshot the velocity into the aim, count to 30 then
        // add the +0.09 gravity.
        p.ai[0] = p.vel.x;
        p.ai[1] = p.vel.y;
        p.local_ai[1] += 1.0;
        if p.local_ai[1] >= 30.0 {
            p.vel.y += 0.09;
            p.local_ai[1] = 30.0;
        }
    } else {
        // Stuck: a SolidCollision miss un-sticks (back to flight); the
        // damage zeroes. W4-g78 wiring: the 1008 spot call (`:27124-27127`)
        // runs AFTER the un-stick check, EVERY stuck tick, behind the
        // netMode + distance gates (the unwired `None` path keeps the
        // S1-P-i42 skip — research/77 §1.2).
        if !flare_solid_collision(world, p.pos.x, p.pos.y, p.w, p.h) {
            p.local_ai[0] = 0.0;
            p.local_ai[1] = 30.0;
        }
        if p.kind == SPELUNKER_FLARE {
            if let Some(s) = spot.as_mut() {
                if s.net_mode_ok {
                    let (cx, cy) = p.center();
                    let center = Vec2::new(cx, cy);
                    if crate::spelunker_helper::SpelunkerProjectileHelper::spot_in_range(
                        center,
                        s.player_center,
                        s.screen_width,
                    ) {
                        s.helper.add_spot_to_check(world, center, rng, s.out);
                    }
                }
            }
        }
        p.damage = 0;
    }
    // `:27130-27133` — the ±16 Y clamp.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `:27134` — `rotation = atan2(ai[1], ai[0]) + 1.57f`.
    p.rot = p.ai[1].atan2(p.ai[0]) + 1.57;
    flare_movement(p, world, rng);
}

/// `Collision.SolidCollision(pos, w, h)` (`Collision.cs:2700-2733`): the
/// box-vs-solid overlap the flare's un-stick test reads — active, not
/// actuated, solid, NOT solidTop, the half-brick shrink. The scan window is
/// the ±1/+2 cell box clamped to the world (the `maxTilesY - 40` cap).
fn flare_solid_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let x0 = ((x / 16.0) as i32 - 1).clamp(0, world.width - 1);
    let x1 = (((x + w) / 16.0) as i32 + 2).clamp(0, world.width - 1);
    let bottom = (world.height - 40).max(0);
    let y0 = ((y / 16.0) as i32 - 1).clamp(0, bottom);
    let y1 = (((y + h) / 16.0) as i32 + 2).clamp(0, bottom);
    for ty in y0..y1 {
        for tx in x0..x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active()
                || t.inactive()
                || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            {
                continue;
            }
            let rx = tx as f32 * 16.0;
            let mut ry = ty as f32 * 16.0;
            let mut rh = 16.0;
            if t.half_brick() {
                ry += 8.0;
                rh -= 8.0;
            }
            if x + w > rx && x < rx + 16.0 && y + h > ry && y < ry + rh {
                return true;
            }
        }
    }
    false
}

/// `HandleMovement`'s flare half: the dry `TileCollision`, then the
/// aiStyle-33 stick block (`Projectile.cs:18619-18634`) on a blocked move —
/// `position += lastVelocity` (the wet `/2` folds dry, module doc),
/// `velocity *= 0`, `localAI[0] = 1`. The `Next(3)` cut roll fires first
/// (`:17948`, the flare is friendly + damage>0 + CanCutTiles).
fn flare_movement(p: &mut Projectile, world: &World, rng: &mut dyn tr_core::loot::DropRng) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last_velocity = p.vel;
    let num2 = (p.w.min(p.h) as i32).clamp(3, 16) as f32;
    if p.vel.length() > num2 {
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            true,
            true,
        );
        let mut num3 = p.vel.length();
        let mut vector8 = safe_normalize(p.vel);
        if tc.vel[1] == 0.0 {
            vector8.y = 0.0;
        }
        let mut zero = Vec2::ZERO;
        let mut num5 = 0;
        while num3 > 0.0 {
            num5 += 1;
            if num5 > 300 {
                break;
            }
            let num6 = num3.min(num2);
            num3 -= num6;
            let step_vel = vector8 * num6;
            let tc = crate::collision::tile_collision(
                world,
                [p.pos.x, p.pos.y],
                [step_vel.x, step_vel.y],
                p.w,
                p.h,
                true,
                true,
            );
            let v10 = Vec2::new(tc.vel[0], tc.vel[1]);
            p.pos += v10;
            p.vel = v10;
            zero += p.vel;
        }
        p.vel = zero;
        if (p.vel.x - last_velocity.x).abs() < 0.0001 {
            p.vel.x = last_velocity.x;
        }
        if (p.vel.y - last_velocity.y).abs() < 0.0001 {
            p.vel.y = last_velocity.y;
        }
    } else {
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            true,
            true,
        );
        p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
    }
    if p.vel != last_velocity {
        // `:17948` — the consumed Next(3).
        let _ = rng.next_range(0, 3);
        // `:18619-18634` — the aiStyle-33 stick: only while still in flight
        // (`localAI[0] == 0`); the wet half folds dry (module doc).
        if p.local_ai[0] == 0.0 {
            p.pos += last_velocity;
            p.vel = Vec2::ZERO;
            p.local_ai[0] = 1.0;
            // The stick consumed the move — UpdatePosition would add the
            // (now zero) velocity, a no-op; return without the integrate.
            return;
        }
    }
    // `UpdatePosition` (`:20445`) — the dry integrate.
    p.pos += p.vel;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    /// A recording stream: every `next_range`/`next_float` push lands here so
    /// the witnesses pin the exact draw count + order (the ai026_c pattern).
    struct Recorder {
        draws: Vec<(i32, i32)>,
    }
    impl tr_core::loot::DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    fn glow(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("a glowstick row");
        // The struct-literal/update forms cannot span the pool-private
        // fields from a sibling module (E0451) — mutate the public surface.
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(320.0, 320.0);
        p.vel = Vec2::new(6.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.time_left = d.time_left;
        p.alpha = d.alpha;
        p.light = d.light;
        p
    }

    #[test]
    fn the_glowstick_rows_carry_the_csharp_defaults() {
        // `Projectile.cs:1326-1336` — 50: 6x6, aiStyle 14, pen −1, alpha 75,
        // light 1, timeLeft 18000, tileCollide true.
        let d = proj_data(GLOWSTICK).unwrap();
        assert_eq!((d.w, d.h, d.ai_style, d.penetrate), (6.0, 6.0, 14, -1));
        assert_eq!((d.alpha, d.time_left, d.tile_collide), (75, 18000, true));
        assert_eq!(d.light, 1.0);
        assert!(d.net_important);
        // 53 adds tileCollide false (`:1364`).
        assert!(!proj_data(STICKY_GLOWSTICK).unwrap().tile_collide);
        // 473: 8x8, timeLeft 7200 (`:5488`/`:5494`).
        let s = proj_data(SPELUNKER_GLOWSTICK).unwrap();
        assert_eq!((s.w, s.h, s.time_left), (8.0, 8.0, 7200));
        // 1089: no alpha override (`:10153-10162`), light 1.
        let r = proj_data(RAINBOW_GLOWSTICK).unwrap();
        assert_eq!((r.alpha, r.light, r.time_left), (0, 1.0, 18000));
    }

    #[test]
    fn the_flare_rows_carry_the_csharp_defaults() {
        // `Projectile.cs:2515-2525` — 163: 6x6, aiStyle 33, friendly, pen −1,
        // alpha 255, timeLeft 36000, NO light (the dust trail glows).
        let d = proj_data(FLARE).unwrap();
        assert_eq!((d.w, d.h, d.ai_style, d.penetrate), (6.0, 6.0, 33, -1));
        assert_eq!((d.alpha, d.time_left), (255, 36000));
        assert_eq!(d.light, 0.0);
        for k in FLARE_KINDS {
            assert_eq!(proj_data(k).unwrap().ai_style, 33, "kind {k}");
        }
    }

    #[test]
    fn the_glowstick_flight_waits_five_ticks_then_adds_gravity() {
        // `:25144-25158` — ai[0] counts to 5; the gravity waits for >5.
        let mut p = glow(GLOWSTICK);
        let w = air();
        let mut rng = Recorder { draws: vec![] };
        // Tick 1..5: no gravity yet (ai[0] reaches 5 on tick 5, the `>5`
        // gate is still false), the rotation rides velocity.X * 0.1.
        for _ in 0..5 {
            glowstick_step(&mut p, &w, &mut rng);
        }
        assert_eq!(p.ai[0], 5.0);
        // The flight is horizontal (vel.y starts 0) — no Y gravity until the
        // 6th tick opens the `>5` gate.
        assert_eq!(p.vel.y, 0.0);
        // Tick 6: gravity +0.2.
        glowstick_step(&mut p, &w, &mut rng);
        assert!((p.vel.y - 0.2).abs() < 1e-6);
        // No blocked move in open air → no Next(3) draw.
        assert!(rng.draws.is_empty());
    }

    #[test]
    fn the_plain_glowstick_bounces_x_at_minus_02_and_y_only_above_15() {
        // `:19673-19680` — the 50 bounce.
        let mut p = glow(GLOWSTICK);
        // X change → ×-0.2.
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(5.0, 0.0));
        assert!((p.vel.x - (-1.0)).abs() < 1e-6);
        // Y change with last.Y = 1.0 (≤1.5) → NO Y bounce.
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(0.0, 1.0));
        assert_eq!(p.vel.y, 0.0);
        // Y change with last.Y = 2.0 (>1.5) → ×-0.2.
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(0.0, 2.0));
        assert!((p.vel.y - (-0.4)).abs() < 1e-6);
    }

    #[test]
    fn the_bouncy_glowstick_bounces_both_axes_at_minus_09_above_1() {
        // `:18481-18488` — the 515-517 arm.
        let mut p = glow(BOUNCY_GLOWSTICK);
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(3.0, 3.0));
        assert!((p.vel.x - (-2.7)).abs() < 1e-6);
        assert!((p.vel.y - (-2.7)).abs() < 1e-6);
        // |last| ≤ 1 → no bounce.
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(0.5, 0.5));
        assert_eq!(p.vel, Vec2::ZERO);
    }

    #[test]
    fn the_rainbow_glowstick_reverses_at_099_with_the_small_snaps() {
        // `:19684-19701` — the 1089 bounce.
        let mut p = glow(RAINBOW_GLOWSTICK);
        // A large X change → ×0.99 reverse, no snap.
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(5.0, 0.0));
        assert!((p.vel.x - (-4.95)).abs() < 1e-5);
        // A small X change (|Δ| < 1) → snap to ±1 by the new sign.
        p.vel = Vec2::new(4.5, 0.0);
        light_bounce(&mut p, Vec2::new(5.0, 0.0));
        assert_eq!(p.vel.x, -1.0);
        // A small positive Y (0 < last.Y < 1) → snap to −1.
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(0.0, 0.5));
        assert_eq!(p.vel.y, -1.0);
    }

    #[test]
    fn the_fairy_glowstick_flips_to_the_hover_on_its_first_bounce() {
        // `:19663-19669` — the full reverse + ai[1] = 1.
        let mut p = glow(FAIRY_GLOWSTICK);
        p.vel = Vec2::new(0.0, 0.0);
        light_bounce(&mut p, Vec2::new(4.0, -2.0));
        assert_eq!(p.vel, Vec2::new(-4.0, 2.0));
        assert_eq!(p.ai[1], 1.0);
        // The next head tick routes to the hover: ai[1] == 1 zeroes X and
        // latches 2 (`:45670-45675`).
        let w = air();
        fairy_glowstick_hover(&mut p, &w);
        assert_eq!(p.vel.x, 0.0);
        assert_eq!(p.ai[1], 2.0);
    }

    #[test]
    fn the_sticky_glowstick_sticks_to_a_solid_tile() {
        // `:24911-24962` — the stick scan zeroes X, pins Y = −0.2.
        let mut w = air();
        // A stone block at tile (20, 20) = px (320, 320).
        let i = w.index(20, 20);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = glow(STICKY_GLOWSTICK);
        p.pos = Vec2::new(322.0, 322.0); // overlapping the block
        p.vel = Vec2::new(3.0, 3.0);
        glowstick_head(&mut p, &w);
        assert_eq!(p.vel.x, 0.0);
        assert_eq!(p.vel.y, -0.2);
        assert!(p.active);
    }

    #[test]
    fn the_sticky_glowstick_dies_on_a_rainbow_boulder() {
        // `:24951-24954` — BreaksToys[740] → Kill.
        let mut w = air();
        let i = w.index(20, 20);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 740,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = glow(STICKY_GLOWSTICK);
        p.pos = Vec2::new(322.0, 322.0);
        glowstick_head(&mut p, &w);
        assert!(!p.active);
    }

    #[test]
    fn the_blocked_move_consumes_exactly_one_next_3() {
        // `:17948` — the cut roll is the first operand, consumed on every
        // blocked move (the ai026_c precedent).
        let mut w = air();
        // A wall of stone at tile x = 21 (px 336) — the glowstick at px 330
        // moving +6 X blocks against it.
        for y in 0..64 {
            let i = w.index(21, y);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = glow(GLOWSTICK);
        p.pos = Vec2::new(330.0, 320.0);
        p.vel = Vec2::new(6.0, 0.0);
        let mut rng = Recorder { draws: vec![] };
        light_movement(&mut p, &w, &mut rng);
        assert_eq!(rng.draws, vec![(0, 3)], "exactly the :17948 Next(3)");
    }

    #[test]
    fn the_flare_fades_alpha_by_50_and_snapshots_its_velocity() {
        // `:27051-27058` + `:27106-27115`.
        let mut p = glow(FLARE);
        p.alpha = 255;
        p.vel = Vec2::new(6.0, 0.0);
        let w = air();
        let mut rng = Recorder { draws: vec![] };
        flare_step(&mut p, &w, &mut rng);
        assert_eq!(p.alpha, 205);
        // In flight: ai[0]/ai[1] snapshot the velocity, localAI[1] counts.
        assert_eq!(p.ai[0], 6.0);
        assert_eq!(p.ai[1], 0.0);
        assert_eq!(p.local_ai[1], 1.0);
        // The rotation is atan2(ai[1], ai[0]) + 1.57 = 0 + 1.57.
        assert!((p.rot - 1.57).abs() < 1e-6);
    }

    #[test]
    fn the_flare_sticks_on_a_blocked_move_and_zeroes_damage() {
        // `:18619-18634` (the stick) + `:27128` (damage = 0 once stuck).
        let mut w = air();
        for y in 0..64 {
            let i = w.index(21, y);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = glow(FLARE);
        p.pos = Vec2::new(330.0, 320.0);
        p.vel = Vec2::new(6.0, 0.0);
        p.damage = 5;
        let mut rng = Recorder { draws: vec![] };
        flare_movement(&mut p, &w, &mut rng);
        // Stuck: localAI[0] = 1, velocity zero, position advanced by the
        // last velocity (the dry stick branch).
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.vel, Vec2::ZERO);
        assert_eq!(rng.draws, vec![(0, 3)]);
        // The next AI tick sees the stuck state and zeroes the damage.
        flare_step(&mut p, &w, &mut rng);
        assert_eq!(p.damage, 0);
    }

    #[test]
    fn the_light_item_arms_draw_no_rand_in_open_air() {
        // The AI arms are RNG-free (the dust skips); only a blocked move
        // rolls. A full flight tick in open air consumes nothing.
        let w = air();
        for k in GLOWSTICK_KINDS {
            let mut p = glow(k);
            let mut rng = Recorder { draws: vec![] };
            glowstick_step(&mut p, &w, &mut rng);
            assert!(rng.draws.is_empty(), "kind {k} drew in open air");
        }
        for k in FLARE_KINDS {
            let mut p = glow(k);
            let mut rng = Recorder { draws: vec![] };
            flare_step(&mut p, &w, &mut rng);
            assert!(rng.draws.is_empty(), "kind {k} drew in open air");
        }
    }

    // ---- the W4-g78 spelunker attach witnesses (research/77 §1) ----

    /// A scripted stream handing back the Next(4) gate draws in order (the
    /// spelunker_helper Scripted precedent) — the attach path's roll count
    /// is pinned without a seed hunt.
    struct SpotRng {
        draws: Vec<i32>,
        pos: usize,
        ranges: Vec<(i32, i32)>,
    }
    impl tr_core::loot::DropRng for SpotRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            let d = self.draws[self.pos];
            self.pos += 1;
            d
        }
        fn next_float(&mut self) -> f32 {
            unreachable!("the spot path draws no floats")
        }
    }

    fn ore_world() -> World {
        let mut w = World::new("t".into(), 80, 80);
        let i = w.index(40, 40);
        w.tiles[i].kind = 6; // copper ore — spelunkable
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        w
    }

    /// `Projectile.cs:24836-24847` — the 473 counter-wrap calls
    /// AddSpotToCheck on the 10th tick ONLY, behind the distance gate; the
    /// helper's Next(4) rides the caller's stream. Ticks 1-9 draw nothing.
    #[test]
    fn w4_g78_the_473_head_attaches_on_the_counter_wrap() {
        let w = ore_world();
        let mut p = glow(SPELUNKER_GLOWSTICK);
        p.pos = Vec2::new(40.0 * 16.0, 40.0 * 16.0); // centre tile (40, 40)
        p.vel = Vec2::ZERO;
        let mut helper = crate::spelunker_helper::SpelunkerProjectileHelper::default();
        let mut out = Vec::new();
        let mut rng = SpotRng { draws: vec![0], pos: 0, ranges: vec![] };
        for _ in 0..9 {
            let spot = SpelunkerSpot {
                helper: &mut helper,
                player_center: Some(p.pos),
                screen_width: 1280.0,
                net_mode_ok: true,
                out: &mut out,
            };
            glowstick_head_inner(&mut p, &w, Some(spot), &mut rng);
        }
        assert_eq!(rng.pos, 0, "ticks 1-9: no wrap, no rolls");
        assert!(out.is_empty());
        let spot = SpelunkerSpot {
            helper: &mut helper,
            player_center: Some(p.pos),
            screen_width: 1280.0,
            net_mode_ok: true,
            out: &mut out,
        };
        glowstick_head_inner(&mut p, &w, Some(spot), &mut rng);
        assert_eq!(p.local_ai[0], 0.0, "the counter wrapped");
        assert_eq!(rng.ranges, vec![(0, 4)], "ONE Next(4) for the new ore");
        assert_eq!(out.len(), 1, "the 0-roll spawned the 204 request");
        assert_eq!(out[0].dust_type, 204);
        assert_eq!(out[0].fade_in, 0.75);
    }

    /// `:24836` — `Main.netMode != 2` gates the WHOLE 473 block: on the
    /// server the counter does not even tick (the C# skips the increment).
    #[test]
    fn w4_g78_the_server_skips_the_whole_473_block() {
        let w = ore_world();
        let mut p = glow(SPELUNKER_GLOWSTICK);
        let mut helper = crate::spelunker_helper::SpelunkerProjectileHelper::default();
        let mut out = Vec::new();
        let mut rng = SpotRng { draws: vec![], pos: 0, ranges: vec![] };
        for _ in 0..12 {
            let spot = SpelunkerSpot {
                helper: &mut helper,
                player_center: Some(p.pos),
                screen_width: 1280.0,
                net_mode_ok: false,
                out: &mut out,
            };
            glowstick_head_inner(&mut p, &w, Some(spot), &mut rng);
        }
        assert_eq!(p.local_ai[0], 0.0, "the counter never ticks on the server");
        assert_eq!(rng.pos, 0);
        assert!(out.is_empty());
    }

    /// `:27124-27127` — the 1008 stuck arm calls AddSpotToCheck EVERY stuck
    /// tick (no counter); the position dedupe means the SECOND tick rolls
    /// nothing (the helper's `_positionsChecked`), and the damage zeroes.
    #[test]
    fn w4_g78_the_1008_stuck_arm_checks_every_tick_deduped() {
        let mut w = ore_world();
        // The flare's 6x6 box at px (640, 640) sits INSIDE tile (40, 40):
        // make that cell stone so SolidCollision keeps it stuck; move the
        // ore to (41, 40) — still inside the ±30 window + radius.
        let i = w.index(40, 40);
        w.tiles[i].kind = 1;
        let i = w.index(41, 40);
        w.tiles[i].kind = 6;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = glow(SPELUNKER_FLARE);
        p.pos = Vec2::new(40.0 * 16.0, 40.0 * 16.0);
        p.vel = Vec2::ZERO;
        p.local_ai[0] = 1.0; // stuck
        p.local_ai[1] = 30.0;
        let mut helper = crate::spelunker_helper::SpelunkerProjectileHelper::default();
        let mut out = Vec::new();
        let mut rng = SpotRng { draws: vec![0], pos: 0, ranges: vec![] };
        let spot = SpelunkerSpot {
            helper: &mut helper,
            player_center: Some(Vec2::new(40.0 * 16.0, 40.0 * 16.0)),
            screen_width: 1280.0,
            net_mode_ok: true,
            out: &mut out,
        };
        flare_step_with_spot(&mut p, &w, &mut rng, Some(spot));
        assert_eq!(p.local_ai[0], 1.0, "the flare stays stuck on the stone");
        assert_eq!(rng.pos, 1, "the first stuck tick scans");
        assert_eq!(out.len(), 1);
        assert_eq!(p.damage, 0, "the stuck arm zeroes the damage");
        // The second stuck tick: the SAME centre is deduped — no rolls.
        let spot = SpelunkerSpot {
            helper: &mut helper,
            player_center: Some(Vec2::new(40.0 * 16.0, 40.0 * 16.0)),
            screen_width: 1280.0,
            net_mode_ok: true,
            out: &mut out,
        };
        flare_step_with_spot(&mut p, &w, &mut rng, Some(spot));
        assert_eq!(rng.pos, 1, "the repeat position rolls nothing");
        assert_eq!(out.len(), 1);
    }

    /// The distance gate at the attach seam: a player beyond
    /// screenWidth + 480 px refuses the scan (no rolls, no dust) even on
    /// the counter wrap.
    #[test]
    fn w4_g78_the_attach_distance_gate_refuses_far_players() {
        let w = ore_world();
        let mut p = glow(SPELUNKER_GLOWSTICK);
        p.pos = Vec2::new(40.0 * 16.0, 40.0 * 16.0);
        p.local_ai[0] = 9.0; // wraps this tick
        let mut helper = crate::spelunker_helper::SpelunkerProjectileHelper::default();
        let mut out = Vec::new();
        let mut rng = SpotRng { draws: vec![], pos: 0, ranges: vec![] };
        let spot = SpelunkerSpot {
            helper: &mut helper,
            // 2000 px away > 1280 + 480 = 1760.
            player_center: Some(Vec2::new(40.0 * 16.0 + 2000.0, 40.0 * 16.0)),
            screen_width: 1280.0,
            net_mode_ok: true,
            out: &mut out,
        };
        glowstick_head_inner(&mut p, &w, Some(spot), &mut rng);
        assert_eq!(p.local_ai[0], 0.0, "the counter still wraps");
        assert_eq!(rng.pos, 0, "the gate refused the scan");
        assert!(out.is_empty());
    }
}