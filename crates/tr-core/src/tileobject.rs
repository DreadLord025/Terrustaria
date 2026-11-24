//! `TileObjectData` + `TileObject`: the multi-tile placement registry, `CanPlace`
//! and `Place`, ported 1:1.
//!
//! **S1-F-7 wave A**: the registry DATA is GENERATED —
//! `tools/GenTileObjectData/gen.py` interprets `TileObjectData.Initialize`
//! (`TileObjectData.cs:2110-5014`) statement-by-statement (the copy-on-write
//! builder chain: `newTile`/`newSubTile`/`newAlternate` cursors, `CopyFrom`
//! module sharing, `addTile`/`addSubTile`/`addAlternate`/`addBaseTile`,
//! `Calculate`'s liquid-death fold, `LinkedAlternates` propagation) and emits
//! all 389 registered tiles (604 defs incl. alternates/sub-tiles) into
//! `tileobject/registry.rs`. The 33 hand-written defs are retired; the system
//! below reads only the generated [`TABLE`]/[`BY_KIND`]. Regenerate with
//! `python tools/GenTileObjectData/gen.py` (`--check` verifies no drift).
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
//! | `WaterDeath` / `LavaDeath` :508/:541 | `water_death` / `lava_death` |
//! | `UsesCustomCanPlace` / `UsesGlobalLiquidChecks` :144/:157 | `uses_custom_can_place` / `uses_global_liquid_checks` |
//! | `GetStyleOverride` :728 | `get_style_override` (tag) + [`get_tile_data_for_tile`] resolution (W4-e11; [`resolve_tile_def`] is the d112 alias) |
//! | `HookCheckIfCanPlace` / `HookPlaceOverride` / `HookPostPlaceMyPlayer` | `hook_check_if_can_place` / `hook_place_override` / `hook_post_place_my_player` |
//! | `CallPostPlacementPlayerHook` :5269 / `OriginToTopLeft` :5285 | [`call_post_placement_player_hook`] / [`origin_to_top_left`] (W4-d114 hook wiring) |
//! | `CheckWaterDeath(type, style)` :5073 / `CheckLavaDeath(type, style)` :5097 | [`check_water_death_for_style`] / [`check_lava_death_for_style`] (W4-d114, the `ApplyNaturalObjectRules` consumers) |
//! | `CoordinateHeights` :1417 | `coordinate_heights` |
//! | `CoordinateWidth` :1503 / `CoordinatePadding` :1537 | `coordinate_width` / `coordinate_padding` |
//! | `CoordinatePaddingFix` | `coordinate_padding_fix` |
//! | `CoordinateFullWidth/Height` :1605/:1621 (`Calculate` :1805) | [`TileObjectDef::coordinate_full_width`] / [`TileObjectDef::coordinate_full_height`] |
//! | `Style` :976 / `StyleHorizontal` :943 | `style` / `style_horizontal` |
//! | `StyleWrapLimit` :1009 / `StyleLineSkip` :1108 / `StyleMultiplier` :1141 | `style_wrap_limit` / `style_line_skip` / `style_multiplier` |
//! | `Alternates` (`addAlternate` :2097) | `alternates` |
//! | `SubTiles` (`addSubTile` :2015) | `subtiles` |
//! | `TryGetTileBounds` :5163-5194 | [`try_get_tile_bounds`] (W4-d113 coordinate walk; its `GetTileData(Tile)` seam :5167 is W4-d112's `resolve_tile_def`, passed as a parameter until the merge wires it) |
//!
//! Not ported (follow-ups, by design):
//! - **Draw fields** (`DrawYOffset`, `DrawStyleOffset`, `DrawStepDown`,
//!   `DrawFlipHorizontal/Vertical`, `DrawFrameOffsets`,
//!   `StyleWrapLimitVisualOverride`): render-only (S1-F-9 DrawData scope); the
//!   generator parses them (they ride the same copy-on-write machinery) but
//!   does not emit them.
//! - **Runtime hook bodies** (`Chest.FindEmptyChest` TileObjectData.cs:3464,
//!   `Chest.AfterPlacement_Hook` :3465, `WorldGen.PlantCheck_CanPlaceHook`,
//!   the `TE*` tile-entity hooks): the generated defs carry the C# method NAME
//!   in [`PlacementHook::callback`] with `hook: None` — a documented callback
//!   seam. The bodies read runtime state (chests, tile entities, plants)
//!   outside this task; the hook *branches* in `can_place`/`place` are ported.
//!   The M3-CHEST-UI seam is unchanged: the chest is a pure tile object here.
//!   W4-d114 ports the after-placement INVOCATION seam
//!   ([`call_post_placement_player_hook`], `Player.cs:40144`/:45816 wiring)
//!   and its `OriginToTopLeft` helper ([`origin_to_top_left`]) — with every
//!   generated body still `hook: None` the seam is a behaviour-frozen no-op
//!   until the chest/TE tasks install bodies.
//! - **~~`GetStyleOverride` resolution~~ — DONE (W4-e11):** the frameY-based
//!   `GetTileData(Tile)` overload :5196-5261 is [`get_tile_data_for_tile`],
//!   with the `GetStyle_Detritus/SmallPiles/Stalactite` arms (:5294-5327),
//!   the `TryGetTileBounds` caller (:5163-5194) and the tile-based
//!   `CheckWaterDeath`/`CheckLavaDeath` (:5083-5119). The `Liquid.cs:1190`
//!   settlement kill wiring is the liquid task's seam.
//! - **`onlyCheck`/preview** (`TileObjectPreviewData.placementCache`,
//!   `objectPreview`, the anchor `Rectangle` bookkeeping TileObject.cs:219-321):
//!   dead code when `onlyCheck=false` (verified by reading; the rectangle and
//!   `num10`/`num11` are not read after :332). Smart-cursor/preview is app work.
//!   **Wave B (S1-F-7b) landed the random-style half**: the `Main.rand` path
//!   (`Main.rand.Next` TileObject.cs:787/:819) draws on the caller's
//!   `UnifiedRandom`, and [`RandomCache`] ports the `randomCache` stickiness
//!   (:764-792/:796-824). The preview's cache writeback (:826-837) stays out
//!   with the preview: `randomCache` is written ONLY inside `onlyCheck`
//!   (verified: no other writer in the tree), so a port without the preview
//!   never sticks — the cache starts `None` and every `CanPlace` draws fresh.
//!   `SpecificRandomStyles` is never assigned by `Initialize` (null in every
//!   def; the branch is ported and tested with a synthetic def). The app's
//!   placement stream (`World::place_rng`) is a dedicated `Main.rand`
//!   stand-in for this family, seeded beside `frame_rng` — the house
//!   per-consumer-stream pattern (`craft_rng`, S1-A-11).
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

/// The GENERATED registry (S1-F-7 wave A): `tools/GenTileObjectData/gen.py`
/// interprets `TileObjectData.Initialize` (`TileObjectData.cs:2110-5014`) 1:1
/// and emits every `addTile` def — width/height, style wrap/multiplier/line-skip,
/// coordinate rules, anchors, alternation, sub-tiles — plus the
/// `Main.tileFrameImportant` and `KillTile_GetItemDrops` empty-group tables.
/// Never hand-edit `registry.rs`; regenerate (`gen.py --check` verifies it).
#[path = "tileobject/registry.rs"]
mod registry;

pub use registry::{BY_KIND, FRAME_IMPORTANT, KILL_TILE_NO_CELL_DROP, TABLE};

/// `TileObjectData.GetStyleOverride` (`TileObjectData.cs:728-748`, the three
/// implementations :5294-5327). The frameY-based resolution is the tile-based
/// `GetTileData(Tile)` overload (:5196-5261) — ported as
/// [`get_tile_data_for_tile`] (W4-e11); [`resolve_tile_def`] (W4-d112) is
/// the delegating alias.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GetStyleOverride {
    #[default]
    None,
    /// `GetStyle_Detritus` (:5294-5301): `frameY >= 36` -> `SubTiles[0]`.
    Detritus,
    /// `GetStyle_SmallPiles` (:5303-5310): `frameY >= 18` -> `SubTiles[0]`.
    SmallPiles,
    /// `GetStyle_Stalactite` (:5312-5327): `frameY >= 90/72/36` -> `SubTiles[2/1/0]`.
    Stalactite,
}

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

/// `Terraria.Enums.LiquidPlacement` (`LiquidPlacement.cs:3-9`).
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
    /// The C# hook method name (`Chest.FindEmptyChest`,
    /// `WorldGen.PlantCheck_CanPlaceHook`, the `TE*` hooks, ...). Wave A keeps
    /// the runtime hooks as documented callbacks: `hook` stays `None` for the
    /// generated defs and this name records which body the consumer must wire
    /// (the hook bodies read runtime state — chests, tile entities, plants —
    /// outside this task). The wooden slice's hand defs carried `None` here.
    pub callback: &'static str,
    pub bad_return: i32,
    pub bad_response: i32,
    pub processed_coordinates: bool,
}

impl PlacementHook {
    /// `PlacementHook.Empty` (`PlacementHook.cs:51-54`).
    pub const EMPTY: Self = Self {
        hook: None,
        callback: "",
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
    /// `WaterDeath` (`TileObjectData.cs:508-539`) — the liquid-death flag
    /// `Calculate` (:1827-1830) folds into `WaterPlacement` and
    /// `CheckWaterDeath` (:5073-5095) reads. Generated.
    pub water_death: bool,
    /// `LavaDeath` (`TileObjectData.cs:541-572`) — folded into `LavaPlacement`
    /// by `Calculate` (:1823-1826); `CheckLavaDeath` (:5097-5119) reads it.
    pub lava_death: bool,
    /// `UsesCustomCanPlace` (:144-155) — the `CustomPlace(type, style)` routing
    /// flag (:5016-5037): `Player.PlaceThing_Tiles_TryPlacing`
    /// (`Player.cs:39771`) sends only these kinds through `TileObject.CanPlace`.
    pub uses_custom_can_place: bool,
    /// `UsesGlobalLiquidChecks` (:157-168) — `ApplyNaturalObjectRules`
    /// (:1847-1851) sets it; `CheckWaterDeath`/`CheckLavaDeath` then read the
    /// global `Main.tile*Death` tables instead of the def's flags.
    pub uses_global_liquid_checks: bool,
    /// `GetStyleOverride` (:728-748) — the frameY-based style resolver tag.
    pub get_style_override: GetStyleOverride,
    pub hook_check_if_can_place: PlacementHook,
    pub hook_place_override: PlacementHook,
    /// `HookPostPlaceMyPlayer` (:684-704) — the tile-entity / chest
    /// after-placement hook (`CallPostPlacementPlayerHook` :5269-5283).
    /// Documented callback seam (wave A): the body reads runtime state.
    pub hook_post_place_my_player: PlacementHook,
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

/// The read side of `TileObjectPreviewData.randomCache`
/// (`TileObjectPreviewData.cs:29-31`, the fields `CanPlace` reads at
/// `TileObject.cs:769-774`/:801-806: `Type`, `Coordinates`, `ObjectStart`,
/// `Random`). The C# static lives on [`World::tile_object_random_cache`];
/// a fresh `new TileObjectPreviewData()` zero-initialises every field
/// (`Random = 0`, NOT `Reset()`'s -1 — `Reset` :177-192 is never called on
/// the random cache). The writeback (`TileObject.cs:826-837`) happens only
/// under `onlyCheck`, which the port does not run (module doc), so the
/// production cache stays `None` and every `CanPlace` draws fresh; tests
/// inject a cache to exercise the stickiness branch 1:1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct RandomCache {
    /// `TileObjectPreviewData.Type` (ushort).
    pub kind: u16,
    /// `TileObjectPreviewData.Coordinates`.
    pub coordinates: Point16,
    /// `TileObjectPreviewData.ObjectStart`.
    pub object_start: Point16,
    /// `TileObjectPreviewData.Random`.
    pub random: i32,
}

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
        water_death: false,
        lava_death: false,
        uses_custom_can_place: false,
        uses_global_liquid_checks: false,
        get_style_override: GetStyleOverride::None,
        hook_check_if_can_place: PlacementHook::EMPTY,
        hook_place_override: PlacementHook::EMPTY,
        hook_post_place_my_player: PlacementHook::EMPTY,
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

/// `Main.tileHammer` (`Main.cs:7204-7207`, 4 ids): the tiles only a
/// hammer breaks.
const TILE_HAMMER: &[u16] = &[26, 695, 31, 696];

/// `Main.tileAxe` (`Main.cs:7208-7223`, 16 ids): the tiles only an axe
/// breaks (the tree trunks and the vanity/gem tops).
const TILE_AXE: &[u16] = &[
    5, 72, 80, 488, 704, 323, 596, 616, 634, 589, 584, 588, 586, 587, 585, 583,
];

/// `TileID.Sets.IsATreeTrunk` (`TileID.cs:487`): the tree-trunk tiles (the
/// oak 5, the cacti 72, the gem/vanity/ash tops).
const IS_A_TREE_TRUNK: &[u16] = &[
    5, 72, 583, 584, 585, 586, 587, 588, 589, 596, 616, 634,
];

/// `TileID.Sets.IgnoreSmartCursorPriorityAxe` (`TileID.cs:508`): the one
/// axe tile the smart-cursor axe step ignores (the fallen log 488).
const IGNORE_SMART_AXE: &[u16] = &[488];

/// `TileObjectData.GetTileData(type)` — the tile's base definition
/// (`TileObjectData.cs:5126-5194` without the sub-tile/alternate walk).
pub fn def_for_tile(kind: u16) -> Option<&'static TileObjectDef> {
    // `_data[type]` (`TileObjectData.cs:5136`) — the generated per-kind table.
    BY_KIND.get(kind as usize).copied().flatten()
}

/// `TileObjectData.CustomPlace(type, style)` (`TileObjectData.cs:5016-5037`):
/// the placement-routing flag — a registered def whose style-resolved
/// `UsesCustomCanPlace` is true. `Player.PlaceThing_Tiles_TryPlacing`
/// (`Player.cs:39771`) gates the `TileObject.CanPlace` route on this (with the
/// explicit 82/227/4 exclusions), NOT on mere registration: the natural
/// objects (pots 28, piles 185/186/187, plants, platforms 19) are registered
/// but route through the legacy `WorldGen.PlaceTile`.
pub fn custom_place(kind: u16, style: i32) -> bool {
    let Some(def) = def_for_tile(kind) else {
        return false;
    };
    if style < 0 {
        return false;
    }
    // :5027-5035 — the style's sub-tile flag wins when present.
    if let Some((_, sub)) = def.subtiles.iter().find(|(s, _)| *s as i32 == style) {
        return sub.uses_custom_can_place;
    }
    def.uses_custom_can_place
}

/// `Main.tileFrameImportant[kind]` — the generated static table (the port's
/// `World.frame_important` is a per-world mirror; this is the boot truth).
pub fn frame_important(kind: u16) -> bool {
    FRAME_IMPORTANT.get(kind as usize).copied().unwrap_or(false)
}

/// The `Player.PlaceThing_Tiles_TryPlacing` registry-route gate
/// (`Player.cs:39771-39772`): `TileObjectData.CustomPlace(tileToCreate,
/// placeStyle) && tileToCreate != 82 && tileToCreate != 227 && tileToCreate
/// != 4`. The kinds that pass ride `TileObject.CanPlace`/`Place`; everything
/// else (the natural objects — pots, piles, plants, platforms, torches, dye
/// plants — and the unregistered kinds) keeps the legacy `WorldGen.PlaceTile`
/// route with the app's neighbour/overlap pre-checks. S1-F-7: the gate is the
/// `CustomPlace` flag, not mere registration (the full registry registers the
/// natural objects too).
pub fn routes_through_tile_object(kind: u16, style: i32) -> bool {
    kind != 82 && kind != 227 && kind != 4 && custom_place(kind, style)
}

/// `WorldGen.KillTile_GetItemDrops`' empty case group (`WorldGen.cs:63988-66090`):
/// the kind drops nothing per-cell (its loot rides the `Check*` cascade record).
pub fn kill_tile_no_cell_drop(kind: u16) -> bool {
    KILL_TILE_NO_CELL_DROP.get(kind as usize).copied().unwrap_or(false)
}

/// `TileObjectData.GetTileData(type, style, alternate)` for an already-resolved
/// tile def (`TileObjectData.cs:5126-5161`): `style` picks a sub-tile,
/// `alternate - 1` indexes the alternates (`alternate = 0` → the def itself,
/// which is how the `CanPlace` loop addresses the base def); an out-of-range
/// alternate keeps the resolved def (W4-e12, :5150-5159).
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
    // :5148-5157 — `alternate--`, then index the alternates. C# replaces the
    // def ONLY with a non-null in-range entry; a null slot or an index past
    // the list KEEPS the current def (:5152-5159). W4-e12: the old `?` here
    // answered None on an out-of-range alternate — a divergence for the
    // `TileObject.Place`/`DrawPreview` callers that carry an arbitrary
    // `alternate` (the `CanPlace` loop only ever passes 0..=AlternatesCount,
    // :203-208, so it never noticed).
    let alt = alternate - 1;
    if alt >= 0 {
        if let Some(a) = resolved.alternates.get(alt as usize) {
            resolved = a;
        }
    }
    Some(resolved)
}

/// `TileObjectData.GetTileData(Tile)` (`TileObjectData.cs:5196-5261`) — the
/// frame-based resolution consumer (W4-d112, S1-F-7 wave B): decodes the
/// stored `frameX`/`frameY` into a style/alternate pair and walks the def's
/// sub-tiles and alternates. The `GetStyleOverride` hook (:5222-5226) wins
/// over the whole decode when present ([`apply_style_override`], the three
/// `GetStyle_*` bodies :5294-5327).
///
/// C# deviations (documented): a null `Tile` is the caller's `world.tile()`
/// `None`; the out-of-range-type throw (:5203-5206) degrades to `None` like
/// [`def_for_tile`] (the port never indexes past `BY_KIND`). Integer division
/// truncates toward zero in both languages (negative frames degrade the same
/// way). The sub-tile null-padding (`addSubTile` :2085-2091) is the port's
/// (style, def) pair lookup — a null slot keeps the current def, 1:1.
pub fn resolve_tile_def(tile: &Tile) -> Option<&'static TileObjectDef> {
    // W4-d112's port of the same overload; the merge keeps ONE body —
    // e11's `get_tile_data_for_tile` (the split `resolve_from_frames`
    // shape the synthetic-def witnesses pin). The name stays for the
    // d112 witness set + the framing consumers.
    get_tile_data_for_tile(tile)
}

/// The alternate-window scan of `GetTileData(Tile)` (`TileObjectData.cs:5249-
/// 5259`): the first alternate whose `[Style, Style + RandomStyleRange]`
/// window contains the decoded offset wins — the upper bound is INCLUSIVE
/// and list order breaks overlapping windows.
fn alternate_in_style_range(
    def: &'static TileObjectDef,
    offset: i32,
) -> Option<&'static TileObjectDef> {
    for alt in def.alternates {
        if offset >= alt.style && offset <= alt.style + alt.random_style_range {
            return Some(alt);
        }
    }
    None
}

/// The three `GetStyleOverride` bodies (`TileObjectData.cs:5294-5327`), called
/// from [`resolve_tile_def`] at :5222-5226. Each indexes `current.SubTiles`
/// POSITIONALLY (C# list index, not style): the generated sub-tile pairs of
/// the three override kinds are dense from 0 (`addSubTile(0)` :4973/:4984,
/// `addSubTile(0/1/2)` :4999/:5005/:5011), so the pair slice index is the C#
/// list index. A missing slot degrades to `None` (C# throws).
fn apply_style_override(
    tag: GetStyleOverride,
    tile: &Tile,
    current: &'static TileObjectDef,
) -> Option<&'static TileObjectDef> {
    let sub = |i: usize| current.subtiles.get(i).map(|(_, def)| *def);
    match tag {
        GetStyleOverride::None => Some(current),
        // GetStyle_Detritus (:5294-5301).
        GetStyleOverride::Detritus => {
            if tile.frame_y >= 36 {
                sub(0)
            } else {
                Some(current)
            }
        }
        // GetStyle_SmallPiles (:5303-5310).
        GetStyleOverride::SmallPiles => {
            if tile.frame_y >= 18 {
                sub(0)
            } else {
                Some(current)
            }
        }
        // GetStyle_Stalactite (:5312-5327) — descending thresholds.
        GetStyleOverride::Stalactite => {
            if tile.frame_y >= 90 {
                sub(2)
            } else if tile.frame_y >= 72 {
                sub(1)
            } else if tile.frame_y >= 36 {
                sub(0)
            } else {
                Some(current)
            }
        }
    }
}


/// `TileObjectData.CallPostPlacementPlayerHook` (`TileObjectData.cs:5269-5283`)
/// — the after-placement hook INVOCATION consumer (W4-d114): resolves
/// `GetTileData(type, style, data.alternate)`, subtracts the def's `Origin`
/// when the hook wants processed coordinates, and calls the hook. Returns
/// `hookResult == badReturn` — the C# quirk verbatim: for the chest
/// (`badReturn = -1`, `Chest.AfterPlacement_Hook` returns -1 on FAILURE,
/// `Chest.cs:559-567`) this answers `true` when the hook FAILED. Both C#
/// call sites (`Player.cs:40144`, `:45816`) ignore the return value.
///
/// The `alternate` parameter is part of the C# signature but the body reads
/// only `data.alternate` (:5271/:5276/:5282) — kept 1:1. Wave A keeps every
/// generated hook body as a documented callback (`hook: None`, see
/// [`PlacementHook::callback`]), so this answers `false` for every registry
/// def today; the invocation seam is what the chest/TE tasks wire into.
/// A negative style throws in C# (:5132-5135); the port degrades to `false`
/// (the [`check_liquid_placement`] precedent).
#[allow(unused_variables)]
pub fn call_post_placement_player_hook(
    world: &World,
    tile_x: i32,
    tile_y: i32,
    kind: u16,
    style: i32,
    dir: i32,
    alternate: i32,
    data: &TileObjectInstance,
) -> bool {
    if style < 0 {
        return false;
    }
    // :5271 — `GetTileData(type, style, data.alternate)`.
    let Some(def) = def_for_tile(kind) else {
        return false;
    };
    let Some(tile_data) = get_tile_data(def, style, data.alternate) else {
        return false;
    };
    call_post_placement_player_hook_with_def(
        world, tile_x, tile_y, kind, style, dir, tile_data, data,
    )
}

/// The body of [`call_post_placement_player_hook`] after the :5271
/// resolution — split out like [`place`]/[`place_with_def`] so witnesses can
/// inject a hook body outside the generated registry.
#[allow(unused_variables)]
pub fn call_post_placement_player_hook_with_def(
    world: &World,
    tile_x: i32,
    tile_y: i32,
    kind: u16,
    style: i32,
    dir: i32,
    tile_data: &TileObjectDef,
    data: &TileObjectInstance,
) -> bool {
    // :5272-5275 — the `_placementHooks == null` / `hook == null` gates (the
    // port folds both into `hook: None`).
    let Some(hook_fn) = tile_data.hook_post_place_my_player.hook else {
        return false;
    };
    let hook = tile_data.hook_post_place_my_player;
    let (mut x, mut y) = (tile_x, tile_y);
    // :5277-5281 — processed coordinates arrive origin-relative.
    if hook.processed_coordinates {
        x -= tile_data.origin.x;
        y -= tile_data.origin.y;
    }
    // :5282 — `== badReturn`, NOT `!=` (see the doc quirk).
    hook_fn(world, x, y, kind, style, dir, data.alternate) == hook.bad_return
}

/// `TileObjectData.OriginToTopLeft` (`TileObjectData.cs:5285-5292`) — the
/// coordinate helper `Chest.AfterPlacement_Hook` opens with (`Chest.cs:562`):
/// a resolved def shifts the anchor point back by its `Origin`; an
/// unregistered kind (or a negative style — the C# throw, degraded like
/// [`call_post_placement_player_hook`]) leaves the point unchanged. The C#
/// `ref Point16` becomes a returned value.
pub fn origin_to_top_left(kind: u16, style: i32, base_coords: Point16) -> Point16 {
    if style < 0 {
        return base_coords;
    }
    if let Some(def) = def_for_tile(kind) {
        if let Some(tile_data) = get_tile_data(def, style, 0) {
            return Point16::new(
                base_coords.x - tile_data.origin.x,
                base_coords.y - tile_data.origin.y,
            );
        }
    }
    base_coords
}

/// `TileObjectData.CheckWaterDeath(int type, int style)`
/// (`TileObjectData.cs:5073-5081`) — the style-indexed liquid-death consumer
/// of `ApplyNaturalObjectRules` (:1847-1851 sets `UsesGlobalLiquidChecks`;
/// W4-d114 takes the (type, style) overloads, the Tile overloads :5083-5119
/// are W4-e11's). A def that opted out of the global checks answers its own
/// `WaterDeath`; a natural-object def (or an unregistered kind) answers the
/// global `Main.tileWaterDeath` table ([`TILE_WATER_DEATH`]). Negative style:
/// the C# throw degrades to the global table (the
/// [`check_liquid_placement`] precedent).
pub fn check_water_death_for_style(kind: u16, style: i32) -> bool {
    if style >= 0 {
        if let Some(def) = def_for_tile(kind) {
            if let Some(td) = get_tile_data(def, style, 0) {
                if !td.uses_global_liquid_checks {
                    return td.water_death;
                }
            }
        }
    }
    TILE_WATER_DEATH.contains(&kind)
}

/// `TileObjectData.CheckLavaDeath(int type, int style)`
/// (`TileObjectData.cs:5097-5105`) — the lava twin of
/// [`check_water_death_for_style`]; the global table is the generated
/// `TILE_LAVA_DEATH` bitset (`Main.tileLavaDeath`, `Main.cs:7233-8218`).
pub fn check_lava_death_for_style(kind: u16, style: i32) -> bool {
    if style >= 0 {
        if let Some(def) = def_for_tile(kind) {
            if let Some(td) = get_tile_data(def, style, 0) {
                if !td.uses_global_liquid_checks {
                    return td.lava_death;
                }
            }
        }
    }
    crate::tiledata::TILE_LAVA_DEATH.has(kind)
}

/// `TileObjectData.GetTileData(Tile)` (`TileObjectData.cs:5196-5261`) — the
/// frameY/frameX-based style RESOLUTION consumer of the registry (the tile
/// counterpart of the index-based [`get_tile_data`]). The frame pair decodes
/// a style index through `CoordinateFullWidth/Height`, `StyleWrapLimit`,
/// `StyleHorizontal` and `StyleMultiplier`; a `GetStyleOverride` tag wins
/// over that decode, `StyleLineSkip > 1` re-decodes, then `SubTiles[style]`
/// narrows and the alternates' `[Style, Style + RandomStyleRange]` bands
/// answer. W4-e11 (the d111-researched "open half").
pub fn get_tile_data_for_tile(tile: &Tile) -> Option<&'static TileObjectDef> {
    // :5198-5201 — a null/inactive tile answers null (the port's Option-tile
    // callers pass a live `&Tile`, so only the active flag gates).
    if !tile.active() {
        return None;
    }
    // :5202-5211 — `_data[type]`; an out-of-range type throws in C# (a
    // corrupt save), the port answers None like an unregistered kind.
    let base = def_for_tile(tile.kind)?;
    resolve_from_frames(base, tile)
}

/// The :5212-5260 body of [`get_tile_data_for_tile`] — the frame decode and
/// sub-tile/alternate walk over an already-looked-up base def (split out so
/// synthetic defs can pin the branches the generated table never combines).
/// `None` only from an override arm whose `SubTiles[i]` slot is null (the
/// C# returns the null element; every generated override def has the slot).
fn resolve_from_frames(
    base: &'static TileObjectDef,
    tile: &Tile,
) -> Option<&'static TileObjectDef> {
    // :5212-5221 — the frame decode. `frame_x`/`frame_y` are i16 in the port
    // (C# short); the division truncates exactly like C#.
    let num = tile.frame_x as i32 / base.coordinate_full_width();
    let num2 = tile.frame_y as i32 / base.coordinate_full_height();
    let wrap = if base.style_wrap_limit == 0 {
        1
    } else {
        base.style_wrap_limit
    };
    let num4 = if base.style_horizontal {
        num2 * wrap + num
    } else {
        num * wrap + num2
    };
    let mut style = num4 / base.style_multiplier;
    let mut remainder = num4 % base.style_multiplier;
    // :5222-5226 — the override delegate short-circuits the whole walk.
    match base.get_style_override {
        GetStyleOverride::None => {}
        // GetStyle_Detritus (:5294-5301): frameY >= 36 -> SubTiles[0].
        GetStyleOverride::Detritus => {
            return if tile.frame_y >= 36 {
                subtile_at(base, 0)
            } else {
                Some(base)
            };
        }
        // GetStyle_SmallPiles (:5303-5310): frameY >= 18 -> SubTiles[0].
        GetStyleOverride::SmallPiles => {
            return if tile.frame_y >= 18 {
                subtile_at(base, 0)
            } else {
                Some(base)
            };
        }
        // GetStyle_Stalactite (:5312-5327): frameY >= 90/72/36 ->
        // SubTiles[2/1/0].
        GetStyleOverride::Stalactite => {
            let idx = if tile.frame_y >= 90 {
                2
            } else if tile.frame_y >= 72 {
                1
            } else if tile.frame_y >= 36 {
                0
            } else {
                return Some(base);
            };
            return subtile_at(base, idx);
        }
    }
    // :5227-5240 — StyleLineSkip > 1 re-decodes from the line grid.
    let skip = base.style_line_skip;
    if skip > 1 {
        if base.style_horizontal {
            style = num2 / skip * wrap + num;
            remainder = num2 % skip;
        } else {
            style = num / skip * wrap + num2;
            remainder = num % skip;
        }
    }
    // :5241-5248 — SubTiles[style] when present (null-padded list in C#; the
    // port stores the non-null (style, def) pairs — a null slot keeps the
    // current def, exactly like the C# null check).
    let mut resolved = base;
    if style >= 0 {
        if let Some(sub) = subtile_at(base, style) {
            resolved = sub;
        }
    }
    // :5249-5258 — the resolved def's alternates answer on the remainder band
    // `[Style, Style + RandomStyleRange]` (inclusive both ends).
    for alt in resolved.alternates {
        if remainder >= alt.style && remainder <= alt.style + alt.random_style_range {
            return Some(alt);
        }
    }
    Some(resolved)
}

/// The `SubTiles[index]` lookup shared by [`get_tile_data_for_tile`] and the
/// override arms — the C# null-padded list indexed directly; the port's
/// (style, def) pairs answer None for a null slot.
fn subtile_at(def: &'static TileObjectDef, index: i32) -> Option<&'static TileObjectDef> {
    def.subtiles
        .iter()
        .find(|(style, _)| *style as i32 == index)
        .map(|(_, sub)| *sub)
}

/// `TileObjectData.TryGetTileBounds` (`TileObjectData.cs:5163-5194`) — the
/// object bounding box a cell belongs to, through the frame-resolved def
/// ([`get_tile_data_for_tile`]): the frame remainder walks back left over
/// `CoordinateWidth` columns and up over the `CoordinateHeights` rows.
/// `None` for an inactive/unregistered cell (the C# `false` + 1x1 default).
pub fn try_get_tile_bounds(world: &World, x: i32, y: i32) -> Option<TileBounds> {
    let tile = world.tile(x, y)?;
    let def = get_tile_data_for_tile(tile)?;
    // :5172-5175 — the full-object grid index and the in-object remainder.
    let full_w = def.coordinate_full_width();
    let full_h = def.coordinate_full_height();
    let num = tile.frame_x as i32 / full_w;
    let num2 = tile.frame_y as i32 / full_h;
    let mut num3 = tile.frame_x as i32 - num * full_w;
    let mut num4 = tile.frame_y as i32 - num2 * full_h;
    let mut bounds = TileBounds {
        x,
        y,
        width: 1,
        height: 1,
    };
    // :5176-5181 — walk left over the coordinate columns.
    while num3 >= def.coordinate_width {
        num3 -= def.coordinate_width;
        bounds.x -= 1;
    }
    // :5182-5190 — walk up over the coordinate rows.
    for h in def.coordinate_heights {
        if num4 >= *h {
            num4 -= h;
            bounds.y -= 1;
        }
    }
    // :5191-5192 — the resolved def's object size.
    bounds.width = def.width;
    bounds.height = def.height;
    Some(bounds)
}

/// The `Rectangle` out-parameter of [`try_get_tile_bounds`]
/// (`TileObjectData.cs:5165`, `System.Drawing.Rectangle` x/y/width/height).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// `TileObjectData.CheckWaterDeath(Tile)` (`TileObjectData.cs:5083-5095`) —
/// the frame-resolved def's `WaterDeath`, or the global `Main.tileWaterDeath`
/// table when no def resolves / the def uses global checks. Read by the
/// liquid settlement kill (`Liquid.cs:1195`) and the rain kill
/// (`WorldGen.cs:72930`).
pub fn check_water_death(tile: &Tile) -> bool {
    if !tile.active() {
        return false;
    }
    match get_tile_data_for_tile(tile) {
        Some(def) if !def.uses_global_liquid_checks => def.water_death,
        _ => TILE_WATER_DEATH.contains(&tile.kind),
    }
}

/// `TileObjectData.CheckLavaDeath(Tile)` (`TileObjectData.cs:5107-5119`) —
/// the lava twin, read by `Liquid.cs:1190` and `WorldGen.cs:80439`.
pub fn check_lava_death(tile: &Tile) -> bool {
    if !tile.active() {
        return false;
    }
    match get_tile_data_for_tile(tile) {
        Some(def) if !def.uses_global_liquid_checks => def.lava_death,
        _ => crate::tiledata::TILE_LAVA_DEATH.has(tile.kind),
    }
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
/// `pub`: the torch placement pre-check reads it (`Player.cs:41158-41169`).
pub fn nactive(tile: &Tile) -> bool {
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

/// `Main.tileWaterDeath` (`Main.cs` `tileWaterDeath` assignments, 10 ids:
/// `:8061-8067` the 215/4/51/697/93/98/552 block, `:8309` the 405, `:9504` the
/// 646, `:9508` the 372). A literal extraction like `torch.rs::NO_ATTACH` —
/// `tools/GenTileData` emits `TileID.Sets` only, not the `Main.`-prefixed
/// death tables. Read by the global `LiquidPlace` fallback
/// (`TileObjectData.cs:5060-5067`).
const TILE_WATER_DEATH: &[u16] = &[215, 4, 51, 697, 93, 98, 552, 405, 646, 372];

/// `TileObjectData.CheckLiquidPlacement` (`TileObjectData.cs:5039-5042`):
/// `GetTileData(type, style)?.LiquidPlace(checkTile) ?? LiquidPlace(type,
/// checkTile)`. A registered def answers through its own `LiquidPlace`
/// ([`liquid_place`]); an unregistered kind falls back to the global
/// `LiquidPlace` (`:5044-5071`) — lava refuses a `tileLavaDeath` kind, any
/// other liquid refuses a `tileWaterDeath` kind, an empty cell always passes.
/// Read by `Player.PlaceThing_Tiles_IsBlockedByLava` (`Player.cs:41498`).
pub fn check_liquid_placement(kind: u16, style: i32, tile: &Tile) -> bool {
    // `GetTileData(type, style)` returns null on a negative style
    // (`TileObjectData.cs:5129` throws → the `?.` yields null), so the global
    // fallback answers; guard before `get_tile_data`'s debug_assert.
    if style >= 0 {
        if let Some(def) = def_for_tile(kind) {
            if let Some(td) = get_tile_data(def, style, 0) {
                return liquid_place(td, tile);
            }
        }
    }
    // :5050-5069 — the global fallback.
    if tile.liquid > 0 {
        match tile.liquid_kind() {
            crate::tile::LiquidKind::Lava => !crate::tiledata::TILE_LAVA_DEATH.has(kind),
            _ => !TILE_WATER_DEATH.contains(&kind),
        }
    } else {
        true
    }
}

/// Membership in [`TILE_CUT`] — read by the placement cut rule
/// (`Player.cs:41303`) and the door sweep (`WorldGen.cs:37704-37717`).
pub fn tile_cut(kind: u16) -> bool {
    TILE_CUT.contains(&kind)
}

/// `Player.PlaceThing_IsReplaceableBlock` (`Player.cs:39753-39760`): a
/// `tileCut` target is replaceable EXCEPT the two named exemptions (484,
/// 711); the drip-tile band 373-375 plus 461/709 is always replaceable;
/// everything else answers `TileID.Sets.BreakableWhenPlacing`. Read by the
/// lava gate's OR chain (`Player.cs:39743`).
pub fn is_replaceable_block(kind: u16) -> bool {
    if (tile_cut(kind) && kind != 484 && kind != 711)
        || (373..=375).contains(&kind)
        || kind == 461
        || kind == 709
    {
        return true;
    }
    breakable_when_placing(kind)
}

/// Membership in [`BREAKABLE_WHEN_PLACING`] (`Player.cs:41303`).
pub fn breakable_when_placing(kind: u16) -> bool {
    BREAKABLE_WHEN_PLACING.contains(&kind)
}

/// Membership in [`TILE_HAMMER`] (`Main.tileHammer`, `Main.cs:7204-7207`).
pub fn tile_hammer(kind: u16) -> bool {
    TILE_HAMMER.contains(&kind)
}

/// Membership in [`TILE_AXE`] (`Main.tileAxe`, `Main.cs:7208-7223`).
pub fn tile_axe(kind: u16) -> bool {
    TILE_AXE.contains(&kind)
}

/// Membership in [`IS_A_TREE_TRUNK`] (`TileID.Sets.IsATreeTrunk`,
/// `TileID.cs:487`).
pub fn is_a_tree_trunk(kind: u16) -> bool {
    IS_A_TREE_TRUNK.contains(&kind)
}

/// Membership in [`IGNORE_SMART_AXE`]
/// (`TileID.Sets.IgnoreSmartCursorPriorityAxe`, `TileID.cs:508`).
pub fn ignore_smart_axe(kind: u16) -> bool {
    IGNORE_SMART_AXE.contains(&kind)
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
/// (`frameX >= 144` — the grown frame). The `AttackMelee` context (the
/// bug net's swing cut, `Player.cs:44661`) reads the same body.
pub fn can_cut_tile(world: &World, x: i32, y: i32) -> bool {
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
/// 1 right). `rand` is `Main.rand` (`UnifiedRandom`, the app's
/// [`World::place_rng`] stream) and `random_cache` is
/// `TileObjectPreviewData.randomCache` ([`World::tile_object_random_cache`]).
pub fn can_place(
    world: &World,
    x: i32,
    y: i32,
    def: &'static TileObjectDef,
    style: i32,
    dir: i32,
    forced_random: Option<i32>,
    rand: &mut crate::unified_random::UnifiedRandom,
    random_cache: Option<RandomCache>,
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

    // :761-793 — random styles (wave B: the Main.rand path + the randomCache
    // stickiness). The C# null-creates a zero-initialised cache (:764-767);
    // `None` models that fresh `new TileObjectPreviewData()`.
    let cache = random_cache.unwrap_or_default();
    let mut num37 = -1i32;
    if has_random_styles {
        // :768-785 — flag10: the cache is stale when the type differs or the
        // cached object start moved. `tileData` here is the style-resolved
        // base def (:176).
        let mut flag10 = false;
        if cache.kind as i32 == def.kind as i32 {
            let num38 = cache.coordinates.x + cache.object_start.x;
            let num39 = cache.coordinates.y + cache.object_start.y;
            let num40 = x - base.origin.x;
            let num41 = y - base.origin.y;
            if num38 != num40 || num39 != num41 {
                flag10 = true;
            }
        } else {
            flag10 = true;
        }
        // :786-791 — the draw is spent even when forcedRandom overwrites it
        // (the C# evaluates `Main.rand.Next` first).
        let random_style_range = base.random_style_range;
        let mut num42 = rand.next_max(random_style_range);
        if let Some(forced) = forced_random {
            num42 = forced.rem_euclid(random_style_range);
        }
        // :792
        num37 = if !flag10 && forced_random.is_none() {
            cache.random
        } else {
            num42
        };
    }
    // :794-825 — SpecificRandomStyles (null in every generated def — the
    // branch stays ported for synthetic defs).
    if !base.specific_random_styles.is_empty() {
        // :800-817 — flag11, the same cache check.
        let mut flag11 = false;
        if cache.kind as i32 == def.kind as i32 {
            let num43 = cache.coordinates.x + cache.object_start.x;
            let num44 = cache.coordinates.y + cache.object_start.y;
            let num45 = x - base.origin.x;
            let num46 = y - base.origin.y;
            if num43 != num45 || num44 != num46 {
                flag11 = true;
            }
        } else {
            flag11 = true;
        }
        // :818-823 — second draw, spent even under forcedRandom.
        let num47 = base.specific_random_styles.len() as i32;
        let mut num48 = rand.next_max(num47);
        if let Some(forced) = forced_random {
            num48 = forced.rem_euclid(num47);
        }
        // :824
        num37 = if !flag11 && forced_random.is_none() {
            cache.random
        } else {
            base.specific_random_styles[num48 as usize] - style
        };
    }
    // :826-837 — the onlyCheck preview writeback (module doc): randomCache is
    // written only there, so the production cache stays as passed in.

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
                    crate::framing::reframe_area(world, num6 + i, num7 + j, 2, true);
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
    /// The SolidTile|SolidWithTop|SolidSide anchor shorthand the hand defs
    /// used (the generated table spells the numeric flags out).
    const ANCHOR_SWTSS: u16 =
        anchor_type::SOLID_TILE | anchor_type::SOLID_WITH_TOP | anchor_type::SOLID_SIDE;
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
        can_place(
            w,
            x,
            y,
            def,
            0,
            dir,
            None,
            &mut crate::unified_random::UnifiedRandom::new(0),
            None,
        )
        .unwrap_or_else(|| panic!("expected placement at ({x}, {y})"))
    }

    /// Test shorthand for `can_place` on a fresh seed-0 stream with no
    /// random cache — the pre-wave-B behaviour of the non-random defs
    /// (their draws never fire).
    fn cp(
        w: &World,
        x: i32,
        y: i32,
        def: &'static TileObjectDef,
        style: i32,
        dir: i32,
        forced: Option<i32>,
    ) -> Option<TileObjectInstance> {
        can_place(
            w,
            x,
            y,
            def,
            style,
            dir,
            forced,
            &mut crate::unified_random::UnifiedRandom::new(0),
            None,
        )
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
        // S1-F-7: the generated table carries the C# sub-tiles — style 13 (the
        // liquid variant, :3182-3185) and style 25 (the 1-tall variant,
        // :3186-3190). The hand slice left these empty (a follow-up then).
        assert_eq!(
            table.subtiles.iter().map(|(s, _)| *s).collect::<Vec<_>>(),
            vec![13, 25]
        );
        assert_eq!(get_tile_data(&table, 25, 0).unwrap().height, 1); // :3187

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
        assert!(cp(&w2, 10, 19, bench, 0, 1, None).is_none());
        assert!(cp(&w2, 11, 19, bench, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_can_place_world_bounds_rules() {
        let w = world();
        let bench = def_for_tile(18).unwrap();
        // :182-187 — the object must fit inside the world.
        assert!(cp(&w, 39, 19, bench, 0, 1, None).is_none());
        // :213-218 — every alternate must clear the 5-tile border
        // (`return false`): x = 34 fails (34 + 2 > 35), x = 33 passes.
        assert!(cp(&w, 34, 19, bench, 0, 1, None).is_none());
        assert!(cp(&w, 4, 19, bench, 0, 1, None).is_none());
        assert!(cp(&w, 33, 19, bench, 0, 1, None).is_some());
        assert!(cp(&w, 5, 19, bench, 0, 1, None).is_some());
    }

    #[test]
    fn test_DW_1_2_can_place_requires_anchor() {
        let w = world();
        let bench = def_for_tile(18).unwrap();
        // Floating: no AnchorBottom support.
        assert!(cp(&w, 10, 10, bench, 0, 1, None).is_none());
        // Partial support: the 2-wide bottom anchor needs both tiles solid.
        let mut w2 = world();
        let idx_11_20 = w2.index(11, 20);
        w2.tiles[idx_11_20] = Tile::default(); // hole under the right half
        assert!(cp(&w2, 10, 19, bench, 0, 1, None).is_none());
        // A table (3 wide) over a 2-wide ledge fails.
        let table = def_for_tile(14).unwrap();
        let mut w3 = world();
        let idx_12_20 = w3.index(12, 20);
        w3.tiles[idx_12_20] = Tile::default();
        assert!(cp(&w3, 11, 19, table, 0, 1, None).is_none());
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
        assert!(cp(&w, 10, 19, bench, 0, 1, None).is_some());
        // Lava is not (`LavaPlacement.NotAllowed` — `LiquidPlace` :1862-1871).
        let mut w2 = world();
        set_liquid(&mut w2, 10, 19, 200, true);
        set_liquid(&mut w2, 11, 19, 200, true);
        assert!(cp(&w2, 10, 19, bench, 0, 1, None).is_none());
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
        assert!(cp(&w, 10, 19, chest, 0, 1, None).is_some());
        // Lava in any footprint cell is refused (2x2, origin (0,1)).
        let mut w2 = world();
        for (x, y) in [(10, 18), (11, 18), (10, 19), (11, 19)] {
            set_liquid(&mut w2, x, y, 200, true);
        }
        assert!(cp(&w2, 10, 19, chest, 0, 1, None).is_none());
        // A single lava cell in the footprint blocks the whole object —
        // `flag5 | flag3 | flag4` marks that cell occupied (TileObject.cs:360).
        let mut w3 = world();
        set_liquid(&mut w3, 11, 19, 200, true);
        assert!(cp(&w3, 10, 19, chest, 0, 1, None).is_none());
    }

    #[test]
    fn test_DW_1_2_can_place_cut_tiles_are_free() {
        let bench = def_for_tile(18).unwrap();
        // Cut tiles (grass plants, tile 82) do not block placement — `Place`
        // kills them (`TileObject.cs:84-93`, `:357`).
        let mut w = world();
        set_active(&mut w, 10, 19, 82);
        set_active(&mut w, 11, 19, 82);
        assert!(cp(&w, 10, 19, bench, 0, 1, None).is_some());
        // Plain blocks still block.
        let mut w2 = world();
        set_active(&mut w2, 10, 19, 1);
        assert!(cp(&w2, 10, 19, bench, 0, 1, None).is_none());
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
        assert!(cp(&w, 10, 19, &TEST_FULL_LIQUID, 0, 1, None).is_none());
        set_liquid(&mut w, 10, 19, 255, false);
        assert!(cp(&w, 10, 19, &TEST_FULL_LIQUID, 0, 1, None).is_some());
        // An empty cell fails OnlyInLiquid/OnlyInFullLiquid (:1889-1905).
        assert!(cp(&w, 11, 19, &TEST_FULL_LIQUID, 0, 1, None).is_none());
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
        assert!(cp(&w, 10, 19, &EMPTY_TOP, 0, 1, None).is_some());
        // A plain solid tile above satisfies ANY top-anchor type: the first
        // top branch (:453-460) has no `anchor.ty` gate — vanilla behavior.
        set_active(&mut w, 10, 18, 1);
        assert!(cp(&w, 10, 19, &EMPTY_TOP, 0, 1, None).is_some());
        // A solid-top tile (platform, kind 19) is not "solid", so the first
        // branch skips and the EmptyTile type rejects the active cell.
        set_active(&mut w, 10, 18, 19);
        assert!(cp(&w, 10, 19, &EMPTY_TOP, 0, 1, None).is_none());
        let idx_10_18 = w.index(10, 18);
        w.tiles[idx_10_18] = Tile::default();
        // SolidSide: anchor against a solid tile to the left (:646-653).
        set_active(&mut w, 9, 19, 1);
        assert!(cp(&w, 10, 19, &SOLID_SIDE, 0, 1, None).is_some());
        let idx_9_19 = w.index(9, 19);
        w.tiles[idx_9_19] = Tile::default();
        // Table anchor over a table tile (:407-410).
        set_active(&mut w, 10, 19, 14);
        assert!(cp(&w, 10, 18, &ON_TABLE, 0, 1, None).is_some());
        let idx_10_19 = w.index(10, 19);
        w.tiles[idx_10_19] = Tile::default();
        // Platform top frame (:394-401): tile 19 with a proper top frame.
        set_active(&mut w, 10, 19, 19);
        let idx_10_19 = w.index(10, 19);
        w.tiles[idx_10_19].frame_x = 0;
        assert!(cp(&w, 10, 18, &ON_PLATFORM, 0, 1, None).is_some());
        let idx_10_19 = w.index(10, 19);
        w.tiles[idx_10_19] = Tile::default();
        // AnchorInvalidTiles (`isValidTileAnchor` :1924-1933).
        set_active(&mut w, 10, 19, 1);
        assert!(cp(&w, 10, 18, &INVALID_LIST, 0, 1, None).is_none());
        // ...and an unlisted tile passes.
        set_active(&mut w, 10, 19, 2);
        assert!(cp(&w, 10, 18, &INVALID_LIST, 0, 1, None).is_some());
        let idx_10_19b = w.index(10, 19);
        w.tiles[idx_10_19b] = Tile::default();
        // Tree trunk anchor (:654-691): the trunk must continue above and
        // below the anchor run, or the extension checks miss and the anchor
        // ratio drops below 1.
        set_active(&mut w, 9, 18, 5);
        set_active(&mut w, 9, 19, 5);
        set_active(&mut w, 9, 20, 5);
        assert!(cp(&w, 10, 19, &ON_TREE, 0, 1, None).is_some());
        // A lone trunk (no continuation below) fails the ratio.
        let idx_9_20 = w.index(9, 20);
        w.tiles[idx_9_20] = Tile::default();
        assert!(cp(&w, 10, 19, &ON_TREE, 0, 1, None).is_none());
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
        let obj = cp(&w, 8, 19, bench, 3, 1, None).unwrap();
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
        let obj = cp(&w, 12, 19, table, 2, 1, None).unwrap();
        assert!(place(&mut w, &obj));
        assert_eq!(w.tile(11, 18).unwrap().frame_x, 108);
        assert_eq!(w.tile(11, 18).unwrap().frame_y, 0);
        assert_eq!(w.tile(12, 19).unwrap().frame_x, 126);
        assert_eq!(w.tile(12, 19).unwrap().frame_y, 18);

        // Closed door, style 1: not horizontal -> frameX = 18*0, frameY = 54*1.
        let door = def_for_tile(10).unwrap();
        set_active(&mut w, 20, 16, 1); // door top anchor
        let obj = cp(&w, 20, 17, door, 1, 1, None).unwrap();
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
        let obj = cp(&w3, 20, 17, door, 36, 1, None).unwrap();
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
        let obj = cp(&w, 24, 19, chest, 1, 1, None).unwrap();
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
        let obj = cp(&w, 30, 19, chair, 2, 1, None).unwrap();
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
        let obj = cp(&w2, 20, 17, open, 0, 1, None).unwrap();
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
        let obj = cp(&w4, 20, 17, open, 0, -1, None).unwrap();
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
        assert!(cp(&w, 21, 17, door, 0, 1, None).is_none());
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
            cp(&w, 20, 17, open, 0, 1, None).is_none(),
            "right leaf blocked"
        );
        assert!(
            cp(&w, 20, 17, open, 0, -1, None).is_some(),
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
            cp(&w2, 20, 17, open, 0, 1, None).is_some(),
            "right leaf free"
        );
        assert!(
            cp(&w2, 20, 17, open, 0, -1, None).is_none(),
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
        assert!(cp(&w2, 12, 19, table, 0, 1, None).is_none());
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
            cp(&w3, 10, 19, chest, 0, 1, None).is_none(),
            "no Table anchor"
        );
        set_active(&mut w3, 10, 20, 1);
        set_active(&mut w3, 11, 20, 1);
        assert!(cp(&w3, 10, 19, chest, 0, 1, None).is_some());
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
            crate::framing::reframe_area(&mut w, x, y, 2, true);
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
        // An alternate index past the list KEEPS the resolved def
        // (:5150-5159, W4-e12 — the old port answered None here).
        assert_eq!(get_tile_data(&WITH_SUBS, 0, 2).unwrap().height, 1);
        // The sub-tile carries NO alternates: an out-of-range alternate
        // keeps the SUB-TILE, not the base (:5141-5148 runs first).
        assert_eq!(get_tile_data(&WITH_SUBS, 5, 1).unwrap().height, 7);
    }

    /// W4-e12 — the alternate tail of `GetTileData(int,int,int)`
    /// (`TileObjectData.cs:5150-5159`) on the REAL registry: a non-null
    /// in-range entry replaces the def, anything else keeps it. The
    /// consumers are `TileObject.Place` (:31, arbitrary `toBePlaced.alternate`)
    /// and `DrawPreview` (:859); the `CanPlace` loop (:203-208) only passes
    /// 0..=AlternatesCount, which is why the old None-on-overflow never bit.
    #[test]
    fn test_w4_e12_alternate_overflow_keeps_the_resolved_def() {
        // Chair 15: base 1 alternate (A1, height 2), sub-tile S16 carries
        // its OWN single alternate (registry D_15_S16_A1).
        let chair = def_for_tile(15).unwrap();
        assert_eq!(chair.alternates.len(), 1);
        let s16 = chair
            .subtiles
            .iter()
            .find(|(s, _)| *s == 16)
            .unwrap()
            .1;
        assert_eq!(s16.alternates.len(), 1);
        // In range: the alternate replaces (:5154-5158).
        assert!(std::ptr::eq(
            get_tile_data(chair, 0, 1).unwrap(),
            &chair.alternates[0]
        ));
        assert!(std::ptr::eq(
            get_tile_data(chair, 16, 1).unwrap(),
            &s16.alternates[0]
        ));
        // Past the list: the resolved def stands (base / sub-tile).
        for alt in [2, 7, i32::MAX] {
            assert!(std::ptr::eq(get_tile_data(chair, 0, alt).unwrap(), chair));
            assert!(std::ptr::eq(get_tile_data(chair, 16, alt).unwrap(), s16));
        }
        // alternate 0 and negatives: `alternate--` < 0 skips the list
        // entirely (:5150-5152).
        for alt in [0, -1, -9] {
            assert!(std::ptr::eq(get_tile_data(chair, 0, alt).unwrap(), chair));
        }
        // Table 14: base alternates EMPTY, sub-tiles 13/25 carry none —
        // every alternate index keeps the resolved node.
        let table = def_for_tile(14).unwrap();
        assert!(table.alternates.is_empty());
        let s13 = table.subtiles.iter().find(|(s, _)| *s == 13).unwrap().1;
        assert!(std::ptr::eq(get_tile_data(table, 0, 3).unwrap(), table));
        assert!(std::ptr::eq(get_tile_data(table, 13, 3).unwrap(), s13));
    }

    /// W4-e12 — the consumer end of the overflow fix: `TileObject.Place`
    /// (`TileObject.cs:31-35`) resolves through `get_tile_data` with the
    /// instance's arbitrary `alternate`; an overflow keeps the def, so the
    /// placement RUNS (the old None refused it before any world write).
    #[test]
    fn test_w4_e12_place_rides_the_alternate_overflow() {
        let w = world();
        let chair = def_for_tile(15).unwrap();
        // Facing left keeps the base def (alternate 0).
        let mut obj = must_place(&w, 10, 19, chair, -1);
        assert_eq!(obj.alternate, 0);
        let mut w_ok = world();
        obj.alternate = 5; // past the 1-entry list -> the base def stands.
        assert!(place(&mut w_ok, &obj));
        // Byte-equal to the alternate-0 placement (same resolved def).
        let mut w_ref = world();
        obj.alternate = 0;
        assert!(place(&mut w_ref, &obj));
        assert_eq!(w_ok.tiles, w_ref.tiles);
    }

    // ---- W4-d113: TryGetTileBounds coordinate walk (TileObjectData.cs:5163-5194)

    /// The base-def slice of the `GetTileData(Tile)` seam (:5196-5261) for
    /// kinds WITHOUT sub-tiles/overrides: the active gate (:5198-5201) +
    /// `_data[type]` (:5207-5211). The full frame decode is W4-d112's
    /// `resolve_tile_def`; the merge wires that in as the `resolve`
    /// argument. Every walk witness below uses a kind whose resolved def
    /// has the BASE geometry (270's platform alternate D_270_A1 is
    /// field-identical in width/heights/width/padding), so this slice is
    /// exact for them even under the full resolver.

    /// Places an active tile with the given frames (the walk reads only
    /// kind/active/frameX/frameY).
    fn set_framed(w: &mut World, x: i32, y: i32, kind: u16, fx: i32, fy: i32) {
        set_active(w, x, y, kind);
        w.set_tile_frame_x(x, y, fx);
        w.set_tile_frame_y(x, y, fy);
    }

    #[test]
    fn w4_d113_walk_heavy_work_bench_283() {
        // D_283 (registry): 3x3, heights [16,16,16], width 16, pad 2 →
        // CFW = CFH = 54 (TileObjectData.cs:4062-4064, Style3x3).
        let mut w = world();
        // Top-left cell: frames (0,0) → no steps.
        set_framed(&mut w, 12, 12, 283, 0, 0);
        assert_eq!(
            try_get_tile_bounds(&w, 12, 12),
            Some((12, 12, 3, 3))
        );
        // Bottom-right cell: frames (36,36) → :5177-5181 two column steps
        // (36→20→4), :5183-5190 two row steps (36→20→4).
        set_framed(&mut w, 12, 12, 283, 36, 36);
        assert_eq!(
            try_get_tile_bounds(&w, 12, 12),
            Some((10, 10, 3, 3))
        );
        // Padding column/row (frame 18 = 16+2): the C# while/for still step
        // once — the remainder 18 >= 16 (:5177/:5185), padding is NOT skipped.
        set_framed(&mut w, 12, 12, 283, 18, 18);
        assert_eq!(
            try_get_tile_bounds(&w, 12, 12),
            Some((11, 11, 3, 3))
        );
        // A second style column (frameX 54 = one CFW): the remainder resets,
        // the walk is style-independent (:5172-5175 divides first).
        set_framed(&mut w, 12, 12, 283, 54, 0);
        assert_eq!(
            try_get_tile_bounds(&w, 12, 12),
            Some((12, 12, 3, 3))
        );
    }

    #[test]
    fn w4_d113_walk_bunny_cage_275_six_wide() {
        // D_275 (registry, TileObjectData.cs:2696-2708 Style6x3): 6x3,
        // CFW = (16+2)*6 = 108, CFH = 54.
        let mut w = world();
        // Cell (20,12), frames (90,36): 90 = 5*16+10 → five column steps
        // (:5177-5181), 36 → two row steps → top-left (15,10).
        set_framed(&mut w, 20, 12, 275, 90, 36);
        assert_eq!(
            try_get_tile_bounds(&w, 20, 12),
            Some((15, 10, 6, 3))
        );
    }

    #[test]
    fn w4_d113_walk_chest_21_uneven_heights() {
        // D_21 (registry, TileObjectData.cs:3461-3470): 2x2, heights [16,18],
        // pad 2 → CFH = 16+2+18+2 = 38, CFW = 36. The row loop reads EVERY
        // height (:5183-5190), not just the first.
        let mut w = world();
        // Bottom-row cell: frameY 20 = row 1 start (16+2) + 2 → subtract 16
        // (row 0), remainder 4 < 18 (row 1) → one row step.
        set_framed(&mut w, 11, 11, 21, 18, 20);
        assert_eq!(
            try_get_tile_bounds(&w, 11, 11),
            Some((10, 10, 2, 2))
        );
        // C# quirk pinned: frameY 36 is row 1's PADDING (row 1 spans 18..36);
        // the full-array loop subtracts 16 then 18 → TWO row steps, landing
        // one tile above the true top. Faithful, not "fixed" (:5183-5190).
        set_framed(&mut w, 11, 11, 21, 18, 36);
        assert_eq!(
            try_get_tile_bounds(&w, 11, 11),
            Some((10, 9, 2, 2))
        );
    }

    #[test]
    fn w4_d113_walk_bottle_270_and_gates() {
        // D_270 (registry, TileObjectData.cs:2442-2448 Style1x2Top): 1x2,
        // heights [16,16] → CFH 36.
        let mut w = world();
        set_framed(&mut w, 10, 11, 270, 0, 18);
        assert_eq!(
            try_get_tile_bounds(&w, 10, 11),
            Some((10, 10, 1, 2))
        );
        // Inactive tile: the seam's :5198-5201 gate → None.
        set_framed(&mut w, 10, 11, 270, 0, 18);
        w.tiles[w.index(10, 11)].flags &= !FLAG_ACTIVE;
        assert_eq!(try_get_tile_bounds(&w, 10, 11), None);
        // Unregistered kind (stone 1): `_data[type]` null (:5208-5211).
        set_framed(&mut w, 10, 11, 1, 0, 0);
        assert_eq!(try_get_tile_bounds(&w, 10, 11), None);
        // Out of bounds: the port's world.tile None (C# throws; documented).
        assert_eq!(try_get_tile_bounds(&w, -1, 0), None);
        // Negative frames: C# int division truncates toward zero — num 0,
        // remainder -1 < 16 → no steps (:5172-5175, both languages agree).
        set_framed(&mut w, 10, 11, 270, -1, -1);
        assert_eq!(
            try_get_tile_bounds(&w, 10, 11),
            Some((10, 11, 1, 2))
        );
    }

    #[test]
    fn w4_d113_walk_uses_the_resolved_def() {
        // The walk reads the RESOLVED def's geometry (:5167 → :5172-5192),
        // not the base def's. Synthetic resolver returning a 2x2 def with
        // heights [10,20], width 8, pad 0 → CFW 16, CFH 30.
        static RESOLVED: TileObjectDef = TileObjectDef {
            width: 2,
            height: 2,
            coordinate_heights: &[10, 20],
            coordinate_width: 8,
            ..base_def()
        };
        fn seam(_: &Tile) -> Option<&'static TileObjectDef> {
            Some(&RESOLVED)
        }
        let mut w = world();
        // Frames (8,20): one column step (8 >= 8), row loop 20 >= 10 → one
        // step, remainder 10 < 20 → stop. Bounds carry RESOLVED's 2x2.
        set_framed(&mut w, 10, 10, 283, 8, 20);
        assert_eq!(try_get_tile_bounds(&w, 10, 10, seam), Some((9, 9, 2, 2)));
        // A None resolution short-circuits before any frame math (:5168-5171).
        fn none_seam(_: &Tile) -> Option<&'static TileObjectDef> {
            None
        }
        assert_eq!(try_get_tile_bounds(&w, 10, 10, none_seam), None);
    }

    // ---- W4-e11: the tile-based GetTileData(Tile) resolution (:5196-5261) ----

    /// An active tile of `kind` with the given frames (the resolution reads
    /// only `active`/`kind`/`frame_x`/`frame_y`).
    fn framed_tile(kind: u16, frame_x: i16, frame_y: i16) -> Tile {
        Tile {
            kind,
            frame_x,
            frame_y,
            flags: FLAG_ACTIVE,
            ..Default::default()
        }
    }

    #[test]
    fn test_w4_e11_horizontal_line_skip_redecode() {
        // No generated def combines StyleHorizontal with StyleLineSkip > 1
        // (the 27 skip defs are all vertical doors/plates), so the :5230-5234
        // horizontal arm is pinned synthetically: wrap 3, skip 2, 18-px grid.
        static SUB4: TileObjectDef = TileObjectDef {
            height: 7,
            ..base_def()
        };
        static HSKIP: TileObjectDef = TileObjectDef {
            coordinate_width: 16,
            coordinate_padding: 2,
            style_horizontal: true,
            style_wrap_limit: 3,
            style_line_skip: 2,
            subtiles: &[(4u16, &SUB4)],
            ..base_def()
        };
        // fx 18 -> num 1; fy 54 -> num2 3. The pre-skip decode would be
        // num4 = 3*3+1 = 10 (style 10); the skip re-decode (:5232-5233) is
        // style = 3/2*3 + 1 = 4, remainder = 3%2 = 1 -> SubTiles[4].
        let t = framed_tile(0, 18, 54);
        let r = resolve_from_frames(&HSKIP, &t).unwrap();
        assert!(std::ptr::eq(r, &SUB4));
        // fy 36 -> num2 2: style = 2/2*3+1 = 4 again, remainder 0.
        let t = framed_tile(0, 18, 36);
        assert!(std::ptr::eq(resolve_from_frames(&HSKIP, &t).unwrap(), &SUB4));
        // fy 18 -> num2 1: style = 1/2*3+1 = 1 (no sub), remainder 1.
        let t = framed_tile(0, 18, 18);
        assert!(std::ptr::eq(resolve_from_frames(&HSKIP, &t).unwrap(), &HSKIP));
    }

    #[test]
    fn test_w4_e11_null_subtile_slot_keeps_the_def() {
        // The C# SubTiles list is null-padded (:5241-5247 keeps the current
        // def on a null slot); the port's (style, def) pairs model that.
        static SUB5: TileObjectDef = TileObjectDef {
            height: 7,
            ..base_def()
        };
        static SPARSE: TileObjectDef = TileObjectDef {
            coordinate_width: 16,
            coordinate_padding: 2,
            style_horizontal: true,
            subtiles: &[(5u16, &SUB5)],
            ..base_def()
        };
        // fx 36 -> num4 = 2 (wrap 0 -> 1): a null slot -> the base def.
        let t = framed_tile(0, 36, 0);
        assert!(std::ptr::eq(resolve_from_frames(&SPARSE, &t).unwrap(), &SPARSE));
        // fx 90 -> num4 = 5 -> the sub-tile.
        let t = framed_tile(0, 90, 0);
        assert!(std::ptr::eq(resolve_from_frames(&SPARSE, &t).unwrap(), &SUB5));
    }

    #[test]
    fn test_w4_e11_alternate_band_is_inclusive_both_ends() {
        // :5254 — `num6 >= Style && num6 <= Style + RandomStyleRange`.
        static ALT: TileObjectDef = TileObjectDef {
            style: 1,
            random_style_range: 1,
            height: 9,
            ..base_def()
        };
        static BANDED: TileObjectDef = TileObjectDef {
            coordinate_width: 16,
            coordinate_padding: 2,
            style_horizontal: true,
            style_multiplier: 3,
            alternates: &[ALT],
            ..base_def()
        };
        // remainder 0 -> below the band -> base; 1 and 2 -> the alternate.
        for (fx, want_alt) in [(0i16, false), (18, true), (36, true), (54, false)] {
            let t = framed_tile(0, fx, 0);
            let r = resolve_from_frames(&BANDED, &t).unwrap();
            assert_eq!(std::ptr::eq(r, &ALT), want_alt, "fx {fx}");
        }
    }

    #[test]
    fn test_w4_e11_inactive_and_unregistered_answer_none() {
        // :5198-5201 inactive -> null; :5207-5211 unregistered -> null.
        let mut t = framed_tile(165, 0, 90);
        t.flags &= !FLAG_ACTIVE;
        assert!(get_tile_data_for_tile(&t).is_none());
        assert!(get_tile_data_for_tile(&framed_tile(1, 0, 0)).is_none());
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
                callback: "",
                bad_return: -1,
                bad_response: 0,
                processed_coordinates: false,
            },
            ..base_def()
        };
        // Everything free, but the hook returns badReturn with badResponse 0 ->
        // the scores zero out and the def is refused (:716-729).
        let w = world();
        assert!(cp(&w, 10, 19, &HOOKED, 0, 1, None).is_none());
    }

    #[test]
    fn test_place_hook_override_branch() {
        static OVERRIDDEN: TileObjectDef = TileObjectDef {
            width: 2,
            height: 1,
            hook_place_override: PlacementHook {
                hook: Some(hook_accept),
                callback: "",
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
                callback: "",
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
        // (`TileObject.cs:761-793`; the Main.rand path draws only for
        // random-style defs — wave B).
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
        let obj = cp(&w, 10, 19, &RANDOM, 0, 1, Some(7)).unwrap();
        assert_eq!(obj.random, 3);
        let obj = cp(&w, 10, 19, &RANDOM, 0, 1, Some(-1)).unwrap();
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
        let obj = cp(&w, 10, 19, &SPECIFIC, 2, 1, Some(1)).unwrap();
        assert_eq!(obj.random, 7); // 9 - 2
    }

    // ---- S1-F-7 wave B: the RandomStyleRange Main.rand runtime hook --------

    /// The sunflower fixture: grass floor (tile 2), the def's first
    /// `AnchorValidTiles` entry (`TileObjectData.cs:4859`).
    fn sunflower_world() -> World {
        let mut w = world();
        floor_of(&mut w, 2);
        w
    }

    #[test]
    fn s1_f_7b_random_style_range_rng_order() {
        // `Main.rand.Next(tileData.RandomStyleRange)` (TileObject.cs:787) is
        // ONE draw per CanPlace, spent in call order on the caller's stream.
        // Proof: a witness `UnifiedRandom` replays `next_max(3)` per call and
        // the FULL stream state (seed array + cursor, PartialEq) must match
        // after every placement — no extra, missing or reordered draws.
        let sunflower = def_for_tile(27).unwrap();
        assert_eq!(sunflower.random_style_range, 3); // :4861
        for seed in [0i32, 1, 7, 42, 12345, -99] {
            let mut w = sunflower_world();
            let mut rand = crate::unified_random::UnifiedRandom::new(seed);
            let mut witness = crate::unified_random::UnifiedRandom::new(seed);
            for x in [10, 14, 18, 22, 26] {
                let expected = witness.next_max(3); // :787
                let obj = can_place(&w, x, 19, sunflower, 0, 1, None, &mut rand, None)
                    .unwrap_or_else(|| panic!("seed {seed} x {x}"));
                assert_eq!(obj.random, expected, "seed {seed} x {x}");
                assert!((0..3).contains(&obj.random));
                assert_eq!(rand, witness, "stream state after seed {seed} x {x}");
                // The pick rides `CalculatePlacementStyle` (:60, :1990-1998):
                // frameX = CFW(36) * (0*1 + 0 + random) + column*18.
                assert!(place(&mut w, &obj));
                assert_eq!(
                    w.tile(x, 19).unwrap().frame_x,
                    (36 * expected) as i16,
                    "seed {seed} x {x}"
                );
            }
        }
    }

    #[test]
    fn s1_f_7b_random_cache_stickiness_still_draws() {
        // :764-792 — a matching cache (same type, same object start) sticks:
        // `num37 = randomCache.Random` — but `Main.rand.Next` at :787 is
        // evaluated FIRST, so the draw is spent even on a cache hit.
        let sunflower = def_for_tile(27).unwrap();
        let w = sunflower_world();
        let mut rand = crate::unified_random::UnifiedRandom::new(7);
        let mut witness = crate::unified_random::UnifiedRandom::new(7);
        // The object start of (10, 19) is (10 - 0, 19 - 3) = (10, 16)
        // (:775-776, Origin (0,3) :4853).
        let cache = RandomCache {
            kind: 27,
            coordinates: Point16::new(10, 16),
            object_start: Point16::ZERO,
            random: 1,
        };
        let _ = witness.next_max(3); // spent even though the cache wins (:787)
        let obj = can_place(&w, 10, 19, sunflower, 0, 1, None, &mut rand, Some(cache)).unwrap();
        assert_eq!(obj.random, 1, "the cached pick sticks (:792)");
        assert_eq!(rand, witness, "the :787 draw is spent on a cache hit");
        // A moved object start unsticks (:777-780): the fresh draw wins.
        let expected = witness.next_max(3);
        let obj = can_place(&w, 14, 19, sunflower, 0, 1, None, &mut rand, Some(cache)).unwrap();
        assert_eq!(obj.random, expected);
        assert_eq!(rand, witness);
        // A different type unsticks (:782-785).
        let other_type = RandomCache { kind: 20, ..cache };
        let expected = witness.next_max(3);
        let obj =
            can_place(&w, 10, 19, sunflower, 0, 1, None, &mut rand, Some(other_type)).unwrap();
        assert_eq!(obj.random, expected);
        assert_eq!(rand, witness);
        // forcedRandom beats the cache (:792 `!forcedRandom.HasValue`) and
        // still spends the draw (:787 before the :788-791 overwrite).
        let expected_state = {
            let mut w2 = witness.clone();
            w2.next_max(3);
            w2
        };
        let obj =
            can_place(&w, 10, 19, sunflower, 0, 1, Some(5), &mut rand, Some(cache)).unwrap();
        assert_eq!(obj.random, 2, "5 % 3 (:790)");
        assert_eq!(rand, expected_state, "forcedRandom spends the :787 draw");
    }

    #[test]
    fn s1_f_7b_specific_random_styles_rand_path_and_draw_order() {
        // :794-825 — the SpecificRandomStyles rand path (null in every
        // generated def; synthetic here). With BOTH fields set the C# draws
        // twice, range first (:787) then list index (:819), and the list
        // result overwrites num37 (:824).
        static BOTH: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
            random_style_range: 4,
            specific_random_styles: &[2, 9],
            ..base_def()
        };
        let w = world();
        let mut rand = crate::unified_random::UnifiedRandom::new(11);
        let mut witness = crate::unified_random::UnifiedRandom::new(11);
        let _range_draw = witness.next_max(4); // :787 — spent, then overwritten
        let idx = witness.next_max(2); // :819
        let obj = can_place(&w, 10, 19, &BOTH, 2, 1, None, &mut rand, None).unwrap();
        assert_eq!(obj.random, [2, 9][idx as usize] - 2, ":824");
        assert_eq!(rand, witness, "two draws, range before list");
        // The list-only def draws once (:819), no range draw.
        static SPECIFIC: TileObjectDef = TileObjectDef {
            width: 1,
            height: 1,
            anchor_bottom: AnchorData::new(anchor_type::SOLID_TILE, 1, 0),
            specific_random_styles: &[2, 9],
            ..base_def()
        };
        let idx = witness.next_max(2);
        let obj = can_place(&w, 10, 19, &SPECIFIC, 2, 1, None, &mut rand, None).unwrap();
        assert_eq!(obj.random, [2, 9][idx as usize] - 2);
        assert_eq!(rand, witness);
        // A matching cache sticks for the list branch too (:824), both draws
        // still spent (:787 + :819).
        let cache = RandomCache {
            kind: 0,
            coordinates: Point16::new(10, 19),
            object_start: Point16::ZERO,
            random: 5,
        };
        let mut after = witness.clone();
        after.next_max(4);
        after.next_max(2);
        let obj = can_place(&w, 10, 19, &BOTH, 2, 1, None, &mut rand, Some(cache)).unwrap();
        assert_eq!(obj.random, 5);
        assert_eq!(rand, after);
    }

    #[test]
    fn s1_f_7b_place_item_tile_rides_the_world_place_rng() {
        // The registry consumer (`edit::place_item_tile`, Player.cs:39773)
        // passes `World::place_rng` as Main.rand: one `next_max(3)` per
        // sunflower placement, deterministic per world seed, and the
        // non-random defs draw nothing.
        let sunflower_kind = 27u16;
        let mut a = sunflower_world();
        a.seed = 777;
        a.seed_frame_rng();
        let mut b = sunflower_world();
        b.seed = 777;
        b.seed_frame_rng();
        let mut witness = crate::unified_random::UnifiedRandom::new(777);
        for x in [10, 14, 18] {
            let expected = witness.next_max(3);
            assert!(crate::edit::place_item_tile(&mut a, x, 19, sunflower_kind, 0, 1));
            assert!(crate::edit::place_item_tile(&mut b, x, 19, sunflower_kind, 0, 1));
            assert_eq!(a.tile(x, 19).unwrap().frame_x, (36 * expected) as i16);
            assert_eq!(a.tiles, b.tiles, "same seed, same placements");
            assert_eq!(a.place_rng, witness, "one draw per placement, in order");
        }
        // The production randomCache stays None (the onlyCheck writeback is
        // not ported — TileObject.cs:826-837), so every placement draws fresh.
        assert_eq!(a.tile_object_random_cache, None);
        // A non-random def (bench 18) spends no draw.
        let before = a.place_rng.clone();
        assert!(crate::edit::place_item_tile(&mut a, 24, 19, 18, 0, 1));
        assert_eq!(a.place_rng, before, "no RandomStyleRange, no draw");
        // The framing stream is untouched by placement draws.
        let mut c = sunflower_world();
        c.seed = 777;
        c.seed_frame_rng();
        let frame_before = c.frame_rng.clone();
        assert!(crate::edit::place_item_tile(&mut c, 10, 19, sunflower_kind, 0, 1));
        assert_eq!(c.frame_rng, frame_before, "place_rng is a separate stream");
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
        // 83/84 are `FullCopyFrom(82)`/`FullCopyFrom(83)` chains (:4785-4788).
        // S1-F-7 divergence (C# wins): `FullCopyFrom`'s sub-tile copy ctor loops
        // `i < this.data.Count` over the FRESH list (Count 0 —
        // TileObjectSubTilesModule.cs:17-27), so the copy's sub-tile list is
        // EMPTY in vanilla. The hand defs wrongly carried 82's sub-tiles along;
        // the generated table reproduces the C# bug: 83/84 resolve no sub-tiles,
        // so style 5 stays the base def (LavaPlacement NotAllowed).
        for kind in [83u16, 84] {
            let def = def_for_tile(kind).unwrap();
            assert!(def.subtiles.is_empty(), "FullCopyFrom empties the sub-tiles (C# bug)");
            assert_eq!(get_tile_data(def, 5, 0).unwrap().lava_placement, LiquidPlacement::NotAllowed);
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
        // forcedRandom = 0 pins the random style slot to 0 (wave B: the
        // Main.rand path now draws; :788-791 folds the forced value).
        let obj = cp(&w, 10, 19, sunflower, 0, 1, Some(0)).unwrap();
        assert_eq!((obj.x_coord, obj.y_coord), (10, 16));
        assert_eq!(obj.random, 0);
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
                cp(&w, 10, 19, def_for_tile(27).unwrap(), 0, 1, None).is_some(),
                "soil {soil} must support the sunflower"
            );
        }
        // A stone floor is solid but unlisted — refused.
        let w = world();
        assert!(
            cp(&w, 10, 19, def_for_tile(27).unwrap(), 0, 1, None).is_none(),
            "stone is not in the sunflower's anchor set"
        );
        // RandomStyleRange resolves through the random slot
        // (`CalculatePlacementStyle` :1990-1998): placement = 0*1 + 0 + random.
        let mut w2 = world();
        floor_of(&mut w2, 2);
        let sunflower = def_for_tile(27).unwrap();
        let obj = cp(&w2, 10, 19, sunflower, 0, 1, Some(2)).unwrap();
        assert_eq!(obj.random, 2);
        assert!(place(&mut w2, &obj));
        // frameX = CFW(36) * placement(2) + column(18) = 90 on the right cell.
        assert_eq!(w2.tile(10, 19).unwrap().frame_x, 72);
        assert_eq!(w2.tile(11, 19).unwrap().frame_x, 90);
        // forced_random folds into the range (:788-791); a clear spot for the
        // next resolution.
        let obj = cp(&w2, 20, 19, sunflower, 0, 1, Some(5)).unwrap();
        assert_eq!(obj.random, 2);
    }

    #[test]
    fn test_DW_1_2_pots_natural_anchor_set() {
        // The pot anchor (:3556) keeps SolidTile/SolidWithTop/SolidSide and
        // drops the Style2x2 preset's Table.
        let pots = def_for_tile(28).unwrap();
        let mut w = world(); // stone floor: solid, flat
        assert!(cp(&w, 10, 19, pots, 0, 1, None).is_some());
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
        assert!(cp(&wt, 10, 19, pots, 0, 1, None).is_none());
        assert!(w.frame_important[28]);
    }

    #[test]
    fn test_DW_1_2_clay_pot_table_and_solid_anchors() {
        let pot = def_for_tile(78).unwrap();
        // On a table (:2385 keeps AnchorType.Table)…
        let mut w = world();
        set_active(&mut w, 10, 19, 14);
        assert!(cp(&w, 10, 18, pot, 0, 1, None).is_some());
        // …on plain solid ground…
        let w2 = world();
        assert!(cp(&w2, 10, 19, pot, 0, 1, None).is_some());
        // …and in lava it is refused (LavaDeath=false never re-allows lava —
        // the baked NotAllowed, same chain as the chest).
        let mut wl = world();
        set_liquid(&mut wl, 10, 19, 200, true);
        assert!(cp(&wl, 10, 19, pot, 0, 1, None).is_none());
        // Water is fine (base WaterPlacement.Allowed).
        let mut ww = world();
        set_liquid(&mut ww, 10, 19, 200, false);
        assert!(cp(&ww, 10, 19, pot, 0, 1, None).is_some());
    }

    #[test]
    fn test_DW_1_2_pile_subtile_styles_and_anchors() {
        // 186 snow style 26 sits on snow (:3293) and refuses stone…
        let large = def_for_tile(186).unwrap();
        let mut snow = world();
        floor_of(&mut snow, 147);
        assert!(cp(&snow, 10, 19, large, 26, 1, None).is_some());
        let stone = world();
        assert!(cp(&stone, 10, 19, large, 26, 1, None).is_none());
        // …while the base styles (0-25, no valid-tile list) take any solid
        // floor.
        assert!(cp(&stone, 10, 19, large, 0, 1, None).is_some());
        // 187 sand style 29 on sand (:3310-3311), refuses stone; style 0 is
        // the jungle sub-tile (SubTiles[0] non-null) — refuses stone too.
        let large2 = def_for_tile(187).unwrap();
        let mut sand = world();
        floor_of(&mut sand, 53);
        assert!(cp(&sand, 10, 19, large2, 29, 1, None).is_some());
        assert!(cp(&sand, 10, 19, large2, 0, 1, None).is_none());
        assert!(cp(&stone, 10, 19, large2, 29, 1, None).is_none());
        // Place a 3x2: origin (1,1), click (11, 19) → origin (10, 18). The
        // generic Place math puts style 29 on sheet row 29: wrap 35 keeps the
        // column (`num4 = 29 % 35`), so frameX = CFW(54) * 29 + dx*18,
        // frameY = dy*18 (`TileObject.cs:56-110`).
        let obj = cp(&sand, 11, 19, large2, 29, 1, None).unwrap();
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
        assert!(cp(&w, 10, 19, piles, 0, 1, None).is_some());
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
        // S1-F-7: the full registry registers the Solidifier 220 too
        // (`TileObjectData.cs:4048`, Style3x3 + DrawYOffset 2) — the hand slice
        // had only the two extractinators. A genuinely unregistered neighbour
        // (Palladium ore 221) stays absent.
        assert!(def_for_tile(220).is_some());
        assert!(def_for_tile(221).is_none());
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
        let inst = cp(&w, 10, 19, def, 0, 1, None).expect("3x3 headroom");
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
        assert!(cp(&w, 10, 5, def, 0, 1, None).is_none());
        // Placed on the floor, the kill cascade removes all nine cells
        // (`KillTile`'s registry cascade, `edit::kill_tile`).
        let inst = cp(&w, 10, 19, def, 0, 1, None).unwrap();
        assert!(place(&mut w, &inst));
        assert!(crate::edit::kill_tile(&mut w, 11, 18, false).is_some());
        for sub_y in 0..3 {
            for sub_x in 0..3 {
                assert!(!w.tile(9 + sub_x, 17 + sub_y).unwrap().active());
            }
        }
    }

    // ---- S1-F-7 wave A: per-family spot tests of the GENERATED registry ------
    //
    // Each family pins the static fields straight from the C# builder chain
    // (the generator interprets `Initialize` 1:1; these are the regression
    // anchors for the families the task names). The full-table snapshot is the
    // `tileobject_registry_freeze` integration test; the regen-idempotence is
    // `tools/GenTileObjectData/test_gen.py`.

    /// Platforms (tile 19 + the 427/435..439 variants): the `Initialize` prelude
    /// (`TileObjectData.cs:2127-2160`) — 1x1, 16-px cell, horizontal styles,
    /// multiplier/wrap 27, `LavaDeath=true` folded to `LavaPlacement NotAllowed`,
    /// `UsesCustomCanPlace=false` (the legacy `WorldGen.PlaceTile` case-19 route).
    #[test]
    fn s1_f_7_platforms_match_the_csharp_prelude() {
        for kind in [19u16, 427, 435, 436, 437, 438, 439] {
            let def = def_for_tile(kind).unwrap_or_else(|| panic!("platform {kind} registered"));
            assert_eq!((def.width, def.height), (1, 1), "kind {kind}");
            assert_eq!(def.origin, Point16::ZERO, "kind {kind}");
            assert_eq!(def.coordinate_width, 16, "kind {kind}");
            assert_eq!(def.coordinate_padding, 2, "kind {kind}");
            assert_eq!(def.coordinate_heights, &[16], "kind {kind}");
            assert!(def.style_horizontal, "kind {kind}");
            assert_eq!(def.style_multiplier, 27, "kind {kind}"); // :2131/:2144/:2155
            assert_eq!(def.style_wrap_limit, 27, "kind {kind}"); // :2132/:2145/:2156
            assert_eq!(def.lava_placement, LiquidPlacement::NotAllowed, "kind {kind}"); // LavaDeath fold
            assert!(!def.uses_custom_can_place, "kind {kind}"); // :2133/:2146/:2157
            assert!(def.frame_important, "kind {kind}");
            // `PlatformFrameWidth()` (:5121-5124) = `_data[19].CoordinateFullWidth`.
            assert_eq!(def.coordinate_full_width(), 18, "kind {kind}");
        }
        // The platform sub-tiles 13/43/47 (:2138) allow lava (the prelude's
        // newSubTile LavaDeath=false + LavaPlacement=Allowed).
        let platform = def_for_tile(19).unwrap();
        for style in [13u16, 43, 47] {
            let sub = get_tile_data(platform, style as i32, 0).unwrap();
            assert_eq!(sub.lava_placement, LiquidPlacement::Allowed, "sub {style}");
        }
    }

    /// Torches (tile 4): `StyleTorch` (`TileObjectData.cs:4577-4628`) — 1x1,
    /// 20-px cell, multiplier/wrap 6, `FlattenAnchors`, water/lava death, and the
    /// three wall/side alternates (left/right/tree-anchor + wall).
    #[test]
    fn s1_f_7_torches_match_the_csharp_style_torch() {
        let torch = def_for_tile(4).unwrap();
        assert_eq!((torch.width, torch.height), (1, 1)); // :4577-4578
        assert_eq!(torch.coordinate_width, 20); // :4583
        assert_eq!(torch.coordinate_padding, 2); // :4584
        assert_eq!(torch.coordinate_heights, &[20]); // :4581
        assert!(torch.flatten_anchors); // :4580
        assert_eq!(torch.style_multiplier, 6); // :4585
        assert_eq!(torch.style_wrap_limit, 6); // :4586
        assert!(torch.style_horizontal); // :4587
        assert_eq!(torch.water_placement, LiquidPlacement::NotAllowed); // :4588 + WaterDeath fold
        assert_eq!(torch.lava_placement, LiquidPlacement::NotAllowed); // :4589
        assert!(torch.water_death); // :4590
        assert!(torch.lava_death); // :4591
        assert!(torch.uses_custom_can_place); // :4592
        // The three alternates (:4596-4606): left-anchor (style 1), right-anchor
        // (style 2), wall-anchor (style 0). The side anchors carry the tree/
        // alternate-tile set {124, 561, 574..578}.
        assert_eq!(torch.alternates.len(), 3);
        let left = &torch.alternates[0];
        assert_eq!(left.style, 1);
        assert_eq!(left.anchor_left.ty, 1 | 8 | 0x10 | 0x20); // SolidTile|SolidSide|Tree|AlternateTile
        assert_eq!(left.anchor_alternate_tiles, &[124, 561, 574, 575, 576, 577, 578]);
        let right = &torch.alternates[1];
        assert_eq!(right.style, 2);
        assert_eq!(right.anchor_right.ty, 1 | 8 | 0x10 | 0x20);
        let wall = &torch.alternates[2];
        assert_eq!(wall.style, 0);
        assert!(wall.anchor_wall);
        // The torch sub-tiles 8/11/17 (:4607-4627) allow water AND lava.
        for style in [8u16, 11, 17] {
            let sub = get_tile_data(torch, style as i32, 0).unwrap();
            assert_eq!(sub.water_placement, LiquidPlacement::Allowed, "sub {style}");
            assert_eq!(sub.lava_placement, LiquidPlacement::Allowed, "sub {style}");
        }
    }

    /// Doors (tiles 10/11): the closed door's three click-point alternates and
    /// the open door's five (right-open ×2 + left-open ×3), style wrap 36.
    #[test]
    fn s1_f_7_doors_match_the_csharp_alternation() {
        let closed = def_for_tile(10).unwrap();
        assert_eq!((closed.width, closed.height), (1, 3)); // :2223-2224
        assert_eq!(closed.style_wrap_limit, 36); // :2234
        assert_eq!(closed.style_line_skip, 3); // :2235
        assert!(!closed.style_horizontal); // :2233
        assert_eq!(closed.alternates.len(), 2); // :2236-2241
        assert_eq!(closed.alternates[0].origin, Point16::new(0, 1));
        assert_eq!(closed.alternates[1].origin, Point16::new(0, 2));
        // The closed-door sub-tiles 19/48 (:2242-2246) allow lava.
        for style in [19u16, 48] {
            let sub = get_tile_data(closed, style as i32, 0).unwrap();
            assert_eq!(sub.lava_placement, LiquidPlacement::Allowed, "sub {style}");
        }

        let open = def_for_tile(11).unwrap();
        assert_eq!((open.width, open.height), (2, 3)); // :2248-2249
        assert_eq!(open.style_line_skip, 2); // :2260
        assert_eq!(open.direction, TileObjectDirection::PlaceRight); // :2261
        assert_eq!(open.alternates.len(), 5); // :2262-2285
        // Right-open click points (style 0), then left-open (style 1, PlaceLeft,
        // hinge-column anchors checkStart 1).
        for (i, alt) in open.alternates.iter().enumerate() {
            let (style, dir) = if i < 2 { (0, TileObjectDirection::PlaceRight) }
                else { (1, TileObjectDirection::PlaceLeft) };
            assert_eq!(alt.style, style, "alt {i}");
            assert_eq!(alt.direction, dir, "alt {i}");
        }
        assert_eq!(open.alternates[2].origin, Point16::new(1, 0)); // :2269
        assert_eq!(open.alternates[2].anchor_top.check_start, 1); // :2270
    }

    /// Chairs (tile 15): `Style1x2` + the chair overrides
    /// (`TileObjectData.cs:4141-4173`) — wrap/multiplier 2, padding fix (0,2),
    /// the PlaceRight alternate, and the sub-tile families 16/47 + 51..67.
    #[test]
    fn s1_f_7_chairs_match_the_csharp_style_1x2() {
        let chair = def_for_tile(15).unwrap();
        assert_eq!((chair.width, chair.height), (1, 2)); // Style1x2 :4131-4132
        assert_eq!(chair.origin, Point16::new(0, 1)); // :4133
        assert_eq!(chair.coordinate_heights, &[16, 18]); // :4142
        assert_eq!(chair.coordinate_padding_fix, Point16::new(0, 2)); // :4146
        assert_eq!(chair.style_wrap_limit, 2); // :4144
        assert_eq!(chair.style_multiplier, 2); // :4145
        assert_eq!(chair.direction, TileObjectDirection::PlaceLeft); // :4143
        assert_eq!(chair.alternates.len(), 1); // :4148-4150
        assert_eq!(chair.alternates[0].direction, TileObjectDirection::PlaceRight);
        assert_eq!(chair.alternates[0].style, 1);
        // The sub-tiles: 16/47 (the lava-allowed variants, :4151-4155) and
        // 51..67 (the toilet family, :4156-4172).
        let sub_styles: Vec<u16> = chair.subtiles.iter().map(|(s, _)| *s).collect();
        assert!(sub_styles.contains(&16) && sub_styles.contains(&47));
        for s in 51..=67u16 {
            assert!(sub_styles.contains(&s), "toilet sub-tile {s}");
        }
        assert_eq!(get_tile_data(chair, 16, 0).unwrap().lava_placement, LiquidPlacement::Allowed);
    }

    /// S1-F-7a regression: sub-tiles carry their OWN alternates lists —
    /// `GetTileData` does `alternate--` then indexes the RESOLVED sub-tile's
    /// Alternates (TileObjectData.cs:5148-5157). Dropping them refused the
    /// Obsidian Chair (item 1459, tile 15 style 16) right-facing placement
    /// and lost the torch wall-anchor alternate.
    #[test]
    fn s1_f_7a_sub_tile_alternates_resolve() {
        // Chair style 16 (Obsidian, :4151-4155): alternate 1 is the
        // PlaceRight variant (style 1) — the CanPlace direction filter's
        // only right-facing candidate for that style.
        let chair = def_for_tile(15).unwrap();
        let right = get_tile_data(chair, 16, 1).unwrap();
        assert_eq!(right.direction, TileObjectDirection::PlaceRight);
        assert_eq!(right.style, 1);
        // Torch sub-tile 8 (:4607-4627, CopyFrom of the torch base):
        // alternate 3 is the wall-anchor variant (:4596-4606).
        let torch = def_for_tile(4).unwrap();
        let wall = get_tile_data(torch, 8, 3).unwrap();
        assert!(wall.anchor_wall);
    }

    /// Chests (tile 21): `Style2x2` + the chest overrides
    /// (`TileObjectData.cs:3461-3470`) — 18-px bottom row, horizontal styles,
    /// the 11-id invalid-anchor list, the bottom anchor WITHOUT Table, and the
    /// `Chest.FindEmptyChest`/`AfterPlacement_Hook` callback seams.
    #[test]
    fn s1_f_7_chests_match_the_csharp_style_2x2() {
        let chest = def_for_tile(21).unwrap();
        assert_eq!((chest.width, chest.height), (2, 2)); // Style2x2 :3451-3452
        assert_eq!(chest.origin, Point16::new(0, 1)); // :3462
        assert_eq!(chest.coordinate_heights, &[16, 18]); // :3463
        assert!(chest.style_horizontal); // :3467
        // :3469 — the bottom anchor drops the preset's AnchorType.Table.
        assert_eq!(chest.anchor_bottom.ty, 1 | 2 | 8); // SolidTile|SolidWithTop|SolidSide
        assert_eq!(chest.anchor_bottom.tile_count, 2);
        // :3466 — the shared 11-id invalid list (:2112-2116).
        assert_eq!(chest.anchor_invalid_tiles, &[127, 138, 664, 665, 484, 711, 712, 713, 714, 715, 716]);
        // :3468 sets only LavaDeath=false — the baked NotAllowed stands.
        assert_eq!(chest.lava_placement, LiquidPlacement::NotAllowed);
        assert!(!chest.lava_death);
        // The hook callback seams (:3464-3465): the names are generated data,
        // the bodies stay runtime callbacks (wave A).
        assert_eq!(chest.hook_check_if_can_place.callback, "Chest.FindEmptyChest");
        assert_eq!(chest.hook_check_if_can_place.bad_return, -1);
        assert!(chest.hook_check_if_can_place.processed_coordinates);
        assert_eq!(chest.hook_post_place_my_player.callback, "Chest.AfterPlacement_Hook");
        assert!(chest.hook_check_if_can_place.hook.is_none(), "the body is a wave-B callback");
    }
}
