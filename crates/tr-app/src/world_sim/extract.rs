//! The Extractinator interaction — the app half of `Player.PlaceThing_ItemInExtractinator`
//! (`Player.cs:41665-41697`) and `ExtractinatorUse`/`DropItemFromExtractinator`
//! (`:42072-42089`).
//!
//! Vanilla runs this inside `PlaceThing` (the held-item swing, `:39580-39598`),
//! not on a click event: it re-fires on every action tick (`ItemTimeIsZero &&
//! itemAnimation > 0 && controlUseItem`, `:41669`), exactly like the tile/wall
//! placement gate (`PlaceThing_Tiles` `:39743`). The caller therefore routes it
//! through the same `starting || inside_swing` cadence the place route uses
//! (`editing.rs`), one extraction per `useTime` — and the tile-kind dispatch
//! itself is the core's `interact_at` arm (`Interaction::Extractinator`), which
//! answers "this tile is an extractinator" the way the C# chain tests the kind
//! literals.

use crate::app::{rand01, App};
use crate::registry;
use crate::world_sim::items;
use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::extract;
use tr_core::tileobject::{self, Interaction};

/// One extraction attempt at `(tx, ty)` with the cursor at `world_pos`.
///
/// Returns the swing length in seconds when the action fired (`Player.ApplyItemTime`,
/// `Player.cs:4416-4433` — the extractinator's swing IS its action time, the
/// `SetItemTime` shape at `:4406-4413`, scaled ×0.33 on the chlorophyte
/// extractinator `:41673-41677`), or `None` when this tick extracts nothing
/// (the caller then falls through to the place route unchanged).
pub(crate) fn use_extractinator(app: &mut App, tx: i32, ty: i32, world_pos: Vec2) -> Option<f32> {
    let held = app.inventory[app.selected];
    if held.count == 0 {
        return None;
    }
    // The tile gate: `Main.tile[myX, myY].active()` + the kind test
    // (`Player.cs:41669`) — through the core dispatch, so the registry owns
    // which tiles are extractinators (`TileObjectData.cs:4042-4045`).
    let Some(tile) = app.world.tile(tx, ty).copied() else {
        return None;
    };
    if !tile.active()
        || tileobject::def_for_tile(tile.kind).is_none_or(|d| d.interaction != Interaction::Extractinator)
    {
        return None;
    }
    // The reach gate (`Player.cs:41669`): `IsInTileInteractionRange(tx, ty,
    // TileReachCheckSettings.Simple, item.tileBoost + blockRange)`. Every
    // extractinator input has `tileBoost = 0` (the C# bodies carry no
    // `tileBoost`), and the port has no `blockRange` (the builder-potion buff
    // 107 / gizmo-pack 407) — so the plain Simple region is the gate.
    if !in_simple_reach(app, tx, ty) {
        return None;
    }
    let is_chlorophyte = tile.kind == 642;
    // The trader arm first (`Player.cs:41678-41690`, `:41699-41707`): tile 642
    // only, the first matching 1-for-1 option of `ItemTrader`
    // (`ItemTrader.cs:65-80`).
    if is_chlorophyte {
        if let Some((giving, taking)) = extract::chlorophyte_trade(held.kind, held.count) {
            let mut secs = apply_item_time_secs(app, true);
            app.audio
                .play_at("Grab", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
            // `context.SkipItemConsumption = true` (`:41683`) + the trader's
            // own decrement (`:41684-41688`) — the standard consumable path is
            // skipped so it cannot double-spend.
            let slot = &mut app.inventory[app.selected];
            slot.count = slot.count.saturating_sub(taking);
            if slot.count == 0 {
                slot.kind = 0;
            }
            items::spawn_item_drop_px(app, giving, 1, world_pos);
            let _ = &mut secs;
            return Some(secs);
        }
    }
    // The roll arm (`Player.cs:41691-41696`).
    let mode = extract::extractinator_mode(held.kind);
    if mode < 0 {
        return None;
    }
    // The roll spends the port's `Main.rand` seam (`app.item_rng`) through the
    // same LootRng wrapper the coin spawns use, so the stream order is the C#
    // `Main.rand` order (`ExtractinatorHelper.cs:5-525` — the chain order is
    // the roll order).
    let (item_type, stack) = {
        let mut rng_state = app.item_rng;
        let mut rng = super::npcs::LootRng(&mut rng_state);
        let out = extract::roll_extractinator_drop(mode, tile.kind, app.world.hard_mode, &mut rng);
        app.item_rng = rng_state;
        out
    };
    // `ExtractinatorUse`'s `itemType > 0` gate (`Player.cs:42075`).
    if item_type <= 0 {
        return None;
    }
    let secs = apply_item_time_secs(app, is_chlorophyte);
    app.audio
        .play_at("Grab", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
    // The held stack decrements one per action (the C#'s generic consumable
    // path, `Player.cs:43280`): the silt/slush/fossil inputs are consumable in
    // vanilla and the port's `itemdata` carries that flag for all of them but
    // the fossil pair (3347/3348 — the 1.4.4-era thrower data, a documented
    // port bug); the arm's own decrement makes the behaviour right regardless,
    // exactly like the C# never checking weapon-ness here.
    let slot = &mut app.inventory[app.selected];
    slot.count = slot.count.saturating_sub(1);
    if slot.count == 0 {
        slot.kind = 0;
    }
    // `DropItemFromExtractinator` (`Player.cs:42081-42089`): the drop spawns at
    // the cursor's world position (the smart-cursor/gamepad player-centre
    // branch has no port mechanic yet — ANALYSIS). The velocity/prefix draws
    // come after the roll, like `Item.NewItem`.
    items::spawn_item_drop_px(app, item_type as u16, stack.max(1) as u32, world_pos);
    Some(secs)
}

/// `Player.ApplyItemTime(Item, float)` (`Player.cs:4421-4433`): `val =
/// (int)(useTime * multiplier)`, floored at 1 when `useTime > 0`
/// (`:4424-4426`), and `SetItemTime` then makes the swing and the item time the
/// same value (`:4406-4413`). `is_chlorophyte` is the ×0.33 multiplier of
/// `:41673-41677` (10 → 3 frames for the standard 10-frame inputs).
fn apply_item_time_secs(app: &App, is_chlorophyte: bool) -> f32 {
    let held = app.inventory[app.selected];
    let use_time = registry::get(held.kind).map_or(0, |d| d.use_time);
    let multiplier = if is_chlorophyte { 0.33f32 } else { 1.0 };
    let frames = if use_time > 0 {
        ((use_time as f32) * multiplier) as i32
    } else {
        0
    };
    let frames = if use_time > 0 { frames.max(1) } else { frames };
    frames as f32 / 60.0
}

/// `Player.IsInTileInteractionRange` over `TileReachCheckSettings.Simple`
/// (`Player.cs:32025-32030`, `TileReachCheckSettings.cs:45-54` — ranges 5/3,
/// `GetTileRegion`'s `:44-55` math, `Collision.InTileBounds` inclusive).
/// `tileBoost + blockRange` is 0 for every extractinator input.
fn in_simple_reach(app: &App, tx: i32, ty: i32) -> bool {
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let lx = (p.x / TILE_PX) as i32 - 5;
    let hx = ((p.x + crate::player::WIDTH) / TILE_PX).ceil() as i32 - 1 + 5;
    let ly = (p.y / TILE_PX) as i32 - 3;
    let hy = ((p.y + p.height) / TILE_PX).ceil() as i32 - 1 + 3;
    (lx..=hx).contains(&tx) && (ly..=hy).contains(&ty)
}

/// `Main.rand` scatter helper kept local to the tests.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::edit;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A world with a stone floor at y = 20 and a placed 3x3 extractinator at
    /// origin (9, 17) (tile 219 or 642 by `kind`), the player standing beside
    /// it with a silt stack in hand. Returns the app and the object origin.
    fn extract_world(kind: u16) -> (crate::app::App, i32, i32) {
        let mut w = tr_core::World::new("x".into(), 40, 40);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
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
        let def = tileobject::def_for_tile(kind).unwrap();
        let inst = tileobject::can_place(&w, 10, 19, def, 0, 1, None).unwrap();
        assert!(tileobject::place(&mut w, &inst));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 9.0 * TILE_PX;
        player.y = 17.0 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(9.0 * TILE_PX, 17.0 * TILE_PX);
        app.mouse_pos = Vec2::new(640.0, 360.0); // screen centre == the camera
        // Silt block (424) in the selected slot.
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 424,
            name: "Silt Block",
            count: 5,
            prefix: 0,
        };
        (app, 9, 17)
    }

    #[test]
    fn silt_at_219_extracts_and_consumes_one() {
        let (mut app, tx, ty) = extract_world(219);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        let before = app.items.len();
        let secs = use_extractinator(&mut app, tx, ty, pos).expect("silt extracts");
        // Silt's useTime is 10 frames (`Item.cs:6319-6345`) — the swing is the
        // action time, no multiplier on tile 219.
        assert!((secs - 10.0 / 60.0).abs() < 1e-6);
        assert_eq!(app.inventory[app.selected].count, 4, "one silt consumed");
        assert_eq!(app.items.len(), before + 1, "one drop spawned");
        // The drop is one of mode 0's possible outputs — the coin table is the
        // first branch of the chain (`ExtractinatorHelper.cs:88`), so most
        // seeds land on a coin, but the branch is the app's seeded stream.
        let drop = app.items.last().unwrap();
        assert!(
            matches!(
                drop.kind,
                71..=74 | 177..=182 | 999 | 11..=14 | 699..=702 | 1242
            ),
            "mode-0 output {}",
            drop.kind
        );
    }

    #[test]
    fn chlorophyte_slows_the_action_to_a_third() {
        let (mut app, tx, ty) = extract_world(642);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        app.inventory[app.selected].kind = 4354; // Lava Moss — mode 3.
        let secs = use_extractinator(&mut app, tx, ty, pos).expect("moss extracts");
        // `ApplyItemTime(item, 0.33)` on a 10-frame item → 3 frames
        // (`Player.cs:4421-4428`).
        assert!((secs - 3.0 / 60.0).abs() < 1e-6);
    }

    #[test]
    fn moss_trades_one_for_one_on_642() {
        let (mut app, tx, ty) = extract_world(642);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        app.inventory[app.selected].kind = 12; // Copper Ore → Tin Ore.
        app.inventory[app.selected].count = 3;
        let secs = use_extractinator(&mut app, tx, ty, pos).expect("trader fires");
        assert_eq!(app.items.last().unwrap().kind, 699);
        assert_eq!(app.inventory[app.selected].count, 2);
        assert!((secs - 3.0 / 60.0).abs() < 1e-6);
    }

    #[test]
    fn moss_rolls_on_219_not_trades() {
        let (mut app, tx, ty) = extract_world(219);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        // The trader table is 642-only (`TryGettingItemTraderFromBlock`,
        // `Player.cs:41699-41707`): copper ore is not an extractinator input at
        // all, so nothing fires on 219.
        app.inventory[app.selected].kind = 12; // Copper Ore — trader-only input.
        app.inventory[app.selected].count = 3;
        assert!(use_extractinator(&mut app, tx, ty, pos).is_none());
        assert_eq!(app.inventory[app.selected].count, 3);
        assert!(app.items.is_empty());
    }

    #[test]
    fn non_extractable_item_falls_through() {
        let (mut app, tx, ty) = extract_world(219);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        app.inventory[app.selected].kind = 2; // Dirt Block.
        assert!(use_extractinator(&mut app, tx, ty, pos).is_none());
        assert_eq!(app.inventory[app.selected].count, 5);
    }

    #[test]
    fn out_of_reach_extracts_nothing() {
        let (mut app, _, _) = extract_world(219);
        if let Some(p) = app.player.as_mut() {
            p.x = 1.0 * TILE_PX;
            p.y = 1.0 * TILE_PX;
        }
        let pos = Vec2::new(9.0 * TILE_PX, 17.0 * TILE_PX);
        assert!(use_extractinator(&mut app, 9, 17, pos).is_none());
        assert_eq!(app.inventory[app.selected].count, 5);
        assert!(app.items.is_empty());
    }

    #[test]
    fn fossil_extracts_despite_the_thrower_item_data_bug() {
        // The port's itemdata has 3347 as a melee thrower (1.4.4-era data);
        // the arm's own decrement keeps the extraction correct anyway.
        let (mut app, tx, ty) = extract_world(219);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        app.inventory[app.selected].kind = 3347;
        app.inventory[app.selected].count = 2;
        assert!(use_extractinator(&mut app, tx, ty, pos).is_some());
        assert_eq!(app.inventory[app.selected].count, 1);
        // The last silt slot airs out at zero.
        app.inventory[app.selected].count = 1;
        assert!(use_extractinator(&mut app, tx, ty, pos).is_some());
        assert_eq!(app.inventory[app.selected].kind, 0);
    }

    #[test]
    fn dispatch_via_interact_at_answers_extractinator() {
        let (app, tx, ty) = extract_world(642);
        let blocked = |_: i32, _: i32| false;
        let mut world = app.world;
        assert!(matches!(
            edit::interact_at(&mut world, tx, ty, 1, &blocked),
            edit::TileInteraction::Extractinator
        ));
    }

    #[test]
    fn two_runs_roll_identically() {
        // Determinism: the same seed + the same inputs roll the same output.
        let (mut a, tx, ty) = extract_world(219);
        let (mut b, _, _) = extract_world(219);
        let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        use_extractinator(&mut a, tx, ty, pos);
        use_extractinator(&mut b, tx, ty, pos);
        assert_eq!(a.items.last().unwrap().kind, b.items.last().unwrap().kind);
        assert_eq!(a.items.last().unwrap().count, b.items.last().unwrap().count);
    }
}
