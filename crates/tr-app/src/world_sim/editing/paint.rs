//! The paint-tool USE family of the editing loop (W4-g79): the Paintbrush /
//! Paint Roller / Paint Scraper and their Spectre variants
//! (`Player.PlaceThing_Paintbrush` / `PlaceThing_PaintRoller` /
//! `PlaceThing_PaintScraper`, `Player.cs:41808-41925`), the shared
//! `TryPainting` / `ApplyCoating` / `ApplyPaint` chain (`:41946-42024`) and the
//! scraper's long-moss harvest (`:41822-41868`), answered ahead of the
//! generic place route; `use_paint_tool` is imported by `editing`, so the
//! `update_editing` caller seam is unchanged.
//!
//! The tools consume a PAINT/COATING item from the inventory (the
//! `FindPaintOrCoating` pick, `Player.cs:41927-41944`), never the tool itself.
//! The world writes are the core's `edit::paint_tile` / `paint_wall` /
//! `paint_coat_tile` / `paint_coat_wall` (`WorldGen.cs:44110-44252`); the dust
//! (`paintEffect` / `paintCoatEffect`, `:44064-44108`) and the 63/64
//! broadcasts ride the app-side wrappers here (the broadcast through the
//! port's `request_edit` kinds 4..7, the `apply_edit` stand-in for
//! `MessageBuffer.cs:2909-2948`). The cursor item-icon
//! (`Player.cursorItemIconEnabled`) is set by the gates and cleared at the
//! `update_editing` head (`Main.MouseOversClear`, `Main.cs:46503-46513`); its
//! draw site is the separate `DrawInterface_40` port (W4-g79 phase 5).

use glam::Vec2;
use tr_core::particles::Color;
use tr_core::tile_light::FastRandom;
use tr_core::{TILE_PX, edit};

use crate::app::App;
use crate::item::ItemSlot;
use crate::registry;

use super::super::items::spawn_item_drop_px;
use super::{
    ONE_TICK_SECS, break_tile, held_use_time_ticks, mining_tile_range, start_use_timers,
    ticks_of, tile_in_reach_rect,
};

/// `ItemID.Sets.IsPaintScraper` (`ItemID.cs:496`): `{1100, 1545}` — the Paint
/// Scraper and the Spectre Paint Scraper.
fn is_paint_scraper(kind: u16) -> bool {
    matches!(kind, 1100 | 1545)
}

/// The paint tool the held item is, if any. The brush family (1071/1543)
/// paints tiles, the roller family (1072/1544) paints walls, the scraper
/// family (1100/1545) strips paint/coatings and harvests long moss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaintTool {
    Brush,
    Roller,
    Scraper,
}

fn paint_tool(kind: u16) -> Option<PaintTool> {
    match kind {
        1071 | 1543 => Some(PaintTool::Brush),
        1072 | 1544 => Some(PaintTool::Roller),
        _ if is_paint_scraper(kind) => Some(PaintTool::Scraper),
        _ => None,
    }
}

/// `Player.IsInTileInteractionRange(tx, ty, TileReachCheckSettings.Simple,
/// item.tileBoost + blockRange)` (`Player.cs:32025-32029`) for the paint tools
/// (`:41810`/`:41893`/`:41911`): the mining reach rect plus the held item's
/// `tileBoost` and the builder `blockRange` (the Builder buff 107 / Toolbelt
/// 407, W4-g79 phase 4a).
fn paint_in_reach(app: &App, tx: i32, ty: i32, held: ItemSlot) -> bool {
    let (range_x, range_y) = mining_tile_range(app);
    let tile_boost = registry::get(held.kind).map_or(0, |d| d.tile_boost);
    let block_range = app.player.as_ref().map_or(0, |p| p.block_range);
    app.player.as_ref().is_some_and(|p| {
        tile_in_reach_rect(p, tx, ty, range_x, range_y, tile_boost + block_range)
    })
}

/// `Player.ApplyItemTime(sItem, multiplier)` (`Player.cs:4421-4430`): `val =
/// (int)(useTime * multiplier)`, floored at 1 when `useTime > 0`. The paint
/// chain re-arms the ACTION cadence with the tileSpeed (brush / scraper-tile)
/// or wallSpeed (roller / scraper-wall) multiplier. Returns the seconds.
fn apply_item_time_secs(app: &App, held: ItemSlot, multiplier: f32) -> f32 {
    let use_time = held_use_time_ticks(held);
    let mut val = (use_time as f32 * multiplier) as i32;
    if use_time > 0 {
        val = val.max(1);
    }
    val as f32 * ONE_TICK_SECS
}

/// The paint/coating item the tool applies — `Player.FindPaintOrCoating`
/// (`Player.cs:41927-41944`)'s pick, resolved to its inventory slot so the
/// consume can write it back. The ammo slots 54..57 win, else the first over
/// the flat 0..58 carry range (grid, coins, ammo). `None` = nothing stocked.
fn find_paint_slot(app: &App) -> Option<PaintSlot> {
    // `:41929-41935` — the ammo row first.
    for (i, s) in app.equip.ammo.iter().enumerate() {
        if s.count > 0 && crate::item::paint_or_coating(s.kind) {
            return Some(PaintSlot::Ammo(i));
        }
    }
    // `:41936-41942` — the flat 0..58 pass: grid 0..49, coins 50..53, ammo
    // 54..57 again.
    for (i, s) in app.inventory.iter().enumerate() {
        if s.count > 0 && crate::item::paint_or_coating(s.kind) {
            return Some(PaintSlot::Grid(i));
        }
    }
    for (i, s) in app.equip.coins.iter().enumerate() {
        if s.count > 0 && crate::item::paint_or_coating(s.kind) {
            return Some(PaintSlot::Coin(i));
        }
    }
    for (i, s) in app.equip.ammo.iter().enumerate() {
        if s.count > 0 && crate::item::paint_or_coating(s.kind) {
            return Some(PaintSlot::Ammo(i));
        }
    }
    None
}

/// The inventory location of the `FindPaintOrCoating` pick — the port's split
/// arrays (`App.inventory` + `EquipSlots`) keep the C# 0..58 index order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaintSlot {
    Grid(usize),
    Coin(usize),
    Ammo(usize),
}

impl PaintSlot {
    fn get(self, app: &App) -> ItemSlot {
        match self {
            PaintSlot::Grid(i) => app.inventory[i],
            PaintSlot::Coin(i) => app.equip.coins[i],
            PaintSlot::Ammo(i) => app.equip.ammo[i],
        }
    }

    /// `targetItem.stack--; if (stack <= 0) SetDefaults(0)` (`Player.cs:41969-
    /// 41973` etc.): one paint/coating consumed, the slot emptied to air.
    fn consume_one(self, app: &mut App) {
        let slot = match self {
            PaintSlot::Grid(i) => &mut app.inventory[i],
            PaintSlot::Coin(i) => &mut app.equip.coins[i],
            PaintSlot::Ammo(i) => &mut app.equip.ammo[i],
        };
        slot.count = slot.count.saturating_sub(1);
        if slot.count == 0 {
            *slot = crate::item::EMPTY_SLOT;
        }
    }
}

/// `WorldGen.paintColor`'s dust carrier (`WorldGen.cs:43921-44033`): the byte
/// table's colour at the C# `Color`'s alpha — white 255 by default, the
/// negative-paint 30 arm alone writes `A = 150` (`:44031`).
fn paint_dust_color(color: u8) -> Color {
    let [r, g, b] = tr_core::tile_light::paint_color_bytes(color);
    Color::new(r, g, b, if color == 30 { 150 } else { 255 })
}

/// `WorldGen.coatingColor` (`WorldGen.cs:44036-44044`): 1 Glow =
/// (235, 170, 255, 255), 2 Echo = (180, 245, 255, 255), anything else
/// `Color.Transparent`.
fn coating_color(coating: u8) -> Color {
    match coating {
        1 => Color::new(235, 170, 255, 255),
        2 => Color::new(180, 245, 255, 255),
        _ => Color::TRANSPARENT,
    }
}

/// `WorldGen.coatingColors(tile, block)` (`WorldGen.cs:44046-44062`): the
/// Glow colour first when the bit rides, then the Echo colour — the list the
/// id-0 `paintCoatEffect` picks its per-dust colour from. Captured BEFORE the
/// coat write clears the bits (the C# reads it at the function head).
fn coating_colors(app: &App, tx: i32, ty: i32, block: bool) -> Vec<Color> {
    let mut colors = Vec::new();
    let Some(t) = app.world.tile(tx, ty) else {
        return colors;
    };
    let (glow, echo) = if block {
        (t.fullbright_block(), app.world.invisible_block(tx, ty))
    } else {
        (
            app.world.fullbright_wall(tx, ty),
            app.world.invisible_wall(tx, ty),
        )
    };
    if glow {
        colors.push(coating_color(1));
    }
    if echo {
        colors.push(coating_color(2));
    }
    colors
}

/// One `paintEffect` / `paintCoatEffect` dust (`WorldGen.cs:44072-44083` /
/// `:44090-44106`): `Dust.NewDust((x*16, y*16), 16, 16, 143, 0, 0, 50,
/// newColor)` then the `genRand.Next(2)` arm — noGravity + ×1.2 scale, else
/// ×0.5. The pool's own rolls ride the `UpdatePlayers` stream (the paint
/// runs in `Player.ItemCheck` under the `Main.cs:17754` swap); the
/// `genRand.Next(2)` takes the one-throw-seeded `FastRandom` stand-in (the
/// `break_tile` palm-roll precedent). `new_color` is this dust's resolved
/// colour (the id-0 coating arm picks per dust, `Main.rand.Next(count)`).
fn one_paint_dust(app: &mut App, tx: i32, ty: i32, new_color: Color) {
    let ctx = app.spawn_ctx();
    let idx = app.dust.new_dust(
        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        &ctx,
        &app.world,
        Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX),
        16,
        16,
        143,
        0.0,
        0.0,
        50,
        new_color,
        1.0,
    );
    if idx == tr_core::particles::DUST_SENTINEL {
        return;
    }
    let mut rng = FastRandom::new(
        app.rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .next() as u64,
    );
    let slot = app.dust.slot_mut(idx);
    if rng.next(2) == 0 {
        slot.no_gravity = true;
        slot.scale *= 1.2;
    } else {
        slot.scale *= 0.5;
    }
}

/// `WorldGen.paintEffect(x, y, color, oldColor)` (`WorldGen.cs:44064-44085`):
/// ten dust 143 in the NEW paint's colour — the id-0 scrape falls back to the
/// OLD colour (`color2 = oldColor`, `:44067-44070`).
fn paint_effect(app: &mut App, tx: i32, ty: i32, color: u8, old_color: u8) {
    let color2 = if color == 0 { old_color } else { color };
    let new_color = paint_dust_color(color2);
    for _ in 0..10 {
        one_paint_dust(app, tx, ty, new_color);
    }
}

/// `WorldGen.paintCoatEffect(x, y, paintCoatId, oldColors)`
/// (`WorldGen.cs:44087-44108`): ten dust 143 in the coating's colour; the
/// id-0 clear picks each dust's colour from the PRE-write coating list via
/// `Main.rand.Next(oldColors.Count)` (`:44093-44096`) — the stage stream, not
/// the genRand stand-in.
fn paint_coat_effect(app: &mut App, tx: i32, ty: i32, coat_id: u8, old_colors: &[Color]) {
    let color = coating_color(coat_id);
    for _ in 0..10 {
        let mut new_color = color;
        if coat_id == 0 && !old_colors.is_empty() {
            let pick = app
                .rngs
                .stream(crate::app::RngKey::UpdatePlayers)
                .next_max(old_colors.len() as i32) as usize;
            new_color = old_colors[pick];
        }
        one_paint_dust(app, tx, ty, new_color);
    }
}

/// `WorldGen.paintTile(x, y, color, broadCast: true)` (`WorldGen.cs:44110-
/// 44132`): the C# order is dust FIRST (`paintEffect` `:44122-44125`), then
/// the colour write (`:44126`), then the broadcast (`SendData(63)`
/// `:44127-44130`) — the refusals (null/inactive/same colour) run before any
/// of the three. The old colour is read pre-write for the id-0 dust
/// fallback. The broadcast is the port's `request_edit` kind 4 (tile paint)
/// — `apply_edit` replays it.
fn paint_tile_broadcast(app: &mut App, tx: i32, ty: i32, color: u8) -> bool {
    let Some(t) = app.world.tile(tx, ty).copied() else {
        return false;
    };
    if !t.active() || t.paint() == color {
        return false;
    }
    paint_effect(app, tx, ty, color, t.paint());
    // The pre-gates above mirror the core's refusals, so the write always
    // lands here — the core call stays the authoritative gate.
    if !edit::paint_tile(&mut app.world, tx, ty, color) {
        return false;
    }
    app.request_edit(tx, ty, 4, color as u16);
    app.world_key = None;
    true
}

/// `WorldGen.paintWall(x, y, color, broadCast: true)` (`WorldGen.cs:44230-
/// 44252`): the wall colour write; here the broadcast (`SendData(64)`,
/// `:44243-44246`) precedes the dust (`:44247-44250`) — the C#'s wall order
/// differs from the tile's. Kind 5 = wall paint.
fn paint_wall_broadcast(app: &mut App, tx: i32, ty: i32, color: u8) -> bool {
    let old = app.world.wall_paint(tx, ty);
    if !edit::paint_wall(&mut app.world, tx, ty, color) {
        return false;
    }
    app.request_edit(tx, ty, 5, color as u16);
    app.world_key = None;
    paint_effect(app, tx, ty, color, old);
    true
}

/// `WorldGen.paintCoatTile(x, y, id, broadcast: true)` (`WorldGen.cs:44134-
/// 44180`): the old coating colours are captured BEFORE the write (the C#
/// reads `coatingColors` at the head, `:44141`); the broadcast (`63` with
/// `number7 = 1f`, `:44171-44174`) precedes the dust (`:44175-44178`). Kind 6
/// = tile coating (the `apply_edit` arm reads the id out of `tile`).
fn paint_coat_tile_broadcast(app: &mut App, tx: i32, ty: i32, coat_id: u8) -> bool {
    let old_colors = coating_colors(app, tx, ty, true);
    if !edit::paint_coat_tile(&mut app.world, tx, ty, coat_id) {
        return false;
    }
    app.request_edit(tx, ty, 6, coat_id as u16);
    app.world_key = None;
    paint_coat_effect(app, tx, ty, coat_id, &old_colors);
    true
}

/// `WorldGen.paintCoatWall(x, y, id, broadcast: true)` (`WorldGen.cs:44182-
/// 44228`): the wall twin — head colours, write, `64`+1f broadcast, dust.
/// Kind 7 = wall coating.
fn paint_coat_wall_broadcast(app: &mut App, tx: i32, ty: i32, coat_id: u8) -> bool {
    let old_colors = coating_colors(app, tx, ty, false);
    if !edit::paint_coat_wall(&mut app.world, tx, ty, coat_id) {
        return false;
    }
    app.request_edit(tx, ty, 7, coat_id as u16);
    app.world_key = None;
    paint_coat_effect(app, tx, ty, coat_id, &old_colors);
    true
}

/// `Player.ApplyPaint` (`Player.cs:41994-42024`): the same-colour refusal
/// BEFORE the write (`tile.color() != paint` / `wallColor() != paint`,
/// `:41999`/`:42012` — the core `paint_tile`/`paint_wall` refuse it again),
/// then the consume + the speed-scaled `ApplyItemTime` (wallSpeed on walls,
/// tileSpeed on tiles). The `applyItemAnimation: false` callers are the
/// unported auto-paint-on-build tails.
fn apply_paint(app: &mut App, tx: i32, ty: i32, wall: bool, slot: PaintSlot, paint: u8) -> bool {
    let already = if wall {
        app.world.wall_paint(tx, ty) == paint
    } else {
        app.world.tile(tx, ty).is_some_and(|t| t.paint() == paint)
    };
    if already {
        return false;
    }
    let wrote = if wall {
        paint_wall_broadcast(app, tx, ty, paint)
    } else {
        paint_tile_broadcast(app, tx, ty, paint)
    };
    if !wrote {
        return false;
    }
    slot.consume_one(app);
    let held = app.inventory[app.selected];
    let mult = app.player.as_ref().map_or(1.0, |p| {
        if wall { p.wall_speed_multiplier() } else { p.tile_speed_multiplier() }
    });
    app.item_time = apply_item_time_secs(app, held, mult);
    true
}

/// `Player.ApplyCoating` (`Player.cs:41962-41992`): the coating write (the
/// core's bit arms own the same-state refusals), the consume and the
/// speed-scaled cadence.
fn apply_coating(app: &mut App, tx: i32, ty: i32, wall: bool, slot: PaintSlot, coat: u8) -> bool {
    let wrote = if wall {
        paint_coat_wall_broadcast(app, tx, ty, coat)
    } else {
        paint_coat_tile_broadcast(app, tx, ty, coat)
    };
    if !wrote {
        return false;
    }
    slot.consume_one(app);
    let held = app.inventory[app.selected];
    let mult = app.player.as_ref().map_or(1.0, |p| {
        if wall { p.wall_speed_multiplier() } else { p.tile_speed_multiplier() }
    });
    app.item_time = apply_item_time_secs(app, held, mult);
    true
}

/// `Player.TryPainting` (`Player.cs:41946-41960`): the `FindPaintOrCoating`
/// pick, then the coating arm FIRST (`paintCoating != 0`) and the paint arm
/// (`paint != 0`) — a single item never carries both (`Item.SetDefaults`),
/// but the C# checks both and the port keeps the order. Returns whether
/// either arm landed a write.
fn try_painting(app: &mut App, tx: i32, ty: i32, wall: bool) -> bool {
    let Some(slot) = find_paint_slot(app) else {
        return false;
    };
    let pick = slot.get(app);
    let def = registry::get(pick.kind);
    let coat = def.map_or(0, |d| d.paint_coating) as u8;
    let paint = def.map_or(0, |d| d.paint) as u8;
    let mut wrote = false;
    if coat != 0 {
        wrote |= apply_coating(app, tx, ty, wall, slot, coat);
    }
    if paint != 0 {
        wrote |= apply_paint(app, tx, ty, wall, slot, paint);
    }
    wrote
}

/// `Player.PlaceThing_PaintScraper_TryScraping`'s cell gate
/// (`Player.cs:41873`): the cell must carry SOMETHING to strip — a
/// painted/coated wall OR a painted/coated active tile (the C#'s bit-or
/// composition). The icon arms on the gate alone, before the action tick.
fn scraper_cell_scrapable(app: &App, tx: i32, ty: i32) -> bool {
    let Some(t) = app.world.tile(tx, ty).copied() else {
        return false;
    };
    let wall_scrapable = t.wall > 0
        && (app.world.wall_paint(tx, ty) > 0
            || app.world.invisible_wall(tx, ty)
            || app.world.fullbright_wall(tx, ty));
    let tile_scrapable = t.active()
        && (t.paint() > 0 || app.world.invisible_block(tx, ty) || t.fullbright_block());
    wall_scrapable || tile_scrapable
}

/// `Player.PlaceThing_PaintScraper_TryScraping` (`Player.cs:41870-41889`):
/// the scrape order is the C#'s short-circuit chain: tile paint 0, tile
/// coating 0 (tileSpeed), else wall paint 0, wall coating 0 (wallSpeed) —
/// ONE `ApplyItemTime` per action tick, no paint consumed (the scraper
/// strips, it never applies). The cell gate + icon ran in the caller.
fn scraper_try_scraping(app: &mut App, tx: i32, ty: i32, held: ItemSlot) -> bool {
    if !scraper_cell_scrapable(app, tx, ty) {
        return false;
    }
    let tile_mult = app.player.as_ref().map_or(1.0, |p| p.tile_speed_multiplier());
    let wall_mult = app.player.as_ref().map_or(1.0, |p| p.wall_speed_multiplier());
    // `:41880-41883` — the tile arms first (short-circuit `||`).
    if paint_tile_broadcast(app, tx, ty, 0) || paint_coat_tile_broadcast(app, tx, ty, 0) {
        app.item_time = apply_item_time_secs(app, held, tile_mult);
        return true;
    }
    // `:41884-41887` — then the wall arms.
    if paint_wall_broadcast(app, tx, ty, 0) || paint_coat_wall_broadcast(app, tx, ty, 0) {
        app.item_time = apply_item_time_secs(app, held, wall_mult);
        return true;
    }
    false
}

/// `Player.PlaceThing_PaintScraper_LongMoss`'s cell gate + icon
/// (`Player.cs:41824-41828`): the LongMoss 184 cell arms the icon BEFORE the
/// `ItemTimeIsZero` gate — the harvest itself waits for the action tick.
fn scraper_long_moss_icon(app: &mut App, tx: i32, ty: i32) -> bool {
    let moss = app.world.tile(tx, ty).is_some_and(|t| t.kind == 184);
    if moss {
        if let Some(p) = app.player.as_mut() {
            p.cursor_item_icon_enabled = true;
        }
    }
    moss
}

/// `Player.PlaceThing_PaintScraper_LongMoss` (`Player.cs:41822-41868`): the
/// LongMoss 184 harvest. The `frameX / 22` style is read BEFORE the kill; the
/// drop is the `4349 + style` base with the five switch overrides
/// (`:4348-4365`), rolled at `Main.rand.Next(9) == 0` on the `UpdatePlayers`
/// stream and spawned at `new Point(x, y).ToWorldCoordinates()` = the cell
/// CENTRE `(x*16+8, y*16+8)`. The `SendData(17, …, 0, x, y)` client kill
/// (`:41841-41844`) is the port's `request_edit` kind 0 inside `break_tile`.
/// The kill itself is the shared `break_tile` route (the pick-break
/// precedent). The cell gate + icon ran in the caller.
fn scraper_long_moss(app: &mut App, tx: i32, ty: i32, held: ItemSlot) -> bool {
    let Some(t) = app.world.tile(tx, ty).copied() else {
        return false;
    };
    if t.kind != 184 {
        return false;
    }
    let frame_x = i32::from(t.frame_x);
    break_tile(app, tx, ty);
    // `:41836-41839` — a refused kill (the `CanKillTile` gate) leaves the
    // moss standing: no cadence, no drop roll.
    if app.world.tile(tx, ty).is_some_and(|t| t.active()) {
        return false;
    }
    // `:41840` — the plain `ApplyItemTime(item)` (no multiplier).
    app.item_time = held_use_time_ticks(held).max(0) as f32 * ONE_TICK_SECS;
    // `:41845-41866` — the 1-in-9 drop roll.
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    if rng.next_max(9) == 0 {
        let style = frame_x / 22;
        let kind = match style {
            6 => 4377,
            7 => 4378,
            8 => 4389,
            9 => 5127,
            10 => 5128,
            _ => 4349 + style,
        };
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        spawn_item_drop_px(
            &mut app.items,
            rng,
            kind as u16,
            1,
            Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0),
        );
    }
    true
}

/// The paint-tool USE family — `Player.PlaceThing`'s brush → roller →
/// scraper order (`Player.cs:39586-39588`), the three `PlaceThing_Paint*`
/// gates (`:41808-41925`). `starting`/`inside_swing` are the caller's
/// `ItemCheck` start/action windows; the arm owns its swing start (the
/// brush/roller/scraper are `autoReuse` 15-anim/10-time tools,
/// `Item.cs:13373-13392`/`:13555-13564`) and its `ApplyItemTime` cadence.
/// Returns whether the held item IS a paint tool (the caller's place route
/// must not compete — the tools place no tile/wall).
pub(super) fn use_paint_tool(
    app: &mut App,
    tx: i32,
    ty: i32,
    starting: bool,
    inside_swing: bool,
) -> bool {
    let held = app.inventory[app.selected];
    let Some(tool) = paint_tool(held.kind) else {
        return false;
    };
    // The port maps the C# `controlUseItem` placement family to the
    // right-button `place_down` (the `update_editing` place-route
    // convention, `Player.cs:39580`'s `PlaceThing` runs under the use
    // button in the source).
    if held.count == 0 || app.inventory_open || !app.place_down {
        return true;
    }
    // `:41810`/`:41893`/`:41911` — the reach gate answers before anything
    // else (no icon, no swing out of reach).
    if !paint_in_reach(app, tx, ty, held) {
        return true;
    }
    // The use start: the swing arms on the press edge like the generic
    // place route (`ItemCheck_StartActualUse` → `ApplyItemAnimation`,
    // `Player.cs:51678-51680`); the action itself fires at
    // `ItemTimeIsZero && itemAnimation > 0 && controlUseItem`.
    if starting {
        start_use_timers(app, held);
    }
    let action = inside_swing || (starting && app.item_time <= 0.0);
    match tool {
        PaintTool::Brush => {
            // `PlaceThing_Paintbrush` (`:41909-41925`): the ACTIVE-tile gate,
            // the icon, then `TryPainting` on tiles.
            if !app.world.tile(tx, ty).is_some_and(|t| t.active()) {
                return true;
            }
            if let Some(p) = app.player.as_mut() {
                p.cursor_item_icon_enabled = true;
            }
            if action {
                try_painting(app, tx, ty, false);
            }
        }
        PaintTool::Roller => {
            // `PlaceThing_PaintRoller` (`:41891-41907`): the wall > 0 gate,
            // the icon, then `TryPainting(paintingAWall: true)`.
            if !app.world.tile(tx, ty).is_some_and(|t| t.wall > 0) {
                return true;
            }
            if let Some(p) = app.player.as_mut() {
                p.cursor_item_icon_enabled = true;
            }
            if action {
                try_painting(app, tx, ty, true);
            }
        }
        PaintTool::Scraper => {
            // `PlaceThing_PaintScraper` (`:41808-41820`): the null-tile gate,
            // then TryScraping FIRST and LongMoss — both called
            // unconditionally inside the reach gate, each owning its own
            // cell gate + icon (which arm every tick, `:41828`/`:41877`
            // precede the `ItemTimeIsZero` gates) and its own action gate.
            if !app.world.in_bounds(tx, ty) {
                return true;
            }
            let scrapable = scraper_cell_scrapable(app, tx, ty);
            if scrapable {
                if let Some(p) = app.player.as_mut() {
                    p.cursor_item_icon_enabled = true;
                }
            }
            let moss = scraper_long_moss_icon(app, tx, ty);
            if action {
                if scrapable {
                    scraper_try_scraping(app, tx, ty, held);
                }
                // The moss arm re-reads the action gate AFTER the scrape
                // (`:41829`): a scrape that landed re-armed `item_time`, so
                // the moss harvest waits for the next action tick.
                if moss && ticks_of(app.item_time) <= 0 && app.swing > 0.0 {
                    scraper_long_moss(app, tx, ty, held);
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::super::tests::{object_world, slot};
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::FLAG_ACTIVE;

    /// A paint-tool rig: the 40x40 stone-floor world, the player three tiles
    /// right of (10, 20) (inside the base reach), the held tool selected and
    /// the use button down. The camera parks on the target cell so the dust
    /// spawn cull keeps it on screen; the `UpdatePlayers` stream is pinned
    /// (the dust/genRand rolls draw it lazily otherwise).
    fn paint_app(tool: u16) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 13.0 * TILE_PX;
        p.y = 20.0 * TILE_PX;
        app.player = Some(p);
        app.camera = Vec2::new(10.0 * TILE_PX + 8.0, 20.0 * TILE_PX + 8.0);
        app.inventory[app.selected] = slot(tool, 1);
        app.place_down = true;
        // The full dust budget — the default 0 would put every slot past
        // the pressure bands (`Dust.cs:114-152`).
        app.gfx_quality.max_dust_to_draw = 6000;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) =
            tr_core::UnifiedRandom::new(0);
        app
    }

    /// The brush paints the active tile, burns ONE paint from the ammo slot
    /// first (`FindPaintOrCoating`'s 54..57 preference) and re-arms the
    /// tileSpeed cadence — 10 ticks unspeeded (`Player.cs:41909-41925` →
    /// `TryPainting` → `ApplyPaint`, `Item.cs:13377` useTime 10).
    #[test]
    fn w4_g79_the_brush_paints_the_tile_and_consumes_the_ammo_paint() {
        let mut app = paint_app(1071);
        app.equip.ammo[0] = slot(1073, 2); // Red Paint in the ammo row
        app.inventory[5] = slot(1074, 9); // Orange in the grid — loses
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(app.world.tile(10, 20).unwrap().paint(), 1, "Red landed");
        assert_eq!(app.equip.ammo[0].count, 1, "one ammo paint consumed");
        assert_eq!(app.inventory[5].count, 9, "the grid paint untouched");
        assert_eq!(app.inventory[app.selected].count, 1, "the tool stays");
        assert_eq!(ticks_of(app.item_time), 10, "the plain tileSpeed cadence");
        assert!(app.player.as_ref().unwrap().cursor_item_icon_enabled);
        // The dust: ten 143 particles in the Red bytes (255, 0, 0).
        let dusts: Vec<_> = app
            .dust
            .slots()
            .iter()
            .filter(|d| d.active && d.dust_type == 143)
            .collect();
        assert_eq!(dusts.len(), 10, "paintEffect spawns ten");
        assert!(dusts.iter().all(|d| d.color == Color::new(255, 0, 0, 255)));
    }

    /// The same-colour refusal keeps the paint AND the cadence
    /// (`ApplyPaint`'s `tile.color() != paint` gate, `Player.cs:42012`).
    #[test]
    fn w4_g79_the_brush_refuses_the_same_colour_without_consuming() {
        let mut app = paint_app(1071);
        app.equip.ammo[0] = slot(1073, 2);
        assert!(edit::paint_tile(&mut app.world, 10, 20, 1));
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(app.equip.ammo[0].count, 2, "nothing consumed");
        assert_eq!(app.item_time, 0.0, "no cadence re-arm on the refusal");
    }

    /// The roller needs a wall (`PlaceThing_PaintRoller`'s `wall > 0` gate,
    /// `Player.cs:41899`) and paints the WALL colour, not the tile's.
    #[test]
    fn w4_g79_the_roller_paints_the_wall_behind_the_tile() {
        let mut app = paint_app(1072);
        let i = app.world.index(10, 20);
        app.world.tiles[i].wall = 1;
        app.equip.ammo[0] = slot(1073, 1);
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(app.world.wall_paint(10, 20), 1, "the wall took Red");
        assert_eq!(app.world.tile(10, 20).unwrap().paint(), 0, "tile untouched");
        assert_eq!(app.equip.ammo[0], crate::item::EMPTY_SLOT, "last one air");
        // The wall-less cell refuses before the icon.
        let mut app = paint_app(1072);
        app.equip.ammo[0] = slot(1073, 1);
        let i = app.world.index(10, 19);
        app.world.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(use_paint_tool(&mut app, 10, 19, true, false));
        assert_eq!(app.equip.ammo[0].count, 1);
        assert!(!app.player.as_ref().unwrap().cursor_item_icon_enabled);
    }

    /// The Glow coating rides the coating arm (`TryPainting`'s
    /// `paintCoating != 0` first, `Player.cs:41951-41954`): the fullbright
    /// bit lands, the coating consumed, the tile paint untouched.
    #[test]
    fn w4_g79_the_brush_applies_the_glow_coating() {
        let mut app = paint_app(1071);
        app.equip.ammo[0] = slot(4668, 1); // Glow Paint
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert!(app.world.tile(10, 20).unwrap().fullbright_block());
        assert_eq!(app.world.tile(10, 20).unwrap().paint(), 0);
        assert_eq!(app.equip.ammo[0], crate::item::EMPTY_SLOT);
        // The coating dust: ten 143 in the Glow colour (235, 170, 255).
        let glow = Color::new(235, 170, 255, 255);
        let n = app
            .dust
            .slots()
            .iter()
            .filter(|d| d.active && d.dust_type == 143 && d.color == glow)
            .count();
        assert_eq!(n, 10, "paintCoatEffect's Glow colour");
    }

    /// The scraper strips the tile paint FIRST (the `||` chain,
    /// `Player.cs:41880-41887`), consumes nothing and re-arms tileSpeed; a
    /// second action tick takes the wall paint behind it at wallSpeed.
    #[test]
    fn w4_g79_the_scraper_strips_tile_then_wall_without_consuming() {
        let mut app = paint_app(1100);
        let i = app.world.index(10, 20);
        app.world.tiles[i].wall = 1;
        assert!(edit::paint_tile(&mut app.world, 10, 20, 1));
        assert!(edit::paint_wall(&mut app.world, 10, 20, 2));
        app.equip.ammo[0] = slot(1073, 3); // paint aboard — never burned
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(app.world.tile(10, 20).unwrap().paint(), 0, "tile stripped");
        assert_eq!(app.world.wall_paint(10, 20), 2, "the wall waits its turn");
        assert_eq!(ticks_of(app.item_time), 10, "tileSpeed cadence");
        assert_eq!(app.equip.ammo[0].count, 3, "the scraper consumes nothing");
        // The next action tick (mid-swing) takes the wall.
        app.item_time = 0.0;
        assert!(use_paint_tool(&mut app, 10, 20, false, true));
        assert_eq!(app.world.wall_paint(10, 20), 0, "wall stripped");
    }

    /// The scraper's coating clear: the id-0 arm needs a bit set
    /// (`WorldGen.cs:44147-44150`), the dust picks from the PRE-write
    /// coating colours, and the bare cell refuses the whole scrape. The
    /// scrape's own break-dust is filtered out by the 143 type.
    #[test]
    fn w4_g79_the_scraper_clears_the_coating_and_refuses_the_bare_cell() {
        let mut app = paint_app(1100);
        assert!(edit::paint_coat_tile(&mut app.world, 10, 20, 1));
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert!(!app.world.tile(10, 20).unwrap().fullbright_block());
        // The id-0 dust colour is the OLD Glow colour, not Transparent.
        let glow = Color::new(235, 170, 255, 255);
        let n = app
            .dust
            .slots()
            .iter()
            .filter(|d| d.active && d.dust_type == 143 && d.color == glow)
            .count();
        assert_eq!(n, 10, "the clear dust rides the old coating colour");
        // The bare cell: no icon, no cadence.
        let mut app = paint_app(1100);
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert!(!app.player.as_ref().unwrap().cursor_item_icon_enabled);
        assert_eq!(app.item_time, 0.0);
    }

    /// The LongMoss harvest (`Player.cs:41822-41868`): the 184 cell breaks,
    /// the plain `ApplyItemTime` (10 ticks) re-arms, and the 1-in-9 roll
    /// drops the `4349 + frameX/22` item at the cell centre. The seed scan
    /// finds a firing seed; the style-6 override (4377) is pinned directly.
    #[test]
    fn w4_g79_the_scraper_harvests_the_long_moss() {
        let mut saw_drop = false;
        for seed in 0..40i32 {
            let mut app = paint_app(1100);
            let i = app.world.index(10, 19);
            app.world.tiles[i] = tr_core::tile::Tile {
                kind: 184,
                frame_x: 6 * 22, // style 6 → the 4377 override
                ..Default::default()
            };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers) =
                tr_core::UnifiedRandom::new(seed);
            assert!(use_paint_tool(&mut app, 10, 19, true, false));
            assert!(
                !app.world.tile(10, 19).unwrap().active(),
                "the moss broke"
            );
            assert_eq!(ticks_of(app.item_time), 10, "the plain cadence");
            if let Some(drop) = app.items.first() {
                assert_eq!(drop.kind, 4377, "the style-6 override");
                saw_drop = true;
                break;
            }
        }
        assert!(saw_drop, "some seed fires the Next(9)==0 drop");
    }

    /// The reach gate answers before the icon/swing (`Player.cs:41911`):
    /// the Spectre brush's tileBoost 3 + the Toolbelt blockRange widen the
    /// rect, the plain brush refuses the same cell.
    #[test]
    fn w4_g79_the_paint_reach_reads_tile_boost_plus_block_range() {
        // Player at tile 19, target tile 10 — 9 left. The plain brush's
        // 5-wide rect refuses (lx = 14); the Spectre brush (+3) with the
        // Toolbelt's blockRange (+1) reaches (5+3+1 = 9, lx = 10).
        let mut app = paint_app(1071);
        app.player.as_mut().unwrap().x = 19.0 * TILE_PX;
        app.equip.ammo[0] = slot(1073, 1);
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(app.world.tile(10, 20).unwrap().paint(), 0, "out of reach");
        assert!(!app.player.as_ref().unwrap().cursor_item_icon_enabled);
        let mut app = paint_app(1543);
        app.player.as_mut().unwrap().x = 19.0 * TILE_PX;
        app.player.as_mut().unwrap().block_range = 1;
        app.equip.ammo[0] = slot(1073, 1);
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(app.world.tile(10, 20).unwrap().paint(), 1, "boosted reach");
    }

    /// The Builder buff's tileSpeed (+0.25 → ×0.8) scales the action cadence:
    /// `(int)(10 * 0.8) = 8` ticks (`Player.cs:4421-4430`).
    #[test]
    fn w4_g79_the_builder_speed_scales_the_paint_cadence() {
        let mut app = paint_app(1071);
        app.player.as_mut().unwrap().tile_speed_bonus = 0.25;
        app.equip.ammo[0] = slot(1073, 1);
        assert!(use_paint_tool(&mut app, 10, 20, true, false));
        assert_eq!(ticks_of(app.item_time), 8, "the ×0.8 tileSpeed cadence");
    }

    /// The `apply_edit` replay arms (kinds 4..7) converge a remote paint:
    /// the `MessageBuffer.cs:2909-2948` colour/coating split.
    #[test]
    fn w4_g79_apply_edit_replays_the_paint_broadcasts() {
        let mut w = object_world();
        let i = w.index(10, 20);
        w.tiles[i].wall = 1;
        super::super::apply_edit(&mut w, 10, 20, 4, 1);
        assert_eq!(w.tile(10, 20).unwrap().paint(), 1, "kind 4 tile paint");
        super::super::apply_edit(&mut w, 10, 20, 5, 2);
        assert_eq!(w.wall_paint(10, 20), 2, "kind 5 wall paint");
        super::super::apply_edit(&mut w, 10, 20, 6, 1);
        assert!(w.tile(10, 20).unwrap().fullbright_block(), "kind 6 Glow");
        super::super::apply_edit(&mut w, 10, 20, 7, 2);
        assert!(w.invisible_wall(10, 20), "kind 7 Echo wall");
    }
}
