//! W4-d25 (aiStyle-25) — the chest/mimic family, `NPC.AI`'s
//! `else if (this.aiStyle == 25)` block (`NPC.cs:25538-25630`), ported
//! 1:1 as a PURE machine for the TWO unported members (code-only mode:
//! no compiler/tests were run on this branch; the witness tests below
//! are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 25` rows, whole-file grep)
//!
//! THREE vanilla members:
//! * Mimic 85 (`NPCID.cs` `Mimic = 85`, `SetDefaults`
//!   `NPC.cs:9905-9925`: 24×24, dmg 80, def 30, 500 HP, kb 0.3,
//!   NPCHit4/NPCDeath6, value 100000, rarity 4; pre-hardmode
//!   dmg 30 / def 12 / 300 HP / value `Item.buyPrice(0, 2)` = 200);
//! * PresentMimic 341 (`NPC.cs:13314-13326`) — ALREADY PORTED and
//!   WIRED: `npc_ai_frost_mob.rs::present_mimic_ai_retarget` (S1-D-3
//!   W-6, the kind-gated axis arm `25 if kind == PresentMimic`).
//!   VERIFY-ONLY here: that machine is unchanged by this census;
//! * IceMimic 629 (`IceMimic = 629`, `NPC.cs:17085-17106`: the Mimic's
//!   row + `coldDamage = true`; the same pre-hardmode fold).
//! The port has no `NpcKind::Mimic`/`IceMimic` yet, so this module
//! lands UNWIRED (the `npc_ai_style17.rs` pre-wiring precedent): a
//! pure fn keyed on the vanilla type, reachable and testable before
//! the kinds land. The wiring step adds the kinds + the axis routing.
//!
//! ## The machine (verbatim, the 85/629 fold)
//!
//! * `flag40 = type == 341 && !Main.snowMoon` (`:25540`) — FALSE for
//!   both members here (the 341 terms live in the frost module).
//! * the one-time placement (`:25541-25557`): `position.X += 8f`,
//!   then the layer pick — `y/16 > Main.UnderworldLayer`
//!   (`maxTilesY - 200`, `Main.cs:2790`) → `ai[3] = 3`;
//!   `(double)(y/16) > Main.worldSurface` → `TargetClosest()` +
//!   `ai[3] = 2`; else `ai[3] = 1`.
//! * the 341/629 override (`:25558-25561`): 629 forces `ai[3] = 1`.
//! * the disguised idle (`:25562-25584`): `TargetClosest()` (flag40
//!   false); the movement gate `vx != 0 || vy < 0 || (double)vy >
//!   0.3` wakes (`ai[0] = 1`, netUpdate, return); else the ±100
//!   int-truncated inflated-box overlap (XNA `Rectangle.Intersects`,
//!   strict `<`) or any damage (`life < lifeMax`) wakes.
//! * the grounded hop cycle (`:25585-25621`): `ai[2]++`; the period
//!   is 12 while `ai[1] == 0` else 20; below it `vx *= 0.9` + return;
//!   at it `ai[2] = 0`, `TargetClosest()`, `direction == 0 → -1`,
//!   `spriteDirection = direction`, `ai[1]++`; the SECOND hop is
//!   (2.5·dir, −8) with `ai[1] = 0`, the first (3.5·dir, −4);
//!   netUpdate.
//! * the airborne drift (`:25622-25629`): ±0.1 toward the facing
//!   while |vx| < 1 on the facing's side.
//!
//! Seam: writes position.X (the one-time nudge), velocity,
//! `ai[0..3]`, `direction` (`Npc::facing`), `spriteDirection`,
//! `netUpdate` (+ the target fields through `target_closest`). Does
//! NOT integrate: the mimic is a grounded walker (no
//! noGravity/noTileCollide row) — the wiring caller owns the gravity +
//! tile-collision tail (the frost module's `walker_integrate`
//! precedent). `Main.netMode == 1` folds away (the port is the host).
//!
//! ## RNG by construction
//!
//! The arm draws NO `Main.rand` — `NPC.cs:25538-25630` contains no
//! `Next`/`NextFloat`; neither does `TargetClosest`. The fn takes no
//! RNG parameter; `the_style25_arm_draws_no_rng` pins a
//! `UnifiedRandom` state across the idle, wake and hop ticks
//! (written, not run).
#![allow(dead_code)] // UNWIRED (W4-d25): the Mimic/IceMimic kinds + the axis routing are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// One member's `SetDefaults` row (the census above).
/// `SoundID.NPCHit4`/`NPCDeath6` = `LegacySoundStyle(3, 4)`/`(4, 6)`.
/// The pre-hardmode tier (`:9918-9924` / `:17099-17105`) is carried
/// as the `_nm` fields (`Item.buyPrice(0, 2)` = 200, `Item.cs`).
pub struct MimicDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub hit_sound: (i32, i32),
    pub death_sound: (i32, i32),
    pub value: f32,
    pub rarity: i32,
    pub cold_damage: bool,
    /// The `!Main.hardMode` fold: (damage, defense, lifeMax, value).
    pub pre_hardmode: (i32, i32, i32, f32),
}

/// `NPC.cs:9905-9925` — `else if (this.type == 85)` (Mimic).
pub const MIMIC: MimicDefaults = MimicDefaults {
    ty: 85,
    width: 24,
    height: 24,
    ai_style: 25,
    damage: 80,
    defense: 30,
    life_max: 500,
    knock_back_resist: 0.3,
    hit_sound: (3, 4),
    death_sound: (4, 6),
    value: 100000.0,
    rarity: 4,
    cold_damage: false,
    pre_hardmode: (30, 12, 300, 200.0),
};

/// `NPC.cs:17085-17106` — `else if (this.type == 629)` (IceMimic).
pub const ICE_MIMIC: MimicDefaults = MimicDefaults {
    ty: 629,
    width: 24,
    height: 24,
    ai_style: 25,
    damage: 80,
    defense: 30,
    life_max: 500,
    knock_back_resist: 0.3,
    hit_sound: (3, 4),
    death_sound: (4, 6),
    value: 100000.0,
    rarity: 4,
    cold_damage: true,
    pre_hardmode: (30, 12, 300, 200.0),
};

/// The target slot's candidate — the `npc_ai_style17.rs::target_of`
/// fallback (an unset/stale slot reads slot 0, the documented
/// session-player stand-in).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// XNA `Rectangle.Intersects` (`Rectangle.cs`): strict `<` on both
/// axes (the `npc_ai_style17.rs` transcription).
fn rects_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    b.0 < a.0 + a.2 && a.0 < b.0 + b.2 && b.1 < a.1 + a.3 && a.1 < b.1 + b.3
}

/// The aiStyle-25 arm (`NPC.cs:25538-25630`) for Mimic 85 / IceMimic
/// 629, one fixed step. `flag40` folds FALSE (neither type is 341), so
/// both `TargetClosest()` sites (`:25566` the idle, `:25601` the hop
/// launch) run, plus the layer pick's (`:25550`). The 629 override
/// forces `ai[3] = 1` (`:25558-25561`). RNG-free by construction; the
/// caller owns the walker integration tail.
pub fn ai_style25(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
) {
    debug_assert!(ty == 85 || ty == 629, "the 341 terms ride npc_ai_frost_mob");
    let (w, h) = npc.size();
    // `:25540` — flag40 = type == 341 && !snowMoon: FALSE here.
    // `:25541-25557` — the one-time placement nudge + layer pick.
    if npc.ai[3] == 0.0 {
        npc.x += 8.0;
        let underworld_layer = (world.height - 200) as f32;
        if npc.y / 16.0 > underworld_layer {
            npc.ai[3] = 3.0;
        } else if (npc.y / 16.0) as f64 > world.world_surface {
            // `:25550` — TargetClosest().
            target_closest(npc, world, players, true, 25, ticker);
            npc.ai[3] = 2.0;
        } else {
            npc.ai[3] = 1.0;
        }
    }
    // `:25558-25561` — the 341/629 override (629 only here).
    if ty == 629 {
        npc.ai[3] = 1.0;
    }
    if npc.ai[0] == 0.0 {
        // `:25564-25567` — the idle re-target (!flag40) + the
        // netMode == 1 return folds away (the port is the host).
        target_closest(npc, world, players, true, 25, ticker);
        // `:25572-25577` — the movement wake; `(double)vy > 0.3` is
        // the f32→f64 promote of the FIELD value (the style17 note).
        if npc.vx != 0.0 || npc.vy < 0.0 || (npc.vy as f64) > 0.3 {
            npc.ai[0] = 1.0;
            npc.net_update = true;
            return;
        }
        // `:25578-25583` — the ±100 inflated int-truncated boxes, or
        // any damage.
        let p = target_of(npc, players);
        let player_rect = (p.x as i32, p.y as i32, p.width, p.height);
        let own_rect = (
            npc.x as i32 - 100,
            npc.y as i32 - 100,
            w as i32 + 200,
            h as i32 + 200,
        );
        if rects_intersect(own_rect, player_rect) || npc.hp < npc.max_hp {
            npc.ai[0] = 1.0;
            npc.net_update = true;
        }
    } else if npc.vy == 0.0 {
        // `:25585-25621` — the grounded hop cycle.
        npc.ai[2] += 1.0;
        let num335 = if npc.ai[1] == 0.0 { 12.0 } else { 20.0 };
        if npc.ai[2] < num335 {
            npc.vx *= 0.9;
            return;
        }
        npc.ai[2] = 0.0;
        // `:25599-25602` — TargetClosest() (!flag40).
        target_closest(npc, world, players, true, 25, ticker);
        // `:25603-25607`.
        if npc.facing == 0 {
            npc.facing = -1;
        }
        npc.sprite_dir = npc.facing;
        // `:25608-25619` — the alternating hops: the second is the
        // high (2.5, −8) with the ai[1] reset, the first (3.5, −4).
        npc.ai[1] += 1.0;
        if npc.ai[1] == 2.0 {
            npc.vx = npc.facing as f32 * 2.5;
            npc.vy = -8.0;
            npc.ai[1] = 0.0;
        } else {
            npc.vx = npc.facing as f32 * 3.5;
            npc.vy = -4.0;
        }
        npc.net_update = true;
    } else if npc.facing == 1 && npc.vx < 1.0 {
        // `:25622-25625`.
        npc.vx += 0.1;
    } else if npc.facing == -1 && npc.vx > -1.0 {
        // `:25626-25629`.
        npc.vx -= 0.1;
    }
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins
    //! one C# branch against its citation (`NPC.cs:25538-25630`).
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    /// A tall world so the layer pick's three bands are reachable
    /// (UnderworldLayer = 1200 − 200 = 1000; worldSurface = 300).
    fn air() -> World {
        World::new("t".into(), 200, 1200)
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

    /// A stand-in body with the mimics' shared 24×24 box (the kinds
    /// are unported; both members share the machine).
    fn chest(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((24.0, 24.0));
        n.hp = 500;
        n.max_hp = 500;
        n.target = 0;
        n
    }

    /// The `SetDefaults` rows (`NPC.cs:9905-9925` / `:17085-17106`).
    #[test]
    fn the_defaults_rows_match_the_census() {
        for (row, ty, cold) in [(MIMIC, 85, false), (ICE_MIMIC, 629, true)] {
            assert_eq!(row.ty, ty);
            assert_eq!((row.width, row.height), (24, 24));
            assert_eq!(row.ai_style, 25);
            assert_eq!((row.damage, row.defense, row.life_max), (80, 30, 500));
            assert_eq!(row.knock_back_resist, 0.3);
            assert_eq!(row.hit_sound, (3, 4));
            assert_eq!(row.death_sound, (4, 6));
            assert_eq!(row.value, 100000.0);
            assert_eq!(row.rarity, 4);
            assert_eq!(row.cold_damage, cold);
            assert_eq!(row.pre_hardmode, (30, 12, 300, 200.0));
        }
    }

    /// `:25541-25557` — the first tick nudges X by 8 and picks the
    /// layer: surface → 1, cavern → 2 (with the retarget), underworld
    /// → 3.
    #[test]
    fn the_first_tick_nudges_and_picks_the_layer() {
        let w = air();
        let p = cand(5000.0, 1600.0);
        // Surface (y/16 = 100 <= 300): ai3 = 1.
        let mut n = chest(1000.0, 1600.0);
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.x, 1008.0);
        assert_eq!(n.ai[3], 1.0);
        // Cavern (y/16 = 500 > 300, <= 1000): ai3 = 2 + retarget.
        let mut n = chest(1000.0, 8000.0);
        n.target = 255;
        ai_style25(85, &mut n, &w, &[cand(5000.0, 8000.0)], 3);
        assert_eq!(n.ai[3], 2.0);
        assert_eq!(n.target, 0, ":25550 — the layer pick re-targets");
        assert_eq!(n.target_set_frame, 3);
        // Underworld (y/16 = 1100 > 1000): ai3 = 3.
        let mut n = chest(1000.0, 17600.0);
        ai_style25(85, &mut n, &w, &[cand(5000.0, 17600.0)], 0);
        assert_eq!(n.ai[3], 3.0);
    }

    /// `:25558-25561` — the 629 override forces ai[3] = 1 even after
    /// the cavern pick; 85 keeps its pick.
    #[test]
    fn the_ice_mimic_forces_ai3_to_one() {
        let w = air();
        let mut n = chest(1000.0, 8000.0);
        ai_style25(629, &mut n, &w, &[cand(5000.0, 8000.0)], 0);
        assert_eq!(n.ai[3], 1.0);
        let mut n = chest(1000.0, 8000.0);
        ai_style25(85, &mut n, &w, &[cand(5000.0, 8000.0)], 0);
        assert_eq!(n.ai[3], 2.0);
    }

    /// `:25562-25584` — the idle wakes on movement (the double-promote
    /// 0.3 gate), on the ±100 overlap, or on damage; a far untouched
    /// chest stays disguised.
    #[test]
    fn the_idle_wake_conditions() {
        let w = air();
        let far = cand(5000.0, 1600.0);
        // Untouched: stays asleep (ai3 pre-latched to skip the nudge).
        let mut n = chest(1000.0, 1600.0);
        n.ai[3] = 1.0;
        ai_style25(85, &mut n, &w, &[far], 0);
        assert_eq!(n.ai[0], 0.0);
        assert!(!n.net_update);
        // vy > 0.3 wakes (the f32 field promotes above the f64 0.3).
        let mut n2 = n;
        n2.vy = 0.4;
        ai_style25(85, &mut n2, &w, &[far], 0);
        assert_eq!(n2.ai[0], 1.0);
        assert!(n2.net_update);
        // The overlap wakes (player inside the inflated box).
        let mut n3 = n;
        ai_style25(85, &mut n3, &w, &[cand(1050.0, 1600.0)], 0);
        assert_eq!(n3.ai[0], 1.0);
        // Damage wakes.
        let mut n4 = n;
        n4.hp = 499;
        ai_style25(85, &mut n4, &w, &[far], 0);
        assert_eq!(n4.ai[0], 1.0);
    }

    /// `:25587-25597` — the hop cadence: 12 ticks while ai[1] == 0
    /// (the ×0.9 damp below it), 20 after the first hop.
    #[test]
    fn the_hop_cadence_is_twelve_then_twenty() {
        let w = air();
        let p = cand(5000.0, 1600.0);
        let mut n = chest(1000.0, 1600.0);
        n.ai[3] = 1.0;
        n.ai[0] = 1.0;
        n.ai[2] = 10.0;
        n.vx = 1.0;
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.ai[2], 11.0);
        assert_eq!(n.vx, 1.0f32 * 0.9, "below the period: the damp");
        // The 12th tick launches the FIRST hop (3.5, −4).
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!((n.vx, n.vy), (3.5, -4.0));
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.sprite_dir, n.facing);
        assert!(n.net_update);
        // Grounded again: the period is now 20.
        n.vy = 0.0;
        n.ai[2] = 18.0;
        n.net_update = false;
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.ai[2], 19.0, "19 < 20: still damping");
        assert!(!n.net_update);
        ai_style25(85, &mut n, &w, &[p], 0);
        // The SECOND hop: (2.5, −8) with the ai[1] reset.
        assert_eq!((n.vx, n.vy), (2.5, -8.0));
        assert_eq!(n.ai[1], 0.0);
    }

    /// `:25603-25606` — a zero direction flips to −1 before the hop
    /// (the launch retarget with only dead candidates leaves the
    /// facing untouched, so the zero survives to the flip).
    #[test]
    fn the_zero_direction_flips_left() {
        let w = air();
        let mut n2 = chest(1000.0, 1600.0);
        n2.ai[3] = 1.0;
        n2.ai[0] = 1.0;
        n2.ai[2] = 11.0;
        n2.facing = 0;
        let mut dead = cand(5000.0, 1600.0);
        dead.dead = true;
        ai_style25(85, &mut n2, &w, &[dead], 0);
        assert_eq!(n2.facing, -1, ":25603-25606");
        assert_eq!(n2.sprite_dir, -1);
        assert_eq!(n2.vx, -3.5);
    }

    /// `:25622-25629` — the airborne drift: ±0.1 toward the facing
    /// while on the facing's side of ±1; past it, no drift.
    #[test]
    fn the_airborne_drifts_toward_the_facing() {
        let w = air();
        let p = cand(5000.0, 1600.0);
        let mut n = chest(1000.0, 1600.0);
        n.ai[3] = 1.0;
        n.ai[0] = 1.0;
        n.vy = -2.0;
        n.vx = 0.5;
        n.facing = 1;
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.vx, 0.5f32 + 0.1);
        n.vx = 1.5;
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.vx, 1.5, "past +1: no drift");
        n.facing = -1;
        n.vx = -0.5;
        ai_style25(85, &mut n, &w, &[p], 0);
        assert_eq!(n.vx, -0.5f32 - 0.1);
    }

    /// The RNG census: the arm takes no stream and the C# block has
    /// no `Next` — a probe stream is untouched across idle/wake/hops.
    #[test]
    fn the_style25_arm_draws_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(25);
        let probe = rng.clone();
        let mut n = chest(1000.0, 1600.0);
        let p = cand(1050.0, 1600.0);
        for _ in 0..80 {
            ai_style25(85, &mut n, &w, &[p], 0);
            if n.vy < 0.0 {
                n.vy = 0.0; // keep it grounded for the hop cycle
            }
        }
        assert_eq!(rng, probe, "the arm never touches the stream");
    }
}
