//! `NPC.HitEffect` — the per-NPC gore/dust table, wave B: the NEXT 40
//! `Gore.NewGore*` death sites in source order (`NPC.cs:81942-82240`),
//! continuing wave A (`npc_hit_effect_a.rs`, W4-d55, ec78d152).
//!
//! Wave B is appended to wave A's dispatch: [`hit_effect_b`] runs right
//! after wave A's last arm (`npc_hit_effect_a.rs::hit_effect`), the same
//! `this.type` blocks in source order. The RNG contract is wave A's — the
//! strike caller's stream (the `StrikeNPC_Inner` `:81488` swap), every
//! draw cited per site.
//!
//! ## Scope — sites 31-70 (source order)
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
//! | (77-86) | `:82206`-`:82215` | the 551 `-1` twin (10, wave B commit 4) |
//! | (87-91) | `:82237`-`:82241` | the 572/573 DD2 flyer death (5, commit 4) |
//!
//! Everything after `:82241` (574/575 onward, the 566/567 `Next(2)` gates,
//! the 576/577 pair, …) is wave C (`W4-f10-gore-sites-3`).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * The hit halves (`life > 0`): the dmg-scaled dust loops
//!   (`dmg / lifeMax * 20`, the 657 12-dust loop, the 658-660/667 7-dust
//!   loops) have NO gore sites and stay with the wiring wave's dust pass
//!   — wave A's convention. Exception: the 551 `DD2Event.CheckProgress`
//!   gate (`:82174-82177`) fires on the HIT path and is RETURNED
//!   (`HitEffectOutcome::dd2_crystal_check_progress`).
//! * The death dust loops that share an arm with gore sites RUN here on
//!   the pool (the 657 150-loop, the 658-660/667 count loops, the DD2
//!   20-dust type-5 loops) — the C# order: 657 gores BEFORE the dust,
//!   660 gores BEFORE the dust, 667 gores BEFORE the dust, DD2 dust
//!   BEFORE the gores. The 636 dust loop keeps wave A's draw-census skip
//!   (`empress_hit_dust_draws`).
//! * `hitDirection` rides an explicit parameter (the C# `HitEffect(int
//!   hitDirection = 0, double dmg = 10.0)`, `:81654`).
//! * The 572/573 sound stops (`:82230-82231`) are RETURNED
//!   (`dd2_goblin_sound_stops`) — the tracked-sound seam is the wiring
//!   wave's.
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

/// The 667 (Dark Caster?) death/hit (`NPC.cs:82025-82057`, sites 44-51):
/// the death arm spawns 8×1218 at `(position.X, Center.Y-10)` with
/// velocity Zero, then OVERWRITES the slot velocity — `Next(1,10)*0.3*
/// 2.5*hitDirection` X, `-(3+Next(4)*0.3)` Y (the kicks are drawn inside
/// `NewGore` first, then discarded by the overwrite). The hit arm is the
/// 3-gore twin (`×2`, `-(2.5+…)`, `:82045-82046`) — wave C. The dust loop
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

/// The wave-B dispatch tail — appended to wave A's `hit_effect` in source
/// order (`NPC.cs:81942` onward). `npc.hp` is `this.life` at the call.
/// Commit 1 scope: the 636 tail, 657, 658-660, 667.
pub(crate) fn hit_effect_b(
    npc: &mut Npc,
    npc_type: i32,
    hit_direction: i32,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    outcome: &mut HitEffectOutcome,
) {
    let _ = outcome; // the sound/check-progress seams arrive with commit 4
    // `:81952-81990` — the 657 Queen Slime death (sites 39-41).
    if npc_type == 657 && npc.hp <= 0 {
        queen_slime_657_death(npc, gore, dust, dust_ctx, rng, world);
    }
    // `:81991-82024` — the 658-660 crystal family (sites 42-43).
    if (658..=660).contains(&npc_type) {
        crystal_658_660(npc, npc_type, hit_direction, gore, dust, dust_ctx, rng, world);
    }
    // `:82025-82057` — the 667 death (sites 44-51); the hit arm's 3-gore
    // twin is wave C, its dust loop rides the wiring wave's dust pass.
    if npc_type == 667 && npc.hp <= 0 {
        skeleton_667_death(npc, hit_direction, gore, dust, dust_ctx, rng, world);
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
}
