//! `TileObjectData` + `TileObject`: the multi-tile placement registry, `CanPlace`
//! and `Place`, ported 1:1 for the wooden set (work bench, table, chair, door,
//! chest).
//!
//! Adding an object = one data entry in [`TABLE`]; the system below reads only
//! the table (full registry generator is a follow-up).
//!
//! # C#→Rust map (DW-1.5)
//!
//! Ported fields (`TileObjectData.cs` — property getters fall through to
//! `_baseObject`, whose defaults come from `SetupBaseObject` :1743-1803 and
//! `Initialize` :2117-2133; `LavaDeath=true` is folded into
//! `LavaPlacement=NotAllowed` by `Calculate` :1821-1831 and baked into the data):
//!
//! | C# (`TileObjectData`) | Rust (`TileObjectDef`) |
//! |---|---|
//! | `Width` / `Height` :1174/:1213 | `width` / `height` |
//! | `Origin` :1252 | `origin` |
//! | `Direction` :1285 | `direction` |
//! | `RandomStyleRange` :1318 | `random_style_range` |
//! | `SpecificRandomStyles` :1351 | `specific_random_styles` |
//! | `FlattenAnchors` :1384 | `flatten_anchors` |
//! | `AnchorTop/Bottom/Left/Right` :191/:224/:257/:290 | `anchor_top/bottom/left/right` |
//! | `AnchorWall` :323 | `anchor_wall` |
//! | `AnchorValidTiles` :356 | `anchor_valid_tiles` |
//! | `AnchorInvalidTiles` :395 | `anchor_invalid_tiles` |
//! | `AnchorAlternateTiles` :434 | `anchor_alternate_tiles` |
//! | `AnchorValidWalls` :473 | `anchor_valid_walls` |
//! | `WaterPlacement` / `LavaPlacement` | `water_placement` / `lava_placement` |
//! | `HookCheckIfCanPlace` / `HookPlaceOverride` | `hook_check_if_can_place` / `hook_place_override` |
//! | `CoordinateHeights` :1417 | `coordinate_heights` |
//! | `CoordinateWidth` :1503 / `CoordinatePadding` :1537 | `coordinate_width` / `coordinate_padding` |
//! | `CoordinatePaddingFix` | `coordinate_padding_fix` |
//! | `CoordinateFullWidth/Height` :1605/:1621 (`Calculate` :1805) | [`TileObjectDef::coordinate_full_width`] / [`TileObjectDef::coordinate_full_height`] |
//! | `Style` :976 / `StyleHorizontal` :943 | `style` / `style_horizontal` |
//! | `StyleWrapLimit` :1009 / `StyleLineSkip` :1108 / `StyleMultiplier` :1141 | `style_wrap_limit` / `style_line_skip` / `style_multiplier` |
//! | `Alternates` (`addAlternate` :2097) | `alternates` |
//! | `SubTiles` (`addSubTile` :2015) | `subtiles` |
//!
//! Not ported (follow-ups, by design):
//! - **Draw fields** (`DrawYOffset`, `DrawStyleOffset`, `DrawStepDown`,
//!   `DrawFlipHorizontal/Vertical`, `DrawFrameOffsets`,
//!   `StyleWrapLimitVisualOverride`): render-only; multi-tile drawing already
//!   works from saved frames (TILE-DRAW-1).
//! - **`UsesCustomCanPlace` / `UsesGlobalLiquidChecks`**: read by
//!   `Player.PlaceThing` routing and liquid-death checks, not by
//!   `CanPlace`/`Place`; lands with Phase 2's dispatcher.
//! - **Chest hooks** (`Chest.FindEmptyChest` TileObjectData.cs:3464,
//!   `Chest.AfterPlacement_Hook` :3465): the M3-CHEST-UI seam — the chest is a
//!   pure tile object here (no `Chest` entity, no item storage). The hook
//!   *branches* in `can_place`/`place` are ported; the wooden chest carries
//!   `None`.
//! - **SubTiles data** (styles 13/14/16/19/25/43/47/48, 51-67 of the wooden
//!   tiles, `TileObjectData.cs:2246/2286/3182-3190/4151-4172`): the resolution
//!   mechanism is ported ([`get_tile_data`]); the wooden slice places style 0
//!   only, so the entries carry empty subtile lists. Full registry follow-up.
//! - **Random styles** (`Main.rand` path TileObject.cs:787-793, the
//!   `randomCache` stickiness :764-781 and `SpecificRandomStyles` :794-825):
//!   every wooden def has `RandomStyleRange = 0`, so the resolved `random` is
//!   always `-1`; the `forced_random` path is ported, the `Main.rand` path
//!   needs app-side RNG (Phase 2 wiring).
//! - **`onlyCheck`/preview** (`TileObjectPreviewData.placementCache`,
//!   `objectPreview`, the anchor `Rectangle` bookkeeping TileObject.cs:219-321):
//!   dead code when `onlyCheck=false` (verified by reading; the rectangle and
//!   `num10`/`num11` are not read after :332). Smart-cursor/preview is app work.
//! - **`GetStyleOverride`**: no wooden def uses it.
//!
//! Hand-extracted tile tables (the `GenTileData` generator cannot emit
//! `Main.`-prefixed assignments — see `torch.rs`'s module doc for the same
//! precedent): [`TILE_CUT`], [`TILE_TABLE`], [`BREAKABLE_WHEN_PLACING`],
//! [`IS_A_DRIP_TILE`], [`IS_VINE`], [`NOT_REALLY_SOLID`]. `Main.tileNoAttach`,
//! `TileID.Sets.IsATreeTrunk`,
//! `PlatformFrameWidth()` and `WorldGen.PlatformProperTopFrame` are reused from
//! [`crate::torch`].

use crate::tile::Tile;
use crate::world::World;

/// `Microsoft.Xna.Framework.Point16` as used by `TileObjectData` (tile coords).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Point16 {
    pub x: i32,
    pub y: i32,
}

impl Point16 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self { x: 0, y: 0 };
}

/// `Terraria.Enums.AnchorType` (`AnchorType.cs:6-20`), as plain bit flags.
pub mod anchor_type {
    pub const NONE: u16 = 0;
    pub const SOLID_TILE: u16 = 1;
    pub const SOLID_WITH_TOP: u16 = 2;
    pub const TABLE: u16 = 4;
    pub const SOLID_SIDE: u16 = 8;
    pub const TREE: u16 = 0x10;
    pub const ALTERNATE_TILE: u16 = 0x20;
    pub const EMPTY_TILE: u16 = 0x40;
    pub const SOLID_BOTTOM: u16 = 0x80;
    pub const PLATFORM: u16 = 0x100;
    pub const PLANTER_BOX: u16 = 0x200;
    pub const PLATFORM_NON_HAMMERED: u16 = 0x400;
    pub const ALL_FLAT_HEIGHT: u16 = 0x800;
}

/// `Terraria.DataStructures.AnchorData` (`AnchorData.cs:3-9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AnchorData {
    pub ty: u16,
    pub tile_count: i32,
    pub check_start: i32,
}

impl AnchorData {
    pub const fn new(ty: u16, tile_count: i32, check_start: i32) -> Self {
        Self {
            ty,
            tile_count,
            check_start,
        }
    }

    pub const EMPTY: Self = Self {
        ty: anchor_type::NONE,
        tile_count: 0,
        check_start: 0,
    };
}

/// `Terraria.Enums.TileObjectDirection` (`TileObjectDirection.cs:3-8`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TileObjectDirection {
    #[default]
    None,
    PlaceLeft,
    PlaceRight,
}

/// `Terraria.Enums.LiquidPlacement` (`LiquidPlacement.cs:5-10`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LiquidPlacement {
    #[default]
    Allowed,
    NotAllowed,
    OnlyInLiquid,
    OnlyInFullLiquid,
}

/// Interaction dispatch tag (Phase 4's right-click hook routes on this).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Interaction {
    #[default]
    None,
    /// Closed door (tile 10) and open door (tile 11).
    Door,
    /// The lit toggle — the candle family (33/49/174/372/646,
    /// `Player.cs:32378-32390`): `frameX += (frameX > 0) ? -18 : 18`.
    Candle,
    /// The chair sit (tile 15, `Player.cs:32313-32321` →
    /// `PlayerSittingHelper.SitDown`).
    Chair,
    /// The bank open (tiles 29/97/463/491, `Player.cs:33056-33138`): the
    /// field carries the chest index -2..-5 (piggy/safe/forge/vault).
    Bank(i32),
    /// The bed (tile 79, `Player.cs:32654-32692`): the top half sleeps, the
    /// bottom half sets the spawn point.
    Bed,
    /// The grandfather clock (tile 104, `Player.cs:32725-32759`): the time
    /// text.
    Clock,
    /// The Extractinator (tiles 219/642): the held item is converted into an
    /// output item that spawns at the cursor (`Player.cs:41665-41697`,
    /// `ExtractinatorUse` `:42072-42079`). The roll itself lives in
    /// [`crate::extract`]; this tag only answers "this tile is an extractinator"
    /// the way the C# chain tests the `Main.tile[myX, myY].type` literals.
    Extractinator,
}

/// `Terraria.DataStructures.PlacementHook` (`PlacementHook.cs:3-15`). The
/// `World` parameter is the port's stand-in for the hook's global `Main` state;
/// no wooden def installs a hook (chest hooks are the M3-CHEST-UI seam).
pub type PlacementHookFn = fn(&World, i32, i32, u16, i32, i32, i32) -> i32;

#[derive(Clone, Copy, Debug, Default)]
pub struct PlacementHook {
    pub hook: Option<PlacementHookFn>,
    pub bad_return: i32,
    pub bad_response: i32,
    pub processed_coordinates: bool,
}

impl PlacementHook {
    /// `PlacementHook.Empty` (`PlacementHook.cs:51-54`).
    pub const EMPTY: Self = Self {
        hook: None,
        bad_return: 0,
        bad_response: 0,
        processed_coordinates: false,
    };
}

/// One resolved `TileObjectData` definition (a tile's base def, one of its
/// alternates, or one of its sub-tiles). Public fields mirror the C# class
/// (see the module doc map).
#[derive(Clone, Copy, Debug, Default)]
pub struct TileObjectDef {
    /// Registry key — the `_data[tileType]` index this def is registered under
    /// (`TileObjectData.addTile(tileType)`, `TileObjectData.cs:2008-2013`).
    /// Only top-level [`TABLE`] entries carry it; nested alternates/sub-tiles
    /// are reached through their parent and leave it 0.
    pub kind: u16,
    pub width: i32,
    pub height: i32,
    pub origin: Point16,
    pub direction: TileObjectDirection,
    pub random_style_range: i32,
    pub specific_random_styles: &'static [i32],
    pub flatten_anchors: bool,
    pub anchor_top: AnchorData,
    pub anchor_bottom: AnchorData,
    pub anchor_left: AnchorData,
    pub anchor_right: AnchorData,
    pub anchor_wall: bool,
    pub anchor_valid_tiles: &'static [u16],
    pub anchor_invalid_tiles: &'static [u16],
    pub anchor_alternate_tiles: &'static [u16],
    pub anchor_valid_walls: &'static [u16],
    pub water_placement: LiquidPlacement,
    pub lava_placement: LiquidPlacement,
    pub hook_check_if_can_place: PlacementHook,
    pub hook_place_override: PlacementHook,
    pub coordinate_heights: &'static [i32],
    pub coordinate_width: i32,
    pub coordinate_padding: i32,
    pub coordinate_padding_fix: Point16,
    pub style: i32,
    pub style_horizontal: bool,
    pub style_wrap_limit: i32,
    pub style_multiplier: i32,
    pub style_line_skip: i32,
    pub alternates: &'static [TileObjectDef],
    pub subtiles: &'static [(u16, &'static TileObjectDef)],
    /// `Main.tileFrameImportant[type]` (`Main.cs:7637-7656` for the wooden set);
    /// `place` mirrors it into `world.frame_important` so `framing` keeps the
    /// stored frames.
    pub frame_important: bool,
    pub interaction: Interaction,
}

impl TileObjectDef {
    /// `TileObjectData.CoordinateFullWidth` (`TileObjectData.cs:1605-1619` with
    /// `Calculate` :1812).
    pub fn coordinate_full_width(&self) -> i32 {
        (self.coordinate_width + self.coordinate_padding) * self.width
            + self.coordinate_padding_fix.x
    }

    /// `TileObjectData.CoordinateFullHeight` (`TileObjectData.cs:1621-1635` with
    /// `Calculate` :1813-1820).
    pub fn coordinate_full_height(&self) -> i32 {
        let mut total = 0;
        for h in self.coordinate_heights {
            total += h + self.coordinate_padding;
        }
        total + self.coordinate_padding_fix.y
    }
}

/// A resolved placement — the C# `TileObject` struct (`TileObject.cs:11-24`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileObjectInstance {
    pub x_coord: i32,
    pub y_coord: i32,
    /// `TileObject.type` (the tile id).
    pub kind: u16,
    pub style: i32,
    pub alternate: i32,
    pub random: i32,
}

/// The wooden slice of the registry.
///
/// Every entry is the hand-resolved result of its `InitializeTileObjects`
/// builder chain: the base defaults come from `SetupBaseObject`
/// (`TileObjectData.cs:1743-1803`) and `Initialize` (:2117-2133), and
/// `LavaDeath=true` is pre-folded into `lava_placement = NotAllowed` exactly
/// like `Calculate` does (:1821-1831).
pub const TABLE: &[TileObjectDef] = &[
    DOOR_CLOSED,
    DOOR_OPEN,
    TABLE_WOOD,
    CHAIR,
    WORK_BENCH,
    CHEST,
    SUNFLOWER,
    POTS,
    CLAY_POT,
    IMMATURE_HERBS,
    MATURE_HERBS,
    BLOOMING_HERBS,
    SMALL_PILES,
    LARGE_PILES,
    LARGE_PILES2,
    CANDLE,
    CANDELABRA,
    CHANDELIER,
    HANGING_LANTERN,
    LAMP,
    BOOKCASE,
    PIANO,
    DRESSER,
    SINK,
    GRANDFATHER_CLOCK,
    BED,
    BATHTUB,
    // The banks (M3-STORE-1): the piggy 29, the safe 97, the forge 463,
    // the vault 491.
    PIGGY_BANK,
    SAFE,
    DEFENDERS_FORGE,
    VOID_VAULT,
    // The extractinators (M7-EXTRACT-1): the silt 219, the chlorophyte 642.
    SILT_EXTRACTINATOR,
    CHLOROPHYTE_EXTRACTINATOR,
];

// -- piggy bank (tile 29) — `TileObjectData.cs:2801-2811` + Style2x1 --------
/// `Style2x1` (:2798-2808) with the table anchor swap (`:2805`: `AnchorBottom
/// = new AnchorData(AnchorType.Table, newTile.Width, 0)`); item 87, value
/// 10000 (`Item.cs:2407-2416`). The kill rides `Check2x1`'s case-29 arm
/// (`WorldGen.cs:47003-47013`).
const PIGGY_BANK: TileObjectDef = TileObjectDef {
    kind: 29,
    width: 2,
    height: 1,
    origin: Point16::new(0, 0),
    anchor_bottom: AnchorData::new(anchor_type::TABLE, 2, 0),
    coordinate_heights: &[16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true, // `Main.cs:7656`'s list
    interaction: Interaction::Bank(-2),
    ..base_def()
};

// -- safe (tile 97) — `TileObjectData.cs:3590` (a plain Style2x2 copy) ------
/// `Style2x2` (:3451-3460) with no overrides (`TileObjectData.cs:3588-3590`);
/// item 346, value 200000 (`Item.cs:5438-5449`).
const SAFE: TileObjectDef = TileObjectDef {
    kind: 97,
    width: 2,
    height: 2,
    origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    interaction: Interaction::Bank(-3),
    ..base_def()
};

// -- defender's forge (tile 463) — `TileObjectData.cs:3402-3406` + Style3x4 --
/// `Style3x4` with `DrawYOffset = 2` (`:3405` — draw-only, unported here);
/// item 3813, rare 3, value 100000 (`Item.cs:33745-33754`).
const DEFENDERS_FORGE: TileObjectDef = TileObjectDef {
    kind: 463,
    width: 3,
    height: 4,
    origin: Point16::new(1, 3),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    interaction: Interaction::Bank(-4),
    ..base_def()
};

// -- void vault (tile 491) — `TileObjectData.cs:3797-3800` + Style3x3 --------
/// `Style3x3` with `DrawYOffset = 2` (`:3799` — draw-only); item 4076, rare
/// 3, value 100000 (`Item.cs:35741-35752`). The placed tile opens bank4
/// regardless of holding the Void Lens (`Player.cs:33127-33133` — the only
/// gate is `disableVoidBag`).
const VOID_VAULT: TileObjectDef = TileObjectDef {
    kind: 491,
    width: 3,
    height: 3,
    origin: Point16::new(1, 2),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    interaction: Interaction::Bank(-5),
    ..base_def()
};

// -- silt extractinator (tile 219) / chlorophyte extractinator (tile 642) -----
// M7-EXTRACT-1 — `TileObjectData.cs:4042-4045`:
//     newTile.CopyFrom(TileObjectData.Style3x3);
//     newTile.DrawYOffset = 2;
//     addTile(219);                      // and the identical 642 at :4043-4045
// `Style3x3` itself is `:4015-4031` (width 3, height 3, origin (1, 2),
// AnchorBottom SolidTile|SolidWithTop|SolidSide over the width, custom
// CanPlace, CoordinateHeights {16,16,16}, CoordinateWidth 16, Padding 2,
// LavaDeath). No alternates, no hooks, `StyleHorizontal` stays false (the
// module default `:1800`). Items: 997 Extractinator (`ItemID.cs:3677`) and
// 5296 Chlorophyte Extractinator (`ItemID.cs:12275`); `Main.tileFrameImportant`
// for both (`Main.cs:7401-7402`).
//
/// `Style3x3` + `DrawYOffset = 2`, the silt extractinator (tile 219, item 997).
const SILT_EXTRACTINATOR: TileObjectDef = TileObjectDef {
    kind: 219,
    width: 3,
    height: 3,
    origin: Point16::new(1, 2),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    frame_important: true,
    interaction: Interaction::Extractinator,
    ..base_def()
};

/// `Style3x3` + `DrawYOffset = 2`, the chlorophyte extractinator (tile 642,
/// item 5296) — the same def with the cheaper ×0.33 use time
/// (`Player.cs:41673-41677`) applied by the app's extraction arm.
const CHLOROPHYTE_EXTRACTINATOR: TileObjectDef = TileObjectDef {
    kind: 642,
    ..SILT_EXTRACTINATOR
};

/// `Main.tileAlch` (`Main.cs:7806-7808`): the three herb-stage tiles the legacy
/// `WorldGen.PlaceTile` dispatcher routes to `PlaceAlch` (`WorldGen.cs:60177-60181`).
pub const TILE_ALCH: &[u16] = &[82, 83, 84];

/// Membership in [`TILE_ALCH`].
pub fn tile_alch(kind: u16) -> bool {
    TILE_ALCH.contains(&kind)
}

/// `TileObjectData._baseObject` effective defaults
/// (`TileObjectData.cs:1743-1803` SetupBaseObject; `LavaDeath` folding per
/// `Calculate` :1821-1831).
const fn base_def() -> TileObjectDef {
    TileObjectDef {
        kind: 0,
        width: 1,
        height: 1,
        origin: Point16::ZERO,
        direction: TileObjectDirection::None,
        random_style_range: 0,
        specific_random_styles: &[],
        flatten_anchors: false,
        anchor_top: AnchorData::EMPTY,
        anchor_bottom: AnchorData::EMPTY,
        anchor_left: AnchorData::EMPTY,
        anchor_right: AnchorData::EMPTY,
        anchor_wall: false,
        anchor_valid_tiles: &[],
        anchor_invalid_tiles: &[],
        anchor_alternate_tiles: &[],
        anchor_valid_walls: &[],
        water_placement: LiquidPlacement::Allowed,
        lava_placement: LiquidPlacement::NotAllowed,
        hook_check_if_can_place: PlacementHook::EMPTY,
        hook_place_override: PlacementHook::EMPTY,
        coordinate_heights: &[16],
        coordinate_width: 0,
        coordinate_padding: 0,
        coordinate_padding_fix: Point16::ZERO,
        style: 0,
        style_horizontal: false,
        style_wrap_limit: 0,
        style_multiplier: 1,
        style_line_skip: 1,
        alternates: &[],
        subtiles: &[],
        frame_important: false,
        interaction: Interaction::None,
    }
}

/// `TileID.Sets.BreakableWhenPlacing` (`TileID.cs:604`).
const BREAKABLE_WHEN_PLACING: &[u16] = &[324, 186, 187, 185, 165, 530, 233, 227, 485, 81, 624];

/// `TileID.Sets.IsADripTile` (`TileID.cs:485`).
const IS_A_DRIP_TILE: &[u16] = &[373, 374, 375, 461, 709];

/// `TileID.Sets.IsVine` (`TileID.cs:531`).
const IS_VINE: &[u16] = &[52, 382, 62, 115, 205, 528, 636, 638];

/// `TileID.Sets.NotReallySolid` (`TileID.cs:592`).
const NOT_REALLY_SOLID: &[u16] = &[387, 388, 10];

/// `Main.tileCut` (`Main.cs` `tileCut` assignments, 41 ids).
const TILE_CUT: &[u16] = &[
    3, 24, 28, 32, 51, 52, 61, 62, 69, 71, 73, 74, 82, 83, 84, 110, 113, 115, 184, 201, 205, 231,
    236, 254, 352, 382, 444, 454, 484, 485, 518, 519, 528, 529, 549, 636, 637, 638, 654, 655, 711,
];

/// `Main.tileTable` (`Main.cs` `tileTable` assignments, 77 ids).
const TILE_TABLE: &[u16] = &[
    14, 18, 19, 87, 88, 101, 114, 275, 276, 277, 278, 279, 280, 281, 285, 286, 296, 297, 298, 299,
    309, 310, 339, 358, 359, 361, 362, 363, 364, 376, 380, 391, 392, 393, 394, 405, 413, 414, 427,
    469, 532, 533, 538, 542, 544, 550, 551, 553, 554, 555, 556, 558, 559, 582, 599, 600, 601, 602,
    603, 604, 605, 606, 607, 608, 609, 610, 611, 612, 619, 629, 632, 640, 643, 644, 645, 710,
];

/// `TileObjectData.anchorInvalidTiles` shared by the chest
/// (`TileObjectData.cs:2112-2116`, applied at :3466).
const CHEST_ANCHOR_INVALID: &[u16] = &[127, 138, 664, 665, 484, 711, 712, 713, 714, 715, 716];

const ANCHOR_SWTSS: u16 =
    anchor_type::SOLID_TILE | anchor_type::SOLID_WITH_TOP | anchor_type::SOLID_SIDE;

// -- door, closed (tile 10) — `TileObjectData.cs:2223-2247` ------------------

/// Base def (`:2223-2235`).
const DOOR_CLOSED_BASE: TileObjectDef = TileObjectDef {
    width: 1,
    height: 3,
    origin: Point16::new(0, 0),
    anchor_top: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
    anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
    coordinate_heights: &[16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_wrap_limit: 36,
    style_line_skip: 3,
    frame_important: true, // `Main.cs:7637`
    interaction: Interaction::Door,
    ..base_def()
};

/// Click-point alternates: the door can be targeted on any of its three tiles
/// (`:2236-2241`, both `addAlternate(0)` — `Style` stays 0).
const DOOR_CLOSED_ALT_MID: TileObjectDef = TileObjectDef {
    origin: Point16::new(0, 1),
    ..DOOR_CLOSED_BASE
};
const DOOR_CLOSED_ALT_BOT: TileObjectDef = TileObjectDef {
    origin: Point16::new(0, 2),
    ..DOOR_CLOSED_BASE
};
const DOOR_CLOSED: TileObjectDef = TileObjectDef {
    kind: 10,
    alternates: &[DOOR_CLOSED_ALT_MID, DOOR_CLOSED_ALT_BOT],
    ..DOOR_CLOSED_BASE
};

// -- door, open (tile 11) — `TileObjectData.cs:2248-2291` --------------------

/// Base def (`:2248-2261`): 2 wide, opens to the right (`PlaceRight`).
const DOOR_OPEN_BASE: TileObjectDef = TileObjectDef {
    width: 2,
    height: 3,
    origin: Point16::new(0, 0),
    anchor_top: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
    anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
    coordinate_heights: &[16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_wrap_limit: 36,
    style_line_skip: 2,
    direction: TileObjectDirection::PlaceRight,
    frame_important: true, // `Main.cs:7638`
    interaction: Interaction::Door,
    ..base_def()
};

/// Right-open click points (`:2262-2267`, both `addAlternate(0)`).
const DOOR_OPEN_RIGHT_MID: TileObjectDef = TileObjectDef {
    origin: Point16::new(0, 1),
    ..DOOR_OPEN_BASE
};
const DOOR_OPEN_RIGHT_BOT: TileObjectDef = TileObjectDef {
    origin: Point16::new(0, 2),
    ..DOOR_OPEN_BASE
};
/// Left-open variants: origin on the right column, anchors follow the hinge
/// column (`checkStart 1`), `Direction = PlaceLeft`, `Style = 1`
/// (`:2268-2285`, three `addAlternate(1)`).
const DOOR_OPEN_LEFT_TOP: TileObjectDef = TileObjectDef {
    origin: Point16::new(1, 0),
    anchor_top: AnchorData::new(anchor_type::SOLID_TILE, 1, 1),
    anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 1),
    direction: TileObjectDirection::PlaceLeft,
    style: 1,
    ..DOOR_OPEN_BASE
};
const DOOR_OPEN_LEFT_MID: TileObjectDef = TileObjectDef {
    origin: Point16::new(1, 1),
    anchor_top: AnchorData::new(anchor_type::SOLID_TILE, 1, 1),
    anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 1),
    direction: TileObjectDirection::PlaceLeft,
    style: 1,
    ..DOOR_OPEN_BASE
};
const DOOR_OPEN_LEFT_BOT: TileObjectDef = TileObjectDef {
    origin: Point16::new(1, 2),
    anchor_top: AnchorData::new(anchor_type::SOLID_TILE, 1, 1),
    anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 1),
    direction: TileObjectDirection::PlaceLeft,
    style: 1,
    ..DOOR_OPEN_BASE
};
const DOOR_OPEN: TileObjectDef = TileObjectDef {
    kind: 11,
    alternates: &[
        DOOR_OPEN_RIGHT_MID,
        DOOR_OPEN_RIGHT_BOT,
        DOOR_OPEN_LEFT_TOP,
        DOOR_OPEN_LEFT_MID,
        DOOR_OPEN_LEFT_BOT,
    ],
    ..DOOR_OPEN_BASE
};

// -- work bench (tile 18) — `TileObjectData.cs:2818-2825` + Style2x1 ---------

/// `Style2x1` preset (`:2798-2808`) with the bench's 18 px rows (`:2820`) and
/// `StyleWrapLimitVisualOverride = 56` (`:2819`, draw-only — not ported).
const WORK_BENCH: TileObjectDef = TileObjectDef {
    kind: 18,
    width: 2,
    height: 1,
    origin: Point16::new(0, 0),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[18],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true, // `Main.cs:7653`
    ..base_def()
};

// -- table (tile 14) — `TileObjectData.cs:3169-3191` + Style3x2 --------------

/// `Style3x2` preset (`:3169-3179`) with the 18 px bottom row (`:3181`).
/// Sub-tiles for styles 13 (liquid variant, `:3182-3185`) and 25 (1-tall
/// variant, `:3186-3190`) exist in C# — full-registry follow-up.
const TABLE_WOOD: TileObjectDef = TileObjectDef {
    kind: 14,
    width: 3,
    height: 2,
    origin: Point16::new(1, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 18],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true, // `Main.cs:7641`
    ..base_def()
};

// -- chair (tile 15) — `TileObjectData.cs:4141-4173` + Style1x2 --------------

/// `Style1x2` preset (`:4131-4140`) with the chair overrides (`:4142-4147`):
/// 18 px bottom row, `PlaceLeft` base, wrap 2 / multiplier 2 (left and right
/// frames are adjacent columns), `CoordinatePaddingFix` (0, 2).
const CHAIR_BASE: TileObjectDef = TileObjectDef {
    width: 1,
    height: 2,
    origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 1, 0),
    coordinate_heights: &[16, 18],
    coordinate_width: 16,
    coordinate_padding: 2,
    coordinate_padding_fix: Point16::new(0, 2),
    style_wrap_limit: 2,
    style_multiplier: 2,
    style_horizontal: true,
    direction: TileObjectDirection::PlaceLeft,
    frame_important: true, // `Main.cs:7649`
    interaction: Interaction::Chair,
    ..base_def()
};

/// The `PlaceRight` alternate (`:4148-4150`, `addAlternate(1)` → `Style = 1`).
const CHAIR_RIGHT: TileObjectDef = TileObjectDef {
    direction: TileObjectDirection::PlaceRight,
    style: 1,
    ..CHAIR_BASE
};
const CHAIR: TileObjectDef = TileObjectDef {
    kind: 15,
    alternates: &[CHAIR_RIGHT],
    ..CHAIR_BASE
};

// -- chest (tile 21) — `TileObjectData.cs:3461-3470` + Style2x2 --------------

/// `Style2x2` preset (`:3451-3460`) with the chest overrides (`:3461-3469`):
/// 18 px bottom row, horizontal styles, `LavaDeath = false`, the
/// `AnchorInvalidTiles` list, and the bottom anchor WITHOUT `AnchorType.Table`
/// (`:3469` — a chest will not hang off a table edge). The C# hooks
/// `Chest.FindEmptyChest` (:3464) / `Chest.AfterPlacement_Hook` (:3465) are the
/// M3-CHEST-UI seam: the chest stays a pure tile object here.
///
/// `LavaDeath = false` (:3468) never touches placement: `SetupBaseObject`
/// defaults `LavaPlacement = NotAllowed` (:1768) and `Calculate` (:1821-1831)
/// folds death -> placement only when the death flag is *true*, so the chest
/// keeps the base default — vanilla refuses to place a chest in lava
/// (`TileObject.CanPlace` → `LiquidPlace`, TileObject.cs:342).
const CHEST: TileObjectDef = TileObjectDef {
    kind: 21,
    width: 2,
    height: 2,
    origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16, 18],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    anchor_invalid_tiles: CHEST_ANCHOR_INVALID,
    frame_important: true, // `Main.cs:7656`
    ..base_def()
};

// -- sunflower (tile 27) — `TileObjectData.cs:4851-4864` ----------------------

/// The 2x4 sunflower (`:4851-4864`): Origin(0,3), `AnchorBottom = SolidTile`
/// over the whole width, 18 px bottom row, `AnchorValidTiles` grass family,
/// `RandomStyleRange = 3` (the three sheet variants resolve through
/// `CalculatePlacementStyle` + `can_place`'s random slot), `LavaDeath = true`
/// (folded to `lava_placement = NotAllowed`) and water refused. `DrawYOffset`
/// is not set; draw-only fields stay unported (module doc).
const SUNFLOWER: TileObjectDef = TileObjectDef {
    kind: 27,
    width: 2,
    height: 4,
    origin: Point16::new(0, 3),
    anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 2, 0),
    anchor_valid_tiles: &[2, 477, 109, 60, 492, 633],
    coordinate_heights: &[16, 16, 16, 18],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    random_style_range: 3,
    water_placement: LiquidPlacement::NotAllowed,
    frame_important: true, // `Main.cs:7663`
    ..base_def()
};

// -- loot pots (tile 28) — `TileObjectData.cs:3553-3559` + Style2x2 -----------

/// `Style2x2` preset (`:3451-3460`: 2x2, Origin(0,1), heights 16/16) with the
/// pot overrides (`:3554-3558`): `ApplyNaturalObjectRules()` (UsesCustomCanPlace
/// /UsesGlobalLiquidChecks — routing/liquid-death flags, not ported), the
/// bottom anchor WITHOUT `AnchorType.Table`, horizontal styles, wrap 3.
const POTS: TileObjectDef = TileObjectDef {
    kind: 28,
    width: 2,
    height: 2,
    origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    style_wrap_limit: 3,
    frame_important: true, // `Main.cs:7664`
    ..base_def()
};

// -- clay pot (tile 78) — `TileObjectData.cs:2384-2388` + StyleOnTable1x1 -----

/// `StyleOnTable1x1` preset (`:2324-2334`: 1x1, Table anchor, heights 16) with
/// the pot overrides (`:2385-2387`): the anchor widened to solid/full-top/
/// table/side and `LavaDeath = false`. As with the chest (:3468), setting only
/// `LavaDeath = false` never re-allows lava placement — `SetupBaseObject`
/// defaults `LavaPlacement = NotAllowed` (:1768) and `Calculate` (:1821-1831)
/// folds death -> placement only when the flag is true, so the baked
/// `NotAllowed` stands and the pot refuses lava (`TileObject.cs:342`).
const CLAY_POT: TileObjectDef = TileObjectDef {
    kind: 78,
    width: 1,
    height: 1,
    origin: Point16::ZERO,
    anchor_bottom: AnchorData::new(
        anchor_type::SOLID_TILE
            | anchor_type::SOLID_WITH_TOP
            | anchor_type::TABLE
            | anchor_type::SOLID_SIDE,
        1,
        0,
    ),
    coordinate_heights: &[16],
    coordinate_width: 16,
    coordinate_padding: 2,
    frame_important: true, // `Main.cs:7682`
    ..base_def()
};

// -- herbs (tiles 82/83/84) — `TileObjectData.cs:4738-4788` + StyleAlch -------

/// `StyleAlch` with the style-0 anchor set (`:4738-4756`): 1x1, 20 px row,
/// `AnchorBottom = SolidTile | AlternateTile` over `AnchorAlternateTiles {78}`,
/// water refused, `LavaDeath = true` (folded to `lava_placement =
/// NotAllowed`). The herb stage is the tile TYPE (82 immature / 83 mature /
/// 84 blooming) and the herb KIND is `frameX = 18 * kind` (`PlaceAlch`
/// `WorldGen.cs:45470-45475`) — the registry entry exists for the kill
/// cascade's gateway and the frames, because the seed route bypasses
/// `TileObject.CanPlace` (`Player.cs:39772` excludes 82).
///
/// Draw-only fields stay unported (module doc): `DrawYOffset = -2`,
/// `DrawFlipHorizontal = true`.
const HERB_BASE: TileObjectDef = TileObjectDef {
    width: 1,
    height: 1,
    origin: Point16::ZERO,
    anchor_bottom: AnchorData::new(
        anchor_type::SOLID_TILE | anchor_type::ALTERNATE_TILE,
        1,
        0,
    ),
    anchor_valid_tiles: &[2, 477, 109, 492],
    anchor_alternate_tiles: &[78],
    coordinate_heights: &[20],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    water_placement: LiquidPlacement::NotAllowed,
    frame_important: true, // `Main.cs:7685`
    ..base_def()
};

/// Sub-styles 1-6 (`:4757-4783`) — each `CopyFrom(StyleAlch)` plus its soil
/// set and liquid overrides. Style 4 (water: 53/116) allows water; style 5
/// (ash: 57/633) allows lava and drops `LavaDeath`; style 6 (snow) allows
/// water.
const HERB_1: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[60],
    ..HERB_BASE
};
const HERB_2: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[0, 59],
    ..HERB_BASE
};
const HERB_3: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[199, 203, 25, 23],
    ..HERB_BASE
};
const HERB_4: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[53, 116],
    water_placement: LiquidPlacement::Allowed,
    ..HERB_BASE
};
const HERB_5: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[57, 633],
    lava_placement: LiquidPlacement::Allowed,
    ..HERB_BASE
};
const HERB_6: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[147, 161, 163, 164, 200],
    water_placement: LiquidPlacement::Allowed,
    ..HERB_BASE
};

/// Tile 82 (`:4784`): the base def carries style 0, sub-tiles 1-6 resolve
/// through `get_tile_data` exactly like C# `SubTiles[style]` (:5136-5144).
const IMMATURE_HERBS: TileObjectDef = TileObjectDef {
    kind: 82,
    subtiles: &[
        (1, &HERB_1),
        (2, &HERB_2),
        (3, &HERB_3),
        (4, &HERB_4),
        (5, &HERB_5),
        (6, &HERB_6),
    ],
    ..HERB_BASE
};

/// Tile 83 — `FullCopyFrom(82)` (`:4785-4786`).
const MATURE_HERBS: TileObjectDef = TileObjectDef {
    kind: 83,
    frame_important: true, // `Main.cs:7686`
    ..IMMATURE_HERBS
};

/// Tile 84 — `FullCopyFrom(83)` (`:4787-4788`).
const BLOOMING_HERBS: TileObjectDef = TileObjectDef {
    kind: 84,
    frame_important: true, // `Main.cs:7687`
    ..MATURE_HERBS
};

// -- small piles (tile 185) — `TileObjectData.cs:4975-4985` + Style1x1/2x1 ----

/// The `SubTiles[0]` of the small piles (`:4981-4984`): a `Style2x1` copy
/// (`:2798-2808`) with wrap 53 — selected whenever the tile's `frameY >= 18`
/// (`GetStyle_SmallPiles` :5303-5310; the frameY-based override is the
/// tile-based `GetTileData(Tile)` overload, phase-2 cascade material — the
/// `(type, style)` overload indexes `SubTiles[0]` directly, :5136-5144).
const SMALL_PILES_2X1: TileObjectDef = TileObjectDef {
    width: 2,
    height: 1,
    origin: Point16::ZERO,
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    style_wrap_limit: 53,
    frame_important: true, // `Main.cs:7415`
    ..base_def()
};

/// Tile 185 (`:4975-4981`): a `Style1x1` copy (`:4238-4250`) — which carries
/// the shared 11-id `AnchorInvalidTiles` (`:4244` → the list built at
/// `:2112-2116`, the same list the chest uses) — plus `StyleHorizontal`,
/// `ApplyNaturalObjectRules()` (not ported) and `GetStyleOverride` (see the
/// sub-tile above). `LavaDeath = true` folds to `lava_placement = NotAllowed`.
const SMALL_PILES: TileObjectDef = TileObjectDef {
    kind: 185,
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 1, 0),
    anchor_invalid_tiles: CHEST_ANCHOR_INVALID,
    coordinate_heights: &[16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    subtiles: &[(0, &SMALL_PILES_2X1)],
    frame_important: true, // `Main.cs:7415`
    ..base_def()
};

// -- large piles (tiles 186/187) — `TileObjectData.cs:3285-3312` + Style3x2 ---

/// The large-pile `Style3x2` copy with the shared overrides (`:3286-3290` /
/// `:3301-3304`): `LavaDeath = false` (baked `lava_placement = NotAllowed`
/// stands — the Calculate fold only applies when the flag is true) and
/// `DrawYOffset = 2` (draw-only, unported).
const LARGE_PILES_BASE: TileObjectDef = TileObjectDef {
    width: 3,
    height: 2,
    origin: Point16::new(1, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    ..base_def()
};

/// 186's snow/ice sub-styles (`:3291-3294`): 26-31 sit on the snow family,
/// 32-34 on ice/mushroom grass.
const LARGE_PILES_SNOW: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[147, 161, 163, 200, 164, 162, 224],
    ..LARGE_PILES_BASE
};
const LARGE_PILES_ICE: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[59, 70],
    ..LARGE_PILES_BASE
};

/// Tile 186 (`:3285-3295`): the base def (styles 0-25 carry no valid-tile
/// list — any solid anchor passes) plus the two sub-style groups.
const LARGE_PILES: TileObjectDef = TileObjectDef {
    kind: 186,
    subtiles: &[
        (26, &LARGE_PILES_SNOW),
        (27, &LARGE_PILES_SNOW),
        (28, &LARGE_PILES_SNOW),
        (29, &LARGE_PILES_SNOW),
        (30, &LARGE_PILES_SNOW),
        (31, &LARGE_PILES_SNOW),
        (32, &LARGE_PILES_ICE),
        (33, &LARGE_PILES_ICE),
        (34, &LARGE_PILES_ICE),
    ],
    frame_important: true, // `Main.cs:7416`
    ..LARGE_PILES_BASE
};

/// 187's sub-styles (`:3305-3311`): 0-5 jungle grass family, 6-8 ash family,
/// 29-34 the sand family.
const LARGE_PILES2_JUNGLE: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[59, 60, 226],
    ..LARGE_PILES_BASE
};
const LARGE_PILES2_ASH: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[57, 58, 75, 76],
    ..LARGE_PILES_BASE
};
const LARGE_PILES2_SAND: TileObjectDef = TileObjectDef {
    anchor_valid_tiles: &[
        53, 397, 396, 112, 398, 400, 234, 399, 401, 116, 402, 403,
    ],
    ..LARGE_PILES_BASE
};

/// Tile 187 (`:3301-3312`): `Style3x2` + `StyleWrapLimit = 35`,
/// `LavaDeath = false`, three sub-style groups.
const LARGE_PILES2: TileObjectDef = TileObjectDef {
    kind: 187,
    style_wrap_limit: 35,
    subtiles: &[
        (0, &LARGE_PILES2_JUNGLE),
        (1, &LARGE_PILES2_JUNGLE),
        (2, &LARGE_PILES2_JUNGLE),
        (3, &LARGE_PILES2_JUNGLE),
        (4, &LARGE_PILES2_JUNGLE),
        (5, &LARGE_PILES2_JUNGLE),
        (6, &LARGE_PILES2_ASH),
        (7, &LARGE_PILES2_ASH),
        (8, &LARGE_PILES2_ASH),
        (29, &LARGE_PILES2_SAND),
        (30, &LARGE_PILES2_SAND),
        (31, &LARGE_PILES2_SAND),
        (32, &LARGE_PILES2_SAND),
        (33, &LARGE_PILES2_SAND),
        (34, &LARGE_PILES2_SAND),
    ],
    frame_important: true, // `Main.cs:7417`
    ..LARGE_PILES_BASE
};

// -- candle (33) + water candle (49) — `TileObjectData.cs:2334-2358` ---------

/// `StyleOnTable1x1` (`:2325-2334`): 1x1 on a table surface, heights {20}.
/// The water candle (49) shares the def (`:2351-2357` — heights {20},
/// `DrawYOffset -4`; the draw case covers the offset).
const CANDLE: TileObjectDef = TileObjectDef {
    kind: 33,
    width: 1,
    height: 1,
    anchor_bottom: AnchorData::new(anchor_type::TABLE, 1, 0),
    coordinate_heights: &[20],
    coordinate_width: 16,
    coordinate_padding: 2,
    frame_important: true,
    interaction: Interaction::Candle,
    ..base_def()
};

// -- candelabra (100) — `TileObjectData.cs:3600-3606` ------------------------
// Style2x2 (`:3451-3461`) + `StyleWrapLimitVisualOverride 56` (draw-only),
// `StyleLineSkip 2`, `DrawYOffset 2` (preview-only). Sub-tiles 25/42/40 are
// the metal/lava variants — the style-0 wooden slice only.
const CANDELABRA: TileObjectDef = TileObjectDef {
    kind: 100,
    width: 2,
    height: 2,
    origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS | anchor_type::TABLE, 2, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    style_line_skip: 2,
    frame_important: true,
    ..base_def()
};

// -- chandelier (34) — `TileObjectData.cs:3139-3160` -------------------------
// `Style3x3` (`:3095-3106`: 3x3, origin (1,2), AnchorBottom SWTSS 3) hangs:
// the origin moves to the top-centre cell, the bottom anchor empties, and the
// top anchor takes one solid/side tile in the centre column (checkStart 1).
const CHANDELIER: TileObjectDef = TileObjectDef {
    kind: 34,
    width: 3,
    height: 3,
    origin: Point16::new(1, 0),
    anchor_top: AnchorData::new(anchor_type::SOLID_TILE | anchor_type::SOLID_SIDE, 1, 1),
    anchor_bottom: AnchorData::EMPTY,
    coordinate_heights: &[16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: false,
    style_wrap_limit: 37,
    style_line_skip: 2,
    frame_important: true,
    ..base_def()
};

// -- hanging lantern (42) — `TileObjectData.cs:2441-2461` --------------------
// `Style1x2Top` (`:2432-2441`: 1x2, origin (0,0), `AnchorTop = SolidTile |
// SolidSide | PlanterBox`) + the platform alternate (`addAlternate(0)`,
// `DrawYOffset -10` preview-only).
const HANGING_LANTERN: TileObjectDef = TileObjectDef {
    kind: 42,
    width: 1,
    height: 2,
    origin: Point16::new(0, 0),
    anchor_top: AnchorData::new(
        anchor_type::SOLID_TILE | anchor_type::SOLID_SIDE | anchor_type::PLANTER_BOX,
        1,
        0,
    ),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    frame_important: true,
    alternates: &[TileObjectDef {
        anchor_top: AnchorData::new(anchor_type::PLATFORM, 1, 0),
        ..base_def()
    }],
    ..base_def()
};

// -- lamp (93) — `TileObjectData.cs:2403-2418` -------------------------------
// `Style1xX` (`:2392-2403`: 1x3, origin (0,2), AnchorBottom SWTSS 1);
// `StyleWrapLimitVisualOverride 37` is draw-only. Sub-tiles 23/42/40 are the
// metal variants.
const LAMP: TileObjectDef = TileObjectDef {
    kind: 93,
    width: 1,
    height: 3,
    origin: Point16::new(0, 2),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 1, 0),
    coordinate_heights: &[16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    frame_important: true,
    ..base_def()
};

// -- bookcase (101) — `TileObjectData.cs:3384-3400` --------------------------
// `Style3x4` (`:3384-3393`): 3x4, origin (1,3), AnchorBottom SWTSS 3.
const BOOKCASE: TileObjectDef = TileObjectDef {
    kind: 101,
    width: 3,
    height: 4,
    origin: Point16::new(1, 3),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    ..base_def()
};

// -- piano (87) — `TileObjectData.cs:3240-3249` ------------------------------
// `Style3x2` (the table's preset, `:3179-3191`) with the default heights.
const PIANO: TileObjectDef = TileObjectDef {
    kind: 87,
    width: 3,
    height: 2,
    origin: Point16::new(1, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    ..base_def()
};

// -- dresser (88) — `TileObjectData.cs:3199-3209` ----------------------------
// Style3x2 + the chest's invalid-anchor tiles + `LavaDeath = false`
// (the placement hooks are the M3-CHEST-UI seam).
const DRESSER: TileObjectDef = TileObjectDef {
    kind: 88,
    width: 3,
    height: 2,
    origin: Point16::new(1, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 3, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    anchor_invalid_tiles: CHEST_ANCHOR_INVALID,
    lava_placement: LiquidPlacement::Allowed,
    frame_important: true,
    ..base_def()
};

// -- sink (172) — `TileObjectData.cs:3577-3583` ------------------------------
// Style2x2 + the 18px bottom row (`StyleWrapLimitVisualOverride 53` is
// draw-only).
const SINK: TileObjectDef = TileObjectDef {
    kind: 172,
    width: 2,
    height: 2,
    origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16, 18],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    ..base_def()
};

// -- grandfather clock (104) — `TileObjectData.cs:3024-3035` -----------------
// `Style2xX` (`:2861-2870`: 2x3, origin (1,2), AnchorBottom SWTSS 2,
// horizontal) grown to 5 tall with the origin moved to the top-left cell and
// `DrawYOffset 2` (preview-only).
const GRANDFATHER_CLOCK: TileObjectDef = TileObjectDef {
    kind: 104,
    width: 2,
    height: 5,
    origin: Point16::new(0, 4),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16, 16, 16, 16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    style_horizontal: true,
    frame_important: true,
    interaction: Interaction::Clock,
    ..base_def()
};

// -- bed (79) — `TileObjectData.cs:2606-2645` --------------------------------
// `Style4x2` (`:2555-2568`: 4x2, origin (1,1), AnchorBottom SWTSS 4,
// `PlaceLeft` + the `PlaceRight` alternate, `StyleMultiplier/WrapLimit 2`) +
// the 18px bottom row + `CoordinatePaddingFix (0, -2)`. Sub-tiles 8/42/56-64
// are the other bed colours.
const BED_BASE: TileObjectDef = TileObjectDef {
    width: 4,
    height: 2,
    origin: Point16::new(1, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 4, 0),
    coordinate_heights: &[16, 18],
    coordinate_width: 16,
    coordinate_padding: 2,
    coordinate_padding_fix: Point16::new(0, -2),
    direction: TileObjectDirection::PlaceLeft,
    style_horizontal: true,
    style_wrap_limit: 2,
    style_multiplier: 2,
    frame_important: true,
    interaction: Interaction::Bed,
    ..base_def()
};
const BED: TileObjectDef = TileObjectDef {
    kind: 79,
    alternates: &[TileObjectDef {
        direction: TileObjectDirection::PlaceRight,
        style: 1, // `addAlternate(1)` (`TileObjectData.cs:2567`)
        ..BED_BASE
    }],
    ..BED_BASE
};

// -- bathtub (90) — `TileObjectData.cs:2568-2605` ----------------------------
// `Style4x2` verbatim (heights {16,16}, no padding fix).
const BATHTUB: TileObjectDef = TileObjectDef {
    kind: 90,
    width: 4,
    height: 2,
    origin: Point16::new(1, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 4, 0),
    coordinate_heights: &[16, 16],
    coordinate_width: 16,
    coordinate_padding: 2,
    direction: TileObjectDirection::PlaceLeft,
    style_horizontal: true,
    style_wrap_limit: 2,
    style_multiplier: 2,
    frame_important: true,
    alternates: &[TileObjectDef {
        direction: TileObjectDirection::PlaceRight,
        style: 1, // `addAlternate(1)`
        ..base_def()
    }],
    ..base_def()
};

/// `TileObjectData.GetTileData(type)` — the tile's base definition
/// (`TileObjectData.cs:5126-5194` without the sub-tile/alternate walk).
pub fn def_for_tile(kind: u16) -> Option<&'static TileObjectDef> {
    TABLE.iter().find(|def| def.kind == kind)
}

/// `TileObjectData.GetTileData(type, style, alternate)` for an already-resolved
/// tile def (`TileObjectData.cs:5126-5194`): `style` picks a sub-tile,
/// `alternate - 1` indexes the alternates (`alternate = 0` → the def itself,
/// which is how the `CanPlace` loop addresses the base def).
pub fn get_tile_data(
    def: &'static TileObjectDef,
    style: i32,
    alternate: i32,
) -> Option<&'static TileObjectDef> {
    debug_assert!(style >= 0, "GetTileData throws on a negative style (:5129)");
    if style < 0 {
        return None;
    }
    let mut resolved = def;
    // :5139-5147 — `SubTiles[style]` when present (the C# list is null-padded;
    // the port stores the non-null entries as (style, def) pairs).
    if let Some((_, sub)) = resolved
        .subtiles
        .iter()
        .find(|(sub_style, _)| *sub_style as i32 == style)
    {
        resolved = sub;
    }
    // :5148-5157 — `alternate--`, then index the alternates.
    let alt = alternate - 1;
    if alt >= 0 {
        resolved = resolved.alternates.get(alt as usize)?;
    }
    Some(resolved)
}

/// `TileObjectData.CalculatePlacementStyle` (`TileObjectData.cs:1990-1998`).
/// The `alternate` parameter is part of the C# signature but its body never
/// reads it — kept 1:1.
#[allow(unused_variables)]
pub fn calculate_placement_style(
    def: &TileObjectDef,
    style: i32,
    alternate: i32,
    random: i32,
) -> i32 {
    let mut placement = style * def.style_multiplier + def.style;
    if random >= 0 {
        placement += random;
    }
    placement
}

// ---------------------------------------------------------------- helpers --

/// `Tile.nactive` (`Tile.cs:240-246`): active and not `inActive` (framed off).
pub(crate) fn nactive(tile: &Tile) -> bool {
    tile.active() && tile.flags & crate::tile::FLAG_INACTIVE == 0
}

/// `Tile.blockType` (`Tile.cs:202-214`): 0 flat, 1 half brick, 2..=5 slopes.
pub(crate) fn block_type(tile: &Tile) -> i32 {
    if tile.half_brick() {
        return 1;
    }
    let slope = tile.slope() as i32;
    if slope > 0 {
        slope + 1
    } else {
        0
    }
}

/// `TileObjectData.isValidTileAnchor` (`TileObjectData.cs:1910-1946`).
fn is_valid_tile_anchor(def: &TileObjectDef, kind: u16) -> bool {
    if def.anchor_invalid_tiles.contains(&kind) {
        return false;
    }
    if def.anchor_valid_tiles.is_empty() {
        return true;
    }
    def.anchor_valid_tiles.contains(&kind)
}

/// `TileObjectData.isValidWallAnchor` (`TileObjectData.cs:1948-1967`).
fn is_valid_wall_anchor(def: &TileObjectDef, wall: u16) -> bool {
    if def.anchor_valid_walls.is_empty() {
        return wall != 0;
    }
    def.anchor_valid_walls.contains(&wall)
}

/// `TileObjectData.isValidAlternateAnchor` (`TileObjectData.cs:1969-1988`).
fn is_valid_alternate_anchor(def: &TileObjectDef, kind: u16) -> bool {
    def.anchor_alternate_tiles.contains(&kind)
}

/// `TileObjectData.LiquidPlace` (`TileObjectData.cs:1853-1908`). `liquid_type`:
/// 0 water, 1 lava, 2 honey, 3 shimmer (`Tile.liquidType`, `Tile.cs:226-234`).
fn liquid_place(def: &TileObjectDef, tile: &Tile) -> bool {
    let (placement, full_only) = match tile.liquid_kind() {
        crate::tile::LiquidKind::Lava => (def.lava_placement, def.lava_placement),
        _ => (def.water_placement, def.water_placement),
    };
    if tile.liquid > 0 {
        // :1862-1885
        placement != LiquidPlacement::NotAllowed
            && (placement != LiquidPlacement::OnlyInFullLiquid || tile.liquid == u8::MAX)
    } else {
        // :1887-1906 — an empty liquid slot only fails "OnlyIn…" requirements.
        !matches!(
            full_only,
            LiquidPlacement::OnlyInFullLiquid | LiquidPlacement::OnlyInLiquid
        )
    }
}

/// Membership in [`TILE_CUT`] — read by the placement cut rule
/// (`Player.cs:41303`) and the door sweep (`WorldGen.cs:37704-37717`).
pub fn tile_cut(kind: u16) -> bool {
    TILE_CUT.contains(&kind)
}

/// Membership in [`BREAKABLE_WHEN_PLACING`] (`Player.cs:41303`).
pub fn breakable_when_placing(kind: u16) -> bool {
    BREAKABLE_WHEN_PLACING.contains(&kind)
}

/// Membership in [`IS_A_DRIP_TILE`] (`Player.cs:41310`).
pub fn is_drip_tile(kind: u16) -> bool {
    IS_A_DRIP_TILE.contains(&kind)
}

/// Membership in [`IS_VINE`] (`Player.cs:41309`).
pub fn is_vine(kind: u16) -> bool {
    IS_VINE.contains(&kind)
}

pub(crate) fn not_really_solid(kind: u16) -> bool {
    NOT_REALLY_SOLID.contains(&kind)
}

/// `WorldGen.CanCutTile` (`WorldGen.cs:86828-86842`, the
/// `TileCuttingContext.TilePlacement` arm — the context does not reach the
/// body): the cell may be cut during placement when the tile below is not
/// a pot/planter-box/seed-provider seat (78/380/579), the wall behind is
/// not the unsafe dungeon wall 350, and a pumpkin (254) is ripe
/// (`frameX >= 144` — the grown frame).
pub(crate) fn can_cut_tile(world: &World, x: i32, y: i32) -> bool {
    let (Some(here), Some(below)) = (world.tile(x, y), world.tile(x, y + 1)) else {
        return false;
    };
    if here.wall != 350 && below.kind != 78 && below.kind != 380 && below.kind != 579 {
        if here.kind == 254 {
            return here.frame_x >= 144;
        }
        return true;
    }
    false
}

pub(crate) fn is_tile_table(kind: u16) -> bool {
    is_table(kind)
}

/// `WorldGen.IsOpenDoorAnchorFrame` (`WorldGen.cs:69171-69184`): the open
/// door's 11 (framed off, type 11) whose frame column anchors a wall grid
/// point — the columns 0..17 (the top frame) and 54..71 (the bottom).
pub(crate) fn is_open_door_anchor_frame(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() || t.kind != 11 {
        return false;
    }
    let num = t.frame_x % 72;
    if num >= 18 {
        return num >= 54;
    }
    true
}

pub(crate) fn is_table(kind: u16) -> bool {
    TILE_TABLE.contains(&kind)
}

fn is_platform(kind: u16) -> bool {
    crate::tiledata::SET_PLATFORMS.has(kind)
}

fn is_solid(kind: u16) -> bool {
    crate::tiledata::TILE_SOLID.has(kind)
}

fn is_solid_top(kind: u16) -> bool {
    crate::tiledata::TILE_SOLID_TOP.has(kind)
}

fn no_attach(kind: u16) -> bool {
    crate::torch::NO_ATTACH.contains(&kind)
}

fn is_tree_trunk(kind: u16) -> bool {
    crate::torch::TREE_TRUNKS.contains(&kind)
}

// ---------------------------------------------------------------- CanPlace --

/// `TileObject.CanPlace` (`TileObject.cs:174-853`), without the `onlyCheck`
/// preview machinery (see the module doc). Returns the resolved placement the
/// C# version writes into `objectData`, or `None` when the object cannot be
/// placed. `dir` is the player direction (`Player.direction`: -1 left,
/// 1 right).
pub fn can_place(
    world: &World,
    x: i32,
    y: i32,
    def: &'static TileObjectDef,
    style: i32,
    dir: i32,
    forced_random: Option<i32>,
) -> Option<TileObjectInstance> {
    // :176-181 — the style-resolved base def.
    let base = get_tile_data(def, style, 0)?;
    // :182-187 — coarse bounds check against the base def.
    let num = x - base.origin.x;
    let num2 = y - base.origin.y;
    if num < 0 || num + base.width >= world.width || num2 < 0 || num2 + base.height >= world.height
    {
        return None;
    }
    // :188
    let has_random_styles = base.random_style_range > 0;
    // :194-198 — the alternate loop covers the base def (num7 = 0) and every
    // registered alternate (num7 = 1..=AlternatesCount).
    let alternates_count = base.alternates.len() as i32;

    // :199-204 — best-alternate tracking (f32 ratios exactly like C#).
    let mut best_anchor = -1.0f32;
    let mut best_free = -1.0f32;
    let mut best_alternate = 0i32;
    let mut winning: Option<&'static TileObjectDef> = None;
    // :204 — `flag2`
    let mut saw_imperfect = false;

    let mut num7 = -1i32;
    while num7 < alternates_count {
        num7 += 1;
        let tile_data = get_tile_data(def, style, num7)?;
        // :209-212 — direction filter: a def facing one way is skipped when the
        // player faces the other.
        if tile_data.direction != TileObjectDirection::None
            && ((tile_data.direction == TileObjectDirection::PlaceLeft && dir == 1)
                || (tile_data.direction == TileObjectDirection::PlaceRight && dir == -1))
        {
            continue;
        }
        // :213-218 — every alternate must sit 5 tiles clear of the world edge
        // (`return false`, not `continue`).
        let num8 = x - tile_data.origin.x;
        let num9 = y - tile_data.origin.y;
        if num8 < 5
            || num8 + tile_data.width > world.width - 5
            || num9 < 5
            || num9 + tile_data.height > world.height - 5
        {
            return None;
        }
        // :219-321 — preview-only `Rectangle` bookkeeping, not ported (module doc).

        // :333-375 — score the object cells.
        let mut num22 = 0.0f32; // free cells
        let mut num23 = (tile_data.width * tile_data.height) as f32;
        let mut num24 = 0.0f32; // satisfied anchor/wall requirements
        let mut num25 = 0.0f32; // required anchor/wall checks
        for i in 0..tile_data.width {
            for j in 0..tile_data.height {
                let Some(tile) = world.tile(num8 + i, num9 + j) else {
                    // Framing.GetTileSafely never fails; outside the world this
                    // port treats the cell as an inactive empty tile.
                    continue;
                };
                let flag3 = !liquid_place(tile_data, tile);
                let mut flag4 = false;
                if tile_data.anchor_wall {
                    num25 += 1.0;
                    if !is_valid_wall_anchor(tile_data, tile.wall) {
                        flag4 = true;
                    } else {
                        num24 += 1.0;
                    }
                }
                // :357-360 — occupied cells block, unless they would be cut
                // away by `Place` (484/654 are exempt from the cut exemption).
                let flag5 = tile.active()
                    && (!tile_cut(tile.kind) || tile.kind == 484 || tile.kind == 654)
                    && !breakable_when_placing(tile.kind);
                if flag5 | flag3 | flag4 {
                    continue;
                }
                num22 += 1.0;
            }
        }

        // :376-438 — AnchorBottom (row `height` below the object).
        let anchor = tile_data.anchor_bottom;
        if anchor.tile_count != 0 {
            num25 += anchor.tile_count as f32;
            let height = tile_data.height;
            for k in 0..anchor.tile_count {
                let num26 = anchor.check_start + k;
                let Some(tile) = world.tile(num8 + num26, num9 + height) else {
                    continue;
                };
                let mut flag6 = false;
                if nactive(tile) {
                    // :388-391
                    if (anchor.ty & anchor_type::SOLID_TILE) == anchor_type::SOLID_TILE
                        && is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                        && !no_attach(tile.kind)
                        && (tile_data.flatten_anchors || block_type(tile) == 0)
                    {
                        flag6 = is_valid_tile_anchor(tile_data, tile.kind);
                    }
                    // :392-406 — SolidWithTop / Table: platforms with a proper
                    // top frame, or solid-top tiles.
                    if !flag6
                        && ((anchor.ty & anchor_type::SOLID_WITH_TOP) != 0
                            || (anchor.ty & anchor_type::TABLE) != 0)
                    {
                        if is_platform(tile.kind) {
                            if !tile.half_brick()
                                && crate::torch::platform_proper_top_frame(tile.frame_x)
                            {
                                flag6 = true;
                            }
                        } else if is_solid(tile.kind) && is_solid_top(tile.kind) {
                            flag6 = true;
                        }
                    }
                    // :407-410
                    if !flag6
                        && (anchor.ty & anchor_type::TABLE) != 0
                        && !is_platform(tile.kind)
                        && is_table(tile.kind)
                        && block_type(tile) == 0
                    {
                        flag6 = true;
                    }
                    // :411-414 — SolidSide: slopes 3/4 (blockType 4/5).
                    if !flag6
                        && (anchor.ty & anchor_type::SOLID_SIDE) != 0
                        && is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                        && matches!(block_type(tile), 4 | 5)
                    {
                        flag6 = is_valid_tile_anchor(tile_data, tile.kind);
                    }
                    // :415-418
                    if !flag6
                        && (anchor.ty & anchor_type::ALTERNATE_TILE) != 0
                        && is_valid_alternate_anchor(tile_data, tile.kind)
                    {
                        flag6 = true;
                    }
                } else if (anchor.ty & anchor_type::EMPTY_TILE) != 0 {
                    // :420-423
                    flag6 = true;
                }
                if !flag6 {
                    continue;
                }
                num24 += 1.0;
            }
        }

        // :439-543 — AnchorTop (row -1 above the object).
        let anchor = tile_data.anchor_top;
        let mut num27 = -1i32; // :439 — AllFlatHeight tracker
        if anchor.tile_count != 0 {
            num25 += anchor.tile_count as f32;
            let num28 = -1i32;
            for l in 0..anchor.tile_count {
                let num30 = anchor.check_start + l;
                let Some(tile) = world.tile(num8 + num30, num9 + num28) else {
                    continue;
                };
                let mut flag7 = false;
                let mut num29 = -1i32;
                if nactive(tile) {
                    // :453-460
                    if is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                        && !no_attach(tile.kind)
                        && (tile_data.flatten_anchors || block_type(tile) == 0)
                    {
                        flag7 = is_valid_tile_anchor(tile_data, tile.kind);
                        if flag7 {
                            num29 = 0;
                        }
                    }
                    // :461-468 — SolidBottom: anything you can stand on.
                    if !flag7
                        && (anchor.ty & anchor_type::SOLID_BOTTOM) == anchor_type::SOLID_BOTTOM
                        && ((is_solid(tile.kind)
                            && (!is_solid_top(tile.kind)
                                || (is_platform(tile.kind)
                                    && (tile.half_brick() || tile.top_slope()))))
                            || tile.half_brick()
                            || tile.top_slope())
                        && !not_really_solid(tile.kind)
                        && !tile.bottom_slope()
                    {
                        flag7 = is_valid_tile_anchor(tile_data, tile.kind);
                        if flag7 {
                            num29 = 0;
                        }
                    }
                    // :469-476 — Platform (hammered tops get height 8).
                    if !flag7
                        && (anchor.ty & anchor_type::PLATFORM) == anchor_type::PLATFORM
                        && is_platform(tile.kind)
                    {
                        flag7 = is_valid_tile_anchor(tile_data, tile.kind);
                        if flag7 {
                            num29 = if !tile.half_brick() && !tile.top_slope() {
                                8
                            } else {
                                0
                            };
                        }
                    }
                    // :477-484 — PlatformNonHammered.
                    if !flag7
                        && (anchor.ty & anchor_type::PLATFORM_NON_HAMMERED)
                            == anchor_type::PLATFORM_NON_HAMMERED
                        && is_platform(tile.kind)
                        && tile.slope() == 0
                        && !tile.half_brick()
                    {
                        flag7 = is_valid_tile_anchor(tile_data, tile.kind);
                        if flag7 {
                            num29 = 8;
                        }
                    }
                    // :485-492 — PlanterBox (tile 380).
                    if !flag7
                        && (anchor.ty & anchor_type::PLANTER_BOX) == anchor_type::PLANTER_BOX
                        && tile.kind == 380
                    {
                        flag7 = is_valid_tile_anchor(tile_data, tile.kind);
                        if flag7 {
                            num29 = 0;
                        }
                    }
                    // :493-503 — SolidSide: slopes 1/2 (blockType 2/3).
                    if !flag7
                        && (anchor.ty & anchor_type::SOLID_SIDE) == anchor_type::SOLID_SIDE
                        && is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                    {
                        if matches!(block_type(tile), 2 | 3) {
                            flag7 = is_valid_tile_anchor(tile_data, tile.kind);
                        }
                        if flag7 {
                            num29 = 0;
                        }
                    }
                    // :504-511
                    if !flag7
                        && (anchor.ty & anchor_type::ALTERNATE_TILE) == anchor_type::ALTERNATE_TILE
                        && is_valid_alternate_anchor(tile_data, tile.kind)
                    {
                        flag7 = true;
                        num29 = 0;
                    }
                } else if (anchor.ty & anchor_type::EMPTY_TILE) != 0 {
                    // :513-517
                    flag7 = true;
                    num29 = 0;
                }
                // :518-528 — every AllFlatHeight anchor must share one height.
                if flag7
                    && (anchor.ty & anchor_type::ALL_FLAT_HEIGHT) == anchor_type::ALL_FLAT_HEIGHT
                {
                    if num27 == -1 {
                        num27 = num29;
                    }
                    if num27 != num29 {
                        flag7 = false;
                    }
                }
                if !flag7 {
                    continue;
                }
                num24 += 1.0;
            }
        }

        // :544-629 — AnchorRight (column `width` right of the object).
        let anchor = tile_data.anchor_right;
        if anchor.tile_count != 0 {
            num25 += anchor.tile_count as f32;
            let width = tile_data.width;
            for m in 0..anchor.tile_count {
                let num31 = anchor.check_start + m;
                let Some(tile) = world.tile(num8 + width, num9 + num31) else {
                    continue;
                };
                let mut flag8 = false;
                if nactive(tile) {
                    // :556-559
                    if is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                        && !no_attach(tile.kind)
                        && (tile_data.flatten_anchors || block_type(tile) == 0)
                    {
                        flag8 = is_valid_tile_anchor(tile_data, tile.kind);
                    }
                    // :560-567 — SolidSide: blockType 2 or 4.
                    if !flag8
                        && (anchor.ty & anchor_type::SOLID_SIDE) == anchor_type::SOLID_SIDE
                        && is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                    {
                        if matches!(block_type(tile), 2 | 4) {
                            flag8 = is_valid_tile_anchor(tile_data, tile.kind);
                        }
                    }
                    // :568-605 — Tree trunks count, plus the tiles above/below
                    // the anchor run when it starts/ends there.
                    if !flag8
                        && (anchor.ty & anchor_type::TREE) == anchor_type::TREE
                        && is_tree_trunk(tile.kind)
                    {
                        flag8 = true;
                        if m == 0 {
                            num25 += 1.0;
                            if let Some(above) = world.tile(num8 + width, num9 + num31 - 1) {
                                if nactive(above) && is_tree_trunk(above.kind) {
                                    num24 += 1.0;
                                }
                            }
                        }
                        if m == anchor.tile_count - 1 {
                            num25 += 1.0;
                            if let Some(below) = world.tile(num8 + width, num9 + num31 + 1) {
                                if nactive(below) && is_tree_trunk(below.kind) {
                                    num24 += 1.0;
                                }
                            }
                        }
                    }
                    // :606-609
                    if !flag8
                        && (anchor.ty & anchor_type::ALTERNATE_TILE) == anchor_type::ALTERNATE_TILE
                        && is_valid_alternate_anchor(tile_data, tile.kind)
                    {
                        flag8 = true;
                    }
                } else if (anchor.ty & anchor_type::EMPTY_TILE) != 0 {
                    // :611-614
                    flag8 = true;
                }
                if !flag8 {
                    continue;
                }
                num24 += 1.0;
            }
        }

        // :630-715 — AnchorLeft (column -1 left of the object).
        let anchor = tile_data.anchor_left;
        if anchor.tile_count != 0 {
            num25 += anchor.tile_count as f32;
            let num33 = -1i32;
            for n in 0..anchor.tile_count {
                let num34 = anchor.check_start + n;
                let Some(tile) = world.tile(num8 + num33, num9 + num34) else {
                    continue;
                };
                let mut flag9 = false;
                if nactive(tile) {
                    // :642-645
                    if is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                        && !no_attach(tile.kind)
                        && (tile_data.flatten_anchors || block_type(tile) == 0)
                    {
                        flag9 = is_valid_tile_anchor(tile_data, tile.kind);
                    }
                    // :646-653 — SolidSide: blockType 3 or 5.
                    if !flag9
                        && (anchor.ty & anchor_type::SOLID_SIDE) == anchor_type::SOLID_SIDE
                        && is_solid(tile.kind)
                        && !is_solid_top(tile.kind)
                        && matches!(block_type(tile), 3 | 5)
                    {
                        flag9 = is_valid_tile_anchor(tile_data, tile.kind);
                    }
                    // :654-691 — Tree, as on the right.
                    if !flag9
                        && (anchor.ty & anchor_type::TREE) == anchor_type::TREE
                        && is_tree_trunk(tile.kind)
                    {
                        flag9 = true;
                        if n == 0 {
                            num25 += 1.0;
                            if let Some(above) = world.tile(num8 + num33, num9 + num34 - 1) {
                                if nactive(above) && is_tree_trunk(above.kind) {
                                    num24 += 1.0;
                                }
                            }
                        }
                        if n == anchor.tile_count - 1 {
                            num25 += 1.0;
                            if let Some(below) = world.tile(num8 + num33, num9 + num34 + 1) {
                                if nactive(below) && is_tree_trunk(below.kind) {
                                    num24 += 1.0;
                                }
                            }
                        }
                    }
                    // :692-695
                    if !flag9
                        && (anchor.ty & anchor_type::ALTERNATE_TILE) == anchor_type::ALTERNATE_TILE
                        && is_valid_alternate_anchor(tile_data, tile.kind)
                    {
                        flag9 = true;
                    }
                } else if (anchor.ty & anchor_type::EMPTY_TILE) != 0 {
                    // :697-700
                    flag9 = true;
                }
                if !flag9 {
                    continue;
                }
                num24 += 1.0;
            }
        }

        // :716-729 — the can-place hook (none installed in the wooden slice).
        if let Some(hook) = tile_data.hook_check_if_can_place.hook {
            if hook(world, x, y, def.kind, style, dir, num7)
                == tile_data.hook_check_if_can_place.bad_return
                && tile_data.hook_check_if_can_place.bad_response == 0
            {
                num24 = 0.0;
                num22 = 0.0;
            }
        }

        // :730-742 — the ratios (guards exactly as written; the `num23`/`num25`
        // writes mirror the C# defensively and are dead by :743).
        let mut num35 = num24 / num25;
        if num25 == 0.0 {
            num35 = 1.0;
        }
        let mut num36 = num22 / num23;
        if num36 == 1.0 && num25 == 0.0 {
            #[allow(unused_assignments)]
            {
                num23 = 1.0;
                num25 = 1.0;
            }
            num35 = 1.0;
            num36 = 1.0;
        }
        // :743-750 — a perfect alternate wins immediately.
        if num35 == 1.0 && num36 == 1.0 {
            best_anchor = 1.0;
            best_free = 1.0;
            best_alternate = num7;
            winning = Some(tile_data);
            break;
        }
        // :751-759 — otherwise keep the best-scoring alternate. The `&` is
        // non-short-circuit in C# and stays that way here.
        if !((num35 < 1.0) & saw_imperfect)
            && (num35 > best_anchor || (num35 == best_anchor && num36 > best_free))
        {
            saw_imperfect = true;
            best_anchor = num35;
            best_free = num36;
            winning = Some(tile_data);
            best_alternate = num7;
        }
    }

    // :761-793 — random styles. The `Main.rand`/`randomCache` path needs app
    // wiring (module doc); `forced_random` ports the deterministic branch.
    let mut num37 = -1i32;
    if has_random_styles {
        num37 = match forced_random {
            // :788-791
            Some(forced) => forced.rem_euclid(base.random_style_range),
            // :787 — Main.rand.Next(range): follow-up.
            None => -1,
        };
    }
    // :794-825 — SpecificRandomStyles (same follow-up for the rand path).
    if !base.specific_random_styles.is_empty() {
        num37 = match forced_random {
            Some(forced) => {
                // :820-823
                let count = base.specific_random_styles.len() as i32;
                let idx = forced.rem_euclid(count);
                // :824
                base.specific_random_styles[idx as usize] - style
            }
            None => -1,
        };
    }

    // :839-852 — resolve the instance and decide.
    let winning = winning?; // every alternate filtered out — neutral refusal
    if best_anchor == 1.0 {
        if best_free != 1.0 {
            return None;
        }
        return Some(TileObjectInstance {
            x_coord: x - winning.origin.x,
            y_coord: y - winning.origin.y,
            kind: def.kind,
            style,
            alternate: best_alternate,
            random: num37,
        });
    }
    None
}

// ------------------------------------------------------------------- Place --

/// `WorldGen.SlopeTile(i, j)` (`WorldGen.cs:80768-80772`) — the world-state
/// effect `Place`'s FlattenAnchors pass needs: clear half brick and slope.
/// The sound / player-push / networking side of `SlopeTile` is app-scope, and
/// its `CanPoundTile` gate is not reproduced (unreachable for the wooden slice:
/// every wooden def has `flatten_anchors = false`; revisit with the full
/// registry).
fn slope_tile_flatten(world: &mut World, x: i32, y: i32) {
    if let Some(tile) = world.tile_mut(x, y) {
        tile.flags &= !crate::tile::FLAG_HALF_BRICK;
        tile.set_slope(0);
    }
}

/// `TileObject.Place` (`TileObject.cs:29-172`). Writes the whole object (tiles,
/// frames, `world.frame_important`), kills cut/breakable tiles under it and
/// flattens anchored slopes for `FlattenAnchors` defs. `false` = the def does
/// not resolve (no world change).
pub fn place(world: &mut World, to_be_placed: &TileObjectInstance) -> bool {
    // :31-35 — `GetTileData(type, style, alternate)` resolution.
    let Some(def) = def_for_tile(to_be_placed.kind) else {
        return false;
    };
    let Some(tile_data) = get_tile_data(def, to_be_placed.style, to_be_placed.alternate) else {
        return false;
    };
    place_with_def(world, tile_data, to_be_placed)
}

/// The body of `TileObject.Place` after the `GetTileData` resolution (:35).
/// Split out so callers that already hold the resolved def (and tests with
/// defs outside [`TABLE`]) can run it 1:1.
pub fn place_with_def(
    world: &mut World,
    tile_data: &TileObjectDef,
    to_be_placed: &TileObjectInstance,
) -> bool {
    if tile_data.hook_place_override.hook.is_some() {
        // :36-54 — the override hook does the placement itself (none installed
        // in the wooden slice; the branch is ported for the full registry).
        let hook = tile_data.hook_place_override;
        let (x, y) = if hook.processed_coordinates {
            (to_be_placed.x_coord, to_be_placed.y_coord)
        } else {
            (
                to_be_placed.x_coord + tile_data.origin.x,
                to_be_placed.y_coord + tile_data.origin.y,
            )
        };
        if hook.hook.unwrap()(
            world,
            x,
            y,
            to_be_placed.kind,
            to_be_placed.style,
            1,
            to_be_placed.alternate,
        ) == hook.bad_return
        {
            return false;
        }
    } else {
        // :56-110
        let kind = to_be_placed.kind;
        let num6 = to_be_placed.x_coord;
        let num7 = to_be_placed.y_coord;
        let mut num4 = calculate_placement_style(
            tile_data,
            to_be_placed.style,
            to_be_placed.alternate,
            to_be_placed.random,
        );
        let mut num5 = 0;
        // :62-66
        if tile_data.style_wrap_limit > 0 {
            num5 = num4 / tile_data.style_wrap_limit * tile_data.style_line_skip;
            num4 %= tile_data.style_wrap_limit;
        }
        // :67-76
        let (num2, num3) = if tile_data.style_horizontal {
            (
                tile_data.coordinate_full_width() * num4,
                tile_data.coordinate_full_height() * num5,
            )
        } else {
            (
                tile_data.coordinate_full_width() * num5,
                tile_data.coordinate_full_height() * num4,
            )
        };
        debug_assert_eq!(
            tile_data.coordinate_heights.len(),
            tile_data.height as usize,
            "CoordinateHeights must cover every row (:2255/:3174/...)"
        );
        // :79-93 — cut/breakable tiles under the object are killed first. The
        // world-state effect of `WorldGen.KillTile` on such tiles is the bare
        // removal plus a neighbour re-frame (`edit::remove_tile` +
        // `reframe_area`); the full KillTile port lands in Phase 3.
        for i in 0..tile_data.width {
            for j in 0..tile_data.height {
                let Some(tile) = world.tile(num6 + i, num7 + j) else {
                    continue;
                };
                if tile.active()
                    && tile.kind != 484
                    && (tile_cut(tile.kind) || breakable_when_placing(tile.kind))
                {
                    crate::edit::remove_tile(world, num6 + i, num7 + j);
                    crate::framing::reframe_area(world, num6 + i, num7 + j, 2);
                }
            }
        }
        // Mirror `Main.tileFrameImportant[type]` into the world so `framing`
        // keeps the stored frames of every tile written below (DW-1.3;
        // `Main.cs:7637-7656` for the wooden set).
        if tile_data.frame_important {
            if world.frame_important.len() <= kind as usize {
                world
                    .frame_important
                    .resize(crate::tiledata::TILE_COUNT, false);
            }
            world.frame_important[kind as usize] = true;
        }
        // :94-110 — write the object: only inactive cells are filled, keeping
        // wall and liquid (vanilla lets furniture sit in water).
        for k in 0..tile_data.width {
            let num8 = num2 + k * (tile_data.coordinate_width + tile_data.coordinate_padding);
            let mut num9 = num3;
            for l in 0..tile_data.height {
                if let Some(tile) = world.tile_mut(num6 + k, num7 + l) {
                    if !tile.active() {
                        tile.flags |= crate::tile::FLAG_ACTIVE;
                        tile.frame_x = num8 as i16;
                        tile.frame_y = num9 as i16;
                        tile.kind = kind;
                        world.mark_chunk(num6 + k, num7 + l);
                    }
                }
                // :108
                num9 += tile_data
                    .coordinate_heights
                    .get(l as usize)
                    .copied()
                    .unwrap_or(0)
                    + tile_data.coordinate_padding;
            }
        }
        world.revision = world.revision.wrapping_add(1);
    }
    // :112-170 — FlattenAnchors: level the slopes the object sits on/against.
    if tile_data.flatten_anchors {
        // :114-127 — AnchorBottom.
        let anchor = tile_data.anchor_bottom;
        if anchor.tile_count != 0
            && (anchor.ty & anchor_type::SOLID_TILE) == anchor_type::SOLID_TILE
        {
            let num10 = to_be_placed.x_coord + anchor.check_start;
            let j2 = to_be_placed.y_coord + tile_data.height;
            for m in 0..anchor.tile_count {
                if let Some(tile) = world.tile(num10 + m, j2) {
                    if is_solid(tile.kind) && !is_solid_top(tile.kind) && block_type(tile) != 0 {
                        slope_tile_flatten(world, num10 + m, j2);
                    }
                }
            }
        }
        // :128-141 — AnchorTop (row -1).
        let anchor = tile_data.anchor_top;
        if anchor.tile_count != 0
            && (anchor.ty & anchor_type::SOLID_TILE) == anchor_type::SOLID_TILE
        {
            let num11 = to_be_placed.x_coord + anchor.check_start;
            let j3 = to_be_placed.y_coord - 1;
            for n in 0..anchor.tile_count {
                if let Some(tile) = world.tile(num11 + n, j3) {
                    if is_solid(tile.kind) && !is_solid_top(tile.kind) && block_type(tile) != 0 {
                        slope_tile_flatten(world, num11 + n, j3);
                    }
                }
            }
        }
        // :142-155 — AnchorRight (column `width`).
        let anchor = tile_data.anchor_right;
        if anchor.tile_count != 0
            && (anchor.ty & anchor_type::SOLID_TILE) == anchor_type::SOLID_TILE
        {
            let i2 = to_be_placed.x_coord + tile_data.width;
            let num12 = to_be_placed.y_coord + anchor.check_start;
            for num13 in 0..anchor.tile_count {
                if let Some(tile) = world.tile(i2, num12 + num13) {
                    if is_solid(tile.kind) && !is_solid_top(tile.kind) && block_type(tile) != 0 {
                        slope_tile_flatten(world, i2, num12 + num13);
                    }
                }
            }
        }
        // :156-169 — AnchorLeft (column -1).
        let anchor = tile_data.anchor_left;
        if anchor.tile_count != 0
            && (anchor.ty & anchor_type::SOLID_TILE) == anchor_type::SOLID_TILE
        {
            let i3 = to_be_placed.x_coord - 1;
            let num14 = to_be_placed.y_coord + anchor.check_start;
            for num15 in 0..anchor.tile_count {
                if let Some(tile) = world.tile(i3, num14 + num15) {
                    if is_solid(tile.kind) && !is_solid_top(tile.kind) && block_type(tile) != 0 {
                        slope_tile_flatten(world, i3, num14 + num15);
                    }
                }
            }
        }
    }
    // :171
    true
}

/// One multi-tile object instance located in the world: its registry def and
/// the top-left cell of its bounding box.
#[derive(Clone, Copy, Debug)]
pub struct ObjectTiles {
    pub def: &'static TileObjectDef,
    /// Top-left of the object in world tiles.
    pub origin: Point16,
}

impl ObjectTiles {
    /// Every cell of the object, row-major origin-relative. May yield
    /// out-of-world coordinates for an object anchored at the border —
    /// callers bounds-check.
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)> {
        let (ox, oy) = (self.origin.x, self.origin.y);
        let (w, h) = (self.def.width, self.def.height);
        (0..h).flat_map(move |j| (0..w).map(move |i| (ox + i, oy + j)))
    }
}

/// Anchor lookup for the kill cascade: resolves the object instance a cell
/// belongs to. The origin is reconstructed from the anchor cell's frames —
/// the same derivation the `WorldGen.Check*` routines perform
/// (`WorldGen.cs:57929-57937` chest, `:86439` closed door, `:86350-86368`
/// open door, `:43343-43349` chair, `:46981-46988` bench, `:49199-49209`
/// table) — generalized through the registry: for every def
/// `coordinate_full_width()/18 % width == 0` and
/// `coordinate_full_height()/18 % height == 0`, so
/// `dx = frameX/18 % width`, `dy = frameY/18 % height` reproduce each C# loop
/// for the wooden slice.
pub fn object_tiles_at(world: &World, x: i32, y: i32) -> Option<ObjectTiles> {
    let t = world.tile(x, y)?;
    if !t.active() {
        return None;
    }
    let def = def_for_tile(t.kind)?;
    // The C# per-type loops read `frameX / 18` and `frameY / 18` with plain
    // truncating integer division (positive frames in practice); a corrupt
    // frame degrades exactly like the C# arithmetic does. Each def satisfies
    // `full_width/18 % width == 0` and `full_height/18 % height == 0`
    // (18/18=1, 20/18=1, 36/18=2, 38/18=2, 40/18=2, 54/18=3), so one formula
    // reproduces all six C# loops for the wooden slice.
    let dx = (t.frame_x as i32 / 18) % def.width;
    let dy = (t.frame_y as i32 / 18) % def.height;
    Some(ObjectTiles {
        def,
        origin: Point16::new(x - dx, y - dy),
    })
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)] // DW-IDs are part of the test names (traceability)
    use super::*;
    use crate::tile::FLAG_ACTIVE;
    use crate::world::World;

    /// 40x40 world, stone floor (tile 1) at y = 20 — deep enough inside the world for
    /// `CanPlace`'s 5-tile edge margin (`TileObject.cs:215`).
    fn world() -> World {
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

    fn set_active(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn set_liquid(w: &mut World, x: i32, y: i32, amount: u8, lava: bool) {
        let ti = w.index(x, y);
        let t = &mut w.tiles[ti];
        t.liquid = amount;
        if lava {
            t.flags |= crate::tile::FLAG_LIQUID_LAVA;
        }
    }

    /// Runs `can_place` and asserts a `Some` result, returning the instance.
    fn must_place(
        w: &World,
        x: i32,
        y: i32,
        def: &'static TileObjectDef,
        dir: i32,
    ) -> TileObjectInstance {
        can_place(w, x, y, def, 0, dir, None)
            .unwrap_or_else(|| panic!("expected placement at ({x}, {y})"))
    }

    // ---- DW-1.1: data model + wooden set ----------------------------------

    #[test]
    fn test_DW_1_1_wooden_set_geometry_matches_csharp() {
        // Numbers straight from the builder chains cited on each entry.
        let bench = def_for_tile(18).unwrap();
        assert_eq!((bench.width, bench.height), (2, 1)); // :2818-2819
        assert_eq!(bench.origin, Point16::new(0, 0)); // Style2x1 :2800
        assert_eq!(
            bench.anchor_bottom,
            AnchorData::new(
                anchor_type::SOLID_TILE | anchor_type::SOLID_WITH_TOP | anchor_type::SOLID_SIDE,
                2,
                0
            )
        ); // Style2x1 :2801
        assert_eq!(bench.coordinate_heights, &[18]); // :2820
        assert!(bench.style_horizontal); // :2806
        assert_eq!(bench.lava_placement, LiquidPlacement::NotAllowed); // :2807 + Calculate :1821-1831
        assert!(bench.frame_important); // Main.cs:7653

        let table = def_for_tile(14).unwrap();
        assert_eq!((table.width, table.height), (3, 2)); // :3169-3170
        assert_eq!(table.origin, Point16::new(1, 1)); // :3171
        assert_eq!(table.anchor_bottom.tile_count, 3); // Style3x2 :3172
        assert_eq!(table.coordinate_heights, &[16, 18]); // :3181
        assert!(table.style_horizontal); // :3177
        assert!(table.subtiles.is_empty(), "subtile data is a follow-up");

        let door = def_for_tile(10).unwrap();
        assert_eq!((door.width, door.height), (1, 3)); // :2223-2224
        assert_eq!(
            door.anchor_top,
            AnchorData::new(anchor_type::SOLID_TILE, 1, 0)
        ); // :2226
        assert_eq!(
            door.anchor_bottom,
            AnchorData::new(anchor_type::SOLID_TILE, 1, 0)
        ); // :2227
        assert_eq!(door.style_wrap_limit, 36); // :2234
        assert_eq!(door.style_line_skip, 3); // :2235
        assert!(!door.style_horizontal); // :2233
        assert_eq!(door.alternates.len(), 2); // :2236-2241
        assert_eq!(door.alternates[0].origin, Point16::new(0, 1));
        assert_eq!(door.alternates[1].origin, Point16::new(0, 2));
        assert_eq!(door.interaction, Interaction::Door);

        let open = def_for_tile(11).unwrap();
        assert_eq!((open.width, open.height), (2, 3)); // :2248-2249
        assert_eq!(open.direction, TileObjectDirection::PlaceRight); // :2261
        assert_eq!(open.style_line_skip, 2); // :2260
        assert_eq!(open.alternates.len(), 5); // :2262-2285
        // Right-open click points (`addAlternate(0)`), then left-open
        // (`addAlternate(1)` -> Style 1, PlaceLeft, hinge-column anchors).
        assert_eq!(open.alternates[0].origin, Point16::new(0, 1));
        assert_eq!(open.alternates[1].origin, Point16::new(0, 2));
        for (i, alt) in open.alternates.iter().enumerate() {
            let expected_dir = if i < 2 {
                TileObjectDirection::PlaceRight
            } else {
                TileObjectDirection::PlaceLeft
            };
            assert_eq!(alt.direction, expected_dir, "alternate {i}");
            assert_eq!(alt.style, if i < 2 { 0 } else { 1 }, "alternate {i}");
        }
        assert_eq!(open.alternates[2].origin, Point16::new(1, 0)); // :2269
        assert_eq!(
            open.alternates[2].anchor_top,
            AnchorData::new(anchor_type::SOLID_TILE, 1, 1)
        ); // :2270
        assert_eq!(open.interaction, Interaction::Door);

        let chest = def_for_tile(21).unwrap();
        assert_eq!((chest.width, chest.height), (2, 2)); // :3451-3452
        assert_eq!(chest.origin, Point16::new(0, 1)); // :3462
        // :3469 — bottom anchors WITHOUT AnchorType.Table.
        assert_eq!(
            chest.anchor_bottom,
            AnchorData::new(
                anchor_type::SOLID_TILE | anchor_type::SOLID_WITH_TOP | anchor_type::SOLID_SIDE,
                2,
                0
            )
        );
        assert_eq!(chest.anchor_invalid_tiles.len(), 11); // :3466, list :2112-2116
        // :3468 sets only `LavaDeath = false`; the base default is
        // `LavaPlacement = NotAllowed` (SetupBaseObject :1768) and `Calculate`
        // (:1821-1831) folds death -> placement only when the flag is true.
        assert_eq!(chest.lava_placement, LiquidPlacement::NotAllowed);
        assert!(
            chest.hook_check_if_can_place.hook.is_none(),
            "M3-CHEST-UI seam"
        );
        assert_eq!(chest.interaction, Interaction::None);

        let chair = def_for_tile(15).unwrap();
        assert_eq!((chair.width, chair.height), (1, 2)); // Style1x2 :4131-4132
        assert_eq!(chair.origin, Point16::new(0, 1)); // :4133
        assert_eq!(chair.coordinate_padding_fix, Point16::new(0, 2)); // :4146
        assert_eq!(chair.style_wrap_limit, 2); // :4144
        assert_eq!(chair.style_multiplier, 2); // :4145
        assert_eq!(chair.direction, TileObjectDirection::PlaceLeft); // :4143
        assert_eq!(chair.alternates.len(), 1); // :4148-4150
        assert_eq!(
            chair.alternates[0].direction,
            TileObjectDirection::PlaceRight
        );
        assert_eq!(chair.alternates[0].style, 1);
    }

    #[test]
    fn test_DW_1_1_def_for_tile_lookup() {
        for kind in [10u16, 11, 14, 15, 18, 21] {
            let def = def_for_tile(kind).unwrap();
            assert_eq!(def.kind, kind);
        }
        assert!(def_for_tile(1).is_none(), "dirt is not a tile object");
        assert!(def_for_tile(0).is_none());
    }

    #[test]
    fn test_DW_1_1_coordinate_full_extents() {
        // `Calculate` (`TileObjectData.cs:1812-1820`): CFW = (cw+pad)*W + fix.x,
        // CFH = sum(heights[i]+pad) + fix.y.
        let cases = [
            (18u16, 36, 20), // bench: (16+2)*2, 18+2
            (14, 54, 38),    // table: (16+2)*3, (16+2)+(18+2)
            (10, 18, 54),    // door: (16+2)*1, (16+2)*3
            (11, 36, 54),    // open door: (16+2)*2, (16+2)*3
            (21, 36, 38),    // chest: (16+2)*2, (16+2)+(18+2)
            (15, 18, 40),    // chair: (16+2)*1, (16+2)+(18+2)+fix 2
        ];
        for (kind, cfw, cfh) in cases {
            let def = def_for_tile(kind).unwrap();
            assert_eq!(def.coordinate_full_width(), cfw, "kind {kind}");
            assert_eq!(def.coordinate_full_height(), cfh, "kind {kind}");
        }
    }

    // ---- DW-1.2: CanPlace --------------------------------------------------

    #[test]
    fn test_DW_1_2_can_place_free_and_occupied() {
        let w = world();
        let bench = def_for_tile(18).unwrap();
        must_place(&w, 10, 19, bench, 1);
        // Occupied cells (any of the two) reject.
        let mut w2 = world();
        set_active(&mut w2, 11, 19, 1);
        assert!(can_place(&w2, 10, 19, bench, 0, 1, None).is_none());
        assert!(can_place(&w2, 11, 19, bench, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_can_place_world_bounds_rules() {
        let w = world();
        let bench = def_for_tile(18).unwrap();
        // :182-187 — the object must fit inside the world.
        assert!(can_place(&w, 39, 19, bench, 0, 1, None).is_none());
        // :213-218 — every alternate must clear the 5-tile border
        // (`return false`): x = 34 fails (34 + 2 > 35), x = 33 passes.
        assert!(can_place(&w, 34, 19, bench, 0, 1, None).is_none());
        assert!(can_place(&w, 4, 19, bench, 0, 1, None).is_none());
        assert!(can_place(&w, 33, 19, bench, 0, 1, None).is_some());
        assert!(can_place(&w, 5, 19, bench, 0, 1, None).is_some());
    }

    #[test]
    fn test_DW_1_2_can_place_requires_anchor() {
        let w = world();
        let bench = def_for_tile(18).unwrap();
        // Floating: no AnchorBottom support.
        assert!(can_place(&w, 10, 10, bench, 0, 1, None).is_none());
        // Partial support: the 2-wide bottom anchor needs both tiles solid.
        let mut w2 = world();
        let idx_11_20 = w2.index(11, 20);
        w2.tiles[idx_11_20] = Tile::default(); // hole under the right half
        assert!(can_place(&w2, 10, 19, bench, 0, 1, None).is_none());
        // A table (3 wide) over a 2-wide ledge fails.
        let table = def_for_tile(14).unwrap();
        let mut w3 = world();
        let idx_12_20 = w3.index(12, 20);
        w3.tiles[idx_12_20] = Tile::default();
        assert!(can_place(&w3, 11, 19, table, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_can_place_direction_filter() {
        let w = world();
        let chair = def_for_tile(15).unwrap();
        // Facing right -> the PlaceRight alternate wins (num7 = 1); facing left
        // -> the base def (num7 = 0) (`TileObject.cs:209-212`).
        let right = must_place(&w, 10, 19, chair, 1);
        assert_eq!(right.alternate, 1);
        let left = must_place(&w, 10, 19, chair, -1);
        assert_eq!(left.alternate, 0);
        // Open door: dir = 1 keeps the PlaceRight base (alternate 0); dir = -1
        // keeps the first PlaceLeft alternate (num7 = 3).
        let open = def_for_tile(11).unwrap();
        let mut w2 = world();
        // Hinge column top anchor; the bottom anchor is the floor row itself.
        set_active(&mut w2, 20, 16, 1);
        let r = must_place(&w2, 20, 17, open, 1);
        assert_eq!(r.alternate, 0);
        assert_eq!((r.x_coord, r.y_coord), (20, 17));
        let l = must_place(&w2, 20, 17, open, -1);
        assert_eq!(l.alternate, 3);
        assert_eq!((l.x_coord, l.y_coord), (19, 17));
    }

    #[test]
    fn test_DW_1_2_can_place_liquid_rules() {
        let bench = def_for_tile(18).unwrap();
        // Water is allowed for the wooden set (`WaterPlacement.Allowed`).
        let mut w = world();
        set_liquid(&mut w, 10, 19, 200, false);
        set_liquid(&mut w, 11, 19, 200, false);
        assert!(can_place(&w, 10, 19, bench, 0, 1, None).is_some());
        // Lava is not (`LavaPlacement.NotAllowed` — `LiquidPlace` :1862-1871).
        let mut w2 = world();
        set_liquid(&mut w2, 10, 19, 200, true);
        set_liquid(&mut w2, 11, 19, 200, true);
        assert!(can_place(&w2, 10, 19, bench, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_1_chest_cannot_be_placed_in_lava() {
        // The chest chain (:3461-3470) sets only `LavaDeath = false` (:3468);
        // `SetupBaseObject` defaults `LavaPlacement = NotAllowed` (:1768) and
        // `Calculate` (:1821-1831) folds death -> placement only when the death
        // flag is true — so the chest resolves to `NotAllowed` and
        // `TileObject.CanPlace` (TileObject.cs:342 → `LiquidPlace`) refuses
        // every lava-covered cell.
        let chest = def_for_tile(21).unwrap();
        // Water stays allowed (base `WaterPlacement.Allowed`, :1767).
        let mut w = world();
        for (x, y) in [(10, 18), (11, 18), (10, 19), (11, 19)] {
            set_liquid(&mut w, x, y, 200, false);
        }
        assert!(can_place(&w, 10, 19, chest, 0, 1, None).is_some());
        // Lava in any footprint cell is refused (2x2, origin (0,1)).
        let mut w2 = world();
        for (x, y) in [(10, 18), (11, 18), (10, 19), (11, 19)] {
            set_liquid(&mut w2, x, y, 200, true);
        }
        assert!(can_place(&w2, 10, 19, chest, 0, 1, None).is_none());
        // A single lava cell in the footprint blocks the whole object —
        // `flag5 | flag3 | flag4` marks that cell occupied (TileObject.cs:360).
        let mut w3 = world();
        set_liquid(&mut w3, 11, 19, 200, true);
        assert!(can_place(&w3, 10, 19, chest, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_can_place_cut_tiles_are_free() {
        let bench = def_for_tile(18).unwrap();
        // Cut tiles (grass plants, tile 82) do not block placement — `Place`
        // kills them (`TileObject.cs:84-93`, `:357`).
        let mut w = world();
        set_active(&mut w, 10, 19, 82);
        set_active(&mut w, 11, 19, 82);
        assert!(can_place(&w, 10, 19, bench, 0, 1, None).is_some());
        // Plain blocks still block.
        let mut w2 = world();
        set_active(&mut w2, 10, 19, 1);
        assert!(can_place(&w2, 10, 19, bench, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_can_place_liquid_only_in_full() {
        // `LiquidPlace` `OnlyInFullLiquid` (`TileObjectData.cs:1866-1884`).
        static TEST_FULL_LIQUID: TileObjectDef = TileObjectDef {
            kind: 250,
            water_placement: LiquidPlacement::OnlyInFullLiquid,
            ..base_def()
        };
        let mut w = world();
        set_liquid(&mut w, 10, 19, 100, false);
        assert!(can_place(&w, 10, 19, &TEST_FULL_LIQUID, 0, 1, None).is_none());
        set_liquid(&mut w, 10, 19, 255, false);
        assert!(can_place(&w, 10, 19, &TEST_FULL_LIQUID, 0, 1, None).is_some());
        // An empty cell fails OnlyInLiquid/OnlyInFullLiquid (:1889-1905).
        assert!(can_place(&w, 11, 19, &TEST_FULL_LIQUID, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_anchor_type_branches() {
        // Test-local defs exercise the anchor branches the wooden set does not
        // use; every def keeps `kind` 0.
        static EMPTY_TOP: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_top: AnchorData::new(anchor_type::EMPTY_TILE, 1, 0),
            ..base_def()
        };
        static SOLID_SIDE: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_left: AnchorData::new(anchor_type::SOLID_SIDE, 1, 0),
            ..base_def()
        };
        static ON_TABLE: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::TABLE, 1, 0),
            ..base_def()
        };
        static ON_PLATFORM: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(
                anchor_type::SOLID_TILE | anchor_type::SOLID_WITH_TOP,
                1,
                0,
            ),
            ..base_def()
        };
        static ON_TREE: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_left: AnchorData::new(anchor_type::TREE, 1, 0),
            ..base_def()
        };
        static INVALID_LIST: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
            anchor_invalid_tiles: &[1],
            ..base_def()
        };
        let mut w = world();
        // EmptyTile: free above -> the else-if branch accepts (:513-517).
        assert!(can_place(&w, 10, 19, &EMPTY_TOP, 0, 1, None).is_some());
        // A plain solid tile above satisfies ANY top-anchor type: the first
        // top branch (:453-460) has no `anchor.ty` gate — vanilla behavior.
        set_active(&mut w, 10, 18, 1);
        assert!(can_place(&w, 10, 19, &EMPTY_TOP, 0, 1, None).is_some());
        // A solid-top tile (platform, kind 19) is not "solid", so the first
        // branch skips and the EmptyTile type rejects the active cell.
        set_active(&mut w, 10, 18, 19);
        assert!(can_place(&w, 10, 19, &EMPTY_TOP, 0, 1, None).is_none());
        let idx_10_18 = w.index(10, 18);
        w.tiles[idx_10_18] = Tile::default();
        // SolidSide: anchor against a solid tile to the left (:646-653).
        set_active(&mut w, 9, 19, 1);
        assert!(can_place(&w, 10, 19, &SOLID_SIDE, 0, 1, None).is_some());
        let idx_9_19 = w.index(9, 19);
        w.tiles[idx_9_19] = Tile::default();
        // Table anchor over a table tile (:407-410).
        set_active(&mut w, 10, 19, 14);
        assert!(can_place(&w, 10, 18, &ON_TABLE, 0, 1, None).is_some());
        let idx_10_19 = w.index(10, 19);
        w.tiles[idx_10_19] = Tile::default();
        // Platform top frame (:394-401): tile 19 with a proper top frame.
        set_active(&mut w, 10, 19, 19);
        let idx_10_19 = w.index(10, 19);
        w.tiles[idx_10_19].frame_x = 0;
        assert!(can_place(&w, 10, 18, &ON_PLATFORM, 0, 1, None).is_some());
        let idx_10_19 = w.index(10, 19);
        w.tiles[idx_10_19] = Tile::default();
        // AnchorInvalidTiles (`isValidTileAnchor` :1924-1933).
        set_active(&mut w, 10, 19, 1);
        assert!(can_place(&w, 10, 18, &INVALID_LIST, 0, 1, None).is_none());
        // ...and an unlisted tile passes.
        set_active(&mut w, 10, 19, 2);
        assert!(can_place(&w, 10, 18, &INVALID_LIST, 0, 1, None).is_some());
        let idx_10_19b = w.index(10, 19);
        w.tiles[idx_10_19b] = Tile::default();
        // Tree trunk anchor (:654-691): the trunk must continue above and
        // below the anchor run, or the extension checks miss and the anchor
        // ratio drops below 1.
        set_active(&mut w, 9, 18, 5);
        set_active(&mut w, 9, 19, 5);
        set_active(&mut w, 9, 20, 5);
        assert!(can_place(&w, 10, 19, &ON_TREE, 0, 1, None).is_some());
        // A lone trunk (no continuation below) fails the ratio.
        let idx_9_20 = w.index(9, 20);
        w.tiles[idx_9_20] = Tile::default();
        assert!(can_place(&w, 10, 19, &ON_TREE, 0, 1, None).is_none());
    }

    // ---- DW-1.3: Place -----------------------------------------------------

    #[test]
    fn test_DW_1_3_place_writes_tiles_frames_flag() {
        let mut w = world();
        let bench = def_for_tile(18).unwrap();
        let obj = must_place(&w, 10, 19, bench, 1);
        assert!(place(&mut w, &obj));
        for k in 0..2 {
            let t = *w.tile(10 + k, 19).unwrap();
            assert!(t.active() && t.kind == 18, "cell {k}");
            assert_eq!((t.frame_x, t.frame_y), ((18 * k) as i16, 0), "cell {k}");
        }
        assert!(w.frame_important[18], "frame_important set for the kind");
        // The liquid of the cell survives (:101-107 writes only
        // active/frameX/frameY/type).
        let mut w2 = world();
        set_liquid(&mut w2, 10, 19, 200, false);
        set_liquid(&mut w2, 11, 19, 200, false);
        let obj2 = must_place(&w2, 10, 19, bench, 1);
        assert!(place(&mut w2, &obj2));
        assert_eq!(w2.tile(10, 19).unwrap().liquid, 200);
    }

    #[test]
    fn test_DW_1_3_place_style_frames() {
        // placeStyle -> frame coordinates, per object (U4): the math of
        // `Place` :56-76 applied to each def's fields.
        let mut w = world();

        // Work bench, style 3: horizontal, wrap 0 -> frameX = CFW*3 = 108.
        let bench = def_for_tile(18).unwrap();
        let obj = can_place(&w, 8, 19, bench, 3, 1, None).unwrap();
        assert!(place(&mut w, &obj));
        assert_eq!(
            (
                w.tile(8, 19).unwrap().frame_x,
                w.tile(8, 19).unwrap().frame_y
            ),
            (108, 0)
        );
        assert_eq!(
            (
                w.tile(9, 19).unwrap().frame_x,
                w.tile(9, 19).unwrap().frame_y
            ),
            (126, 0)
        );

        // Table, style 2: frameX = 54*2 = 108, second row frameY = 18.
        let table = def_for_tile(14).unwrap();
        let obj = can_place(&w, 12, 19, table, 2, 1, None).unwrap();
        assert!(place(&mut w, &obj));
        assert_eq!(w.tile(11, 18).unwrap().frame_x, 108);
        assert_eq!(w.tile(11, 18).unwrap().frame_y, 0);
        assert_eq!(w.tile(12, 19).unwrap().frame_x, 126);
        assert_eq!(w.tile(12, 19).unwrap().frame_y, 18);

        // Closed door, style 1: not horizontal -> frameX = 18*0, frameY = 54*1.
        let door = def_for_tile(10).unwrap();
        set_active(&mut w, 20, 16, 1); // door top anchor
        let obj = can_place(&w, 20, 17, door, 1, 1, None).unwrap();
        assert!(place(&mut w, &obj));
        assert_eq!(
            (
                w.tile(20, 17).unwrap().frame_x,
                w.tile(20, 17).unwrap().frame_y
            ),
            (0, 54)
        );
        assert_eq!(
            (
                w.tile(20, 19).unwrap().frame_x,
                w.tile(20, 19).unwrap().frame_y
            ),
            (0, 90)
        );

        // Closed door, style 36: wrap 36 / skip 3 -> frameX = 18*3 = 54,
        // frameY = 0 (the layout `OpenDoor` decodes at WorldGen.cs:37671).
        let mut w3 = world();
        set_active(&mut w3, 20, 16, 1);
        let obj = can_place(&w3, 20, 17, door, 36, 1, None).unwrap();
        assert!(place(&mut w3, &obj));
        assert_eq!(
            (
                w3.tile(20, 17).unwrap().frame_x,
                w3.tile(20, 17).unwrap().frame_y
            ),
            (54, 0)
        );

        // Chest, style 1: frameX = 36*1, rows 0/18.
        let chest = def_for_tile(21).unwrap();
        let obj = can_place(&w, 24, 19, chest, 1, 1, None).unwrap();
        assert!(place(&mut w, &obj));
        assert_eq!(
            (
                w.tile(24, 18).unwrap().frame_x,
                w.tile(24, 18).unwrap().frame_y
            ),
            (36, 0)
        );
        assert_eq!(
            (
                w.tile(25, 19).unwrap().frame_x,
                w.tile(25, 19).unwrap().frame_y
            ),
            (54, 18)
        );

        // Chair, style 2 right-facing: multiplier 2 -> placement style 5;
        // wrap 2 -> column 1, row 2 -> frameX = 18, frameY = 40*2 = 80.
        let chair = def_for_tile(15).unwrap();
        let obj = can_place(&w, 30, 19, chair, 2, 1, None).unwrap();
        assert!(place(&mut w, &obj));
        assert_eq!(
            (
                w.tile(30, 18).unwrap().frame_x,
                w.tile(30, 18).unwrap().frame_y
            ),
            (18, 80)
        );
        assert_eq!(
            (
                w.tile(30, 19).unwrap().frame_x,
                w.tile(30, 19).unwrap().frame_y
            ),
            (18, 98)
        );

        // Open door, right-facing style 0 (`OpenDoor`-compatible layout:
        // frameX 0/18, frameY rows 0/18/36 — WorldGen.cs:37708+).
        let open = def_for_tile(11).unwrap();
        let mut w2 = world();
        set_active(&mut w2, 20, 16, 1);
        let obj = can_place(&w2, 20, 17, open, 0, 1, None).unwrap();
        assert!(place(&mut w2, &obj));
        assert_eq!(
            (
                w2.tile(20, 17).unwrap().frame_x,
                w2.tile(20, 17).unwrap().frame_y
            ),
            (0, 0)
        );
        assert_eq!(
            (
                w2.tile(21, 18).unwrap().frame_x,
                w2.tile(21, 18).unwrap().frame_y
            ),
            (18, 18)
        );

        // Open door, left-facing (alternate 3, Style=1): the generic
        // `TileObject.Place` math (`TileObject.cs:56-76`) shifts the style ROW
        // (`num4 = 0*1 + 1` -> frameY = 54) — vanilla's hand-written `OpenDoor`
        // uses a different hardcoded layout for the left leaf
        // (`WorldGen.cs:37695-37698`, frameX +36). Both are ported as-is; the
        // discrepancy is a vanilla quirk, recorded for `docs/ANALYSIS.md`.
        let mut w4 = world();
        set_active(&mut w4, 20, 16, 1);
        let obj = can_place(&w4, 20, 17, open, 0, -1, None).unwrap();
        assert!(place(&mut w4, &obj));
        assert_eq!(
            (
                w4.tile(19, 17).unwrap().frame_x,
                w4.tile(19, 17).unwrap().frame_y
            ),
            (0, 54)
        );
        assert_eq!(
            (
                w4.tile(20, 19).unwrap().frame_x,
                w4.tile(20, 19).unwrap().frame_y
            ),
            (18, 90)
        );
    }

    #[test]
    fn test_DW_1_3_failed_place_leaves_world_unchanged() {
        let mut w = world();
        set_active(&mut w, 10, 19, 1);
        set_active(&mut w, 11, 19, 1); // both bench cells occupied
        let before: Vec<Tile> = w.tiles.clone();
        // Unknown kind -> the def does not resolve -> false, no change.
        assert!(!place(
            &mut w,
            &TileObjectInstance {
                x_coord: 10,
                y_coord: 19,
                kind: 999,
                style: 0,
                alternate: 0,
                random: -1
            }
        ));
        // Overwriting an active cell is skipped by the write loop (:101).
        assert!(place(
            &mut w,
            &TileObjectInstance {
                x_coord: 10,
                y_coord: 19,
                kind: 18,
                style: 0,
                alternate: 0,
                random: -1
            }
        ));
        assert_eq!(before, w.tiles, "world unchanged");
    }

    #[test]
    fn test_DW_1_3_place_kills_cut_tiles() {
        let mut w = world();
        let bench = def_for_tile(18).unwrap();
        set_active(&mut w, 10, 19, 82);
        set_active(&mut w, 11, 19, 82);
        let obj = must_place(&w, 10, 19, bench, 1);
        assert!(place(&mut w, &obj));
        for k in 0..2 {
            let t = *w.tile(10 + k, 19).unwrap();
            assert!(t.active() && t.kind == 18, "cut tile replaced by cell {k}");
        }
    }

    // ---- DW-1.4: doors, overlap, geometry ----------------------------------

    #[test]
    fn test_DW_1_4_door_closed_three_click_points() {
        let mut w = world();
        let door = def_for_tile(10).unwrap();
        set_active(&mut w, 20, 16, 1); // door top anchor
        // Gap at x = 20, y = 17..19: top anchor (20,16) = dirt, bottom (20,20) =
        // floor. Clicking any of the three tiles resolves to the same origin.
        let top = must_place(&w, 20, 17, door, 1);
        let mid = must_place(&w, 20, 18, door, 1);
        let bot = must_place(&w, 20, 19, door, 1);
        assert_eq!((top.x_coord, top.y_coord), (20, 17));
        assert_eq!((mid.x_coord, mid.y_coord), (20, 17));
        assert_eq!((bot.x_coord, bot.y_coord), (20, 17));
        assert_eq!((top.alternate, mid.alternate, bot.alternate), (0, 1, 2));
        // A cell next to the gap is not a click point for this door.
        assert!(can_place(&w, 21, 17, door, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_4_door_open_both_sides() {
        let open = def_for_tile(11).unwrap();
        // Hinge column x = 20 with anchors; the leaf needs one free column.
        let mut w = world();
        set_active(&mut w, 20, 16, 1);
        set_active(&mut w, 20, 23, 1);
        // Facing right: leaf at 21 -> blocked by a solid tile there.
        set_active(&mut w, 21, 17, 1);
        set_active(&mut w, 21, 18, 1);
        set_active(&mut w, 21, 19, 1);
        assert!(
            can_place(&w, 20, 17, open, 0, 1, None).is_none(),
            "right leaf blocked"
        );
        assert!(
            can_place(&w, 20, 17, open, 0, -1, None).is_some(),
            "left leaf free"
        );
        // Facing left with the left column blocked instead.
        let mut w2 = world();
        set_active(&mut w2, 20, 16, 1);
        set_active(&mut w2, 20, 23, 1);
        set_active(&mut w2, 19, 17, 1);
        set_active(&mut w2, 19, 18, 1);
        set_active(&mut w2, 19, 19, 1);
        assert!(
            can_place(&w2, 20, 17, open, 0, 1, None).is_some(),
            "right leaf free"
        );
        assert!(
            can_place(&w2, 20, 17, open, 0, -1, None).is_none(),
            "left leaf blocked"
        );
        // Open-door click points: any of the three rows of the hinge column.
        let mut w3 = world();
        set_active(&mut w3, 20, 16, 1);
        set_active(&mut w3, 20, 23, 1);
        let a = must_place(&w3, 20, 17, open, 1);
        let b = must_place(&w3, 20, 18, open, 1);
        let c = must_place(&w3, 20, 19, open, 1);
        assert_eq!((a.x_coord, a.y_coord), (20, 17));
        assert_eq!((b.x_coord, b.y_coord), (20, 17));
        assert_eq!((c.x_coord, c.y_coord), (20, 17));
        assert_eq!((a.alternate, b.alternate, c.alternate), (0, 1, 2));
    }

    #[test]
    fn test_DW_1_4_overlap_partial_rejection() {
        let table = def_for_tile(14).unwrap();
        // One occupied cell of a 3x2 object rejects the whole object — and
        // `can_place` never writes (the world is unchanged).
        let mut w2 = world();
        set_active(&mut w2, 13, 18, 1);
        assert!(can_place(&w2, 12, 19, table, 0, 1, None).is_none());
        assert!(
            !w2.tile(11, 18).unwrap().active(),
            "no partial writes from can_place"
        );
        // Chest over a table edge: the chest's bottom anchor drops
        // `AnchorType.Table` (:3469) — sitting on two tables is refused while
        // full solid support works.
        let chest = def_for_tile(21).unwrap();
        let mut w3 = world();
        set_active(&mut w3, 10, 20, 14); // table
        set_active(&mut w3, 11, 20, 14); // table
        assert!(
            can_place(&w3, 10, 19, chest, 0, 1, None).is_none(),
            "no Table anchor"
        );
        set_active(&mut w3, 10, 20, 1);
        set_active(&mut w3, 11, 20, 1);
        assert!(can_place(&w3, 10, 19, chest, 0, 1, None).is_some());
    }

    // ---- DW-1.5: module doc -------------------------------------------------

    #[test]
    fn test_DW_1_5_module_doc_maps_csharp_fields() {
        const DOC: &str = include_str!("tileobject.rs");
        assert!(DOC.contains("# C#->Rust map (DW-1.5)") || DOC.contains("C#→Rust map (DW-1.5)"));
        // Ported fields are listed with their C# anchors...
        for marker in [
            "SetupBaseObject",
            "CalculatePlacementStyle",
            "GetTileData",
            "hook_check_if_can_place",
            "coordinate_padding_fix",
            "style_line_skip",
        ] {
            assert!(DOC.contains(marker), "doc mentions {marker}");
        }
        // ...and the follow-up boundaries are written down.
        for marker in [
            "Not ported",
            "M3-CHEST-UI",
            "SubTiles",
            "Main.rand",
            "onlyCheck",
        ] {
            assert!(DOC.contains(marker), "doc records {marker}");
        }
    }

    // ---- DW-1.6: reframe_area keeps frame_important tiles ------------------

    #[test]
    fn test_DW_1_6_reframe_area_keeps_object_frames() {
        let mut w = world();
        let bench = def_for_tile(18).unwrap();
        let door = def_for_tile(10).unwrap();
        set_active(&mut w, 20, 16, 1); // door top anchor
        let bench_obj = must_place(&w, 8, 19, bench, 1);
        assert!(place(&mut w, &bench_obj));
        let door_obj = must_place(&w, 20, 17, door, 1);
        assert!(place(&mut w, &door_obj));
        // Snapshot every object cell's frame.
        let mut object_frames = Vec::new();
        for (x, y) in [(8, 19), (9, 19), (20, 17), (20, 18), (20, 19)] {
            let t = *w.tile(x, y).unwrap();
            object_frames.push((x, y, t.kind, t.frame_x, t.frame_y));
        }
        // A non-important control tile with a "must be computed" frame.
        let i = w.index(14, 19);
        w.tiles[i] = Tile {
            kind: 1,
            frame_x: -1,
            frame_y: -1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        // Re-frame the whole area around every object cell and the control.
        for (x, y) in [(8, 19), (9, 19), (20, 17), (20, 18), (20, 19), (14, 19)] {
            crate::framing::reframe_area(&mut w, x, y, 2);
        }
        for (x, y, kind, fx, fy) in object_frames {
            let t = *w.tile(x, y).unwrap();
            assert_eq!(
                (t.kind, t.frame_x, t.frame_y),
                (kind, fx, fy),
                "reframe_area must not touch the frame_important object at ({x},{y})"
            );
        }
        // Control: the dirt tile did get a real frame.
        let t = *w.tile(14, 19).unwrap();
        assert!(
            t.frame_x >= 0 && t.frame_y >= 0,
            "control tile was re-framed"
        );
    }

    // ---- beyond the DW list -------------------------------------------------

    #[test]
    fn test_calculate_placement_style_matches_csharp() {
        let chair = def_for_tile(15).unwrap();
        // `style * StyleMultiplier + Style` (+ random when >= 0) (:1990-1998).
        assert_eq!(calculate_placement_style(chair, 3, 0, -1), 6);
        assert_eq!(calculate_placement_style(&chair.alternates[0], 3, 1, -1), 7);
        assert_eq!(calculate_placement_style(chair, 3, 0, 0), 6);
        assert_eq!(calculate_placement_style(chair, 0, 0, 2), 2);
    }

    #[test]
    fn test_get_tile_data_resolution() {
        static TEST_SUB: TileObjectDef = TileObjectDef {
            height: 7,
            ..base_def()
        };
        static TEST_ALT: TileObjectDef = TileObjectDef {
            height: 9,
            ..base_def()
        };
        static WITH_SUBS: TileObjectDef = TileObjectDef {
            alternates: &[TEST_ALT],
            subtiles: &[(5u16, &TEST_SUB)],
            ..base_def()
        };
        // alternate = 0 -> the def itself (base or sub-tile) (:5139-5157).
        assert_eq!(get_tile_data(&WITH_SUBS, 0, 0).unwrap().height, 1);
        // style selects a sub-tile.
        assert_eq!(get_tile_data(&WITH_SUBS, 5, 0).unwrap().height, 7);
        // alternate - 1 indexes the alternates of the resolved def.
        assert_eq!(get_tile_data(&WITH_SUBS, 0, 1).unwrap().height, 9);
        // An alternate index past the list -> None.
        assert!(get_tile_data(&WITH_SUBS, 0, 2).is_none());
    }

    #[test]
    fn test_is_valid_anchor_lists() {
        static DEF: TileObjectDef = TileObjectDef {
            anchor_valid_tiles: &[80],
            anchor_invalid_tiles: &[127],
            anchor_alternate_tiles: &[753],
            anchor_valid_walls: &[],
            ..base_def()
        };
        // `isValidTileAnchor` (:1910-1946): the invalid list wins first; a
        // null valid list (the port's empty slice) means "everything valid";
        // a non-null list is strict membership.
        assert!(!is_valid_tile_anchor(&DEF, 127));
        assert!(is_valid_tile_anchor(&DEF, 80), "listed valid tile");
        assert!(!is_valid_tile_anchor(&DEF, 5), "valid list is strict");
        static ANY: TileObjectDef = TileObjectDef {
            anchor_invalid_tiles: &[127],
            ..base_def()
        };
        assert!(is_valid_tile_anchor(&ANY, 5), "null valid list = anything");
        assert!(!is_valid_tile_anchor(&ANY, 127));
        // `isValidWallAnchor` (:1948-1967): empty list = "any non-zero wall".
        assert!(is_valid_wall_anchor(&DEF, 5));
        assert!(!is_valid_wall_anchor(&DEF, 0));
        // `isValidAlternateAnchor` (:1969-1988).
        assert!(is_valid_alternate_anchor(&DEF, 753));
        assert!(!is_valid_alternate_anchor(&DEF, 5));
    }

    #[test]
    fn test_block_type_and_nactive() {
        let mut t = Tile::default();
        assert_eq!(block_type(&t), 0);
        t.set_slope(2);
        assert_eq!(block_type(&t), 3); // slope + 1
        t.set_slope(0);
        t.flags |= crate::tile::FLAG_HALF_BRICK;
        assert_eq!(block_type(&t), 1); // half brick
        t.flags |= FLAG_ACTIVE;
        assert!(nactive(&t));
        t.flags |= crate::tile::FLAG_INACTIVE;
        assert!(!nactive(&t), "inActive tiles fail nactive (Tile.cs:240)");
    }

    #[test]
    fn test_can_place_hook_check_branch() {
        static HOOKED: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            hook_check_if_can_place: PlacementHook {
                hook: Some(hook_reject),
                bad_return: -1,
                bad_response: 0,
                processed_coordinates: false,
            },
            ..base_def()
        };
        // Everything free, but the hook returns badReturn with badResponse 0 ->
        // the scores zero out and the def is refused (:716-729).
        let w = world();
        assert!(can_place(&w, 10, 19, &HOOKED, 0, 1, None).is_none());
    }

    #[test]
    fn test_place_hook_override_branch() {
        static OVERRIDDEN: TileObjectDef = TileObjectDef {
            width: 2,
            height: 1,
            hook_place_override: PlacementHook {
                hook: Some(hook_accept),
                bad_return: -1,
                bad_response: 0,
                processed_coordinates: false,
            },
            ..base_def()
        };
        static REJECTING: TileObjectDef = TileObjectDef {
            width: 2,
            height: 1,
            hook_place_override: PlacementHook {
                hook: Some(hook_reject),
                bad_return: -1,
                bad_response: 0,
                processed_coordinates: false,
            },
            ..base_def()
        };
        let mut w = world();
        // The override hook replaces the tile-writing branch entirely
        // (:36-54): accepted -> true, world untouched by the main branch.
        assert!(place_with_def(
            &mut w,
            &OVERRIDDEN,
            &TileObjectInstance {
                x_coord: 10,
                y_coord: 19,
                kind: OVERRIDDEN.kind,
                style: 0,
                alternate: 0,
                random: -1
            }
        ));
        assert!(
            !w.tile(10, 19).unwrap().active(),
            "no tile writes on the hook path"
        );
        // badReturn -> false.
        assert!(!place_with_def(
            &mut w,
            &REJECTING,
            &TileObjectInstance {
                x_coord: 10,
                y_coord: 19,
                kind: REJECTING.kind,
                style: 0,
                alternate: 0,
                random: -1
            }
        ));
    }

    fn hook_reject(
        _world: &World,
        _x: i32,
        _y: i32,
        _kind: u16,
        _style: i32,
        _dir: i32,
        _alt: i32,
    ) -> i32 {
        -1
    }

    fn hook_accept(
        _world: &World,
        _x: i32,
        _y: i32,
        _kind: u16,
        _style: i32,
        _dir: i32,
        _alt: i32,
    ) -> i32 {
        0
    }

    #[test]
    fn test_place_flatten_anchors_branch() {
        static FLATTENING: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
            flatten_anchors: true,
            ..base_def()
        };
        let mut w = world();
        // A sloped tile below the object gets levelled (`Place` :114-127,
        // `SlopeTile` WorldGen.cs:80768).
        let i = w.index(10, 20);
        w.tiles[i].set_slope(1);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        assert!(place_with_def(
            &mut w,
            &FLATTENING,
            &TileObjectInstance {
                x_coord: 10,
                y_coord: 19,
                kind: FLATTENING.kind,
                style: 0,
                alternate: 0,
                random: -1
            }
        ));
        let t = *w.tile(10, 20).unwrap();
        assert_eq!(t.slope(), 0);
        assert!(!t.half_brick());
    }

    #[test]
    fn test_place_out_of_bounds_is_neutral() {
        let mut w = world();
        let bench = def_for_tile(18).unwrap();
        // An instance aimed past the world edge writes nothing and does not
        // panic (plan constraint: neutral "empty tile", no panic) — C# writes
        // into `Framing.GetTileSafely` dummies, which the port models as
        // skipped cells.
        let before: Vec<Tile> = w.tiles.clone();
        assert!(place(
            &mut w,
            &TileObjectInstance {
                x_coord: 40,
                y_coord: 19,
                kind: bench.kind,
                style: 0,
                alternate: 0,
                random: -1
            }
        ));
        assert_eq!(before, w.tiles, "out-of-bounds writes are discarded");
    }

    #[test]
    fn test_can_place_forced_random() {
        // No wooden def uses random styles, so `random` resolves to -1
        // (`TileObject.cs:761-793`; the Main.rand path is a follow-up).
        let w = world();
        let bench = def_for_tile(18).unwrap();
        let obj = must_place(&w, 10, 19, bench, 1);
        assert_eq!(obj.random, -1);
        static RANDOM: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
            random_style_range: 4,
            ..base_def()
        };
        // forcedRandom folds into the range (:788-791).
        let obj = can_place(&w, 10, 19, &RANDOM, 0, 1, Some(7)).unwrap();
        assert_eq!(obj.random, 3);
        let obj = can_place(&w, 10, 19, &RANDOM, 0, 1, Some(-1)).unwrap();
        assert_eq!(obj.random, 3);
        static SPECIFIC: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
            specific_random_styles: &[2, 9],
            ..base_def()
        };
        // SpecificRandomStyles picks from the list and subtracts the style
        // (:818-824).
        let obj = can_place(&w, 10, 19, &SPECIFIC, 2, 1, Some(1)).unwrap();
        assert_eq!(obj.random, 7); // 9 - 2
    }

    // ---- M1.4-TO-4 phase 1 (DW-1.2): the herb/pot/pile registry slice ------

    /// The `world()` fixture floor repainted to `kind` — a support row the
    /// anchor checks judge.
    fn floor_of(w: &mut World, kind: u16) {
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i].kind = kind;
        }
    }

    #[test]
    fn test_DW_1_2_herb_pot_pile_geometry_matches_csharp() {
        // Numbers straight from the builder chains cited on each entry.
        let sunflower = def_for_tile(27).unwrap();
        assert_eq!((sunflower.width, sunflower.height), (2, 4)); // :4851-4852
        assert_eq!(sunflower.origin, Point16::new(0, 3)); // :4853
        assert_eq!(
            sunflower.anchor_bottom,
            AnchorData::new(anchor_type::SOLID_TILE, 2, 0)
        ); // :4854
        assert_eq!(sunflower.anchor_valid_tiles, &[2, 477, 109, 60, 492, 633]); // :4859
        assert_eq!(sunflower.coordinate_heights, &[16, 16, 16, 18]); // :4856
        assert!(sunflower.style_horizontal); // :4860
        assert_eq!(sunflower.random_style_range, 3); // :4861
        assert_eq!(sunflower.water_placement, LiquidPlacement::NotAllowed); // :4863
        assert_eq!(sunflower.lava_placement, LiquidPlacement::NotAllowed); // :4862 + Calculate
        assert_eq!(sunflower.coordinate_full_width(), 36);
        assert_eq!(sunflower.coordinate_full_height(), 74);

        let pots = def_for_tile(28).unwrap();
        assert_eq!((pots.width, pots.height), (2, 2)); // Style2x2 :3451-3452
        assert_eq!(pots.origin, Point16::new(0, 1)); // :3453
        // :3556 — the pot anchor drops the preset's `AnchorType.Table`.
        assert_eq!(
            pots.anchor_bottom,
            AnchorData::new(
                anchor_type::SOLID_TILE | anchor_type::SOLID_WITH_TOP | anchor_type::SOLID_SIDE,
                2,
                0
            )
        );
        assert_eq!(pots.coordinate_heights, &[16, 16]); // :3456
        assert!(pots.style_horizontal); // :3557
        assert_eq!(pots.style_wrap_limit, 3); // :3558

        let pot = def_for_tile(78).unwrap();
        assert_eq!((pot.width, pot.height), (1, 1)); // StyleOnTable1x1 :2324-2325
        assert_eq!(
            pot.anchor_bottom,
            AnchorData::new(
                anchor_type::SOLID_TILE
                    | anchor_type::SOLID_WITH_TOP
                    | anchor_type::TABLE
                    | anchor_type::SOLID_SIDE,
                1,
                0
            )
        ); // :2385
        assert_eq!(pot.coordinate_heights, &[16]); // :2332
        // :2387 sets only LavaDeath = false — the baked NotAllowed stands
        // (the chest precedent, :3468).
        assert_eq!(pot.lava_placement, LiquidPlacement::NotAllowed);

        // Herbs 82/83/84: `FullCopyFrom` chains (:4784-4788) — one shared
        // shape, stage = tile type.
        let immature = def_for_tile(82).unwrap();
        for herb in [immature, def_for_tile(83).unwrap(), def_for_tile(84).unwrap()] {
            assert_eq!((herb.width, herb.height), (1, 1)); // StyleAlch :4738-4739
            assert_eq!(herb.coordinate_heights, &[20]); // :4741
            assert_eq!(herb.coordinate_width, 16); // :4742
            assert_eq!(herb.coordinate_padding, 2); // :4743
            assert!(herb.style_horizontal); // :4745
            assert_eq!(herb.water_placement, LiquidPlacement::NotAllowed); // :4748
            assert_eq!(herb.lava_placement, LiquidPlacement::NotAllowed); // :4750
            assert_eq!(
                herb.anchor_bottom,
                AnchorData::new(
                    anchor_type::SOLID_TILE | anchor_type::ALTERNATE_TILE,
                    1,
                    0
                )
            ); // :4746-4747
            assert_eq!(herb.anchor_alternate_tiles, &[78]); // the style-0 chain :4756
        }
        assert_eq!(immature.subtiles.len(), 6, "sub-styles 1-6 (:4757-4783)");

        let piles = def_for_tile(185).unwrap();
        assert_eq!((piles.width, piles.height), (1, 1)); // Style1x1 :4238-4239
        assert_eq!(
            piles.anchor_bottom,
            AnchorData::new(ANCHOR_SWTSS, 1, 0) // Style1x1 :4240
        );
        assert_eq!(piles.anchor_invalid_tiles.len(), 11, "Style1x1's shared list (:4244)");
        assert!(piles.style_horizontal); // :4978
        assert_eq!(piles.subtiles.len(), 1, "SubTiles[0] = the 2x1 variant (:4981-4984)");

        for (kind, wrap) in [(186u16, 0), (187, 35)] {
            let def = def_for_tile(kind).unwrap();
            assert_eq!((def.width, def.height), (3, 2)); // Style3x2 :3170-3171
            assert_eq!(def.origin, Point16::new(1, 1)); // :3172
            assert_eq!(def.anchor_bottom.tile_count, 3); // :3173
            assert_eq!(def.coordinate_heights, &[16, 16]); // :3175
            assert!(def.style_horizontal); // :3177
            assert_eq!(def.style_wrap_limit, wrap, "kind {kind}"); // 187 only, :3302
            assert_eq!(def.lava_placement, LiquidPlacement::NotAllowed, "kind {kind} (LavaDeath=false keeps the baked NotAllowed)");
        }
        assert_eq!(def_for_tile(186).unwrap().subtiles.len(), 9, "26-31 snow + 32-34 ice (:3291-3294)");
        assert_eq!(def_for_tile(187).unwrap().subtiles.len(), 15, "0-5 + 6-8 + 29-34 (:3305-3311)");
    }

    #[test]
    fn test_DW_1_2_herb_subtile_styles_resolve_anchor_sets() {
        // `GetTileData(82, style)` (`:5136-5144`): styles 1-6 hit the
        // sub-tiles, style 0 stays the base — each with its C# soil set.
        let herb = def_for_tile(82).unwrap();
        let sets: [(i32, &[u16]); 7] = [
            (0, &[2, 477, 109, 492]),
            (1, &[60]),
            (2, &[0, 59]),
            (3, &[199, 203, 25, 23]),
            (4, &[53, 116]),
            (5, &[57, 633]),
            (6, &[147, 161, 163, 164, 200]),
        ];
        for (style, expected) in sets {
            let resolved = get_tile_data(herb, style, 0).unwrap();
            assert_eq!(resolved.anchor_valid_tiles, expected, "style {style}");
            // Every style keeps the clay-pot alternate anchor (:4756 etc.).
            assert_eq!(resolved.anchor_alternate_tiles, &[78], "style {style}");
        }
        // Style 4 allows water (:4757-4782's subtile chain), style 5 allows
        // lava and drops LavaDeath, style 6 allows water.
        assert_eq!(get_tile_data(herb, 4, 0).unwrap().water_placement, LiquidPlacement::Allowed);
        assert_eq!(get_tile_data(herb, 5, 0).unwrap().lava_placement, LiquidPlacement::Allowed);
        assert_eq!(get_tile_data(herb, 6, 0).unwrap().water_placement, LiquidPlacement::Allowed);
        // 83/84 are FullCopyFrom chains — the sub-tiles ride along.
        for kind in [83u16, 84] {
            let def = def_for_tile(kind).unwrap();
            assert_eq!(get_tile_data(def, 5, 0).unwrap().lava_placement, LiquidPlacement::Allowed);
        }
        // Small piles: style 0 IS the 2x1 sub-tile (SubTiles[0] non-null,
        // :4981-4984); any other style falls back to the 1x1 base.
        let piles = def_for_tile(185).unwrap();
        assert_eq!(get_tile_data(piles, 0, 0).unwrap().width, 2);
        assert_eq!(get_tile_data(piles, 0, 0).unwrap().style_wrap_limit, 53);
        assert_eq!(get_tile_data(piles, 1, 0).unwrap().width, 1);
        // 186's sparse sub-tiles: 26-31 snow, 32-34 ice, others the base.
        let large = def_for_tile(186).unwrap();
        assert_eq!(
            get_tile_data(large, 26, 0).unwrap().anchor_valid_tiles,
            &[147, 161, 163, 200, 164, 162, 224]
        );
        assert_eq!(get_tile_data(large, 32, 0).unwrap().anchor_valid_tiles, &[59, 70]);
        assert!(get_tile_data(large, 0, 0).unwrap().anchor_valid_tiles.is_empty());
        // 187: 0-5 jungle, 6-8 ash, 29-34 sands.
        let large2 = def_for_tile(187).unwrap();
        assert_eq!(get_tile_data(large2, 0, 0).unwrap().anchor_valid_tiles, &[59, 60, 226]);
        assert_eq!(get_tile_data(large2, 6, 0).unwrap().anchor_valid_tiles, &[57, 58, 75, 76]);
        assert_eq!(
            get_tile_data(large2, 29, 0).unwrap().anchor_valid_tiles,
            &[53, 397, 396, 112, 398, 400, 234, 399, 401, 116, 402, 403]
        );
        assert!(get_tile_data(large2, 9, 0).unwrap().anchor_valid_tiles.is_empty());
    }

    #[test]
    fn test_DW_1_2_sunflower_places_2x4_with_anchors() {
        let mut w = world();
        floor_of(&mut w, 2); // grass — in the valid set (:4859)
        let sunflower = def_for_tile(27).unwrap();
        // Clicking the bottom row (y = 19, origin (0,3) → origin row 16).
        let obj = must_place(&w, 10, 19, sunflower, 1);
        assert_eq!((obj.x_coord, obj.y_coord), (10, 16));
        assert!(place(&mut w, &obj));
        // The 8 cells: columns frameX 0/18, rows frameY 0/18/36/54 — the
        // legacy PlaceSunflower layout (`WorldGen.cs:53658-53668`) through the
        // generic Place math (`TileObject.cs:56-110`).
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2), (0, 3), (1, 3)] {
            let t = w.tile(10 + dx, 16 + dy).unwrap();
            assert!(t.active() && t.kind == 27, "cell ({},{})", 10 + dx, 16 + dy);
            assert_eq!((t.frame_x, t.frame_y), ((18 * dx) as i16, (18 * dy) as i16));
        }
        // The single click-point scheme: the resolved origin is always the
        // click shifted by Origin (0,3) — `TileObject.cs:839-844`.
        assert_eq!((obj.x_coord, obj.y_coord), (10, 16));
        assert!(w.frame_important[27]);
    }

    #[test]
    fn test_DW_1_2_sunflower_anchor_set_and_random_style() {
        // Every `AnchorValidTiles` entry supports the flower (:4859).
        for soil in [2u16, 477, 109, 60, 492, 633] {
            let mut w = world();
            floor_of(&mut w, soil);
            assert!(
                can_place(&w, 10, 19, def_for_tile(27).unwrap(), 0, 1, None).is_some(),
                "soil {soil} must support the sunflower"
            );
        }
        // A stone floor is solid but unlisted — refused.
        let w = world();
        assert!(
            can_place(&w, 10, 19, def_for_tile(27).unwrap(), 0, 1, None).is_none(),
            "stone is not in the sunflower's anchor set"
        );
        // RandomStyleRange resolves through the random slot
        // (`CalculatePlacementStyle` :1990-1998): placement = 0*1 + 0 + random.
        let mut w2 = world();
        floor_of(&mut w2, 2);
        let sunflower = def_for_tile(27).unwrap();
        let obj = can_place(&w2, 10, 19, sunflower, 0, 1, Some(2)).unwrap();
        assert_eq!(obj.random, 2);
        assert!(place(&mut w2, &obj));
        // frameX = CFW(36) * placement(2) + column(18) = 90 on the right cell.
        assert_eq!(w2.tile(10, 19).unwrap().frame_x, 72);
        assert_eq!(w2.tile(11, 19).unwrap().frame_x, 90);
        // forced_random folds into the range (:788-791); a clear spot for the
        // next resolution.
        let obj = can_place(&w2, 20, 19, sunflower, 0, 1, Some(5)).unwrap();
        assert_eq!(obj.random, 2);
    }

    #[test]
    fn test_DW_1_2_pots_natural_anchor_set() {
        // The pot anchor (:3556) keeps SolidTile/SolidWithTop/SolidSide and
        // drops the Style2x2 preset's Table.
        let pots = def_for_tile(28).unwrap();
        let mut w = world(); // stone floor: solid, flat
        assert!(can_place(&w, 10, 19, pots, 0, 1, None).is_some());
        let obj = must_place(&w, 10, 19, pots, 1);
        assert!(place(&mut w, &obj));
        // Cells (10..11, 18..19), frames 0/18 × 0/18 (heights 16/16).
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            let t = w.tile(10 + dx, 18 + dy).unwrap();
            assert!(t.active() && t.kind == 28);
            assert_eq!((t.frame_x, t.frame_y), ((18 * dx) as i16, (18 * dy) as i16));
        }
        // A table floor is no anchor at all: tables are neither solid nor
        // solid-top, and the preset's Table type was dropped from the pot's
        // anchor (:3556), so every branch of the check fails.
        let mut wt = world();
        floor_of(&mut wt, 14);
        assert!(can_place(&wt, 10, 19, pots, 0, 1, None).is_none());
        assert!(w.frame_important[28]);
    }

    #[test]
    fn test_DW_1_2_clay_pot_table_and_solid_anchors() {
        let pot = def_for_tile(78).unwrap();
        // On a table (:2385 keeps AnchorType.Table)…
        let mut w = world();
        set_active(&mut w, 10, 19, 14);
        assert!(can_place(&w, 10, 18, pot, 0, 1, None).is_some());
        // …on plain solid ground…
        let w2 = world();
        assert!(can_place(&w2, 10, 19, pot, 0, 1, None).is_some());
        // …and in lava it is refused (LavaDeath=false never re-allows lava —
        // the baked NotAllowed, same chain as the chest).
        let mut wl = world();
        set_liquid(&mut wl, 10, 19, 200, true);
        assert!(can_place(&wl, 10, 19, pot, 0, 1, None).is_none());
        // Water is fine (base WaterPlacement.Allowed).
        let mut ww = world();
        set_liquid(&mut ww, 10, 19, 200, false);
        assert!(can_place(&ww, 10, 19, pot, 0, 1, None).is_some());
    }

    #[test]
    fn test_DW_1_2_pile_subtile_styles_and_anchors() {
        // 186 snow style 26 sits on snow (:3293) and refuses stone…
        let large = def_for_tile(186).unwrap();
        let mut snow = world();
        floor_of(&mut snow, 147);
        assert!(can_place(&snow, 10, 19, large, 26, 1, None).is_some());
        let stone = world();
        assert!(can_place(&stone, 10, 19, large, 26, 1, None).is_none());
        // …while the base styles (0-25, no valid-tile list) take any solid
        // floor.
        assert!(can_place(&stone, 10, 19, large, 0, 1, None).is_some());
        // 187 sand style 29 on sand (:3310-3311), refuses stone; style 0 is
        // the jungle sub-tile (SubTiles[0] non-null) — refuses stone too.
        let large2 = def_for_tile(187).unwrap();
        let mut sand = world();
        floor_of(&mut sand, 53);
        assert!(can_place(&sand, 10, 19, large2, 29, 1, None).is_some());
        assert!(can_place(&sand, 10, 19, large2, 0, 1, None).is_none());
        assert!(can_place(&stone, 10, 19, large2, 29, 1, None).is_none());
        // Place a 3x2: origin (1,1), click (11, 19) → origin (10, 18). The
        // generic Place math puts style 29 on sheet row 29: wrap 35 keeps the
        // column (`num4 = 29 % 35`), so frameX = CFW(54) * 29 + dx*18,
        // frameY = dy*18 (`TileObject.cs:56-110`).
        let obj = can_place(&sand, 11, 19, large2, 29, 1, None).unwrap();
        assert_eq!((obj.x_coord, obj.y_coord), (10, 18));
        assert!(place(&mut sand, &obj));
        for dx in 0..3i32 {
            for dy in 0..2i32 {
                let t = sand.tile(10 + dx, 18 + dy).unwrap();
                assert!(t.active() && t.kind == 187, "cell ({},{})", 10 + dx, 18 + dy);
                assert_eq!(
                    (t.frame_x, t.frame_y),
                    ((54 * 29 + 18 * dx) as i16, (18 * dy) as i16),
                    "cell ({},{})",
                    10 + dx,
                    18 + dy
                );
            }
        }
        assert!(sand.frame_important[187]);
        // 185's two shapes both resolve through `can_place` (no items place
        // them in vanilla — worldgen only; the registry still judges them):
        // style 0 → the 2x1 sub-tile over a 2-wide support.
        let piles = def_for_tile(185).unwrap();
        let w = world();
        assert!(can_place(&w, 10, 19, piles, 0, 1, None).is_some());
    }

    #[test]
    fn test_DW_1_2_herbs_registered_but_legacy_routed() {
        // The registry slice: all three herb stages resolve (`def_for_tile`),
        // the seed style sub-tiles are reachable, and the `tileAlch` set
        // drives the legacy routing (`Main.cs:7806-7808`).
        for kind in [82u16, 83, 84] {
            let def = def_for_tile(kind).unwrap();
            assert_eq!(def.kind, kind);
        }
        assert_eq!(TILE_ALCH, &[82u16, 83, 84]);
        assert!(tile_alch(82) && tile_alch(83) && tile_alch(84));
        assert!(!tile_alch(28) && !tile_alch(380));
        // 380 is deliberately NOT registered (`TileObjectData` has no entry;
        // vanilla routes it through the legacy case, `WorldGen.cs:60226`).
        assert!(def_for_tile(380).is_none());
    }

    #[test]
    fn test_M7_EXTRACT_1_defs_registered_as_style_3x3() {
        // M7-EXTRACT-1 — `TileObjectData.cs:4042-4045`: both extractinators are
        // `Style3x3` + `DrawYOffset = 2` (origin (1, 2), SWTSS bottom anchor
        // over the width, 16-px coordinate cells, frame-important), no
        // alternates, `StyleHorizontal` false (the module default).
        for kind in [219u16, 642] {
            let def = def_for_tile(kind).unwrap();
            assert_eq!((def.width, def.height), (3, 3));
            assert_eq!(def.origin, Point16::new(1, 2));
            assert_eq!(def.coordinate_heights, &[16, 16, 16]);
            assert_eq!(def.coordinate_width, 16);
            assert_eq!(def.coordinate_padding, 2);
            assert!(def.frame_important);
            assert!(!def.style_horizontal);
            assert_eq!(def.alternates.len(), 0);
            assert_eq!(def.interaction, Interaction::Extractinator);
            assert_eq!(
                def.anchor_bottom,
                AnchorData::new(ANCHOR_SWTSS, 3, 0)
            );
        }
        // The tiles were NOT registered before this task — the anonymous
        // neighbours stay absent.
        assert!(def_for_tile(220).is_none());
    }

    #[test]
    fn test_M7_EXTRACT_1_placement_writes_the_3x3_frames() {
        // The click at the bottom-centre writes the 3x3 object: every cell
        // active with kind 219, frameX the column stride, frameY the row
        // stride (`TileObject.Place`, `TileObject.cs:29-172`).
        let mut w = world();
        let def = def_for_tile(219).unwrap();
        // Clicking x=10 with origin (1, 2) → the object spans x 9..11, y 17..19
        // (the click is the bottom-centre cell, as in vanilla).
        let inst = can_place(&w, 10, 19, def, 0, 1, None).expect("3x3 headroom");
        assert_eq!((inst.x_coord, inst.y_coord), (9, 17));
        assert!(place(&mut w, &inst));
        for sub_y in 0..3 {
            for sub_x in 0..3 {
                let t = w.tile(9 + sub_x, 17 + sub_y).unwrap();
                assert_eq!(t.kind, 219);
                assert_eq!((t.frame_x, t.frame_y), ((18 * sub_x) as i16, (18 * sub_y) as i16));
            }
        }
        // The origin decode is the registry-generalized one
        // (`frameX/18 % width`, `frameY/18 % height`).
        let at = object_tiles_at(&w, 11, 18).expect("a sub-tile decodes");
        assert_eq!((at.origin.x, at.origin.y), (9, 17));
    }

    #[test]
    fn test_M7_EXTRACT_1_anchors_refuse_open_air_and_kill_removes_all() {
        let mut w = world();
        let def = def_for_tile(642).unwrap();
        // A floating click (no solid floor under the object) is refused.
        assert!(can_place(&w, 10, 5, def, 0, 1, None).is_none());
        // Placed on the floor, the kill cascade removes all nine cells
        // (`KillTile`'s registry cascade, `edit::kill_tile`).
        let inst = can_place(&w, 10, 19, def, 0, 1, None).unwrap();
        assert!(place(&mut w, &inst));
        assert!(crate::edit::kill_tile(&mut w, 11, 18, false).is_some());
        for sub_y in 0..3 {
            for sub_x in 0..3 {
                assert!(!w.tile(9 + sub_x, 17 + sub_y).unwrap().active());
            }
        }
    }
}
