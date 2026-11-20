//! The aiStyle-87 family — the Big Mimics (`NPC.AI()`'s
//! `else if (this.aiStyle == 87)` arm, `NPC.cs:37471-37826`). W4-d32,
//! research/25 row 87 (MISSING → ported), CODE-ONLY MODE (no compiler
//! invoked; the witness tests below are written, not run).
//!
//! ## Census
//!
//! The census of `this.aiStyle = 87;` across the decompiled tree
//! returns exactly ONE assignment: `NPC.cs:14512`, the grouped
//! `else if (this.type == 473 || 474 || 475 || 476)` row
//! (`:14508-14521`) — the BigMimicCorruption 473 / BigMimicCrimson 474
//! / BigMimicHallow 475 / BigMimicJungle 476 (`ID/NPCID.cs:11619-11625`).
//! No dynamic `aiStyle` reassignment to 87 exists. None of the four
//! kinds has an `NpcKind` yet — the module lands UNWIRED (the
//! style73/style86 precedent); the machine keys on the vanilla
//! `npc_type` until the roster wave.
//!
//! ## The machine
//!
//! The head (`:37473-37477`): kb = `GetKnockbackMultiplier_ScaledByDifficulty(0.2)`
//! (`NPC.cs:7031-7034` — the style74 difficulty seam),
//! `dontTakeDamage = false`, `noTileCollide = false`,
//! `noGravity = false`, `reflectsProjectiles = false` — the flags ride
//! [`Style87Outcome`] (the style50 integration split).
//!
//! The dead-target bail (`:37478-37489`): `ai[0] != 7` + a dead
//! target retargets; still dead → state 7, ai[1..3] zeroed, netUpdate.
//!
//! State 0 dormant (`:37490-37500`): retarget, then ANY of
//! (vx != 0 | vy > 100 | justHit | player within 80 px) wakes to
//! state 1. State 1 (`:37501-37510`): the 36-tick awakening timer.
//!
//! State 2 hop (`:37511-37591`): grounded — vx ×0.85, the
//! life-scaled hop wait `15 + 30·life/lifeMax` and hop speed
//! `3 + 4·(1 − life/lifeMax)`, the LOS-missed +2 jump (`Collision.CanHit`,
//! `:37530`), every 3rd hop DOUBLES the jump and HALVES the speed
//! (`ai[3]` cycle, `:37537-37542`); airborne — kb 0, vx ×0.99 with
//! the ±1 direction floors. Past 210 ticks grounded the attack roll
//! `Next(3)` (`:37565`): 0 → state 3 (shell), 1 → state 4 (leap,
//! noTileCollide + vy −8), 2 → state 6 (charge); the get-fixed-world
//! `Next(2)` override (`:37582`, type 476 + state 3 only) flips to
//! state 8 (the stuff cannon). The >600 px bail (`:37514`) → state 5.
//!
//! State 3 shell (`:37592-37608`): vx ×0.85, dontTakeDamage, the
//! 180-tick timer, and the EXPERT projectile reflection
//! (`ReflectProjectiles(Hitbox)`, `NPC.cs:66395-66404` — the
//! projectile-pool seam: [`Style87Outcome::reflect_projectiles`] is
//! the request, the sim's projectile pass owns the mutation + its
//! draws; `reflectsProjectiles = true`).
//!
//! State 4 leap (`:37609-37658`): noTileCollide + noGravity + kb 0,
//! direction from vx, retarget; hover at player Center −350 Y with
//! the ×12 / 6-weight steer; within 40 X and 300 above → the
//! `ai[2] = 1` dive latch, the ×8 / 5-weight steer for 6 ticks, then
//! state 4.1 with `velocity = vector170` verbatim.
//!
//! State 4.1 fall (`:37659-37693`): the LOS + free-space latch, the
//! bottom-edge/ascending 10-tick timer → state 2 (or 5 when inside
//! solids), else the noTileCollide/noGravity re-arm; vy += 0.2 capped 16.
//!
//! State 5 retreat (`:37694-37724`): noTileCollide/noGravity/kb 0,
//! the (player −4 Y) ×10 / 5-weight steer, the <200 px + free-space
//! return to state 2; the ≤10 px delta is NOT normalized (verbatim).
//!
//! State 6 charge (`:37725-37790`): grounded hops every 5 ticks
//! with the SIX stacked player-height kicks (−1.25/−1.5/−1.75/−2/
//! −2.25/−2.5 past cy−0/40/80/120/160/200) + the LOS-missed −2,
//! vx = 12·direction; airborne ×0.98 with the ±8 floors; 3 hops
//! grounded → state 2.
//!
//! State 7 despawn (`:37791-37803`): damage 0, life = lifeMax,
//! defense 9999, noTileCollide, alpha +7 capped 255, vx ×0.98.
//!
//! State 8 stuff cannon (`:37804-37825`): vx ×0.85; NOT
//! get-fixed-world OR ≥180 ticks → state 2, else every 20 ticks
//! `AI_87_BigMimic_FireStuffCannonBurst` (`NPC.cs:44821-44841`):
//! 10 × [`shoot_item`] — the item roll `Next(50)` over
//! `ItemID.Sets.ItemsForStuffCannon` (`ID/ItemID.cs:358-365`), the
//! speed `Next(10,26)`, the aim at player Center −(0,120) with the
//! ±5 jitter, the ×num4 normalize and the ±5 velocity jitter. The
//! `Item.NewItem` call passes an explicit velocity → its fallback
//! draws never fire (`Item.cs:49738-49741`); the slot picker is
//! draw-free (`Item.cs:49774-49812`) and `Prefix(-1)` on the block
//! items bails at `CanHavePrefixes` (`Item.cs:515-518`) — 6 explicit
//! draws per shot, the item spawn itself is the
//! [`Style87Outcome::stuff_cannon_items`] request seam.
//!
//! ## RNG census (by construction)
//!
//! The arm's explicit `Main.rand` draws, in C# order:
//! 1. State 2's attack roll `Next(3)` (`:37565`) — only when
//!    `ai[2] > 210` AND grounded AND netMode != 1.
//! 2. The get-fixed-world override `Next(2)` (`:37582`) — only when
//!    `Main.tenthAnniversaryWorld && type == 476 && ai[0] == 3`
//!    (i.e. only after roll 0).
//! 3. State 8's burst (`:37820-37823` → `:44826`/`:44832-44840`):
//!    every 20 ticks, 10 × (Next(50), Next(10,26), Next(-50,51),
//!    Next(-50,51), Next(-50,51), Next(-50,51)).
//! Everything else (TargetClosest, CanHit, the steers) is
//! deterministic. The witnesses replay the sequences against a
//! mirrored oracle stream.
//!
//! ## Seams (the wiring obligations)
//!
//! * The gravity kinds integrate through the caller's movement step
//!   (`NPC.UpdateNPC`'s tail, the style50 split) — the outcome's
//!   `no_tile_collide`/`no_gravity` pick the branch.
//! * `Collision.CanHit(Center,1,1,player.Center,1,1)` rides
//!   `crate::projectile::can_hit` (the ported LOS walk);
//!   `Collision.SolidCollision` rides `Player::solid_collision`.
//! * The expert reflection + the stuff-cannon item spawns are
//!   REQUESTS — the projectile/item passes own their draws.
//! * `netMode` is the classic solo 0 (the port's session), the
//!   `Main.netMode != 1` gates fold true; the client branches are
//!   cited no-ops.
//!
//! See docs/ANALYSIS.md "The Big Mimics (W4-d32 aiStyle-87, code-only)".
#![allow(dead_code)] // wiring deferred — the kinds + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The grouped 473-476 `SetDefaults` row (`NPC.cs:14508-14521`),
/// verbatim — all four Big Mimics share it.
pub struct BigMimicDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit4` (`ID/SoundID.cs:1260`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath6` (`ID/SoundID.cs:1320`).
    pub death_sound: (i32, i32),
    pub value: f32,
    pub knock_back_resist: f32,
    pub rarity: i32,
}

/// `NPC.cs:14508-14521` — `else if (this.type == 473 || 474 || 475 || 476)`.
pub const BIG_MIMIC: BigMimicDefaults = BigMimicDefaults {
    width: 28,
    height: 44,
    ai_style: 87,
    damage: 90,
    defense: 34,
    life_max: 3500,
    hit_sound: (3, 4),
    death_sound: (4, 6),
    value: 30000.0,
    knock_back_resist: 0.1,
    rarity: 5,
};

/// `ItemID.Sets.ItemsForStuffCannon` (`ID/ItemID.cs:358-365`) — the
/// 50-item block roster the state-8 cannon rolls over, in table order.
pub const ITEMS_FOR_STUFF_CANNON: [i32; 50] = [
    2, 3, 61, 836, 409, 593, 664, 169, 370, 1246,
    408, 3271, 3277, 3339, 3276, 3272, 3274, 3275, 3338, 176,
    172, 424, 1103, 3087, 3066, 9, 2503, 2504, 619, 911,
    621, 620, 1727, 276, 4564, 751, 1124, 1125, 824, 129,
    131, 607, 594, 883, 414, 413, 609, 4050, 192, 412,
];

/// The per-tick environment (the `Style86Env` shape + the style74
/// difficulty seam).
pub struct Style87Env<'a> {
    /// The read-only world `TargetClosest`/`CanHit`/`SolidCollision` read.
    pub world: &'a World,
    /// The `Main.player` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp.
    pub ticker: u64,
    /// `this.difficulty` (`NPC.cs:7033`) — the kb curve's input.
    pub difficulty: f32,
    /// `Main.expertMode` — the state-3 reflection gate (`:37603`).
    pub expert: bool,
    /// `Main.tenthAnniversaryWorld` — the get-fixed-world gates
    /// (`:37582`, `:37814`).
    pub tenth_anniversary: bool,
    /// The `Main.rand` stream — every draw cited in the module header
    /// rides it.
    pub rng: &'a mut UnifiedRandom,
}

/// One stuff-cannon item spawn request (`AI_87_BigMimic_ShootItem`'s
/// `Item.NewItem` arguments, `NPC.cs:44840`).
#[derive(Debug, PartialEq)]
pub struct StuffCannonItem {
    pub item_type: i32,
    /// The `NewItem` X/Y/Width/Height box — the NPC's position + size.
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// The explicit velocity (the `NewItem` fallback draws never fire).
    pub vx: f32,
    pub vy: f32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style87Outcome {
    /// The head/state kb writes (`:37473` scaled 0.2, the airborne/
    /// dive/retreat/charge/shell-adjacent 0 rows) — the style74 seam.
    pub knock_back_resist: Option<f32>,
    /// The head's `noTileCollide = false` (`:37475`) + the state
    /// overrides — the caller's integration split.
    pub no_tile_collide: bool,
    /// The head's `noGravity = false` (`:37476`) + the state overrides.
    pub no_gravity: bool,
    /// The head's `dontTakeDamage = false` (`:37474`) + state 3's true
    /// (`:37595`) — written to the NPC directly AND mirrored here for
    /// the witnesses.
    pub dont_take_damage: bool,
    /// The head's `reflectsProjectiles = false` (`:37477`) + the
    /// expert state-3 true (`:37606`).
    pub reflects_projectiles: bool,
    /// The expert state-3 `ReflectProjectiles(Hitbox)` request
    /// (`:37605`, `NPC.cs:66395-66404`) — the projectile pass owns it.
    pub reflect_request: bool,
    /// The state-7 damage/defense writes (`:37793`/`:37795`) — the
    /// port's override fields are the caller's to apply.
    pub despawn_stats: bool,
    /// The state-8 burst's item spawn requests, in draw order.
    pub stuff_cannon_items: Vec<StuffCannonItem>,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the f32
/// sum widened to f64 for the root, the style74 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED: a zero vector gives NaN,
/// exactly like the C# (the style74 `normalize_xna` precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `GameDifficultyData.KnockbackToEnemiesMultiplier`
/// (`GameDifficultyData.cs:86`): Classic 1→1, Master 3→0.8 — the
/// style74 curve, clamped at the ends.
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

/// The target slot's candidate — the style74 `target_of` fallback
/// (the C# indexes `Main.player[this.target]`, 255 slots always
/// exist; an unset/stale slot falls back to slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Entity.Center` for a player candidate (the style74 shape).
fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (p.x + (p.width / 2) as f32, p.y + (p.height / 2) as f32)
}

/// `Collision.CanHit(base.Center, 1, 1, player.Center, 1, 1)` — the
/// ported LOS walk (`crate::projectile::can_hit`).
fn can_hit_player(npc: &Npc, env: &Style87Env<'_>, p: &TargetCandidate) -> bool {
    let (cx, cy) = npc.center();
    let (px, py) = candidate_center(p);
    crate::projectile::can_hit(env.world, cx, cy, 1.0, 1.0, px, py, 1.0, 1.0)
}

/// `Collision.SolidCollision(base.position, base.width, base.height)`.
fn solid_collision(npc: &Npc, world: &World) -> bool {
    let (w, h) = npc.size();
    crate::player::Player::solid_collision(world, npc.x, npc.y, w, h)
}

/// `AI_87_BigMimic_ShootItem` (`NPC.cs:44830-44841`) — ONE shot: the
/// 5 explicit draws after the caller's item roll, the aim maths, the
/// `Item.NewItem` request (the spawn's own draws are cited no-ops —
/// module docs).
fn shoot_item(npc: &Npc, env: &mut Style87Env<'_>, item_type: i32, out: &mut Style87Outcome) {
    // `:44832` — the speed roll.
    let num = env.rng.next_range(10, 26) as f32;
    // `:44833-44836` — the NPC centre and the (player −120 Y) aim.
    let (cx, cy) = npc.center();
    let p = target_of(npc, env.players);
    let (pcx, pcy) = candidate_center(p);
    let mut num2 = pcx - cx;
    let mut num3 = (pcy - 120.0) - cy;
    // `:44837-44838` — the ±5 aim jitter.
    num2 += env.rng.next_range(-50, 51) as f32 * 0.1;
    num3 += env.rng.next_range(-50, 51) as f32 * 0.1;
    // `:44839` — the normalize scale.
    let num4 = num / sqrt_f32(num2, num3);
    // `:44840` — the ±5 velocity jitter inside the NewItem velocity.
    let (w, h) = npc.size();
    out.stuff_cannon_items.push(StuffCannonItem {
        item_type,
        x: npc.x as i32,
        y: npc.y as i32,
        width: w as i32,
        height: h as i32,
        vx: num2 * num4 + env.rng.next_range(-50, 51) as f32 * 0.1,
        vy: num3 * num4 + env.rng.next_range(-50, 51) as f32 * 0.1,
    });
}

/// One fixed step of the aiStyle-87 arm (`NPC.cs:37471-37826`), in
/// C# order. `npc` is the mimic, `ty` its vanilla type (the census
/// seam — 473/474/475/476). The caller owns the movement integration
/// after `AI()` (the outcome's flags) and drains the requests.
pub fn style87_step(npc: &mut Npc, ty: i32, env: &mut Style87Env<'_>) -> Style87Outcome {
    let mut out = Style87Outcome::default();
    // `:37473-37477` — the head.
    out.knock_back_resist = Some(knockback_scaled(0.2, env.difficulty));
    npc.dont_take_damage = false;
    out.dont_take_damage = false;
    out.no_tile_collide = false;
    out.no_gravity = false;
    out.reflects_projectiles = false;
    // `:37478-37489` — the dead-target bail.
    if npc.ai[0] != 7.0 && target_of(npc, env.players).dead {
        target_closest(npc, env.world, env.players, true, 87, env.ticker);
        if target_of(npc, env.players).dead {
            npc.ai[0] = 7.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
    }
    if npc.ai[0] == 0.0 {
        // `:37490-37500` — dormant.
        target_closest(npc, env.world, env.players, true, 87, env.ticker);
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let len = sqrt_f32(pcx - cx, pcy - cy);
        // `Main.netMode != 1` folds true (the solo session).
        if npc.vx != 0.0 || npc.vy > 100.0 || npc.just_hit || len < 80.0 {
            npc.ai[0] = 1.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 1.0 {
        // `:37501-37510` — the awakening timer.
        npc.ai[1] += 1.0;
        if npc.ai[1] > 36.0 {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 2.0 {
        // `:37511-37591` — the hop cycle.
        {
            let p = target_of(npc, env.players);
            let (pcx, pcy) = candidate_center(p);
            let (cx, cy) = npc.center();
            if sqrt_f32(pcx - cx, pcy - cy) > 600.0 {
                npc.ai[0] = 5.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        if npc.vy == 0.0 {
            // `:37522-37548` — grounded.
            target_closest(npc, env.world, env.players, true, 87, env.ticker);
            npc.vx *= 0.85;
            npc.ai[1] += 1.0;
            let life_frac = npc.hp as f32 / npc.max_hp as f32;
            let num1226 = 15.0 + 30.0 * life_frac;
            let mut num1227 = 3.0 + 4.0 * (1.0 - life_frac);
            let mut num1228 = 4.0f32;
            let p = target_of(npc, env.players);
            if !can_hit_player(npc, env, p) {
                num1228 += 2.0;
            }
            if npc.ai[1] > num1226 {
                npc.ai[3] += 1.0;
                if npc.ai[3] >= 3.0 {
                    npc.ai[3] = 0.0;
                    num1228 *= 2.0;
                    num1227 /= 2.0;
                }
                npc.ai[1] = 0.0;
                npc.vy -= num1228;
                npc.vx = num1227 * npc.facing as f32;
                npc.net_update = true;
            }
        } else {
            // `:37549-37561` — airborne.
            out.knock_back_resist = Some(0.0);
            npc.vx *= 0.99;
            if npc.facing < 0 && npc.vx > -1.0 {
                npc.vx = -1.0;
            }
            if npc.facing > 0 && npc.vx < 1.0 {
                npc.vx = 1.0;
            }
        }
        npc.ai[2] += 1.0;
        // `:37563-37590` — the attack roll (>210 as f64, grounded).
        if (npc.ai[2] as f64) > 210.0 && npc.vy == 0.0 {
            match env.rng.next_max(3) {
                0 => npc.ai[0] = 3.0,
                1 => {
                    npc.ai[0] = 4.0;
                    out.no_tile_collide = true;
                    npc.vy = -8.0;
                }
                2 => npc.ai[0] = 6.0,
                _ => npc.ai[0] = 2.0,
            }
            // `:37582-37585` — the get-fixed-world override.
            if env.tenth_anniversary && ty == 476 && npc.ai[0] == 3.0 && env.rng.next_max(2) == 0 {
                npc.ai[0] = 8.0;
            }
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 3.0 {
        // `:37592-37608` — the shell.
        npc.vx *= 0.85;
        npc.dont_take_damage = true;
        out.dont_take_damage = true;
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 180.0 {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
        }
        if env.expert {
            out.reflect_request = true;
            out.reflects_projectiles = true;
        }
    } else if npc.ai[0] == 4.0 {
        // `:37609-37658` — the leap.
        out.no_tile_collide = true;
        out.no_gravity = true;
        out.knock_back_resist = Some(0.0);
        npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
        npc.sprite_dir = npc.facing;
        target_closest(npc, env.world, env.players, true, 87, env.ticker);
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        // `:37624-37626` — the hover point 350 above the player.
        let mut dx = pcx - cx;
        let mut dy = (pcy - 350.0) - cy;
        if npc.ai[2] == 1.0 {
            // `:37627-37642` — the dive latch's 6-tick steer.
            npc.ai[1] += 1.0;
            dx = pcx - cx;
            dy = pcy - cy;
            let (nx, ny) = normalize_xna(dx, dy);
            let (tx, ty2) = (nx * 8.0, ny * 8.0);
            npc.vx = (npc.vx * 4.0 + tx) / 5.0;
            npc.vy = (npc.vy * 4.0 + ty2) / 5.0;
            if npc.ai[1] > 6.0 {
                npc.ai[1] = 0.0;
                npc.ai[0] = 4.1;
                npc.ai[2] = 0.0;
                npc.vx = tx;
                npc.vy = ty2;
                npc.net_update = true;
            }
        } else if (cx - pcx).abs() < 40.0 && cy < pcy - 300.0 {
            // `:37643-37651` — the latch.
            npc.ai[1] = 0.0;
            npc.ai[2] = 1.0;
            npc.net_update = true;
        } else {
            // `:37652-37657` — the ×12 / 6-weight hover steer.
            let (nx, ny) = normalize_xna(dx, dy);
            npc.vx = (npc.vx * 5.0 + nx * 12.0) / 6.0;
            npc.vy = (npc.vy * 5.0 + ny * 12.0) / 6.0;
        }
    } else if npc.ai[0] == 4.1 {
        // `:37659-37693` — the fall.
        out.knock_back_resist = Some(0.0);
        let p = target_of(npc, env.players);
        if npc.ai[2] == 0.0 && can_hit_player(npc, env, p) && !solid_collision(npc, env.world) {
            npc.ai[2] = 1.0;
        }
        if npc.y + npc.size().1 >= p.y || npc.vy <= 0.0 {
            npc.ai[1] += 1.0;
            if npc.ai[1] > 10.0 {
                npc.ai[0] = 2.0;
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
                if solid_collision(npc, env.world) {
                    npc.ai[0] = 5.0;
                }
            }
        } else if npc.ai[2] == 0.0 {
            out.no_tile_collide = true;
            out.no_gravity = true;
            out.knock_back_resist = Some(0.0);
        }
        npc.vy += 0.2;
        if npc.vy > 16.0 {
            npc.vy = 16.0;
        }
    } else if npc.ai[0] == 5.0 {
        // `:37694-37724` — the retreat.
        npc.facing = if npc.vx > 0.0 { 1 } else { -1 };
        npc.sprite_dir = npc.facing;
        out.no_tile_collide = true;
        out.no_gravity = true;
        out.knock_back_resist = Some(0.0);
        let p = target_of(npc, env.players);
        let (pcx, pcy) = candidate_center(p);
        let (cx, cy) = npc.center();
        let mut dx = pcx - cx;
        let mut dy = pcy - cy;
        dy -= 4.0;
        let len = sqrt_f32(dx, dy);
        if len < 200.0 && !solid_collision(npc, env.world) {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        // `:37718-37722` — normalize ONLY past 10 px (verbatim).
        if len > 10.0 {
            let (nx, ny) = normalize_xna(dx, dy);
            dx = nx * 10.0;
            dy = ny * 10.0;
        }
        npc.vx = (npc.vx * 4.0 + dx) / 5.0;
        npc.vy = (npc.vy * 4.0 + dy) / 5.0;
    } else if npc.ai[0] == 6.0 {
        // `:37725-37790` — the charge.
        out.knock_back_resist = Some(0.0);
        if npc.vy == 0.0 {
            target_closest(npc, env.world, env.players, true, 87, env.ticker);
            npc.vx *= 0.8;
            npc.ai[1] += 1.0;
            if npc.ai[1] > 5.0 {
                npc.ai[1] = 0.0;
                npc.vy -= 4.0;
                let p = target_of(npc, env.players);
                let p_bottom = p.y + p.height as f32;
                let (_, cy) = npc.center();
                // `:37737-37760` — the SIX stacked height kicks.
                if p_bottom < cy {
                    npc.vy -= 1.25;
                }
                if p_bottom < cy - 40.0 {
                    npc.vy -= 1.5;
                }
                if p_bottom < cy - 80.0 {
                    npc.vy -= 1.75;
                }
                if p_bottom < cy - 120.0 {
                    npc.vy -= 2.0;
                }
                if p_bottom < cy - 160.0 {
                    npc.vy -= 2.25;
                }
                if p_bottom < cy - 200.0 {
                    npc.vy -= 2.5;
                }
                // `:37761-37764` — the LOS-missed kick.
                if !can_hit_player(npc, env, p) {
                    npc.vy -= 2.0;
                }
                npc.vx = 12.0 * npc.facing as f32;
                npc.ai[2] += 1.0;
                npc.net_update = true;
            }
        } else {
            // `:37770-37781` — airborne.
            npc.vx *= 0.98;
            if npc.facing < 0 && npc.vx > -8.0 {
                npc.vx = -8.0;
            }
            if npc.facing > 0 && npc.vx < 8.0 {
                npc.vx = 8.0;
            }
        }
        // `:37782-37789` — 3 hops grounded → back to the hop cycle.
        if npc.ai[2] >= 3.0 && npc.vy == 0.0 {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 7.0 {
        // `:37791-37803` — the despawn fade.
        out.despawn_stats = true;
        npc.hp = npc.max_hp;
        out.no_tile_collide = true;
        npc.alpha = npc.alpha.saturating_add(7);
        npc.vx *= 0.98;
    } else if npc.ai[0] == 8.0 {
        // `:37804-37825` — the stuff cannon.
        npc.vx *= 0.85;
        npc.ai[1] += 1.0;
        if !env.tenth_anniversary || npc.ai[1] >= 180.0 {
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
        } else if npc.ai[1] % 20.0 == 0.0 {
            // `AI_87_BigMimic_FireStuffCannonBurst` (`:44821-44828`).
            for _ in 0..10 {
                let item = ITEMS_FOR_STUFF_CANNON[env.rng.next_max(50) as usize];
                shoot_item(npc, env, item, &mut out);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 473-shaped NPC: no kind yet, so the fixture rides a
    /// placeholder with the C# box through `size_override` (the
    /// style86 apparition convention).
    fn mimic(x: f32, y: f32) -> Npc {
        let d = &BIG_MIMIC;
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
    ) -> Style87Env<'a> {
        Style87Env {
            world: w,
            players,
            ticker: 0,
            difficulty: 1.0,
            expert: false,
            tenth_anniversary: false,
            rng,
        }
    }

    /// `NPC.cs:14508-14521` — the grouped SetDefaults row, field by
    /// field; the 50-item cannon roster's bounds.
    #[test]
    fn w4_d32_s87_the_defaults_row_and_the_cannon_roster() {
        let d = &BIG_MIMIC;
        assert_eq!((d.width, d.height), (28, 44));
        assert_eq!(d.ai_style, 87);
        assert_eq!((d.damage, d.defense, d.life_max), (90, 34, 3500));
        assert_eq!(d.hit_sound, (3, 4), "SoundID.NPCHit4");
        assert_eq!(d.death_sound, (4, 6), "SoundID.NPCDeath6");
        assert_eq!(d.value, 30000.0);
        assert_eq!(d.knock_back_resist, 0.1);
        assert_eq!(d.rarity, 5);
        assert_eq!(ITEMS_FOR_STUFF_CANNON.len(), 50);
        assert_eq!(ITEMS_FOR_STUFF_CANNON[0], 2);
        assert_eq!(ITEMS_FOR_STUFF_CANNON[49], 412);
    }

    /// `:37473-37477` — the head: the scaled 0.2 kb, the four flag
    /// resets, every tick.
    #[test]
    fn w4_d32_s87_the_head_resets_the_flags_and_scales_the_kb() {
        let w = air();
        let players = [cand(2000.0, 2000.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 9.0; // no state arm
        n.dont_take_damage = true;
        let mut e = env(&w, &players, &mut rng);
        let out = style87_step(&mut n, 473, &mut e);
        assert_eq!(out.knock_back_resist, Some(0.2));
        assert!(!out.no_tile_collide);
        assert!(!out.no_gravity);
        assert!(!n.dont_take_damage);
        assert!(!out.reflects_projectiles);
        // The Master curve: 0.2 × 0.8.
        e.difficulty = 3.0;
        let out = style87_step(&mut n, 473, &mut e);
        assert_eq!(out.knock_back_resist, Some(0.2 * 0.8));
    }

    /// `:37478-37489` — a dead target (even after the retarget)
    /// latches state 7 with ai[1..3] zeroed.
    #[test]
    fn w4_d32_s87_the_dead_target_latches_the_despawn_state() {
        let w = air();
        let mut dead = cand(200.0, 100.0);
        dead.dead = true;
        let players = [dead];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mimic(100.0, 100.0);
        n.ai = [2.0, 5.0, 6.0, 1.0];
        let mut e = env(&w, &players, &mut rng);
        let _ = style87_step(&mut n, 473, &mut e);
        assert_eq!(n.ai[0], 7.0);
        assert_eq!((n.ai[1], n.ai[2], n.ai[3]), (0.0, 0.0, 0.0));
        assert!(n.net_update);
        // State 7 itself: the fade + the stats request.
        n.alpha = 250;
        n.vx = 10.0;
        let out = style87_step(&mut n, 473, &mut e);
        assert_eq!(n.alpha, 255, "the +7 fade caps at 255");
        assert_eq!(n.vx, 10.0 * 0.98);
        assert!(out.despawn_stats);
        assert!(out.no_tile_collide);
    }

    /// `:37490-37510` — the dormant wake (the 80-px proximity arm)
    /// and the 36-tick awakening timer.
    #[test]
    fn w4_d32_s87_the_dormant_wake_and_the_36_tick_timer() {
        let w = air();
        let players = [cand(140.0, 100.0)]; // within 80 px of the centre
        let mut rng = UnifiedRandom::new(7);
        let mut n = mimic(100.0, 100.0);
        let mut e = env(&w, &players, &mut rng);
        let _ = style87_step(&mut n, 473, &mut e);
        assert_eq!(n.ai[0], 1.0);
        for _ in 0..36 {
            let _ = style87_step(&mut n, 473, &mut e);
        }
        assert_eq!(n.ai[0], 1.0, "36 ticks is not > 36");
        let _ = style87_step(&mut n, 473, &mut e);
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.ai[1], 0.0);
    }

    /// `:37527-37546` — the life-scaled hop: at FULL life the wait is
    /// 45 and the speed 3; every 3rd hop doubles the jump (4 → 8) and
    /// halves the speed (3 → 1.5).
    #[test]
    fn w4_d32_s87_the_hop_wait_and_the_every_third_double_jump() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[1] = 45.0; // the next grounded tick hops
        let mut e = env(&w, &players, &mut rng);
        let _ = style87_step(&mut n, 473, &mut e);
        assert_eq!(n.vy, -4.0, "the base jump (LOS clear in air world)");
        assert_eq!(n.vx, 3.0 * n.facing as f32);
        assert_eq!(n.ai[3], 1.0);
        // Hops 2 and 3: the third doubles.
        for hop in 2..=3 {
            n.vy = 0.0;
            n.ai[1] = 45.0;
            let _ = style87_step(&mut n, 473, &mut e);
            if hop == 3 {
                assert_eq!(n.vy, -8.0, "the every-3rd double jump");
                assert_eq!(n.vx, 1.5 * n.facing as f32, "the halved speed");
                assert_eq!(n.ai[3], 0.0, "the cycle resets");
            }
        }
    }

    /// `:37563-37590` — the attack roll consumes exactly ONE Next(3)
    /// (no get-fixed-world override on a non-476); the mirrored
    /// oracle pins the stream.
    #[test]
    fn w4_d32_s87_the_attack_roll_draws_one_next3() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(87);
        let mut oracle = UnifiedRandom::new(87);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[2] = 210.0; // the bump makes 211 > 210
        let mut e = env(&w, &players, &mut rng);
        let _ = style87_step(&mut n, 473, &mut e);
        let roll = oracle.next_max(3);
        assert_eq!(rng.peek(), oracle.peek(), "exactly one draw");
        let expect = match roll {
            0 => 3.0,
            1 => 4.0,
            _ => 6.0,
        };
        assert_eq!(n.ai[0], expect);
        assert_eq!((n.ai[1], n.ai[2], n.ai[3]), (0.0, 0.0, 0.0));
    }

    /// `:37582-37585` — the get-fixed-world override: type 476 +
    /// roll 0 draws the SECOND Next(2); a non-476 never draws it.
    #[test]
    fn w4_d32_s87_the_getfixed_override_is_476_only() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        // Find a seed whose Next(3) is 0 (the shell roll).
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(3) != 0 {
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut oracle = UnifiedRandom::new(seed);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 2.0;
        n.ai[2] = 210.0;
        let mut e = env(&w, &players, &mut rng);
        e.tenth_anniversary = true;
        let _ = style87_step(&mut n, 476, &mut e);
        oracle.next_max(3);
        let flip = oracle.next_max(2);
        assert_eq!(rng.peek(), oracle.peek(), "the override draws Next(2)");
        assert_eq!(n.ai[0], if flip == 0 { 8.0 } else { 3.0 });
    }

    /// `:37592-37608` — the shell: dontTakeDamage, the 180-tick
    /// timer, the expert-only reflection request.
    #[test]
    fn w4_d32_s87_the_shell_reflects_only_in_expert() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 3.0;
        n.vx = 10.0;
        let mut e = env(&w, &players, &mut rng);
        let out = style87_step(&mut n, 473, &mut e);
        assert!(n.dont_take_damage);
        assert_eq!(n.vx, 8.5);
        assert!(!out.reflect_request, "classic: no reflection");
        e.expert = true;
        let out = style87_step(&mut n, 473, &mut e);
        assert!(out.reflect_request);
        assert!(out.reflects_projectiles);
        n.ai[1] = 179.0;
        let _ = style87_step(&mut n, 473, &mut e);
        assert_eq!(n.ai[0], 2.0, "the 180th tick exits the shell");
    }

    /// `:37804-37825` + `:44821-44841` — the state-8 burst: every 20
    /// ticks, 10 shots × 6 draws (the item roll + speed + 2 aim + 2
    /// velocity jitters); the mirrored oracle replays the sequence
    /// and the requests carry the rolled items.
    #[test]
    fn w4_d32_s87_the_cannon_burst_draws_ten_times_six() {
        let w = air();
        let players = [cand(300.0, 400.0)];
        let mut rng = UnifiedRandom::new(476);
        let mut oracle = UnifiedRandom::new(476);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 8.0;
        n.ai[1] = 19.0; // the bump makes 20 → the burst
        let mut e = env(&w, &players, &mut rng);
        e.tenth_anniversary = true;
        let out = style87_step(&mut n, 476, &mut e);
        assert_eq!(out.stuff_cannon_items.len(), 10);
        for shot in &out.stuff_cannon_items {
            let item = ITEMS_FOR_STUFF_CANNON[oracle.next_max(50) as usize];
            let _speed = oracle.next_range(10, 26);
            let _ax = oracle.next_range(-50, 51);
            let _ay = oracle.next_range(-50, 51);
            let _vx = oracle.next_range(-50, 51);
            let _vy = oracle.next_range(-50, 51);
            assert_eq!(shot.item_type, item, "the rolled roster item");
            assert_eq!((shot.x, shot.y, shot.width, shot.height), (100, 100, 28, 44));
        }
        assert_eq!(rng.peek(), oracle.peek(), "the 60-draw burst order");
        // The 180-tick exit (get-fixed-world keeps firing until then).
        n.ai[1] = 179.0;
        let out = style87_step(&mut n, 476, &mut e);
        assert_eq!(n.ai[0], 2.0);
        assert!(out.stuff_cannon_items.is_empty());
    }

    /// `:37725-37768` — the charge hop: the six stacked height kicks
    /// fire when the player's bottom is 200+ px above the centre
    /// (−4 −1.25 −1.5 −1.75 −2 −2.25 −2.5 = −15), vx = 12·direction.
    #[test]
    fn w4_d32_s87_the_charge_hop_stacks_the_six_height_kicks() {
        let w = air();
        // The player far ABOVE: bottom (y+42) < cy−200.
        let players = [cand(300.0, -400.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 6.0;
        n.ai[1] = 5.0; // the next grounded tick hops
        let mut e = env(&w, &players, &mut rng);
        let _ = style87_step(&mut n, 473, &mut e);
        // cy = 122; p_bottom = −358 < 122−200 = −78 → all six fire.
        // LOS in the air world is clear → no extra −2.
        assert_eq!(n.vy, -15.0);
        assert_eq!(n.vx, 12.0 * n.facing as f32);
        assert_eq!(n.ai[2], 1.0);
    }

    /// The state machine's zero-draw census: the hop/leap/charge
    /// states away from the attack-roll threshold consume NO
    /// `Main.rand` (TargetClosest/CanHit/steers are deterministic).
    #[test]
    fn w4_d32_s87_the_states_away_from_the_roll_draw_nothing() {
        let w = air();
        let players = [cand(300.0, 100.0)];
        let mut rng = UnifiedRandom::new(87);
        let mut n = mimic(100.0, 100.0);
        n.ai[0] = 2.0;
        let mut e = env(&w, &players, &mut rng);
        for _ in 0..200 {
            let _ = style87_step(&mut n, 473, &mut e);
            n.vy = 0.0; // stay grounded, ai[2] never passes 210
            n.ai[2] = 0.0;
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(87).peek(), "no draws");
    }
}
