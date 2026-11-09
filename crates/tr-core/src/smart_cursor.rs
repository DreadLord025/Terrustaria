//! The smart-cursor lookup — `SmartCursorHelper.SmartCursorLookup`
//! (`GameContent/SmartCursorHelper.cs:61-165`) with the ported steps.
//!
//! The vanilla scans the reachable rect for a match and re-points
//! `Main.SmartCursorX/Y` (a copy of `Player.tileTargetX/Y`, `:158-162`).
//! The port models the lookup over its tile grid: the caller passes the
//! reachable rect and the held item; the module re-points
//! `App.smart_cursor: Option<(i32, i32)>` when a better spot exists.
//!
//! Ported steps, in the C# dispatch order (`:118-156`): `Step_PumpkinSeeds`
//! (:1027-1098), `Step_GrassSeeds` (:266-332), `Step_Moss` (:334-379),
//! `Step_Pigronata` (:969-1025), `Step_Torch` (:2795-2866 +
//! `IsValidSpotForTorch` :2867-2936), `Step_LawnMower` (:2895-2939),
//! `Step_PaintRoller` (:772-812), `Step_PaintBrush` (:727-770),
//! `Step_PaintScrapper` (:678-725), `Step_Acorns` (:2032-2157),
//! `Step_AlchemySeeds` (:492-580), `Step_PlanterBox` (:437-490),
//! `Step_ClayPots` (:381-435), `Step_StaffOfRegrowth` (:224-264),
//! `Step_GemCorns` (:2159-2227, over growth' `GrowTreeSettings` profiles),
//! `Step_EmptyBuckets` (:622-676), `Step_BlocksLines` (:814-888, over the
//! caller-passed `Lookup::smart_blocks_enabled`), `Step_Walls`
//! (:1100-1164, over the caller-passed player hitbox and the
//! `Collision.CanHitWithCheck` line walk), `Step_Platforms`
//! (:1324-1389, over the [`ContinuityState`] locks and
//! [`tile_target_desired`]'s placement gate) and the wire family
//! (`Step_WireCutter` :1455-1493, `Step_Actuators` :582-620,
//! `Step_ActuationRod` :1495-1535, `Step_ColoredWrenches` :1879-2030,
//! `Step_MulticolorWrench` :1725-1877 (the last two over the caller-passed
//! `Lookup::tool_mode`/`actuation_rod_lock`), `Step_MinecartTracks`
//! (:1166-1322, over the caller-passed player centre) and
//! `Step_ForceCursorToAnyMinableThing` (:2229-2268, over the caller-passed
//! gamepad state and the held item's pick/axe/hammer power) and
//! `Step_Pickaxe_MineShinies` (:2270-2348, over the held item's pick power)
//! — the seed/herb/planting family plus the paint family, gated on item id
//! / `createTile` / `createWall` and shared with the closest-to-mouse vote.
//! The paint steps read the caller-passed
//! `Lookup::paint_lookup`/`paint_coating_lookup`
//! (the C#'s inventory scan is app-side, `:38-39`).
//!
//! Out of scope (documented in the ROADMAP row): the pickaxe/axe/hammer/
//! platform/wall/wrench families, `UsingSmartCursorWithContinuity`'s
//! lock state, the grapple target arms, the inventory-driven paint lookup
//! (`Player.FindPaintOrCoating`, `Player.cs:41927-41944`), and the
//! app-side wiring (the reachable rect, the `App.smart_cursor` field, the
//! settings gate).
//!
//! Edge convention: the C# `Main.tile` array is exactly world-sized
//! (`Main.cs:66984`) and the reachable rect never touches the border, so
//! vanilla never reads out of world. The port is strictly more robust —
//! `World::tile` returns `None` at the edge and every step skips such a
//! cell, while a *neighbour* read (the flag halves) counts an out-of-world
//! cell as the inactive tile the C#'s default `Tile` would read.

use crate::tile::{FLAG_INACTIVE, Tile};

/// `Main.SmartCursorIsUsed` is a per-player setting the port reads off the
/// `TR_SMARTCURSOR` pin (the input model lands with the UI parity pass);
/// the engine's gate lives here so the caller has one condition.
pub const SMART_CURSOR_IS_USED: bool = true;

/// `ItemID.Sets.Torches` (`ItemID.cs:1371`): the 24 item ids whose
/// `SetDefaults` places a torch tile — the gate `Step_Torch` :2812 reads.
pub const TORCH_ITEMS: [u16; 24] = [
    8, 427, 3004, 523, 433, 429, 974, 1333, 1245, 3114, 430, 3045, 428, 2274, 431, 432, 4383, 4384,
    4385, 4386, 4387, 4388, 5293, 5353,
];
/// `ItemID.Sets.WaterTorches` (`ItemID.cs:1372`): the item ids that place
/// underwater torches.
pub const WATER_TORCH_ITEMS: [u16; 3] = [523, 1333, 4384];
/// `TileID.Sets.Torches` (`TileID.cs:615`) — the vanilla set is the single
/// torch tile 4 (`Sets.Torches = Factory.CreateBoolSet(4)`).
pub const TORCH_TILE: u16 = 4;
/// `ItemID.Sets.GrassSeeds` (`ItemID.cs:351`): the seed items whose step
/// plants on dirt/grass (`Step_GrassSeeds` :274).
pub const GRASS_SEED_ITEMS: [u16; 7] = [62, 59, 2171, 369, 195, 194, 5214];
/// `ItemID.Sets.Moss` (`ItemID.cs:352`): the moss-spreading items
/// (`Step_Moss` :344).
pub const MOSS_ITEMS: [u16; 11] = [
    4349, 4350, 4351, 4352, 4353, 4354, 4377, 4378, 4389, 5127, 5128,
];
/// `ItemID.Sets.IsPaintScraper` (`ItemID.cs:496`): the scraper items — the
/// PaintScraper 1100 and the Spectre PaintScraper 1545. The brush/roller
/// gates read the literal ids in the C# step bodies (no set exists for
/// them).
pub const PAINT_SCRAPER_ITEMS: [u16; 2] = [1100, 1545];
/// `ItemID.Count` (`ItemID.cs:14861`): the item-id count the placement
/// gates' range checks read.
pub const ITEM_COUNT: u16 = 6196;

/// `WiresUI.Settings.MultiToolMode` (`WiresUI.cs:13-21`): the multi-wrench's
/// channel flags, carried as the raw bitfield the C# enum stores.
pub mod multi_tool_mode {
    /// `MultiToolMode.Red` — the first wire channel.
    pub const RED: u8 = 1;
    /// `MultiToolMode.Green` — the third wire channel.
    pub const GREEN: u8 = 2;
    /// `MultiToolMode.Blue` — the second wire channel.
    pub const BLUE: u8 = 4;
    /// `MultiToolMode.Yellow` — the fourth wire channel.
    pub const YELLOW: u8 = 8;
    /// `MultiToolMode.Actuator`.
    pub const ACTUATOR: u8 = 0x10;
    /// `MultiToolMode.Cutter`.
    pub const CUTTER: u8 = 0x20;
}

/// The held-item fields the smart-cursor steps read (the C# reads them off
/// `SmartCursorUsageInfo.item`).
#[derive(Clone, Copy)]
pub struct ItemPort {
    /// `Item.type` — the item id (`ItemID`).
    pub kind: u16,
    /// `Item.createTile` (`Item.cs:169`'s `-1` default): the tile the item
    /// places.
    pub create_tile: i32,
    /// `Item.createWall` (`Item.cs:171`'s `-1` default): the wall the item
    /// places.
    pub create_wall: i32,
    /// `Item.placeStyle` (`Item.cs:173`'s `0` default): the sub-style the
    /// placement takes.
    pub place_style: i32,
    /// `Item.pick` (`Item.cs:161`): the pickaxe power — the force-cursor
    /// step's tool gate.
    pub pick: i32,
    /// `Item.axe` (`Item.cs:163`): the axe power.
    pub axe: i32,
    /// `Item.hammer` (`Item.cs:165`): the hammer power.
    pub hammer: i32,
}

/// `SmartCursorUsageInfo` (`SmartCursorHelper.cs:14-41`): the per-call
/// inputs every step shares.
pub struct Lookup {
    /// The held item.
    pub item: ItemPort,
    /// `SmartCursorUsageInfo.mouse` — the mouse world position.
    pub mouse: (f32, f32),
    /// `screenTargetX/Y` — the un-re-pointed cursor tile the steps' pre-checks
    /// read (`Step_ClayPots` :389, `Step_PlanterBox` :445).
    pub screen_target: (i32, i32),
    /// `reachableStartX` — the clamped reach rect's first column.
    pub start_x: i32,
    /// `reachableStartY` — the clamped reach rect's first row.
    pub start_y: i32,
    /// `reachableEndX` — the clamped reach rect's last column.
    pub end_x: i32,
    /// `reachableEndY` — the clamped reach rect's last row.
    pub end_y: i32,
    /// `SmartCursorUsageInfo.position` (`:21`, the C# `Vector2`) plus the
    /// holding player's `width`/`height` (`Player.cs`) — the wall step's
    /// line-of-sight origin (`Step_Walls` :1100-1102). The player entity is
    /// app-side, so the caller passes them.
    pub player_position: (f32, f32),
    /// `providedInfo.player.width`.
    pub player_width: i32,
    /// `providedInfo.player.height`.
    pub player_height: i32,
    /// `Main`'s weather/seed state `IsAlchemyPlantHarvestable` reads
    /// (`WorldGen.cs:66148-66175`) beyond the `World`'s saved fields.
    pub weather: crate::growth::WeatherCtx,
    /// `SmartCursorUsageInfo.paintLookup` (`SmartCursorHelper.cs:38`): the
    /// held brush/roller's selected tile/wall paint. The C# fills it with
    /// `TryFindingPaintInplayerInventory` (:177-191) from the player's
    /// inventory (`Player.FindPaintOrCoating`, `Player.cs:41927-41944`) —
    /// the inventory is app-side, so the port's caller passes it.
    pub paint_lookup: u32,
    /// `paintCoatingLookup` (`SmartCursorHelper.cs:39`): the selected
    /// coating — 1 Glow (the fullbright flags, GlowPaint 4668) and 2 Echo
    /// (the invisible flags, EchoCoating 5344), decoded by
    /// `WorldGen.paintCoatTile` (`WorldGen.cs:44134-44170`).
    pub paint_coating_lookup: u32,
    /// `Player.SmartCursorSettings.SmartBlocksEnabled` (`Player.cs:331`):
    /// the setting the block-lines step reads. It is a per-player settings
    /// value the app persists (`settings_ui`/`menu` own the toggle), so the
    /// port's caller passes it.
    pub smart_blocks_enabled: bool,
    /// `player.controlUseItem` (`SmartCursorHelper.cs:66`): the use button's
    /// held state — the continuity-lock reset reads it. Input is app-side,
    /// so the caller passes it.
    pub use_item_held: bool,
    /// `WiresUI.Settings.ToolMode` (`WiresUI.cs:23`): the multi-wrench's
    /// channel flags (`MultiToolMode`, :13-21). The app's wiring UI owns
    /// the setting, so the caller passes it.
    pub tool_mode: u8,
    /// `player.ActuationRodLock` (`Player.cs:2616`) — the actuation rod's
    /// channel lock (app-side player state, caller-passed).
    pub actuation_rod_lock: bool,
    /// `player.ActuationRodLockSetting` (`Player.cs:2618`) — the lock's
    /// target actuation state.
    pub actuation_rod_lock_setting: bool,
    /// `PlayerInput.UsingGamepad` (`Player.cs` input state): the
    /// force-cursor step refuses on a gamepad (input is app-side, so the
    /// caller passes it).
    pub using_gamepad: bool,
}

/// `Collision.InTileBounds` (`Collision.cs:3776`): the inclusive rect
/// test the steps' final gate runs (`Step_Torch` :2859).
pub fn in_tile_bounds(x: i32, y: i32, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> bool {
    x >= start_x && x <= end_x && y >= start_y && y <= end_y
}

/// The shared vote (`Step_Torch` :2849-2854`'s `_targets` pick, the same
/// tail in every other step): the candidate whose tile centre
/// (`X*16+8, Y*16+8`) is closest to the mouse, the C# `num == -1f`
/// sentinel and the strict `<` (the first candidate wins ties).
fn pick_closest(cands: &[(i32, i32)], mouse: (f32, f32)) -> Option<(i32, i32)> {
    pick_closest_if(cands, mouse, |_, _| true)
}

/// [`pick_closest`] with a per-candidate filter run inside the vote loop
/// (`Step_ClayPots` :418's `Collision.EmptyTile`).
fn pick_closest_if(
    cands: &[(i32, i32)],
    mouse: (f32, f32),
    keep: impl Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = -1.0f32;
    for &(x, y) in cands {
        if !keep(x, y) {
            continue;
        }
        let d = dist_to_mouse(x, y, mouse);
        if best_dist == -1.0 || d < best_dist {
            best_dist = d;
            best = Some((x, y));
        }
    }
    best
}

/// `Vector2.Distance(new Vector2(x, y) * 16f + Vector2.One * 8f, mouse)`:
/// the tile centre to the mouse (`:2849`).
fn dist_to_mouse(x: i32, y: i32, mouse: (f32, f32)) -> f32 {
    (x as f32 * 16.0 + 8.0 - mouse.0).hypot(y as f32 * 16.0 + 8.0 - mouse.1)
}

/// The seed/herb steps' shared neighbour halves (`:236-239`): `flag` =
/// any of the four orthogonal neighbours inactive, `flag2` = any of the
/// four diagonals. The C# reads the neighbours' raw `active()` (a present
/// but `inActive` tile still blocks); an out-of-world cell counts
/// inactive (see the module's edge convention).
fn neighbours_inactive(w: &crate::World, x: i32, y: i32) -> (bool, bool) {
    let active_at = |x: i32, y: i32| w.tile(x, y).is_some_and(Tile::active);
    let flag = !active_at(x - 1, y)
        || !active_at(x, y + 1)
        || !active_at(x + 1, y)
        || !active_at(x, y - 1);
    let flag2 = !active_at(x - 1, y - 1)
        || !active_at(x - 1, y + 1)
        || !active_at(x + 1, y + 1)
        || !active_at(x + 1, y - 1);
    (flag, flag2)
}

/// `Tile.lava()` (`Tile.cs:345-347`): a positive liquid amount whose kind
/// flag is lava — the alch style-4/6 clause's liquid discriminator
/// (`Step_AlchemySeeds` :536, :547).
fn is_lava(t: &Tile) -> bool {
    t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava
}

/// `Collision.EmptyTile` (`Collision.cs:1359-1386`): the cell must not be
/// an occupied tile (unless `ignoreTiles`) and must not stand inside a
/// player's or NPC's hitbox. tr-core carries no entity store — the tile
/// half is all the placement filters' `ignoreTiles` choice reads, and the
/// entity arms wait on the app-side wiring.
fn collision_empty_tile(w: &crate::World, i: i32, j: i32, ignore_tiles: bool) -> bool {
    if !ignore_tiles && w.tile(i, j).is_some_and(Tile::active) {
        return false;
    }
    true
}

/// `WorldGen.IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66138`) lives
/// in the herb module (`growth`); the alch step consults it at :506.

// ------------------------------------------------------------- torch step --

/// `SmartCursorHelper.IsValidSpotForTorch` (`SmartCursorHelper.cs:2867-2936`):
/// the tile accepts a torch — a wall behind it, a valid side anchor (left or
/// right, the beam/three-tree-column arm included), or a floor anchor
/// (`tileSolid && !tileSolidTop`, the platform row, no halfBrick).
pub fn is_valid_spot_for_torch(w: &crate::World, x: i32, y: i32, cache: Option<&Tile>) -> bool {
    if let Some(t) = cache {
        if t.wall > 0 {
            return true;
        }
        if t.kind == TORCH_TILE {
            return false;
        }
    } else {
        let Some(t) = w.tile(x, y) else {
            return false;
        };
        if t.wall > 0 {
            return true;
        }
        if t.kind == TORCH_TILE {
            return false;
        }
    }
    // `TileID.Sets.NotReallySolid` (`TileID.cs:592` — {387, 388, 10}) via
    // the shared membership test.
    let not_really_solid = crate::tileobject::not_really_solid;
    let attachable = |t: &Tile, allow_slopes: bool| -> bool {
        if !t.active() {
            return false;
        }
        let slope_ok = allow_slopes || t.slope() == 0;
        if !slope_ok {
            return false;
        }
        (crate::tiledata::TILE_SOLID.has(t.kind)
            && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
            && !not_really_solid(t.kind))
            || t.kind == 5 // the beam family tile
    };
    // The side anchors: the C#'s `slope() == 0 || slope() % 2 != X` split
    // (left: even, right: odd) — the slope-0 default covers the port's
    // flat tiles; the tree column's `IsTreeType` triple arm is dropped
    // (worldgen shrinks trees to trunks in this slice).
    if let Some(t) = w.tile(x - 1, y) {
        if attachable(t, false) {
            return true;
        }
    }
    if let Some(t) = w.tile(x + 1, y) {
        if attachable(t, false) {
            return true;
        }
    }
    if let Some(t) = w.tile(x, y + 1) {
        if t.active()
            && t.slope() == 0
            && !t.half_brick()
            && (crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                || crate::tiledata::SET_PLATFORMS.has(t.kind))
            && !not_really_solid(t.kind)
        {
            return true;
        }
    }
    false
}

/// `SmartCursorHelper.Step_Torch` (`:2795-2866`): the held item is a
/// torch → the reachable rect's closest valid empty spot. Returns the
/// re-pointed `(x, y)` or `None`. `mouse` is the mouse world position (the
/// closeness vote, `:2848`) and `(fx, fy)` the current focus (−1 when
/// empty — an earlier step's target wins).
pub fn step_torch(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    if fx != -1 || fy != -1 {
        return None;
    }
    // `:2812` — the item id gate: the vanilla reads
    // `ItemID.Sets.Torches[type]` on the ITEM id (every member is below
    // `ItemID.Count`, so the range check is implied).
    let is_water = WATER_TORCH_ITEMS.contains(&l.item.kind);
    if !TORCH_ITEMS.contains(&l.item.kind) {
        return None;
    }
    let water_flag = !is_water;
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = f32::INFINITY;
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:2832-2835`: the water-torch gate / the occupied non-cut skip
            // (`Main.tileCut` via the shared 41-id membership).
            if (water_flag && t.liquid > 0)
                || (t.active()
                    && !crate::tileobject::breakable_when_placing(t.kind)
                    && !crate::tileobject::tile_cut(t.kind))
            {
                continue;
            }
            // `:2837-2848`: no torch within the ±8 box (`17x17`'s `k/l`
            // sweep over the whole width including the row just outside).
            let mut torch_near = false;
            for k in (i - 8)..=(i + 8) {
                for l in (j - 8)..=(j + 8) {
                    if let Some(t2) = w.tile(k, l) {
                        if t2.kind == TORCH_TILE {
                            torch_near = true;
                            break;
                        }
                    }
                }
                if torch_near {
                    break;
                }
            }
            if torch_near {
                continue;
            }
            if !is_valid_spot_for_torch(w, i, j, Some(t)) {
                continue;
            }
            // `:2849-2854`: the closest-to-mouse pick.
            let dist = dist_to_mouse(i, j, l.mouse);
            if dist < best_dist {
                best_dist = dist;
                best = Some((i, j));
            }
        }
    }
    best
}

// --------------------------------------------------- seed / herb / planting --

/// `SmartCursorHelper.Step_StaffOfRegrowth` (`:224-264`): the held staff /
/// acorn axe (213/5295) re-points at the closest dirt tile with an open
/// neighbour — the tile the staff would break to plant.
pub fn step_staff_of_regrowth(
    w: &crate::World,
    l: &Lookup,
    fx: i32,
    fy: i32,
) -> Option<(i32, i32)> {
    // `:227` — the item-kind gate + the focus sentinel.
    if (l.item.kind != 213 && l.item.kind != 5295) || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            let (flag, flag2) = neighbours_inactive(w, i, j);
            // `:236-239` — the dirt tile with an open orthogonal or
            // diagonal neighbour. The C#'s second arm is
            // `(tile.type == 0) & flag2` — the non-short-circuit `&`
            // over an already-required `type == 0` reduces to `flag2`.
            if t.active() && !t.inactive() && t.kind == 0 && (flag || flag2) {
                cands.push((i, j));
            }
        }
    }
    // `:244-258` — the closest pick + the InTileBounds gate.
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_GrassSeeds` (`:266-332`): the held seed
/// (`ItemID.Sets.GrassSeeds`) re-points at the closest dirt/grass tile
/// with an open neighbour whose type matches the seed (the vanilla plants
/// the seed's grass on the targeted tile).
pub fn step_grass_seeds(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:268-274` — the focus gate uses the stricter `> -1` here; the item
    // gate reads `ItemID.Sets.GrassSeeds` (every member is below
    // `ItemID.Count`, so the range check is implied).
    if fx > -1 || fy > -1 || !GRASS_SEED_ITEMS.contains(&l.item.kind) {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            let (flag, flag2) = neighbours_inactive(w, i, j);
            if t.active() && !t.inactive() && (flag || flag2) {
                // `:290-304` — the planted-on tile per seed item kind.
                let plants_here = match l.item.kind {
                    59 | 2171 => t.kind == 0 || t.kind == 59, // corrupt/crimson
                    194 | 195 => t.kind == 59,                // mushroom/jungle
                    5214 => t.kind == 57,                     // ash
                    _ => t.kind == 0,                         // the plain grass seeds
                };
                if plants_here {
                    cands.push((i, j));
                }
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_Moss` (`:334-379`): the held moss item
/// (`ItemID.Sets.Moss`) re-points at the closest stone/pearlstone tile
/// with an open neighbour (the moss spreads onto it).
pub fn step_moss(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:336-344` — the focus gate's `> -1` form + the moss item set.
    if fx > -1 || fy > -1 || !MOSS_ITEMS.contains(&l.item.kind) {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            let (flag, flag2) = neighbours_inactive(w, i, j);
            // `:353` — stone 1 or pearlstone 38 with an open neighbour.
            if t.active() && !t.inactive() && (flag || flag2) && (t.kind == 1 || t.kind == 38) {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_ClayPots` (`:381-435`): the held clay pot
/// (`createTile == 78`) re-points at the closest cell that may take a pot
/// — empty or replaceable, above a clean full solid.
pub fn step_clay_pots(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:383` — the placement-tile gate.
    if l.item.create_tile != 78 || fx != -1 || fy != -1 {
        return None;
    }
    // `:389-396` — the screen-target pre-check: an occupied or
    // out-of-rect target disables the scan.
    let mut skip = w
        .tile(l.screen_target.0, l.screen_target.1)
        .is_some_and(Tile::active);
    if !in_tile_bounds(
        l.screen_target.0,
        l.screen_target.1,
        l.start_x,
        l.start_y,
        l.end_x,
        l.end_y,
    ) {
        skip = true;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    if !skip {
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                let (Some(t), Some(t2)) = (w.tile(i, j), w.tile(i, j + 1)) else {
                    continue;
                };
                // `:405` — the cell may be replaced (`tileCut` /
                // `BreakableWhenPlacing`) and the cell below is a clean
                // full solid (`nactive` = active, not framed off).
                if (!t.active()
                    || crate::tileobject::tile_cut(t.kind)
                    || crate::tileobject::breakable_when_placing(t.kind))
                    && crate::tileobject::nactive(t2)
                    && !t2.half_brick()
                    && t2.slope() == 0
                    && crate::tiledata::TILE_SOLID.has(t2.kind)
                {
                    cands.push((i, j));
                }
            }
        }
    }
    // `:414-428` — the EmptyTile filter inside the vote, the closest pick,
    // and the C#'s `num != -1f` + InTileBounds commit guards.
    let p = pick_closest_if(&cands, l.mouse, |x, y| collision_empty_tile(w, x, y, true))?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_PlanterBox` (`:437-490`): the held planter box
/// (`createTile == 380`) re-points beside an existing box — the placement
/// extends the row sideways.
pub fn step_planter_box(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:439` — the placement-tile gate.
    if l.item.create_tile != 380 || fx != -1 || fy != -1 {
        return None;
    }
    // `:445` — the screen-target pre-check: an occupied planter box
    // disables the scan.
    let skip = w
        .tile(l.screen_target.0, l.screen_target.1)
        .is_some_and(|t| t.active() && t.kind == 380);
    let mut cands: Vec<(i32, i32)> = Vec::new();
    if !skip {
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                let Some(t) = w.tile(i, j) else {
                    continue;
                };
                if t.active() && t.kind == 380 {
                    // `:458-465` — the box extends into an empty or
                    // replaceable left/right neighbour.
                    for nx in [i - 1, i + 1] {
                        if let Some(tn) = w.tile(nx, j) {
                            if !tn.active()
                                || crate::tileobject::tile_cut(tn.kind)
                                || crate::tileobject::breakable_when_placing(tn.kind)
                            {
                                cands.push((nx, j));
                            }
                        }
                    }
                }
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_AlchemySeeds` (`:492-580`): the held alch seed
/// (`createTile == 82`) re-points at the closest plantable cell — the
/// `placeStyle` selects the ground family the herb wants.
pub fn step_alchemy_seeds(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:494` — the placement-tile gate.
    if l.item.create_tile != 82 || fx != -1 || fy != -1 {
        return None;
    }
    let style = l.item.place_style;
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let (Some(t), Some(t2)) = (w.tile(i, j), w.tile(i, j + 1)) else {
                continue;
            };
            // `:506` — the cell may host a herb: empty/replaceable, or a
            // harvestable grown herb (`frameX / 18` = the herb style).
            let plantable = !t.active()
                || crate::tileobject::breakable_when_placing(t.kind)
                || (crate::tileobject::tile_cut(t.kind) && t.kind != 82 && t.kind != 83)
                || crate::growth::is_harvestable_herb_with_seed(
                    w,
                    t.kind,
                    t.frame_x as i32 / 18,
                    j,
                    &l.weather,
                );
            // `:507` — the ground below is a clean full tile.
            let ground_ok = crate::tileobject::nactive(t2) && !t2.half_brick() && t2.slope() == 0;
            if !plantable || !ground_ok {
                continue;
            }
            // `:509-556` — per-style ground clause: a matching reject-set
            // grounds the candidate unless the cell's liquid overrides it
            // (the C# `continue`s the column there).
            let rejected = match style {
                0 => matches!(t2.kind, 78 | 380 | 2 | 477 | 109 | 492) && t.liquid == 0,
                1 => matches!(t2.kind, 78 | 380 | 60) && t.liquid == 0,
                2 => matches!(t2.kind, 78 | 380 | 0 | 59) && t.liquid == 0,
                3 => matches!(t2.kind, 78 | 380 | 203 | 199 | 23 | 25) && t.liquid == 0,
                4 => matches!(t2.kind, 78 | 380 | 53 | 116) && !(t.liquid > 0 && is_lava(t)),
                5 => matches!(t2.kind, 78 | 380 | 57 | 633) && !(t.liquid > 0 && !is_lava(t)),
                6 => {
                    matches!(t2.kind, 78 | 380 | 147 | 161 | 163 | 164 | 200)
                        && !(t.liquid > 0 && is_lava(t))
                }
                _ => false,
            };
            if rejected {
                continue;
            }
            cands.push((i, j));
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_Pigronata` (`:969-1025`): the held pigronata
/// statue (`createTile == 454`) re-points at the closest surface-level
/// spot whose 4x4 box sits on solid ground and is otherwise clear.
pub fn step_pigronata(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:971` — the placement-tile gate.
    if l.item.create_tile != 454 || fx != -1 || fy != -1 {
        return None;
    }
    // `:978` — the column stops at the world surface minus two rows (the
    // C# fuses the bound into the j-loop condition).
    let j_max = l.end_y.min((w.world_surface - 2.0).floor() as i32);
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=j_max {
            // `:980-996` — the 4x4 sweep: the row below the candidate is
            // solid (`SolidTile`), the rows above are clear (active cells
            // only survive as cut tiles or the statue itself).
            let mut clear = true;
            'box_sweep: for k in (i - 2)..=(i + 1) {
                for m in (j - 1)..=(j + 2) {
                    let Some(t) = w.tile(k, m) else {
                        continue;
                    };
                    if m == j - 1 {
                        if !crate::edit::solid_tile(w, k, m) {
                            clear = false;
                            break 'box_sweep;
                        }
                    } else if t.active() && (!crate::tileobject::tile_cut(t.kind) || t.kind == 454)
                    {
                        clear = false;
                        break 'box_sweep;
                    }
                }
            }
            if clear {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_PumpkinSeeds` (`:1027-1098`): the held pumpkin
/// seed (`createTile == 254`) re-points at the closest surface-level
/// 2-wide grass spot with clear space above.
pub fn step_pumpkin_seeds(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:1029` — the placement-tile gate.
    if l.item.create_tile != 254 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            // `:1035-1036` — the two ground cells are read before the
            // surface guard (the C#'s order).
            let (Some(t), Some(t2)) = (w.tile(i, j + 1), w.tile(i + 1, j + 1)) else {
                continue;
            };
            // `:1040-1042` — the inner break: the rest of the column is
            // abandoned once past the surface.
            if j as f64 > w.world_surface - 2.0 {
                break;
            }
            // `:1046-1068` — both ground cells are flat grass-family
            // tiles, the two header cells are cuttable.
            let mut ok = t.active()
                && t2.active()
                && t.slope() == 0
                && t2.slope() == 0
                && !t.half_brick()
                && !t2.half_brick()
                && matches!(t.kind, 2 | 477 | 109 | 492)
                && matches!(t2.kind, 2 | 477 | 109 | 492);
            if ok {
                for k in i..=i + 1 {
                    for m in (j - 1)..=j {
                        if let Some(t3) = w.tile(k, m) {
                            if t3.active()
                                && (crate::tiledata::TILE_SOLID.has(t3.kind)
                                    || !crate::tileobject::can_cut_tile(w, k, m))
                            {
                                ok = false;
                            }
                        }
                    }
                }
            }
            if ok {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_LawnMower` (`:2895-2939`): the held lawn mower
/// (4049) re-points at the closest grass/moss tile (the tool cuts it).
pub fn step_lawn_mower(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:2901` — the item-kind gate (the C#'s `_ = screenTargetX/Y`
    // discards are decompiler artifacts with no side effects).
    if l.item.kind != 4049 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:2913` — grass 2 or the moss-free variant 109.
            if t.active() && (t.kind == 2 || t.kind == 109) {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// ------------------------------------------------------------- paint family --

/// `SmartCursorHelper.Step_PaintScrapper` (`:678-725`): the held scraper
/// (1100/1545) re-points at the closest tile or wall carrying paint or a
/// coating — or at a LongMoss tile (184), the scraper's second target
/// (`Player.cs:41822-41824`).
pub fn step_paint_scrapper(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:680` — the item gate + the focus sentinel. The scraper does not
    // read the paint lookup: `TryFindingPaintInplayerInventory` :182 only
    // fills it for the brush/roller ids, and its scan targets anything
    // paintable or coated.
    if !PAINT_SCRAPER_ITEMS.contains(&l.item.kind) || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:692-697` — the two OR-ed groups: a painted / coated block
            // (LongMoss 184 included), or a painted / coated wall behind
            // the cell (the C# `|` is the non-short-circuit bool OR over
            // pure operands — `||` is equivalent).
            let mut painted = false;
            if t.active() {
                painted |= t.paint() > 0
                    || t.kind == 184
                    || t.fullbright_block()
                    || w.invisible_block(i, j);
            }
            if t.wall > 0 {
                painted |=
                    w.wall_paint(i, j) > 0 || w.fullbright_wall(i, j) || w.invisible_wall(i, j);
            }
            if painted {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_PaintBrush` (`:727-770`): the held brush
/// (1071/1543) re-points at the closest block that still needs the
/// selected paint (a different tile paint) or coating.
pub fn step_paint_brush(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:729` — the item-type gate, the "no paint selected" gate (the C#
    // `||` chain) and the focus sentinel.
    if (l.item.kind != 1071 && l.item.kind != 1543)
        || (l.paint_lookup == 0 && l.paint_coating_lookup == 0)
        || fx != -1
        || fy != -1
    {
        return None;
    }
    let (paint, coating) = (l.paint_lookup, l.paint_coating_lookup);
    let mut cands: Vec<(i32, i32)> = Vec::new();
    // `:734` — the C#'s redundant `paintLookup != 0 || paintCoatingLookup != 0`
    // re-check (the gate above already refused the both-zero case).
    if paint != 0 || coating != 0 {
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                let Some(t) = w.tile(i, j) else {
                    continue;
                };
                // `:743` — the block needs the paint (`tile.color() !=
                // paintLookup`) or the coating (Glow 1 = !fullbrightBlock,
                // Echo 2 = !invisibleBlock).
                if t.active()
                    && ((paint != 0 && t.paint() as u32 != paint)
                        || (coating == 1 && !t.fullbright_block())
                        || (coating == 2 && !w.invisible_block(i, j)))
                {
                    cands.push((i, j));
                }
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_PaintRoller` (`:772-812`): the held roller
/// (1072/1544) re-points at the closest wall — behind a non-solid cell —
/// that still needs the selected paint or coating.
pub fn step_paint_roller(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:774` — the roller's item-type gate + the no-paint gate + the
    // focus sentinel.
    if (l.item.kind != 1072 && l.item.kind != 1544)
        || (l.paint_lookup == 0 && l.paint_coating_lookup == 0)
        || fx != -1
        || fy != -1
    {
        return None;
    }
    let (paint, coating) = (l.paint_lookup, l.paint_coating_lookup);
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:786` — the wall behind a cell that is not a full solid
            // (empty, non-solid or solidTop), needing the wall paint or
            // the wall coating (Glow 1, Echo 2).
            if t.wall > 0
                && (!t.active()
                    || !crate::tiledata::TILE_SOLID.has(t.kind)
                    || crate::tiledata::TILE_SOLID_TOP.has(t.kind))
                && ((paint != 0 && w.wall_paint(i, j) as u32 != paint)
                    || (coating == 1 && !w.fullbright_wall(i, j))
                    || (coating == 2 && !w.invisible_wall(i, j)))
            {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// ------------------------------------------------------------ tree-seed step --

/// `SmartCursorHelper.Step_Acorns` (`:2032-2157`): the held acorn (27)
/// re-points at the closest plantable sapling spot — an open cell above a
/// fitting ground with no sapling too close. The vanilla's three ground
/// arms (jungle grass 60, the grass family, the sand family) and the
/// 15-tile removal pass drop every spot whose ground row stands alone.
pub fn step_acorns(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:2037` — the item-kind gate + the focus sentinel + the height gate
    // (the vanilla never plants at or above the row 20).
    if l.item.kind != 27 || fx != -1 || fy != -1 || l.start_y <= 20 {
        return None;
    }
    let (num, num2, num3) = (9, 14, 20); // `:2034-2036` — the height budgets
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            // `:2044-2052` — the nine reads; an out-of-world cell skips the
            // candidate (the port's edge convention — the C#'s array is
            // exactly world-sized and the rect never reaches the border).
            let (Some(t), Some(t2), Some(t3), Some(t4), Some(t5)) = (
                w.tile(i, j),
                w.tile(i, j - 1),
                w.tile(i, j + 1),
                w.tile(i - 1, j),
                w.tile(i + 1, j),
            ) else {
                continue;
            };
            // `:2054` — the cell and the one above must be empty (a cut /
            // replaceable cell passes) and the cell below must be a solid
            // ground (`SolidTile2`).
            let occupied = |t: &Tile| {
                t.active()
                    && !crate::tileobject::tile_cut(t.kind)
                    && !crate::tileobject::breakable_when_placing(t.kind)
            };
            if occupied(t) || occupied(t2) || !t3.active() || !crate::edit::solid_tile2(w, i, j + 1)
            {
                continue;
            }
            // `:2058-2060` — the sapling proximity: `flag` the immediate
            // neighbours, `flag2` the ±2/±3 row.
            let sapling_at = |x: i32, y: i32| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && crate::growth::is_common_sapling(t.kind))
            };
            let flag = sapling_at(i - 1, j) || sapling_at(i + 1, j);
            let flag2 = flag
                || sapling_at(i - 2, j)
                || sapling_at(i + 2, j)
                || sapling_at(i - 3, j)
                || sapling_at(i + 3, j);
            // `:2061-2093` — the three ground arms.
            match t3.kind {
                60 => {
                    // Jungle grass: the 14-row box.
                    if !flag2
                        && crate::growth::empty_tile_check(
                            w,
                            i - 2,
                            i + 2,
                            j - num2 + 1,
                            j,
                            Some(20),
                        )
                    {
                        cands.push((i, j));
                    }
                }
                2 | 23 | 109 | 147 | 199 | 477 | 492 | 633 | 661 | 662 => {
                    // The grass family: the 9-row box, dry only.
                    if !flag2
                        && t4.liquid == 0
                        && t.liquid == 0
                        && t5.liquid == 0
                        && crate::growth::empty_tile_check(
                            w,
                            i - 2,
                            i + 2,
                            j - num + 1,
                            j,
                            Some(20),
                        )
                    {
                        cands.push((i, j));
                    }
                }
                53 | 112 | 116 | 234 => {
                    // The sand family: the 20-row box over one column.
                    if !flag
                        && t.liquid == 0
                        && crate::growth::empty_tile_check(w, i, i, j - num3, j, Some(20))
                    {
                        cands.push((i, j));
                    }
                }
                _ => {}
            }
        }
    }
    // `:2096-2136` — the removal pass: the spot survives only when a cell
    // beside its ground row is itself an acorn ground (the 15-tile list) —
    // a ground row standing alone is dropped.
    cands.retain(|&(x, y)| {
        [-1, 1].iter().any(|&dx| {
            w.tile(x + dx, y + 1).is_some_and(|t| {
                t.active()
                    && matches!(
                        t.kind,
                        2 | 23
                            | 53
                            | 60
                            | 109
                            | 112
                            | 116
                            | 147
                            | 199
                            | 234
                            | 477
                            | 492
                            | 633
                            | 661
                            | 662
                    )
            })
        })
    });
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_GemCorns` (`:2159-2227`): the held gem/vanity
/// seed re-points at the closest plantable spot for its
/// `GrowTreeSettings` profile — the profile's ground test, the dry clear
/// box of the profile's height and no sapling within the ±3 columns.
pub fn step_gem_corns(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:2161` — the profile lookup (the C#'s `TryGetFromItemId`), the focus
    // sentinel and the height gate.
    let Some(profile) = crate::growth::grow_tree_profile_from_item(l.item.kind) else {
        return None;
    };
    if fx != -1 || fy != -1 || l.start_y <= 20 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            // `:2171-2178` — the nine reads (an out-of-world cell skips,
            // the port's edge convention).
            let (Some(t), Some(t2), Some(t3), Some(t4), Some(t5)) = (
                w.tile(i, j),
                w.tile(i, j - 1),
                w.tile(i, j + 1),
                w.tile(i - 1, j),
                w.tile(i + 1, j),
            ) else {
                continue;
            };
            // `:2179` — the one long predicate: the ground passes the
            // profile's test, the spot and the cell above are empty (a cut/
            // replaceable cell passes), no sapling within ±3 columns, the
            // ground is solid (`SolidTile2`), the three cells dry and the
            // profile-height box is clear (the profile's own sapling is the
            // `EmptyTileCheck`'s ignore id).
            let sapling_at = |x: i32, y: i32| {
                w.tile(x, y)
                    .is_some_and(|t| t.active() && crate::growth::is_common_sapling(t.kind))
            };
            if profile.ground.test(t3.kind)
                && (!t.active()
                    || crate::tileobject::tile_cut(t.kind)
                    || crate::tileobject::breakable_when_placing(t.kind))
                && (!t2.active()
                    || crate::tileobject::tile_cut(t2.kind)
                    || crate::tileobject::breakable_when_placing(t2.kind))
                && (!t4.active() || !crate::growth::is_common_sapling(t4.kind))
                && (!t5.active() || !crate::growth::is_common_sapling(t5.kind))
                && !(sapling_at(i - 2, j)
                    || sapling_at(i + 2, j)
                    || sapling_at(i - 3, j)
                    || sapling_at(i + 3, j))
                && t3.active()
                && crate::edit::solid_tile2(w, i, j + 1)
                && t4.liquid == 0
                && t.liquid == 0
                && t5.liquid == 0
                && crate::growth::empty_tile_check(
                    w,
                    i - 2,
                    i + 2,
                    j - profile.tree_height_max,
                    j,
                    Some(profile.sapling_tile_type),
                )
            {
                cands.push((i, j));
            }
        }
    }
    // `:2189-2206` — the removal pass: the spot survives when a cell beside
    // its ground row passes the profile's ground test (the C# runs the
    // profile's delegate here, not the acorn's 15-tile list).
    cands.retain(|&(x, y)| {
        [-1, 1].iter().any(|&dx| {
            w.tile(x + dx, y + 1)
                .is_some_and(|t| t.active() && profile.ground.test(t.kind))
        })
    });
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.AllowNormalBlockPlacementBehaviourForItemType`
/// (`:211-222`): the block-placement steps' item gate — the staff/acorn axe
/// (213/5295), the seed and moss items and any id past `ItemID.Count` take
/// their own steps instead.
fn allow_normal_block_placement(kind: u16) -> bool {
    if kind >= ITEM_COUNT || kind == 213 || kind == 5295 {
        return false;
    }
    !GRASS_SEED_ITEMS.contains(&kind) && !MOSS_ITEMS.contains(&kind)
}

/// `SmartCursorHelper.Step_EmptyBuckets` (`:622-676`): the held empty
/// bucket (205) re-points at the closest liquid cell whose 3x3 box of the
/// same liquid kind holds more than 100 (the bucket's minimum draw).
pub fn step_empty_buckets(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:624` — the item-kind gate + the focus sentinel.
    if l.item.kind != 205 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:634` — the dry cell skips.
            if t.liquid == 0 {
                continue;
            }
            let kind = t.liquid_kind();
            // `:639-648` — the 3x3 box sum of the same liquid kind (an
            // out-of-world cell contributes nothing — the C#'s default
            // `Tile` reads liquid 0 with kind `Water`, which only ever adds
            // a zero).
            let mut sum: u32 = 0;
            for k in (i - 1)..=(i + 1) {
                for m in (j - 1)..=(j + 1) {
                    if let Some(t2) = w.tile(k, m) {
                        if t2.liquid_kind() == kind {
                            sum += t2.liquid as u32;
                        }
                    }
                }
            }
            // `:650` — the 100-liquid minimum.
            if sum > 100 {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_BlocksLines` (`:814-888`): the held block item
/// re-points at the closest replaceable cell with one full-solid neighbour
/// — the smart-blocks line placement.
pub fn step_blocks_lines(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:817` — the single gate chain: the item range, the settings toggle,
    // the createTile's presence, the placement-behaviour gate and the
    // createTile's solid/non-solidTop/non-frameImportant flags.
    let create_tile = l.item.create_tile;
    if l.item.kind >= ITEM_COUNT
        || !l.smart_blocks_enabled
        || create_tile <= -1
        || !allow_normal_block_placement(l.item.kind)
        || !crate::tiledata::TILE_SOLID.has(create_tile as u16)
        || crate::tiledata::TILE_SOLID_TOP.has(create_tile as u16)
        || w.frame_important.get(create_tile as usize).copied() == Some(true)
        || fx != -1
        || fy != -1
    {
        return None;
    }
    // `:822-830` — the screen-target pre-check: an occupied or out-of-rect
    // target disables the scan.
    let mut skip = w
        .tile(l.screen_target.0, l.screen_target.1)
        .is_some_and(Tile::active);
    if !in_tile_bounds(
        l.screen_target.0,
        l.screen_target.1,
        l.start_x,
        l.start_y,
        l.end_x,
        l.end_y,
    ) {
        skip = true;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    if !skip {
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                let Some(t) = w.tile(i, j) else {
                    continue;
                };
                // `:838` — the cell may be replaced.
                if !t.active()
                    || crate::tileobject::tile_cut(t.kind)
                    || crate::tileobject::breakable_when_placing(t.kind)
                {
                    // `:841-856` — any of the four neighbours is a full
                    // solid (`tileSolid && !tileSolidTop`).
                    let anchored = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|&(dx, dy)| {
                        w.tile(i + dx, j + dy).is_some_and(|tn| {
                            tn.active()
                                && crate::tiledata::TILE_SOLID.has(tn.kind)
                                && !crate::tiledata::TILE_SOLID_TOP.has(tn.kind)
                        })
                    });
                    if anchored {
                        cands.push((i, j));
                    }
                }
            }
        }
    }
    // `:871-881` — the `Collision.EmptyTile` filter (the default
    // `ignoreTiles: false` — an occupied cell fails here even when the scan
    // let it through as a cut tile), the closest pick and the
    // `num != -1f` + InTileBounds commit guards.
    let p = pick_closest_if(&cands, l.mouse, |x, y| collision_empty_tile(w, x, y, false))?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// --------------------------------------------------------------- wall step --

/// `DelegateMethods.NotDoorStand` (`DelegateMethods.cs:854-865`): the open
/// door's 11 whose frame column (18..53) blocks the line walk — the C#'s
/// `Utils.TileActionAttempt` the walk applies at every step.
fn not_door_stand(w: &crate::World, x: i32, y: i32) -> bool {
    match w.tile(x, y) {
        Some(t) if t.active() && t.kind == 11 => t.frame_x >= 18 && t.frame_x < 54,
        _ => true,
    }
}

/// The C#'s inline "solid full block" test (`CanHitWithCheck` :526-534,
/// :564-571): `!inActive() && active() && tileSolid && !tileSolidTop &&
/// slope() == 0 && !halfBrick()` — i.e. `Tile.nactive()` plus the flags.
fn solid_full(t: &Tile) -> bool {
    crate::tileobject::nactive(t)
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
}

/// `Collision.CanHitWithCheck` (`Collision.cs:490-586`): the line walk from
/// the player's centre tile to the target cell — the wall step's reach
/// check. The C#'s `Utils.TileActionAttempt` delegate is the port's `check`
/// closure; the C#'s `Main.tile == null` / catch guards read as the port's
/// `World::tile` returning `None` → `false` (the same result as the vanilla
/// catch, which returns `false`).
fn can_hit_with_check(
    w: &crate::World,
    p1: (f32, f32),
    w1: i32,
    h1: i32,
    p2: (f32, f32),
    w2: i32,
    h2: i32,
    check: impl Fn(i32, i32) -> bool,
) -> bool {
    let (max_x, max_y) = (w.width, w.height);
    // `:493-518` — the two centre tiles, clamped to the C#'s windows
    // (columns 1..maxTilesX-1, rows 1..maxTilesY-40).
    let mut num = ((p1.0 + (w1 / 2) as f32) / 16.0) as i32;
    let mut num2 = ((p1.1 + (h1 / 2) as f32) / 16.0) as i32;
    let mut num3 = ((p2.0 + (w2 / 2) as f32) / 16.0) as i32;
    let mut num4 = ((p2.1 + (h2 / 2) as f32) / 16.0) as i32;
    num = num.clamp(1, max_x - 1);
    num3 = num3.clamp(1, max_x - 1);
    num2 = num2.clamp(1, max_y - 40);
    num4 = num4.clamp(1, max_y - 40);
    loop {
        // `:523-548` — the Manhattan step: the wider axis advances first,
        // and a full solid tower on both sides of the crossed column blocks.
        let (num5, num6) = ((num - num3).abs(), (num2 - num4).abs());
        if num == num3 && num2 == num4 {
            return true;
        }
        if num5 > num6 {
            num = if num >= num3 { num - 1 } else { num + 1 };
            let (Some(above), Some(below)) = (w.tile(num, num2 - 1), w.tile(num, num2 + 1)) else {
                return false;
            };
            if solid_full(above) && solid_full(below) {
                return false;
            }
        } else {
            num2 = if num2 >= num4 { num2 - 1 } else { num2 + 1 };
            let (Some(left), Some(right)) = (w.tile(num - 1, num2), w.tile(num + 1, num2)) else {
                return false;
            };
            if solid_full(left) && solid_full(right) {
                return false;
            }
        }
        // `:549-556` — the reached cell may not itself be a full solid.
        let Some(cur) = w.tile(num, num2) else {
            return false;
        };
        if cur.active()
            && crate::tileobject::nactive(&cur)
            && crate::tiledata::TILE_SOLID.has(cur.kind)
            && !crate::tiledata::TILE_SOLID_TOP.has(cur.kind)
        {
            return false;
        }
        // `:576-578` — the do/while's condition: the walk continues while
        // the check passes; a false ends it with `return false`.
        if !check(num, num2) {
            return false;
        }
    }
}

/// `SmartCursorHelper.Step_Walls` (`:1100-1164`): the held wall item
/// (`createWall > 0`) re-points at the closest reachable cell whose wall
/// slot is free, that stands beside a tile-or-wall neighbour and is not an
/// open-door anchor frame.
pub fn step_walls(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:1101-1106` — the player's hitbox (the C#'s line-of-sight origin)
    // and the createWall gate + the focus sentinel.
    let (width, height) = (l.player_width, l.player_height);
    if l.item.create_wall <= 0 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:1114` — a free wall slot behind a non-full-solid cell,
            // reachable from the player (`NotDoorStand` is the walk's check).
            if t.wall == 0
                && (!t.active()
                    || !crate::tiledata::TILE_SOLID.has(t.kind)
                    || crate::tiledata::TILE_SOLID_TOP.has(t.kind))
                && can_hit_with_check(
                    w,
                    l.player_position,
                    width,
                    height,
                    (i as f32 * 16.0, j as f32 * 16.0),
                    16,
                    16,
                    |x, y| not_door_stand(w, x, y),
                )
            {
                // `:1116-1132` — any of the four neighbours carries a tile
                // or a wall (the `||`, not the `&&`).
                let mut flag = false;
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    if let Some(tn) = w.tile(i + dx, j + dy) {
                        if tn.active() || tn.wall > 0 {
                            flag = true;
                        }
                    }
                }
                // `:1133-1136` — an open-door anchor frame vetoes the spot.
                if crate::tileobject::is_open_door_anchor_frame(w, i, j) {
                    flag = false;
                }
                if flag {
                    cands.push((i, j));
                }
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// -------------------------------------------------- platform / continuity --

/// The C#'s two `SmartCursorHelper` statics (`SmartCursorHelper.cs:53-57`)
/// — the platform-continuity locks. The C#'s static fields become the
/// port's per-session state the caller holds across lookups (a world's
/// smart cursor is single-threaded, one player at a time).
#[derive(Clone, Copy, Default)]
pub struct ContinuityState {
    /// `_lockedDesiredDirection` (`:55`) — the direction the platform line
    /// keeps extending.
    pub locked_desired_direction: Option<(i32, i32)>,
    /// `_lockedContinuityCoords` (`:57`) — the focus cell the line started
    /// from.
    pub locked_continuity_coords: Option<(i32, i32)>,
}

/// `SmartCursorHelper.UsingSmartCursorWithContinuity` (`:167-175`): the
/// continuity locks are armed only while the held item places a platform
/// (`TileID.Sets.Platforms`, `TileID.cs:534`).
fn using_smart_cursor_with_continuity(create_tile: i32) -> bool {
    create_tile >= 0 && crate::tiledata::SET_PLATFORMS.has(create_tile as u16)
}

/// `SmartCursorHelper.IsPlatform` (`:1446-1453`): the cell carries a
/// platform tile.
fn is_platform(w: &crate::World, x: i32, y: i32) -> bool {
    w.tile(x, y)
        .is_some_and(|t| t.active() && crate::tiledata::SET_PLATFORMS.has(t.kind))
}

/// `SmartCursorHelper.AllowedForContinuity` (`:1404-1429`): the candidate
/// continues the locked line — the lock's own cell or a
/// `skips_allowed` walk along the locked direction.
fn allowed_for_continuity(x: i32, y: i32, state: &ContinuityState, skips_allowed: i32) -> bool {
    let Some(coords) = state.locked_continuity_coords else {
        return true;
    };
    if coords == (x, y) {
        return true;
    }
    let Some(dir) = state.locked_desired_direction else {
        return false;
    };
    let mut walk = coords;
    for _ in 0..skips_allowed {
        walk.0 += dir.0;
        walk.1 += dir.1;
        if (x, y) == walk {
            return true;
        }
    }
    false
}

/// `SmartCursorHelper.GetDesiredDirectionFrom` (`:1431-1444`): the mouse's
/// bearing from the cell, quantised to one of eight directions ×1.5 — the
/// C#'s `ToRotation` atan2, the π/4 bucket with its half-bucket offset, the
/// `ToRotationVector2` unit vector scaled by 1.5 and `ToPoint`'s truncation
/// (`Utils.cs:1655-1658`, `:1817-1820`).
fn get_desired_direction_from(offset: (f32, f32), state: &ContinuityState) -> (i32, i32) {
    if let Some(dir) = state.locked_desired_direction {
        return dir;
    }
    let mut num = offset.1.atan2(offset.0);
    if num < 0.0 {
        num += std::f32::consts::PI * 2.0;
    }
    let num2 = std::f32::consts::PI / 4.0;
    // `:1443` — the bucket index (the `(int)` truncation), its angle, the
    // unit vector, the ×1.5 scale and the second truncation.
    let bucket = ((num + num2 / 2.0) % (std::f32::consts::PI * 2.0) / num2) as i32;
    let angle = bucket as f32 * num2;
    ((angle.cos() * 1.5) as i32, (angle.sin() * 1.5) as i32)
}

/// `SmartCursorHelper.TileTargetDesired` (`:1391-1402`): the placement gate
/// the C# reads in `Player.PlaceThing_Tiles` (`Player.cs:39721`) — while a
/// continuity lock stands, the player's tile target must sit on the smart
/// cursor's own (re-pointed) cell.
pub fn tile_target_desired(
    state: &ContinuityState,
    smart_cursor_showing: bool,
    target: (i32, i32),
    smart_cursor: (i32, i32),
) -> bool {
    if state.locked_continuity_coords.is_some() {
        if smart_cursor_showing && target.0 == smart_cursor.0 {
            return target.1 == smart_cursor.1;
        }
        return false;
    }
    true
}

/// `SmartCursorHelper.Step_Platforms` (`:1324-1389`): the held platform
/// item re-points at the closest cell that extends the platform line —
/// the two-key vote (distance, then the dot product along the locked
/// direction) and, on a hit, both continuity locks are written.
pub fn step_platforms(
    w: &crate::World,
    l: &Lookup,
    fx: i32,
    fy: i32,
    state: &mut ContinuityState,
) -> Option<(i32, i32)> {
    // `:1326` — the createTile gate (a platform-placing item) + the focus
    // sentinel + the screen-target platform pre-check.
    if l.item.create_tile < 0
        || !crate::tiledata::SET_PLATFORMS.has(l.item.create_tile as u16)
        || fx != -1
        || fy != -1
        || is_platform(w, l.screen_target.0, l.screen_target.1)
    {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    let mut dirs: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:1338` — the desired direction from the mouse's bearing to
            // this cell; `flag` = the cell is not itself a platform.
            let desired = get_desired_direction_from(
                (
                    l.mouse.0 - (i as f32 * 16.0 + 8.0),
                    l.mouse.1 - (j as f32 * 16.0 + 8.0),
                ),
                state,
            );
            let mut flag = !is_platform(w, i, j);
            // `:1338-1341` — the horizontal-join exception: an active,
            // non-solid cell beside a platform row is NOT skipped when the
            // direction is horizontal (the row extends).
            if flag
                && desired.1 == 0
                && t.active()
                && !crate::edit::solid_tile(w, i, j)
                && (is_platform(w, i - 1, j) || is_platform(w, i + 1, j))
            {
                flag = false;
            }
            // `:1343` — the C# skips the cell while the flag stands.
            if flag {
                continue;
            }
            // `:1346-1347` — the slope arm (2 for a diagonal, 1 for the
            // anti-diagonal, 0 for flat): when the cell has no matching
            // slope, the placement needs the side-neighbour clearance.
            let num = if desired.0 == desired.1 {
                2
            } else if desired.0 == -desired.1 {
                1
            } else {
                0
            };
            if (num == 0 || t.slope() as i32 != num)
                && (desired.0 != 0
                    || (!is_platform(w, i - 1, j + desired.1)
                        && !is_platform(w, i + 1, j + desired.1)))
            {
                // `:1350-1356` — the target cell may be replaced and the
                // walk continues the locked line.
                let (nx, ny) = (i + desired.0, j + desired.1);
                let Some(tile) = w.tile(nx, ny) else {
                    continue;
                };
                if (!tile.active() || crate::tileobject::tile_cut(tile.kind))
                    && allowed_for_continuity(nx, ny, state, 2)
                {
                    cands.push((nx, ny));
                    dirs.push(desired);
                }
            }
        }
    }
    // `:1364-1378` — the two-key vote: the closest centre to the mouse,
    // then the larger dot product along the stored direction (the C#'s
    // `num3` sentinel is `-1f`, so the first candidate always takes it).
    let mut best: Option<((i32, i32), (i32, i32))> = None;
    let mut best_dist = -1.0f32;
    let mut best_dot = -1.0f32;
    for (k, &(x, y)) in cands.iter().enumerate() {
        let dx = l.mouse.0 - (x as f32 * 16.0 + 8.0);
        let dy = l.mouse.1 - (y as f32 * 16.0 + 8.0);
        let dist = dx.hypot(dy);
        let dot = dx * dirs[k].0 as f32 + dy * dirs[k].1 as f32;
        if best_dist == -1.0 || dist < best_dist || (dist == best_dist && dot > best_dot) {
            best_dist = dist;
            best_dot = dot;
            best = Some(((x, y), dirs[k]));
        }
    }
    let ((px, py), dir) = best?;
    if !in_tile_bounds(px, py, l.start_x, l.start_y, l.end_x, l.end_y) {
        return None;
    }
    // `:1383-1384` — the only writer of the locks outside the reset.
    state.locked_desired_direction = Some(dir);
    state.locked_continuity_coords = Some((px, py));
    Some((px, py))
}

// ------------------------------------------------------------- wire family --

/// The C#'s `Tile.wire()`/`wire2()`/`wire3()`/`wire4()` (`Tile.cs:623/640/657/
/// 437`) — the port's sparse `WireState` per channel (`wiring.rs`).
fn wire_at(w: &crate::World, x: i32, y: i32, color: crate::wiring::WireColor) -> bool {
    w.wires.get(x, y, color)
}

/// The four channels (`Step_WireCutter` :1467 / `Step_Actuators` :594): any
/// wire at the cell.
fn has_any_wire(w: &crate::World, x: i32, y: i32) -> bool {
    use crate::wiring::WireColor;
    wire_at(w, x, y, WireColor::Red)
        || wire_at(w, x, y, WireColor::Blue)
        || wire_at(w, x, y, WireColor::Green)
        || wire_at(w, x, y, WireColor::Yellow)
}

/// The C#'s wrench-channel number → the wire colour (the C#'s `num` 1..4
/// maps `Step_ColoredWrenches` :1931-2006 onto wire/wire2/wire3/wire4).
fn wrench_channel(num: i32) -> crate::wiring::WireColor {
    use crate::wiring::WireColor;
    match num {
        1 => WireColor::Red,
        2 => WireColor::Blue,
        3 => WireColor::Green,
        _ => WireColor::Yellow,
    }
}

/// `SmartCursorHelper.Step_WireCutter` (`:1455-1493`): the held wire cutter
/// (510) re-points at the closest wired or actuated cell.
pub fn step_wire_cutter(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:1457` — the item-kind gate + the focus sentinel.
    if l.item.kind != 510 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:1467` — any wire colour or the actuator.
            if has_any_wire(w, i, j) || t.actuator() {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_Actuators` (`:582-620`): the held actuator (849)
/// re-points at the closest active wired cell that has no actuator yet.
pub fn step_actuators(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:584` — the item-kind gate + the focus sentinel.
    if l.item.kind != 849 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:594` — wired, not actuated, active.
            if has_any_wire(w, i, j) && !t.actuator() && t.active() {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_ActuationRod` (`:1495-1535`): the held actuation
/// rod (3620) re-points at the closest actuated cell — the rod's channel
/// lock (`Player.cs:2616/:2618`, caller-passed) restricts the actuation
/// states it toggles.
pub fn step_actuation_rod(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:1497-1498` — the lock reads happen before the gate in the C# (no
    // side effects; the port keeps the order).
    let (rod_lock, rod_lock_setting) = (l.actuation_rod_lock, l.actuation_rod_lock_setting);
    // `:1500` — the item-kind gate + the focus sentinel.
    if l.item.kind != 3620 || fx != -1 || fy != -1 {
        return None;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:1509` — actuated, and the lock accepts its state.
            if t.active() && t.actuator() && (!rod_lock || rod_lock_setting == t.inactive()) {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_ColoredWrenches` (`:1879-2030`): the held
/// colour wrench (509/850/851/3612) re-points beside a cell that already
/// carries the matching wire — the line extends from its ends.
pub fn step_colored_wrenches(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:1881` — the four item kinds + the focus sentinel.
    if !(l.item.kind == 509 || l.item.kind == 850 || l.item.kind == 851 || l.item.kind == 3612)
        || fx != -1
        || fy != -1
    {
        return None;
    }
    // `:1886-1902` — the item → channel map.
    let num = match l.item.kind {
        509 => 1,
        850 => 2,
        851 => 3,
        _ => 4,
    };
    let color = wrench_channel(num);
    // `:1904-1918` — the screen target already carries the wrench's own
    // channel → the scan stands down (the C#'s four arms each test the
    // channel that matches `num`).
    let held = wire_at(w, l.screen_target.0, l.screen_target.1, color);
    let mut cands: Vec<(i32, i32)> = Vec::new();
    if !held {
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                if w.tile(i, j).is_none() {
                    continue;
                }
                // `:1924` — the cell carries the wrench's channel (the C#'s
                // four `||` arms over `!channel || num != n`).
                if !wire_at(w, i, j, color) {
                    continue;
                }
                // `:1931-2006` — the four neighbours lacking the channel
                // become the candidates (an out-of-world neighbour is
                // skipped, as the C#'s border reads never happen).
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    if !wire_at(w, i + dx, j + dy, color) {
                        cands.push((i + dx, j + dy));
                    }
                }
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

/// `SmartCursorHelper.Step_MulticolorWrench` (`:1725-1877`): the held
/// multi-wrench (3625) re-points by the selected channels
/// (`WiresUI.Settings.ToolMode`, `WiresUI.cs:23`) — the cutter arm targets
/// the wired cells themselves, the placement arm their free neighbours.
pub fn step_multicolor_wrench(
    w: &crate::World,
    l: &Lookup,
    fx: i32,
    fy: i32,
) -> Option<(i32, i32)> {
    // `:1727` — the item-kind gate + the focus sentinel.
    if l.item.kind != 3625 || fx != -1 || fy != -1 {
        return None;
    }
    // `:1732-1746` — the hovered cell's channels.
    let mut hovered: u8 = 0;
    if wire_at(w, l.screen_target.0, l.screen_target.1, wrench_channel(1)) {
        hovered |= multi_tool_mode::RED;
    }
    if wire_at(w, l.screen_target.0, l.screen_target.1, wrench_channel(2)) {
        hovered |= multi_tool_mode::BLUE;
    }
    if wire_at(w, l.screen_target.0, l.screen_target.1, wrench_channel(3)) {
        hovered |= multi_tool_mode::GREEN;
    }
    if wire_at(w, l.screen_target.0, l.screen_target.1, wrench_channel(4)) {
        hovered |= multi_tool_mode::YELLOW;
    }
    // `:1748` — the hovered cell already matches the placement channels →
    // the scan stands down (the C#'s `toolMode & ~Cutter == multiToolMode`).
    let tool_mode = l.tool_mode;
    let matches = (tool_mode & !multi_tool_mode::CUTTER) == hovered;
    let mut cands: Vec<(i32, i32)> = Vec::new();
    if !matches {
        // `:1750-1756` — the channel arms + the cutter arm.
        let (red, blue, green, yellow) = (
            tool_mode & multi_tool_mode::RED != 0,
            tool_mode & multi_tool_mode::BLUE != 0,
            tool_mode & multi_tool_mode::GREEN != 0,
            tool_mode & multi_tool_mode::YELLOW != 0,
        );
        let cutter = tool_mode & multi_tool_mode::CUTTER != 0;
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                if w.tile(i, j).is_none() {
                    continue;
                }
                // `:1764-1770` — the cutter arm: the cell itself.
                if cutter
                    && ((wire_at(w, i, j, wrench_channel(1)) && red)
                        || (wire_at(w, i, j, wrench_channel(2)) && blue)
                        || (wire_at(w, i, j, wrench_channel(3)) && green)
                        || (wire_at(w, i, j, wrench_channel(4)) && yellow))
                {
                    cands.push((i, j));
                } else {
                    // `:1773` — a selected channel must be present.
                    if !((wire_at(w, i, j, wrench_channel(1)) && red)
                        || (wire_at(w, i, j, wrench_channel(2)) && blue)
                        || (wire_at(w, i, j, wrench_channel(3)) && green)
                        || (wire_at(w, i, j, wrench_channel(4)) && yellow))
                    {
                        continue;
                    }
                    // `:1776-1849` — the free neighbours per selected
                    // channel (the C#'s non-short-circuit bool `&` reads as
                    // `&&` over pure operands).
                    for channel in [1, 2, 3, 4] {
                        let on = match channel {
                            1 => red,
                            2 => blue,
                            3 => green,
                            _ => yellow,
                        };
                        if on {
                            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                if !wire_at(w, i + dx, j + dy, wrench_channel(channel)) {
                                    cands.push((i + dx, j + dy));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// ------------------------------------------------------------ track step --

/// `TileID.MinecartTrack` (`TileID.cs` — the C# reads the literal 314 in
/// `Step_MinecartTracks`).
const TRACK_TILE: u16 = 314;

/// The cell is a minecart track.
fn is_track(w: &crate::World, x: i32, y: i32) -> bool {
    w.tile(x, y)
        .is_some_and(|t| t.active() && t.kind == TRACK_TILE)
}

/// `Entity.Center` (`Entity.cs:50-54`): the player's box centre —
/// `position + (width/2f, height/2f)` with the FLOAT halves.
fn player_center(l: &Lookup) -> (f32, f32) {
    (
        l.player_position.0 + (l.player_width as f32) / 2.0,
        l.player_position.1 + (l.player_height as f32) / 2.0,
    )
}

/// `Vector2.SafeNormalize` (`Utils.cs:1837-1844`): the unit vector, or the
/// default for a zero/NaN input.
fn safe_normalize(v: (f32, f32), default: (f32, f32)) -> (f32, f32) {
    let len = v.0.hypot(v.1);
    if len == 0.0 || !v.0.is_finite() || !v.1.is_finite() {
        return default;
    }
    (v.0 / len, v.1 / len)
}

/// The C#'s "replaceable cell" predicate the track arms read
/// (`!active() || tileCut || BreakableWhenPlacing`).
fn replaceable_cell(w: &crate::World, x: i32, y: i32) -> bool {
    w.tile(x, y).is_some_and(|t| {
        !t.active()
            || crate::tileobject::tile_cut(t.kind)
            || crate::tileobject::breakable_when_placing(t.kind)
    })
}

/// The C#'s track pick filter (`:1250-1262`, :1301-1313`): the candidate
/// stands freestanding — no track directly above or below it.
fn freestanding_track(w: &crate::World, x: i32, y: i32) -> bool {
    !is_track(w, x, y - 1) && !is_track(w, x, y + 1)
}

/// The shared track vote (`:1244-1266` and `:1299-1321`): the closest
/// freestanding candidate to the mouse, with the `num != -1f` commit guard.
fn pick_track(cands: &[(i32, i32)], w: &crate::World, l: &Lookup) -> Option<(i32, i32)> {
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = -1.0f32;
    for &(x, y) in cands {
        if freestanding_track(w, x, y) {
            let d = dist_to_mouse(x, y, l.mouse);
            if best_dist == -1.0 || d < best_dist {
                best_dist = d;
                best = Some((x, y));
            }
        }
    }
    best
}

/// `SmartCursorHelper.Step_MinecartTracks` (`:1166-1322`): the held
/// minecart-track item (2340/2739) re-points in the mouse's direction from
/// the player's centre, extending the existing track; the pressure track
/// (2492) extends the line sideways only.
pub fn step_minecart_tracks(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:1168` — the minecart-track block's gate: the item kinds + an empty
    // focus (the C#'s `if`, not an early return — the pressure-track block
    // below still runs).
    if (l.item.kind == 2340 || l.item.kind == 2739) && fx == -1 && fy == -1 {
        // `:1170-1197` — the mouse's bearing from the player's centre, the
        // eight direction flags and the diagonal dominance clearing the
        // axis flags (the C#'s `&` on bools is the non-short-circuit AND
        // over pure operands).
        let center = player_center(l);
        let dir = safe_normalize((l.mouse.0 - center.0, l.mouse.1 - center.1), (0.0, 1.0));
        let num = -dir.1; // `Vector2.Dot(value, -Vector2.UnitY)`
        let (up, down) = (num >= 0.5, num <= -0.5);
        let num2 = dir.0; // `Vector2.Dot(value, Vector2.UnitX)`
        let (right, left) = (num2 >= 0.5, num2 <= -0.5);
        let (up_left, up_right, down_left, down_right) =
            (up && left, up && right, down && left, down && right);
        let (mut left, mut right) = (left, right);
        if up_left {
            left = false;
        }
        if up_right {
            right = false;
        }
        if down_left {
            left = false;
        }
        if down_right {
            right = false;
        }
        // `:1199-1203` — the screen target already a track → the scan
        // stands down.
        let mut held = false;
        if is_track(w, l.screen_target.0, l.screen_target.1) {
            held = true;
        }
        let mut cands: Vec<(i32, i32)> = Vec::new();
        if !held {
            for i in l.start_x..=l.end_x {
                for j in l.start_y..=l.end_y {
                    if !is_track(w, i, j) {
                        continue;
                    }
                    // `:1210-1213` — the four diagonal track presences.
                    let down_right_present = is_track(w, i + 1, j + 1);
                    let up_right_present = is_track(w, i + 1, j - 1);
                    let down_left_present = is_track(w, i - 1, j + 1);
                    let up_left_present = is_track(w, i - 1, j - 1);
                    // `:1215-1239` — the six directional adds. The diagonal
                    // arms carry the De Morgan guards (`!(!a & b) && !c`);
                    // an out-of-world neighbour reads as the C#'s default
                    // (inactive, not a track) — the port's edge convention.
                    if up_left
                        && replaceable_cell(w, i - 1, j - 1)
                        && !(!down_right_present && up_right_present)
                        && !down_left_present
                    {
                        cands.push((i - 1, j - 1));
                    }
                    if left && replaceable_cell(w, i - 1, j) {
                        cands.push((i - 1, j));
                    }
                    if down_left
                        && replaceable_cell(w, i - 1, j + 1)
                        && !(!up_right_present && down_right_present)
                        && !up_left_present
                    {
                        cands.push((i - 1, j + 1));
                    }
                    if up_right
                        && replaceable_cell(w, i + 1, j - 1)
                        && !(!down_left_present && up_left_present)
                        && !down_right_present
                    {
                        cands.push((i + 1, j - 1));
                    }
                    if right && replaceable_cell(w, i + 1, j) {
                        cands.push((i + 1, j));
                    }
                    if down_right
                        && replaceable_cell(w, i + 1, j + 1)
                        && !(!up_left_present && down_left_present)
                        && !up_right_present
                    {
                        cands.push((i + 1, j + 1));
                    }
                }
            }
        }
        // `:1244-1266` — the freestanding vote + the double commit guard.
        if let Some(p) = pick_track(&cands, w, l) {
            if in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y) {
                return Some(p);
            }
        }
    }
    // `:1268` — the pressure track's gate.
    if l.item.kind != 2492 || fx != -1 || fy != -1 {
        return None;
    }
    // `:1274-1279` — the screen-target pre-check.
    let mut held = false;
    if is_track(w, l.screen_target.0, l.screen_target.1) {
        held = true;
    }
    let mut cands: Vec<(i32, i32)> = Vec::new();
    if !held {
        for i in l.start_x..=l.end_x {
            for j in l.start_y..=l.end_y {
                if !is_track(w, i, j) {
                    continue;
                }
                // `:1285-1294` — the line extends left/right only.
                if replaceable_cell(w, i - 1, j) {
                    cands.push((i - 1, j));
                }
                if replaceable_cell(w, i + 1, j) {
                    cands.push((i + 1, j));
                }
            }
        }
    }
    // `:1296-1321` — the same freestanding vote + commit guards.
    let p = pick_track(&cands, w, l)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// ------------------------------------------------------- force-cursor step --

/// `SmartCursorHelper.Step_ForceCursorToAnyMinableThing` (`:2229-2268`):
/// when the tile under the mouse is minable with the held item, the cursor
/// is forced onto it — the step writes the mouse's own tile, not a scan.
pub fn step_force_cursor(w: &crate::World, l: &Lookup, fx: i32, fy: i32) -> Option<(i32, i32)> {
    // `:2236-2239` — the local aliases + the C#'s two decompiler artifacts
    // (`_ = screenTargetX/Y`, no side effects) are dropped.
    // `:2241` — the focus sentinel + the gamepad arm (input is app-side,
    // caller-passed).
    if fx != -1 || fy != -1 || l.using_gamepad {
        return None;
    }
    // `:2243-2245` — the mouse's tile (`Utils.ToTileCoordinates`' `>> 4`
    // shift — the arithmetic shift floors, matching the C#'s int shift).
    let (x, y) = ((l.mouse.0 as i32) >> 4, (l.mouse.1 as i32) >> 4);
    if !in_tile_bounds(x, y, l.start_x, l.start_y, l.end_x, l.end_y) {
        return None;
    }
    let Some(t) = w.tile(x, y) else {
        return None;
    };
    // `:2249` — the minable non-full-solid half: a tile the world lets the
    // player kill and that is not a solid (or is a solidTop). The chest
    // arm's store is app-side (`edit::can_kill_tile`'s closure) — the
    // sources without one pass `false`, as the actuation slice does.
    let mut flag = t.active()
        && crate::edit::can_kill_tile(w, x, y, |_, _| false)
        && (!crate::tiledata::TILE_SOLID.has(t.kind)
            || crate::tiledata::TILE_SOLID_TOP.has(t.kind));
    // `:2250-2261` — the three tool demotions: an axe tile needs the axe,
    // a hammer tile the hammer, everything else the pickaxe.
    if flag && crate::tileobject::tile_axe(t.kind) && l.item.axe < 1 {
        flag = false;
    }
    if flag && crate::tileobject::tile_hammer(t.kind) && l.item.hammer < 1 {
        flag = false;
    }
    if flag
        && !crate::tileobject::tile_hammer(t.kind)
        && !crate::tileobject::tile_axe(t.kind)
        && l.item.pick < 1
    {
        flag = false;
    }
    if flag {
        return Some((x, y));
    }
    None
}

// ------------------------------------------------------------- shinies step --

/// `TileID.Sets.SmartCursorPickaxePriorityOverride` (`TileID.cs:507`,
/// `CreateIntSet(0, 12 → 1, 665 → 1, 639 → 1)`): the per-tile priority the
/// pickaxe's shiny scan tracks — the C# is an `int[]` over `TileID.Count`,
/// the port a per-id map.
const SMART_CURSOR_PICKAXE_PRIORITY: [(u16, i32); 3] = [(12, 1), (665, 1), (639, 1)];

/// The tile's priority override (0 when the set does not carry it — the
/// C#'s array default).
fn pickaxe_priority_override(kind: u16) -> i32 {
    SMART_CURSOR_PICKAXE_PRIORITY
        .iter()
        .find(|(id, _)| *id == kind)
        .map(|(_, v)| *v)
        .unwrap_or(0)
}

/// `SmartCursorHelper.Step_Pickaxe_MineShinies` (`:2270-2348`): the held
/// pickaxe re-points at the closest rare tile the priority override flags.
pub fn step_pickaxe_mine_shinies(
    w: &crate::World,
    l: &Lookup,
    fx: i32,
    fy: i32,
) -> Option<(i32, i32)> {
    // `:2283` — the pick-power gate + the focus sentinel.
    if l.item.pick <= 0 || fx != -1 || fy != -1 {
        return None;
    }
    // `:2285-2290` — the C#'s `if (item.type != 1333 && ...)` arm is a
    // decompiler artifact with no side effects (dropped).
    let mut cands: Vec<(i32, i32)> = Vec::new();
    for i in l.start_x..=l.end_x {
        for j in l.start_y..=l.end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:2295-2297` — the C#'s `_ = Main.tile[i±1, j]; _ = Main.tile[i, j+1];`
            // reads are decompiler artifacts with no side effects (dropped).
            if !t.active() {
                continue;
            }
            // `:2298-2312` — every override-carrying tile is a candidate.
            // The C# also tracks the maximum priority (`num`), but the only
            // consumer of it — the two `_targets2` loops at :2315-2327 —
            // iterates the just-cleared list, so both are provably empty in
            // vanilla and the priority filter is inert. The port implements
            // the OBSERVABLE behaviour (the closest of all override tiles),
            // not the intended one, and does not emulate C#'s
            // foreach-mutation hazard.
            if pickaxe_priority_override(t.kind) > 0 {
                cands.push((i, j));
            }
        }
    }
    let p = pick_closest(&cands, l.mouse)?;
    in_tile_bounds(p.0, p.1, l.start_x, l.start_y, l.end_x, l.end_y).then_some(p)
}

// ------------------------------------------------------------ the dispatch --

/// `SmartCursorHelper.SmartCursorLookup` (`:61-165`) restricted to the
/// ported steps, run in the vanilla chain's order (`:118-156`):
/// ForceCursorToAnyMinableThing :122 → PickaxeMineShinies :123 →
/// ColoredWrenches :129 → MulticolorWrench :130 → ActuationRod :132 →
/// WireCutter :133 →
/// Platforms :134 → MinecartTracks :135 → Walls :136 → PumpkinSeeds :137
/// → GrassSeeds :138 → Moss :139 → Pigronata :140 → Torch :142 →
/// LawnMower :143 → BlocksLines :145 → PaintRoller :146 → PaintBrush :147
/// → PaintScrapper :148 → Acorns :149 → GemCorns :150 → EmptyBuckets :151
/// → Actuators :152 → AlchemySeeds :153 → PlanterBox :154 → ClayPots :155
/// → StaffOfRegrowth :156. Each step's gate refuses a taken focus, so the
/// first step that finds a spot wins — the C# straight-line chain.
///
/// The interactable-block gate (`IsHoveringOverAnInteractableTileThatBlocksSmartCursor`,
/// :193 — the port has no sign/chest hover routing yet) stays a no-op that
/// always passes, and the rect clamps stay the caller's (the C# clamps in
/// `GetTileRegion`, `TileReachCheckSettings.cs:45-52`).
///
/// Returns the re-pointed tile or `None` (the C# writeback
/// `Main.SmartCursorX/Y = fX/fY`, :157-162, stands when nothing moved).
pub fn smart_cursor_lookup(
    w: &crate::World,
    l: &Lookup,
    fx: i32,
    fy: i32,
    state: &mut ContinuityState,
) -> Option<(i32, i32)> {
    // `:66-67` — the continuity-lock reset: the locks survive only while the
    // use button is held, the smart cursor is on and the held item places a
    // platform (`UsingSmartCursorWithContinuity`, :167-175). The C#'s
    // `Main.SmartCursorShowing = false` (:63) stays the caller's flag.
    if !l.use_item_held
        || !SMART_CURSOR_IS_USED
        || !using_smart_cursor_with_continuity(l.item.create_tile)
    {
        *state = ContinuityState::default();
    }
    if !SMART_CURSOR_IS_USED {
        return None;
    }
    let mut focus = (fx, fy);
    // `:134` — the platform step runs first in the C#'s chain and is the
    // only step that writes the continuity state.
    if let Some(p) = step_platforms(w, l, focus.0, focus.1, state) {
        focus = p;
    }
    let steps: [fn(&crate::World, &Lookup, i32, i32) -> Option<(i32, i32)>; 26] = [
        step_force_cursor,
        step_pickaxe_mine_shinies,
        step_colored_wrenches,
        step_multicolor_wrench,
        step_actuation_rod,
        step_wire_cutter,
        step_minecart_tracks,
        step_walls,
        step_pumpkin_seeds,
        step_grass_seeds,
        step_moss,
        step_pigronata,
        step_torch,
        step_lawn_mower,
        step_blocks_lines,
        step_paint_roller,
        step_paint_brush,
        step_paint_scrapper,
        step_acorns,
        step_gem_corns,
        step_empty_buckets,
        step_actuators,
        step_alchemy_seeds,
        step_planter_box,
        step_clay_pots,
        step_staff_of_regrowth,
    ];
    for step in steps {
        if let Some(p) = step(w, l, focus.0, focus.1) {
            focus = p;
        }
    }
    (focus != (fx, fy)).then_some(focus)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small world with a floor row at y = 20 (the `object_world`-style
    /// fixture).
    fn w() -> crate::World {
        let mut w = crate::World::new("t".into(), 60, 40);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 0..60 {
            let i = w.index(x, 20);
            w.tiles[i] = crate::tile::Tile {
                kind: 1,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w
    }

    fn put(w: &mut crate::World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = crate::tile::Tile {
            kind,
            flags: crate::tile::FLAG_ACTIVE,
            ..Default::default()
        };
    }

    /// A full solid row (the fixtures' ground).
    fn put_row(w: &mut crate::World, x0: i32, x1: i32, y: i32, kind: u16) {
        for x in x0..=x1 {
            put(w, x, y, kind);
        }
    }

    /// The tile paint on the cell (`Tile::set_paint`, `tile.rs:52`).
    fn paint_at(w: &mut crate::World, x: i32, y: i32, paint: u8) {
        let i = w.index(x, y);
        w.tiles[i].set_paint(paint);
    }

    /// A wall behind the cell (`Tile.wall`, the WallID — the wall's paint
    /// and coatings ride `World::extras`).
    fn wall_at(w: &mut crate::World, x: i32, y: i32, wall: u16) {
        let i = w.index(x, y);
        w.tiles[i].wall = wall;
    }

    /// The tile's Glow-coating (fullbright) flag.
    fn fullbright_at(w: &mut crate::World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i].flags |= crate::tile::FLAG_FULLBRIGHT;
    }

    /// A bare world — no fixture floor (the steps whose box sweeps need
    /// the rows above their seat clear start from it).
    fn bare() -> crate::World {
        let mut w = crate::World::new("b".into(), 60, 40);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w
    }

    /// The mouse at a tile's centre (`X*16+8, Y*16+8`) — the tie-free
    /// vote position (an off-centre mouse ties the neighbours and the
    /// C#'s strict `<` keeps the first candidate).
    fn mouse_at(x: i32, y: i32) -> (f32, f32) {
        (x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0)
    }

    /// The default lookup inputs: the reachable rect 20..=40 × 10..=19,
    /// the mouse over the floor row, the screen target on the floor.
    fn lookup(item: ItemPort, mouse: (f32, f32)) -> Lookup {
        Lookup {
            item,
            mouse,
            screen_target: (30, 19),
            start_x: 20,
            start_y: 10,
            end_x: 40,
            end_y: 19,
            weather: crate::growth::WeatherCtx::default(),
            paint_lookup: 0,
            paint_coating_lookup: 0,
            smart_blocks_enabled: false,
            use_item_held: false,
            tool_mode: 0,
            actuation_rod_lock: false,
            actuation_rod_lock_setting: false,
            using_gamepad: false,
            player_position: (30.0 * 16.0, 21.0 * 16.0),
            player_width: 20,
            player_height: 42,
        }
    }

    fn item(kind: u16) -> ItemPort {
        ItemPort {
            kind,
            create_tile: -1,
            create_wall: -1,
            place_style: 0,
            pick: 0,
            axe: 0,
            hammer: 0,
        }
    }

    /// A tool item with the given pickaxe power.
    fn pickaxe(kind: u16, pick: i32) -> ItemPort {
        ItemPort {
            kind,
            create_tile: -1,
            create_wall: -1,
            place_style: 0,
            pick,
            axe: 0,
            hammer: 0,
        }
    }

    fn placeable(kind: u16, create_tile: i32) -> ItemPort {
        ItemPort {
            kind,
            create_tile,
            create_wall: -1,
            place_style: 0,
            pick: 0,
            axe: 0,
            hammer: 0,
        }
    }

    /// A wall-placing item (`Item.createWall > 0`).
    fn placeable_wall(kind: u16, create_wall: i32) -> ItemPort {
        ItemPort {
            kind,
            create_tile: -1,
            create_wall,
            place_style: 0,
            pick: 0,
            axe: 0,
            hammer: 0,
        }
    }

    #[test]
    fn test_smart_1_a_torch_item_finds_the_floor_spot() {
        // `Step_Torch`: the held torch (item 8, in `ItemID.Sets.Torches`)
        // re-points at the closest empty cell above the floor row
        // (`tileSolid && !tileSolidTop` below).
        let mut world = w();
        put(&mut world, 30, 19, 1); // a solid block: not a torch spot
        let l = lookup(item(8), (30.0 * 16.0, 19.0 * 16.0));
        let found = step_torch(&world, &l, -1, -1);
        let (fx, fy) = found.expect("the torch finds a spot");
        assert!(in_tile_bounds(fx, fy, 20, 10, 40, 19));
        // The spot must be empty and anchored to the floor row.
        let spot = world.tile(fx, fy).unwrap();
        assert!(!spot.active());
        assert!(world.tile(fx, fy + 1).is_some_and(|t| t.active()));
    }

    #[test]
    fn test_smart_1_a_non_torch_item_skips_the_step() {
        let world = w();
        let l = lookup(item(1), (0.0, 0.0));
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            None,
            "the pickaxe's lookup rides a later step"
        );
    }

    #[test]
    fn test_smart_1_an_existing_torch_blocks_the_box() {
        // The ±8 sweep: an existing torch within 8 tiles refuses the whole
        // box.
        let mut world = w();
        put(&mut world, 30, 19, TORCH_TILE);
        let l = lookup(item(8), (30.0 * 16.0, 19.0 * 16.0));
        // The rect 25..=35 × 15..=19 keeps the torch inside the ±8 sweep.
        let l = Lookup {
            start_x: 25,
            start_y: 15,
            end_x: 35,
            end_y: 19,
            ..l
        };
        assert_eq!(
            step_torch(&world, &l, -1, -1),
            None,
            "the no-torch-around sweep covers the reachable rect"
        );
    }

    #[test]
    fn test_smart_1_a_filled_cell_is_skipped() {
        // The occupied non-cut cell: solid stones in the rect leave only
        // the free row.
        let mut world = w();
        for x in 20..=40 {
            for y in 15..=19 {
                put(&mut world, x, y, 1);
            }
        }
        // The rect 20..=40 × 15..=19 is solid, but the reachable window
        // extends above it (y = 10..19): the empty rows 10-14 still offer
        // spots — the closest valid one is picked (the vanilla scans the
        // whole reachable rect, not just the filled pocket).
        let l = lookup(item(8), (30.0 * 16.0, 19.0 * 16.0));
        let found = step_torch(&world, &l, -1, -1);
        let (fx, fy) = found.expect("the empty rows above the pocket are spots");
        assert!(fy < 15, "the picked spot sits above the filled block: {fy}");
    }

    #[test]
    fn test_smart_1_the_wall_behind_the_cell_anchors_a_torch() {
        // `IsValidSpotForTorch`'s first arm: a wall behind the cell accepts
        // the torch even with no side/floor anchor.
        let mut world = crate::World::new("w".into(), 20, 20);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        let i = world.index(10, 10);
        world.tiles[i].wall = 1; // the dirt wall
        assert!(is_valid_spot_for_torch(&world, 10, 10, None));
        assert!(!is_valid_spot_for_torch(&world, 10, 11, None));
    }

    #[test]
    fn test_smart_1_the_regrowth_staff_finds_the_open_dirt() {
        // `Step_StaffOfRegrowth`: the staff (213) re-points at the dirt
        // tile with an open orthogonal neighbour.
        let mut world = w();
        put(&mut world, 25, 19, 0); // dirt above the floor, open on top
        let l = lookup(item(213), mouse_at(25, 19));
        let found = step_staff_of_regrowth(&world, &l, -1, -1);
        assert_eq!(found, Some((25, 19)), "the staff targets the open dirt");

        // A closed pocket (all eight neighbours solid) is not a target.
        let mut closed = w();
        put(&mut closed, 25, 19, 0);
        for (dx, dy) in [
            (-1, 0),
            (1, 0),
            (0, 1),
            (0, -1),
            (-1, -1),
            (-1, 1),
            (1, 1),
            (1, -1),
        ] {
            put(&mut closed, 25 + dx, 19 + dy, 1);
        }
        let l2 = lookup(item(213), mouse_at(25, 19));
        assert_eq!(
            step_staff_of_regrowth(&closed, &l2, -1, -1),
            None,
            "a fully surrounded dirt is not a staff target"
        );

        // The item-kind gate: a pickaxe skips the step.
        assert_eq!(
            step_staff_of_regrowth(&world, &lookup(item(1), (0.0, 0.0)), -1, -1),
            None,
            "the staff gate is item 213/5295 only"
        );
    }

    #[test]
    fn test_smart_1_grass_seeds_find_the_open_dirt() {
        // `Step_GrassSeeds`: the plain grass seeds (62) plant on dirt (0).
        let mut world = w();
        put(&mut world, 25, 19, 0);
        let l = lookup(item(62), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_grass_seeds(&world, &l, -1, -1),
            Some((25, 19)),
            "the grass seed re-points at the open dirt"
        );
        // A stone cell is not a grass-seed target (the default arm wants 0).
        let mut stone = w();
        put(&mut stone, 25, 19, 1);
        let l2 = lookup(item(62), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_grass_seeds(&stone, &l2, -1, -1),
            None,
            "the default arm only accepts dirt"
        );
    }

    #[test]
    fn test_smart_1_mushroom_seeds_want_corrupt_grass() {
        // `Step_GrassSeeds` :299 — the mushroom grass seeds (194) only
        // plant on corrupt grass (59), unlike the plain seeds.
        let mut world = w();
        put(&mut world, 25, 19, 59); // corrupt grass, open on top
        let l = lookup(item(194), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_grass_seeds(&world, &l, -1, -1),
            Some((25, 19)),
            "the mushroom seed re-points at corrupt grass"
        );
        // The same world with the plain seeds finds nothing (dirt only).
        let l2 = lookup(item(62), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_grass_seeds(&world, &l2, -1, -1),
            None,
            "plain seeds do not target corrupt grass"
        );
    }

    #[test]
    fn test_smart_1_moss_wants_stone() {
        // `Step_Moss` :353 — the green moss (4349) spreads onto stone (1)
        // or pearlstone (38), never dirt.
        let mut world = w();
        put(&mut world, 25, 19, 1); // stone with an open top
        let l = lookup(item(4349), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_moss(&world, &l, -1, -1),
            Some((25, 19)),
            "the moss re-points at the open stone"
        );
        let mut dirt = w();
        put(&mut dirt, 25, 19, 0);
        let l2 = lookup(item(4349), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_moss(&dirt, &l2, -1, -1),
            None,
            "dirt is not a moss target"
        );
    }

    #[test]
    fn test_smart_1_clay_pots_sit_on_the_floor() {
        // `Step_ClayPots`: the clay pot (createTile 78) re-points at the
        // empty cell above the solid floor (`Main.tileSolid` — a platform
        // row seats a pot too, the C# checks the full solid set).
        let world = w();
        let l = lookup(placeable(0, 78), mouse_at(25, 19));
        let found = step_clay_pots(&world, &l, -1, -1);
        assert_eq!(found, Some((25, 19)), "the pot sits right above the floor");
        // The below-cell must be solid: a sapling row (20 — active but not
        // `tileSolid`) refuses.
        let mut sapling = w();
        put_row(&mut sapling, 20, 40, 20, 20);
        let l2 = lookup(placeable(0, 78), mouse_at(25, 19));
        assert_eq!(
            step_clay_pots(&sapling, &l2, -1, -1),
            None,
            "a non-solid seat is not a pot seat"
        );
    }

    #[test]
    fn test_smart_1_clay_pots_skip_an_occupied_screen_target() {
        // `Step_ClayPots` :389-396 — an occupied in-rect screen target
        // disables the scan.
        let mut world = w();
        put(&mut world, 30, 19, 1);
        let l = Lookup {
            screen_target: (30, 19),
            ..lookup(placeable(0, 78), (25.0 * 16.0, 19.0 * 16.0))
        };
        assert_eq!(
            step_clay_pots(&world, &l, -1, -1),
            None,
            "the occupied screen target disables the pot scan"
        );
    }

    #[test]
    fn test_smart_1_planter_boxes_extend_sideways() {
        // `Step_PlanterBox`: the planter box (createTile 380) re-points
        // beside an existing box.
        let mut world = w();
        put(&mut world, 25, 19, 380); // an existing planter box
        let l = lookup(placeable(0, 380), (26.0 * 16.0, 19.0 * 16.0));
        let found = step_planter_box(&world, &l, -1, -1);
        assert_eq!(
            found,
            Some((26, 19)),
            "the new box extends the row to the right"
        );
        // Without an existing box there is nothing to extend.
        let fresh = w();
        let l2 = lookup(placeable(0, 380), (26.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_planter_box(&fresh, &l2, -1, -1),
            None,
            "the box step needs an existing box to extend"
        );
    }

    #[test]
    fn test_smart_1_alchemy_seeds_respect_the_style_ground() {
        // `Step_AlchemySeeds` :509-556 — style 0 (daybloom) accepts the
        // stone floor, style 1 (moonglow) refuses jungle grass (60).
        let mut jungle = w();
        put_row(&mut jungle, 20, 40, 20, 60);
        let l0 = lookup(
            ItemPort {
                kind: 0,
                create_tile: 82,
                create_wall: -1,
                pick: 0,
                axe: 0,
                hammer: 0,
                place_style: 0,
            },
            mouse_at(25, 19),
        );
        assert_eq!(
            step_alchemy_seeds(&jungle, &l0, -1, -1),
            Some((25, 19)),
            "style 0 plants on jungle grass"
        );
        let l1 = lookup(
            ItemPort {
                kind: 0,
                create_tile: 82,
                create_wall: -1,
                pick: 0,
                axe: 0,
                hammer: 0,
                place_style: 1,
            },
            mouse_at(25, 19),
        );
        assert_eq!(
            step_alchemy_seeds(&jungle, &l1, -1, -1),
            None,
            "style 1 refuses jungle grass ground"
        );
        // The same style 1 on stone plants.
        let stone = w();
        assert_eq!(
            step_alchemy_seeds(&stone, &l1, -1, -1),
            Some((25, 19)),
            "style 1 plants on stone"
        );
    }

    #[test]
    fn test_smart_1_pigronata_sits_on_the_surface() {
        // `Step_Pigronata`: the statue (createTile 454) needs a solid row
        // below and clear space above, above the surface gate. A bare
        // world — the 4x4 box sweeps the rows j-1..j+2 and the fixture
        // floor would block the top rows.
        let mut world = bare();
        world.world_surface = 25.0; // the surface gate allows j <= 23
        put_row(&mut world, 20, 40, 18, 1); // the solid row below the spot
        let l = lookup(placeable(0, 454), mouse_at(25, 19));
        let found = step_pigronata(&world, &l, -1, -1);
        assert_eq!(found, Some((25, 19)), "the statue sits above the solid row");
        // Deep below the surface the step refuses.
        let mut deep = bare();
        deep.world_surface = 10.0; // the gate allows j <= 8 only
        put_row(&mut deep, 20, 40, 18, 1);
        let l2 = lookup(placeable(0, 454), mouse_at(25, 19));
        assert_eq!(
            step_pigronata(&deep, &l2, -1, -1),
            None,
            "the surface gate stops the scan below the surface"
        );
    }

    #[test]
    fn test_smart_1_pumpkin_seeds_need_grass_ground() {
        // `Step_PumpkinSeeds`: the seed (createTile 254) re-points at the
        // 2-wide grass-family ground with clear headers.
        let mut world = w();
        put_row(&mut world, 20, 40, 20, 2); // grass ground
        world.world_surface = 25.0;
        let l = lookup(placeable(0, 254), mouse_at(25, 19));
        assert_eq!(
            step_pumpkin_seeds(&world, &l, -1, -1),
            Some((25, 19)),
            "the pumpkin seed re-points above the grass row"
        );
        // Stone ground is not a pumpkin bed.
        let stone = w();
        let l2 = lookup(placeable(0, 254), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_pumpkin_seeds(&stone, &l2, -1, -1),
            None,
            "the pumpkin seed needs grass-family ground"
        );
    }

    #[test]
    fn test_smart_1_the_lawn_mower_targets_grass() {
        // `Step_LawnMower`: the mower (4049) re-points at grass (2) or
        // moss (109).
        let mut world = w();
        put(&mut world, 25, 19, 2); // grass with open space around
        let l = lookup(item(4049), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_lawn_mower(&world, &l, -1, -1),
            Some((25, 19)),
            "the mower re-points at the grass"
        );
        // Dirt is not a mow target.
        let mut dirt = w();
        put(&mut dirt, 25, 19, 0);
        let l2 = lookup(item(4049), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            step_lawn_mower(&dirt, &l2, -1, -1),
            None,
            "dirt is not a mow target"
        );
    }

    #[test]
    fn test_smart_1_the_seed_family_chain_order() {
        // The dispatch chain: with a held grass seed the seed step wins
        // over the torch step that would also match a torch-free rect.
        let mut world = w();
        put(&mut world, 25, 19, 0); // open dirt: the seed step's target
        let l = lookup(item(62), (25.0 * 16.0, 19.0 * 16.0));
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the grass-seed step runs before the torch step"
        );
    }

    #[test]
    fn test_smart_1_the_paint_scrapper_finds_painted_tiles() {
        // `Step_PaintScrapper` :692-697 — a painted tile, a LongMoss tile
        // (184) and a coated wall are scraper targets; clean stone is not.
        let mut world = w();
        put(&mut world, 25, 19, 1);
        paint_at(&mut world, 25, 19, 3);
        let l = lookup(item(1100), mouse_at(25, 19));
        assert_eq!(
            step_paint_scrapper(&world, &l, -1, -1),
            Some((25, 19)),
            "the scraper snaps to the painted tile"
        );
        let mut moss = w();
        put(&mut moss, 25, 19, 184); // LongMoss: the scraper's second target
        let l2 = lookup(item(1100), mouse_at(25, 19));
        assert_eq!(
            step_paint_scrapper(&moss, &l2, -1, -1),
            Some((25, 19)),
            "the scraper snaps to LongMoss (Player.cs:41822)"
        );
        // A coated wall behind an empty cell is a target too.
        let mut coated = w();
        wall_at(&mut coated, 25, 19, 1);
        coated.set_fullbright_wall(25, 19, true);
        let l3 = lookup(item(1100), mouse_at(25, 19));
        assert_eq!(
            step_paint_scrapper(&coated, &l3, -1, -1),
            Some((25, 19)),
            "the scraper snaps to the coated wall"
        );
        // Clean stone with no wall is not a target.
        let clean = w();
        let l4 = lookup(item(1100), mouse_at(25, 19));
        assert_eq!(
            step_paint_scrapper(&clean, &l4, -1, -1),
            None,
            "a clean stone is not a scraper target"
        );
    }

    #[test]
    fn test_smart_1_the_paint_brush_finds_the_unpainted_tile() {
        // `Step_PaintBrush` :743 — with paint 3 selected the unpainted
        // stone is the target and the already-painted one is skipped.
        let mut world = w();
        put(&mut world, 25, 19, 1);
        let l = Lookup {
            paint_lookup: 3,
            ..lookup(item(1071), mouse_at(25, 19))
        };
        assert_eq!(
            step_paint_brush(&world, &l, -1, -1),
            Some((25, 19)),
            "the brush snaps to the unpainted stone"
        );
        let mut painted = w();
        put(&mut painted, 25, 19, 1);
        paint_at(&mut painted, 25, 19, 3);
        let l2 = Lookup {
            paint_lookup: 3,
            ..lookup(item(1071), mouse_at(25, 19))
        };
        assert_eq!(
            step_paint_brush(&painted, &l2, -1, -1),
            None,
            "the tile already carries the paint"
        );
        // No paint or coating selected — the step no-ops (:729).
        let l3 = lookup(item(1071), mouse_at(25, 19));
        assert_eq!(
            step_paint_brush(&world, &l3, -1, -1),
            None,
            "the brush needs a selected paint or coating"
        );
    }

    #[test]
    fn test_smart_1_the_brush_coating_arm_wants_the_glow() {
        // `Step_PaintBrush` :743's coating arms — Glow (coating 1) skips a
        // fullbright block, Echo (coating 2) skips an invisible block.
        let mut world = w();
        put(&mut world, 25, 19, 1);
        fullbright_at(&mut world, 25, 19);
        put(&mut world, 26, 19, 1); // the plain neighbour
        let glow = Lookup {
            paint_coating_lookup: 1,
            ..lookup(item(1543), mouse_at(26, 19)) // the Spectre brush
        };
        assert_eq!(
            step_paint_brush(&world, &glow, -1, -1),
            Some((26, 19)),
            "the Glow brush skips the fullbright block"
        );
        let mut echo = w();
        put(&mut echo, 25, 19, 1);
        echo.set_invisible_block(25, 19, true);
        put(&mut echo, 26, 19, 1);
        let echo_l = Lookup {
            paint_coating_lookup: 2,
            ..lookup(item(1071), mouse_at(26, 19))
        };
        assert_eq!(
            step_paint_brush(&echo, &echo_l, -1, -1),
            Some((26, 19)),
            "the Echo brush skips the invisible block"
        );
    }

    #[test]
    fn test_smart_1_the_paint_roller_finds_the_unpainted_wall() {
        // `Step_PaintRoller` :786 — a wall behind an empty cell is the
        // target; the already-painted wall and the wall behind a solid
        // block are not.
        let mut world = w();
        wall_at(&mut world, 25, 19, 1);
        let l = Lookup {
            paint_lookup: 4,
            ..lookup(item(1072), mouse_at(25, 19))
        };
        assert_eq!(
            step_paint_roller(&world, &l, -1, -1),
            Some((25, 19)),
            "the roller snaps to the unpainted wall"
        );
        let mut painted = w();
        wall_at(&mut painted, 25, 19, 1);
        painted.set_wall_paint(25, 19, 4);
        let l2 = Lookup {
            paint_lookup: 4,
            ..lookup(item(1072), mouse_at(25, 19))
        };
        assert_eq!(
            step_paint_roller(&painted, &l2, -1, -1),
            None,
            "the wall already carries the paint"
        );
        // The wall behind a full solid cell is not a roller target.
        let mut solid = w();
        put(&mut solid, 25, 19, 1);
        wall_at(&mut solid, 25, 19, 1);
        let l3 = Lookup {
            paint_lookup: 4,
            ..lookup(item(1072), mouse_at(25, 19))
        };
        assert_eq!(
            step_paint_roller(&solid, &l3, -1, -1),
            None,
            "the wall behind a solid block is not a target"
        );
    }

    #[test]
    fn test_smart_1_the_paint_steps_chain_after_the_mower() {
        // The dispatch order (:146-148): with a held scraper the paint
        // family runs after the mower and before the alch seeds.
        let mut world = w();
        put(&mut world, 25, 19, 1);
        paint_at(&mut world, 25, 19, 3);
        let l = lookup(item(1545), mouse_at(25, 19)); // the Spectre scraper
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the scraper step runs inside the lookup chain"
        );
    }

    /// An acorn lookup over the fixture's rows 21..=30 (the step's height
    /// gate refuses `startY <= 20`).
    fn acorn_lookup(kind: u16, mouse: (f32, f32)) -> Lookup {
        Lookup {
            start_y: 21,
            end_y: 30,
            ..lookup(item(kind), mouse)
        }
    }

    #[test]
    fn test_smart_1_the_acorn_finds_the_grass_spot() {
        // `Step_Acorns` :2061-2093 — the grass-family arm: an open cell
        // above a grass row with the 9-row box clear replants there.
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 2); // the grass row (the ground)
        let l = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&world, &l, -1, -1),
            Some((25, 21)),
            "the acorn re-points above the grass row"
        );
        // A non-ground row (the fixture's stone) refuses.
        let mut stone = bare();
        put_row(&mut stone, 20, 40, 22, 1);
        let l2 = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&stone, &l2, -1, -1),
            None,
            "stone is not an acorn ground"
        );
    }

    #[test]
    fn test_smart_1_the_acorn_needs_open_space_above() {
        // `Step_Acorns` :2054 — a cell occupied by a non-replaceable tile
        // (the one above or the spot itself) drops the candidate: a solid
        // row over the whole grass row leaves no plant.
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 2);
        put_row(&mut world, 20, 40, 20, 1); // the stone row above the spots
        let l = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&world, &l, -1, -1),
            None,
            "the occupied cell above blocks the plant"
        );
    }

    #[test]
    fn test_smart_1_the_acorn_removal_pass_needs_a_ground_neighbour() {
        // `Step_Acorns` :2096-2136 — a ground row standing alone (no acorn
        // ground beside the spot's own ground cell) is removed.
        let mut world = bare();
        put(&mut world, 25, 22, 2); // a lone grass cell under the spot
        let l = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&world, &l, -1, -1),
            None,
            "the lone ground is dropped by the removal pass"
        );
        // A second grass cell beside it survives.
        let mut row = bare();
        put_row(&mut row, 24, 26, 22, 2);
        let l2 = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&row, &l2, -1, -1),
            Some((25, 21)),
            "the ground row beside the spot keeps it"
        );
    }

    #[test]
    fn test_smart_1_the_acorn_respects_the_far_saplings() {
        // `Step_Acorns` :2058-2060 — `flag2 = flag || (±2/±3)`, so ANY
        // sapling within the ±3 column window blocks the grass arm; the
        // vote falls on the closest free cell outside the window.
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 2);
        put(&mut world, 27, 21, 20); // a sapling two columns right of 25
        let l = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&world, &l, -1, -1),
            Some((23, 21)),
            "the sapling's ±3 window (24..=27) drops, the vote takes 23"
        );
        // An immediate ±1 sapling feeds `flag` and therefore `flag2` — the
        // blocked window widens to 23..=29 and the vote falls to 22.
        let mut near = bare();
        put_row(&mut near, 20, 40, 22, 2);
        put(&mut near, 26, 21, 20); // a sapling directly beside the spot
        let l2 = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            step_acorns(&near, &l2, -1, -1),
            Some((22, 21)),
            "the immediate sapling widens the block to the full ±3 window"
        );
    }

    #[test]
    fn test_smart_1_the_acorn_runs_in_the_chain_and_the_pumpkin_fixes_its_place() {
        // The dispatch order (:137, :149): the pumpkin seed runs before the
        // grass seeds, the acorn after the paint family — the chain dispatches
        // both steps.
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 2);
        let acorn = acorn_lookup(27, mouse_at(25, 21));
        assert_eq!(
            smart_cursor_lookup(&world, &acorn, -1, -1, &mut ContinuityState::default()),
            Some((25, 21)),
            "the acorn step runs inside the lookup chain"
        );
        let mut pumpkins = w();
        put_row(&mut pumpkins, 20, 40, 20, 2); // grass ground at the fixture's floor row
        pumpkins.world_surface = 25.0;
        let l = lookup(placeable(0, 254), mouse_at(25, 19));
        assert_eq!(
            smart_cursor_lookup(&pumpkins, &l, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the pumpkin-seed step is dispatched before the grass seeds"
        );
    }

    #[test]
    fn test_smart_1_the_gem_corn_finds_the_stone_ground() {
        // `Step_GemCorns` :2179 — the Ruby seed (4855) plants on the gem
        // trees' ground (Conversion.Stone {1,25,117,203}); the 12-row clear
        // box ignores the profile's own sapling (590).
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 1); // stone ground
        let l = acorn_lookup(4855, mouse_at(25, 21));
        assert_eq!(
            step_gem_corns(&world, &l, -1, -1),
            Some((25, 21)),
            "the gem seed re-points above the stone ground"
        );
        // The gem profile's ground test refuses dirt (2 — a Conversion.Grass
        // tile, not Conversion.Stone/Moss): the vanilla's GemTreeGroundTest
        // reads only those two sets (WorldGen.cs:30304-30315).
        let mut dirt = bare();
        put_row(&mut dirt, 20, 40, 22, 2);
        let l2 = acorn_lookup(4855, mouse_at(25, 21));
        assert_eq!(
            step_gem_corns(&dirt, &l2, -1, -1),
            None,
            "dirt is not a gem-tree ground"
        );
    }

    #[test]
    fn test_smart_1_the_sakura_seed_finds_grass_but_not_corrupt_grass() {
        // `VanityTreeGroundTest` (WorldGen.cs:30317-30328): the grass
        // conversion minus the corruption/crimson pair — the sakura seed
        // (4871) plants on grass but refuses corrupt grass (23).
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 2); // grass ground
        let l = acorn_lookup(4871, mouse_at(25, 21));
        assert_eq!(
            step_gem_corns(&world, &l, -1, -1),
            Some((25, 21)),
            "the sakura seed re-points above the grass"
        );
        let mut corrupt = bare();
        put_row(&mut corrupt, 20, 40, 22, 23); // corrupt grass ground
        let l2 = acorn_lookup(4871, mouse_at(25, 21));
        assert_eq!(
            step_gem_corns(&corrupt, &l2, -1, -1),
            None,
            "corrupt grass is excluded from the vanity ground test"
        );
    }

    #[test]
    fn test_smart_1_only_the_profile_seed_items_pass_the_gate() {
        // The `TryGetFromItemId` gate (:2161): nine item ids carry a
        // profile; a plain block or the acorn does not.
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 1);
        assert_eq!(
            step_gem_corns(&world, &acorn_lookup(27, mouse_at(25, 21)), -1, -1),
            None,
            "the acorn is not a gem-corn seed"
        );
        assert_eq!(
            step_gem_corns(&world, &acorn_lookup(1, mouse_at(25, 21)), -1, -1),
            None,
            "a plain item carries no profile"
        );
        // The ash profile (634) has no seed item — the C#'s
        // `TryGetFromItemId` does not list it (:3853-3884).
        assert_eq!(
            crate::growth::grow_tree_profile_from_item(634),
            None,
            "the ash tree has no seed item"
        );
    }

    #[test]
    fn test_smart_1_the_gem_corn_removal_pass_needs_a_ground_neighbour() {
        // `Step_GemCorns` :2189-2206 — the removal pass runs the profile's
        // ground test on the cells beside the ground row: a lone stone cell
        // is dropped, a stone row survives.
        let mut world = bare();
        put(&mut world, 25, 22, 1); // a lone stone cell under the spot
        let l = acorn_lookup(4855, mouse_at(25, 21));
        assert_eq!(
            step_gem_corns(&world, &l, -1, -1),
            None,
            "the lone ground is dropped by the gem-corn removal pass"
        );
        let mut row = bare();
        put_row(&mut row, 24, 26, 22, 1);
        let l2 = acorn_lookup(4855, mouse_at(25, 21));
        assert_eq!(
            step_gem_corns(&row, &l2, -1, -1),
            Some((25, 21)),
            "the stone row beside the spot keeps it"
        );
    }

    #[test]
    fn test_smart_1_the_gem_corn_runs_after_the_acorn_in_the_chain() {
        // The dispatch order (:150): the gem-corn step sits between the
        // acorn and the alch seeds.
        let mut world = bare();
        put_row(&mut world, 20, 40, 22, 2);
        let sakura = acorn_lookup(4871, mouse_at(25, 21));
        assert_eq!(
            smart_cursor_lookup(&world, &sakura, -1, -1, &mut ContinuityState::default()),
            Some((25, 21)),
            "the gem-corn step runs inside the lookup chain"
        );
    }

    /// Water of the given amount at the cell (the fixture water: no liquid
    /// flag, `liquid_kind() == Water`).
    fn water_at(w: &mut crate::World, x: i32, y: i32, amount: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
    }

    #[test]
    fn test_smart_1_the_empty_bucket_finds_the_deep_liquid() {
        // `Step_EmptyBuckets` :650 — the 3x3 box of the same liquid kind
        // must hold more than 100.
        let mut world = bare();
        for x in 24..=26 {
            for y in 14..=16 {
                water_at(&mut world, x, y, 60);
            }
        }
        let l = lookup(item(205), mouse_at(25, 15));
        assert_eq!(
            step_empty_buckets(&world, &l, -1, -1),
            Some((25, 15)),
            "the bucket re-points at the 540-liquid box"
        );
        // A shallow puddle (the 3x3 sum 45) is below the minimum.
        let mut shallow = bare();
        water_at(&mut shallow, 25, 15, 45);
        let l2 = lookup(item(205), mouse_at(25, 15));
        assert_eq!(
            step_empty_buckets(&shallow, &l2, -1, -1),
            None,
            "the 45-liquid box is below the bucket's 100 minimum"
        );
        // The item gate: a non-bucket item skips the step.
        let l3 = lookup(item(1), mouse_at(25, 15));
        assert_eq!(
            step_empty_buckets(&world, &l3, -1, -1),
            None,
            "the step's gate is item 205 only"
        );
    }

    #[test]
    fn test_smart_1_the_empty_bucket_sums_only_the_same_kind() {
        // `Step_EmptyBuckets` :644 — the box sum counts the matching
        // `liquidType()` only: a lava pocket beside water does not lift the
        // water total.
        let mut world = bare();
        for x in 24..=26 {
            for y in 14..=16 {
                water_at(&mut world, x, y, 60);
            }
        }
        let i = world.index(25, 15);
        world.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Lava);
        world.tiles[i].liquid = 60; // lava: the water sum drops to 480
        let l = lookup(item(205), mouse_at(25, 15));
        // The water cells still total 480 > 100 → the water spot stands
        // (the lava cell itself is a candidate of its own kind, 180).
        let found = step_empty_buckets(&world, &l, -1, -1);
        assert!(found.is_some(), "the water box still draws the bucket");
    }

    #[test]
    fn test_smart_1_the_block_line_places_on_the_anchored_cell() {
        // `Step_BlocksLines` :838-856 — a replaceable cell with one
        // full-solid neighbour re-points; the settings toggle gates it.
        let blocks = Lookup {
            smart_blocks_enabled: true,
            ..lookup(placeable(1, 1), mouse_at(25, 19)) // the stone block item
        };
        let world = w(); // the fixture's stone floor row at y = 20
        assert_eq!(
            step_blocks_lines(&world, &blocks, -1, -1),
            Some((25, 19)),
            "the block line re-points above the solid floor"
        );
        // The toggle off — the step no-ops (:817).
        let off = lookup(placeable(1, 1), mouse_at(25, 19));
        assert_eq!(
            step_blocks_lines(&world, &off, -1, -1),
            None,
            "SmartBlocksEnabled off disables the step"
        );
        // A solidTop createTile (the platform 19) is refused by the gate.
        let platform = Lookup {
            smart_blocks_enabled: true,
            ..lookup(placeable(0, 19), mouse_at(25, 19))
        };
        assert_eq!(
            step_blocks_lines(&world, &platform, -1, -1),
            None,
            "the solidTop createTile fails the gate"
        );
        // A fully occupied rect offers no line spot.
        let mut occupied = w();
        for y in 10..=19 {
            put_row(&mut occupied, 20, 40, y, 1);
        }
        assert_eq!(
            step_blocks_lines(&occupied, &blocks, -1, -1),
            None,
            "every cell occupied — no placement target"
        );
    }

    #[test]
    fn test_smart_1_the_block_line_refuses_the_seed_items() {
        // `AllowNormalBlockPlacementBehaviourForItemType` (:217): a grass
        // seed holding nothing to place — even with a createTile — is not a
        // normal block placement.
        let world = w();
        let seed = Lookup {
            smart_blocks_enabled: true,
            ..lookup(placeable(62, 1), mouse_at(25, 19))
        };
        assert_eq!(
            step_blocks_lines(&world, &seed, -1, -1),
            None,
            "the grass seed takes its own step, not the block lines"
        );
    }

    /// A tall bare world (100 rows — the walk's `maxTilesY - 40` clamp needs
    /// a real world height; the 40-row fixture clamps every row to 0 and the
    /// walk fails on its first out-of-world read, exactly as the C# would).
    fn tall() -> crate::World {
        let mut w = crate::World::new("tall".into(), 60, 100);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w
    }

    /// The frame column of the cell (`Tile.frameX`, the door frames' key).
    fn frame_x_at(w: &mut crate::World, x: i32, y: i32, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i].frame_x = frame_x;
    }

    #[test]
    fn test_smart_1_the_wall_step_finds_the_anchored_cell() {
        // `Step_Walls` :1114-1132 — a free wall slot behind an empty cell
        // that stands beside a wall neighbour and is reachable from the
        // player (the default lookup's player centre is the tile (30, 22)).
        let mut world = tall();
        wall_at(&mut world, 25, 20, 1); // the neighbour wall below the spot
        let l = lookup(placeable_wall(0, 1), mouse_at(25, 19));
        assert_eq!(
            step_walls(&world, &l, -1, -1),
            Some((25, 19)),
            "the wall re-points beside the wall neighbour"
        );
        // A taken slot is not the answer for that cell (the step moves on
        // to the neighbour the taken cell anchors — the C#'s `tile.wall == 0`
        // reads per cell).
        let mut taken = tall();
        wall_at(&mut taken, 25, 20, 1);
        wall_at(&mut taken, 25, 19, 1);
        let l2 = lookup(placeable_wall(0, 1), mouse_at(25, 19));
        assert_ne!(
            step_walls(&taken, &l2, -1, -1),
            Some((25, 19)),
            "the cell's wall slot is taken"
        );
        // No tile-or-wall neighbour — the spot is not a wall.
        let bare_world = tall();
        let l3 = lookup(placeable_wall(0, 1), mouse_at(25, 19));
        assert_eq!(
            step_walls(&bare_world, &l3, -1, -1),
            None,
            "the spot needs a tile-or-wall neighbour"
        );
    }

    #[test]
    fn test_smart_1_the_wall_step_line_of_sight_blocks() {
        // `CanHitWithCheck` :526-534 — a full-solid tower on both sides of
        // the crossed column breaks the walk, so the spot is unreachable.
        let mut world = tall();
        wall_at(&mut world, 25, 20, 1); // the neighbour that anchors the spot
        put(&mut world, 28, 21, 1); // the tower beside the walked column
        put(&mut world, 28, 23, 1);
        let l = lookup(placeable_wall(0, 1), mouse_at(25, 19));
        assert_eq!(
            step_walls(&world, &l, -1, -1),
            None,
            "the solid tower blocks the line walk"
        );
        // Without the tower the same spot is reachable.
        let mut open = tall();
        wall_at(&mut open, 25, 20, 1);
        assert_eq!(
            step_walls(
                &open,
                &lookup(placeable_wall(0, 1), mouse_at(25, 19)),
                -1,
                -1
            ),
            Some((25, 19)),
            "the open walk reaches the spot"
        );
    }

    #[test]
    fn test_smart_1_the_open_door_anchor_frame_veto() {
        // `IsOpenDoorAnchorFrame` (WorldGen.cs:69171-69184): the open door's
        // 11 anchors a wall on its top (columns 0..17) and bottom (54..71)
        // frame columns — the middle columns (18..53) do not.
        let mut world = tall();
        put(&mut world, 25, 19, 11);
        frame_x_at(&mut world, 25, 19, 0); // the top frame column
        assert!(crate::tileobject::is_open_door_anchor_frame(&world, 25, 19));
        frame_x_at(&mut world, 25, 19, 36); // the door's middle
        assert!(!crate::tileobject::is_open_door_anchor_frame(
            &world, 25, 19
        ));
        frame_x_at(&mut world, 25, 19, 60); // the bottom column
        assert!(crate::tileobject::is_open_door_anchor_frame(&world, 25, 19));
        // A closed door (10) or a plain tile is never an anchor.
        put(&mut world, 26, 19, 10);
        assert!(!crate::tileobject::is_open_door_anchor_frame(
            &world, 26, 19
        ));
        put(&mut world, 27, 19, 1);
        assert!(!crate::tileobject::is_open_door_anchor_frame(
            &world, 27, 19
        ));
    }

    #[test]
    fn test_smart_1_the_wall_step_runs_first_in_the_chain() {
        // The dispatch order (:136): the wall step runs before the pumpkin
        // seeds — the chain dispatches it.
        let mut world = tall();
        wall_at(&mut world, 25, 20, 1);
        let l = lookup(placeable_wall(0, 1), mouse_at(25, 19));
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the wall step runs inside the lookup chain"
        );
    }

    /// The desired-direction quantiser (`GetDesiredDirectionFrom` :1443):
    /// the mouse bearing from a cell to one of the eight directions ×1.5.
    fn direction_of(offset: (f32, f32)) -> (i32, i32) {
        get_desired_direction_from(offset, &ContinuityState::default())
    }

    #[test]
    fn test_smart_1_the_desired_direction_quantises_the_bearing() {
        // The eight buckets: flat right, the diagonals, flat down and their
        // mirrors (the `(int)` truncation of the ×1.5 unit vectors).
        assert_eq!(direction_of((40.0, 0.0)), (1, 0));
        assert_eq!(direction_of((40.0, 40.0)), (1, 1));
        assert_eq!(direction_of((0.0, 40.0)), (0, 1));
        assert_eq!(direction_of((-40.0, 40.0)), (-1, 1));
        assert_eq!(direction_of((-40.0, 0.0)), (-1, 0));
        assert_eq!(direction_of((-40.0, -40.0)), (-1, -1));
        assert_eq!(direction_of((0.0, -40.0)), (0, -1));
        assert_eq!(direction_of((40.0, -40.0)), (1, -1));
        // A lock short-circuits the bearing — the line keeps its direction.
        let locked = ContinuityState {
            locked_desired_direction: Some((0, 1)),
            locked_continuity_coords: Some((25, 18)),
        };
        assert_eq!(get_desired_direction_from((40.0, 0.0), &locked), (0, 1));
    }

    #[test]
    fn test_smart_1_the_platform_step_extends_the_row() {
        // `Step_Platforms` :1343-1356 — the step works from the platform
        // cells themselves (a plain empty cell is skipped by the flag), and
        // re-points at the first free cell beyond the row's end, in the
        // mouse's direction.
        let mut world = bare();
        put_row(&mut world, 20, 30, 18, 19); // the platform row
        let l = lookup(placeable(0, 19), (35.0 * 16.0 + 8.0, 18.0 * 16.0 + 8.0));
        let mut state = ContinuityState::default();
        assert_eq!(
            step_platforms(&world, &l, -1, -1, &mut state),
            Some((31, 18)),
            "the platform extends the row to the right"
        );
        // The hit writes both locks (:1383-1384).
        assert_eq!(state.locked_desired_direction, Some((1, 0)));
        assert_eq!(state.locked_continuity_coords, Some((31, 18)));
        // The gate: a non-platform item refuses.
        assert_eq!(
            step_platforms(
                &world,
                &lookup(placeable(1, 1), (0.0, 0.0)),
                -1,
                -1,
                &mut state
            ),
            None,
            "the step's gate is a platform-placing item"
        );
    }

    #[test]
    fn test_smart_1_the_continuity_lock_and_the_placement_gate() {
        // `TileTargetDesired` (:1391-1402): without a lock the gate always
        // passes; with one the tile target must be the smart cursor's own
        // re-pointed cell (`Player.cs:39721`).
        let open = ContinuityState::default();
        assert!(tile_target_desired(&open, true, (12, 12), (25, 18)));
        assert!(tile_target_desired(&open, false, (12, 12), (25, 18)));
        let locked = ContinuityState {
            locked_desired_direction: Some((1, 0)),
            locked_continuity_coords: Some((31, 18)),
        };
        assert!(tile_target_desired(&locked, true, (31, 18), (31, 18)));
        assert!(!tile_target_desired(&locked, true, (32, 18), (31, 18)));
        // The smart cursor not showing — the gate refuses (the C#'s
        // `Main.SmartCursorShowing` half).
        assert!(!tile_target_desired(&locked, false, (31, 18), (31, 18)));
    }

    #[test]
    fn test_smart_1_the_continuity_lock_resets_without_the_use_button() {
        // `SmartCursorLookup` :66-67 — the locks survive only while the use
        // button is held, the smart cursor is on and the item places a
        // platform.
        let mut world = bare();
        put_row(&mut world, 20, 30, 18, 19);
        let mut state = ContinuityState {
            locked_desired_direction: Some((1, 0)),
            locked_continuity_coords: Some((31, 18)),
        };
        // Not held — the reset clears the locks.
        let l = lookup(placeable(0, 19), (0.0, 0.0));
        smart_cursor_lookup(&world, &l, -1, -1, &mut state);
        assert_eq!(state.locked_desired_direction, None, "the lock reset");
        assert_eq!(state.locked_continuity_coords, None, "the coords reset");
        // Held — the locks stand (and the step may re-write them).
        let held = Lookup {
            use_item_held: true,
            ..lookup(placeable(0, 19), (35.0 * 16.0 + 8.0, 18.0 * 16.0 + 8.0))
        };
        smart_cursor_lookup(&world, &held, -1, -1, &mut state);
        assert_eq!(
            state.locked_continuity_coords,
            Some((31, 18)),
            "the lock stood"
        );
    }

    #[test]
    fn test_smart_1_the_platform_step_runs_first_in_the_chain() {
        // The dispatch order (:134): the platform step runs before the wall
        // step — the chain dispatches it.
        let mut world = bare();
        put_row(&mut world, 20, 30, 18, 19);
        let l = Lookup {
            use_item_held: true,
            ..lookup(placeable(0, 19), (35.0 * 16.0 + 8.0, 18.0 * 16.0 + 8.0))
        };
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            Some((31, 18)),
            "the platform step runs inside the lookup chain"
        );
    }

    /// A red wire at the cell (the port's sparse `WireState`).
    fn red_wire_at(w: &mut crate::World, x: i32, y: i32) {
        w.wires.set(x, y, crate::wiring::WireColor::Red, true);
    }

    #[test]
    fn test_smart_1_the_wire_cutter_finds_the_wired_cell() {
        // `Step_WireCutter` :1467 — any wire colour or the actuator makes a
        // cell a cutter target; a clean cell does not.
        let mut world = bare();
        red_wire_at(&mut world, 25, 19);
        let l = lookup(item(510), mouse_at(25, 19));
        assert_eq!(
            step_wire_cutter(&world, &l, -1, -1),
            Some((25, 19)),
            "the cutter re-points at the wired cell"
        );
        // The actuator alone is a target too (the C#'s `||` arm).
        let mut actuated = bare();
        let i = actuated.index(25, 19);
        actuated.tiles[i].flags |= crate::tile::FLAG_ACTUATOR;
        assert_eq!(
            step_wire_cutter(&actuated, &l, -1, -1),
            Some((25, 19)),
            "the cutter re-points at the actuated cell"
        );
        // Nothing wired and no actuator — the step no-ops.
        let clean = bare();
        assert_eq!(
            step_wire_cutter(&clean, &l, -1, -1),
            None,
            "a clean cell is not a cutter target"
        );
    }

    #[test]
    fn test_smart_1_the_actuator_step_wants_the_wired_unactuated_cell() {
        // `Step_Actuators` :594 — wired, not actuated, active.
        let mut world = bare();
        put(&mut world, 25, 19, 1);
        red_wire_at(&mut world, 25, 19);
        let l = lookup(item(849), mouse_at(25, 19));
        assert_eq!(
            step_actuators(&world, &l, -1, -1),
            Some((25, 19)),
            "the actuator re-points at the wired active cell"
        );
        // Already actuated — not a target.
        let mut done = bare();
        put(&mut done, 25, 19, 1);
        red_wire_at(&mut done, 25, 19);
        let i = done.index(25, 19);
        done.tiles[i].flags |= crate::tile::FLAG_ACTUATOR;
        assert_eq!(
            step_actuators(&done, &l, -1, -1),
            None,
            "the actuated cell is not an actuator target"
        );
        // Wired but inactive — the `active()` half refuses.
        let mut inert = bare();
        red_wire_at(&mut inert, 25, 19);
        assert_eq!(
            step_actuators(&inert, &l, -1, -1),
            None,
            "the inactive cell is not an actuator target"
        );
    }

    #[test]
    fn test_smart_1_the_actuation_rod_respects_the_lock() {
        // `Step_ActuationRod` :1509 — the rod's channel lock (Player.cs:2616/
        // :2618, caller-passed) filters the actuation states it toggles.
        let mut world = bare();
        let i = world.index(25, 19);
        world.tiles[i].flags |= crate::tile::FLAG_ACTIVE | crate::tile::FLAG_ACTUATOR;
        // Lock off — every actuated cell is a target.
        let off = lookup(item(3620), mouse_at(25, 19));
        assert_eq!(
            step_actuation_rod(&world, &off, -1, -1),
            Some((25, 19)),
            "the rod re-points at the actuated cell"
        );
        // Lock on, setting wants the actuated (false) state — the cell's
        // `inActive()` is false, so it matches.
        let locked_false = Lookup {
            actuation_rod_lock: true,
            actuation_rod_lock_setting: false,
            ..lookup(item(3620), mouse_at(25, 19))
        };
        assert_eq!(
            step_actuation_rod(&world, &locked_false, -1, -1),
            Some((25, 19)),
            "the locked rod accepts the matching state"
        );
        // Lock on, setting wants the inert (true) state — mismatch, no target.
        let locked_true = Lookup {
            actuation_rod_lock: true,
            actuation_rod_lock_setting: true,
            ..lookup(item(3620), mouse_at(25, 19))
        };
        assert_eq!(
            step_actuation_rod(&world, &locked_true, -1, -1),
            None,
            "the locked rod refuses the mismatched state"
        );
    }

    #[test]
    fn test_smart_1_the_colored_wrench_extends_the_line() {
        // `Step_ColoredWrenches` :1924-2006 — the wrench re-points beside a
        // cell that carries its channel, into a neighbour lacking it.
        let mut world = bare();
        red_wire_at(&mut world, 25, 19);
        let l = lookup(item(509), mouse_at(25, 19)); // the red wrench
        assert_eq!(
            step_colored_wrenches(&world, &l, -1, -1),
            Some((24, 19)),
            "the red wrench re-points at the free neighbour"
        );
        // The screen target already carries the channel — the scan stands
        // down (:1904-1918).
        let mut hovered = bare();
        red_wire_at(&mut hovered, 25, 19);
        red_wire_at(&mut hovered, 30, 19); // the fixture's screen target
        let l2 = lookup(item(509), mouse_at(25, 19));
        assert_eq!(
            step_colored_wrenches(&hovered, &l2, -1, -1),
            None,
            "the hovered channel disables the wrench scan"
        );
        // A different channel's wrench ignores the red wire.
        let l3 = lookup(item(850), mouse_at(25, 19)); // the blue wrench
        assert_eq!(
            step_colored_wrenches(&world, &l3, -1, -1),
            None,
            "the blue wrench wants blue wire"
        );
    }

    #[test]
    fn test_smart_1_the_multicolor_wrench_cutter_and_placement_arms() {
        // `Step_MulticolorWrench` :1764-1849 — the cutter arm targets the
        // wired cells themselves, the placement arm their free neighbours.
        let mut world = bare();
        red_wire_at(&mut world, 25, 19);
        // Cutter + red: the wired cell itself.
        let cutter = Lookup {
            tool_mode: multi_tool_mode::RED | multi_tool_mode::CUTTER,
            ..lookup(item(3625), mouse_at(25, 19))
        };
        assert_eq!(
            step_multicolor_wrench(&world, &cutter, -1, -1),
            Some((25, 19)),
            "the cutter arm re-points at the wired cell"
        );
        // Red only (placement): the free neighbour.
        let place = Lookup {
            tool_mode: multi_tool_mode::RED,
            ..lookup(item(3625), mouse_at(25, 19))
        };
        assert_eq!(
            step_multicolor_wrench(&world, &place, -1, -1),
            Some((24, 19)),
            "the placement arm re-points at the free neighbour"
        );
        // The hovered cell matches the placement channels — the scan stands
        // down (:1748).
        let mut hovered = bare();
        red_wire_at(&mut hovered, 25, 19);
        red_wire_at(&mut hovered, 30, 19); // the fixture's screen target
        assert_eq!(
            step_multicolor_wrench(&hovered, &place, -1, -1),
            None,
            "the matching hover disables the multi-wrench scan"
        );
    }

    #[test]
    fn test_smart_1_the_wire_family_runs_in_the_chain() {
        // The dispatch order (:129-133, :152): the wire tools run at their
        // C# places — the chain dispatches them.
        let mut world = bare();
        red_wire_at(&mut world, 25, 19);
        let cutter = lookup(item(510), mouse_at(25, 19));
        assert_eq!(
            smart_cursor_lookup(&world, &cutter, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the wire cutter runs inside the lookup chain"
        );
        let mut wired = bare();
        put(&mut wired, 25, 19, 1);
        red_wire_at(&mut wired, 25, 19);
        let actuator = lookup(item(849), mouse_at(25, 19));
        assert_eq!(
            smart_cursor_lookup(&wired, &actuator, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the actuator step runs inside the lookup chain"
        );
    }

    /// A track tile at the cell (the literal 314).
    fn track_at(w: &mut crate::World, x: i32, y: i32) {
        put(w, x, y, TRACK_TILE);
    }

    #[test]
    fn test_smart_1_the_minecart_track_extends_in_the_mouse_direction() {
        // `Step_MinecartTracks` :1170-1239 — the flat-right bearing extends
        // the existing track to the right.
        let mut world = bare();
        track_at(&mut world, 30, 15);
        let l = lookup(item(2340), (590.0, 357.0)); // right of the player's centre
        assert_eq!(
            step_minecart_tracks(&world, &l, -1, -1),
            Some((31, 15)),
            "the track extends to the right"
        );
        // The diagonal bearing (up-right): the diagonal arm extends the
        // track up-right, and the axis arm is cleared by the dominance.
        let diag = lookup(item(2340), (550.0, 297.0));
        assert_eq!(
            step_minecart_tracks(&world, &diag, -1, -1),
            Some((31, 14)),
            "the track extends up-right"
        );
        // The gate: a non-track item skips the step.
        let other = lookup(item(1), (0.0, 0.0));
        assert_eq!(
            step_minecart_tracks(&world, &other, -1, -1),
            None,
            "the step's gate is the track items"
        );
    }

    #[test]
    fn test_smart_1_the_track_candidate_must_stand_freestanding() {
        // `Step_MinecartTracks` :1250-1256 — the candidate under a track is
        // filtered, but the line continues from the last track's own end:
        // the track at (31,16) still extends rightward, so the vote falls
        // on its free neighbour (32,16) — the C#'s scan anchors on every
        // track cell in the rect, not just the first.
        let mut world = bare();
        track_at(&mut world, 30, 15);
        track_at(&mut world, 31, 16); // the track below the right candidate
        let l = lookup(item(2340), (590.0, 357.0));
        assert_eq!(
            step_minecart_tracks(&world, &l, -1, -1),
            Some((32, 16)),
            "the filtered candidate drops, the line's tail extends"
        );
    }

    #[test]
    fn test_smart_1_the_pressure_track_extends_the_line_sideways() {
        // `Step_MinecartTracks` :1285-1294 — the pressure track (2492)
        // extends the line left/right only, ignoring the mouse's bearing.
        let mut world = bare();
        track_at(&mut world, 30, 15);
        let l = lookup(item(2492), (590.0, 357.0));
        assert_eq!(
            step_minecart_tracks(&world, &l, -1, -1),
            Some((31, 15)),
            "the pressure track extends to the right"
        );
        // The screen target already a track → the scan stands down
        // (:1274-1279).
        let mut held = bare();
        track_at(&mut held, 30, 15);
        track_at(&mut held, 30, 19); // the fixture's screen target
        assert_eq!(
            step_minecart_tracks(&held, &l, -1, -1),
            None,
            "the hovered track disables the pressure scan"
        );
    }

    #[test]
    fn test_smart_1_the_track_step_runs_in_the_chain() {
        // The dispatch order (:135): the track step runs between the
        // platform and the wall steps — the chain dispatches it.
        let mut world = bare();
        track_at(&mut world, 30, 15);
        let l = lookup(item(2340), (590.0, 357.0));
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            Some((31, 15)),
            "the track step runs inside the lookup chain"
        );
    }

    #[test]
    fn test_smart_1_the_force_cursor_locks_the_minable_tile() {
        // `Step_ForceCursorToAnyMinableThing` :2243-2268 — the step writes
        // the mouse's own tile when it is minable with the held tool.
        let mut world = bare();
        put(&mut world, 25, 19, 4); // a torch: minable, non-solid
        let pick = lookup(pickaxe(1, 1), mouse_at(25, 19));
        assert_eq!(
            step_force_cursor(&world, &pick, -1, -1),
            Some((25, 19)),
            "the cursor locks onto the minable torch"
        );
        // A full solid tile is not forced (the `!tileSolid || tileSolidTop`
        // half).
        let mut stone = bare();
        put(&mut stone, 25, 19, 1);
        assert_eq!(
            step_force_cursor(&stone, &pick, -1, -1),
            None,
            "the solid tile is not a force target"
        );
        // A taken focus keeps the earlier step's target.
        assert_eq!(
            step_force_cursor(&world, &pick, 12, 12),
            None,
            "the focus sentinel gate holds"
        );
        // The gamepad arm (input state, caller-passed).
        let pad = Lookup {
            using_gamepad: true,
            ..lookup(pickaxe(1, 1), mouse_at(25, 19))
        };
        assert_eq!(
            step_force_cursor(&world, &pad, -1, -1),
            None,
            "the gamepad skips the force cursor"
        );
    }

    #[test]
    fn test_smart_1_the_force_cursor_demotes_the_tool_families() {
        // `Step_ForceCursorToAnyMinableThing` :2250-2261 — the axe tiles
        // need the axe, the hammer tiles the hammer, the rest the pickaxe.
        let tree = {
            let mut w = bare();
            put(&mut w, 25, 19, 5); // a tree trunk: tileAxe
            w
        };
        let no_axe = lookup(item(1), mouse_at(25, 19));
        assert_eq!(
            step_force_cursor(&tree, &no_axe, -1, -1),
            None,
            "the tree trunk needs an axe"
        );
        let with_axe = lookup(ItemPort { axe: 1, ..item(1) }, mouse_at(25, 19));
        assert_eq!(
            step_force_cursor(&tree, &with_axe, -1, -1),
            Some((25, 19)),
            "the axe-equipped item forces the cursor onto the trunk"
        );
        let altar = {
            let mut w = bare();
            put(&mut w, 25, 19, 26); // the demon altar: tileHammer
            w
        };
        let no_hammer = lookup(item(1), mouse_at(25, 19));
        assert_eq!(
            step_force_cursor(&altar, &no_hammer, -1, -1),
            None,
            "the altar needs a hammer"
        );
        let with_hammer = lookup(
            ItemPort {
                hammer: 1,
                ..item(1)
            },
            mouse_at(25, 19),
        );
        assert_eq!(
            step_force_cursor(&altar, &with_hammer, -1, -1),
            Some((25, 19)),
            "the hammer-equipped item forces the cursor onto the altar"
        );
        // And the demoted families do not accept the wrong tool: an axe
        // does not break the altar, a pickaxe does not break the trunk.
        let axe_only = lookup(ItemPort { axe: 1, ..item(1) }, mouse_at(25, 19));
        assert_eq!(
            step_force_cursor(&altar, &axe_only, -1, -1),
            None,
            "the axe does not break the hammer tile"
        );
    }

    #[test]
    fn test_smart_1_the_force_cursor_runs_first_in_the_chain() {
        // The dispatch order (:122): the force-cursor step runs at the head
        // of the chain — the chain dispatches it.
        let mut world = bare();
        put(&mut world, 25, 19, 4);
        let pick = lookup(pickaxe(1, 1), mouse_at(25, 19));
        assert_eq!(
            smart_cursor_lookup(&world, &pick, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the force-cursor step runs inside the lookup chain"
        );
    }

    #[test]
    fn test_smart_1_the_mine_shinies_finds_the_override_tiles() {
        // `Step_Pickaxe_MineShinies` :2298-2312 — the three priority-
        // override tiles (12/665/639, TileID.cs:507) are the step's only
        // targets; the C#'s priority filter is inert (the `_targets2` loops
        // iterate the just-cleared list), so the closest override tile
        // wins.
        let mut world = bare();
        put(&mut world, 25, 19, 12); // the override tile
        let l = lookup(pickaxe(1, 1), mouse_at(25, 19));
        assert_eq!(
            step_pickaxe_mine_shinies(&world, &l, -1, -1),
            Some((25, 19)),
            "the pickaxe re-points at the override tile"
        );
        let mut rare = bare();
        put(&mut rare, 25, 19, 665);
        put(&mut rare, 26, 19, 639);
        assert_eq!(
            step_pickaxe_mine_shinies(&rare, &l, -1, -1),
            Some((25, 19)),
            "both override ids are candidates, the closest wins"
        );
        // A plain tile is not a target.
        let mut stone = bare();
        put(&mut stone, 25, 19, 1);
        assert_eq!(
            step_pickaxe_mine_shinies(&stone, &l, -1, -1),
            None,
            "a plain tile is not a shiny target"
        );
        // The gates: pick power and the focus sentinel.
        let no_pick = lookup(item(1), mouse_at(25, 19));
        assert_eq!(step_pickaxe_mine_shinies(&world, &no_pick, -1, -1), None);
        assert_eq!(step_pickaxe_mine_shinies(&world, &l, 12, 12), None);
    }

    #[test]
    fn test_smart_1_the_mine_shinies_step_runs_in_the_chain() {
        // The dispatch order (:123): the shiny step runs between the force
        // cursor and the wrenches — the chain dispatches it.
        let mut world = bare();
        put(&mut world, 25, 19, 12);
        let l = lookup(pickaxe(1, 1), mouse_at(25, 19));
        assert_eq!(
            smart_cursor_lookup(&world, &l, -1, -1, &mut ContinuityState::default()),
            Some((25, 19)),
            "the shinies step runs inside the lookup chain"
        );
    }
}
