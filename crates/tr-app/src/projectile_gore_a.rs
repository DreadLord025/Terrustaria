//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave A: the FIRST 11
//! sites in source order (`:17452`, `:22155`, `:23312`-`:23402`,
//! `:23965`-`:24060`). The `:24037` type-9 trail was missed by the first
//! wave-A pass and filled in the gap commit (the census is authoritative).
//!
//! The pool itself is already ported (`tr_core::gore::GorePool::new_gore`,
//! S1-E-4); this module is the projectile-side spawn table the pool feeds.
//! W4-g16 (code-only mode): the site fns + witnesses land first; the
//! `UpdateProjectiles` slot dispatch (which arm of `AI()`/`Kill()` runs for
//! which slot) is the wiring wave — the `npc_hit_effect_a.rs` (W4-d55)
//! precedent. The NPC-file sites are d55/e5/f10's; the `Main.cs` world and
//! `Player.cs` sites are g17's. Every site here lives in `Projectile.cs`.
//!
//! ## Scope — the first 11 gore sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1 | `:17452` | `EmitEnchantmentVisualsAt`, meleeEnchant 7 |
//! | 2 | `:22155` | `AI_151_SuperStar` trail |
//! | 3 | `:23312` | `AI()` type 162 (Tombstone) landing burst |
//! | 4 | `:23357` | `AI()` type 281 landing burst |
//! | 5 | `:23402` | `AI()` type 240 landing burst |
//! | 6 | `:23965` | `AI()` type 503 trail |
//! | 7 | `:23980` | `AI()` type 12/955 trail (anniversary arm) |
//! | 8 | `:23995` | `AI()` type 725/726/1133 trail |
//! | 9 | `:24018` | `AI()` type 723 trail |
//! | 10 | `:24037` | `AI()` type 9 trail (the gap fill) |
//! | 11 | `:24060` | `AI()` `ai[1] == 1`/type 92 trail |
//!
//! Sites 12+ (`:24267` onward) are wave B (`projectile_gore_b.rs`) and
//! later; the full 165-site census lives in docs/ANALYSIS.md
//! "The Projectile gore sites (W4-g16)".
//!
//! ## RNG contract (the update's stream)
//!
//! Every site runs inside `Main.UpdateProjectiles` (`Main.cs:34865`
//! forward) under that pass's `Main.rand`; the caller hands the stream in
//! and every draw hits it in C# order:
//!
//! * Enchant-7: the `Next(20)` dust gate (`:17441`) ALWAYS rolls (the dust
//!   body is the documented skip); the `Next(40)` gore gate (`:17450`)
//!   then the taken branch's `Next(276, 283)` type, the pool's two kicks
//!   (`Gore.cs:1327-1328`) and the five post-writes' rolls (`:17453-17457`).
//! * The trail gates (`Next(6)`/`Next(24)`/`Next(20)`) roll ONLY when the
//!   hitbox intersects the padded screen rect — the C# `&&` short-circuit;
//!   `on_screen` is the caller-resolved stand-in (the port's screen seam).
//! * The type picks (`SelectRandom` = `Next(len)`, `Utils.cs:2459-2462`;
//!   `Next(16, 18)`; the landing `Next(61, 64)`) evaluate BEFORE the pool
//!   call, then the kicks roll inside `new_gore`.
//! * Post-spawn writes land on the returned slot, the sentinel included —
//!   the C# writes `Main.gore[600]` too (`npc_hit_effect_a.rs` precedent).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * The dust bodies around each site are the dust lanes' (the
//!   `projectile_ai_misc.rs` census precedent); only the GORE draws ride
//!   here. The enchant-7 dust GATE roll is consumed because it precedes
//!   the gore gate on the same stream (`:17441`).
//! * `Main.tenthAnniversaryWorld` (site 7) rides [`TrailCtx`] — the
//!   second `SelectRandom` REPLACES the first's value but BOTH draws stay
//!   on the stream (`:23975-23978`).
//! * `netMode` guards: single-player port (the `!= 1` arms run, `== 2`
//!   arms drop), the pool's own convention.
//! * `ChildSafety` stays at the pool default (disabled) — the App arms it
//!   at wiring; the witnesses read the pool's own gate.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the UpdateProjectiles claim is the follow-up round

use crate::projectile::Projectile;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::World;
use tr_core::loot::DropRng;
#[cfg(test)]
use tr_core::UnifiedRandom;

/// The screen/world flags the trail gates read beyond the port's
/// `Projectile` fields (the stand-in convention, `HitEffectInputs`).
#[derive(Default)]
pub struct TrailCtx {
    /// `base.Hitbox.Intersects(Utils.CenteredRectangle(screenPosition +
    /// screen/2, screen + (400,400)))` (`:23973` etc.) resolved by the
    /// caller — the port's screen seam. The `Next` gate rolls only when
    /// this is true (the C# `&&`).
    pub on_screen: bool,
    /// `Main.tenthAnniversaryWorld` (`:23976`) — the 12/955 type re-pick.
    pub anniversary: bool,
}

/// `Entity.Center` (`Entity.cs:50-60`) — the int width/height widen to
/// float exactly; the port's `w`/`h` are already f32.
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w / 2.0, p.pos.y + p.h / 2.0)
}

/// Site 1 — `EmitEnchantmentVisualsAt`'s meleeEnchant-7 gore
/// (`Projectile.cs:17434-17458`). `box_position`/`box_width`/`box_height`
/// are the caller's arguments (the melee swing's box); only the enchant-7
/// arm's draws ride here — the earlier enchant arms' dust gates are the
/// dust lane's. The velocity clamp (`:17436-17440`) runs BEFORE the gates.
pub fn enchant_seven_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    // `:17436-17440` — clamp the velocity copy to length 4.
    let mut vector = p.vel;
    let len = vector.length();
    if len > 4.0 {
        vector *= 4.0 / len;
    }
    // `:17441` — the dust gate ALWAYS rolls; the dust body is the skip.
    let _dust_gate = rng.next_max(20);
    // `:17450-17457` — the gore gate, then the type + kicks + five rolls.
    if rng.next_max(40) == 0 {
        let ty = rng.next_range(276, 283);
        let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, vector, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
    }
}

/// Site 2 — `AI_151_SuperStar`'s trail gore (`Projectile.cs:22153-22156`):
/// the padded-screen gate, `Next(6)`, then `SelectRandom(16, 17, 17, 17)`
/// at velocity ×0.2.
pub fn super_star_trail(
    p: &Projectile,
    ctx: &TrailCtx,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if ctx.on_screen && rng.next_max(6) == 0 {
        // `Utils.SelectRandom` = `Next(4)` over the list (`Utils.cs:2459`).
        let ty = [16, 17, 17, 17][rng.next_max(4) as usize];
        gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, p.vel * 0.2, ty, 1.0);
    }
}

/// Sites 3-5 — the `AI()` tombstone landing bursts (`Projectile.cs:23310-
/// 23317` type 162, `:23355-23362` type 281, `:23400-23407` type 240).
/// The dust loops before each gore are the dust lane's; the gore row is
/// identical bar the scale (1.5 / 1.5 / 1.25). The caller gates on
/// `ai[0] == 3` (the C# `ai[0] > 2 && ai[0] == 3` pair).
pub fn tombstone_landing_gore(
    p: &Projectile,
    scale: f32,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    // `:23312` / `:23357` / `:23402` — position −10/−10, default velocity,
    // the `Next(61, 64)` type; the kicks roll inside the pool.
    let ty = rng.next_range(61, 64);
    let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world,
        Vec2::new(p.pos.x - 10.0, p.pos.y - 10.0),
        Vec2::ZERO, ty, 1.0,
    );
    // `:23313-23316` — the post-writes, sentinel included.
    let g = gore.slot_mut(idx);
    g.position += p.vel * 1.25;
    g.scale = scale;
    g.velocity += p.vel * 0.5;
    g.velocity *= 0.02;
}

/// Site 6 — the `AI()` type-503 trail gore (`Projectile.cs:23963-23968`):
/// the `Next(48)` gate, type 16 at the centre, velocity ×0.2 then ×0.66
/// plus `velocity * 0.3`. The `Next(16)` dust gate before it is the dust
/// lane's.
pub fn ai503_trail(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if rng.next_max(48) == 0 {
        let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, center(p),
            Vec2::new(p.vel.x * 0.2, p.vel.y * 0.2), 16, 1.0,
        );
        let g = gore.slot_mut(idx);
        g.velocity *= 0.66;
        g.velocity += p.vel * 0.3;
    }
}

/// Site 7 — the `AI()` type-12/955 trail gore (`Projectile.cs:23973-
/// 23981`): the screen gate, `Next(6)`, `SelectRandom(16,17,17,17)`; the
/// anniversary world re-picks via `SelectRandom(16,16,16,17)` — BOTH draws
/// stay on the stream, the second value wins (`:23975-23978`).
pub fn falling_star_trail(
    p: &Projectile,
    ctx: &TrailCtx,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if ctx.on_screen && rng.next_max(6) == 0 {
        let mut ty = [16, 17, 17, 17][rng.next_max(4) as usize];
        if ctx.anniversary {
            ty = [16, 16, 16, 17][rng.next_max(4) as usize];
        }
        gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, p.vel * 0.2, ty, 1.0);
    }
}

/// Site 8 — the `AI()` type-725/726/1133 trail gore (`Projectile.cs:23992-
/// 23996`): the screen gate, `Next(24)`, `SelectRandom(16, 17)`. The C#
/// inner gate (`:23990`) lists 726 twice and omits 724 — the caller's
/// dispatch reproduces that quirk; this fn is the taken branch.
pub fn shadowflare_trail(
    p: &Projectile,
    ctx: &TrailCtx,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if ctx.on_screen && rng.next_max(24) == 0 {
        let ty = [16, 17][rng.next_max(2) as usize];
        gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, p.vel * 0.2, ty, 1.0);
    }
}

/// Site 9 — the `AI()` type-723 trail gore (`Projectile.cs:24016-24019`):
/// the screen gate, `Next(6)`, the literal type 17.
pub fn ai723_trail(
    p: &Projectile,
    ctx: &TrailCtx,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if ctx.on_screen && rng.next_max(6) == 0 {
        gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, p.vel * 0.2, 17, 1.0);
    }
}

/// Site 10 — the `AI()` type-9 trail gore (`Projectile.cs:24034-24038`):
/// the screen gate, `Next(20)`, `Next(16, 18)` at velocity ×0.2. The
/// `Next(4)` dust gate after is the dust lane's. (The wave-A gap fill.)
pub fn ai9_trail(
    p: &Projectile,
    ctx: &TrailCtx,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if ctx.on_screen && rng.next_max(20) == 0 {
        let ty = rng.next_range(16, 18);
        gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, p.vel * 0.2, ty, 1.0);
    }
}

/// Site 11 — the `AI()` `ai[1] == 1 || type == 92` trail gore
/// (`Projectile.cs:24058-24061`): the `Next(20)` gate (NO screen gate on
/// this arm), `Next(16, 18)` at velocity ×0.2. The `Next(10)` dust gate
/// before it is the dust lane's.
pub fn ai_style2_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if rng.next_max(20) == 0 {
        let ty = rng.next_range(16, 18);
        gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos,
            Vec2::new(p.vel.x * 0.2, p.vel.y * 0.2), ty, 1.0,
        );
    }
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// A stand-in projectile with the C# box (the `npc_hit_effect_a`
    /// boss-shape precedent): position (100,100), 20×20, velocity (2,-1).
    fn proj() -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(2.0, -1.0);
        p.w = 20.0;
        p.h = 20.0;
        p
    }

    /// `Projectile.cs:17441-17457` — the enchant-7 stream: the dust gate
    /// ALWAYS rolls, then the gore gate; a taken branch adds the type, the
    /// two kicks and the five post-write rolls. The twin replays the C#
    /// argument draws + the NewGore internals on a second stream.
    #[test]
    fn w4_g16_the_enchant_seven_gore_keeps_both_gates_on_the_stream() {
        let w = air();
        let p = proj();
        // Find a seed whose Next(40) (the second draw) is 0 — the taken
        // branch — without assuming the stream's values.
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_max(20);
            if probe.next_max(40) == 0 {
                break;
            }
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        enchant_seven_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(20); // the dust gate
        let _ = twin.next_max(40); // the gore gate (taken)
        let _ty = twin.next_range(276, 283);
        let _ky = twin.next_range(10, 31); // NewGore Y kick
        let _kx = twin.next_range(-20, 21); // NewGore X kick
        let _sx = twin.next_range(-50, 51);
        let _sy = twin.next_range(-50, 51);
        let _ss = twin.next_range(-20, 21);
        let _ax = twin.next_range(-50, 51);
        let _ay = twin.next_range(-50, 51);
        assert_eq!(rng.peek(), twin.peek(), "the enchant-7 row mirrors");
        // One gore landed with a 276..282 type at the projectile position.
        let g = gore.slot(0);
        assert!(g.active);
        assert!((276..283).contains(&g.gore_type));
        assert_eq!(g.position, p.pos);
    }

    /// `Projectile.cs:23973-23981` — the 12/955 anniversary arm draws TWO
    /// SelectRandom picks (the second wins); the normal arm draws one.
    #[test]
    fn w4_g16_the_falling_star_anniversary_repick_draws_twice() {
        let w = air();
        let p = proj();
        // A seed whose Next(6) is 0 (the gate taken).
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let ctx = TrailCtx { on_screen: true, anniversary: true };
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        falling_star_trail(&p, &ctx, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(6);
        let _first = twin.next_max(4);
        let second = twin.next_max(4);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "both picks stay on the stream");
        assert_eq!(gore.slot(0).gore_type, [16, 16, 16, 17][second as usize]);
        // Off-screen: NO draws at all (the && short-circuit).
        let off = TrailCtx { on_screen: false, anniversary: true };
        let mut rng2 = UnifiedRandom::new(seed);
        let mut gore2 = GorePool::new();
        falling_star_trail(&p, &off, &mut gore2, &mut rng2, &w);
        assert_eq!(rng2.peek(), UnifiedRandom::new(seed).peek());
    }

    /// `Projectile.cs:23312-23316` — the tombstone row: the Next(61,64)
    /// type + the two kicks, then the four post-writes (no draws). The
    /// 162/281 scale is 1.5, the 240 scale 1.25.
    #[test]
    fn w4_g16_the_tombstone_landing_gore_writes_the_csharp_postlude() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(9);
        tombstone_landing_gore(&p, 1.5, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(9);
        let ty = twin.next_range(61, 64);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "three draws, no post-write rolls");
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.scale, 1.5);
        // position = (pos − 10) + vel × 1.25; velocity = (kicks + vel×0.5) × 0.02.
        assert_eq!(g.position, Vec2::new(90.0, 90.0) + p.vel * 1.25);
        let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
        assert_eq!(g.velocity, (kicked + p.vel * 0.5) * 0.02);
    }

    /// `Projectile.cs:23963-23968` — the 503 trail: the Next(48) gate, the
    /// fixed type 16, the kicks, then ×0.66 + vel×0.3 (no extra draws).
    #[test]
    fn w4_g16_the_ai503_trail_gates_on_forty_eight() {
        let w = air();
        let p = proj();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(48) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        ai503_trail(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(48);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, 16);
        assert_eq!(g.position, Vec2::new(110.0, 110.0)); // the centre
        let kicked = Vec2::new(p.vel.x * 0.2 + kx as f32 * 0.1, p.vel.y * 0.2 - ky as f32 * 0.1);
        assert_eq!(g.velocity, kicked * 0.66 + p.vel * 0.3);
    }

    /// `Projectile.cs:22155` / `:23995` / `:24018` / `:24060` — the four
    /// remaining trail shapes: their gates (6/24/6/20) and type draws
    /// (SelectRandom(4)/SelectRandom(2)/none/Next(16,18)) in order.
    #[test]
    fn w4_g16_the_remaining_trails_match_their_gate_and_type_draws() {
        let w = air();
        let p = proj();
        let on = TrailCtx { on_screen: true, anniversary: false };
        // SuperStar: Next(6) + Next(4) + kicks.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        super_star_trail(&p, &on, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(6);
        let pick = twin.next_max(4);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, [16, 17, 17, 17][pick as usize]);
        // 725/726/1133: Next(24) + Next(2) + kicks.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(24) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        shadowflare_trail(&p, &on, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(24);
        let pick = twin.next_max(2);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, [16, 17][pick as usize]);
        // 723: Next(6) + kicks only (the type is the literal 17).
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        ai723_trail(&p, &on, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(6);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, 17);
        // ai[1]==1/92: Next(20) + Next(16,18) + kicks (no screen gate).
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(20) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        ai_style2_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(20);
        let ty = twin.next_range(16, 18);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, ty);
        // Type 9 (the gap fill): Next(20) + Next(16,18) + kicks, screen-gated.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(20) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        ai9_trail(&p, &on, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(20);
        let ty = twin.next_range(16, 18);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, ty);
    }
}
