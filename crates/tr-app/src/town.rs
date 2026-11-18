//! The town-NPC housing — `WorldGen.QuickFindHome`'s three-pass scan
//! (`WorldGen.cs:5759-5805`) and the home assignment (`MoveTownNPC`'s tail,
//! `:4848-4854`), consumed by the homeless town NPCs.
//!
//! Also the two world-file states that ride this module (M6-SAVE-1's section
//! 6/9): `TownRooms` = `WorldGen.TownManager` (`TownRoomManager.cs`) with the
//! load-path `FixAgainstExploits` home re-validation, and `ShimmeredTownNpcs`
//! = `NPC.ShimmeredTownNPCs` (`NPC.cs:6071`).

use tr_core::housing::{
    check_room, check_special_town_npc_spawning_conditions, room_needs, score_room,
    start_room_check, RoomFailure, RoomFurniture, RoomScore,
};
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
    let scored = score_room(world, &scan, shared_room_x);
    let (bx, by) = scored.best?;
    if scored.hi_score <= 0 {
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
    rooms.set_room(npc.kind.net_id(), spot.0, spot.1);
}

/// `IRoomCheckFeedback`'s two behaviour flags as `MoveTownNPC` reads them
/// (`WorldGen.cs:4691-4692`). `NoRoomCheckFeedback.WithText` — the default
/// (`:4687-4690`) — stops on the first failure and prints it;
/// `NoRoomCheckFeedback.WithoutText` stops silently
/// (`NoRoomCheckFeedback.cs:9-11`, `:84-85`). `RichRoomCheckFeedback.Instance`
/// (`Main.PerformHousingCheck`'s first pass, `Main.cs:45165`) neither stops
/// nor prints — it only draws the spread/score overlay, which the port does
/// not model; its flags are `stop_on_fail: false, display_text: false`.
#[derive(Clone, Copy, Debug)]
pub struct RoomCheckFeedback {
    pub stop_on_fail: bool,
    pub display_text: bool,
}

impl RoomCheckFeedback {
    /// `NoRoomCheckFeedback.WithText` (`NoRoomCheckFeedback.cs:84`).
    pub const WITH_TEXT: Self = Self { stop_on_fail: true, display_text: true };
    /// `NoRoomCheckFeedback.WithoutText` (`:85`).
    pub const WITHOUT_TEXT: Self = Self { stop_on_fail: true, display_text: false };
    /// `RichRoomCheckFeedback.Instance`'s flags (both false).
    pub const RICH: Self = Self { stop_on_fail: false, display_text: false };
}

/// `NPCID.Sets.IsTownPet` (`ID/NPCID.cs:10118`): the eleven pet ids the
/// stinkbug refusals exempt (`WorldGen.cs:4777-4798`).
pub fn is_town_pet(net_id: i32) -> bool {
    matches!(net_id, 637 | 638 | 656 | 670 | 678..=684)
}

/// `WorldGen.builtHouseWithNoFurniture`/`builtHouseWithNoLight`
/// (`WorldGen.cs:4204`/`:4206`): the flags `MoveTownNPC` latches when a room
/// fails `RoomNeeds` without a table/chair or without a light (`:4755-4762`),
/// reset by `WorldGen.clearWorld` (`:7030-7031`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuiltHouseFlags {
    pub no_furniture: bool,
    pub no_light: bool,
}

/// `WorldGen.MoveTownNPC`'s failure feedback (`WorldGen.cs:4685-4841`): the
/// reason ladder's output. Every refused rung pushes its `Main.NewText` line
/// here in source order when `display_text` is set; with `stop_on_fail` the
/// walk ends at the first refusal, so at most one line lands. The color is
/// always `(255, 240, 20)` (`ChatColors.ServerMessage`, `chat.rs`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MoveTownNpcFeedback {
    /// The `Main.NewText` lines, in the order the C# emits them.
    pub messages: Vec<String>,
    /// `WorldGen.builtHouseWithNoFurniture`/`NoLight` latches (`:4755-4762`).
    pub built_house: BuiltHouseFlags,
}

impl MoveTownNpcFeedback {
    /// `Main.NewText(text, byte.MaxValue, 240, 20)` under `displayText`.
    fn say(&mut self, display_text: bool, text: String) {
        if display_text {
            self.messages.push(text);
        }
    }
}

/// `WorldGen.MoveTownNPC` (`WorldGen.cs:4685-4841`): the relocation
/// validation ladder — the room scan (`StartRoomCheck`), the furniture needs
/// (`RoomNeeds`), the stinkbug refusals (pet-exempt), the room score's
/// occupied/evil/no-standing-space ladder and the Truffle's special spawning
/// conditions. Returns whether the room may house the NPC.
///
/// `n` is the asking NPC's index in `npcs` (the C#'s `n`; `None` = the C#'s
/// `n = -1` housing query, whose type falls back to
/// `WorldGen.prioritizedTownNPCType`, passed here as `prioritized_type`).
/// The success arm's home assignment is `move_room`'s (`:4843-4855`); this
/// function is pure validation plus feedback, like the C#.
pub fn move_town_npc(
    world: &World,
    npcs: &[Npc],
    x: i32,
    y: i32,
    n: Option<usize>,
    prioritized_type: i32,
    feedback: RoomCheckFeedback,
    out: &mut MoveTownNpcFeedback,
) -> bool {
    let stop_on_fail = feedback.stop_on_fail;
    let display_text = feedback.display_text;
    // `:4694-4695`: BeginSpread/EndSpread are the rich overlay's hooks —
    // no-ops for every feedback the port models.
    let scan = check_room(world, x, y);
    if let Err(reason) = &scan {
        // `:4698-4719`: Lang.inter[40] unless one of the six reasons has its
        // own TownNPCHousingFailureReasons key.
        let text = match reason {
            RoomFailure::TooManyUnsafeWalls => {
                crate::lang::text("TownNPCHousingFailureReasons.RoomHasUnsafeWalls")
            }
            RoomFailure::HoleInWallIsTooBig => {
                crate::lang::text("TownNPCHousingFailureReasons.HoleInWallIsTooBig")
            }
            RoomFailure::StartedInASolidTile => {
                crate::lang::text("TownNPCHousingFailureReasons.RoomCheckStartedInASolidTile")
            }
            RoomFailure::RoomIsTooBig => {
                crate::lang::text("TownNPCHousingFailureReasons.RoomIsTooBig")
            }
            RoomFailure::RoomIsTooSmall => {
                crate::lang::text("TownNPCHousingFailureReasons.RoomIsTooSmall")
            }
            RoomFailure::TooCloseToWorldEdge => {
                crate::lang::text("TownNPCHousingFailureReasons.TooCloseToWorldEdge")
            }
        };
        out.say(display_text, text);
        if stop_on_fail {
            return false;
        }
    }
    let scan = scan.ok();
    // `:4729-4771`: the missing-furniture list — torch, door, table, chair,
    // in that order, into Game.HouseMissing_<count>.
    if let Some(scan) = &scan {
        let furniture = RoomFurniture::of(&scan.house_tiles);
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
            // `:4755-4762`: the two latch flags, set even without text.
            if !furniture.table || !furniture.chair {
                out.built_house.no_furniture = true;
            }
            if !furniture.torch {
                out.built_house.no_light = true;
            }
            out.say(display_text, text);
            if stop_on_fail {
                return false;
            }
        }
    }
    // `:4772-4776`: the type — the asking NPC's, else the prioritized one.
    let asking = n.and_then(|i| npcs.get(i));
    let npc_type = asking.map_or(prioritized_type, |npc| npc.kind.net_id());
    // `:4777-4798`: the stinkbug refusals — a town pet is exempt from both.
    let pet = is_town_pet(npc_type);
    if scan.as_ref().is_some_and(|s| s.has_stinkbug) && !pet {
        out.say(
            display_text,
            crate::lang::text("TownNPCHousingFailureReasons.RoomHasAStinkbug"),
        );
        if stop_on_fail {
            return false;
        }
    }
    if scan.as_ref().is_some_and(|s| s.has_echo_stinkbug) && !pet {
        out.say(
            display_text,
            crate::lang::text("TownNPCHousingFailureReasons.RoomHasAnEchoStinkbug"),
        );
        if stop_on_fail {
            return false;
        }
    }
    // `:4800-4803`: BeginScoring/EndScoring are the rich overlay's hooks.
    // `ScoreRoom(-1, type)` — the C# passes ignoreNPC = -1 here, so even the
    // asking NPC's own home counts as occupancy.
    let scored = match &scan {
        Some(scan) => {
            let (occupied, shared_room_x) = room_occupancy(scan, npcs, None, npc_type);
            if occupied {
                // `:5868-5872`: roomOccupied, hiScore = -1.
                RoomScore {
                    hi_score: -1,
                    best: None,
                    room_evil: false,
                    room_has_standing_space: false,
                }
            } else {
                score_room(world, scan, shared_room_x)
            }
        }
        // A failed scan leaves the C#'s globals at their reset values
        // (`:5864-5866` never runs) — the port models hiScore 0.
        None => RoomScore {
            hi_score: 0,
            best: None,
            room_evil: false,
            room_has_standing_space: false,
        },
    };
    let occupied = scored.hi_score == -1;
    if scored.hi_score <= 0 {
        // `:4805-4822`: occupied → evil → no standing space → the generic
        // "not valid housing", first match only.
        let text = if occupied {
            crate::lang::inter(41)
        } else if scored.room_evil {
            crate::lang::inter(42)
        } else if !scored.room_has_standing_space {
            crate::lang::text("TownNPCHousingFailureReasons.RoomHasNoStandingSpace")
        } else {
            crate::lang::inter(40)
        };
        out.say(display_text, text);
        if stop_on_fail {
            return false;
        }
    }
    // `:4829-4837`: the Truffle's special conditions — only for a real NPC.
    if let (Some(npc), Some(scan)) = (asking, &scan) {
        // `NPC.unlockedTruffleSpawn` and `Main.NoFunctionalSurface` have no
        // port state (no unlock path, no secret seeds) — both false.
        if !check_special_town_npc_spawning_conditions(world, scan, npc.kind.net_id(), false, false)
        {
            // `Lang.inter[55] + " " + Main.npc[n].TypeName` — the type name,
            // not the given name (`NPC.TypeName`, `Lang.GetNPCNameValue`).
            out.say(
                display_text,
                format!("{} {}", crate::lang::inter(55), crate::lang::npc_name(npc.kind.net_id())),
            );
            if stop_on_fail {
                return false;
            }
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

    // --- `MoveTownNPC`'s validation ladder (S1-E-11c relocation) ----------

    /// The English rows, pinned like `ui/housing.rs`'s text test.
    fn lang_english() -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        guard
    }

    fn run_ladder(
        w: &World,
        npcs: &[Npc],
        x: i32,
        y: i32,
        n: Option<usize>,
        prioritized: i32,
        feedback: RoomCheckFeedback,
    ) -> (bool, MoveTownNpcFeedback) {
        let mut out = MoveTownNpcFeedback::default();
        let ok = move_town_npc(w, npcs, x, y, n, prioritized, feedback, &mut out);
        (ok, out)
    }

    #[test]
    fn a_furnished_empty_room_passes_the_ladder() {
        // `MoveTownNPC` (`WorldGen.cs:4685-4841`): scan → needs → stinkbugs →
        // score → special conditions, all green.
        let _g = lang_english();
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let guide = town_npc(crate::npc::NpcKind::Guide, 0.0, 0.0, None);
        let (ok, out) = run_ladder(&w, &[guide], 65, 105, Some(0), 0, RoomCheckFeedback::WITH_TEXT);
        assert!(ok);
        assert!(out.messages.is_empty());
        assert_eq!(out.built_house, BuiltHouseFlags::default());
    }

    #[test]
    fn the_six_scan_failure_reasons_map_to_their_keys() {
        // `:4699-4719`: each TownNPCRoomCheckFailureReason has its own
        // TownNPCHousingFailureReasons key; the stop-on-fail default returns
        // false with exactly one line.
        let _g = lang_english();
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        // A solid start tile (`RoomCheckStartedInASolidTile`).
        let (ok, out) = run_ladder(&w, &[], 65, 120, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This is a solid block!"]);
        // Too close to the world edge.
        let (ok, out) = run_ladder(&w, &[], 5, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["We need better text for this!!!"]);
        // A hole in the wall (the open field, wall == 0 arm).
        let (ok, out) = run_ladder(&w, &[], 30, 100, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This room is missing a wall"]);
        // Unsafe (dirt) walls inside the frame.
        let mut unsafe_w = ground();
        build_furnished_room(&mut unsafe_w, 60, 100, 12, 9);
        for x in 60..72 {
            for y in 100..109 {
                let i = unsafe_w.index(x, y);
                unsafe_w.tiles[i].wall = 3;
            }
        }
        let (ok, out) = run_ladder(&unsafe_w, &[], 65, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["Natural walls in this room need replacing!"]);
        // A too-small room: a 4x4 frame holds fewer than 60 tiles (no
        // furniture — the default pieces would overwrite the tiny frame).
        let mut tiny = ground();
        build_furnished_room_with(&mut tiny, 60, 100, 4, 4, &[]);
        let (ok, out) = run_ladder(&tiny, &[], 61, 101, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This room is too small"]);
        // A too-big room: a 110-wide frame crosses maxRoomSize = 100
        // (`CheckRoom:6158-6164`) before the 750-tile cap.
        let mut big = ground();
        for x in 40..150 {
            for y in 100..108 {
                let i = big.index(x, y);
                big.tiles[i].wall = 27;
                if x == 40 || x == 149 || y == 100 || y == 107 {
                    big.tiles[i] = tr_core::Tile { kind: 9, frame_x: -1, frame_y: -1, ..Default::default() };
                    big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
        }
        let (ok, out) = run_ladder(&big, &[], 95, 103, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This room is too big"]);
        // WithoutText: the same refusal, no line (`NoRoomCheckFeedback.cs:85`).
        let (ok, out) = run_ladder(&w, &[], 65, 120, None, GUIDE_ID, RoomCheckFeedback::WITHOUT_TEXT);
        assert!(!ok);
        assert!(out.messages.is_empty());
        // The rich pass neither stops nor prints (`Main.cs:45165`): the walk
        // runs to the end and returns true with no text.
        let (ok, out) = run_ladder(&w, &[], 65, 120, None, GUIDE_ID, RoomCheckFeedback::RICH);
        assert!(ok);
        assert!(out.messages.is_empty());
    }

    #[test]
    fn the_missing_furniture_text_lists_torch_door_table_chair() {
        // `:4729-4762`: Game.HouseMissing_<n> over the missing pieces in the
        // torch/door/table/chair order, plus the two latch flags.
        let _g = lang_english();
        let mut w = ground();
        build_furnished_room_with(&mut w, 60, 100, 12, 9, &[]);
        let (ok, out) = run_ladder(&w, &[], 65, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(
            out.messages,
            ["This house is missing a light source, a door, a table, and a chair."]
        );
        assert_eq!(out.built_house, BuiltHouseFlags { no_furniture: true, no_light: true });
        // Only the light missing: one piece, no_furniture stays clear.
        let mut no_light = ground();
        build_furnished_room_with(&mut no_light, 60, 100, 12, 9, &[15, 14, 10]);
        let (ok, out) = run_ladder(&no_light, &[], 65, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This house is missing a light source."]);
        assert_eq!(out.built_house, BuiltHouseFlags { no_furniture: false, no_light: true });
        // Only the chair missing: no_light stays clear.
        let mut no_chair = ground();
        build_furnished_room_with(&mut no_chair, 60, 100, 12, 9, &[14, 4, 10]);
        let (ok, out) = run_ladder(&no_chair, &[], 65, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This house is missing a chair."]);
        assert_eq!(out.built_house, BuiltHouseFlags { no_furniture: true, no_light: false });
    }

    #[test]
    fn the_stinkbug_refusals_exempt_the_town_pets() {
        // `:4777-4798`: roomHasStinkbug / roomHasEchoStinkbug refuse everyone
        // except NPCID.Sets.IsTownPet (`NPCID.cs:10118`). The port ships no
        // pet kind, so the query's prioritized type carries the pet id.
        let _g = lang_english();
        for (cage, text) in [
            (630u16, "This room is infested by a stinkbug!"),
            (631, "This room is infested by a ghostly stinkbug!"),
        ] {
            let mut w = ground();
            build_furnished_room(&mut w, 60, 100, 12, 9);
            let i = w.index(66, 107);
            w.tiles[i] = tr_core::Tile { kind: cage, frame_x: 0, frame_y: 0, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            // A common NPC is refused.
            let (ok, out) = run_ladder(&w, &[], 65, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
            assert!(!ok, "cage {cage}");
            assert_eq!(out.messages, [text]);
            // A town pet (TownCat 637) walks past both cages.
            let (ok, out) = run_ladder(&w, &[], 65, 105, None, TOWN_CAT_ID, RoomCheckFeedback::WITH_TEXT);
            assert!(ok, "the pet is exempt from cage {cage}");
            assert!(out.messages.is_empty());
        }
        assert!(is_town_pet(637) && is_town_pet(684));
        assert!(!is_town_pet(GUIDE_ID) && !is_town_pet(160));
    }

    #[test]
    fn the_score_ladder_reports_occupied_evil_and_no_standing_space() {
        // `:4805-4822`: hiScore <= 0 → occupied (inter[41]) → evil
        // (inter[42]) → no standing space → the generic inter[40].
        let _g = lang_english();
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        // Occupied: a housed Guide lives in the room; the asking Merchant
        // (same category) is refused. `ScoreRoom(-1, type)` ignores nobody.
        let resident = town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, Some((65, 106)));
        let merchant = town_npc(crate::npc::NpcKind::Merchant, 0.0, 0.0, None);
        let (ok, out) = run_ladder(&w, &[resident, merchant], 65, 105, Some(1), 0, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This housing is already occupied."]);
        // Evil: a corruption field across the tested area drops the base
        // below -250 (`:5890-5893`), roomEvil set (`:5886-5889`).
        let mut evil_w = ground();
        build_furnished_room(&mut evil_w, 60, 100, 12, 9);
        for x in 10..110 {
            for y in 110..120 {
                let i = evil_w.index(x, y);
                evil_w.tiles[i] = tr_core::Tile { kind: 25, ..Default::default() };
                evil_w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let (ok, out) = run_ladder(&evil_w, &[], 65, 105, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This housing is corrupted."]);
        // No standing space: a 3-row chamber (rows 101-103) between a door
        // ceiling (row 100) and a door floor (row 104). The floor candidates
        // pass the gate (the box above is the empty chamber) but the score
        // block's five ceiling doors cost -20 each (`:5927`), so every num
        // lands <= 0 and `roomHasStandingSpace` stays false (`:5932-5935`).
        // The chamber + its settled border is exactly 60 tiles.
        let mut doors = ground();
        for x in 60..72 {
            for y in 100..105 {
                let i = doors.index(x, y);
                doors.tiles[i].wall = 27;
                let kind = if y == 100 || y == 104 {
                    10 // the door ceiling and floor
                } else if x == 60 || x == 71 {
                    9 // the wood sides
                } else {
                    continue; // the empty chamber
                };
                doors.tiles[i] = tr_core::Tile { kind, frame_x: 0, frame_y: 0, wall: 27, ..Default::default() };
                doors.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        // The furniture floats in the chamber's bottom row (non-solid).
        for (n, kind) in [15u16, 14, 4].iter().enumerate() {
            let i = doors.index(61 + n as i32, 103);
            doors.tiles[i] = tr_core::Tile { kind: *kind, frame_x: 0, frame_y: 0, wall: 27, ..Default::default() };
            doors.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let (ok, out) = run_ladder(&doors, &[], 65, 101, None, GUIDE_ID, RoomCheckFeedback::WITH_TEXT);
        assert!(!ok);
        assert_eq!(out.messages, ["This room has no space to stand in!"]);
    }

    #[test]
    fn the_special_conditions_refusal_names_the_npc_type() {
        // `:4829-4837`: Lang.inter[55] + " " + the NPC's TypeName. Only the
        // Truffle (160) has a condition (`CheckSpecialTownNPCSpawningConditions`
        // `:4970-4998`); the port ships no Truffle kind, so the gate itself is
        // pinned in tr-core (`the_truffle_gate_matches_the_source`) and every
        // ported kind passes it — this asserts the pass-through and the text
        // pieces the refusal would compose.
        let _g = lang_english();
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let guide = town_npc(crate::npc::NpcKind::Guide, 0.0, 0.0, None);
        let (ok, out) = run_ladder(&w, &[guide], 65, 105, Some(0), 0, RoomCheckFeedback::WITH_TEXT);
        assert!(ok, "no ported kind has a special condition");
        assert!(out.messages.is_empty());
        assert_eq!(crate::lang::inter(55), "This housing does not meet the requirements for a");
        assert_eq!(crate::lang::npc_name(160), "Truffle");
    }
}
