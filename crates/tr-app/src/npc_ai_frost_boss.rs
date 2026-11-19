//! Frost Moon boss AI — S1-F-13c slice C (this round: **IceQueen only**).
//!
//! The C# arm ported here, keyed by the vanilla `NPC.type` / `aiStyle`
//! (the `SetDefaults` row `NPC.cs:13369-13385`):
//!
//! * **IceQueen (345, aiStyle 60)** — the three-phase frost flyer
//!   (`NPC.cs:33374-33680`, the `aiStyle == 60` arm of `NPC.AI`): the
//!   side charge (phase 0, the 348 FrostWave), the shard rain (phase 1,
//!   the 349 FrostShard drops) and the spin barrage (phase 2, the 349
//!   twirl), the hp-tiered speeds and the `ai[0] == -1` phase re-pick
//!   (`:33667-33679` — a SIBLING `if` of the phase chain, so the re-pick
//!   runs on the SAME tick a phase flips to -1, and even by day when
//!   `ai[0]` is already -1).
//!
//! **Deferred to the next slice-C round (docs/ANALYSIS.md, "IceQueen
//! core AI"):** Everscream (344, aiStyle 57, `NPC.cs:32761-33043` — the
//! type-344 branches only; the shared arm's MourningWood branches are
//! S1-F-13b2's) and SantaNK1 (346, aiStyle 61, `:33681-33921`). Also
//! deferred: the `FindFrame` arm (`:72425+`), the boss-head helper
//! (`GetBossHeadRotation` `:7624-7629`) and the death/bag/drop arms.
//! The aiStyle-60 arm itself has NO `Lighting.AddLight`, NO dust and NO
//! `netUpdate` writes — nothing else is skipped inside it.
//!
//! **Integration seam / dependency:** the `NpcKind::IceQueen` variant +
//! the full `SetDefaults` row for 345 are owned by the parallel worker B
//! (`task/S1-F-13c-defaults-b`, 345-352). Until it lands, this module is
//! deliberately disjoint (the pushed `npc_ai_frost_mob.rs` pattern,
//! commit 5ef9b24b): the machine runs on the plain [`FrostBossState`]
//! (the AI-visible subset of `NPC`'s fields) and the dispatch
//! ([`frost_boss_ai_step`]) keys on the vanilla type id 345 + the C#
//! `aiStyle` 60 ([`ICE_QUEEN_AI_STYLE`]). The wiring worker copies
//! `Npc`'s fields in/out and runs `npc_target::target_closest` where the
//! C# arm calls `TargetClosest()` (`:33393`, `:33403`, `:33518`,
//! `:33622`, `:33670`); the resolved slot rides [`FrostBossTarget`].
//! The size/lifeMax below are the AI-relevant `SetDefaults` inputs
//! (`NPC.cs:13371-13376`: 130×140, 34000) and must agree with B's table
//! when it lands. `Main.netMode != 1` (`:33479`, `:33507`, `:33576`,
//! `:33611`, `:33658`) is the port's always-true server/local path.
//! IceQueen flies: `noTileCollide`/`noGravity` come from `SetDefaults`
//! (`:13381-13382`) — the caller integrates the velocity directly.
//!
//! **RNG order (by construction — every `Main.rand.Next` in the arm):**
//! phase 0: the per-tick `Next(1, 4)` (`:33509` — the FrostWave shot
//! itself draws NOTHING); phase 1: the shard-drop `Next(5)` (ai0,
//! `:33607` — ONLY when the cadence fires AND the `SolidTile` gate
//! passes) then the per-tick `Next(1, 4)` (`:33613`); phase 2:
//! `Next(-1000, 1001)` ×2 (`:33624-33625`, EVERY tick, BEFORE the
//! velocity brake) then the per-tick `Next(1, 4)` (`:33660`); the
//! re-pick: `Next(3)` (`:33669`, drawn even when the >1000px check
//! forces the pick to 0). The day flee and an unmatched phase draw
//! nothing (unless `ai[0] == -1` re-picks). The witness tests pin each
//! order against a same-seeded probe stream.

// S1-F-13c wiring landed: the style axis (`npc.rs::update_rng_dust`) claims
// 345 through `ice_queen_step` below. The seam constants stay test-pinned,
// so the module keeps the dead-code allowance.
#![allow(dead_code)]

use tr_core::{UnifiedRandom, World};

use crate::npc::{HostileShotAi, Npc};
use crate::player::Player;

/// `NPCID.IceQueen` — `NPCID.cs:11367`.
pub const ICE_QUEEN_TYPE: u16 = 345;
/// The C#'s `this.aiStyle` for 345 — `NPC.cs:13373`.
pub const ICE_QUEEN_AI_STYLE: u8 = 60;

/// The AI-relevant `SetDefaults` inputs `(width, height, lifeMax)` —
/// `NPC.cs:13371-13376` (130×140, 34000). The full stat row
/// (damage 120/defense 38/value/knockBackResist 0/npcSlots 5/coldDamage)
/// is worker B's table.
pub fn ice_queen_ai_defaults() -> (f32, f32, i32) {
    (130.0, 140.0, 34000)
}

/// The AI-visible subset of `NPC`'s fields the IceQueen machine
/// reads/writes (the `npc_ai_frost_mob.rs` `FrostMobState` shape, trimmed
/// to what aiStyle 60 touches — `direction`/`localAI`/`timeLeft` are
/// unread by this arm; the next round's Everscream/Santa machines widen
/// it). The wiring worker copies these in/out of `Npc`.
#[derive(Clone, Copy, Debug)]
pub struct FrostBossState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `NPC.ai[0..3]`.
    pub ai: [f32; 4],
    pub rotation: f32,
    /// `NPC.life`/`lifeMax`.
    pub life: i32,
    pub life_max: i32,
    /// `Entity.width`/`height` (the `SetDefaults` box; the C# fields are
    /// ints — the `(float)base.height` casts are exact).
    pub width: f32,
    pub height: f32,
}

impl FrostBossState {
    /// A fresh IceQueen at (x, y) with the `SetDefaults` AI inputs and
    /// the C#'s field defaults (`ai` zeroed).
    pub fn ice_queen(x: f32, y: f32) -> Self {
        let (width, height, life_max) = ice_queen_ai_defaults();
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            ai: [0.0; 4],
            rotation: 0.0,
            life: life_max,
            life_max,
            width,
            height,
        }
    }

    /// `Entity.Center` (`Entity.cs:50-55`): position + size/2. The C#
    /// re-reads `Center` throughout the arm, but the position never
    /// moves inside `AI()` (the integrate happens after), so one read
    /// per tick is exact.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// The resolved `Main.player[this.target]` box. `TargetClosest()` call
/// sites: `NPC.cs:33393` (the initial side pick), `:33403` (phase 0 per
/// tick), `:33518` (phase 1), `:33622` (phase 2), `:33670` (the re-pick).
/// The wiring worker runs `npc_target::target_closest` at those points
/// and passes the picked slot here. aiStyle 60 reads neither `active`
/// nor `dead` (Santa's `flag64` does — next round).
#[derive(Clone, Copy, Debug)]
pub struct FrostBossTarget {
    /// `Player.position.X`/`Y`.
    pub x: f32,
    pub y: f32,
    /// `Player.width`/`height` (the 20×42 box).
    pub width: f32,
    pub height: f32,
}

impl FrostBossTarget {
    /// `Player.Center` (`Entity.cs:50-55`).
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// A hostile-shot request — `Projectile.NewProjectile`'s
/// `(X, Y, SpeedX, SpeedY, Type, Damage, KnockBack, ai0, ai1)` payload.
/// KnockBack is 0f at every call site ported here, so it is not carried.
/// The only non-zero extra: the shard-rain drop's `ai0 = Next(5)`
/// (`:33607`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrostBossShot {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub ai0: f32,
    pub ai1: f32,
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`) — the IceQueen
/// shard-drop gate (`NPC.cs:33600`): active, solid, not solid-top, not
/// half-brick, slopeless and not actuated off. An out-of-world index
/// throws in the C# and the empty `catch` returns false (`:70128-70131`,
/// the `growth.rs::growth_solid` convention); an in-world null tile
/// returns TRUE (`:70115-70117`) but in-world tiles are never null in
/// the port, so `None` ⇒ false is exact.
fn worldgen_solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// **IceQueen (345)** — the `aiStyle == 60` arm (`NPC.cs:33374-33680`),
/// verbatim. `noGravity`/`noTileCollide` come from `SetDefaults`
/// (`:13381-13382`) — the caller integrates. `TargetClosest()` call
/// sites (`:33393`, `:33403`, `:33518`, `:33622`, `:33670`) are the
/// caller's seam. The RNG draws ride the caller's `Main.rand` stream in
/// the module-header order; the witness tests pin them.
pub fn ice_queen_ai(
    s: &mut FrostBossState,
    world: &World,
    target: &FrostBossTarget,
    day_time: bool,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostBossShot>,
) {
    let (cx, cy) = s.center();
    let (pcx, pcy) = target.center();
    if day_time {
        // `:33376-33388` — the day flee (NO EncourageDespawn here,
        // unlike aiStyles 57/61).
        if s.vx > 0.0 {
            s.vx += 0.25;
        } else {
            s.vx -= 0.25;
        }
        s.vy -= 0.1;
        s.rotation = s.vx * 0.05;
    } else if s.ai[0] == 0.0 {
        // `:33389-33515` — phase 0: the side charge + FrostWave.
        // `:33391-33402` — the initial side pick (TargetClosest ×2 is
        // the caller's seam).
        if s.ai[2] == 0.0 {
            s.ai[2] = if cx < pcx { 1.0 } else { -1.0 };
        }
        // `:33404-33413` — the 800px turn-around.
        let num897 = 800.0f32;
        let num898 = (cx - pcx).abs();
        if cx < pcx && s.ai[2] < 0.0 && num898 > num897 {
            s.ai[2] = 0.0;
        }
        if cx > pcx && s.ai[2] > 0.0 && num898 > num897 {
            s.ai[2] = 0.0;
        }
        // `:33414-33430` — the hp-tiered accel/cap (the `(double)`
        // compares).
        let (mut num899, mut num900) = (0.45f32, 7.0f32);
        if (s.life as f64) < (s.life_max as f64) * 0.75 {
            num899 = 0.55;
            num900 = 8.0;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.5 {
            num899 = 0.7;
            num900 = 10.0;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.25 {
            num899 = 0.8;
            num900 = 11.0;
        }
        // `:33431-33439`.
        s.vx += s.ai[2] * num899;
        if s.vx > num900 {
            s.vx = num900;
        }
        if s.vx < 0.0 - num900 {
            s.vx = 0.0 - num900;
        }
        // `:33440-33456` — the hover band around the player's TOP.
        let num901 = target.y - (s.y + s.height);
        if num901 < 150.0 {
            s.vy -= 0.2;
        }
        if num901 > 200.0 {
            s.vy += 0.2;
        }
        if s.vy > 8.0 {
            s.vy = 8.0;
        }
        if s.vy < -8.0 {
            s.vy = -8.0;
        }
        // `:33457`.
        s.rotation = s.vx * 0.05;
        // `:33458-33506` — the FrostWave cadence (above the player and
        // inside 500px, or counting down a negative ai[3]).
        if (num898 < 500.0 || s.ai[3] < 0.0) && s.y < target.y {
            s.ai[3] += 1.0;
            let mut num902 = 13;
            if (s.life as f64) < (s.life_max as f64) * 0.75 {
                num902 = 12;
            }
            if (s.life as f64) < (s.life_max as f64) * 0.5 {
                num902 = 11;
            }
            if (s.life as f64) < (s.life_max as f64) * 0.25 {
                num902 = 10;
            }
            num902 += 1; // `:33474`.
            if s.ai[3] > num902 as f32 {
                s.ai[3] = -(num902 as f32);
            }
            if s.ai[3] == 0.0 {
                // `:33481-33484` — the 7-tick lead on the shot origin
                // (the ALREADY-updated velocity.X); the aim is the
                // player's centre X (`position.X + width*0.5`) and
                // centre Y.
                let v_x = cx + s.vx * 7.0;
                let v_y = cy;
                let num903 = pcx - v_x;
                let num904 = pcy - v_y;
                let mut num905 = (num903 * num903 + num904 * num904).sqrt();
                // `:33486-33498` — the hp-tiered shot speed.
                let mut num906 = 6.0f32;
                if (s.life as f64) < (s.life_max as f64) * 0.75 {
                    num906 = 7.0;
                }
                if (s.life as f64) < (s.life_max as f64) * 0.5 {
                    num906 = 8.0;
                }
                if (s.life as f64) < (s.life_max as f64) * 0.25 {
                    num906 = 9.0;
                }
                num905 = num906 / num905;
                // `:33500` — Type 348 (FrostWave, `ProjectileID.cs:1250`),
                // Damage 42, KnockBack 0f, no RNG.
                shots.push(FrostBossShot {
                    kind: 348,
                    x: v_x,
                    y: v_y,
                    vx: num903 * num905,
                    vy: num904 * num905,
                    damage: 42,
                    ai0: 0.0,
                    ai1: 0.0,
                });
            }
        } else if s.ai[3] < 0.0 {
            // `:33503-33506`.
            s.ai[3] += 1.0;
        }
        // `:33507-33514` — the per-tick phase timer.
        s.ai[1] += rng.next_range(1, 4) as f32;
        if s.ai[1] > 800.0 && num898 < 600.0 {
            s.ai[0] = -1.0;
        }
    } else if s.ai[0] == 1.0 {
        // `:33516-33619` — phase 1: the hover + shard rain.
        // `:33519-33537` — the hp tiers, then the -0.05/-1 reductions.
        let (mut num908, mut num909) = (0.15f32, 7.0f32);
        if (s.life as f64) < (s.life_max as f64) * 0.75 {
            num908 = 0.17;
            num909 = 8.0;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.5 {
            num908 = 0.2;
            num909 = 9.0;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.25 {
            num908 = 0.25;
            num909 = 10.0;
        }
        num908 -= 0.05;
        num909 -= 1.0;
        // `:33538-33557` — the centre-seek with the counter-drift 0.98
        // and the over-cap 0.95.
        if cx < pcx {
            s.vx += num908;
            if s.vx < 0.0 {
                s.vx *= 0.98;
            }
        }
        if cx > pcx {
            s.vx -= num908;
            if s.vx > 0.0 {
                s.vx *= 0.98;
            }
        }
        if s.vx > num909 || s.vx < 0.0 - num909 {
            s.vx *= 0.95;
        }
        // `:33558-33574` — the hover band (180/200, ±0.1, cap 6).
        let num910 = target.y - (s.y + s.height);
        if num910 < 180.0 {
            s.vy -= 0.1;
        }
        if num910 > 200.0 {
            s.vy += 0.1;
        }
        if s.vy > 6.0 {
            s.vy = 6.0;
        }
        if s.vy < -6.0 {
            s.vy = -6.0;
        }
        // `:33575`.
        s.rotation = s.vx * 0.01;
        // `:33576-33610` — the shard-drop cadence.
        s.ai[3] += 1.0;
        let mut num911 = 15;
        if (s.life as f64) < (s.life_max as f64) * 0.75 {
            num911 = 14;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.5 {
            num911 = 12;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.25 {
            num911 = 10;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.1 {
            num911 = 8;
        }
        if s.ai[3] >= (num911 + 3) as f32 {
            s.ai[3] = 0.0;
            // `:33599-33600` — the drop point 14px above the bottom;
            // the C# truncates `(int)(x / 16f)` (NOT the >>4 shift).
            let v_x = cx;
            let v_y = s.y + s.height - 14.0;
            if !worldgen_solid_tile(world, (v_x / 16.0) as i32, (v_y / 16.0) as i32) {
                // `:33602-33607` — SpeedY floors the fall at 0 then +3;
                // ai0 = Next(5) draws ONLY here (the gate passed). The
                // named args carry no extra draws; SpeedX rides the
                // ALREADY-updated velocity.
                let mut num912 = s.vy;
                if num912 < 0.0 {
                    num912 = 0.0;
                }
                shots.push(FrostBossShot {
                    kind: 349,
                    x: v_x,
                    y: v_y,
                    vx: s.vx * 0.25,
                    vy: num912 + 3.0,
                    damage: 37,
                    ai0: rng.next_max(5) as f32,
                    ai1: 0.0,
                });
            }
        }
        // `:33611-33618` — the per-tick phase timer.
        s.ai[1] += rng.next_range(1, 4) as f32;
        if s.ai[1] > 600.0 {
            s.ai[0] = -1.0;
        }
    } else if s.ai[0] == 2.0 {
        // `:33620-33666` — phase 2: the spin barrage.
        // `:33623-33625` — the two draws fire EVERY tick, BEFORE the
        // velocity brake (`:33628`).
        let mut v_x = cx;
        let mut v_y = cy - 20.0;
        let mut num914 = rng.next_range(-1000, 1001) as f32;
        let mut num915 = rng.next_range(-1000, 1001) as f32;
        let mut num916 = (num914 * num914 + num915 * num915).sqrt();
        let num917 = 15.0f32;
        // `:33628` — both components brake.
        s.vx *= 0.95;
        s.vy *= 0.95;
        // `:33629-33634`.
        num916 = num917 / num916;
        num914 *= num916;
        num915 *= num916;
        s.rotation += 0.2;
        v_x += num914 * 4.0;
        v_y += num915 * 4.0;
        // `:33635-33652` — the hp-tiered cadence (7 minus 1/2/3/4).
        s.ai[3] += 1.0;
        let mut num918 = 7;
        if (s.life as f64) < (s.life_max as f64) * 0.75 {
            num918 -= 1;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.5 {
            num918 -= 2;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.25 {
            num918 -= 3;
        }
        if (s.life as f64) < (s.life_max as f64) * 0.1 {
            num918 -= 4;
        }
        // `:33653-33657` — Type 349 (FrostShard, `ProjectileID.cs:1252`),
        // Damage 35, no RNG in the shot.
        if s.ai[3] > num918 as f32 {
            s.ai[3] = 0.0;
            shots.push(FrostBossShot {
                kind: 349,
                x: v_x,
                y: v_y,
                vx: num914,
                vy: num915,
                damage: 35,
                ai0: 0.0,
                ai1: 0.0,
            });
        }
        // `:33658-33665` — the per-tick phase timer.
        s.ai[1] += rng.next_range(1, 4) as f32;
        if s.ai[1] > 500.0 {
            s.ai[0] = -1.0;
        }
    }
    // `:33667-33679` — the phase re-pick: a SIBLING if (runs the same
    // tick a phase set ai[0] = -1, and by day when ai[0] is already -1).
    if s.ai[0] == -1.0 {
        let mut num920 = rng.next_max(3);
        // `:33670` — TargetClosest() (the caller's seam).
        if (cx - pcx).abs() > 1000.0 {
            num920 = 0;
        }
        s.ai[0] = num920 as f32;
        s.ai[1] = 0.0;
        s.ai[2] = 0.0;
        s.ai[3] = 0.0;
    }
}

/// The dispatch seam — gated to **345 only** this round (Everscream 344
/// / SantaNK1 346 are the next slice-C round; the non-boss Frost set is
/// `npc_ai_frost_mob.rs`'s, worker D). Keyed on the vanilla type id; the
/// machine is the C# `aiStyle` 60 arm ([`ICE_QUEEN_AI_STYLE`],
/// `NPC.cs:13373`). Returns `false` for every other id. `rng` rides the
/// caller's `Main.rand` stream in the module-header order; the witness
/// tests pin it.
pub fn frost_boss_ai_step(
    type_id: u16,
    s: &mut FrostBossState,
    world: &World,
    target: &FrostBossTarget,
    rng: &mut UnifiedRandom,
    shots: &mut Vec<FrostBossShot>,
) -> bool {
    if type_id != ICE_QUEEN_TYPE {
        return false;
    }
    // The SetDefaults row pins aiStyle 60 for 345 (`NPC.cs:13373`) —
    // the arm `ice_queen_ai` ports (`:33374`); ICE_QUEEN_AI_STYLE is the
    // wiring worker's gate axis.
    ice_queen_ai(s, world, target, world.day_time, rng, shots);
    true
}

/// `NPC.TargetClosest` (`NPC.cs:77652-77781`) through the shared selection
/// (`npc_target.rs`, S1-D-3p) — one session player, the zombie/pumpkin
/// wiring's candidate shape (`npc_ai_pumpkin_boss.rs::target_closest`).
fn target_closest(npc: &mut Npc, world: &World, player: &Player, ai_style: i32) {
    let cand = crate::npc_target::TargetCandidate {
        active: true,
        dead: player.dead,
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    crate::npc_target::target_closest(npc, world, &[cand], true, ai_style, 0);
}

/// The live wiring seam (S1-F-13c) — the style axis (`npc.rs`) claims the
/// IceQueen (345, aiStyle 60) here. Copies the AI-visible `Npc` fields into
/// [`FrostBossState`], runs `TargetClosest` where the C# arm calls it
/// (`:33393` the initial side pick, `:33403` phase 0 per tick, `:33518`
/// phase 1, `:33622` phase 2, `:33670` the re-pick — all at night; the
/// single session player makes every call resolve to the same box, so one
/// pre-pass retarget covers them, `TargetClosest` draws no rand), runs the
/// dispatch ([`frost_boss_ai_step`]) on the caller's `UpdateNPCs` stream,
/// copies the state back, drains the shot requests into
/// `wants_hostile_shot(_ai)` and integrates the raw velocity
/// (`noTileCollide`/`noGravity` from SetDefaults, `:13381-13382` —
/// `NPC.cs:90495-90498`).
///
/// The aiStyle-60 day flee draws no `EncourageDespawn` (unlike 57/61) and
/// the arm has no dust — the stream is the shot/timer draws only.
pub fn ice_queen_step(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    let type_id = npc.kind.to_u16();
    let ai_style = npc.kind.ai_style() as i32;
    // The five `TargetClosest()` sites all run at night (the day flee,
    // `:33376-33388`, calls none). One pre-pass retarget covers them: the
    // port has one session player, so every call resolves to the same box
    // and `TargetClosest` draws no rand (the stream order is exact).
    if !world.day_time {
        target_closest(npc, world, player, ai_style);
    }
    let (w, h) = npc.size();
    let mut s = FrostBossState {
        x: npc.x,
        y: npc.y,
        vx: npc.vx,
        vy: npc.vy,
        ai: npc.ai,
        rotation: npc.rotation,
        life: npc.hp,
        life_max: npc.max_hp,
        width: w,
        height: h,
    };
    let target = FrostBossTarget {
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH,
        height: player.height,
    };
    let mut shots = Vec::new();
    frost_boss_ai_step(type_id, &mut s, world, &target, rng, &mut shots);
    // Copy the AI-visible writes back.
    npc.vx = s.vx;
    npc.vy = s.vy;
    npc.ai = s.ai;
    npc.rotation = s.rotation;
    // Drain the shot requests. The FrostWave (348) and the spin-barrage
    // shard (349) carry zero ai0/ai1 → the plain lane; the shard-rain drop
    // (349, `ai0 = Next(5)`, `:33607`) carries its payload on the ai lane.
    for shot in shots {
        if shot.ai0 == 0.0 && shot.ai1 == 0.0 {
            npc.wants_hostile_shot
                .push((shot.kind, shot.x, shot.y, shot.vx, shot.vy, shot.damage));
        } else {
            npc.wants_hostile_shot_ai.push(HostileShotAi {
                kind: shot.kind,
                x: shot.x,
                y: shot.y,
                vx: shot.vx,
                vy: shot.vy,
                damage: shot.damage,
                ai0: shot.ai0,
                ai1: shot.ai1,
                // The frost bosses roll no spawn-scale override — the
                // proj_data row's scale stands (S1-D-4 W-10 field).
                scale: None,
            });
        }
    }
    // `NPC.cs:90495-90498` — the noTileCollide integration.
    npc.x += npc.vx;
    npc.y += npc.vy;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        let mut w = World::new("t".into(), 200, 120);
        w.day_time = false;
        w
    }

    fn target(x: f32, y: f32) -> FrostBossTarget {
        FrostBossTarget {
            x,
            y,
            width: 20.0,
            height: 42.0,
        }
    }

    // ---- IceQueen (NPC.cs:33374-33680) ----

    /// `:33391-33413` — the side pick + the 800px turn-around.
    #[test]
    fn ice_queen_side_pick_and_turnaround() {
        let w = air();
        let mut rng = UnifiedRandom::new(1);
        let mut shots = Vec::new();
        // Player right → ai[2] = 1, accel 0.45 at full hp.
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        ice_queen_ai(&mut s, &w, &target(1100.0, 700.0), false, &mut rng, &mut shots);
        assert_eq!(s.ai[2], 1.0);
        assert_eq!(s.vx, 0.45);
        // Moving right while the player is >800px LEFT → ai[2] = 0.
        let mut s2 = FrostBossState::ice_queen(1000.0, 600.0);
        s2.ai[2] = 1.0;
        ice_queen_ai(&mut s2, &w, &target(100.0, 700.0), false, &mut rng, &mut shots);
        assert_eq!(s2.ai[2], 0.0);
        assert_eq!(s2.vx, 0.0);
    }

    /// `:33414-33439` — the hp-tiered accel/cap table.
    #[test]
    fn ice_queen_charge_speed_tiers() {
        let w = air();
        let t = target(1100.0, 700.0);
        for (frac, accel, cap) in [
            (1.0f32, 0.45f32, 7.0f32),
            (0.6, 0.55, 8.0),
            (0.4, 0.7, 10.0),
            (0.2, 0.8, 11.0),
        ] {
            let mut s = FrostBossState::ice_queen(1000.0, 600.0);
            s.life = (s.life_max as f32 * frac) as i32;
            s.ai[2] = 1.0;
            s.vx = 100.0; // over every cap
            let mut rng = UnifiedRandom::new(1);
            let mut shots = Vec::new();
            ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
            assert_eq!(s.vx, cap, "frac {frac}");
            s.vx = 0.0;
            ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
            assert_eq!(s.vx, accel, "frac {frac}");
        }
    }

    /// `:33458-33500` — the FrostWave fires when ai[3] wraps to 0, from
    /// Centre + velocity.X*7 (the UPDATED velocity), at the hp-tiered
    /// speed — NO RNG in the shot; the per-tick Next(1, 4) follows
    /// (`:33509`).
    #[test]
    fn ice_queen_frostwave_cadence_and_the_tick_draw() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        s.ai[2] = 1.0;
        s.ai[3] = -1.0; // wraps to 0 → fires
        let mut rng = UnifiedRandom::new(31);
        let mut probe = UnifiedRandom::new(31);
        let mut shots = Vec::new();
        ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
        assert_eq!(shots.len(), 1);
        let v_x = 1065.0f32 + 0.45 * 7.0;
        let (mut ax, mut ay) = (1110.0f32 - v_x, 721.0f32 - 670.0f32);
        let k = 6.0 / (ax * ax + ay * ay).sqrt();
        ax *= k;
        ay *= k;
        assert_eq!(
            shots[0],
            FrostBossShot { kind: 348, x: v_x, y: 670.0, vx: ax, vy: ay, damage: 42, ai0: 0.0, ai1: 0.0 }
        );
        assert_eq!(s.ai[1], probe.next_range(1, 4) as f32);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // `:33475-33478` — the wrap: ai[3] 14 → 15 > 14 → -14.
        s.ai[3] = 14.0;
        ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
        assert_eq!(s.ai[3], -14.0);
        assert_eq!(shots.len(), 1);
    }

    /// `:33507-33514` + `:33667-33679` — the phase flip AND the re-pick
    /// run on the SAME tick: Next(1, 4) then Next(3), ai[1..3] reset.
    #[test]
    fn ice_queen_phase_flip_repicks_on_the_same_tick() {
        let w = air();
        let t = target(1100.0, 700.0); // 45px away < 600
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        s.ai[2] = 1.0;
        s.ai[1] = 800.0;
        s.ai[3] = 5.0;
        let mut rng = UnifiedRandom::new(42);
        let mut probe = UnifiedRandom::new(42);
        let mut shots = Vec::new();
        ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
        let tick = probe.next_range(1, 4);
        assert!(800.0 + tick as f32 > 800.0);
        let pick = probe.next_max(3);
        assert_eq!(s.ai[0], pick as f32);
        assert_eq!((s.ai[1], s.ai[2], s.ai[3]), (0.0, 0.0, 0.0));
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // `:33671-33674` — beyond 1000px the pick is forced to 0 (the
        // Next(3) draw STILL fires first).
        let mut s2 = FrostBossState::ice_queen(1000.0, 600.0);
        s2.ai[0] = -1.0;
        ice_queen_ai(&mut s2, &w, &target(5000.0, 700.0), false, &mut rng, &mut shots);
        assert_eq!(s2.ai[0], 0.0);
    }

    /// `:33576-33618` — the shard drop: the cadence, the tile gate
    /// (a solid tile under the drop point suppresses the shot AND its
    /// Next(5) draw), the floored fall speed, ai0 = Next(5), then the
    /// per-tick Next(1, 4). The shot's SpeedX rides the ALREADY-updated
    /// velocity (`:33538-33557` runs before `:33607`).
    #[test]
    fn ice_queen_shard_drop_tile_gate_and_draw_order() {
        let mut w = air();
        let t = target(1100.0, 900.0);
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        s.ai[0] = 1.0;
        s.ai[3] = 17.0; // 18 >= 15+3 fires
        s.vy = -2.0;
        s.vx = 4.0;
        let mut rng = UnifiedRandom::new(99);
        let mut probe = UnifiedRandom::new(99);
        let mut shots = Vec::new();
        ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
        assert_eq!(shots.len(), 1);
        // The centre-seek ran first: cx 1065 < pcx 1110 → vx += 0.15-0.05
        // (full hp), under the 6f cap so no 0.95 brake.
        let seeked_vx = 4.0f32 + (0.15f32 - 0.05f32);
        let ai0 = probe.next_max(5);
        assert_eq!(
            shots[0],
            FrostBossShot { kind: 349, x: 1065.0, y: 726.0, vx: seeked_vx * 0.25, vy: 3.0, damage: 37, ai0: ai0 as f32, ai1: 0.0 }
        );
        assert_eq!(s.ai[3], 0.0);
        let tick = probe.next_range(1, 4);
        assert_eq!(s.ai[1], tick as f32);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // The gate: stone at (1065/16, 726/16) = (66, 45) suppresses
        // the drop — and the Next(5) with it.
        let tile = w.tile_mut(66, 45).unwrap();
        tile.kind = 1;
        tile.flags |= tr_core::tile::FLAG_ACTIVE;
        let mut s2 = FrostBossState::ice_queen(1000.0, 600.0);
        s2.ai[0] = 1.0;
        s2.ai[3] = 17.0;
        let mut rng2 = UnifiedRandom::new(99);
        let mut probe2 = UnifiedRandom::new(99);
        let mut shots2 = Vec::new();
        ice_queen_ai(&mut s2, &w, &t, false, &mut rng2, &mut shots2);
        assert!(shots2.is_empty());
        assert_eq!(s2.ai[3], 0.0);
        assert_eq!(s2.ai[1], probe2.next_range(1, 4) as f32);
        for _ in 0..8 {
            assert_eq!(rng2.next(), probe2.next());
        }
    }

    /// `:33620-33665` — the spin barrage: the two Next(-1000, 1001)
    /// draws EVERY tick (before the 0.95 brake), the 15f normalisation,
    /// the 4× offset origin, rotation += 0.2, the >7 cadence, then the
    /// per-tick Next(1, 4).
    #[test]
    fn ice_queen_spin_barrage_draw_order() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        s.ai[0] = 2.0;
        s.ai[3] = 7.0; // 8 > 7 fires
        s.vx = 2.0;
        s.vy = 1.0;
        let mut rng = UnifiedRandom::new(2024);
        let mut probe = UnifiedRandom::new(2024);
        let mut shots = Vec::new();
        ice_queen_ai(&mut s, &w, &t, false, &mut rng, &mut shots);
        let mut n1 = probe.next_range(-1000, 1001) as f32;
        let mut n2 = probe.next_range(-1000, 1001) as f32;
        let k = 15.0 / (n1 * n1 + n2 * n2).sqrt();
        n1 *= k;
        n2 *= k;
        assert_eq!(shots.len(), 1);
        assert_eq!(
            shots[0],
            FrostBossShot { kind: 349, x: 1065.0 + n1 * 4.0, y: 650.0 + n2 * 4.0, vx: n1, vy: n2, damage: 35, ai0: 0.0, ai1: 0.0 }
        );
        assert_eq!((s.vx, s.vy), (2.0 * 0.95, 1.0 * 0.95));
        assert_eq!(s.rotation, 0.2);
        assert_eq!(s.ai[3], 0.0);
        assert_eq!(s.ai[1], probe.next_range(1, 4) as f32);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // The hp cadence: at <10% hp num918 = 7-1-2-3-4 = -3, so EVERY
        // tick fires (ai[3] 1 > -3).
        let mut s2 = FrostBossState::ice_queen(1000.0, 600.0);
        s2.ai[0] = 2.0;
        s2.life = s2.life_max / 20;
        ice_queen_ai(&mut s2, &w, &t, false, &mut rng, &mut shots);
        assert_eq!(shots.len(), 2);
    }

    /// `:33376-33388` — the day flee: ±0.25 X accel, -0.1 Y, the 0.05
    /// tilt — NO EncourageDespawn and ZERO draws (ai[0] != -1).
    #[test]
    fn ice_queen_day_flee_draws_nothing() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        s.vx = 1.0;
        let mut rng = UnifiedRandom::new(8);
        let mut probe = UnifiedRandom::new(8);
        let mut shots = Vec::new();
        ice_queen_ai(&mut s, &w, &t, true, &mut rng, &mut shots);
        assert_eq!(s.vx, 1.25);
        assert_eq!(s.vy, -0.1);
        assert_eq!(s.rotation, 1.25 * 0.05);
        assert!(shots.is_empty());
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
        // The negative-X arm (`:33382-33385`).
        s.vx = -1.0;
        ice_queen_ai(&mut s, &w, &t, true, &mut rng, &mut shots);
        assert_eq!(s.vx, -1.25);
        // By day with ai[0] == -1 the re-pick STILL draws Next(3).
        s.ai[0] = -1.0;
        ice_queen_ai(&mut s, &w, &t, true, &mut rng, &mut shots);
        assert_eq!(s.ai[0], probe.next_max(3) as f32);
        for _ in 0..8 {
            assert_eq!(rng.next(), probe.next());
        }
    }

    // ---- the dispatch seam ----

    /// The seam claims exactly 345 this round; the other two bosses
    /// (344/346 — the next slice-C round), the non-boss Frost set
    /// (worker D) and every other id fall through.
    #[test]
    fn dispatch_claims_ice_queen_only() {
        let w = air();
        let t = target(1100.0, 700.0);
        let mut s = FrostBossState::ice_queen(1000.0, 600.0);
        let mut rng = UnifiedRandom::new(1);
        let mut shots = Vec::new();
        assert!(frost_boss_ai_step(345, &mut s, &w, &t, &mut rng, &mut shots));
        for id in [0u16, 3, 325, 327, 338, 341, 344, 346, 347, 352, 353, 670] {
            let mut s = FrostBossState::ice_queen(1000.0, 600.0);
            let mut rng = UnifiedRandom::new(1);
            let mut probe = UnifiedRandom::new(1);
            let mut shots = Vec::new();
            assert!(!frost_boss_ai_step(id, &mut s, &w, &t, &mut rng, &mut shots), "{id}");
            // A declined id draws nothing.
            assert_eq!(rng.next(), probe.next(), "{id}");
        }
    }

    /// The AI defaults match the SetDefaults row (`NPC.cs:13371-13376`)
    /// — worker B's agreement point — and the aiStyle constant matches
    /// `:13373`.
    #[test]
    fn the_defaults_match_set_defaults() {
        assert_eq!(ICE_QUEEN_TYPE, 345);
        assert_eq!(ICE_QUEEN_AI_STYLE, 60);
        assert_eq!(ice_queen_ai_defaults(), (130.0, 140.0, 34000));
        let s = FrostBossState::ice_queen(0.0, 0.0);
        assert_eq!((s.width, s.height, s.life, s.life_max), (130.0, 140.0, 34000, 34000));
        assert_eq!(s.center(), (65.0, 70.0));
    }
}
