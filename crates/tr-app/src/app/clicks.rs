//! The App's HUD click handling — the slot/hotbar hit tests, the minimap
//! frame's buttons, the inventory stack moves and the legacy-menu action
//! dispatch. (A6: extracted from `main.rs`; the front screens' click tables
//! live in [`crate::app::flow`].)

use winit::event_loop::ActiveEventLoop;

use crate::craft;
use crate::app::flow::{FlowClick, FrontScreen, next_flow_state};
use crate::app::netglue::compressed_world;
use crate::app::{App, NetRole, no_hud};
use crate::item::{EMPTY_SLOT, HOTBAR, ItemSlot, MAX_STACK};
use crate::menu;
use crate::achievements;
use crate::net;
use crate::ui;
use glam::Vec2;

impl App {
    /// The slot under the cursor, using the same geometry as `draw_hud`.
    pub(crate) fn slot_at_mouse(&self) -> Option<usize> {
        let m = self.mouse_pos;
        if self.inventory_open {
            let s = 0.85;
            let i = ((m.x - 20.0) / (56.0 * s)).floor() as i32;
            let j = ((m.y - 20.0) / (56.0 * s)).floor() as i32;
            if !(0..10).contains(&i) || !(0..5).contains(&j) {
                return None;
            }
            let lx = m.x - 20.0 - i as f32 * 56.0 * s;
            let ly = m.y - 20.0 - j as f32 * 56.0 * s;
            if lx > 52.0 * s || ly > 52.0 * s {
                return None;
            }
            Some((i + j * 10) as usize)
        } else {
            // `GUIHotbarDrawInner` (`Main.cs:43614-43639`): same per-slot geometry
            // as the draw pass — row `y` `(int)(20 + 22 * (1 - scale))`, x advance
            // `(int)(Width * scale) + 4`, hover size the untruncated `Width * scale`
            // (`ui::slot::closed_hotbar_metrics`).
            let mut x = 20.0;
            for i in 0..HOTBAR {
                let sc = self.hotbar_scales[i].clamp(0.75, 1.0);
                let (y, advance, size) = ui::slot::closed_hotbar_metrics(sc);
                if m.x >= x && m.x <= x + size && m.y >= y && m.y <= y + size {
                    return Some(i);
                }
                x += advance;
            }
            None
        }
    }

    /// `MinimapFrameManager.DrawTo` -> `MinimapFrame.Update`
    /// (`MinimapFrameManager.cs:45`, `MinimapFrame.cs:138-177`): the hover +
    /// held-click handling of the three zoom buttons, run every frame while the
    /// minimap HUD is live (the same gate as the draw branch). Clicks repeat
    /// every frame while the button is held (`:169-171`); `apply` mutates the
    /// minimap scale (`:107-136`). The resulting hover also feeds the
    /// `draw_foreground` highlight (the source's `IsHighlighted`, `:163`) and
    /// the mine/place suppression (`:164`'s `mouseInterface`).
    ///
    /// Deviation: `:167,174` play `SoundEngine.PlaySound(12)` on hover changes
    /// and click releases — the port loads no UI cue (`audio.rs:22-31`), so it
    /// stays silent, consistent with the menus.
    pub(crate) fn handle_minimap_clicks(&mut self) {
        // The buttons exist only while the frame draws (`Main.cs:55546`'s
        // `DrawTo` gate): `mapEnabled && !mapFullscreen && mapStyle == 1`,
        // plus the port's screenshot `no_hud` knob that hides the draw too.
        let live = self.settings.map_enabled
            && !no_hud()
            && self.map.as_ref().is_some_and(|m| !m.open)
            && self.map_style == 1;
        if !live {
            self.minimap_hover = None;
            return;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        // The frame position from the same formula as the draw branch
        // (`Main.cs:55546`'s `DrawTo` position + `_frameOffset`).
        let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin(screen));
        let s = ui::minimap::style_index(&self.settings.minimap_frame);
        let frame_pos = ui::minimap::frame_position(pos, &ui::minimap::FRAME_STYLES[s]);
        let sizes = ui::minimap::button_sizes(&self.lookups, s);
        // `Main.mouseLeft` is the held state (`MinimapFrame.cs:169`).
        let action = ui::minimap::update(
            self.mouse_pos,
            frame_pos,
            &ui::minimap::FRAME_STYLES[s],
            sizes,
            self.mine_down,
        );
        // `buttonUnderMouse.IsHighlighted = true` (`:163`); a click implies it.
        self.minimap_hover = match action {
            ui::minimap::MinimapAction::None => None,
            ui::minimap::MinimapAction::Hover(b) | ui::minimap::MinimapAction::Click(b) => Some(b),
        };
        // `buttonUnderMouse.Click()` -> the zoom handlers (`:171`, `:107-136`);
        // `apply` is a no-op unless the action is a click.
        if let Some(m) = self.map.as_mut() {
            ui::minimap::apply(action, &mut m.minimap_scale);
        }
    }

    /// `ItemSlot.Handle` (context 0) subset: left click takes, places or swaps a whole
    /// stack, right click takes half or places one; on the closed hotbar a left click
    /// selects the slot like the original.
    /// `NewCraftingUI`'s recipe clicks: a click on an available recipe cell
    /// crafts it — `try_craft` consumes the ingredients and the result goes to
    /// the inventory via `slot_for_pickup` (a full inventory drops on the
    /// ground in the original; the drop bridge is not wired yet, so a full
    /// inventory stacks only what fits — see docs/ANALYSIS.md). Consumes the
    /// press only when a recipe was crafted.
    pub(crate) fn handle_crafting_clicks(&mut self) -> bool {
        if !self.crafting_open || !self.ui_left_press {
            return false;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let Some(player) = self.player.as_ref() else { return false; };
        let adj = craft::adjacent_tiles(&self.world, player);
        let available = craft::available(&self.inventory, &adj);
        let Some(idx) = ui::crafting::hit_test(screen, &self.lookups, self.mouse_pos, &available)
        else {
            return false;
        };
        let recipe = &craft::RECIPES[idx];
        let Some((kind, count)) = craft::try_craft(&mut self.inventory, recipe, &adj) else {
            // The recipe row shows with a partial ingredient set; a click
            // without the full set does nothing (`Main.cs:42250` hover only).
            return false;
        };
        if let Some(i) = crate::item::slot_for_pickup(&mut self.inventory, kind) {
            self.inventory[i].count = (self.inventory[i].count + count).min(crate::registry::max_stack(kind));
        }
        // `AchievementsHelper.NotifyItemCraft` — the station craft feeds the
        // ItemCraftCondition rows (BENCHED).
        let _ = self
            .achievements
            .notify(achievements::CondKind::ItemCraft);
        self.ui_left_press = false;
        true
    }

    /// The bestiary screen's clicks: the inventory button that opens it
    /// (`Main.cs:42455-42462` — `SetTalkNPC(-1)`/`npcChatCornerItem = 0` have
    /// no port counterpart), then the grid cards and the Back panel while it
    /// is open. Consumes the press on every hit.
    pub(crate) fn handle_bestiary_clicks(&mut self) {
        let left = self.ui_left_press;
        if !left {
            return;
        }
        if self.bestiary_open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            // The Back panel closes the screen (`MakeExitButton` ->
            // `Click_GoBack`); the inventory stays closed, like the source's
            // `IngameFancyUI` return.
            if ui::bestiary::back_hit(screen, self.mouse_pos) {
                self.bestiary_open = false;
                self.bestiary_selected = None;
                self.ui_left_press = false;
                return;
            }
            // A card opens its info page (`Click_SelectEntryButton`).
            if let Some(idx) = ui::bestiary::card_at(screen, self.mouse_pos) {
                self.bestiary_selected = Some(idx);
                self.ui_left_press = false;
                return;
            }
            return;
        }
        // The inventory button (`DrawBestiaryIcon`'s click arm): open the
        // bestiary, close the inventory (`IngameFancyUI.OpenUIState`).
        if self.inventory_open && ui::bestiary::button_hit(self.mouse_pos) {
            self.inventory_open = false;
            self.bestiary_open = true;
            self.bestiary_selected = None;
            self.ui_left_press = false;
            return;
        }
        // `DrawEmoteBubblesButton`'s click arm (`Main.cs:42400-42405`):
        // open the emotes wheel.
        if self.inventory_open && ui::bestiary::emote_button_hit(self.mouse_pos) {
            self.inventory_open = false;
            self.emotes.menu_open = true;
            self.ui_left_press = false;
            return;
        }
        // The wheel: a pick fires `MakeLocalPlayerEmote` (`:281-286`) and
        // closes; the Back panel just closes.
        if self.emotes.menu_open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            if let Some(id) = ui::emotes::card_at(screen, self.mouse_pos, self.emotes.scroll) {
                self.emotes.new_bubble(id);
                self.emotes.menu_open = false;
                self.ui_left_press = false;
                return;
            }
            if ui::bestiary::back_hit(screen, self.mouse_pos) {
                self.emotes.menu_open = false;
                self.ui_left_press = false;
            }
        }
    }

    pub(crate) fn handle_inventory_clicks(&mut self) {
        let left = self.ui_left_press;
        let right = self.ui_right_press;
        self.ui_left_press = false;
        self.ui_right_press = false;
        if !left && !right {
            return;
        }
        // `Main.cs:41087-41120`: the map-style icon row's clicks — one-shot on
        // the press (`Main.mouseLeft && Main.mouseLeftRelease`), only while the
        // inventory is open (their home is `DrawInventory`, `:41055`) and the
        // map system is enabled (`:41056`'s `Main.mapEnabled` wraps the row).
        // A hit consumes the press and returns before the slot handling.
        // `PlaySound(10)/(12)` (`:41100,41109-41119`) is not ported — no UI cue
        // is loaded (`audio.rs:22-31`).
        if left && self.inventory_open && self.settings.map_enabled {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            for k in 0..4usize {
                let [x, y] = ui::chrome::map_icon_rect(k, screen);
                // `Main.cs:41087`: inclusive on all four edges.
                if self.mouse_pos.x >= x
                    && self.mouse_pos.x <= x + ui::chrome::MAP_ICON_HIT[0]
                    && self.mouse_pos.y >= y
                    && self.mouse_pos.y <= y + ui::chrome::MAP_ICON_HIT[1]
                {
                    if k == 0 {
                        // `Main.cs:41095-41104`: close the inventory, open the
                        // fullscreen map. `set_open(true)` already resets the
                        // scale to `2.5` and arms the recentre (`Main.cs:41101,
                        // 41104`, `crates/tr-app/src/map.rs:451-457`);
                        // `SetTalkNPC(-1)` / `npcChatCornerItem = 0` have no
                        // port counterpart.
                        self.inventory_open = false;
                        if let Some(m) = self.map.as_mut() {
                            m.set_open(true);
                        }
                    } else {
                        // `Main.cs:41106-41120`: `k == 1/2/3` ->
                        // `Main.mapStyle = 0/1/2`.
                        self.map_style = k as i32 - 1;
                    }
                    return;
                }
            }
        }
        let Some(slot) = self.slot_at_mouse() else {
            return;
        };
        if !self.inventory_open {
            if left {
                self.selected = slot;
            }
            return;
        }
        let mut held = self.mouse_item;
        let mut target = self.inventory[slot];
        if left {
            if held.count == 0 {
                held = target;
                target = EMPTY_SLOT;
            } else if target.count == 0 {
                target = held;
                held = EMPTY_SLOT;
            } else if target.kind == held.kind && target.count < MAX_STACK {
                let moved = (MAX_STACK - target.count).min(held.count);
                target.count += moved;
                held.count -= moved;
                if held.count == 0 {
                    held = EMPTY_SLOT;
                }
            } else {
                std::mem::swap(&mut held, &mut target);
            }
        } else if held.count == 0 {
            if target.count > 0 {
                let take = (target.count + 1) / 2;
                held = ItemSlot {
                    kind: target.kind,
                    name: target.name,
                    count: take,
                };
                target.count -= take;
                if target.count == 0 {
                    target = EMPTY_SLOT;
                }
            }
        } else if target.count == 0 {
            target = ItemSlot {
                kind: held.kind,
                name: held.name,
                count: 1,
            };
            held.count -= 1;
            if held.count == 0 {
                held = EMPTY_SLOT;
            }
        } else if target.kind == held.kind && target.count < MAX_STACK {
            target.count += 1;
            held.count -= 1;
            if held.count == 0 {
                held = EMPTY_SLOT;
            }
        }
        self.mouse_item = held;
        self.inventory[slot] = target;
    }

    /// Applies a menu action (start singleplayer, host on a port, join a server).
    pub(crate) fn apply_menu_action(&mut self, action: menu::Action, event_loop: &ActiveEventLoop) {
        use menu::Action;
        match action {
            Action::None => {}
            Action::Quit => {
                self.save_world();
                event_loop.exit();
            }
            Action::Singleplayer => {
                // `Main.cs:48981-48987`: menuMode 1 -> `OpenCharacterSelectUI`
                // (888) — the real screen flow replaces the old "Single
                // Player closes the menu" shortcut.
                self.enter_front(FrontScreen::CharSelect);
            }
            // The confirm pages (`Main.cs:49420-49439`, `:49674-49692`,
            // `:49440-49467`): the resolution runs the erase on Yes, then
            // reopens the screen the confirm came from.
            Action::ConfirmYes | Action::ConfirmNo => {
                let yes = matches!(action, menu::Action::ConfirmYes);
                let (page, index) = self
                    .menu
                    .as_ref()
                    .map_or((None, 0usize), |m| (Some(m.page), m.confirm_index));
                let click = if yes { FlowClick::ConfirmYes } else { FlowClick::ConfirmNo };
                // Clones: `FrontScreen` is no longer `Copy`, and the erase
                // calls below need `self` whole.
                let (front, backup, _) =
                    next_flow_state(self.front.clone(), self.front_backup.clone(), page, click);
                // Mode 40's Yes drops the parked state (`Main.cs:49455`:
                // `BackupConfirmationState = null`); the flow function returns
                // `None` for it, and No's restore value for the reopen.
                self.front_backup = backup;
                if yes && page == Some(menu::Page::ConfirmDeletePlayer) {
                    // `Main.ErasePlayer` (`:49432-49434` -> `:4772-4791`).
                    self.erase_player_entry(index);
                }
                if yes && page == Some(menu::Page::ConfirmDeleteWorld) {
                    // `Main.EraseWorld` (`:49682-49684` -> `:4796-4813`).
                    self.erase_world_entry(index);
                }
                match (page, front) {
                    // Mode 40's No restores the exact parked screen without
                    // the `OnActivate` re-list (`menuMode = 888` +
                    // `SetState(BackupConfirmationState)`, `:49460-49467`).
                    (Some(menu::Page::ConfirmDiscard), Some(f)) if !yes => {
                        self.menu = None;
                        self.front = Some(f);
                    }
                    // Modes 5/9 (both rows) and 40's Yes re-enter through the
                    // openers, which run the `OnActivate` re-list
                    // (`menuMode = 1/6` -> `OpenCharacterSelectUI` /
                    // `OpenWorldSelectUI`).
                    (_, Some(f)) => self.enter_front(f),
                    // Unreachable from the confirm pages (every branch
                    // navigates); keep the invariant by closing both sides.
                    (_, None) => {
                        self.front = None;
                        self.park_menu();
                    }
                }
            }
            Action::Host { port, password } => {
                let map = compressed_world(&self.world);
                match net::Server::start(port, password, map) {
                    Ok(server) => {
                        log::info!("hosting on {}", server.addr());
                        self.net = NetRole::Host(server);
                        self.close_menu();
                    }
                    Err(e) => {
                        if let Some(m) = self.menu.as_mut() {
                            m.connecting = false;
                            m.error = Some(format!("host failed: {e}"));
                        }
                    }
                }
            }
            Action::Join { addr, password } => match net::Client::connect(
                &addr,
                self.player_name.clone(),
                password.clone(),
            ) {
                Ok(client) => {
                    self.net = NetRole::Client(client);
                    self.join_target = Some((addr, password));
                }
                Err(e) => {
                    if let Some(m) = self.menu.as_mut() {
                        m.connecting = false;
                        m.error = Some(format!("connect failed: {e}"));
                    }
                }
            },
        }
    }

    /// Leaves the menu and applies the settings it edited (`Main.SaveSettings`).
    pub(crate) fn close_menu(&mut self) {
        if let Some(m) = self.menu.take() {
            self.settings = m.settings;
        }
        self.apply_settings();
    }

    /// Takes the legacy menu into the App without the launch semantics — the
    /// settings sync of [`App::close_menu`] for every hop where a front screen
    /// takes over or the world launches.
    pub(crate) fn park_menu(&mut self) {
        if let Some(m) = self.menu.take() {
            self.settings = m.settings;
        }
        self.apply_settings();
    }
}
