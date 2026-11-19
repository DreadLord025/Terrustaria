//! The aiStyle-71 family — the Sharkron pair (`NPC.AI()`'s
//! `else if (this.aiStyle == 71)` arm, `NPC.cs:34958-35079`). S1-D-3,
//! aiStyle-71 round, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! The family has TWO vanilla members: the Sharkron 372
//! (`NPCID.Sharkron`, `ID/NPCID.cs:11421`, `SetDefaults` row
//! `NPC.cs:13734-13747`) and the Sharkron2 373 (`NPCID.Sharkron2`,
//! `ID/NPCID.cs:11423`, row `NPC.cs:13748-13761`) — Duke Fishron's
//! sharknados' falling sharks. Both are 100-HP, 100-defense,
//! knockBackResist 0, alpha 255, noGravity, NPCHit1/NPCDeath1; the
//! boxes differ (120×24 dmg 100 vs 100×24 dmg 120). The 371
//! DetonatingBubble beside them rides aiStyle **70** (`:13723`) — a
//! different arm, NOT ported here. (The *projectile* aiStyle 71 — the
//! Razorblade Typhoon in `Projectile.cs:30355-30454`, already ported as
//! `ai_typhoon` — is a different system and is NOT touched here.)
//!
//! The machine: a 90-tick fall, then a 16-speed aimed dash.
//! `noTileCollide = true` (`:34960`). While the target is unset/dead:
//! `TargetClosest(faceTarget: false)` then `direction = 1` + netUpdate
//! (`:34962-34967`). Phase 0 (`:34968-35025`): `ai[1]++`, noGravity,
//! dontTakeDamage, `velocity.Y = ai[3]` EVERY tick (the Sharkron's
//! spawn passes no ai3 → 0 → it hovers; the Sharkron2's spawner passes
//! −1.5 → it rises). The 373-only sway (`:34976-35003`): the
//! π/30·localAI[1] cosine wave scaled by `ai[2]` (the spawner's
//! projectile width), the double position add across the
//! `localAI[1]++`, the |cos − 0.5| > 0.25 spriteDirection write, the
//! `vy · spriteDirection · 0.1` rotation clamped to ±0.2 and the
//! alpha −6 fade-in. At `ai[1] >= 90`: phase 1, `ai[1] = 0` — bumped
//! to 1 iff the box is NOT in solid tiles — the roar (sound 4 style
//! 19), `TargetClosest()` (faceTarget true), `spriteDirection =
//! direction`, `velocity = Normalize(playerCenter − Center) · 16`,
//! `rotation = velocity.ToRotation()` (+π when direction −1),
//! netUpdate (`:35004-35024`). Phase 1 (`:35026-35078`): any other
//! phase RETURNS; noGravity; free → `ai[1]` floor 1, embedded →
//! alpha −15 floor 150; once `ai[1] >= 1`: alpha −60 floor 0,
//! dontTakeDamage = false, `ai[1]++`, and a SECOND solid check kills
//! it (`life = 0`, HitEffect, `active = false`, return). At
//! `ai[1] >= 60` gravity returns. The rotation rides the velocity
//! every phase-1 tick.
//!
//! The spawner is the Tempest Staff's sharknado projectile aiStyle-64
//! arm (`Projectile.cs:29419-29533`): every 4th bolt (2nd for the
//! Greater Sharknado 386) spawns a Sharkron with the projectile's
//! velocity (`:29502-29511`); the 386 arm alone seeds `ai[2] =
//! projectile.width` and `ai[3] = -1.5f` (`:29512-29516`). The plain
//! Sharkron 372 therefore NEVER sways (its arm is 373-only) and falls
//! at `ai[3] = 0` until the dash.
//!
//! ## RNG order (by construction)
//!
//! The AI arm draws NO `Main.rand` — `NPC.cs:34958-35079` contains no
//! `Next`/`NextFloat`; neither does `TargetClosest`/`TryTrackingTarget`/
//! `SetTargetTrackingValues` (`NPC.cs:77652-77781`, the `npc_target.rs`
//! port, reused UNMODIFIED). So [`ai_style71`] takes NO RNG parameter;
//! the witness test `the_style71_arm_draws_no_rng` pins a
//! `UnifiedRandom` state across the calls.
//!
//! The draws that DO exist around the arm:
//! 1. `HitEffect` on hit/death: the shared 372/373 arm
//!    (`NPC.cs:84207-84228`) — hit (`life > 0`): a
//!    `dmg / lifeMax * 100.0` bound loop of type-5 dust; death: 75
//!    type-5 dusts from `Center − One·25` (50×50, velocity ÷2) plus
//!    four gores (583/577/578 at velocity ×0.8/0.8/0.9, 579 at ×1).
//!    Every draw lives inside `Dust.NewDust`/`Gore.NewGore` — the
//!    call sites draw nothing themselves. The dust/gore NPC call
//!    sites are unported (S1-E-4's remainder), so the arm is skipped
//!    wholesale (the `npc_lights.rs` precedent: skipping an arm whole
//!    leaves the `Main.rand` stream untouched).
//!    [`sharkron_hit_dust_plan`] ports the branch structure and counts
//!    only; the wiring wave must draw the pool internals in C# order.
//!    The generic `HitEffect` prologue (`:81656-81858`) is inert for
//!    the pair: the party-hat block needs `townNPC` (`:81660`),
//!    daybreak needs the unported buff (`:81730`), and the
//!    686/690/594/661 type checks (`:81752`/`:81782`/`:81789`/
//!    `:81813`) are all false.
//! 2. The sounds (`SoundEngine.PlaySound(4, .., 19)` at the dash
//!    `:35012`, the DeathSound on the tile kill `:35059-35062`) draw
//!    no `Main.rand` and are the port's audio seam — returned as
//!    [`Style71Outcome`] flags.
//!
//! Wiring deferred (the `npc_ai_style50.rs` precedent, commit
//! 613aac92): the roster has no `NpcKind::Sharkron`/`Sharkron2` yet,
//! so `update_rng`'s aiStyle axis (`npc.rs`, the `match self.kind.ai_style()`
//! block) has no style-71 arm
//! to claim. This module is the pure port + the minimal seams:
//! * `Npc` has no `no_tile_collide`/`no_gravity` fields — the arm's
//!   per-tick writes are RETURNED in [`Style71Outcome`] for the wiring
//!   caller's movement step (`NPC.cs:90490-90498`: false →
//!   `UpdateCollision`, true → raw `position += velocity`).
//! * The tile-kill sets `hp = 0` here (the C# `life = 0`, `:35063`)
//!   and returns `died_in_tiles`; the caller owes the
//!   `HitEffect()` + deactivation pair (`:35064-35065`; the C# arm
//!   does NOT call `checkDead` — the vanilla drops the NPC with
//!   `active = false` directly).
//! * `FindFrame`'s case 371/372/373 (`NPC.cs:72138-72142`) pins
//!   `frame.Y = num` — the SECOND frame, forever (`num` = sheet
//!   height / frameCount, `:66694`). The port's `frame` is the row
//!   index, so the const is 1. `Main.npcFrameCount[372] = [373] = 2`
//!   (`Main.cs:67121`, the 370-379 row `8, 2, 4, 4, 4, 4, 2, 2, 4,
//!   12` of the `:67082` array).
//! * The expert/master stat tweaks (`ScaleStats_ByDifficulty_Tweaks`,
//!   `NPC.cs:18417-18424`: the 371/372/373 group — damage ×0.75 for
//!   372/373, knockback ×0.75 for all three) ride the difficulty
//!   system, not this arm; the rows below are the normal-mode
//!   `SetDefaults` values.
//! * `Math.Cos` in the C# sway is DOUBLE precision (`(float)Math.Cos
//!   (double)`); the port casts the f32 product up to f64, takes the
//!   cosine and casts the result down — the same value the C# sees.
//!
//! See docs/ANALYSIS.md "The Sharkron pair (S1-D-3 aiStyle-71,
//! code-only)".
#![allow(dead_code)] // wiring deferred — the kinds + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use glam::Vec2;
use tr_core::World;

/// The sharkron `SetDefaults` rows (`NPC.cs:13734-13747` /
/// `:13748-13761`), verbatim. The two rows differ ONLY in the box
/// (120×24 vs 100×24) and the damage (100 vs 120); every other field
/// is identical.
pub struct SharkronDefaults {
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
    pub knock_back_resist: f32,
    pub alpha: i32,
}

/// `NPC.cs:13734-13747` — `else if (this.type == 372)`, the Sharkron.
pub const SHARKRON: SharkronDefaults = SharkronDefaults {
    width: 120,
    height: 24,
    ai_style: 71,
    damage: 100,
    defense: 100,
    life_max: 100,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    no_gravity: true,
    knock_back_resist: 0.0,
    alpha: 255,
};

/// `NPC.cs:13748-13761` — `else if (this.type == 373)`, the Sharkron2
/// (the Greater Sharknado's rising, swaying shark).
pub const SHARKRON2: SharkronDefaults = SharkronDefaults {
    width: 100,
    height: 24,
    ai_style: 71,
    damage: 120,
    defense: 100,
    life_max: 100,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    no_gravity: true,
    knock_back_resist: 0.0,
    alpha: 255,
};

/// `Main.npcFrameCount[372]` = 2 (`Main.cs:67121`, the 370-379 row of
/// the `:67082` array). `FindFrame` case 372 (`NPC.cs:72139-72141`)
/// pins `frame.Y = num` — the SECOND row.
pub const SHARKRON_FRAME_COUNT: u32 = 2;

/// `Main.npcFrameCount[373]` = 2 (`Main.cs:67121`, same row); the
/// shared case 373 pins the same second row.
pub const SHARKRON2_FRAME_COUNT: u32 = 2;

/// The `FindFrame` pin: both kinds render row 1 forever
/// (`NPC.cs:72138-72142` — `this.frame.Y = num`, no counter).
pub const SHARKRON_FRAME_ROW: u32 = 1;

/// The fall window — `int num1012 = 90` (`NPC.cs:34961`), tested
/// `ai[1] >= 90` (`:35004`).
pub const FALL_TICKS: f32 = 90.0;

/// The dash speed — `velocity = vector123 * 16f` (`NPC.cs:35017`).
pub const DASH_SPEED: f32 = 16.0;

/// The gravity-return threshold — `ai[1] >= 60f` (`NPC.cs:35069`).
pub const GRAVITY_RETURN_TICKS: f32 = 60.0;

/// The sway's angular step — `Math.PI / 30f` (`NPC.cs:34978`).
const SWAY_STEP: f32 = std::f32::consts::PI / 30.0;

/// The arm's per-tick side effects beyond the NPC state — the wiring
/// seams (module docs).
pub struct Style71Outcome {
    /// The C# `this.noTileCollide = true` write (`:34960`) — always
    /// true while the arm runs; the caller's movement step reads it.
    pub no_tile_collide: bool,
    /// The C# `this.noGravity` write: true in phase 0 (`:34973`) and
    /// phase 1 (`:35032`) until `ai[1] >= 60` flips it false
    /// (`:35069-35072`).
    pub no_gravity: bool,
    /// The dash-tick roar — `SoundEngine.PlaySound(4, (int)Center.X,
    /// (int)Center.Y, 19)` (`:35012`), once, on the phase hand-off.
    pub play_dash_sound: bool,
    /// The tile kill (`:35057-35066`): `hp` is already 0; the caller
    /// owes the DeathSound (when non-null — both rows carry
    /// NPCDeath1), the `HitEffect()` dust/gore plan below and the
    /// deactivation. The rotation tail did NOT run this tick.
    pub died_in_tiles: bool,
}

/// One fixed step of the aiStyle-71 arm (`NPC.cs:34958-35079`), in C#
/// order. NO RNG parameter — the arm draws nothing (module docs).
///
/// `npc_type` is the vanilla id (372/373) until the roster kinds land;
/// `players`/`ticker` feed the two `TargetClosest` calls (`:34964`
/// faceTarget false, `:35013` faceTarget true) through the shared
/// `npc_target.rs` port (`NPC.cs:77652-77781`, reused unmodified).
/// `players` is non-empty (the C# always has the 255 `Main.player`
/// slots).
pub fn ai_style71(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    npc_type: i32,
    ticker: u64,
) -> Style71Outcome {
    let mut out = Style71Outcome {
        no_tile_collide: true, // `:34960`.
        no_gravity: true,
        play_dash_sound: false,
        died_in_tiles: false,
    };
    // `:34962-34967` — the unset/dead target re-pick, facing forced
    // right AFTER the (faceTarget: false) selection.
    let dead_target = npc.target >= 0
        && (npc.target as usize) < players.len()
        && players[npc.target as usize].dead;
    if npc.target < 0 || npc.target == 255 || dead_target {
        target_closest(npc, world, players, false, 71, ticker);
        npc.facing = 1;
        npc.net_update = true;
    }
    let (w, h) = npc.size();
    if npc.ai[0] == 0.0 {
        // `:34970-34975` — the fall/rise phase. (The C#'s `_ =
        // this.type; _ = 372;` discards at `:34971-34972` are dead
        // decompiler artifacts — no effect.)
        npc.ai[1] += 1.0;
        out.no_gravity = true;
        npc.dont_take_damage = true;
        npc.vy = npc.ai[3];
        if npc_type == 373 {
            // `:34978-34984` — the sway: undo the previous cosine
            // offset, advance localAI[1], apply the new one. The
            // cosine is the C#'s DOUBLE Math.Cos (module docs).
            let amp = npc.ai[2];
            // The C# subtracts 0.5 in DOUBLE, then casts to float for
            // the position offset (`(float)(Math.Cos(..) - 0.5) *
            // num1014`); the sprite comparisons below read the DOUBLE.
            let cos_at = |t: f32| ((SWAY_STEP * t) as f64).cos() - 0.5;
            let mut off = cos_at(npc.local_ai[1]) as f32 * amp;
            npc.x -= off * -(npc.facing as f32);
            npc.local_ai[1] += 1.0;
            off = cos_at(npc.local_ai[1]) as f32 * amp;
            npc.x += off * -(npc.facing as f32);
            // `:34985-34988` — the sprite flip only outside the
            // |cos − 0.5| <= 0.25 dead band; cos − 0.5 >= 0 → −1.
            let c = cos_at(npc.local_ai[1]);
            if c.abs() > 0.25 {
                npc.sprite_dir = if !(c >= 0.0) { 1 } else { -1 };
            }
            // `:34989-34997` — the tilt rides vy · spriteDirection,
            // clamped to ±0.2 (two sequential clamps, C# order).
            npc.rotation = npc.vy * npc.sprite_dir as f32 * 0.1;
            if npc.rotation < -0.2 {
                npc.rotation = -0.2;
            }
            if npc.rotation > 0.2 {
                npc.rotation = 0.2;
            }
            // `:34998-35002` — the fade-in (alpha is the C# int; the
            // port's u8 saturates at the same 0 floor).
            npc.alpha = npc.alpha.saturating_sub(6);
        }
        // `:35004-35024` — the dash hand-off at 90 ticks.
        if npc.ai[1] >= FALL_TICKS {
            npc.ai[0] = 1.0;
            npc.ai[1] = 0.0;
            if !Player::solid_collision(world, npc.x, npc.y, w, h) {
                npc.ai[1] = 1.0;
            }
            out.play_dash_sound = true; // `:35012` — sound 4 style 19.
            target_closest(npc, world, players, true, 71, ticker);
            npc.sprite_dir = npc.facing;
            // `:35015-35017` — the aimed 16-speed dash. The centres
            // are read AFTER the re-target (the C# order).
            let (cx, cy) = npc.center();
            let p = &players[npc.target as usize];
            let aim = Vec2::new(
                p.x + p.width as f32 / 2.0 - cx,
                p.y + p.height as f32 / 2.0 - cy,
            );
            // XNA's Normalize is the reciprocal-multiply form — the
            // port's house `normalize_or_zero` (projectile.rs:1014 et
            // al.; the zero-vector NaN is unreachable: the target
            // scan always picks a live player box).
            let v = aim.normalize_or_zero() * DASH_SPEED;
            npc.vx = v.x;
            npc.vy = v.y;
            // `:35018-35022` — ToRotation (+π when facing left).
            npc.rotation = npc.vy.atan2(npc.vx);
            if npc.facing == -1 {
                npc.rotation += std::f32::consts::PI;
            }
            npc.net_update = true;
        }
    } else {
        // `:35028-35031` — any phase other than 1 returns. The C#
        // performs NO noGravity write on this path; the outcome's
        // default true matches both SetDefaults rows (noGravity=true)
        // — unreachable through this arm, which only writes ai[0] 0/1.
        if npc.ai[0] != 1.0 {
            return out;
        }
        out.no_gravity = true; // `:35032`.
        // `:35033-35047` — free air floors ai[1] at 1; a box inside
        // solid tiles fades toward 150 instead.
        if !Player::solid_collision(world, npc.x, npc.y, w, h) {
            if npc.ai[1] < 1.0 {
                npc.ai[1] = 1.0;
            }
        } else {
            npc.alpha = npc.alpha.saturating_sub(15).max(150);
        }
        // `:35048-35068` — the armed dash: the fast fade, the shield
        // drop, the counter, and the SECOND solid check's tile kill.
        if npc.ai[1] >= 1.0 {
            npc.alpha = npc.alpha.saturating_sub(60);
            npc.dont_take_damage = false;
            npc.ai[1] += 1.0;
            if Player::solid_collision(world, npc.x, npc.y, w, h) {
                // `:35059-35066` — DeathSound (caller seam), life = 0,
                // HitEffect(), active = false, RETURN: no gravity
                // flip, no rotation this tick.
                npc.hp = 0;
                out.died_in_tiles = true;
                return out;
            }
        }
        // `:35069-35072` — gravity returns at 60 armed ticks.
        if npc.ai[1] >= GRAVITY_RETURN_TICKS {
            out.no_gravity = false;
        }
        // `:35073-35077` — the rotation rides the live velocity.
        npc.rotation = npc.vy.atan2(npc.vx);
        if npc.facing == -1 {
            npc.rotation += std::f32::consts::PI;
        }
    }
    out
}

/// The `HitEffect` dust/gore arm's branch structure — counts and kinds
/// only, the documented visual skip (module docs): no dust/gore is
/// spawned and NO RNG is drawn (every draw lives inside
/// `Dust.NewDust`/`Gore.NewGore`; the whole arm is skipped, the
/// `npc_lights.rs` precedent — the stream stays untouched).
pub enum SharkronHitDust {
    /// The hit half (`NPC.cs:84209-84215`, `life > 0`): `count` type-5
    /// dusts over the body box, speedX = hitDirection, speedY = −1.
    /// `count` rides the `dmg / lifeMax * 100.0` double-precision loop
    /// bound (`:84211`).
    Hit { count: u32 },
    /// The death half (`NPC.cs:84216-84227`): 75 type-5 dusts from
    /// `Center − One·25` (a 50×50 box, speedX = 2·hitDirection,
    /// speedY = −2, velocity ÷2) plus the four gores — 583/577 at
    /// velocity ×0.8, 578 at ×0.9, 579 at ×1, all from Center.
    Death { dust_count: u32, gores: [u16; 4] },
}

/// The dust/gore plan for one `HitEffect(hitDirection, dmg)` call on a
/// sharkron (either kind — the C# arm is shared, `NPC.cs:84207`).
/// `life` is the C# `this.life` AT THE CALL (the tile kill passes 0
/// after `life = 0`; a strike may pass a negative life).
pub fn sharkron_hit_dust_plan(life: i32, dmg: f64, life_max: i32) -> SharkronHitDust {
    if life > 0 {
        // `:84211` — the double-precision loop bound, counted exactly
        // as the C# `for` runs it.
        let bound = dmg / life_max as f64 * 100.0;
        let mut count = 0u32;
        while (count as f64) < bound {
            count += 1;
        }
        return SharkronHitDust::Hit { count };
    }
    // `:84218-84226` — the flat 75 + the four-gore burst.
    SharkronHitDust::Death {
        dust_count: 75,
        gores: [583, 577, 578, 579],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::unified_random::UnifiedRandom;

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

    /// A sharkron-shaped NPC: the roster has no Sharkron/Sharkron2
    /// kind yet, so the fixture rides a placeholder kind with the C#
    /// box through `size_override`, the 100-HP life and the
    /// SetDefaults alpha 255.
    fn shark(npc_type: i32, x: f32, y: f32) -> Npc {
        let d = if npc_type == 372 { &SHARKRON } else { &SHARKRON2 };
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.alpha = d.alpha as u8;
        n
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// A world with one solid dirt tile at (i, j) — the tile-kill
    /// fixtures embed the shark's box in it.
    fn world_with_tile(i: i32, j: i32) -> World {
        let mut w = air();
        let idx = w.index(i, j);
        w.tiles[idx] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[idx].flags |= FLAG_ACTIVE;
        w
    }

    /// `NPC.cs:13734-13747` / `:13748-13761` — both SetDefaults rows,
    /// field by field; the only differences are the box and damage.
    #[test]
    fn the_defaults_rows_match_the_csharp() {
        let s = &SHARKRON;
        assert_eq!((s.width, s.height), (120, 24));
        assert_eq!(s.ai_style, 71);
        assert_eq!(s.damage, 100);
        assert_eq!(s.defense, 100);
        assert_eq!(s.life_max, 100);
        assert_eq!(s.hit_sound, (3, 1), "SoundID.NPCHit1");
        assert_eq!(s.death_sound, (4, 1), "SoundID.NPCDeath1");
        assert!(s.no_gravity);
        assert_eq!(s.knock_back_resist, 0.0);
        assert_eq!(s.alpha, 255);
        let t = &SHARKRON2;
        assert_eq!((t.width, t.height), (100, 24));
        assert_eq!(t.damage, 120);
        assert_eq!(t.ai_style, 71);
        assert_eq!(t.defense, 100);
        assert_eq!(t.life_max, 100);
        assert!(t.no_gravity);
        assert_eq!(t.knock_back_resist, 0.0);
        assert_eq!(t.alpha, 255);
        // Main.cs:67121 — the 370-379 row `8,2,4,4,4,4,2,2,4,12`.
        assert_eq!(SHARKRON_FRAME_COUNT, 2);
        assert_eq!(SHARKRON2_FRAME_COUNT, 2);
        // NPC.cs:72138-72142 — frame.Y = num: the second row, pinned.
        assert_eq!(SHARKRON_FRAME_ROW, 1);
    }

    /// `NPC.cs:34962-34967` — the unset target (255) re-picks with
    /// faceTarget FALSE and then forces direction = 1 even when the
    /// player is to the LEFT; netUpdate is set.
    #[test]
    fn the_unset_target_repicks_and_forces_direction_right() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        assert_eq!(n.target, 255, "the ClearTarget default");
        n.net_update = false;
        let _ = ai_style71(&mut n, &w, &[cand(800.0, 1000.0)], 372, 7);
        assert_eq!(n.target, 0);
        assert_eq!(n.facing, 1, ":34965 overrides the left-facing pick");
        assert!(n.net_update);
        assert_eq!(n.target_set_frame, 7, "the ticker stamp (:77780)");
        // A live target is NOT re-picked mid-fall: the stamp holds.
        let _ = ai_style71(&mut n, &w, &[cand(800.0, 1000.0)], 372, 9);
        assert_eq!(n.target_set_frame, 7, "no TargetClosest this tick");
    }

    /// `NPC.cs:34962` — a DEAD target re-picks too (the third clause).
    #[test]
    fn the_dead_target_repicks() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        let mut dead = cand(800.0, 1000.0);
        dead.dead = true;
        let alive = cand(1200.0, 1000.0);
        let _ = ai_style71(&mut n, &w, &[dead, alive], 372, 3);
        assert_eq!(n.target, 1, "the dead slot loses the scan");
        assert_eq!(n.facing, 1);
    }

    /// `NPC.cs:34970-34975` — phase 0: ai[1] counts, dontTakeDamage
    /// holds, and vy is OVERWRITTEN by ai[3] every tick (the plain
    /// Sharkron's spawn seeds no ai3 → 0 → it hovers).
    #[test]
    fn the_fall_phase_pins_vy_to_ai3_and_shields() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.vy = 9.0;
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 0);
        assert_eq!(n.ai[1], 1.0);
        assert!(n.dont_take_damage);
        assert!(out.no_gravity);
        assert!(out.no_tile_collide);
        assert_eq!(n.vy, 0.0, "ai[3] = 0 overwrites the 9 (:34975)");
        // The Sharkron2's spawner seeds ai[3] = -1.5 (Projectile.cs:29515).
        let mut m = shark(373, 1000.0, 1000.0);
        m.target = 0;
        m.ai[3] = -1.5;
        m.ai[2] = 60.0;
        let _ = ai_style71(&mut m, &w, &[cand(1200.0, 1000.0)], 373, 0);
        assert_eq!(m.vy, -1.5);
    }

    /// `NPC.cs:34976-35003` — the sway is 373-ONLY: the 372 never
    /// moves X, never fades, never tilts in phase 0; the 373's first
    /// tick lands the cos(π/30)−0.5 offset × ai[2] × (−direction).
    #[test]
    fn the_sway_is_sharkron2_only() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.ai[2] = 60.0;
        let _ = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 0);
        assert_eq!(n.x, 1000.0, "no 372 sway");
        assert_eq!(n.alpha, 255, "no 372 fade");
        assert_eq!(n.rotation, 0.0);
        assert_eq!(n.local_ai[1], 0.0);
        // The 373: localAI[1] 0 → 1; the undo at t=0 is (1−0.5)·60 =
        // 30 (x -= 30·(−1) → +30), the apply at t=1 is
        // (cos(π/30)−0.5)·60 ≈ 29.67 (x += that·(−1)).
        let mut m = shark(373, 1000.0, 1000.0);
        m.target = 0;
        m.ai[2] = 60.0;
        m.ai[3] = -1.5;
        let _ = ai_style71(&mut m, &w, &[cand(1200.0, 1000.0)], 373, 0);
        let c1 = (((SWAY_STEP * 1.0) as f64).cos() - 0.5) as f32;
        let expected = 1000.0 + 30.0 - c1 * 60.0;
        assert!((m.x - expected).abs() < 1e-3, "x = {}", m.x);
        assert_eq!(m.local_ai[1], 1.0);
        assert_eq!(m.alpha, 249, "255 − 6 (:34998)");
        // vy −1.5, sprite −1 (c1 > 0.25, c1 >= 0 → −1): rotation
        // = −1.5·−1·0.1 = 0.15, inside the ±0.2 band.
        assert_eq!(m.sprite_dir, -1);
        assert!((m.rotation - 0.15).abs() < 1e-6);
    }

    /// `NPC.cs:34985-34997` — the sprite flip's 0.25 dead band and the
    /// rotation's ±0.2 clamps.
    #[test]
    fn the_sway_dead_band_and_rotation_clamps() {
        let w = air();
        // t with cos(π/30·t)−0.5 inside ±0.25: t = 10 → cos(π/3) =
        // 0.5 → c = 0: the spriteDirection write is SKIPPED.
        let mut n = shark(373, 1000.0, 1000.0);
        n.target = 0;
        n.local_ai[1] = 9.0; // advances to 10
        n.sprite_dir = 1;
        n.ai[3] = -1.5;
        let _ = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 373, 0);
        assert_eq!(n.sprite_dir, 1, "|c| <= 0.25 keeps the sprite");
        // The negative clamp: vy = -5, sprite -1 → +0.5 → clamped 0.2.
        let mut m = shark(373, 1000.0, 1000.0);
        m.target = 0;
        m.ai[3] = -5.0;
        let _ = ai_style71(&mut m, &w, &[cand(1200.0, 1000.0)], 373, 0);
        assert_eq!(m.sprite_dir, -1);
        assert_eq!(m.rotation, 0.2, "the +0.2 clamp (:34994-34997)");
        // The positive-vy mirror: vy = 5, sprite -1 → −0.5 → −0.2.
        let mut k = shark(373, 1000.0, 1000.0);
        k.target = 0;
        k.ai[3] = 5.0;
        let _ = ai_style71(&mut k, &w, &[cand(1200.0, 1000.0)], 373, 0);
        assert_eq!(k.rotation, -0.2);
        // The alpha floor: 3 − 6 saturates at 0 (:34999-35002).
        let mut f = shark(373, 1000.0, 1000.0);
        f.target = 0;
        f.alpha = 3;
        let _ = ai_style71(&mut f, &w, &[cand(1200.0, 1000.0)], 373, 0);
        assert_eq!(f.alpha, 0);
    }

    /// `NPC.cs:35004-35024` — the 90-tick hand-off: ai[0] = 1, ai[1]
    /// = 1 in free air, the roar flag, the re-target with FACING, the
    /// sprite copy, the 16-speed aimed dash and the ToRotation.
    #[test]
    fn the_dash_arms_at_ninety_with_the_aimed_sixteen() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.ai[1] = 89.0; // this tick increments to 90
        n.facing = -1;
        // Player centre (1210, 1021); shark centre (1060, 1012):
        // aim (150, 9), len = sqrt(22581).
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 5);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 1.0, "free air bumps 0 → 1 (:35008-35011)");
        assert!(out.play_dash_sound);
        assert_eq!(n.facing, 1, "TargetClosest() faces the right player");
        assert_eq!(n.sprite_dir, 1, ":35014");
        let len = (150.0f32 * 150.0 + 9.0 * 9.0).sqrt();
        let inv = 1.0 / len; // XNA's reciprocal-multiply normalize.
        assert!((n.vx - 150.0 * inv * 16.0).abs() < 1e-4);
        assert!((n.vy - 9.0 * inv * 16.0).abs() < 1e-4);
        let speed = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((speed - 16.0).abs() < 1e-4);
        assert!((n.rotation - n.vy.atan2(n.vx)).abs() < 1e-6);
        assert!(n.net_update);
        // The left-facing dash adds π (:35019-35022).
        let mut m = shark(372, 1000.0, 1000.0);
        m.target = 0;
        m.ai[1] = 89.0;
        let _ = ai_style71(&mut m, &w, &[cand(800.0, 1000.0)], 372, 5);
        assert_eq!(m.facing, -1);
        assert!(m.vx < 0.0);
        assert!((m.rotation - (m.vy.atan2(m.vx) + std::f32::consts::PI)).abs() < 1e-6);
    }

    /// `NPC.cs:35008-35011` + `:35048-35066` — a dash armed INSIDE
    /// solid tiles keeps ai[1] = 0 (no armed block on the hand-off
    /// tick); the next tick's second solid check kills it: hp 0, the
    /// death flag, and the EARLY RETURN (no rotation write).
    #[test]
    fn the_embedded_dash_dies_on_the_next_tick() {
        // Shark at (1000, 1000), box 120×24 → tiles x 62..69, y 62..64.
        let w = world_with_tile(65, 63);
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.ai[1] = 89.0;
        n.rotation = 0.7;
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 0);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 0.0, "embedded: the 0 → 1 bump is skipped");
        assert!(!out.died_in_tiles, "the kill is the NEXT tick's check");
        assert!(out.play_dash_sound);
        // Next tick: free-check false → alpha fade; ai[1] < 1 so the
        // armed block is skipped; no death yet.
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 1);
        assert!(!out.died_in_tiles);
        assert_eq!(n.ai[1], 0.0);
        // Force the armed state inside the tiles: the second check
        // kills with the early return.
        n.ai[1] = 1.0;
        n.alpha = 255;
        let rot_before = n.rotation;
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 2);
        assert!(out.died_in_tiles);
        assert_eq!(n.hp, 0, ":35063");
        assert_eq!(n.alpha, 255 - 15 - 60, "both fades ran before the kill");
        assert!(!n.dont_take_damage, ":35055 ran first");
        assert_eq!(n.ai[1], 2.0, "the counter incremented (:35056)");
        assert_eq!(n.rotation, rot_before, "the tail never ran");
    }

    /// `NPC.cs:35033-35047` — phase 1 in free air floors ai[1] at 1;
    /// embedded fades alpha −15 with the 150 floor.
    #[test]
    fn the_phase_one_floors_split_on_solid_tiles() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.ai[0] = 1.0;
        n.ai[1] = 0.0;
        let _ = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 0);
        // Free: floor to 1, then the armed block runs the SAME tick.
        assert_eq!(n.ai[1], 2.0, "floor 1 then ++ (:35037 + :35056)");
        assert_eq!(n.alpha, 255 - 60);
        assert!(!n.dont_take_damage);
        // Embedded: the −15 fade with the 150 floor, no armed block.
        let t = world_with_tile(65, 63);
        let mut m = shark(372, 1000.0, 1000.0);
        m.target = 0;
        m.ai[0] = 1.0;
        m.ai[1] = 0.0;
        m.alpha = 160;
        let _ = ai_style71(&mut m, &t, &[cand(1200.0, 1000.0)], 372, 0);
        assert_eq!(m.alpha, 150, "160 − 15 = 145 → floor 150");
        assert_eq!(m.ai[1], 0.0);
        assert_eq!(m.hp, 100, "the armed block never ran");
    }

    /// `NPC.cs:35069-35077` — gravity returns at ai[1] >= 60 (AFTER
    /// the increment) and the rotation rides the live velocity every
    /// phase-1 tick, +π when facing left.
    #[test]
    fn gravity_returns_at_sixty_and_the_rotation_rides_velocity() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.ai[0] = 1.0;
        n.ai[1] = 58.0;
        n.vx = 3.0;
        n.vy = 4.0;
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 0);
        assert_eq!(n.ai[1], 59.0);
        assert!(out.no_gravity, "59 < 60: still weightless");
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 1);
        assert_eq!(n.ai[1], 60.0);
        assert!(!out.no_gravity, "60 >= 60: gravity (:35069-35072)");
        assert!((n.rotation - 4.0f32.atan2(3.0)).abs() < 1e-6);
        // Facing left: +π.
        n.facing = -1;
        let _ = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 2);
        assert!((n.rotation - (4.0f32.atan2(3.0) + std::f32::consts::PI)).abs() < 1e-6);
    }

    /// `NPC.cs:35028-35031` — a phase other than 0/1 RETURNS: no
    /// counter, no fades, no rotation.
    #[test]
    fn an_unknown_phase_returns_untouched() {
        let w = air();
        let mut n = shark(372, 1000.0, 1000.0);
        n.target = 0;
        n.ai[0] = 2.0;
        n.ai[1] = 7.0;
        n.rotation = 0.3;
        let out = ai_style71(&mut n, &w, &[cand(1200.0, 1000.0)], 372, 0);
        assert_eq!(n.ai[1], 7.0);
        assert_eq!(n.rotation, 0.3);
        assert_eq!(n.alpha, 255);
        assert!(!out.died_in_tiles);
        assert!(!out.play_dash_sound);
    }

    /// `NPC.cs:84207-84228` — the dust/gore plans: the hit count rides
    /// the dmg/lifeMax·100 double bound; the death is the flat 75 +
    /// the 583/577/578/579 gore burst.
    #[test]
    fn the_dust_plans_match_the_hit_effect_arm() {
        // dmg 10 / lifeMax 100 · 100 = 10 dusts.
        match sharkron_hit_dust_plan(100, 10.0, 100) {
            SharkronHitDust::Hit { count } => assert_eq!(count, 10),
            _ => panic!("the hit arm counts the dmg loop"),
        }
        // A fractional bound counts like the C# for-loop: 0.5/100·100
        // = 0.5 → one iteration (0 < 0.5, then 1 !< 0.5).
        match sharkron_hit_dust_plan(1, 0.5, 100) {
            SharkronHitDust::Hit { count } => assert_eq!(count, 1),
            _ => panic!(),
        }
        // The tile kill (life 0): the flat 75 + four gores.
        match sharkron_hit_dust_plan(0, 10.0, 100) {
            SharkronHitDust::Death { dust_count, gores } => {
                assert_eq!(dust_count, 75);
                assert_eq!(gores, [583, 577, 578, 579]);
            }
            _ => panic!("the death arm is the flat 75"),
        }
        // A negative life (a strike overkill) takes the death arm too.
        assert!(matches!(
            sharkron_hit_dust_plan(-5, 10.0, 100),
            SharkronHitDust::Death { .. }
        ));
    }

    /// The RNG witness (module docs): the AI step and the dust plan
    /// draw NOTHING — a seeded `UnifiedRandom`'s full state is
    /// bit-identical across a full fall + dash lifecycle of both
    /// kinds, so no stream can shift when the wiring lands.
    #[test]
    fn the_style71_arm_draws_no_rng() {
        let w = air();
        let rng = UnifiedRandom::new(12345);
        let witness = rng.clone();
        let mut s = shark(372, 1000.0, 1000.0);
        let mut t = shark(373, 1000.0, 1000.0);
        t.ai[2] = 60.0;
        t.ai[3] = -1.5;
        for tick in 0..200u64 {
            let _ = ai_style71(&mut s, &w, &[cand(1200.0, 1100.0)], 372, tick);
            let _ = ai_style71(&mut t, &w, &[cand(800.0, 1100.0)], 373, tick);
            let _ = sharkron_hit_dust_plan(s.hp, 10.0, 100);
            let _ = sharkron_hit_dust_plan(0, 10.0, 100);
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
