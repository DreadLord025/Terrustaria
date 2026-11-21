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
//! | 133-137 | `:82448-82452` | the 564/565 DD2 Dark Mage death: the +6/+10/+7/+8/+9 tail |
//! | 138-141 | `:82470-82473` | the 578 DD2 Lightning Bug death (4 gores 999/998/998/997) |
//! | 142-145 | `:82496-82499` | the 620 Goblin Shark death (4 gores, base 1183 ladder, 150-dust burst) |
//! | 146-149 | `:82522-82525` | the 619 Blood Squid death (4 gores, base 1179 ladder, 150-dust fadeIn burst) |
//! | 150-155 | `:82548-82553` | the 618 Blood Nautilus death: the first six gores (base 1172, 200-dust burst) |
//! | 156-161 | `:82554-82559` | the 618 Blood Nautilus death: the +4/+4/+5/+5/+6/+6 tail |
//! | 162-165 | `:82600-82603` | the 541 Sand Elemental death (4 gores 960/961/963/962, tinted 60-dust burst) |
//! | 166-168 | `:82650-82652` | the 546 Tumbleweed death (3 gores 964/965/966, tinted 40-dust + 20 type-54 burst) |
//! | 169-174 | `:82676` | the 437 Cultist Tablet `dmg == 9999` burst (6 gores 728-733, RotatedByRandom velocity) |
//! | 175-177 | `:82743` | the 513-515 Tomb Crawler death (1 gore each, type-513+822, alpha copy) |
//! | 178-180 | `:82762` | the 510-512 Dune Splicer death (1 gore each, type-510+819, alpha copy) |
//! | 181-183 | `:82781` | the 621-623 Blood Eel death (1 gore each, type-621+1187, alpha copy, type-5 dust) |
//! | 184-188 | `:82816-82820` | the 518/416 Solar Spearman/Drakomire Rider death (5 gores, gated type-6 dust) |
//! | 189-194 | `:82854-82859` | the 419 Solar Solenian death (6 gores, gated type-6 dust) |
//! | 195-197 | `:82881` | the 412-414 Solar Crawltipede death (1 gore each, type-412+828) |
//! | 198-201 | `:82912-82915` | the 415 Solar Flare death (4 gores at Center, vel x0.8/x0.9) |
//! | 202-207 | `:82949-82954` | the 417 Solar Sroller death: the first six gores (837/838x3/839x2) |
//! | 208-209 | `:82955-82956` | the 417 Solar Sroller death: the 840/840 tail |
//! | 210-214 | `:82987-82991` | the 418 Solar Corite death (5 gores at Center, vel x0.8/x0.9) |
//! | 215-217 | `:83072-83074` | the 426 Nebula Floater death (3 gores, SelectRandom dust + NPC-428 spawn seam) |
//! | 218-221 | `:83119-83122` | the 427 Nebula Soldier death (4 gores at Center, vel x0.8/x0.9) |
//! | 222-226 | `:83156-83160` | the 429 Nebula Predictor death (5 gores, projectile-578 spawn seam) |
//! | 227-233 | `:83238-83244` | the 425 Nebula Beast death (7 gores, gated type-229 dust) |
//! | 234-235 | `:83270-83271` | the 428 Nebula Brain death (2 gores, vel x0.8, Next(2) type-229/240 dust) |
//! | 236-240 | `:83305-83309` | the 424 Vortex Soldier death (5 gores, gated type-242 dust) |
//! | 241-243 | `:83340-83342` | the 423 Vortex Hornet death (3 gores at Center, vel x0.8/x0.9) |
//! | 244-246 | `:83373-83375` | the 421 Vortex Larva death (3 gores at Center, vel x0.8/x0.9) |
//! | 247-249 | `:83406-83408` | the 420 Vortex Rider death (3 gores at Center, vel x0.8/x0.9) |
//! | 250-252 | `:83503-83505` | the 409 Stardust Spider death (3 gores at Center, vel x0.8/x0.9) |
//! | 253-258 | `:83557-83562` | the 407 Stardust Cell death (6 gores, Next(3)!=0 type-17 dust gate) |
//! | 259-263 | `:83657-83661` | the 411 Stardust Dragon body death (5 gores, Next(2)-gated type-229 dust) |
//! | 264-274 | `:83670-83676` | the 491 Flying Dutchman death: the head block (2 fixed + 3x3 loop gores) |
//! | 275-284 | `:83680-83684` | the 491 Flying Dutchman death: the RandomVector2 fan, iterations -1/0 |
//! | 285-289 | `:83680-83684` | the 491 Flying Dutchman death: the RandomVector2 fan, iteration 1 |
//! | 290-293 | `:83688-83689` | the 491 Flying Dutchman death: the 855 loop (2x2 gores) |
//! | 294-302 | `:83693-83695` | the 491 Flying Dutchman death: the 854/853/856 loop (3x3 gores) |
//! | 303-312 | `:83699` | the 491 Flying Dutchman death: the 861/862 scatter burst, gores 0-9 |
//! | 313-322 | `:83699` | the 491 Flying Dutchman death: the 861/862 scatter burst, gores 10-19 |
//! | 323-332 | `:83699` | the 491 Flying Dutchman death: the 861/862 scatter burst, gores 20-29 |
//! | 333-342 | `:83734` | the 492 Flying Dutchman (cannon) death: 7-dust type-31 burst + 10-gore Next(61,64) scatter |
//! | 343-348 | `:83789-83794` | the 524-527 Desert Ghoul (normal/corrupt/crimson/hallow) death (6 gores, per-tier base) |
//! | 349-352 | `:83818-83821` | the 528-529 Desert Lamia (light/dark) death (4 gores, per-tier head) |
//! | 353-357 | `:83839-83843` | the 530/531 Desert Scorpion death (5 gores 883-887, alpha-100 type-250 dust) |
//! | 358-361 | `:83861-83864` | the 532 Desert Beast death (4 gores 888-891, alpha-100 type-5 dust) |
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

/// `Entity.Right` (`Entity.cs:104-114`) — `(pos.X + width, pos.Y + height/2f)`.
fn right_x(npc: &Npc) -> f32 {
    let (w, _) = npc.size();
    npc.x + w
}

/// `Entity.Top` (`Entity.cs:86-96`) — `(pos.X + width/2f, pos.Y)`.
fn top(npc: &Npc) -> Vec2 {
    let (w, _) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y)
}

/// `Utils.SelectRandom<T>` (`Utils.cs:2459-2462`) — `choices[Next(len)]`.
fn select_random(rng: &mut UnifiedRandom, choices: &[i32]) -> i32 {
    choices[rng.next_max(choices.len() as i32) as usize]
}

/// `Utils.RandomVector2(Main.rand, min, max)` (`Utils.cs:2431-2434`): TWO
/// `NextDouble` draws, X then Y, each `(max-min)*NextDouble + min`.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// The side-effects the wiring caller owes (the roster seams this module
/// cannot reach — the wave-A `HitEffectOutcome` precedent).
#[derive(Default)]
pub struct HitEffectOutcomeC {
    /// The 426 Nebula Floater's brain-spawn seam (`NPC.cs:83075-83083`):
    /// when `CountNPCS(428) + CountNPCS(427) + CountNPCS(426)*3 < 20`, spawn
    /// 3 NPC-428s at Center with velocity
    /// `-UnitY.RotatedByRandom(2π) * Next(3,6) - UnitY*2`. `Some((cx,cy))` =
    /// the caller resolves the count gate + the spawns; the per-spawn
    /// RotatedByRandom/Next draws ride the wiring wave's roster pass.
    pub nebula_brain_spawn: Option<(f32, f32)>,
    /// The 429 Nebula Predictor's projectile-spawn seam (`NPC.cs:83161-83204`):
    /// a tile-scan loop (up to 100 tries, two `Next` draws per try) around
    /// the target player's tile that spawns ONE projectile-578 at a valid
    /// cell. `Some((cx,cy))` = the NPC Center the scan anchors on; the
    /// target-player read, the tile probes and the per-try draws are the
    /// wiring wave's (the roster/projectile seams).
    pub nebula_predictor_shot: Option<(f32, f32)>,
    /// The 491 Flying Dutchman's projectile-spawn seam (`NPC.cs:83713-83720`):
    /// 4 projectile-594s at Center, each velocity
    /// `RotatedBy((num191<2 ? +1 : -1) * (π/8 + π/4*NextFloat), (0, -NextFloat*0.5 - 0.5)) * 6`.
    /// `Some((cx,cy))` = the Center; the per-spawn NextFloat draws + the
    /// projectile spawns are the wiring wave's.
    pub dutchman_death_shots: Option<(f32, f32)>,
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`), centre default (0,0): the
/// cos/sin are FLOAT casts of the double radians, the products float.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    Vec2::new(v.x * num - v.y * num2, v.x * num2 + v.y * num)
}

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`): TWO `NextDouble` draws
/// (`NextDouble()*max - NextDouble()*max`), then `RotatedBy`. The draws hit
/// the strike stream in that order.
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let a = rng.next_double() * max_radians;
    let b = rng.next_double() * max_radians;
    rotated_by(v, a - b)
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
/// (`:82441`). The type order is `base/+1/+2/+3/+4/+5/+6/+10/+7/+8/+9`
/// (the +10 = 1019 gore precedes the +7/+8/+9 trio, `:82449-82452`) at the
/// Y ladder +0/+15/+15/+15/+35/+35/+35/+35/+25/+25/+25, the +4 gore shifted
/// -20 in X (`:82446`).
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
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 35.0), vel, num74 + 6, s); // :82448
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 35.0), vel, num74 + 10, s); // :82449
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 25.0), vel, num74 + 7, s); // :82450
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 25.0), vel, num74 + 8, s); // :82451
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 25.0), vel, num74 + 9, s); // :82452
}

/// The 578 DD2 Lightning Bug death (`NPC.cs:82466-82473`, sites 138-141):
/// the 20-dust type-5 burst, then the 4 gores 999/998/998/997 at the
/// +0/+10/+10/+10 Y ladder (the 999 head at position, the rest at +10).
fn lightning_bug_death(
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
    gore.new_gore(rng, world, pos, vel, 999, s); // :82470
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 998, s); // :82471
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 998, s); // :82472
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 997, s); // :82473
}

/// The 620 Goblin Shark death (`NPC.cs:82489-82499`, sites 142-145): the
/// 150-dust `NewDustDirect` type-5 burst (velocity ×2.5, scale = 1.7 — the
/// post-writes draw nothing), then 4 gores at base `num79 = 1183`
/// (`:82495`) with the type order `base/+2/+3/+1` at the +0/+20/+20/+34 Y
/// ladder. The dust runs through `new_dust_direct` (the C# `NewDustDirect`,
/// `Dust.cs:65-73`) on the strike stream.
fn goblin_shark_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82489-82494` — the 150-dust direct burst.
    for _ in 0..150 {
        let idx = dust.new_dust_direct(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.5;
        d.scale = 1.7;
    }
    // `:82495` — num79 = 1183.
    let num79 = 1183;
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num79, s); // :82496
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num79 + 2, s); // :82497
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num79 + 3, s); // :82498
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num79 + 1, s); // :82499
}

/// The 619 Blood Squid death (`NPC.cs:82515-82525`, sites 146-149): the
/// 150-dust `NewDustDirect` type-5 burst (velocity ×2.5, fadeIn = 1.7 — the
/// post-writes draw nothing), then 4 gores at base `num82 = 1179`
/// (`:82521`) with the type order `base/+2/+3/+1` at the +0/+20/+20/+34 Y
/// ladder. Same shape as the Goblin Shark, but the dust post-write is
/// `fadeIn` (not `scale`) and the base is 1179.
fn blood_squid_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82515-82520` — the 150-dust direct burst (fadeIn, not scale).
    for _ in 0..150 {
        let idx = dust.new_dust_direct(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.5;
        d.fade_in = 1.7;
    }
    // `:82521` — num82 = 1179.
    let num82 = 1179;
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num82, s); // :82522
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num82 + 2, s); // :82523
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num82 + 3, s); // :82524
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num82 + 1, s); // :82525
}

/// The 618 Blood Nautilus death (`NPC.cs:82541-82559`, sites 150-161): the
/// 200-dust `NewDustDirect` type-5 burst (velocity ×2.5, scale = 2.5 — the
/// DEATH scale; the hit half uses 2.0, `:82536`), then 12 gores at base
/// `num85 = 1172` (`:82547`). The first two type pairs alternate the
/// `Right.X - 30` anchor with `position`; the rest sit at `position`.
/// Sites 150-161 (all twelve gores).
fn blood_nautilus_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82541-82546` — the 200-dust direct burst (death scale 2.5).
    for _ in 0..200 {
        let idx = dust.new_dust_direct(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.5;
        d.scale = 2.5;
    }
    // `:82547` — num85 = 1172.
    let num85 = 1172;
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let rx = right_x(npc) - 30.0;
    gore.new_gore(rng, world, Vec2::new(rx, npc.y), vel, num85, s); // :82548
    gore.new_gore(rng, world, pos, vel, num85, s); // :82549
    gore.new_gore(rng, world, Vec2::new(rx, npc.y), vel, num85 + 1, s); // :82550
    gore.new_gore(rng, world, pos, vel, num85 + 1, s); // :82551
    gore.new_gore(rng, world, pos, vel, num85 + 2, s); // :82552
    gore.new_gore(rng, world, pos, vel, num85 + 3, s); // :82553
    gore.new_gore(rng, world, pos, vel, num85 + 4, s); // :82554
    gore.new_gore(rng, world, pos, vel, num85 + 4, s); // :82555
    gore.new_gore(rng, world, pos, vel, num85 + 5, s); // :82556
    gore.new_gore(rng, world, pos, vel, num85 + 5, s); // :82557
    gore.new_gore(rng, world, pos, vel, num85 + 6, s); // :82558
    gore.new_gore(rng, world, pos, vel, num85 + 6, s); // :82559
}

/// The 541 Sand Elemental death (`NPC.cs:82585-82603`, sites 162-165): the
/// 30-iteration burst, each spawning TWO type-268 dusts tinted
/// `new Color(222,108,48) * 0.7f` with noGravity, scale 1.5, fadeIn 0.7,
/// velocity ×3 (the post-writes draw nothing), then the 4 gores
/// 960/961/963/962 at the +0/+20/+20/+40 Y ladder (the 963 precedes the
/// 962, `:82602-82603`). The tint rides the XNA `Color * float` truncation
/// (`ui::color::mul`): (155,75,33,178).
fn sand_elemental_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    _hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82564` — the tint `new Color(222,108,48) * 0.7f` (alpha 255 * 0.7).
    let t = crate::ui::color::mul([222, 108, 48, 255], 0.7);
    let tint = Color::new(t[0], t[1], t[2], t[3]);
    // `:82585-82599` — 30 iterations x 2 type-268 dusts. The C# NewDust
    // here passes NO hitDirection (the 4-arg overload, SpeedX/Y default 0).
    for _ in 0..30 {
        for _ in 0..2 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 268, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.color = tint;
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 0.7;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 960, s); // :82600
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 961, s); // :82601
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 963, s); // :82602
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 40.0), vel, 962, s); // :82603
}

/// The 546 Tumbleweed death (`NPC.cs:82629-82652`, sites 166-168): the
/// 20-iteration burst, each spawning TWO type-268 dusts tinted
/// `new Color(150,108,78) * 0.7f` (noGravity, scale 1.5, fadeIn 0.7,
/// velocity ×3), then a 20-dust type-54 burst (alpha 50, scale 1.5,
/// velocity ×2, noGravity), then the 3 gores 964/965/966 at the +0/+20/+20
/// Y ladder. The tint rides the XNA truncation: (105,75,54,178).
fn tumbleweed_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    _hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82608` — the tint `new Color(150,108,78) * 0.7f`.
    let t = crate::ui::color::mul([150, 108, 78, 255], 0.7);
    let tint = Color::new(t[0], t[1], t[2], t[3]);
    // `:82629-82643` — 20 iterations x 2 type-268 dusts (4-arg NewDust).
    for _ in 0..20 {
        for _ in 0..2 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 268, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.color = tint;
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 0.7;
            d.velocity *= 3.0;
        }
    }
    // `:82644-82649` — the 20-dust type-54 burst (alpha 50, scale 1.5).
    for _ in 0..20 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 54, 0.0, 0.0, 50,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.no_gravity = true;
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 964, s); // :82650
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 965, s); // :82651
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 966, s); // :82652
}

/// The 437 Cultist Tablet `dmg == 9999` burst (`NPC.cs:82672-82677`, sites
/// 169-174): 6 gores 728-733 at `Center - (10,10)`, each with velocity
/// `Vector2.UnitY.RotatedByRandom(2π) * 5f`. The `RotatedByRandom` draws TWO
/// `NextDouble` per gore BEFORE the spawn's kicks; the types are the loop
/// index (no type draw). Gated on `dmg == 9999.0` (the C# exact-double
/// compare) — NOT a `life <= 0` gate.
fn cultist_tablet_burst(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let anchor = center(npc) - Vec2::new(10.0, 10.0);
    for num94 in 0..6 {
        // `:82676` — UnitY.RotatedByRandom(2π) * 5 (two NextDouble draws).
        let vel = rotated_by_random(Vec2::new(0.0, 1.0), 6.2831854820251465, rng) * 5.0;
        gore.new_gore(rng, world, anchor, vel, 728 + num94, 1.0);
    }
}

/// The 513-515 Tomb Crawler death (`NPC.cs:82739-82744`, sites 175-177):
/// the 50-dust type-250 burst (SpeedY -2, alpha/color/scale from the NPC),
/// then ONE gore of type `this.type - 513 + 822` at position with
/// `Main.gore[num].alpha = this.alpha` post-write. The three segments
/// (513/514/515) map to gores 822/823/824.
///
/// `this.color` stand-in: the worm kinds set NO color in `SetDefaults`
/// (`NPC.cs:8526` `this.color = default`), so the dust color is
/// `Color::default()` (all-zero); the port's `Npc` carries `alpha` but no
/// color field (the wave-A `HitEffectInputs` stand-in convention).
fn tomb_crawler_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:82739-82742` — the 50-dust type-250 burst (SpeedY -2).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, hit_direction, -2.0, alpha,
            Color::default(), npc.scale,
        );
    }
    // `:82743-82744` — the single gore + the alpha copy.
    let ty = npc_type - 513 + 822;
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 510-512 Dune Splicer death (`NPC.cs:82758-82763`, sites 178-180):
/// the 50-dust type-250 burst (SpeedY -2), then ONE gore of type
/// `this.type - 510 + 819` with the alpha copy. The three segments
/// (510/511/512) map to gores 819/820/821. Same shape as the Tomb Crawler,
/// different base/type offset.
fn dune_splicer_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:82758-82761` — the 50-dust type-250 burst (SpeedY -2).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, hit_direction, -2.0, alpha,
            Color::default(), npc.scale,
        );
    }
    // `:82762-82763` — the single gore + the alpha copy.
    let ty = npc_type - 510 + 819;
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 621-623 Blood Eel death (`NPC.cs:82777-82782`, sites 181-183): the
/// 50-dust type-5 burst (SpeedY -2, alpha/color/scale from the NPC), then
/// ONE gore of type `this.type - 621 + 1187` with the alpha copy. The three
/// segments (621/622/623) map to gores 1187/1188/1189. Same shape as the
/// other worm arms, but the dust is type 5 (not 250). The 621-623 family
/// sets `alpha = 255` in SetDefaults (`NPC.cs:16985`), carried by `npc.alpha`.
fn blood_eel_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:82777-82780` — the 50-dust type-5 burst (SpeedY -2).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -2.0, alpha,
            Color::default(), npc.scale,
        );
    }
    // `:82781-82782` — the single gore + the alpha copy.
    let ty = npc_type - 621 + 1187;
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 518/416 Solar Spearman / Drakomire Rider death (`NPC.cs:82804-82820`,
/// sites 184-188): the 20-iteration burst, each a type-241 dust then a
/// `Main.rand.Next(3) == 0`-gated type-6 dust (noGravity, scale 1.5,
/// fadeIn 1, velocity ×3); then the 5 gores 844/845/847/835/836 at the
/// +0/+20/+20/+20/+20 Y ladder. The death gate is `Next(3)` (the hit half
/// uses `Next(4)`, `:82792`).
fn solar_spearman_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82804-82815` — 20 iterations: type-241 dust + the Next(3)-gated type-6.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        // `:82807` — the death gate is Next(3) == 0 (draws before the spawn).
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 844, s); // :82816
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 845, s); // :82817
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 847, s); // :82818
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 835, s); // :82819
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 836, s); // :82820
}

/// The 419 Solar Solenian death (`NPC.cs:82842-82859`, sites 189-194): the
/// 20-iteration burst (type-241 dust + `Next(3) == 0`-gated type-6 dust,
/// same as the Spearman), then the 6 gores 844/845/846/846/847/847 at the
/// +0/+20/+20/+20/+20/+20 Y ladder.
fn solar_solenian_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82842-82853` — 20 iterations: type-241 + the Next(3)-gated type-6.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 844, s); // :82854
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 845, s); // :82855
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 846, s); // :82856
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 846, s); // :82857
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 847, s); // :82858
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 847, s); // :82859
}

/// The 412-414 Solar Crawltipede death (`NPC.cs:82873-82881`, sites
/// 195-197): the 6-dust type-241 burst (SpeedX `2.5*hitDirection`, SpeedY
/// -2.5) then the 2-dust type-6 burst (alpha 100, Color.Transparent, scale
/// 2), then ONE gore of type `this.type - 412 + 828` (412/413/414 ->
/// 828/829/830). No alpha copy here (unlike the worm arms).
fn solar_crawltipede_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82873-82876` — the 6-dust type-241 burst.
    for _ in 0..6 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    // `:82877-82880` — the 2-dust type-6 burst (alpha 100, transparent, scale 2).
    for _ in 0..2 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 2.5 * hit_direction, -2.5, 100,
            Color::TRANSPARENT, 2.0,
        );
    }
    // `:82881` — the single gore (no alpha copy).
    let ty = npc_type - 412 + 828;
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
}

/// The 415 Solar Flare death (`NPC.cs:82903-82915`, sites 198-201): the
/// 40-iteration burst, each a type-241 dust AND an UNGATED type-6 dust
/// (noGravity, scale 1.5, fadeIn 1, velocity ×3 — no Next gate here), then
/// the 4 gores 831/832/833/834 at Center with velocity ×0.8 (831/832) and
/// ×0.9 (833/834). Scale defaults to 1 (the 3-arg NewGore).
fn solar_flare_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82903-82911` — 40 iterations: type-241 + ungated type-6.
    for _ in 0..40 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.5;
        d.fade_in = 1.0;
        d.velocity *= 3.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 831, 1.0); // :82912
    gore.new_gore(rng, world, c, vel * 0.8, 832, 1.0); // :82913
    gore.new_gore(rng, world, c, vel * 0.9, 833, 1.0); // :82914
    gore.new_gore(rng, world, c, vel * 0.9, 834, 1.0); // :82915
}

/// The 417 Solar Sroller death (`NPC.cs:82937-82956`, sites 202-209): the
/// 30-iteration burst (type-241 dust + `Next(2) == 0`-gated type-6 dust),
/// then the 8 gores 837/838/838/838/839/839/840/840 at the +0/+20/+38 Y
/// ladder. Sites 202-209 (all eight gores).
fn solar_sroller_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82937-82948` — 30 iterations: type-241 + the Next(2)-gated type-6.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(2) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 837, s); // :82949
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 838, s); // :82950
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 838, s); // :82951
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 838, s); // :82952
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 839, s); // :82953
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 839, s); // :82954
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 38.0), vel, 840, s); // :82955
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 38.0), vel, 840, s); // :82956
}

/// The 418 Solar Corite death (`NPC.cs:82978-82991`, sites 210-214): the
/// 20-iteration burst (type-241 dust + UNGATED type-6 dust, noGravity,
/// scale 1.5, fadeIn 1, velocity ×3), then the 5 gores 841/842/842/843/843
/// at Center with velocity ×0.8 (841/842/842) and ×0.9 (843/843). Scale
/// defaults to 1. Same shape as the Solar Flare, different types/counts.
fn solar_corite_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82978-82986` — 20 iterations: type-241 + ungated type-6.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.5;
        d.fade_in = 1.0;
        d.velocity *= 3.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 841, 1.0); // :82987
    gore.new_gore(rng, world, c, vel * 0.8, 842, 1.0); // :82988
    gore.new_gore(rng, world, c, vel * 0.8, 842, 1.0); // :82989
    gore.new_gore(rng, world, c, vel * 0.9, 843, 1.0); // :82990
    gore.new_gore(rng, world, c, vel * 0.9, 843, 1.0); // :82991
}

/// The 426 Nebula Floater death (`NPC.cs:83058-83083`, sites 215-217): the
/// 20-dust type-241 burst, then the 30-dust `SelectRandom(229,240,161)`
/// burst (noGravity, scale `1.25 + NextFloat`, fadeIn 0.25, velocity ×2,
/// noLight), then the 3 gores 799/800/801 at Top/Center/Center with
/// velocity ×0.7/×0.8/×0.9. The NPC-428 brain-spawn seam
/// (`:83075-83083`) is RETURNED ([`HitEffectOutcomeC::nebula_brain_spawn`])
/// — the roster count gate + the per-spawn draws are the wiring wave's.
fn nebula_floater_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83058-83061` — the 20-dust type-241 burst.
    death_dust_loop(npc, dust, dust_ctx, rng, world, 241, hit_direction, 20);
    // `:83062-83071` — the 30-dust SelectRandom(229,240,161) burst.
    for _ in 0..30 {
        let ty = select_random(rng, &[229, 240, 161]);
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.25 + rng.next_float();
        d.fade_in = 0.25;
        d.velocity *= 2.0;
        d.no_light = true;
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, top(npc), vel * 0.7, 799, 1.0); // :83072
    gore.new_gore(rng, world, center(npc), vel * 0.8, 800, 1.0); // :83073
    gore.new_gore(rng, world, center(npc), vel * 0.9, 801, 1.0); // :83074
    // `:83075-83083` — the brain-spawn seam (netMode != 1, single-player
    // runs); the count gate + the spawns are the caller's.
    let c = center(npc);
    outcome.nebula_brain_spawn = Some((c.x, c.y));
}

/// The 427 Nebula Soldier death (`NPC.cs:83105-83122`, sites 218-221): the
/// 10-dust type-241 burst, then the 20-dust `SelectRandom(229,229,240)`
/// burst (noGravity, scale `1.25 + NextFloat`, fadeIn 0.25, velocity ×3,
/// noLight), then the 4 gores 802/803/804/805 at Center with velocity
/// ×0.8/×0.9/×0.9/×0.8. No spawn seam here.
fn nebula_soldier_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83105-83108` — the 10-dust type-241 burst.
    death_dust_loop(npc, dust, dust_ctx, rng, world, 241, hit_direction, 10);
    // `:83109-83118` — the 20-dust SelectRandom(229,229,240) burst.
    for _ in 0..20 {
        let ty = select_random(rng, &[229, 229, 240]);
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.25 + rng.next_float();
        d.fade_in = 0.25;
        d.velocity *= 3.0;
        d.no_light = true;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 802, 1.0); // :83119
    gore.new_gore(rng, world, c, vel * 0.9, 803, 1.0); // :83120
    gore.new_gore(rng, world, c, vel * 0.9, 804, 1.0); // :83121
    gore.new_gore(rng, world, c, vel * 0.8, 805, 1.0); // :83122
}

/// The 429 Nebula Predictor death (`NPC.cs:83144-83204`, sites 222-226):
/// the 20-iteration burst (type-241 dust + `Next(3) == 0`-gated type-229
/// dust, noGravity, scale 1.5, fadeIn 1, velocity ×3), then the 5 gores
/// 808/809/809/810/810 at the +0/+20/+20/+34/+34 Y ladder. The
/// projectile-578 tile-scan spawn seam (`:83161-83204`) is RETURNED
/// ([`HitEffectOutcomeC::nebula_predictor_shot`]) — the target-player read,
/// the tile probes and the per-try `Next` draws are the wiring wave's.
fn nebula_predictor_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83144-83155` — 20 iterations: type-241 + the Next(3)-gated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 808, s); // :83156
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83157
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83158
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83159
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83160
    // `:83161-83204` — the projectile-spawn seam (netMode != 1, single-player
    // runs); the scan + the spawn are the caller's.
    let c = center(npc);
    outcome.nebula_predictor_shot = Some((c.x, c.y));
}

/// The 425 Nebula Beast death (`NPC.cs:83226-83244`, sites 227-233): the
/// 20-iteration burst (type-241 dust + `Next(3) == 0`-gated type-229 dust,
/// noGravity, scale 1.5, fadeIn 1, velocity ×3), then the 7 gores
/// 796/797/798/809/809/810/810 — 796 at position, 797 at Center, 798/809/809
/// at +20 Y, 810/810 at +34 Y.
fn nebula_beast_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83226-83237` — 20 iterations: type-241 + the Next(3)-gated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 796, s); // :83238
    gore.new_gore(rng, world, center(npc), vel, 797, s); // :83239
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 798, s); // :83240
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83241
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83242
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83243
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83244
}

/// The 428 Nebula Brain death (`NPC.cs:83258-83271`, sites 234-235): the
/// 10-iteration burst, each a type-241 dust then a `Next(2) == 0` ? type-229
/// : type-240 dust (the gate picks the type, both branches spawn), then the
/// 2 gores 806/807 at position with velocity ×0.8. Scale defaults to 1.
fn nebula_brain_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83258-83269` — 10 iterations: type-241 + the Next(2)-picked 229/240.
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        // `:83261-83268` — the gate picks the type; both branches spawn.
        let ty = if rng.next_max(2) == 0 { 229 } else { 240 };
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy) * 0.8;
    gore.new_gore(rng, world, pos, vel, 806, 1.0); // :83270
    gore.new_gore(rng, world, pos, vel, 807, 1.0); // :83271
}

/// The Vortex death dust burst shared by the 424/423/421/420 arms
/// (`NPC.cs:83293-83304` et al.): `count` iterations, each a type-249 dust
/// then a type-242 dust (noGravity, scale 1.5, fadeIn 1, velocity ×3). When
/// `gate` is `Some(n)` the type-242 spawn is `Next(n) == 0`-gated (the draw
/// still fires); when `None` it is ungated. The 424 arm gates `Next(3)`;
/// the 423/421/420 death halves are ungated.
fn vortex_death_dust(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    count: i32,
    gate: Option<i32>,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 249, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let take = match gate {
            Some(n) => rng.next_max(n) == 0,
            None => true,
        };
        if take {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 242, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
}

/// The 424 Vortex Soldier death (`NPC.cs:83293-83309`, sites 236-240): the
/// 20-iteration type-249 + `Next(3) == 0`-gated type-242 burst, then the 5
/// gores 793/794/794/795/795 at the +0/+20/+20/+34/+34 Y ladder.
fn vortex_soldier_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 20, Some(3));
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 793, s); // :83305
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 794, s); // :83306
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 794, s); // :83307
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 795, s); // :83308
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 795, s); // :83309
}

/// The 423 Vortex Hornet death (`NPC.cs:83331-83342`, sites 241-243): the
/// 20-iteration UNGATED type-249 + type-242 burst, then the 3 gores
/// 790/791/792 at Center with velocity ×0.8/×0.8/×0.9. Scale defaults to 1.
fn vortex_hornet_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 20, None);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 790, 1.0); // :83340
    gore.new_gore(rng, world, c, vel * 0.8, 791, 1.0); // :83341
    gore.new_gore(rng, world, c, vel * 0.9, 792, 1.0); // :83342
}

/// The 421 Vortex Larva death (`NPC.cs:83364-83375`, sites 244-246): the
/// 10-iteration UNGATED type-249 + type-242 burst, then the 3 gores
/// 785/786/787 at Center with velocity ×0.8/×0.8/×0.9. Same shape as the
/// Hornet, 10 iterations and different types.
fn vortex_larva_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 10, None);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 785, 1.0); // :83373
    gore.new_gore(rng, world, c, vel * 0.8, 786, 1.0); // :83374
    gore.new_gore(rng, world, c, vel * 0.9, 787, 1.0); // :83375
}

/// The 420 Vortex Rider death (`NPC.cs:83397-83408`, sites 247-249): the
/// 10-iteration UNGATED type-249 + type-242 burst, then the 3 gores
/// 782/783/784 at Center with velocity ×0.8/×0.8/×0.9. Same shape as the
/// Larva, different types.
fn vortex_rider_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 10, None);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 782, 1.0); // :83406
    gore.new_gore(rng, world, c, vel * 0.8, 783, 1.0); // :83407
    gore.new_gore(rng, world, c, vel * 0.9, 784, 1.0); // :83408
}

/// The 409 Stardust Spider death (`NPC.cs:83496-83505`, sites 250-252): the
/// 20-iteration UNGATED type-17 + type-229 burst (the 229 gets noGravity,
/// velocity ×3 — NO scale/fadeIn writes here), then the 3 gores 775/776/777
/// at Center with velocity ×0.8/×0.8/×0.9. Scale defaults to 1.
fn stardust_spider_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83496-83502` — 20 iterations: type-17 + ungated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 17, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 775, 1.0); // :83503
    gore.new_gore(rng, world, c, vel * 0.8, 776, 1.0); // :83504
    gore.new_gore(rng, world, c, vel * 0.9, 777, 1.0); // :83505
}

/// The 407 Stardust Cell death (`NPC.cs:83547-83562`, sites 253-258): the
/// 20-iteration burst, each a `Next(3) != 0`-gated type-17 dust (the INVERSE
/// gate — the draw fires, the spawn is skipped on 0) then an UNGATED
/// type-229 dust (noGravity, velocity ×3); then the 6 gores 778/779 at Top
/// (vel ×0.8) and 780/781/780/781 at Center (vel ×0.9). Scale defaults to 1.
fn stardust_cell_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83547-83556` — 20 iterations: the Next(3)!=0-gated type-17 + ungated 229.
    for _ in 0..20 {
        // `:83549` — the INVERSE gate: spawn when Next(3) != 0.
        if rng.next_max(3) != 0 {
            dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 17, hit_direction, -1.0, 0,
                Color::default(), 1.0,
            );
        }
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    let t = top(npc);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, t, vel * 0.8, 778, 1.0); // :83557
    gore.new_gore(rng, world, t, vel * 0.8, 779, 1.0); // :83558
    gore.new_gore(rng, world, c, vel * 0.9, 780, 1.0); // :83559
    gore.new_gore(rng, world, c, vel * 0.9, 781, 1.0); // :83560
    gore.new_gore(rng, world, c, vel * 0.9, 780, 1.0); // :83561
    gore.new_gore(rng, world, c, vel * 0.9, 781, 1.0); // :83562
}

/// The 411 Stardust Dragon body death (`NPC.cs:83649-83661`, sites
/// 259-263): the 20-iteration burst (type-17 dust + `Next(2) == 0`-gated
/// type-229 dust, noGravity only — no velocity/scale write), then the 5
/// gores 772/773/773/774/774 at the +0/+20/+20/+34/+34 Y ladder.
fn stardust_dragon_body_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83649-83656` — 20 iterations: type-17 + the Next(2)-gated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 17, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(2) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            dust.slot_mut(idx).no_gravity = true;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 772, s); // :83657
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 773, s); // :83658
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 773, s); // :83659
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 774, s); // :83660
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 774, s); // :83661
}

/// The 491 Flying Dutchman death's HEAD block (`NPC.cs:83668-83677`, sites
/// 264-274): the anchor `vector15 = Center - (60,60)` and the facing unit
/// `vector16 = ((spriteDirection != 1) ? 1 : -1, 0).RotatedBy(rotation)`,
/// then the 858 gore at `+ vector16 * (width/2)` and the 857 at
/// `+ vector16 * (width/3)` (the C# `base.width / 2` / `/ 3` are INTEGER
/// divisions — width is int), then the `num183 = -1..1` loop's 3×3 = 9 gores
/// of type 859 at `width * (0.45/0.4/0.35 + num183*0.1)`. No arm-level draws
/// (the types/anchors are data); each spawn draws only the NewGore kicks.
/// The rest of the 491 arm (the RandomVector2 fan, the 855/854/853/856
/// loops, the 30-gore 861/862 burst, the 120-dust burst and the projectile
/// seam) lands with the next batches.
fn dutchman_death_head(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    // `:83668` — vector15 = Center - (60,60).
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    // `:83669` — vector16 = (dir, 0).RotatedBy(rotation); dir = +1 unless
    // spriteDirection == 1.
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83670-83671` — the integer width/2 and width/3.
    let wi = w as i32;
    gore.new_gore(rng, world, v15 + v16 * (wi / 2) as f32, vel, 858, s);
    gore.new_gore(rng, world, v15 + v16 * (wi / 3) as f32, vel, 857, s);
    // `:83672-83677` — the num183 = -1..1 loop, 3 gores of 859 each.
    for num183 in -1..2 {
        let f = num183 as f32;
        gore.new_gore(rng, world, v15 + v16 * (w * (0.45 + f * 0.1)), vel, 859, s); // :83674
        gore.new_gore(rng, world, v15 + v16 * (w * (0.4 + f * 0.1)), vel, 859, s); // :83675
        gore.new_gore(rng, world, v15 + v16 * (w * (0.35 + f * 0.1)), vel, 859, s); // :83676
    }
}

/// The 491 Flying Dutchman death's RandomVector2 FAN (`NPC.cs:83678-83685`,
/// sites 275-289): the `num184 = -1..1` loop, each spawning 5 gores of type
/// `SelectRandom(860,892,893)` at `v15 + v16*(width*k) - (0,drop) +
/// RandomVector2(-40,40)`. The per-gore draw order is the C# argument
/// evaluation: the position's `RandomVector2` (TWO NextDouble, X then Y)
/// FIRST, then the type's `SelectRandom` (ONE Next(3)), then the NewGore
/// kicks. Sites 275-289 (all three iterations).
fn dutchman_death_fan(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83678` — num184 = -1..1 (all three iterations).
    for num184 in -1..2 {
        let _ = num184; // the anchors do not depend on the loop index here
        // `:83680` — width*0.35, drop 200.
        let p = v15 + v16 * (w * 0.35) - Vec2::new(0.0, 200.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83681` — width*0.15, drop 200.
        let p = v15 + v16 * (w * 0.15) - Vec2::new(0.0, 200.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83682` — width*-0.35, drop 200.
        let p = v15 + v16 * (w * -0.35) - Vec2::new(0.0, 200.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83683` — width*-0.15, drop 300.
        let p = v15 + v16 * (w * -0.15) - Vec2::new(0.0, 300.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83684` — width*-0.15, drop 300 (the repeat).
        let p = v15 + v16 * (w * -0.15) - Vec2::new(0.0, 300.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
    }
}

/// The 491 Flying Dutchman death's 855 loop (`NPC.cs:83686-83690`, sites
/// 290-293): the `num185 = 0..1` loop, each spawning 2 gores of type 855 at
/// `v15 + v16*(width*±0.15) - (0, 160 + 60*num185)`. No arm-level draws.
fn dutchman_death_855(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83686` — num185 = 0..1.
    for num185 in 0..2 {
        let drop = (160 + 60 * num185) as f32;
        gore.new_gore(rng, world, v15 + v16 * (w * 0.15) - Vec2::new(0.0, drop), vel, 855, s); // :83688
        gore.new_gore(rng, world, v15 + v16 * (w * -0.15) - Vec2::new(0.0, drop), vel, 855, s); // :83689
    }
}

/// The 491 Flying Dutchman death's 854/853/856 loop (`NPC.cs:83691-83696`,
/// sites 294-302): the `num186 = -1..1` loop, each spawning 3 gores — 854 at
/// `width*(0.25 + num186*0.1)`, 853 at `width*(-0.45 + num186*0.1)`, 856 at
/// `width*(0.05 + num186*0.1)`, all dropped by `(0, 20 + 40*num186)`. No
/// arm-level draws.
fn dutchman_death_854(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83691` — num186 = -1..1.
    for num186 in -1..2 {
        let f = num186 as f32;
        let drop = (20 + 40 * num186) as f32;
        gore.new_gore(rng, world, v15 + v16 * (w * (0.25 + f * 0.1)) - Vec2::new(0.0, drop), vel, 854, s); // :83693
        gore.new_gore(rng, world, v15 + v16 * (w * (-0.45 + f * 0.1)) - Vec2::new(0.0, drop), vel, 853, s); // :83694
        gore.new_gore(rng, world, v15 + v16 * (w * (0.05 + f * 0.1)) - Vec2::new(0.0, drop), vel, 856, s); // :83695
    }
}

/// The 491 Flying Dutchman death's 861/862 SCATTER burst (`NPC.cs:83697-83703`,
/// sites 303-332): 30 gores of type `Next(861,863)` at `Center - (60,60)`
/// with velocity Zero, then per-gore post-writes: `alpha = 100`, velocity =
/// `Vector2.Transform((4,0), rotZ(NextDouble*2π))` = `(4cosθ, 4sinθ)` (ONE
/// NextDouble), position = `(pos.X - 20 + (width+40)*NextFloat, pos.Y - 240
/// + (height+240-20)*NextFloat)` (TWO NextFloat). The per-gore draw order:
/// the type `Next(861,863)` FIRST (the C# named-arg Type evaluates before
/// Position/Velocity, but those are data here), then the NewGore kicks
/// (pool), then the post-write NextDouble + two NextFloat. Sites 303-332
/// (all 30 gores).
///
/// NOTE: the 861/862 types hit the pool's sticky=false arm (`Gore.cs:1343`)
/// and the SpecialAI==1 velocity roll is NOT in the 861/862 set, so the
/// post-write velocity overwrite is the final value.
fn dutchman_death_scatter(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let anchor = center(npc) - Vec2::new(60.0, 60.0);
    // `:83697` — num187 = 0..30 (all 30 gores).
    for _ in 0..30 {
        // `:83699` — the type draw, then the spawn at the anchor, vel Zero.
        let ty = rng.next_range(861, 863);
        let idx = gore.new_gore(rng, world, anchor, Vec2::ZERO, ty, 1.0);
        // `:83700` — alpha = 100 (no draw).
        let g = gore.slot_mut(idx);
        g.alpha = 100;
        // `:83701` — velocity = Transform((4,0), rotZ(NextDouble*2π)).
        let theta = rng.next_double() * 6.2831854820251465;
        let (s, c) = (theta as f32).sin_cos();
        gore.slot_mut(idx).velocity = Vec2::new(4.0 * c, 4.0 * s);
        // `:83702` — the position re-roll (two NextFloat).
        let px = npc.x - 20.0 + (w + 40.0) * rng.next_float();
        let py = npc.y - 240.0 + (h + 240.0 - 20.0) * rng.next_float();
        gore.slot_mut(idx).position = Vec2::new(px, py);
    }
}

/// The 491 Flying Dutchman death's TAIL (`NPC.cs:83704-83720`): the
/// 120-iteration dust burst, each a type-6 dust (alpha 100, scale 2.5,
/// noGravity, velocity ×5) then a type-6 dust (alpha 100, scale 1.5,
/// velocity ×4, noGravity) — the post-writes draw nothing; then the
/// projectile-594 spawn seam (`:83713-83720`) is RETURNED
/// ([`HitEffectOutcomeC::dutchman_death_shots`]) — the per-spawn NextFloat
/// draws + the spawns are the wiring wave's. No gore sites here; this
/// completes the 491 arm's strike-stream census.
fn dutchman_death_tail(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83704-83712` — 120 iterations x 2 type-6 dusts.
    for _ in 0..120 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 100,
            Color::default(), 2.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 5.0;
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 100,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 4.0;
        d.no_gravity = true;
    }
    // `:83713-83720` — the projectile-spawn seam (netMode != 1, single-player
    // runs); the per-spawn draws + the spawns are the caller's.
    let c = center(npc);
    outcome.dutchman_death_shots = Some((c.x, c.y));
}

/// The 492 Flying Dutchman cannon death (`NPC.cs:83727-83737`, sites
/// 333-342): the 7-dust type-31 burst (alpha 100, scale 1.5, velocity ×1.4
/// — the post-write draws nothing), then 10 gores of type `Next(61,64)` at
/// position with velocity Zero, then per-gore post-writes: `alpha = 100`,
/// velocity = `Vector2.Transform((1,0), rotZ(NextDouble*2π))` =
/// `(cosθ, sinθ)` (ONE NextDouble). The per-gore draw order: the type
/// `Next(61,64)` FIRST, then the NewGore kicks (pool), then the post-write
/// NextDouble. The 61/62/63 types hit the pool's sticky=false arm
/// (`Gore.cs:1341`), no SpecialAI roll.
fn dutchman_cannon_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83727-83731` — the 7-dust type-31 burst.
    for _ in 0..7 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 31, 0.0, 0.0, 100,
            Color::default(), 1.5,
        );
        dust.slot_mut(idx).velocity *= 1.4;
    }
    // `:83732-83737` — the 10-gore scatter.
    for _ in 0..10 {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.alpha = 100;
        let theta = rng.next_double() * 6.2831854820251465;
        let (s, c) = (theta as f32).sin_cos();
        gore.slot_mut(idx).velocity = Vec2::new(c, s);
    }
}

/// The 524-527 Desert Ghoul death (`NPC.cs:83742-83794`, sites 343-348):
/// the per-tier params (`num196` accent-dust type, `num197 = 5` base dust,
/// `num198` gore base): 524 → (0, 5, 863), 525 → (75, 5, 867), 526 →
/// (170, 5, 871), 527 → (67, 5, 879). The 20-iteration type-5 burst with a
/// `Next(3) == 0 && num196 > 0`-gated accent dust (noGravity, scale 1.5,
/// fadeIn 1, velocity ×3) — the C# `&&` evaluates `Next(3)` FIRST, so the
/// draw fires even for 524 (num196 == 0) but the spawn is skipped. Then the
/// 6 gores `num198 / +1 / +2 / +2 / +3 / +3` at the +0/+0/+20/+20/+34/+34 Y
/// ladder.
fn desert_ghoul_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:83742-83759` — the per-tier params.
    let (num196, num198) = match npc_type {
        525 => (75, 867),
        526 => (170, 871),
        527 => (67, 879),
        _ => (0, 863), // 524
    };
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83777-83788` — 20 iterations: type-5 + the gated accent dust.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        // `:83780` — Next(3) draws FIRST (the && left operand), then the
        // num196 > 0 check gates the spawn.
        if rng.next_max(3) == 0 && num196 > 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, num196, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num198, s); // :83789
    gore.new_gore(rng, world, pos, vel, num198 + 1, s); // :83790
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num198 + 2, s); // :83791
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num198 + 2, s); // :83792
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num198 + 3, s); // :83793
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num198 + 3, s); // :83794
}

/// The 528-529 Desert Lamia death (`NPC.cs:83799-83821`, sites 349-352):
/// the 20-dust type-5 burst (no accent), then the 4 gores — the head is
/// `num202` (528 → 875, 529 → 876, `:83800-83804`), then 877/877 at +20 Y
/// and 878 at +34 Y (the tail types are FIXED, not base-relative).
fn desert_lamia_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:83799-83804` — the per-tier head.
    let num202 = if npc_type == 529 { 876 } else { 875 };
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num202, s); // :83818
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 877, s); // :83819
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 877, s); // :83820
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 878, s); // :83821
}

/// The 530/531 Desert Scorpion death (`NPC.cs:83835-83843`, sites 353-357):
/// the 20-dust type-250 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5, alpha
/// 100), then the 5 gores 883/884/885/886/887 at position. Scale defaults
/// to 1 (the 3-arg NewGore).
fn desert_scorpion_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83835-83838` — the 20-dust type-250 burst (alpha 100).
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, 2.5 * hit_direction, -2.5, 100,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 883, 1.0); // :83839
    gore.new_gore(rng, world, pos, vel, 884, 1.0); // :83840
    gore.new_gore(rng, world, pos, vel, 885, 1.0); // :83841
    gore.new_gore(rng, world, pos, vel, 886, 1.0); // :83842
    gore.new_gore(rng, world, pos, vel, 887, 1.0); // :83843
}

/// The 532 Desert Beast death (`NPC.cs:83857-83864`, sites 358-361): the
/// 50-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5, alpha 100),
/// then the 4 gores 888/889/890/891 at position. Scale defaults to 1.
fn desert_beast_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83857-83860` — the 50-dust type-5 burst (alpha 100).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 100,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 888, 1.0); // :83861
    gore.new_gore(rng, world, pos, vel, 889, 1.0); // :83862
    gore.new_gore(rng, world, pos, vel, 890, 1.0); // :83863
    gore.new_gore(rng, world, pos, vel, 891, 1.0); // :83864
}

/// `NPC.HitEffect(hitDirection, dmg)` (`NPC.cs:81654+`) — wave C: the third
/// batch of gore sites, `:82206` onward. `npc.hp` is `this.life` AT THE CALL
/// (the strike writes it before calling, `:81488`); the death (`life <= 0`)
/// halves carry gore here, plus the 437 tablet's `dmg == 9999.0` burst (a
/// dmg-gated arm, not life-gated). `hit_direction` is the C# `hitDirection`
/// (a velocity offset for the dust, no extra draw); `dmg` rides the tablet
/// gate (the dmg-scaled hit-half dust loops are the wiring wave's).
///
/// The dispatch grows one arm per commit; the wiring wave folds it into the
/// wave-A/B `StrikeNPC_Inner` claim. Returns the roster side-effects the
/// wiring caller owes ([`HitEffectOutcomeC`]).
pub fn hit_effect_c(
    npc: &Npc,
    npc_type: i32,
    hit_direction: f32,
    dmg: f64,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) -> HitEffectOutcomeC {
    let mut outcome = HitEffectOutcomeC::default();
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
    // `:82426-82454` — the 564/565 DD2 Dark Mage arm (sites 127-137, all
    // eleven gores).
    if (npc_type == 564 || npc_type == 565) && npc.hp <= 0 {
        dark_mage_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82455-82475` — the 578 DD2 Lightning Bug arm (sites 138-141).
    if npc_type == 578 && npc.hp <= 0 {
        lightning_bug_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82476-82501` — the 620 Goblin Shark arm (sites 142-145).
    if npc_type == 620 && npc.hp <= 0 {
        goblin_shark_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82502-82527` — the 619 Blood Squid arm (sites 146-149).
    if npc_type == 619 && npc.hp <= 0 {
        blood_squid_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82528-82561` — the 618 Blood Nautilus arm (sites 150-161, all
    // twelve gores).
    if npc_type == 618 && npc.hp <= 0 {
        blood_nautilus_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82562-82605` — the 541 Sand Elemental arm (sites 162-165).
    if npc_type == 541 && npc.hp <= 0 {
        sand_elemental_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82606-82654` — the 546 Tumbleweed arm (sites 166-168).
    if npc_type == 546 && npc.hp <= 0 {
        tumbleweed_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82672-82678` — the 437 Cultist Tablet burst (sites 169-174), gated
    // on the exact `dmg == 9999.0` double compare (NOT life <= 0).
    if npc_type == 437 && dmg == 9999.0 {
        cultist_tablet_burst(npc, gore, rng, world);
    }
    // `:82728-82746` — the 513-515 Tomb Crawler arm (sites 175-177).
    if (513..=515).contains(&npc_type) && npc.hp <= 0 {
        tomb_crawler_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82747-82765` — the 510-512 Dune Splicer arm (sites 178-180).
    if (510..=512).contains(&npc_type) && npc.hp <= 0 {
        dune_splicer_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82766-82784` — the 621-623 Blood Eel arm (sites 181-183).
    if (621..=623).contains(&npc_type) && npc.hp <= 0 {
        blood_eel_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82785-82822` — the 518/416 Solar Spearman / Drakomire Rider arm
    // (sites 184-188).
    if (npc_type == 518 || npc_type == 416) && npc.hp <= 0 {
        solar_spearman_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82823-82861` — the 419 Solar Solenian arm (sites 189-194).
    if npc_type == 419 && npc.hp <= 0 {
        solar_solenian_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82862-82883` — the 412-414 Solar Crawltipede arm (sites 195-197).
    if (412..=414).contains(&npc_type) && npc.hp <= 0 {
        solar_crawltipede_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82884-82917` — the 415 Solar Flare arm (sites 198-201).
    if npc_type == 415 && npc.hp <= 0 {
        solar_flare_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82918-82958` — the 417 Solar Sroller arm (sites 202-209, all eight
    // gores).
    if npc_type == 417 && npc.hp <= 0 {
        solar_sroller_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82959-82993` — the 418 Solar Corite arm (sites 210-214).
    if npc_type == 418 && npc.hp <= 0 {
        solar_corite_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83039-83085` — the 426 Nebula Floater arm (sites 215-217).
    if npc_type == 426 && npc.hp <= 0 {
        nebula_floater_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:83086-83124` — the 427 Nebula Soldier arm (sites 218-221).
    if npc_type == 427 && npc.hp <= 0 {
        nebula_soldier_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83125-83206` — the 429 Nebula Predictor arm (sites 222-226).
    if npc_type == 429 && npc.hp <= 0 {
        nebula_predictor_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:83207-83246` — the 425 Nebula Beast arm (sites 227-233).
    if npc_type == 425 && npc.hp <= 0 {
        nebula_beast_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83247-83273` — the 428 Nebula Brain arm (sites 234-235).
    if npc_type == 428 && npc.hp <= 0 {
        nebula_brain_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83274-83311` — the 424 Vortex Soldier arm (sites 236-240).
    if npc_type == 424 && npc.hp <= 0 {
        vortex_soldier_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83312-83344` — the 423 Vortex Hornet arm (sites 241-243).
    if npc_type == 423 && npc.hp <= 0 {
        vortex_hornet_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83345-83377` — the 421 Vortex Larva arm (sites 244-246).
    if npc_type == 421 && npc.hp <= 0 {
        vortex_larva_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83378-83410` — the 420 Vortex Rider arm (sites 247-249).
    if npc_type == 420 && npc.hp <= 0 {
        vortex_rider_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83481-83507` — the 409 Stardust Spider arm (sites 250-252).
    if npc_type == 409 && npc.hp <= 0 {
        stardust_spider_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83532-83564` — the 407 Stardust Cell arm (sites 253-258).
    if npc_type == 407 && npc.hp <= 0 {
        stardust_cell_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83634-83663` — the 411 Stardust Dragon body arm (sites 259-263).
    if npc_type == 411 && npc.hp <= 0 {
        stardust_dragon_body_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83664-83722` — the 491 Flying Dutchman arm, COMPLETE: the HEAD block
    // (sites 264-274) + the full fan (sites 275-289) + the 855 loop (sites
    // 290-293) + the 854/853/856 loop (sites 294-302) + the full scatter
    // (sites 303-332) + the 120-dust tail + the projectile seam.
    if npc_type == 491 && npc.hp <= 0 {
        dutchman_death_head(npc, gore, rng, world);
        dutchman_death_fan(npc, gore, rng, world);
        dutchman_death_855(npc, gore, rng, world);
        dutchman_death_854(npc, gore, rng, world);
        dutchman_death_scatter(npc, gore, rng, world);
        dutchman_death_tail(npc, dust, dust_ctx, rng, world, &mut outcome);
    }
    // `:83723-83739` — the 492 Flying Dutchman cannon arm (sites 333-342).
    if npc_type == 492 && npc.hp <= 0 {
        dutchman_cannon_death(npc, dust, dust_ctx, gore, rng, world);
    }
    // `:83740-83796` — the 524-527 Desert Ghoul arm (sites 343-348).
    if (524..=527).contains(&npc_type) && npc.hp <= 0 {
        desert_ghoul_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83797-83823` — the 528-529 Desert Lamia arm (sites 349-352).
    if (npc_type == 528 || npc_type == 529) && npc.hp <= 0 {
        desert_lamia_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83824-83845` — the 530/531 Desert Scorpion arm (sites 353-357).
    if (npc_type == 530 || npc_type == 531) && npc.hp <= 0 {
        desert_scorpion_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83846-83866` — the 532 Desert Beast arm (sites 358-361).
    if npc_type == 532 && npc.hp <= 0 {
        desert_beast_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    outcome
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

    /// `NPC.cs:82442-82452` — the Dark Mage death: 11 gores at base 1009,
    /// the +10 (1019) gore precedes the +7/+8/+9 trio, the +4 gore shifted
    /// -20 in X, and the Y ladder +0/+15/+35/+25.
    #[test]
    fn w4_f10_the_dark_mage_death_spawns_the_eleven_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 64.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        dark_mage_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..11].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1009, 1010, 1011, 1012, 1013, 1014, 1015, 1019, 1016, 1017, 1018]
        );
        // The +4 gore (index 4) is shifted -20 in X at +35 Y.
        assert_eq!(gore.slot(4).position, Vec2::new(80.0, 135.0));
        // The +10 gore (index 7) sits at +35 Y with no X shift.
        assert_eq!(gore.slot(7).position, Vec2::new(100.0, 135.0));
        // The +7/+8/+9 trio (indices 8-10) sits at +25 Y.
        assert_eq!(gore.slot(8).position, Vec2::new(100.0, 125.0));
        assert_eq!(gore.slot(10).position, Vec2::new(100.0, 125.0));
    }

    /// `NPC.cs:82470-82473` — the Lightning Bug death: the 999 head at
    /// position, then 998/998/997 at +10 Y.
    #[test]
    fn w4_f10_the_lightning_bug_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 34.0, 34.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        lightning_bug_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![999, 998, 998, 997]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 110.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 110.0));
    }

    /// `NPC.cs:82496-82499` — the Goblin Shark death: base 1183, the
    /// base/+2/+3/+1 type order at +0/+20/+20/+34 Y, after the 150-dust
    /// direct burst (velocity ×2.5, scale 1.7).
    #[test]
    fn w4_f10_the_goblin_shark_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 64.0, 44.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        goblin_shark_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1183, 1185, 1186, 1184]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        // The 150-dust burst ran (the pool holds the dust); the post-writes
        // set scale 1.7 on the live slots.
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 100, "the burst spawned: {dust_active}");
    }

    /// `NPC.cs:82522-82525` — the Blood Squid death: base 1179, the
    /// base/+2/+3/+1 type order, and the dust post-write is fadeIn = 1.7
    /// (NOT scale) — the shape differs from the Goblin Shark only there.
    #[test]
    fn w4_f10_the_blood_squid_death_uses_fade_in_not_scale() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 48.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        blood_squid_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1179, 1181, 1182, 1180]);
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        // The live dust slots carry fadeIn 1.7 (the post-write); the scale
        // stays the NewDust roll x the 1.0 Scale arg (in [0.8, 1.2], never
        // the forced 1.7 the Goblin Shark arm writes).
        let live = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(live.fade_in, 1.7);
        assert!(live.scale <= 1.2, "scale stays the roll: {}", live.scale);
    }

    /// `NPC.cs:82548-82559` — the Blood Nautilus death: all twelve gores at
    /// base 1172, the first two type pairs alternate the Right.X-30 anchor
    /// with position, then +2/+3/+4/+4/+5/+5/+6/+6 at position; after the
    /// 200-dust burst (death scale 2.5).
    #[test]
    fn w4_f10_the_blood_nautilus_death_spawns_the_twelve_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 90.0, 90.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(59);
        blood_nautilus_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..12].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1172, 1172, 1173, 1173, 1174, 1175, 1176, 1176, 1177, 1177, 1178, 1178]
        );
        // Right.X = 190; the alternation anchor is (160, 100).
        assert_eq!(gore.slot(0).position, Vec2::new(160.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(160.0, 100.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 100.0));
        // The tail (indices 4-11) all sit at position.
        for i in 4..12 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
        // The 200-dust burst ran; the death post-write is scale 2.5.
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 150, "the burst spawned: {dust_active}");
        let live = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(live.scale, 2.5);
    }

    /// `NPC.cs:82600-82603` — the Sand Elemental death: the 4 gores
    /// 960/961/963/962 (963 precedes 962) at +0/+20/+20/+40 Y, after the
    /// 30x2 tinted type-268 burst (tint (155,75,33,178), noGravity, scale
    /// 1.5, fadeIn 0.7).
    #[test]
    fn w4_f10_the_sand_elemental_death_tints_the_burst() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        sand_elemental_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![960, 961, 963, 962]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 140.0));
        // The tinted burst: (222,108,48,255) * 0.7 truncates to (155,75,33,178).
        let live = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(live.color, Color::new(155, 75, 33, 178));
        assert!(live.no_gravity);
        assert_eq!(live.scale, 1.5);
        assert_eq!(live.fade_in, 0.7);
    }

    /// `NPC.cs:82650-82652` — the Tumbleweed death: the 3 gores
    /// 964/965/966 at +0/+20/+20 Y, after the 20x2 tinted type-268 burst
    /// (tint (105,75,54,178)) and the 20-dust type-54 burst (alpha 50,
    /// scale 1.5, noGravity).
    #[test]
    fn w4_f10_the_tumbleweed_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 48.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(67);
        tumbleweed_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![964, 965, 966]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        // The tinted type-268 dust: (150,108,78,255) * 0.7 = (105,75,54,178).
        let tinted = dust.slots().iter().find(|d| d.active && d.dust_type == 268).unwrap();
        assert_eq!(tinted.color, Color::new(105, 75, 54, 178));
        assert_eq!(tinted.scale, 1.5);
        // The type-54 dust carries alpha 50 and noGravity.
        let t54 = dust.slots().iter().find(|d| d.active && d.dust_type == 54).unwrap();
        assert_eq!(t54.alpha, 50);
        assert!(t54.no_gravity);
    }

    /// `NPC.cs:82676` — the Cultist Tablet burst: 6 gores 728-733 at
    /// Center-(10,10), each velocity UnitY.RotatedByRandom(2π)*5 (two
    /// NextDouble draws per gore BEFORE the kicks). The twin replay pins
    /// the 6x(2 rotation + 2 kick) draw sequence.
    #[test]
    fn w4_f10_the_cultist_tablet_burst_rotates_the_velocity() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 60.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(71);
        cultist_tablet_burst(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![728, 729, 730, 731, 732, 733]);
        // Center = (120,130); the anchor is (110,120).
        for i in 0..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(110.0, 120.0));
        }
        // The twin: per gore two NextDouble (the rotation) then the two kicks.
        let mut twin = UnifiedRandom::new(71);
        for _ in 0..6 {
            let _a = twin.next_double();
            let _b = twin.next_double();
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the tablet burst mirrors the C# draws");
    }

    /// `NPC.cs:82743-82744` — the Tomb Crawler death: one gore per segment
    /// (513/514/515 -> 822/823/824) with the alpha copy, after the 50-dust
    /// type-250 burst. The color stand-in is default (the worm kinds set no
    /// SetDefaults color).
    #[test]
    fn w4_f10_the_tomb_crawler_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(513, 822), (514, 823), (515, 824)] {
            let mut n = boss(100.0, 100.0, 22.0, 22.0);
            n.alpha = 120;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(73);
            tomb_crawler_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 120);
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            // The 50-dust type-250 burst ran.
            let dust_active = dust.slots().iter().filter(|d| d.active).count();
            assert!(dust_active >= 40, "the burst spawned: {dust_active}");
        }
    }

    /// `NPC.cs:82762-82763` — the Dune Splicer death: one gore per segment
    /// (510/511/512 -> 819/820/821) with the alpha copy.
    #[test]
    fn w4_f10_the_dune_splicer_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(510, 819), (511, 820), (512, 821)] {
            let mut n = boss(100.0, 100.0, 34.0, 34.0);
            n.alpha = 0;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(79);
            dune_splicer_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 0);
        }
    }

    /// `NPC.cs:82781-82782` — the Blood Eel death: one gore per segment
    /// (621/622/623 -> 1187/1188/1189) with the alpha copy (the family sets
    /// alpha 255), after the 50-dust type-5 burst.
    #[test]
    fn w4_f10_the_blood_eel_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(621, 1187), (622, 1188), (623, 1189)] {
            let mut n = boss(100.0, 100.0, 28.0, 28.0);
            n.alpha = 255;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(83);
            blood_eel_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 255);
            // The dust is type 5 (not 250).
            let d5 = dust.slots().iter().find(|d| d.active && d.dust_type == 5);
            assert!(d5.is_some(), "the type-5 burst ran");
        }
    }

    /// `NPC.cs:82816-82820` — the Solar Spearman death: the 5 gores
    /// 844/845/847/835/836 at +0/+20 Y, after the 20-iteration type-241 +
    /// Next(3)-gated type-6 burst. The twin replay pins the gate draws.
    #[test]
    fn w4_f10_the_solar_spearman_death_gates_the_type_six_dust() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(89);
        solar_spearman_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![844, 845, 847, 835, 836]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        // The twin: 20 x (type-241 6-draw census + Next(3) gate; a taken gate
        // adds the type-6 census + its tweak draw), then the 5 gores' kicks.
        let mut twin = UnifiedRandom::new(89);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(36);
            let _py = r.next_max(52);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            census(&mut twin); // type 241 (no tweak)
            if twin.next_max(3) == 0 {
                census(&mut twin); // type 6 default block
                let _tweak = twin.next_range(-10, 6); // type-6 tweak (Dust.cs:216)
            }
        }
        for _ in 0..5 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the spearman death mirrors the C# draws");
    }

    /// `NPC.cs:82854-82859` — the Solar Solenian death: the 6 gores
    /// 844/845/846/846/847/847 at +0/+20 Y (the 846/847 pairs).
    #[test]
    fn w4_f10_the_solar_solenian_death_spawns_the_six_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 44.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        solar_solenian_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![844, 845, 846, 846, 847, 847]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        for i in 1..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 120.0));
        }
    }

    /// `NPC.cs:82881` — the Solar Crawltipede death: one gore per segment
    /// (412/413/414 -> 828/829/830), no alpha copy, after the 6-dust
    /// type-241 + 2-dust type-6 (alpha 100, transparent, scale 2) bursts.
    #[test]
    fn w4_f10_the_solar_crawltipede_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(412, 828), (413, 829), (414, 830)] {
            let n = boss(100.0, 100.0, 30.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(101);
            solar_crawltipede_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 0); // no alpha copy
            // The type-6 dust carries alpha 100 + transparent + the 2x scale.
            let d6 = dust.slots().iter().find(|d| d.active && d.dust_type == 6).unwrap();
            assert_eq!(d6.alpha, 100);
            assert_eq!(d6.color, Color::TRANSPARENT);
        }
    }

    /// `NPC.cs:82912-82915` — the Solar Flare death: the 4 gores
    /// 831/832/833/834 at Center, velocity x0.8 (831/832) and x0.9
    /// (833/834), after the 40x(type-241 + ungated type-6) burst.
    #[test]
    fn w4_f10_the_solar_flare_death_scales_the_velocity() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.vx = 4.0;
        n.vy = 2.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        solar_flare_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![831, 832, 833, 834]);
        // Center = (120,120).
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(120.0, 120.0));
        }
        // The velocity is x0.8 / x0.9 plus the NewGore kicks (the Y kick
        // lifts, so check the X term: 4*0.8=3.2 +/- the 0.1-step kick).
        assert!((gore.slot(0).velocity.x - 3.2).abs() <= 2.0);
        assert!((gore.slot(2).velocity.x - 3.6).abs() <= 2.0);
    }

    /// `NPC.cs:82949-82956` — the Solar Sroller death: all 8 gores
    /// 837/838x3/839x2/840x2 at +0/+20/+38 Y, after the 30x(type-241 +
    /// Next(2)-gated type-6) burst.
    #[test]
    fn w4_f10_the_solar_sroller_death_spawns_the_eight_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 46.0, 46.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(107);
        solar_sroller_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..8].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![837, 838, 838, 838, 839, 839, 840, 840]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        for i in 1..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 120.0));
        }
        // The 840/840 tail sits at +38 Y.
        assert_eq!(gore.slot(6).position, Vec2::new(100.0, 138.0));
        assert_eq!(gore.slot(7).position, Vec2::new(100.0, 138.0));
    }

    /// `NPC.cs:82987-82991` — the Solar Corite death: the 5 gores
    /// 841/842/842/843/843 at Center, velocity x0.8 (841/842/842) and x0.9
    /// (843/843), after the 20x(type-241 + ungated type-6) burst.
    #[test]
    fn w4_f10_the_solar_corite_death_spawns_the_five_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 36.0, 36.0);
        n.vx = 3.0;
        n.vy = -2.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(109);
        solar_corite_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![841, 842, 842, 843, 843]);
        // Center = (118,118).
        for i in 0..5 {
            assert_eq!(gore.slot(i).position, Vec2::new(118.0, 118.0));
        }
        // The velocity x0.8 / x0.9 plus the kicks.
        assert!((gore.slot(0).velocity.x - 2.4).abs() <= 2.0);
        assert!((gore.slot(3).velocity.x - 2.7).abs() <= 2.0);
    }

    /// `NPC.cs:83072-83074` — the Nebula Floater death: the 3 gores
    /// 799/800/801 at Top/Center/Center, velocity x0.7/x0.8/x0.9, after the
    /// 20-dust type-241 + 30-dust SelectRandom(229,240,161) bursts; the
    /// brain-spawn seam is returned.
    #[test]
    fn w4_f10_the_nebula_floater_death_returns_the_brain_seam() {
        let w = air();
        let mut n = boss(100.0, 100.0, 44.0, 44.0);
        n.vx = 2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(113);
        let mut out = HitEffectOutcomeC::default();
        nebula_floater_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![799, 800, 801]);
        // Top = (122,100); Center = (122,122).
        assert_eq!(gore.slot(0).position, Vec2::new(122.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(122.0, 122.0));
        assert_eq!(gore.slot(2).position, Vec2::new(122.0, 122.0));
        // The brain-spawn seam carries the Center.
        assert_eq!(out.nebula_brain_spawn, Some((122.0, 122.0)));
        // The twin: 20 type-241 dusts, then 30 x (SelectRandom Next(3) +
        // 6-draw census + NextFloat scale), then the 3 gores' kicks.
        let mut twin = UnifiedRandom::new(113);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(40);
            let _py = r.next_max(40);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            census(&mut twin);
        }
        for _ in 0..30 {
            let _sel = twin.next_max(3);
            census(&mut twin);
            let _sf = twin.next_float();
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the floater death mirrors the C# draws");
    }

    /// `NPC.cs:83119-83122` — the Nebula Soldier death: the 4 gores
    /// 802/803/804/805 at Center, velocity x0.8/x0.9/x0.9/x0.8, after the
    /// 10-dust type-241 + 20-dust SelectRandom(229,229,240) bursts.
    #[test]
    fn w4_f10_the_nebula_soldier_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 52.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        nebula_soldier_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![802, 803, 804, 805]);
        // Center = (120,126).
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(120.0, 126.0));
        }
    }

    /// `NPC.cs:83156-83160` — the Nebula Predictor death: the 5 gores
    /// 808/809/809/810/810 at +0/+20/+34 Y, after the 20x(type-241 +
    /// Next(3)-gated type-229) burst; the projectile-spawn seam is returned.
    #[test]
    fn w4_f10_the_nebula_predictor_death_returns_the_shot_seam() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let mut out = HitEffectOutcomeC::default();
        nebula_predictor_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![808, 809, 809, 810, 810]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        // The shot seam carries the Center = (124,130).
        assert_eq!(out.nebula_predictor_shot, Some((124.0, 130.0)));
    }

    /// `NPC.cs:83238-83244` — the Nebula Beast death: the 7 gores
    /// 796/797/798/809/809/810/810 (796 at position, 797 at Center, the
    /// 798/809 pair at +20, the 810 pair at +34).
    #[test]
    fn w4_f10_the_nebula_beast_death_spawns_the_seven_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(137);
        nebula_beast_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![796, 797, 798, 809, 809, 810, 810]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // Center = (125,125).
        assert_eq!(gore.slot(1).position, Vec2::new(125.0, 125.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(5).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83270-83271` — the Nebula Brain death: the 2 gores 806/807
    /// at position, velocity x0.8, after the 10x(type-241 + Next(2)-picked
    /// 229/240) burst. The twin replay pins the gate draws (both branches
    /// spawn a dust).
    #[test]
    fn w4_f10_the_nebula_brain_death_picks_the_dust_type() {
        let w = air();
        let mut n = boss(100.0, 100.0, 36.0, 36.0);
        n.vx = 5.0;
        n.vy = 0.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(139);
        nebula_brain_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![806, 807]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // The velocity x0.8 plus the kicks.
        assert!((gore.slot(0).velocity.x - 4.0).abs() <= 2.0);
        // The twin: 10 x (type-241 census + Next(2) gate + the picked dust's
        // census), then the 2 gores' kicks.
        let mut twin = UnifiedRandom::new(139);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(32);
            let _py = r.next_max(32);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..10 {
            census(&mut twin); // type 241
            let _gate = twin.next_max(2);
            census(&mut twin); // the picked 229/240 (neither hits a tweak)
        }
        for _ in 0..2 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the brain death mirrors the C# draws");
    }

    /// `NPC.cs:83305-83309` — the Vortex Soldier death: the 5 gores
    /// 793/794/794/795/795 at +0/+20/+34 Y, after the 20x(type-249 +
    /// Next(3)-gated type-242) burst.
    #[test]
    fn w4_f10_the_vortex_soldier_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 52.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(149);
        vortex_soldier_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![793, 794, 794, 795, 795]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83340-83342` — the Vortex Hornet death: the 3 gores
    /// 790/791/792 at Center, velocity x0.8/x0.8/x0.9, after the 20x
    /// UNGATED (type-249 + type-242) burst. The twin replay pins the
    /// ungated census (no gate draw).
    #[test]
    fn w4_f10_the_vortex_hornet_death_is_ungated() {
        let w = air();
        let mut n = boss(100.0, 100.0, 38.0, 38.0);
        n.vx = 2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(151);
        vortex_hornet_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![790, 791, 792]);
        // Center = (119,119).
        for i in 0..3 {
            assert_eq!(gore.slot(i).position, Vec2::new(119.0, 119.0));
        }
        // The twin: 20 x (type-249 census + type-242 census + its tweak
        // draw), no gate; then the 3 gores' kicks.
        let mut twin = UnifiedRandom::new(151);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(34);
            let _py = r.next_max(34);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            census(&mut twin); // type 249 (no tweak)
            census(&mut twin); // type 242 default block
            let _tweak = twin.next_range(-10, 6); // type-242 tweak (Dust.cs:216)
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the hornet death mirrors the C# draws");
    }

    /// `NPC.cs:83373-83375` / `:83406-83408` — the Vortex Larva / Rider
    /// deaths: 3 gores each at Center, velocity x0.8/x0.8/x0.9, after the
    /// 10x ungated (type-249 + type-242) burst. The types differ per arm.
    #[test]
    fn w4_f10_the_vortex_larva_and_rider_deaths_spawn_their_gores() {
        let w = air();
        for (ty, gores, f) in [
            (421, [785, 786, 787], vortex_larva_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32)),
            (420, [782, 783, 784], vortex_rider_death),
        ] {
            let n = boss(100.0, 100.0, 40.0, 40.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(157);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "kind {ty}");
            // Center = (120,120).
            for i in 0..3 {
                assert_eq!(gore.slot(i).position, Vec2::new(120.0, 120.0));
            }
        }
    }

    /// `NPC.cs:83503-83505` — the Stardust Spider death: the 3 gores
    /// 775/776/777 at Center, velocity x0.8/x0.8/x0.9, after the 20x
    /// ungated (type-17 + type-229) burst.
    #[test]
    fn w4_f10_the_stardust_spider_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(163);
        stardust_spider_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![775, 776, 777]);
        // Center = (120,115).
        for i in 0..3 {
            assert_eq!(gore.slot(i).position, Vec2::new(120.0, 115.0));
        }
    }

    /// `NPC.cs:83557-83562` — the Stardust Cell death: the 6 gores
    /// 778/779 at Top (vel x0.8) and 780/781/780/781 at Center (vel x0.9),
    /// after the 20x(Next(3)!=0-gated type-17 + ungated type-229) burst. The
    /// twin replay pins the INVERSE gate (the draw fires even when skipped).
    #[test]
    fn w4_f10_the_stardust_cell_death_uses_the_inverse_gate() {
        let w = air();
        let n = boss(100.0, 100.0, 36.0, 36.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(167);
        stardust_cell_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![778, 779, 780, 781, 780, 781]);
        // Top = (118,100); Center = (118,118).
        assert_eq!(gore.slot(0).position, Vec2::new(118.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(118.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(118.0, 118.0));
        // The twin: 20 x (Next(3) gate; a taken gate adds the type-17
        // census; then the type-229 census), then the 6 gores' kicks.
        let mut twin = UnifiedRandom::new(167);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(32);
            let _py = r.next_max(32);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            if twin.next_max(3) != 0 {
                census(&mut twin); // type 17 (no tweak)
            }
            census(&mut twin); // type 229 (no tweak)
        }
        for _ in 0..6 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the cell death mirrors the C# draws");
    }

    /// `NPC.cs:83657-83661` — the Stardust Dragon body death: the 5 gores
    /// 772/773/773/774/774 at +0/+20/+34 Y, after the 20x(type-17 +
    /// Next(2)-gated type-229) burst.
    #[test]
    fn w4_f10_the_stardust_dragon_body_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(173);
        stardust_dragon_body_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![772, 773, 773, 774, 774]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83670-83676` — the Dutchman death head block: the 858/857
    /// gores at the integer width/2 and width/3 anchors, then the 3x3 loop
    /// of 859 gores. The facing unit flips on spriteDirection == 1.
    #[test]
    fn w4_f10_the_dutchman_death_head_spawns_the_eleven_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1; // dir = +1
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(179);
        dutchman_death_head(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..11].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![858, 857, 859, 859, 859, 859, 859, 859, 859, 859, 859]
        );
        // Center = (275,160); v15 = (215,100); v16 = (1,0).
        // width/2 = 175 -> (390,100); width/3 = 116 -> (331,100).
        assert_eq!(gore.slot(0).position, Vec2::new(390.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(331.0, 100.0));
        // The num183=-1 first 859: width*(0.45-0.1)=350*0.35=122.5 -> (337.5,100).
        assert_eq!(gore.slot(2).position, Vec2::new(215.0 + 350.0 * 0.35, 100.0));
    }

    /// `NPC.cs:83669` — the facing unit flips when spriteDirection == 1
    /// (dir = -1), mirroring the head anchors across the anchor point.
    #[test]
    fn w4_f10_the_dutchman_death_head_flips_on_sprite_direction() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = 1; // dir = -1
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(181);
        dutchman_death_head(&n, &mut gore, &mut rng, &w);
        // v16 = (-1,0); width/2 = 175 -> (215-175,100) = (40,100).
        assert_eq!(gore.slot(0).position, Vec2::new(40.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(215.0 - 116.0, 100.0));
    }

    /// `NPC.cs:83680-83684` — the Dutchman fan (all three iterations): 15
    /// gores of type SelectRandom(860,892,893), each drawing RandomVector2
    /// (two NextDouble) THEN SelectRandom (Next(3)) THEN the kicks. The twin
    /// replay pins that per-gore order.
    #[test]
    fn w4_f10_the_dutchman_fan_draws_position_then_type() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1;
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(191);
        dutchman_death_fan(&n, &mut gore, &mut rng, &w);
        // 3 iterations x 5 gores = 15.
        let active = gore.slots()[..15].iter().filter(|g| g.active).count();
        assert_eq!(active, 15);
        for g in &gore.slots()[..15] {
            assert!(matches!(g.gore_type, 860 | 892 | 893), "fan type {}", g.gore_type);
        }
        // The twin: per gore two NextDouble (RandomVector2) + Next(3)
        // (SelectRandom) + the two kicks.
        let mut twin = UnifiedRandom::new(191);
        for _ in 0..15 {
            let _rx = twin.next_double();
            let _ry = twin.next_double();
            let _sel = twin.next_max(3);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the fan mirrors the C# argument order");
    }

    /// `NPC.cs:83688-83689` — the Dutchman 855 loop: 4 gores of type 855 at
    /// width*±0.15 with drops 160 and 220 (160 + 60*num185).
    #[test]
    fn w4_f10_the_dutchman_855_loop_spawns_the_four_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1;
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(193);
        dutchman_death_855(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![855, 855, 855, 855]);
        // v15 = (215,100); width*0.15 = 52.5. num185=0 drop 160 -> y=-60.
        assert_eq!(gore.slot(0).position, Vec2::new(215.0 + 52.5, 100.0 - 160.0));
        assert_eq!(gore.slot(1).position, Vec2::new(215.0 - 52.5, 100.0 - 160.0));
        // num185=1 drop 220 -> y=-120.
        assert_eq!(gore.slot(2).position, Vec2::new(215.0 + 52.5, 100.0 - 220.0));
        assert_eq!(gore.slot(3).position, Vec2::new(215.0 - 52.5, 100.0 - 220.0));
    }

    /// `NPC.cs:83693-83695` — the Dutchman 854/853/856 loop: 9 gores, the
    /// per-iteration type order 854/853/856 at the width-fraction anchors
    /// with drops 20 + 40*num186 (-20/20/60).
    #[test]
    fn w4_f10_the_dutchman_854_loop_spawns_the_nine_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1;
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(197);
        dutchman_death_854(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..9].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![854, 853, 856, 854, 853, 856, 854, 853, 856]);
        // v15 = (215,100). num186=-1: 854 at width*0.15=52.5, drop -20 -> y=120.
        assert_eq!(gore.slot(0).position, Vec2::new(215.0 + 350.0 * 0.15, 120.0));
        // 853 at width*(-0.55)=-192.5, drop -20.
        assert_eq!(gore.slot(1).position, Vec2::new(215.0 - 350.0 * 0.55, 120.0));
        // num186=0: 854 at width*0.25=87.5, drop 20 -> y=80.
        assert_eq!(gore.slot(3).position, Vec2::new(215.0 + 350.0 * 0.25, 80.0));
        // num186=1: 856 at width*0.15=52.5, drop 60 -> y=40.
        assert_eq!(gore.slot(8).position, Vec2::new(215.0 + 350.0 * 0.15, 40.0));
    }

    /// `NPC.cs:83699-83702` — the Dutchman scatter (all 30 gores): 30 gores
    /// of type Next(861,863), alpha 100, velocity (4cosθ,4sinθ) from one
    /// NextDouble, position re-rolled with two NextFloat. The twin replay
    /// pins the per-gore order: type Next(861,863) + 2 kicks + NextDouble +
    /// 2 NextFloat.
    #[test]
    fn w4_f10_the_dutchman_scatter_draws_type_kicks_then_post() {
        let w = air();
        let n = boss(100.0, 100.0, 350.0, 120.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(199);
        dutchman_death_scatter(&n, &mut gore, &mut rng, &w);
        let active = gore.slots()[..30].iter().filter(|g| g.active).count();
        assert_eq!(active, 30);
        for g in &gore.slots()[..30] {
            assert!(matches!(g.gore_type, 861 | 862), "scatter type {}", g.gore_type);
            assert_eq!(g.alpha, 100);
            // The velocity magnitude is 4 (the (4,0) rotation).
            assert!((g.velocity.length() - 4.0).abs() < 0.01);
            // sticky=false for 861/862 (Gore.cs:1341-1343).
            assert!(!g.sticky);
        }
        // The twin: per gore Next(861,863) + 2 kicks + NextDouble + 2 NextFloat.
        let mut twin = UnifiedRandom::new(199);
        for _ in 0..30 {
            let _ty = twin.next_range(861, 863);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _theta = twin.next_double();
            let _px = twin.next_float();
            let _py = twin.next_float();
        }
        assert_eq!(rng.peek(), twin.peek(), "the scatter mirrors the C# draw order");
    }

    /// `NPC.cs:83704-83712` — the Dutchman tail: the 120x2 type-6 dust
    /// burst (alpha 100, scales 2.5/1.5, velocity x5/x4, noGravity) then the
    /// projectile seam returned. The twin replay pins the 240 dust censuses
    /// (type 6 hits the tweak draw, Dust.cs:216).
    #[test]
    fn w4_f10_the_dutchman_tail_runs_the_dust_burst_and_returns_the_seam() {
        let w = air();
        let n = boss(100.0, 100.0, 350.0, 120.0);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(211);
        let mut out = HitEffectOutcomeC::default();
        dutchman_death_tail(&n, &mut dust, &ctx(), &mut rng, &w, &mut out);
        // Center = (275,160).
        assert_eq!(out.dutchman_death_shots, Some((275.0, 160.0)));
        // The twin: 240 type-6 dusts, each the 6-draw census + the tweak draw.
        let mut twin = UnifiedRandom::new(211);
        for _ in 0..240 {
            let _px = twin.next_max(346);
            let _py = twin.next_max(116);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _tweak = twin.next_range(-10, 6); // type-6 tweak
        }
        assert_eq!(rng.peek(), twin.peek(), "the tail mirrors the C# dust draws");
    }

    /// `NPC.cs:83734-83736` — the Dutchman cannon death: 10 gores of type
    /// Next(61,64), alpha 100, velocity (cosθ,sinθ) from one NextDouble,
    /// after the 7-dust type-31 burst. The twin replay pins the per-gore
    /// order: type Next(61,64) + 2 kicks + NextDouble.
    #[test]
    fn w4_f10_the_dutchman_cannon_death_scatters_the_ten_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(223);
        dutchman_cannon_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w);
        let active = gore.slots()[..10].iter().filter(|g| g.active).count();
        assert_eq!(active, 10);
        for g in &gore.slots()[..10] {
            assert!(matches!(g.gore_type, 61 | 62 | 63), "cannon type {}", g.gore_type);
            assert_eq!(g.alpha, 100);
            assert!((g.velocity.length() - 1.0).abs() < 0.01);
            assert!(!g.sticky); // 61/62/63 sticky=false
        }
        // The twin: 7 type-31 dusts (6-draw census, no tweak), then per gore
        // Next(61,64) + 2 kicks + NextDouble.
        let mut twin = UnifiedRandom::new(223);
        for _ in 0..7 {
            let _px = twin.next_max(26);
            let _py = twin.next_max(26);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..10 {
            let _ty = twin.next_range(61, 64);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _theta = twin.next_double();
        }
        assert_eq!(rng.peek(), twin.peek(), "the cannon death mirrors the C# draw order");
    }

    /// `NPC.cs:83789-83794` — the Desert Ghoul death: the per-tier gore base
    /// (524→863, 525→867, 526→871, 527→879) with the base/+1/+2/+2/+3/+3
    /// order. The accent-dust gate draws Next(3) even for 524 (num196==0).
    #[test]
    fn w4_f10_the_desert_ghoul_death_uses_the_per_tier_base() {
        let w = air();
        for (ty, base) in [(524, 863), (525, 867), (526, 871), (527, 879)] {
            let n = boss(100.0, 100.0, 40.0, 50.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(227);
            desert_ghoul_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base, base + 1, base + 2, base + 2, base + 3, base + 3],
                "ghoul {ty}"
            );
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(4).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:83780` — the ghoul accent gate draws Next(3) BEFORE the
    /// num196>0 check, so the 524 arm (num196==0) still advances the stream
    /// 20 times. The twin replay pins that for both 524 and 525.
    #[test]
    fn w4_f10_the_desert_ghoul_gate_draws_even_when_accent_is_zero() {
        let w = air();
        for ty in [524, 525] {
            let n = boss(100.0, 100.0, 40.0, 50.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(229);
            desert_ghoul_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let mut twin = UnifiedRandom::new(229);
            let census = |r: &mut UnifiedRandom| {
                let _px = r.next_max(36);
                let _py = r.next_max(46);
                let _vx = r.next_range(-20, 21);
                let _vy = r.next_range(-20, 21);
                let _row = r.next_max(3);
                let _scale = r.next_range(-20, 21);
            };
            for _ in 0..20 {
                census(&mut twin); // type 5
                if twin.next_max(3) == 0 && ty != 524 {
                    census(&mut twin); // the accent dust (no tweak for 75/170/67)
                }
            }
            for _ in 0..6 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "ghoul {ty} mirrors the C# draws");
        }
    }

    /// `NPC.cs:83818-83821` — the Desert Lamia death: the per-tier head
    /// (528→875, 529→876) then the FIXED 877/877/878 tail.
    #[test]
    fn w4_f10_the_desert_lamia_death_uses_the_per_tier_head() {
        let w = air();
        for (ty, head) in [(528, 875), (529, 876)] {
            let n = boss(100.0, 100.0, 40.0, 60.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(233);
            desert_lamia_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![head, 877, 877, 878], "lamia {ty}");
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:83839-83843` — the Desert Scorpion death: the 5 gores
    /// 883-887 at position, after the 20-dust type-250 alpha-100 burst.
    #[test]
    fn w4_f10_the_desert_scorpion_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(239);
        desert_scorpion_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![883, 884, 885, 886, 887]);
        for i in 0..5 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
        // The dust carries alpha 100.
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(d.alpha, 100);
    }

    /// `NPC.cs:83861-83864` — the Desert Beast death: the 4 gores 888-891
    /// at position, after the 50-dust type-5 alpha-100 burst.
    #[test]
    fn w4_f10_the_desert_beast_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(241);
        desert_beast_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![888, 889, 890, 891]);
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
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
        hit_effect_c(&n, 551, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert_eq!(gore.slots().iter().filter(|g| g.active).count(), 10);
        // Non-flipped -> nothing here (wave B's branch).
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut n2 = boss(100.0, 100.0, 232.0, 156.0);
        n2.sprite_dir = 1;
        hit_effect_c(&n2, 551, 1.0, 10.0, &mut gore2, &mut dust2, &ctx(), &mut rng, &w);
        assert_eq!(gore2.slots().iter().filter(|g| g.active).count(), 0);
        // Living -> nothing (the hit half is dust-only, the wiring wave's).
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut n3 = boss(100.0, 100.0, 232.0, 156.0);
        n3.hp = 50;
        hit_effect_c(&n3, 551, 1.0, 10.0, &mut gore3, &mut dust3, &ctx(), &mut rng, &w);
        assert_eq!(gore3.slots().iter().filter(|g| g.active).count(), 0);
    }
}
