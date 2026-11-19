//! The aiStyle-29 family arm — the Hungry (`NPC.AI()`'s
//! `else if (this.aiStyle == 29)` block, `NPC.cs:26209-26404`).
//!
//! S1-D-3 (aiStyle-29 round, code-only mode): this module is the faithful
//! 1:1 port of the arm as a pure function over `Npc` + a small context
//! seam. It is NOT wired into `Npc::update_rng`'s style axis yet — the
//! live `npc.rs::update_hungry` keeps running until the wiring round
//! (the sibling D-3 rounds land the same way: module + tests first,
//! dispatch claim second). Every other file is byte-identical except the
//! one `mod` line in `main.rs` and the docs.
//!
//! The only type on this style is `NPCID.TheHungry` (115, the WoF's
//! leech minion); its `SetDefaults` row (`NPC.cs:10378-10392`): 30×30,
//! aiStyle 29, damage 30, defense 10, lifeMax 240,
//! `SoundID.NPCHit9`/`NPCDeath11`, noGravity, noTileCollide, behindTiles,
//! knockBackResist 1.1. `NPCID.TheHungryII` (116, `NPC.cs:10393-10404`)
//! is NOT on this style — it is the 30×32 aiStyle-2 jumper (80 HP). The
//! port's static tables already carry 115 (`npc/data.rs`:
//! `ai_style() == 29`, size (30, 30), def 10, 240 HP). There are no
//! per-type branches inside the arm — the whole block below is the
//! family. The init spawns live on the wall's aiStyle-27 arm
//! (`NPC.cs:26041-26051`, the 11 Hungries at `ai[0] = i * 0.1 - 0.05`;
//! the port's sim resolve: `world_sim/npcs.rs:3605-3613`). The expert
//! respawn roll above it (`:25999-26039`) draws `Main.rand` but lives
//! on the WALL's arm, not this one.
//!
//! RNG by construction: the C# arm (`NPC.cs:26209-26404`) draws NO
//! `Main.rand` — every pacing value is a deterministic counter
//! (`ai[1]` the recoil pause, `ai[2]` the dive cycle) or an hp-band
//! comparison; `TargetClosest` (`NPC.cs:77652-77781`) is a deterministic
//! Manhattan scan. The port therefore takes no RNG parameter at all;
//! the witness test `the_arm_is_deterministic_without_an_rng` pins two
//! identical inputs to identical state.
//!
//! Faithfulness notes vs the live `update_hungry` (the wiring round
//! replaces it with this arm):
//! * The anchor X is the wall's `position.X + (float)(width / 2)` — the
//!   wall's INTEGER half-width (`:26280`); the live arm uses `wof_x`
//!   alone (the wall's left edge), a ~50-px offset.
//! * The hp ladder compares `(double)life < (double)lifeMax * k`
//!   (`:26225`/`:26238`) in f64 against the wall's raw ints; the live
//!   arm reads the sim's precomputed f32 `World.wof_hp_ratio`.
//! * `TargetClosest()` runs right after the wall-presence gate
//!   (`:26220`) — the live arm never selects a target.
//! * `damage = defDamage` resets EVERY tick (`:26223`); the tier writes
//!   `GetAttackDamage_ScaledByDifficulty(num388)` (`:26253`,
//!   `NPC.cs:7036-7039` — the classic multiplier is 1, so 60/45). The
//!   live arm's override never falls back to the kind's base 30.
//! * The high tier does NOT reset the defense to defDefense in the
//!   classic path (`:26225-26250` only writes 30/20; the `defense =
//!   defDefense` restore is expert-only, `:26257`) — the live arm
//!   clears its override at full hp, dropping 20→10 a tick early.
//! * `ai[2]++` runs AFTER the anchor reads (`:26284`), and the dive's
//!   `(int)(num387 * 1.3f)` truncates (`:26287`).
//! * The aim deltas use INTEGER half-extents on BOTH entities
//!   (`:26294-26295`, `(float)(width / 2)`), not `center()`'s float
//!   halves; the leash clamp reuses `num393` as the scale (`:26301`).
//! * The recoil chain has the C#'s third arm: `ai[1] < 0` resets to 0
//!   (`:26389-26392`) — the live arm lets a negative pause run forever.
//! * `spriteDirection = ±1` rides the two one-sided aim ifs
//!   (`:26395`/`:26400`); a dead-centre aim (`num391 == 0`) keeps the
//!   previous flip AND rotation. The left-facing rotation adds the
//!   literal `3.14f` (`:26401`, not π). The live arm writes neither
//!   the flip nor the +3.14.
//! * `justHit` (`:26211-26214`) runs BEFORE the wall-presence gate —
//!   the port has no per-tick flag, so `Style29Context::just_hit` is
//!   the seam (the wiring maps the strike tick; the live arm's
//!   `hurt_cd > 0` stand-in re-arms the pause for the whole cooldown).
//! * The arm does NOT integrate `position += velocity` — the Hungry is
//!   `noTileCollide` (`:10390`), so `UpdateNPC` integrates after `AI()`;
//!   the caller owns the step (the live arm integrates inline).
//! * `this.active = false` (`:26217`) has no port field — the port's
//!   death convention `hp = 0` stands in (same as the live arm).
//! * `Lighting.AddLight(centre/16, 0.3, 0.2, 0.1)` (`:26403`) is the
//!   entity-light follow-up (S1-E-5's NPC family) — skipped, documented
//!   in ANALYSIS.
//! * The expert arms (`:26229-26236`/`:26242-26249` the accel bonuses,
//!   `:26255-26279` the defense restore + the whoAmI-mod leash ladder,
//!   `:26338-26367` the hp-scaled speed cap + the wall-side catch-up
//!   +6) are ported verbatim behind `Style29Context::expert` — the port
//!   is single-difficulty today, so the wiring passes false. Note
//!   `:26341` divides two INTs (`life / lifeMax` → 1 only at full hp,
//!   0 at ANY damaged hp — so all four cap bands fire) — kept verbatim,
//!   not "fixed" to a float ratio.
#![allow(dead_code)] // wiring deferred (code-only mode) — the dispatch claim is the follow-up round

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The WoF state the arm reads through `Main`'s statics + the wall NPC.
/// The caller (the future style-axis arm) fills it from the wall's own
/// slot — the sim's `World.wof_*` write-through
/// (`world_sim/npcs.rs:3646-3649`) carries presence/x/dir today; the
/// raw life ints, the wall width and the INT band statics are the
/// wiring round's additions.
#[derive(Clone, Copy, Debug)]
pub struct Style29Context {
    /// `Main.wofNPCIndex >= 0` (`NPC.cs:26215`, `Main.cs:755`).
    pub wof_present: bool,
    /// `Main.npc[Main.wofNPCIndex].life` (`:26225`/`:26238`/`:26341`).
    pub wof_life: i32,
    /// `Main.npc[Main.wofNPCIndex].lifeMax` (the ladder's f64 base).
    pub wof_life_max: i32,
    /// `Main.npc[Main.wofNPCIndex].position.X` (`:26280`).
    pub wof_x: f32,
    /// `Main.npc[Main.wofNPCIndex].width` (`:26280` — an INT; the
    /// anchor adds `(float)(width / 2)`, integer division).
    pub wof_width: i32,
    /// `Main.npc[Main.wofNPCIndex].velocity.X` (`:26359`/`:26363`,
    /// the expert catch-up gate).
    pub wof_vx: f32,
    /// `Main.npc[Main.wofNPCIndex].Center.X` (`:26359`/`:26363`;
    /// `Entity.Center` uses FLOAT halves, `Entity.cs:54`).
    pub wof_center_x: f32,
    /// `Main.wofDrawAreaTop` (`Main.cs:757`) — an INT in the C#.
    pub wof_draw_top: i32,
    /// `Main.wofDrawAreaBottom` (`Main.cs:759`) — an INT in the C#.
    pub wof_draw_bottom: i32,
    /// `Main.expertMode` (`:26229`/`:26242`/`:26255`/`:26338`).
    pub expert: bool,
    /// `NPC.justHit` (`:26211`) — the strike-tick flag. The port has no
    /// per-tick flag field yet; the wiring maps the hit event (the
    /// live arm's `hurt_cd > 0` stand-in is NOT this flag — it stays
    /// true for the whole cooldown).
    pub just_hit: bool,
    /// `base.whoAmI` (`:26258-26277`, the expert leash ladder) — the
    /// NPC's slot index (`Npc::slot`).
    pub who_am_i: i32,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp — not
    /// plumbed into `Npc::update` yet (same stand-in as `update_zombie`).
    pub ticker: u64,
}

/// `NPC.GetAttackDamage_ScaledByDifficulty` (`NPC.cs:7036-7039`):
/// `(int)(normalDamage * EnemyDamageMultiplier.Sample(difficulty))`.
/// The curve's keys are Journey 0.5→0.5, Master 3→3, Legendary
/// 4→5.3333335 (`GameDifficultyData.cs:84`); Classic is difficulty 1
/// (`GameDifficultyLevel.cs:18`) — between Journey and Master the
/// sample is exactly 1, so the classic tier damage is `num388` itself.
/// The port is single-difficulty (classic); the expert/master sample
/// lands with the difficulty system.
fn attack_damage_scaled_by_difficulty(normal_damage: f32) -> i32 {
    normal_damage as i32
}

/// `NPC.AI()`'s aiStyle-29 arm (`NPC.cs:26209-26404`), one tick.
/// `players` is the target-scan slice (at least one slot — the C# always
/// has the 255 `Main.player` slots; the port's session player is slot 0).
pub fn style29_step(npc: &mut Npc, world: &World, ctx: &Style29Context, players: &[TargetCandidate]) {
    // `:26211-26214` — justHit arms the recoil pause BEFORE the
    // wall-presence gate (a hit on the wall's death tick still writes).
    if ctx.just_hit {
        npc.ai[1] = 10.0;
    }
    // `:26215-26219` — no wall: deactivate and return.
    if !ctx.wof_present {
        npc.hp = 0; // `this.active = false` — the port's death convention.
        return;
    }
    // `:26220` — TargetClosest (faceTarget = true; the facing write is
    // the selection's, the arm's own flips below are spriteDirection).
    target_closest(npc, world, players, true, 29, ctx.ticker);
    // `:26221-26224` — the accel, the leash, the per-tick damage reset
    // (`damage = defDamage`; the port's override-None = the kind base).
    let mut num386 = 0.1f32;
    let mut num387 = 300.0f32;
    npc.damage_override = None;
    let mut num388 = 0i32;
    // `:26225-26250` — the hp ladder in f64 against the wall's raw ints.
    let below = |k: f64| (ctx.wof_life as f64) < (ctx.wof_life_max as f64) * k;
    if below(0.5) {
        // `:26227-26236`.
        num388 = 60;
        npc.defense_override = Some(30);
        if !ctx.expert {
            num387 = 700.0;
        } else {
            num386 += 0.066;
        }
    } else if below(0.75) {
        // `:26240-26249`.
        num388 = 45;
        npc.defense_override = Some(20);
        if !ctx.expert {
            num387 = 500.0;
        } else {
            num386 += 0.033;
        }
    }
    // NOTE: the classic high tier writes NO defense — the C# only
    // restores defDefense in the expert block below (`:26257`).
    // `:26251-26254` — the tier damage through the difficulty scale.
    if num388 > 0 {
        npc.damage_override = Some(attack_damage_scaled_by_difficulty(num388 as f32));
    }
    if ctx.expert {
        // `:26255-26279` — the defense restore + the whoAmI-mod ladder.
        npc.defense_override = None; // `defense = defDefense`
        if ctx.who_am_i % 4 == 0 {
            num387 *= 1.75;
        }
        if ctx.who_am_i % 4 == 1 {
            num387 *= 1.5;
        }
        if ctx.who_am_i % 4 == 2 {
            num387 *= 1.25;
        }
        if ctx.who_am_i % 3 == 0 {
            num387 *= 1.5;
        }
        if ctx.who_am_i % 3 == 1 {
            num387 *= 1.25;
        }
        num387 *= 0.75;
    }
    // `:26280-26283` — the anchor: the wall's X + its INTEGER half-width,
    // the band row at `ai[0]`. `:26281`'s `y2 = wof.position.Y` is a dead
    // store — `:26283` overwrites it before any read. The band statics
    // are INTs (`Main.cs:757`/`:759`); the difference widens to float.
    let num389 = ctx.wof_x + (ctx.wof_width / 2) as f32;
    let num390 = (ctx.wof_draw_bottom - ctx.wof_draw_top) as f32;
    let y2 = ctx.wof_draw_top as f32 + num390 * npc.ai[0];
    // `:26284-26292` — the dive cycle: after 100 ticks the leash grows
    // ×1.3 with an INT truncation, resetting at 200.
    npc.ai[2] += 1.0;
    if npc.ai[2] > 100.0 {
        // `:26287` `(int)(num387 * 1.3f)` - the FLOAT literal: the f32
        // product (700 * 1.3f32 = 909.99994 -> 909), not the f64 one.
        num387 = (num387 * 1.3f32) as i32 as f32;
        if npc.ai[2] > 200.0 {
            npc.ai[2] = 0.0;
        }
    }
    // `:26293-26296` — the aim deltas with INTEGER half-extents on both
    // entities (`(float)(width / 2)`, not `center()`'s float halves).
    let (w, h) = npc.size();
    let p = &players[npc.target as usize];
    let mut num391 =
        p.x + (p.width / 2) as f32 - (w as i32 / 2) as f32 - num389;
    let mut num392 =
        p.y + (p.height / 2) as f32 - (h as i32 / 2) as f32 - y2;
    let mut num393 = (num391 * num391 + num392 * num392).sqrt();
    if npc.ai[1] == 0.0 {
        // `:26299-26304` — the leash clamp: num393 becomes the scale.
        if num393 > num387 {
            num393 = num387 / num393;
            num391 *= num393;
            num392 *= num393;
        }
        // `:26305-26336` — the approach with the sign-flip double-add
        // (`num386 * 2.5f` — 0.25 classic, NOT a hardcoded 0.25).
        if npc.x < num389 + num391 {
            npc.vx += num386;
            if npc.vx < 0.0 && num391 > 0.0 {
                npc.vx += num386 * 2.5;
            }
        } else if npc.x > num389 + num391 {
            npc.vx -= num386;
            if npc.vx > 0.0 && num391 < 0.0 {
                npc.vx -= num386 * 2.5;
            }
        }
        if npc.y < y2 + num392 {
            npc.vy += num386;
            if npc.vy < 0.0 && num392 > 0.0 {
                npc.vy += num386 * 2.5;
            }
        } else if npc.y > y2 + num392 {
            npc.vy -= num386;
            if npc.vy > 0.0 && num392 < 0.0 {
                npc.vy -= num386 * 2.5;
            }
        }
        // `:26337-26367` — the speed cap: 4 classic; the expert arm
        // scales it by the wall's INT-divided hp ratio (`:26341` —
        // `life / lifeMax` is INTEGER division: 1 only at full hp, 0
        // at any damaged hp → all four bands fire) and adds the
        // wall-side catch-up +6.
        let mut num394 = 4.0f32;
        if ctx.expert && ctx.wof_present {
            let num396 = (ctx.wof_life / ctx.wof_life_max) as f32;
            let mut num395 = 1.5f32;
            if (num396 as f64) < 0.75 {
                num395 += 0.7;
            }
            if (num396 as f64) < 0.5 {
                num395 += 0.7;
            }
            if (num396 as f64) < 0.25 {
                num395 += 0.9;
            }
            if (num396 as f64) < 0.1 {
                num395 += 0.9;
            }
            num394 += (num395 * 1.25 + 0.3) * 0.35;
            let npc_center_x = npc.x + w / 2.0; // Entity.Center (float halves)
            if npc_center_x < ctx.wof_center_x && ctx.wof_vx > 0.0 {
                num394 += 6.0;
            }
            if npc_center_x > ctx.wof_center_x && ctx.wof_vx < 0.0 {
                num394 += 6.0;
            }
        }
        // `:26368-26383` — the four one-sided clamps.
        if npc.vx > num394 {
            npc.vx = num394;
        }
        if npc.vx < 0.0 - num394 {
            npc.vx = 0.0 - num394;
        }
        if npc.vy > num394 {
            npc.vy = num394;
        }
        if npc.vy < 0.0 - num394 {
            npc.vy = 0.0 - num394;
        }
    } else if npc.ai[1] > 0.0 {
        // `:26385-26388` — the recoil pause ticks down.
        npc.ai[1] -= 1.0;
    } else {
        // `:26389-26392` — a negative pause snaps to 0.
        npc.ai[1] = 0.0;
    }
    // `:26393-26402` — the two one-sided aim ifs: the sprite flip and
    // the rotation (the left arm adds the literal 3.14f). A dead-centre
    // aim (num391 == 0) writes NEITHER — the previous flip/rotation hold.
    if num391 > 0.0 {
        npc.sprite_dir = 1;
        // `:26396` `(float)Math.Atan2(num392, num391)` - the double seam:
        // widen, atan2 in f64, cast back (the house Math.* pattern).
        npc.rotation = (num392 as f64).atan2(num391 as f64) as f32;
    }
    if num391 < 0.0 {
        npc.sprite_dir = -1;
        npc.rotation = (num392 as f64).atan2(num391 as f64) as f32 + 3.14;
    }
    // `:26403` — Lighting.AddLight(centre/16, 0.3, 0.2, 0.1): the
    // entity-light follow-up (S1-E-5), skipped here.
    // No `position += velocity` here: the Hungry is noTileCollide
    // (`NPC.cs:10390`), so `UpdateNPC` integrates after `AI()`.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// The session player as the scan sees it (the `update_zombie`
    /// wiring's stock inputs: no aggro/stealth/tank systems yet).
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
        World::new("t".into(), 200, 200)
    }

    /// The wall context: band 1000..1600, full hp 8000, left edge at
    /// x 1000 with width 100 (the anchor X is 1050), moving right.
    fn ctx() -> Style29Context {
        Style29Context {
            wof_present: true,
            wof_life: 8000,
            wof_life_max: 8000,
            wof_x: 1000.0,
            wof_width: 100,
            wof_vx: 1.5,
            wof_center_x: 1050.0,
            wof_draw_top: 1000,
            wof_draw_bottom: 1600,
            expert: false,
            just_hit: false,
            who_am_i: 0,
            ticker: 0,
        }
    }

    /// A Hungry at the band's mid row (`ai[0] = 0.5` → y2 = 1300).
    fn hungry(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Hungry, x, y, 1, 0.0);
        n.ai[0] = 0.5;
        n
    }

    /// `NPC.cs:26215-26219` — without a wall the Hungry deactivates
    /// (`active = false` → the port's hp-0 convention) and nothing else
    /// runs; `:26211-26214` — the justHit write lands BEFORE the gate.
    #[test]
    fn the_hungry_deactivates_without_a_wall_but_just_hit_still_writes() {
        let w = air();
        let mut n = hungry(1050.0, 1300.0);
        let mut c = ctx();
        c.wof_present = false;
        c.just_hit = true;
        style29_step(&mut n, &w, &c, &[cand(1340.0, 1279.0)]);
        assert_eq!(n.hp, 0, "the active=false stand-in");
        assert_eq!(n.ai[1], 10.0, "justHit precedes the gate (:26211)");
        assert_eq!(n.ai[2], 0.0, "the arm returned before the dive cycle");
        assert_eq!(n.damage_override, None, "no tier ran");
    }

    /// `NPC.cs:26280-26283` — the anchor X carries the wall's INTEGER
    /// half-width: left edge 1000 + width/2 (100/2 = 50) = 1050. A
    /// Hungry parked exactly on the anchor with the player on the same
    /// row gets NO acceleration (both approach comparisons are strict).
    #[test]
    fn the_anchor_x_adds_the_walls_integer_half_width() {
        let w = air();
        let mut c = ctx();
        c.wof_width = 101; // odd width exposes the int division: 101/2 = 50
        // Player centre on the anchor row: x + 20/2 = 1050 → x = 1040;
        // y + 42/2 = 1300 → y = 1279. num391 = 1050 - 15 - 1050 = -15.
        // Park the Hungry at num389 + num391 = 1035 so X is exact; the
        // Y aim: 1300 - 15 - 1300 = -15 → park y at 1285.
        let mut n = hungry(1035.0, 1285.0);
        style29_step(&mut n, &w, &c, &[cand(1040.0, 1279.0)]);
        assert_eq!(n.vx, 0.0, "x == num389 + num391 → no accel (:26305/:26313)");
        assert_eq!(n.vy, 0.0, "y == y2 + num392 → no accel (:26321/:26329)");
        // The live arm's anchor (wof_x alone) would sit 50 px left and
        // accelerate here — the witness of the missing half-width.
    }

    /// `NPC.cs:26223-26254` — the classic tiers: full hp resets the
    /// damage to defDamage (override None) and writes NO defense; the
    /// <75% tier is 45 dmg / def 20 / leash 500; the <50% tier is
    /// 60 dmg / def 30 / leash 700. The ladder is f64 against raw ints.
    #[test]
    fn the_classic_tiers_reset_damage_and_keep_the_defense_write() {
        let w = air();
        let mut c = ctx();
        let players = [cand(2000.0, 1279.0)];
        // Full hp: damage reset, no defense write (a stale override
        // from a previous tier survives — the C# never clears it).
        let mut n = hungry(1050.0, 1300.0);
        n.defense_override = Some(20);
        n.damage_override = Some(45);
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.damage_override, None, "damage = defDamage (:26223)");
        assert_eq!(n.defense_override, Some(20), "no classic defense restore (:26225-26250)");
        // 74% hp: the <0.75 tier.
        c.wof_life = 5920; // 8000 * 0.74
        let mut n = hungry(1050.0, 1300.0);
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.damage_override, Some(45), "GetAttackDamage_ScaledByDifficulty(45) = 45");
        assert_eq!(n.defense_override, Some(20));
        // 49% hp: the <0.5 tier.
        c.wof_life = 3920;
        let mut n = hungry(1050.0, 1300.0);
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.damage_override, Some(60));
        assert_eq!(n.defense_override, Some(30));
        // The boundary is STRICT: exactly 50% falls to the <0.75 tier.
        c.wof_life = 4000;
        let mut n = hungry(1050.0, 1300.0);
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.damage_override, Some(45), "(double)4000 < 8000*0.5 is false");
    }

    /// `NPC.cs:26284-26292` + `:26299-26304` — the dive cycle: at tick
    /// 101 the leash 300 becomes `(int)(300 * 1.3f)` = 390; at tick 201
    /// `ai[2]` wraps to 0. The witness is behavioural: a player 350 px
    /// out is clamped at the base leash (the approach steers toward the
    /// clamped target, left of the Hungry) but chased freely once the
    /// dive leash grows to 390.
    #[test]
    fn the_dive_cycle_truncates_the_leash_and_wraps_at_200() {
        let w = air();
        let c = ctx();
        // Player at (1405, 1279): delta = (1405+10-15-1050, -15) =
        // (350, -15), length ≈ 350.32 > 300.
        let players = [cand(1405.0, 1279.0)];
        // Base leash: the Hungry sits BEYOND the clamped radius (x
        // 1390, 340 px right of the anchor): the clamped target ≈ 1349.7
        // is left of it, so the approach steers LEFT (vx -0.1) even
        // though the player is to the right — the leash is the aim.
        let mut n = hungry(1390.0, 1300.0);
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.vx, -0.1, "the 300-px leash clamps the 350-px aim");
        // Tick 101: the leash is (int)(300 * 1.3f) = 390 > 350.32 →
        // no clamp → the target is the player's row-aim at x 1400,
        // right of the Hungry → vx +0.1.
        let mut n2 = hungry(1390.0, 1300.0);
        n2.ai[2] = 100.0; // this tick increments to 101 → the ×1.3 arm
        style29_step(&mut n2, &w, &c, &players);
        assert_eq!(n2.ai[2], 101.0);
        assert_eq!(n2.vx, 0.1, "the 390-px dive leash frees the aim");
        // Tick 201 wraps the counter.
        let mut n3 = hungry(1050.0, 1300.0);
        n3.ai[2] = 200.0; // → 201 → wrap
        style29_step(&mut n3, &w, &c, &players);
        assert_eq!(n3.ai[2], 0.0, "ai[2] > 200 resets (:26288-26291)");
        // The truncation witness uses the C#'s FLOAT literal 1.3f
        // (`:26287`): f32 700 * 1.3f32 = 909.99994 -> (int) 909 (the
        // f64-literal form would round to 910 - not the C#'s value).
        assert_eq!((300.0f32 * 1.3f32) as i32, 390);
        assert_eq!((500.0f32 * 1.3f32) as i32, 650);
        assert_eq!((700.0f32 * 1.3f32) as i32, 909);
    }

    /// `NPC.cs:26299-26304` — beyond the leash the aim vector scales to
    /// exactly the leash length; inside it, it is untouched.
    #[test]
    fn the_leash_clamps_the_aim_vector_to_the_radius() {
        let w = air();
        let c = ctx();
        // Player at (2050, 1279): delta = (2050+10-15-1050, 1300-15-1300)
        // = (995, -15); length ≈ 995.11 > 300 → clamped.
        let players = [cand(2050.0, 1279.0)];
        let mut n = hungry(1050.0, 1300.0);
        style29_step(&mut n, &w, &c, &players);
        // The Hungry is at the anchor; the clamped target is
        // num389 + num391 ≈ 1050 + 299.97 → the X accel is +0.1.
        assert!(n.vx > 0.0, "the clamped aim still points right");
        assert_eq!(n.vx, 0.1, "one accel step toward the clamped target");
        // The leash witness: a player 350 px out is clamped to 300 —
        // a Hungry beyond the radius (x 1390) steers LEFT toward the
        // clamped target (the leash, not the player, is the aim).
        let mut n3 = hungry(1390.0, 1300.0);
        style29_step(&mut n3, &w, &c, &[cand(1405.0, 1279.0)]);
        assert_eq!(n3.vx, -0.1, "the 300-px leash clamps the 350-px aim");
        // Inside the leash: player at (1200, 1279) → delta (145, -15),
        // length ≈ 145.77 < 300 → no clamp; same +0.1 first step.
        let mut n2 = hungry(1050.0, 1300.0);
        style29_step(&mut n2, &w, &c, &[cand(1200.0, 1279.0)]);
        assert_eq!(n2.vx, 0.1);
    }

    /// `NPC.cs:26305-26336` — the sign-flip double-add: moving away
    /// from the target adds `num386 + num386 * 2.5f` = 0.35 in one tick.
    #[test]
    fn the_sign_flip_double_add_is_num386_times_2_point_5() {
        let w = air();
        let c = ctx();
        // The target is to the right (delta +145) but the Hungry moves
        // left: vx = -1 → +0.1 then +0.25 → -0.65.
        let mut n = hungry(1050.0, 1300.0);
        n.vx = -1.0;
        style29_step(&mut n, &w, &c, &[cand(1200.0, 1279.0)]);
        assert_eq!(n.vx, -0.65, "0.1 + 0.1*2.5 on the flip (:26307-26311)");
        // No flip when already moving toward the target: +0.1 only.
        let mut n2 = hungry(1050.0, 1300.0);
        n2.vx = 1.0;
        style29_step(&mut n2, &w, &c, &[cand(1200.0, 1279.0)]);
        assert_eq!(n2.vx, 1.1);
    }

    /// `NPC.cs:26368-26383` — the classic speed cap is 4 on both axes,
    /// applied as four one-sided clamps after the approach.
    #[test]
    fn the_classic_speed_cap_is_four() {
        let w = air();
        let c = ctx();
        let mut n = hungry(1050.0, 1300.0);
        n.vx = 10.0;
        n.vy = -10.0;
        style29_step(&mut n, &w, &c, &[cand(1200.0, 1279.0)]);
        assert_eq!(n.vx, 4.0);
        assert_eq!(n.vy, -4.0);
    }

    /// `NPC.cs:26297`/`:26385-26392` — the recoil chain: ai[1] == 0
    /// chases; ai[1] > 0 ticks down with NO movement; ai[1] < 0 snaps
    /// to 0 (the live arm misses this third arm).
    #[test]
    fn the_recoil_pause_ticks_down_and_a_negative_pause_snaps_to_zero() {
        let w = air();
        let c = ctx();
        let players = [cand(1200.0, 1279.0)];
        let mut n = hungry(1050.0, 1300.0);
        n.ai[1] = 3.0;
        n.vx = 2.0;
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.ai[1], 2.0);
        assert_eq!(n.vx, 2.0, "the pause branch does not steer (:26385-26388)");
        let mut n2 = hungry(1050.0, 1300.0);
        n2.ai[1] = -5.0;
        style29_step(&mut n2, &w, &c, &players);
        assert_eq!(n2.ai[1], 0.0, "the else arm resets (:26389-26392)");
        assert_eq!(n2.vx, 0.0, "the reset tick does not chase either");
    }

    /// `NPC.cs:26393-26402` — the aim ifs write spriteDirection and the
    /// rotation; the left arm adds the literal 3.14f; a dead-centre aim
    /// (num391 == 0) keeps the previous flip AND rotation.
    #[test]
    fn the_aim_flips_the_sprite_and_the_left_rotation_adds_3_point_14() {
        let w = air();
        let c = ctx();
        // Target right: delta (145, -15) → sprite 1, rotation atan2.
        let mut n = hungry(1050.0, 1300.0);
        style29_step(&mut n, &w, &c, &[cand(1200.0, 1279.0)]);
        assert_eq!(n.sprite_dir, 1);
        assert_eq!(n.rotation, (-15.0f64).atan2(145.0) as f32);
        // Target left: player at (900, 1279) -> delta (-155, -15)
        // (900 + 10 - 15 - 1050 = -155).
        let mut n2 = hungry(1050.0, 1300.0);
        style29_step(&mut n2, &w, &c, &[cand(900.0, 1279.0)]);
        assert_eq!(n2.sprite_dir, -1);
        assert_eq!(n2.rotation, (-15.0f64).atan2(-155.0) as f32 + 3.14);
        // Dead centre: player x + 10 - 15 - 1050 = 0 -> x = 1055; the
        // aim delta X is 0 -> neither if fires.
        let mut n3 = hungry(1050.0, 1300.0);
        n3.sprite_dir = -1;
        n3.rotation = 1.234;
        style29_step(&mut n3, &w, &c, &[cand(1055.0, 1279.0)]);
        assert_eq!(n3.sprite_dir, -1, "a dead-centre aim keeps the flip");
        assert_eq!(n3.rotation, 1.234, "and the rotation");
    }

    /// `NPC.cs:26220` — TargetClosest runs every tick: the target slot
    /// and the facing follow the nearest living player.
    #[test]
    fn the_arm_selects_its_target_through_target_closest() {
        let w = air();
        let c = ctx();
        let mut n = hungry(1050.0, 1300.0);
        n.target = 255; // the ClearTarget default
        // Slot 1 is nearer than slot 0.
        let players = [cand(3000.0, 1279.0), cand(1200.0, 1279.0)];
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.target, 1, "the Manhattan scan picks slot 1");
        assert_eq!(n.facing, 1, "the selection faces the target");
    }

    /// `NPC.cs:26255-26279` — the expert ladder: the defense restores
    /// to defDefense, the leash rides the whoAmI mods (both %4 and %3
    /// arms stack) and the final ×0.75. whoAmI 0: 300 ×1.75 ×1.5 ×0.75
    /// = 590.625. The accel bonus replaces the leash growth at <50%.
    #[test]
    fn the_expert_leash_ladder_stacks_the_mod_arms() {
        let w = air();
        let mut c = ctx();
        c.expert = true;
        c.wof_life = 3920; // <50%: num386 += 0.066, leash stays 300
        c.who_am_i = 0; // %4==0 (×1.75) and %3==0 (×1.5)
        let players = [cand(1200.0, 1279.0)];
        let mut n = hungry(1050.0, 1300.0);
        n.defense_override = Some(30);
        style29_step(&mut n, &w, &c, &players);
        assert_eq!(n.defense_override, None, "defense = defDefense (:26257)");
        // The accel witness: 0.1 + 0.066 = 0.166 per step.
        assert_eq!(n.vx, 0.1 + 0.066, "num386 += 0.066 (:26235)");
        // whoAmI 5: %4==1 (×1.5), %3==2 (no arm) → 300 ×1.5 ×0.75 =
        // 337.5. The behavioural witness: a 320-px aim (length ≈ 320.35)
        // is freed by the 337.5 leash, a 500-px aim (≈ 500.22) clamped.
        c.who_am_i = 5;
        let mut n2 = hungry(1050.0, 1300.0);
        style29_step(&mut n2, &w, &c, &[cand(1375.0, 1279.0)]);
        assert_eq!(n2.vx, 0.1 + 0.066, "the 337.5 leash frees the 320-px aim");
        let mut n4 = hungry(1450.0, 1300.0); // beyond the 337.5 radius
        style29_step(&mut n4, &w, &c, &[cand(1555.0, 1279.0)]);
        assert!(n4.vx < 0.0, "whoAmI 5: the 337.5 leash clamps the 500-px aim");
        // whoAmI 0's ladder is 300 ×1.75 ×1.5 ×0.75 = 590.625 — the
        // same 500-px aim is freed there: the mod arms' product witness.
        c.who_am_i = 0;
        let mut n5 = hungry(1050.0, 1300.0);
        style29_step(&mut n5, &w, &c, &[cand(1555.0, 1279.0)]);
        assert_eq!(n5.vx, 0.1 + 0.066, "whoAmI 0: the 590.625 leash frees it");
        // The expert speed cap (:26338-26367): `life / lifeMax` is INT
        // division — at 3920/8000 it is 0, so ALL four hp bands fire:
        // num395 = 1.5 + 0.7 + 0.7 + 0.9 + 0.9 = 4.7 → the cap is
        // 4 + (4.7*1.25+0.3)*0.35; the Hungry sits left of the wall's
        // centre with the wall moving right → the catch-up +6.
        let mut n3 = hungry(1050.0, 1300.0);
        n3.x = 900.0; // centre 915 < wof centre 1050, wof_vx > 0
        n3.vx = 20.0;
        style29_step(&mut n3, &w, &c, &players);
        let num395 = 1.5f32 + 0.7 + 0.7 + 0.9 + 0.9;
        let expected = 4.0 + (num395 * 1.25 + 0.3) * 0.35 + 6.0;
        assert_eq!(n3.vx, expected, "the expert cap + the catch-up +6");
        // The full-hp wall: 8000/8000 = 1 → no band fires → num395
        // stays 1.5 (the INT division's other extreme).
        c.wof_life = 8000;
        let mut n6 = hungry(900.0, 1300.0);
        n6.vx = 20.0;
        style29_step(&mut n6, &w, &c, &players);
        let expected_full = 4.0 + (1.5f32 * 1.25 + 0.3) * 0.35 + 6.0;
        assert_eq!(n6.vx, expected_full, "life/lifeMax == 1 keeps num395 at 1.5");
    }

    /// RNG by construction: the arm draws no `Main.rand` — two identical
    /// inputs produce identical state, tick after tick, with no RNG
    /// parameter in the signature.
    #[test]
    fn the_arm_is_deterministic_without_an_rng() {
        let w = air();
        let c = ctx();
        let players = [cand(1200.0, 1279.0)];
        let mut a = hungry(1050.0, 1300.0);
        let mut b = hungry(1050.0, 1300.0);
        for _ in 0..300 {
            style29_step(&mut a, &w, &c, &players);
            style29_step(&mut b, &w, &c, &players);
            // The caller owns the integration (noTileCollide).
            a.x += a.vx;
            a.y += a.vy;
            b.x += b.vx;
            b.y += b.vy;
            assert_eq!((a.x, a.y, a.vx, a.vy), (b.x, b.y, b.vx, b.vy));
            assert_eq!(a.ai, b.ai);
            assert_eq!(a.rotation, b.rotation);
        }
    }

    /// The SetDefaults row witness (`NPC.cs:10378-10392`): the port's
    /// static tables carry the Hungry's 30×30 box, aiStyle 29, def 10
    /// and 240 HP — the arm's integer half-extents read 15/15. (The
    /// base damage 30 rides the `pub(super)` `contact_damage` table,
    /// `npc/data.rs:2487` — not reachable from this module; the tier
    /// test above witnesses the override path the contact reads.)
    #[test]
    fn the_hungry_defaults_match_the_set_defaults_row() {
        let n = Npc::new(NpcKind::Hungry, 0.0, 0.0, 1, 0.0);
        assert_eq!(n.size(), (30.0, 30.0));
        assert_eq!(n.kind.ai_style(), 29);
        assert_eq!(n.max_hp, 240);
        assert_eq!(n.defense(), 10);
    }
}
