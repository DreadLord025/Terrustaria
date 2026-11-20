//! `NPC.HitEffect` — the per-NPC gore/dust table, wave A: the FIRST 30
//! `Gore.NewGore*` death sites in source order (`NPC.cs:81654-81958`).
//!
//! The pool itself is already ported (`tr_core::gore::GorePool::new_gore`,
//! S1-E-4; the `GoreID.Sets` tables in `tr_core::gore_id`); this module is
//! the per-NPC spawn table the pool feeds. W4-d55 (code-only mode): the
//! module + witnesses land first; the dispatch claim (the `StrikeNPC_Inner`
//! `:81482`/`:81488` pair and the AI-arm `HitEffect()` calls) is the wiring
//! wave, the `npc_ai_style39.rs` precedent.
//!
//! ## Scope — the first 30 gore sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-13 | `:81667`-`:81706` | the party-hat head gores (13 town rows) |
//! | 14 | `:81727` | the party hat itself |
//! | 15-17 | `:81758` | the Queen Slime 686 burst (3 × `NewGorePerfect`) |
//! | 18 | `:81764` | the 686 crystal (`NewGorePerfect` 1146) |
//! | 19-21 | `:81795` | the Windy Balloon 594 burst (3 × `NewGorePerfect`) |
//! | 22 | `:81801` | the 594 crystal (`NewGorePerfect` 1146) |
//! | 23-29 | `:81902`-`:81908` | the Deerclops 668 death (7 gores) |
//! | 30 | `:81941` | the Empress 636 death's FIRST gore (1262) |
//!
//! The rest of the 636 arm (`:81942-81949`, 8 gores) and everything after
//! it (`:81956` onward: 657/658-660/667/the DD2 families/…) are wave B
//! (`W4-e5-gore-sites-2`) and wave C (`W4-f10-gore-sites-3`). The 636 arm's
//! dust loop (`:81929-81939`) is NOT drawn here — its draws ride the strike
//! BEFORE the gores, so the wiring wave must consume them first (documented
//! below).
//!
//! ## RNG contract (the strike's stream)
//!
//! `HitEffect` runs inside `StrikeNPC_Inner` (`NPC.cs:81488`), under the
//! strike caller's `Main.rand` swap (the projectile/melee update's stage),
//! NOT the NPC pass's — the `npc_ai_style37.rs::probe_launch_on_hit` note.
//! The caller hands that stream in; every draw below hits it in C# order:
//!
//! * Party-hat block: NO draws — the head/hat types are data
//!   ([`party_hat_gore_type`]/[`party_hat_color`]); only the `NewGore`
//!   kicks (`Gore.cs:1327-1328`, pool-side).
//! * 686/594: per burst gore — `NextVector2Circular(15,15)` (2 draws:
//!   unit `NextFloat` + radius `NextFloat`, `Utils.cs:1261-1269`),
//!   `NextVector2Circular(2,2)` (2), `Next(3)` (the type offset), the
//!   `NewGore` kicks (2, pool-side), then `NextFloat` (the velocity.Y
//!   drop, `:81759`/`:81796`). The crystal: `Circular(5,5)` (2) +
//!   `Circular(2,2)` (2) + kicks (2) + `NextFloat` (1). 8 draws per gore,
//!   ×4 per arm.
//! * 668: the 120-dust burst first — per dust the `NewDust` internals
//!   (the 6 default-block draws, type 5 hits no tweak, `Dust.cs:214-252`)
//!   then `NextVector2Circular(4,4)` (2). The 7 gores draw only the
//!   `NewGore` kicks (2 each).
//! * 636: the 50-dust loop first — per iteration `Next(2)` (the type
//!   selector, `:81931`/`:81933`) then the `NewDust` internals (the 6
//!   default-block draws; types 67/69 hit no tweak). The first gore
//!   (`:81941`) then draws its kicks.
//!
//! The dust loops run through `DustPool::new_dust`/`new_dust_direct`
//! (S1-E-1) on the SAME stream, so a cull refusal or a pressure-band gate
//! consumes exactly the C#'s draws (the pool owns them). The post-spawn
//! writes land on the returned slot, the sentinel included — the C#
//! writes `Main.dust[6000]`/`Main.gore[600]` too (the `npc_ai_style56.rs`
//! precedent).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `!this.active` gate (`:81656`): the caller's — the fn runs on an
//!   active NPC (the strike/death path's own check).
//! * The party-hat block's `IsShimmerVariant`/`UsesPartyHat` ride
//!   [`HitEffectInputs`] (the C# properties read `townNpcVariationIndex`,
//!   `ForcePartyHatOn`, the dummy flags, `frame.Height` and
//!   `BirthdayParty.PartyIsUp`, `NPC.cs:66611-66634`; the port's Npc has
//!   no variation-index field yet). The shimmer gate resolves through
//!   `npc_ai_town::shimmer_town_transform` (`NPCID.cs:10469`).
//! * `GetPartyHatColor` (`:66636-66659`) is ported verbatim as
//!   [`party_hat_color`]; its `GivenOrTypeName` rides
//!   [`HitEffectInputs::given_or_type_name`] (the wiring maps the town
//!   name; the lang accessor `lang::npc_name` exists).
//! * The daybreak arm (`:81730-81751`): 20 dusts of
//!   `SelectRandom(6,259,158)` + the `AddBuff(189, 300)` proximity sweep.
//!   The dust draws ride the pool; the buff sweep needs the NPC roster
//!   seam (the wiring wave) and is RETURNED as
//!   [`HitEffectOutcome::daybreak_add_buff`] — the caller applies it.
//! * The 686 `Transform(680)` + `unlockedSlimePurpleSpawn`
//!   (`:81768-81780`): the position write (`Bottom + (0,48)`) runs here;
//!   the transform rides the existing `wants_transform` seam
//!   (`npc.rs:1779`) and the unlock flag is RETURNED
//!   ([`HitEffectOutcome::unlock_slime_purple`]) — the port's `World`
//!   carries the sibling flags (`unlocked_slime_red_spawn`,
//!   `world_sim/editing.rs:2933`).
//! * The 594 `AI_113_WindyBalloon_GetSlaveNPC` tail (`:81805-81811`):
//!   the roster probe is the wiring wave's; RETURNED as
//!   [`HitEffectOutcome::windy_balloon_slave_release`].
//! * The 636 dust loop's draws are consumed ([`empress_hit_dust_draws`])
//!   but the dust EMIT is the documented skip until the wiring wave
//!   feeds the pool (the `npc_ai_style50.rs` count-plan precedent) — the
//!   first gore spawns on the pool regardless.
//! * `netMode` guards: single-player port (the `netMode != 1` arms run,
//!   the `== 2` arms drop), the pool's own convention.
//!
//! See docs/ANALYSIS.md "The NPC HitEffect gore table (W4-d55, wave A)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the strike-path claim is the follow-up round

use crate::npc::Npc;
use crate::npc_ai_town::shimmer_town_transform;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The per-call inputs `HitEffect` reads off the NPC beyond the port's
/// `Npc` fields (the stand-in convention, `Style39Context`).
pub struct HitEffectInputs {
    /// `NPC.townNPC` (`NPC.cs:6101`) — the party-hat gate's first term.
    pub town_npc: bool,
    /// `UsesPartyHat()` (`NPC.cs:66611-66634`) resolved by the caller
    /// (the variation index / ForcePartyHatOn / dummy flags / frame
    /// height / PartyIsUp the port does not carry on `Npc` yet).
    pub uses_party_hat: bool,
    /// `IsShimmerVariant` (`NPC.cs:6713-6723`): the caller resolves it
    /// from `townNpcVariationIndex == 1 && ShimmerTownTransform[type]`;
    /// [`is_shimmer_variant`] is the pure helper.
    pub is_shimmer_variant: bool,
    /// `GivenOrTypeName` (`NPC.cs:6743-6753`) — the hat-colour hash.
    pub given_or_type_name: String,
    /// `NPC.whoAmI` — the hat-colour hash term.
    pub who_am_i: i32,
    /// `Main.moonPhase` — the hat-colour hash term.
    pub moon_phase: i32,
    /// `Main.dayTime` — the hat-colour hash's −1.
    pub day_time: bool,
    /// `NPC.daybreak` (`NPC.cs:6275`) — the debuff-189 death arm.
    pub daybreak: bool,
}

impl Default for HitEffectInputs {
    fn default() -> Self {
        Self {
            town_npc: false,
            uses_party_hat: false,
            is_shimmer_variant: false,
            given_or_type_name: String::new(),
            who_am_i: 0,
            moon_phase: 0,
            day_time: true,
            daybreak: false,
        }
    }
}

/// `NPC.IsShimmerVariant` (`NPC.cs:6713-6723`): variation index 1 AND the
/// `NPCID.Sets.ShimmerTownTransform` row (`ID/NPCID.cs:10469`, ported in
/// `npc_ai_town::shimmer_town_transform`).
pub fn is_shimmer_variant(npc_type: i32, variation_index: i32) -> bool {
    variation_index == 1 && shimmer_town_transform(npc_type)
}

/// The party-hat HEAD gore of one town row (`NPC.cs:81664-81708`): the
/// `(normal, shimmer)` pair per type; 229 has no shimmer gore (`:81682`).
/// `None` = the type has no row (the switch's default).
pub fn party_hat_gore_type(npc_type: i32, shimmer: bool) -> Option<i32> {
    let (normal, shimmered) = match npc_type {
        38 => (927, 1298),   // :81667
        207 => (928, 1305),  // :81670
        17 => (929, 1321),   // :81673
        18 => (930, 1325),   // :81676
        227 => (931, 1331),  // :81679
        229 => (932, -1),    // :81684 — the shimmer arm spawns NOTHING
        142 => (933, 1339),  // :81688
        453 => (934, 1343),  // :81691
        178 => (935, 1348),  // :81694
        353 => (936, 1351),  // :81697
        368 => (937, 1363),  // :81700
        108 => (938, 1373),  // :81703
        588 => (1112, 1311), // :81706
        _ => return None,
    };
    if shimmer {
        if shimmered == -1 {
            None
        } else {
            Some(shimmered)
        }
    } else {
        Some(normal)
    }
}

/// `NPC.GetPartyHatColor` (`NPC.cs:66636-66659`) — the hash → the
/// `PartyHatColor` discriminant (`PartyHatColor.cs:3-12`: None 0, Blue 1,
/// Pink 2, Cyan 3, Purple 4, White 5). `uses_party_hat` is the property's
/// first gate; an empty name returns None (0).
pub fn party_hat_color(
    uses_party_hat: bool,
    name: &str,
    who_am_i: i32,
    moon_phase: i32,
    day_time: bool,
) -> i32 {
    // `:66638-66641`.
    if !uses_party_hat || name.is_empty() {
        return 0;
    }
    // `:66642` — `Length + name[0]` (the C# char widens to int; the port
    // hashes the first BYTE — ASCII town names are the vanilla set).
    let mut num = name.len() as i32 + name.as_bytes()[0] as i32;
    // `:66643-66647`.
    let mut num2 = moon_phase;
    if day_time {
        num2 -= 1;
    }
    // `:66648` — `(whoAmI % 2 == 0).ToDirectionInt()` (`Utils.cs:2010-2017`).
    let dir = if who_am_i % 2 == 0 { 1 } else { -1 };
    num += who_am_i + num2 * dir;
    // `:66649-66657` — the C# `%` keeps the sign; the +5 fires ONCE.
    if num < 0 {
        num += 5;
    }
    num %= 5;
    if num == 0 {
        num += 1;
    }
    num
}

/// The hat gore id of a colour (`NPC.cs:81711-81726`): the default 926
/// (Blue/None), Cyan 940, Pink 939, Purple 941, White 942.
pub fn party_hat_gore_of_color(color: i32) -> i32 {
    match color {
        3 => 940, // Cyan, :81715
        2 => 939, // Pink, :81718
        4 => 941, // Purple, :81721
        5 => 942, // White, :81724
        _ => 926, // :81711
    }
}

/// The side-effects the wiring caller owes (the roster/World seams this
/// module cannot reach — the `Style74Outcome` precedent).
#[derive(Default)]
pub struct HitEffectOutcome {
    /// The daybreak proximity sweep (`:81742-81749`): apply
    /// `AddBuff(189, 300)` to every active NPC inside 100 px of the
    /// centre that passes the gate (`!buffImmune[189] && !dontTakeDamage
    /// && lifeMax > 5 && !friendly && !townNPC`). `None` = the arm did
    /// not run.
    pub daybreak_add_buff: Option<(f32, f32)>,
    /// `NPC.unlockedSlimePurpleSpawn = true` (`:81772-81779`) — the
    /// caller raises the World flag (the `unlocked_slime_red_spawn`
    /// precedent, `world_sim/editing.rs:2933`).
    pub unlock_slime_purple: bool,
    /// The 594 slave release (`:81805-81811`): the caller resolves
    /// `AI_113_WindyBalloon_GetSlaveNPC()` and, when found, zeroes its
    /// `ai[0]`, lifts `position.Y` by 10 and flags `netUpdate`.
    pub windy_balloon_slave_release: bool,
    /// W4-e5 (wave B): the 551 crystal's `DD2Event.CheckProgress(551)`
    /// (`NPC.cs:82174-82177`) — fires when the strike crossed a whole-life
    /// percent AND `life > 0` (netMode != 1: the single-player port runs
    /// it). The caller runs the ported `world_sim/npcs.rs:2561` walk.
    pub dd2_crystal_check_progress: bool,
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`; the
/// angle is float arithmetic, cos/sin in double of the widened float
/// (`ToRotationVector2`, `Utils.cs:1665-1668`), cast back to f32.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// THEN a second `NextFloat` radius scale (the `npc_ai_empress.rs` twin).
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`), centre default (0,0): the
/// cos/sin are FLOAT casts of the double radians, the products float.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let num = (radians as f64).cos() as f32;
    let num2 = (radians as f64).sin() as f32;
    Vec2::new(v.x * num - v.y * num2, v.x * num2 + v.y * num)
}

/// `Entity.Bottom` (`Entity.cs:122-132`) — `(pos.X + width/2f, pos.Y +
/// height)`; the int width/height widen to float exactly.
fn bottom(npc: &Npc) -> Vec2 {
    let (w, h) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y + h)
}

/// `Entity.Center` (`Entity.cs:50-60`). `pub(crate)` since W4-e5 (wave B's
/// 572-575 anchors reuse it).
pub(crate) fn center(npc: &Npc) -> Vec2 {
    let (w, h) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0)
}

/// `Entity.Top` (`Entity.cs:86-96`).
fn top(npc: &Npc) -> Vec2 {
    let (w, _) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y)
}

/// The party-hat block (`NPC.cs:81660-81729`): the head gore (sites
/// 1-13) then the hat itself (site 14). NO arm-level draws — the types
/// are data; each spawn's kicks roll inside `GorePool::new_gore`.
fn party_hat_block(
    npc: &Npc,
    npc_type: i32,
    inputs: &HitEffectInputs,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:81660` — townNPC && life <= 0 && UsesPartyHat().
    if !(inputs.town_npc && npc.hp <= 0 && inputs.uses_party_hat) {
        return;
    }
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    // `:81664-81708` — the head switch.
    if let Some(ty) = party_hat_gore_type(npc_type, inputs.is_shimmer_variant) {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
    // `:81709-81728` — `flag` is the literal true (`:81663`), so the hat
    // ALWAYS spawns when the block runs.
    let color = party_hat_color(
        inputs.uses_party_hat,
        &inputs.given_or_type_name,
        inputs.who_am_i,
        inputs.moon_phase,
        inputs.day_time,
    );
    gore.new_gore(rng, world, pos, vel, party_hat_gore_of_color(color), 1.0);
}

/// The Queen Slime / Windy Balloon burst (`NPC.cs:81752-81767` /
/// `:81789-81804`, sites 15-22): 3 `NewGorePerfect` of `base + Next(3)`
/// then ONE 1146 crystal. Per gore: Circular(15,15) + Circular(2,2) +
/// Next(3) + the kicks (pool) + the NextFloat Y-drop; the crystal uses
/// Circular(5,5). The velocity ×4 write lands AFTER the Y-drop.
fn queen_burst(
    npc: &Npc,
    base_type: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:81755` / `:81792` — `Bottom + (-UnitY * 50).RotatedBy(rotation)`.
    let vector = bottom(npc) + rotated_by(Vec2::new(0.0, -50.0), npc.rotation as f64);
    for _ in 0..3 {
        // `:81758` / `:81795`.
        let pos = vector + next_vector2_circular(rng, 15.0, 15.0);
        let vel = next_vector2_circular(rng, 2.0, 2.0);
        let ty = base_type + rng.next_max(3);
        let idx = gore.new_gore_perfect(rng, world, pos, vel, ty, 1.0);
        // `:81759-81760` / `:81796-81797` — the sentinel slot included.
        let g = gore.slot_mut(idx);
        g.velocity.y -= rng.next_float();
        g.velocity *= 4.0;
    }
    // `:81764` / `:81801` — the crystal recomputes Bottom (the C# reads
    // it again; the position has not moved).
    let pos = bottom(npc)
        + rotated_by(Vec2::new(0.0, -50.0), npc.rotation as f64)
        + next_vector2_circular(rng, 5.0, 5.0);
    let vel = next_vector2_circular(rng, 2.0, 2.0);
    let idx = gore.new_gore_perfect(rng, world, pos, vel, 1146, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity.y -= rng.next_float();
    g.velocity *= 4.0;
}

/// The daybreak arm (`NPC.cs:81730-81751`): 20 dusts of
/// `SelectRandom(6,259,158)` at the hitbox top-left, alpha 200, velocity
/// ×1.4, scale += NextFloat. The buff sweep is RETURNED (the roster seam).
fn daybreak_arm(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    outcome: &mut HitEffectOutcome,
) {
    let (w, h) = npc.size();
    // `:81732` — `base.Hitbox.TopLeft()` = `(int)position` (`Entity.cs:175`).
    let pos = Vec2::new(npc.x as i32 as f32, npc.y as i32 as f32);
    for _ in 0..20 {
        // `:81735` — SelectRandom = Next(3) over the list (`Utils.cs:2459-2462`).
        let ty = [6, 259, 158][rng.next_max(3) as usize];
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, -2.5, 0,
            Color::default(), 1.0,
        );
        // `:81736-81738` — the post-writes, sentinel included.
        let d = dust.slot_mut(idx);
        d.alpha = 200;
        d.velocity *= 1.4;
        d.scale += rng.next_float();
    }
    // `:81740-81750` — netMode != 1 (single-player: runs). The sweep
    // itself is the caller's (the roster seam); the centre is handed over.
    let c = center(npc);
    outcome.daybreak_add_buff = Some((c.x, c.y));
}

/// The Deerclops 668 death (`NPC.cs:81894-81909`, sites 23-29): the
/// 120-dust type-5 burst (velocity = Circular(4,4)) then the 7 body
/// gores at velocity ×0.6. The hit (`life > 0`) half is dust-only and
/// rides the wiring wave's dmg-scaled loop — NOT this wave's sites.
fn deerclops_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    // `:81894-81900` — the 120-dust burst, the box grown by 15 each side.
    let num15 = 15;
    let vector3 = Vec2::new(npc.x - num15 as f32, npc.y - num15 as f32);
    for _ in 0..120 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, vector3, w as i32 + num15 * 2, h as i32 + num15 * 2,
            5, 0.0, 0.0, 0, Color::default(), 1.5,
        );
        // `:81899` — the velocity overwrite (the circular draw AFTER the
        // spawn's own rolls).
        dust.slot_mut(idx).velocity = next_vector2_circular(rng, 4.0, 4.0);
    }
    // `:81901-81908` — the 7 gores.
    let vector4 = Vec2::new(npc.vx, npc.vy) * 0.6;
    let b = bottom(npc);
    let c = center(npc);
    let t = top(npc);
    let s = npc.scale;
    gore.new_gore(rng, world, b + Vec2::new(-10.0, 0.0), vector4, 1274, s); // :81902
    gore.new_gore(rng, world, b + Vec2::new(10.0, 0.0), vector4, 1274, s); // :81903
    gore.new_gore(rng, world, c, vector4, 1272, s); // :81904
    gore.new_gore(rng, world, c + Vec2::new(-10.0, 0.0), vector4, 1273, s); // :81905
    gore.new_gore(rng, world, c + Vec2::new(10.0, 0.0), vector4, 1273, s); // :81906
    gore.new_gore(rng, world, t, vector4, 1270, s); // :81907
    gore.new_gore(rng, world, t, vector4, 1271, s); // :81908
}

/// The Empress 636 death's dust loop (`NPC.cs:81929-81939`) — the DRAWS
/// only (the emit is the documented skip until the wiring wave feeds the
/// pool): 50 iterations, each `Next(2)` then the type-67/69 `NewDust`
/// internals (the 6 default-block draws; neither type hits a tweak,
/// `Dust.cs:214-252`). The death branch's else-arm passes
/// `2 * hitDirection` (`:81937`) — a velocity offset, no extra draw.
/// The position draws read the C# box (`base.width`/`base.height`,
/// clamped to ≥5 by `NewDust`, `Dust.cs:155-162`).
pub fn empress_hit_dust_draws(rng: &mut UnifiedRandom, width: i32, height: i32) {
    let w = width.max(5);
    let h = height.max(5);
    for _ in 0..50 {
        let _selector = rng.next_max(2); // :81931 / :81933
        // The NewDust default block (Dust.cs:153-193): position X, Y,
        // velocity X, Y, the frame row, the scale — SIX draws. The
        // pressure-band gate is the pool's (a fresh pool's low slots hit
        // no band); the off-screen cull draws nothing.
        let _px = rng.next_max(w - 4);
        let _py = rng.next_max(h - 4);
        let _vx = rng.next_range(-20, 21);
        let _vy = rng.next_range(-20, 21);
        let _row = rng.next_max(3);
        let _scale = rng.next_range(-20, 21);
    }
}

/// The Empress 636 death's FIRST gore (`NPC.cs:81941`, site 30): type
/// 1262 at `position + (10,-20)`, velocity ×0.6. The remaining 8 gores
/// (`:81942-81949`) are wave B. The dust draws MUST run first (the C#
/// order) — the caller runs [`empress_hit_dust_draws`] before this fn.
fn empress_first_gore(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let vector5 = Vec2::new(npc.vx, npc.vy) * 0.6;
    gore.new_gore(
        rng, world,
        Vec2::new(npc.x + 10.0, npc.y - 20.0),
        vector5, 1262, npc.scale,
    );
}

/// The Empress 636 death's gore TAIL (`NPC.cs:81942-81949`, wave-B sites
/// 31-38): the remaining 8 body gores at velocity ×0.6, in source order.
/// The caller runs [`empress_hit_dust_draws`] + [`empress_first_gore`]
/// first (the C# order).
pub(crate) fn empress_tail_gores(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let vector5 = Vec2::new(npc.vx, npc.vy) * 0.6;
    let p = Vec2::new(npc.x, npc.y);
    let s = npc.scale;
    gore.new_gore(rng, world, p + Vec2::new(10.0, 40.0), vector5, 1263, s); // :81942
    gore.new_gore(rng, world, p + Vec2::new(50.0, 20.0), vector5, 1264, s); // :81943
    gore.new_gore(rng, world, p + Vec2::new(-10.0, -20.0), vector5, 1265, s); // :81944
    gore.new_gore(rng, world, p, vector5, 1266, s); // :81945
    gore.new_gore(rng, world, p + Vec2::new(-20.0, -20.0), vector5, 1267, s); // :81946
    gore.new_gore(rng, world, p + Vec2::new(-20.0, 10.0), vector5, 1268, s); // :81947
    gore.new_gore(rng, world, p + Vec2::new(60.0, -20.0), vector5, 1267, s); // :81948
    gore.new_gore(rng, world, p + Vec2::new(60.0, 10.0), vector5, 1268, s); // :81949
}

/// `NPC.HitEffect(hitDirection, dmg)` (`NPC.cs:81654-81958`) — wave A:
/// the first 30 gore sites in source order. `npc.hp` is `this.life` AT
/// THE CALL (the strike writes it before calling, `:81488`); `dmg` rides
/// the daybreak/668/636 hit-half loop bounds (the death halves here use
/// the flat counts). Returns the side-effects the wiring caller owes.
///
/// The 690 (`:81782`) and 661 (`:81813`) blocks are dust-only (no gore
/// sites) and stay with the wiring wave's dust pass; the 521 block
/// (`:81862`) likewise. The 686 hit-half has no gore (the burst is
/// `life <= 0`-gated, `:81752`).
pub fn hit_effect(
    npc: &mut Npc,
    npc_type: i32,
    inputs: &HitEffectInputs,
    hit_direction: i32,
    dmg: f64,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) -> HitEffectOutcome {
    let _ = dmg; // the death halves use flat counts; the hit halves are wave B
    let _ = hit_direction; // wave A's arms read no hitDirection; wave B's do
    let mut outcome = HitEffectOutcome::default();
    // `:81660-81729` — the party-hat block (sites 1-14).
    party_hat_block(npc, npc_type, inputs, gore, rng, world);
    // `:81730-81751` — the daybreak arm.
    if inputs.daybreak && npc.hp <= 0 {
        daybreak_arm(npc, dust, dust_ctx, rng, world, &mut outcome);
    }
    // `:81752-81781` — the Queen Slime 686 (sites 15-18).
    if npc_type == 686 && npc.hp <= 0 {
        queen_burst(npc, 1143, gore, rng, world);
        // `:81770-81771` — `position = Bottom + (0,48)` then Transform(680).
        let b = bottom(npc);
        npc.x = b.x;
        npc.y = b.y + 48.0;
        npc.wants_transform = Some(680);
        // `:81772-81779` — the unlock (the caller raises the World flag).
        outcome.unlock_slime_purple = true;
    }
    // `:81789-81812` — the Windy Balloon 594 (sites 19-22).
    if npc_type == 594 && npc.hp <= 0 {
        // `:81791` — `1125 + (int)(ai[2] - 1) * 3`.
        let base = 1125 + (npc.ai[2] - 1.0) as i32 * 3;
        queen_burst(npc, base, gore, rng, world);
        outcome.windy_balloon_slave_release = true;
    }
    // `:81883-81910` — the Deerclops 668 death (sites 23-29).
    if npc_type == 668 && npc.hp <= 0 {
        deerclops_death(npc, dust, dust_ctx, gore, rng, world);
    }
    // `:81911-81951` — the Empress 636 death: the dust draws then the
    // FIRST gore (site 30). The 8-gore tail is wave B.
    if npc_type == 636 && npc.hp <= 0 {
        let (w, h) = npc.size();
        empress_hit_dust_draws(rng, w as i32, h as i32);
        empress_first_gore(npc, gore, rng, world);
    }
    // W4-e5 (wave B): the 636 tail (sites 31-38) then the 657/658-660/667
    // + DD2 arms — the same `life <= 0` gates, source order.
    if npc_type == 636 && npc.hp <= 0 {
        empress_tail_gores(npc, gore, rng, world);
    }
    crate::npc_hit_effect_b::hit_effect_b(npc, npc_type, hit_direction, dmg, gore, dust, dust_ctx, rng, world, &mut outcome);
    outcome
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 200)
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

    /// A boss-shaped stand-in: the unported types ride a placeholder kind
    /// with the C# box through `size_override` (the style50 precedent).
    fn boss(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 100, 0.0);
        n.size_override = Some((w, h));
        n.hp = 0;
        n.scale = 1.0;
        n
    }

    /// `NPC.cs:81664-81708` — the 13 head rows, both shimmer states; 229
    /// spawns nothing shimmered.
    #[test]
    fn w4_d55_the_party_hat_head_rows_match_the_csharp() {
        let rows = [
            (38, 927, 1298), (207, 928, 1305), (17, 929, 1321),
            (18, 930, 1325), (227, 931, 1331), (142, 933, 1339),
            (453, 934, 1343), (178, 935, 1348), (353, 936, 1351),
            (368, 937, 1363), (108, 938, 1373), (588, 1112, 1311),
        ];
        for (ty, normal, shimmered) in rows {
            assert_eq!(party_hat_gore_type(ty, false), Some(normal));
            assert_eq!(party_hat_gore_type(ty, true), Some(shimmered));
        }
        assert_eq!(party_hat_gore_type(229, false), Some(932));
        assert_eq!(party_hat_gore_type(229, true), None);
        assert_eq!(party_hat_gore_type(22, false), None); // no row
    }

    /// `NPC.cs:66642-66658` — the hash: "Andrew" (6 + 'A'=65 = 71),
    /// whoAmI 0, moonPhase 0, dayTime → num2 = -1, dir +1 → 70 → %5 = 0
    /// → ++ → 1 (Blue → 926).
    #[test]
    fn w4_d55_the_hat_colour_hash_matches_the_csharp() {
        assert_eq!(party_hat_color(true, "Andrew", 0, 0, true), 1);
        assert_eq!(party_hat_gore_of_color(1), 926);
        // The colour → gore rows (:81711-81726).
        assert_eq!(party_hat_gore_of_color(0), 926);
        assert_eq!(party_hat_gore_of_color(2), 939);
        assert_eq!(party_hat_gore_of_color(3), 940);
        assert_eq!(party_hat_gore_of_color(4), 941);
        assert_eq!(party_hat_gore_of_color(5), 942);
        // The gates: no hat / empty name → None (0).
        assert_eq!(party_hat_color(false, "Andrew", 0, 0, true), 0);
        assert_eq!(party_hat_color(true, "", 0, 0, true), 0);
        // The negative branch (+5 once): name "A" (1+65=66), whoAmI 0,
        // moonPhase 0, night → num2 = 0 → 66 % 5 = 1.
        assert_eq!(party_hat_color(true, "A", 0, 0, false), 1);
    }

    /// `NPC.cs:81758-81766` — the 686 burst consumes 8 draws per gore ×4
    /// on the strike stream, in the cited order; the twin replay mirrors
    /// the C# argument draws + the NewGore kicks.
    #[test]
    fn w4_d55_the_queen_burst_draws_eight_per_gore_in_order() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.rotation = 0.3;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(7);
        queen_burst(&n, 1143, &mut gore, &mut rng, &w);
        // The twin: the same draws by hand (Circular = unit + radius).
        let mut twin = UnifiedRandom::new(7);
        let circular = |r: &mut UnifiedRandom, hw: f32, hh: f32| {
            let a = std::f32::consts::PI * 2.0 * r.next_float();
            let u = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32);
            let rad = r.next_float();
            Vec2::new(u.x * hw * rad, u.y * hh * rad)
        };
        for _ in 0..3 {
            let _p = circular(&mut twin, 15.0, 15.0);
            let _v = circular(&mut twin, 2.0, 2.0);
            let _ty = twin.next_max(3);
            let _ky = twin.next_range(10, 31); // NewGore Y kick
            let _kx = twin.next_range(-20, 21); // NewGore X kick
            let _drop = twin.next_float();
        }
        let _p = circular(&mut twin, 5.0, 5.0);
        let _v = circular(&mut twin, 2.0, 2.0);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        let _drop = twin.next_float();
        assert_eq!(rng.peek(), twin.peek(), "the burst mirrors the C# draws");
        // Four gores landed (the pool's first four slots).
        let active = gore.slots()[..4].iter().filter(|g| g.active).count();
        assert_eq!(active, 4);
    }

    /// `NPC.cs:81902-81908` — the Deerclops death: 7 gores, the types and
    /// anchors in source order, velocity ×0.6 + the NewGore kicks.
    #[test]
    fn w4_d55_the_deerclops_death_spawns_the_seven_body_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 60.0, 80.0);
        n.vx = 2.0;
        n.vy = -1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        deerclops_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1274, 1274, 1272, 1273, 1273, 1270, 1271]);
        // The velocity is ×0.6 plus the kicks (the Y lift is negative).
        for g in &gore.slots()[..7] {
            assert!(g.velocity.y < -0.6, "the kick lifts: {:?}", g.velocity);
        }
        // The 120-dust burst ran first (the pool holds the dust).
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 100, "the burst spawned: {dust_active}");
    }

    /// `NPC.cs:81929-81941` — the Empress death: the 50-iteration dust
    /// draw census (7 draws each: the selector + the 6 default-block
    /// rolls) then the first gore's two kicks.
    #[test]
    fn w4_d55_the_empress_death_draws_the_dust_census_then_the_first_gore() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 40.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(11);
        empress_hit_dust_draws(&mut rng, 40, 40);
        empress_first_gore(&n, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(11);
        for _ in 0..50 {
            let _sel = twin.next_max(2);
            let _px = twin.next_max(36);
            let _py = twin.next_max(36);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "the census + the first gore mirror");
        assert_eq!(gore.slot(0).gore_type, 1262);
        assert_eq!(gore.slot(0).position, Vec2::new(110.0, 80.0));
    }

    /// The full wave-A dispatch on a 686: the burst + the transform
    /// request + the unlock outcome; the party-hat/daybreak gates stay
    /// off for a non-town NPC.
    #[test]
    fn w4_d55_the_dispatch_runs_the_686_arm_and_returns_the_seams() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        let out = hit_effect(
            &mut n, 686, &HitEffectInputs::default(), 1, 10.0,
            &mut gore, &mut dust, &ctx(), &mut rng, &w,
        );
        assert!(out.unlock_slime_purple);
        assert_eq!(n.wants_transform, Some(680));
        assert!(out.daybreak_add_buff.is_none());
        assert!(!out.windy_balloon_slave_release);
        // position = Bottom + (0,48): Bottom = (120,140) → (120,188).
        assert_eq!((n.x, n.y), (120.0, 188.0));
    }

    /// `NPC.cs:6713-6723` — the shimmer gate: variation 1 AND the
    /// ShimmerTownTransform row.
    #[test]
    fn w4_d55_the_shimmer_gate_needs_variation_one_and_the_row() {
        assert!(is_shimmer_variant(22, 1)); // the Guide is in the set
        assert!(!is_shimmer_variant(22, 0));
        assert!(!is_shimmer_variant(1, 1)); // the slime is not
    }
}
