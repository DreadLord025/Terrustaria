//! The mid-game housing relocation driver (W4-d119) — the `WorldGen.UpdateWorld`
//! tail that re-homes a town NPC when its house is lost, plus the eviction
//! predicate. Three C# functions, in the `UpdateWorld` order (`WorldGen.cs:71495`):
//!
//! * [`update_prioritized_town_npc`] — `WorldGen.UpdatePrioritizedTownNPC`
//!   (`:71627-71641`): the first homeless town NPC whose `lookForHomeTimeout`
//!   has expired becomes `WorldGen.prioritizedTownNPCType`. RNG-free.
//! * [`check_for_houses_near_a_player`] — `WorldGen.CheckForHousesNearAPlayer`
//!   (`:71643-71674`): 300 random house-wall picks around the active player;
//!   the first pick runs `SpawnTownNPC(x, y, canSpawnNewTownNPC: false)` — the
//!   RELOCATION-ONLY arm (it moves an existing homeless NPC in, never spawns a
//!   new one). The picks ride `Main.rand` under the `UpdateWorldGen` swap
//!   (`Main.cs:17802`), i.e. the `UpdateWorldGen` named stream.
//! * [`is_npc_evictable`] — `WorldGen.IsNPCEvictable` (`:4857-4867`): the
//!   house-banner right-click eviction gate (`Main.cs:40824-40829`). RNG-free.
//!
//! The home-LOSS half is `Main.UpdateTime_SpawnTownNPCs`'s `QuickFindHome(k)`
//! call on every housed resident (`Main.cs:65611-65613`), wired in
//! `world_sim/npcs.rs`; the `kickOut`/`moveRoom` primitives live in `town.rs`.
//!
//! RNG census (the whole driver): `UpdatePrioritizedTownNPC` and
//! `IsNPCEvictable` draw NOTHING; `CheckForHousesNearAPlayer` draws exactly two
//! `Main.rand.Next` per pick (`Utils.NextFromRectangle`, `Utils.cs:1246-1249`)
//! and `SpawnTownNPC`'s relocation arm draws nothing (`WorldGen.cs:5409-5463`,
//! `canSpawnNewTownNPC: false` short-circuits the spawn tail). The
//! `SpawnHomelessNPC`/`UnspawnHomelessNPC` pair (`Main.cs:65043-65062`,
//! `WorldGen.cs:5036-5249`) is the NEW-NPC homeless arrival/departure — out of
//! this relocation-only lane (recorded in ANALYSIS).

use tr_core::housing::{check_room, room_needs, score_room_report, RoomScan};
use tr_core::world::World;
use tr_core::UnifiedRandom;

use crate::npc::Npc;
use crate::town::{room_occupancy, TownRooms};
use crate::town_arrival::ArrivalState;

/// `WorldGen.UpdatePrioritizedTownNPC` (`WorldGen.cs:71627-71641`): unless the
/// prioritized type is the Old Man (37 — no ported kind, so the early return
/// never fires), the first live homeless town NPC with `lookForHomeTimeout == 0`
/// and a type other than the Travelling Merchant (368) / Truffle (160) becomes
/// `WorldGen.prioritizedTownNPCType`. The C# writes the static every call; the
/// port parks the kind on [`ArrivalState::prioritized`].
///
/// The port's `is_town_npc` set is the housing subset (the ten kinds the room
/// scan can place); the C#'s `townNPC` flag is wider, but the excluded sellers
/// (Angler, Wizard, …) are not homeless-relocatable until their own housing
/// arms land, so the narrower set is the reachable one. RNG-free.
pub(crate) fn update_prioritized_town_npc(npcs: &[Npc], state: &mut ArrivalState) {
    // `:71629-71632`: `prioritizedTownNPCType == 37` returns. The port has no
    // Old Man kind, so `prioritized` is never 37 — the guard is vacuous.
    for n in npcs {
        // `:71635`: active && homeless && townNPC && timeout == 0 && type
        // != 368 && type != 160. The live list holds only active NPCs.
        if n.kind.is_town_npc()
            && n.home.is_none()
            && n.look_for_home_timeout == 0
            && !matches!(n.kind.net_id(), 368 | 160)
        {
            state.prioritized = Some(n.kind);
            return;
        }
    }
}

/// One `SpawnTownNPC(x, y, canSpawnNewTownNPC: false)` relocation attempt
/// (`WorldGen.cs:5409-5463`): the room check + furniture + stinkbug + score
/// gates, then `FindNPCLookingForHomeThatCanMoveIn` moves an existing homeless
/// NPC of the prioritized type into the scored spot. With
/// `canSpawnNewTownNPC: false` the new-spawn tail (`:5468-5611`) is unreachable,
/// so this returns only `RelocatedHomeless` or `Blocked`/`FoundHouseNoSpawn`.
///
/// The C#'s `WorldGen.canSpawn` gate (`:5411-5418`) is satisfied by the caller's
/// house-wall pick (`Main.wallHouse[wall]` sets it true); the port folds that
/// into the caller's `wall_house` probe, so the room check runs directly.
/// Returns the relocated NPC's index, or `None`.
fn spawn_town_npc_relocate(
    world: &World,
    npcs: &mut [Npc],
    rooms: &mut TownRooms,
    state: &mut ArrivalState,
    x: i32,
    y: i32,
    unlocked_truffle_spawn: bool,
) -> Option<usize> {
    // `:5419-5426`: StartRoomCheck + RoomNeeds.
    let scan: RoomScan = check_room(world, x, y).ok()?;
    if !room_needs(&scan.house_tiles) {
        return None;
    }
    // `:5427-5428`: the prioritized type + the homeless NPC that can move in.
    let kind = state.prioritized?;
    let net_id = kind.net_id();
    let mover = crate::town::find_npc_looking_for_home_that_can_move_in(
        world,
        &scan,
        npcs,
        net_id,
        unlocked_truffle_spawn,
    )?;
    // `:5430-5434`: the stinkbug gates — a town pet with a stored room is
    // exempt. The port's housing set has no pet kind, so `flag` is false and
    // any stinkbug blocks.
    let pet_with_room =
        crate::town::is_town_pet(net_id) && rooms.has_room(net_id).is_some();
    if (scan.has_stinkbug || scan.has_echo_stinkbug) && !pet_with_room {
        return None;
    }
    // `:5435-5439`: ScoreRoom(-1, type) — no ignoreNPC, so the mover's own
    // (homeless) home does not occupy; the occupancy scan skips homeless NPCs.
    let (occupied, shared_room_x) = room_occupancy(&scan, npcs, None, net_id);
    let report = score_room_report(world, &scan, occupied, shared_room_x);
    if report.hi_score <= 0 {
        return None;
    }
    let (bx, by) = report.best?;
    // `:5453-5463`: the relocation arm — home = bestX/bestY, homeless cleared,
    // homelessDespawn cleared, NO_HOBO (8), prioritized zeroed. The caller
    // fires the progression event (it needs `&mut App`).
    npcs[mover].home = Some((bx, by));
    npcs[mover].homeless_despawn = false;
    rooms.set_room(net_id, bx, by);
    state.prioritized = None;
    Some(mover)
}

/// `WorldGen.CheckForHousesNearAPlayer` (`WorldGen.cs:71643-71674`): for the
/// one active player the C# selects this tick (`GameUpdateCount %
/// ActivePlayersCount`; the port has a single player, so it runs every tick),
/// build the view rectangle around the player's centre tile, inflate it by
/// `MaxWorldViewSize / 32` (1920/32 = 60, 1200/32 = 37 — `Main.cs:66714`),
/// clamp it to the world with a 10-tile fluff, then make 300 random picks. The
/// first pick whose wall is a house wall runs the relocation-only
/// `SpawnTownNPC` and breaks.
///
/// The picks draw `Main.rand.Next(rect.Left, rect.Right)` then
/// `Next(rect.Top, rect.Bottom)` (`Utils.NextFromRectangle`, `Utils.cs:1248`)
/// on the `UpdateWorldGen` stream — exactly two draws per pick, up to 300.
/// Returns the relocated NPC's index (the caller fires NO_HOBO), or `None`.
pub(crate) fn check_for_houses_near_a_player(
    world: &World,
    npcs: &mut [Npc],
    rooms: &mut TownRooms,
    state: &mut ArrivalState,
    player_center: Option<(f32, f32)>,
    unlocked_truffle_spawn: bool,
    rng: &mut UnifiedRandom,
) -> Option<usize> {
    let (cx, cy) = player_center?;
    // `:71658`: `Utils.CenteredRectangle(player.Center.ToTileCoordinates(),
    // Point.Zero)` — a 0×0 rectangle at the centre tile. `ToTileCoordinates`
    // is `(int)x >> 4` (`Utils.cs:1809`), an arithmetic shift (floor for
    // negatives), unlike the port's usual `as i32` truncation.
    let tx = (cx as i32) >> 4;
    let ty = (cy as i32) >> 4;
    // `:71659`: inflate by MaxWorldViewSize/32 = (60, 37).
    let (left, top, right, bottom) = (tx - 60, ty - 37, tx + 60, ty + 37);
    // `:71660`: `WorldUtils.ClampToWorld(rect, 10)` (`WorldBuilding/
    // WorldUtils.cs:9-14`) — each edge clamps into `[10, max - 10]`.
    let (wx, wy) = (world.width, world.height);
    let clamp = |v: i32, max: i32| v.max(10).min(max - 10);
    let left = clamp(left, wx);
    let top = clamp(top, wy);
    let right = clamp(right, wx);
    let bottom = clamp(bottom, wy);
    // `:71661-71672`: 300 picks; the first house-wall pick runs the relocation.
    // A degenerate rectangle (right <= left or bottom <= top, a tiny fixture
    // world) would ask `Next(min, max)` for a reversed band — the C# would
    // throw; the faithful reading is "no pick found a cell", so skip.
    if right <= left || bottom <= top {
        return None;
    }
    for _ in 0..300 {
        let px = rng.next_range(left, right);
        let py = rng.next_range(top, bottom);
        let wall = world.tile(px, py).map(|t| t.wall).unwrap_or(0);
        if tr_core::edit::wall_house(wall) {
            return spawn_town_npc_relocate(
                world,
                npcs,
                rooms,
                state,
                px,
                py,
                unlocked_truffle_spawn,
            );
        }
    }
    None
}

/// `WorldGen.IsNPCEvictable` (`WorldGen.cs:4857-4867`): the house-banner
/// eviction gate — true when the local player's hitbox, inflated by half the
/// max world view size (960, 600), intersects the NPC's 16×16 home tile. A
/// negative index is never evictable. RNG-free.
///
/// `player_hitbox` is `(x, y, width, height)`; `home` is the NPC's
/// `(homeTileX, homeTileY)`. The C# reads `Main.npc[n].homeTileX/Y` even for a
/// homeless NPC (the banner draw only shows housed NPCs, so the gate is only
/// reached for them); the port passes the home explicitly.
pub(crate) fn is_npc_evictable(
    n: i32,
    player_hitbox: Option<(f32, f32, f32, f32)>,
    home: Option<(i32, i32)>,
) -> bool {
    if n < 0 {
        return false; // `:4859-4862`
    }
    let Some((px, py, pw, ph)) = player_hitbox else {
        return false;
    };
    let Some((hx, hy)) = home else {
        return false;
    };
    // `:4863-4864`: `hitbox.Inflate(MaxWorldViewSize.X / 2, .Y / 2)`.
    let (ix, iy, iw, ih) = (px - 960.0, py - 600.0, pw + 1920.0, ph + 1200.0);
    // `:4866`: the 16×16 home-tile rectangle.
    let (bx, by) = (hx as f32 * 16.0, hy as f32 * 16.0);
    ix < bx + 16.0 && ix + iw > bx && iy < by + 16.0 && iy + ih > by
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::test_world_helpers::{build_furnished_room, ground};

    fn guide(x: f32, y: f32, home: Option<(i32, i32)>) -> Npc {
        let mut n = Npc::new(NpcKind::Guide, x, y, 1, 0.0);
        n.home = home;
        n.hp = n.max_hp;
        n
    }

    /// `UpdatePrioritizedTownNPC` (`WorldGen.cs:71633-71640`): the FIRST live
    /// homeless town NPC with an expired timer becomes the prioritized type;
    /// a housed NPC, a non-zero timer and the 368/160 exclusions all skip.
    #[test]
    fn the_prioritized_type_is_the_first_homeless_town_npc() {
        let mut state = ArrivalState::new();
        // A housed Guide is not prioritized.
        let housed = guide(0.0, 0.0, Some((60, 100)));
        update_prioritized_town_npc(&[housed], &mut state);
        assert_eq!(state.prioritized, None);
        // A homeless Guide with a live timer is gated (`:71635` timeout == 0).
        let mut gated = guide(0.0, 0.0, None);
        gated.look_for_home_timeout = 5;
        update_prioritized_town_npc(&[gated], &mut state);
        assert_eq!(state.prioritized, None);
        // The homeless Guide with an expired timer wins.
        let ready = guide(0.0, 0.0, None);
        update_prioritized_town_npc(&[ready], &mut state);
        assert_eq!(state.prioritized, Some(NpcKind::Guide));
        // First-in-list wins: a Merchant ahead of the Guide takes the slot.
        let merchant = {
            let mut n = Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0);
            n.home = None;
            n.hp = n.max_hp;
            n
        };
        let mut state2 = ArrivalState::new();
        update_prioritized_town_npc(&[merchant, guide(0.0, 0.0, None)], &mut state2);
        assert_eq!(state2.prioritized, Some(NpcKind::Merchant));
    }

    /// The Travelling Merchant (368) is excluded from the prioritized scan
    /// (`WorldGen.cs:71635`), and a non-town NPC never qualifies.
    #[test]
    fn the_travelling_merchant_and_non_town_npcs_are_excluded() {
        let mut state = ArrivalState::new();
        let mut traveller = Npc::new(NpcKind::TravellingMerchant, 0.0, 0.0, 1, 0.0);
        traveller.home = None;
        traveller.hp = traveller.max_hp;
        let slime = Npc::new(NpcKind::Slime, 0.0, 0.0, 1, 0.0);
        update_prioritized_town_npc(&[traveller, slime], &mut state);
        assert_eq!(state.prioritized, None, "368 and the slime both skip");
    }

    /// `CheckForHousesNearAPlayer` (`WorldGen.cs:71661-71672`): a house-wall
    /// pick runs the relocation-only `SpawnTownNPC` — a homeless Guide of the
    /// prioritized type moves into the scored spot, the room registers and the
    /// prioritized slot zeroes (`:5455-5461`).
    #[test]
    fn a_house_wall_pick_relocates_the_homeless_guide() {
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        // The player stands inside the room so the view rect covers it.
        let center = (66.0 * 16.0, 104.0 * 16.0);
        let mut npcs = vec![guide(66.0 * 16.0, 104.0 * 16.0, None)];
        let mut rooms = TownRooms::new();
        let mut state = ArrivalState::new();
        state.prioritized = Some(NpcKind::Guide);
        let mut rng = UnifiedRandom::new(1234);
        let moved = check_for_houses_near_a_player(
            &w,
            &mut npcs,
            &mut rooms,
            &mut state,
            Some(center),
            false,
            &mut rng,
        );
        assert!(moved.is_some(), "a house-wall pick relocates the Guide");
        let i = moved.unwrap();
        assert!(npcs[i].home.is_some(), "the Guide is housed");
        let (hx, hy) = npcs[i].home.unwrap();
        assert!((60..=71).contains(&hx) && (100..=108).contains(&hy));
        assert!(!npcs[i].homeless_despawn);
        assert_eq!(rooms.has_room(22), Some((hx, hy)), "the room registers");
        assert_eq!(state.prioritized, None, "the prioritized slot zeroes");
    }

    /// With no prioritized type the pick finds the house but relocates nobody
    /// (`SpawnTownNPC`'s `FindNPCLookingForHomeThatCanMoveIn(0)` returns −1,
    /// `WorldGen.cs:5428`), and the stream still advanced by the picks.
    #[test]
    fn no_prioritized_type_means_no_relocation() {
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = vec![guide(66.0 * 16.0, 104.0 * 16.0, None)];
        let mut rooms = TownRooms::new();
        let mut state = ArrivalState::new(); // prioritized = None
        let mut rng = UnifiedRandom::new(1234);
        let before = rng.clone();
        let moved = check_for_houses_near_a_player(
            &w,
            &mut npcs,
            &mut rooms,
            &mut state,
            Some((66.0 * 16.0, 104.0 * 16.0)),
            false,
            &mut rng,
        );
        assert!(moved.is_none());
        assert!(npcs[0].home.is_none(), "the Guide stays homeless");
        assert_ne!(rng, before, "the picks still drew");
    }

    /// A player far from any house wall: 300 picks, two draws each, no
    /// relocation (`WorldGen.cs:71661-71672`). The stream advanced by exactly
    /// 600 draws — the RNG census pin for the no-house arm.
    #[test]
    fn the_no_house_arm_draws_exactly_six_hundred() {
        let w = ground(); // no house walls anywhere
        let mut npcs = vec![guide(66.0 * 16.0, 104.0 * 16.0, None)];
        let mut rooms = TownRooms::new();
        let mut state = ArrivalState::new();
        state.prioritized = Some(NpcKind::Guide);
        let mut rng = UnifiedRandom::new(7);
        let moved = check_for_houses_near_a_player(
            &w,
            &mut npcs,
            &mut rooms,
            &mut state,
            Some((66.0 * 16.0, 104.0 * 16.0)),
            false,
            &mut rng,
        );
        assert!(moved.is_none());
        // Twin: 300 picks × Next(left,right) + Next(top,bottom).
        let mut twin = UnifiedRandom::new(7);
        for _ in 0..300 {
            let _ = twin.next_range(10, w.width - 10);
            let _ = twin.next_range(10, w.height - 10);
        }
        assert_eq!(rng, twin, "exactly 600 draws, no more, no fewer");
    }

    /// `IsNPCEvictable` (`WorldGen.cs:4857-4867`): the inflated player hitbox
    /// (±960/±600) must intersect the 16×16 home tile. A negative index, a
    /// missing player or a missing home is never evictable.
    #[test]
    fn the_eviction_gate_is_the_inflated_hitbox_intersection() {
        let home = Some((100, 100)); // tile rect (1600,1600,16,16)
        let near = Some((1600.0, 1600.0, 20.0, 40.0));
        assert!(is_npc_evictable(0, near, home));
        // Far outside the ±960/±600 inflation.
        let far = Some((5000.0, 5000.0, 20.0, 40.0));
        assert!(!is_npc_evictable(0, far, home));
        assert!(!is_npc_evictable(-1, near, home), "negative index");
        assert!(!is_npc_evictable(0, None, home), "no player");
        assert!(!is_npc_evictable(0, near, None), "no home");
        // Just inside the inflation edge: the player 950px left of the home.
        let edge = Some((1600.0 - 950.0, 1600.0, 20.0, 40.0));
        assert!(is_npc_evictable(0, edge, home));
    }
}
