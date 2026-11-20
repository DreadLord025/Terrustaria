//! The aiStyle-92 machine — the Target Dummy (`NPC.AI()`'s
//! `else if (this.aiStyle == 92)` arm, `NPC.cs:38810-38857`). W4-d32,
//! research/25 row 92 (MISSING → ported), CODE-ONLY MODE (no compiler
//! invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 92;` across the decompiled tree
//! returns exactly ONE assignment: `NPC.cs:14594`, the
//! `else if (this.type == 488)` row (`:14590-14605`) — the
//! TargetDummy (`ID/NPCID.cs:11649`). No dynamic `aiStyle`
//! reassignment to 92 exists. The kind has no `NpcKind` yet — the
//! module lands UNWIRED (the style73/style86 precedent).
//!
//! ## The machine
//!
//! The hit-sound shuffle (`:38812-38826`): `Next(20) == 0` → a
//! SECOND draw `Next(15,18)` picks `SoundID.NPCHit15/16/17`
//! ((3,15)/(3,16)/(3,17), `ID/SoundID.cs:1271-1273`) — the dummy's
//! three wooden thunk variants.
//!
//! The defense zero (`:38827`): `defDefense = defense = 0` (the
//! chained write, verbatim).
//!
//! The client bail (`:38828-38831`): netMode == 1 returns — the
//! solo session folds past it.
//!
//! The tile gate (`:38832-38838`): `flag98` starts false (the
//! decompiled `bool flag98 = false` — the placeholder for the
//! stripped net branch); the tile at `(ai[0], ai[1])` must be
//! active AND type 378 (`TileID.TargetDummy`, `ID/TileID.cs:1385`),
//! else the dummy dies.
//!
//! The target gate (`:38839-38846`): target 255 / dead / >4800 px
//! (`Vector2.Distance`) → `TargetClosest(faceTarget: false)` and
//! the RE-CHECK; still invalid → die.
//!
//! The death (`:38847-38856`): `life = 0`, `HitEffect()` (the dust
//! burst — the port's documented dust skip, the death removal rides
//! the sim's `hp > 0` retain), `active = false`, and the
//! `TETrainingDummy.TryGetAt` → `Deactivate()` (the tile-entity
//! seam — [`Style92Outcome::deactivate_dummy`] carries the site).
//!
//! ## RNG census (by construction)
//!
//! 1. The shuffle gate `Next(20)` (`:38812`) — EVERY tick.
//! 2. On a 0 roll, the variant `Next(15,18)` (`:38814`) — the
//!    second draw only fires on the gate's success.
//! `TargetClosest(faceTarget: false)` is deterministic.
//!
//! ## Seams (the wiring obligations)
//!
//! * The dummy never moves (no velocity writes) — the caller's
//!   gravity keeps it planted on its tile.
//! * The `HitSound` write rides [`Style92Outcome::hit_sound`]
//!   ((SoundType, Style) — the audio pass owns the playback).
//! * The death is [`Style92Outcome::died`] + the NPC's `hp = 0`
//!   (the sim's retain removes it); the tile-entity deactivation
//!   is a REQUEST ([`Style92Outcome::deactivate_dummy`]).
//! * The tile read is the port's `World::tile` (type 378 = the
//!   TargetDummy tile).
//!
//! See docs/ANALYSIS.md "The Target Dummy (W4-d32 aiStyle-92, code-only)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The type-488 `SetDefaults` row (`NPC.cs:14590-14605`), verbatim.
pub struct TargetDummyDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit15` (`ID/SoundID.cs:1271`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath2` (`ID/SoundID.cs:1316`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub value: f32,
    pub immortal: bool,
    pub net_always: bool,
}

/// `NPC.cs:14590-14605` — `else if (this.type == 488)`.
pub const TARGET_DUMMY: TargetDummyDefaults = TargetDummyDefaults {
    width: 18,
    height: 40,
    ai_style: 92,
    damage: 0,
    defense: 0,
    life_max: 1000,
    hit_sound: (3, 15),
    death_sound: (4, 2),
    knock_back_resist: 0.0,
    value: 0.0,
    immortal: true,
    net_always: true,
};

/// The per-tick environment.
pub struct Style92Env<'a> {
    pub world: &'a World,
    pub players: &'a [TargetCandidate],
    pub ticker: u64,
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style92Outcome {
    /// The shuffle's `HitSound` write (`:38817-38823`) —
    /// (SoundType, Style), only on a successful gate.
    pub hit_sound: Option<(i32, i32)>,
    /// The tile/target-gate death (`:38849-38851`): life 0 +
    /// HitEffect (the dust skip) + active false — the sim's retain
    /// removes the slot.
    pub died: bool,
    /// The `TETrainingDummy.Deactivate()` request (`:38852-38855`)
    /// — the tile site (ai[0], ai[1]).
    pub deactivate_dummy: Option<(i32, i32)>,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Distance` (the
/// style74 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// The target slot's candidate — the style74 `target_of` fallback.
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32)
}

/// One fixed step of the aiStyle-92 arm (`NPC.cs:38810-38857`), in
/// C# order. `npc` is the Target Dummy (type 488). The caller keeps
/// it planted (no movement writes) and drains the outcome.
pub fn style92_step(npc: &mut Npc, env: &mut Style92Env<'_>) -> Style92Outcome {
    let mut out = Style92Outcome::default();
    // `:38812-38826` — the hit-sound shuffle.
    if env.rng.next_max(20) == 0 {
        out.hit_sound = Some(match env.rng.next_range(15, 18) {
            15 => (3, 15),
            16 => (3, 16),
            _ => (3, 17),
        });
    }
    // `:38827` — the chained defense zero (the port's override seam
    // — the dummy's defense is 0 by row anyway).
    npc.defense_override = Some(0);
    // `:38828-38831` — the client bail folds past (the solo session).
    // `:38832-38838` — the tile gate.
    let mut flag98 = false;
    let num1276 = npc.ai[0] as i32;
    let num1277 = npc.ai[1] as i32;
    let tile_ok = env
        .world
        .tile(num1276, num1277)
        .is_some_and(|t| t.active() && t.kind == 378);
    if !tile_ok {
        flag98 = true;
    }
    // `:38839-38846` — the target gate + the re-check.
    if !flag98 {
        let (cx, cy) = npc.center();
        let invalid = |npc: &Npc| -> bool {
            if npc.target == 255 {
                return true;
            }
            // The C# indexes `Main.player[this.target]` — an
            // out-of-range slot is inactive (dead false).
            if npc.target < 0 || npc.target as usize >= env.players.len() {
                return false;
            }
            let p = target_of(npc, env.players);
            let (pcx, pcy) = candidate_center(p);
            p.dead || sqrt_f32(cx - pcx, cy - pcy) > 4800.0
        };
        if invalid(npc) {
            // `TargetClosest(faceTarget: false)` (`:38841`).
            target_closest(npc, env.world, env.players, false, 92, env.ticker);
            if invalid(npc) {
                flag98 = true;
            }
        }
    }
    // `:38847-38856` — the death.
    if flag98 {
        npc.hp = 0;
        // `HitEffect()` — the dust burst is the documented skip
        // (the port's gore/dust deviation); `active = false` rides
        // the sim's hp-retain.
        out.died = true;
        out.deactivate_dummy = Some((num1276, num1277));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 488-shaped NPC: no kind yet — the placeholder + the C# box
    /// (the style86 convention).
    fn dummy(x: f32, y: f32) -> Npc {
        let d = &TARGET_DUMMY;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.immortal = d.immortal;
        n
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

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// Plant the TargetDummy tile (378) at the site.
    fn with_dummy_tile(site: (i32, i32)) -> World {
        let mut w = air();
        if let Some(t) = w.tile_mut(site.0, site.1) {
            t.kind = 378;
            t.flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w
    }

    /// `NPC.cs:14590-14605` — the SetDefaults row, field by field.
    #[test]
    fn w4_d32_s92_the_defaults_row() {
        let d = &TARGET_DUMMY;
        assert_eq!((d.width, d.height), (18, 40));
        assert_eq!(d.ai_style, 92);
        assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 1000));
        assert_eq!(d.hit_sound, (3, 15), "SoundID.NPCHit15");
        assert_eq!(d.death_sound, (4, 2), "SoundID.NPCDeath2");
        assert_eq!(d.knock_back_resist, 0.0);
        assert_eq!(d.value, 0.0);
        assert!(d.immortal);
        assert!(d.net_always);
    }

    /// `:38812-38826` — the shuffle: every tick draws Next(20); a 0
    /// roll draws the Next(15,18) variant; the oracle pins both.
    #[test]
    fn w4_d32_s92_the_shuffle_draws_the_gate_then_the_variant() {
        let w = with_dummy_tile((10, 10));
        let players = [cand(200.0, 100.0)];
        let mut rng = UnifiedRandom::new(92);
        let mut oracle = UnifiedRandom::new(92);
        let mut n = dummy(160.0, 120.0);
        n.ai[0] = 10.0;
        n.ai[1] = 10.0;
        n.target = 0;
        let mut e = Style92Env { world: &w, players: &players, ticker: 0, rng: &mut rng };
        for _ in 0..60 {
            let out = style92_step(&mut n, &mut e);
            let gate = oracle.next_max(20);
            if gate == 0 {
                let v = oracle.next_range(15, 18);
                assert_eq!(out.hit_sound, Some((3, v)));
            } else {
                assert_eq!(out.hit_sound, None);
            }
            assert!(!out.died, "the planted dummy lives");
        }
        assert_eq!(rng.peek(), oracle.peek(), "the shuffle order");
    }

    /// `:38835-38838` + `:38847-38856` — a missing/wrong tile kills
    /// the dummy: hp 0 + the deactivate request at the site.
    #[test]
    fn w4_d32_s92_the_missing_tile_kills_and_deactivates() {
        let w = air(); // no tile at the site
        let players = [cand(200.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = dummy(160.0, 120.0);
        n.ai[0] = 10.0;
        n.ai[1] = 10.0;
        n.target = 0;
        let mut e = Style92Env { world: &w, players: &players, ticker: 0, rng: &mut rng };
        let out = style92_step(&mut n, &mut e);
        assert!(out.died);
        assert_eq!(n.hp, 0);
        assert_eq!(out.deactivate_dummy, Some((10, 10)));
    }

    /// `:38839-38846` — the dead-player re-check: the retarget
    /// (faceTarget false) cannot save it → die.
    #[test]
    fn w4_d32_s92_the_dead_player_recheck_kills() {
        let w = with_dummy_tile((10, 10));
        let mut dead = cand(200.0, 100.0);
        dead.dead = true;
        let players = [dead];
        let mut rng = UnifiedRandom::new(7);
        let mut n = dummy(160.0, 120.0);
        n.ai[0] = 10.0;
        n.ai[1] = 10.0;
        n.target = 0;
        let mut e = Style92Env { world: &w, players: &players, ticker: 0, rng: &mut rng };
        let out = style92_step(&mut n, &mut e);
        assert!(out.died, "the only player is dead");
        assert_eq!(out.deactivate_dummy, Some((10, 10)));
    }

    /// `:38827` — the chained defense zero rides the override every
    /// tick; the planted dummy never moves.
    #[test]
    fn w4_d32_s92_the_defense_zero_and_the_planted_body() {
        let w = with_dummy_tile((10, 10));
        let players = [cand(200.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = dummy(160.0, 120.0);
        n.ai[0] = 10.0;
        n.ai[1] = 10.0;
        n.target = 0;
        let mut e = Style92Env { world: &w, players: &players, ticker: 0, rng: &mut rng };
        let _ = style92_step(&mut n, &mut e);
        assert_eq!(n.defense_override, Some(0));
        assert_eq!((n.x, n.y, n.vx, n.vy), (160.0, 120.0, 0.0, 0.0));
    }
}
