//! The aiStyle-88 machine — the Mothron (`NPC.AI()`'s
//! `else if (this.aiStyle == 88)` arm, `NPC.cs:37827-38351`). W4-d32,
//! research/25 row 88 (MISSING → ported), CODE-ONLY MODE (no compiler
//! invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 88;` across the decompiled tree
//! returns exactly ONE assignment: `NPC.cs:15606`, the
//! `else if (this.type == 477)` row (`:15600-15615`) — the Mothron
//! (`ID/NPCID.cs:11627`). No dynamic `aiStyle` reassignment to 88
//! exists. The kind has no `NpcKind` yet — the module lands UNWIRED
//! (the style73/style86 precedent); the machine keys on the vanilla
//! `npc_type` until the roster wave.
//!
//! ## The machine
//!
//! The head (`:37829-37833`): `num1229 = 7` (the egg+spawn brood
//! cap), `noTileCollide = false`, `noGravity = true`, kb =
//! `GetKnockbackMultiplier_ScaledByDifficulty(0.2)` (`NPC.cs:7031`),
//! `damage = defDamage` (the 80 row).
//!
//! The eclipse gate (`:37834-37866`): no eclipse → state −1 (the
//! flee); a dead/inactive/absent target retargets, dead-or->3000 px
//! → −1; else >1000 px while `ai[0] > 1` demotes to the 1 chase.
//!
//! State −1 flee (`:37867-37873`): the (0,−8) ×9/10 steer,
//! noTileCollide + dontTakeDamage.
//!
//! State 0 hover (`:37874-37975`): retarget, the ±2 X direction
//! pick, the ×9/10 rotation ease, the collideX/Y bounce
//! (`velocity *= −oldVelocity × 0.5`, ±4 caps — the C#'s literal
//! product, NOT a plain reflect); the (player −200 Y) hover point:
//! >800 → state 1, >80 → the ×6 / 30-weight steer, else the
//! 0.95/1.05 speed band around 1-2. Then `ai[1]++`, the justHit
//! `Next(10,30)` acceleration, and at ≥180 the attack loop:
//! `while (ai[0] == 0)` rolls `Next(3)` — 0 needs LOS (else the loop
//! RE-ROLLS), 1 → the 3 swoop, 2 → the 4 egg run only when
//! `CountNPCS(478) + CountNPCS(479) < 7` (else re-roll).
//!
//! State 1 chase (`:37976-38010`): collide flags cleared,
//! noTileCollide + kb 0, the retarget guard, the vx direction, the
//! ×9/10 rotation (0.08), the <300 px + free-space return to 0, the
//! `7 + len/100` ×25-weight steer.
//!
//! State 2 ram (`:38011-38082`): damage = `(int)(defDamage × 0.5)`
//! (the f64 product), kb 0, the dead-target bail to 0, the ±10 X
//! direction pick, the ×4/5 rotation, the collide bounces, the
//! (player −20 Y) point, `ai[2] += 1/45` (+1/60 expert), the
//! `4 + ai[2] + len/120` ×20-weight steer; >240 ticks OR LOS-lost → 0.
//!
//! State 3 swoop setup (`:38083-38119`): kb 0 + noTileCollide, the
//! ×4/5 rotation (0.07), the (player −12 Y ±400 X) run-up point, the
//! >350 X + <20 Y alignment → 3.1, `ai[1] += 1/30`, the
//! `8 + ai[1]` ×4-weight steer.
//!
//! State 3.1 wind-up (`:38120-38158`): the ×16 / 8-weight steer at
//! the −12 Y point, >10 ticks → `velocity = vector179` verbatim,
//! state 3.2 with `ai[1] = direction` (the double write, verbatim).
//!
//! State 3.2 dash (`:38159-38188`): damage = `(int)(defDamage × 1.3)`,
//! collide flags cleared, `ai[2] += 1/30`,
//! `vx = (16 + ai[2]) × ai[1]`; past ±260 X: free-space → 0, else
//! >800 X → 1; the ×4/5 rotation.
//!
//! State 4 egg-site scan (`:38189-38258`): the above-surface flag,
//! retarget, ai[0..2] zeroed, then the 1000-iteration scan — TWO
//! draws per iteration (`Next(−num1244, num1244+1)` X with
//! `num1244 = 30 + i/50`, `Next(−num1245, num1245+1)` Y with
//! `num1245 = 20 + i/75`), the SolidTile skip (draws ALREADY
//! consumed), the ≤50-row lava/in-world/surface fall, the <600 px
//! accept → 4.1 with the site in ai[1]/ai[2].
//!
//! State 4.1 approach (`:38259-38293`): the ±2 X direction, the
//! ×9/10 rotation, noTileCollide, the site point
//! `(x·16+8, y·16−20)`, the `6 + len/150` cap-10 speed, <10 px →
//! 4.2, the SafeNormalize ×10-weight steer + the speed cap.
//!
//! State 4.2 lay (`:38294-38350`): the ×9/10 rotation, kb 0 +
//! noTileCollide, the ×4 / 2-weight steer + cap; within 4 px:
//! `ai[3]++`, at 70 (expert `(int)(70×0.75)` = 52) the egg 478
//! spawn request (`NewNPC` at `(x·16+8, y·16)`, `ai[3]`-paced), at
//! 2× → state 0 with the brood-cap `Next(3) != 0` re-run roll (else
//! the solid-collision → 1 bail).
//!
//! ## RNG census (by construction)
//!
//! 1. State 0's justHit `Next(10,30)` (`:37944`).
//! 2. State 0's attack loop `Next(3)` per iteration (`:37956`) —
//!    the LOS/brood gates can RE-ROLL (an unbounded loop).
//! 3. State 4's scan: exactly 2 draws per iteration (`:38204-38205`),
//!    consumed BEFORE the SolidTile continue.
//! 4. State 4.2's exit `Next(3)` (`:38329`) — only when the brood
//!    count is under the cap (the C# `&&` short-circuit).
//! The steers/TargetClosest/CanHit/SolidCollision are deterministic.
//!
//! ## Seams (the wiring obligations)
//!
//! * `Main.eclipse` rides [`Style88Env::eclipse`]; the brood count
//!   (`CountNPCS(478) + CountNPCS(479)`) rides
//!   [`Style88Env::brood_count`] — the sim supplies it from the
//!   pool (the style75 host-scan precedent).
//! * The egg spawn is a REQUEST ([`Style88Outcome::egg_spawn`]) —
//!   the sim's `NewNPC` drain owns the slot + `netUpdate`.
//! * The `damage` writes ride [`Style88Outcome::damage`] (the
//!   port's `damage_override` seam); `defDamage` is the 80 row.
//! * The gravity-off flight integrates raw (the caller's movement
//!   step, the style74 split); the collideX/Y bounces read the
//!   PREVIOUS tick's latches + `oldVelocity` (the port's
//!   `old_vx`/`old_vy`).
//!
//! See docs/ANALYSIS.md "The Mothron (W4-d32 aiStyle-88, code-only)".
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The type-477 `SetDefaults` row (`NPC.cs:15600-15615`), verbatim.
pub struct MothronDefaults {
    pub no_gravity: bool,
    pub net_always: bool,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit44` (`ID/SoundID.cs:1300`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath46` (`ID/SoundID.cs:1360`).
    pub death_sound: (i32, i32),
    pub value: f32,
    pub knock_back_resist: f32,
    pub rarity: i32,
}

/// `NPC.cs:15600-15615` — `else if (this.type == 477)`.
pub const MOTHRON: MothronDefaults = MothronDefaults {
    no_gravity: true,
    net_always: true,
    width: 80,
    height: 50,
    ai_style: 88,
    damage: 80,
    defense: 30,
    life_max: 6000,
    hit_sound: (3, 44),
    death_sound: (4, 46),
    value: 50000.0,
    knock_back_resist: 0.2,
    rarity: 1,
};

/// The per-tick environment (the `Style87Env` shape + the eclipse
/// and brood seams).
pub struct Style88Env<'a> {
    pub world: &'a World,
    pub players: &'a [TargetCandidate],
    pub ticker: u64,
    /// `this.difficulty` (`NPC.cs:7033`).
    pub difficulty: f32,
    /// `Main.expertMode`.
    pub expert: bool,
    /// `Main.eclipse` (`:37834`).
    pub eclipse: bool,
    /// `NPC.CountNPCS(478) + NPC.CountNPCS(479)` (`:37968`/`:38329`)
    /// — the sim's pool scan.
    pub brood_count: i32,
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style88Outcome {
    pub knock_back_resist: Option<f32>,
    pub no_tile_collide: bool,
    /// The head's `noGravity = true` (`:37831`) — every tick.
    pub no_gravity: bool,
    /// The −1 flee's `dontTakeDamage = true` (`:37872`) — the head
    /// does NOT reset it (verbatim: only the flee writes it).
    pub dont_take_damage: Option<bool>,
    /// The head's `damage = defDamage` (`:37833`) and the state-2/3.2
    /// scalings — the port's `damage_override` seam.
    pub damage: i32,
    /// The state-4.2 egg spawn request: the tile site (the C#
    /// `NewNPC(..., x*16+8, y*16, 478, whoAmI)`, `:38319`).
    pub egg_spawn: Option<(i32, i32)>,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the
/// style74 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED (the style74 precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `Utils.SafeNormalize` (`Utils.cs:1837-1844`) — the style74 shape.
fn safe_normalize(x: f32, y: f32, dx: f32, dy: f32) -> (f32, f32) {
    if (x == 0.0 && y == 0.0) || x.is_nan() || y.is_nan() {
        (dx, dy)
    } else {
        normalize_xna(x, y)
    }
}

/// `NPC.GetKnockbackMultiplier_ScaledByDifficulty` (`NPC.cs:7031`)
/// over the Classic/Master curve (the style87 seam).
fn knockback_scaled(normal: f32, difficulty: f32) -> f32 {
    let m = if difficulty <= 1.0 {
        1.0
    } else if difficulty >= 3.0 {
        0.8
    } else {
        (difficulty - 1.0) * (0.8 - 1.0) / (3.0 - 1.0) + 1.0
    };
    normal * m
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

/// `Collision.CanHit(Center,1,1,player.Center,1,1)` — the ported LOS.
fn can_hit_player(npc: &Npc, env: &Style88Env<'_>, p: &TargetCandidate) -> bool {
    let (cx, cy) = npc.center();
    let (px, py) = candidate_center(p);
    crate::projectile::can_hit(env.world, cx, cy, 1.0, 1.0, px, py, 1.0, 1.0)
}

fn solid_collision(npc: &Npc, world: &World) -> bool {
    let (w, h) = npc.size();
    crate::player::Player::solid_collision(world, npc.x, npc.y, w, h)
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`): null →
/// true (the out-of-world catch), else the full-solid predicate
/// (active, solid, not solid-top, not half, no slope, not actuated).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        None => true,
        Some(t) => {
            t.active()
                && tr_core::tiledata::TILE_SOLID.has(t.kind)
                && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                && !t.half_brick()
                && t.slope() == 0
                && t.flags & tr_core::tile::FLAG_INACTIVE == 0
        }
    }
}

/// `WorldGen.InWorld(x, y, 5)` (`WorldGen.cs:9688-9695`).
fn in_world5(world: &World, x: i32, y: i32) -> bool {
    x >= 5 && x < world.width - 5 && y >= 5 && y < world.height - 5
}

/// The collideX/Y bounce (`:37887-37910` and the state-2 twin):
/// `velocity *= (0 − oldVelocity) × 0.5`, the ±4 caps — the C#'s
/// literal product (a velocity-squared reflect, NOT a plain flip).
fn bounce(v: f32, old_v: f32) -> f32 {
    let mut r = v * (0.0 - old_v) * 0.5;
    if r > 4.0 {
        r = 4.0;
    }
    if r < -4.0 {
        r = -4.0;
    }
    r
}

/// One fixed step of the aiStyle-88 arm (`NPC.cs:37827-38351`), in
/// C# order. `npc` is the Mothron (type 477 — the census's only
/// kind). The caller owns the raw flight integration and drains the
/// requests.
pub fn style88_step(npc: &mut Npc, env: &mut Style88Env<'_>) -> Style88Outcome {
    let num1229 = 7;
    let def_damage = MOTHRON.damage;
    let mut out = Style88Outcome {
        no_gravity: true,
        damage: def_damage,
        ..Default::default()
    };
    // `:37830-37833` — the head.
    out.no_tile_collide = false;
    out.knock_back_resist = Some(knockback_scaled(0.2, env.difficulty));
    // `:37834-37866` — the eclipse/target gates (netMode != 1 folds
    // true, the solo session).
    if !env.eclipse {
        if npc.ai[0] != -1.0 {
            npc.net_update = true;
        }
        npc.ai[0] = -1.0;
    } else {
        let p = target_of(npc, env.players);
        // The C# indexes `Main.player[this.target]` — an unset 255
        // slot is INACTIVE (the range check stands in for it).
        if npc.target < 0 || npc.target as usize >= env.players.len() || p.dead || !p.active {
            target_closest(npc, env.world, env.players, true, 88, env.ticker);
            let p = target_of(npc, env.players);
            let (pcx, pcy) = candidate_center(p);
            let (cx, cy) = npc.center();
            if p.dead || sqrt_f32(pcx - cx, pcy - cy) > 3000.0 {
                if npc.ai[0] != -1.0 {
                    npc.net_update = true;
                }
                npc.ai[0] = -1.0;
            }
        } else {
            let (pcx, pcy) = candidate_center(p);
            let (cx, cy) = npc.center();
            if npc.ai[0] > 1.0 && sqrt_f32(pcx - cx, pcy - cy) > 1000.0 {
                if npc.ai[0] != 1.0 {
                    npc.net_update = true;
                }
                npc.ai[0] = 1.0;
            }
        }
    }
    if npc.ai[0] == -1.0 {
        // `:37867-37873` — the flee.
        npc.vx = (npc.vx * 9.0 + 0.0) / 10.0;
        npc.vy = (npc.vy * 9.0 + -8.0) / 10.0;
        out.no_tile_collide = true;
        npc.dont_take_damage = true;
        out.dont_take_damage = Some(true);
    } else if npc.ai[0] == 0.0 {
        // `:37874-37975` — the hover.
        target_closest(npc, env.world, env.players, true, 88, env.ticker);
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        if cx < pcx - 2.0 {
            npc.facing = 1;
        }
        if cx > pcx + 2.0 {
            npc.facing = -1;
        }
        npc.sprite_dir = npc.facing;
        npc.rotation = (npc.rotation * 9.0 + npc.vx * 0.1) / 10.0;
        if npc.collide_x {
            npc.vx = bounce(npc.vx, npc.old_vx);
        }
        if npc.collide_y {
            npc.vy = bounce(npc.vy, npc.old_vy);
        }
        let mut dx = pcx - cx;
        let mut dy = pcy - cy;
        dy -= 200.0;
        let len = sqrt_f32(dx, dy);
        if len > 800.0 {
            npc.ai[0] = 1.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        } else if len > 80.0 {
            let (nx, ny) = normalize_xna(dx, dy);
            dx = nx * 6.0;
            dy = ny * 6.0;
            npc.vx = (npc.vx * (30.0 - 1.0) + dx) / 30.0;
            npc.vy = (npc.vy * (30.0 - 1.0) + dy) / 30.0;
        } else {
            let vlen = sqrt_f32(npc.vx, npc.vy);
            if vlen > 2.0 {
                npc.vx *= 0.95;
                npc.vy *= 0.95;
            } else if vlen < 1.0 {
                npc.vx *= 1.05;
                npc.vy *= 1.05;
            }
        }
        // `:37941-37974` — the attack timer + the re-roll loop.
        npc.ai[1] += 1.0;
        if npc.just_hit {
            npc.ai[1] += env.rng.next_range(10, 30) as f32;
        }
        if npc.ai[1] >= 180.0 {
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
            while npc.ai[0] == 0.0 {
                let num1232 = env.rng.next_max(3);
                let p = target_of(npc, env.players);
                if num1232 == 0 && can_hit_player(npc, env, p) {
                    npc.ai[0] = 2.0;
                    continue;
                }
                match num1232 {
                    1 => npc.ai[0] = 3.0,
                    2 => {
                        if env.brood_count < num1229 {
                            npc.ai[0] = 4.0;
                        }
                    }
                    _ => {}
                }
            }
        }
    } else if npc.ai[0] == 1.0 {
        // `:37976-38010` — the chase.
        npc.collide_x = false;
        npc.collide_y = false;
        out.no_tile_collide = true;
        out.knock_back_resist = Some(0.0);
        {
            let p = target_of(npc, env.players);
            if npc.target < 0 || npc.target as usize >= env.players.len() || !p.active || p.dead {
                target_closest(npc, env.world, env.players, true, 88, env.ticker);
            }
        }
        if npc.vx < 0.0 {
            npc.facing = -1;
        } else if npc.vx > 0.0 {
            npc.facing = 1;
        }
        npc.sprite_dir = npc.facing;
        npc.rotation = (npc.rotation * 9.0 + npc.vx * 0.08) / 10.0;
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let len = sqrt_f32(dx, dy);
        if len < 300.0 && !solid_collision(npc, env.world) {
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        let num1233 = 7.0 + len / 100.0;
        let (nx, ny) = normalize_xna(dx, dy);
        npc.vx = (npc.vx * (25.0 - 1.0) + nx * num1233) / 25.0;
        npc.vy = (npc.vy * (25.0 - 1.0) + ny * num1233) / 25.0;
    } else if npc.ai[0] == 2.0 {
        // `:38011-38082` — the ram.
        out.damage = (def_damage as f64 * 0.5) as i32;
        out.knock_back_resist = Some(0.0);
        {
            let p = target_of(npc, env.players);
            if npc.target < 0 || npc.target as usize >= env.players.len() || !p.active || p.dead {
                target_closest(npc, env.world, env.players, true, 88, env.ticker);
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        if pcx - 10.0 < cx {
            npc.facing = -1;
        } else if pcx + 10.0 > cx {
            npc.facing = 1;
        }
        npc.sprite_dir = npc.facing;
        npc.rotation = (npc.rotation * 4.0 + npc.vx * 0.1) / 5.0;
        if npc.collide_x {
            npc.vx = bounce(npc.vx, npc.old_vx);
        }
        if npc.collide_y {
            npc.vy = bounce(npc.vy, npc.old_vy);
        }
        let dx = pcx - cx;
        let mut dy = pcy - cy;
        dy -= 20.0;
        npc.ai[2] += 1.0 / 45.0;
        if env.expert {
            npc.ai[2] += 1.0 / 60.0;
        }
        let len = sqrt_f32(dx, dy);
        let num1235 = 4.0 + npc.ai[2] + len / 120.0;
        let (nx, ny) = normalize_xna(dx, dy);
        npc.vx = (npc.vx * (20.0 - 1.0) + nx * num1235) / 20.0;
        npc.vy = (npc.vy * (20.0 - 1.0) + ny * num1235) / 20.0;
        npc.ai[1] += 1.0;
        if npc.ai[1] > 240.0 || !can_hit_player(npc, env, p) {
            npc.ai[0] = 0.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 3.0 {
        // `:38083-38119` — the swoop setup.
        out.knock_back_resist = Some(0.0);
        out.no_tile_collide = true;
        npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
        npc.sprite_dir = npc.facing;
        npc.rotation = (npc.rotation * 4.0 + npc.vx * 0.07) / 5.0;
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let mut dx = pcx - cx;
        let mut dy = pcy - cy;
        dy -= 12.0;
        if cx > pcx {
            dx += 400.0;
        } else {
            dx -= 400.0;
        }
        if (cx - pcx).abs() > 350.0 && (cy - pcy).abs() < 20.0 {
            npc.ai[0] = 3.1;
            npc.ai[1] = 0.0;
            npc.net_update = true;
        }
        npc.ai[1] += 1.0 / 30.0;
        let num1237 = 8.0 + npc.ai[1];
        let (nx, ny) = normalize_xna(dx, dy);
        npc.vx = (npc.vx * (4.0 - 1.0) + nx * num1237) / 4.0;
        npc.vy = (npc.vy * (4.0 - 1.0) + ny * num1237) / 4.0;
    } else if npc.ai[0] == 3.1 {
        // `:38120-38158` — the wind-up.
        out.knock_back_resist = Some(0.0);
        out.no_tile_collide = true;
        npc.rotation = (npc.rotation * 4.0 + npc.vx * 0.07) / 5.0;
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let dx = pcx - cx;
        let dy = pcy - cy - 12.0;
        let (nx, ny) = normalize_xna(dx, dy);
        let (tx, ty) = (nx * 16.0, ny * 16.0);
        npc.vx = (npc.vx * (8.0 - 1.0) + tx) / 8.0;
        npc.vy = (npc.vy * (8.0 - 1.0) + ty) / 8.0;
        npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
        npc.sprite_dir = npc.facing;
        npc.ai[1] += 1.0;
        if npc.ai[1] > 10.0 {
            npc.vx = tx;
            npc.vy = ty;
            npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
            npc.ai[0] = 3.2;
            npc.ai[1] = 0.0;
            npc.ai[1] = npc.facing as f32; // the double write, verbatim
            npc.net_update = true;
        }
    } else if npc.ai[0] == 3.2 {
        // `:38159-38188` — the dash.
        out.damage = (def_damage as f64 * 1.3) as i32;
        npc.collide_x = false;
        npc.collide_y = false;
        out.knock_back_resist = Some(0.0);
        out.no_tile_collide = true;
        npc.ai[2] += 1.0 / 30.0;
        npc.vx = (16.0 + npc.ai[2]) * npc.ai[1];
        let p = target_of(npc, env.players);
        let (pcx, _) = candidate_center(p);
        let (cx, _) = npc.center();
        if (npc.ai[1] > 0.0 && cx > pcx + 260.0) || (npc.ai[1] < 0.0 && cx < pcx - 260.0) {
            if !solid_collision(npc, env.world) {
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            } else if (cx - pcx).abs() > 800.0 {
                npc.ai[0] = 1.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        npc.rotation = (npc.rotation * 4.0 + npc.vx * 0.07) / 5.0;
    } else if npc.ai[0] == 4.0 {
        // `:38189-38258` — the egg-site scan.
        let (_, cy) = npc.center();
        let flag96 = ((cy / 16.0) as f64) < env.world.world_surface;
        target_closest(npc, env.world, env.players, true, 88, env.ticker);
        npc.ai[0] = 0.0;
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        for num1241 in 0..1000 {
            let mut num1242 = pcx as i32 / 16;
            let mut num1243 = pcy as i32 / 16;
            let num1244 = 30 + num1241 / 50;
            let num1245 = 20 + num1241 / 75;
            num1242 += env.rng.next_range(-num1244, num1244 + 1);
            num1243 += env.rng.next_range(-num1245, num1245 + 1);
            if solid_tile(env.world, num1242, num1243) {
                continue;
            }
            let mut flag97 = false;
            let mut num1246 = 50;
            while num1246 > 0 {
                num1246 -= 1;
                if !in_world5(env.world, num1242, num1243) {
                    flag97 = true;
                    break;
                }
                let Some(tile2) = env.world.tile(num1242, num1243) else {
                    flag97 = true;
                    break;
                };
                if tile2.liquid > 0 && tile2.flags & tr_core::tile::FLAG_LIQUID_LAVA != 0 {
                    flag97 = true;
                    break;
                }
                let Some(tile3) = env.world.tile(num1242, num1243 - 1) else {
                    flag97 = true;
                    break;
                };
                if tile3.liquid > 0 && tile3.flags & tr_core::tile::FLAG_LIQUID_LAVA != 0 {
                    flag97 = true;
                    break;
                }
                if solid_tile(env.world, num1242, num1243)
                    || (flag96 && num1243 as f64 > env.world.world_surface)
                {
                    break;
                }
                num1243 += 1;
            }
            // `:38248` — the C#'s non-short-circuit `|` (both sides
            // already evaluated; no draws inside, so `||` is
            // behaviour-identical).
            if !(num1246 <= 0 || flag97) {
                let sx = num1242 as f32 * 16.0 + 8.0;
                let sy = num1243 as f32 * 16.0 + 8.0;
                if sqrt_f32(sx - pcx, sy - pcy) < 600.0 {
                    npc.ai[0] = 4.1;
                    npc.ai[1] = num1242 as f32;
                    npc.ai[2] = num1243 as f32;
                    break;
                }
            }
        }
        npc.net_update = true;
    } else if npc.ai[0] == 4.1 {
        // `:38259-38293` — the approach.
        if npc.vx < -2.0 {
            npc.facing = -1;
        } else if npc.vx > 2.0 {
            npc.facing = 1;
        }
        npc.sprite_dir = npc.facing;
        npc.rotation = (npc.rotation * 9.0 + npc.vx * 0.1) / 10.0;
        out.no_tile_collide = true;
        let (cx, cy) = npc.center();
        let mut dx = npc.ai[1] as i32 as f32 * 16.0 + 8.0 - cx;
        let mut dy = npc.ai[2] as i32 as f32 * 16.0 - 20.0 - cy;
        let len = sqrt_f32(dx, dy);
        let mut num1247 = 6.0 + len / 150.0;
        if num1247 > 10.0 {
            num1247 = 10.0;
        }
        if len < 10.0 {
            npc.ai[0] = 4.2;
            npc.net_update = true;
        }
        let (nx, ny) = safe_normalize(dx, dy, 0.0, 0.0);
        dx = nx * num1247;
        dy = ny * num1247;
        npc.vx = (npc.vx * (10.0 - 1.0) + dx) / 10.0;
        npc.vy = (npc.vy * (10.0 - 1.0) + dy) / 10.0;
        let vlen = sqrt_f32(npc.vx, npc.vy);
        if vlen > num1247 {
            let (nx, ny) = normalize_xna(npc.vx, npc.vy);
            npc.vx = nx * num1247;
            npc.vy = ny * num1247;
        }
    } else if npc.ai[0] == 4.2 {
        // `:38294-38350` — the lay.
        npc.rotation = (npc.rotation * 9.0 + npc.vx * 0.1) / 10.0;
        out.knock_back_resist = Some(0.0);
        out.no_tile_collide = true;
        let num1249 = npc.ai[1] as i32;
        let num1250 = npc.ai[2] as i32;
        let (cx, cy) = npc.center();
        let mut dx = num1249 as f32 * 16.0 + 8.0 - cx;
        let mut dy = num1250 as f32 * 16.0 - 20.0 - cy;
        let num1251 = 4.0f32;
        let len = sqrt_f32(dx, dy);
        if len < 4.0 {
            let mut num1253 = 70;
            if env.expert {
                num1253 = (num1253 as f64 * 0.75) as i32;
            }
            npc.ai[3] += 1.0;
            if npc.ai[3] == num1253 as f32 {
                // `:38319-38320` — the egg 478 spawn request.
                out.egg_spawn = Some((num1249 * 16 + 8, num1250 * 16));
            } else if npc.ai[3] == (num1253 * 2) as f32 {
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
                if env.brood_count < num1229 && env.rng.next_max(3) != 0 {
                    npc.ai[0] = 4.0;
                } else if solid_collision(npc, env.world) {
                    npc.ai[0] = 1.0;
                }
            }
        }
        if len > num1251 {
            let (nx, ny) = normalize_xna(dx, dy);
            dx = nx * num1251;
            dy = ny * num1251;
        }
        npc.vx = (npc.vx * (2.0 - 1.0) + dx) / 2.0;
        npc.vy = (npc.vy * (2.0 - 1.0) + dy) / 2.0;
        let vlen = sqrt_f32(npc.vx, npc.vy);
        if vlen > num1251 {
            let (nx, ny) = normalize_xna(npc.vx, npc.vy);
            npc.vx = nx * num1251;
            npc.vy = ny * num1251;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 477-shaped NPC: no kind yet — the placeholder + the C# box
    /// (the style86 convention).
    fn mothron(x: f32, y: f32) -> Npc {
        let d = &MOTHRON;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
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

    fn env<'a>(
        w: &'a World,
        players: &'a [TargetCandidate],
        rng: &'a mut UnifiedRandom,
    ) -> Style88Env<'a> {
        Style88Env {
            world: w,
            players,
            ticker: 0,
            difficulty: 1.0,
            expert: false,
            eclipse: true,
            brood_count: 0,
            rng,
        }
    }

    /// `NPC.cs:15600-15615` — the SetDefaults row, field by field.
    #[test]
    fn w4_d32_s88_the_defaults_row() {
        let d = &MOTHRON;
        assert!(d.no_gravity);
        assert!(d.net_always);
        assert_eq!((d.width, d.height), (80, 50));
        assert_eq!(d.ai_style, 88);
        assert_eq!((d.damage, d.defense, d.life_max), (80, 30, 6000));
        assert_eq!(d.hit_sound, (3, 44), "SoundID.NPCHit44");
        assert_eq!(d.death_sound, (4, 46), "SoundID.NPCDeath46");
        assert_eq!(d.value, 50000.0);
        assert_eq!(d.knock_back_resist, 0.2);
        assert_eq!(d.rarity, 1);
    }

    /// `:37834-37841` + `:37867-37873` — no eclipse: state −1, the
    /// (0,−8) ×9/10 flee steer, noTileCollide + dontTakeDamage.
    #[test]
    fn w4_d32_s88_the_no_eclipse_flee() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mothron(100.0, 100.0);
        n.vx = 10.0;
        n.vy = 2.0;
        let mut e = env(&w, &players, &mut rng);
        e.eclipse = false;
        let out = style88_step(&mut n, &mut e);
        assert_eq!(n.ai[0], -1.0);
        assert!(n.net_update);
        assert_eq!(n.vx, 9.0);
        assert_eq!(n.vy, (2.0 * 9.0 - 8.0) / 10.0);
        assert!(out.no_tile_collide);
        assert!(n.dont_take_damage);
        assert!(out.no_gravity);
        assert_eq!(out.damage, 80, "the head restores defDamage");
    }

    /// `:37887-37910` — the collide bounce is the LITERAL product
    /// `v × (−oldV) × 0.5` with the ±4 caps (not a plain reflect).
    #[test]
    fn w4_d32_s88_the_collide_bounce_is_the_literal_product() {
        let w = air();
        // The hover point (150, 121) is ~14 px from the centre
        // (140, 125) — inside the ≤80 band, so the 0.95 damp runs.
        let players = [cand(140.0, 300.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mothron(100.0, 100.0);
        n.collide_x = true;
        n.vx = 2.0;
        n.old_vx = 3.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style88_step(&mut n, &mut e);
        // 2 × (−3) × 0.5 = −3, inside the caps; then the hover band
        // (vlen 3 > 2) scales ×0.95.
        assert_eq!(n.vx, -3.0 * 0.95);
        // The cap: 10 × (−10) × 0.5 = −50 → −4.
        assert_eq!(bounce(10.0, 10.0), -4.0);
        assert_eq!(bounce(-10.0, -10.0), -4.0);
        assert_eq!(bounce(2.0, -3.0), 3.0);
    }

    /// `:37941-37974` — the justHit acceleration draws Next(10,30);
    /// the ≥180 attack loop re-rolls until a gate passes. In the air
    /// world LOS is clear, so roll 0 accepts; the oracle pins the
    /// stream.
    #[test]
    fn w4_d32_s88_the_attack_loop_and_the_justhit_draw() {
        let w = air();
        let players = [cand(140.0, 300.0)];
        let mut rng = UnifiedRandom::new(88);
        let mut oracle = UnifiedRandom::new(88);
        let mut n = mothron(100.0, 100.0);
        n.just_hit = true;
        n.ai[1] = 179.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style88_step(&mut n, &mut e);
        let accel = oracle.next_range(10, 30);
        assert!(accel >= 10 && accel < 30);
        // 179 + 1 + accel >= 180 → the loop runs; every roll accepts
        // (LOS clear, brood 0 < 7) → exactly one Next(3).
        oracle.next_max(3);
        assert_eq!(rng.peek(), oracle.peek(), "the draw order");
        assert!(n.ai[0] == 2.0 || n.ai[0] == 3.0 || n.ai[0] == 4.0);
    }

    /// `:37968-37972` — the brood cap: at 7 the roll-2 arm re-rolls
    /// (the loop keeps drawing until 0-with-LOS or 1).
    #[test]
    fn w4_d32_s88_the_brood_cap_rerolls_the_egg_arm() {
        let w = air();
        let players = [cand(140.0, 300.0)];
        let mut rng = UnifiedRandom::new(88);
        let mut oracle = UnifiedRandom::new(88);
        let mut n = mothron(100.0, 100.0);
        n.ai[1] = 180.0;
        let mut e = env(&w, &players, &mut rng);
        e.brood_count = 7;
        let _ = style88_step(&mut n, &mut e);
        loop {
            let r = oracle.next_max(3);
            if r == 0 || r == 1 {
                break; // LOS clear / the swoop always accepts
            }
        }
        assert_eq!(rng.peek(), oracle.peek(), "the re-roll stream");
        assert!(n.ai[0] == 2.0 || n.ai[0] == 3.0, "never the egg run at cap");
    }

    /// `:38013`/`:38161` — the damage scalings: (int)(80×0.5) = 40,
    /// (int)(80×1.3) = 104 (the f64 product truncation).
    #[test]
    fn w4_d32_s88_the_ram_and_dash_damage_scalings() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mothron(100.0, 100.0);
        n.ai[0] = 2.0;
        let mut e = env(&w, &players, &mut rng);
        let out = style88_step(&mut n, &mut e);
        assert_eq!(out.damage, 40);
        n.ai[0] = 3.2;
        n.ai[1] = 1.0;
        n.ai[2] = 0.0; // the 3.2 ramp starts from a clean counter
        let out = style88_step(&mut n, &mut e);
        assert_eq!(out.damage, 104);
        assert_eq!(n.vx, (16.0 + 1.0 / 30.0) * 1.0);
    }

    /// `:38141-38157` — the wind-up exit: velocity = the ×16 steer
    /// verbatim, ai[1] = direction (the double write).
    #[test]
    fn w4_d32_s88_the_windup_exit_latches_the_direction() {
        let w = air();
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mothron(100.0, 100.0);
        n.ai[0] = 3.1;
        n.ai[1] = 10.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style88_step(&mut n, &mut e);
        assert_eq!(n.ai[0], 3.2);
        assert_eq!(n.ai[1], 1.0, "the player is right → +1");
        // The velocity is EXACTLY the normalized ×16 aim.
        let (cx, cy) = n.center();
        let dx = 1010.0 - cx;
        let dy = 121.0 - cy - 12.0;
        let (nx, ny) = normalize_xna(dx, dy);
        assert_eq!(n.vx, nx * 16.0);
        assert_eq!(n.vy, ny * 16.0);
    }

    /// `:38198-38255` — the scan: the Mothron BELOW the surface
    /// (flag96 false) in the EMPTY air world — every sampled tile is
    /// non-solid, the fall runs its full 50 rows and `num1246 <= 0`
    /// rejects — all 1000 iterations × 2 draws fire.
    #[test]
    fn w4_d32_s88_the_empty_world_scan_draws_two_thousand() {
        let w = air();
        let players = [cand(3000.0, 1000.0)];
        let mut rng = UnifiedRandom::new(477);
        let mut oracle = UnifiedRandom::new(477);
        // cy = 1325 → 82.8 >= worldSurface 75 → flag96 false.
        let mut n = mothron(2900.0, 1300.0);
        n.ai[0] = 4.0;
        let mut e = env(&w, &players, &mut rng);
        let _ = style88_step(&mut n, &mut e);
        for i in 0..1000 {
            let a = 30 + i / 50;
            let b = 20 + i / 75;
            oracle.next_range(-a, a + 1);
            oracle.next_range(-b, b + 1);
        }
        assert_eq!(rng.peek(), oracle.peek(), "2 draws per iteration");
        assert_eq!(n.ai[0], 0.0, "no site found in the empty world");
        assert!(n.net_update);
    }

    /// `:38309-38337` — the lay: ai[3] 70 → the egg request; 140 →
    /// the exit with the brood-gated Next(3); expert shortens to
    /// 52/104 ((int)(70×0.75)).
    #[test]
    fn w4_d32_s88_the_lay_paces_the_egg_and_the_exit_roll() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut oracle = UnifiedRandom::new(7);
        // The centre (137, 125) sits √2 px from the site point
        // (136, 124) — inside the 4-px lay gate.
        let mut n = mothron(97.0, 100.0);
        n.ai[0] = 4.2;
        n.ai[1] = 8.0; // site x=8 → 136
        n.ai[2] = 9.0; // site y=9 → 124
        n.ai[3] = 69.0;
        let mut e = env(&w, &players, &mut rng);
        let out = style88_step(&mut n, &mut e);
        assert_eq!(out.egg_spawn, Some((8 * 16 + 8, 9 * 16)));
        assert_eq!(e.rng.peek(), oracle.peek(), "the lay tick draws nothing");
        n.ai[3] = 139.0;
        let _ = style88_step(&mut n, &mut e);
        let roll = oracle.next_max(3);
        assert_eq!(e.rng.peek(), oracle.peek(), "the exit roll");
        if roll != 0 {
            assert_eq!(n.ai[0], 4.0, "the re-run");
        } else {
            assert_eq!(n.ai[0], 0.0, "air world: no solid bail");
        }
        // Expert: (int)(70 × 0.75) = 52. The exit zeroed ai[1..3] —
        // re-seat the site for the expert leg.
        n.ai[0] = 4.2;
        n.ai[1] = 8.0;
        n.ai[2] = 9.0;
        n.ai[3] = 51.0;
        e.expert = true;
        let out = style88_step(&mut n, &mut e);
        assert_eq!(out.egg_spawn, Some((136, 144)), "the expert 52-tick egg");
    }

    /// The steers' zero-draw census: the chase state away from the
    /// timers consumes NO `Main.rand`.
    #[test]
    fn w4_d32_s88_the_steer_states_draw_nothing() {
        let w = air();
        let players = [cand(900.0, 300.0)];
        let mut rng = UnifiedRandom::new(478);
        let mut n = mothron(100.0, 100.0);
        n.ai[0] = 1.0;
        let mut e = env(&w, &players, &mut rng);
        for _ in 0..100 {
            let _ = style88_step(&mut n, &mut e);
            n.ai[0] = 1.0; // hold the chase (the <300 return fires later)
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(478).peek(), "no draws");
    }
}
