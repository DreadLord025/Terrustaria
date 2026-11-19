//! S1-D-4 — the Frost Moon projectile family (`ProjectileID.cs:1244-1258`).
//! R25's style-61 census documented the SantaNK1 rows 350/351/352 unported
//! (`Projectile.cs:4400-4429`); this round's census extended to the whole
//! family and found **all eight rows missing** on main: Everscream's
//! 345/346/347 and IceQueen's 348/349 too. The rows land in
//! `projectile/data.rs` (`proj_data`, cited per field); this module owns the
//! dispatch arms:
//!
//! * **345 PineNeedleHostile / 348 FrostWave / 349 FrostShard / 350 Missile**
//!   — aiStyle-1 per-type hooks inside `AI_001` (`Projectile.cs:54002-
//!   57794`), ported in the slice-A/B shape (`projectile_ai001_a/b`): the
//!   hook group above the counter, the `flag3` counter switch, the gravity
//!   else-if chain, the shared flight tail, the default rotation. The gate
//!   [`covers`] is kind-disjoint from A's `handles` (14/81/83/98/100/180/
//!   184/248/259/270/325/389/436/438), B's `covers` (449/462/581/601/622/
//!   680/686/962/980), the M4-PROJ-6 riders (20/88/355/660), the Book of
//!   Skulls (837) and the parallel slice C's set (110/275/276/634/635/645/
//!   662/675/685, `task/S1-D-5-aistyle1-types-c` — merge-time the two gates
//!   chain, both kind-disjoint). R19b owns the ai001-C kinds; none is here.
//! * **346 OrnamentHostile / 352 Spike** — aiStyle-14 arms. The generic
//!   `ai_spear` (projectile.rs) ports the branch's default tail
//!   (`:25143-25159`); its inset kill scan cites `:25726-25737` (the port's
//!   pre-existing `HandleMovement` stand-in — kept untouched). The dispatch
//!   gates these two kinds HERE ahead of `ai_spear`: [`ornament`] ports the
//!   346 block (`:24867-24897`) including the last-tick shrapnel burst (the
//!   one behaviour-feeding RNG site in the family); [`spike_pulse`] ports
//!   the 352 block (`:24848-24866`) — both then ride the SAME generic tail
//!   via `ai_spear`, no parallel copy of the flight.
//! * **351 Present** — aiStyle 58 (`AI()`'s `else if (this.aiStyle == 58)`,
//!   `Projectile.cs:28906-28937`): the alpha fade, the two-state drop
//!   machine, the velocity aim. Small arm, inline here; the dispatch adds
//!   the style-58 arm gated to 351 (the only rowed kind of the style).
//!
//! Movement is NOT in `AI_001`/the style-14 branch — the aiStyle-1 kinds
//! ride the shared `finish_flight` tail (the slice-B replication of
//! `ai_style1_flight`, projectile.rs:5243-5267); the style-14/58 kinds ride
//! `ai_spear`'s integration / [`present`]'s own `kill_step`-shaped tail.
//!
//! RNG by construction (ANALYSIS "The Frost Moon projectile family"):
//! - 345/348/349/352/351 arms draw **zero** `Main.rand` (their hooks'
//!   sound/light/dust riders are skipped, and none of those blocks rolls:
//!   `:57118-57122` sound-only latch, `:56248-56252` sound-only latch,
//!   `:24869-24882` sound + light).
//! - 350's `:55138-55157` dust window rolls `Next(3)` (+ inner `Next(10)`)
//!   and `Next(4)` (+ inner `Next(5)`) — dust-only draws. The module keeps
//!   the aiStyle-1 pool's zero-RNG convention (the slice-A/B precedent):
//!   the window is SKIPPED whole, draws included. No observable state
//!   depends on them (`Main.dust` has no port layer), so the skip cannot
//!   change a trajectory; the stream-position deviation is the documented
//!   slice-B convention, recorded in ANALYSIS.
//! - 346's last-tick burst (`:24883-24896`) DOES feed observable state
//!   (five spawned projectiles' velocities + frames) — those draws are
//!   KEPT, in the C#'s exact order, on the pass's threaded stream: per
//!   shrapnel `Next(-20,21)` → `Next(-20,0)` → `Next(-30,31)` (X jitter) →
//!   `Next(-30,31)` (Y jitter). The witness test pins the order on a
//!   recording stream.
//!
//! CODE-ONLY MODE (S1-D-4): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); verified by re-reading the
//! cited C# blocks and hand arithmetic.
//!
//! Deliberate skips (ANALYSIS): dust, `Lighting.AddLight` (346's per-frame
//! light `:24877`/`:24881`, 350's `:55137`), sound (345 Item1 `:57121`, 346
//! Item1 `:24872`, 348 Item8 `:56251`, 350 Item42 `:55161`), `netUpdate`.
//! The 346 burst's `owner == Main.myPlayer` gate folds to `owner ==
//! local_owner` (the port's single-player stand-in, the saucer precedent).

use tr_core::World;

use crate::projectile::{
    FROST_SHARD, FROST_WAVE, PINE_NEEDLE_HOSTILE, SANTA_MISSILE, Projectile,
};

/// The aiStyle-1 kinds this module owns (the census's four missing rows).
pub(crate) const FROSTMOON_AI001_KINDS: [u16; 4] =
    [PINE_NEEDLE_HOSTILE, FROST_WAVE, FROST_SHARD, SANTA_MISSILE];

/// Whether this module owns the kind's aiStyle-1 arm. Kind-disjoint from
/// slices A/B/C, the riders and the Book of Skulls by construction (the
/// sets share no id — see the module doc).
pub(crate) fn covers(kind: u16) -> bool {
    FROSTMOON_AI001_KINDS.contains(&kind)
}

/// The `flag3` skip switch (`Projectile.cs:55947-56032`): 348 (`:55985`),
/// 349 (`:55986`) and 350 (`:55987`) rest (no `ai[0]++`); 345 is NOT on the
/// list — it counts, and its own arm pins the counter at 50 (`:57123-
/// 57127`). `flag4` (inverted gravity) is 1006-only (`:56034-56036`).
fn counts_flight(kind: u16) -> bool {
    kind == PINE_NEEDLE_HOSTILE
}

/// The shared flight tail — a verbatim replication of
/// `Projectile::ai_style1_flight`'s body below the gravity
/// (projectile.rs:5250-5266): the ±16 Y clamp, the default rotation
/// (`AI_001`'s `:57750` `atan2(vy, vx) + 1.57f` — none of the four kinds
/// has a per-kind rotation tail: 345/348/349/350 appear in no arm of the
/// `:57582-57751` chain) and the kill-on-tile integration (the port's
/// `HandleMovement` stand-in, the slice-B `finish_flight` precedent).
fn finish_flight(p: &mut Projectile, world: &World) {
    p.vel.y = p.vel.y.clamp(-16.0, 16.0);
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.active = false;
            p.time_left = 0;
        } else {
            p.pos.x = nx;
            p.pos.y = ny;
        }
    } else {
        p.pos += p.vel;
    }
}

/// `AI_001`'s Frost Moon body for one slot, in the C#'s own block order.
/// Zero RNG (the module doc's argument): the only rolling block in the
/// four arms is 350's dust window, skipped whole with its draws.
pub(crate) fn step(p: &mut Projectile, world: &World) {
    // ---- 1. The hook group above the counter ------------------------------
    // `:55130-55163` — SantaNK1's missile (350): the alpha fade 100, the
    // light (`:55137`, skipped), the alpha == 0 dust window (`:55138-55157`
    // — the `Next(3)`/`Next(10)`/`Next(4)`/`Next(5)` draws ride the skipped
    // dust, the aiStyle-1 zero-RNG convention), the ai[1] sound latch
    // (Item42 `:55161` skipped — the latch state itself rides).
    if p.kind == SANTA_MISSILE {
        p.alpha -= 100;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        if p.ai[1] == 0.0 {
            p.ai[1] = 1.0;
        }
    }

    // ---- 2. The flight counter (`:55945-56041`) ---------------------------
    if counts_flight(p.kind) {
        p.ai[0] += 1.0;
    }

    // ---- 3. The hook group below the counter ------------------------------
    // `:56221-56245` — IceQueen's frost shard (349): `frame = (int)ai[0]`
    // (the `NPC.cs:33607` `Next(5)` spawn payload picks the row), the
    // +0.2 gravity EVERY tick, the localAI[0] three-state pulse — states
    // 0/2 grow the scale +0.01 and fade the alpha 50 (at ≤ 0 → state 1,
    // alpha 0), state 1 shrinks −0.01 and ramps +50 (at ≥ 255 → state 2,
    // alpha 255). The pulse cycles forever: visible → invisible → visible.
    if p.kind == FROST_SHARD {
        p.frame = p.ai[0] as i32;
        p.vel.y += 0.2;
        if p.local_ai[0] == 0.0 || p.local_ai[0] == 2.0 {
            p.scale += 0.01;
            p.alpha -= 50;
            if p.alpha <= 0 {
                p.local_ai[0] = 1.0;
                p.alpha = 0;
            }
        } else if p.local_ai[0] == 1.0 {
            p.scale -= 0.01;
            p.alpha += 50;
            if p.alpha >= 255 {
                p.local_ai[0] = 2.0;
                p.alpha = 255;
            }
        }
    }
    // `:56246-56273` — IceQueen's frost wave (348): the localAI[1] sound
    // latch (Item8 `:56251` skipped), then the SAME three-state pulse on
    // ai[0] instead of localAI[0] (states 0/2 fade in, state 1 fades out).
    if p.kind == FROST_WAVE {
        if p.local_ai[1] == 0.0 {
            p.local_ai[1] = 1.0;
        }
        if p.ai[0] == 0.0 || p.ai[0] == 2.0 {
            p.scale += 0.01;
            p.alpha -= 50;
            if p.alpha <= 0 {
                p.ai[0] = 1.0;
                p.alpha = 0;
            }
        } else if p.ai[0] == 1.0 {
            p.scale -= 0.01;
            p.alpha += 50;
            if p.alpha >= 255 {
                p.ai[0] = 2.0;
                p.alpha = 255;
            }
        }
    }

    // ---- 4. The pre-rotation/gravity else-if chain (`:56753-57580`) -------
    // The chain is mutually exclusive. 348/349/350 match no arm before the
    // tail's inner chain and none inside it, so they reach the generic
    // `ai[0] >= 15` arm (`:57568-57579`) — but their counters REST at 0
    // (flag3), so the gate never opens: they fly exactly straight (the
    // 349's +0.2 above is its own, already applied). 345 has its own arm.
    if p.kind == PINE_NEEDLE_HOSTILE {
        // `:57116-57128` (the 336/345 arm): the 345-only localAI[0] sound
        // latch (Item1 `:57121` skipped), then from ai[0] >= 50 the counter
        // pins at 50 and the +0.5 gravity opens. The counter counted above
        // (`:56038-56041` — 345 is not on the flag3 list), so the drop
        // starts on tick 50.
        if p.local_ai[0] == 0.0 {
            p.local_ai[0] = 1.0;
        }
        if p.ai[0] >= 50.0 {
            p.ai[0] = 50.0;
            p.vel.y += 0.5;
        }
    } else if p.ai[0] >= 15.0 {
        // The generic arm (`:57568-57579`) — unreachable for the three
        // resting kinds (ai[0] stays 0), kept for shape with the slice-B
        // precedent's C# pin (the port's generic `ai_style1_flight` omits
        // the pin; a claimed kind gets the C#'s).
        p.ai[0] = 15.0;
        p.vel.y += 0.1;
    }

    // ---- 5/6. The shared tail ---------------------------------------------
    // `:57748-57751` — none of the four kinds is 344/498, so the default
    // rotation applies; `:57753-57768` — the ±16 clamp (flag8 is true for
    // all four; flag4 false). Both ride `finish_flight`.
    finish_flight(p, world);
}

/// One shrapnel spawn of the ornament's last-tick burst
/// (`Projectile.cs:24885-24895`): position = the ornament's centre, the
/// velocity's raw components, the 40 damage / 0 knockback constants, the
/// `ai1 = ornament.ai[1]` frame payload. `NewProjectile` centres the box
/// (`:10876-10877`) — the pool's spawn site subtracts w/2, h/2 (the
/// sentry-blast precedent, projectile.rs:14846-14855).
pub(crate) struct ShrapnelSpawn {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ai1: f32,
}

/// `AI()`'s aiStyle-14 type-346 block (`Projectile.cs:24867-24897`) —
/// Everscream's ornament. The `localAI[0]` one-shot latch (the Item1 cue
/// `:24872` skipped), `frame = (int)ai[1]` (the `NPC.cs:32885` `Next(2)`
/// payload), the per-frame light (`:24877`/`:24881`, skipped — no pool
/// layer), and the last-tick burst: when `owner == Main.myPlayer &&
/// timeLeft == 1`, five 347 shrapnel fly out at speed 10 with the C#'s
/// four draws per fragment IN ORDER on the pass's stream:
/// `Next(-20,21)` (X) → `Next(-20,0)` (Y, always ≤ −1 so the sqrt never
/// divides by zero) → `Next(-30,31)` (X jitter ×0.01) → `Next(-30,31)`
/// (Y jitter ×0.01). The spawns ride back to the pool (the slot arm cannot
/// see the slot array — the nebula precedent). The generic aiStyle-14
/// tail (`:25143-25159`) still runs afterwards (`ai_spear`, the dispatch).
///
/// The C#'s `owner == Main.myPlayer` gate (`:24883`) folds to TRUE here:
/// the pool only runs the arm on locally-simulated slots (the `simulated`
/// gate, projectile.rs:12961-12965 — the port's single-player stand-in for
/// the owner test; the C# spawns the ornament with `Owner: Main.myPlayer`,
/// `NPC.cs:32885`). The port models hostile shots as owner 255
/// (`spawn_hostile`), so a literal `owner == local_owner` test would never
/// fire — the simulation gate IS the owner test in this port.
pub(crate) fn ornament(
    p: &mut Projectile,
    rng: &mut dyn tr_core::loot::DropRng,
    out: &mut Vec<ShrapnelSpawn>,
) {
    // `:24869-24873` — the one-shot latch.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
    }
    // `:24874` — the frame pin.
    p.frame = p.ai[1] as i32;
    // `:24883-24896` — the burst window (the owner gate folded per above).
    if p.time_left == 1 {
        let (cx, cy) = (p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        for _ in 0..5 {
            // `:24887-24893` — num119 = 10f; the direction draws, the
            // normalise, the two jitters. Draw order is the C#'s named-
            // argument evaluation order: SpeedX's `Next(-30,31)` evaluates
            // before SpeedY's (`:24894`).
            let mut dx = rng.next_range(-20, 21) as f32;
            let mut dy = rng.next_range(-20, 0) as f32;
            let inv = 10.0 / (dx * dx + dy * dy).sqrt();
            dx *= inv;
            dy *= inv;
            let jx = 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
            let jy = 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
            out.push(ShrapnelSpawn {
                x: cx,
                y: cy,
                vx: dx * jx,
                vy: dy * jy,
                ai1: p.ai[1],
            });
        }
    }
}

/// `AI()`'s aiStyle-14 type-352 block (`Projectile.cs:24848-24866`) —
/// SantaNK1's spike: the `localAI[1]` direction latch (0 → +1), the alpha
/// pulse ±25/tick bouncing between 0 and 255 (at ≤ 0 clamp 0 and reverse
/// to +1; at ≥ 255 clamp 255 and reverse to −1), the scale drift
/// `localAI[1] * 0.01` per tick. Zero RNG. The generic aiStyle-14 tail
/// (`:25143-25159`) runs afterwards via `ai_spear` at the dispatch — this
/// is the head only, no parallel flight.
pub(crate) fn spike_pulse(p: &mut Projectile) {
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = 1.0;
    }
    p.alpha += (25.0 * p.local_ai[1]) as i32;
    if p.alpha <= 0 {
        p.alpha = 0;
        p.local_ai[1] = 1.0;
    } else if p.alpha >= 255 {
        p.alpha = 255;
        p.local_ai[1] = -1.0;
    }
    p.scale += p.local_ai[1] * 0.01;
}

/// `AI()`'s aiStyle-58 arm (`Projectile.cs:28906-28937`) — SantaNK1's
/// present drop (351 is the style's only rowed kind). The alpha fade 50
/// (clamp 0), the two-state machine: state 0 pins frame 0, counts ai[1],
/// from ai[1] > 30 adds +0.1 fall, and at velocity.Y >= 0 flips to state 1;
/// state 1 pins frame 1, +0.1 fall capped at 3, X drag 0.99. The rotation
/// aims along the velocity (+1.57). Zero RNG. Movement: the row's
/// `tileCollide = false` (`:4419`) — the plain integration, the port's
/// `kill_step` non-colliding arm (projectile.rs:7811-7813).
pub(crate) fn present(p: &mut Projectile) {
    // `:28908-28912`.
    p.alpha -= 50;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:28913-28925` — state 0. NOTE: the C# uses two consecutive `if`s,
    // not else-if — the tick that flips ai[0] to 1 ALSO runs the state-1
    // block below (frame 1, the second +0.1, the drag) in the same tick.
    if p.ai[0] == 0.0 {
        p.frame = 0;
        p.ai[1] += 1.0;
        if p.ai[1] > 30.0 {
            p.vel.y += 0.1;
        }
        if p.vel.y >= 0.0 {
            p.ai[0] = 1.0;
        }
    }
    // `:28926-28935` — state 1 (runs on the flip tick too, per above).
    if p.ai[0] == 1.0 {
        p.frame = 1;
        p.vel.y += 0.1;
        if p.vel.y > 3.0 {
            p.vel.y = 3.0;
        }
        p.vel.x *= 0.99;
    }
    // `:28936`.
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    // HandleMovement's non-colliding integration (the row's
    // `tileCollide = false`, `:4419`).
    p.pos += p.vel;
}

// ---------------------------------------------------------------------------
// Witness tests-as-code — WRITTEN NOT RUN (code-only mode, S1-D-4). Verified
// by re-reading the cited C# and hand arithmetic; run them with
// `cargo test -p tr-app projectile_ai_frostmoon` once a compiler is allowed.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{
        ORNAMENT_HOSTILE, ORNAMENT_SHRAPNEL, SANTA_PRESENT, SANTA_SPIKE, proj_data, proj_frames,
    };
    use glam::Vec2;
    use tr_core::loot::DropRng;

    fn air() -> World {
        tr_core::World::new("t".into(), 200, 200)
    }

    /// A rowed family shot at (1600, 800) — open air in the 200×200 test
    /// world — flying right at 10 px/tick.
    fn shot(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("rowed kind");
        // Field-by-field on `Default` — the struct has private bookkeeping
        // fields (E0451 outside projectile.rs).
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 255; // the hostile-shot owner (spawn_hostile)
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(10.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.extra_updates = d.extra_updates;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p
    }

    /// A recording `DropRng` — the draw log pins the C#'s `Main.rand` order
    /// (the ai026_b precedent).
    #[derive(Default)]
    struct RecordRng {
        draws: Vec<String>,
    }
    impl DropRng for RecordRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push(format!("range({min},{max})"));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float".into());
            0.5
        }
    }

    /// The census rows (`Projectile.cs:4353-4429`) — every field the row
    /// carries, cited per row in data.rs.
    #[test]
    fn the_eight_rows_carry_the_csharp_set_defaults() {
        let n = proj_data(PINE_NEEDLE_HOSTILE).unwrap();
        assert_eq!((n.w, n.h, n.ai_style, n.penetrate), (4.0, 4.0, 1, 1));
        assert_eq!((n.time_left, n.extra_updates, n.tile_collide), (3600, 0, true));
        assert_eq!(n.scale, 0.8); // `:4359`
        let o = proj_data(ORNAMENT_HOSTILE).unwrap();
        assert_eq!((o.w, o.h, o.ai_style, o.penetrate, o.time_left), (18.0, 18.0, 14, -1, 300));
        let s = proj_data(ORNAMENT_SHRAPNEL).unwrap();
        assert_eq!((s.w, s.h, s.ai_style, s.penetrate), (6.0, 6.0, 2, -1));
        let w = proj_data(FROST_WAVE).unwrap();
        assert_eq!((w.w, w.h, w.ai_style, w.penetrate, w.time_left), (48.0, 48.0, 1, -1, 900));
        assert_eq!((w.extra_updates, w.tile_collide), (1, false)); // `:4385`/`:4387`
        let f = proj_data(FROST_SHARD).unwrap();
        assert_eq!((f.w, f.h, f.ai_style, f.penetrate, f.time_left), (12.0, 12.0, 1, -1, 900));
        assert!(f.tile_collide); // the 349 row does NOT clear it (`:4390-4399`)
        let m = proj_data(SANTA_MISSILE).unwrap();
        assert_eq!((m.w, m.h, m.ai_style, m.penetrate, m.time_left), (14.0, 14.0, 1, -1, 300));
        assert_eq!((m.alpha, m.tile_collide), (255, false)); // `:4402`/`:4408`
        let pr = proj_data(SANTA_PRESENT).unwrap();
        assert_eq!((pr.w, pr.h, pr.ai_style, pr.penetrate), (24.0, 24.0, 58, -1));
        assert_eq!((pr.alpha, pr.tile_collide, pr.time_left), (255, false, 3600));
        let sp = proj_data(SANTA_SPIKE).unwrap();
        assert_eq!((sp.w, sp.h, sp.ai_style, sp.penetrate, sp.time_left), (30.0, 30.0, 14, -1, 1200));
        // The frame sheets (`Main.cs:8529-8530`/`:8565-8566`).
        assert_eq!(proj_frames(SANTA_PRESENT), 2);
        assert_eq!(proj_frames(FROST_SHARD), 5);
        assert_eq!(proj_frames(ORNAMENT_HOSTILE), 2);
        assert_eq!(proj_frames(ORNAMENT_SHRAPNEL), 2);
    }

    /// The gate is kind-disjoint from every other aiStyle-1 owner.
    #[test]
    fn the_gate_claims_only_the_four_frostmoon_kinds() {
        for k in FROSTMOON_AI001_KINDS {
            assert!(covers(k));
        }
        // Slices A/B, the riders, the Book of Skulls, slice C's set, and
        // the family's non-aiStyle-1 members.
        for k in [
            1u16, 14, 81, 83, 98, 100, 180, 184, 248, 259, 270, 325, 389, 436, 438, // A + generic
            449, 462, 581, 601, 622, 680, 686, 962, 980, // B
            20, 88, 355, 660, 837, // riders + book
            110, 275, 276, 634, 635, 645, 662, 675, 685, // C (R19b)
            346, 347, 351, 352, // the family's other styles
        ] {
            assert!(!covers(k), "kind {k} must not be claimed");
        }
    }

    /// 345 (`:57116-57128` + `:56038-56041`): the counter counts (not on
    /// the flag3 list), the sound latch rides, the +0.5 gravity opens on
    /// tick 50 and the counter pins there.
    #[test]
    fn the_pine_needle_counts_to_fifty_then_drops() {
        let w = air();
        let mut p = shot(PINE_NEEDLE_HOSTILE);
        p.vel = Vec2::ZERO;
        step(&mut p, &w);
        assert_eq!(p.local_ai[0], 1.0); // the `:57118` latch
        assert_eq!(p.ai[0], 1.0);
        assert_eq!(p.vel.y, 0.0);
        for _ in 0..48 {
            step(&mut p, &w);
        }
        assert_eq!(p.ai[0], 49.0);
        assert_eq!(p.vel.y, 0.0); // 49 < 50
        step(&mut p, &w); // tick 50
        assert_eq!(p.ai[0], 50.0);
        assert!((p.vel.y - 0.5).abs() < 1e-6);
        step(&mut p, &w); // pinned at 50, another +0.5
        assert_eq!(p.ai[0], 50.0);
        assert!((p.vel.y - 1.0).abs() < 1e-6);
    }

    /// 348 (`:56246-56273`): the ai[0] pulse from a 0-alpha spawn — state 0
    /// grows/fades (alpha already 0 → flips to state 1 on tick 1), state 1
    /// shrinks/ramps +50 until 255 (6 ticks: 50·6 = 300 ≥ 255 clamps), then
    /// state 2 fades again. The counter rests (flag3 `:55985`) — the pulse
    /// writes are the ONLY ai[0] motion. No gravity: straight flight.
    #[test]
    fn the_frost_wave_pulses_and_flies_straight() {
        let w = air();
        let mut p = shot(FROST_WAVE);
        p.vel = Vec2::new(6.0, 0.0);
        step(&mut p, &w);
        assert_eq!(p.local_ai[1], 1.0); // the `:56248` latch
        assert_eq!(p.ai[0], 1.0); // alpha 0 − 50 ≤ 0 → state 1 (`:56257-56261`)
        assert_eq!(p.alpha, 0);
        assert!((p.scale - 1.01).abs() < 1e-6);
        for _ in 0..4 {
            step(&mut p, &w);
        }
        assert_eq!(p.alpha, 200); // 4 × 50
        step(&mut p, &w); // 250 — still under 255, state 1 holds
        assert_eq!(p.alpha, 250);
        assert_eq!(p.ai[0], 1.0);
        step(&mut p, &w); // 300 → clamp 255, state 2 (`:56267-56271`)
        assert_eq!(p.alpha, 255);
        assert_eq!(p.ai[0], 2.0);
        assert!((p.scale - (1.01 - 0.06)).abs() < 1e-5); // 6 shrink ticks
        step(&mut p, &w); // state 2: +0.01 scale, alpha 205
        assert_eq!(p.alpha, 205);
        assert_eq!(p.vel.y, 0.0); // no gravity — flag3 rest
        assert_eq!(p.vel.x, 6.0);
    }

    /// 349 (`:56221-56245`): the frame pin reads ai[0] (the spawn payload),
    /// the +0.2 gravity rides EVERY tick, the localAI[0] pulse mirrors the
    /// wave's on localAI[0].
    #[test]
    fn the_frost_shard_pins_its_frame_and_falls_at_02() {
        let w = air();
        let mut p = shot(FROST_SHARD);
        p.ai[0] = 3.0; // the NPC.cs:33607 Next(5) payload
        p.vel = Vec2::new(0.0, 3.0);
        step(&mut p, &w);
        assert_eq!(p.frame, 3);
        assert_eq!(p.ai[0], 3.0); // flag3 rest (`:55986`) — no count
        assert!((p.vel.y - 3.2).abs() < 1e-6);
        assert_eq!(p.local_ai[0], 1.0); // alpha 0 → state 1
        for _ in 0..6 {
            step(&mut p, &w);
        }
        assert_eq!(p.alpha, 255); // 6 × 50 = 300 clamps at 255, state 2
        assert_eq!(p.local_ai[0], 2.0);
        assert_eq!(p.frame, 3); // the pin holds
    }

    /// 350 (`:55130-55163`): the alpha fade 100 (255 → 155 → 55 → 0), the
    /// ai[1] latch, the resting counter, the straight tile-less flight.
    #[test]
    fn the_missile_fades_hundred_and_flies_straight() {
        let w = air();
        let mut p = shot(SANTA_MISSILE);
        step(&mut p, &w);
        assert_eq!(p.alpha, 155);
        assert_eq!(p.ai[1], 1.0);
        step(&mut p, &w);
        assert_eq!(p.alpha, 55);
        step(&mut p, &w);
        assert_eq!(p.alpha, 0);
        step(&mut p, &w);
        assert_eq!(p.alpha, 0); // clamped
        assert_eq!(p.ai[0], 0.0); // flag3 (`:55987`)
        assert_eq!(p.vel.y, 0.0);
        assert_eq!(p.pos.x, 1640.0); // 4 × 10, no tile collide
    }

    /// 346's burst (`:24883-24896`): only at timeLeft == 1 (the owner gate
    /// folds to the pool's simulation gate — the module doc); five spawns;
    /// the four draws per fragment IN ORDER on the stream (the RNG
    /// witness); the frame payload rides ai1.
    #[test]
    fn the_ornament_bursts_five_shrapnel_in_draw_order() {
        let mut p = shot(ORNAMENT_HOSTILE);
        p.ai[1] = 1.0;
        p.time_left = 1;
        let mut rng = RecordRng::default();
        let mut out = Vec::new();
        ornament(&mut p, &mut rng, &mut out);
        assert_eq!(p.local_ai[0], 1.0); // the `:24869` latch
        assert_eq!(p.frame, 1); // the `:24874` pin
        assert_eq!(out.len(), 5);
        // RecordRng returns min: dx = −20, dy = −20, jitters ×0.70.
        let inv = 10.0f32 / (800.0f32).sqrt();
        for s in &out {
            assert_eq!((s.x, s.y), (1609.0, 809.0)); // centre of the 18×18 box
            assert!((s.vx - (-20.0 * inv * 0.70)).abs() < 1e-4);
            assert!((s.vy - (-20.0 * inv * 0.70)).abs() < 1e-4);
            assert_eq!(s.ai1, 1.0);
        }
        // The draw order: per fragment X, Y, X-jitter, Y-jitter — 20 draws.
        assert_eq!(rng.draws.len(), 20);
        for f in 0..5 {
            assert_eq!(rng.draws[f * 4], "range(-20,21)");
            assert_eq!(rng.draws[f * 4 + 1], "range(-20,0)");
            assert_eq!(rng.draws[f * 4 + 2], "range(-30,31)");
            assert_eq!(rng.draws[f * 4 + 3], "range(-30,31)");
        }
        // No burst outside the window.
        let mut q = shot(ORNAMENT_HOSTILE);
        q.time_left = 2;
        let mut out2 = Vec::new();
        ornament(&mut q, &mut rng, &mut out2);
        assert!(out2.is_empty());
    }

    /// 352 (`:24848-24866`): the alpha pulse ±25 bouncing 0..255 with the
    /// scale drift ±0.01 — from a 0-alpha spawn the first tick latches +1
    /// and climbs; 255 is reached on tick 11 (25 × 11 = 275 ≥ 255), the
    /// direction reverses, the scale drift follows.
    #[test]
    fn the_spike_pulses_alpha_and_scale() {
        let mut p = shot(SANTA_SPIKE);
        spike_pulse(&mut p);
        assert_eq!(p.local_ai[1], 1.0);
        assert_eq!(p.alpha, 25);
        assert!((p.scale - 1.01).abs() < 1e-6);
        for _ in 0..9 {
            spike_pulse(&mut p);
        }
        assert_eq!(p.alpha, 250); // 10 × 25
        spike_pulse(&mut p); // 275 → clamp 255, reverse BEFORE the scale drift
        assert_eq!(p.alpha, 255);
        assert_eq!(p.local_ai[1], -1.0);
        spike_pulse(&mut p);
        assert_eq!(p.alpha, 230);
        // 10 grow ticks (+0.10), then ticks 11/12 drift −0.01 each.
        assert!((p.scale - (1.0 + 10.0 * 0.01 - 2.0 * 0.01)).abs() < 1e-5);
    }

    /// 351 (`:28906-28937`): the alpha fade 50, the state-0 hold with the
    /// 30-tick grace, the flip tick runs BOTH blocks (the consecutive-if
    /// shape), state 1's +0.1 capped at 3 and the 0.99 drag.
    #[test]
    fn the_present_holds_thirty_ticks_then_drops() {
        // The grace: an upward lob holds still for 30 ticks, then falls.
        let mut p = shot(SANTA_PRESENT);
        p.vel = Vec2::new(2.0, -1.0);
        present(&mut p);
        assert_eq!(p.alpha, 205); // 255 − 50
        assert_eq!(p.frame, 0);
        assert_eq!(p.ai[0], 0.0);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.vel.y, -1.0); // ai[1] = 1 ≤ 30: no fall yet
        for _ in 0..29 {
            present(&mut p);
        }
        assert_eq!(p.ai[1], 30.0);
        assert_eq!(p.vel.y, -1.0); // the grace holds through tick 30
        present(&mut p); // ai[1] = 31 > 30: the first +0.1
        assert!((p.vel.y - (-0.9)).abs() < 1e-6);
        assert_eq!(p.ai[0], 0.0); // still rising
        // The flip tick (vy = 0 at entry): state 0 flips ai[0] to 1, then
        // the SAME tick's state-1 block runs — frame 1, the second +0.1,
        // the drag (the consecutive-if shape, `:28913`/`:28926`).
        let mut q = shot(SANTA_PRESENT);
        q.vel = Vec2::new(2.0, 0.0);
        present(&mut q);
        assert_eq!(q.ai[0], 1.0);
        assert_eq!(q.frame, 1);
        assert!((q.vel.y - 0.1).abs() < 1e-6);
        assert!((q.vel.x - 2.0 * 0.99).abs() < 1e-6);
        // The cap at 3 (the float drift may overshoot a hair — the C#'s
        // `> 3f` clamp catches it the same way).
        for _ in 0..40 {
            present(&mut q);
        }
        assert!((q.vel.y - 3.0).abs() < 1e-5);
        assert_eq!(q.alpha, 0); // long faded
    }
}
