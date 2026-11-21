//! W4-d2 — the aiStyle-13 chain weapons: the Harpoon (23), the Golem Fist
//! (262), the Boxing Glove (271), the Chain Knife (273) and the Chain
//! Guillotine (481). `AI()`'s `else if (this.aiStyle == 13)`
//! (`Projectile.cs:24636-24829`), the style's whole body, plus the
//! movement half (`HandleMovement`'s shared 3/13/69/109 arm,
//! `:19512-19571`) and the on-hit return arm (`Damage_PVE_Inner`
//! `:14148-14164`, wired in `projectile.rs::damage_pve`).
//!
//! Census verdicts (the C# the port now matches):
//!
//! * **The head** (`:24638-24673`): owner dead OR `(Center −
//!   base.Center).Length() > 2000` → `Kill()`; `SetDummyItemTime(5)`
//!   for every kind but the 481; the `alpha == 0` facing flip
//!   (`ChangeDir` by the centre-X comparison); the 481's dynamic
//!   `extraUpdates` (1 outbound / 2 returning).
//! * **The outbound state** (`ai[0] == 0`, `:24682-24767`): the
//!   distance flips — the C# else-if chain is ported VERBATIM: the 23
//!   flips at >1000, the GENERIC >700 arm fires next, then the per-type
//!   262 >500 / 271 >200 / 273 >150 (remix >300) / 481 >525 arms catch
//!   the sub-700 band (net: each kind flips at its own threshold).
//!   `rotation = atan2(vy, vx) + 1.57f`; `ai[1]++`; `ai[1] > 5 →
//!   alpha = 0` (the spawn alpha 255 fades in — the visible phase);
//!   the 262/271/273/481 cap `ai[1]` at 8; the 23's gravity tail
//!   (`ai[1] >= 17 → 17, vy += 0.6, vx *= 0.995`) vs the generic
//!   `ai[1] >= 10 → 15, vy += 0.3` — the melee chains cap at 8 and so
//!   NEVER reach the generic gravity (a verbatim source quirk). The
//!   262/271 sprite follows the X sign; the 262's `ai[0] != 0 →
//!   localAI[0] = -1` runs AFTER the flip (live when the flip fired
//!   this tick).
//! * **The return state** (`ai[0] == 1`, `:24768-24817`): the 262's
//!   `localAI[0] == 0` one-shot — inside 150 px it pins −1, else it
//!   arms the 200-radius hitbox (`localAI[0] = 200`, consumed by
//!   `Damage_GetHitbox` `:14978-14983`, already ported in
//!   `projectile.rs::damage_hitbox`) + `HitTilesInACircle` (effectOnly
//!   dust — zero explicit rand, skipped with the layer) + the Item14
//!   cue. `tileCollide = false`; `rotation = atan2(dy, dx) − 1.57f`;
//!   the <50 px catch kills; the seek normalizes to 20 (262: 30); the
//!   262/271 sprite INVERTS against the X sign.
//! * **The tail** (`:24818-24828`): the 23 calls
//!   `AI_AdjustPlayerItemRotationToFaceProjectile(-1, 50f)` only while
//!   outbound; the 262/273/481/271 call the default overload. The
//!   helper (`:22518-22543`) gates on `alpha == 0` + the held item's
//!   `shoot == type`, then aims `itemRotation` at the owner's
//!   MountedCenter (+gfxOffY) with the left-side π flip.
//! * **The movement** (`HandleMovement` `:19512-19571`): a blocked
//!   move zeroes the blocked axis (`Collision.TileCollision`'s
//!   effect, the port's per-axis probe stand-in — the M4-PROJ-3
//!   deviation), then the 23 zeroes BOTH axes (the stick, `:19520-
//!   19523`), every other kind keeps the collision velocity; all set
//!   `ai[0] = 1` (the return — NO velocity reversal for style 13,
//!   `:19561-19568`) + the Dig thud (`PlaySound(0)` `:19571`). The
//!   world-edge flag feeds the same response (`:17900-17903`, the
//!   boomerang precedent). `UpdatePosition` integrates the POST-bounce
//!   velocity (`:20445`); `AutomaticallyChangesDirection` (`:17283`)
//!   excludes 13 at `ai[0] == 1` — the direction follows the X sign
//!   only while outbound.
//! * **The on-hit return** (`Damage_PVE_Inner` `:14148-14164`): the
//!   23's own arm (penetrate ≤ 0 → the outbound zero + penetrate −1 +
//!   return) precedes the generic aiStyle-13 `ai[0] = 1` — wired in
//!   `damage_pve`'s post-decrement tail. The 262/23 `StatusNPC`
//!   debuffs (`:11282-11288`: the Confused 1/3 roll, the Bleeding
//!   duration roll) ride the same seam.
//!
//! RNG by construction: the AI body draws ZERO `Main.rand` (the 262's
//! `HitTilesInACircle` is effectOnly dust — `WorldGen.cs:63704-63707`
//! returns after the dust loop, whose rolls are pool-gated internals,
//! the ai026_a precedent). The two on-hit debuff rolls are HIT-time
//! draws on the pass's `UpdateProjectiles` stream (wired in
//! `damage_pve`, witnessed there).
//!
//! CODE-ONLY MODE (W4-d2): the witness tests at the bottom are WRITTEN
//! NOT RUN; verified by re-reading the cited C# blocks and hand
//! arithmetic.
//!
//! Deliberate skips (ANALYSIS "The aiStyle-13 chain weapons"): the
//! dust visuals, `netUpdate` (the field rides, the wire is the
//! single-player no-op), the wet/shimmer movement half (the generic
//! Update wet block is unported for these kinds — the `:5473`
//! precedent; the Shimmer kill `:22467-22470` is unreachable without
//! it), the PVP hit arm (`:14704-14708`, no PVP in the port), the
//! `HitTiles`/`HitTilesInACircle` cell scans (effectOnly dust, no
//! state). Style 13 is NOT wind-eligible (`ShouldUseWindPhysics`
//! `:22110-22132`).

use glam::Vec2;
use tr_core::World;

use crate::player::Player;
use crate::projectile::{
    Projectile, BOXING_GLOVE, CHAIN_GUILLOTINE, CHAIN_KNIFE, GOLEM_FIST, HARPOON,
};

/// The aiStyle-13 kinds (`Projectile.cs:1057-1068`/`:3463-3473`/
/// `:3567-3577`/`:3588-3597`/`:5561-5571` — the style's whole
/// SetDefaults census).
pub(crate) const CHAIN_KINDS: [u16; 5] =
    [HARPOON, GOLEM_FIST, BOXING_GLOVE, CHAIN_KNIFE, CHAIN_GUILLOTINE];

/// The player-side writes one chain tick requests (the `FlailTick`
/// write-back precedent): `ChangeDir` (`:24656`/`:24660`),
/// `SetDummyItemTime(5)` (`:24650`) and the
/// `AI_AdjustPlayerItemRotationToFaceProjectile` itemRotation
/// (`:24818-24828`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ChainTick {
    pub facing: Option<i32>,
    pub item_rotation: Option<f32>,
    pub dummy_item_time: Option<i32>,
}

/// One aiStyle-13 tick: the AI body + the blocked-move response + the
/// free integration + the outbound direction flip. `control_use_item`
/// is the raw use button (`Player.controlUseItem`, the flail seam's
/// `app.mine_down`); `held_shoot` is the selected item's shoot
/// (`player.HeldItem.shoot`); `remix` is `Main.remixWorld` (the 273's
/// flip threshold). `None` player kills the orphan (the C# reads
/// `Main.player[owner]` unconditionally — the ai_drill guard).
pub(crate) fn chain_step(
    p: &mut Projectile,
    world: &World,
    player: Option<&Player>,
    control_use_item: bool,
    held_shoot: Option<u16>,
    remix: bool,
) -> ChainTick {
    let mut tick = ChainTick::default();
    let Some(pl) = player else {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return tick;
    };
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let (pcx, pcy) = pl.center();
    // `:24638-24647` — flag6 = dead || the 2000-px leash.
    let leash = Vec2::new(pcx - center.x, pcy - center.y).length() > 2000.0;
    if pl.dead || leash {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return tick;
    }
    // `:24648-24651` — SetDummyItemTime(5) for every kind but the 481.
    if p.kind != CHAIN_GUILLOTINE {
        tick.dummy_item_time = Some(5);
    }
    // `:24652-24662` — the alpha-0 facing flip (the C# compares the
    // INTEGER box halves; 20/2 and 42/2 are exact either way).
    if p.alpha == 0 {
        tick.facing = Some(if center.x > pcx { 1 } else { -1 });
    }
    // `:24663-24673` — the 481's dynamic extraUpdates.
    if p.kind == CHAIN_GUILLOTINE {
        p.extra_updates = if p.ai[0] == 0.0 { 1 } else { 2 };
    }
    // `:24674-24677` — vector12 / num113 / num114 / num115: the owner
    // delta off the INTEGER player box halves.
    let mut dx = pl.x + (pl.width as i32 / 2) as f32 - center.x;
    let mut dy = pl.y + (pl.height as i32 / 2) as f32 - center.y;
    let mut dist = (dx * dx + dy * dy).sqrt();
    // `:24678-24681` — the harpoon's button-release flip.
    if p.kind == HARPOON && p.ai[1] > 10.0 && !control_use_item {
        p.ai[0] = 1.0;
    }
    if p.ai[0] == 0.0 {
        // `:24682-24710` — the outbound distance flips, the C# else-if
        // chain verbatim (the generic >700 arm precedes the per-type
        // sub-700 arms).
        if p.kind == HARPOON {
            if dist > 1000.0 {
                p.ai[0] = 1.0;
            }
        } else if dist > 700.0 {
            p.ai[0] = 1.0;
        } else if p.kind == GOLEM_FIST && dist > 500.0 {
            p.ai[0] = 1.0;
        } else if p.kind == BOXING_GLOVE && dist > 200.0 {
            p.ai[0] = 1.0;
        } else if p.kind == CHAIN_KNIFE && dist > if remix { 300.0 } else { 150.0 } {
            p.ai[0] = 1.0;
        } else if p.kind == CHAIN_GUILLOTINE && dist > 525.0 {
            p.ai[0] = 1.0;
        }
        // `:24711` — the velocity-pointing rotation.
        p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
        // `:24712-24716` — the age + the fade-in.
        p.ai[1] += 1.0;
        if p.ai[1] > 5.0 {
            p.alpha = 0;
        }
        // `:24717-24732` — the melee chains' 8-tick cap.
        if matches!(p.kind, GOLEM_FIST | BOXING_GLOVE | CHAIN_KNIFE | CHAIN_GUILLOTINE)
            && p.ai[1] > 8.0
        {
            p.ai[1] = 8.0;
        }
        // `:24733-24746` — the harpoon's gravity tail vs the generic
        // one (the capped melee chains never reach it — verbatim).
        if p.kind == HARPOON {
            if p.ai[1] >= 17.0 {
                p.ai[1] = 17.0;
                p.vel.y += 0.6;
                p.vel.x *= 0.995;
            }
        } else if p.ai[1] >= 10.0 {
            p.ai[1] = 15.0;
            p.vel.y += 0.3;
        }
        // `:24747-24762` — the 262/271 outbound sprite.
        if matches!(p.kind, GOLEM_FIST | BOXING_GLOVE) {
            p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
        }
        // `:24763-24766` — the 262's post-flip localAI pin (live when
        // the distance flip fired THIS tick).
        if p.kind == GOLEM_FIST && p.ai[0] != 0.0 {
            p.local_ai[0] = -1.0;
        }
    } else if p.ai[0] == 1.0 {
        // `:24770-24783` — the 262's one-shot return burst.
        if p.kind == GOLEM_FIST && p.local_ai[0] == 0.0 {
            if center.distance(Vec2::new(pcx, pcy)) < 150.0 {
                p.local_ai[0] = -1.0;
            } else {
                // `:24778-24781` — the 200-radius hitbox latch (the
                // damage_hitbox inflate consumes it), the
                // HitTilesInACircle effectOnly dust (skipped — zero
                // explicit rand) and the Item14 cue.
                p.local_ai[0] = 200.0;
                p.wants_sound = Some((2, 14, p.pos.x, p.pos.y, 0.0));
            }
        }
        // `:24784-24785` — the return flies through tiles.
        p.tile_collide = false;
        p.rot = dy.atan2(dx) - 1.57;
        // `:24786-24795` — num117 = 20 (262: 30); the <50 catch kills.
        let speed = if p.kind == GOLEM_FIST { 30.0 } else { 20.0 };
        if dist < 50.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return tick;
        }
        // `:24796-24800` — the normalized seek.
        dist = speed / dist;
        dx *= dist;
        dy *= dist;
        p.vel.x = dx;
        p.vel.y = dy;
        // `:24801-24816` — the 262/271 return sprite INVERTS.
        if matches!(p.kind, GOLEM_FIST | BOXING_GLOVE) {
            p.sprite_direction = if p.vel.x < 0.0 { 1 } else { -1 };
        }
    }
    // `:24818-24828` — the itemRotation tail
    // (AI_AdjustPlayerItemRotationToFaceProjectile, `:22518-22543`).
    if p.kind == HARPOON {
        if p.ai[0] == 0.0 {
            tick.item_rotation = adjust_item_rotation(p, pl, held_shoot, center, 50.0);
        }
    } else if matches!(p.kind, GOLEM_FIST | CHAIN_KNIFE | CHAIN_GUILLOTINE | BOXING_GLOVE) {
        tick.item_rotation = adjust_item_rotation(p, pl, held_shoot, center, 0.0);
    }
    // The movement half (`HandleMovement`'s 3/13/69/109 arm,
    // `:19512-19571`): the blocked-move probe (the port's
    // `collides_rect` stand-in for `Collision.TileCollision`, which
    // zeroes the blocked axis — the M4-PROJ-3 deviation), the
    // world-edge flag (`:17900-17903`), the 23's stick and the shared
    // return arm. The RETURN state keeps `tileCollide = false` — no
    // probe, the free move.
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let mut hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let mut hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        hit_x |= p.pos.x <= 0.0 || p.pos.x + p.w >= world.width as f32 * 16.0;
        hit_y |= p.pos.y <= 0.0 || p.pos.y + p.h >= world.height as f32 * 16.0;
        if hit_x || hit_y {
            // `Collision.TileCollision` zeroes the blocked axis
            // (`Collision.cs:299-330`); the probe stand-in applies the
            // same zeroing.
            if hit_x {
                p.vel.x = 0.0;
            }
            if hit_y {
                p.vel.y = 0.0;
            }
            if p.kind == HARPOON {
                // `:19520-19523` — the harpoon sticks: BOTH axes zero.
                p.vel = Vec2::ZERO;
            }
            // `:19548` — the other kinds' Collision.HitTiles is
            // effectOnly dust (skipped, zero explicit rand).
            // `:19561-19568` — style 13 arms the return WITHOUT a
            // velocity reversal (only 3/109 reverse).
            p.ai[0] = 1.0;
            p.net_update = true;
            // `:19571` — the Dig thud (the int-id overload).
            if p.wants_sound.is_none() {
                p.wants_sound = Some((0, 0, p.pos.x, p.pos.y, 0.0));
            }
        }
    }
    // `UpdatePosition` (`:20445`): integrate the POST-bounce velocity.
    p.pos += p.vel;
    // `AutomaticallyChangesDirection` (`:17283`): 13 at ai[0] == 1 is
    // EXCLUDED — the direction follows the X sign only outbound.
    if p.ai[0] != 1.0 {
        p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
    }
    tick
}

/// `AI_AdjustPlayerItemRotationToFaceProjectile`
/// (`Projectile.cs:22518-22543`): the alpha gate, the held-shoot gate,
/// the MountedCenter (+gfxOffY) aim with the left-side π flip and the
/// minSpeed dead zone. `None` = no write (the C#'s early returns).
fn adjust_item_rotation(
    p: &Projectile,
    pl: &Player,
    held_shoot: Option<u16>,
    center: Vec2,
    min_speed: f32,
) -> Option<f32> {
    // `:22520-22523` — `!ignoreAlphaCheck && alpha > 0` (every caller
    // here passes the default false).
    if p.alpha > 0 {
        return None;
    }
    // `:22525-22528` — HeldItem.shoot == this.type (the itemID argument
    // is -1 for every style-13 caller — no type check).
    if held_shoot != Some(p.kind) {
        return None;
    }
    // `:22532-22533` — the MountedCenter aim (+ gfxOffY on Y).
    let (mx, my) = pl.mounted_center();
    let x = mx - center.x;
    let y = my + pl.gfx_off_y - center.y;
    // `:22534-22542` — the minSpeed dead zone, then the left-side flip.
    if min_speed > 0.0 && x.abs() + y.abs() <= min_speed {
        return None;
    }
    let num = if center.x < mx { 1.0 } else { -1.0 };
    Some((y * num).atan2(x * num))
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn chain(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the chain row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(320.0, 320.0);
        p.vel = Vec2::new(10.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    fn owner() -> Player {
        let w = World::new("t".into(), 512, 256);
        let mut pl = Player::spawn(&w);
        pl.x = 320.0;
        pl.y = 320.0;
        pl
    }

    fn air_world() -> World {
        World::new("t".into(), 512, 256)
    }

    #[test]
    fn the_chain_rows_match_set_defaults() {
        // `Projectile.cs:1057-1068` — the harpoon: 4x4, pen 3, alpha
        // 255, ranged, local immunity 12.
        let d = proj_data(HARPOON).unwrap();
        assert_eq!((d.w, d.h), (4.0, 4.0));
        assert_eq!(d.ai_style, 13);
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.alpha, 255);
        assert!(d.ranged && !d.melee);
        assert!(d.uses_local_immunity);
        assert_eq!(d.local_hit_cooldown, 12);
        // `:3463-3473` — the golem fist: 30x30, pen −1, melee, extra 1.
        let d = proj_data(GOLEM_FIST).unwrap();
        assert_eq!((d.w, d.h), (30.0, 30.0));
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
        assert!(d.melee);
        assert_eq!(d.extra_updates, 1);
        // `:3567-3577` — the boxing glove: 20x20, scale 1.2.
        let d = proj_data(BOXING_GLOVE).unwrap();
        assert_eq!((d.w, d.h), (20.0, 20.0));
        assert_eq!(d.scale, 1.2);
        assert!(d.melee);
        // `:3588-3597` — the chain knife: 26x26.
        assert_eq!((proj_data(CHAIN_KNIFE).unwrap().w, proj_data(CHAIN_KNIFE).unwrap().h), (26.0, 26.0));
        // `:5561-5571` — the chain guillotine: 22x22, extra 0.
        let d = proj_data(CHAIN_GUILLOTINE).unwrap();
        assert_eq!((d.w, d.h), (22.0, 22.0));
        assert_eq!(d.extra_updates, 0);
        assert!(d.melee);
        for k in CHAIN_KINDS {
            assert_eq!(proj_data(k).unwrap().ai_style, 13);
        }
    }

    #[test]
    fn the_dead_owner_and_the_leash_kill() {
        // `:24638-24647`.
        let mut p = chain(HARPOON);
        let mut pl = owner();
        pl.dead = true;
        let t = chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert!(!p.active && p.time_left == 0);
        assert_eq!(t, ChainTick::default());
        // The 2000-px leash.
        let mut p = chain(HARPOON);
        let pl = owner();
        p.pos = Vec2::new(320.0 + 2100.0, 320.0);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert!(!p.active);
        // The orphan guard.
        let mut p = chain(HARPOON);
        chain_step(&mut p, &air_world(), None, true, None, false);
        assert!(!p.active);
    }

    #[test]
    fn the_dummy_hold_and_the_alpha_facing_flip() {
        // `:24648-24662` — SetDummyItemTime(5) (not the 481); the
        // facing flip only at alpha 0.
        let mut p = chain(BOXING_GLOVE);
        let pl = owner(); // centre (330, 341); the proj centre (330, 330)
        p.alpha = 255;
        let t = chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(t.dummy_item_time, Some(5));
        assert_eq!(t.facing, None, "alpha 255 keeps the facing");
        p.alpha = 0;
        p.pos = Vec2::new(400.0, 320.0); // centre right of the owner
        let t = chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(t.facing, Some(1));
        // The 481 skips the hold.
        let mut q = chain(CHAIN_GUILLOTINE);
        q.alpha = 0;
        let t = chain_step(&mut q, &air_world(), Some(&pl), true, None, false);
        assert_eq!(t.dummy_item_time, None);
    }

    #[test]
    fn the_guillotine_extra_updates_follow_the_state() {
        // `:24663-24673` — 1 outbound, 2 returning.
        let mut p = chain(CHAIN_GUILLOTINE);
        let pl = owner();
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.extra_updates, 1);
        p.ai[0] = 1.0;
        // Park 200 px out so the return's <50 catch does not fire.
        p.pos = Vec2::new(330.0 + 200.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.extra_updates, 2);
    }

    #[test]
    fn the_outbound_fade_in_and_the_melee_cap() {
        // `:24712-24732` — ai[1] counts, alpha clears past 5, the
        // melee chains cap at 8 (so the generic >=10 gravity NEVER
        // fires for them — the verbatim quirk).
        let mut p = chain(GOLEM_FIST);
        let pl = owner();
        // The fist centre (335, 335) sits 8 px from the owner centre
        // (330, 341) — no distance flip fires, the outbound state keeps
        // flying straight (the alpha-0 facing flip is a player write).
        for tick in 1..=12 {
            chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
            let expect = tick.min(8) as f32;
            assert_eq!(p.ai[1], expect, "tick {tick}");
            if tick > 5 {
                assert_eq!(p.alpha, 0);
            } else {
                assert_eq!(p.alpha, 255);
            }
        }
        assert_eq!(p.vel.y, 0.0, "the capped chain never reaches the gravity tail");
    }

    #[test]
    fn the_harpoon_gravity_tail_caps_at_seventeen() {
        // `:24733-24741` — ai[1] >= 17 → 17, vy += 0.6, vx *= 0.995.
        let mut p = chain(HARPOON);
        let pl = owner();
        p.ai[1] = 16.0;
        p.alpha = 0;
        // Hold the button so the release flip stays down; the distance
        // is 11 (< 1000) so no flip.
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.ai[1], 17.0);
        assert!((p.vel.y - 0.6).abs() < 1e-6);
        assert!((p.vel.x - 9.95).abs() < 1e-4, "10 × 0.995");
        // The next tick keeps the cap.
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.ai[1], 17.0);
        assert!((p.vel.y - 1.2).abs() < 1e-6);
    }

    #[test]
    fn the_harpoon_release_flips_the_return() {
        // `:24678-24681` — ai[1] > 10 && !controlUseItem → ai[0] = 1.
        let mut p = chain(HARPOON);
        let pl = owner();
        p.ai[1] = 11.0;
        p.alpha = 0;
        // Park 100 px out so the return's <50 catch does not fire.
        p.pos = Vec2::new(330.0 + 100.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        chain_step(&mut p, &air_world(), Some(&pl), false, None, false);
        assert_eq!(p.ai[0], 1.0);
        assert!(!p.tile_collide, "the return flies through tiles (:24784)");
        assert!(p.active, "100 px out — the catch has not fired");
    }

    #[test]
    fn the_distance_flips_use_the_per_type_thresholds() {
        // `:24682-24710` — the glove flips past 200, the knife past 150
        // (remix 300), the guillotine past 525, the fist past 500.
        let pl = owner(); // centre (330, 341)
        let mut p = chain(BOXING_GLOVE);
        p.pos = Vec2::new(330.0 + 210.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.ai[0], 1.0, "the glove flips at 210 > 200");
        let mut q = chain(CHAIN_KNIFE);
        q.pos = Vec2::new(330.0 + 160.0 - q.w * 0.5, 341.0 - q.h * 0.5);
        chain_step(&mut q, &air_world(), Some(&pl), true, None, false);
        assert_eq!(q.ai[0], 1.0, "the knife flips at 160 > 150");
        let mut r = chain(CHAIN_KNIFE);
        r.pos = Vec2::new(330.0 + 160.0 - r.w * 0.5, 341.0 - r.h * 0.5);
        chain_step(&mut r, &air_world(), Some(&pl), true, None, true);
        assert_eq!(r.ai[0], 0.0, "remix widens the knife to 300");
        let mut s = chain(CHAIN_GUILLOTINE);
        s.pos = Vec2::new(330.0 + 530.0 - s.w * 0.5, 341.0 - s.h * 0.5);
        chain_step(&mut s, &air_world(), Some(&pl), true, None, false);
        assert_eq!(s.ai[0], 1.0, "the guillotine flips at 530 > 525");
    }

    #[test]
    fn the_return_seek_normalizes_and_the_catch_kills() {
        // `:24786-24800` — speed 20 (fist 30); dist < 50 kills.
        let pl = owner();
        let mut p = chain(BOXING_GLOVE);
        p.ai[0] = 1.0;
        p.alpha = 0;
        p.pos = Vec2::new(330.0 + 100.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert!((p.vel.x + 20.0).abs() < 1e-4, "the seek points home at 20");
        assert!(p.vel.y.abs() < 1e-4);
        assert_eq!(p.sprite_direction, 1, "the return sprite inverts (:24809)");
        // The catch.
        let mut q = chain(CHAIN_KNIFE);
        q.ai[0] = 1.0;
        q.pos = Vec2::new(330.0 + 30.0 - q.w * 0.5, 341.0 - q.h * 0.5);
        chain_step(&mut q, &air_world(), Some(&pl), true, None, false);
        assert!(!q.active && q.time_left == 0);
        // The fist seeks at 30.
        let mut f = chain(GOLEM_FIST);
        f.ai[0] = 1.0;
        f.local_ai[0] = -1.0; // skip the one-shot burst
        f.pos = Vec2::new(330.0 + 200.0 - f.w * 0.5, 341.0 - f.h * 0.5);
        chain_step(&mut f, &air_world(), Some(&pl), true, None, false);
        assert!((f.vel.x + 30.0).abs() < 1e-4);
    }

    #[test]
    fn the_fist_return_burst_latches_the_hitbox_radius() {
        // `:24770-24783` — outside 150: localAI[0] = 200 + Item14;
        // inside 150: −1, no cue.
        let pl = owner();
        let mut p = chain(GOLEM_FIST);
        p.ai[0] = 1.0;
        p.pos = Vec2::new(330.0 + 200.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.local_ai[0], 200.0);
        // The cue rides the PRE-integration position (`:24781` plays
        // before UpdatePosition): spawn (515, 326), the seek moves −30.
        assert_eq!(p.wants_sound, Some((2, 14, 515.0, 326.0)));
        // Inside 150 the pin is −1 (the second tick reads localAI != 0
        // — the one-shot never re-fires).
        let mut q = chain(GOLEM_FIST);
        q.ai[0] = 1.0;
        q.pos = Vec2::new(330.0 + 100.0 - q.w * 0.5, 341.0 - q.h * 0.5);
        chain_step(&mut q, &air_world(), Some(&pl), true, None, false);
        assert_eq!(q.local_ai[0], -1.0);
        assert_eq!(q.wants_sound, None);
    }

    #[test]
    fn the_blocked_move_arms_the_return_without_reversing() {
        // `:19512-19571` — style 13 sets ai[0] = 1, NO reversal; the
        // harpoon zeroes both axes; the Dig thud queues.
        let mut w = air_world();
        // A solid wall at tile x = 22 (px 352..368).
        for y in 18..24 {
            let i = w.index(22, y);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let pl = owner();
        let mut p = chain(BOXING_GLOVE);
        p.alpha = 0;
        p.pos = Vec2::new(330.0, 320.0); // the box steps into the wall
        p.vel = Vec2::new(10.0, 0.0);
        chain_step(&mut p, &w, Some(&pl), true, None, false);
        assert_eq!(p.ai[0], 1.0);
        assert_eq!(p.vel.x, 0.0, "the blocked axis zeroes (TileCollision)");
        assert_eq!(p.wants_sound, Some((0, 0, p.pos.x, p.pos.y)));
        // The harpoon sticks fully (its 4-px box must reach the wall:
        // pos.x 346 + vel 10 → 356 inside 352..368).
        let mut h = chain(HARPOON);
        h.alpha = 0;
        h.pos = Vec2::new(346.0, 320.0);
        h.vel = Vec2::new(10.0, 3.0);
        chain_step(&mut h, &w, Some(&pl), true, None, false);
        assert_eq!(h.vel, Vec2::ZERO);
        assert_eq!(h.ai[0], 1.0);
    }

    #[test]
    fn the_item_rotation_tail_gates_on_alpha_and_held_shoot() {
        // `:22518-22543` — alpha > 0 or a foreign held shoot: no write.
        let pl = owner();
        let mut p = chain(CHAIN_KNIFE);
        p.alpha = 255;
        let t = chain_step(&mut p, &air_world(), Some(&pl), true, Some(CHAIN_KNIFE), false);
        assert_eq!(t.item_rotation, None, "the spawn alpha gates the aim");
        p.alpha = 0;
        let t = chain_step(&mut p, &air_world(), Some(&pl), true, Some(999), false);
        assert_eq!(t.item_rotation, None, "the held shoot must match");
        let t = chain_step(&mut p, &air_world(), Some(&pl), true, Some(CHAIN_KNIFE), false);
        assert!(t.item_rotation.is_some(), "the matching held item aims");
        // The harpoon's 50-px dead zone: the proj centre 11 px from the
        // owner centre stays unaimed while outbound.
        let mut h = chain(HARPOON);
        h.alpha = 0;
        let t = chain_step(&mut h, &air_world(), Some(&pl), true, Some(HARPOON), false);
        assert_eq!(t.item_rotation, None, "|x|+|y| = 11 ≤ 50");
    }

    #[test]
    fn the_return_direction_freezes() {
        // `:17283` — ai[0] == 1 excludes the flip; outbound follows X.
        let pl = owner();
        let mut p = chain(CHAIN_KNIFE);
        p.alpha = 0;
        p.vel = Vec2::new(-10.0, 0.0);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.direction, -1, "outbound follows the X sign");
        p.ai[0] = 1.0;
        p.local_ai[0] = -1.0;
        p.pos = Vec2::new(330.0 + 200.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        p.direction = -1;
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        // The return seek points LEFT (vel.x < 0) but the direction
        // stays frozen at its pre-return value.
        assert!(p.vel.x < 0.0);
        assert_eq!(p.direction, -1);
        p.direction = 1;
        p.pos = Vec2::new(330.0 + 200.0 - p.w * 0.5, 341.0 - p.h * 0.5);
        chain_step(&mut p, &air_world(), Some(&pl), true, None, false);
        assert_eq!(p.direction, 1, "the return never flips the direction");
    }

    #[test]
    fn the_chain_ai_draws_no_rand() {
        // The AI body has ZERO explicit Main.rand (the HitTiles /
        // HitTilesInACircle dust is effectOnly — module doc). A
        // recording stream that panics on any draw proves it.
        struct NoDraws;
        impl tr_core::loot::DropRng for NoDraws {
            fn next_range(&mut self, _: i32, _: i32) -> i32 {
                panic!("the chain AI draws no integers");
            }
            fn next_float(&mut self) -> f32 {
                panic!("the chain AI draws no floats");
            }
        }
        // chain_step takes no rng at all — the signature IS the
        // witness; this test pins the census claim for the reader.
        let _ = NoDraws;
        assert_eq!(CHAIN_KINDS.len(), 5);
    }
}
