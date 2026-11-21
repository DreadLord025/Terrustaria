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

/// The 50 King Slime death (`NPC.cs:85670-85703`, site 1): the 200x tinted
/// dust-4 burst (`2*hd, -2`, alpha 175, `Color(0,80,255,100)`), then ONE
/// gore 734 at `Center + (-40, -height/2)` — scale defaults to 1f (the C#
/// passes no `this.scale`). The `netMode == 1` early-return (`:85685`)
/// drops in SP; the `Next(4)+4` blue-slime spawn loop (`:85689-85702`) is
/// the roster seam — its count draw + per-spawn draws ride the wiring
/// wave (the f10 `nebula_brain_spawn` precedent), returned as
/// `king_slime_spawns`.
fn king_slime_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeD,
) {
    // `:85680-85683` — the 200x tinted dust-4 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        4,
        hit_direction,
        200,
        175,
        Color::new(0, 80, 255, 100),
        1.0,
    );
    // `:85684` — the crown gore at Center + (-40, -height/2), scale 1f.
    let (_, h) = npc.size();
    let pos = center(npc) + Vec2::new(-40.0, -h / 2.0);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 734, 1.0);
    // `:85689-85702` — the blue-slime spawn seam (the wiring wave's).
    outcome.king_slime_spawns = Some((npc.x, npc.y));
}

/// The 153 Giant Tortoise death (`NPC.cs:85704-85722`, sites 2-5): the
/// 15x dust-5 burst (`2*hd, -2`, default alpha/color/scale), then 4 gores
/// 177/178/179/179 at position — scale defaults to 1f (no `this.scale`
/// argument).
fn tortoise_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85714-85717` — the 15x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        15,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 177, 1.0); // :85718
    gore.new_gore(rng, world, pos, vel, 178, 1.0); // :85719
    gore.new_gore(rng, world, pos, vel, 179, 1.0); // :85720
    gore.new_gore(rng, world, pos, vel, 179, 1.0); // :85721
}

/// The 177 Derpling death (`NPC.cs:85723-85741`, sites 6-9): the same
/// shape as the tortoise — the 15x dust-5 burst, then 4 gores
/// 234/235/236/236 at position, scale 1f.
fn derpling_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85733-85736` — the 15x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        15,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 234, 1.0); // :85737
    gore.new_gore(rng, world, pos, vel, 235, 1.0); // :85738
    gore.new_gore(rng, world, pos, vel, 236, 1.0); // :85739
    gore.new_gore(rng, world, pos, vel, 236, 1.0); // :85740
}

/// The classic-mob chain's death halves (`NPC.cs:85560-89835`), in source
/// order. The dispatcher mirrors the C# else-if chain: the FIRST matching
/// arm runs (the arms are mutually exclusive on `this.type`). Every arm
/// gates on `npc.hp <= 0` — the hit (`life > 0`) halves are g62's lane.
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
    if npc.hp > 0 {
        return outcome;
    }
    // `:85670-85703` — the 50 King Slime arm (site 1).
    if npc_type == 50 {
        king_slime_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:85704-85722` — the 153 Giant Tortoise arm (sites 2-5).
    else if npc_type == 153 {
        tortoise_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85723-85741` — the 177 Derpling arm (sites 6-9).
    else if npc_type == 177 {
        derpling_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 400, 400)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// A classic-mob stand-in: the unported kinds ride a placeholder kind
    /// with the C# box through `size_override` (the f10 `boss` precedent).
    fn mob(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 100, 0.0);
        n.size_override = Some((w, h));
        n.hp = 0;
        n.scale = 1.0;
        n
    }

    /// The 6 default-block draws of one `Dust.NewDust` (Dust.cs:153-193)
    /// for a type with NO tweak (4/5) and ChildSafety off — the f10 twin
    /// census.
    fn twin_dust(twin: &mut UnifiedRandom, bw: i32, bh: i32) {
        let _px = twin.next_max(bw - 4);
        let _py = twin.next_max(bh - 4);
        let _vx = twin.next_range(-20, 21);
        let _vy = twin.next_range(-20, 21);
        let _row = twin.next_max(3);
        let _scale = twin.next_range(-20, 21);
    }

    /// `NPC.cs:85684` — the King Slime death spawns ONE gore 734 at
    /// Center + (-40, -height/2), scale 1f (no this.scale argument), after
    /// the 200x tinted dust-4 burst; the blue-slime spawn seam is returned.
    #[test]
    fn w4_f5d_the_king_slime_death_spawns_the_crown_gore() {
        let w = air();
        let mut n = mob(100.0, 100.0, 120.0, 100.0);
        n.vx = 2.0;
        n.vy = -1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let out = hit_effect_d(&n, 50, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert_eq!(gore.slot(0).gore_type, 734);
        // Center = (160,150); the crown anchor = (120,100).
        assert_eq!(gore.slot(0).position, Vec2::new(120.0, 100.0));
        assert_eq!(gore.slot(0).scale, 1.0);
        assert!(!gore.slot(1).active, "exactly one gore");
        // The 200-dust tinted burst ran first (the pool holds the dust).
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 150, "the death burst spawned: {dust_active}");
        let tinted = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(tinted.dust_type, 4);
        assert_eq!(tinted.alpha, 175);
        assert_eq!(tinted.color, Color::new(0, 80, 255, 100));
        // The roster seam is returned for the wiring wave.
        assert_eq!(out.king_slime_spawns, Some((100.0, 100.0)));
    }

    /// `NPC.cs:85680-85684` — the King Slime death's draw census: 200
    /// tinted dust-4 (6 default-block draws each, no type-4 tweak) then
    /// ONE gore's two kicks (Y then X). The spawn seam draws NOTHING here
    /// (its count/per-spawn draws are the wiring wave's).
    #[test]
    fn w4_f5d_the_king_slime_death_mirrors_the_csharp_draws() {
        let w = air();
        let n = mob(100.0, 100.0, 120.0, 100.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        let _ = hit_effect_d(&n, 50, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(3);
        for _ in 0..200 {
            twin_dust(&mut twin, 120, 100);
        }
        let _ky = twin.next_range(10, 31); // NewGore Y kick
        let _kx = twin.next_range(-20, 21); // NewGore X kick
        assert_eq!(rng.peek(), twin.peek(), "the king slime death mirrors the C# draws");
    }

    /// `NPC.cs:85718-85721` — the Giant Tortoise death: 4 gores
    /// 177/178/179/179 at position, scale 1f, after the 15x dust-5 burst.
    #[test]
    fn w4_f5d_the_tortoise_death_spawns_the_four_gores() {
        let w = air();
        let mut n = mob(100.0, 100.0, 48.0, 36.0);
        n.vx = 1.5;
        n.vy = -0.5;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        let out = hit_effect_d(&n, 153, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![177, 178, 179, 179]);
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(i).scale, 1.0);
        }
        assert!(!gore.slot(4).active, "exactly four gores");
        assert_eq!(out.king_slime_spawns, None);
    }

    /// `NPC.cs:85737-85740` — the Derpling death: 4 gores 234/235/236/236
    /// at position, scale 1f.
    #[test]
    fn w4_f5d_the_derpling_death_spawns_the_four_gores() {
        let w = air();
        let mut n = mob(100.0, 100.0, 40.0, 40.0);
        n.vx = -2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        let _ = hit_effect_d(&n, 177, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![234, 235, 236, 236]);
        assert!(!gore.slot(4).active, "exactly four gores");
    }

    /// `NPC.cs:85714-85721`/`:85733-85740` — both 15-dust deaths draw the
    /// 15x6 dust census then only the 4 gores' two kicks each; the twin
    /// replay pins the stream.
    #[test]
    fn w4_f5d_the_15_dust_deaths_mirror_the_csharp_draws() {
        let w = air();
        let n = mob(100.0, 100.0, 48.0, 36.0);
        for kind in [153, 177] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(5);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let mut twin = UnifiedRandom::new(5);
            for _ in 0..15 {
                twin_dust(&mut twin, 48, 36);
            }
            for _ in 0..4 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85706-85712` — the hit (`life > 0`) half is g62's lane: the
    /// dispatcher draws NOTHING and spawns NOTHING on a living NPC.
    #[test]
    fn w4_f5d_the_hit_halves_stay_g62s() {
        let w = air();
        let mut n = mob(100.0, 100.0, 48.0, 36.0);
        n.hp = 20;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let _ = hit_effect_d(&n, 153, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert!(!gore.slot(0).active, "no gore on the hit half");
        assert!(!dust.slots()[0].active, "no dust on the hit half");
        assert_eq!(rng.peek(), UnifiedRandom::new(11).peek(), "no draws on the hit half");
    }
}
