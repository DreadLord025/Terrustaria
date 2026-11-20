//! W4-g60 — the NPC death dust/effects lane 2: `NPC.HitEffect`'s per-kind
//! DEATH halves for the vanilla types 231-697 (g59 carries the first
//! half, 1-230), drawn when the death retain picks the corpse up.
//!
//! # The seam (the W4-c12 `style74_death_feed` precedent)
//!
//! The C# runs `HitEffect(hitDirection, num)` inside `StrikeNPC_Inner`
//! (`NPC.cs:81488`) on the striking stage's stream, BEFORE `checkDead`'s
//! DeathSound (`:81559` → `:78186`). The port batches deaths into the
//! NPC pass (`world_sim/npcs.rs`'s death retain), so every draw below
//! rides the `UpdateNPCs` stream — the S1-E-21c recorded deviation (the
//! death-cue precedent). The feed runs BEFORE the death cue, after the
//! moon-event kill reports (neither draws the RNG), under the same
//! carrier guard as the death events (`NPC.cs:78040-78042`'s checkDead
//! guard; the C# HitEffect itself runs per struck segment — the batch
//! model folds the segment draws onto the carrier's death, documented
//! in ANALYSIS). The `life <= 0` halves only: the `life > 0` hit halves
//! stay with the unported strike-side HitEffect (the g62 lane).
//!
//! # Eligibility (the g59 census convention)
//!
//! A kind rides this lane when EVERY top-level arm covering it has a
//! dust-bearing death half, all dust draws precede the gores
//! (`dust_first`), and no side effects (`NewNPC`/`Transform`/`PlaySound`/
//! `AddBuff`/`NewProjectile`/`netMode`/...) sit in the death half. 121
//! ported kinds qualify in 231-697; 388/418 are W4-c12's in-flight death
//! feed (kept out — the merge reconciles), and the 663 Princess's second
//! arm (`NPC.cs:88411`) is DEAD CODE: the else-if chain's town arm
//! (`:88046`) matches 663 first, so only the 25-dust smoke burst fires.
//! The 402 Stardust Worm Head's `oldPos` trail loop (`:83611-83631`)
//! reads the C#'s 10-slot trail array (`NPC.cs:6119`) the port's Npc
//! does not carry: an all-zero array draws each slot's `Next(4)` gate
//! until the first `== 0` breaks — the port replays exactly that (the
//! documented stand-in; the live trail is the wiring wave's).
//!
//! # RNG contract (d50's DustPool, S1-E-1/S1-E-4)
//!
//! Every draw hits the caller's stream in C# statement order: the
//! explicit `Main.rand` argument draws FIRST (C# evaluates arguments
//! before the call), then the `NewDust` internals roll inside
//! `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn writes
//! (which read `Main.dust[6000]` when the pool refuses — the sentinel
//! slot exists and takes the writes exactly like the C#). The gore
//! BODIES are NOT drawn here (the d55/e5/f10 gore lane owns `NewGore`):
//! where a gore call carries ARGUMENT draws (`Main.rand.Next(...)` in
//! the call, incl. the conditional-gore gates), those draws ARE consumed
//! here in C# order — the e2 lane's split — so the stream stays
//! faithful; the gore lane must NOT re-draw them (the site comments
//! cite each consumed argument). Constant-argument gore calls draw
//! nothing outside the pool. The `SoundEngine...Stop()` calls
//! (572-575, `:82230-82231`/`:82255`) draw nothing.
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled
//! by the merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{Npc, NpcKind};

/// The per-death inputs the C# reads off `this` — snapshotted by the
/// death retain before the pool borrows. `hit_dir` is the recorded
/// strike side (`Npc::last_hit_dir`, the W4-c12 field; 0 = no strike /
/// the argument-less `HitEffect()` fuses, `NPC.cs:81654`'s default).
pub(crate) struct DeathFx<'a> {
    /// The vanilla `NPC.type` (the Sharkron2 fold rides the wrapper's
    /// explicit 373 — `cs_id` folds it onto the bestiary's 372 row).
    pub ty: u16,
    pub pos: Vec2,
    pub w: i32,
    pub h: i32,
    pub vel: Vec2,
    pub hit_dir: i32,
    pub rng: &'a mut UnifiedRandom,
    pub dust: &'a mut DustPool,
    pub ctx: &'a SpawnCtx,
    pub world: &'a World,
}

impl DeathFx<'_> {
    /// One `Dust.NewDust(position, width, height, type, sx, sy)` — the
    /// 6-argument default form (alpha 0, default Color, scale 1).
    fn d(&mut self, ty: i32, sx: f32, sy: f32) -> usize {
        self.dust.new_dust(
            self.rng, self.ctx, self.world, self.pos, self.w, self.h, ty, sx, sy, 0,
            Color::default(), 1.0,
        )
    }

    /// The full-argument form.
    #[allow(clippy::too_many_arguments)]
    fn d_full(
        &mut self, pos: Vec2, w: i32, h: i32, ty: i32, sx: f32, sy: f32,
        alpha: i32, color: Color, scale: f32,
    ) -> usize {
        self.dust.new_dust(
            self.rng, self.ctx, self.world, pos, w, h, ty, sx, sy, alpha, color, scale,
        )
    }

    /// The flat N-dust burst at `base.position`, speeds `(sx, sy)`.
    fn burst(&mut self, n: i32, ty: i32, sx: f32, sy: f32) {
        for _ in 0..n {
            self.d(ty, sx, sy);
        }
    }

    /// The tower-mob pair loop: `n` iterations of one `base_ty` dust at
    /// `(hitDirection, -1)` then a gated `second_ty` dust. `gate_mod`
    /// 0 = unconditional; else `Next(gate_mod) == 0`. `full` writes
    /// noGravity/scale 1.5/fadeIn 1/velocity ×3 (the 241/6 + 249/242 +
    /// 241/229 shape); the stardust 17/229 loop has its own lighter
    /// shape ([`Self::stardust_loop`]).
    fn pair_loop(&mut self, n: i32, gate_mod: i32, base_ty: i32, second_ty: i32) {
        let hd = self.hit_dir as f32;
        for _ in 0..n {
            self.d(base_ty, hd, -1.0);
            if gate_mod == 0 || self.rng.next_max(gate_mod) == 0 {
                let i = self.d(second_ty, 0.0, 0.0);
                let d = self.dust.slot_mut(i);
                d.no_gravity = true;
                d.scale = 1.5;
                d.fade_in = 1.0;
                d.velocity *= 3.0;
            }
        }
    }

    /// The stardust 17/229 pair loop (`NPC.cs:83496-83502` shape): the
    /// GATE applies to the 17 (base) dust — `gate_mod` 0 = always,
    /// else `Next(gate_mod) == 0` XOR `gate_ne` (407's `!= 0` form) —
    /// and the 229 ALWAYS spawns with noGravity + velocity ×3 ONLY.
    fn stardust_loop(&mut self, n: i32, gate_mod: i32, gate_ne: bool) {
        let hd = self.hit_dir as f32;
        for _ in 0..n {
            let spawn17 = if gate_mod == 0 {
                true
            } else {
                (self.rng.next_max(gate_mod) == 0) != gate_ne
            };
            if spawn17 {
                self.d(17, hd, -1.0);
            }
            let i = self.d(229, 0.0, 0.0);
            let d = self.dust.slot_mut(i);
            d.no_gravity = true;
            d.velocity *= 3.0;
        }
    }

    /// The 54-smoke loop (250/315, `NPC.cs:88971-88976`): `n` × type-54
    /// at (0,0), alpha 50, scale 1.5, then velocity ×2 + noGravity.
    fn smoke54(&mut self, n: i32) {
        for _ in 0..n {
            let i = self.d_full(self.pos, self.w, self.h, 54, 0.0, 0.0, 50, Color::default(), 1.5);
            let d = self.dust.slot_mut(i);
            d.velocity *= 2.0;
            d.no_gravity = true;
        }
    }

    /// The 226 scale-0.5 loop (the martian machines): `n` × type-226 at
    /// `(-k × hitDirection, -k)`, post-spawn scale = 0.5.
    fn burst226(&mut self, n: i32, k: f32) {
        let hd = self.hit_dir as f32;
        for _ in 0..n {
            let i = self.d(226, -k * hd, -k);
            self.dust.slot_mut(i).scale = 0.5;
        }
    }
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): the unit draw
/// (`NextVector2Unit`, `:1261-1264` — ONE NextFloat, float angle, double
/// cos/sin) THEN a second `NextFloat` radius scale. The
/// `npc_ai_empress.rs` helper's shape, kept local (that one is private).
fn circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
    let r = rng.next_float();
    Vec2::new(unit.x * hw * r, unit.y * hh * r)
}

/// The death retain's entry: snapshot the corpse, run the kind's death
/// half on the `UpdateNPCs` stream. Kinds outside the lane no-op (their
/// HitEffect stays the documented visual skip / the sibling lanes').
pub(crate) fn death_dust_feed(app: &mut crate::app::App, npc: &Npc) {
    // The vanilla type: the Sharkron2 fold is explicit (the style71
    // adapter's precedent — `cs_id`'s bestiary fallback folds 373 onto
    // the Sharkron 372 row, and the shared arm covers both).
    let ty = match npc.kind {
        NpcKind::Sharkron2 => 373,
        _ => npc.kind.cs_id(),
    };
    if !lane_covers(ty) {
        return;
    }
    let (w, h) = npc.size();
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdateNpcs);
    let mut fx = DeathFx {
        ty,
        pos: Vec2::new(npc.x, npc.y),
        w: w as i32,
        h: h as i32,
        vel: Vec2::new(npc.vx, npc.vy),
        hit_dir: npc.last_hit_dir,
        rng,
        dust: &mut app.dust,
        ctx: &ctx,
        world: &app.world,
    };
    death_dust_draws(&mut fx);
}

/// The lane's kind gate (the census's eligible set, minus c12's
/// 388/418). Kept as a table so the wrapper can skip the snapshot for
/// the ~570 uncovered types.
fn lane_covers(ty: u16) -> bool {
    matches!(
        ty,
        668 | 552..=554 | 561..=563 | 555..=557 | 551 | 558..=560 | 566 | 567
            | 576 | 577 | 564 | 565 | 572..=575 | 570 | 571 | 568 | 569 | 550 | 441
            | 588 | 589 | 633 | 368 | 353 | 354 | 369 | 376 | 663 | 342
            | 338..=340 | 343 | 348 | 349 | 350 | 351 | 489 | 490
    )
}

/// The per-kind death halves, in C# source order. Every arm cites its
/// `NPC.cs` window; the dust draws roll the pool, the gore ARGUMENT
/// draws are consumed in place (the module-header split).
pub(crate) fn death_dust_draws(fx: &mut DeathFx) {
    let hd = fx.hit_dir as f32;
    match fx.ty {
        // `NPC.cs:81892-81909` — Deerclops: 120 type-5 dusts from
        // `position - (15,15)`, box `(width+30, height+30)`, scale 1.5,
        // then velocity = `NextVector2Circular(4,4)` (TWO draws AFTER
        // the spawn internals). The seven gores (`:81902-81908`) are
        // constant-argument — the gore lane's, zero draws here.
        668 => {
            let pos = fx.pos - Vec2::new(15.0, 15.0);
            let (w, h) = (fx.w + 30, fx.h + 30);
            for _ in 0..120 {
                let i = fx.d_full(pos, w, h, 5, 0.0, 0.0, 0, Color::default(), 1.5);
                let v = circular(fx.rng, 4.0, 4.0);
                fx.dust.slot_mut(i).velocity = v;
            }
        }
        // `NPC.cs:82088-82108` — the DD2 Goblin trio: 20 type-5 at
        // `(hitDirection, -1)` (the death half keeps the 1× side, NOT
        // the hit half's). The five gores (`:82103-82107`, the
        // 983/1067/1071 tier rows) are constant-argument — zero draws.
        552..=554 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82119-82141` — the DD2 Javelin trio: the same 20×5;
        // the seven gores (`:82134-82140`, 987/1055/1061) draw nothing.
        561..=563 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82152-82170` — the DD2 Bomber trio: the tier pick
        // (`:82154-82162`) precedes the loop in C# but draws nothing;
        // 20×5 then three constant gores.
        555..=557 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82185-82217` — DD2 Betsy: 20×5 at `(hitDirection,-1)`;
        // the ten-gore fan is spriteDirection-branched (`:82191`/
        // `:82204`) but BOTH branches are constant-argument — zero
        // draws outside the dust loop.
        551 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82323-82345` — the DD2 Wyvern trio: 20×5; the seven
        // gores (974/1040/1045 tier rows) are constant-argument.
        558..=560 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82356-82387` — the DD2 Skeleton pair: 20×236, then
        // gore 970 (constant) and SIX `Next(2) == 0`-gated gores
        // (`:82363-82386`: 972×2, 973×2, 971×2). The gate draws are
        // ARGUMENT-side conditional spawns — consumed here in order
        // (the gore bodies stay the gore lane's; it must NOT re-draw
        // the six gates).
        566 | 567 => {
            fx.burst(20, 236, hd, -1.0);
            for _ in 0..6 {
                // `:82363`/`:82367`/`:82371`/`:82375`/`:82379`/`:82383`
                // — each gate draws unconditionally; == 0 spawns the
                // gore (the gore lane reads the same stream position).
                fx.rng.next_max(2);
            }
        }
        // `NPC.cs:82398-82424` — the DD2 Ogre pair: 20×273 (a
        // DangerousDust id — the ChildSafety re-type rolls INSIDE
        // new_dust when armed), then the spriteDirection-branched fan:
        // BOTH branches' first gore carries `1005 + ((Next(5) == 0) ?
        // 1 : 0)` (`:82406`/`:82417`) — ONE Next(5) argument draw
        // consumed here; the remaining five gores are constant. The
        // left branch's two `rotation = PI` writes are gore-side.
        576 | 577 => {
            fx.burst(20, 273, hd, -1.0);
            fx.rng.next_max(5);
        }
        // `NPC.cs:82435-82453` — the DD2 Dark Mage pair: 20×5; the
        // eleven gores (1009..1019) are constant-argument.
        564 | 565 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82228-82242` — the DD2 Kobold Walker pair: the two
        // `SoundEngine...Stop()` calls (`:82230-82231`) draw nothing;
        // 20×5 then five constant gores at Center-(10,10).
        572 | 573 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82253-82267` — the DD2 Kobold Flyer pair: one Stop
        // (`:82255`, no draws); 20×5 then five constant gores.
        574 | 575 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82278-82290` — the DD2 Drakin pair: 20×5; the five
        // gores (1075/979 tier rows) are constant-argument.
        570 | 571 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82301-82312` — the DD2 Wither Beast pair: 20×5; the
        // five gores (1020-1023) are constant-argument.
        568 | 569 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:87952-87960` — the DD2 Bartender: 50×5 at
        // `(2.5×hitDirection, -2.5)`; the five gores are
        // IsShimmerVariant-conditional but BOTH arms constant.
        550 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:87685-87693` — the Tax Collector: the same 50×5; the
        // five shimmer-conditional gores are constant both ways.
        441 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:87972-87983` — the Golfer pair (588/589): 50×5; the
        // altTexture==0 head gore + the four body gores are
        // shimmer-conditional but constant both ways — zero draws.
        588 | 589 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:87995-88021` — the Zoologist: 50×5; the altTexture/
        // shimmer gore picks (`:87999-88015`) are data, not draws.
        633 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:88033-88044` — the Travelling Merchant: 50×5; the
        // altTexture/shimmer gores are constant both ways.
        368 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:88208-88220` — the Stylist pair (353/354): 50×5; the
        // altTexture/shimmer gores are constant both ways.
        353 | 354 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:88048-88074` — the town smoke arm (369 Angler / 376
        // Sleeping Angler / 663 Princess; the IsTownPet/695/696 rows
        // have no port kinds yet): `life <= 0` gates the WHOLE arm;
        // 25 type-31 dusts at (0,0), alpha 100, scale 2, then velocity
        // ×1.4 + noLight + noGravity. The four tombstone gores
        // (`:88058-88073`) each carry `Main.rand.Next(61, 64)` — FOUR
        // argument draws consumed here in order (the gore bodies +
        // their scale/velocity±1 writes stay the gore lane's). The
        // 663's second arm (`:88411`) is dead code — this chain arm
        // matches 663 first (the else-if at `:88046`).
        369 | 376 | 663 => {
            for _ in 0..25 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 31, 0.0, 0.0, 100, Color::default(), 2.0);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 1.4;
                d.no_light = true;
                d.no_gravity = true;
            }
            for _ in 0..4 {
                // `:88058`/`:88062`/`:88066`/`:88070` — the tombstone
                // type pick, argument-side.
                fx.rng.next_range(61, 64);
            }
        }
        // `NPC.cs:86858-86866` — the Gingerbread Man: 50×type-0 at
        // `(2.5×hitDirection, -2.5)`; five constant gores.
        342 => fx.burst(50, 0, 2.5 * hd, -2.5),
        // `NPC.cs:86878-86896` — the Zombie Elf trio: 50×5; the
        // per-type head gore (494/495/496) + three body gores are
        // constant-argument.
        338..=340 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:86908-86915` — the Yeti: 50×5; four constant gores.
        343 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:86322-86330` — the Nutcracker pair (348/349): 10×5
        // at `(2×hitDirection, -2)`; five constant gores (528-532).
        348 | 349 => fx.burst(10, 5, 2.0 * hd, -2.0),
        // `NPC.cs:86362-86369` — the Elf Archer: 10×5; four constant
        // gores (533-536).
        350 => fx.burst(10, 5, 2.0 * hd, -2.0),
        // `NPC.cs:86342-86350` — the Krampus: 10×5; five constant
        // gores (538-542).
        351 => fx.burst(10, 5, 2.0 * hd, -2.0),
        // `NPC.cs:86669-86688` — the Blood Zombie (489; the 586
        // Crimslime shares the arm but has no port kind): 50×5; the
        // type-gated five gores (722-724) are constant-argument.
        489 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:86720-86728` — the Drippler: 50×5, then gores
        // 725/726/727 (constant) and TWO `725 + Main.rand.Next(3)`
        // wing picks (`:86727-86728`) — TWO argument draws consumed
        // here in order.
        490 => {
            fx.burst(50, 5, 2.5 * hd, -2.5);
            fx.rng.next_max(3);
            fx.rng.next_max(3);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On-screen ctx: the (100,100) box sits inside the 400-px cull
    /// margin; maxDustToDraw 6000 puts the early slots in the low band
    /// (no gate draw) — the style56 witness convention.
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

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// The low-band accepted-spawn draw replay: pos X, pos Y, vel X,
    /// vel Y, frame row, scale (`Dust.cs:170-190`), no per-type tweak
    /// for the plain types (5/241/249/17/229/226...). Returns the six
    /// rolls.
    fn replay_spawn(r: &mut UnifiedRandom, w: i32, h: i32) -> [i32; 6] {
        [
            r.next_max(w.max(5) - 4),
            r.next_max(h.max(5) - 4),
            r.next_range(-20, 21),
            r.next_range(-20, 21),
            r.next_max(3),
            r.next_range(-20, 21),
        ]
    }

    /// `NPC.cs:82090-82093` — the DD2 Goblin trio's death half: exactly
    /// 20 type-5 dusts at `(hitDirection, -1)`, six internals each, no
    /// other draws (the five gores are constant-argument). The stream
    /// replay pins the count AND the order.
    #[test]
    fn w4_g60_the_dd2_goblin_burst_replays_20_six_draw_spawns() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(777);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 552,
            pos: Vec2::new(100.0, 100.0),
            w: 18,
            h: 38,
            vel: Vec2::new(2.0, -1.0),
            hit_dir: 1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 18, 38);
        }
        assert_eq!(rng, r, "20 spawns × 6 internals, nothing else");
        for i in 0..20 {
            let d = pool.slot(i);
            assert!(d.active);
            assert_eq!(d.dust_type, 5);
        }
        assert!(!pool.slot(20).active, "exactly 20");
        // The velocity carries the recorded side: roll ×0.1 + hit_dir.
        let d0 = pool.slot(0);
        assert!(d0.velocity.x > 0.0, "hit_dir 1 rides SpeedX");
    }

    /// `NPC.cs:81894-81900` — Deerclops: 120 spawns from
    /// `position - (15,15)` with the +30 box, scale 1.5, then the
    /// Circular(4,4) velocity OVERWRITE (two NextFloat draws AFTER the
    /// six internals). The witness replays the first two iterations
    /// and pins the box/scale/velocity shape.
    #[test]
    fn w4_g60_the_deerclops_burst_replays_the_circular_overwrite() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(42);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 668,
            pos: Vec2::new(200.0, 200.0),
            w: 100,
            h: 120,
            vel: Vec2::ZERO,
            hit_dir: -1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..2 {
            let rolls = replay_spawn(&mut r, 130, 150);
            // Circular: unit NextFloat then radius NextFloat.
            let a = std::f32::consts::PI * 2.0 * r.next_float();
            let rad = r.next_float();
            let vx = (a as f64).cos() as f32 * 4.0 * rad;
            let vy = (a as f64).sin() as f32 * 4.0 * rad;
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 5);
            assert_eq!(d.position.x, 185.0 + rolls[0] as f32 + 4.0);
            assert_eq!(d.position.y, 185.0 + rolls[1] as f32 + 4.0);
            assert_eq!(d.velocity, Vec2::new(vx, vy), "the overwrite, not the roll");
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.5);
        }
        // All 120 landed (the low band accepts every slot).
        assert!(pool.slot(119).active);
        assert!(!pool.slot(120).active);
        // The full-stream replay: 120 × (6 + 2).
        let mut r2 = peek;
        for _ in 0..120 {
            replay_spawn(&mut r2, 130, 150);
            r2.next_float();
            r2.next_float();
        }
        assert_eq!(rng, r2, "120 × (6 internals + 2 circular), nothing else");
    }

    /// `NPC.cs:82358-82386` — the DD2 Skeleton pair: 20×236 (six
    /// internals each — 236 hits no tweak arm) then SIX unconditional
    /// `Next(2)` gore gates consumed in order. The gore bodies are the
    /// gore lane's; this witness pins the argument-side consumption.
    #[test]
    fn w4_g60_the_dd2_skeleton_consumes_the_six_gore_gates() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 566,
            pos: Vec2::new(100.0, 100.0),
            w: 26,
            h: 44,
            vel: Vec2::ZERO,
            hit_dir: -1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 26, 44);
        }
        for _ in 0..6 {
            r.next_max(2);
        }
        assert_eq!(rng, r, "20×6 internals + 6 gore gates");
        assert_eq!(pool.slot(0).dust_type, 236);
        assert!(pool.slot(19).active);
        assert!(!pool.slot(20).active);
    }

    /// `NPC.cs:82400-82423` — the DD2 Ogre pair: 20×273 then ONE
    /// `Next(5)` (the 1005/1006 head-gore pick, both facing branches).
    /// 273 is a DangerousDust id but ChildSafety defaults DISABLED
    /// (`ChildSafety.cs:33`) — no re-type draws in the default pool.
    #[test]
    fn w4_g60_the_dd2_ogre_consumes_the_head_gore_pick() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(32);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 576,
            pos: Vec2::new(100.0, 100.0),
            w: 60,
            h: 84,
            vel: Vec2::ZERO,
            hit_dir: 1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 60, 84);
        }
        r.next_max(5);
        assert_eq!(rng, r, "20×6 internals + 1 head-gore pick");
        assert_eq!(pool.slot(0).dust_type, 273, "no ChildSafety re-type by default");
    }

    /// `NPC.cs:88050-88073` — the town smoke arm: 25 type-31 dusts
    /// (alpha 100, scale 2, velocity ×1.4, noLight, noGravity) then
    /// FOUR `Next(61,64)` tombstone argument draws. The 663 Princess
    /// rides THIS arm — the `:88411` second arm is dead code.
    #[test]
    fn w4_g60_the_town_smoke_consumes_the_four_tombstone_picks() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(55);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 663,
            pos: Vec2::new(100.0, 100.0),
            w: 18,
            h: 40,
            vel: Vec2::ZERO,
            hit_dir: 1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..2 {
            let rolls = replay_spawn(&mut r, 18, 40);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 31);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 2.0);
            assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 * 1.4);
            assert!(d.no_light && d.no_gravity);
        }
        for _ in 2..25 {
            replay_spawn(&mut r, 18, 40);
        }
        for _ in 0..4 {
            r.next_range(61, 64);
        }
        assert_eq!(rng, r, "25×6 internals + 4 tombstone picks");
        assert!(pool.slot(24).active);
        assert!(!pool.slot(25).active);
    }

    /// `NPC.cs:86720-86728` — the Drippler: 50×5 then TWO `Next(3)`
    /// wing-gore picks (725+roll), argument-side.
    #[test]
    fn w4_g60_the_drippler_consumes_the_two_wing_picks() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 490,
            pos: Vec2::new(100.0, 100.0),
            w: 26,
            h: 26,
            vel: Vec2::ZERO,
            hit_dir: -1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..50 {
            replay_spawn(&mut r, 26, 26);
        }
        r.next_max(3);
        r.next_max(3);
        assert_eq!(rng, r, "50×6 internals + 2 wing picks");
        assert!(pool.slot(49).active);
        assert!(!pool.slot(50).active);
    }

    /// The lane gate: 388/418 stay c12's, uncovered kinds no-op with
    /// ZERO draws (the wrapper's early return).
    #[test]
    fn w4_g60_uncovered_kinds_draw_nothing() {
        assert!(lane_covers(668));
        assert!(lane_covers(552));
        assert!(!lane_covers(388), "c12's feed");
        assert!(!lane_covers(418), "c12's feed");
        assert!(!lane_covers(1), "g59's half");
        assert!(!lane_covers(400), "no arm");
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 400,
            pos: Vec2::new(100.0, 100.0),
            w: 20,
            h: 20,
            vel: Vec2::ZERO,
            hit_dir: 1,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        assert_eq!(rng, peek, "the default arm draws nothing");
        assert!(!pool.slot(0).active);
    }
}
