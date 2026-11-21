//! The Prismatic Lacewing's machine — the type-661 branch of
//! `NPC.AI_065_Butterflies` (`NPC.cs:44896-45110`), aiStyle 65,
//! `NPCID.EmpressButterfly` (661, `ID/NPCID.cs:11999`). W4-d125 empress
//! wiring: the census found the critter entirely missing (the release
//! whitelist `bug_net::npc_catchable` carried 661 and the item rows
//! existed, but no `NpcKind`, no AI arm — a released Lacewing spawned
//! nothing). The other butterfly kinds (356/444/653 + the gem/gold
//! families) stay unclaimed; this module ports ONLY the 661 path plus
//! the shared tail it rides.
//!
//! The machine:
//! * **the release/vanish gate** (`:44900-44926`): `num3 = 60`,
//!   `num4 = 50`; an invalid target or one ≥ 300 px away raises the
//!   counter, and a non-remix player target OUTSIDE the Hallow flips
//!   `num4` to 60 and raises it too (`:44912-44916`);
//!   `ai[2] = Clamp(ai[2] + ToDirectionInt(flag), 0, num4)` — the ±1
//!   walk (`Utils.cs:2010-2017`). At `ai[2] >= 60` the critter vanishes:
//!   `active = false` + return (`:44919-44926`), NO death events.
//!   `Opacity = GetLerpValue(60, num4/2, ai[2], clamped)` — the INVERTED
//!   range (`:44927`); `dontTakeDamage = ai[2] >= num4/2` (`:44954`,
//!   INTEGER division).
//! * **the shared butterfly tail** (`:44956-45110`): the `ai[3]` scale
//!   latch (`Next(75,111) * 0.01` once), the `localAI[0]` re-aim clock
//!   (`Next(90,240)`, `TargetClosest`, the >700-px long-range aim vs the
//!   one-shot `localAI[3]` close-range scatter), the 60:1 velocity
//!   blend, the 3-tile-down / 30-tile-up avoidance probes, the 10-tick
//!   enemy-separation push (100-px `Hitbox.Distance`, cap 16), the
//!   `collideX` bounce (×−0.2) and the direction latch.
//! * The type-356 catchItem re-derive (`:45106`) and the 653 lava-fly
//!   arm (`:45109-45117`) are NOT 661 — unclaimed.
//!
//! **RNG by construction** (every `Main.rand` draw in C# order):
//! * the vanish-trail gate (`:44931`): `Next(5)` ALWAYS drawn while
//!   `ai[2] > 0` (1); on a 0 the dust fires — `NewDust`'s own 6
//!   (`Dust.cs:170-175`, `:190`, w/h = 10; the `hslToRgb` colour argument
//!   draws NOTHING), `NextVector2Circular` (2, `Utils.cs:1266-1269`),
//!   the velocity `NextFloat` (1), the fadeIn `NextFloat` (1),
//!   `CloneDust` → `NewDust` w/h 0 → 6 more (`Dust.cs:263-269`) = 16 on
//!   a dust tick, 1 otherwise;
//! * the scale latch (`:44971-44974`): `Next(75,111)` once;
//! * the re-aim (`:44976-45012`): `Next(90,240)` (1), then the
//!   long-range ladder `Next(50,151)` / `Next(100,151)` / `Next(150,201)`
//!   (1) + `Next(100,251)` (1) + `Next(-50,51)` (1) + the conditional
//!   `Next(100,251)` (1, only when `position.Y > player.Y - 100`), OR the
//!   close-range scatter `Next(26,301)` (1) + `Next(-100,101)` ×2.
//!   `TargetClosest` itself is RNG-free (`npc_target`).
//! Dust-pool assumption (the R7 documented deviation): the pool is
//! below the 50 % skip tier and on-screen, `ChildSafety.Disabled` true;
//! dust type 267 is in NONE of the extra-draw lists — exactly 6 draws
//! per `NewDust`. The visuals are skipped; the draws are CONSUMED.
//!
//! Documented deviations:
//! * the `Lighting.AddLight` head (`:44902-44904`) is the draw-side
//!   feed's (skipped);
//! * `GetTargetData` maps the session player (the S1-D-3p convention);
//!   an invalid target reads the (0,0) default (`NPCAimedTarget.cs:21`)
//!   and `Main.player[255]`'s zero position for the re-aim deltas;
//! * the enemy-separation scan reads the caller's threat slice (the
//!   sim stamps it; a solo caller passes `&[]` — no push);
//! * the movement integration is the CALLER's (`UpdateCollision`,
//!   `NPC.cs:90489`: 661 is noGravity but NOT noTileCollide — the
//!   port's `integrate_flyer_collision`).

#![allow(dead_code)] // the sim wiring lands with W4-d125's phase 3

use tr_core::{UnifiedRandom, World};

use crate::npc::Npc;
use crate::player::Player;

/// One sibling NPC as the separation scan reads it (`NPC.cs:45073`):
/// `active && damage > 0 && !friendly` filtered by the caller, the
/// hitbox rect + centre for `Hitbox.Distance(Center) <= 100`.
#[derive(Clone, Copy, Debug)]
pub struct LacewingThreat {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub cx: f32,
    pub cy: f32,
}

/// The per-tick outcome beside the NPC mutations.
#[derive(Clone, Copy, Debug, Default)]
pub struct LacewingOut {
    /// `this.active = false` (`:44921`) — the vanish, NOT a death: no
    /// loot, no death events (the port's `encourage_despawn(0)` shape).
    pub deactivate: bool,
}

/// `Utils.GetLerpValue(..., clamped)` (`Utils.cs:315-343`) — the
/// inverted-range branch included (the Opacity ramp reads 60 → num4/2).
fn get_lerp_value(from: f32, to: f32, t: f32, clamped: bool) -> f32 {
    if clamped {
        if from < to {
            if t < from {
                return 0.0;
            }
            if t > to {
                return 1.0;
            }
        } else {
            if t < to {
                return 1.0;
            }
            if t > from {
                return 0.0;
            }
        }
    }
    (t - from) / (to - from)
}

/// `Utils.ToDirectionInt` (`Utils.cs:2010-2017`) — bool → ±1 (NOT 0/1).
fn to_direction_int(b: bool) -> f32 {
    if b { 1.0 } else { -1.0 }
}

/// `Dust.NewDust`'s 6 draws (`Dust.cs:170-175`, `:190`) under the
/// documented pool assumption; w/h clamp ≥ 5 (`:155-162`).
fn new_dust_draws(rng: &mut UnifiedRandom, width: i32, height: i32) {
    let w = width.max(5);
    let h = height.max(5);
    rng.next_max(w - 4);
    rng.next_max(h - 4);
    rng.next_range(-20, 21);
    rng.next_range(-20, 21);
    rng.next_max(3);
    rng.next_range(-20, 21);
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit angle
/// draw THEN the radius `NextFloat` (2 draws).
fn next_vector2_circular_draws(rng: &mut UnifiedRandom) {
    rng.next_float(); // NextVector2Unit's angle
    rng.next_float(); // the radius scale
}

/// `Utils.Distance(this Rectangle, Vector2)` (`Utils.cs:1507-1540`) —
/// 0 inside, the edge gap on the axis bands, the corner distance
/// outside both.
fn rect_distance(x: f32, y: f32, w: f32, h: f32, px: f32, py: f32) -> f32 {
    let (l, t, r, b) = (x, y, x + w, y + h);
    if px >= l && px <= r && py >= t && py <= b {
        return 0.0;
    }
    if px >= l && px <= r {
        return if py < t { t - py } else { py - b };
    }
    if py >= t && py <= b {
        return if px < l { l - px } else { px - r };
    }
    let cx = if px < l { l } else { r };
    let cy = if py < t { t } else { b };
    ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt()
}

/// One fixed step of the 661 branch + the shared tail
/// (`NPC.cs:44896-45110`). `threats` is the pre-filtered sibling slice
/// (`active && damage > 0 && !friendly`, `:45073`); the caller owns the
/// movement integration (`UpdateCollision`).
pub fn lacewing_step(
    n: &mut Npc,
    world: &World,
    player: &Player,
    threats: &[LacewingThreat],
    rng: &mut UnifiedRandom,
) -> LacewingOut {
    let mut out = LacewingOut::default();
    let mut num = n.ai[0]; // the desired velocity X
    let mut num2 = n.ai[1]; // the desired velocity Y
    let (w, h) = n.size();
    let (cx, cy) = (n.x + w * 0.5, n.y + h * 0.5);
    // `:44900-44926` — the release/vanish gate. `GetTargetData`
    // (`NPC.cs:6901-6912`): the session player when the target is
    // valid, the (0,0) default otherwise.
    let target_valid = n.target >= 0 && n.target < 255 && !player.dead;
    let (pcx, pcy) = if target_valid { player.center() } else { (0.0, 0.0) };
    let num3 = 60.0f32;
    let mut flag = false;
    let mut num4 = 50.0f32;
    if !target_valid || ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt() >= 300.0 {
        flag = true; // :44908-44911
    }
    let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    if !remix && target_valid && !player.zones.hallow {
        // `:44912-44916` — the target is the session PLAYER by the
        // single-player fold (`NPCTargetType.Player`).
        num4 = num3;
        flag = true;
    }
    n.ai[2] = (n.ai[2] + to_direction_int(flag)).clamp(0.0, num4); // :44917
    if n.ai[2] >= num3 {
        // `:44919-44926` — active = false + return: the tail is SKIPPED.
        out.deactivate = true;
        return out;
    }
    // `:44927` — the INVERTED-range Opacity ramp; the port's alpha is
    // the truncating `(1 - v) * 255` cast (`NPC.cs:6826-6829`).
    let opacity = get_lerp_value(num3, num4 / 2.0, n.ai[2], true);
    n.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as u8;
    if n.ai[2] > 0.0 {
        // `:44928-44953` — the 1-iteration dust loop: Next(5) ALWAYS,
        // 16 draws on a hit (the module doc's census).
        if rng.next_max(5) == 0 {
            new_dust_draws(rng, w as i32, h as i32); // :44935
            next_vector2_circular_draws(rng); // :44936
            rng.next_float(); // :44937 velocity *= NextFloat() * 0.8f
            rng.next_float(); // :44939 fadeIn
            new_dust_draws(rng, 0, 0); // :44943 CloneDust
        }
    }
    // `:44954` — INTEGER num4/2 (25 or 30).
    n.dont_take_damage = n.ai[2] >= (num4 / 2.0).trunc();
    // `:44956-45013` — the shared netMode != 1 block. The `ai[2] == 0`
    // colour init EXCLUDES 661 (`:44958`); the scale latch does not.
    if n.ai[3] == 0.0 {
        n.ai[3] = rng.next_range(75, 111) as f32 * 0.01; // :44971-44974
    }
    n.local_ai[0] -= 1.0; // :44975
    if n.local_ai[0] <= 0.0 {
        n.local_ai[0] = rng.next_range(90, 240) as f32; // :44977
        let cand = crate::npc_target::session_candidate(player);
        crate::npc_target::target_closest(n, world, &[cand], true, 65, 0); // :44978
        // `:44979-45011` — the re-aim reads Main.player[target]; an
        // invalid target reads slot 255's zero position (documented).
        let (tpcx, tpcy) = if n.target >= 0 && n.target < 255 {
            player.center()
        } else {
            (0.0, 0.0)
        };
        let tpy = if n.target >= 0 && n.target < 255 { player.y } else { 0.0 };
        let num17 = (cx - tpcx).abs();
        if num17 > 700.0 && n.local_ai[3] == 0.0 {
            // `:44981-44999` — the long-range ladder.
            let mut num18 = rng.next_range(50, 151) as f32 * 0.01;
            if num17 > 1000.0 {
                num18 = rng.next_range(150, 201) as f32 * 0.01;
            } else if num17 > 850.0 {
                num18 = rng.next_range(100, 151) as f32 * 0.01;
            }
            let num19 = n.facing * rng.next_range(100, 251);
            let mut num20 = rng.next_range(-50, 51);
            if n.y > tpy - 100.0 {
                num20 -= rng.next_range(100, 251);
            }
            let num21 = num18 / ((num19 * num19 + num20 * num20) as f32).sqrt();
            num = num19 as f32 * num21;
            num2 = num20 as f32 * num21;
        } else {
            // `:45001-45009` — the one-shot close-range scatter.
            n.local_ai[3] = 1.0;
            let num22 = rng.next_range(26, 301) as f32 * 0.01;
            let num23 = rng.next_range(-100, 101);
            let num24 = rng.next_range(-100, 101);
            let num25 = num22 / ((num23 * num23 + num24 * num24) as f32).sqrt();
            num = num23 as f32 * num25;
            num2 = num24 as f32 * num25;
        }
        n.net_update = true; // :45011
    }
    n.scale = n.ai[3]; // :45014
    // `:45015-45017` — the 60:1 blend.
    let num26 = 60.0f32;
    n.vx = (n.vx * (num26 - 1.0) + num) / num26;
    n.vy = (n.vy * (num26 - 1.0) + num2) / num26;
    // `:45018-45032` — the 3-tile-down probe (solid OR liquid).
    if n.vy > 0.0 {
        let tx = (cx / 16.0) as i32;
        let ty = (cy / 16.0) as i32;
        for j in ty..ty + 3 {
            let blocked = world.tile(tx, j).is_some_and(|t| {
                (t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)) || t.liquid > 0
            });
            if blocked {
                num2 *= -1.0;
                if n.vy > 0.0 {
                    n.vy *= 0.9;
                }
            }
        }
    }
    // `:45033-45056` — the 30-tile-up probe (solid only, ANY hit blocks).
    if n.vy < 0.0 {
        let tx = (cx / 16.0) as i32;
        let ty = (cy / 16.0) as i32;
        let mut flag2 = false;
        for k in ty..ty + 30 {
            if world
                .tile(tx, k)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                flag2 = true;
            }
        }
        if !flag2 {
            num2 *= -1.0;
            if n.vy < 0.0 {
                n.vy *= 0.9;
            }
        }
    }
    // `:45057-45086` — the enemy-separation push (10-tick for 661).
    if n.local_ai[1] > 0.0 {
        n.local_ai[1] -= 1.0;
    } else {
        n.local_ai[1] = 10.0; // :45065-45068
        let mut num33 = 0.0f32;
        let (mut zx, mut zy) = (0.0f32, 0.0f32);
        for t in threats {
            if rect_distance(t.x, t.y, t.w, t.h, cx, cy) <= 100.0 {
                num33 += 1.0;
                // `base.DirectionFrom(nPC.Center)` = Normalize(Center -
                // source) (`Entity.cs:210-213`); the port's zero guard
                // (the Fishron precedent — XNA yields NaN on overlap).
                let (dx, dy) = (cx - t.cx, cy - t.cy);
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.0 {
                    zx += dx / len;
                    zy += dy / len;
                }
            }
        }
        if num33 > 0.0 {
            n.vx += zx / num33 * 2.0;
            n.vy += zy / num33 * 2.0;
            let len = (n.vx * n.vx + n.vy * n.vy).sqrt();
            if len > 16.0 {
                // `SafeNormalize(Vector2.Zero) * 16f` (`:45081-45084`).
                if len > 0.0 {
                    n.vx = n.vx / len * 16.0;
                    n.vy = n.vy / len * 16.0;
                } else {
                    n.vx = 0.0;
                    n.vy = 0.0;
                }
            }
        }
    }
    // `:45087-45091` — the collideX bounce.
    if n.collide_x {
        num = if !(n.vx < 0.0) { 0.0 - num.abs() } else { num.abs() };
        n.vx *= -0.2;
    }
    // `:45092-45099` — the direction latch.
    if n.vx < 0.0 {
        n.facing = -1;
    }
    if n.vx > 0.0 {
        n.facing = 1;
    }
    n.ai[0] = num; // :45100
    n.ai[1] = num2; // :45101
    out
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN NOT RUN (code-only mode, no compiler
    //! invoked). Each pins one cited block of the 661 branch.
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 400, 240)
    }

    fn lacewing(cx: f32, cy: f32) -> Npc {
        let mut n = Npc::new(NpcKind::PrismaticLacewing, cx - 5.0, cy - 5.0, 1, 0.0);
        n.hp = n.max_hp;
        n.target = 0;
        n
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = air();
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p.zones.hallow = true;
        p
    }

    /// The roster row (`NPC.cs:17351-17365`).
    #[test]
    fn the_lacewing_roster_row() {
        assert_eq!(NpcKind::PrismaticLacewing.size(), (10.0, 10.0));
        assert_eq!(NpcKind::PrismaticLacewing.ai_style(), 65);
        assert_eq!(NpcKind::PrismaticLacewing.max_hp(), 5);
        assert_eq!(NpcKind::PrismaticLacewing.contact_damage(), 0);
        assert_eq!(NpcKind::PrismaticLacewing.cs_id(), 661);
        assert!(NpcKind::PrismaticLacewing.no_gravity());
        // The release whitelist + the catch item (`Main.cs:9411`,
        // `NPC.cs:17364`).
        assert!(crate::bug_net::npc_catchable(661));
        assert_eq!(crate::bug_net::catch_item(661, 0.0), 4961);
        // The ctor's friendly + 90-tick window (the catchable arm,
        // `NPC.cs:17851-17855`).
        let n = Npc::new(NpcKind::PrismaticLacewing, 0.0, 0.0, 1, 0.0);
        assert!(n.friendly);
        assert_eq!(n.catchable_immunity, 90);
        assert_eq!(n.knock_back_resist(), 1.0, "no kb row — the class default");
    }

    /// The vanish gate: a Hallow player far away raises ai[2] by +1 per
    /// tick; at 60 the critter deactivates WITHOUT the tail (`:44919`).
    #[test]
    fn the_vanish_gate_counts_up_and_deactivates_at_60() {
        let w = air();
        let p = player_at(4000.0, 1000.0); // > 300 px away
        let mut n = lacewing(100.0, 100.0);
        let mut rng = UnifiedRandom::new(42);
        n.ai[3] = 1.0; // skip the scale latch draw for the count pin
        n.local_ai[0] = 100.0; // skip the re-aim
        for tick in 1..60 {
            let out = lacewing_step(&mut n, &w, &p, &[], &mut rng);
            assert!(!out.deactivate);
            assert_eq!(n.ai[2], tick as f32);
        }
        let out = lacewing_step(&mut n, &w, &p, &[], &mut rng);
        assert!(out.deactivate, "ai[2] 60: active = false + return");
    }

    /// A non-remix player OUTSIDE the Hallow flips num4 to 60 and the
    /// dontTakeDamage threshold to 30 (`:44912-44916`, `:44954`).
    #[test]
    fn the_outside_hallow_gate_extends_the_window() {
        let w = air();
        let mut p = player_at(100.0, 100.0); // close, valid
        p.zones.hallow = false;
        let mut n = lacewing(100.0, 100.0);
        let mut rng = UnifiedRandom::new(42);
        n.ai[3] = 1.0;
        n.local_ai[0] = 100.0;
        lacewing_step(&mut n, &w, &p, &[], &mut rng);
        assert_eq!(n.ai[2], 1.0, "the outside-Hallow flag raises the counter");
        // The half-window invulnerability: ai[2] 30 → dontTakeDamage.
        n.ai[2] = 29.0;
        lacewing_step(&mut n, &w, &p, &[], &mut rng);
        assert!(n.dont_take_damage, "ai[2] 30 >= num4/2 = 30");
    }

    /// In the Hallow with a close valid target the counter DECAYS (−1)
    /// and clamps at 0 (`:44917` — ToDirectionInt(false) = −1).
    #[test]
    fn the_hallow_calm_decays_the_counter() {
        let w = air();
        let p = player_at(100.0, 100.0);
        let mut n = lacewing(100.0, 100.0);
        let mut rng = UnifiedRandom::new(42);
        n.ai[2] = 5.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 100.0;
        lacewing_step(&mut n, &w, &p, &[], &mut rng);
        assert_eq!(n.ai[2], 4.0);
        n.ai[2] = 0.0;
        lacewing_step(&mut n, &w, &p, &[], &mut rng);
        assert_eq!(n.ai[2], 0.0, "the Clamp floor");
    }

    /// RNG census: the scale latch draws ONE Next(75,111); ai[2] == 0
    /// draws NO dust gate; the re-aim clock draws Next(90,240) + the
    /// close-range scatter triple (`:44971`/`:44977`/`:45003-45005`).
    #[test]
    fn the_first_tick_rng_order() {
        let w = air();
        let p = player_at(100.0, 100.0);
        let mut n = lacewing(100.0, 100.0);
        let mut rng = UnifiedRandom::new(42);
        let mut probe = UnifiedRandom::new(42);
        lacewing_step(&mut n, &w, &p, &[], &mut rng);
        // The expected stream: scale (75..111), re-aim (90..240),
        // scatter (26..301), (-100..101) ×2. NO Next(5) — ai[2] was 0
        // at the dust gate (the gate reads the PRE-increment value? No:
        // ai[2] became 1 BEFORE the dust block — but the calm target
        // DECAYS it to 0 first: ToDirectionInt(false) = −1 clamped at 0).
        let scale = probe.next_range(75, 111) as f32 * 0.01;
        let clock = probe.next_range(90, 240) as f32;
        let speed = probe.next_range(26, 301) as f32 * 0.01;
        let dx = probe.next_range(-100, 101);
        let dy = probe.next_range(-100, 101);
        assert_eq!(n.ai[3], scale);
        assert_eq!(n.local_ai[0], clock);
        assert_eq!(n.local_ai[3], 1.0, "the scatter latches localAI[3]");
        let len = ((dx * dx + dy * dy) as f32).sqrt();
        assert_eq!(n.ai[0], dx as f32 * (speed / len));
        assert_eq!(n.ai[1], dy as f32 * (speed / len));
        assert_eq!(rng.next(), probe.next(), "the streams stayed in lockstep");
    }

    /// The dust-tick census: ai[2] > 0 draws Next(5) every tick, 16 on a
    /// hit (`:44931-44943`).
    #[test]
    fn the_dust_gate_draws_one_or_seventeen() {
        let w = air();
        let p = player_at(4000.0, 1000.0); // far — the counter climbs
        let mut n = lacewing(100.0, 100.0);
        n.ai[2] = 5.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 100.0;
        // Find a seed whose first Next(5) is 0 (a dust tick).
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(5) != 0 {
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut probe = UnifiedRandom::new(seed);
        lacewing_step(&mut n, &w, &p, &[], &mut rng);
        assert_eq!(probe.next_max(5), 0, "the gate draw");
        // 6 NewDust draws (w/h = 10 → Next(6) ×2, Next(-20,21) ×2,
        // Next(3), Next(-20,21)), 2 circular, 1 velocity, 1 fadeIn,
        // 6 CloneDust (w/h 0 → clamped 5 → Next(1) ×2 + the same tail).
        probe.next_max(6);
        probe.next_max(6);
        probe.next_range(-20, 21);
        probe.next_range(-20, 21);
        probe.next_max(3);
        probe.next_range(-20, 21);
        probe.next_float();
        probe.next_float();
        probe.next_float();
        probe.next_float();
        probe.next_max(1);
        probe.next_max(1);
        probe.next_range(-20, 21);
        probe.next_range(-20, 21);
        probe.next_max(3);
        probe.next_range(-20, 21);
        assert_eq!(rng.next(), probe.next(), "17 draws on the dust tick");
    }

    /// The separation push: one threat at the centre pushes the desired
    /// velocity away, capped at 16 (`:45069-45085`).
    #[test]
    fn the_separation_push_caps_at_16() {
        let w = air();
        let p = player_at(100.0, 100.0);
        let mut n = lacewing(100.0, 100.0);
        let mut rng = UnifiedRandom::new(42);
        n.ai[3] = 1.0;
        n.local_ai[0] = 100.0;
        n.local_ai[1] = 0.0; // the scan fires this tick
        n.vx = 10.0;
        let threat = LacewingThreat {
            x: 60.0, y: 90.0, w: 20.0, h: 20.0, cx: 70.0, cy: 100.0,
        };
        lacewing_step(&mut n, &w, &p, &[threat], &mut rng);
        assert_eq!(n.local_ai[1], 10.0, "the 661 re-arm is 10, not 15");
        // DirectionFrom(threat) = +X; the blend kept vx > 0, +2 push.
        assert!(n.vx > 0.0, "pushed away from the left-side threat");
    }

    /// FindFrame case 661 (`NPC.cs:72742-72771`): the vx*0.3 tilt, the
    /// accumulator bands 0/1/2/1 over 7-tick strides, the wrap at 27.
    #[test]
    fn the_find_frame_bands() {
        let mut n = lacewing(100.0, 100.0);
        n.vx = 2.0;
        n.frame_counter = 0.0;
        n.find_frame();
        assert_eq!(n.rotation, 0.6, "vx * 0.3");
        assert_eq!(n.frame, 0, "counter 2 < 7");
        n.frame_counter = 13.0;
        n.find_frame();
        assert_eq!(n.frame, 2, "counter 15 in [14, 21)");
        n.frame_counter = 25.5;
        n.find_frame();
        assert_eq!(n.frame, 1, "the fourth band");
        assert_eq!(n.frame_counter, 0.0, "the >= 27 wrap");
    }
}
