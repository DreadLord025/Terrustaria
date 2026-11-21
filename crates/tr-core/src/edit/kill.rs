//! The KillTile destruction cascade (S1-K-4 round 3): `WorldGen.KillTile` /
//! `SquareTileFrame` and the per-object `Check*` routines, the `CanKillTile`
//! break refusals and the `KillTile_PlaySounds` cue table. Moved verbatim from
//! `edit.rs`; behaviour frozen. The shared edit helpers (`touch`, `after_edit`,
//! `liquid_flags`, `is_locked_door`, `ALCH_SOILS`, `WALL_UNBREAKABLE`) stay in
//! the parent module; a child module sees its parent's private items, so the
//! only seams are the `pub(super)` test helpers and the re-exports at `edit::`.

use super::{
    after_edit, is_locked_door, liquid_flags, touch, ALCH_SOILS, WALL_UNBREAKABLE,
};
use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

/// One object destruction the kill cascade performed — the world-state effect
/// of a `WorldGen.Check*` routine (`WorldGen.cs:57923/:86339/:86432/:43335/
/// :46973/:49187`): the whole object was removed and vanilla would spawn
/// `Item.NewItem` with the rect origin at `(x, y)` (tile coordinates; the
/// rect is 32x32 px for furniture, 16x16 for doors). `tile_kind` is the
/// object's tile id; `item` carries the id only when the `Check*` body spawns
/// the item itself — the sunflower's `Item.NewItem(…, 63)`
/// (`WorldGen.cs:53787-53791`); `None` resolves app-side through
/// `registry::tile_drop` (style-0 drops: door 25, table 32, chair 34, bench
/// 36, chest 48 — `WorldGen.cs:69204/:42194/:42635/:42447/:41677`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ObjectDrop {
    pub tile_kind: u16,
    pub x: i32,
    pub y: i32,
    /// The item the check spawned with the destruction, if any (see above).
    pub item: Option<i32>,
}

/// One broken loot pot — `CheckPot`'s tail (`WorldGen.cs:57011-57014`): after
/// the 2x2 dies, vanilla runs `SpawnThingsFromPot(i, j, num, num2, num4)` at
/// the check's trigger cell with the pot's style row (`frameY / 36`). The app
/// resolves the loot rolls (phase 3) and plays the break sound
/// ([`pot_break_sound`]) from `style`; gated by `netMode != 1` only, so the
/// event ignores `kill_tile`'s `no_item`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PotEvent {
    pub style: i32,
    pub x: i32,
    pub y: i32,
}

/// One cascade-killed herb cell — `CheckAlch`'s kill loop (`WorldGen.cs:46088-46091`)
/// whose `KillTile` runs the per-cell `KillTile_DropItems` herb rolls
/// (`KillTile_GetItemDrops`, `WorldGen.cs:65674-65698`: the herb `313 + frameX / 18`
/// plus the seed stack behind `IsHarvestableHerbWithSeed`). The app resolves the
/// RNG rolls (phase 3); `noItem`-gated like the records (the C# drop is a
/// `KillTile_DropItems` drop).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HerbEvent {
    pub kind: u16,
    /// `frameX / 18` — the herb style (0 Daybloom … 6 Shiverthorn).
    pub style: i32,
    pub x: i32,
    pub y: i32,
}

/// One broken 2x1 pile's loot block — `Check2x1`'s `Item.NewItem` table for
/// tile 185 (`WorldGen.cs:47130-47245`), keyed on the origin cell's `frameX`
/// (the right cell's when the origin already died). The app resolves the
/// `genRand` coin/ore counts (phase 3); a `Check*`-level `Item.NewItem`, so it
/// is not `noItem`-gated. `kind` disambiguates the two pile loot blocks:
/// 185's 2x1 coin/ore table (`Check2x1`, `WorldGen.cs:47130-47245`) and
/// 187's sword-in-stone roll (`Check3x2`'s `case 187`,
/// `WorldGen.cs:49828-49839`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PileLootEvent {
    pub kind: u16,
    pub frame_x: i32,
    pub x: i32,
    pub y: i32,
}

/// One killed tree cell's drop record — `WorldGen.KillTile_GetTreeDrops`
/// (`WorldGen.cs:66189-66261`) for the trunk kinds (5/596/616/634) and the
/// `case 323` palm arm of `KillTile_GetItemDrops` (`:65232-65266`). The C#
/// rolls its RNG inside the drop; the core has no stream, so the event carries
/// everything the rolls need and the app resolves them in the C# order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeDropEvent {
    /// A trunk cell — `KillTile_GetTreeDrops`.
    Trunk {
        x: i32,
        y: i32,
        /// The killed cell's `frameX`/`frameY` — the canopy marker test
        /// (`frameX >= 22 && frameY >= 198`, `:66191`).
        frame_x: i16,
        frame_y: i16,
        /// `WorldGen.GetTreeTypeAndTreeBottom` — the family of the ground
        /// under the tree and the trunk column's ground cell.
        tree_type: crate::tree::TreeType,
        bottom_x: i32,
        bottom_y: i32,
    },
    /// A palm cell — `KillTile_GetItemDrops`'s `case 323`.
    Palm {
        x: i32,
        y: i32,
        /// The killed cell's `frameX` — the acorn marker (`88..=132`).
        frame_x: i16,
        /// The wood the ground names (2504 by default, 911/621/619 on the
        /// crimson/hallow/ebon sand).
        item: u16,
    },
    /// A gem-tree cell (583-589) — `KillTile_GetItemDrops`'s gem cases
    /// (`WorldGen.cs:65699-65746`) through `SetGemTreeDrops` (`:66094-66108`).
    /// All rolls are `Main.rand` (no `genRand`): the gem-or-stone `Next(10)`,
    /// the canopy marker's seed `Next(2)`, then the stone stack `Next(1, 3)`
    /// — in that order (the stack roll sits in the case body AFTER
    /// `SetGemTreeDrops` returns).
    Gem {
        x: i32,
        y: i32,
        /// The killed cell's frames — the canopy marker test
        /// (`frameX >= 22 && frameY >= 198`, `:66104`).
        frame_x: i16,
        frame_y: i16,
        /// The gem the 1-in-10 roll mints (`SetGemTreeDrops`'s `gemType`).
        gem: u16,
        /// The gemcorn seed the canopy marker's 1-in-2 roll mints
        /// (`seedType`).
        seed: u16,
    },
    /// A mushroom-tree cell (72) — `KillTile_GetItemDrops`'s `case 71:
    /// case 72:` (`WorldGen.cs:65642-65652`). `Main.rand` only: `Next(40) == 0`
    /// mints the Mushroom Grass Seeds 194, else `Next(2) == 0` mints the
    /// Glowing Mushroom 183, else nothing.
    Mushroom { x: i32, y: i32 },
    /// A cactus cell (80) — `KillTile_GetItemDrops`'s `case 80`
    /// (`WorldGen.cs:64717-64719`): a flat Cactus 276, no RNG. Recorded per
    /// cell so the `CheckCactus` cascade drops every collapsing segment (the
    /// hit cell's `cell_drops` is suppressed for 80 to avoid a double spawn).
    Cactus { x: i32, y: i32 },
}

/// The result of `kill_tile`: the kind of the hit cell plus every object
/// destruction the cascade performed, in `SquareTileFrame` visit order, every
/// loot pot the cascade broke, every cascade-killed herb cell and every broken
/// 2x1 pile's loot block.
#[derive(Clone, Debug, Default)]
pub struct TileKill {
    /// Tile kind of the killed (hit) cell.
    pub kind: u16,
    pub object_drops: Vec<ObjectDrop>,
    pub events: Vec<PotEvent>,
    pub herb_events: Vec<HerbEvent>,
    pub pile_events: Vec<PileLootEvent>,
    /// Every killed tree cell's drop record.
    pub tree_events: Vec<TreeDropEvent>,
    /// The palm cells whose canopy frame the C# re-rolls
    /// (`WorldGen.CheckPalmTree`, `WorldGen.cs:54385-54388`) — the roll needs
    /// the `genRand` stream, so the app resolves it and writes the frame.
    pub palm_frame_rolls: Vec<(i32, i32)>,
    /// The mushroom-tree top cells whose canopy frame the C# re-rolls
    /// (`TileFrameImportant`'s `case 72`, `WorldGen.cs:85855-85873`:
    /// `genRand.Next(3)` picks frame (18, 0/18/36)) — the app resolves it.
    pub mushroom_frame_rolls: Vec<(i32, i32)>,
}

// The ring walk's re-entrancy cap (`square_tile_frame` → the check → the
// sweep → the ring → `square_tile_frame` — the vanilla terminates through
// the objects dying; the cap guards the port's pathologies).
thread_local! {
    static RING_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// `SquareTileFrame`'s `TileFrame` visit order (`WorldGen.cs:80185-80196`):
/// row-major over the 3x3 neighbourhood, self last of its row.
const SQUARE_FRAME_ORDER: [(i32, i32); 9] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 0),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// `WorldGen.KillTile` (`WorldGen.cs:63517`): kills the active tile at
/// `(x, y)`, keeping its background wall, then runs the destruction cascade —
/// `SquareTileFrame(i, j)` (:63919 → :80185) frames the 3x3 neighbourhood in
/// fixed order and the per-object `Check*` routines destroy any object that
/// lost a cell or its support, dropping its item. `no_item` mirrors the C#
/// `noItem` flag (worldgen/replication suppress the drop reporting; the
/// cascade itself still runs). Returns `None` when nothing was killed.
pub fn kill_tile(world: &mut World, x: i32, y: i32, no_item: bool) -> Option<TileKill> {
    let mut destroy_object = false;
    kill_tile_cascading(world, x, y, no_item, &mut destroy_object)
}

/// `WorldGen.SquareTileFrame(i, j, resetFrame: false)` (`Wiring.cs:3212`,
/// `:3229` — the `DeActive`/`ReActive` tails): the 3×3 `TileFrame` walk with
/// the frame kept. Returns the object drops the support checks produced (a
/// torch or a piece of furniture whose anchor actuated away pops, exactly
/// like the C# `TileFrame` → `Check*` → `destroyObject` chain). The actuated
/// tile itself stays "active" (the inactive state is a separate bit), so the
/// cascade treats it as a normal cell.
pub fn actuation_square_frame(world: &mut World, x: i32, y: i32) -> Vec<ObjectDrop> {
    let mut destroy_object = false;
    let mut kill = TileKill {
        kind: world.tile(x, y).map(|t| t.kind).unwrap_or(0),
        ..Default::default()
    };
    square_tile_frame(world, x, y, false, &mut destroy_object, &mut kill);
    after_edit(world, x, y);
    kill.object_drops
}

/// `WorldGen.SquareTileFrame(i, j)` (`WorldGen.cs:80185-80196`) — the
/// worldgen entry the temple passes call over the altar and the furniture:
/// the 3×3 `TileFrame` walk with no item drops and no start-up
/// `destroyObject` (the pass-local worldgen flag), the kills' events
/// discarded (the port's worldgen has no loot side channel — none of the
/// pass contracts report the objects the checks destroy).
pub(crate) fn square_tile_frame_gen(world: &mut World, x: i32, y: i32) {
    let mut destroy_object = false;
    let mut kill = TileKill::default();
    square_tile_frame(world, x, y, true, &mut destroy_object, &mut kill);
}

/// `WorldGen.SquareTileFrame(i, j)` with the default `resetFrame: true`
/// (`WorldGen.cs:80185-80196`) — the wiring devices' post-toggle wake
/// (`Wiring.cs:1009`, `:1045`, `:1641`-area, `:2385`-area): the 3×3
/// `TileFrame` walk whose `Check*` chain may pop the tile's object, the
/// produced drops handed to the app layer.
pub fn square_tile_frame_default(world: &mut World, x: i32, y: i32) -> Vec<ObjectDrop> {
    let mut destroy_object = false;
    let mut kill = TileKill {
        kind: world.tile(x, y).map(|t| t.kind).unwrap_or(0),
        ..Default::default()
    };
    square_tile_frame(world, x, y, false, &mut destroy_object, &mut kill);
    after_edit(world, x, y);
    kill.object_drops
}

/// The body of `KillTile` plus the shared `WorldGen.destroyObject` flag
/// (:4236). The guarded checks re-entered through the cascade's own `KillTile`
/// calls early-return while the flag is set; the unguarded `CheckAlch`,
/// `CheckOnTable1x1` and `CheckPile`-1x1 run and kill straight through it —
/// exactly like the C# static.
fn kill_tile_cascading(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
) -> Option<TileKill> {
    // :63519-63521 (bounds) and :63529-63532 (inactive) — `remove_tile`
    // covers both; the cell keeps its wall and liquid (:63885-63917 clear
    // everything but wall/liquid state on the C# side).
    //
    // The tree drop record is read BEFORE the cell is cleared: the C# drops
    // the wood while the tile is still active (`KillTile_DropItems` at
    // :63863 runs before the clear at :63885), and `GetTreeBottom` walks the
    // still-standing trunk column.
    let tree_drop = if no_item {
        None
    } else {
        tree_drop_record(world, x, y)
    };
    // W4-d16 residual 9 — the leashed-anchor tile-entity teardown
    // (`WorldGen.cs:63691-63694` `DropItemForTileBreak` + `:63873-63880`
    // `TEKiteAnchor.Kill`/`TECritterAnchor.Kill`): a broken 723/724 anchor
    // drops its stored item (`TELeashedEntityAnchorWithItem.cs:20-30` —
    // `Item.NewItem` at the cell, `itemType > 0` only) and removes the
    // entity. The C# splits the drop (`:63691`, pre-clear, `!fail` ONLY —
    // the `noItem` gate at `:63860` covers `KillTile_DropItems`, not this
    // arm) from the entity kill (`:63873`, post-`KillTile_DropItems`);
    // nothing between reads the entity, so the port folds both into one
    // `remove_at` ahead of the cell clear. The drop rides `object_drops`
    // (the app's `break_drop_events` spawns `drop.item`); the port's
    // `kill_tile` is the real-break path (`fail = false`), so the drop
    // always fires. The leashed kite/critter despawn (`OnRemoved` →
    // `DespawnLeashedEntity`) is M7-TE-2 — the entity removal stands, the
    // visual despawn unported. Runs only past `remove_tile`'s gates (the
    // C# inactive/unbreakable refusals `:63519-63532` return before
    // `:63691`).
    let kind = remove_tile(world, x, y)?;
    let anchor_drop = anchor_teardown(world, x, y, kind);
    let mut kill = TileKill {
        kind,
        ..Default::default()
    };
    if let Some(event) = tree_drop {
        kill.tree_events.push(event);
    }
    if let Some(drop) = anchor_drop {
        kill.object_drops.push(drop);
    }
    square_tile_frame(world, x, y, no_item, destroy_object, &mut kill);
    // The reframing half of the C# `TileFrame` calls: neighbours re-derive
    // their frames; `frame_important` kinds keep theirs and unsupported
    // torches fall (`reframe_area` = the non-object arm of `TileFrame`).
    after_edit(world, x, y);
    Some(kill)
}

/// W4-d16 residual 9 — the leashed-anchor teardown (the `KillTile` arms
/// `WorldGen.cs:63691-63694` + `:63873-63880`): a broken kite/critter
/// anchor (723/724) removes its tile entity (`TEKiteAnchor.Kill` →
/// `TileEntity.Kill(x, y, type)`, `DataStructures/TileEntity.cs:110-116`
/// — the position lookup + the type match) and drops the stored item
/// (`TELeashedEntityAnchorWithItem.DropItemForTileBreak`,
/// `TELeashedEntityAnchorWithItem.cs:20-30`: `itemType > 0` →
/// `Item.NewItem(…, X*16, Y*16, 16, 16, itemType)` then `itemType = 0`).
/// The 1x1 anchor's entity sits at the cell itself (no frame decode —
/// the C# `TryGetAt(i, j)` reads the raw cell, unlike the item frame's
/// `frameX % 36 / 18` origin walk `:63655`).
fn anchor_teardown(world: &mut World, x: i32, y: i32, kind: u16) -> Option<ObjectDrop> {
    // The C# arms gate on the PRE-clear `tile.type` (`:63691`/`:63873`/
    // `:63877`) — the removed kind carries it.
    if kind != 723 && kind != 724 {
        return None;
    }
    // `TileEntity.Kill`'s type match (`TileEntity.cs:112`): a 723 cell
    // kills only the KiteAnchor entity, a 724 only the CritterAnchor —
    // a mismatch leaves the entity untouched.
    let te_kind = world.tile_entities.get_at(x as i16, y as i16)?.kind;
    if te_kind.tile() != kind {
        return None;
    }
    let (te_kind, data) = world.tile_entities.remove_at(x as i16, y as i16)?;
    match (te_kind, data) {
        (
            crate::tile_entities::TeKind::KiteAnchor
            | crate::tile_entities::TeKind::CritterAnchor,
            crate::tile_entities::TeData::Anchor(item_type),
        ) if item_type > 0 => Some(ObjectDrop {
            tile_kind: te_kind.tile(),
            x,
            y,
            item: Some(item_type),
        }),
        _ => None,
    }
}

/// The tree drop record of the cell about to die — `KillTile_GetItemDrops`'s
/// tree arms (`WorldGen.cs:65214-65266`).
///
/// The trunk kinds (5/596/616/634) run `KillTile_GetTreeDrops`
/// (`:66189-66261`): the family comes from `GetTreeTypeAndTreeBottom` (the
/// ground under the trunk column), the acorn marker is the canopy cell's
/// frame. The palm (323) walks down to the first solid ground and takes the
/// biome wood (`:65243-65261`).
fn tree_drop_record(world: &World, x: i32, y: i32) -> Option<TreeDropEvent> {
    let cell = world.tile(x, y)?;
    if !cell.active() {
        return None;
    }
    let kind = cell.kind;
    match kind {
        5 | 596 | 616 | 634 => {
            let (tree_type, bottom_x, bottom_y) = crate::tree::tree_type_and_bottom(world, x, y);
            Some(TreeDropEvent::Trunk {
                x,
                y,
                frame_x: cell.frame_x,
                frame_y: cell.frame_y,
                tree_type,
                bottom_x,
                bottom_y,
            })
        }
        323 => {
            // `case 323` (`:65232-65266`): the palm wood, then the ground
            // walk down to the first solid tile (`:65243-65261`).
            let mut item = 2504u16;
            let mut j = y;
            while j < world.height as i32 - 1 {
                let Some(t) = world.tile(x, j) else {
                    break;
                };
                if t.active() && td::TILE_SOLID.has(t.kind) {
                    break;
                }
                j += 1;
            }
            if let Some(t) = world.tile(x, j) {
                if t.active() {
                    item = match t.kind {
                        234 => 911, // the crimson sand → Shadewood
                        116 => 621, // the hallow sand → Pearlwood
                        112 => 619, // the ebon sand → Ebonwood
                        _ => item,
                    };
                }
            }
            Some(TreeDropEvent::Palm {
                x,
                y,
                frame_x: cell.frame_x,
                item,
            })
        }
        // The gem trees — `case 589/584/583/586/585/587/588`
        // (`WorldGen.cs:65699-65746`): `SetGemTreeDrops(gemType, seedType, …)`
        // (`:66094-66108`). The gem/seed pair per kind is the C# call site's.
        583..=589 => {
            let (gem, seed) = match kind {
                583 => (180, 4851), // Topaz
                584 => (181, 4852), // Amethyst
                585 => (177, 4853), // Sapphire
                586 => (179, 4854), // Emerald
                587 => (178, 4855), // Ruby
                588 => (182, 4856), // Diamond
                _ => (999, 4857),   // 589 Amber
            };
            Some(TreeDropEvent::Gem {
                x,
                y,
                frame_x: cell.frame_x,
                frame_y: cell.frame_y,
                gem,
                seed,
            })
        }
        // The mushroom tree — `case 71: case 72:` (`:65642-65652`). Only 72
        // reaches the axe arm (`Main.tileAxe[72]`, `Main.cs:7209`); the
        // mushroom plants 71 die through their own support check and share
        // the drop arm.
        72 => Some(TreeDropEvent::Mushroom { x, y }),
        // The cactus — `case 80` (`:64717-64719`): the flat Cactus 276.
        // Recorded per cell (like the trunk kinds) so the `CheckCactus`
        // cascade's kills drop too; the app's `cell_drops` suppresses 80.
        80 => Some(TreeDropEvent::Cactus { x, y }),
        _ => None,
    }
}

/// `WorldGen.SquareTileFrame` (`WorldGen.cs:80185-80196`) with the
/// `TileFrame` guards inline: cells within 5 of the world edge are never
/// framed (:81330), inactive cells return early (:81348-81351), and
/// frame-important object kinds reach `TileFrameImportant`'s `Check*`
/// routines (:81353-81359, dispatch :84962+). Kills found here run their own
/// recursive `KillTile` (vanilla's inner `SquareTileFrame` chain).
fn square_tile_frame(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    for (dx, dy) in SQUARE_FRAME_ORDER {
        let (cx, cy) = (x + dx, y + dy);
        // `TileFrame` guard (:81330) — no framing within 5 tiles of the edge,
        // so no cascade either (objects cannot legitimately stand there:
        // `CanPlace`'s 5-tile margin, `TileObject.cs:215`).
        if cx <= 5 || cy <= 5 || cx >= world.width - 5 || cy >= world.height - 5 {
            continue;
        }
        let Some(cell) = world.tile(cx, cy) else {
            continue;
        };
        // The vanilla's `TileFrameImportant` runs off `Main.tileFrameImportant`
        // alone (`WorldGen.cs:81353-81359`) — the frame-important kinds
        // (the orbs 12/31/639, the altars 26) need no `TileObject` def.
        let important = world
            .frame_important
            .get(cell.kind as usize)
            .copied()
            .unwrap_or(false);
        if !cell.active()
            || (crate::tileobject::def_for_tile(cell.kind).is_none()
                && !important
                // The cactus rides `TileFrame`'s non-solid arm, not the
                // frame-important dispatch (`WorldGen.cs:81361-81374`).
                && cell.kind != 80)
        {
            continue;
        }
        // `TileFrameImportant`'s tree arms (`WorldGen.cs:85876-85911`):
        // `CheckTree` (the forest tree 5), `CheckTreeWithSettings` (the
        // variant kinds 583-589/596/616/634) and `CheckPalmTree` (323). The
        // routines kill the tree cells that lost their support and carry no
        // `WorldGen.destroyObject` guard — the C#'s `TileFrameImportant` does
        // not read the flag for them.
        if let Some(variant) = crate::tree::TreeVariant::of(cell.kind) {
            match variant {
                crate::tree::TreeVariant::Palm => {
                    check_palm_tree(world, cx, cy, no_item, destroy_object, kill)
                }
                _ => check_tree(world, cx, cy, variant, no_item, destroy_object, kill),
            }
            continue;
        }
        // `TileFrameImportant`'s `case 72` (`WorldGen.cs:85837-85874`): the
        // mushroom tree's support check + the top-cell canopy re-roll.
        if cell.kind == 72 {
            check_mushroom_tree(world, cx, cy, no_item, destroy_object, kill);
            continue;
        }
        // `TileFrame`'s non-solid arm (`WorldGen.cs:81361-81374`): the cactus
        // 80 is NOT frame-important, but its `CheckCactus` runs before the
        // cosmetic framing — a cactus that lost its sand dies (and the kill
        // ring reaches its neighbours, so the whole plant collapses).
        if cell.kind == 80 {
            check_cactus(world, cx, cy, no_item, destroy_object, kill);
            continue;
        }
        // `Check*` re-entrancy guard (`WorldGen.destroyObject`, read at
        // :43338/:46976/:49190/:53732/:56800/:57926/:86341/:86434): only the
        // multi-tile object checks carry the entry guard. `CheckAlch`
        // (:45972-46092), `CheckOnTable1x1` (:43503-43546) and `CheckPile`'s
        // 1x1 body (:46875-46971) have none — they run and kill while another
        // object's kill loop is in flight.
        let guarded = destroy_object_guarded(cell);
        if *destroy_object && guarded {
            continue;
        }
        let Some(plan) = check_object(world, cx, cy) else {
            continue;
        };
        if !no_item {
            if let Some(drop) = plan.drop {
                kill.object_drops.push(drop);
            }
            // The herb event stands for the killed cell's `KillTile_DropItems`
            // roll (`WorldGen.cs:63860-63864`), so it carries the same
            // `noItem` gate; the pile loot blocks are `Check2x1`/`Check3x2`
            // -level `Item.NewItem` calls and are not gated (:47130-47245,
            // :49828-49839).
            if let Some(herb) = plan.herb_event {
                kill.herb_events.push(herb);
            }
        }
        if let Some(pile) = plan.pile_event {
            kill.pile_events.push(pile);
        }
        // `SpawnThingsFromPot`'s only gate is `Main.netMode != 1`
        // (:57011-57014) — the `noItem` flag never suppresses pot loot, so
        // the event passes through unchanged.
        if let Some(event) = plan.pot_event {
            kill.events.push(event);
        }
        // The guarded checks' shared tail: `destroyObject = true`, kill every
        // object cell of the type (`WorldGen.cs:57941-57952`, :86414-86428,
        // :86494-86502, :43466-43472, :47108-47115, :49416-49423),
        // `destroyObject = false`. Each `KillTile` is a full recursive kill
        // (inner `SquareTileFrame`), but the flag keeps the re-entered
        // guarded checks silent until the loop ends — a supported object
        // whose support died inside the loop stays put until a later break
        // near it (`WorldGen.cs:49841` resets the flag last in `Check3x2`).
        // The unguarded checks call `KillTile` plainly (:43513-43543, :46091,
        // :46910): the flag passes through unchanged, so a herb or a clay pot
        // dies inside another object's loop while that loop's own silence
        // holds.
        if guarded {
            *destroy_object = true;
            kill_plan_cells(world, &plan.cells, no_item, destroy_object, kill);
            *destroy_object = false;
        } else {
            kill_plan_cells(world, &plan.cells, no_item, destroy_object, kill);
        }
        // The `Check*` tail's `TileFrame` ring (`WorldGen.cs:49841-49849`,
        // :50161-50168, :47499-47506): walked with `destroyObject` reset so a
        // neighbouring piece whose anchor died inside this sweep reaches its
        // own `Check*`. The port caps the recursion depth — the vanilla's
        // ring is a bounded per-check walk and the port's sweep re-enters
        // through `kill_tile_cascading`, so the cap only guards the
        // pathological nests.
        if let Some(((rx0, ry0), (rx1, ry1))) = plan.ring {
            RING_DEPTH.with(|d| {
                let depth = d.get();
                if depth < 8 {
                    d.set(depth + 1);
                    let (mid_y, mid_x) = ((ry0 + ry1) / 2, (rx0 + rx1) / 2);
                    for (mx, my) in [
                        (rx0, ry0),
                        (rx1, ry0),
                        (rx0, ry1),
                        (rx1, ry1),
                        (mid_x, ry0),
                        (mid_x, ry1),
                        (rx0, mid_y),
                        (rx1, mid_y),
                    ] {
                        square_tile_frame(world, mx, my, no_item, destroy_object, kill);
                    }
                    d.set(depth);
                }
            });
        }
    }
}

/// Whether the cell's `Check*` routine carries the `WorldGen.destroyObject`
/// entry guard and wraps its kill loop in the flag. The multi-tile object
/// checks do (`Check1x2` :43337/:43442-43500, `Check2x1` :46976/:47108-47363,
/// `Check3x2` :49190/:49417-49841, `CheckSunflower` :53732/:53779-53791,
/// `CheckPot` :56800/:56855-57015, `CheckChest` :57926/:57970-57986,
/// `CheckDoorOpen` :86341/:86414-86428, `CheckDoorClosed` :86434/:86495-86510);
/// `CheckAlch` (:45972-46092), `CheckOnTable1x1` (:43503-43546) and
/// `CheckPile`'s 1x1 body (:46875-46971) have neither. `CheckPile` hands its
/// 2x1 variant (`frameY == 18`, :46899-46902 — the echo 649 is not in the
/// registry) to the guarded `Check2x1`, so only the 1x1 pile runs unwrapped.
fn destroy_object_guarded(cell: &Tile) -> bool {
    match cell.kind {
        78 | 82 | 83 | 84 => false,
        185 => cell.frame_y == 18,
        _ => true,
    }
}

/// Kills a plan's cells — one recursive `KillTile` per cell — and folds every
/// nested record into `kill`: a nested `KillTile` runs the full cascade in the
/// C#, so its drops, pot loot, herb rolls and pile loot all surface.
fn kill_plan_cells(
    world: &mut World,
    cells: &[(i32, i32)],
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    for &(mx, my) in cells {
        if let Some(inner) = kill_tile_cascading(world, mx, my, no_item, destroy_object) {
            kill.object_drops.extend(inner.object_drops);
            kill.events.extend(inner.events);
            kill.herb_events.extend(inner.herb_events);
            kill.pile_events.extend(inner.pile_events);
            kill.tree_events.extend(inner.tree_events);
            kill.palm_frame_rolls.extend(inner.palm_frame_rolls);
            kill.mushroom_frame_rolls.extend(inner.mushroom_frame_rolls);
        }
    }
}

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:69957-69969`, the indexed
/// overload :70111-70131): an `nactive` (active, non-actuated), solid,
/// non-solidTop, non-halfBrick, slopeless tile. Out-of-world cells are not
/// solid (the C# try/catch turns the OOB into `false`).
pub(crate) fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && !t.inactive()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70185`): an
/// active, solid or solidTop, non-halfBrick tile whose top is not a slope.
/// Out-of-world and never-initialized cells count as solid (`InWorld`/null →
/// `true` — vanilla quirk, kept 1:1). The `PlatformProperTopFrame`
/// refinement only reaches platforms with a top slope, which cannot occur.
pub(crate) fn solid_tile_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && !t.top_slope()
        && !t.half_brick()
}

/// The destruction plan of one `Check*` routine: the object cells to
/// `KillTile` (active cells of the object's type — the exact cell guard each
/// C# kill loop applies), the drop vanilla would spawn (`None` for checks
/// whose loot is per-cell `KillTile_GetItemDrops`, an event, or RNG-deferred —
/// `check_alch`/`check_pot`/`check_pile*`/the piles) and the pot-loot event.
struct CheckPlan {
    drop: Option<ObjectDrop>,
    cells: Vec<(i32, i32)>,
    pot_event: Option<PotEvent>,
    herb_event: Option<HerbEvent>,
    pile_event: Option<PileLootEvent>,
    /// The `Check*` tail's `TileFrame` ring — only the checks that CLOSE with
    /// one in C# (`Check3x2` `WorldGen.cs:49841-49849`, `Check3x4`
    /// :50161-50168, `Check4x2` :47499-47506; the others — CheckOnTable1x1,
    /// Check1xX, Check2xX, Check1x2Top, Check2x2, Check3x3, CheckChest — have
    /// no tail, so their remainders float until a later break, the vanilla
    /// deferred-pop semantics).
    ring: Option<((i32, i32), (i32, i32))>,
}

/// The `TileFrameImportant` object dispatch (`WorldGen.cs:84962`): runs the
/// `Check*` routine for the cell's registry kind and returns its destruction
/// plan. Cell routines: 10 → `CheckDoorClosed` (:86432), 11 →
/// `CheckDoorOpen` (:86339), 13/33/49/78 → `CheckOnTable1x1` (:43503, the
/// 13/33/49/50/78/174/372/646/707 group at :85604-85614), 14/87/88/186/187 →
/// `Check3x2` (:49187, dispatch :85980-85989), 15 → `Check1x2` (:43335),
/// 18 → `Check2x1` (:46973), 21 → `CheckChest` (:57923, via
/// `TileID.Sets.BasicChest` :85616-85619), 27 → `CheckSunflower` (:85639),
/// 28 → `CheckPot` (:85643), 34 → `Check3x3` (:53059), 42 → `Check1x2Top`
/// (:46706), 79/90 → `Check4x2` (:47408), 82/83/84 → `CheckAlch` (the
/// `Main.tileAlch` default arm, :85830-85832), 93 → `Check1xX` (:37788),
/// 100/172 → `Check2x2`'s default (:48397), 101 → `Check3x4` (:50076),
/// 104 → `Check2xX` (:37993), 185 → `CheckPile` (:85275), 219/642 →
/// `Check3x3`'s bottom-support arm (:53059, the `:53114` support group — the
/// M7-EXTRACT-1 extractinators; tile 34's is the ceiling arm, the `else` at
/// `:53129-53139`).
fn check_object(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    match kind {
        10 => check_door_closed(world, x, y),
        11 => check_door_open(world, x, y),
        13 | 33 | 49 | 78 => check_on_table_1x1(world, x, y),
        29 => check_piggy(world, x, y),
        // `WorldGen.cs:85249-85252`'s TileFrameImportant arm: 12 (Crimson
        // Heart), 31 (Shadow Orb), 639 (Mana Crystal).
        12 | 31 | 639 => check_orb(world, x, y),
        14 | 87 | 88 | 186 | 187 => check_3x2(world, x, y, kind),
        15 => check_chair(world, x, y),
        18 => check_bench(world, x, y),
        21 => check_chest(world, x, y),
        27 => check_sunflower(world, x, y),
        28 => check_pot(world, x, y),
        34 => check_chandelier(world, x, y),
        // M7-EXTRACT-1: the extractinators ride `Check3x3`'s bottom-support
        // arm (`WorldGen.cs:53114-53127`, the group the C# lists 219/642 in).
        219 | 642 => check_3x3_bottom(world, x, y, kind),
        42 => check_lantern(world, x, y),
        79 | 90 => check_bed_bathtub(world, x, y),
        82 | 83 | 84 => check_alch(world, x, y),
        93 => check_lamp(world, x, y),
        100 | 172 => check_candelabra_sink(world, x, y),
        101 => check_bookcase(world, x, y),
        104 => check_clock(world, x, y),
        185 => check_pile(world, x, y),
        _ => None,
    }
}

/// The type of a cell, or `-1` when the cell is empty (the C#'s
/// `Main.tile[x, y] != null && Main.tile[x, y].active()` probe).
fn type_at(world: &World, x: i32, y: i32) -> i32 {
    match world.tile(x, y) {
        Some(t) if t.active() => i32::from(t.kind),
        _ => -1,
    }
}

// The recursion cap of the tree cascade: `CheckTree`'s `KillTile` chain walks
// up the trunk column, so the depth is the tree's height (the vanilla's trees
// are ≤ 30 tall). The cap only guards the port's pathological nests.
thread_local! {
    static TREE_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// `WorldGen.CheckTree` (the forest tree, `WorldGen.cs:54570-54780`) and
/// `WorldGen.CheckTreeWithSettings` (the variant kinds, `:54402-54568`).
///
/// The two routines are the same machine with a different ground test: the
/// forest tree folds the grass family onto plain dirt (`:54611-54642`), the
/// variants use `GemTreeGroundTest` (`:30304`), `VanityTreeGroundTest`
/// (`:30317`) and `AshTreeGroundTest` (`:30330`). The port keeps one body and
/// parameterises the ground test.
///
/// The routine's kills are the vanilla's `WorldGen.KillTile(i, j)` — a real
/// kill that drops the cell's wood and re-frames the neighbours, which is what
/// makes the whole trunk collapse when its base goes.
fn check_tree(
    world: &mut World,
    x: i32,
    y: i32,
    variant: crate::tree::TreeVariant,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    let Some(cell) = world.tile(x, y) else {
        return;
    };
    if !cell.active() {
        return;
    }
    let kind = i32::from(cell.kind);
    let fx0 = i32::from(cell.frame_x);
    let fy0 = i32::from(cell.frame_y);

    let left = type_at(world, x - 1, y);
    let right = type_at(world, x + 1, y);
    let above = type_at(world, x, y - 1);
    let below = variant.ground_type(type_at(world, x, y + 1));
    let ground_ok = variant.ground_valid(type_at(world, x, y + 1));

    let mut fx = fx0;
    let mut fy = fy0;

    // (1) The trunk's base lost its ground (`:54643-54646`).
    if !ground_ok
        && below != kind
        && ((fx == 0 && fy <= 130) || (fx == 22 && fy <= 130) || (fx == 44 && fy <= 130))
    {
        kill_tree_cell(world, x, y, no_item, destroy_object, kill);
        // The C#'s later arms read the cleared cell: `KillTile` resets the
        // frames to `-1` (`:63887-63888`).
        fx = -1;
        fy = -1;
    }

    // (2) The branch/root row (`:54647-54657`).
    if (22..=44).contains(&fx) && (132..=176).contains(&fy) {
        if !ground_ok {
            kill_tree_cell(world, x, y, no_item, destroy_object, kill);
            fx = -1;
            fy = -1;
        } else if (fx != 22 || left != kind) && (fx != 44 || right != kind) {
            kill_tree_cell(world, x, y, no_item, destroy_object, kill);
            fx = -1;
            fy = -1;
        }
    } else if (fx == 88 && (0..=44).contains(&fy))
        || (fx == 66 && (66..=130).contains(&fy))
        || (fx == 110 && (66..=110).contains(&fy))
        || (fx == 132 && (0..=176).contains(&fy))
    {
        // (3) The branch's frame follows its neighbours (`:54658-54681`).
        let num5 = fy % 66;
        if left == kind && right == kind {
            fx = 110;
            fy = 66 + num5;
        } else if left == kind {
            fx = 88;
            fy = num5;
        } else if right == kind {
            fx = 66;
            fy = 66 + num5;
        } else {
            fx = 0;
            fy = num5;
        }
    }

    // (4) The root row (`:54682-54709`). The forest tree's frame chain is a
    // separate `if` (it writes the frames even after the kill); the variants'
    // chain is an `else if` (`:54473-54497`).
    if (132..=176).contains(&fy) && (fx == 0 || fx == 66 || fx == 88) {
        let num6 = fy % 66;
        if !ground_ok {
            kill_tree_cell(world, x, y, no_item, destroy_object, kill);
            fx = -1;
            fy = -1;
        }
        if variant == crate::tree::TreeVariant::Forest || ground_ok {
            if left != kind && right != kind {
                fx = 0;
                fy = num6;
            } else if left != kind {
                fx = 0;
                fy = 132 + num6;
            } else if right != kind {
                fx = 66;
                fy = 132 + num6;
            } else {
                fx = 88;
                fy = 132 + num6;
            }
        }
    }

    // (5) The canopy marker without its trunk column (`:54710-54727`). The
    // forest tree's `num4 == 23` arm is dead code — the ground fold at
    // `:54611-54614` already turned 23 into 2.
    if (fx == 66 && (fy == 0 || fy == 22 || fy == 44))
        || (fx == 44 && (fy == 198 || fy == 220 || fy == 242))
    {
        if right != kind {
            kill_tree_cell(world, x, y, no_item, destroy_object, kill);
            fx = -1;
            fy = -1;
        }
    } else if (fx == 88 && (fy == 66 || fy == 88 || fy == 110))
        || (fx == 66 && (fy == 198 || fy == 220 || fy == 242))
    {
        if left != kind {
            kill_tree_cell(world, x, y, no_item, destroy_object, kill);
            fx = -1;
            fy = -1;
        }
    } else if below == -1 {
        kill_tree_cell(world, x, y, no_item, destroy_object, kill);
        fx = -1;
        fy = -1;
    } else if above != kind && fy < 198 && ((fx != 22 && fx != 44) || fy < 132) {
        // (6) The trunk's frame follows its neighbours (`:54728-54772`).
        let num7 = fy % 66;
        if left == kind || right == kind {
            if below == kind {
                if left == kind && right == kind {
                    fx = 132;
                    fy = 132 + num7;
                } else if left == kind {
                    fx = 132;
                    fy = num7;
                } else {
                    fx = 132;
                    fy = 66 + num7;
                }
            } else if left == kind && right == kind {
                fx = 154;
                fy = 132 + num7;
            } else if left == kind {
                fx = 154;
                fy = num7;
            } else {
                fx = 154;
                fy = 66 + num7;
            }
        } else {
            fx = 110;
            fy = num7;
        }
    }

    // The rewritten frames go back to the cell (the C# writes them into
    // `Main.tile[i, j]`).
    if fx != fx0 || fy != fy0 {
        if let Some(cell) = world.tile_mut(x, y) {
            cell.frame_x = fx as i16;
            cell.frame_y = fy as i16;
        }
    }

    // (7) The frame changed → re-frame the neighbours (`:54773-54779`).
    if fx != fx0 && fy != fy0 && fx0 >= 0 && fy0 >= 0 {
        tree_ring(world, x, y, no_item, destroy_object, kill);
    }
}

/// `WorldGen.CheckPalmTree` (`WorldGen.cs:54346-54400`): the palm's ground
/// must be sand; a palm that lost it dies, and the palm's canopy frame is
/// re-rolled when the cell above it is gone.
fn check_palm_tree(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    let Some(cell) = world.tile(x, y) else {
        return;
    };
    if !cell.active() {
        return;
    }
    let kind = i32::from(cell.kind);
    let fx0 = i32::from(cell.frame_x);
    let fy0 = i32::from(cell.frame_y);
    let above = type_at(world, x, y - 1);
    // The evil and hallowed sands read as plain sand (`:54361-54376`).
    let mut below = type_at(world, x, y + 1);
    if below == 234 || below == 116 || below == 112 {
        below = 53;
    }
    let mut fx = fx0;
    let mut fy = fy0;
    // The palm's ground must be sand (`:54377-54384`).
    if below != 53 && below != kind {
        kill_tree_cell(world, x, y, no_item, destroy_object, kill);
        fx = -1;
        fy = -1;
    }
    if (fx == 66 || fx == 220) && below != 53 {
        kill_tree_cell(world, x, y, no_item, destroy_object, kill);
        fx = -1;
        fy = -1;
    }
    if above != kind && fx <= 44 {
        // `Main.tile[i, j].frameX = (short)(WorldGen.genRand.Next(7, 10) * 22);`
        // (`:54385-54388`) — the roll needs the `genRand` stream, so the port
        // records the request and the app resolves it (the frame is written
        // after the cascade; the C#'s ring does not fire for this arm because
        // `frameY` is unchanged).
        kill.palm_frame_rolls.push((x, y));
    } else if above != kind && fx == 66 {
        fx = 220;
    }
    if fx != fx0 || fy != fy0 {
        if let Some(cell) = world.tile_mut(x, y) {
            cell.frame_x = fx as i16;
            cell.frame_y = fy as i16;
        }
    }
    // The frame changed → re-frame the neighbours (`:54393-54399`).
    if fx != fx0 && fy != fy0 && fx0 >= 0 && fy0 >= 0 {
        tree_ring(world, x, y, no_item, destroy_object, kill);
    }
}

/// The `CheckTree`/`CheckPalmTree` tail's `TileFrame` ring
/// (`WorldGen.cs:54773-54779`, `:54393-54399`): the four neighbours of the
/// re-framed cell.
fn tree_ring(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    let depth = TREE_DEPTH.with(|d| d.get());
    if depth >= 64 {
        return;
    }
    TREE_DEPTH.with(|d| d.set(depth + 1));
    for (mx, my) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
        square_tile_frame(world, mx, my, no_item, destroy_object, kill);
    }
    TREE_DEPTH.with(|d| d.set(depth));
}

/// Kills a tree cell the way the C#'s `CheckTree` does — `WorldGen.KillTile(i, j)`
/// — and folds the nested record into `kill`: the nested kill runs the whole
/// cascade (its own drops, pot loot, herb rolls and pile loot all surface).
fn kill_tree_cell(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    if let Some(inner) = kill_tile_cascading(world, x, y, no_item, destroy_object) {
        kill.object_drops.extend(inner.object_drops);
        kill.events.extend(inner.events);
        kill.herb_events.extend(inner.herb_events);
        kill.pile_events.extend(inner.pile_events);
        kill.tree_events.extend(inner.tree_events);
        kill.palm_frame_rolls.extend(inner.palm_frame_rolls);
        kill.mushroom_frame_rolls.extend(inner.mushroom_frame_rolls);
    }
}

/// `TileFrameImportant`'s `case 72` (`WorldGen.cs:85837-85874`): the mushroom
/// tree's cell dies when the cell below is neither another 72 nor the
/// mushroom grass 70; the top cell (the one whose above is not a 72, with
/// `frameX == 0`) re-rolls its canopy frame — `genRand.Next(3)` picks
/// (18, 0), (18, 18) or (18, 36). The roll needs the `genRand` stream, so the
/// core records the request and the app resolves it (the palm precedent).
fn check_mushroom_tree(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    let Some(cell) = world.tile(x, y) else {
        return;
    };
    if !cell.active() || cell.kind != 72 {
        return;
    }
    let kind = i32::from(cell.kind);
    let fx = i32::from(cell.frame_x);
    let below = type_at(world, x, y + 1);
    let above = type_at(world, x, y - 1);
    // `:85851-85854`: `num9 != type && num9 != 70` → `KillTile(i, j)`.
    if below != kind && below != 70 {
        kill_tree_cell(world, x, y, no_item, destroy_object, kill);
        return;
    }
    // `:85855-85873`: the top cell's canopy re-roll.
    if above != kind && fx == 0 {
        kill.mushroom_frame_rolls.push((x, y));
    }
}

/// `WorldGen.CheckCactus` (`WorldGen.cs:54116-54158`): walk DOWN the cactus
/// column (leaning one column toward a side arm whose own base continues
/// down), then the ground test — the base cell must be a full (nactive, no
/// halfBrick, slopeless) sand tile (53/112/116/234), else the STARTING cell
/// dies. A side-arm cell (`i != num2`) dies when nothing cactus continues
/// below/left/right; the trunk-base cell dies when the cell below is neither
/// cactus nor sand. Returns `true` when the cell was killed (the C#'s
/// `TileFrame` early-return, `:81369-81372`).
fn check_cactus(
    world: &mut World,
    i: i32,
    j: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) -> bool {
    let is_cactus = |w: &World, x: i32, y: i32| {
        w.tile(x, y).is_some_and(|t| t.active() && t.kind == 80)
    };
    let mut num = j; // y walker
    let mut num2 = i; // x walker
    // `:54120-54138`: down while the cell is an active cactus; at the first
    // non-cactus cell, lean toward a side arm whose column continues down.
    while is_cactus(world, num2, num) {
        num += 1;
        if world.tile(num2, num).is_none() {
            return false;
        }
        if !is_cactus(world, num2, num) {
            if is_cactus(world, num2 - 1, num) && is_cactus(world, num2 - 1, num - 1) && num2 >= i
            {
                num2 -= 1;
            }
            if is_cactus(world, num2 + 1, num) && is_cactus(world, num2 + 1, num - 1) && num2 <= i
            {
                num2 += 1;
            }
        }
    }
    // `:54139-54143`: the base's ground test.
    let base_ok = world.tile(num2, num).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && !t.half_brick()
            && t.slope() == 0
            && matches!(t.kind, 53 | 112 | 116 | 234)
    });
    if !base_ok {
        kill_tree_cell(world, i, j, no_item, destroy_object, kill);
        return true;
    }
    if i != num2 {
        // `:54144-54151`: the side-arm cell — dies when no cactus neighbour
        // continues it (below, left, right).
        if !is_cactus(world, i, j + 1)
            && !is_cactus(world, i - 1, j)
            && !is_cactus(world, i + 1, j)
        {
            kill_tree_cell(world, i, j, no_item, destroy_object, kill);
            return true;
        }
    } else {
        // `:54152-54156`: the trunk base — the cell below must be cactus or
        // sand.
        let below_ok = world.tile(i, j + 1).is_some_and(|t| {
            t.active() && matches!(t.kind, 80 | 53 | 112 | 116 | 234)
        });
        if !below_ok {
            kill_tree_cell(world, i, j, no_item, destroy_object, kill);
            return true;
        }
    }
    false
}

/// `WorldGen.CheckOrb` (`WorldGen.cs:54169-54344`) — the 2x2 object arm shared
/// by the Shadow Orb (31), the Crimson Heart (12, the +36 frameX) and
/// the Mana Crystal (639): the top-left anchor snap, the integrity scan,
/// the break plan, and the world-state flags (`shadowOrbSmashed =
/// true`, the shared `shadowOrbCount++`). The 5-way drop tables
/// (`:54226-54290`) need the two-item `ObjectDrop` extension — the
/// documented follow-up; the boss-summon edge (`:54294-54321`'s
/// `count >= 3`) rides the sim pass (the app's `update_npcs` reads the
/// count delta).
fn check_orb(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let kind = t.kind;
    let frame_x = t.frame_x as i32;
    let frame_y = t.frame_y as i32;
    // `:54185-54188`'s anchor: the 2x2's top-left — frameX 0/36 is the
    // left column, 18/54 the right; frameY 18 is the bottom row.
    let ox = x - (if frame_x == 18 || frame_x == 54 { 1 } else { 0 });
    let oy = y - (if frame_y != 0 { 1 } else { 0 });
    let crimson = frame_x >= 36;
    let mut broken = false;
    let mut cells = Vec::with_capacity(4);
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            cells.push((k, l));
            let expected_x = (if k == ox { 0 } else { 18 }) + if crimson { 36 } else { 0 };
            let expected_y = if l == oy { 0 } else { 18 };
            let ok = world.tile(k, l).is_some_and(|c| {
                c.active()
                    && c.kind == kind
                    && c.frame_x as i32 == expected_x
                    && c.frame_y as i32 == expected_y
            });
            if !ok {
                broken = true;
            }
        }
    }
    if !broken {
        // `:54213-54214`: the intact object returns without a plan — the
        // break that triggered this check is a different cell.
        return None;
    }
    // `:54226-54290`'s drop switch: the 5-way roll, the forced case 0
    // until the world's first smash (`:54240`/`:54268`'s `!shadowOrbSmashed`
    // arm). The port rolls the deterministic position seed (the vanilla's
    // `Main.rand`); the case-0 Musket Ball stack rolls into the documented
    // note (the ObjectDrop carries the single item).
    let mut seed = crate::flame::frame_seed_for(0x5EAF_00D5, ox, oy);
    let forced = !world.shadow_orb_smashed;
    let pick = if forced {
        0
    } else {
        crate::flame::random_int(&mut seed, 0, 5)
    };
    let item = if crimson {
        match pick {
            0 => Some(800),  // The Undertaker
            1 => Some(1256), // Crimson Rod
            2 => Some(802),  // The Rotted Fork
            3 => Some(3062), // Crimson Heart (the pet item)
            _ => Some(1290), // Panic Necklace
        }
    } else if kind == 639 {
        Some(109) // the Mana Crystal
    } else {
        match pick {
            0 => Some(96),  // The Musket
            1 => Some(64),  // Vilethorn
            2 => Some(162), // Ball O' Hurt
            3 => Some(115), // Shadow Orb (the pet item)
            _ => Some(111), // Band of Starpower
        }
    };
    // The 2x2's drop position: the orb's centre tile (`Item.NewItem`'s
    // spawn at the object centre).
    let cx = ox + 1;
    let cy = oy + 1;
    // `:54292-54293`'s flags (outside the drop switch — all three kinds
    // count toward the same world counter).
    world.shadow_orb_smashed = true;
    world.shadow_orb_count += 1;
    Some(CheckPlan {
        drop: Some(crate::edit::ObjectDrop {
            tile_kind: kind,
            x: cx,
            y: cy,
            item,
        }),
        cells,
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
    })
}

/// `WorldGen.Check2x1`'s case-29 arm (`WorldGen.cs:47003-47013`) for the
/// piggy bank: the 2x1 pair with the second cell's `frameX == first + 18`
/// (:47000-47008), and BOTH bottom cells must hold an active table tile
/// (`Main.tileTable[type]`) without halfBrick/topSlope — the case-29 arm
/// checks `Main.tileTable` directly, no `AnchorValid` fallback (unlike the
/// case-103 arm :46988-46998). The port's placement anchor is broader
/// (`AnchorType.Table` accepts platforms, `TileObject.cs:392-409`) — the
/// C# kill is stricter than its own placement, ported verbatim.
fn check_piggy(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let ox = x - frame_x / 18 % 2;
    let left = world.tile(ox, y);
    let right = world.tile(ox + 1, y);
    let left_x = left.map(|l| l.frame_x as i32).unwrap_or(i32::MIN);
    // :47000-47008: the pair intact = the right cell's frameX is first+18
    // and both cells are the same active kind.
    let intact = right.is_some_and(|c| c.frame_x as i32 == left_x + 18)
        && left.is_some_and(|c| c.active() && c.kind == 29)
        && right.is_some_and(|c| c.active() && c.kind == 29)
        // :47003-47013: both bottom cells on a table.
        && world
            .tile(ox, y + 1)
            .is_some_and(|b| b.active() && crate::tileobject::is_table(b.kind) && !b.half_brick() && !b.top_slope())
        && world
            .tile(ox + 1, y + 1)
            .is_some_and(|b| b.active() && crate::tileobject::is_table(b.kind) && !b.half_brick() && !b.top_slope());
    if intact {
        return None;
    }
    let region = [(ox, y), (ox + 1, y)];
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 29,
            x: ox,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 29),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s Check2x1 tail).
    })
}

/// `WorldGen.Check1xX` (`WorldGen.cs:37788-37878`) for the lamp (93): three
/// cells with the style-row frames and a solid floor below.
fn check_lamp(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let height = 3; // the lamppost's 6 waits for its registry entry
    let frame_x = world.tile(x, y)?.frame_x;
    let mut num2 = world.tile(x, y)?.frame_y as i32;
    let mut style_row = 0i32;
    while num2 >= 18 * height {
        num2 -= 18 * height;
        style_row += 1;
    }
    let oy = y - num2 / 18;
    let mut broken = false;
    let mut region = Vec::with_capacity(height as usize);
    for l in oy..oy + height {
        region.push((x, l));
        // The C# scan sets `flag` and CONTINUES (`WorldGen.cs:37809-37827`).
        let broken_cell = world.tile(x, l).is_none_or(|c| {
            !c.active()
                || c.kind != kind
                || c.frame_y != ((l - oy) * 18 + style_row * height * 18) as i16
                || c.frame_x != frame_x
        });
        if broken_cell {
            broken = true;
        }
    }
    if !solid_tile_allow_bottom_slope(world, x, oy + height) {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_lamp).
    })
}

/// `WorldGen.Check2xX` (`WorldGen.cs:37993-38147`) for the grandfather clock
/// (104): the 2-wide column pair, five cells tall, the style column from
/// `frameX % 36`, and the two solid floor tiles below.
fn check_clock(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x;
    let mut ox = x;
    if frame_x % 36 == 18 {
        ox -= 1;
    }
    let mut num3 = world.tile(ox, y)?.frame_y as i32 / 18;
    let height = 5;
    let mut style_offset = 0i32;
    while num3 >= height {
        num3 -= height;
        style_offset += height * 18;
    }
    let oy = y - num3;
    let top_frame_x = world.tile(ox, y)?.frame_x;
    let mut broken = false;
    let mut region = Vec::with_capacity(10);
    for l in oy..oy + height {
        for (dx, fx) in [(0i32, top_frame_x), (1, top_frame_x + 18)] {
            let k = ox + dx;
            region.push((k, l));
            // The C# scan sets `flag` and CONTINUES — no early break, or the
            // sweep's cell list loses the rest of the row
            // (`WorldGen.cs:38003-38040`).
            let broken_cell = world.tile(k, l).is_none_or(|c| {
                !c.active()
                    || c.kind != kind
                    || c.frame_y != ((l - oy) * 18 + style_offset) as i16
                    || c.frame_x != fx
            });
            if broken_cell {
                broken = true;
            }
        }
    }
    if !solid_tile_allow_bottom_slope(world, ox, oy + height)
        || !solid_tile_allow_bottom_slope(world, ox + 1, oy + height)
    {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_clock).
    })
}

/// `WorldGen.Check3x4` (`WorldGen.cs:50076-50162`) for the bookcase (101).
fn check_bookcase(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let mut style_col = t.frame_x as i32 / 18;
    let mut style_row = t.frame_y as i32 / 18;
    let mut sx = 0i32;
    let mut sy = 0i32;
    while style_col >= 3 {
        style_col -= 3;
        sx += 1;
    }
    while style_row >= 4 {
        style_row -= 4;
        sy += 1;
    }
    let ox = x - style_col;
    let oy = y - style_row;
    let mut broken = false;
    let mut region = Vec::with_capacity(12);
    for k in ox..ox + 3 {
        for l in oy..oy + 4 {
            region.push((k, l));
            let expected = (
                (sx * 54 + (k - ox) * 18) as i16,
                (sy * 72 + (l - oy) * 18) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        if !solid_tile_allow_bottom_slope(world, k, oy + 4) {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: Some(region_ring(&region)),
    })
}

/// `WorldGen.Check1x2Top` (`WorldGen.cs:46706-46776`) for the hanging lantern
/// (42): the two cells carry the style-row frames and the tile ABOVE is a
/// solid (or a platform — the placement alternate's anchor).
fn check_lantern(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let mut num2 = world.tile(x, y)?.frame_y as i32 / 18;
    let mut style_row = 0i32;
    while num2 >= 2 {
        num2 -= 2;
        style_row += 1;
    }
    let oy = y - num2;
    let top_frame = (style_row * 36) as i16;
    let mut broken = false;
    let mut region = vec![(x, oy), (x, oy + 1)];
    let top = world.tile(x, oy);
    let bot = world.tile(x, oy + 1);
    if !top.is_some_and(|c| c.active() && c.kind == kind && c.frame_y == top_frame)
        || !bot.is_some_and(|c| c.active() && c.kind == kind && c.frame_y == top_frame + 18)
    {
        broken = true;
    }
    // The anchor: solid above; the platform flag mirrors the placement
    // alternate (`flag4`, `WorldGen.cs:46752-46756` — the rope branch is the
    // Dead-Cells jar's, not the lantern's).
    let above = world.tile(x, oy - 1);
    let platform_ok =
        above.is_some_and(|a| a.active() && (td::SET_PLATFORMS.has(a.kind) || a.kind == 380));
    let solid_ok = above.is_some_and(|a| {
        (a.active() || a.has_any_liquid())
            && td::TILE_SOLID.has(a.kind)
            && !td::TILE_SOLID_TOP.has(a.kind)
    });
    if !platform_ok && !solid_ok {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_lantern).
    })
}

/// `WorldGen.Check4x2` (`WorldGen.cs:47408-47500`) for the bed (79) and the
/// bathtub (90): the four cells carry the style frames (the second style
/// column continues at frameX 72) and the floor row is solid-or-platform.
fn check_bed_bathtub(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let mut ox = x + frame_x / 18 * -1;
    if (kind == 79 || kind == 90) && frame_x >= 72 {
        ox += 4;
    }
    let mut num2 = t.frame_y as i32 / 18;
    let mut style_row = 0i32;
    while num2 > 1 {
        num2 -= 2;
        style_row += 1;
    }
    let oy = y - num2;
    let second_col = frame_x >= 72;
    let mut broken = false;
    let mut region = Vec::with_capacity(8);
    for k in ox..ox + 4 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let fx = if second_col {
                (k - ox + 4) * 18
            } else {
                (k - ox) * 18
            };
            let expected = (fx as i16, ((l - oy) * 18 + style_row * 36) as i16);
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // `SolidTileAllowBottomSlope` or a platform (`:47456-47463`).
        let floor = world.tile(k, oy + 2);
        let floor_ok = floor.is_some_and(|f| {
            solid_tile_allow_bottom_slope(world, k, oy + 2)
                || (f.active() && td::SET_PLATFORMS.has(f.kind))
        });
        if !floor_ok {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: Some(region_ring(&region)),
    })
}

/// `WorldGen.Check2x2`'s default branch (`WorldGen.cs:48397-48470`) for the
/// candelabra (100) and the sink (172): the 2x2 cells with the style frames
/// (the sink's full height is 38) and the solid-or-table floor without
/// half-bricks.
fn check_candelabra_sink(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let full = if kind == 172 { 38 } else { 36 };
    let mut num = 0i32;
    let mut ox = x - t.frame_x as i32 / 18;
    if ox < x - 1 {
        ox += 2;
        num = 36;
    }
    let mut num4 = t.frame_y as i32;
    let mut style_row = 0i32;
    while num4 >= full {
        num4 -= full;
        style_row += 1;
    }
    let oy = y - num4 / 18;
    let mut broken = false;
    let mut region = Vec::with_capacity(4);
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + num) as i16,
                ((l - oy) * 18 + style_row * full) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // `:48447-48456`: solid or `Main.tileTable`, no half-bricks.
        let floor = world.tile(k, oy + 2);
        let floor_ok = floor.is_some_and(|f| {
            f.active() && (td::TILE_SOLID.has(f.kind) || crate::tileobject::is_tile_table(f.kind))
        }) && !floor.is_some_and(|f| f.half_brick());
        if !floor_ok {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_candelabra_sink).
    })
}

/// `WorldGen.Check3x3`'s bottom-support arm (`WorldGen.cs:53059-53158`) —
/// the group the C# lists at `:53114` (106/212/219/220/228/231/243/247/283,
/// 300-308, 354/355/406/412/452/455/491/642/733): the nine cells must carry
/// the matching kind and the style frames, and all three cells of the row
/// below the object must be `SolidTileAllowBottomSlope` (`:53116-53127`).
/// Otherwise `destroyObject` kills the whole 3x3 and the object's item drops at
/// the trigger cell (`:53159-53175`, 219 → 997, 642 → 5296). The frame check is
/// the same math `check_chandelier` ports.
fn check_3x3_bottom(world: &mut World, x: i32, y: i32, kind: u16) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let mut style_col = t.frame_x as i32 / 18;
    let mut sx = 0i32;
    while style_col >= 3 {
        style_col -= 3;
        sx += 1;
    }
    let ox = x - style_col;
    let style_x = 54 * sx;
    let style_row = t.frame_y as i32 / 54;
    let oy = y - (t.frame_y as i32 % 54 / 18);
    let mut broken = false;
    let mut region = Vec::with_capacity(9);
    for k in ox..ox + 3 {
        for l in oy..oy + 3 {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + style_x) as i16,
                ((l - oy) * 18 + style_row * 54) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
    }
    // `:53116-53127` — the three cells below the object.
    for n in ox..ox + 3 {
        let supported = world.tile(n, oy + 3).is_some_and(|c| {
            (c.active() || c.has_any_liquid())
                && td::TILE_SOLID.has(c.kind)
                && !td::TILE_SOLID_TOP.has(c.kind)
        });
        if !supported {
            broken = true;
            break;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            // `:53170-53175` — the item per tile kind (219 → 997, 642 → 5296).
            item: Some(if kind == 219 { 997 } else { 5296 }),
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s Check3x3 has no tail).
    })
}

/// `WorldGen.Check3x3`'s else branch (`WorldGen.cs:53059-53145`) for the
/// chandelier (34): the nine cells carry the style frames and the ceiling
/// tile above the middle column is a solid non-solid-top.
fn check_chandelier(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let mut style_col = t.frame_x as i32 / 18;
    let mut sx = 0i32;
    while style_col >= 3 {
        style_col -= 3;
        sx += 1;
    }
    let ox = x - style_col;
    let style_x = 54 * sx;
    let style_row = t.frame_y as i32 / 54;
    let oy = y - (t.frame_y as i32 % 54 / 18);
    let mut broken = false;
    let mut region = Vec::with_capacity(9);
    for k in ox..ox + 3 {
        for l in oy..oy + 3 {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + style_x) as i16,
                ((l - oy) * 18 + style_row * 54) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
    }
    // `:53117-53124`: the ceiling anchor above the origin column.
    let ceil = world.tile(ox + 1, oy - 1);
    let ceil_ok = ceil.is_some_and(|c| {
        (c.active() || c.has_any_liquid())
            && td::TILE_SOLID.has(c.kind)
            && !td::TILE_SOLID_TOP.has(c.kind)
    });
    if !ceil_ok {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_chandelier).
    })
}

/// The `Check*` tail's `TileFrame` ring: the piece's bbox expanded by one.
fn region_ring(region: &[(i32, i32)]) -> ((i32, i32), (i32, i32)) {
    let (mut x0, mut y0) = (i32::MAX, i32::MAX);
    let (mut x1, mut y1) = (i32::MIN, i32::MIN);
    for (x, y) in region {
        x0 = x0.min(*x);
        y0 = y0.min(*y);
        x1 = x1.max(*x);
        y1 = y1.max(*y);
    }
    (x0 - 1, y0 - 1);
    (x1 + 1, y1 + 1);
    ((x0 - 1, y0 - 1), (x1 + 1, y1 + 1))
}

/// Cells of `region` that are active and of `kind` — the cell guard shared by
/// every C# kill loop.
fn killable_cells(world: &World, region: &[(i32, i32)], kind: u16) -> Vec<(i32, i32)> {
    region
        .iter()
        .copied()
        .filter(|&(x, y)| {
            world
                .tile(x, y)
                .is_some_and(|t| t.active() && t.kind == kind)
        })
        .collect()
}

/// `WorldGen.CheckDoorClosed` (`WorldGen.cs:86432-86511`): the 3-tall closed
/// door needs `SolidTile` above and below its column and all three cells
/// active type-10. The drop is `DropDoorItem(i, j)` (:86508) — 16x16 at the
/// trigger cell; the style-0 door item is 25 (`GetDoorItem`, :69200-69204,
/// `doorStyle = frameY / 54 + frameX / 54 * 36` :86438 — the port's registry
/// slice is style 0).
fn check_door_closed(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let oy = y - (t.frame_y as i32 % 54) / 18; // :86439
    let broken = !solid_tile(world, x, oy - 1)
        || !solid_tile(world, x, oy + 3)
        || (0..3).any(|dy| {
            world
                .tile(x, oy + dy)
                .is_none_or(|c| !c.active() || c.kind != 10)
        });
    if !broken {
        return None;
    }
    let region: Vec<(i32, i32)> = (0..3).map(|dy| (x, oy + dy)).collect();
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 10,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 10),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_door_closed).
    })
}

/// `WorldGen.CheckDoorOpen` (`WorldGen.cs:86339-86430`): the open door is a
/// 2x3 region; the hinge column follows the `frameX % 72` switch
/// (:86355-86368), solidity is checked above/below the hinge
/// (:86376-86387), and all six cells must be active type-11 (:86396-86412).
/// Drop = `DropDoorItem(i, j)` (:86415), 16x16 at the trigger cell.
fn check_door_open(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let oy = y - (t.frame_y as i32 % 54) / 18; // :86349
    let (hinge, leftmost) = match t.frame_x as i32 % 72 {
        0 => (x, x),
        18 => (x - 1, x - 1),
        36 => (x + 1, x),
        54 => (x, x - 1),
        // Valid open-door frames are 18 px multiples; anything else degrades
        // to the first switch case (the C# would read an uninitialised `num`).
        _ => (x, x),
    };
    let mut broken = !solid_tile(world, hinge, oy - 1) || !solid_tile(world, hinge, oy + 3);
    let mut region = Vec::with_capacity(6);
    for mx in leftmost..leftmost + 2 {
        for my in oy..oy + 3 {
            region.push((mx, my));
            if world
                .tile(mx, my)
                .is_none_or(|c| !c.active() || c.kind != 11)
            {
                broken = true;
            }
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 11,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 11),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_door_open).
    })
}

/// `WorldGen.Check3x2` (`WorldGen.cs:49187-49430`) for the table family —
/// 14 tables, 87 pianos, 88 dressers — and the large piles (186/187 —
/// dispatch `WorldGen.cs:85980-85989`): origin from `frameY % 36 / 18` and
/// `frameX / 18` mod 3 (:49199-49209 — the hardcoded 36 is the `Style3x2`
/// full height; the family's styles are horizontal with `StyleWrapLimit = 0`,
/// so `frameY` only ever encodes the row), every cell must be active of
/// `kind` with the expected frames (:49219-49226), and the per-column support
/// judge branches on the kind (:49280-49373): 186/187 refuse
/// `SolidTileAllowBottomSlope` failures, Boulders
/// (`InvalidTileForPilesOrSpeleothems` :38979-38994, set `TileID.cs:504`) and
/// each style's ground family; the table keeps the default
/// `SolidTileAllowBottomSlope` arm (the trailing `else if`, :49396-49399).
/// Table style 25 spans a single row (`flag2`, :49223-49229).
/// The 187→186 conversion (:49379-49393) runs before the early return: an
/// intact snowy 187 over non-dirt ground becomes a 186 (`frameX -= 378` on
/// all six cells) and survives — the kill loop below then matches zero cells
/// of `kind`. Drop: only the table family carries one —
/// `GetItemDrop_Tables(style)` for 14 (:49434, 32x32 at the trigger cell;
/// style 0 → 32 :42194), `GetItemDrop_Pianos/Dressers(style)` for 87/88
/// (:49693-49696); the piles' case group is empty (:65759+).
fn check_3x2(world: &mut World, x: i32, y: i32, kind: u16) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let frame_y = t.frame_y as i32;
    let oy = y - frame_y % 36 / 18;
    let style_row = frame_y / 36;
    let mut dx = frame_x / 18;
    let mut style = 0;
    while dx > 2 {
        dx -= 3;
        style += 1;
    }
    let ox = x - dx;
    // `flag2` (:49223-49229): table style 25 is a single-row 3x1.
    let rows = if kind == 14 && style == 25 { 1 } else { 2 };
    let mut broken = false;
    let mut region = Vec::with_capacity(6);
    for k in ox..ox + 3 {
        for l in oy..oy + rows {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + style * 54) as i16,
                ((l - oy) * 18 + style_row * 36) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // The per-column support judge (:49280-49373 + the trailing
        // `else if` :49396-49399). The 186/ style sets re-judge the ground
        // family from `TileID.Sets` (`TileID.cs:89-94/558`).
        match kind {
            186 => {
                if !solid_tile_allow_bottom_slope(world, k, oy + rows)
                    || invalid_tile_for_piles(world, k, oy + rows)
                {
                    broken = true;
                } else if let Some(g) = world.tile(k, oy + rows).filter(|g| g.active()) {
                    match style {
                        26..=31 => {
                            if !(td::SET_ICES_SNOW.has(g.kind) || g.kind == 162 || g.kind == 224) {
                                broken = true;
                            }
                        }
                        32..=34 => {
                            if !(td::SET_MUD.has(g.kind) || g.kind == 70) {
                                broken = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            187 => {
                if !solid_tile_allow_bottom_slope(world, k, oy + rows)
                    || invalid_tile_for_piles(world, k, oy + rows)
                {
                    broken = true;
                } else if let Some(g) = world.tile(k, oy + rows).filter(|g| g.active()) {
                    match style {
                        0..=5 => {
                            if !(td::SET_MUD.has(g.kind) || g.kind == 60 || g.kind == 226) {
                                broken = true;
                            }
                        }
                        6..=8 => {
                            if !matches!(g.kind, 57 | 58 | 75 | 76) {
                                broken = true;
                            }
                        }
                        29..=34 => {
                            if !(td::SET_CONVERSION_SAND.has(g.kind)
                                || td::SET_CONVERSION_HARDENED_SAND.has(g.kind)
                                || td::SET_CONVERSION_SANDSTONE.has(g.kind))
                            {
                                broken = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if !solid_tile_allow_bottom_slope(world, k, oy + rows) {
                    broken = true;
                }
            }
        }
    }
    // The 187→186 conversion (:49379-49393): the origin's `frameX` in
    // 756..=900 (the snowy LargePiles2 styles) and none of the six ground
    // cells is dirt/grass — every cell becomes a 186 with `frameX -= 378`.
    // Runs even when `flag` is set; the kill loop's kind filter then spares
    // the converted cells, exactly like the C# `type == 187` test.
    if kind == 187
        && world
            .tile(ox, oy)
            .is_some_and(|c| (756..=900).contains(&i32::from(c.frame_x)))
        && (0..3).all(|d| {
            world
                .tile(ox + d, oy + rows)
                .is_none_or(|g| !matches!(g.kind, 2 | 477 | 492))
        })
    {
        for l in oy..oy + rows {
            for k in ox..ox + 3 {
                let cell = world.tile_mut(k, l).unwrap();
                cell.kind = 186;
                cell.frame_x -= 378;
                touch(world, k, l);
            }
        }
    }
    if !broken {
        return None;
    }
    // The drop is the table family's alone (`GetItemDrop_Tables(style)` for
    // 14, `GetItemDrop_Pianos/Dressers(style)` for 87/88 — the slices cover
    // style 0 → 32); the piles' per-cell
    // case group is empty (:65759-65815) and `Check3x2`'s destroy switch
    // carries the pile specials — for 187 the sword-in-stone styles roll
    // the Enchanted Sword (:49828-49839), resolved app-side like the 2x1
    // coin table.
    let pile_event = if kind == 187 {
        // `int frameX = Main.tile[i, j].frameX;` (:49443) — the CHECKED
        // cell's frameX (a survivor of the pile; the dispatch never runs on
        // the dead cell). The 54-wide style stride puts every cell of a
        // style in one 54 bucket, so the 918..=970 gate selects exactly the
        // sword-in-stone style whichever cell was checked.
        Some(PileLootEvent {
            kind: 187,
            frame_x,
            x: ox,
            y: oy,
        })
    } else {
        None
    };
    Some(CheckPlan {
        drop: (kind == 14 || kind == 87 || kind == 88).then(|| ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event,
        ring: Some(region_ring(&region)),
    })
}

/// `WorldGen.Check1x2` (`WorldGen.cs:43335-43501`) for the wooden chair
/// (type 15): origin row from `frameY % 40` (:43343-43349 — 40 is the
/// chair's `CoordinateFullHeight`, so `frameY / 40` is the style row), both
/// cells must be type-15 with frames `40*style` / `40*style + 18`
/// (:43367-43369), and the support below allows platforms explicitly
/// (:43457-43459). Drop = `GetItemDrop_Chair(style)` (:43494), 32x32 at
/// (trigger x, **origin y**); style 0 → 34 (:42635).
fn check_chair(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_y = t.frame_y as i32;
    let style_row = frame_y / 40;
    let oy = if frame_y % 40 == 18 { y - 1 } else { y };
    let top = world.tile(x, oy);
    let bottom = world.tile(x, oy + 1);
    let intact = top.is_some_and(|c| c.frame_y as i32 == style_row * 40 && c.kind == 15)
        && bottom.is_some_and(|c| c.frame_y as i32 == style_row * 40 + 18 && c.kind == 15);
    let supported = solid_tile_allow_bottom_slope(world, x, oy + 2)
        || world
            .tile(x, oy + 2)
            .is_some_and(|c| td::SET_PLATFORMS.has(c.kind));
    if intact && supported {
        return None;
    }
    let region = [(x, oy), (x, oy + 1)];
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 15,
            x,
            y: oy,
            item: None,
        }),
        cells: killable_cells(world, &region, 15),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_chair).
    })
}

/// `WorldGen.Check2x1` (`WorldGen.cs:46973-47140`) for the work bench
/// (type 18): origin column from `frameX / 18 % 2` (:46981-46988), the right
/// cell's `frameX` must continue the left's by 18 with both active type-18
/// (:47016-47020), and `SolidTileAllowBottomSlope` below both cells (the
/// default support branch, :47026-47031). Drop = `GetItemDrop_Workbenches
/// (frameX / 36)` (:47121-47124), 32x32 at (**origin x**, trigger y); style
/// 0 → 36 (:42447).
fn check_bench(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let ox = x - frame_x / 18 % 2;
    let left = world.tile(ox, y);
    let right = world.tile(ox + 1, y);
    let intact = right.is_some_and(|c| {
        c.frame_x as i32 == left.map(|l| l.frame_x as i32).unwrap_or(i32::MIN) + 18
    }) && left.is_some_and(|c| c.active() && c.kind == 18)
        && right.is_some_and(|c| c.active() && c.kind == 18)
        && solid_tile_allow_bottom_slope(world, ox, y + 1)
        && solid_tile_allow_bottom_slope(world, ox + 1, y + 1);
    if intact {
        return None;
    }
    let region = [(ox, y), (ox + 1, y)];
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 18,
            x: ox,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 18),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_bench).
    })
}

/// `WorldGen.CheckChest` (`WorldGen.cs:57923-57984`): the 2x2 chest needs
/// every cell active type-21 with frames `frameX / 18 % 2 == dx` and
/// `frameY == dy * 18` (:57936-57949), and `Main.tileSolid` below both
/// columns (:57950-57958 — platforms pass: `Main.tileSolid[19] = true`,
/// `Main.cs:7893`). The `Chest.CanDestroyChest` gate (:57958) is always true
/// with no chest entities (M3-CHEST-UI seam). Drop =
/// `GetItemDrop_Chests(frameX / 36)` (:57969-57970), 32x32 at the trigger
/// cell; style 0 → 48 (:41677-41681).
fn check_chest(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let frame_y = t.frame_y as i32;
    let ox = x - (frame_x / 18 % 2);
    let oy = y - frame_y / 18;
    let mut broken = false;
    let mut region = Vec::with_capacity(6); // the kill loop spans 2x3 (:57942-57952)
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let expected = (((k - ox) * 18) as i16, ((l - oy) * 18) as i16);
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != 21 || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // Support row (:57950-57958).
        if world
            .tile(k, oy + 2)
            .is_none_or(|c| !c.active() || !td::TILE_SOLID.has(c.kind))
        {
            broken = true;
        }
        region.push((k, oy + 2));
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 21,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 21),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
        // floats until a later break (`WorldGen.cs`'s check_chest).
    })
}

/// `WorldGen.InvalidTileForPilesOrSpeleothems` (`WorldGen.cs:38979-38994`):
/// an active support cell whose kind is in `TileID.Sets.Boulders`
/// (`TileID.cs:504`) refuses to carry piles/speleothems. Inactive and
/// out-of-world cells are valid (`!active` / the `InWorld` guard → false).
fn invalid_tile_for_piles(world: &World, x: i32, y: i32) -> bool {
    /// `TileID.Sets.Boulders` (`TileID.cs:504`).
    const BOULDERS: &[u16] = &[138, 484, 664, 665, 711, 712, 713, 714, 715, 716];
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && BOULDERS.contains(&t.kind))
}

/// `WorldGen.SolidTile2(Tile)` (`WorldGen.cs:70134-70147`; the indexed
/// overload :70492-70504): active, solid, slopeless (or a platform with a top
/// slope — the refinement cannot occur in the port), non-halfBrick, not
/// actuated. Unlike [`solid_tile`] solid-top tiles pass; an out-of-world cell
/// is refused (the C# try/catch returns false) — the in-world null case does
/// not exist in the port's dense array.
pub(crate) fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && ((td::SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope())) || t.slope() == 0)
        && !t.half_brick()
        && crate::tileobject::nactive(t)
}

/// `Tile.anyLava` (`Tile.cs:410-415`): a positive liquid amount whose kind
/// flag is lava (`Tile.lava()`, `Tile.cs:345-347`).
fn any_lava(tile: &Tile) -> bool {
    tile.liquid > 0 && tile.liquid_kind() == crate::tile::LiquidKind::Lava
}

/// `WorldGen.CheckSunflower` (`WorldGen.cs:53729-53792`): the 2x4 sunflower
/// needs all eight cells active type-27 with the origin derived from the
/// frames (:53739-53751 — the column from `frameX / 18` reduced mod 2, the
/// row from `frameY / 18`) and `nactive` ground of {2,477,492,109,60,633}
/// that is also `SolidTile` under both columns (:53756-53770 — 78/380 are
/// absent, unlike the herbs). On a break every matching cell dies
/// (:53773-53783) and the sunflower item spawns at the check's trigger cell
/// (:53787-53791 — `Item.NewItem(…, 63)`, 32x32 at `(i, j)`).
fn check_sunflower(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    /// :53758-53764 — the ground family.
    const SOILS: &[u16] = &[2, 477, 492, 109, 60, 633];
    let t = world.tile(x, y)?;
    let ox = x - i32::from(t.frame_x) / 18 % 2; // :53739-53747
    let oy = y - i32::from(t.frame_y) / 18; // :53741
    let mut broken = false;
    let mut region = Vec::with_capacity(8);
    for k in ox..ox + 2 {
        for l in oy..oy + 4 {
            region.push((k, l));
            // :53746-53755 — `nactive`, the type, the column and the row.
            let intact = world.tile(k, l).is_some_and(|c| {
                crate::tileobject::nactive(c)
                    && c.kind == 27
                    && i32::from(c.frame_x) / 18 % 2 == k - ox
                    && i32::from(c.frame_y) == (l - oy) * 18
            });
            if !intact {
                broken = true;
            }
        }
        // :53756-53770 — `nactive` + the soil set + `SolidTile`.
        let ground = world.tile(k, oy + 4);
        if !ground.is_some_and(|g| crate::tileobject::nactive(g) && SOILS.contains(&g.kind))
            || !solid_tile(world, k, oy + 4)
        {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 27,
            x,
            y,
            item: Some(63), // :53787-53791
        }),
        cells: killable_cells(world, &region, 27),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
    })
}

/// The `CheckPot` break sound (`WorldGen.cs:56856-56867`): styles 7-9 break
/// with sound 6, 16-24 with 4, everything else with 13. The app plays it at
/// the event's cell.
pub fn pot_break_sound(style: i32) -> u16 {
    match style {
        7..=9 => 6,
        16..=24 => 4,
        _ => 13,
    }
}

/// One `KillTile_PlaySounds` cue: the legacy `(sound id, style)` pair — the
/// app resolves it to the pinned asset ([`crate::audio`], research D3).
pub type SoundCue = (u16, u16);

/// The cues one `KillTile_PlaySounds` call emits, in play order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct KillSounds {
    /// The type arm (`WorldGen.cs:66434-66551`).
    pub primary: Option<SoundCue>,
    /// The extra cue the 231 arm adds (`:66518-66521`, `SoundID.Item173`).
    pub extra: Option<SoundCue>,
    /// The `!fail` shatter tail (`:66554-66574`).
    pub shatter: Option<SoundCue>,
}

/// `WorldGen.KillTile_PlaySounds(i, j, fail, tileCache)`
/// (`WorldGen.cs:66427-66575`) for the hit cell's `kind`/`frame_x`. `fail`
/// mirrors the C# argument (`false` on the real break, `true` from the
/// failure-tick callers, `Player.cs:39880`/`Projectile.cs:70219`).
///
/// Deviation (research D3): the 147/224 arm rolls `genRand.Next(2)` between
/// `Item48`/`Item49`; the port pins `Item_48` and draws nothing — audio-only
/// variation with no state effect.
pub fn kill_tile_sound(kind: u16, frame_x: i16, fail: bool) -> KillSounds {
    let fx = i32::from(frame_x);
    let grass = Some((6, 1));
    // :66434-66448 — the tail-early arms.
    let first = if kind == 127 || kind == 623 {
        Some((2, 27))
    } else if kind == 147 || kind == 224 {
        // :66440-66447 — the genRand roll between Item48/Item49 (D3: pinned).
        Some((2, 48))
    } else if matches!(kind, 161 | 163 | 164 | 200 | 541 | 736) {
        Some((2, 50))
    } else if matches!(kind, 518 | 519 | 528 | 529 | 549 | 637 | 638 | 636) {
        grass
    } else if kind == 530 && fx < 270 {
        grass
    } else if kind == 705 && fx % 6 < 270 {
        grass
    } else {
        // :66467-66552 — the switch.
        match kind {
            // :66469-66478
            3 | 110 | 254 | 24 => grass,
            // :66480-66484 — `Main.tileAlch` + the huge herb/plant list.
            _ if crate::tileobject::tile_alch(kind)
                || matches!(
                    kind,
                    384 | 227 | 32 | 51 | 697 | 52 | 61 | 703 | 62 | 69 | 655 | 71 | 73 | 74
                        | 113 | 115 | 184 | 192 | 205 | 233 | 352 | 382 | 624 | 656 | 700 | 701
                ) =>
            {
                grass
            }
            // :66485-66489 — the mummy's killed cue (Style 11).
            485 => Some((4, 11)),
            // :66490-66494
            201 => grass,
            // :66495-66499
            481 | 482 | 483 => Some((2, 127)),
            // :66500-66504 — the long dirt/stone/metal list → Tink.
            _ if matches!(
                kind,
                1 | 6 | 7 | 8 | 9 | 22 | 140 | 25 | 37 | 38 | 39 | 41 | 43 | 44 | 45 | 46
                    | 47 | 48 | 56 | 58 | 63 | 64 | 65 | 66 | 67 | 68 | 75 | 76 | 107 | 108
                    | 111 | 117 | 118 | 119 | 120 | 121 | 122 | 150 | 151 | 152 | 153 | 154
                    | 155 | 156 | 160 | 161 | 166 | 167 | 168 | 169 | 175 | 176 | 177 | 203
                    | 202 | 204 | 206 | 211 | 221 | 222 | 223 | 226 | 248 | 249 | 250 | 272
                    | 273 | 274 | 478 | 284 | 325 | 346 | 347 | 348 | 350 | 367 | 357 | 368
                    | 369 | 370 | 407 | 472 | 473 | 500 | 501 | 502 | 503 | 546 | 557 | 566
                    | 618 | 408 | 409 | 669 | 670 | 671 | 672 | 673 | 674 | 675 | 676 | 677
                    | 678 | 679 | 680 | 681 | 682 | 683 | 684 | 685 | 686 | 687 | 689 | 688
                    | 690 | 691 | 692 | 726 | 749 | 747 | 743 | 735 | 737 | 745 | 750
            ) =>
            {
                Some((21, 1))
            }
            // :66505-66509
            666 => Some((2, 177)),
            // :66510-66514
            739 => Some((3, 25)),
            // :66515-66523 — the killed sound, plus the crate's item for 231.
            231 | 195 | 474 => Some((4, 1)),
            // :66524-66528
            26 | 695 if fx >= 54 => Some((4, 1)),
            // :66529-66539 — the inner switch.
            314 => Some((2, 52)),
            330..=333 => Some((18, 1)),
            // :66540-66549 — the default: the glass family pops only on a real
            // break, everything else digs except the silent 138.
            _ if !fail
                && (matches!(kind, 162 | 385 | 129) || (kind == 165 && fx < 54)) =>
            {
                Some((2, 27))
            }
            _ if kind == 138 => None,
            _ => Some((0, 1)),
        }
    };
    KillSounds {
        primary: first,
        extra: (kind == 231).then_some((2, 173)),
        // :66554-66574 — the `!fail` shatter tail.
        shatter: (!fail
            && matches!(
                kind,
                13 | 54 | 326 | 327 | 328 | 329 | 345 | 458 | 459 | 507 | 508 | 708 | 748
            ))
        .then_some((13, 1)),
    }
}

/// `WorldGen.CheckPot` (`WorldGen.cs:56797-57016`) for tile 28 — the echo pot
/// 653 is out of scope. The 2x2 needs all four cells active type-28 with the
/// origin derived from the frames (:56808-56817 — the column from `frameX /
/// 18` reduced mod 2, the row reduced mod 2 while the style accumulates
/// `frameY / 36`) and `SolidTile2` ground under both columns (:56845-56849 —
/// solid-top platforms pass, `Main.tileSolid[19] = true`, `Main.cs:7893`).
/// On a break the sound plays by style (:56856-56867, [`pot_break_sound`]),
/// every matching cell dies (:56871-56877) and the loot event fires
/// (:57011-57014). The Gore shards (:56878-56953) have no port mechanism
/// (only the leaf `LeafGore`) and stay deferred — no item spawns here for
/// tile 28 (`Item.NewItem` exists only in the 653 arm, :56955-57009).
fn check_pot(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let ox = x - i32::from(t.frame_x) / 18 % 2; // :56808-56811
    let mut row = i32::from(t.frame_y) / 18; // :56812-56817
    let mut style = 0;
    while row > 1 {
        row -= 2;
        style += 1;
    }
    let oy = y - row;
    let mut broken = false;
    let mut region = Vec::with_capacity(4);
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            region.push((k, l));
            // :56829-56837 — `active` (not `nactive`), the type, the column
            // and the style-row frame.
            let intact = world.tile(k, l).is_some_and(|c| {
                c.active()
                    && c.kind == 28
                    && i32::from(c.frame_x) / 18 % 2 == k - ox
                    && i32::from(c.frame_y) == (l - oy) * 18 + style * 36
            });
            if !intact {
                broken = true;
            }
        }
        if !solid_tile2(world, k, oy + 2) {
            broken = true; // :56845-56849
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        // No `Item.NewItem` for tile 28 — the loot runs through the event.
        drop: None,
        cells: killable_cells(world, &region, 28),
        pot_event: Some(PotEvent { style, x, y }),
        herb_event: None,
        pile_event: None,
        ring: None,
    })
}

/// `WorldGen.CheckPile` (`WorldGen.cs:46875-46971`) for tile 185: a `frameY`
/// of 18 is the 2x1 variant and routes to [`check_pile_2x1`] (:46896);
/// otherwise the pile needs `SolidTileAllowBottomSlope` below (:46898-46902)
/// and, for the ground-judged styles, the soil family under it
/// (:46915-46969): 36-47 the snow/ice family (162/224 included), 54-59 and
/// 73-76 the sand families; the remaining styles judge nothing. The kill is
/// the single cell (`KillTile(i, y)`); no item spawns for 185 (the empty
/// per-cell group :65759+ — the coin-pile rolls of the 2x1 arm are RNG drops,
/// phase 3).
fn check_pile(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = *world.tile(x, y)?;
    if t.frame_y == 18 {
        return check_pile_2x1(world, x, y, t.kind);
    }
    if !solid_tile_allow_bottom_slope(world, x, y + 1) {
        return Some(CheckPlan {
            drop: None,
            cells: killable_cells(world, &[(x, y)], t.kind),
            pot_event: None,
            herb_event: None,
            pile_event: None,
            ring: None,
        });
    }
    // :46904-46913 — only 185 carries the ground rules (the echo 649 is out
    // of scope); a dead ground cell below returns without a kill.
    if t.kind != 185 {
        return None;
    }
    let Some(below) = world.tile(x, y + 1).filter(|g| g.active()) else {
        return None;
    };
    let style = i32::from(t.frame_x) / 18;
    let grounded = match style {
        // :46915-46931 — the snow/ice styles.
        36..=47 => td::SET_ICES_SNOW.has(below.kind) || below.kind == 162 || below.kind == 224,
        // :46932-46952 — the sand styles.
        54..=59 | 73..=76 => {
            td::SET_CONVERSION_SAND.has(below.kind)
                || td::SET_CONVERSION_HARDENED_SAND.has(below.kind)
                || td::SET_CONVERSION_SANDSTONE.has(below.kind)
        }
        // :46953-46968 — the explicit empty cases and everything before 36.
        _ => true,
    };
    if grounded {
        return None;
    }
    Some(CheckPlan {
        drop: None,
        cells: killable_cells(world, &[(x, y)], t.kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
    })
}

/// The `Check2x1` type-185 arm (`WorldGen.cs:46973-47140`): the 2x1 pile
/// needs both cells active of `kind` with the right-hand frame continuing the
/// left's by 18 (:47000-47006), `SolidTileAllowBottomSlope` below both
/// columns (the default arm, :47026-47031), no Boulder under either column
/// and the style's ground family (`num5 = frameX / 36`, :47044-47086: 25-30
/// snow/ice, 41-46 and 62-64 sand). The grass-covered variants (`frameX`
/// 1368-1458) revert to the bare pile (`frameX -= 1368` on both cells) when
/// neither ground cell is dirt/grass (:47040-47048) — before the style is
/// re-judged. The coin-pile `Item.NewItem` rolls (:47142-47232) are RNG drops
/// — phase 3.
fn check_pile_2x1(world: &mut World, x: i32, y: i32, kind: u16) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let ox = x - i32::from(t.frame_x) / 18 % 2; // :46981-46988
    let left = world.tile(ox, y);
    let right = world.tile(ox + 1, y);
    // :47000-47006 — the pair intact.
    let mut broken = !right
        .is_some_and(|c| c.frame_x == left.map(|l| l.frame_x).unwrap_or(i16::MIN) + 18)
        || !left.is_some_and(|c| c.active() && c.kind == kind)
        || !right.is_some_and(|c| c.active() && c.kind == kind);
    // :47026-47031 — the default support arm.
    if !solid_tile_allow_bottom_slope(world, ox, y + 1)
        || !solid_tile_allow_bottom_slope(world, ox + 1, y + 1)
    {
        broken = true;
    }
    if kind == 185 {
        // :47041-47048 — the grass-covered variants revert on non-dirt
        // ground (both columns).
        let ground_not_dirt = |world: &World, dx: i32| {
            world
                .tile(ox + dx, y + 1)
                .is_none_or(|g| !matches!(g.kind, 2 | 477 | 492))
        };
        if left.is_some_and(|c| (1368..=1458).contains(&i32::from(c.frame_x)))
            && ground_not_dirt(world, 0)
            && ground_not_dirt(world, 1)
        {
            for dx in 0..2 {
                if let Some(cell) = world.tile_mut(ox + dx, y) {
                    cell.frame_x -= 1368;
                    touch(world, ox + dx, y);
                }
            }
        }
        // :47050-47086 — no Boulder below, then the style's ground family
        // (`num5` reads the origin after the conversion; it is only
        // load-bearing when the pair is intact, so the origin is active).
        let origin = world.tile(ox, y);
        let num5 = i32::from(origin.map(|c| c.frame_x).unwrap_or(0)) / 36
            + (i32::from(origin.map(|c| c.frame_y).unwrap_or(0)) / 18 - 1) * 52;
        for dx in 0..2 {
            if invalid_tile_for_piles(world, ox + dx, y + 1) {
                broken = true;
                break;
            }
            if let Some(g) = world.tile(ox + dx, y + 1).filter(|g| g.active()) {
                match num5 {
                    25..=30 => {
                        if !(td::SET_ICES_SNOW.has(g.kind) || g.kind == 162 || g.kind == 224) {
                            broken = true;
                        }
                    }
                    41..=46 | 62..=64 => {
                        if !(td::SET_CONVERSION_SAND.has(g.kind)
                            || td::SET_CONVERSION_HARDENED_SAND.has(g.kind)
                            || td::SET_CONVERSION_SANDSTONE.has(g.kind))
                        {
                            broken = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if !broken {
        return None;
    }
    let region = [(ox, y), (ox + 1, y)];
    // `int frameX = Main.tile[num, y].frameX; if (!Main.tile[num, y].active())
    // frameX = Main.tile[num + 1, y].frameX;` (:47127-47131) — read before the
    // kills, after the grass-cover conversion. Only type 185 carries the loot
    // block (`if (type == 185)`, :47130).
    let pile_event = if kind == 185 {
        let frame_x = world
            .tile(ox, y)
            .filter(|c| c.active())
            .or_else(|| world.tile(ox + 1, y))
            .map(|c| i32::from(c.frame_x))
            .unwrap_or(0);
        Some(PileLootEvent {
            kind: 185,
            frame_x,
            x: ox,
            y,
        })
    } else {
        None
    };
    Some(CheckPlan {
        drop: None,
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event,
        ring: None,
    })
}

/// `WorldGen.AnchorValid(tile, AnchorType.Table)` (`WorldGen.cs:51841-51871`)
/// reduced to the Table bit: `nactive`, then the platform-frame arm (the flat
/// platform segments — `PlatformFrameWidth()` is tile 19's full width 18,
/// `TileObjectData.cs:5121-5124`/`2127-2139`), the solid-top arm, or the plain
/// tile-table arm (`blockType() == 0`, [`crate::tileobject::block_type`]).
fn anchor_valid_table(below: &Tile) -> bool {
    if !crate::tileobject::nactive(below) {
        return false;
    }
    if td::SET_PLATFORMS.has(below.kind) {
        let seg = i32::from(below.frame_x) / 18;
        return (!below.half_brick() && (0..=7).contains(&seg))
            || (12..=16).contains(&seg)
            || (25..=26).contains(&seg);
    }
    if td::TILE_SOLID.has(below.kind) && td::TILE_SOLID_TOP.has(below.kind) {
        return true;
    }
    crate::tileobject::is_table(below.kind) && crate::tileobject::block_type(below) == 0
}

/// `WorldGen.CheckOnTable1x1` (`WorldGen.cs:43503-43546`) — the dispatch
/// group 13/33/49/50/78/174/372/646/707 (:85604-85614); the registry carries
/// 13 the placed bottle, 33/49 the candles and 78 the clay pot. The support
/// cell below decides:
/// a top slope kills unless the platform corner rule holds (:43513-43515 —
/// `blockType` 3/slope 2 leans on an active flat platform at `x - 1`,
/// `blockType` 2/slope 1 on one at `x + 1`), a half brick kills
/// (:43525-43528), the table anchors survive before the type test
/// (:43530-43532), and — the type-78 exception, :43534-43540 — any
/// `SolidTileAllowBottomSlope` below carries the pot; the group's
/// unconditional kill (:43541-43543) is the non-78 path. The kill is the
/// single cell; no `Item.NewItem` runs here — the pot's 222 comes from the
/// per-cell `KillTile_GetItemDrops` `case 78` (:64711-64713), resolved
/// app-side through `registry::tile_drop` (phase 3), so the record carries
/// `item: None`.
fn check_on_table_1x1(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    // `Tile tile = Main.tile[x, y + 1]` — an uninitialised support cell
    // returns without a kill (:43505-43508). The 1x1 object is its own
    // origin: no frame reconstruction.
    let below = *world.tile(x, y + 1)?;
    let kill = if below.top_slope() {
        // :43510-43524 — every top slope kills unless the platform corner
        // rule holds (a valid partner: an active platform with
        // `blockType() == 0`).
        let corner = |world: &World, dx: i32, slope: u8| {
            below.slope() == slope
                && world.tile(x + dx, y + 1).is_some_and(|n| {
                    n.active()
                        && crate::tileobject::block_type(n) == 0
                        && td::SET_PLATFORMS.has(n.kind)
                })
        };
        !(corner(world, -1, 2) || corner(world, 1, 1))
    } else if below.half_brick() {
        true // :43525-43528
    } else {
        // :43530-43532 — the table anchors survive before the type test.
        if anchor_valid_table(&below) || (below.active() && crate::tileobject::is_table(below.kind))
        {
            return None;
        }
        // :43534-43543 — the group's unconditional kill, with the type-78
        // exception: any `SolidTileAllowBottomSlope` support carries the pot.
        !(kind == 78 && solid_tile_allow_bottom_slope(world, x, y + 1))
    };
    if !kill {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &[(x, y)], kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
    })
}

/// `WorldGen.CheckAlch` (`WorldGen.cs:45972-46092`): the herb dies when the
/// soil below is not `nactive` or is a half brick (:45983-45990 — slopes
/// pass: no slope test exists here), or — per the `frameX / 18` variant —
/// when the soil is not in the variant's set ([`ALCH_SOILS`], :45998-46086)
/// or the herb cell itself holds lava (`anyLava` — every variant except 5).
/// Variant 5 (Fireblossom) never dies from lava: the cell blinks instead —
/// `liquid > 16` turns a mature 83 into the blooming 84, `liquid <= 16`
/// turns an 84 back (:46052-46075; skipped for the immature 82 and on
/// multiplayer clients — the port is single-player). The unconditional
/// `frameY = 0` (:45991) is kept as a gated write (the port's herbs always
/// carry it). The kill is the single cell; no item spawns here (the per-cell
/// herb drops are phase 3 — `KillTile_GetItemDrops` :65674-65698).
fn check_alch(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let herb = *world.tile(x, y)?;
    // :45983-45990 — the support pair.
    let mut flag = world
        .tile(x, y + 1)
        .is_none_or(|soil| !crate::tileobject::nactive(soil) || soil.half_brick());
    // :45991 — `frameY = 0`.
    if herb.frame_y != 0 {
        world.tile_mut(x, y).unwrap().frame_y = 0;
        touch(world, x, y);
    }
    if !flag {
        let style = i32::from(herb.frame_x) / 18;
        match style {
            s @ 0..=6 => {
                // :45998-46086 — the variant's soil set.
                if !world
                    .tile(x, y + 1)
                    .is_some_and(|soil| ALCH_SOILS[s as usize].contains(&soil.kind))
                {
                    flag = true;
                }
                if s == 5 {
                    // :46052-46075 — the Fireblossom blink (no lava death).
                    if herb.kind != 82 && any_lava(&herb) {
                        let cell = world.tile_mut(x, y).unwrap();
                        if herb.liquid > 16 {
                            if cell.kind == 83 {
                                cell.kind = 84;
                                touch(world, x, y);
                            }
                        } else if cell.kind == 84 {
                            cell.kind = 83;
                            touch(world, x, y);
                        }
                    }
                } else if any_lava(&herb) {
                    flag = true; // :46006-46008 and peers
                }
            }
            // No case matches: the C# switch falls through unchecked
            // (:45996) — the seed fall-through's mirror.
            _ => {}
        }
    }
    if !flag {
        return None;
    }
    Some(CheckPlan {
        drop: None,
        cells: killable_cells(world, &[(x, y)], herb.kind),
        pot_event: None,
        // The killed cell's per-cell drop rolls (`KillTile_GetItemDrops`,
        // `WorldGen.cs:65674-65698`) ride the herb event; the app resolves
        // them with the RNG streams and the harvest predicates.
        herb_event: Some(HerbEvent {
            kind: herb.kind,
            style: i32::from(herb.frame_x) / 18,
            x,
            y,
        }),
        pile_event: None,
        ring: None,
    })
}

/// Clears the active tile at `(x, y)` and keeps the wall and the liquid. The caller is
/// responsible for re-framing; `kill_tile` is the version that does it.
pub(crate) fn remove_tile(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let t = *world.tile(x, y)?;
    if !t.active() || t.wall == WALL_UNBREAKABLE {
        return None;
    }
    let kind = t.kind;
    let i = world.index(x, y);
    world.tiles[i] = Tile {
        wall: t.wall,
        liquid: t.liquid,
        flags: liquid_flags(t.flags),
        ..Default::default()
    };
    world.revision = world.revision.wrapping_add(1);
    world.mark_chunk(x, y);
    Some(kind)
}

/// `WorldGen.CanKillTile` (`WorldGen.cs:62219-62326`) — the three break
/// refusals. The mining chain zeroes the pick damage on a refusal
/// (`Player.PickTile_DetermineDamage`, `Player.cs:53830-53833`) and the
/// `KillTile` entry duplicates the same cases through
/// `CheckTileBreakability` (the 1/2 flavours, `:62377-62461`), so every
/// break source refuses, not only the pick. The C#'s
/// `out bool blockDamaged` collapses here — the 2-arg overload
/// (`:62213-62217`) discards it and no caller reads it.
///
/// Unported arms of the same C# function (M1.4-TO-6 ports the three named
/// refusals only): the tree-trunk / palm / cactus-80 anchors above
/// (`:62247-62257`, `:62271-62281`), the boulder `CheckBoulderChest` scan
/// (`:62283-62287`) and the teleporter 235 (`:62297-62310`).
///
/// `chest_holds_items` stands in for `Chest.CanDestroyChest`'s registry scan
/// (`Chest.cs:609-623`: no entry → destroyable; any slot with
/// `type > 0 && stack > 0` → refused) keyed by the object's top-left cell.
/// The app answers from its in-memory chest store; sources without one pass
/// `|_, _| false` (the store stays empty until the M3-CHEST-UI seam fills
/// it, so the refusal passes there today — same as the actuation slice in
/// `wiring.rs`).
pub fn can_kill_tile(
    world: &World,
    x: i32,
    y: i32,
    chest_holds_items: impl Fn(i32, i32) -> bool,
) -> bool {
    // `:62222-62235` — the bounds and the null/inactive guards; the port's
    // dense array folds them into the `Option`.
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    // `:62236-62239` — the dungeon echo wall refuses every break.
    if t.wall == WALL_UNBREAKABLE {
        return false;
    }
    // `:62240-62281` — the anchor above (`j >= 1` guard kept); only the
    // container-family arm is ported.
    if y >= 1 {
        if let Some(above) = world.tile(x, y - 1) {
            if above.active() {
                let kind = above.kind;
                match kind {
                    // `:62259-62270` — 21 Containers, 26 DemonAltar,
                    // 72 MushroomTrees, 77 Hellforge, 88 Dressers,
                    // 467 Containers2, 488 FallenLog (TileID.cs:671/:681/
                    // :773/:783/:805/:1563/:1605). A different kind under the
                    // object refuses: the object stands on this cell.
                    21 | 26 | 72 | 77 | 88 | 467 | 488 => {
                        if t.kind != kind {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    // `:62288-62296` — the locked golden door (`is_locked_door`,
    // `:69191-69197`); `OpenDoor` refuses the same frames (`:37654-37657`).
    if t.kind == 10 && is_locked_door(t) {
        return false;
    }
    // `:62311-62323` — a chest 21/467 (2 cells wide) or dresser 88 (3 wide)
    // holding items refuses; the anchor is the object's top-left cell, the
    // `Chest._chestsByCoords` key (`Chest.cs:92`).
    let wide: i32 = match t.kind {
        21 | 467 => 2,
        88 => 3,
        _ => return true,
    };
    let anchor_x = x - i32::from(t.frame_x) / 18 % wide;
    let anchor_y = y - i32::from(t.frame_y) / 18;
    !chest_holds_items(anchor_x, anchor_y)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)] // DW-IDs are part of the test names (traceability)
    use super::super::tests::{
        doorway_world, ground_world, object_world, open_the_door, set_active, set_liquid,
        set_object_cell, soil_world, ALCH_SOIL_TABLE,
    };
    use super::super::{place_item_tile, place_tile};
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A tree over a dirt ground: a trunk column of `tree::TREE` cells with
    /// the canopy marker on top.
    fn tree_world(ground: u16, height: i32) -> World {
        let mut w = ground_world();
        w.frame_important[5] = true;
        // The ground the tree grows on.
        for x in 8..14 {
            let i = w.index(x, 20);
            w.tiles[i].kind = ground;
        }
        // The trunk column at x = 10, the base at y = 19.
        for dy in 0..height {
            let i = w.index(10, 19 - dy);
            let (fx, fy) = if dy == height - 1 {
                (22, 198) // the canopy marker
            } else {
                (0, 0)
            };
            w.tiles[i] = Tile {
                kind: 5,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w.compute_column_surface();
        w
    }

    /// BUG-16: felling the base of a tree collapses the whole trunk
    /// (`WorldGen.CheckTree`'s `KillTile` chain, `WorldGen.cs:54643-54727`:
    /// the cell above a missing ground dies, and its own `TileFrame` ring
    /// reaches the next one up).
    #[test]
    fn BUG_16_the_felled_tree_collapses_whole() {
        let mut w = tree_world(2, 8);
        let kill = kill_tile(&mut w, 10, 19, false).expect("the base dies");
        // Every trunk cell is gone — the trunk does not float.
        for dy in 0..8 {
            assert!(
                !w.tile(10, 19 - dy).unwrap().active(),
                "the cell at dy {dy} survived"
            );
        }
        // One drop record per killed cell (the C#'s `KillTile_DropItems` runs
        // for every killed cell).
        assert_eq!(kill.tree_events.len(), 8, "one wood drop per cell");
        // The family comes from the ground under the trunk column.
        for event in &kill.tree_events {
            match event {
                TreeDropEvent::Trunk { tree_type, .. } => {
                    assert_eq!(*tree_type, crate::tree::TreeType::Forest);
                }
                _ => panic!("not a trunk"),
            }
        }
    }

    /// BUG-16: the jungle ground names the tree's wood family
    /// (`WorldGen.GetTreeType`, `WorldGen.cs:63079-63109`).
    #[test]
    fn BUG_16_the_tree_family_comes_from_the_ground() {
        let mut w = tree_world(60, 4);
        let kill = kill_tile(&mut w, 10, 19, false).expect("the base dies");
        assert_eq!(kill.tree_events.len(), 4);
        for event in &kill.tree_events {
            match event {
                TreeDropEvent::Trunk { tree_type, .. } => {
                    assert_eq!(*tree_type, crate::tree::TreeType::Jungle);
                }
                _ => panic!("not a trunk"),
            }
        }
    }

    /// BUG-16: the canopy marker carries the acorn roll (`frameX >= 22 &&
    /// frameY >= 198`, `WorldGen.cs:66191`) — only the top cell does.
    #[test]
    fn BUG_16_only_the_canopy_marker_may_drop_an_acorn() {
        let mut w = tree_world(2, 5);
        let kill = kill_tile(&mut w, 10, 19, false).expect("the base dies");
        let markers = kill
            .tree_events
            .iter()
            .filter(|e| match e {
                TreeDropEvent::Trunk {
                    frame_x, frame_y, ..
                } => *frame_x >= 22 && *frame_y >= 198,
                _ => false,
            })
            .count();
        assert_eq!(markers, 1, "only the canopy cell carries the acorn roll");
    }

    /// BUG-16: a palm over sand dies with the sand (`WorldGen.CheckPalmTree`,
    /// `WorldGen.cs:54377-54384`) and drops the palm wood
    /// (`KillTile_GetItemDrops`'s `case 323`, `:65232-65261`).
    #[test]
    fn BUG_16_the_palm_dies_with_its_sand() {
        let mut w = ground_world();
        w.frame_important[323] = true;
        // The palm column at x = 10 over the sand at y = 20.
        for dy in 0..4 {
            let i = w.index(10, 19 - dy);
            w.tiles[i] = Tile {
                kind: 323,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        let i = w.index(10, 20);
        w.tiles[i].kind = 53; // the sand
        // Chop the sand away: the palm above loses its ground.
        let kill = kill_tile(&mut w, 10, 20, false).expect("the sand dies");
        assert!(
            !w.tile(10, 19).unwrap().active(),
            "the palm below the lost sand survived"
        );
        assert!(
            kill.tree_events
                .iter()
                .any(|e| matches!(e, TreeDropEvent::Palm { item: 2504, .. }))
        );
    }

    /// S1-P-i30: a variant tree (gem/vanity/ash) over its ground — the trunk
    /// column of `kind` cells with the canopy marker on top.
    fn variant_tree_world(kind: u16, ground: u16, height: i32) -> World {
        let mut w = ground_world();
        w.frame_important[kind as usize] = true;
        for x in 8..14 {
            let i = w.index(x, 20);
            w.tiles[i].kind = ground;
        }
        for dy in 0..height {
            let i = w.index(10, 19 - dy);
            let (fx, fy) = if dy == height - 1 { (22, 198) } else { (0, 0) };
            w.tiles[i] = Tile {
                kind,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w.compute_column_surface();
        w
    }

    /// S1-P-i30: the gem tree (583 Topaz) collapses whole over stone and every
    /// cell records a `Gem` drop with the kind's gem/seed pair
    /// (`KillTile_GetItemDrops` `case 583`, `WorldGen.cs:65713-65719`:
    /// `SetGemTreeDrops(180, 4851, …)`). The ground test is
    /// `GemTreeGroundTest` (`:30304-30315`, stone/moss).
    #[test]
    fn S1_P_i30_the_gem_tree_collapses_and_records_the_gem_pair() {
        // Stone 1 is in `SET_CONVERSION_STONE` — the gem ground.
        let mut w = variant_tree_world(583, 1, 5);
        let kill = kill_tile(&mut w, 10, 19, false).expect("the base dies");
        for dy in 0..5 {
            assert!(
                !w.tile(10, 19 - dy).unwrap().active(),
                "the gem cell at dy {dy} survived"
            );
        }
        assert_eq!(kill.tree_events.len(), 5, "one gem drop per cell");
        for event in &kill.tree_events {
            match event {
                TreeDropEvent::Gem { gem, seed, .. } => {
                    assert_eq!(*gem, 180, "Topaz");
                    assert_eq!(*seed, 4851, "the Topaz gemcorn");
                }
                _ => panic!("not a gem record"),
            }
        }
        // The Amber tree (589) carries its own pair (999, 4857).
        let mut w = variant_tree_world(589, 1, 3);
        let kill = kill_tile(&mut w, 10, 19, false).expect("the base dies");
        assert!(kill.tree_events.iter().all(|e| matches!(
            e,
            TreeDropEvent::Gem { gem: 999, seed: 4857, .. }
        )));
    }

    /// S1-P-i30: the mushroom tree (72) collapses whole over the mushroom
    /// grass 70 and every cell records a `Mushroom` drop
    /// (`TileFrameImportant` `case 72`, `WorldGen.cs:85851-85854`; the drop
    /// arm `case 71/72`, `:65642-65652`).
    #[test]
    fn S1_P_i30_the_mushroom_tree_collapses_over_the_mushroom_grass() {
        let mut w = variant_tree_world(72, 70, 4);
        let kill = kill_tile(&mut w, 10, 19, false).expect("the base dies");
        for dy in 0..4 {
            assert!(
                !w.tile(10, 19 - dy).unwrap().active(),
                "the mushroom cell at dy {dy} survived"
            );
        }
        assert_eq!(kill.tree_events.len(), 4);
        assert!(kill
            .tree_events
            .iter()
            .all(|e| matches!(e, TreeDropEvent::Mushroom { .. })));
    }

    /// S1-P-i30: a surviving mushroom-tree top cell (frameX 0, nothing above)
    /// re-rolls its canopy frame when a neighbour is framed
    /// (`WorldGen.cs:85855-85873`) — the core records the request, the app
    /// resolves `genRand.Next(3)`.
    #[test]
    fn S1_P_i30_the_mushroom_top_records_a_canopy_roll() {
        let mut w = ground_world();
        w.frame_important[72] = true;
        // The mushroom grass support at (10,20) (the floor row), the top
        // cell above it.
        let i = w.index(10, 20);
        w.tiles[i].kind = 70;
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 72,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        w.compute_column_surface();
        // Break the stone beside the support: the 3x3 frame reaches the top
        // cell, which survives (its below is the 70) and records the roll.
        let kill = kill_tile(&mut w, 11, 20, false).expect("the stone dies");
        assert!(w.tile(10, 19).unwrap().active(), "the top cell survives");
        assert!(
            kill.mushroom_frame_rolls.contains(&(10, 19)),
            "the top cell records the canopy re-roll"
        );
    }

    /// S1-P-i30: the cactus (80) collapses whole when its sand dies and every
    /// segment records a flat Cactus 276 drop (`WorldGen.CheckCactus`,
    /// `:54116-54158`; the drop `case 80`, `:64717-64719`).
    #[test]
    fn S1_P_i30_the_cactus_collapses_whole_and_drops_per_cell() {
        let mut w = ground_world();
        // The cactus column at x = 10 over the sand at y = 20.
        for dy in 0..4 {
            let i = w.index(10, 19 - dy);
            w.tiles[i] = Tile {
                kind: 80,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        let i = w.index(10, 20);
        w.tiles[i].kind = 53; // the sand
        w.compute_column_surface();
        let kill = kill_tile(&mut w, 10, 20, false).expect("the sand dies");
        for dy in 0..4 {
            assert!(
                !w.tile(10, 19 - dy).unwrap().active(),
                "the cactus segment at dy {dy} survived"
            );
        }
        assert_eq!(kill.tree_events.len(), 4, "one cactus drop per segment");
        assert!(kill
            .tree_events
            .iter()
            .all(|e| matches!(e, TreeDropEvent::Cactus { .. })));
    }

    /// S1-P-i30: a cactus on a non-sand ground dies (`CheckCactus`'s base test,
    /// `WorldGen.cs:54139-54143` — the ground must be 53/112/116/234, full and
    /// slopeless).
    #[test]
    fn S1_P_i30_the_cactus_dies_on_a_non_sand_ground() {
        let mut w = ground_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 80,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        // The ground below is stone (the `ground_world` floor at y = 20),
        // not sand.
        w.compute_column_surface();
        // Frame the cactus through a neighbour break: it loses the ground test.
        let kill = kill_tile(&mut w, 11, 20, false).expect("the stone dies");
        assert!(!w.tile(10, 19).unwrap().active(), "the cactus dies");
        assert!(kill
            .tree_events
            .iter()
            .any(|e| matches!(e, TreeDropEvent::Cactus { .. })));
    }

    /// M1.4-TO-6: the container family above refuses the cell below
    /// (`WorldGen.cs:62259-62270`); the same kind passes (`tile.type != type`).
    #[test]
    fn the_container_family_above_refuses_can_kill_tile() {
        let mut w = ground_world();
        for (n, kind) in [21u16, 26, 72, 77, 88, 467, 488].into_iter().enumerate() {
            let x = 10 + n as i32;
            set_object_cell(&mut w, x, 19, kind, 0, 0);
            assert!(
                !can_kill_tile(&w, x, 20, |_, _| false),
                "{kind} refuses below"
            );
        }
        // The same kind under itself does not refuse (`tile.type != type`).
        set_object_cell(&mut w, 20, 19, 1, 0, 0);
        assert!(can_kill_tile(&w, 20, 20, |_, _| false));
    }

    /// M1.4-TO-6: the locked golden door refuses (`:62288-62296` through
    /// `IsLockedDoor` `:69191-69197`); the wooden frames and the wide
    /// `frameX` column pass.
    #[test]
    fn the_locked_door_refuses_can_kill_tile() {
        let mut w = ground_world();
        set_object_cell(&mut w, 10, 20, 10, 0, 594);
        assert!(
            !can_kill_tile(&w, 10, 20, |_, _| false),
            "lock row opens at 594"
        );
        set_object_cell(&mut w, 10, 20, 10, 0, 646);
        assert!(
            !can_kill_tile(&w, 10, 20, |_, _| false),
            "lock row ends at 646"
        );
        set_object_cell(&mut w, 10, 20, 10, 0, 0);
        assert!(
            can_kill_tile(&w, 10, 20, |_, _| false),
            "wooden frames pass"
        );
        set_object_cell(&mut w, 10, 20, 10, 54, 594);
        assert!(
            can_kill_tile(&w, 10, 20, |_, _| false),
            "frameX >= 54 is not the lock"
        );
    }

    /// M1.4-TO-6: a chest 21/467 (2 cells wide) or dresser 88 (3 wide)
    /// holding items refuses through the anchor scan (`:62311-62323`,
    /// `Chest.cs:609-623`); the anchor is the top-left cell from
    /// `frameX / 18 % wide`.
    #[test]
    fn the_stocked_chest_and_dresser_refuse_can_kill_tile() {
        let mut w = ground_world();
        // The chest's right cell maps back to the top-left anchor.
        set_object_cell(&mut w, 10, 18, 21, 0, 0);
        set_object_cell(&mut w, 11, 18, 21, 18, 0);
        let holds = |x: i32, y: i32| x == 10 && y == 18;
        assert!(!can_kill_tile(&w, 10, 18, holds));
        assert!(
            !can_kill_tile(&w, 11, 18, holds),
            "the right cell maps to the anchor"
        );
        assert!(
            can_kill_tile(&w, 10, 18, |_, _| false),
            "no registry entry passes"
        );
        // Containers2 anchors like the chest.
        set_object_cell(&mut w, 12, 18, 467, 18, 0);
        let holds_467 = |x: i32, y: i32| x == 11 && y == 18;
        assert!(!can_kill_tile(&w, 12, 18, holds_467));
        // The dresser is 3 wide: `frameX / 18 % 3`.
        set_object_cell(&mut w, 14, 18, 88, 36, 0);
        let holds_dresser = |x: i32, y: i32| x == 12 && y == 18;
        assert!(!can_kill_tile(&w, 14, 18, holds_dresser));
        assert!(can_kill_tile(&w, 14, 18, |_, _| false));
    }

    /// M1.4-TO-6: the `CanKillTile` contract head — air, out-of-bounds,
    /// inactive and the dungeon echo wall refuse (`:62222-62239`).
    #[test]
    fn can_kill_tile_contract_head_refuses_air_inactive_and_wall_350() {
        let w = ground_world();
        assert!(!can_kill_tile(&w, 10, 5, |_, _| false), "air refuses");
        assert!(
            !can_kill_tile(&w, -1, 20, |_, _| false),
            "out of bounds refuses"
        );
        let mut w = ground_world();
        let i = w.index(10, 20);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false), "inactive refuses");
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].wall = WALL_UNBREAKABLE;
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false), "wall 350 refuses");
    }

    /// M1.4-TO-6: the refusals gate the outer break sources only —
    /// `kill_tile`'s body (and the cascade's re-entries) stay ungated, the
    /// same split the C# draws between `CanKillTile` and the `KillTile`
    /// entry (`CheckTileBreakability`). Recorded in ANALYSIS as the port's
    /// single-choked-gate deviation.
    #[test]
    fn kill_tile_itself_stays_ungated() {
        let mut w = ground_world();
        set_object_cell(&mut w, 10, 19, 21, 0, 0);
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false));
        assert_eq!(kill_tile(&mut w, 10, 20, true).map(|k| k.kind), Some(1));
    }

    #[test]
    fn the_kill_cascade_takes_the_whole_piece() {
        // Breaking the floor under one column destroys the entire object and
        // yields its drop — the `Check*` routines' shared shape.
        for (kind, tx, ty) in [
            (87u16, 10i32, 19i32),
            (101, 10, 19),
            (104, 10, 19),
            (79, 11, 19),
        ] {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, tx, ty, kind, 0, 1), "place {kind}");
            let def = crate::tileobject::def_for_tile(kind).unwrap();
            let ox = tx - def.origin.x;
            let oy = ty - def.origin.y;
            // `kill_tile` on one of the piece's cells runs its `Check*`
            // routine (`WorldGen.KillTile`'s object dispatch) — the whole
            // piece sweeps.
            kill_tile(&mut w, ox, oy + def.height - 1, true);
            let mut alive = 0;
            for x in ox..ox + def.width {
                for y in oy..oy + def.height {
                    if w.tile(x, y).is_some_and(|t| t.active() && t.kind == kind) {
                        alive += 1;
                    }
                }
            }
            assert_eq!(alive, 0, "kind {kind} fully destroyed");
        }
    }

    /// Asserts every cell of `cells` is inactive (the object is gone).
    fn assert_all_gone(w: &World, cells: &[(i32, i32)]) {
        for &(x, y) in cells {
            assert!(
                !w.tile(x, y).is_some_and(|t| t.active()),
                "cell ({x}, {y}) survived"
            );
        }
    }

    /// The cell whose `Check*` routine runs first for a hit: the object's
    /// first surviving cell in `SquareTileFrame` visit order around the hit —
    /// the anchor the vanilla drop position is derived from. The hit cell
    /// itself is already inactive when the scan reaches it.
    fn expected_trigger(cells: &[(i32, i32)], hit: (i32, i32)) -> (i32, i32) {
        for (dx, dy) in SQUARE_FRAME_ORDER {
            let probe = (hit.0 + dx, hit.1 + dy);
            if probe != hit && cells.contains(&probe) {
                return probe;
            }
        }
        panic!("no surviving object cell in the 3x3 of {hit:?}");
    }

    /// The per-check drop position rule: the bench drops at
    /// (origin x, trigger y) (:47121-47124), the chair at (trigger x,
    /// origin y) (:43494), everything else at the trigger cell.
    fn expected_drop(cells: &[(i32, i32)], hit: (i32, i32), kind: u16) -> (i32, i32) {
        let trigger = expected_trigger(cells, hit);
        match kind {
            18 => (cells.iter().map(|c| c.0).min().unwrap(), trigger.1),
            15 => (trigger.0, cells.iter().map(|c| c.1).min().unwrap()),
            _ => trigger,
        }
    }

    #[test]
    fn test_DW_3_1_bench_cascades_from_every_cell() {
        // `Check2x1` (`WorldGen.cs:46973`): hitting either cell of the 2x1
        // bench destroys the whole bench. The drop is 32x32 at
        // (**origin x**, trigger y) (:47121-47124) — both hits resolve the
        // trigger to the surviving cell, so the drop is (10, 19) either way.
        let cells = [(10i32, 19i32), (11, 19)];
        for hit in cells {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 18);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, hit, 18);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 18,
                    x: dx,
                    y: dy,
                    item: None
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_1_table_cascades_from_every_cell() {
        // `Check3x2` (`WorldGen.cs:49187`): any of the 3x2 table's six cells
        // triggers the full destruction (`SolidTileAllowBottomSlope` support
        // per column, :49396-49399). Drop at the check's trigger cell — the
        // first surviving cell in `SquareTileFrame` order (:49424-49426).
        let cells: Vec<(i32, i32)> = (9..=11)
            .flat_map(|x| (18..=19).map(move |y| (x, y)))
            .collect();
        for hit in &cells {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 14, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 14);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, *hit, 14);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 14,
                    x: dx,
                    y: dy,
                    item: None
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_1_chest_cascades_from_every_cell() {
        // `CheckChest` (`WorldGen.cs:57923`): frame-exact 2x2 integrity plus
        // `Main.tileSolid` below both columns (:57950-57958 — the floor's
        // stone passes). Drop at the check's trigger cell (:57969-57972).
        let cells: Vec<(i32, i32)> = (10..=11)
            .flat_map(|x| (18..=19).map(move |y| (x, y)))
            .collect();
        for hit in &cells {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 21, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 21);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, *hit, 21);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 21,
                    x: dx,
                    y: dy,
                    item: None
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_1_chair_cascades_with_origin_row_drop() {
        // `Check1x2` (`WorldGen.cs:43335`): the chair drop spawns at
        // (trigger x, **origin y**) (:43494) — the one wooden drop position
        // that is not the trigger cell.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 15, 0, 1));
        let kill = kill_tile(&mut w, 10, 18, false).unwrap();
        assert_eq!(kill.kind, 15);
        assert_all_gone(&w, &[(10, 18), (10, 19)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 15,
                x: 10,
                y: 18,
                item: None
            }]
        );
    }

    #[test]
    fn test_DW_3_1_closed_door_cascades_from_every_cell() {
        // `CheckDoorClosed` (`WorldGen.cs:86432`): `SolidTile` above and
        // below the column (:86475-86479) and all three cells type-10. Drop
        // = `DropDoorItem` at the check's trigger cell (:86508).
        let cells = [(10i32, 17i32), (10, 18), (10, 19)];
        for hit in cells {
            let mut w = doorway_world();
            assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 10);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, hit, 10);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 10,
                    x: dx,
                    y: dy,
                    item: None
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_3_open_door_cascade_right_leaf_from_every_cell() {
        // `CheckDoorOpen` (`WorldGen.cs:86339`): the right-opening leaf
        // (`frameX % 72` 0/18 → hinge = leftmost column) dies from any of
        // its six cells; the drop is at the check's trigger cell (:86415).
        let cells = [
            (10i32, 17i32),
            (11, 17),
            (10, 18),
            (11, 18),
            (10, 19),
            (11, 19),
        ];
        for hit in cells {
            let mut w = doorway_world();
            assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
            let open_cells = open_the_door(&mut w, false);
            assert_eq!(open_cells, cells.to_vec());
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 11);
            assert_all_gone(&w, &open_cells);
            let (dx, dy) = expected_drop(&cells, hit, 11);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 11,
                    x: dx,
                    y: dy,
                    item: None
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_3_open_door_cascade_left_leaf() {
        // The left-opening leaf (`frameX % 72` 36/54 → hinge = rightmost
        // column, :86363-86368): killing the hinge-column top cell still
        // destroys the whole 2x3 at columns 9..10. The check triggers on
        // (9, 18) — the first surviving cell in scan order.
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
        let open_cells = open_the_door(&mut w, true);
        let kill = kill_tile(&mut w, 9, 17, false).unwrap();
        assert_eq!(kill.kind, 11);
        assert_all_gone(&w, &open_cells);
        let (dx, dy) = expected_drop(&open_cells, (9, 17), 11);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 11,
                x: dx,
                y: dy,
                item: None
            }]
        );
    }

    #[test]
    fn test_DW_3_1_support_break_kills_the_object_above() {
        // Mining the ground under a chest column: the cascade reaches the
        // chest through `SquareTileFrame`'s neighbourhood and `CheckChest`'s
        // support test destroys it (:57950-57958).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1, "the floor block is the hit cell");
        assert_all_gone(&w, &[(10, 18), (11, 18), (10, 19), (11, 19)]);
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 21);
    }

    #[test]
    fn test_DW_3_1_adjacent_intact_object_survives() {
        // Two benches side by side: destroying one leaves the other intact
        // and produces exactly one drop (`Check2x1`'s integrity holds for
        // the survivor).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        assert!(place_item_tile(&mut w, 12, 19, 18, 0, 1));
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_all_gone(&w, &[(10, 19), (11, 19)]);
        for x in [12, 13] {
            assert!(w.tile(x, 19).unwrap().active(), "bench B cell ({x}, 19)");
        }
        assert_eq!(kill.object_drops.len(), 1);
    }

    #[test]
    fn test_DW_3_1_destroy_object_guard_defers_supported_object_pops() {
        // `WorldGen.destroyObject` (:4236) stays true through a check's whole
        // kill loop (`Check3x2` resets it only at :49841), so the chair's
        // re-entered `Check1x2` (via the inner `KillTile` → `SquareTileFrame`
        // → `TileFrame` chain) early-returns. A chair standing on a chest
        // therefore SURVIVES the chest's cascade, floating, until a later
        // break near it runs the check unguarded — both vanilla steps here.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 21, 0, 1));
        // Chair at (10, 16..17), bottom resting on the chest's top-left cell
        // (10, 18): frames `frameY 0/18`, facing left (`frameX 0`).
        set_object_cell(&mut w, 10, 16, 15, 0, 0);
        set_object_cell(&mut w, 10, 17, 15, 0, 18);
        // A stone block next to the chair: a later break of it reaches the
        // chair's cells through `SquareTileFrame` (a placement next to an
        // intact chair does not pop it — its support still exists).
        assert!(place_tile(&mut w, 11, 17, 1));
        let kill = kill_tile(&mut w, 11, 19, false).unwrap();
        assert_all_gone(&w, &[(10, 18), (11, 18), (10, 19), (11, 19)]);
        assert!(w.tile(10, 16).unwrap().active(), "chair top floats");
        assert!(w.tile(10, 17).unwrap().active(), "chair bottom floats");
        assert_eq!(kill.object_drops.len(), 1, "the chest only");
        assert_eq!(
            (
                kill.object_drops[0].tile_kind,
                kill.object_drops[0].x,
                kill.object_drops[0].y
            ),
            (21, 10, 18)
        );
        // The later break: the stone next to the chair — the chair's check
        // runs unguarded now, finds the support gone, and pops it.
        let kill = kill_tile(&mut w, 11, 17, false).unwrap();
        assert_eq!(kill.kind, 1);
        assert_all_gone(&w, &[(10, 16), (10, 17)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 15,
                x: 10,
                y: 16,
                item: None
            }]
        );
    }

    #[test]
    fn test_DW_3_1_no_item_suppresses_drop_records_but_not_the_cascade() {
        // `noItem` (`WorldGen.cs:63546-63549`) gates the drop reporting;
        // the destruction itself always runs.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        let kill = kill_tile(&mut w, 10, 19, true).unwrap();
        assert_eq!(kill.kind, 18);
        assert_all_gone(&w, &[(10, 19), (11, 19)]);
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_DW_3_3_world_edge_skips_the_cascade_like_tile_frame() {
        // `TileFrame`'s guard (:81330): cells within 5 tiles of the edge are
        // never framed, so a synthetic bench straddling the guard line dies
        // only cell-wise when the surviving cell is outside the guard's
        // reach — and cascades normally one tile further in.
        let mut w = object_world();
        set_object_cell(&mut w, 5, 19, 18, 0, 0);
        set_object_cell(&mut w, 6, 19, 18, 18, 0);
        let kill = kill_tile(&mut w, 6, 19, false).unwrap();
        assert_all_gone(&w, &[(6, 19)]);
        assert!(w.tile(5, 19).unwrap().active(), "guard skips x = 5");
        assert!(kill.object_drops.is_empty(), "no check ran");

        let mut w = object_world();
        set_object_cell(&mut w, 6, 19, 18, 0, 0);
        set_object_cell(&mut w, 7, 19, 18, 18, 0);
        let kill = kill_tile(&mut w, 7, 19, false).unwrap();
        assert_all_gone(&w, &[(6, 19), (7, 19)]);
        assert_eq!(kill.object_drops.len(), 1);
    }

    #[test]
    fn test_DW_3_1_lone_object_cell_drops_nothing() {
        // Vanilla quirk kept 1:1: a lone chest cell (the rest of the object
        // already gone — synthetic state) is killed without a drop — no
        // sibling cell survives to run the check, and the per-cell drop
        // table is empty for object kinds (`WorldGen.cs:65748-66091`).
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 21, 0, 0);
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_eq!(kill.kind, 21);
        assert!(w.tile(10, 19).is_some_and(|t| !t.active()));
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn legacy_tiles_keep_their_single_cell_break() {
        // A non-registry kind breaks as before: one cell, the legacy drop
        // position (the hit cell), no cascade records.
        let mut w = object_world();
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1);
        assert!(!w.tile(10, 20).unwrap().active());
        assert!(
            w.tile(9, 20).unwrap().active(),
            "neighbouring floor untouched"
        );
        assert!(kill.object_drops.is_empty());
        // Inactive and out-of-world cells return None (:63519-63532).
        assert!(kill_tile(&mut w, 10, 20, false).is_none());
        assert!(kill_tile(&mut w, -1, 20, false).is_none());
        assert!(kill_tile(&mut w, 10, 200, false).is_none());
    }

    #[test]
    fn platforms_count_as_support_and_their_death_still_pops_the_sitter() {
        // `CheckChest`'s support test passes on platforms
        // (`Main.tileSolid[19] = true`, `Main.cs:7893`, :57953-57958) and
        // `Check3x2`'s `SolidTileAllowBottomSlope` includes solidTop
        // (:70167-70185, :49396-49399). Both objects survive a neighbour kill
        // on the platform row; breaking the platform under a column still
        // destroys the sitter (the support-inactive path).
        let mut w = object_world();
        for x in 9..=17 {
            set_object_cell(&mut w, x, 19, 19, 0, 0); // a platform row
        }
        // Chest (10..11, 17..18) and table (14..16, 17..18) with canonical
        // style-0 frames, standing on the platform.
        for (x, y, kind, fx_off) in [(10, 17, 21, 0), (14, 17, 14, 0)] {
            for dy in 0..2i32 {
                for dx in 0..kind_widths(kind) {
                    set_object_cell(
                        &mut w,
                        x + dx,
                        y + dy,
                        kind,
                        (fx_off + dx * 18) as i16,
                        (dy * 18) as i16,
                    );
                }
            }
        }
        // A neighbour kill on the platform row leaves both intact.
        let kill = kill_tile(&mut w, 12, 19, false).unwrap();
        assert_eq!(kill.kind, 19);
        assert!(kill.object_drops.is_empty(), "both sitters supported");
        for x in 10..=11 {
            assert!(w.tile(x, 17).unwrap().active(), "chest {x}");
        }
        for x in 14..=16 {
            assert!(w.tile(x, 17).unwrap().active(), "table {x}");
        }
        // Killing the platform under the chest's left column pops the chest;
        // the table keeps its platform.
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_all_gone(&w, &[(10, 17), (11, 17), (10, 18), (11, 18)]);
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 21);
        for x in 14..=16 {
            assert!(w.tile(x, 17).unwrap().active(), "table {x} survives");
        }
        // And the platform's death under a table column pops the table.
        let kill = kill_tile(&mut w, 15, 19, false).unwrap();
        assert_all_gone(
            &w,
            &[(14, 17), (15, 17), (16, 17), (14, 18), (15, 18), (16, 18)],
        );
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 14);
    }

    /// The wooden def widths for the synthetic frames above.
    fn kind_widths(kind: u16) -> i32 {
        match kind {
            21 => 2,
            14 => 3,
            _ => 1,
        }
    }

    // ---- M1.4-TO-4 phase 2 (DW-2.1..2.5): the KillTile cascade arms --------

    /// Kills the active block at `(x, y)` — the neighbouring break whose
    /// `SquareTileFrame` pass frames the fixture object.
    fn trigger_at(w: &mut World, x: i32, y: i32) -> TileKill {
        kill_tile(w, x, y, false).unwrap()
    }

    /// A herb of `kind`/`frame_x` at (10, 19) on a `soil` floor with a stone
    /// trigger block at (11, 19) — killing the block frames the herb's cell
    /// (its 3x3 covers x 10..=12). All three alch kinds carry the
    /// frame-important boot static (`Main.cs:7685-7687`) because the
    /// Fireblossom blink rewrites the kind under the cascade's re-frame.
    /// The soil row rests on a stone row at y = 21: the pot soils 78/380 are
    /// registry kinds whose `CheckOnTable1x1` (:43534-43540) kills an
    /// unsupported pot mid-cascade — taking the herb with it (:45983-45990)
    /// — so the "survives on its own soils" scenario needs the pots
    /// supported, as a planted pot always is. The row is inert for the plain
    /// soils (nothing frames or judges y = 21).
    fn herb_world(soil: u16, kind: u16, frame_x: i16) -> World {
        let mut w = soil_world(soil);
        set_object_cell(&mut w, 10, 19, kind, frame_x, 0);
        for alch in crate::tileobject::TILE_ALCH {
            w.frame_important[*alch as usize] = true;
        }
        set_active(&mut w, 11, 19, 1);
        for x in 0..40 {
            set_active(&mut w, x, 21, 1);
        }
        w
    }

    #[test]
    fn test_store_1_piggy_survives_on_a_table_and_dies_off_it() {
        // `Check2x1`'s case-29 arm (`WorldGen.cs:47003-47013`): both bottom
        // cells must hold an active table tile. A stone kill ABOVE the piggy
        // frames its cells (the cascade's 3x3 walk) without touching them.
        let piggy_world = |table: bool| -> World {
            let mut w = soil_world(2);
            if table {
                set_object_cell(&mut w, 10, 21, 14, 0, 0);
                set_object_cell(&mut w, 11, 21, 14, 18, 0);
                set_object_cell(&mut w, 12, 21, 14, 36, 0);
            }
            set_object_cell(&mut w, 10, 20, 29, 0, 0);
            set_object_cell(&mut w, 11, 20, 29, 18, 0);
            set_active(&mut w, 10, 19, 1); // the cascade trigger block
            w
        };
        // On the table: the pair survives the pass-through framing.
        let mut w = piggy_world(true);
        trigger_at(&mut w, 10, 19);
        let t = w.tile(10, 20).unwrap();
        assert!(t.active() && t.kind == 29, "the piggy sits on the table");
        // Off the table: the pass-through framing kills the pair through the
        // case-29 arm (the piggy drops item 87).
        let mut w = piggy_world(false);
        let kill = trigger_at(&mut w, 10, 19);
        assert!(
            !w.tile(10, 20).unwrap().active(),
            "no table below — the case-29 arm kills"
        );
        assert!(!kill.object_drops.is_empty(), "the piggy drops item 87");
        // The direct hit kills through the piggy's own check.
        let mut w = piggy_world(true);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 29);
        assert!(!w.tile(10, 20).unwrap().active(), "the direct hit kills");
    }

    #[test]
    fn test_DW_2_1_herb_dies_without_support() {
        // `CheckAlch` (:45983-45990): the soil below must be `nactive` —
        // breaking the soil block under the herb kills it through the
        // cascade. No drop and no pot event (the herb per-cell drops are
        // phase 3).
        let mut w = soil_world(2);
        set_object_cell(&mut w, 10, 19, 83, 0, 0);
        for alch in crate::tileobject::TILE_ALCH {
            w.frame_important[*alch as usize] = true;
        }
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2, "the dirt block is the hit cell");
        assert!(!w.tile(10, 19).unwrap().active(), "the herb died");
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
        // A direct hit on the herb itself kills it alone.
        let mut w = soil_world(2);
        set_object_cell(&mut w, 10, 19, 84, 18, 0);
        for alch in crate::tileobject::TILE_ALCH {
            w.frame_important[*alch as usize] = true;
        }
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_eq!(kill.kind, 84);
        assert!(!w.tile(10, 19).unwrap().active());
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_DW_2_1_herb_soil_sets_by_variant() {
        // `CheckAlch`'s per-variant soil sets (:45998-46086) — the same
        // families `PlaceAlch` plants on (78/380 included everywhere): a
        // mature herb survives a passing cascade on its own soils and dies
        // off them (stone 1 is in no set).
        for (style, soils) in ALCH_SOIL_TABLE {
            let frame_x = (18 * style) as i16;
            for soil in soils {
                let mut w = herb_world(*soil, 83, frame_x);
                trigger_at(&mut w, 11, 19);
                let t = w.tile(10, 19).unwrap();
                assert!(
                    t.active() && t.kind == 83 && t.frame_x == frame_x,
                    "style {style} survives on soil {soil}"
                );
            }
            let mut w = herb_world(1, 83, frame_x);
            trigger_at(&mut w, 11, 19);
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "style {style} dies on stone"
            );
        }
    }

    #[test]
    fn test_DW_2_1_herb_half_brick_kills_slope_survives() {
        // :45983-45990 — the support test is `nactive` + `halfBrick` only:
        // a half-brick soil kills the herb, a sloped one does not (no slope
        // test exists in `CheckAlch`).
        let mut w = herb_world(2, 83, 0);
        let soil = w.index(10, 20);
        w.tiles[soil].flags |= crate::tile::FLAG_HALF_BRICK;
        trigger_at(&mut w, 11, 19);
        assert!(!w.tile(10, 19).unwrap().active(), "half brick kills");

        let mut w = herb_world(2, 83, 0);
        let soil = w.index(10, 20);
        w.tiles[soil].set_slope(2);
        trigger_at(&mut w, 11, 19);
        let t = w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 83, "a sloped soil passes");
    }

    #[test]
    fn test_DW_2_1_lava_kills_by_style() {
        // Every variant except 5 dies when the herb cell itself holds lava
        // (`liquid > 0 && lava()`, :46006-46008 and peers); the Fireblossom
        // (style 5) never does — its lava handling is the blink.
        for style in 0..=6i32 {
            let soil = ALCH_SOIL_TABLE[style as usize].1[0];
            let mut w = herb_world(soil, 83, (18 * style) as i16);
            set_liquid(&mut w, 10, 19, 100, true);
            trigger_at(&mut w, 11, 19);
            assert_eq!(
                w.tile(10, 19).unwrap().active(),
                style == 5,
                "style {style} in lava"
            );
        }
        // Water is not a `CheckAlch` death (the global `WaterCheck` path is
        // out of scope): a style-0 herb in water survives the framing.
        let mut w = herb_world(2, 83, 0);
        set_liquid(&mut w, 10, 19, 200, false);
        trigger_at(&mut w, 11, 19);
        assert!(w.tile(10, 19).unwrap().active(), "water does not kill");
    }

    #[test]
    fn test_DW_2_1_fireblossom_blinks_liquid_16_17() {
        // :46052-46075 — the style-5 lava branch: `liquid > 16` turns a
        // mature 83 into the blooming 84, `liquid <= 16` turns an 84 back;
        // no death either way and the frames stay (the stage is the type).
        // The boundary pair 16/17 plus the guard cases (82 never blinks, no
        // lava no blink).
        let cases: [(u16, u8, u16); 6] = [
            (83, 17, 84), // 83 above the boundary blooms
            (84, 16, 83), // 84 at/below it reverts
            (84, 17, 84), // 84 above stays blooming
            (83, 16, 83), // 83 at/below stays mature
            (82, 17, 82), // the immature herb never blinks
            (83, 0, 83),  // no lava: the branch is skipped
        ];
        for (kind, liquid, expected) in cases {
            let mut w = herb_world(57, kind, 90); // style 5, ash soil
            set_liquid(&mut w, 10, 19, liquid, true);
            trigger_at(&mut w, 11, 19);
            let t = w.tile(10, 19).unwrap();
            assert!(t.active(), "kind {kind} liquid {liquid} survives");
            assert_eq!(t.kind, expected, "kind {kind} liquid {liquid}");
            assert_eq!((t.frame_x, t.frame_y), (90, 0), "frames untouched");
        }
    }

    #[test]
    fn herb_frame_y_is_normalized_and_out_of_range_style_unchecked() {
        // :45991 — `frameY = 0` unconditionally (a gated write in the port);
        // :45996 — a variant outside 0-6 matches no switch case and the herb
        // is judged on the support alone (the seed fall-through's mirror).
        let mut w = herb_world(2, 83, 0);
        w.tile_mut(10, 19).unwrap().frame_y = 36;
        trigger_at(&mut w, 11, 19);
        let t = w.tile(10, 19).unwrap();
        assert!(t.active() && (t.frame_x, t.frame_y) == (0, 0), "normalized");

        let mut w = herb_world(1, 82, 162); // style 9 on stone
        trigger_at(&mut w, 11, 19);
        assert!(w.tile(10, 19).unwrap().active(), "no case: unchecked");
    }

    #[test]
    fn test_DW_2_1_herb_in_a_pot_dies_when_the_pot_cascade_kills() {
        // Neither `CheckOnTable1x1` (:43503-43546) nor `CheckAlch`
        // (:45972-46092) carries the `destroyObject` guard or wraps its
        // `KillTile` in the flag: breaking the floor under a potted herb
        // kills the pot through its own check, and the pot's plain `KillTile`
        // re-frames the herb cell, whose `CheckAlch` sees the vanished
        // support (78 is in every style's soil set, :45998-46086) and kills
        // it — the herb's drop roll (`KillTile_GetItemDrops` :65674-65698)
        // rides out of the nested kill as an event.
        let mut w = clay_pot_world();
        set_object_cell(&mut w, 10, 18, 83, 0, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1, "the floor block is the hit cell");
        assert_all_gone(&w, &[(10, 19), (10, 18)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 78,
                x: 10,
                y: 19,
                item: None,
            }]
        );
        assert_eq!(
            kill.herb_events,
            vec![HerbEvent {
                kind: 83,
                style: 0,
                x: 10,
                y: 18
            }]
        );
    }

    /// The eight cells of the canonical style-0 sunflower at (10..11,
    /// 16..19) on a dirt floor.
    const SUNFLOWER_CELLS: [(i32, i32); 8] = [
        (10, 16),
        (11, 16),
        (10, 17),
        (11, 17),
        (10, 18),
        (11, 18),
        (10, 19),
        (11, 19),
    ];

    /// [`sunflower_world`] with the floor repainted to `soil` and the sheet
    /// column shifted to `style` (`frameX = dx*18 + style*36`).
    fn sunflower_world_with_soil(soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        for dy in 0..4i32 {
            for dx in 0..2i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    16 + dy,
                    27,
                    (dx * 18 + style * 36) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        w
    }

    fn sunflower_world() -> World {
        sunflower_world_with_soil(2, 0)
    }

    #[test]
    fn test_DW_2_2_sunflower_cascades_from_every_cell() {
        // `CheckSunflower` (:53729-53792): any of the eight cells triggers
        // the full destruction and the sunflower item drops at the check's
        // trigger cell (:53787-53791 — `Item.NewItem(…, 63)`), carried as
        // the record's `item`.
        for hit in SUNFLOWER_CELLS {
            let mut w = sunflower_world();
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 27);
            assert_all_gone(&w, &SUNFLOWER_CELLS);
            let (dx, dy) = expected_drop(&SUNFLOWER_CELLS, hit, 27);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 27,
                    x: dx,
                    y: dy,
                    item: Some(63),
                }],
                "hit {hit:?}"
            );
            assert!(kill.events.is_empty());
        }
    }

    #[test]
    fn test_DW_2_2_sunflower_ground_set_and_solidtile() {
        // :53756-53770 — the ground must be `nactive`, in {2,477,492,109,60,
        // 633} AND `SolidTile`: dirt passes, stone (1) and the clay pot 78
        // are not in the sunflower set (unlike the herbs), a half brick
        // fails `SolidTile`. The frame column decode is style-agnostic
        // (`frameX / 18` mod 2 only) — the style-2 variant checks the same.
        let mut w = sunflower_world();
        set_active(&mut w, 9, 19, 1); // a stone block beside the stem
        kill_tile(&mut w, 9, 19, false).unwrap();
        for x in [10, 11] {
            assert!(w.tile(x, 19).unwrap().active(), "dirt ground holds");
        }

        for soil in [1u16, 78] {
            let mut w = sunflower_world_with_soil(soil, 0);
            set_active(&mut w, 9, 19, 1);
            kill_tile(&mut w, 9, 19, false).unwrap();
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "soil {soil} refuses the sunflower"
            );
        }

        let mut w = sunflower_world();
        let soil = w.index(10, 20);
        w.tiles[soil].flags |= crate::tile::FLAG_HALF_BRICK;
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        assert!(!w.tile(10, 19).unwrap().active(), "half brick ground kills");

        // The style-2 sheet column (frames 72/90) passes the same checks.
        let mut w = sunflower_world_with_soil(2, 2);
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        for (x, y) in SUNFLOWER_CELLS {
            assert!(w.tile(x, y).unwrap().active(), "({x}, {y})");
        }
    }

    #[test]
    fn test_DW_2_2_support_break_drops_63() {
        // Mining the ground under one stem column: the cascade reaches the
        // sunflower through `SquareTileFrame`, all eight cells die and the
        // item-63 record spawns (per-cell drops stay empty — 27 is in the
        // empty `KillTile_GetItemDrops` group, :65759+).
        let mut w = sunflower_world();
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2, "the dirt block is the hit cell");
        assert_all_gone(&w, &SUNFLOWER_CELLS);
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(
            (
                kill.object_drops[0].tile_kind,
                kill.object_drops[0].x,
                kill.object_drops[0].y,
                kill.object_drops[0].item
            ),
            (27, 10, 19, Some(63))
        );
    }

    /// A 2x2 pot of `style` at (10..11, 18..19) on the stone floor.
    fn pot_world(style: i32) -> World {
        let mut w = object_world();
        for dy in 0..2i32 {
            for dx in 0..2i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    18 + dy,
                    28,
                    (dx * 18) as i16,
                    (dy * 18 + style * 36) as i16,
                );
            }
        }
        w
    }

    /// The four cells of [`pot_world`].
    const POT_CELLS: [(i32, i32); 4] = [(10, 18), (11, 18), (10, 19), (11, 19)];

    #[test]
    fn test_DW_2_3_pot_cascades_from_every_cell() {
        // `CheckPot` (:56797-57016): any of the four cells triggers the full
        // destruction; no item record exists for tile 28 (the loot runs
        // through the pot event, :57011-57014) — `object_drops` stays empty
        // and the event carries the check's trigger cell.
        for hit in POT_CELLS {
            let mut w = pot_world(0);
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 28);
            assert_all_gone(&w, &POT_CELLS);
            assert!(kill.object_drops.is_empty(), "no items for tile 28");
            let (tx, ty) = expected_trigger(&POT_CELLS, hit);
            assert_eq!(
                kill.events,
                vec![PotEvent {
                    style: 0,
                    x: tx,
                    y: ty
                }],
                "hit {hit:?}"
            );
        }
    }

    #[test]
    fn test_DW_2_3_pot_style_decode_and_event() {
        // :56812-56817 — the style is the `frameY / 36` row (`num4`),
        // independent of the `frameX` sheet column. A support break yields
        // the same event.
        for style in [0i32, 8, 20] {
            let mut w = pot_world(style);
            let kill = kill_tile(&mut w, 10, 20, false).unwrap();
            assert_eq!(kill.kind, 1);
            assert_all_gone(&w, &POT_CELLS);
            assert_eq!(
                kill.events,
                vec![PotEvent {
                    style,
                    x: 10,
                    y: 19
                }],
                "style {style}"
            );
        }
    }

    #[test]
    fn test_DW_2_3_pot_break_sound_by_style() {
        // :56856-56867 — styles 7-9 break with sound 6, 16-24 with 4,
        // everything else (including 100) with 13.
        for (style, sound) in [
            (0i32, 13u16),
            (6, 13),
            (7, 6),
            (9, 6),
            (10, 13),
            (15, 13),
            (16, 4),
            (24, 4),
            (25, 13),
            (100, 13),
        ] {
            assert_eq!(pot_break_sound(style), sound, "style {style}");
        }
    }

    #[test]
    fn s1_e_21c_kill_tile_sound_table() {
        // `WorldGen.KillTile_PlaySounds` (`WorldGen.cs:66427-66575`) — the
        // head arms, the switch arms, the default and the shatter tail.
        let s = |kind: u16, fx: i16, fail: bool| kill_tile_sound(kind, fx, fail);
        // :66434-66437 — the Item27 family (a real break keeps the tail off).
        assert_eq!(s(127, 0, false).primary, Some((2, 27)));
        assert_eq!(s(623, 0, false).primary, Some((2, 27)));
        // :66438-66448 — 147/224 resolve to Item48 (D3: the roll is pinned).
        assert_eq!(s(147, 0, false).primary, Some((2, 48)));
        assert_eq!(s(224, 0, false).primary, Some((2, 48)));
        // :66449-66452 / :66453-66456
        assert_eq!(s(161, 0, false).primary, Some((2, 50)));
        assert_eq!(s(736, 0, false).primary, Some((2, 50)));
        assert_eq!(s(518, 0, false).primary, Some((6, 1)));
        assert_eq!(s(636, 0, false).primary, Some((6, 1)));
        // :66457-66464 — the two frame-gated grass arms.
        assert_eq!(s(530, 269, false).primary, Some((6, 1)));
        assert_eq!(s(530, 270, false).primary, Some((0, 1)));
        // :66461 — `frameX % 6 < 270` is always true (the C# quirk), so 705
        // is grass at every frame.
        assert_eq!(s(705, 0, false).primary, Some((6, 1)));
        assert_eq!(s(705, 276, false).primary, Some((6, 1)));
        // :66469-66478 — the switch's grass arms.
        for kind in [3u16, 110, 254, 24] {
            assert_eq!(s(kind, 0, false).primary, Some((6, 1)), "kind {kind}");
        }
        // :66480-66484 — `Main.tileAlch` (82/83/84) + the hair/herb list.
        for kind in [82u16, 83, 84, 384, 227, 32] {
            assert_eq!(s(kind, 0, false).primary, Some((6, 1)), "kind {kind}");
        }
        // :66485-66489 — the mummy's killed style 11.
        assert_eq!(s(485, 0, false).primary, Some((4, 11)));
        // :66495-66499
        assert_eq!(s(481, 0, false).primary, Some((2, 127)));
        // :66500-66504 — the dirt/stone list → Tink.
        assert_eq!(s(1, 0, false).primary, Some((21, 1)));
        assert_eq!(s(750, 0, false).primary, Some((21, 1)));
        // :66505-66509 / :66510-66514 / :66515-66523
        assert_eq!(s(666, 0, false).primary, Some((2, 177)));
        assert_eq!(s(739, 0, false).primary, Some((3, 25)));
        assert_eq!(s(231, 0, false).primary, Some((4, 1)));
        assert_eq!(s(231, 0, false).extra, Some((2, 173)));
        assert_eq!(s(195, 0, false).extra, None);
        // :66524-66528 — the frame-gated altar arm.
        assert_eq!(s(26, 54, false).primary, Some((4, 1)));
        assert_eq!(s(26, 53, false).primary, Some((0, 1)));
        // :66529-66539
        assert_eq!(s(314, 0, false).primary, Some((2, 52)));
        assert_eq!(s(331, 0, false).primary, Some((18, 1)));
        // :66540-66549 — the default: the glass family only on a real break.
        assert_eq!(s(162, 0, false).primary, Some((2, 27)));
        assert_eq!(s(162, 0, true).primary, Some((0, 1)));
        assert_eq!(s(165, 0, false).primary, Some((2, 27)));
        assert_eq!(s(165, 54, false).primary, Some((0, 1)));
        assert_eq!(s(138, 0, false).primary, None, "138 breaks silently");
        assert_eq!(s(30, 0, false).primary, Some((0, 1)));
        // :66554-66574 — the shatter tail rides on `!fail` only.
        assert_eq!(s(13, 0, false).primary, Some((0, 1)));
        assert_eq!(s(13, 0, false).shatter, Some((13, 1)));
        assert_eq!(s(13, 0, true).shatter, None);
        for kind in [54u16, 326, 748] {
            assert_eq!(s(kind, 0, false).shatter, Some((13, 1)), "kind {kind}");
        }
        assert_eq!(s(21, 0, false).shatter, None);
    }

    #[test]
    fn test_DW_2_3_pot_ground_solid_tile2() {
        // :56845-56849 — the ground is `SolidTile2` (:70134-70147): solid
        // and slopeless and not actuated — a platform row passes
        // (`Main.tileSolid[19] = true`, `Main.cs:7893`), a half brick or an
        // actuated floor fails.
        let mut w = pot_world(0);
        for x in 10..=11 {
            set_object_cell(&mut w, x, 20, 19, 0, 0); // platforms under both
        }
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        for (x, y) in POT_CELLS {
            assert!(w.tile(x, y).unwrap().active(), "({x}, {y}) on platforms");
        }

        let mut w = pot_world(0);
        let soil = w.index(10, 20);
        w.tiles[soil].flags |= crate::tile::FLAG_HALF_BRICK;
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        assert!(!w.tile(10, 19).unwrap().active(), "half brick ground kills");

        let mut w = pot_world(0);
        let floor = w.index(10, 20);
        w.tiles[floor].flags |= crate::tile::FLAG_INACTIVE;
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        assert!(!w.tile(10, 19).unwrap().active(), "actuated ground kills");
    }

    #[test]
    fn test_DW_2_3_no_gore_items_spawn() {
        // The Gore-shard block (:56878-56953) has no port mechanism — a
        // broken pot spawns no items at all (the shard styles included) and
        // only the loot event leaves; the ANALYSIS entry lands in phase 5.
        for style in [0i32, 25, 31] {
            let mut w = pot_world(style);
            let kill = kill_tile(&mut w, 10, 20, false).unwrap();
            assert_all_gone(&w, &POT_CELLS);
            assert!(kill.object_drops.is_empty(), "style {style}");
            assert_eq!(kill.events.len(), 1, "style {style}");
        }
    }

    /// A 1x1 pile of `style` (`frameX = style * 18`, `frameY = 0`) at
    /// (10, 19) on a `soil` floor with a stone trigger block beside it.
    fn pile_1x1_world(soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        set_object_cell(&mut w, 10, 19, 185, (style * 18) as i16, 0);
        set_active(&mut w, 11, 19, 1);
        w
    }

    #[test]
    fn test_DW_2_4_pile_1x1_ground_rules() {
        // `CheckPile` (:46875-46971): `SolidTileAllowBottomSlope` below the
        // cell (:46898-46902), then the style's ground family
        // (:46915-46969) — 36-47 snow/ice (162/224 included), 54-59 and
        // 73-76 sand; the other styles judge nothing.
        for (style, soil) in [(36i32, 147u16), (47, 161), (54, 53), (59, 53), (73, 53)] {
            let mut w = pile_1x1_world(soil, style);
            trigger_at(&mut w, 11, 19);
            assert!(
                w.tile(10, 19).unwrap().active(),
                "style {style} on soil {soil}"
            );
        }
        // Off-family grounds kill.
        for (style, soil) in [(36i32, 2u16), (47, 2), (54, 2), (76, 2)] {
            let mut w = pile_1x1_world(soil, style);
            trigger_at(&mut w, 11, 19);
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "style {style} dies on dirt"
            );
        }
        // Styles outside the judged ranges are free.
        for style in [0i32, 35, 48, 60, 72, 77] {
            let mut w = pile_1x1_world(2, style);
            trigger_at(&mut w, 11, 19);
            assert!(
                w.tile(10, 19).unwrap().active(),
                "style {style} is unjudged"
            );
        }
        // A dead support kills regardless of style (:46898-46902).
        let mut w = pile_1x1_world(2, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2);
        assert!(!w.tile(10, 19).unwrap().active(), "lost support kills");
        assert!(kill.object_drops.is_empty(), "no items for tile 185");
        assert!(kill.events.is_empty());
    }

    /// A 2x1 pile of `style` (`frameY = 18`, `frameX = style*36 + dx*18`)
    /// at (10..11, 19) on a `soil` floor, with a stone trigger block at
    /// (12, 19).
    fn pile_2x1_world(soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        set_object_cell(&mut w, 10, 19, 185, (style * 36) as i16, 18);
        set_object_cell(&mut w, 11, 19, 185, (style * 36 + 18) as i16, 18);
        set_active(&mut w, 12, 19, 1);
        w
    }

    #[test]
    fn test_DW_2_4_pile_2x1_support_and_styles() {
        // The `Check2x1` type-185 arm (:47040-47086): both columns need
        // `SolidTileAllowBottomSlope` and no Boulder, then the `frameX / 36`
        // style's family — 25-30 snow/ice, 41-46 and 62-64 sand.
        // Unjudged styles stand anywhere.
        for style in [0i32, 24, 31] {
            let mut w = pile_2x1_world(2, style);
            trigger_at(&mut w, 12, 19);
            for x in [10, 11] {
                assert!(w.tile(x, 19).unwrap().active(), "style {style}");
            }
        }
        // Snow styles on snow hold, on dirt die; sand styles mirror.
        for (style, good, bad) in [(25i32, 147u16, 2u16), (41, 53, 2), (62, 53, 2)] {
            let mut w = pile_2x1_world(good, style);
            trigger_at(&mut w, 12, 19);
            assert!(w.tile(10, 19).unwrap().active(), "style {style} on {good}");
            let mut w = pile_2x1_world(bad, style);
            trigger_at(&mut w, 12, 19);
            assert!(!w.tile(10, 19).unwrap().active(), "style {style} on {bad}");
        }
        // A Boulder under one column refuses the support (:47052-47055).
        let mut w = pile_2x1_world(2, 0);
        set_active(&mut w, 11, 20, 138);
        trigger_at(&mut w, 12, 19);
        assert!(!w.tile(10, 19).unwrap().active(), "boulder refuses");
        // A lost support column kills; no items for tile 185 (the coin-pile
        // rolls are phase-3 RNG).
        let mut w = pile_2x1_world(2, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2);
        assert!(!w.tile(10, 19).unwrap().active());
        assert!(!w.tile(11, 19).unwrap().active());
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
    }

    #[test]
    fn test_DW_2_4_pile_2x1_grass_cover_conversion() {
        // :47041-47048 — the grass-covered variants (`frameX` 1368-1458)
        // revert (`frameX -= 1368`) when neither ground cell is dirt/grass,
        // BEFORE the style is re-judged: over stone the converted pile
        // survives with bare frames; over dirt no conversion happens and
        // the covered pile keeps its frames; over stone the covered snow
        // style (1404 → 39 → 36) converts and the stone ground kills.
        let mut w = pile_2x1_world(1, 38); // frameX 1368 over stone
        trigger_at(&mut w, 12, 19);
        assert!(w.tile(10, 19).unwrap().active(), "converted pile survives");
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 0, "reverted to bare");
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 18);

        let mut w = pile_2x1_world(2, 38); // over dirt: no conversion
        trigger_at(&mut w, 12, 19);
        assert!(w.tile(10, 19).unwrap().active());
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 1368, "frames kept");

        let mut w = pile_2x1_world(1, 39); // 1404 → 36 → num5 = 1 (unjudged)
        trigger_at(&mut w, 12, 19);
        assert!(w.tile(10, 19).unwrap().active(), "converted style 1 passes");
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36);
    }

    /// A 3x2 pile of `kind`/`style` at (10..12, 18..19) on a `soil` floor,
    /// with a stone trigger block at (13, 19).
    fn pile_3x2_world(kind: u16, soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    18 + dy,
                    kind,
                    (dx * 18 + style * 54) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        set_active(&mut w, 13, 19, 1);
        // Both pile kinds are boot-static frame-important (`Main.cs:7416`/
        // `7417`) — the 187→186 conversion rewrites the kind under the
        // cascade's re-frame, so both bits must be set.
        w.frame_important[186] = true;
        w.frame_important[187] = true;
        w
    }

    #[test]
    fn test_DW_2_4_pile_3x2_support_and_styles() {
        // `Check3x2`'s 186/187 arms (:49300-49373): `SolidTileAllowBottomSlope`
        // plus no Boulder per column, then the style's ground family — 186:
        // 26-31 snow/ice, 32-34 mud/70; 187: 0-5 mud/60/226, 6-8 ash,
        // 29-34 sand. Unjudged styles stand on any solid ground.
        for (kind, style, soil) in [
            (186u16, 0i32, 1u16),
            (186, 26, 147),
            (186, 32, 59),
            (187, 0, 59),
            (187, 6, 57),
            (187, 29, 53),
            (187, 9, 1),
        ] {
            let mut w = pile_3x2_world(kind, soil, style);
            trigger_at(&mut w, 13, 19);
            for x in 10..=12 {
                assert!(
                    w.tile(x, 18).unwrap().active(),
                    "kind {kind} style {style} on {soil}"
                );
            }
        }
        for (kind, style, soil) in [(186u16, 26i32, 2u16), (187, 0, 2), (187, 29, 2)] {
            let mut w = pile_3x2_world(kind, soil, style);
            trigger_at(&mut w, 13, 19);
            for x in 10..=12 {
                assert!(
                    !w.tile(x, 18).unwrap().active(),
                    "kind {kind} style {style} dies on {soil}"
                );
            }
        }
        // A Boulder under one column refuses the whole pile.
        for kind in [186u16, 187] {
            let mut w = pile_3x2_world(kind, 1, 0);
            set_active(&mut w, 11, 20, 138);
            trigger_at(&mut w, 13, 19);
            for x in 10..=12 {
                assert!(
                    !w.tile(x, 18).unwrap().active(),
                    "kind {kind} over a boulder dies"
                );
            }
        }
        // A broken support column kills (:49396-49399 arm); no items for
        // the piles (the empty case group, :65759+).
        let mut w = pile_3x2_world(186, 1, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1);
        for x in 10..=12 {
            for y in 18..=19 {
                assert!(!w.tile(x, y).unwrap().active());
            }
        }
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
    }

    #[test]
    fn test_DW_2_4_187_converts_to_186_frames() {
        // :49379-49393 — an 187 whose origin `frameX` is 756..=900 (the
        // snowy styles 14-16; canonical origins are 756/810/864, the 900
        // range end is only reachable on degenerate frames) over non-dirt
        // ground converts in place: every cell becomes a 186 with
        // `frameX -= 378` and the pile survives — even with a broken cell,
        // where the kill loop's kind filter then matches nothing.
        for style in [14i32, 15, 16] {
            let mut w = pile_3x2_world(187, 1, style);
            trigger_at(&mut w, 13, 19);
            for dy in 0..2i32 {
                for dx in 0..3i32 {
                    let t = w.tile(10 + dx, 18 + dy).unwrap();
                    assert!(t.active() && t.kind == 186, "style {style}");
                    assert_eq!(t.frame_x, (dx * 18 + (style - 7) * 54) as i16);
                    assert_eq!(t.frame_y, (dy * 18) as i16, "frameY untouched");
                }
            }
        }
        // Style 13 (origin 702) is below the range: no conversion.
        let mut w = pile_3x2_world(187, 1, 13);
        trigger_at(&mut w, 13, 19);
        let t = w.tile(10, 18).unwrap();
        assert!(
            t.active() && t.kind == 187,
            "below the range: no conversion"
        );
        assert_eq!(t.frame_x, 702);
        // Over dirt (2) the conversion refuses; style 14 has no ground case,
        // so the pile also survives as an 187.
        let mut w = pile_3x2_world(187, 2, 14);
        trigger_at(&mut w, 13, 19);
        let t = w.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 187 && t.frame_x == 756);
        // A broken cell with the conversion: the rewrite still runs (the
        // cells become 186, so this check's own kind-filtered kill loop
        // matches nothing), but the hole keeps the object broken — the next
        // framing visit re-checks the pile as a 186 and takes the
        // survivors. Nothing drops either way (the piles' case group is
        // empty).
        let mut w = pile_3x2_world(187, 1, 14);
        let gone = w.index(11, 19);
        w.tiles[gone] = Tile::default();
        let kill = trigger_at(&mut w, 13, 19);
        for (x, y) in [
            (10i32, 18i32),
            (11, 18),
            (12, 18),
            (10, 19),
            (11, 19),
            (12, 19),
        ] {
            assert!(!w.tile(x, y).unwrap().active(), "({x}, {y}) gone");
        }
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_DW_2_5_pile_on_boulder_refuses_support() {
        // `InvalidTileForPilesOrSpeleothems` (:38979-38994): an active
        // Boulder under any column sets the plan even with every other
        // support intact — the pile dies while the Boulder and the
        // neighbouring ground survive (the kill region never includes
        // supports).
        for kind in [186u16, 187] {
            let mut w = pile_3x2_world(kind, 1, 0);
            set_active(&mut w, 11, 20, 138);
            let kill = kill_tile(&mut w, 13, 19, false).unwrap();
            for x in 10..=12 {
                for y in 18..=19 {
                    assert!(!w.tile(x, y).unwrap().active(), "kind {kind}");
                }
            }
            let boulder = w.tile(11, 20).unwrap();
            assert!(boulder.active() && boulder.kind == 138, "boulder stays");
            assert!(w.tile(10, 20).unwrap().active(), "stone stays");
            assert!(kill.object_drops.is_empty());
        }
    }

    #[test]
    fn test_DW_2_5_intact_pile_survives_passing_cascade() {
        // A fully supported pile survives a cascade that merely passes by
        // (a neighbouring block breaks): the check runs and finds nothing.
        let mut w = pile_3x2_world(186, 1, 0);
        let kill = kill_tile(&mut w, 13, 19, false).unwrap();
        assert_eq!(kill.kind, 1);
        for x in 10..=12 {
            for y in 18..=19 {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 186, "({x}, {y}) survives");
                assert_eq!(t.frame_x, ((x - 10) * 18) as i16, "frames kept");
            }
        }
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
    }

    #[test]
    fn test_DW_2_5_conversion_spares_neighbours() {
        // The 187→186 rewrite touches exactly the converting pile's six
        // cells: an adjacent 186 keeps its kind and frames through the same
        // cascade (the trigger block sits between the two piles).
        let mut w = pile_3x2_world(187, 1, 14);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                set_object_cell(
                    &mut w,
                    14 + dx,
                    18 + dy,
                    186,
                    (dx * 18) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        trigger_at(&mut w, 13, 19);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                let t = w.tile(10 + dx, 18 + dy).unwrap();
                assert!(t.active() && t.kind == 186, "converted pile");
                assert_eq!(t.frame_x, (dx * 18 + 378) as i16);
                let n = w.tile(14 + dx, 18 + dy).unwrap();
                assert!(n.active() && n.kind == 186, "neighbour intact");
                assert_eq!(n.frame_x, (dx * 18) as i16, "neighbour frames kept");
            }
        }
    }

    #[test]
    fn style_25_table_spans_a_single_row() {
        // `Check3x2`'s `flag2` (:49223-49229): table style 25 is a 3x1 —
        // the support row is `oy + 1` and the cell region one row.
        let mut w = object_world();
        for dx in 0..3i32 {
            set_object_cell(&mut w, 10 + dx, 18, 14, (dx * 18 + 25 * 54) as i16, 0);
            set_active(&mut w, 10 + dx, 19, 1); // its own support row
        }
        set_active(&mut w, 9, 18, 1);
        kill_tile(&mut w, 9, 18, false).unwrap();
        for dx in 0..3i32 {
            assert!(
                w.tile(10 + dx, 18).unwrap().active(),
                "style-25 table supported"
            );
        }
        // Breaking its support row still kills it.
        let kill = kill_tile(&mut w, 11, 19, false).unwrap();
        assert_eq!(kill.kind, 1);
        for dx in 0..3i32 {
            assert!(!w.tile(10 + dx, 18).unwrap().active());
        }
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 14);
    }

    // ---- DW-2.6: the clay pot (78) — CheckOnTable1x1 -----------------------

    /// A 1x1 clay pot at (10, 19) on the stone floor — frames (0, 0), the
    /// single-cell origin (the arm reads no frames).
    fn clay_pot_world() -> World {
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 78, 0, 0);
        w
    }

    /// Sets a trigger stone at (9, 19) and kills it: the break's
    /// `SquareTileFrame` pass frames the pot's cell (its 3x3 covers
    /// x 8..=10, y 18..=20) without touching the configured support row at
    /// (10, 20).
    fn pot_trigger(w: &mut World) {
        set_active(w, 9, 19, 1);
        trigger_at(w, 9, 19);
    }

    #[test]
    fn test_DW_2_6_pot_support_break_kills() {
        // `CheckOnTable1x1` (`WorldGen.cs:43503-43546`): mining the block
        // under the pot leaves air below — no slope, no half brick, no table
        // anchor and `SolidTileAllowBottomSlope` false (:43534-43540) — the
        // pot dies. The drop record is per-cell (`KillTile_GetItemDrops`
        // case 78 → 222, :64711-64713; resolved through `registry::tile_drop`
        // in phase 3, so `item` is `None`).
        let mut w = clay_pot_world();
        let kill = trigger_at(&mut w, 10, 20);
        assert_eq!(kill.kind, 1, "the floor block is the hit cell");
        assert_all_gone(&w, &[(10, 19)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 78,
                x: 10,
                y: 19,
                item: None,
            }]
        );
    }

    #[test]
    fn test_DW_2_6_pot_survives_on_plain_solid_ground() {
        // :43534-43540 — the pot stands on ANY `SolidTileAllowBottomSlope`
        // below (the stone floor): a cascade pass over a neighbouring cell
        // leaves it standing.
        let mut w = clay_pot_world();
        trigger_at(&mut w, 9, 20);
        assert!(
            w.tile(10, 19).unwrap().active(),
            "pot survives on solid ground"
        );
    }

    #[test]
    fn test_DW_2_6_pot_half_brick_below_kills() {
        // :43525-43528 — the half-brick arm runs before the table anchors
        // and the type-78 arm (the C# order: topSlope → halfBrick → anchors
        // → 78).
        let mut w = clay_pot_world();
        let floor = w.index(10, 20);
        w.tiles[floor].flags |= crate::tile::FLAG_HALF_BRICK;
        trigger_at(&mut w, 9, 20);
        assert!(!w.tile(10, 19).unwrap().active(), "half brick below kills");
    }

    #[test]
    fn test_DW_2_6_pot_top_slope_below_kills_unless_platform_corner() {
        // :43510-43524 — a top slope below kills (:43519-43521 for
        // non-platforms) unless the sloped platform has its corner partner:
        // `blockType` 3 (slope 2) needs the active flat platform at `x - 1`,
        // `blockType` 2 (slope 1) at `x + 1` (:43513-43515). A bottom slope
        // is no top slope and falls through to the solid arm — the pot
        // stands (:43534-43540).
        let platform_below = |slope: u8| {
            let mut w = clay_pot_world();
            set_object_cell(&mut w, 10, 20, 19, 0, 0); // platform below the pot
            w.tile_mut(10, 20).unwrap().set_slope(slope);
            w
        };
        // Stone slope 2 — a top slope on a non-platform kills.
        let mut w = clay_pot_world();
        w.tile_mut(10, 20).unwrap().set_slope(2);
        pot_trigger(&mut w);
        assert!(!w.tile(10, 19).unwrap().active(), "top slope kills");

        // Sloped platform without the corner partner (stone at x - 1) kills.
        let mut w = platform_below(2);
        pot_trigger(&mut w);
        assert!(
            !w.tile(10, 19).unwrap().active(),
            "cornerless platform slope kills"
        );

        // Sloped platform with the flat platform at `x - 1` holds the pot.
        let mut w = platform_below(2);
        set_object_cell(&mut w, 9, 20, 19, 0, 0);
        pot_trigger(&mut w);
        assert!(
            w.tile(10, 19).unwrap().active(),
            "platform corner partner holds"
        );

        // A bottom slope (3) is not a top slope — `SolidTileAllowBottomSlope`
        // carries the pot.
        let mut w = clay_pot_world();
        w.tile_mut(10, 20).unwrap().set_slope(3);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "bottom slope holds");
    }

    #[test]
    fn test_DW_2_6_pot_stands_on_a_platform() {
        // :43534-43540 — a platform is solidTop, so `SolidTileAllowBottomSlope`
        // carries the pot (the `AnchorValid` platform arm :51857-51863 reaches
        // the same verdict for the flat segments).
        let mut w = clay_pot_world();
        set_object_cell(&mut w, 10, 20, 19, 0, 0);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "pot stands on a platform");
    }

    #[test]
    fn test_DW_2_6_pot_survives_on_a_table_even_actuated() {
        // :43530-43532 — the anchors survive before the type test
        // (`AnchorValid(…, AnchorType.Table)` :51841-51871 or the plain
        // `tile.active() && Main.tileTable[...]` arm). An intact registry
        // table (14, style 0) under the pot holds it, and the actuated twin
        // still does: the plain disjunct reads `active()`, not `nactive()` —
        // the `nactive` gate (:51844) only silences the anchor arms, and the
        // disjunct keeps the pot. (The pre-registry fixture — a lone tile-87
        // cell — died with the furniture registry work: 87/88 are registered
        // and `TileFrameImportant` (:85980-85989) routes them to `Check3x2`,
        // so a lone piano cell collapses under the pot exactly like the C#.)
        let table_under_pot = |w: &mut World| {
            for dy in 0..2i32 {
                for dx in 0..3i32 {
                    set_object_cell(w, 9 + dx, 20 + dy, 14, (dx * 18) as i16, (dy * 18) as i16);
                }
            }
            for dx in 0..3i32 {
                set_active(w, 9 + dx, 22, 1); // the table's support row
            }
        };
        let mut w = clay_pot_world();
        table_under_pot(&mut w);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "pot stands on a table");

        let mut w = clay_pot_world();
        table_under_pot(&mut w);
        let below = w.index(10, 20);
        w.tiles[below].flags |= crate::tile::FLAG_INACTIVE;
        pot_trigger(&mut w);
        assert!(
            w.tile(10, 19).unwrap().active(),
            "actuated table still holds the pot"
        );
    }

    #[test]
    fn test_DW_2_6_pot_on_a_table_dies_when_the_table_cascade_kills() {
        // `CheckOnTable1x1` has no `destroyObject` guard (:43503-43546), so
        // when the registry table's `Check3x2` kill loop (:49416-49423) takes
        // the pot's support cell, the re-framed pot is re-checked inside the
        // loop and dies — while the table's own cells stay silent (guarded).
        // Its 222 record (`KillTile_GetItemDrops` case 78, :64711-64713)
        // rides out of the nested kill after the table's own record.
        let mut w = object_world();
        // Style-0 table 3x2 at (10..12, 18..19); the pot stands on the
        // top-middle cell.
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    18 + dy,
                    14,
                    (dx * 18) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        set_object_cell(&mut w, 11, 17, 78, 0, 0);
        // The hit bottom-left table cell is two rows below the pot: the pot
        // survives the first framing pass (its support is still alive) and
        // its verdict comes from inside `Check3x2`'s flag-wrapped loop.
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_eq!(kill.kind, 14);
        assert_all_gone(
            &w,
            &[
                (10, 18),
                (10, 19),
                (11, 18),
                (11, 19),
                (12, 18),
                (12, 19),
                (11, 17),
            ],
        );
        assert_eq!(
            kill.object_drops,
            vec![
                ObjectDrop {
                    tile_kind: 14,
                    x: 10,
                    y: 18,
                    item: None,
                },
                ObjectDrop {
                    tile_kind: 78,
                    x: 11,
                    y: 17,
                    item: None,
                },
            ]
        );
    }

    #[test]
    fn test_DW_2_6_pot_on_a_workbench_dies_when_the_bench_cascade_kills() {
        // The review's repro: the bench (`Check2x1`, guarded :46976) holds
        // the pot through `Main.tileTable[18]` (`Main.cs:8034`) while alive;
        // killing the far bench cell re-frames the pot inside `Check2x1`'s
        // kill loop (:47108-47115), where the ungated `CheckOnTable1x1`
        // (:43503-43546) kills it — mid-loop, with the flag still in flight.
        let mut w = object_world();
        // Work bench 2x1 at (10..11, 20); the pot on its left cell.
        set_object_cell(&mut w, 10, 20, 18, 0, 0);
        set_object_cell(&mut w, 11, 20, 18, 18, 0);
        set_object_cell(&mut w, 10, 19, 78, 0, 0);
        // Mine the bench cell that is not the pot's support: the first
        // framing pass anchors the pot (the table arm, :43530-43532), the
        // kill loop's re-frame kills it.
        let kill = kill_tile(&mut w, 11, 20, false).unwrap();
        assert_eq!(kill.kind, 18);
        assert_all_gone(&w, &[(10, 20), (11, 20), (10, 19)]);
        assert_eq!(
            kill.object_drops,
            vec![
                ObjectDrop {
                    tile_kind: 18,
                    x: 10,
                    y: 20,
                    item: None,
                },
                ObjectDrop {
                    tile_kind: 78,
                    x: 10,
                    y: 19,
                    item: None,
                },
            ]
        );
    }

    // ---- Phase 3 (M1.4-TO-4): the herb/pile loot event channels -------------

    #[test]
    fn test_DW_3_2_herb_event_records_the_killed_cell() {
        // `CheckAlch`'s kill records the per-cell herb drop as an event —
        // the kind, the `frameX / 18` style and the cell (`KillTile_DropItems`
        // rolls the app resolves, `WorldGen.cs:65674-65698`).
        let mut w = object_world();
        // Soil swap: dirt → sand (the style-2 Blinkroot set {0, 59}) is not
        // needed — style 0 also accepts 0 via {2,477,109,492}? no: dirt 2 is
        // the style-0 set, so the bare fixture stands.
        set_object_cell(&mut w, 10, 19, 83, 2 * 18, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_all_gone(&w, &[(10, 19)]);
        assert_eq!(
            kill.herb_events,
            vec![HerbEvent {
                kind: 83,
                style: 2,
                x: 10,
                y: 19
            }]
        );
        assert!(kill.object_drops.is_empty(), "no per-cell record");
    }

    #[test]
    fn test_DW_3_2_herb_event_gated_by_no_item() {
        // The herb event stands for a `KillTile_DropItems` roll — the C#
        // `noItem` gate suppresses it like the records (:63860-63864).
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 84, 0, 0);
        let kill = kill_tile(&mut w, 10, 20, true).unwrap();
        assert_all_gone(&w, &[(10, 19)]);
        assert!(kill.herb_events.is_empty());
    }

    #[test]
    fn test_DW_3_3_pile_event_records_the_loot_block() {
        // A broken 2x1 coin pile (185, frameX 576 — the copper sub-style)
        // records the loot block with the origin's frameX, read before the
        // kills and after the grass-cover conversion (:47127-47131).
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 185, 576, 18);
        set_object_cell(&mut w, 11, 19, 185, 594, 18);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_all_gone(&w, &[(10, 19), (11, 19)]);
        assert_eq!(
            kill.pile_events,
            vec![PileLootEvent {
                kind: 185,
                frame_x: 576,
                x: 10,
                y: 19
            }]
        );
    }

    #[test]
    fn test_DW_3_3_bench_2x1_carries_no_pile_event() {
        // The loot block keys on `type == 185` (:47130) — the bench's 2x1
        // destruction records only its item drop.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.object_drops.len(), 1);
        assert!(kill.pile_events.is_empty());
    }

    #[test]
    fn the_orb_break_sets_the_world_flag_and_count() {
        // `WorldGen.CheckOrb` (`:54169-54344`): the 2x2 integrity scan,
        // the smash flag + the shared counter. The third break rides the
        // app's summon edge.
        use crate::worldgen::evil::{TILE_SHADOW_ORB, add_shadow_orb};
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 0..200 {
                let i = w.index(x, y);
                w.tiles[i] = crate::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        // The frame-important bits ride the generated world's pipeline
        // init () — the direct-built test world sets
        // them by hand.
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[31] = true;
        w.frame_important[12] = true;
        w.frame_important[639] = true;
        add_shadow_orb(&mut w, 100, 100, false);
        // The first hit on any member breaks the whole 2x2 — the C#'s
        // `CheckOrb` integrity scan (`:54200-54213`) re-kills the rest
        // and raises the flags at once (`:54292-54293`).
        let kill = super::kill_tile(&mut w, 99, 99, true).expect("the break");
        let _ = kill;
        assert!(w.shadow_orb_smashed, "the smash flag");
        assert_eq!(w.shadow_orb_count, 1, "the shared counter");
        // A second hit on the already-broken object does not re-flag.
        let _ = super::kill_tile(&mut w, 100, 100, true);
        assert_eq!(
            w.shadow_orb_count, 1,
            "the broken object holds its single count"
        );
        // The tiles are gone.
        assert!(!w.tile(100, 100).unwrap().active());
    }

    #[test]
    fn the_first_smash_forced_gun_and_the_roll_table() {
        // `:54240`/`:54268`'s forced case 0: the FIRST smash in the world
        // always drops the Musket (96); after the flag sets the 5-way
        // roll takes over (the table's ids 96/64/162/115/111).
        use crate::worldgen::evil::add_shadow_orb;
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 0..200 {
                let i = w.index(x, y);
                w.tiles[i] = crate::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[31] = true;
        w.frame_important[12] = true;
        w.frame_important[639] = true;
        add_shadow_orb(&mut w, 100, 100, false);
        // no_item = false — the drop flows (the frame walk pushes the
        // plan's drop only when the drops are not suppressed).
        let kill = super::kill_tile(&mut w, 99, 99, false).expect("the break");
        let drop = kill.object_drops.first().copied().expect("the drop");
        assert_eq!(drop.item, Some(96), "the forced gun on the first smash");
        // A second orb (the flag now set) rolls the table.
        add_shadow_orb(&mut w, 140, 100, false);
        let kill2 = super::kill_tile(&mut w, 139, 99, false).expect("the second break");
        let drop2 = kill2
            .object_drops
            .first()
            .copied()
            .expect("the rolled drop");
        assert!(
            matches!(
                drop2.item,
                Some(96) | Some(64) | Some(162) | Some(115) | Some(111)
            ),
            "the 5-way table item: {:?}",
            drop2.item
        );
        assert_eq!(w.shadow_orb_count, 2, "both breaks counted");
    }
}
