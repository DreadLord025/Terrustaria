//! S1-P-i17 — the special arrows' shoot/hit effects: `AI_001`'s per-type
//! arrow arms (`Projectile.cs:54002-57798`), `StatusNPC`'s per-type debuff
//! arms (`:11173-11725`), `Damage_PVE_Inner`'s arrow hits (`:13252-13255`),
//! the `Kill()` death spawns (`:75212-75248` Holy Arrow star, `:76308-76341`
//! Hellfire blast, `:76407-76429` Bee Arrow swarm) and the Chlorophyte
//! bounce (`HandleMovement` `:18539-18561`).
//!
//! Kind set ([`ARROW_KINDS`]): 4 Unholy / 5 Jester's / 41 Hellfire / 91 Holy /
//! 103 Cursed / 117 Bone / 120 Frost / 172 Frostburn / 225 Chlorophyte /
//! 278 Ichor / 282 Venom / 469 Bee / 474 Bone-from-merchant / 495
//! Shadowflame / 1006 Shimmer. Disjoint from the ai001 A/B/C kind lists,
//! the riders (20/88/355/660/110), the frost shots, the empress kinds and
//! the moon-lord drop shots (638/639/640 keep `projectile_ai_moonlord`).
//! 1/2/81/82 (the plain wooden/flame arrows and their hostile twins) keep
//! their existing arms: the generic flight IS their C# arm (the slice-C
//! census escape clause), and their effects live in [`status_npc`] (the
//! flame arrow's OnFire roll) and the pool (the 469 wet kill needs world
//! access, wired at the dispatch).
//!
//! Structure mirrors `AI_001`'s own order (the ai001_b/c precedent):
//! 1. the hook group above the counter (469's fade `:55197-55208`),
//! 2. the `flag3` counter switch (`:55945-56041` — 5 `:55949` and 469
//!    `:56002` REST; 1006 sets `flag4` `:56034-56036`),
//! 3. the gravity else-if chain: 91 joins 81's 20-tick/+0.07 arm
//!    (`:56888-56895`), 172's 17/+0.085 (`:57233-57240`), 117's
//!    35/+0.06 (`:57241-57248` — the chain names ONLY 117; the 474
//!    falls to the generic 15/+0.1 tail), 120's 30/+0.05
//!    (`:57249-57259`), 495's 30/+0.04 (`:57479-57489`),
//!    the generic 15/+0.1 with the `flag4` inversion (`:57568-57579`),
//! 4. the rotation tail (469's spriteDirection arm `:57671-57683`, the
//!    rest the default `:57750`),
//! 5. the `flag8` clamp (`:57753-57768` — the C#'s exact bounds: −16 only
//!    for `flag4` kinds, +16 for all), then the movement stand-in.
//!
//! RNG: the module takes NO rng parameter, so the `UpdateProjectiles`
//! stream (`Main.cs:17775-17777`) is untouched BY CONSTRUCTION for the
//! flight. Every `Main.rand` draw in these kinds' C# AI blocks feeds
//! dust/gore only and is SKIPPED with the dust layer (the ai001_a/b/c
//! convention, cited at each point):
//! - 5's rainbow dust `Next(3)` type pick (`:17071-17076`, the Update
//!   chain), 4's 1/5 dust gate (`:17064`), 41's two dusts (no draws,
//!   `:55483-55489`), 91's 1/2 + 1/2 dust gates (`:55587-55590`), 103's
//!   1/2 dust gate (`:17047`), 278's 1/2 dust gate (`:17056`), 282's
//!   `Next(1,10)` dust scale (`:55006`), 225's 1/2 dust gate (`:55025`),
//!   120's dust (no explicit draws, `:57251`), 495's dust (no explicit
//!   draws, `:57481`), 819 keeps the generic flight (its own arm
//!   `:56388-56422` is dust/alpha/light only — ANALYSIS records the
//!   deferral; the Blood Rain Bow's rain is a SHOOT-side effect,
//!   `Player.cs:47466-47499`), 1006's 1/8 dust gate +
//!   the burst's floats (`:56042-56055`).
//! The STATUS (hit-time) and KILL (death-time) halves DO draw — their
//! rolls are gameplay (debuff durations, spawn positions, bee counts) and
//! ride the caller's `UpdateProjectiles` stream in the C#'s exact order;
//! each function's doc lists its draws.
//!
//! Movement is the port's documented kill-on-tile stand-in
//! (`HandleMovement`'s default arm `:19843-19855`): a blocked move takes
//! one last full-velocity step and kills (`kill_step`, projectile.rs).
//! The 225 Chlorophyte bounce substitutes the C#'s `TileCollision`
//! blocked-axis detection with the port's per-axis destination probe
//! (the same stand-in every arrow arm uses); the bounce STATE machine
//! (ai[1] counter, ×0.66 first-bounce damage, 4-bounce kill, retarget
//! lead) is verbatim.
//!
//! CODE-ONLY MODE (S1-P-i17): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); they were verified by
//! re-reading the cited C# blocks and hand arithmetic.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, BEE, BEE_ARROW, BONE_ARROW, BONE_ARROW_MERCHANT, CHLOROPHYTE_ARROW, CURSED_ARROW,
    FALLING_STAR, FLAME_ARROW, FROSTBURN_ARROW, FROST_ARROW, GIANT_BEE, HELLFIRE_ARROW, HOLY_ARROW,
    ICHOR_ARROW, JESTERS_ARROW, SHADOWFLAME_ARROW, SHIMMER_ARROW, UNHOLY_ARROW, VENOM_ARROW,
};

/// The aiStyle-1 per-type kind set this module owns. Disjoint from
/// `projectile_ai001_a::SLICE_A_KINDS`, `_b::SLICE_B_KINDS`,
/// `_c::SLICE_C_KINDS`, `AI_STYLE_1_RIDERS`, the frost/empress/moon-lord
/// gates. 1/2 keep the generic flight (their C# arm IS the generic tail);
/// 81/82 ride slice A; 639/640 ride `projectile_ai_moonlord`.
pub(crate) const ARROW_KINDS: [u16; 15] = [
    UNHOLY_ARROW,        // 4
    JESTERS_ARROW,       // 5
    HELLFIRE_ARROW,      // 41
    HOLY_ARROW,          // 91
    CURSED_ARROW,        // 103
    BONE_ARROW,          // 117
    FROST_ARROW,         // 120
    FROSTBURN_ARROW,     // 172
    CHLOROPHYTE_ARROW,   // 225
    ICHOR_ARROW,         // 278
    VENOM_ARROW,         // 282
    BEE_ARROW,           // 469
    BONE_ARROW_MERCHANT, // 474
    SHADOWFLAME_ARROW,   // 495
    SHIMMER_ARROW,       // 1006
];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    ARROW_KINDS.contains(&kind)
}

/// The per-kind gravity arm of the `:56888-57580` else-if chain.
enum Gravity {
    /// `flag3 = false` (`:55949` the Jester's 5, `:56002` the Bee 469):
    /// the counter rests at 0, no gravity arm fires — straight flight.
    Rest,
    /// `:56888-56895` — the 81/91 arm: `if (ai[0] >= 20f) { ai[0] = 20f;
    /// velocity.Y += 0.07f; }` (the Holy Arrow shares the hostile wooden
    /// arrow's softer gravity).
    Holy,
    /// `:57233-57240` — the Frostburn 172: pin 17, +0.085.
    Frostburn,
    /// `:57241-57248` — the Bone 117: pin 35, +0.06.
    Bone,
    /// `:57249-57259` — the Frost 120: pin 30, +0.05 (its dust-67 trail
    /// `:57251-57253` is the documented skip).
    Frost,
    /// `:57479-57489` — the Shadowflame 495: pin 30, +0.04 (the dust-27
    /// trail `:57481-57483` skipped).
    Shadowflame,
    /// `:57568-57579` — the generic arrow tail: pin 15, +0.1; `flag4`
    /// (the Shimmer 1006, `:56034-56036`) INVERTS it to −0.1.
    Generic { inverted: bool },
}

fn gravity_of(kind: u16) -> Gravity {
    match kind {
        JESTERS_ARROW | BEE_ARROW => Gravity::Rest,
        HOLY_ARROW => Gravity::Holy,
        FROSTBURN_ARROW => Gravity::Frostburn,
        BONE_ARROW => Gravity::Bone,
        FROST_ARROW => Gravity::Frost,
        SHADOWFLAME_ARROW => Gravity::Shadowflame,
        SHIMMER_ARROW => Gravity::Generic { inverted: true },
        // 4/41/103/225/278/282/474 — no own arm in the C# chain, the
        // generic tail serves them (474 is NOT named at `:57241` — only
        // 117 is).
        _ => Gravity::Generic { inverted: false },
    }
}

/// `AI_001` for one special-arrow kind, in the C#'s own block order.
/// The 225's bounce needs the NPC scan, so the whole step takes `npcs`
/// (the ai001_c precedent); no other kind reads it. W4-f20 F7: `immune`
/// feeds the Harpy-charm tail (`:57769-57793`) — the special arrows ARE
/// charm-eligible (the producer's `arrow` gate).
pub(crate) fn step(p: &mut Projectile, world: &World, npcs: &[Npc], immune: &[i32]) {
    // ---- 1. The hook group above the counter ------------------------------
    // `:55197-55208` — the Bee Arrow (469): `localAI[1]++`, then from the
    // 3rd sub-step the alpha loses 50 with the 0 floor (the row seeds
    // alpha 255, `:5437`).
    if p.kind == BEE_ARROW {
        p.local_ai[1] += 1.0;
        if p.local_ai[1] > 2.0 {
            p.alpha -= 50;
            if p.alpha < 0 {
                p.alpha = 0;
            }
        }
    }
    // The 469's water kill (`:54012-54015`, `wet && !honeyWet &&
    // !shimmerWet`) needs the world's liquid flags — it rides the pool's
    // dispatch seam (`bee_water_kill` below), NOT the flight step.

    // ---- 2. The flight counter (`:55945-56041`) ---------------------------
    let count_flight = !matches!(p.kind, JESTERS_ARROW | BEE_ARROW);
    if count_flight {
        p.ai[0] += 1.0;
    }

    // ---- 3. The gravity else-if chain -------------------------------------
    match gravity_of(p.kind) {
        Gravity::Rest => {}
        Gravity::Holy => {
            if p.ai[0] >= 20.0 {
                p.ai[0] = 20.0;
                p.vel.y += 0.07;
            }
        }
        Gravity::Frostburn => {
            if p.ai[0] >= 17.0 {
                p.ai[0] = 17.0;
                p.vel.y += 0.085;
            }
        }
        Gravity::Bone => {
            if p.ai[0] >= 35.0 {
                p.ai[0] = 35.0;
                p.vel.y += 0.06;
            }
        }
        Gravity::Frost => {
            if p.ai[0] >= 30.0 {
                p.ai[0] = 30.0;
                p.vel.y += 0.05;
            }
        }
        Gravity::Shadowflame => {
            if p.ai[0] >= 30.0 {
                p.ai[0] = 30.0;
                p.vel.y += 0.04;
            }
        }
        Gravity::Generic { inverted } => {
            if p.ai[0] >= 15.0 {
                p.ai[0] = 15.0;
                if inverted {
                    p.vel.y -= 0.1;
                } else {
                    p.vel.y += 0.1;
                }
            }
        }
    }

    // ---- 4. The rotation tail (`:57582-57752`) ----------------------------
    let angle = p.vel.y.atan2(p.vel.x);
    if p.kind == BEE_ARROW {
        // `:57671-57683` — both branches compute the SAME rotation; only
        // the spriteDirection differs (the C# spells both arms out).
        p.sprite_direction = if p.vel.x > 0.0 { -1 } else { 1 };
        p.rot = angle + 1.57;
    } else {
        // `:57748-57751` — the default: `Atan2(vy, vx) + 1.57f` (the
        // literal 1.57, not π/2 — the C# spells the constant).
        p.rot = angle + 1.57;
    }

    // ---- 5. The `flag8` clamp (`:57753-57768`) ----------------------------
    // The C#'s exact bounds: the −16 clamp fires ONLY for `flag4` kinds
    // (the 1006); the +16 clamp fires for every kind. The legacy generic
    // flight's symmetric ±16 (projectile.rs `ai_style1_flight`) is NOT
    // copied here — the per-type arms take the exact C# bounds like
    // slices A/C.
    if p.kind == SHIMMER_ARROW && p.vel.y < -16.0 {
        p.vel.y = -16.0;
    }
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }

    // ---- 5b. The Harpy-charm tail (`:57769-57793`, W4-f20 F7) -----------
    // After the clamp, before the movement — the C#'s own slot.
    crate::projectile_harpy_charm::charm_tail(p, world, npcs, immune);

    // ---- 6. Movement -------------------------------------------------------
    if p.kind == CHLOROPHYTE_ARROW {
        chlorophyte_move(p, world, npcs);
    } else {
        // The `HandleMovement` default arm (`:19843-19855`): free move
        // steps forward; a blocked move takes one last full-velocity step
        // and kills (the port's `kill_step` shape, replicated so the
        // module keeps the ai001 modules' no-pool-dependency convention).
        if p.tile_collide {
            let last = p.vel;
            let hit_x =
                crate::player::collides_rect(world, p.pos.x + last.x, p.pos.y, p.w, p.h);
            let hit_y =
                crate::player::collides_rect(world, p.pos.x, p.pos.y + last.y, p.w, p.h);
            p.pos += p.vel;
            if hit_x || hit_y {
                p.kill_time_left = p.time_left;
                p.active = false;
                p.time_left = 0;
            }
        } else {
            p.pos += p.vel;
        }
    }
}

/// The Bee Arrow's water kill (`Projectile.cs:54012-54015`): `type == 469
/// && base.wet && !base.honeyWet && !base.shimmerWet` → `Kill()`. The
/// port probes the tile liquid flags directly (the pool carries no wet
/// state — the `is_wet` precedent): any overlapping liquid tile arms
/// `wet`; a HONEY or SHIMMER tile suppresses the kill (the C# flags are
/// per-tick sticky, the port's probe is instantaneous — a bee leaving
/// honey into water kills a tick earlier than the C#'s flag decay;
/// recorded in ANALYSIS). Lava does NOT suppress (`!honeyWet &&
/// !shimmerWet` only — a lava-wet bee dies, the C# arm names no lava).
pub(crate) fn bee_water_kill(p: &Projectile, world: &World) -> bool {
    if p.kind != BEE_ARROW {
        return false;
    }
    let x0 = (p.pos.x / 16.0).floor() as i32;
    let x1 = ((p.pos.x + p.w - 0.01) / 16.0).floor() as i32;
    let y0 = (p.pos.y / 16.0).floor() as i32;
    let y1 = ((p.pos.y + p.h - 0.01) / 16.0).floor() as i32;
    let mut wet = false;
    let mut honey = false;
    let mut shimmer = false;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if let Some(t) = world.tile(tx, ty) {
                if t.liquid > 0 {
                    wet = true;
                    if t.flags & tr_core::tile::FLAG_LIQUID_HONEY != 0 {
                        honey = true;
                    }
                    if t.flags & tr_core::tile::FLAG_LIQUID_SHIMMER != 0 {
                        shimmer = true;
                    }
                }
            }
        }
    }
    wet && !honey && !shimmer
}

/// The Chlorophyte Arrow's bounce (`HandleMovement` `:18539-18561`): on a
/// blocked move, `ai[1]++`; the FIRST bounce scales the damage ×0.66
/// (`(int)((float)damage * 0.66f)`), the FOURTH kills; the velocity
/// reverses (`0f - lastVelocity`), then `FindTargetWithLineOfSight`
/// (default range 800, `:45627-45660`) retargets: the aim leads ABOVE the
/// NPC's centre by `-UnitY * Lerp(h*0.1, h*0.5, GetLerpValue(0, 300,
/// dist, clamped))` — the near target gets the low lead, the far one the
/// high — normalized (`SafeNormalize(-UnitY)`) to the reversed speed.
/// A free move integrates normally.
fn chlorophyte_move(p: &mut Projectile, world: &World, npcs: &[Npc]) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last = p.vel;
    let hit_x = crate::player::collides_rect(world, p.pos.x + last.x, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, p.pos.y + last.y, p.w, p.h);
    if !hit_x && !hit_y {
        p.pos += p.vel;
        return;
    }
    // `:18541-18550` — the bounce counter, the first-bounce damage cut,
    // the 4-bounce kill.
    p.ai[1] += 1.0;
    if p.ai[1] == 1.0 {
        p.damage = (p.damage as f32 * 0.66) as i32;
    }
    if p.ai[1] >= 4.0 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:18551-18552` — the full reversal (the C# writes `0f - last`, not
    // a per-axis conditional — both axes flip unconditionally).
    p.vel = -last;
    // `:18553-18560` — the retarget. The port's blocked-axis probe stands
    // in for `TileCollision`'s velocity rewrite, so `last` is the
    // pre-collision velocity (the same stand-in the movement arms use).
    let target = crate::projectile::find_target_with_line_of_sight_pub(
        world, npcs, p.pos, p.w, p.h, 800.0, &[],
    );
    if target != -1 {
        let n = &npcs[target as usize];
        let (nw, nh) = n.size();
        let nc = Vec2::new(n.x + nw * 0.5, n.y + nh * 0.5);
        let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5);
        let t = center.distance(nc);
        // `Utils.GetLerpValue(0f, 300f, t)` clamped (`:18558`).
        let lerp = crate::projectile::get_lerp_value(0.0, 300.0, t);
        let lead = nh * 0.1 + (nh * 0.5 - nh * 0.1) * lerp;
        let aim = nc + Vec2::new(0.0, -lead) - center;
        let speed = p.vel.length();
        // `SafeNormalize(-Vector2.UnitY)` — a zero aim falls back to up.
        let dir = if aim.length() > 0.0 {
            aim / aim.length()
        } else {
            Vec2::new(0.0, -1.0)
        };
        p.vel = dir * speed;
    }
    // `UpdatePosition(wetVelocity)` (`:19858` → `:20433-20446`) runs
    // UNCONDITIONALLY after the collision chain. Its DRY arm (`:20443-
    // 20445`, `!base.wet`) integrates `base.velocity` — the POST-bounce
    // reversed/retargeted velocity, NOT the pre-collision `wetVelocity`
    // (that is only read on the `base.wet` arm `:20439-20441`, and an
    // arrow in open air is dry). So the bounce tick steps the NEW
    // (reversed/retargeted) direction immediately. The port's `p.vel`
    // already holds that velocity (the reversal + retarget above), so the
    // integration is `p.pos += p.vel`.
    p.pos += p.vel;
}

// ---------------------------------------------------------------------------
// StatusNPC — the hit-time debuffs (`Projectile.cs:11173-11725`)
// ---------------------------------------------------------------------------

/// The arrow-family arms of `StatusNPC`, in the C#'s own statement order.
/// Called from `damage_pve`'s landed block at the `StatusNPC(victimIndex)`
/// site (`:13580`) — AFTER the strike, BEFORE the penetrate decrement.
/// Only the arms whose kinds the port rows are ported; the accessory
/// chains (`accPoisonBarb` `:11180`, `meleeEnchant` `:11192`, the
/// `frostBurn`/`magmaStone` set bonuses `:11518-11536`) are gated on
/// player flags the port carries — the melee-only ones never fire for an
/// arrow (`this.melee` false) and are skipped by construction.
///
/// RNG draws, in order (all on the caller's `UpdateProjectiles` stream):
/// 1. 495 Shadowflame: `Next(120, 300)` — the ShadowFlame duration
///    (`:11460`).
/// 2. 2 FlameArrow: `Next(3)` — the 1/3 OnFire roll (`:11552`).
/// 3. 172 Frostburn: `Next(3)` — the 1/3 Frostburn roll (`:11558`).
///    NOTE the C# shape: the 172 test OPENS an else-if chain (`:11556-
///    11677`) whose later arms include 103/278/282 — so for those three
///    kinds the 172/15/253/… tests all evaluate FALSE without drawing,
///    and their own arm's fixed-duration AddBuff runs draw-free.
/// 4. 282 Venom: no draw (the fixed 600/120 pair, `:11673-11677`).
/// The flame arrow's roll is INDEPENDENT of the 172 chain (a separate
/// `if`, `:11552`), so a FlameArrow hit draws `Next(3)` once; a Frostburn
/// hit draws `Next(3)` once; Cursed/Ichor/Venom hits draw nothing.
pub(crate) fn status_npc(kind: u16, npc: &mut Npc, rng: &mut dyn DropRng) {
    // `:11458-11461` — the Shadowflame Arrow: ShadowFlame (153) for
    // `Next(120, 300)` ticks.
    if kind == SHADOWFLAME_ARROW {
        let time = rng.next_range(120, 300);
        npc.buffs.add_buff(153, time);
    }
    // `:11552-11555` — the FlameArrow (2, the Flaming Arrow item's shot
    // and the Molten Fury/Molten Quiver conversion): 1/3 OnFire (24) 180.
    if kind == FLAME_ARROW && rng.next_range(0, 3) == 0 {
        npc.buffs.add_buff(24, 180);
    }
    // `:11556-11677` — the else-if chain. The ported arms:
    if kind == FROSTBURN_ARROW {
        // `:11556-11562` — 1/3 Frostburn (44) 180.
        if rng.next_range(0, 3) == 0 {
            npc.buffs.add_buff(44, 180);
        }
    } else if kind == CURSED_ARROW {
        // `:11665-11668` — the 95/103/104/482 group: CursedInferno (39)
        // 420, unconditional.
        npc.buffs.add_buff(39, 420);
    } else if kind == ICHOR_ARROW {
        // `:11669-11672` — the 278/279/280 group: Ichor (69) 600.
        npc.buffs.add_buff(69, 600);
    } else if kind == VENOM_ARROW {
        // `:11673-11677` — the 282/283 group: Venom (70) 600 AND
        // PotentAcid (395) 120.
        npc.buffs.add_buff(70, 600);
        npc.buffs.add_buff(395, 120);
    }
    // The wooden/unholy/jester/holy/hellfire/bone/frost/chlorophyte/bee/
    // shimmer arrows have NO StatusNPC arm — their effects are the pierce
    // counts (rows), the death spawns (below) and the flight quirks.
}

/// `Damage_PVE_Inner`'s Hellfire clamp (`Projectile.cs:13252-13255`):
/// `if (type == 41 && timeLeft > 1) timeLeft = 1` — the FIRST landed hit
/// arms the arrow to die at the next lifetime decrement, which fires the
/// Kill() blast. Runs BEFORE the strike (the C# site sits above the
/// `StrikeNPC` call at `:13649`).
pub(crate) fn hellfire_hit_clamp(p: &mut Projectile) {
    if p.kind == HELLFIRE_ARROW && p.time_left > 1 {
        p.time_left = 1;
    }
}

// ---------------------------------------------------------------------------
// Kill() — the death spawns
// ---------------------------------------------------------------------------

/// The Holy Arrow's `Kill()` arm (`Projectile.cs:75212-75248`): the star
/// spawn. Returns the spawn request `(x, y, vx, vy, damage, ai0, ai1)`
/// for the pool to resolve through `spawn_with_data_ai_owned` (the slot
/// array is the pool's). `None` for every other kind.
///
/// The C# block (`:75223-75247`): `x = position.X + Next(-400, 400)`,
/// `y = position.Y - Next(600, 900)`; the velocity aims from (x, y) at
/// the arrow's CENTRE (`position + width/2, position + height/2`),
/// normalized to 22 (`22f / sqrt(dx² + dy²)`); the star's damage is the
/// arrow's `damage / 3` (integer division, `:75236`); `ai[1] =
/// position.Y` (the tile-collision floor) and `ai[0] = 1f` (the
/// "spawned by a Holy Arrow" marker — the star's OWN death re-spawn
/// `:75223` gates on `ai[0] > 0` for kind 92, so a Holy star chains ONE
/// more star at full star damage).
///
/// RNG: 2 draws — `Next(-400, 400)` then `Next(600, 900)` (`:75225-
/// 75226`). The 10-dust + 3-gore burst above it (`:75215-75222`) is the
/// documented dust/gore skip (its `Next(16, 18)` gore-type draws ×3 and
/// the NewDust/NewGore internals go with the layer, the pool-wide
/// convention). The `SoundID.Item10` cue is the sound skip.
pub(crate) fn holy_arrow_kill(
    p: &Projectile,
    rng: &mut dyn DropRng,
) -> Option<(f32, f32, f32, f32, i32, f32, f32)> {
    if p.kind != HOLY_ARROW {
        return None;
    }
    let x = p.pos.x + rng.next_range(-400, 400) as f32;
    let y = p.pos.y - rng.next_range(600, 900) as f32;
    let mut dx = p.pos.x + p.w * 0.5 - x;
    let mut dy = p.pos.y + p.h * 0.5 - y;
    let scale = 22.0 / (dx * dx + dy * dy).sqrt();
    dx *= scale;
    dy *= scale;
    let damage = p.damage / 3;
    Some((x, y, dx, dy, damage, 1.0, p.pos.y))
}

/// The falling star's OWN `Kill()` re-spawn (`Projectile.cs:75223`,
/// the `type == 92 && ai[0] > 0f` arm): the same spawn shape as the Holy
/// Arrow's but at FULL damage (`num646 = this.damage`, no /3) and NO
/// `ai[0] = 1` write (`:75246` sets only `ai[1]`) — so the chain is
/// exactly two stars deep. Same 2 draws.
pub(crate) fn falling_star_kill(
    p: &Projectile,
    rng: &mut dyn DropRng,
) -> Option<(f32, f32, f32, f32, i32, f32, f32)> {
    if p.kind != FALLING_STAR || p.ai[0] <= 0.0 {
        return None;
    }
    let x = p.pos.x + rng.next_range(-400, 400) as f32;
    let y = p.pos.y - rng.next_range(600, 900) as f32;
    let mut dx = p.pos.x + p.w * 0.5 - x;
    let mut dy = p.pos.y + p.h * 0.5 - y;
    let scale = 22.0 / (dx * dx + dy * dy).sqrt();
    dx *= scale;
    dy *= scale;
    Some((x, y, dx, dy, p.damage, 0.0, p.pos.y))
}

/// The Hellfire Arrow's `Kill()` blast resize (`Projectile.cs:76331-
/// 76340`): `penetrate = -1`, the box grows to 64×64 centred on the old
/// centre, then `Damage()` runs through the resized box (the pool's
/// `damage_pve_boxed` seam, the gem-bolt AoE precedent). Returns `true`
/// when the resize fired so the pool runs the box walk. The dust/gore
/// burst (`:76311-76330`, 2× `Next(61, 64)` gore types + the NewGore
/// kicks) and the `SoundID.Item14` cue are the documented skips.
/// The C# gates the resize on `owner == Main.myPlayer` — the pool only
/// simulates local-owned slots, so the gate folds true at the seam.
pub(crate) fn hellfire_kill_resize(p: &mut Projectile) -> bool {
    if p.kind != HELLFIRE_ARROW {
        return false;
    }
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    p.penetrate = -1;
    p.w = 64.0;
    p.h = 64.0;
    p.pos = Vec2::new(cx - 32.0, cy - 32.0);
    true
}

/// One bee of the Bee Arrow's death swarm — the spawn request the pool
/// resolves. `(kind, x, y, vx, vy, damage, kb, penetrate)`.
pub(crate) struct BeeSpawn {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub kb: f32,
}

/// The Bee Arrow's `Kill()` swarm (`Projectile.cs:76407-76429`): 6 bee
/// slots, the odd-indexed ones spawn only on `Next(3) == 0`; each bee
/// spawns at `position - normalize(oldVelocity)*8 * i` (the trail behind
/// the arrow), velocity `Next(-35,36)*0.01 + oldVelocity/6` per axis,
/// kind `beeType()` (the strong-bee coin flip), damage
/// `beeDamage(damage/3)`, knockback `beeKB(0f)`; the pool writes
/// `penetrate = 2` after the spawn (`:76426`).
///
/// RNG per spawned bee, in the C#'s named-argument evaluation order
/// (`:76423` — SpeedX, SpeedY, then Type, Damage, KnockBack): the odd
/// gate `Next(3)` FIRST (only for i = 1/3/5), then `Next(-35,36)` ×2
/// (the velocity jitters), then `beeType`'s `Next(2)` (only when
/// `strongBees`, `Player.cs:12419`), then `beeDamage`'s `Next(1,4)` or
/// `Next(2)` (`:12432`/`:12434`).
///
/// `old_velocity` is the slot's velocity at the START of the update tick
/// (`base.oldVelocity = base.velocity`, `:16766`) — the pool passes it.
pub(crate) fn bee_arrow_kill(
    p: &Projectile,
    old_velocity: Vec2,
    strong_bees: bool,
    rng: &mut dyn DropRng,
    out: &mut Vec<BeeSpawn>,
) {
    if p.kind != BEE_ARROW {
        return;
    }
    let mut lead = old_velocity;
    let len = lead.length();
    if len > 0.0 {
        lead /= len;
    }
    lead *= 8.0;
    for i in 0..6 {
        // `:76414` — `num825 % 2 != 1 || Main.rand.Next(3) == 0`: the
        // even indices always spawn (no draw), the odd ones roll.
        if i % 2 == 1 && rng.next_range(0, 3) != 0 {
            continue;
        }
        let pos = p.pos - lead * i as f32;
        let vx = rng.next_range(-35, 36) as f32 * 0.01 + old_velocity.x / 6.0;
        let vy = rng.next_range(-35, 36) as f32 * 0.01 + old_velocity.y / 6.0;
        // `Player.beeType` (`Player.cs:12417-12426`).
        let strong = strong_bees && rng.next_range(0, 2) == 0;
        let kind = if strong { GIANT_BEE } else { BEE };
        // `Player.beeDamage(this.damage / 3)` (`:12428-12435`) — the
        // integer /3 BEFORE the bee roll.
        let base = p.damage / 3;
        let damage = if strong {
            base + rng.next_range(1, 4)
        } else {
            base + rng.next_range(0, 2)
        };
        // `Player.beeKB(0f)` (`:12437-12444`): strong → 0.5, plain → 0.
        let kb = if strong { 0.5 } else { 0.0 };
        out.push(BeeSpawn { kind, x: pos.x, y: pos.y, vx, vy, damage, kb });
    }
}

/// `AI()`'s aiStyle-5 block for the falling star 92 (`Projectile.cs:
/// 23830-24065`): the `position.Y > ai[1]` tile-collision arm (`:23858-
/// 23864`), the alpha pulse (`:23928-23945` — the generic else arm: the
/// localAI[0] latch, `alpha += (int)(25 * localAI[0])` with the 200/0
/// bounce), the spin (`:23952` — `rotation += (|vx|+|vy|) * 0.01 *
/// direction`), the light 0.9 (`:24053`). The star's `soundDelay` arm
/// (`:23881-23885`, `20 + Next(40)`) is ported (S1-E-21h d75): the re-arm
/// roll rides the arm's rng seam, the Item9 cue queues draw-free. The
/// 1/10 dust + 1/20 gore gates (`:24054-24061`) stay the documented dust
/// skips — their draws go with the layer.
pub(crate) fn falling_star_step(p: &mut Projectile, world: &World, rng: &mut dyn DropRng) {
    // `:23858-23864` — the 92 arm: below the spawn floor the star
    // collides with tiles (above it, it falls through — the Holy Arrow
    // spawns it 600-900 px up with ai[1] = the arrow's death Y).
    if p.pos.y > p.ai[1] {
        p.tile_collide = true;
    }
    // S1-E-21h d75 — `:23881-23885`: the hum cooldown (the 92 is neither
    // 1037 nor 1049 — the gate is unconditional over this row). The
    // `20 + Main.rand.Next(40)` re-arm rolls on the UpdateProjectiles
    // stream BEFORE the Item9 cue queues at `base.position`.
    if p.sound_delay == 0 {
        p.sound_delay = 20 + rng.next_range(0, 40);
        p.wants_sound = Some((2, 9, p.pos.x, p.pos.y, 0.0));
    }
    // `:23928-23945` — the generic alpha pulse (the 92 is not on any of
    // the special-case lists above it).
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
    }
    p.alpha += (25.0 * p.local_ai[0]) as i32;
    if p.alpha > 200 {
        p.alpha = 200;
        p.local_ai[0] = -1.0;
    }
    if p.alpha < 0 {
        p.alpha = 0;
        p.local_ai[0] = 1.0;
    }
    // `:23952` — the spin (the 92 is not 503/1037/1049).
    p.rot += (p.vel.x.abs() + p.vel.y.abs()) * 0.01 * p.direction as f32;
    // `:24051-24053` — the `ai[1] == 1 || type == 92` arm: light 0.9
    // (the dust/gore gates skipped).
    p.light = 0.9;
    // Movement: the star keeps the generic `HandleMovement` default —
    // free fall until tileCollide arms, then kill-on-contact
    // (`:19843-19855`).
    if p.tile_collide {
        let last = p.vel;
        let hit_x = crate::player::collides_rect(world, p.pos.x + last.x, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, p.pos.y + last.y, p.w, p.h);
        p.pos += p.vel;
        if hit_x || hit_y {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    } else {
        p.pos += p.vel;
    }
}

// ---------------------------------------------------------------------------
// Witness tests-as-code — WRITTEN NOT RUN (code-only mode, S1-P-i17).
// Verified by re-reading the cited C# and hand arithmetic; run them with
// `cargo test -p tr-app special_arrows` once a compiler is available.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::{proj_data, FlailInputs, ProjectilePool, YoyoInputs, WOODEN_ARROW};

    fn air() -> World {
        tr_core::World::new("t".into(), 200, 200)
    }

    /// A rowed arrow at (1600, 800) flying right at 10 px/tick (the
    /// ai001/moonlord `shot` fixture shape).
    fn shot(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(10.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.extra_updates = d.extra_updates;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.arrow = d.arrow;
        p
    }

    /// A scripted `DropRng` that records every `(min, max)` range draw
    /// (the hurt_modifiers `ScriptRng` pattern).
    struct ScriptRng {
        rolls: Vec<i32>,
        cursor: usize,
        ranges: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn new(rolls: Vec<i32>) -> Self {
            Self { rolls, cursor: 0, ranges: Vec::new() }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            let v = self.rolls[self.cursor];
            self.cursor += 1;
            v
        }
        fn next_float(&mut self) -> f32 {
            panic!("the arrow arms draw no floats")
        }
    }

    fn npc_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 0, 0.0);
        n.hp = 100;
        n.max_hp = 100;
        n
    }

    /// The gate claims ONLY the 15 special arrows — never the plain
    /// arrows (1/2 generic, 81/82 slice A), the ai001 A/B/C kinds, the
    /// riders or the moon-lord shots.
    #[test]
    fn the_gate_is_disjoint() {
        for k in [1u16, 2, 81, 82, 14, 98, 184, 259, 270, 449, 462, 601,
                  680, 980, 110, 275, 634, 645, 20, 88, 355, 660, 837,
                  638, 639, 640, 92] {
            assert!(!covers(k), "kind {k} belongs to another arm");
        }
        for k in ARROW_KINDS {
            assert!(covers(k));
        }
    }

    /// The rows are the verbatim `SetDefaults` slices.
    #[test]
    fn the_rows_are_verbatim() {
        // `:837-848` — the Unholy Arrow: penetrate 5, 1200 life.
        let u = proj_data(UNHOLY_ARROW).unwrap();
        assert_eq!((u.w, u.h), (10.0, 10.0));
        assert_eq!(u.ai_style, 1);
        assert_eq!(u.penetrate, 5);
        assert_eq!(u.time_left, 1200);
        assert!(u.arrow);
        assert_eq!(u.light, 0.35);
        // `:849-863` — the Jester's: infinite pierce, 120 life,
        // extraUpdates 1, alpha 100, ignoreWater, light 0.4.
        let j = proj_data(JESTERS_ARROW).unwrap();
        assert_eq!(j.penetrate, -1);
        assert_eq!(j.time_left, 120);
        assert_eq!(j.extra_updates, 1);
        assert_eq!(j.alpha, 100);
        assert!(j.ignore_water);
        assert_eq!(j.light, 0.4);
        // `:1237-1247` — the Hellfire: infinite pierce, STOCK 3600 life.
        let h = proj_data(HELLFIRE_ARROW).unwrap();
        assert_eq!(h.penetrate, -1);
        assert_eq!(h.time_left, 3600);
        assert_eq!(h.light, 0.3);
        // `:1743-1752` — the Holy: stock pierce 1, 1200 life.
        let ho = proj_data(HOLY_ARROW).unwrap();
        assert_eq!(ho.penetrate, 1);
        assert_eq!(ho.time_left, 1200);
        // `:1753-1764` — the star: 24x24 aiStyle 5, alpha 50, scale 0.8,
        // tileCollide FALSE.
        let s = proj_data(FALLING_STAR).unwrap();
        assert_eq!((s.w, s.h), (24.0, 24.0));
        assert_eq!(s.ai_style, 5);
        assert_eq!(s.penetrate, 1);
        assert_eq!(s.alpha, 50);
        assert_eq!(s.scale, 0.8);
        assert!(!s.tile_collide);
        // `:1904-1914` / `:2579-2590` / `:3638-3649` — the debuff trio's
        // lights and updates.
        assert_eq!(proj_data(CURSED_ARROW).unwrap().light, 1.0);
        assert_eq!(proj_data(FROSTBURN_ARROW).unwrap().light, 1.0);
        let ic = proj_data(ICHOR_ARROW).unwrap();
        assert_eq!(ic.light, 1.0);
        assert_eq!(ic.extra_updates, 1);
        assert_eq!(proj_data(VENOM_ARROW).unwrap().extra_updates, 1);
        // `:2093-2104` / `:3100-3110` — Frost/Chlorophyte extraUpdates 1.
        assert_eq!(proj_data(FROST_ARROW).unwrap().extra_updates, 1);
        assert_eq!(proj_data(CHLOROPHYTE_ARROW).unwrap().extra_updates, 1);
        // `:5474-5484` — the merchant bone: penetrate 2.
        assert_eq!(proj_data(BONE_ARROW_MERCHANT).unwrap().penetrate, 2);
        // `:9570-9579` — the Shimmer: stock values.
        let sh = proj_data(SHIMMER_ARROW).unwrap();
        assert_eq!(sh.penetrate, 1);
        assert_eq!(sh.time_left, 1200);
        // `:5435-5445` — the Bee Arrow's alpha 255 (the fade seeds it).
        assert_eq!(proj_data(BEE_ARROW).unwrap().alpha, 255);
    }

    /// The Jester's and the Bee rest their flight counters (`:55949`,
    /// `:56002`) — 30 straight ticks, no gravity, `ai[0]` stays 0.
    #[test]
    fn the_resting_arrows_fly_straight() {
        let w = air();
        for kind in [JESTERS_ARROW, BEE_ARROW] {
            let mut p = shot(kind);
            for _ in 0..30 {
                step(&mut p, &w, &[], &[]);
            }
            assert_eq!(p.ai[0], 0.0, "kind {kind}: the counter rests");
            assert_eq!(p.vel.y, 0.0, "kind {kind}: no gravity");
            assert_eq!(p.pos.x, 1600.0 + 300.0);
        }
    }

    /// The Holy Arrow shares the hostile wooden arrow's 20-tick/+0.07
    /// gravity (`:56888-56895`), NOT the generic 15/+0.1.
    #[test]
    fn the_holy_arrow_takes_the_soft_gravity() {
        let w = air();
        let mut p = shot(HOLY_ARROW);
        for _ in 0..19 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.vel.y, 0.0, "19 ticks: below the 20 gate");
        step(&mut p, &w, &[], &[]); // tick 20: ai[0] = 20 → pin + 0.07
        assert_eq!(p.ai[0], 20.0);
        assert!((p.vel.y - 0.07).abs() < 1e-6);
        step(&mut p, &w, &[], &[]);
        assert_eq!(p.ai[0], 20.0, "the pin holds");
        assert!((p.vel.y - 0.14).abs() < 1e-6);
    }

    /// The per-kind gravity pins: Frostburn 17/+0.085 (`:57235`), Bone
    /// 35/+0.06 (`:57243`), Frost 30/+0.05 (`:57254`), Shadowflame
    /// 30/+0.04 (`:57484`), the merchant Bone 474 the GENERIC 15/+0.1
    /// (the C# chain names only 117).
    #[test]
    fn the_gravity_pins_are_per_kind() {
        let w = air();
        let cases: [(u16, f32, f32); 5] = [
            (FROSTBURN_ARROW, 17.0, 0.085),
            (BONE_ARROW, 35.0, 0.06),
            (FROST_ARROW, 30.0, 0.05),
            (SHADOWFLAME_ARROW, 30.0, 0.04),
            (BONE_ARROW_MERCHANT, 15.0, 0.1),
        ];
        for (kind, pin, grav) in cases {
            let mut p = shot(kind);
            let ticks = pin as usize;
            for _ in 0..ticks - 1 {
                step(&mut p, &w, &[], &[]);
            }
            assert_eq!(p.vel.y, 0.0, "kind {kind}: below the pin");
            step(&mut p, &w, &[], &[]);
            assert_eq!(p.ai[0], pin, "kind {kind}");
            assert!((p.vel.y - grav).abs() < 1e-6, "kind {kind}");
        }
    }

    /// The Shimmer Arrow's gravity INVERTS (`flag4`, `:56034-56036` +
    /// `:57571-57574`) and the −16 clamp arms for it alone (`:57760`).
    #[test]
    fn the_shimmer_arrow_falls_up() {
        let w = air();
        let mut p = shot(SHIMMER_ARROW);
        for _ in 0..15 {
            step(&mut p, &w, &[], &[]);
        }
        assert!((p.vel.y - (-0.1)).abs() < 1e-6, "tick 15: −0.1");
        // Drive it past the −16 bound.
        for _ in 0..200 {
            step(&mut p, &w, &[], &[]);
        }
        assert!(p.vel.y >= -16.0, "the flag4 −16 clamp holds");
        // A NON-flag4 arrow keeps the C#'s +16-only clamp — its upward
        // velocity is never clamped at −16 by AI_001 (`:57760` gates on
        // flag4).
        let mut q = shot(UNHOLY_ARROW);
        q.vel.y = -20.0;
        q.ai[0] = 0.0;
        step(&mut q, &w, &[], &[]);
        assert_eq!(q.vel.y, -20.0, "no −16 clamp without flag4");
    }

    /// The Bee Arrow's fade (`:55197-55208`): localAI[1] counts every
    /// tick; from the 3rd the alpha loses 50/tick with the 0 floor.
    #[test]
    fn the_bee_arrow_fades_from_the_third_tick() {
        let w = air();
        let mut p = shot(BEE_ARROW);
        step(&mut p, &w, &[], &[]);
        assert_eq!(p.alpha, 255, "tick 1: no fade");
        step(&mut p, &w, &[], &[]);
        assert_eq!(p.alpha, 255, "tick 2: no fade");
        step(&mut p, &w, &[], &[]);
        assert_eq!(p.alpha, 205, "tick 3: −50");
        for _ in 0..5 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.alpha, 0, "the floor holds");
        // `:57671-57683` — the spriteDirection flips by the velocity sign.
        assert_eq!(p.sprite_direction, -1, "flying right → −1");
        p.vel.x = -10.0;
        step(&mut p, &w, &[], &[]);
        assert_eq!(p.sprite_direction, 1);
    }

    /// StatusNPC's draw order and durations: the Shadowflame roll
    /// (`Next(120,300)`), the FlameArrow 1/3, the Frostburn 1/3, and the
    /// draw-free Cursed/Ichor/Venom arms.
    #[test]
    fn status_npc_draws_in_c_order() {
        // 495: one Next(120, 300) → ShadowFlame 153.
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![200]);
        status_npc(SHADOWFLAME_ARROW, &mut n, &mut r);
        assert_eq!(r.ranges, vec![(120, 300)]);
        let i = n.buffs.find_buff_index(153).expect("shadowflame");
        assert_eq!(n.buffs.buff_time[i], 200);
        // 2: Next(3) == 0 → OnFire 24/180; != 0 → nothing.
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![0]);
        status_npc(FLAME_ARROW, &mut n, &mut r);
        assert_eq!(r.ranges, vec![(0, 3)]);
        assert!(n.buffs.find_buff_index(24).is_some());
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![1]);
        status_npc(FLAME_ARROW, &mut n, &mut r);
        assert!(n.buffs.find_buff_index(24).is_none());
        // 172: Next(3) == 0 → Frostburn 44/180.
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![0]);
        status_npc(FROSTBURN_ARROW, &mut n, &mut r);
        assert_eq!(r.ranges, vec![(0, 3)]);
        let i = n.buffs.find_buff_index(44).expect("frostburn");
        assert_eq!(n.buffs.buff_time[i], 180);
        // 103/278/282: NO draws (the else-if chain's earlier tests are
        // kind comparisons, not rolls).
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![]);
        status_npc(CURSED_ARROW, &mut n, &mut r);
        assert!(r.ranges.is_empty());
        let i = n.buffs.find_buff_index(39).expect("cursed inferno");
        assert_eq!(n.buffs.buff_time[i], 420);
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![]);
        status_npc(ICHOR_ARROW, &mut n, &mut r);
        let i = n.buffs.find_buff_index(69).expect("ichor");
        assert_eq!(n.buffs.buff_time[i], 600);
        let mut n = npc_at(0.0, 0.0);
        let mut r = ScriptRng::new(vec![]);
        status_npc(VENOM_ARROW, &mut n, &mut r);
        assert!(r.ranges.is_empty());
        let i = n.buffs.find_buff_index(70).expect("venom");
        assert_eq!(n.buffs.buff_time[i], 600);
        let i = n.buffs.find_buff_index(395).expect("potent acid");
        assert_eq!(n.buffs.buff_time[i], 120);
        // The plain arrows draw nothing and apply nothing.
        for kind in [WOODEN_ARROW, UNHOLY_ARROW, JESTERS_ARROW, HOLY_ARROW,
                     HELLFIRE_ARROW, BONE_ARROW, FROST_ARROW, CHLOROPHYTE_ARROW,
                     BEE_ARROW, SHIMMER_ARROW] {
            let mut n = npc_at(0.0, 0.0);
            let mut r = ScriptRng::new(vec![]);
            status_npc(kind, &mut n, &mut r);
            assert!(r.ranges.is_empty(), "kind {kind} draws nothing");
        }
    }

    /// The Hellfire clamp (`:13252-13255`): the first hit pins timeLeft
    /// to 1; a timeLeft already ≤ 1 stays; other kinds untouched.
    #[test]
    fn the_hellfire_hit_clamps_the_life() {
        let mut p = shot(HELLFIRE_ARROW);
        hellfire_hit_clamp(&mut p);
        assert_eq!(p.time_left, 1);
        p.time_left = 1;
        hellfire_hit_clamp(&mut p);
        assert_eq!(p.time_left, 1);
        let mut q = shot(UNHOLY_ARROW);
        hellfire_hit_clamp(&mut q);
        assert_eq!(q.time_left, 1200, "only the 41 clamps");
    }

    /// The Hellfire blast resize (`:76331-76340`): 64×64 centred, the
    /// penetrate opens to −1.
    #[test]
    fn the_hellfire_blast_resizes_centred() {
        let mut p = shot(HELLFIRE_ARROW);
        assert!(hellfire_kill_resize(&mut p));
        assert_eq!((p.w, p.h), (64.0, 64.0));
        assert_eq!(p.pos, Vec2::new(1600.0 + 5.0 - 32.0, 800.0 + 5.0 - 32.0));
        assert_eq!(p.penetrate, -1);
        let mut q = shot(HOLY_ARROW);
        assert!(!hellfire_kill_resize(&mut q));
    }

    /// The Holy Arrow's star spawn (`:75223-75247`): the 2 draws in
    /// order, the 22-speed aim at the centre, damage/3, ai0 = 1,
    /// ai1 = position.Y.
    #[test]
    fn the_holy_arrow_spawns_a_third_damage_star() {
        let mut p = shot(HOLY_ARROW);
        p.damage = 30;
        // Next(-400,400) → 0, Next(600,900) → 600: the star at
        // (1600, 200), aiming straight down at (1605, 805).
        let mut r = ScriptRng::new(vec![0, 600]);
        let (x, y, vx, vy, dmg, ai0, ai1) =
            holy_arrow_kill(&p, &mut r).expect("the star");
        assert_eq!(r.ranges, vec![(-400, 400), (600, 900)]);
        assert_eq!((x, y), (1600.0, 200.0));
        let len = (vx * vx + vy * vy).sqrt();
        assert!((len - 22.0).abs() < 1e-4, "the 22 speed");
        assert!(vx > 0.0 && vy > 0.0, "aims at the centre");
        assert_eq!(dmg, 10, "30 / 3");
        assert_eq!(ai0, 1.0);
        assert_eq!(ai1, 800.0);
        // The star's own re-spawn: full damage, ai0 = 0 (no third star).
        let mut s = shot(FALLING_STAR);
        s.damage = 10;
        s.ai[0] = 1.0;
        let mut r = ScriptRng::new(vec![0, 600]);
        let (_, _, _, _, dmg, ai0, _) = falling_star_kill(&s, &mut r).expect("the chain");
        assert_eq!(dmg, 10, "the star chains at FULL damage");
        assert_eq!(ai0, 0.0, "the chain stops at two");
        s.ai[0] = 0.0;
        let mut r = ScriptRng::new(vec![]);
        assert!(falling_star_kill(&s, &mut r).is_none(), "ai0 = 0: no spawn");
        assert!(r.ranges.is_empty(), "the gate draws nothing");
    }

    /// The Bee Arrow's swarm (`:76407-76429`): the odd-slot gates, the
    /// trail positions, the velocity jitters + oldVelocity/6, the
    /// beeType/beeDamage/beeKB draw order, the /3 base.
    #[test]
    fn the_bee_arrow_swarm_draws_in_order() {
        let mut p = shot(BEE_ARROW);
        p.damage = 30; // base 10
        let old = Vec2::new(6.0, 0.0);
        // The script interleaves: i0 (no gate) vx vy dmg, i1 gate vx vy
        // dmg, … — in the exact call order (all odd gates pass).
        let rolls = vec![
            0, 0, 1,        // i0: vx, vy, dmg
            0, 0, 0, 1,     // i1: gate, vx, vy, dmg
            0, 0, 1,        // i2
            0, 0, 0, 1,     // i3
            0, 0, 1,        // i4
            0, 0, 0, 1,     // i5
        ];
        let mut r = ScriptRng::new(rolls);
        let mut out = Vec::new();
        bee_arrow_kill(&p, old, false, &mut r, &mut out);
        assert_eq!(out.len(), 6, "all gates passed");
        assert_eq!(out[0].kind, BEE);
        assert_eq!(out[0].damage, 11, "10 + Next(2)=1");
        assert_eq!(out[0].kb, 0.0);
        assert_eq!(out[0].vx, 1.0, "0*0.01 + 6/6");
        // The trail: bee i at position − (8,0)*i.
        assert_eq!(out[3].x, 1600.0 - 24.0);
        assert_eq!(out[3].y, 800.0);
        // An odd gate that fails skips the bee AND its draws.
        let mut r = ScriptRng::new(vec![
            0, 0, 1,        // i0
            1,              // i1 gate FAILS
            0, 0, 1,        // i2
            1,              // i3 fails
            0, 0, 1,        // i4
            1,              // i5 fails
        ]);
        let mut out = Vec::new();
        bee_arrow_kill(&p, old, false, &mut r, &mut out);
        assert_eq!(out.len(), 3, "only the even slots");
        // Strong bees: the coin flip precedes the damage roll; a strong
        // bee is 566, damage 10 + Next(1,4), kb 0.5. i0 is the strong bee;
        // every odd gate fails (1) and i2/i4 take plain-bee draws, so the
        // script never runs off.
        let mut r = ScriptRng::new(vec![
            0, 0, 0, 2,     // i0: vx, vy, beeType Next(2)=0 → strong, Next(1,4)=2
        ]);
        let mut out = Vec::new();
        r.rolls.extend([
            1,              // i1 gate FAILS
            0, 0, 1, 0,     // i2: vx, vy, beeType=1 → plain, Next(2)=0
            1,              // i3 gate FAILS
            0, 0, 1, 0,     // i4
            1,              // i5 gate FAILS
        ]);
        bee_arrow_kill(&p, old, true, &mut r, &mut out);
        assert_eq!(out[0].kind, GIANT_BEE);
        assert_eq!(out[0].damage, 12);
        assert_eq!(out[0].kb, 0.5);
        assert_eq!(out.len(), 3, "i0/i2/i4 — the odd gates failed");
    }

    /// The Chlorophyte bounce (`:18539-18561`): first bounce ×0.66, the
    /// reversal, the retarget lead, the 4-bounce kill.
    #[test]
    fn the_chlorophyte_bounce_machine() {
        let mut w = air();
        // A solid tile at (101, 50) — the arrow at (1600, 800) flying
        // right at 10 hits it (the box spans x 1600-1610 → 1610-1620).
        let t = w.tile_mut(101, 50).unwrap();
        t.kind = 1; // dirt
        t.flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = shot(CHLOROPHYTE_ARROW);
        p.damage = 100;
        let mut n = npc_at(1400.0, 700.0); // up-left of the bounce
        n.max_hp = 100;
        step(&mut p, &w, &[n], &[]);
        assert_eq!(p.ai[1], 1.0, "first bounce");
        assert_eq!(p.damage, 66, "(int)(100 * 0.66)");
        assert!(p.active);
        assert!(p.vel.x < 0.0 || p.vel.y < 0.0, "reversed/retargeted");
        // The bounce tick integrates the POST-bounce velocity: the C#
        // `UpdatePosition` dry arm (`:20443-20445`) steps `base.velocity`
        // (the reversed/retargeted value), NOT the pre-collision
        // `wetVelocity` (only the `base.wet` arm reads that, `:20439-
        // 20441`). The retarget aims up-left at the NPC, so the arrow
        // moves BACK (x decreases), not forward into the tile.
        assert!(p.pos.x < 1600.0, "the bounce steps the retargeted velocity");
        // The retarget aims at the NPC (within 800 px, clear line).
        let speed = p.vel.length();
        assert!((speed - 10.0).abs() < 1e-4, "the speed survives");
        // Three more bounces kill it (ai[1] reaches 4).
        for _ in 0..3 {
            // Re-arm the collision: put the arrow back against the tile.
            p.pos = Vec2::new(1600.0, 800.0);
            p.vel = Vec2::new(10.0, 0.0);
            step(&mut p, &w, &[], &[]);
        }
        assert!(!p.active, "the 4th bounce kills");
        assert_eq!(p.ai[1], 4.0);
    }

    /// The falling star's AI_005 slice: the ai[1] floor arms tileCollide,
    /// the alpha pulse bounces 50→200→0, the spin rides the direction.
    #[test]
    fn the_falling_star_pulse_and_floor() {
        let w = air();
        let mut p = shot(FALLING_STAR);
        p.ai[1] = 900.0; // the floor below the star
        p.vel = Vec2::new(0.0, 22.0);
        assert!(!p.tile_collide);
        let mut rng = ScriptRng::new(vec![7, 7]);
        falling_star_step(&mut p, &w, &mut rng);
        assert!(!p.tile_collide, "above the floor: falls through");
        assert_eq!(p.alpha, 75, "50 + 25");
        assert_eq!(p.light, 0.9);
        // S1-E-21h d75 — the hum: the first tick re-arms 20 + Next(40)
        // and queues the Item9 cue; the cooldown holds the next tick.
        assert_eq!(p.sound_delay, 27, "20 + the scripted 7");
        assert_eq!(p.wants_sound, Some((2, 9, p.pos.x, p.pos.y, 0.0)));
        assert_eq!(rng.ranges, vec![(0, 40)]);
        p.wants_sound = None;
        p.pos.y = 950.0;
        falling_star_step(&mut p, &w, &mut rng);
        assert!(p.tile_collide, "below the floor: collides");
        assert_eq!(p.wants_sound, None, "the cooldown holds");
        assert_eq!(rng.ranges.len(), 1, "no second re-arm roll");
        // Drive the pulse to the 200 cap and the flip: 50 +25×6 = 200 on
        // step 6 (NOT > 200, no flip yet); step 7 rolls 225 → clamps 200
        // and flips localAI[0] to −1.
        let mut q = shot(FALLING_STAR);
        q.ai[1] = 100000.0;
        let mut rng2 = ScriptRng::new(vec![7]);
        for _ in 0..7 {
            falling_star_step(&mut q, &w, &mut rng2);
        }
        assert_eq!(q.alpha, 200);
        assert_eq!(q.local_ai[0], -1.0, "the cap flips the pulse");
        falling_star_step(&mut q, &w, &mut rng2);
        assert_eq!(q.alpha, 175, "200 − 25");
    }

    /// The pool seam end-to-end: a Holy Arrow killed by the lifetime
    /// block spawns its star through the death hook (the moonlord-trail
    /// precedent), the star rowed and aimed. The arrow sits at y 1600
    /// so the star's 600-900 px upward offset stays inside the world
    /// (the out-of-world guard would eat a negative-Y spawn).
    #[test]
    fn the_pool_death_hook_spawns_the_star() {
        let mut w = air();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(HOLY_ARROW, 0, 1600.0, 1600.0, 10.0, 0.0, 30, 2.0, 0.0, 0.0)
            .expect("the arrow");
        pool.slots_mut()[slot].time_left = 1;
        pool.update(
            &mut w,
            &mut [],
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::UnifiedRandom::new(7),
            0,
            true,
        );
        assert!(!pool.slots()[slot].active, "the arrow died");
        let star = pool
            .slots()
            .iter()
            .find(|p| p.active && p.kind == FALLING_STAR)
            .expect("the star spawned");
        assert_eq!(star.damage, 10, "30/3");
        assert_eq!(star.ai[0], 1.0);
        assert_eq!(star.ai[1], 1600.0, "the death-Y floor");
        assert!(!star.tile_collide, "the star row's false survives");
        assert!(star.pos.y < 1600.0, "the star spawns above");
    }

    /// The pool seam: a Bee Arrow's death spawns the swarm through the
    /// death hook — the bees carry the row's box/extraUpdates and the
    /// `penetrate = 2` override (`:76426`).
    #[test]
    fn the_pool_death_hook_spawns_the_bee_swarm() {
        let mut w = air();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(BEE_ARROW, 0, 1600.0, 1600.0, 6.0, 0.0, 30, 2.0, 0.0, 0.0)
            .expect("the arrow");
        pool.slots_mut()[slot].time_left = 1;
        pool.update(
            &mut w,
            &mut [],
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut tr_core::UnifiedRandom::new(7),
            0,
            true,
        );
        assert!(!pool.slots()[slot].active, "the arrow died");
        let bees: Vec<_> = pool
            .slots()
            .iter()
            .filter(|p| p.active && (p.kind == BEE || p.kind == GIANT_BEE))
            .collect();
        assert!((3..=6).contains(&bees.len()), "the even slots + the odd gates");
        for b in &bees {
            assert_eq!(b.penetrate, 2, "the :76426 override");
            assert!(b.damage >= 10, "the /3 base 10 + the bee roll");
            assert_eq!(b.extra_updates, 3, "the bee row's extraUpdates");
        }
    }
}
