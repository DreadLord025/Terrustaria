//! W4-g56 — the Void Bag's projectile half: the Void Lens (734) open/close
//! interaction and the `voidLensChest` tracker.
//!
//! C# pieces ported here:
//! * `Main.TryInteractingWithVoidLens` (`Main.cs:37673-37745`) — the
//!   draw-time hover/right-click gate `DrawProjDirect`'s type-734 arm calls
//!   (`:27924`): in-range + hitbox-hover → the cursor-icon/`noThrow` arm;
//!   the right press toggles `Player.chest = -5` (the void vault) with
//!   `SoundID.Item130`, `Main.stackSplit = 600` and the tracker latch.
//! * `Player.HandleBeingInChestRange`'s void-lens arm
//!   (`Player.cs:31019-31076`) — the per-tick close: a dead/absent tracked
//!   lens or an out-of-range hitbox point force-closes with Item130; the
//!   `chest != -5` / `chest == -1` clears (`:31023-31026`/`:31095`).
//! * `TrackedProjectileReference` (`DataStructures/
//!   TrackedProjectileReference.cs:5-71`) + `ProjectileKey`
//!   (`DataStructures/ProjectileKey.cs:16-31`) — the (spawner, index,
//!   generation) key the tracker latches; the port packs the same bits.
//!
//! Out (recorded): the smart-cursor arms (`Main.SmartCursorIsUsed` /
//! `SmartInteractProj` — unported, so selection mode 2 never fires), the
//! gamepad arms (`PlayerInput.UsingGamepad`), `Player.cursorItemIcon*`
//! (no port cursor-icon surface — the `noThrow` half lands), the
//! `tileInteractAttempted/Happened`/`releaseUseTile` writes (the port's
//! press-edge consume is `ui_right_press`/`place_prev_down`), and
//! `ItemSlot.SetGlowForChest` (the glow-array resize the port's glow system
//! does not carry — the `ui/chest.rs` module doc's same skip).

use glam::Vec2;
use tr_core::TILE_PX;

use crate::app::App;
use crate::projectile::{Projectile, VOID_LENS};

/// `ProjectileKey.Pack` (`ProjectileKey.cs:27-32`):
/// `(spawner & 0xFF) | ((index & 0x3FF) << 8) | ((generation & 0x3FFF) << 18)`.
pub(crate) fn pack_key(spawner: u8, index: usize, generation: u16) -> i32 {
    ((spawner as u32 & 0xFF)
        | ((index as u32 & 0x3FF) << 8)
        | ((generation as u32 & 0x3FFF) << 18)) as i32
}

/// `ProjectileKey`'s unpack (`ProjectileKey.cs:16-20`).
pub(crate) fn unpack_key(key: i32) -> (u8, usize, u16) {
    let bits = key as u32;
    (
        (bits & 0xFF) as u8,
        ((bits >> 8) & 0x3FF) as usize,
        ((bits >> 18) & 0x3FFF) as u16,
    )
}

/// `TrackedProjectileReference` (`TrackedProjectileReference.cs:5-20`):
/// the default (key 0, type 0) is "not tracking" — `Clear()` writes it.
/// The port keeps the struct on `Player` (`voidLensChest`,
/// `Player.cs:2874`); `net.rs::TrackedProjectileRef` is the wire mirror.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LensTracker {
    pub key: i32,
    pub kind: i16,
}

impl LensTracker {
    /// `Set(proj)` (`:11-15`).
    pub fn set(&mut self, spawner: u8, index: usize, p: &Projectile) {
        self.key = pack_key(spawner, index, p.generation);
        self.kind = p.kind as i16;
    }

    /// `Clear()` (`:17-20`) — `this = default`.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `Key.TryGetActive(734, out proj)` (`ProjectileKey.cs:100-108`):
    /// the index/generation must still address an ACTIVE 734. Returns the
    /// slot index.
    pub fn try_get_active(&self, pool: &crate::projectile::ProjectilePool) -> Option<usize> {
        if *self == Self::default() {
            return None;
        }
        let (_, index, generation) = unpack_key(self.key);
        let p = pool.slots().get(index)?;
        (p.active && p.generation == generation && p.kind == VOID_LENS).then_some(index)
    }
}

/// `Player.IsProjectileInteractableAndInInteractionRange`
/// (`Player.cs:23153-23169`) for the lens: active + `IsInteractable`
/// (`Projectile.cs:78833` — 734 is in the set) + the hitbox's closest
/// point to the player centre, tile-truncated, inside the Simple reach
/// region (`IsInTileInteractionRange`, `:32025-32029` — the same live
/// `tile_interaction_reach` bounds `chest.rs`'s range close reads).
pub(crate) fn lens_in_interaction_range(p: &crate::player::Player, proj: &Projectile) -> bool {
    if !proj.active || proj.kind != VOID_LENS {
        return false;
    }
    let (cx, cy) = p.center();
    // `Utils.ClosestPointInRect` (`Utils.cs:1581-1595`) — the two
    // sequential clamps (the chest.rs range-close precedent).
    let px = cx.clamp(proj.pos.x, proj.pos.x + proj.w);
    let py = cy.clamp(proj.pos.y, proj.pos.y + proj.h);
    let tx = (px / TILE_PX) as i32;
    let ty = (py / TILE_PX) as i32;
    let (reach_x, reach_y) = p.tile_interaction_reach();
    let lx = (p.x / TILE_PX) as i32 - reach_x;
    let hx = ((p.x + crate::player::WIDTH) / TILE_PX).ceil() as i32 - 1 + reach_x;
    let ly = (p.y / TILE_PX) as i32 - reach_y;
    let hy = ((p.y + p.height) / TILE_PX).ceil() as i32 - 1 + reach_y;
    (lx..=hx).contains(&tx) && (ly..=hy).contains(&ty)
}

/// `Main.TryInteractingWithVoidLens` (`Main.cs:37673-37745`) — the
/// draw-time interaction gate. Returns the selection mode (0 none / 1
/// hoverable-but-not-hovered — the smart-cursor arm's answer the port
/// never draws / 2 selected, unported). The right-press toggle fires
/// HERE, exactly like the C# (the draw call is the interaction site).
///
/// `mouse_interface` is `Player.lastMouseInterface` (`Player.cs:1454`,
/// latched at `Main.cs:61811` — LAST frame's `mouseInterface`); the port
/// snapshots it on the App at the interface-draw tail.
pub(crate) fn try_interacting(app: &mut App, index: usize) -> u8 {
    // `:37675-37678` — `gamePaused || gameMenu` refuses. The port's world
    // draw only runs in-world (`frame.rs:443`'s menu/front gate covers
    // gameMenu); the pause flag is live.
    if app.game_paused {
        return 0;
    }
    let Some(p) = app.player.as_ref() else {
        return 0;
    };
    if p.dead {
        // The C# reads `IsProjectileInteractableAndInInteractionRange`
        // only; a dead player's reach still answers, but the interface
        // is hidden — the port's HUD gate keeps the draw off anyway.
        return 0;
    }
    let proj = app.projectiles.slots()[index];
    if !lens_in_interaction_range(p, &proj) {
        return 0; // `:37682-37685`
    }
    // `:37686-37687` — `ReverseGravitySupport(MouseScreen) + screenPosition`
    // (the gravDir −1 mirror has no port) → the world mouse point; the
    // hitbox `Contains` is the `(int)`-truncated point test.
    let (sw, sh) = app.screen_size();
    let world_mouse = app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom;
    let mx = world_mouse.x as i32;
    let my = world_mouse.y as i32;
    let hovered = mx >= proj.pos.x as i32
        && mx < (proj.pos.x + proj.w) as i32
        && my >= proj.pos.y as i32
        && my < (proj.pos.y + proj.h) as i32;
    // `:37688-37695` — `(flag2 || SmartInteractProj == whoAmI) &
    // !lastMouseInterface`; the smart-cursor half is unported (false).
    if !hovered || app.last_mouse_interface {
        return 0;
    }
    // `:37696-37702` — `HasInteractableObjectThatIsNotATile` (no port
    // state), `noThrow = 2` + the cursor-icon 4131 (the icon surface is
    // unported; the throw lockout lands).
    app.no_throw = 2;
    // `:37707-37735` — the right press (`Main.mouseRight &&
    // Main.mouseRightRelease` — the port's press edge;
    // `Player.BlockInteractionWithProjectiles` is net-only, 0 here).
    if app.place_down && !app.place_prev_down {
        app.ui_right_press = false; // the press is consumed (`mouseRightRelease = false`)
        let chest = app.player.as_ref().map_or(-1, |p| p.chest);
        if chest == -5 {
            // `:37713-37717` — the re-click closes, Item130.
            if let Some(p) = app.player.as_mut() {
                p.chest = -1;
            }
            app.audio.play("Item_130");
        } else {
            // `:37718-37734` — `disableVoidBag < 0`: the net-slot lock refuses
            // the open (W4-g56 — the flag is ported now, MP-only; SP keeps −1
            // and always opens). stackSplit 600, chest −5, the tracker latch,
            // the tile point off the lens CENTRE, the talk/shop clears, the
            // inventory open, Item130.
            let locked = app
                .player
                .as_ref()
                .is_some_and(|p| p.disable_void_bag >= 0);
            if locked {
                return 0; // `:37718` — the `else if` falls through, no open
            }
            app.stack_split = 600; // `:37720`
            let (pcx, pcy) = (proj.pos.x + proj.w * 0.5, proj.pos.y + proj.h * 0.5);
            let tx = (pcx / TILE_PX) as i32;
            let ty = (pcy / TILE_PX) as i32;
            app.close_shop(); // `:37731` — `SetNPCShopIndex(0)`'s close half
            app.talk_npc = None; // `:37730` — `SetTalkNPC(-1)`
            app.npc_chat_text.clear();
            app.crafting_open = false; // `OpenChest`'s `NewCraftingUI.Close`
            app.inventory_open = true; // `:37732`
            if let Some(p) = app.player.as_mut() {
                p.chest = -5; // `:37721`
                p.chest_x = tx; // `:37728-37729`
                p.chest_y = ty;
                p.void_lens_chest.set(app.local_id as u8, index, &proj); // `:37727`
            }
            app.audio.play("Item_130"); // `:37733`
        }
    }
    // `:37736-37744` — the smart-cursor tail: mode 2 only under the
    // unported smart cursor / gamepad; the mouse path answers 0.
    0
}

/// W4-g56 — `Main.HoveringAnInteractable`'s lens half (`Main.cs:2861-2870`:
/// `HoveringOverAnNPC || SmartInteractShowingGenuine ||
/// HasInteractableObjectThatIsNotATile`; the NPC/smart-cursor halves have no
/// port state). `TryInteractingWithVoidLens` sets the non-tile flag when an
/// in-range lens is hovered with `!lastMouseInterface` (`:37688-37696`); the
/// held-item right-click swap gate (`Player.cs:31528`'s `!mouseRightRelease
/// || !HoveringAnInteractable`) reads it so a press over a lens opens the
/// vault instead of flipping the held bag. This probe re-derives the same
/// hover condition without the side effects (the draw pass owns those).
pub(crate) fn hovering_lens(app: &App) -> bool {
    if app.game_paused || app.last_mouse_interface {
        return false;
    }
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    if p.dead {
        return false;
    }
    let (sw, sh) = app.screen_size();
    let world_mouse = app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom;
    let mx = world_mouse.x as i32;
    let my = world_mouse.y as i32;
    for proj in app.projectiles.slots() {
        if !proj.active || proj.kind != VOID_LENS {
            continue;
        }
        if !lens_in_interaction_range(p, proj) {
            continue;
        }
        let hovered = mx >= proj.pos.x as i32
            && mx < (proj.pos.x + proj.w) as i32
            && my >= proj.pos.y as i32
            && my < (proj.pos.y + proj.h) as i32;
        if hovered {
            return true;
        }
    }
    false
}

/// The per-frame draw-side pass: run `try_interacting` for every active
/// lens slot (the C# calls it once per lens inside `DrawProjDirect`),
/// storing the selection modes for the draw loop. Runs at the draw head,
/// before the field destructure.
pub(crate) fn update_selection(app: &mut App) {
    app.void_lens_selection.clear();
    let n = app.projectiles.slots().len();
    app.void_lens_selection.resize(n, 0);
    for i in 0..n {
        if app.projectiles.slots()[i].active && app.projectiles.slots()[i].kind == VOID_LENS {
            let mode = try_interacting(app, i);
            app.void_lens_selection[i] = mode;
        }
    }
}

/// `Player.HandleBeingInChestRange`'s void-lens half
/// (`Player.cs:31023-31026` + `:31051-31073` + `:31095`), called from
/// `chest::tick_open` ahead of the tile-range close. Returns the legacy
/// sound id to play (130 = Item130, 0 = none) — the caller plays it, the
/// `force_close` precedent.
pub(crate) fn tick_tracker(app: &mut App) -> u32 {
    let Some(p) = app.player.as_ref() else {
        return 0;
    };
    if p.chest == -1 {
        // `:31092-31096` — both trackers clear while closed.
        if let Some(p) = app.player.as_mut() {
            p.void_lens_chest.clear();
        }
        return 0;
    }
    if p.chest != -5 {
        // `:31023-31026` — another container owns the open state.
        if let Some(p) = app.player.as_mut() {
            p.void_lens_chest.clear();
        }
        return 0;
    }
    let tracker = p.void_lens_chest;
    if tracker == LensTracker::default() {
        return 0;
    }
    // `:31054-31058` — the tracked lens died: Item130 + close.
    let Some(index) = tracker.try_get_active(&app.projectiles) else {
        if let Some(p) = app.player.as_mut() {
            p.chest = -1;
            p.void_lens_chest.clear();
        }
        return 130;
    };
    // `:31061-31063` — the hitbox's closest point to the player centre
    // re-seeds chestX/Y every tick.
    let proj = app.projectiles.slots()[index];
    let (cx, cy) = app.player.as_ref().unwrap().center();
    let px = cx.clamp(proj.pos.x, proj.pos.x + proj.w);
    let py = cy.clamp(proj.pos.y, proj.pos.y + proj.h);
    let tx = (px / TILE_PX) as i32;
    let ty = (py / TILE_PX) as i32;
    if let Some(p) = app.player.as_mut() {
        p.chest_x = tx;
        p.chest_y = ty;
    }
    // `:31064-31071` — the Simple reach test on that point.
    let in_range = {
        let p = app.player.as_ref().unwrap();
        let (reach_x, reach_y) = p.tile_interaction_reach();
        let lx = (p.x / TILE_PX) as i32 - reach_x;
        let hx = ((p.x + crate::player::WIDTH) / TILE_PX).ceil() as i32 - 1 + reach_x;
        let ly = (p.y / TILE_PX) as i32 - reach_y;
        let hy = ((p.y + p.height) / TILE_PX).ceil() as i32 - 1 + reach_y;
        (lx..=hx).contains(&tx) && (ly..=hy).contains(&ty)
    };
    if !in_range {
        if let Some(p) = app.player.as_mut() {
            p.chest = -1;
            p.void_lens_chest.clear();
        }
        return 130;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn lens_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = tr_core::World::new("t".into(), 200, 200);
        app.player = Some(Player::spawn(&app.world));
        app
    }

    /// Spawn a lens whose CENTRE sits at `(cx, cy)` world px.
    fn spawn_lens(app: &mut App, cx: f32, cy: f32) -> usize {
        app.projectiles
            .spawn_with_data_ai_owned(VOID_LENS, 0, cx - 15.0, cy - 12.0, 0.0, 0.0, 0, 0.0, 0.0, 0.0)
            .expect("a free slot")
    }

    /// Point the mouse at a world position (the camera-relative screen
    /// mapping `try_interacting` inverts).
    fn aim(app: &mut App, world: Vec2) {
        let (sw, sh) = app.screen_size();
        app.mouse_pos = (world - app.camera) * app.zoom + Vec2::new(sw * 0.5, sh * 0.5);
    }

    /// `ProjectileKey.Pack`/unpack (`ProjectileKey.cs:16-31`): the bit
    /// layout round-trips, the generation mask is 14 bits.
    #[test]
    fn w4_g56_the_key_pack_matches_the_csharp_layout() {
        let key = pack_key(7, 300, 0x1234);
        assert_eq!(key as u32, 7 | (300 << 8) | (0x1234 << 18));
        assert_eq!(unpack_key(key), (7, 300, 0x1234));
        // The masks truncate: index 10 bits, generation 14.
        assert_eq!(unpack_key(pack_key(0, 0x7FF, 0x7FFF)), (0, 0x3FF, 0x3FFF));
    }

    /// `TryInteractingWithVoidLens` (`Main.cs:37707-37734`): the right
    /// press on a hovered in-range lens opens chest −5 with stackSplit
    /// 600, the tracker latch, the tile point off the lens centre, the
    /// inventory open and the talk/shop clears; the re-click closes.
    /// Both transitions play Item130.
    #[test]
    fn w4_g56_the_right_press_toggles_the_vault_open_and_closed() {
        let mut app = lens_app();
        let (px, py) = app.player.as_ref().unwrap().center();
        let idx = spawn_lens(&mut app, px + 20.0, py);
        aim(&mut app, Vec2::new(px + 20.0, py));
        app.place_down = true;
        app.place_prev_down = false;
        let before = app.audio.played_count();
        let mode = try_interacting(&mut app, idx);
        assert_eq!(mode, 0, "the mouse path answers 0 (:37736-37744)");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -5, ":37721");
        assert_eq!(p.chest_x, ((px + 20.0) / 16.0) as i32, ":37728");
        assert_eq!(p.chest_y, (py / 16.0) as i32, ":37729");
        assert_ne!(p.void_lens_chest, LensTracker::default(), ":37727 — the tracker latches");
        assert_eq!(p.void_lens_chest.kind, 734);
        assert_eq!(app.stack_split, 600, ":37720");
        assert!(app.inventory_open, ":37732");
        assert_eq!(app.no_throw, 2, ":37699");
        assert_eq!(app.audio.played_count(), before + 1, "Item130 (:37733)");
        assert!(!app.ui_right_press, "the press is consumed");
        // The re-click closes (:37713-37717), Item130 again.
        app.place_prev_down = false;
        let before = app.audio.played_count();
        try_interacting(&mut app, idx);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        assert_eq!(app.audio.played_count(), before + 1);
    }

    /// W4-g56 — the lens open's `disableVoidBag` guard (`Main.cs:37718`):
    /// a locked vault slot (`disable_void_bag >= 0`) refuses the open — no
    /// chest −5, no stackSplit, no cue, no tracker latch. The press is still
    /// consumed (`mouseRightRelease = false`, `:37709`, ahead of the branch).
    /// The lock is MP-only; the witness arms it directly (the net producer is
    /// unported). Clearing the lock re-opens.
    #[test]
    fn w4_g56_the_lens_open_refuses_while_the_vault_slot_is_locked() {
        let mut app = lens_app();
        let (px, py) = app.player.as_ref().unwrap().center();
        let idx = spawn_lens(&mut app, px + 20.0, py);
        aim(&mut app, Vec2::new(px + 20.0, py));
        app.place_down = true;
        app.place_prev_down = false;
        app.player.as_mut().unwrap().lock_net_slot(3); // bank4 slot 3 locked
        let before = app.audio.played_count();
        let mode = try_interacting(&mut app, idx);
        assert_eq!(mode, 0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -1, "the locked vault never opens (:37718)");
        assert_eq!(p.void_lens_chest, LensTracker::default(), "no tracker latch");
        assert_eq!(app.stack_split, 0, "no stackSplit on the refused open");
        assert_eq!(app.audio.played_count(), before, "no Item130");
        assert!(!app.ui_right_press, "the press is still consumed (:37709)");
        // The slot-5 echo clears the lock; the next press opens.
        app.player.as_mut().unwrap().clear_void_bag_lock_on_echo(3);
        assert_eq!(app.player.as_ref().unwrap().disable_void_bag, -1);
        app.place_prev_down = false;
        try_interacting(&mut app, idx);
        assert_eq!(app.player.as_ref().unwrap().chest, -5, "the cleared lock opens");
    }

    /// `:37682-37695` — out of the Simple reach, or the pointer off the
    /// hitbox, or lastMouseInterface: no interaction, no toggle.
    #[test]
    fn w4_g56_the_gates_refuse_out_of_range_off_hitbox_and_interface() {
        let mut app = lens_app();
        let (px, py) = app.player.as_ref().unwrap().center();
        // Far away: the reach gate refuses (:37682-37685).
        let idx = spawn_lens(&mut app, px + 400.0, py);
        aim(&mut app, Vec2::new(px + 400.0, py));
        app.place_down = true;
        app.place_prev_down = false;
        try_interacting(&mut app, idx);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        // In range but the pointer misses the box: no toggle (:37688).
        let idx2 = spawn_lens(&mut app, px + 20.0, py);
        aim(&mut app, Vec2::new(px + 60.0, py));
        try_interacting(&mut app, idx2);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        // Hovered but lastMouseInterface: refused (:37688's second half).
        aim(&mut app, Vec2::new(px + 20.0, py));
        app.last_mouse_interface = true;
        try_interacting(&mut app, idx2);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        // Held button (no press edge): no toggle either.
        app.last_mouse_interface = false;
        app.place_prev_down = true;
        try_interacting(&mut app, idx2);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
    }

    /// `HandleBeingInChestRange`'s lens arm (`Player.cs:31051-31073`):
    /// a dead tracked lens force-closes with 130; an out-of-range hitbox
    /// point closes too; in range stays open and re-seeds chestX/Y.
    #[test]
    fn w4_g56_the_tracker_tick_closes_on_death_and_range() {
        let mut app = lens_app();
        let (px, py) = app.player.as_ref().unwrap().center();
        let idx = spawn_lens(&mut app, px + 20.0, py);
        aim(&mut app, Vec2::new(px + 20.0, py));
        app.place_down = true;
        app.place_prev_down = false;
        try_interacting(&mut app, idx);
        assert_eq!(app.player.as_ref().unwrap().chest, -5);
        // In range: stays open, no cue.
        assert_eq!(tick_tracker(&mut app), 0);
        assert_eq!(app.player.as_ref().unwrap().chest, -5);
        // The lens dies: 130 + close (:31054-31058).
        app.projectiles.kill(idx);
        assert_eq!(tick_tracker(&mut app), 130);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
        assert_eq!(
            app.player.as_ref().unwrap().void_lens_chest,
            LensTracker::default(),
            "the close clears the tracker"
        );
        // Re-open, then walk out of the Simple reach: 130 + close
        // (:31064-31071). The lens stays alive.
        let idx2 = spawn_lens(&mut app, px + 20.0, py);
        aim(&mut app, Vec2::new(px + 20.0, py));
        app.place_prev_down = false;
        try_interacting(&mut app, idx2);
        assert_eq!(app.player.as_ref().unwrap().chest, -5);
        app.player.as_mut().unwrap().x += 400.0;
        assert_eq!(tick_tracker(&mut app), 130);
        assert_eq!(app.player.as_ref().unwrap().chest, -1);
    }

    /// `:31019-31026`/`:31095` — the tracker clears when another container
    /// owns the open state or the chest is closed; a tile-vault open
    /// (no tracker) never rides the lens close.
    #[test]
    fn w4_g56_the_tracker_clears_on_other_containers_and_tile_opens() {
        let mut app = lens_app();
        let (px, py) = app.player.as_ref().unwrap().center();
        let idx = spawn_lens(&mut app, px + 20.0, py);
        aim(&mut app, Vec2::new(px + 20.0, py));
        app.place_down = true;
        app.place_prev_down = false;
        try_interacting(&mut app, idx);
        // Another container takes the open state: the tracker clears,
        // no cue (:31023-31026).
        app.player.as_mut().unwrap().chest = -2;
        assert_eq!(tick_tracker(&mut app), 0);
        assert_eq!(
            app.player.as_ref().unwrap().void_lens_chest,
            LensTracker::default()
        );
        // A tile-vault open (chest −5, NO tracker — the BankOpen arm) is
        // not a lens open: the tick answers 0 and the tile-range close
        // owns it (the chest.rs `tracked_open` gate).
        app.player.as_mut().unwrap().chest = -5;
        assert_eq!(tick_tracker(&mut app), 0);
        assert_eq!(app.player.as_ref().unwrap().chest, -5);
    }

    /// `IsProjectileInteractableAndInInteractionRange`
    /// (`Player.cs:23153-23169`): the hitbox's closest point decides the
    /// reach — a lens whose box edge is in range answers true even when
    /// its centre tile is outside the ±5/±3 region.
    #[test]
    fn w4_g56_the_range_probe_clamps_to_the_hitbox() {
        let mut app = lens_app();
        let p = app.player.as_ref().unwrap();
        let (px, py) = p.center();
        // The box spans [px+90, px+120]: the closest point px+90 clamps
        // into the ±5 reach even though the centre px+105 sits further.
        let idx = spawn_lens(&mut app, px + 105.0, py);
        let proj = app.projectiles.slots()[idx];
        assert!(lens_in_interaction_range(p, &proj), "the clamped edge is in reach");
        // A dead lens never answers (:23155-23158).
        app.projectiles.kill(idx);
        let proj = app.projectiles.slots()[idx];
        assert!(!lens_in_interaction_range(app.player.as_ref().unwrap(), &proj));
    }
}