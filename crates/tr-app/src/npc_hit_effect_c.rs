//! `NPC.HitEffect` — the per-NPC gore/dust table, wave C (W4-f10): the
//! THIRD batch of `Gore.NewGore*` death sites in source order, starting
//! where the second batch's nominal ~40 ends (`NPC.cs:82206` onward).
//!
//! The pool itself is already ported (`tr_core::gore::GorePool::new_gore`,
//! S1-E-4; the `GoreID.Sets` tables in `tr_core::gore_id`); this module is
//! the per-NPC spawn table the pool feeds. W4-f10 (code-only mode): the
//! module + witnesses land first; the dispatch claim (the `StrikeNPC_Inner`
//! `:81482`/`:81488` pair and the AI-arm `HitEffect()` calls) is the wiring
//! wave, the `npc_ai_style39.rs` / wave-A (`npc_hit_effect_a.rs`) precedent.
//!
//! ## Claim boundary (the pushed-claims probe)
//!
//! * wave A (`W4-d55`, `ec78d152`): the FIRST 30 sites, `NPC.cs:81660-81941`
//!   (the party-hat rows, the 686/594 bursts, daybreak, 668 Deerclops, the
//!   636 Empress first gore). PUSHED.
//! * wave B (`W4-e5-gore-sites-2`): the SECOND ~40 sites, `NPC.cs:81942-82202`
//!   (the 636 tail, 657/658-660/667, the DD2 goblin/bomber/javelinst rows and
//!   the 551 Betsy NON-flipped branch). Nominal range; no pushed commits at
//!   probe time.
//! * wave C (THIS module, `W4-f10-gore-sites-3`): the remainder from
//!   `NPC.cs:82206` — the 551 Betsy FLIPPED branch, the DD2 kobold/drakin/
//!   wither-beast/wyvern/skeleton/ogre/dark-mage rows, the 578 lightning bug,
//!   and the blood-moon sea monsters (620/619/618).
//! * `W4-g16` (projectile) / `W4-g17` (world) own the non-NPC files.
//!
//! ## Scope — this batch (source order, death halves only)
//!
//! | sites | C# lines | arm |
//! | --- | --- | --- |
//! | 71-80 | `:82206-82215` | the 551 Betsy FLIPPED branch (`spriteDirection == -1`) |
//! | 81-85 | `:82237-82241` | the 572/573 DD2 Kobold Walker death (5 gores) |
//! | 86-90 | `:82262-82266` | the 574/575 DD2 Kobold Flyer death (5 gores) |
//! | 91-95 | `:82285-82289` | the 570/571 DD2 Drakin death (5 gores, per-tier base) |
//! | 96-100 | `:82307-82311` | the 568/569 DD2 Wither Beast death (5 gores) |
//! | 101-107 | `:82338-82344` | the 558/559/560 DD2 Wyvern death (7 gores, per-tier base) |
//! | 108-110 | `:82362-82369` | the 566/567 DD2 Skeleton death: head + the first two coin-gates |
//! | 111-114 | `:82373-82385` | the 566/567 DD2 Skeleton death: the remaining four coin-gates |
//! | 115-120 | `:82406-82413` | the 576/577 DD2 Ogre FLIPPED death (6 gores, Next(5) type gate + π rotations) |
//! | 121-126 | `:82417-82422` | the 576/577 DD2 Ogre NON-flipped death (6 gores, Next(5) type gate) |
//! | 127-132 | `:82442-82447` | the 564/565 DD2 Dark Mage death: the first six gores (base 1009 ladder) |
//!
//! Later commits extend the table through `:82559`. The hit (`life > 0`)
//! halves are dmg-scaled dust loops with NO gore — they ride the wiring
//! wave (wave A's "the death halves here use the flat counts" convention).
//!
//! ## RNG contract (the strike's stream)
//!
//! `HitEffect` runs inside `StrikeNPC_Inner` (`NPC.cs:81488`), under the
//! strike caller's `Main.rand` swap (the projectile/melee update's stage),
//! NOT the NPC pass's — the `npc_ai_style37.rs::probe_launch_on_hit` note.
//! The caller hands that stream in; every draw below hits it in C# order:
//!
//! * 551 Betsy death: the 20-dust type-5 loop first (each `NewDust`'s 6
//!   default-block draws + the type-5 tweak, `Dust.cs:153-252`, pool-side),
//!   then the 10 gores draw ONLY the `NewGore` kicks (2 each, `Gore.cs:1327-1328`,
//!   pool-side) — the types/anchors are data, no arm-level draw.
//!
//! The dust loops run through `DustPool::new_dust` (S1-E-1) on the SAME
//! stream, so a cull refusal or a pressure-band gate consumes exactly the
//! C#'s draws (the pool owns them). The post-spawn writes land on the
//! returned slot, the sentinel included — the C# writes `Main.gore[600]`
//! too (the `npc_ai_style56.rs` precedent).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `!this.active` gate (`:81656`): the caller's — the fn runs on an
//!   active NPC (the strike/death path's own check).
//! * The `this.type` axis rides an explicit `npc_type` parameter (the
//!   wave-A `HitEffectInputs` / `Style39Type` precedent — the DD2/blood-moon
//!   kinds have no `NpcKind` rows yet).
//! * The 551 Betsy NON-flipped branch (`:8193-82202`, `spriteDirection != -1`)
//!   is wave B's; this module ports ONLY the flipped (`== -1`) branch and
//!   gates on `sprite_dir == -1`. The shared 20-dust death loop is ported
//!   here; the merge keeps ONE copy (keep-one).
//! * `DD2Event.CheckProgress` (`:82176`) is a hit-half (`life > 0`) event
//!   seam — the wiring wave's, no draw here.
//! * `netMode` guards: single-player port (the `netMode != 1` arms run,
//!   the `== 2` arms drop), the pool's own convention.
//!
//! See docs/ANALYSIS.md "The NPC HitEffect gore table (W4-f10, wave C)".
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

/// The death-half dust loop shared by the DD2 rows (`NPC.cs` per-arm
/// `for (… < 20f) Dust.NewDust(position, width, height, Type, hitDirection, -1f)`).
/// `count` is the flat death count; the hit (`life > 0`) half's dmg-scaled
/// bound is the wiring wave's. The pool owns every draw.
fn death_dust_loop(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    dust_type: i32,
    hit_direction: f32,
    count: i32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, dust_type, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
    }
}

/// The 551 Betsy FLIPPED death branch (`NPC.cs:82206-82215`, sites 71-80):
/// the 10 body gores when `spriteDirection == -1`. The types/anchors are
/// data; each spawn draws only the `NewGore` kicks (pool-side). The shared
/// 20-dust type-5 death loop (`:82187-82190`) runs FIRST (the C# order).
fn betsy_flipped_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:82187-82190` — the 20-dust type-5 death burst (shared with the
    // non-flipped branch; keep-one at merge).
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82206-82215` — the flipped anchors, velocity = base.velocity,
    // scale = this.scale.
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos + Vec2::new(176.0, 54.0), vel, 1079, s); // :82206
    gore.new_gore(rng, world, pos + Vec2::new(158.0, 90.0), vel, 1080, s); // :82207
    gore.new_gore(rng, world, pos + Vec2::new(128.0, 94.0), vel, 1080, s); // :82208
    gore.new_gore(rng, world, pos + Vec2::new(0.0, 0.0), vel, 1081, s); // :82209
    gore.new_gore(rng, world, pos + Vec2::new(60.0, 12.0), vel, 1082, s); // :82210
    gore.new_gore(rng, world, pos + Vec2::new(88.0, 18.0), vel, 1083, s); // :82211
    gore.new_gore(rng, world, pos + Vec2::new(86.0, 26.0), vel, 1084, s); // :82212
    gore.new_gore(rng, world, pos + Vec2::new(40.0, 16.0), vel, 1085, s); // :82213
    gore.new_gore(rng, world, pos + Vec2::new(20.0, 10.0), vel, 1085, s); // :82214
    gore.new_gore(rng, world, pos + Vec2::new(142.0, 36.0), vel, 1086, s); // :82215
}

/// The 572/573 DD2 Kobold Walker death (`NPC.cs:82232-82241`, sites 81-85):
/// the 20-dust type-5 burst, then 5 gores 993/996/995/994/994 at
/// `Center - (10,10)` (+0/+20/+20/+34/+34 Y). The two tracked-sound stops
/// (`:82230-82231`, `localAI[0]`/`localAI[1]`) draw NO random numbers — the
/// tracked-sound seam is the wiring wave's (the S1-D-4b convention).
fn kobold_walker_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82236` — vector6 = Center - (10,10).
    let v6 = center(npc) - Vec2::new(10.0, 10.0);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, v6, vel, 993, s); // :82237
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 20.0), vel, 996, s); // :82238
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 20.0), vel, 995, s); // :82239
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 34.0), vel, 994, s); // :82240
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 34.0), vel, 994, s); // :82241
}

/// The 574/575 DD2 Kobold Flyer death (`NPC.cs:82256-82266`, sites 86-90):
/// the 20-dust type-5 burst, then 5 gores 1050/1051/1052/1053/1054 at
/// `Center - (30,10)` (+10/+30/+30/+0/+0 Y). The tracked-sound stop
/// (`:82255`, `localAI[2]`) draws no random numbers (the wiring seam).
fn kobold_flyer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82260-82261` — num58 = 1050; vector7 = Center - (30,10).
    let v7 = center(npc) - Vec2::new(30.0, 10.0);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y + 10.0), vel, 1050, s); // :82262
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y + 30.0), vel, 1051, s); // :82263
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y + 30.0), vel, 1052, s); // :82264
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y), vel, 1053, s); // :82265
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y), vel, 1054, s); // :82266
}

/// The 570/571 DD2 Drakin death (`NPC.cs:82280-82289`, sites 91-95): the
/// 20-dust type-5 burst, then 5 gores at the per-tier base
/// (`num61 = type == 570 ? 1075 : 979`, `:82284`) with the type order
/// `num61 / +2 / +2 / +3 / +1` at the +0/+20/+20/+20/+34 Y ladder.
fn drakin_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82284` — the per-tier base.
    let num61 = if npc_type == 570 { 1075 } else { 979 };
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num61, s); // :82285
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num61 + 2, s); // :82286
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num61 + 2, s); // :82287
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num61 + 3, s); // :82288
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num61 + 1, s); // :82289
}

/// The 568/569 DD2 Wither Beast death (`NPC.cs:82302-82311`, sites 96-100):
/// the 20-dust type-5 burst, then the 5 fixed gores 1020/1021/1023/1022/1022
/// at the +0/+20/+20/+34/+34 Y ladder (the type order is NOT a base+offset
/// ladder — 1023 precedes the 1022 pair, `:82309-82311`).
fn wither_beast_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 1020, s); // :82307
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 1021, s); // :82308
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 1023, s); // :82309
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 1022, s); // :82310
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 1022, s); // :82311
}

/// The 558/559/560 DD2 Wyvern death (`NPC.cs:82325-82344`, sites 101-107):
/// the 20-dust type-5 burst, then 7 gores at the per-tier base
/// (`num66`: 558→974, 559→1040, 560→1045, `:82329-82337`) with the type
/// order `+2 / base / base / +4 / +1 / +3 / +3` at the +0/+10/+10/+10/+24/+24/+24
/// Y ladder.
fn wyvern_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82329-82337` — the per-tier base (two independent ifs, not else-if).
    let mut num66 = 974;
    if npc_type == 559 {
        num66 = 1040;
    }
    if npc_type == 560 {
        num66 = 1045;
    }
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num66 + 2, s); // :82338
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, num66, s); // :82339
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, num66, s); // :82340
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, num66 + 4, s); // :82341
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 24.0), vel, num66 + 1, s); // :82342
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 24.0), vel, num66 + 3, s); // :82343
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 24.0), vel, num66 + 3, s); // :82344
}

/// The 566/567 DD2 Skeleton death (`NPC.cs:82358-82386`, sites 108-114):
/// the 20-dust type-236 burst, the 970 head, then SIX independent
/// `Main.rand.Next(2) == 0` coin-gates (972/972/973/973/971/971). Each gate
/// draws its `Next(2)` BEFORE the conditional spawn — a refused gate still
/// consumes the draw. Sites 108-114 (the head + all six gates).
fn skeleton_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:82358-82361` — the dust is type 236 (NOT 5).
    death_dust_loop(npc, dust, dust_ctx, rng, world, 236, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 970, s); // :82362
    // `:82363-82366` — gate 1: Next(2) then the conditional 972.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 972, s); // :82365
    }
    // `:82367-82370` — gate 2: Next(2) then the conditional 972.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 972, s); // :82369
    }
    // `:82371-82374` — gate 3: Next(2) then the conditional 973.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 973, s); // :82373
    }
    // `:82375-82378` — gate 4: Next(2) then the conditional 973.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 973, s); // :82377
    }
    // `:82379-82382` — gate 5: Next(2) then the conditional 971 at +34.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 971, s); // :82381
    }
    // `:82383-82386` — gate 6: Next(2) then the conditional 971 at +34.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 971, s); // :82385
    }
}

/// The 576/577 DD2 Ogre FLIPPED death (`NPC.cs:82404-82413`, sites 115-120):
/// the 20-dust type-273 burst, then the 6 flipped gores. The head type is
/// `1005 + (Next(5) == 0 ? 1 : 0)` (`:82406`, the draw rides the strike
/// stream BEFORE the spawn); gores 1001 (`:82408`) and 1002 (`:82410`) get
/// `rotation = π` post-writes on the returned slot (sentinel included).
/// The non-flipped branch (`:82417-82422`) is the NEXT batch.
fn ogre_flipped_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:82400-82403` — the dust is type 273.
    death_dust_loop(npc, dust, dust_ctx, rng, world, 273, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:82406` — the head's Next(5) type gate draws first.
    let head = 1005 + if rng.next_max(5) == 0 { 1 } else { 0 };
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, pos + Vec2::new(18.0, 28.0), vel, 1000, s); // :82407
    // `:82408-82409` — the 1001 gore, rotation = π on the slot.
    let num71 = gore.new_gore(rng, world, pos + Vec2::new(-40.0, 70.0), vel, 1001, s);
    gore.slot_mut(num71).rotation = std::f32::consts::PI;
    // `:82410-82411` — the 1002 gore, rotation = π on the slot.
    let num71 = gore.new_gore(rng, world, pos + Vec2::new(4.0, 28.0), vel, 1002, s);
    gore.slot_mut(num71).rotation = std::f32::consts::PI;
    gore.new_gore(rng, world, pos + Vec2::new(36.0, 90.0), vel, 1003, s); // :82412
    gore.new_gore(rng, world, pos + Vec2::new(32.0, 90.0), vel, 1004, s); // :82413
}

/// The 576/577 DD2 Ogre NON-flipped death (`NPC.cs:82417-82422`, sites
/// 121-126): the 20-dust type-273 burst, then the 6 non-flipped gores. The
/// head type is `1005 + (Next(5) == 0 ? 1 : 0)` (`:82417`); unlike the
/// flipped branch, the 1001/1002 gores get NO rotation write here.
fn ogre_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 273, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:82417` — the head's Next(5) type gate draws first.
    let head = 1005 + if rng.next_max(5) == 0 { 1 } else { 0 };
    gore.new_gore(rng, world, pos + Vec2::new(48.0, 0.0), vel, head, s);
    gore.new_gore(rng, world, pos + Vec2::new(22.0, 28.0), vel, 1000, s); // :82418
    gore.new_gore(rng, world, pos + Vec2::new(48.0, 70.0), vel, 1001, s); // :82419
    gore.new_gore(rng, world, pos + Vec2::new(4.0, 28.0), vel, 1002, s); // :82420
    gore.new_gore(rng, world, pos + Vec2::new(36.0, 90.0), vel, 1003, s); // :82421
    gore.new_gore(rng, world, pos + Vec2::new(32.0, 90.0), vel, 1004, s); // :82422
}

/// The 564/565 DD2 Dark Mage death (`NPC.cs:82437-82452`, sites 127-137):
/// the 20-dust type-5 burst, then 11 gores at base `num74 = 1009`
/// (`:82441`). This commit ports the first six gores (sites 127-132,
/// `:82442-82447`); the +6/+10/+7/+8/+9 tail lands with the next batch.
fn dark_mage_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82441` — num74 = 1009.
    let num74 = 1009;
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num74, s); // :82442
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 15.0), vel, num74 + 1, s); // :82443
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 15.0), vel, num74 + 2, s); // :82444
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 15.0), vel, num74 + 3, s); // :82445
    gore.new_gore(rng, world, Vec2::new(pos.x - 20.0, pos.y + 35.0), vel, num74 + 4, s); // :82446
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 35.0), vel, num74 + 5, s); // :82447
}

/// `NPC.HitEffect(hitDirection, dmg)` (`NPC.cs:81654+`) — wave C: the third
/// batch of gore sites, `:82206` onward. `npc.hp` is `this.life` AT THE CALL
/// (the strike writes it before calling, `:81488`); only the death
/// (`life <= 0`) halves carry gore here. `hit_direction` is the C#
/// `hitDirection` (a velocity offset for the dust, no extra draw).
///
/// The dispatch grows one arm per commit; the wiring wave folds it into the
/// wave-A/B `StrikeNPC_Inner` claim.
pub fn hit_effect_c(
    npc: &Npc,
    npc_type: i32,
    hit_direction: f32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:82172-82218` — the 551 Betsy arm; this module owns the FLIPPED
    // (`spriteDirection == -1`) death branch only (the non-flipped branch
    // is wave B's).
    if npc_type == 551 && npc.hp <= 0 && npc.sprite_dir == -1 {
        betsy_flipped_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82219-82243` — the 572/573 DD2 Kobold Walker arm (sites 81-85).
    if (npc_type == 572 || npc_type == 573) && npc.hp <= 0 {
        kobold_walker_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82244-82268` — the 574/575 DD2 Kobold Flyer arm (sites 86-90).
    if (npc_type == 574 || npc_type == 575) && npc.hp <= 0 {
        kobold_flyer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82269-82291` — the 570/571 DD2 Drakin arm (sites 91-95).
    if (npc_type == 570 || npc_type == 571) && npc.hp <= 0 {
        drakin_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82292-82313` — the 568/569 DD2 Wither Beast arm (sites 96-100).
    if (npc_type == 568 || npc_type == 569) && npc.hp <= 0 {
        wither_beast_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82314-82346` — the 558/559/560 DD2 Wyvern arm (sites 101-107).
    if (npc_type == 558 || npc_type == 559 || npc_type == 560) && npc.hp <= 0 {
        wyvern_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82347-82388` — the 566/567 DD2 Skeleton arm (sites 108-114, all
    // six coin-gates).
    if (npc_type == 566 || npc_type == 567) && npc.hp <= 0 {
        skeleton_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82389-82425` — the 576/577 DD2 Ogre arm: the FLIPPED branch
    // (`spriteDirection == -1`, sites 115-120) vs the non-flipped branch
    // (sites 121-126).
    if (npc_type == 576 || npc_type == 577) && npc.hp <= 0 {
        if npc.sprite_dir == -1 {
            ogre_flipped_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        } else {
            ogre_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        }
    }
    // `:82426-82454` — the 564/565 DD2 Dark Mage arm (sites 127-132 here;
    // the +6/+10/+7/+8/+9 tail lands with the next batch).
    if (npc_type == 564 || npc_type == 565) && npc.hp <= 0 {
        dark_mage_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
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

    /// A DD2-boss-shaped stand-in: the unported kinds ride a placeholder
    /// kind with the C# box through `size_override` (the wave-A `boss`
    /// precedent). Betsy's box is 232x156 (`NPC.cs` SetDefaults case 551).
    fn boss(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 100, 0.0);
        n.size_override = Some((w, h));
        n.hp = 0;
        n.scale = 1.0;
        n.sprite_dir = -1;
        n
    }

    /// `NPC.cs:82206-82215` — the flipped Betsy death spawns the 10 body
    /// gores with the flipped anchors, in source order, at velocity =
    /// base.velocity (+ the NewGore kicks) and scale = this.scale.
    #[test]
    fn w4_f10_the_betsy_flipped_death_spawns_the_ten_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 232.0, 156.0);
        n.vx = 2.0;
        n.vy = -1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        betsy_flipped_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..10].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1079, 1080, 1080, 1081, 1082, 1083, 1084, 1085, 1085, 1086]
        );
        // The flipped anchors (position + offset), the first gore at (276,154).
        assert_eq!(gore.slot(0).position, Vec2::new(100.0 + 176.0, 100.0 + 54.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(9).position, Vec2::new(100.0 + 142.0, 100.0 + 36.0));
        // The 20-dust type-5 death burst ran first (the pool holds the dust).
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 15, "the death burst spawned: {dust_active}");
    }

    /// `NPC.cs:82206-82215` — the flipped branch draws ONLY the NewGore
    /// kicks per gore (the types are data); the twin replay mirrors the
    /// 20-dust census then the 10 gores' two kicks each on the strike stream.
    #[test]
    fn w4_f10_the_betsy_flipped_death_draws_only_the_kicks() {
        let w = air();
        let n = boss(100.0, 100.0, 232.0, 156.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        betsy_flipped_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The twin: 20 type-5 dusts (6 default-block draws each; type 5 hits
        // no tweak, Dust.cs:214-252) then 10 gores x 2 kicks.
        let mut twin = UnifiedRandom::new(3);
        let (bw, bh) = (232, 156);
        for _ in 0..20 {
            let _px = twin.next_max(bw - 4);
            let _py = twin.next_max(bh - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..10 {
            let _ky = twin.next_range(10, 31); // NewGore Y kick
            let _kx = twin.next_range(-20, 21); // NewGore X kick
        }
        assert_eq!(rng.peek(), twin.peek(), "the flipped death mirrors the C# draws");
    }

    /// `NPC.cs:82237-82241` — the Kobold Walker death: 5 gores
    /// 993/996/995/994/994 at Center-(10,10) with the +0/+20/+20/+34/+34
    /// Y ladder, after the 20-dust burst.
    #[test]
    fn w4_f10_the_kobold_walker_death_spawns_the_five_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 46.0);
        n.vx = 1.5;
        n.vy = -0.5;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        kobold_walker_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![993, 996, 995, 994, 994]);
        // Center = (120,123); vector6 = (110,113).
        assert_eq!(gore.slot(0).position, Vec2::new(110.0, 113.0));
        assert_eq!(gore.slot(1).position, Vec2::new(110.0, 133.0));
        assert_eq!(gore.slot(3).position, Vec2::new(110.0, 147.0));
        assert_eq!(gore.slot(4).position, Vec2::new(110.0, 147.0));
    }

    /// `NPC.cs:82262-82266` — the Kobold Flyer death: 5 gores
    /// 1050-1054 at Center-(30,10) with the +10/+30/+30/+0/+0 Y ladder.
    #[test]
    fn w4_f10_the_kobold_flyer_death_spawns_the_five_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 54.0, 44.0);
        n.vx = -2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        kobold_flyer_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1050, 1051, 1052, 1053, 1054]);
        // Center = (127,122); vector7 = (97,112).
        assert_eq!(gore.slot(0).position, Vec2::new(97.0, 122.0));
        assert_eq!(gore.slot(1).position, Vec2::new(97.0, 142.0));
        assert_eq!(gore.slot(3).position, Vec2::new(97.0, 112.0));
        assert_eq!(gore.slot(4).position, Vec2::new(97.0, 112.0));
    }

    /// `NPC.cs:82219-82268` — both kobold arms draw the 20-dust census then
    /// only the NewGore kicks (5 gores x 2); the twin replay pins the stream.
    #[test]
    fn w4_f10_the_kobold_deaths_mirror_the_csharp_draws() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 46.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(21);
        kobold_walker_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let mut twin = UnifiedRandom::new(21);
        for _ in 0..20 {
            let _px = twin.next_max(36);
            let _py = twin.next_max(42);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..5 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the walker death mirrors the C# draws");
    }

    /// `NPC.cs:82284-82289` — the Drakin death's per-tier base: 570 rides
    /// 1075, 571 rides 979; the type order is base/+2/+2/+3/+1.
    #[test]
    fn w4_f10_the_drakin_death_uses_the_per_tier_base() {
        let w = air();
        for (ty, base) in [(570, 1075), (571, 979)] {
            let n = boss(100.0, 100.0, 60.0, 48.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(17);
            drakin_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base, base + 2, base + 2, base + 3, base + 1],
                "tier {ty}"
            );
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(4).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:82307-82311` — the Wither Beast death: the fixed type order
    /// 1020/1021/1023/1022/1022 (1023 precedes the 1022 pair).
    #[test]
    fn w4_f10_the_wither_beast_death_keeps_the_fixed_type_order() {
        let w = air();
        let n = boss(100.0, 100.0, 66.0, 52.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        wither_beast_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1020, 1021, 1023, 1022, 1022]);
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        assert_eq!(gore.slot(4).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:82329-82344` — the Wyvern death's per-tier base and the
    /// +2/base/base/+4/+1/+3/+3 type order at the +0/+10/+24 Y ladder.
    #[test]
    fn w4_f10_the_wyvern_death_uses_the_per_tier_base() {
        let w = air();
        for (ty, base) in [(558, 974), (559, 1040), (560, 1045)] {
            let n = boss(100.0, 100.0, 60.0, 60.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(23);
            wyvern_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base + 2, base, base, base + 4, base + 1, base + 3, base + 3],
                "tier {ty}"
            );
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 110.0));
            assert_eq!(gore.slot(4).position, Vec2::new(100.0, 124.0));
        }
    }

    /// `NPC.cs:82362-82386` — the Skeleton death: the 970 head always
    /// spawns; the six coin-gates draw Next(2) BEFORE the conditional spawn,
    /// so the stream advances even when a gate refuses. The twin replay pins
    /// the head kicks + all six gate draws regardless of the gate outcomes.
    #[test]
    fn w4_f10_the_skeleton_death_gates_draw_before_the_spawn() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        skeleton_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The head always lands (site 108).
        assert_eq!(gore.slot(0).gore_type, 970);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // The twin: 20 type-236 dusts, the head's 2 kicks, then the six
        // Next(2) gates; a taken gate adds its 2 kicks.
        let mut twin = UnifiedRandom::new(29);
        for _ in 0..20 {
            let _px = twin.next_max(40);
            let _py = twin.next_max(52);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        for _ in 0..6 {
            if twin.next_max(2) == 0 {
                let _gy = twin.next_range(10, 31);
                let _gx = twin.next_range(-20, 21);
            }
        }
        assert_eq!(rng.peek(), twin.peek(), "the skeleton gates mirror the C# draws");
        // Every spawned coin-gore is one of the six gate types.
        for g in gore.slots()[1..7].iter().filter(|g| g.active) {
            assert!(matches!(g.gore_type, 971 | 972 | 973), "gate type {}", g.gore_type);
        }
    }

    /// `NPC.cs:82406-82413` — the Ogre FLIPPED death: the head's Next(5)
    /// type gate draws first, the 1001/1002 gores get rotation = π, and the
    /// twin replay mirrors the census + the head gate + the six gores' kicks.
    #[test]
    fn w4_f10_the_ogre_flipped_death_rotates_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 96.0, 104.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        ogre_flipped_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The head is 1005 or 1006 (the Next(5) gate).
        assert!(matches!(gore.slot(0).gore_type, 1005 | 1006));
        assert_eq!(gore.slot(1).gore_type, 1000);
        assert_eq!(gore.slot(2).gore_type, 1001);
        assert_eq!(gore.slot(2).rotation, std::f32::consts::PI);
        assert_eq!(gore.slot(3).gore_type, 1002);
        assert_eq!(gore.slot(3).rotation, std::f32::consts::PI);
        assert_eq!(gore.slot(4).gore_type, 1003);
        assert_eq!(gore.slot(5).gore_type, 1004);
        // The twin: 20 type-273 dusts, the Next(5) head gate, then 6 gores'
        // kicks (the rotation writes draw nothing).
        let mut twin = UnifiedRandom::new(31);
        for _ in 0..20 {
            let _px = twin.next_max(92);
            let _py = twin.next_max(100);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let _head_gate = twin.next_max(5);
        for _ in 0..6 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the ogre flipped death mirrors the C# draws");
    }

    /// `NPC.cs:82417-82422` — the Ogre NON-flipped death: the Next(5) head
    /// gate, the 6 non-flipped anchors, and NO rotation write on 1001/1002.
    #[test]
    fn w4_f10_the_ogre_non_flipped_death_has_no_rotation() {
        let w = air();
        let mut n = boss(100.0, 100.0, 96.0, 104.0);
        n.sprite_dir = 1;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(37);
        ogre_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert!(matches!(gore.slot(0).gore_type, 1005 | 1006));
        assert_eq!(gore.slot(0).position, Vec2::new(148.0, 100.0));
        assert_eq!(gore.slot(2).gore_type, 1001);
        assert_eq!(gore.slot(2).rotation, 0.0); // no π write in this branch
        assert_eq!(gore.slot(3).gore_type, 1002);
        assert_eq!(gore.slot(3).rotation, 0.0);
        assert_eq!(gore.slot(5).position, Vec2::new(132.0, 190.0));
    }

    /// `NPC.cs:82442-82447` — the Dark Mage death's first six gores: base
    /// 1009, the +1/+2/+3 trio at +15 Y, the +4 gore shifted -20 in X at
    /// +35 Y, and the +5 gore at +35 Y.
    #[test]
    fn w4_f10_the_dark_mage_death_spawns_the_first_six_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 64.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        dark_mage_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1009, 1010, 1011, 1012, 1013, 1014]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 115.0));
        // The +4 gore (index 4) is shifted -20 in X at +35 Y.
        assert_eq!(gore.slot(4).position, Vec2::new(80.0, 135.0));
        assert_eq!(gore.slot(5).position, Vec2::new(100.0, 135.0));
    }

    /// The wave-C dispatch gates the flipped Betsy branch on
    /// `spriteDirection == -1` and `life <= 0`; a non-flipped or living
    /// Betsy spawns nothing here (the non-flipped branch is wave B's).
    #[test]
    fn w4_f10_the_dispatch_gates_the_flipped_betsy_branch() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        // Flipped + dead -> the 10 gores.
        let n = boss(100.0, 100.0, 232.0, 156.0);
        hit_effect_c(&n, 551, 1.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert_eq!(gore.slots().iter().filter(|g| g.active).count(), 10);
        // Non-flipped -> nothing here (wave B's branch).
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut n2 = boss(100.0, 100.0, 232.0, 156.0);
        n2.sprite_dir = 1;
        hit_effect_c(&n2, 551, 1.0, &mut gore2, &mut dust2, &ctx(), &mut rng, &w);
        assert_eq!(gore2.slots().iter().filter(|g| g.active).count(), 0);
        // Living -> nothing (the hit half is dust-only, the wiring wave's).
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut n3 = boss(100.0, 100.0, 232.0, 156.0);
        n3.hp = 50;
        hit_effect_c(&n3, 551, 1.0, &mut gore3, &mut dust3, &ctx(), &mut rng, &w);
        assert_eq!(gore3.slots().iter().filter(|g| g.active).count(), 0);
    }
}
