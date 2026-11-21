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
//! reads the LIVE 10-slot trail ring (`Npc.old_pos`, `NPC.cs:6119`):
//! `Npc::update_trails` ports `UpdateNPC_UpdateTrails`'s mode-7 shift
//! (`NPC.cs:90863-90872`, called at `:90512` after `FindFrame`) — the
//! W4-g60 wiring wave retired the all-zero stand-in. Only a
//! never-updated corpse still reads an all-zero ring (the gates draw
//! until the first `== 0` breaks, exactly the C#'s fresh-NPC shape).
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
//! # Keep-one census vs the d55/e5/f10 gore dispatchers (wiring wave)
//!
//! The gore lanes' `hit_effect`/`hit_effect_b`/`hit_effect_c` re-run
//! death DUST loops for kinds this lane already covers. When their
//! dispatch claims the strike/death slot, the wiring MUST strip those
//! loops for exactly the overlap kinds (keep this feed, keep the gore
//! BODIES — one draw per C# statement):
//! * d55 wave A: 668 (`deerclops_death`'s 120-burst — g60 owns it; the
//!   7 gores stay d55's, constant-argument). 636 is NOT in this lane
//!   (census-ineligible, hit half + gore mix): d55/e5 own it whole —
//!   their `empress_hit_dust_draws` count-plan must become a pool feed
//!   at their wiring, not a second census.
//! * e5 wave B: 551/552-554/555-557/561-563 (`dd2_death_dust`'s 20×5
//!   preamble — g60 owns it; the gore bodies stay e5's). 657/660/667
//!   are NOT in this lane (gore-FIRST arms): e5 owns them whole.
//! * f10 wave C: 558-560/564-565/566-567/568-571/572-575/576-577
//!   (`death_dust_loop`) + 412-417/419/518/416 (the solar pair loops)
//!   — g60 owns the dust AND the consumed argument draws (the 566/567
//!   six `Next(2)` gates, the 576/577 `Next(5)` head pick — f10 re-draws
//!   them today; strip at wiring). 418 stays c12's feed. The e5 551
//!   mirrored twin vs the f10 551 flipped arm ALSO overlap each other
//!   (same `spriteDirection == -1` branch) — keep one there too.
//!
//! # Gore-remainder re-census (the 66 GORE-UNIMPL rows, research/110 §11)
//!
//! The playbook's blanket `GORE-UNIMPL` for the 66 g60-owned remainder
//! kinds was probe-time-stale (f10 tip `a69fb00c`, sites ≤444). The
//! re-read: f10's pushed tip `67c51ec7` now covers `NPC.cs:82206-85334`
//! (batches 24-60 landed the `:83562-85559` continuation — the martian
//! machines 385-387/389/391/399/520, the saucer family 392-395, the
//! pumpkin family 305-314/325/327/344), and f5's wave-D
//! claim (`97c70c97`, tip `b33ba0a9`) owns the WHOLE
//! `NPC.cs:85560-89835` classic-mob chain — gore bodies AND the dust
//! census ("this module ports the death-half dust loops ONLY as the
//! gore bodies' draw census … keep-one at merge"). Of the 66: **24
//! kinds are f10-owned (gore pushed), 32 f5-owned (11 pushed + 21
//! claimed), 10 have NO gore sites at all** (261/265/288/326/330/352/
//! 384/402/410/533 — dust-only arms no gore lane carries because there
//! is nothing to carry; g60's feed is their whole port).
//! **ZERO gore bodies are unclaimed — g60 ports NO gore bodies here**;
//! the ARG draws stay consumed by this feed, exactly once. The strip
//! surface grows: f10's pushed continuation adds 24 overlap kinds
//! (305-314/325/327/344/385-387/389/391/392-395/399/520), f5's wave-D
//! claim adds 32 (the 11 PUSHED re-draws 338-340/342/343/348-351/489/
//! 490 — f5 batches 14/15/24/25/31/32 — + 21 claimed-not-pushed arms
//! past `:86986`). Per-kind table: research/110 §11.
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
    /// `NPC.oldPos` (`NPC.cs:6119`) — the LIVE trail ring the 402 death
    /// arm's loop reads (`:83611-83631`); `Npc::update_trails` shifts it
    /// per NPC update (the W4-g60 wiring wave — the all-zero stand-in is
    /// retired, only a never-updated corpse still reads all-zero).
    pub old_pos: [Vec2; 10],
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
    // `NPC.cs:87222-87225` — the Spore's gate is `life >= 0` (NOT the
    // usual `life > 0`): at exactly life == 0 the arm returns with NO
    // dust. The death retain picks hp <= 0, so the hp == 0 edge skips.
    if ty == 265 && npc.hp == 0 {
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
        old_pos: npc.old_pos,
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
            | 338..=340 | 343 | 348 | 349 | 350 | 351 | 489 | 490 | 305..=314
            | 352 | 327 | 325 | 326 | 330 | 329 | 315 | 250 | 261 | 288
            | 344 | 262..=264 | 265 | 266 | 267 | 370 | 372 | 373
            | 439 | 381 | 382 | 390 | 383 | 384 | 385 | 386 | 387 | 389
            | 391 | 392..=395 | 399 | 520 | 341 | 471 | 533
            | 518 | 416 | 419 | 412..=414 | 415 | 417 | 427 | 425
            | 428 | 424 | 423 | 421 | 420 | 409 | 410 | 407 | 411 | 402
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
        // `NPC.cs:84804-84815` — the Scarecrow decade (305-314): 60×
        // type-194 at `(2×hitDirection, -2)`, scale 1.1; the five
        // gores (441-445) are constant-argument.
        305..=314 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 194, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:84786-84793` — Flocko: 60×67 at `(2×hitDirection,
        // -2)`, post-spawn noGravity; no gores.
        352 => {
            for _ in 0..60 {
                let i = fx.d(67, 2.0 * hd, -2.0);
                fx.dust.slot_mut(i).no_gravity = true;
            }
        }
        // `NPC.cs:84826-84836` — Pumpking: 60×189 scale 1.1; the nine
        // gores (476-484, velocity ×0) are constant-argument.
        327 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 189, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:84847-84857` — Mourning Wood: 60×191 scale 1.1; the
        // seven gores (469-475, velocity ×0) are constant-argument.
        325 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 191, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:84901-84907` — Splinterling: 60×191 scale 1.1; no
        // gores.
        326 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 191, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:88994-89001` — Poltergeist (330; the 316 Pirate
        // ghost shares the arm, no port kind): 50×192 at `(2×hd, -2)`,
        // alpha 100, Color(100,100,100,100), post-spawn noGravity.
        330 => {
            for _ in 0..50 {
                let i = fx.d_full(
                    fx.pos, fx.w, fx.h, 192, 2.0 * hd, -2.0, 100,
                    Color::new(100, 100, 100, 100), 1.0,
                );
                fx.dust.slot_mut(i).no_gravity = true;
            }
        }
        // `NPC.cs:87745-87751` — Hellhound: 50×5; three constant gores.
        329 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:89038-89051` — the Headless Horseman: the 20×54
        // smoke loop, then three gore-99 calls + gore 446 — ALL
        // constant-argument (`(hitDirection, 0)` velocities).
        315 => fx.smoke54(20),
        // `NPC.cs:88971-88982` — Angry Nimbus (250; the 82/158/159/253
        // rows share the arm, g59's half / no port kinds): the 20×54
        // smoke loop; three constant gore-99 calls.
        250 => fx.smoke54(20),
        // `NPC.cs:89065-89079` — Fungi Spore (261; the 254-260/634/635
        // spore rows share the arm, no port kinds): 20×165 at (0,0),
        // alpha 50, scale 1.5, velocity ×2 + noGravity. The three
        // gores are `type != 261`-gated (`:89071`) — 261 spawns NONE.
        261 => {
            for _ in 0..20 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 165, 0.0, 0.0, 50, Color::default(), 1.5);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:85000-85009` — the Dungeon Spirit's death burst (the
        // `npc_ai_style56.rs` cited skip, now drawn): `life <= 0`
        // condition-gated arm; 50×180 at `(velocity.X, velocity.Y)`,
        // then velocity ×2 + noGravity + scale = 1.4 (OVERWRITE — the
        // rolled scale is consumed but replaced).
        288 => {
            for _ in 0..50 {
                let i = fx.d(180, fx.vel.x, fx.vel.y);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.scale = 1.4;
            }
        }
        // `NPC.cs:84876-84890` — Everscream: 60×196 at `(2×hd, -2)`;
        // the two gore loops (508-512, 509-512, velocity ×0) are
        // constant-argument.
        344 => fx.burst(60, 196, 2.0 * hd, -2.0),
        // `NPC.cs:87186-87218` — the Plantera family (262/263/264):
        // 150 iterations; the `type == 265 && Next(3) != 0` gate is
        // FALSE for every family type (265 short-circuits — NO gate
        // draw), so all 150 are type-167 at `(2×hd, -2)`. The 262's
        // nine gores carry `Next(width)`/`Next(height)` ARGUMENT pairs
        // (`:87199-87207`) — 18 draws consumed here; the 263/264 pairs
        // (`:87211-87217`) — 4 each. The gore bodies stay the lane's.
        262..=264 => {
            fx.burst(150, 167, 2.0 * hd, -2.0);
            let n_gores = match fx.ty {
                262 => 9,
                _ => 2,
            };
            for _ in 0..n_gores {
                fx.rng.next_max(fx.w);
                fx.rng.next_max(fx.h);
            }
        }
        // `NPC.cs:87226-87236` — the Spore: `life >= 0` returns (the
        // death half is life < 0); 15 iterations of `Next(3) != 0` →
        // type-166 else type-167, at `(hitDirection, -1)`. The gate
        // draws EVERY iteration (the `npc_ai_style50.rs` count plan's
        // draws, now emitted).
        265 => {
            for _ in 0..15 {
                if fx.rng.next_max(3) != 0 {
                    fx.d(166, hd, -1.0);
                } else {
                    fx.d(167, hd, -1.0);
                }
            }
        }
        // `NPC.cs:87248-87257` — the Brain of Cthulhu: 150×5 at
        // `(2×hd, -2)`; the six gores (396-401) each carry TWO
        // `Next(-30,31)*0.2` velocity ARGUMENT draws (`:87252-87257`)
        // — 12 draws consumed here in order.
        266 => {
            fx.burst(150, 5, 2.0 * hd, -2.0);
            for _ in 0..6 {
                fx.rng.next_range(-30, 31);
                fx.rng.next_range(-30, 31);
            }
        }
        // `NPC.cs:87269-87273` — the Creeper: 20×5 at `(2×hd, -2)`;
        // one constant gore (402).
        267 => fx.burst(20, 5, 2.0 * hd, -2.0),
        // `NPC.cs:84267-84279` — Duke Fishron: 150×5 at `(2×hd, -2)`;
        // the six gores (576/574/575/573/574/575) are constant-
        // argument (the direction offsets are data).
        370 => fx.burst(150, 5, 2.0 * hd, -2.0),
        // `NPC.cs:84216-84227` — the Sharkron pair (372/373): 75×5
        // from `Center - (25,25)`, box 50×50, at `(2×hd, -2)`, then
        // velocity /= 2 (the `npc_ai_style71.rs` death plan's draws).
        // The four gores (583/577/578/579) are constant-argument.
        372 | 373 => {
            let (cx, cy) = (fx.pos.x + fx.w as f32 * 0.5, fx.pos.y + fx.h as f32 * 0.5);
            let pos = Vec2::new(cx - 25.0, cy - 25.0);
            for _ in 0..75 {
                let i = fx.d_full(pos, 50, 50, 5, 2.0 * hd, -2.0, 0, Color::default(), 1.0);
                fx.dust.slot_mut(i).velocity /= 2.0;
            }
        }
        // `NPC.cs:84138-84159` — the Ancient Cultist: 30×5 at
        // `(2.5×hd, -2.5)`, then 10×16 at (0,0) scale 1.5 with
        // velocity ×2 + noGravity. The hood-gore loop: ONE `Next(1,4)`
        // count draw (`:84150`), then per gore ONE `Next(11,14)` type
        // pick (`:84153`) — the count gates how many picks are drawn;
        // the velocity `(hitDirection, 0)` + the ×0.8 write are
        // argument-free/gore-side. The four body gores (902/903/901×2)
        // are constant.
        439 => {
            fx.burst(30, 5, 2.5 * hd, -2.5);
            for _ in 0..10 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 16, 0.0, 0.0, 0, Color::default(), 1.5);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            let n = fx.rng.next_range(1, 4);
            for _ in 0..n {
                fx.rng.next_range(11, 14);
            }
        }
        // `NPC.cs:84290-84300` — Brain Scrambler: 30×227 at
        // `(2.5×hd, -2.5)`; four constant gores (587/590/588/589).
        381 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84311-84320` — Ray Gunner / Scutlix Rider (382/390):
        // 30×227; three constant gores (591-593).
        382 | 390 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84331-84340` — Martian Officer: 30×227; three
        // constant gores (594-596).
        383 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84353-84360` — Force Bubble: 70×226 at
        // `(-2.5×hd, -2.5)`, scale = 0.5 OVERWRITE; no gores.
        384 => fx.burst226(70, 2.5),
        // `NPC.cs:84371-84380` — Gray Grunt: 30×227; three constant
        // gores (597-599).
        385 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84391-84400` — Martian Engineer: 30×227; three
        // constant gores (600-602).
        386 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84413-84423` — Martian Turret (Tesla): 30×226 at
        // `(-1.5×hd, -1.5)`, scale = 0.5; three constant gores
        // (603×2/604).
        387 => fx.burst226(30, 1.5),
        // `NPC.cs:84459-84468` — Giga Zapper: 30×227; three constant
        // gores (904-906).
        389 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84543-84552` — Scutlix: 30×227; three constant gores
        // (607/608 at position + width/2, 609).
        391 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84565-84590` — the Martian Saucer family (392/393/
        // 394/395): 30×228 at `(-1.5×hd, -1.5)`, scale = 0.5 OVERWRITE
        // (228's NewDust tweak — vy re-roll, vx ×0.3, scale ×0.7 —
        // rolls INSIDE new_dust BEFORE the overwrite). The 395's six
        // gores / 393's 616 / 394's 617 are constant-argument; the
        // base 392 spawns no gores.
        392..=395 => {
            let hd2 = fx.hit_dir as f32;
            for _ in 0..30 {
                let i = fx.d(228, -1.5 * hd2, -1.5);
                fx.dust.slot_mut(i).scale = 0.5;
            }
        }
        // `NPC.cs:84513-84532` — the Martian Probe: 10×226 at
        // `(-2.5×hd, -2.5)`, scale = 0.5; the two gores (898/899) ride
        // the ai[0]==3 implosion velocity (argument-free — the gore
        // lane reads `Npc.ai[0]`/`rotation`/`sprite_dir` itself).
        399 => fx.burst226(10, 2.5),
        // `NPC.cs:84436-84448` — the Martian Walker: 30×226 at
        // `(-1.5×hd, -1.5)`, scale = 0.5; five constant gores
        // (851/852/848/849/850).
        520 => fx.burst226(30, 1.5),
        // `NPC.cs:88755-88764` — the Present Mimic: 20×196 (the
        // `num706 = 196` constant, `:88746`) at (0,0); the three gores
        // (61/62/63) carry `(hitDirection, 0)` velocities — data, no
        // draws; the ×0.3 writes are gore-side.
        341 => fx.burst(20, 196, 0.0, 0.0),
        // `NPC.cs:89617-89625` — the Goblin Summoner: 50×5 at
        // `(2.5×hd, -2.5)`; five constant gores (675-678).
        471 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:83878-83887` — the Desert Djinn: 40×27 at (0,0),
        // alpha 50, scale 1.5, then velocity ×2 + noGravity + fadeIn 1.
        533 => {
            for _ in 0..40 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 27, 0.0, 0.0, 50, Color::default(), 1.5);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 1.0;
            }
        }
        // `NPC.cs:82802-82821` — Solar Spearman / Drakomire Rider
        // (518/416): 20 × (241 + `Next(3)==0`-gated 6 full-write);
        // five constant gores (844/845/847/835/836).
        518 | 416 => fx.pair_loop(20, 3, 241, 6),
        // `NPC.cs:82840-82860` — Solar Selenian: the same 20×(241+6)
        // gated pair; six constant gores (844/845/846×2/847×2).
        419 => fx.pair_loop(20, 3, 241, 6),
        // `NPC.cs:82871-82882` — the Solar Crawltipede family (412/
        // 413/414): 6×241 at `(2.5×hd, -2.5)` then 2×6 at the same
        // speeds, alpha 100, Color.Transparent, scale 2 (NO post
        // writes — the C# passes the full arguments). One gore
        // (`type - 412 + 828`, Scale-default) — constant-argument.
        412..=414 => {
            let hd25 = 2.5 * fx.hit_dir as f32;
            fx.burst(6, 241, hd25, -2.5);
            for _ in 0..2 {
                fx.d_full(fx.pos, fx.w, fx.h, 6, hd25, -2.5, 100, Color::TRANSPARENT, 2.0);
            }
        }
        // `NPC.cs:82901-82916` — Solar Drakomire: 40 × (241 + 6
        // UNCONDITIONAL full-write pair); four constant gores at
        // Center (831/832 ×0.8, 833/834 ×0.9).
        415 => fx.pair_loop(40, 0, 241, 6),
        // `NPC.cs:82935-82957` — Solar Sroller: 30 × (241 +
        // `Next(2)==0`-gated 6); eight constant gores (837/838×3/
        // 839×2/840×2).
        417 => fx.pair_loop(30, 2, 241, 6),
        // `NPC.cs:83103-83123` — Vortex Alien: 10×241 at `(hd, -1)`,
        // then 20 × (`SelectRandom(229,229,240)` — ONE `Next(3)` pick
        // BEFORE the spawn, `Utils.cs:2459-2462` — then noGravity,
        // scale = 1.25 + NextFloat, fadeIn 0.25, velocity ×3, noLight).
        // Four constant gores at Center (802-805).
        427 => {
            let hd1 = fx.hit_dir as f32;
            fx.burst(10, 241, hd1, -1.0);
            for _ in 0..20 {
                let pick = fx.rng.next_max(3);
                let ty = [229, 229, 240][pick as usize];
                let i = fx.d(ty, 0.0, 0.0);
                let extra = fx.rng.next_float();
                let d = fx.dust.slot_mut(i);
                d.no_gravity = true;
                d.scale = 1.25 + extra;
                d.fade_in = 0.25;
                d.velocity *= 3.0;
                d.no_light = true;
            }
        }
        // `NPC.cs:83224-83245` — Vortex Rifleman: 20 × (241 +
        // `Next(3)==0`-gated 229 full-write); seven constant gores
        // (796/797/798/809×2/810×2).
        425 => fx.pair_loop(20, 3, 241, 229),
        // `NPC.cs:83256-83272` — Vortex Headcrab: 10 × (241 + a
        // `Next(2)`-gated EITHER 229 OR 240 — the gate draws, then the
        // branch picks the type, NO post writes); two constant gores
        // (806/807, velocity ×0.8).
        428 => {
            let hd1 = fx.hit_dir as f32;
            for _ in 0..10 {
                fx.d(241, hd1, -1.0);
                if fx.rng.next_max(2) == 0 {
                    fx.d(229, 0.0, 0.0);
                } else {
                    fx.d(240, 0.0, 0.0);
                }
            }
        }
        // `NPC.cs:83291-83310` — Predictor: 20 × (249 +
        // `Next(3)==0`-gated 242 full-write); five constant gores
        // (793/794×2/795×2).
        424 => fx.pair_loop(20, 3, 249, 242),
        // `NPC.cs:83329-83343` — Nebula Beast: 20 × (249 + 242
        // UNCONDITIONAL full-write); three constant Center gores
        // (790/791 ×0.8, 792 ×0.9).
        423 => fx.pair_loop(20, 0, 249, 242),
        // `NPC.cs:83362-83376` — Brain Suckler: 10 × (249 + 242
        // unconditional); three constant Center gores (785-787).
        421 => fx.pair_loop(10, 0, 249, 242),
        // `NPC.cs:83395-83409` — Nebula Floater: 10 × (249 + 242
        // unconditional); three constant Center gores (782-784).
        420 => fx.pair_loop(10, 0, 249, 242),
        // `NPC.cs:83494-83506` — Stardust Spider (big): 20 × (17 +
        // 229 noGravity/×3 — the LIGHT stardust shape); three constant
        // Center gores (775/776 ×0.8, 777 ×0.9).
        409 => fx.stardust_loop(20, 0, false),
        // `NPC.cs:83521-83530` — Stardust Spider (small): 5 × (17 +
        // 229); no gores.
        410 => fx.stardust_loop(5, 0, false),
        // `NPC.cs:83545-83563` — Stardust Jellyfish: 20 × (`Next(3)
        // != 0`-gated 17 + ALWAYS 229); six constant gores (778/779
        // Top ×0.8, 780/781/780/781 Center ×0.9).
        407 => fx.stardust_loop(20, 3, true),
        // `NPC.cs:83647-83662` — Stardust Soldier: 20 × (17 + a
        // `Next(2)==0`-gated 229 with noGravity ONLY — no ×3); five
        // constant gores (772/773×2/774×2).
        411 => {
            let hd1 = fx.hit_dir as f32;
            for _ in 0..20 {
                fx.d(17, hd1, -1.0);
                if fx.rng.next_max(2) == 0 {
                    let i = fx.d(229, 0.0, 0.0);
                    fx.dust.slot_mut(i).no_gravity = true;
                }
            }
        }
        // `NPC.cs:83599-83632` — the Stardust Worm Head: 5 ×
        // (`Next(3)==0`-gated 17 + ALWAYS 229 noGravity/×3), then the
        // oldPos trail loop over the LIVE 10-slot ring (`Npc.old_pos`,
        // `Npc::update_trails`'s mode-7 shift — the W4-g60 wiring wave):
        // each slot draws `Next(4)` FIRST (`:83613`), `!= 0` continues;
        // on `== 0` a zero slot breaks (`:83617-83620`), else TWO gated
        // pairs spawn at `oldPos[i]` (`:83621-83630`, the same 17/229
        // shape as the head burst).
        402 => {
            fx.stardust_loop(5, 3, false);
            for &slot in &fx.old_pos {
                if fx.rng.next_max(4) != 0 {
                    continue; // `:83613-83616`
                }
                if slot == Vec2::ZERO {
                    break; // `:83617-83620` — an unfilled ring slot
                }
                for _ in 0..2 {
                    if fx.rng.next_max(3) == 0 {
                        fx.dust.new_dust(
                            fx.rng, fx.ctx, fx.world, slot, fx.w, fx.h, 17, hd, -1.0,
                            0, Color::default(), 1.0,
                        ); // `:83625`
                    }
                    let i = fx.dust.new_dust(
                        fx.rng, fx.ctx, fx.world, slot, fx.w, fx.h, 229, 0.0, 0.0,
                        0, Color::default(), 1.0,
                    ); // `:83627`
                    let d = fx.dust.slot_mut(i);
                    d.no_gravity = true;
                    d.velocity *= 3.0;
                }
            }
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
            old_pos: [Vec2::ZERO; 10],
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
            old_pos: [Vec2::ZERO; 10],
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
            old_pos: [Vec2::ZERO; 10],
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
            old_pos: [Vec2::ZERO; 10],
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
            old_pos: [Vec2::ZERO; 10],
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
            old_pos: [Vec2::ZERO; 10],
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

    /// `NPC.cs:84806-84809` — the Scarecrow decade: 60×194, scale 1.1
    /// (the rolled scale ×1.1, NOT an overwrite), speeds (2×hd, -2).
    #[test]
    fn w4_g60_the_scarecrow_burst_scales_the_roll() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 305,
            pos: Vec2::new(100.0, 100.0),
            w: 24,
            h: 46,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let rolls = replay_spawn(&mut r, 24, 46);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 194);
        assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.1);
        assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 + 2.0);
        assert_eq!(d.velocity.y, rolls[3] as f32 * 0.1 - 2.0);
        for _ in 1..60 {
            replay_spawn(&mut r, 24, 46);
        }
        assert_eq!(rng, r, "60×6 internals, nothing else");
        assert!(pool.slot(59).active);
        assert!(!pool.slot(60).active);
    }

    /// `NPC.cs:85002-85008` — the Dungeon Spirit death burst: 50×180
    /// at the NPC velocity, then velocity ×2 + noGravity + scale = 1.4
    /// OVERWRITE (the rolled scale is consumed but replaced). The
    /// style56 cited skip, now drawn.
    #[test]
    fn w4_g60_the_spirit_burst_overwrites_the_scale() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(81);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 288,
            pos: Vec2::new(100.0, 100.0),
            w: 20,
            h: 20,
            vel: Vec2::new(3.0, -1.0),
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let rolls = replay_spawn(&mut r, 20, 20);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 180);
        assert_eq!(d.scale, 1.4, "assignment, not the rolled scale");
        assert_eq!(d.velocity.x, (rolls[2] as f32 * 0.1 + 3.0) * 2.0);
        assert_eq!(d.velocity.y, (rolls[3] as f32 * 0.1 - 1.0) * 2.0);
        assert!(d.no_gravity);
        for _ in 1..50 {
            replay_spawn(&mut r, 20, 20);
        }
        assert_eq!(rng, r, "50×6 internals, nothing else");
        assert!(pool.slot(49).active);
        assert!(!pool.slot(50).active);
    }

    /// `NPC.cs:89065-89079` — the Fungi Spore: 20×165 alpha 50 scale
    /// 1.5, velocity ×2 + noGravity; the three gores are type!=261
    /// gated so 261 spawns NONE (no argument draws).
    #[test]
    fn w4_g60_the_fungi_spore_smoke_has_no_gore_draws() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(82);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 261,
            pos: Vec2::new(100.0, 100.0),
            w: 30,
            h: 30,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 30, 30);
        }
        assert_eq!(rng, r, "20×6 internals, no gore draws");
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 165);
        assert_eq!(d.alpha, 50);
        assert!(d.no_gravity);
    }

    /// `NPC.cs:87248-87257` — the Brain of Cthulhu: 150×5 then SIX
    /// gores × TWO `Next(-30,31)` velocity argument draws each.
    #[test]
    fn w4_g60_the_brain_consumes_the_twelve_gore_velocity_draws() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(91);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 266,
            pos: Vec2::new(100.0, 100.0),
            w: 100,
            h: 100,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..150 {
            replay_spawn(&mut r, 100, 100);
        }
        for _ in 0..6 {
            r.next_range(-30, 31);
            r.next_range(-30, 31);
        }
        assert_eq!(rng, r, "150×6 internals + 12 gore velocity draws");
        assert!(pool.slot(149).active);
        assert!(!pool.slot(150).active);
    }

    /// `NPC.cs:87226-87236` — the Spore: 15 × (gate + spawn); the
    /// `Next(3) != 0` gate draws EVERY iteration and picks 166 vs 167.
    #[test]
    fn w4_g60_the_spore_gate_draws_every_iteration() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(92);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 265,
            pos: Vec2::new(100.0, 100.0),
            w: 22,
            h: 22,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..15 {
            let g = r.next_max(3);
            replay_spawn(&mut r, 22, 22);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, if g != 0 { 166 } else { 167 });
        }
        assert_eq!(rng, r, "15 × (1 gate + 6 internals)");
        assert!(pool.slot(14).active);
        assert!(!pool.slot(15).active);
    }

    /// `NPC.cs:84140-84155` — the Cultist: 30×5 + 10×16 (scale 1.5,
    /// velocity ×2, noGravity), then ONE `Next(1,4)` hood count and
    /// that many `Next(11,14)` type picks.
    #[test]
    fn w4_g60_the_cultist_hood_count_gates_the_type_picks() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 439,
            pos: Vec2::new(100.0, 100.0),
            w: 30,
            h: 50,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..30 {
            replay_spawn(&mut r, 30, 50);
        }
        for k in 30..40 {
            let rolls = replay_spawn(&mut r, 30, 50);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 16);
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.5);
            assert!(d.no_gravity);
        }
        let n = r.next_range(1, 4);
        for _ in 0..n {
            r.next_range(11, 14);
        }
        assert_eq!(rng, r, "40×6 internals + 1 count + n picks");
        assert!(pool.slot(39).active);
        assert!(!pool.slot(40).active);
    }

    /// `NPC.cs:84567-84571` — the Saucer family: 30×228 with the
    /// NewDust 228 tweak (vy re-roll `Next(-10,6)×0.1`, vx ×0.3, scale
    /// ×0.7 — THREE extra internals, `Dust.cs:214-220`) then the
    /// scale = 0.5 OVERWRITE. The witness replays the 9-draw spawn.
    #[test]
    fn w4_g60_the_saucer_228_tweak_rolls_before_the_overwrite() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(111);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 392,
            pos: Vec2::new(100.0, 100.0),
            w: 80,
            h: 40,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..2 {
            let rolls = replay_spawn(&mut r, 80, 40);
            // The 228 tweak: vy = Next(-10,6)*0.1, vx *= 0.3, scale *= 0.7.
            let tweak_vy = r.next_range(-10, 6);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 228);
            assert_eq!(d.scale, 0.5, "the overwrite replaces roll×0.7");
            assert_eq!(d.velocity.x, (rolls[2] as f32 * 0.1 - 1.5) * 0.3);
            assert_eq!(d.velocity.y, tweak_vy as f32 * 0.1, "the re-roll, not roll-1.5");
        }
        for _ in 2..30 {
            replay_spawn(&mut r, 80, 40);
            r.next_range(-10, 6);
        }
        assert_eq!(rng, r, "30 × (6 internals + 1 tweak re-roll)");
        assert!(pool.slot(29).active);
        assert!(!pool.slot(30).active);
    }

    /// `NPC.cs:83105-83118` — Vortex Alien: 10×241 then 20 ×
    /// (SelectRandom `Next(3)` pick BEFORE the spawn, then the 229/240
    /// internals, then scale `1.25 + NextFloat`). The pick precedes
    /// the spawn (argument evaluation order).
    #[test]
    fn w4_g60_the_vortex_alien_pick_precedes_the_spawn() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(121);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 427,
            pos: Vec2::new(100.0, 100.0),
            w: 34,
            h: 34,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..10 {
            replay_spawn(&mut r, 34, 34);
        }
        for k in 10..30 {
            let pick = r.next_max(3);
            let ty = [229, 229, 240][pick as usize];
            replay_spawn(&mut r, 34, 34);
            let extra = r.next_float();
            let d = pool.slot(k);
            assert_eq!(d.dust_type, ty);
            assert_eq!(d.scale, 1.25 + extra);
            assert_eq!(d.fade_in, 0.25);
            assert!(d.no_gravity && d.no_light);
        }
        assert_eq!(rng, r, "10×6 + 20×(1 pick + 6 internals + 1 scale)");
        assert!(pool.slot(29).active);
        assert!(!pool.slot(30).active);
    }

    /// `NPC.cs:82873-82880` — the Crawltipede family: 6×241 then 2×6
    /// with FULL arguments (alpha 100, Transparent, scale 2) — the 6
    /// hits the NewDust tweak (vy re-roll, vx ×0.3, scale ×0.7) but
    /// the C# passes scale 2 as the argument so the rolled×0.7×2 is
    /// the result; NO post writes. The witness pins the 6's tweak.
    #[test]
    fn w4_g60_the_crawltipede_six_dusts_carry_full_args() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(122);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 412,
            pos: Vec2::new(100.0, 100.0),
            w: 30,
            h: 30,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..6 {
            replay_spawn(&mut r, 30, 30);
        }
        for k in 6..8 {
            let rolls = replay_spawn(&mut r, 30, 30);
            let tweak_vy = r.next_range(-10, 6); // the type-6 tweak
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.color, Color::TRANSPARENT);
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 2.0 * 0.7);
            assert_eq!(d.velocity.y, tweak_vy as f32 * 0.1);
        }
        assert_eq!(rng, r, "6×6 + 2×(6+1 tweak)");
        assert!(pool.slot(7).active);
        assert!(!pool.slot(8).active);
    }

    /// `NPC.cs:83601-83631` — the Stardust Worm Head: 5 × (gate + 17?
    /// + 229) then the oldPos loop's `Next(4)` gates until the first
    /// `== 0` breaks — an ALL-ZERO ring (a never-updated corpse; the
    /// live ring rides `Npc::update_trails`, the next witness).
    #[test]
    fn w4_g60_the_worm_head_trail_gates_until_the_zero_break() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 402,
            pos: Vec2::new(100.0, 100.0),
            w: 32,
            h: 32,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let mut spawned = 0usize;
        for _ in 0..5 {
            let g = r.next_max(3);
            if g == 0 {
                replay_spawn(&mut r, 32, 32);
                spawned += 1;
            }
            replay_spawn(&mut r, 32, 32); // the always-229
            spawned += 1;
        }
        // The trail gates: draw until the first == 0 (max 10).
        let mut gates = 0;
        for _ in 0..10 {
            gates += 1;
            if r.next_max(4) == 0 {
                break;
            }
        }
        assert_eq!(rng, r, "5 pair iterations + {gates} trail gates");
        assert!(pool.slot(spawned - 1).active);
        assert!(!pool.slot(spawned).active);
    }

    /// `NPC.cs:83611-83631` with a LIVE ring (the W4-g60 wiring wave):
    /// slots 0-2 carry positions, slot 3 is zero. Every slot draws its
    /// `Next(4)` gate; a `== 0` gate on a filled slot spawns TWO gated
    /// pairs AT THE SLOT (not the head position); the zero slot breaks
    /// AFTER its gate draw — slots 4-9 draw nothing.
    #[test]
    fn w4_g60_the_worm_head_trail_spawns_at_the_live_slots() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(4242);
        let peek = rng.clone();
        let ring = [
            Vec2::new(100.0, 100.0),
            Vec2::new(94.0, 101.0),
            Vec2::new(88.0, 103.0),
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
        ];
        let mut fx = DeathFx {
            ty: 402,
            pos: Vec2::new(106.0, 100.0),
            w: 32,
            h: 32,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: ring,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let mut spawned = 0usize;
        let mut slot_positions: Vec<Vec2> = Vec::new();
        for _ in 0..5 {
            if r.next_max(3) == 0 {
                replay_spawn(&mut r, 32, 32);
                spawned += 1;
            }
            replay_spawn(&mut r, 32, 32);
            spawned += 1;
        }
        for &slot in &ring {
            if r.next_max(4) != 0 {
                continue;
            }
            if slot == Vec2::ZERO {
                break;
            }
            for _ in 0..2 {
                if r.next_max(3) == 0 {
                    replay_spawn(&mut r, 32, 32);
                    slot_positions.push(slot);
                    spawned += 1;
                }
                replay_spawn(&mut r, 32, 32);
                slot_positions.push(slot);
                spawned += 1;
            }
        }
        assert_eq!(rng, r, "the head burst + the gated trail pairs");
        // Every trail dust spawned inside its slot's box — the default
        // block stamps `position + Next(w-4) + 4` (`Dust.cs:170-171`),
        // so the bounds are `[slot+4, slot+w)` — NOT the head position
        // (106,100) when the slot differs.
        for (k, &slot) in slot_positions.iter().enumerate() {
            let d = pool.slot(spawned - slot_positions.len() + k);
            assert!(
                d.position.x >= slot.x + 4.0 && d.position.x < slot.x + 32.0,
                "trail dust {k} rides its slot's X"
            );
            assert!(
                d.position.y >= slot.y + 4.0 && d.position.y < slot.y + 32.0,
                "trail dust {k} rides its slot's Y"
            );
        }
        assert!(pool.slot(spawned - 1).active);
        assert!(!pool.slot(spawned).active);
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
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        assert_eq!(rng, peek, "the default arm draws nothing");
        assert!(!pool.slot(0).active);
    }

    /// The gore-remainder sweep (§11, f5-owned arms): every listed kind
    /// pays EXACTLY its census dust count on the UpdateNPCs stream and
    /// ZERO gore-argument draws — all gores in these arms are
    /// constant-argument (the shimmer/altTexture picks are data). The
    /// per-kind draw counts pin the C# death halves one witness row
    /// each; the shared-arm kinds ride their own row.
    #[test]
    fn w4_g60_gore_tail_f5_arms_pay_dust_only() {
        // (kind, dust count, dust type, w, h, seed) — the §11 census.
        for (ty, n, dty, w, h, seed) in [
            (250u16, 20i32, 54i32, 30i32, 30i32, 201i32), // :88971 3×99
            (315, 20, 54, 30, 40, 202),                  // :89038 3×99+446
            (329, 50, 5, 26, 40, 203),                   // :87745 485-487
            (341, 20, 196, 30, 30, 205),                 // :88755 61/62/63
            (353, 50, 5, 20, 40, 207),                   // :88208 6 cond.
            (354, 50, 5, 20, 40, 208),                   // the shared arm
            (368, 50, 5, 20, 40, 209),                   // :88033 5 cond.
            (441, 50, 5, 20, 40, 210),                   // :87685 768-771
            (471, 50, 5, 30, 46, 211),                   // :89617 675-678
            (550, 50, 5, 20, 40, 213),                   // :87952 967-969
            (588, 50, 5, 20, 40, 214),                   // :87972 5 cond.
            (589, 50, 5, 20, 40, 215),                   // the shared arm
            (633, 50, 5, 20, 40, 216),                   // :87995 6 picks
            (267, 20, 5, 30, 30, 217),                   // :87269 gore 402
            (338, 50, 5, 20, 40, 218),                   // :86878 494+tail
            (339, 50, 5, 20, 40, 219),                   // 495 head
            (340, 50, 5, 20, 40, 220),                   // 496 head
            (342, 50, 0, 20, 40, 221),                   // :86858 500-502
            (343, 50, 5, 26, 46, 222),                   // :86908 503-506
            (348, 10, 5, 24, 40, 223),                   // :86322 528-532
            (349, 10, 5, 24, 40, 224),                   // the shared arm
            (350, 10, 5, 20, 40, 225),                   // :86362 533-536
            (351, 10, 5, 24, 40, 226),                   // :86342 538-542
            (489, 50, 5, 20, 40, 227),                   // :86669 722-724
        ] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w,
                h,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 10],
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..n {
                let rolls = replay_spawn(&mut r, w, h);
                let d = pool.slot(k as usize);
                assert_eq!(d.dust_type, dty, "{ty} dust {k}");
                if dty == 54 {
                    // the smoke54 full-arg shape: alpha 50, scale roll
                    // ×1.5 then velocity ×2 + noGravity (`:89039-89042`).
                    assert_eq!(d.alpha, 50);
                    assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.5);
                    assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 * 2.0);
                    assert!(d.no_gravity);
                }
                if ty == 341 {
                    // the Present Mimic's plain (0,0) burst — no writes.
                    assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1);
                }
            }
            assert_eq!(rng, r, "{ty}: {n}×6 internals, zero gore draws");
            assert!(pool.slot((n - 1) as usize).active);
            assert!(!pool.slot(n as usize).active, "{ty}: exactly {n} dusts");
        }
    }

    /// The gore-remainder sweep (§11, f10-owned arms): the martian
    /// machines + probe + walker + spider-small. The 226 bursts carry
    /// the scale-0.5 OVERWRITE (no extra internals); 227 is a plain
    /// 6-draw spawn; the 410 stardust pair is 5×(17+229) ungated. All
    /// gores constant-argument (the probe's ai[0]==3 velocity is data).
    #[test]
    fn w4_g60_gore_tail_f10_arms_pay_dust_only() {
        for (ty, n, w, h, seed) in [
            (385u16, 30i32, 24i32, 40i32, 302i32), // :84373 597-599
            (386, 30, 24, 40, 303),                // :84393 600-602
            (387, 30, 30, 30, 304),                // :84415 603×2/604
            (389, 30, 30, 40, 305),                // :84459 904-906
            (391, 30, 40, 20, 306),                // :84545 607/608/609
            (399, 10, 16, 16, 307),                // :84515 898/899
            (520, 30, 36, 56, 308),                // :84438 851/852+3
            (393, 30, 40, 40, 310),                // :84567 gore 616
            (394, 30, 40, 40, 311),                // :84567 gore 617
            (395, 30, 60, 60, 312),                // :84567 610-615
            (306, 60, 24, 46, 313),                // :84806 441-445
            (307, 60, 24, 46, 314),                // the shared arm
            (308, 60, 24, 46, 315),                // the shared arm
            (309, 60, 24, 46, 316),                // the shared arm
            (310, 60, 24, 46, 317),                // the shared arm
            (311, 60, 24, 46, 318),                // the shared arm
            (312, 60, 24, 46, 319),                // the shared arm
            (313, 60, 24, 46, 320),                // the shared arm
            (314, 60, 24, 46, 321),                // the shared arm
            (325, 60, 30, 50, 322),                // :84849 469-475
            (327, 60, 40, 40, 323),                // :84828 476-484
            (344, 60, 40, 60, 324),                // :84878 508-512×2
        ] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w,
                h,
                vel: Vec2::ZERO,
                hit_dir: -1,
                old_pos: [Vec2::ZERO; 10],
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..n {
                let rolls = replay_spawn(&mut r, w, h);
                let d = pool.slot(k as usize);
                match ty {
                    385 | 386 | 389 | 391 => assert_eq!(d.dust_type, 227),
                    393..=395 => {
                        // the 228 tweak re-rolls vy (one extra draw),
                        // then the scale-0.5 overwrite (`:84569-84570`).
                        assert_eq!(d.dust_type, 228);
                        r.next_range(-10, 6);
                        assert_eq!(d.scale, 0.5);
                    }
                    306..=314 | 325 | 327 => {
                        let dty = match ty {
                            325 => 191,
                            327 => 189,
                            _ => 194,
                        };
                        assert_eq!(d.dust_type, dty);
                        assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.1);
                    }
                    344 => assert_eq!(d.dust_type, 196),
                    _ => {
                        assert_eq!(d.dust_type, 226);
                        assert_eq!(d.scale, 0.5, "the 226 overwrite");
                        let k_f = if ty == 399 { 2.5 } else { 1.5 };
                        // hit_dir = -1 → sx = -k×hd = +k
                        assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 + k_f);
                    }
                }
            }
            assert_eq!(rng, r, "{ty}: {n}×6 internals, zero gore draws");
            assert!(pool.slot((n - 1) as usize).active);
            assert!(!pool.slot(n as usize).active, "{ty}: exactly {n} dusts");
        }
        // 410 Stardust Spider (small): 5 × (17 + 229 noGravity ×3), no
        // gates on the death half (`NPC.cs:83523-83529`); no gores.
        let wld = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(309);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 410,
            pos: Vec2::new(100.0, 100.0),
            w: 16,
            h: 16,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 10],
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &wld,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..5 {
            replay_spawn(&mut r, 16, 16);
            assert_eq!(pool.slot(k * 2).dust_type, 17);
            replay_spawn(&mut r, 16, 16);
            let d = pool.slot(k * 2 + 1);
            assert_eq!(d.dust_type, 229);
            assert!(d.no_gravity);
        }
        assert_eq!(rng, r, "410: 5×(17+229) pairs, zero gore draws");
        assert!(pool.slot(9).active);
        assert!(!pool.slot(10).active);
    }

    /// `NPC.cs:87186-87218` — the Plantera family, per kind: 150×167
    /// (the 265 short-circuit kills the gate draw for ALL three family
    /// types), then the per-kind gore-position pairs: 262 pays NINE
    /// `Next(w)+Next(h)` pairs (gores 381-387 with 386/387 doubled),
    /// 263/264 pay TWO each (390/391, 388/389). The gores are f5's
    /// (claimed `97c70c97`); g60 consumes exactly 18/4/4 argument draws.
    #[test]
    fn w4_g60_gore_tail_plantera_trio_pays_per_kind_position_pairs() {
        for (ty, n_gores, seed) in [(262u16, 9i32, 501i32), (263, 2, 502), (264, 2, 503)] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w: 48,
                h: 48,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 10],
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..150 {
                replay_spawn(&mut r, 48, 48);
                assert_eq!(pool.slot(k).dust_type, 167, "{ty}: no 166 gate");
            }
            for _ in 0..n_gores {
                r.next_max(48);
                r.next_max(48);
            }
            assert_eq!(rng, r, "{ty}: 150×6 + {} pairs", n_gores * 2);
            assert!(pool.slot(149).active);
            assert!(!pool.slot(150).active, "{ty}: exactly 150 dusts");
        }
    }

    /// `NPC.cs:88048-88073` — the town smoke arm's OTHER two g60 kinds
    /// (369 Angler / 376 Sleeping Angler; 663 rides the dedicated
    /// tombstone witness): each pays 25×31 (alpha 100, scale ×2 roll,
    /// velocity ×1.4, noLight+noGravity) + FOUR `Next(61,64)` tombstone
    /// picks. The gores (61/62/63 with scale-1 + velocity±1 writes) are
    /// f5's — claimed, not yet pushed (`:88046` is past f5's `:86986`).
    #[test]
    fn w4_g60_gore_tail_angler_pair_pays_the_four_tombstone_picks() {
        for (ty, seed) in [(369u16, 601i32), (376, 602)] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w: 18,
                h: 40,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 10],
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..25 {
                let rolls = replay_spawn(&mut r, 18, 40);
                let d = pool.slot(k);
                assert_eq!(d.dust_type, 31);
                assert_eq!(d.alpha, 100);
                assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 2.0);
                assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 * 1.4);
                assert!(d.no_light && d.no_gravity);
            }
            for _ in 0..4 {
                r.next_range(61, 64);
            }
            assert_eq!(rng, r, "{ty}: 25×6 internals + 4 tombstone picks");
            assert!(pool.slot(24).active);
            assert!(!pool.slot(25).active, "{ty}: exactly 25 dusts");
        }
    }

    /// The gore-remainder sweep (§11, the NO-gore arms): 330/352/384/
    /// 533 have zero `NewGore` sites — their whole death half is the
    /// dust burst (288/265/402 already carry dedicated witnesses; 326
    /// shares the Mourning Wood shape, 663 folds onto the town smoke).
    #[test]
    fn w4_g60_gore_tail_no_gore_arms_pay_dust_only() {
        // (kind, n, dust type, w, h, seed)
        for (ty, n, dty, w, h, seed) in [
            (326u16, 60i32, 191i32, 20i32, 20i32, 401i32), // :84903
            (330, 50, 192, 30, 40, 402),                   // :88996
            (352, 60, 67, 20, 20, 403),                    // :84788
            (384, 70, 226, 20, 20, 404),                   // :84355
            (533, 40, 27, 30, 46, 405),                    // :83880
        ] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w,
                h,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 10],
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..n {
                let rolls = replay_spawn(&mut r, w, h);
                let d = pool.slot(k as usize);
                assert_eq!(d.dust_type, dty, "{ty} dust {k}");
                match ty {
                    326 => assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.1),
                    330 => {
                        assert_eq!(d.alpha, 100);
                        assert!(d.no_gravity);
                    }
                    352 => assert!(d.no_gravity),
                    384 => assert_eq!(d.scale, 0.5),
                    533 => {
                        assert_eq!(d.alpha, 50);
                        assert!(d.no_gravity);
                        assert_eq!(d.fade_in, 1.0);
                    }
                    _ => unreachable!(),
                }
            }
            assert_eq!(rng, r, "{ty}: {n}×6 internals, zero gore draws");
            assert!(pool.slot((n - 1) as usize).active);
            assert!(!pool.slot(n as usize).active, "{ty}: exactly {n} dusts");
        }
    }
}
