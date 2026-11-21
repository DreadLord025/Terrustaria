//! `NPC.HitEffect` — the per-NPC gore/dust table, wave B: the 40-site
//! `:81942-82205` window (the f5 seal's e5 claim, research/111 §5.2),
//! continuing wave A (`npc_hit_effect_a.rs`, W4-d55, ec78d152).
//!
//! Wave B is appended to wave A's dispatch: [`hit_effect_b`] runs right
//! after wave A's last arm (`npc_hit_effect_a.rs::hit_effect`), the same
//! `this.type` blocks in source order. The RNG contract is wave A's — the
//! strike caller's stream (the `StrikeNPC_Inner` `:81488` swap), every
//! draw cited per site.
//!
//! ## Scope — sites 31-76 (source order, the `:81942-82205` window)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 31-38 | `:81942`-`:81949` | the Empress 636 8-gore tail (`npc_hit_effect_a::empress_tail_gores`) |
//! | 39-41 | `:81956`-`:81958` | the 657 Queen Slime death (2×1259 + 1258) |
//! | 42-43 | `:82012`-`:82013` | the 660 healing-crystal death (2×1260) |
//! | 44-51 | `:82037` | the 667 death's 8×1218 burst |
//! | 52-56 | `:82103`-`:82107` | the 552/553/554 DD2 goblin death (5) |
//! | 57-63 | `:82134`-`:82140` | the 561/562/563 DD2 javeliner death (7) |
//! | 64-66 | `:82167`-`:82169` | the 555/556/557 DD2 kobold death (3) |
//! | 67-76 | `:82193`-`:82202` | the 551 crystal death, `spriteDirection != -1` (10) |
//!
//! KEEP-ONE (research/304): the 551 `-1` mirrored twin (`:82206-82215`,
//! this wave's old sites 77-86) is DROPPED — it is f10's claim
//! (`npc_hit_effect_c.rs::betsy_flipped_death`, its sites 71-80, the legit
//! 385 the f5 seal §5.2/§8 keeps). The wave-B dispatch gates the crystal
//! death on `sprite_dir != -1`, the exact complement of f10's gate.
//! NAMING: NPCID 551 = `DD2Betsy` (`NPCID.cs`); this module's `dd2_crystal_*`
//! fn names + "DD2 crystal" label are an internal misnomer for the SAME
//! kind f10 calls "Betsy" (the Eternia Crystal is 548/522, a different
//! arm) — the names are kept for API stability, the kind is Betsy.
//! Everything from `:82206` (the flipped twin, 572/573 onward, the
//! 566/567 `Next(2)` gates, the 576/577 pair, …) is wave C
//! (`W4-f10-gore-sites-3`). The 667 HIT arm's 3-gore twin (`:82045`, the
//! window's one hit-half site) is a wiring hand-off (research/304 §4 —
//! the seal §7 pattern, the FIFTH hit-half): g62's live 667 arm
//! (`npc_hit_dust.rs`) draws the 3×(Next(1,10)+Next(4)) velocity-overwrite
//! pairs but NOT the 3×2 NewGore kicks — the emit-at-wiring must insert
//! the kicks BEFORE each overwrite pair (g62's discard census + witness
//! gain +6 draws; the C# :82045 NewGore draws kicks pool-side, then :82046
//! overwrites). The body is g62 EMIT-AT-WIRING, not a window-lane port.
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * The hit halves (`life > 0`): the dmg-scaled dust loops
//!   (`dmg / lifeMax * 20`, the 657 12-dust loop, the 658-660/667 7-dust
//!   loops) stay with the wiring wave's dust pass — wave A's convention.
//!   The 667 hit arm ALSO carries a 3-gore twin (`:82045`, the window's
//!   one hit-half gore site) — its body is the g62 emit-at-wiring hand-off
//!   (research/304 §4), not ported here. Exception: the 551
//!   `DD2Event.CheckProgress` gate (`:82174-82177`) fires on the HIT path
//!   and is RETURNED (`HitEffectOutcome::dd2_crystal_check_progress`).
//! * The death dust loops that share an arm with gore sites RUN here on
//!   the pool (the 657 150-loop, the 658-660/667 count loops, the DD2
//!   20-dust type-5 loops) — the C# order: 657 gores BEFORE the dust,
//!   660 gores BEFORE the dust, 667 gores BEFORE the dust, DD2 dust
//!   BEFORE the gores. The 636 dust loop keeps wave A's draw-census skip
//!   (`empress_hit_dust_draws`).
//! * `hitDirection`/`dmg` ride explicit parameters (the C# `HitEffect(int
//!   hitDirection = 0, double dmg = 10.0)`, `:81654`); `dmg` feeds the 551
//!   `CheckProgress` percent-crossing gate.
//! * The 551 `DD2Event.CheckProgress` gate (`:82174-82177`) is ported as
//!   the pure [`dd2_crystal_crossed_percent`] predicate and RETURNED
//!   (`HitEffectOutcome::dd2_crystal_check_progress`) — the caller runs
//!   the ported walk (`world_sim/npcs.rs:2561`).
//! * `AI_121_QueenSlime_GetDustColor` (`NPC.cs:45121-45124`) is ported as
//!   [`queen_slime_dust_color`] — TWO `NextFloat` draws (the inner lerp's
//!   amount evaluates first, C# argument order), `Color.Lerp` through
//!   `tr_core::dust_table::lerp_color` (the XNA per-channel truncate).
//!   `newColor.A = 200` lands AFTER the draw pair.
//! * `netMode` guards: single-player port (the `netMode != 1` arms run).
//!
//! See docs/ANALYSIS.md "The NPC HitEffect gore table (W4-e5, wave B)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the strike-path claim is the follow-up round

use crate::npc::Npc;
use crate::npc_hit_effect_a::{center, HitEffectOutcome};
use glam::Vec2;
use tr_core::dust_table::lerp_color;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `NPC.AI_121_QueenSlime_GetDustColor` (`NPC.cs:45121-45124`):
/// `Lerp((0,160,255), Lerp((200,200,200), (255,80,255), NextFloat()),
/// NextFloat())` — the inner amount draws FIRST (C# argument evaluation),
/// then the outer; both `Color.Lerp`s truncate per channel (XNA).
pub fn queen_slime_dust_color(rng: &mut UnifiedRandom) -> Color {
    let inner_t = rng.next_float();
    let outer_t = rng.next_float();
    let inner = lerp_color(
        Color::new(200, 200, 200, 255),
        Color::new(255, 80, 255, 255),
        inner_t,
    );
    lerp_color(Color::new(0, 160, 255, 255), inner, outer_t)
}

/// The 657 Queen Slime death (`NPC.cs:81952-81990`, sites 39-41): the 3
/// gores FIRST (2×1259 at scale 0.8 — the second at `position + Size*0.5`
/// — then 1258 at `this.scale`), then the 150-dust type-4 burst: per dust
/// the colour pair (2 draws), `8*NextFloatDirection` + `-2+6*
/// NextFloatDirection` (2), `1+NextFloat*0.5` scale (1), the pool's own
/// rolls, then the ×0.65/+velocity×0.5 writes and the `Next(5)==0`
/// no-gravity gate (1).
fn queen_slime_657_death(
    npc: &Npc,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let p = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let (w, h) = npc.size();
    // `:81956-81958` — the gores (sites 39-41).
    gore.new_gore(rng, world, p, vel, 1259, 0.8);
    gore.new_gore(rng, world, p + Vec2::new(w, h) * 0.5, vel, 1259, 0.8);
    gore.new_gore(rng, world, p, vel, 1258, npc.scale);
    // `:81959-81972` — the 150-dust burst, box grown by 10 each side.
    let num20 = 10;
    for _ in 0..150 {
        let mut color = queen_slime_dust_color(rng); // :81962
        color.a = 200; // :81963
        let sx = 8.0 * rng.next_float_direction();
        let sy = -2.0 + 6.0 * rng.next_float_direction();
        let scale = 1.0 + rng.next_float() * 0.5;
        let idx = dust.new_dust(
            rng, dust_ctx, world, p - Vec2::new(num20 as f32, num20 as f32),
            w as i32 + num20 * 2, h as i32 + num20 * 2, 4, sx, sy, 50, color, scale,
        ); // :81964
        // `:81965-81971` — the post-writes, sentinel included.
        let d = dust.slot_mut(idx);
        d.velocity *= 0.65;
        d.velocity += vel * 0.5;
        if rng.next_max(5) == 0 {
            d.no_gravity = true;
            d.scale *= 1.2;
        }
    }
}

/// The 658-660 healing/lightning/mana crystal family (`NPC.cs:81991-82024`,
/// sites 42-43): ONLY the 660 death spawns gores (2×1260 at position,
/// `this.scale`), BEFORE the shared dust loop. The loop count is 7 on a
/// hit / 40 on death; the type is 4 (658/660) or 243 (659); the colour is
/// (78,136,255,150) / default / (150,100,200,150); alpha 80; scale 1.0 hit
/// / 1.5 death. Per dust: the pool rolls then `Next(3)!=0` → noGravity.
fn crystal_658_660(
    npc: &Npc,
    npc_type: i32,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    // `:81993-82005` — the per-type constants.
    let mut count = 7;
    let mut scale = 1.0f32;
    let mut dust_type = 4;
    let mut color = Color::new(78, 136, 255, 150);
    if npc_type == 659 {
        dust_type = 243;
        color = Color::default();
    }
    if npc_type == 660 {
        color = Color::new(150, 100, 200, 150);
    }
    // `:82006-82015` — the death arm: the 660 gores (sites 42-43).
    if npc.hp <= 0 {
        scale = 1.5;
        count = 40;
        if npc_type == 660 {
            let p = Vec2::new(npc.x, npc.y);
            let vel = Vec2::new(npc.vx, npc.vy);
            gore.new_gore(rng, world, p, vel, 1260, npc.scale); // :82012
            gore.new_gore(rng, world, p, vel, 1260, npc.scale); // :82013
        }
    }
    // `:82016-82023` — the shared dust loop (the hit half runs here too:
    // no gore sites, but its draws precede any later arm's).
    let p = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        let idx = dust.new_dust(
            rng, dust_ctx, world, p, w as i32, h as i32, dust_type,
            (2 * hit_direction) as f32, -1.0, 80, color, scale,
        ); // :82018
        if rng.next_max(3) != 0 {
            dust.slot_mut(idx).no_gravity = true; // :82019-82022
        }
    }
}

/// The 667 GoldenSlime death/hit (`NPC.cs:82025-82057`, sites 44-51;
/// `NPCID.cs` GoldenSlime = 667):
/// the death arm spawns 8×1218 at `(position.X, Center.Y-10)` with
/// velocity Zero, then OVERWRITES the slot velocity — `Next(1,10)*0.3*
/// 2.5*hitDirection` X, `-(3+Next(4)*0.3)` Y (the kicks are drawn inside
/// `NewGore` first, then discarded by the overwrite). The hit arm is the
/// 3-gore twin (`×2`, `-(2.5+…)`, `:82045-82046`) — the g62 emit-at-wiring
/// hand-off (research/304 §4), NOT wave C. The dust loop
/// (7 hit / 40 death, type 10, default colour, alpha 80, scale 1.1/1.5,
/// `Next(3)!=0` noGravity) runs AFTER the gores.
fn skeleton_667_death(
    npc: &Npc,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    // `:82027-82034` — the death constants.
    let count = 40;
    let scale = 1.5f32;
    // `:82035-82039` — the 8-gore burst (sites 44-51).
    let pos = Vec2::new(npc.x, center(npc).y - 10.0);
    for _ in 0..8 {
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, 1218, 1.0); // :82037
        // `:82038` — the velocity overwrite, sentinel included.
        let vx = rng.next_range(1, 10) as f32 * 0.3 * 2.5 * hit_direction as f32;
        let vy = 0.0 - (3.0 + rng.next_max(4) as f32 * 0.3);
        gore.slot_mut(idx).velocity = Vec2::new(vx, vy);
    }
    // `:82049-82056` — the dust loop.
    let p = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        let idx = dust.new_dust(
            rng, dust_ctx, world, p, w as i32, h as i32, 10,
            (2 * hit_direction) as f32, -1.0, 80, Color::default(), scale,
        ); // :82051
        if rng.next_max(3) != 0 {
            dust.slot_mut(idx).no_gravity = true; // :82052-82055
        }
    }
}

/// The DD2 death dust preamble shared by the 552-575 families
/// (`NPC.cs:82090-82093` and twins): 20 dusts of `dust_type` at the
/// hitbox, `hitDirection`/−1 velocity, default colour/scale. The C#
/// loop is `(float)num < 20f` — 20 iterations.
fn dd2_death_dust(
    npc: &Npc,
    dust_type: i32,
    hit_direction: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let p = Vec2::new(npc.x, npc.y);
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, p, w as i32, h as i32, dust_type,
            hit_direction as f32, -1.0, 0, Color::default(), 1.0,
        );
    }
}

/// The 552/553/554 DD2 goblin death (`NPC.cs:82079-82109`, sites 52-56):
/// the 20-dust type-5 preamble then 5 gores — base 983 (553→1067,
/// 554→1071): `base` at position, `base+3`/`base+2` at Y+20,
/// `base+1`×2 at Y+34, all `base.velocity`/`this.scale`.
fn dd2_goblin_death(
    npc: &Npc,
    npc_type: i32,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    dd2_death_dust(npc, 5, hit_direction, dust, dust_ctx, rng, world); // :82090-82093
    // `:82094-82102` — the per-type base.
    let base = match npc_type {
        553 => 1067,
        554 => 1071,
        _ => 983,
    };
    let p = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, p, vel, base, s); // :82103
    gore.new_gore(rng, world, p + Vec2::new(0.0, 20.0), vel, base + 3, s); // :82104
    gore.new_gore(rng, world, p + Vec2::new(0.0, 20.0), vel, base + 2, s); // :82105
    gore.new_gore(rng, world, p + Vec2::new(0.0, 34.0), vel, base + 1, s); // :82106
    gore.new_gore(rng, world, p + Vec2::new(0.0, 34.0), vel, base + 1, s); // :82107
}

/// The 561/562/563 DD2 javeliner death (`NPC.cs:82110-82142`, sites
/// 57-63): the 20-dust type-5 preamble then 7 gores — base 987
/// (562→1055, 563→1061): `base+3` at position, `base`/`base+4`/
/// `base+5`/`base+2` at Y+20, `base+1`×2 at Y+34. NOTE the order
/// differs from the goblin's (the +3 leads).
fn dd2_javeliner_death(
    npc: &Npc,
    npc_type: i32,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    dd2_death_dust(npc, 5, hit_direction, dust, dust_ctx, rng, world); // :82121-82124
    // `:82125-82133` — the per-type base.
    let base = match npc_type {
        562 => 1055,
        563 => 1061,
        _ => 987,
    };
    let p = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, p, vel, base + 3, s); // :82134
    gore.new_gore(rng, world, p + Vec2::new(0.0, 20.0), vel, base, s); // :82135
    gore.new_gore(rng, world, p + Vec2::new(0.0, 20.0), vel, base + 4, s); // :82136
    gore.new_gore(rng, world, p + Vec2::new(0.0, 20.0), vel, base + 5, s); // :82137
    gore.new_gore(rng, world, p + Vec2::new(0.0, 20.0), vel, base + 2, s); // :82138
    gore.new_gore(rng, world, p + Vec2::new(0.0, 34.0), vel, base + 1, s); // :82139
    gore.new_gore(rng, world, p + Vec2::new(0.0, 34.0), vel, base + 1, s); // :82140
}

/// The 555/556/557 DD2 kobold death (`NPC.cs:82143-82171`, sites 64-66):
/// the base pick (1031, 556→1034, 557→1037) precedes the 20-dust type-5
/// loop (no draws in the pick), then 3 gores: `base` at (X,Y), `base+1`
/// at (X+10,Y+20), `base+2` at (X+10,Y+30).
fn dd2_kobold_death(
    npc: &Npc,
    npc_type: i32,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:82154-82162` — the per-type base.
    let base = match npc_type {
        556 => 1034,
        557 => 1037,
        _ => 1031,
    };
    dd2_death_dust(npc, 5, hit_direction, dust, dust_ctx, rng, world); // :82163-82166
    let p = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, p, vel, base, s); // :82167
    gore.new_gore(rng, world, p + Vec2::new(10.0, 20.0), vel, base + 1, s); // :82168
    gore.new_gore(rng, world, p + Vec2::new(10.0, 30.0), vel, base + 2, s); // :82169
}

/// The 551 DD2 crystal's `DD2Event.CheckProgress` gate
/// (`NPC.cs:82174-82177`): the strike crossed a whole-life percent —
/// `(int)(life/(float)lifeMax*100f) != (int)((life+dmg)/(double)
/// (float)lifeMax*100.0)` — AND `life > 0`; netMode != 1 (the
/// single-player port runs it). Pure predicate; the caller runs the
/// ported walk (`world_sim/npcs.rs:2561`).
pub fn dd2_crystal_crossed_percent(life: i32, life_max: i32, dmg: f64) -> bool {
    let before = (life as f32 / life_max as f32 * 100.0) as i32;
    let after = ((life as f64 + dmg) / (life_max as f32 as f64) * 100.0) as i32;
    before != after && life > 0
}

/// The 551 DD2 crystal death, `spriteDirection != -1` arm
/// (`NPC.cs:82185-82203`, sites 67-76): the 20-dust type-5 preamble then
/// the 10-gore body (types 1079/1080/1080/1081/1082/1083/1084/1085/1085/
/// 1086 at the cited offsets).
///
/// KEEP-ONE (research/304, the e5-vs-f10 HitEffect overlap): the `-1`
/// mirrored twin (`:82204-82216`, the same type sequence at the mirrored
/// offsets) is f10's claim (`npc_hit_effect_c.rs::betsy_flipped_death`,
/// its sites 71-80, window `:82206-85559` — the legit 385 the f5 seal
/// §5.2/§8 and f10's own overrun-delete `afea4c0fc` KEEP). e5's window is
/// `:81942-82205` (40 census sites), so e5 DROPS the flipped else-arm.
/// The dispatch gates this whole body on `sprite_dir != -1` so the shared
/// 20-dust preamble runs exactly ONCE per death: e5 owns it for `!= -1`,
/// f10's `betsy_flipped_death` owns it for `== -1` (its own preamble copy).
fn dd2_crystal_death(
    npc: &Npc,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    dd2_death_dust(npc, 5, hit_direction, dust, dust_ctx, rng, world); // :82187-82190
    let p = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:82191-82203` — the spriteDirection != -1 arm (sites 67-76). The
    // `-1` twin (`:82206-82215`) is f10's — the caller gates on != -1.
    gore.new_gore(rng, world, p + Vec2::new(0.0, 54.0), vel, 1079, s); // :82193
    gore.new_gore(rng, world, p + Vec2::new(72.0, 90.0), vel, 1080, s); // :82194
    gore.new_gore(rng, world, p + Vec2::new(102.0, 94.0), vel, 1080, s); // :82195
    gore.new_gore(rng, world, p + Vec2::new(178.0, 0.0), vel, 1081, s); // :82196
    gore.new_gore(rng, world, p + Vec2::new(136.0, 12.0), vel, 1082, s); // :82197
    gore.new_gore(rng, world, p + Vec2::new(82.0, 18.0), vel, 1083, s); // :82198
    gore.new_gore(rng, world, p + Vec2::new(100.0, 26.0), vel, 1084, s); // :82199
    gore.new_gore(rng, world, p + Vec2::new(170.0, 16.0), vel, 1085, s); // :82200
    gore.new_gore(rng, world, p + Vec2::new(190.0, 10.0), vel, 1085, s); // :82201
    gore.new_gore(rng, world, p + Vec2::new(100.0, 36.0), vel, 1086, s); // :82202
}

/// The wave-B dispatch tail — appended to wave A's `hit_effect` in source
/// order (`NPC.cs:81942` onward). `npc.hp` is `this.life` at the call.
/// Scope: the 657/658-660/667 arms + the DD2 goblin/javeliner/kobold/
/// crystal families (the 636 tail rides wave A's dispatch).
pub(crate) fn hit_effect_b(
    npc: &mut Npc,
    npc_type: i32,
    hit_direction: i32,
    dmg: f64,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    outcome: &mut HitEffectOutcome,
) {
    // `:81952-81990` — the 657 Queen Slime death (sites 39-41).
    if npc_type == 657 && npc.hp <= 0 {
        queen_slime_657_death(npc, gore, dust, dust_ctx, rng, world);
    }
    // `:81991-82024` — the 658-660 crystal family (sites 42-43).
    if (658..=660).contains(&npc_type) {
        crystal_658_660(npc, npc_type, hit_direction, gore, dust, dust_ctx, rng, world);
    }
    // `:82025-82057` — the 667 death (sites 44-51); the hit arm's 3-gore
    // twin (`:82045`) is the g62 emit-at-wiring hand-off (research/304 §4),
    // its dust loop rides the wiring wave's dust pass.
    if npc_type == 667 && npc.hp <= 0 {
        skeleton_667_death(npc, hit_direction, gore, dust, dust_ctx, rng, world);
    }
    // `:82079-82109` — the 552-554 DD2 goblin death (sites 52-56). The
    // 676 block (`:82058-82078`) is dust-only — the wiring wave's pass.
    if matches!(npc_type, 552 | 553 | 554) && npc.hp <= 0 {
        dd2_goblin_death(npc, npc_type, hit_direction, gore, dust, dust_ctx, rng, world);
    }
    // `:82110-82142` — the 561-563 DD2 javeliner death (sites 57-63).
    if matches!(npc_type, 561 | 562 | 563) && npc.hp <= 0 {
        dd2_javeliner_death(npc, npc_type, hit_direction, gore, dust, dust_ctx, rng, world);
    }
    // `:82143-82171` — the 555-557 DD2 kobold death (sites 64-66).
    if matches!(npc_type, 555 | 556 | 557) && npc.hp <= 0 {
        dd2_kobold_death(npc, npc_type, hit_direction, gore, dust, dust_ctx, rng, world);
    }
    // `:82172-82218` — the 551 DD2 crystal: the CheckProgress gate on the
    // HIT path (returned), the death arm's 10-gore body (sites 67-76, the
    // `spriteDirection != -1` arm ONLY). KEEP-ONE (research/304): the `-1`
    // flipped twin (`:82206-82215`) is f10's `betsy_flipped_death` — its
    // gate (`npc_type == 551 && hp <= 0 && sprite_dir == -1`) is the exact
    // complement, so the shared 20-dust preamble runs exactly once.
    if npc_type == 551 {
        // `:82174-82177` — the percent-crossing gate (netMode != 1 runs).
        if dd2_crystal_crossed_percent(npc.hp, npc.max_hp, dmg) {
            outcome.dd2_crystal_check_progress = true;
        }
        if npc.hp <= 0 && npc.sprite_dir != -1 {
            dd2_crystal_death(npc, hit_direction, gore, dust, dust_ctx, rng, world);
        }
    }
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

    /// A boss-shaped stand-in (the wave-A precedent).
    fn boss(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 100, 0.0);
        n.size_override = Some((w, h));
        n.hp = 0;
        n.scale = 1.0;
        n
    }

    /// `NPC.cs:81942-81949` — the 8-gore tail: types + anchors in source
    /// order, velocity ×0.6 + the NewGore kicks.
    #[test]
    fn w4_e5_the_empress_tail_spawns_the_eight_gores_in_order() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.vx = 2.0;
        n.vy = -1.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(13);
        crate::npc_hit_effect_a::empress_tail_gores(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..8].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1263, 1264, 1265, 1266, 1267, 1268, 1267, 1268]);
        let anchors = [
            (110.0, 140.0), (150.0, 120.0), (90.0, 80.0), (100.0, 100.0),
            (80.0, 80.0), (80.0, 110.0), (160.0, 80.0), (160.0, 110.0),
        ];
        for (g, (x, y)) in gore.slots()[..8].iter().zip(anchors) {
            assert_eq!(g.position, Vec2::new(x, y));
        }
        // The twin: 8 × the two NewGore kicks, nothing else.
        let mut twin = UnifiedRandom::new(13);
        for _ in 0..8 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the tail draws only the kicks");
    }

    /// `NPC.cs:45121-45124` — the colour draws the INNER amount first,
    /// then the outer; the lerp truncates per channel.
    #[test]
    fn w4_e5_the_queen_slime_colour_draws_inner_then_outer() {
        let mut rng = UnifiedRandom::new(21);
        let c = queen_slime_dust_color(&mut rng);
        let mut twin = UnifiedRandom::new(21);
        let t_in = twin.next_float();
        let t_out = twin.next_float();
        assert_eq!(rng.peek(), twin.peek(), "exactly two NextFloat draws");
        let inner = lerp_color(
            Color::new(200, 200, 200, 255), Color::new(255, 80, 255, 255), t_in,
        );
        assert_eq!(c, lerp_color(Color::new(0, 160, 255, 255), inner, t_out));
        // t=0/1 endpoints: inner(0)=(200,200,200), outer(0)=(0,160,255).
        assert_eq!(
            lerp_color(Color::new(0, 160, 255, 255), Color::new(200, 200, 200, 255), 0.0),
            Color::new(0, 160, 255, 255)
        );
    }

    /// `NPC.cs:81956-81972` — the 657 death: 3 gores FIRST, then the
    /// 150-dust burst; per dust 2 colour + 2 direction + 1 scale + the
    /// pool's 6 default-block rolls + the Next(5) gate = 12 draws.
    #[test]
    fn w4_e5_the_657_death_runs_gores_then_the_dust_burst() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.vx = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        queen_slime_657_death(&n, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1259, 1259, 1258]);
        assert_eq!(gore.slot(0).scale, 0.8);
        assert_eq!(gore.slot(1).position, Vec2::new(120.0, 120.0)); // + Size*0.5
        assert_eq!(gore.slot(2).scale, 1.0);
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 140, "the 150-burst spawned: {dust_active}");
        // The twin replay: 3×2 kicks, then 150×(2+2+1+6+1).
        let mut twin = UnifiedRandom::new(17);
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        for _ in 0..150 {
            let _cin = twin.next_float();
            let _cout = twin.next_float();
            let _sx = twin.next_float_direction();
            let _sy = twin.next_float_direction();
            let _sc = twin.next_float();
            let _px = twin.next_max(56); // width+20-4
            let _py = twin.next_max(56);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _gate = twin.next_max(5);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 657 arm mirrors the C# draws");
    }

    /// `NPC.cs:82012-82022` — the 660 death: 2×1260 then the 40-dust loop
    /// (type 4, colour (150,100,200,150), alpha 80, scale 1.5); per dust
    /// the pool's 6 rolls + the Next(3) gate.
    #[test]
    fn w4_e5_the_660_death_spawns_two_1260_then_the_dust() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        crystal_658_660(&n, 660, -1, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert_eq!(gore.slot(0).gore_type, 1260);
        assert_eq!(gore.slot(1).gore_type, 1260);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        let mut twin = UnifiedRandom::new(19);
        for _ in 0..2 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        for _ in 0..40 {
            let _px = twin.next_max(26);
            let _py = twin.next_max(26);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _gate = twin.next_max(3);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 660 arm mirrors the C# draws");
        // The 658/659 deaths spawn NO gores (the dust loop only).
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(19);
        crystal_658_660(&n, 659, -1, &mut gore2, &mut dust2, &ctx(), &mut rng2, &w);
        assert!(!gore2.slot(0).active);
        assert_eq!(dust2.slot(0).dust_type, 243); // the 659 type row
    }

    /// `NPC.cs:82035-82055` — the 667 death: 8×1218 with the velocity
    /// OVERWRITE (kicks drawn then discarded), then the 40-dust loop.
    #[test]
    fn w4_e5_the_667_death_overwrites_the_gore_velocities() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        skeleton_667_death(&n, 1, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(23);
        for i in 0..8 {
            let g = gore.slot(i);
            assert_eq!(g.gore_type, 1218);
            assert_eq!(g.position, Vec2::new(100.0, 120.0)); // Center.Y-10
            let _ky = twin.next_range(10, 31); // the discarded kicks
            let _kx = twin.next_range(-20, 21);
            let vx = twin.next_range(1, 10) as f32 * 0.3 * 2.5;
            let vy = -(3.0 + twin.next_max(4) as f32 * 0.3);
            assert_eq!(g.velocity, Vec2::new(vx, vy), "gore {i} overwrite");
        }
        for _ in 0..40 {
            let _px = twin.next_max(36);
            let _py = twin.next_max(56);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _gate = twin.next_max(3);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 667 arm mirrors the C# draws");
    }

    /// `NPC.cs:82090-82107` — the 552-554 goblin death: the 20-dust type-5
    /// preamble then the 5-gore base row (983/1067/1071), the +3/+2 at
    /// Y+20 and the +1 twins at Y+34.
    #[test]
    fn w4_e5_the_dd2_goblin_death_rows_match_the_csharp() {
        let w = air();
        for (ty, base) in [(552, 983), (553, 1067), (554, 1071)] {
            let n = boss(100.0, 100.0, 30.0, 40.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(29);
            dd2_goblin_death(&n, ty, -1, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![base, base + 3, base + 2, base + 1, base + 1]);
            let ys: Vec<f32> = gore.slots()[..5].iter().map(|g| g.position.y).collect();
            assert_eq!(ys, vec![100.0, 120.0, 120.0, 134.0, 134.0]);
            // The twin: 20×6 dust rolls then 5×2 kicks.
            let mut twin = UnifiedRandom::new(29);
            for _ in 0..20 {
                let _px = twin.next_max(26);
                let _py = twin.next_max(36);
                let _vx = twin.next_range(-20, 21);
                let _vy = twin.next_range(-20, 21);
                let _row = twin.next_max(3);
                let _scale = twin.next_range(-20, 21);
            }
            for _ in 0..5 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "goblin {ty} mirrors the C# draws");
        }
    }

    /// `NPC.cs:82121-82140` — the 561-563 javeliner death: the +3 LEADS
    /// (the goblin order differs), base 987/1055/1061.
    #[test]
    fn w4_e5_the_dd2_javeliner_death_leads_with_plus_three() {
        let w = air();
        for (ty, base) in [(561, 987), (562, 1055), (563, 1061)] {
            let n = boss(100.0, 100.0, 30.0, 40.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(31);
            dd2_javeliner_death(&n, ty, 1, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base + 3, base, base + 4, base + 5, base + 2, base + 1, base + 1]
            );
            let ys: Vec<f32> = gore.slots()[..7].iter().map(|g| g.position.y).collect();
            assert_eq!(ys, vec![100.0, 120.0, 120.0, 120.0, 120.0, 134.0, 134.0]);
            let mut twin = UnifiedRandom::new(31);
            for _ in 0..20 {
                let _px = twin.next_max(26);
                let _py = twin.next_max(36);
                let _vx = twin.next_range(-20, 21);
                let _vy = twin.next_range(-20, 21);
                let _row = twin.next_max(3);
                let _scale = twin.next_range(-20, 21);
            }
            for _ in 0..7 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "javeliner {ty} mirrors the C# draws");
        }
    }

    /// `NPC.cs:82154-82169` — the 555-557 kobold death: base 1031/1034/
    /// 1037, the 3-gore staircase (X,Y)/(X+10,Y+20)/(X+10,Y+30).
    #[test]
    fn w4_e5_the_dd2_kobold_death_rows_match_the_csharp() {
        let w = air();
        for (ty, base) in [(555, 1031), (556, 1034), (557, 1037)] {
            let n = boss(100.0, 100.0, 30.0, 40.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            dd2_kobold_death(&n, ty, 1, &mut gore, &mut dust, &ctx(), &mut rng, &w);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![base, base + 1, base + 2]);
            let pos: Vec<Vec2> = gore.slots()[..3].iter().map(|g| g.position).collect();
            assert_eq!(
                pos,
                vec![
                    Vec2::new(100.0, 100.0),
                    Vec2::new(110.0, 120.0),
                    Vec2::new(110.0, 130.0),
                ]
            );
            let mut twin = UnifiedRandom::new(37);
            for _ in 0..20 {
                let _px = twin.next_max(26);
                let _py = twin.next_max(36);
                let _vx = twin.next_range(-20, 21);
                let _vy = twin.next_range(-20, 21);
                let _row = twin.next_max(3);
                let _scale = twin.next_range(-20, 21);
            }
            for _ in 0..3 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "kobold {ty} mirrors the C# draws");
        }
    }

    /// `NPC.cs:82174` — the percent-crossing gate: the float-100 cast on
    /// the before side, the double/(float)lifeMax on the after side, the
    /// life > 0 term.
    #[test]
    fn w4_e5_the_crystal_percent_gate_matches_the_csharp() {
        // life 100/200 = 50%, dmg 1 → 50.5% → 50 — no crossing.
        assert!(!dd2_crystal_crossed_percent(100, 200, 1.0));
        // life 100/200 = 50%, dmg 10 → 55% — crossed.
        assert!(dd2_crystal_crossed_percent(100, 200, 10.0));
        // life 0 — the gate never fires (the death path).
        assert!(!dd2_crystal_crossed_percent(0, 200, 50.0));
        // life 1/3 = 33.33% → 33; dmg 1 → 66.67% → 66 — crossed.
        assert!(dd2_crystal_crossed_percent(1, 3, 1.0));
    }

    /// `NPC.cs:82187-82202` — the 551 crystal death (!= -1 arm): the
    /// 20-dust preamble then the 10-gore body at the cited offsets.
    #[test]
    fn w4_e5_the_crystal_death_spawns_the_ten_gore_body() {
        let w = air();
        let mut n = boss(100.0, 100.0, 200.0, 120.0);
        n.sprite_dir = 1;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        dd2_crystal_death(&n, 1, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..10].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1079, 1080, 1080, 1081, 1082, 1083, 1084, 1085, 1085, 1086]
        );
        let offsets = [
            (0.0, 54.0), (72.0, 90.0), (102.0, 94.0), (178.0, 0.0),
            (136.0, 12.0), (82.0, 18.0), (100.0, 26.0), (170.0, 16.0),
            (190.0, 10.0), (100.0, 36.0),
        ];
        for (g, (dx, dy)) in gore.slots()[..10].iter().zip(offsets) {
            assert_eq!(g.position, Vec2::new(100.0 + dx, 100.0 + dy));
        }
        let mut twin = UnifiedRandom::new(41);
        for _ in 0..20 {
            let _px = twin.next_max(196);
            let _py = twin.next_max(116);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..10 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the crystal mirrors the C# draws");
    }

    /// KEEP-ONE (research/304): the 551 `-1` flipped death (`NPC.cs:82204-
    /// 82216`) is f10's `betsy_flipped_death` — e5's dispatch spawns
    /// NOTHING for it (no gores AND no dust: the shared 20-dust preamble
    /// rides f10's copy for `== -1`, so the merge runs it exactly once).
    #[test]
    fn w4_e5_the_flipped_crystal_death_is_f10s_claim() {
        let w = air();
        let mut n = boss(100.0, 100.0, 200.0, 120.0);
        n.sprite_dir = -1;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        let out = crate::npc_hit_effect_a::hit_effect(
            &mut n, 551, &crate::npc_hit_effect_a::HitEffectInputs::default(),
            -1, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w,
        );
        assert_eq!(gore.slots().iter().filter(|g| g.active).count(), 0);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 0);
        assert!(!out.dd2_crystal_check_progress); // the death path
        let mut twin = UnifiedRandom::new(43);
        assert_eq!(rng.peek(), twin.peek(), "the -1 death draws nothing in e5");
        // The != -1 twin still runs e5's full body (10 gores + 20 dusts).
        let mut n2 = boss(100.0, 100.0, 200.0, 120.0);
        n2.sprite_dir = 1;
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(43);
        crate::npc_hit_effect_a::hit_effect(
            &mut n2, 551, &crate::npc_hit_effect_a::HitEffectInputs::default(),
            -1, 10.0, &mut gore2, &mut dust2, &ctx(), &mut rng2, &w,
        );
        assert_eq!(gore2.slots().iter().filter(|g| g.active).count(), 10);
        let mut twin2 = UnifiedRandom::new(43);
        for _ in 0..20 {
            let _px = twin2.next_max(196);
            let _py = twin2.next_max(116);
            let _vx = twin2.next_range(-20, 21);
            let _vy = twin2.next_range(-20, 21);
            let _row = twin2.next_max(3);
            let _scale = twin2.next_range(-20, 21);
        }
        for _ in 0..10 {
            let _ky = twin2.next_range(10, 31);
            let _kx = twin2.next_range(-20, 21);
        }
        assert_eq!(rng2.peek(), twin2.peek(), "the != -1 body keeps its census");
    }

    /// The wave-B dispatch on a dead 553 goblin through wave A's
    /// `hit_effect`: the wave-A arms stay off, the goblin row runs, and
    /// the CheckProgress seam stays false (the death path).
    #[test]
    fn w4_e5_the_dispatch_routes_the_goblin_through_wave_a() {
        let w = air();
        let mut n = boss(100.0, 100.0, 30.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let out = crate::npc_hit_effect_a::hit_effect(
            &mut n, 553, &crate::npc_hit_effect_a::HitEffectInputs::default(),
            -1, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w,
        );
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1067, 1070, 1069, 1068, 1068]);
        assert!(!out.dd2_crystal_check_progress);
        assert!(out.daybreak_add_buff.is_none());
        assert!(!out.unlock_slime_purple);
        // A living 551 crossing a percent returns the CheckProgress seam.
        let mut crystal = boss(100.0, 100.0, 200.0, 120.0);
        crystal.hp = 100;
        crystal.max_hp = 200;
        let out2 = crate::npc_hit_effect_a::hit_effect(
            &mut crystal, 551, &crate::npc_hit_effect_a::HitEffectInputs::default(),
            1, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w,
        );
        assert!(out2.dd2_crystal_check_progress);
    }
}
