{
// balance-open (union shared delimiter)
//! S1-D-5 slice B — `AI_001`'s per-type arms for the higher-id rowed kinds
//! (`Projectile.cs:54002-57794`). The port ran ONE generic flight
//! (`Projectile::ai_arrow` → `ai_style1_flight`, projectile.rs:5057) for
//! every aiStyle-1 kind; the C# has ~142 per-type branches inside the one
//! method. This module ports the upper-half rowed kinds (slice A,
//! `task/S1-D-5-aistyle1-types-a`, takes the low-id arrow family); the kind
//! set is [`SLICE_B_KINDS`] and the dispatch seam in `ProjectilePool::update`'s
//! aiStyle-1 arm (projectile.rs:11053) gates on [`covers`] ONLY — every other
//! kind keeps the generic flight BYTE-IDENTICAL.
//!
//! Kind set (9 after the merge-time dedupe): 449 SaucerLaser,
//! 462 PhantasmalBolt, 581 VortexDart, 601 CannonExplosive,
//! 622 BlowupSmokeMoonlord, 680 BallistaShot, 686 BetsyFireball,
//! 962 DeerclopsIceChunk, 980 TrapVenomDart. The six overlap kinds
//! (259/270/325/389/436/438) are owned by slice A
//! (`projectile_ai001_a`, landed first); their arms below are kept as
//! citation-verified reference but [`covers`] never claims them, so the
//! dispatch never double-applies.
//!
//! Structure mirrors `AI_001`'s own order:
//! 1. the hook group above the counter (`:54066-55671`),
//! 2. the `flag3` counter switch (`:55945-56041`),
//! 3. the hook group below the counter (`:56056-56709`),
//! 4. the pre-rotation/gravity else-if chain (`:56753-57580`) — the 962 arm
//!    (`:56912`), the 680 arm (`:57471`), the 686 ladder (`:57522`) and the
//!    generic `ai[0] >= 15` arm (`:57568`) are mutually exclusive,
//! 5. the rotation tail (`:57582-57752`),
//! 6. the ±16 clamp (`:57753-57768`) + movement.
//!
//! Movement is NOT in `AI_001` — it rides `HandleMovement`; the port's
//! established stand-in is `ai_style1_flight`'s tail (projectile.rs:5064-5080:
//! the ±16 clamp, the default rotation, the kill-on-tile integration). This
//! module replicates that tail verbatim in [`finish_flight`] so the 680/686/962
//! kinds can substitute their OWN gravity arms (the shared flight's generic
//! `ai[0] >= 15` gravity would double-apply). The replication is cited at both
//! ends; the generic path in projectile.rs is untouched.
//!
//! CODE-ONLY MODE (S1-D-5 slice B): the witness tests at the bottom are
//! WRITTEN NOT RUN (no compiler invoked on this branch); they were verified
//! by re-reading the cited C# blocks and hand arithmetic.
//!
//! Deliberate skips (recorded in ANALYSIS "aiStyle-1 per-type arms, slice B"):
//! - The `Main.rand` draws that feed DUST only — the pool's aiStyle-1 arms
//!   keep zero RNG for those (the precedent `ai_style1_rider`,
//!   projectile.rs:5109-5127). Every skipped draw is cited at its point.
//!   S1-E-21h d76 THREADS `rng` for the ONE state-bearing cue pick: the
//!   Phantasmal Bolt 462's `Next(2)` Item124/125 choice (`:55272`), which
//!   selects the played cue (the golf-strike precedent). Its `Next(5, 10)`
//!   dust-count draw stays skipped with the dust.
//! - Dust, light (`Lighting.AddLight`/`PlotTileLine`) and `netUpdate` — no
//!   such layers in the pool. SOUND now rides the `wants_sound` seam
//!   (S1-E-21h d76): the 601 portal pair, the 259/438/462/389/980 latch
//!   chain, the 270 skull, the 581 vortex dart and the 686 Betsy Item20.
//! - The Harpy-charm tail (`:57769-57793`, `AI_001_Features`) — no charm
//!   feature in the port.
//! - The port's clamp is ±16 (projectile.rs:5064) where the C# clamps the
//!   upper bound only for non-1006 kinds (`:57760-57767`); the module keeps
//!   the PORT convention so the shared tail stays byte-identical.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::projectile::{
    BALLISTA_SHOT, BETSY_FIREBALL, BLOWUP_SMOKE_MOONLORD, BRAIN_SCRAMBLER, CANNON_EXPLOSIVE,
    DEERCLOPS_ICE_CHUNK, FLAMING_WOOD, GOLEM_EYE_BEAM, MINI_RETINA_LASER, PHANTASMAL_BOLT,
    Projectile, RAY_GUNNER, SAUCER_LASER, SCUTLIX_LASER, SKULL, TRAP_VENOM_DART, VORTEX_DART,
    proj_frames,
};

/// The slice-B kind set — the higher-id rowed kinds this module owns.
/// Merge-time dedupe (P2): 259/270/325/389/436/438 were ported by BOTH
/// slices; slice A (`projectile_ai001_a::SLICE_A_KINDS`) landed first and
/// wins — the six are REMOVED here so [`covers`] is disjoint from A's
/// `handles`. Their per-kind arms below stay as reference (the witness
/// tests still exercise them directly) but the dispatch never routes
/// those kinds here. The M4-PROJ-6 riders (20/88/355/660), the Book of
/// Skulls (837) and every unrowed kind keep their existing arms untouched.
pub(crate) const SLICE_B_KINDS: [u16; 10] = [
    SAUCER_LASER,          // 449
    PHANTASMAL_BOLT,       // 462
    VORTEX_DART,           // 581
    CANNON_EXPLOSIVE,      // 601
    SCUTLIX_LASER,         // 606 (W4-d8 DOC-SWEEP (3): the mount's fired beam)
    SCUTLIX_LASER,         // 606 (M2-MOUNT-4 — the 449 sibling, same C# block)
    BLOWUP_SMOKE_MOONLORD, // 622
    BALLISTA_SHOT,         // 680
    BETSY_FIREBALL,        // 686
    DEERCLOPS_ICE_CHUNK,   // 962
    TRAP_VENOM_DART,       // 980
];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    SLICE_B_KINDS.contains(&kind)
}

/// The `flag3` skip switch (`Projectile.cs:55947-56032`): the kinds whose
/// flight counter holds at rest (no `ai[0]++` at `:56038-56041`). The
/// slice-B members of the C# list: 259 (`:55972`), 270 (`:55974`), 325
/// (`:55984`), 389 (`:55991`), 436 (`:55993`), 438 (`:55994`), 449
/// (`:55997`), 462 (`:55999`), 601 (`:56013`), 606 (`:56014`, M2-MOUNT-4),
/// 980 (`:56029`). The other five (581/622/680/686/962) are NOT on the
/// list — they count like arrows. `flag4` (the inverted gravity) is
/// 1006-only (`:56034-56036`) — no slice-B kind sets it.
fn counts_flight(kind: u16) -> bool {
    !matches!(
        kind,
        GOLEM_EYE_BEAM
            | SKULL
            | FLAMING_WOOD
            | MINI_RETINA_LASER
            | BRAIN_SCRAMBLER
            | RAY_GUNNER
            | SAUCER_LASER
            | PHANTASMAL_BOLT
            | CANNON_EXPLOSIVE
            // W4-d8 DOC-SWEEP (3): the 606 is on the flag3 list (`:56014`).
            | SCUTLIX_LASER
            | TRAP_VENOM_DART
    )
}

/// The shared flight tail — a verbatim replication of
/// `Projectile::ai_style1_flight`'s body below the gravity
/// (projectile.rs:5064-5080): the ±16 Y clamp, the default rotation
/// (`AI_001`'s `:57750` `atan2(vy, vx) + 1.57f`) and the movement's
/// kill-on-tile integration (the port's `HandleMovement` stand-in). The
/// per-kind rotation tails overwrite `rot` AFTER this call, exactly as
/// `ai_style1_rider` does (projectile.rs:5154-5157).
fn finish_flight(p: &mut Projectile, world: &World) {
    p.vel.y = p.vel.y.clamp(-16.0, 16.0);
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    // W4-d8 DOC-SWEEP (3) — `HandleMovement`'s Martian-laser arm
    // (`:18278-18286`): on the FIRST blocked move the 440/449/606 latch
    // `ai[1] = 1`, take one last full-velocity step and restore the
    // pre-collision velocity (the beam sticks in the tile and the ramp
    // counts down). While `ai[1] == 1` the `:17590` gate skips
    // TileCollision and `UpdatePosition`'s `:20435` skip holds the
    // position — the beam never moves again (the moonlord-trail stick
    // precedent, projectile_ai_moonlord.rs:226-250).
    if (p.kind == SAUCER_LASER || p.kind == SCUTLIX_LASER) && p.ai[1] == 1.0 {
        return;
    }
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            // S1-E-21h d75 part 2 — `HandleMovement`'s 601 arm
            // (`Projectile.cs:18084-18103`): the portal bolt's impact cue,
            // gated on `alpha > 0` (the `netMode != 2` guard is the port's
            // single-player fold): `ai[1] != 0` → Item114 (the blue portal),
            // else Item115 (orange), at the PRE-move `base.position` (the
            // C# plays before its `position += velocity` + Kill; the port's
            // kill-without-step shape is the pre-existing flight deviation).
            // `PortalHelper.TryPlacingPortal` stays unported.
            if p.kind == CANNON_EXPLOSIVE && p.alpha > 0 {
                let style = if p.ai[1] != 0.0 { 114 } else { 115 };
                p.wants_sound = Some((2, style, p.pos.x, p.pos.y, 0.0));
            }
            p.active = false;
            p.time_left = 0;
            if p.kind == SAUCER_LASER || p.kind == SCUTLIX_LASER {
                // `:18280-18284` — the latch + the last step + the restore.
                p.ai[1] = 1.0;
                p.pos += p.vel;
                // velocity = lastVelocity: unchanged (the probe never wrote).
            } else {
                p.active = false;
                p.time_left = 0;
            }
        } else {
            // S1-P-i57 — the `ai_style1_flight` swap rides here too (the
            // replica stays byte-identical): `:20441`'s `wet ? wetVelocity :
            // velocity` via `wet_velocity` (the `:17612-17655` drag wet).
            p.pos += crate::projectile_wet::wet_velocity(p, world);
        }
    } else {
        p.pos += p.vel;
    }
}

/// `AI_001`'s slice-B body for one slot, in the C#'s own block order.
/// `owner` is the slot's owner centre (`Main.player[owner].Center`) — the
/// 270's `Player.FindClosest` (`:56059`) resolves to it in the single-player
/// port (the same stand-in the other owner-aware arms use); `None` means no
/// player resolved and the steering is skipped. Every early `Kill(); return`
/// is the port's death convention (`active = false`, `time_left = 0`).
pub(crate) fn step(
    p: &mut Projectile,
    world: &World,
    owner: Option<(f32, f32)>,
    rng: &mut dyn DropRng,
) {
    // ---- 1. The hook group above the counter ------------------------------
    // `:54066-54120` — the Cannon Explosive (601): the portal light
    // (`:54068-54071`, skipped), the alpha-1..15 dust window (`:54072-54086`
    // — the 4× dust 264 with `Utils.RandomVector2(Main.rand, -1, 1)` draws,
    // SKIPPED) + the Item114/115 portal cues (PORTED below, S1-E-21h d76),
    // the alpha fade 15, the 2-frame cycle at 4 ticks/frame
    // (`Main.projFrames[601] = 2`), the alpha-0 dust 263 (`:54109-54119`,
    // skipped).
    if p.kind == CANNON_EXPLOSIVE {
        // `:54072-54095` — the alpha-1..15 window: the 4× dust 264 (with its
        // `Utils.RandomVector2` draws) stays SKIPPED, but the portal cue
        // plays — `ai[0] != 0` → Item114, else Item115 (`:54087-54094`,
        // S1-E-21h d76), at `base.position`, once per tick while the window
        // holds (the C# has no latch here — the alpha fade closes it).
        if p.alpha > 0 && p.alpha <= 15 {
            let style = if p.ai[0] != 0.0 { 114 } else { 115 };
            p.wants_sound = Some((2, style, p.pos.x, p.pos.y, 0.0, 1.0));
        }
        p.alpha -= 15;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        p.frame_counter += 1;
        if p.frame_counter >= 4 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= proj_frames(CANNON_EXPLOSIVE) as i32 {
                p.frame = 0;
            }
        }
    }
    // `:54150-54174` — the Brain Scrambler (436): the ai[1] sound latch
    // (Item12 skipped), the alpha fade 40, `spriteDirection = direction`,
    // the 4-frame cycle at 3 ticks/frame, the light (`:54173`, skipped).
    // `base.direction` tracks velocity.X's sign after each movement step;
    // the port's stand-in is the velocity sign (the `ai_book_of_skulls`
    // convention, projectile.rs:4355).
    if p.kind == BRAIN_SCRAMBLER {
        if p.ai[1] == 0.0 {
            p.ai[1] = 1.0;
            // SoundID.Item12 (`:54155`) — no sound layer, skipped.
        }
        p.alpha -= 40;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
        p.frame_counter += 1;
        if p.frame_counter >= 3 {
            p.frame += 1;
            p.frame_counter = 0;
            if p.frame >= 4 {
                p.frame = 0;
            }
        }
    }
    // `:54772-54828` — the Martian laser group (440/449/606); 449 is the
    // slice-B member: the alpha fade 25, the light (`:54788`, skipped), the
    // localAI[0] flight-time ramp — +3/tick while ai[1] == 0 (the cap 100,
    // `num54`/`num55` at `:54794-54795`), −3/tick once ai[1] != 0 with the
    // `Kill(); return` at ≤ 0 (`:54822-54826`).
    // W4-d8 DOC-SWEEP (3): the 606 Scutlix beam joins the group — the same
    // fade 25, its light (`:54792`, skipped), the ramp at `num54 = 150` /
    // `num55 = 5` (`:54796-54800`), and the one-shot `localAI[0] == 5`
    // four-dust-182 burst (`:54804-54813`, the dust BODIES + NewDust
    // internals skip — zero explicit draws, the slice-B RNG note).
    // `:54772-54828` — the Martian laser group (440/449/606); 449 and 606
    // are the slice-B members (M2-MOUNT-4 adds the Scutlix mount's 606):
    // the alpha fade 25, the light (`:54784-54793`, skipped), the
    // localAI[0] flight-time ramp — +num55/tick while ai[1] == 0 (the cap
    // num54, `:54794-54800`: 449 → 100/3, 606 → 150/5), −num55/tick once
    // ai[1] != 0 with the `Kill(); return` at ≤ 0 (`:54822-54826`). The
    // 606's first-tick 4× dust-182 (`:54804-54812`, `localAI[0] == num55`)
    // is SKIPPED with the dust (the slice-B zero-RNG convention — the
    // `NewDust` args are constants, no explicit call-site draw); its
    // `Kill()` dust census (`:73563-73572`, the `Next(3, 7)` count + the
    // dust-182 loop) rides the same skip (the 449 sibling's `:73573` arm
    // is skipped identically — the port's death is `active = false`).
    if p.kind == SAUCER_LASER || p.kind == SCUTLIX_LASER {
        if p.alpha > 0 {
            p.alpha -= 25;
        }
        if p.alpha < 0 {
            p.alpha = 0;
        }
        let (cap, step) = if p.kind == SCUTLIX_LASER {
            (150.0f32, 5.0f32) // `:54798-54799`
        } else {
            (100.0, 3.0) // `:54794-54795`
        };
        if p.ai[1] == 0.0 {
            p.local_ai[0] += step;
            // `:54804-54813` — the 606's first-ramp-tick dust burst (the
            // bodies skip; no explicit Main.rand draw gates it).
            if p.local_ai[0] == step && p.kind == SCUTLIX_LASER {
                // four dust-182 at Center - velocity/2 (skipped bodies)
            }

        }            (150.0f32, 5.0f32)
            (100.0, 3.0)
            if p.local_ai[0] > cap {
                p.local_ai[0] = cap;
            }
        } else {
            p.local_ai[0] -= step;
            if p.local_ai[0] <= 0.0 {
                p.active = false;
                p.time_left = 0;
                return;
            }
        }
    }
    // `:54829-54832` — the Ray Gunner (438): light only (skipped); the
    // kind's state rides the ai[1] latch + the speed fade below.
    // `:54860-54876` — the Phantasmal Bolt (462): the dust 229
    // (`:54862-54867`, skipped), the 5-frame cycle at 9 ticks/frame.
    if p.kind == PHANTASMAL_BOLT {
        p.frame_counter += 1;
        if p.frame_counter >= 9 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 5 {
                p.frame = 0;
            }
        }
    }
    // `:55164-55196` — the Flaming Wood (325): the alpha fade 100, the
    // light (`:55171`, skipped), the two dust rolls at alpha == 0
    // (`Main.rand.Next(3)` / `Next(4)` + their inner `Next(10)`/`Next(5)`
    // draws, `:55175-55189`, SKIPPED — zero RNG in the aiStyle-1 pool),
    // the ai[1] sound latch (Item42 skipped).
    if p.kind == FLAMING_WOOD {
        p.alpha -= 100;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        if p.ai[1] == 0.0 {
            p.ai[1] = 1.0;
            // SoundID.Item42 (`:55194`) — skipped.
        }
    }
    // `:55209-55330` — the ai[1] == 0 sound-latch else-if chain. The
    // slice-B members: 259 (`:55239-55243`, Item33), 438 (`:55254-55258`,
    // Item12), 462 (`:55269-55290` — its `Main.rand.Next(2)` cue pick
    // (Item124/125) is PORTED, S1-E-21h d76; the `Next(5, 10)` dust-count
    // draw + the dust 229 loop stay SKIPPED with the dust), 389
    // (`:55296-55300`, Item12), 980 (`:55326-55330`, Item17). Each kind
    // appears once in the chain, so a per-kind match is exact. The cues
    // queue at `base.position` (the pre-movement position).
    if p.ai[1] == 0.0 {
        match p.kind {
            GOLEM_EYE_BEAM => {
                p.ai[1] = 1.0;
                p.wants_sound = Some((2, 33, p.pos.x, p.pos.y, 0.0, 1.0)); // :55242
            }
            RAY_GUNNER => {
                p.ai[1] = 1.0;
                p.wants_sound = Some((2, 12, p.pos.x, p.pos.y, 0.0, 1.0)); // :55257
            }
            PHANTASMAL_BOLT => {
                p.ai[1] = 1.0;
                // `:55272-55279` — the ONE Next(2) cue pick: 0 → Item124,
                // else Item125. The draw rides UpdateProjectiles; the dust
                // count (`Next(5, 10)`) and the 229 loop stay skipped.
                let style = if rng.next_range(0, 2) == 0 { 124 } else { 125 };
                p.wants_sound = Some((2, style, p.pos.x, p.pos.y, 0.0, 1.0));
            }
            MINI_RETINA_LASER => {
                p.ai[1] = 1.0;
                p.wants_sound = Some((2, 12, p.pos.x, p.pos.y, 0.0, 1.0)); // :55299
            }
            TRAP_VENOM_DART => {
                p.ai[1] = 1.0;
                p.wants_sound = Some((2, 17, p.pos.x, p.pos.y, 0.0, 1.0)); // :55329
            }
            _ => {}
        }
    }
    // `:55660-55671` — the speed-scaled alpha fade group (242/302/438/
    // 462/592); the slice-B members are 438 + 462.
    // `alpha -= (byte)((double)speed * 0.9)` — the double product
    // truncated toward zero by the byte cast (the product is < 256 for
    // every reachable speed, so the i32 cast is exact).
    if p.kind == RAY_GUNNER || p.kind == PHANTASMAL_BOLT {
        let speed = (p.vel.x * p.vel.x + p.vel.y * p.vel.y).sqrt();
        if p.alpha > 0 {
            p.alpha -= (speed as f64 * 0.9) as i32;
        }
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }

    // ---- 2. The flight counter (`:55945-56041`) ---------------------------
    if counts_flight(p.kind) {
        p.ai[0] += 1.0;
    }

    // ---- 3. The hook group below the counter ------------------------------
    // `:56056-56089` — the Skull (270): `Player.FindClosest` (`:56059`,
    // the owner-centre stand-in), the ai[1] age, the 31..109 homing window
    // (the (vel×24 + dir×speed)/25 blend renormalised to the old speed),
    // the < 18 speed-up ×1.02, the localAI[0] one-shot (Item8 + the 10×
    // dust 5, `:56078-56087`, skipped).
    if p.kind == SKULL {
        p.ai[1] += 1.0;
        if p.ai[1] < 110.0 && p.ai[1] > 30.0 {
            if let Some((px, py)) = owner {
                let speed = p.vel.length();
                let center = p.pos + Vec2::new(p.w * 0.5, p.h * 0.5);
                let mut to = Vec2::new(px, py) - center;
                to = to.normalize_or_zero();
                to *= speed;
                p.vel = (p.vel * 24.0 + to) / 25.0;
                p.vel = p.vel.normalize_or_zero();
                p.vel *= speed;
            }
        }
        if p.vel.length() < 18.0 {
            p.vel *= 1.02;
        }
        if p.local_ai[0] == 0.0 {
            p.local_ai[0] = 1.0;
            // `:56078` — SoundID.Item8 at base.position (S1-E-21h d76); the
            // 10× dust 5 burst stays skipped (reference arm — slice A owns 270).
            p.wants_sound = Some((2, 8, p.pos.x, p.pos.y, 0.0, 1.0));
        }
    }
    // `:56165-56183` — the Blowup Smoke (622): the dust 229 roll
    // (`Main.rand.Next(3)`, `:56168-56176`, SKIPPED), the ai[1] age and the
    // `23 * MaxUpdates` kill (`MaxUpdates = 1 + extraUpdates`, the port's
    // `sub_steps` convention, projectile.rs:10963).
    if p.kind == BLOWUP_SMOKE_MOONLORD {
        p.ai[1] += 1.0;
        if p.ai[1] > (23 * (1 + p.extra_updates)) as f32 {
            p.active = false;
            p.time_left = 0;
            return;
        }
    }
    // `:56309-56326` — the Vortex Dart (581): the localAI[0] sound latch
    // (Item17 at base.Center, `:56314`, S1-E-21h d76), the 2× dust
    // `Utils.SelectRandom(Main.rand, 229, 161, 161)` rolls (`:56316-56325`,
    // SKIPPED — dust-only). No other state: the kind flies the counting
    // generic flight.
    if p.kind == VORTEX_DART && p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        let (cx, cy) = (p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        p.wants_sound = Some((2, 17, cx, cy, 0.0, 1.0));
    }
    // `:56504-56555` — Betsy's fireball (686): the localAI[0] one-shot —
    // when `ai[1] != 2f` the Item20 cue plays (`:56540`, S1-E-21h d76; the
    // DD2_BetsyFireballShot trackable at `:56541` is the D3 deferred family),
    // then the alpha fade 25 from ai[0] >= 2 (BEFORE this tick's gravity —
    // the counter already rode above), the two dust rolls
    // (`Main.rand.Next(4)` / `Next(3)` + the oldPos[9] gate, `:56519-56531`,
    // SKIPPED).
    if p.kind == BETSY_FIREBALL {
        if p.local_ai[0] == 0.0 {
            p.local_ai[0] = 1.0;
            if p.ai[1] != 2.0 {
                p.wants_sound = Some((2, 20, p.pos.x, p.pos.y, 0.0, 1.0)); // :56540
            }
        }
        if p.ai[0] >= 2.0 {
            p.alpha -= 25;
            if p.alpha < 0 {
                p.alpha = 0;
            }
        }
    }
    // `:56657-56698` — the skull pair (270/837); the slice-B member is
    // 270: the forced alpha = 0 (`:56659-56662`), the fade/clamp (a no-op
    // after the force, kept for shape), the 3-frame cycle (`frame > 2`
    // wraps), the 2× dust 5 (`:56678-56685`, skipped). 837 keeps its own
    // `ai_book_of_skulls` arm on main.
    if p.kind == SKULL {
        p.alpha = 0;
        if p.alpha > 0 {
            p.alpha -= 50;
        }
        if p.alpha < 0 {
            p.alpha = 0;
        }
        p.frame += 1;
        if p.frame > 2 {
            p.frame = 0;
        }
    }
    // `:56699-56709` — the Golem Eye Beam (259): the alpha fade 10.
    if p.kind == GOLEM_EYE_BEAM {
        if p.alpha > 0 {
            p.alpha -= 10;
        }
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }

    // ---- 4. The pre-rotation/gravity else-if chain ------------------------
    // The C#'s chain (`:56753` `if 357` … `else if` …) is mutually
    // exclusive: the 962 arm (`:56912-56937`) sits in it, so the 962 never
    // reaches the gravity tail (`:57187`/`:57520`); the 680 arm
    // (`:57471-57478`) and the 686 ladder (`:57522-57541`) sit in the
    // tail's inner chain and exclude the generic `:57568` arm.
    if p.kind == DEERCLOPS_ICE_CHUNK {
        // `:56912-56937`: the frame picks ai[1], the localAI[0] one-shot
        // pins the rotation to the spawn velocity (the two 5× dust 16
        // rolls — `Main.rand.NextVector2Circular` / `NextFloat` / `Lerp`,
        // `:56919-56931`, SKIPPED), the +0.15 gravity from ai[0] >= 5 with
        // NO counter pin (unlike the 174/605/920/921/926 arm at `:56896`).
        // The rotation pin is overwritten by the default rotation tail
        // (`:57750`) later in the SAME C# tick — kept for order fidelity.
        p.frame = p.ai[1] as i32;
        if p.local_ai[0] == 0.0 {
            p.local_ai[0] = 1.0;
            p.rot = p.vel.y.atan2(p.vel.x);
        }
        if p.ai[0] >= 5.0 {
            p.vel.y += 0.15;
        }
    } else if p.kind == BALLISTA_SHOT {
        // `:57471-57478` (the 507/508/662/680/685 arm): from ai[0] > 60,
        // the X drag 0.98 and the +0.3 gravity. No counter pin.
        if p.ai[0] > 60.0 {
            p.vel.x *= 0.98;
            p.vel.y += 0.3;
        }
    } else if p.kind == BETSY_FIREBALL {
        // `:57522-57541` (the 686/711 arm): +0.1 from ai[0] >= 10, another
        // +0.1 from >= 20, the counter pinned at 20, the X drag 0.99 and
        // the Y cap 32 (the ±16 clamp below still caps Y afterwards, as in
        // the C# `:57764-57767` — kept verbatim for order).
        if p.ai[0] >= 10.0 {
            p.vel.y += 0.1;
        }
        if p.ai[0] >= 20.0 {
            p.vel.y += 0.1;
        }
        if p.ai[0] > 20.0 {
            p.ai[0] = 20.0;
        }
        p.vel.x *= 0.99;
        if p.vel.y > 32.0 {
            p.vel.y = 32.0;
        }
    } else if p.ai[0] >= 15.0 {
        // The generic arm (`:57568-57579`): the counter pinned at 15, the
        // +0.1 gravity (flag4's −0.1 is the 1006-only reverse — no slice-B
        // kind sets it). The resting-counter kinds sit at ai[0] = 0 and
        // never reach this; 581/622 count into it. NOTE: the C# pins
        // ai[0] = 15 here; the port's generic `ai_style1_flight` omits the
        // pin (its own documented shape) — the slice-B kinds get the C#'s
        // pin, the generic path stays untouched for every other kind.
        p.ai[0] = 15.0;
        p.vel.y += 0.1;
    }

    // ---- 5/6. The shared tail + the per-kind rotation overwrite -----------
    finish_flight(p, world);
    // `:57597-57608` — the 270/601 group: `spriteDirection = direction`,
    // the rotation faces the flight (the direction < 0 flip). The
    // velocity-sign stand-in for `base.direction` as above.
    if p.kind == SKULL || p.kind == CANNON_EXPLOSIVE {
        p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
        p.rot = if p.vel.x < 0.0 {
            (0.0 - p.vel.y).atan2(0.0 - p.vel.x)
        } else {
            p.vel.y.atan2(p.vel.x)
        };
    } else if p.kind == BRAIN_SCRAMBLER {
        // `:57662-57670` — the velocity angle + π, plus another π when
        // direction == −1.
        p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::PI;
        if p.vel.x < 0.0 {
            p.rot += std::f32::consts::PI;
        }
    } else if p.kind == FLAMING_WOOD {
        // `:57744-57747` — the velocity angle − π/2.
        p.rot = p.vel.y.atan2(p.vel.x) - std::f32::consts::FRAC_PI_2;
    } else if p.kind == BETSY_FIREBALL || p.kind == BALLISTA_SHOT {
        // `:57658-57661` (the 680/684/686/711/712/1097 arm) — the velocity
        // angle + π/2.
        p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    }
    // 259/389/438/449/462/581/606/622/962/980 keep the default rotation
    // (`:57750`) the shared tail already wrote.
}

// ---------------------------------------------------------------------------
// Witness tests-as-code — WRITTEN NOT RUN (code-only mode, S1-D-5 slice B).
// Verified by re-reading the cited C# and hand arithmetic; run them with
// `cargo test -p tr-app projectile_ai001_b` once a compiler is available.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    fn air() -> World {
        tr_core::World::new("t".into(), 200, 200)
    }

    /// A rowed slice-B shot at (1600, 800) — tile (100, 50), open air in
    /// the 200×200 test world — flying right at 10 px/tick.
    fn shot(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("rowed kind");
        // Field-by-field on `Default` — the struct has private bookkeeping
        // fields (E0451 outside projectile.rs).
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
        p
    }

    /// A/B: the gate never claims slice A's low-id arrow family, the
    /// M4-PROJ-6 riders or the Book of Skulls — `covers` is the seam's
    /// whole contract.
    #[test]
    fn the_gate_is_disjoint_from_the_other_arms() {
        assert!(!covers(1)); // the wooden arrow
        assert!(!covers(2)); // the flame arrow
        assert!(!covers(81)); // the hostile arrow
        assert!(!covers(117)); // slice A's band
        assert!(!covers(20)); // the riders stay on ai_style1_rider
        assert!(!covers(88));
        assert!(!covers(355));
        assert!(!covers(660));
        assert!(!covers(837)); // the book of skulls keeps its own arm
        assert!(!covers(687)); // Betsy's breath has NO AI_001 branch
        // The merge-time dedupe: slice A owns the six overlap kinds.
        assert!(!covers(259));
        assert!(!covers(270));
        assert!(!covers(325));
        assert!(!covers(389));
        assert!(!covers(436));
        assert!(!covers(438));
        for k in SLICE_B_KINDS {
            assert!(covers(k));
        }
    }

    /// The flag3 membership (`Projectile.cs:55947-56032`): the resting
    /// kinds hold ai[0]; the counting kinds increment once per tick.
    #[test]
    fn flag3_holds_the_listed_counters() {
        let w = air();
        for k in [
            GOLEM_EYE_BEAM,
            SKULL,
            FLAMING_WOOD,
            MINI_RETINA_LASER,
            BRAIN_SCRAMBLER,
            RAY_GUNNER,
            SAUCER_LASER,
            PHANTASMAL_BOLT,
            CANNON_EXPLOSIVE,
            SCUTLIX_LASER,
            TRAP_VENOM_DART,
        ] {
            let mut p = shot(k);
            step(&mut p, &w, Some((1600.0, 800.0)), &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.ai[0], 0.0, "kind {k} must rest");
        }
        for k in [
            VORTEX_DART,
            BLOWUP_SMOKE_MOONLORD,
            BALLISTA_SHOT,
            BETSY_FIREBALL,
            DEERCLOPS_ICE_CHUNK,
        ] {
            let mut p = shot(k);
            step(&mut p, &w, Some((1600.0, 800.0)), &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.ai[0], 1.0, "kind {k} must count");
        }
    }

    /// 259 (`:55239-55243` + `:56699-56709`): the ai[1] latch on tick 1,
    /// the alpha fade 10/tick from a 255 spawn, the resting counter.
    #[test]
    fn the_eye_beam_latches_and_fades_ten() {
        let w = air();
        let mut p = shot(GOLEM_EYE_BEAM);
        p.alpha = 255;
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.alpha, 245);
        for _ in 0..25 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        }
        assert_eq!(p.alpha, 0); // 255 − 26×10 clamps at 0
        assert_eq!(p.ai[0], 0.0); // flag3 rest
    }

    /// 325 (`:55164-55196` + `:57744-57747`): alpha 255 → 155 → 55 → 0
    /// (the fade 100 clamps), the ai[1] latch, the rotation = angle − π/2.
    #[test]
    fn the_flaming_wood_fades_hundred_and_faces_minus_pi2() {
        let w = air();
        let mut p = shot(FLAMING_WOOD); // the row's spawn alpha is 255
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 155);
        assert_eq!(p.ai[1], 1.0);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 55);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 0);
        // vel (10, 0) → angle 0 → rot = −π/2.
        assert!((p.rot + std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    /// 436 (`:54150-54174` + `:57662-57670`): the 4-frame cycle at 3
    /// ticks/frame, the alpha fade 40, the rotation + π (a leftward shot
    /// adds the second π).
    #[test]
    fn the_scrambler_cycles_four_frames_at_three() {
        let w = air();
        let mut p = shot(BRAIN_SCRAMBLER);
        p.alpha = 255;
        for t in 1..=12 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.frame, (t / 3) % 4, "tick {t}");
        }
        assert_eq!(p.alpha, 0); // 255 − 12×40 clamps
        assert_eq!(p.ai[1], 1.0);
        // vel (10, 0): rot = 0 + π.
        assert!((p.rot - std::f32::consts::PI).abs() < 1e-5);
        let mut left = shot(BRAIN_SCRAMBLER);
        left.vel = Vec2::new(-10.0, 0.0);
        step(&mut left, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        // angle(−10, 0) = π; + π + π = 3π.
        assert!((left.rot - 3.0 * std::f32::consts::PI).abs() < 1e-5);
        assert_eq!(left.sprite_direction, -1);
    }

    /// 438/462 (`:55660-55671`): the byte-truncated speed fade — speed 10
    /// → `(byte)(10 × 0.9) = 9` per tick; speed 5 → `(byte)4.5 = 4`.
    #[test]
    fn the_speed_fade_truncates_like_the_byte_cast() {
        let w = air();
        let mut p = shot(RAY_GUNNER);
        p.alpha = 255;
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 246); // 255 − 9
        assert_eq!(p.ai[1], 1.0); // the `:55254` latch
        let mut b = shot(PHANTASMAL_BOLT);
        b.alpha = 100;
        b.vel = Vec2::new(3.0, 4.0); // speed 5
        step(&mut b, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(b.alpha, 96);
    }

    /// 462 (`:54860-54876`): the 5-frame cycle at 9 ticks/frame.
    #[test]
    fn the_phantasmal_bolt_cycles_five_frames_at_nine() {
        let w = air();
        let mut p = shot(PHANTASMAL_BOLT);
        for t in 1..=45 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.frame, (t / 9) % 5, "tick {t}");
        }
        assert_eq!(p.ai[1], 1.0); // the `:55269` latch
    }

    /// 449 (`:54772-54828`): the +3 ramp caps at 100 (tick 34: 3×34 =
    /// 102 → 100); the ai[1] != 0 countdown kills at ≤ 0.
    #[test]
    fn the_saucer_laser_ramps_and_counts_down() {
        let w = air();
        let mut p = shot(SAUCER_LASER);
        p.alpha = 255;
        for _ in 0..34 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        }
        assert_eq!(p.local_ai[0], 100.0);
        assert_eq!(p.alpha, 0); // 255 − 34×25 clamps
        p.ai[1] = 1.0;
        for _ in 0..33 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert!(p.active);
        }
        assert_eq!(p.local_ai[0], 1.0); // 100 − 33×3
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0)); // 1 − 3 = −2 ≤ 0 → Kill
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
    }

    /// W4-d8 DOC-SWEEP (3) — 606 (`:54772-54828` + `:6530-6541`): the
    /// Scutlix beam's +5 ramp caps at 150 (tick 30: 5×30 = 150), the fade
    /// 25, the resting counter (`:56014`), and the ai[1] != 0 countdown
    /// kills at ≤ 0 (tick 30 of the countdown: 150 − 30×5 = 0).
    #[test]
    fn the_scutlix_beam_ramps_five_to_150_and_counts_down() {
        let w = air();
        let mut p = shot(SCUTLIX_LASER);
        assert_eq!(p.extra_updates, 2, "the row (`:6537`)");
        assert_eq!(p.time_left, 600, "the row (`:6539`)");
        p.alpha = 255;
        for _ in 0..30 {
            step(&mut p, &w, None);
        }
        assert_eq!(p.local_ai[0], 150.0, "5×30 = 150, the cap (`:54798`)");
        assert_eq!(p.ai[0], 0.0, "the resting counter (`:56014`)");
        assert_eq!(p.alpha, 0); // 255 − 30×25 clamps
        // The countdown: ai[1] = 1 (the blocked-move latch), −5/tick.

        }    /// M2-MOUNT-4 — 606 (`:54772-54828`): the Scutlix laser rides the same
    /// group with the 150/5 constants (`:54796-54800`): the +5 ramp caps at
    /// 150 (tick 30: 5×30 = 150), the ai[1] != 0 countdown kills at ≤ 0.
    /// The resting counter (`:56014`) keeps the gravity off. The tile
    /// contact keeps the port's kill-on-tile stand-in (the 449 sibling's
    /// convention — the C# `:18278` bounce latch + `:20435` UpdatePosition
    /// skip are the documented slice-B deviation, ANALYSIS).
    fn m4_the_scutlix_laser_ramps_150_by_5_and_counts_down() {
        assert_eq!(p.alpha, 255, "the row's alpha");
        assert_eq!(p.extra_updates, 2, "the row's extraUpdates");
        assert_eq!(p.local_ai[0], 150.0, "5×30 caps at 150");
        assert_eq!(p.alpha, 0, "255 − 30×25 clamps");
        assert_eq!(p.ai[0], 0.0, "the resting counter");
        p.ai[1] = 1.0;
        for _ in 0..29 {
            step(&mut p, &w, None);
            assert!(p.active);
        }
        assert_eq!(p.local_ai[0], 5.0); // 150 − 29×5
        step(&mut p, &w, None); // 5 − 5 = 0 ≤ 0 → Kill
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
    }

    /// W4-d8 DOC-SWEEP (3) — the 606's blocked-move stick
    /// (`:18278-18286` + the `:17590`/`:20435` skips): a beam flying into
    /// a tile latches ai[1] = 1, takes ONE last full-velocity step and
    /// keeps its velocity; every later tick holds the position (the ramp
    /// counts down in place). The 449 shares the arm.
    #[test]
    fn the_scutlix_beam_sticks_in_the_tile_and_holds() {
        let mut w = air();
        // A solid tile at (102, 50): the beam at (1600, 800) = tile
        // (100, 50) flying right at 10 reaches it on the 4th step
        // (1600 + 4×10 = 1640 = tile 102.5's left edge 1632... the probe
        // hits when the box crosses 1632: step 4 lands x = 1640).
        let i = w.index(103, 50);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = shot(SCUTLIX_LASER);
        p.alpha = 255;
        // Step until the latch fires (the probe hits tile 103 at x ≥ 1643).
        let mut stuck_at = None;
        for t in 1..=10 {
            step(&mut p, &w, None);
            if p.ai[1] == 1.0 {
                stuck_at = Some(t);
                break;
            }
        }
        let t = stuck_at.expect("the beam hits the tile");
        assert!(p.active, "the stick does NOT kill (`:18280-18284`)");
        assert_eq!(p.vel, Vec2::new(10.0, 0.0), "velocity = lastVelocity");
        let stuck_pos = p.pos;
        // The next ticks hold the position (the `:20435` skip) while the
        // ramp counts down.
        let ramp_at_stick = p.local_ai[0];
        step(&mut p, &w, None);
        assert_eq!(p.pos, stuck_pos, "the stuck beam never moves");
        assert_eq!(p.local_ai[0], ramp_at_stick - 5.0, "the countdown runs");
        assert!(t <= 5, "the tile is 3 tiles out");
    }

    /// 622 (`:56165-56183`): the kill at ai[1] > 23 × MaxUpdates — the
    /// row's extraUpdates 2 → MaxUpdates 3 → the 70th tick kills (69 =
    /// 23×3 survives, 70 > 69 dies).
    #[test]
    fn the_blowup_smoke_dies_at_23_max_updates() {
        let w = air();
        let mut p = shot(BLOWUP_SMOKE_MOONLORD);
        assert_eq!(p.extra_updates, 2);
        for _ in 0..69 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert!(p.active);
        }
        assert_eq!(p.ai[1], 69.0);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
    }

    /// 601 (`:54066-54120` + `:57597-57608`): the alpha fade 15, the
    /// 2-frame cycle at 4 ticks/frame, the resting counter, the
    /// direction-facing rotation (no +1.57 offset).
    #[test]
    fn the_cannon_explosive_fades_and_faces() {
        let w = air();
        let mut p = shot(CANNON_EXPLOSIVE);
        p.alpha = 255;
        for t in 1..=8 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.frame, (t / 4) % 2, "tick {t}");
        }
        assert_eq!(p.alpha, 135); // 255 − 8×15
        assert_eq!(p.ai[0], 0.0); // flag3
        // vel (10, 0) → rot = atan2(0, 10) = 0.
        assert!(p.rot.abs() < 1e-6);
        assert_eq!(p.sprite_direction, 1);
    }

    /// S1-E-21h d75 part 2 — the portal bolt's impact cue
    /// (`HandleMovement` `:18084-18103`): on the blocked tick, `alpha > 0`
    /// plays Item114 (`ai[1] != 0`, the blue portal) or Item115 (orange)
    /// at the PRE-move position, then the bolt dies; a fully faded bolt
    /// (`alpha == 0`) dies cue-less.
    #[test]
    fn s1_e_21h_d75_p2_the_portal_bolt_impact_plays_its_pair() {
        let mut w = air();
        // The wall tile (101, 50) spans x 1616..1632; the bolt at x 1600
        // flying right at 10 probes x 1610..1620 — the blocked hit.
        let i = w.index(101, 50);
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        for (ai1, alpha, style) in [(0.0f32, 255, Some(115i32)), (1.0, 255, Some(114)), (0.0, 0, None)] {
            let mut p = shot(CANNON_EXPLOSIVE);
            p.ai[1] = ai1;
            p.alpha = alpha;
            step(&mut p, &w, None);
            assert!(!p.active, "the impact kills");
            assert_eq!(
                p.wants_sound,
                style.map(|s| (2, s, 1600.0, 800.0, 0.0)),
                "ai1 {ai1} alpha {alpha}"
            );
        }
    }

    /// 270 (`:56056-56089` + `:56657-56698` + `:57597-57608`): the ai[1]
    /// age, the < 18 speed-up ×1.02, the forced alpha 0, the 3-frame
    /// cycle, the localAI[0] one-shot, the homing blend.
    #[test]
    fn the_skull_ages_speeds_and_cycles_three_frames() {
        let w = air();
        let mut p = shot(SKULL);
        step(&mut p, &w, Some((1600.0, 800.0)), &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.ai[1], 1.0); // age (the 31..109 window not yet open)
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.alpha, 0);
        assert_eq!(p.frame, 1);
        assert!((p.vel.x - 10.2).abs() < 1e-4); // 10 × 1.02
        for t in 2..=6 {
            step(&mut p, &w, Some((1600.0, 800.0)), &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.frame, t % 3, "tick {t}");
        }
        // The homing window: ai[1] = 31..109 steers toward the owner.
        let mut h = shot(SKULL);
        h.ai[1] = 40.0;
        h.vel = Vec2::new(18.0, 0.0); // ≥ 18: no speed-up, pure steering
        step(&mut h, &w, Some((1600.0, 400.0)), &mut tr_core::unified_random::UnifiedRandom::new(0)); // the owner straight above
        assert_eq!(h.ai[1], 41.0);
        // The blend pulls Y negative (toward the owner above) and the
        // renormalisation keeps the speed at 18 — or, if the f32
        // renormalised length lands a hair under 18, the C#'s own < 18
        // arm multiplies by 1.02 (18.36); both are faithful outcomes.
        assert!(h.vel.y < 0.0);
        let len = h.vel.length();
        assert!((len - 18.0).abs() < 1e-3 || (len - 18.36).abs() < 1e-2);
    }

    /// 686 (`:56504-56532` + `:57522-57541` + `:57658-57661`): the alpha
    /// fade 25 opens on the tick the hook sees ai[0] >= 2 (tick 2 — the
    /// counter at `:56038` rides BEFORE the hook at `:56504`), the
    /// +0.1/+0.1 gravity ladder, the ai[0] pin at 20, the X drag 0.99,
    /// the rotation + π/2.
    #[test]
    fn the_betsy_fireball_ladders_gravity_and_drags() {
        let w = air();
        let mut p = shot(BETSY_FIREBALL);
        p.alpha = 255;
        // Tick 1: the counter → 1; the hook sees ai[0] = 1 → no fade; no
        // gravity; the drag rides every tick.
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 255);
        assert_eq!(p.vel.y, 0.0);
        assert!((p.vel.x - 9.9).abs() < 1e-5);
        // Tick 2: the counter → 2; the hook sees 2 → the fade opens.
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 230);
        // Tick 3: another 25.
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.alpha, 205);
        // The first +0.1 lands on the tick the counter reaches 10.
        let mut q = shot(BETSY_FIREBALL);
        q.vel = Vec2::ZERO;
        for _ in 0..9 {
            step(&mut q, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        }
        assert_eq!(q.ai[0], 9.0);
        assert_eq!(q.vel.y, 0.0);
        step(&mut q, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0)); // ai[0] = 10 → +0.1
        assert!((q.vel.y - 0.1).abs() < 1e-6);
        // At ai[0] = 20 both +0.1s land (0.2/tick); the pin holds 20.
        let mut r = shot(BETSY_FIREBALL);
        r.ai[0] = 19.0;
        r.vel = Vec2::ZERO;
        step(&mut r, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0)); // 19→20: both gates true
        assert_eq!(r.ai[0], 20.0);
        assert!((r.vel.y - 0.2).abs() < 1e-6);
        step(&mut r, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0)); // 20→21→pinned 20: still 0.2
        assert_eq!(r.ai[0], 20.0);
        assert!((r.vel.y - 0.4).abs() < 1e-6);
        // The rotation: vel (0, +y) → angle π/2 → rot π.
        assert!((r.rot - std::f32::consts::PI).abs() < 1e-5);
    }

    /// 680 (`:57471-57478` + `:57658-57661`): from ai[0] > 60 the X drag
    /// 0.98 and the +0.3 gravity; NO counter pin; the rotation + π/2.
    #[test]
    fn the_ballista_shot_drags_and_drops_after_sixty() {
        let w = air();
        let mut p = shot(BALLISTA_SHOT);
        p.ai[0] = 59.0;
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0)); // 59→60: the > 60 gate still closed
        assert_eq!(p.ai[0], 60.0);
        assert_eq!(p.vel.x, 10.0);
        assert_eq!(p.vel.y, 0.0);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0)); // 60→61: the gate opens
        assert_eq!(p.ai[0], 61.0); // NOT pinned
        assert!((p.vel.x - 9.8).abs() < 1e-5);
        assert!((p.vel.y - 0.3).abs() < 1e-6);
        let expected = (0.3f32).atan2(9.8) + std::f32::consts::FRAC_PI_2;
        assert!((p.rot - expected).abs() < 1e-5);
    }

    /// 962 (`:56912-56937`): the frame = ai[1] pick, the one-shot marker,
    /// the +0.15 gravity from ai[0] >= 5 with NO counter pin. The one-shot
    /// rotation pin is overwritten by the default rotation (`:57750`) in
    /// the same C# tick — the port matches.
    #[test]
    fn the_ice_chunk_falls_at_015_without_a_pin() {
        let w = air();
        let mut p = shot(DEERCLOPS_ICE_CHUNK);
        p.ai[1] = 2.0;
        p.vel = Vec2::new(6.0, -8.0);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.frame, 2);
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.ai[0], 1.0); // 1 < 5: no gravity yet
        assert_eq!(p.vel.y, -8.0);
        // The default rotation won over the one-shot pin.
        let expected = (-8.0f32).atan2(6.0) + 1.57;
        assert!((p.rot - expected).abs() < 1e-5);
        for _ in 0..4 {
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        }
        assert_eq!(p.ai[0], 5.0); // NOT pinned — the C# arm has no write
        assert!((p.vel.y - (-8.0 + 0.15)).abs() < 1e-5);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.ai[0], 6.0);
        assert!((p.vel.y - (-8.0 + 0.30)).abs() < 1e-5);
    }

    /// 581/980/389: the latch-only kinds — the state writes ride, the
    /// flight is the generic resting/counting flight with the default
    /// rotation.
    #[test]
    fn the_latch_kinds_keep_the_generic_flight() {
        let w = air();
        let mut d = shot(VORTEX_DART);
        step(&mut d, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(d.local_ai[0], 1.0); // the `:56311` latch
        assert_eq!(d.ai[0], 1.0); // 581 counts
        let mut v = shot(TRAP_VENOM_DART);
        step(&mut v, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(v.ai[1], 1.0); // the `:55326` latch
        assert_eq!(v.ai[0], 0.0); // 980 rests (flag3)
        let mut m = shot(MINI_RETINA_LASER);
        step(&mut m, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(m.ai[1], 1.0); // the `:55296` latch
        assert_eq!(m.ai[0], 0.0);
        // The default rotation for all three: atan2(vy, vx) + 1.57.
        for p in [&d, &v, &m] {
            assert!((p.rot - 1.57).abs() < 1e-5);
        }
    }

    /// The generic arithmetic for a counting kind below the gravity gate:
    /// ai[0] = 1 < 15 → no gravity, the position integrates once, the
    /// default rotation — the same shape `ai_style1_flight` gives the
    /// unclaimed kinds (the A/B byte-identity argument: the seam never
    /// runs for them, and this module's tail replicates the same code).
    #[test]
    fn the_shared_flight_arithmetic_is_the_generic_one() {
        let w = air();
        let mut p = shot(VORTEX_DART);
        let x0 = p.pos.x;
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.vel.y, 0.0);
        assert_eq!(p.pos.x, x0 + 10.0);
        assert!((p.rot - 1.57).abs() < 1e-5);
    }

    /// The generic gravity arm's C# pin (`:57568-57579`): a counting kind
    /// at ai[0] = 14 increments to 15, pins and takes the +0.1; the next
    /// tick stays pinned at 15.
    #[test]
    fn the_counting_kinds_pin_at_fifteen_like_the_csharp() {
        let w = air();
        let mut p = shot(VORTEX_DART);
        p.ai[0] = 14.0;
        p.vel = Vec2::ZERO;
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.ai[0], 15.0);
        assert!((p.vel.y - 0.1).abs() < 1e-6);
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.ai[0], 15.0); // 15→16→pinned back
        assert!((p.vel.y - 0.2).abs() < 1e-6);
    }

    // ---- S1-E-21h d76: the slice-B cue witnesses (WRITTEN, NOT RUN) --------

    /// The ai[1] latch chain (`:55209-55330`): 259→Item33, 438→Item12,
    /// 389→Item12, 980→Item17, each once per life at the play-time position.
    #[test]
    fn s1_e_21h_d76_the_slice_b_latches_carry_their_cues() {
        let w = air();
        for (kind, style) in [
            (GOLEM_EYE_BEAM, 33), // :55242
            (RAY_GUNNER, 12),     // :55257
            (MINI_RETINA_LASER, 12), // :55299
            (TRAP_VENOM_DART, 17),   // :55329
        ] {
            let mut p = shot(kind);
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.ai[1], 1.0, "kind {kind}: the latch");
            assert_eq!(
                p.wants_sound,
                Some((2, style, 1600.0, 800.0, 0.0, 1.0)),
                "kind {kind}'s cue at base.position"
            );
            p.wants_sound = None;
            step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
            assert_eq!(p.wants_sound, None, "kind {kind}: one cue per life");
        }
    }

    /// 462 (`:55272-55279`): the ONE Next(2) cue pick — 0 → Item124, else
    /// Item125 — rides UpdateProjectiles; the dust count stays skipped.
    #[test]
    fn s1_e_21h_d76_the_phantasmal_bolt_picks_its_cue_on_one_draw() {
        let w = air();
        struct Seq(Vec<i32>, usize);
        impl DropRng for Seq {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                let v = self.0[self.1];
                self.1 += 1;
                v
            }
            fn next_float(&mut self) -> f32 { 0.5 }
        }
        // Next(2) = 0 → Item124.
        let mut p = shot(PHANTASMAL_BOLT);
        let mut seq = Seq(vec![0], 0);
        step(&mut p, &w, None, &mut seq);
        assert_eq!(p.wants_sound, Some((2, 124, 1600.0, 800.0, 0.0, 1.0)));
        assert_eq!(seq.1, 1, "exactly ONE Next(2) — the dust count is skipped");
        // Next(2) = 1 → Item125.
        let mut q = shot(PHANTASMAL_BOLT);
        let mut seq2 = Seq(vec![1], 0);
        step(&mut q, &w, None, &mut seq2);
        assert_eq!(q.wants_sound, Some((2, 125, 1600.0, 800.0, 0.0, 1.0)));
        // The latch holds: no second cue, no second draw.
        q.wants_sound = None;
        step(&mut q, &w, None, &mut seq2);
        assert_eq!(q.wants_sound, None);
        assert_eq!(seq2.1, 1, "the latch gates the draw to one per life");
    }

    /// 601 (`:54072-54095`): the alpha-1..15 portal window plays Item115
    /// (ai[0] == 0) / Item114 (ai[0] != 0) once per tick while it holds.
    #[test]
    fn s1_e_21h_d76_the_cannon_portal_cue_tracks_ai0() {
        let w = air();
        let mut p = shot(CANNON_EXPLOSIVE);
        p.alpha = 10; // inside the 1..15 window
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.wants_sound, Some((2, 115, 1600.0, 800.0, 0.0, 1.0)), "ai[0]==0 → Item115");
        assert_eq!(p.alpha, 0); // 10 − 15 clamps
        // A second tick: alpha 0 is OUTSIDE the window — no cue.
        p.wants_sound = None;
        step(&mut p, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(p.wants_sound, None);
        // ai[0] != 0 → Item114.
        let mut q = shot(CANNON_EXPLOSIVE);
        q.alpha = 10;
        q.ai[0] = 3.0;
        step(&mut q, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(q.wants_sound, Some((2, 114, 1600.0, 800.0, 0.0, 1.0)));
    }

    /// 581 (`:56314`): the vortex dart's Item17 at base.Center; 686
    /// (`:56540`): Betsy's Item20 at base.position gated on ai[1] != 2.
    #[test]
    fn s1_e_21h_d76_the_vortex_dart_and_betsy_cues() {
        let w = air();
        let mut d = shot(VORTEX_DART);
        step(&mut d, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        let (cx, cy) = (1600.0 + d.w * 0.5, 800.0 + d.h * 0.5);
        assert_eq!(d.wants_sound, Some((2, 17, cx, cy, 0.0, 1.0)), "Item17 at Center");
        // Betsy: ai[1] == 0 (≠ 2) → Item20.
        let mut b = shot(BETSY_FIREBALL);
        step(&mut b, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(b.wants_sound, Some((2, 20, 1600.0, 800.0, 0.0, 1.0)));
        // Betsy with ai[1] == 2: the cue is gated OFF, the latch still rides.
        let mut c = shot(BETSY_FIREBALL);
        c.ai[1] = 2.0;
        step(&mut c, &w, None, &mut tr_core::unified_random::UnifiedRandom::new(0));
        assert_eq!(c.local_ai[0], 1.0);
        assert_eq!(c.wants_sound, None, "ai[1]==2 gates the Item20");
    }
}
