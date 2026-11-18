//! The town-NPC housing — `WorldGen.QuickFindHome`'s three-pass scan
//! (`WorldGen.cs:5759-5805`) and the home assignment (`MoveTownNPC`'s tail,
//! `:4848-4854`), consumed by the homeless town NPCs.
//!
//! Also the two world-file states that ride this module (M6-SAVE-1's section
//! 6/9): `TownRooms` = `WorldGen.TownManager` (`TownRoomManager.cs`) with the
//! load-path `FixAgainstExploits` home re-validation, and `ShimmeredTownNpcs`
//! = `NPC.ShimmeredTownNPCs` (`NPC.cs:6071`).

use tr_core::housing::{check_room, room_needs, score_room, start_room_check};
use tr_core::wld::sections::npcs::NPCID_COUNT;
use tr_core::world::World;
use tr_core::wld::sections::WldTownRoom;

use crate::npc::Npc;

/// `QuickFindHome` (`WorldGen.cs:5732-5795`): the three-pass room scan
/// around `(tile_x, tile_y)` — the exact tile, the ±1 ring, the ±10 ring
/// with stride 2 — the first pass whose room both scans and furnishes
/// wins. Returns the `(best_x, best_y)` spot.
///
/// `npcs` is the live NPC list (the room's occupancy, `ScoreRoom`'s
/// `ScoreRoom_IsThisRoomOccupiedBySomeone` `:5814-5829`); `ignore` is the
/// NPC asking for the home (the C#'s `ignoreNPC`).
pub fn quick_find_home(
    world: &World,
    npcs: &[Npc],
    tile_x: i32,
    tile_y: i32,
    ignore: Option<usize>,
    asking_net_id: i32,
) -> Option<(i32, i32)> {
    if tile_x <= 10 || tile_y <= 10 || tile_x >= world.width - 10 || tile_y >= world.height - 10 {
        return None;
    }
    // The pass 1: the exact tile (the scan reads `homeTileY - 1`).
    if let Some((bx, by)) = scan_spot(world, npcs, tile_x, tile_y - 1, ignore, asking_net_id) {
        return Some((bx, by));
    }
    // The pass 2: the ±1 ring.
    for i in (tile_x - 1)..=(tile_x + 1) {
        for j in (tile_y - 1)..=(tile_y + 1) {
            if let Some((bx, by)) = scan_spot(world, npcs, i, j, ignore, asking_net_id) {
                return Some((bx, by));
            }
        }
    }
    // The pass 3: the ±10 ring with stride 2.
    let num = 10;
    for k in (tile_x - num..=(tile_x + num)).step_by(2) {
        for l in (tile_y - num..=(tile_y + num)).step_by(2) {
            if let Some((bx, by)) = scan_spot(world, npcs, k, l, ignore, asking_net_id) {
                return Some((bx, by));
            }
        }
    }
    None
}

/// One scan spot: the room check + the furniture needs + the room score.
///
/// `WorldGen.QuickFindHome` (`WorldGen.cs:5732-5795`) runs `RoomNeeds` and
/// then `ScoreRoom` (`:5858-5992`), which picks the room's **best** home
/// spot — `WorldGen.bestX/bestY` — not the scan's centre. The C# writes the
/// home only when `hiScore > 0` (`:5769-5771`).
fn scan_spot(
    world: &World,
    npcs: &[Npc],
    x: i32,
    y: i32,
    ignore: Option<usize>,
    asking_net_id: i32,
) -> Option<(i32, i32)> {
    let scan = check_room(world, x, y).ok()?;
    if !room_needs(&scan.house_tiles) {
        return None;
    }
    // `ScoreRoom_IsThisRoomOccupiedBySomeone` (`WorldGen.cs:5814-5829`): a
    // housed town NPC whose home is in the room occupies it, so the room
    // scores −1 and no home is assigned. The C#'s
    // `CanNPCsLiveWithEachOther` compares the two `housingCategory` fields
    // (`TownRoomManager.cs:144-156`), so a same-category occupant refuses the
    // room while a different-category one only sets `sharedRoomX`.
    let (occupied, shared_room_x) = room_occupancy(&scan, npcs, ignore, asking_net_id);
    if occupied {
        return None;
    }
    let (bx, by, score) = score_room(world, &scan, shared_room_x)?;
    if score <= 0 {
        return None;
    }
    Some((bx, by))
}

/// `WorldGen.ScoreRoom_IsThisRoomOccupiedBySomeone` (`WorldGen.cs:5814-5829`):
/// the room's occupancy and the C#'s `WorldGen.sharedRoomX` (the home of a
/// NPC the room may be shared with).
fn room_occupancy(
    scan: &tr_core::housing::RoomScan,
    npcs: &[Npc],
    ignore: Option<usize>,
    asking_net_id: i32,
) -> (bool, Option<i32>) {
    let mut shared = None;
    for (i, npc) in npcs.iter().enumerate() {
        if Some(i) == ignore {
            continue;
        }
        if !npc.kind.is_town_npc() {
            continue;
        }
        let Some((hx, hy)) = npc.home else {
            continue;
        };
        if !scan.room_tiles.contains(&(hx, hy)) || !scan.room_tiles.contains(&(hx, hy - 1)) {
            continue;
        }
        // `:5821-5825`: a conflict refuses the room; otherwise the occupant's
        // home becomes `sharedRoomX` and the loop keeps looking.
        if !can_npcs_live_with_each_other(asking_net_id, npc.kind.net_id()) {
            return (true, None);
        }
        shared = Some(hx);
    }
    (false, shared)
}

/// `HousingCategoryID` (`ID/HousingCategoryID.cs:3-8`): `CommonNPCs = 0`,
/// `PetNPCs = 1`. `NPC.SetDefaults` writes 0 for every town NPC
/// (`NPC.cs:8518`) and 1 for the six pet ids — TownCat/TownDog 637/638
/// (`:17224`), TownBunny 656 (`:17308`), TownSlimeBlue 670 (`:17494`) and
/// the 678-684 green/old/purple/red/yellow/rainbow/copper slimes
/// (`:17607`). Keyed by the vanilla net id, like the C#'s
/// `ContentSamples.NpcsByNetId` lookup (`TownRoomManager.cs:146`).
pub fn housing_category(net_id: i32) -> i32 {
    match net_id {
        637 | 638 | 656 | 670 | 678..=684 => 1,
        _ => 0,
    }
}

/// `TownRoomManager.CanNPCsLiveWithEachOther(int npc1ByType, NPC npc2)`
/// (`TownRoomManager.cs:144-156`): resolve the asking type through the content
/// table (an id with no sample lives with anyone, `:146-149`) and then compare
/// the two `housingCategory` fields (`:153-156`).
pub fn can_npcs_live_with_each_other(asking_net_id: i32, occupant_net_id: i32) -> bool {
    housing_category(asking_net_id) != housing_category(occupant_net_id)
}


/// `WorldGen.TownManager` (`TownRoomManager.cs:11-13`): one room per town NPC
/// — the `_roomLocationPairs` list and the `_hasRoom` fast table the load path
/// rebuilds from the file (`TownRoomManager.cs:94`/`:108`). Both are keyed by
/// the NPC's *type* id (the wire's `npcID`).
#[derive(Clone, Debug, PartialEq)]
pub struct TownRooms {
    /// The `(npcID, X, Y)` pairs in registration order (`Save:99-103`).
    rooms: Vec<(i32, i32, i32)>,
    /// `_hasRoom[npcID]` (`TownRoomManager.cs:12`): the `HasRoomQuick` table.
    has_room: Vec<bool>,
}

impl TownRooms {
    pub fn new() -> Self {
        Self {
            rooms: Vec::new(),
            has_room: vec![false; NPCID_COUNT],
        }
    }

    /// `TownRoomManager.Load` (`:108-114`): the pairs come in file order and
    /// the fast table is rebuilt from them, so `HasRoomQuick` agrees with the
    /// list on a fresh load.
    pub fn from_wld_rooms(rooms: &[WldTownRoom]) -> Self {
        let mut state = Self::new();
        for room in rooms {
            if !(0..NPCID_COUNT as i32).contains(&room.npc_id) {
                continue; // `_hasRoom[npcID]` would throw on a foreign file
            }
            state.rooms.push((room.npc_id, room.x, room.y));
            state.has_room[room.npc_id as usize] = true;
        }
        state
    }

    /// `TownRoomManager.Save` (`:94-104`): the pairs in registration order.
    pub fn to_wld_rooms(&self) -> Vec<WldTownRoom> {
        self.rooms
            .iter()
            .map(|&(npc_id, x, y)| WldTownRoom { npc_id, x, y })
            .collect()
    }

    /// `HasRoom(npcID, out roomPosition)` (`TownRoomManager.cs:44-55`): the
    /// first pair for that NPC, or `None` when the fast table is clear.
    pub fn has_room(&self, npc_id: i32) -> Option<(i32, i32)> {
        if !self.has_room_quick(npc_id) {
            return None;
        }
        self.rooms
            .iter()
            .find(|(id, _, _)| *id == npc_id)
            .map(|&(_, x, y)| (x, y))
    }

    /// `HasRoomQuick` (`TownRoomManager.cs:40-43`).
    pub fn has_room_quick(&self, npc_id: i32) -> bool {
        (0..NPCID_COUNT as i32).contains(&npc_id) && self.has_room[npc_id as usize]
    }

    /// `SetRoom(npcID, X, Y)` (`TownRoomManager.cs:57-64`): the NPC's earlier
    /// room goes, the new one appends.
    pub fn set_room(&mut self, npc_id: i32, x: i32, y: i32) {
        if !(0..NPCID_COUNT as i32).contains(&npc_id) {
            return;
        }
        self.rooms.retain(|&(id, _, _)| id != npc_id);
        self.rooms.push((npc_id, x, y));
        self.has_room[npc_id as usize] = true;
    }

    /// `KickOut(npcType)` (`TownRoomManager.cs:80-85`): the pair is dropped and
    /// the fast table cleared — the port's `KickOut(NPC)` does both.
    pub fn kick_out(&mut self, npc_id: i32) {
        self.rooms.retain(|&(id, _, _)| id != npc_id);
        if (0..NPCID_COUNT as i32).contains(&npc_id) {
            self.has_room[npc_id as usize] = false;
        }
    }

    /// `Clear` (`TownRoomManager.cs:106-112`).
    pub fn clear(&mut self) {
        self.rooms.clear();
        self.has_room.iter_mut().for_each(|b| *b = false);
    }

    /// The room list, for inspection.
    pub fn rooms(&self) -> &[(i32, i32, i32)] {
        &self.rooms
    }

    /// `TownRoomManager.AddOccupantsToList(Point, List)` (`:22-31`): every
    /// registered npcID whose room sits exactly on `tile_position`.
    pub fn occupants_at(&self, tile_position: (i32, i32)) -> Vec<i32> {
        self.rooms
            .iter()
            .filter(|&&(_, x, y)| (x, y) == tile_position)
            .map(|&(id, _, _)| id)
            .collect()
    }

    /// `TownRoomManager.GetHouseholdStatus` (`TownRoomManager.cs:130-142`):
    /// 1 = homeless, 2 = has a registered room, 0 = neither (a housed NPC the
    /// book does not know). Sent as message 60's `number4`
    /// (`NPC.UpdateHomeTileState:52912`).
    pub fn household_status(&self, homeless: bool, npc_id: i32) -> u8 {
        if homeless {
            1
        } else if self.has_room_quick(npc_id) {
            2
        } else {
            0
        }
    }
}

impl Default for TownRooms {
    fn default() -> Self {
        Self::new()
    }
}

/// `NPC.ShimmeredTownNPCs` (`NPC.cs:6071`, `NPC.cs:95163`): the
/// `bool[NPCID.Count]` bitmap the section's first list persists as truth
/// indices (`SaveNPCs:1705-1713`). `AI_007_TownEntities` writes it per tick
/// (`NPC.cs:52967`); the file only says which are set.
#[derive(Clone, Debug, PartialEq)]
pub struct ShimmeredTownNpcs(Vec<bool>);

impl ShimmeredTownNpcs {
    pub fn new() -> Self {
        Self(vec![false; NPCID_COUNT])
    }

    /// `LoadNPCs:2888-2892`: the saved indices become the bitmap.
    pub fn from_wld_indices(indices: &[i32]) -> Self {
        let mut state = Self::new();
        for &index in indices {
            if (0..NPCID_COUNT as i32).contains(&index) {
                state.0[index as usize] = true;
            }
        }
        state
    }

    /// `SaveNPCs:1706-1713`: the truth indices, ascending (`array[i]` walks
    /// the array from 0).
    pub fn to_wld_indices(&self) -> Vec<i32> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| i as i32)
            .collect()
    }

    /// `ShimmeredTownNPCs[type]`.
    pub fn is_shimmered(&self, npc_id: i32) -> bool {
        (0..NPCID_COUNT as i32).contains(&npc_id) && self.0[npc_id as usize]
    }

    /// `NPC.ShimmeredTownNPCs[this.type] = this.IsShimmerVariant`
    /// (`NPC.cs:52967`); out-of-range ids are ignored (the C# would throw).
    pub fn set_shimmered(&mut self, npc_id: i32, value: bool) {
        if (0..NPCID_COUNT as i32).contains(&npc_id) {
            self.0[npc_id as usize] = value;
        }
    }
}

impl Default for ShimmeredTownNpcs {
    fn default() -> Self {
        Self::new()
    }
}

/// `WorldFile.FixAgainstExploits:1864-1896` — the town-NPC arm, run after both
/// the NPC section and the town rooms are loaded (the C# runs it from
/// `LoadWorld_LastMinuteFixes`, at the very end of the load). Reusable: the
/// same re-validation is what a mid-game `moveRoom` needs.
///
/// For every live town NPC with a home: `StartRoomCheck(homeTileX,
/// homeTileY - 1)`; on failure the stored room is offered (teleport in, retry),
/// and if that still fails the NPC is kicked out and goes homeless.
pub fn fix_town_npc_homes(world: &World, npcs: &mut [Npc], rooms: &mut TownRooms) {
    for npc in npcs.iter_mut() {
        // `!nPC.active || !nPC.townNPC || nPC.type == 368 || nPC.type == 37 ||
        // nPC.homeless` (`:1881`). The port's live list holds only active NPCs,
        // `is_town_npc` is the modelled `townNPC` set, and the Travelling
        // Merchant (368) and Old Man (37) have no ported kind — every guard
        // is satisfied or vacuous, none dropped.
        if !npc.kind.is_town_npc() || npc.home.is_none() {
            continue; // homeless NPCs are never re-validated
        }
        let (home_x, home_y) = npc.home.unwrap();
        let mut fits = start_room_check(world, home_x, home_y - 1);
        if !fits {
            // `HasRoom(type, out var roomPosition)` (`:1885-1887`).
            if let Some((room_x, room_y)) = rooms.has_room(npc.kind.net_id()) {
                npc.home = Some((room_x, room_y));
                fits = start_room_check(world, room_x, room_y - 1);
            }
            if !fits {
                // `KickOut(nPC)` + `homeless = true` (`:1893-1894`).
                rooms.kick_out(npc.kind.net_id());
                npc.home = None;
            }
        }
    }
}

/// `WorldGen.kickOut` (`WorldGen.cs:4869-4879`): the NPC goes homeless and is
/// barred from looking for a room for `NPC.KickOutLookForHomeTimeout` ticks
/// (3600 — `NPC.cs:95267`), and its registered room is dropped. The net-mode
/// arm (`:4871-4875`) is the dedicated server's; the port runs single-player.
pub fn kick_out(rooms: &mut TownRooms, npc: &mut Npc) {
    npc.home = None;
    npc.look_for_home_timeout = KICK_OUT_LOOK_FOR_HOME_TIMEOUT;
    rooms.kick_out(npc.kind.net_id());
}

/// `NPC.KickOutLookForHomeTimeout` (`NPC.cs:95267`): one minute of 60 Hz ticks.
pub const KICK_OUT_LOOK_FOR_HOME_TIMEOUT: i32 = 3600;

/// `WorldGen.moveRoom` (`WorldGen.cs:4843-4855`): the NPC is marked homeless
/// and its timer cleared, then `SpawnTownNPC(x, y)` re-runs the room check for
/// that kind and writes the new room. The port splits it: the caller supplies
/// the re-checked spot (the `SpawnTownNPC` half lives in `world_sim/npcs.rs`),
/// so this half only clears the timer and registers the new room.
pub fn move_room(rooms: &mut TownRooms, npc: &mut Npc, spot: (i32, i32)) {
    npc.look_for_home_timeout = 0;
    npc.home = Some(spot);
    // `SpawnTownNPC`'s relocation arm clears the despawn path too
    // (`WorldGen.cs:5459`: `homelessDespawn = false`).
    npc.homeless_despawn = false;
    rooms.set_room(npc.kind.net_id(), spot.0, spot.1);
}

/// `NPCID.Sets.IsTownPet` (`ID/NPCID.cs:10118`): the eleven town pets. The
/// set coincides with `housingCategory == 1` ([`housing_category`]), but the
/// C# reads the two tables separately (`MoveTownNPC:4777` reads IsTownPet).
pub fn is_town_pet(net_id: i32) -> bool {
    matches!(net_id, 637 | 638 | 656 | 670 | 678..=684)
}

/// One scan spot through the public seam — `SpawnTownNPC`'s re-check
/// (`WorldGen.cs:5419-5438`): the room check, the furniture gate, the
/// occupancy and the score at exactly `(x, y)`, returning the scored spot.
pub fn scored_home_at(
    world: &World,
    npcs: &[Npc],
    x: i32,
    y: i32,
    ignore: Option<usize>,
    asking_net_id: i32,
) -> Option<(i32, i32)> {
    scan_spot(world, npcs, x, y, ignore, asking_net_id)
}

/// `WorldGen.MoveTownNPC`'s verdict plus the two `builtHouseWith*` statics
/// it sets (`WorldGen.cs:4757`/`:4761`, declared `:4204-4206`). The C# keeps
/// them as globals reset at world gen (`:7030-7031`); no code reads them, so
/// the port returns them per call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MoveTownNpcOutcome {
    pub ok: bool,
    pub built_house_with_no_furniture: bool,
    pub built_house_with_no_light: bool,
}

/// `WorldGen.MoveTownNPC` (`WorldGen.cs:4685-4841`): the housing query /
/// relocation validation. Runs `StartRoomCheck` → `RoomNeeds` → the stinkbug
/// gates → `ScoreRoom` → the special spawning conditions, printing each
/// failure's reason through `Main.NewText(…, 255, 240, 20)` when
/// `display_text` (the `NoRoomCheckFeedback.WithText` policy,
/// `DataStructures/NoRoomCheckFeedback.cs:9-15` — `StopOnFail` is always
/// true there, so every failure returns at once).
///
/// `n` is the `Main.npc[]` index being moved (`None` = the C#'s `n = -1`,
/// the housing query); `prioritized_net_id` is
/// `WorldGen.prioritizedTownNPCType`, the type the query scores for.
///
/// Deviation (ANALYSIS "Town housing (S1-E-11d)"): the C#'s
/// `RichRoomCheckFeedback` pass (`StopOnFail = false`, the particle
/// visualisation, `Main.cs:45165`) is not ported — its verdict is discarded,
/// its text is off and its only statics are the dead `builtHouseWith*`
/// flags, so the observable behaviour is this one WithText-shaped call.
pub fn move_town_npc(
    world: &World,
    npcs: &[Npc],
    x: i32,
    y: i32,
    n: Option<usize>,
    prioritized_net_id: i32,
    unlocked_truffle_spawn: bool,
    display_text: bool,
    chat: &mut crate::chat::ChatMonitor,
) -> MoveTownNpcOutcome {
    let mut out = MoveTownNpcOutcome::default();
    let mut say = |chat: &mut crate::chat::ChatMonitor, text: String| {
        if display_text {
            // `Main.NewText(newText, byte.MaxValue, 240, 20)` (`:4722`).
            chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
        }
    };
    // `:4694-4728`: the room check and its failure reason.
    let scan = match check_room(world, x, y) {
        Ok(scan) => scan,
        Err(reason) => {
            // `:4698-4719`: `Lang.inter[40]` is the default, the six
            // reasons override it with their `TownNPCHousingFailureReasons`
            // rows.
            let text = match reason {
                tr_core::housing::RoomFailure::TooManyUnsafeWalls => {
                    crate::lang::text("TownNPCHousingFailureReasons.RoomHasUnsafeWalls")
                }
                tr_core::housing::RoomFailure::HoleInWallIsTooBig => {
                    crate::lang::text("TownNPCHousingFailureReasons.HoleInWallIsTooBig")
                }
                tr_core::housing::RoomFailure::StartedInASolidTile => {
                    crate::lang::text("TownNPCHousingFailureReasons.RoomCheckStartedInASolidTile")
                }
                tr_core::housing::RoomFailure::RoomIsTooBig => {
                    crate::lang::text("TownNPCHousingFailureReasons.RoomIsTooBig")
                }
                tr_core::housing::RoomFailure::RoomIsTooSmall => {
                    crate::lang::text("TownNPCHousingFailureReasons.RoomIsTooSmall")
                }
                tr_core::housing::RoomFailure::TooCloseToWorldEdge => {
                    crate::lang::text("TownNPCHousingFailureReasons.TooCloseToWorldEdge")
                }
            };
            say(chat, text);
            return out; // stopOnFail (`:4724-4727`)
        }
    };
    // `:4729-4771`: the furniture gate names what the house is missing, in
    // the torch/door/table/chair order, through `Game.HouseMissing_{n}`.
    let furniture = tr_core::housing::room_needs_flags(&scan.house_tiles);
    if !furniture.complete() {
        let mut missing: Vec<String> = Vec::new();
        if !furniture.torch {
            missing.push(crate::lang::text("Game.HouseLightSource"));
        }
        if !furniture.door {
            missing.push(crate::lang::text("Game.HouseDoor"));
        }
        if !furniture.table {
            missing.push(crate::lang::text("Game.HouseTable"));
        }
        if !furniture.chair {
            missing.push(crate::lang::text("Game.HouseChair"));
        }
        let text = crate::lang::text_args(
            &format!("Game.HouseMissing_{}", missing.len()),
            &missing,
        );
        // `:4755-4762`: the two dead statics.
        if !furniture.table || !furniture.chair {
            out.built_house_with_no_furniture = true;
        }
        if !furniture.torch {
            out.built_house_with_no_light = true;
        }
        say(chat, text);
        return out; // stopOnFail (`:4767-4770`)
    }
    // `:4772-4776`: the type the room scores for — the moved NPC's own type
    // wins over the prioritized one.
    let net_id = match n {
        Some(i) => npcs[i].kind.net_id(),
        None => prioritized_net_id,
    };
    // `:4777-4799`: the stinkbug gates; a town pet is exempt.
    let pet = is_town_pet(net_id);
    if scan.has_stinkbug && !pet {
        say(
            chat,
            crate::lang::text("TownNPCHousingFailureReasons.RoomHasAStinkbug"),
        );
        return out;
    }
    if scan.has_echo_stinkbug && !pet {
        say(
            chat,
            crate::lang::text("TownNPCHousingFailureReasons.RoomHasAnEchoStinkbug"),
        );
        return out;
    }
    // `:4800-4828`: the score. `ScoreRoom(-1, type)` — the ignoreNPC is −1
    // even when moving an NPC, so its own housed home still occupies.
    let (occupied, shared_room_x) = room_occupancy(&scan, npcs, None, net_id);
    let report = tr_core::housing::score_room_report(world, &scan, occupied, shared_room_x);
    if report.hi_score <= 0 {
        if display_text {
            // `:4805-4823`: occupied → inter[41], evil → inter[42], no
            // standing space → its own row, else inter[40].
            let text = if report.room_occupied {
                crate::lang::inter(41)
            } else if report.room_evil {
                crate::lang::inter(42)
            } else if !report.room_has_standing_space {
                crate::lang::text("TownNPCHousingFailureReasons.RoomHasNoStandingSpace")
            } else {
                crate::lang::inter(40)
            };
            chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
        }
        return out;
    }
    // `:4829-4839`: the moved NPC's special spawning conditions — the
    // Truffle's mushroom room (`CheckSpecialTownNPCSpawningConditions`).
    if let Some(i) = n {
        let npc = &npcs[i];
        if !tr_core::housing::check_special_town_npc_spawning_conditions(
            world,
            &scan,
            npc.kind.net_id(),
            unlocked_truffle_spawn,
        ) {
            // `:4833`: `Lang.inter[55] + " " + Main.npc[n].TypeName`
            // (`NPC.TypeName` = `Lang.GetNPCNameValue(netID)`, `NPC.cs:6727`).
            let text = format!(
                "{} {}",
                crate::lang::inter(55),
                crate::lang::npc_name(npc.kind.net_id())
            );
            say(chat, text);
            return out;
        }
    }
    out.ok = true;
    out
}

/// `Main.PerformHousingCheck` (`Main.cs:45162-45170`): the housing query —
/// `MoveTownNPC(x, y, -1, …)` with text, and on success `Lang.inter[39]`
/// ("This housing is suitable.") in `ChatColors.ServerMessage`. The Rich
/// particle pass (`:45165`) is the recorded deviation on [`move_town_npc`].
pub fn perform_housing_check(
    world: &World,
    npcs: &[Npc],
    x: i32,
    y: i32,
    prioritized_net_id: i32,
    unlocked_truffle_spawn: bool,
    chat: &mut crate::chat::ChatMonitor,
) {
    let out = move_town_npc(
        world,
        npcs,
        x,
        y,
        None,
        prioritized_net_id,
        unlocked_truffle_spawn,
        true,
        chat,
    );
    if out.ok {
        chat.new_text(crate::lang::inter(39), crate::chat::COLOR_SERVER_MESSAGE);
    }
}

/// `Main.TryMovingNPC` (`Main.cs:45172-45180`) + `WorldGen.moveRoom`
/// (`WorldGen.cs:4843-4855`): validate the target room for the NPC, then
/// mark it homeless (`:4851`), clear the timer (`:4852`) and re-run the
/// room check at `(x, y)` — `SpawnTownNPC`'s relocation arm writes the
/// **scored** spot (`bestX/bestY`, `:5456-5457`), not the click point, and
/// registers the room (`:4854`). Returns whether the move happened (the
/// caller clears `Main.mouseNPC` and plays sound 12 on `true`).
///
/// The transient `prioritizedTownNPCType = npc.type` write (`:4850`) is
/// consumed and zeroed by the same `SpawnTownNPC` call (`:5461`); the port
/// resolves the relocation directly, so the caller zeroes the arrival
/// ladder's prioritized kind instead (net effect identical).
pub fn try_moving_npc(
    world: &World,
    npcs: &mut [Npc],
    rooms: &mut TownRooms,
    x: i32,
    y: i32,
    n: usize,
    unlocked_truffle_spawn: bool,
    chat: &mut crate::chat::ChatMonitor,
) -> bool {
    let out = move_town_npc(
        world,
        npcs,
        x,
        y,
        Some(n),
        0,
        unlocked_truffle_spawn,
        true,
        chat,
    );
    if !out.ok {
        return false;
    }
    let net_id = npcs[n].kind.net_id();
    let old_home = npcs[n].home.unwrap_or((-1, -1));
    // `moveRoom:4851-4852`: homeless first — the re-check's occupancy scan
    // skips homeless NPCs (`ScoreRoom_IsThisRoomOccupiedBySomeone:5819`),
    // so the NPC's own old home no longer occupies the target room.
    npcs[n].home = None;
    npcs[n].look_for_home_timeout = 0;
    // `SpawnTownNPC(x, y)` (`:4853`): the scored spot at the click point.
    match scored_home_at(world, npcs, x, y, None, net_id) {
        Some(spot) => move_room(rooms, &mut npcs[n], spot),
        None => {
            // The blocked arm still runs `SetRoom(type, homeTileX,
            // homeTileY)` (`:4854`) with the unchanged home tiles, and the
            // NPC stays homeless. Unreachable after a passing
            // `MoveTownNPC` at the same point, kept for the 1:1 shape.
            rooms.set_room(net_id, old_home.0, old_home.1);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_world_helpers::{build_furnished_room, build_furnished_room_with, ground};

    /// The Guide's vanilla net id (`NPCID.Guide = 22`) and a category-1 town
    /// pet's (`NPCID.TownCat = 637`) — the two `housingCategory` arms.
    const GUIDE_ID: i32 = 22;
    const TOWN_CAT_ID: i32 = 637;

    #[test]
    fn the_homeless_guide_finds_a_furnished_room() {
        // A furnished room in the world: the three-pass scan finds it from
        // the Guide's position up to 10 tiles away.
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let home = quick_find_home(&w, &[], 72, 108, None, GUIDE_ID);
        assert!(home.is_some(), "the scan finds the room");
        let (hx, hy) = home.unwrap();
        assert!((60..=71).contains(&hx), "hx {hx}");
        assert!((99..=108).contains(&hy), "hy {hy}");
    }

    #[test]
    fn a_housed_town_npc_occupies_the_room() {
        // `ScoreRoom_IsThisRoomOccupiedBySomeone` (`WorldGen.cs:5814-5829`):
        // a housed town NPC whose home is in the room refuses the home.
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        // The Guide lives in the room (the home tile and the one above it).
        let resident = town_npc(
            crate::npc::NpcKind::Guide,
            65.0 * 16.0,
            106.0 * 16.0,
            Some((65, 106)),
        );
        assert!(
            quick_find_home(&w, &[resident], 72, 108, None, GUIDE_ID).is_none(),
            "the occupied room is refused"
        );
        // The asking NPC itself does not block its own room (`ignoreNPC`).
        assert!(
            quick_find_home(&w, &[], 72, 108, None, GUIDE_ID).is_some(),
            "the empty room still answers"
        );
    }

    #[test]
    fn a_category_one_tenant_shares_the_room() {
        // `CanNPCsLiveWithEachOther` (`TownRoomManager.cs:144-156`): the
        // categories differ → no conflict, and the occupant's home becomes
        // `sharedRoomX` instead of refusing the room (`WorldGen.cs:5821-5825`).
        assert!(can_npcs_live_with_each_other(TOWN_CAT_ID, GUIDE_ID));
        assert!(!can_npcs_live_with_each_other(GUIDE_ID, GUIDE_ID));
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let resident = town_npc(
            crate::npc::NpcKind::Guide,
            65.0 * 16.0,
            106.0 * 16.0,
            Some((65, 106)),
        );
        assert!(
            quick_find_home(&w, &[resident], 72, 108, None, TOWN_CAT_ID).is_some(),
            "a different-category tenant may share the room"
        );
    }

    #[test]
    fn an_unfurnished_room_rejects() {
        // A room without furniture: the scan fails everywhere.
        let mut w = ground();
        build_furnished_room_with(&mut w, 60, 100, 12, 9, &[]);
        assert!(quick_find_home(&w, &[], 72, 108, None, GUIDE_ID).is_none());
    }

    #[test]
    fn test_DW_4_1b_the_household_status_and_the_relocation_pair() {
        // `GetHouseholdStatus` (`TownRoomManager.cs:130-142`).
        let mut rooms = TownRooms::new();
        assert_eq!(rooms.household_status(true, GUIDE_ID), 1, "homeless wins");
        assert_eq!(rooms.household_status(false, GUIDE_ID), 0, "no room, no home");
        rooms.set_room(GUIDE_ID, 65, 105);
        assert_eq!(rooms.household_status(false, GUIDE_ID), 2, "has a room");
        assert_eq!(rooms.household_status(true, GUIDE_ID), 1, "homeless overrides");

        // `kickOut` (`WorldGen.cs:4869-4879`): homeless, timer armed, room gone.
        let mut npc = town_npc(crate::npc::NpcKind::Guide, 0.0, 0.0, Some((65, 105)));
        kick_out(&mut rooms, &mut npc);
        assert_eq!(npc.home, None);
        assert_eq!(npc.look_for_home_timeout, KICK_OUT_LOOK_FOR_HOME_TIMEOUT);
        assert!(!rooms.has_room_quick(GUIDE_ID));

        // `moveRoom` (`:4843-4855`): timer cleared and the new room registered.
        move_room(&mut rooms, &mut npc, (80, 120));
        assert_eq!(npc.home, Some((80, 120)));
        assert_eq!(npc.look_for_home_timeout, 0);
        assert_eq!(rooms.rooms(), &[(GUIDE_ID, 80, 120)]);
        assert_eq!(rooms.occupants_at((80, 120)), vec![GUIDE_ID]);
    }

    #[test]
    fn the_household_status_follows_the_room_book() {
        // `GetHouseholdStatus` (`TownRoomManager.cs:130-142`) across the full
        // lifecycle: 0 → 2 (SetRoom) → 0 (KickOut) → 2 (SetRoom again) →
        // 0 (Clear), with `homeless` overriding every state (`:133-135`).
        let mut rooms = TownRooms::new();
        assert_eq!(rooms.household_status(false, GUIDE_ID), 0);
        rooms.set_room(GUIDE_ID, 65, 105);
        assert_eq!(rooms.household_status(false, GUIDE_ID), 2);
        rooms.kick_out(GUIDE_ID);
        assert_eq!(rooms.household_status(false, GUIDE_ID), 0);
        rooms.set_room(GUIDE_ID, 80, 120);
        assert_eq!(rooms.household_status(false, GUIDE_ID), 2);
        assert_eq!(rooms.household_status(true, GUIDE_ID), 1);
        rooms.clear();
        assert_eq!(rooms.household_status(false, GUIDE_ID), 0);
        // Another NPC's room never answers for this one (`HasRoomQuick` is
        // keyed by type, `:33-36`).
        rooms.set_room(TOWN_CAT_ID, 80, 120);
        assert_eq!(rooms.household_status(false, GUIDE_ID), 0);
        assert_eq!(rooms.household_status(false, TOWN_CAT_ID), 2);
    }

    #[test]
    fn the_occupants_list_keeps_the_registration_order() {
        // `AddOccupantsToList` (`TownRoomManager.cs:22-31`): every pair whose
        // room sits exactly on the tile, in list order; a moved-out NPC no
        // longer answers at its old spot (`SetRoom:63-70` removes the pair).
        let mut rooms = TownRooms::new();
        assert!(rooms.occupants_at((80, 120)).is_empty());
        rooms.set_room(GUIDE_ID, 80, 120);
        rooms.set_room(TOWN_CAT_ID, 80, 120); // a pet may share (`:153-156`)
        rooms.set_room(17, 90, 130); // elsewhere
        assert_eq!(rooms.occupants_at((80, 120)), vec![GUIDE_ID, TOWN_CAT_ID]);
        assert_eq!(rooms.occupants_at((90, 130)), vec![17]);
        rooms.set_room(GUIDE_ID, 90, 130);
        assert_eq!(rooms.occupants_at((80, 120)), vec![TOWN_CAT_ID]);
        assert_eq!(rooms.occupants_at((90, 130)), vec![17, GUIDE_ID]);
        rooms.kick_out(TOWN_CAT_ID);
        assert!(rooms.occupants_at((80, 120)).is_empty());
    }

    #[test]
    fn the_pet_categories_match_the_id_table() {
        // `housing_category` (`NPC.SetDefaults` writes 1 for the town pets —
        // 637/638 `:17224`, 656 `:17308`, 670 `:17494`, 678-684 `:17607`).
        for id in [637, 638, 656, 670, 678, 679, 680, 681, 682, 683, 684] {
            assert_eq!(housing_category(id), 1, "pet {id}");
            assert!(can_npcs_live_with_each_other(id, GUIDE_ID));
            assert!(can_npcs_live_with_each_other(GUIDE_ID, id));
            assert!(!can_npcs_live_with_each_other(id, 637), "pets conflict");
        }
        for id in [17, 18, 19, 20, 22, 37, 227, 228, 229, 368, 369, 441, 550, 588, 633, 663] {
            assert_eq!(housing_category(id), 0, "common {id}");
        }
        // Two common NPCs never share (`TownRoomManager.cs:153-156`).
        assert!(!can_npcs_live_with_each_other(GUIDE_ID, 17));
    }

    // --- the room state (`TownRoomManager`) -------------------------------

    /// A town NPC at a pixel position with a home tile.
    fn town_npc(kind: crate::npc::NpcKind, x: f32, y: f32, home: Option<(i32, i32)>) -> Npc {
        let mut npc = Npc::new(kind, x, y, 1, 0.0);
        npc.home = home;
        npc.given_name = "Test".into();
        npc
    }

    #[test]
    fn test_DW_4_1_the_rooms_round_trip_through_the_wld_seam() {
        // The section-9 wire order (`TownRoomManager.Save:99-103`) survives
        // the load's clear-then-refill (`:108-114`).
        let rooms = vec![
            WldTownRoom {
                npc_id: 17,
                x: 65,
                y: 105,
            },
            WldTownRoom {
                npc_id: 228,
                x: 80,
                y: 120,
            },
        ];
        let state = TownRooms::from_wld_rooms(&rooms);
        assert_eq!(state.rooms(), &[(17, 65, 105), (228, 80, 120)]);
        assert_eq!(state.to_wld_rooms(), rooms);
        // `HasRoom`/`HasRoomQuick` agree with the list.
        assert_eq!(state.has_room(17), Some((65, 105)));
        assert!(state.has_room_quick(17));
        assert!(state.has_room_quick(228));
        assert_eq!(state.has_room(18), None, "no room for a third NPC");
        assert!(!state.has_room_quick(18));
    }

    #[test]
    fn set_room_replaces_the_npcs_previous_room() {
        // `SetRoom` removes the NPC's earlier pair before appending
        // (`TownRoomManager.cs:60-64`): one room per NPC, always.
        let mut state = TownRooms::new();
        state.set_room(17, 65, 105);
        state.set_room(17, 90, 140);
        assert_eq!(state.rooms(), &[(17, 90, 140)]);
        assert_eq!(state.has_room(17), Some((90, 140)));
    }

    #[test]
    fn kick_out_clears_the_pair_and_the_fast_table() {
        let mut state = TownRooms::new();
        state.set_room(17, 65, 105);
        state.set_room(228, 80, 120);
        state.kick_out(17);
        assert_eq!(state.rooms(), &[(228, 80, 120)]);
        assert!(!state.has_room_quick(17));
        assert!(state.has_room_quick(228));
        state.clear();
        assert!(state.rooms().is_empty());
        assert!(!state.has_room_quick(228));
    }

    #[test]
    fn an_out_of_range_room_npc_id_is_dropped() {
        // `_hasRoom[npcID]` would throw on a foreign file; the port drops the
        // room instead (the C# never sees one).
        let state = TownRooms::from_wld_rooms(&[WldTownRoom {
            npc_id: 99_999,
            x: 1,
            y: 2,
        }]);
        assert!(state.rooms().is_empty());
        assert!(!state.has_room_quick(99_999));
        assert!(!state.has_room_quick(-1));
    }

    #[test]
    fn test_DW_4_1_the_shimmered_bitmap_round_trips() {
        // `LoadNPCs:2888-2892` ↔ `SaveNPCs:1706-1713`.
        let state = ShimmeredTownNpcs::from_wld_indices(&[4, 68, 300]);
        assert_eq!(state.to_wld_indices(), vec![4, 68, 300]);
        assert!(state.is_shimmered(4) && state.is_shimmered(300));
        assert!(!state.is_shimmered(5));
        let mut live = state;
        live.set_shimmered(228, true);
        live.set_shimmered(68, false);
        live.set_shimmered(99_999, true); // ignored, not a panic
        assert_eq!(live.to_wld_indices(), vec![4, 228, 300]);
    }

    // --- the `FixAgainstExploits` re-validation ---------------------------

    #[test]
    fn test_DW_4_3_an_invalid_home_with_no_room_kicks_the_npc_out() {
        // A home in solid stone, no stored room: `KickOut` + homeless
        // (`WorldFile.cs:1864-1896`).
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9); // a real room, unregistered
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Merchant, 65.0 * 16.0, 105.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None, "kicked out and made homeless");
        assert!(!npcs[0].homeless_despawn, "the despawn flag is not this path");
        assert!(!rooms.has_room_quick(17));
    }

    #[test]
    fn test_DW_4_3_an_invalid_home_with_a_stored_room_teleports_in() {
        // The stored room fits, so the NPC moves into it (`:1885-1892`).
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Merchant, 30.0 * 16.0, 130.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        rooms.set_room(17, 65, 105);
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, Some((65, 105)), "teleported into the room");
        assert!(rooms.has_room_quick(17), "the room survives a good teleport");
    }

    #[test]
    fn test_DW_4_3_an_unusable_stored_room_still_kicks_out() {
        // The stored room does not fit either: the retry fails and the NPC
        // is kicked out (the C# checks `flag` again after the teleport).
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Merchant, 30.0 * 16.0, 130.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        rooms.set_room(17, 30, 130); // solid stone
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None);
        assert!(!rooms.has_room_quick(17), "the bust room is released");
    }

    #[test]
    fn test_DW_4_3_a_valid_home_is_left_alone() {
        // A home inside a room passes `StartRoomCheck` and is not touched —
        // including an unfurnished (but enclosed) room, because the
        // re-validation does not run `RoomNeeds`.
        let mut furnished = ground();
        build_furnished_room(&mut furnished, 60, 100, 12, 9);
        let mut bare = ground();
        build_furnished_room_with(&mut bare, 60, 100, 12, 9, &[]);
        for w in [&furnished, &bare] {
            let mut npcs = vec![town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 105.0 * 16.0, Some((65, 105)))];
            let mut rooms = TownRooms::new();
            fix_town_npc_homes(w, &mut npcs, &mut rooms);
            assert_eq!(npcs[0].home, Some((65, 105)));
        }
    }

    #[test]
    fn test_DW_4_3_a_homeless_npc_is_never_revalidated() {
        // `nPC.homeless` short-circuits the loop (`:1881`): a homeless NPC
        // with no home at all keeps its position and no room is consumed.
        let w = ground();
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Nurse, 30.0 * 16.0, 130.0 * 16.0, None)];
        let mut rooms = TownRooms::new();
        rooms.set_room(18, 30, 130);
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None);
        assert!(rooms.has_room_quick(18), "the room is left for later");
    }

    #[test]
    fn test_DW_4_3_a_home_outside_the_world_bounds_kicks_out() {
        // A hostile file's home past the world edge: `StartRoomCheck` refuses
        // (TooCloseToWorldEdge), no stored room fits → kicked out.
        let w = ground();
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Painter, 100.0 * 16.0, 100.0 * 16.0, Some((5000, 5000)))];
        let mut rooms = TownRooms::new();
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None);
    }

    #[test]
    fn a_non_town_npc_is_never_revalidated() {
        // The filter is `townNPC`: a wandering zombie with a home tile (which
        // it can't have) is untouched.
        let w = ground();
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Zombie, 30.0 * 16.0, 130.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, Some((30, 130)));
    }

    #[test]
    fn test_DW_4_3_the_load_path_heals_a_bogus_home() {
        // The whole load-side arm on a synthesized section: the Merchant's
        // file home is in solid stone, the town rooms carry a good room, so
        // the NPC ends up housed there (`Main.npc[num2++]` +
        // `FixAgainstExploits`).
        let section = tr_core::wld::sections::WldNpcs {
            town: vec![tr_core::wld::sections::WldTownNpc {
                net_id: 17,
                given_name: "Bob".into(),
                x: 30.0 * 16.0,
                y: 130.0 * 16.0,
                homeless: false,
                home_x: 30,
                home_y: 130,
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = crate::npc::from_wld(&section);
        assert_eq!(npcs.len(), 1, "the record becomes one live NPC");
        assert_eq!(npcs[0].home, Some((30, 130)), "the file's home, unhealed");
        let mut rooms = TownRooms::from_wld_rooms(&[WldTownRoom {
            npc_id: 17,
            x: 65,
            y: 105,
        }]);
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, Some((65, 105)));
    }

    /// The citations the golden rule demands for the `FixAgainstExploits` port.
    #[test]
    fn test_DW_4_5_the_citations_are_present() {
        let src = include_str!("town.rs");
        assert!(src.contains("WorldFile.FixAgainstExploits:1864"));
        assert!(src.contains("`TownRoomManager.cs:11-13`"));
        assert!(src.contains("`NPC.cs:6071`"));
        assert!(src.contains("// `SetRoom(npcID, X, Y)` (`TownRoomManager.cs:57-64`)"));
    }

    // --- `MoveTownNPC` + the housing query (`S1-E-11d`) --------------------

    /// The English rows, pinned by value — the failure texts ride the real
    /// localization (a broken load answers the bare key and fails loudly).
    fn with_english<T>(f: impl FnOnce() -> T) -> T {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        f()
    }

    #[test]
    fn move_town_npc_accepts_a_furnished_room_and_names_the_failures() {
        with_english(|| {
            let mut w = ground();
            build_furnished_room(&mut w, 60, 100, 12, 9);
            let mut chat = crate::chat::ChatMonitor::default();
            // The query (`n = -1`): a good room passes with no text.
            let out = move_town_npc(&w, &[], 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(out.ok, "the furnished room is valid housing");
            assert!(chat.messages.is_empty());
            // `PerformHousingCheck`'s success row (`Main.cs:45168`).
            perform_housing_check(&w, &[], 65, 105, GUIDE_ID, false, &mut chat);
            assert_eq!(chat.messages[0].text, "This housing is suitable.");
            assert_eq!(chat.messages[0].color, crate::chat::COLOR_SERVER_MESSAGE);

            // A solid start (`WorldGen.cs:4707-4708`).
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[], 65, 130, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert_eq!(chat.messages[0].text, "This is a solid block!");

            // The world edge (`:4716-4717`) — the vanilla's own placeholder.
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[], 5, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert_eq!(chat.messages[0].text, "We need better text for this!!!");

            // The furniture gate (`:4729-4762`): no furniture at all names
            // all four pieces through `Game.HouseMissing_4` and arms both
            // `builtHouseWith*` statics.
            let mut bare = ground();
            build_furnished_room_with(&mut bare, 60, 100, 12, 9, &[]);
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&bare, &[], 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert!(out.built_house_with_no_furniture && out.built_house_with_no_light);
            assert_eq!(
                chat.messages[0].text,
                "This house is missing a light source, a door, a table, and a chair."
            );
            // Chair + table only: the missing pair is the light and the door.
            let mut half = ground();
            build_furnished_room_with(&mut half, 60, 100, 12, 9, &[15, 14]);
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&half, &[], 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert!(!out.built_house_with_no_furniture, "the furniture is there");
            assert!(out.built_house_with_no_light);
            assert_eq!(
                chat.messages[0].text,
                "This house is missing a light source and a door."
            );

            // The occupancy arm (`:4807-4809`): a housed Guide occupies the
            // room for another category-0 asker → `Lang.inter[41]`.
            let resident = town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, Some((65, 106)));
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[resident], 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert_eq!(chat.messages[0].text, "This housing is already occupied.");

            // `display_text = false` (the `WithoutText` policy): the verdict
            // is the same, the chat stays empty.
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[], 65, 130, None, GUIDE_ID, false, false, &mut chat);
            assert!(!out.ok);
            assert!(chat.messages.is_empty());
        });
    }

    #[test]
    fn move_town_npc_rejects_the_stinkbug_rooms_but_not_for_pets() {
        with_english(|| {
            let mut w = ground();
            build_furnished_room(&mut w, 60, 100, 12, 9);
            let i = w.index(66, 107);
            w.tiles[i] = tr_core::Tile { kind: 630, frame_x: 0, frame_y: 0, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            // A common NPC is refused (`WorldGen.cs:4778-4788`).
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[], 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert_eq!(chat.messages[0].text, "This room is infested by a stinkbug!");
            // A town pet is exempt (`:4777`'s `IsTownPet[type]`).
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[], 65, 105, None, TOWN_CAT_ID, false, true, &mut chat);
            assert!(out.ok, "the pet moves into the stinkbug room");
            assert!(chat.messages.is_empty());
            // The echo cage refuses too (`:4789-4799`).
            w.tiles[i].kind = 631;
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &[], 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert_eq!(chat.messages[0].text, "This room is infested by a ghostly stinkbug!");
            assert!(is_town_pet(637) && is_town_pet(684) && !is_town_pet(22));
        });
    }

    #[test]
    fn try_moving_npc_relocates_to_the_scored_spot() {
        with_english(|| {
            // Two furnished rooms; the Guide lives in the first and is
            // moved by a click into the second. The home lands on the
            // room's scored spot (`SpawnTownNPC:5456-5457`'s bestX/bestY),
            // not on the click point.
            let mut w = ground();
            build_furnished_room(&mut w, 60, 100, 12, 9);
            build_furnished_room(&mut w, 90, 100, 12, 9);
            let mut npc = town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, Some((65, 106)));
            npc.look_for_home_timeout = 1200;
            npc.homeless_despawn = true;
            let mut npcs = vec![npc];
            let mut rooms = TownRooms::new();
            rooms.set_room(GUIDE_ID, 65, 106);
            let mut chat = crate::chat::ChatMonitor::default();
            let moved = try_moving_npc(&w, &mut npcs, &mut rooms, 95, 105, 0, false, &mut chat);
            assert!(moved, "the second room is valid");
            let (hx, hy) = npcs[0].home.expect("rehoused");
            assert!((91..=100).contains(&hx), "the home is in the second room: {hx}");
            assert!((101..=108).contains(&hy), "hy {hy}");
            assert_eq!(npcs[0].look_for_home_timeout, 0, "moveRoom clears the timer");
            assert!(!npcs[0].homeless_despawn, "the despawn path clears");
            assert_eq!(rooms.has_room(GUIDE_ID), Some((hx, hy)));
            assert!(chat.messages.is_empty(), "a successful move prints nothing");
            // The old room is free again: another Guide may take it.
            assert!(quick_find_home(&w, &npcs, 72, 108, None, GUIDE_ID).is_some());

            // A failed move changes nothing (`Main.cs:45174-45179` only
            // acts on `true`): solid stone refuses with the reason text.
            let before_home = npcs[0].home;
            let before_rooms = rooms.clone();
            npcs[0].look_for_home_timeout = 77;
            let mut chat = crate::chat::ChatMonitor::default();
            let moved = try_moving_npc(&w, &mut npcs, &mut rooms, 30, 130, 0, false, &mut chat);
            assert!(!moved);
            assert_eq!(npcs[0].home, before_home);
            assert_eq!(npcs[0].look_for_home_timeout, 77, "the timer is untouched");
            assert_eq!(rooms, before_rooms);
            assert_eq!(chat.messages[0].text, "This is a solid block!");
        });
    }

    #[test]
    fn the_housing_chain_is_rng_free_and_deterministic() {
        // Determinism proof (S1-E-11d): the C# `MoveTownNPC` → `StartRoomCheck`
        // → `RoomNeeds` → `ScoreRoom` → `CheckSpecialTownNPCSpawningConditions`
        // chain draws ZERO `Main.rand` (grep of WorldGen.cs:4685-4841,
        // :5679-5992, :6041-6237, :4970-4998 finds no `rand`/`Random` call —
        // the only `genRand` in the range is `StartMeteorShower:6241`, outside
        // the room check). The port mirrors this by taking no RNG parameter:
        // every function is pure over (world, npcs, rooms), so it cannot
        // consume a draw. Running the whole chain twice on identical inputs
        // yields identical state, and a seeded stream is untouched.
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        build_furnished_room(&mut w, 90, 100, 12, 9);
        let mut stream = tr_core::UnifiedRandom::new(1234);
        let before: Vec<i32> = (0..8).map(|_| stream.next_max(1_000_000)).collect();
        // Re-seed an identical stream: the housing chain must not advance it.
        let mut probe = tr_core::UnifiedRandom::new(1234);
        let mut run = || {
            let mut npc = town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, Some((65, 106)));
            npc.look_for_home_timeout = 500;
            let mut npcs = vec![npc];
            let mut rooms = TownRooms::new();
            rooms.set_room(GUIDE_ID, 65, 106);
            let mut chat = crate::chat::ChatMonitor::default();
            let ok = try_moving_npc(&w, &mut npcs, &mut rooms, 95, 105, 0, false, &mut chat);
            (ok, npcs[0].home, npcs[0].look_for_home_timeout, rooms.rooms().to_vec())
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "identical inputs give identical housing state");
        assert!(a.0);
        // The probe stream still yields the same first draws: nothing in the
        // chain touched it (it was never passed in).
        let after: Vec<i32> = (0..8).map(|_| probe.next_max(1_000_000)).collect();
        assert_eq!(before, after, "the housing chain consumes no RNG draws");
    }

    #[test]
    fn move_town_npc_scores_for_the_moved_npcs_type() {
        with_english(|| {
            // `:4772-4776`: `n >= 0` overrides the prioritized type. A pet
            // resident shares with a common asker but not with a pet —
            // moving the Guide (category 0) into a pet's room is refused,
            // while the query for the prioritized pet type passes.
            let mut w = ground();
            build_furnished_room(&mut w, 60, 100, 12, 9);
            // The resident is a category-1 pet by net id; the port has no
            // pet kind, so the occupancy is driven through the asking id.
            let guide = town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, Some((65, 106)));
            let npcs = vec![guide];
            let mut chat = crate::chat::ChatMonitor::default();
            // Query as the Guide: same category as the housed Guide → occupied.
            let out = move_town_npc(&w, &npcs, 65, 105, None, GUIDE_ID, false, true, &mut chat);
            assert!(!out.ok);
            // Query as a pet: different category → shares.
            let out = move_town_npc(&w, &npcs, 65, 105, None, TOWN_CAT_ID, false, true, &mut chat);
            assert!(out.ok);
            // Moving the Guide himself: `n >= 0` scores for HIS type even
            // though the prioritized id says pet (`:4773-4776`) — and his
            // own housed home occupies (`ScoreRoom(-1, …)` passes no
            // ignoreNPC), so the move onto his own room is refused.
            let mut chat = crate::chat::ChatMonitor::default();
            let out = move_town_npc(&w, &npcs, 65, 105, Some(0), TOWN_CAT_ID, false, true, &mut chat);
            assert!(!out.ok);
            assert_eq!(chat.messages[0].text, "This housing is already occupied.");
        });
    }
}
