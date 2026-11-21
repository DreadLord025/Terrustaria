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

/// The 494/495 Crawdad death (`NPC.cs:85742-85759`, sites 10-11): the
/// 15x dust-5 burst, then `num454 = (type - 494) * 2` picks the pair —
/// `736 + num454` then `735 + num454` (the HIGHER type first, 495→738/737),
/// at position, scale 1f.
fn crawdad_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85752-85755` — the 15x dust-5 burst.
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
    // `:85756` — the per-variant offset.
    let num454 = (npc_type - 494) * 2;
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 736 + num454, 1.0); // :85757
    gore.new_gore(rng, world, pos, vel, 735 + num454, 1.0); // :85758
}

/// The 496/497 Giant Shelly death (`NPC.cs:85760-85777`, site 12): the
/// 15x dust-5 burst, then ONE gore `739 + (type - 496)`. NOTE the C# hit
/// half here has NO `return` (`:85762-85768` — the if/else shape, not the
/// early-return shape); the dispatcher's `hp <= 0` gate covers the death
/// half either way, the hit half stays g62's.
fn shelly_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85771-85774` — the 15x dust-5 burst.
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
    gore.new_gore(rng, world, pos, vel, 739 + (npc_type - 496), 1.0); // :85775
}

/// The 498-506 Salamander death (`NPC.cs:85778-85796`, sites 13-15 per
/// variant, 27 across the nine): the 15x dust-5 burst, then
/// `num459 = type - 498` picks the triple — `750 + num459` at position,
/// `741 + num459` at +16 Y, `759 + num459` at +18 Y, scale 1f.
fn salamander_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85788-85791` — the 15x dust-5 burst.
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
    // `:85792` — the per-variant offset.
    let num459 = npc_type - 498;
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 750 + num459, 1.0); // :85793
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 16.0), vel, 741 + num459, 1.0); // :85794
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 18.0), vel, 759 + num459, 1.0); // :85795
}

/// The bat family death (`NPC.cs:85797-85841`, sites 16-27): the 15x
/// dust-5 burst, then the per-kind gore(s) at position, scale 1f. The
/// arm's sub-branches in source order: 51 Jungle Bat → 83 (`:85813`),
/// 93 Giant Bat → 107 (`:85817`), 150 Ice Bat → 164/165 (`:85821-85822`),
/// 152 Giant Flying Fox → 175/176/176 (`:85826-85828`), 226 Flying Snake
/// → 317/318/318/319 (`:85832-85835`), the 49 Cave Bat fallthrough → 82
/// (`:85839`).
fn bat_family_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85807-85810` — the 15x dust-5 burst.
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
    if npc_type == 51 {
        gore.new_gore(rng, world, pos, vel, 83, 1.0); // :85813
    } else if npc_type == 93 {
        gore.new_gore(rng, world, pos, vel, 107, 1.0); // :85817
    } else if npc_type == 150 {
        gore.new_gore(rng, world, pos, vel, 164, 1.0); // :85821
        gore.new_gore(rng, world, pos, vel, 165, 1.0); // :85822
    } else if npc_type == 152 {
        gore.new_gore(rng, world, pos, vel, 175, 1.0); // :85826
        gore.new_gore(rng, world, pos, vel, 176, 1.0); // :85827
        gore.new_gore(rng, world, pos, vel, 176, 1.0); // :85828
    } else if npc_type == 226 {
        gore.new_gore(rng, world, pos, vel, 317, 1.0); // :85832
        gore.new_gore(rng, world, pos, vel, 318, 1.0); // :85833
        gore.new_gore(rng, world, pos, vel, 318, 1.0); // :85834
        gore.new_gore(rng, world, pos, vel, 319, 1.0); // :85835
    } else {
        // `:85837-85840` — the 49 Cave Bat fallthrough.
        gore.new_gore(rng, world, pos, vel, 82, 1.0); // :85839
    }
}

/// The 611/689 Owl death (`NPC.cs:85842-85859`, sites 28-30): the 15x
/// dust-5 burst, then 1164/1164/1165 at position, scale 1f. The 689 is
/// the Owl Mimic (`NPCID.cs:12055`) — same gore row.
fn owl_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85852-85855` — the 15x dust-5 burst.
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
    gore.new_gore(rng, world, pos, vel, 1164, 1.0); // :85856
    gore.new_gore(rng, world, pos, vel, 1164, 1.0); // :85857
    gore.new_gore(rng, world, pos, vel, 1165, 1.0); // :85858
}

/// The 624 Gnome death (`NPC.cs:85860-85878`, sites 31-34): the 10x dust-5
/// burst (the count drops to 10 here), then 1190/1191/1192/1193 at
/// position, scale 1f.
fn gnome_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85870-85873` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1190, 1.0); // :85874
    gore.new_gore(rng, world, pos, vel, 1191, 1.0); // :85875
    gore.new_gore(rng, world, pos, vel, 1192, 1.0); // :85876
    gore.new_gore(rng, world, pos, vel, 1193, 1.0); // :85877
}

/// The 628 Dandelion death (`NPC.cs:85879-85897`, sites 35-38): the 10x
/// dust-3 burst (type 3, the leaf puff — NOT 5), then
/// 1205/1206/1206/1207 at position, scale 1f.
fn dandelion_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85889-85892` — the 10x dust-3 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        3,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1205, 1.0); // :85893
    gore.new_gore(rng, world, pos, vel, 1206, 1.0); // :85894
    gore.new_gore(rng, world, pos, vel, 1206, 1.0); // :85895
    gore.new_gore(rng, world, pos, vel, 1207, 1.0); // :85896
}

/// The 646-652 Gem Bunny death (`NPC.cs:85898-85920`, sites 39-40): the
/// dust type is the gem's own (`num468 = 86 + (type - 646)`, except the
/// 652 Amber Topaz variant's 138 — `:85900-85904`), 10x burst, then
/// `num471 = (type - 646) * 2` picks the pair 1234+n/1235+n at position,
/// scale 1f.
fn gem_bunny_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85900-85904` — the gem dust type.
    let num468 = if npc_type < 652 { 86 + (npc_type - 646) } else { 138 };
    // `:85913-85916` — the 10x gem-dust burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        num468,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    // `:85917` — the per-variant offset.
    let num471 = (npc_type - 646) * 2;
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1234 + num471, 1.0); // :85918
    gore.new_gore(rng, world, pos, vel, 1235 + num471, 1.0); // :85919
}

/// The 639-645 Gem Squirrel death (`NPC.cs:85921-85941`, site 41): the
/// same gem-dust shape (`num472 = 86 + (type - 639)`, the 645's 138), 10x
/// burst, then ONE gore `1227 + (type - 639)` at position, scale 1f.
fn gem_squirrel_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85923-85927` — the gem dust type.
    let num472 = if npc_type < 645 { 86 + (npc_type - 639) } else { 138 };
    // `:85936-85939` — the 10x gem-dust burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        num472,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1227 + (npc_type - 639), 1.0); // :85940
}

/// The 29-type critter arm's death (`NPC.cs:85942-86091`): the 10x dust-5
/// burst, then the per-kind sub-branch. The branches land in source order
/// — the live sub-branches: 46/303/337/540 → 76/77 (+ the 303's extra 10x
/// dust-193 AFTER the two gores, + the 540's extra gore 926,
/// `:85956-85971`), 67 → 95/95/96 (`:85972-85977`), 55/230 → 553/554
/// (`:85978-85982`), 615 Dolphin → 1198/1199/1200/1200 (`:85983-85989`),
/// 74 Bird → 100 (`:85992`), 297 Blue Bird → 431 (`:85996`), 298 Red Bird
/// → 432 (`:86000`), the 671/672/673 macaw/toucan triples — head at
/// position + wing pair at Center ±(4,0) (`:86002-86019`), the 674/675
/// cockatiel triples (same shape, `:86020-86031`), the 299 Squirrel → 433
/// (`:86034`), 538 Red Squirrel → 897 (`:86038`), 300 Mouse → 434
/// (`:86042`), 610 Rat → 1163 (`:86046`), 626 Seahorse → 1197 (`:86050`),
/// 102 Angler Fish → 116 (`:86054`), 224 Flying Fish → 312/313
/// (`:86058-86059`), 587 Eyeball Flying Fish → extra 50x dust-5 at
/// (2.5·hd, −2.5) then 1104/1105 (`:86061-86069`), 616 Turtle →
/// 1166/1167/1168/1168 (`:86070-86076`), 617 Jungle Turtle →
/// 1169/1170/1171/1171 (`:86077-86083`), 625 Sea Turtle →
/// 1194/1195/1196/1196 (`:86084-86090`). The arm's remaining kinds
/// (614/627 — no gore sub-branch in the C#) spawn the dust burst only.
fn critter_arm_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:85952-85955` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if matches!(npc_type, 46 | 303 | 337 | 540) {
        gore.new_gore(rng, world, pos, vel, 76, 1.0); // :85958
        gore.new_gore(rng, world, pos, vel, 77, 1.0); // :85959
        if npc_type == 303 {
            // `:85960-85966` — the 303 Bunny's extra 10x dust-193 burst,
            // AFTER the two gores (the C# order).
            let (w, h) = npc.size();
            for _ in 0..10 {
                dust.new_dust(
                    rng,
                    dust_ctx,
                    world,
                    pos,
                    w as i32,
                    h as i32,
                    193,
                    2.0 * hit_direction,
                    -2.0,
                    0,
                    Color::default(),
                    1.0,
                );
            }
        }
        if npc_type == 540 {
            gore.new_gore(rng, world, pos, vel, 926, 1.0); // :85969
        }
    } else if npc_type == 67 {
        gore.new_gore(rng, world, pos, vel, 95, 1.0); // :85974
        gore.new_gore(rng, world, pos, vel, 95, 1.0); // :85975
        gore.new_gore(rng, world, pos, vel, 96, 1.0); // :85976
    } else if npc_type == 55 || npc_type == 230 {
        gore.new_gore(rng, world, pos, vel, 553, 1.0); // :85980
        gore.new_gore(rng, world, pos, vel, 554, 1.0); // :85981
    } else if npc_type == 615 {
        gore.new_gore(rng, world, pos, vel, 1198, 1.0); // :85985
        gore.new_gore(rng, world, pos, vel, 1199, 1.0); // :85986
        gore.new_gore(rng, world, pos, vel, 1200, 1.0); // :85987
        gore.new_gore(rng, world, pos, vel, 1200, 1.0); // :85988
    } else if npc_type == 74 {
        gore.new_gore(rng, world, pos, vel, 100, 1.0); // :85992
    } else if npc_type == 297 {
        gore.new_gore(rng, world, pos, vel, 431, 1.0); // :85996
    } else if npc_type == 298 {
        gore.new_gore(rng, world, pos, vel, 432, 1.0); // :86000
    } else if npc_type == 671 {
        // `:86002-86007` — the Scarlet Macaw: head at position, the wing
        // pair at Center ±(4,0).
        gore.new_gore(rng, world, pos, vel, 1277, 1.0); // :86004
        gore.new_gore(rng, world, center(npc) + Vec2::new(4.0, 0.0), vel, 1279, 1.0); // :86005
        gore.new_gore(rng, world, center(npc) + Vec2::new(-4.0, 0.0), vel, 1279, 1.0); // :86006
    } else if npc_type == 672 {
        gore.new_gore(rng, world, pos, vel, 1280, 1.0); // :86010
        gore.new_gore(rng, world, center(npc) + Vec2::new(4.0, 0.0), vel, 1281, 1.0); // :86011
        gore.new_gore(rng, world, center(npc) + Vec2::new(-4.0, 0.0), vel, 1281, 1.0); // :86012
    } else if npc_type == 673 {
        gore.new_gore(rng, world, pos, vel, 1282, 1.0); // :86016
        gore.new_gore(rng, world, center(npc) + Vec2::new(4.0, 0.0), vel, 1283, 1.0); // :86017
        gore.new_gore(rng, world, center(npc) + Vec2::new(-4.0, 0.0), vel, 1283, 1.0); // :86018
    } else if npc_type == 674 {
        gore.new_gore(rng, world, pos, vel, 1284, 1.0); // :86022
        gore.new_gore(rng, world, center(npc) + Vec2::new(4.0, 0.0), vel, 1285, 1.0); // :86023
        gore.new_gore(rng, world, center(npc) + Vec2::new(-4.0, 0.0), vel, 1285, 1.0); // :86024
    } else if npc_type == 675 {
        gore.new_gore(rng, world, pos, vel, 1286, 1.0); // :86028
        gore.new_gore(rng, world, center(npc) + Vec2::new(4.0, 0.0), vel, 1287, 1.0); // :86029
        gore.new_gore(rng, world, center(npc) + Vec2::new(-4.0, 0.0), vel, 1287, 1.0); // :86030
    } else if npc_type == 299 {
        gore.new_gore(rng, world, pos, vel, 433, 1.0); // :86034
    } else if npc_type == 538 {
        gore.new_gore(rng, world, pos, vel, 897, 1.0); // :86038
    } else if npc_type == 300 {
        gore.new_gore(rng, world, pos, vel, 434, 1.0); // :86042
    } else if npc_type == 610 {
        gore.new_gore(rng, world, pos, vel, 1163, 1.0); // :86046
    } else if npc_type == 626 {
        gore.new_gore(rng, world, pos, vel, 1197, 1.0); // :86050
    } else if npc_type == 102 {
        gore.new_gore(rng, world, pos, vel, 116, 1.0); // :86054
    } else if npc_type == 224 {
        gore.new_gore(rng, world, pos, vel, 312, 1.0); // :86058
        gore.new_gore(rng, world, pos, vel, 313, 1.0); // :86059
    } else if npc_type == 587 {
        // `:86061-86069` — the 587 Eyeball Flying Fish: an EXTRA 50x
        // dust-5 burst at (2.5*hd, -2.5) runs AFTER the shared 10x burst
        // and BEFORE the two gores (the C# order).
        let (w, h) = npc.size();
        for _ in 0..50 {
            dust.new_dust(
                rng,
                dust_ctx,
                world,
                pos,
                w as i32,
                h as i32,
                5,
                2.5 * hit_direction,
                -2.5,
                0,
                Color::default(),
                1.0,
            );
        }
        gore.new_gore(rng, world, pos, vel, 1104, 1.0); // :86067
        gore.new_gore(rng, world, pos, vel, 1105, 1.0); // :86068
    } else if npc_type == 616 {
        gore.new_gore(rng, world, pos, vel, 1166, 1.0); // :86072
        gore.new_gore(rng, world, pos, vel, 1167, 1.0); // :86073
        gore.new_gore(rng, world, pos, vel, 1168, 1.0); // :86074
        gore.new_gore(rng, world, pos, vel, 1168, 1.0); // :86075
    } else if npc_type == 617 {
        gore.new_gore(rng, world, pos, vel, 1169, 1.0); // :86079
        gore.new_gore(rng, world, pos, vel, 1170, 1.0); // :86080
        gore.new_gore(rng, world, pos, vel, 1171, 1.0); // :86081
        gore.new_gore(rng, world, pos, vel, 1171, 1.0); // :86082
    } else if npc_type == 625 {
        gore.new_gore(rng, world, pos, vel, 1194, 1.0); // :86086
        gore.new_gore(rng, world, pos, vel, 1195, 1.0); // :86087
        gore.new_gore(rng, world, pos, vel, 1196, 1.0); // :86088
        gore.new_gore(rng, world, pos, vel, 1196, 1.0); // :86089
    }
}

/// The 148/149/168/470 penguin arm (`NPC.cs:86092-86127`, sites
/// 94-102): the 10x dust-5 burst, then the per-kind pair/triple at
/// position, scale 1f. The 148 Penguin → 160/161 (`:86108-86109`), the
/// 149 Black Penguin → 162/163 (`:86113-86114`), the 168 Corrupt Penguin
/// → 215/216 (`:86118-86119`), the 470 Crimson Penguin → the triple
/// 673/672/674 (`:86123-86125`; NOTE 673 precedes 672, not a ladder).
fn penguin_frog_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86102-86105` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc_type == 148 {
        gore.new_gore(rng, world, pos, vel, 160, 1.0); // :86108
        gore.new_gore(rng, world, pos, vel, 161, 1.0); // :86109
    } else if npc_type == 149 {
        gore.new_gore(rng, world, pos, vel, 162, 1.0); // :86113
        gore.new_gore(rng, world, pos, vel, 163, 1.0); // :86114
    } else if npc_type == 168 {
        gore.new_gore(rng, world, pos, vel, 215, 1.0); // :86118
        gore.new_gore(rng, world, pos, vel, 216, 1.0); // :86119
    } else if npc_type == 470 {
        gore.new_gore(rng, world, pos, vel, 673, 1.0); // :86123
        gore.new_gore(rng, world, pos, vel, 672, 1.0); // :86124
        gore.new_gore(rng, world, pos, vel, 674, 1.0); // :86125
    }
}

/// The 361 Frog death (`NPC.cs:86128-86144`, sites 103-104):
/// the 10x dust-5 burst, then 551/552 at position, scale 1f.
fn frog_361_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86138-86141` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 551, 1.0); // :86142
    gore.new_gore(rng, world, pos, vel, 552, 1.0); // :86143
}

/// The 688 Pufferfish death (`NPC.cs:86190-86216`, sites 107-112): the
/// 10x dust-5 burst, then the `ai[2] == 1f` puffed branch spawns
/// 1388/1388/1389/1390, else the deflated pair 1386/1387 — all at
/// position, scale 1f. The `ai[2]` axis rides the NPC's own ai array.
fn pufferfish_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86200-86203` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc.ai[2] == 1.0 {
        gore.new_gore(rng, world, pos, vel, 1388, 1.0); // :86206
        gore.new_gore(rng, world, pos, vel, 1388, 1.0); // :86207
        gore.new_gore(rng, world, pos, vel, 1389, 1.0); // :86208
        gore.new_gore(rng, world, pos, vel, 1390, 1.0); // :86209
    } else {
        gore.new_gore(rng, world, pos, vel, 1386, 1.0); // :86213
        gore.new_gore(rng, world, pos, vel, 1387, 1.0); // :86214
    }
}

/// The 366/367 scorpion death (`NPC.cs:86217-86243`, sites 113-118): the
/// 10x dust-203 burst (the golden dust, NOT 5), then the per-kind triple
/// — 366 Black Scorpion → 561/562/563 (`:86233-86235`), 367 Scorpion →
/// 564/565/566 (`:86239-86241`). All at position, scale 1f.
fn scorpion_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86227-86230` — the 10x dust-203 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        203,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc_type == 366 {
        gore.new_gore(rng, world, pos, vel, 561, 1.0); // :86233
        gore.new_gore(rng, world, pos, vel, 562, 1.0); // :86234
        gore.new_gore(rng, world, pos, vel, 563, 1.0); // :86235
    } else {
        gore.new_gore(rng, world, pos, vel, 564, 1.0); // :86239
        gore.new_gore(rng, world, pos, vel, 565, 1.0); // :86240
        gore.new_gore(rng, world, pos, vel, 566, 1.0); // :86241
    }
}

/// The 362-365 duck death (`NPC.cs:86244-86270`, sites 119-121 per pair):
/// the 10x dust-5 burst, then the pair split — 362/363 → 555/556/557
/// (`:86260-86262`), 364/365 white → 558/559/560 (`:86266-86268`). All at
/// position, scale 1f.
fn duck_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86254-86257` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc_type == 362 || npc_type == 363 {
        gore.new_gore(rng, world, pos, vel, 555, 1.0); // :86260
        gore.new_gore(rng, world, pos, vel, 556, 1.0); // :86261
        gore.new_gore(rng, world, pos, vel, 557, 1.0); // :86262
    } else {
        gore.new_gore(rng, world, pos, vel, 558, 1.0); // :86266
        gore.new_gore(rng, world, pos, vel, 559, 1.0); // :86267
        gore.new_gore(rng, world, pos, vel, 560, 1.0); // :86268
    }
}

/// The 602/603 seagull death (`NPC.cs:86271-86290`, sites 122-123): the
/// 10x dust-5 burst, then 1150/1151 at position, scale 1f. The C#
/// re-checks `type == 602 || type == 603` inside the arm (`:86285`) —
/// always true here, ported flat.
fn seagull_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86281-86284` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1150, 1.0); // :86287
    gore.new_gore(rng, world, pos, vel, 1151, 1.0); // :86288
}

/// The 607 Pupfish death (`NPC.cs:86173-86189`, sites 105-106): the hit
/// half puffs dust-45 (g62's), the death half the 10x dust-5 burst then
/// 1152/1153 at position, scale 1f.
fn pupfish_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86183-86186` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1152, 1.0); // :86187
    gore.new_gore(rng, world, pos, vel, 1153, 1.0); // :86188
}

/// The 608/609 grebe death (`NPC.cs:86291-86311`, sites 124-126): the
/// 10x dust-5 burst, then 1154/1155/1156 at position, scale 1f (the
/// inner type re-check always true, ported flat).
fn grebe_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86301-86304` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1154, 1.0); // :86307
    gore.new_gore(rng, world, pos, vel, 1155, 1.0); // :86308
    gore.new_gore(rng, world, pos, vel, 1156, 1.0); // :86309
}

/// The 348/349 Nutcracker death (`NPC.cs:86312-86331`, sites 127-131):
/// the 10x dust-5 burst, then the five gores 528/529/530/531/532 at
/// position, scale 1f.
fn nutcracker_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86322-86325` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 528, 1.0); // :86326
    gore.new_gore(rng, world, pos, vel, 529, 1.0); // :86327
    gore.new_gore(rng, world, pos, vel, 530, 1.0); // :86328
    gore.new_gore(rng, world, pos, vel, 531, 1.0); // :86329
    gore.new_gore(rng, world, pos, vel, 532, 1.0); // :86330
}

/// The 351 Krampus death (`NPC.cs:86332-86351`, sites 132-136): the 10x
/// dust-5 burst, then the five gores 538/539/540/541/542 at position,
/// scale 1f.
fn krampus_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86342-86345` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 538, 1.0); // :86346
    gore.new_gore(rng, world, pos, vel, 539, 1.0); // :86347
    gore.new_gore(rng, world, pos, vel, 540, 1.0); // :86348
    gore.new_gore(rng, world, pos, vel, 541, 1.0); // :86349
    gore.new_gore(rng, world, pos, vel, 542, 1.0); // :86350
}

/// The 350 Elf Archer death (`NPC.cs:86352-86370`, sites 137-140): the
/// 10x dust-5 burst, then the four gores 533/534/535/536 at position,
/// scale 1f.
fn elf_archer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86362-86365` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 533, 1.0); // :86366
    gore.new_gore(rng, world, pos, vel, 534, 1.0); // :86367
    gore.new_gore(rng, world, pos, vel, 535, 1.0); // :86368
    gore.new_gore(rng, world, pos, vel, 536, 1.0); // :86369
}

/// The 47/57/58/464/465 corrupt/crimson critter arm
/// (`NPC.cs:86371-86408`, sites 141-148): the 10x dust-5 burst, then the
/// per-kind sub-branch — 57 Corrupt Goldfish → 84 (`:86387`), 58 Piranha
/// → 85 (`:86391`), 464 Crimson Bunny → 668/669 (`:86395-86396`), 465
/// Crimson Goldfish → 670/671 (`:86400-86401`), the 47 Corrupt Bunny
/// fallthrough → 78/79 (`:86405-86406`). All at position, scale 1f.
fn corrupt_critter_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86381-86384` — the 10x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        10,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc_type == 57 {
        gore.new_gore(rng, world, pos, vel, 84, 1.0); // :86387
    } else if npc_type == 58 {
        gore.new_gore(rng, world, pos, vel, 85, 1.0); // :86391
    } else if npc_type == 464 {
        gore.new_gore(rng, world, pos, vel, 668, 1.0); // :86395
        gore.new_gore(rng, world, pos, vel, 669, 1.0); // :86396
    } else if npc_type == 465 {
        gore.new_gore(rng, world, pos, vel, 670, 1.0); // :86400
        gore.new_gore(rng, world, pos, vel, 671, 1.0); // :86401
    } else {
        // `:86403-86407` — the 47 Corrupt Bunny fallthrough.
        gore.new_gore(rng, world, pos, vel, 78, 1.0); // :86405
        gore.new_gore(rng, world, pos, vel, 79, 1.0); // :86406
    }
}

/// The 173/174/181/182/268 crimson mob arm (`NPC.cs:86409-86448`, sites
/// 149-158): the 50x dust-5 burst carries `this.alpha` (the fade-in
/// mobs), then the per-kind sub-branch — 173 Crimera → 223/224
/// (`:86425-86426`), 174 Herpling → 225/226 (`:86430-86431`), 181 Face
/// Monster → 237 (`:86435`), 182 Floaty Gross → 238/239
/// (`:86439-86440`), 268 Ichor Sticker → 403/404/405 (`:86444-86446`).
/// All at position, scale 1f.
fn crimson_mob_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86419-86422` — the 50x dust-5 burst, alpha = this.alpha.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        npc.alpha as i32,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc_type == 173 {
        gore.new_gore(rng, world, pos, vel, 223, 1.0); // :86425
        gore.new_gore(rng, world, pos, vel, 224, 1.0); // :86426
    } else if npc_type == 174 {
        gore.new_gore(rng, world, pos, vel, 225, 1.0); // :86430
        gore.new_gore(rng, world, pos, vel, 226, 1.0); // :86431
    } else if npc_type == 181 {
        gore.new_gore(rng, world, pos, vel, 237, 1.0); // :86435
    } else if npc_type == 182 {
        gore.new_gore(rng, world, pos, vel, 238, 1.0); // :86439
        gore.new_gore(rng, world, pos, vel, 239, 1.0); // :86440
    } else if npc_type == 268 {
        gore.new_gore(rng, world, pos, vel, 403, 1.0); // :86444
        gore.new_gore(rng, world, pos, vel, 404, 1.0); // :86445
        gore.new_gore(rng, world, pos, vel, 405, 1.0); // :86446
    }
}

/// The demon-eye variant arm (`NPC.cs:86449-86503`, sites 159-174): the
/// 50x dust-5 burst, then the per-kind pair — head gore at position + the
/// second at `position.X + 14`, BOTH carrying `this.scale` (unlike the
/// critter rows). The sub-branches: 190 Cataract Eye → 249/2
/// (`:86465-86466`), 191 Sleepy Eye → 248/2 (`:86470-86471`), 192 Dilated
/// Eye → 247/2 (`:86475-86476`), 193 Green Eye → 252/253
/// (`:86480-86481`), 194 Purple Eye → 250/251 (`:86485-86486`), 317 Owl
/// Demon Eye → 447/448 (`:86490-86491`), 318 Spaceship Demon Eye →
/// 449/450 (`:86495-86496`), the 2 Demon Eye fallthrough → 1/2
/// (`:86500-86501`).
fn demon_eye_variant_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86459-86462` — the 50x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let pos14 = Vec2::new(npc.x + 14.0, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    if npc_type == 190 {
        gore.new_gore(rng, world, pos, vel, 249, s); // :86465
        gore.new_gore(rng, world, pos14, vel, 2, s); // :86466
    } else if npc_type == 191 {
        gore.new_gore(rng, world, pos, vel, 248, s); // :86470
        gore.new_gore(rng, world, pos14, vel, 2, s); // :86471
    } else if npc_type == 192 {
        gore.new_gore(rng, world, pos, vel, 247, s); // :86475
        gore.new_gore(rng, world, pos14, vel, 2, s); // :86476
    } else if npc_type == 193 {
        gore.new_gore(rng, world, pos, vel, 252, s); // :86480
        gore.new_gore(rng, world, pos14, vel, 253, s); // :86481
    } else if npc_type == 194 {
        gore.new_gore(rng, world, pos, vel, 250, s); // :86485
        gore.new_gore(rng, world, pos14, vel, 251, s); // :86486
    } else if npc_type == 317 {
        gore.new_gore(rng, world, pos, vel, 447, s); // :86490
        gore.new_gore(rng, world, pos14, vel, 448, s); // :86491
    } else if npc_type == 318 {
        gore.new_gore(rng, world, pos, vel, 449, s); // :86495
        gore.new_gore(rng, world, pos14, vel, 450, s); // :86496
    } else {
        // `:86498-86502` — the 2 Demon Eye fallthrough.
        gore.new_gore(rng, world, pos, vel, 1, s); // :86500
        gore.new_gore(rng, world, pos14, vel, 2, s); // :86501
    }
}

/// The 157 Arapaima death (`NPC.cs:86504-86520`, sites 175-176): the 50x
/// dust-5 burst, then 186 at position + 187 at +14 X — scale defaults to
/// 1f (no this.scale argument here).
fn arapaima_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86514-86517` — the 50x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 186, 1.0); // :86518
    gore.new_gore(rng, world, Vec2::new(pos.x + 14.0, pos.y), vel, 187, 1.0); // :86519
}

/// The 133 Wandering Eye death (`NPC.cs:86521-86544`, sites 177-178):
/// the death half's 50x dust-5 burst, then 155 at position + 155 at
/// +14 Y, scale 1f. The HIT half's `life < lifeMax*0.5 && localAI[0] == 0`
/// one-shot gore 1 (`:86529-86533`) is a hit-half seam — g62's lane, not
/// ported here (the dispatcher's hp gate skips it).
fn wandering_eye_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86537-86540` — the 50x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 155, 1.0); // :86541
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 14.0), vel, 155, 1.0); // :86542
}

/// The 61 Vulture death (`NPC.cs:86607-86624`, sites 179-181): the 50x
/// dust-5 burst, then 86 at position + 87/88 at +14 X, scale 1f.
fn vulture_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86617-86620` — the 50x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let pos14 = Vec2::new(npc.x + 14.0, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 86, 1.0); // :86621
    gore.new_gore(rng, world, pos14, vel, 87, 1.0); // :86622
    gore.new_gore(rng, world, pos14, vel, 88, 1.0); // :86623
}

/// The antlion arm (`NPC.cs:86545-86606`, sites 182-203): the dust-250
/// census — `num513 = 100` hit / `num514 = 50` death (the 582 Larvae's
/// 30/10, `:86549-86553`), then the per-kind sub-branch: 69 Antlion →
/// 97/98 (`:86568-86569`), 580 Walking Antlion → 1094/1095/1096/1097
/// (`:86573-86576`), 581 Flying Antlion → 1098/1099/1100/1101/1101
/// (`:86580-86584`), 508 Giant Walking Antlion → 811/812/813/814
/// (`:86588-86591`), 509 Giant Flying Antlion → 815/816/817/818/818
/// (`:86595-86599`), 582 Larvae Antlion → 1102/1103 (`:86603-86604`,
/// the 10-dust count). All at position, scale 1f.
fn antlion_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86547-86553` — the death count (the 582's 10, else 50).
    let num514 = if npc_type == 582 { 10 } else { 50 };
    // `:86562-86565` — the dust-250 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        250,
        hit_direction,
        num514,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc_type == 69 {
        gore.new_gore(rng, world, pos, vel, 97, 1.0); // :86568
        gore.new_gore(rng, world, pos, vel, 98, 1.0); // :86569
    } else if npc_type == 580 {
        gore.new_gore(rng, world, pos, vel, 1094, 1.0); // :86573
        gore.new_gore(rng, world, pos, vel, 1095, 1.0); // :86574
        gore.new_gore(rng, world, pos, vel, 1096, 1.0); // :86575
        gore.new_gore(rng, world, pos, vel, 1097, 1.0); // :86576
    } else if npc_type == 581 {
        gore.new_gore(rng, world, pos, vel, 1098, 1.0); // :86580
        gore.new_gore(rng, world, pos, vel, 1099, 1.0); // :86581
        gore.new_gore(rng, world, pos, vel, 1100, 1.0); // :86582
        gore.new_gore(rng, world, pos, vel, 1101, 1.0); // :86583
        gore.new_gore(rng, world, pos, vel, 1101, 1.0); // :86584
    } else if npc_type == 508 {
        gore.new_gore(rng, world, pos, vel, 811, 1.0); // :86588
        gore.new_gore(rng, world, pos, vel, 812, 1.0); // :86589
        gore.new_gore(rng, world, pos, vel, 813, 1.0); // :86590
        gore.new_gore(rng, world, pos, vel, 814, 1.0); // :86591
    } else if npc_type == 509 {
        gore.new_gore(rng, world, pos, vel, 815, 1.0); // :86595
        gore.new_gore(rng, world, pos, vel, 816, 1.0); // :86596
        gore.new_gore(rng, world, pos, vel, 817, 1.0); // :86597
        gore.new_gore(rng, world, pos, vel, 818, 1.0); // :86598
        gore.new_gore(rng, world, pos, vel, 818, 1.0); // :86599
    } else if npc_type == 582 {
        gore.new_gore(rng, world, pos, vel, 1102, 1.0); // :86603
        gore.new_gore(rng, world, pos, vel, 1103, 1.0); // :86604
    }
}

/// The 301 Raven death (`NPC.cs:86625-86641`, sites 204-205): the 50x
/// dust-5 burst, then 438 at position + 439 at +14 X, scale 1f.
fn raven_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86635-86638` — the 50x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 438, 1.0); // :86639
    gore.new_gore(rng, world, Vec2::new(pos.x + 14.0, pos.y), vel, 439, 1.0); // :86640
}

/// The 252 Parrot death (`NPC.cs:86642-86658`, sites 206-207): the 50x
/// dust-5 burst, then 373/374 at position, scale 1f.
fn parrot_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86652-86655` — the 50x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        50,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 373, 1.0); // :86656
    gore.new_gore(rng, world, pos, vel, 374, 1.0); // :86657
}

/// The 489/586 blood zombie arm (`NPC.cs:86659-86689`, sites 208-217):
/// the 50x dust-5 burst at (2.5·hd, −2.5) — the WIDER velocity of the
/// blood-moon rows — then the per-kind ladder carrying `this.scale`:
/// 489 Blood Zombie → 722/723/723/724/724 (`:86675-86679`), 586 Zombie
/// Merman → 1106/1107/1107/1108/1108 (`:86683-86687`), at the
/// +0/+20/+20/+34/+34 Y ladder. NOTE the C# uses two independent ifs
/// (`:86673`, `:86681`), not else-if — mutually exclusive on type, so
/// the port's else-if is equivalent.
fn blood_zombie_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86669-86672` — the 50x dust-5 burst at the wider velocity.
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng,
            dust_ctx,
            world,
            pos,
            w as i32,
            h as i32,
            5,
            2.5 * hit_direction,
            -2.5,
            0,
            Color::default(),
            1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let (head, mid, legs) = if npc_type == 489 {
        (722, 723, 724) // :86675-86679
    } else {
        (1106, 1107, 1108) // :86683-86687
    };
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, mid, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, mid, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, legs, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, legs, s);
}

/// The 534 Demon Tax Collector death (`NPC.cs:86690-86709`, sites
/// 218-222): the 50x dust-5 burst at (2.5·hd, −2.5), then the ladder
/// 894/895/895/896/896 at +0/+20/+20/+34/+34 Y, `this.scale` carried.
fn tax_collector_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86700-86703` — the 50x dust-5 burst at the wider velocity.
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng,
            dust_ctx,
            world,
            pos,
            w as i32,
            h as i32,
            5,
            2.5 * hit_direction,
            -2.5,
            0,
            Color::default(),
            1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 894, s); // :86704
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 895, s); // :86705
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 895, s); // :86706
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 896, s); // :86707
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 896, s); // :86708
}

/// The 490 Drippler death (`NPC.cs:86710-86729`, sites 223-227): the 50x
/// dust-5 burst at (2.5·hd, −2.5), then 725/726/727 at position, then
/// TWO `725 + Next(3)` random picks — the argument draws evaluate BEFORE
/// each NewGore (one Next(3) per gore, in source order), `this.scale`
/// carried.
fn drippler_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86720-86723` — the 50x dust-5 burst at the wider velocity.
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng,
            dust_ctx,
            world,
            pos,
            w as i32,
            h as i32,
            5,
            2.5 * hit_direction,
            -2.5,
            0,
            Color::default(),
            1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 725, s); // :86724
    gore.new_gore(rng, world, pos, vel, 726, s); // :86725
    gore.new_gore(rng, world, pos, vel, 727, s); // :86726
    // `:86727` — the first random pick: Next(3) BEFORE the NewGore.
    let pick1 = 725 + rng.next_max(3);
    gore.new_gore(rng, world, pos, vel, pick1, s);
    // `:86728` — the second random pick.
    let pick2 = 725 + rng.next_max(3);
    gore.new_gore(rng, world, pos, vel, pick2, s);
}

/// The shark arm (`NPC.cs:86730-86786`, sites 228-251): the 75x dust-5
/// burst, then the per-kind quad — head at position + three at +14 X,
/// ALL at `velocity * 0.8` (the dampened kick base), scale 1f. The
/// branches: 65 Shark → 89/90/91/92 (`:86746-86749`), 692 Orca →
/// 1403/1404/1405/1406 (`:86753-86756`), the 542-545 sand sharks →
/// 944+n ladder (`:86758-86785`, base 944/948/952/956 + the 0/1/2/3
/// tail). The C# uses independent ifs — mutually exclusive on type, the
/// port's else-if is equivalent.
fn shark_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:86740-86743` — the 75x dust-5 burst.
    death_dust_loop(
        npc,
        dust,
        dust_ctx,
        rng,
        world,
        5,
        hit_direction,
        75,
        0,
        Color::default(),
        1.0,
    );
    let pos = Vec2::new(npc.x, npc.y);
    let pos14 = Vec2::new(npc.x + 14.0, npc.y);
    // `:86746` — the dampened velocity base.
    let vel = Vec2::new(npc.vx, npc.vy) * 0.8;
    if npc_type == 65 {
        gore.new_gore(rng, world, pos, vel, 89, 1.0); // :86746
        gore.new_gore(rng, world, pos14, vel, 90, 1.0); // :86747
        gore.new_gore(rng, world, pos14, vel, 91, 1.0); // :86748
        gore.new_gore(rng, world, pos14, vel, 92, 1.0); // :86749
    } else if npc_type == 692 {
        gore.new_gore(rng, world, pos, vel, 1403, 1.0); // :86753
        gore.new_gore(rng, world, pos14, vel, 1404, 1.0); // :86754
        gore.new_gore(rng, world, pos14, vel, 1405, 1.0); // :86755
        gore.new_gore(rng, world, pos14, vel, 1406, 1.0); // :86756
    } else if npc_type == 542 {
        gore.new_gore(rng, world, pos, vel, 944, 1.0); // :86760
        gore.new_gore(rng, world, pos14, vel, 945, 1.0); // :86761
        gore.new_gore(rng, world, pos14, vel, 946, 1.0); // :86762
        gore.new_gore(rng, world, pos14, vel, 947, 1.0); // :86763
    } else if npc_type == 543 {
        gore.new_gore(rng, world, pos, vel, 948, 1.0); // :86767
        gore.new_gore(rng, world, pos14, vel, 949, 1.0); // :86768
        gore.new_gore(rng, world, pos14, vel, 950, 1.0); // :86769
        gore.new_gore(rng, world, pos14, vel, 951, 1.0); // :86770
    }
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
    // `:85742-85759` — the 494/495 Crawdad arm (sites 10-11).
    else if (494..=495).contains(&npc_type) {
        crawdad_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85760-85777` — the 496/497 Giant Shelly arm (site 12).
    else if (496..=497).contains(&npc_type) {
        shelly_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85778-85796` — the 498-506 Salamander arm (sites 13-15 per variant).
    else if (498..=506).contains(&npc_type) {
        salamander_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85797-85841` — the bat family arm (49/51/93/150/152/226, sites
    // 16-27, all six sub-branches).
    else if matches!(npc_type, 49 | 51 | 93 | 150 | 152 | 226) {
        bat_family_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85842-85859` — the 611/689 Owl arm (sites 28-30).
    else if npc_type == 611 || npc_type == 689 {
        owl_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85860-85878` — the 624 Gnome arm (sites 31-34).
    else if npc_type == 624 {
        gnome_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85879-85897` — the 628 Dandelion arm (sites 35-38).
    else if npc_type == 628 {
        dandelion_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85898-85920` — the 646-652 Gem Bunny arm (sites 39-40).
    else if (646..=652).contains(&npc_type) {
        gem_bunny_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85921-85941` — the 639-645 Gem Squirrel arm (site 41).
    else if (639..=645).contains(&npc_type) {
        gem_squirrel_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85942-86091` — the 29-type critter arm, COMPLETE (sites 42-93):
    // every gore sub-branch landed; the 614/627 kinds have no gore
    // sub-branch in the C# (dust burst only).
    else if matches!(
        npc_type,
        46 | 55
            | 67
            | 74
            | 102
            | 224
            | 230
            | 297
            | 298
            | 299
            | 300
            | 303
            | 337
            | 538
            | 540
            | 587
            | 610
            | 614
            | 616
            | 617
            | 625
            | 626
            | 627
            | 615
            | 671
            | 672
            | 673
            | 674
            | 675
    ) {
        critter_arm_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86092-86127` — the 148/149/168/470 penguin arm, COMPLETE (sites
    // 94-102).
    else if matches!(npc_type, 148 | 149 | 168 | 470) {
        penguin_frog_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86128-86144` — the 361 Frog arm (sites 103-104).
    else if npc_type == 361 {
        frog_361_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86145-86158` — the 687 bound-town-slime arm: NO gore sites (the
    // hit half puffs dust-16, the death half PoofOfSmoke — the dust
    // lanes'; recorded, not ported).
    // `:86159-86172` — the 685 bound-town-slime arm: NO gore sites (the
    // hit half dust-0, the death half PoofOfSmoke; same drop).
    // `:86173-86189` — the 607 Pupfish arm (sites 105-106).
    else if npc_type == 607 {
        pupfish_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86190-86216` — the 688 Pufferfish arm (sites 107-112, the ai[2]
    // puffed/deflated split).
    else if npc_type == 688 {
        pufferfish_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86217-86243` — the 366/367 scorpion arm, COMPLETE (sites
    // 113-118).
    else if npc_type == 366 || npc_type == 367 {
        scorpion_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86244-86270` — the 362-365 duck arm (sites 119-121 per pair).
    else if (362..=365).contains(&npc_type) {
        duck_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86271-86290` — the 602/603 seagull arm (sites 122-123).
    else if npc_type == 602 || npc_type == 603 {
        seagull_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86291-86311` — the 608/609 grebe arm (sites 124-126).
    else if npc_type == 608 || npc_type == 609 {
        grebe_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86312-86331` — the 348/349 Nutcracker arm (sites 127-131).
    else if npc_type == 348 || npc_type == 349 {
        nutcracker_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86332-86351` — the 351 Krampus arm (sites 132-136).
    else if npc_type == 351 {
        krampus_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86352-86370` — the 350 Elf Archer arm (sites 137-140).
    else if npc_type == 350 {
        elf_archer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86371-86408` — the 47/57/58/464/465 corrupt/crimson critter arm
    // (sites 141-148).
    else if matches!(npc_type, 47 | 57 | 58 | 464 | 465) {
        corrupt_critter_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86409-86448` — the 173/174/181/182/268 crimson mob arm (sites
    // 149-158, the alpha-carrying 50x dust burst).
    else if matches!(npc_type, 173 | 174 | 181 | 182 | 268) {
        crimson_mob_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86449-86503` — the demon-eye variant arm, COMPLETE (sites
    // 159-174).
    else if matches!(npc_type, 2 | 190 | 191 | 192 | 193 | 194 | 317 | 318) {
        demon_eye_variant_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86504-86520` — the 157 Arapaima arm (sites 175-176).
    else if npc_type == 157 {
        arapaima_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86521-86544` — the 133 Wandering Eye arm (sites 177-178; the hit
    // half's half-life one-shot gore is g62's seam).
    else if npc_type == 133 {
        wandering_eye_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86545-86606` — the antlion arm, COMPLETE (sites 182-203).
    else if matches!(npc_type, 69 | 580 | 581 | 508 | 509 | 582) {
        antlion_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86607-86624` — the 61 Vulture arm (sites 179-181).
    else if npc_type == 61 {
        vulture_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86625-86641` — the 301 Raven arm (sites 204-205).
    else if npc_type == 301 {
        raven_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86642-86658` — the 252 Parrot arm (sites 206-207).
    else if npc_type == 252 {
        parrot_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86659-86689` — the 489/586 blood zombie arm (sites 208-217).
    else if npc_type == 489 || npc_type == 586 {
        blood_zombie_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86690-86709` — the 534 Demon Tax Collector arm (sites 218-222).
    else if npc_type == 534 {
        tax_collector_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86710-86729` — the 490 Drippler arm (sites 223-227, the two
    // Next(3) argument picks).
    else if npc_type == 490 {
        drippler_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86730-86786` — the shark arm (65/692/542-545); the 65/692/542/543
    // quads are live (sites 228-243), the 544/545 tails land next batch.
    else if matches!(npc_type, 65 | 692 | 542 | 543 | 544 | 545) {
        shark_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
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

    /// `NPC.cs:85757-85758` — both Crawdad variants: the per-variant pair
    /// with the HIGHER type first (494→736/735, 495→738/737).
    #[test]
    fn w4_f5d_the_crawdad_deaths_pick_the_variant_pair() {
        let w = air();
        let n = mob(100.0, 100.0, 44.0, 30.0);
        for (kind, want) in [(494, vec![736, 735]), (495, vec![738, 737])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(17);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "crawdad {kind}");
            assert!(!gore.slot(2).active, "exactly two gores");
        }
    }

    /// `NPC.cs:85775` — both Shelly variants: ONE gore 739/740.
    #[test]
    fn w4_f5d_the_shelly_deaths_spawn_one_gore() {
        let w = air();
        let n = mob(100.0, 100.0, 44.0, 30.0);
        for (kind, want) in [(496, 739), (497, 740)] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(19);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            assert_eq!(gore.slot(0).gore_type, want, "shelly {kind}");
            assert!(!gore.slot(1).active, "exactly one gore");
        }
    }

    /// `NPC.cs:85793-85795` — the Salamander triple: 750+n at position,
    /// 741+n at +16 Y, 759+n at +18 Y (n = type - 498); the 498 and 506
    /// ends pinned.
    #[test]
    fn w4_f5d_the_salamander_deaths_spawn_the_variant_triple() {
        let w = air();
        let n = mob(100.0, 100.0, 40.0, 34.0);
        for (kind, base) in [(498, 0), (506, 8)] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(23);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![750 + base, 741 + base, 759 + base], "salamander {kind}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 116.0));
            assert_eq!(gore.slot(2).position, Vec2::new(100.0, 118.0));
            assert!(!gore.slot(3).active, "exactly three gores");
        }
    }

    /// `NPC.cs:85742-85796` — the crawdad/shelly/salamander deaths all draw
    /// the 15x6 dust census then only their gores' two kicks each; the twin
    /// replay pins the stream (the named-argument :85775 draws no arm-level
    /// randoms — Type is arithmetic, Position/Velocity are data).
    #[test]
    fn w4_f5d_the_underground_deaths_mirror_the_csharp_draws() {
        let w = air();
        let n = mob(100.0, 100.0, 44.0, 30.0);
        for (kind, gores) in [(494, 2), (497, 1), (502, 3)] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(29);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let mut twin = UnifiedRandom::new(29);
            for _ in 0..15 {
                twin_dust(&mut twin, 44, 30);
            }
            for _ in 0..gores {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85813-85828` — the bat family's per-kind gores: 51→83,
    /// 93→107, 150→164/165, 152→175/176/176, all at position scale 1f.
    #[test]
    fn w4_f5d_the_bat_family_deaths_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        for (kind, want) in [
            (51, vec![83]),
            (93, vec![107]),
            (150, vec![164, 165]),
            (152, vec![175, 176, 176]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(31);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "bat kind {kind}");
            for (i, _) in want.iter().enumerate() {
                assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
            }
        }
    }

    /// `NPC.cs:85807-85828` — the bat deaths draw the 15x6 dust census
    /// then only their gores' two kicks each; the twin replay pins the
    /// stream for every live sub-branch.
    #[test]
    fn w4_f5d_the_bat_deaths_mirror_the_csharp_draws() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        for (kind, gores) in [(51, 1), (93, 1), (150, 2), (152, 3)] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let mut twin = UnifiedRandom::new(37);
            for _ in 0..15 {
                twin_dust(&mut twin, 30, 26);
            }
            for _ in 0..gores {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "bat kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85832-85839` — the bat tail: 226 Flying Snake
    /// 317/318/318/319, the 49 Cave Bat fallthrough 82.
    #[test]
    fn w4_f5d_the_bat_tail_spawns_the_snake_and_cave_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        for (kind, want) in [(226, vec![317, 318, 318, 319]), (49, vec![82])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(41);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "bat kind {kind}");
            let mut twin = UnifiedRandom::new(41);
            for _ in 0..15 {
                twin_dust(&mut twin, 30, 26);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "bat kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85856-85858` — the Owl/Owl Mimic death: 1164/1164/1165
    /// after the 15x dust-5 burst; both kinds share the row.
    #[test]
    fn w4_f5d_the_owl_deaths_spawn_the_triple() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        for kind in [611, 689] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(43);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![1164, 1164, 1165], "owl kind {kind}");
            assert!(!gore.slot(3).active, "exactly three gores");
            let mut twin = UnifiedRandom::new(43);
            for _ in 0..15 {
                twin_dust(&mut twin, 30, 26);
            }
            for _ in 0..3 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "owl kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85874-85877` — the Gnome death: the count drops to 10 dust
    /// (not 15), then 1190/1191/1192/1193.
    #[test]
    fn w4_f5d_the_gnome_death_drops_the_dust_count_to_ten() {
        let w = air();
        let n = mob(100.0, 100.0, 26.0, 32.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let _ = hit_effect_d(&n, 624, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1190, 1191, 1192, 1193]);
        assert!(!gore.slot(4).active, "exactly four gores");
        let mut twin = UnifiedRandom::new(47);
        for _ in 0..10 {
            twin_dust(&mut twin, 26, 32);
        }
        for _ in 0..4 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the gnome death mirrors the C# draws");
    }

    /// `NPC.cs:85893-85896` — the Dandelion death: 10x dust-3 (the leaf
    /// puff, NOT dust-5), then 1205/1206/1206/1207.
    #[test]
    fn w4_f5d_the_dandelion_death_puffs_dust_three() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 34.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        let _ = hit_effect_d(&n, 628, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1205, 1206, 1206, 1207]);
        let leaf = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(leaf.dust_type, 3, "the dandelion puffs type 3");
        let mut twin = UnifiedRandom::new(53);
        for _ in 0..10 {
            twin_dust(&mut twin, 30, 34);
        }
        for _ in 0..4 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the dandelion death mirrors the C# draws");
    }

    /// `NPC.cs:85918-85919` — the Gem Bunny pair: per-variant offset
    /// (type-646)*2, the 652 variant's dust type is 138 (not 92).
    #[test]
    fn w4_f5d_the_gem_bunny_deaths_pick_the_variant_pair() {
        let w = air();
        let n = mob(100.0, 100.0, 22.0, 20.0);
        for (kind, want, dust_type) in [
            (646, vec![1234, 1235], 86),
            (649, vec![1240, 1241], 89),
            (652, vec![1246, 1247], 138),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(59);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "gem bunny {kind}");
            let gem = dust.slots().iter().find(|d| d.active).unwrap();
            assert_eq!(gem.dust_type, dust_type, "gem bunny {kind} dust");
            let mut twin = UnifiedRandom::new(59);
            for _ in 0..10 {
                twin_dust(&mut twin, 22, 20);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "gem bunny {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85940` — the Gem Squirrel single gore: 1227+(type-639),
    /// the 645 variant's dust type is 138.
    #[test]
    fn w4_f5d_the_gem_squirrel_deaths_spawn_one_gore() {
        let w = air();
        let n = mob(100.0, 100.0, 22.0, 20.0);
        for (kind, want, dust_type) in [(639, 1227, 86), (645, 1233, 138)] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(61);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            assert_eq!(gore.slot(0).gore_type, want, "gem squirrel {kind}");
            assert!(!gore.slot(1).active, "exactly one gore");
            let gem = dust.slots().iter().find(|d| d.active).unwrap();
            assert_eq!(gem.dust_type, dust_type, "gem squirrel {kind} dust");
            let mut twin = UnifiedRandom::new(61);
            for _ in 0..10 {
                twin_dust(&mut twin, 22, 20);
            }
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            assert_eq!(rng.peek(), twin.peek(), "gem squirrel {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85958-85969` — the critter arm's first branch: 46/337 →
    /// 76/77; the 303 Bunny adds the 10x dust-193 burst AFTER the gores
    /// (the stream order: 10 dust-5, 2 gores' kicks, 10 dust-193); the
    /// 540 adds the extra gore 926.
    #[test]
    fn w4_f5d_the_bunny_branch_spawns_the_pair_plus_extras() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, want) in [
            (46, vec![76, 77]),
            (337, vec![76, 77]),
            (303, vec![76, 77]),
            (540, vec![76, 77, 926]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(67);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "critter {kind}");
            let mut twin = UnifiedRandom::new(67);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            if kind == 303 {
                // The extra dust-193 burst runs AFTER the two gores.
                for _ in 0..10 {
                    twin_dust(&mut twin, 24, 22);
                }
                let has193 = dust.slots().iter().any(|d| d.active && d.dust_type == 193);
                assert!(has193, "the 303's extra dust-193 burst spawned");
            }
            if kind == 540 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "critter {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:85974-85976` — the 67 Goldfish branch: 95/95/96 after the
    /// 10x dust-5 burst.
    #[test]
    fn w4_f5d_the_goldfish_branch_spawns_the_triple() {
        let w = air();
        let n = mob(100.0, 100.0, 22.0, 18.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        let _ = hit_effect_d(&n, 67, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![95, 95, 96]);
        assert!(!gore.slot(3).active, "exactly three gores");
        let mut twin = UnifiedRandom::new(71);
        for _ in 0..10 {
            twin_dust(&mut twin, 22, 18);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the goldfish branch mirrors the C# draws");
    }

    /// `NPC.cs:85980-86000` — the critter singles/pairs: 55/230 → 553/554,
    /// 615 → 1198/1199/1200/1200, 74 → 100, 297 → 431, 298 → 432; all
    /// after the shared 10x dust-5 burst, draw census pinned per kind.
    #[test]
    fn w4_f5d_the_critter_singles_and_pairs_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, want) in [
            (55, vec![553, 554]),
            (230, vec![553, 554]),
            (615, vec![1198, 1199, 1200, 1200]),
            (74, vec![100]),
            (297, vec![431]),
            (298, vec![432]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(73);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "critter {kind}");
            let mut twin = UnifiedRandom::new(73);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "critter {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86004-86018` — the macaw/toucan triples: head at position,
    /// the wing pair at Center ±(4,0); Center = pos + size/2.
    #[test]
    fn w4_f5d_the_macaw_triples_anchor_the_wings_at_center() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, head, wing) in [(671, 1277, 1279), (672, 1280, 1281), (673, 1282, 1283)] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(79);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![head, wing, wing], "macaw {kind}");
            // Center = (112,111); the wings at (116,111)/(108,111).
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(116.0, 111.0));
            assert_eq!(gore.slot(2).position, Vec2::new(108.0, 111.0));
            assert!(!gore.slot(3).active, "exactly three gores");
            let mut twin = UnifiedRandom::new(79);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..3 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "macaw {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86022-86046` — the cockatiel triples (674→1284+1285x2,
    /// 675→1286+1287x2) and the critter singles (299→433, 538→897,
    /// 300→434, 610→1163); the draw census pinned per kind.
    #[test]
    fn w4_f5d_the_cockatiels_and_critter_singles_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, want) in [
            (674, vec![1284, 1285, 1285]),
            (675, vec![1286, 1287, 1287]),
            (299, vec![433]),
            (538, vec![897]),
            (300, vec![434]),
            (610, vec![1163]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(83);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "critter {kind}");
            if want.len() == 3 {
                assert_eq!(gore.slot(1).position, Vec2::new(116.0, 111.0));
                assert_eq!(gore.slot(2).position, Vec2::new(108.0, 111.0));
            }
            let mut twin = UnifiedRandom::new(83);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "critter {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86050-86075` — the water critters: 626 Seahorse 1197,
    /// 102 Angler Fish 116, 224 Flying Fish 312/313, 616 Turtle
    /// 1166/1167/1168/1168; the draw census pinned per kind.
    #[test]
    fn w4_f5d_the_water_critters_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, want) in [
            (626, vec![1197]),
            (102, vec![116]),
            (224, vec![312, 313]),
            (616, vec![1166, 1167, 1168, 1168]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(89);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "water critter {kind}");
            let mut twin = UnifiedRandom::new(89);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "water critter {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86061-86068` — the 587 Eyeball Flying Fish: the extra 50x
    /// dust-5 burst runs BETWEEN the shared 10x burst and the two gores
    /// (the stream order: 10 dust, 50 dust, 2 gores' kicks).
    #[test]
    fn w4_f5d_the_eyeball_fish_inserts_the_fifty_dust_burst() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        let _ = hit_effect_d(&n, 587, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1104, 1105]);
        let mut twin = UnifiedRandom::new(97);
        for _ in 0..10 {
            twin_dust(&mut twin, 30, 26);
        }
        for _ in 0..50 {
            twin_dust(&mut twin, 30, 26);
        }
        for _ in 0..2 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the eyeball fish mirrors the C# draws");
    }

    /// `NPC.cs:86079-86089` — the turtle quartets: 617 Jungle Turtle
    /// 1169/1170/1171/1171, 625 Sea Turtle 1194/1195/1196/1196; the 614/627
    /// kinds have NO gore sub-branch (dust burst only).
    #[test]
    fn w4_f5d_the_turtle_quartets_and_the_goreless_kinds() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 24.0);
        for (kind, want) in [
            (617, vec![1169, 1170, 1171, 1171]),
            (625, vec![1194, 1195, 1196, 1196]),
            (614, vec![]),
            (627, vec![]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(101);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "turtle {kind}");
            let mut twin = UnifiedRandom::new(101);
            for _ in 0..10 {
                twin_dust(&mut twin, 30, 24);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "turtle {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86108-86119` — the penguin/frog pairs: 148→160/161,
    /// 149→162/163, 168→215/216, after the 10x dust-5 burst.
    #[test]
    fn w4_f5d_the_penguin_frog_pairs_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 26.0);
        for (kind, want) in [(148, vec![160, 161]), (149, vec![162, 163]), (168, vec![215, 216])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(103);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "penguin/frog {kind}");
            assert!(!gore.slot(2).active, "exactly two gores");
            let mut twin = UnifiedRandom::new(103);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 26);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "penguin/frog {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86123-86125` — the 470 Crimson Penguin triple: 673/672/674
    /// (NOT a ladder — 673 precedes 672).
    #[test]
    fn w4_f5d_the_crimson_penguin_triple_is_not_a_ladder() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 26.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(107);
        let _ = hit_effect_d(&n, 470, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![673, 672, 674]);
        assert!(!gore.slot(3).active, "exactly three gores");
        let mut twin = UnifiedRandom::new(107);
        for _ in 0..10 {
            twin_dust(&mut twin, 24, 26);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the crimson penguin mirrors the C# draws");
    }

    /// `NPC.cs:86142-86143`/`:86187-86188` — the 361 Frog pair 551/552 and
    /// the 607 Pupfish pair 1152/1153, both after the 10x dust-5 burst.
    #[test]
    fn w4_f5d_the_frog_and_pupfish_pairs_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, want) in [(361, vec![551, 552]), (607, vec![1152, 1153])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(109);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "kind {kind}");
            assert!(!gore.slot(2).active, "exactly two gores");
            let mut twin = UnifiedRandom::new(109);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86204-86214` — the Pufferfish split: ai[2]==1 puffs
    /// 1388/1388/1389/1390, else the deflated pair 1386/1387; the draw
    /// census pinned for both branches.
    #[test]
    fn w4_f5d_the_pufferfish_splits_on_ai_two() {
        let w = air();
        let mut n = mob(100.0, 100.0, 26.0, 24.0);
        for (puffed, want) in [(true, vec![1388, 1388, 1389, 1390]), (false, vec![1386, 1387])] {
            n.ai[2] = if puffed { 1.0 } else { 0.0 };
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(113);
            let _ = hit_effect_d(&n, 688, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "puffed={puffed}");
            let mut twin = UnifiedRandom::new(113);
            for _ in 0..10 {
                twin_dust(&mut twin, 26, 24);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "puffed={puffed} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86233-86235` — the 366 Black Scorpion triple 561/562/563
    /// after the 10x dust-203 burst (the golden dust, NOT 5).
    #[test]
    fn w4_f5d_the_black_scorpion_puffs_golden_dust() {
        let w = air();
        let n = mob(100.0, 100.0, 26.0, 20.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        let _ = hit_effect_d(&n, 366, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![561, 562, 563]);
        let golden = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(golden.dust_type, 203, "the scorpion puffs type 203");
        let mut twin = UnifiedRandom::new(127);
        for _ in 0..10 {
            twin_dust(&mut twin, 26, 20);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the black scorpion mirrors the C# draws");
    }

    /// `NPC.cs:86239-86241` — the 367 Scorpion triple 564/565/566, same
    /// dust-203 census.
    #[test]
    fn w4_f5d_the_scorpion_tail_spawns_the_second_triple() {
        let w = air();
        let n = mob(100.0, 100.0, 26.0, 20.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let _ = hit_effect_d(&n, 367, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![564, 565, 566]);
        let mut twin = UnifiedRandom::new(131);
        for _ in 0..10 {
            twin_dust(&mut twin, 26, 20);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the scorpion tail mirrors the C# draws");
    }

    /// `NPC.cs:86260-86268` — the duck pair split: 362/363 → 555/556/557,
    /// 364/365 white → 558/559/560; the draw census pinned per kind.
    #[test]
    fn w4_f5d_the_duck_deaths_split_on_the_white_pair() {
        let w = air();
        let n = mob(100.0, 100.0, 26.0, 24.0);
        for (kind, want) in [
            (362, vec![555, 556, 557]),
            (363, vec![555, 556, 557]),
            (364, vec![558, 559, 560]),
            (365, vec![558, 559, 560]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(137);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "duck {kind}");
            let mut twin = UnifiedRandom::new(137);
            for _ in 0..10 {
                twin_dust(&mut twin, 26, 24);
            }
            for _ in 0..3 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "duck {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86287-86288` — the seagull pair 1150/1151 after the 10x
    /// dust-5 burst (both kinds share the row).
    #[test]
    fn w4_f5d_the_seagull_deaths_spawn_the_pair() {
        let w = air();
        let n = mob(100.0, 100.0, 22.0, 16.0);
        for kind in [602, 603] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(139);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![1150, 1151], "seagull {kind}");
            assert!(!gore.slot(2).active, "exactly two gores");
            let mut twin = UnifiedRandom::new(139);
            for _ in 0..10 {
                twin_dust(&mut twin, 22, 16);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "seagull {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86307-86309` — the grebe triple 1154/1155/1156 after the
    /// 10x dust-5 burst (both kinds share the row).
    #[test]
    fn w4_f5d_the_grebe_deaths_spawn_the_triple() {
        let w = air();
        let n = mob(100.0, 100.0, 26.0, 22.0);
        for kind in [608, 609] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(149);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![1154, 1155, 1156], "grebe {kind}");
            assert!(!gore.slot(3).active, "exactly three gores");
            let mut twin = UnifiedRandom::new(149);
            for _ in 0..10 {
                twin_dust(&mut twin, 26, 22);
            }
            for _ in 0..3 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "grebe {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86326-86330` — the Nutcracker five: 528/529/530/531/532
    /// after the 10x dust-5 burst (both kinds share the row).
    #[test]
    fn w4_f5d_the_nutcracker_deaths_spawn_the_five() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 40.0);
        for kind in [348, 349] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(151);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![528, 529, 530, 531, 532], "nutcracker {kind}");
            assert!(!gore.slot(5).active, "exactly five gores");
            let mut twin = UnifiedRandom::new(151);
            for _ in 0..10 {
                twin_dust(&mut twin, 30, 40);
            }
            for _ in 0..5 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "nutcracker {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86346-86350` — the Krampus five 538-542; `:86366-86369` —
    /// the Elf Archer four 533-536; both after the 10x dust-5 burst.
    #[test]
    fn w4_f5d_the_krampus_and_elf_archer_spawn_the_frostmoon_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 40.0);
        for (kind, want) in [
            (351, vec![538, 539, 540, 541, 542]),
            (350, vec![533, 534, 535, 536]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(157);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "frostmoon {kind}");
            let mut twin = UnifiedRandom::new(157);
            for _ in 0..10 {
                twin_dust(&mut twin, 30, 40);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "frostmoon {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86387-86406` — the corrupt/crimson critter sub-branches:
    /// 57→84, 58→85, 464→668/669, 465→670/671, the 47 fallthrough→78/79;
    /// the draw census pinned per kind.
    #[test]
    fn w4_f5d_the_corrupt_critters_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 22.0);
        for (kind, want) in [
            (57, vec![84]),
            (58, vec![85]),
            (464, vec![668, 669]),
            (465, vec![670, 671]),
            (47, vec![78, 79]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "corrupt critter {kind}");
            let mut twin = UnifiedRandom::new(163);
            for _ in 0..10 {
                twin_dust(&mut twin, 24, 22);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "corrupt critter {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86425-86446` — the crimson mob sub-branches: 173→223/224,
    /// 174→225/226, 181→237, 182→238/239, 268→403/404/405; the 50x dust
    /// burst carries this.alpha (the fade-in mobs).
    #[test]
    fn w4_f5d_the_crimson_mobs_spawn_the_kind_gores_with_alpha_dust() {
        let w = air();
        let mut n = mob(100.0, 100.0, 30.0, 28.0);
        n.alpha = 120;
        for (kind, want) in [
            (173, vec![223, 224]),
            (174, vec![225, 226]),
            (181, vec![237]),
            (182, vec![238, 239]),
            (268, vec![403, 404, 405]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(167);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "crimson mob {kind}");
            let faded = dust.slots().iter().find(|d| d.active).unwrap();
            assert_eq!(faded.alpha, 120, "the dust carries this.alpha");
            let mut twin = UnifiedRandom::new(167);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 28);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "crimson mob {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86465-86486` — the demon-eye variant pairs: 190→249/2,
    /// 191→248/2, 192→247/2, 193→252/253, 194→250/251; the second gore at
    /// +14 X, BOTH carrying this.scale.
    #[test]
    fn w4_f5d_the_demon_eye_variants_carry_the_scale() {
        let w = air();
        let mut n = mob(100.0, 100.0, 30.0, 30.0);
        n.scale = 1.25;
        for (kind, want) in [
            (190, vec![249, 2]),
            (191, vec![248, 2]),
            (192, vec![247, 2]),
            (193, vec![252, 253]),
            (194, vec![250, 251]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(173);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "demon eye {kind}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
            assert_eq!(gore.slot(0).scale, 1.25, "the head carries this.scale");
            assert_eq!(gore.slot(1).scale, 1.25, "the second carries this.scale");
            assert!(!gore.slot(2).active, "exactly two gores");
            let mut twin = UnifiedRandom::new(173);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 30);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "demon eye {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86490-86501` — the demon-eye tail: 317 Owl → 447/448,
    /// 318 Spaceship → 449/450, the 2 fallthrough → 1/2; same +14 X /
    /// this.scale contract.
    #[test]
    fn w4_f5d_the_demon_eye_tail_spawns_the_variant_pairs() {
        let w = air();
        let mut n = mob(100.0, 100.0, 30.0, 30.0);
        n.scale = 0.9;
        for (kind, want) in [(317, vec![447, 448]), (318, vec![449, 450]), (2, vec![1, 2])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(179);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "demon eye {kind}");
            assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
            assert_eq!(gore.slot(0).scale, 0.9);
            let mut twin = UnifiedRandom::new(179);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 30);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "demon eye {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86518-86519`/`:86541-86542`/`:86621-86623` — the Arapaima
    /// pair 186/187 (+14 X), the Wandering Eye pair 155/155 (+14 Y), the
    /// Vulture triple 86/87/88 (+14 X); all scale 1f after the 50x dust-5
    /// burst.
    #[test]
    fn w4_f5d_the_arapaima_wandering_eye_vulture_spawn_the_pairs() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 30.0);
        for (kind, want) in [
            (157, vec![186, 187]),
            (133, vec![155, 155]),
            (61, vec![86, 87, 88]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(181);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "kind {kind}");
            match kind {
                133 => assert_eq!(gore.slot(1).position, Vec2::new(100.0, 114.0)),
                _ => assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0)),
            }
            for i in 0..want.len() {
                assert_eq!(gore.slot(i).scale, 1.0);
            }
            let mut twin = UnifiedRandom::new(181);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 30);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86568-86591` — the antlion branches: 69→97/98,
    /// 580→1094-1097, 508→811-814; the dust-250 census (50 death), Child
    /// Safety OFF (the pool default) so the blood gate draws nothing.
    #[test]
    fn w4_f5d_the_antlion_branches_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        for (kind, want) in [
            (69, vec![97, 98]),
            (580, vec![1094, 1095, 1096, 1097]),
            (508, vec![811, 812, 813, 814]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(191);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "antlion {kind}");
            let sandy = dust.slots().iter().find(|d| d.active).unwrap();
            assert_eq!(sandy.dust_type, 250, "the antlion puffs type 250");
            let mut twin = UnifiedRandom::new(191);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 26);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "antlion {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86580-86599` — the antlion flying tails: 581 Flying →
    /// 1098/1099/1100/1101/1101, 509 Giant Flying → 815/816/817/818/818
    /// (50 dust each).
    #[test]
    fn w4_f5d_the_antlion_flying_tails_spawn_the_five_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 26.0);
        for (kind, want) in [
            (581, vec![1098, 1099, 1100, 1101, 1101]),
            (509, vec![815, 816, 817, 818, 818]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(193);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore
                .slots()
                .iter()
                .take_while(|g| g.active)
                .map(|g| g.gore_type)
                .collect();
            assert_eq!(types, want, "antlion {kind}");
            let mut twin = UnifiedRandom::new(193);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 26);
            }
            for _ in 0..want.len() {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "antlion {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86603-86604` — the 582 Larvae Antlion pair 1102/1103 with
    /// the 10-dust count (num514 = 10, not 50).
    #[test]
    fn w4_f5d_the_larvae_antlion_drops_the_dust_count_to_ten() {
        let w = air();
        let n = mob(100.0, 100.0, 24.0, 20.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(197);
        let _ = hit_effect_d(&n, 582, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1102, 1103]);
        assert!(!gore.slot(2).active, "exactly two gores");
        let mut twin = UnifiedRandom::new(197);
        for _ in 0..10 {
            twin_dust(&mut twin, 24, 20);
        }
        for _ in 0..2 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the larvae antlion mirrors the C# draws");
    }

    /// `NPC.cs:86639-86640`/`:86656-86657` — the Raven pair 438/439
    /// (+14 X) and the Parrot pair 373/374, both after the 50x dust-5
    /// burst, scale 1f.
    #[test]
    fn w4_f5d_the_raven_and_parrot_pairs_spawn_the_kind_gores() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 28.0);
        for (kind, want) in [(301, vec![438, 439]), (252, vec![373, 374])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(199);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "kind {kind}");
            if kind == 301 {
                assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
            }
            let mut twin = UnifiedRandom::new(199);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 28);
            }
            for _ in 0..2 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "kind {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86675-86687` — the blood zombie ladders: 489 →
    /// 722/723/723/724/724, 586 → 1106/1107/1107/1108/1108 at the
    /// +0/+20/+20/+34/+34 Y ladder, this.scale carried, the 50x dust
    /// census at the wider 2.5 velocity.
    #[test]
    fn w4_f5d_the_blood_zombie_ladders_carry_the_scale() {
        let w = air();
        let mut n = mob(100.0, 100.0, 30.0, 44.0);
        n.scale = 1.1;
        for (kind, want) in [
            (489, vec![722, 723, 723, 724, 724]),
            (586, vec![1106, 1107, 1107, 1108, 1108]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(211);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "blood zombie {kind}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
            for i in 0..5 {
                assert_eq!(gore.slot(i).scale, 1.1);
            }
            assert!(!gore.slot(5).active, "exactly five gores");
            let mut twin = UnifiedRandom::new(211);
            for _ in 0..50 {
                twin_dust(&mut twin, 30, 44);
            }
            for _ in 0..5 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "blood zombie {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86704-86708` — the Tax Collector ladder 894/895x2/896x2,
    /// same +20/+34 Y / this.scale contract as the blood zombies.
    #[test]
    fn w4_f5d_the_tax_collector_ladder_carries_the_scale() {
        let w = air();
        let mut n = mob(100.0, 100.0, 30.0, 44.0);
        n.scale = 1.05;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(223);
        let _ = hit_effect_d(&n, 534, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![894, 895, 895, 896, 896]);
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        assert_eq!(gore.slot(0).scale, 1.05);
        let mut twin = UnifiedRandom::new(223);
        for _ in 0..50 {
            twin_dust(&mut twin, 30, 44);
        }
        for _ in 0..5 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the tax collector mirrors the C# draws");
    }

    /// `NPC.cs:86724-86728` — the Drippler: 725/726/727 then TWO
    /// `725 + Next(3)` picks; the twin replay pins the argument draws
    /// BETWEEN the third and fourth / fourth and fifth gores' kicks
    /// (Next(3) before each NewGore, then the gore's Y/X kicks).
    #[test]
    fn w4_f5d_the_drippler_draws_the_two_type_picks() {
        let w = air();
        let n = mob(100.0, 100.0, 30.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(227);
        let _ = hit_effect_d(&n, 490, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(&types[..3], &[725, 726, 727]);
        assert!((725..=727).contains(&types[3]), "pick1 in range: {}", types[3]);
        assert!((725..=727).contains(&types[4]), "pick2 in range: {}", types[4]);
        let mut twin = UnifiedRandom::new(227);
        for _ in 0..50 {
            twin_dust(&mut twin, 30, 30);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        // The two picks: Next(3) then the gore's kicks, twice.
        let p1 = 725 + twin.next_max(3);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        let p2 = 725 + twin.next_max(3);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(types[3], p1, "pick1 matches the twin");
        assert_eq!(types[4], p2, "pick2 matches the twin");
        assert_eq!(rng.peek(), twin.peek(), "the drippler mirrors the C# draws");
    }

    /// `NPC.cs:86746-86756` — the Shark quad 89/90/91/92 and the Orca
    /// quad 1403-1406: head at position + three at +14 X, ALL at
    /// velocity*0.8 (the kicks land on the dampened base), scale 1f,
    /// after the 75x dust-5 burst.
    #[test]
    fn w4_f5d_the_shark_quads_dampen_the_velocity() {
        let w = air();
        let mut n = mob(100.0, 100.0, 50.0, 30.0);
        n.vx = 2.0;
        n.vy = 1.0;
        for (kind, want) in [
            (65, vec![89, 90, 91, 92]),
            (692, vec![1403, 1404, 1405, 1406]),
        ] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(229);
            let _ = hit_effect_d(&n, kind, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "shark {kind}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
            assert_eq!(gore.slot(3).position, Vec2::new(114.0, 100.0));
            // The velocity base is dampened BEFORE the kicks: the X
            // component starts at 2.0*0.8 = 1.6 (plus the pool's kick).
            let kick_x = gore.slot(0).velocity.x - 1.6;
            assert!((-2.0..=2.0).contains(&kick_x), "the X kick rides the 0.8 base: {kick_x}");
            let mut twin = UnifiedRandom::new(229);
            for _ in 0..75 {
                twin_dust(&mut twin, 50, 30);
            }
            for _ in 0..4 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "shark {kind} mirrors the C# draws");
        }
    }

    /// `NPC.cs:86760-86770` — the sand shark quads: 542 → 944-947,
    /// 543 Corrupt → 948-951; same +14 X / velocity*0.8 contract.
    #[test]
    fn w4_f5d_the_sand_shark_quads_spawn_the_ladder() {
        let w = air();
        let n = mob(100.0, 100.0, 50.0, 30.0);
        for (kind, want) in [(542, vec![944, 945, 946, 947]), (543, vec![948, 949, 950, 951])] {
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(233);
            let _ = hit_effect_d(&n, kind, -1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, want, "sand shark {kind}");
            assert!(!gore.slot(4).active, "exactly four gores");
            let mut twin = UnifiedRandom::new(233);
            for _ in 0..75 {
                twin_dust(&mut twin, 50, 30);
            }
            for _ in 0..4 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "sand shark {kind} mirrors the C# draws");
        }
    }
}
