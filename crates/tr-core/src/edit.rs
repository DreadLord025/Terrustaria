//! Tile edits shared by the player, wire tools and (later) NPCs.
//!
//! A local re-frame is enough after a single block change: only the edited tile and its
//! neighbours (up to two cells for the merge rules) can change frames or wall cells.

use crate::framing;
use crate::tile::Tile;
use crate::unified_random::UnifiedRandom;
use crate::world::World;

mod place;
pub use place::{place_alch, place_item_tile, place_tile, place_tile_forced, place_torch};
pub use place::{place_alch, place_chest, place_item_tile, place_tile, place_torch};
pub use place::{place_alch, place_chest, place_item_tile, place_object, place_tile, place_torch};
mod wall;
// S1-P-i33 — the wall-edit table (drop conversions, the FULL Main.wallHouse,
// the smash gate, the unsafe→safe replacement). `can_player_smash_wall` lives
// HERE (one copy — the i31 duplicate was folded into this one at the gate;
// the i33 copy reads the full 279-wall Main.wallHouse, the C# table the
// Player.cs:45924 gate actually reads, not the housing room-check subset).
pub use wall::{
    break_wall, can_player_smash_wall, kill_wall, kill_wall_drop, kill_wall_sound,
    near_friendly_wall, place_wall, replace_wall, wall_house, WallBreak, WALL_COUNT,
};
mod hammer;
pub use hammer::{
    can_pound_tile, forbids_sloping, is_a_hammer_target, is_tile_poundable, pound_damage,
    pound_tile, slope_tile, smooth_slope, smooth_slope_one, try_finding_wall_to_hammer,
    try_pound_tile, PoundOutcome,
};
mod paint;
pub use paint::{paint_coat_tile, paint_coat_wall, paint_tile, paint_wall};
mod replace;
pub use replace::{
    is_tile_replaceable, move_replace_anchor, replace_tile_write, would_tile_replacement_work,
    Replaced,
};
mod kill;
pub use kill::{
    actuation_square_frame, can_kill_tile, kill_tile, kill_tile_sound, pot_break_sound,
    square_tile_frame_default, BulbSummonEvent, HerbEvent, KillSounds, ObjectDrop,
    PileLootEvent, PotEvent, SoundCue, TileKill, TreeDropEvent,
    actuation_square_frame, can_kill_tile, check_alch, check_lily_pad, kill_tile, kill_tile_sound,
    pot_break_sound, square_tile_frame_default, square_tile_frame_record, HerbEvent, KillSounds,
    ObjectDrop, PileLootEvent, PotEvent, SoundCue, TileKill, TreeDropEvent,
};
mod shift;
// W4-c42 batch 2 — the wired trapdoor/tall-gate shifts (`WorldGen.cs:51447-
// 51596`/`:51723-51775`), the `HitWireSingle` arms `Wiring.cs:1440-1460` call.
pub use shift::{shift_tall_gate, shift_trapdoor};
pub(crate) use kill::{
    remove_tile, solid_tile, solid_tile2, solid_tile_allow_bottom_slope,
    solid_tile_allow_left_slope, solid_tile_allow_right_slope, solid_tile_allow_top_slope,
    square_tile_frame_gen,
mod replace;
// W4-g48 — the net-17 kind-21 `WorldGen.ReplaceTile` family
// (`WorldGen.cs:62558-62775`), the smart-cursor block swap.
pub use replace::{
    is_tile_replaceable, replace_tile, replace_tile_commit, replace_tile_prepare,
    would_tile_replacement_work, ReplaceOutcome,
};
pub use kill::solid_tile;
pub(crate) use kill::{
    remove_tile, solid_tile2, solid_tile_allow_bottom_slope, square_tile_frame_gen,
};

/// `WallID.UnbreakableBlockWall` / `WallID.Count - 1` region used by the dungeon
/// control lines. Vanilla refuses it in every removal/replacement path:
/// `Player.CanPlayerSmashWall` (hammering), `WorldGen.ReplaceWall`, and it makes any
/// tile in front of it unbreakable (`WorldGen.CanKillTile`, `IsTileReplaceable`).
pub const WALL_UNBREAKABLE: u16 = 350;

/// The alch soil table — the per-variant ground families shared by
/// `WorldGen.PlaceAlch` (`WorldGen.cs:45393-45441`) and `CheckAlch`
/// (:45998-46086); every variant also accepts the clay pot 78 and the
/// planter box 380.
const ALCH_SOILS: [&[u16]; 7] = [
    &[2, 477, 492, 78, 380, 109],
    &[60, 78, 380],
    &[0, 59, 78, 380],
    &[661, 662, 199, 203, 23, 25, 78, 380],
    &[53, 78, 380, 116],
    &[57, 633, 78, 380],
    &[78, 380, 147, 161, 163, 164, 200],
];

/// `WorldGen.PlaceTile`'s tail sound switch (`WorldGen.cs:60263-60282`,
/// S1-E-21 wave B): 127 → `SoundID.Item30` (`SoundID.cs:1413`), 314 →
/// `SoundID.Item52` (`:1435`), 330-333 → the legacy id 18 (Coins), the
/// default → the legacy id 0 (Dig). The `TileObject`-routed placements play
/// the plain 0 instead (`Player.cs:40124`, `WorldGen.PlaceObject` `:51441`);
/// none of the special kinds above is a `CustomPlace` kind (the registry's
/// `BY_KIND` has no def for them), so this switch covers both routes.
/// `WorldGen.PlaceWall` plays the 0 as well (`:75141`).
pub fn place_tile_sound(kind: u16) -> SoundCue {
    match kind {
        127 => (2, 30),
        314 => (2, 52),
        330..=333 => (18, 1),
        _ => (0, 1),
    }
}

/// `WorldGen.PlaceWire` / `PlaceWire2` / `PlaceWire3` / `PlaceWire4`
/// (`WorldGen.cs:58111-58218`): the four wire colours' placement. The C# has
/// one one-line function per `Tile.wire*` bit (`Tile.cs:623-657`); the port
/// keeps the wires in the side table ([`crate::wiring::WireState`]), so the
/// colour is a parameter and the four functions are one.
///
/// The C#'s sound (`SoundEngine.PlaySound(0, i * 16, j * 16)`) and its net
/// message are app-side.
pub fn place_wire(
    world: &mut World,
    x: i32,
    y: i32,
    color: crate::wiring::WireColor,
) -> bool {
    if !world.in_bounds(x, y) {
        return false;
    }
    if world.wires.get(x, y, color) {
        return false;
    }
    world.wires.set(x, y, color, true);
    true
}

/// `WorldGen.KillWire` / `KillWire2` / `KillWire3` / `KillWire4`
/// (`WorldGen.cs:58122-58230`): the wire comes off the tile. The C#'s item
/// drop (`Item.NewItem(..., 530)`) and its dust are app-side — the core has
/// no item pool.
pub fn kill_wire(
    world: &mut World,
    x: i32,
    y: i32,
    color: crate::wiring::WireColor,
) -> bool {
    if !world.in_bounds(x, y) {
        return false;
    }
    if !world.wires.get(x, y, color) {
        return false;
    }
    world.wires.set(x, y, color, false);
    true
}

/// `WorldGen.PlaceActuator` (`WorldGen.cs:58011-58020`): the actuator bit on
/// the tile (`Tile.actuator`, `Tile.cs:691-694`).
pub fn place_actuator(world: &mut World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.flags & crate::tile::FLAG_ACTUATOR != 0 {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].flags |= crate::tile::FLAG_ACTUATOR;
    true
}

/// `WorldGen.KillActuator` (`WorldGen.cs:58022-58040`): the actuator bit
/// comes off. The C#'s item drop (849) and its dust are app-side.
pub fn kill_actuator(world: &mut World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.flags & crate::tile::FLAG_ACTUATOR == 0 {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].flags &= !crate::tile::FLAG_ACTUATOR;
    true
}

/// `WorldGen.paintTile` (`WorldGen.cs:44110-44132`): the active tile's paint
/// colour becomes `color`. Returns false on a null/inactive cell or when the
/// colour already matches (`b == color`, `:44118-44121`). The C#'s
/// `paintEffect` dust (`:44122-44125`) and the `NetMessage.SendData(63)`
/// broadcast (`:44127-44130`) are app-side — the core writes the colour only.
pub fn paint_tile(world: &mut World, x: i32, y: i32, color: u8) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    if t.paint() == color {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].set_paint(color);
    true
}

/// `WorldGen.paintWall` (`WorldGen.cs:44230-44252`): the wall's paint colour
/// becomes `color`. Returns false on a null/wall-less cell or when the colour
/// already matches (`b == color`, `:44238-44241`). The C#'s `paintEffect` dust
/// (`:44247-44250`) and the `NetMessage.SendData(64)` broadcast (`:44243-44246`)
/// are app-side.
pub fn paint_wall(world: &mut World, x: i32, y: i32, color: u8) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.wall == 0 {
        return false;
    }
    if world.wall_paint(x, y) == color {
        return false;
    }
    world.set_wall_paint(x, y, color);
    true
}

/// `WorldGen.paintCoatTile` (`WorldGen.cs:44134-44180`): the active tile's
/// Glow/Echo coating bits. `paint_coat_id` 0 clears both (only when at least
/// one is set), 1 sets Glow (`fullbrightBlock`), 2 sets Echo (`invisibleBlock`);
/// any other id returns false (`default:` arm). The id-0/id-2 arms re-frame the
/// 3×3 neighbourhood (`SquareTileFrame(x, y, resetFrame: false)`, `:44153`/
/// `:44168`); the id-1 arm does not. The C#'s `paintCoatEffect` dust
/// (`:44175-44178`) and the `NetMessage.SendData(63, …, 1f)` broadcast
/// (`:44171-44174`) are app-side.
pub fn paint_coat_tile(world: &mut World, x: i32, y: i32, paint_coat_id: u8) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    let glow = t.fullbright_block();
    let echo = world.invisible_block(x, y);
    match paint_coat_id {
        0 => {
            if !glow && !echo {
                return false;
            }
            let i = world.index(x, y);
            world.tiles[i].flags &= !crate::tile::FLAG_FULLBRIGHT;
            world.set_invisible_block(x, y, false);
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        1 => {
            if glow {
                return false;
            }
            let i = world.index(x, y);
            world.tiles[i].flags |= crate::tile::FLAG_FULLBRIGHT;
        }
        2 => {
            if echo {
                return false;
            }
            world.set_invisible_block(x, y, true);
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        _ => return false,
    }
    true
}

/// `WorldGen.paintCoatWall` (`WorldGen.cs:44182-44228`): the wall's Glow/Echo
/// coating bits. `paint_coat_id` 0 clears both (only when at least one is set),
/// 1 sets Glow (`fullbrightWall`), 2 sets Echo (`invisibleWall`); any other id
/// returns false. The id-0/id-2 arms re-frame the 3×3 neighbourhood
/// (`SquareWallFrame(x, y, resetFrame: false)`, `:44201`/`:44216`); the id-1 arm
/// does not. The C#'s `paintCoatEffect` dust (`:44223-44226`) and the
/// `NetMessage.SendData(64, …, 1f)` broadcast (`:44219-44222`) are app-side.
pub fn paint_coat_wall(world: &mut World, x: i32, y: i32, paint_coat_id: u8) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.wall == 0 {
        return false;
    }
    let glow = world.fullbright_wall(x, y);
    let echo = world.invisible_wall(x, y);
    match paint_coat_id {
        0 => {
            if !glow && !echo {
                return false;
            }
            world.set_fullbright_wall(x, y, false);
            world.set_invisible_wall(x, y, false);
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        1 => {
            if glow {
                return false;
            }
            world.set_fullbright_wall(x, y, true);
        }
        2 => {
            if echo {
                return false;
            }
            world.set_invisible_wall(x, y, true);
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        _ => return false,
    }
    true
}

/// The result of the right-click interaction dispatcher: what the click did,
/// and every object destruction its cleanup kills performed (the port of
/// vanilla's `Item.NewItem` side effects — `WorldGen.cs:86508` and peers).
/// `None` = the tile has no interaction; the caller falls through to the
/// place path.
#[derive(Debug)]
pub enum TileInteraction {
    None,
    /// `Player.cs:33279-33284` — the unconditional tail of
    /// `TileInteractionsUse`: every tile that reaches it hands the click to
    /// `GetCraftingFilterForTile` (`:33312-33335`); the app layer decides
    /// whether the kind is a crafting station (`TileUsedInRecipes`, the
    /// `TileCountsAs` inheritance included) or a water source, and only a
    /// resolved filter consumes the click (the C#'s `flag2 = true` under
    /// the non-null check).
    StationInteract { kind: u16 },
    /// `WorldGen.OpenDoor` succeeded (`SoundEngine.PlaySound(8)`, `:37732`).
    DoorOpened {
        object_drops: Vec<ObjectDrop>,
    },
    /// `WorldGen.CloseDoor` succeeded (`SoundEngine.PlaySound(9)`, `:31906`).
    DoorClosed {
        object_drops: Vec<ObjectDrop>,
    },
    /// The door branch ran but the toggle was refused (blocked, partial
    /// door). Vanilla still consumes the click (`Player.cs:32836` sets
    /// `flag2` before attempting) and the validate-cleanup kills may have
    /// dropped objects; no sound plays on this path (`:37732`/`:31906` sit
    /// behind every failure return).
    DoorBlocked {
        object_drops: Vec<ObjectDrop>,
    },
    /// The bank open ran (`Player.cs:33122-33138`'s num73 switch): `bank`
    /// is the chest index -2..-5. No sound on the open path in the port's
    /// cue set (`OpenChest`'s SoundID 10/11/12 keys are absent).
    BankOpen {
        bank: i32,
    },
    /// The dial branch fired: `Main.Sundialing()`/`Moondialing()` armed the
    /// fast-forward (`Main.cs:6178-6180`/:6192-6194, SP branch). `sundial`
    /// picks which dial — the cue is the same `SoundID.Item4` for both
    /// (`Player.cs:32642`/:32652).
    DialStarted {
        sundial: bool,
    },
    /// The candle lit-toggle ran (`Player.cs:32378-32390`): `frameX +=
    /// (frameX > 0) ? -18 : 18`. No sound; the flame overlay and the light
    /// emission follow the frame.
    CandleToggled,
    /// `Player.cs:32174-32178`: the used tile is a lever (132), a switch
    /// (136) or a timer (144) - `Wiring.HitSwitch` fires and the click is
    /// consumed. The wiring itself is app-side (the core has no wiring
    /// state), so the interaction carries the cell.
    HitSwitch { x: i32, y: i32 },
    /// `Player.cs:32271-32277` (W4-c42 final, the g78 research/76 contested
    /// seam request): the used tile is one of the nine monolith kinds
    /// (410/480/509/657/658/720/721/725/733) - `PlaySound(28)` +
    /// `GamepadEnableGrappleCooldown` + `WorldGen.SwitchMonolith(myX, myY)`.
    /// The flip is app-side (`Wiring::use_switch_monolith` over the wired
    /// path's kept body - the e21 nine-kind gate); the interaction carries
    /// the cell like `HitSwitch`.
    MonolithToggle { x: i32, y: i32 },
    /// The chair sit decoded (`PlayerSittingHelper.GetSittingTargetInfo`'s
    /// case 15, `PlayerSittingHelper.cs:163-182`): the seat world position is
    /// the chair's bottom-centre shifted `targetDirection * 6` px, the
    /// facing flips with `frameX`. `seat_offset` is the per-style
    /// `seatDownOffset` (`:158` out param) the renderer's seat drop reads.
    ChairSit {
        seat_x: f32,
        seat_y: f32,
        direction: i32,
        seat_offset: (f32, f32),
    },
    /// The bed branch (`Player.cs:32654-32692`). `set_spawn` picks the half:
    /// the bottom side (`IsHoveringOverABottomSideOfABed`) runs
    /// `FindSpawn`/`CheckSpawn`/`ChangeSpawn`; the top half is
    /// `sleeping.StartSleeping` (`:32669-32677`), decoded by
    /// [`sleeping_target_info`] — the anchor the player's bottom snaps to,
    /// the facing the bed imposes and the `frameY / 36` style the visual
    /// offset table (`SetOffsetbyBed`, `PlayerSleepingHelper.cs:226-279`)
    /// keys on.
    BedInteract {
        set_spawn: bool,
        spawn_x: i32,
        spawn_y: i32,
    },
    /// The sleep half of the bed branch (`Player.cs:32669-32677` →
    /// `PlayerSleepingHelper.StartSleeping`, `PlayerSleepingHelper.cs:155-190`).
    /// The 96-px snap gate (`IsWithinSnappngRangeToTile(myX, myY, 96)`) is the
    /// caller's, like the sit gate.
    BedSleep {
        anchor_x: f32,
        anchor_y: f32,
        direction: i32,
        bed_style: u32,
    },
    /// The grandfather clock's time read (`Player.cs:32725-32759`).
    ClockRead {
        text: String,
    },
    /// The dial branch ran but the gate refused (already fast-forwarding to
    /// that target or the cooldown is running, `Player.cs:32638`/:32648).
    /// Vanilla still consumes the click (`flag2 = true`, `:32637`); no state
    /// changes and no sound plays. No object drops — the dial arms kill no
    /// tiles.
    DialRefused,
    /// W4-e10 — the buff-station family (`Player.cs:32594-32634`): the seven
    /// right-click buff tiles (Crystal Ball 125, Slice of Cake 621, War Table
    /// 464, Dead Cells Potion Station 699, Sharpening Station 377, Bewitching
    /// Table 354, Ammo Box 287). Each arm sets `flag2 = true`, grants its buff
    /// for the fixed duration, then plays its `SoundID.ItemNN` style at
    /// `base.position` (the player's TOP-LEFT). The buff grant + cue are
    /// app-side (the core has no player); this variant carries the tile kind.
    /// The arms kill no tiles, so there are never drops. The C# chain tests
    /// these kind literals BEFORE the dial branches (`:32636`) and the
    /// crafting-station tail (`:33279`) — 699 also counts as a Bone Welder
    /// (`TileCountsAs` 699→13), but the buff arm wins the click first.
    BuffStation { kind: u16 },
    /// The Extractinator branch (tiles 219/642) ran: the clicked tile is an
    /// extractinator and the held item decides the rest (`Player.cs:41665-41697`
    /// — the C# arm is item-anchored inside `PlaceThing`, so it rides the
    /// held-item swing cadence rather than the click edge; the app-side arm in
    /// `world_sim/extract.rs` owns the held-item gate, the roll, the drop and
    /// the consumption). No world write here — the extraction only consumes the
    /// held item and spawns the output.
    Extractinator,
    /// The right-click toggle family (`Player.cs:32228-32284`): the music
    /// box 139 (`SwitchMB`), the `TileID.Sets.Campfires` set (the 3×2
    /// frameY ±36 flip), the fountain 207 (`SwitchFountain`), the nine
    /// monolith kinds (`SwitchMonolith`) and the party centre 455
    /// (`BirthdayParty.ToggleManualParty`). ALL five arms play
    /// `SoundEngine.PlaySound(28, myX * 16, myY * 16, 0)` BEFORE the
    /// toggle (`:32231`/`:32237`/`:32268`/`:32274`/`:32281` — W4-e10);
    /// the wiring halves are app-side (the core has no wiring state), so
    /// the interaction carries the clicked cell + the kind. The campfire
    /// flip is pure tile state — the core runs it here.
    RightClickToggle { x: i32, y: i32, kind: u16 },
    /// The station-buff arms (W4-e28 + W4-f34): the Crystal Ball (tile 125,
    /// `Player.cs:32594-32599`) grants Clairvoyance (29), the Slice of Cake
    /// (621, `:32600-32605`) Sugar Rush (192, 7200 ticks), the War Table
    /// (464, `:32606-32611`) War Table (348), the Dead Cells Potion Station
    /// (699, `:32612-32617`) its buff 366, the Sharpening Station (377,
    /// `:32618-32623`) Sharpened (159), the Bewitching Table (354,
    /// `:32624-32629`) Bewitched (150) and the Ammo Box (287,
    /// `:32630-32635`) Ammo Box (93) — all but the cake `AddBuff(id,
    /// 108000)` (30 min) then `SoundEngine.PlaySound(cue, base.position)`
    /// — the buff slots and the cue are app-side (the core has neither);
    /// the sound rides the PLAYER's position, not the tile's.
    StationBuff {
        buff_type: i32,
        buff_time: i32,
    },
}

/// The world-interaction dispatcher — the port of the right-click tile hook:
/// `Player.LookForTileInteractions` (`Player.cs:31937`) →
/// `TileInteractionsCheck` (`:32031`, active tiles only) →
/// `TileInteractionsUse` (`:32118`), whose type chain routes tile 10/11
/// through the door branch (`:32835-32865`) and tiles 356/663 through the
/// dial branches (`:32636-32653`). The port dispatches the doors on the
/// registry def's `interaction` tag instead of a type chain (Decision #3) so
/// chests/signs can join as data later; the dials keep the C# kind-literal
/// shape (see the arm below). `direction` is `Player.direction`
/// (`:32841`), `cell_blocked` the entity half of `Collision.EmptyTile`
/// (`Collision.cs:1359`, called with `ignoreTiles: true` — the door ops port
/// the tile half themselves and delegate entities to the caller).
/// `rng` is the ACTIVE `genRand` stream of the caller's context (W4-g83
/// contract §2 — the tile use runs under the `UpdatePlayers` swap,
/// `Main.cs:17754`): the door ops' validate-cleanup `KillTile`s run their
/// framing dispatch on it.
pub fn interact_at(
    world: &mut World,
    i: i32,
    j: i32,
    direction: i32,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
    rng: &mut UnifiedRandom,
) -> TileInteraction {
    // `TileInteractionsCheck` (`Player.cs:32031-32041`): active tiles only.
    let Some(t) = world.tile(i, j) else {
        return TileInteraction::None;
    };
    if !t.active() {
        return TileInteraction::None;
    }
    // The switch family (`Player.cs:32171-32179`): using a lever (132), a
    // switch (136) or a timer (144) trips the wiring. The C#'s chain tests
    // the kind literal before the dials and the furniture.
    if matches!(t.kind, 132 | 136 | 144) {
        return TileInteraction::HitSwitch { x: i, y: j };
    }
    // W4-e10 — the buff-station family (`Player.cs:32594-32634`): the seven
    // right-click buff tiles, tested as kind literals BEFORE the dial branches
    // (`:32636`) and the crafting-station tail (`:33279`), exactly the C# chain
    // order. The buff grant + the `SoundID.ItemNN` style play are app-side (the
    // core has no player); the variant carries the kind. 699 also counts as a
    // Bone Welder (`TileCountsAs` 699→13), but the buff arm wins the click first.
    if matches!(t.kind, 125 | 621 | 464 | 699 | 377 | 354 | 287) {
        return TileInteraction::BuffStation { kind: t.kind };
    // The station-buff arms (W4-e28 + W4-f34, `Player.cs:32594-32635`): the
    // C# chain tests the kind literals in this order before the dials —
    // 125 Crystal Ball → 29 (:32594-32599), 621 Slice of Cake → 192 for
    // 7200 (:32600-32605), 464 War Table → 348 (:32606-32611), 699 Dead
    // Cells Potion Station → 366 (:32612-32617), 377 Sharpening Station →
    // 159 (:32618-32623), 354 Bewitching Table → 150 (:32624-32629), 287
    // Ammo Box → 93 (:32630-32635). All but the cake grant 108000 ticks;
    // every arm consumes the click (`flag2 = true`). The buff + cue land
    // app-side. The 699 capture supersedes the old station-tail craft-filter
    // click: the C# arm grants the buff and never reaches `:33279` (the
    // adjacency alchemy-table flag, `craft.rs`'s `AdjTiles`, is separate).
    let station = match t.kind {
        125 => Some((29, 108_000)),
        621 => Some((192, 7_200)),
        464 => Some((348, 108_000)),
        699 => Some((366, 108_000)),
        377 => Some((159, 108_000)),
        354 => Some((150, 108_000)),
        287 => Some((93, 108_000)),
        _ => None,
    };
    if let Some((buff_type, buff_time)) = station {
        return TileInteraction::StationBuff {
            buff_type,
            buff_time,
        };
    // The monolith family (`Player.cs:32271-32277`, W4-c42 final — the g78
    // research/76 request): the nine kinds flip via `SwitchMonolith` on
    // right-click. The C# chain sits after the fountain (207) arm and
    // before the party box (455); neither has a port arm, so the literal
    // test lands here beside the switch family (the same pre-registry
    // shape — 657's def carries `Interaction::None`).
    if matches!(t.kind, 410 | 480 | 509 | 657 | 658 | 720 | 721 | 725 | 733) {
        return TileInteraction::MonolithToggle { x: i, y: j };
    }
    // The dial branches (`Player.cs:32636-32653`): the C# chain tests
    // `Main.tile[myX, myY].type` literals here. The port's registry has no
    // defs for 356/663 (they place through the legacy single-tile route), so
    // this arm reads the kind the same way, before the registry lookup.
    if t.kind == 356 || t.kind == 663 {
        return dial_interaction(world, t.kind == 356);
    }
    // The right-click toggle family (`Player.cs:32228-32284`), in the C#
    // chain order: the music box 139, the Campfires set, the fountain 207,
    // the monolith kinds, the party centre 455. All play the Mech-28 click
    // at the CLICKED cell before the toggle (the app half); the wiring
    // halves are app-side (`RightClickToggle` carries the cell), the
    // campfire's 3×2 frameY flip is pure tile state and runs here.
    if t.kind == 139
        || crate::tiledata::SET_CAMPFIRES.has(t.kind)
        || t.kind == 207
        || matches!(t.kind, 410 | 480 | 509 | 657 | 658 | 720 | 721 | 725 | 733)
        || t.kind == 455
    {
        if crate::tiledata::SET_CAMPFIRES.has(t.kind) {
            // `Player.cs:32238-32262`: the 3×2 anchor from the frame mods,
            // the all-active-and-same-kind validation (the inline shape —
            // the right-click arm does NOT kill mismatched cells, unlike
            // `ValidateTileSquareIsActiveAndOfType`'s door use), then the
            // ±36 frameY flip on every matching cell.
            let num19 = i32::from(t.frame_x) % (3 * 18) / 18;
            let num20 = i32::from(t.frame_y) % (2 * 18) / 18;
            let (num21, num22) = (i - num19, j - num20);
            let mut valid = true;
            for x in num21..num21 + 3 {
                for y in num22..num22 + 2 {
                    if !world
                        .tile(x, y)
                        .is_some_and(|c| c.active() && c.kind == t.kind)
                    {
                        valid = false;
                    }
                }
            }
            if valid {
                let step: i32 = if world.tile_frame_y(num21, num22) >= 36 {
                    -36
                } else {
                    36
                };
                for x in num21..num21 + 3 {
                    for y in num22..num22 + 2 {
                        if world
                            .tile(x, y)
                            .is_some_and(|c| c.active() && c.kind == t.kind)
                        {
                            let f = world.tile_frame_y(x, y);
                            world.set_tile_frame_y(x, y, f + step);
                        }
                    }
                }
            }
        }
        return TileInteraction::RightClickToggle { x: i, y: j, kind: t.kind };
    }
    // The bank tiles (tiles 29/97/463/491, `Player.cs:33056-33138`): the
    // num73 switch maps 29 -> -2, 97 -> -3, 463 -> -4, 491 -> -5. The
    // vanilla opens via `OpenChest(num74, num75, num76)`; the port's banks
    // are player-bounded (`Player.bank[1..4]`, `Player.cs:1580-1591`), so
    // only the chest index moves. The vault's `disableVoidBag` gate
    // (:33134-33139) is enforced downstream by `chest::toggle_open`
    // (W4-g56 — the MP-only net-slot lock, `Player.disable_void_bag`).
    if matches!(t.kind, 29 | 97 | 463 | 491) {
        let bank = match t.kind {
            29 => -2,
            97 => -3,
            463 => -4,
            _ => -5,
        };
        return TileInteraction::BankOpen { bank };
    }
    // The sit cases beyond the registry chair (`Player.cs:32313-32330`): the
    // C# chain tests the kind literals — chairs/toilets (15/497) and
    // benches/thrones/picnic tables (89/102/487) — and routes both to
    // `sitting.SitDown`. Tile 15 keeps riding the registry def above
    // (identical decode); these four have no defs, so they read the kind the
    // same way the dials/banks do, before the registry lookup.
    if matches!(t.kind, 497 | 89 | 102 | 487) {
        return chair_sit_target(world, i, j, direction);
    }
    // W4-c39 exec (research/114 §1h): the Slice of Cake (621) dispatch arm
    // DROPPED — f34's `StationBuff { buff_type, buff_time }` route covers
    // 621 in the seven-row chain order (`621 => Some((192, 7_200))`, the
    // anchor-bearing superset per research/70 §1e). Two dispatch arms for
    // 621 cannot coexist; f41c `f1f95be4` is canonical.
    // The port routes on the registry def's `interaction` tag (Decision #3)
    // where the C# chain tests `Main.tile[myX, myY].type` literals
    // (`Player.cs:32835`/:32857 for 10/11).
    let Some(def) = crate::tileobject::def_for_tile(t.kind) else {
        // No registry def: the C# chain falls through to the crafting
        // station tail (`Player.cs:33279`), which the legacy single-tile
        // kinds (the dials aside — they returned above) reach as well.
        return TileInteraction::StationInteract { kind: t.kind };
    };
    match def.interaction {
        crate::tileobject::Interaction::Bank(bank) => {
            return TileInteraction::BankOpen { bank };
        }
        crate::tileobject::Interaction::Candle => {
            // `Player.cs:32378-32390`: `num27 = (frameX > 0) ? -18 : 18;
            // tile.frameX += num27` — the lit/unlit toggle, no sound.
            if let Some(mt) = world.tile_mut(i, j) {
                if mt.frame_x > 0 {
                    mt.frame_x -= 18;
                } else {
                    mt.frame_x += 18;
                }
            }
            TileInteraction::CandleToggled
        }
        crate::tileobject::Interaction::Chair => {
            // `Player.cs:32313-32321` + `PlayerSittingHelper.SitDown`'s
            // target decode (`PlayerSittingHelper.cs:144-321`). The range
            // gate (`IsWithinSnappngRangeToTile(myX, myY, 40)`) is the
            // caller's — the click distance is already smart-gated.
            chair_sit_target(world, i, j, direction)
        }
        crate::tileobject::Interaction::Bed => {
            // `Player.cs:32654-32692`: the frame decode finds the bed's
            // spawn cell; the bottom half sets/removes the spawn, the top
            // half starts sleeping (`sleeping.StartSleeping`,
            // `PlayerSleepingHelper.cs:155-190` — the decode below, the snap
            // state lands in `Player::sleep_down`). The click is consumed on
            // both halves (`flag2 = true` at `:32655`/:32672).
            let frame_x = t.frame_x as i32;
            let mut num36 = i + frame_x / 18 * -1;
            if frame_x < 72 {
                num36 += 2;
            } else {
                num36 += 4 + 1;
            }
            let mut num38 = t.frame_y as i32 / 18;
            while num38 > 1 {
                num38 -= 2;
            }
            let num37 = j - num38 + 2;
            let bottom_side = {
                let flip = frame_x / 72 == 1;
                let mut flag2 = frame_x % 72 < 36;
                if flip {
                    flag2 = !flag2;
                }
                flag2
            };
            if bottom_side {
                TileInteraction::BedInteract {
                    set_spawn: true,
                    spawn_x: num36,
                    spawn_y: num37,
                }
            } else {
                // `Player.cs:32669-32677`: `if (!IsHoveringOverABottomSide...)
                // { if (IsWithinSnappngRangeToTile(myX, myY, 96)) StartSleeping; }`
                // — the decode mirrors `StartSleeping`'s
                // `GetSleepingTargetInfo` call; a failed decode leaves the
                // click consumed but inert (the old arm's shape).
                match sleeping_target_info(world, i, j) {
                    Some((anchor_x, anchor_y, sleep_direction, bed_style)) => {
                        TileInteraction::BedSleep {
                            anchor_x,
                            anchor_y,
                            direction: sleep_direction,
                            bed_style,
                        }
                    }
                    None => TileInteraction::BedInteract {
                        set_spawn: false,
                        spawn_x: num36,
                        spawn_y: num37,
                    },
                }
            }
        }
        crate::tileobject::Interaction::Clock => {
            // `Player.cs:32725-32759`: the in-game time, `h:mm AM/PM`.
            let mut num44 = world.time;
            if !world.day_time {
                num44 += 54000.0;
            }
            num44 = num44 / 86400.0 * 24.0 - 7.5 - 12.0;
            if num44 < 0.0 {
                num44 += 24.0;
            }
            let mut am_pm = "AM";
            if num44 >= 12.0 {
                am_pm = "PM";
            }
            let mut num45 = num44 as i32;
            let num46 = ((num44 - num45 as f64) * 60.0) as i32;
            if num45 > 12 {
                num45 -= 12;
            }
            if num45 == 0 {
                num45 = 12;
            }
            TileInteraction::ClockRead {
                text: format!("{num45}:{num46:02} {am_pm}"),
            }
        }
        crate::tileobject::Interaction::Extractinator => {
            // The extractinator tiles (219/642) — the C# arm is not part of
            // `TileInteractionsUse`'s chain at all: it lives in the held-item
            // `PlaceThing` path (`Player.cs:39591`, `PlaceThing_ItemInExtractinator`
            // `:41665-41697`), which fires on the swing's action ticks. The
            // dispatch here only tells the caller the tile IS an extractinator
            // (the C# chain tests the kind literals, `:41669`/`:41691`), so the
            // app-layer arm can gate on the held item before consuming the tick.
            // No world change: vanilla only consumes the held item and spawns
            // the output at the cursor (`ExtractinatorUse` `:42072-42079`,
            // `DropItemFromExtractinator` `:42081-42089`).
            TileInteraction::Extractinator
        }
        crate::tileobject::Interaction::None => {
            TileInteraction::StationInteract { kind: t.kind }
        }
        crate::tileobject::Interaction::Door => {
            let mut drops = Vec::new();
            if t.kind == 10 {
                // The closed-door branch (`Player.cs:32835-32856`): open with
                // the player's facing; while the tile is still a closed door,
                // retry with the opposite direction. After a validate-kill
                // the cell no longer has kind 10 (`KillTile` resets `type`,
                // `WorldGen.cs:63915`), so the cleanup path skips the retry.
                if open_door(world, i, j, direction, &mut drops, None) {
                if open_door(world, i, j, direction, &mut drops, rng) {
                    return TileInteraction::DoorOpened {
                        object_drops: drops,
                    };
                }
                if world.tile(i, j).is_some_and(|t| t.kind == 10)
                    && open_door(world, i, j, -direction, &mut drops, None)
                    && open_door(world, i, j, -direction, &mut drops, rng)
                {
                    return TileInteraction::DoorOpened {
                        object_drops: drops,
                    };
                }
                TileInteraction::DoorBlocked {
                    object_drops: drops,
                }
            } else if close_door(world, i, j, false, cell_blocked, &mut drops, None) {
            } else if close_door(world, i, j, false, cell_blocked, &mut drops, rng) {
                // The open-door branch (`Player.cs:32857-32864`).
                TileInteraction::DoorClosed {
                    object_drops: drops,
                }
            } else {
                TileInteraction::DoorBlocked {
                    object_drops: drops,
                }
            }
        }
    }
}

/// `PlayerSittingHelper.GetSittingTargetInfo` (`PlayerSittingHelper.cs:144-321`)
/// — the full sit decode: returns `(seat_x, seat_y, target_direction,
/// seat_down_offset)` (the C# `playerSittingPosition`/`targetDirection`/
/// `seatDownOffset` outs) or `None` when the tile is not sittable. The
/// toilet flag (`extraInfo.IsAToilet`) and the Skeletron-hat easter egg
/// (`UpdateSitting`'s `:74-83` bench arm) are not ported — the toilet's
/// regen hook and the NPC visual both ride systems this task does not touch.
pub fn sitting_target_info(
    world: &World,
    x: i32,
    y: i32,
    player_direction: i32,
) -> Option<(f32, f32, i32, (f32, f32))> {
    let t = world.tile(x, y)?;
    if !t.active() || !matches!(t.kind, 15 | 497 | 89 | 102 | 487) {
        return None;
    }
    let mut num = x;
    let mut num2 = y;
    let mut target_direction = 1i32;
    let mut seat_down_offset = (0.0f32, 0.0f32);
    let mut num3 = 6i32;
    let mut zero = (0.0f32, 0.0f32);
    match t.kind {
        // case 15 / 497 (`:163-181`).
        15 | 497 => {
            if t.kind == 15 && t.frame_y / 40 == 27 {
                seat_down_offset.1 = 4.0;
            }
            if t.frame_y % 40 != 0 {
                num2 -= 1;
            }
            target_direction = if t.frame_x != 0 { 1 } else { -1 };
        }
        // case 102 (`:183-209`).
        102 => {
            let num4 = t.frame_x / 18;
            if num4 == 0 {
                num += 1;
            }
            if num4 == 2 {
                num -= 1;
            }
            match t.frame_y / 18 {
                0 => num2 += 2,
                1 => num2 += 1,
                3 => num2 -= 1,
                _ => {}
            }
            target_direction = player_direction;
            num3 = 0;
        }
        // case 487 (`:211-229`).
        487 => {
            let num7 = t.frame_x % 72 / 18;
            if num7 == 1 {
                num -= 1;
            }
            if num7 == 2 {
                num += 1;
            }
            if t.frame_y / 18 != 0 {
                num2 -= 1;
            }
            // `(num7 <= 1).ToDirectionInt()` — false → -1, true → +1.
            target_direction = if num7 <= 1 { -1 } else { 1 };
            num3 = 0;
            seat_down_offset.1 -= 1.0;
        }
        // case 89 (`:231-314`): the per-style seat-offset table over
        // `frameX / 54`, the side pick over `frameX % 54` and the facing.
        89 => {
            target_direction = player_direction;
            num3 = 0;
            let mut vector = (-4.0f32, 2.0f32);
            let mut vector2 = (4.0f32, 2.0f32);
            let mut vector3 = (0.0f32, 2.0f32);
            zero.0 = -1.0;
            match t.frame_x / 54 {
                0 => {
                    vector3.1 = 1.0;
                    vector.1 = 1.0;
                    vector2.1 = 1.0;
                }
                1 => {
                    vector3.1 = 1.0;
                }
                2 | 14 | 15 | 17 | 20..=23 | 25..=28 | 35 | 37..=42 => {
                    vector3.1 = 1.0;
                    vector.1 = 1.0;
                    vector2.1 = 1.0;
                }
                3 | 4 | 5 | 7..=13 | 16 | 18 | 19 | 36 => {
                    vector3.1 = 0.0;
                    vector.1 = 0.0;
                    vector2.1 = 0.0;
                }
                6 => {
                    vector3.1 = -1.0;
                    vector.1 = -1.0;
                    vector2.1 = -1.0;
                }
                24 => {
                    vector3.1 = 0.0;
                    vector = (0.0, -4.0);
                    vector2 = (0.0, -4.0);
                }
                _ => {}
            }
            if t.frame_y % 40 != 0 {
                num2 -= 1;
            }
            if (t.frame_x % 54 == 0 && target_direction == -1)
                || (t.frame_x % 54 == 36 && target_direction == 1)
            {
                seat_down_offset = vector;
            } else if (t.frame_x % 54 == 0 && target_direction == 1)
                || (t.frame_x % 54 == 36 && target_direction == -1)
            {
                seat_down_offset = vector2;
            } else {
                seat_down_offset = vector3;
            }
            // `seatDownOffset += zero2` with `zero2.X = 1f` (`:239`, `:313`).
            seat_down_offset.0 += 1.0;
        }
        _ => unreachable!("the kind gate above"),
    }
    // `playerSittingPosition = new Point(num, num2 + 1).ToWorldCoordinates(8, 16)`
    // `.X += targetDirection * num3; .X += zero.X` (`:317-319`).
    let seat_x = num as f32 * 16.0 + 8.0 + target_direction as f32 * num3 as f32 + zero.0;
    let seat_y = (num2 + 1) as f32 * 16.0 + 16.0;
    Some((seat_x, seat_y, target_direction, seat_down_offset))
}

/// The dispatcher's sit arm: wraps [`sitting_target_info`] into the
/// [`TileInteraction::ChairSit`] event.
fn chair_sit_target(world: &World, x: i32, y: i32, player_direction: i32) -> TileInteraction {
    match sitting_target_info(world, x, y, player_direction) {
        Some((seat_x, seat_y, direction, seat_offset)) => TileInteraction::ChairSit {
            seat_x,
            seat_y,
            direction,
            seat_offset,
        },
        None => TileInteraction::None,
    }
}

/// `PlayerSleepingHelper.GetSleepingTargetInfo` (`PlayerSleepingHelper.cs:192-224`)
/// — the bed sleep decode: `(anchor_x, anchor_y, target_direction, bed_style)`
/// where the style keys `SetOffsetbyBed` (`:226-279`). `CanBeSleptIn` is
/// `CreateBoolSet(false, 79)` (`TileID.cs:514`).
pub fn sleeping_target_info(world: &World, x: i32, y: i32) -> Option<(f32, f32, i32, u32)> {
    let t = world.tile(x, y)?;
    if !t.active() || t.kind != 79 {
        return None;
    }
    let mut num = y;
    let num2 = x - (t.frame_x as i32 % 72) / 18;
    if t.frame_y % 36 != 0 {
        num -= 1;
    }
    let mut target_direction = 1i32;
    let mut num4 = num2;
    match t.frame_x / 72 {
        0 => {
            target_direction = -1;
            num4 += 1;
        }
        1 => {
            num4 += 2;
        }
        _ => {}
    }
    let anchor_x = num4 as f32 * 16.0 + 8.0;
    let anchor_y = (num + 1) as f32 * 16.0 + 16.0;
    Some((anchor_x, anchor_y, target_direction, t.frame_y as u32 / 36))
}

/// The dial use-trigger (`Player.cs:32636-32653`): the click is consumed
/// unconditionally (`flag2 = true`, `:32637`); the arm fires only under the
/// compound gate `!fastForwardTimeToX && cooldown == 0` (`:32638`/:32648 —
/// the `netMode == 1` disjunct is the net client's, not ported). On fire,
/// `Main.Sundialing()`/`Moondialing()` (`Main.cs:6169-6196`, SP branch) arm
/// the fast-forward and start the 8-day cooldown; a refused click changes
/// nothing and plays no sound. `sundial` selects the dial (356) over the
/// moondial (663).
fn dial_interaction(world: &mut World, sundial: bool) -> TileInteraction {
    let (already_ff, cooling) = if sundial {
        (
            world.fast_forward_to_dawn,
            world.sundial_cooldown != 0, // `Main.sundialCooldown == 0`, `:32638`
        )
    } else {
        (
            world.fast_forward_to_dusk,
            world.moondial_cooldown != 0, // `Main.moondialCooldown == 0`, `:32648`
        )
    };
    if already_ff || cooling {
        // Refused: the gate at `:32638`/:32648 is false — the click was still
        // consumed, but no dial arms and no sound plays.
        return TileInteraction::DialRefused;
    }
    if sundial {
        world.sundialing(); // Main.Sundialing, Main.cs:6169-6181
    } else {
        world.moondialing(); // Main.Moondialing, Main.cs:6184-6196
    }
    TileInteraction::DialStarted { sundial }
}

/// `WorldGen.UnlockDoor` (`WorldGen.cs:37599-37628`): the locked golden
/// door's column walks UP from `(i, j)` to the `frameY == 594` top cell;
/// a `frameY < 594` cell or the world edge aborts BEFORE any write (the
/// C# reads `Main.tile[i, num]` after the decrement — an out-of-world read
/// throws; the port answers `false`, the fail-neutral guard). On the found
/// top the three door rows shift `frameY += 54` (the locked sheet row to
/// the unlocked one). The sound (`PlaySound(22, i * 16, num * 16 + 16)`
/// `:37614` — the Unlock cue at the door's MIDDLE row), the 4-per-row dust
/// 11 (:37622-37625) and the progression event 22 (:37627) are app-side;
/// the returned `Some(top_y)` is the cue row's anchor. `None` = no locked
/// top found (nothing written, no sound).
pub fn unlock_door(world: &mut World, i: i32, j: i32) -> Option<i32> {
    // `:37602-37605` — the clicked cell must exist.
    world.tile(i, j)?;
    let mut num = j;
    // `:37606-37613` — walk up to the frameY-594 top; a lower frame or the
    // edge aborts (the C#'s `num <= 0` guard rides the same read).
    while world.tile(i, num)?.frame_y != 594 {
        num -= 1;
        let t = world.tile(i, num)?;
        if t.frame_y < 594 || num <= 0 {
            return None;
        }
    }
    // `:37615-37626` — the three rows shift to the unlocked sheet row.
    for k in num..=num + 2 {
        if let Some(t) = world.tile_mut(i, k) {
            t.frame_y += 54;
            touch(world, i, k);
        }
    }
    Some(num)
}

/// `WorldGen.GetTopLeftAndStyles` (`WorldGen.cs:51777-51790`): the object's
/// top-left snap + the style point. `frameXinc`/`frameYinc` are the sheet
/// strides (18/18 for the trapdoors); a zero stride skips that axis' snap
/// (the C# guard). Returns `(style_x, style_y)` and mutates `(x, y)` to the
/// top-left, exactly like the C# `ref` parameters.
fn get_top_left_and_styles(
    world: &World,
    x: &mut i32,
    y: &mut i32,
    w: i32,
    h: i32,
    frame_xinc: i32,
    frame_yinc: i32,
) -> (i32, i32) {
    let t = world.tile(*x, *y).copied().unwrap_or_default();
    let (fx, fy) = (i32::from(t.frame_x), i32::from(t.frame_y));
    let result = (fx / (w * frame_xinc), fy / (h * frame_yinc));
    if frame_xinc != 0 {
        *x -= fx / frame_xinc % w;
    }
    if frame_yinc != 0 {
        *y -= fy / frame_yinc % h;
    }
    result
}

/// A `TileFrame` rectangle walk (`WorldGen.TileFrame(x+m, y+n)` over the
/// given inclusive offsets) — the trapdoor/gate tails frame a NON-square
/// region, so `reframe_area`'s square radius cannot stand in. Each cell runs
/// the port's `TileFrame` stand-in (the 1-cell reframe); the frame-important
/// kinds keep their stored frames (`reframe_area`'s gate).
fn tile_frame_rect(world: &mut World, x: i32, y: i32, mx: std::ops::RangeInclusive<i32>, my: std::ops::RangeInclusive<i32>) {
    for m in mx {
        for n in my.clone() {
            framing::reframe_area(world, x + m, y + n, 1, false);
        }
    }
}

/// One trapdoor/gate toggle's result: the success flag, the object drops the
/// refusal/cut kills produced (the `open_door`/`close_door` convention — the
/// app spawns them on its stage's stream), and the refuse-cue position.
/// `refuse_cue` is `Some` only on `ShiftTrapdoor`'s fall-through (`:51594`,
/// the 9 at the clicked cell) — the in-arm `return false` refusals and the
/// tall gate's refusals play NOTHING and leave it `None`.
#[derive(Default)]
pub struct ShiftResult {
    pub shifted: bool,
    pub object_drops: Vec<ObjectDrop>,
    pub refuse_cue: Option<(i32, i32)>,
}

/// `WorldGen.ShiftTrapdoor` (`WorldGen.cs:51447-51596`): the 2x2 open
/// trapdoor (386) flips to the 2x1 closed one (387) and back. `player_above`
/// picks the closing side (`ToDirectionInt`: true → +1, the door closes
/// BELOW the player's row; false → -1, above). `only_close_or_open`: 1
/// refuses the 386→387 arm, 0 refuses the 387→386 arm, -1 (the interaction
/// default) allows both. The three success arms play the pitched door cue
/// (`PlaySound(8, ...)` `:51470`/:51512/:51563` — app-side, at the returned
/// `cue_pos`); the fall-through refuse plays 9 (`:51594`) at the SNAPPED
/// `(x, y)` — the refuse position is the caller's clicked cell for the
/// wrong-kind fall-through (the snap only runs inside the arms). The
/// `Wiring.SkipWire` quads (`:51459-51464` etc.) are the standing no-wiring
/// skip; `cell_blocked` is the `Collision.EmptyTile(ignoreTiles: true)`
/// entity half.
pub fn shift_trapdoor(
    world: &mut World,
    x: i32,
    y: i32,
    player_above: bool,
    only_close_or_open: i32,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
) -> (ShiftResult, Option<(i32, i32)>) {
    let mut out = ShiftResult::default();
    let Some(tile) = world.tile(x, y).copied() else {
        return (out, None);
    };
    let (mut sx, mut sy) = (x, y);
    if tile.kind == 386 && only_close_or_open != 1 {
        // `:51452` — the 2x2 top-left snap, styles (frameX/36, frameY/36).
        let styles = get_top_left_and_styles(world, &mut sx, &mut sy, 2, 2, 18, 18);
        if styles.0 == 0 {
            // `:51455-51469` — the square must be intact; the two cells
            // BELOW the bottom row must be entity-free.
            if !validate_tile_square(world, sx, sy, 2, 2, 386, &mut out.object_drops) {
                return (out, None);
            }
            if cell_blocked(sx, sy + 1) || cell_blocked(sx + 1, sy + 1) {
                return (out, None);
            }
            // `:51470` — the cue at (x*16+16, y*16+16): the 2x2's centre.
            let cue = (sx * 16 + 16, sy * 16 + 16);
            // `:51471-51478` — the TOP row's 386 cells clear (`ClearTile`:
            // active off, slope/halfbrick off — the kind stays until the
            // re-frame, the C# shape).
            for i in 0..2 {
                if let Some(t) = world.tile_mut(sx + i, sy) {
                    if t.active() && t.kind == 386 {
                        t.flags &= !crate::tile::FLAG_ACTIVE & !crate::tile::FLAG_INACTIVE;
                        t.set_slope(0);
                        t.flags &= !crate::tile::FLAG_HALF_BRICK;
                        touch(world, sx + i, sy);
                    }
                }
            }
            // `:51479-51485` — the BOTTOM row becomes the closed 2x1.
            for j in 0..2 {
                if let Some(t) = world.tile_mut(sx + j, sy + 1) {
                    t.kind = 387;
                    t.frame_x = (j * 18) as i16;
                    t.frame_y = (styles.1 * 18) as i16;
                    touch(world, sx + j, sy + 1);
                }
            }
            ensure_frame_important(world, 387);
            // `:51486-51492` — the 4x3 TileFrame ring (m -1..=2, l -1..=2).
            tile_frame_rect(world, sx, sy, -1..=2, -1..=2);
            out.shifted = true;
            return (out, Some(cue));
        }
        if styles.0 == 1 {
            // `:51495-51531` — the mirrored arm: the BOTTOM row clears, the
            // TOP row becomes the closed 2x1; the entity gate reads the TOP
            // row (`:51508`), the cue sits at (x*16+16, y*16) (`:51512`).
            if !validate_tile_square(world, sx, sy, 2, 2, 386, &mut out.object_drops) {
                return (out, None);
            }
            if cell_blocked(sx, sy) || cell_blocked(sx + 1, sy) {
                return (out, None);
            }
            let cue = (sx * 16 + 16, sy * 16);
            // `:51513-51516` — the BOTTOM row clears UNCONDITIONALLY (the
            // C#'s `Framing.GetTileSafely(x + m, y + 1).ClearTile()` — no
            // active/kind guard, unlike the X==0 arm).
            for m in 0..2 {
                if let Some(t) = world.tile_mut(sx + m, sy + 1) {
                    t.flags &= !crate::tile::FLAG_ACTIVE & !crate::tile::FLAG_INACTIVE;
                    t.set_slope(0);
                    t.flags &= !crate::tile::FLAG_HALF_BRICK;
                    touch(world, sx + m, sy + 1);
                }
            }
            for n in 0..2 {
                if let Some(t) = world.tile_mut(sx + n, sy) {
                    t.kind = 387;
                    t.frame_x = (n * 18) as i16;
                    t.frame_y = (styles.1 * 18) as i16;
                    touch(world, sx + n, sy);
                }
            }
            ensure_frame_important(world, 387);
            // `:51524-51530` — the ring: m -1..=2, num2 -1..=1 (one row
            // SHORTER than the X==0 arm — the C# asymmetry, kept).
            tile_frame_rect(world, sx, sy, -1..=2, -1..=1);
            out.shifted = true;
            return (out, Some(cue));
        }
    }
    if tile.kind == 387 && only_close_or_open != 0 {
        // `:51536` — the 2x1 snap (styles (frameX/36, frameY/18)).
        let styles = get_top_left_and_styles(world, &mut sx, &mut sy, 2, 1, 18, 18);
        // `:51539-51542` — either closed cell unbreakable refuses.
        let holds = |_, _| false;
        if (world.tile(sx, sy).is_some_and(|t| t.active() && t.kind == 387)
            && !can_kill_tile(world, sx, sy, holds))
            || (world.tile(sx + 1, sy).is_some_and(|t| t.active() && t.kind == 387)
                && !can_kill_tile(world, sx + 1, sy, holds))
        {
            return (out, None);
        }
        if !validate_tile_square(world, sx, sy, 2, 1, 387, &mut out.object_drops) {
            return (out, None);
        }
        // `:51547-51555` — the closing side's row must hold no real blocker
        // (cut/drip tiles pass — they die below).
        let num3 = if player_above { 1 } else { -1 };
        for num4 in 0..2 {
            if let Some(t) = world.tile(sx + num4, sy + num3) {
                if t.active()
                    && !crate::tileobject::tile_cut(t.kind)
                    && !crate::tileobject::is_drip_tile(t.kind)
                {
                    return (out, None);
                }
            }
        }
            // `:51563` — the cue at (x*16+16, y*16).
            let cue = (sx * 16 + 16, sy * 16);
            // `:51564-51571` — the cut/drip tiles in the closing row die.
            for num5 in 0..2 {
                let cut = world.tile(sx + num5, sy + num3).is_some_and(|t| {
                    t.active()
                        && (crate::tileobject::tile_cut(t.kind)
                            || crate::tileobject::is_drip_tile(t.kind))
                });
                if cut {
                    if let Some(kill) = kill_tile(world, sx + num5, sy + num3, false) {
                        out.object_drops.extend(kill.object_drops);
                    }
                }
            }
            // `:51572-51584` — the 2x2 open trapdoor writes, each column taking
            // its closed cell's paint/coating cache (`BlockColorAndCoating` →
            // `UseBlockColors`). `frameX = col*18 + playerAbove*36` (the sheet's
            // above/below halves), `frameY = row*18`; the row offset shifts by
            // `(!playerAbove).ToInt()` — the door grows AWAY from the player.
            let above = i32::from(player_above);
            let row_shift = i32::from(!player_above);
            for num6 in 0..2 {
                let paint = world.tile(sx + num6, sy).map_or(0, |t| t.paint());
                let fullbright = world
                    .tile(sx + num6, sy)
                    .is_some_and(|t| t.fullbright_block());
                let invisible = world.invisible_block(sx + num6, sy);
                for num7 in 0..2 {
                    let (cx, cy) = (sx + num6, sy + num7 - row_shift);
                    if let Some(t) = world.tile_mut(cx, cy) {
                        t.kind = 386;
                        t.frame_x = (num6 * 18 + above * 36) as i16;
                        t.frame_y = (num7 * 18) as i16;
                        t.set_paint(paint);
                        if fullbright {
                            t.flags |= crate::tile::FLAG_FULLBRIGHT;
                        } else {
                            t.flags &= !crate::tile::FLAG_FULLBRIGHT;
                        }
                        t.flags |= crate::tile::FLAG_ACTIVE;
                    }
                    world.set_invisible_block(cx, cy, invisible);
                    touch(world, cx, cy);
                }
            }
            ensure_frame_important(world, 386);
            // `:51585-51591` — the 4x4 ring: num8 -1..=2, num9 -1..=2 with
            // the row offset ×2 (`y + num9 - (!playerAbove)*2`).
            tile_frame_rect(world, sx, sy - row_shift * 2, -1..=2, -1..=2);
            out.shifted = true;
            return (out, Some(cue));
        }
        // `:51594` — the refuse plays 9 at `(x * 16, y * 16)`. The C# `x`/`y`
        // are `ref` params: `GetTopLeftAndStyles` already snapped them when
        // the 386 arm ran (the X∉{0,1} fall-through), so the cue rides the
        // SNAPPED `sx`/`sy` (equal to the clicked cell when no snap ran).
        out.refuse_cue = Some((sx * 16, sy * 16));
        (out, None)
    }

/// `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`): the 1x5 tall gate
/// flips 388 (closed) ↔ 389 (open). `closing` picks the target kind;
/// `forced` skips the per-row `Collision.EmptyTile` gate (the wired caller;
/// the interaction path sends false). The success plays the pitched 8 at
/// `(x*16+16, y*16+16)` (`:51762`, app-side via the returned cue position).
/// The gate's `CoordinateHeights` are {18,16,16,16,18} (the registry def,
/// `TileObjectData.cs:2300`), full height 94 — the row walk finds the clicked
/// cell's row inside the style block.
pub fn shift_tall_gate(
    world: &mut World,
    x: i32,
    y: i32,
    closing: bool,
    forced: bool,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
) -> (ShiftResult, Option<(i32, i32)>) {
    let mut out = ShiftResult::default();
    let target: u16 = if closing { 388 } else { 389 };
    let source: u16 = if closing { 389 } else { 388 };
    let Some(tile) = world.tile(x, y).copied() else {
        return (out, None);
    };
    if tile.kind != source {
        return (out, None); // `:51728-51731`
    }
    // `:51732-51743` — the style column (frameX/18) and the row decode: the
    // style block is 94 px tall, the clicked row walks the CoordinateHeights.
    let style_x = i32::from(tile.frame_x) / 18;
    let Some(data) = crate::tileobject::def_for_tile(388) else {
        return (out, None);
    };
    let full_h = data.coordinate_full_height();
    let mut row = 0usize;
    let mut num2 = i32::from(tile.frame_y) % full_h;
    while row < data.coordinate_heights.len()
        && num2 - data.coordinate_heights[row] >= 0
    {
        num2 -= data.coordinate_heights[row];
        row += 1;
    }
    let gy = y - row as i32;
    let height = data.height;
    let width = data.width;
    // `:51744-51747` — the 1x5 square must be intact.
    if !validate_tile_square(world, x, gy, width, height, source, &mut out.object_drops) {
        return (out, None);
    }
    // `:51755-51761` — every row entity-free unless forced.
    if !forced {
        for k in 0..height {
            if cell_blocked(x, gy + k) {
                return (out, None);
            }
        }
    }
    // `:51762` — the cue at the gate's second-row centre.
    let cue = (x * 16 + 16, gy * 16 + 16);
    // `:51763-51766` — the column flips kind, frames kept.
    for l in 0..height {
        if let Some(t) = world.tile_mut(x, gy + l) {
            t.kind = target;
            touch(world, x, gy + l);
        }
    }
    ensure_frame_important(world, target);
    // `:51767-51773` — the 3x(height+2) TileFrame ring (m -1..=1,
    // n -1..=height).
    tile_frame_rect(world, x, gy, -1..=1, -1..=height);
    out.shifted = true;
    (out, Some(cue))
}

/// `WorldGen.OpenDoor` (`WorldGen.cs:37630-37784`): turns the closed door
/// (tile 10) covering `(i, j)` into the open door (tile 11) leaning to
/// `direction` (+1 right, -1 left). No `KillTile` on the door cells — the
/// six open-door cells are written directly; `KillTile` appears only in the
/// validate-cleanup helper and the cut/drip sweep in the opening column.
/// `drops` collects the object destructions of those kills (vanilla's
/// `Item.NewItem` side effects). `false` = refused, door cells untouched.
/// `wire_skips` collects the `Wiring.SkipWire` cells of the `:37717-37725`
/// block (the caller passes `Some` only while `Wiring.running`, the C#
/// `Main.netMode != 1 && Wiring.running` gate; the interaction/NPC callers
/// pass `None`). The wired caller inserts them into its skip set after the
/// call (order-identical: the C# skips precede the writes and the trailing
/// reframe reads no skip state). The Option shape is f11's §1 seam, adopted
/// by the research/95 union-exec (W4-c42).
/// `rng` threads the caller's ACTIVE `genRand` stream (W4-g83 contract §2):
/// the validate-cleanup and cut/drip `KillTile`s' framing dispatch draws its
/// seaweed tip re-rolls on it.
pub fn open_door(
    world: &mut World,
    i: i32,
    j: i32,
    direction: i32,
    drops: &mut Vec<ObjectDrop>,
    wire_skips: Option<&mut Vec<(i32, i32)>>,
    rng: &mut UnifiedRandom,
) -> bool {
    // The per-cell null-init guards (:37634-37644) are managed-Tile engine
    // noise; the port reads through `World::tile`'s `Option` instead.
    let Some(tile) = world.tile(i, j) else {
        return false;
    };
    if tile.kind != 10 {
        return false; // :37645-37647
    }
    if is_locked_door(tile) {
        return false; // :37649-37652 → IsLockedDoor :69191
    }
    // Frame decode (:37653-37668): `frameY` splits into a 54 px style row
    // (`num4`) and an 18 px in-door row (`num3`); `frameX >= 54` carries
    // horizontal door styles that shift the open door's sheet cell.
    let mut num3 = i32::from(tile.frame_y);
    let mut num4 = 0i32;
    let mut num2 = 0i32;
    while num3 >= 54 {
        num3 -= 54;
        num4 += 1;
    }
    if tile.frame_x >= 54 {
        let num5 = i32::from(tile.frame_x) / 54;
        num4 += 36 * num5;
        num2 += 72 * num5;
    }
    let num = j - num3 / 18; // :37669 — the door's top row
    let mut num6 = i;
    // :37672-37675 — the 1x3 closed door must be intact, or whatever matches
    // is killed and the open refused.
    if !validate_tile_square(world, num6, num, 1, 3, 10, drops, rng) {
        return false;
    }
    // Paint caches (:37676-37690), read before any write — always at column
    // `i` (`num6` is reassigned only afterwards), one per door row, so the
    // closed door's paint carries onto both written columns in either
    // direction.
    let cache: [u8; 3] = [
        world.tile(i, num).map_or(0, |t| t.paint()),
        world.tile(i, num + 1).map_or(0, |t| t.paint()),
        world.tile(i, num + 2).map_or(0, |t| t.paint()),
    ];
    let num7;
    if direction == -1 {
        num6 = i - 1; // :37698-37702
        num2 += 36;
        num7 = i - 1;
    } else {
        num6 = i; // :37703-37706
        num7 = i + 1;
    }
    // The opening column must hold no real blocker (:37702-37710): active
    // tiles that are not cut / 165 / drip refuse the open.
    for k in num..num + 3 {
        if let Some(t) = world.tile(num7, k) {
            if t.active()
                && !crate::tileobject::tile_cut(t.kind)
                && t.kind != 165
                && !crate::tileobject::is_drip_tile(t.kind)
            {
                return false;
            }
        }
    }
    // Cut / 165 / drip tiles in the way die with a full `KillTile`
    // (:37711-37718; default flags — no suppression exists in the C# body).
    for l in num..num + 3 {
        if let Some(t) = world.tile(num7, l) {
            if t.active()
                && (crate::tileobject::tile_cut(t.kind)
                    || t.kind == 165
                    || crate::tileobject::is_drip_tile(t.kind))
            {
                if let Some(kill) = kill_tile(world, num7, l, false, rng) {
                    drops.extend(kill.object_drops);
                }
            }
        }
    }
    // `Wiring.SkipWire` ×6 (:37717-37725) — the `Wiring.running`-gated block;
    // the wired caller collects them through `wire_skips` (the §1 Option
    // seam), the interaction path has no running flood.
    if let Some(skips) = wire_skips {
        for dy in 0..3 {
            skips.push((num6, num + dy));
            skips.push((num6 + 1, num + dy));
        }
    }
    let num8 = num4 % 36 * 54; // :37731 — the open door's style row
    // `SoundEngine.PlaySound(8, i*16, j*16)` (:37732) — app-side, via
    // `TileInteraction::DoorOpened`.
    // Fail-neutral world-edge guard (the C# `Main.tile[...]` writes would
    // throw; `CanPlace`'s 5-tile margin makes the case unreachable in play).
    if !world.in_bounds(num6, num) || !world.in_bounds(num6 + 1, num + 2) {
        return false;
    }
    // The 2x3 open door (:37733-37776): `frameX = num2 + dx*18`, `frameY =
    // num8 + dy*18`, `UseBlockColors(cache)` row by row.
    for dy in 0..3i32 {
        for dx in 0..2i32 {
            let t = world.tile_mut(num6 + dx, num + dy).unwrap();
            t.flags |= crate::tile::FLAG_ACTIVE;
            t.kind = 11;
            t.frame_y = (num8 + dy * 18) as i16;
            t.frame_x = (num2 + dx * 18) as i16;
            t.set_paint(cache[dy as usize]);
            touch(world, num6 + dx, num + dy);
        }
    }
    // The trailing `WorldGen.TileFrame` 4x4 loop (:37778-37784): door cells
    // keep their frames (frame-important), neighbours re-derive theirs.
    ensure_frame_important(world, 11);
    framing::reframe_area(world, num6, num, 2, false);
    true
}

/// `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`): swaps the tall gate
/// covering `(x, y)` between its closed (388) and open (389) kinds. `closing`
/// selects the target kind (true → 388, false → 389); the source is the
/// opposite. `forced` skips the `Collision.EmptyTile` occupancy gate (the
/// wired caller); the NPC door-bash drain sends false. `cell_blocked` is the
/// player/NPC overlap check of `EmptyTile(ignoreTiles: true)`
/// (`Collision.cs:1359`). The `PlaySound(8)` cue is app-side (the drain plays
/// it on the `UpdateNPCs` stream, the `open_door` convention). `drops`
/// collects the validate-cleanup kills.
pub fn shift_tall_gate(
    world: &mut World,
    x: i32,
    y: i32,
    closing: bool,
    forced: bool,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
    drops: &mut Vec<ObjectDrop>,
) -> bool {
    let target: u16 = if closing { 388 } else { 389 }; // :51725
    let source: u16 = if closing { 389 } else { 388 }; // :51726
    let Some(tile) = world.tile(x, y) else {
        return false;
    };
    if tile.kind != source {
        return false; // :51728-51731
    }
    // :51732-51743 — the gate's top row: the style from frameX/18, the
    // in-gate row from frameY modulo the def's full coordinate height, then
    // the coordinate-heights walk back to the top.
    let style = i32::from(tile.frame_x) / 18;
    let Some(def) = crate::tileobject::def_for_tile(388) else {
        return false;
    };
    let Some(data) = crate::tileobject::get_tile_data(def, style, 1) else {
        return false;
    };
    let full_h = data.coordinate_full_height();
    if full_h <= 0 {
        return false;
    }
    let mut row = i32::from(tile.frame_y) % full_h;
    let mut i = 0i32;
    while i < data.height && row - data.coordinate_heights[i as usize] >= 0 {
        row -= data.coordinate_heights[i as usize];
        i += 1;
    }
    let top = y - i;
    let (width, height) = (data.width, data.height);
    // :51744-51747 — the intact source square, or the cleanup kills refuse.
    if !validate_tile_square(world, x, top, width, height, source, drops) {
        return false;
    }
    // :51748-51754 — `Wiring.SkipWire`: no wiring in the port.
    // :51755-51761 — every gate cell must be entity-free (ignoreTiles: true).
    for k in 0..height {
        if !forced && cell_blocked(x, top + k) {
            return false;
        }
    }
    // :51762 — `SoundEngine.PlaySound(8, x*16+16, y*16+16)`: app-side.
    // :51763-51766 — the kind swap, row by row.
    for l in 0..height {
        if let Some(t) = world.tile_mut(x, top + l) {
            t.kind = target;
            touch(world, x, top + l);
        }
    }
    // :51767-51773 — the `WorldGen.TileFrame` 3×(height+2) neighbourhood:
    // gate cells keep their frames (frame-important), neighbours re-derive.
    ensure_frame_important(world, target);
    framing::reframe_area(world, x, top + height / 2, 2, false);
    true
}

/// `WorldGen.CloseDoor` (`WorldGen.cs:31805-31907`): turns the open door
/// (tile 11) covering `(i, j)` back into the closed door (tile 10) at the
/// hinge column the open frames encode. `forced` skips the `Collision.
/// EmptyTile` occupancy gate (the wired caller, `Wiring.cs:1463`); the
/// interaction path sends `false`. `cell_blocked` is the player/NPC overlap
/// check of `EmptyTile` (`Collision.cs:1359` — with `ignoreTiles: true` the
/// tile check is skipped, entities are not). `wire_skips` collects the
/// `Wiring.SkipWire` cells of the `:31895-31899` block (the caller passes
/// `Some` only while `Wiring.running`; the f11 §1 Option seam, adopted by
/// the research/95 union-exec — the C# skips precede the trailing reframe,
/// which reads no skip state, so collecting them out and inserting after
/// the call is order-identical).
/// EmptyTile` occupancy gate (the wired caller, `Wiring.cs:1463`); the port
/// has no wiring so the interaction path always sends `false`. `cell_blocked`
/// is the player/NPC overlap check of `EmptyTile` (`Collision.cs:1359` —
/// with `ignoreTiles: true` the tile check is skipped, entities are not).
/// `rng` threads the caller's ACTIVE `genRand` stream (W4-g83 contract §2,
/// the [`open_door`] note).
pub fn close_door(
    world: &mut World,
    i: i32,
    j: i32,
    forced: bool,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
    drops: &mut Vec<ObjectDrop>,
    wire_skips: Option<&mut Vec<(i32, i32)>>,
    rng: &mut UnifiedRandom,
) -> bool {
    let Some(tile) = world.tile(i, j) else {
        return false;
    };
    let frame_x = i32::from(tile.frame_x); // :31816
    if tile.kind != 11 {
        return false; // :31818-31820
    }
    // Frame decode (:31821-31833). `num5` is written but never read in the
    // decompile — kept out; `num6` is the closed-door style offset the hinge
    // cell receives.
    let mut num4 = i32::from(tile.frame_y);
    while num4 >= 54 {
        num4 -= 54;
    }
    let mut num6 = 0i32;
    if frame_x >= 72 {
        num6 += 54 * (frame_x / 72);
    }
    let num3 = j - num4 / 18; // :31833 — the door's top row
    // `switch (frameX % 72)` (:31834-31851): which column of the open pair is
    // the hinge (`num2`) and which way the door swung (`num`, only -1 is
    // tested). The C# default case leaves `num = 0`, `num2 = i`.
    let mut num2 = i;
    let mut num = 0i32;
    match frame_x % 72 {
        0 => {
            num2 = i;
            num = 1;
        }
        18 => {
            num2 = i - 1;
            num = 1;
        }
        36 => {
            num2 = i + 1;
            num = -1;
        }
        54 => {
            num2 = i;
            num = -1;
        }
        _ => {}
    }
    let mut num7 = num2; // :31852-31854 — the pair's left column
    if num == -1 {
        num7 = num2 - 1;
    }
    // `Collision.EmptyTile(num2, k, ignoreTiles: true)` (:31856-31864): with
    // `ignoreTiles` the tile check is skipped — only player/NPC overlap can
    // refuse the close (:1359-1378). `forced` (the wired caller,
    // `Wiring.cs:1463`) skips the gate.
    if !forced {
        for k in num3..num3 + 3 {
            if cell_blocked(num2, k) {
                return false;
            }
        }
    }
    // :31866-31869 — the 2x3 open door must be intact, or whatever matches is
    // killed and the close refused.
    if !validate_tile_square(world, num7, num3, 2, 3, 11, drops, rng) {
        return false;
    }
    // The write (:31871-31891): the hinge column becomes the closed door —
    // `frameX = genRand.Next(3) * 18 + num6`, `frameY` deliberately NOT
    // rewritten (vanilla quirk) — and the other column deactivates. The port
    // has no runtime `WorldGen.genRand` stream to draw from (and introducing
    // one would touch the determinism invariant), so the frame variant is
    // always 0 — the same value a fresh style-0 placement writes.
    for l in num7..num7 + 2 {
        for m in num3..num3 + 3 {
            let Some(t) = world.tile_mut(l, m) else {
                continue;
            };
            if t.active() && t.kind == 11 {
                if l == num2 {
                    t.kind = 10;
                    t.frame_x = num6 as i16;
                } else {
                    t.flags &= !crate::tile::FLAG_ACTIVE;
                }
                touch(world, l, m);
            }
        }
    }
    // `Wiring.SkipWire` ×3 (:31895-31899) — the `Wiring.running`-gated block;
    // the wired caller collects them through `wire_skips` (the §1 Option
    // seam), the interaction path has no running flood.
    if let Some(skips) = wire_skips {
        for dy in 0..3i32 {
            skips.push((num2, num3 + dy));
        }
    }
    // The trailing `WorldGen.TileFrame` 3x4 loop (:31900-31905).
    ensure_frame_important(world, 10);
    framing::reframe_area(world, num2, num3, 2, false);
    // `SoundEngine.PlaySound(9, i*16, j*16)` (:31906) — app-side, via
    // `TileInteraction::DoorClosed`.
    true
}

/// `WorldGen.ValidateTileSquareIsActiveAndOfType` (`WorldGen.cs:37869-37917`):
/// `true` when every cell of the `width`×`height` square at `(x, y)` is
/// active and of `kind`. Otherwise every matching cell is `KillTile`d (full
/// cascade — drops collected) and `false` returned; an out-of-world square
/// returns `false` without killing (`WorldGen.InWorld`, `:9688`).
/// `pub`: the campfire light toggle (`wiring::try_toggle_light`) shares it.
pub fn validate_tile_square(
pub(crate) fn validate_tile_square(
    world: &mut World,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    kind: u16,
    drops: &mut Vec<ObjectDrop>,
    rng: &mut UnifiedRandom,
) -> bool {
    // :37874-37890 — an out-of-world square refuses without killing
    // (`WorldGen.InWorld` fluff 0, `:9688`); a mismatched cell marks the
    // square dirty.
    let mut mismatch = false;
    'check: for i in 0..width {
        for jj in 0..height {
            let (cx, cy) = (x + i, y + jj);
            if !world.in_bounds(cx, cy) {
                return false;
            }
            if !world.is_active(cx, cy) || world.tile(cx, cy).is_some_and(|t| t.kind != kind) {
                mismatch = true;
                break 'check;
            }
        }
    }
    if mismatch {
        // :37891-37914 — every active matching cell dies with a full
        // `KillTile` (default flags — the cascade runs, drops are collected).
        for i in 0..width {
            for jj in 0..height {
                let (cx, cy) = (x + i, y + jj);
                if world.is_active(cx, cy) && world.tile(cx, cy).is_some_and(|t| t.kind == kind) {
                    if let Some(kill) = kill_tile(world, cx, cy, false, rng) {
                        drops.extend(kill.object_drops);
                    }
                }
            }
        }
        return false;
    }
    true
}

/// Mirrors `Main.tileFrameImportant[type]` for the door kind a toggle writes
/// (`Main.cs:7637-7638` — a boot-time static both door tiles carry; `place`
/// mirrors the same flag for its own def, `tileobject.rs` DW-1.3, and the
/// toggle switches kinds under the same guarantee).
fn ensure_frame_important(world: &mut World, kind: u16) {
    if world.frame_important.len() <= kind as usize {
        world
            .frame_important
            .resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[kind as usize] = true;
}

/// `WorldGen.IsLockedDoor` (`WorldGen.cs:69191-69197`): the golden-door
/// frames (tile 10, `frameY` 594-646, `frameX` < 54). The wooden door never
/// matches; `OpenDoor` calls it, and the tile-use unlock arm reads the
/// indexed overload (`:69186-69189`, W4-g24).
pub fn is_locked_door(tile: &Tile) -> bool {
    tile.kind == 10 && (594..=646).contains(&tile.frame_y) && tile.frame_x < 54
}

/// The indexed `IsLockedDoor(x, y)` overload (`WorldGen.cs:69186-69189`):
/// an out-of-world cell reads false (the C# `Main.tile` read would throw;
/// the tile-use caller only reaches it with an active in-world cell).
pub fn is_locked_door_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(is_locked_door)
}

fn liquid_flags(flags: u8) -> u8 {
    use crate::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
    flags & (FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER)
}

/// Marks the cell as changed for the caches (`World::revision`, chunk occupancy).
fn touch(world: &mut World, x: i32, y: i32) {
    world.revision = world.revision.wrapping_add(1);
    world.mark_chunk(x, y);
}

/// `WorldGen.SolidTileNoAttach(i, j)` (`WorldGen.cs:70511-70528`): active,
/// solid, NOT `tileNoAttach`, flat (`slope == 0`, not half brick), not
/// actuated. Out-of-world reads TRUE (`:70515-70517`, the null arm). The
/// torch placement support gate reads it (`Player.cs:41114-41197`,
/// `WorldGen.cs:59835`'s below arm uses the platform variant instead).
pub fn solid_tile_no_attach(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_none_or(|t| {
        t.active()
            && crate::tiledata::TILE_SOLID.has(t.kind)
            && !crate::torch::no_attach_pub(t.kind)
            && t.slope() == 0
            && !t.half_brick()
            && !t.inactive()
    })
}

fn after_edit(world: &mut World, x: i32, y: i32) {
    world.recompute_column_surface(x);
    framing::reframe_area(world, x, y, 2, true);
}

// Gate merge (i31 ⨉ i42): the i42 torch-flatten copies of
// `WorldGen.SlopeTile`/`CanPoundTile` are folded into the i31
// `edit::hammer` canonical family (`slope_tile(world, x, y, 0, false,
// chest_gate)` — the full `:80768-80803` form incl. the `SquareTileFrame`
// cascade; the i42 copy's bare `reframe_area` was the placement-route
// slice). The torch flatten's caller (`flatten_torch_supports`) passes the
// real chest gate and keeps its app-side Dig cue.

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)] // DW-IDs are part of the test names (traceability)
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    // W4-g83: the edit entries thread the caller's ACTIVE genRand stream
    // (the door ops' cleanup KillTile framing dispatch). The witnesses pin
    // door state/drops, not streams — a fresh seed-0 stream per call.
    pub(super) fn rng() -> UnifiedRandom {
        UnifiedRandom::new(0)
    }

    pub(super) fn ground_world() -> World {
        let mut w = World::new("t".into(), 32, 32);
        w.frame_important = vec![false; 754];
        for x in 0..32 {
            for y in 20..32 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// W4-g24 — `WorldGen.UnlockDoor` (`WorldGen.cs:37599-37628`): the
    /// column walk finds the frameY-594 top from ANY of the three rows and
    /// shifts all three `+54`; a wooden door (frameY < 594) aborts with no
    /// write, and the returned top anchors the app's middle-row Unlock cue.
    #[test]
    fn w4_g24_unlock_door_walks_to_the_locked_top_and_shifts() {
        let mut w = object_world();
        // The locked golden door at column 10, rows 17-19 (frameY 594/612/630).
        set_object_cell(&mut w, 10, 17, 10, 0, 594);
        set_object_cell(&mut w, 10, 18, 10, 0, 612);
        set_object_cell(&mut w, 10, 19, 10, 0, 630);
        assert!(is_locked_door(w.tile(10, 19).unwrap()));
        // A click on the BOTTOM row walks up two cells to the 594 top.
        assert_eq!(unlock_door(&mut w, 10, 19), Some(17));
        assert_eq!(w.tile(10, 17).unwrap().frame_y, 648);
        assert_eq!(w.tile(10, 18).unwrap().frame_y, 666);
        assert_eq!(w.tile(10, 19).unwrap().frame_y, 684);
        // The unlocked frames are no longer locked (`:69193`'s band).
        assert!(!is_locked_door(w.tile(10, 17).unwrap()));
        // A second call finds no 594 top — the walk aborts, nothing shifts.
        assert_eq!(unlock_door(&mut w, 10, 17), None);
        assert_eq!(w.tile(10, 17).unwrap().frame_y, 648);
        // The wooden door (frameY 0-36) aborts on the first step down.
        let mut w2 = object_world();
        set_object_cell(&mut w2, 10, 17, 10, 0, 0);
        set_object_cell(&mut w2, 10, 18, 10, 0, 18);
        set_object_cell(&mut w2, 10, 19, 10, 0, 36);
        assert_eq!(unlock_door(&mut w2, 10, 19), None);
        assert_eq!(w2.tile(10, 19).unwrap().frame_y, 36);
        // An empty cell answers None (`:37602-37605`).
        assert_eq!(unlock_door(&mut w2, 20, 10), None);
    }

    /// W4-g24 — `WorldGen.ShiftTrapdoor` (`WorldGen.cs:51447-51596`): the
    /// 2x2 open (386) flips to the 2x1 closed (387) on the X==0 arm (the
    /// bottom row becomes the closed pair, the top row clears), the cue at
    /// the 2x2 centre; the reverse flip writes the 2x2 with the above/below
    /// sheet halves and carries the closed cells' paint.
    #[test]
    fn w4_g24_trapdoor_flips_both_ways_with_the_arm_cues() {
        let mut w = object_world();
        // The open 2x2 at (10, 18)-(11, 19), style 0 (frameX 0/18, frameY
        // 0/18 — the X==0 arm: frameX/36 == 0).
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            set_object_cell(&mut w, 10 + dx, 18 + dy, 386, (dx * 18) as i16, (dy * 18) as i16);
        }
        let free = |_: i32, _: i32| false;
        // Close (player_above = true → the door closes below): the bottom
        // row (y 19) becomes 387, the top row (y 18) clears.
        let (res, cue) = shift_trapdoor(&mut w, 10, 18, true, -1, &free);
        assert!(res.shifted);
        assert_eq!(cue, Some((10 * 16 + 16, 18 * 16 + 16)), "the 2x2 centre");
        assert_eq!(w.tile(10, 19).unwrap().kind, 387);
        assert_eq!(w.tile(11, 19).unwrap().kind, 387);
        assert!(!w.tile(10, 18).unwrap().active(), "the top row cleared");
        // Re-open from the closed pair (player_above = true → the 2x2 grows
        // DOWN, rows 19-20; frameX carries the +36 above-half).
        let (res, cue) = shift_trapdoor(&mut w, 10, 19, true, -1, &free);
        assert!(res.shifted);
        assert_eq!(cue, Some((10 * 16 + 16, 19 * 16)));
        assert_eq!(w.tile(10, 19).unwrap().kind, 386);
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36, "the above half");
        assert_eq!(w.tile(10, 20).unwrap().kind, 386);
        assert_eq!(w.tile(11, 20).unwrap().frame_x, 36 + 18);
        // The wrong-kind fall-through refuses with the 9 cue at the cell.
        let mut w2 = object_world();
        set_active(&mut w2, 10, 18, 1);
        let (res, cue) = shift_trapdoor(&mut w2, 10, 18, true, -1, &free);
        assert!(!res.shifted);
        assert!(cue.is_none());
        assert_eq!(res.refuse_cue, Some((10 * 16, 18 * 16)));
        // An entity in the closing row refuses the close (no cue — the
        // in-arm return, not the fall-through).
        let mut w3 = object_world();
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            set_object_cell(&mut w3, 10 + dx, 18 + dy, 386, (dx * 18) as i16, (dy * 18) as i16);
        }
        let blocked = |x: i32, y: i32| (x, y) == (10, 19);
        let (res, cue) = shift_trapdoor(&mut w3, 10, 18, true, -1, &blocked);
        assert!(!res.shifted);
        assert!(cue.is_none());
        assert!(res.refuse_cue.is_none(), "the in-arm refuse is silent");
        assert_eq!(w3.tile(10, 19).unwrap().kind, 386, "nothing shifted");
    }

    /// W4-g24 — `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`): the
    /// 1x5 column flips 388↔389 from ANY member row (the CoordinateHeights
    /// row walk), frames kept; the cue at the second-row centre; an entity
    /// in any row refuses unless forced.
    #[test]
    fn w4_g24_tall_gate_flips_from_any_row_and_gates_on_entities() {
        let mut w = object_world();
        // The closed gate at column 10, rows 14-18 (the 5 CoordinateHeights
        // {18,16,16,16,18} → frameY 0/18/34/50/66 within the style block).
        let fys = [0i16, 18, 34, 50, 66];
        for (row, fy) in fys.iter().enumerate() {
            set_object_cell(&mut w, 10, 14 + row as i32, 388, 0, *fy);
        }
        let free = |_: i32, _: i32| false;
        // A click on the MIDDLE row (16, frameY 34) walks up two rows.
        let (res, cue) = shift_tall_gate(&mut w, 10, 16, false, false, &free);
        assert!(res.shifted);
        assert_eq!(cue, Some((10 * 16 + 16, 14 * 16 + 16)));
        for row in 0..5 {
            let t = w.tile(10, 14 + row).unwrap();
            assert_eq!(t.kind, 389, "row {row} flipped");
            assert_eq!(t.frame_y, fys[row as usize], "the frame kept");
        }
        // The reverse (closing) flips back.
        let (res, _) = shift_tall_gate(&mut w, 10, 14, true, false, &free);
        assert!(res.shifted);
        assert_eq!(w.tile(10, 18).unwrap().kind, 388);
        // An entity in the top row refuses; forced skips the gate.
        let blocked = |x: i32, y: i32| (x, y) == (10, 14);
        let (res, cue) = shift_tall_gate(&mut w, 10, 16, false, false, &blocked);
        assert!(!res.shifted);
        assert!(cue.is_none());
        let (res, _) = shift_tall_gate(&mut w, 10, 16, false, true, &blocked);
        assert!(res.shifted, "forced skips the entity gate");
    }

    #[test]
    fn unbreakable_wall_blocks_wall_and_tile_edits() {
        let mut w = ground_world();
        // Wall 350 (`WallID.UnbreakableBlockWall`) cannot be hammered away
        // (`Player.CanPlayerSmashWall`).
        let i = w.index(10, 15);
        w.tiles[i].wall = WALL_UNBREAKABLE;
        w.compute_chunks();
        assert_eq!(break_wall(&mut w, 10, 15), None);
        assert_eq!(w.tile(10, 15).unwrap().wall, WALL_UNBREAKABLE);
        // A tile in front of it cannot be mined (`WorldGen.CanKillTile`,
        // `IsTileReplaceable`), and nothing may be placed into that cell.
        let i = w.index(10, 20);
        w.tiles[i].wall = WALL_UNBREAKABLE;
        w.compute_chunks();
        assert!(kill_tile(&mut w, 10, 20, true, &mut rng()).is_none());
        assert!(w.tile(10, 20).unwrap().active());
        assert_eq!(break_wall(&mut w, 10, 20), None);
        assert_eq!(w.tile(10, 20).unwrap().wall, WALL_UNBREAKABLE);
    }

    // ---- W4-g79: the paint/coating world writes ----------------------------

    /// `WorldGen.paintTile` (`WorldGen.cs:44110-44132`): an active tile takes
    /// the colour; a same-colour repaint and an inactive cell refuse.
    #[test]
    fn w4_g79_paint_tile_writes_the_colour_and_refuses_a_repaint() {
        let mut w = ground_world();
        assert!(paint_tile(&mut w, 10, 20, 1), "the unpainted floor takes Red");
        assert_eq!(w.tile(10, 20).unwrap().paint(), 1);
        assert!(!paint_tile(&mut w, 10, 20, 1), "the same colour refuses");
        assert!(paint_tile(&mut w, 10, 20, 30), "Negative over Red lands");
        assert_eq!(w.tile(10, 20).unwrap().paint(), 30);
        // An inactive cell (above the floor) refuses.
        assert!(!paint_tile(&mut w, 10, 15, 1));
    }

    /// `WorldGen.paintWall` (`WorldGen.cs:44230-44252`): a walled cell takes
    /// the colour; a wall-less cell and a same-colour repaint refuse.
    #[test]
    fn w4_g79_paint_wall_writes_the_colour_and_needs_a_wall() {
        let mut w = ground_world();
        let i = w.index(10, 15);
        w.tiles[i].wall = 4;
        assert!(paint_wall(&mut w, 10, 15, 9), "the wood wall takes Blue");
        assert_eq!(w.wall_paint(10, 15), 9);
        assert!(!paint_wall(&mut w, 10, 15, 9), "the same colour refuses");
        // A wall-less cell refuses even when active.
        assert!(!paint_wall(&mut w, 10, 20, 1));
    }

    /// `WorldGen.paintCoatTile` (`WorldGen.cs:44134-44180`): the Glow (1) and
    /// Echo (2) bits set once, the clear (0) arm needs at least one bit, and an
    /// unknown id refuses. The Glow bit rides the tile flag, the Echo bit the
    /// extras byte.
    #[test]
    fn w4_g79_paint_coat_tile_sets_glow_and_echo_then_clears() {
        let mut w = ground_world();
        assert!(paint_coat_tile(&mut w, 10, 20, 1), "Glow lands");
        assert!(w.tile(10, 20).unwrap().fullbright_block());
        assert!(!paint_coat_tile(&mut w, 10, 20, 1), "Glow already set refuses");
        assert!(paint_coat_tile(&mut w, 10, 20, 2), "Echo lands beside Glow");
        assert!(w.invisible_block(10, 20));
        assert!(paint_coat_tile(&mut w, 10, 20, 0), "the clear arm takes both");
        assert!(!w.tile(10, 20).unwrap().fullbright_block());
        assert!(!w.invisible_block(10, 20));
        assert!(!paint_coat_tile(&mut w, 10, 20, 0), "nothing left to clear");
        assert!(!paint_coat_tile(&mut w, 10, 20, 3), "an unknown id refuses");
        // An inactive cell refuses every id.
        assert!(!paint_coat_tile(&mut w, 10, 15, 1));
    }

    /// `WorldGen.paintCoatWall` (`WorldGen.cs:44182-44228`): the wall's Glow
    /// and Echo bits set once and clear together; a wall-less cell refuses.
    #[test]
    fn w4_g79_paint_coat_wall_sets_glow_and_echo_then_clears() {
        let mut w = ground_world();
        let i = w.index(10, 15);
        w.tiles[i].wall = 4;
        assert!(paint_coat_wall(&mut w, 10, 15, 1), "Glow lands on the wall");
        assert!(w.fullbright_wall(10, 15));
        assert!(!paint_coat_wall(&mut w, 10, 15, 1), "Glow already set refuses");
        assert!(paint_coat_wall(&mut w, 10, 15, 2), "Echo lands beside Glow");
        assert!(w.invisible_wall(10, 15));
        assert!(paint_coat_wall(&mut w, 10, 15, 0), "the clear arm takes both");
        assert!(!w.fullbright_wall(10, 15));
        assert!(!w.invisible_wall(10, 15));
        assert!(!paint_coat_wall(&mut w, 10, 15, 0), "nothing left to clear");
        // A wall-less cell refuses.
        assert!(!paint_coat_wall(&mut w, 10, 20, 1));
    }

    // ---- Phase 2 (DW-2.1 / DW-2.4): the item-use placement route -----------

    /// 40x40 world with a stone floor at y = 20 — deep enough inside the world
    /// for `CanPlace`'s 5-tile edge margin (`TileObject.cs:215`).
    pub(super) fn object_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// `object_world` plus a stone ceiling at y = 16 — a 3-tall doorway
    /// opening (rows 17..=19) for the door tests.
    pub(super) fn doorway_world() -> World {
        let mut w = object_world();
        for x in 0..40 {
            let i = w.index(x, 16);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    pub(super) fn set_active(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_candle_toggle_flips_the_lit_frame() {
        // `Player.cs:32378-32390`: `frameX += (frameX > 0) ? -18 : 18`.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 14, 0, 1));
        assert!(place_item_tile(&mut w, 11, 17, 33, 0, 1));
        assert_eq!(w.tile(11, 17).unwrap().frame_x, 0);
        let blocked = |_: i32, _: i32| false;
        let hit = interact_at(&mut w, 11, 17, 1, &blocked, &mut rng());
        assert!(matches!(hit, TileInteraction::CandleToggled));
        assert_eq!(w.tile(11, 17).unwrap().frame_x, 18);
        let hit = interact_at(&mut w, 11, 17, 1, &blocked, &mut rng());
        assert!(matches!(hit, TileInteraction::CandleToggled));
        assert_eq!(w.tile(11, 17).unwrap().frame_x, 0);
    }

    /// W4-f41 — the party monolith dispatch (`Player.cs:32278-32284`): the
    /// kind-literal arm answers `PartyMonolithToggle` for any active 455
    /// cell and writes NO world state (the mech cue, the manual-party
    /// toggle and the achievement edge are the app layer's).
    #[test]
    fn w4_f41_the_party_monolith_dispatches_the_toggle() {
        let mut w = object_world();
        set_active(&mut w, 10, 19, 455);
        let revision = w.revision;
        let blocked = |_: i32, _: i32| false;
        let hit = interact_at(&mut w, 10, 19, 1, &blocked);
        assert!(matches!(hit, TileInteraction::PartyMonolithToggle));
        assert!(w.tile(10, 19).unwrap().active(), "no world write");
        assert_eq!(w.revision, revision);
        // An inactive cell never reaches the chain (`TileInteractionsCheck`
        // gates on active tiles only, `Player.cs:32031-32041`).
        let i = w.index(10, 19);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        let hit = interact_at(&mut w, 10, 19, 1, &blocked);
        assert!(matches!(hit, TileInteraction::None));
    }

    /// W4-f41 final sweep — the monolith's ITEM-USE placement half
    /// (`Item.cs:33176-33188`: 3747 `createTile = 455`, consumable,
    /// placeStyle 0; `TileObjectData.cs:4099-4101`: Style3x3 +
    /// DrawYOffset 2 — origin (1, 2), the SWTSS+Table bottom anchor over
    /// the width, `UsesCustomCanPlace` true with NO hook): the click at
    /// the bottom-centre writes the 3x3 object through the registry
    /// route, and every cell answers the right-click toggle. The break
    /// records are the kill.rs witnesses' half.
    #[test]
    fn w4_f41_the_monolith_item_places_the_3x3_and_toggles() {
        let mut w = object_world();
        assert!(
            crate::tileobject::routes_through_tile_object(455, 0),
            "the CustomPlace route (TileObjectData.cs:3101/4101)"
        );
        // Clicking (11, 19) with origin (1, 2) → the object spans
        // x 10..=12, y 17..=19 (the bottom-centre click, as vanilla).
        assert!(place_item_tile(&mut w, 11, 19, 455, 0, 1));
        for dy in 0..3i32 {
            for dx in 0..3i32 {
                let t = w.tile(10 + dx, 17 + dy).unwrap();
                assert!(t.active(), "cell ({dx},{dy}) written");
                assert_eq!(t.kind, 455);
                assert_eq!(
                    (t.frame_x, t.frame_y),
                    ((dx * 18) as i16, (dy * 18) as i16),
                    "the style-0 frames"
                );
            }
        }
        // Every cell of the placed object answers the toggle dispatch
        // (`Player.cs:32278-32284`'s kind test).
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 12, 17, 1, &blocked),
            TileInteraction::PartyMonolithToggle
        ));
        // A floating click (no solid floor under the object) is refused —
        // the bottom anchor gate, and the world stays unchanged.
        let mut w2 = object_world();
        assert!(!place_item_tile(&mut w2, 11, 10, 455, 0, 1));
        assert!(!w2.tile(11, 10).unwrap().active());
    }

    #[test]
    fn the_chair_sit_decodes_the_seat() {
        // `PlayerSittingHelper.cs:163-182` + `:317-319`: the seat is the
        // chair's bottom cell centre shifted `direction * 6`; the facing is
        // -1 on the left-facing frame (frameX == 0).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 15, 0, -1)); // the left style
        let blocked = |_: i32, _: i32| false;
        // Clicking the chair's bottom cell (y 19; the top is 18).
        let hit = interact_at(&mut w, 10, 19, 1, &blocked, &mut rng());
        let TileInteraction::ChairSit {
            seat_x,
            seat_y,
            direction,
            ..
        } = hit
        else {
            panic!("not a chair sit");
        };
        assert!(
            (seat_x - (10.0 * 16.0 + 8.0 - 6.0)).abs() < 0.01,
            "{seat_x}"
        );
        assert!((seat_y - (19.0 * 16.0 + 16.0)).abs() < 0.01, "{seat_y}");
        assert_eq!(direction, -1);
        // The right-facing chair flips both.
        let mut w2 = object_world();
        assert!(place_item_tile(&mut w2, 10, 19, 15, 0, 1));
        let hit = interact_at(&mut w2, 10, 19, 1, &blocked, &mut rng());
        let TileInteraction::ChairSit {
            seat_x, direction, ..
        } = hit
        else {
            panic!("not a chair sit");
        };
        assert!((seat_x - (10.0 * 16.0 + 8.0 + 6.0)).abs() < 0.01);
        assert_eq!(direction, 1);
    }

    #[test]
    fn the_bed_decodes_the_spawn_cell_and_the_halves() {
        // `Player.cs:32657-32668`: the left-facing bed at (11, 19) spans
        // x 9-12; the bottom half (`frameX % 72 < 36`) targets the spawn cell
        // `x + 2`, the top half sleeps.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 11, 19, 79, 0, -1));
        let blocked = |_: i32, _: i32| false;
        // The bottom-left cell (9, 19): frameX 0 → the bottom side; the spawn
        let hit = interact_at(&mut w, 10, 19, 1, &blocked, &mut rng());
        let TileInteraction::BedInteract {
            set_spawn,
            spawn_x,
            spawn_y,
        } = hit
        else {
            panic!("not a bed interact");
        };
        // The bottom half (`frameX % 72 < 36`): the spawn cell decodes to
        // `myX - frameX/18 + 2 = 12`, the floor row `myY - 1 + 2 = 20`.
        assert!(set_spawn);
        assert_eq!(spawn_x, 12);
        assert_eq!(spawn_y, 20);
        // The sleep half: the RIGHT columns (frameX >= 36 — the vanilla keys
        // the halves on frameX only, `IsHoveringOverABottomSideOfABed`).
        let hit = interact_at(&mut w, 12, 18, 1, &blocked, &mut rng());
        let TileInteraction::BedSleep {
            anchor_x,
            anchor_y,
            direction,
            bed_style,
        } = hit
        else {
            panic!("not a bed sleep");
        };
        // `GetSleepingTargetInfo` (`PlayerSleepingHelper.cs:192-224`): the
        // clicked cell (12, 18) carries frameX 36 (the bed placed at
        // (11, 19) spans columns 10-13) → the bed's leftmost column is
        // `12 - 36 % 72 / 18 = 10`; the unflipped bed faces -1 with the
        // anchor column `10 + 1 = 11`, row `18 + 1`.
        assert_eq!(anchor_x, 184.0);
        assert_eq!(anchor_y, 320.0);
        assert_eq!(direction, -1);
        assert_eq!(bed_style, 0);
    }

    #[test]
    fn the_sleep_decode_follows_the_bed_facing_and_style() {
        // `PlayerSleepingHelper.GetSleepingTargetInfo` (`:192-224`): the
        // unflipped bed (frameX/72 == 0) faces -1 with the anchor column
        // `leftmost + 1`; the flipped one (frameX/72 == 1) faces +1 with
        // `leftmost + 2`; `frameY % 36 != 0` reads the row above and the
        // style is `frameY / 36` (36 alone is style 1 row 0 — the shift
        // needs `frameY = 54`).
        let mut w = object_world();
        for (fx, fy) in [(0i16, 0i16), (72, 0), (0, 54)] {
            let i = w.index(10, 20);
            w.tiles[i] = Tile {
                kind: 79,
                frame_x: fx,
                frame_y: fy,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let (anchor_x, anchor_y, direction, bed_style) =
                sleeping_target_info(&w, 10, 20).expect("the bed decodes");
            match fx {
                0 if fy == 0 => {
                    assert_eq!((anchor_x, anchor_y), (184.0, 352.0));
                    assert_eq!(direction, -1);
                    assert_eq!(bed_style, 0);
                }
                72 => {
                    assert_eq!((anchor_x, anchor_y), (200.0, 352.0));
                    assert_eq!(direction, 1);
                    assert_eq!(bed_style, 0);
                }
                _ => {
                    // frameY 36: the row reads up (`num--`) and the style is 1.
                    assert_eq!((anchor_x, anchor_y), (184.0, 336.0));
                    assert_eq!(direction, -1);
                    assert_eq!(bed_style, 1);
                }
            }
        }
    }

    #[test]
    fn the_sit_decode_covers_the_furniture_family() {
        // `PlayerSittingHelper.GetSittingTargetInfo` (`:144-321`): the
        // per-kind seat anchors and the `seatDownOffset` table.
        let mut w = object_world();
        let put = |w: &mut World, kind: u16, fx: i16, fy: i16| {
            let i = w.index(10, 20);
            w.tiles[i] = Tile {
                kind,
                frame_x: fx,
                frame_y: fy,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        };
        // The left-facing chair (15): direction -1, `num3 = 6`.
        put(&mut w, 15, 0, 0);
        assert_eq!(
            sitting_target_info(&w, 10, 20, 1),
            Some((162.0, 352.0, -1, (0.0, 0.0)))
        );
        // The chair's style 27 (`frameY / 40 == 27`): `seatDownOffset.Y = 4`.
        put(&mut w, 15, 0, 27 * 40);
        assert_eq!(
            sitting_target_info(&w, 10, 20, 1),
            Some((162.0, 352.0, -1, (0.0, 4.0)))
        );
        // The toilet (497) shares the chair arm.
        put(&mut w, 497, 0, 0);
        assert_eq!(
            sitting_target_info(&w, 10, 20, 1),
            Some((162.0, 352.0, -1, (0.0, 0.0)))
        );
        // The throne (102): `frameX / 18 == 0` → `num++`, `frameY / 18 == 0`
        // → `num2 += 2`, the facing follows the player, `num3 = 0`.
        put(&mut w, 102, 0, 0);
        assert_eq!(
            sitting_target_info(&w, 10, 20, 1),
            Some((184.0, 384.0, 1, (0.0, 0.0)))
        );
        // The picnic table (487): `frameX % 72 / 18 == 0` → facing -1,
        // `seatDownOffset.Y--`.
        put(&mut w, 487, 0, 0);
        assert_eq!(
            sitting_target_info(&w, 10, 20, 1),
            Some((168.0, 352.0, -1, (0.0, -1.0)))
        );
        // The bench (89), style 0 (`frameX / 54 == 0`): the side pick with
        // the player facing -1 takes `vector`, `+= zero2` (1, 0), and the
        // position shift `zero` (-1, 0).
        put(&mut w, 89, 0, 0);
        assert_eq!(
            sitting_target_info(&w, 10, 20, -1),
            Some((167.0, 352.0, -1, (-3.0, 1.0)))
        );
        // The bench's style 24: `vector`/`vector2` collapse to (0, -4).
        put(&mut w, 89, 24 * 54, 0);
        assert_eq!(
            sitting_target_info(&w, 10, 20, -1),
            Some((167.0, 352.0, -1, (1.0, -4.0)))
        );
    }

    #[test]
    fn the_clock_reads_the_time() {
        // `Player.cs:32725-32759`: noon (time 27000, day) → 12:00 PM.
        let mut w = object_world();
        w.time = 27000.0;
        w.day_time = true;
        assert!(place_item_tile(&mut w, 10, 19, 104, 0, 1));
        let blocked = |_: i32, _: i32| false;
        let hit = interact_at(&mut w, 10, 19, 1, &blocked, &mut rng());
        let TileInteraction::ClockRead { text } = hit else {
            panic!("not a clock read");
        };
        assert_eq!(text, "12:00 PM");
    }

    #[test]
    fn test_DW_2_4_door_click_points() {
        // Targeting the middle cell of the opening resolves the mid
        // alternate; the whole 3-tall door is written with column frames
        // (0, 18, 36 — `coordinate_heights` 16 + padding 2).
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
        for (y, frame_y) in [(17i32, 0i16), (18, 18), (19, 36)] {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 10, "door cell {y}");
            assert_eq!(t.frame_y, frame_y, "door frame row {y}");
        }
        // Targeting the bottom cell resolves the bottom-origin alternate.
        let mut w2 = doorway_world();
        assert!(place_item_tile(&mut w2, 10, 19, 10, 0, 1));
        for y in 17..=19 {
            assert!(w2.tile(10, y).unwrap().active(), "door cell {y}");
        }
    }

    // ---- Phase 3 (DW-3.1 / DW-3.3): the KillTile destruction cascade -------

    /// Writes a synthetic object cell with explicit frames (the DW-3.3
    /// sanctioned construction for states no item route produces).
    pub(super) fn set_object_cell(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        // `Main.tileFrameImportant[type]` is a static every vanilla world
        // honours — `place` mirrors it into `world.frame_important` (Phase 1
        // DW-1.3) and a loaded save carries it — so synthetic states must set
        // it too, or `reframe_area` mangles the frames the cascade reads.
        if w.frame_important.len() <= kind as usize {
            w.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        w.frame_important[kind as usize] = true;
    }

    /// Converts a freshly placed closed door into the open state with the
    /// frames `WorldGen.OpenDoor` writes (:37727-37770): `frameX = num2 +
    /// dx*18` (`num2` 0 for the right leaf, 36 for the left), `frameY =
    /// dy*18` for style row 0.
    pub(super) fn open_the_door(w: &mut World, left_leaf: bool) -> Vec<(i32, i32)> {
        let (x0, num2) = if left_leaf { (9, 36) } else { (10, 0) };
        let mut cells = Vec::new();
        for dy in 0..3i32 {
            for dx in 0..2i32 {
                set_object_cell(
                    w,
                    x0 + dx,
                    17 + dy,
                    11,
                    num2 + (dx * 18) as i16,
                    (dy * 18) as i16,
                );
                cells.push((x0 + dx, 17 + dy));
            }
        }
        cells
    }

    #[test]
    fn test_object_tiles_at_resolves_origins_through_the_frames() {
        // The anchor lookup generalizes the C# per-type frame math
        // (`WorldGen.cs:57929-57937` etc.): any cell of the object resolves
        // the same top-left origin.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 14, 19, 21, 0, 1)); // chest origin (14, 18)
        for cell in [(14i32, 18i32), (15, 18), (14, 19), (15, 19)] {
            let tiles = crate::tileobject::object_tiles_at(&w, cell.0, cell.1).unwrap();
            assert_eq!(tiles.origin.x, 14, "chest cell {cell:?}");
            assert_eq!(tiles.origin.y, 18, "chest cell {cell:?}");
            assert_eq!(tiles.def.kind, 21);
            let mut got: Vec<(i32, i32)> = tiles.cells().collect();
            got.sort_unstable();
            assert_eq!(
                got,
                vec![(14, 18), (14, 19), (15, 18), (15, 19)],
                "cells() of the 2x2 chest"
            );
        }
        // Inactive and unregistered cells resolve to nothing.
        assert!(crate::tileobject::object_tiles_at(&w, 20, 19).is_none());
        assert!(crate::tileobject::object_tiles_at(&w, 200, 200).is_none());
        // The open door's left leaf (frameX 54): dx = 54/18 % 2 = 1 → origin
        // one column left — matches the C# `frameX % 72` case 54 mapping
        // (:86366-86368).
        let mut w2 = doorway_world();
        assert!(place_item_tile(&mut w2, 10, 18, 10, 0, 1));
        open_the_door(&mut w2, true);
        let tiles = crate::tileobject::object_tiles_at(&w2, 10, 18).unwrap();
        assert_eq!(tiles.def.kind, 11);
        assert_eq!((tiles.origin.x, tiles.origin.y), (9, 17));
    }

    /// A fresh closed door at column 10 (rows 17-19) through the placement
    /// route — the frames `open_door` decodes: `frameX` 0, `frameY`
    /// 0/18/36.
    fn closed_door_world() -> World {
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
        w
    }

    fn cells_of(w: &World, cells: &[(i32, i32)]) -> Vec<(bool, u16, i16, i16)> {
        cells
            .iter()
            .map(|&(x, y)| {
                let t = w.tile(x, y).unwrap();
                (t.active(), t.kind, t.frame_x, t.frame_y)
            })
            .collect()
    }

    fn no_blocker(_: i32, _: i32) -> bool {
        false
    }

    /// W4-g4 — `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`): the
    /// zombie door-bash drain's gate-388 open. A closed tall gate (388, the
    /// 1×5 style-0 object) shifts to its open kind (389) row by row; the
    /// frames keep, the occupancy gate refuses a blocked cell, and a
    /// non-gate cell refuses outright.
    #[test]
    fn w4_g4_shift_tall_gate_opens_the_closed_gate() {
        let mut w = doorway_world();
        // The closed tall gate at column 10, rows 14..18 (the 1×5 style-0
        // object): frameX 0, frameY the coordinate-heights prefix
        // (0/18/34/50/66 — the def's [18,16,16,16,18] rows). Set directly so
        // the witness pins the SHIFT, not the placement anchors.
        for (k, y) in (14..=18).enumerate() {
            set_active(&mut w, 10, y, 388);
            let fy = [0i16, 18, 34, 50, 66][k];
            w.tile_mut(10, y).unwrap().frame_y = fy;
        }
        let mut drops = Vec::new();
        assert!(shift_tall_gate(&mut w, 10, 18, false, false, &no_blocker, &mut drops));
    /// W4-g4 — `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`): the
    /// zombie door-bash drain's gate-388 open, re-pointed at the c42
    /// canonical body (research/95 §3 witness union — the richest unit
    /// pin: open/close round-trip + blocked refusal + forced bypass +
    /// non-gate refusal). The 1×5 style-0 object at column 10, rows
    /// 14..18; frameY on the real placement grid (the [18,16,16,16,18]
    /// coordinate-heights prefix + 2px padding → 0/20/38/56/74).
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_g4_shift_tall_gate_opens_the_closed_gate() {
        let mut w = doorway_world();
        for (k, y) in (14..=18).enumerate() {
            set_active(&mut w, 10, y, 388);
            let fy = [0i16, 20, 38, 56, 74][k];
            w.tile_mut(10, y).unwrap().frame_y = fy;
        }
        let mut drops = Vec::new();
        let mut cues = Vec::new();
        assert!(shift_tall_gate(
            &mut w, 10, 18, false, false, &no_blocker, &mut drops, None, &mut cues
        ));
        for y in 14..=18 {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 389, "gate row {y} opens to 389");
        }
        // The open gate shifts back to closed (388) with `closing: true`.
        assert!(shift_tall_gate(&mut w, 10, 18, true, false, &no_blocker, &mut drops));
        // The cue queues at the ADJUSTED top centre (`:51762`).
        assert_eq!(cues, vec![(true, 10 * 16 + 16, 14 * 16 + 16)]);
        // The open gate shifts back to closed (388) with `closing: true`.
        assert!(shift_tall_gate(
            &mut w, 10, 18, true, false, &no_blocker, &mut drops, None, &mut cues
        ));
        for y in 14..=18 {
            assert_eq!(w.tile(10, y).unwrap().kind, 388, "gate row {y} closes");
        }
        // A blocked cell refuses the shift (the occupancy gate).
        let blocked = |_: i32, _: i32| true;
        assert!(!shift_tall_gate(&mut w, 10, 18, false, false, &blocked, &mut drops));
        assert_eq!(w.tile(10, 18).unwrap().kind, 388, "the blocked shift leaves the gate");
        // `forced` skips the occupancy gate.
        assert!(shift_tall_gate(&mut w, 10, 18, false, true, &blocked, &mut drops));
        assert_eq!(w.tile(10, 18).unwrap().kind, 389, "the forced shift opens");
        // A non-gate cell refuses outright.
        assert!(!shift_tall_gate(&mut w, 12, 18, false, false, &no_blocker, &mut drops));
        assert!(!shift_tall_gate(
            &mut w, 10, 18, false, false, &blocked, &mut drops, None, &mut cues
        ));
        assert_eq!(w.tile(10, 18).unwrap().kind, 388, "the blocked shift leaves the gate");
        // `forced` skips the occupancy gate.
        assert!(shift_tall_gate(
            &mut w, 10, 18, false, true, &blocked, &mut drops, None, &mut cues
        ));
        assert_eq!(w.tile(10, 18).unwrap().kind, 389, "the forced shift opens");
        // A non-gate cell refuses outright.
        assert!(!shift_tall_gate(
            &mut w, 12, 18, false, false, &no_blocker, &mut drops, None, &mut cues
        ));
    }

    #[test]
    fn test_DW_4_2_open_door_right_leaf_frames() {
        // `WorldGen.OpenDoor` direction +1 (`WorldGen.cs:37733-37776`): the
        // hinge column stays at i with `frameX = num2` (0) and the leaf opens
        // into i+1 with `frameX = num2 + 18`; `frameY` rows 0/18/36.
        let mut w = closed_door_world();
        w.tile_mut(10, 17).unwrap().set_paint(3);
        w.tile_mut(10, 18).unwrap().set_paint(5);
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, 1, &mut drops, None));
        assert!(open_door(&mut w, 10, 18, 1, &mut drops, &mut rng()));
        assert!(drops.is_empty());
        let row_paint = [3u8, 5, 0]; // cache rows (:37676-37690)
        for (k, y, fy) in [(0usize, 17i32, 0i16), (1, 18, 18), (2, 19, 36)] {
            let hinge = w.tile(10, y).unwrap();
            assert!(hinge.active() && hinge.kind == 11, "hinge row {y}");
            assert_eq!((hinge.frame_x, hinge.frame_y), (0, fy), "hinge row {y}");
            // `UseBlockColors(cache)`: each written row takes its own closed
            // door row's paint.
            assert_eq!(hinge.paint(), row_paint[k], "hinge paint row {y}");
            let leaf = w.tile(11, y).unwrap();
            assert!(leaf.active() && leaf.kind == 11, "leaf row {y}");
            assert_eq!((leaf.frame_x, leaf.frame_y), (18, fy), "leaf row {y}");
            assert_eq!(leaf.paint(), row_paint[k], "leaf paint row {y}");
        }
        // Nothing else changed: the closed-door cells were overwritten, not
        // killed (no KillTile on the door body in 1.4.5.8).
        assert!(!w.tile(12, 18).unwrap().active());
    }

    #[test]
    fn test_DW_4_2_open_door_left_leaf_frames() {
        // Direction -1 (:37698-37702): the door leans left — hinge column
        // i-1 gets `frameX = 36`, column i `frameX = 54`.
        let mut w = closed_door_world();
        w.tile_mut(10, 17).unwrap().set_paint(3);
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, -1, &mut drops, None));
        assert!(open_door(&mut w, 10, 18, -1, &mut drops, &mut rng()));
        for (y, fy) in [(17i32, 0i16), (18, 18), (19, 36)] {
            let hinge = w.tile(9, y).unwrap();
            assert!(hinge.active() && hinge.kind == 11, "hinge row {y}");
            assert_eq!((hinge.frame_x, hinge.frame_y), (36, fy), "hinge row {y}");
            let leaf = w.tile(10, y).unwrap();
            assert!(leaf.active() && leaf.kind == 11, "leaf row {y}");
            assert_eq!((leaf.frame_x, leaf.frame_y), (54, fy), "leaf row {y}");
        }
        // The paint caches are read at column `i` before `num6` is
        // reassigned (:37676-37690), so direction -1 carries the closed
        // door's paint onto the new hinge column too.
        assert_eq!(w.tile(10, 17).unwrap().paint(), 3);
        assert_eq!(w.tile(9, 17).unwrap().paint(), 3);
    }

    #[test]
    fn test_DW_4_2_close_door_hinge_from_both_leaves() {
        // Right leaf (`frameX % 72 == 0`/`18`, `:31834-31851`): the hinge is
        // the pair's left column and becomes the closed door; the other
        // column deactivates. `frameY` is deliberately not rewritten — the
        // style-0 open rows equal the closed rows, so the frames come out as
        // a fresh placement's.
        for click in [(10i32, 18i32), (11, 18)] {
            let mut w = doorway_world();
            let cells = open_the_door(&mut w, false);
            let mut drops = Vec::new();
            assert!(close_door(
                &mut w,
                click.0,
                click.1,
                false,
                &no_blocker,
                &mut drops,
                None
                &mut rng()
            ));
            assert!(drops.is_empty());
            let written: Vec<_> = (0..3)
                .flat_map(|dy| [(10, 17 + dy), (11, 17 + dy)])
                .collect();
            let got = cells_of(&w, &written);
            for (k, (active, kind, fx, fy)) in got.iter().enumerate() {
                if k % 2 == 0 {
                    assert!(*active && *kind == 10, "hinge cell {k}");
                    assert_eq!(*fx, 0, "hinge cell {k}"); // genRand variant 0 (see the port note)
                    // `frameY` is not rewritten (:31880-31886): the style-0
                    // open rows 0/18/36 stay.
                    assert_eq!(*fy, (k / 2) as i16 * 18, "hinge cell {k}");
                } else {
                    assert!(!*active, "leaf cell {k}");
                }
            }
            assert_eq!(cells.len(), 6); // the state the close consumed
        }
        // Left leaf (`frameX % 72 == 36`/`54`): hinge is the pair's RIGHT
        // column (`num2 = i` for 54, `i + 1` for 36).
        let mut w = doorway_world();
        open_the_door(&mut w, true);
        let mut drops = Vec::new();
        assert!(close_door(&mut w, 10, 18, false, &no_blocker, &mut drops, None));
        assert!(close_door(&mut w, 10, 18, false, &no_blocker, &mut drops, &mut rng()));
        for dy in 0..3i32 {
            let hinge = w.tile(10, 17 + dy).unwrap();
            assert!(hinge.active() && hinge.kind == 10, "left hinge row {dy}");
            assert_eq!(hinge.frame_x, 0);
            assert!(!w.tile(9, 17 + dy).unwrap().active(), "left leaf row {dy}");
        }
    }

    #[test]
    fn test_DW_4_2_door_toggle_round_trip() {
        // Closed → open → close → open, all through the dispatcher — the
        // repeated-click (toggle) edge case; frames identical each open.
        let mut w = closed_door_world();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        let expected = [(0i16, 0i16), (18, 0), (0, 18), (18, 18), (0, 36), (18, 36)];
        for round in 0..2 {
            match interact_at(&mut w, 10, 18, 1, &blocked, &mut rng()) {
                TileInteraction::DoorOpened { object_drops } => {
                    assert!(object_drops.is_empty());
                }
                other => panic!("round {round}: expected DoorOpened, got {other:?}"),
            }
            let mut got = Vec::new();
            for dy in 0..3i32 {
                for dx in 0..2i32 {
                    let t = w.tile(10 + dx, 17 + dy).unwrap();
                    got.push((t.frame_x, t.frame_y));
                }
            }
            assert_eq!(got, expected, "round {round}");
            if round == 0 {
                match interact_at(&mut w, 11, 18, 1, &blocked, &mut rng()) {
                    TileInteraction::DoorClosed { object_drops } => {
                        assert!(object_drops.is_empty());
                    }
                    other => panic!("expected DoorClosed, got {other:?}"),
                }
                assert!(w.tile(10, 18).unwrap().active() && w.tile(10, 18).unwrap().kind == 10);
            }
        }
    }

    #[test]
    fn test_DW_4_2_open_door_blocked_by_solid() {
        // A real blocker in the opening column refuses the open and leaves
        // the door untouched (:37702-37710).
        let mut w = closed_door_world();
        for y in 17..=19 {
            set_active(&mut w, 11, y, 1);
        }
        let mut drops = Vec::new();
        assert!(!open_door(&mut w, 10, 18, 1, &mut drops, None));
        assert!(!open_door(&mut w, 10, 18, 1, &mut drops, &mut rng()));
        assert!(drops.is_empty());
        for y in 17..=19 {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 10, "door row {y}");
            assert_eq!(
                (t.frame_x, t.frame_y),
                (0, (y - 17) as i16 * 18),
                "door row {y}"
            );
        }
        // The opposite direction's column is free — the open succeeds there
        // (the dispatcher-level retry is DW-4.1's test).
        assert!(open_door(&mut w, 10, 18, -1, &mut drops, None));
        assert!(open_door(&mut w, 10, 18, -1, &mut drops, &mut rng()));
    }

    #[test]
    fn test_DW_4_2_open_door_kills_cut_tiles_in_the_way() {
        // Cut tiles in the opening column die with a full `KillTile`
        // (:37711-37718) and the open proceeds.
        let mut w = closed_door_world();
        set_active(&mut w, 11, 18, 3); // kind 3 ∈ Main.tileCut
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, 1, &mut drops, None));
        assert!(open_door(&mut w, 10, 18, 1, &mut drops, &mut rng()));
        assert!(drops.is_empty()); // a cut plant carries no object drop
        let t = w.tile(11, 18).unwrap();
        assert!(t.active() && t.kind == 11);
    }

    #[test]
    fn test_DW_4_2_close_door_blocked_by_entity() {
        // `Collision.EmptyTile(ignoreTiles: true)` (`Collision.cs:1359`):
        // entities refuse the close, `forced` (the wired caller's flag)
        // ignores them.
        let mut w = doorway_world();
        open_the_door(&mut w, false);
        let blocked = |x: i32, y: i32| x == 10 && y == 18;
        let mut drops = Vec::new();
        assert!(!close_door(&mut w, 10, 18, false, &blocked, &mut drops, None));
        assert!(w.tile(10, 18).unwrap().kind == 11, "door stays open");
        assert!(close_door(&mut w, 10, 18, true, &blocked, &mut drops, None));
        assert!(!close_door(&mut w, 10, 18, false, &blocked, &mut drops, &mut rng()));
        assert!(w.tile(10, 18).unwrap().kind == 11, "door stays open");
        assert!(close_door(&mut w, 10, 18, true, &blocked, &mut drops, &mut rng()));
        assert!(w.tile(10, 18).unwrap().kind == 10, "forced closes");
    }

    #[test]
    fn test_DW_4_2_wrong_kind_returns_false() {
        // Cross-kind calls refuse without touching the world (:37645/:31818).
        let mut w = closed_door_world();
        let mut drops = Vec::new();
        assert!(!open_door(&mut w, 40, 40, 1, &mut drops, None)); // out of bounds
        assert!(!close_door(&mut w, 10, 18, false, &no_blocker, &mut drops, None));
        let mut w2 = doorway_world();
        open_the_door(&mut w2, false);
        assert!(!open_door(&mut w2, 10, 18, 1, &mut drops, None));
        assert!(!open_door(&mut w, 40, 40, 1, &mut drops, &mut rng())); // out of bounds
        assert!(!close_door(&mut w, 10, 18, false, &no_blocker, &mut drops, &mut rng()));
        let mut w2 = doorway_world();
        open_the_door(&mut w2, false);
        assert!(!open_door(&mut w2, 10, 18, 1, &mut drops, &mut rng()));
    }

    #[test]
    fn test_DW_4_2_style_offset_frames() {
        // A style-1 closed door (`frameY` 54/72/90, the C# `num4`/`num8`
        // decode, :37657-37731): the open door lands in sheet row 54.
        let mut w = doorway_world();
        for (dy, fy) in [(0i32, 54i16), (1, 72), (2, 90)] {
            set_object_cell(&mut w, 10, 17 + dy, 10, 0, fy);
        }
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 17, 1, &mut drops, None));
        assert!(open_door(&mut w, 10, 17, 1, &mut drops, &mut rng()));
        for dy in 0..3i32 {
            let t = w.tile(10, 17 + dy).unwrap();
            assert_eq!(t.frame_y, 54 + dy as i16 * 18, "style row {dy}");
            assert_eq!(t.frame_x, 0);
            let leaf = w.tile(11, 17 + dy).unwrap();
            assert_eq!(leaf.frame_x, 18);
        }
    }

    #[test]
    fn test_DW_4_2_partial_door_validate_cleanup() {
        // `ValidateTileSquareIsActiveAndOfType` (:37869-37917): a door with a
        // broken cell has its remaining door cells killed (full `KillTile` —
        // the cascade records the object destruction) and the open refused.
        let mut w = closed_door_world();
        let broken = w.index(10, 19);
        w.tiles[broken] = crate::tile::Tile::default();
        let mut drops = Vec::new();
        assert!(!open_door(&mut w, 10, 18, 1, &mut drops, None));
        assert!(!open_door(&mut w, 10, 18, 1, &mut drops, &mut rng()));
        for y in 17..=19 {
            assert!(!w.tile(10, y).unwrap().active(), "row {y} cleaned up");
        }
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].tile_kind, 10);
        // The drop record sits at the `Check*` visit cell the cascade ran
        // from (Phase 3's `square_tile_frame` order).
        assert_eq!((drops[0].x, drops[0].y), (10, 18));
    }

    #[test]
    fn test_DW_4_1_interact_at_ignores_non_interaction_tiles() {
        // The fall-through contract: a stone tile, an air cell and a chest
        // (registry kind with `Interaction::None`). The C# tail
        // (`Player.cs:33279-33284`) hands every reached tile to the crafting
        // station probe — the active tiles answer `StationInteract`, the air
        // cell never reaches the dispatcher (`None`); the app layer resolves
        // whether the probed kind is a station.
        let mut w = closed_door_world();
        assert!(place_item_tile(&mut w, 14, 19, 21, 0, 1));
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 20, 1, &blocked, &mut rng()),
            TileInteraction::StationInteract { kind: 1 }
        ));
        assert!(matches!(
            interact_at(&mut w, 12, 18, 1, &blocked, &mut rng()),
            TileInteraction::None
        ));
        assert!(matches!(
            interact_at(&mut w, 14, 19, 1, &blocked, &mut rng()),
            TileInteraction::StationInteract { kind: 21 }
        ));
    }

    /// A 40x40 world with a placed 3x3 extractinator at origin (9, 17)
    /// (tile 219) — the registry route, so the frames come from
    /// `TileObject.Place`.
    fn extractinator_world() -> World {
        let mut w = World::new("e".into(), 40, 40);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        let def = crate::tileobject::def_for_tile(219).unwrap();
        let inst = crate::tileobject::can_place(
            &w,
            10,
            19,
            def,
            0,
            1,
            None,
            &mut crate::unified_random::UnifiedRandom::new(0),
            None,
        )
        .unwrap();
        assert!(crate::tileobject::place(&mut w, &inst));
        // Clear the floor cells the object now covers is NOT done by Place; the
        // object's own cells are what the dispatch reads.
        w
    }

    #[test]
    fn test_M7_EXTRACT_1_interact_at_dispatches_the_extractinators() {
        // M7-EXTRACT-1: both extractinator tiles (219/642) answer the dispatch
        // (`Player.cs:41669`/`:41691`'s kind literals) with the new bare
        // variant — no world change; the held-item semantics are the app's.
        let mut w = extractinator_world();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 11, 18, 1, &blocked, &mut rng()),
            TileInteraction::Extractinator
        ));
        // The cells keep their frames (the extraction only consumes the item).
        let t = w.tile(11, 18).unwrap();
        // (11, 18) is the object's middle row, right column of the (9, 17)
        // origin — frames (36, 18), untouched by the dispatch.
        assert_eq!((t.kind, t.frame_x, t.frame_y), (219, 36, 18));
        // An unrelated tile (the stone floor below) is still `None` (the
        // registry gate) — but every cell of the object answers Extractinator,
        // the way the C# kind test does.
        assert!(matches!(
            interact_at(&mut w, 9, 17, 1, &blocked, &mut rng()),
            TileInteraction::Extractinator
        ));
        // The non-extractinator cell reaches the C#'s station tail
        // (`Player.cs:33279`).
        assert!(matches!(
            interact_at(&mut w, 5, 20, 1, &blocked, &mut rng()),
            TileInteraction::StationInteract { .. }
        ));
    }

    // W4-c39 exec (research/114 §1h): the cake-dispatch witness DROPPED
    // with the `CakeEat` route — f34's `StationBuff` dispatch witness
    // (`621 => Some((192, 7_200))`) covers the surviving arm.

    #[test]
    fn test_DW_4_1_interact_at_dispatches_door_kinds() {
        let mut w = closed_door_world();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 18, 1, &blocked, &mut rng()),
            TileInteraction::DoorOpened { .. }
        ));
        assert!(w.tile(10, 17).unwrap().kind == 11, "opened");
        assert!(matches!(
            interact_at(&mut w, 10, 18, 1, &blocked, &mut rng()),
            TileInteraction::DoorClosed { .. }
        ));
        assert!(w.tile(10, 17).unwrap().kind == 10, "closed");
    }

    #[test]
    fn test_DW_4_1_interact_at_retries_opposite_direction() {
        // `Player.cs:32841-32852`: open with the facing; while the tile is
        // still a closed door, retry with the opposite direction.
        let mut w = closed_door_world();
        for y in 17..=19 {
            set_active(&mut w, 11, y, 1);
        }
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 18, 1, &blocked, &mut rng()) {
            TileInteraction::DoorOpened { object_drops } => assert!(object_drops.is_empty()),
            other => panic!("expected DoorOpened via retry, got {other:?}"),
        }
        for dy in 0..3i32 {
            let hinge = w.tile(9, 17 + dy).unwrap();
            assert!(hinge.active() && hinge.kind == 11);
            assert_eq!(hinge.frame_x, 36);
        }
    }

    #[test]
    fn test_DW_4_1_interact_at_blocked_door_consumes_without_drops() {
        // Both directions blocked: the click is still consumed (vanilla sets
        // `flag2` before attempting, `Player.cs:32836`) — `DoorBlocked` with
        // no drops and no world change.
        let mut w = closed_door_world();
        for y in 17..=19 {
            set_active(&mut w, 9, y, 1);
            set_active(&mut w, 11, y, 1);
        }
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 18, 1, &blocked, &mut rng()) {
            TileInteraction::DoorBlocked { object_drops } => assert!(object_drops.is_empty()),
            other => panic!("expected DoorBlocked, got {other:?}"),
        }
        assert!(w.tile(10, 17).unwrap().kind == 10, "door intact");
    }

    #[test]
    fn test_DW_4_3_open_door_survives_missing_wall() {
        // Vanilla's `OpenDoor` has no wall/support check — a door whose wall
        // was mined away still opens (the closed-door square check is
        // type-only, `WorldGen.cs:37672-37675`).
        let mut w = closed_door_world();
        for y in 17..=19 {
            let t = w.tile_mut(10, y).unwrap();
            t.wall = 0; // no wall behind the door anywhere
        }
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, 1, &mut drops, None));
        assert!(open_door(&mut w, 10, 18, 1, &mut drops, &mut rng()));
        assert!(w.tile(10, 17).unwrap().kind == 11);
    }

    #[test]
    fn test_DW_4_3_broken_door_interaction_cleans_up_and_blocks() {
        // Right-clicking a partially-broken door through the dispatcher: the
        // validate cleanup kills the rest (drop recorded) and the click is
        // consumed without a cue (`DoorBlocked`).
        let mut w = closed_door_world();
        let broken = w.index(10, 19);
        w.tiles[broken] = crate::tile::Tile::default();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 18, 1, &blocked, &mut rng()) {
            TileInteraction::DoorBlocked { object_drops } => {
                assert_eq!(object_drops.len(), 1);
                assert_eq!(object_drops[0].tile_kind, 10);
            }
            other => panic!("expected DoorBlocked, got {other:?}"),
        }
    }

    // ---- M1.4-TO-4 phase 1 (DW-1.1 / DW-1.3 / DW-1.5): the seed and
    // planter-box placement paths -------------------------------------------

    /// The stone floor of `object_world` repainted to `kind` — the soil row
    /// `PlaceAlch`/the planter-box case judge.
    pub(super) fn soil_world(kind: u16) -> World {
        let mut w = object_world();
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i].kind = kind;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    pub(super) fn set_liquid(w: &mut World, x: i32, y: i32, amount: u8, lava: bool) {
        let ti = w.index(x, y);
        let t = &mut w.tiles[ti];
        t.liquid = amount;
        if lava {
            t.flags |= crate::tile::FLAG_LIQUID_LAVA;
        }
    }

    /// The C# soil table of `PlaceAlch` (`WorldGen.cs:45393-45441`) — style →
    /// the soils it accepts (78/380 included in every style).
    pub(super) const ALCH_SOIL_TABLE: [(i32, &[u16]); 7] = [
        (0, &[2, 477, 492, 78, 380, 109]),
        (1, &[60, 78, 380]),
        (2, &[0, 59, 78, 380]),
        (3, &[661, 662, 203, 199, 23, 25, 78, 380]),
        (4, &[53, 78, 380, 116]),
        (5, &[57, 633, 78, 380]),
        (6, &[78, 380, 147, 161, 163, 164, 200]),
    ];

    #[test]
    fn test_store_1_bank_tiles_resolve_the_interaction() {
        // `Player.cs:33056-33138`'s num73 switch: 29 -> -2, 97 -> -3,
        // 463 -> -4, 491 -> -5. The port's interact_at returns BankOpen for
        // each literal (the defs carry Interaction::Bank too).
        for (kind, chest) in [(29, -2), (97, -3), (463, -4), (491, -5)] {
            let mut w = soil_world(2);
            set_object_cell(&mut w, 10, 20, kind, 0, 0);
            let hit = interact_at(&mut w, 10, 20, 1, &|_, _| false, &mut rng());
            assert!(
                matches!(hit, TileInteraction::BankOpen { bank } if bank == chest),
                "tile {kind} opens chest {chest}"
            );
        }
        // An ordinary tile: the C# station tail answers (`Player.cs:33279`).
        let mut w = soil_world(2);
        let hit = interact_at(&mut w, 10, 20, 1, &|_, _| false, &mut rng());
        assert!(matches!(hit, TileInteraction::StationInteract { .. }));
    }

    #[test]
    fn w4_e28_the_station_buff_arms_dispatch_the_grants() {
        // W4-e28 — `Player.cs:32594-32599`: tile 125 (Crystal Ball) grants
        // buff 29 for 108000 ticks; `:32630-32635`: tile 287 (Ammo Box)
        // grants buff 93 for 108000. Both consume the click; the buff slots
        // and the cue are the app's, the dispatch only carries the row.
        for (kind, buff) in [(125u16, 29), (287u16, 93)] {
            let mut w = object_world();
            set_active(&mut w, 10, 19, kind);
            let blocked = no_blocker as fn(i32, i32) -> bool;
            match interact_at(&mut w, 10, 19, 1, &blocked) {
                TileInteraction::StationBuff {
                    buff_type,
                    buff_time,
                } => {
                    assert_eq!(buff_type, buff, "tile {kind}");
                    assert_eq!(buff_time, 108000, "the 30-minute grant");
                }
                other => panic!("expected StationBuff for {kind}, got {other:?}"),
            }
            // No world write: the cell keeps its kind and frame.
            let t = w.tile(10, 19).unwrap();
            assert_eq!((t.kind, t.frame_x, t.frame_y), (kind, 0, 0));
        }
        // The neighbouring kinds keep the station tail (no over-capture).
        let mut w = object_world();
        set_active(&mut w, 10, 19, 126);
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &no_blocker),
            TileInteraction::StationInteract { .. }
        ));
    }

    #[test]
    fn w4_f34_the_five_neighbour_station_arms_dispatch_the_grants() {
        // W4-f34 — the census's remaining `TileInteractionsUse` station
        // buffs, verbatim rows (`Player.cs:32600-32629`): 621 Slice of Cake
        // → buff 192 for 7200 ticks + Item2 (:32600-32605), 464 War Table →
        // 348 + Item4 (:32606-32611), 699 Dead Cells Potion Station → 366 +
        // Item179 (:32612-32617), 377 Sharpening Station → 159 + Item37
        // (:32618-32623), 354 Bewitching Table → 150 + Item4
        // (:32624-32629). All consume the click; no world write.
        for (kind, buff, time) in [
            (621u16, 192, 7_200),
            (464, 348, 108_000),
            (699, 366, 108_000),
            (377, 159, 108_000),
            (354, 150, 108_000),
        ] {
            let mut w = object_world();
            set_active(&mut w, 10, 19, kind);
            let blocked = no_blocker as fn(i32, i32) -> bool;
            match interact_at(&mut w, 10, 19, 1, &blocked) {
                TileInteraction::StationBuff {
                    buff_type,
                    buff_time,
                } => {
                    assert_eq!(buff_type, buff, "tile {kind}");
                    assert_eq!(buff_time, time, "tile {kind}'s grant duration");
                }
                other => panic!("expected StationBuff for {kind}, got {other:?}"),
            }
            // No world write: the cell keeps its kind and frame.
            let t = w.tile(10, 19).unwrap();
            assert_eq!((t.kind, t.frame_x, t.frame_y), (kind, 0, 0));
        }
        // The inactive cell of a station kind is invisible to the use path
        // (`TileInteractionsCheck`, `Player.cs:32031-32041`).
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 699,
            ..Default::default()
        };
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &no_blocker),
            TileInteraction::None
        ));
    }

    // ---- Phase 2 (M1.4-BG-21): the dial triggers (356/663) -----------------

    #[test]
    fn dw_2_1_lmb_sundial_arms_fast_forward_with_cooldown() {
        // `Player.cs:32636-32645`: the click on tile 356 fires
        // `Main.Sundialing()` (SP branch, `Main.cs:6178-6180`) — the
        // FF-to-dawn flag arms and the 8-day cooldown starts.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()) {
            TileInteraction::DialStarted { sundial } => assert!(sundial),
            other => panic!("expected DialStarted, got {other:?}"),
        }
        assert!(w.fast_forward_to_dawn);
        assert!(!w.fast_forward_to_dusk);
        assert_eq!(w.sundial_cooldown, 8);
        assert_eq!(w.moondial_cooldown, 0);
    }

    #[test]
    fn dw_2_2_retrigger_while_dialing_is_consumed_without_effect() {
        // `Player.cs:32637-32638`: the second click while
        // `fastForwardTimeToDawn` runs is still consumed (`flag2`) but the
        // gate refuses — no state change, no cue.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()),
            TileInteraction::DialStarted { .. }
        ));
        match interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()) {
            TileInteraction::DialRefused => {}
            other => panic!("expected DialRefused, got {other:?}"),
        }
        assert!(w.fast_forward_to_dawn, "the first dial still runs");
        assert_eq!(w.sundial_cooldown, 8, "the cooldown did not restart");
    }

    #[test]
    fn dw_2_3_cooldown_blocks_the_dial_without_cue() {
        // `Player.cs:32638`: `Main.sundialCooldown == 0` is the gate's other
        // conjunct — a running cooldown refuses the dial (consumed silently,
        // no cue since `SoundID.Item4` sits behind the gate).
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        w.sundial_cooldown = 3;
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()) {
            TileInteraction::DialRefused => {}
            other => panic!("expected DialRefused, got {other:?}"),
        }
        assert!(!w.fast_forward_to_dawn);
        assert_eq!(w.sundial_cooldown, 3, "the cooldown did not reset");
    }

    #[test]
    fn dw_2_4_moondial_mirrors_sundial() {
        // `Player.cs:32646-32653` + `Main.Moondialing` (`Main.cs:6184-6196`):
        // tile 663 arms the FF-to-dusk and its own 8-day cooldown.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 663);
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()) {
            TileInteraction::DialStarted { sundial } => assert!(!sundial),
            other => panic!("expected DialStarted, got {other:?}"),
        }
        assert!(w.fast_forward_to_dusk);
        assert!(!w.fast_forward_to_dawn);
        assert_eq!(w.moondial_cooldown, 8);
        assert_eq!(w.sundial_cooldown, 0);

        // The refused mirror: a re-click mid-dial is consumed, changes
        // nothing (`:32648`).
        match interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()) {
            TileInteraction::DialRefused => {}
            other => panic!("expected DialRefused, got {other:?}"),
        }
        assert!(w.fast_forward_to_dusk);
        assert_eq!(w.moondial_cooldown, 8);
    }

    #[test]
    fn dw_2_6_adjacent_non_dial_tile_has_no_interaction() {
        // Dirty check: the dispatcher routes on the clicked cell's own kind —
        // a stone tile beside the dial, the floor under it and plain air all
        // fall through as `None` with no dial state touched.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        set_active(&mut w, 11, 19, 1); // stone next to the sundial
        let blocked = no_blocker as fn(i32, i32) -> bool;
        // The dial's own cell (10, 19) answered above; the active tiles
        // (the stone (11, 19), the floor (10, 20)) reach the C#'s station
        // tail (`Player.cs:33279`), the air cell (12, 19) stays `None`.
        for (x, y, air) in [(11i32, 19i32, false), (10, 20, false), (12, 19, true)] {
            let hit = interact_at(&mut w, x, y, 1, &blocked, &mut rng());
            if air {
                assert!(matches!(hit, TileInteraction::None), "({x}, {y}) is air");
            } else {
                assert!(
                    matches!(hit, TileInteraction::StationInteract { .. }),
                    "({x}, {y}) probes the station tail"
                );
            }
        }
        assert!(!w.fast_forward_to_dawn && !w.fast_forward_to_dusk);
        assert_eq!(w.sundial_cooldown, 0);
        assert_eq!(w.moondial_cooldown, 0);
    }

    /// W4-e10 — the right-click toggle family (`Player.cs:32228-32284`):
    /// the music box 139, the campfire 215, the fountain 207, the monolith
    /// 410 and the party centre 455 all answer `RightClickToggle` with the
    /// CLICKED cell (the Mech-28 click's position, the app half plays it).
    /// The campfire's 3×2 frameY flip is the core's; a partial campfire
    /// keeps its frames (the inline validation refuses, no kills).
    #[test]
    fn w4_e10_the_right_click_toggle_family_routes_with_the_clicked_cell() {
        let blocked = no_blocker as fn(i32, i32) -> bool;
        for kind in [139u16, 207, 410, 455] {
            let mut w = object_world();
            set_active(&mut w, 10, 19, kind);
            match interact_at(&mut w, 10, 19, 1, &blocked) {
                TileInteraction::RightClickToggle { x, y, kind: k } => {
                    assert_eq!((x, y, k), (10, 19, kind));
                }
                other => panic!("kind {kind}: expected RightClickToggle, got {other:?}"),
            }
        }
        // The campfire: a full 3×2 at (9..12, 18..20), lit (anchor frameY
        // 0), each cell carrying its sprite frame (frameX (x-9)*18,
        // frameY (y-18)*18) — the click on the middle cell (10, 19)
        // decodes the anchor (9, 18) and flips every cell's frameY +36.
        let mut w = object_world();
        for x in 9..12 {
            for y in 18..20 {
                set_active(&mut w, x, y, 215);
                let i = w.index(x, y);
                w.tiles[i].frame_x = ((x - 9) * 18) as i16;
                w.tiles[i].frame_y = ((y - 18) * 18) as i16;
            }
        }
        match interact_at(&mut w, 10, 19, 1, &blocked) {
            TileInteraction::RightClickToggle { kind: 215, .. } => {}
            other => panic!("campfire: expected RightClickToggle, got {other:?}"),
        }
        for x in 9..12 {
            for y in 18..20 {
                assert_eq!(
                    w.tile_frame_y(x, y),
                    (y - 18) * 18 + 36,
                    "the lit->unlit flip ({x},{y})"
                );
            }
        }
        // The re-click flips back (`frameY >= 36` -> -36).
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &blocked),
            TileInteraction::RightClickToggle { kind: 215, .. }
        ));
        assert_eq!(w.tile_frame_y(9, 18), 0);
        // A partial campfire (one cell missing): consumed, frames untouched.
        let mut w2 = object_world();
        for x in 9..12 {
            for y in 18..20 {
                if (x, y) != (11, 19) {
                    set_active(&mut w2, x, y, 215);
                    let i = w2.index(x, y);
                    w2.tiles[i].frame_x = ((x - 9) * 18) as i16;
                    w2.tiles[i].frame_y = ((y - 18) * 18) as i16;
                }
            }
        }
        assert!(matches!(
            interact_at(&mut w2, 10, 19, 1, &blocked),
            TileInteraction::RightClickToggle { kind: 215, .. }
        ));
        assert_eq!(w2.tile_frame_y(9, 18), 0, "the invalid square keeps its frames");
    }

    /// W4-e10 — the buff-station family (`Player.cs:32594-32634`): the seven
    /// right-click buff tiles answer `BuffStation` with their kind, BEFORE the
    /// dial branches (`:32636`) and the crafting-station tail (`:33279`). 699
    /// also counts as a Bone Welder (`TileCountsAs` 699→13), but the buff arm
    /// wins the click first — it never reaches the station tail.
    #[test]
    fn w4_e10_the_buff_station_family_routes_before_the_station_tail() {
        let blocked = no_blocker as fn(i32, i32) -> bool;
        for kind in [125u16, 621, 464, 699, 377, 354, 287] {
            let mut w = object_world();
            set_active(&mut w, 10, 19, kind);
            match interact_at(&mut w, 10, 19, 1, &blocked) {
                TileInteraction::BuffStation { kind: k } => assert_eq!(k, kind),
                other => panic!("kind {kind}: expected BuffStation, got {other:?}"),
            }
        }
        // An inactive station cell is invisible (`TileInteractionsCheck`).
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 125,
            ..Default::default()
        };
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &blocked),
            TileInteraction::None
        ));
    }

    #[test]
    fn inactive_dial_tile_has_no_interaction() {
        // `TileInteractionsCheck` gates on active tiles only
        // (`Player.cs:32031-32041`): an inactive dial cell is invisible to
        // the use path.
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 356,
            ..Default::default()
        }; // no FLAG_ACTIVE
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &blocked, &mut rng()),
            TileInteraction::None
        ));
        assert!(!w.fast_forward_to_dawn);
    }
}
}
