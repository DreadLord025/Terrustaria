//! The smart-interact tile-targeting lookup — `Player.SmartInteractLookup`
//! (`Player.cs:17374-17398`) + `SmartInteractSystem.RunQuery`
//! (`GameContent/ObjectInteractions/SmartInteractSystem.cs:31-61`) + the
//! `TileSmartInteractCandidateProvider`
//! (`TileSmartInteractCandidateProvider.cs:63-480`), the feed of
//! `Main.SmartInteractTileCoords`/`…Selected` (`Main.cs:2035-2037`) that
//! `render/tile_outlines.rs`'s highlight-area gate reads.
//!
//! W4-g86 wiring seam 1. The landed smart-cursor lane
//! (`tr_core::smart_cursor`, `SmartCursorHelper.SmartCursorLookup`) is the
//! TOOL lookup (`Main.SmartCursorX/Y` re-point); this module is the OTHER
//! half of the `Player.cs:25446-25447` pair — the interactable-object scan
//! that fills the outline lists. It runs right after the smart-cursor
//! re-point in `world_sim/editing.rs` (the port's `Player.Update` seam).
//!
//! ## Census (what feeds vs what outlines)
//!
//! `FillPotentialTargetTiles` (`:359-480`) admits: campfires (215),
//! `Recipe.TileUsedInRecipes` (the recipe stations incl. the altar 26 and
//! the `TileCountsAs`-inherited 77/133/134/302/304/355/699/17/96),
//! `CountsAsWaterForCrafting` (172/207), the 57-id literal switch
//! (`:385-444`) and the six conditional ids (chairs 15 / toilets 497 within
//! 40 px of the player's feet, LihzahrdAltar 237 with item 1293,
//! SnowballLauncher 212 with item 949, Sundial 356 / Moondial 663 off
//! cooldown). `ProvideCandidate` (`:76-357`) then sizes each target through
//! the big type switch (`:115-289`), anchors it at its top-left cell and
//! fills the point lists. Every one of the 95 `HasOutlines` ids
//! (`TileID.cs:551`) has BOTH a feed arm and a size arm — the census found
//! no orphan outline tile; the five sized-but-not-outlined ids (14 Tables,
//! 469 Tables2, 491 VoidVault, 573 TatteredWoodSign, 597 TeleportationPylon)
//! feed the interact machinery but draw no outline (491/597 have their own
//! selection glow in `render/pylons.rs`).
//!
//! ## Deliberate state (documented in ANALYSIS "The smart-interact feed")
//!
//! * The provider chain's first three providers are unported: the
//!   Potion-of-Return gate (`PotionOfReturnSmartInteractCandidateProvider`),
//!   the interactable-projectile list (`ProjectileSmartInteractCandidate
//!   Provider` over `GetListOfProjectilesToInteractWithHack`) and the
//!   town-NPC scan (`NPCSmartInteractCandidateProvider`). Only the TILE
//!   provider runs, so its candidate always wins (`RunQuery`'s
//!   min-distance vote over one candidate). The NPC/proj wins would set
//!   `SmartInteractNPC/Proj` — no outline-list effect either way; the
//!   potion gate would steal the win from a farther tile candidate.
//! * `Main.SmartInteractShowingGenuine/Fake` and `SmartInteractX/Y` are
//!   carried on [`SmartInteractState`] for the future right-click routing;
//!   the outline pass reads only the lists + the interaction rect.
//! * `UILinkPointNavigator.InUse` / gamepad hints (`Player.cs:17377`,
//!   `:17381`'s `ShowGamepadHints`) are false — no gamepad UI in the port,
//!   so `DemandOnlyZeroDistanceTargets` is false and the lookup gates on
//!   `Main.SmartCursorIsUsed` alone (the runtime `smart_cursor_used`).
//! * `Main.hideUI` has no port state (the F11 toggle is unported) — the
//!   outline gate's `hide_ui` reads false.
//! * RNG census: ZERO draws — the whole scan is integer/float maths.

use crate::app::App;
use tr_core::smart_cursor::{in_tile_bounds, reachable_rect};
use tr_core::tiledata as td;
use tr_core::World;

/// `Main.SmartInteractTileCoords`/`…Selected` + the interaction rect + the
/// genuine/fake flags (`Main.cs:2035-2043`, `ClearSmartInteract`
/// `:48054-48062`). The lists keep insertion order (the C# `List<Point>`);
/// membership is order-irrelevant, duplicates never enter (`:321`).
#[derive(Default)]
pub(crate) struct SmartInteractState {
    /// `Main.SmartInteractTileCoords` (`Main.cs:2035`).
    pub coords: Vec<(i32, i32)>,
    /// `Main.SmartInteractTileCoordsSelected` (`Main.cs:2037`).
    pub selected: Vec<(i32, i32)>,
    /// `Main.TileInteractionLX/LY/HX/HY` (`Main.cs:2039-2045`), −1 when
    /// nothing won (`ClearSelfAndPrepareForCheck` `:67-70`).
    pub lx: i32,
    pub ly: i32,
    pub hx: i32,
    pub hy: i32,
    /// `Main.SmartInteractX/Y` (`WinCandidacy` `:42-43`), −1 idle.
    pub aimed_x: i32,
    pub aimed_y: i32,
    /// `Main.SmartInteractShowingGenuine/Fake` (`:44-51`).
    pub showing_genuine: bool,
    pub showing_fake: bool,
}

impl SmartInteractState {
    /// `Main.ClearSmartInteract` (`Main.cs:48054-48062`) — the flags, the
    /// NPC/proj picks (−1, unported fields) and both lists. The interaction
    /// rect is NOT cleared here (the C# clears it in
    /// `ClearSelfAndPrepareForCheck` / `DrawCursor` only).
    pub fn clear_smart_interact(&mut self) {
        self.showing_genuine = false;
        self.showing_fake = false;
        self.coords.clear();
        self.selected.clear();
    }

    /// `TileSmartInteractCandidateProvider.ClearSelfAndPrepareForCheck`
    /// (`:63-74`): the aimed cell + the rect to −1 and both lists empty.
    fn clear_self_and_prepare(&mut self) {
        self.aimed_x = -1;
        self.aimed_y = -1;
        self.lx = -1;
        self.hx = -1;
        self.ly = -1;
        self.hy = -1;
        self.coords.clear();
        self.selected.clear();
    }
}

/// `ProvideCandidate`'s size switch (`TileSmartInteractCandidateProvider.cs:115-289`):
/// `(width, height, frameY-stride padding)` in tiles; the padding `num11`
/// defaults to 2 and the stride is `18 * height + padding` (`:300`).
/// `None` = the type has no arm (`num7 == 0 || num8 == 0` skip, `:295`).
/// The campfire override (`:290-294`) forces (3, 2) with the default 2.
pub(crate) fn interact_tile_size(kind: u16) -> Option<(i32, i32, i32)> {
    let (w, h, pad) = match kind {
        136 | 144 | 494 => (1, 1, 0),
        216 | 338 => (1, 2, 2),
        15 | 497 => (1, 2, 4),
        10 => (1, 3, 0),
        388 | 389 => (1, 5, 2),
        16 | 18 | 29 | 134 | 387 => (2, 1, 2),
        21 | 55 | 85 | 94 | 96 | 97 | 125 | 132 | 172 | 287 | 335 | 386 | 411 | 425 | 441
        | 467 | 468 | 573 | 621 | 622 => (2, 2, 2),
        79 | 139 | 510 | 511 => (2, 2, 0),
        11 | 356 | 410 | 470 | 480 | 509 | 657 | 658 | 663 | 720 | 721 | 725 => (2, 3, 0),
        207 => (2, 4, 0),
        104 => (2, 5, 2),
        88 => (3, 1, 0),
        14 | 17 | 26 | 77 | 86 | 89 | 114 | 133 | 217 | 218 | 237 | 377 | 469 => (3, 2, 2),
        106 | 220 | 228 | 243 | 247 | 283 | 300 | 301 | 302 | 303 | 304 | 305 | 306 | 307
        | 308 | 355 | 412 | 499 => (3, 3, 2),
        354 | 455 | 491 | 733 => (3, 3, 0),
        487 => (4, 2, 0),
        212 => (4, 3, 2),
        209 => (4, 3, 0),
        101 | 102 | 463 | 475 | 597 => (3, 4, 2),
        699 => (4, 4, 2),
        464 => (5, 4, 2),
        _ => return None,
    };
    // `:290-294` — the campfire set override (215, `TileID.cs:616`).
    if td::SET_CAMPFIRES.has(kind) {
        return Some((3, 2, 2));
    }
    Some((w, h, pad))
}

/// `FillPotentialTargetTiles` (`:359-480`): the row-major scan of the reach
/// rect collecting every interactable target cell. `has_1293`/`has_949`
/// are the player's `HasItem` reads (`Player.cs:55894-55904`), `snapping`
/// the `IsWithinSnappngRangeToTile(i, j, 40)` predicate
/// (`Player.cs:17365-17372`).
fn fill_potential_target_tiles(
    world: &World,
    lx: i32,
    ly: i32,
    hx: i32,
    hy: i32,
    has_1293: bool,
    has_949: bool,
    snapping: impl Fn(i32, i32) -> bool,
    fast_forward_dawn: bool,
    fast_forward_dusk: bool,
    sundial_cooldown: i32,
    moondial_cooldown: i32,
    is_mp_client: bool,
    targets: &mut Vec<(i32, i32)>,
) {
    for i in lx..=hx {
        for j in ly..=hy {
            let Some(tile) = world.tile(i, j) else { continue };
            if !tile.active() {
                continue;
            }
            let kind = tile.kind;
            if td::SET_CAMPFIRES.has(kind) {
                targets.push((i, j));
                continue;
            }
            if crate::craft::tile_used_in_recipes(kind) {
                targets.push((i, j));
                continue;
            }
            if td::SET_COUNTS_AS_WATER_FOR_CRAFTING.has(kind) {
                targets.push((i, j));
                continue;
            }
            match kind {
                10 | 11 | 21 | 29 | 55 | 79 | 85 | 88 | 89 | 97 | 102 | 104 | 125 | 132
                | 136 | 139 | 144 | 207 | 209 | 216 | 287 | 335 | 338 | 354 | 377 | 386
                | 387 | 388 | 389 | 410 | 411 | 425 | 441 | 455 | 463 | 464 | 467 | 468
                | 470 | 475 | 480 | 487 | 491 | 494 | 509 | 510 | 511 | 573 | 597 | 621
                | 657 | 658 | 699 | 720 | 721 | 725 | 733 => targets.push((i, j)),
                15 | 497 => {
                    if snapping(i, j) {
                        targets.push((i, j));
                    }
                }
                237 => {
                    if has_1293 {
                        targets.push((i, j));
                    }
                }
                212 => {
                    if has_949 {
                        targets.push((i, j));
                    }
                }
                356 => {
                    if !fast_forward_dawn && (is_mp_client || sundial_cooldown == 0) {
                        targets.push((i, j));
                    }
                }
                663 => {
                    if !fast_forward_dusk && (is_mp_client || moondial_cooldown == 0) {
                        targets.push((i, j));
                    }
                }
                _ => {}
            }
        }
    }
}

/// `TileSmartInteractCandidateProvider.ProvideCandidate` (`:76-357`) with
/// `FullInteraction = true` and `DemandOnlyZeroDistanceTargets = false`
/// (the mouse-mode `SmartInteractLookup` settings, `Player.cs:17389-17390`).
/// Fills the lists on `state` and returns the winning candidate's
/// `(aimed_x, aimed_y, distance)` — `WinCandidacy` (`:40-56`) is the
/// caller's (the single-provider fold of `RunQuery`'s vote).
#[allow(clippy::too_many_arguments)]
fn provide_tile_candidate(
    world: &World,
    state: &mut SmartInteractState,
    mouse: (f32, f32),
    lx: i32,
    ly: i32,
    hx: i32,
    hy: i32,
    targets: &[(i32, i32)],
) -> Option<(i32, i32)> {
    // `:79` — `settings.mousevec.ToTileCoordinates()` (`Utils.cs:1807-1810`:
    // `(int)v >> 4`, the truncation-then-arithmetic-shift).
    let point = ((mouse.0 as i32) >> 4, (mouse.1 as i32) >> 4);
    // `:81-103` — the closest-target vote: tile centre `*16 + 8`, the
    // `num5 == -1f || num6 <= num5` sentinel (LATER targets win ties), then
    // the reach-rect gate on the winner.
    let mut num = -1;
    let mut num2 = -1;
    if !targets.is_empty() {
        let mut best_d = -1.0f32;
        let mut best = targets[0];
        for &(tx, ty) in targets {
            let cx = tx as f32 * 16.0 + 8.0;
            let cy = ty as f32 * 16.0 + 8.0;
            let d = ((cx - mouse.0).powi(2) + (cy - mouse.1).powi(2)).sqrt();
            if best_d == -1.0 || d <= best_d {
                best_d = d;
                best = (tx, ty);
            }
        }
        if in_tile_bounds(best.0, best.1, lx, ly, hx, hy) {
            num = best.0;
            num2 = best.1;
        }
    }
    // `:104-338` — the per-target size/anchor walk filling the lists.
    let mut flag = false; // a selected object was already found (`:104`)
    for &(tx, ty) in targets {
        let Some(tile) = world.tile(tx, ty) else { continue };
        let Some((w, h, pad)) = interact_tile_size(tile.kind) else {
            continue; // `:295-298`
        };
        // `:299-300` — the top-left anchor: the frame modulo the object's
        // pixel stride, back to tiles.
        let ax = tx - (tile.frame_x as i32) % (18 * w) / 18;
        let ay = ty - (tile.frame_y as i32) % (18 * h + pad) / 18;
        // `:301-302` — the winner inside this object (`flag2`) and the
        // mouse tile inside it (`flag3`).
        let mut flag2 = in_tile_bounds(num, num2, ax, ay, ax + w - 1, ay + h - 1);
        let flag3 = in_tile_bounds(point.0, point.1, ax, ay, ax + w - 1, ay + h - 1);
        // `:308-311` — `FullInteraction` is true, so no `flag2 &= flag3`.
        // `:312-315` — only the FIRST selected object keeps its selection.
        if flag {
            flag2 = false;
        }
        // `:316-333` — the cell fill: dedup against the coords list; the
        // selected list only for the winning object, the coords list for
        // every object (FullInteraction).
        for k in ax..ax + w {
            for l in ay..ay + h {
                let p = (k, l);
                if !state.coords.contains(&p) {
                    if flag2 {
                        state.selected.push(p);
                    }
                    state.coords.push(p);
                }
            }
        }
        // `:334-337` — `if (!flag & flag2) flag = true`.
        if !flag && flag2 {
            flag = true;
        }
    }
    // `:339-348` — DemandOnlyZeroDistanceTargets is false (mouse mode).
    // `:349-356` — the genuine candidate: the winner's 16 px cell rect
    // distance (`Utils.ClosestPointInRect` + `Vector2.Distance`,
    // `Utils.cs:1581-1601`).
    if num != -1 && num2 != -1 {
        Some((num, num2))
    } else {
        None
    }
}

/// `Player.SmartInteractLookup` (`Player.cs:17374-17398`) +
/// `SmartInteractLookup_PrepareCommonlyUsedInfo` (`:17400-17408`) +
/// `SmartInteractSystem.RunQuery` (`SmartInteractSystem.cs:31-61`) with the
/// tile provider only (module deviations). `mouse` is `Main.MouseWorld`
/// (no gravity flip in the port), `tile_target` the CURRENT
/// `Player.tileTargetX/Y` (post-smart-cursor re-point — the block provider
/// reads it, `BlockBecauseYouAreOverAnImportantTile.cs:9-10`).
pub(crate) fn smart_interact_lookup(
    app: &mut App,
    mouse: (f32, f32),
    tile_target: (i32, i32),
) {
    let state = &mut app.smart_interact;
    // `:17376` — ClearSmartInteract first, every tick.
    state.clear_smart_interact();
    // `:17377-17380` — the gamepad/navigator selected-clear: no port state.
    // `:17381-17385` — `(!ShowGamepadHints && !SmartCursorIsUsed) ||
    // !mouseItem.IsAir`: the port has no gamepad hints, so the gate is the
    // runtime smart-cursor flag + an empty cursor stack.
    let mouse_item_air = app.mouse_item.kind == 0 || app.mouse_item.count == 0;
    if !crate::world_sim::smart_cursor::enabled(app) || !mouse_item_air {
        state.clear_self_and_prepare();
        return;
    }
    let Some(player) = app.player.as_ref() else {
        state.clear_self_and_prepare();
        return;
    };
    // `:17402-17407` — the Simple reach region + the 10-tile world clamps.
    let (lx, ly, hx, hy) = reachable_rect(
        (player.x, player.y),
        crate::player::WIDTH as i32,
        player.height as i32,
        0,
    );
    let (w, h) = (app.world.width, app.world.height);
    let lx = lx.clamp(10, w - 10);
    let hx = hx.clamp(10, w - 10);
    let ly = ly.clamp(10, h - 10);
    let hy = hy.clamp(10, h - 10);
    // `RunQuery`'s Clear (`SmartInteractSystem.cs:33` → the provider's
    // `ClearSelfAndPrepareForCheck`, `:63-74`).
    state.clear_self_and_prepare();
    // `BlockBecauseYouAreOverAnImportantTile` (`:7-25`): InWorld fluff 10
    // on the tile target + the DisableSmartInteract set.
    let (ttx, tty) = tile_target;
    if ttx < 10 || ttx >= w - 10 || tty < 10 || tty >= h - 10 {
        return;
    }
    if app
        .world
        .tile(ttx, tty)
        .is_some_and(|t| t.active() && td::SET_DISABLE_SMART_INTERACT.has(t.kind))
    {
        return;
    }
    // The provider's inputs: HasItem over the 58-slot inventory (the port
    // carries the 50 main grid — coins/ammo never hold 1293/949), the
    // snapping predicate and the dial cooldowns.
    let has = |kind: u16| app.inventory.iter().any(|s| s.kind == kind && s.count > 0);
    let has_1293 = has(1293);
    let has_949 = has(949);
    let (pcx, pcy) = player.center();
    let (_, pbottom) = player.bottom();
    let snapping = |i: i32, j: i32| {
        let dx = i as f32 * 16.0 + 8.0 - pcx;
        let dy = j as f32 * 16.0 + 8.0 - (pbottom - 16.0);
        (dx * dx + dy * dy).sqrt() <= 40.0
    };
    let mut targets = Vec::new();
    fill_potential_target_tiles(
        &app.world,
        lx,
        ly,
        hx,
        hy,
        has_1293,
        has_949,
        snapping,
        app.world.fast_forward_to_dawn,
        app.world.fast_forward_to_dusk,
        app.world.sundial_cooldown,
        app.world.moondial_cooldown,
        false, // `Main.netMode == 1` — the port is singleplayer.
        &mut targets,
    );
    let state = &mut app.smart_interact;
    let won = provide_tile_candidate(&app.world, state, mouse, lx, ly, hx, hy, &targets);
    // `WinCandidacy` (`:40-56`): the genuine win sets the aimed cell + the
    // interaction rect padded by 10 (`:52-55`); the strict (zero-distance)
    // arm is gamepad-only and never runs here.
    if let Some((ax, ay)) = won {
        state.aimed_x = ax;
        state.aimed_y = ay;
        state.showing_genuine = true;
        state.lx = lx - 10;
        state.ly = ly - 10;
        state.hx = hx + 10;
        state.hy = hy + 10;
    }
}

#[cfg(test)]
mod w4_g86_feed_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A 120x120 world, the player at tile (35, 40), a 3x2 altar (26) with
    /// its top-left at (40, 38) — every cell framed like `Framing` would.
    fn altar_app() -> App {
        let mut w = World::new("w".into(), 120, 120);
        w.frame_important = vec![false; td::TILE_COUNT];
        for dx in 0..3 {
            for dy in 0..2 {
                let i = w.index(40 + dx, 38 + dy);
                w.tiles[i] = Tile {
                    kind: 26,
                    frame_x: (dx * 18) as i16,
                    frame_y: (dy * 18) as i16,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 35.0 * 16.0;
        player.y = 40.0 * 16.0;
        app.player = Some(player);
        app.smart_cursor_used = true;
        app
    }

    /// The altar's six cells all enter the coords list; the mouse over the
    /// object selects all six; the interaction rect is the reach ± 10.
    #[test]
    fn w4_g86_the_altar_fills_six_cells_and_selects_under_the_mouse() {
        let mut app = altar_app();
        // The mouse at the altar's centre cell (41, 38).
        let mouse = (41.0 * 16.0 + 8.0, 38.0 * 16.0 + 8.0);
        smart_interact_lookup(&mut app, mouse, (41, 38));
        let s = &app.smart_interact;
        assert_eq!(s.coords.len(), 6, "the 3x2 altar's cells");
        for dx in 0..3 {
            for dy in 0..2 {
                assert!(s.coords.contains(&(40 + dx, 38 + dy)));
                assert!(s.selected.contains(&(40 + dx, 38 + dy)));
            }
        }
        assert!(s.showing_genuine);
        assert_eq!((s.aimed_x, s.aimed_y), (41, 38));
        // The reach rect around (35, 40): x 30..41, y 37..43 (width 20,
        // height 42 → ceil rows 40..42), padded ±10.
        assert_eq!((s.lx, s.ly, s.hx, s.hy), (20, 27, 51, 53));
    }

    /// The mouse OFF the altar: the coords list still fills
    /// (FullInteraction), the selected list stays empty.
    #[test]
    fn w4_g86_the_off_object_mouse_fills_coords_but_not_selected() {
        let mut app = altar_app();
        let mouse = (35.0 * 16.0 + 8.0, 39.0 * 16.0 + 8.0);
        smart_interact_lookup(&mut app, mouse, (35, 39));
        let s = &app.smart_interact;
        assert_eq!(s.coords.len(), 6);
        assert!(s.selected.is_empty(), "no object under the mouse");
        assert!(s.showing_genuine, "the nearest target still wins");
    }

    /// The runtime smart-cursor flag off / a loaded cursor stack: the
    /// lists clear and stay empty (`Player.cs:17381-17385`).
    #[test]
    fn w4_g86_the_gates_clear_the_lists() {
        let mut app = altar_app();
        app.smart_cursor_used = false;
        smart_interact_lookup(&mut app, (656.0, 616.0), (41, 38));
        assert!(app.smart_interact.coords.is_empty());
        assert_eq!(app.smart_interact.lx, -1, "the rect resets to -1");
        let mut app = altar_app();
        app.mouse_item = crate::item::ItemSlot { kind: 1, name: "x", count: 1, prefix: 0 };
        smart_interact_lookup(&mut app, (656.0, 616.0), (41, 38));
        assert!(app.smart_interact.coords.is_empty(), "mouseItem not air");
    }

    /// The block provider: a DisableSmartInteract tile under the tile
    /// target refuses the whole query (`BlockBecauseYouAreOverAnImportantTile`).
    #[test]
    fn w4_g86_the_important_tile_block_refuses_the_query() {
        let mut app = altar_app();
        // A torch (4) at the target cell — in DisableSmartInteract
        // (`TileID.cs:510`).
        let i = app.world.index(35, 39);
        app.world.tiles[i] = Tile { kind: 4, ..Default::default() };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        smart_interact_lookup(&mut app, (656.0, 616.0), (35, 39));
        assert!(app.smart_interact.coords.is_empty());
        assert!(!app.smart_interact.showing_genuine);
    }

    /// The size switch's spot rows (`:115-289`): the altar (3,2,2), the
    /// chest (2,2,2), the door (1,3,0), the campfire override (3,2,2), the
    /// work bench (2,1,2) and the unswitched kind's None.
    #[test]
    fn w4_g86_the_size_switch_rows_match_the_csharp() {
        assert_eq!(interact_tile_size(26), Some((3, 2, 2)));
        assert_eq!(interact_tile_size(21), Some((2, 2, 2)));
        assert_eq!(interact_tile_size(10), Some((1, 3, 0)));
        assert_eq!(interact_tile_size(215), Some((3, 2, 2)));
        assert_eq!(interact_tile_size(18), Some((2, 1, 2)));
        assert_eq!(interact_tile_size(15), Some((1, 2, 4)));
        assert_eq!(interact_tile_size(464), Some((5, 4, 2)));
        assert_eq!(interact_tile_size(0), None);
        assert_eq!(interact_tile_size(695), None, "the replica never feeds");
    }

    /// Every HasOutlines id has a size arm (the census: no orphan outline
    /// tile) and the five sized-but-not-outlined ids stay out of the set.
    #[test]
    fn w4_g86_every_has_outlines_tile_has_a_size_arm() {
        for kind in 0..td::TILE_COUNT as u16 {
            if td::SET_HAS_OUTLINES.has(kind) {
                assert!(
                    interact_tile_size(kind).is_some(),
                    "HasOutlines {kind} has no feed size arm"
                );
            }
        }
        for kind in [14u16, 469, 491, 573, 597] {
            assert!(interact_tile_size(kind).is_some());
            assert!(!td::SET_HAS_OUTLINES.has(kind));
        }
    }
}
