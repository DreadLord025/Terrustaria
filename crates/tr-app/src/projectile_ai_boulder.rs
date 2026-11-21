//! W4-d3 — the aiStyle-25 boulder family (`AI()`'s inline block
//! `Projectile.cs:26015-26415`, census `docs/research/26b`).
//!
//! The 13 kinds (`SetDefaults` `Projectile.cs:1833-1870`, `:7038-7047`,
//! `:9918-9937`): 99 Boulder / 727 RollingCactus / 1013 BouncyBoulder /
//! 1014 LifeCrystalBoulder / 1053 LavaBoulder / 1054 SpiderBoulder /
//! 1055 Ghoulder / 1057 BoulderThatSpawnsPet / 1005 MiniBoulder /
//! 1021 MoonBoulder / 655 BeeHive / 1047 RainbowBoulder / 1048 Poulder.
//! The 727's bounce children (763 RollingCactusSpike, aiStyle 1,
//! `:7855-7865`) ride the generic arrow flight — their row lands in
//! `projectile/data.rs` with this module.
//!
//! Ported here, in the C#'s own order:
//! * the AI arm ([`boulder_step`]'s head half, `:26015-26415`): the
//!   `localAI[2]` tick, the 1013/1048 age, the 1014 `Main.tileFrame[665]`
//!   frame pin, the 1053/1048 dust gates (drawn, bodies skipped), the 1054
//!   cobweb trail (`WorldGen.PlaceTile(51)`), the 1047 frame machine +
//!   `tileCollide` pin + smart-bounce gates, the 1055 rotation wrap +
//!   player-chase flight, the wall-nudge ladder (`:26248-26331`), the
//!   rotation/gravity/accel tail and the wet kills (`:26400-26414`);
//! * the Shimmer anti-gravity arm (`Shimmer()` `:22504-22515`, called from
//!   `Update` `:17009-17012` — the style-25/17 exemption at `:22363` means
//!   even the hostile boulders float);
//! * `HandleMovement`'s collision + per-kind bounce ladder
//!   (`:18941-19277`) with the slope pass (`:17656-17698`/`:17744-17845`/
//!   `:17850-17892`) and the blocked-move tile-cut prologue
//!   (`:17910-17984`, the 484 cut + the `Next(3)` cracked-brick cut);
//! * the RainbowBoulder smart-bounce machine (`:20024-20315`):
//!   PreventSpam (the pool precomputes the sibling count — the
//!   `snowflake_sibling_push` pattern), HighlightNearbyTreasure, the
//!   enemy/treasure/player target pick and the intercept-solve jump
//!   (`Utils.GetChaseResults` + `FactorAcceleration`);
//! * the `Kill()` death hooks ([`boulder_kill`], pool-side): the getGood
//!   BoulderExplosion (`:70563-70575`), the 1047 rainbow burst
//!   (`:71192-71255`), the family arm (`:74841-74968`, the item drops, the
//!   Poulder poop pyramid and the LavaBoulderExplosion), the 1005 arm
//!   (`:74970-74981`) and the 655 bee/honey arm (`:75080-75123`).
//!
//! RNG (the `UpdateProjectiles` stream): every EXPLICIT `Main.rand` call in
//! the ported C# blocks stays, in order — the dust gates, the spike-burst
//! count/seed, the smart-bounce `Next(4)`/`Next(180)` gates, the particle
//! settings' `NextVector2*` draws, the kill-arm `Next(2)` scale gates, the
//! gore type picks + `NewGore` kicks (the G3 census, `Gore.cs:1327-1328`),
//! the bee count/velocity rolls. SKIPPED with their layers: the
//! `Dust.NewDust`/`CloneDust` internal rolls (the
//! `rainbow_crystal_explosion_draws` precedent), the
//! `ParticleOrchestrator.RequestParticleSpawn` bodies (the shadowbeam
//! precedent `projectile.rs:8543` — the port has no particle orchestration,
//! `app/netmodules.rs:448`), `Collision.HitTiles` (effect-only dust,
//! `Collision.cs:3103`), `Lighting.AddLight`/`CastLightOpen` and the sounds
//! (parked on `wants_sound`; the id-3/4/42 cues the port's `play_proj_sound`
//! façade does not serve drop at the drain — the sound-layer census).
//!
//! Documented deviations (ANALYSIS "The boulder family (W4-d3)"):
//! * the wet flags probe at the arm head (the bobber tick-head convention,
//!   one-tick lag vs the C#'s post-AI `Update` probe `:16790-17007`);
//! * the 1047 trail ring is the port's fixed 10 entries vs the C#'s
//!   `TrailCacheLength[1047] = 25` (`ProjectileID.cs:522`) — the kill-arm
//!   draw census walks the port ring (the 640 moonlord-trail precedent);
//! * the 1054 spider spawn (`Kill_SpiderBoulder_SpawnSpider`
//!   `:70618-70639`) drops: the WallCreeper 164/165 are unported NPC kinds
//!   (the statue precedent); the arm's own reads are RNG-free;
//! * the bee-hive slot cap folds `GetAvailableAmountOfNPCsToSpawnUpToSlot`
//!   (`NPC.cs:80635-80654`) onto `100 - npcs.len()` (the port's Vec roster
//!   has no 200-slot array);
//! * the player-contact lane ([`player_contact`]) is the `damage_player`
//!   stand-in without the single-hit kill (the boulders are penetrate −1;
//!   `Damage_EVP` `:14773-14900` never kills the shot) — the ×2 /
//!   hostileDamageScaling / deadMansSweater modifiers ride the port's
//!   existing hurt lane unported, like `damage_player`;
//! * the 1057/1014/655 item drops park on the pool and drain on the
//!   `UpdateItems` stream (the bobber-catch precedent); the C# rolls them
//!   inline under `UpdateProjectiles` — the stack rolls themselves stay on
//!   this arm's stream.
//!
//! CODE-ONLY MODE: the witnesses below are WRITTEN NOT RUN (no compiler
//! invoked on this branch); every expected value is hand-derived from the
//! cited C# lines.

use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::{
    can_hit, chaseable, get_chase_results, npc_center, proj_data, remap, safe_normalize_default,
    solid_collision, solid_tile, Projectile, BEE_HIVE, BOUNCY_BOULDER, BOULDER,
    BOULDER_THAT_SPAWNS_PET, GHOULDER, LIFE_CRYSTAL_BOULDER, LAVA_BOULDER, MINI_BOULDER,
    MOON_BOULDER, POULDER, RAINBOW_BOULDER, ROLLING_CACTUS, ROLLING_CACTUS_SPIKE, SPIDER_BOULDER,
};
use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::tiledata as td;
use tr_core::world::World;

/// The 13 aiStyle-25 kinds (the census 26b row set).
pub(crate) const BOULDER_KINDS: [u16; 13] = [
    BOULDER,
    ROLLING_CACTUS,
    BOUNCY_BOULDER,
    LIFE_CRYSTAL_BOULDER,
    LAVA_BOULDER,
    SPIDER_BOULDER,
    GHOULDER,
    BOULDER_THAT_SPAWNS_PET,
    MINI_BOULDER,
    MOON_BOULDER,
    BEE_HIVE,
    RAINBOW_BOULDER,
    POULDER,
];

/// The kinds whose `SetDefaults` sets `hostile = true` (`:1839`/`:1857`/
/// `:1866`/`:7044`/`:9934`) — every boulder EXCEPT the friendly-only 1047
/// (`:9918-9927`), plus the 727's spike child (`:7859`). The pool's
/// player-contact lane reads this set (the `Damage_EVP` `hostile` gate
/// `:14775`); the generic `!friendly` proxy cannot serve the friendly+hostile
/// traps (the dungeon-debris precedent, inverted).
pub(crate) fn hurts_player(kind: u16) -> bool {
    (BOULDER_KINDS.contains(&kind) && kind != RAINBOW_BOULDER)
        || kind == ROLLING_CACTUS_SPIKE
}

/// One 727 bounce spike (`Projectile.cs:19103`/`:19121`): the resolved
/// `NewProjectile(Center, vector, 763, 30, 2f, myPlayer)` payload. The pool
/// spawns it through `spawn_with_data_ai_owned` with the corpse-flag lift
/// (the electrosphere precedent — the X-block arm kills BEFORE the burst).
pub(crate) struct SpikeSpawn {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// One `Collision.SwitchTiles` request the arm parks (the pool cannot borrow
/// the wiring): `(pos_x, pos_y, old_x, old_y, w, h, obj_type)` — the
/// `:17089` objType-3 plate scan and the `:19863` objType-4 pad call. The
/// world sim drains them through `crate::collision::switch_tiles` (the
/// `world_sim/npcs.rs:4654` precedent).
pub(crate) type SwitchRequest = (f32, f32, f32, f32, i32, i32, i32);

/// The 12 kinds the `:17087` objType-3 `SwitchTiles` gate names (every
/// boulder EXCEPT the 1005 MiniBoulder; the 763 spike is aiStyle 1).
fn plate_scan_kind(kind: u16) -> bool {
    BOULDER_KINDS.contains(&kind) && kind != MINI_BOULDER
}

/// `ProjectileID.Sets.ForcePlateDetection` (`ProjectileID.cs:434-520`): the
/// 11 boulder kinds carry an explicit `false` — `flag4 = true` suppresses
/// the `:19859` pad call for them. The 1005 (and the 763) are unlisted
/// (`null`): `flag4 = false`, the pad call may fire.
fn force_plate_detection_false(kind: u16) -> bool {
    matches!(
        kind,
        BOULDER | ROLLING_CACTUS | BEE_HIVE | BOUNCY_BOULDER | LIFE_CRYSTAL_BOULDER
            | RAINBOW_BOULDER | POULDER | LAVA_BOULDER | SPIDER_BOULDER | GHOULDER
            | BOULDER_THAT_SPAWNS_PET
    )
}

/// `Entity.Center` for the slot.
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `Utils.CenteredRectangle(center, size)` (`Utils.cs:947-950`) as the
/// `(left, top, right, bottom)` inclusive-exclusive box the scans walk.
fn centered_rect(c: Vec2, size: Vec2) -> (i32, i32, i32, i32) {
    let x = (c.x - size.x / 2.0) as i32;
    let y = (c.y - size.y / 2.0) as i32;
    (x, y, x + size.x as i32, y + size.y as i32)
}

/// `Rectangle.Intersects` — the strict XNA overlap (touching edges do not
/// intersect), on the `(left, top, right, bottom)` form above.
fn rect_hits_box(r: (i32, i32, i32, i32), x: f32, y: f32, w: f32, h: f32) -> bool {
    (r.0 as f32) < x + w && (r.2 as f32) > x && (r.1 as f32) < y + h && (r.3 as f32) > y
}

/// The single-player `Player.FindClosest` fold (`Player.cs:4942-4967`): the
/// port has one player, so the scan resolves to it. `None` reads the C#'s
/// inactive-player-0 stand-in (position 0, the stock 20×42 box, direction 0
/// — the orphan read the pool-only tests exercise).
struct ClosestPlayer {
    center: Vec2,
    left: f32,
    right: f32,
    direction: i32,
}

fn closest_player(player: Option<&Player>) -> ClosestPlayer {
    match player {
        Some(pl) => {
            let (cx, cy) = pl.center();
            ClosestPlayer {
                center: Vec2::new(cx, cy),
                left: pl.x,
                right: pl.x + pl.width,
                direction: pl.facing,
            }
        }
        None => ClosestPlayer {
            center: Vec2::new(10.0, 21.0),
            left: 0.0,
            right: 20.0,
            direction: 0,
        },
    }
}

/// `RainbowBoulder_GetAreas` (`Projectile.cs:20193-20199`): the closest
/// player, the 1920×1200 play area (`Main.MaxWorldViewSize`,
/// `Main.cs:677-679`) and the 300×150 close-to-me box.
fn rainbow_areas(
    p: &Projectile,
    player: Option<&Player>,
) -> (ClosestPlayer, (i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let cp = closest_player(player);
    let play = centered_rect(cp.center, Vec2::new(1920.0, 1200.0));
    let close = centered_rect(center(p), Vec2::new(300.0, 150.0));
    (cp, play, close)
}

/// `Utils.FactorAcceleration` (`Utils.cs:705-709`):
/// `velocity - descend * max(0, t - lenience)^2 / 2 / t`.
fn factor_acceleration(vel: Vec2, time: f32, descend: Vec2, lenience: i32) -> Vec2 {
    let num = (time - lenience as f32).max(0.0);
    vel - descend * (num * num) / 2.0 / time
}

/// `RainbowBoulder_PreventSpam` (`Projectile.cs:20024-20048`): the pool
/// precomputes the count (the sibling-scan pattern — the count cannot
/// change inside this slot's sub-step: only THIS slot dies, and it is
/// excluded). `>= 7` same-owner same-kind boulders inside the 6000×6000
/// centre box kills the shot. Returns `true` when the slot survives.
pub(crate) fn rainbow_spam_survives(siblings: &[Projectile], me: usize) -> bool {
    let p = &siblings[me];
    let (l, t, r, b) = centered_rect(center(p), Vec2::new(6000.0, 6000.0));
    let mut count = 0;
    for (i, q) in siblings.iter().enumerate() {
        if count >= 7 {
            break;
        }
        if i != me
            && q.active
            && q.owner == p.owner
            && q.kind == p.kind
            && rect_hits_box((l, t, r, b), q.pos.x, q.pos.y, q.w, q.h)
        {
            count += 1;
        }
    }
    count < 7
}

/// `RainbowBoulder_HighlightNearbyTreasure` (`Projectile.cs:20164-20191`):
/// the 5×5 tile box around the centre tile; every lure-able ore-finder tile
/// draws the particle settings' `NextVector2Circular(16, 16)` pair (the
/// RainbowBoulder1 body is the skipped particle layer) and skips the next
/// row (`j++`).
fn rainbow_highlight_treasure(p: &Projectile, world: &World, rng: &mut dyn DropRng) {
    let c = center(p);
    let (l, t, r, b) = centered_rect(
        Vec2::new((c.x / 16.0) as i32 as f32, (c.y / 16.0) as i32 as f32),
        Vec2::new(5.0, 5.0),
    );
    // The C# loops are `i <= rectangle.Right` INCLUSIVE (Right = X + Width
    // — a 6-column walk for the 5-wide box, a verbatim quirk).
    for i in l..=r {
        let mut j = t;
        while j <= b {
            // `WorldGen.InWorld(i, j, 2)`.
            if i < 2 || j < 2 || i >= world.width - 2 || j >= world.height - 2 {
                j += 1;
                continue;
            }
            let lure = world.tile(i, j).is_some_and(|tile| {
                tile.active()
                    && td::SET_FRIENDLY_FAIRY_CAN_LURE_TO.has(tile.kind)
                    && tr_core::scene::is_valid_for_ore_finder(tile)
            });
            if lure {
                // `:20178` — the settings' circular pair (2 floats).
                let _ = (rng.next_float(), rng.next_float());
                // The RainbowBoulder1 body (`ParticleOrchestrator.cs:1502`)
                // is the particle layer — skipped.
                j += 1; // `:20187`
            }
            j += 1;
        }
    }
}

/// `RainbowBoulder_GetTreasureCoords` (`Projectile.cs:20258-20315`): the
/// play-area tile scan (the 40-tile world-margin clamp), the ore-finder
/// priority pick with the 7×7 same-kind `>= 40` ore-cluster gate; ties go
/// to the strictly closer tile. Returns the treasure tile.
fn rainbow_treasure_coords(
    p: &Projectile,
    world: &World,
    play: (i32, i32, i32, i32),
) -> Option<(i32, i32)> {
    // `:20249-20253` — the play area /16 (C# int division), intersected
    // with the world box inflated −40.
    let mut l = play.0 / 16;
    let mut t = play.1 / 16;
    let mut r = play.2 / 16;
    let mut b = play.3 / 16;
    l = l.max(40);
    t = t.max(40);
    r = r.min(world.width - 40);
    b = b.min(world.height - 40);
    let c = center(p);
    let mut best_priority: i16 = -1;
    let mut best_dist = -1.0f32;
    let mut coords = None;
    // The C# scan is `i <= rectangle.Right` inclusive (the highlight quirk).
    for i in l..=r {
        for j in t..=b {
            let Some(tile) = world.tile(i, j) else { continue };
            if !tile.active()
                || !td::SET_FRIENDLY_FAIRY_CAN_LURE_TO.has(tile.kind)
                || !tr_core::scene::is_valid_for_ore_finder(tile)
            {
                continue;
            }
            let mut priority = td::ORE_FINDER_PRIORITY[tile.kind as usize];
            if td::SET_ORE.has(tile.kind) {
                // `:20282-20299` — the 7×7 same-kind count; < 40 demotes.
                let mut count = 0;
                for k in (i - 3)..=(i + 3) {
                    for m in (j - 3)..=(j + 3) {
                        if world
                            .tile(k, m)
                            .is_some_and(|q| q.active() && q.kind == tile.kind)
                        {
                            count += 1;
                        }
                    }
                }
                if count < 40 {
                    priority = -1;
                }
            }
            if best_priority <= priority {
                let dist = c.distance(Vec2::new(i as f32 * 16.0 + 8.0, j as f32 * 16.0 + 8.0));
                if best_priority != priority || dist < best_dist {
                    best_priority = priority;
                    best_dist = dist;
                    coords = Some((i, j));
                }
            }
        }
    }
    // `:20314` — `num2 != -1`.
    if best_priority != -1 {
        coords
    } else {
        None
    }
}

/// `RainbowBoulder_JumpToPosition` (`Projectile.cs:20222-20256`): the
/// distance-remapped launch speed, the 8..32 intercept-solve sweep (the LAST
/// successful solve wins — the C# loop has no break) and the
/// `FactorAcceleration` gravity lead; `ai[1]` carries the flight time.
fn rainbow_jump_to(p: &mut Projectile, target: Vec2, target_vel: Vec2, local_owner: u8) {
    let c = center(p);
    let v = target - c;
    let speed = remap(v.length(), 0.0, 200.0, 4.0, 32.0);
    let mut flight = v.length() / speed;
    let descend = Vec2::new(0.0, 0.6);
    p.vel = safe_normalize_default(v, Vec2::new(0.0, -1.0)) * speed;
    // `:20230-20237` — the owner-gated RainbowBoulder4 particle: no settings
    // draws, the body is the skipped layer.
    let mut solved = false;
    let mut s = 8.0f32;
    while s <= 32.0 {
        if let Some(chase) = get_chase_results(c, s, target, target_vel) {
            p.vel = factor_acceleration(chase.chaser_velocity, chase.interception_time, descend, 0);
            flight = (chase.interception_time - 2.0) as i32 as f32;
            solved = true;
        }
        s += 2.0;
    }
    if !solved {
        p.vel = factor_acceleration(p.vel, flight.min(120.0), descend, 0);
    }
    p.ai[1] = flight;
    p.net_update = true;
    let _ = local_owner; // the particle gate's only read (body skipped)
}

/// `RainbowBoulder_PerformSmartBounce` (`Projectile.cs:20112-20162`): the
/// spam guard, the treasure highlight, then the target pick — the FARTHEST
/// chaseable NPC in the play area (the C#'s `num <= num3` keeps the max, a
/// verbatim quirk), else the treasure tile, else the player within 3000 px.
fn rainbow_smart_bounce(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    player: Option<&Player>,
    rng: &mut dyn DropRng,
    local_owner: u8,
    spam_survives: bool,
) {
    // `:20114-20118` — PreventSpam kills and the `!active` return.
    if !spam_survives {
        p.kill_now();
        return;
    }
    rainbow_highlight_treasure(p, world, rng);
    let (cp, play, close) = rainbow_areas(p, player);
    let mut farthest: Option<Vec2> = None;
    let mut far_dist = f32::INFINITY;
    let mut near_outside_dist = f32::INFINITY;
    let mut near_outside = false;
    for n in npcs {
        if !chaseable(n) {
            continue;
        }
        let (nw, nh) = n.size();
        if !rect_hits_box(play, n.x, n.y, nw, nh) {
            continue;
        }
        let nc = npc_center(n);
        let dist = cp.center.distance(nc);
        // `:20133` — `if (!(num <= num3))`: strictly greater replaces.
        if far_dist <= dist {
            continue;
        }
        far_dist = dist;
        farthest = Some(nc);
        if !rect_hits_box(close, n.x, n.y, nw, nh) && near_outside_dist > dist {
            near_outside_dist = dist;
            near_outside = true;
        }
    }
    let _ = near_outside; // nPC2 only gates the branch; the jump reads nPC.
    if let Some(nc) = farthest {
        // `RainbowBoulder_JumpToEnemy` (`:20206-20209`): the NPC's velocity
        // is the intercept lead — the port's Npc carries (vx, vy).
        let lead = npcs
            .iter()
            .find(|n| npc_center(n) == nc)
            .map(|n| Vec2::new(n.vx, n.vy))
            .unwrap_or(Vec2::ZERO);
        rainbow_jump_to(p, nc, lead, local_owner);
        return;
    }
    if let Some((tx, ty)) = rainbow_treasure_coords(p, world, play) {
        // `Point.ToWorldCoordinates()` = (x*16+8, y*16+8).
        rainbow_jump_to(
            p,
            Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0),
            Vec2::ZERO,
            local_owner,
        );
    } else if center(p).distance(cp.center) < 3000.0 {
        // `RainbowBoulder_JumpToPlayer` (`:20201-20204`): the player's
        // POSITION (not velocity) is the C# lead argument — a verbatim
        // quirk (`player.position` passed as targetVelocity).
        let lead = player
            .map(|pl| Vec2::new(pl.x, pl.y))
            .unwrap_or(Vec2::ZERO);
        rainbow_jump_to(p, cp.center, lead, local_owner);
    }
}

/// `RainbowBoulder_CheckBounce` (`Projectile.cs:20050-20110`): the
/// 0.95/0.5/0.25/0.1 speed-band bounce, the `localAI[1]` hard-hit counter
/// and the smart bounce on ANY blocked axis.
fn rainbow_check_bounce(
    p: &mut Projectile,
    last: Vec2,
    world: &World,
    npcs: &[Npc],
    player: Option<&Player>,
    rng: &mut dyn DropRng,
    local_owner: u8,
    spam_survives: bool,
) {
    let mut fx = 0.95f32;
    let mut fy = 0.95f32;
    if last.x.abs() < 0.5 {
        fx = 0.1;
    } else if last.x.abs() < 0.75 {
        fx = 0.25;
    } else if last.x.abs() < 1.0 {
        fx = 0.5;
    }
    if last.y.abs() < 0.5 {
        fy = 0.1;
    } else if last.y.abs() < 0.75 {
        fy = 0.25;
    } else if last.y.abs() < 1.0 {
        fy = 0.5;
    }
    let mut hard = false;
    let mut bounced = false;
    if p.vel.y != last.y {
        if last.y.abs() > 5.0 {
            hard = true;
        }
        p.vel.y = -last.y * fy;
        bounced = true;
    }
    if p.vel.x != last.x {
        if last.x.abs() > 5.0 {
            hard = true;
        }
        p.vel.x = -last.x * fx;
        bounced = true;
    }
    if hard {
        p.local_ai[1] += 1.0;
        // HitTiles (effect-only dust) skipped; SoundID.RainbowBoulder.
        p.wants_sound = Some((42, 274, p.pos.x, p.pos.y, 0.0));
    }
    if bounced {
        rainbow_smart_bounce(p, world, npcs, player, rng, local_owner, spam_survives);
    }
}

/// The 727's spike burst (`Projectile.cs:19092-19105`/`:19110-19123`): the
/// owner-gated `Next(3, 6)` count + `NextFloat` seed, then the even ring —
/// downward halves flip ×−0.7. The C# float-step loop (`num50 += 1/count`)
/// keeps its own accumulation.
fn cactus_spike_burst(p: &Projectile, rng: &mut dyn DropRng, local_owner: u8) -> Vec<SpikeSpawn> {
    if p.owner != local_owner {
        return Vec::new();
    }
    let count = rng.next_range(3, 6);
    let seed = rng.next_float();
    let c = center(p);
    let mut out = Vec::new();
    let mut t = 0.0f32;
    while t < 1.0 {
        let angle = (seed + t) * std::f32::consts::TAU;
        let mut v = Vec2::from_angle(angle) * 2.0;
        if v.y > 0.0 {
            v *= -0.7;
        }
        out.push(SpikeSpawn { x: c.x, y: c.y, vx: v.x, vy: v.y });
        t += 1.0 / count as f32;
    }
    out
}

/// The `:16740-16764` StepUp draw-lift decay (the C# runs it at the
/// sub-step head, ahead of the AI — the AI_026 slice-C order).
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

/// `Shimmer()`'s aiStyle-25/17 arm (`Projectile.cs:22504-22515`): the
/// anti-gravity float. The `:22363` head gate exempts styles 25/17 from the
/// hostile refusal, so EVERY boulder floats in shimmer.
fn shimmer_step(p: &mut Projectile) {
    if p.vel.y > 10.0 {
        p.vel.y *= 0.97;
    }
    p.vel.y -= 0.7;
    if p.vel.y < -10.0 {
        p.vel.y = -10.0;
    }
}

/// The wall-nudge ladder (`Projectile.cs:26248-26331`): a resting boulder
/// (`ai[0] != 0`, `vel.Y <= 0`, `!flag9`, `vel.X == 0`) probes ±8/±24/±40 px
/// walls and rolls away at 0.5; with no wall the centre-tile parity picks
/// the direction.
fn wall_nudge(p: &mut Projectile, world: &World) {
    let speed = 0.5f32;
    for margin in [8.0f32, 24.0, 40.0] {
        let mut ty = (p.pos.y / 16.0) as i32;
        let left_x = ((p.pos.x - margin) / 16.0) as i32;
        let left = solid_tile(world, left_x, ty) || solid_tile(world, left_x, ty + 1);
        let right_x = ((p.pos.x + p.w + margin) / 16.0) as i32;
        ty = (p.pos.y / 16.0) as i32;
        let right = solid_tile(world, right_x, ty) || solid_tile(world, right_x, ty + 1);
        if left {
            p.vel.x = speed;
            return;
        }
        if right {
            p.vel.x = -speed;
            return;
        }
        // The third rung's no-wall fallback (`:26310-26320`).
        if margin == 40.0 {
            if ((center(p).x / 16.0) as i32) % 2 == 0 {
                p.vel.x = speed;
            } else {
                p.vel.x = -speed;
            }
            return;
        }
    }
}

/// The AI arm (`Projectile.cs:26015-26415`). `tile_frame_665` is the app's
/// `Main.tileFrame[665]` (the pool setter feeds it — the `set_disco`
/// pattern); `spam_survives` is the pool's PreventSpam precompute.
#[allow(clippy::too_many_arguments)]
fn ai_boulder(
    p: &mut Projectile,
    world: &mut World,
    npcs: &[Npc],
    player: Option<&Player>,
    rng: &mut dyn DropRng,
    local_owner: u8,
    tile_frame_665: i32,
    spam_survives: bool,
) {
    // `:26017` — the localAI[2] tick. The `:26018-26021` netMode-2 netUpdate
    // cadence is the MP layer (the port simulates netMode 0) — skipped.
    p.local_ai[2] += 1.0;
    // `:26022-26025` — the 1013/1048 age.
    if p.kind == BOUNCY_BOULDER || p.kind == POULDER {
        p.local_ai[0] += 1.0;
    }
    // `:26026-26029` — the 1014 frame rides the life-crystal tile animation.
    if p.kind == LIFE_CRYSTAL_BOULDER {
        p.frame = tile_frame_665;
    }
    let mut flag9 = false;
    // `:26031-26049` — the 1053 ember dust: three `Next(3)` gates DRAWN,
    // the dust bodies skipped.
    if p.kind == LAVA_BOULDER {
        let _ = rng.next_range(0, 3);
        let _ = rng.next_range(0, 3);
        let _ = rng.next_range(0, 3);
    }
    // `:26050-26066` — the 1054 cobweb trail (netMode != 1 folds true): the
    // box's tile walk, `!active && liquid == 0` then the `Next(7)` gate,
    // `WorldGen.PlaceTile(51, mute)`.
    if p.kind == SPIDER_BOULDER {
        let x0 = ((p.pos.x + 1.0) / 16.0) as i32;
        let x1 = ((p.pos.x + p.w) / 16.0) as i32;
        let y0 = ((p.pos.y + 1.0) / 16.0) as i32;
        let y1 = ((p.pos.y + p.h) / 16.0) as i32;
        for tx in x0..=x1 {
            for ty in y0..=y1 {
                let empty = world
                    .tile(tx, ty)
                    .is_some_and(|t| !t.active() && t.liquid == 0);
                if empty && rng.next_range(0, 7) == 0 {
                    let _ = tr_core::edit::place_tile(world, tx, ty, 51);
                }
            }
        }
    }
    // `:26067-26116` — the 1047 RainbowBoulder block.
    if p.kind == RAINBOW_BOULDER {
        // `:26069` — the hslToRgb hue: no draw. `:26070-26077` — the
        // 2-tick 100-frame wrap.
        p.frame_counter += 1;
        if p.frame_counter >= 2 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 100 {
                p.frame = 0;
            }
        }
        // `:26078-26086` — the oldPos light cast: the lighting layer, no
        // draws — skipped.
        // `:26087` — the tileCollide pin.
        p.tile_collide = p.ai[1] == 0.0 && !solid_collision(world, p.pos, p.w, p.h);
        if p.ai[1] > 0.0 {
            // `:26088-26095` — the flight countdown; flag9 = !tileCollide.
            flag9 = !p.tile_collide;
            p.ai[1] -= 1.0;
        } else {
            // `:26098-26105` — the Next(4) out-of-play-area gate.
            if rng.next_range(0, 4) == 0 {
                let (_, play, _) = rainbow_areas(p, player);
                if !rect_hits_box(play, p.pos.x, p.pos.y, p.w, p.h) {
                    rainbow_smart_bounce(p, world, npcs, player, rng, local_owner, spam_survives);
                }
            }
            // `:26106-26109` — the owner-gated Next(180) wander (the C#
            // `&&` draws only for the local owner).
            if p.owner == local_owner && rng.next_range(0, 180) == 0 {
                rainbow_smart_bounce(p, world, npcs, player, rng, local_owner, spam_survives);
            }
        }
        // `:26111-26115` — the ShimmerBlock particle settings'
        // `NextVector2CircularEdge(w/2, h/2)` (1 float); the body skipped.
        let _ = rng.next_float();
    }
    // `:26117-26247` — the 1055 Ghoulder block.
    if p.kind == GHOULDER {
        // `:26119-26126` — the ±2π rotation wrap.
        if p.rot < -std::f32::consts::TAU {
            p.rot += std::f32::consts::TAU;
        } else if p.rot > std::f32::consts::TAU {
            p.rot -= std::f32::consts::TAU;
        }
        if p.ai[1] == 1.0 {
            flag9 = true;
            let cp = closest_player(player);
            // `:26132-26146` — the overlap-or-facing chase gate.
            let mut chase = false;
            if p.pos.x + p.w > cp.left && p.pos.x < cp.right {
                chase = true;
            } else if center(p).x < cp.center.x {
                if cp.direction > 0 {
                    chase = true;
                }
            } else if cp.direction < 0 {
                chase = true;
            }
            if chase {
                // `:26149-26156` — the localAI[0] ramp to 84.
                if p.local_ai[0] < 84.0 {
                    p.local_ai[0] += 3.0;
                } else {
                    p.local_ai[0] = 84.0;
                }
                // `:26157-26179` — the frame walk toward 7 at >3 cadence.
                if p.frame < 7 {
                    p.frame_counter += 1;
                    if p.frame_counter > 3 {
                        p.frame += 1;
                        p.frame_counter = 0;
                    }
                } else if p.frame > 7 {
                    p.frame_counter += 1;
                    if p.frame_counter > 3 {
                        p.frame -= 1;
                        p.frame_counter = 0;
                    }
                } else {
                    p.frame = 7;
                    p.frame_counter = 0;
                }
                // `:26180-26195` — the flight: no tiles, the rotation
                // decay, the facing, the (vel*35 + toward*6)/36 blend.
                p.tile_collide = false;
                p.rot *= 0.6;
                if p.vel.x < 0.0 {
                    p.sprite_direction = -1;
                } else if p.vel.x > 0.0 {
                    p.sprite_direction = 1;
                }
                let mut toward = cp.center - center(p);
                // `Vector2.Normalize()` on a zero vector leaves NaN in XNA;
                // the port's normalize_or_zero is the safe stand-in (the
                // coincident-centre case never steers).
                toward = toward.normalize_or_zero();
                toward *= 6.0;
                p.vel = (p.vel * 35.0 + toward) / 36.0;
            } else {
                // `:26199-26239` — the rest: the ramp down, the frame walk
                // (the >= 9 wrap to 4), tiles on, the 0.92 drag + 0.3 fall.
                if p.local_ai[0] > 0.0 {
                    p.local_ai[0] -= 3.0;
                } else {
                    p.local_ai[0] = 0.0;
                }
                if p.frame >= 7 {
                    p.frame_counter += 1;
                    if p.frame_counter > 3 {
                        if p.frame >= 9 {
                            p.frame = 4;
                        } else {
                            p.frame += 1;
                        }
                        p.frame_counter = 0;
                    }
                } else if p.frame > 0 {
                    p.frame_counter += 1;
                    if p.frame_counter > 3 {
                        p.frame -= 1;
                        p.frame_counter = 0;
                    }
                } else {
                    p.frame = 0;
                    p.frame_counter = 0;
                }
                p.tile_collide = true;
                p.vel.x *= 0.92;
                p.vel.y += 0.3;
            }
        } else {
            // `:26242-26246` — the rolling rest.
            p.rot += p.vel.x * 0.06;
            p.frame = 0;
        }
    }
    // `:26248-26331` — the wall-nudge ladder.
    if p.ai[0] != 0.0 && p.vel.y <= 0.0 && !flag9 && p.vel.x == 0.0 {
        wall_nudge(p, world);
    }
    // `:26332-26335` — the shared roll (the 1055 owns its rotation above).
    if p.kind != GHOULDER {
        p.rot += p.vel.x * 0.06;
    }
    // `:26336` — the first-tick latch.
    p.ai[0] = 1.0;
    // `:26337-26340` — the 16 px/tick fall cap.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `:26341-26379` — the per-kind X accel.
    if p.kind == GHOULDER {
        if p.ai[1] == 0.0 {
            if p.vel.x > 0.0 && p.vel.x < 7.0 {
                p.vel.x += 0.05;
            }
            if p.vel.x < 0.0 && p.vel.x > -7.0 {
                p.vel.x -= 0.05;
            }
        }
    } else if p.kind == MOON_BOULDER {
        if p.vel.y.abs() <= 1.0 {
            if p.vel.x > 0.0 && p.vel.x < 3.5 {
                p.vel.x += 0.025;
            }
            if p.vel.x < 0.0 && p.vel.x > -3.5 {
                p.vel.x -= 0.025;
            }
        }
    } else if p.vel.y <= 6.0 {
        if p.vel.x > 0.0 && p.vel.x < 7.0 {
            p.vel.x += 0.05;
        }
        if p.vel.x < 0.0 && p.vel.x > -7.0 {
            p.vel.x -= 0.05;
        }
    }
    // `:26380-26394` — the per-kind gravity.
    if p.kind == MOON_BOULDER {
        p.vel.y += 0.06;
    } else if p.kind == GHOULDER {
        if p.ai[1] == 0.0 {
            p.vel.y += 0.3;
        }
    } else {
        p.vel.y += 0.3;
    }
    // `:26395-26399` — the 1048 poo-fly dust (netMode != 2 folds true): the
    // settings' circular pair + the scale float DRAWN, the body skipped.
    if p.kind == POULDER && p.vel.length() > 1.0 {
        let _ = (rng.next_float(), rng.next_float());
        let _ = rng.next_float();
    }
    // `:26400-26414` — the wet kills (netMode != 1 folds true). `AnyWet`
    // (`Entity.cs:36-46`) = wet || lavaWet || honeyWet || shimmerWet.
    let any_wet = p.wet || p.lava_wet || p.honey_wet || p.shimmer_wet;
    if any_wet {
        if p.kind == LAVA_BOULDER {
            p.kill_now();
        } else if p.wet && p.kind == BEE_HIVE {
            p.kill_now();
        } else if p.lava_wet && (p.kind == ROLLING_CACTUS || p.kind == SPIDER_BOULDER) {
            p.kill_now();
        }
    }
}

/// `WorldGen.CanCutTile` (`WorldGen.cs:86828-86839`) — the below-tile
/// planter/rope refusal (the 254 frame arm never fires for the cut kinds).
fn can_cut_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(below) = world.tile(x, y + 1) else {
        return false;
    };
    let Some(tile) = world.tile(x, y) else {
        return false;
    };
    let wall_ok = tile.wall != 350;
    let planter_ok = !matches!(below.kind, 78 | 380 | 579);
    if !wall_ok || !planter_ok {
        return false;
    }
    // `WorldGen.cs:86832-86835` — the 254 Lihzahrd altar cuts only once its
    // frameX reaches 144 (the powered state).
    if tile.kind == 254 {
        return tile.frame_x >= 144;
    }
    true
}

/// The blocked-move tile-cut prologue (`Projectile.cs:17910-17984`): the
/// 484 enchanted-sword cut (unconditional) then the `Next(3)`-gated
/// cracked-brick cut. The boulder gate folds: `CanCutTiles()` true (no
/// boulder on the `:15329` refusal list), `friendly || hostile` true,
/// `GetTileCutIgnorance(false, trap)` = the None set for the traps
/// (`Player.cs:44632-44644` — `fromTrap` skips the dontHurtNature arm; the
/// 1021/1047 non-traps keep the port's no-accessory stand-in). The owner
/// gate and `damage > 0` stay live.
fn tile_cut_prologue(
    p: &Projectile,
    world: &mut World,
    rng: &mut dyn DropRng,
    local_owner: u8,
) {
    let x0 = (((p.pos.x + p.vel.x - 1.0) / 16.0) as i32).max(0);
    let x1 = ((((p.pos.x + p.w + p.vel.x + 1.0) / 16.0) as i32) + 1).min(world.width);
    let y0 = (((p.pos.y + p.vel.y - 1.0) / 16.0) as i32).max(0);
    let y1 = ((((p.pos.y + p.h + p.vel.y + 1.0) / 16.0) as i32) + 1).min(world.height);
    let gate = p.owner == local_owner && p.damage > 0;
    // `:17910-17946` — the 484 cut (the gate precedes the scan).
    if gate {
        for i in x0..x1 {
            for j in y0..y1 {
                let cut = world
                    .tile(i, j)
                    .is_some_and(|t| t.kind == 484 && can_cut_tile(world, i, j));
                if cut {
                    let _ = tr_core::edit::kill_tile(world, i, j, false);
                }
            }
        }
    }
    // `:17948` — the C# evaluates `Main.rand.Next(3) == 0` FIRST, then the
    // owner/CanCutTiles/friendly/damage gate: the draw fires on EVERY
    // blocked move, even for a non-local owner or a zero-damage boulder.
    let roll = rng.next_range(0, 3);
    if roll != 0 || p.owner != local_owner || p.damage <= 0 {
        return;
    }
    for i in x0..x1 {
        for j in y0..y1 {
            let cut = world.tile(i, j).is_some_and(|t| {
                t.active()
                    && td::SET_CRACKED_BRICKS.has(t.kind)
                    && can_cut_tile(world, i, j)
            });
            if cut {
                let _ = tr_core::edit::kill_tile(world, i, j, false);
            }
        }
    }
}

/// The per-kind bounce ladder (`Projectile.cs:18941-19277`) — runs on a
/// blocked move (`flag`, the `:17896-17904` velocity/slope change). Returns
/// the 727's spike requests (the pool resolves them with the corpse lift).
#[allow(clippy::too_many_arguments)]
fn bounce_ladder(
    p: &mut Projectile,
    last: Vec2,
    world: &World,
    npcs: &[Npc],
    player: Option<&Player>,
    rng: &mut dyn DropRng,
    local_owner: u8,
    spam_survives: bool,
) -> Vec<SpikeSpawn> {
    let mut spikes = Vec::new();
    let dig = |p: &mut Projectile| {
        p.wants_sound = Some((0, 0, p.pos.x, p.pos.y, 0.0));
    };
    match p.kind {
        // `:18941-18986` — the 1055/99/1005/1014/1053/1057 group.
        GHOULDER | BOULDER | MINI_BOULDER | LIFE_CRYSTAL_BOULDER | LAVA_BOULDER
        | BOULDER_THAT_SPAWNS_PET => {
            if p.vel.y != last.y && last.y > 5.0 {
                // The 1053's 10×2 dust (`:18947-18956`): no explicit draws.
                dig(p);
                p.vel.y = -last.y * 0.2;
                if p.kind == GHOULDER {
                    p.ai[1] = 1.0;
                } else if p.kind == BOULDER_THAT_SPAWNS_PET {
                    p.kill_now();
                }
            }
            if p.vel.x != last.x {
                if p.kind == GHOULDER {
                    if p.ai[1] != 1.0 {
                        p.ai[1] = 1.0;
                        dig(p);
                    }
                } else {
                    p.kill_now();
                }
            }
        }
        // `:18987-19014` — the 1021 MoonBoulder.
        MOON_BOULDER => {
            if p.vel.y != last.y {
                if last.y > 4.0 {
                    dig(p);
                    p.vel.y = -last.y * 0.9;
                } else if last.y > 0.0 {
                    p.local_ai[1] = 999.0;
                }
            }
            if p.vel.x != last.x {
                if p.local_ai[1] <= 3.0 {
                    p.vel.x = -last.x * 0.75;
                    p.local_ai[1] += 1.0;
                } else {
                    p.kill_now();
                }
            }
        }
        // `:19015-19077` — the 1013 BouncyBoulder.
        BOUNCY_BOULDER => {
            let (fx, fy) = bounce_factors(last, 0.95);
            let aged = p.local_ai[0] >= 500.0;
            let mut hard = false;
            if p.vel.y != last.y {
                if aged || last.y.abs() > 5.0 {
                    hard = true;
                }
                p.vel.y = -last.y * fy;
            }
            if p.vel.x != last.x {
                if aged || last.x.abs() > 5.0 {
                    hard = true;
                }
                p.vel.x = -last.x * fx;
            }
            if hard {
                p.local_ai[1] += 1.0;
                dig(p);
            }
            if p.vel.length() < 0.1 && p.local_ai[0] > 50.0 {
                p.kill_now();
            }
            if p.local_ai[1] > 20.0 {
                p.kill_now();
            }
        }
        // `:19078-19081` — the 1047 smart bounce.
        RAINBOW_BOULDER => {
            rainbow_check_bounce(p, last, world, npcs, player, rng, local_owner, spam_survives);
        }
        // `:19082-19125` — the 727 RollingCactus.
        ROLLING_CACTUS => {
            if p.vel.y != last.y && last.y > 5.0 {
                dig(p);
                p.vel.y = -last.y * 0.65;
                spikes.extend(cactus_spike_burst(p, rng, local_owner));
            }
            if p.vel.x != last.x {
                p.kill_now();
                spikes.extend(cactus_spike_burst(p, rng, local_owner));
            }
        }
        // `:19126-19188` — the 1054 SpiderBoulder. The localAI[0] age never
        // advances for the 1054 (the `:26022` arm names only 1013/1048 — a
        // verbatim quirk: `flag20` and the rest-kill never fire).
        SPIDER_BOULDER => {
            let (fx, fy) = bounce_factors_custom(last, 0.8, 0.6);
            let aged = p.local_ai[0] >= 500.0;
            let mut hard = false;
            if p.vel.y != last.y {
                if aged || last.y.abs() > 5.0 {
                    hard = true;
                }
                p.vel.y = -last.y * fy;
            }
            if p.vel.x != last.x {
                if aged || last.x.abs() > 2.0 {
                    hard = true;
                    p.local_ai[1] += 1.0;
                }
                p.vel.x = -last.x * fx;
            }
            if hard {
                dig(p);
            }
            if p.vel.length() < 0.1 && p.local_ai[0] > 50.0 {
                p.kill_now();
            }
            if p.local_ai[1] > 1.0 {
                p.kill_now();
            }
        }
        // `:19189-19264` — the 1048 Poulder.
        POULDER => {
            let (fx, fy) = bounce_factors_custom(last, 0.75, 0.75);
            let aged = p.local_ai[0] >= 500.0;
            let mut hard = false;
            if p.vel.y != last.y {
                if aged || last.y.abs() > 4.0 {
                    hard = true;
                }
                p.vel.y = -last.y * fy;
            }
            if p.vel.x != last.x {
                if aged || last.x.abs() > 4.0 {
                    hard = true;
                }
                p.vel.x = -last.x * fx;
            }
            if hard {
                p.local_ai[1] += 3.0;
                p.wants_sound = Some((2, 177, p.pos.x, p.pos.y, 0.0));
                // `:19242-19246` — 10 dust, each settings'
                // NextVector2FromRectangle pair (2 floats) DRAWN.
                for _ in 0..10 {
                    let _ = (rng.next_float(), rng.next_float());
                }
                // `:19247-19254` — the 2+Next(2) PooFly particles: the
                // count draw + per-particle rectangle pair DRAWN, bodies
                // skipped.
                let flies = 2 + rng.next_range(0, 2);
                for _ in 0..flies {
                    let _ = (rng.next_float(), rng.next_float());
                }
            }
            if p.vel.length() < 0.1 && p.local_ai[0] > 50.0 {
                p.kill_now();
            }
            if p.local_ai[1] > 20.0 {
                p.kill_now();
            }
        }
        // `:19265-19277` — the 655 BeeHive.
        BEE_HIVE => {
            if p.vel.y != last.y && last.y > 5.0 {
                dig(p);
                p.vel.y = -last.y * 0.2;
            }
            if p.vel.x != last.x {
                p.kill_now();
            }
        }
        _ => {}
    }
    spikes
}

/// The shared 0.95-band bounce factors (`:19017-19044` and the 1047 twin
/// `:20052-20079`): `< 0.5 → 0.1`, `< 0.75 → 0.25`, `< 1 → 0.5`, else the
/// full factor.
fn bounce_factors(last: Vec2, full: f32) -> (f32, f32) {
    bounce_factors_custom(last, full, full)
}

/// The per-axis-full-factor form (the 1054's 0.8/0.6, the 1048's 0.75/0.75).
fn bounce_factors_custom(last: Vec2, full_x: f32, full_y: f32) -> (f32, f32) {
    let band = |v: f32, full: f32| {
        if v < 0.5 {
            0.1
        } else if v < 0.75 {
            0.25
        } else if v < 1.0 {
            0.5
        } else {
            full
        }
    };
    (band(last.x.abs(), full_x), band(last.y.abs(), full_y))
}

/// `HandleMovement`'s boulder half (`Projectile.cs:17469-19864`): the
/// `flag6 = true` collision (no boulder is projPet / on the fall-through
/// switch), the wet drag (`:17612-17655`), the 16-px hoik step loop
/// (`:17702-17845`) with the slope pass after every step, the blocked-move
/// tile-cut + bounce ladder, and `UpdatePosition`'s integrate
/// (`:20433-20446`). Returns the 727's spike requests.
#[allow(clippy::too_many_arguments)]
fn boulder_movement(
    p: &mut Projectile,
    world: &mut World,
    npcs: &[Npc],
    player: Option<&Player>,
    rng: &mut dyn DropRng,
    local_owner: u8,
    spam_survives: bool,
) -> Vec<SpikeSpawn> {
    let mut spikes = Vec::new();
    // `:16765` — wetVelocity starts at the velocity.
    let mut wet_vel = p.vel;
    let mut hoik_ran = false;
    if p.tile_collide {
        let last = p.vel;
        let mut blocked = false;
        if p.wet {
            // `:17612-17655` — the liquid factor: shimmer 0.375, honey
            // 0.25, water 0.5; a BLOCKED axis keeps its full value.
            let factor = if p.shimmer_wet {
                0.375
            } else if p.honey_wet {
                0.25
            } else {
                0.5
            };
            let before = p.vel;
            let tc = crate::collision::tile_collision(
                world,
                [p.pos.x, p.pos.y],
                [p.vel.x, p.vel.y],
                p.w,
                p.h,
                true,
                true,
            );
            p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
            wet_vel = p.vel * factor;
            if p.vel.x != before.x {
                wet_vel.x = p.vel.x;
            }
            if p.vel.y != before.y {
                wet_vel.y = p.vel.y;
            }
            // `:17656-17698` — the wet slope pass (fall: true).
            let sc = crate::collision::slope_collision(
                world,
                [p.pos.x, p.pos.y],
                [p.vel.x, p.vel.y],
                p.w,
                p.h,
                true,
            );
            if p.pos.x != sc.pos[0] || p.pos.y != sc.pos[1]
                || p.vel.x != sc.vel[0] || p.vel.y != sc.vel[1]
            {
                blocked = true;
            }
            p.pos = Vec2::new(sc.pos[0], sc.pos[1]);
            p.vel = Vec2::new(sc.vel[0], sc.vel[1]);
        } else {
            // `:17702-17710` — the step-size clamp.
            let step = (p.w.min(p.h) as i32).clamp(3, 16) as f32;
            if p.vel.length() > step {
                // `:17711-17845` — the hoik step loop. The C# accumulates
                // into the LOCAL `vector`; the slope pass's
                // `vector12 = base.position - vector` offset keeps
                // `base.position` stationary (only the slope's own push
                // lands) — `UpdatePosition` integrates ONCE at the tail.
                hoik_ran = true;
                let probe = crate::collision::tile_collision(
                    world,
                    [p.pos.x, p.pos.y],
                    [p.vel.x, p.vel.y],
                    p.w,
                    p.h,
                    true,
                    true,
                );
                let mut remaining = p.vel.length();
                let mut dir = crate::projectile::safe_normalize(p.vel);
                if probe.vel[1] == 0.0 {
                    dir.y = 0.0;
                }
                let mut vector = p.pos;
                let mut zero = Vec2::ZERO;
                let mut steps = 0;
                while remaining > 0.0 {
                    steps += 1;
                    if steps > 300 {
                        break;
                    }
                    let vector9 = vector;
                    let take = remaining.min(step);
                    remaining -= take;
                    let sv = dir * take;
                    let tc = crate::collision::tile_collision(
                        world,
                        [vector.x, vector.y],
                        [sv.x, sv.y],
                        p.w,
                        p.h,
                        true,
                        true,
                    );
                    let moved = Vec2::new(tc.vel[0], tc.vel[1]);
                    vector += moved;
                    p.vel = moved;
                    // `:17744-17786` — the per-step slope pass with the
                    // position-offset trick.
                    let sc = crate::collision::slope_collision(
                        world,
                        [vector.x, vector.y],
                        [p.vel.x, p.vel.y],
                        p.w,
                        p.h,
                        true,
                    );
                    if vector.x != sc.pos[0] || vector.y != sc.pos[1]
                        || p.vel.x != sc.vel[0] || p.vel.y != sc.vel[1]
                    {
                        blocked = true;
                    }
                    let offset = p.pos - vector;
                    vector = Vec2::new(sc.pos[0], sc.pos[1]);
                    p.pos = vector + offset;
                    p.vel = Vec2::new(sc.vel[0], sc.vel[1]);
                    // `:17788-17791` — the owner-gated in-loop SwitchTiles
                    // (objType 4, `!flag4`): only the 1005 MiniBoulder is
                    // unlisted in ForcePlateDetection among the boulders.
                    // Parked for the world sim (the wiring borrow).
                    if p.owner == local_owner
                        && vector != vector9
                        && !force_plate_detection_false(p.kind)
                    {
                        p.wants_boulder_switches.push((
                            vector.x, vector.y, vector9.x, vector9.y,
                            p.w as i32, p.h as i32, 4,
                        ));
                    }
                    zero += p.vel;
                }
                p.vel = zero;
                // `:17795-17802` — the last-velocity snap.
                if (p.vel.x - last.x).abs() < 0.0001 {
                    p.vel.x = last.x;
                }
                if (p.vel.y - last.y).abs() < 0.0001 {
                    p.vel.y = last.y;
                }
                // `:17803-17845` — the post-loop slope pass (same offset).
                let sc = crate::collision::slope_collision(
                    world,
                    [vector.x, vector.y],
                    [p.vel.x, p.vel.y],
                    p.w,
                    p.h,
                    true,
                );
                if vector.x != sc.pos[0] || vector.y != sc.pos[1]
                    || p.vel.x != sc.vel[0] || p.vel.y != sc.vel[1]
                {
                    blocked = true;
                }
                let offset = p.pos - vector;
                p.pos = Vec2::new(sc.pos[0], sc.pos[1]) + offset;
                p.vel = Vec2::new(sc.vel[0], sc.vel[1]);
            } else {
                // `:17849-17892` — the single collision + slope pass.
                let tc = crate::collision::tile_collision(
                    world,
                    [p.pos.x, p.pos.y],
                    [p.vel.x, p.vel.y],
                    p.w,
                    p.h,
                    true,
                    true,
                );
                p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
                let sc = crate::collision::slope_collision(
                    world,
                    [p.pos.x, p.pos.y],
                    [p.vel.x, p.vel.y],
                    p.w,
                    p.h,
                    true,
                );
                if p.pos.x != sc.pos[0] || p.pos.y != sc.pos[1]
                    || p.vel.x != sc.vel[0] || p.vel.y != sc.vel[1]
                {
                    blocked = true;
                }
                p.pos = Vec2::new(sc.pos[0], sc.pos[1]);
                p.vel = Vec2::new(sc.vel[0], sc.vel[1]);
            }
        }
        // `:17896-17899` — the velocity-change flag.
        if last != p.vel {
            blocked = true;
        }
        if blocked {
            // `:17910-17984` — the tile-cut prologue, then the ladder.
            tile_cut_prologue(p, world, rng, local_owner);
            spikes = bounce_ladder(p, last, world, npcs, player, rng, local_owner, spam_survives);
        }
    }
    // `UpdatePosition` (`:20433-20446`) — the wet/dry integrate. The C#
    // hoik loop already integrated its steps; the port's loop mirrors that
    // (pos += moved per step), so the tail integrate double-steps the hoik
    // path in C# TOO (`UpdatePosition` runs unconditionally at `:19858`) —
    // verbatim.
    p.pos += if p.wet { wet_vel } else { p.vel };
    // `:19859-19863` — the objType-4 pad call: `!flag2 && !flag4 &&
    // owner == myPlayer && (tileCollide | flag5) && pos != oldPosition`.
    // flag5 = false for every boulder (ForcePlateDetection carries no
    // `true` row for them). The request parks for the world sim.
    if !hoik_ran
        && !force_plate_detection_false(p.kind)
        && p.owner == local_owner
        && p.tile_collide
        && p.pos != p.old_position
    {
        p.wants_boulder_switches.push((
            p.pos.x, p.pos.y, p.old_position.x, p.old_position.y,
            p.w as i32, p.h as i32, 4,
        ));
    }
    spikes
}

/// The full per-sub-step boulder tick: the gfx decay, the wet probe (the
/// bobber tick-head convention), the AI arm, the Shimmer float, the
/// `:17087` objType-3 plate scan, the `:17013` oldPosition latch,
/// `HandleMovement` and the `:17016-17026` direction write. Returns the
/// 727's spike requests.
#[allow(clippy::too_many_arguments)]
pub(crate) fn boulder_step(
    p: &mut Projectile,
    world: &mut World,
    npcs: &[Npc],
    player: Option<&Player>,
    rng: &mut dyn DropRng,
    local_owner: u8,
    tile_frame_665: i32,
    spam_survives: bool,
) -> Vec<SpikeSpawn> {
    if !BOULDER_KINDS.contains(&p.kind) {
        return Vec::new();
    }
    // The 1021 is hostile-ONLY (`:1866` sets no friendly flag — the `:788`
    // stock false); the port's slot default is true, so the arm holds the
    // C# state (the dungeon-debris precedent). The PVE walk's kind gate
    // reads it.
    if p.kind == MOON_BOULDER {
        p.friendly = false;
    }
    gfx_decay(p);
    // The wet probe (the C# `Update` tail `:16790-17007` probes AFTER the
    // AI; the tick-head probe is the bobber's recorded one-tick lag).
    // `ignoreWater` (the 1047, `:9926`) skips it.
    if !p.ignore_water {
        crate::projectile_bobber::probe_wet(p, world);
    }
    ai_boulder(p, world, npcs, player, rng, local_owner, tile_frame_665, spam_survives);
    // `:17009-17012` — the Shimmer float (the AI may have killed the slot;
    // the C# runs Shimmer on the dead slot too — its writes land on a corpse
    // nothing reads, but the port keeps the order).
    if p.shimmer_wet {
        shimmer_step(p);
    }
    // `:17087-17090` — the objType-3 plate scan (netMode != 1 folds true),
    // reading LAST sub-step's oldPosition latch. Parked for the world sim.
    if plate_scan_kind(p.kind) {
        p.wants_boulder_switches.push((
            p.pos.x, p.pos.y, p.old_position.x, p.old_position.y,
            p.w as i32, p.h as i32, 3,
        ));
    }
    // `:17013` — the oldPosition latch between the AI and HandleMovement.
    p.old_position = p.pos;
    let spikes = boulder_movement(p, world, npcs, player, rng, local_owner, spam_survives);
    // `:17016-17026` + `AutomaticallyChangesDirection` (`:17281-17288`,
    // true for aiStyle 25): the direction follows velocity.X's sign.
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
    spikes
}

/// The hostile-boulder player contact (`Damage_EVP` `Projectile.cs:
/// 14773-14900`, the `damage_player` stand-in WITHOUT the single-hit kill —
/// the boulders are penetrate −1 and the C# EVP arm never kills the shot).
/// The box overlap is the strict rect intersect (the C# `Colliding` default
/// arm); the hurt rides the port's iframe-gated lane.
pub(crate) fn player_contact(
    p: &mut Projectile,
    player: Option<&mut Player>,
    rng: &mut dyn DropRng,
) {
    let Some(player) = player else { return };
    if p.damage <= 0 || player.dead {
        return;
    }
    let hit = p.pos.x < player.x + player.width
        && p.pos.x + p.w > player.x
        && p.pos.y < player.y + player.height
        && p.pos.y + p.h > player.y;
    if !hit {
        return;
    }
    let cx = p.pos.x + p.w * 0.5;
    let _ = player.hurt_with_source(p.damage, cx, rng, p.is_damage_dodgeable(), p.kind);
}

// =====================================================================
// The Kill() death hooks (pool-side; the corpse-flag lift precedents).
// =====================================================================

/// One parked item drop (`Item.NewItem`, `:74952`/`:74956`/`:75096`):
/// `(item, stack, x, y)` — the world sim drains it on the `UpdateItems`
/// stream (the bobber-catch precedent).
pub(crate) type BoulderDrop = (u16, i32, f32, f32);

/// One parked bee spawn (`:75116-75118`): `(kind 210/211, x, y, vx, vy)` —
/// `NewNPC`'s Bottom placement, the world sim resolves it (the Queen-Bee
/// volley precedent).
pub(crate) type BoulderBee = (u16, f32, f32, f32, f32);

/// `BoulderExplosion` (`Projectile.cs:70563-70575`, the getGoodWorld 99
/// death): the `Next(2, 5)` shard count, per shard the two-float direction
/// (normalized ×8) and the 1005 spawn at `(int)(damage * 0.75)` / kb 0 /
/// timeLeft 300. Returns the shard payloads `(x, y, vx, vy, damage)`.
pub(crate) fn boulder_explosion(p: &Projectile, rng: &mut dyn DropRng) -> Vec<(f32, f32, f32, f32, i32)> {
    let count = rng.next_range(2, 5);
    let c = center(p);
    let damage = (p.damage as f64 * 0.75) as i32;
    let mut out = Vec::new();
    for _ in 0..count {
        let v = Vec2::new(rng.next_float() - 0.5, -rng.next_float() / 2.0);
        let v = v.normalize_or_zero() * 8.0;
        out.push((c.x, c.y, v.x, v.y, damage));
    }
    out
}

/// The 1047 rainbow death burst's draw census (`Projectile.cs:71192-71255`):
/// the `NextFloat` hue seed, then per filled oldPos entry the `Next(1, 4)`
/// count (the 0.3/0.65/0.85 threshold decrements) and 3 floats per dust
/// (velocity scale / scale / fadeIn — the NewDust+CloneDust internals ride
/// the skipped dust layer), then the 14 colour floats. The two RainbowRodHit
/// particles (`:71242-71249`) carry no settings draws — bodies skipped. The
/// ring is the port's 10 entries (the C#'s 25 — the module-doc deviation).
pub(crate) fn rainbow_boulder_death_draws(p: &Projectile, rng: &mut dyn DropRng) {
    let _ = rng.next_float(); // `:71196`
    let len = p.old_pos.len() as f32;
    for (i, &op) in p.old_pos.iter().enumerate() {
        if op == Vec2::ZERO {
            break;
        }
        let mut count = rng.next_range(1, 4); // `:71205`
        let step = i as f32;
        if step >= len * 0.3 {
            count -= 1;
        }
        if step >= len * 0.65 {
            count -= 2;
        }
        if step >= len * 0.85 {
            count -= 3;
        }
        for _ in 0..count.max(0) {
            let _ = rng.next_float(); // `:71226` velocity scale
            let _ = rng.next_float(); // `:71228` scale
            let _ = rng.next_float(); // `:71229` fadeIn
        }
    }
    for _ in 0..14 {
        let _ = rng.next_float(); // `:71252` the hslToRgb hue
    }
}

/// `WorldGen.PlaceLiquid`'s lava arm (`WorldGen.cs:4578-4629`) — the
/// LavaBoulderExplosion's flood. The full-box solid refusal, the
/// same-type/empty fill (saturating at 255) and the merge tiles (lava over
/// water → 56 obsidian, over honey → 230, over shimmer → 659 —
/// `Liquid.GetLiquidMergeTypes`, `Liquid.cs:1375`). The SquareTileFrame
/// refresh rides `edit::place_tile`/the framing seam; the net sends skip.
fn place_lava(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile(x, y).copied() else { return };
    if t.active()
        && !t.inactive()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
    {
        return;
    }
    let existing = if t.flags & tr_core::tile::FLAG_LIQUID_LAVA != 0 {
        1
    } else if t.flags & tr_core::tile::FLAG_LIQUID_HONEY != 0 {
        2
    } else if t.flags & tr_core::tile::FLAG_LIQUID_SHIMMER != 0 {
        3
    } else {
        0
    };
    if t.liquid == 0 || existing == 1 {
        if let Some(tile) = world.tile_mut(x, y) {
            tile.set_liquid_kind(tr_core::tile::LiquidKind::Lava);
            tile.liquid = tile.liquid.saturating_add(255);
        }
        return;
    }
    // The merge: clear the liquid, place the merge tile.
    let merge = match existing {
        0 => 56,
        2 => 230,
        3 => 659,
        _ => return,
    };
    if let Some(tile) = world.tile_mut(x, y) {
        tile.liquid = 0;
        tile.set_liquid_kind(tr_core::tile::LiquidKind::Water);
    }
    let _ = tr_core::edit::place_tile(world, x, y, merge);
}

/// `LavaBoulderExplosion` (`Projectile.cs:70538-70552`): the radius-4 lava
/// flood around `((int)Center.X / 16, (int)Center.Y / 16)` (the C# truncates
/// the centre BEFORE the tile divide), gated on `Distance < 4` +
/// `Collision.CanHit` from the shot's box to the tile.
pub(crate) fn lava_boulder_explosion(p: &Projectile, world: &mut World) {
    let c = center(p);
    let cx = c.x as i32 / 16;
    let cy = c.y as i32 / 16;
    let origin = Vec2::new(cx as f32, cy as f32);
    for i in (cx - 4)..=(cx + 4) {
        for j in (cy - 4)..=(cy + 4) {
            if origin.distance(Vec2::new(i as f32, j as f32)) < 4.0
                && can_hit(
                    world,
                    p.pos.x, p.pos.y, p.w, p.h,
                    i as f32 * 16.0, j as f32 * 16.0, 16.0, 16.0,
                )
            {
                place_lava(world, i, j);
            }
        }
    }
}

/// `Kill_DirtAndFluidProjectiles_RunDelegateMethodPushUpForHalfBricks`
/// (`Projectile.cs:70522-70536`) + `Utils.FloodFillTile`
/// (`Utils.cs:2380-2424`) + `DelegateMethods.SpreadPoopPyramid`/`SpreadTile`
/// (`DelegateMethods.cs:468-498`): the Poulder's death spreads the 666 poop
/// pyramid (size 2) from the centre tile (pushed up one row off a
/// half-brick). The BFS expands only through cells the pyramid plot accepts
/// (the C# flood-fill's `plot` gate); the dust bodies skip (no explicit
/// draws). `TryKillingReplaceableTile` (`WorldGen.cs:59312-59340`): the
/// tileCut/breakable/drip cut with the below-tile planter refusal (the 666
/// is not an alch soil, so the herb exception never fires).
pub(crate) fn poulder_spread(p: &Projectile, world: &mut World) {
    let c = center(p);
    let mut px = (c.x / 16.0) as i32;
    let mut py = (c.y / 16.0) as i32;
    // `:70524-70531` — the half-brick push-up.
    let half = world
        .tile(px, py)
        .is_some_and(|t| t.active() && t.half_brick());
    if half
        && py - 1 >= 0
        && !world
            .tile(px, py - 1)
            .is_some_and(|t| crate::collision::solid_or_sloped_tile(t, false))
    {
        py -= 1;
    }
    let size = 2.0f32.max(0.1);
    let origin = Vec2::new(px as f32, py as f32);
    let plot = |world: &World, x: i32, y: i32| -> bool {
        // `SpreadTile`'s pyramid gate (`DelegateMethods.cs:476-484`).
        let dy = (y as f32 - (origin.y + size / 2.0)).abs() / size;
        if (x as f32 - origin.x).abs() > size * (1.0 - dy)
            || (y as f32 - origin.y).abs() > size
        {
            return false;
        }
        true
    };
    // The BFS (`Utils.FloodFillTile`): the visited set bounds to the
    // ceil(size)+1 box (`BitSet2D.Reset`), the four-neighbour expansion in
    // the C#'s left/right/up/down order.
    let radius = size.ceil() as i32 + 1;
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert((px, py));
    queue.push_back((px, py));
    while let Some((x, y)) = queue.pop_front() {
        if !plot(world, x, y) {
            continue;
        }
        // `TryKillingReplaceableTile` + `PlaceTile(666)`.
        let replaceable = world.tile(x, y).is_some_and(|t| {
            t.active()
                && (tr_core::tileobject::tile_cut(t.kind)
                    || td::SET_BREAKABLE_WHEN_PLACING.has(t.kind)
                    || td::SET_IS_A_DRIP_TILE.has(t.kind))
                && t.kind != 666
        });
        if replaceable {
            let below_ok = world
                .tile(x, y + 1)
                .is_some_and(|t| !matches!(t.kind, 78 | 380 | 579));
            if below_ok {
                let _ = tr_core::edit::kill_tile(world, x, y, false);
            }
        }
        let _ = tr_core::edit::place_tile(world, x, y, 666);
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if (nx - px).abs() > radius || (ny - py).abs() > radius {
                continue;
            }
            if nx < 0 || ny < 0 || nx >= world.width || ny >= world.height {
                continue;
            }
            if visited.insert((nx, ny)) {
                queue.push_back((nx, ny));
            }
        }
    }
}

/// The 655 bee-hive death's bee payload (`Projectile.cs:75100-75121`): the
/// base-2 count + three `Next(3)` bump gates, the slot-cap fold
/// (`100 - npc_count`, the module-doc deviation), then per bee the
/// `Next(210, 212)` kind and the two `Next(-200, 201) * 0.002` velocity
/// kicks. The wet arm's honey drop (`:75096`, `Next(2, 6)` stack) is the
/// caller's. Returns the bee spawns at the truncated centre.
pub(crate) fn bee_hive_bees(
    p: &Projectile,
    npc_count: usize,
    rng: &mut dyn DropRng,
) -> Vec<BoulderBee> {
    let mut want = 2;
    if rng.next_range(0, 3) == 0 {
        want += 1;
    }
    if rng.next_range(0, 3) == 0 {
        want += 1;
    }
    if rng.next_range(0, 3) == 0 {
        want += 1;
    }
    let available = (100usize.saturating_sub(npc_count)).min(want as usize);
    let c = center(p);
    let mut out = Vec::new();
    for _ in 0..available {
        let kind = rng.next_range(210, 212) as u16;
        let vx = rng.next_range(-200, 201) as f32 * 0.002;
        let vy = rng.next_range(-200, 201) as f32 * 0.002;
        out.push((kind, c.x as i32 as f32, c.y as i32 as f32, vx, vy));
    }
    out
}

/// The family Kill arm's dust-loop census (`Projectile.cs:74938-74948` /
/// `:74973-74981` / `:75083-75091`): `count` dust, each with the explicit
/// `Next(2)` scale gate DRAWN and the ×factor velocity growth (a verbatim
/// C# quirk — the corpse's velocity compounds per dust).
pub(crate) fn family_dust_loop(p: &mut Projectile, count: i32, factor: f32, rng: &mut dyn DropRng) {
    for _ in 0..count {
        let _ = rng.next_range(0, 2);
        p.vel *= factor;
    }
}

/// The 1053 lava death's gore census (`Projectile.cs:74898-74913`): four
/// gores, each the `Next(61, 64)` type + the `NewGore` kicks
/// (`Next(10, 31)` Y then `Next(-20, 21)` X, `Gore.cs:1327-1328` — the G3
/// order). The bodies skip.
pub(crate) fn lava_death_gore_draws(rng: &mut dyn DropRng) {
    for _ in 0..4 {
        let _ = rng.next_range(61, 64);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
    }
}

// =====================================================================
// Witnesses — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    /// A scripted `DropRng` recording every draw (the `projectile_ai_thrown`
    /// ScriptRng pattern); floats come from a parallel script.
    struct ScriptRng {
        rolls: Vec<i32>,
        floats: Vec<f32>,
        draws: Vec<(i32, i32)>,
        float_draws: usize,
    }
    impl ScriptRng {
        fn new(rolls: &[i32], floats: &[f32]) -> Self {
            Self {
                rolls: rolls.to_vec(),
                floats: floats.to_vec(),
                draws: Vec::new(),
                float_draws: 0,
            }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.rolls.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            self.float_draws += 1;
            if self.floats.is_empty() {
                0.0
            } else {
                self.floats.remove(0)
            }
        }
    }

    fn proj(kind: u16) -> Projectile {
        let mut p = Projectile { kind, active: true, owner: 0, ..Default::default() };
        if let Some(d) = proj_data(kind) {
            p.w = d.w;
            p.h = d.h;
            p.time_left = d.time_left;
            p.penetrate = d.penetrate;
            p.max_penetrate = d.penetrate;
            p.extra_updates = d.extra_updates;
            p.tile_collide = d.tile_collide;
            p.trap = d.trap;
            p.ignore_water = d.ignore_water;
        }
        p
    }

    fn world() -> World {
        World::new("t".into(), 200, 200)
    }

    /// The 13 rows + the 763 spike match `SetDefaults`
    /// (`Projectile.cs:1833-1870`, `:7038-7047`, `:9918-9937`, `:7855-7865`).
    #[test]
    fn w4_d3_the_boulder_rows_match_set_defaults() {
        // (kind, w, h, penetrate, timeLeft, extraUpdates, trap)
        let cases: [(u16, f32, f32, i32, i32, u32, bool); 14] = [
            (BOULDER, 31.0, 31.0, -1, 3600, 0, true),
            (ROLLING_CACTUS, 31.0, 31.0, -1, 3600, 0, true),
            (BOUNCY_BOULDER, 31.0, 31.0, -1, 3600, 0, true),
            (LIFE_CRYSTAL_BOULDER, 31.0, 31.0, -1, 3600, 0, true),
            (LAVA_BOULDER, 31.0, 31.0, -1, 3600, 0, true),
            (SPIDER_BOULDER, 31.0, 31.0, -1, 300, 0, true),
            (GHOULDER, 31.0, 31.0, -1, 900, 0, true),
            (BOULDER_THAT_SPAWNS_PET, 31.0, 31.0, -1, 3600, 0, true),
            (MINI_BOULDER, 15.0, 15.0, -1, 3600, 0, true),
            (MOON_BOULDER, 31.0, 31.0, -1, 10800, 1, false),
            (BEE_HIVE, 31.0, 31.0, -1, 3600, 0, true),
            (RAINBOW_BOULDER, 31.0, 31.0, -1, 3600, 0, false),
            (POULDER, 31.0, 31.0, -1, 3600, 0, true),
            (ROLLING_CACTUS_SPIKE, 10.0, 10.0, -1, 3600, 1, false),
        ];
        for (kind, w, h, pen, life, extra, trap) in cases {
            let d = proj_data(kind).unwrap_or_else(|| panic!("the {kind} row"));
            assert_eq!((d.w, d.h), (w, h), "kind {kind} box");
            assert_eq!(d.penetrate, pen, "kind {kind} penetrate");
            assert_eq!(d.time_left, life, "kind {kind} timeLeft");
            assert_eq!(d.extra_updates, extra, "kind {kind} extraUpdates");
            assert_eq!(d.trap, trap, "kind {kind} trap");
            assert!(d.tile_collide, "kind {kind} tileCollide");
            let style = if kind == ROLLING_CACTUS_SPIKE { 1 } else { 25 };
            assert_eq!(d.ai_style, style, "kind {kind} aiStyle");
        }
        // The 1047 alone ignores water (`:9926`).
        assert!(proj_data(RAINBOW_BOULDER).unwrap().ignore_water);
        assert!(!proj_data(BOULDER).unwrap().ignore_water);
        // The frame sheets (`Main.cs:8713-8714`).
        assert_eq!(crate::projectile::proj_frames(LIFE_CRYSTAL_BOULDER), 11);
        assert_eq!(crate::projectile::proj_frames(GHOULDER), 10);
        // The trailing modes (`ProjectileID.cs:521`: 1047 → 0, 1055 → 0).
        assert_eq!(crate::projectile::trailing_mode(RAINBOW_BOULDER), 0);
        assert_eq!(crate::projectile::trailing_mode(GHOULDER), 0);
    }

    /// The shared gravity/accel tail (`:26336-26394`): a resting 99 gains
    /// +0.3 Y per tick, the roll follows velocity.X ×0.06, ai[0] latches 1.
    #[test]
    fn w4_d3_the_boulder_gravity_and_roll() {
        let mut w = world();
        let mut p = proj(BOULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        p.vel = Vec2::new(2.0, 0.0);
        p.ai[0] = 1.0; // skip the wall-nudge ladder
        let mut rng = ScriptRng::new(&[], &[]);
        ai_boulder(&mut p, &mut w, &[], None, &mut rng, 0, 0, true);
        assert_eq!(p.ai[0], 1.0, ":26336");
        assert!((p.vel.y - 0.3).abs() < 1e-6, ":26393");
        assert!((p.vel.x - 2.05).abs() < 1e-6, ":26371-26374 the +0.05 accel");
        assert!((p.rot - 2.0 * 0.06).abs() < 1e-6, ":26334");
        assert_eq!(p.local_ai[2], 1.0, ":26017");
        // The 16 px/tick cap (`:26337-26340`).
        p.vel.y = 20.0;
        ai_boulder(&mut p, &mut w, &[], None, &mut rng, 0, 0, true);
        assert!((p.vel.y - 16.3).abs() < 1e-6, "cap 16 then +0.3");
    }

    /// The wall-nudge ladder (`:26248-26331`): a left wall at the ±8 probe
    /// rolls the boulder right at 0.5; with no wall the parity fallback
    /// picks by `(int)(Center.X / 16) % 2`.
    #[test]
    fn w4_d3_the_wall_nudge_ladder() {
        let mut w = world();
        // A solid column at tile x=99 (px 1584..1600), rows 100/101.
        for ty in [100, 101] {
            let idx = w.index(99, ty);
            w.tiles[idx].kind = 1;
            w.tiles[idx].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = proj(BOULDER);
        // pos.x - 8 = 1592 → tile 99 (the left probe hits).
        p.pos = Vec2::new(1600.0, 1600.0);
        p.vel = Vec2::ZERO;
        p.ai[0] = 1.0;
        let mut rng = ScriptRng::new(&[], &[]);
        ai_boulder(&mut p, &mut w, &[], None, &mut rng, 0, 0, true);
        // The nudge sets 0.5, then the shared +0.05 accel (`:26371`).
        assert!((p.vel.x - 0.55).abs() < 1e-6, "the left wall rolls right");
        // No walls: centre 1615.5 / 16 = 100 (even) → +0.5, then +0.05.
        let mut q = proj(BOULDER);
        q.pos = Vec2::new(1600.0, 1600.0);
        q.vel = Vec2::ZERO;
        q.ai[0] = 1.0;
        let empty = world();
        let mut w2 = empty;
        ai_boulder(&mut q, &mut w2, &[], None, &mut rng, 0, 0, true);
        assert!((q.vel.x - 0.55).abs() < 1e-6, "the even-parity fallback");
    }

    /// The 1013 bounce factors (`:19015-19077`): a fast hit reverses ×0.95
    /// and bumps localAI[1]; a slow hit reverses ×0.1; >20 hard hits kill.
    #[test]
    fn w4_d3_the_bouncy_boulder_bounce() {
        let w = world();
        let mut p = proj(BOUNCY_BOULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        p.vel = Vec2::new(0.0, 0.0); // the collision zeroed both axes
        let last = Vec2::new(6.0, 6.0);
        let mut rng = ScriptRng::new(&[], &[]);
        let spikes = bounce_ladder(&mut p, last, &w, &[], None, &mut rng, 0, true);
        assert!(spikes.is_empty());
        assert!((p.vel.x + 5.7).abs() < 1e-6, "-6 × 0.95");
        assert!((p.vel.y + 5.7).abs() < 1e-6);
        assert_eq!(p.local_ai[1], 1.0, "the hard-hit counter");
        assert_eq!(p.wants_sound, Some((0, 0, 1600.0, 1600.0, 0.0)));
        // The slow band: |0.4| < 0.5 → ×0.1, no hard hit.
        let mut q = proj(BOUNCY_BOULDER);
        q.vel = Vec2::ZERO;
        let mut rng2 = ScriptRng::new(&[], &[]);
        bounce_ladder(&mut q, Vec2::new(0.4, 0.4), &w, &[], None, &mut rng2, 0, true);
        assert!((q.vel.x + 0.04).abs() < 1e-6);
        assert_eq!(q.local_ai[1], 0.0);
        // The bounce budget (`:19073-19076`).
        let mut r = proj(BOUNCY_BOULDER);
        r.local_ai[1] = 21.0;
        r.vel = Vec2::ZERO;
        let mut rng3 = ScriptRng::new(&[], &[]);
        bounce_ladder(&mut r, Vec2::new(6.0, 0.0), &w, &[], None, &mut rng3, 0, true);
        assert!(!r.active, "localAI[1] > 20 kills");
    }

    /// The 1021 MoonBoulder ladder (`:18987-19014`): >4 falls bounce ×0.9,
    /// a soft landing pins localAI[1] = 999, the X bounce budget is 4.
    #[test]
    fn w4_d3_the_moon_boulder_ladder() {
        let w = world();
        let mut p = proj(MOON_BOULDER);
        p.vel = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[], &[]);
        bounce_ladder(&mut p, Vec2::new(0.0, 5.0), &w, &[], None, &mut rng, 0, true);
        assert!((p.vel.y + 4.5).abs() < 1e-6, "-5 × 0.9");
        let mut q = proj(MOON_BOULDER);
        q.vel = Vec2::new(0.0, 0.0);
        bounce_ladder(&mut q, Vec2::new(0.0, 2.0), &w, &[], None, &mut rng, 0, true);
        assert_eq!(q.local_ai[1], 999.0, "the soft-landing pin");
        // Four X bounces at ×−0.75, the fifth kills.
        let mut r = proj(MOON_BOULDER);
        r.vel = Vec2::new(0.0, 0.0);
        for i in 0..4 {
            r.vel.x = 0.0;
            bounce_ladder(&mut r, Vec2::new(4.0, 0.0), &w, &[], None, &mut rng, 0, true);
            assert!(r.active, "bounce {i}");
            assert_eq!(r.local_ai[1], (i + 1) as f32);
        }
        r.vel.x = 0.0;
        bounce_ladder(&mut r, Vec2::new(4.0, 0.0), &w, &[], None, &mut rng, 0, true);
        assert!(!r.active, "the fifth X hit kills");
    }

    /// The 727 spike burst (`:19092-19105`): Next(3,6)=3 + NextFloat=0 →
    /// three spikes at t = 0, 1/3, 2/3; the downward halves flip ×−0.7.
    #[test]
    fn w4_d3_the_cactus_spike_burst_draw_order() {
        let mut p = proj(ROLLING_CACTUS);
        p.pos = Vec2::new(1600.0, 1600.0);
        let mut rng = ScriptRng::new(&[3], &[0.0]);
        let spikes = cactus_spike_burst(&p, &mut rng, 0);
        assert_eq!(rng.draws, vec![(3, 6)], "the count draw");
        assert_eq!(rng.float_draws, 1, "the seed draw");
        assert_eq!(spikes.len(), 3);
        // t=0: angle 0 → (2, 0); y == 0 is NOT > 0 — no flip.
        assert!((spikes[0].vx - 2.0).abs() < 1e-5);
        assert!(spikes[0].vy.abs() < 1e-5);
        // t=1/3: angle 2π/3 → (−1, √3); y > 0 → ×−0.7 = (0.7, −1.2124).
        assert!((spikes[1].vx - 0.7).abs() < 1e-4);
        assert!((spikes[1].vy + 3.0f32.sqrt() * 0.7).abs() < 1e-4);
        // t=2/3: angle 4π/3 → (−1, −√3); y < 0 keeps it.
        assert!((spikes[2].vx + 1.0).abs() < 1e-4);
        assert!((spikes[2].vy + 3.0f32.sqrt()).abs() < 1e-4);
        // The centre spawn point.
        assert_eq!((spikes[0].x, spikes[0].y), (1615.5, 1615.5));
        // The owner gate: a non-local owner draws NOTHING.
        p.owner = 1;
        let mut rng2 = ScriptRng::new(&[], &[]);
        assert!(cactus_spike_burst(&p, &mut rng2, 0).is_empty());
        assert!(rng2.draws.is_empty());
    }

    /// The 1055 ground bounce arms the chase (`:18961-18964`), and the
    /// chase blend (`:26192-26195`) steers (vel×35 + toward×6)/36.
    #[test]
    fn w4_d3_the_ghoulder_bounce_and_chase() {
        let w = world();
        let mut p = proj(GHOULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        p.vel = Vec2::new(0.0, 0.0);
        let mut rng = ScriptRng::new(&[], &[]);
        bounce_ladder(&mut p, Vec2::new(0.0, 6.0), &w, &[], None, &mut rng, 0, true);
        assert_eq!(p.ai[1], 1.0, "the bounce arms the flight");
        assert!((p.vel.y + 1.2).abs() < 1e-6, "-6 × 0.2");
        // The chase: the player directly right, overlapping X → chase.
        let mut pl = Player::spawn(&w);
        pl.x = 1610.0;
        pl.y = 1600.0;
        pl.width = 20.0;
        pl.height = 42.0;
        pl.facing = 1;
        p.vel = Vec2::ZERO;
        p.tile_collide = true;
        let mut w2 = world();
        ai_boulder(&mut p, &mut w2, &[], Some(&pl), &mut rng, 0, 0, true);
        assert!(!p.tile_collide, ":26180");
        assert_eq!(p.local_ai[0], 3.0, "the ramp");
        // toward = player.Center − boulder.Center, normalized ×6, /36.
        let pc = Vec2::new(pl.x + 10.0, pl.y + 21.0);
        let toward = (pc - Vec2::new(1615.5, 1615.5)).normalize() * 6.0;
        let expect = toward / 36.0;
        assert!((p.vel.x - expect.x).abs() < 1e-5);
        assert!((p.vel.y - expect.y).abs() < 1e-5, "the chase owns Y (no gravity)");
    }

    /// The Shimmer arm (`:22504-22515`): >10 decays ×0.97, −0.7 lift, −10
    /// floor.
    #[test]
    fn w4_d3_the_shimmer_float() {
        let mut p = proj(BOULDER);
        p.vel.y = 12.0;
        shimmer_step(&mut p);
        assert!((p.vel.y - (12.0 * 0.97 - 0.7)).abs() < 1e-6);
        p.vel.y = -9.8;
        shimmer_step(&mut p);
        assert_eq!(p.vel.y, -10.0, "the floor");
    }

    /// The RainbowBoulder jump (`:20222-20256`): a stationary target 100 px
    /// right solves at every speed 8..32; the LAST (32) wins —
    /// time = 100/32 = 3.125, ai[1] = (int)(3.125 − 2) = 1, and
    /// FactorAcceleration((32,0), 3.125, (0,0.6), 0) = (32, −0.9375).
    #[test]
    fn w4_d3_the_rainbow_boulder_jump_solve() {
        let mut p = proj(RAINBOW_BOULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        let target = Vec2::new(1715.5, 1615.5); // 100 px right of the centre
        rainbow_jump_to(&mut p, target, Vec2::ZERO, 0);
        assert!((p.vel.x - 32.0).abs() < 1e-4);
        assert!((p.vel.y + 0.9375).abs() < 1e-4, "0.6 × 3.125 / 2");
        assert_eq!(p.ai[1], 1.0);
        assert!(p.net_update);
    }

    /// PreventSpam (`:20024-20048`): seven same-owner same-kind siblings in
    /// the 6000-box kill; six survive; other owners/kinds do not count.
    #[test]
    fn w4_d3_the_rainbow_boulder_spam_cap() {
        let mut pool: Vec<Projectile> = (0..8)
            .map(|_| {
                let mut q = proj(RAINBOW_BOULDER);
                q.pos = Vec2::new(1600.0, 1600.0);
                q
            })
            .collect();
        // 7 siblings + me = 8 → the cap kills.
        assert!(!rainbow_spam_survives(&pool, 0));
        pool[7].active = false;
        assert!(rainbow_spam_survives(&pool, 0), "six siblings survive");
        pool[7].active = true;
        pool[7].owner = 1;
        assert!(rainbow_spam_survives(&pool, 0), "another owner's boulder");
    }

    /// The 1047 AI gates (`:26087-26109`): ai[1] > 0 counts down and pins
    /// tileCollide false in solids; ai[1] == 0 draws Next(4) then the
    /// owner-gated Next(180).
    #[test]
    fn w4_d3_the_rainbow_boulder_ai_gates() {
        let mut w = world();
        let mut p = proj(RAINBOW_BOULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        p.ai[1] = 5.0;
        let mut rng = ScriptRng::new(&[], &[]);
        ai_boulder(&mut p, &mut w, &[], None, &mut rng, 0, 0, true);
        assert_eq!(p.ai[1], 4.0, "the countdown");
        assert!(rng.draws.is_empty(), "no gates during the flight");
        assert_eq!(rng.float_draws, 1, "the ShimmerBlock settings float");
        // ai[1] == 0: Next(4) = 1 (no bounce), Next(180) = 1 (no wander).
        p.ai[1] = 0.0;
        let mut rng2 = ScriptRng::new(&[1, 1], &[]);
        ai_boulder(&mut p, &mut w, &[], None, &mut rng2, 0, 0, true);
        assert_eq!(rng2.draws, vec![(0, 4), (0, 180)]);
        // A non-local owner skips the Next(180) draw entirely.
        p.owner = 1;
        let mut rng3 = ScriptRng::new(&[1], &[]);
        ai_boulder(&mut p, &mut w, &[], None, &mut rng3, 0, 0, true);
        assert_eq!(rng3.draws, vec![(0, 4)]);
    }

    /// The 1054 cobweb trail (`:26050-26066`): the Next(7) gate draws only
    /// for empty dry cells; a 0 places the cobweb (51).
    #[test]
    fn w4_d3_the_spider_boulder_cobweb_trail() {
        let mut w = world();
        let mut p = proj(SPIDER_BOULDER);
        // A 31×31 box at (1600, 1600): tiles 100..101 both axes (4 cells).
        p.pos = Vec2::new(1600.0, 1600.0);
        p.ai[0] = 1.0;
        let mut rng = ScriptRng::new(&[0, 1, 1, 1], &[]);
        ai_boulder(&mut p, &mut w, &[], None, &mut rng, 0, 0, true);
        assert_eq!(rng.draws, vec![(0, 7); 4], "one gate per empty cell");
        assert!(w.tile(100, 100).unwrap().active(), "the first cell webbed");
        assert_eq!(w.tile(100, 100).unwrap().kind, 51);
        assert!(!w.tile(101, 100).unwrap().active());
    }

    /// The family dust-loop census (`:74938-74948`): 30 Next(2) gates and
    /// the ×1.9 velocity compounding (the verbatim corpse-velocity quirk).
    #[test]
    fn w4_d3_the_family_dust_loop_census() {
        let mut p = proj(BOULDER);
        p.vel = Vec2::new(1.0, 0.0);
        let mut rng = ScriptRng::new(&[0; 30], &[]);
        family_dust_loop(&mut p, 30, 1.9, &mut rng);
        assert_eq!(rng.draws, vec![(0, 2); 30]);
        assert!(
            (p.vel.x / 1.9f32.powi(30) - 1.0).abs() < 1e-4,
            "×1.9 per dust (relative — the f32 accumulation drifts)",
        );
    }

    /// The getGood BoulderExplosion (`:70563-70575`): Next(2,5) = 2 shards,
    /// each two floats; the damage is (int)(damage × 0.75).
    #[test]
    fn w4_d3_the_boulder_explosion_shards() {
        let mut p = proj(BOULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        p.damage = 70;
        let mut rng = ScriptRng::new(&[2], &[0.75, 0.5, 0.25, 0.25]);
        let shards = boulder_explosion(&p, &mut rng);
        assert_eq!(rng.draws, vec![(2, 5)]);
        assert_eq!(rng.float_draws, 4);
        assert_eq!(shards.len(), 2);
        assert_eq!(shards[0].4, 52, "(int)(70 × 0.75)");
        // Shard 0: v = (0.25, −0.25), normalized ×8 = (5.657, −5.657).
        assert!((shards[0].2 - 8.0 / 2.0f32.sqrt()).abs() < 1e-4);
        assert!((shards[0].3 + 8.0 / 2.0f32.sqrt()).abs() < 1e-4);
    }

    /// The 655 bee payload (`:75100-75121`): the three Next(3) bump gates
    /// (all 0 → 5 bees), the cap fold, then per bee the kind + velocity
    /// pair in order.
    #[test]
    fn w4_d3_the_bee_hive_bee_rolls() {
        let mut p = proj(BEE_HIVE);
        p.pos = Vec2::new(1600.0, 1600.0);
        let mut rng = ScriptRng::new(
            &[0, 0, 0, 210, 100, -100, 211, 0, 0, 210, 0, 0, 211, 0, 0, 210, 0, 0],
            &[],
        );
        let bees = bee_hive_bees(&p, 0, &mut rng);
        assert_eq!(bees.len(), 5);
        assert_eq!(
            rng.draws[..9],
            [(0, 3), (0, 3), (0, 3), (210, 212), (-200, 201), (-200, 201),
             (210, 212), (-200, 201), (-200, 201)],
        );
        assert_eq!(rng.draws.len(), 3 + 5 * 3);
        assert_eq!(bees[0].0, 210);
        assert!((bees[0].3 - 0.2).abs() < 1e-6, "100 × 0.002");
        assert!((bees[0].4 + 0.2).abs() < 1e-6);
        assert_eq!((bees[0].1, bees[0].2), (1615.0, 1615.0), "the truncated centre");
        // The cap: 98 live NPCs → only 2 of the 5 spawn.
        let mut rng2 = ScriptRng::new(&[0, 0, 0, 210, 0, 0, 211, 0, 0], &[]);
        assert_eq!(bee_hive_bees(&p, 98, &mut rng2).len(), 2);
    }

    /// The 1047 kill draw census (`:71192-71255`): the seed float, then per
    /// filled ring entry the Next(1,4) count with the threshold decrements
    /// and 3 floats per surviving dust, then the 14 colour floats.
    #[test]
    fn w4_d3_the_rainbow_boulder_death_census() {
        let mut p = proj(RAINBOW_BOULDER);
        // Three filled entries (the port's 10-ring): steps 0, 1, 2 — all
        // below the 0.3×10 = 3 threshold, so no decrements.
        p.old_pos[0] = Vec2::new(1.0, 1.0);
        p.old_pos[1] = Vec2::new(2.0, 2.0);
        p.old_pos[2] = Vec2::new(3.0, 3.0);
        // counts 4, 1, 2 → 7 dust × 3 floats + 1 seed + 14 colour = 36.
        let mut rng = ScriptRng::new(&[4, 1, 2], &[]);
        rainbow_boulder_death_draws(&p, &mut rng);
        assert_eq!(rng.draws, vec![(1, 4), (1, 4), (1, 4)]);
        assert_eq!(rng.float_draws, 1 + 7 * 3 + 14);
        // The threshold decrements: step 3 (≥ 3) loses 1, step 7 (≥ 6.5)
        // loses 3 total, step 9 (≥ 8.5) loses 6 → a 4-count dies at step 9.
        let mut q = proj(RAINBOW_BOULDER);
        for i in 0..10 {
            q.old_pos[i] = Vec2::new(i as f32 + 1.0, 1.0);
        }
        let mut rng2 = ScriptRng::new(&[4; 10], &[]);
        rainbow_boulder_death_draws(&q, &mut rng2);
        // dust counts: 4,4,4,3,3,3,3,1,1,0 → 26 dust × 3 = 78 + 15 = 93.
        assert_eq!(rng2.float_draws, 1 + 26 * 3 + 14);
    }

    /// The lava flood (`:70538-70552`): the radius-4 diamond fills lava on
    /// the empty cells in line of sight; a solid block refuses.
    #[test]
    fn w4_d3_the_lava_boulder_explosion_flood() {
        let mut w = world();
        let mut p = proj(LAVA_BOULDER);
        // Centre (1615.5, 1615.5) → tile (100, 100).
        p.pos = Vec2::new(1600.0, 1600.0);
        lava_boulder_explosion(&p, &mut w);
        let t = w.tile(100, 100).unwrap();
        assert_eq!(t.liquid, 255);
        assert!(t.flags & tr_core::tile::FLAG_LIQUID_LAVA != 0);
        // The radius gate: (104, 100) is distance 4 — NOT < 4, no lava.
        assert_eq!(w.tile(104, 100).unwrap().liquid, 0);
        // (103, 100) is distance 3 — flooded.
        assert_eq!(w.tile(103, 100).unwrap().liquid, 255);
    }

    /// The Poulder pyramid (`DelegateMethods.cs:476-484`): the size-2
    /// pyramid places 666 at the centre and the narrowing rows; the
    /// |x−cx| > 2 cells stay empty.
    #[test]
    fn w4_d3_the_poulder_pyramid_spread() {
        let mut w = world();
        let mut p = proj(POULDER);
        p.pos = Vec2::new(1600.0, 1600.0);
        poulder_spread(&p, &mut w);
        assert!(w.tile(100, 100).unwrap().active(), "the apex");
        assert_eq!(w.tile(100, 100).unwrap().kind, 666);
        // Row +1: dy = |101 − 101| / 2 = 0 → the full ±2 width.
        assert!(w.tile(102, 101).unwrap().active());
        // Row −1: dy = |99 − 101| / 2 = 1 → width 0, only the centre column.
        assert!(w.tile(100, 99).unwrap().active());
        assert!(!w.tile(101, 99).unwrap().active());
        // Beyond the pyramid: (103, 100) — |3| > 2 × (1 − 0.5) = 1.
        assert!(!w.tile(103, 100).unwrap().active());
    }

    /// The hostile-lane set: every boulder except the 1047 hurts the player
    /// (`:1839`/`:1866`/`:7044`/`:9934` hostile; `:9918-9927` friendly-only),
    /// and the 763 spike rides it too (`:7859`).
    #[test]
    fn w4_d3_the_hostile_lane_set() {
        for kind in BOULDER_KINDS {
            assert_eq!(
                hurts_player(kind),
                kind != RAINBOW_BOULDER,
                "kind {kind}",
            );
        }
        assert!(hurts_player(ROLLING_CACTUS_SPIKE));
        assert!(!hurts_player(1)); // a plain arrow stays on the generic lane
    }
}
