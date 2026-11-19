//! The aiStyle-50 family — the spore pair (`NPC.AI()`'s
//! `else if (this.aiStyle == 50)` arm, `NPC.cs:31651-31716`). S1-D-3,
//! aiStyle-50 round, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! The family has TWO vanilla members: the FungiSpore 261
//! (`NPCID.FungiSpore`, `ID/NPCID.cs:11199`, `SetDefaults` row
//! `NPC.cs:12364-12378`) and the Spore 265 (`NPCID.Spore`,
//! `ID/NPCID.cs:11207`, row `NPC.cs:12423-12437`). Both are 1-HP
//! contact hazards: noGravity + noTileCollide, knockBackResist 0,
//! npcSlots 0, NPCHit1/NPCDeath1. (The *projectile* aiStyle 50 — the
//! spore cloud's sibling in `Projectile.cs` — is a different system and
//! is NOT touched here.)
//!
//! The machine: a slow guided fall. `EncourageDespawn(5)` runs FIRST
//! (`:31653`). The FungiSpore alone flips `noTileCollide = false` and
//! dies on any tile contact — `life = 0`, `HitEffect(0, 100.0)`,
//! `checkDead()`, `return` (`:31654-31664`): the velocity/target tail is
//! SKIPPED on the death tick. The Spore keeps `noTileCollide = true`
//! (`:31667`). Then: `vy += 0.02` (`:31669`), `TargetClosest()`
//! (`:31670`), the rising-but-player-below damp `vy *= 0.95`
//! (`:31671-31674`, strict `player.Y > npc.Y + 100`), the `vy > 1` clamp
//! (`:31675-31678`), the X steering with the expert double-damp/double-
//! accel (`:31679-31710`, strict edge comparisons, no equality case),
//! the `|vx| > 5` overspeed damp `*= 0.97` (`:31711-31714`) and
//! `rotation = vx * 0.2` (`:31715`).
//!
//! ## RNG order (by construction)
//!
//! The AI arm draws NO `Main.rand` — `NPC.cs:31651-31716` contains no
//! `Next`/`NextFloat`; neither does `TargetClosest`/`TryTrackingTarget`/
//! `SetTargetTrackingValues` (`NPC.cs:77652-77781`, the `npc_target.rs`
//! port) nor `EncourageDespawn` (`NPC.cs:7249-7256`). So [`ai_style50`]
//! takes NO RNG parameter; the witness test
//! `the_style50_arm_draws_no_rng` pins a `UnifiedRandom` state across
//! the calls.
//!
//! The draws that DO exist around the arm, in C# update order:
//! 1. `UpdateNPC_CastLights` (`NPC.cs:90320`, BEFORE `AI()` at
//!    `:90423`): the 254..=261 light arm (`:92384-92396`) draws ONE
//!    `Main.rand.Next(28, 42)` (`:92386`) per tick — for the FungiSpore
//!    ONLY. The Spore 265 is outside the range and lights NOTHING.
//!    Ported as [`fungi_spore_light`], which takes the RNG and draws
//!    exactly that one `Next(28, 42)`.
//! 2. `HitEffect` on death/hit: the Spore's 15-dust loop draws
//!    `Main.rand.Next(3)` per dust (`NPC.cs:87226-87236`); the
//!    FungiSpore's dust loops (`:89055-89070`) draw only inside
//!    `Dust.NewDust` itself. The dust system is unported (S1-E-2/S1-E-3,
//!    other branches), so BOTH arms are skipped wholesale — the
//!    `npc_lights.rs` precedent: skipping an arm whole leaves the
//!    `Main.rand` stream untouched. [`spore_hit_dust_plan`] ports the
//!    branch structure and counts only (no RNG parameter, no draws);
//!    the wiring wave must draw the `Next(3)` selectors and the
//!    `Dust.NewDust` internals in C# order when the dust pass lands.
//!
//! Wiring deferred (the BlazingWheel / `npc_ai_style21.rs` precedent,
//! commit 901452bd): the roster has no `NpcKind::FungiSpore`/`Spore`
//! yet, so `update_rng`'s aiStyle axis (`npc.rs:3255`) has no style-50
//! arm to claim. This module is the pure port + the minimal seams:
//! * `Npc` has no `no_tile_collide` field — the arm's per-tick write is
//!   RETURNED in [`Style50Outcome`] for the wiring caller to feed its
//!   movement step (`NPC.cs:90490-90498`: false → `UpdateCollision`,
//!   true → raw `position += velocity`).
//! * The FungiSpore's contact death sets `hp = 0` here (the C# `life =
//!   0`, `:31659`) and returns `died_on_contact`; the caller then runs
//!   the `HitEffect(0, 100.0)` + `checkDead()` pair (`:31660-31661`).
//!   `checkDead` (`NPC.cs:78040-78149`) has no 261/265-specific branch;
//!   the port's death resolution is the `world_sim/npcs.rs` retain.
//! * `FindFrame` (`NPC.cs:66684-77370`) has NO case 261/265 — the
//!   switch falls through and the post-switch tail is aiStyle-39 only
//!   (`:77292`), so the frame NEVER advances: both kinds are
//!   single-frame (`Main.npcFrameCount[261] = [265] = 1`, the 260-269
//!   row `2, 1, 8, 4, 4, 1, 8, 1, 4, 15` at `Main.cs:67110` of the
//!   `:67082` array). No frame helper is needed; the consts pin it.
//! * `EncourageDespawn` also sets `despawnEncouraged = true`
//!   (`NPC.cs:7255`); the port's `Npc::encourage_despawn`
//!   (`npc.rs:1643`) carries the timeLeft half only — the flag has no
//!   field, the established port seam.
//!
//! See docs/ANALYSIS.md "The spore pair (S1-D-3 aiStyle-50, code-only)".
#![allow(dead_code)] // wiring deferred — the kinds + dispatch arm land with the roster wave

use glam::Vec2;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// The spore `SetDefaults` rows (`NPC.cs:12364-12378` / `:12423-12437`),
/// verbatim. The two rows differ ONLY in the box (12×12 vs 18×18) and
/// the damage (80 vs 70); every other field is identical.
pub struct SporeDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1` (`ID/SoundID.cs:1257`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1` (`ID/SoundID.cs:1315`).
    pub death_sound: (i32, i32),
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub knock_back_resist: f32,
    pub npc_slots: f32,
}

/// `NPC.cs:12364-12378` — `else if (this.type == 261)`, the FungiSpore.
pub const FUNGI_SPORE: SporeDefaults = SporeDefaults {
    width: 12,
    height: 12,
    ai_style: 50,
    damage: 80,
    defense: 0,
    life_max: 1,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    no_gravity: true,
    no_tile_collide: true,
    knock_back_resist: 0.0,
    npc_slots: 0.0,
};

/// `NPC.cs:12423-12437` — `else if (this.type == 265)`, the Spore.
pub const SPORE: SporeDefaults = SporeDefaults {
    width: 18,
    height: 18,
    ai_style: 50,
    damage: 70,
    defense: 0,
    life_max: 1,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    no_gravity: true,
    no_tile_collide: true,
    knock_back_resist: 0.0,
    npc_slots: 0.0,
};

/// `Main.npcFrameCount[261]` = 1 (`Main.cs:67110`, the 260-269 row of
/// the `:67082` array). `FindFrame` has no case 261 — the single frame
/// never advances.
pub const FUNGI_SPORE_FRAME_COUNT: u32 = 1;

/// `Main.npcFrameCount[265]` = 1 (`Main.cs:67110`, same row). No
/// `FindFrame` case 265 either.
pub const SPORE_FRAME_COUNT: u32 = 1;

/// The arm's per-tick side effects beyond the NPC state — the wiring
/// seams (module docs).
pub struct Style50Outcome {
    /// The C# `this.noTileCollide` write (`:31656` false for 261,
    /// `:31667` true for 265). `Npc` has no such field; the caller's
    /// movement step reads this (`NPC.cs:90490-90498`).
    pub no_tile_collide: bool,
    /// The FungiSpore's contact death (`:31657-31662`): `hp` is already
    /// 0; the caller owes the `HitEffect(0, 100.0)` + `checkDead()`
    /// pair. The velocity/target tail did NOT run this tick.
    pub died_on_contact: bool,
}

/// One fixed step of the aiStyle-50 arm (`NPC.cs:31651-31716`), in C#
/// order. NO RNG parameter — the arm draws nothing (module docs).
///
/// `npc_type` is the vanilla id (261/265) until the roster kinds land;
/// `expert` is `Main.expertMode` (the port is single-difficulty normal,
/// `world.rs:727`, so the wiring caller passes false today — the flag
/// is a parameter so the expert maths stay pinned); `players`/`ticker`
/// feed `TargetClosest()` (`:31670`, faceTarget true) through the
/// shared `npc_target.rs` port (`NPC.cs:77652-77781`). `players` is
/// non-empty (the C# always has the 255 `Main.player` slots).
pub fn ai_style50(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    npc_type: i32,
    expert: bool,
    ticker: u64,
) -> Style50Outcome {
    // `:31653` — EncourageDespawn(5) FIRST, before any branch.
    npc.encourage_despawn(5);
    let no_tile_collide;
    // `:31654-31668` — the type split.
    if npc_type == 261 {
        // `:31656` — the FungiSpore opts INTO tile collision.
        no_tile_collide = false;
        // `:31657-31663` — any contact kills it and RETURNS: no
        // velocity/target/rotation writes this tick.
        if npc.collide_x || npc.collide_y {
            npc.hp = 0; // `:31659` — this.life = 0.
            // `:31660-31661` — HitEffect(0, 100.0) + checkDead(): the
            // caller's seam (dust plan below; checkDead has no
            // 261/265 branch, NPC.cs:78040-78149).
            return Style50Outcome {
                no_tile_collide,
                died_on_contact: true,
            };
        }
    } else {
        // `:31667` — the Spore flies through tiles.
        no_tile_collide = true;
    }
    // `:31669` — the float gravity.
    npc.vy += 0.02;
    // `:31670` — TargetClosest() (faceTarget defaults true).
    target_closest(npc, world, players, true, 50, ticker);
    let p = &players[npc.target as usize];
    // `:31671-31674` — rising while the player is >100px BELOW: damp.
    // Strict `>`; the position read is the post-TargetClosest target.
    if npc.vy < 0.0 && p.y > npc.y + 100.0 {
        npc.vy *= 0.95;
    }
    // `:31675-31678` — the fall-speed clamp (AFTER the damp).
    if npc.vy > 1.0 {
        npc.vy = 1.0;
    }
    let (w, _) = npc.size();
    // `:31679-31694` — the player is strictly right of our right edge.
    if npc.x + w < p.x {
        // `:31681-31684` — moving away: damp once ...
        if npc.vx < 0.0 {
            npc.vx *= 0.98;
        }
        // `:31685-31688` — ... and AGAIN in expert (two separate
        // multiplies, not one 0.9604).
        if expert && npc.vx < 0.0 {
            npc.vx *= 0.98;
        }
        // `:31689-31693` — accelerate, doubled in expert.
        npc.vx += 0.1;
        if expert {
            npc.vx += 0.1;
        }
    }
    // `:31695-31710` — the mirror: strictly left of the player's right
    // edge. No equality case — overlapping/touching boxes steer not.
    else if npc.x > p.x + p.width as f32 {
        if npc.vx > 0.0 {
            npc.vx *= 0.98;
        }
        if expert && npc.vx > 0.0 {
            npc.vx *= 0.98;
        }
        npc.vx -= 0.1;
        if expert {
            npc.vx -= 0.1;
        }
    }
    // `:31711-31714` — the overspeed damp (strict |vx| > 5).
    if npc.vx > 5.0 || npc.vx < -5.0 {
        npc.vx *= 0.97;
    }
    // `:31715` — the tilt rides the final velocity.
    npc.rotation = npc.vx * 0.2;
    Style50Outcome {
        no_tile_collide,
        died_on_contact: false,
    }
}

/// The FungiSpore's per-tick light — `UpdateNPC_CastLights`'s
/// 254..=261 arm (`NPC.cs:92384-92396`), called BEFORE `AI()` in
/// `UpdateNPC` (`:90320` vs `:90423`). The Spore 265 is OUTSIDE the
/// range: it contributes no light anywhere.
///
/// Draws exactly ONE `Main.rand.Next(28, 42)` (`:92386`) — the only
/// RNG touch in the family's per-tick path. Returns
/// `(tileX, tileY, [r, g, b])` for the caller's `EntityLights` feed;
/// the anchor is `(int)Center.X / 16` (`:92395`) — the float-half
/// centre truncated per axis, the `CenterTruncated` form of
/// `entity_lights.rs`.
pub fn fungi_spore_light(
    x: f32,
    y: f32,
    size: (f32, f32),
    mouse_text_color: u8,
    rng: &mut UnifiedRandom,
) -> (i32, i32, [f32; 3]) {
    // `:92386` — num9 = (float)Next(28, 42) * 0.005f
    //          + (float)(270 - Main.mouseTextColor) / 500f.
    let num9 =
        rng.next_range(28, 42) as f32 * 0.005 + (270 - mouse_text_color as i32) as f32 / 500.0;
    // `:92387-92390` — num10 = 0.1, num11 = 0.3 + num9/2,
    // num12 = 0.6 + num9, num13 = 0.35 (the 0.65 override is type 634
    // only, `:92391-92394` — not a spore).
    let r = 0.1 * 0.35;
    let g = (0.3 + num9 / 2.0) * 0.35;
    let b = (0.6 + num9) * 0.35;
    let pos = Vec2::new(x, y);
    (
        tile_x(pos, size.0, XAnchor::CenterTruncated),
        tile_y(pos, size.1, XAnchor::CenterTruncated),
        [r, g, b],
    )
}

/// The `HitEffect` dust arms' branch structure — counts and kinds only,
/// the documented visual skip (module docs): no dust is spawned and NO
/// RNG is drawn (the Spore's `Next(3)` selectors and every
/// `Dust.NewDust` internal draw are skipped wholesale, the
/// `npc_lights.rs` precedent — the stream stays untouched).
///
/// The generic `HitEffect` prologue (`NPC.cs:81656-81858`) is inert for
/// the spores: the `!active` gate is the caller's, the party-hat block
/// needs `townNPC` (`:81660`), daybreak needs the unported buff
/// (`:81730`), and the 686/690/594/661 type checks (`:81752`/`:81782`/
/// `:81789`/`:81813`) are all false.
pub enum SporeHitDust {
    /// No dust: the Spore's `life >= 0` early return (`NPC.cs:87222-
    /// 87225` — note `>=`, so even a just-zeroed life draws nothing).
    None,
    /// The FungiSpore arm (`NPC.cs:89053-89070`, the 254-261 group):
    /// `count` dusts of type 165, alpha 50, scale 1.5, velocity ×2,
    /// noGravity. Hit (`life > 0`): `count` rides the
    /// `dmg / lifeMax * 50.0` loop bound (`:89057`) and RETURNS
    /// (`:89063`); death: a flat 20 (`:89065`). NO gore either way —
    /// the `type != 261` gate (`:89071`) excludes the FungiSpore from
    /// the 375/376/377 trio.
    FungiSpore { count: u32 },
    /// The Spore death arm (`NPC.cs:87226-87236`): 15 dusts, each
    /// `Main.rand.Next(3) != 0` → type 166 else 167, speedX =
    /// hitDirection, speedY = -1. The per-dust selector is the wiring
    /// wave's draw (skipped here with the whole arm).
    Spore { count: u32 },
}

/// The dust plan for one `HitEffect(hitDirection, dmg)` call on a
/// spore. `life` is the C# `this.life` AT THE CALL (the contact death
/// passes 0 after `life = 0`; a strike may pass a negative life).
pub fn spore_hit_dust_plan(npc_type: i32, life: i32, dmg: f64, life_max: i32) -> SporeHitDust {
    if npc_type == 265 {
        // `:87222-87225` — the >= gate.
        if life >= 0 {
            return SporeHitDust::None;
        }
        // `:87226` — the flat 15-iteration loop.
        return SporeHitDust::Spore { count: 15 };
    }
    // `:89055-89064` — the hit half: the double-precision loop bound,
    // counted exactly as the C# `for` runs it.
    if life > 0 {
        let bound = dmg / life_max as f64 * 50.0;
        let mut count = 0u32;
        while (count as f64) < bound {
            count += 1;
        }
        return SporeHitDust::FungiSpore { count };
    }
    // `:89065-89070` — the death half: 20 dusts, no gore for 261.
    SporeHitDust::FungiSpore { count: 20 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

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

    /// A spore-shaped NPC: the roster has no FungiSpore/Spore kind yet,
    /// so the fixture rides a placeholder kind with the C# box through
    /// `size_override` and the 1-HP life.
    fn spore(npc_type: i32, x: f32, y: f32) -> Npc {
        let d = if npc_type == 261 { &FUNGI_SPORE } else { &SPORE };
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    /// `NPC.cs:12364-12378` / `:12423-12437` — both SetDefaults rows,
    /// field by field; the only differences are the box and damage.
    #[test]
    fn the_defaults_rows_match_the_csharp() {
        let f = &FUNGI_SPORE;
        assert_eq!((f.width, f.height), (12, 12));
        assert_eq!(f.ai_style, 50);
        assert_eq!(f.damage, 80);
        assert_eq!(f.defense, 0);
        assert_eq!(f.life_max, 1);
        assert_eq!(f.hit_sound, (3, 1), "SoundID.NPCHit1");
        assert_eq!(f.death_sound, (4, 1), "SoundID.NPCDeath1");
        assert!(f.no_gravity);
        assert!(f.no_tile_collide);
        assert_eq!(f.knock_back_resist, 0.0);
        assert_eq!(f.npc_slots, 0.0);
        let s = &SPORE;
        assert_eq!((s.width, s.height), (18, 18));
        assert_eq!(s.damage, 70);
        assert_eq!(s.ai_style, 50);
        assert_eq!(s.life_max, 1);
        assert_eq!(s.defense, 0);
        assert!(s.no_gravity && s.no_tile_collide);
        assert_eq!(s.knock_back_resist, 0.0);
        assert_eq!(s.npc_slots, 0.0);
        // Main.cs:67110 — the 260-269 row `2,1,8,4,4,1,8,1,4,15`.
        assert_eq!(FUNGI_SPORE_FRAME_COUNT, 1);
        assert_eq!(SPORE_FRAME_COUNT, 1);
    }

    /// `NPC.cs:31653` — EncourageDespawn(5) runs FIRST and only lowers
    /// the window (`NPC.cs:7249-7256` via `npc.rs:1643`).
    #[test]
    fn the_despawn_window_is_encouraged_to_five_first() {
        let w = air();
        let mut n = spore(265, 1000.0, 1000.0);
        n.despawn_timer = 120;
        let _ = ai_style50(&mut n, &w, &[cand(1000.0, 1000.0)], 265, false, 0);
        assert_eq!(n.despawn_timer, 5);
        // A tighter window is never extended.
        n.despawn_timer = 3;
        let _ = ai_style50(&mut n, &w, &[cand(1000.0, 1000.0)], 265, false, 0);
        assert_eq!(n.despawn_timer, 3);
        // Even the death tick encourages first (:31653 precedes :31657).
        let mut m = spore(261, 1000.0, 1000.0);
        m.despawn_timer = 120;
        m.collide_x = true;
        let out = ai_style50(&mut m, &w, &[cand(1000.0, 1000.0)], 261, false, 0);
        assert!(out.died_on_contact);
        assert_eq!(m.despawn_timer, 5);
    }

    /// `NPC.cs:31654-31663` — the FungiSpore's contact death: hp = 0,
    /// the outcome flags, and the EARLY RETURN — no gravity, no
    /// TargetClosest, no rotation this tick. collideY alone suffices.
    #[test]
    fn the_fungi_spore_dies_on_contact_and_skips_the_tail() {
        let w = air();
        for (cx, cy) in [(true, false), (false, true), (true, true)] {
            let mut n = spore(261, 1000.0, 1000.0);
            n.collide_x = cx;
            n.collide_y = cy;
            n.vx = 2.0;
            n.vy = -0.5;
            let out = ai_style50(&mut n, &w, &[cand(1200.0, 1200.0)], 261, false, 7);
            assert!(out.died_on_contact, "collide ({cx},{cy})");
            assert!(!out.no_tile_collide, ":31656 wrote false first");
            assert_eq!(n.hp, 0, ":31659");
            assert_eq!(n.vy, -0.5, "the +0.02 (:31669) never ran");
            assert_eq!(n.vx, 2.0, "no steering on the death tick");
            assert_eq!(n.rotation, 0.0, ":31715 never ran");
            assert_eq!(n.target_set_frame, 0, "TargetClosest never ran");
        }
    }

    /// `NPC.cs:31656`/`:31667` — the flag split: 261 returns false
    /// (opts into UpdateCollision), 265 returns true even with the
    /// collide flags set (it never dies on contact).
    #[test]
    fn the_no_tile_collide_split_follows_the_type() {
        let w = air();
        let mut f = spore(261, 1000.0, 1000.0);
        let out = ai_style50(&mut f, &w, &[cand(1000.0, 1000.0)], 261, false, 0);
        assert!(!out.no_tile_collide);
        assert!(!out.died_on_contact);
        assert_eq!(f.hp, 1, "no contact, no death");
        let mut s = spore(265, 1000.0, 1000.0);
        s.collide_x = true;
        s.collide_y = true;
        let out = ai_style50(&mut s, &w, &[cand(1000.0, 1000.0)], 265, false, 0);
        assert!(out.no_tile_collide);
        assert!(!out.died_on_contact);
        assert_eq!(s.hp, 1, "the Spore ignores the collide flags");
    }

    /// `NPC.cs:31669-31678` — the gravity/damp/clamp ORDER: +0.02 runs
    /// before the damp test (a -0.02 rise becomes 0 and is NOT damp-
    /// eligible), the damp is ×0.95, the clamp to 1 runs after.
    #[test]
    fn the_vertical_order_is_gravity_then_damp_then_clamp() {
        let w = air();
        // Rising, player far below: (-1 + 0.02) * 0.95.
        let mut n = spore(265, 1000.0, 1000.0);
        n.vy = -1.0;
        let _ = ai_style50(&mut n, &w, &[cand(1000.0, 1200.0)], 265, false, 0);
        assert!((n.vy - (-0.98 * 0.95)).abs() < 1e-6);
        // The +0.02 flips -0.02 to 0: not < 0, so no damp even though
        // the player is below.
        let mut m = spore(265, 1000.0, 1000.0);
        m.vy = -0.02;
        let _ = ai_style50(&mut m, &w, &[cand(1000.0, 1200.0)], 265, false, 0);
        assert!((m.vy - 0.0).abs() < 1e-6, "0 is not < 0");
        // Falling fast: +0.02 then the clamp to exactly 1.
        let mut k = spore(265, 1000.0, 1000.0);
        k.vy = 2.0;
        let _ = ai_style50(&mut k, &w, &[cand(1000.0, 1200.0)], 265, false, 0);
        assert_eq!(k.vy, 1.0);
    }

    /// `NPC.cs:31671` — the damp's strict `>`: a player exactly 100px
    /// below does NOT damp; 100.5 does. A rising spore with the player
    /// above never damps.
    #[test]
    fn the_damp_boundary_is_strict_and_directional() {
        let w = air();
        let mut n = spore(265, 1000.0, 1000.0);
        n.vy = -1.0;
        let _ = ai_style50(&mut n, &w, &[cand(1000.0, 1100.0)], 265, false, 0);
        assert!((n.vy - (-0.98)).abs() < 1e-6, "y == npc.y + 100: no damp");
        let mut m = spore(265, 1000.0, 1000.0);
        m.vy = -1.0;
        let _ = ai_style50(&mut m, &w, &[cand(1000.0, 1100.5)], 265, false, 0);
        assert!((m.vy - (-0.98 * 0.95)).abs() < 1e-6, "100.5 > 100: damp");
        // Player above: no damp while rising.
        let mut k = spore(265, 1000.0, 1000.0);
        k.vy = -1.0;
        let _ = ai_style50(&mut k, &w, &[cand(1000.0, 800.0)], 265, false, 0);
        assert!((k.vy - (-0.98)).abs() < 1e-6);
    }

    /// `NPC.cs:31679-31693` — steering right: the away-damp ×0.98 then
    /// +0.1; expert doubles BOTH (two separate ×0.98 multiplies, two
    /// separate +0.1 adds). A spore already moving right only accelers.
    #[test]
    fn the_right_steering_damps_and_accelerates_with_the_expert_double() {
        let w = air();
        // 12-wide FungiSpore at 1000: right edge 1012 < player 1200.
        let mut n = spore(261, 1000.0, 1000.0);
        n.vx = -2.0;
        let _ = ai_style50(&mut n, &w, &[cand(1200.0, 1000.0)], 261, false, 0);
        assert!((n.vx - (-2.0 * 0.98 + 0.1)).abs() < 1e-6);
        let mut e = spore(261, 1000.0, 1000.0);
        e.vx = -2.0;
        let _ = ai_style50(&mut e, &w, &[cand(1200.0, 1000.0)], 261, true, 0);
        assert!(
            (e.vx - (-2.0 * 0.98 * 0.98 + 0.1 + 0.1)).abs() < 1e-6,
            "expert: double damp + double accel"
        );
        // Already moving toward the player: no damp, just the accel.
        let mut t = spore(261, 1000.0, 1000.0);
        t.vx = 2.0;
        let _ = ai_style50(&mut t, &w, &[cand(1200.0, 1000.0)], 261, false, 0);
        assert!((t.vx - 2.1).abs() < 1e-6);
    }

    /// `NPC.cs:31695-31710` — the left mirror: ×0.98 (×2 expert) then
    /// −0.1 (−0.2 expert), keyed on the player's RIGHT edge.
    #[test]
    fn the_left_steering_mirrors_the_right() {
        let w = air();
        // Player at 800, width 20: right edge 820 < spore x 1000.
        let mut n = spore(265, 1000.0, 1000.0);
        n.vx = 2.0;
        let _ = ai_style50(&mut n, &w, &[cand(800.0, 1000.0)], 265, false, 0);
        assert!((n.vx - (2.0 * 0.98 - 0.1)).abs() < 1e-6);
        let mut e = spore(265, 1000.0, 1000.0);
        e.vx = 2.0;
        let _ = ai_style50(&mut e, &w, &[cand(800.0, 1000.0)], 265, true, 0);
        assert!((e.vx - (2.0 * 0.98 * 0.98 - 0.1 - 0.1)).abs() < 1e-6);
    }

    /// `NPC.cs:31679`/`:31695` — the strict edge comparisons: touching
    /// (right edge == player.x) or overlapping boxes steer NOTHING.
    #[test]
    fn the_steering_bounds_are_strict_with_no_equality_case() {
        let w = air();
        // 18-wide Spore at 1000: right edge 1018 == player.x 1018.
        let mut n = spore(265, 1000.0, 1000.0);
        n.vx = -2.0;
        let _ = ai_style50(&mut n, &w, &[cand(1018.0, 1000.0)], 265, false, 0);
        assert_eq!(n.vx, -2.0, "edge == player.x: neither branch");
        // Overlap: spore x inside the player box.
        let mut m = spore(265, 1000.0, 1000.0);
        m.vx = 2.0;
        let _ = ai_style50(&mut m, &w, &[cand(990.0, 1000.0)], 265, false, 0);
        assert_eq!(m.vx, 2.0);
        // The left bound's equality: spore.x == player right edge.
        let mut k = spore(265, 1000.0, 1000.0);
        k.vx = 2.0;
        let _ = ai_style50(&mut k, &w, &[cand(980.0, 1000.0)], 265, false, 0);
        assert_eq!(k.vx, 2.0, "x == player.x + width: neither branch");
    }

    /// `NPC.cs:31711-31715` — the overspeed damp is strict (|vx| == 5
    /// untouched, > 5 ×0.97) and runs AFTER the steering; the rotation
    /// rides the FINAL velocity.
    #[test]
    fn the_overspeed_damp_and_rotation_ride_the_final_velocity() {
        let w = air();
        // Exactly 5, overlapping (no steering): no damp.
        let mut n = spore(265, 1000.0, 1000.0);
        n.vx = 5.0;
        let _ = ai_style50(&mut n, &w, &[cand(990.0, 1000.0)], 265, false, 0);
        assert_eq!(n.vx, 5.0);
        assert!((n.rotation - 1.0).abs() < 1e-6, "rotation = vx * 0.2");
        // Over 5: damp then rotate on the damped value.
        let mut m = spore(265, 1000.0, 1000.0);
        m.vx = -6.0;
        let _ = ai_style50(&mut m, &w, &[cand(990.0, 1000.0)], 265, false, 0);
        assert!((m.vx - (-6.0 * 0.97)).abs() < 1e-6);
        assert!((m.rotation - (-6.0 * 0.97 * 0.2)).abs() < 1e-6);
        // The damp applies to the STEERED value: 5.05 + 0.1 = 5.15 → ×0.97.
        let mut k = spore(265, 1000.0, 1000.0);
        k.vx = 5.05;
        let _ = ai_style50(&mut k, &w, &[cand(1200.0, 1000.0)], 265, false, 0);
        assert!((k.vx - (5.15 * 0.97)).abs() < 1e-6);
    }

    /// `NPC.cs:31670` — TargetClosest runs EVERY tick (unlike the
    /// BlazingWheel's one-shot): the ticker stamp advances and the
    /// facing follows the pick.
    #[test]
    fn target_closest_runs_every_tick() {
        let w = air();
        let mut n = spore(265, 1000.0, 1000.0);
        let _ = ai_style50(&mut n, &w, &[cand(1200.0, 1000.0)], 265, false, 7);
        assert_eq!(n.target, 0);
        assert_eq!(n.target_set_frame, 7);
        assert_eq!(n.facing, 1, "the player is to the right");
        let _ = ai_style50(&mut n, &w, &[cand(800.0, 1000.0)], 265, false, 9);
        assert_eq!(n.target_set_frame, 9, "re-targeted on the next tick");
        assert_eq!(n.facing, -1);
    }

    /// `NPC.cs:92384-92396` — the FungiSpore light: ONE Next(28, 42)
    /// draw, the mouseTextColor pulse term, the 0.35 scale, the
    /// truncated centre tile. The RNG state after the call equals a
    /// clone that drew exactly that one range.
    #[test]
    fn the_fungi_spore_light_draws_one_range_and_pulses() {
        let mut rng = UnifiedRandom::new(12345);
        let mut witness = rng.clone();
        let draw = witness.next_range(28, 42);
        // 12×12 at (160, 176): centre (166, 182) → tiles (10, 11).
        let (i, j, rgb) = fungi_spore_light(160.0, 176.0, (12.0, 12.0), 225, &mut rng);
        assert_eq!(rng, witness, "exactly one Next(28, 42), nothing else");
        assert_eq!((i, j), (10, 11));
        let num9 = draw as f32 * 0.005 + (270 - 225) as f32 / 500.0;
        assert!((rgb[0] - 0.1 * 0.35).abs() < 1e-6);
        assert!((rgb[1] - (0.3 + num9 / 2.0) * 0.35).abs() < 1e-6);
        assert!((rgb[2] - (0.6 + num9) * 0.35).abs() < 1e-6);
    }

    /// `NPC.cs:87222-87236` / `:89053-89070` — the dust plans: the
    /// Spore's `>=` gate (life 0 draws NOTHING), its flat 15 on death;
    /// the FungiSpore's dmg-scaled hit count and flat-20 death, no gore.
    #[test]
    fn the_dust_plans_match_the_hit_effect_arms() {
        // The Spore: >= 0 returns early — even life == 0.
        assert!(matches!(spore_hit_dust_plan(265, 1, 100.0, 1), SporeHitDust::None));
        assert!(matches!(spore_hit_dust_plan(265, 0, 100.0, 1), SporeHitDust::None));
        match spore_hit_dust_plan(265, -1, 100.0, 1) {
            SporeHitDust::Spore { count } => assert_eq!(count, 15),
            _ => panic!("the death arm is the 15-dust loop"),
        }
        // The FungiSpore hit: dmg/lifeMax*50 = 100/1*50 = 5000.
        match spore_hit_dust_plan(261, 1, 100.0, 1) {
            SporeHitDust::FungiSpore { count } => assert_eq!(count, 5000),
            _ => panic!("the hit arm counts the dmg loop"),
        }
        // A fractional bound counts like the C# for-loop: 0.5/1*50 = 25.
        match spore_hit_dust_plan(261, 1, 0.5, 1) {
            SporeHitDust::FungiSpore { count } => assert_eq!(count, 25),
            _ => panic!(),
        }
        // The contact death (life 0, dmg 100): the flat 20, no gore.
        match spore_hit_dust_plan(261, 0, 100.0, 1) {
            SporeHitDust::FungiSpore { count } => assert_eq!(count, 20),
            _ => panic!("the death arm is the flat 20"),
        }
    }

    /// The RNG witness (module docs): the AI step and the dust plan
    /// draw NOTHING — a seeded `UnifiedRandom`'s full state is
    /// bit-identical across 120 ticks of both kinds, expert on and off,
    /// so no stream can shift when the wiring lands.
    #[test]
    fn the_style50_arm_draws_no_rng() {
        let w = air();
        let rng = UnifiedRandom::new(12345);
        let witness = rng.clone();
        let mut f = spore(261, 1000.0, 1000.0);
        let mut s = spore(265, 1000.0, 1000.0);
        for tick in 0..120 {
            let _ = ai_style50(&mut f, &w, &[cand(1200.0, 1100.0)], 261, false, tick);
            let _ = ai_style50(&mut s, &w, &[cand(800.0, 1100.0)], 265, true, tick);
            let _ = spore_hit_dust_plan(261, f.hp, 100.0, 1);
            let _ = spore_hit_dust_plan(265, -1, 100.0, 1);
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
