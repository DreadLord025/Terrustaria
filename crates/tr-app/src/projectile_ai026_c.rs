//! AI_026 slice C (S1-D-4b-16) - the ground-pet idle/walk/jump machine
//! of the aiStyle-26 mega-family (`Projectile.cs:58102-65024`), landed
//! for its one row-carrying unclaimed type: the **266** slime minion
//! (`ProjectileID.BabySlime`, the Slime Staff's summon).
//!
//! # The census (the whole remaining block was read)
//!
//! Slices A (`task/S1-D-4b-10-melee-a`, the 14 early-return flyers
//! `:59120-60091`) and B (`task/S1-D-4b-11-melee-b`, the 885/889 spinning
//! boss heads `:60092-60253`) are landed. The remaining block
//! `:60254-65024` is the shared ground machine every non-early-return
//! type falls into. Reading it WHOLE, the type ids it dispatches on are:
//!
//! * **ground pets**: 111 (bunny), 112 (penguin), 127 (turtle), 200
//!   (lizard), 208 (parrot), 209 (truffle), 210 (sapling), 236 (dino),
//!   268 (eye spring), 269 (baby snowman), 313 (pet spider - the WALL
//!   pet, not a minion), 314 (squashling), 319 (black cat), 324 (corrupt
//!   sapling), 334 (puppy), 353 (baby grinch), 398 (mini minotaur), 499
//!   (baby face monster), 765 (sugar glider), 816 (fennec fox), 821
//!   (baby imp), 825 (red panda), 854 (plantero), 858 (dynamite kitten),
//!   859 (baby werewolf), 860 (shadow mimic chest), 875 (volt bunny),
//!   881 (king slime pet), 884 (brain pet), 890 (plantera pet), 891
//!   (golem pet), 897 (everscream pet), 899 (martian pet), 900 (ogre
//!   pet), 934 (queen slime pet), 956 (Bernie), 958 (deerclops pet),
//!   959 (pig pet), 960 (Chester), 994 (junimo), 998 (blue chicken),
//!   1003 (spiffo), 1004 (caveling), 1027 (swarm biter), 1095/1096
//!   (chillet pair);
//! * **minions**: 191-194 (the pygmies, `flag6`), 266 (this slice),
//!   390-392 (the spiders, `flag7`);
//! * **Pal summons**: 1094/1113 (foxsparks pair, `flag8` - the
//!   `ai[0] >= 1000` held-flamethrower state `:59027-59116` returns
//!   early like the flyers).
//!
//! Of ALL of those, the port's `proj_data` carries a row for exactly
//! ONE: **266** (`projectile/data.rs:2237`, `Projectile.cs:3516-3528`:
//! 24x16, aiStyle 26, penetrate -1, `timeLeft *= 5` -> 18000, `minion =
//! true`, `minionSlots = 1`, `usesIDStaticNPCImmunity` + cooldown 12,
//! `netImportant`). The gate routes ONLY the 266; every other type
//! keeps the byte-identical no-arm fall-through (they cannot spawn
//! without a row anyway). The remaining types are documented for the
//! follow-up slices - their per-type tails are catalogued in
//! [`ground_machine_census`] below and in ANALYSIS "AI_026 slice C".
//!
//! # RNG order (the golden rule)
//!
//! The 266 draws **no `Main.rand` anywhere in `AI_026`**:
//!
//! * the head: the pygmy `Next(2, 10)` (`:58919`) is `flag6`-gated -
//!   not the 266; the Pal sound/appear arms (`:58136-58152`) are
//!   `flag8`-gated;
//! * the fly-out arm (`:60387-61548`): the `Next(12, 18)` frame roll
//!   (`:60623`) is `flag6`-gated; the dust chain (`:61458-61548`)
//!   EXCLUDES the 266 by type (`:61541`), so its `NewDust` internal
//!   rolls never fire; the NPC chase scan (`:60418-60435`) draws
//!   nothing;
//! * the ground arm (`:61550-65015`): the pygmy spear spawn's three
//!   draws (`:61698-61705`) are `flag10`-gated; the 266's own scan
//!   (`:61755-62004`), hop ladder (`:62271-62274`), step-up and frames
//!   (`:64435-64494`) are pure arithmetic - the 268's dust hop
//!   (`:63864-63869`) and the Pal dust (`:63725`) are other types'.
//!
//! The ONE draw the 266 pays lives in `Update`'s blocked-move chain:
//! `Main.rand.Next(3)` at `Projectile.cs:17948` - the FIRST operand of
//! the `&&`, so it fires unconditionally whenever the movement was
//! blocked (`flag`, `:17896-17899`), even when the owner is dead or the
//! damage is 0. The 266 passes `CanCutTiles()` (`:15323-15334`: 266 is
//! on no refusal list), so the draw is CONSUMED on the pass's stream
//! (`ground_movement`) while the cut itself skips with the unported
//! tile-cut layer (the `dd2_sentry_step` precedent, ANALYSIS). The
//! witness `the_blocked_move_consumes_the_tile_cut_roll` pins it.
//!
//! # Documented stand-ins
//!
//! * `Collision.SolidCollision` -> [`crate::projectile::box_solid`] (the
//!   5-point approximation, M4-PROJ-8 precedent).
//! * `Collision.CanHit` -> [`crate::projectile::can_hit`] (the Manhattan
//!   line walk, the Book of Skulls precedent).
//! * `NPC.CanBeChasedBy(this)` -> [`crate::projectile::chaseable`] (the
//!   three observable fields, the minion-scan precedent).
//! * `OwnerMinionAttackTargetNPC` (`Projectile.cs:566-576`) -> the
//!   owner's `minion_attack_target_npc` whip mark, exactly the property.
//! * `Collision.TileCollision(..., ignoreDoors: Main.projPet[266] =
//!   true, Main.cs:8739)` -> the port's `tile_collision` carries the
//!   fall pair only - doors block it (the `pirate_movement` precedent).
//! * `base.wet` -> the slot carries no liquid-wet flags (the slice A
//!   precedent), so the 266's wet-swim branch (`:64475-64485`) folds to
//!   the dry `velocity.Y += 0.4f` always.
//! * The `UpdatePosition` slope pass (`:20447-20474`,
//!   `correctSlopeCollision = Main.projPet[266] = true`, `:707`) is
//!   DEFERRED (the twins/pirate precedent - the port's `slope_collision`
//!   is the non-correcting variant).
//! * `netUpdate` -> the single-player no-op (the pool's convention).
//! * The petting flag (`flag9`, `:58118`) reads `player.petting` -
//!   unported (the sleep/sit precedent), folds false: the
//!   `(ai[1] != 0) | flag9` direction clear (`:62005-62009`) keeps its
//!   `ai[1]` half.
//! * `Main.player[owner].slime` IS live (`player.rs:505` - the buff-216
//!   grant side); the keep-alive clear/pin is exact.
//! * A blocked move only STOPS the minion: the default
//!   `position += velocity; Kill()` (`:19851-19852`) sits inside the
//!   `!correctSlopeCollision` gate (`:18398`), which the 266's
//!   `Main.projPet[266] = true` (`Main.cs:8739` -> `:707`) closes (the
//!   `pirate_movement` precedent).

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::{
    Projectile, SLIME_MINION, box_solid, can_hit, chaseable, proj_frames, safe_normalize,
    solid_tile,
};

/// The slice-C type set - the 266 slime minion, the only unclaimed
/// AI_026 type with a `proj_data` row (the census above). Every other
/// aiStyle-26 kind keeps the pre-existing no-arm fall-through,
/// byte-identical.
pub(crate) const AI_026_C_KINDS: [u16; 1] = [SLIME_MINION];

/// The owner reads the machine shares - a snapshot of
/// `Main.player[this.owner]` taken once per tick. Nothing in the 266's
/// path writes the player fields except the keep-alive death clear,
/// which no later read consumes, so the C#'s re-reads are equivalent.
struct OwnerView {
    center: Vec2,
    /// `position.Y + height` - the grounded comparisons.
    bottom: f32,
    vel: Vec2,
    /// `Player.direction` (the port's `facing`).
    dir: i32,
    /// `Player.MinionAttackTargetNPC` - the whip mark
    /// (`OwnerMinionAttackTargetNPC`, `Projectile.cs:566-576`).
    attack_target: i32,
}

impl OwnerView {
    fn capture(p: &Player) -> Self {
        let (cx, cy) = p.center();
        Self {
            center: Vec2::new(cx, cy),
            bottom: p.y + p.height,
            vel: Vec2::new(p.vx, p.vy),
            dir: p.facing,
            attack_target: p.minion_attack_target_npc,
        }
    }
}

/// The dispatch seam: `AI()`'s `else if (this.aiStyle == 26)
/// this.AI_026();` (`Projectile.cs:26416-26419`), gated to the slice-C
/// kinds and chained AFTER slices A/B (the kind sets are disjoint).
/// Returns `false` for every other kind - the caller keeps its
/// `_ => {}` fall-through, so non-slice routing is byte-identical.
///
/// On a slice-C tick this runs `Update`'s pre-AI `gfxOffY` decay
/// (`:16740-16764`), the observable `AI_026` head (the owner-inactive
/// kill `:58104-58108`, the keep-alive `:58449-58459`, the `flag11` pin
/// `:58944-58947`, the direction band `:58948-58960` + the
/// `flag12`/`flag13` forcing `:59016-59026`), the ground machine
/// (`:60254-65015` with the non-266 arms folded out), the
/// `oldPosition` latch (`:17013`) and `HandleMovement`'s collision +
/// integrate (`ground_movement`).
pub(crate) fn ai026_c_tick(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    player: Option<&mut Player>,
    rng: &mut dyn DropRng,
) -> bool {
    if !AI_026_C_KINDS.contains(&p.kind) {
        return false;
    }
    // `:16740-16764` - the StepUp draw-lift decay rides BEFORE the AI
    // (the C# order; `pirate_movement` folds it after its arm instead -
    // this slice keeps the source order).
    gfx_decay(p);
    // `:58104-58108` - `if (!Main.player[this.owner].active) { this.active
    // = false; return; }`. The port's `None` player is the unresolved /
    // inactive owner (the slice A/B dead-owner precedent).
    let Some(player) = player else {
        p.active = false;
        return true;
    };
    // `:58449-58459` - the 266 keep-alive: death clears `player.slime`
    // FIRST, then a standing flag pins `timeLeft = 2` (the C# reads the
    // flag after the death write - the slice B order).
    if player.dead {
        player.slime = false;
    }
    if player.slime {
        p.time_left = 2;
    }
    // `:58200-58202` + `:58944-58947` - `flag11 = ai[0] == -1 || -2`
    // pins `timeLeft = 2` (the despawn request the buff pass writes).
    let flag11 = p.ai[0] == -1.0 || p.ai[0] == -2.0;
    if flag11 {
        p.time_left = 2;
    }
    let owner = OwnerView::capture(player);
    // `:58196-58198` - the 266's head `num = 60 + 30 * minionPos`
    // (the band half-width), then `:58948-58950` re-pins `num = 10`
    // for the direction band itself - the 60+30 value is dead for the
    // 266 (nothing between the two writes reads it), verbatim skip.
    // `:58951-58959` - the formation slot BEHIND the owner:
    // `num2 = 40 * (minionPos + 1) * direction`.
    let mut flag2 = false;
    let mut flag3 = false;
    let num2 = 40.0 * (p.minion_pos as f32 + 1.0) * owner.dir as f32;
    if owner.center.x < p.pos.x + p.w * 0.5 - 10.0 + num2 {
        flag2 = true;
    } else if owner.center.x > p.pos.x + p.w * 0.5 + 10.0 + num2 {
        flag3 = true;
    }
    // `:59016-59026` - the despawn request forces the direction:
    // `flag12` (ai[0] == -1) walks right (its `num = 30` re-pin is dead
    // - no band re-test follows), `flag13` (ai[0] == -2) stands still.
    if p.ai[0] == -1.0 {
        flag2 = false;
        flag3 = true;
    }
    if p.ai[0] == -2.0 {
        flag2 = false;
        flag3 = false;
    }
    ground_machine(p, world, npcs, &owner, flag2, flag3, flag11);
    // `:17013` - `base.oldPosition = base.position` latches BETWEEN
    // `AI()` (`:16775`) and `HandleMovement` (`:17015`): the machine's
    // `flag32` (`:62318`) read the PREVIOUS tick's latch (last tick's
    // pre-movement position), and this tick's AI writes (the >2000
    // teleport `:60317-60318`, `StepUp`'s lift `:62197`) land before
    // the new latch - exactly the C# order.
    p.old_position = p.pos;
    // `HandleMovement` (`:17469-19865`) + `UpdatePosition`
    // (`:20433-20446`): the dry collision + integrate, and the ONE
    // consumed `Next(3)` on a blocked move.
    ground_movement(p, world, owner.bottom, rng);
    true
}

/// `Update`'s `gfxOffY` decay (`Projectile.cs:16740-16764`) - the
/// per-tick ease of `StepUp`'s draw lift, clamped at +/-16.
fn gfx_decay(p: &mut Projectile) {
    let num = 1.0 + p.vel.x.abs() / 3.0;
    if p.gfx_off_y > 0.0 {
        p.gfx_off_y -= num * p.step_speed;
        if p.gfx_off_y < 0.0 {
            p.gfx_off_y = 0.0;
        }
    } else if p.gfx_off_y < 0.0 {
        p.gfx_off_y += num * p.step_speed;
        if p.gfx_off_y > 0.0 {
            p.gfx_off_y = 0.0;
        }
    }
    p.gfx_off_y = p.gfx_off_y.clamp(-16.0, 16.0);
}

/// The pet's center - recomputed at each C# read site (the machine
/// moves the pet between them: the fly-out teleport, StepUp).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// The ground machine (`Projectile.cs:60254-65015`) with every
/// non-266 arm folded out - the folds are cited at each site. The
/// `flag6`/`flag7`/`flag8`/`flag10` type groups are all false for the
/// 266 (`:58115-58119`), `flag9` (petting) folds false (unported), and
/// `flag` (the Chester interactable pin, `:58182-58186`) is 960-only.
#[allow(clippy::too_many_arguments)]
fn ground_machine(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    owner: &OwnerView,
    mut flag2: bool,
    mut flag3: bool,
    flag11: bool,
) {
    // ===== The fly-out decision (`:60254-60344`) =====
    // `flag22 = ai[1] == 0f` (`:60254`; the Chester `flag` OR is false).
    if p.ai[1] == 0.0 {
        // `:60261-60307` - the return distance: 500 base, +40 per
        // formation slot, +500 while attacking (`localAI[0] > 0`),
        // +100 more for the 266 (`:60299-60302`). The `flag7` +400 is
        // the spiders'.
        let mut num82 = 500.0 + 40.0 * p.minion_pos as f32;
        if p.local_ai[0] > 0.0 {
            num82 += 500.0 + 100.0;
        }
        let c = center(p);
        let num83 = owner.center.x - c.x;
        let num84 = owner.center.y - c.y;
        let num85 = (num83 * num83 + num84 * num84).sqrt();
        let mut flag23 = false;
        if !flag11 {
            if num85 > 2000.0 {
                // `:60315-60319` - the far teleport: the center snaps
                // to the owner, NO dust (unlike the flyers'), no rand.
                p.pos = owner.center - Vec2::new(p.w * 0.5, p.h * 0.5);
            } else if num85 > num82 || (num84.abs() > 300.0 && !(p.local_ai[0] > 0.0)) {
                // `:60320-60334` - the vertical-velocity sign kill on
                // the way out (the 324 exemption is not the 266).
                if num84 > 0.0 && p.vel.y < 0.0 {
                    p.vel.y = 0.0;
                }
                if num84 < 0.0 && p.vel.y > 0.0 {
                    p.vel.y = 0.0;
                }
                flag23 = true;
            }
        }
        if flag23 {
            // `:60336-60343` - the fly-out state (the `flag8` alpha pin
            // is the Pal summons').
            p.ai[0] = 1.0;
        }
    }
    if p.ai[0] != 0.0 && !flag11 {
        // ===== The fly-out arm (`:60387-61548`) =====
        // (The 209/956/1027 materialize arm `:60345-60386` is other
        // types'.)
        let num90 = 0.2f32; // accel
        let num91 = 200.0f32; // the grounded-reset range
        p.tile_collide = false; // `:60405`
        let c = center(p);
        let mut num92 = owner.center.x - c.x;
        // `:60408-60444` - the chase scan: the target point sits 40 px
        // behind the owner; a chaseable NPC within 700 px (Manhattan,
        // owner-relative) with line of sight cancels the fly-out and
        // drops the formation offset.
        num92 -= 40.0 * owner.dir as f32;
        let num93 = 700.0f32;
        let mut flag25 = false;
        let mut num94 = -1i32;
        for (i, n) in npcs.iter().enumerate() {
            if !chaseable(n) {
                continue;
            }
            let (nw, nh) = n.size();
            let nx = n.x + nw * 0.5;
            let ny = n.y + nh * 0.5;
            if (owner.center.x - nx).abs() + (owner.center.y - ny).abs() < num93 {
                if can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
                    num94 = i as i32;
                }
                flag25 = true;
                break;
            }
        }
        if !flag25 {
            num92 -= 40.0 * p.minion_pos as f32 * owner.dir as f32;
        }
        if flag25 && num94 >= 0 {
            p.ai[0] = 0.0;
        }
        let mut num98 = owner.center.y - c.y;
        let mut num99 = (num92 * num92 + num98 * num98).sqrt();
        let num101 = 10.0f32; // speed
        // `:60471-60483` - the long type list does not contain the
        // 266: speed stays 10, accel 0.2 (no owner-velocity boost).
        // `:60488-60495` - the grounded reset back to the idle state.
        if num99 < num91
            && owner.vel.y == 0.0
            && p.pos.y + p.h <= owner.bottom
            && !box_solid(world, p.pos, p.w, p.h)
        {
            p.ai[0] = 0.0;
            if p.vel.y < -6.0 {
                p.vel.y = -6.0;
            }
        }
        // `:60496-60506` - inside 60 px the target IS the velocity
        // (the damping); beyond, normalize to speed 10.
        if num99 < 60.0 {
            num92 = p.vel.x;
            num98 = p.vel.y;
        } else {
            num99 = num101 / num99;
            num92 *= num99;
            num98 *= num99;
        }
        // `:60582-60616` - the generic steering: the same-sign second
        // boost (x1.5) HAS the sign guard here (unlike slice B's).
        if p.vel.x < num92 {
            p.vel.x += num90;
            if p.vel.x < 0.0 {
                p.vel.x += num90 * 1.5;
            }
        }
        if p.vel.x > num92 {
            p.vel.x -= num90;
            if p.vel.x > 0.0 {
                p.vel.x -= num90 * 1.5;
            }
        }
        if p.vel.y < num98 {
            p.vel.y += num90;
            if p.vel.y < 0.0 {
                p.vel.y += num90 * 1.5;
            }
        }
        if p.vel.y > num98 {
            p.vel.y -= num90;
            if p.vel.y > 0.0 {
                p.vel.y -= num90 * 1.5;
            }
        }
        // `:60621-60625` - the `flag6` frame roll is the pygmies'.
        // `:60626-60636` - the sprite faces AGAINST the motion (the
        // 313 exemption is the wall pet's).
        if (p.vel.x as f64) > 0.5 {
            p.sprite_direction = -1;
        } else if (p.vel.x as f64) < -0.5 {
            p.sprite_direction = 1;
        }
        // `:61201-61214` - the 266's flight frames: the 6-tick counter
        // steps frames 8..projFrames-1 (12 frames, `Main.cs:8612`),
        // rotation = velocity.X * 0.1.
        p.frame_counter += 1;
        if p.frame_counter > 5 {
            p.frame += 1;
            p.frame_counter = 0;
        }
        if p.frame < 8 || p.frame >= proj_frames(SLIME_MINION) as i32 {
            p.frame = 8;
        }
        p.rot = p.vel.x * 0.1;
        // `:61458-61548` - the flight dust chain excludes the 266 by
        // type (`:61541`): no dust, no rand.
        return;
    }
    // ===== The ground arm (`:61550-65015`) =====
    // (The `flag10` pygmy/Pal attack block `:61552-61751` is folded
    // out - false for the 266.)
    // `:61755-62004` - the 266's target scan.
    let num151 = 40.0 * p.minion_pos as f32;
    let num152 = 60.0f32; // the attack cooldown
    p.local_ai[0] -= 1.0;
    if p.local_ai[0] < 0.0 {
        p.local_ai[0] = 0.0;
    }
    if p.ai[1] > 0.0 {
        p.ai[1] -= 1.0;
    } else {
        // `:61770-61826` - the closest chaseable: the whip mark first
        // (no velocity lead), then the scan (the fallback keeps the
        // nearest-by-Manhattan point even without line of sight).
        let c = center(p);
        let mut num153 = p.pos.x;
        let mut num154 = p.pos.y;
        let mut num155 = 100000.0f32;
        let mut num156 = num155;
        let mut num157 = -1i32;
        if owner.attack_target >= 0 {
            if let Some(n) = npcs.get(owner.attack_target as usize) {
                if chaseable(n) {
                    let (nw, nh) = n.size();
                    let x = n.x + nw * 0.5;
                    let y = n.y + nh * 0.5;
                    let num158 = (c.x - x).abs() + (c.y - y).abs();
                    if num158 < num155 {
                        if num157 == -1 && num158 <= num156 {
                            num156 = num158;
                            num153 = x;
                            num154 = y;
                        }
                        if can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
                            num155 = num158;
                            num153 = x;
                            num154 = y;
                            num157 = owner.attack_target;
                        }
                    }
                }
            }
        }
        if num157 == -1 {
            for (i, n) in npcs.iter().enumerate() {
                if !chaseable(n) {
                    continue;
                }
                let (nw, nh) = n.size();
                let num160 = n.x + nw * 0.5;
                let num161 = n.y + nh * 0.5;
                let num162 = (c.x - num160).abs() + (c.y - num161).abs();
                if num162 < num155 {
                    if num157 == -1 && num162 <= num156 {
                        num156 = num162;
                        num153 = num160;
                        num154 = num161;
                    }
                    if can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
                        num155 = num162;
                        num153 = num160;
                        num154 = num161;
                        num157 = i as i32;
                    }
                }
            }
        }
        // `:61827-61830` - the spiders' tileCollide re-arm is flag7.
        if num157 == -1 && num156 < num155 {
            num155 = num156;
        }
        // `:61835-61913` - the flag28/flag29 spider contact block is
        // flag7-gated inside; for the 266 both stay false.
        // `:61914-61926` - the idle walk range: 300 above the surface,
        // 150 below (`(double)position.Y > worldSurface * 16.0`).
        let mut num168 = 300.0f32;
        if (p.pos.y as f64) > world.world_surface * 16.0 {
            num168 = 150.0;
        }
        // `:61927-61940` - no target: face the nearest point within
        // range (+/-5 px dead band).
        if num155 < num168 + num151 && num157 == -1 {
            let num169 = num153 - (p.pos.x + p.w * 0.5);
            if num169 < -5.0 {
                flag2 = true;
                flag3 = false;
            } else if num169 > 5.0 {
                flag3 = true;
                flag2 = false;
            }
        }
        // `:61941-61946` - the spiders' post-bite recoil is flag7.
        if num157 >= 0 && num155 < 800.0 + num151 {
            // `:61947-61998` - the attack: friendly on, the 60-tick
            // cooldown, face the target (+/-10 px dead band).
            p.friendly = true;
            p.local_ai[0] = num152;
            let num170 = num153 - (p.pos.x + p.w * 0.5);
            if num170 < -10.0 {
                flag2 = true;
                flag3 = false;
            } else if num170 > 10.0 {
                flag3 = true;
                flag2 = false;
            }
            // `:61962-61985` - the height ladder: a target 100+ px
            // above within +/-50 px X and grounded -> the jump impulse
            // grows with the height difference.
            let c = center(p);
            if num154 < c.y - 100.0 && num170 > -50.0 && num170 < 50.0 && p.vel.y == 0.0 {
                let num171 = (num154 - c.y).abs();
                if num171 < 120.0 {
                    p.vel.y = -10.0;
                } else if num171 < 210.0 {
                    p.vel.y = -13.0;
                } else if num171 < 270.0 {
                    p.vel.y = -15.0;
                } else if num171 < 310.0 {
                    p.vel.y = -17.0;
                } else if num171 < 380.0 {
                    p.vel.y = -18.0;
                }
            }
            // `:61986-61997` - the flag30 recoil is the spiders'.
        } else {
            p.friendly = false;
        }
    }
    // `:62005-62013` - the attack-cooldown direction clear (flag9
    // folds false); the flag10 direction pin is the pygmies'/Pals'.
    if p.ai[1] != 0.0 {
        flag2 = false;
        flag3 = false;
    }
    // `:62014-62022` - flag31 (the Pal shot-stall) folds true.
    // `:62023-62041` - the 127 rotation ease is the turtle's; the 266
    // takes the `type != 313 && !flag29` zero (`:62038-62041`).
    p.rot = 0.0;
    // `:62042-62045` - `!flag7` re-arms tileCollide every ground tick.
    p.tile_collide = true;
    // `:62046-62118` - the walk parameters: the 266 rides the big
    // minion group (`:62083-62100`): speed 6 / accel 0.2, boosted to
    // the owner's |vx|+|vy| with accel 0.3 when the owner is faster;
    // the flag10 x2 and 1027 x1.5 are other types'. flag11 pins
    // speed 6 (`:62115-62118`).
    let mut num172 = 0.2f32;
    let mut num173 = 6.0f32;
    let owner_speed = owner.vel.x.abs() + owner.vel.y.abs();
    if num173 < owner_speed {
        num173 = owner_speed;
        num172 = 0.3;
    }
    if flag11 {
        num173 = 6.0;
    }
    // `:62119-62148` - the walk: the +/-3.5 band gets the full accel,
    // beyond it a quarter; no direction -> the 0.9 damping with the
    // accel-wide zero snap.
    if flag2 {
        if (p.vel.x as f64) > -3.5 {
            p.vel.x -= num172;
        } else {
            p.vel.x -= num172 * 0.25;
        }
    } else if flag3 {
        if (p.vel.x as f64) < 3.5 {
            p.vel.x += num172;
        } else {
            p.vel.x += num172 * 0.25;
        }
    } else {
        p.vel.x *= 0.9;
        if p.vel.x >= 0.0 - num172 && p.vel.x <= num172 {
            p.vel.x = 0.0;
        }
    }
    // `:62149-62158` - the 208 parrot damping is another type's.
    // `:62159-62180` - the wall probe ahead (flag5): the tile beside
    // the center, shifted by the direction and the velocity, solid
    // and not a closed door -> the hop intent.
    let mut flag5 = false;
    if flag2 | flag3 {
        let mut num174 = (p.pos.x + p.w * 0.5) as i32 / 16;
        let num175 = (p.pos.y + p.h * 0.5) as i32 / 16;
        if flag2 {
            num174 -= 1;
        }
        if flag3 {
            num174 += 1;
        }
        num174 += p.vel.x as i32;
        let in_world =
            num174 >= 0 && num174 < world.width && num175 >= 0 && num175 < world.height;
        if in_world
            && solid_tile(world, num174, num175)
            && world
                .tile(num174, num175)
                .is_some_and(|t| !tr_core::tiledata::SET_CLOSED_DOORS.has(t.kind))
        {
            flag5 = true;
        }
    }
    // `:62181-62184` - flag4: the owner's feet 8+ px BELOW the pet's.
    let flag4 = owner.bottom - 8.0 > p.pos.y + p.h;
    // `:62185-62196` - the 268/860/881/934 flag5 overrides are other
    // types'.
    // `:62197` - `Collision.StepUp` (the port's gravDir-1 variant;
    // `holds_matching` false - the 266 holds nothing).
    let mut pos = [p.pos.x, p.pos.y];
    let mut vel = [p.vel.x, p.vel.y];
    crate::collision::step_up(
        world,
        &mut pos,
        &mut vel,
        p.w,
        p.h,
        &mut p.step_speed,
        &mut p.gfx_off_y,
        false,
    );
    p.pos = Vec2::new(pos[0], pos[1]);
    p.vel = Vec2::new(vel[0], vel[1]);
    if p.vel.y == 0.0 {
        // `:62198-62213` - the !flag4 ground probe's `SolidTile` call
        // discards its result (a decompiled no-op) - skipped; `flag4`
        // itself feeds nothing else on the 266's path.
        let _ = flag4;
        if flag5 {
            // `:62214-62270` - the ledge hop: the tile under the feet
            // solid (bottom slopes allowed), half-brick or sloped ->
            // the wall-height ladder picks the jump impulse.
            let num177 = (p.pos.x + p.w * 0.5) as i32 / 16;
            let num178 = (p.pos.y + p.h) as i32 / 16;
            let stands = solid_tile_allow_bottom_slope(world, num177, num178)
                || world
                    .tile(num177, num178)
                    .is_some_and(|t| t.half_brick() || t.slope() > 0);
            if stands {
                // `:62226-62263` - the try/catch ladder (the 067_b
                // precedent: null probe tiles throw into the catch's
                // -9.1; `SolidTile` reads null as solid).
                let mut num177 = (p.pos.x + p.w * 0.5) as i32 / 16;
                let num178 = (p.pos.y + p.h * 0.5) as i32 / 16;
                if flag2 {
                    num177 -= 1;
                }
                if flag3 {
                    num177 += 1;
                }
                num177 += p.vel.x as i32;
                if world.tile(num177, num178 - 1).is_none()
                    || world.tile(num177, num178 - 2).is_none()
                {
                    p.vel.y = -9.1;
                } else if !solid_tile(world, num177, num178 - 1)
                    && !solid_tile(world, num177, num178 - 2)
                {
                    p.vel.y = -5.1;
                } else if !solid_tile(world, num177, num178 - 2) {
                    p.vel.y = -7.1;
                } else if solid_tile(world, num177, num178 - 5) {
                    p.vel.y = -11.1;
                } else if solid_tile(world, num177, num178 - 4) {
                    p.vel.y = -10.1;
                } else {
                    p.vel.y = -9.1;
                }
                // `:62265-62268` - the 127 fly-out pin is the turtle's.
            }
        } else if flag2 | flag3 {
            // `:62271-62274` - THE 266 HOP: walking on the ground with
            // no wall ahead -> velocity.Y -= 6 (the slime's bounce).
            p.vel.y -= 6.0;
        }
    }
    // `:62276-62283` - the speed clamp.
    if p.vel.x > num173 {
        p.vel.x = num173;
    }
    if p.vel.x < 0.0 - num173 {
        p.vel.x = 0.0 - num173;
    }
    // `:62284-62302` - flag31 folds true: the direction follows the
    // motion, then the accel-sign pins.
    if p.vel.x < 0.0 {
        p.direction = -1;
    }
    if p.vel.x > 0.0 {
        p.direction = 1;
    }
    if (p.vel.x > num172) & flag3 {
        p.direction = 1;
    }
    if (p.vel.x < 0.0 - num172) & flag2 {
        p.direction = -1;
    }
    // `:62303-62313` - the sprite faces AGAINST the direction (the
    // 313 exemption is the wall pet's; the `:62314-62317` with-facing
    // list does not contain the 266).
    if p.direction == -1 {
        p.sprite_direction = 1;
    }
    if p.direction == 1 {
        p.sprite_direction = -1;
    }
    // `:62318` - flag32: the X stall against the PREVIOUS tick's
    // pre-movement latch (`oldPosition` updates after the AI, `:17013`
    // - the tick-head order note in `ai026_c_tick`).
    let flag32 = p.pos.x - p.old_position.x == 0.0;
    // `:62319-63803` - the per-type frame chains: the 266 matches none
    // of the early type arms, flag6/flag8 are false, the 268/269/313
    // arms are other types', flag7 is false - it lands on its own arm
    // at `:64435`.
    // `:64435-64494` - the 266's ground frames + gravity.
    if p.vel.y != 0.0 {
        // `:64437-64449` - airborne: the 5-tick counter steps frames
        // 1-7 (0 and 8+ wrap to 1).
        p.frame_counter += 1;
        if p.frame_counter > 4 {
            p.frame_counter = 0;
            p.frame += 1;
        }
        if p.frame == 0 || p.frame >= 8 {
            p.frame = 1;
        }
    } else {
        // `:64450-64474` - grounded: the idle sprite faces the owner
        // when stalled; the 8-tick counter steps frames 5/6/7 (the
        // airborne rows 1-4 and 8+ fold to 0, the 1 -> 5 jump).
        if flag32 {
            p.sprite_direction = -1;
            if owner.center.x < p.pos.x + p.w * 0.5 {
                p.sprite_direction = 1;
            }
        }
        p.frame_counter += 1;
        if p.frame_counter > 7 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame == 1 {
                p.frame = 5;
            }
        }
        if (p.frame >= 1 && p.frame <= 4) || p.frame >= 8 {
            p.frame = 0;
        }
    }
    // `:64475-64489` - the wet swim folds to the dry gravity (the slot
    // carries no wet flags, module doc).
    p.vel.y += 0.4;
    // `:64490-64493` - the fall clamp.
    if p.vel.y > 10.0 {
        p.vel.y = 10.0;
    }
    // `:65016-65022` - the 891 light plot is the golem pet's.
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70189`) -
/// the FULL form: out-of-world / null reads solid, active +
/// (solid || solidTop), no top slope UNLESS a platform with a proper
/// top frame, no half-brick, not actuated. (The `tr-core` edit-side
/// variant drops the platform clause and the inactive test - its
/// callers never place sloped platforms; the hop probe here keeps the
/// C# conjunction verbatim.)
///
/// S1-P-i27: `pub(crate)` — the Forbidden set bonus's rest-spot aim
/// (`Player.MinionRestTargetAim`, `Player.cs:56976`/`:56986`) shares it.
pub(crate) fn solid_tile_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return true;
    }
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (tr_core::tiledata::TILE_SOLID.has(t.kind)
            || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind))
        && (!t.top_slope()
            || (tr_core::tiledata::SET_PLATFORMS.has(t.kind)
                && platform_proper_top_frame(t.frame_x)))
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.PlatformProperTopFrame` (`WorldGen.cs:70153-70165`,
/// `PlatformFrameWidth() = 18`, `TileObjectData.cs:5121-5124`) - the
/// `tr-core` torch-side copy is `pub(crate)`; this local mirror keeps
/// the frame bands verbatim.
fn platform_proper_top_frame(frame_x: i16) -> bool {
    let num = frame_x as i32 / 18;
    if (num < 0 || num > 7) && (num < 12 || num > 16) {
        return num >= 25 && num <= 26;
    }
    true
}

/// `HandleMovement`'s half for the 266 (`Projectile.cs:17469-19865`):
/// the `Main.projPet[266]` fall-through decision (`:17479-17487` -
/// false unless the owner's feet are 12+ px below the pet's), the dry
/// collision branch - the >16 px/tick hoik step loop
/// (`:17711-17846`, the `pirate_movement` port) else the single
/// `TileCollision` (`:17849`) - then the blocked-move chain: the
/// CONSUMED `Next(3)` (`:17948`, module doc) and NO kill - the default
/// `position += velocity; Kill()` (`:19851-19852`) sits inside the
/// `!correctSlopeCollision` gate (`:18398`), which the 266's
/// `Main.projPet` true closes: a blocked move only stops it (the
/// `pirate_movement` precedent). Then `UpdatePosition`'s dry integrate
/// (`:20445`). SKIPPED (ANALYSIS): the inner `SlopeCollision` passes
/// (`correctSlopeCollision` true), the `UpdatePosition` slope pass
/// (`:20447-20474`), the `SwitchTiles` plate presses
/// (`:17788-17791`/`:19859-19864`, no plate layer), the tile-cut bodies
/// (`:17910-17985`, no cutting layer - the ROLL stays) and the
/// `ignoreDoors` parameter (the port's `tile_collision` carries the
/// fall pair only).
fn ground_movement(p: &mut Projectile, world: &World, owner_bottom: f32, rng: &mut dyn DropRng) {
    if !p.tile_collide {
        // `:17476`'s gate - no tile pass, just the integrate (the
        // fly-out arm's `tileCollide = false` tick).
        p.pos += p.vel;
        return;
    }
    // `:17479-17487` - `flag6`: projPet starts false, the owner's feet
    // 12+ px below the pet's bottom re-arms it. (No later override
    // matches the 266; `decidesManualFallThrough` is false, `:708`.)
    let fall = owner_bottom - 12.0 > p.pos.y + p.h;
    let last_velocity = p.vel;
    // `:17702-17710` - the step size clamp.
    let num2 = (p.w.min(p.h) as i32).clamp(3, 16) as f32;
    if p.vel.length() > num2 {
        // `:17711-17846` - the hoik step loop.
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            fall,
            fall,
        );
        let mut num3 = p.vel.length();
        let num4 = num2;
        let mut vector8 = safe_normalize(p.vel);
        if tc.vel[1] == 0.0 {
            vector8.y = 0.0;
        }
        let mut zero = Vec2::ZERO;
        let mut num5 = 0;
        while num3 > 0.0 {
            num5 += 1;
            if num5 > 300 {
                break;
            }
            let num6 = num3.min(num4);
            num3 -= num6;
            let step_vel = vector8 * num6;
            let tc = crate::collision::tile_collision(
                world,
                [p.pos.x, p.pos.y],
                [step_vel.x, step_vel.y],
                p.w,
                p.h,
                fall,
                fall,
            );
            let vector10 = Vec2::new(tc.vel[0], tc.vel[1]);
            p.pos += vector10;
            p.vel = vector10;
            zero += p.vel;
        }
        p.vel = zero;
        // `:17795-17802` - the last-velocity snap.
        if (p.vel.x - last_velocity.x).abs() < 0.0001 {
            p.vel.x = last_velocity.x;
        }
        if (p.vel.y - last_velocity.y).abs() < 0.0001 {
            p.vel.y = last_velocity.y;
        }
    } else {
        // `:17849` - the single collision pass.
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            fall,
            fall,
        );
        p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
    }
    // `:17896-17899` - a changed velocity is a blocked move.
    if p.vel != last_velocity {
        // `:17948` - `Main.rand.Next(3) == 0 && ...`: the draw is the
        // FIRST operand - consumed unconditionally on a blocked move
        // (the 266 passes `CanCutTiles()`, `:15323-15334`). The cut
        // body skips with the unported layer (module doc).
        let _ = rng.next_range(0, 3);
        // `:17986-18398` - the blocked-move type chain: the 266
        // matches no arm, and the default `position += velocity;
        // Kill()` (`:19851-19852`) sits INSIDE the
        // `!correctSlopeCollision` gate (`:18398`) - the 266's
        // `Main.projPet` true (`Main.cs:8739` -> `:707`) closes it, so
        // a blocked move only STOPS the minion (the `pirate_movement`
        // precedent). No kill, no extra step.
    }
    // `UpdatePosition` (`:20445`) - the dry integrate (the wet branch
    // folds dry, module doc; the slope pass is deferred).
    p.pos += p.vel;
}

/// The follow-up slices' map of the remaining block - every per-type
/// tail the 266 does not reach, with its citation. Pure documentation
/// (the census above lists the types); kept as a doc fn so the map
/// travels with the code it describes.
///
/// * fly-out per-type frame/rotation arms (`:60617-61457`): the 111
///   frame-7 pin, the Pal `flag8` mounted-center snap (`:60637-60651`),
///   the 112/900/899/897/891/890/884 spin-frame chains, the 881/934
///   gore-drop frames, the 875/825/854/858/859/860/816/821 chains, the
///   958/960/959/1095/1096/994/998/1003/1004 clamped-rotation pets,
///   the flag7 wall-crawl frames (`:61100-61136`), the 334/353/127/
///   269/266/324/268/200/208/236/499/765/314/319/210/313/398 chains;
/// * fly-out dusts (`:61458-61548`) - every draw there is another
///   type's (the 266 is excluded at `:61541`);
/// * the pygmy/Pal attack block (`:61552-61751`): the spear spawn's
///   `Next(-20,21)`, `Next(0,100)`, `Next(-num144,num144+1)` draws
///   (`:61698-61705`), the 1097 bolt + the 1113 triple-shot ladder;
/// * the spider contact block (`:61839-61912`): the platform
///   fall-through, the 14-px clamp, the sibling separation loop
///   (`:61884-61910` - note the C#'s `velocity.Y +=` typo in the X
///   else-branch at `:61907`, verbatim when ported);
/// * the ground per-type frame chains (`:62319-65014`): the 956/958/
///   959/998/1003/1004/1027/1095/1096/994/960 walk/idle machines (the
///   958 idle `Next(100)` re-roll `:62389`, the 1027 materialize
///   `NextFloat` + 25-dust ring `:62559-62566`), the 816/860/859/858/
///   900/899/897/891/890/884/881/934/875/854/825/821 chains (the 860
///   gore burst's `Next(3)`/`Next(1,10)`/`Next(4)` draws `:62889-62892`,
///   the 825 idle `Next(50)`+`Next(2)` `:63500-63502`), the flag6 pygmy
///   frames, the flag8 Pal frames (the 36-41 shot cycle + the 45-52
///   petting cycle), the 268/269/313/flag7/314/319/236/499/765/334/
///   353/111/112/127/200/208/209/324/210/398 chains;
/// * the 891 light plot (`:65016-65022`).
#[allow(dead_code)]
fn ground_machine_census() {}

#[cfg(test)]
mod tests {
    //! Witness tests-as-code (S1-D-4b-16, code-only mode - written, NOT
    //! run this round). Each pins a C# behaviour with its citation; the
    //! RNG-order witness records the draw sequence on a counting
    //! stream.

    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::{BABY_EATER, ProjectilePool, SKELETRON_PET, proj_data};
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::tile_light::FastRandom;

    /// A recording `DropRng` - the draw log pins the C#'s `Main.rand`
    /// order (the golden rule's RNG witness).
    #[derive(Default)]
    struct RecordRng {
        draws: Vec<String>,
    }
    impl DropRng for RecordRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push(format!("range({min},{max})"));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float".into());
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    /// A flat floor at tile row 40 (y = 640) in a 64x100 world - the
    /// ground fixtures stand on it; the default `world_surface`
    /// (height * 0.25 = 25) puts everything below the surface (the
    /// 150-px idle range). The 100-row height keeps the fixtures above
    /// `can_hit`'s `maxTilesY - 40` underworld clamp (row 60) - a
    /// 64-row world would clamp the line-walk anchors into the air.
    fn ground_world() -> World {
        let mut w = World::new("t".into(), 64, 100);
        for x in 0..64 {
            for y in 40..100 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn owner(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 500.0;
        p.y = 500.0;
        p.vx = 0.0;
        p.vy = 0.0;
        p.facing = 1;
        p
    }

    fn slime(x: f32, y: f32) -> Projectile {
        let data = proj_data(SLIME_MINION).expect("the 266 row");
        // Field-by-field on `Default` - the struct has private
        // bookkeeping fields (E0451 outside projectile.rs).
        let mut p = Projectile::default();
        p.active = true;
        p.kind = SLIME_MINION;
        p.owner = 0;
        p.pos = Vec2::new(x, y);
        p.w = data.w;
        p.h = data.h;
        p.friendly = data.minion; // the summon spawn's friendly
        p.time_left = data.time_left;
        p.penetrate = data.penetrate;
        p.max_penetrate = data.penetrate;
        p.tile_collide = data.tile_collide;
        p.minion = data.minion;
        p.minion_slots = data.minion_slots;
        p
    }

    /// A chaseable blue slime (`Npc::new` sets the row's box; the hp
    /// pins clear `chaseable`'s `lifeMax > 5` gate).
    fn target_npc(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Slime, x, y, 1, 0.0);
        n.hp = 50;
        n.max_hp = 50;
        n
    }

    /// The 266 row carries the C# `SetDefaults` slice
    /// (`Projectile.cs:3516-3528`): 24x16, aiStyle 26, penetrate -1,
    /// timeLeft 18000, minion + 1 slot, ID-static immunity 12,
    /// netImportant; the sheet is 12 frames (`Main.cs:8612`).
    #[test]
    fn the_row_carries_the_csharp_set_defaults() {
        let d = proj_data(SLIME_MINION).expect("row");
        assert_eq!((d.w, d.h), (24.0, 16.0));
        assert_eq!(d.ai_style, 26);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.time_left, 18000, "timeLeft *= 5");
        assert!(d.minion && !d.sentry);
        assert_eq!(d.minion_slots, 1.0);
        assert!(d.uses_id_static_immunity);
        assert_eq!(d.id_static_hit_cooldown, 12);
        assert!(d.net_important);
        assert_eq!(proj_frames(SLIME_MINION), 12);
    }

    /// The gate routes ONLY the 266 - slice A's 175, slice B's 885 and
    /// a row-less ground pet (111) return false and nothing mutates,
    /// RNG untouched.
    #[test]
    fn the_gate_leaves_non_slice_kinds_byte_identical() {
        let w = world();
        let mut p = owner(&w);
        for kind in [BABY_EATER, SKELETRON_PET, 111u16] {
            let mut slot = slime(480.0, 480.0);
            slot.kind = kind;
            let before = slot.clone();
            let mut rng = RecordRng::default();
            assert!(!ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
            assert_eq!(slot.pos, before.pos);
            assert_eq!(slot.vel, before.vel);
            assert_eq!(slot.time_left, before.time_left);
            assert_eq!(slot.frame, before.frame);
            assert!(rng.draws.is_empty(), "no rand for a non-slice kind");
        }
    }

    /// `:58104-58108` - an inactive owner kills the minion.
    #[test]
    fn an_absent_owner_kills_the_minion() {
        let w = world();
        let mut slot = slime(480.0, 480.0);
        let mut rng = RecordRng::default();
        assert!(ai026_c_tick(&mut slot, &w, &[], None, &mut rng));
        assert!(!slot.active);
        assert!(rng.draws.is_empty());
    }

    /// `:58449-58459` - a standing `player.slime` pins timeLeft = 2;
    /// death clears the flag FIRST, so the death tick does not pin.
    #[test]
    fn the_keep_alive_pins_and_the_death_clears_first() {
        let w = world();
        let mut p = owner(&w);
        // At the formation slot (40 px BEHIND the owner - `num2 = 40 *
        // (minionPos + 1) * direction` puts the slot at `center - num2`,
        // `:58951-58959`), same center height: the band is inside its
        // +/-10 dead zone, so the idle machine neither walks nor hops.
        let (cx, cy) = p.center();
        let mut slot = slime(cx - 40.0 - 12.0, cy - 8.0);
        p.slime = true;
        let mut rng = RecordRng::default();
        assert!(ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
        assert_eq!(slot.time_left, 2, "the standing flag pins");
        slot.time_left = 18000;
        p.dead = true;
        assert!(ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
        assert!(!p.slime, "death clears");
        assert_eq!(slot.time_left, 18000, "the cleared flag does not pin");
    }

    /// `:58944-58947` + `:59016-59026` - `ai[0] == -1` pins timeLeft
    /// and forces the walk right; `-2` pins and stands still. The
    /// fly-out decision is skipped (flag11), the ground arm runs.
    #[test]
    fn the_despawn_request_pins_and_forces_the_direction() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        for (state, expect_walk) in [(-1.0f32, true), (-2.0, false)] {
            let mut slot = slime(cx - 40.0 - 12.0, cy - 8.0);
            slot.ai[0] = state;
            let mut rng = RecordRng::default();
            assert!(ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
            assert_eq!(slot.time_left, 2, "ai[0] = {state}");
            if expect_walk {
                assert!(slot.vel.x > 0.0, "flag12 walks right");
            } else {
                assert_eq!(slot.vel.x, 0.0, "flag13 stands still");
            }
            // The empty test world has no floor: the stand tick's
            // +0.4 gravity is unblocked, so no :17948 draw fires here
            // (the grounded stand tick's draw is pinned by the
            // blocked-move witness on the floor world).
            assert!(rng.draws.is_empty(), "the airborne ground arm draws no rand");
        }
    }

    /// `:62271-62274` - THE HOP: grounded (vel.y == 0 after the
    /// collision), walking, no wall ahead -> `velocity.Y -= 6`, then
    /// the arm's `+0.4` gravity -> -5.6 at the integrate. The fixture
    /// stands on the floor 100 px left of the owner (inside the
    /// 150-px below-surface idle range, beyond the +/-10 band).
    #[test]
    fn the_walking_slime_hops() {
        let w = ground_world();
        let mut p = owner(&w);
        p.x = 500.0;
        p.y = 640.0 - p.height; // the owner stands on the floor too
        let mut slot = slime(400.0, 640.0 - 16.0);
        slot.vel = Vec2::ZERO;
        let mut rng = RecordRng::default();
        assert!(ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
        // The hop impulse -6 + the 0.4 gravity = -5.6; the collision
        // pass keeps it (moving up, away from the floor).
        assert!(
            (slot.vel.y - (-5.6)).abs() < 1e-4,
            "hop then gravity: {}",
            slot.vel.y
        );
        assert!(slot.vel.x > 0.0, "the walk steers toward the owner");
        assert!(rng.draws.is_empty(), "the hop draws no rand");
    }

    /// `:61947-62002` - a chaseable NPC in line of sight within
    /// 800 + 40*minionPos px flips `friendly` true and pins the
    /// 60-tick attack cooldown; without one it stays false.
    #[test]
    fn the_attack_window_flips_friendly_and_pins_the_cooldown() {
        let w = ground_world();
        let mut p = owner(&w);
        p.x = 500.0;
        p.y = 640.0 - p.height;
        let mut slot = slime(400.0, 640.0 - 16.0);
        let npc = target_npc(450.0, 640.0 - 24.0);
        let mut rng = RecordRng::default();
        assert!(ai026_c_tick(&mut slot, &w, &[npc], Some(&mut p), &mut rng));
        assert!(slot.friendly, "the attack window is friendly");
        assert_eq!(slot.local_ai[0], 60.0, "the attack cooldown");
        // No NPC -> the else arm clears friendly (`:61999-62002`).
        let mut idle = slime(400.0, 640.0 - 16.0);
        idle.friendly = true;
        assert!(ai026_c_tick(&mut idle, &w, &[], Some(&mut p), &mut rng));
        assert!(!idle.friendly, "no target -> friendly false");
        assert!(rng.draws.is_empty(), "the scan draws no rand");
    }

    /// `:60315-60319` - beyond 2000 px the fly-out teleports the
    /// center onto the owner with NO dust and NO rand (unlike the
    /// flyers' 24-draw teleports). The fixture is airborne (ai[1] == 0
    /// opens the decision) far right of the owner, high enough that
    /// the out-of-world guard (`:16684`) does not fire first.
    #[test]
    fn the_far_teleport_snaps_without_draws() {
        let w = world();
        let mut p = owner(&w);
        let (cx, cy) = p.center();
        let mut slot = slime(cx + 2500.0 - 12.0, cy - 8.0);
        let mut rng = RecordRng::default();
        assert!(ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
        // The teleport snaps the center; the decision does NOT set
        // ai[0] (flag23 stays false), so the GROUND arm runs on the
        // snapped position: the band walks it left (the formation slot
        // is behind the owner) and the 266 hop fires (-6 + 0.4 gravity
        // = -5.6) - the C# does the same on the teleport tick.
        assert!(
            (slot.pos.x + slot.w * 0.5 - cx).abs() < 2.0,
            "the center snapped"
        );
        assert!(
            (slot.pos.y + slot.h * 0.5 - cy).abs() < 6.5,
            "the center snapped, then the hop lifted it"
        );
        assert!(rng.draws.is_empty(), "the ground teleport has no dust");
    }

    /// `:17948` - a blocked move consumes exactly ONE `Next(3)` (the
    /// first `&&` operand - even with damage 0). The 266's
    /// `correctSlopeCollision` true closes the default-kill gate
    /// (`:18398`), so the minion SURVIVES the stop (the
    /// `pirate_movement` precedent). The fixture stands on the floor
    /// right of the owner with a wall column flush against its left
    /// edge, walking left into it.
    #[test]
    fn the_blocked_move_consumes_the_tile_cut_roll() {
        let mut w = ground_world();
        // A wall at tile x = 24 (px 384-400), rows 36-39 (above the
        // floor, beside the slime's box).
        for y in 36..40 {
            let i = w.index(24, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = owner(&w);
        // The owner stands LEFT of the wall so the band walks the
        // slime INTO it (the slime at px 400 is flush with the wall's
        // right edge).
        p.x = 300.0;
        p.y = 640.0 - p.height;
        p.facing = -1;
        let mut slot = slime(400.0, 640.0 - 16.0);
        slot.vel = Vec2::new(-3.0, 0.0);
        let mut rng = RecordRng::default();
        assert!(ai026_c_tick(&mut slot, &w, &[], Some(&mut p), &mut rng));
        assert_eq!(
            rng.draws,
            vec!["range(0,3)".to_string()],
            "exactly the :17948 Next(3)"
        );
        assert!(slot.active, "the projPet gate keeps the minion alive");
        assert_eq!(slot.vel.x, 0.0, "the wall stopped the walk");
    }

    /// The pool seam: the `26 =>` chain routes the 266 through
    /// `ai026_c_tick` after A/B's false returns - the slime now MOVES
    /// (the pre-slice-C fall-through froze it; slices A/B's own
    /// witnesses keep their non-266 fixtures).
    #[test]
    fn the_pool_dispatch_routes_the_slime_to_slice_c() {
        let mut w = ground_world();
        let mut p = owner(&w);
        p.x = 500.0;
        p.y = 640.0 - p.height;
        let (cx, cy) = p.center();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(SLIME_MINION, 0, cx - 100.0, cy, 0.0, 0.0, 10, 0.0, 0.0, 0.0)
            .expect("slime slot");
        let before = pool.slots()[slot].pos;
        pool.update(
            &mut w,
            &mut [],
            Some((cx, cy)),
            None,
            Some(&mut p),
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut FastRandom::new(7),
            0,
            true,
        );
        assert_ne!(pool.slots()[slot].pos, before, "the slime moves now");
        assert!(pool.slots()[slot].vel.x > 0.0, "it walks toward the owner");
    }
}
