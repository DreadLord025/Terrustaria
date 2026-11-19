//! The shared NPC target selection — `NPC.TargetClosest` /
//! `TryTrackingTarget` / `SetTargetTrackingValues` (`NPC.cs:77652-77781`,
//! S1-D-3p). Every aiStyle machine that calls `TargetClosest` in the C#
//! picks its player here: the Manhattan-distance scan with the `aggro`
//! offset, the `npcTypeNoAggro` +1000 penalty, the tank-pet steal, and the
//! `targetRect`/`direction`/`directionY` writes with the `oldTarget`
//! facing-inheritance gate and the `confused` flip.
//!
//! The port has one session player, so the caller passes a slice of
//! [`TargetCandidate`]s indexed by player slot (the C# loops the 255
//! `Main.player` slots, `NPC.cs:77658-77664`; a slot the port does not
//! model is simply absent from the slice). The candidate carries exactly
//! the `Player` fields the three methods read.

use crate::npc::Npc;
use tr_core::World;

/// The `Main.projectile[tankPet]` box the tank arm reads
/// (`NPC.cs:77703-77709`, `:77717-77727`). The rect is captured at pick
/// time — the C# re-reads the projectile in `SetTargetTrackingValues`,
/// but nothing moves it between the two calls in one `TargetClosest`.
#[derive(Clone, Copy, Debug)]
pub struct TankPet {
    /// The projectile slot (`Player.tankPet`, `Player.cs:870`).
    pub index: i32,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

/// One `Main.player[j]` slot as the target scan sees it.
#[derive(Clone, Copy, Debug)]
pub struct TargetCandidate {
    /// `Player.active` (`NPC.cs:77660`).
    pub active: bool,
    /// `Player.dead` (`NPC.cs:77660`, `:77736`).
    pub dead: bool,
    /// `Player.ghost` (`NPC.cs:77660`, `Player.cs:1350`) — the port has no
    /// hardcore ghosts, so the wiring passes false.
    pub ghost: bool,
    /// `Player.position` (`NPC.cs:77735`).
    pub x: f32,
    pub y: f32,
    /// `Player.width`/`height` (the 20×42 hitbox).
    pub width: i32,
    pub height: i32,
    /// `Player.aggro` (`Player.cs:1446`) — the stealth offset
    /// (`NPC.cs:77688`) and the facing-inheritance gate (`:77749`).
    pub aggro: i32,
    /// `Player.itemAnimation` (`NPC.cs:77749`) — the port's raw counter is
    /// `Player.swing_anim`.
    pub item_animation: i32,
    /// `Player.npcTypeNoAggro[this.type]` (`Player.cs:2994`,
    /// `NPC.cs:77689`/`:77701`/`:77740`) — one bool for THIS npc's type.
    pub npc_type_no_aggro: bool,
    /// `Player.tankPet >= 0` (`NPC.cs:77701`) with the projectile's box.
    pub tank_pet: Option<TankPet>,
}

/// The single session player as a slot-0 [`TargetCandidate`] — the
/// S1-D-3p wiring convention shared by the aiStyle machines that call
/// `TargetClosest` (the zombie's inline candidate, `npc.rs`, and the
/// aiStyle-42 arm, `npc_ai_style42.rs`): stock aggro 0 / no-aggro false /
/// no tank pet until those systems land, `itemAnimation` from the raw
/// swing counter.
#[allow(dead_code)] // UNWIRED until the S1-D-3 merge step wires the aiStyle machines.
pub fn session_candidate(player: &crate::player::Player) -> TargetCandidate {
    TargetCandidate {
        active: true,
        dead: player.dead,
        // The port has no hardcore ghosts (`Player.ghost`).
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    }
}

/// `NPC.TargetClosest(bool faceTarget = true)` (`NPC.cs:77652-77666`):
/// scan the player slots, then commit the tracking values. `ai_style` is
/// `this.aiStyle` (the `:77761` fighter arm); the port's kinds carry no
/// general aiStyle table yet, so the caller passes its C# style.
/// `ticker` is `Main.EverLastingTicker` (`Main.cs:413`, `:17177`) for the
/// `targetSetFrame` stamp (`:77780`).
pub fn target_closest(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    face_target: bool,
    ai_style: i32,
    ticker: u64,
) {
    // `:77654-77657` — the scan accumulators.
    let mut distance = 0.0f32;
    let mut real_dist = 0.0f32;
    let mut t = false;
    let mut tank_target = -1i32;
    let mut tank_rect = (0i32, 0i32, 0i32, 0i32);
    // `:77658-77664` — the slot loop, in slot order.
    for (j, p) in players.iter().enumerate() {
        if p.active && !p.dead && !p.ghost {
            try_tracking_target(
                npc,
                world,
                &mut distance,
                &mut real_dist,
                &mut t,
                &mut tank_target,
                &mut tank_rect,
                p,
                j,
            );
        }
    }
    set_target_tracking_values(
        npc,
        players,
        face_target,
        real_dist,
        tank_target,
        tank_rect,
        ai_style,
        ticker,
    );
}

/// `NPC.TryTrackingTarget` (`NPC.cs:77684-77711`): the Manhattan distance
/// minus `aggro`, the `npcTypeNoAggro` +1000 penalty while the NPC already
/// faces somewhere, the strict `<` replacement (first slot wins ties),
/// and the tank-pet steal (distance −200, both `< distance` and `< 200`,
/// plus the 1×1 `Collision.CanHit` centre walk).
#[allow(clippy::too_many_arguments)]
fn try_tracking_target(
    npc: &mut Npc,
    world: &World,
    distance: &mut f32,
    real_dist: &mut f32,
    t: &mut bool,
    tank_target: &mut i32,
    tank_rect: &mut (i32, i32, i32, i32),
    p: &TargetCandidate,
    j: usize,
) {
    let (cx, cy) = npc.center();
    // `Entity.Center` (`Entity.cs:50-55`): position + size/2.
    let center_x = p.x + p.width as f32 / 2.0;
    let center_y = p.y + p.height as f32 / 2.0;
    // `:77687-77692`.
    let num = (center_x - cx).abs() + (center_y - cy).abs();
    let mut num2 = num - p.aggro as f32;
    if p.npc_type_no_aggro && npc.facing != 0 {
        num2 += 1000.0;
    }
    // `:77693-77700`.
    if !*t || num2 < *distance {
        *t = true;
        *tank_target = -1;
        *real_dist = num;
        *distance = num2;
        npc.target = j as i32;
    }
    // `:77701-77710` — the tank pet.
    if let Some(pet) = p.tank_pet {
        if !p.npc_type_no_aggro {
            let pet_cx = pet.x + pet.width as f32 / 2.0;
            let pet_cy = pet.y + pet.height as f32 / 2.0;
            let num3 = (pet_cx - cx).abs() + (pet_cy - cy).abs() - 200.0;
            if num3 < *distance
                && num3 < 200.0
                && crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pet_cx, pet_cy, 1.0, 1.0)
            {
                *tank_target = pet.index;
                *tank_rect = (pet.x as i32, pet.y as i32, pet.width, pet.height);
            }
        }
    }
}

/// `NPC.SetTargetTrackingValues` (`NPC.cs:77713-77781`): the tank rect or
/// the player rect (int-truncated position), the dead/no-aggro facing
/// suppression, the `oldTarget` inheritance gate, the centre facing with
/// the aiStyle-3 bottom-edge `directionY` arm, the `confused` flip, the
/// `netUpdate` change flag and the `targetSetFrame` stamp.
#[allow(clippy::too_many_arguments)]
fn set_target_tracking_values(
    npc: &mut Npc,
    players: &[TargetCandidate],
    mut face_target: bool,
    _real_dist: f32,
    tank_target: i32,
    tank_rect: (i32, i32, i32, i32),
    ai_style: i32,
    ticker: u64,
) {
    let (w, h) = npc.size();
    // The C# facing comparisons use INTEGER half-extents
    // (`(float)(base.width / 2)`, `:77719`/`:77724`/`:77752`/`:77757`).
    let half_w = (w as i32 / 2) as f32;
    let half_h = (h as i32 / 2) as f32;
    if tank_target >= 0 {
        // `:77715-77727` — the tank projectile's rect and centre facing.
        npc.target_rect = tank_rect;
        let (rx, ry, rw, rh) = tank_rect;
        npc.facing = 1;
        if ((rx + rw / 2) as f32) < npc.x + half_w {
            npc.facing = -1;
        }
        npc.direction_y = 1;
        if ((ry + rh / 2) as f32) < npc.y + half_h {
            npc.direction_y = -1;
        }
    } else {
        // `:77731-77734` — the no-player fallback keeps slot 0.
        if npc.target < 0 || npc.target >= 255 {
            npc.target = 0;
        }
        let p = &players[npc.target as usize];
        // `:77735` — `new Rectangle((int)position.X, (int)position.Y, w, h)`.
        npc.target_rect = (p.x as i32, p.y as i32, p.width, p.height);
        // `:77736-77743` — a dead or no-aggro target is tracked, not faced.
        if p.dead {
            face_target = false;
        }
        if p.npc_type_no_aggro && npc.facing != 0 {
            face_target = false;
        }
        if face_target {
            // `:77746-77747` — the C# discards two reads (`_ = aggro`,
            // `_ = (h+w+h+w)/4`); they have no effect.
            // `:77748-77749` — the facing inheritance: a stealthed
            // (aggro < 0), idle (itemAnimation == 0) player that was
            // already the target keeps the NPC's current facing —
            // unless the NPC is a boss.
            let flag = npc.old_target >= 0 && npc.old_target <= 254;
            if !((p.item_animation == 0 && p.aggro < 0) & flag & !npc.boss) {
                // `:77751-77760` — centre facing (ties face +1).
                npc.facing = 1;
                if ((npc.target_rect.0 + npc.target_rect.2 / 2) as f32) < npc.x + half_w {
                    npc.facing = -1;
                }
                npc.direction_y = 1;
                if ((npc.target_rect.1 + npc.target_rect.3 / 2) as f32) < npc.y + half_h {
                    npc.direction_y = -1;
                }
                // `:77761-77768` — the fighters compare BOTTOM edges:
                // `targetRect.Y + Height <= position.Y + height`.
                if ai_style == 3 {
                    npc.direction_y = 1;
                    if ((npc.target_rect.1 + npc.target_rect.3) as f32) <= npc.y + h {
                        npc.direction_y = -1;
                    }
                }
            }
        }
    }
    // `:77772-77775` — the Confused debuff (31) flips the facing. The C#
    // latches `this.confused` from `buffType[i] == 31` in the buff scan
    // (`NPC.cs:92029-92032`); the port reads the buff engine directly.
    if npc.buffs.find_buff_index(31).is_some() {
        npc.facing *= -1;
    }
    // `:77776-77779` — the net-sync change flag, gated on the collision
    // axes (`oldDirection`/`oldDirectionY`/`oldTarget` are latched by
    // `UpdateNPC` before the AI, `NPC.cs:90393-90395`).
    if (npc.facing != npc.old_direction
        || npc.direction_y != npc.old_direction_y
        || npc.target != npc.old_target)
        && !npc.collide_x
        && !npc.collide_y
    {
        npc.net_update = true;
    }
    // `:77780`.
    npc.target_set_frame = ticker;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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

    fn zombie(x: f32, y: f32) -> Npc {
        // 18×40 at (x, y); facing 1, the SetDefaults target 255.
        Npc::new(NpcKind::Zombie, x, y, 1, 0.0)
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// `NPC.cs:77687-77699` — the pick is Manhattan distance minus aggro,
    /// not Euclidean and not a range window: a farther player with a
    /// strong negative-aggro offset (stealth) loses to a nearer one, and
    /// a positive offset can pull the pick to the farther slot.
    #[test]
    fn the_pick_is_manhattan_distance_minus_aggro() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        // Slot 0: dx 300, dy 0 → 300. Slot 1: dx 200, dy 150 → 350.
        let mut near = cand(1300.0 - 10.0, 1000.0 - 21.0);
        let far = cand(1200.0 - 10.0, 1150.0 - 21.0);
        near.aggro = 0;
        target_closest(&mut n, &w, &[near, far], true, 3, 7);
        assert_eq!(n.target, 0, "the 300-Manhattan slot wins");
        // Slot 1 takes aggro 200: 350 - 200 = 150 < 300.
        let mut far2 = far;
        far2.aggro = 200;
        target_closest(&mut n, &w, &[near, far2], true, 3, 7);
        assert_eq!(n.target, 1, "the aggro offset pulls the pick");
        assert_eq!(n.target_set_frame, 7, "the ticker stamp (:77780)");
    }

    /// `NPC.cs:77689-77692` — `npcTypeNoAggro` adds 1000 to the metric
    /// while the NPC faces somewhere, so the other slot wins; and
    /// `:77740-77743` suppresses the facing write for the tracked
    /// no-aggro target.
    #[test]
    fn no_aggro_penalises_the_pick_and_suppresses_facing() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        let mut a = cand(1010.0, 1000.0 - 21.0); // right next door
        a.npc_type_no_aggro = true;
        let b = cand(1500.0, 1000.0 - 21.0); // 500 away
        target_closest(&mut n, &w, &[a, b], true, 3, 0);
        assert_eq!(n.target, 1, "the +1000 penalty hands the pick over");
        // The no-aggro player alone: still tracked, but not faced.
        let mut n2 = zombie(1000.0, 1000.0);
        n2.facing = -1;
        let mut solo = cand(1200.0, 1000.0 - 21.0);
        solo.npc_type_no_aggro = true;
        target_closest(&mut n2, &w, &[solo], true, 3, 0);
        assert_eq!(n2.target, 0);
        assert_eq!(n2.facing, -1, "the facing write is suppressed (:77740)");
        assert_eq!(n2.target_rect, (1200, 979, 20, 42), "the rect is still set");
    }

    /// `NPC.cs:77735`, `:77751-77760` — the rect truncates the position
    /// to ints; the facing compares int centres with the NPC's INTEGER
    /// half-width, and ties face +1.
    #[test]
    fn the_rect_truncates_and_the_facing_uses_int_centres() {
        let w = air();
        let mut n = zombie(1000.5, 1000.0);
        let p = cand(1100.9, 900.7);
        target_closest(&mut n, &w, &[p], true, 0, 0);
        assert_eq!(n.target_rect, (1100, 900, 20, 42));
        assert_eq!(n.facing, 1);
        assert_eq!(n.direction_y, -1, "the player centre is above");
        // Exactly aligned centres: the `<` keeps +1 on both axes.
        let mut n2 = zombie(1000.0, 1000.0);
        // NPC centre (1009, 1020); player centre must equal it.
        let p2 = cand(999.0, 999.0);
        target_closest(&mut n2, &w, &[p2], true, 0, 0);
        assert_eq!(n2.facing, 1, "the x tie faces +1");
        assert_eq!(n2.direction_y, 1, "the y tie faces +1");
    }

    /// `NPC.cs:77761-77768` — aiStyle 3 compares BOTTOM edges with `<=`,
    /// not centres: a player whose centre is below the NPC's centre but
    /// whose feet are level with the NPC's feet still reads as "above".
    #[test]
    fn ai_style_3_direction_y_compares_bottoms() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0); // bottom 1040
                                            // Player bottom 1040 (y 998 + 42), centre 1019 < NPC centre 1020.
        let p = cand(1100.0, 998.0);
        target_closest(&mut n, &w, &[p], true, 3, 0);
        assert_eq!(n.direction_y, -1, "level bottoms read as above (<=)");
        // One px lower: bottom 1041 > 1040 → below.
        let p2 = cand(1100.0, 999.0);
        target_closest(&mut n, &w, &[p2], true, 3, 0);
        assert_eq!(n.direction_y, 1);
        // The same fixture with a non-fighter style keeps the centre rule
        // (centre 1019 < 1020 → -1 even one px lower... centre 1020 = tie).
        let mut n3 = zombie(1000.0, 1000.0);
        target_closest(&mut n3, &w, &[p2], true, 0, 0);
        assert_eq!(n3.direction_y, 1, "the centre tie faces +1");
    }

    /// `NPC.cs:77748-77749` — the oldTarget inheritance: an idle
    /// (itemAnimation == 0), stealthed (aggro < 0) player that was
    /// already the target (oldTarget in 0..=254) keeps the NPC's facing;
    /// a boss ignores the gate.
    #[test]
    fn the_old_target_gate_inherits_facing_unless_boss() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        n.facing = -1;
        n.old_target = 0; // was already targeting slot 0
        let mut p = cand(1200.0, 1000.0 - 21.0); // to the RIGHT
        p.aggro = -100;
        p.item_animation = 0;
        target_closest(&mut n, &w, &[p], true, 3, 0);
        assert_eq!(n.facing, -1, "the inherited facing is kept");
        // A swinging player breaks the gate.
        let mut p2 = p;
        p2.item_animation = 10;
        target_closest(&mut n, &w, &[p2], true, 3, 0);
        assert_eq!(n.facing, 1);
        // A fresh target (oldTarget 255) breaks it too.
        let mut n2 = zombie(1000.0, 1000.0);
        n2.facing = -1;
        n2.old_target = 255;
        target_closest(&mut n2, &w, &[p], true, 3, 0);
        assert_eq!(n2.facing, 1);
        // A boss faces regardless.
        let mut n3 = zombie(1000.0, 1000.0);
        n3.facing = -1;
        n3.old_target = 0;
        n3.boss = true;
        target_closest(&mut n3, &w, &[p], true, 3, 0);
        assert_eq!(n3.facing, 1, "the boss ignores the inheritance");
    }

    /// `NPC.cs:77772-77775` — Confused (buff 31) flips the facing after
    /// every write.
    #[test]
    fn confused_flips_the_facing() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        n.buffs.add_buff(31, 100);
        let p = cand(1200.0, 1000.0 - 21.0);
        target_closest(&mut n, &w, &[p], true, 3, 0);
        assert_eq!(n.facing, -1, "the confused flip mirrors the aim");
    }

    /// `NPC.cs:77701-77709`, `:77715-77727` — the tank pet steals the
    /// target when its (Manhattan − 200) beats the player metric and is
    /// itself < 200 with a clear 1×1 centre walk; the rect and facing
    /// then come from the projectile.
    #[test]
    fn the_tank_pet_steals_the_target_within_range_and_sight() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        let mut p = cand(1300.0, 1000.0 - 21.0); // player 300 away
        p.tank_pet = Some(TankPet {
            index: 5,
            x: 1100.0,
            y: 1010.0,
            width: 10,
            height: 10,
        });
        target_closest(&mut n, &w, &[p], true, 3, 0);
        // pet metric ≈ |1105-1009| + |1015-1020| - 200 = -99 < 300, < 200.
        assert_eq!(n.target, 0, "the player slot stays the target index");
        assert_eq!(n.target_rect, (1100, 1010, 10, 10), "the pet rect wins");
        assert_eq!(n.facing, 1);
        // Out of the 200 window: pet at dx 350 → metric 150+... > 200? no:
        // |1359-1009|+|1015-1020|-200 = 155 < 200 but must also beat the
        // player metric — put the player nearer than the pet metric.
        let mut p2 = cand(1050.0, 1000.0 - 21.0); // player metric ~50
        p2.tank_pet = Some(TankPet {
            index: 5,
            x: 1350.0,
            y: 1010.0,
            width: 10,
            height: 10,
        });
        let mut n2 = zombie(1000.0, 1000.0);
        target_closest(&mut n2, &w, &[p2], true, 3, 0);
        assert_eq!(n2.target_rect.2, 20, "the nearer player keeps the rect");
    }

    /// `NPC.cs:77706` — the `Collision.CanHit` centre walk blocks the
    /// tank steal through a wall.
    #[test]
    fn the_tank_steal_needs_line_of_sight() {
        let mut w = air();
        // A solid column at tile x 66 (px 1056..1072) between the NPC
        // (centre 1009) and the pet (centre ~1105), across the centre row.
        for y in 55..70 {
            let i = w.index(66, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut n = zombie(1000.0, 1000.0);
        let mut p = cand(1300.0, 1000.0 - 21.0);
        p.tank_pet = Some(TankPet {
            index: 5,
            x: 1100.0,
            y: 1010.0,
            width: 10,
            height: 10,
        });
        target_closest(&mut n, &w, &[p], true, 3, 0);
        assert_eq!(
            n.target_rect,
            (1300, 979, 20, 42),
            "the wall keeps the player"
        );
    }

    /// `NPC.cs:77736-77739`, `:77731-77734` — a dead picked player is
    /// tracked without facing; with no live candidates the target falls
    /// back to slot 0.
    #[test]
    fn dead_and_absent_players_fall_back_without_facing() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        n.facing = -1;
        let mut p = cand(1200.0, 1000.0 - 21.0);
        p.dead = true;
        target_closest(&mut n, &w, &[p], true, 3, 0);
        assert_eq!(n.target, 0, "the fallback slot (:77731-77734)");
        assert_eq!(n.facing, -1, "a dead target is not faced");
        assert_eq!(n.target_rect, (1200, 979, 20, 42));
    }

    /// `NPC.cs:77776-77779` — netUpdate arms on a target/facing change
    /// only while neither collision axis is blocked.
    #[test]
    fn net_update_arms_on_change_unless_colliding() {
        let w = air();
        let mut n = zombie(1000.0, 1000.0);
        n.old_target = 255;
        n.old_direction = n.facing;
        n.old_direction_y = n.direction_y;
        let p = cand(1200.0, 1000.0 - 21.0);
        target_closest(&mut n, &w, &[p], true, 3, 0);
        assert!(n.net_update, "the target changed 255 -> 0");
        // Steady state: no change, no flag.
        let mut n2 = zombie(1000.0, 1000.0);
        target_closest(&mut n2, &w, &[p], true, 3, 0); // settle
        n2.old_target = n2.target;
        n2.old_direction = n2.facing;
        n2.old_direction_y = n2.direction_y;
        n2.net_update = false;
        target_closest(&mut n2, &w, &[p], true, 3, 0);
        assert!(!n2.net_update);
        // A change while collideX holds: suppressed.
        let mut n3 = zombie(1000.0, 1000.0);
        n3.old_target = 255;
        n3.collide_x = true;
        target_closest(&mut n3, &w, &[p], true, 3, 0);
        assert!(!n3.net_update, "collideX suppresses the flag");
    }

    /// The wiring proof (S1-D-3p): `update_zombie` runs the shared
    /// selection — the zombie's target/rect/facing come from
    /// `TargetClosest`, not the ad-hoc dx sign.
    #[test]
    fn the_zombie_wires_the_shared_selection() {
        let mut w = World::new("t".into(), 200, 120);
        // Night: the daytime despawn-discouragement gate (`NPC.cs:57336`)
        // must not silence the chase — the zombie is a night kind.
        w.day_time = false;
        for x in 0..200 {
            let i = w.index(x, 70);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.spawn_x = 60;
        w.spawn_y = 60;
        w.compute_column_surface();
        let mut player = crate::player::Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut z = zombie(1300.0, 70.0 * 16.0 - 40.0);
        z.update(&w, &mut player, 1.0 / 60.0);
        assert_eq!(z.target, 0);
        assert_eq!(
            z.target_rect,
            (player.x as i32, player.y as i32, 20, 42),
            "the rect tracks the session player"
        );
        assert_eq!(z.facing, -1, "the player is to the left");
        assert_eq!(z.sprite_dir, z.facing);
    }
}
