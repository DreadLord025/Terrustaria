//! The app-side smart-cursor seam — the `SmartCursorHelper.SmartCursorLookup`
//! call site (`Player.cs:25446`): the once-per-frame re-point of the tile
//! target before the mining/placement arms run.
//!
//! The vanilla lookup runs inside `Player.Update`'s item check with the
//! player's held item and the `TileReachCheckSettings.Simple` region; the
//! port builds the same inputs from the `App` state, runs the core module's
//! chain, and hands the re-pointed cell back to the editing path. The
//! module's continuity locks live on `App` (`SmartCursorHelper`'s two
//! static `Point?` fields) and are reset by the lookup itself.

use crate::app::App;
use tr_core::smart_cursor::{self, Lookup};

/// The smart cursor's gate — the vanilla runtime state
/// (`Main.SmartCursorIsUsed` → `Main.SmartCursorWanted_Mouse`,
/// `Main.cs:2011`/`:2889`): the press-edge flag flipped by the left-Shift
/// toggle (`Player.TryToToggleSmartCursor`, `Player.cs:28814-28820`), with
/// the `TR_SMARTCURSOR=0` pin as the debug kill switch.
///
/// The persisted `settings.smart_cursor_toggle` is the toggle-mode flag
/// (`Main.cSmartCursorModeIsToggleAndNotHold`, `Main.cs:67178`) and is
/// deliberately **not** read here: vanilla's wanted state defaults off
/// (`Main.cs:67179`) and is runtime-only.
pub(crate) fn enabled(app: &App) -> bool {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let off = OFF.get_or_init(|| {
        std::env::var("TR_SMARTCURSOR")
            .map(|v| v == "0" || v.eq_ignore_ascii_case("off") || v.eq_ignore_ascii_case("false"))
            .unwrap_or(false)
    });
    app.smart_cursor_used && !*off
}

/// The held item's smart-cursor port fields — the C# reads
/// `Item.type/createTile/createWall/placeStyle/pick/axe/hammer` off the
/// held item.
fn item_port(app: &App) -> smart_cursor::ItemPort {
    let held = &app.inventory[app.selected];
    let (place_style, pick, axe, hammer) = match crate::registry::get(held.kind) {
        Some(d) => (d.place_style, d.pick as i32, d.axe as i32, d.hammer as i32),
        None => (0, 0, 0, 0),
    };
    smart_cursor::ItemPort {
        kind: held.kind,
        create_tile: crate::registry::place_tile(held.kind).map_or(-1, |t| t as i32),
        create_wall: crate::registry::place_wall(held.kind).map_or(-1, |t| t as i32),
        place_style,
        pick,
        axe,
        hammer,
    }
}

/// `SmartCursorHelper.cs:94-101` — the reach boost: the held item's
/// `tileBoost` plus the builder `blockRange` for the placing items. The
/// port carries neither (both are potion/gizmo state), so the boost stays
/// 0.
fn tile_boost(app: &App) -> i32 {
    let _ = app; // tileBoost/blockRange unported — 0 either way
    0
}

/// `Player.cs:25446` — the lookup re-points the tile target; returns the
/// re-pointed `(x, y)` or `None` (nothing better was found). The
/// `screen_target` is the un-re-pointed cursor tile and `mouse` the mouse
/// world position.
pub(crate) fn repoint(app: &mut App, tx: i32, ty: i32, mouse: (f32, f32)) -> Option<(i32, i32)> {
    if !enabled(app) {
        return None;
    }
    let Some(player) = app.player.as_ref() else {
        return None;
    };
    let (start_x, start_y, end_x, end_y) = smart_cursor::reachable_rect(
        (player.x, player.y),
        crate::player::WIDTH as i32,
        player.height as i32,
        tile_boost(app),
    );
    // `SmartCursorHelper.cs:106-116` — the `_grappleTargets` fill: the
    // player's grappling projectiles' centers over 16. The hooks live in the
    // pool as aiStyle-7 slots (`AI_007_GrapplingHooks`), so the scan is the
    // same-owner walk over them; the slot's `pos` is its top-left, hence the
    // half-box offset to its center.
    let local = app.local_id as u8;
    let grapple_targets = smart_cursor::grapple_target_cells(
        &app.projectiles
            .slots()
            .iter()
            .filter(|pr| {
                pr.active
                    && pr.owner == local
                    && crate::projectile::proj_data(pr.kind).is_some_and(|d| d.ai_style == 7)
            })
            .map(|pr| (pr.grapple_center.x, pr.grapple_center.y))
            .collect::<Vec<_>>(),
    );
    let lookup = Lookup {
        grapple_targets,
        item: item_port(app),
        mouse,
        screen_target: (tx, ty),
        start_x,
        start_y,
        end_x,
        end_y,
        weather: tr_core::growth::WeatherCtx {
            raining: app.rain.is_raining(),
            cloud_alpha: app.rain.visual_alpha(),
            // `Main.remixWorld` — no port state (the M5-SEED-2 arm).
            remix_world: false,
        },
        paint_lookup: 0,
        paint_coating_lookup: 0,
        // `TryFindingPaintInplayerInventory` (`SmartCursorHelper.cs:177-191`)
        // — the inventory scan is app-side and unported: the paint steps
        // stand down until it lands.
        smart_blocks_enabled: app.settings.smart_blocks,
        // `Player.SmartCursorSettings.SmartAxeAfterPickaxe` (`Player.cs:333`)
        // — the port's settings do not carry the toggle yet; the default
        // (false) runs the axe step before the pickaxe family.
        smart_axe_after_pickaxe: false,
        // `Player.controlUseItem` — the continuity lock's held-button
        // condition.
        use_item_held: app.place_down || app.mine_down,
        tool_mode: 0,
        actuation_rod_lock: false,
        actuation_rod_lock_setting: false,
        using_gamepad: false,
        gamepad_idle: false,
        // `Player.gravDir` — the port has no gravity flip (1.0).
        grav_dir: 1.0,
        // `Player.direction` — the port's player has no facing field yet;
        // the gamepad override (the only reader) is off.
        direction: 1,
        // `Main.netMode == 1` — the port is singleplayer.
        is_mp_client: false,
        mp_miners: Default::default(),
        player_position: (player.x, player.y),
        player_width: crate::player::WIDTH as i32,
        player_height: player.height as i32,
    };
    smart_cursor::smart_cursor_lookup(&app.world, &lookup, -1, -1, &mut app.smart_cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemSlot;
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A big world with a stone floor at y = 40, the player standing on it
    /// at tile (35, 40) and a torch in hand; the mouse at the tile below
    /// the cursor. The smart cursor is in use — the vanilla default is off
    /// (`Main.cs:67179`), so the lookup wiring is exercised with the runtime
    /// flag on (`gates_the_lookup` covers the off half).
    fn wiring_app() -> crate::app::App {
        let mut w = tr_core::World::new("w".into(), 120, 120);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for x in 0..120 {
            let i = w.index(x, 40);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 35.0 * tr_core::TILE_PX;
        player.y = 40.0 * tr_core::TILE_PX;
        app.player = Some(player);
        app.smart_cursor_used = true;
        // The torch (item 8) in the selected slot.
        app.inventory[app.selected] = ItemSlot {
            kind: 8,
            name: "Torch",
            count: 1,
            prefix: 0,
        };
        app
    }

    #[test]
    fn the_lookup_carries_the_grapple_hook_cell() {
        // `SmartCursorHelper.cs:106-116` — the reach rect's off-screen gate
        // fills `_grappleTargets` from the player's grappling projectiles.
        // The hooks are aiStyle-7 pool slots owned by the local player, so the
        // test spawns one through `spawn_hook`; the torch step still
        // re-points (the targets only gate the pickaxe-solids walk).
        let mut app = wiring_app();
        app.projectiles.spawn_hook(
            crate::projectile::HOOK,
            glam::Vec2::new(40.0 * 16.0, 42.0 * 16.0),
            glam::Vec2::new(41.0 * 16.0, 42.0 * 16.0),
            16.0,
            0,
            0.0,
            app.local_id as u8,
            1,
        );
        let found = repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0));
        assert_eq!(found, Some((35, 39)), "the torch step still re-points");
    }

    #[test]
    fn the_torch_lookup_repoints_above_the_floor() {
        // `Player.cs:25446` — the held torch's step re-points the cursor
        // from the floor cell (35, 41) to the empty cell above it (35, 39).
        let mut app = wiring_app();
        let found = repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0));
        assert_eq!(
            found,
            Some((35, 39)),
            "the torch step re-points at the empty cell above the floor"
        );
        // A non-torch item cannot re-point (the pickaxe has no ported step
        // whose gate it passes).
        let mut axe_app = wiring_app();
        axe_app.inventory[axe_app.selected] = ItemSlot {
            kind: 1,
            name: "Iron Pickaxe",
            count: 1,
            prefix: 0,
        };
        assert_eq!(
            repoint(&mut axe_app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0)),
            None,
            "the pickaxe has no ported step"
        );
    }

    #[test]
    fn the_runtime_flag_gates_the_lookup() {
        // `Main.SmartCursorIsUsed` (`Main.cs:2889`) reads the runtime
        // `SmartCursorWanted_Mouse`, whose vanilla default is **off**
        // (`Main.cs:67179`) — a fresh app does not re-point.
        let mut app = wiring_app();
        app.smart_cursor_used = false;
        assert_eq!(
            repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0)),
            None,
            "the runtime state off: no re-point"
        );
        // The press edge turns it on and the lookup runs. `Player.controlSmart`
        // is `Main.cSmart` — **LeftControl** (`Main.cs:67175`), not `cTorch`'s
        // LeftShift (`:67173`).
        app.world_key(winit::keyboard::KeyCode::ControlLeft);
        assert!(app.smart_cursor_used, "the Control press turns it on");
        assert_eq!(
            repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0)),
            Some((35, 39)),
            "the runtime state on: the torch step re-points"
        );
        // The second press turns it back off and the lookup stands down.
        app.world_key(winit::keyboard::KeyCode::ControlLeft);
        assert!(!app.smart_cursor_used, "the second press turns it off");
        assert_eq!(
            repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0)),
            None,
            "the runtime state off: no re-point"
        );
    }

    #[test]
    fn the_persisted_toggle_is_not_the_runtime_state() {
        // `settings.smart_cursor_toggle` is the toggle-mode flag
        // (`Main.cSmartCursorModeIsToggleAndNotHold`, `Main.cs:67178`), not
        // the wanted state: it stays a saved key that no longer gates the
        // lookup either way.
        let mut app = wiring_app();
        app.settings.smart_cursor_toggle = false;
        assert_eq!(
            repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0)),
            Some((35, 39)),
            "the setting off does not disable the runtime smart cursor"
        );
        let mut app = wiring_app();
        app.settings.smart_cursor_toggle = true;
        app.smart_cursor_used = false;
        assert_eq!(
            repoint(&mut app, 35, 41, (35.0 * 16.0 + 8.0, 41.0 * 16.0 + 8.0)),
            None,
            "the setting on does not enable the runtime smart cursor"
        );
    }
}
