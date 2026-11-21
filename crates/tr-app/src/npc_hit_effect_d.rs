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
/// position + wing pair at Center ±(4,0) (`:86002-86019`). The remaining
/// sub-branches land in later batches; an unported kind in the arm spawns
/// the dust burst only (the census note — the gores follow in source
/// order).
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
    // `:85942-86091` — the 29-type critter arm; the 46/303/337/540, 67,
    // 55/230, 615, 74, 297, 298, 671/672/673 sub-branches are live (sites
    // 42-65), the rest land later.
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
}
