//! `NPC.HitEffect` — the CLASSIC-MOB else-if chain (W4-f5 phase 2, wave D):
//! the giant `else { if (this.type == 59 || ...) ... }` tail of the per-NPC
//! gore/dust table, `NPC.cs:85560-89835` — 853 `Gore.NewGore*` sites in
//! source order, from the King Slime's 734 (`:85684`) to the 156 arm's
//! 184/185/185 (`:89829-89831`).
//!
//! The pool itself is already ported (`tr_core::gore::GorePool::new_gore`,
//! S1-E-4; the `GoreID.Sets` tables in `tr_core::gore_id`); this module is
//! the per-NPC spawn table the pool feeds. Code-only mode (the f10
//! precedent): the module + witnesses land first; the dispatch claim (the
//! `StrikeNPC_Inner` `:81482`/`:81488` pair and the AI-arm `HitEffect()`
//! calls) is the wiring wave.
//!
//! ## Claim boundary (the pushed-claims probe, 2026-11-21)
//!
//! * wave A (`W4-d55`, `ec78d152`): sites 1-30, `NPC.cs:81660-81941`.
//! * wave B (`W4-e5-gore-sites-2`): sites 31-70 nominal, `:81942-82202`.
//! * wave C (`W4-f10-gore-sites-3`, tip `7d94a465`): sites 71-258,
//!   `:82206-83562` — f10 owns EVERYTHING below `:85560` (its in-flight
//!   continuation covers `:83562-85559`).
//! * wave D (THIS module, `task/W4-f5-dust-proj-3` phase 2): the
//!   classic-mob chain `:85560-89835` ONLY. Disjoint from f10's writer.
//! * The dust lanes stay theirs: `W4-g62` owns the HIT (`life > 0`) halves
//!   (its `npc_hit_dust.rs` live-arm replay already covers the bat/demon
//!   eye/zombie/skeleton/goblin/EoS/hornet hit loops INSIDE this window);
//!   `W4-g59`/`W4-g60` own the death-dust feeds. This module ports the
//!   death-half dust loops ONLY as the gore bodies' draw census (the f10
//!   `death_dust_loop` precedent — keep-one at merge); the hit halves are
//!   NOT ported here.
//! * `W4-g16` (projectile) / `W4-g17` (world) own the non-NPC files.
//!
//! ## Scope — the chain (source order, death halves + gore bodies)
//!
//! Sites are numbered 1-853 in `Gore.NewGore` source order within
//! `:85560-89835` (the claim census). Batches land ≤10 sites per commit:
//!
//! | sites | C# lines | arm |
//! | --- | --- | --- |
//! | 1 | `:85684` | the 50 King Slime death (200x tinted dust-4 + gore 734 + the blue-slime spawn seam) |
//! | 2-5 | `:85718-85721` | the 153 Giant Tortoise death (177/178/179/179) |
//! | 6-9 | `:85737-85740` | the 177 Derpling death (234/235/236/236) |
//!
//! Later commits extend the table through `:89831`. The per-arm census
//! grows in this header as the batches land.
//!
//! ## RNG contract (the strike's stream)
//!
//! `HitEffect` runs inside `StrikeNPC_Inner` (`NPC.cs:81488`), under the
//! strike caller's `Main.rand` swap (the projectile/melee update's stage),
//! NOT the NPC pass's — the `npc_ai_style37.rs::probe_launch_on_hit` note.
//! The caller hands that stream in; every draw hits it in C# order:
//!
//! * the death dust loops run through `DustPool::new_dust` (S1-E-1) on the
//!   SAME stream — the 6 default-block draws (`Dust.cs:153-193`) plus the
//!   per-type tweak draws, pool-owned; a cull refusal or a pressure-band
//!   gate consumes exactly the C#'s draws;
//! * ARGUMENT draws evaluate BEFORE the `NewGore` call, in the C#
//!   argument-evaluation order (named arguments evaluate in SOURCE order —
//!   the `:87405` Velocity/Position/Type triple draws Next(-80,81),
//!   Next(-60,21), Next(140,143) in that written order);
//! * each `NewGore` then draws ONLY its two kicks, Y then X
//!   (`Gore.cs:1327-1328`, pool-side: `velocity.y -= Next(10,31)*0.1` then
//!   `velocity.x += Next(-20,21)*0.1`); the types/anchors are data.
//!
//! The post-spawn writes land on the returned slot, the sentinel included
//! — the C# writes `Main.gore[600]` too (the `npc_ai_style56.rs` precedent).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `!this.active` gate (`:81656`): the caller's — the fn runs on an
//!   active NPC (the strike/death path's own check).
//! * The `this.type` axis rides an explicit `npc_type` parameter (the
//!   wave-A `HitEffectInputs` / f10 precedent — most classic kinds have no
//!   `NpcKind` rows yet; witnesses ride `size_override` stand-ins).
//! * The HIT (`life > 0`) halves: g62's lane (see the claim boundary); the
//!   dispatcher gates every arm on `npc.hp <= 0`.
//! * `netMode` guards: single-player port (the `netMode != 1` arms run,
//!   the `== 1` early-returns drop, the `== 2` sends drop).
//! * The 59/60/151 lava arm (`:85562-85669`) has NO gore sites — its dust
//!   census is the dust lanes', its expert/skyblock/remix lava writes are
//!   a World seam (the wiring wave); recorded here, not ported.
//! * Sounds ride `W4-c13`; `Lighting.AddLight` rides the light lane.
//!
//! See docs/ANALYSIS.md "The NPC HitEffect classic-mob chain (W4-f5, wave D)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the strike-path claim is the follow-up round

use crate::npc::Npc;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Entity.Center` (`Entity.cs:50-60`) — `(pos.X + width/2f, pos.Y + height/2f)`.
fn center(npc: &Npc) -> Vec2 {
    let (w, h) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0)
}

/// The side-effects the wiring caller owes (the roster/World seams this
/// module cannot reach — the wave-A `HitEffectOutcome` / f10
/// `HitEffectOutcomeC` precedent).
#[derive(Default)]
pub struct HitEffectOutcomeD {
    /// The 50 King Slime's blue-slime spawn seam (`NPC.cs:85689-85702`):
    /// `num445 = Next(4) + 4` spawns of NPC-1 at
    /// `(pos.X + Next(width-32), pos.Y + Next(height-32))`, each with
    /// `velocity = (Next(-15,16)*0.1, Next(-30,1)*0.1)`,
    /// `ai[0] = -1000*Next(3)`, `ai[1] = -1`. `Some((x,y))` = the NPC
    /// position the spawns anchor on; the count draw, the per-spawn
    /// position/velocity/ai draws and the `Start: whoAmI+1` scan ride the
    /// wiring wave's roster pass (the f10 `nebula_brain_spawn` precedent).
    /// The C# `netMode == 1` early-return (`:85685-85688`) drops in SP.
    pub king_slime_spawns: Option<(f32, f32)>,
}

/// The death-half dust loop shared by the classic rows (`NPC.cs` per-arm
/// `for (… < N; …) Dust.NewDust(position, width, height, Type, 2*hd, -2f)`).
/// `count` is the flat death count; the hit (`life > 0`) half's dmg-scaled
/// bound is g62's lane. The pool owns every draw. The f10
/// `death_dust_loop` shape, widened with the alpha/color/scale arguments
/// the King Slime's tinted burst needs.
#[allow(clippy::too_many_arguments)]
fn death_dust_loop(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    dust_type: i32,
    hit_direction: f32,
    count: i32,
    alpha: i32,
    color: Color,
    scale: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        dust.new_dust(
            rng,
            dust_ctx,
            world,
            pos,
            w as i32,
            h as i32,
            dust_type,
            2.0 * hit_direction,
            -2.0,
            alpha,
            color,
            scale,
        );
    }
}

/// The classic-mob chain's death halves (`NPC.cs:85560-89835`), in source
/// order. The dispatcher mirrors the C# else-if chain: the FIRST matching
/// arm runs (the arms are mutually exclusive on `this.type`).
pub fn hit_effect_d(
    npc: &Npc,
    npc_type: i32,
    hit_direction: f32,
    _dmg: f64,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) -> HitEffectOutcomeD {
    let mut outcome = HitEffectOutcomeD::default();
    // `:85562-85669` — the 59/60/151 lava arm: NO gore sites (the claim's
    // documented drop; the dust census is the dust lanes', the lava
    // writes are the wiring wave's World seam).
    // The per-arm dispatch grows here as the batches land.
    let _ = (npc, npc_type, hit_direction, gore, dust, dust_ctx, rng, world);
    outcome
}
