//! W4-d25 (aiStyle-23) — the possessed-weapon family, `NPC.AI`'s
//! `else if (this.aiStyle == 23)` block (`NPC.cs:25235-25312`), ported
//! 1:1 as a PURE velocity machine (code-only mode: no compiler/tests
//! were run on this branch; the witness tests below are written, not
//! executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 23` rows, whole-file grep)
//!
//! THREE vanilla members, all with the IDENTICAL stat row:
//! * CursedHammer 83 (`NPCID.cs` `CursedHammer = 83`, `SetDefaults`
//!   `NPC.cs:9879-9891`: 40×40, dmg 80, def 18, 200 HP, kb 0.4,
//!   NPCHit4/NPCDeath6, value 1000);
//! * EnchantedSword 84 (`EnchantedSword = 84`, `NPC.cs:9892-9904`:
//!   the same row);
//! * CrimsonAxe 179 (`CrimsonAxe = 179`, `NPC.cs:11266-11278`: the
//!   same row).
//! None writes `noGravity`/`noTileCollide` in `SetDefaults` — the ARM
//! owns both flags (`:25237-25238`, true every tick). The port has no
//! `NpcKind::CursedHammer`/`EnchantedSword`/`CrimsonAxe` yet, so this
//! module lands UNWIRED (the `npc_ai_style17.rs` pre-wiring precedent):
//! a pure fn keyed on the vanilla type, reachable and testable before
//! the kinds land. The wiring step adds the kinds + the style-axis
//! claim.
//!
//! ## The machine
//!
//! * the per-type light (`:25239-25250`): 83 → (0.2, 0.05, 0.3),
//!   179 → (0.3, 0.15, 0.05), else (84) → (0.05, 0.2, 0.3), at the
//!   INTEGER-half centre tile (`(float)(width / 2)`, int division).
//! * the invalid-target retarget (`:25251-25254`): `target < 0 ||
//!   target == 255 || player.dead` → `TargetClosest()`.
//! * `ai[0] == 0` — the launch (`:25255-25272`): the 9f-normalised aim
//!   from the FLOAT-half own centre to the player's INTEGER-half-width
//!   centre (`(float)(width / 2)` on the player, `* 0.5f` on the NPC —
//!   verbatim), `rotation = atan2(vy, vx) + 0.785f`, `ai[0] = 1`,
//!   `ai[1] = 0`, `netUpdate`.
//! * `ai[0] == 1` — the drift (`:25273-25294`): `justHit` arms the spin
//!   (`ai[0] = 2`, `ai[1] = 0`) but the drift body STILL runs this tick
//!   (the C# if/else chain already picked the branch); `velocity *=
//!   0.99`, `ai[1]++`; at 100 → `netUpdate`, spin state, velocity
//!   zeroed; else the rotation re-aims (`atan2 + 0.785f`).
//! * else — the spin (`:25295-25311`): `justHit` re-zeroes the counter;
//!   `velocity *= 0.96`, `ai[1]++`, `rotation += (0.1f + ai[1]/120f *
//!   0.4f) * direction`; at 120 → `netUpdate`, back to the launch
//!   state (`ai[0] = 0`, `ai[1] = 0`).
//!
//! Seam: the module writes only velocity, `ai[0..1]`, `rotation`,
//! `netUpdate` (and the target fields through `target_closest`). It
//! does NOT integrate position: the C# arm ends before `UpdateNPC`'s
//! movement split, and both flags are constant-true, so the wiring
//! caller owns the raw `position += velocity` tail (`NPC.cs:90495-
//! 90498`). The per-tick `Lighting.AddLight` leaves as the return value
//! (the `npc_ai_style22.rs` lights convention — the draw-side feed).
//!
//! ## RNG by construction
//!
//! The arm draws NO `Main.rand` — `NPC.cs:25235-25312` contains no
//! `Next`/`NextFloat`; neither does `TargetClosest` (`NPC.cs:77652-
//! 77781`, the `npc_target.rs` port). So the fn takes no RNG parameter;
//! the witness `the_style23_arm_draws_no_rng` pins a `UnifiedRandom`
//! state across the launch, drift, spin and justHit ticks (written,
//! not run).
#![allow(dead_code)] // UNWIRED (W4-d25): the three kinds + the axis claim are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// One member's `SetDefaults` row (the census above — all three rows
/// are stat-identical; only the light colour differs, keyed on type).
/// `SoundID.NPCHit4`/`NPCDeath6` = `LegacySoundStyle(3, 4)`/`(4, 6)`.
pub struct WeaponDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    /// `(SoundType, Style)` — the HitSound.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — the DeathSound.
    pub death_sound: (i32, i32),
    pub value: f32,
}

/// `NPC.cs:9879-9891` — `else if (this.type == 83)` (CursedHammer).
pub const CURSED_HAMMER: WeaponDefaults = WeaponDefaults {
    ty: 83,
    width: 40,
    height: 40,
    ai_style: 23,
    damage: 80,
    defense: 18,
    life_max: 200,
    knock_back_resist: 0.4,
    hit_sound: (3, 4),
    death_sound: (4, 6),
    value: 1000.0,
};

/// `NPC.cs:9892-9904` — `else if (this.type == 84)` (EnchantedSword).
pub const ENCHANTED_SWORD: WeaponDefaults = WeaponDefaults {
    ty: 84,
    width: 40,
    height: 40,
    ai_style: 23,
    damage: 80,
    defense: 18,
    life_max: 200,
    knock_back_resist: 0.4,
    hit_sound: (3, 4),
    death_sound: (4, 6),
    value: 1000.0,
};

/// `NPC.cs:11266-11278` — `else if (this.type == 179)` (CrimsonAxe).
pub const CRIMSON_AXE: WeaponDefaults = WeaponDefaults {
    ty: 179,
    width: 40,
    height: 40,
    ai_style: 23,
    damage: 80,
    defense: 18,
    life_max: 200,
    knock_back_resist: 0.4,
    hit_sound: (3, 4),
    death_sound: (4, 6),
    value: 1000.0,
};

/// The target slot's candidate — the `npc_ai_style17.rs::target_of`
/// fallback (the C# indexes `Main.player[this.target]`; an unset/stale
/// slot reads slot 0, the documented session-player stand-in).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// The aiStyle-23 arm (`NPC.cs:25235-25312`), one fixed step, for one
/// of the THREE weapon types. Returns the tick's `Lighting.AddLight`
/// payload `(tileX, tileY, [r, g, b])` (`:25239-25250`) for the
/// wiring caller's light feed. `noGravity`/`noTileCollide` are
/// constant-true (`:25237-25238`) — the caller integrates raw.
/// RNG-free by construction.
pub fn ai_style23(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
) -> (i32, i32, [f32; 3]) {
    let (w, h) = npc.size();
    // `:25237-25238` — both flags true every tick (the caller's
    // movement split; no per-tick branch to carry).
    // `:25239-25250` — the per-type light at the INTEGER-half centre.
    let light_tile = (
        ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32,
        ((npc.y + (h as i32 / 2) as f32) / 16.0) as i32,
    );
    let rgb = if ty == 83 {
        [0.2, 0.05, 0.3]
    } else if ty == 179 {
        [0.3, 0.15, 0.05]
    } else {
        [0.05, 0.2, 0.3]
    };
    // `:25251-25254` — the invalid-target retarget.
    if npc.target < 0 || npc.target == 255 || target_of(npc, players).dead {
        target_closest(npc, world, players, true, 23, ticker);
    }
    if npc.ai[0] == 0.0 {
        // `:25255-25272` — the launch: the 9f aim. The NPC centre uses
        // FLOAT halves (`* 0.5f`), the player's centre-X the INTEGER
        // half-width (`(float)(width / 2)`) — verbatim asymmetry.
        let num321 = 9.0f32;
        let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
        let p = target_of(npc, players);
        let mut num322 = p.x + (p.width / 2) as f32 - cx;
        let mut num323 = p.y + (p.height / 2) as f32 - cy;
        let num324 = (num322 * num322 + num323 * num323).sqrt();
        let scale = num321 / num324;
        num322 *= scale;
        num323 *= scale;
        npc.vx = num322;
        npc.vy = num323;
        // `:25268` — the +0.785f sprite offset (NOT π/4 exactly).
        npc.rotation = npc.vy.atan2(npc.vx) + 0.785;
        npc.ai[0] = 1.0;
        npc.ai[1] = 0.0;
        npc.net_update = true;
    } else if npc.ai[0] == 1.0 {
        // `:25273-25294` — the drift. The justHit arm writes the spin
        // state but the drift body still runs THIS tick (the C# chain
        // already picked the branch).
        if npc.just_hit {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
        }
        npc.vx *= 0.99;
        npc.vy *= 0.99;
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 100.0 {
            npc.net_update = true;
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.vx = 0.0;
            npc.vy = 0.0;
        } else {
            npc.rotation = npc.vy.atan2(npc.vx) + 0.785;
        }
    } else {
        // `:25295-25311` — the spin.
        if npc.just_hit {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
        }
        npc.vx *= 0.96;
        npc.vy *= 0.96;
        npc.ai[1] += 1.0;
        // `:25304` — the accelerating spin, signed by direction.
        npc.rotation += (0.1 + npc.ai[1] / 120.0 * 0.4) * npc.facing as f32;
        if npc.ai[1] >= 120.0 {
            npc.net_update = true;
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
        }
    }
    (light_tile.0, light_tile.1, rgb)
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation (`NPC.cs:25235-25312`).
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A stand-in body with the weapons' shared 40×40 box (the kinds
    /// are unported; the machine is stat-identical across the three).
    fn weapon(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((40.0, 40.0));
        n.hp = 200;
        n.max_hp = 200;
        n.target = 0;
        n
    }

    /// The `SetDefaults` rows (`NPC.cs:9879-9891` / `:9892-9904` /
    /// `:11266-11278`) — all three stat-identical.
    #[test]
    fn the_defaults_rows_match_the_census() {
        for (row, ty) in [(CURSED_HAMMER, 83), (ENCHANTED_SWORD, 84), (CRIMSON_AXE, 179)] {
            assert_eq!(row.ty, ty);
            assert_eq!((row.width, row.height), (40, 40));
            assert_eq!(row.ai_style, 23);
            assert_eq!((row.damage, row.defense, row.life_max), (80, 18, 200));
            assert_eq!(row.knock_back_resist, 0.4);
            assert_eq!(row.hit_sound, (3, 4));
            assert_eq!(row.death_sound, (4, 6));
            assert_eq!(row.value, 1000.0);
        }
    }

    /// `:25255-25272` — the launch: the 9f-normalised aim (the NPC's
    /// FLOAT-half centre vs the player's INTEGER-half centre), the
    /// +0.785 rotation, the state latch + netUpdate.
    #[test]
    fn the_launch_normalises_to_nine_and_latches() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        let p = cand(1300.0, 1000.0);
        // Own centre (1020, 1020); player centre (1310, 1021).
        let (mut dx, mut dy) = (1310.0f32 - 1020.0, 1021.0f32 - 1020.0);
        let k = 9.0 / (dx * dx + dy * dy).sqrt();
        dx *= k;
        dy *= k;
        let light = ai_style23(84, &mut n, &w, &[p], 0);
        assert_eq!(n.vx, dx);
        assert_eq!(n.vy, dy);
        assert_eq!(n.rotation, dy.atan2(dx) + 0.785);
        assert_eq!((n.ai[0], n.ai[1]), (1.0, 0.0));
        assert!(n.net_update);
        // `:25249` — the else-branch light at the int-half centre tile.
        assert_eq!(light, (1020 / 16, 1020 / 16, [0.05, 0.2, 0.3]));
    }

    /// `:25239-25246` — the 83/179 light colours.
    #[test]
    fn the_per_type_lights_match_the_census() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        let l83 = ai_style23(83, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!(l83.2, [0.2, 0.05, 0.3]);
        let mut n = weapon(1000.0, 1000.0);
        let l179 = ai_style23(179, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!(l179.2, [0.3, 0.15, 0.05]);
    }

    /// `:25280-25293` — the drift: ×0.99 both axes, the counter climb,
    /// the rotation re-aim each tick below 100.
    #[test]
    fn the_drift_damps_and_reaims() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 40.0;
        n.vx = 9.0;
        n.vy = 0.0;
        ai_style23(84, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!(n.vx, 9.0f32 * 0.99);
        assert_eq!(n.vy, 0.0);
        assert_eq!(n.ai[1], 41.0);
        assert_eq!(n.rotation, n.vy.atan2(n.vx) + 0.785);
        assert!(!n.net_update);
    }

    /// `:25282-25289` — the 100-tick drift expiry: the spin state, the
    /// zeroed velocity, netUpdate — and NO rotation re-aim that tick.
    #[test]
    fn the_drift_expires_at_one_hundred() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 99.0;
        n.vx = 3.0;
        n.vy = -1.0;
        n.rotation = 0.5;
        ai_style23(84, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!((n.ai[0], n.ai[1]), (2.0, 0.0));
        assert_eq!((n.vx, n.vy), (0.0, 0.0));
        assert!(n.net_update);
        assert_eq!(n.rotation, 0.5, "the expiry tick skips the re-aim");
    }

    /// `:25295-25311` — the spin: ×0.96, the accelerating rotation
    /// `(0.1 + ai[1]/120*0.4) * direction`, the 120-tick relaunch.
    #[test]
    fn the_spin_accelerates_and_relaunches_at_one_twenty() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        n.ai[0] = 2.0;
        n.ai[1] = 59.0;
        n.vx = 1.0;
        n.vy = 0.0;
        n.rotation = 2.0;
        n.facing = -1;
        ai_style23(84, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!(n.vx, 1.0f32 * 0.96);
        assert_eq!(n.ai[1], 60.0);
        assert_eq!(n.rotation, 2.0 + (0.1 + 60.0f32 / 120.0 * 0.4) * -1.0);
        // The expiry tick: the rotation bump STILL runs, then relaunch.
        n.ai[1] = 119.0;
        n.net_update = false;
        ai_style23(84, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!((n.ai[0], n.ai[1]), (0.0, 0.0));
        assert!(n.net_update);
    }

    /// `:25275-25279` — justHit in the drift arms the spin state, but
    /// the drift body (×0.99, the counter climb from the RESET 0)
    /// still runs on the same tick.
    #[test]
    fn just_hit_arms_the_spin_inside_the_drift_body() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 50.0;
        n.vx = 4.0;
        n.just_hit = true;
        ai_style23(84, &mut n, &w, &[cand(1300.0, 1000.0)], 0);
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.ai[1], 1.0, "the reset 0 then the drift's ++");
        assert_eq!(n.vx, 4.0f32 * 0.99, "the drift damp still ran");
    }

    /// `:25251-25254` — a dead target re-picks through TargetClosest
    /// (the fresh 255 sentinel picks slot 0).
    #[test]
    fn the_invalid_target_retargets() {
        let w = air();
        let mut n = weapon(1000.0, 1000.0);
        n.target = 255;
        ai_style23(84, &mut n, &w, &[cand(1300.0, 1000.0)], 7);
        assert_eq!(n.target, 0);
        assert_eq!(n.target_set_frame, 7);
    }

    /// The RNG census: the arm takes no stream and the C# block has no
    /// `Next` — a probe stream is untouched across every state.
    #[test]
    fn the_style23_arm_draws_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(23);
        let probe = rng.clone();
        let mut n = weapon(1000.0, 1000.0);
        let p = cand(1300.0, 1000.0);
        ai_style23(83, &mut n, &w, &[p], 0); // launch
        for _ in 0..120 {
            ai_style23(83, &mut n, &w, &[p], 0); // drift → spin
        }
        n.just_hit = true;
        ai_style23(83, &mut n, &w, &[p], 0);
        assert_eq!(rng, probe, "the arm never touches the stream");
    }
}
