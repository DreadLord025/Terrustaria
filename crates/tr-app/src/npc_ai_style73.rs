//! The aiStyle-73 family — the Martian Turret (`NPC.AI()`'s
//! `else if (this.aiStyle == 73)` arm, `NPC.cs:35102-35217`). S1-D-3,
//! aiStyle-73 round, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the MartianTurret 387
//! (`NPCID.MartianTurret`, `ID/NPCID.cs:11451`, `SetDefaults` row
//! `NPC.cs:13922-13934`). The census of `this.aiStyle = 73;` across the
//! decompiled tree returns exactly two hits: `NPC.cs:13926` (the
//! `type == 387` branch) and `Projectile.cs:4955` (the FOUNTAIN
//! projectiles 419-422 — a different machine in projectile space, out of
//! NPC scope; the wiring engine's dropped fountain requests, ANALYSIS
//! "Wiring engine (M7-WIRE-2)", ride that style). No dynamic
//! `aiStyle` reassignment to 73 exists; no second NPC kind rides it.
//!
//! WIRED by W4-c11: the roster carries `NpcKind::MartianTurret` (387,
//! the wire id 214) and `update_rng`'s aiStyle axis claims style 73
//! (`npc.rs::ai_073_martian_turret` — the session candidate, netMode 0,
//! classic difficulty, the one-tick `just_hit` flag; the shot queues on
//! `wants_hostile_shot` and the sim's drain spawns the 435 row). The
//! machine still keys on the vanilla `npc_type` (the module's seam).
//!
//! ## The machine
//!
//! A stationary gun emplacement: `TargetClosest(faceTarget: false)`
//! (`:35104`) tracks without facing, `spriteDirection = direction`
//! (`:35105`), the X velocity decays ×0.93 with the ±0.1 snap-to-zero
//! (`:35106-35110`). The firing gate `flag74` (`:35112`) is
//! `player.active && !player.dead && AI_GlobalFiringDistanceCheck`
//! (`NPC.cs:55835-55841` — the 1920×1200 Point-centred rect inflated
//! −50, the `npc_ai_caster.rs` port).
//!
//! The 387 materialize block (`:35113-35178`): while `ai[1] < 120` the
//! turret is an invulnerable ghost — `ai[1]++`, the alpha ramp
//! `(int)((1 - (ai[1] - 60) / 60) * 255)` above 60 / flat 255 below,
//! `dontTakeDamage = true`, `frameCounter = 0` / `frame.Y = 0`, the
//! four paired dust-226 rings spinning around
//! `(0, -30).RotatedBy(ai[1] / 60 * 1.5 * 2π) * (1, 0.4)`, the fading-in
//! blue light, and an early `return` (`:35168`). At `ai[1] == 120` the
//! one-shot latch bumps to 121 + `netUpdate` (`:35170-35174`); from
//! there on the constant blue light, `dontTakeDamage = false` and
//! `alpha = 0` (`:35175-35177`).
//!
//! The shot cycle (`:35179-35216`): `ai[0] == 0 && !flag74` re-arms to
//! −120 (the out-of-view hold); `ai[0] < 60` increments; `justHit`
//! resets to −30 + `netUpdate`; at `ai[0] == 60` on a non-client
//! (`netMode != 1`) the bolt fires — `ai[0] = -120` + `netUpdate`, the
//! aim from `Center - (0, 10)` to the player centre with FOUR
//! `Main.rand` draws (X jitter −100..100, Y jitter −100..100, X scale
//! 70..130 ×0.01, Y scale 70..130 ×0.01), `Normalize()` with the
//! NaN → −UnitY fallback, ×14 speed, and the type-435
//! `MartianTurretBolt` (`ID/ProjectileID.cs:1424`) at
//! `GetAttackDamage_ForProjectiles(35, 35 * 0.8)` — the 0.8 scale is
//! the `type >= 381 && type <= 392` Martian band (`:35211-35214`),
//! always true for the census's single kind.
//!
//! ## RNG order (by construction)
//!
//! The arm's ONLY arm-level draws are the shot's four
//! `Main.rand.Next` calls (`:35199-35202`), consumed in exactly that
//! order (X jitter, Y jitter, X scale, Y scale) on each
//! `ai[0] == 60 && netMode != 1` tick — the witness
//! `the_shot_consumes_the_four_draws_in_order` replays them against a
//! mirrored oracle stream.
//!
//! The materialize dust ring (`:35133-35166`) is a VISUAL: eight
//! `Dust.NewDust` calls per fading tick whose draws all live inside
//! `Dust.NewDust` itself (`Dust.cs:170-175`/`:190` — the pool-gate
//! `Next`, the position/velocity/frame/scale draws). The dust system
//! is unported (S1-E-2/S1-E-3), so the ring is skipped wholesale — the
//! `npc_lights.rs` / ForceBubble precedent: skipping an arm whole
//! leaves the `Main.rand` stream untouched, because in the port there
//! is no dust consumer drawing those values. [`turret_materialize_dust_plan`]
//! ports the ring's branch structure (the spin angle, the four
//! offsets/scales, the alternating mirror) and counts only — no RNG
//! parameter; the wiring wave must draw the `Dust.NewDust` internals
//! in C# order when the dust pass lands.
//!
//! `TargetClosest` (`NPC.cs:77652-77781`, the `npc_target.rs` port),
//! `AI_GlobalFiringDistanceCheck` and `Lighting.AddLight` draw nothing.
//! The gravity/integration tail of `UpdateNPC` (the turret has
//! `noGravity`/`noTileCollide` false) is the caller's movement step,
//! not this arm's — as in the C#, `AI()` runs before it (`NPC.cs:90423`).
//!
//! ## FindFrame case 387
//!
//! `NPC.cs:71663-71696`: while `ai[0] > 0` the counter advances 1/tick
//! +1 more at `ai[0] >= 20` +1 more at `ai[0] >= 40` (so 3/tick at the
//! shot trigger), flipping the two-frame strip every 10; otherwise the
//! flat 15-tick flip. The materialize block pins `frameCounter = 0` /
//! `frame.Y = 0` every fading tick, so the strip only runs once the
//! turret is solid. [`turret_find_frame`] ports it as a pure pair
//! function (`Npc::frame_counter` is private to `npc.rs` — the
//! BlazingWheel precedent); `Main.npcFrameCount[387] = 2`
//! (`Main.cs:67122`, the 380-389 row `12, 9, 9, 9, 1, 9, 11, 2, 2, 9`,
//! index 387 = the 8th entry).
//!
//! See docs/ANALYSIS.md "The Martian Turret (S1-D-3 aiStyle-73,
//! code-only)".
#![allow(dead_code)] // WIRED by W4-c11 (the axis' 73 arm + the gravity/collision tail); the defaults row + the dust plan stay as the audit cross-check (the ring itself is the deferred dust draw — module docs)

use crate::npc::Npc;
use crate::npc_ai_caster::global_firing_distance_check;
use crate::npc_ai_style37::{attack_damage_for_projectiles, rotated_by};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// The type-387 `SetDefaults` row (`NPC.cs:13922-13934`), verbatim,
/// with the head defaults the branch leaves untouched (`:8507`
/// `dontTakeDamage = false`, `:8522` `noTileCollide = false`, `:8525`
/// `alpha = 0`, `:8535` `noGravity = false`, `:8536` `scale = 1`,
/// `:8544` `value = 0`, `:8553` `canGhostHeal = true` — overridden
/// false at `:13933`).
pub struct MartianTurretDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit53` (`ID/SoundID.cs:1309`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath56` (`ID/SoundID.cs:1370`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub can_ghost_heal: bool,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub value: f32,
}

/// `NPC.cs:13922-13934` — `else if (this.type == 387)`.
pub const MARTIAN_TURRET: MartianTurretDefaults = MartianTurretDefaults {
    width: 20,
    height: 50,
    ai_style: 73,
    damage: 10,
    defense: 40,
    life_max: 200,
    hit_sound: (3, 53),
    death_sound: (4, 56),
    knock_back_resist: 0.0,
    can_ghost_heal: false,
    no_gravity: false,
    no_tile_collide: false,
    value: 0.0,
};

/// `Main.npcFrameCount[387]` = 2 (`Main.cs:67122`, the 380-389 row of
/// the `:67082` array, index 387 = the 8th entry).
pub const MARTIAN_TURRET_FRAME_COUNT: u32 = 2;

/// The arm's per-tick side effects beyond the NPC state — the wiring
/// seams (module docs).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style73Outcome {
    /// The `Lighting.AddLight((int)Center.X / 16,
    /// (int)(Center.Y - 10) / 16, …)` contribution (`:35167` fading /
    /// `:35175` solid): `(tileX, tileY, [r, g, b])` for the caller's
    /// `EntityLights` feed. `None` on non-387 ticks (no other kind
    /// rides the style).
    pub light: Option<(i32, i32, [f32; 3])>,
    /// The materialize dust ring plan (`:35133-35166`) — the visual
    /// skip's branch structure, count-only (module docs). `Some` on
    /// each fading tick (`ai[1] < 120`), `None` otherwise.
    pub materialize_dust: Option<TurretMaterializeDust>,
    /// The shot request (`:35215`): the type-435 bolt spawned at the
    /// muzzle `Center - (0, 10)` with the jittered ×14 velocity, the
    /// difficulty-remapped damage and knockback 0, owner `Main.myPlayer`.
    pub shot: Option<TurretBolt>,
}

/// The `Projectile.NewProjectile` request of the shot cycle
/// (`NPC.cs:35215`). The pure module never touches the pools; the
/// wiring caller drains this into the projectile spawn (the type-435
/// `SetDefaults` row is the projectile family's obligation —
/// `Projectile.cs`, the aiStyle-29 bolt lineage).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurretBolt {
    /// `ProjectileID.MartianTurretBolt` (`ID/ProjectileID.cs:1424`).
    pub projectile_type: i32,
    /// `vector126` — `base.Center - Vector2.UnitY * 10f` (`:35197`).
    pub x: f32,
    pub y: f32,
    /// The normalized, ×14-scaled jittered aim (`:35198-35208`).
    pub vx: f32,
    pub vy: f32,
    /// `GetAttackDamage_ForProjectiles(35, 35 * 0.8)` (`:35209-35215`)
    /// — classic 35, expert/master 28.
    pub damage: i32,
    /// The C# `KnockBack: 0f` (`:35215`).
    pub knock_back: f32,
}

/// The materialize ring's geometry (`NPC.cs:35131-35166`), count-only
/// — the dust spawn itself is the documented visual skip (module docs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurretMaterializeDust {
    /// `num1019 = ai[1] / 60f` AFTER the increment (`:35131`) — also
    /// the light's fade factor (`:35167`).
    pub spin_factor: f32,
    /// `vector124` — `(0, -30).RotatedBy(num1019 * 1.5 * 2π) * (1, 0.4)`
    /// (`:35132`), the spin vector BEFORE the per-pair mirror.
    pub spin: (f32, f32),
    /// The eight dust placements in C# order: for each of the four
    /// offsets (`(0, -15)` ×0.15, `(0, -5)` ×0.3, `(0, 5)` ×0.6,
    /// `(0, 20)` ×0.45, `:35137-35156`) a pair — first at
    /// `Center + spin * scale + offset`, then `spin *= -1` (`:35161`)
    /// and the mirrored twin. Every dust is type 226, noGravity,
    /// velocity zero, alpha 100, scale 0.5 (`:35157-35165`).
    pub placements: [(f32, f32); 8],
}

/// `NPC.FindFrame` case 387 (`NPC.cs:71663-71696`) — one tick, as a
/// pure pair function over `(frame_row, frameCounter)` (the
/// `blazing_wheel_frame` precedent: `Npc::frame_counter` is private to
/// `npc.rs`; the wiring step copies them to/from `Npc`). `num` is the
/// sheet row height (`TextureAssets.Npc[387].Height() / 2`,
/// `NPC.cs:66694`); the port's row index is `frame.Y / num`.
pub fn turret_find_frame(frame_row: u32, frame_counter: f32, ai0: f32) -> (u32, f32) {
    let mut counter = frame_counter;
    let mut row = frame_row;
    if ai0 > 0.0 {
        // `:71664-71684` — the charged strip: 1/tick, +1 at >= 20,
        // +1 at >= 40, flip every 10 over the 2-row sheet.
        counter += 1.0;
        if ai0 >= 20.0 {
            counter += 1.0;
        }
        if ai0 >= 40.0 {
            counter += 1.0;
        }
        if counter >= 10.0 {
            counter = 0.0;
            row += 1;
            if row >= MARTIAN_TURRET_FRAME_COUNT {
                row = 0;
            }
        }
    } else {
        // `:71686-71695` — the idle strip: the flat 15-tick flip.
        counter += 1.0;
        if counter >= 15.0 {
            counter = 0.0;
            row += 1;
            if row >= MARTIAN_TURRET_FRAME_COUNT {
                row = 0;
            }
        }
    }
    (row, counter)
}

/// The materialize ring's geometry for one fading tick — the branch
/// structure of `NPC.cs:35131-35166` without the `Dust.NewDust` draws
/// (module docs). `ai1` is the value AFTER the tick's increment;
/// `center` is `base.Center` (the turret never moves during the fade —
/// the X decay runs but the caller's integration is what moves it).
pub fn turret_materialize_dust_plan(ai1: f32, center: (f32, f32)) -> TurretMaterializeDust {
    // `:35131-35132` — num1019 and the spin vector. `1.5f * ((float)
    // Math.PI * 2f)` is the f32 product chain, widened to double for
    // RotatedBy (`Utils.cs:1675-1684`, the style-37 port).
    let spin_factor = ai1 / 60.0;
    let angle = (spin_factor * 1.5 * (std::f32::consts::PI * 2.0)) as f64;
    let (rx, ry) = rotated_by(0.0, -30.0, angle, 0.0, 0.0);
    let mut spin = (rx * 1.0, ry * 0.4);
    // `:35133-35166` — the four offsets/scales, each a mirrored pair.
    let offsets: [(f32, f32, f32); 4] = [
        (0.0, -15.0, 0.15), // `:35137-35141`
        (0.0, -5.0, 0.3),   // `:35142-35146`
        (0.0, 5.0, 0.6),    // `:35147-35151`
        (0.0, 20.0, 0.45),  // `:35152-35156`
    ];
    let mut placements = [(0.0f32, 0.0f32); 8];
    for (i, &(ox, oy, scale)) in offsets.iter().enumerate() {
        // `:35159` — Center + vector124 * num1021 + vector125.
        placements[i * 2] = (center.0 + spin.0 * scale + ox, center.1 + spin.1 * scale + oy);
        // `:35161` — vector124 *= -1f, then the mirrored twin (`:35164`).
        spin = (-spin.0, -spin.1);
        placements[i * 2 + 1] = (center.0 + spin.0 * scale + ox, center.1 + spin.1 * scale + oy);
    }
    TurretMaterializeDust { spin_factor, spin: (rx * 1.0, ry * 0.4), placements }
}

/// The arm's `Lighting.AddLight` source for the draw-side feed
/// (`app/frame/draw.rs`, the ForceBubble precedent — the sim's outcome
/// light is discarded, the feed recomputes from state): while
/// `ai[1] < 120` the fade light scales by `ai[1] / 60` (`:35167`, the
/// post-increment value the arm reads), else the constant
/// (0.1, 0.5, 0.7) (`:35175`). Both sit at `((int)Center.X / 16,
/// (int)(Center.Y - 10) / 16)` — the C# truncates each quotient. RNG-free.
pub fn turret_light(ai1: f32, center: (f32, f32)) -> (i32, i32, [f32; 3]) {
    let rgb = if ai1 < 120.0 {
        let f = ai1 / 60.0;
        [0.1 * f, 0.5 * f, 0.7 * f]
    } else {
        [0.1, 0.5, 0.7]
    };
    (center.0 as i32 / 16, (center.1 - 10.0) as i32 / 16, rgb)
}

/// One fixed step of the aiStyle-73 arm (`NPC.cs:35102-35217`), in C#
/// order. `npc_type` is the vanilla id (387) until the roster kind
/// lands; `just_hit` is `this.justHit` for this tick (the port's `Npc`
/// has no such field — the style-10/16 convention); `net_mode` is
/// `Main.netMode` (0 single-player, 1 client, 2 server); `difficulty`
/// is `this.difficulty` (`GameDifficultyLevel` — 1 classic, 2 expert,
/// 3 master); `rng` is the caller's `Main.rand` stream, drawn from ONLY
/// at the shot's four jitters (`:35199-35202`, module docs).
///
/// The arm's early `return` (`:35168`) exits only `AI()` — the caller's
/// gravity/integration tail still runs, as in the C# (`NPC.cs:90423`+).
pub fn style73_step(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
    npc_type: i32,
    just_hit: bool,
    net_mode: i32,
    difficulty: f32,
    rng: &mut UnifiedRandom,
) -> Style73Outcome {
    let mut out = Style73Outcome::default();
    // `:35104` — TargetClosest(faceTarget: false): track, never face.
    // Style 73 takes no per-style arm in SetTargetTrackingValues (only
    // style 3 does, `NPC.cs:77761`).
    target_closest(npc, world, players, false, 73, ticker);
    // `:35105` — spriteDirection = base.direction.
    npc.sprite_dir = npc.facing;
    // `:35106-35110` — the X decay + the ±0.1 snap (strict inequalities).
    npc.vx *= 0.93;
    if npc.vx > -0.1 && npc.vx < 0.1 {
        npc.vx = 0.0;
    }
    // `:35111-35112` — player18 / flag74. The C# indexes
    // Main.player[this.target] without a bounds check; target_closest
    // guarantees 0..=254, and the port's slice carries the live slots
    // (an absent slot reads as inactive — the defensive seam).
    let player = players.get(npc.target as usize).copied();
    let (cx, cy) = npc.center();
    let flag74 = match player {
        Some(p) if p.active && !p.dead => {
            let pcx = p.x + p.width as f32 / 2.0;
            let pcy = p.y + p.height as f32 / 2.0;
            global_firing_distance_check(cx, cy, pcx, pcy)
        }
        _ => false,
    };
    // `:35113-35178` — the 387 materialize block (the census's only kind).
    if npc_type == 387 {
        let num1017 = 120.0f32;
        let num1018 = 60.0f32;
        if npc.ai[1] < num1017 {
            // `:35119` — ai[1]++ FIRST, then every read sees the new value.
            npc.ai[1] += 1.0;
            // `:35120-35127` — the alpha ramp (the C# `(int)` truncates).
            if npc.ai[1] > 60.0 {
                npc.alpha =
                    ((1.0 - (npc.ai[1] - num1018) / (num1017 - num1018)) * 255.0) as i32 as u8;
            } else {
                npc.alpha = 255;
            }
            // `:35128-35130` — invulnerable ghost, frame pinned.
            npc.dont_take_damage = true;
            // frameCounter = 0 / frame.Y = 0: `Npc::frame` is the row
            // (the counter is private — the wiring step pins it too,
            // the FindFrame pair function's input is 0 on fade ticks).
            npc.frame = 0;
            // `:35131-35166` — the dust ring (the count-only plan seam).
            out.materialize_dust = Some(turret_materialize_dust_plan(npc.ai[1], (cx, cy)));
            // `:35167` — the fading blue light, num1019-scaled.
            let num1019 = npc.ai[1] / num1018;
            out.light = Some((
                cx as i32 / 16,
                (cy - 10.0) as i32 / 16,
                [0.1 * num1019, 0.5 * num1019, 0.7 * num1019],
            ));
            // `:35168` — return (exits AI() only; the caller's tail runs).
            return out;
        }
        // `:35170-35174` — the one-shot 120 → 121 latch.
        if npc.ai[1] == num1017 {
            npc.ai[1] += 1.0;
            npc.net_update = true;
        }
        // `:35175-35177` — the solid state: constant light, hittable, opaque.
        out.light = Some((cx as i32 / 16, (cy - 10.0) as i32 / 16, [0.1, 0.5, 0.7]));
        npc.dont_take_damage = false;
        npc.alpha = 0;
    }
    // `:35179-35182` — the out-of-view hold re-arm.
    if npc.ai[0] == 0.0 && !flag74 {
        npc.ai[0] = -120.0;
    }
    // `:35183-35186` — the cycle increment.
    if npc.ai[0] < 60.0 {
        npc.ai[0] += 1.0;
    }
    // `:35187-35191` — the hit reset (AFTER the increment, as in the C#).
    if just_hit {
        npc.ai[0] = -30.0;
        npc.net_update = true;
    }
    // `:35192-35216` — the shot.
    if npc.ai[0] == 60.0 && net_mode != 1 {
        npc.ai[0] = -120.0;
        npc.net_update = true;
        // `:35196-35198` — the muzzle and the raw aim. The player read
        // re-fetches Main.player[this.target] (the same slot).
        let (pcx, pcy) = match player {
            Some(p) => (p.x + p.width as f32 / 2.0, p.y + p.height as f32 / 2.0),
            // Unreachable: ai[0] == 60 needs 60 increment ticks, and
            // the no-player path holds ai[0] at -120..-119 (the
            // :35179 re-arm fires every tick ai[0] hits 0). Defensive.
            None => (cx, cy),
        };
        let muzzle = (cx, cy - 10.0);
        let mut ax = pcx - muzzle.0;
        let mut ay = pcy - muzzle.1;
        // `:35199-35202` — the FOUR draws, in C# order.
        ax += rng.next_range(-100, 101) as f32;
        ay += rng.next_range(-100, 101) as f32;
        ax *= rng.next_range(70, 131) as f32 * 0.01;
        ay *= rng.next_range(70, 131) as f32 * 0.01;
        // `:35203-35207` — Vector2.Normalize() then the NaN fallback.
        // XNA's Normalize divides by Length() (sqrt(x²+y²)); a zero
        // vector yields NaN and takes the -UnitY branch.
        let len = (ax * ax + ay * ay).sqrt();
        ax /= len;
        ay /= len;
        if ax.is_nan() || ay.is_nan() {
            ax = 0.0;
            ay = -1.0;
        }
        // `:35208` — ×14.
        ax *= 14.0;
        ay *= 14.0;
        // `:35209-35214` — num1023 = 35, num1024 = 1 → 0.8 for the
        // Martian band 381..=392 (387 is inside; the census's only kind).
        let num1023 = 35.0f32;
        let mut num1024 = 1.0f32;
        if (381..=392).contains(&npc_type) {
            num1024 = 0.8;
        }
        // `:35215` — the bolt request (damage remap: NPC.cs:7052-7055).
        out.shot = Some(TurretBolt {
            projectile_type: 435,
            x: muzzle.0,
            y: muzzle.1,
            vx: ax,
            vy: ay,
            damage: attack_damage_for_projectiles(difficulty, num1023, num1023 * num1024),
            knock_back: 0.0,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A turret-shaped NPC: the roster has no MartianTurret kind yet,
    /// so the fixture rides a placeholder kind with the C# box through
    /// `size_override` and the 200-HP life (the style72 convention).
    fn turret(x: f32, y: f32) -> Npc {
        let d = &MARTIAN_TURRET;
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
        World::new("t".into(), 200, 120)
    }

    /// `NPC.cs:13922-13934` + the head defaults — the SetDefaults row,
    /// field by field.
    #[test]
    fn the_defaults_row_matches_the_csharp() {
        let d = &MARTIAN_TURRET;
        assert_eq!((d.width, d.height), (20, 50));
        assert_eq!(d.ai_style, 73);
        assert_eq!(d.damage, 10);
        assert_eq!(d.defense, 40);
        assert_eq!(d.life_max, 200);
        assert_eq!(d.hit_sound, (3, 53), "SoundID.NPCHit53");
        assert_eq!(d.death_sound, (4, 56), "SoundID.NPCDeath56");
        assert_eq!(d.knock_back_resist, 0.0, ":13932");
        assert!(!d.can_ghost_heal, ":13933");
        assert!(!d.no_gravity, ":8535 head default");
        assert!(!d.no_tile_collide, ":8522 head default");
        assert_eq!(d.value, 0.0, ":8544 head default");
        // Main.cs:67122 — the 380-389 row, index 387 = 2.
        assert_eq!(MARTIAN_TURRET_FRAME_COUNT, 2);
    }

    /// `NPC.cs:35106-35110` — the ×0.93 decay and the STRICT ±0.1 snap:
    /// exactly ±0.1 survives (the comparisons are `>` / `<`), inside
    /// snaps to zero.
    #[test]
    fn the_x_velocity_decays_and_snaps_inside_the_deadband() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0; // solid — skip the fade
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(1);
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.vx, 0.93, ":35106");
        // 0.1 decays to 0.093 — INSIDE the band, so it snaps (the C#
        // decays first, then tests the strict inequalities).
        n.vx = 0.1;
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.vx, 0.0, "0.093 is inside the band → snapped");
        // A value that decays to exactly the boundary survives: the
        // comparisons are strict, so only the open interval snaps.
        n.vx = -0.2;
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.vx, -0.2 * 0.93, "-0.186 is outside the band");
        n.vx = -0.09;
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.vx, 0.0, ":35109");
    }

    /// `NPC.cs:35117-35130` — the fade: ai[1] increments first, alpha
    /// is flat 255 through 60 then ramps, dontTakeDamage holds, the
    /// frame pins to row 0, and the arm RETURNS before the shot cycle.
    #[test]
    fn the_materialize_fade_pins_alpha_and_returns_early() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(7);
        // Tick 1: ai[1] 0 → 1, alpha 255, ai[0] untouched (early return).
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.alpha, 255, ":35126");
        assert!(n.dont_take_damage, ":35128");
        assert_eq!(n.frame, 0, ":35130");
        assert_eq!(n.ai[0], 0.0, "the shot cycle never ran (:35168 return)");
        assert!(out.shot.is_none());
        assert!(out.materialize_dust.is_some(), "the ring plan rides the fade");
        // Tick 60: ai[1] == 60 → still flat 255 (the ramp is `> 60`).
        n.ai[1] = 59.0;
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.alpha, 255, ":35120 is strict >");
        // Tick 61: (1 - 1/60) * 255 = 250.75 → (int) 250.
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.ai[1], 61.0);
        assert_eq!(n.alpha, 250, ":35122 truncation");
        // Tick 119: (1 - 59/60) * 255 = 4.25 → 4.
        n.ai[1] = 118.0;
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.alpha, 4);
    }

    /// `NPC.cs:35170-35177` — the 120 → 121 one-shot latch with
    /// netUpdate, then the solid state: hittable, alpha 0, constant
    /// light. The latch fires exactly once.
    #[test]
    fn the_solid_latch_fires_once_and_clears_the_ghost() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 119.0;
        let mut rng = UnifiedRandom::new(7);
        // The last fade tick: 119 → 120, still the ghost branch.
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.ai[1], 120.0);
        assert!(n.dont_take_damage);
        assert!(out.materialize_dust.is_some());
        // The latch tick: 120 → 121, netUpdate, solid.
        n.net_update = false;
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.ai[1], 121.0);
        assert!(n.net_update, ":35173");
        assert!(!n.dont_take_damage, ":35176");
        assert_eq!(n.alpha, 0, ":35177");
        assert!(out.materialize_dust.is_none());
        let (tx, ty, rgb) = out.light.expect("the solid light");
        // Centre (1010, 1025): (int)X/16 = 63, (int)(Y-10)/16 = 63.
        assert_eq!((tx, ty), (1010 / 16, 1015 / 16));
        assert_eq!(rgb, [0.1, 0.5, 0.7], ":35175");
        // The next tick: no second latch.
        n.net_update = false;
        style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.ai[1], 121.0, "the latch is one-shot");
    }

    /// `NPC.cs:35167` — the fade light scales by num1019 = ai[1]/60 at
    /// the truncated centre tile, 10 px up.
    #[test]
    fn the_fade_light_scales_with_the_spin_factor() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 29.0; // → 30, factor 0.5
        let mut rng = UnifiedRandom::new(7);
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        let (_, _, rgb) = out.light.expect("the fade light");
        assert_eq!(rgb, [0.1 * 0.5, 0.5 * 0.5, 0.7 * 0.5], ":35167");
    }

    /// `NPC.cs:35179-35186` — the out-of-view hold: ai[0] == 0 with a
    /// dead player re-arms to -120 then increments to -119; an in-view
    /// player lets the cycle climb. The firing check is the inflated
    /// 1920×1200 rect (NPC.cs:55835-55841).
    #[test]
    fn the_out_of_view_hold_rearms_the_cycle() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0;
        let mut rng = UnifiedRandom::new(7);
        // A dead player: flag74 false.
        let mut dead = cand(1100.0, 1000.0);
        dead.dead = true;
        style73_step(&mut n, &w, &[dead], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.ai[0], -119.0, "-120 re-arm then ++ (:35181/:35185)");
        // Far outside the rect: dx 2000 > 910 half-width.
        let mut n2 = turret(1000.0, 1000.0);
        n2.ai[1] = 121.0;
        style73_step(&mut n2, &w, &[cand(3000.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n2.ai[0], -119.0, "the distance check fails");
        // In view: the cycle climbs from 0.
        let mut n3 = turret(1000.0, 1000.0);
        n3.ai[1] = 121.0;
        style73_step(&mut n3, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n3.ai[0], 1.0);
    }

    /// `NPC.cs:35187-35191` — justHit resets ai[0] to -30 AFTER the
    /// increment and arms netUpdate, even on the would-be shot tick.
    #[test]
    fn just_hit_resets_the_cycle_after_the_increment() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0;
        n.ai[0] = 59.0; // would reach 60 and fire
        let mut rng = UnifiedRandom::new(7);
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, true, 0, 1.0, &mut rng);
        assert_eq!(n.ai[0], -30.0, ":35189");
        assert!(n.net_update, ":35190");
        assert!(out.shot.is_none(), "the hit reset precedes the shot test");
    }

    /// `NPC.cs:35192-35215` — the shot at ai[0] == 60: the -120 re-arm,
    /// the muzzle 10 px above centre, the ×14 normalized aim, damage 35
    /// classic / 28 expert (the 0.8 Martian band), kb 0, type 435.
    #[test]
    fn the_shot_fires_at_sixty_with_the_martian_damage_band() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0;
        n.ai[0] = 59.0;
        let mut rng = UnifiedRandom::new(42);
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        let shot = out.shot.expect("the 60-tick shot");
        assert_eq!(n.ai[0], -120.0, ":35194");
        assert!(n.net_update);
        assert_eq!(shot.projectile_type, 435, "MartianTurretBolt");
        // Centre (1010, 1025) → muzzle (1010, 1015).
        assert_eq!((shot.x, shot.y), (1010.0, 1015.0), ":35197");
        assert_eq!(shot.damage, 35, "classic: Remap(1, 1..2, 35, 28) = 35");
        assert_eq!(shot.knock_back, 0.0);
        // The velocity is the jittered aim ×14 — length 14 up to the
        // f32 normalize rounding.
        let len = (shot.vx * shot.vx + shot.vy * shot.vy).sqrt();
        assert!((len - 14.0).abs() < 0.001, "the ×14 normalized aim, got {len}");
        // Expert: 35 * 0.8 = 28.
        let mut n2 = turret(1000.0, 1000.0);
        n2.ai[1] = 121.0;
        n2.ai[0] = 59.0;
        let out2 = style73_step(&mut n2, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 2.0, &mut rng);
        assert_eq!(out2.shot.unwrap().damage, 28, ":35213 the 0.8 band");
        // Master clamps at expert (the Remap clamp, Utils.cs:317-342).
        let mut n3 = turret(1000.0, 1000.0);
        n3.ai[1] = 121.0;
        n3.ai[0] = 59.0;
        let out3 = style73_step(&mut n3, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 3.0, &mut rng);
        assert_eq!(out3.shot.unwrap().damage, 28);
    }

    /// `NPC.cs:35192` — the client (netMode 1) never fires: ai[0] hits
    /// 60 and stays (the increment gate is `< 60`), no draws consumed.
    #[test]
    fn the_client_never_fires() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0;
        n.ai[0] = 59.0;
        let mut rng = UnifiedRandom::new(42);
        let mut oracle = UnifiedRandom::new(42);
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 1, 1.0, &mut rng);
        assert!(out.shot.is_none());
        assert_eq!(n.ai[0], 60.0, "the cycle parks at 60 on the client");
        assert_eq!(rng.next(), oracle.next(), "no draws on the client path");
    }

    /// `NPC.cs:35199-35202` — the RNG witness: the shot consumes
    /// EXACTLY four draws, in the C# order (X jitter, Y jitter, X scale,
    /// Y scale), against a mirrored oracle stream; the aim reproduces
    /// the oracle's values bit-for-bit.
    #[test]
    fn the_shot_consumes_the_four_draws_in_order() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0;
        n.ai[0] = 59.0;
        let mut rng = UnifiedRandom::new(1234);
        let mut oracle = UnifiedRandom::new(1234);
        let out = style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        let shot = out.shot.expect("the shot");
        // Replay the C# expression with the oracle's four draws.
        let jx = oracle.next_range(-100, 101) as f32;
        let jy = oracle.next_range(-100, 101) as f32;
        let sx = oracle.next_range(70, 131) as f32 * 0.01;
        let sy = oracle.next_range(70, 131) as f32 * 0.01;
        // Player centre (1110, 1021), muzzle (1010, 1015).
        let mut ax = (1110.0 - 1010.0) + jx;
        let mut ay = (1021.0 - 1015.0) + jy;
        ax *= sx;
        ay *= sy;
        let len = (ax * ax + ay * ay).sqrt();
        ax = ax / len * 14.0;
        ay = ay / len * 14.0;
        assert_eq!(shot.vx, ax, "the X chain matches the oracle");
        assert_eq!(shot.vy, ay, "the Y chain matches the oracle");
        assert_eq!(rng.next(), oracle.next(), "exactly four draws consumed");
    }

    /// `NPC.cs:35203-35207` — the NaN fallback: a zero aim vector
    /// (player centre exactly on the muzzle, all four jitters/scales
    /// landing zero — unreachable with the 70..130 scale band, so the
    /// normalize helper is pinned through a constructed zero) takes
    /// -UnitY ×14. The witness drives the pure normalize chain.
    #[test]
    fn the_nan_aim_falls_back_to_negative_unit_y() {
        // The arm's chain, spelled out: Normalize of (0,0) is NaN.
        let (mut ax, mut ay) = (0.0f32, 0.0f32);
        let len = (ax * ax + ay * ay).sqrt();
        ax /= len;
        ay /= len;
        assert!(ax.is_nan() && ay.is_nan());
        if ax.is_nan() || ay.is_nan() {
            ax = 0.0;
            ay = -1.0;
        }
        ax *= 14.0;
        ay *= 14.0;
        assert_eq!((ax, ay), (0.0, -14.0), ":35206 -Vector2.UnitY");
    }

    /// `NPC.cs:35104-35105` — TargetClosest(faceTarget: false): the
    /// target is tracked but the facing is NOT written by the pick;
    /// spriteDirection mirrors the (unchanged) direction.
    #[test]
    fn the_pick_tracks_without_facing() {
        let w = air();
        let mut n = turret(1000.0, 1000.0);
        n.ai[1] = 121.0;
        n.facing = -1;
        n.old_target = 255;
        let mut rng = UnifiedRandom::new(7);
        // The player is to the RIGHT — a facing pick would flip to +1.
        style73_step(&mut n, &w, &[cand(1200.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        assert_eq!(n.target, 0, "the pick still tracks");
        assert_eq!(n.facing, -1, "faceTarget: false (:35104)");
        assert_eq!(n.sprite_dir, -1, ":35105 mirrors direction");
    }

    /// `NPC.cs:71663-71696` — FindFrame case 387: the idle 15-tick
    /// flip, the charged 10-tick flip with the 20/40 extra increments
    /// (3/tick at ai[0] >= 40 → a flip every 4 ticks: 10/3 ceil).
    #[test]
    fn the_frame_strip_flips_at_the_csharp_cadences() {
        // Idle: 15 ticks per flip over the 2-row sheet.
        let (mut row, mut counter) = (0u32, 0.0f32);
        for _ in 0..14 {
            (row, counter) = turret_find_frame(row, counter, 0.0);
        }
        assert_eq!((row, counter), (0, 14.0));
        (row, counter) = turret_find_frame(row, counter, 0.0);
        assert_eq!((row, counter), (1, 0.0), "the 15th tick flips");
        (row, counter) = turret_find_frame(row, counter, -30.0);
        assert_eq!(row, 1, "negative ai[0] rides the idle strip");
        // Charged below 20: 1/tick, flip every 10.
        let (mut row, mut counter) = (0u32, 0.0f32);
        for _ in 0..10 {
            (row, counter) = turret_find_frame(row, counter, 1.0);
        }
        assert_eq!((row, counter), (1, 0.0));
        // At 20..39: 2/tick, flip every 5.
        let (mut row, mut counter) = (0u32, 0.0f32);
        for _ in 0..5 {
            (row, counter) = turret_find_frame(row, counter, 20.0);
        }
        assert_eq!((row, counter), (1, 0.0));
        // At >= 40: 3/tick — ticks 1..3 reach 9, tick 4 hits 12 → flip.
        let (mut row, mut counter) = (0u32, 0.0f32);
        for _ in 0..3 {
            (row, counter) = turret_find_frame(row, counter, 59.0);
        }
        assert_eq!((row, counter), (0, 9.0));
        (row, counter) = turret_find_frame(row, counter, 59.0);
        assert_eq!((row, counter), (1, 0.0), "the >= 10 reset (:71675)");
        // The wrap: row 1 → 0 at the second flip (:71679-71682).
        for _ in 0..4 {
            (row, counter) = turret_find_frame(row, counter, 59.0);
        }
        assert_eq!(row, 0, "the 2-row wrap");
    }

    /// `NPC.cs:35131-35166` — the ring plan: at ai[1] = 60 the spin
    /// angle is 1.5 × 2π (a full 1.5 turns), the vector is (0,-30)
    /// rotated then y-scaled ×0.4, and the eight placements alternate
    /// the mirror per pair. At factor 1 the rotation is 3π ≡ π:
    /// (0, -30) → (0, 30) → ×(1, 0.4) → (0, 12).
    #[test]
    fn the_ring_plan_mirrors_the_spin_per_pair() {
        let plan = turret_materialize_dust_plan(60.0, (1000.0, 1000.0));
        assert_eq!(plan.spin_factor, 1.0);
        // cos(3π) = -1, sin(3π) ≈ 0 (the f32 angle widened to f64 —
        // the tiny sin residue is the C#'s own rounding, kept).
        assert!((plan.spin.0 - 0.0).abs() < 0.001);
        assert!((plan.spin.1 - 12.0).abs() < 0.001, "the ×0.4 y scale");
        // Pair 0 (offset (0,-15), scale 0.15): first at +spin*0.15,
        // then the mirrored twin at -spin*0.15.
        let p0 = plan.placements[0];
        let p1 = plan.placements[1];
        assert!((p0.1 - (1000.0 + plan.spin.1 * 0.15 - 15.0)).abs() < 0.001);
        assert!((p1.1 - (1000.0 - plan.spin.1 * 0.15 - 15.0)).abs() < 0.001);
        // Pair 3 (offset (0,20), scale 0.45) — after three end-of-pair
        // flips the carried vector is -spin, so the FIRST dust of the
        // pair is the mirrored one and the twin is +spin.
        let p6 = plan.placements[6];
        let p7 = plan.placements[7];
        assert!((p6.1 - (1000.0 - plan.spin.1 * 0.45 + 20.0)).abs() < 0.001);
        assert!((p7.1 - (1000.0 + plan.spin.1 * 0.45 + 20.0)).abs() < 0.001);
        // The mirror alternates: pair 1 starts from -spin (the C#
        // multiplies vector124 by -1 at the END of each pair).
        let p2 = plan.placements[2];
        assert!((p2.1 - (1000.0 - plan.spin.1 * 0.3 - 5.0)).abs() < 0.001);
    }

    /// `NPC.cs:35167`/`:35175` — the draw-side light feed: the fade
    /// scales by ai[1]/60 below 120, then the constant (0.1, 0.5, 0.7);
    /// both truncate EACH quotient ((int)Center.X / 16, not (int)(X/16)).
    #[test]
    fn the_light_feed_fades_then_pins_the_constant() {
        // Fade at ai[1] = 30: factor 0.5; centre (1000.9, 1000.9) →
        // (int)1000.9/16 = 62, (int)990.9/16 = 61.
        let (x, y, rgb) = turret_light(30.0, (1000.9, 1000.9));
        assert_eq!((x, y), (62, 61));
        assert_eq!(rgb, [0.1 * 0.5, 0.5 * 0.5, 0.7 * 0.5]);
        // The last fade tick (119) still scales; 120 pins the constant.
        let f = 119.0f32 / 60.0;
        let (_, _, rgb) = turret_light(119.0, (0.0, 0.0));
        assert_eq!(rgb, [0.1 * f, 0.5 * f, 0.7 * f]);
        let (x, y, rgb) = turret_light(120.0, (32.9, 26.5));
        assert_eq!((x, y, rgb), (2, 1, [0.1, 0.5, 0.7]), ":35175 solid");
        // Negative-centre truncation toward zero: (int)(-0.5) = 0 and
        // (int)(-10.5)/16 = -10/16 = 0 (C# integer division truncates).
        let (x, y, _) = turret_light(121.0, (-0.5, -0.5));
        assert_eq!((x, y), (0, 0));
    }

    /// The RNG-by-construction witness: every non-shot path (the fade,
    /// the solid idle, the out-of-view hold, the client) consumes ZERO
    /// draws — the `UnifiedRandom` state is pinned across the calls.
    #[test]
    fn the_non_shot_paths_draw_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(99);
        let mut oracle = UnifiedRandom::new(99);
        let mut n = turret(1000.0, 1000.0);
        // 130 ticks: the whole fade + latch + the idle climb (the
        // in-view cycle reaches 60 at tick ~181 from 0 — never here).
        for _ in 0..130 {
            style73_step(&mut n, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
        }
        assert_eq!(rng.next(), oracle.next(), "130 fade/idle ticks drew nothing");
        // The out-of-view hold and the client park: also draw-free.
        let mut n2 = turret(1000.0, 1000.0);
        n2.ai[1] = 121.0;
        // ai[0] = 0: the out-of-view hold re-arms to -120 every time
        // the cycle reaches 0, so it never climbs to the 60 trigger.
        n2.ai[0] = 0.0;
        for _ in 0..10 {
            style73_step(&mut n2, &w, &[cand(3000.0, 1000.0)], 0, 387, false, 0, 1.0, &mut rng);
            style73_step(&mut n2, &w, &[cand(1100.0, 1000.0)], 0, 387, false, 1, 1.0, &mut rng);
        }
        assert_eq!(rng.next(), oracle.next(), "the hold/client paths drew nothing");
    }
}
