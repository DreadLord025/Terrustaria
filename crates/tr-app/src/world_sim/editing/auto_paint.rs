//! The autoPaint build pass — the Paint Sprayer (2216) / Architect Gizmo
//! Pack (3061) / Hand Of Creation (5126) auto-paint every newly placed tile
//! (and wall). The C# pair: the paint half of
//! `Player.PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`
//! (`Player.cs:40632-40635`, the SAME 11×11 window + skip rules as the
//! actuator half — `auto_actuator.rs`) + `Player.TryPainting`
//! (`:41946-41960`) → `ApplyCoating` (`:41962-41992`) / `ApplyPaint`
//! (`:41994-42024`), with `FindPaintOrCoating` (`:41927-41944`)'s
//! ammo-rows-first scan. The flag is granted by `grant_mechanical_accs`
//! (`world_sim/player.rs`, `Player.cs:12813-12816` + the held-item arms
//! `:15271-15294`).
//!
//! The paint/coating world writes are `tr_core::edit::paint_*`
//! (`WorldGen.cs:44110-44252`); the dust effects (`paintEffect`
//! `:44064-44085`, `paintCoatEffect` `:44087-44108`) live here — 10 dusts
//! of type 143 per apply, the `genRand.Next(2)` scale/gravity arm on the
//! world's framing stream (the port's `genRand` stand-in, the
//! `reframe_area` convention), the pool's own rolls on `UpdatePlayers`
//! (placement runs under the `Main.cs:17754` swap).

use tr_core::edit;
use tr_core::particles::Color;
use tr_core::{TILE_PX, World};

use crate::app::App;
use crate::item::EMPTY_SLOT;

/// One inventory cell `FindPaintOrCoating` may return — the vanilla slot
/// index (the ammo rows 54..57 scan FIRST, `Player.cs:41929-41935`) plus
/// the item's paint/coating bytes (`Item.PaintOrCoating`, `Item.cs:353-363`:
/// paint > 0 OR paintCoating > 0).
#[derive(Clone, Copy)]
pub(super) struct PaintSource {
    /// The slot in `App::inventory` (grid) or `App::equip.ammo` (the four
    /// ammo rows the C# keeps in the same 58-slot array).
    pub ammo_row: Option<usize>,
    pub grid_slot: Option<usize>,
    pub paint: u8,
    pub coating: u8,
}

/// `Player.FindPaintOrCoating` (`Player.cs:41927-41944`): the ammo rows
/// 54..57 first, then the 0..57 walk (coins 50..53 never hold paint; the
/// grid covers 0..49). The FIRST slot with a positive stack whose item is
/// paint or coating wins.
pub(super) fn find_paint_or_coating(app: &App) -> Option<PaintSource> {
    let is_paint = |kind: u16| {
        crate::registry::get(kind)
            .filter(|d| d.paint > 0 || d.paint_coating > 0)
            .map(|d| (d.paint as u8, d.paint_coating as u8))
    };
    for (k, slot) in app.equip.ammo.iter().enumerate() {
        if slot.count > 0 {
            if let Some((paint, coating)) = is_paint(slot.kind) {
                return Some(PaintSource {
                    ammo_row: Some(k),
                    grid_slot: None,
                    paint,
                    coating,
                });
            }
        }
    }
    for (i, slot) in app.inventory.iter().enumerate() {
        if slot.count > 0 {
            if let Some((paint, coating)) = is_paint(slot.kind) {
                return Some(PaintSource {
                    ammo_row: None,
                    grid_slot: Some(i),
                    paint,
                    coating,
                });
            }
        }
    }
    None
}

/// Burn one item from the found source (`ApplyCoating`/`ApplyPaint`'s
/// `targetItem.stack--; if (stack <= 0) SetDefaults(0)`,
/// `Player.cs:41969-41973`/:42014-42018). The C# holds ONE `Item` reference
/// across both halves — an emptied slot's `SetDefaults(0)` zeroes its
/// paint/coating bytes, so the second half sees nothing; the snapshot
/// zeroes with it. (No stock item carries BOTH bytes — paints 1073-1099/
/// 1966-1968, coatings 4668/5344 — the arm is unreachable in practice.)
fn burn_one(app: &mut App, src: &mut PaintSource) {
    let slot = match (src.ammo_row, src.grid_slot) {
        (Some(k), _) => &mut app.equip.ammo[k],
        (_, Some(i)) => &mut app.inventory[i],
        _ => return,
    };
    slot.count = slot.count.saturating_sub(1);
    if slot.count == 0 {
        *slot = EMPTY_SLOT;
        src.paint = 0;
        src.coating = 0;
    }
}

/// `WorldGen.paintEffect` (`WorldGen.cs:44064-44085`): 10 dusts of type
/// 143 in the paint's colour (a 0 colour re-uses the OLD colour, `:44067-
/// 44070`); each `genRand.Next(2) == 0` → noGravity + scale ×1.2, else
/// scale ×0.5. `genRand` = the world's framing stream (the port's stand-in
/// — the reframe rolls' convention); the pool's rolls ride the caller's
/// `UpdatePlayers` stream.
fn paint_effect(app: &mut App, x: i32, y: i32, color: u8, old_color: u8) {
    let color2 = if color == 0 { old_color } else { color };
    let [r, g, b] = tr_core::tile_light::paint_color_bytes(color2);
    let new_color = Color::new(r, g, b, 255);
    let ctx = app.spawn_ctx();
    for _ in 0..10 {
        let idx = app.dust.new_dust(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            &ctx,
            &app.world,
            glam::Vec2::new(x as f32 * TILE_PX, y as f32 * TILE_PX),
            16,
            16,
            143,
            0.0,
            0.0,
            50,
            new_color,
            1.0,
        );
        if app.world.frame_rng.next_max(2) == 0 {
            let d = app.dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale *= 1.2;
        } else {
            app.dust.slot_mut(idx).scale *= 0.5;
        }
    }
}

/// `WorldGen.coatingColor` (`WorldGen.cs:44036-44044`) +
/// `coatingColors(tile, block)` (`:44046-44062`): the Glow/Echo colours of
/// the tile's CURRENT coating bits, in the C#'s add order (Glow first).
fn coating_colors(world: &World, x: i32, y: i32, block: bool) -> Vec<Color> {
    let mut out = Vec::new();
    let (glow, echo) = if block {
        (
            world.tile(x, y).is_some_and(|t| t.fullbright_block()),
            world.invisible_block(x, y),
        )
    } else {
        (world.fullbright_wall(x, y), world.invisible_wall(x, y))
    };
    if glow {
        out.push(Color::new(235, 170, 255, 255));
    }
    if echo {
        out.push(Color::new(180, 245, 255, 255));
    }
    out
}

/// `WorldGen.paintCoatEffect` (`WorldGen.cs:44087-44108`): 10 dusts of
/// type 143 in the coating's colour; a scraper (0) picks each dust's
/// colour from the OLD coating colours on `Main.rand`
/// (`oldColors[Main.rand.Next(oldColors.Count)]`, `:44095` — an empty list
/// keeps `coatingColor(0)` = Transparent). The `genRand.Next(2)` arm
/// matches `paint_effect`.
fn paint_coat_effect(app: &mut App, x: i32, y: i32, id: u8, old_colors: &[Color]) {
    let base = match id {
        1 => Color::new(235, 170, 255, 255),
        2 => Color::new(180, 245, 255, 255),
        _ => Color::TRANSPARENT,
    };
    let ctx = app.spawn_ctx();
    for _ in 0..10 {
        let mut new_color = base;
        if id == 0 && !old_colors.is_empty() {
            let pick = app
                .rngs
                .stream(crate::app::RngKey::UpdatePlayers)
                .next_max(old_colors.len() as i32) as usize;
            new_color = old_colors[pick];
        }
        let idx = app.dust.new_dust(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            &ctx,
            &app.world,
            glam::Vec2::new(x as f32 * TILE_PX, y as f32 * TILE_PX),
            16,
            16,
            143,
            0.0,
            0.0,
            50,
            new_color,
            1.0,
        );
        if app.world.frame_rng.next_max(2) == 0 {
            let d = app.dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale *= 1.2;
        } else {
            app.dust.slot_mut(idx).scale *= 0.5;
        }
    }
}

/// `Player.TryPainting(x, y, paintingAWall, applyItemAnimation: false)`
/// (`Player.cs:41946-41960`): find the paint/coating item, apply the
/// coating FIRST (`:41951-41954`), then the paint (`:41955-41958`). The
/// auto callers pass `applyItemAnimation: false`, so the
/// `ApplyItemTime(tileSpeed/wallSpeed)` arms (`:41974-41977`/:41987-41990/
/// :42006-42009/:42019-42022) never run — the placement's own cadence
/// stands.
///
/// The net 63/64 broadcasts ride `request_edit` (the tile record carries
/// paint + coating bits — the port's edit-replication convention, the
/// actuator tool arm's).
pub(super) fn try_painting(app: &mut App, x: i32, y: i32, painting_a_wall: bool) {
    let Some(mut src) = find_paint_or_coating(app) else {
        return;
    };
    // `ApplyCoating` (`:41962-41992`) — the coating half.
    if src.coating != 0 {
        let old = coating_colors(&app.world, x, y, !painting_a_wall);
        let applied = if painting_a_wall {
            edit::paint_coat_wall(&mut app.world, x, y, src.coating)
        } else {
            edit::paint_coat_tile(&mut app.world, x, y, src.coating)
        };
        if applied {
            // The net 63/64 (`WorldGen.cs:44173`/:44221) rides the tile
            // record — the port's edit-replication convention.
            app.request_edit(x, y, if painting_a_wall { 3 } else { 1 }, 0);
            paint_coat_effect(app, x, y, src.coating, &old);
            burn_one(app, &mut src);
            app.world_key = None;
        }
    }
    // `ApplyPaint` (`:41994-42024`) — the paint half. The colour-equal
    // refusal (`Main.tile[x,y].color() != paint`, `:41999`/:42012) is
    // INSIDE the core write too (paintTile's own `b == color` gate); the
    // C# checks it twice, the port once — same observable.
    if src.paint != 0 {
        let applied = if painting_a_wall {
            // The wall order: write → net → effect (`WorldGen.cs:44242-
            // 44250` — the tile twin effects BEFORE the net).
            if let Some(old) = edit::paint_wall(&mut app.world, x, y, src.paint) {
                app.request_edit(x, y, 3, 0);
                paint_effect(app, x, y, src.paint, old);
                true
            } else {
                false
            }
        } else if let Some(old) = edit::paint_tile(&mut app.world, x, y, src.paint) {
            paint_effect(app, x, y, src.paint, old);
            app.request_edit(x, y, 1, 0);
            true
        } else {
            false
        };
        if applied {
            burn_one(app, &mut src);
            app.world_key = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{object_world, slot};
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    /// A player with the autoPaint flag; `paints` Red Paint (1073) in the
    /// given grid slot, the held item a Stone Block stack.
    fn painter_app(paints: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let mut p = crate::player::Player::spawn(&app.world);
        p.auto_paint = true;
        app.player = Some(p);
        app.inventory[app.selected] = slot(3, 10);
        if paints > 0 {
            app.inventory[7] = slot(1073, paints);
        }
        app
    }

    /// The build pass paints the freshly placed block and burns one paint
    /// (`Player.cs:40632-40635` → `ApplyPaint` `:42012-42018`), through the
    /// real `place_tile` route.
    #[test]
    fn the_auto_paint_pass_paints_the_placed_block_and_burns_one() {
        let mut app = painter_app(3);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.world.tile(10, 19).unwrap().paint(), 1, "Red Paint");
        assert_eq!(app.inventory[7].count, 2, "one paint burned");
        assert_eq!(app.inventory[app.selected].count, 9, "the block burned too");
    }

    /// The AutoPaint builder toggle off (`builderAccStatus[3] != 0`,
    /// `:40632`) skips the paint half.
    #[test]
    fn the_toggle_off_skips_the_paint() {
        let mut app = painter_app(3);
        app.player.as_mut().unwrap().builder_acc_status[3] = 1;
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.world.tile(10, 19).unwrap().paint(), 0);
        assert_eq!(app.inventory[7].count, 3, "nothing burned");
    }

    /// No paint in the inventory → the block places unpainted
    /// (`FindPaintOrCoating() == null`, `:41948-41949`).
    #[test]
    fn the_empty_inventory_places_unpainted() {
        let mut app = painter_app(0);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.world.tile(10, 19).unwrap().paint(), 0);
    }

    /// The last paint empties the slot (`SetDefaults(0)`, `:42015-42018`).
    #[test]
    fn the_last_paint_empties_the_slot() {
        let mut app = painter_app(1);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.inventory[7], EMPTY_SLOT);
    }

    /// `FindPaintOrCoating` scans the ammo rows FIRST
    /// (`Player.cs:41929-41935`): a paint in ammo row 0 beats the grid copy.
    #[test]
    fn the_ammo_rows_scan_first() {
        let mut app = painter_app(3);
        app.equip.ammo[0] = slot(1081, 2); // Blue Paint (paint 9)
        let src = find_paint_or_coating(&app).unwrap();
        assert_eq!(src.ammo_row, Some(0));
        assert_eq!(src.paint, 9);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.world.tile(10, 19).unwrap().paint(), 9, "the ammo paint");
        assert_eq!(app.equip.ammo[0].count, 1);
        assert_eq!(app.inventory[7].count, 3, "the grid copy untouched");
    }

    /// The unchanged neighbour is skipped by the SAME window rules as the
    /// actuator half (`:40628`'s first disjunct) — no paint, no burn.
    #[test]
    fn the_unchanged_neighbour_is_skipped() {
        let mut app = painter_app(3);
        let i = app.world.index(11, 19);
        app.world.tiles[i].kind = 1;
        app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.world.tile(11, 19).unwrap().paint(), 0);
        assert_eq!(app.inventory[7].count, 2, "only the target burned one");
    }

    /// A coating item (Glow Paint 4668) sets the fullbright bit instead of
    /// a colour (`ApplyCoating`, `:41951-41954` → `paintCoatTile` case 1).
    #[test]
    fn the_glow_coating_sets_the_fullbright_bit() {
        let mut app = painter_app(0);
        app.inventory[7] = slot(4668, 2);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        let t = app.world.tile(10, 19).unwrap();
        assert!(t.fullbright_block(), "the Glow bit");
        assert_eq!(t.paint(), 0, "no colour");
        assert_eq!(app.inventory[7].count, 1);
    }

    /// The wall route paints the placed wall (`PlaceThing_Walls`,
    /// `Player.cs:39629-39632`).
    #[test]
    fn the_wall_route_paints_the_placed_wall() {
        let mut app = painter_app(3);
        app.inventory[app.selected] = slot(30, 10); // Dirt Wall
        assert!(super::super::place_wall_route(&mut app, 10, 19));
        assert_eq!(app.world.wall_paint(10, 19), 1, "the wall took Red Paint");
        assert_eq!(app.inventory[7].count, 2);
    }

    /// Both flags on: the placed block takes the paint AND the actuator
    /// (the C# runs both halves per cell, `:40632` then `:40636`).
    #[test]
    fn both_halves_apply_to_the_same_cell() {
        let mut app = painter_app(3);
        app.player.as_mut().unwrap().auto_actuator = true;
        app.inventory[5] = slot(super::super::wiring::ACTUATOR_ITEM, 3);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        let t = app.world.tile(10, 19).unwrap();
        assert_eq!(t.paint(), 1);
        assert!(t.actuator(), "the actuator bit too");
        assert_eq!(app.inventory[7].count, 2);
        assert_eq!(app.inventory[5].count, 2);
    }
}
