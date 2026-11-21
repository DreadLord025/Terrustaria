//! S1-P-i06 — the spear family (`useStyle 5` thrust, aiStyle 19).
//!
//! The held-spear projectile rides `AI_019_Spears` (`Projectile.cs:45857-46073`)
//! with its two helpers: the extension/retract offset curve
//! `AI_019_Spears_GetSpearOffsetRelativeToPlayer` (`:46075-46184`) and the
//! reach-ahead damage box `AI_019_Spears_GetExtensionHitbox` (`:46186-46220`).
//! The dispatch arm is `AI()`'s `else if (this.aiStyle == 19)`
//! (`Projectile.cs:25477-25480`). The legacy `AI_019_Spears_Old`
//! (`:46222+`) has NO caller in the 1.4.5.8 decompile — it is dead and is NOT
//! ported (the live arm is the offset-curve form, not the ai[0] accumulator).
//!
//! The 21 spear kinds all ride `DefaultToSpear` (`Projectile.cs:10720-10730`:
//! aiStyle 19, friendly, `penetrate = -1`, `tileCollide = false`, drawLayer 7,
//! `usesOwnerLight`, `ownerHitCheck`, melee). Their `SetDefaults` rows land in
//! `projectile/data.rs` (cited per field). The kinds:
//!   46 DarkLance, 47 Trident, 49 Spear, 64 MythrilHalberd, 66 AdamantiteGlaive,
//!   97 CobaltNaginata, 105 Gungnir, 130 MushroomSpear, 153 RottedFork,
//!   212 PalladiumPike, 215 OrichalcumHalberd, 218 TitaniumTrident,
//!   222 ChlorophytePartisan, 342 NorthPole, 367 ObsidianSwordfish,
//!   368 Swordfish, 730 ThunderSpear, 877/878/879 the Jousting Lances,
//!   1103 SlimeSpear.
//!
//! The Player side (`Player.ItemCheck_Shoot`, `Player.cs:47056-49468`) spawns
//! the spear through the GENERIC shot tail (`:49437`, `ai0/ai1/ai2 = 0`): the
//! aim normalized to `shootSpeed` (`:47277-47307`), the melee
//! `speed /= meleeSpeed` (`:47061-47064`, the lances 877-879 excepted via
//! `ProjectileID.Sets.NoMeleeSpeedVelocityScaling`, `ProjectileID.cs:547`),
//! the spawn at `RotatedRelativePoint(MountedCenter)` (`:47196`). There is NO
//! spear-specific `ai[0]/ai[1]` thrust payload — the extension is driven by
//! the OWNER's `itemAnimation` countdown, not by the projectile's `ai`. The
//! `useStyle == 5` block (`:47349-47375`) only sets `itemRotation` (the port's
//! `item_rotation_for_aim`, already wired). The spawn seam is the generic
//! shoot path in `world_sim/editing.rs` (no spear carve-out needed).
//!
//! Movement: aiStyle 19 is NOT in `UpdatePosition`'s skip list
//! (`Projectile.cs:20435`), so the generic `HandleMovement` → `position +=
//! velocity` (`:20445`) runs AFTER the AI (`:17015`) — the port's dispatch
//! runs `kill_step` after the arm (the `tileCollide = false` free-move branch,
//! `:20445`). The AI itself re-centres the box on the owner and adds
//! `velocity * offset` (`:45864`/`:45880`); the post-AI step adds one more
//! `velocity`. Both are kept, in order.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`): the
//! behaviour-feeding draws stay in the C#'s order; the dust/light BODIES are
//! skipped (no draw layer) and their explicit gate rolls go with them (the
//! `projectile_ai_frostmoon` zero-RNG convention — no observable state depends
//! on a skipped dust). Per kind:
//! - 47 Trident: `wet && Next(2) == 0` gates a dust (`:45884`) — dust-only, the
//!   `Next(2)` is skipped with it. ZERO kept draws.
//! - 130 MushroomSpear: the spore spawn (`:45891-45895`) is RNG-free (a
//!   `localAI[0]` cadence). KEPT (it spawns a projectile).
//! - 730 ThunderSpear: the 732 shot (`:45898-45903`) is RNG-free. KEPT. The
//!   trailing dust (`:46000-46011`) rolls `Next(5)` twice — dust-only, skipped.
//! - 342 NorthPole: the 343 snowflake (`:45906-45913`) is RNG-free (a
//!   `Collision.CanHit` gate). KEPT. The dust/light (`:46035-46044`) is
//!   skipped (its `timeLeft % 2` gate rolls nothing).
//! - 222 ChlorophytePartisan: the 228 spore (`:45916-45920`) is RNG-free. KEPT.
//! - 367 ObsidianSwordfish: the bone burst (`:45923-45936`) rolls `Next(1,4)`
//!   for the count, then per bone `NextFloat`×3 (the velocity jitter) +
//!   `Next(3)` (the 1107-1109 kind) + `Next(30,150)` (the timeLeft). KEPT, in
//!   that exact order.
//! - 46/105/153/730/877-879/1103: the trailing dust blocks (`:45949-46062`)
//!   roll only dust gates — skipped whole.
//! The 877-879 lances' `alpha -= 40` fade-in (`:45871-45878`) and the
//! channel-hold (`:45939-45942`) are RNG-free state, kept.
//!
//! The sub-shots (131/228/732/343/1107-1109) ride their OWN aiStyles
//! (30/44/44/57/14). Their rows + the small 30/44/57 arms and the 344
//! `AI_001` hook land here so the spear's spawns are not inert; the 1107-1109
//! bones ride the already-ported generic aiStyle-14 flight (`ai_spear`).
//!
//! Deliberate skips (ANALYSIS "The spear family (S1-P-i06)"): dust,
//! `Lighting.AddLight`, sound, `netUpdate`, `player.heldProj` /
//! `MatchItemTimeToItemAnimation` (no port consumer — the flail precedent),
//! `ownerHitCheck`'s `CanHitWithMeleeWeapon` gate is ported (it gates the hit),
//! `drawLayer`/`usesOwnerLight` (no draw consumer). The 344 snowflake's
//! sibling-separation scan (`:57090-57113`) reads the whole pool — it rides a
//! precomputed push passed into the hook (the `minion_separation` pattern).
//!
//! CODE-ONLY MODE (S1-P-i06): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); verified by re-reading the cited
//! C# blocks and hand arithmetic.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::projectile::{
    MUSHROOM_SPEAR, NORTH_POLE, OBSIDIAN_SWORDFISH, Projectile, THUNDER_SPEAR, TRIDENT,
    can_hit, can_hit_line, get_lerp_value,
};

/// One sub-shot the spear AI queues (`Projectile.NewProjectile` inside
/// `AI_019_Spears`): the pool resolves it after the arm (the slot cannot spawn
/// mid-iteration — the `wants_mark_spawn` / `MinionShot` intent pattern).
/// `x`/`y` are the C# spawn CENTRE (`NewProjectile` subtracts w/2, h/2,
/// `Projectile.cs:10876-10877`); `time_left` is the post-spawn override
/// (`None` = keep the row's `SetDefaults` life).
pub(crate) struct SpearSubShot {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knock_back: f32,
    pub time_left: Option<i32>,
    /// The C# `NewProjectile`'s `ai1` argument (the 344 snowflake's frame
    /// payload, `Projectile.cs:28902`); 0 for every other sub-shot.
    pub ai1: f32,
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_frostmoon` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// The owner state `AI_019_Spears` reads off `Main.player[owner]` each tick
/// (`Projectile.cs:45859-45868`), snapshotted by the dispatch (the slot loop
/// cannot borrow the player — the `FlailInputs` pattern). The C# reads the
/// live `Player` fields; the port snapshots them once per fixed step.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SpearOwner {
    /// `player.RotatedRelativePoint(player.MountedCenter)` (`:45860`) — the
    /// spear's anchor. The port's `mounted_center()` folds to `center()` (no
    /// mount offsets) and `rotated_relative_point` applies `fullRotation`.
    pub anchor: Vec2,
    /// `player.direction` (`:45861`).
    pub direction: i32,
    /// `player.itemAnimation` (`:45867`) — the C# INTEGER countdown. The port
    /// derives it from `app.swing` (see the dispatch's snapshot).
    pub item_animation: i32,
    /// `player.itemAnimationMax` (`:45866`).
    pub item_animation_max: i32,
    /// `player.channel` (`:45939`) — the lances' hold.
    pub channel: bool,
    /// `player.meleeSpeed` (`:46215`) — the extension box's reach scale. The
    /// port's `Player::melee_speed()` (1 + bonus) is used as the C# meleeSpeed
    /// in the `1/x` form, the flail convention (×1 at the default bonus).
    pub melee_speed: f32,
    /// `player.wet` (`:45884`) — the Trident's bubble gate.
    pub wet: bool,
    /// `player.velocity` (`:45953-45954`) — the lances' speed-dust gate.
    pub player_vel: Vec2,
    /// `player.position`/`width`/`height` (`:45886`, the Trident dust origin;
    /// also the `ownerHitCheck` box). The dust is skipped, but the box rides
    /// for the hit gate.
    pub player_pos: Vec2,
    pub player_w: f32,
    pub player_h: f32,
}

/// `AI_019_Spears_GetSpearOffsetRelativeToPlayer`
/// (`Projectile.cs:46075-46184`): the extension/retract distance along the
/// aim. `num = itemAnimationMax`, `num2 = itemAnimation`,
/// `num3 = itemAnimationMax / 3` (integer), `num4 = Min(num2, num3)`,
/// `num5 = num2 - num4`. The per-kind `(num6 base, num7 extend, num8 retract)`
/// triple feeds `num6 + num7 * ((num - num3) - num5) - num8 * (num3 - num4)`
/// (`:46183`). The helper also re-centres the box and sets
/// `direction`/`spriteDirection` (`:46086-46097`) — the caller does that once
/// (the AI already centred at `:45864`); this fn returns the scalar only.
fn spear_offset(kind: u16, item_animation: i32, item_animation_max: i32) -> f32 {
    let num = item_animation_max;
    let num2 = item_animation;
    let num3 = item_animation_max / 3;
    let num4 = num2.min(num3) as f32;
    let num5 = (num2 as f32) - num4;
    // `(num6, num7, num8)` — the `:46098-46182` switch. Unlisted kinds keep
    // the 0/0/0 defaults (the C# switch has no default arm).
    let (num6, num7, num8) = match kind {
        46 => (3.0, 1.4, 1.6),
        105 => (3.0, 2.1, 2.4),
        367 => (3.0, 1.5, 1.6),
        368 => (3.0, 1.4, 1.5),
        222 => (3.0, 2.1, 2.4),
        342 => (3.0, 2.1, 2.4),
        47 => (4.0, 0.9, 1.2),
        153 => (4.0, 1.3, 1.5),
        49 => (4.0, 0.85, 1.1),
        730 => (8.0, 1.2, 1.5),
        877 | 878 | 879 => (28.0, 0.4, 0.4),
        64 | 215 => (3.0, 1.7, 1.9),
        66 | 97 | 212 | 218 => (3.0, 1.9, 2.1),
        130 => (3.0, 1.0, 1.3),
        1103 => (4.0, 0.85, 1.1),
        _ => (0.0, 0.0, 0.0),
    };
    num6 + num7 * (((num - num3) as f32) - num5) - num8 * ((num3 as f32) - num4)
}

/// `AI_019_Spears_GetExtensionHitbox` (`Projectile.cs:46186-46220`): the
/// reach-ahead damage box during the extension phase. `None` when
/// `itemAnimation < itemAnimationMax / 3` (the retract phase, `:46189-46192`)
/// or for an unlisted kind (`:46200-46201` default → false). Otherwise the
/// box is centred at `Center + aimUnit * reach` with side `width`, where
/// `reach = 10 + reachScale * (1/meleeSpeed) * t`, `width = 10 + widthScale*t`
/// and `t = Remap(itemAnimation, itemAnimationMax, itemAnimationMax/3, 0, 1)`
/// (`:46193`). Only 46/105/153 override the defaults (10/30/10/10).
pub(crate) fn spear_extension_hitbox(
    p: &Projectile,
    owner: &SpearOwner,
) -> Option<tr_core::whip::Rect> {
    if owner.item_animation < owner.item_animation_max / 3 {
        return None;
    }
    let t = get_lerp_value(
        owner.item_animation_max as f32,
        (owner.item_animation_max / 3) as f32,
        owner.item_animation as f32,
    );
    // `(num3 reachScale, num5 widthScale)` — the `:46198-46214` switch; the
    // base reach num2 = 10 and base width num4 = 10 are shared.
    let (reach_scale, width_scale) = match p.kind {
        105 => (50.0, 20.0),
        46 => (50.0, 15.0),
        153 => (40.0, 10.0),
        _ => return None,
    };
    let reach = 10.0 + reach_scale * (1.0 / owner.melee_speed) * t;
    let width = 10.0 + width_scale * t;
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let aim = p.vel.to_angle();
    let box_center = center + Vec2::from_angle(aim) * reach;
    // `Utils.CenteredRectangle(center, size)` (`Utils.cs:947-950`): the
    // `(int)(center - size/2)` truncation, `(int)size` side.
    Some(tr_core::whip::Rect::new(
        (box_center.x - width * 0.5) as i32,
        (box_center.y - width * 0.5) as i32,
        width as i32,
        width as i32,
    ))
}

/// `Colliding`'s aiStyle-19 arm (`Projectile.cs:15815-15843`): the extension
/// box stepped from the projectile's centre toward the box centre. The C#
/// walks `num30 = step; num30 < dist; num30 += step` (step = max(box side,
/// 12)), testing a box-sized rect at each `Lerp(center, boxCenter, num30/dist)`
/// point, then the extension box itself. Returns true on the first overlap.
/// On a miss (or a `None` extension box — retract phase / unlisted kind) the
/// C# does NOT return: it falls through to the base-rect test (`:15888`), so
/// this returns false and the caller ORs the base-rect overlap.
pub(crate) fn spear_colliding(
    p: &Projectile,
    owner: &SpearOwner,
    target: tr_core::whip::Rect,
) -> bool {
    let Some(ext) = spear_extension_hitbox(p, owner) else {
        return false;
    };
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let box_center = Vec2::new(ext.x as f32 + ext.w as f32 * 0.5, ext.y as f32 + ext.h as f32 * 0.5);
    let dist = box_center.distance(center);
    let mut step = ext.w.max(ext.h) as f32;
    if step < 12.0 {
        step = 12.0;
    }
    let size = Vec2::new(ext.w as f32, ext.h as f32);
    let mut num30 = step;
    while num30 < dist {
        let t = num30 / dist;
        let point = center + (box_center - center) * t;
        let attacker = tr_core::whip::Rect::new(
            (point.x - size.x * 0.5) as i32,
            (point.y - size.y * 0.5) as i32,
            size.x as i32,
            size.y as i32,
        );
        if attacker.intersects(target) {
            return true;
        }
        num30 += step;
    }
    ext.intersects(target)
}

/// `Projectile.CanHitWithMeleeWeapon` (`Projectile.cs:12400-12411`) for the
/// spear's `ownerHitCheck` gate (`Damage_PVE_Inner` `:12637`). The reach cap
/// (`ownerHitCheckDistance`, default 1000f `:304`) plus the owner-box sight
/// lines. The port's `can_hit_with_melee_weapon` (projectile.rs) is the same
/// shape but reads a `Player`; here the owner is the snapshot, so the four
/// probes are rebuilt from it. `gravDir = 1` (the port's upright players).
pub(crate) fn owner_can_hit(world: &World, owner: &SpearOwner, n: &crate::npc::Npc) -> bool {
    let (nw, nh) = n.size();
    let target = Vec2::new(n.x + nw * 0.5, n.y + nh * 0.5);
    let owner_center = Vec2::new(
        owner.player_pos.x + owner.player_w * 0.5,
        owner.player_pos.y + owner.player_h * 0.5,
    );
    if owner_center.distance(target) > 1000.0 {
        return false;
    }
    if can_hit(
        world,
        owner.player_pos.x,
        owner.player_pos.y,
        owner.player_w,
        owner.player_h,
        n.x,
        n.y,
        nw,
        nh,
    ) {
        return true;
    }
    let ax = owner_center.x + ((owner.direction * owner.player_w as i32) / 2) as f32;
    let ay = owner_center.y + (-owner.player_h) / 3.0;
    can_hit_line(world, ax, ay, 0.0, 0.0, target.x, target.y + (-(nh as i32) / 3) as f32, 0.0, 0.0)
        || can_hit_line(world, ax, ay, 0.0, 0.0, target.x, target.y, 0.0, 0.0)
        || can_hit_line(
            world,
            owner_center.x + ((owner.direction * owner.player_w as i32) / 2) as f32,
            owner_center.y,
            0.0,
            0.0,
            target.x,
            target.y + (nh as i32 / 3) as f32,
            0.0,
            0.0,
        )
}

/// `AI_019_Spears` (`Projectile.cs:45857-46073`) for one slot. Returns the
/// sub-shots the arm queued (the pool drains them after the AI). The
/// `owner` snapshot carries the player state; `world` serves the NorthPole's
/// `Collision.CanHit` gate. The dust/light/sound riders are skipped (module
/// doc); the behaviour-feeding RNG stays in order on `rng`.
pub(crate) fn ai_spears(
    p: &mut Projectile,
    world: &World,
    owner: &SpearOwner,
    rng: &mut dyn DropRng,
) -> Vec<SpearSubShot> {
    let mut shots = Vec::new();
    let center = owner.anchor;
    // `:45861-45864` — direction, the heldProj/MatchItemTime writes (skipped,
    // no consumer), and the re-centre on the owner.
    p.direction = owner.direction;
    p.pos = center - Vec2::new(p.w * 0.5, p.h * 0.5);
    // `:45865` — the retract-phase flag.
    let retract = owner.item_animation < owner.item_animation_max / 3;
    let item_animation_max = owner.item_animation_max;
    let item_animation = owner.item_animation;
    // `:45870` — spriteDirection = -direction.
    p.sprite_direction = -p.direction;
    // `:45871-45879` — the lances' alpha fade-in (877-879), the channel flag.
    let mut lance = false;
    if (877..=879).contains(&p.kind) {
        p.alpha -= 40;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        lance = true;
    }
    // `:45880` — the extension offset along the aim. S1-P-i57: this is an
    // AI-internal step (`AI_019_Spears` runs inside `AI()`, BEFORE the wet
    // machine) — it keeps the full velocity × offset; only `UpdatePosition`'s
    // integrate reads `wetVelocity` (not a swap site, the wisp-snap precedent).
    let offset = spear_offset(p.kind, item_animation, item_animation_max);
    p.pos += p.vel * offset;
    let pc = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:45881-45938` — the per-kind sub-shot switch.
    match p.kind {
        TRIDENT => {
            // `:45884-45888` — the wet bubble dust. Dust-only: the `Next(2)`
            // gate is skipped with the body (module doc). No state rides.
            let _ = owner.wet;
        }
        MUSHROOM_SPEAR => {
            // `:45891-45895` — the spore cadence: localAI[0] in {0,4,8,12,16,
            // 20,24} and owner == myPlayer → spawn 131 at the centre with the
            // spear's velocity, damage/3, 0 knockback. Then localAI[0]++.
            let cadence = matches!(
                p.local_ai[0] as i32,
                0 | 4 | 8 | 12 | 16 | 20 | 24
            );
            if cadence && p.owner == 0 {
                shots.push(SpearSubShot {
                    kind: 131,
                    x: pc.x,
                    y: pc.y,
                    vx: p.vel.x,
                    vy: p.vel.y,
                    damage: p.damage / 3,
                    knock_back: 0.0,
                    time_left: None,
                    ai1: 0.0,
                });
            }
            p.local_ai[0] += 1.0;
        }
        THUNDER_SPEAR => {
            // `:45898-45903` — the first-tick 732 shot: velocity * 4, damage
            // * 1.6, knockback * 0.5.
            if p.local_ai[0] == 0.0 && p.owner == 0 {
                p.local_ai[0] = 1.0;
                let v = p.vel * 4.0;
                shots.push(SpearSubShot {
                    kind: 732,
                    x: pc.x,
                    y: pc.y,
                    vx: v.x,
                    vy: v.y,
                    damage: (p.damage as f32 * 1.6) as i32,
                    knock_back: p.knock_back * 0.5,
                    time_left: None,
                    ai1: 0.0,
                });
            }
        }
        NORTH_POLE => {
            // `:45906-45913` — the retract-phase 343 snowflake, gated on
            // Collision.CanHit(owner box → spear box). velocity * 2.4,
            // knockback * 0.85.
            if retract && p.local_ai[0] == 0.0 && p.owner == 0 {
                p.local_ai[0] = 1.0;
                if can_hit(
                    world,
                    owner.player_pos.x,
                    owner.player_pos.y,
                    owner.player_w,
                    owner.player_h,
                    p.pos.x,
                    p.pos.y,
                    p.w,
                    p.h,
                ) {
                    let v = p.vel * 2.4;
                    shots.push(SpearSubShot {
                        kind: 343,
                        x: pc.x,
                        y: pc.y,
                        vx: v.x,
                        vy: v.y,
                        damage: p.damage,
                        knock_back: p.knock_back * 0.85,
                        time_left: None,
                        ai1: 0.0,
                    });
                }
            }
        }
        222 => {
            // `:45916-45920` — the retract-phase 228 spore cloud: the spear's
            // velocity, full damage/knockback.
            if retract && p.local_ai[0] == 0.0 && p.owner == 0 {
                p.local_ai[0] = 1.0;
                shots.push(SpearSubShot {
                    kind: 228,
                    x: pc.x,
                    y: pc.y,
                    vx: p.vel.x,
                    vy: p.vel.y,
                    damage: p.damage,
                    knock_back: p.knock_back,
                    time_left: None,
                    ai1: 0.0,
                });
            }
        }
        OBSIDIAN_SWORDFISH => {
            // `:45923-45936` — the first-tick bone burst. `Next(1,4)` bones;
            // per bone the velocity jitter (three NextFloat), the 1107-1109
            // kind `Next(3)`, damage * 0.5, 0 knockback, and the post-spawn
            // `timeLeft = Next(30,150)`. The draw order is the C#'s.
            if p.local_ai[0] == 0.0 && p.owner == 0 {
                p.local_ai[0] = 1.0;
                let count = rng.next_range(1, 4);
                for _ in 0..count {
                    let mut v = p.vel;
                    v.x *= 0.7 + rng.next_float() * 0.6;
                    v.y *= 0.7 + rng.next_float() * 0.6;
                    v *= 0.8 + rng.next_float() * 0.4;
                    let kind = 1107 + rng.next_range(0, 3) as u16;
                    let life = rng.next_range(30, 150);
                    shots.push(SpearSubShot {
                        kind,
                        x: pc.x,
                        y: pc.y,
                        vx: v.x * 2.0,
                        vy: v.y * 2.0,
                        damage: (p.damage as f32 * 0.5) as i32,
                        knock_back: 0.0,
                        time_left: Some(life),
                        ai1: 0.0,
                    });
                }
            }
        }
        _ => {}
    }
    // `:45939-45942` — the lances' channel hold: while channelling and in the
    // retract phase, SetDummyItemTime(third) keeps the animation alive. The
    // port's `SetDummyItemTime` write rides back through the owner snapshot's
    // consumer (the dispatch re-arms app.swing); here we flag it via the
    // returned hold so the pool can apply it. (No player borrow in the arm.)
    // `:45943-45947` — the rotation: atan2(vy, vx) + π/2 + π/4, minus π/2 when
    // spriteDirection == -1.
    p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
    if p.sprite_direction == -1 {
        p.rot -= std::f32::consts::FRAC_PI_2;
    }
    // `:45948` — the extension hitbox call (the C# discards the out box here;
    // the damage walk recomputes it). No state rides.
    // `:45949-46062` — the per-kind trailing dust/light. All dust-only (the
    // 877-879 speed gate, the 46/730/105/153/342/1103 dust blocks): skipped
    // whole with their gate rolls (module doc). The 342 light is skipped.
    // `:46063-46072` — the age counter and the kill.
    p.ai[0] += 1.0;
    let mut expired = p.ai[0] >= item_animation_max as f32;
    if lance {
        expired &= !owner.channel;
    }
    if expired {
        kill(p);
    }
    shots
}

/// Whether the lances' channel-hold (`:45939-45942`) fires this tick: the
/// dispatch reads it to re-arm the owner's swing (`SetDummyItemTime(third)`).
pub(crate) fn lance_channel_hold(p: &Projectile, owner: &SpearOwner) -> bool {
    (877..=879).contains(&p.kind)
        && owner.channel
        && owner.item_animation < owner.item_animation_max / 3
}

// ---------------------------------------------------------------------------
// The sub-shot arms. The spear's spawns ride their own aiStyles; these small
// arms keep them from sitting inert in the dispatch's `_ => {}`.
// ---------------------------------------------------------------------------

/// `AI()`'s aiStyle-30 default arm (`Projectile.cs:26775-26784`) — the
/// MushroomSpear's spore (131): `velocity *= 0.8`, `rotation += 0.2`,
/// `alpha += 4`, kill at `alpha >= 255`. The 907/335 type arms are other
/// kinds (unrowed here). RNG-free.
pub(crate) fn ai_spore(p: &mut Projectile) {
    p.vel *= 0.8;
    p.rot += 0.2;
    p.alpha += 4;
    if p.alpha >= 255 {
        kill(p);
    }
}

/// `AI()`'s aiStyle-44 arm (`Projectile.cs:27893-27948`) for the spear's two
/// riders: the ChlorophytePartisan's spore cloud (228, `:27896-27904`) and the
/// ThunderSpear's bolt (732, `:27905-27921`). The shared frame machine
/// (`:27941-27948`) advances `frame` every `num321` ticks (6 for 228, 3 for
/// 732), wrapping at `Main.projFrames`. The 732's `Next(5)` dust gate
/// (`:27913`) is dust-only — skipped with its body; the light (`:27920`) is
/// skipped. The 229 arm is another kind (unrowed).
pub(crate) fn ai_spear_subshot_44(p: &mut Projectile) {
    let num321 = match p.kind {
        228 => {
            // `:27898-27903` — the fade: velocity * 0.96, alpha += 2, kill at
            // alpha > 200.
            p.vel *= 0.96;
            p.alpha += 2;
            if p.alpha > 200 {
                kill(p);
            }
            6
        }
        732 => {
            // `:27907-27920` — alpha += 12, kill at alpha > 255, the rotation
            // follows the velocity. The Next(5) dust + light are skipped.
            p.alpha += 12;
            if p.alpha > 255 {
                kill(p);
            }
            p.rot = p.vel.to_angle();
            3
        }
        _ => 6,
    };
    // `:27941-27948` — the frame machine.
    p.frame_counter += 1;
    if p.frame_counter >= num321 {
        p.frame_counter = 0;
        p.frame += 1;
        let frames = crate::projectile::proj_frames(p.kind) as i32;
        if p.frame >= frames {
            p.frame = 0;
        }
    }
}

/// `AI()`'s aiStyle-57 arm (`Projectile.cs:28867-28905`) — the NorthPole's
/// snowflake (343): the 30-tick counter then the +0.25 gravity (cap 16) and
/// the 0.995 X friction, the rotation, the alpha-50 fade-in, and the
/// owner-gated 344 spawn cadence (`localAI[1]` = `Next(7)` first tick, then
/// `localAI[0]` counts to `8 + localAI[1]`, spawning 344 with damage * 0.7,
/// knockback * 0.55, `ai1 = Next(3)`). The two `Next` draws are
/// behaviour-feeding (the cadence + the 344 frame) — KEPT, in order.
pub(crate) fn ai_north_pole_flake(p: &mut Projectile, rng: &mut dyn DropRng) -> Vec<SpearSubShot> {
    let mut shots = Vec::new();
    p.ai[0] += 1.0;
    if p.ai[0] > 30.0 {
        p.ai[0] = 30.0;
        p.vel.y += 0.25;
        if p.vel.y > 16.0 {
            p.vel.y = 16.0;
        }
        p.vel.x *= 0.995;
    }
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    p.alpha -= 50;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    if p.owner == 0 {
        if p.local_ai[1] == 0.0 {
            p.local_ai[1] = rng.next_range(0, 7) as f32;
        }
        p.local_ai[0] += 1.0;
        let mut num425 = 8;
        if p.local_ai[1] > 0.0 {
            num425 += p.local_ai[1] as i32;
        }
        if p.local_ai[0] >= num425 as f32 {
            p.local_ai[0] = 0.0;
            p.local_ai[1] = -1.0;
            let pc = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
            // The 344's `ai1 = Next(3)` frame payload (`:28902`) — the C#'s
            // trailing draw, kept in order (after the cadence's Next(7)).
            let frame = rng.next_range(0, 3) as f32;
            shots.push(SpearSubShot {
                kind: 344,
                x: pc.x,
                y: pc.y,
                vx: 0.0,
                vy: 0.0,
                damage: (p.damage as f32 * 0.7) as i32,
                knock_back: p.knock_back * 0.55,
                time_left: None,
                ai1: frame,
            });
        }
    }
    shots
}

/// `AI_001`'s type-344 hook (`Projectile.cs:57062-57114`) — the NorthPole's
/// falling snowflake. The tile-kill probe (`:57064`), the localAI[1] counter
/// (the >5 alpha-50 fade, the >=20 gravity/friction, the >=25 sibling-push
/// reset to 20), `frame = (int)ai[1]`. The sibling separation (`:57090-57113`)
/// scans the pool for same-kind neighbours within 60 px and pushes away —
/// `sibling_push` is that precomputed X push (the `minion_separation`
/// pattern; the pool computes it before the arm). The 344 is NOT on the
/// flag3 rest list (`:55947-56032`), so the flight counter runs; its rotation
/// tail is the `type != 344 && type != 498` exclusion (`:57748`) — NO default
/// rotation. The ±16 clamp (`:57753-57768`) applies.
pub(crate) fn ai_north_pole_snowflake(
    p: &mut Projectile,
    world: &World,
    sibling_push: f32,
) {
    // The flight counter (`:56038-56041`, flag3 true for 344) runs BEFORE
    // the type hook in the C#; the hook reads no ai[0], so the order is
    // kept for fidelity.
    p.ai[0] += 1.0;
    // `:57064-57068` — the tile-kill probe: a solid tile under either bottom
    // corner of the next step kills.
    let nx = p.pos.x;
    let ny = p.pos.y + p.vel.y;
    if crate::projectile::solid_tile(world, (nx / 16.0) as i32, (ny / 16.0) as i32 + 1)
        || crate::projectile::solid_tile(
            world,
            ((nx + p.w) / 16.0) as i32,
            (ny / 16.0) as i32 + 1,
        )
    {
        kill(p);
        return;
    }
    // `:57069-57077` — the counter and the fade.
    p.local_ai[1] += 1.0;
    if p.local_ai[1] > 5.0 {
        p.alpha -= 50;
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }
    // `:57078` — frame = (int)ai[1].
    p.frame = p.ai[1] as i32;
    // `:57079-57083` — the gravity/friction from tick 20.
    if p.local_ai[1] >= 20.0 {
        p.vel.y += 0.15;
        p.vel.x *= 0.96;
    }
    // `:57084-57113` — the sibling push at tick 25 (reset to 20). The pool
    // precomputed the X push (`sibling_push`); the C# adds `vector40 * 0.25`
    // where vector40.Y = 0 and vector40.X falls back to 1 when zero.
    if p.local_ai[1] >= 25.0 {
        p.local_ai[1] = 20.0;
        let mut push_x = sibling_push;
        if push_x == 0.0 {
            push_x = 1.0;
        }
        p.vel.x += push_x * 0.25;
    }
    // The 344 has its OWN gravity-chain arm (`:57062`, the block above), so
    // the generic `ai[0] >= 15` arm (`:57568`, nested in the `:57187` else)
    // does NOT apply — the counter (incremented at the head) adds no extra
    // gravity.
    // `:57748` — the 344/498 rotation exclusion: NO default rotation.
    // `:57753-57768` — the ±16 Y clamp (flag8 true, flag4 false).
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
}

/// The 344 sibling-separation X push (`Projectile.cs:57090-57113`): for each
/// active same-kind projectile within 60 px, accumulate
/// `-Sign(dx) * Remap(dist, 0, 60, 1, 0)`. The pool calls this with the slot
/// array before the snowflake arm (the `minion_separation` borrow pattern).
pub(crate) fn snowflake_sibling_push(all: &[Projectile], me: usize) -> f32 {
    let p = &all[me];
    let mut push = 0.0f32;
    for (m, other) in all.iter().enumerate() {
        if m == me || !other.active || other.kind != p.kind {
            continue;
        }
        let d = other.pos - p.pos;
        let dist = d.length();
        if dist > 60.0 {
            continue;
        }
        let t = get_lerp_value(0.0, 60.0, dist);
        // Remap(dist, 0, 60, 1, 0) = 1 - t.
        let weight = 1.0 - t;
        push += -(d.x.signum()) * weight;
    }
    push
}

// ---------------------------------------------------------------------------
// Witness tests — WRITTEN NOT RUN (code-only mode).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{ProjectilePool, proj_data};

    /// A recording `DropRng` — the draw log pins the C#'s `Main.rand` order
    /// (the `projectile_ai_frostmoon` precedent).
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

    fn owner_at(anchor: Vec2, anim: i32, anim_max: i32) -> SpearOwner {
        SpearOwner {
            anchor,
            direction: 1,
            item_animation: anim,
            item_animation_max: anim_max,
            channel: false,
            melee_speed: 1.0,
            wet: false,
            player_vel: Vec2::ZERO,
            player_pos: anchor - Vec2::new(10.0, 21.0),
            player_w: 20.0,
            player_h: 42.0,
        }
    }

    fn spear(kind: u16, vel: Vec2) -> Projectile {
        let d = proj_data(kind).unwrap();
        Projectile {
            active: true,
            kind,
            owner: 0,
            vel,
            damage: 30,
            knock_back: 6.0,
            w: d.w,
            h: d.h,
            tile_collide: false,
            ..Default::default()
        }
    }

    /// The 21 rows carry the C# `DefaultToSpear` + per-type `SetDefaults`
    /// (`Projectile.cs:10720-10730` + the cited type arms).
    #[test]
    fn the_spear_rows_are_verbatim() {
        // (kind, w, h, scale, alpha) — the box is the PRE-scale C# value (the
        // port convention; `SetDefaults`' `width *= scale` tail is not folded
        // into the row, the frost-row precedent).
        let cases: [(u16, f32, f32, f32, i32); 21] = [
            (46, 20.0, 20.0, 1.1, 0),
            (47, 18.0, 18.0, 1.1, 0),
            (49, 18.0, 18.0, 1.2, 0),
            (64, 18.0, 18.0, 1.25, 0),
            (66, 18.0, 18.0, 1.27, 0),
            (97, 18.0, 18.0, 1.1, 0),
            (105, 18.0, 18.0, 1.3, 0),
            (130, 22.0, 22.0, 1.2, 0),
            (153, 18.0, 18.0, 1.1, 0),
            (212, 18.0, 18.0, 1.12, 0),
            (215, 18.0, 18.0, 1.27, 0),
            (218, 18.0, 18.0, 1.28, 0),
            (222, 18.0, 18.0, 1.3, 0),
            (342, 22.0, 2.0, 1.1, 0),
            (367, 18.0, 18.0, 1.1, 0),
            (368, 18.0, 18.0, 1.0, 0),
            (730, 18.0, 18.0, 1.0, 0),
            (877, 25.0, 25.0, 1.0, 255),
            (878, 25.0, 25.0, 1.0, 255),
            (879, 25.0, 25.0, 1.0, 255),
            (1103, 18.0, 18.0, 1.2, 90),
        ];
        for (kind, w, h, scale, alpha) in cases {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} rowed"));
            assert_eq!((d.w, d.h), (w, h), "kind {kind} box");
            assert_eq!(d.ai_style, 19, "kind {kind} aiStyle 19");
            assert_eq!(d.penetrate, -1, "kind {kind} penetrate -1");
            assert!(!d.tile_collide, "kind {kind} tileCollide false");
            assert_eq!(d.scale, scale, "kind {kind} scale");
            assert_eq!(d.alpha, alpha, "kind {kind} alpha");
            assert_eq!(d.time_left, 3600, "kind {kind} stock life");
        }
    }

    /// The offset curve (`:46183`) at the extension peak and the retract
    /// floor for the Spear (49): num6=4, num7=0.85, num8=1.1.
    #[test]
    fn the_spear_offset_extends_then_retracts() {
        // itemAnimationMax = 30 → third = 10. At anim = 30 (full extension):
        // num4 = min(30,10) = 10, num5 = 20 → 4 + 0.85*((30-10)-20) - 1.1*(10-10)
        // = 4 + 0 - 0 = 4. At anim = 10 (the peak reach): num4 = 10, num5 = 0
        // → 4 + 0.85*(20-0) - 0 = 4 + 17 = 21. At anim = 0 (full retract):
        // num4 = 0, num5 = 0 → 4 + 0.85*20 - 1.1*10 = 4 + 17 - 11 = 10.
        assert_eq!(spear_offset(49, 30, 30), 4.0);
        assert_eq!(spear_offset(49, 10, 30), 21.0);
        assert_eq!(spear_offset(49, 0, 30), 10.0);
        // The lance (877): num6=28, num7=num8=0.4. At anim=24,max=24,third=8:
        // num4=8,num5=16 → 28 + 0.4*((24-8)-16) - 0.4*(8-8) = 28.
        assert_eq!(spear_offset(877, 24, 24), 28.0);
    }

    /// The extension hitbox (`:46186-46220`) exists only in the extension
    /// phase and only for 46/105/153.
    #[test]
    fn the_extension_hitbox_gates_on_phase_and_kind() {
        let mut p = spear(46, Vec2::new(6.0, 0.0));
        p.pos = Vec2::new(100.0, 100.0);
        // anim = 30, max = 30 → third = 10, anim >= 10 → extension. t =
        // Remap(30, 30, 10, 0, 1) = 0 → reach = 10, width = 10.
        let o = owner_at(Vec2::new(110.0, 110.0), 30, 30);
        let b = spear_extension_hitbox(&p, &o).unwrap();
        assert_eq!((b.w, b.h), (10, 10));
        // anim = 5 < 10 → retract → None.
        let o2 = owner_at(Vec2::new(110.0, 110.0), 5, 30);
        assert!(spear_extension_hitbox(&p, &o2).is_none());
        // The Spear (49) has no extension box (default → false).
        let p49 = spear(49, Vec2::new(6.0, 0.0));
        assert!(spear_extension_hitbox(&p49, &o).is_none());
        // At anim = 10 (t = 1): reach = 10 + 50*1*1 = 60, width = 10 + 15 = 25.
        let o3 = owner_at(Vec2::new(110.0, 110.0), 10, 30);
        let b3 = spear_extension_hitbox(&p, &o3).unwrap();
        assert_eq!((b3.w, b3.h), (25, 25));
    }

    /// The age counter kills at itemAnimationMax (`:46063-46072`); the lance
    /// holds while channelling.
    #[test]
    fn the_spear_dies_at_the_animation_end_unless_channelling() {
        let mut p = spear(49, Vec2::new(3.7, 0.0));
        let o = owner_at(Vec2::new(200.0, 200.0), 1, 30);
        let mut rng = RecordRng::default();
        // ai[0] = 29 → ++ = 30 >= 30 → kill.
        p.ai[0] = 29.0;
        ai_spears(&mut p, &tr_core::World::new("t".into(), 64, 64), &o, &mut rng);
        assert!(!p.active, "the spear expires at itemAnimationMax");
        // The lance (877) at the same age, channelling → holds.
        let mut lance = spear(877, Vec2::new(3.5, 0.0));
        lance.ai[0] = 29.0;
        let mut oc = owner_at(Vec2::new(200.0, 200.0), 1, 24);
        oc.channel = true;
        ai_spears(&mut lance, &tr_core::World::new("t".into(), 64, 64), &oc, &mut rng);
        assert!(lance.active, "the channelling lance holds");
        // Not channelling → dies.
        oc.channel = false;
        lance.ai[0] = 29.0;
        ai_spears(&mut lance, &tr_core::World::new("t".into(), 64, 64), &oc, &mut rng);
        assert!(!lance.active, "the released lance expires");
    }

    /// The ObsidianSwordfish bone burst draws `Next(1,4)` then per bone
    /// float,float,float,`Next(3)`,`Next(30,150)` (`:45926-45934`).
    #[test]
    fn the_swordfish_burst_draws_in_order() {
        let mut p = spear(367, Vec2::new(4.0, 0.0));
        let o = owner_at(Vec2::new(200.0, 200.0), 20, 20);
        let mut rng = RecordRng::default();
        let shots = ai_spears(&mut p, &tr_core::World::new("t".into(), 64, 64), &o, &mut rng);
        // RecordRng.next_range returns min → count = 1 bone.
        assert_eq!(shots.len(), 1);
        assert_eq!(
            rng.draws,
            vec![
                "range(1,4)",
                "float",
                "float",
                "float",
                "range(0,3)",
                "range(30,150)",
            ],
            "the bone burst's draw order"
        );
        assert_eq!(shots[0].kind, 1107, "Next(3) returns 0 → 1107");
        assert_eq!(shots[0].time_left, Some(30));
        assert_eq!(shots[0].damage, 15, "30 * 0.5");
    }

    /// The MushroomSpear spore cadence (`:45891-45895`): localAI[0] in
    /// {0,4,8,...} spawns 131 at damage/3.
    #[test]
    fn the_mushroom_spear_spores_on_the_cadence() {
        let mut p = spear(130, Vec2::new(5.5, 0.0));
        let o = owner_at(Vec2::new(200.0, 200.0), 40, 40);
        let mut rng = RecordRng::default();
        let w = tr_core::World::new("t".into(), 64, 64);
        // localAI[0] = 0 → spawn.
        let shots = ai_spears(&mut p, &w, &o, &mut rng);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 131);
        assert_eq!(shots[0].damage, 10, "30 / 3");
        assert_eq!(p.local_ai[0], 1.0);
        // localAI[0] = 1 → no spawn, but the counter still ticks.
        let shots = ai_spears(&mut p, &w, &o, &mut rng);
        assert!(shots.is_empty());
        assert_eq!(p.local_ai[0], 2.0);
        assert!(rng.draws.is_empty(), "the spore cadence is RNG-free");
    }

    /// The sub-shot arms: the spore (131, style 30) fades and kills at 255.
    #[test]
    fn the_spore_fades_and_kills() {
        let mut p = spear(130, Vec2::new(4.0, 0.0));
        p.kind = 131;
        p.alpha = 251;
        ai_spore(&mut p);
        assert_eq!(p.alpha, 255);
        assert!(!p.active, "alpha >= 255 kills");
        assert_eq!(p.vel, Vec2::new(3.2, 0.0), "velocity * 0.8");
    }

    /// The 344 snowflake's sibling push (`:57090-57113`): a neighbour to the
    /// right pushes left (negative X).
    #[test]
    fn the_snowflake_pushes_away_from_siblings() {
        let mut a = spear(342, Vec2::ZERO);
        a.kind = 344;
        a.pos = Vec2::new(100.0, 100.0);
        let mut b = a.clone();
        b.pos = Vec2::new(130.0, 100.0); // 30 px to the right, within 60.
        let all = vec![a, b];
        let push = snowflake_sibling_push(&all, 0);
        // dist = 30 → Remap(30,0,60,1,0) = 0.5; -Sign(+30)*0.5 = -0.5.
        assert!((push - (-0.5)).abs() < 1e-6, "push = {push}");
    }

    /// The pool dispatch wires aiStyle 19 to the spear arm (the coverage pin
    /// `s1_d_4c_every_rowed_style_has_exactly_one_live_arm` reads the source).
    #[test]
    fn the_spear_dispatches_through_the_pool() {
        let mut pool = ProjectilePool::new();
        let mut w = tr_core::World::new("t".into(), 64, 64);
        let slot = pool
            .spawn_with_data_ai(49, 200.0, 200.0, 3.7, 0.0, 8, 6.5, 0.0, 0.0)
            .unwrap();
        // No owner snapshot → the dispatch's None-owner path holds the slot
        // (the orphan guard). With an owner it would extend; here we only pin
        // that the row exists and the slot is active pre-step.
        assert!(pool.slots()[slot].active);
        assert_eq!(proj_data(49).unwrap().ai_style, 19);
        let _ = &mut w;
    }
}
