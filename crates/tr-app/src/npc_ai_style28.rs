//! The aiStyle-28 family arm — the Wall of Flesh Eye (`NPC.AI()`'s
//! `else if (this.aiStyle == 28)` block, `NPC.cs:26053-26208`).
//!
//! S1-D-3 (aiStyle-28 round, code-only mode): this module is the faithful
//! 1:1 port of the arm as pure functions over `Npc` + a small context
//! seam. It is NOT wired into `Npc::update_rng`'s style axis yet — the
//! live `npc.rs::update_wof_eye` keeps running until the wiring round
//! (the sibling D-3 rounds land the same way: module + tests first,
//! dispatch claim second). Every other file is byte-identical except the
//! one `mod` line in `main.rs` and the docs.
//!
//! The only type on this style is `NPCID.WallofFleshEye` (114,
//! `ID/NPCID.cs:10905`); its `SetDefaults` row (`NPC.cs:10361-10377`):
//! 100×100, aiStyle 28, damage 50, defense 0, lifeMax 8000,
//! `SoundID.NPCHit8`/`NPCDeath10`, noGravity, noTileCollide, behindTiles,
//! knockBackResist 0, scale 1.2, value 80000. The port's static tables
//! already carry it (`npc/data.rs`: `ai_style() == 28`, size (100, 100)).
//! There are no per-type branches inside the arm — the whole block below
//! is the family.
//!
//! RNG by construction: the C# arm (`NPC.cs:26053-26208`) draws NO
//! `Main.rand` — every `Next` in the WoF fight lives in the wall's own
//! aiStyle-27 arm and the Hungry's aiStyle-29 arm, not here. The pacing
//! is deterministic counters (`localAI[1]` the tick accumulator,
//! `localAI[2]` the burst index) and hp-band comparisons, so the port
//! takes no RNG parameter at all; the witness test
//! `the_arm_is_deterministic_without_an_rng` pins two identical inputs
//! to identical state.
//!
//! Faithfulness notes vs the live `update_wof_eye` (the wiring round
//! replaces it with this arm):
//! * `position.X = wof.position.X` is a LITERAL copy (`:26066`) — no
//!   `- width/2` centering (the live arm subtracts it).
//! * The band midpoint `(wofDrawAreaBottom + wofDrawAreaTop) / 2`
//!   (`:26069`) is INTEGER division — both statics are `int`
//!   (`Main.cs:757`/`:759`); the live arm uses f32 throughout.
//! * The hp ladder compares `(double)life < (double)lifeMax * k`
//!   (`:26127` etc.) in f64, not a precomputed f32 ratio.
//! * The pacing counters are `localAI[1]`/`localAI[2]` (`:26126`+), not
//!   `ai[1]`/`ai[2]` (the live arm rides the wrong array).
//! * The `CanHit` sight gate (`:26167`) returns BEFORE the counters
//!   reset — a blocked tick keeps its accumulated `localAI[1]`.
//! * The aim rotation multiplies the delta by the distance (`:26097-
//!   26098`, `num377 *= num379` — a C# quirk; the angle is unchanged for
//!   a positive scale, but the operation order is kept verbatim).
//! * The shot spawns at centre + one velocity step (`:26203-26205`,
//!   `vector34.X += num377` before `NewProjectile`), and the shot aim
//!   uses the FLOAT player half-width (`:26198`, `width * 0.5f`) while
//!   the rotation aim uses the INTEGER half (`:26093`, `(width / 2)`).
//! * The arm does NOT integrate `position += velocity` — the eye is
//!   `noTileCollide`, so `UpdateNPC` integrates after `AI()`
//!   (`NPC.cs:90498`); the caller owns the step.
//! * `this.active = false` (`:26057`) has no port field — the port's
//!   death convention `hp = 0` stands in (same as the live arm).
//! * `realLife = Main.wofNPCIndex` (`:26060`) — the port's carrier
//!   machinery is worm-keyed (`Npc::carrier_index`); the eye's life
//!   pool rides the `wof_life` mirror below until a WoF carrier lands.
//! * `Main.netMode == 1` early return (`:26121-26124`) rides
//!   `Style28Context::net_client`; `Main.expertMode` (`:26147`) rides
//!   `Style28Context::expert` (the port is single-difficulty today).
#![allow(dead_code)] // wiring deferred (code-only mode) — the dispatch claim is the follow-up round

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The WoF state the arm reads through `Main`'s statics + the wall NPC.
/// The caller (the future style-axis arm) fills it from the sim's
/// `World.wof_*` write-through (`world_sim/npcs.rs:3646-3651`) and the
/// wall's own slot.
#[derive(Clone, Copy, Debug)]
pub struct Style28Context {
    /// `Main.wofNPCIndex >= 0` (`NPC.cs:26055`, `Main.cs:755`).
    pub wof_present: bool,
    /// `Main.npc[Main.wofNPCIndex].life` (`:26061-26064`, `:26127`+).
    pub wof_life: i32,
    /// `Main.npc[Main.wofNPCIndex].lifeMax` (the ladder's f64 base).
    pub wof_life_max: i32,
    /// `Main.npc[Main.wofNPCIndex].position.X` (`:26066`).
    pub wof_x: f32,
    /// `Main.npc[Main.wofNPCIndex].direction` (`:26067`).
    pub wof_dir: i32,
    /// `Main.wofDrawAreaTop` (`Main.cs:757`) — an INT in the C#.
    pub wof_draw_top: i32,
    /// `Main.wofDrawAreaBottom` (`Main.cs:759`) — an INT in the C#.
    pub wof_draw_bottom: i32,
    /// `Main.expertMode` (`:26147-26156`).
    pub expert: bool,
    /// `Main.netMode == 1` (`:26121`) — the client skips the volley.
    pub net_client: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp — not
    /// plumbed into `Npc::update` yet (same stand-in as `update_zombie`).
    pub ticker: u64,
}

/// `NPC.AI()`'s aiStyle-28 arm (`NPC.cs:26053-26208`), one tick.
/// `players` is the target-scan slice (at least one slot — the C# always
/// has the 255 `Main.player` slots; the port's session player is slot 0).
/// The shot request rides `Npc::wants_hostile_shot` (the existing seam,
/// `(type, x, y, vx, vy, damage)`); the C#'s knockback `0f` and
/// `Main.myPlayer` owner are the tuple's documented non-carried fields.
pub fn style28_step(npc: &mut Npc, world: &World, ctx: &Style28Context, players: &[TargetCandidate]) {
    // `:26055-26059` — no wall: deactivate and return.
    if !ctx.wof_present {
        npc.hp = 0; // `this.active = false` — the port's death convention.
        return;
    }
    // `:26060` — `realLife = Main.wofNPCIndex` (the carrier note above).
    // `:26061-26064` — the life mirror, only while the wall lives.
    if ctx.wof_life > 0 {
        npc.hp = ctx.wof_life;
    }
    // `:26065` — `TargetClosest()` runs BEFORE the X/direction mirror,
    // so its facing write is overwritten by the wall's direction below.
    target_closest(npc, world, players, true, 28, ctx.ticker);
    let (w, h) = npc.size();
    // `:26066-26068` — the literal X copy + the direction/sprite mirror.
    npc.x = ctx.wof_x;
    npc.facing = ctx.wof_dir;
    npc.sprite_dir = npc.facing;
    // `:26069` — `(wofDrawAreaBottom + wofDrawAreaTop) / 2`: INTEGER
    // division (both statics are `int`), then widened to float.
    let mut num376 = ((ctx.wof_draw_bottom + ctx.wof_draw_top) / 2) as f32;
    // `:26070` — the half-band pick; `- height/2` sits OUTSIDE the `?:`
    // and `base.height / 2` is integer division.
    num376 = if !(npc.ai[0] > 0.0) {
        (num376 + ctx.wof_draw_bottom as f32) / 2.0
    } else {
        (num376 + ctx.wof_draw_top as f32) / 2.0
    } - (h as i32 / 2) as f32;
    // `:26071-26083` — the ±1 band: outside it the velocity is ∓1,
    // inside it zeroes and the position snaps to the target.
    if npc.y > num376 + 1.0 {
        npc.vy = -1.0;
    } else if npc.y < num376 - 1.0 {
        npc.vy = 1.0;
    } else {
        npc.vy = 0.0;
        npc.y = num376;
    }
    // `:26084-26091` — the ±5 clamp (two one-sided ifs in the C#).
    if npc.vy > 5.0 {
        npc.vy = 5.0;
    }
    if npc.vy < -5.0 {
        npc.vy = -5.0;
    }
    // The aim (`:26092-26120`). The rotation aim reads the player's
    // INTEGER half-extents (`:26093-26094`, `(float)(width / 2)`).
    let p = &players[npc.target as usize];
    let mut vector34 = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut num377 = p.x + (p.width / 2) as f32 - vector34.0;
    let mut num378 = p.y + (p.height / 2) as f32 - vector34.1;
    let num379 = (num377 * num377 + num378 * num378).sqrt();
    // `:26096` — `num380 = num379` is a dead store in the C# (never read).
    // `:26097-26098` — the delta is MULTIPLIED by the distance (the C#
    // quirk; kept verbatim — a positive scale leaves atan2's angle).
    num377 *= num379;
    num378 *= num379;
    let mut flag42 = true;
    let npc_mid_x = npc.x + (w as i32 / 2) as f32;
    let p_aim_x = p.x + (p.width / 2) as f32;
    if npc.facing > 0 {
        // `:26100-26110`.
        if p_aim_x > npc_mid_x {
            npc.rotation = (0.0 - num378).atan2(0.0 - num377) + 3.14;
        } else {
            npc.rotation = 0.0;
            flag42 = false;
        }
    } else if p_aim_x < npc_mid_x {
        // `:26112-26115` — the `else` covers direction <= 0.
        npc.rotation = num378.atan2(num377) + 3.14;
    } else {
        // `:26116-26120`.
        npc.rotation = 0.0;
        flag42 = false;
    }
    // `:26121-26124` — the client stops here (the server volleys).
    if ctx.net_client {
        return;
    }
    // The volley pacing (`:26125-26156`). `num381` is the burst length;
    // the ladder compares in f64 exactly like the C#'s `(double)` casts.
    let below = |k: f64| (ctx.wof_life as f64) < (ctx.wof_life_max as f64) * k;
    let mut num381 = 4i32;
    npc.local_ai[1] += 1.0;
    if below(0.75) {
        npc.local_ai[1] += 1.0;
        num381 += 1;
    }
    if below(0.5) {
        npc.local_ai[1] += 1.0;
        num381 += 1;
    }
    if below(0.25) {
        npc.local_ai[1] += 1.0;
        num381 += 2;
    }
    if below(0.1) {
        npc.local_ai[1] += 2.0;
        num381 += 3;
    }
    if ctx.expert {
        // `:26147-26156`.
        npc.local_ai[1] += 0.5;
        num381 += 1;
        if below(0.1) {
            npc.local_ai[1] += 2.0;
            num381 += 3;
        }
    }
    if npc.local_ai[2] == 0.0 {
        // `:26157-26164` — the idle 600-tick gate opens the burst.
        if npc.local_ai[1] > 600.0 {
            npc.local_ai[2] = 1.0;
            npc.local_ai[1] = 0.0;
        }
    } else {
        // `:26165-26170` — the 45-tick shot gate AND the sight line; a
        // blocked tick returns BEFORE any counter reset.
        if !(npc.local_ai[1] > 45.0)
            || !crate::projectile::can_hit(
                world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
            )
        {
            return;
        }
        // `:26171-26176` — the reset, the burst index, the wrap.
        npc.local_ai[1] = 0.0;
        npc.local_ai[2] += 1.0;
        if npc.local_ai[2] >= num381 as f32 {
            npc.local_ai[2] = 0.0;
        }
        if flag42 {
            // `:26177-26206` — the shot: speed 9 / damage 11 / type 83
            // with the hp ladder (`Projectile.cs:1632-1646`: the 83 is
            // 4×4, aiStyle 1, hostile, penetrate 3, extraUpdates 2,
            // scale 1.7, timeLeft 600 — the port's `EYE_LASER` row).
            let mut num382 = 9.0f32;
            let mut num383 = 11i32;
            let num384 = crate::projectile::EYE_LASER; // 83
            if below(0.5) {
                num383 += 1;
                num382 += 1.0;
            }
            if below(0.25) {
                num383 += 1;
                num382 += 1.0;
            }
            if below(0.1) {
                num383 += 2;
                num382 += 2.0;
            }
            // `:26197-26202` — the shot aim re-reads the centre and uses
            // the FLOAT player half-extents this time (`width * 0.5f`);
            // the normaliser DIVIDES by the distance (no epsilon floor —
            // a zero distance yields the C#'s NaN/Inf verbatim).
            vector34 = (npc.x + w * 0.5, npc.y + h * 0.5);
            num377 = p.x + p.width as f32 * 0.5 - vector34.0;
            num378 = p.y + p.height as f32 * 0.5 - vector34.1;
            let scale = num382 / (num377 * num377 + num378 * num378).sqrt();
            num377 *= scale;
            num378 *= scale;
            // `:26203-26205` — the spawn point steps one velocity ahead
            // of the centre before `NewProjectile`.
            vector34.0 += num377;
            vector34.1 += num378;
            npc.wants_hostile_shot
                .push((num384, vector34.0, vector34.1, num377, num378, num383));
        }
    }
    // No `position += velocity` here: the eye is noTileCollide, so
    // `UpdateNPC` integrates after `AI()` (`NPC.cs:90498`).
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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

    /// The wall context: band 1000..1600, full hp 8000, at x 1000,
    /// facing right. `ai[0] = 1` (the upper eye) puts the band target at
    /// ((1000+1600)/2 + 1000)/2 - 100/2 = 1100.
    fn ctx() -> Style28Context {
        Style28Context {
            wof_present: true,
            wof_life: 8000,
            wof_life_max: 8000,
            wof_x: 1000.0,
            wof_dir: 1,
            wof_draw_top: 1000,
            wof_draw_bottom: 1600,
            expert: false,
            net_client: false,
            ticker: 0,
        }
    }

    fn eye(y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::WallOfFleshEye, 1000.0, y, 1, 0.0);
        n.ai[0] = 1.0; // the upper half-band
        n
    }

    /// `NPC.cs:26055-26059` — without a wall the eye deactivates
    /// (`active = false` → the port's hp-0 convention) and nothing else
    /// runs.
    #[test]
    fn the_eye_deactivates_without_a_wall() {
        let w = air();
        let mut n = eye(1100.0);
        let mut c = ctx();
        c.wof_present = false;
        style28_step(&mut n, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n.hp, 0, "the active=false stand-in");
        assert_eq!(n.local_ai[1], 0.0, "the arm returned before the pacing");
        assert_eq!(n.x, 1000.0, "no mirror ran");
    }

    /// `NPC.cs:26061-26068` — the life mirror (only while the wall
    /// lives), the LITERAL X copy (no `- width/2` centering) and the
    /// direction/spriteDirection mirror.
    #[test]
    fn the_life_and_x_and_direction_mirrors_are_literal() {
        let w = air();
        let mut n = eye(1100.0);
        let mut c = ctx();
        c.wof_life = 5000;
        c.wof_x = 1234.5;
        c.wof_dir = -1;
        style28_step(&mut n, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n.hp, 5000, "life = wof.life (:26063)");
        assert_eq!(n.x, 1234.5, "position.X = wof.position.X, verbatim (:26066)");
        assert_eq!(n.facing, -1);
        assert_eq!(n.sprite_dir, -1, "spriteDirection = direction (:26068)");
        // A dead wall's life does not overwrite the mirror.
        let mut n2 = eye(1100.0);
        n2.hp = 777;
        c.wof_life = 0;
        style28_step(&mut n2, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n2.hp, 777, "the life > 0 gate (:26061)");
    }

    /// `NPC.cs:26069-26083` — the band midpoint is INTEGER division:
    /// top 1000 + bottom 1601 = 2601 → 1300 (not 1300.5), so the upper
    /// target is (1300 + 1000)/2 - 50 = 1100 exactly. A y of 1100.25
    /// sits inside the ±1 band and SNAPS to 1100; the f32-midpoint
    /// variant would have kept 1100.25.
    #[test]
    fn the_band_midpoint_is_integer_division_and_snaps() {
        let w = air();
        let mut c = ctx();
        c.wof_draw_bottom = 1601; // the odd sum exposes the int division
        let mut n = eye(1100.25);
        style28_step(&mut n, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n.y, 1100.0, "the snap to the int-divided target");
        assert_eq!(n.vy, 0.0);
        // Outside the band: ±1 toward it, no snap.
        let mut n2 = eye(1200.0);
        style28_step(&mut n2, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n2.vy, -1.0, "above the target → rise (:26071-26074)");
        assert_eq!(n2.y, 1200.0, "the arm does not integrate (:90498 owns it)");
        let mut n3 = eye(1000.0);
        style28_step(&mut n3, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n3.vy, 1.0, "below the target → sink (:26075-26078)");
        // The lower eye (ai[0] <= 0) picks the bottom half:
        // (1300 + 1601)/2 - 50 = 1400.5.
        let mut n4 = eye(1400.5);
        n4.ai[0] = 0.0;
        style28_step(&mut n4, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n4.y, 1400.5);
        assert_eq!(n4.vy, 0.0);
    }

    /// `NPC.cs:26100-26120` — the rotation: facing right with the player
    /// to the right, `atan2(-dy·d, -dx·d) + 3.14f`. The fixture centres
    /// are 300 right / 400 up apart (dx 300, dy −400, d 500), so the
    /// angle is atan2(4, −3) + 3.14 ≈ 5.354297. With the player BEHIND
    /// the facing the rotation zeroes and `flag42` kills the shot.
    #[test]
    fn the_rotation_aims_and_the_behind_target_never_shoots() {
        let w = air();
        let c = ctx();
        // The eye snaps to y 1100 → centre (1050, 1150). The player's
        // INTEGER-half aim centre: (1340+10, 729+21) = (1350, 750).
        let mut n = eye(1100.0);
        n.local_ai[1] = 46.0;
        n.local_ai[2] = 1.0;
        style28_step(&mut n, &w, &c, &[cand(1340.0, 729.0)]);
        let expected = 4.0f32.atan2(-3.0) + 3.14;
        assert!(
            (n.rotation - expected).abs() < 1e-3,
            "atan2(-dy·d, -dx·d) + 3.14: {} vs {expected}",
            n.rotation
        );
        assert_eq!(n.wants_hostile_shot.len(), 1, "flag42 true → the shot");
        // The player behind the facing (left of the eye): rotation 0,
        // the counters still advance, but no shot.
        let mut n2 = eye(1100.0);
        n2.local_ai[1] = 46.0;
        n2.local_ai[2] = 1.0;
        style28_step(&mut n2, &w, &c, &[cand(500.0, 729.0)]);
        assert_eq!(n2.rotation, 0.0, "the behind arm (:26106-26110)");
        assert_eq!(n2.local_ai[2], 2.0, "the burst index advanced");
        assert!(n2.wants_hostile_shot.is_empty(), "flag42 false → no shot");
        // Facing left with the player left: atan2(dy·d, dx·d) + 3.14.
        let mut c3 = ctx();
        c3.wof_dir = -1;
        let mut n3 = eye(1100.0);
        style28_step(&mut n3, &w, &c3, &[cand(500.0, 729.0)]);
        // dx = 510-1050 = -540, dy = 750-1150 = -400 → the angle is
        // atan2(-400, -540) + 3.14 (the distance scale cancels).
        let expected3 = (-400.0f32).atan2(-540.0) + 3.14;
        assert!((n3.rotation - expected3).abs() < 1e-3, "{}", n3.rotation);
    }

    /// `NPC.cs:26125-26176` — the pacing: the idle 600 gate, the 45-tick
    /// shot spacing and the `num381` burst wrap (4 at full hp).
    #[test]
    fn the_idle_600_gate_and_the_four_shot_burst_wrap() {
        let w = air();
        let c = ctx();
        let p = [cand(1340.0, 729.0)];
        let mut n = eye(1100.0);
        // Idle: 600 accumulated → the ++ crosses 600 and opens the burst.
        n.local_ai[1] = 600.0;
        style28_step(&mut n, &w, &c, &p);
        assert_eq!(n.local_ai[2], 1.0, "the burst opens (:26159-26163)");
        assert_eq!(n.local_ai[1], 0.0);
        assert!(n.wants_hostile_shot.is_empty(), "no shot on the open tick");
        // 45 is not > 45: the gate holds.
        n.local_ai[1] = 44.0;
        style28_step(&mut n, &w, &c, &p);
        assert_eq!(n.local_ai[1], 45.0, "45 is not > 45 (:26167)");
        assert!(n.wants_hostile_shot.is_empty());
        // Three more shots wrap the index 2 → 3 → 4 → 0 (num381 = 4).
        for want in [2.0, 3.0, 0.0] {
            n.local_ai[1] = 46.0;
            style28_step(&mut n, &w, &c, &p);
            assert_eq!(n.local_ai[2], want, "the burst index (:26172-26176)");
        }
        assert_eq!(n.wants_hostile_shot.len(), 3);
        // Wrapped to idle: a 46-tick accumulator does NOT fire.
        n.local_ai[1] = 46.0;
        style28_step(&mut n, &w, &c, &p);
        assert_eq!(n.wants_hostile_shot.len(), 3, "the idle gate closed");
        assert_eq!(n.local_ai[1], 47.0);
    }

    /// `NPC.cs:26127-26156` — the ladder compares `(double)life <
    /// (double)lifeMax * k`: at exactly 75 % (6000/8000) the first tier
    /// does NOT fire; one hp below it does. The expert adds 0.5/tick and
    /// a burst slot.
    #[test]
    fn the_hp_ladder_is_a_strict_double_comparison() {
        let w = air();
        let p = [cand(1340.0, 729.0)];
        let mut c = ctx();
        c.wof_life = 6000; // exactly 0.75 — the `<` keeps the tier off
        let mut n = eye(1100.0);
        style28_step(&mut n, &w, &c, &p);
        assert_eq!(n.local_ai[1], 1.0, "6000 < 6000 is false (:26127)");
        c.wof_life = 5999;
        let mut n2 = eye(1100.0);
        style28_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.local_ai[1], 2.0, "the <0.75 tier fires");
        // Full hp + expert: 1 + 0.5 per tick.
        let mut c3 = ctx();
        c3.expert = true;
        let mut n3 = eye(1100.0);
        style28_step(&mut n3, &w, &c3, &p);
        assert_eq!(n3.local_ai[1], 1.5, "the expert 0.5 (:26149)");
        // The <0.1 tiers: classic +2, expert +2 more (:26144, :26153).
        let mut c4 = ctx();
        c4.wof_life = 799; // < 800 = 0.1 * 8000
        let mut n4 = eye(1100.0);
        style28_step(&mut n4, &w, &c4, &p);
        // 1 + 1 + 1 + 1 + 2 = 6 (all four tiers).
        assert_eq!(n4.local_ai[1], 6.0);
        c4.expert = true;
        let mut n5 = eye(1100.0);
        style28_step(&mut n5, &w, &c4, &p);
        // 6 + 0.5 + 2 = 8.5.
        assert_eq!(n5.local_ai[1], 8.5);
    }

    /// `NPC.cs:26167` — the sight gate returns BEFORE the counters
    /// reset: a solid column between the eye and the player keeps the
    /// accumulated `localAI[1]` and the burst index.
    #[test]
    fn the_sight_gate_returns_before_the_counter_reset() {
        let mut w = air();
        // A full-height solid column at tile x 80 (px 1280) — between
        // the eye (centre tile 65) and the player (tile ~84).
        for y in 0..200 {
            let i = w.index(80, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let c = ctx();
        let mut n = eye(1100.0);
        n.local_ai[1] = 46.0;
        n.local_ai[2] = 1.0;
        style28_step(&mut n, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n.local_ai[1], 47.0, "the ++ ran, the reset did not");
        assert_eq!(n.local_ai[2], 1.0, "the burst index held");
        assert!(n.wants_hostile_shot.is_empty(), "no shot through the wall");
    }

    /// `NPC.cs:26179-26205` — the shot: type 83, the 9-speed / 11-damage
    /// base with the <0.5/<0.25/<0.1 ladder, the FLOAT-half aim and the
    /// spawn one velocity step ahead of the centre.
    #[test]
    fn the_shot_ladders_and_spawns_one_step_ahead() {
        let w = air();
        let p = [cand(1340.0, 729.0)];
        let mut c = ctx();
        // Full hp: 9 speed, 11 damage.
        let mut n = eye(1100.0);
        n.local_ai[1] = 46.0;
        n.local_ai[2] = 1.0;
        style28_step(&mut n, &w, &c, &p);
        let shot = n.wants_hostile_shot.last().copied().expect("the volley");
        assert_eq!(shot.0, 83, "the Eye Laser (:26181)");
        assert_eq!(shot.5, 11, "the base damage (:26180)");
        let sp = (shot.3 * shot.3 + shot.4 * shot.4).sqrt();
        assert!((sp - 9.0).abs() < 1e-3, "the 9 speed ({sp})");
        // The spawn = centre + velocity, exactly (:26203-26205). The
        // shot aim uses FLOAT halves: player centre (1350, 750).
        assert_eq!(shot.1, 1050.0 + shot.3, "centre.X + vx");
        assert_eq!(shot.2, 1150.0 + shot.4, "centre.Y + vy");
        // <0.5 (3999/8000): speed 10, damage 12.
        c.wof_life = 3999;
        let mut n2 = eye(1100.0);
        n2.local_ai[1] = 46.0;
        n2.local_ai[2] = 1.0;
        style28_step(&mut n2, &w, &c, &p);
        let shot2 = n2.wants_hostile_shot.last().copied().expect("the volley");
        assert_eq!(shot2.5, 12, "the <0.5 tier (:26182-26186)");
        let sp2 = (shot2.3 * shot2.3 + shot2.4 * shot2.4).sqrt();
        assert!((sp2 - 10.0).abs() < 1e-3, "the 10 speed ({sp2})");
        // <0.1 (799/8000): speed 13, damage 15 (11+1+1+2).
        c.wof_life = 799;
        let mut n3 = eye(1100.0);
        n3.local_ai[1] = 46.0;
        n3.local_ai[2] = 1.0;
        style28_step(&mut n3, &w, &c, &p);
        let shot3 = n3.wants_hostile_shot.last().copied().expect("the volley");
        assert_eq!(shot3.5, 15, "the <0.1 tier (:26192-26196)");
        let sp3 = (shot3.3 * shot3.3 + shot3.4 * shot3.4).sqrt();
        assert!((sp3 - 13.0).abs() < 1e-3, "the 13 speed ({sp3})");
    }

    /// `NPC.cs:26121-26124` — the client (`Main.netMode == 1`) mirrors
    /// and aims but never runs the volley pacing.
    #[test]
    fn the_client_returns_before_the_volley() {
        let w = air();
        let mut c = ctx();
        c.net_client = true;
        let mut n = eye(1100.0);
        n.local_ai[1] = 46.0;
        n.local_ai[2] = 1.0;
        style28_step(&mut n, &w, &c, &[cand(1340.0, 729.0)]);
        assert_eq!(n.local_ai[1], 46.0, "no pacing on the client");
        assert_eq!(n.local_ai[2], 1.0);
        assert!(n.wants_hostile_shot.is_empty());
        assert_ne!(n.rotation, 0.0, "the aim still ran");
    }

    /// RNG by construction: the C# arm (`NPC.cs:26053-26208`) contains
    /// no `Main.rand.Next` — the port takes no RNG parameter, so two
    /// identical inputs must produce identical state, shot included.
    /// This is the witness that the arm cannot perturb any RNG stream.
    #[test]
    fn the_arm_is_deterministic_without_an_rng() {
        let w = air();
        let c = ctx();
        let p = [cand(1340.0, 729.0)];
        let mut a = eye(1100.0);
        let mut b = eye(1100.0);
        a.local_ai[1] = 46.0;
        a.local_ai[2] = 1.0;
        b.local_ai[1] = 46.0;
        b.local_ai[2] = 1.0;
        style28_step(&mut a, &w, &c, &p);
        style28_step(&mut b, &w, &c, &p);
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.vy, b.vy);
        assert_eq!(a.hp, b.hp);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(a.local_ai, b.local_ai);
        assert_eq!(a.wants_hostile_shot, b.wants_hostile_shot);
    }
}
