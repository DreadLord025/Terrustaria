//! Player-side `Gore.NewGore*` sites, wave D: the death confetti + the
//! item-use visuals (`Player.cs:38869-45027`). WIRED (W4-g17 wiring
//! wave): the KillMe confetti rides `world_sim::npcs`' death seam (the
//! tenth-anniversary seed gate, the `UpdatePlayers` stream); the
//! item-use arms (65 / 3065 / meleeEnchant-7 / 1304) ride
//! `world_sim::editing::item_use_visuals_tick` — the per-tick ItemCheck
//! drain resolves the melee rect through the shared `melee_swing` ports
//! and runs the gates in C# order on the same stream. The dust-only
//! item arms (989/2880/44-46/273/723/190/213/…) and the other enchant
//! rows stay the dust lane's (documented).
//!
//! ## Scope — this commit's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1 | `:38884` | `KillMe` tenth-anniversary — 40× Next(276,283) |
//! | 2 | `:44804` | `ItemCheck_EmitUseVisuals` type 65 — Next(16,18) |
//! | 3 | `:44813` | type 3065 — gore 16 |
//! | 4 | `:45001` | meleeEnchant 7 — Next(276,283) |
//! | 5 | `:45026` | item 1304 (ChildSafety) — Next(3,6) |
//!
//! ## RNG contract (the player tick's `Main.rand` stream)
//!
//! * Death confetti: the `tenthAnniversaryWorld` gate is the caller's.
//!   85 dusts FIRST — per dust the type `Next(139,143)` (argument),
//!   the internals (6, no tweak), then 7 post rolls (X/Y adds, X/Y
//!   multipliers, X/Y `*0.05` adds, scale `Next(-30,31)`). Then 40
//!   gores — per gore the type `Next(276,283)`, kicks (2), then 7
//!   post rolls (X/Y adds, X/Y multipliers, scale `Next(-20,21)`,
//!   X/Y `*0.05` adds).
//! * Type 65: the `Next(5)` dust gate ALWAYS draws (type 58, no
//!   tweak); the `Next(10)` gore gate ALWAYS draws; when it hits,
//!   `Next(16,18)` type + kicks (the 16/17 arm writes alpha/scale/
//!   light, NO draws, `Gore.cs:1361-1366`).
//! * Type 3065: one type-58 dust ALWAYS (velocity ×0.5, no draw);
//!   the `Next(8)` gate ALWAYS draws; when it hits, gore 16 + kicks,
//!   velocity ×0.5 + `(direction, 0)`.
//! * meleeEnchant 7: the `Next(20)` dust gate ALWAYS draws (type
//!   `Next(139,143)` + 6 internals + 5 post rolls when it hits); the
//!   `Next(40)` gore gate ALWAYS draws; when it hits, type
//!   `Next(276,283)` + kicks + 5 post rolls (X/Y multipliers, scale,
//!   X/Y `*0.05` adds).
//! * Item 1304: the `Next(60)` gate ALWAYS draws; when it hits (and
//!   ChildSafety.Disabled), the velocity's TWO `NextFloat`s ride
//!   BEFORE the `Next(3,6)` type (C# argument order: Position,
//!   Velocity, Type), then kicks.
//!
//! The post-spawn writes land on the returned slot, the sentinel
//! included (the `npc_hit_effect_a.rs` convention).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `KillMe`'s drop/coin/tombstone/respawn pipeline is `death.rs`'s —
//!   this module runs ONLY the confetti visual (`:38869-38893`).
//! * The `ItemCheck_EmitUseVisuals` item-type gates, the melee
//!   `!noMelee && !noUseGraphic` flags and `meleeEnchant > 0` are the
//!   caller's; `itemRectangle` rides the state (the swing hitbox the
//!   item lane computes). `ChildSafety.Disabled` defaults true
//!   (`ChildSafety.cs:33`); the 1304 gate reads it.
//! * The dust `shader` writes are dropped fields (the pool's
//!   convention). netMode: single-player port.
//!
//! See docs/ANALYSIS.md "The player gore sites (W4-g17, wave D)".
// W4-g17 wiring: the confetti + the four item-use arms are live.

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The player snapshot the death confetti reads.
#[derive(Clone, Copy, Debug)]
pub struct DeathConfettiState {
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

/// `Player.KillMe`'s tenth-anniversary confetti (`Player.cs:38869-38893`):
/// the 85 type-`Next(139,143)` dusts then the 40 gores (site 1), types
/// `Next(276,283)` at the death spot with velocity `(0, -10)`.
pub fn death_confetti_gores(
    state: &DeathConfettiState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:38871-38881` — the 85 dusts.
    for _ in 0..85 {
        let ty = rng.next_range(139, 143);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(state.x, state.y),
            state.width, state.height, ty, 0.0, -10.0, 0, Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
        d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
    }
    // `:38882-38892` — the 40 gores.
    for _ in 0..40 {
        let ty = rng.next_range(276, 283);
        let idx = gore.new_gore(
            rng, world, Vec2::new(state.x, state.y), Vec2::new(0.0, -10.0), ty, 1.0,
        );
        let g = gore.slot_mut(idx);
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
    }
}

/// The item-use snapshot (`ItemCheck_EmitUseVisuals`, `:44754-45046`).
#[derive(Clone, Copy, Debug)]
pub struct ItemUseGoreState {
    /// `itemRectangle` top-left (`Rectangle.X`/`.Y`).
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_w: i32,
    pub rect_h: i32,
    /// `base.direction` / `base.velocity`.
    pub direction: i32,
    pub vx: f32,
    pub vy: f32,
}

impl ItemUseGoreState {
    fn top_left(&self) -> Vec2 {
        Vec2::new(self.rect_x, self.rect_y)
    }
    fn center(&self) -> Vec2 {
        Vec2::new(
            self.rect_x + self.rect_w as f32 / 2.0,
            self.rect_y + self.rect_h as f32 / 2.0,
        )
    }
}

/// `ItemCheck_EmitUseVisuals`' type-65 arm (`Player.cs:44796-44806`,
/// site 2): the `Next(5)` type-58 dust gate then the `Next(10)` gore
/// gate, type `Next(16,18)` at the rect top-left.
pub fn item65_use_gore(
    state: &ItemUseGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:44798-44801` — the dust gate ALWAYS draws.
    if rng.next_max(5) == 0 {
        dust.new_dust(
            rng, ctx, world, state.top_left(), state.rect_w, state.rect_h,
            58, 0.0, 0.0, 150, Color::default(), 1.2,
        );
    }
    // `:44802-44805` — the gore gate ALWAYS draws.
    if rng.next_max(10) == 0 {
        let ty = rng.next_range(16, 18);
        gore.new_gore(rng, world, state.top_left(), Vec2::ZERO, ty, 1.0);
    }
}

/// `ItemCheck_EmitUseVisuals`' type-3065 arm (`Player.cs:44807-44817`,
/// site 3): one type-58 dust (velocity ×0.5) then the `Next(8)` gore
/// gate, gore 16 at the rect centre, velocity ×0.5 + `(direction, 0)`.
pub fn item3065_use_gore(
    state: &ItemUseGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:44809-44810` — the dust ALWAYS spawns.
    let idx = dust.new_dust(
        rng, ctx, world, state.top_left(), state.rect_w, state.rect_h,
        58, 0.0, 0.0, 150, Color::default(), 1.2,
    );
    dust.slot_mut(idx).velocity *= 0.5;
    // `:44811-44816` — the gore gate ALWAYS draws.
    if rng.next_max(8) == 0 {
        let idx = gore.new_gore(rng, world, state.center(), Vec2::ZERO, 16, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.5;
        g.velocity += Vec2::new(state.direction as f32, 0.0);
    }
}

/// `ItemCheck_EmitUseVisuals`' meleeEnchant-7 arm (`Player.cs:44988-45008`,
/// site 4): the `Next(20)` confetti-dust gate then the `Next(40)` gore
/// gate, type `Next(276,283)` at the rect top-left with the player vel.
/// The caller owns the `melee && !noMelee && !noUseGraphic &&
/// meleeEnchant == 7` gate.
pub fn enchant7_use_gore(
    state: &ItemUseGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:44990-44998` — the dust gate ALWAYS draws.
    if rng.next_max(20) == 0 {
        let ty = rng.next_range(139, 143);
        let idx = dust.new_dust(
            rng, ctx, world, state.top_left(), state.rect_w, state.rect_h,
            ty, state.vx, state.vy, 0, Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
        d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
    }
    // `:44999-45007` — the gore gate ALWAYS draws.
    if rng.next_max(40) == 0 {
        let ty = rng.next_range(276, 283);
        let idx = gore.new_gore(
            rng, world, state.top_left(), Vec2::new(state.vx, state.vy), ty, 1.0,
        );
        let g = gore.slot_mut(idx);
        g.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
    }
}

/// `ItemCheck_EmitUseVisuals`' item-1304 arm (`Player.cs:45024-45027`,
/// site 5): the `Next(60)` gate ALWAYS draws; when it hits AND
/// ChildSafety is disabled, the velocity's two `NextFloat`s ride
/// BEFORE the `Next(3,6)` type, at the rect centre.
pub fn item1304_use_gore(
    state: &ItemUseGoreState,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    child_safety_disabled: bool,
) {
    // `:45024` — the gate ALWAYS draws; ChildSafety gates the spawn.
    if rng.next_max(60) == 0 && child_safety_disabled {
        // `:45026` — argument order: Velocity.X NextFloat, Velocity.Y
        // NextFloat, then Type Next(3,6).
        let vx = state.direction as f32 * (2.0 + rng.next_float() * 3.0);
        let vy = -2.0 - rng.next_float() * 3.0;
        let ty = rng.next_range(3, 6);
        gore.new_gore(rng, world, state.center(), Vec2::new(vx, vy), ty, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn dust_block(r: &mut UnifiedRandom, w: i32, h: i32) {
        let _px = r.next_max(w - 4);
        let _py = r.next_max(h - 4);
        let _vx = r.next_range(-20, 21);
        let _vy = r.next_range(-20, 21);
        let _row = r.next_max(3);
        let _scale = r.next_range(-20, 21);
    }

    /// `Player.cs:38871-38892` — the confetti: 85×(type + 6 + 7) dust
    /// draws then 40×(type + 2 kicks + 7) gore draws.
    #[test]
    fn w4_g17_the_death_confetti_draws_eighty_five_dusts_then_forty_gores() {
        let w = air();
        let s = DeathConfettiState { x: 100.0, y: 200.0, width: 20, height: 42 };
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        death_confetti_gores(&s, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        for _ in 0..85 {
            let _ty = twin.next_range(139, 143);
            dust_block(&mut twin, 20, 42);
            for _ in 0..6 {
                let _r = twin.next_range(-50, 51);
            }
            let _sc = twin.next_range(-30, 31);
        }
        for _ in 0..40 {
            let _ty = twin.next_range(276, 283);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            for _ in 0..4 {
                let _r = twin.next_range(-50, 51);
            }
            let _sc = twin.next_range(-20, 21);
            for _ in 0..2 {
                let _r = twin.next_range(-50, 51);
            }
        }
        assert_eq!(rng.peek(), twin.peek(), "the confetti mirrors the C# draws");
        let active: Vec<&tr_core::gore::Gore> =
            gore.slots().iter().filter(|g| g.active).collect();
        assert_eq!(active.len(), 40);
        assert!(active.iter().all(|g| (276..=282).contains(&g.gore_type)));
    }

    /// `Player.cs:44796-44817` — the type-65 / 3065 gates ALWAYS draw;
    /// a both-miss seed consumes exactly the gate rolls.
    #[test]
    fn w4_g17_the_item_use_gates_always_draw() {
        let w = air();
        let s = ItemUseGoreState {
            rect_x: 100.0, rect_y: 200.0, rect_w: 32, rect_h: 32,
            direction: 1, vx: 2.0, vy: -3.0,
        };
        // type 65: Next(5) then Next(10). Find a both-miss seed.
        let mut miss = None;
        for seed in 0..500 {
            let mut p = UnifiedRandom::new(seed);
            if p.next_max(5) != 0 && p.next_max(10) != 0 {
                miss = Some(seed);
                break;
            }
        }
        let m = miss.expect("a both-miss seed");
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(m);
        item65_use_gore(&s, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(m);
        let _g1 = twin.next_max(5);
        let _g2 = twin.next_max(10);
        assert_eq!(rng.peek(), twin.peek(), "two gate draws only");
        assert!(!gore.slots().iter().any(|g| g.active));
        // type 3065: one dust (6 draws) + Next(8). A miss seed draws 7.
        let mut miss8 = None;
        for seed in 0..500 {
            let mut p = UnifiedRandom::new(seed);
            dust_block(&mut p, 32, 32);
            if p.next_max(8) != 0 {
                miss8 = Some(seed);
                break;
            }
        }
        let m8 = miss8.expect("a miss-8 seed");
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(m8);
        item3065_use_gore(&s, &mut gore2, &mut dust2, &ctx(), &mut rng2, &w);
        let mut twin2 = UnifiedRandom::new(m8);
        dust_block(&mut twin2, 32, 32);
        let _g = twin2.next_max(8);
        assert_eq!(rng2.peek(), twin2.peek(), "dust + gate only");
        assert!(!gore2.slots().iter().any(|g| g.active));
    }

    /// `Player.cs:45024-45027` — the 1304 gate ALWAYS draws; the
    /// velocity floats ride BEFORE the type; ChildSafety gates spawn.
    #[test]
    fn w4_g17_the_1304_gate_draws_and_the_floats_precede_the_type() {
        let w = air();
        let s = ItemUseGoreState {
            rect_x: 100.0, rect_y: 200.0, rect_w: 32, rect_h: 32,
            direction: 1, vx: 0.0, vy: 0.0,
        };
        // A gate-hit seed.
        let mut hit = None;
        for seed in 0..500 {
            let mut p = UnifiedRandom::new(seed);
            if p.next_max(60) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let h = hit.expect("a gate-hit seed");
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(h);
        item1304_use_gore(&s, &mut gore, &mut rng, &w, true);
        let mut twin = UnifiedRandom::new(h);
        let _g = twin.next_max(60);
        let _fx = twin.next_float();
        let _fy = twin.next_float();
        let ty = twin.next_range(3, 6);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "the 1304 arm mirrors");
        assert_eq!(gore.slot(0).gore_type, ty);
        assert!((3..=5).contains(&gore.slot(0).gore_type));
        // ChildSafety armed (disabled=false) refuses the spawn but the
        // gate still drew.
        let mut gore2 = GorePool::new();
        let mut rng2 = UnifiedRandom::new(h);
        item1304_use_gore(&s, &mut gore2, &mut rng2, &w, false);
        assert!(!gore2.slot(0).active);
        let mut twin2 = UnifiedRandom::new(h);
        let _g2 = twin2.next_max(60);
        assert_eq!(rng2.peek(), twin2.peek(), "gate only when safety armed");
    }
}
