//! W4-d119 phase 2 — the homeless arrival pair and the town-spawn machinery.
//!
//! The d119 relocation lane (`town_rehome.rs`) ported the RELOCATION-ONLY arm
//! of `WorldGen.SpawnTownNPC` (`WorldGen.cs:5409-5463`) and explicitly parked
//! this pair as out of scope; the Lead's follow-up puts it in scope:
//!
//! * [`spawn_homeless_npc`] — `WorldGen.SpawnHomelessNPC`
//!   (`WorldGen.cs:5036-5204`): the prioritized town type arrives at the world
//!   edge near a resident's home when no house is free, `homeless` +
//!   `homelessDespawn` armed, then one `QuickFindHome` pass. Called from
//!   `Main.UpdateTime` (`Main.cs:65059`) on the `UpdateTime` stream.
//! * [`unspawn_homeless_npc`] — `WorldGen.UnspawnHomelessNPC`
//!   (`WorldGen.cs:5206-5249`): the first `homelessDespawn` NPC leaves at
//!   night once no player stands in its safe-range box (`Main.cs:65047`).
//! * [`spawn_town_npc`] — the FULL `WorldGen.SpawnTownNPC`
//!   (`WorldGen.cs:5409-5615`): the relocation arm PLUS the new-spawn tail
//!   (the stored-room alternate spot, the `IsRoomConsideredAlreadyOccupied`
//!   gate, the player safe-range scan, the 500-step surface spiral, `NewNPC`,
//!   the arrival chat/NO_HOBO/unlock switch and the `LastFoundHouse` write).
//! * [`is_there_a_spawnable_prioritized_town_npc`] —
//!   `WorldGen.IsThereASpawnablePrioritizedTownNPC` (`WorldGen.cs:4881-4917`).
//! * [`try_spawning_town_npc_gate`] — `WorldGen.TrySpawningTownNPC`
//!   (`WorldGen.cs:74869-74880`): the per-tile-update arrival driver the
//!   growth walkers call (`:72497` overground / `:73807` underground, the
//!   inactive-tile arms). WIRED (W4-d119 escalation 1) through the tr-core
//!   walker seam [`tr_core::growth::try_spawning_town_npc`]: the pure
//!   walkers push `(x, y)` spawn-requests onto a shared queue and
//!   [`drain_town_spawn_requests`] runs the full `SpawnTownNPC` at the
//!   `UpdateWorldGen` slot (`world_sim/growth.rs`).
//!
//! RNG census (exact slots):
//! * `SpawnHomelessNPC` (`UpdateTime` stream — `Main.cs:65043-65062` runs
//!   inside `Main.UpdateTime` under the `UpdateTime` swap, `Main.cs:17795`):
//!   the cadence roll `Next(27000/dayRate)` (`Main.cs:65057`, caller), the
//!   house pick `Next(num2)` (`:5093`), up to 30 roofline picks
//!   `Next(minValue, num4+1)` (`:5109`, first accepted pick breaks), and the
//!   chat-variant `Next(6)` (`:5200`). The gates, the roofline walks, the
//!   surface spiral and `QuickFindHome` draw NOTHING. Full homeless arm:
//!   1 + k (1 <= k <= 30) + 1 draws, plus the caller's cadence roll.
//! * `UnspawnHomelessNPC`: ZERO draws.
//! * `SpawnTownNPC`: ZERO draws of its own. `NewNPC`'s naming
//!   (`GiveTownUniqueDataToNPCsThatNeedIt`, `NPC.cs:80839-80856`) draws on
//!   `WorldGen.genRand` (== the active swapped `Main.rand`) + the profile
//!   variation on `Main.rand` — the W4-f38 naming lane owns that seam
//!   (`npc_names::give_town_unique_data`, pushed UNWIRED); the port keeps
//!   `given_name` empty here exactly like the landed arrival paths.
//!   `NewNPC`'s `getGoodWorld` `Next(3)` gate (`NPC.cs:80752`) is
//!   constants-off (no zenith-seed `NewNPC` path ported).
//! * `TrySpawningTownNPC`: ONE `Next(4)` draw, and only when the picked
//!   tile's wall is 34 (blue dungeon wall) AND the delay gate passed
//!   (`WorldGen.cs:74874`), on the `UpdateWorldGen` stream.
//! * `IsThereASpawnablePrioritizedTownNPC`: ZERO draws.
//! * The `Main.tileSolid[379]` windows (`:5061-5075`, `:5734`, `:74876`)
//!   need no toggle: the generated `TILE_SOLID` base table already carries
//!   379 (the d118 candelabra witness pins it).

use tr_core::housing::{check_room, room_needs, score_room_report, RoomScan};
use tr_core::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use tr_core::world::World;
use tr_core::UnifiedRandom;

use crate::npc::{Npc, NpcKind};
use crate::town::{self, room_occupancy, TownRooms};
use crate::town_arrival::ArrivalState;

/// `WorldGen.homelessSpawnTimeout` / `LastFoundHouse` / `npcSpawnDelay` /
/// `npcSpawnPeriod` / `canSpawn` / `currentlyTryingToUseAlternateHousingSpot`
/// (`WorldGen.cs:4244`/`:4320`/`:4238`/`:4240`/`:4282`/`:4322`) — the six
/// statics the spawn machinery carries between calls. `clearWorld`
/// (`WorldGen.cs:7025-7026`) resets `LastFoundHouse` to `Point.Zero` and
/// arms the timeout to 54000 (a 15-minute day at 60 Hz).
#[derive(Debug)]
pub(crate) struct TownSpawnStatics {
    /// `WorldGen.homelessSpawnTimeout` — decremented once per
    /// `WorldGen.UpdateWorld` call while positive (`:71545-71548`).
    pub(crate) homeless_spawn_timeout: i32,
    /// `WorldGen.LastFoundHouse` — the last `SpawnTownNPC` call that found a
    /// house but spawned nobody (`:5613`); `Point.Zero` at world clear.
    pub(crate) last_found_house: (i32, i32),
    /// `WorldGen.npcSpawnDelay` (`:4238`) — `TrySpawningTownNPC`'s counter.
    pub(crate) npc_spawn_delay: i32,
    /// `WorldGen.npcSpawnPeriod` (`:4240`) — `20 * GetWorldUpdateRate()`,
    /// re-armed every `UpdateWorld` head (`:71549`).
    pub(crate) npc_spawn_period: i32,
    /// `WorldGen.canSpawn` (`:4282`) — the sticky room-check flag.
    pub(crate) wg_can_spawn: bool,
    /// `WorldGen.currentlyTryingToUseAlternateHousingSpot` (`:4322`) — the
    /// stored-room recursion guard (`:5470-5477`).
    pub(crate) alternate_spot: bool,
}

impl Default for TownSpawnStatics {
    fn default() -> Self {
        Self {
            // `WorldGen.clearWorld` (`WorldGen.cs:7025-7026`).
            homeless_spawn_timeout: 54000,
            last_found_house: (0, 0),
            npc_spawn_delay: 0,
            npc_spawn_period: 0,
            wg_can_spawn: false,
            alternate_spot: false,
        }
    }
}

/// `TownNPCSpawnResult` (`Enums/TownNPCSpawnResult.cs:3-10`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TownNpcSpawnResult {
    Blocked,
    Successful,
    RelocatedHomeless,
    BlockedInfiHousing,
    /// The fixed-200-slot cap (`WorldGen.cs:5555-5558`) — the port's live
    /// list is a Vec, so the arm is unreachable (kept for the 1:1 shape).
    #[allow(dead_code)]
    BlockedTooManyNPCs,
    FoundHouseNoSpawn,
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`, `noDoors =
/// false`): a null tile is SOLID; the cell is solid when
/// `active && tileSolid && !tileSolidTop && !halfBrick && slope == 0 &&
/// !inActive`. The C# out-of-bounds indexer throws into the catch (false);
/// the port's dense grid returns `None` only out of bounds, so `None` maps
/// to the throw arm (the null arm is unreachable in the port).
fn worldgen_solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): the edge guards block, a null cell returns
/// false, a cell blocks on `active && !inActive && tileSolid && !tileSolidTop`
/// (the d118 verbatim shape, `housing::solid_tiles_in_box`'s twin — d118
/// rewrote that fn at its tip, so the merge keeps d118's housing.rs copy and
/// NEVER takes g44's housing.rs wholesale; research/87 RETIRE §5 blob census,
/// consumed receipt in research/304 §3).
fn collision_solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for x in x0..=x1 {
        for y in y0..=y1 {
            let Some(t) = world.tile(x, y) else {
                return false;
            };
            if t.active() && !t.inactive() && TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind)
            {
                return true;
            }
        }
    }
    false
}

/// `NPC.sWidth/sHeight` (`NPC.cs:6767-6769`) + `safeRangeX/Y`
/// (`NPC.cs:935-936`: `(int)(sWidth/16 * 0.52)` = 62, 39).
const SAFE_X: i32 = 62;
const SAFE_Y: i32 = 39;

/// The safe-range rectangle around a tile centre (`WorldGen.cs:5490`,
/// `:5160`, `:5233`): `new Rectangle(tx*16 + 8 - sWidth/2 - safeRangeX,
/// ty*16 + 8 - sHeight/2 - safeRangeY, sWidth + safeRangeX*2, sHeight +
/// safeRangeY*2)`.
fn safe_range_rect(tx: i32, ty: i32) -> (f32, f32, f32, f32) {
    (
        (tx * 16 + 8 - 960 - SAFE_X) as f32,
        (ty * 16 + 8 - 600 - SAFE_Y) as f32,
        (1920 + SAFE_X * 2) as f32,
        (1200 + SAFE_Y * 2) as f32,
    )
}

/// `Rectangle.Intersects` against the one active player's hitbox
/// (`(int)position.X, (int)position.Y, width, height` — the C# truncates).
fn player_in_rect(player: Option<(f32, f32, f32, f32)>, r: (f32, f32, f32, f32)) -> bool {
    let Some((px, py, pw, ph)) = player else {
        return false;
    };
    let (px, py) = (px as i32 as f32, py as i32 as f32);
    let (rx, ry, rw, rh) = r;
    px < rx + rw && px + pw > rx && py < ry + rh && py + ph > ry
}

/// `NPC.AnyNPCs(type)` — the port's liveness is `hp > 0` on the live list.
fn any_npcs(npcs: &[Npc], net_id: i32) -> bool {
    npcs.iter().any(|n| n.hp > 0 && n.kind.net_id() == net_id)
}

/// `WorldGen.IsThereASpawnablePrioritizedTownNPC` (`WorldGen.cs:4881-4917`):
/// first the room's registered occupants (`TownManager.AddOccupantsToList`,
/// `:4884-4892`) — the first occupant type whose `canSpawn` entry is open,
/// with no resident and passing the special conditions wins. Then the
/// type-order walk (`:4894-4915`): a resident CLEARS the `canSpawn` entry;
/// `HasRoomQuick` or `IsTownPet` returns at once; the prioritized type is
/// only the fallback result. Returns the vanilla type id (the C#'s `num`,
/// `-1` → `None`). ZERO draws.
///
/// The port's `can_spawn` table is LADDER-indexed, so the type-order walk
/// visits the modelled kinds sorted by net id (unmodelled ids carry no
/// entry — the C# table's false rows).
pub(crate) fn is_there_a_spawnable_prioritized_town_npc(
    world: &World,
    scan: &RoomScan,
    state: &mut ArrivalState,
    rooms: &TownRooms,
    npcs: &[Npc],
    best: (i32, i32),
    unlocked_truffle_spawn: bool,
) -> Option<i32> {
    let special = |id: i32| {
        tr_core::housing::check_special_town_npc_spawning_conditions(
            world,
            scan,
            id,
            unlocked_truffle_spawn,
        )
    };
    // `:4884-4892`: the occupants at the scored spot, in registration order.
    for id in rooms.occupants_at(best) {
        let kind = crate::npc::kind_from_net_id(id).map(|(k, _)| k);
        let open = kind.is_some_and(|k| state.can_spawn(k));
        if open && !any_npcs(npcs, id) && special(id) {
            return Some(id);
        }
    }
    // `:4894-4915`: the type-order walk over the modelled entries.
    let mut ladder: Vec<NpcKind> = crate::town_arrival::LADDER.to_vec();
    ladder.sort_by_key(|k| k.net_id());
    let mut result = None;
    let prioritized = state.prioritized.map_or(0, |k| k.net_id());
    for kind in ladder {
        let id = kind.net_id();
        if !state.can_spawn(kind) || !special(id) {
            continue;
        }
        if any_npcs(npcs, id) {
            // `:4901-4905`: a resident closes the entry.
            state.clear_can_spawn(kind);
            continue;
        }
        if rooms.has_room_quick(id) {
            return Some(id);
        }
        if town::is_town_pet(id) {
            return Some(id);
        }
        if id == prioritized {
            result = Some(id);
        }
    }
    result
}

/// `WorldGen.TrySpawningTownNPC`'s gate (`WorldGen.cs:74869-74880`): the
/// invasion/eclipse guard, the `++npcSpawnDelay >= npcSpawnPeriod` counter
/// (reset on fire), and the blue-dungeon-wall `Next(4) == 0` roll — the ONE
/// draw, and only when `wall == 34`. Returns whether `SpawnTownNPC(x, y)`
/// should run.
///
/// W4-d119 escalation 1: WIRED through the tr-core walker seam
/// ([`tr_core::growth::try_spawning_town_npc`] + [`TownSpawnWalker`]) — the
/// growth walkers' inactive-tile arms (`:72497`/`:73807`) push `(x, y)`
/// spawn-requests onto the shared queue and `world_sim/growth.rs` drains it
/// at the `UpdateWorldGen` slot into [`spawn_town_npc`]. This app-side gate
/// stays as the statics-carrier form the drain's witness pins (the tr-core
/// gate is the same logic with the queue push instead of the bool return).
pub(crate) fn try_spawning_town_npc_gate(
    wall: u16,
    invasion_type: i32,
    eclipse: bool,
    statics: &mut TownSpawnStatics,
    rng: &mut UnifiedRandom,
) -> bool {
    // `:74871`: `Main.invasionType <= 0 && !Main.eclipse &&
    // ++WorldGen.npcSpawnDelay >= WorldGen.npcSpawnPeriod`.
    if invasion_type > 0 || eclipse {
        return false;
    }
    statics.npc_spawn_delay += 1;
    if statics.npc_spawn_delay < statics.npc_spawn_period {
        return false;
    }
    statics.npc_spawn_delay = 0;
    // `:74874`: `Main.tile[x, y].wall != 34 || Main.rand.Next(4) == 0` —
    // the short-circuit means a non-34 wall draws NOTHING.
    wall != 34 || rng.next_max(4) == 0
}

/// The app drain of the walkers' `TrySpawningTownNPC` spawn-request queue
/// (W4-d119 escalation 1): each queued `(x, y)` runs the FULL
/// [`spawn_town_npc`] (`WorldGen.cs:5409-5615`) — the C#'s inline call at
/// `:74878`, deferred to the `UpdateWorldGen` slot because the tr-core
/// walkers have no NPC store. Returns the NO_HOBO (8) count (one per
/// `Successful`/`RelocatedHomeless`); the caller fires the progression
/// events after the borrow releases (the `notify_progression_net` seam).
/// The `tileSolid[379]` window (`:74876-74879`) needs no toggle — the
/// generated `TILE_SOLID` base table already carries 379.
pub(crate) fn drain_town_spawn_requests(
    requests: &[(i32, i32)],
    world: &mut World,
    npcs: &mut Vec<Npc>,
    rooms: &mut TownRooms,
    state: &mut ArrivalState,
    statics: &mut TownSpawnStatics,
    player: Option<(f32, f32, f32, f32)>,
    unlocked_truffle_spawn: bool,
    chat: &mut crate::chat::ChatMonitor,
) -> u32 {
    let mut nohobo = 0u32;
    for &(x, y) in requests {
        // `:74878`: `WorldGen.SpawnTownNPC(x, y)` — the default
        // `canSpawnNewTownNPC: true`.
        let out = spawn_town_npc(
            world, npcs, rooms, state, statics, x, y, true, player,
            unlocked_truffle_spawn, chat,
        );
        if matches!(out, TownNpcSpawnResult::Successful | TownNpcSpawnResult::RelocatedHomeless) {
            nohobo += 1;
        }
    }
    nohobo
}

/// The roofline span around a home tile (`WorldGen.cs:5097-5107`, the
/// `SpawnHomelessNPC` walks; `SpawnTravelNPC:5310-5318` is the same shape):
/// walk left while the floor cell is `SolidTile || tileSolidTop` and the
/// three cells above are each `!active || !tileSolid || tileSolidTop`; the
/// right walk is the `for` twin. Returns `(minValue, num4)` — the inclusive
/// pick band. ZERO draws.
fn roofline_span(world: &World, best_x: i32, best_y: i32) -> (i32, i32) {
    let standable = |x: i32| -> bool {
        let floor_ok = worldgen_solid_tile(world, x, best_y)
            || world
                .tile(x, best_y)
                .is_some_and(|t| TILE_SOLID_TOP.has(t.kind));
        if !floor_ok {
            return false;
        }
        (1..=3).all(|k| {
            world
                .tile(x, best_y - k)
                .is_none_or(|t| !t.active() || !TILE_SOLID.has(t.kind) || TILE_SOLID_TOP.has(t.kind))
        })
    };
    let mut min_x = best_x;
    let mut x = best_x;
    while x > best_x - 10 && standable(x) {
        min_x = x;
        x -= 1;
    }
    let mut max_x = best_x;
    let mut l = best_x;
    while l < best_x + 10 && standable(l) {
        max_x = l;
        l += 1;
    }
    (min_x, max_x)
}

/// The 30-attempt roofline pick (`WorldGen.cs:5108-5124`): the first 20
/// attempts reject the ±1 band around the home tile, the last 10 accept
/// anything but the exact home tile. Each attempt draws
/// `Next(minValue, num4 + 1)`; the first accepted pick breaks. Returns the
/// new `bestX`.
fn roofline_pick(
    best_x: i32,
    min_x: i32,
    max_x: i32,
    rng: &mut UnifiedRandom,
) -> i32 {
    let mut best_x = best_x;
    for m in 0..30 {
        let num7 = rng.next_range(min_x, max_x + 1);
        if m < 20 {
            if num7 < best_x - 1 || num7 > best_x + 1 {
                best_x = num7;
                break;
            }
        } else if num7 != best_x {
            best_x = num7;
            break;
        }
    }
    best_x
}

/// The outward surface spiral (`WorldGen.cs:5131-5183`, the
/// `SpawnHomelessNPC` scan): `n = 20..500`, both sides at `bestX ± n*2`,
/// the column band `[max(bestY-n, 10), min(bestY+n, worldSurface))`; the
/// first active+solid cell whose three overhead cells are dry and whose
/// 3×3 overhead box is clear wins, unless a player stands in the safe-range
/// box. Returns `(num8, num9)` — the C# keeps the LAST scanned cell even
/// when nothing was found (the `NewNPC` quirk, `:5185`). ZERO draws.
fn homeless_surface_scan(
    world: &World,
    best_x: i32,
    best_y: i32,
    player: Option<(f32, f32, f32, f32)>,
) -> (i32, i32) {
    let mut num8 = best_x;
    let mut num9 = best_y;
    // `:5132`: `!((double)num9 > Main.worldSurface)` — the whole scan skips
    // for an underground home.
    if (num9 as f64) > world.world_surface {
        return (num8, num9);
    }
    let mut found = false;
    for n in 20..500 {
        for side in 0..2 {
            num8 = if side != 0 { best_x - n * 2 } else { best_x + n * 2 };
            if num8 <= 10 || num8 >= world.width - 10 {
                continue;
            }
            let top = (best_y - n).max(10);
            // `:5145-5150`: the double bound is NOT truncated before the
            // `(double)num13 < num12` compare.
            let bottom = ((best_y + n) as f64).min(world.world_surface);
            let mut num13 = top;
            while (num13 as f64) < bottom {
                num9 = num13;
                let floor_solid = world
                    .tile(num8, num9)
                    .is_some_and(|t| t.active() && !t.inactive() && TILE_SOLID.has(t.kind));
                if !floor_solid {
                    num13 += 1;
                    continue;
                }
                // `:5156`: the three overhead liquid cells + the solid box.
                let wet = (1..=3).any(|k| {
                    world
                        .tile(num8, num9 - k)
                        .is_some_and(|t| t.liquid != 0)
                });
                if wet || collision_solid_tiles(world, num8 - 1, num8 + 1, num9 - 3, num9 - 1) {
                    break;
                }
                found = true;
                // `:5160-5170`: the player safe-range veto.
                if player_in_rect(player, safe_range_rect(num8, num9)) {
                    found = false;
                }
                break;
            }
            if found {
                break;
            }
        }
        if found {
            break;
        }
    }
    (num8, num9)
}

/// `NPC.NewNPC(source, X, Y, type, 1)`'s town placement
/// (`NPC.cs:80749-80771`): `Bottom = (X, Y)` — the port's top-left is
/// `(X - width/2, Y - height)`. The naming seam
/// (`GiveTownUniqueDataToNPCsThatNeedIt`) is the W4-f38 lane's; the port
/// keeps `given_name` empty like the landed arrival paths. The
/// `BlockedTooManyNPCs` slot cap has no port analogue (the live list is a
/// Vec, not the fixed 200-slot array) — the push always succeeds.
fn new_town_npc(kind: NpcKind, bottom_x: i32, bottom_y: i32) -> Npc {
    let (w, h) = kind.size();
    let mut n = Npc::new(
        kind,
        bottom_x as f32 - w * 0.5,
        bottom_y as f32 - h,
        1,
        0.0,
    );
    n.hp = n.max_hp;
    n
}

/// The full `WorldGen.SpawnTownNPC(x, y, canSpawnNewTownNPC = true)`
/// (`WorldGen.cs:5409-5615`). `player` is the one active player's hitbox
/// `(x, y, w, h)`; the NO_HOBO progression events (`:5460`, `:5576`) are the
/// CALLER's seam (`notify_progression_net` needs `&mut App`) — fire one per
/// `Successful`/`RelocatedHomeless` result. The unlock switch
/// (`:5577-5608`) writes the two modelled flags (`unlockedPartyGirlSpawn`,
/// `unlockedPrincessSpawn`); the other seven setters are the W4-f45 Ph4
/// lane's claim (pushed, `8c28a7ae`). `CheckAchievement_RealEstateAndTownSlimes`
/// (`:5609`) waits on the full town-slime roster (d119 note).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_town_npc(
    world: &mut World,
    npcs: &mut Vec<Npc>,
    rooms: &mut TownRooms,
    state: &mut ArrivalState,
    statics: &mut TownSpawnStatics,
    x: i32,
    y: i32,
    mut can_spawn_new: bool,
    player: Option<(f32, f32, f32, f32)>,
    unlocked_truffle_spawn: bool,
    chat: &mut crate::chat::ChatMonitor,
) -> TownNpcSpawnResult {
    // `:5411-5418`: the house-wall pick arms the sticky flag; a clear flag
    // blocks. The C# reads `Main.wallHouse[wall]` DIRECTLY here — since the
    // g44e dedupe `housing::is_house_wall` delegates to the ONE full-table
    // `edit::wall_house` copy (research/87 RETIRE §5, EXECUTED dd8a2f764),
    // so this call site is C#-correct with no local edit (the free :5411
    // gain the retire receipt names).
    let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
    if tr_core::housing::is_house_wall(wall) {
        statics.wg_can_spawn = true;
    }
    if !statics.wg_can_spawn {
        return TownNpcSpawnResult::Blocked;
    }
    // `:5419-5426`: StartRoomCheck + RoomNeeds (the check writes the flag).
    let scan = match check_room(world, x, y) {
        Ok(s) => {
            statics.wg_can_spawn = true;
            s
        }
        Err(_) => {
            statics.wg_can_spawn = false;
            return TownNpcSpawnResult::Blocked;
        }
    };
    if !room_needs(&scan.house_tiles) {
        // `RoomNeeds` writes the flag too (`WorldGen.cs:5719-5727`).
        statics.wg_can_spawn = false;
        return TownNpcSpawnResult::Blocked;
    }
    statics.wg_can_spawn = true;
    // `:5427-5428`: the prioritized type + the homeless mover.
    let mut num = state.prioritized.map_or(0, |k| k.net_id());
    let mut num2 = town::find_npc_looking_for_home_that_can_move_in(
        world,
        &scan,
        npcs,
        num,
        unlocked_truffle_spawn,
    );
    // `:5429-5434`: a town pet with a stored room is stinkbug-exempt.
    let pet_with_room = town::is_town_pet(num) && rooms.has_room(num).is_some();
    if (scan.has_stinkbug || scan.has_echo_stinkbug) && !pet_with_room {
        return TownNpcSpawnResult::Blocked;
    }
    // `:5435-5439`: ScoreRoom(-1, type).
    let (occupied, shared_room_x) = room_occupancy(&scan, npcs, None, num);
    let report = score_room_report(world, &scan, occupied, shared_room_x);
    if report.hi_score <= 0 {
        return TownNpcSpawnResult::Blocked;
    }
    let Some((mut best_x, mut best_y)) = report.best else {
        return TownNpcSpawnResult::Blocked;
    };
    // `:5440-5452`: a resident of the prioritized type blocks the new spawn;
    // otherwise the mover slot clears and the spawnable-type query runs.
    if tr_core::housing::check_special_town_npc_spawning_conditions(
        world,
        &scan,
        num,
        unlocked_truffle_spawn,
    ) && any_npcs(npcs, num)
    {
        can_spawn_new = false;
    } else {
        num2 = None;
        if can_spawn_new {
            let picked = is_there_a_spawnable_prioritized_town_npc(
                world,
                &scan,
                state,
                rooms,
                npcs,
                (best_x, best_y),
                unlocked_truffle_spawn,
            );
            num = picked.unwrap_or(0);
            can_spawn_new = num > 0;
        }
    }
    // `:5453-5463`: the relocation arm.
    if let Some(mover) = num2 {
        let prioritized = state.prioritized.map_or(0, |k| k.net_id());
        if let Some((k, _)) = crate::npc::kind_from_net_id(prioritized) {
            state.clear_can_spawn(k); // `:5454`
        }
        npcs[mover].home = Some((best_x, best_y));
        npcs[mover].homeless_despawn = false;
        // NO_HOBO (8) — the caller fires it (`:5460`).
        state.prioritized = None; // `:5461`
        return TownNpcSpawnResult::RelocatedHomeless;
    }
    // `:5464-5467`: a resident of the (possibly re-picked) type blocks.
    if any_npcs(npcs, state.prioritized.map_or(0, |k| k.net_id())) {
        can_spawn_new = false;
    }
    if can_spawn_new {
        let Some((kind, _)) = crate::npc::kind_from_net_id(num) else {
            return TownNpcSpawnResult::Blocked;
        };
        // `:5470-5482`: the stored-room alternate spot — one recursion,
        // bestX/bestY saved and restored around it.
        if let Some((rx, ry)) = rooms.has_room(num) {
            if !statics.alternate_spot {
                let (save_x, save_y) = (best_x, best_y);
                statics.alternate_spot = true;
                let inner = spawn_town_npc(
                    world, npcs, rooms, state, statics, rx, ry - 2, true, player,
                    unlocked_truffle_spawn, chat,
                );
                statics.alternate_spot = false;
                best_x = save_x;
                best_y = save_y;
                if inner == TownNpcSpawnResult::Successful {
                    return inner;
                }
            }
        }
        // `:5485-5488`: the infi-housing gate — reads
        // `WorldGen.prioritizedTownNPCType`, NOT the re-picked num.
        let prioritized_id = state.prioritized.map_or(0, |k| k.net_id());
        if town::is_room_considered_already_occupied(npcs, best_x, best_y, prioritized_id) {
            return TownNpcSpawnResult::BlockedInfiHousing;
        }
        // `:5489-5501`: the player safe-range scan at the scored spot.
        let mut flag2 = !player_in_rect(player, safe_range_rect(best_x, best_y));
        // `:5483-5484` + `:5502-5553`: the spiral writes the LOCAL num5/num6
        // (seeded from bestX/bestY); bestX/bestY stay the scored spot — the
        // home tiles (`:5563-5564`) and the facing compare (`:5565-5572`)
        // read the ORIGINAL best, NewNPC reads the spiral result.
        let (mut num5, mut num6) = (best_x, best_y);
        if !flag2 && !((num6 as f64) > world.world_surface) {
            'spiral: for j in 1..500 {
                for k in 0..2 {
                    num5 = if k != 0 { best_x - j } else { best_x + j };
                    if num5 <= 10 || num5 >= world.width - 10 {
                        continue;
                    }
                    let top = (best_y - j).max(10);
                    let bottom = ((best_y + j) as f64).min(world.world_surface);
                    let mut l = top;
                    while (l as f64) < bottom {
                        num6 = l;
                        let solid = world
                            .tile(num5, num6)
                            .is_some_and(|t| t.active() && !t.inactive() && TILE_SOLID.has(t.kind));
                        if !solid {
                            l += 1;
                            continue;
                        }
                        if collision_solid_tiles(world, num5 - 1, num5 + 1, num6 - 3, num6 - 1) {
                            break;
                        }
                        flag2 = true;
                        if player_in_rect(player, safe_range_rect(num5, num6)) {
                            flag2 = false;
                        }
                        break;
                    }
                    if flag2 {
                        break 'spiral;
                    }
                }
            }
        }
        // `:5554-5576`: NewNPC at (num5*16, num6*16), the home tiles = the
        // scored bestX/bestY, the facing, the arrival chat and NO_HOBO (the
        // caller's seam). The C# keeps the LAST scanned num5/num6 even when
        // nothing was found — the quirk is kept.
        let mut n = new_town_npc(kind, num5 * 16, num6 * 16);
        n.home = Some((best_x, best_y));
        if num5 < best_x {
            n.facing = 1;
        } else if num5 > best_x {
            n.facing = -1;
        }
        npcs.push(n);
        state.clear_can_spawn(kind); // `:5562`
        crate::chat::broadcast_death_text(
            chat,
            &crate::lang::text_args("Announcement.HasArrived", &[kind.name().to_string()]),
            crate::chat::COLOR_NPC_TRAVEL,
        );
        // `:5577-5608`: the unlock switch — the two modelled flags; the
        // other seven setters are W4-f45 Ph4's pushed claim.
        match num {
            208 => world.unlocked_party_girl = true,
            663 => world.unlocked_princess = true,
            _ => {}
        }
        state.prioritized = None; // `:5610`
        return TownNpcSpawnResult::Successful;
    }
    // `:5613-5614`: a house was found but nobody spawned.
    statics.last_found_house = (x, y);
    TownNpcSpawnResult::FoundHouseNoSpawn
}

/// `WorldGen.UnspawnHomelessNPC` (`WorldGen.cs:5206-5249`): the FIRST live
/// `homelessDespawn` NPC; a housed / non-town one only clears the flag.
/// With no player in the safe-range box around its centre tile it departs:
/// the `Lang.misc[35]` broadcast, `active = false`, `life = 0`,
/// `SendData(23)` (the port's removal rides the cadence despawn path, the
/// `unspawn_travel_npc` precedent). ZERO draws.
pub(crate) fn unspawn_homeless_npc(
    npcs: &mut Vec<Npc>,
    player: Option<&crate::player::Player>,
    chat: &mut crate::chat::ChatMonitor,
) {
    // `:5208-5216`: the first homelessDespawn NPC.
    let Some(mut num) = npcs.iter().position(|n| n.homeless_despawn) else {
        return;
    };
    // `:5221-5225`: `!homeless || !active || !townNPC` clears the flag.
    // The port's `home.is_none()` is `homeless`; the live list is active.
    if npcs[num].home.is_some() || !npcs[num].kind.is_town_npc_like() {
        npcs[num].homeless_despawn = false;
        num = usize::MAX; // the C#'s `num = -1`
    }
    if num == usize::MAX {
        return;
    }
    // `:5230-5231`: `(int)(Center.X / 16f)` — truncation, not the shift.
    let (cx, cy) = npcs[num].center();
    let tx = (cx / 16.0) as i32;
    let ty = (cy / 16.0) as i32;
    let hitbox = player.map(|p| {
        (p.x, p.y, crate::player::WIDTH, crate::player::HEIGHT)
    });
    // `:5233-5241`: the safe-range veto.
    if player_in_rect(hitbox, safe_range_rect(tx, ty)) {
        return;
    }
    // `:5244-5247`: the departure broadcast + deactivation.
    let name = npcs[num].kind.name().to_string();
    crate::chat::broadcast_death_text(
        chat,
        &crate::lang::text_args("LegacyMisc.35", &[name]),
        crate::chat::COLOR_NPC_TRAVEL,
    );
    npcs.remove(num);
}

/// `WorldGen.SpawnHomelessNPC` (`WorldGen.cs:5036-5204`) — the field-level
/// driver (the `UpdateTime` cadence roll is the CALLER's, `Main.cs:65057`;
/// `rng` is the `UpdateTime` stream). Gates: eclipse / night / a started
/// invasion / no prioritized type / a live `homelessSpawnTimeout`; at least
/// one housed resident (37/453/368 excluded); no homeless town NPC anywhere
/// (`FindAnyHomelessTownNPC`, Guide included); no live NPC of the
/// prioritized type. Then the `LastFoundHouse` retry (a `Successful` result
/// arms the 54000-tick timeout), else the edge arrival: a random resident's
/// home, the roofline pick, the surface spiral, `NewNPC` homeless +
/// `homelessDespawn`, one `QuickFindHome`, and the `Next(6)` chat variant.
/// Returns the NO_HOBO (8) count — the caller fires the progression events
/// (the `notify_progression_net` &mut App seam).
///
/// Port model note: `Npc.home` conflates homeless + homeTile — the arrival's
/// anchor tiles are consumed by the immediate `QuickFindHome` and not
/// stored on a homeless NPC (the later re-scans ride the 2 s look pass at
/// the NPC's position, the landed convention).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_homeless_npc(
    world: &mut World,
    npcs: &mut Vec<Npc>,
    rooms: &mut TownRooms,
    arrival: &mut ArrivalState,
    statics: &mut TownSpawnStatics,
    player: Option<&crate::player::Player>,
    rng: &mut UnifiedRandom,
    chat: &mut crate::chat::ChatMonitor,
) -> u32 {
    let mut nohobo = 0u32;
    // `:5038-5041`: the gate chain.
    if world.eclipse
        || !world.day_time
        || (world.invasion_type > 0 && world.invasion_delay == 0 && world.invasion_size > 0)
        || arrival.prioritized.is_none()
        || statics.homeless_spawn_timeout > 0
    {
        return nohobo;
    }
    // `:5042-5050`: the housed-resident count (37/453/368 excluded; the
    // port's `is_town_resident` carries 368, so it is filtered here).
    let housed = npcs
        .iter()
        .filter(|n| {
            n.hp > 0
                && n.kind.is_town_resident()
                && n.home.is_some()
                && !matches!(n.kind.net_id(), 37 | 453 | 368)
        })
        .count();
    // `:5051-5054`: no residents, or a homeless town NPC already exists.
    if housed == 0 || town::find_any_homeless_town_npc(npcs, true).is_some() {
        return nohobo;
    }
    let prioritized0 = arrival.prioritized.unwrap();
    // `:5055-5061`: a live NPC of the prioritized type blocks.
    if any_npcs(npcs, prioritized0.net_id()) {
        return nohobo;
    }
    let hitbox = player.map(|p| (p.x, p.y, crate::player::WIDTH, crate::player::HEIGHT));
    // `:5062-5075`: the LastFoundHouse retry (the tileSolid[379] window is
    // the base table state — no toggle). A Successful result arms the
    // timeout and returns; every other result falls through.
    let (lx, ly) = statics.last_found_house;
    let retry = spawn_town_npc(
        world, npcs, rooms, arrival, statics, lx, ly, true, hitbox, false, chat,
    );
    if retry == TownNpcSpawnResult::Successful {
        statics.homeless_spawn_timeout = 54000;
        return nohobo + 1;
    }
    if retry == TownNpcSpawnResult::RelocatedHomeless {
        nohobo += 1;
    }
    // The C# reads `WorldGen.prioritizedTownNPCType` LIVE at the NewNPC
    // (`:5185`) — a relocation inside the retry zeroed it (`:5461`); the
    // type-0 spawn that would follow is an unmodelled kind, so the faithful
    // reading is "no kind to arrive".
    let Some(prioritized) = arrival.prioritized else {
        return nohobo;
    };
    // `:5077-5091`: the housed residents' home tiles (37 excluded, the
    // C#'s array walk; the port keeps the live-list order).
    let homes: Vec<(i32, i32)> = npcs
        .iter()
        .filter(|n| {
            n.hp > 0 && n.kind.is_town_npc_like() && n.kind.net_id() != 37 && n.home.is_some()
        })
        .map(|n| n.home.unwrap())
        .collect();
    if homes.is_empty() {
        return nohobo;
    }
    // `:5093`: `Main.rand.Next(num2)` — the house pick.
    let pick = rng.next_max(homes.len() as i32) as usize;
    let (mut best_x, best_y) = homes[pick];
    // `:5097-5124`: the roofline span + the 30-attempt pick.
    let (min_x, max_x) = roofline_span(world, best_x, best_y);
    best_x = roofline_pick(best_x, min_x, max_x, rng);
    // `:5131-5183`: the surface spiral.
    let (num8, num9) = homeless_surface_scan(world, best_x, best_y, hitbox);
    // `:5185-5199`: NewNPC, the home tiles, homeless + homelessDespawn, the
    // facing. The port's home Option: the anchor feeds QuickFindHome below.
    let mut n = new_town_npc(prioritized, num8 * 16, num9 * 16);
    n.homeless_despawn = true;
    if num8 < best_x {
        n.facing = 1;
    } else if num8 > best_x {
        n.facing = -1;
    }
    npcs.push(n);
    let idx = npcs.len() - 1;
    // `:5199`: QuickFindHome(num15) — the three-pass scan around the anchor.
    if let Some((hx, hy)) = town::quick_find_home(
        world,
        npcs,
        best_x,
        best_y,
        Some(idx),
        prioritized.net_id(),
        false,
    ) {
        npcs[idx].home = Some((hx, hy));
        npcs[idx].homeless_despawn = false;
        nohobo += 1;
    }
    // `:5200-5202`: the Next(6) chat variant — drawn even when the NPC just
    // housed (the key picks HasArrived instead).
    let num16 = rng.next_max(6);
    let key = if npcs[idx].home.is_none() {
        format!("Announcement.HomelessArrived_{num16}")
    } else {
        "Announcement.HasArrived".to_string()
    };
    let name = prioritized.name().to_string();
    crate::chat::broadcast_death_text(
        chat,
        &crate::lang::text_args(&key, &[name]),
        crate::chat::COLOR_NPC_TRAVEL,
    );
    nohobo
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_world_helpers::{build_furnished_room, ground};

    fn guide(home: Option<(i32, i32)>) -> Npc {
        let mut n = Npc::new(NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, 1, 0.0);
        n.home = home;
        n.hp = n.max_hp;
        n
    }

    /// The field-level driver's harness: the homeless gates + the edge
    /// arrival through one call shape.
    struct Harness {
        world: World,
        npcs: Vec<Npc>,
        rooms: TownRooms,
        arrival: ArrivalState,
        statics: TownSpawnStatics,
        rng: UnifiedRandom,
        chat: crate::chat::ChatMonitor,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                world: ground(),
                npcs: Vec::new(),
                rooms: TownRooms::new(),
                arrival: ArrivalState::new(),
                statics: TownSpawnStatics::default(),
                rng: UnifiedRandom::new(42),
                chat: crate::chat::ChatMonitor::default(),
            }
        }

        fn spawn(&mut self) -> u32 {
            spawn_homeless_npc(
                &mut self.world,
                &mut self.npcs,
                &mut self.rooms,
                &mut self.arrival,
                &mut self.statics,
                None,
                &mut self.rng,
                &mut self.chat,
            )
        }
    }

    /// `WorldGen.cs:5038-5041`: the gate chain — a live timeout, the night,
    /// an eclipse, a started invasion or no prioritized type each block the
    /// arrival before any draw.
    #[test]
    fn the_homeless_gates_block_before_any_draw() {
        let mut h = Harness::new();
        h.arrival.prioritized = Some(NpcKind::Merchant);
        h.npcs.push(guide(Some((65, 106))));
        let pristine = h.rng.clone();
        // The fresh-world timeout (54000, clearWorld) blocks.
        h.spawn();
        assert_eq!(h.rng, pristine);
        assert_eq!(h.npcs.len(), 1);
        // Timeout expired but night blocks.
        h.statics.homeless_spawn_timeout = 0;
        h.world.day_time = false;
        h.spawn();
        assert_eq!(h.rng, pristine);
        // Day but eclipse blocks.
        h.world.day_time = true;
        h.world.eclipse = true;
        h.spawn();
        assert_eq!(h.rng, pristine);
        // A started invasion blocks.
        h.world.eclipse = false;
        h.world.invasion_type = 1;
        h.world.invasion_delay = 0;
        h.world.invasion_size = 10;
        h.spawn();
        assert_eq!(h.rng, pristine);
        assert_eq!(h.npcs.len(), 1, "no arrival through any gate");
    }

    /// `WorldGen.cs:5051-5061`: an existing homeless town NPC (Guide
    /// included) or a live NPC of the prioritized type blocks the arrival.
    #[test]
    fn an_existing_homeless_or_resident_type_blocks() {
        let mut h = Harness::new();
        h.statics.homeless_spawn_timeout = 0;
        h.arrival.prioritized = Some(NpcKind::Merchant);
        h.npcs.push(guide(Some((65, 106))));
        // A homeless Guide blocks (includeGuide: true, `:5051`).
        h.npcs.push(guide(None));
        let pristine = h.rng.clone();
        h.spawn();
        assert_eq!(h.rng, pristine);
        assert_eq!(h.npcs.len(), 2);
        // Swap the homeless Guide for a resident Merchant: the type gate.
        h.npcs.pop();
        let mut merchant = Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0);
        merchant.home = Some((70, 106));
        merchant.hp = merchant.max_hp;
        h.npcs.push(merchant);
        h.spawn();
        assert_eq!(h.rng, pristine);
        assert_eq!(h.npcs.len(), 2);
    }

    /// `WorldGen.cs:5093-5202`'s RNG census: the full edge-arrival arm draws
    /// the house pick (1), k roofline picks (1 <= k <= 30) and the chat
    /// variant (1) — the twin replays the exact stream prefix.
    #[test]
    fn the_edge_arrival_draws_pick_plus_roofline_plus_chat() {
        let mut h = Harness::new();
        h.world.day_time = true;
        h.statics.homeless_spawn_timeout = 0;
        h.arrival.prioritized = Some(NpcKind::Merchant);
        h.npcs.push(guide(Some((65, 106))));
        h.spawn();
        assert_eq!(h.npcs.len(), 2, "the Merchant arrived");
        let arrival = &h.npcs[1];
        assert_eq!(arrival.kind, NpcKind::Merchant);
        assert!(arrival.homeless_despawn || arrival.home.is_some());
        // Twin: replay the census on a fresh stream — 1 house pick, the
        // roofline picks until the first accepted one, 1 chat roll.
        let mut twin = UnifiedRandom::new(42);
        let _ = twin.next_max(1); // Next(num2), one resident home
        let (min_x, max_x) = roofline_span(&h.world, 65, 106);
        let mut best_x = 65;
        for m in 0..30 {
            let roll = twin.next_range(min_x, max_x + 1);
            if m < 20 {
                if roll < best_x - 1 || roll > best_x + 1 {
                    best_x = roll;
                    break;
                }
            } else if roll != best_x {
                best_x = roll;
                break;
            }
        }
        let _ = twin.next_max(6);
        assert_eq!(
            h.rng, twin,
            "exactly 1 + k + 1 draws on the UpdateTime stream"
        );
    }

    /// `WorldGen.cs:5206-5249`: the first homelessDespawn NPC departs at
    /// night with no player near; a housed one only clears the flag; a
    /// player in the safe-range box vetoes the departure. ZERO draws.
    #[test]
    fn the_homeless_departure_is_the_safe_range_veto() {
        let mut npcs = vec![guide(None)];
        npcs[0].homeless_despawn = true;
        let mut chat = crate::chat::ChatMonitor::default();
        // No player: the departure runs.
        unspawn_homeless_npc(&mut npcs, None, &mut chat);
        assert!(npcs.is_empty(), "the homeless NPC departed");
        assert_eq!(chat.messages.len(), 1, "the Lang.misc[35] broadcast");
        // A housed homelessDespawn NPC only clears the flag (`:5221-5225`).
        let mut housed = guide(Some((65, 106)));
        housed.homeless_despawn = true;
        let mut npcs = vec![housed];
        unspawn_homeless_npc(&mut npcs, None, &mut chat);
        assert_eq!(npcs.len(), 1);
        assert!(!npcs[0].homeless_despawn, "the flag cleared, no departure");
        // A player inside the box vetoes (`:5233-5241`).
        let mut n = guide(None);
        n.homeless_despawn = true;
        let mut npcs = vec![n];
        let mut player = crate::player::Player::spawn(&ground());
        let (cx, cy) = npcs[0].center();
        player.x = cx;
        player.y = cy;
        unspawn_homeless_npc(&mut npcs, Some(&player), &mut chat);
        assert_eq!(npcs.len(), 1, "the player's presence keeps the NPC");
    }

    /// `WorldGen.cs:74869-74880`: the delay counter gates the spawn; the
    /// Next(4) draw fires ONLY on a wall-34 tile (the short-circuit).
    #[test]
    fn the_try_spawning_gate_counts_and_draws_only_on_wall_34() {
        let mut statics = TownSpawnStatics {
            npc_spawn_period: 20,
            ..Default::default()
        };
        let mut rng = UnifiedRandom::new(7);
        let pristine = rng.clone();
        // 19 sub-period calls: no fire, no draw (wall 0).
        for _ in 0..19 {
            assert!(!try_spawning_town_npc_gate(0, 0, false, &mut statics, &mut rng));
        }
        assert_eq!(rng, pristine, "a non-34 wall draws nothing");
        // The 20th fires with no draw.
        assert!(try_spawning_town_npc_gate(0, 0, false, &mut statics, &mut rng));
        assert_eq!(statics.npc_spawn_delay, 0, "the counter reset");
        assert_eq!(rng, pristine);
        // Wall 34 draws the Next(4) on the fire tick.
        for _ in 0..19 {
            try_spawning_town_npc_gate(34, 0, false, &mut statics, &mut rng);
        }
        let _ = try_spawning_town_npc_gate(34, 0, false, &mut statics, &mut rng);
        let mut twin = UnifiedRandom::new(7);
        let _ = twin.next_max(4);
        assert_eq!(rng, twin, "exactly one Next(4) on the wall-34 fire");
        // The invasion/eclipse guards skip before the counter.
        let before = statics.npc_spawn_delay;
        assert!(!try_spawning_town_npc_gate(0, 1, false, &mut statics, &mut rng));
        assert!(!try_spawning_town_npc_gate(0, 0, true, &mut statics, &mut rng));
        assert_eq!(statics.npc_spawn_delay, before);
    }

    /// `WorldGen.cs:5409-5615`: a house-wall pick with an open canSpawn
    /// entry and a furnished room spawns the prioritized type, writes the
    /// room home, clears the entry + prioritized and broadcasts HasArrived.
    #[test]
    fn the_full_spawn_places_the_prioritized_type_in_the_room() {
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs: Vec<Npc> = Vec::new();
        let mut rooms = TownRooms::new();
        let mut state = ArrivalState::new();
        state.prioritized = Some(NpcKind::Guide);
        state.set_can_spawn(NpcKind::Guide);
        let mut statics = TownSpawnStatics::default();
        let mut chat = crate::chat::ChatMonitor::default();
        let out = spawn_town_npc(
            &mut w, &mut npcs, &mut rooms, &mut state, &mut statics,
            65, 105, true, None, false, &mut chat,
        );
        assert_eq!(out, TownNpcSpawnResult::Successful);
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0].kind, NpcKind::Guide);
        let (hx, hy) = npcs[0].home.expect("the arrival writes its home");
        assert!((60..=71).contains(&hx) && (100..=108).contains(&hy));
        assert!(!state.can_spawn(NpcKind::Guide), "the entry closed");
        assert_eq!(state.prioritized, None, "the prioritized slot zeroed");
        assert_eq!(chat.messages.len(), 1, "the HasArrived broadcast");
    }

    /// `WorldGen.cs:5613-5614`: a house found with no spawnable type writes
    /// LastFoundHouse and returns FoundHouseNoSpawn — the SpawnHomelessNPC
    /// retry anchor.
    #[test]
    fn a_house_with_no_spawn_writes_last_found_house() {
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs: Vec<Npc> = Vec::new();
        let mut rooms = TownRooms::new();
        let mut state = ArrivalState::new(); // prioritized = None
        let mut statics = TownSpawnStatics::default();
        let mut chat = crate::chat::ChatMonitor::default();
        let out = spawn_town_npc(
            &mut w, &mut npcs, &mut rooms, &mut state, &mut statics,
            65, 105, true, None, false, &mut chat,
        );
        assert_eq!(out, TownNpcSpawnResult::FoundHouseNoSpawn);
        assert_eq!(statics.last_found_house, (65, 105));
        assert!(npcs.is_empty());
    }

    /// W4-d119 escalation 1 — the walker drain: each queued `(x, y)` runs the
    /// full `SpawnTownNPC` (`WorldGen.cs:74878`); a `Successful` arrival counts
    /// one NO_HOBO, a `FoundHouseNoSpawn` writes LastFoundHouse and counts none.
    #[test]
    fn the_walker_drain_runs_the_full_spawn_per_request() {
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs: Vec<Npc> = Vec::new();
        let mut rooms = TownRooms::new();
        let mut state = ArrivalState::new();
        state.prioritized = Some(NpcKind::Guide);
        state.set_can_spawn(NpcKind::Guide);
        let mut statics = TownSpawnStatics::default();
        let mut chat = crate::chat::ChatMonitor::default();
        // One request at the furnished room: the Guide arrives (NO_HOBO 1).
        let nohobo = drain_town_spawn_requests(
            &[(65, 105)],
            &mut w, &mut npcs, &mut rooms, &mut state, &mut statics,
            None, false, &mut chat,
        );
        assert_eq!(nohobo, 1);
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0].kind, NpcKind::Guide);
        // A second request with the prioritized slot now empty: the house is
        // found but nobody spawns — LastFoundHouse writes, NO_HOBO stays 0.
        let nohobo = drain_town_spawn_requests(
            &[(65, 105)],
            &mut w, &mut npcs, &mut rooms, &mut state, &mut statics,
            None, false, &mut chat,
        );
        assert_eq!(nohobo, 0);
        assert_eq!(statics.last_found_house, (65, 105));
        assert_eq!(npcs.len(), 1, "no second copy spawned");
    }

    /// `WorldGen.cs:4881-4917`: the type-order walk — a resident clears its
    /// own canSpawn entry; HasRoomQuick returns at once; the prioritized
    /// type is only the fallback.
    #[test]
    fn the_spawnable_query_walks_types_and_clears_residents() {
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let scan = check_room(&w, 65, 105).unwrap();
        let mut state = ArrivalState::new();
        state.set_can_spawn(NpcKind::Guide);
        state.set_can_spawn(NpcKind::Merchant);
        state.prioritized = Some(NpcKind::Merchant);
        let mut rooms = TownRooms::new();
        // A resident Guide: his entry clears, the Merchant (priorized, no
        // room, not a pet) is the fallback result.
        let npcs = vec![guide(Some((65, 106)))];
        let got = is_there_a_spawnable_prioritized_town_npc(
            &w, &scan, &mut state, &rooms, &npcs, (65, 106), false,
        );
        assert_eq!(got, Some(17), "the Merchant fallback");
        assert!(!state.can_spawn(NpcKind::Guide), "the resident closed his entry");
        // A stored Merchant room wins outright (`:4906-4909`).
        rooms.set_room(17, 65, 106);
        let got = is_there_a_spawnable_prioritized_town_npc(
            &w, &scan, &mut state, &rooms, &npcs, (65, 106), false,
        );
        assert_eq!(got, Some(17));
    }
}
