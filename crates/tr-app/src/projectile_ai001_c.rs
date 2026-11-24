//! S1-D-5 slice C — `AI_001`'s per-type arms for the MID-range rowed kinds
//! (`Projectile.cs:54002-57798`, the aiStyle-1 inline region). Slice A
//! (`projectile_ai001_a`, landed) owns the 14 low-id kinds
//! (14/81/83/98/100/180/184/248/259/270/325/389/436/438); slice B
//! (`projectile_ai001_b`, landed) owns 449/462/581/601/622/680/686/962/980.
//! This module takes every OTHER rowed aiStyle-1 kind of
//! `projectile/data.rs`'s census that has a real per-type `AI_001` branch —
//! the kind set is [`SLICE_C_KINDS`] and the dispatch seam in
//! `ProjectilePool::update`'s aiStyle-1 arm chains this gate AFTER A's and
//! B's; every other kind keeps its existing arm BYTE-IDENTICAL.
//!
//! Kind set (9): 110 BulletSnowman, 275 SeedPlantera, 276 PoisonSeedPlantera,
//! 634 NebulaBlaze1, 635 NebulaBlaze2, 645 PoisonDart, 662 Dd2Spear,
//! 675 DarkMageFlame, 685 Dd2SpearT3. B deferred 634/635 ("RNG-heavy") and
//! 645 ("placed-cloud state machine") — both land here: the blaze's only
//! state-feeding rand (`localAI[0] = -Next(48)`, `:54487`) feeds dust alone
//! (`:54583`), so the dust-skip convention covers it (see the RNG note).
//!
//! Census escape clause (already faithful on main, NOT claimed here):
//! - 1/2 WoodenArrow/FlameArrow: the generic `ai_arrow` flight IS their C#
//!   arm (`:57568-57579` gravity + `:57750` rotation; the port's omitted
//!   ai[0] pin is the documented generic-path convention).
//! - 20/88/355/660: `ai_style1_rider` (M4-PROJ-6, already per-type).
//! - 837: `ai_book_of_skulls` (already per-type).
//! - 178 CannonBomb: `AI_001` has ZERO `type == 178` references (verified
//!   by reading `:54002-57798`); with `timeLeft = 2` (`SetDefaults`
//!   `:2641-2649`) the generic 15-tick gravity is unreachable — the generic
//!   flight is byte-faithful for its whole 2-tick life.
//! Data deviations deferred (as slice A deferred 185): 185 TrapSpikyBall is
//! aiStyle **14** in C# (`:2717-2727`); 687 BetsyBreath is aiStyle **136**
//! (`:7315-7324`) — both rowed aiStyle 1 in `projectile/data.rs`, a separate
//! data-fix task.
//!
//! Structure mirrors `AI_001`'s own order per kind: the hook group above
//! the counter (`:54066-55671`), the `flag3` counter switch
//! (`:55945-56041`), the hook group below it (`:56056-56709`), the
//! pre-rotation/gravity else-if chain (`:56753-57580`), the rotation tail
//! (`:57582-57752`), the clamp (`:57753-57768`) and the movement stand-in.
//!
//! RNG: the module takes NO rng parameter, so the `UpdateProjectiles`
//! stream (`Main.cs:17775-17777`) is untouched BY CONSTRUCTION. Every
//! `Main.rand` draw in these kinds' C# blocks feeds dust/sound only:
//! - 634/635: the dust-type `Utils.SelectRandom` picks (`:54472`/`:54482`),
//!   `localAI[0] = -Main.rand.Next(48)` (`:54487` — its ONLY reader is the
//!   dust orbit at `:54583`; the port keeps the `localAI[0]++`/48-wrap
//!   machine, `:54573-54577`, seeded at 0), and the dust gates
//!   `:54591`/`:54602`/`:54616`/`:54631`/`:54640`.
//! - 645: the per-tick dust 229 spawn (`:56968`, `Dust.NewDust`'s internal
//!   jitter draws) — the `flag6` solid-tile read (`:56967`) feeds only
//!   `dust.noLight`, so it is skipped with the dust.
//! - 110/275/276/662/675/685: no `Main.rand` at all in their blocks.
//!
//! CODE-ONLY MODE (S1-D-5 slice C): the witness tests at the bottom are
//! WRITTEN NOT RUN (no compiler invoked on this branch); they were verified
//! by re-reading the cited C# blocks and hand arithmetic.
//!
//! Deliberate skips (recorded in ANALYSIS "aiStyle-1 per-type arms, slice C"):
//! - Dust, light (`Lighting.AddLight` `:54571`) and `netUpdate` — no such
//!   layers. SOUND now rides the `wants_sound` seam (S1-E-21h d76): the
//!   634/635 blaze Item34 (`:54488`), the 110 Item11 (`:55247`) and the
//!   275/276 Item17 (`:55339`/`:55344`). The 675 Dark Mage's
//!   `DD2_DarkMageAttack` (`:56466`) is a PlayTrackedSound — the D3
//!   trackable family, DEFERRED (its latch rides cue-less).
//! - 275/276's expert homing (`:57212-57231`): the port is single-difficulty
//!   NORMAL (`tr-core/src/world.rs:772`), so `Main.expertMode` is const
//!   false and the branch is unreachable — skipped by construction.
//! - 634/635's `owner == Main.myPlayer` scan gate (`:54490`): the pool only
//!   dispatches locally-simulated slots (projectile.rs's `simulated` gate)
//!   and both kinds are friendly, so the gate is always true at the seam.
//! - The clamp is the C#'s `flag8` tail EXACTLY (`:57753-57768`, slice A's
//!   convention): only the +16 bound — the −16 bound is `flag4`'s
//!   (kind 1006 only) and no slice-C kind sets it. The legacy generic
//!   flight's ±16 `clamp` (projectile.rs:5250) is NOT copied here; the
//!   per-type arms take the exact C# bound like slice A.
//! - Movement is the port's documented kill-on-tile stand-in
//!   (`HandleMovement`'s default arm `:19843-19848`), replicated verbatim
//!   from `ai_style1_flight`'s tail as slices A/B do.

use glam::Vec2;
use tr_core::World;

use crate::npc::Npc;
use crate::projectile::{
    can_hit_line, chaseable, proj_frames, Projectile, BULLET_SNOWMAN, DARK_MAGE_FLAME, DD2_SPEAR,
    DD2_SPEAR_T3, NEBULA_BLAZE_1, NEBULA_BLAZE_2, POISON_DART, POISON_SEED_PLANTERA,
    SEED_PLANTERA,
};

/// The slice-C kind set — the mid-range rowed kinds this module owns,
/// disjoint from `projectile_ai001_a::SLICE_A_KINDS` and
/// `projectile_ai001_b::SLICE_B_KINDS` (the seam chains C after A and B).
pub(crate) const SLICE_C_KINDS: [u16; 9] = [
    BULLET_SNOWMAN,        // 110
    SEED_PLANTERA,         // 275
    POISON_SEED_PLANTERA,  // 276
    NEBULA_BLAZE_1,        // 634
    NEBULA_BLAZE_2,        // 635
    POISON_DART,           // 645
    DD2_SPEAR,             // 662
    DARK_MAGE_FLAME,       // 675
    DD2_SPEAR_T3,          // 685
];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    SLICE_C_KINDS.contains(&kind)
}

/// The per-kind gravity arm of the `:56888-57580` else-if chain.
enum Gravity {
    /// No arm fires: 110 rests its counter (`flag3 = false`, `:55962`);
    /// 675 is EXCLUDED from the chain's generic tail by the `:57520`
    /// `type != 675` guard even though its counter runs.
    None,
    /// `:57197-57211` — the seeds: alpha fade 30, then from `ai[0] >= 35`
    /// the pin at 35 and `velocity.Y += 0.025f`. (The expert homing
    /// `:57212-57231` is unreachable in the normal-mode port.)
    Seed,
    /// `:57471-57478` (the 507/508/662/680/685 arm) — from `ai[0] > 60`:
    /// `velocity.X *= 0.98f; velocity.Y += 0.3f`. No counter pin.
    Spear,
}

/// The per-kind rotation tail of the `:57582-57752` chain.
enum Rot {
    /// `:57748-57751` — `rotation = Atan2(vy, vx) + 1.57f` (the literal
    /// 1.57, not π/2 — the C# spells the constant).
    Default,
    /// `:57654-57657` — `rotation = ToRotation() - π - π/4` (the DD2
    /// spears; the C# subtracts left-to-right).
    Spear,
    /// `:57628-57631` — `rotation += velocity.X * 0.03f` (the Dark Mage
    /// flame ACCUMULATES its spin).
    DarkMage,
}

/// `AI_001`'s slice-C body for one slot, in the C#'s own block order.
/// `npcs` is the pool's NPC slice (the blaze's acquisition/homing scans,
/// `:54494-54506`/`:54522-54544`, and the dart's placed-cloud scan,
/// `:57006-57018`); `num_updates` is the C# `this.numUpdates` value the
/// sub-step sees — the seam computes `sub_steps - 2 - sub` because the C#
/// decrements BEFORE the body (`Projectile.cs:16675-16678` then `AI()` at
/// `:16775`), so `numUpdates == 0` is the SECOND-TO-LAST sub-step (the
/// dart's scan gate, `:57002`).
pub(crate) fn step(p: &mut Projectile, world: &World, npcs: &[Npc], num_updates: i32) {
    match p.kind {
        BULLET_SNOWMAN => {
            // `:55244-55248` — `ai[1] = 1f` + `SoundID.Item11` (S1-E-21h d76).
            latch_ai1(p, 11);
            // `:55613` + `:55626-55633` — the alpha switch: `alpha -= 15`
            // while positive, clamp 0.
            fade_alpha(p, 15);
            // `flag3 = false` (`:55962`): the counter rests, the generic
            // gravity never fires — a straight bullet, like the musket ball.
            finish(p, world, false, Gravity::None, Rot::Default);
        }
        SEED_PLANTERA | POISON_SEED_PLANTERA => {
            // `:55012-55024` — the 2-frame flutter: `frameCounter++`, at
            // `> 1`: reset, `frame++`, wrap `> 1` to 0
            // (`Main.projFrames[275/276] = 2`, `Main.cs:8574-8575`).
            p.frame_counter += 1;
            if p.frame_counter > 1 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame > 1 {
                    p.frame = 0;
                }
            }
            // `:55336-55345` — `ai[1] = 1f` + `SoundID.Item17` (S1-E-21h d76).
            latch_ai1(p, 17);
            // 275/276 are NOT on the flag3 skip list — they count into the
            // 35/0.025 gravity arm (`:57197-57211`).
            finish(p, world, true, Gravity::Seed, Rot::Default);
        }
        NEBULA_BLAZE_1 | NEBULA_BLAZE_2 => blaze(p, world, npcs),
        POISON_DART => poison_dart(p, world, npcs, num_updates),
        DD2_SPEAR | DD2_SPEAR_T3 => {
            // No hook group: the only per-type blocks are the shared
            // 507/508/662/680/685 gravity arm (`:57471-57478`) and the
            // 662/685 rotation tail (`:57654-57657`). Not on the flag3
            // skip list — the counter runs.
            finish(p, world, true, Gravity::Spear, Rot::Spear);
        }
        DARK_MAGE_FLAME => {
            // `:56461-56473` (below the counter in the C#, but the counter
            // and this hook commute — no shared state): the localAI[0]
            // one-shot (`PlayTrackedSound(DD2_DarkMageAttack)`, `:56466` —
            // W4-d80) and `alpha -= 20`, clamp 0.
            if p.local_ai[0] == 0.0 {
                p.local_ai[0] = 1.0;
                p.wants_tracked = Some(crate::projectile::ProjTrackedCue::OneShot("DD2_DarkMageAttack"));
            }
            fade_alpha(p, 20);
            // Not on the flag3 skip list — the counter runs, but the
            // `:57520` chain guard (`type != 675`) excludes the generic
            // gravity: the flame never falls.
            finish(p, world, true, Gravity::None, Rot::DarkMage);
        }
        _ => unreachable!("the dispatch gates on SLICE_C_KINDS"),
    }
}

/// The `ai[1] == 0f → ai[1] = 1f` first-tick latch. S1-E-21h d76: the
/// arm's `SoundID.ItemN` cue queues with the latch at `base.position`
/// (110→Item11 `:55247`, 275/276→Item17 `:55339`/`:55344`); the latch
/// state gates the cue to one play in C#.
fn latch_ai1(p: &mut Projectile, style: i32) {
    if p.ai[1] == 0.0 {
        p.ai[1] = 1.0;
        p.wants_sound = Some((2, style, p.pos.x, p.pos.y, 0.0, 1.0));
    }
}

/// The C# `alpha -= n` with the `< 0 → 0` clamp (the int alpha cannot
/// underflow the port's i32, so the unconditional subtract is faithful).
fn fade_alpha(p: &mut Projectile, n: i32) {
    p.alpha -= n;
    if p.alpha < 0 {
        p.alpha = 0;
    }
}

/// The port's `Kill()` stand-in for the AI_001 early-kill paths — the same
/// deactivation the movement tail's tile hit uses (projectile.rs:5258-5259).
fn kill(p: &mut Projectile) {
    p.active = false;
    p.time_left = 0;
}

/// The 634/635 Nebula Blaze block (`Projectile.cs:54463-54650`). The
/// per-kind constants (`:54465-54483`): the retry cap `num26 = 5`, the
/// acquisition reach 250/500, the steer speed 6, the frame cadence
/// `4*MaxUpdates` / `3*MaxUpdates` (both 12 with the rows' extraUpdates
/// 2/3 — the formula is kept, not the folded value). The dust-type
/// `SelectRandom` picks (`:54472`/`:54482`), the light (`:54571`) and every
/// dust block (`:54578-54649`) are SKIPPED; `localAI[0] = -Next(48)`
/// (`:54487`) is skipped with them (dust-only reader, `:54583`) while the
/// `localAI[0]++`/48-wrap machine (`:54573-54577`) rides.
fn blaze(p: &mut Projectile, world: &World, npcs: &[Npc]) {
    let is_2 = p.kind == NEBULA_BLAZE_2;
    let retry_cap = 5.0f32; // num26, `:54465`
    let reach = if is_2 { 500.0 } else { 250.0 }; // num27, `:54466`/`:54478`
    let steer_speed = 6.0f32; // num28, `:54467`
    // num30, `:54471`/`:54480` — `MaxUpdates = extraUpdates + 1`
    // (`Projectile.cs:554-561`).
    let frame_every = (if is_2 { 3 } else { 4 } * (p.extra_updates + 1)) as i32;
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    if p.ai[1] == 0.0 {
        // `:54484-54489` — the first-tick latch; the rand seed
        // (`localAI[0] = -Next(48)`) stays SKIPPED (dust-only reader), the
        // Item34 cue queues at base.position (S1-E-21h d76, `:54488`).
        p.ai[1] = 1.0;
        p.wants_sound = Some((2, 34, p.pos.x, p.pos.y, 0.0, 1.0));
    } else if p.ai[1] == 1.0 {
        // `:54490-54518` — the acquisition scan. The `owner ==
        // Main.myPlayer` gate folds into the pool's locally-simulated
        // dispatch (module doc). C# QUIRK kept (`:54500`): the
        // `num33 == -1` term means the FIRST chaseable NPC inside the
        // reach with a clear line wins — later, nearer NPCs never
        // replace it (unlike the dart's true nearest scan).
        let mut found: Option<usize> = None;
        let mut best = reach;
        for (idx, n) in npcs.iter().enumerate() {
            if !chaseable(n) {
                continue;
            }
            let (nw, nh) = n.size();
            let (nx, ny) = (n.x + nw * 0.5, n.y + nh * 0.5);
            let d = (cx - nx).hypot(cy - ny);
            if d < best
                && found.is_none()
                && can_hit_line(world, cx, cy, 1.0, 1.0, nx, ny, 1.0, 1.0)
            {
                best = d;
                found = Some(idx);
            }
        }
        // `:54507-54511` — inside 20 px of the picked target the blaze
        // detonates: `Kill(); return;` (no rotation/clamp/movement ride).
        if best < 20.0 {
            kill(p);
            return;
        }
        // `:54512-54517` — the lock: `ai[1] = num26 + 1`, `ai[0] = index`
        // (netUpdate SKIPPED).
        if let Some(idx) = found {
            p.ai[1] = retry_cap + 1.0;
            p.ai[0] = idx as f32;
        }
    } else if p.ai[1] > retry_cap {
        // `:54519-54545` — the held-target steer.
        p.ai[1] += 1.0;
        let idx = p.ai[0] as usize;
        let target = npcs.get(idx).filter(|n| chaseable(n)).map(|n| {
            let (nw, nh) = n.size();
            (n.x + nw * 0.5, n.y + nh * 0.5)
        });
        match target {
            None => {
                // `:54523-54528` — the target died / stopped being
                // chaseable: back to the retry window.
                p.ai[1] = 1.0;
                p.ai[0] = 0.0;
            }
            Some((nx, ny)) => {
                let mut toward = Vec2::new(nx - cx, ny - cy);
                let len = toward.length();
                // `:54533-54537` — the detonation radius again.
                if len < 20.0 {
                    kill(p);
                    return;
                }
                // `:54538-54544` — normalize, scale to 6, blend at the
                // weight 30: `velocity = (velocity * 29 + toward) / 30`.
                if len > 0.0 {
                    toward = toward / len * steer_speed;
                }
                p.vel = (p.vel * 29.0 + toward) / 30.0;
            }
        }
    }
    // `:54547-54554` — the retry window: `1 <= ai[1] < 5` counts up; at 5
    // it wraps to 1 (a fresh scan next tick).
    if p.ai[1] >= 1.0 && p.ai[1] < retry_cap {
        p.ai[1] += 1.0;
        if p.ai[1] == retry_cap {
            p.ai[1] = 1.0;
        }
    }
    // `:54555-54559` — `alpha -= 40`, clamp 0.
    fade_alpha(p, 40);
    // `:54560` — `spriteDirection = base.direction` (the velocity-sign
    // stand-in, slices A/B's convention).
    p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
    // `:54561-54570` — the frame machine at num30, wrapping at 4
    // (`proj_frames` 634/635 = 4, data.rs).
    p.frame_counter += 1;
    if p.frame_counter >= frame_every {
        p.frame += 1;
        p.frame_counter = 0;
        if p.frame >= 4 {
            p.frame = 0;
        }
    }
    // `:54572` — `rotation = velocity.ToRotation()`; the default tail
    // (`:57750`) overwrites it in the SAME C# tick — kept for order
    // fidelity (the 962 one-shot precedent, slice B).
    p.rot = p.vel.y.atan2(p.vel.x);
    // `:54573-54577` — the dust-orbit phase counter rides (its rand seed
    // is skipped, module doc).
    p.local_ai[0] += 1.0;
    if p.local_ai[0] == 48.0 {
        p.local_ai[0] = 0.0;
    }
    // `flag3 = false` (`:56016-56017`) and the `:57520` chain guard
    // (`type != 634 && type != 635`) — no counter, no gravity.
    finish(p, world, false, Gravity::None, Rot::Default);
}

/// The 645 PoisonDart block (`Projectile.cs:56956-57027`) — the
/// thrown-dart / placed-cloud state machine. `ai[1]` carries the launch
/// height threshold while flying and `-1` once the cloud is placed.
fn poison_dart(p: &mut Projectile, world: &World, npcs: &[Npc], num_updates: i32) {
    // `:56958-56961` — below the launch height the dart regains tile
    // collision.
    if p.ai[1] != -1.0 && p.pos.y > p.ai[1] {
        p.tile_collide = true;
    }
    // `flag3` (`:56038-56041`): 645 is NOT on the skip list — the counter
    // increment runs BEFORE this block in the C# (so it rides even when
    // the NaN kill below fires); the cloud adds its own second increment.
    p.ai[0] += 1.0;
    // `:56962-56966` — `position.HasNaNs()` (XNA: NaN or infinity on
    // either component) kills.
    if p.pos.x.is_nan() || p.pos.y.is_nan() || p.pos.x.is_infinite() || p.pos.y.is_infinite() {
        kill(p);
        return;
    }
    // `:56967-56975` — the flag6 solid-tile read feeds only the dust's
    // noLight; the dust 229 spawn is SKIPPED with it (module doc).
    if p.ai[1] == -1.0 {
        // `:56976-56999` — the placed cloud.
        p.ai[0] += 1.0; // `:56978` — the SECOND increment (flag3 above).
        p.vel = Vec2::ZERO;
        p.tile_collide = false;
        p.penetrate = -1;
        // `:56982-56984` — `position = Center; width = height = 140;
        // Center = position` — the box grows to 140×140 about the old
        // center.
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        p.w = 140.0;
        p.h = 140.0;
        p.pos = Vec2::new(cx - 70.0, cy - 70.0);
        // `:56985-56989` — `alpha -= 10`, clamp 0.
        fade_alpha(p, 10);
        // `:56990-56994` — `++frameCounter >= MaxUpdates * 3` → reset,
        // `frame++` (NO wrap in the C# here — kept).
        p.frame_counter += 1;
        let cadence = ((p.extra_updates + 1) * 3) as i32;
        if p.frame_counter >= cadence {
            p.frame_counter = 0;
            p.frame += 1;
        }
        // `:56995-56998` — the cloud's life:
        // `ai[0] >= projFrames[645] * MaxUpdates * 3` (7 × 18 = 126 with
        // the row's extraUpdates 5; `Main.cs:8499`, data.rs `645 => 7`).
        if p.ai[0] >= (proj_frames(p.kind) * (p.extra_updates + 1) * 3) as f32 {
            kill(p);
            return;
        }
        // `:56999` — `return`: the rotation tail and clamp are skipped,
        // but `HandleMovement` still runs (vel = 0, tileCollide = false →
        // a no-op integration). The port's movement stand-in rides.
        move_only(p, world);
        return;
    }
    // `:57001` — the flying dart is fully opaque.
    p.alpha = 255;
    // `:57002-57026` — on `numUpdates == 0` (the SECOND-TO-LAST sub-step,
    // module doc) scan for a chaseable NPC within 60 px with a clear
    // line; this scan IS a true nearest (`:57012` updates the running
    // minimum without the blaze's first-wins quirk).
    if num_updates == 0 {
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        let mut best = 60.0f32;
        let mut found = false;
        for n in npcs {
            if !chaseable(n) {
                continue;
            }
            let (nw, nh) = n.size();
            let (nx, ny) = (n.x + nw * 0.5, n.y + nh * 0.5);
            let d = (cx - nx).hypot(cy - ny);
            if d < best && can_hit_line(world, cx, cy, 0.0, 0.0, nx, ny, 0.0, 0.0) {
                best = d;
                found = true;
            }
        }
        if found {
            // `:57019-57024` — place the cloud: `ai[0] = 0`, `ai[1] = -1`,
            // `return` (netUpdate SKIPPED; movement still rides).
            p.ai[0] = 0.0;
            p.ai[1] = -1.0;
            move_only(p, world);
            return;
        }
    }
    // The 645 arm consumed the gravity chain (`:56956`) — the generic
    // `:57568` tail never fires; the default rotation (`:57750`) does.
    finish(p, world, false, Gravity::None, Rot::Default);
}

/// The shared tail: the `flag3` counter (`:56038-56041`), the per-kind
/// gravity arm, the rotation tail (`:57582-57752`), the clamp
/// (`:57753-57768` — the exact C# +16 bound, module doc), then the
/// movement stand-in.
fn finish(p: &mut Projectile, world: &World, count_flight: bool, gravity: Gravity, rot: Rot) {
    // `Projectile.cs:56038-56041` — `if (flag3) ai[0]++`.
    if count_flight {
        p.ai[0] += 1.0;
    }
    match gravity {
        Gravity::None => {}
        // `Projectile.cs:57199-57211` — the fade rides INSIDE the arm,
        // before the counter test (the C#'s order).
        Gravity::Seed => {
            fade_alpha(p, 30);
            if p.ai[0] >= 35.0 {
                p.ai[0] = 35.0;
                p.vel.y += 0.025;
            }
        }
        // `Projectile.cs:57473-57477`.
        Gravity::Spear => {
            if p.ai[0] > 60.0 {
                p.vel.x *= 0.98;
                p.vel.y += 0.3;
            }
        }
    }
    // The rotation tail chain, in the C#'s order BEFORE the clamp.
    let angle = p.vel.y.atan2(p.vel.x);
    match rot {
        // `Projectile.cs:57750`.
        Rot::Default => p.rot = angle + 1.57,
        // `Projectile.cs:57656` — `ToRotation() - π - π/4`, left-to-right.
        Rot::Spear => p.rot = angle - std::f32::consts::PI - std::f32::consts::FRAC_PI_4,
        // `Projectile.cs:57630`.
        Rot::DarkMage => p.rot += p.vel.x * 0.03,
    }
    // `Projectile.cs:57753-57768` — `flag8` is true for every slice-C
    // kind (only 577 opts out); `flag4` is false, so just the +16 bound
    // (slice A's exact-tail convention).
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    move_only(p, world);
}

/// The movement stand-in alone — a verbatim replication of
/// `ai_style1_flight`'s integration (projectile.rs:5252-5266, the
/// `HandleMovement` default arm `Projectile.cs:19843-19848`). The 645
/// early-return paths use it directly: the C# `return`s skip the rotation
/// tail and clamp but NOT the movement.
fn move_only(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.active = false;
            p.time_left = 0;
        } else {
            p.pos.x = nx;
            p.pos.y = ny;
        }
    } else {
        p.pos += p.vel;
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines;
// the RNG-order argument is by construction (the module takes no rng).
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::{
        proj_data, FlailInputs, ProjectilePool, YoyoInputs, MUSKET_BALL, SAUCER_LASER,
        WOODEN_ARROW,
    };
    use tr_core::{World, TILE_PX};

    /// A tile-free world: the flight witnesses run in open air.
    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    /// A slot carrying its `SetDefaults` row slice, spawned at (160, 160)
    /// with the given velocity. Field-by-field on `Default` — the struct
    /// has private bookkeeping fields (E0451 outside projectile.rs).
    fn proj(kind: u16, vx: f32, vy: f32) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    fn npc_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Slime, x, y, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        n
    }

    fn step_pool(pool: &mut ProjectilePool, w: &mut World) {
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut npcs = Vec::new();
        pool.update(
            w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
    }

    /// The gate is disjoint from A's and B's kind lists, the riders, the
    /// Book of Skulls and the escape-clause kinds — `covers` is the
    /// seam's whole contract.
    #[test]
    fn s1_d_5c_the_gate_is_disjoint_from_a_and_b() {
        for k in crate::projectile_ai001_a::SLICE_A_KINDS {
            assert!(!covers(k), "slice A owns {k}");
        }
        for k in crate::projectile_ai001_b::SLICE_B_KINDS {
            assert!(!covers(k), "slice B owns {k}");
        }
        assert!(!covers(WOODEN_ARROW)); // the generic arrow
        assert!(!covers(2)); // the flame arrow
        assert!(!covers(178)); // the cannon bomb — generic is its arm
        assert!(!covers(20)); // the riders stay on ai_style1_rider
        assert!(!covers(88));
        assert!(!covers(355));
        assert!(!covers(660));
        assert!(!covers(837)); // the book of skulls keeps its own arm
        assert!(!covers(185)); // the aiStyle-14 data deviation
        assert!(!covers(687)); // the aiStyle-136 data deviation
        for k in SLICE_C_KINDS {
            assert!(covers(k));
        }
    }

    /// 110 (`:55244-55248` latch, `:55613`+`:55626-55633` fade 15,
    /// `:55962` flag3 = false): 30 ticks at (2, 0) → vel stays (2, 0),
    /// ai[0] = 0, ai[1] = 1, alpha 255 − 15×17 → 0 (17 ticks to clear),
    /// rot = atan2(0, 2) + 1.57 = 1.57 every tick (`:57750`).
    #[test]
    fn s1_d_5c_the_snowman_bullet_flies_straight_and_fades() {
        let w = world();
        let mut p = proj(BULLET_SNOWMAN, 2.0, 0.0);
        for _ in 0..30 {
            step(&mut p, &w, &[], -1);
        }
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:55962`)");
        assert_eq!(p.ai[1], 1.0, "the latch (`:55246`)");
        assert_eq!(p.alpha, 0, "255 - 15/tick clamps at 0 (`:55628`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — the counter rests");
        assert!((p.rot - 1.57).abs() < 1e-6);
        assert!((p.pos.x - (160.0 + 60.0)).abs() < 1e-4, "2 px/tick × 30");
    }

    /// 275/276 (`:55012-55024` flutter, `:55336-55345` latch,
    /// `:57197-57211` gravity): 40 ticks → ai[0] pinned at 35, gravity
    /// fired on ticks 35..40 = 6 times → vel.y ≈ 6 × 0.025 = 0.15; the
    /// alpha fades 30/tick to 0; the frame alternates every 2 ticks
    /// (40 ticks → 20 advances → frame 0). The expert homing
    /// (`:57212-57231`) is unreachable in the normal-mode port.
    #[test]
    fn s1_d_5c_the_plantera_seeds_pin_at_thirtyfive_and_fall_at_0025() {
        let w = world();
        for kind in [SEED_PLANTERA, POISON_SEED_PLANTERA] {
            let mut p = proj(kind, 2.0, 0.0);
            for _ in 0..40 {
                step(&mut p, &w, &[], -1);
            }
            assert_eq!(p.ai[1], 1.0, "the latch (`:55338`/`:55343`)");
            assert_eq!(p.ai[0], 35.0, "the pin (`:57209`)");
            assert!(
                (p.vel.y - 6.0 * 0.025).abs() < 1e-5,
                "6 gravity ticks × 0.025, got {}",
                p.vel.y
            );
            assert_eq!(p.alpha, 0, "the fade 30 (`:57201`)");
            assert_eq!(p.frame, 0, "20 advances mod 2 (`:55018`)");
        }
    }

    /// 662/685 (`:57471-57478` + `:57654-57657`): 65 ticks at (10, 0) →
    /// ai[0] = 65 (no pin), the drag/gravity fired on ticks 61..65 = 5
    /// times: vx = 10 × 0.98^5, vy = 5 × 0.3 = 1.5; the rotation is
    /// atan2(vy, vx) − π − π/4.
    #[test]
    fn s1_d_5c_the_dd2_spears_drag_and_drop_after_sixty() {
        let w = world();
        for kind in [DD2_SPEAR, DD2_SPEAR_T3] {
            let mut p = proj(kind, 10.0, 0.0);
            for _ in 0..65 {
                step(&mut p, &w, &[], -1);
            }
            assert_eq!(p.ai[0], 65.0, "the counter runs, no pin");
            let want_vx = 10.0 * 0.98f32.powi(5);
            assert!((p.vel.x - want_vx).abs() < 1e-4, "vx {}", p.vel.x);
            assert!((p.vel.y - 1.5).abs() < 1e-5, "vy {}", p.vel.y);
            let want_rot =
                p.vel.y.atan2(p.vel.x) - std::f32::consts::PI - std::f32::consts::FRAC_PI_4;
            assert!((p.rot - want_rot).abs() < 1e-6);
        }
    }

    /// 675 (`:56461-56473` + `:57520` exclusion + `:57628-57631`):
    /// 10 ticks at (2, 0) → localAI[0] = 1 (the one-shot), alpha
    /// 255 − 20×10 = 55, ai[0] = 10 (the counter runs) but vel.y = 0
    /// (the chain guard excludes the gravity), rot = 10 × 2 × 0.03 = 0.6
    /// (the accumulated spin).
    #[test]
    fn s1_d_5c_the_dark_mage_flame_spins_without_falling() {
        let w = world();
        let mut p = proj(DARK_MAGE_FLAME, 2.0, 0.0);
        for _ in 0..10 {
            step(&mut p, &w, &[], -1);
        }
        assert_eq!(p.local_ai[0], 1.0, "the one-shot (`:56465`)");
        assert_eq!(p.alpha, 55, "255 - 20 × 10 (`:56468`)");
        assert_eq!(p.ai[0], 10.0, "675 is off the flag3 skip list");
        assert_eq!(p.vel.y, 0.0, "the `:57520` guard excludes the gravity");
        assert!((p.rot - 0.6).abs() < 1e-5, "the accumulated spin");
    }

    /// 634 with no NPCs (`:54484-54554`): tick 1 latches ai[1] = 1 then
    /// the retry window bumps it to 2; the window cycles 2,3,4,1 every
    /// 4 ticks. 12 ticks → ai[1] = 1 (12 mod 4 = 0 → the wrap tick),
    /// localAI[0] = 12 (the skipped rand seed leaves 0; `:54573`),
    /// alpha 255 − 40×12 → 0, frame 1 (frameCounter hits 12 = 4 ×
    /// MaxUpdates 3, the row's extraUpdates 2).
    #[test]
    fn s1_d_5c_the_blaze_retry_window_cycles_without_targets() {
        let w = world();
        let mut p = proj(NEBULA_BLAZE_1, 4.0, 0.0);
        for _ in 0..12 {
            step(&mut p, &w, &[], -1);
        }
        assert_eq!(p.ai[1], 1.0, "the 4-tick retry cycle (`:54547-54554`)");
        assert_eq!(p.ai[0], 0.0, "no target acquired");
        assert_eq!(p.local_ai[0], 12.0, "the phase counter (`:54573`)");
        assert_eq!(p.alpha, 0, "the fade 40 (`:54555`)");
        assert_eq!(p.frame, 1, "12 ticks / (4 × MaxUpdates 3)");
        assert_eq!(p.vel.y, 0.0, "flag3 rests + the `:57520` guard");
    }

    /// 635's acquisition (`:54490-54517`): a chaseable NPC 100 px away
    /// with a clear line, ai[1] preset to 1 (the scan tick) → the lock
    /// ai[1] = 6, ai[0] = the index. The next tick steers:
    /// vel = (vel × 29 + toward × 6) / 30 (`:54544`).
    #[test]
    fn s1_d_5c_the_blaze_locks_and_steers_at_the_weight_thirty() {
        let w = world();
        let mut p = proj(NEBULA_BLAZE_2, 4.0, 0.0);
        p.ai[1] = 1.0;
        // A 32×32 target box centered exactly on (280, 180) — 100 px
        // from the blaze center (180, 180), inside the 500 reach,
        // outside the 20 detonation radius.
        let mut n = npc_at(264.0, 164.0);
        n.size_override = Some((32.0, 32.0));
        let (nw, nh) = n.size();
        let nc = Vec2::new(n.x + nw * 0.5, n.y + nh * 0.5);
        let npcs = [n];
        step(&mut p, &w, &npcs, -1);
        assert_eq!(p.ai[1], 6.0, "the lock = num26 + 1 (`:54514`)");
        assert_eq!(p.ai[0], 0.0, "the target index (`:54515`)");
        let v0 = p.vel;
        // The first step's movement rode — the steer reads the NEW center.
        let c1 = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        step(&mut p, &w, &npcs, -1);
        assert_eq!(p.ai[1], 7.0, "the held counter (`:54521`)");
        let mut toward = nc - c1;
        toward = toward / toward.length() * 6.0;
        let want = (v0 * 29.0 + toward) / 30.0;
        assert!((p.vel.x - want.x).abs() < 1e-4, "vx {}", p.vel.x);
        assert!((p.vel.y - want.y).abs() < 1e-4, "vy {}", p.vel.y);
    }

    /// 634's detonation (`:54507-54511`): a chaseable NPC inside 20 px
    /// on the scan tick → Kill, no state writes ride.
    #[test]
    fn s1_d_5c_the_blaze_detonates_inside_twenty() {
        let w = world();
        let mut p = proj(NEBULA_BLAZE_1, 4.0, 0.0);
        p.ai[1] = 1.0;
        // A 32×32 box centered on (190, 180) — 10 px from the blaze
        // center (180, 180), inside the 20 detonation radius.
        let mut n = npc_at(174.0, 164.0);
        n.size_override = Some((32.0, 32.0));
        let npcs = [n];
        step(&mut p, &w, &npcs, -1);
        assert!(!p.active, "the Kill (`:54509`)");
        assert_eq!(p.time_left, 0);
    }

    /// 645's placed cloud (`:56976-56999`): ai[1] = -1, extraUpdates 5 →
    /// each call adds TWO to ai[0] (flag3 `:56040` + the cloud `:56978`);
    /// 63 calls → ai[0] = 126 = 7 frames × 6 MaxUpdates × 3 → Kill. The
    /// box grows to 140×140 about the old center on the first call; the
    /// frame advances every 18 calls (frame 3 at call 63... 63/18 = 3).
    #[test]
    fn s1_d_5c_the_poison_cloud_double_counts_and_dies_at_126() {
        let w = world();
        let mut p = proj(POISON_DART, 3.0, 0.0);
        p.ai[1] = -1.0;
        step(&mut p, &w, &[], -1);
        assert_eq!(p.ai[0], 2.0, "flag3 + the cloud increment");
        assert_eq!(p.w, 140.0);
        assert_eq!(p.h, 140.0);
        assert!((p.pos.x - (160.0 + 5.0 - 70.0)).abs() < 1e-4, "re-centered");
        assert!((p.pos.y - (160.0 + 5.0 - 70.0)).abs() < 1e-4);
        assert_eq!(p.vel, Vec2::ZERO);
        assert!(!p.tile_collide);
        assert_eq!(p.penetrate, -1);
        for _ in 1..63 {
            step(&mut p, &w, &[], -1);
        }
        assert!(!p.active, "ai[0] = 126 = projFrames 7 × 18 (`:56995`)");
        assert_eq!(p.frame, 3, "63 / 18 = 3 advances, no wrap (`:56993`)");
    }

    /// 645's flying scan (`:57001-57026`): numUpdates == 0 with a
    /// chaseable NPC inside 60 px places the cloud (ai[0] = 0 — the
    /// flag3 increment is reset — ai[1] = -1) and returns early: the
    /// rotation tail never ran (rot stays 0), alpha = 255, and the
    /// movement still rode (pos advanced by vel). Above the launch
    /// height (pos.y 160 > ai[1] 200 is false) tileCollide stays false.
    #[test]
    fn s1_d_5c_the_flying_dart_places_its_cloud_on_the_scan_substep() {
        let w = world();
        let mut p = proj(POISON_DART, 3.0, 0.0);
        p.ai[1] = 200.0; // the launch-height threshold
        // A 32×32 box centered on (190, 165) — 25 px from the dart
        // center (165, 165), inside the 60 scan reach.
        let mut n = npc_at(174.0, 149.0);
        n.size_override = Some((32.0, 32.0));
        let npcs = [n];
        step(&mut p, &w, &npcs, 0);
        assert_eq!(p.ai[0], 0.0, "the scan resets the counter (`:57021`)");
        assert_eq!(p.ai[1], -1.0, "the cloud latch (`:57022`)");
        assert_eq!(p.alpha, 255, "the flying opacity (`:57001`)");
        assert_eq!(p.rot, 0.0, "the early return skipped the tail");
        assert!(!p.tile_collide, "still above the launch height");
        assert!((p.pos.x - 163.0).abs() < 1e-4, "the movement rode");
        // numUpdates != 0: no scan, the default tail runs.
        let mut q = proj(POISON_DART, 3.0, 0.0);
        q.ai[1] = 200.0;
        step(&mut q, &w, &npcs, -1);
        assert_eq!(q.ai[1], 200.0, "no scan off the numUpdates == 0 tick");
        assert_eq!(q.ai[0], 1.0, "flag3 counted");
        assert!((q.rot - 1.57).abs() < 1e-6, "the default tail ran");
    }

    /// The A/B/C seam witness, pool-level: a slice-C kind (110) takes the
    /// new arm through the dispatch (resting counter), the plain wooden
    /// arrow keeps the generic flight byte-identical (15 ticks →
    /// vel.y = 0.1, the DW-3.2 pin), slice A's musket ball keeps its
    /// resting counter and slice B's saucer laser keeps its ramp.
    #[test]
    fn s1_d_5c_the_seam_chains_after_a_and_b() {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(BULLET_SNOWMAN, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..30 {
            step_pool(&mut pool, &mut w);
        }
        // 110 has extraUpdates 1 → 2 sub-steps/tick; the counter rests
        // either way, the fade clears in 17 sub-steps.
        assert_eq!(pool.slots()[0].ai[0], 0.0, "the seam routed 110 to C");
        assert_eq!(pool.slots()[0].vel.y, 0.0);
        assert_eq!(pool.slots()[0].alpha, 0);

        let mut arrow = ProjectilePool::new();
        arrow.spawn_with_data(WOODEN_ARROW, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..15 {
            step_pool(&mut arrow, &mut w);
        }
        assert!(
            (arrow.slots()[0].vel.y - 0.1).abs() < 1e-6,
            "the generic flight is byte-identical for the plain arrow"
        );

        let mut musket = ProjectilePool::new();
        musket.spawn_with_data(MUSKET_BALL, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..30 {
            step_pool(&mut musket, &mut w);
        }
        assert_eq!(musket.slots()[0].ai[0], 0.0, "slice A still owns 14");

        let mut saucer = ProjectilePool::new();
        saucer.spawn_with_data(SAUCER_LASER, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..20 {
            step_pool(&mut saucer, &mut w);
        }
        assert_eq!(
            saucer.slots()[0].local_ai[0], 100.0,
            "slice B still owns 449 (its +3/tick ramp saturates at 100)"
        );
    }

    // ---- S1-E-21h d76: the slice-C cue witnesses (WRITTEN, NOT RUN) --------

    /// 110 (`:55247`) → Item11, 275/276 (`:55339`/`:55344`) → Item17,
    /// 634/635 (`:54488`) → Item34 — each once per life at base.position.
    #[test]
    fn s1_e_21h_d76_the_slice_c_latches_carry_their_cues() {
        let w = world();
        for (kind, style) in [
            (BULLET_SNOWMAN, 11),       // :55247
            (SEED_PLANTERA, 17),        // :55339
            (POISON_SEED_PLANTERA, 17), // :55344
            (NEBULA_BLAZE_1, 34),       // :54488
            (NEBULA_BLAZE_2, 34),       // :54488
        ] {
            let mut p = proj(kind, 2.0, 0.0);
            step(&mut p, &w, &[], -1);
            assert_eq!(p.ai[1], 1.0, "kind {kind}: the latch");
            assert_eq!(
                p.wants_sound,
                Some((2, style, 160.0, 160.0, 0.0, 1.0)),
                "kind {kind}'s cue at base.position"
            );
            p.wants_sound = None;
            step(&mut p, &w, &[], -1);
            assert_eq!(p.wants_sound, None, "kind {kind}: one cue per life");
        }
    }
}
