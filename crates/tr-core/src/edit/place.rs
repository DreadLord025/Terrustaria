//! The tile-placement family (`WorldGen.PlaceTile` and the item-use route).
//!
//! S1-K-4: extracted verbatim from `edit.rs` — behaviour frozen. The shared
//! edit helpers (`touch`, `after_edit`, `liquid_flags`, `ensure_frame_important`,
//! `ALCH_SOILS`) stay in the parent module; a child module sees its parent's
//! private items, so no seam changed.

use super::{
    after_edit, ensure_frame_important, liquid_flags, touch, ALCH_SOILS, WALL_UNBREAKABLE,
};
use crate::framing;
use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

/// Places a block/torch into an empty cell, preserving the wall. The caller checks
/// reach, support and player overlap.
pub fn place_tile(world: &mut World, x: i32, y: i32, kind: u16) -> bool {
    // S1-P-i42 — the torch family rides its own arm (`WorldGen.PlaceTile`
    // case 4, style 0): the support gate + the liquid gate + `frameY =
    // 22 * style` + `CheckTorch`, exactly like the item-use path
    // (`place_item_tile`). The pre-fix legacy write placed a torch with NO
    // support gate and frameY 0; the dev `TR_PLACE` hook and the liquid-merge
    // caller now judge a torch the same way vanilla does.
    if kind == 4 {
        return place_torch(world, x, y, 0);
    }
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    if t.active() || t.wall == WALL_UNBREAKABLE {
        return false;
    }
    let i = world.index(x, y);
    let mut tile = Tile {
        kind,
        wall: t.wall,
        liquid: t.liquid,
        frame_x: -1,
        frame_y: -1,
        flags: liquid_flags(t.flags),
        ..Default::default()
    };
    tile.flags |= crate::tile::FLAG_ACTIVE;
    world.tiles[i] = tile;
    touch(world, x, y);
    // `WorldGen.TileFrame` runs `CheckTorch` for torch tiles: the frame follows the
    // attachment, and an unsupported torch is removed again (the caller then keeps the
    // item instead of consuming it). (The torch family routes above; this stays
    // for any other `SET_TORCHES` kind the table carries.)
    let kept = !td::SET_TORCHES.has(kind) || crate::torch::check_torch(world, x, y);
    after_edit(world, x, y);
    kept
}

/// The FORCED-placement route — `WorldGen.PlaceTile(i, j, Type, mute,
/// forced: true, plr: -1, style: 0)` (`WorldGen.cs:59342-60295`) for the
/// falling-block family's death placement (`Projectile.cs:77545`, W4-d1
/// retire — the forced flag's exact bypass set, censused whole).
///
/// The forced flag is the FIRST disjunct of the `:59373` gate, so it
/// bypasses `Collision.EmptyTile` (`Collision.cs:1359-1381` — the active
/// occupant AND the player/NPC rectangle overlap scan) and the port's
/// wall-unbreakable refusal (the `place_tile` head's `CanKillTile`-derived
/// gate — the C# `PlaceTile` carries no such refusal at all). The
/// `:59375-59430` grass-on-dirt/moss rewrites never match a served kind
/// (53/112/116/123/224/234/59/57/330-333/495/0 — no 2/23/60/70/109/199/
/// 633/661/662, no `Main.tileMoss` member); the `:59450` drip gate and the
/// `:59454-59472` liquid refusals never match either (no torch, no
/// 3/20/24/27/32/51/69/72/201/352/529/624/637/656/697/700/701). The cell
/// gate is the caller's (`Kill_ExplodeTiles`'s `!Main.tile[..].active()`
/// `:77516` — the forced route itself does NOT re-check occupancy). The
/// port serves the free-cell shape (the only caller today); an occupied
/// cell would keep its wall/liquid but reset paint/frames ahead of the
/// re-frame — the C# keeps the occupant's paint (no Clear on the active
/// branch, and the `:59477` `ResetsHalfBrickPlacementAttempt` reset never
/// matches a served kind), a recorded micro-deviation for a path no
/// caller reaches.
///
/// The write is the legacy default arm (`:60234-60250`: `active = true`,
/// `type = num`): the `:59473-59476` `Clear(Tile | TilePaint | Slope)` on
/// the free cell (the stale frames/paint/slope wipe — the liquid and wall
/// survive, the `place_tile` precedent), then the type write with the
/// frames left for the tail's `SquareTileFrame` (`:60261`, the
/// `after_edit` re-frame — the `TruncatesWalls` `SquareWallFrame`
/// `:60257-60260` never fires for a served kind). The 53/396/397 paint
/// arms (`:60237-60247`) ride the world's seed flags: the rainbow paint
/// needs `SpecialSeedFeatures.RainbowSandAndBlackSandWalls` (the shimmer
/// ocean mode the port does not generate — folds off, the `convert_sized`
/// precedent); the anniversary `color(7)` needs `tenthAnniversaryWorld &&
/// !remixWorld && !notTheBeesWorld && !dontStarveWorld` — ported verbatim
/// over `world.seeds`. The `:60263-60290` sound/dust tail is the caller's
/// (the falling arm queues the `place_tile_sound` cue on the pool's sound
/// intent — the `mute: false` arm; the 22/140 dust never matches).
pub fn place_tile_forced(world: &mut World, x: i32, y: i32, kind: u16) -> bool {
    // `:59349-59352` — `num >= TileID.Count` refuses; `:59354` the bounds.
    if kind as usize >= crate::tiledata::TILE_COUNT || !world.in_bounds(x, y) {
        return false;
    }
    // `:59345-59348` — the gen-time 488 refusal (`isGeneratingOrLoadingWorld`)
    // never fires at runtime (the flag folds false).
    let i = world.index(x, y);
    let prev = world.tiles[i];
    // `:59473-59476` — the free cell's Clear: wall + liquid survive.
    let mut tile = Tile {
        kind,
        wall: prev.wall,
        liquid: prev.liquid,
        frame_x: -1,
        frame_y: -1,
        flags: liquid_flags(prev.flags),
        ..Default::default()
    };
    tile.flags |= crate::tile::FLAG_ACTIVE;
    // `:60237-60247` — the sand paint arms (53 is the only served member).
    if kind == 53
        && world.seeds.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY)
        && !world.seeds.intersects(
            crate::seeds::SeedFlags::REMIX
                | crate::seeds::SeedFlags::NOT_THE_BEES
                | crate::seeds::SeedFlags::DONT_STARVE,
        )
    {
        tile.set_paint(7);
    }
    world.tiles[i] = tile;
    touch(world, x, y);
    // `:60255-60262` — the tile is active: the SquareTileFrame re-frame
    // (the served kinds are not frame-important — the generic block chain
    // assigns the merge frames). `result = true` (`:60262`).
    after_edit(world, x, y);
    true
}

/// The item-use placement route (`Player.PlaceThing_Tiles_TryPlacing`,
/// `Player.cs:39746-39788`, entering `PlaceThing_Tiles_PlaceIt` :40108): a tile
/// with registered `TileObjectData` (`TileObjectData.CustomPlace`,
/// `TileObjectData.cs:5016`) is placed through `TileObject.CanPlace`/`Place`;
/// tile 82 is registered but explicitly excluded from that route (:39772) and
/// the rest keeps the legacy `WorldGen.PlaceTile` paths (`WorldGen.cs:59342`):
/// the `Main.tileAlch` kinds through `PlaceAlch` (:60177-60181), tile 380
/// through its own case (:60226-60230), everything else through the legacy
/// single-tile write. `style` is the item's `placeStyle`, `dir` the player
/// direction (`Player.direction`: -1 left, 1 right). `false` = refused, the
/// world is unchanged.
pub fn place_item_tile(world: &mut World, x: i32, y: i32, kind: u16, style: i32, dir: i32) -> bool {
    // Registry route (`Player.cs:39771-39772`): `TileObjectData.CustomPlace
    // (tileToCreate, placeStyle) && tileToCreate != 82 && tileToCreate != 227
    // && tileToCreate != 4` sends the kind through `TileObject.CanPlace`/
    // `Place`. S1-F-7: the gate is the C# `CustomPlace` flag (the style-resolved
    // `UsesCustomCanPlace`), NOT mere registration — the full registry registers
    // the natural objects too (pots 28, piles 185/186/187, plants, platforms 19,
    // torches 4, dye plants 227), and those keep the legacy `WorldGen.PlaceTile`
    // route exactly as vanilla (`CustomPlace` false, or the 82/227/4 exclusions).
    if crate::tileobject::routes_through_tile_object(kind, style) {
        if let Some(def) = crate::tileobject::def_for_tile(kind) {
            // S1-F-7b: `Main.rand` = the world's placement stream
            // (`World::place_rng`), `randomCache` = the world's stickiness
            // cache (`TileObject.cs:787`/:819, :764-792). The stream is taken
            // out for the read-only `can_place` call and restored after.
            let mut rand = std::mem::take(&mut world.place_rng);
            let instance = crate::tileobject::can_place(
                world,
                x,
                y,
                def,
                style,
                dir,
                None,
                &mut rand,
                world.tile_object_random_cache,
            );
            world.place_rng = rand;
            let Some(instance) = instance else {
                return false;
            };
            if !crate::tileobject::place(world, &instance) {
                return false;
            }
            // `WorldGen.SquareTileFrame(tileTargetX, tileTargetY)`
            // (`Player.cs:40121`); `reframe_area` leaves `frame_important` tiles
            // intact, so the object's stored frames survive (DW-1.6 guarantee).
            framing::reframe_area(world, x, y, 2, true);
            return true;
        }
    }
    // Legacy `WorldGen.PlaceTile` dispatcher (`WorldGen.cs:60177-60230`):
    // `Main.tileAlch` kinds go to `PlaceAlch`, tile 380 has its own case,
    // tile 4 (the torch family) its own `else if (num == 4)` arm
    // (`WorldGen.cs:59798-59842`) — the style ride and the support gate the
    // single-tile write never carried (S1-P-i42).
    if crate::tileobject::tile_alch(kind) {
        return place_alch(world, x, y, style);
    }
    if kind == 380 {
        return place_planter_box(world, x, y, style);
    }
    if kind == 19 {
        return place_platform(world, x, y, style);
    }
    if kind == 4 {
        return place_torch(world, x, y, style);
    }
    // Legacy single-tile route (`WorldGen.PlaceTile`).
    place_tile(world, x, y, kind)
}

/// `WorldGen.PlaceTile`'s torch arm (`WorldGen.cs:59798-59842`) plus the
/// liquid gate ahead of the write (`:59454-59467`): the torch family writes
/// `frameY = 22 * style` (`:59839`) — the style column `CheckTorch` preserves
/// and `TileLightScanner` reads (`tile_light/scanner.rs`'s `fy / 22` arm) —
/// only when the cell has a wall behind it OR an attachable support below /
/// left / right (`:59835`). A refused placement writes nothing and keeps the
/// item (the caller's consumption gate). `style` is the item's `placeStyle`
/// (0 plain … 23 shimmer, the 24 `ItemID.Sets.Torches` rows).
///
/// The liquid gate (`:59454-59467`): a cell with liquid refuses every torch
/// style EXCEPT 8 (Cursed) / 11 (Ichor) / 17 (Coral) — the three
/// `ItemID.Sets.WaterTorches` (`ItemID.cs:1372`); those three resolve their
/// `TileObjectData` sub-tile (`D_4_S8/S11/S17`, all `WaterPlacement.Allowed`),
/// so the `WaterPlacement != Allowed` refusal never fires for them. The C#
/// also reads `tile.checkingLiquid()` — the port's tile flags carry no
/// checking bit (`tile.rs`), so that half folds to false (ANALYSIS).
pub fn place_torch(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let Some(prev) = world.tile(x, y).copied() else {
        return false;
    };
    // `:59373` — the write needs an empty cell (`Collision.EmptyTile`); the
    // unbreakable-wall refusal is the port's shared placement gate
    // (`place_tile`'s head).
    if prev.active() || prev.wall == WALL_UNBREAKABLE {
        return false;
    }
    // `:59454-59467` — the liquid gate (module doc). `TileID.Sets.Torches[4]`
    // is true, so the arm always judges a torch cell with liquid.
    if prev.liquid > 0 {
        // `:59458` — `num == 4 && style != 8 && style != 11 && style != 17`.
        if !matches!(style, 8 | 11 | 17) {
            return false;
        }
        // `:59462-59466` — the water-torch sub-tiles' `WaterPlacement` is
        // `Allowed` (registry `D_4_S8`/`D_4_S11`/`D_4_S17`), so the refusal
        // never fires; an unregistered style would refuse (`GetTileData`
        // null skips the check in C# — the styles 8/11/17 are the only ones
        // past the gate above, and all three are registered).
        let allowed = crate::tileobject::def_for_tile(4)
            .and_then(|def| crate::tileobject::get_tile_data(def, style, 0))
            .is_none_or(|sub| {
                sub.water_placement == crate::tileobject::LiquidPlacement::Allowed
            });
        if !allowed {
            return false;
        }
    }
    // `:59835` — the support gate: wall behind, OR the left neighbour
    // (active, slope 0 or even, solid-non-top-non-NotReallySolid / beam /
    // tree-trunk triple), OR the right neighbour (same, odd slopes), OR the
    // below neighbour (active, solid, platform-with-top-edge OR
    // non-solid-top flat). The neighbour reads are RAW `active()` (no
    // `nactive()` — actuated tiles still hold a torch at placement time;
    // `CheckTorch`'s later re-frame may drop it, exactly like the C#).
    // The tree triples read the diagonals RAW too (`tile7.type` /
    // `tile5.type` — no active gate, `:59835`).
    let left_ok = world.tile(x - 1, y).is_some_and(|t| {
        t.active()
            && (t.slope() == 0 || t.slope() % 2 != 1)
            && (side_support(t)
                || (crate::torch::is_tree_type(t.kind as i32)
                    && crate::torch::is_tree_type(
                        world.tile(x - 1, y - 1).map_or(0, |d| d.kind as i32),
                    )
                    && crate::torch::is_tree_type(
                        world.tile(x - 1, y + 1).map_or(0, |d| d.kind as i32),
                    )))
    });
    let right_ok = world.tile(x + 1, y).is_some_and(|t| {
        t.active()
            && (t.slope() == 0 || t.slope() % 2 != 0)
            && (side_support(t)
                || (crate::torch::is_tree_type(t.kind as i32)
                    && crate::torch::is_tree_type(
                        world.tile(x + 1, y - 1).map_or(0, |d| d.kind as i32),
                    )
                    && crate::torch::is_tree_type(
                        world.tile(x + 1, y + 1).map_or(0, |d| d.kind as i32),
                    )))
    });
    let below_ok = world.tile(x, y + 1).is_some_and(|t| {
        t.active()
            && is_solid_kind(t.kind)
            && ((crate::tiledata::SET_PLATFORMS.has(t.kind)
                && crate::torch::top_edge_can_be_attached_to(world, x, y + 1))
                || ((!crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                    || (t.kind == 380 && t.slope() == 0))
                    && !crate::tileobject::not_really_solid(t.kind)
                    && !t.half_brick()
                    && t.slope() == 0))
    });
    if !(prev.wall > 0 || left_ok || right_ok || below_ok) {
        return false;
    }
    // `:59473-59476` — `tile.Clear(Tile | TilePaint | Slope)` on the empty
    // cell: the stale frames/paint/slope of a broken occupant are wiped
    // before the write (the liquid survives — `TileDataType.Tile` excludes
    // it, the `place_tile` precedent).
    // `:59837-59839` — the write: active, type 4, `frameY = 22 * style`.
    // `frameX` is left for `SquareTileFrame` → `CheckTorch` (`:59840`), which
    // the `after_edit` re-frame below runs (the attachment frame 0/22/44).
    let i = world.index(x, y);
    let cell = &mut world.tiles[i];
    cell.flags |= crate::tile::FLAG_ACTIVE;
    cell.flags &= !(crate::tile::FLAG_HALF_BRICK | crate::tile::FLAG_FULLBRIGHT);
    cell.set_slope(0);
    cell.set_paint(0);
    cell.kind = 4;
    cell.frame_x = 0;
    cell.frame_y = (22 * style) as i16;
    touch(world, x, y);
    ensure_frame_important(world, 4);
    // `:59840`/`:60261` — `SquareTileFrame`: the attachment re-frame. An
    // unsupported torch (the gate above can pass on a wall the `CheckTorch`
    // 3x3 scan then judges differently — e.g. the edge-of-world early return)
    // is removed again and the caller keeps the item.
    let kept = crate::torch::check_torch(world, x, y);
    after_edit(world, x, y);
    kept
}

/// `WorldGen.PlaceTile` case 4's side-support conjunction (`:59835`):
/// `(tileSolid && !tileSolidTop && !NotReallySolid) || IsBeam` — the tree
/// triple is judged inline by [`place_torch`] (it needs the diagonal
/// neighbours).
fn side_support(t: &crate::tile::Tile) -> bool {
    (is_solid_kind(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !crate::tileobject::not_really_solid(t.kind))
        || crate::tiledata::SET_IS_BEAM.has(t.kind)
}

fn is_solid_kind(kind: u16) -> bool {
    crate::tiledata::TILE_SOLID.has(kind)
}

/// `WorldGen.PlaceAlch` (`WorldGen.cs:45380-45479`): the seed write — plants
/// tile 82 (`frameX = 18 * style, frameY = 0`) on the style's soil, judged
/// 1:1 with the C# soil/liquid rules. `style` is the seed's `placeStyle`
/// (0 Daybloom … 6 Shiverthorn). `false` = refused, the world is unchanged.
pub fn place_alch(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    // :45383-45388 — the C# null-creates both cells; the port requires them
    // in-world (an out-of-world write would be lost anyway).
    let Some(herb) = world.tile(x, y).copied() else {
        return false;
    };
    let Some(soil) = world.tile(x, y + 1).copied() else {
        return false;
    };
    // :45390-45392 — the herb cell must be empty and the soil below must be
    // `nactive()` — active and not actuated (`Tile.nactive`, `Tile.cs:240-246`;
    // wiring actuation keeps `FLAG_ACTIVE` and sets `FLAG_INACTIVE`) — and flat
    // (`!halfBrick() && slope() == 0`).
    if herb.active() || !crate::tileobject::nactive(&soil) || soil.half_brick() || soil.slope() != 0
    {
        return false;
    }
    // :45393-45441 — the per-style soil sets (every style also accepts the
    // clay pot 78 and the planter box 380); shared with `check_alch`.
    let soil_rejected = match style {
        s @ 0..=6 => !ALCH_SOILS[s as usize].contains(&soil.kind),
        // No case matches: the C# switch falls through with the reject flag
        // still false — an out-of-range style plants with no checks at all.
        _ => false,
    };
    // :45442-45455 — the liquid rules: styles 0-3 refuse any liquid, 4/6
    // refuse lava only, 5 (the ash style) refuses everything but lava.
    let herb_lava = herb.liquid_kind() == crate::tile::LiquidKind::Lava;
    let liquid_rejected = match style {
        0..=3 => herb.liquid > 0,
        4 | 6 => herb.liquid > 0 && herb_lava,
        5 => herb.liquid > 0 && !herb_lava,
        _ => false,
    };
    if soil_rejected || liquid_rejected {
        return false;
    }
    // :45458-45477 — the write: `active, type = 82, frameX = 18 * style,
    // frameY = 0` plus `CopyPaintAndCoating(tile2)` (`Tile.cs:867-873` —
    // color and fullbright; the port has no invisibleBlock state, which only
    // special-world tiles carry). Wall and liquid of the cell stay.
    let i = world.index(x, y);
    let cell = &mut world.tiles[i];
    cell.flags |= crate::tile::FLAG_ACTIVE;
    cell.kind = 82;
    cell.frame_x = (18 * style) as i16;
    cell.frame_y = 0;
    cell.set_paint(soil.paint());
    cell.flags =
        (cell.flags & !crate::tile::FLAG_FULLBRIGHT) | (soil.flags & crate::tile::FLAG_FULLBRIGHT);
    touch(world, x, y);
    // `Main.tileFrameImportant[82..84]` are boot statics (`Main.cs:7685-7687`);
    // mirror all three so the re-frame below keeps the fresh herb and any
    // sibling-stage herbs in the 5x5 (their frames are hand-written).
    for alch_kind in crate::tileobject::TILE_ALCH {
        ensure_frame_important(world, *alch_kind);
    }
    after_edit(world, x, y);
    true
}

/// `WorldGen.PlaceTile` case 380 (`WorldGen.cs:60226-60230`): the planter-box
/// legacy write — `frameY = 18 * style`, active, type 380. The C# case
/// touches no other field, so `frameX` stays exactly as the cell had it (a
/// virgin cell's frames are 0 — `Tile.cs:92-104`). Tile 380 is NOT
/// registered in `TileObjectData` (vanilla routes it through this legacy
/// branch), so it never joins [`crate::tileobject::TABLE`].
fn place_planter_box(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let Some(prev) = world.tile(x, y).copied() else {
        return false;
    };
    if prev.active() || prev.wall == WALL_UNBREAKABLE {
        return false;
    }
    let i = world.index(x, y);
    let cell = &mut world.tiles[i];
    cell.flags |= crate::tile::FLAG_ACTIVE;
    cell.kind = 380;
    cell.frame_y = (18 * style) as i16;
    touch(world, x, y);
    ensure_frame_important(world, 380); // `Main.cs:7632`
    after_edit(world, x, y);
    true
}

/// `WorldGen.PlaceTile` case 19 (`WorldGen.cs:60221-60225`): the platform
/// legacy write — `frameY = 18 * style`, active, type 19. Like the planter
/// box, the C# case touches no other field, so `frameX` stays as the cell had
/// it. Tile 19 is registered in `TileObjectData` but `CustomPlace`-false
/// (`registry.rs` D_19), so vanilla routes it through this legacy branch, NOT
/// `TileObject.Place`. `Main.tileFrameImportant[19]` is a boot static
/// (`Main.cs:7654`): mirroring it keeps the style's `frameY` through the
/// re-frame (the generic block chain would otherwise clobber it with a
/// merge-frame — the pre-fix defect, the platform drew as a plain block).
fn place_platform(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let Some(prev) = world.tile(x, y).copied() else {
        return false;
    };
    if prev.active() || prev.wall == WALL_UNBREAKABLE {
        return false;
    }
    let i = world.index(x, y);
    let cell = &mut world.tiles[i];
    cell.flags |= crate::tile::FLAG_ACTIVE;
    cell.kind = 19;
    cell.frame_y = (18 * style) as i16;
    touch(world, x, y);
    ensure_frame_important(world, 19); // `Main.cs:7654`
    after_edit(world, x, y);
    true
}

/// `WorldGen.PlaceChest(x, y, type, notNearOtherChests, style)`'s tile-write
/// half (`WorldGen.cs:57813-57845`): the boulder-support refusal (`:57816-57819`),
/// then `TileObject.CanPlace`/`Place` (`:57820-57829`). Returns the placed
/// object's TOP-LEFT coords — the `Chest.CreateChest(objectData.xCoord,
/// objectData.yCoord)` inputs (`:57830`) — or `None` for the C# `-1` refusal.
/// The `notNearOtherChests` flag is `false` at every `34` host call site
/// (`MessageBuffer.cs:2003/:2045/:2080`), so the `NearOtherChests` gate
/// (`:57823-57826`) never fires there; the chest-store creation and the net
/// sends are app-scope (the `34 ChestUpdates` host arm, W4-g52).
pub fn place_chest(world: &mut World, x: i32, y: i32, kind: u16, style: i32) -> Option<(i32, i32)> {
    // `:57816-57819` — a boulder under either support column refuses. The C#
    // reads `Main.tile[…].type` directly (no `active` test); an inactive cell's
    // type is 0, which is not a boulder, so the port's kind read matches.
    let boulder_below = |w: &World, cx: i32| {
        w.tile(cx, y + 1)
            .is_some_and(|t| td::SET_BOULDERS.has(t.kind))
    };
    if boulder_below(world, x) || boulder_below(world, x + 1) {
        return None;
    }
    let def = crate::tileobject::def_for_tile(kind)?;
    // `TileObject.CanPlace(x, y, type, style, 1, out objectData)` (`:57820`) —
    // dir 1, the world's placement stream + stickiness cache (the
    // `place_item_tile` registry-route precedent, S1-F-7b).
    let mut rand = std::mem::take(&mut world.place_rng);
    let instance = crate::tileobject::can_place(
        world,
        x,
        y,
        def,
        style,
        1,
        None,
        &mut rand,
        world.tile_object_random_cache,
    );
    world.place_rng = rand;
    let instance = instance?;
    if !crate::tileobject::place(world, &instance) {
        return None;
    }
    Some((instance.x_coord, instance.y_coord))
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)] // DW-IDs are part of the test names (traceability)
    use super::super::kill_tile;
    use super::super::tests::{
        doorway_world, ground_world, object_world, set_active, set_liquid, soil_world,
        ALCH_SOIL_TABLE,
    };
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// W4-g52 — `WorldGen.PlaceChest`'s tile-write half (`WorldGen.cs:
    /// 57813-57845`): the boulder-support refusal and the top-left return the
    /// `Chest.CreateChest` call reads.
    #[test]
    fn w4_g52_place_chest_returns_the_top_left_and_refuses_boulders() {
        let mut w = object_world();
        // Target (14,19), origin (0,1) → top-left (14,18), support row 20.
        assert_eq!(place_chest(&mut w, 14, 19, 21, 0), Some((14, 18)));
        assert!(w.tile(14, 18).is_some_and(|t| t.active() && t.kind == 21));
        assert!(w.tile(15, 19).is_some_and(|t| t.active() && t.kind == 21));
        // A boulder under either support column refuses (`:57816-57819`).
        let bi = w.index(20, 20);
        w.tiles[bi].kind = 138; // `TileID.Sets.Boulders`
        assert_eq!(place_chest(&mut w, 20, 19, 21, 0), None);
        assert!(!w.tile(20, 18).is_some_and(|t| t.active() && t.kind == 21));
    }

    #[test]
    fn break_then_place_round_trips_a_tile() {
        let mut w = ground_world();
        assert_eq!(kill_tile(&mut w, 10, 20, true).map(|k| k.kind), Some(1));
        assert!(!w.tile(10, 20).unwrap().active());
        assert_eq!(w.surface_of(10), 21);
        assert!(place_tile(&mut w, 10, 20, 0));
        let t = *w.tile(10, 20).unwrap();
        assert!(t.active() && t.kind == 0);
        // Local framing assigned a real frame from the generic table.
        assert!(t.frame_x >= 0 && t.frame_y >= 0);
        assert_eq!(w.surface_of(10), 20);
        assert!(w.revision >= 2);
    }

    #[test]
    fn torch_keeps_its_explicit_frame() {
        let mut w = ground_world();
        assert!(place_tile(&mut w, 5, 19, 4));
        let t = *w.tile(5, 19).unwrap();
        assert!(t.active());
        assert_eq!((t.frame_x, t.frame_y), (0, 0));
    }

    #[test]
    fn placing_into_an_active_cell_fails() {
        let mut w = ground_world();
        assert!(!place_tile(&mut w, 5, 25, 0));
        assert!(!place_tile(&mut w, 999, 25, 0));
    }

    #[test]
    fn test_DW_2_1_object_kinds_route_through_the_registry() {
        // The work bench (tile 18, registry def 2x1) fills both cells through
        // the TileObject route; the legacy single-tile write would leave the
        // second cell inactive.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        for x in [10, 11] {
            let t = w.tile(x, 19).unwrap();
            assert!(t.active() && t.kind == 18, "bench cell ({x}, 19)");
        }
    }

    #[test]
    fn test_DW_2_1_non_registry_kinds_keep_the_legacy_path() {
        // Dirt (tile 0) has no registry entry: the legacy single-tile write
        // runs, generic framing assigned.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 5, 19, 0, 0, 1));
        let t = *w.tile(5, 19).unwrap();
        assert!(t.active() && t.kind == 0);
        assert!(t.frame_x >= 0 && t.frame_y >= 0);
        // The torch keeps its explicit frame through the same fallback
        // (`WorldGen.PlaceTile` case 4 lives in the legacy path).
        assert!(place_item_tile(&mut w, 7, 19, 4, 0, 1));
        let torch = w.tile(7, 19).unwrap();
        assert_eq!((torch.frame_x, torch.frame_y), (0, 0));
    }

    #[test]
    fn test_DW_2_4_route_place_frames_and_frame_important() {
        // Bench frames: `coordinate_full_width` 18, style 0
        // (`TileObjectData.cs:2818-2825`) — columns (0, 0) and (18, 0).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 0);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 18);
        assert_eq!(w.tile(10, 19).unwrap().frame_y, 0);
        // `place` marks the whole object `frame_important` (Phase 1 DW-1.3
        // guarantee, exercised through the route).
        assert!(w.frame_important[18]);

        // Chest (tile 21): 2x2, origin (0,1) — targeting (14, 19) fills
        // rows 18..=19 with frames x 0/18, y 0/18.
        assert!(place_item_tile(&mut w, 14, 19, 21, 0, 1));
        assert_eq!(w.tile(14, 18).unwrap().frame_x, 0);
        assert_eq!(w.tile(15, 18).unwrap().frame_x, 18);
        assert_eq!(w.tile(14, 19).unwrap().frame_y, 18);
        assert!(w.tile(15, 19).unwrap().active() && w.tile(15, 19).unwrap().kind == 21);
    }

    #[test]
    fn the_furniture_family_places_on_their_anchors() {
        // The `TileObjectData` verbatim data (see `tileobject.rs`): each kind
        // fills its footprint with the style-0 frames.
        // Piano 87 (3x2, origin (1,1)) at (10, 19): rows 18-19, x 9-11.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 87, 0, 1));
        for x in 9..=11 {
            for y in 18..=19 {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 87, "piano {x},{y}");
            }
        }
        // Dresser 88 (3x2), sink 172 (2x2, origin (0,1)), candelabra 100
        // (2x2), bookcase 101 (3x4, origin (1,3)), lamp 93 (1x3, origin
        // (0,2)), clock 104 (2x5, origin (0,4)) — the footprints fill.
        let cases: &[(u16, i32, i32, usize, usize)] = &[
            (88, 10, 19, 3, 2),
            (172, 10, 19, 2, 2),
            (100, 10, 19, 2, 2),
            (101, 10, 19, 3, 4),
            (93, 10, 19, 1, 3),
            (104, 10, 19, 2, 5),
        ];
        for &(kind, tx, ty, w_, h) in cases {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, tx, ty, kind, 0, 1), "place {kind}");
            let def = crate::tileobject::def_for_tile(kind).unwrap();
            let ox = tx - def.origin.x;
            let oy = ty - def.origin.y;
            for x in ox..ox + w_ as i32 {
                for y in oy..oy + h as i32 {
                    let t = w.tile(x, y).unwrap();
                    assert!(t.active() && t.kind == kind, "kind {kind} cell {x},{y}");
                }
            }
        }
    }

    #[test]
    fn the_bed_places_with_the_facing_alternate() {
        // `Style4x2` (`TileObjectData.cs:2555-2568`): facing left = the base
        // (frames 0/18/36/54), facing right = the `PlaceRight` alternate —
        // the second style column continues at frameX 72 (`Check4x2`'s
        // `frameX >= 72` branch, `WorldGen.cs:47416-47420`).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 11, 19, 79, 0, -1));
        // The origin (1, 1): the cells span x 10-13, rows 18-19; the frames
        // are the column strides 0/18/36/54.
        assert_eq!(w.tile(10, 18).unwrap().frame_x, 0);
        assert_eq!(w.tile(13, 19).unwrap().frame_x, 54);
        let mut w2 = object_world();
        assert!(place_item_tile(&mut w2, 11, 19, 79, 0, 1));
        // The `PlaceRight` alternate = style 1: the second style column
        // starts at frameX `CoordinateFullWidth` = 72 (`Check4x2`'s
        // `frameX >= 72` branch, `WorldGen.cs:47416-47420`).
        assert_eq!(w2.tile(10, 18).unwrap().frame_x, 72);
        assert_eq!(w2.tile(13, 19).unwrap().frame_x, 72 + 54);
        // The bottom row is 18 tall (`coordinate_heights [16, 18]`).
        assert_eq!(w2.tile(10, 19).unwrap().frame_y, 18);
    }

    #[test]
    fn the_candle_places_on_a_table_only() {
        // `StyleOnTable1x1` (`TileObjectData.cs:2325-2334`): the anchor is
        // `Table` — a bare floor refuses, a placed table accepts.
        let mut w = object_world();
        assert!(!place_item_tile(&mut w, 10, 19, 33, 0, 1));
        assert!(place_item_tile(&mut w, 10, 19, 14, 0, 1)); // the table
        assert!(place_item_tile(&mut w, 11, 17, 33, 0, 1)); // on its top
        assert!(w.tile(11, 17).unwrap().active() && w.tile(11, 17).unwrap().kind == 33);
    }

    #[test]
    fn the_chandelier_hangs_from_the_ceiling() {
        // `TileObjectData.cs:3139-3160`: `AnchorTop = (SolidTile | SolidSide,
        // 1, 1)` + `AnchorBottom Empty` — the doorway world's stone ceiling
        // at y = 16 carries the middle column.
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 11, 17, 34, 0, 1));
        for x in 10..=12 {
            for y in 17..=19 {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 34, "chandelier {x},{y}");
            }
        }
        // The lantern (42, `Style1x2Top`) hangs the same way, 1x2.
        let mut w2 = doorway_world();
        assert!(place_item_tile(&mut w2, 10, 17, 42, 0, 1));
        assert!(w2.tile(10, 17).unwrap().kind == 42 && w2.tile(10, 18).unwrap().kind == 42);
    }

    #[test]
    fn test_DW_2_4_chair_direction_picks_the_alternate() {
        // Facing right, the `PlaceRight` alternate wins (`TileObject.cs:209`):
        // placement style 1 → frame_x 18. Facing left keeps the base (0).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 15, 0, 1));
        assert!(w.tile(10, 18).unwrap().active() && w.tile(10, 19).unwrap().active());
        assert_eq!(w.tile(10, 18).unwrap().frame_x, 18);
        let mut w2 = object_world();
        assert!(place_item_tile(&mut w2, 10, 19, 15, 0, -1));
        assert_eq!(w2.tile(10, 18).unwrap().frame_x, 0);
    }

    #[test]
    fn test_DW_2_4_partial_overlap_refuses_without_world_change() {
        // A block occupies one of the bench's two cells: the route refuses and
        // nothing is written — not even the free origin cell.
        let mut w = object_world();
        set_active(&mut w, 11, 19, 1);
        let before = w.revision;
        assert!(!place_item_tile(&mut w, 10, 19, 18, 0, 1));
        assert!(!w.tile(10, 19).unwrap().active(), "origin cell untouched");
        assert_eq!(w.tile(11, 19).unwrap().kind, 1, "occupant untouched");
        assert_eq!(w.revision, before, "no world change on refusal");
    }

    #[test]
    fn test_DW_2_4_world_edge_refuses() {
        // `CanPlace`'s 5-tile margin (`TileObject.cs:215`) rejects x = 3.
        let mut w = object_world();
        assert!(!place_item_tile(&mut w, 3, 19, 18, 0, 1));
        assert!(!w.tile(3, 19).unwrap().active());
        assert!(!place_item_tile(&mut w, 200, 19, 18, 0, 1), "out of world");
    }

    #[test]
    fn place_style_shifts_the_frame_column() {
        // The item's `placeStyle` feeds `CalculatePlacementStyle`
        // (`TileObjectData.cs:1990-1998`): style 1 shifts the horizontal
        // style column by the bench's 36 px full width (2 cells x 18).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 1, 1));
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 54);
    }

    #[test]
    fn test_DW_1_1_seeds_plant_on_csharp_soil_sets() {
        // Every seed style plants on every soil of its C# set — the tile
        // reads `82` with `frameX = 18*style, frameY = 0` (:45458-45477).
        for (style, soils) in ALCH_SOIL_TABLE {
            for soil in soils {
                let mut w = soil_world(*soil);
                assert!(
                    place_item_tile(&mut w, 10, 19, 82, style, 1),
                    "style {style} must plant on soil {soil}"
                );
                let t = *w.tile(10, 19).unwrap();
                assert!(t.active() && t.kind == 82, "soil {soil}");
                assert_eq!(
                    (t.frame_x, t.frame_y),
                    ((18 * style) as i16, 0),
                    "soil {soil}"
                );
            }
        }
    }

    #[test]
    fn test_DW_1_1_seed_liquid_rules_by_style() {
        // :45442-45455 — styles 0-3 refuse ANY liquid in the herb cell, 4/6
        // refuse lava only, 5 (ash) refuses everything but lava. Each style
        // sits on the first soil of its own set so only the liquid verdict
        // differs.
        for style in 0..=6i32 {
            let soil = ALCH_SOIL_TABLE[style as usize].1[0];
            // Water.
            let mut w = soil_world(soil);
            set_liquid(&mut w, 10, 19, 200, false);
            assert_eq!(
                place_item_tile(&mut w, 10, 19, 82, style, 1),
                matches!(style, 4 | 6),
                "style {style} in water"
            );
            // Lava.
            let mut wl = soil_world(soil);
            set_liquid(&mut wl, 10, 19, 200, true);
            assert_eq!(
                place_item_tile(&mut wl, 10, 19, 82, style, 1),
                style == 5,
                "style {style} in lava"
            );
        }
        // The style-5 positive control actually wrote the herb.
        let mut w5 = soil_world(57);
        set_liquid(&mut w5, 10, 19, 200, true);
        assert!(place_item_tile(&mut w5, 10, 19, 82, 5, 1));
        let t = *w5.tile(10, 19).unwrap();
        assert!(
            t.active() && t.kind == 82 && t.liquid == 200,
            "liquid survives the write"
        );
    }

    #[test]
    fn test_DW_1_1_seed_rejected_on_slope_or_half_brick_soil() {
        // :45390-45392 — the soil must be flat (`!halfBrick && slope == 0`).
        let cases: [(&str, fn(&mut Tile)); 2] = [
            ("half brick", |t: &mut Tile| {
                t.flags |= crate::tile::FLAG_HALF_BRICK
            }),
            ("slope", |t: &mut Tile| t.set_slope(1)),
        ];
        for (label, mutate) in cases {
            let mut w = soil_world(2);
            let i = w.index(10, 20);
            mutate(&mut w.tiles[i]);
            assert!(
                !place_item_tile(&mut w, 10, 19, 82, 0, 1),
                "{label} soil refuses the seed"
            );
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "{label} soil writes nothing"
            );
        }
    }

    #[test]
    fn test_DW_1_1_seed_rejected_on_actuated_soil() {
        // :45390-45392 — the soil must pass `tile2.nactive()` (`Tile.cs:240-246`:
        // active AND not inActive). `Wiring.Actuate` keeps `FLAG_ACTIVE` and sets
        // `FLAG_INACTIVE`, so an actuated soil reads `active()` but fails
        // `nactive()` — vanilla refuses the seed and nothing is written. The
        // same soil without the actuated flag accepts it.
        let mut w = soil_world(2);
        let i = w.index(10, 20);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        assert!(
            !place_item_tile(&mut w, 10, 19, 82, 0, 1),
            "actuated soil refuses the seed"
        );
        assert!(
            !w.tile(10, 19).unwrap().active(),
            "actuated soil writes nothing"
        );
        // `Wiring.ReActive` clears the actuation bit; the identical soil now plants.
        w.tiles[i].flags &= !crate::tile::FLAG_INACTIVE;
        assert!(
            place_item_tile(&mut w, 10, 19, 82, 0, 1),
            "de-actuated soil accepts the seed"
        );
        let t = *w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 82 && t.frame_x == 0 && t.frame_y == 0);
    }

    #[test]
    fn test_DW_1_1_place_item_tile_routes_82_via_place_alch() {
        // `Player.cs:39772` excludes 82 from the TileObject route even though
        // the registry now has the def: the planter box (380) is accepted as
        // soil by `PlaceAlch` (:45395-45397) while the registry anchor list
        // (`AnchorAlternateTiles {78}`) would refuse it — the discriminator
        // proves the seed went through the legacy `PlaceAlch` branch.
        let mut w = soil_world(380);
        assert!(place_item_tile(&mut w, 10, 19, 82, 0, 1));
        assert_eq!(w.tile(10, 19).unwrap().kind, 82);
        // And a style-2 seed carries its own frames.
        let mut w2 = soil_world(0);
        assert!(place_item_tile(&mut w2, 10, 19, 82, 2, 1));
        assert_eq!(
            (
                w2.tile(10, 19).unwrap().frame_x,
                w2.tile(10, 19).unwrap().frame_y
            ),
            (36, 0)
        );
    }

    #[test]
    fn test_DW_1_1_alch_reframe_keeps_herb_frames() {
        // The post-placement 5x5 re-frame must leave the fresh herb — and any
        // sibling-stage herb — framed as written (`Main.tileFrameImportant
        // [82..84]` are boot statics, `Main.cs:7685-7687`).
        let mut w = soil_world(0); // dirt — a style-2 (Blinkroot) soil
        // A blooming sibling with explicit frames next to the planting spot.
        let i = w.index(12, 19);
        w.tiles[i] = Tile {
            kind: 84,
            frame_x: 90,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(place_item_tile(&mut w, 10, 19, 82, 2, 1));
        let fresh = *w.tile(10, 19).unwrap();
        assert_eq!((fresh.kind, fresh.frame_x, fresh.frame_y), (82, 36, 0));
        let sibling = *w.tile(12, 19).unwrap();
        assert_eq!(
            (sibling.kind, sibling.frame_x, sibling.frame_y),
            (84, 90, 0),
            "reframe_area must not re-frame the sibling herb"
        );
        for kind in [82u16, 83, 84] {
            assert!(w.frame_important[kind as usize], "frame_important[{kind}]");
        }
        // The paint rode over from the soil (`CopyPaintAndCoating`).
        let mut wp = soil_world(2);
        let i = wp.index(10, 20);
        wp.tiles[i].set_paint(7);
        assert!(place_item_tile(&mut wp, 10, 19, 82, 0, 1));
        assert_eq!(wp.tile(10, 19).unwrap().paint(), 7);
    }

    #[test]
    fn test_DW_1_3_planter_box_items_write_frame_y() {
        // `WorldGen.cs:60226-60230` — `frameY = 18 * style`, type 380, the
        // rest of the cell untouched (frames of a virgin cell are 0).
        for style in 0..=7i32 {
            let mut w = object_world();
            assert!(
                place_item_tile(&mut w, 10, 19, 380, style, 1),
                "style {style}"
            );
            let t = *w.tile(10, 19).unwrap();
            assert!(t.active() && t.kind == 380, "style {style}");
            assert_eq!(t.frame_y, (18 * style) as i16, "style {style}");
            assert_eq!(t.frame_x, 0, "frameX stays as the cell had it");
            assert!(w.frame_important[380], "frame_important[380]");
        }
    }

    #[test]
    fn test_DW_1_3_planter_box_stays_out_of_the_registry() {
        // Tile 380 is not in `TileObjectData` (vanilla places it through the
        // legacy case): no registry def, and an occupied cell refuses.
        assert!(crate::tileobject::def_for_tile(380).is_none());
        let mut w = object_world();
        set_active(&mut w, 10, 19, 1);
        let before = w.revision;
        assert!(!place_item_tile(&mut w, 10, 19, 380, 0, 1));
        assert_eq!(w.revision, before, "a refused placement writes nothing");
        assert!(w.tile(10, 19).unwrap().active() && w.tile(10, 19).unwrap().kind == 1);
    }

    /// S1-P-i32: `WorldGen.PlaceTile` case 19 (`WorldGen.cs:60221-60225`) —
    /// the platform legacy write is `frameY = 18 * style`, type 19, with
    /// `Main.tileFrameImportant[19]` mirrored (`Main.cs:7654`). The pre-fix
    /// port fell through to the generic single-tile write, which left the
    /// frames at the `-1` sentinel and never marked the kind frame-important,
    /// so the re-frame drew the platform as a plain merged block. Written,
    /// not run (code-only mode).
    #[test]
    fn s1_p_i32_platform_items_write_the_style_frame_y() {
        // Tile 19 is registered but `CustomPlace`-false: the legacy route.
        assert!(crate::tileobject::def_for_tile(19).is_some());
        assert!(!crate::tileobject::routes_through_tile_object(19, 0));
        for style in [0i32, 1, 2, 5] {
            let mut w = object_world();
            assert!(
                place_item_tile(&mut w, 10, 19, 19, style, 1),
                "style {style}"
            );
            let t = *w.tile(10, 19).unwrap();
            assert!(t.active() && t.kind == 19, "style {style}");
            assert_eq!(t.frame_y, (18 * style) as i16, "style {style} frameY");
            assert!(w.frame_important[19], "frame_important[19] mirrored");
        }
        // An occupied cell refuses and writes nothing (the case-19 guard).
        let mut w = object_world();
        set_active(&mut w, 10, 19, 1);
        let before = w.revision;
        assert!(!place_item_tile(&mut w, 10, 19, 19, 0, 1));
        assert_eq!(w.revision, before, "a refused placement writes nothing");
    }

    #[test]
    fn seed_style_out_of_range_runs_no_checks_like_the_csharp_switch() {
        // `WorldGen.cs:45393-45455` — a style outside 0-6 matches no switch
        // case, the reject flag stays false, and the herb plants unchecked
        // with `frameX = 18 * style`. Unreachable from the seed items (their
        // placeStyle is 0-6) — pinned 1:1 with the fall-through.
        let mut w = soil_world(1);
        assert!(place_item_tile(&mut w, 10, 19, 82, 9, 1));
        let t = *w.tile(10, 19).unwrap();
        assert_eq!((t.kind, t.frame_x, t.frame_y), (82, 162, 0));
    }

    #[test]
    fn test_DW_1_5_seed_rejected_on_wrong_soil_leaves_cell_untouched() {
        // Style 0 refuses sand (53) and stone (1); style 1 refuses dirt (2).
        // The cell stays empty and the world untouched (`PlaceAlch` returns
        // before any write, :45458).
        for (style, soil) in [(0i32, 53u16), (0, 1), (1, 2), (4, 2), (6, 2)] {
            let mut w = soil_world(soil);
            let before: Vec<Tile> = w.tiles.clone();
            assert!(
                !place_item_tile(&mut w, 10, 19, 82, style, 1),
                "style {style} must refuse soil {soil}"
            );
            assert_eq!(before, w.tiles, "soil {soil}: world unchanged");
        }
    }

    #[test]
    fn test_DW_1_5_lava_rules_refuse_by_style() {
        // Style 4 (waterleaf) in lava: refused, cell and liquid unchanged
        // (the DW-1.5 dirty case); style 5 (fireblossom) is the positive
        // control (covered in DW-1.1's liquid test).
        let mut w = soil_world(53);
        set_liquid(&mut w, 10, 19, 200, true);
        let before: Vec<Tile> = w.tiles.clone();
        assert!(!place_item_tile(&mut w, 10, 19, 82, 4, 1));
        assert_eq!(before, w.tiles, "world unchanged");
        assert!(w.tile(10, 19).unwrap().liquid == 200, "liquid untouched");
    }

    /// Plan-assumption check: the dials have no registry def, so
    /// `place_item_tile` takes the legacy single-tile route and the placed
    /// tile is a plain active 356/663 — exactly the shape `interact_at`'s
    /// dial arm reads (`Player.cs:32636`/`:32646`).
    #[test]
    fn dial_tiles_place_through_the_legacy_route() {
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 356, 0, 1));
        let sundial = *w.tile(10, 19).unwrap();
        assert!(sundial.active() && sundial.kind == 356);
        assert!(place_item_tile(&mut w, 14, 19, 663, 0, 1));
        let moondial = *w.tile(14, 19).unwrap();
        assert!(moondial.active() && moondial.kind == 663);
    }

    // ---- S1-P-i42: the torch placement arm (WorldGen.cs:59798-59842) -------

    #[test]
    fn the_torch_style_rides_frame_y_at_22_per_style() {
        // `:59839` — `frameY = 22 * style`. Every torch style writes its own
        // row; `CheckTorch` keeps frameY (it only sets frameX).
        for style in [0i32, 1, 5, 9, 13, 16, 23] {
            let mut w = object_world();
            assert!(
                place_item_tile(&mut w, 10, 19, 4, style, 1),
                "style {style} places on the floor"
            );
            let t = *w.tile(10, 19).unwrap();
            assert!(t.active() && t.kind == 4, "style {style}");
            assert_eq!(t.frame_y, (22 * style) as i16, "style {style} frameY");
        }
    }

    #[test]
    fn the_torch_places_on_a_wall_only_cell() {
        // `:59835` — `tile.wall > 0` holds the torch with no block support.
        let mut w = object_world();
        // (10, 10): no floor (the floor is y = 20), no neighbours — wall only.
        let i = w.index(10, 10);
        w.tiles[i].wall = 2;
        assert!(place_item_tile(&mut w, 10, 10, 4, 0, 1));
        let t = *w.tile(10, 10).unwrap();
        assert!(t.active() && t.kind == 4);
        // CheckTorch's wall arm sets frameX 0 (`:86172-86174`).
        assert_eq!(t.frame_x, 0);
    }

    #[test]
    fn the_torch_refuses_a_supportless_empty_cell() {
        // `:59835` — no wall, no support: the gate refuses, nothing writes.
        let mut w = object_world();
        let before: Vec<Tile> = w.tiles.clone();
        assert!(!place_item_tile(&mut w, 10, 10, 4, 0, 1));
        assert_eq!(before, w.tiles, "the refused placement writes nothing");
    }

    #[test]
    fn the_torch_places_on_a_side_block() {
        // `:59835` — the left/right solid support (the side arm).
        let mut w = object_world();
        set_active(&mut w, 9, 10, 1); // stone left of (10, 10)
        assert!(place_item_tile(&mut w, 10, 10, 4, 0, 1));
        let t = *w.tile(10, 10).unwrap();
        assert!(t.active() && t.kind == 4);
        // CheckTorch's left arm sets frameX 22 (`:86164-86166`).
        assert_eq!(t.frame_x, 22);
    }

    #[test]
    fn the_plain_torch_refuses_liquid_but_the_water_torches_place() {
        // `:59454-59467` — a liquid cell refuses every style except 8/11/17
        // (the WaterTorches, ItemID.cs:1372).
        for style in [0i32, 1, 5, 9, 13, 16, 23] {
            let mut w = object_world();
            set_liquid(&mut w, 10, 19, 200, false);
            assert!(
                !place_item_tile(&mut w, 10, 19, 4, style, 1),
                "style {style} refuses water"
            );
            assert!(!w.tile(10, 19).unwrap().active(), "style {style} writes nothing");
        }
        for style in [8i32, 11, 17] {
            let mut w = object_world();
            set_liquid(&mut w, 10, 19, 200, false);
            assert!(
                place_item_tile(&mut w, 10, 19, 4, style, 1),
                "water torch style {style} places"
            );
            let t = *w.tile(10, 19).unwrap();
            assert!(t.active() && t.kind == 4, "style {style}");
            assert_eq!(t.frame_y, (22 * style) as i16, "style {style} frameY");
            assert_eq!(t.liquid, 200, "the liquid survives the write");
        }
    }

    #[test]
    fn the_water_torch_places_in_lava_the_csharp_gate_reads_water_placement_only() {
        // The 8/11/17 sub-tiles carry `lava_placement: Allowed` (registry
        // D_4_S8/S11/S17), so the C#'s `WaterPlacement != Allowed` gate
        // (`:59463`) — which reads WaterPlacement only — lets them into lava
        // too. The port's gate reads water_placement (Allowed), so lava is
        // accepted exactly like the C# (the lava_death flag is a separate
        // post-placement system, unported).
        let mut w = object_world();
        set_liquid(&mut w, 10, 19, 200, true);
        assert!(place_item_tile(&mut w, 10, 19, 4, 8, 1));
        assert!(w.tile(10, 19).unwrap().active());
    }

    #[test]
    fn the_torch_refuses_an_occupied_cell() {
        // `:59373` — the write needs an empty cell.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 1);
        let before: Vec<Tile> = w.tiles.clone();
        assert!(!place_item_tile(&mut w, 10, 19, 4, 0, 1));
        assert_eq!(before, w.tiles, "the occupied cell is untouched");
    }

    // ---- W4-d1 retire: the forced-placement route (WorldGen.cs:59342) ----

    /// `WorldGen.PlaceTile(forced: true)` (`:59373` — the forced flag is
    /// the FIRST disjunct): the unbreakable-wall refusal the plain
    /// `place_tile` carries is BYPASSED — the C# `PlaceTile` has no such
    /// gate at all. The wall survives the write (`:59473-59476`'s Clear
    /// excludes walls), the frames re-frame through the tail's
    /// `SquareTileFrame`. WRITTEN NOT RUN (code-only retire round).
    #[test]
    fn w4_d1_the_forced_place_bypasses_the_unbreakable_wall_refusal() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i].wall = WALL_UNBREAKABLE;
        // The plain route refuses (the port's CanKillTile-derived gate).
        assert!(!place_tile(&mut w, 10, 19, 53));
        assert!(!w.tile(10, 19).unwrap().active());
        // The forced route places — the wall-350 cell takes the sand.
        assert!(place_tile_forced(&mut w, 10, 19, 53));
        let t = *w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 53);
        assert_eq!(t.wall, WALL_UNBREAKABLE, "the wall survives the Clear");
        assert!(t.frame_x >= 0 && t.frame_y >= 0, "the SquareTileFrame tail");
    }

    /// The forced route keeps the cell's liquid (the Clear is
    /// `Tile | TilePaint | Slope` — `TileDataType.Liquid` excluded,
    /// `:59475`), wipes the stale paint/slope of a broken occupant and
    /// re-frames. The served kinds are on none of the `:59454-59472`
    /// liquid-refusal lists, so a wet cell places. WRITTEN NOT RUN.
    #[test]
    fn w4_d1_the_forced_place_keeps_liquid_and_clears_the_stale_state() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i].liquid = 200;
        w.tiles[i].set_paint(5);
        w.tiles[i].set_slope(2);
        assert!(place_tile_forced(&mut w, 10, 19, 123));
        let t = *w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 123);
        assert_eq!(t.liquid, 200, "the liquid survives");
        assert_eq!(t.paint(), 0, "the stale paint wiped");
        assert_eq!(t.slope(), 0, "the stale slope wiped");
        // The bounds + TileID.Count gates (`:59349-59354`).
        assert!(!place_tile_forced(&mut w, 999, 19, 53));
        assert!(!place_tile_forced(&mut w, 10, 19, crate::tiledata::TILE_COUNT as u16));
    }

    /// The 53's anniversary paint arm (`:60243-60246`: `color(7)` when
    /// `tenthAnniversaryWorld && !remixWorld && !notTheBeesWorld &&
    /// !dontStarveWorld`); a classic world keeps the paint 0, and each
    /// exclusion flag folds the arm off. WRITTEN NOT RUN.
    #[test]
    fn w4_d1_the_forced_sand_paint_is_the_anniversary_arm() {
        use crate::seeds::SeedFlags;
        let mut w = object_world();
        assert!(place_tile_forced(&mut w, 10, 19, 53));
        assert_eq!(w.tile(10, 19).unwrap().paint(), 0, "classic world");
        let mut w = object_world();
        w.seeds.insert(SeedFlags::TENTH_ANNIVERSARY);
        assert!(place_tile_forced(&mut w, 10, 19, 53));
        assert_eq!(w.tile(10, 19).unwrap().paint(), 7, "the anniversary 7");
        for excl in [SeedFlags::REMIX, SeedFlags::NOT_THE_BEES, SeedFlags::DONT_STARVE] {
            let mut w = object_world();
            w.seeds.insert(SeedFlags::TENTH_ANNIVERSARY);
            w.seeds.insert(excl);
            assert!(place_tile_forced(&mut w, 10, 19, 53));
            assert_eq!(w.tile(10, 19).unwrap().paint(), 0, "the exclusion folds off");
        }
        // A non-sand kind never paints (the arm is 53/396/397 only).
        let mut w = object_world();
        w.seeds.insert(SeedFlags::TENTH_ANNIVERSARY);
        assert!(place_tile_forced(&mut w, 10, 19, 59));
        assert_eq!(w.tile(10, 19).unwrap().paint(), 0);
    }
}
